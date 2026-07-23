//! Detection (spec §8.4, +§5.4/§6.6/§8.6): computes a tweak's live [`TweakStatus`] by reading its
//! applicable, detectable, non-shared surface once, mapping `Missing` through `if_missing`,
//! folding in probeable-Action and shared-claim state, and finding the (at most one) option whose
//! entire detectable projection agrees with what was read. Never mutates anything.
//!
//! **Unknown poisons the whole tweak, never a partial read** (invariant 3). Any effect this pass
//! could not read (or a non-optional effect that reads `Missing`) makes the WHOLE tweak `Unknown` —
//! matching a live surface we could only partially observe would be a guess, so no option-match or
//! System-Default verdict is ever returned alongside an unreadable effect.

use std::collections::{HashMap, HashSet};

use crate::tweaks::kinds::{Error as KindError, ExecCx};
use crate::tweaks::model::{
    ActionDef, Corpus, Effect, EffectDef, EffectId, Opt, OptLabel, OptValue, SharedId, Tweak, Value,
};
use crate::tweaks::shared_claims::ClaimsStore;
use crate::tweaks::validate::{
    applicable_surface, applicable_value, option_unavailable, scope_admits, Milestone,
};

use super::Deps;

/// Why one applicable effect could not be read (spec §8.4, invariant 3). Every variant traces back
/// to a real `Err` (or a non-optional `Missing` read) — never fabricated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownCause {
    /// The read/probe failed with [`KindError::AccessDenied`] — elevating may fix it.
    AccessDenied,
    /// A stored type didn't match its declared shape, or a packed value could not be parsed.
    Malformed,
    /// A non-optional effect read `Value::Missing` (spec §5.4) — typed error, not a guess.
    MissingRequired,
    /// Any other read/probe failure — never implies elevation would help (controller decision 5:
    /// under-reporting `needs_elevation` is safe, over-reporting is a defect). Also covers the
    /// (build-guard-prevented, invariant 9) case of more than one option matching at once: logged
    /// loudly by `detect` and surfaced here rather than silently picking a winner.
    Other,
}

/// One applicable effect this pass could not read (spec §8.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownReason {
    pub effect: EffectId,
    pub cause: UnknownCause,
    /// `true` only when `cause` is `AccessDenied` — never guessed for any other cause.
    pub needs_elevation: bool,
}

/// An option that cannot be selected on this machine/build right now (spec §5.4/§6.6): either its
/// own applicable surface is empty here (`option_unavailable`), or it authors a real value for an
/// effect whose live resource actually reads `Missing` — the engine never installs/uninstalls, so
/// applying it is impossible, not merely "not currently active".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnavailableOpt {
    pub label: OptLabel,
    pub reason: String,
}

/// A corpus-level shared setting's current claimants, surfaced as info regardless of match state
/// (spec §8.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldInfo {
    pub shared: SharedId,
    pub holders: Vec<String>,
}

/// A tweak's detected state (spec §8.4/§6.1, ADR-0003).
///
/// `Unavailable` is a controller-decided 4th variant beyond the brief's literal three
/// (`Active`/`SystemDefault`/`Unknown`): spec §6.6 requires "a tweak whose applicable surface is
/// empty on the running build is shown unavailable, with the reason" — distinct from `Unknown`
/// (surface exists but is unreadable) and from `SystemDefault` (surface exists and reads as none of
/// the authored options). Folding an out-of-scope tweak into either would blur a real distinction
/// the frontend needs to render (there is nothing to detect at all, vs. something we tried and
/// failed to read, vs. something we read cleanly that just isn't any authored option).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakState {
    Active(OptLabel),
    SystemDefault,
    Unavailable(String),
    Unknown(Vec<UnknownReason>),
}

/// A tweak's live status (spec §8.4/§8.6). `residues`/`held_shared` are informational disclosures,
/// never a failure; `has_history` means only "a snapshot exists to restore from" (detection is
/// decoupled from history, spec §8.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakStatus {
    pub state: TweakState,
    pub unavailable: Vec<UnavailableOpt>,
    pub residues: Vec<EffectId>,
    pub has_history: bool,
    pub held_shared: Vec<HeldInfo>,
}

/// One applicable Setting's live reading — already `if_missing`-mapped when it originated from an
/// optional `Missing` resource. `missing_origin` (tracked alongside) is what lets the
/// unavailable-option check distinguish "reads Missing, mapped" from an ordinary live value that
/// merely happens to share the mapped value.
type Readings = HashMap<EffectId, Value>;

/// Computes `tweak`'s live [`TweakStatus`] against `deps` (spec §8.4). `corpus` is needed only for
/// the snapshot-history lookup (`SnapshotStore::head`'s own signature); shared-claim matching goes
/// through `deps.claims` directly, never the corpus's `shared:` values (spec §8.6: claim state, not
/// the underlying live value, is what detection compares).
pub fn detect(tweak: &Tweak, corpus: &Corpus, deps: &Deps) -> TweakStatus {
    let milestone = deps.running;
    let surface = applicable_surface(tweak, &milestone);

    if surface.is_empty() {
        return TweakStatus {
            state: TweakState::Unavailable(format!(
                "'{}' has no applicable effects on this Windows build",
                tweak.name
            )),
            unavailable: Vec::new(),
            residues: Vec::new(),
            has_history: has_history(tweak, corpus, deps),
            held_shared: Vec::new(),
        };
    }

    let cx = ExecCx::new(deps.level);
    let mut readings: Readings = HashMap::new();
    let mut missing_origin: HashSet<EffectId> = HashSet::new();
    let mut probe_present: HashMap<EffectId, bool> = HashMap::new();
    let mut unknown = Vec::new();
    let mut held_shared = Vec::new();

    // Read each applicable, detectable, non-shared effect exactly once (spec §8.4) — shared across
    // every option's comparison below, never re-read per option.
    for effect in &surface {
        match &effect.kind {
            Effect::Setting(setting) => {
                classify_setting_read(
                    deps.kinds.read(setting, &cx),
                    effect,
                    &mut readings,
                    &mut missing_origin,
                    &mut unknown,
                );
            }
            Effect::Shared(shared_id) => {
                if deps.claims.is_claimed(shared_id) {
                    held_shared.push(HeldInfo {
                        shared: shared_id.clone(),
                        holders: deps.claims.holders(shared_id),
                    });
                }
            }
            Effect::Action(action_def) => {
                if !contributes_to_detection(action_def) {
                    continue; // probe-less action -- never read, never compared (spec §6.4)
                }
                match probe_cached(deps, &tweak.id, &effect.id, action_def, &cx) {
                    Ok(present) => {
                        probe_present.insert(effect.id.clone(), present);
                    }
                    Err(KindError::AccessDenied(_)) => unknown.push(UnknownReason {
                        effect: effect.id.clone(),
                        cause: UnknownCause::AccessDenied,
                        needs_elevation: true,
                    }),
                    Err(e) => {
                        log::warn!("detect '{}': probe '{}' failed: {e}", tweak.id, effect.id);
                        unknown.push(UnknownReason {
                            effect: effect.id.clone(),
                            cause: UnknownCause::Other,
                            needs_elevation: false,
                        });
                    }
                }
            }
        }
    }

    if !unknown.is_empty() {
        return TweakStatus {
            state: TweakState::Unknown(unknown),
            unavailable: Vec::new(),
            residues: Vec::new(),
            has_history: has_history(tweak, corpus, deps),
            held_shared,
        };
    }

    let mut unavailable = Vec::new();
    let mut matched: Vec<(&Opt, Vec<EffectId>)> = Vec::new();

    for opt in &tweak.options {
        if option_unavailable(tweak, opt, &milestone) {
            unavailable.push(UnavailableOpt {
                label: opt.label.clone(),
                reason: "not applicable on this Windows build".to_string(),
            });
            continue;
        }
        if let Some(effect_id) =
            missing_mismatch(opt, &surface, &milestone, &missing_origin, &readings)
        {
            unavailable.push(UnavailableOpt {
                label: opt.label.clone(),
                reason: format!(
                    "requires effect '{effect_id}' to have a real value, but its resource is not present on this machine"
                ),
            });
            continue;
        }
        if let Some(residues) = option_matches(
            opt,
            &surface,
            &milestone,
            &readings,
            &probe_present,
            deps.claims,
        ) {
            matched.push((opt, residues));
        }
    }

    let has_history = has_history(tweak, corpus, deps);
    match matched.len() {
        0 => TweakStatus {
            state: TweakState::SystemDefault,
            unavailable,
            residues: Vec::new(),
            has_history,
            held_shared,
        },
        1 => {
            let (opt, residues) = matched.into_iter().next().expect("checked len == 1");
            TweakStatus {
                state: TweakState::Active(opt.label.clone()),
                unavailable,
                residues,
                has_history,
                held_shared,
            }
        }
        n => {
            // Guaranteed unreachable by the build-time distinctness guard (spec §10, invariant 9) —
            // surfaced loudly rather than silently picking a winner (controller decision 3).
            let labels: Vec<_> = matched.iter().map(|(o, _)| o.label.to_string()).collect();
            log::error!(
                "detect '{}': {n} options matched simultaneously ({labels:?}) -- distinctness guard violated",
                tweak.id
            );
            TweakStatus {
                state: TweakState::Unknown(vec![UnknownReason {
                    effect: surface[0].id.clone(),
                    cause: UnknownCause::Other,
                    needs_elevation: false,
                }]),
                unavailable,
                residues: Vec::new(),
                has_history,
                held_shared,
            }
        }
    }
}

/// Classifies one Setting read into a live `Readings` entry or an `UnknownReason` (spec §5.4/§8.4).
fn classify_setting_read(
    result: Result<Value, KindError>,
    effect: &EffectDef,
    readings: &mut Readings,
    missing_origin: &mut HashSet<EffectId>,
    unknown: &mut Vec<UnknownReason>,
) {
    match result {
        Ok(Value::Missing) if effect.optional => {
            // "detection treats this effect as reading <if_missing>" (spec §5.4); undeclared
            // if_missing maps to the literal `Missing` value itself -- since no option can ever
            // author `Missing` (spec §5.4), every option needing a real value here naturally falls
            // into the unavailable path below, with no special-casing required.
            let mapped = effect.if_missing.clone().unwrap_or(Value::Missing);
            missing_origin.insert(effect.id.clone());
            readings.insert(effect.id.clone(), mapped);
        }
        Ok(Value::Missing) => unknown.push(UnknownReason {
            effect: effect.id.clone(),
            cause: UnknownCause::MissingRequired,
            needs_elevation: false,
        }),
        Ok(v) => {
            readings.insert(effect.id.clone(), v);
        }
        Err(KindError::AccessDenied(_)) => unknown.push(UnknownReason {
            effect: effect.id.clone(),
            cause: UnknownCause::AccessDenied,
            needs_elevation: true,
        }),
        Err(KindError::TypeMismatch { .. } | KindError::MalformedPacked { .. }) => {
            unknown.push(UnknownReason {
                effect: effect.id.clone(),
                cause: UnknownCause::Malformed,
                needs_elevation: false,
            });
        }
        Err(e) => {
            log::warn!("detect: effect '{}' unreadable: {e}", effect.id);
            unknown.push(UnknownReason {
                effect: effect.id.clone(),
                cause: UnknownCause::Other,
                needs_elevation: false,
            });
        }
    }
}

/// Whether `action` can ever contribute a detection signal (spec §6.4): a probe-less Action
/// (`Script` with no `probe`, or `DeleteTree`, which has no `probe` field at all) never can. Local
/// equivalent of `validate::is_detectable_dimension` (private there, and this task's boundary
/// excludes touching `validate.rs` beyond the two reused helpers) — kept in sync by hand.
fn contributes_to_detection(action: &ActionDef) -> bool {
    matches!(action, ActionDef::Script { probe: Some(_), .. })
}

/// Whether `action` reverts cleanly (spec §7) — decides the omitted-probe expectation's
/// strict-vs-Residue split (spec §8.4/§10). Local equivalent of a slice of `validate.rs`'s
/// reversibility check, scoped to one action.
fn has_undo(action: &ActionDef) -> bool {
    match action {
        ActionDef::Script { undo, .. } | ActionDef::DeleteTree { undo, .. } => undo.is_some(),
    }
}

/// Runs (or reads the cached) probe for `effect_id` on `tweak_id` — populates the cache on miss,
/// reads it on hit (spec §7: session-cached, never re-spawned per status poll).
fn probe_cached(
    deps: &Deps,
    tweak_id: &str,
    effect_id: &EffectId,
    action: &ActionDef,
    cx: &ExecCx,
) -> Result<bool, KindError> {
    if let Some(cached) = deps.probe_cache.get(tweak_id, effect_id) {
        return Ok(cached);
    }
    let present = deps.probes.probe(action, cx)?;
    deps.probe_cache.insert(tweak_id, effect_id, present);
    Ok(present)
}

/// `Some(effect_id)` when `opt` authors a real value for an effect whose live resource actually
/// reads `Missing` (`missing_origin`) and that value doesn't equal the `if_missing`-mapped reading
/// — the machine cannot satisfy it (spec §5.4: the engine never installs the resource), so `opt` is
/// unavailable rather than merely non-matching. `None` when every missing-origin effect `opt`
/// answers for already agrees, including the "opt doesn't cover this effect here" case (spec §6.6's
/// per-value scoping).
fn missing_mismatch(
    opt: &Opt,
    surface: &[&EffectDef],
    milestone: &Milestone,
    missing_origin: &HashSet<EffectId>,
    readings: &Readings,
) -> Option<EffectId> {
    for effect in surface {
        if !missing_origin.contains(&effect.id) {
            continue;
        }
        let Some(OptValue::Set(scoped)) = applicable_value(opt, &effect.id, milestone) else {
            continue;
        };
        let mapped = readings
            .get(&effect.id)
            .expect("missing_origin implies a reading was recorded for this effect");
        if &scoped.value != mapped {
            return Some(effect.id.clone());
        }
    }
    None
}

/// Whether `opt` matches the live surface (spec §8.4): every applicable, detectable effect it
/// covers must agree with what was read/probed/claimed. `Some(residues)` on a match (the no-undo
/// probeable Actions whose probe read present despite `opt` omitting them — spec §8.4's Residue
/// tolerance); `None` on no match.
fn option_matches(
    opt: &Opt,
    surface: &[&EffectDef],
    milestone: &Milestone,
    readings: &Readings,
    probe_present: &HashMap<EffectId, bool>,
    claims: &ClaimsStore,
) -> Option<Vec<EffectId>> {
    let mut residues = Vec::new();
    for effect in surface {
        match &effect.kind {
            Effect::Setting(_) => {
                // Settings are always covered (build-guarded, spec §6.3): `applicable_value`'s
                // `None` here can only mean this option-value's own scope excludes this milestone.
                let Some(opt_value) = applicable_value(opt, &effect.id, milestone) else {
                    continue;
                };
                let OptValue::Set(scoped) = opt_value else {
                    return None; // shape mismatch -- build-guard-prevented; treat as non-match
                };
                let live = readings
                    .get(&effect.id)
                    .expect("every applicable Setting was read once above");
                if &scoped.value != live {
                    return None;
                }
            }
            Effect::Shared(shared_id) => {
                let Some(opt_value) = applicable_value(opt, &effect.id, milestone) else {
                    continue;
                };
                let claimed = claims.is_claimed(shared_id);
                match opt_value {
                    OptValue::Claim(_) if claimed => {}
                    OptValue::Unclaimed(_) if !claimed => {}
                    _ => return None,
                }
            }
            Effect::Action(action_def) => {
                if !contributes_to_detection(action_def) {
                    continue;
                }
                // `applicable_value` cannot distinguish "genuinely omitted" (legally absent from
                // `opt.values`, spec §6.3) from "scoped out" for an Action -- both read back `None`.
                // Only a *scoped-out* authored `run` is skipped like an uncovered effect; a genuine
                // omission still needs the strict/Residue check below (spec §8.4).
                let raw = opt.values.get(&effect.id);
                let scoped_out =
                    matches!(raw, Some(OptValue::Run(w)) if !scope_admits(w.as_ref(), milestone));
                if scoped_out {
                    continue;
                }
                let runs = matches!(raw, Some(OptValue::Run(_)));
                let present = *probe_present
                    .get(&effect.id)
                    .expect("every detectable action was probed once above");
                if runs {
                    if !present {
                        return None; // an option that runs the action expects it present
                    }
                } else if present {
                    if has_undo(action_def) {
                        return None; // strict: an undo-carrying action's presence disqualifies
                    }
                    residues.push(effect.id.clone()); // no-undo Residue -- tolerated (spec §8.4)
                }
            }
        }
    }
    Some(residues)
}

/// "A history exists to restore from" (spec §8.4) — decoupled from the match result above. A
/// snapshot I/O error is logged and treated as "no history": this is a UI hint, not a correctness-
/// critical reading, so it must never poison the tweak's detected state.
fn has_history(tweak: &Tweak, corpus: &Corpus, deps: &Deps) -> bool {
    match deps
        .snapshots
        .head(&tweak.id, corpus, deps.machine_guid, deps.running.build)
    {
        Ok(entry) => entry.is_some(),
        Err(e) => {
            log::warn!("detect '{}': snapshot history unreadable: {e}", tweak.id);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::{ProbeCache, RealActions};
    use crate::tweaks::kinds::EffectKind;
    use crate::tweaks::model::{
        BuildExpr, Effect, EffectDef, Hive, Level, Opt, OptLabel, RegAddr, RegType, RiskLevel,
        ScopedValue, Script, Setting, SharedDef, Shell, StartupType, SvcAddr, Tweak, TypedRegValue,
        Value, WindowsScope,
    };
    use crate::tweaks::snapshot::SnapshotStore;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU32, Ordering};

    // --- in-memory mocks (zero OS contact) ----------------------------------------------------

    #[derive(Clone)]
    enum MockRead {
        Ok(Value),
        AccessDenied,
        TypeMismatch,
    }

    #[derive(Default)]
    struct MockKind {
        settings: Vec<(Setting, MockRead)>,
    }

    impl MockKind {
        fn with(mut self, setting: Setting, outcome: MockRead) -> Self {
            self.settings.push((setting, outcome));
            self
        }
    }

    impl EffectKind for MockKind {
        fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, KindError> {
            let (_, outcome) = self
                .settings
                .iter()
                .find(|(k, _)| k == s)
                .unwrap_or_else(|| {
                    panic!("MockKind: unexpected read for {s:?} -- scoped-out/unregistered effect must never be read")
                });
            match outcome {
                MockRead::Ok(v) => Ok(v.clone()),
                MockRead::AccessDenied => Err(KindError::AccessDenied("mock access denied".into())),
                MockRead::TypeMismatch => Err(KindError::TypeMismatch {
                    path: "mock".into(),
                    name: "mock".into(),
                    expected: RegType::Dword,
                    actual: RegType::Sz,
                }),
            }
        }

        fn drive(&self, _s: &Setting, _target: &Value, _cx: &ExecCx) -> Result<(), KindError> {
            unimplemented!("detect never drives")
        }
    }

    #[derive(Clone, Copy)]
    enum MockProbeOutcome {
        Present,
        Absent,
    }

    #[derive(Default)]
    struct MockProbes {
        outcomes: Vec<(ActionDef, MockProbeOutcome)>,
        calls: AtomicU32,
    }

    impl MockProbes {
        fn with(mut self, action: ActionDef, outcome: MockProbeOutcome) -> Self {
            self.outcomes.push((action, outcome));
            self
        }
    }

    impl super::super::ProbeSource for MockProbes {
        fn probe(&self, action: &ActionDef, _cx: &ExecCx) -> Result<bool, KindError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (_, outcome) = self
                .outcomes
                .iter()
                .find(|(a, _)| a == action)
                .unwrap_or_else(|| panic!("MockProbes: unexpected probe for {action:?}"));
            match outcome {
                MockProbeOutcome::Present => Ok(true),
                MockProbeOutcome::Absent => Ok(false),
            }
        }
    }

    // --- fixture builders ----------------------------------------------------------------------

    fn svc_effect(id: &str, optional: bool, if_missing: Option<Value>) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Setting(Setting::Service(SvcAddr {
                name: id.to_string(),
            })),
            elevation: None,
            optional,
            if_missing,
            windows: None,
        }
    }

    fn shared_effect(id: &str, shared: &str) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Shared(SharedId(shared.to_string())),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn action_effect(id: &str, undo: bool) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Action(ActionDef::Script {
                apply: Script("exit 0".into()),
                undo: undo.then(|| Script("exit 0".into())),
                probe: Some(Script("exit 0".into())),
                ephemeral: false,
                shell: Shell::PowerShell,
            }),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn set(value: Value) -> OptValue {
        OptValue::Set(ScopedValue {
            value,
            windows: None,
        })
    }

    fn opt(label: &str, values: Vec<(&str, OptValue)>) -> Opt {
        let mut map = BTreeMap::new();
        for (id, v) in values {
            map.insert(EffectId(id.to_string()), v);
        }
        Opt {
            label: OptLabel(label.to_string()),
            values: map,
        }
    }

    fn tweak(id: &str, surface: Vec<EffectDef>, options: Vec<Opt>) -> Tweak {
        tweak_scoped(id, surface, options, None)
    }

    fn tweak_scoped(
        id: &str,
        surface: Vec<EffectDef>,
        options: Vec<Opt>,
        windows: Option<WindowsScope>,
    ) -> Tweak {
        Tweak {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            category: "misc".to_string(),
            info: None,
            warning: None,
            requires_reboot: false,
            risk_level: RiskLevel::Low,
            elevation: Level::User,
            reversible: true,
            surface,
            options,
            windows,
        }
    }

    fn corpus(tweaks: Vec<Tweak>) -> Corpus {
        Corpus {
            categories: Vec::new(),
            tweaks,
            shared: Vec::new(),
        }
    }

    /// A harness bundling the owned stores/mocks a test needs, so `Deps` (all borrows) can outlive
    /// the call to `detect` without lifetime gymnastics at each call site.
    struct Harness {
        kind: MockKind,
        probes: MockProbes,
        claims: ClaimsStore,
        snapshots: SnapshotStore,
        cache: ProbeCache,
        _tmp: tempfile::TempDir,
    }

    impl Harness {
        fn new(kind: MockKind, probes: MockProbes) -> Self {
            let tmp = tempfile::tempdir().unwrap();
            Self {
                kind,
                probes,
                claims: ClaimsStore::open(tmp.path().to_path_buf(), None),
                snapshots: SnapshotStore::open(tmp.path().to_path_buf()),
                cache: ProbeCache::new(),
                _tmp: tmp,
            }
        }

        fn deps(&self) -> Deps<'_> {
            Deps {
                kinds: &self.kind,
                probes: &self.probes,
                // `detect` never runs an Action, only probes it (spec §8.4) -- the real,
                // stateless dispatcher is safe to wire here since nothing in this module ever
                // calls it.
                actions: &RealActions,
                claims: &self.claims,
                snapshots: &self.snapshots,
                probe_cache: &self.cache,
                machine_guid: None,
                level: Level::User,
                running: Milestone { build: 19045 },
            }
        }
    }

    // --- the 14 scenarios ----------------------------------------------------------------------

    #[test]
    fn matching_option_wins() {
        let surface = vec![svc_effect("svc", false, None)];
        let options = vec![
            opt("A", vec![("svc", set(Value::Startup(StartupType::Manual)))]),
            opt(
                "B",
                vec![("svc", set(Value::Startup(StartupType::Disabled)))],
            ),
        ];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::Ok(Value::Startup(StartupType::Disabled)),
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert_eq!(status.state, TweakState::Active(OptLabel("B".into())));
    }

    #[test]
    fn at_most_one_match_holds() {
        // Guard-legal corpus: every option authors a distinct value on the one effect. Every live
        // state in the support matrix is enumerated, and exactly its own option must match.
        let states = [
            StartupType::Boot,
            StartupType::Manual,
            StartupType::Disabled,
        ];
        let surface = vec![svc_effect("svc", false, None)];
        let options: Vec<Opt> = states
            .iter()
            .enumerate()
            .map(|(i, s)| opt(&format!("Opt{i}"), vec![("svc", set(Value::Startup(*s)))]))
            .collect();
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        for (i, live) in states.iter().enumerate() {
            let kind = MockKind::default().with(
                Setting::Service(SvcAddr { name: "svc".into() }),
                MockRead::Ok(Value::Startup(*live)),
            );
            let h = Harness::new(kind, MockProbes::default());
            let status = detect(&t, &c, &h.deps());
            assert_eq!(
                status.state,
                TweakState::Active(OptLabel(format!("Opt{i}"))),
                "live state {live:?} must match exactly Opt{i}, never another option"
            );
        }
    }

    #[test]
    fn no_match_reads_system_default() {
        let surface = vec![svc_effect("svc", false, None)];
        let options = vec![
            opt("A", vec![("svc", set(Value::Startup(StartupType::Manual)))]),
            opt(
                "B",
                vec![("svc", set(Value::Startup(StartupType::Disabled)))],
            ),
        ];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::Ok(Value::Startup(StartupType::Automatic)), // authored by neither option
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert_eq!(status.state, TweakState::SystemDefault);
    }

    #[test]
    fn optional_missing_maps_if_missing() {
        let surface = vec![svc_effect(
            "svc",
            true,
            Some(Value::Startup(StartupType::Disabled)),
        )];
        let options = vec![
            opt(
                "Disabled",
                vec![("svc", set(Value::Startup(StartupType::Disabled)))],
            ),
            opt(
                "Enabled",
                vec![("svc", set(Value::Startup(StartupType::Manual)))],
            ),
        ];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::Ok(Value::Missing),
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert_eq!(
            status.state,
            TweakState::Active(OptLabel("Disabled".into())),
            "a missing optional service must map through if_missing and match the Disabled option"
        );
    }

    #[test]
    fn option_needing_missing_resource_is_unavailable() {
        let surface = vec![svc_effect(
            "svc",
            true,
            Some(Value::Startup(StartupType::Disabled)),
        )];
        let options = vec![
            opt(
                "Disabled",
                vec![("svc", set(Value::Startup(StartupType::Disabled)))],
            ),
            opt(
                "Enabled",
                vec![("svc", set(Value::Startup(StartupType::Manual)))],
            ),
        ];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::Ok(Value::Missing),
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert_eq!(
            status.state,
            TweakState::Active(OptLabel("Disabled".into()))
        );
        assert_eq!(
            status.unavailable,
            vec![UnavailableOpt {
                label: OptLabel("Enabled".into()),
                reason: "requires effect 'svc' to have a real value, but its resource is not present on this machine".into(),
            }],
            "Enabled wants a real (non-if_missing) value on a Missing resource -- unavailable, not a plain non-match"
        );
    }

    #[test]
    fn nonoptional_missing_is_unknown() {
        let surface = vec![svc_effect("svc", false, None)];
        let options = vec![opt(
            "Disabled",
            vec![("svc", set(Value::Startup(StartupType::Disabled)))],
        )];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::Ok(Value::Missing),
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        let TweakState::Unknown(reasons) = status.state else {
            panic!("expected Unknown, got {:?}", status.state);
        };
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].effect, EffectId("svc".into()));
        assert_eq!(reasons[0].cause, UnknownCause::MissingRequired);
        assert!(!reasons[0].needs_elevation);
    }

    #[test]
    fn access_denied_is_unknown_never_sd() {
        let surface = vec![svc_effect("svc", false, None)];
        let options = vec![opt(
            "Disabled",
            vec![("svc", set(Value::Startup(StartupType::Disabled)))],
        )];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::AccessDenied,
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert_ne!(
            status.state,
            TweakState::SystemDefault,
            "an unreadable effect must never present as System Default (invariant 3)"
        );
        let TweakState::Unknown(reasons) = status.state else {
            panic!("expected Unknown, got {:?}", status.state);
        };
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].cause, UnknownCause::AccessDenied);
        assert!(reasons[0].needs_elevation);
    }

    #[test]
    fn malformed_packed_is_unknown() {
        let surface = vec![svc_effect("svc", false, None)];
        let options = vec![opt(
            "Disabled",
            vec![("svc", set(Value::Startup(StartupType::Disabled)))],
        )];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let kind = MockKind::default().with(
            Setting::Service(SvcAddr { name: "svc".into() }),
            MockRead::TypeMismatch,
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        let TweakState::Unknown(reasons) = status.state else {
            panic!("expected Unknown, got {:?}", status.state);
        };
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].cause, UnknownCause::Malformed);
        assert!(!reasons[0].needs_elevation);
    }

    /// Every Setting effect below is a fixed anchor both options agree on (`Manual`, never
    /// touched) purely so `option_unavailable` sees each option answering *something* real --
    /// isolating the Action-only strict/Residue behavior under test without accidentally hitting
    /// the (correct, and separately guarded at build time) "an option that answers for nothing at
    /// all is unavailable" rule these tests are not about.
    fn anchor_svc() -> EffectDef {
        svc_effect("anchor", false, None)
    }
    fn anchor_value() -> (&'static str, OptValue) {
        ("anchor", set(Value::Startup(StartupType::Manual)))
    }
    fn anchor_kind() -> MockKind {
        MockKind::default().with(
            Setting::Service(SvcAddr {
                name: "anchor".into(),
            }),
            MockRead::Ok(Value::Startup(StartupType::Manual)),
        )
    }

    #[test]
    fn omitted_undo_action_expectation_strict() {
        let effect = action_effect("act", true); // undo-carrying
        let Effect::Action(action_def) = effect.kind.clone() else {
            unreachable!()
        };
        let surface = vec![anchor_svc(), effect];
        let options = vec![
            opt("Run", vec![anchor_value(), ("act", OptValue::Run(None))]),
            opt("Skip", vec![anchor_value()]), // genuinely omits the action
        ];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let probes = MockProbes::default().with(action_def, MockProbeOutcome::Present);
        let h = Harness::new(anchor_kind(), probes);

        let status = detect(&t, &c, &h.deps());
        assert_eq!(
            status.state,
            TweakState::Active(OptLabel("Run".into())),
            "Skip must be disqualified (undo-carrying + probe present), leaving only Run to match"
        );
        assert!(status.residues.is_empty());

        // In isolation: with only "Skip" on the surface, a present probe must disqualify it too,
        // falling through to System Default rather than a spurious match.
        let skip_only = tweak(
            "demo_skip_only",
            vec![anchor_svc(), action_effect("act", true)],
            vec![opt("Skip", vec![anchor_value()])],
        );
        let c2 = corpus(vec![skip_only.clone()]);
        let probes2 = MockProbes::default().with(
            match skip_only.surface[1].kind.clone() {
                Effect::Action(a) => a,
                _ => unreachable!(),
            },
            MockProbeOutcome::Present,
        );
        let h2 = Harness::new(anchor_kind(), probes2);
        let status2 = detect(&skip_only, &c2, &h2.deps());
        assert_eq!(status2.state, TweakState::SystemDefault);

        // Symmetric sanity: with the probe reading absent, "Run" (which expects present) must be
        // disqualified instead, leaving "Skip" (which expects absent, and got it) to match.
        let Effect::Action(action_def_absent) = action_effect("act", true).kind else {
            unreachable!()
        };
        let probes3 = MockProbes::default().with(action_def_absent, MockProbeOutcome::Absent);
        let h3 = Harness::new(anchor_kind(), probes3);
        let status3 = detect(&t, &c, &h3.deps());
        assert_eq!(status3.state, TweakState::Active(OptLabel("Skip".into())));
    }

    #[test]
    fn omitted_noundo_action_residue_tolerated() {
        let effect = action_effect("act", false); // no undo -- one-way
        let Effect::Action(action_def) = effect.kind.clone() else {
            unreachable!()
        };
        let surface = vec![anchor_svc(), effect];
        let options = vec![opt("Skip", vec![anchor_value()])]; // genuinely omits the action
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let probes = MockProbes::default().with(action_def, MockProbeOutcome::Present);
        let h = Harness::new(anchor_kind(), probes);

        let status = detect(&t, &c, &h.deps());
        assert_eq!(
            status.state,
            TweakState::Active(OptLabel("Skip".into())),
            "a no-undo action's lingering presence must not disqualify the omitting option"
        );
        assert_eq!(status.residues, vec![EffectId("act".into())]);
    }

    #[test]
    fn claimed_shared_matches_all_claimants() {
        let surface = vec![shared_effect("telemetry", "telemetry_off")];
        let options = vec![
            opt("Claims", vec![("telemetry", OptValue::Claim(None))]),
            opt("Leaves", vec![("telemetry", OptValue::Unclaimed(None))]),
        ];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let h = Harness::new(MockKind::default(), MockProbes::default());

        // Seed a claim held by some other tweak, via the real ClaimsStore (spec §8.6).
        let shared_def = SharedDef {
            id: SharedId("telemetry_off".into()),
            setting: Setting::Registry(RegAddr {
                hive: Hive::Hklm,
                path: "SOFTWARE\\Test".into(),
                name: "V".into(),
                ty: RegType::Dword,
                field: None,
            }),
            value: Value::Reg(TypedRegValue::Dword(0)),
        };
        // `ClaimsStore::claim`/`release` both read AND drive (spec §8.6) -- unlike `MockKind`
        // (a fixed read-only fixture for `detect` itself, which never drives), this needs a
        // stateful mock so the post-drive read-back verification succeeds.
        struct StatefulKind(std::sync::Mutex<Value>);
        impl EffectKind for StatefulKind {
            fn read(&self, _s: &Setting, _cx: &ExecCx) -> Result<Value, KindError> {
                Ok(self.0.lock().unwrap().clone())
            }
            fn drive(&self, _s: &Setting, target: &Value, _cx: &ExecCx) -> Result<(), KindError> {
                *self.0.lock().unwrap() = target.clone();
                Ok(())
            }
        }
        let claim_kind = StatefulKind(std::sync::Mutex::new(Value::Reg(TypedRegValue::Dword(1))));
        h.claims
            .claim(
                &shared_def,
                "other_tweak",
                &claim_kind,
                &ExecCx::new(Level::User),
            )
            .unwrap();

        let status = detect(&t, &c, &h.deps());
        assert_eq!(status.state, TweakState::Active(OptLabel("Claims".into())));
        assert_eq!(
            status.held_shared,
            vec![HeldInfo {
                shared: SharedId("telemetry_off".into()),
                holders: vec!["other_tweak".to_string()],
            }]
        );

        // Release the claim: now the Unclaimed option must match instead.
        h.claims
            .release(
                &shared_def.id,
                "other_tweak",
                &claim_kind,
                &ExecCx::new(Level::User),
            )
            .unwrap();
        let status2 = detect(&t, &c, &h.deps());
        assert_eq!(status2.state, TweakState::Active(OptLabel("Leaves".into())));
        assert!(status2.held_shared.is_empty());
    }

    #[test]
    fn scoped_out_effect_excluded() {
        let always = svc_effect("always", false, None);
        let mut future_only = svc_effect("future_only", false, None);
        future_only.windows = Some(WindowsScope {
            products: None,
            build: Some(BuildExpr::Min(26100)), // excludes the running milestone (19045)
            revision: None,
        });
        let surface = vec![always, future_only];
        let options = vec![opt(
            "On",
            vec![
                ("always", set(Value::Startup(StartupType::Manual))),
                ("future_only", set(Value::Startup(StartupType::Manual))),
            ],
        )];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        // Deliberately do NOT register "future_only" -- if detect ever reads it, MockKind panics.
        let kind = MockKind::default().with(
            Setting::Service(SvcAddr {
                name: "always".into(),
            }),
            MockRead::Ok(Value::Startup(StartupType::Manual)),
        );
        let h = Harness::new(kind, MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert_eq!(status.state, TweakState::Active(OptLabel("On".into())));
    }

    #[test]
    fn empty_applicable_surface_is_unavailable_tweak() {
        let surface = vec![svc_effect("svc", false, None)];
        let options = vec![opt(
            "On",
            vec![("svc", set(Value::Startup(StartupType::Manual)))],
        )];
        let scope = WindowsScope {
            products: None,
            build: Some(BuildExpr::Min(26100)), // excludes the running milestone (19045)
            revision: None,
        };
        let t = tweak_scoped("demo", surface, options, Some(scope));
        let c = corpus(vec![t.clone()]);

        // No settings registered at all -- an empty surface must never attempt a read.
        let h = Harness::new(MockKind::default(), MockProbes::default());

        let status = detect(&t, &c, &h.deps());
        assert!(
            matches!(status.state, TweakState::Unavailable(_)),
            "expected Unavailable, got {:?}",
            status.state
        );
    }

    #[test]
    fn probe_cache_hit_no_respawn() {
        let effect = action_effect("act", true);
        let Effect::Action(action_def) = effect.kind.clone() else {
            unreachable!()
        };
        let surface = vec![effect];
        let options = vec![opt("Run", vec![("act", OptValue::Run(None))])];
        let t = tweak("demo", surface, options);
        let c = corpus(vec![t.clone()]);

        let probes = MockProbes::default().with(action_def, MockProbeOutcome::Present);
        let h = Harness::new(MockKind::default(), probes);

        let s1 = detect(&t, &c, &h.deps());
        let s2 = detect(&t, &c, &h.deps());
        assert_eq!(s1.state, TweakState::Active(OptLabel("Run".into())));
        assert_eq!(s2.state, TweakState::Active(OptLabel("Run".into())));
        assert_eq!(
            h.probes.calls.load(Ordering::SeqCst),
            1,
            "the probe must be cached across detects of the same tweak, never re-spawned"
        );
    }
}
