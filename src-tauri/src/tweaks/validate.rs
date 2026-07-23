//! Build-time guards (spec §10). Two parts:
//! - **Structural** (Task 3): ownership/duplicate address, kind canonicalization, option coverage,
//!   path/literal validity (via `ValidationError` variants that wrap `parse::ParseError`),
//!   reversibility honesty, TI self-availability, and `if_missing` requiring `optional` (Task 4).
//! - **Semantic** (Task 4, [`validate_semantic`]): per-milestone quantification over each tweak's
//!   applicable projection — detectability, and distinctness (byte, detectable-projection,
//!   non-shared, and the Residue rule).
//!
//! No YAML here: both entry points run over the already-loaded [`Corpus`], so this module compiles
//! into the shipped binary same as `model`/`parse` (only `schema.rs`, the YAML binding layer, is
//! test-only).

use super::model::{
    ActionDef, BuildExpr, Corpus, Effect, EffectDef, EffectId, Hive, Opt, OptLabel, OptValue,
    ScopedValue, Setting, SharedId, StartupType, Tweak, Value, WindowsScope,
};
use super::parse::{expand_product, ParseError};
use std::collections::{HashMap, HashSet};

/// Identifies which side of a [`ValidationError::DuplicateAddress`] conflict an owner is — a
/// tweak's direct effect, or a corpus-level `shared:` declaration (spec §10: counted together).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressOwner {
    Effect { tweak: String, effect: EffectId },
    Shared { id: SharedId },
}

impl std::fmt::Display for AddressOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressOwner::Effect { tweak, effect } => {
                write!(f, "tweak `{tweak}` effect `{effect}`")
            }
            AddressOwner::Shared { id } => write!(f, "shared `{id}`"),
        }
    }
}

/// Every rejection a build-time structural guard can raise. Each message names the fix — tweak
/// authors read these verbatim (no separate lookup table).
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// A corpus file failed to parse as YAML, or used an unknown/mistyped key
    /// (`#[serde(deny_unknown_fields)]`).
    #[error("{file}: {message}")]
    Yaml { file: String, message: String },

    /// An effect's own address (registry path, etc.) failed to parse.
    #[error("tweak `{tweak}` effect `{effect}`: {source}")]
    InvalidAddress {
        tweak: String,
        effect: EffectId,
        #[source]
        source: ParseError,
    },

    /// A corpus-level `shared:` declaration's address or value failed to parse.
    #[error("shared `{id}`: {reason}")]
    InvalidSharedSetting { id: SharedId, reason: String },

    /// An effect's `shared: <id>` names a `shared:` declaration that does not exist anywhere in
    /// the corpus (spec §10: "shared references resolve").
    #[error(
        "tweak `{tweak}` effect `{effect}` references shared `{shared}`, which no `shared:` block declares — check for a typo or add the missing entry"
    )]
    UnresolvedSharedRef {
        tweak: String,
        effect: EffectId,
        shared: SharedId,
    },

    /// An option supplied a value for an effect that does not parse, or does not match the shape
    /// its effect kind requires (e.g. an Action given anything but `run`).
    #[error("tweak `{tweak}` option `{option}` effect `{effect}`: {reason}")]
    InvalidOptionValue {
        tweak: String,
        option: OptLabel,
        effect: EffectId,
        reason: String,
    },

    /// Two owners (direct effects and/or `shared:` declarations) claim the same address —
    /// includes the whole-value-vs-field-addressed mix on one packed value (spec §10, ADR-0006).
    #[error(
        "{address} is claimed by both {first} and {second} — merge them into one effect, reassign one to a different address, or extract a corpus-level `shared:` entry if they must always agree"
    )]
    DuplicateAddress {
        address: String,
        first: AddressOwner,
        second: AddressOwner,
    },

    /// Two `shared:` declarations (anywhere in the corpus) reuse the same id.
    #[error(
        "shared id `{id}` is declared more than once — shared ids must be unique corpus-wide; rename one of the declarations"
    )]
    DuplicateSharedId { id: SharedId },

    /// A raw Registry effect addresses a value that a canonicalized kind (Service/Task) owns —
    /// closing the alias route the ownership guard would otherwise miss (spec §10).
    #[error(
        "tweak `{tweak}` effect `{effect}` addresses {path} as a raw registry value — use the `{use_kind}` kind instead"
    )]
    NonCanonicalKind {
        tweak: String,
        effect: EffectId,
        path: String,
        use_kind: &'static str,
    },

    /// An option's `values:` map has no entry for one of the tweak's Setting effects (spec §6.3).
    #[error(
        "tweak `{tweak}` option `{option}` does not cover effect `{effect}` — every option must supply a value for every Setting effect on the surface"
    )]
    MissingCoverage {
        tweak: String,
        option: OptLabel,
        effect: EffectId,
    },

    /// An option's `values:` map does not explicitly say `claim`/`unclaimed` for a shared effect
    /// (spec §6.3: omission is a build error, so sharing stays a visible decision).
    #[error(
        "tweak `{tweak}` option `{option}` does not explicitly say `claim` or `unclaimed` for shared effect `{effect}`"
    )]
    SharedNotExplicit {
        tweak: String,
        option: OptLabel,
        effect: EffectId,
    },

    /// The declared `reversible:` flag does not match the computed value (spec §6.4).
    #[error(
        "tweak `{tweak}` declares reversible: {declared} but the computed value is {computed} — reversible requires every effect to be a Setting, an undo-carrying Action, or an ephemeral Action"
    )]
    ReversibilityMismatch {
        tweak: String,
        declared: bool,
        computed: bool,
    },

    /// A typed Service effect disables the TrustedInstaller service itself (spec §10: the app's
    /// own TI elevation path must stay available; script contents are out of scope by design).
    #[error(
        "tweak `{tweak}` effect `{effect}` disables the TrustedInstaller service via a typed effect — this would strand the app's own TI elevation path"
    )]
    TrustedInstallerDisabled { tweak: String, effect: EffectId },

    /// A `windows:` block (tweak, effect, or option-value level, spec §6.6) failed to parse —
    /// a bad build/revision expression, an unknown product, or `revision` without a pinned build.
    #[error("tweak `{tweak}` {context}: {source}")]
    InvalidWindowsScope {
        tweak: String,
        context: String,
        #[source]
        source: ParseError,
    },

    /// An effect's `if_missing:` literal failed to parse against its own Setting domain (§5.4).
    #[error("tweak `{tweak}` effect `{effect}` if_missing: {reason}")]
    InvalidIfMissing {
        tweak: String,
        effect: EffectId,
        reason: String,
    },

    /// `if_missing:` declares a Missing-reading without `optional: true` — dead authoring, since a
    /// non-optional effect is a typed error the moment it reads Missing, never `if_missing` (§5.4).
    #[error(
        "tweak `{tweak}` effect `{effect}` declares if_missing without optional: true — add `optional: true`, or drop if_missing"
    )]
    IfMissingWithoutOptional { tweak: String, effect: EffectId },

    /// `ephemeral: true` takes no `undo`/`probe` (spec §7): a transient side-effect is *exempt*
    /// from reversibility/detectability, not merely allowed to skip them — carrying either would
    /// let a later engine call `run_undo`/`run_probe` on an action the tweak's own `reversible`/
    /// detectability computation never accounted for.
    #[error(
        "tweak `{tweak}` effect `{effect}` is ephemeral but declares undo/probe — an ephemeral action takes neither (spec §7)"
    )]
    EphemeralWithUndoOrProbe { tweak: String, effect: EffectId },

    /// spec §10 Detectability: this option has no non-optional detectable effect on `build` — an
    /// optional effect may read `Missing` on a real machine, so the option would become
    /// indistinguishable from every other state there. `build` is the first milestone this failed
    /// on (deduped across the support matrix — an author fixes the option, not each build).
    #[error(
        "tweak `{tweak}` option `{option}` has no non-optional detectable effect on Windows build {build} — every option must stay distinguishable without effects that may read Missing"
    )]
    NotDetectable {
        tweak: String,
        option: OptLabel,
        build: u32,
    },

    /// spec §10 Distinctness (1/3): two options are byte-identical over the whole applicable
    /// projection on `build` — nothing at all, of any kind, differs between them.
    #[error(
        "tweak `{tweak}` options `{first}` and `{second}` are byte-identical on Windows build {build} — merge them or give one a distinct value"
    )]
    OptionsByteIdentical {
        tweak: String,
        first: OptLabel,
        second: OptLabel,
        build: u32,
    },

    /// spec §10 Distinctness (2/3): two options differ, but only on effects that never
    /// contribute to detection (probe-less Actions) — `detect()` cannot tell them apart on `build`.
    #[error(
        "tweak `{tweak}` options `{first}` and `{second}` are identical on their detectable projection on Windows build {build} — they differ only by effects detection cannot observe (e.g. a probe-less Action)"
    )]
    OptionsNotDetectablyDistinct {
        tweak: String,
        first: OptLabel,
        second: OptLabel,
        build: u32,
    },

    /// spec §10 Distinctness (3/3): every differing effect between two options is a `shared`
    /// reference — a claim can be held by other tweaks too, so it cannot be the sole distinguisher.
    #[error(
        "tweak `{tweak}` options `{first}` and `{second}` differ only by a shared effect on Windows build {build} — a claimed shared value can be held by another tweak too, so it cannot be the sole distinguisher"
    )]
    SharedOnlyDistinguisher {
        tweak: String,
        first: OptLabel,
        second: OptLabel,
        build: u32,
    },

    /// The Residue rule (spec §8.4/§10): every differing effect is a no-undo (or probe-less, or
    /// shared) effect — none of them keeps at most one option matching once the state is reached,
    /// since a one-way action's Residue is tolerated by the omitting option (§8.4).
    #[error(
        "tweak `{tweak}` options `{first}` and `{second}` have no reliable distinguisher on Windows build {build} — add a Setting or an undo-carrying probeable Action that differs between them; a no-undo Action's Residue lets the omitting option match too once it has run"
    )]
    ResidueOnlyDistinguisher {
        tweak: String,
        first: OptLabel,
        second: OptLabel,
        build: u32,
    },
}

/// Runs every structural guard in scope for this task (spec §10) over an already-loaded corpus.
/// Detectability, distinctness, and per-milestone quantification are Task 4's job.
pub fn validate_structural(corpus: &Corpus) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    check_duplicate_shared_ids(corpus, &mut errors);
    check_shared_refs_resolve(corpus, &mut errors);
    check_ownership(corpus, &mut errors);
    check_canonicalization(corpus, &mut errors);
    for tweak in &corpus.tweaks {
        check_coverage(tweak, &mut errors);
        check_reversibility(tweak, &mut errors);
        check_ti_self_availability(tweak, &mut errors);
        check_if_missing_requires_optional(tweak, &mut errors);
        check_ephemeral_has_no_undo_probe(tweak, &mut errors);
    }
    errors
}

fn check_duplicate_shared_ids(corpus: &Corpus, errors: &mut Vec<ValidationError>) {
    let mut seen: HashSet<&SharedId> = HashSet::new();
    for shared in &corpus.shared {
        if !seen.insert(&shared.id) {
            errors.push(ValidationError::DuplicateSharedId {
                id: shared.id.clone(),
            });
        }
    }
}

fn check_shared_refs_resolve(corpus: &Corpus, errors: &mut Vec<ValidationError>) {
    let known: HashSet<&SharedId> = corpus.shared.iter().map(|s| &s.id).collect();
    for tweak in &corpus.tweaks {
        for effect in &tweak.surface {
            if let Effect::Shared(id) = &effect.kind {
                if !known.contains(id) {
                    errors.push(ValidationError::UnresolvedSharedRef {
                        tweak: tweak.id.clone(),
                        effect: effect.id.clone(),
                        shared: id.clone(),
                    });
                }
            }
        }
    }
}

/// The coarse grouping key for one-address-one-owner (spec §10, ADR-0006): registry values group
/// by (hive, path, name) *ignoring* the field, so a whole-value claim and any field claim on the
/// same value land in one group — exactly the whole-xor-field rule needs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CoarseKey {
    Registry(Hive, String, String),
    RegistryKey(Hive, String),
    Service(String),
    Task(String),
    Hosts(String, String),
    Firewall(String),
}

struct Claim {
    field: Option<String>,
    display: String,
    owner: AddressOwner,
}

fn coarse_key_and_field(setting: &Setting) -> (CoarseKey, Option<String>, String) {
    match setting {
        Setting::Registry(addr) => (
            CoarseKey::Registry(addr.hive, addr.path.clone(), addr.name.clone()),
            addr.field.as_ref().map(|f| f.field.clone()),
            format!("{:?}\\{}\\{}", addr.hive, addr.path, addr.name),
        ),
        Setting::RegistryKey(addr) => (
            CoarseKey::RegistryKey(addr.hive, addr.path.clone()),
            None,
            format!("{:?}\\{}", addr.hive, addr.path),
        ),
        Setting::Service(addr) => (
            CoarseKey::Service(addr.name.clone()),
            None,
            format!("service `{}`", addr.name),
        ),
        Setting::Task(addr) => (
            CoarseKey::Task(addr.path.clone()),
            None,
            format!("task `{}`", addr.path),
        ),
        Setting::Hosts(addr) => (
            CoarseKey::Hosts(addr.ip.clone(), addr.domain.clone()),
            None,
            format!("hosts entry `{} {}`", addr.ip, addr.domain),
        ),
        Setting::Firewall(addr) => (
            CoarseKey::Firewall(addr.name.clone()),
            None,
            format!("firewall rule `{}`", addr.name),
        ),
    }
}

/// One address, one owner, corpus-wide — direct effects and `shared` declarations counted
/// together; a packed value is whole-owned xor field-addressed (spec §10, ADR-0006).
fn check_ownership(corpus: &Corpus, errors: &mut Vec<ValidationError>) {
    let mut groups: HashMap<CoarseKey, Vec<Claim>> = HashMap::new();

    for tweak in &corpus.tweaks {
        for effect in &tweak.surface {
            if let Effect::Setting(setting) = &effect.kind {
                let (key, field, display) = coarse_key_and_field(setting);
                groups.entry(key).or_default().push(Claim {
                    field,
                    display,
                    owner: AddressOwner::Effect {
                        tweak: tweak.id.clone(),
                        effect: effect.id.clone(),
                    },
                });
            }
        }
    }
    for shared in &corpus.shared {
        let (key, field, display) = coarse_key_and_field(&shared.setting);
        groups.entry(key).or_default().push(Claim {
            field,
            display,
            owner: AddressOwner::Shared {
                id: shared.id.clone(),
            },
        });
    }

    // Deterministic reporting order regardless of HashMap iteration order.
    let mut group_list: Vec<Vec<Claim>> = groups.into_values().collect();
    group_list.sort_by(|a, b| a[0].display.cmp(&b[0].display));

    for claims in group_list {
        if claims.len() < 2 {
            continue;
        }
        if claims.iter().any(|c| c.field.is_none()) {
            // A whole-value (or non-registry) claim coexists with at least one other claim.
            // Every additional owner beyond the first must be surfaced — not just the second —
            // or a 3rd+ colliding owner silently escapes the guard (ADR-0006's core guarantee).
            for other in &claims[1..] {
                errors.push(ValidationError::DuplicateAddress {
                    address: claims[0].display.clone(),
                    first: claims[0].owner.clone(),
                    second: other.owner.clone(),
                });
            }
        } else {
            // All field-addressed: each field must be owned once.
            let mut field_seen: HashMap<&str, &AddressOwner> = HashMap::new();
            for claim in &claims {
                let field = claim
                    .field
                    .as_deref()
                    .expect("checked: all Some in this branch");
                if let Some(&prev) = field_seen.get(field) {
                    errors.push(ValidationError::DuplicateAddress {
                        address: format!("{}[{field}]", claims[0].display),
                        first: prev.clone(),
                        second: claim.owner.clone(),
                    });
                } else {
                    field_seen.insert(field, &claim.owner);
                }
            }
        }
    }
}

/// A raw Registry effect must not address a value that a canonicalized kind (Service/Task) owns —
/// otherwise ownership can be dodged via a second address space (spec §10).
fn check_canonicalization(corpus: &Corpus, errors: &mut Vec<ValidationError>) {
    for tweak in &corpus.tweaks {
        for effect in &tweak.surface {
            let Effect::Setting(Setting::Registry(addr)) = &effect.kind else {
                continue;
            };
            if addr.hive != Hive::Hklm {
                continue;
            }
            let use_kind = if is_service_start_value(&addr.path, &addr.name) {
                Some("Service")
            } else if is_task_scheduler_path(&addr.path) {
                Some("Task")
            } else {
                None
            };
            if let Some(use_kind) = use_kind {
                errors.push(ValidationError::NonCanonicalKind {
                    tweak: tweak.id.clone(),
                    effect: effect.id.clone(),
                    path: format!(r"HKLM\{}\{}", addr.path, addr.name),
                    use_kind,
                });
            }
        }
    }
}

/// A service's start type lives at `...\Services\<name>` with a value named `Start`.
fn is_service_start_value(path: &str, name: &str) -> bool {
    if !name.eq_ignore_ascii_case("Start") {
        return false;
    }
    let segments: Vec<&str> = path.split('\\').collect();
    segments.len() >= 2 && segments[segments.len() - 2].eq_ignore_ascii_case("Services")
}

/// The Task Scheduler's own registry storage tree.
fn is_task_scheduler_path(path: &str) -> bool {
    const PREFIX: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache";
    path.get(..PREFIX.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(PREFIX))
}

/// Every option covers every Setting effect; shared entries are explicit `claim`/`unclaimed`;
/// Action entries are `run` or omitted, so they need no coverage check (spec §6.3).
fn check_coverage(tweak: &Tweak, errors: &mut Vec<ValidationError>) {
    for opt in &tweak.options {
        for effect in &tweak.surface {
            match &effect.kind {
                Effect::Setting(_) => {
                    if !matches!(opt.values.get(&effect.id), Some(OptValue::Set(_))) {
                        errors.push(ValidationError::MissingCoverage {
                            tweak: tweak.id.clone(),
                            option: opt.label.clone(),
                            effect: effect.id.clone(),
                        });
                    }
                }
                Effect::Shared(_) => {
                    if !matches!(
                        opt.values.get(&effect.id),
                        Some(OptValue::Claim(_)) | Some(OptValue::Unclaimed(_))
                    ) {
                        errors.push(ValidationError::SharedNotExplicit {
                            tweak: tweak.id.clone(),
                            option: opt.label.clone(),
                            effect: effect.id.clone(),
                        });
                    }
                }
                Effect::Action(_) => {}
            }
        }
    }
}

/// `reversible` = every effect is a Setting, a Shared reference (reversible via claim/release,
/// §8.6), an undo-carrying Action, or an ephemeral Action (spec §6.4).
fn check_reversibility(tweak: &Tweak, errors: &mut Vec<ValidationError>) {
    let computed = tweak.surface.iter().all(|effect| match &effect.kind {
        Effect::Setting(_) | Effect::Shared(_) => true,
        Effect::Action(ActionDef::Script {
            undo, ephemeral, ..
        }) => undo.is_some() || *ephemeral,
        Effect::Action(ActionDef::DeleteTree { undo, .. }) => undo.is_some(),
    });
    if computed != tweak.reversible {
        errors.push(ValidationError::ReversibilityMismatch {
            tweak: tweak.id.clone(),
            declared: tweak.reversible,
            computed,
        });
    }
}

/// No typed Service effect may disable the TrustedInstaller service itself — script contents are
/// out of scope by design (spec §10).
fn check_ti_self_availability(tweak: &Tweak, errors: &mut Vec<ValidationError>) {
    for effect in &tweak.surface {
        let Effect::Setting(Setting::Service(addr)) = &effect.kind else {
            continue;
        };
        if !addr.name.eq_ignore_ascii_case("TrustedInstaller") {
            continue;
        }
        let disables = tweak.options.iter().any(|opt| {
            matches!(
                opt.values.get(&effect.id),
                Some(OptValue::Set(ScopedValue {
                    value: Value::Startup(StartupType::Disabled),
                    ..
                }))
            )
        });
        if disables {
            errors.push(ValidationError::TrustedInstallerDisabled {
                tweak: tweak.id.clone(),
                effect: effect.id.clone(),
            });
        }
    }
}

/// `if_missing:` only means something on an `optional` effect — a non-optional effect never reads
/// `Missing` (it is a typed error instead, §5.4), so `if_missing` there would be dead authoring.
fn check_if_missing_requires_optional(tweak: &Tweak, errors: &mut Vec<ValidationError>) {
    for effect in &tweak.surface {
        if effect.if_missing.is_some() && !effect.optional {
            errors.push(ValidationError::IfMissingWithoutOptional {
                tweak: tweak.id.clone(),
                effect: effect.id.clone(),
            });
        }
    }
}

/// `ephemeral: true` takes no `undo`/`probe` (spec §7) — schema.rs parses the three fields
/// independently, so nothing upstream stops an author writing both; this is the one place that
/// rejects it.
fn check_ephemeral_has_no_undo_probe(tweak: &Tweak, errors: &mut Vec<ValidationError>) {
    for effect in &tweak.surface {
        if let Effect::Action(ActionDef::Script {
            ephemeral: true,
            undo,
            probe,
            ..
        }) = &effect.kind
        {
            if undo.is_some() || probe.is_some() {
                errors.push(ValidationError::EphemeralWithUndoOrProbe {
                    tweak: tweak.id.clone(),
                    effect: effect.id.clone(),
                });
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// Semantic guards (spec §10, Task 4): per-milestone quantification over each tweak's applicable
// projection — detectability, and three-way distinctness plus the Residue rule.
// -------------------------------------------------------------------------------------------

/// One supported Windows build (spec §10/§14). Milestones are build-only: `revision`/UBR is a
/// finer runtime axis than the build-time guards quantify over (see `scope_admits`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Milestone {
    pub build: u32,
}

/// The declared support matrix (spec §14 default) — the single source of truth every guard loops
/// over: Win10 22H2, then Win11 21H2 / 22H2 / 24H2.
pub const SUPPORT_MATRIX: &[Milestone] = &[
    Milestone { build: 19045 },
    Milestone { build: 22621 },
    Milestone { build: 22631 },
    Milestone { build: 26100 },
];

fn build_expr_contains(expr: BuildExpr, build: u32) -> bool {
    match expr {
        BuildExpr::Exact(n) => build == n,
        BuildExpr::Min(n) => build >= n,
        BuildExpr::Max(n) => build <= n,
        BuildExpr::Range(lo, hi) => (lo..=hi).contains(&build),
    }
}

/// Whether a `windows:` scope (tweak/effect/option-value level, spec §6.6) admits `milestone`.
/// `revision` is intentionally ignored: milestones are build-only, and §6.6 already requires
/// `revision` to pin a single exact `build`, so it adds nothing at this granularity. An invalid
/// `products` entry can't occur here — schema.rs already rejected it at load time via the same
/// `expand_product`; `is_ok_and` just avoids a panic path over an already-loaded corpus.
///
/// `pub(crate)`: `tweaks::snapshot`'s `classify` reuses this verbatim for `TargetUnavailable`
/// (spec §8.3) instead of reimplementing Windows-version admission.
pub(crate) fn scope_admits(scope: Option<&WindowsScope>, milestone: &Milestone) -> bool {
    let Some(scope) = scope else { return true };
    let build_ok = scope
        .build
        .is_none_or(|expr| build_expr_contains(expr, milestone.build));
    let product_ok = scope.products.as_ref().is_none_or(|products| {
        products.iter().any(|&p| {
            expand_product(p).is_ok_and(|expr| build_expr_contains(expr, milestone.build))
        })
    });
    build_ok && product_ok
}

/// One tweak's applicable surface for one milestone (spec §6.6): effects excluded by their own
/// effect-level scope are dropped entirely (the tweak-level scope gates the whole call — an
/// excluded tweak returns empty), and so are ephemeral Actions (spec §7: "exempt from the
/// reversibility and detectability computations" — a transient side-effect leaves no persistent
/// state for detection to observe, so it cannot carry either requirement). Per-option-value
/// scoping is resolved separately per option, since it can differ between options sharing one
/// effect (`applicable_value`).
///
/// `pub(crate)`: `tweaks::engine::detect` reuses this verbatim (spec §8.4) instead of
/// reimplementing version-scoping — detection reads exactly this projection, once per effect.
pub(crate) fn applicable_surface<'a>(
    tweak: &'a Tweak,
    milestone: &Milestone,
) -> Vec<&'a EffectDef> {
    if !scope_admits(tweak.windows.as_ref(), milestone) {
        return Vec::new();
    }
    tweak
        .surface
        .iter()
        .filter(|e| scope_admits(e.windows.as_ref(), milestone) && !is_ephemeral(&e.kind))
        .collect()
}

/// One option's answer for one in-scope effect, on one milestone — `None` if the option's own
/// per-value `windows:` scope (spec §6.6's third scoping level) excludes this milestone, in which
/// case the option simply has no answer for this effect here, as if it did not cover it. Every
/// `OptValue` variant carries that same optional scope (model.rs) — not just `Set`.
///
/// `pub(crate)`: reused verbatim by `tweaks::snapshot::classify` and `tweaks::engine::detect`
/// (spec §8.3/§8.4) — note that for an *omitted* Action entry (legally absent from `opt.values`,
/// spec §6.3), this returns `None` indistinguishably from a scoped-out entry; `engine::detect`
/// disambiguates the two itself where the difference matters (a genuine omission still needs the
/// strict/Residue check, spec §8.4, while a scoped-out entry is skipped like an uncovered effect).
pub(crate) fn applicable_value<'a>(
    opt: &'a Opt,
    effect: &EffectId,
    milestone: &Milestone,
) -> Option<&'a OptValue> {
    let value = opt.values.get(effect)?;
    let windows = match value {
        OptValue::Set(scoped) => scoped.windows.as_ref(),
        OptValue::Run(w) | OptValue::Claim(w) | OptValue::Unclaimed(w) => w.as_ref(),
    };
    if !scope_admits(windows, milestone) {
        return None;
    }
    Some(value)
}

/// Whether `opt` is unavailable as a restore target on `milestone` (spec §8.3/§8.4) — reused by
/// `tweaks::snapshot::classify`'s `TargetUnavailable` instead of a tweak-level-only approximation.
/// Unavailable means `opt` has **no surviving in-surface effect at all** — mirrors
/// `has_detectable_effect`'s any-survives quantification, negated. It is NOT "any covered effect is
/// scoped out": an option that drives several effects and only some of them are scoped out on this
/// milestone is still a perfectly good restore target for the ones that remain (the engine simply
/// skips the inapplicable effect, exactly as `applicable_value`'s own doc says) — only an option
/// left with *nothing* to drive here is actually unavailable. Two ways to end up with nothing: the
/// tweak's own applicable surface is empty (folded in via `applicable_surface`'s tweak-level
/// `windows:` check), or every effect `opt` answers for has its own per-option-value `windows:`
/// scope exclude this exact milestone — a gap narrower than the four `SUPPORT_MATRIX` builds the
/// build-time coverage guard quantifies over, so a running build outside that matrix can hit it
/// even on a corpus that passed every build-time guard.
pub(crate) fn option_unavailable(tweak: &Tweak, opt: &Opt, milestone: &Milestone) -> bool {
    let surface = applicable_surface(tweak, milestone);
    if surface.is_empty() {
        return true;
    }
    !surface
        .iter()
        .any(|effect| applicable_value(opt, &effect.id, milestone).is_some())
}

fn is_ephemeral(kind: &Effect) -> bool {
    matches!(
        kind,
        Effect::Action(ActionDef::Script {
            ephemeral: true,
            ..
        })
    )
}

/// Whether `kind` can ever contribute a detection signal at all (spec §6.4/§8.6): Settings and
/// Shared claims always can; a probe-less Action (including `DeleteTree`, which has no probe
/// field) never can.
fn is_detectable_dimension(kind: &Effect) -> bool {
    match kind {
        Effect::Setting(_) | Effect::Shared(_) => true,
        Effect::Action(ActionDef::Script { probe, .. }) => probe.is_some(),
        Effect::Action(ActionDef::DeleteTree { .. }) => false,
    }
}

/// Whether `kind` is a *reliable* sole distinguisher — one that keeps at most one option matching
/// once the runtime actually reaches that state (spec §8.4/§10). Settings always are. A Shared
/// claim never is (held corpus-wide by any claimant, §8.6 — the dedicated non-shared guard already
/// requires a different differentiator). An Action is iff it carries both `probe` (or there is
/// nothing to read) and `undo` (or a no-undo action's Residue lets the omitting option match too
/// once it has run — the Residue rule, §8.4).
fn is_reliable_dimension(kind: &Effect) -> bool {
    matches!(
        kind,
        Effect::Setting(_)
            | Effect::Action(ActionDef::Script {
                probe: Some(_),
                undo: Some(_),
                ..
            })
    )
}

/// The effects (within `surface`) where options `a` and `b` disagree on `milestone` — the full
/// applicable-projection diff every distinctness check below is defined over (spec §10).
fn differing_effects<'a>(
    a: &Opt,
    b: &Opt,
    surface: &[&'a EffectDef],
    milestone: &Milestone,
) -> Vec<&'a EffectDef> {
    surface
        .iter()
        .filter(|e| applicable_value(a, &e.id, milestone) != applicable_value(b, &e.id, milestone))
        .copied()
        .collect()
}

/// spec §10 Detectability: does `opt` have ≥1 non-optional detectable effect in `surface` on
/// `milestone`? A per-option-value-scoped-out effect (`applicable_value` returning `None`) can't
/// count. A Shared effect additionally needs `opt`'s own answer to be `Claim` — `Unclaimed` is
/// excluded from *this option's* detectable projection (spec §8.6: a claim can be satisfied by any
/// claimant corpus-wide, but "unclaimed" asserts nothing at all). This is evaluated here, with the
/// option's actual `OptValue` in hand, deliberately unlike `is_detectable_dimension` above (which
/// stays kind-only, Shared always counted "detectable" — that domain feeds check 2's pairwise
/// cascade, where a shared-only difference must fall through to check 3's non-shared guard, not be
/// caught here).
fn has_detectable_effect(opt: &Opt, surface: &[&EffectDef], milestone: &Milestone) -> bool {
    surface.iter().any(|effect| {
        if effect.optional {
            return false;
        }
        let Some(value) = applicable_value(opt, &effect.id, milestone) else {
            return false;
        };
        match &effect.kind {
            Effect::Shared(_) => matches!(value, OptValue::Claim(_)),
            kind => is_detectable_dimension(kind),
        }
    })
}

/// Runs every semantic guard (spec §10) over `corpus`, quantified per milestone in `milestones`.
/// A tweak whose applicable surface is empty on a milestone is skipped there, not an error (spec
/// §6.6: it is simply unavailable at runtime). Each violation is reported once — the first
/// milestone it is observed on — never once per failing milestone, so an author fixes the option
/// or pair, not each build separately.
pub fn validate_semantic(corpus: &Corpus, milestones: &[Milestone]) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut reported_detect: HashSet<(String, OptLabel)> = HashSet::new();
    let mut reported_byte: HashSet<(String, OptLabel, OptLabel)> = HashSet::new();
    let mut reported_detectable: HashSet<(String, OptLabel, OptLabel)> = HashSet::new();
    let mut reported_shared: HashSet<(String, OptLabel, OptLabel)> = HashSet::new();
    let mut reported_residue: HashSet<(String, OptLabel, OptLabel)> = HashSet::new();

    for milestone in milestones {
        for tweak in &corpus.tweaks {
            let surface = applicable_surface(tweak, milestone);
            if surface.is_empty() {
                continue; // inapplicable on this milestone — not an error (spec §6.6)
            }

            for opt in &tweak.options {
                if has_detectable_effect(opt, &surface, milestone) {
                    continue;
                }
                let key = (tweak.id.clone(), opt.label.clone());
                if reported_detect.insert(key) {
                    errors.push(ValidationError::NotDetectable {
                        tweak: tweak.id.clone(),
                        option: opt.label.clone(),
                        build: milestone.build,
                    });
                }
            }

            for (i, a) in tweak.options.iter().enumerate() {
                for b in &tweak.options[i + 1..] {
                    let diff = differing_effects(a, b, &surface, milestone);
                    let pair_key = || (tweak.id.clone(), a.label.clone(), b.label.clone());

                    if diff.is_empty() {
                        if reported_byte.insert(pair_key()) {
                            errors.push(ValidationError::OptionsByteIdentical {
                                tweak: tweak.id.clone(),
                                first: a.label.clone(),
                                second: b.label.clone(),
                                build: milestone.build,
                            });
                        }
                    } else if diff.iter().all(|e| !is_detectable_dimension(&e.kind)) {
                        if reported_detectable.insert(pair_key()) {
                            errors.push(ValidationError::OptionsNotDetectablyDistinct {
                                tweak: tweak.id.clone(),
                                first: a.label.clone(),
                                second: b.label.clone(),
                                build: milestone.build,
                            });
                        }
                    } else if diff.iter().all(|e| matches!(e.kind, Effect::Shared(_))) {
                        if reported_shared.insert(pair_key()) {
                            errors.push(ValidationError::SharedOnlyDistinguisher {
                                tweak: tweak.id.clone(),
                                first: a.label.clone(),
                                second: b.label.clone(),
                                build: milestone.build,
                            });
                        }
                    } else if diff.iter().all(|e| !is_reliable_dimension(&e.kind))
                        && reported_residue.insert(pair_key())
                    {
                        errors.push(ValidationError::ResidueOnlyDistinguisher {
                            tweak: tweak.id.clone(),
                            first: a.label.clone(),
                            second: b.label.clone(),
                            build: milestone.build,
                        });
                    }
                }
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::super::schema::load_corpus;
    use super::*;
    use std::path::Path;

    /// Loads one fixture (a single file — see `schema::load_corpus`'s file-vs-dir handling) and
    /// returns every error it produces, whichever phase (load or structural) caught it — a
    /// fixture test should not need to know which.
    fn errors_for(name: &str) -> Vec<ValidationError> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tweaks_fixtures/bad")
            .join(name);
        match load_corpus(&path) {
            Err(errors) => errors,
            Ok(corpus) => validate_structural(&corpus),
        }
    }

    #[test]
    fn dup_address_two_tweaks_is_rejected() {
        let errors = errors_for("dup_address_two_tweaks.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateAddress { first, second, .. } = &errors[0] else {
            panic!("expected DuplicateAddress, got {:?}", errors[0]);
        };
        let owners = format!("{first} {second}");
        assert!(
            owners.contains("tweak_a") && owners.contains("tweak_b"),
            "{owners}"
        );
    }

    #[test]
    fn dup_address_direct_vs_shared_is_rejected() {
        let errors = errors_for("dup_address_direct_vs_shared.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateAddress { first, second, .. } = &errors[0] else {
            panic!("expected DuplicateAddress, got {:?}", errors[0]);
        };
        let owners = format!("{first} {second}");
        assert!(
            owners.contains("holds_it_directly") && owners.contains("telemetry_off"),
            "{owners}"
        );
    }

    #[test]
    fn dup_address_in_one_tweak_is_rejected() {
        let errors = errors_for("dup_address_in_one_tweak.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateAddress { first, second, .. } = &errors[0] else {
            panic!("expected DuplicateAddress, got {:?}", errors[0]);
        };
        let owners = format!("{first} {second}");
        assert!(
            owners.contains("effect_one") && owners.contains("effect_two"),
            "{owners}"
        );
    }

    #[test]
    fn whole_vs_field_mix_is_rejected() {
        let errors = errors_for("whole_vs_field_mix.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateAddress { address, .. } = &errors[0] else {
            panic!("expected DuplicateAddress, got {:?}", errors[0]);
        };
        assert!(address.contains("DirectXUserGlobalSettings"), "{address}");
    }

    /// Review fix: with 3 colliding owners the whole-value branch must report every one of them,
    /// not just the first two (a silently-unreported owner would defeat ADR-0006's guarantee).
    #[test]
    fn dup_address_three_tweaks_reports_every_colliding_owner() {
        let errors = errors_for("dup_address_three_tweaks.yaml");
        assert_eq!(
            errors.len(),
            2,
            "expected one error per additional colliding owner (3 claims => 2 errors), got {errors:?}"
        );
        assert!(
            errors
                .iter()
                .all(|e| matches!(e, ValidationError::DuplicateAddress { .. })),
            "{errors:?}"
        );
        let combined = errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" | ");
        for tweak in ["tweak_a", "tweak_b", "tweak_c"] {
            assert!(combined.contains(tweak), "{combined}");
        }
    }

    #[test]
    fn dup_address_hosts_entry_is_rejected() {
        let errors = errors_for("dup_address_hosts_entry.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateAddress {
            address,
            first,
            second,
        } = &errors[0]
        else {
            panic!("expected DuplicateAddress, got {:?}", errors[0]);
        };
        assert!(address.contains("ads.example.com"), "{address}");
        let owners = format!("{first} {second}");
        assert!(
            owners.contains("block_ads_tweak") && owners.contains("block_ads_tweak_2"),
            "{owners}"
        );
    }

    /// Task 7: confirms the ownership guard treats a firewall rule name as a first-class address
    /// (already wired via `coarse_key_and_field`'s `Setting::Firewall` arm, ahead of `RuleAddr`
    /// being settled) — keyed on the rule name alone, so two effects with the *same name* but
    /// different definitions still collide.
    #[test]
    fn dup_address_firewall_rule_is_rejected() {
        let errors = errors_for("dup_address_firewall_rule.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateAddress {
            address,
            first,
            second,
        } = &errors[0]
        else {
            panic!("expected DuplicateAddress, got {:?}", errors[0]);
        };
        assert!(address.contains("MagicX Duplicate Rule"), "{address}");
        let owners = format!("{first} {second}");
        assert!(
            owners.contains("block_rule_tweak") && owners.contains("block_rule_tweak_2"),
            "{owners}"
        );
    }

    #[test]
    fn dup_shared_decls_is_rejected() {
        let errors = errors_for("dup_shared_decls.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::DuplicateSharedId { id } = &errors[0] else {
            panic!("expected DuplicateSharedId, got {:?}", errors[0]);
        };
        assert_eq!(id.0, "telemetry_off");
    }

    #[test]
    fn services_start_raw_registry_is_rejected() {
        let errors = errors_for("services_start_raw_registry.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::NonCanonicalKind {
            use_kind, effect, ..
        } = &errors[0]
        else {
            panic!("expected NonCanonicalKind, got {:?}", errors[0]);
        };
        assert_eq!(*use_kind, "Service");
        assert_eq!(effect.0, "raw_start_value");
    }

    /// Beyond the named fixture list: the same guard's other named pattern (spec §10), a scheduled
    /// task's registry storage path — one line of the same match arm the fixture already forces.
    #[test]
    fn task_scheduler_raw_registry_is_rejected() {
        assert!(is_task_scheduler_path(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\Foo"
        ));
        assert!(!is_task_scheduler_path(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Other"
        ));
    }

    #[test]
    fn option_missing_setting_is_rejected() {
        let errors = errors_for("option_missing_setting.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::MissingCoverage { effect, .. } = &errors[0] else {
            panic!("expected MissingCoverage, got {:?}", errors[0]);
        };
        assert_eq!(effect.0, "second_effect");
    }

    #[test]
    fn shared_entry_omitted_is_rejected() {
        let errors = errors_for("shared_entry_omitted.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::SharedNotExplicit { effect, .. } = &errors[0] else {
            panic!("expected SharedNotExplicit, got {:?}", errors[0]);
        };
        assert_eq!(effect.0, "telemetry");
    }

    #[test]
    fn bad_path_trailing_backslash_is_rejected() {
        let errors = errors_for("bad_path_trailing_backslash.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::InvalidAddress { source, .. } = &errors[0] else {
            panic!("expected InvalidAddress, got {:?}", errors[0]);
        };
        assert!(
            source.to_string().contains("trailing backslash"),
            "{source}"
        );
    }

    #[test]
    fn null_option_value_is_rejected() {
        let errors = errors_for("null_option_value.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::InvalidOptionValue { reason, .. } = &errors[0] else {
            panic!("expected InvalidOptionValue, got {:?}", errors[0]);
        };
        assert!(reason.contains("absent"), "{reason}");
    }

    #[test]
    fn reversible_flag_lies_is_rejected() {
        let errors = errors_for("reversible_flag_lies.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::ReversibilityMismatch {
            declared, computed, ..
        } = &errors[0]
        else {
            panic!("expected ReversibilityMismatch, got {:?}", errors[0]);
        };
        assert!(*declared && !*computed);
    }

    #[test]
    fn ti_disabled_by_typed_effect_is_rejected() {
        let errors = errors_for("ti_disabled_by_typed_effect.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::TrustedInstallerDisabled { tweak, effect } = &errors[0] else {
            panic!("expected TrustedInstallerDisabled, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "disables_ti");
        assert_eq!(effect.0, "ti_service");
    }

    #[test]
    fn ephemeral_with_undo_is_rejected() {
        let errors = errors_for("ephemeral_with_undo.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::EphemeralWithUndoOrProbe { tweak, effect } = &errors[0] else {
            panic!("expected EphemeralWithUndoOrProbe, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "ephemeral_with_undo_tweak");
        assert_eq!(effect.0, "flush_dns");
    }

    /// Fix 4b: `ephemeral_with_undo_is_rejected` above only exercises the `undo` half of the
    /// guard's `undo.is_some() || probe.is_some()` condition -- this pins the `probe` half too.
    #[test]
    fn ephemeral_with_probe_is_rejected() {
        let errors = errors_for("ephemeral_with_probe.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::EphemeralWithUndoOrProbe { tweak, effect } = &errors[0] else {
            panic!("expected EphemeralWithUndoOrProbe, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "ephemeral_with_probe_tweak");
        assert_eq!(effect.0, "flush_dns");
    }

    #[test]
    fn good_corpus_loads_and_validates_clean() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tweaks_fixtures/good");
        let corpus = load_corpus(&dir).expect("the good fixture corpus must load");
        let errors = validate_structural(&corpus);
        assert!(
            errors.is_empty(),
            "expected no structural errors, got {errors:?}"
        );
    }

    #[test]
    fn if_missing_without_optional_is_rejected() {
        let errors = errors_for("if_missing_without_optional.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::IfMissingWithoutOptional { tweak, effect } = &errors[0] else {
            panic!("expected IfMissingWithoutOptional, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "bad_if_missing_tweak");
        assert_eq!(effect.0, "some_effect");
    }

    // --- Task 4 semantic guards (spec §10) ----------------------------------------------------

    /// Loads one `bad/` fixture and runs the semantic guards over the declared support matrix.
    /// Panics (via `errors_for`'s `Err` arm-equivalent) would hide a structural problem as a
    /// semantic one, so each test also asserts `validate_structural` is clean first.
    fn semantic_errors_for(name: &str) -> Vec<ValidationError> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tweaks_fixtures/bad")
            .join(name);
        let corpus = load_corpus(&path).unwrap_or_else(|e| panic!("{name} must load: {e:?}"));
        assert!(
            validate_structural(&corpus).is_empty(),
            "{name} must be structurally clean so the semantic guard is isolated"
        );
        validate_semantic(&corpus, SUPPORT_MATRIX)
    }

    #[test]
    fn undetectable_on_one_milestone_is_rejected() {
        let errors = semantic_errors_for("undetectable_on_one_milestone.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::NotDetectable {
            tweak,
            option,
            build,
        } = &errors[0]
        else {
            panic!("expected NotDetectable, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "scoped_detector_tweak");
        assert_eq!(option.0, "On");
        assert_eq!(*build, 19045);
    }

    #[test]
    fn all_optional_option_is_rejected() {
        let errors = semantic_errors_for("all_optional_option.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error (deduped across all 4 milestones), got {errors:?}"
        );
        let ValidationError::NotDetectable { tweak, option, .. } = &errors[0] else {
            panic!("expected NotDetectable, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "all_optional_tweak");
        assert_eq!(option.0, "Enabled");
    }

    /// Fix 1 (code review): a single-option tweak whose only effect is a shared reference left
    /// `unclaimed` has zero detectable signal (spec §8.6) — no pairwise guard can catch this,
    /// since there is no second option to compare against.
    #[test]
    fn shared_unclaimed_undetectable_is_rejected() {
        let errors = semantic_errors_for("shared_unclaimed_undetectable.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::NotDetectable { tweak, option, .. } = &errors[0] else {
            panic!("expected NotDetectable, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "shared_unclaimed_tweak");
        assert_eq!(option.0, "Leaves It");
    }

    #[test]
    fn identical_on_detectable_projection_is_rejected() {
        let errors = semantic_errors_for("identical_on_detectable_projection.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::OptionsNotDetectablyDistinct {
            tweak,
            first,
            second,
            ..
        } = &errors[0]
        else {
            panic!("expected OptionsNotDetectablyDistinct, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "silent_action_tweak");
        let labels = format!("{first} {second}");
        assert!(
            labels.contains("Run It") && labels.contains("Skip It"),
            "{labels}"
        );
    }

    #[test]
    fn differ_only_by_noundo_action_is_rejected() {
        let errors = semantic_errors_for("differ_only_by_noundo_action.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::ResidueOnlyDistinguisher {
            tweak,
            first,
            second,
            ..
        } = &errors[0]
        else {
            panic!("expected ResidueOnlyDistinguisher, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "residue_risk_tweak");
        let labels = format!("{first} {second}");
        assert!(
            labels.contains("Enabled") && labels.contains("Disabled"),
            "{labels}"
        );
    }

    #[test]
    fn shared_only_distinguisher_is_rejected() {
        let errors = semantic_errors_for("shared_only_distinguisher.yaml");
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error, got {errors:?}"
        );
        let ValidationError::SharedOnlyDistinguisher {
            tweak,
            first,
            second,
            ..
        } = &errors[0]
        else {
            panic!("expected SharedOnlyDistinguisher, got {:?}", errors[0]);
        };
        assert_eq!(tweak, "shared_only_tweak");
        let labels = format!("{first} {second}");
        assert!(
            labels.contains("Claims It") && labels.contains("Leaves It"),
            "{labels}"
        );
    }

    /// The one fixture that must PASS: an undo-carrying probeable Action is a legal sole
    /// distinguisher (spec §10), unlike its no-undo counterpart above.
    #[test]
    fn differ_only_by_undo_action_ok_validates_clean() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tweaks_fixtures/good/differ_only_by_undo_action_ok.yaml");
        let corpus = load_corpus(&path).expect("fixture must load");
        assert!(validate_structural(&corpus).is_empty());
        let errors = validate_semantic(&corpus, SUPPORT_MATRIX);
        assert!(
            errors.is_empty(),
            "expected no semantic errors, got {errors:?}"
        );
    }

    #[test]
    fn good_corpus_validates_semantically_clean() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tweaks_fixtures/good");
        let corpus = load_corpus(&dir).expect("the good fixture corpus must load");
        let errors = validate_semantic(&corpus, SUPPORT_MATRIX);
        assert!(
            errors.is_empty(),
            "expected no semantic errors, got {errors:?}"
        );
    }

    /// Fix 3 (code review): the empty-applicable-surface skip (spec §6.6) driven end-to-end, not
    /// just at the `scope_admits` primitive — a tweak-level `windows:` scope that excludes most of
    /// the support matrix must load clean, proving a build-specific tweak can ship at all.
    #[test]
    fn build_specific_tweak_is_skipped_not_errored_before_its_windows_scope() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tweaks_fixtures/good");
        let corpus = load_corpus(&dir).expect("the good fixture corpus must load");
        let tweak = corpus
            .tweaks
            .iter()
            .find(|t| t.id == "build_26100_only_tweak")
            .expect("build_26100_only_tweak must load");

        // Sanity: the scope genuinely empties the surface on the earlier milestones and genuinely
        // admits it on 26100 — proves the skip path is actually exercised, not vacuously true.
        assert!(applicable_surface(tweak, &Milestone { build: 19045 }).is_empty());
        assert!(applicable_surface(tweak, &Milestone { build: 22621 }).is_empty());
        assert!(applicable_surface(tweak, &Milestone { build: 22631 }).is_empty());
        assert!(!applicable_surface(tweak, &Milestone { build: 26100 }).is_empty());

        let errors = validate_semantic(&corpus, SUPPORT_MATRIX);
        assert!(
            errors.is_empty(),
            "expected no semantic errors, got {errors:?}"
        );
    }

    #[test]
    fn tweak_scoped_out_on_a_milestone_is_skipped_not_errored() {
        // A tweak entirely out of scope for a milestone (spec §6.6) must not be flagged, even
        // though a single option can never be "distinct" from nothing.
        let milestone = Milestone { build: 19045 };
        assert!(scope_admits(None, &milestone));
        let scope = super::super::model::WindowsScope {
            products: None,
            build: Some(BuildExpr::Min(22000)),
            revision: None,
        };
        assert!(!scope_admits(Some(&scope), &milestone));
        assert!(scope_admits(Some(&scope), &Milestone { build: 22621 }));
    }
}
