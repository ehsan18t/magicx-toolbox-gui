//! Core typed model for the redesigned tweak engine — Effect/Setting/Value and the compiled
//! Tweak shape. See spec §5 (the Effect abstraction) and §6 (data model) at
//! docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md.
//!
//! This module is the compiled-model representation only: no parsing, validation, or engine
//! behavior lives here (later tasks add those). `Value`'s `#[derive(PartialEq)]` is the whole
//! implementation of invariant 1 ("one comparison per kind") — different variants are never
//! equal, so e.g. `Absent` and `Present(false)` cannot be confused by construction.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Elevation floor for a tweak or an individual effect (spec §9). Declared, never inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Level {
    User,
    Admin,
    System,
    Ti,
}

/// Advisory impact rating shown to the user (spec §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// v1 supports exactly these two hives (spec §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hive {
    Hklm,
    Hkcu,
}

/// The declared type of a registry value address, matching the `.reg` type names (spec §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegType {
    Dword,
    Qword,
    Sz,
    ExpandSz,
    MultiSz,
    Binary,
}

/// v1 ships exactly one packed-value format (spec §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackedFormat {
    KvSemicolon,
}

/// One field within a packed registry value (spec §5.2), e.g. `SwapEffectUpgradeEnable` inside
/// `DirectXUserGlobalSettings`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldAddr {
    pub field: String,
    pub format: PackedFormat,
}

/// A registry value address. `path` excludes the hive prefix (carried separately in `hive`) —
/// the compiled form splits what the YAML author writes as one merged string (spec §5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegAddr {
    pub hive: Hive,
    pub path: String,
    pub name: String,
    pub ty: RegType,
    pub field: Option<FieldAddr>,
}

/// A registry key address — existence only, no value (spec §5.1: `RegistryKey` Setting).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyAddr {
    pub hive: Hive,
    pub path: String,
}

/// A service address, keyed by exact service name (spec §5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvcAddr {
    pub name: String,
}

/// A scheduled-task address, keyed by exact task path — no patterns (spec §5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAddr {
    pub path: String,
}

/// A hosts-file entry address (spec §5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostsAddr {
    pub ip: String,
    pub domain: String,
}

/// A firewall rule address. Spec §5.1 describes this as "named rule + full rule definition";
/// the definition fields are deferred to Task 7 (Hosts and Firewall kinds), which pins the
/// schema against the real `firewall_service` primitive — nothing in §5/§6 or the brief's
/// interface list specifies them, and no Task 1 test exercises this type's fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleAddr {
    pub name: String,
}

/// One address kind on a tweak's managed surface (spec §5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    Registry(RegAddr),
    RegistryKey(KeyAddr),
    Service(SvcAddr),
    Task(TaskAddr),
    Hosts(HostsAddr),
    Firewall(RuleAddr),
}

/// A typed registry literal (spec §6.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypedRegValue {
    Dword(u32),
    Qword(u64),
    Sz(String),
    ExpandSz(String),
    MultiSz(Vec<String>),
    Binary(Vec<u8>),
}

/// Service start type (spec §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StartupType {
    Boot,
    System,
    Automatic,
    AutomaticDelayed,
    Manual,
    Disabled,
}

/// The one value domain shared by capture, apply, detect, and restore (spec §5, invariant 1).
/// `Missing` is capture-only (spec §5.4) — that rule is enforced by the parser/validator
/// (Tasks 2-3), not by this type; here it is an ordinary variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Absent,
    Missing,
    Reg(TypedRegValue),
    Startup(StartupType),
    TaskEnabled(bool),
    Present(bool),
}

/// The text of a `cmd`/`powershell` script, resolved at build time (inline or filed — spec §7
/// says filed scripts are "embedded by `build.rs`", so by the compiled-model stage there is only
/// ever a body string left).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script(pub String);

/// Which interpreter runs a `Script` (spec §5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shell {
    Cmd,
    PowerShell,
}

/// An imperative Action (spec §7). `DeleteTree` is the one surviving structural op (spec §5.1);
/// it drives the registry primitive directly rather than running a script, so — unlike `Script`
/// — it carries no `shell`/`probe`/`ephemeral`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDef {
    Script {
        apply: Script,
        undo: Option<Script>,
        probe: Option<Script>,
        ephemeral: bool,
        shell: Shell,
    },
    DeleteTree {
        key: KeyAddr,
        undo: Option<Script>,
    },
}

/// Identifies one effect within a tweak's declared surface. Newtype (not a bare `String`) so it
/// cannot be mixed up with a `SharedId` or an `OptLabel` at compile time.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EffectId(pub String);

/// Identifies a corpus-level shared setting (spec §6.5). Newtype for the same reason as `EffectId`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SharedId(pub String);

/// The display label of an authored option. Newtype for the same reason as `EffectId`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OptLabel(pub String);

// `Display` (not just `Debug`) on all three id newtypes: build-time `ValidationError` messages
// (Task 3) print these verbatim in author-facing text, where `EffectId("foo")` would be noise.
impl std::fmt::Display for EffectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for SharedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for OptLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One atomic unit of change (spec §5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Setting(Setting),
    Shared(SharedId),
    Action(ActionDef),
}

/// Windows-build range grammar shared by `build`/`revision` (spec §6.6): `N | >=N | <=N | A..B`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildExpr {
    Exact(u32),
    Min(u32),
    Max(u32),
    Range(u32, u32),
}

/// Windows-version applicability, ANDed across axes, legal at tweak/effect/per-option-value
/// level (spec §6.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsScope {
    pub products: Option<Vec<u8>>,
    pub build: Option<BuildExpr>,
    pub revision: Option<BuildExpr>,
}

/// A `Value` together with the per-option-value Windows scope carried alongside it (spec §6.6
/// allows scoping at the tweak, effect, or per-option-value level; this is the third one).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedValue {
    pub value: Value,
    pub windows: Option<WindowsScope>,
}

/// What one option says about one effect on the surface (spec §6). Only `Set` carries a value —
/// `Run`/`Claim`/`Unclaimed` are the whole answer for Action/Shared effects, so they carry no
/// scope of their own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptValue {
    Set(ScopedValue),
    Run,
    Claim,
    Unclaimed,
}

/// One selectable state: a label plus a value for every effect on the tweak's surface (spec §6.1,
/// §6.3 — every option covers every Setting effect; build-guarded, not enforced here).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opt {
    pub label: OptLabel,
    pub values: BTreeMap<EffectId, OptValue>,
}

/// One entry on a tweak's declared managed surface (spec §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectDef {
    pub id: EffectId,
    pub kind: Effect,
    pub elevation: Option<Level>,
    pub optional: bool,
    pub if_missing: Option<Value>,
    pub windows: Option<WindowsScope>,
}

/// A corpus-level shared setting (spec §6.5): the single target value claiming tweaks agree on
/// by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedDef {
    pub id: SharedId,
    pub setting: Setting,
    pub value: Value,
}

/// A compiled tweak: its declared surface plus the options offered over that surface (spec §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tweak {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub info: Option<String>,
    pub warning: Option<String>,
    pub requires_reboot: bool,
    pub risk_level: RiskLevel,
    pub elevation: Level,
    pub reversible: bool,
    pub surface: Vec<EffectDef>,
    pub options: Vec<Opt>,
    pub windows: Option<WindowsScope>,
}

/// A corpus file's `category:` header (spec §6 shows the shape; the id/name/icon/description
/// fields match today's `src-tauri/tweaks/*.yaml`). Stamped onto every `Tweak` loaded from that
/// file — authors never repeat it per-tweak (Task 3, schema.rs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryDef {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
}

/// The fully loaded, corpus-wide result of `schema::load_corpus` (Task 3): every category header,
/// every tweak (already stamped with its file's category), and every merged `shared:` declaration.
/// `shared` is corpus-wide (spec §6.5) — a single flat list regardless of which file declared each
/// entry — which is exactly why duplicate `shared` ids must be checked across the whole `Corpus`,
/// not per-file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Corpus {
    pub categories: Vec<CategoryDef>,
    pub tweaks: Vec<Tweak>,
    pub shared: Vec<SharedDef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_roundtrips_serde() {
        let values = vec![
            Value::Absent,
            Value::Missing,
            Value::Reg(TypedRegValue::Dword(42)),
            Value::Startup(StartupType::AutomaticDelayed),
            Value::TaskEnabled(true),
            Value::Present(false),
        ];
        for value in values {
            let json = serde_json::to_string(&value).expect("serialize");
            let restored: Value = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(value, restored, "roundtrip mismatch for {value:?}");
        }
    }

    #[test]
    fn value_equality_is_per_kind() {
        assert_ne!(Value::Absent, Value::Present(false));
        assert_ne!(Value::Missing, Value::Absent);
        assert_eq!(Value::Present(true), Value::Present(true));
        assert_ne!(Value::Present(true), Value::Present(false));
    }

    #[test]
    fn model_types_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<Effect>();
        assert_send_sync::<Setting>();
        assert_send_sync::<Value>();
        assert_send_sync::<TypedRegValue>();
        assert_send_sync::<StartupType>();
        assert_send_sync::<Level>();
        assert_send_sync::<RiskLevel>();
        assert_send_sync::<Hive>();
        assert_send_sync::<RegType>();
        assert_send_sync::<RegAddr>();
        assert_send_sync::<KeyAddr>();
        assert_send_sync::<SvcAddr>();
        assert_send_sync::<TaskAddr>();
        assert_send_sync::<HostsAddr>();
        assert_send_sync::<RuleAddr>();
        assert_send_sync::<FieldAddr>();
        assert_send_sync::<PackedFormat>();
        assert_send_sync::<Tweak>();
        assert_send_sync::<EffectDef>();
        assert_send_sync::<Opt>();
        assert_send_sync::<OptValue>();
        assert_send_sync::<ScopedValue>();
        assert_send_sync::<ActionDef>();
        assert_send_sync::<Script>();
        assert_send_sync::<Shell>();
        assert_send_sync::<WindowsScope>();
        assert_send_sync::<BuildExpr>();
        assert_send_sync::<SharedDef>();
        assert_send_sync::<EffectId>();
        assert_send_sync::<SharedId>();
        assert_send_sync::<OptLabel>();
    }
}
