//! Build-time structural guards (spec §10) — the STRUCTURAL subset only: ownership/duplicate
//! address, kind canonicalization, option coverage, path/literal validity (via `ValidationError`
//! variants that wrap `parse::ParseError`), reversibility honesty, and TI self-availability over
//! typed effects. Per-milestone quantification, detectability, and distinctness are Task 4's job —
//! they need the declared support matrix, which this task does not touch.
//!
//! No YAML here: `validate_structural` runs over the already-loaded [`Corpus`], so this module
//! compiles into the shipped binary same as `model`/`parse` (only `schema.rs`, the YAML binding
//! layer, is test-only).

use super::model::{
    ActionDef, Corpus, Effect, EffectId, Hive, OptLabel, OptValue, ScopedValue, Setting, SharedId,
    StartupType, Tweak, Value,
};
use super::parse::ParseError;
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
                        Some(OptValue::Claim) | Some(OptValue::Unclaimed)
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
    fn good_corpus_loads_and_validates_clean() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tweaks_fixtures/good");
        let corpus = load_corpus(&dir).expect("the good fixture corpus must load");
        let errors = validate_structural(&corpus);
        assert!(
            errors.is_empty(),
            "expected no structural errors, got {errors:?}"
        );
    }
}
