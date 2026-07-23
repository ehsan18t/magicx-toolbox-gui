//! The redesigned tweak engine's lifecycle (spec §4/§8). This task (11) adds detection only —
//! `detect.rs` — plus the shared plumbing every later engine task (apply/restore) will also need:
//! the `Setting → EffectKind` dispatcher, an injectable probe source, and the bundle of external
//! dependencies (`Deps`) that lets every engine entry point run against real stores/kinds in
//! production and in-memory mocks with zero OS contact in tests.
//!
//! ## `Deps` stands in for two things later tasks supply for real
//! - **Elevation**: reads never escalate (spec invariant 24) — they run at whatever level the
//!   process currently holds — so `Deps::level` is the plain [`Level`] a later task (the broker
//!   wiring already noted in `kinds/mod.rs`'s module docs) will derive from the real process state.
//! - **Running Windows build**: the brief's `Deps` asks for the running `WinVer` (major/build/
//!   revision), but `WinVer` is Task 14's type (`RtlGetVersion` is out of scope here). This build
//!   carries the running build as a [`Milestone`] instead — exactly the type `validate.rs`'s
//!   version-scoping helpers already take — and Task 14 supplies it from the real API.

pub mod detect;

use crate::tweaks::kinds::{
    action::ActionKind, firewall::FirewallKind, hosts::HostsKind, registry::RegistryKind,
    service::ServiceKind, task::TaskKind, EffectKind, Error as KindError, ExecCx,
};
use crate::tweaks::model::{ActionDef, EffectId, Level, Setting, Value};
use crate::tweaks::shared_claims::ClaimsStore;
use crate::tweaks::snapshot::SnapshotStore;
use crate::tweaks::validate::Milestone;
use std::collections::HashMap;
use std::sync::Mutex;

/// Production `Setting → EffectKind` dispatcher (the brief's "kinds registry"): delegates by
/// `Setting` variant to the already-reviewed per-kind `EffectKind` impls. Carries no state of its
/// own, so it is trivially `Send + Sync` and cheap to construct per call.
pub struct AllKinds;

impl EffectKind for AllKinds {
    fn read(&self, s: &Setting, cx: &ExecCx) -> Result<Value, KindError> {
        match s {
            Setting::Registry(_) | Setting::RegistryKey(_) => RegistryKind.read(s, cx),
            Setting::Service(_) => ServiceKind.read(s, cx),
            Setting::Task(_) => TaskKind.read(s, cx),
            Setting::Hosts(_) => HostsKind.read(s, cx),
            Setting::Firewall(_) => FirewallKind.read(s, cx),
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), KindError> {
        match s {
            Setting::Registry(_) | Setting::RegistryKey(_) => RegistryKind.drive(s, target, cx),
            Setting::Service(_) => ServiceKind.drive(s, target, cx),
            Setting::Task(_) => TaskKind.drive(s, target, cx),
            Setting::Hosts(_) => HostsKind.drive(s, target, cx),
            Setting::Firewall(_) => FirewallKind.drive(s, target, cx),
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

/// Bundles detect's (and later apply/restore's) external dependencies (controller decision 1): one
/// injection seam so production wires real stores/kinds and tests wire in-memory mocks with zero OS
/// contact. See the module docs for why `level`/`running` are plain fields rather than the broker
/// handle / `WinVer` a later task will supply.
pub struct Deps<'a> {
    pub kinds: &'a dyn EffectKind,
    pub probes: &'a dyn ProbeSource,
    pub claims: &'a ClaimsStore,
    pub snapshots: &'a SnapshotStore,
    pub probe_cache: &'a ProbeCache,
    pub machine_guid: Option<&'a str>,
    pub level: Level,
    pub running: Milestone,
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
