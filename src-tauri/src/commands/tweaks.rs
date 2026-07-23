//! Tauri command surface for the redesigned tweak engine (Task 16; spec §8.4/§9). Thin by design:
//! every command builds `Deps` from managed, app-lifetime state ([`TweakEngineState`]) and
//! delegates straight to the engine (`tweaks::engine::{detect, apply, revert}`) -- no tweak logic
//! lives here. The `*View`/`TweakStatusEvent` types translate engine result types -- which
//! intentionally carry no `Serialize`, since the engine internals are outside this task's touch
//! boundary -- into IPC-safe shapes for the frontend.
//!
//! ## Availability + SID gating (spec §9, controller decision 5)
//! Detection is never gated: reads run at whatever level the app currently holds regardless
//! (invariant 24), so `get_tweaks`'s status is always attempted read-only. Only `apply_tweak`/
//! `restore_tweak` refuse (typed [`Error::TweakUnavailable`]) when [`compute_availability`] reports
//! anything but [`Availability::Available`] -- a tweak whose declared elevation floor exceeds the
//! app's current ceiling ([`needs_elevation`]), or a User-level (HKCU-touching) tweak while the
//! over-the-shoulder SID guard reports a mismatch (`engine::context::user_level_disabled_by_sid_mismatch`).

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{Error, Result};
use crate::services::system_info_service;
use crate::tweaks::compiled_corpus;
use crate::tweaks::engine::apply::{ApplyOutcome, EffectResult, EffectResultKind, EngineError};
use crate::tweaks::engine::context::{self, RealSidProbe};
use crate::tweaks::engine::detect::{
    self, HeldInfo, TweakState, TweakStatus, UnavailableOpt, UnknownCause, UnknownReason,
};
use crate::tweaks::engine::revert::{self, RestoreOutcome};
use crate::tweaks::engine::{apply, lifecycle, AllKinds, Deps, ProbeCache, RealActions, RealProbe};
use crate::tweaks::model::{Corpus, EffectId, Level, OptLabel, RiskLevel, SharedId, Tweak, Value};
use crate::tweaks::shared_claims::ClaimsStore;
use crate::tweaks::snapshot::{EntrySummary, Seq, SnapshotStore};
use crate::tweaks::winver::running_winver;

// --- managed state (controller decision 2) -----------------------------------------------------

/// App-lifetime singletons the engine needs across every tweak command: managed once via Tauri
/// state (`TweakEngineState::new` in `setup.rs`), never re-opened per call. A fresh
/// `SnapshotStore`/`ClaimsStore` per command would still be correct (both are pure on-disk stores
/// with no in-memory state of their own), but a fresh `ProbeCache` per call would silently defeat
/// the whole point of caching probeable-Action reads across a session (spec §7).
pub struct TweakEngineState {
    claims: ClaimsStore,
    snapshots: SnapshotStore,
    probe_cache: ProbeCache,
    machine_guid: Option<String>,
}

impl TweakEngineState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            snapshots: SnapshotStore::open_default().map_err(|e| Error::Tweak(e.to_string()))?,
            claims: ClaimsStore::open_default().map_err(|e| Error::Tweak(e.to_string()))?,
            probe_cache: ProbeCache::new(),
            machine_guid: system_info_service::machine_guid(),
        })
    }

    /// Startup carry-forward (spec §8.1 invariant 5, Task 11's `scan_for_crash_residue` wired here
    /// for the first time): walks every tweak's most recent snapshot entry and logs any left in a
    /// crash-interrupted `intended && !completed` state -- surfaced as Needs Attention via the log;
    /// no dedicated command exists yet to push this list to the UI.
    pub fn scan_startup_crash_residue(&self) {
        let corpus = compiled_corpus();
        let running_build = running_winver().build;
        for tweak in &corpus.tweaks {
            match self.snapshots.head(
                &tweak.id,
                corpus,
                self.machine_guid.as_deref(),
                running_build,
            ) {
                Ok(Some(entry)) => {
                    if let Some(needs_attention) =
                        lifecycle::scan_for_crash_residue(&tweak.id, &entry)
                    {
                        log::error!(
                            "tweak '{}' needs attention after a crash-interrupted apply (seq {:?}): {:?}",
                            tweak.id, needs_attention.seq, needs_attention.unrecoverable
                        );
                    }
                }
                Ok(None) => {}
                Err(e) => log::warn!(
                    "tweak '{}': could not read snapshot history during startup crash scan: {e}",
                    tweak.id
                ),
            }
        }
    }
}

// --- Deps construction (controller decision 3) -------------------------------------------------

// Zero-sized, stateless dispatchers (see their own docs: "trivially Send + Sync and cheap to
// construct per call") -- `static` rather than constructed fresh per command purely so `build_deps`
// can hand back references with no lifetime tied to the calling command's stack frame.
static KINDS: AllKinds = AllKinds;
static PROBES: RealProbe = RealProbe;
static ACTIONS: RealActions = RealActions;

/// The one place that builds `Deps` for a command (controller decision 3): the real dispatcher/
/// probe/action sources, the three managed app-lifetime stores, and the app's current elevation
/// ceiling + running Windows build.
fn build_deps(state: &TweakEngineState) -> Deps<'_> {
    Deps {
        kinds: &KINDS,
        probes: &PROBES,
        actions: &ACTIONS,
        claims: &state.claims,
        snapshots: &state.snapshots,
        probe_cache: &state.probe_cache,
        machine_guid: state.machine_guid.as_deref(),
        level: current_app_level(),
        running: running_winver(),
    }
}

/// The app's current elevation ceiling (controller decision 3): `User` if not running elevated,
/// else `Admin` -- never `System`/`Ti` itself (the whole PROCESS never runs at those levels; only
/// individual effects escalate there per-op through the broker, spec §9).
fn current_app_level() -> Level {
    if system_info_service::is_running_as_admin() {
        Level::Admin
    } else {
        Level::User
    }
}

// --- availability + SID gating (controller decision 5) ------------------------------------------

/// Whether the current app elevation/SID state permits applying/restoring a tweak right now (spec
/// §9). Detection itself never consults this -- only `apply_tweak`/`restore_tweak` refuse on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Availability {
    Available,
    NeedsElevation { reason: String },
    SidMismatch { reason: String },
}

fn compute_availability(
    tweak_elevation: Level,
    current_level: Level,
    sid_mismatch: bool,
) -> Availability {
    if context::user_level_disabled_by_sid_mismatch(tweak_elevation, sid_mismatch) {
        return Availability::SidMismatch {
            reason: "a different administrator's session elevated this app -- User-level tweaks \
                      are disabled until the same account restarts it (over-the-shoulder guard)"
                .to_string(),
        };
    }
    if needs_elevation(tweak_elevation, current_level) {
        return Availability::NeedsElevation {
            reason: format!(
                "requires {tweak_elevation:?} privileges; restart the app as administrator to enable it"
            ),
        };
    }
    Availability::Available
}

/// Whether `tweak_elevation`'s floor is out of reach at `current_level` (spec §9, controller
/// decision 3): the app's own process level is only ever `User` or `Admin` -- once Admin, the
/// elevation broker reaches System/TrustedInstaller for any declared floor, so Admin is the one
/// ceiling that unlocks everything above `User`.
fn needs_elevation(tweak_elevation: Level, current_level: Level) -> bool {
    current_level == Level::User && tweak_elevation != Level::User
}

/// `apply_tweak`/`restore_tweak`'s shared refusal gate: `Ok(())` iff [`Availability::Available`].
fn refuse_if_unavailable(tweak: &Tweak, level: Level, sid_mismatch: bool) -> Result<()> {
    match compute_availability(tweak.elevation, level, sid_mismatch) {
        Availability::Available => Ok(()),
        Availability::NeedsElevation { reason } | Availability::SidMismatch { reason } => {
            Err(Error::TweakUnavailable(reason))
        }
    }
}

fn find_tweak<'a>(corpus: &'a Corpus, tweak_id: &str) -> Result<&'a Tweak> {
    corpus
        .tweaks
        .iter()
        .find(|t| t.id == tweak_id)
        .ok_or_else(|| Error::NotFound(format!("tweak '{tweak_id}'")))
}

/// Maps a restore-originated `EngineError` to the app's error type. Wording-only carry-forward
/// (Task 13): `EngineError::RollbackReport`'s `Display` was written for `apply`'s own rollback
/// ("apply failed (...)"), so a restore failure is re-prefixed HERE, at the command boundary --
/// never by editing `EngineError` in the engine (outside this task's touch boundary).
fn map_restore_err(e: EngineError) -> Error {
    let msg = e.to_string();
    let msg = msg
        .strip_prefix("apply failed")
        .map(|rest| format!("restore failed{rest}"))
        .unwrap_or(msg);
    Error::Tweak(msg)
}

// --- view/event DTOs (IPC-safe projections of the engine's own result types) ---------------------

/// The compiled tweak model for the UI (controller decision 4): identity/display metadata plus
/// this moment's [`Availability`] -- everything the frontend needs to render a tweak before any
/// status has arrived from `get_statuses_stream`.
#[derive(Debug, Clone, Serialize)]
pub struct TweakView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk: RiskLevel,
    pub reversible: bool,
    pub options: Vec<OptLabel>,
    pub elevation: Level,
    pub availability: Availability,
}

/// `get_elevation_state`'s result: the app's own elevation ceiling plus the over-the-shoulder SID
/// guard's current reading (spec §9, ADR-0005).
#[derive(Debug, Clone, Serialize)]
pub struct ElevationState {
    pub level: Level,
    pub sid_mismatch: bool,
}

/// `tweak-status`'s event payload (spec §8.4 grill Q1/Q5): one tweak's freshly detected status,
/// emitted per-tweak by [`scan_and_emit`] -- never batched into one final blob.
#[derive(Debug, Clone, Serialize)]
pub struct TweakStatusEvent {
    pub tweak_id: String,
    pub status: TweakStatusView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TweakStatusView {
    pub state: TweakStateView,
    pub unavailable: Vec<UnavailableOptView>,
    pub residues: Vec<EffectId>,
    pub has_history: bool,
    pub held_shared: Vec<HeldInfoView>,
}

impl From<TweakStatus> for TweakStatusView {
    fn from(s: TweakStatus) -> Self {
        Self {
            state: s.state.into(),
            unavailable: s.unavailable.into_iter().map(Into::into).collect(),
            residues: s.residues,
            has_history: s.has_history,
            held_shared: s.held_shared.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TweakStateView {
    Active { option: OptLabel },
    SystemDefault,
    Unavailable { reason: String },
    Unknown { reasons: Vec<UnknownReasonView> },
}

impl From<TweakState> for TweakStateView {
    fn from(s: TweakState) -> Self {
        match s {
            TweakState::Active(label) => Self::Active { option: label },
            TweakState::SystemDefault => Self::SystemDefault,
            TweakState::Unavailable(reason) => Self::Unavailable { reason },
            TweakState::Unknown(reasons) => Self::Unknown {
                reasons: reasons.into_iter().map(Into::into).collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnknownReasonView {
    pub effect: EffectId,
    pub cause: UnknownCauseView,
    pub needs_elevation: bool,
}

impl From<UnknownReason> for UnknownReasonView {
    fn from(r: UnknownReason) -> Self {
        Self {
            effect: r.effect,
            cause: r.cause.into(),
            needs_elevation: r.needs_elevation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum UnknownCauseView {
    AccessDenied,
    Malformed,
    MissingRequired,
    Other,
}

impl From<UnknownCause> for UnknownCauseView {
    fn from(c: UnknownCause) -> Self {
        match c {
            UnknownCause::AccessDenied => Self::AccessDenied,
            UnknownCause::Malformed => Self::Malformed,
            UnknownCause::MissingRequired => Self::MissingRequired,
            UnknownCause::Other => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnavailableOptView {
    pub label: OptLabel,
    pub reason: String,
}

impl From<UnavailableOpt> for UnavailableOptView {
    fn from(u: UnavailableOpt) -> Self {
        Self {
            label: u.label,
            reason: u.reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeldInfoView {
    pub shared: SharedId,
    pub holders: Vec<String>,
}

impl From<HeldInfo> for HeldInfoView {
    fn from(h: HeldInfo) -> Self {
        Self {
            shared: h.shared,
            holders: h.holders,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApplyOutcomeView {
    pub effects: Vec<EffectResultView>,
    pub status: TweakStatusView,
}

impl From<ApplyOutcome> for ApplyOutcomeView {
    fn from(o: ApplyOutcome) -> Self {
        Self {
            effects: o.effects.into_iter().map(Into::into).collect(),
            status: o.status.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectResultView {
    pub effect: EffectId,
    pub kind: EffectResultKindView,
}

impl From<EffectResult> for EffectResultView {
    fn from(r: EffectResult) -> Self {
        Self {
            effect: r.effect,
            kind: r.kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectResultKindView {
    Driven { desired: Value },
    Claimed,
    StillHeld { holders: Vec<String> },
    Released,
    Ran,
    UndoDrivenBack,
    NoOp,
}

impl From<EffectResultKind> for EffectResultKindView {
    fn from(k: EffectResultKind) -> Self {
        match k {
            EffectResultKind::Driven { desired } => Self::Driven { desired },
            EffectResultKind::Claimed => Self::Claimed,
            EffectResultKind::StillHeld(holders) => Self::StillHeld { holders },
            EffectResultKind::Released => Self::Released,
            EffectResultKind::Ran => Self::Ran,
            EffectResultKind::UndoDrivenBack => Self::UndoDrivenBack,
            EffectResultKind::NoOp => Self::NoOp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RestoreOutcomeView {
    pub status: TweakStatusView,
    pub consumed: Option<Seq>,
    pub reboot_advisory: bool,
    pub skipped_invalid: Vec<EntrySummary>,
}

impl From<RestoreOutcome> for RestoreOutcomeView {
    fn from(o: RestoreOutcome) -> Self {
        Self {
            status: o.status.into(),
            consumed: o.consumed,
            reboot_advisory: o.reboot_advisory,
            skipped_invalid: o.skipped_invalid,
        }
    }
}

// --- scan/emit + apply, factored into plain functions for testing (brief's own testing note) -----

/// Runs `detect` for every tweak in `corpus`, invoking `emit` once per tweak in corpus order (spec
/// §8.4 grill Q1/Q5: incremental arrival, never one final blob) -- factored out of
/// `get_statuses_stream`/`rescan_after_elevation` so `statuses_emit_incrementally` can prove the
/// one-event-per-tweak property with an injectable emitter and zero Tauri runtime.
fn scan_and_emit(corpus: &Corpus, deps: &Deps<'_>, mut emit: impl FnMut(TweakStatusEvent)) {
    for tweak in &corpus.tweaks {
        let status = detect::detect(tweak, corpus, deps);
        emit(TweakStatusEvent {
            tweak_id: tweak.id.clone(),
            status: status.into(),
        });
    }
}

/// Spawns the corpus-wide detect sweep on a plain OS thread (never the UI thread, controller
/// decision 7) and emits one `tweak-status` event per tweak as [`scan_and_emit`] produces it --
/// shared by `get_statuses_stream` (the initial background-progressive scan) and
/// `rescan_after_elevation` (the same full re-scan, replayed after Elevate).
fn spawn_full_scan(app: AppHandle) {
    std::thread::spawn(move || {
        let state = app.state::<TweakEngineState>();
        let deps = build_deps(state.inner());
        let corpus = compiled_corpus();
        scan_and_emit(corpus, &deps, |event| {
            if let Err(e) = app.emit("tweak-status", &event) {
                log::warn!(
                    "tweak status scan: failed to emit for '{}': {e}",
                    event.tweak_id
                );
            }
        });
    });
}

/// `apply_tweak`'s engine call + view conversion, factored out so `apply_returns_fresh_status_no_rescan`
/// can prove the returned status is the engine outcome's own (never a second, fresh `detect` call)
/// without needing a live Tauri runtime.
async fn apply_tweak_logic(
    tweak: &Tweak,
    corpus: &Corpus,
    target: &OptLabel,
    deps: &Deps<'_>,
) -> std::result::Result<ApplyOutcomeView, EngineError> {
    apply::apply(tweak, corpus, target, deps)
        .await
        .map(ApplyOutcomeView::from)
}

// --- commands -------------------------------------------------------------------------------------

#[tauri::command]
pub async fn get_tweaks() -> Result<Vec<TweakView>> {
    log::info!("get_tweaks: building the compiled tweak view for the UI");
    let corpus = compiled_corpus();
    let level = current_app_level();
    let mismatch = context::sid_mismatch(&RealSidProbe);
    Ok(corpus
        .tweaks
        .iter()
        .map(|t| TweakView {
            id: t.id.clone(),
            name: t.name.clone(),
            description: t.description.clone(),
            category: t.category.clone(),
            risk: t.risk_level,
            reversible: t.reversible,
            options: t.options.iter().map(|o| o.label.clone()).collect(),
            elevation: t.elevation,
            availability: compute_availability(t.elevation, level, mismatch),
        })
        .collect())
}

/// Kicks the background-progressive full scan (spec §8.4 grill Q1) and returns immediately -- the
/// scan itself runs on a separate OS thread and streams results back via `tweak-status` events.
#[tauri::command]
pub async fn get_statuses_stream(app: AppHandle) -> Result<()> {
    log::info!("get_statuses_stream: kicking the background-progressive full scan");
    spawn_full_scan(app);
    Ok(())
}

/// The full re-scan run after the user Elevates (spec §8.4 grill Q1: the moment Unknowns become
/// readable) -- reuses the exact same scan path as `get_statuses_stream`.
#[tauri::command]
pub async fn rescan_after_elevation(app: AppHandle) -> Result<()> {
    log::info!("rescan_after_elevation: kicking a full re-scan after an elevation change");
    spawn_full_scan(app);
    Ok(())
}

#[tauri::command]
pub async fn apply_tweak(
    state: State<'_, TweakEngineState>,
    tweak_id: String,
    option_label: String,
) -> Result<ApplyOutcomeView> {
    log::info!("apply_tweak: '{tweak_id}' -> '{option_label}'");
    let corpus = compiled_corpus();
    let tweak = find_tweak(corpus, &tweak_id)?;

    let level = current_app_level();
    let mismatch = context::sid_mismatch(&RealSidProbe);
    refuse_if_unavailable(tweak, level, mismatch)?;

    let deps = build_deps(state.inner());
    let target = OptLabel(option_label);
    apply_tweak_logic(tweak, corpus, &target, &deps)
        .await
        .map_err(|e| Error::Tweak(e.to_string()))
}

#[tauri::command]
pub async fn restore_tweak(
    state: State<'_, TweakEngineState>,
    tweak_id: String,
) -> Result<RestoreOutcomeView> {
    log::info!("restore_tweak: '{tweak_id}'");
    let corpus = compiled_corpus();
    let tweak = find_tweak(corpus, &tweak_id)?;

    let level = current_app_level();
    let mismatch = context::sid_mismatch(&RealSidProbe);
    refuse_if_unavailable(tweak, level, mismatch)?;

    let deps = build_deps(state.inner());
    revert::restore(tweak, corpus, &deps)
        .await
        .map(RestoreOutcomeView::from)
        .map_err(map_restore_err)
}

#[tauri::command]
pub async fn list_snapshot_entries(
    state: State<'_, TweakEngineState>,
    tweak_id: String,
) -> Result<Vec<EntrySummary>> {
    log::info!("list_snapshot_entries: '{tweak_id}'");
    let corpus = compiled_corpus();
    state
        .snapshots
        .list(
            &tweak_id,
            corpus,
            state.machine_guid.as_deref(),
            running_winver().build,
        )
        .map_err(|e| Error::Tweak(e.to_string()))
}

/// The explicit-consent snapshot release (ADR-0002) -- `SnapshotStore::discard` never runs on a
/// failure path, only here, on a direct user decision.
#[tauri::command]
pub async fn discard_snapshot_entry(
    state: State<'_, TweakEngineState>,
    tweak_id: String,
    seq: Seq,
) -> Result<()> {
    log::info!("discard_snapshot_entry: '{tweak_id}' seq {seq:?}");
    state
        .snapshots
        .discard(&tweak_id, seq)
        .map_err(|e| Error::Tweak(e.to_string()))
}

#[tauri::command]
pub async fn get_elevation_state() -> Result<ElevationState> {
    log::info!("get_elevation_state");
    Ok(ElevationState {
        level: current_app_level(),
        sid_mismatch: context::sid_mismatch(&RealSidProbe),
    })
}

// --- tests ------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::{ActionRunner, ProbeSource};
    use crate::tweaks::kinds::{EffectKind, Error as KindError, ExecCx};
    use crate::tweaks::model::{
        ActionDef, Effect, EffectDef, Opt, OptValue, RiskLevel as ModelRisk, ScopedValue, Setting,
        StartupType, SvcAddr,
    };
    use crate::tweaks::winver::WinVer;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;

    // --- minimal mocks (mirrors engine::apply's own test-harness pattern; kept local since those
    // fixtures are `#[cfg(test)]`-private to that module) ---------------------------------------

    /// A single shared live `Value`, regardless of which `Setting` is asked -- adequate here since
    /// every fixture tweak below has exactly one Setting effect; counts every `read` call so the
    /// no-rescan test can assert on it directly.
    struct CountingKind {
        live: Mutex<Value>,
        reads: AtomicU32,
    }
    impl CountingKind {
        fn new(initial: Value) -> Self {
            Self {
                live: Mutex::new(initial),
                reads: AtomicU32::new(0),
            }
        }
    }
    impl EffectKind for CountingKind {
        fn read(&self, _s: &Setting, _cx: &ExecCx) -> std::result::Result<Value, KindError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            Ok(self.live.lock().unwrap().clone())
        }
        fn drive(
            &self,
            _s: &Setting,
            target: &Value,
            _cx: &ExecCx,
        ) -> std::result::Result<(), KindError> {
            *self.live.lock().unwrap() = target.clone();
            Ok(())
        }
    }

    /// No fixture tweak below declares an Action -- both traits panic if ever actually called.
    struct NoProbesActions;
    impl ProbeSource for NoProbesActions {
        fn probe(&self, _a: &ActionDef, _cx: &ExecCx) -> std::result::Result<bool, KindError> {
            unreachable!("no Action effects on these fixtures")
        }
    }
    impl ActionRunner for NoProbesActions {
        fn apply(&self, _a: &ActionDef, _cx: &ExecCx) -> std::result::Result<(), KindError> {
            unreachable!("no Action effects on these fixtures")
        }
        fn undo(&self, _a: &ActionDef, _cx: &ExecCx) -> std::result::Result<(), KindError> {
            unreachable!("no Action effects on these fixtures")
        }
    }

    // --- fixture builders --------------------------------------------------------------------

    fn svc_effect() -> EffectDef {
        EffectDef {
            id: EffectId("svc".to_string()),
            kind: Effect::Setting(Setting::Service(SvcAddr {
                name: "svc".to_string(),
            })),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn opt(label: &str, value: StartupType) -> Opt {
        let mut map = BTreeMap::new();
        map.insert(
            EffectId("svc".to_string()),
            OptValue::Set(ScopedValue {
                value: Value::Startup(value),
                windows: None,
            }),
        );
        Opt {
            label: OptLabel(label.to_string()),
            values: map,
        }
    }

    fn tweak(id: &str, options: Vec<Opt>) -> Tweak {
        Tweak {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            category: "misc".to_string(),
            info: None,
            warning: None,
            requires_reboot: false,
            risk_level: ModelRisk::Low,
            elevation: Level::User,
            reversible: true,
            surface: vec![svc_effect()],
            options,
            windows: None,
        }
    }

    fn corpus(tweaks: Vec<Tweak>) -> Corpus {
        Corpus {
            categories: Vec::new(),
            tweaks,
            shared: Vec::new(),
        }
    }

    /// Owns everything a test needs so `Deps` (all borrows) can outlive the call under test.
    struct Harness {
        kind: CountingKind,
        probes_actions: NoProbesActions,
        claims: ClaimsStore,
        snapshots: SnapshotStore,
        cache: ProbeCache,
        _tmp: tempfile::TempDir,
    }
    impl Harness {
        fn new(initial: Value) -> Self {
            let tmp = tempfile::tempdir().unwrap();
            Self {
                kind: CountingKind::new(initial),
                probes_actions: NoProbesActions,
                claims: ClaimsStore::open(tmp.path().to_path_buf(), Some("test-guid".into())),
                snapshots: SnapshotStore::open(tmp.path().to_path_buf()),
                cache: ProbeCache::new(),
                _tmp: tmp,
            }
        }
        fn deps(&self) -> Deps<'_> {
            Deps {
                kinds: &self.kind,
                probes: &self.probes_actions,
                actions: &self.probes_actions,
                claims: &self.claims,
                snapshots: &self.snapshots,
                probe_cache: &self.cache,
                machine_guid: Some("test-guid"),
                level: Level::User,
                running: WinVer {
                    build: 19045,
                    revision: 0,
                },
            }
        }
    }

    /// Blocks on an async call without pulling in a full async-test harness: mirrors
    /// `engine::apply`/`engine::revert`'s own minimal single-poll executor -- the only await point
    /// anywhere in this call chain is an uncontended per-tweak lock acquire, which resolves on the
    /// first poll.
    fn futures_block_on<F: std::future::Future>(mut fut: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        fn noop(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);
        // SAFETY: `fut` is a local, never moved after this point.
        let mut fut = unsafe { std::pin::Pin::new_unchecked(&mut fut) };
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(v) => return v,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    // --- the brief's two named tests -----------------------------------------------------------

    #[test]
    fn statuses_emit_incrementally() {
        let h = Harness::new(Value::Startup(StartupType::Manual));
        let c = corpus(vec![
            tweak("t1", vec![opt("On", StartupType::Manual)]),
            tweak("t2", vec![opt("On", StartupType::Manual)]),
            tweak("t3", vec![opt("On", StartupType::Manual)]),
        ]);
        let deps = h.deps();

        let mut events: Vec<TweakStatusEvent> = Vec::new();
        scan_and_emit(&c, &deps, |event| events.push(event));

        assert_eq!(
            events.len(),
            3,
            "one event per tweak, never one final blob (grill Q1/Q5)"
        );
        assert_eq!(events[0].tweak_id, "t1");
        assert_eq!(events[1].tweak_id, "t2");
        assert_eq!(events[2].tweak_id, "t3");
    }

    #[test]
    fn apply_returns_fresh_status_no_rescan() {
        let h = Harness::new(Value::Startup(StartupType::Manual)); // starts at "Off"
        let t = tweak(
            "demo",
            vec![
                opt("Off", StartupType::Manual),
                opt("On", StartupType::Disabled),
            ],
        );
        let c = corpus(vec![t.clone()]);
        let deps = h.deps();

        // A real transition: mutates the mock live value to "On".
        let first = futures_block_on(apply_tweak_logic(&t, &c, &OptLabel("On".into()), &deps))
            .expect("apply succeeds");
        assert_eq!(
            first.status.state,
            TweakStateView::Active {
                option: OptLabel("On".into())
            }
        );

        // Isolate the next call's own read count.
        h.kind.reads.store(0, Ordering::SeqCst);

        // Re-applying the SAME target is the engine's verified no-op fast path (apply.rs step 0):
        // exactly ONE read (the pre-status detect), nothing driven. If this command layer ever
        // performed its own extra `detect` before/after handing back the outcome, this would read
        // more than once.
        let second = futures_block_on(apply_tweak_logic(&t, &c, &OptLabel("On".into()), &deps))
            .expect("no-op apply succeeds");
        assert_eq!(
            h.kind.reads.load(Ordering::SeqCst),
            1,
            "the command layer must reuse apply's own outcome status, never re-detect"
        );
        assert_eq!(
            second.status.state,
            TweakStateView::Active {
                option: OptLabel("On".into())
            }
        );
        assert!(second.effects.is_empty(), "a verified no-op drives nothing");
    }
}
