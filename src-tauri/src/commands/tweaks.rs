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
//! app's current ceiling ([`needs_elevation`]), or a tweak that *touches HKCU* while the
//! over-the-shoulder SID guard has not confirmed the session owner
//! (`engine::context::hkcu_disabled_by_sid_mismatch`). Note the second is keyed on the hive the
//! tweak writes, never on its `elevation:` floor -- see ADR-0005's 2026-07-27 amendment.

use rayon::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{Error, Result};
use crate::services::system_info_service;
use crate::tweaks::compiled_corpus;
use crate::tweaks::engine::apply::{ApplyOutcome, EffectResult, EffectResultKind, EngineError};
use crate::tweaks::engine::context::{self, RealSidProbe, SidCheck};
use crate::tweaks::engine::detect::{
    self, HeldInfo, ObservedEffect, TweakState, TweakStatus, UnavailableOpt, UnknownCause,
    UnknownReason,
};
use crate::tweaks::engine::revert::{self, RestoreOutcome};
use crate::tweaks::engine::{apply, lifecycle, AllKinds, Deps, ProbeCache, RealActions, RealProbe};
use crate::tweaks::model::{
    ActionDef, Corpus, Effect, EffectDef, EffectId, FwAction, FwDirection, FwProtocol, Hive, Level,
    Opt, OptLabel, OptValue, RegType, RiskLevel, Setting, SharedId, StartupType, Tweak,
    TypedRegValue, Value,
};
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
    NeedsElevation {
        reason: String,
    },
    SidMismatch {
        reason: String,
    },
    /// The SID guard could not read one of the two SIDs, so it does not know whose hive an HKCU
    /// write would land in. Distinct from `SidMismatch` on purpose: telling the user another admin
    /// elevated the app when we simply could not tell is a fabricated accusation, and it sends them
    /// chasing a cause that does not exist.
    SidUnknown {
        reason: String,
    },
}

/// `touches_hkcu` comes from `context::tweak_touches_hkcu`, NOT from the tweak's `elevation:` floor.
/// The floor says which privilege the tweak needs; the hive says whose state it changes. Those are
/// independent, and conflating them was the original defect: 31 `admin`-floor tweaks in the corpus
/// drive HKCU effects and went unguarded, while all 54 `user`-floor tweaks were blocked outright.
fn compute_availability(
    touches_hkcu: bool,
    tweak_elevation: Level,
    current_level: Level,
    sid_check: SidCheck,
) -> Availability {
    if context::hkcu_disabled_by_sid_mismatch(touches_hkcu, sid_check) {
        return match sid_check {
            SidCheck::DifferentUser => Availability::SidMismatch {
                reason: "Another account elevated this app, so per-user tweaks stay off until you \
                         restart it under your own account."
                    .to_string(),
            },
            _ => Availability::SidUnknown {
                reason: "This app could not confirm which account owns this session, so per-user \
                         tweaks stay off rather than risk changing the wrong account's settings."
                    .to_string(),
            },
        };
    }
    if needs_elevation(tweak_elevation, current_level) {
        return Availability::NeedsElevation {
            reason: "Restart the app as administrator to enable this tweak.".to_string(),
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
fn refuse_if_unavailable(
    tweak: &Tweak,
    corpus: &Corpus,
    level: Level,
    sid_check: SidCheck,
) -> Result<()> {
    let touches_hkcu = context::tweak_touches_hkcu(tweak, corpus);
    match compute_availability(touches_hkcu, tweak.elevation, level, sid_check) {
        Availability::Available => Ok(()),
        Availability::NeedsElevation { reason }
        | Availability::SidMismatch { reason }
        | Availability::SidUnknown { reason } => Err(Error::TweakUnavailable(reason)),
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
    /// Rich markdown detail shown in the tweak's Details modal (spec: authored `info:`).
    pub info: Option<String>,
    pub category: String,
    pub risk: RiskLevel,
    pub reversible: bool,
    /// Whether applying/restoring this tweak needs a reboot to take full effect (spec §6).
    pub requires_reboot: bool,
    /// Each option with the concrete effects it drives, so the Details modal can show a power
    /// user exactly what a state writes (registry values, service start-types, tasks, and so on).
    pub options: Vec<TweakOptionView>,
    pub elevation: Level,
    pub availability: Availability,
}

/// One option projected as the exact per-address changes it makes. The `*_changes` shapes mirror
/// the frontend's long-standing detail types (`RegistryChange`/`ServiceChange`/…), so the existing
/// detail components render them unchanged; the projection just joins the tweak's surface (address
/// per `EffectId`) with this option's value for that same id.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TweakOptionView {
    pub label: String,
    pub registry_changes: Vec<RegistryChangeView>,
    pub service_changes: Vec<ServiceChangeView>,
    pub scheduler_changes: Vec<SchedulerChangeView>,
    pub hosts_changes: Vec<HostsChangeView>,
    pub firewall_changes: Vec<FirewallChangeView>,
    /// Action scripts this option runs (Appx removal, powercfg, DISM, …), shown verbatim.
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegistryChangeView {
    pub hive: String,
    pub key: String,
    pub value_name: String,
    /// `set` | `delete_value` | `delete_key` | `create_key`.
    pub action: String,
    pub value_type: Option<String>,
    /// The concrete value this option writes (number / string / list), or null for a delete.
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_versions: Option<Vec<u8>>,
    pub skip_validation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ServiceChangeView {
    pub name: String,
    pub startup: String,
    pub skip_validation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SchedulerChangeView {
    pub task_path: String,
    /// `enable` | `disable`.
    pub action: String,
    pub skip_validation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HostsChangeView {
    pub ip: String,
    pub domain: String,
    /// `add` | `remove`.
    pub action: String,
    pub skip_validation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FirewallChangeView {
    pub name: String,
    /// `create` | `delete`.
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ports: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ports: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub skip_validation: bool,
}

fn hive_str(h: Hive) -> &'static str {
    match h {
        Hive::Hklm => "HKLM",
        Hive::Hkcu => "HKCU",
    }
}

fn reg_type_str(t: RegType) -> &'static str {
    match t {
        RegType::Dword => "REG_DWORD",
        RegType::Qword => "REG_QWORD",
        RegType::Sz => "REG_SZ",
        RegType::ExpandSz => "REG_EXPAND_SZ",
        RegType::MultiSz => "REG_MULTI_SZ",
        RegType::Binary => "REG_BINARY",
    }
}

fn startup_str(s: StartupType) -> &'static str {
    match s {
        StartupType::Boot => "boot",
        StartupType::System => "system",
        StartupType::Automatic => "automatic",
        StartupType::AutomaticDelayed => "automatic_delayed",
        StartupType::Manual => "manual",
        StartupType::Disabled => "disabled",
    }
}

fn reg_value_json(v: &TypedRegValue) -> serde_json::Value {
    use serde_json::json;
    match v {
        TypedRegValue::Dword(n) => json!(n),
        TypedRegValue::Qword(n) => json!(n),
        TypedRegValue::Sz(s) | TypedRegValue::ExpandSz(s) => json!(s),
        TypedRegValue::MultiSz(items) => json!(items),
        TypedRegValue::Binary(bytes) => json!(bytes),
    }
}

/// Appends the display change(s) for one `Setting` driven to `value` by this option.
fn push_setting_change(
    setting: &Setting,
    value: &Value,
    effect: &EffectDef,
    o: &mut TweakOptionView,
) {
    let skip = effect.optional;
    let win = effect.windows.as_ref().and_then(|w| w.products.clone());
    match setting {
        Setting::Registry(a) => {
            let (action, value_type, val) = match value {
                Value::Absent => ("delete_value", None, None),
                Value::Reg(tv) => (
                    "set",
                    Some(reg_type_str(a.ty).to_string()),
                    Some(reg_value_json(tv)),
                ),
                _ => ("set", Some(reg_type_str(a.ty).to_string()), None),
            };
            // Packed fields address a sub-field inside one value; show both so it isn't mistaken
            // for the whole value.
            let value_name = match &a.field {
                Some(f) => format!("{} [{}]", a.name, f.field),
                None => a.name.clone(),
            };
            o.registry_changes.push(RegistryChangeView {
                hive: hive_str(a.hive).to_string(),
                key: a.path.clone(),
                value_name,
                action: action.to_string(),
                value_type,
                value: val,
                windows_versions: win,
                skip_validation: skip,
            });
        }
        Setting::RegistryKey(k) => {
            let action = if matches!(value, Value::Present(true)) {
                "create_key"
            } else {
                "delete_key"
            };
            o.registry_changes.push(RegistryChangeView {
                hive: hive_str(k.hive).to_string(),
                key: k.path.clone(),
                value_name: String::new(),
                action: action.to_string(),
                value_type: None,
                value: None,
                windows_versions: win,
                skip_validation: skip,
            });
        }
        Setting::Service(s) => {
            if let Value::Startup(st) = value {
                o.service_changes.push(ServiceChangeView {
                    name: s.name.clone(),
                    startup: startup_str(*st).to_string(),
                    skip_validation: skip,
                });
            }
        }
        Setting::Task(t) => {
            if let Value::TaskEnabled(enabled) = value {
                o.scheduler_changes.push(SchedulerChangeView {
                    task_path: t.path.clone(),
                    action: if *enabled { "enable" } else { "disable" }.to_string(),
                    skip_validation: skip,
                });
            }
        }
        Setting::Hosts(h) => {
            if let Value::Present(present) = value {
                o.hosts_changes.push(HostsChangeView {
                    ip: h.ip.clone(),
                    domain: h.domain.clone(),
                    action: if *present { "add" } else { "remove" }.to_string(),
                    skip_validation: skip,
                });
            }
        }
        Setting::Firewall(r) => {
            if let Value::Present(present) = value {
                o.firewall_changes.push(FirewallChangeView {
                    name: r.name.clone(),
                    operation: if *present { "create" } else { "delete" }.to_string(),
                    direction: Some(
                        match r.direction {
                            FwDirection::Inbound => "inbound",
                            FwDirection::Outbound => "outbound",
                        }
                        .to_string(),
                    ),
                    action: Some(
                        match r.action {
                            FwAction::Block => "block",
                            FwAction::Allow => "allow",
                        }
                        .to_string(),
                    ),
                    protocol: r.protocol.map(|p| {
                        match p {
                            FwProtocol::Any => "any",
                            FwProtocol::Tcp => "tcp",
                            FwProtocol::Udp => "udp",
                            FwProtocol::Icmpv4 => "icmpv4",
                            FwProtocol::Icmpv6 => "icmpv6",
                        }
                        .to_string()
                    }),
                    program: r.program.clone(),
                    service: r.service.clone(),
                    remote_addresses: r.remote_addresses.clone(),
                    remote_ports: r.remote_ports.clone(),
                    local_ports: r.local_ports.clone(),
                    description: r.description.clone(),
                    skip_validation: skip,
                });
            }
        }
    }
}

/// Projects one option into the concrete changes it drives across the tweak's surface (joining each
/// surface `EffectDef`'s address with this option's value for the same `EffectId`).
fn option_view(tweak: &Tweak, opt: &Opt, corpus: &Corpus) -> TweakOptionView {
    let mut o = TweakOptionView {
        label: opt.label.0.clone(),
        registry_changes: Vec::new(),
        service_changes: Vec::new(),
        scheduler_changes: Vec::new(),
        hosts_changes: Vec::new(),
        firewall_changes: Vec::new(),
        commands: Vec::new(),
    };
    for effect in &tweak.surface {
        let value_for_effect = opt.values.get(&effect.id);
        match &effect.kind {
            Effect::Setting(setting) => {
                if let Some(OptValue::Set(sv)) = value_for_effect {
                    push_setting_change(setting, &sv.value, effect, &mut o);
                }
            }
            // A claimed shared setting resolves to its declared target value (spec §6.5); an
            // unclaimed/absent option makes no change to that address.
            Effect::Shared(shared_id) => {
                if matches!(value_for_effect, Some(OptValue::Claim(_))) {
                    if let Some(sd) = corpus.shared.iter().find(|s| &s.id == shared_id) {
                        push_setting_change(&sd.setting, &sd.value, effect, &mut o);
                    }
                }
            }
            Effect::Action(action) => {
                if matches!(value_for_effect, Some(OptValue::Run(_))) {
                    match action {
                        ActionDef::Script { apply, .. } => o.commands.push(apply.0.clone()),
                        // A DeleteTree is a registry key removal; show it as one.
                        ActionDef::DeleteTree { key, .. } => {
                            o.registry_changes.push(RegistryChangeView {
                                hive: hive_str(key.hive).to_string(),
                                key: key.path.clone(),
                                value_name: String::new(),
                                action: "delete_key".to_string(),
                                value_type: None,
                                value: None,
                                windows_versions: None,
                                skip_validation: effect.optional,
                            })
                        }
                    }
                }
            }
        }
    }
    o
}

/// Builds one IPC [`TweakView`] from a compiled `Tweak` at the given elevation/SID context.
/// Factored out of [`get_tweaks`] so a command-layer test can assert field carry-through
/// (e.g. `requires_reboot`, spec §6) without needing a live Tauri runtime.
fn tweak_view(t: &Tweak, corpus: &Corpus, level: Level, sid_check: SidCheck) -> TweakView {
    TweakView {
        id: t.id.clone(),
        name: t.name.clone(),
        description: t.description.clone(),
        info: t.info.clone(),
        category: t.category.clone(),
        risk: t.risk_level,
        reversible: t.reversible,
        requires_reboot: t.requires_reboot,
        options: t
            .options
            .iter()
            .map(|o| option_view(t, o, corpus))
            .collect(),
        elevation: t.elevation,
        availability: compute_availability(
            context::tweak_touches_hkcu(t, corpus),
            t.elevation,
            level,
            sid_check,
        ),
    }
}

/// `get_elevation_state`'s result: the app's own elevation ceiling plus the over-the-shoulder SID
/// guard's current reading (spec §9, ADR-0005).
///
/// `sid_mismatch` stays a `bool` for the UI's benefit -- it only ever asks "are per-user tweaks
/// blocked" -- but it is now `SidCheck::blocks_hkcu()`, which is true for `Undetermined` as well as
/// `DifferentUser`. The per-tweak `Availability` carries the distinction where it matters.
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TweakStatusView {
    pub state: TweakStateView,
    pub unavailable: Vec<UnavailableOptView>,
    pub residues: Vec<EffectId>,
    pub has_history: bool,
    pub held_shared: Vec<HeldInfoView>,
    /// Present only at System Default. Requires the tweak to build, so [`From`] leaves it `None`
    /// and [`scan_and_emit`] fills it in where the tweak is in scope.
    pub observed: Option<ObservedStateView>,
}

impl From<TweakStatus> for TweakStatusView {
    fn from(s: TweakStatus) -> Self {
        Self {
            state: s.state.into(),
            unavailable: s.unavailable.into_iter().map(Into::into).collect(),
            residues: s.residues,
            has_history: s.has_history,
            held_shared: s.held_shared.into_iter().map(Into::into).collect(),
            observed: None,
        }
    }
}

/// What the machine actually reads when it matches no authored option (spec §8.4, ADR-0003).
///
/// `changes` is deliberately a [`TweakOptionView`]: the frontend renders it with the same
/// components as the options it failed to match, so the user compares like with like instead of
/// reading a value dump and doing the translation in their head.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObservedStateView {
    pub changes: TweakOptionView,
    pub agreement: Vec<EffectAgreementView>,
}

/// Which authored options wanted the value one effect actually holds. Empty `wanted_by` means no
/// option does; a surface split across several options is the usual reason nothing matched.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EffectAgreementView {
    pub effect: EffectId,
    /// What the effect actually addresses on the machine (registry value, service, task, ...). The
    /// `effect` id is an author's slug and means nothing to a user reading the change list above.
    pub name: String,
    pub wanted_by: Vec<String>,
}

/// The concrete thing an effect addresses, named the way the change list above already names it.
/// Paths are dropped: the row directly above carries the full address, so the short name is what
/// tells the two rows apart. Falls back to the effect id for the kinds that have no address.
fn effect_display_name(effect: &EffectDef) -> String {
    let Effect::Setting(setting) = &effect.kind else {
        return effect.id.0.clone();
    };
    match setting {
        Setting::Registry(a) => match &a.field {
            Some(f) => format!("{} [{}]", a.name, f.field),
            None => a.name.clone(),
        },
        Setting::RegistryKey(k) => leaf_of(&k.path, '\\'),
        Setting::Service(s) => s.name.clone(),
        Setting::Task(t) => leaf_of(&t.path, '\\'),
        Setting::Hosts(h) => h.domain.clone(),
        Setting::Firewall(r) => r.name.clone(),
    }
}

/// Last segment of a backslash-delimited address, or the whole thing when it has no separator.
fn leaf_of(path: &str, sep: char) -> String {
    path.rsplit(sep)
        .find(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

/// Builds the System Default explanation by pushing each live reading through the very same
/// change-view builder the authored options use.
fn observed_view(tweak: &Tweak, observed: &[ObservedEffect]) -> Option<ObservedStateView> {
    if observed.is_empty() {
        return None;
    }
    let mut changes = TweakOptionView {
        label: "Your system right now".to_string(),
        registry_changes: Vec::new(),
        service_changes: Vec::new(),
        scheduler_changes: Vec::new(),
        hosts_changes: Vec::new(),
        firewall_changes: Vec::new(),
        commands: Vec::new(),
    };
    let mut agreement = Vec::new();
    for o in observed {
        let Some(effect) = tweak.surface.iter().find(|e| e.id == o.effect) else {
            continue;
        };
        // `detect::observe` only ever reports Settings: shared refs carry claim state rather than a
        // value, and probe-backed Actions have no address to show beside an option's.
        if let Effect::Setting(setting) = &effect.kind {
            push_setting_change(setting, &o.value, effect, &mut changes);
        }
        agreement.push(EffectAgreementView {
            effect: o.effect.clone(),
            name: effect_display_name(effect),
            wanted_by: o.wanted_by.iter().map(|l| l.0.clone()).collect(),
        });
    }
    Some(ObservedStateView { changes, agreement })
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

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
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

/// Detects one tweak and projects it into the event the frontend consumes.
fn scan_one(tweak: &Tweak, corpus: &Corpus, deps: &Deps<'_>) -> TweakStatusEvent {
    let status = detect::detect(tweak, corpus, deps);
    let observed = observed_view(tweak, &status.observed);
    let mut view: TweakStatusView = status.into();
    view.observed = observed;
    TweakStatusEvent {
        tweak_id: tweak.id.clone(),
        status: view,
    }
}

/// Runs `detect` for every tweak in `corpus`, invoking `emit` once per tweak (spec §8.4 grill
/// Q1/Q5: incremental arrival, never one final blob) -- factored out of
/// `get_statuses_stream`/`rescan_after_elevation` so `statuses_emit_incrementally` can prove the
/// one-event-per-tweak property with an injectable emitter and zero Tauri runtime.
///
/// Detection is a pure read, so tweaks are independent and run on a rayon pool. That is worth real
/// wall-clock: a handful of Action probes each spawn a process, and serially those dominate the
/// whole sweep. Results are streamed back over a channel and emitted on the calling thread, which
/// keeps `emit` free of any `Send` bound and preserves the progressive arrival the UI is built
/// around. **Emission order is therefore completion order, not corpus order** -- every event names
/// its `tweak_id` and the frontend keys on it, so order carries no meaning.
///
/// Shared state the workers touch is already prepared for this: `ProbeCache` is behind a mutex, the
/// two stores are immutable path holders that re-read per call, and `scheduler_service` serializes
/// its own COM activation behind a lock with a once-per-thread MTA init.
fn scan_and_emit(corpus: &Corpus, deps: &Deps<'_>, mut emit: impl FnMut(TweakStatusEvent)) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::scope(|s| {
        s.spawn(move || {
            corpus.tweaks.par_iter().for_each_with(tx, |tx, tweak| {
                // A closed receiver only happens if the drain below panicked; nothing to do.
                let _ = tx.send(scan_one(tweak, corpus, deps));
            });
        });
        for event in rx {
            emit(event);
        }
    });
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
    let sid_check = context::sid_check(&RealSidProbe);
    Ok(corpus
        .tweaks
        .iter()
        .map(|t| tweak_view(t, corpus, level, sid_check))
        .collect())
}

/// Corpus category metadata (id + display name + icon + description) for the sidebar. The compiled
/// corpus is the single source of truth; the frontend must not re-derive names from category ids.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryView {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
}

#[tauri::command]
pub async fn get_categories() -> Result<Vec<CategoryView>> {
    log::info!("get_categories: corpus category metadata for the UI");
    Ok(compiled_corpus()
        .categories
        .iter()
        .map(|c| CategoryView {
            id: c.id.clone(),
            name: c.name.clone(),
            icon: c.icon.clone(),
            description: c.description.clone(),
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
    let sid_check = context::sid_check(&RealSidProbe);
    refuse_if_unavailable(tweak, corpus, level, sid_check)?;

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
    let sid_check = context::sid_check(&RealSidProbe);
    refuse_if_unavailable(tweak, corpus, level, sid_check)?;

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
        sid_mismatch: context::sid_check(&RealSidProbe).blocks_hkcu(),
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

    // --- availability + SID/elevation gating (pure logic, no Tauri runtime, no OS) --------------

    #[test]
    fn sid_guard_blocks_every_hkcu_touching_tweak_whatever_its_floor() {
        // The guard's input is the HIVE, not the floor. Every floor is checked, including Admin,
        // System and Ti: 31 admin-floor tweaks in the shipped corpus drive HKCU effects, and the
        // old floor-keyed guard let all of them through while blocking the 54 user-floor ones.
        for floor in [Level::User, Level::Admin, Level::System, Level::Ti] {
            let avail = compute_availability(true, floor, Level::Admin, SidCheck::DifferentUser);
            assert!(
                matches!(avail, Availability::SidMismatch { .. }),
                "an HKCU-touching {floor:?}-floor tweak must be blocked, got {avail:?}"
            );
        }

        // A tweak that touches no HKCU state is unaffected: whose session this is cannot change
        // where an HKLM write lands.
        assert_eq!(
            compute_availability(false, Level::Admin, Level::Admin, SidCheck::DifferentUser),
            Availability::Available
        );

        // Precedence: the SID arm is checked before `needs_elevation`, so an HKCU-touching
        // admin-floor tweak on an unelevated app reports the SID reason, not the elevation one.
        let avail = compute_availability(true, Level::Admin, Level::User, SidCheck::DifferentUser);
        assert!(
            matches!(avail, Availability::SidMismatch { .. }),
            "got {avail:?}"
        );
    }

    #[test]
    fn undetermined_sid_blocks_but_does_not_accuse() {
        // Two distinct states on purpose. Telling the user another admin elevated the app when the
        // guard simply could not read a SID sends them chasing a cause that does not exist.
        let unknown = compute_availability(true, Level::User, Level::User, SidCheck::Undetermined);
        assert!(
            matches!(unknown, Availability::SidUnknown { .. }),
            "got {unknown:?}"
        );
        let Availability::SidUnknown { reason } = &unknown else {
            unreachable!()
        };
        assert!(
            !reason.contains("Another account"),
            "an undetermined SID must not be reported as a different account: {reason}"
        );

        let mismatch =
            compute_availability(true, Level::User, Level::User, SidCheck::DifferentUser);
        assert!(
            matches!(mismatch, Availability::SidMismatch { .. }),
            "got {mismatch:?}"
        );
    }

    #[test]
    fn confirmed_same_user_never_blocks_hkcu() {
        // The regression this whole change exists to prevent: with the SID guard answering
        // SameUser, a user-floor HKCU tweak must be applyable at BOTH app levels. It was blocked at
        // both, because the old probe used WTSQueryUserToken (SE_TCB_NAME, LocalSystem only) and
        // the guard read its guaranteed failure as a mismatch.
        for level in [Level::User, Level::Admin] {
            assert_eq!(
                compute_availability(true, Level::User, level, SidCheck::SameUser),
                Availability::Available,
                "a per-user tweak must be applyable at app level {level:?}"
            );
        }
    }

    #[test]
    fn elevation_floor_above_app_level_is_needs_elevation() {
        // App level User + an Admin/System/Ti-floor tweak -> disabled, needs elevation.
        for floor in [Level::Admin, Level::System, Level::Ti] {
            let avail = compute_availability(false, floor, Level::User, SidCheck::SameUser);
            assert!(
                matches!(avail, Availability::NeedsElevation { .. }),
                "floor={floor:?} at app level User must need elevation, got {avail:?}"
            );
        }

        // App level Admin + any floor (including System/Ti) -> available: Admin is the one
        // ceiling that reaches System/TrustedInstaller via the broker too (controller decision 3).
        for floor in [Level::User, Level::Admin, Level::System, Level::Ti] {
            assert_eq!(
                compute_availability(false, floor, Level::Admin, SidCheck::SameUser),
                Availability::Available,
                "floor={floor:?} at app level Admin must be available"
            );
        }
    }

    #[test]
    fn available_when_level_sufficient_and_no_sid_mismatch() {
        for (touches_hkcu, floor, level) in [
            (true, Level::User, Level::User),
            (true, Level::User, Level::Admin),
            (false, Level::Admin, Level::Admin),
        ] {
            assert_eq!(
                compute_availability(touches_hkcu, floor, level, SidCheck::SameUser),
                Availability::Available,
                "hkcu={touches_hkcu} floor={floor:?} level={level:?}"
            );
        }
    }

    #[test]
    fn apply_and_restore_refuse_unavailable_before_the_engine() {
        // `refuse_if_unavailable` is the exact gate `apply_tweak`/`restore_tweak` call before ever
        // building `Deps` or reaching the engine -- calling it directly (never `build_deps`, never
        // `apply::apply`/`revert::restore`) is itself the proof that the refusal happens ahead of
        // any engine call: there is no engine call in this test at all, only the gate.
        let mut t = tweak("demo", vec![opt("On", StartupType::Manual)]);
        let c = corpus(vec![]);

        // Needs-elevation refusal.
        t.elevation = Level::Admin;
        let err = refuse_if_unavailable(&t, &c, Level::User, SidCheck::SameUser)
            .expect_err("must refuse");
        assert!(matches!(err, Error::TweakUnavailable(_)), "got {err:?}");

        // The happy path: an available tweak must not refuse.
        t.elevation = Level::User;
        assert!(refuse_if_unavailable(&t, &c, Level::User, SidCheck::SameUser).is_ok());

        // This fixture's surface is a Service effect, so it touches no HKCU state and the SID guard
        // must not gate it even under a genuine mismatch. That is the point of keying on the hive.
        assert!(refuse_if_unavailable(&t, &c, Level::User, SidCheck::DifferentUser).is_ok());
    }

    #[test]
    fn get_tweaks_carries_requires_reboot() {
        // spec §6: `requires_reboot` must reach the UI. `tweak_view` is exactly the mapping
        // `get_tweaks` runs over every corpus tweak, so asserting on it proves the field is
        // carried through without a live Tauri runtime or the embedded corpus.
        let mut t = tweak("demo", vec![opt("On", StartupType::Manual)]);
        let c = corpus(vec![]);
        t.requires_reboot = true;
        assert!(tweak_view(&t, &c, Level::User, SidCheck::SameUser).requires_reboot);
        t.requires_reboot = false;
        assert!(!tweak_view(&t, &c, Level::User, SidCheck::SameUser).requires_reboot);

        // The option projection joins the surface address with the option's value: this fixture's
        // one Service effect driven to Manual must surface as exactly one service change.
        let view = tweak_view(&t, &c, Level::User, SidCheck::SameUser);
        assert_eq!(view.options.len(), 1);
        assert_eq!(view.options[0].service_changes.len(), 1);
        assert_eq!(view.options[0].service_changes[0].startup, "manual");
        assert!(view.options[0].registry_changes.is_empty());
    }

    #[test]
    fn option_view_projects_real_registry_values() {
        // End-to-end against the embedded corpus: the registry values a power user sees in the
        // Details modal must be the tweak's actual authored values, not a placeholder. `dark mode`
        // drives two HKCU REG_DWORDs to 0 for its "Dark" option.
        let corpus = compiled_corpus();
        let dark = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "enable_dark_mode")
            .expect("enable_dark_mode is in the corpus");
        let dark_opt = dark
            .options
            .iter()
            .find(|o| o.label.0 == "Dark")
            .expect("the Dark option exists");

        let view = option_view(dark, dark_opt, corpus);
        assert_eq!(
            view.registry_changes.len(),
            2,
            "both theme values are surfaced"
        );
        for change in &view.registry_changes {
            assert_eq!(change.hive, "HKCU");
            assert_eq!(change.action, "set");
            assert_eq!(change.value_type.as_deref(), Some("REG_DWORD"));
            assert_eq!(
                change.value,
                Some(serde_json::json!(0)),
                "Dark sets each flag to 0"
            );
        }
        assert!(view.service_changes.is_empty());
        assert!(view.commands.is_empty());
    }

    // --- the brief's two named tests -----------------------------------------------------------

    /// One event per tweak, never one final blob (grill Q1/Q5), and every tweak covered exactly
    /// once. Deliberately asserts the SET of ids rather than their positions: the sweep runs in
    /// parallel, so events arrive in completion order and position carries no meaning. Comparing
    /// sets is also the stronger check, since it catches a duplicate or a dropped tweak, which
    /// indexing into the first three events would not.
    #[test]
    fn statuses_emit_incrementally() {
        let h = Harness::new(Value::Startup(StartupType::Manual));
        let ids = ["t1", "t2", "t3"];
        let c = corpus(
            ids.iter()
                .map(|id| tweak(id, vec![opt("On", StartupType::Manual)]))
                .collect(),
        );
        let deps = h.deps();

        let mut events: Vec<TweakStatusEvent> = Vec::new();
        scan_and_emit(&c, &deps, |event| events.push(event));

        assert_eq!(
            events.len(),
            ids.len(),
            "one event per tweak, never one final blob (grill Q1/Q5)"
        );
        let mut got: Vec<&str> = events.iter().map(|e| e.tweak_id.as_str()).collect();
        got.sort_unstable();
        assert_eq!(got, ids, "every tweak reported exactly once");
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

    /// The details UI renders one section per change kind straight off `TweakOptionView`. If
    /// `push_setting_change` ever stops projecting a kind, that section silently renders empty and
    /// the tweak looks like it only touches the registry -- a wrong answer that no other test would
    /// catch, since apply/detect keep working perfectly. So: across the whole shipped corpus, every
    /// Setting effect an option actually values must land in exactly one of the six change lists.
    #[test]
    fn every_valued_setting_effect_reaches_the_option_view() {
        let corpus = compiled_corpus();
        let mut kinds_seen = (0usize, 0usize, 0usize); // registry, service, scheduler
        for t in &corpus.tweaks {
            for opt in &t.options {
                // A claimed `shared` effect resolves to its declared setting and projects a change
                // too (option_view's Shared arm), so it counts alongside plain Settings.
                let valued_settings = t
                    .surface
                    .iter()
                    .filter(|e| match &e.kind {
                        Effect::Setting(_) => {
                            matches!(opt.values.get(&e.id), Some(OptValue::Set(_)))
                        }
                        Effect::Shared(_) => {
                            matches!(opt.values.get(&e.id), Some(OptValue::Claim(_)))
                        }
                        Effect::Action(_) => false,
                    })
                    .count();
                let v = option_view(t, opt, corpus);
                let projected = v.registry_changes.len()
                    + v.service_changes.len()
                    + v.scheduler_changes.len()
                    + v.hosts_changes.len()
                    + v.firewall_changes.len();
                assert_eq!(
                    projected, valued_settings,
                    "tweak `{}` option `{}`: {valued_settings} valued Setting effects but only \
                     {projected} reached the details view -- a change kind is being dropped",
                    t.id, opt.label.0
                );
                kinds_seen.0 += v.registry_changes.len();
                kinds_seen.1 += v.service_changes.len();
                kinds_seen.2 += v.scheduler_changes.len();
            }
        }
        // Guards the guard: if the corpus ever stopped shipping services or tasks the loop above
        // would pass vacuously for those kinds.
        assert!(
            kinds_seen.0 > 0 && kinds_seen.1 > 0 && kinds_seen.2 > 0,
            "expected the corpus to exercise registry, service and scheduled-task projection; got {kinds_seen:?}"
        );
    }
}
