//! The redesigned tweak engine's lifecycle (spec §4/§8). Task 11 added detection; Task 14 adds
//! execution-context routing — the `Setting → EffectKind` dispatcher now also decides, per
//! `ExecCx::level()`, whether a drive runs in-process or through the elevation broker (see
//! `AllKinds::drive` below and `kinds/mod.rs`'s module docs for the placement rationale) — and
//! wires the real running Windows version.
//!
//! ## `Deps` now carries the real elevation level and Windows version
//! - **Elevation**: reads never escalate (spec invariant 24) — they run at whatever level the
//!   process currently holds — so `Deps::level` is the plain [`Level`] the command layer (a later
//!   task) derives from the real process state.
//! - **Running Windows build**: `Deps::running` is now [`WinVer`] (build + revision, spec §6.6),
//!   supplied by `winver::running_winver()` in production. Runtime call sites that only need the
//!   build-only shape (`validate.rs`'s Milestone-based helpers) go through
//!   [`WinVer::to_milestone`] — see `winver.rs`'s module docs for the full reconciliation.

pub mod apply;
pub mod context;
pub mod detect;
pub mod lifecycle;
pub mod revert;

use crate::services::elevation::{self, BrokerOp, BrokerOpError, Elevation};
use crate::tweaks::kinds::{
    action::ActionKind,
    firewall::FirewallKind,
    hosts::HostsKind,
    registry::{self, RegistryKind},
    service::{self, ServiceKind},
    task::{self, TaskKind},
    EffectKind, Error as KindError, ExecCx,
};
use crate::tweaks::model::{ActionDef, EffectId, Level, Setting, Value};
use crate::tweaks::shared_claims::ClaimsStore;
use crate::tweaks::snapshot::SnapshotStore;
use crate::tweaks::winver::WinVer;
use std::collections::HashMap;
use std::sync::Mutex;

/// Production `Setting → EffectKind` dispatcher (the brief's "kinds registry"): delegates by
/// `Setting` variant to the already-reviewed per-kind `EffectKind` impls for `Level::User`/
/// `Level::Admin`; for `Level::System`/`Level::Ti` it routes through the elevation broker instead
/// (spec §9) — see [`drive_via_broker`] and `kinds/mod.rs`'s module docs for why the routing
/// decision lives here rather than in each kind's own `drive`. Carries no state of its own, so it
/// is trivially `Send + Sync` and cheap to construct per call.
pub struct AllKinds;

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
            level @ (Level::System | Level::Ti) => match s {
                Setting::Registry(_) | Setting::RegistryKey(_) => {
                    drive_via_broker(level, vec![registry::to_broker_op(s, target, level)?])
                }
                Setting::Service(_) => drive_via_broker(level, service::to_broker_ops(s, target)?),
                Setting::Task(_) => drive_via_broker(level, task::to_broker_ops(s, target)?),
                // No BrokerOp exists for Hosts/Firewall in this build (spec §9's mechanical
                // translation list does not cover them) -- fall through to the in-process kind,
                // which correctly still rejects System/Ti itself.
                Setting::Hosts(_) => HostsKind.drive(s, target, cx),
                Setting::Firewall(_) => FirewallKind.drive(s, target, cx),
            },
        }
    }
}

/// Submits `ops` through the elevation broker in ONE child (spec §9) and maps the result onto
/// [`KindError`]'s did-it-work-preserving distinction (invariant 24): [`BrokerOpError::CouldNotAcquire`]
/// (environmental -- the child was never acquired) becomes [`KindError::CouldNotAcquireElevation`];
/// [`BrokerOpError::OpFailed`] (acquired, but the op itself was denied) becomes the existing
/// [`KindError::AccessDenied`]. An empty `ops` list (e.g. a Service/Task drive to `Missing`, spec
/// §5.4) is a verified no-op -- never spawns a child for nothing.
fn drive_via_broker(level: Level, ops: Vec<BrokerOp>) -> Result<(), KindError> {
    if ops.is_empty() {
        return Ok(());
    }
    elevation::run_ops(to_elevation(level), ops).map_err(|e| match e {
        BrokerOpError::CouldNotAcquire(err) => {
            KindError::CouldNotAcquireElevation(level, err.to_string())
        }
        BrokerOpError::OpFailed(err) => KindError::AccessDenied(err.to_string()),
    })
}

/// Maps a tweak's declared [`Level`] to the broker's own [`Elevation`] — only ever called for
/// `System`/`Ti`; `User`/`Admin` run in-process and never reach the broker at all.
fn to_elevation(level: Level) -> Elevation {
    match level {
        Level::System => Elevation::System,
        Level::Ti => Elevation::TrustedInstaller,
        Level::User | Level::Admin => {
            unreachable!("drive_via_broker is only ever reached for System/Ti")
        }
    }
}

/// Injectable source for an Action's probe (spec §7), separate from [`EffectKind`] because Actions
/// are not Settings (`kinds/mod.rs`'s module docs). Production runs the real `ActionKind::run_probe`
/// ([`RealProbe`]); tests substitute an in-memory mock that also counts invocations, so
/// `probe_cache_hit_no_respawn` can prove the cache — not the mock — is what suppresses a respawn.
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

/// Injectable source for an Action's `apply`/`undo` (spec §7) — added alongside [`ProbeSource`]
/// (kept separate and unchanged, so Task 11's `detect` -- which only ever probes an Action, never
/// runs one -- needs no behavioral mock, just a trivial implementor to satisfy [`Deps`]'s shape).
/// Without this seam, `engine::apply` (a later task) would have to call the concrete `ActionKind`
/// directly, spawning a real process even in unit tests -- exactly what `ProbeSource` already
/// exists to avoid for probes. Tests substitute an in-memory mock that also records call order, so
/// apply's capture-before-mutation and completion-after-each-action invariants are provable with
/// zero OS contact.
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

/// Bundles detect's (and apply/restore's) external dependencies (controller decision 1): one
/// injection seam so production wires real stores/kinds and tests wire in-memory mocks with zero OS
/// contact. See the module docs for why `level`/`running` are plain fields rather than the broker
/// handle / `WinVer` a later task will supply.
pub struct Deps<'a> {
    pub kinds: &'a dyn EffectKind,
    pub probes: &'a dyn ProbeSource,
    /// Task 12: the Action apply/undo seam (see [`ActionRunner`]'s docs). `detect` never reads
    /// this field.
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
#[derive(Default)]
pub struct ProbeCache {
    entries: Mutex<HashMap<(String, EffectId), bool>>,
}

impl ProbeCache {
    pub fn new() -> Self {
        Self::default()
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

    /// Drops every cached probe for `tweak_id` (spec §7): that tweak's own apply/restore
    /// invalidates its probes so the next detect re-observes live state. `detect` itself never
    /// calls this — later engine tasks (apply/restore) do, after they mutate the tweak's surface.
    pub fn invalidate(&self, tweak_id: &str) {
        self.entries
            .lock()
            .expect("ProbeCache mutex poisoned")
            .retain(|(t, _), _| t != tweak_id);
    }
}
