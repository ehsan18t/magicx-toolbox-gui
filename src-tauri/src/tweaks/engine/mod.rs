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
use crate::services::elevation::{self, BrokerOp, BrokerOpError, Elevation};
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
                    .map_err(|error| BatchFailure { index, error })?;
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
                Err(error) => return Err(BatchFailure { index, error }),
            }
            spans.push(start..ops.len());
        }

        if ops.is_empty() {
            return Ok(());
        }
        let sent = ops.len();
        elevation::run_ops(Elevation::TrustedInstaller, ops).map_err(|e| match e {
            // Acquisition failed, so no op ran. Attributing it to the first item is the truthful
            // choice: it is where the batch stopped, and the error type still says plainly that
            // nothing was attempted.
            BrokerOpError::CouldNotAcquire(err) => BatchFailure {
                index: 0,
                error: KindError::CouldNotAcquireElevation(Level::Ti, err.to_string()),
            },
            ref failed @ BrokerOpError::OpFailed { ref source, .. } => BatchFailure {
                index: failing_item(&spans, failed.failed_op_index(), sent),
                error: KindError::ElevatedOpFailed(Level::Ti, source.to_string()),
            },
            // No op index to attribute this to: the point is that we do not know which ran. The
            // first item is where the batch is treated as having stopped.
            BrokerOpError::Indeterminate(err) => BatchFailure {
                index: 0,
                error: KindError::ElevatedOutcomeUnknown(Level::Ti, err.to_string()),
            },
        })
    }
}

/// Map a broker op failure back to the batch item whose translation produced that op.
///
/// Falls back to the last item when the index cannot be placed, so a response we did not expect
/// still leaves the caller rolling back from a real position rather than panicking.
fn failing_item(spans: &[std::ops::Range<usize>], failed_op: Option<usize>, sent: usize) -> usize {
    let op_index = failed_op.unwrap_or(sent.saturating_sub(1));
    spans
        .iter()
        .position(|span| span.contains(&op_index))
        .unwrap_or(spans.len().saturating_sub(1))
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
        BrokerOpError::OpFailed { source, .. } => {
            KindError::ElevatedOpFailed(level, source.to_string())
        }
        BrokerOpError::Indeterminate(err) => {
            KindError::ElevatedOutcomeUnknown(level, err.to_string())
        }
    })
}

/// One release-visible line per elevated failure: which tweak, where in it, which level, and how
/// the broker classified it. Called by the command layer, the first point that owns both halves:
/// the drive sites below know the classification but not which tweak is running.
pub fn log_elevated_failure(tweak_id: &str, e: &EngineError) {
    for line in elevated_failure_lines(tweak_id, e) {
        log::error!("{line}");
    }
}

/// Every elevated failure `e` carries, a rollback's own failures included: ADR-0001's Needs
/// Attention state is exactly a rollback that hit one, so it can never be the line that is missing.
fn elevated_failure_lines(tweak_id: &str, e: &EngineError) -> Vec<String> {
    let mut lines = Vec::new();
    collect_elevated_failures(tweak_id, e, "", &mut lines);
    lines
}

fn collect_elevated_failures(tweak_id: &str, e: &EngineError, phase: &str, out: &mut Vec<String>) {
    let (site, source) = match e {
        EngineError::RollbackReport {
            original,
            rollback_failures,
        } => {
            collect_elevated_failures(tweak_id, original, phase, out);
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
            "tweak '{tweak_id}': {site} failed{phase} at {level:?} elevation: {classification}"
        ));
    }
}

/// The three ways a failure can have reached the elevated child, named without the detail behind
/// them: that text is the broker's and can carry a path, a nonce or a registry value.
fn elevated_failure(e: &KindError) -> Option<(Level, &'static str)> {
    match e {
        KindError::CouldNotAcquireElevation(level, _) => Some((
            *level,
            "could not acquire the elevated child, so nothing ran",
        )),
        KindError::ElevatedOpFailed(level, _) => {
            Some((*level, "an operation was refused inside the elevated child"))
        }
        KindError::ElevatedOutcomeUnknown(level, _) => Some((
            *level,
            "the elevated child ran but its outcome is unknown, so the machine may have changed",
        )),
        _ => None,
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
    use crate::tweaks::model::{OptLabel, SharedId, StartupType, SvcAddr, TaskAddr};

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
                KindError::ElevatedOpFailed(Level::Ti, HOSTILE_DETAIL.into()),
                "refused inside the elevated child",
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
                    source: KindError::ElevatedOpFailed(Level::Ti, HOSTILE_DETAIL.into()),
                }),
                rollback_failures: vec![EngineError::DriveFailed {
                    effect: EffectId("wu_orch".into()),
                    source: KindError::ElevatedOutcomeUnknown(Level::Ti, HOSTILE_DETAIL.into()),
                }],
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
}
