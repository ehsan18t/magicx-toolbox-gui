//! The tweak engine's lifecycle (spec §4/§8), plus the `Setting → EffectKind` dispatcher that
//! decides, per `ExecCx::level()`, whether a drive runs in-process or through the elevation broker
//! (see [`AllKinds::drive`] and `kinds/mod.rs`'s module docs for why the decision lives here).
//!
//! Two `Deps` fields are worth naming:
//! - `level` is the plain [`Level`] the process actually holds. Reads never escalate (spec
//!   invariant 24), so this is a ceiling for reads, not a target.
//! - `running` is the live [`WinVer`] (build + revision, spec §6.6). Call sites that only need the
//!   build-only shape go through [`WinVer::to_milestone`]; see `winver.rs` for the reconciliation.

pub mod apply;
pub mod context;
pub mod detect;
pub mod lifecycle;
pub mod revert;

use apply::EngineError;

use crate::services::appx_index::AppxIndex;
use crate::services::elevation::{self, BrokerOp, BrokerOpError, Elevation, OpFailureClass};
use crate::tweaks::kinds::{
    action::ActionKind,
    firewall::FirewallKind,
    hosts::HostsKind,
    registry::{self, RegistryKind},
    service::{self, ServiceKind},
    task::{self, TaskKind},
    BatchFailure, EffectKind, Error as KindError, ExecCx,
};
use crate::tweaks::model::{ActionDef, EffectId, Level, Setting, Value};
use crate::tweaks::shared_claims::{ClaimsError, ClaimsStore};
use crate::tweaks::snapshot::SnapshotStore;
use crate::tweaks::winver::WinVer;
use std::collections::HashMap;
use std::sync::Mutex;

/// Production `Setting → EffectKind` dispatcher: delegates by `Setting` variant at `User`/`Admin`,
/// and routes through the elevation broker at `System`/`Ti` (spec §9, see [`drive_via_broker`]).
/// Stateless, so it is trivially `Send + Sync` and cheap to construct per call.
pub struct AllKinds;

impl AllKinds {
    /// The typed ops one elevated Setting drive becomes, including the existence pre-check.
    ///
    /// Split out of `drive` so `drive_batch` can translate a whole run before spawning anything,
    /// and so both paths perform the pre-check identically.
    ///
    /// The pre-check mirrors what `drive_service`/`drive_task` do in-process (invariant 12). The
    /// broker translations are deliberately pure, and without this an absent service or task
    /// reaches the child, fails there, and returns as an opaque `OpFailed` -> `ElevatedOpFailed`.
    /// That is the wrong shape twice over: apply's `optional`/`if_missing` no-op guard matches only
    /// `ResourceMissing`, so an `optional` effect whose resource is absent on this build would
    /// abort the tweak and roll it back instead of reading as the verified no-op detect already
    /// advertises. Reads never escalate, so this costs the same in-process read detect just
    /// performed, and it means an absent resource never spawns a child at all.
    fn broker_ops_for(
        &self,
        s: &Setting,
        target: &Value,
        cx: &ExecCx,
    ) -> Result<Vec<BrokerOp>, KindError> {
        let level = cx.level();
        if !matches!(target, Value::Missing)
            && matches!(s, Setting::Service(_) | Setting::Task(_))
            && self.read(s, cx)? == Value::Missing
        {
            return Err(KindError::ResourceMissing(match s {
                Setting::Service(addr) => format!("service '{}' does not exist", addr.name),
                Setting::Task(addr) => format!("scheduled task '{}' does not exist", addr.path),
                _ => unreachable!("guarded by the matches! above"),
            }));
        }

        match s {
            Setting::Registry(_) | Setting::RegistryKey(_) => {
                Ok(vec![registry::to_broker_op(s, target, level)?])
            }
            Setting::Service(_) => service::to_broker_ops(s, target),
            Setting::Task(_) => task::to_broker_ops(s, target),
            // No BrokerOp exists for Hosts/Firewall in this build (spec §9's mechanical translation
            // list does not cover them). The in-process kinds refuse System/Ti themselves, so this
            // is the same refusal, raised one layer earlier.
            Setting::Hosts(_) | Setting::Firewall(_) => Err(KindError::UnsupportedLevel(level)),
        }
    }
}

impl EffectKind for AllKinds {
    fn read(&self, s: &Setting, cx: &ExecCx) -> Result<Value, KindError> {
        // Reads never escalate (invariant 24) -- always in-process, regardless of `cx.level()`.
        match s {
            Setting::Registry(_) | Setting::RegistryKey(_) => RegistryKind.read(s, cx),
            Setting::Service(_) => ServiceKind.read(s, cx),
            Setting::Task(_) => TaskKind.read(s, cx),
            Setting::Hosts(_) => HostsKind.read(s, cx),
            Setting::Firewall(_) => FirewallKind.read(s, cx),
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), KindError> {
        match cx.level() {
            Level::User | Level::Admin => match s {
                Setting::Registry(_) | Setting::RegistryKey(_) => RegistryKind.drive(s, target, cx),
                Setting::Service(_) => ServiceKind.drive(s, target, cx),
                Setting::Task(_) => TaskKind.drive(s, target, cx),
                Setting::Hosts(_) => HostsKind.drive(s, target, cx),
                Setting::Firewall(_) => FirewallKind.drive(s, target, cx),
            },
            level @ Level::Ti => drive_via_broker(level, self.broker_ops_for(s, target, cx)?),
        }
    }

    /// One elevated child for a whole run of TrustedInstaller steps instead of one per effect.
    ///
    /// Acquiring TrustedInstaller is the expensive part and it is entirely per-spawn: connect to
    /// the SCM, query and possibly start the service, poll for it, open and verify its process,
    /// build the attribute list, cold-start this executable again, and round-trip JSON through the
    /// filesystem. None of that gets cheaper for being done twenty times. Each spawn is also a
    /// separate behavioural event for whatever security product is watching, which is its own
    /// reason not to repeat it.
    ///
    /// Below `Ti`, and for a run of one, this defers to the trait default: the per-effect loop.
    fn drive_batch(&self, items: &[(&Setting, &Value)], cx: &ExecCx) -> Result<(), BatchFailure> {
        if cx.level() != Level::Ti || items.len() < 2 {
            for (index, (setting, target)) in items.iter().enumerate() {
                self.drive(setting, target, cx)
                    .map_err(|error| BatchFailure {
                        index,
                        error,
                        completed: index,
                    })?;
            }
            return Ok(());
        }

        // Translate first, keeping each effect's op span, so a failure the child reports by op
        // index can be attributed back to the effect that produced it. Translation performs the
        // same existence pre-check `drive` does, so an absent resource still refuses before
        // anything is spawned.
        let mut ops: Vec<BrokerOp> = Vec::new();
        let mut spans: Vec<std::ops::Range<usize>> = Vec::with_capacity(items.len());
        for (index, (setting, target)) in items.iter().enumerate() {
            let start = ops.len();
            match self.broker_ops_for(setting, target, cx) {
                Ok(mut translated) => ops.append(&mut translated),
                Err(error) => {
                    // Translation refuses before anything is spawned, so no earlier item ran either.
                    return Err(BatchFailure {
                        index,
                        error,
                        completed: 0,
                    });
                }
            }
            spans.push(start..ops.len());
        }

        if ops.is_empty() {
            return Ok(());
        }
        elevation::run_ops(Elevation::TrustedInstaller, ops).map_err(|e| match e {
            // Acquisition failed, so no op ran. Attributing it to the first item is the truthful
            // choice: it is where the batch stopped, and the error type still says plainly that
            // nothing was attempted.
            BrokerOpError::CouldNotAcquire(err) => BatchFailure {
                index: 0,
                error: KindError::CouldNotAcquireElevation(Level::Ti, err.to_string()),
                completed: 0,
            },
            ref failed @ BrokerOpError::OpFailed { ref class, .. } => {
                let named = failing_item(&spans, failed.failed_op_index());
                BatchFailure {
                    // Ops run in order and the child stops at its first failure, so an op inside a
                    // named item's span proves every item ahead of it drove. An op that names no
                    // item proves nothing about any, so the run re-drives from its start.
                    index: named.unwrap_or(spans.len().saturating_sub(1)),
                    error: KindError::ElevatedOpFailed(Level::Ti, *class),
                    completed: named.unwrap_or(0),
                }
            }
            // No op index to attribute this to: the point is that we do not know which ran. The
            // first item is where the batch is treated as having stopped.
            BrokerOpError::Indeterminate(err) => BatchFailure {
                index: 0,
                error: KindError::ElevatedOutcomeUnknown(Level::Ti, err.to_string()),
                completed: 0,
            },
        })
    }
}

/// The batch item whose translation produced `failed_op`.
///
/// `None` when the child named no op, or named one outside every span: the failure is
/// unattributable, which a caller must never read as progress through the run.
fn failing_item(spans: &[std::ops::Range<usize>], failed_op: Option<usize>) -> Option<usize> {
    let op_index = failed_op?;
    spans.iter().position(|span| span.contains(&op_index))
}

/// Submits `ops` through the elevation broker in ONE child (spec §9), keeping its three failure
/// modes apart. An empty `ops` list, which a Service/Task drive to `Missing` produces, is a
/// verified no-op that never spawns a child.
fn drive_via_broker(level: Level, ops: Vec<BrokerOp>) -> Result<(), KindError> {
    if ops.is_empty() {
        return Ok(());
    }
    elevation::run_ops(to_elevation(level), ops).map_err(|e| match e {
        BrokerOpError::CouldNotAcquire(err) => {
            KindError::CouldNotAcquireElevation(level, err.to_string())
        }
        BrokerOpError::OpFailed { class, .. } => KindError::ElevatedOpFailed(level, class),
        BrokerOpError::Indeterminate(err) => {
            KindError::ElevatedOutcomeUnknown(level, err.to_string())
        }
    })
}

/// The elevated half of the command layer's one-line-per-failure record: which tweak, where in it,
/// which level, and how the broker classified it, with the broker's own text (a temp path, the
/// transport nonce) left out. `false` when nothing here is elevated, so the caller logs instead.
pub fn log_elevated_failure(tweak_id: &str, e: &EngineError) -> bool {
    for line in elevated_failure_lines(tweak_id, e) {
        log::error!("{line}");
    }
    // A line for a rollback failure is no record of the original, which then still needs its own.
    let original = match e {
        EngineError::RollbackReport { original, .. } => original.as_ref(),
        other => other,
    };
    !elevated_failure_lines(tweak_id, original).is_empty()
}

/// Every elevated failure `e` carries, a rollback's own failures included: ADR-0001's Needs
/// Attention state is exactly a rollback that hit one, so it can never be the line that is missing.
fn elevated_failure_lines(tweak_id: &str, e: &EngineError) -> Vec<String> {
    let mut lines = Vec::new();
    collect_elevated_failures(tweak_id, e, "", &mut lines);
    lines
}

fn collect_elevated_failures(tweak_id: &str, e: &EngineError, during: &str, out: &mut Vec<String>) {
    let (site, source) = match e {
        EngineError::RollbackReport {
            original,
            rollback_failures,
            ..
        } => {
            collect_elevated_failures(tweak_id, original, during, out);
            for failed in rollback_failures {
                collect_elevated_failures(tweak_id, failed, " during rollback", out);
            }
            return;
        }
        EngineError::CaptureFailed { effect, source }
        | EngineError::DriveFailed { effect, source }
        | EngineError::ActionFailed { effect, source } => (format!("effect '{effect}'"), source),
        EngineError::Claim {
            shared,
            source: ClaimsError::Kind(source),
        } => (format!("shared '{shared}'"), source),
        _ => return,
    };
    if let Some((level, classification)) = elevated_failure(source) {
        out.push(format!(
            "tweak '{tweak_id}': {site} failed{during} at {level:?} elevation: {classification}"
        ));
    }
}

/// The three ways a failure can have reached the elevated child, named without the detail behind
/// them: that text is the broker's and can carry a path, a nonce or a registry value. The refused
/// case carries its class, which is the parent's own value and names no resource.
fn elevated_failure(e: &KindError) -> Option<(Level, String)> {
    match e {
        KindError::CouldNotAcquireElevation(level, _) => Some((
            *level,
            "could not acquire the elevated child, so nothing ran".to_owned(),
        )),
        KindError::ElevatedOpFailed(level, class) => Some((
            *level,
            format!("an operation was refused inside the elevated child: {class}"),
        )),
        KindError::ElevatedOutcomeUnknown(level, _) => Some((
            *level,
            "the elevated child ran but its outcome is unknown, so the machine may have changed"
                .to_owned(),
        )),
        _ => None,
    }
}

/// Which operation a failure is being described for: a restore has no rollback phase, so it cannot
/// borrow apply's word for what happened to the machine afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Apply,
    Restore,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Phase::Apply => "apply",
            Phase::Restore => "restore",
        })
    }
}

/// ADR-0001's Needs Attention state in the user's own words, rather than a count of "item(s)".
fn needing_attention(count: usize) -> String {
    if count == 1 {
        "one effect still needs attention".to_owned()
    } else {
        format!("{count} effects still need attention")
    }
}

/// The user-interface half of the decision recorded on `OpFailureClass`: the engine's own text
/// names the key, the value, the path or the script behind a failure, so the frontend is handed the
/// structure around it instead -- which effect, and which class of failure.
pub fn user_facing_failure(phase: Phase, e: &EngineError) -> String {
    match e {
        EngineError::UnknownOption(label) => {
            format!("'{label}' is not one of this tweak's options")
        }
        EngineError::SurfaceUnreadable(reasons) => {
            let effects: Vec<String> = reasons.iter().map(|r| format!("'{}'", r.effect)).collect();
            format!(
                "this tweak's current state is unreadable: effect(s) {}",
                effects.join(", ")
            )
        }
        EngineError::Unavailable(reason) => reason.clone(),
        EngineError::AppExiting(exiting) => exiting.to_string(),
        EngineError::CaptureFailed { effect, source } => format!(
            "effect '{effect}' could not be read before anything was changed: {}",
            kind_failure(source)
        ),
        EngineError::CaptureMissingRequired(effect) => {
            format!("effect '{effect}' is required but is not present on this machine")
        }
        EngineError::SnapshotWrite(_) => {
            "the pre-apply snapshot could not be saved, so nothing was changed".to_owned()
        }
        EngineError::DriveFailed { effect, source } => format!(
            "effect '{effect}' could not be changed: {}",
            kind_failure(source)
        ),
        EngineError::ResourceMissing(effect) => {
            format!("effect '{effect}': the target no longer exists on this machine")
        }
        EngineError::VerifyMismatch { effect, .. } => {
            format!("effect '{effect}' did not read back what was written to it")
        }
        EngineError::ActionFailed { effect, source } => {
            format!("action '{effect}' failed: {}", kind_failure(source))
        }
        EngineError::ActionVerifyMismatch { effect, .. } => {
            format!("action '{effect}' ran but did not leave the state it declares")
        }
        EngineError::JournalMark { effect, .. } => {
            format!("action '{effect}' ran but could not be recorded")
        }
        EngineError::Claim { shared, source } => {
            format!("shared setting '{shared}': {}", claims_failure(source))
        }
        EngineError::Invalid(_) => "internal engine inconsistency".to_owned(),
        EngineError::NoUndo(effect) => {
            format!("action '{effect}' ran and declares no undo, so it cannot be reversed")
        }
        EngineError::EntryCleanup(_) => {
            "the machine was restored, but its spent snapshot could not be released".to_owned()
        }
        EngineError::AttentionWrite(_) => "Needs Attention could not be recorded".to_owned(),
        EngineError::RestoreFailed { failures, store } => {
            let named: Vec<String> = failures
                .iter()
                .chain(store)
                .map(|f| user_facing_failure(phase, f))
                .collect();
            format!("{}; the snapshot was kept", named.join("; "))
        }
        EngineError::RollbackReport {
            original,
            rollback_failures,
            outcome_unknown,
            store,
        } => {
            let undone = match phase {
                Phase::Apply => "the tweak was rolled back",
                Phase::Restore => "the restore was undone",
            };
            let mut after = if !rollback_failures.is_empty() {
                format!(
                    "{undone}, but {}",
                    needing_attention(rollback_failures.len())
                )
            } else if *outcome_unknown {
                format!("{undone}, but an elevated step's outcome cannot be proven either way")
            } else {
                format!("{undone} and verified")
            };
            // Named as itself: a rollback that restored everything must not read as one that left
            // the machine unrecovered just because the store could not be written afterwards.
            for failure in store {
                after.push_str("; ");
                after.push_str(&user_facing_failure(phase, failure));
            }
            format!("{}; {after}", user_facing_failure(phase, original))
        }
    }
}

/// The class of one kind failure, never the text: `KindError`'s own `Display` quotes the key, the
/// path, or the value it was given.
fn kind_failure(e: &KindError) -> String {
    match e {
        KindError::NotFound(_) | KindError::ResourceMissing(_) => "not found".to_owned(),
        KindError::AccessDenied(_) => "access denied".to_owned(),
        KindError::TypeMismatch { .. } => {
            "the live value is stored as a different type than this tweak declares".to_owned()
        }
        KindError::MalformedPacked { .. } => "the live value could not be parsed".to_owned(),
        KindError::UnsupportedLevel(level) => {
            format!("{level} elevation is not routed by this build")
        }
        KindError::CouldNotAcquireElevation(level, _) => {
            format!("could not acquire {level} elevation, so nothing ran")
        }
        KindError::ElevatedOpFailed(level, class) => {
            format!("refused at {level} elevation: {class}")
        }
        KindError::ElevatedOutcomeUnknown(level, _) => format!(
            "{level} elevation ran but its outcome is unknown, so the machine may have changed"
        ),
        // A `&'static str` this crate wrote, so it names no live key, value or path.
        KindError::Invalid(what) => (*what).to_owned(),
        KindError::Backend(_) => OpFailureClass::Failed.to_string(),
        KindError::ActionFailed(code) => format!("its script exited with code {code}"),
        KindError::ActionExecFailed(_) => "its script could not be run".to_owned(),
    }
}

fn claims_failure(e: &ClaimsError) -> String {
    match e {
        ClaimsError::Kind(source) => kind_failure(source),
        ClaimsError::VerifyMismatch { .. } => {
            "it did not read back what was written to it".to_owned()
        }
        ClaimsError::NotHeld { .. } => "this tweak does not currently hold it".to_owned(),
        ClaimsError::Corrupt => "the shared-claims record is corrupt".to_owned(),
        ClaimsError::Io(_) | ClaimsError::ExeDir => {
            "the shared-claims record could not be read or written".to_owned()
        }
    }
}

/// Maps a tweak's declared [`Level`] to the broker's [`Elevation`]. Only ever called for
/// `System`/`Ti`; `User`/`Admin` run in-process and never reach the broker.
fn to_elevation(level: Level) -> Elevation {
    match level {
        Level::Ti => Elevation::TrustedInstaller,
        Level::User | Level::Admin => {
            unreachable!("drive_via_broker is only ever reached for System/Ti")
        }
    }
}

/// Injectable source for an Action's probe (spec §7), separate from [`EffectKind`] because Actions
/// are not Settings. Tests substitute an in-memory mock that counts invocations, so
/// `probe_cache_hit_no_respawn` can prove the cache, not the mock, is what suppresses a respawn.
pub trait ProbeSource: Send + Sync {
    fn probe(&self, action: &ActionDef, cx: &ExecCx) -> Result<bool, KindError>;
}

/// Production probe source: `ActionKind::run_probe`, unmodified.
pub struct RealProbe;

impl ProbeSource for RealProbe {
    fn probe(&self, action: &ActionDef, cx: &ExecCx) -> Result<bool, KindError> {
        ActionKind.run_probe(action, cx)
    }
}

/// Injectable source for an Action's `apply`/`undo` (spec §7). Kept separate from [`ProbeSource`]
/// because `detect` only ever probes an Action, never runs one, so it needs no behavioral mock
/// here. Without this seam `engine::apply` would call `ActionKind` directly and spawn a real
/// process in unit tests; the mock records call order instead, making apply's
/// capture-before-mutation and completion-after-each-action invariants provable with zero OS
/// contact.
pub trait ActionRunner: Send + Sync {
    fn apply(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), KindError>;
    fn undo(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), KindError>;
}

/// Production action runner: `ActionKind::run_apply`/`run_undo`, unmodified.
pub struct RealActions;

impl ActionRunner for RealActions {
    fn apply(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), KindError> {
        ActionKind.run_apply(action, cx)
    }
    fn undo(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), KindError> {
        ActionKind.run_undo(action, cx)
    }
}

/// The external dependencies of detect, apply, and restore: one injection seam, so production
/// wires real stores and kinds while tests wire in-memory mocks with zero OS contact.
pub struct Deps<'a> {
    pub kinds: &'a dyn EffectKind,
    pub probes: &'a dyn ProbeSource,
    /// The Action apply/undo seam (see [`ActionRunner`]). `detect` never reads this field.
    pub actions: &'a dyn ActionRunner,
    pub claims: &'a ClaimsStore,
    pub snapshots: &'a SnapshotStore,
    pub probe_cache: &'a ProbeCache,
    pub machine_guid: Option<&'a str>,
    pub level: Level,
    pub running: WinVer,
}

/// Per-session cache of probeable-Action present/absent readings, keyed `(tweak_id, effect_id)`
/// (spec §7: "cached per session ... detection must not re-spawn PowerShell per status poll").
/// Interior-mutable so `detect` can populate it on a miss through a shared `&ProbeCache` in `Deps`.
///
/// Also owns the shared package enumeration [`AppxIndex`], which is the same idea one level up: a
/// reading of the machine that many probes want and none should pay for separately. It lives here
/// so it is invalidated by the same call that invalidates the per-effect readings, since removing
/// an app is exactly what makes both stale.
#[derive(Default)]
pub struct ProbeCache {
    entries: Mutex<HashMap<(String, EffectId), bool>>,
    appx: AppxIndex,
}

impl ProbeCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// The sweep-wide package enumeration. Built on first ask, then shared.
    pub(crate) fn appx(&self) -> &AppxIndex {
        &self.appx
    }

    /// Private (module-private, so `engine::detect` -- a descendant -- can still call it):
    /// only engine internals read/populate the cache directly; external callers only `invalidate`.
    fn get(&self, tweak_id: &str, effect_id: &EffectId) -> Option<bool> {
        self.entries
            .lock()
            .expect("ProbeCache mutex poisoned")
            .get(&(tweak_id.to_string(), effect_id.clone()))
            .copied()
    }

    fn insert(&self, tweak_id: &str, effect_id: &EffectId, present: bool) {
        self.entries
            .lock()
            .expect("ProbeCache mutex poisoned")
            .insert((tweak_id.to_string(), effect_id.clone()), present);
    }

    /// Drops every cached probe for `tweak_id` (spec §7), so the next detect re-observes live
    /// state. Called by apply/restore after they mutate that tweak's surface, never by `detect`.
    ///
    /// The package enumeration goes with it, unconditionally rather than per-tweak: it is one
    /// machine-wide reading with no tweak to key on, and the applies that invalidate a probe are
    /// the same ones that install or remove packages.
    pub fn invalidate(&self, tweak_id: &str) {
        self.entries
            .lock()
            .expect("ProbeCache mutex poisoned")
            .retain(|(t, _), _| t != tweak_id);
        self.appx.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::{OptLabel, RegType, SharedId, StartupType, SvcAddr, TaskAddr};

    /// A detail carrying one of everything a log line must never print: a path, a transport nonce,
    /// and a registry key with its data.
    const HOSTILE_DETAIL: &str =
        "C:\\Users\\Someone\\AppData\\Local\\Temp\\magicx-broker-req.json \
         nonce 0x0123456789abcdef HKLM\\Software\\Policies\\Secret = 1";

    fn assert_no_detail(line: &str) {
        for leaked in [
            "C:\\Users",
            "magicx-broker",
            "0x0123456789abcdef",
            "HKLM",
            "Secret",
        ] {
            assert!(!line.contains(leaked), "{line} names {leaked}");
        }
    }

    /// The parent's identity line for a failed elevated drive: which tweak, which effect, which
    /// level, which classification, and none of the detail behind it.
    #[test]
    fn every_elevated_drive_failure_is_named_by_tweak_effect_level_and_classification() {
        for (source, classification) in [
            (
                KindError::CouldNotAcquireElevation(Level::Ti, HOSTILE_DETAIL.into()),
                "nothing ran",
            ),
            (
                KindError::ElevatedOpFailed(Level::Ti, OpFailureClass::InvalidData),
                "refused inside the elevated child: invalid data",
            ),
            (
                KindError::ElevatedOutcomeUnknown(Level::Ti, HOSTILE_DETAIL.into()),
                "outcome is unknown",
            ),
        ] {
            let lines = elevated_failure_lines(
                "wu_block",
                &EngineError::DriveFailed {
                    effect: EffectId("block_updates".into()),
                    source,
                },
            );
            let [line] = &lines[..] else {
                panic!("one line per elevated failure, got {lines:?}");
            };
            assert!(line.contains("wu_block"), "{line}");
            assert!(line.contains("block_updates"), "{line}");
            assert!(line.contains("Ti"), "{line}");
            assert!(line.contains(classification), "{line}");
            assert_no_detail(line);
        }
    }

    /// A drive is not the only step that reaches the elevated child: a shared claim drives through
    /// it too, and capture and action failures carry the same kinds.
    #[test]
    fn every_shape_that_can_carry_an_elevated_failure_is_named() {
        let unknown = || KindError::ElevatedOutcomeUnknown(Level::Ti, HOSTILE_DETAIL.into());
        let effect = || EffectId("wu_sih".into());
        for shape in [
            EngineError::Claim {
                shared: SharedId("wu_service".into()),
                source: ClaimsError::Kind(unknown()),
            },
            EngineError::CaptureFailed {
                effect: effect(),
                source: unknown(),
            },
            EngineError::ActionFailed {
                effect: effect(),
                source: unknown(),
            },
        ] {
            let lines = elevated_failure_lines("wu_block", &shape);
            let [line] = &lines[..] else {
                panic!("{shape:?} went unnamed");
            };
            assert!(line.contains("outcome is unknown"), "{line}");
            assert_no_detail(line);
        }
    }

    /// A rollback report is what the command layer actually receives, and an elevated failure
    /// inside the rollback itself is ADR-0001's Needs Attention state: both have to be named.
    #[test]
    fn a_rollback_report_names_the_original_and_every_rollback_failure() {
        let lines = elevated_failure_lines(
            "wu_block",
            &EngineError::RollbackReport {
                original: Box::new(EngineError::DriveFailed {
                    effect: EffectId("wu_sih".into()),
                    source: KindError::ElevatedOpFailed(Level::Ti, OpFailureClass::AccessDenied),
                }),
                rollback_failures: vec![EngineError::DriveFailed {
                    effect: EffectId("wu_orch".into()),
                    source: KindError::ElevatedOutcomeUnknown(Level::Ti, HOSTILE_DETAIL.into()),
                }],
                outcome_unknown: true,
                store: Vec::new(),
            },
        );
        let [original, rolled_back] = &lines[..] else {
            panic!("both failures must be named, got {lines:?}");
        };
        assert!(original.contains("wu_sih"), "{original}");
        assert!(!original.contains("rollback"), "{original}");
        assert!(rolled_back.contains("wu_orch"), "{rolled_back}");
        assert!(rolled_back.contains("during rollback"), "{rolled_back}");
        assert_no_detail(original);
        assert_no_detail(rolled_back);
    }

    /// A failure that never reached the broker must not be recorded as an elevated one.
    #[test]
    fn a_failure_that_never_reached_the_broker_is_not_an_elevated_one() {
        for e in [
            EngineError::DriveFailed {
                effect: EffectId("e".into()),
                source: KindError::AccessDenied("denied in-process".into()),
            },
            EngineError::Claim {
                shared: SharedId("s".into()),
                source: ClaimsError::Corrupt,
            },
            EngineError::UnknownOption(OptLabel("On".into())),
        ] {
            assert!(elevated_failure_lines("wu_block", &e).is_empty(), "{e:?}");
        }
    }

    /// A line for a rollback failure is no record of the original, so an in-process original under
    /// an elevated rollback failure must still send the caller to its own full-text line.
    #[test]
    fn an_unelevated_original_is_not_covered_by_an_elevated_rollback_failure() {
        let elevated = || EngineError::DriveFailed {
            effect: EffectId("wu_orch".into()),
            source: KindError::ElevatedOutcomeUnknown(Level::Ti, HOSTILE_DETAIL.into()),
        };
        let in_process = || EngineError::DriveFailed {
            effect: EffectId("wu_sih".into()),
            source: KindError::AccessDenied("denied in-process".into()),
        };

        assert!(
            !log_elevated_failure(
                "wu_block",
                &EngineError::RollbackReport {
                    original: Box::new(in_process()),
                    rollback_failures: vec![elevated()],
                    outcome_unknown: false,
                    store: Vec::new(),
                }
            ),
            "the original went unrecorded"
        );
        assert!(log_elevated_failure(
            "wu_block",
            &EngineError::RollbackReport {
                original: Box::new(elevated()),
                rollback_failures: vec![in_process()],
                outcome_unknown: false,
                store: Vec::new(),
            }
        ));
    }

    /// The other side of the log's guarantee: what the frontend is handed carries the effect and
    /// the class, and none of the text the failure itself came with.
    #[test]
    fn what_the_frontend_is_handed_never_quotes_the_failure_text() {
        let effect = || EffectId("wu_sih".into());
        let hostile = || EngineError::DriveFailed {
            effect: effect(),
            source: KindError::NotFound(HOSTILE_DETAIL.into()),
        };
        for e in [
            hostile(),
            EngineError::CaptureFailed {
                effect: effect(),
                source: KindError::AccessDenied(HOSTILE_DETAIL.into()),
            },
            EngineError::ActionFailed {
                effect: effect(),
                source: KindError::ActionExecFailed(HOSTILE_DETAIL.into()),
            },
            EngineError::DriveFailed {
                effect: effect(),
                source: KindError::Backend(HOSTILE_DETAIL.into()),
            },
            EngineError::DriveFailed {
                effect: effect(),
                source: KindError::TypeMismatch {
                    path: HOSTILE_DETAIL.into(),
                    name: "Secret".into(),
                    expected: RegType::Dword,
                    actual: RegType::Dword,
                },
            },
            EngineError::Claim {
                shared: SharedId("wu_sih".into()),
                source: ClaimsError::Kind(KindError::AccessDenied(HOSTILE_DETAIL.into())),
            },
            EngineError::Invalid(HOSTILE_DETAIL.into()),
            EngineError::RollbackReport {
                original: Box::new(hostile()),
                rollback_failures: vec![hostile()],
                outcome_unknown: false,
                store: vec![EngineError::AttentionWrite(
                    crate::tweaks::snapshot::SnapshotError::ExeDir,
                )],
            },
        ] {
            let shown = user_facing_failure(Phase::Apply, &e);
            assert_no_detail(&shown);
            assert!(!shown.is_empty(), "{e:?} rendered as nothing");
        }
    }

    /// A verify mismatch is the one shape whose own text is a registry value rather than a key.
    #[test]
    fn a_verify_mismatch_never_shows_the_value_it_read_back() {
        let shown = user_facing_failure(
            Phase::Apply,
            &EngineError::VerifyMismatch {
                effect: EffectId("wu_sih".into()),
                expected: Value::Present(true),
                actual: Value::Present(false),
            },
        );
        assert!(shown.contains("wu_sih"), "{shown}");
        assert!(!shown.contains("Present"), "{shown}");
    }

    /// The Needs Attention record is persisted and then printed verbatim in the card tooltip and the
    /// details modal, so it is held to the error channel's rule: the structure around the failure,
    /// never the text the failure came with.
    #[test]
    fn a_persisted_attention_item_never_quotes_the_failure_text() {
        let effect = || EffectId("wu_sih".into());
        for e in [
            EngineError::DriveFailed {
                effect: effect(),
                source: KindError::AccessDenied(HOSTILE_DETAIL.into()),
            },
            EngineError::CaptureFailed {
                effect: effect(),
                source: KindError::NotFound(HOSTILE_DETAIL.into()),
            },
            EngineError::ActionFailed {
                effect: effect(),
                source: KindError::ActionExecFailed(HOSTILE_DETAIL.into()),
            },
            EngineError::VerifyMismatch {
                effect: effect(),
                expected: Value::Present(true),
                actual: Value::Present(false),
            },
            EngineError::Claim {
                shared: SharedId("wu_sih".into()),
                source: ClaimsError::Kind(KindError::AccessDenied(HOSTILE_DETAIL.into())),
            },
            EngineError::JournalMark {
                effect: effect(),
                source: crate::tweaks::snapshot::SnapshotError::ExeDir,
            },
        ] {
            for phase in [Phase::Apply, Phase::Restore] {
                let item = apply::attention_item(phase, &e);
                assert_no_detail(&item.message);
                // `VerifyMismatch`'s own text, the one shape whose raw form quotes both values.
                assert!(
                    !item.message.contains("verify mismatch"),
                    "{}",
                    item.message
                );
                assert!(!item.message.contains("Present"), "{}", item.message);
                assert_eq!(item.message, user_facing_failure(phase, &e));
            }
        }
    }

    /// A restore has no rollback phase, so it never borrows apply's word for one, and neither
    /// phase counts "item(s)" at the user.
    #[test]
    fn a_restore_failure_is_never_described_as_a_rollback() {
        let failed = || EngineError::DriveFailed {
            effect: EffectId("wu_sih".into()),
            source: KindError::AccessDenied(HOSTILE_DETAIL.into()),
        };
        let report = |n: usize| EngineError::RollbackReport {
            original: Box::new(failed()),
            rollback_failures: (0..n).map(|_| failed()).collect(),
            outcome_unknown: false,
            store: Vec::new(),
        };

        let apply = user_facing_failure(Phase::Apply, &report(1));
        assert!(apply.contains("the tweak was rolled back"), "{apply}");
        assert!(
            apply.contains("one effect still needs attention"),
            "{apply}"
        );

        let restore = user_facing_failure(Phase::Restore, &report(2));
        assert!(!restore.contains("roll"), "{restore}");
        assert!(
            restore.contains("2 effects still need attention"),
            "{restore}"
        );

        for shown in [
            apply,
            restore,
            user_facing_failure(Phase::Restore, &report(0)),
        ] {
            assert!(!shown.contains("item"), "{shown}");
            assert_no_detail(&shown);
        }
    }

    /// User copy spells the level out; "Ti" is the internal name and belongs in the log only.
    #[test]
    fn user_copy_never_prints_the_internal_level_name() {
        for source in [
            KindError::ElevatedOpFailed(Level::Ti, OpFailureClass::Busy),
            KindError::CouldNotAcquireElevation(Level::Ti, HOSTILE_DETAIL.into()),
            KindError::ElevatedOutcomeUnknown(Level::Ti, HOSTILE_DETAIL.into()),
            KindError::UnsupportedLevel(Level::Ti),
        ] {
            let shown = user_facing_failure(
                Phase::Apply,
                &EngineError::DriveFailed {
                    effect: EffectId("wu_sih".into()),
                    source,
                },
            );
            assert!(shown.contains("TrustedInstaller elevation"), "{shown}");
            assert!(!shown.contains("Ti elevation"), "{shown}");
        }
    }

    /// The generic class has to compose inside the frame that already says "failed".
    #[test]
    fn an_unclassified_failure_does_not_double_its_own_wording() {
        let shown = user_facing_failure(
            Phase::Apply,
            &EngineError::DriveFailed {
                effect: EffectId("wu_sih".into()),
                source: KindError::ElevatedOpFailed(Level::Ti, OpFailureClass::Failed),
            },
        );
        assert!(shown.ends_with("for an unclassified reason"), "{shown}");
        assert!(!shown.contains("failed: failed"), "{shown}");
    }

    /// Names that certainly do not exist, so these need no elevation and no real resource.
    const NO_SUCH_SERVICE: &str = "MagicXNoSuchService_5F3F1D2E-6A4B-4C9E-9B0A-6B6E6C7D8E9F";
    const NO_SUCH_TASK: &str = r"\MagicXNoSuchFolder_5F3F1D2E\NoSuchTask";

    /// An `optional` effect whose resource is absent has to reach apply as `ResourceMissing`: that
    /// is the only error its `if_missing` no-op guard matches on (`apply::drive_forward`). Without
    /// the pre-check the child sees the absence instead and it comes back opaque, which aborts and
    /// rolls back a tweak whose task is simply not on this Windows build.
    ///
    /// The pre-check runs before any spawn, which is exactly why this test needs no elevation.
    #[test]
    fn an_absent_resource_refuses_as_resource_missing_at_system_and_ti() {
        let level = Level::Ti;
        let cx = ExecCx::new(level);

        let svc = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        let err = AllKinds
            .drive(&svc, &Value::Startup(StartupType::Manual), &cx)
            .expect_err("an absent service must refuse, not reach the broker");
        assert!(
            matches!(err, KindError::ResourceMissing(_)),
            "{level:?}: got {err:?}"
        );

        let task = Setting::Task(TaskAddr {
            path: NO_SUCH_TASK.to_string(),
        });
        let err = AllKinds
            .drive(&task, &Value::TaskEnabled(false), &cx)
            .expect_err("an absent task must refuse, not reach the broker");
        assert!(
            matches!(err, KindError::ResourceMissing(_)),
            "{level:?}: got {err:?}"
        );
    }

    /// Driving *to* `Missing` stays the defined no-op whether or not the resource exists (spec
    /// §5.4, invariant 12). The pre-check must not turn that into a refusal, and it must still
    /// spawn nothing.
    #[test]
    fn driving_an_absent_resource_to_missing_is_still_a_no_op() {
        let cx = ExecCx::new(Level::Ti);

        let svc = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        AllKinds
            .drive(&svc, &Value::Missing, &cx)
            .expect("driving an absent service to Missing is a no-op");

        let task = Setting::Task(TaskAddr {
            path: NO_SUCH_TASK.to_string(),
        });
        AllKinds
            .drive(&task, &Value::Missing, &cx)
            .expect("driving an absent task to Missing is a no-op");
    }

    /// The child reports a failing OP, and only the span it lands in says which ITEM produced it.
    /// An op outside every span names no item: nothing about the run can be inferred from it, so
    /// the items ahead of it must not read as completed.
    #[test]
    fn failing_item_names_only_an_op_that_lands_inside_a_span() {
        // One op, then two (a Service drive's startup plus its companion write), then one.
        let spans = [0..1, 1..3, 3..4];
        for (op, expected) in [(0, Some(0)), (1, Some(1)), (2, Some(1)), (3, Some(2))] {
            assert_eq!(failing_item(&spans, Some(op)), expected, "op {op}");
        }
        assert_eq!(
            failing_item(&spans, Some(4)),
            None,
            "an op just past the batch belongs to no item"
        );
        assert_eq!(failing_item(&spans, Some(99)), None);
        assert_eq!(
            failing_item(&spans, None),
            None,
            "an unnamed op belongs to no item"
        );
    }

    /// A translation that refuses partway through a run aborts before any child exists, so the run
    /// is still entirely undriven: the failure names the item that refused, and proves no drive.
    #[test]
    fn a_translation_refusal_names_its_item_and_proves_nothing_drove() {
        let cx = ExecCx::new(Level::Ti);
        // Item 0 translates to no ops (driving to Missing is the defined no-op); item 1 is an
        // absent service, which the pre-check refuses before anything is spawned.
        let svc = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        let (missing, manual) = (Value::Missing, Value::Startup(StartupType::Manual));
        let failure = AllKinds
            .drive_batch(&[(&svc, &missing), (&svc, &manual)], &cx)
            .expect_err("an absent service must refuse the batch");

        assert_eq!(failure.index, 1, "the item that refused is the one charged");
        assert_eq!(
            failure.completed, 0,
            "no child was spawned, so no item may count as driven"
        );
        assert!(
            matches!(failure.error, KindError::ResourceMissing(_)),
            "got {:?}",
            failure.error
        );
    }
}
