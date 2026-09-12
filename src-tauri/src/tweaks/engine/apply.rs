//! Apply pipeline + atomic rollback (spec §8.1, +§8.6/§5.4; ADR-0001/0002; invariants 2, 4, 5, 10,
//! 12, 13, 18, 20, 25). The highest-stakes module in the engine: a bug here can strand a machine
//! half-modified, so every mutation is preceded by a durable, gap-free record of what it will do
//! and what it must undo if it fails.
//!
//! ## The five steps (spec §8.1), each a named invariant this file's tests pin directly
//! 0. **Lock + detect.** [`lifecycle::lock_tweak`] serializes the whole sequence per tweak id
//!    (spec §8.7). Already-`Active(target)` is a verified no-op: no snapshot, nothing driven. An
//!    `Unknown` surface aborts before touching anything: a partially-observed surface is never a
//!    safe base for a decision (invariant 3).
//! 1. **Capture, all reads before any mutation** (invariant 4). Every applicable non-shared
//!    Setting is read once; every probeable Action the target *omits* is probed once, to decide
//!    up front (never touched again after this point) whether it needs driving back. A read/probe
//!    failure aborts here, having driven nothing.
//! 2. **Persist the WAL entry before mutating** (invariant 5). The journal is exactly the actions
//!    Step 1 already decided will run, built and pushed to disk before Step 3 drives a single
//!    effect.
//!
//!    3/4. **Drive + verify, in declaration order** (invariant 18). Each effect kind's own
//!    did-it-work check (invariant 2) triggers rollback the instant it fails.
//!
//! ## Rollback (ADR-0001)
//! On any Step 3/4 failure: undo the journal's completed actions in reverse order (a completed
//! no-undo action is reported un-undoable, never fatal to the rest); then drive the *whole*
//! captured pre-apply state back via [`drive_to_captured`]: drive-to-value is absolute, so partial
//! forward progress on Settings needs no per-step tracking. Shared claims taken/released during the
//! failed attempt are reversed too (claim ↔ release), tracked via [`ProcessedEffect`] alongside
//! completed actions, since the captured entry itself excludes shared effects (spec §8.1 step 1);
//! without this, a claim taken just before a later effect failed would survive the rollback
//! unreversed, leaving the claims record silently wrong. A verified full rollback consumes the
//! just-captured entry; any unverified restore keeps it and reports every unrecoverable item
//! (ADR-0001/0002, invariant 20); never `let _ =` on a rollback outcome.
//!
//! ## Deviation from the brief (flagged per the task's own instruction)
//! The brief's `apply` signature omits `corpus`; both `detect` (already reviewed, Task 11) and
//! resolving a captured `OptionRef`'s current definition (ADR-0007) need it, so it is added here:
//! `apply(tweak, corpus, target, deps)`.
//!
//! ## Per-effect routing (spec §9, ADR-0005; invariant 24)
//! DRIVES use [`context::route`] per effect (`max(floor, step)`; HKCU always in-process as the
//! user). READS use [`context::read_route`] and never escalate, staying at `Deps.level`. Reversals
//! route per effect too, re-derived from the CURRENT corpus (ADR-0007): [`rollback`],
//! `revert::undo_journal`, `revert::release_shared_claims`; a claim releases at its recorded level.

use std::collections::{BTreeMap, BTreeSet};

use crate::tweaks::kinds::{BatchFailure, Error as KindError, ExecCx};
use crate::tweaks::model::{
    ActionDef, Corpus, Effect, EffectDef, EffectId, Level, Opt, OptLabel, OptValue, ScopedValue,
    Setting, SharedDef, SharedId, Tweak, Value,
};
use crate::tweaks::shared_claims::{ClaimsError, ReleaseOutcome};
use crate::tweaks::snapshot::{
    Attention, AttentionItem, AttentionKind, AttentionReason, Captured, JournalRow, NewEntry, Seq,
    SnapshotError,
};
use crate::tweaks::validate::{applicable_surface, applicable_value, Milestone};
use crate::tweaks::winver::WinVer;

use super::context;
use super::detect::{self, HeldInfo, TweakState, TweakStatus, UnknownReason};
use super::{lifecycle, user_facing_failure, Deps, Phase};

/// Every way this pipeline can fail (spec §8.1). Each abort point names exactly what stage it
/// happened in, so a caller (and this file's own tests) can tell "aborted before touching
/// anything" apart from "rolled back after a partial mutation."
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("'{0}' is not one of this tweak's declared options")]
    UnknownOption(OptLabel),

    #[error("tweak surface is unreadable -- apply refuses to mutate: {0:?}")]
    SurfaceUnreadable(Vec<UnknownReason>),

    #[error("tweak is unavailable on this machine/build: {0}")]
    Unavailable(String),

    #[error(transparent)]
    AppExiting(#[from] lifecycle::AppExiting),

    /// Step 1: a capture read failed -- aborts before any mutation (invariant 4).
    #[error("capture of effect '{effect}' failed before any mutation: {source}")]
    CaptureFailed {
        effect: EffectId,
        #[source]
        source: KindError,
    },

    /// Step 1: a non-optional effect's capture read `Missing` (spec §5.4) -- typed, never guessed.
    #[error("effect '{0}' is required but its resource reads Missing")]
    CaptureMissingRequired(EffectId),

    /// Step 2: the WAL entry could not be persisted before mutating.
    #[error("could not persist the pre-apply snapshot entry: {0}")]
    SnapshotWrite(#[source] SnapshotError),

    /// Step 3: driving a Setting failed.
    #[error("drive of effect '{effect}' failed: {source}")]
    DriveFailed {
        effect: EffectId,
        #[source]
        source: KindError,
    },

    /// Step 3: a real (non-`Missing`) value was driven at a resource that reads `Missing` (spec
    /// §5.4) -- the resource vanished between detect and apply.
    #[error("effect '{0}': target resource no longer exists on this machine")]
    ResourceMissing(EffectId),

    /// Step 4: a Setting's read-back did not match what was driven (invariant 2).
    #[error("effect '{effect}' verify mismatch: drove {expected:?}, read back {actual:?}")]
    VerifyMismatch {
        effect: EffectId,
        expected: Value,
        actual: Value,
    },

    /// Step 3: an Action's `apply`/`undo` failed.
    #[error("action '{effect}' failed: {source}")]
    ActionFailed {
        effect: EffectId,
        #[source]
        source: KindError,
    },

    /// Step 4: an Action's post-run probe did not read the expected presence.
    #[error(
        "action '{effect}' verify mismatch: expected present={expected}, probed present={actual}"
    )]
    ActionVerifyMismatch {
        effect: EffectId,
        expected: bool,
        actual: bool,
    },

    /// Step 3: an action ran but its completion mark could not be durably persisted.
    #[error("could not durably mark action '{effect}' completed: {source}")]
    JournalMark {
        effect: EffectId,
        #[source]
        source: SnapshotError,
    },

    /// Step 3: a shared claim/release failed (spec §8.6).
    #[error("shared claim/release for '{shared}' failed: {source}")]
    Claim {
        shared: SharedId,
        #[source]
        source: ClaimsError,
    },

    /// A build-guard-prevented shape mismatch reached runtime anyway -- surfaced typed rather than
    /// panicking (a panic here could abort an entire elevated-broker batch).
    #[error("internal engine inconsistency: {0}")]
    Invalid(String),

    /// Step 1 of a restore: a completed action declares no `undo` (spec §8.5). No retry changes
    /// that, which is why it is typed apart from an internal inconsistency.
    #[error(
        "action '{0}' ran and cannot be undone (no undo script), so the restore is incomplete"
    )]
    NoUndo(EffectId),

    /// Atomic rollback's result (ADR-0001, invariant 20): the original failure, plus every failure
    /// the rollback itself hit.
    #[error("apply failed ({original}); {}", rollback_summary(rollback_failures, *outcome_unknown, store))]
    RollbackReport {
        original: Box<EngineError>,
        rollback_failures: Vec<EngineError>,
        /// Keeps the snapshot even when every restore verified (ADR-0005 amendment).
        outcome_unknown: bool,
        /// Snapshot-store failures, named as themselves: neither is an unrecovered resource.
        store: Vec<EngineError>,
    },

    /// A restore that did not fully verify; its entry is kept (ADR-0002).
    #[error(
        "restore failed ({}); the snapshot was kept{}",
        join_errors(failures),
        store_suffix(store)
    )]
    RestoreFailed {
        failures: Vec<EngineError>,
        store: Vec<EngineError>,
    },

    /// Every effect verified, so the machine is restored; only releasing the entry failed. Reported
    /// apart from a failed restore, which this is not (ADR-0002).
    #[error("the machine was restored, but its snapshot entry could not be removed: {0}")]
    EntryCleanup(#[source] SnapshotError),

    /// Needs Attention could not be persisted, so the failure it describes may not survive a
    /// restart. Never folded into the unrecovered-item count.
    #[error("Needs Attention could not be recorded: {0}")]
    AttentionWrite(#[source] SnapshotError),
}

fn rollback_summary(
    failures: &[EngineError],
    outcome_unknown: bool,
    store: &[EngineError],
) -> String {
    let outcome = match (failures.len(), outcome_unknown) {
        (0, false) => "rollback fully verified".to_string(),
        (0, true) => {
            "rollback restored every captured setting, but the elevated step's outcome is \
                      unknown, so the snapshot was kept"
                .to_string()
        }
        (n, _) => format!("rollback left {n} item(s) unrecoverable, so the snapshot was kept"),
    };
    format!("{outcome}{}", store_suffix(store))
}

fn store_suffix(store: &[EngineError]) -> String {
    if store.is_empty() {
        String::new()
    } else {
        format!("; {}", join_errors(store))
    }
}

fn join_errors(errors: &[EngineError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// One persisted Needs Attention line: the effect and a coarse kind, so the UI can tell a
/// retryable drive from a one-way action without parsing the message back apart. The message is
/// [`user_facing_failure`]'s text: this line reaches both disk and the UI, so it names no detail.
pub(super) fn attention_item(phase: Phase, error: &EngineError) -> AttentionItem {
    let (effect, kind) = match error {
        EngineError::DriveFailed {
            effect,
            source: KindError::ElevatedOutcomeUnknown(..),
        } => (Some(effect), AttentionKind::OutcomeUnknown),
        EngineError::DriveFailed { effect, .. }
        | EngineError::ResourceMissing(effect)
        | EngineError::CaptureFailed { effect, .. }
        | EngineError::CaptureMissingRequired(effect) => (Some(effect), AttentionKind::Drive),
        EngineError::VerifyMismatch { effect, .. }
        | EngineError::ActionVerifyMismatch { effect, .. } => (Some(effect), AttentionKind::Verify),
        EngineError::ActionFailed { effect, .. } => (Some(effect), AttentionKind::Action),
        EngineError::NoUndo(effect) => (Some(effect), AttentionKind::NoUndo),
        EngineError::JournalMark { effect, .. } => (Some(effect), AttentionKind::Store),
        EngineError::Claim {
            source: ClaimsError::Kind(KindError::ElevatedOutcomeUnknown(..)),
            ..
        } => (None, AttentionKind::OutcomeUnknown),
        EngineError::Claim { .. } => (None, AttentionKind::Claim),
        EngineError::SnapshotWrite(_)
        | EngineError::EntryCleanup(_)
        | EngineError::AttentionWrite(_) => (None, AttentionKind::Store),
        _ => (None, AttentionKind::Other),
    };
    AttentionItem {
        effect: effect.cloned(),
        kind,
        message: user_facing_failure(phase, error),
    }
}

/// An elevated child may have run ops and what it did cannot be proven: its response is lost or
/// untrusted, an SCM or Task Scheduler RPC may complete server-side after a kill, or the kill is
/// unconfirmed. So a verified rollback still keeps the entry, which ADR-0002 always allows.
fn outcome_is_unknown(error: &EngineError) -> bool {
    matches!(
        error,
        EngineError::DriveFailed {
            source: KindError::ElevatedOutcomeUnknown(..),
            ..
        } | EngineError::Claim {
            source: ClaimsError::Kind(KindError::ElevatedOutcomeUnknown(..)),
            ..
        }
    )
}

fn map_drive_err(effect: &EffectId, e: KindError) -> EngineError {
    match e {
        KindError::ResourceMissing(_) => EngineError::ResourceMissing(effect.clone()),
        other => EngineError::DriveFailed {
            effect: effect.clone(),
            source: other,
        },
    }
}

/// One effect's outcome (brief's Interfaces section: "reports per-effect results").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectResult {
    pub effect: EffectId,
    pub kind: EffectResultKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectResultKind {
    /// A Setting was driven to `desired` and verified.
    Driven { desired: Value },
    /// A shared effect was claimed (first capture, or a verified additional hold).
    Claimed,
    /// A shared effect was released; other claimants still hold it (info, not failure, spec §8.6).
    StillHeld(Vec<String>),
    /// A shared effect's last release restored the captured original.
    Released,
    /// An Action's `apply` ran and verified.
    Ran,
    /// An omitted undo-carrying Action's `undo` was driven back and verified (grill Q3).
    UndoDrivenBack,
    /// Nothing needed doing (e.g. target authors `Unclaimed` and this tweak never held it).
    NoOp,
}

/// `apply`'s result (grill Q1): per-effect results, plus a status computed from this operation's
/// OWN verify reads -- never a fresh `detect` re-scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub effects: Vec<EffectResult>,
    pub status: TweakStatus,
}

/// Which direction an Action ran in during Step 3 -- decided once during Step 1 (a pure read-time
/// decision) and reused unchanged at drive time and, on failure, at rollback time, so all three
/// phases agree on what actually happened.
/// `pub(crate)`: Task 13's restore reuses [`drive_forward`] verbatim to run an OptionRef target's
/// actions/ephemerals in declaration order (which dispatches to `drive_action` internally -- that
/// helper itself stays private, only reachable through `drive_forward`), so restore must be able to
/// name this plan's variants too (visibility-only -- see this file's module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionPlan {
    /// The target option runs this action's `apply`.
    Apply,
    /// The target omits an undo-carrying probeable action whose probe currently reads present --
    /// its `undo` is driven back instead (grill Q3, spec §8.1 step 3 / §8.4).
    UndoBack,
}

/// One effect Step 3 actually changed, in the order it happened -- rollback walks this in reverse.
/// Settings need no entry here: their pre-apply state is captured wholesale in Step 1, and
/// drive-to-value is absolute, so restoring the captured dump covers every Setting regardless of
/// how far Step 3 got (the brief's own callout: "partial apply progress needs no per-step
/// tracking"). Shared effects DO need tracking -- the captured entry excludes them entirely (their
/// lifecycle is the claims record, spec §8.6), so nothing else remembers what this attempt did.
#[derive(Debug, Clone)]
enum ProcessedEffect {
    Action(EffectId, ActionPlan),
    /// The `Level` is the routed level the claim was taken at: one surface may reference a shared
    /// id from several effects at different steps, so the id alone cannot name the level back.
    SharedClaim(SharedId, Level),
    /// Boxed: `SharedDef` (needed to re-`claim` on rollback) is far larger than the other
    /// variants, and this list is built one push at a time, never densely packed.
    SharedRelease(Box<SharedDef>, Level),
}

/// Whether a drive pass should durably mark completed actions into an on-disk WAL entry, or skip
/// that bookkeeping entirely (spec invariant 5 vs. Task 13's restore review fix). Apply's own WAL
/// discipline is unchanged (`To(seq)`, exactly the seq it just pushed); restore's re-apply of an
/// OptionRef target needs `None` -- the entry being restored is never consumed until the WHOLE
/// restore verifies, so a crash mid-re-apply just leaves that entry on disk and the caller retries
/// the whole restore. Fabricating a throwaway entry purely to satisfy a hardcoded `mark_completed`
/// call was a reviewed CRITICAL (an unconsumed, un-deduped phantom could outlive/mask the real
/// return-point on a crash or a failed discard) -- this makes the bookkeeping itself optional
/// instead of routing around it.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Journaling {
    /// Mark completed actions into the entry at this seq (spec §8.1 step 3, invariant 5).
    To(Seq),
    /// Skip completion marking -- no on-disk journal exists for this drive pass.
    None,
}

/// Bundles what every Step-3 helper needs, purely to keep argument lists short (clippy).
///
/// `pub(crate)`: Task 13's restore constructs one of these to reuse [`drive_forward`] verbatim for
/// an OptionRef target's re-apply (visibility-only -- see this file's module docs).
pub(crate) struct DriveCtx<'a> {
    pub(crate) tweak: &'a Tweak,
    pub(crate) corpus: &'a Corpus,
    pub(crate) target_opt: &'a Opt,
    pub(crate) milestone: Milestone,
    pub(crate) deps: &'a Deps<'a>,
    pub(crate) journal: Journaling,
}

/// `pub(crate)`/`effect_results`/`held_shared` only: Task 13's restore reads these back after
/// calling [`drive_forward`] to build its own outcome, but never touches `processed` (that field's
/// type stays module-private -- see this file's module docs).
#[derive(Default)]
pub(crate) struct DriveState {
    processed: Vec<ProcessedEffect>,
    pub(crate) effect_results: Vec<EffectResult>,
    pub(crate) held_shared: Vec<HeldInfo>,
}

impl DriveState {
    /// The actions this pass drove: on a pass that verified end to end, exactly the journal rows
    /// its outcome may resolve. Settings need no row and Shared effects have none.
    pub(crate) fn driven_actions(&self) -> BTreeSet<EffectId> {
        self.processed
            .iter()
            .filter_map(|p| match p {
                ProcessedEffect::Action(id, _) => Some(id.clone()),
                ProcessedEffect::SharedClaim(..) | ProcessedEffect::SharedRelease(..) => None,
            })
            .collect()
    }
}

/// Marks the journal rows a verified outcome accounted for, before its record is cleared: a crash
/// between the two leaves a stale mark the next clear releases, never a resolved tweak the next
/// crash scan marks again. A store failure only leaves residue that scan raises, so it is logged.
pub(crate) fn resolve_accounted(deps: &Deps, tweak_id: &str, accounted: &BTreeSet<EffectId>) {
    if let Err(e) = deps
        .snapshots
        .resolve_journal_rows(tweak_id, accounted, deps.machine_guid)
    {
        log::warn!(
            "tweak '{tweak_id}': could not resolve the journal rows this run accounted for: {e}"
        );
    }
}

/// Applies `target` to `tweak` (spec §8.1). Async only to hold the per-tweak lock across the whole
/// synchronous step0→4 sequence (spec §8.7) -- every injected effect operation itself is a plain
/// sync call (System/Ti broker routing is Task 14; today `ExecCx`'s `UnsupportedLevel` propagates
/// as an ordinary typed `Err`, not a special case here).
pub async fn apply(
    tweak: &Tweak,
    corpus: &Corpus,
    target: &OptLabel,
    deps: &Deps<'_>,
) -> Result<ApplyOutcome, EngineError> {
    let _guard = lifecycle::lock_tweak(&tweak.id).await?;
    do_apply(tweak, corpus, target, deps)
}

fn do_apply(
    tweak: &Tweak,
    corpus: &Corpus,
    target: &OptLabel,
    deps: &Deps,
) -> Result<ApplyOutcome, EngineError> {
    let target_opt = tweak
        .options
        .iter()
        .find(|o| &o.label == target)
        .ok_or_else(|| EngineError::UnknownOption(target.clone()))?;

    // `validate.rs`'s helpers below stay Milestone-shaped (build-only, see `winver.rs`'s module
    // docs) -- `winver` is kept alongside for the runtime scope decisions that must honor
    // `revision` too (`driving_surface`'s own tweak/effect-level check, and the per-option-value
    // Action scope check below).
    let winver = deps.running;
    let milestone = winver.to_milestone();
    let surface = applicable_surface(tweak, &milestone);
    // Distinct from `surface` in exactly one respect (review fix): ephemeral actions are INCLUDED
    // here. `applicable_surface` is a detection concept (spec §6.4/§8.4) and correctly excludes
    // them there -- an ephemeral leaves no persistent state for detection, capture, or reversal to
    // observe -- but a declared `run` ephemeral action still needs to physically RUN on apply (spec
    // §7). Only the action-plan-building loop and the driving pass below use this; Step 1's Setting
    // capture loop keeps using `surface` (moot either way -- it already ignores non-Setting kinds).
    let drive_surface = driving_surface(tweak, &winver);

    // Step 0: detect current status (the lock is already held by `apply`).
    let pre_status = detect::detect(tweak, corpus, deps);
    match &pre_status.state {
        TweakState::Active(label) if label == target => {
            // Verified no-op (spec §8.1 step 0): no snapshot pushed, nothing driven.
            return Ok(ApplyOutcome {
                effects: Vec::new(),
                status: pre_status,
            });
        }
        TweakState::Unknown(reasons) => {
            return Err(EngineError::SurfaceUnreadable(reasons.clone()));
        }
        TweakState::Unavailable(reason) => {
            return Err(EngineError::Unavailable(reason.clone()));
        }
        TweakState::Active(_) | TweakState::SystemDefault => {}
    }

    // Step 1: capture pre-apply state + decide the action plan -- ALL reads, no mutation yet
    // (invariant 4). A read/probe failure aborts here, having touched nothing. Reads never escalate
    // (invariant 24): each effect's read runs at `context::read_route` -- `Deps.level` (the
    // ceiling) for everything, except an HKCU Setting is still read in-process as the interactive
    // user (see this file's module docs).
    let mut captured_values: BTreeMap<EffectId, Value> = BTreeMap::new();
    for effect in &surface {
        let Effect::Setting(setting) = &effect.kind else {
            continue;
        };
        let cx = context::read_route(effect, deps.level, corpus);
        match deps.kinds.read(setting, &cx) {
            Ok(Value::Missing) if effect.optional => {
                captured_values.insert(effect.id.clone(), Value::Missing);
            }
            Ok(Value::Missing) => {
                return Err(EngineError::CaptureMissingRequired(effect.id.clone()));
            }
            Ok(v) => {
                captured_values.insert(effect.id.clone(), v);
            }
            Err(e) => {
                return Err(EngineError::CaptureFailed {
                    effect: effect.id.clone(),
                    source: e,
                });
            }
        }
    }

    let mut action_plan: Vec<(EffectId, ActionPlan)> = Vec::new();
    let mut residues: Vec<EffectId> = Vec::new();
    for effect in &drive_surface {
        let Effect::Action(action_def) = &effect.kind else {
            continue;
        };
        // Runtime scope decision (spec §6.6/invariant 22): honors `revision` too, unlike the
        // Milestone-based (build-only) `surface`/`drive_surface` computed above -- see winver.rs's
        // module docs.
        let raw = target_opt.values.get(&effect.id);
        let scoped_out =
            matches!(raw, Some(OptValue::Run(w)) if !w.as_ref().is_none_or(|s| s.applies(&winver)));
        if scoped_out {
            continue; // never read, never journaled, never driven
        }
        let runs = matches!(raw, Some(OptValue::Run(_)));
        if runs {
            action_plan.push((effect.id.clone(), ActionPlan::Apply));
            continue;
        }
        // Genuinely omitted -- the expectation splits on `undo` (spec §8.4/§10).
        if let ActionDef::Script {
            probe: Some(_),
            undo,
            ..
        } = action_def
        {
            let cx = context::read_route(effect, deps.level, corpus);
            let present =
                deps.probes
                    .probe(action_def, &cx)
                    .map_err(|e| EngineError::CaptureFailed {
                        effect: effect.id.clone(),
                        source: e,
                    })?;
            if present {
                if undo.is_some() {
                    action_plan.push((effect.id.clone(), ActionPlan::UndoBack));
                } else {
                    residues.push(effect.id.clone()); // no-undo -- disclosed, left in place
                }
            }
        }
    }

    let captured = match &pre_status.state {
        TweakState::Active(label) => Captured::OptionRef(label.0.clone()),
        _ => Captured::Values(captured_values),
    };

    // Step 2: persist the WAL entry BEFORE mutating (invariant 5) -- the journal is exactly the
    // actions Step 1 already decided will run, MINUS ephemeral ones (review fix): an ephemeral
    // changes no persistent state, so a crash after it ran needs no recovery -- journaling it would
    // make the crash-window scan spuriously flag Needs Attention, and (were it ever undone) rollback/
    // restore would find it un-undoable. `ephemeral: true` is exempt from ALL reversibility
    // bookkeeping (spec §7, invariant 10), enforced here at the source rather than by exempting every
    // downstream consumer.
    let journal: Vec<JournalRow> = action_plan
        .iter()
        .filter(|(id, _)| !find_action(tweak, id).is_some_and(|(_, a)| is_ephemeral(a)))
        .map(|(id, _)| JournalRow {
            action_id: id.clone(),
            intended: true,
            completed: false,
            resolved: false,
        })
        .collect();
    let seq = deps
        .snapshots
        .push(
            &tweak.id,
            NewEntry {
                captured: captured.clone(),
                journal,
            },
            corpus,
            deps.machine_guid,
            milestone.build,
        )
        .map_err(EngineError::SnapshotWrite)?;

    // Steps 3/4: drive forward in declaration order (invariant 18), verifying as we go.
    let ctx = DriveCtx {
        tweak,
        corpus,
        target_opt,
        milestone,
        deps,
        journal: Journaling::To(seq),
    };
    let mut state = DriveState::default();
    let drive_result = drive_forward(&ctx, &drive_surface, &action_plan, &mut state);

    match drive_result {
        Ok(()) => {
            deps.probe_cache.invalidate(&tweak.id);
            // A fully verified apply is one of the three clears (ADR-0002), and it accounts for
            // the actions it drove -- never for a row it left alone. A failed clear keeps the
            // record, so the status below still reports it rather than dropping it silently.
            resolve_accounted(deps, &tweak.id, &state.driven_actions());
            if let Err(e) = deps.snapshots.clear_attention(&tweak.id, deps.machine_guid) {
                log::warn!("tweak '{}': could not clear Needs Attention: {e}", tweak.id);
            }
            Ok(ApplyOutcome {
                effects: state.effect_results,
                status: TweakStatus {
                    state: TweakState::Active(target.clone()),
                    unavailable: pre_status.unavailable,
                    residues,
                    has_history: true,
                    attention: detect::attention(&tweak.id, deps),
                    held_shared: state.held_shared,
                    observed: Vec::new(),
                },
            })
        }
        Err(original) => {
            // Atomic rollback (ADR-0001): never `let _ =` this result.
            let rollback_failures = rollback(tweak, corpus, &captured, &state.processed, deps);
            deps.probe_cache.invalidate(&tweak.id);
            let outcome_unknown = outcome_is_unknown(&original);
            let mut store = Vec::new();
            if rollback_failures.is_empty() && !outcome_unknown {
                // Verified full restore: the machine now matches the just-captured entry, so
                // consume it (ADR-0002). A failed consume leaves the entry on disk, the safe
                // failure mode, and is named as itself instead of as an unrecovered resource.
                if let Err(e) = deps.snapshots.consume(&tweak.id, seq) {
                    store.push(EngineError::EntryCleanup(e));
                }
            } else {
                let attention = Attention {
                    reason: AttentionReason::ApplyFailed,
                    items: std::iter::once(&original)
                        .chain(&rollback_failures)
                        .map(|e| attention_item(Phase::Apply, e))
                        .collect(),
                };
                if let Err(e) =
                    deps.snapshots
                        .set_attention(&tweak.id, deps.machine_guid, attention)
                {
                    store.push(EngineError::AttentionWrite(e));
                }
            }
            Err(EngineError::RollbackReport {
                original: Box::new(original),
                rollback_failures,
                outcome_unknown,
                store,
            })
        }
    }
}

/// The surface actually driven on apply (spec §8.1 step 3); `pub(crate)` so the availability gate
/// scopes effects out exactly as apply does. Unlike `validate::applicable_surface` it keeps
/// ephemeral actions: a declared `run` ephemeral still executes when its option is applied (spec §7).
pub(crate) fn driving_surface<'a>(tweak: &'a Tweak, winver: &WinVer) -> Vec<&'a EffectDef> {
    if !tweak.windows.as_ref().is_none_or(|s| s.applies(winver)) {
        return Vec::new();
    }
    tweak
        .surface
        .iter()
        .filter(|e| e.windows.as_ref().is_none_or(|s| s.applies(winver)))
        .collect()
}

/// `pub(crate)`: Task 13's restore reuses this verbatim (not duplicated) to drive an OptionRef
/// target's Shared/Action effects in declaration order (visibility-only -- see this file's module
/// docs; the body below is byte-identical to Task 12's).
pub(crate) fn drive_forward(
    ctx: &DriveCtx,
    surface: &[&EffectDef],
    action_plan: &[(EffectId, ActionPlan)],
    state: &mut DriveState,
) -> Result<(), EngineError> {
    let mut index = 0;
    while index < surface.len() {
        // A run of consecutive same-level elevated Settings shares ONE child (spec §9's grouped
        // execution, invariant 18). Acquiring TrustedInstaller is entirely a per-spawn cost, so a
        // tweak with twenty such effects used to pay it twenty times.
        let run = brokerable_run(ctx, surface, index);
        if run >= 2 {
            drive_setting_run(ctx, &surface[index..index + run], state)?;
            index += run;
            continue;
        }
        let effect = surface[index];
        index += 1;

        match &effect.kind {
            Effect::Setting(setting) => {
                let Some(scoped) = setting_target(ctx, effect)? else {
                    continue; // this option-value is scoped out here (spec §6.6)
                };
                // Per-effect execution context (spec §9): `route` computes effective =
                // max(floor, step), EXCEPT an HKCU Setting always drives in-process as the
                // interactive user regardless of the floor (see this file's module docs).
                let cx = context::route(effect, ctx.tweak, ctx.corpus);
                // An `optional` effect whose resource is absent already reads as its `if_missing`
                // value (spec §5.4) -- that is exactly what `detect::classify_setting_read` does,
                // which is why detect offers such an option as available and satisfied. Apply has
                // to agree: when the option asks for precisely that value there is nothing to drive
                // and nothing to verify, so this is a verified no-op. Without it, detect advertises
                // an option that apply always aborts and rolls back on (a task absent on this
                // Windows build, say), and `optional` would protect capture and detect but not the
                // drive that actually needs it.
                if let Err(e) = ctx.deps.kinds.drive(setting, &scoped.value, &cx) {
                    if !(matches!(e, KindError::ResourceMissing(_))
                        && effect.optional
                        && effect.if_missing.as_ref() == Some(&scoped.value))
                    {
                        return Err(map_drive_err(&effect.id, e));
                    }
                    state.effect_results.push(EffectResult {
                        effect: effect.id.clone(),
                        kind: EffectResultKind::NoOp,
                    });
                    continue;
                }
                let actual = ctx
                    .deps
                    .kinds
                    .read(setting, &cx)
                    .map_err(|e| map_drive_err(&effect.id, e))?;
                if actual != scoped.value {
                    return Err(EngineError::VerifyMismatch {
                        effect: effect.id.clone(),
                        expected: scoped.value.clone(),
                        actual,
                    });
                }
                state.effect_results.push(EffectResult {
                    effect: effect.id.clone(),
                    kind: EffectResultKind::Driven {
                        desired: scoped.value.clone(),
                    },
                });
            }
            Effect::Shared(shared_id) => drive_shared(ctx, effect, shared_id, state)?,
            Effect::Action(action_def) => {
                drive_action(ctx, effect, action_def, action_plan, state)?
            }
        }
    }
    Ok(())
}

fn drive_shared(
    ctx: &DriveCtx,
    effect: &EffectDef,
    shared_id: &SharedId,
    state: &mut DriveState,
) -> Result<(), EngineError> {
    // Per-effect execution context (spec §9). `route` resolves a Shared effect through the corpus
    // to the block's own Setting, so a shared HKCU block gets the same in-process-as-the-user
    // treatment as an inlined one; otherwise this is
    // `effective_level(tweak.elevation, effect.elevation)`, replacing the flat `Deps.level` ceiling.
    let cx = context::route(effect, ctx.tweak, ctx.corpus);
    let Some(opt_value) = applicable_value(ctx.target_opt, &effect.id, &ctx.milestone) else {
        return Ok(());
    };
    let shared_def = ctx
        .corpus
        .shared
        .iter()
        .find(|s| &s.id == shared_id)
        .ok_or_else(|| {
            EngineError::Invalid(format!("shared '{shared_id}' not declared in corpus"))
        })?;

    match opt_value {
        OptValue::Claim(_) => {
            ctx.deps
                .claims
                .claim(shared_def, &ctx.tweak.id, ctx.deps.kinds, &cx)
                .map_err(|e| EngineError::Claim {
                    shared: shared_id.clone(),
                    source: e,
                })?;
            state
                .processed
                .push(ProcessedEffect::SharedClaim(shared_id.clone(), cx.level()));
            state.effect_results.push(EffectResult {
                effect: effect.id.clone(),
                kind: EffectResultKind::Claimed,
            });
        }
        OptValue::Unclaimed(_) => {
            let already_held = ctx
                .deps
                .claims
                .holders(shared_id)
                .iter()
                .any(|h| h == &ctx.tweak.id);
            if already_held {
                let outcome = ctx
                    .deps
                    .claims
                    .release(shared_id, &ctx.tweak.id, ctx.deps.kinds, &cx)
                    .map_err(|e| EngineError::Claim {
                        shared: shared_id.clone(),
                        source: e,
                    })?;
                state.processed.push(ProcessedEffect::SharedRelease(
                    Box::new(shared_def.clone()),
                    cx.level(),
                ));
                state.effect_results.push(EffectResult {
                    effect: effect.id.clone(),
                    kind: match outcome {
                        ReleaseOutcome::StillHeld(holders) => EffectResultKind::StillHeld(holders),
                        ReleaseOutcome::RestoredOriginal => EffectResultKind::Released,
                    },
                });
            } else {
                state.effect_results.push(EffectResult {
                    effect: effect.id.clone(),
                    kind: EffectResultKind::NoOp,
                });
            }
        }
        _ => {
            return Err(EngineError::Invalid(format!(
                "shared effect '{}' has a non-shared option value",
                effect.id
            )));
        }
    }

    let holders = ctx.deps.claims.holders(shared_id);
    if !holders.is_empty() {
        state.held_shared.push(HeldInfo {
            shared: shared_id.clone(),
            holders,
        });
    }
    Ok(())
}

fn drive_action(
    ctx: &DriveCtx,
    effect: &EffectDef,
    action_def: &ActionDef,
    action_plan: &[(EffectId, ActionPlan)],
    state: &mut DriveState,
) -> Result<(), EngineError> {
    // Per-effect execution context (spec §9) -- an Action is never HKCU by construction (see
    // `context::route`'s docs), so this is `effective_level(tweak.elevation, effect.elevation)` in
    // practice, replacing the flat `Deps.level` ceiling.
    let cx = context::route(effect, ctx.tweak, ctx.corpus);
    let Some((_, plan)) = action_plan.iter().find(|(id, _)| id == &effect.id) else {
        return Ok(()); // not in the plan: scoped out, or genuinely nothing to do
    };
    match plan {
        ActionPlan::Apply => {
            ctx.deps
                .actions
                .apply(action_def, &cx)
                .map_err(|e| EngineError::ActionFailed {
                    effect: effect.id.clone(),
                    source: e,
                })?;
            // Ephemeral actions run unconditionally (just above) but participate in NO
            // reversibility bookkeeping (spec §7, invariant 10): no completion mark (Step 2 never
            // journaled it in the first place -- nothing to mark), no `state.processed` entry (so
            // rollback can never encounter it and mistake it for un-undoable).
            if !is_ephemeral(action_def) {
                if let Journaling::To(seq) = ctx.journal {
                    ctx.deps
                        .snapshots
                        .mark_completed(&ctx.tweak.id, seq, &effect.id)
                        .map_err(|e| EngineError::JournalMark {
                            effect: effect.id.clone(),
                            source: e,
                        })?;
                }
                state.processed.push(ProcessedEffect::Action(
                    effect.id.clone(),
                    ActionPlan::Apply,
                ));
            }
            if let ActionDef::Script { probe: Some(_), .. } = action_def {
                let present = ctx.deps.probes.probe(action_def, &cx).map_err(|e| {
                    EngineError::ActionFailed {
                        effect: effect.id.clone(),
                        source: e,
                    }
                })?;
                if !present {
                    return Err(EngineError::ActionVerifyMismatch {
                        effect: effect.id.clone(),
                        expected: true,
                        actual: false,
                    });
                }
            }
            state.effect_results.push(EffectResult {
                effect: effect.id.clone(),
                kind: EffectResultKind::Ran,
            });
        }
        ActionPlan::UndoBack => {
            ctx.deps
                .actions
                .undo(action_def, &cx)
                .map_err(|e| EngineError::ActionFailed {
                    effect: effect.id.clone(),
                    source: e,
                })?;
            if let Journaling::To(seq) = ctx.journal {
                ctx.deps
                    .snapshots
                    .mark_completed(&ctx.tweak.id, seq, &effect.id)
                    .map_err(|e| EngineError::JournalMark {
                        effect: effect.id.clone(),
                        source: e,
                    })?;
            }
            state.processed.push(ProcessedEffect::Action(
                effect.id.clone(),
                ActionPlan::UndoBack,
            ));
            let present =
                ctx.deps
                    .probes
                    .probe(action_def, &cx)
                    .map_err(|e| EngineError::ActionFailed {
                        effect: effect.id.clone(),
                        source: e,
                    })?;
            if present {
                return Err(EngineError::ActionVerifyMismatch {
                    effect: effect.id.clone(),
                    expected: false,
                    actual: true,
                });
            }
            state.effect_results.push(EffectResult {
                effect: effect.id.clone(),
                kind: EffectResultKind::UndoDrivenBack,
            });
        }
    }
    Ok(())
}

/// Atomic rollback (ADR-0001): undo completed actions in reverse order, reverse any shared
/// claim/release the failed attempt made, then drive the captured pre-apply state back. Returns
/// every failure hit along the way -- empty means a fully verified restore.
fn rollback(
    tweak: &Tweak,
    corpus: &Corpus,
    captured: &Captured,
    processed: &[ProcessedEffect],
    deps: &Deps,
) -> Vec<EngineError> {
    let mut failures = Vec::new();

    for item in processed.iter().rev() {
        match item {
            ProcessedEffect::Action(effect_id, ActionPlan::Apply) => {
                let Some((effect, action_def)) = find_action(tweak, effect_id) else {
                    failures.push(EngineError::Invalid(format!(
                        "completed action '{effect_id}' vanished from the surface during rollback"
                    )));
                    continue;
                };
                let cx = context::route(effect, tweak, corpus);
                if is_ephemeral(action_def) {
                    // An ephemeral is exempt from ALL reversibility bookkeeping (spec §7, invariant
                    // 10): skip it, never report it un-undoable. Step 2 journals none and
                    // `drive_action` records none, so reaching this arm means a new path added one.
                } else if has_undo(action_def) {
                    match deps.actions.undo(action_def, &cx) {
                        Ok(()) => {
                            // Did-it-work (invariant 19): probe/read-back must be checked
                            // identically at apply AND rollback time. An `undo` script that
                            // exits 0 without actually reverting the resource (a bug, a race, a
                            // privilege issue that didn't surface as non-zero) must never let
                            // this rollback silently appear complete -- reversing an Apply
                            // expects the produced state to now read absent.
                            verify_reversed_probe(
                                action_def,
                                effect,
                                false,
                                corpus,
                                deps,
                                &mut failures,
                            );
                        }
                        Err(e) => failures.push(EngineError::ActionFailed {
                            effect: effect_id.clone(),
                            source: e,
                        }),
                    }
                } else {
                    log::warn!(
                        "tweak '{}': completed action '{effect_id}' has no undo -- reported un-undoable, rollback incomplete",
                        tweak.id
                    );
                    failures.push(EngineError::Invalid(format!(
                        "action '{effect_id}' ran and cannot be undone (no undo script) -- rollback is incomplete"
                    )));
                }
            }
            ProcessedEffect::Action(effect_id, ActionPlan::UndoBack) => {
                let Some((effect, action_def)) = find_action(tweak, effect_id) else {
                    failures.push(EngineError::Invalid(format!(
                        "completed action '{effect_id}' vanished from the surface during rollback"
                    )));
                    continue;
                };
                let cx = context::route(effect, tweak, corpus);
                // Reversing a drive-back-undo always means re-running `apply` -- mandatory on
                // every `ActionDef::Script`/`DeleteTree`, so this is never "un-undoable".
                match deps.actions.apply(action_def, &cx) {
                    Ok(()) => {
                        // Reversing an UndoBack expects the produced state to now read present
                        // again (invariant 19 -- same probe-verify discipline as the forward
                        // path in `drive_action`, never downgraded to exit-code-only here).
                        verify_reversed_probe(
                            action_def,
                            effect,
                            true,
                            corpus,
                            deps,
                            &mut failures,
                        );
                    }
                    Err(e) => failures.push(EngineError::ActionFailed {
                        effect: effect_id.clone(),
                        source: e,
                    }),
                }
            }
            ProcessedEffect::SharedClaim(shared_id, level) => {
                let cx = ExecCx::new(*level);
                if let Err(e) = deps.claims.release(shared_id, &tweak.id, deps.kinds, &cx) {
                    failures.push(EngineError::Claim {
                        shared: shared_id.clone(),
                        source: e,
                    });
                }
            }
            ProcessedEffect::SharedRelease(shared_def, level) => {
                let cx = ExecCx::new(*level);
                // Re-`claim`s rather than restoring the historical "original" directly: this
                // re-captures a fresh original from the live value at THIS moment. That is
                // equivalent here, not a shortcut -- `release`'s own read-back verification
                // (shared_claims.rs) already guarantees the live value equals the true original
                // by the time this reversal runs, so re-capturing it changes nothing.
                if let Err(e) = deps.claims.claim(shared_def, &tweak.id, deps.kinds, &cx) {
                    failures.push(EngineError::Claim {
                        shared: shared_def.id.clone(),
                        source: e,
                    });
                }
            }
        }
    }

    if let Err(errs) = drive_to_captured(captured, &tweak.id, corpus, deps) {
        failures.extend(errs);
    }

    failures
}

/// Did-it-work for an action reversal (invariant 19): the probe is checked exactly as at apply
/// time, never downgraded to trusting the reversal's exit code, and a probe `Err` is a failure,
/// never a benign `Ok(false)`. It only READS, so it routes at `read_route` and never escalates.
pub(crate) fn verify_reversed_probe(
    action_def: &ActionDef,
    effect: &EffectDef,
    expected_present: bool,
    corpus: &Corpus,
    deps: &Deps,
    failures: &mut Vec<EngineError>,
) {
    let ActionDef::Script { probe: Some(_), .. } = action_def else {
        return;
    };
    let cx = context::read_route(effect, deps.level, corpus);
    match deps.probes.probe(action_def, &cx) {
        Ok(present) if present == expected_present => {}
        Ok(actual) => failures.push(EngineError::ActionVerifyMismatch {
            effect: effect.id.clone(),
            expected: expected_present,
            actual,
        }),
        Err(e) => failures.push(EngineError::ActionFailed {
            effect: effect.id.clone(),
            source: e,
        }),
    }
}

/// The effect is returned alongside its `ActionDef` because every caller that reverses one must
/// route it, and only the `EffectDef` carries the step level [`context::route`] needs.
fn find_action<'a>(
    tweak: &'a Tweak,
    effect_id: &EffectId,
) -> Option<(&'a EffectDef, &'a ActionDef)> {
    tweak
        .surface
        .iter()
        .find(|e| &e.id == effect_id)
        .and_then(|e| match &e.kind {
            Effect::Action(a) => Some((e, a)),
            _ => None,
        })
}

fn has_undo(action: &ActionDef) -> bool {
    match action {
        ActionDef::Script { undo, .. } | ActionDef::DeleteTree { undo, .. } => undo.is_some(),
    }
}

/// Whether `action` is ephemeral (spec §7): exempt from ALL reversibility bookkeeping -- never
/// journaled (Step 2), never added to `state.processed` (`drive_action`), and (defensively, in case
/// a future path ever surfaces one anyway) never treated as an un-undoable failure by `rollback` or
/// Task 13's `revert::undo_journal`.
fn is_ephemeral(action: &ActionDef) -> bool {
    matches!(
        action,
        ActionDef::Script {
            ephemeral: true,
            ..
        }
    )
}

/// Drives every captured Setting back to its pre-apply value (spec §8.1/§8.5, ADR-0007) -- the
/// primitive Task 13's restore reuses verbatim, since a Restore's re-apply-target step and a
/// rollback's captured-state-back step are the same drive-to-value operation. Re-derives the
/// tweak from the CURRENT corpus by id (never trusts a possibly-stale `&Tweak`), matching
/// ADR-0007's "restore re-derives from the current corpus." Returns every drive/verify failure;
/// empty means every captured Setting was restored and verified.
pub(crate) fn drive_to_captured(
    captured: &Captured,
    tweak_id: &str,
    corpus: &Corpus,
    deps: &Deps,
) -> Result<(), Vec<EngineError>> {
    let Some(tweak) = corpus.tweaks.iter().find(|t| t.id == tweak_id) else {
        return Err(vec![EngineError::Invalid(format!(
            "tweak '{tweak_id}' no longer exists in the corpus"
        ))]);
    };
    // Build-only Milestone shape is sufficient here -- this only reuses `applicable_surface` (no
    // direct scope_admits call), see winver.rs's module docs.
    let milestone = deps.running.to_milestone();
    let surface = applicable_surface(tweak, &milestone);
    let mut failures = Vec::new();
    let mut plan: Vec<Restorable> = Vec::new();

    match captured {
        Captured::Values(map) => {
            for (effect_id, value) in map {
                if *value == Value::Missing {
                    continue; // driving to Missing is a defined no-op (spec §5.4)
                }
                let Some(effect) = surface.iter().copied().find(|e| e.id == *effect_id) else {
                    continue; // no longer on the applicable surface here -- nothing to drive
                };
                let Effect::Setting(setting) = &effect.kind else {
                    continue;
                };
                // Per-effect execution context (spec §9): see this file's module docs.
                plan.push(Restorable {
                    effect,
                    setting,
                    value,
                    cx: context::route(effect, tweak, corpus),
                });
            }
        }
        Captured::OptionRef(label) => {
            let Some(opt) = tweak.options.iter().find(|o| &o.label.0 == label) else {
                failures.push(EngineError::Invalid(format!(
                    "captured option '{label}' no longer exists on tweak '{tweak_id}'"
                )));
                return Err(failures);
            };
            for effect in surface.iter().copied() {
                let Effect::Setting(setting) = &effect.kind else {
                    continue;
                };
                let Some(OptValue::Set(scoped)) = applicable_value(opt, &effect.id, &milestone)
                else {
                    continue;
                };
                if scoped.value == Value::Missing {
                    continue;
                }
                plan.push(Restorable {
                    effect,
                    setting,
                    value: &scoped.value,
                    cx: context::route(effect, tweak, corpus),
                });
            }
        }
    }

    // An `optional` effect whose resource is absent already sits at its `if_missing` value (spec
    // §5.4), which is where this would drive it. The forward path drops it from a run for the same
    // reason: inside a batch, its refusal to translate would abort every item behind it.
    plan.retain(|item| !absent_optional(item, deps));

    drive_back(&plan, deps, &mut failures);

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// One captured Setting the undo side has to put back, resolved before anything is driven so a run
/// of consecutive elevated ones can be recognised and share a single child.
struct Restorable<'a> {
    effect: &'a EffectDef,
    setting: &'a Setting,
    value: &'a Value,
    cx: ExecCx,
}

fn absent_optional(item: &Restorable<'_>, deps: &Deps) -> bool {
    item.effect.optional
        && item.effect.if_missing.as_ref() == Some(item.value)
        && matches!(deps.kinds.read(item.setting, &item.cx), Ok(Value::Missing))
}

/// Drives the whole plan back, each run of consecutive elevated Settings through ONE child. Runs
/// are adjacent equals only, exactly as on the forward path (invariant 18): the machine goes back
/// in the plan's own order and grouping never moves an effect past another.
fn drive_back(plan: &[Restorable<'_>], deps: &Deps, failures: &mut Vec<EngineError>) {
    let mut index = 0;
    while index < plan.len() {
        let run = brokerable_undo_run(plan, index);
        if run >= 2 {
            drive_back_run(&plan[index..index + run], deps, failures);
            index += run;
            continue;
        }
        drive_and_verify(&plan[index], deps, failures);
        index += 1;
    }
}

/// How many plan items from `from` can share one elevated child: the undo side's [`brokerable_run`],
/// reading the level off the context already routed per effect.
fn brokerable_undo_run(plan: &[Restorable<'_>], from: usize) -> usize {
    plan[from..]
        .iter()
        .take_while(|item| {
            item.cx.level() == Level::Ti
                && !matches!(item.setting, Setting::Hosts(_) | Setting::Firewall(_))
        })
        .count()
}

/// One run back through a single elevated child, then a read-back per item. An undo never aborts
/// early (ADR-0001): what the child proved it drove is verified, the failing item is charged to
/// its own effect, and what it never reached comes back through here.
fn drive_back_run(run: &[Restorable<'_>], deps: &Deps, failures: &mut Vec<EngineError>) {
    let Some(first) = run.first() else {
        return;
    };
    if run.len() < 2 {
        drive_and_verify(first, deps, failures);
        return;
    }
    let items: Vec<(&Setting, &Value)> = run.iter().map(|i| (i.setting, i.value)).collect();
    let Err(failed) = deps.kinds.drive_batch(&items, &first.cx) else {
        for item in run {
            verify_restored(item, deps, failures);
        }
        return;
    };
    // Clamped, not trusted: a report naming an item outside the run must not panic the one path a
    // half-changed machine depends on.
    let charged = failed.index.min(run.len() - 1);
    // The token failed, not one item: re-spawning for the rest of the run would repeat the same
    // wait and charge every effect for a failure that provably reached none of them.
    if matches!(failed.error, KindError::CouldNotAcquireElevation(..)) {
        failures.push(map_drive_err(&run[charged].effect.id, failed.error));
        return;
    }
    let driven = failed.completed.min(charged);
    for item in &run[..driven] {
        verify_restored(item, deps, failures);
    }
    drive_back_run(&run[driven..charged], deps, failures);
    failures.push(map_drive_err(&run[charged].effect.id, failed.error));
    drive_back_run(&run[charged + 1..], deps, failures);
}

fn drive_and_verify(item: &Restorable<'_>, deps: &Deps, failures: &mut Vec<EngineError>) {
    if let Err(e) = deps.kinds.drive(item.setting, item.value, &item.cx) {
        failures.push(map_drive_err(&item.effect.id, e));
        return;
    }
    verify_restored(item, deps, failures);
}

fn verify_restored(item: &Restorable<'_>, deps: &Deps, failures: &mut Vec<EngineError>) {
    match deps.kinds.read(item.setting, &item.cx) {
        Ok(actual) if &actual == item.value => {}
        Ok(actual) => failures.push(EngineError::VerifyMismatch {
            effect: item.effect.id.clone(),
            expected: item.value.clone(),
            actual,
        }),
        Err(e) => failures.push(map_drive_err(&item.effect.id, e)),
    }
}

/// The scoped target for `effect` under the drive's option, if it applies here.
///
/// `None` means the option-value is scoped out on this build (spec §6.6) and there is nothing to
/// drive; `Err` means the option named a non-Set value for a Setting, which is a corpus bug.
fn setting_target<'a>(
    ctx: &'a DriveCtx,
    effect: &EffectDef,
) -> Result<Option<&'a ScopedValue>, EngineError> {
    match applicable_value(ctx.target_opt, &effect.id, &ctx.milestone) {
        None => Ok(None),
        Some(OptValue::Set(scoped)) => Ok(Some(scoped)),
        Some(_) => Err(EngineError::Invalid(format!(
            "effect '{}' is a Setting but its option value is not Set",
            effect.id
        ))),
    }
}

/// How many effects starting at `from` can share one elevated child.
///
/// A run is consecutive Setting effects that all route to `Ti` and all have a brokerable address.
/// Anything else -- an in-process Setting, a Shared, an Action, a scoped-out value, or a Hosts /
/// Firewall address the broker has no op for -- ends the run, because it has to be driven by the
/// existing per-effect path. Order is never changed; only adjacent equals are grouped
/// (invariant 18).
fn brokerable_run(ctx: &DriveCtx, surface: &[&EffectDef], from: usize) -> usize {
    surface[from..]
        .iter()
        .take_while(|effect| {
            let Effect::Setting(setting) = &effect.kind else {
                return false;
            };
            if matches!(setting, Setting::Hosts(_) | Setting::Firewall(_)) {
                return false;
            }
            if !matches!(setting_target(ctx, effect), Ok(Some(_))) {
                return false;
            }
            context::route(effect, ctx.tweak, ctx.corpus).level() == Level::Ti
        })
        .count()
}

/// Drive a run of same-level Setting effects through ONE elevated child, then verify each.
///
/// Semantics match the per-effect path exactly, with one deliberate difference: every drive in the
/// run happens before any of the verifies, because they all cross into the child together. For the
/// one tweak that reaches this path that is an improvement rather than a compromise -- its first
/// effects disable the WaaSMedic tasks that would otherwise contend with the later ones, so a
/// shorter window between the first drive and the last is strictly better.
fn drive_setting_run(
    ctx: &DriveCtx,
    run: &[&EffectDef],
    state: &mut DriveState,
) -> Result<(), EngineError> {
    // Resolve absent optionals first. In the per-effect path an absent `optional` resource is a
    // verified no-op and the loop moves on; inside a batch the same effect would abort the whole
    // run, so it must never enter one. Reads never escalate, so this is the same in-process read
    // the translation would have done anyway.
    let mut batched: Vec<(&EffectDef, &ScopedValue)> = Vec::with_capacity(run.len());
    for effect in run {
        let Some(scoped) = setting_target(ctx, effect)? else {
            continue;
        };
        let Effect::Setting(setting) = &effect.kind else {
            continue;
        };
        let cx = context::route(effect, ctx.tweak, ctx.corpus);
        let absent_optional = effect.optional
            && effect.if_missing.as_ref() == Some(&scoped.value)
            && matches!(ctx.deps.kinds.read(setting, &cx), Ok(Value::Missing));
        if absent_optional {
            state.effect_results.push(EffectResult {
                effect: effect.id.clone(),
                kind: EffectResultKind::NoOp,
            });
            continue;
        }
        batched.push((effect, scoped));
    }

    let Some((first, _)) = batched.first() else {
        return Ok(());
    };
    let cx = context::route(first, ctx.tweak, ctx.corpus);

    let items: Vec<(&Setting, &Value)> = batched
        .iter()
        .map(|(effect, scoped)| match &effect.kind {
            Effect::Setting(setting) => (setting, &scoped.value),
            _ => unreachable!("brokerable_run admits only Setting effects"),
        })
        .map(|(setting, value): (&Setting, &Value)| (setting, value))
        .collect();

    if let Err(BatchFailure { index, error, .. }) = ctx.deps.kinds.drive_batch(&items, &cx) {
        let effect = batched
            .get(index)
            .map_or(&batched[batched.len() - 1].0.id, |(e, _)| &e.id);
        return Err(map_drive_err(effect, error));
    }

    for (effect, scoped) in &batched {
        let Effect::Setting(setting) = &effect.kind else {
            continue;
        };
        let cx = context::route(effect, ctx.tweak, ctx.corpus);
        let actual = ctx
            .deps
            .kinds
            .read(setting, &cx)
            .map_err(|e| map_drive_err(&effect.id, e))?;
        if actual != scoped.value {
            return Err(EngineError::VerifyMismatch {
                effect: effect.id.clone(),
                expected: scoped.value.clone(),
                actual,
            });
        }
        state.effect_results.push(EffectResult {
            effect: effect.id.clone(),
            kind: EffectResultKind::Driven {
                desired: scoped.value.clone(),
            },
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::{ActionRunner, ProbeCache, ProbeSource};
    use crate::tweaks::kinds::EffectKind;
    use crate::tweaks::model::{
        CategoryDef, Hive, Level, Probe, RegAddr, RegType, RiskLevel, ScopedValue, Script, Shell,
        SvcAddr, TypedRegValue, WindowsScope,
    };
    use crate::tweaks::shared_claims::ClaimsStore;
    use crate::tweaks::snapshot::SnapshotStore;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    // --- shared op log -------------------------------------------------------------------------

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Op {
        Read(String),
        Drive(String),
        Probe(String),
        RunApply(String),
        RunUndo(String),
    }

    type Log = Arc<Mutex<Vec<Op>>>;

    fn setting_key(s: &Setting) -> String {
        match s {
            Setting::Service(addr) => addr.name.clone(),
            other => panic!("apply.rs tests only fixture Service settings, got {other:?}"),
        }
    }

    fn action_key(a: &ActionDef) -> String {
        match a {
            ActionDef::Script { apply, .. } => apply.0.clone(),
            ActionDef::DeleteTree { key, .. } => key.path.clone(),
        }
    }

    // --- MockKind: in-memory Setting -> Value, with per-key scripted behavior ------------------

    #[derive(Clone)]
    enum DrivePlan {
        /// Drive returns Ok but does NOT update the tracked live value -- forces the next
        /// (honest) read-back to reveal a mismatch, without needing a lying read.
        NoOp,
        Err,
        ResourceMissing,
        /// Drives the value, then reports an unknown elevated outcome; later drives succeed.
        OutcomeUnknownOnce,
    }

    #[derive(Default)]
    struct MockKind {
        log: Log,
        live: Mutex<HashMap<String, Value>>,
        read_call_count: Mutex<HashMap<String, u32>>,
        /// Fails every read from this 1-indexed call number onward -- lets a test model "detect's
        /// own read (call 1) succeeds, but the resource becomes unreadable by the time Step 1's
        /// independent capture read (call 2) runs," distinct from "detect itself already saw
        /// Unknown" (fail from call 1).
        fail_read_from_call: Mutex<HashMap<String, u32>>,
        drive_plan: Mutex<HashMap<String, DrivePlan>>,
        assert_before_drive: Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
        /// The size of each `drive_batch` call, so a test can count children instead of drives.
        batches: Mutex<Vec<usize>>,
        /// The `ExecCx::level()` each drive received, so a test can prove which level a reversal
        /// actually ran at.
        levels: Mutex<Vec<(String, Level)>>,
    }

    impl MockKind {
        fn new(log: Log) -> Self {
            Self {
                log,
                ..Default::default()
            }
        }
        fn seed(&self, name: &str, v: Value) -> &Self {
            self.live.lock().unwrap().insert(name.into(), v);
            self
        }
        fn fail_read_from_call(&self, name: &str, call_num: u32) -> &Self {
            self.fail_read_from_call
                .lock()
                .unwrap()
                .insert(name.into(), call_num);
            self
        }
        fn drive_plan(&self, name: &str, plan: DrivePlan) -> &Self {
            self.drive_plan.lock().unwrap().insert(name.into(), plan);
            self
        }
        fn live_value(&self, name: &str) -> Value {
            self.live
                .lock()
                .unwrap()
                .get(name)
                .cloned()
                .unwrap_or(Value::Absent)
        }
        fn on_drive(&self, f: impl Fn() + Send + Sync + 'static) {
            *self.assert_before_drive.lock().unwrap() = Some(Box::new(f));
        }
        fn batches(&self) -> Vec<usize> {
            self.batches.lock().unwrap().clone()
        }
        /// Every drive level recorded for `name`, in order: a test pins both THAT a drive happened
        /// and which level ran it, so a vanished reversal fails as loudly as a mis-routed one.
        fn drive_levels(&self, name: &str) -> Vec<Level> {
            self.levels
                .lock()
                .unwrap()
                .iter()
                .filter(|(k, _)| k == name)
                .map(|(_, l)| *l)
                .collect()
        }
    }

    impl EffectKind for MockKind {
        fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, KindError> {
            let key = setting_key(s);
            self.log.lock().unwrap().push(Op::Read(key.clone()));
            let call_num = {
                let mut counts = self.read_call_count.lock().unwrap();
                let n = counts.entry(key.clone()).or_insert(0);
                *n += 1;
                *n
            };
            if let Some(&fail_from) = self.fail_read_from_call.lock().unwrap().get(&key) {
                if call_num >= fail_from {
                    return Err(KindError::Backend("mock read failure".into()));
                }
            }
            Ok(self.live_value(&key))
        }

        fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), KindError> {
            let key = setting_key(s);
            if let Some(f) = &*self.assert_before_drive.lock().unwrap() {
                f();
            }
            self.log.lock().unwrap().push(Op::Drive(key.clone()));
            self.levels.lock().unwrap().push((key.clone(), cx.level()));
            let plan = self.drive_plan.lock().unwrap().get(&key).cloned();
            match plan {
                Some(DrivePlan::Err) => Err(KindError::Backend("mock drive failure".into())),
                Some(DrivePlan::OutcomeUnknownOnce) => {
                    self.drive_plan.lock().unwrap().remove(&key);
                    self.live.lock().unwrap().insert(key, target.clone());
                    Err(KindError::ElevatedOutcomeUnknown(
                        Level::Ti,
                        "mock timeout".into(),
                    ))
                }
                Some(DrivePlan::ResourceMissing) => {
                    Err(KindError::ResourceMissing("mock resource missing".into()))
                }
                Some(DrivePlan::NoOp) => Ok(()), // deliberately does not update `live`
                None => {
                    self.live.lock().unwrap().insert(key, target.clone());
                    Ok(())
                }
            }
        }

        /// Records the run size, then behaves exactly like the trait default, so a batched call
        /// stays indistinguishable from the per-effect one to every other assertion.
        fn drive_batch(
            &self,
            items: &[(&Setting, &Value)],
            cx: &ExecCx,
        ) -> Result<(), BatchFailure> {
            self.batches.lock().unwrap().push(items.len());
            for (index, (setting, target)) in items.iter().enumerate() {
                self.drive(setting, target, cx)
                    .map_err(|error| BatchFailure {
                        index,
                        error,
                        completed: index,
                    })?;
            }
            Ok(())
        }
    }

    // --- MockProbes / MockActions, sharing one presence map -------------------------------------

    type Presence = Arc<Mutex<HashMap<String, bool>>>;

    #[derive(Default)]
    struct MockProbes {
        log: Log,
        presence: Presence,
        /// The `ExecCx::level()` each probe received, so a test can prove a read never escalated.
        levels: Mutex<Vec<(String, Level)>>,
    }
    impl MockProbes {
        fn new(log: Log, presence: Presence) -> Self {
            Self {
                log,
                presence,
                ..Default::default()
            }
        }
        /// The level of the MOST RECENT probe of `key` -- on a rollback, the reversal's verify.
        fn last_level_of(&self, key: &str) -> Option<Level> {
            self.levels
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, l)| *l)
        }
    }
    impl ProbeSource for MockProbes {
        fn probe(&self, action: &ActionDef, cx: &ExecCx) -> Result<bool, KindError> {
            let key = action_key(action);
            self.log.lock().unwrap().push(Op::Probe(key.clone()));
            self.levels.lock().unwrap().push((key.clone(), cx.level()));
            Ok(*self.presence.lock().unwrap().get(&key).unwrap_or(&false))
        }
    }

    #[derive(Default)]
    struct MockActions {
        log: Log,
        presence: Presence,
        /// The `ExecCx::level()` each call received, keyed `"<action>:apply"` / `"<action>:undo"`.
        levels: Mutex<Vec<(String, Level)>>,
        fail_apply: Mutex<HashSet<String>>,
        fail_undo: Mutex<HashSet<String>>,
        /// Simulates a buggy/racy `undo` that exits 0 (`Ok`) without actually reverting the
        /// resource -- presence is deliberately left untouched, so a probe taken right after
        /// still reads present. Models exactly the gap invariant 19 guards against.
        lie_on_undo: Mutex<HashSet<String>>,
    }
    impl MockActions {
        fn new(log: Log, presence: Presence) -> Self {
            Self {
                log,
                presence,
                ..Default::default()
            }
        }
        fn fail_apply(&self, key: &str) -> &Self {
            self.fail_apply.lock().unwrap().insert(key.into());
            self
        }
        fn lie_on_undo(&self, key: &str) -> &Self {
            self.lie_on_undo.lock().unwrap().insert(key.into());
            self
        }
        fn level_of(&self, key: &str) -> Option<Level> {
            self.levels
                .lock()
                .unwrap()
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, l)| *l)
        }
    }
    impl ActionRunner for MockActions {
        fn apply(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), KindError> {
            let key = action_key(action);
            self.log.lock().unwrap().push(Op::RunApply(key.clone()));
            self.levels
                .lock()
                .unwrap()
                .push((format!("{key}:apply"), cx.level()));
            if self.fail_apply.lock().unwrap().contains(&key) {
                return Err(KindError::ActionFailed(1));
            }
            self.presence.lock().unwrap().insert(key, true);
            Ok(())
        }
        fn undo(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), KindError> {
            let key = action_key(action);
            self.log.lock().unwrap().push(Op::RunUndo(key.clone()));
            self.levels
                .lock()
                .unwrap()
                .push((format!("{key}:undo"), cx.level()));
            if self.fail_undo.lock().unwrap().contains(&key) {
                return Err(KindError::ActionFailed(1));
            }
            if !self.lie_on_undo.lock().unwrap().contains(&key) {
                self.presence.lock().unwrap().insert(key, false);
            }
            Ok(())
        }
    }

    // --- fixture builders (mirrors engine::detect's test fixtures) -----------------------------

    fn svc_effect(id: &str, optional: bool) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Setting(Setting::Service(SvcAddr {
                name: id.to_string(),
            })),
            elevation: None,
            optional,
            if_missing: None,
            windows: None,
        }
    }

    /// A service effect that escalates to TrustedInstaller, the shipped corpus's shape: an `admin`
    /// floor with `ti` steps interleaved among plain admin ones.
    fn ti_svc_effect(id: &str) -> EffectDef {
        EffectDef {
            elevation: Some(Level::Ti),
            ..svc_effect(id, false)
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

    fn action_effect(id: &str, undo: bool, probe: bool) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Action(ActionDef::Script {
                apply: Script(format!("{id}_apply")),
                undo: undo.then(|| Script(format!("{id}_undo"))),
                probe: probe.then(|| Probe::Script(Script(format!("{id}_probe")))),
                ephemeral: false,
                shell: Shell::PowerShell,
            }),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn ephemeral_effect(id: &str) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Action(ActionDef::Script {
                apply: Script(format!("{id}_apply")),
                undo: None,
                probe: None,
                ephemeral: true,
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
            windows: None,
        }
    }

    fn corpus(tweaks: Vec<Tweak>, shared: Vec<SharedDef>) -> Corpus {
        Corpus {
            categories: Vec::<CategoryDef>::new(),
            tweaks,
            shared,
        }
    }

    /// Owns everything a test needs so `Deps` (all borrows) can outlive the `apply` call.
    struct Harness {
        kind: MockKind,
        probes: MockProbes,
        actions: MockActions,
        claims: ClaimsStore,
        snapshots: SnapshotStore,
        cache: ProbeCache,
        _tmp: tempfile::TempDir,
    }

    impl Harness {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let log: Log = Arc::new(Mutex::new(Vec::new()));
            let presence: Presence = Arc::new(Mutex::new(HashMap::new()));
            Self {
                kind: MockKind::new(log.clone()),
                probes: MockProbes::new(log.clone(), presence.clone()),
                actions: MockActions::new(log, presence),
                claims: ClaimsStore::open(tmp.path().to_path_buf(), Some("test-guid".into())),
                snapshots: SnapshotStore::open(tmp.path().to_path_buf()),
                cache: ProbeCache::new(),
                _tmp: tmp,
            }
        }

        fn deps(&self) -> Deps<'_> {
            Deps {
                kinds: &self.kind,
                probes: &self.probes,
                actions: &self.actions,
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

        fn log(&self) -> Vec<Op> {
            self.actions.log.lock().unwrap().clone()
        }

        /// Discards ops recorded so far (e.g. from test setup that pre-seeds claims state through
        /// the same mock kind) so later assertions only see what `apply` itself did.
        fn clear_log(&self) {
            self.actions.log.lock().unwrap().clear();
        }

        fn set_present(&self, key: &str, present: bool) {
            self.probes
                .presence
                .lock()
                .unwrap()
                .insert(key.to_string(), present);
        }
    }

    /// Blocks on `apply` without pulling in a full async-test harness for every scenario -- none
    /// of these tests need real concurrency, only the lock's acquire/release to work at all
    /// (`lifecycle`'s own suite proves the concurrency property).
    fn run_apply(
        tweak: &Tweak,
        corpus: &Corpus,
        target: &OptLabel,
        deps: &Deps,
    ) -> Result<ApplyOutcome, EngineError> {
        futures_block_on(apply(tweak, corpus, target, deps))
    }

    /// Minimal single-poll executor: `apply`'s only await point is an uncontended lock acquire,
    /// which resolves on first poll -- no real reactor/timer machinery is needed to drive it.
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

    // --- the 15 named scenarios + a few pinning extras ------------------------------------------

    #[test]
    fn already_active_is_verified_noop_no_snapshot() {
        let h = Harness::new();
        h.kind.seed(
            "svc",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![svc_effect("svc", false)],
            vec![opt(
                "A",
                vec![(
                    "svc",
                    set(Value::Startup(crate::tweaks::model::StartupType::Manual)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let outcome = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("no-op succeeds");
        assert!(outcome.effects.is_empty());
        assert_eq!(
            outcome.status.state,
            TweakState::Active(OptLabel("A".into()))
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_none(),
            "a verified no-op must push no snapshot"
        );
        assert!(
            h.log().iter().all(|op| !matches!(op, Op::Drive(_))),
            "a verified no-op must drive nothing"
        );
    }

    #[test]
    fn capture_before_mutation() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.seed(
            "s2",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), svc_effect("s2", false)],
            vec![opt(
                "A",
                vec![
                    (
                        "s1",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    (
                        "s2",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("apply succeeds");

        let log = h.log();
        let first_drive = log
            .iter()
            .position(|op| matches!(op, Op::Drive(_)))
            .expect("at least one drive must have happened");
        let reads_before: Vec<&Op> = log[..first_drive]
            .iter()
            .filter(|op| matches!(op, Op::Read(_)))
            .collect();
        assert!(
            reads_before.contains(&&Op::Read("s1".into()))
                && reads_before.contains(&&Op::Read("s2".into())),
            "both s1's and s2's capture reads must precede the first drive: {log:?}"
        );
    }

    #[test]
    fn unreadable_capture_aborts_untouched() {
        let h = Harness::new();
        h.kind.seed(
            "svc",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        // Call 1 (detect's own read) succeeds; call 2 (Step 1's independent capture read) fails --
        // models the resource becoming unreadable between detect and capture, distinct from a
        // surface that was already Unknown at Step 0.
        h.kind.fail_read_from_call("svc", 2);
        let t = tweak(
            "demo",
            vec![svc_effect("svc", false)],
            vec![opt(
                "A",
                vec![(
                    "svc",
                    set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err =
            run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("capture read fails");
        assert!(matches!(err, EngineError::CaptureFailed { .. }));
        assert!(
            h.log().iter().all(|op| !matches!(op, Op::Drive(_))),
            "an aborted capture must never drive"
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_none(),
            "an aborted capture must push no snapshot"
        );
    }

    #[test]
    fn entry_persisted_with_intended_actions_before_first_drive() {
        let h = Harness::new();
        h.kind.seed(
            "svc",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![svc_effect("svc", false), action_effect("a1", false, false)],
            vec![opt(
                "A",
                vec![
                    (
                        "svc",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("a1", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let store = h.snapshots.clone();
        let corpus_clone = c.clone();
        h.kind.on_drive(move || {
            let head = store
                .head("demo", &corpus_clone, Some("test-guid"), 19045)
                .unwrap();
            assert!(
                head.is_some(),
                "the WAL entry must exist on disk before the first drive"
            );
            let entry = head.unwrap();
            assert_eq!(entry.journal.len(), 1);
            assert!(entry.journal[0].intended);
            assert!(!entry.journal[0].completed);
        });

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("apply succeeds");
    }

    #[test]
    fn completion_marked_after_each_action() {
        let h = Harness::new();
        // An anchor Setting whose live value disagrees with what "A" authors: without it, a
        // surface made only of probe-less (non-detectable) Actions has nothing for `detect` to
        // disagree on, so the lone option would vacuously read as already Active -- the no-op
        // path -- before this test ever reaches Step 3.
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        // act1 is one-way (no undo) so a forced rollback (act2 fails) cannot fully verify --
        // the entry stays on disk, letting us inspect the journal afterward.
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act1", false, false),
                action_effect("act2", false, false),
            ],
            vec![opt(
                "A",
                vec![
                    (
                        "anchor",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("act1", OptValue::Run(None)),
                    ("act2", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.actions.fail_apply("act2_apply");

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("act2 fails");
        assert!(matches!(err, EngineError::RollbackReport { .. }));

        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("incomplete rollback keeps the entry");
        let row = |id: &str| {
            entry
                .journal
                .iter()
                .find(|r| r.action_id.0 == id)
                .unwrap_or_else(|| panic!("journal missing row for {id}"))
        };
        assert!(
            row("act1").completed,
            "act1 succeeded and must have been marked completed immediately, independent of act2's later failure"
        );
        assert!(
            !row("act2").completed,
            "act2 never succeeded -- never marked"
        );
    }

    #[test]
    fn declaration_order_preserved() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![
                svc_effect("s1", false),
                shared_effect("sh_eff", "sh_a"),
                action_effect("a1", false, false),
            ],
            vec![opt(
                "A",
                vec![
                    (
                        "s1",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("sh_eff", OptValue::Claim(None)),
                    ("a1", OptValue::Run(None)),
                ],
            )],
        );
        let shared = SharedDef {
            id: SharedId("sh_a".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_a_addr".into(),
            }),
            value: Value::Startup(crate::tweaks::model::StartupType::Disabled),
        };
        h.kind.seed(
            "sh_a_addr",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let c = corpus(vec![t.clone()], vec![shared]);

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("apply succeeds");

        let log = h.log();
        let pos_s1 = log
            .iter()
            .position(|op| op == &Op::Drive("s1".into()))
            .unwrap();
        let pos_shared = log
            .iter()
            .position(|op| op == &Op::Drive("sh_a_addr".into()))
            .unwrap();
        let pos_action = log
            .iter()
            .position(|op| op == &Op::RunApply("a1_apply".into()))
            .unwrap();
        assert!(
            pos_s1 < pos_shared && pos_shared < pos_action,
            "effects must drive in declaration order: {log:?}"
        );
    }

    #[test]
    fn verify_mismatch_rolls_back() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        // Drive silently no-ops: the honest read-back afterward reveals the mismatch, so the
        // ORIGINAL failure is a genuine `VerifyMismatch`, not a drive `Err`.
        h.kind.drive_plan("s1", DrivePlan::NoOp);
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![(
                    "s1",
                    set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err =
            run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("verify mismatches");
        let EngineError::RollbackReport {
            original,
            rollback_failures,
            ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(matches!(*original, EngineError::VerifyMismatch { .. }));
        // The drive-back-to-captured-value during rollback also no-ops, but the captured value
        // equals the never-actually-changed live value, so it verifies -- a fully clean rollback.
        assert!(
            rollback_failures.is_empty(),
            "this rollback fully verifies: {rollback_failures:?}"
        );
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(crate::tweaks::model::StartupType::Manual)
        );
    }

    /// Fix 2 regression (a CRITICAL Fix 2 itself exposed by making ephemerals run): an ephemeral
    /// action running earlier in declaration order must never make a later, unrelated failure
    /// spuriously un-rollback-able. Mirrors `verify_mismatch_rolls_back`'s `DrivePlan::NoOp` trick
    /// (the ORIGINAL failure is a genuine `VerifyMismatch`; the rollback's drive-back to the
    /// never-actually-changed captured value trivially re-verifies), with a declared-earlier
    /// ephemeral action added to the surface.
    #[test]
    fn apply_with_ephemeral_then_failure_rolls_back_cleanly() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.drive_plan("s1", DrivePlan::NoOp);
        let t = tweak(
            "demo",
            vec![ephemeral_effect("eph"), svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![
                    ("eph", OptValue::Run(None)),
                    (
                        "s1",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err =
            run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("s1 verify mismatches");
        let EngineError::RollbackReport {
            original,
            rollback_failures,
            ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(matches!(*original, EngineError::VerifyMismatch { .. }));
        assert!(
            rollback_failures.is_empty(),
            "the ephemeral that already ran must never be reported un-undoable -- this rollback \
             fully verifies (s1's captured value was never actually changed): {rollback_failures:?}"
        );
        assert!(
            h.log().contains(&Op::RunApply("eph_apply".into())),
            "the ephemeral must still have run before the later effect failed"
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_none(),
            "a verified rollback must consume the entry -- an ephemeral having run earlier must \
             never spuriously strand it in Needs Attention"
        );
    }

    #[test]
    fn rollback_failure_is_needs_attention_snapshot_kept() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.drive_plan("s1", DrivePlan::Err); // fails both forward AND during rollback
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![(
                    "s1",
                    set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("drive fails");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(
            !rollback_failures.is_empty(),
            "the same persistent drive failure must also break the rollback restore"
        );
        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap();
        assert!(
            entry.is_some(),
            "an incomplete rollback must keep the snapshot (ADR-0001/0002)"
        );
    }

    /// Regression for the reviewed CRITICAL: a probeable, undo-carrying action's `undo` exits 0
    /// (a lying/buggy script -- never surfaced as a non-zero exit) but does NOT actually revert
    /// the resource. Every OTHER effect in this scenario restores cleanly, so under the buggy
    /// code (exit-code-only reversal verification) `rollback_failures` would be empty and
    /// `snapshots.consume` would delete the only return-point while the machine still carries the
    /// un-reverted action -- exactly the consume-on-uncertain-rollback ADR-0002 forbids.
    #[test]
    fn rollback_probe_verifies_action_reversal_never_consumes_on_a_lying_undo() {
        let h = Harness::new();
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        // act1 runs forward and succeeds; its `undo` (during rollback) lies: Ok(()), but the
        // probe still reads present afterward.
        h.actions.lie_on_undo("act1_apply");
        // act2 is what actually fails, triggering rollback; act1 never fails forward.
        h.actions.fail_apply("act2_apply");
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act1", true, true), // undo-carrying, probeable
                action_effect("act2", false, false),
            ],
            vec![opt(
                "A",
                vec![
                    (
                        "anchor",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("act1", OptValue::Run(None)),
                    ("act2", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("act2 fails");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(
            !rollback_failures.is_empty(),
            "act1's undo silently failed to revert -- the probe re-check must catch it"
        );
        assert!(
            rollback_failures
                .iter()
                .any(|e| matches!(e, EngineError::ActionVerifyMismatch { effect, .. } if effect.0 == "act1")),
            "expected an ActionVerifyMismatch naming act1, got {rollback_failures:?}"
        );
        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap();
        assert!(
            entry.is_some(),
            "an un-reverted action must block consume -- the entry must stay on disk (ADR-0001/0002)"
        );
    }

    #[test]
    fn verified_rollback_consumes_entry() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("a1", false, false)],
            vec![opt(
                "A",
                vec![
                    (
                        "s1",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("a1", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.actions.fail_apply("a1_apply"); // s1 drives fine; a1 fails -> rollback

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("a1 fails");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(
            rollback_failures.is_empty(),
            "s1's restore has no failure configured, so this rollback fully verifies: {rollback_failures:?}"
        );
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(crate::tweaks::model::StartupType::Manual)
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_none(),
            "a verified rollback must consume the entry"
        );
    }

    /// The undo side of grouped execution: a rollback puts each run of consecutive elevated effects
    /// through ONE child. Rollback walks the captured values in effect-id order, not declaration
    /// order, and grouping must not change that: `a5` sorts ahead of every `t*`, making one run.
    #[test]
    fn a_rollback_puts_each_run_of_elevated_effects_through_one_child() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        for id in ["t1", "t2", "t3", "t4", "a5"] {
            h.kind.seed(id, Value::Startup(StartupType::Manual));
        }
        // Every Setting drives forward; the action declared last is what rolls the tweak back.
        h.actions.fail_apply("z_act_apply");
        let mut t = tweak(
            "demo",
            vec![
                ti_svc_effect("t2"),
                ti_svc_effect("t1"),
                svc_effect("a5", false),
                ti_svc_effect("t3"),
                ti_svc_effect("t4"),
                action_effect("z_act", false, false),
            ],
            vec![opt(
                "A",
                vec![
                    ("t2", set(Value::Startup(StartupType::Disabled))),
                    ("t1", set(Value::Startup(StartupType::Disabled))),
                    ("a5", set(Value::Startup(StartupType::Disabled))),
                    ("t3", set(Value::Startup(StartupType::Disabled))),
                    ("t4", set(Value::Startup(StartupType::Disabled))),
                    ("z_act", OptValue::Run(None)),
                ],
            )],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("z_act fails");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(
            rollback_failures.is_empty(),
            "nothing is configured to fail the restore: {rollback_failures:?}"
        );

        assert_eq!(
            h.kind.batches(),
            vec![2, 2, 4],
            "two runs forward, one run back: four elevated effects restored through one child, \
             not four"
        );
        let drives: Vec<String> = h
            .log()
            .into_iter()
            .filter_map(|op| match op {
                Op::Drive(key) => Some(key),
                _ => None,
            })
            .collect();
        assert_eq!(
            drives,
            ["t2", "t1", "a5", "t3", "t4", "a5", "t1", "t2", "t3", "t4"],
            "grouping moves nothing: forward stays declaration order and the rollback stays \
             effect-id order"
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_none(),
            "a verified rollback still consumes the entry"
        );
    }

    /// A failure inside a batched rollback is charged to the effect that produced it, and what the
    /// child never reached is still attempted: an undo never stops at its first failure (ADR-0001),
    /// and the entry stays (ADR-0002).
    #[test]
    fn a_failure_inside_a_batched_rollback_names_its_effect_and_still_attempts_the_rest() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        for id in ["t1", "t2", "t3", "t4", "a5"] {
            h.kind.seed(id, Value::Startup(StartupType::Manual));
        }
        h.kind.drive_plan("t3", DrivePlan::Err); // fails forward AND inside the rollback's batch
        let mut t = tweak(
            "demo",
            vec![
                ti_svc_effect("t2"),
                ti_svc_effect("t1"),
                svc_effect("a5", false),
                ti_svc_effect("t3"),
                ti_svc_effect("t4"),
            ],
            vec![opt(
                "A",
                vec![
                    ("t2", set(Value::Startup(StartupType::Disabled))),
                    ("t1", set(Value::Startup(StartupType::Disabled))),
                    ("a5", set(Value::Startup(StartupType::Disabled))),
                    ("t3", set(Value::Startup(StartupType::Disabled))),
                    ("t4", set(Value::Startup(StartupType::Disabled))),
                ],
            )],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("t3 fails");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        let [only] = &rollback_failures[..] else {
            panic!("one failing effect, no more and no fewer: {rollback_failures:?}");
        };
        assert!(
            matches!(only, EngineError::DriveFailed { effect, .. } if effect.0 == "t3"),
            "the batch's failure must name the effect that produced it: {only:?}"
        );

        let drives: Vec<String> = h
            .log()
            .into_iter()
            .filter_map(|op| match op {
                Op::Drive(key) => Some(key),
                _ => None,
            })
            .collect();
        let count = |id: &str| drives.iter().filter(|k| k.as_str() == id).count();
        assert_eq!(
            count("t4"),
            1,
            "t4 never ran forward -- the batch stopped at t3 -- and must still be driven back: \
             {drives:?}"
        );
        assert_eq!(
            count("t1"),
            2,
            "once forward, once back: an item the child proved it drove is verified, never driven \
             a second time: {drives:?}"
        );
        assert_eq!(h.kind.batches(), vec![2, 2, 4]);
        assert_eq!(h.kind.live_value("t4"), Value::Startup(StartupType::Manual));
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_some(),
            "an incomplete rollback keeps the snapshot (ADR-0001/0002)"
        );
    }

    /// An unknown outcome from inside a batched undo classifies exactly as it does on the apply
    /// side -- the child ran, so the machine may have changed -- never as a could-not-acquire that
    /// claims nothing happened.
    #[test]
    fn an_unknown_outcome_inside_a_batched_rollback_keeps_the_entry_with_that_mark() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        for id in ["t1", "t2", "t9"] {
            h.kind.seed(id, Value::Startup(StartupType::Manual));
        }
        // The failing action is declared first, so no Setting drives forward and t9's single
        // unknown outcome lands inside the rollback's batch.
        h.actions.fail_apply("act_apply");
        h.kind.drive_plan("t9", DrivePlan::OutcomeUnknownOnce);
        let mut t = tweak(
            "demo",
            vec![
                action_effect("act", false, false),
                ti_svc_effect("t1"),
                ti_svc_effect("t2"),
                ti_svc_effect("t9"),
            ],
            vec![opt(
                "A",
                vec![
                    ("act", OptValue::Run(None)),
                    ("t1", set(Value::Startup(StartupType::Disabled))),
                    ("t2", set(Value::Startup(StartupType::Disabled))),
                    ("t9", set(Value::Startup(StartupType::Disabled))),
                ],
            )],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("act fails");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        let [only] = &rollback_failures[..] else {
            panic!("one failing effect, no more and no fewer: {rollback_failures:?}");
        };
        assert!(
            matches!(
                only,
                EngineError::DriveFailed {
                    effect,
                    source: KindError::ElevatedOutcomeUnknown(Level::Ti, _),
                } if effect.0 == "t9"
            ),
            "an unknown outcome must survive the batch as one: {only:?}"
        );
        assert_eq!(
            attention_item(Phase::Apply, only).kind,
            AttentionKind::OutcomeUnknown
        );
        assert_eq!(
            h.kind.batches(),
            vec![3],
            "nothing drove forward; the rollback's three elevated effects share one child"
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_some(),
            "an unknown outcome must keep the entry"
        );
    }

    /// The rollback verifies, yet what the child ran cannot be proven: the entry stays (ADR-0002).
    #[test]
    fn an_unknown_elevated_outcome_keeps_the_entry_after_a_verified_rollback() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        h.kind
            .seed("s1", Value::Startup(StartupType::Manual))
            .drive_plan("s1", DrivePlan::OutcomeUnknownOnce);
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps())
            .expect_err("s1's outcome is unknown");
        let EngineError::RollbackReport {
            rollback_failures, ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(
            rollback_failures.is_empty(),
            "the restore verifies: {rollback_failures:?}"
        );
        assert_eq!(h.kind.live_value("s1"), Value::Startup(StartupType::Manual));
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_some(),
            "an unknown outcome must keep the entry"
        );
    }

    #[test]
    fn an_unknown_elevated_outcome_inside_a_shared_claim_keeps_the_entry() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        h.kind
            .seed("sh_addr", Value::Startup(StartupType::Manual))
            .drive_plan("sh_addr", DrivePlan::OutcomeUnknownOnce);
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(StartupType::Disabled),
        };
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), shared_effect("sh_eff", "sh")],
            vec![opt(
                "A",
                vec![
                    ("s1", set(Value::Startup(StartupType::Disabled))),
                    ("sh_eff", OptValue::Claim(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![shared]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps())
            .expect_err("the claim's outcome is unknown");
        let EngineError::RollbackReport {
            original,
            rollback_failures,
            ..
        } = err
        else {
            panic!("expected RollbackReport");
        };
        assert!(
            matches!(*original, EngineError::Claim { .. }),
            "{original:?}"
        );
        assert!(
            rollback_failures.is_empty(),
            "the restore verifies: {rollback_failures:?}"
        );
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_some(),
            "an unknown outcome inside a shared claim must keep the entry"
        );
    }

    /// One failing tweak whose single Setting reaches `plan`, at `Manual`, with option "A" driving
    /// it to `Disabled`.
    fn attention_fixture(h: &Harness, plan: DrivePlan) -> (Tweak, Corpus) {
        use crate::tweaks::model::StartupType;
        h.kind
            .seed("s1", Value::Startup(StartupType::Manual))
            .drive_plan("s1", plan);
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        (t, c)
    }

    /// ADR-0005 amendment: a kept snapshot surfaces as Needs Attention on a rescan and after a
    /// restart (a fresh store over the same directory).
    #[test]
    fn an_unknown_outcome_is_needs_attention_across_rescan_and_restart() {
        use crate::tweaks::snapshot::AttentionReason;
        let h = Harness::new();
        let (t, c) = attention_fixture(&h, DrivePlan::OutcomeUnknownOnce);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps())
            .expect_err("s1's outcome is unknown");
        let message = err.to_string();
        assert!(!message.contains("fully verified"), "{message}");
        assert!(message.contains("snapshot was kept"), "{message}");

        let attention = detect::detect(&t, &c, &h.deps())
            .attention
            .expect("a rescan shows Needs Attention");
        assert_eq!(attention.reason, AttentionReason::ApplyFailed);
        assert!(
            attention
                .items
                .iter()
                .any(|i| i.effect.as_ref() == Some(&EffectId("s1".into()))),
            "{attention:?}"
        );

        let reopened = SnapshotStore::open(h._tmp.path().to_path_buf());
        let after_restart = Deps {
            snapshots: &reopened,
            ..h.deps()
        };
        assert_eq!(
            detect::detect(&t, &c, &after_restart).attention,
            Some(attention)
        );
    }

    #[test]
    fn an_incomplete_rollback_records_needs_attention_and_keeps_its_entry() {
        let h = Harness::new();
        let (t, c) = attention_fixture(&h, DrivePlan::Err);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("drive fails");
        assert!(err.to_string().contains("unrecoverable"), "{err}");
        assert!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .is_some(),
            "an incomplete rollback keeps the entry"
        );
        let attention = h
            .snapshots
            .attention("demo", Some("test-guid"))
            .unwrap()
            .expect("the failure is recorded");
        assert!(!attention.items.is_empty());
    }

    /// The record is not the entry's, so an option label the corpus no longer defines cannot hide
    /// an unresolved failure (ADR-0002's amendment keeps the invalid entry, surfaced, undeleted).
    #[test]
    fn needs_attention_survives_its_entry_turning_invalid() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        h.kind
            .seed("s1", Value::Startup(StartupType::Disabled))
            .drive_plan("s1", DrivePlan::OutcomeUnknownOnce);
        let options = |first: &str| {
            vec![
                opt(
                    first,
                    vec![("s1", set(Value::Startup(StartupType::Disabled)))],
                ),
                opt("B", vec![("s1", set(Value::Startup(StartupType::Manual)))]),
            ]
        };
        let t = tweak("demo", vec![svc_effect("s1", false)], options("A"));
        let c = corpus(vec![t.clone()], vec![]);

        run_apply(&t, &c, &OptLabel("B".into()), &h.deps()).expect_err("s1's outcome is unknown");
        assert!(detect::detect(&t, &c, &h.deps()).attention.is_some());

        // "A" is renamed, so the entry that captured it is now a dangling reference.
        let renamed = tweak("demo", vec![svc_effect("s1", false)], options("A2"));
        let renamed_corpus = corpus(vec![renamed.clone()], vec![]);
        let status = detect::detect(&renamed, &renamed_corpus, &h.deps());
        assert!(!status.has_history, "the entry no longer restores");
        assert!(
            status.attention.is_some(),
            "but the tweak still needs the user"
        );
    }

    #[test]
    fn a_verified_apply_clears_the_record() {
        let h = Harness::new();
        let (t, c) = attention_fixture(&h, DrivePlan::OutcomeUnknownOnce);
        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("s1's outcome is unknown");

        let outcome = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("the retry works");
        assert_eq!(outcome.status.attention, None);
        assert_eq!(detect::detect(&t, &c, &h.deps()).attention, None);
        assert_eq!(
            h.snapshots.attention("demo", Some("test-guid")).unwrap(),
            None
        );
    }

    /// The recurrence the record alone could never stop: journal residue on a `Values` entry, which
    /// no push ever dedups away, outlives the clear and re-marks a tweak the user resolved at every
    /// later launch. The re-apply drove `act1`, so it resolves `act1`'s row and only that row.
    #[test]
    fn a_verified_apply_resolves_the_residue_a_crash_left_on_a_superseded_entry() {
        use crate::tweaks::engine::lifecycle::record_crash_residue;
        use crate::tweaks::model::StartupType;
        use crate::tweaks::snapshot::AttentionReason;
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("act1", true, false)],
            vec![opt(
                "A",
                vec![
                    ("s1", set(Value::Startup(StartupType::Disabled))),
                    ("act1", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        // The crash: an action row planned and never confirmed, on an entry nothing will displace.
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: vec![JournalRow {
                        action_id: EffectId("act1".into()),
                        intended: true,
                        completed: false,
                        resolved: false,
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        record_crash_residue(&h.snapshots, "demo", Some("test-guid"));
        assert_eq!(
            h.snapshots
                .attention("demo", Some("test-guid"))
                .unwrap()
                .map(|a| a.reason),
            Some(AttentionReason::CrashResidue)
        );

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("the re-apply verifies");
        assert_eq!(
            h.snapshots.attention("demo", Some("test-guid")).unwrap(),
            None
        );

        for _ in 0..2 {
            record_crash_residue(&h.snapshots, "demo", Some("test-guid"));
            assert_eq!(
                h.snapshots.attention("demo", Some("test-guid")).unwrap(),
                None,
                "the crash entry's row must not re-mark a tweak the re-apply resolved"
            );
        }
        assert!(
            h.snapshots
                .list("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .len()
                >= 2,
            "and nothing was deleted to achieve it (ADR-0002)"
        );
    }

    /// The record is for uncertainty, not for every failure: a drive that fails with a known
    /// outcome and a rollback that fully verifies consumes its entry and records nothing.
    #[test]
    fn a_failed_apply_whose_rollback_verifies_records_nothing() {
        use crate::tweaks::model::StartupType;
        let h = Harness::new();
        h.kind.seed("anchor", Value::Startup(StartupType::Manual));
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act1", true, true),
            ],
            vec![opt(
                "A",
                vec![
                    ("anchor", set(Value::Startup(StartupType::Disabled))),
                    ("act1", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.actions.fail_apply("act1_apply");

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("act1's apply fails");

        assert_eq!(
            h.snapshots.attention("demo", Some("test-guid")).unwrap(),
            None,
            "a verified rollback with a known outcome leaves nothing to attend to"
        );
        assert_eq!(
            h.snapshots
                .head("demo", &c, Some("test-guid"), 19045)
                .unwrap(),
            None,
            "and its entry is consumed, not kept"
        );
        assert_eq!(detect::detect(&t, &c, &h.deps()).attention, None);
    }

    /// A record that cannot be written is named as itself: a rollback that restored everything must
    /// never be described as leaving items unrecoverable.
    #[test]
    fn a_failed_record_write_is_not_reported_as_an_unrecovered_item() {
        let h = Harness::new();
        let (t, c) = attention_fixture(&h, DrivePlan::OutcomeUnknownOnce);
        // A directory where the record file belongs: the atomic rename cannot replace it.
        std::fs::create_dir_all(h._tmp.path().join("demo").join("_attention.json")).unwrap();

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps())
            .expect_err("s1's outcome is unknown");
        let message = err.to_string();
        assert!(
            message.contains("Needs Attention could not be recorded"),
            "{message}"
        );
        assert!(!message.contains("unrecoverable"), "{message}");

        // And the card reads the same state: a record that cannot be read is never a clean tweak.
        let attention = detect::detect(&t, &c, &h.deps())
            .attention
            .expect("an unreadable record is surfaced, not swallowed");
        assert_eq!(attention.reason, AttentionReason::RecordUnreadable);
    }

    #[test]
    fn omitted_undo_action_driven_back() {
        let h = Harness::new();
        // `action_key` always derives from the action's `apply` script text, even for a probe
        // read -- so the presence seed must use "act_apply", never the probe script's own text.
        h.set_present("act_apply", true); // the live surface currently shows it present
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act", true, true),
            ],
            vec![opt(
                "Skip",
                vec![(
                    "anchor",
                    set(Value::Startup(crate::tweaks::model::StartupType::Manual)),
                )],
            )], // genuinely omits the undo-carrying probeable action
        );
        let c = corpus(vec![t.clone()], vec![]);

        let outcome =
            run_apply(&t, &c, &OptLabel("Skip".into()), &h.deps()).expect("drive-back succeeds");
        assert!(outcome
            .effects
            .iter()
            .any(|r| r.kind == EffectResultKind::UndoDrivenBack));
        assert_eq!(outcome.status.residues, Vec::new());

        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("a successful apply keeps its entry as history");
        assert!(entry
            .journal
            .iter()
            .any(|r| r.action_id.0 == "act" && r.completed));
        let log = h.log();
        // `action_key` derives from the action's `apply` script text uniformly across
        // apply/undo/probe calls, so the undo op is tagged "act_apply" too, not "act_undo".
        assert!(log.contains(&Op::RunUndo("act_apply".into())));
        assert!(!log.contains(&Op::RunApply("act_apply".into())));
    }

    #[test]
    fn noundo_residue_left_in_place() {
        let h = Harness::new();
        h.set_present("act_apply", true);
        // Anchor mismatches "Skip"'s authored value, so detect finds SystemDefault at Step 0
        // (not a vacuous already-Active no-op) -- this apply genuinely traverses Steps 1-4 and
        // exercises this file's OWN residue tracking, not just a reused `detect` result.
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act", false, true),
            ], // no undo -- one-way
            vec![opt(
                "Skip",
                vec![(
                    "anchor",
                    set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let outcome =
            run_apply(&t, &c, &OptLabel("Skip".into()), &h.deps()).expect("apply succeeds");
        assert_eq!(outcome.status.residues, vec![EffectId("act".into())]);
        let log = h.log();
        assert!(!log.contains(&Op::RunApply("act_apply".into())));
        assert!(!log.contains(&Op::RunUndo("act_undo".into())));
    }

    /// Fix 2 regression (review): apply and restore must agree on what applying an option does.
    /// `validate::applicable_surface` (a detection concept) correctly excludes ephemeral actions,
    /// but a declared `run` ephemeral action must still physically execute on apply (spec §7) --
    /// restore's `reapply_option_ref` already did this; `do_apply` did not, until this fix.
    #[test]
    fn apply_runs_a_declared_ephemeral_action() {
        let h = Harness::new();
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![svc_effect("anchor", false), ephemeral_effect("eph")],
            vec![opt(
                "A",
                vec![
                    (
                        "anchor",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("eph", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let outcome = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("apply succeeds");
        assert!(
            outcome
                .effects
                .iter()
                .any(|r| r.effect.0 == "eph" && r.kind == EffectResultKind::Ran),
            "the ephemeral action must run and report Ran: {:?}",
            outcome.effects
        );

        let log = h.log();
        assert!(
            log.contains(&Op::RunApply("eph_apply".into())),
            "a declared `run` ephemeral action must actually execute on apply (spec §7): {log:?}"
        );
        assert!(
            !log.iter().any(|op| matches!(op, Op::Probe(_))),
            "an ephemeral action carries no probe -- it must never be probed/verify-reversed: {log:?}"
        );

        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("a successful apply keeps its entry as history");
        if let Captured::Values(map) = &entry.captured {
            assert!(
                !map.contains_key(&EffectId("eph".into())),
                "an ephemeral action carries no persistent state -- it must never be captured"
            );
        } else {
            panic!("expected a Values capture (pre-apply state was SystemDefault)");
        }
    }

    /// Fix 2 regression: an ephemeral action must never appear in the persisted WAL journal at all
    /// (spec §7, invariant 10) -- a real undo-carrying/probeable action declared alongside it must
    /// still be journaled and marked completed normally.
    #[test]
    fn ephemeral_action_is_not_journaled() {
        let h = Harness::new();
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                ephemeral_effect("eph"),
                action_effect("act", true, true),
            ],
            vec![opt(
                "A",
                vec![
                    (
                        "anchor",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("eph", OptValue::Run(None)),
                    ("act", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("apply succeeds");

        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("a successful apply keeps its entry as history");
        assert!(
            entry.journal.iter().all(|r| r.action_id.0 != "eph"),
            "an ephemeral action must never appear in the journal: {:?}",
            entry.journal
        );
        assert!(
            entry
                .journal
                .iter()
                .any(|r| r.action_id.0 == "act" && r.completed),
            "a real undoable/probeable action must still be journaled and marked completed: {:?}",
            entry.journal
        );
    }

    #[test]
    fn shared_claims_processed_in_order() {
        let h = Harness::new();
        h.kind.seed(
            "sh_a_addr",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.seed(
            "sh_b_addr",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let shared_a = SharedDef {
            id: SharedId("sh_a".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_a_addr".into(),
            }),
            value: Value::Startup(crate::tweaks::model::StartupType::Disabled),
        };
        let shared_b = SharedDef {
            id: SharedId("sh_b".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_b_addr".into(),
            }),
            value: Value::Startup(crate::tweaks::model::StartupType::Disabled),
        };
        // This tweak already holds sh_b from an earlier state.
        h.claims
            .claim(&shared_b, "demo", &h.kind, &ExecCx::new(Level::User))
            .unwrap();
        h.clear_log(); // discard the pre-seed's own ops -- only `apply`'s ordering matters below

        let t = tweak(
            "demo",
            vec![
                shared_effect("eff_a", "sh_a"),
                shared_effect("eff_b", "sh_b"),
            ],
            vec![opt(
                "A",
                vec![
                    ("eff_a", OptValue::Claim(None)),
                    ("eff_b", OptValue::Unclaimed(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![shared_a, shared_b]);

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("apply succeeds");

        assert!(h.claims.is_claimed(&SharedId("sh_a".into())));
        assert!(!h.claims.is_claimed(&SharedId("sh_b".into())));

        let log = h.log();
        let first_a = log
            .iter()
            .position(|op| op == &Op::Drive("sh_a_addr".into()))
            .unwrap();
        let first_b = log
            .iter()
            .position(|op| {
                matches!(op, Op::Drive(k) if k == "sh_b_addr")
                    || matches!(op, Op::Read(k) if k == "sh_b_addr")
            })
            .unwrap();
        assert!(
            first_a < first_b,
            "sh_a (declared first) must be processed before sh_b: {log:?}"
        );
    }

    #[test]
    fn missing_target_apply_fails_typed() {
        let h = Harness::new();
        h.kind.seed(
            "svc",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.drive_plan("svc", DrivePlan::ResourceMissing);
        let t = tweak(
            "demo",
            vec![svc_effect("svc", false)],
            vec![opt(
                "A",
                vec![(
                    "svc",
                    set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err =
            run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("resource vanished");
        let EngineError::RollbackReport { original, .. } = err else {
            panic!("expected RollbackReport");
        };
        assert!(matches!(*original, EngineError::ResourceMissing(_)));
    }

    /// An `optional` effect whose resource is absent already reads as its `if_missing` value (spec
    /// §5.4) -- that is what `detect::classify_setting_read` reports, so detect offers the option as
    /// available and satisfied. Apply has to agree: asking for that same value is a verified no-op,
    /// not a failure. Without this, every `optional` task effect in the corpus is a latent hard
    /// failure on any build where the task is absent.
    #[test]
    fn absent_optional_effect_matching_if_missing_is_a_verified_no_op() {
        let h = Harness::new();
        let disabled = Value::Startup(crate::tweaks::model::StartupType::Disabled);
        h.kind.seed(
            "svc",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.drive_plan("svc", DrivePlan::ResourceMissing);
        let mut e = svc_effect("svc", true);
        e.if_missing = Some(disabled.clone());
        let t = tweak(
            "demo",
            vec![e],
            vec![opt("A", vec![("svc", set(disabled))])],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let outcome = run_apply(&t, &c, &OptLabel("A".into()), &h.deps())
            .expect("an absent optional effect already at its if_missing value must not fail");
        assert_eq!(
            outcome.effects[0].kind,
            EffectResultKind::NoOp,
            "an absent optional effect must be recorded as a no-op, never as Driven"
        );
    }

    /// The no-op above is deliberately narrow: an absent `optional` effect whose option wants
    /// something OTHER than its `if_missing` value is genuinely unsatisfiable here and must still
    /// fail loudly, rather than `ResourceMissing` being swallowed wholesale.
    #[test]
    fn absent_optional_effect_wanting_a_different_value_still_fails() {
        let h = Harness::new();
        h.kind.seed(
            "svc",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.drive_plan("svc", DrivePlan::ResourceMissing);
        let mut e = svc_effect("svc", true);
        e.if_missing = Some(Value::Startup(crate::tweaks::model::StartupType::Manual));
        let t = tweak(
            "demo",
            vec![e],
            vec![opt(
                "A",
                vec![(
                    "svc",
                    set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps())
            .expect_err("a missing resource that cannot satisfy the option must still fail");
        let EngineError::RollbackReport { original, .. } = err else {
            panic!("expected RollbackReport");
        };
        assert!(matches!(*original, EngineError::ResourceMissing(_)));
    }

    /// Beyond the 15 named scenarios: a shared claim taken earlier in a failed apply must be
    /// reversed by rollback (the captured entry excludes shared effects entirely, spec §8.1 step
    /// 1, so nothing else would ever undo it) -- pins the `ProcessedEffect::SharedClaim` path.
    #[test]
    fn rollback_reverses_a_shared_claim_taken_earlier_in_the_same_failed_apply() {
        let h = Harness::new();
        h.kind.seed(
            "sh_addr",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.seed(
            "s2",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.actions.fail_apply("a1_apply"); // fails AFTER the shared claim already succeeded
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(crate::tweaks::model::StartupType::Disabled),
        };
        let t = tweak(
            "demo",
            vec![
                shared_effect("sh_eff", "sh"),
                action_effect("a1", false, false),
            ],
            vec![opt(
                "A",
                vec![
                    ("sh_eff", OptValue::Claim(None)),
                    ("a1", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![shared]);

        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("a1 fails");
        assert!(matches!(err, EngineError::RollbackReport { .. }));
        assert!(
            !h.claims.is_claimed(&SharedId("sh".into())),
            "the claim taken before the later failure must be released by rollback"
        );
        assert_eq!(
            h.kind.live_value("sh_addr"),
            Value::Startup(crate::tweaks::model::StartupType::Manual)
        );
    }

    #[test]
    fn crash_window_simulation_via_apply_journal() {
        // A companion to lifecycle's own `crash_window_simulation` (which pins the pure scan
        // primitive directly): here the SAME on-disk shape arises end-to-end from an ordinary
        // failed apply. act2's row ends up `intended: true, completed: false` -- and that is
        // genuinely indistinguishable, from the journal alone, between "act2's own apply call
        // failed before mark_completed ever ran" and "act2 ran, and the process crashed before
        // the mark reached disk" (an arbitrary script's exit code says nothing about what side
        // effects it had). The scanner correctly treats both the same way: flagged, never
        // silently trusted as safe.
        let h = Harness::new();
        h.kind.seed(
            "anchor",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act1", false, false),
                action_effect("act2", false, false),
            ],
            vec![opt(
                "A",
                vec![
                    (
                        "anchor",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                    ("act1", OptValue::Run(None)),
                    ("act2", OptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.actions.fail_apply("act2_apply");

        run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("act2 fails, act1 one-way");

        let entry = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("kept");
        let flagged = crate::tweaks::engine::lifecycle::scan_for_crash_residue(&[entry])
            .expect("act2's intended-but-unmarked row must be flagged");
        assert_eq!(flagged.items.len(), 1);
        assert!(flagged.items[0].message.contains("act2"));
    }

    #[test]
    fn unknown_option_is_a_typed_error() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("svc", false)],
            vec![opt(
                "A",
                vec![(
                    "svc",
                    set(Value::Startup(crate::tweaks::model::StartupType::Manual)),
                )],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let err = run_apply(&t, &c, &OptLabel("DoesNotExist".into()), &h.deps())
            .expect_err("unknown target option");
        assert!(matches!(err, EngineError::UnknownOption(_)));
    }

    #[test]
    fn unavailable_windows_scope_is_a_typed_error_no_mutation() {
        let h = Harness::new();
        let mut t = tweak(
            "demo",
            vec![svc_effect("svc", false)],
            vec![opt(
                "A",
                vec![(
                    "svc",
                    set(Value::Startup(crate::tweaks::model::StartupType::Manual)),
                )],
            )],
        );
        t.windows = Some(WindowsScope {
            products: None,
            build: Some(crate::tweaks::model::BuildExpr::Min(26100)), // excludes running 19045
            revision: None,
        });
        let c = corpus(vec![t.clone()], vec![]);
        let err = run_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect_err("unavailable");
        assert!(matches!(err, EngineError::Unavailable(_)));
        assert!(
            h.log().is_empty(),
            "an unavailable tweak must never be touched"
        );
    }

    // --- CRITICAL 2 safety test: per-effect execution-context routing is actually wired ----------
    //
    // Before this fix, EVERY effect's drive built its `ExecCx` from the flat `Deps.level` ceiling,
    // so the HKCU-always-in-process-as-User exception (`context::route`) was correct code that no
    // production drive call ever consulted -- an HKCU effect under a System/TI floor would have
    // driven at that ceiling, hitting the elevated child's own HKCU instead of the interactive
    // user's (the exact over-the-shoulder failure ADR-0005 exists to prevent). This test fixture
    // records the `ExecCx::level()` each `drive` call actually receives; against the OLD code it
    // would show `Ti` for both effects below (failing this test) -- against the fix, only the
    // HKLM effect does.
    //
    // A recorded `Level::User` is a sufficient proxy for "did not route to the broker": it is
    // exactly the value `engine::AllKinds::drive` (production, not this mock) branches on --
    // `Level::User | Level::Admin` never reaches the broker translation at all (see
    // `tweaks/engine/mod.rs`). A full end-to-end run through the real broker would need actual
    // elevation, out of place for a pure-mock engine test.

    /// Records the `ExecCx::level()` each `drive` call actually received, keyed by the registry
    /// value name driven.
    #[derive(Default)]
    struct LevelRecordingKind {
        levels: Mutex<Vec<(String, Level)>>,
        live: Mutex<HashMap<String, Value>>,
        /// The size of each `drive_batch` call, so a test can prove a run crossed once.
        batches: Mutex<Vec<usize>>,
    }

    impl EffectKind for LevelRecordingKind {
        fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, KindError> {
            let Setting::Registry(addr) = s else {
                panic!("this fixture only exercises Registry settings, got {s:?}");
            };
            Ok(self
                .live
                .lock()
                .unwrap()
                .get(&addr.name)
                .cloned()
                .unwrap_or(Value::Absent))
        }

        fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), KindError> {
            let Setting::Registry(addr) = s else {
                panic!("this fixture only exercises Registry settings, got {s:?}");
            };
            self.levels
                .lock()
                .unwrap()
                .push((addr.name.clone(), cx.level()));
            self.live
                .lock()
                .unwrap()
                .insert(addr.name.clone(), target.clone());
            Ok(())
        }

        /// Records the run size, then behaves exactly like the trait default so the rest of the
        /// fixture is unchanged.
        fn drive_batch(
            &self,
            items: &[(&Setting, &Value)],
            cx: &ExecCx,
        ) -> Result<(), BatchFailure> {
            self.batches.lock().unwrap().push(items.len());
            for (index, (setting, target)) in items.iter().enumerate() {
                self.drive(setting, target, cx)
                    .map_err(|error| BatchFailure {
                        index,
                        error,
                        completed: index,
                    })?;
            }
            Ok(())
        }
    }

    fn registry_effect(id: &str, hive: Hive) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Setting(Setting::Registry(RegAddr {
                hive,
                path: "Software\\Test".to_string(),
                name: id.to_string(),
                ty: RegType::Dword,
                field: None,
            })),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    #[test]
    fn hkcu_effect_drives_in_process_as_user_even_under_a_ti_floor() {
        let kind = LevelRecordingKind::default();
        let probes = MockProbes::new(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(HashMap::new())),
        );
        let actions = MockActions::new(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(HashMap::new())),
        );
        let tmp = tempfile::tempdir().unwrap();
        let claims = ClaimsStore::open(tmp.path().to_path_buf(), Some("test-guid".into()));
        let snapshots = SnapshotStore::open(tmp.path().to_path_buf());
        let cache = ProbeCache::new();

        let hkcu = registry_effect("hkcu_val", Hive::Hkcu);
        let hklm = registry_effect("hklm_val", Hive::Hklm);
        let mut t = tweak(
            "demo",
            vec![hkcu, hklm],
            vec![opt(
                "On",
                vec![
                    ("hkcu_val", set(Value::Reg(TypedRegValue::Dword(1)))),
                    ("hklm_val", set(Value::Reg(TypedRegValue::Dword(1)))),
                ],
            )],
        );
        t.elevation = Level::Ti; // the floor
        let c = corpus(vec![t.clone()], vec![]);

        let deps = Deps {
            kinds: &kind,
            probes: &probes,
            actions: &actions,
            claims: &claims,
            snapshots: &snapshots,
            probe_cache: &cache,
            machine_guid: Some("test-guid"),
            // "What the app currently has" -- irrelevant to routing a DRIVE (only reads consult
            // it); left deliberately different from the tweak's Ti floor to prove routing doesn't
            // accidentally fall back to it.
            level: Level::Admin,
            running: WinVer {
                build: 19045,
                revision: 0,
            },
        };

        run_apply(&t, &c, &OptLabel("On".into()), &deps)
            .expect("apply must succeed against the mock");

        let levels = kind.levels.lock().unwrap();
        let level_of = |name: &str| levels.iter().find(|(n, _)| n == name).map(|(_, l)| *l);
        assert_eq!(
            level_of("hkcu_val"),
            Some(Level::User),
            "an HKCU effect must drive in-process as User even under a Ti floor -- got {levels:?}"
        );
        assert_eq!(
            level_of("hklm_val"),
            Some(Level::Ti),
            "a sibling HKLM effect at the Ti floor must still route at Ti -- got {levels:?}"
        );
    }

    /// The whole point of grouped execution: a run of same-level elevated Settings crosses into
    /// ONE child, not one per effect. Acquiring TrustedInstaller is entirely a per-spawn cost, so
    /// this is the difference between paying it once and paying it once per effect.
    ///
    /// The HKCU effect in the middle is load-bearing. `route` forces it in-process as the
    /// interactive user regardless of the floor, so it must split the run rather than being
    /// swept into an elevated batch.
    #[test]
    fn a_run_of_elevated_effects_shares_one_child_and_an_hkcu_effect_splits_it() {
        let kind = LevelRecordingKind::default();
        let probes = MockProbes::new(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(HashMap::new())),
        );
        let actions = MockActions::new(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(HashMap::new())),
        );
        let tmp = tempfile::tempdir().unwrap();
        let claims = ClaimsStore::open(tmp.path().to_path_buf(), Some("test-guid".into()));
        let snapshots = SnapshotStore::open(tmp.path().to_path_buf());
        let cache = ProbeCache::new();

        // Two elevated, one in-process, then two more elevated.
        let ids = ["a", "b", "user_c", "d", "e"];
        let effects = vec![
            registry_effect("a", Hive::Hklm),
            registry_effect("b", Hive::Hklm),
            registry_effect("user_c", Hive::Hkcu),
            registry_effect("d", Hive::Hklm),
            registry_effect("e", Hive::Hklm),
        ];
        let values: Vec<_> = ids
            .iter()
            .map(|id| (*id, set(Value::Reg(TypedRegValue::Dword(1)))))
            .collect();
        let mut t = tweak("demo", effects, vec![opt("On", values)]);
        t.elevation = Level::Ti;
        let c = corpus(vec![t.clone()], vec![]);

        let deps = Deps {
            kinds: &kind,
            probes: &probes,
            actions: &actions,
            claims: &claims,
            snapshots: &snapshots,
            probe_cache: &cache,
            machine_guid: Some("test-guid"),
            level: Level::Admin,
            running: WinVer {
                build: 19045,
                revision: 0,
            },
        };

        run_apply(&t, &c, &OptLabel("On".into()), &deps)
            .expect("apply must succeed against the mock");

        assert_eq!(
            *kind.batches.lock().unwrap(),
            vec![2, 2],
            "the two elevated runs must each cross once; the HKCU effect between them splits them"
        );

        // Order is never reordered by grouping (invariant 18).
        let levels = kind.levels.lock().unwrap();
        assert_eq!(
            levels.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            ids,
            "grouping must preserve declaration order -- got {levels:?}"
        );
        assert_eq!(
            levels.iter().find(|(n, _)| n == "user_c").map(|(_, l)| *l),
            Some(Level::User),
            "the HKCU effect must still drive in-process as the user"
        );
    }

    /// An Action's undo routes from the tweak's floor, never from what the app currently holds,
    /// while its verify probe only reads and so stays at `Deps.level` (invariant 24). `ti` is not
    /// tested here: validation forbids a ti-routed action outright (`check_action_never_ti`).
    #[test]
    fn an_action_undo_routes_from_the_floor_not_the_run_level() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        // The later effect's drive failure is what forces the rollback that undoes the action.
        h.kind.drive_plan("s1", DrivePlan::Err);
        let mut t = tweak(
            "demo",
            vec![action_effect("act", true, true), svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![
                    ("act", OptValue::Run(None)),
                    (
                        "s1",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                ],
            )],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);
        let deps = Deps {
            level: Level::User,
            ..h.deps()
        };

        run_apply(&t, &c, &OptLabel("A".into()), &deps).expect_err("the later effect fails");

        assert_eq!(
            h.actions.level_of("act_apply:undo"),
            Some(Level::Admin),
            "the undo must run at the tweak's floor, not at Deps.level"
        );
        assert_eq!(
            h.probes.last_level_of("act_apply"),
            Some(Level::User),
            "the reversal's verify probe only reads, so it stays at Deps.level"
        );
    }

    /// A claim must be given back, and at the level it was taken at: the two recorded drives are
    /// the claim and its reversal, so a release that never runs fails as loudly as a wrong level.
    #[test]
    fn a_shared_claim_is_released_at_the_level_it_was_taken_at() {
        let h = Harness::new();
        h.kind.seed(
            "s1",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.seed(
            "sh_addr",
            Value::Startup(crate::tweaks::model::StartupType::Manual),
        );
        h.kind.drive_plan("s1", DrivePlan::Err);
        let mut sh = shared_effect("sh_eff", "sh");
        sh.elevation = Some(Level::Ti);
        let mut t = tweak(
            "demo",
            vec![sh, svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![
                    ("sh_eff", OptValue::Claim(None)),
                    (
                        "s1",
                        set(Value::Startup(crate::tweaks::model::StartupType::Disabled)),
                    ),
                ],
            )],
        );
        t.elevation = Level::Admin;
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(crate::tweaks::model::StartupType::Disabled),
        };
        let c = corpus(vec![t.clone()], vec![shared]);
        let deps = Deps {
            level: Level::Admin,
            ..h.deps()
        };

        run_apply(&t, &c, &OptLabel("A".into()), &deps).expect_err("the later effect fails");

        assert_eq!(
            h.kind.drive_levels("sh_addr"),
            vec![Level::Ti, Level::Ti],
            "the claim drives at Ti, and the rollback's release must drive it back at Ti"
        );
    }
}
