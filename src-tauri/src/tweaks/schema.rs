//! YAML → compiled-model loader (spec §6). `#[cfg(test)]`-only (see `tweaks/mod.rs`): the shipped
//! binary never links `serde_yaml_bw`. Maps `serde_yaml_bw` nodes onto `parse::LiteralInput` and
//! calls the Task 2 parsers — path/literal/scope logic is never reimplemented here.
//!
//! Scope note: Registry, RegistryKey, Service, Task, Hosts, Shared-reference, and Action(Script)
//! authoring surfaces are wired up. `Firewall` is deliberately NOT wired — `RuleAddr` is still the
//! Task 1 placeholder (`{ name: String }`); its full rule definition is a Task 7 decision.
//! `ActionDef::DeleteTree`, `windows:` scoping, and per-effect `optional`/`if_missing`/`elevation`
//! are real compiled-model fields with no YAML mapping yet — deferred to whichever task first
//! needs to author them (see the Task 3 report's Deviations).

use super::model::{
    ActionDef, CategoryDef, Corpus, Effect, EffectDef, EffectId, FieldAddr, HostsAddr, KeyAddr,
    Level, Opt, OptLabel, OptValue, PackedFormat, RegAddr, RegType, RiskLevel, ScopedValue, Script,
    Setting, SharedDef, SharedId, Shell, StartupType, SvcAddr, TaskAddr, Tweak, Value,
};
use super::parse::{parse_reg_path, parse_value_literal, LiteralInput, LiteralTarget, ParseError};
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
    Shared(SharedRefEffectRaw),
    Action(ActionEffectRaw),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryEffectRaw {
    id: String,
    registry: RegistryRaw,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryKeyEffectRaw {
    id: String,
    registry_key: RegistryKeyRaw,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceEffectRaw {
    id: String,
    service: ServiceRaw,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskEffectRaw {
    id: String,
    task: TaskRaw,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostsEffectRaw {
    id: String,
    hosts: HostsRaw,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedRefEffectRaw {
    id: String,
    shared: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionEffectRaw {
    id: String,
    action: ActionRaw,
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
        Setting::RegistryKey(_) | Setting::Hosts(_) => {
            let input = classify(node)?;
            parse_value_literal(input, LiteralTarget::Presence).map_err(|e| e.to_string())
        }
        Setting::Firewall(_) => {
            unreachable!("schema.rs does not construct Firewall yet — Task 7 decision")
        }
    }
}

/// Parses one `options: [...].values` entry against the effect it addresses.
fn option_value_for(kind: &Effect, node: &YamlValue) -> Result<OptValue, String> {
    match kind {
        Effect::Setting(setting) => setting_value_for(setting, node).map(|value| {
            OptValue::Set(ScopedValue {
                value,
                windows: None,
            })
        }),
        Effect::Shared(_) => match node.as_str() {
            Some("claim") => Ok(OptValue::Claim),
            Some("unclaimed") => Ok(OptValue::Unclaimed),
            _ => Err("a shared effect's value must be `claim` or `unclaimed`".to_string()),
        },
        Effect::Action(_) => match node.as_str() {
            Some("run") => Ok(OptValue::Run),
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
    match kind {
        Ok(kind) => Some(EffectDef {
            id: EffectId(id.to_string()),
            kind,
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }),
        Err(e) => {
            errors.push(e);
            None
        }
    }
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
        windows: None,
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

        // Two files merged: two distinct categories, each tweak stamped with its own file's id.
        let mut category_ids: Vec<&str> = corpus.categories.iter().map(|c| c.id.as_str()).collect();
        category_ids.sort_unstable();
        assert_eq!(category_ids, vec!["category_a", "category_b"]);

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
        assert_eq!(claimed, Some(OptValue::Claim));
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
