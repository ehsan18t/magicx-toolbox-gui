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

use crate::services::appx_index::AppxIndex;
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

/// Production `Setting → EffectKind` dispatcher: delegates by `Setting` variant at `User`/`Admin`,
/// and routes through the elevation broker at `System`/`Ti` (spec §9, see [`drive_via_broker`]).
/// Stateless, so it is trivially `Send + Sync` and cheap to construct per call.
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
            level @ Level::Ti => {
                // Mirror the existence pre-check `drive_service`/`drive_task` do in-process
                // (invariant 12). The broker translations are deliberately pure, and without this
                // an absent service/task reaches the child, fails there, and comes back as an
                // opaque `OpFailed` -> `AccessDenied`. That is the wrong shape twice over: apply's
                // `optional`/`if_missing` no-op guard matches only `ResourceMissing`, so an
                // `optional` effect whose resource is absent on this build would abort the tweak
                // and roll it back instead of reading as the verified no-op detect already
                // advertises. Reads never escalate, so this costs the same in-process read detect
                // already performed, and it means an absent resource never spawns a child at all.
                if !matches!(target, Value::Missing)
                    && matches!(s, Setting::Service(_) | Setting::Task(_))
                    && self.read(s, cx)? == Value::Missing
                {
                    return Err(KindError::ResourceMissing(match s {
                        Setting::Service(addr) => {
                            format!("service '{}' does not exist", addr.name)
                        }
                        Setting::Task(addr) => {
                            format!("scheduled task '{}' does not exist", addr.path)
                        }
                        _ => unreachable!("guarded by the matches! above"),
                    }));
                }
                match s {
                    Setting::Registry(_) | Setting::RegistryKey(_) => {
                        drive_via_broker(level, vec![registry::to_broker_op(s, target, level)?])
                    }
                    Setting::Service(_) => {
                        drive_via_broker(level, service::to_broker_ops(s, target)?)
                    }
                    Setting::Task(_) => drive_via_broker(level, task::to_broker_ops(s, target)?),
                    // No BrokerOp exists for Hosts/Firewall in this build (spec §9's mechanical
                    // translation list does not cover them) -- fall through to the in-process kind,
                    // which correctly still rejects System/Ti itself.
                    Setting::Hosts(_) => HostsKind.drive(s, target, cx),
                    Setting::Firewall(_) => FirewallKind.drive(s, target, cx),
                }
            }
        }
    }
}

/// Submits `ops` through the elevation broker in ONE child (spec §9), keeping the two failure
/// modes apart: the child was never acquired (environmental) versus it ran and the op was refused.
/// An empty `ops` list, which a Service/Task drive to `Missing` produces, is a verified no-op that
/// never spawns a child.
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
    use crate::tweaks::model::{StartupType, SvcAddr, TaskAddr};

    /// Names that certainly do not exist, so these need no elevation and no real resource.
    const NO_SUCH_SERVICE: &str = "MagicXNoSuchService_5F3F1D2E-6A4B-4C9E-9B0A-6B6E6C7D8E9F";
    const NO_SUCH_TASK: &str = r"\MagicXNoSuchFolder_5F3F1D2E\NoSuchTask";

    /// An `optional` effect whose resource is absent has to reach apply as `ResourceMissing`: that
    /// is the only error its `if_missing` no-op guard matches on (`apply::drive_forward`). The
    /// routed System/Ti path used to hand the drive to the elevated child, where absence came back
    /// as an opaque `OpFailed` -> `AccessDenied`, so `optional` silently did nothing above User/
    /// Admin and a tweak with a task absent on this Windows build aborted and rolled back.
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
