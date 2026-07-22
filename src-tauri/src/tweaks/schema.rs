//! YAML → compiled-model loader (spec §6). `#[cfg(test)]`-only (see `tweaks/mod.rs`): the shipped
//! binary never links `serde_yaml_bw`. Maps `serde_yaml_bw` nodes onto `parse::LiteralInput` and
//! calls the Task 2 parsers — path/literal/scope logic is never reimplemented here.
//!
//! Scope note: Registry, RegistryKey, Service, Task, Hosts, Firewall, Shared-reference, and
//! Action(Script) authoring surfaces are wired up, along with tweak/effect/option-value `windows:`
//! scoping and per-effect `optional`/`if_missing`/`elevation` (Task 4). `Firewall`'s `RuleAddr` was
//! settled by Task 7 against the real `firewall_service` primitive (see `tweaks/model.rs`); `shared:`
//! does not offer a Firewall variant — nothing in spec §6.5 asks for one, and adding it is deferred
//! until an author actually needs it. `ActionDef::DeleteTree` has no YAML mapping yet — deferred to
//! whichever task first needs to author it.

use super::model::{
    ActionDef, CategoryDef, Corpus, Effect, EffectDef, EffectId, FieldAddr, FwAction, FwDirection,
    FwProtocol, HostsAddr, KeyAddr, Level, Opt, OptLabel, OptValue, PackedFormat, RegAddr, RegType,
    RiskLevel, RuleAddr, ScopedValue, Script, Setting, SharedDef, SharedId, Shell, StartupType,
    SvcAddr, TaskAddr, Tweak, Value, WindowsScope,
};
use super::parse::{
    expand_product, parse_build_expr, parse_reg_path, parse_value_literal, validate_windows_scope,
    LiteralInput, LiteralTarget, ParseError,
};
use super::validate::ValidationError;
use serde::Deserialize;
use serde_yaml_bw::Value as YamlValue;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------------------------
// YAML-facing DTOs. Every struct denies unknown fields (a typo'd key is a build error, never
// silently ignored); enums use the authoring keywords from spec §6.2/§6.4/§9, not Rust's default
// PascalCase — `model`'s own derives serialize for internal round-tripping, not for authors.
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusFileRaw {
    category: CategoryRaw,
    tweaks: Vec<TweakRaw>,
    #[serde(default)]
    shared: Vec<SharedRaw>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoryRaw {
    id: String,
    name: String,
    icon: String,
    description: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TweakRaw {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    info: Option<String>,
    #[serde(default)]
    warning: Option<String>,
    #[serde(default)]
    requires_reboot: bool,
    risk_level: RiskLevelRaw,
    elevation: LevelRaw,
    reversible: bool,
    effects: Vec<EffectRaw>,
    options: Vec<OptRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}

/// A `windows:` block, authored identically at tweak/effect/option-value level (spec §6.6).
/// `build`/`revision` stay strings here — parsed by the shared `parse_build_expr` grammar in
/// [`convert_windows_scope`], never reimplemented.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowsRaw {
    #[serde(default)]
    products: Option<Vec<u8>>,
    #[serde(default)]
    build: Option<String>,
    #[serde(default)]
    revision: Option<String>,
}

/// Converts one `windows:` block via the Task 2 grammar (spec §6.6) — never reimplemented here.
/// `products` are validated with [`expand_product`] but stored raw: the compiled model keeps the
/// authored ids (`WindowsScope::products: Option<Vec<u8>>`); expansion is a guard/runtime concern.
fn convert_windows_scope(raw: &WindowsRaw) -> Result<WindowsScope, ParseError> {
    let build = raw.build.as_deref().map(parse_build_expr).transpose()?;
    let revision = raw.revision.as_deref().map(parse_build_expr).transpose()?;
    if let Some(products) = &raw.products {
        for &product in products {
            expand_product(product)?;
        }
    }
    let scope = WindowsScope {
        products: raw.products.clone(),
        build,
        revision,
    };
    validate_windows_scope(&scope)?;
    Ok(scope)
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RiskLevelRaw {
    Low,
    Medium,
    High,
    Critical,
}

impl From<RiskLevelRaw> for RiskLevel {
    fn from(raw: RiskLevelRaw) -> Self {
        match raw {
            RiskLevelRaw::Low => RiskLevel::Low,
            RiskLevelRaw::Medium => RiskLevel::Medium,
            RiskLevelRaw::High => RiskLevel::High,
            RiskLevelRaw::Critical => RiskLevel::Critical,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LevelRaw {
    User,
    Admin,
    System,
    Ti,
}

impl From<LevelRaw> for Level {
    fn from(raw: LevelRaw) -> Self {
        match raw {
            LevelRaw::User => Level::User,
            LevelRaw::Admin => Level::Admin,
            LevelRaw::System => Level::System,
            LevelRaw::Ti => Level::Ti,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
enum RegTypeRaw {
    #[serde(rename = "REG_DWORD")]
    Dword,
    #[serde(rename = "REG_QWORD")]
    Qword,
    #[serde(rename = "REG_SZ")]
    Sz,
    #[serde(rename = "REG_EXPAND_SZ")]
    ExpandSz,
    #[serde(rename = "REG_MULTI_SZ")]
    MultiSz,
    #[serde(rename = "REG_BINARY")]
    Binary,
}

impl From<RegTypeRaw> for RegType {
    fn from(raw: RegTypeRaw) -> Self {
        match raw {
            RegTypeRaw::Dword => RegType::Dword,
            RegTypeRaw::Qword => RegType::Qword,
            RegTypeRaw::Sz => RegType::Sz,
            RegTypeRaw::ExpandSz => RegType::ExpandSz,
            RegTypeRaw::MultiSz => RegType::MultiSz,
            RegTypeRaw::Binary => RegType::Binary,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PackedFormatRaw {
    KvSemicolon,
}

impl From<PackedFormatRaw> for PackedFormat {
    fn from(raw: PackedFormatRaw) -> Self {
        match raw {
            PackedFormatRaw::KvSemicolon => PackedFormat::KvSemicolon,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ShellRaw {
    Cmd,
    #[serde(rename = "powershell")]
    PowerShell,
}

impl From<ShellRaw> for Shell {
    fn from(raw: ShellRaw) -> Self {
        match raw {
            ShellRaw::Cmd => Shell::Cmd,
            ShellRaw::PowerShell => Shell::PowerShell,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryRaw {
    key: String,
    name: String,
    #[serde(rename = "type")]
    ty: RegTypeRaw,
    #[serde(default)]
    field: Option<String>,
    #[serde(default)]
    format: Option<PackedFormatRaw>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceRaw {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskRaw {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryKeyRaw {
    key: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostsRaw {
    ip: String,
    domain: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FwDirectionRaw {
    Inbound,
    Outbound,
}

impl From<FwDirectionRaw> for FwDirection {
    fn from(raw: FwDirectionRaw) -> Self {
        match raw {
            FwDirectionRaw::Inbound => FwDirection::Inbound,
            FwDirectionRaw::Outbound => FwDirection::Outbound,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FwActionRaw {
    Block,
    Allow,
}

impl From<FwActionRaw> for FwAction {
    fn from(raw: FwActionRaw) -> Self {
        match raw {
            FwActionRaw::Block => FwAction::Block,
            FwActionRaw::Allow => FwAction::Allow,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FwProtocolRaw {
    Any,
    Tcp,
    Udp,
    Icmpv4,
    Icmpv6,
}

impl From<FwProtocolRaw> for FwProtocol {
    fn from(raw: FwProtocolRaw) -> Self {
        match raw {
            FwProtocolRaw::Any => FwProtocol::Any,
            FwProtocolRaw::Tcp => FwProtocol::Tcp,
            FwProtocolRaw::Udp => FwProtocol::Udp,
            FwProtocolRaw::Icmpv4 => FwProtocol::Icmpv4,
            FwProtocolRaw::Icmpv6 => FwProtocol::Icmpv6,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FirewallRaw {
    name: String,
    direction: FwDirectionRaw,
    action: FwActionRaw,
    #[serde(default)]
    protocol: Option<FwProtocolRaw>,
    #[serde(default)]
    program: Option<String>,
    #[serde(default)]
    service: Option<String>,
    #[serde(default)]
    remote_addresses: Option<Vec<String>>,
    #[serde(default)]
    remote_ports: Option<String>,
    #[serde(default)]
    local_ports: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionRaw {
    apply: String,
    #[serde(default)]
    undo: Option<String>,
    #[serde(default)]
    probe: Option<String>,
    #[serde(default)]
    ephemeral: bool,
    shell: ShellRaw,
}

/// One `effects:` entry: `id` plus exactly one kind field. An untagged enum makes "zero or two
/// kind fields" a plain deserialize failure (surfaced as `ValidationError::Yaml`) instead of a
/// hand-rolled "exactly one Some" check — the shape is unrepresentable by construction.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EffectRaw {
    Registry(RegistryEffectRaw),
    RegistryKey(RegistryKeyEffectRaw),
    Service(ServiceEffectRaw),
    Task(TaskEffectRaw),
    Hosts(HostsEffectRaw),
    Firewall(FirewallEffectRaw),
    Shared(SharedRefEffectRaw),
    Action(ActionEffectRaw),
}

impl EffectRaw {
    fn windows(&self) -> Option<&WindowsRaw> {
        match self {
            EffectRaw::Registry(r) => r.windows.as_ref(),
            EffectRaw::RegistryKey(r) => r.windows.as_ref(),
            EffectRaw::Service(r) => r.windows.as_ref(),
            EffectRaw::Task(r) => r.windows.as_ref(),
            EffectRaw::Hosts(r) => r.windows.as_ref(),
            EffectRaw::Firewall(r) => r.windows.as_ref(),
            EffectRaw::Shared(r) => r.windows.as_ref(),
            EffectRaw::Action(r) => r.windows.as_ref(),
        }
    }

    fn elevation(&self) -> Option<LevelRaw> {
        match self {
            EffectRaw::Registry(r) => r.elevation,
            EffectRaw::RegistryKey(r) => r.elevation,
            EffectRaw::Service(r) => r.elevation,
            EffectRaw::Task(r) => r.elevation,
            EffectRaw::Hosts(r) => r.elevation,
            EffectRaw::Firewall(r) => r.elevation,
            EffectRaw::Shared(r) => r.elevation,
            EffectRaw::Action(r) => r.elevation,
        }
    }

    fn optional(&self) -> bool {
        match self {
            EffectRaw::Registry(r) => r.optional,
            EffectRaw::RegistryKey(r) => r.optional,
            EffectRaw::Service(r) => r.optional,
            EffectRaw::Task(r) => r.optional,
            EffectRaw::Hosts(r) => r.optional,
            EffectRaw::Firewall(r) => r.optional,
            EffectRaw::Shared(_) | EffectRaw::Action(_) => false,
        }
    }

    /// Only the six Setting-kind variants carry `if_missing` (spec §5.4 is a Setting-presence
    /// concept) — Action/Shared authoring a stray `if_missing` is rejected as an unknown field.
    fn if_missing(&self) -> Option<&YamlValue> {
        match self {
            EffectRaw::Registry(r) => r.if_missing.as_ref(),
            EffectRaw::RegistryKey(r) => r.if_missing.as_ref(),
            EffectRaw::Service(r) => r.if_missing.as_ref(),
            EffectRaw::Task(r) => r.if_missing.as_ref(),
            EffectRaw::Hosts(r) => r.if_missing.as_ref(),
            EffectRaw::Firewall(r) => r.if_missing.as_ref(),
            EffectRaw::Shared(_) | EffectRaw::Action(_) => None,
        }
    }
}

// `optional`/`if_missing` are only meaningful on a Setting's own domain (spec §5.4 — a resource
// that can be genuinely Missing), so `if_missing` is wired only on the six Setting-kind variants.
// `windows`/`elevation` apply to every kind (spec §6.6/§9) and so appear on all eight.

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryEffectRaw {
    id: String,
    registry: RegistryRaw,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    if_missing: Option<YamlValue>,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryKeyEffectRaw {
    id: String,
    registry_key: RegistryKeyRaw,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    if_missing: Option<YamlValue>,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceEffectRaw {
    id: String,
    service: ServiceRaw,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    if_missing: Option<YamlValue>,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskEffectRaw {
    id: String,
    task: TaskRaw,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    if_missing: Option<YamlValue>,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostsEffectRaw {
    id: String,
    hosts: HostsRaw,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    if_missing: Option<YamlValue>,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FirewallEffectRaw {
    id: String,
    firewall: FirewallRaw,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    if_missing: Option<YamlValue>,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedRefEffectRaw {
    id: String,
    shared: String,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionEffectRaw {
    id: String,
    action: ActionRaw,
    #[serde(default)]
    elevation: Option<LevelRaw>,
    #[serde(default)]
    windows: Option<WindowsRaw>,
}

/// One corpus-level `shared:` entry: `id` + exactly one Setting kind + the target `value`
/// (spec §6.5). Same untagged-enum shape rationale as [`EffectRaw`].
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SharedRaw {
    Registry(SharedRegistryRaw),
    RegistryKey(SharedRegistryKeyRaw),
    Service(SharedServiceRaw),
    Task(SharedTaskRaw),
    Hosts(SharedHostsRaw),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedRegistryRaw {
    id: String,
    registry: RegistryRaw,
    value: YamlValue,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedRegistryKeyRaw {
    id: String,
    registry_key: RegistryKeyRaw,
    value: YamlValue,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedServiceRaw {
    id: String,
    service: ServiceRaw,
    value: YamlValue,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedTaskRaw {
    id: String,
    task: TaskRaw,
    value: YamlValue,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedHostsRaw {
    id: String,
    hosts: HostsRaw,
    value: YamlValue,
}

impl SharedRaw {
    fn id(&self) -> &str {
        match self {
            SharedRaw::Registry(r) => &r.id,
            SharedRaw::RegistryKey(r) => &r.id,
            SharedRaw::Service(r) => &r.id,
            SharedRaw::Task(r) => &r.id,
            SharedRaw::Hosts(r) => &r.id,
        }
    }

    fn value(&self) -> &YamlValue {
        match self {
            SharedRaw::Registry(r) => &r.value,
            SharedRaw::RegistryKey(r) => &r.value,
            SharedRaw::Service(r) => &r.value,
            SharedRaw::Task(r) => &r.value,
            SharedRaw::Hosts(r) => &r.value,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptRaw {
    label: String,
    values: BTreeMap<String, YamlValue>,
}

// ---------------------------------------------------------------------------------------------
// YAML node → LiteralInput (spec §6.2 shape rules) and per-kind value dispatch.
// ---------------------------------------------------------------------------------------------

/// Classifies a raw YAML node's *shape* into a [`LiteralInput`], before any string content is
/// interpreted — the same shape-first discipline `parse::LiteralInput`'s own docs describe.
fn classify(node: &YamlValue) -> Result<LiteralInput, String> {
    match node {
        YamlValue::Null(_) => Ok(LiteralInput::Null),
        YamlValue::Number(n, _) => n
            .as_i64()
            .map(LiteralInput::Int)
            .ok_or_else(|| format!("{n:?} is not a supported integer literal")),
        YamlValue::String(s, _) if s == "absent" => Ok(LiteralInput::Absent),
        YamlValue::String(s, _) if s == "present" => Ok(LiteralInput::Present),
        YamlValue::String(s, _) => Ok(LiteralInput::Str(s.clone())),
        YamlValue::Sequence(seq) => seq
            .iter()
            .map(|item| item.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()
            .map(LiteralInput::List)
            .ok_or_else(|| "list entries must all be plain strings".to_string()),
        YamlValue::Mapping(map) if map.len() == 1 => node
            .get("literal")
            .and_then(|v| v.as_str())
            .map(|s| LiteralInput::Escape(s.to_string()))
            .ok_or_else(|| "a one-key map value must be `{ literal: <text> }`".to_string()),
        other => Err(format!("{other:?} is not a valid value literal")),
    }
}

fn parse_startup_type(text: &str) -> Result<StartupType, String> {
    match text {
        "boot" => Ok(StartupType::Boot),
        "system" => Ok(StartupType::System),
        "automatic" => Ok(StartupType::Automatic),
        "automatic_delayed" => Ok(StartupType::AutomaticDelayed),
        "manual" => Ok(StartupType::Manual),
        "disabled" => Ok(StartupType::Disabled),
        other => Err(format!(
            "{other:?} is not a valid service start type — use boot, system, automatic, automatic_delayed, manual, or disabled"
        )),
    }
}

fn parse_task_enabled(text: &str) -> Result<bool, String> {
    match text {
        "enabled" => Ok(true),
        "disabled" => Ok(false),
        other => Err(format!("{other:?} is not `enabled` or `disabled`")),
    }
}

/// Parses a value node against the domain `setting`'s kind requires (spec §6.2's kind-specific
/// keywords). Shared by option-value conversion and `shared:` block value conversion.
fn setting_value_for(setting: &Setting, node: &YamlValue) -> Result<Value, String> {
    match setting {
        Setting::Registry(addr) => {
            let input = classify(node)?;
            parse_value_literal(input, LiteralTarget::Reg(addr.ty)).map_err(|e| e.to_string())
        }
        Setting::Service(_) => {
            let text = node
                .as_str()
                .ok_or_else(|| "a service value must be a keyword string".to_string())?;
            parse_startup_type(text).map(Value::Startup)
        }
        Setting::Task(_) => {
            let text = node
                .as_str()
                .ok_or_else(|| "a task value must be `enabled` or `disabled`".to_string())?;
            parse_task_enabled(text).map(Value::TaskEnabled)
        }
        // Presence kinds: the `present`/`absent` keywords, same shape-first classification as a
        // registry literal, targeted at `LiteralTarget::Presence` instead of a `RegType`.
        Setting::RegistryKey(_) | Setting::Hosts(_) | Setting::Firewall(_) => {
            let input = classify(node)?;
            parse_value_literal(input, LiteralTarget::Presence).map_err(|e| e.to_string())
        }
    }
}

/// Splits a per-option-value node into its literal-value node and an optional per-value
/// `windows:` scope — the third scoping level (spec §6.6): `{ value: <literal>, windows: {...} }`.
/// A bare literal (including the `{ literal: ... }` escape, spec §6.2 — its only key is `literal`,
/// never `value`) passes through unscoped; `classify` still owns that shape untouched.
fn split_scoped_node(node: &YamlValue) -> Result<(&YamlValue, Option<WindowsScope>), String> {
    let YamlValue::Mapping(map) = node else {
        return Ok((node, None));
    };
    if node.get("value").is_none() {
        return Ok((node, None));
    }
    if let Some((key, _)) = map
        .iter()
        .find(|(k, _)| !matches!(k.as_str(), Some("value" | "windows")))
    {
        return Err(format!(
            "{key:?} is not a valid key here — a mapped option value only accepts `value` and `windows`"
        ));
    }
    let windows = node
        .get("windows")
        .map(|w| {
            let raw: WindowsRaw = serde_yaml_bw::from_value(w.clone())
                .map_err(|e| format!("invalid windows scope: {e}"))?;
            convert_windows_scope(&raw).map_err(|e| e.to_string())
        })
        .transpose()?;
    Ok((node.get("value").expect("checked above"), windows))
}

/// Parses one `options: [...].values` entry against the effect it addresses. The per-option-value
/// `windows:` scope (spec §6.6's third level) is extracted once, uniformly, for every effect kind —
/// not just Settings — since `Run`/`Claim`/`Unclaimed` carry that same optional scope (model.rs).
fn option_value_for(kind: &Effect, node: &YamlValue) -> Result<OptValue, String> {
    let (value_node, windows) = split_scoped_node(node)?;
    match kind {
        Effect::Setting(setting) => setting_value_for(setting, value_node)
            .map(|value| OptValue::Set(ScopedValue { value, windows })),
        Effect::Shared(_) => match value_node.as_str() {
            Some("claim") => Ok(OptValue::Claim(windows)),
            Some("unclaimed") => Ok(OptValue::Unclaimed(windows)),
            _ => Err("a shared effect's value must be `claim` or `unclaimed`".to_string()),
        },
        Effect::Action(_) => match value_node.as_str() {
            Some("run") => Ok(OptValue::Run(windows)),
            _ => Err("an action effect's value must be `run` (or omit it entirely)".to_string()),
        },
    }
}

// ---------------------------------------------------------------------------------------------
// Raw → compiled-model conversion. Every step accumulates into a shared error `Vec` rather than
// bailing on the first problem, so one `load_corpus` run surfaces everything wrong at once.
// ---------------------------------------------------------------------------------------------

fn convert_registry(raw: &RegistryRaw) -> Result<RegAddr, ParseError> {
    let (hive, path) = parse_reg_path(&raw.key)?;
    let field = raw.field.as_ref().map(|field| FieldAddr {
        field: field.clone(),
        format: raw.format.unwrap_or(PackedFormatRaw::KvSemicolon).into(),
    });
    Ok(RegAddr {
        hive,
        path,
        name: raw.name.clone(),
        ty: raw.ty.into(),
        field,
    })
}

fn convert_registry_key(raw: &RegistryKeyRaw) -> Result<KeyAddr, ParseError> {
    let (hive, path) = parse_reg_path(&raw.key)?;
    Ok(KeyAddr { hive, path })
}

fn convert_hosts(raw: &HostsRaw) -> HostsAddr {
    HostsAddr {
        ip: raw.ip.clone(),
        domain: raw.domain.clone(),
    }
}

fn convert_firewall(raw: &FirewallRaw) -> RuleAddr {
    RuleAddr {
        name: raw.name.clone(),
        direction: raw.direction.into(),
        action: raw.action.into(),
        protocol: raw.protocol.map(Into::into),
        program: raw.program.clone(),
        service: raw.service.clone(),
        remote_addresses: raw.remote_addresses.clone(),
        remote_ports: raw.remote_ports.clone(),
        local_ports: raw.local_ports.clone(),
        description: raw.description.clone(),
    }
}

fn convert_effect(
    raw: &EffectRaw,
    tweak_id: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<EffectDef> {
    let (id, kind): (&str, Result<Effect, ValidationError>) = match raw {
        EffectRaw::Registry(r) => (
            &r.id,
            convert_registry(&r.registry)
                .map(|addr| Effect::Setting(Setting::Registry(addr)))
                .map_err(|source| ValidationError::InvalidAddress {
                    tweak: tweak_id.to_string(),
                    effect: EffectId(r.id.clone()),
                    source,
                }),
        ),
        EffectRaw::RegistryKey(r) => (
            &r.id,
            convert_registry_key(&r.registry_key)
                .map(|addr| Effect::Setting(Setting::RegistryKey(addr)))
                .map_err(|source| ValidationError::InvalidAddress {
                    tweak: tweak_id.to_string(),
                    effect: EffectId(r.id.clone()),
                    source,
                }),
        ),
        EffectRaw::Service(r) => (
            &r.id,
            Ok(Effect::Setting(Setting::Service(SvcAddr {
                name: r.service.name.clone(),
            }))),
        ),
        EffectRaw::Task(r) => (
            &r.id,
            Ok(Effect::Setting(Setting::Task(TaskAddr {
                path: r.task.path.clone(),
            }))),
        ),
        EffectRaw::Hosts(r) => (
            &r.id,
            Ok(Effect::Setting(Setting::Hosts(convert_hosts(&r.hosts)))),
        ),
        EffectRaw::Firewall(r) => (
            &r.id,
            Ok(Effect::Setting(Setting::Firewall(convert_firewall(
                &r.firewall,
            )))),
        ),
        EffectRaw::Shared(r) => (&r.id, Ok(Effect::Shared(SharedId(r.shared.clone())))),
        EffectRaw::Action(r) => (
            &r.id,
            Ok(Effect::Action(ActionDef::Script {
                apply: Script(r.action.apply.clone()),
                undo: r.action.undo.clone().map(Script),
                probe: r.action.probe.clone().map(Script),
                ephemeral: r.action.ephemeral,
                shell: r.action.shell.into(),
            })),
        ),
    };
    let kind = match kind {
        Ok(kind) => kind,
        Err(e) => {
            errors.push(e);
            return None;
        }
    };

    let mut ok = true;
    let windows = match raw.windows() {
        Some(w) => match convert_windows_scope(w) {
            Ok(scope) => Some(scope),
            Err(source) => {
                errors.push(ValidationError::InvalidWindowsScope {
                    tweak: tweak_id.to_string(),
                    context: format!("effect `{id}` windows scope"),
                    source,
                });
                ok = false;
                None
            }
        },
        None => None,
    };

    // Safe by construction: `if_missing()` only returns `Some` for the five Setting-kind
    // variants, and their `kind` conversion above always produces `Effect::Setting(_)`.
    let if_missing = match raw.if_missing() {
        Some(node) => {
            let Effect::Setting(setting) = &kind else {
                unreachable!("if_missing is only wired on Setting-kind EffectRaw variants")
            };
            match setting_value_for(setting, node) {
                Ok(value) => Some(value),
                Err(reason) => {
                    errors.push(ValidationError::InvalidIfMissing {
                        tweak: tweak_id.to_string(),
                        effect: EffectId(id.to_string()),
                        reason,
                    });
                    ok = false;
                    None
                }
            }
        }
        None => None,
    };

    ok.then_some(EffectDef {
        id: EffectId(id.to_string()),
        kind,
        elevation: raw.elevation().map(Into::into),
        optional: raw.optional(),
        if_missing,
        windows,
    })
}

fn convert_option(
    raw: &OptRaw,
    surface: &[EffectDef],
    authored_ids: &std::collections::HashSet<&str>,
    tweak_id: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<Opt> {
    let mut ok = true;
    let mut values = BTreeMap::new();
    for (effect_id_str, node) in &raw.values {
        let Some(effect) = surface.iter().find(|e| &e.id.0 == effect_id_str) else {
            // An id that was authored but failed its own conversion already has an error (e.g.
            // InvalidAddress) — reporting it again here as "unknown" would be a confusing
            // cascade. Only a genuine typo (never authored at all) is reported here.
            if !authored_ids.contains(effect_id_str.as_str()) {
                errors.push(ValidationError::InvalidOptionValue {
                    tweak: tweak_id.to_string(),
                    option: OptLabel(raw.label.clone()),
                    effect: EffectId(effect_id_str.clone()),
                    reason: "no effect with this id is declared on the tweak's surface".to_string(),
                });
                ok = false;
            }
            continue;
        };
        match option_value_for(&effect.kind, node) {
            Ok(value) => {
                values.insert(effect.id.clone(), value);
            }
            Err(reason) => {
                errors.push(ValidationError::InvalidOptionValue {
                    tweak: tweak_id.to_string(),
                    option: OptLabel(raw.label.clone()),
                    effect: effect.id.clone(),
                    reason,
                });
                ok = false;
            }
        }
    }
    ok.then_some(Opt {
        label: OptLabel(raw.label.clone()),
        values,
    })
}

fn effect_id_of(raw: &EffectRaw) -> &str {
    match raw {
        EffectRaw::Registry(r) => &r.id,
        EffectRaw::RegistryKey(r) => &r.id,
        EffectRaw::Service(r) => &r.id,
        EffectRaw::Task(r) => &r.id,
        EffectRaw::Hosts(r) => &r.id,
        EffectRaw::Firewall(r) => &r.id,
        EffectRaw::Shared(r) => &r.id,
        EffectRaw::Action(r) => &r.id,
    }
}

fn convert_tweak(
    raw: &TweakRaw,
    category: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<Tweak> {
    let mut ok = true;
    let authored_ids: std::collections::HashSet<&str> =
        raw.effects.iter().map(effect_id_of).collect();

    let mut surface = Vec::with_capacity(raw.effects.len());
    for effect_raw in &raw.effects {
        match convert_effect(effect_raw, &raw.id, errors) {
            Some(def) => surface.push(def),
            None => ok = false,
        }
    }

    let mut options = Vec::with_capacity(raw.options.len());
    for opt_raw in &raw.options {
        match convert_option(opt_raw, &surface, &authored_ids, &raw.id, errors) {
            Some(opt) => options.push(opt),
            None => ok = false,
        }
    }

    let windows = match &raw.windows {
        Some(w) => match convert_windows_scope(w) {
            Ok(scope) => Some(scope),
            Err(source) => {
                errors.push(ValidationError::InvalidWindowsScope {
                    tweak: raw.id.clone(),
                    context: "windows scope".to_string(),
                    source,
                });
                ok = false;
                None
            }
        },
        None => None,
    };

    if !ok {
        return None;
    }
    Some(Tweak {
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        category: category.to_string(),
        info: raw.info.clone(),
        warning: raw.warning.clone(),
        requires_reboot: raw.requires_reboot,
        risk_level: raw.risk_level.into(),
        elevation: raw.elevation.into(),
        reversible: raw.reversible,
        surface,
        options,
        windows,
    })
}

fn convert_shared(raw: &SharedRaw, errors: &mut Vec<ValidationError>) -> Option<SharedDef> {
    let setting_result: Result<Setting, ParseError> = match raw {
        SharedRaw::Registry(r) => convert_registry(&r.registry).map(Setting::Registry),
        SharedRaw::RegistryKey(r) => {
            convert_registry_key(&r.registry_key).map(Setting::RegistryKey)
        }
        SharedRaw::Service(r) => Ok(Setting::Service(SvcAddr {
            name: r.service.name.clone(),
        })),
        SharedRaw::Task(r) => Ok(Setting::Task(TaskAddr {
            path: r.task.path.clone(),
        })),
        SharedRaw::Hosts(r) => Ok(Setting::Hosts(convert_hosts(&r.hosts))),
    };
    let setting = match setting_result {
        Ok(s) => s,
        Err(source) => {
            errors.push(ValidationError::InvalidSharedSetting {
                id: SharedId(raw.id().to_string()),
                reason: source.to_string(),
            });
            return None;
        }
    };
    let value = match setting_value_for(&setting, raw.value()) {
        Ok(v) => v,
        Err(reason) => {
            errors.push(ValidationError::InvalidSharedSetting {
                id: SharedId(raw.id().to_string()),
                reason,
            });
            return None;
        }
    };
    Some(SharedDef {
        id: SharedId(raw.id().to_string()),
        setting,
        value,
    })
}

/// Every `*.yaml`/`*.yml` file directly inside `path`, sorted for deterministic load order — or
/// `path` itself, if it names a file rather than a directory (lets a single fixture load in
/// isolation without a wrapping directory per fixture).
fn collect_yaml_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    files.sort();
    files
}

/// Loads every corpus file under `dir` (or `dir` itself, if it is a single file) into one
/// [`Corpus`]: categories stamped onto their file's tweaks, shared declarations merged
/// corpus-wide. Accumulates every load-time problem (bad YAML, bad paths/literals) instead of
/// stopping at the first — an author sees everything wrong in one run.
pub fn load_corpus(dir: &Path) -> Result<Corpus, Vec<ValidationError>> {
    if !dir.exists() {
        return Err(vec![ValidationError::Yaml {
            file: dir.display().to_string(),
            message: "path does not exist".to_string(),
        }]);
    }

    let mut errors = Vec::new();
    let mut categories = Vec::new();
    let mut tweaks = Vec::new();
    let mut shared = Vec::new();

    for file in collect_yaml_files(dir) {
        let file_label = file.display().to_string();
        let content = match std::fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => {
                errors.push(ValidationError::Yaml {
                    file: file_label,
                    message: e.to_string(),
                });
                continue;
            }
        };
        let raw: CorpusFileRaw = match serde_yaml_bw::from_str(&content) {
            Ok(r) => r,
            Err(e) => {
                errors.push(ValidationError::Yaml {
                    file: file_label,
                    message: e.to_string(),
                });
                continue;
            }
        };

        let category_id = raw.category.id.clone();
        categories.push(CategoryDef {
            id: raw.category.id,
            name: raw.category.name,
            icon: raw.category.icon,
            description: raw.category.description,
        });
        for tweak_raw in &raw.tweaks {
            if let Some(tweak) = convert_tweak(tweak_raw, &category_id, &mut errors) {
                tweaks.push(tweak);
            }
        }
        for shared_raw in &raw.shared {
            if let Some(shared_def) = convert_shared(shared_raw, &mut errors) {
                shared.push(shared_def);
            }
        }
    }

    if errors.is_empty() {
        Ok(Corpus {
            categories,
            tweaks,
            shared,
        })
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(rel: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tweaks_fixtures")
            .join(rel)
    }

    #[test]
    fn good_corpus_loads_with_correct_shape() {
        let corpus = load_corpus(&fixture("good")).expect("the good fixture corpus must load");

        // Three files merged: three distinct categories, each tweak stamped with its own file's id.
        let mut category_ids: Vec<&str> = corpus.categories.iter().map(|c| c.id.as_str()).collect();
        category_ids.sort_unstable();
        assert_eq!(
            category_ids,
            vec!["category_a", "category_b", "differ_only_by_undo_action_ok"]
        );

        let update_tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "windows_update_control")
            .expect("windows_update_control tweak must load");
        assert_eq!(update_tweak.category, "category_a");
        assert_eq!(update_tweak.surface.len(), 4);
        assert!(update_tweak.reversible);

        let no_auto_update = update_tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "no_auto_update")
            .expect("no_auto_update effect must exist");
        let Effect::Setting(Setting::Registry(addr)) = &no_auto_update.kind else {
            panic!("expected a Registry setting");
        };
        assert_eq!(addr.name, "NoAutoUpdate");
        assert_eq!(addr.ty, RegType::Dword);

        let fully_disabled = update_tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Fully Disabled")
            .expect("Fully Disabled option must exist");
        let no_auto_update_value = fully_disabled.values.get(&no_auto_update.id).cloned();
        assert_eq!(
            no_auto_update_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Reg(super::super::model::TypedRegValue::Dword(1)),
                windows: None,
            }))
        );

        // Cross-file shared merge + explicit claim/unclaimed.
        assert_eq!(corpus.shared.len(), 1);
        let telemetry_tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "disable_telemetry")
            .expect("disable_telemetry tweak must load from the second file");
        assert_eq!(telemetry_tweak.category, "category_b");
        let telemetry_effect = telemetry_tweak
            .surface
            .iter()
            .find(|e| matches!(e.kind, Effect::Shared(_)))
            .expect("a Shared effect must exist");
        let claimed = telemetry_tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Disabled")
            .and_then(|o| o.values.get(&telemetry_effect.id).cloned());
        assert_eq!(claimed, Some(OptValue::Claim(None)));
    }

    /// Review fix: proves the RegistryKey and Hosts presence kinds actually load, resolving
    /// `present`/`absent` to `Value::Present(true)`/`Value::Present(false)`.
    #[test]
    fn good_corpus_loads_registry_key_and_hosts_presence_kinds() {
        let corpus = load_corpus(&fixture("good")).expect("the good fixture corpus must load");

        let key_tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "enable_feature_key")
            .expect("enable_feature_key tweak must load");
        let key_effect = key_tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "feature_key")
            .expect("feature_key effect must exist");
        let Effect::Setting(Setting::RegistryKey(key_addr)) = &key_effect.kind else {
            panic!("expected a RegistryKey setting");
        };
        assert_eq!(key_addr.path, "SOFTWARE\\ExampleCo\\FeatureFlag");
        let present_value = key_tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Present")
            .and_then(|o| o.values.get(&key_effect.id).cloned());
        assert_eq!(
            present_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Present(true),
                windows: None,
            }))
        );
        let absent_value = key_tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Absent")
            .and_then(|o| o.values.get(&key_effect.id).cloned());
        assert_eq!(
            absent_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Present(false),
                windows: None,
            }))
        );

        let hosts_tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "block_known_tracker")
            .expect("block_known_tracker tweak must load");
        let hosts_effect = hosts_tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "tracker_host")
            .expect("tracker_host effect must exist");
        let Effect::Setting(Setting::Hosts(hosts_addr)) = &hosts_effect.kind else {
            panic!("expected a Hosts setting");
        };
        assert_eq!(hosts_addr.ip, "0.0.0.0");
        assert_eq!(hosts_addr.domain, "tracker.example.com");
        let blocked_value = hosts_tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Blocked")
            .and_then(|o| o.values.get(&hosts_effect.id).cloned());
        assert_eq!(
            blocked_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Present(true),
                windows: None,
            }))
        );
    }

    /// Task 7: the `firewall:` effect authoring surface — full `RuleAddr` definition round-trips
    /// from YAML, and `present`/`absent` resolve to `Value::Present(bool)` like the other presence
    /// kinds.
    #[test]
    fn good_corpus_loads_firewall_presence_kind() {
        let corpus = load_corpus(&fixture("good")).expect("the good fixture corpus must load");

        let tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "block_bad_actor_ip")
            .expect("block_bad_actor_ip tweak must load");
        let effect = tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "bad_actor_rule")
            .expect("bad_actor_rule effect must exist");
        let Effect::Setting(Setting::Firewall(addr)) = &effect.kind else {
            panic!("expected a Firewall setting");
        };
        assert_eq!(addr.name, "MagicX Block Bad Actor");
        assert_eq!(addr.direction, FwDirection::Outbound);
        assert_eq!(addr.action, FwAction::Block);
        assert_eq!(addr.protocol, Some(FwProtocol::Tcp));
        assert_eq!(
            addr.remote_addresses,
            Some(vec!["203.0.113.0/24".to_string()])
        );
        assert_eq!(
            addr.description,
            Some("Blocks outbound TCP to a known bad actor range.".to_string())
        );

        let blocked_value = tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Blocked")
            .and_then(|o| o.values.get(&effect.id).cloned());
        assert_eq!(
            blocked_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Present(true),
                windows: None,
            }))
        );
        let allowed_value = tweak
            .options
            .iter()
            .find(|o| o.label.0 == "Allowed")
            .and_then(|o| o.values.get(&effect.id).cloned());
        assert_eq!(
            allowed_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Present(false),
                windows: None,
            }))
        );
    }

    /// Task 4 Half A: tweak/effect/option-value `windows:` scoping, per-effect `optional` +
    /// `if_missing`, and per-effect `elevation` all round-trip from YAML into the compiled model.
    #[test]
    fn good_corpus_loads_windows_scoping_and_presence() {
        let corpus = load_corpus(&fixture("good")).expect("the good fixture corpus must load");
        let tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "version_scoped_demo")
            .expect("version_scoped_demo tweak must load");

        assert_eq!(
            tweak.windows,
            Some(super::super::model::WindowsScope {
                products: Some(vec![10, 11]),
                build: None,
                revision: None,
            })
        );

        let modern_only = tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "modern_only_setting")
            .expect("modern_only_setting effect must exist");
        assert_eq!(
            modern_only.windows,
            Some(super::super::model::WindowsScope {
                products: None,
                build: Some(super::super::model::BuildExpr::Min(22621)),
                revision: None,
            })
        );
        assert_eq!(modern_only.elevation, Some(Level::Admin));
        assert!(!modern_only.optional);

        let legacy_service = tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "legacy_service")
            .expect("legacy_service effect must exist");
        assert!(legacy_service.optional);
        assert_eq!(
            legacy_service.if_missing,
            Some(Value::Startup(StartupType::Disabled))
        );

        let on_option = tweak
            .options
            .iter()
            .find(|o| o.label.0 == "On")
            .expect("On option must exist");
        let modern_only_value = on_option.values.get(&modern_only.id).cloned();
        assert_eq!(
            modern_only_value,
            Some(OptValue::Set(ScopedValue {
                value: Value::Reg(super::super::model::TypedRegValue::Dword(1)),
                windows: Some(super::super::model::WindowsScope {
                    products: None,
                    build: Some(super::super::model::BuildExpr::Min(26100)),
                    revision: None,
                }),
            }))
        );

        // Fix 2 (code review): per-option-value `windows:` scoping is not restricted to Settings —
        // `notify_action` (an Action) carries the same third-level scope on the `On` option.
        let notify_action = tweak
            .surface
            .iter()
            .find(|e| e.id.0 == "notify_action")
            .expect("notify_action effect must exist");
        let notify_action_value = on_option.values.get(&notify_action.id).cloned();
        assert_eq!(
            notify_action_value,
            Some(OptValue::Run(Some(super::super::model::WindowsScope {
                products: None,
                build: Some(super::super::model::BuildExpr::Min(26100)),
                revision: None,
            })))
        );
    }

    #[test]
    fn single_file_path_loads_in_isolation() {
        // Same mechanism the bad-fixture tests in validate.rs rely on: pointing load_corpus at
        // one file (not its directory) loads only that file.
        let corpus = load_corpus(&fixture("good/category_a.yaml")).expect("single file must load");
        assert_eq!(corpus.categories.len(), 1);
        assert!(corpus.tweaks.iter().all(|t| t.category == "category_a"));
    }

    #[test]
    fn missing_directory_is_a_load_error() {
        let result = load_corpus(&fixture("does_not_exist"));
        assert!(
            result.is_err(),
            "a nonexistent path must not silently produce an empty corpus"
        );
    }
}
