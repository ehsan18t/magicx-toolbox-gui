//! Restore Snapshot (spec §8.5, +§8.6; ADR-0002/0003/0007). Thin by design: this module owns only
//! the controller sequencing -- undo the head entry's completed actions in reverse, then re-apply
//! its target -- and composes [`apply`]'s drive/verify primitives for every actual
//! mutation. No new drive/verify logic is written here.
//!
//! ## The steps (spec §8.5), mirrored from `apply.rs`'s own numbering
//! 0. **Lock; find the head entry.** [`lifecycle::lock_tweak`] serializes against a concurrent
//!    apply/restore of the same tweak (spec §8.7). [`SnapshotStore::head`] already skips
//!    invalid/dangling entries (ADR-0002) -- if none remain, the surface simply reads as System
//!    Default (ADR-0003): nothing to restore, nothing consumed. Every entry `head` skipped as
//!    invalid is still surfaced (via `list`) as `skipped_invalid`, never silently dropped.
//! 1. **Reverse the entry's completed journal actions, in reverse order** -- these are exactly the
//!    actions that ran when the user *left* the state this entry captured (ADR-0007). Reuses
//!    [`apply::verify_reversed_probe`] verbatim for the same did-it-work discipline apply's own
//!    rollback uses; a completed no-undo action is reported un-undoable (incomplete), never fatal to
//!    the rest of the walk.
//! 2. **Re-apply the target**, re-derived from the *current* corpus (ADR-0007), never the possibly
//!    stale `tweak` parameter:
//!    - `Captured::OptionRef` -- [`apply::drive_to_captured`] drives its Settings (it re-derives
//!      internally too); a small declaration-order-preserving surface (Shared + Action effects,
//!      ephemerals included) is then driven via [`apply::drive_forward`] verbatim, exactly like a
//!      fresh apply of that option minus its own snapshot capture. Driven with
//!      `Journaling::InFlight` on the entry being restored, never a new entry (see below).
//!    - `Captured::Values` -- `drive_to_captured` for Settings, plus [`release_shared_claims`] for
//!      any Shared effect this tweak currently holds (see below); scripts cannot be
//!      re-run from a dump, so the outcome carries `reboot_advisory: true`.
//! 3. **Shared claims recompute like an ordinary apply** (spec §8.6) -- a side effect of routing
//!    Shared effects through `drive_forward` in step 2, never special-cased here.
//! 4. **Verify + consume/keep** (ADR-0002, invariant 8/20): every undo and re-apply drive verified
//!    ⇒ consume the head entry, the next-most-recent becomes head; any failure ⇒ the entry is kept
//!    and every failure returned, never consumed on uncertainty.
//! 5. **Invalidate the probe cache** -- the tweak's state changed (or a real attempt was made to).
//!
//! ## Reused vs. new
//! `drive_forward`/`verify_reversed_probe`/`drive_to_captured` and their small shared types
//! (`DriveCtx`/`DriveState`/`ActionPlan`/`Journaling`) are `apply.rs`'s, made `pub(crate)` there
//! (visibility-only -- see that file's own docs) and called here unmodified. Only the *sequencing*
//! (undo loop, re-derivation lookups, consume/keep) is new.
//!
//! ## Execution levels
//! Every undo here routes per effect through `context::route` against the CURRENT corpus
//! (ADR-0007), never from `Deps.level`: [`undo_journal`] from the journaled action's own effect,
//! [`release_shared_claims`] from the Shared effect it is releasing. Probes stay on
//! `context::read_route`, since reads never escalate (invariant 24).
//!
//! ## No throwaway snapshot entry
//! Restore pushes nothing: an extra entry takes the next seq, so a crash or failed discard lets it
//! win `head()` and mask the real return point (ADR-0002). Its action steps are marked in flight on
//! the entry being restored instead. `option_ref_reapply_pushes_no_extra_entry` pins this.
//!
//! ## A Values-dump restore releases held shared claims
//! A `Captured::Values` dump means the pre-apply state matched no option, so the tweak held no
//! shared claim then; restoring it must release any claim taken since (spec §8.5/§8.6), via
//! [`release_shared_claims`]. Otherwise the shared value never returns to its original.

use std::collections::BTreeSet;

use super::apply::attention_item;
use crate::tweaks::model::{
    ActionDef, Corpus, Effect, EffectDef, EffectId, OptLabel, OptValue, Tweak,
};
use crate::tweaks::shared_claims::ReleaseOutcome;
use crate::tweaks::snapshot::{
    Attention, AttentionReason, Captured, Entry, EntrySummary, EntryValidity, Seq,
};
use crate::tweaks::validate::{option_unavailable, Milestone};
use crate::tweaks::winver::WinVer;

use super::apply::{self, ActionPlan, DriveCtx, DriveState, EngineError};
use super::detect::{self, HeldInfo, TweakState, TweakStatus, UnavailableOpt};
use super::{context, lifecycle, Deps, Phase};

/// `restore`'s result: a [`TweakStatus`] from this operation's own verify reads (no re-scan), the
/// consumed entry if any, the Values-dump reboot advisory, and every invalid/dangling entry `head`
/// bypassed (surfaced, never silently dropped: ADR-0002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub status: TweakStatus,
    /// `Some(seq)` iff a valid entry existed and was restored + consumed; `None` means there was
    /// nothing to restore (ADR-0003), or the entry was kept for an action step still unfinished.
    pub consumed: Option<Seq>,
    /// `true` only for a consumed `Captured::Values` (dump) restore -- scripts hold no state to
    /// re-run, so a reboot/logoff may be needed for full effect (spec §8.5).
    pub reboot_advisory: bool,
    /// Every entry `head` skipped as invalid/dangling, surfaced for a UI discard affordance --
    /// never restored from, never silently deleted (ADR-0002).
    pub skipped_invalid: Vec<EntrySummary>,
}

/// Restores `tweak`'s most recent snapshot entry (spec §8.5). Async only to hold the per-tweak lock
/// across the whole sequence (spec §8.7), mirroring [`apply::apply`].
pub async fn restore(
    tweak: &Tweak,
    corpus: &Corpus,
    deps: &Deps<'_>,
) -> Result<RestoreOutcome, EngineError> {
    let _guard = lifecycle::lock_tweak(&tweak.id).await?;
    do_restore(tweak, corpus, deps)
}

/// [`restore`] for a caller already holding [`lifecycle::lock_tweak`] for `tweak.id`.
pub(crate) fn do_restore(
    tweak: &Tweak,
    corpus: &Corpus,
    deps: &Deps,
) -> Result<RestoreOutcome, EngineError> {
    // `validate.rs`'s Milestone-shaped helpers below stay build-only (see `winver.rs`'s module
    // docs); `winver` is threaded through to `reapply_option_ref` for its direct runtime scope
    // check, which must honor `revision` too.
    let winver = deps.running;
    let milestone = winver.to_milestone();
    let skipped_invalid = list_invalid(&tweak.id, corpus, deps);

    // Step 0: the head entry (already skips invalid/dangling -- ADR-0002).
    let head = deps
        .snapshots
        .head(&tweak.id, corpus, deps.machine_guid, milestone.build)
        .map_err(EngineError::SnapshotWrite)?;
    let Some(entry) = head else {
        // Nothing to restore: the surface already reads System Default (ADR-0003). Nothing was
        // touched, so a plain fresh detect is exactly the right (and only) read here.
        return Ok(RestoreOutcome {
            status: detect::detect(tweak, corpus, deps),
            consumed: None,
            reboot_advisory: false,
            skipped_invalid,
        });
    };

    // ADR-0007: every lookup below re-derives from the CURRENT corpus, never the possibly-stale
    // `tweak` parameter. `head` already proved this id exists (a dangling tweak_id classifies
    // `DanglingRef` and never reaches `head`) -- this is defensive, never a guessed panic.
    let Some(current_tweak) = corpus.tweaks.iter().find(|t| t.id == tweak.id) else {
        return Err(EngineError::Invalid(format!(
            "tweak '{}' no longer exists in the corpus",
            tweak.id
        )));
    };

    // A crash mid-restore otherwise leaves only a restorable entry, never Needs Attention. An
    // already-open mark is an earlier crash's, which only a verified outcome may settle.
    let inherited = apply::Inherited {
        drive_open: !deps
            .snapshots
            .open_drive(&current_tweak.id, entry.seq)
            .map_err(EngineError::SnapshotWrite)?,
        steps: entry.actions_in_flight.clone(),
    };
    let mut failures: Vec<EngineError> = Vec::new();

    // Step 1: undo the entry's completed journal actions, in reverse order.
    let mut accounted = undo_journal(current_tweak, corpus, &entry, deps, &mut failures);

    // Step 2 (+ step 3, folded in via `drive_forward`'s own Shared handling): re-apply the target.
    let mut reboot_advisory = false;
    let mut held_shared: Vec<HeldInfo> = Vec::new();
    let mut residues: Vec<EffectId> = Vec::new();
    let restored_label = match &entry.captured {
        Captured::OptionRef(label) => {
            let result = reapply_option_ref(
                current_tweak,
                corpus,
                label,
                (milestone, &winver),
                entry.seq,
                deps,
            );
            failures.extend(result.failures);
            held_shared = result.held_shared;
            residues = result.residues;
            accounted.extend(result.driven_actions);
            Some(OptLabel(label.clone()))
        }
        Captured::Values(_) => {
            reboot_advisory = true;
            if let Err(errs) =
                apply::drive_to_captured(&entry.captured, &current_tweak.id, corpus, deps)
            {
                failures.extend(errs);
            }
            release_shared_claims(current_tweak, corpus, deps, &mut held_shared, &mut failures);
            None
        }
    };

    // Step 5: the tweak's state changed (or a real attempt was made to) -- invalidate its probes.
    deps.probe_cache.invalidate(&current_tweak.id);

    // Step 4: verify + consume/keep (ADR-0002, invariant 8/20) -- never consume on uncertainty.
    if !failures.is_empty() {
        let attention = Attention {
            reason: AttentionReason::RestoreFailed,
            items: failures
                .iter()
                .map(|e| attention_item(Phase::Restore, e))
                .collect(),
        };
        let mut store = Vec::new();
        match apply::record_attention(deps, &current_tweak.id, attention) {
            Ok(()) => apply::settle_recorded(deps, &current_tweak.id, entry.seq, &inherited),
            Err(e) => store.push(EngineError::AttentionWrite(e)),
        }
        return Err(EngineError::RestoreFailed { failures, store });
    }
    // Accounts only for actions it undid, re-drove or probed absent; either head kind drives every
    // Setting captured, so it settles every drive mark, and any other residue is recorded.
    let unrecorded =
        match apply::settle_verified(deps, &current_tweak.id, apply::Settle::All, &accounted) {
            apply::Settled::Unrecorded(attention, _) => Some(attention),
            apply::Settled::Clean | apply::Settled::Recorded => None,
        };
    // `consume` keeps an entry still holding a step this restore never drove, which the settle
    // above has already recorded.
    let consumed = match deps.snapshots.consume(&current_tweak.id, entry.seq) {
        Ok(true) => Some(entry.seq),
        Ok(false) => None,
        // Every effect verified, so this is not a failed restore and nothing needs attention: the
        // machine is restored and only the spent return point outlived it.
        Err(e) => return Err(EngineError::EntryCleanup(e)),
    };

    let (has_history, recorded) = detect::history(&current_tweak.id, corpus, deps);
    let attention = unrecorded.or(recorded);
    let status = TweakStatus {
        state: restored_label.map_or(TweakState::SystemDefault, TweakState::Active),
        unavailable: unavailable_options(current_tweak, &milestone),
        residues,
        has_history,
        attention,
        held_shared,
        // Left empty deliberately: the restore path has no readings pass, and the frontend only
        // renders the panel when this is non-empty, so the next detect fills it in.
        observed: Vec::new(),
    };
    Ok(RestoreOutcome {
        status,
        consumed,
        reboot_advisory,
        skipped_invalid,
    })
}

/// Step 1 (spec §8.5): reverses `journal`'s completed rows in reverse declaration order -- the
/// actions that ran when the user left the state now being restored to (ADR-0007): an applied row
/// by its `undo`, an undo-back row by re-running `apply`, each verified by its probe. A completed
/// action with no `undo` is reported un-undoable, never fatal to the rest of the walk. Returns the
/// steps it drove and verified, which is what a verified restore may resolve.
fn undo_journal(
    tweak: &Tweak,
    corpus: &Corpus,
    entry: &Entry,
    deps: &Deps,
    failures: &mut Vec<EngineError>,
) -> BTreeSet<EffectId> {
    let mut undone = BTreeSet::new();
    for row in entry.journal.iter().rev().filter(|r| r.completed) {
        let Some((effect, action_def)) = find_action(tweak, &row.action_id) else {
            failures.push(EngineError::Invalid(format!(
                "completed action '{}' vanished from the surface during restore",
                row.action_id
            )));
            continue;
        };
        if is_ephemeral(action_def) {
            // Apply never journals an ephemeral, but one here must be skipped, never reported
            // un-undoable, or the entry is stranded (spec §7, invariant 10).
            continue;
        }
        let rerun = row.undo_back;
        if !rerun && !has_undo(action_def) {
            log::warn!(
                "tweak '{}': completed action '{}' has no undo -- reported un-undoable, restore incomplete",
                tweak.id, row.action_id
            );
            failures.push(EngineError::NoUndo(row.action_id.clone()));
            continue;
        }
        // Marked in flight until verified: the drive mark cannot stand in for an action.
        if let Err(e) = deps
            .snapshots
            .begin_action(&tweak.id, entry.seq, &row.action_id)
        {
            failures.push(EngineError::JournalMark {
                effect: row.action_id.clone(),
                source: e,
            });
            continue;
        }
        let cx = context::route(effect, tweak, corpus);
        let ran = if rerun {
            deps.actions.apply(action_def, &cx)
        } else {
            deps.actions.undo(action_def, &cx)
        };
        match ran {
            Ok(()) => {
                let before = failures.len();
                apply::verify_reversed_probe(action_def, effect, rerun, corpus, deps, failures);
                if failures.len() == before {
                    match deps
                        .snapshots
                        .end_action(&tweak.id, entry.seq, &row.action_id)
                    {
                        Ok(()) => {
                            undone.insert(row.action_id.clone());
                        }
                        Err(e) => failures.push(EngineError::JournalMark {
                            effect: row.action_id.clone(),
                            source: e,
                        }),
                    }
                }
            }
            Err(e) => failures.push(EngineError::ActionFailed {
                effect: row.action_id.clone(),
                source: e,
            }),
        }
    }
    undone
}

/// What re-applying an OptionRef target's non-Setting effects produced -- bundled so
/// [`reapply_option_ref`] stays under clippy's argument-count lint.
struct OptionRefResult {
    failures: Vec<EngineError>,
    held_shared: Vec<HeldInfo>,
    residues: Vec<EffectId>,
    /// The actions the re-apply drove, so a verified restore resolves their rows and no others.
    driven_actions: BTreeSet<EffectId>,
}

/// Step 2's `Captured::OptionRef` case (spec §8.5, ADR-0007): drives `label`'s Settings via
/// [`apply::drive_to_captured`] (which re-derives the option from `corpus` itself), then drives its
/// Shared/Action effects (ephemerals included) via [`apply::drive_forward`] in declaration order --
/// a full re-apply of the target, minus its own snapshot capture.
fn reapply_option_ref(
    tweak: &Tweak,
    corpus: &Corpus,
    label: &str,
    (milestone, winver): (Milestone, &WinVer),
    seq: Seq,
    deps: &Deps,
) -> OptionRefResult {
    let mut failures = Vec::new();
    let mut held_shared = Vec::new();
    let mut residues = Vec::new();

    if let Err(errs) = apply::drive_to_captured(
        &Captured::OptionRef(label.to_string()),
        &tweak.id,
        corpus,
        deps,
    ) {
        failures.extend(errs);
    }

    let Some(target_opt) = tweak.options.iter().find(|o| o.label.0 == label) else {
        // `head` already proved this label exists on this tweak (a dangling label classifies
        // `DanglingRef`) -- defensive, never a guessed panic.
        failures.push(EngineError::Invalid(format!(
            "captured option '{label}' no longer exists on tweak '{}'",
            tweak.id
        )));
        return OptionRefResult {
            failures,
            held_shared,
            residues,
            driven_actions: BTreeSet::new(),
        };
    };

    // Shared + Action effects only, ephemerals included (unlike `validate::applicable_surface`,
    // which excludes them -- they carry no detectable/reversible signal, but restoring an option
    // that `run`s one must still run it, spec §7/§8.5). Settings were just handled above. Runtime
    // scope decision (spec §6.6/invariant 22): honors `revision` too, unlike the Milestone-based
    // (build-only) helpers elsewhere in this file -- see winver.rs's module docs.
    let surface: Vec<&EffectDef> = tweak
        .surface
        .iter()
        .filter(|e| {
            e.windows.as_ref().is_none_or(|s| s.applies(winver))
                && !matches!(e.kind, Effect::Setting(_))
        })
        .collect();

    // Mirrors apply.rs's own Step-1 action-plan construction (probe once, decide, never touched
    // again) so `drive_forward`/`drive_action` see the exact same shape a fresh apply would build.
    let mut action_plan: Vec<(EffectId, ActionPlan)> = Vec::new();
    let mut probed: BTreeSet<EffectId> = BTreeSet::new();
    let mut plan_failed = false;
    for effect in &surface {
        let Effect::Action(action_def) = &effect.kind else {
            continue;
        };
        let raw = target_opt.values.get(&effect.id);
        let scoped_out =
            matches!(raw, Some(OptValue::Run(w)) if !w.as_ref().is_none_or(|s| s.applies(winver)));
        if scoped_out {
            continue;
        }
        let runs = matches!(raw, Some(OptValue::Run(_)));
        let ActionDef::Script {
            probe: Some(_),
            undo,
            ..
        } = action_def
        else {
            if runs {
                action_plan.push((effect.id.clone(), ActionPlan::Apply));
            }
            continue;
        };
        match detect::probe_live(
            deps,
            action_def,
            &context::read_route(effect, deps.level, corpus),
        ) {
            // A verified read of the action's state, so it accounts for it whatever it reads.
            Ok(present) => {
                probed.insert(effect.id.clone());
                match apply::plan_for(runs, present, undo.is_some()) {
                    apply::ProbedPlan::Drive(plan) => action_plan.push((effect.id.clone(), plan)),
                    apply::ProbedPlan::Residue => residues.push(effect.id.clone()),
                    apply::ProbedPlan::Nothing => {}
                }
            }
            Err(e) => {
                failures.push(EngineError::CaptureFailed {
                    effect: effect.id.clone(),
                    source: e,
                });
                plan_failed = true;
            }
        }
    }
    if plan_failed {
        // The plan is untrustworthy without every probe -- never drive on a partial plan.
        return OptionRefResult {
            failures,
            held_shared,
            residues,
            driven_actions: BTreeSet::new(),
        };
    }

    // Steps are marked in the entry being restored, never a new one (module docs).
    let ctx = DriveCtx {
        tweak,
        corpus,
        target_opt,
        milestone,
        deps,
        journal: apply::Journaling::InFlight(seq),
    };
    let mut state = DriveState::default();
    if let Err(e) = apply::drive_forward(&ctx, &surface, &action_plan, &mut state) {
        failures.push(e);
    }
    let mut driven_actions = state.driven_actions();
    driven_actions.extend(probed);
    held_shared.extend(state.held_shared);

    OptionRefResult {
        failures,
        held_shared,
        residues,
        driven_actions,
    }
}

/// The Values-restore counterpart of `apply::drive_shared`'s `Unclaimed` arm. Walks the whole
/// surface: a claim taken before an OS upgrade scoped its effect out must still be released.
fn release_shared_claims(
    tweak: &Tweak,
    corpus: &Corpus,
    deps: &Deps,
    held_shared: &mut Vec<HeldInfo>,
    failures: &mut Vec<EngineError>,
) {
    for effect in &tweak.surface {
        let Effect::Shared(shared_id) = &effect.kind else {
            continue;
        };
        match apply::holds(deps, shared_id, &tweak.id) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(e) => {
                failures.push(e);
                continue;
            }
        }
        let cx = context::route(effect, tweak, corpus);
        match deps.claims.release(shared_id, &tweak.id, deps.kinds, &cx) {
            Ok(ReleaseOutcome::StillHeld(holders)) => held_shared.push(HeldInfo {
                shared: shared_id.clone(),
                holders,
            }),
            Ok(ReleaseOutcome::RestoredOriginal(_)) => {}
            Err(e) => failures.push(EngineError::Claim {
                shared: shared_id.clone(),
                source: e,
            }),
        }
    }
}

/// The effect is returned alongside its `ActionDef` because undoing one must route it, and only
/// the `EffectDef` carries the step level [`context::route`] needs.
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

/// Mirrors the private `apply::is_ephemeral`; keep in sync. Exempt from ALL reversibility
/// bookkeeping (spec §7, invariant 10).
fn is_ephemeral(action: &ActionDef) -> bool {
    matches!(
        action,
        ActionDef::Script {
            ephemeral: true,
            ..
        }
    )
}

/// A simplified, version-scope-only unavailable check for the restore's own constructed status
/// (no fresh `detect` re-scan); the fuller "authors a real value against a live Missing
/// resource" check `detect` also does would need extra reads restore's own verify pass has no
/// reason to take.
fn unavailable_options(tweak: &Tweak, milestone: &Milestone) -> Vec<UnavailableOpt> {
    tweak
        .options
        .iter()
        .filter(|o| option_unavailable(tweak, o, milestone))
        .map(|o| UnavailableOpt {
            label: o.label.clone(),
            reason: "not applicable on this Windows build".to_string(),
        })
        .collect()
}

/// Every entry `head` would skip (spec §8.3, ADR-0002), for `RestoreOutcome::skipped_invalid`.
fn list_invalid(tweak_id: &str, corpus: &Corpus, deps: &Deps) -> Vec<EntrySummary> {
    match deps
        .snapshots
        .list(tweak_id, corpus, deps.machine_guid, deps.running.build)
    {
        Ok(entries) => entries
            .into_iter()
            .filter(|e| matches!(e.validity, EntryValidity::Invalid(_)))
            .collect(),
        Err(e) => {
            log::warn!(
                "tweak '{tweak_id}': snapshot list unreadable while surfacing invalid entries: {e}"
            );
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::{ActionRunner, ProbeCache, ProbeSource};
    use crate::tweaks::kinds::{BatchFailure, BatchItem, EffectKind, Error as KindError, ExecCx};
    use crate::tweaks::model::{
        Level, OptValue as ModelOptValue, Probe, RiskLevel, ScopedValue, Script, Setting,
        SharedDef, SharedId, Shell, StartupType, SvcAddr, Value,
    };
    use crate::tweaks::shared_claims::ClaimsStore;
    use crate::tweaks::snapshot::{
        is_outstanding, InvalidReason, JournalRow, NewEntry, SnapshotStore,
    };
    use std::collections::{BTreeMap, HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    // --- shared op log (mirrors apply.rs's test harness) ---------------------------------------

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Op {
        Read(String),
        Drive(String),
        Probe(String),
        RunApply(String),
        RunUndo(String),
    }

    type Log = Arc<Mutex<Vec<Op>>>;
    type Presence = Arc<Mutex<HashMap<String, bool>>>;

    fn setting_key(s: &Setting) -> String {
        match s {
            Setting::Service(addr) => addr.name.clone(),
            other => panic!("revert.rs tests only fixture Service settings, got {other:?}"),
        }
    }

    fn action_key(a: &ActionDef) -> String {
        match a {
            ActionDef::Script { apply, .. } => apply.0.clone(),
            ActionDef::DeleteTree { key, .. } => key.path.clone(),
        }
    }

    #[derive(Clone, Copy)]
    enum DrivePlan {
        Err,
        /// The process dies mid-drive: nothing after this point runs.
        Crash,
        /// The resource is absent, so a drive to a real value refuses where `AllKinds`'s pre-check
        /// refuses it: during translation, before any child is spawned.
        ResourceMissing,
    }

    /// A scripted `drive_batch` outcome: the child drove `completed` of the run's items, then
    /// failed naming `index`. The only way a test can produce `completed != index`, which is what
    /// every re-drive decision on the undo path turns on.
    #[derive(Clone, Copy)]
    struct BatchVerdict {
        index: usize,
        completed: usize,
        fail: BatchFail,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum BatchFail {
        OpFailed,
        CouldNotAcquire,
    }

    impl BatchFail {
        fn error(self) -> KindError {
            match self {
                BatchFail::OpFailed => {
                    KindError::Backend("mock: an op inside the child failed".into())
                }
                BatchFail::CouldNotAcquire => KindError::CouldNotAcquireElevation(
                    Level::Ti,
                    crate::services::elevation::AcquireReason::TiServiceNotStarted,
                    "mock: TI unavailable".into(),
                ),
            }
        }
    }

    #[derive(Default)]
    struct MockKind {
        log: Log,
        live: Mutex<HashMap<String, Value>>,
        drive_plan: Mutex<HashMap<String, DrivePlan>>,
        /// The size of each `drive_batch` call, so a test can count children instead of drives.
        batches: Mutex<Vec<usize>>,
        batch_verdicts: Mutex<Vec<BatchVerdict>>,
        /// The `ExecCx::level()` each drive received, so a test can prove which level an undo
        /// actually ran at.
        levels: Mutex<Vec<(String, Level)>>,
        on_drive: Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
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
        fn drive_plan(&self, name: &str, plan: DrivePlan) -> &Self {
            self.drive_plan.lock().unwrap().insert(name.into(), plan);
            self
        }
        fn batch_verdict(&self, verdict: BatchVerdict) -> &Self {
            self.batch_verdicts.lock().unwrap().push(verdict);
            self
        }
        /// A could-not-acquire stays queued: a token this machine cannot acquire is no more
        /// acquirable on the next spawn, which is what makes a re-spawn per item measurable.
        fn take_verdict(&self) -> Option<BatchVerdict> {
            let mut queued = self.batch_verdicts.lock().unwrap();
            let verdict = *queued.first()?;
            if verdict.fail != BatchFail::CouldNotAcquire {
                queued.remove(0);
            }
            Some(verdict)
        }
        fn live_value(&self, name: &str) -> Value {
            self.live
                .lock()
                .unwrap()
                .get(name)
                .cloned()
                .unwrap_or(Value::Absent)
        }
        /// The level of the MOST RECENT drive of `name` -- on a restore that is the undo, any
        /// setup drive having already been recorded.
        fn last_drive_level(&self, name: &str) -> Option<Level> {
            self.levels
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|(k, _)| k == name)
                .map(|(_, l)| *l)
        }
        /// Every drive level recorded for `name`, in order: a test pins both THAT a drive happened
        /// and which level ran it, so a vanished release fails as loudly as a mis-routed one.
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
            Ok(self.live_value(&key))
        }

        fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), KindError> {
            let key = setting_key(s);
            if let Some(f) = &*self.on_drive.lock().unwrap() {
                f();
            }
            self.log.lock().unwrap().push(Op::Drive(key.clone()));
            self.levels.lock().unwrap().push((key.clone(), cx.level()));
            let plan = self.drive_plan.lock().unwrap().get(&key).copied();
            match plan {
                Some(DrivePlan::Err) => Err(KindError::Backend("mock drive failure".into())),
                Some(DrivePlan::Crash) => panic!("simulated crash driving {key}"),
                Some(DrivePlan::ResourceMissing) => {
                    Err(KindError::ResourceMissing(format!("mock: {key} is absent")))
                }
                None => {
                    self.live.lock().unwrap().insert(key, target.clone());
                    Ok(())
                }
            }
        }

        /// Records the run size, then behaves like the trait default unless a test scripted a
        /// [`BatchVerdict`], so an unscripted batched call stays indistinguishable from the
        /// per-effect one to every other assertion.
        fn drive_batch(&self, items: &[BatchItem], cx: &ExecCx) -> Result<(), BatchFailure> {
            self.batches.lock().unwrap().push(items.len());
            if let Some(verdict) = self.take_verdict() {
                for item in &items[..verdict.completed.min(items.len())] {
                    self.drive(item.setting, item.target, cx)
                        .expect("a scripted completed drive must pass");
                }
                return Err(BatchFailure {
                    index: verdict.index,
                    error: verdict.fail.error(),
                    completed: verdict.completed,
                });
            }
            for (index, item) in items.iter().enumerate() {
                self.drive(item.setting, item.target, cx)
                    .map_err(|error| BatchFailure {
                        index,
                        error,
                        completed: index,
                    })?;
            }
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockProbes {
        log: Log,
        presence: Presence,
    }
    impl MockProbes {
        fn new(log: Log, presence: Presence) -> Self {
            Self { log, presence }
        }
    }
    impl ProbeSource for MockProbes {
        fn probe(&self, action: &ActionDef, _cx: &ExecCx) -> Result<bool, KindError> {
            let key = action_key(action);
            self.log.lock().unwrap().push(Op::Probe(key.clone()));
            Ok(*self.presence.lock().unwrap().get(&key).unwrap_or(&false))
        }
    }

    #[derive(Default)]
    struct MockActions {
        log: Log,
        presence: Presence,
        fail_undo: Mutex<HashSet<String>>,
        /// The process dies mid-undo: nothing after this point runs.
        crash_undo: Mutex<HashSet<String>>,
        crash_apply: Mutex<HashSet<String>>,
        /// The `ExecCx::level()` each call received, keyed `"<action>:apply"` / `"<action>:undo"`.
        levels: Mutex<Vec<(String, Level)>>,
    }
    impl MockActions {
        fn new(log: Log, presence: Presence) -> Self {
            Self {
                log,
                presence,
                ..Default::default()
            }
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
            if self.crash_apply.lock().unwrap().contains(&key) {
                panic!("simulated crash running {key}");
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
            if self.crash_undo.lock().unwrap().contains(&key) {
                panic!("simulated crash undoing {key}");
            }
            if self.fail_undo.lock().unwrap().contains(&key) {
                return Err(KindError::ActionFailed(1));
            }
            self.presence.lock().unwrap().insert(key, false);
            Ok(())
        }
    }

    // --- fixture builders ------------------------------------------------------------------------

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
                timeout: None,
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
                timeout: None,
            }),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn set(value: Value) -> ModelOptValue {
        ModelOptValue::Set(ScopedValue {
            value,
            windows: None,
        })
    }

    fn opt(label: &str, values: Vec<(&str, ModelOptValue)>) -> crate::tweaks::model::Opt {
        let mut map = BTreeMap::new();
        for (id, v) in values {
            map.insert(EffectId(id.to_string()), v);
        }
        crate::tweaks::model::Opt {
            label: OptLabel(label.to_string()),
            values: map,
        }
    }

    fn tweak(id: &str, surface: Vec<EffectDef>, options: Vec<crate::tweaks::model::Opt>) -> Tweak {
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
            categories: Vec::new(),
            tweaks,
            shared,
        }
    }

    /// Owns everything a test needs so `Deps` (all borrows) can outlive the call under test.
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
    }

    /// Blocks on `restore`/`apply` without a full async-test harness -- their only await point is
    /// an uncontended lock acquire, which resolves on first poll (mirrors apply.rs's own helper).
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

    fn run_restore(
        tweak: &Tweak,
        corpus: &Corpus,
        deps: &Deps,
    ) -> Result<RestoreOutcome, EngineError> {
        futures_block_on(restore(tweak, corpus, deps))
    }

    /// A `Values` entry holding `s1 = Manual`, with the live surface moved to `Disabled`.
    fn values_restore_setup(h: &Harness) -> (Tweak, Corpus, Seq) {
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        let seq = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::from([(
                        EffectId("s1".into()),
                        Value::Startup(StartupType::Manual),
                    )])),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        (t, c, seq)
    }

    fn open_drives(h: &Harness) -> usize {
        h.snapshots
            .unresolved_entries("demo", Some("test-guid"))
            .unwrap()
            .iter()
            .filter(|e| e.drive_open)
            .count()
    }

    fn crash_reason(h: &Harness) -> Option<AttentionReason> {
        crate::tweaks::engine::lifecycle::record_crash_residue(
            &h.snapshots,
            "demo",
            Some("test-guid"),
        );
        h.snapshots
            .attention("demo", Some("test-guid"))
            .unwrap()
            .map(|a| a.reason)
    }

    fn raised_effects(h: &Harness) -> Vec<Option<EffectId>> {
        h.snapshots
            .attention("demo", Some("test-guid"))
            .unwrap()
            .map(|a| a.items.into_iter().map(|i| i.effect).collect())
            .unwrap_or_default()
    }

    /// Option A runs `x` (undo, no probe) beside `s1`; option B omits `x`. The head entry holds the
    /// state before A, with `x` journaled completed.
    fn unprobed_undo_setup(h: &Harness) -> (Tweak, Corpus) {
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("x", true, false)],
            vec![
                opt(
                    "A",
                    vec![
                        ("s1", set(Value::Startup(StartupType::Disabled))),
                        ("x", ModelOptValue::Run(None)),
                    ],
                ),
                opt("B", vec![("s1", set(Value::Startup(StartupType::Manual)))]),
            ],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("B".into()),
                    journal: vec![JournalRow {
                        action_id: EffectId("x".into()),
                        intended: true,
                        completed: true,
                        resolved: false,
                        undo_back: false,
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        (t, c)
    }

    /// `x`'s row is already completed, so only the in-flight step records the interrupted undo; a
    /// verified apply that never touches `x` must not settle it with the Settings.
    #[test]
    fn a_crash_mid_undo_stays_raised_until_an_operation_drives_that_action() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        h.actions
            .crash_undo
            .lock()
            .unwrap()
            .insert("x_apply".into());

        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err(), "the undo must crash");
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));
        assert!(raised_effects(&h).contains(&Some(EffectId("x".into()))));

        futures_block_on(apply::apply(&t, &c, &OptLabel("B".into()), &h.deps()))
            .expect("B verifies its Settings");
        assert_eq!(open_drives(&h), 0, "the Settings are re-established");
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));
        assert_eq!(raised_effects(&h), vec![Some(EffectId("x".into()))]);
    }

    /// A retried restore that undoes `x` and verifies accounts for the interrupted step.
    #[test]
    fn a_retried_restore_resolves_the_undo_a_crash_interrupted() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        h.actions
            .crash_undo
            .lock()
            .unwrap()
            .insert("x_apply".into());
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err());
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));

        h.actions.crash_undo.lock().unwrap().clear();
        run_restore(&t, &c, &h.deps()).expect("the retry verifies");
        assert_eq!(crash_reason(&h), None);
    }

    /// An apply that runs `x` and verifies accounts for the step another operation left unfinished.
    #[test]
    fn an_apply_that_drives_the_action_resolves_its_unfinished_step() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        h.actions
            .crash_undo
            .lock()
            .unwrap()
            .insert("x_apply".into());
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err());
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));

        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        futures_block_on(apply::apply(&t, &c, &OptLabel("A".into()), &h.deps()))
            .expect("A runs x and verifies");
        assert_eq!(crash_reason(&h), None);
    }

    /// Restore re-runs `x` for option A with no journal row of its own, so only the step marks a
    /// crash mid-run.
    #[test]
    fn a_crash_mid_reapply_run_is_raised_by_its_step() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        h.actions
            .crash_apply
            .lock()
            .unwrap()
            .insert("x_apply".into());

        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err(), "the run must crash");
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));
        assert!(raised_effects(&h).contains(&Some(EffectId("x".into()))));
    }

    /// `x`'s unfinished step sits on the head, and the restore target never drives or probes `x`
    /// (as when it is scoped out after an update): the entry is the only evidence, so it is kept.
    #[test]
    fn a_verified_restore_keeps_an_entry_holding_a_step_it_never_drove() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        let head = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("B".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        h.snapshots
            .begin_action("demo", head, &EffectId("x".into()))
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("B verifies");
        assert_eq!(outcome.consumed, None);
        assert!(h
            .snapshots
            .unresolved_entries("demo", Some("test-guid"))
            .unwrap()
            .iter()
            .any(|e| e.seq == head && !e.actions_in_flight.is_empty()));
        let reason = outcome.status.attention.map(|a| a.reason);
        assert_eq!(reason, Some(AttentionReason::CrashResidue));
    }

    /// `x`'s row was planned and never confirmed, and a Values restore neither undoes nor probes it:
    /// the entry outlives the restore and a later apply that never drives `x`, until one does.
    #[test]
    fn a_verified_restore_keeps_an_entry_holding_a_row_it_never_accounted_for() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        let head = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::from([(
                        EffectId("s1".into()),
                        Value::Startup(StartupType::Manual),
                    )])),
                    journal: vec![JournalRow {
                        action_id: EffectId("x".into()),
                        intended: true,
                        completed: false,
                        resolved: false,
                        undo_back: false,
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("the Settings verify");
        assert_eq!(outcome.consumed, None);
        assert!(h
            .snapshots
            .unresolved_entries("demo", Some("test-guid"))
            .unwrap()
            .iter()
            .any(|e| e.seq == head && e.journal.iter().any(is_outstanding)));
        let reason = outcome.status.attention.map(|a| a.reason);
        assert_eq!(reason, Some(AttentionReason::CrashResidue));

        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        futures_block_on(apply::apply(&t, &c, &OptLabel("B".into()), &h.deps()))
            .expect("B verifies without driving x");
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));
        assert!(raised_effects(&h).contains(&Some(EffectId("x".into()))));

        futures_block_on(apply::apply(&t, &c, &OptLabel("A".into()), &h.deps()))
            .expect("A runs x and verifies");
        assert_eq!(crash_reason(&h), None);
    }

    /// Q omits `x` while it reads present, so Q's apply drove `x`'s undo; its rollback crashed while
    /// re-running `x`. Restoring that entry returns to `x` present, so it runs `x`, never undoes it.
    #[test]
    fn a_restore_re_runs_an_action_the_apply_drove_back_rather_than_undoing_it() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![action_effect("x", true, true), svc_effect("s1", false)],
            vec![
                opt(
                    "P",
                    vec![
                        ("s1", set(Value::Startup(StartupType::Manual))),
                        ("x", ModelOptValue::Run(None)),
                    ],
                ),
                opt(
                    "Q",
                    vec![("s1", set(Value::Startup(StartupType::Disabled)))],
                ),
            ],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Automatic));
        h.probes
            .presence
            .lock()
            .unwrap()
            .insert("x_apply".into(), true);
        h.kind.drive_plan("s1", DrivePlan::Err);
        h.actions
            .crash_apply
            .lock()
            .unwrap()
            .insert("x_apply".into());

        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply::do_apply(&t, &c, &OptLabel("Q".into()), &h.deps())
        }));
        assert!(unwound.is_err(), "the rollback's re-run must crash");
        assert_eq!(
            h.probes.presence.lock().unwrap().get("x_apply"),
            Some(&false)
        );

        h.actions.crash_apply.lock().unwrap().clear();
        h.kind.drive_plan.lock().unwrap().clear();
        let outcome = run_restore(&t, &c, &h.deps()).expect("the restore verifies");
        assert!(outcome.consumed.is_some());
        assert_eq!(
            h.probes.presence.lock().unwrap().get("x_apply"),
            Some(&true)
        );
        assert_eq!(crash_reason(&h), None);
    }

    /// The machine already has `x`'s state but not the option's Setting, so it reads System
    /// Default. Applying must leave `x` alone and unjournaled, and the revert must restore only the
    /// Setting: running `x`'s undo would force off a state the apply never made.
    #[test]
    fn an_action_already_present_is_neither_run_nor_undone() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("x", true, true)],
            vec![opt(
                "On",
                vec![
                    ("s1", set(Value::Startup(StartupType::Disabled))),
                    ("x", ModelOptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        h.probes
            .presence
            .lock()
            .unwrap()
            .insert("x_apply".into(), true);

        apply::do_apply(&t, &c, &OptLabel("On".into()), &h.deps()).expect("apply verifies");
        assert!(!h.log().contains(&Op::RunApply("x_apply".into())));
        let head = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("the apply keeps its entry");
        assert!(head.journal.is_empty(), "nothing to undo was recorded");

        let outcome = run_restore(&t, &c, &h.deps()).expect("the restore verifies");
        assert!(outcome.consumed.is_some());
        assert!(!h.log().contains(&Op::RunUndo("x_apply".into())));
        assert_eq!(
            h.probes.presence.lock().unwrap().get("x_apply"),
            Some(&true),
            "the state the machine already had survives the revert"
        );
        assert_eq!(h.kind.live_value("s1"), Value::Startup(StartupType::Manual));
    }

    /// A skip is decided from the live probe on every apply, never remembered: once option B drives
    /// `x` back, returning to A must run `x` for real and journal it, and the revert must undo it.
    #[test]
    fn an_action_skipped_once_runs_after_another_option_removed_its_state() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("x", true, true)],
            vec![
                opt(
                    "A",
                    vec![
                        ("s1", set(Value::Startup(StartupType::Disabled))),
                        ("x", ModelOptValue::Run(None)),
                    ],
                ),
                opt(
                    "B",
                    vec![("s1", set(Value::Startup(StartupType::Automatic)))],
                ),
            ],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let present = |h: &Harness| h.probes.presence.lock().unwrap().get("x_apply").copied();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        h.probes
            .presence
            .lock()
            .unwrap()
            .insert("x_apply".into(), true);

        apply::do_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("A verifies");
        assert!(
            !h.log().contains(&Op::RunApply("x_apply".into())),
            "already present: skipped"
        );

        apply::do_apply(&t, &c, &OptLabel("B".into()), &h.deps()).expect("B verifies");
        assert!(
            h.log().contains(&Op::RunUndo("x_apply".into())),
            "B drives x back"
        );
        assert_eq!(present(&h), Some(false));

        apply::do_apply(&t, &c, &OptLabel("A".into()), &h.deps()).expect("A verifies again");
        assert!(
            h.log().contains(&Op::RunApply("x_apply".into())),
            "A now really runs x"
        );
        assert_eq!(present(&h), Some(true));
        let head = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("the apply keeps its entry");
        assert!(head
            .journal
            .iter()
            .any(|r| r.action_id.0 == "x" && r.completed && !r.undo_back));

        run_restore(&t, &c, &h.deps()).expect("the restore back to B verifies");
        assert_eq!(
            present(&h),
            Some(false),
            "the revert undoes the run it recorded"
        );
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(StartupType::Automatic)
        );
    }

    /// A crashed re-apply left `x`'s re-run unfinished on the head. The retry's verified undo of `x`
    /// leaves `x` in a known state, so only the retry's own crash on `y` is raised.
    #[test]
    fn a_verified_undo_settles_an_unfinished_re_run_of_the_same_action() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![
                svc_effect("s1", false),
                action_effect("y", true, false),
                action_effect("x", true, false),
            ],
            vec![
                opt(
                    "P",
                    vec![
                        ("s1", set(Value::Startup(StartupType::Manual))),
                        ("y", ModelOptValue::Run(None)),
                        ("x", ModelOptValue::Run(None)),
                    ],
                ),
                opt(
                    "B",
                    vec![("s1", set(Value::Startup(StartupType::Disabled)))],
                ),
            ],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("P".into()),
                    journal: vec![JournalRow {
                        action_id: EffectId("x".into()),
                        intended: true,
                        completed: true,
                        resolved: false,
                        undo_back: false,
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        let crash_running = |key: &str| {
            let mut crash = h.actions.crash_apply.lock().unwrap();
            crash.clear();
            crash.insert(key.to_string());
        };

        crash_running("x_apply");
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err(), "the re-run of x must crash");

        crash_running("y_apply");
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err(), "the retry must crash before re-running x");
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));
        let raised = raised_effects(&h);
        assert!(raised.contains(&Some(EffectId("y".into()))));
        assert!(!raised.contains(&Some(EffectId("x".into()))));
    }

    /// A probeable `x` that reads absent is exactly what a verified undo leaves, so the retry's
    /// probe settles the step and the entry is consumed.
    #[test]
    fn a_probe_reading_absent_settles_an_unfinished_undo() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("x", true, true)],
            vec![opt(
                "B",
                vec![("s1", set(Value::Startup(StartupType::Manual)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        let head = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("B".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        h.snapshots
            .begin_action("demo", head, &EffectId("x".into()))
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("verifies");
        assert_eq!(outcome.consumed, Some(head));
        assert_eq!(outcome.status.attention, None);
    }

    /// A probe reading present is as much a verified read of `x` as one reading absent, so it
    /// settles the step too; `x` has no undo, so it stays as a disclosed residue.
    #[test]
    fn a_probe_reading_present_settles_an_unfinished_step() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), action_effect("x", false, true)],
            vec![opt(
                "B",
                vec![("s1", set(Value::Startup(StartupType::Manual)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        h.probes
            .presence
            .lock()
            .unwrap()
            .insert("x_apply".into(), true);
        let head = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("B".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        h.snapshots
            .begin_action("demo", head, &EffectId("x".into()))
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("verifies");
        assert_eq!(outcome.consumed, Some(head));
        assert_eq!(outcome.status.attention, None);
    }

    /// A failed undo is named by the record, so its step is settled with it and never reads as an
    /// unfinished change at a later launch.
    #[test]
    fn a_failed_undo_is_named_by_its_record_not_left_unfinished() {
        let h = Harness::new();
        let (t, c) = unprobed_undo_setup(&h);
        h.actions.fail_undo.lock().unwrap().insert("x_apply".into());

        run_restore(&t, &c, &h.deps()).expect_err("x cannot be undone");
        assert_eq!(crash_reason(&h), Some(AttentionReason::RestoreFailed));
        let entries = h
            .snapshots
            .unresolved_entries("demo", Some("test-guid"))
            .unwrap();
        assert!(entries
            .iter()
            .all(|e| e.actions_in_flight.is_empty() && !e.drive_open));
    }

    /// The head's mark was an earlier crash's: a failed restore records its own failure and must
    /// leave that mark, whose crash the next scan still adds to the record.
    #[test]
    fn a_failed_restore_leaves_a_mark_an_earlier_crash_left_on_its_head() {
        let h = Harness::new();
        let (t, c, head) = values_restore_setup(&h);
        assert!(h.snapshots.open_drive("demo", head).unwrap());
        h.kind.drive_plan("s1", DrivePlan::Err);

        run_restore(&t, &c, &h.deps()).expect_err("s1 cannot be driven");
        assert_eq!(open_drives(&h), 1);
        assert_eq!(crash_reason(&h), Some(AttentionReason::RestoreFailed));
        assert!(
            raised_effects(&h).contains(&None),
            "the earlier crash joins the record"
        );
    }

    /// An OS upgrade that scopes the effect out must not turn the restore into a verified no-op
    /// that deletes the only record of the prior value.
    #[test]
    fn a_values_restore_drives_an_effect_an_upgrade_scoped_out() {
        use crate::tweaks::model::{BuildExpr, WindowsScope};
        let h = Harness::new();
        let (mut t, _, _) = values_restore_setup(&h);
        t.surface[0].windows = Some(WindowsScope {
            products: None,
            build: Some(BuildExpr::Max(19044)),
            revision: None,
        });
        let c = corpus(vec![t.clone()], vec![]);

        let outcome = run_restore(&t, &c, &h.deps()).expect("the captured value goes back");
        assert_eq!(h.kind.live_value("s1"), Value::Startup(StartupType::Manual));
        assert!(outcome.consumed.is_some());
    }

    /// The upgrade scoped the effect out and removed its resource: nothing is left to put back, so
    /// the restore is not stuck on it.
    #[test]
    fn a_values_restore_skips_an_effect_an_upgrade_scoped_out_and_removed() {
        use crate::tweaks::model::{BuildExpr, WindowsScope};
        let h = Harness::new();
        let (mut t, _, _) = values_restore_setup(&h);
        t.surface[0].windows = Some(WindowsScope {
            products: None,
            build: Some(BuildExpr::Max(19044)),
            revision: None,
        });
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("s1", Value::Missing);
        h.kind.drive_plan("s1", DrivePlan::ResourceMissing);

        let outcome = run_restore(&t, &c, &h.deps()).expect("nothing left to restore");
        assert!(outcome.consumed.is_some());
    }

    #[test]
    fn a_values_restore_of_an_effect_the_corpus_dropped_keeps_the_entry() {
        let h = Harness::new();
        values_restore_setup(&h);
        let t = tweak(
            "demo",
            vec![svc_effect("other", false)],
            vec![opt(
                "A",
                vec![("other", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.kind.seed("other", Value::Startup(StartupType::Manual));

        run_restore(&t, &c, &h.deps()).expect_err("s1 can no longer be driven");
        assert!(h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .is_some());
    }

    /// The entry stays restorable after the crash; only the mark tells the scan it was mid-drive.
    #[test]
    fn a_crash_mid_restore_surfaces_as_needs_attention() {
        let h = Harness::new();
        let (t, c, _) = values_restore_setup(&h);
        h.kind.drive_plan("s1", DrivePlan::Crash);

        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_restore(&t, &c, &h.deps())
        }));
        assert!(unwound.is_err(), "the drive must crash");
        assert_eq!(crash_reason(&h), Some(AttentionReason::CrashResidue));
    }

    /// The record carries this failure, so the restore settles its own mark and no other.
    #[test]
    fn a_failed_restore_that_recorded_attention_settles_only_its_own_drive() {
        let h = Harness::new();
        let (t, c, older) = values_restore_setup(&h);
        h.snapshots.open_drive("demo", older).unwrap();
        values_restore_setup(&h);
        h.kind.drive_plan("s1", DrivePlan::Err);

        run_restore(&t, &c, &h.deps()).expect_err("s1 cannot be driven");
        let open: Vec<Seq> = h
            .snapshots
            .unresolved_entries("demo", Some("test-guid"))
            .unwrap()
            .into_iter()
            .filter(|e| e.drive_open)
            .map(|e| e.seq)
            .collect();
        assert_eq!(open, vec![older]);
        assert_eq!(crash_reason(&h), Some(AttentionReason::RestoreFailed));
    }

    /// A verified restore settles the whole history, so a mark an earlier crash left on an older
    /// entry cannot re-raise Needs Attention at the next launch.
    #[test]
    fn a_verified_restore_leaves_no_open_drive() {
        let h = Harness::new();
        let (t, c, older) = values_restore_setup(&h);
        h.snapshots.open_drive("demo", older).unwrap();
        let (_, _, head) = values_restore_setup(&h);

        let outcome = run_restore(&t, &c, &h.deps()).expect("verifies");
        assert_eq!(outcome.consumed, Some(head));
        assert_eq!(open_drives(&h), 0);
        assert_eq!(crash_reason(&h), None);
    }

    // --- the 12 named scenarios + the crown-jewel property test --------------------------------

    #[test]
    fn undo_runs_reverse_order_before_reapply() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![
                action_effect("a", true, false),
                action_effect("b", true, false),
            ],
            vec![],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: vec![
                        JournalRow {
                            action_id: EffectId("a".into()),
                            intended: true,
                            completed: true,
                            resolved: false,
                            undo_back: false,
                        },
                        JournalRow {
                            action_id: EffectId("b".into()),
                            intended: true,
                            completed: true,
                            resolved: false,
                            undo_back: false,
                        },
                    ],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("restore succeeds");
        assert_eq!(outcome.consumed, Some(Seq(1)));

        let log = h.log();
        let pos_b = log
            .iter()
            .position(|op| op == &Op::RunUndo("b_apply".into()))
            .expect("b's undo ran");
        let pos_a = log
            .iter()
            .position(|op| op == &Op::RunUndo("a_apply".into()))
            .expect("a's undo ran");
        assert!(
            pos_b < pos_a,
            "journal [a, b] completed must undo in order [b, a]: {log:?}"
        );
    }

    #[test]
    fn option_ref_reapplies_current_definition() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Manual)))],
            )],
        );
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &corpus(vec![t.clone()], vec![]),
                Some("test-guid"),
                19045,
            )
            .unwrap();

        // The corpus is redefined AFTER capture: option "A" now authors a DIFFERENT value.
        let t2 = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c2 = corpus(vec![t2.clone()], vec![]);

        run_restore(&t, &c2, &h.deps()).expect("restore succeeds");
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(StartupType::Disabled),
            "restore must drive the NEW definition, never the value captured at apply time (ADR-0007)"
        );
    }

    #[test]
    fn option_ref_runs_actions_and_ephemerals() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![action_effect("act", false, false), ephemeral_effect("eph")],
            vec![opt(
                "A",
                vec![
                    ("act", ModelOptValue::Run(None)),
                    ("eph", ModelOptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        let log = h.log();
        assert!(log.contains(&Op::RunApply("act_apply".into())));
        assert!(log.contains(&Op::RunApply("eph_apply".into())));
    }

    /// An OptionRef restore that runs an action pushes NO snapshot entry of its own. A decoy entry
    /// proves the store only ever shrinks by exactly the consumed entry.
    #[test]
    fn option_ref_reapply_pushes_no_extra_entry() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![
                svc_effect("anchor", false),
                action_effect("act", true, true),
            ],
            vec![opt(
                "A",
                vec![
                    ("anchor", set(Value::Startup(StartupType::Manual))),
                    ("act", ModelOptValue::Run(None)),
                ],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);

        // A decoy that must survive restore completely untouched -- if a phantom leaked onto disk
        // (the CRITICAL this test guards against), the post-restore count would grow instead of
        // shrinking by exactly one (the consumed entry).
        let mut decoy_map = BTreeMap::new();
        decoy_map.insert(
            EffectId("anchor".into()),
            Value::Startup(StartupType::Disabled),
        );
        let decoy_seq = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(decoy_map),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        assert_eq!(
            h.snapshots
                .list("demo", &c, Some("test-guid"), 19045)
                .unwrap()
                .len(),
            2,
            "the decoy plus the entry about to be restored"
        );

        let outcome = run_restore(&t, &c, &h.deps()).expect("restore succeeds");
        assert!(outcome.consumed.is_some());
        assert!(
            h.log().contains(&Op::RunApply("act_apply".into())),
            "the action must genuinely run -- otherwise Journaling::None would be untested"
        );

        let after = h
            .snapshots
            .list("demo", &c, Some("test-guid"), 19045)
            .unwrap();
        assert_eq!(
            after.len(),
            1,
            "only the decoy may remain -- no throwaway/phantom entry left behind: {after:?}"
        );
        assert_eq!(
            after[0].seq, decoy_seq,
            "the surviving entry must be the untouched decoy, not some other/new entry"
        );
    }

    /// An entry journaled by a REAL apply that ran an ephemeral (not a hand-built empty journal,
    /// which hides the defect) is not stranded on restore: the ephemeral never reaches the journal.
    #[test]
    fn restore_through_a_state_that_ran_an_ephemeral_is_not_stranded() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Automatic)); // matches neither option yet
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false), ephemeral_effect("eph")],
            vec![
                opt(
                    "Opt1",
                    vec![("s1", set(Value::Startup(StartupType::Manual)))],
                ),
                opt(
                    "Opt2",
                    vec![
                        ("s1", set(Value::Startup(StartupType::Disabled))),
                        ("eph", ModelOptValue::Run(None)),
                    ],
                ),
            ],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let deps = h.deps();

        futures_block_on(apply::apply(&t, &c, &OptLabel("Opt1".into()), &deps))
            .expect("apply(Opt1) succeeds");
        futures_block_on(apply::apply(&t, &c, &OptLabel("Opt2".into()), &deps))
            .expect("apply(Opt2) succeeds -- runs the ephemeral as part of the transition");
        assert!(
            h.log().contains(&Op::RunApply("eph_apply".into())),
            "the ephemeral must genuinely have run, not just be declared"
        );

        let outcome = run_restore(&t, &c, &deps)
            .expect("restore must not be stranded by an ephemeral that ran on the way here");
        assert!(outcome.consumed.is_some());
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(StartupType::Manual),
            "restore must return s1 to Opt1's captured value"
        );
    }

    #[test]
    fn dump_drives_values_only_sets_reboot_advisory() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        let t = tweak("demo", vec![svc_effect("s1", false)], vec![]);
        let c = corpus(vec![t.clone()], vec![]);
        let mut map = BTreeMap::new();
        map.insert(EffectId("s1".into()), Value::Startup(StartupType::Disabled));
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(map),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("restore succeeds");
        assert!(
            outcome.reboot_advisory,
            "a dump restore always sets the advisory"
        );
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(StartupType::Disabled)
        );
        assert_eq!(outcome.status.state, TweakState::SystemDefault);
    }

    /// A restore batches the way a rollback does: one child per run of consecutive elevated
    /// effects, in the entry's own effect-id order, with `n3` splitting the two runs there exactly
    /// as an in-process effect splits a forward one.
    #[test]
    fn a_restore_puts_each_run_of_elevated_effects_through_one_child() {
        let h = Harness::new();
        let ids = ["m1", "m2", "n3", "p4", "p5"];
        for id in ids {
            h.kind.seed(id, Value::Startup(StartupType::Disabled));
        }
        let mut t = tweak(
            "demo",
            vec![
                ti_svc_effect("m1"),
                ti_svc_effect("m2"),
                svc_effect("n3", false),
                ti_svc_effect("p4"),
                ti_svc_effect("p5"),
            ],
            vec![],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);
        let mut map = BTreeMap::new();
        for id in ids {
            map.insert(EffectId(id.into()), Value::Startup(StartupType::Manual));
        }
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(map),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert_eq!(
            *h.kind.batches.lock().unwrap(),
            vec![2, 2],
            "four elevated effects across two runs cost two children, not four"
        );
        for id in ids {
            assert_eq!(
                h.kind.live_value(id),
                Value::Startup(StartupType::Manual),
                "{id} must be restored"
            );
        }
    }

    /// A `Values` entry over `ids`, every one a TrustedInstaller service captured at `Manual` and
    /// left live at `Disabled`, so one run covers them all and each has something to drive back.
    fn ti_values_entry(h: &Harness, ids: &[&str]) -> (Tweak, Corpus) {
        for id in ids {
            h.kind.seed(id, Value::Startup(StartupType::Disabled));
        }
        let mut t = tweak(
            "demo",
            ids.iter().map(|id| ti_svc_effect(id)).collect(),
            vec![],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);
        let mut map = BTreeMap::new();
        for id in ids {
            map.insert(EffectId((*id).into()), Value::Startup(StartupType::Manual));
        }
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(map),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        (t, c)
    }

    /// A child that drove NOTHING before failing at its third item must leave the first two to be
    /// driven, never verified: `completed` is what the child proved, and it is not the failing
    /// index. The items behind the failure are a run of their own and cross in one more child.
    #[test]
    fn a_batch_that_proved_no_drive_redrives_the_items_ahead_of_its_failure() {
        let h = Harness::new();
        let (t, c) = ti_values_entry(&h, &["m1", "m2", "m3", "m4", "m5"]);
        h.kind.batch_verdict(BatchVerdict {
            index: 2,
            completed: 0,
            fail: BatchFail::OpFailed,
        });

        let err = run_restore(&t, &c, &h.deps()).expect_err("m3 fails inside the child");
        let EngineError::RestoreFailed { failures, .. } = err else {
            panic!("expected RestoreFailed");
        };
        let [only] = &failures[..] else {
            panic!("one failing effect, no more and no fewer: {failures:?}");
        };
        assert!(
            matches!(only, EngineError::DriveFailed { effect, .. } if effect.0 == "m3"),
            "the batch's failure names the effect that produced it: {only:?}"
        );

        assert_eq!(
            *h.kind.batches.lock().unwrap(),
            vec![5, 2, 2],
            "the unproven prefix re-drives in one child and the untouched tail in one more"
        );
        for id in ["m1", "m2", "m4", "m5"] {
            assert_eq!(
                h.kind.live_value(id),
                Value::Startup(StartupType::Manual),
                "{id} was never proven driven, so it must be driven, not read back"
            );
        }
        assert_eq!(
            h.kind.live_value("m3"),
            Value::Startup(StartupType::Disabled),
            "the charged item is the one item left alone"
        );
    }

    /// A report naming an item outside the run is malformed input on the one path a half-changed
    /// machine depends on: it must charge a real item and re-drive the rest, never index past the
    /// run.
    #[test]
    fn a_batch_failure_naming_an_item_outside_the_run_still_charges_a_real_one() {
        let h = Harness::new();
        let (t, c) = ti_values_entry(&h, &["m1", "m2", "m3"]);
        h.kind.batch_verdict(BatchVerdict {
            index: 99,
            completed: 0,
            fail: BatchFail::OpFailed,
        });

        let err = run_restore(&t, &c, &h.deps()).expect_err("the child failed");
        let EngineError::RestoreFailed { failures, .. } = err else {
            panic!("expected RestoreFailed");
        };
        let [only] = &failures[..] else {
            panic!("one failing effect, no more and no fewer: {failures:?}");
        };
        assert!(
            matches!(only, EngineError::DriveFailed { effect, .. } if effect.0 == "m3"),
            "an unplaceable index charges the run's last item: {only:?}"
        );
        assert_eq!(*h.kind.batches.lock().unwrap(), vec![3, 2]);
        for id in ["m1", "m2"] {
            assert_eq!(h.kind.live_value(id), Value::Startup(StartupType::Manual));
        }
    }

    /// A token that could not be acquired failed for the whole run, not for one item: it costs one
    /// acquire attempt and one attention item, where re-spawning per item would cost one timeout
    /// and one duplicate item per effect.
    #[test]
    fn a_run_whose_token_cannot_be_acquired_spawns_once_and_is_charged_once() {
        let h = Harness::new();
        let (t, c) = ti_values_entry(&h, &["m1", "m2", "m3", "m4"]);
        h.kind.batch_verdict(BatchVerdict {
            index: 0,
            completed: 0,
            fail: BatchFail::CouldNotAcquire,
        });

        let err = run_restore(&t, &c, &h.deps()).expect_err("TI is unavailable");
        let EngineError::RestoreFailed { failures, .. } = err else {
            panic!("expected RestoreFailed");
        };
        let [only] = &failures[..] else {
            panic!("one attention item for the run, not one per effect: {failures:?}");
        };
        assert!(
            matches!(
                only,
                EngineError::DriveFailed {
                    effect,
                    source: KindError::CouldNotAcquireElevation(Level::Ti, ..),
                } if effect.0 == "m1"
            ),
            "got {only:?}"
        );
        assert_eq!(
            *h.kind.batches.lock().unwrap(),
            vec![4],
            "one acquire attempt for the run: the remaining items must not re-spawn"
        );
    }

    /// An `optional` effect whose resource is absent and whose option value is its `if_missing` is
    /// already where the restore wants it. It must never enter a batch: its refusal to translate
    /// would abort the run and surface Needs Attention for an option that applies cleanly.
    #[test]
    fn an_absent_optional_effect_never_enters_an_option_ref_restore_batch() {
        let h = Harness::new();
        h.kind.seed("m1", Value::Startup(StartupType::Disabled));
        h.kind.seed("m3", Value::Startup(StartupType::Disabled));
        h.kind.seed("m2", Value::Missing);
        h.kind.drive_plan("m2", DrivePlan::ResourceMissing);
        let disabled = Value::Startup(StartupType::Disabled);
        let mut t = tweak(
            "demo",
            vec![
                ti_svc_effect("m1"),
                EffectDef {
                    optional: true,
                    if_missing: Some(disabled.clone()),
                    ..ti_svc_effect("m2")
                },
                ti_svc_effect("m3"),
            ],
            vec![opt(
                "A",
                vec![
                    ("m1", set(Value::Startup(StartupType::Manual))),
                    ("m2", set(disabled)),
                    ("m3", set(Value::Startup(StartupType::Manual))),
                ],
            )],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps())
            .expect("an absent optional effect already at its if_missing must not fail a restore");

        assert_eq!(
            *h.kind.batches.lock().unwrap(),
            vec![2],
            "the absent optional leaves the run, and its neighbours still share one child"
        );
        for id in ["m1", "m3"] {
            assert_eq!(h.kind.live_value(id), Value::Startup(StartupType::Manual));
        }
    }

    #[test]
    fn captured_missing_restores_as_noop() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        let t = tweak("demo", vec![svc_effect("s1", true)], vec![]);
        let c = corpus(vec![t.clone()], vec![]);
        let mut map = BTreeMap::new();
        map.insert(EffectId("s1".into()), Value::Missing);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(map),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");
        assert!(
            h.log().iter().all(|op| !matches!(op, Op::Drive(_))),
            "driving to a captured Missing must be a defined no-op"
        );
        assert_eq!(
            h.kind.live_value("s1"),
            Value::Startup(StartupType::Manual),
            "untouched"
        );
    }

    #[test]
    fn verified_restore_consumes_head_next_becomes_head() {
        let h = Harness::new();
        let t = tweak("demo", vec![svc_effect("s1", false)], vec![]);
        let c = corpus(vec![t.clone()], vec![]);
        let seq1 = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        let seq2 = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("restore succeeds");
        assert_eq!(outcome.consumed, Some(seq2));
        assert!(outcome.status.has_history);

        let head = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap()
            .expect("one entry remains");
        assert_eq!(head.seq, seq1, "the next-most-recent entry becomes head");
    }

    /// A verified restore marks the rows it drove and no others. With every row it would silently
    /// clear residue it never touched; with none it would leave its own work for the crash scan to
    /// raise again on the next start.
    #[test]
    fn a_verified_restore_marks_only_the_rows_it_drove() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![action_effect("driven", true, false)],
            vec![opt("A", vec![("driven", ModelOptValue::Run(None))])],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let residue = |id: &str| JournalRow {
            action_id: EffectId(id.into()),
            intended: true,
            completed: false,
            resolved: false,
            undo_back: false,
        };
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: vec![residue("driven"), residue("untouched")],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        let still: Vec<EffectId> = h
            .snapshots
            .unresolved_entries("demo", Some("test-guid"))
            .unwrap()
            .iter()
            .flat_map(|e| e.journal.iter())
            .filter(|r| is_outstanding(r))
            .map(|r| r.action_id.clone())
            .collect();
        assert_eq!(still, vec![EffectId("untouched".into())]);
    }

    #[test]
    fn failed_restore_keeps_entry() {
        let h = Harness::new();
        // "act" has no undo -- a completed no-undo action can never be undone; Step 1 must fail.
        let t = tweak("demo", vec![action_effect("act", false, false)], vec![]);
        let c = corpus(vec![t.clone()], vec![]);
        let seq = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: vec![JournalRow {
                        action_id: EffectId("act".into()),
                        intended: true,
                        completed: true,
                        resolved: false,
                        undo_back: false,
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let err = run_restore(&t, &c, &h.deps()).expect_err("an un-undoable action must fail");
        assert!(matches!(err, EngineError::RestoreFailed { .. }));

        let head = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap();
        assert_eq!(
            head.unwrap().seq,
            seq,
            "a failed restore must keep the entry (ADR-0002/invariant 8)"
        );
    }

    #[test]
    fn incomplete_restore_needs_attention_kept() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        h.kind.drive_plan("s1", DrivePlan::Err); // the re-apply drive fails
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let seq = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let err = run_restore(&t, &c, &h.deps()).expect_err("the reapply drive fails");
        assert!(matches!(err, EngineError::RestoreFailed { .. }));
        let head = h
            .snapshots
            .head("demo", &c, Some("test-guid"), 19045)
            .unwrap();
        assert_eq!(
            head.unwrap().seq,
            seq,
            "an incomplete restore keeps the entry, surfaced as Needs Attention"
        );
    }

    /// ADR-0001/0002: the mark survives rescans and clears only when a restore verifies.
    #[test]
    fn a_failed_restore_is_needs_attention_until_a_verified_restore() {
        use crate::tweaks::snapshot::AttentionReason;
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        h.kind.drive_plan("s1", DrivePlan::Err);
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let err = run_restore(&t, &c, &h.deps()).expect_err("the reapply drive fails");
        assert!(!err.to_string().contains("fully verified"), "{err}");
        for _ in 0..2 {
            let attention = detect::detect(&t, &c, &h.deps())
                .attention
                .expect("still Needs Attention");
            assert_eq!(attention.reason, AttentionReason::RestoreFailed);
        }

        h.kind.drive_plan.lock().unwrap().clear();
        let outcome = run_restore(&t, &c, &h.deps()).expect("the retry verifies");
        assert_eq!(outcome.status.attention, None);
        assert_eq!(detect::detect(&t, &c, &h.deps()).attention, None);
    }

    /// ADR-0002: a restore that verified every effect is not a failed restore just because the
    /// store could not settle the spent entry; the entry is kept and the outcome recorded as itself.
    #[test]
    fn a_verified_restore_whose_entry_cannot_be_removed_is_not_a_failed_restore() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Manual));
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Disabled)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let seq = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        // A handle sharing read but not delete, taken mid-drive once the drive mark is written:
        // `consume` fails with a sharing violation.
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
        let path = h
            ._tmp
            .path()
            .join("demo")
            .join(format!("{:020}.json", seq.0));
        let held: Arc<Mutex<Option<std::fs::File>>> = Arc::default();
        let (hold_path, holder) = (path.clone(), held.clone());
        *h.kind.on_drive.lock().unwrap() = Some(Box::new(move || {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(&hold_path)
                .unwrap();
            holder.lock().unwrap().get_or_insert(file);
        }));

        let outcome = run_restore(&t, &c, &h.deps()).expect("every effect verified");
        assert_eq!(
            outcome.consumed, None,
            "its drive mark could not be settled"
        );
        assert_eq!(
            outcome.status.attention.map(|a| a.reason),
            Some(AttentionReason::OutcomeUnrecorded)
        );
        assert!(path.exists(), "the entry is kept, never silently lost");
        assert!(held.lock().unwrap().take().is_some(), "the drive ran");
        // The locked entry kept its mark, so the verified outcome is recorded now, as itself.
        assert_eq!(open_drives(&h), 1);
        let reason = |h: &Harness| {
            detect::detect(&t, &c, &h.deps())
                .attention
                .map(|a| a.reason)
        };
        assert_eq!(reason(&h), Some(AttentionReason::OutcomeUnrecorded));
        assert_eq!(
            crash_reason(&h),
            Some(AttentionReason::OutcomeUnrecorded),
            "the next launch must not call it a crash"
        );

        *h.kind.on_drive.lock().unwrap() = None;
        let outcome = run_restore(&t, &c, &h.deps()).expect("unlocked, the retry settles");
        assert_eq!(outcome.consumed, Some(seq));
        assert_eq!(open_drives(&h), 0);
        assert_eq!(crash_reason(&h), None);
    }

    #[test]
    fn dangling_ref_skipped_and_surfaced_not_restored() {
        let h = Harness::new();
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Manual)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        // "Ghost" is not a defined option -- classifies DanglingRef, `head` must skip it.
        let seq = h
            .snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("Ghost".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let outcome = run_restore(&t, &c, &h.deps()).expect("nothing valid to restore");
        assert_eq!(outcome.consumed, None);
        assert_eq!(outcome.skipped_invalid.len(), 1);
        assert_eq!(outcome.skipped_invalid[0].seq, seq);
        assert!(matches!(
            outcome.skipped_invalid[0].validity,
            EntryValidity::Invalid(InvalidReason::DanglingRef)
        ));
        assert!(
            h.log().iter().all(|op| !matches!(op, Op::Drive(_))),
            "a dangling entry must never be restored from"
        );
    }

    #[test]
    fn walk_to_empty_history_reads_system_default() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Automatic)); // drift: matches no option
        let t = tweak(
            "demo",
            vec![svc_effect("s1", false)],
            vec![opt(
                "A",
                vec![("s1", set(Value::Startup(StartupType::Manual)))],
            )],
        );
        let c = corpus(vec![t.clone()], vec![]);
        let mut map = BTreeMap::new();
        map.insert(
            EffectId("s1".into()),
            Value::Startup(StartupType::Automatic),
        );
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(map),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let first = run_restore(&t, &c, &h.deps()).expect("first restore consumes the only entry");
        assert!(first.consumed.is_some());

        let second = run_restore(&t, &c, &h.deps()).expect("nothing left to restore");
        assert_eq!(second.consumed, None);
        assert_eq!(second.status.state, TweakState::SystemDefault);
    }

    #[test]
    fn claims_recomputed_like_apply() {
        let h = Harness::new();
        h.kind.seed("sh_addr", Value::Startup(StartupType::Manual)); // the true original
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(StartupType::Disabled),
        };
        // This tweak currently holds the claim, as if it is standing on an option that claims.
        h.claims
            .claim(&shared, "demo", &h.kind, &ExecCx::new(Level::User))
            .unwrap();

        let t = tweak(
            "demo",
            vec![shared_effect("sh_eff", "sh")],
            vec![opt("A", vec![("sh_eff", ModelOptValue::Unclaimed(None))])],
        );
        let c = corpus(vec![t.clone()], vec![shared]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::OptionRef("A".into()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert!(
            !h.claims.is_claimed(&SharedId("sh".into())).unwrap(),
            "restoring to an unclaiming target must release the shared setting"
        );
        assert_eq!(
            h.kind.live_value("sh_addr"),
            Value::Startup(StartupType::Manual),
            "last release restores the captured original unconditionally"
        );
    }

    /// The `Captured::Values` path (a first apply on an untouched machine) must release a held
    /// Shared claim, or the shared value never returns to its original. The OptionRef path is
    /// `claims_recomputed_like_apply`.
    #[test]
    fn values_dump_restore_also_releases_a_held_shared_claim() {
        let h = Harness::new();
        h.kind.seed("sh_addr", Value::Startup(StartupType::Manual)); // the true original
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(StartupType::Disabled),
        };
        // "demo" currently holds the claim, as if its own apply drove it there -- but the entry
        // being restored is a plain Values dump, never an OptionRef.
        h.claims
            .claim(&shared, "demo", &h.kind, &ExecCx::new(Level::User))
            .unwrap();

        let t = tweak(
            "demo",
            vec![shared_effect("sh_eff", "sh")],
            vec![opt("A", vec![("sh_eff", ModelOptValue::Claim(None))])],
        );
        let c = corpus(vec![t.clone()], vec![shared]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert!(
            !h.claims.is_claimed(&SharedId("sh".into())).unwrap(),
            "restoring a Values-dump snapshot must still release a shared claim this tweak holds"
        );
        assert_eq!(
            h.kind.live_value("sh_addr"),
            Value::Startup(StartupType::Manual),
            "the last release must drive the shared setting back to its true captured original"
        );
    }

    /// A tweak that never held the claim at all must be left completely untouched by a Values-dump
    /// restore -- `release_shared_claims` must not call `release` (which would error `NotHeld`) for
    /// a shared id it never claimed.
    #[test]
    fn values_dump_restore_leaves_an_unclaimed_shared_setting_alone() {
        let h = Harness::new();
        h.kind.seed("sh_addr", Value::Startup(StartupType::Manual));
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(StartupType::Disabled),
        };
        let t = tweak(
            "demo",
            vec![shared_effect("sh_eff", "sh")],
            vec![opt("A", vec![("sh_eff", ModelOptValue::Unclaimed(None))])],
        );
        let c = corpus(vec![t.clone()], vec![shared]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");
        assert!(!h.claims.is_claimed(&SharedId("sh".into())).unwrap());
    }

    /// A journaled Action's undo routes from the tweak's floor, never from what the app currently
    /// holds: `Deps.level` is the read ceiling (invariant 24), never a drive's level. `ti` is not
    /// tested here because validation forbids a ti-routed action outright.
    #[test]
    fn an_action_undo_routes_from_the_floor_not_the_run_level() {
        let h = Harness::new();
        let mut t = tweak("demo", vec![action_effect("a", true, false)], vec![]);
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: vec![JournalRow {
                        action_id: EffectId("a".into()),
                        intended: true,
                        completed: true,
                        resolved: false,
                        undo_back: false,
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert_eq!(
            h.actions.level_of("a_apply:undo"),
            Some(Level::Admin),
            "the undo must run at the tweak's floor, not at Deps.level"
        );
    }

    /// The release must happen, and at the Shared effect's own routed level: the setup claims at
    /// Admin, so the second drive is the release, and no second drive means none happened.
    #[test]
    fn a_values_dump_releases_a_shared_claim_at_its_routed_level() {
        let h = Harness::new();
        h.kind.seed("sh_addr", Value::Startup(StartupType::Manual));
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(StartupType::Disabled),
        };
        h.claims
            .claim(&shared, "demo", &h.kind, &ExecCx::new(Level::Admin))
            .unwrap();

        let mut sh = shared_effect("sh_eff", "sh");
        sh.elevation = Some(Level::Ti);
        let mut t = tweak(
            "demo",
            vec![sh],
            vec![opt("A", vec![("sh_eff", ModelOptValue::Claim(None))])],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![shared]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert_eq!(
            h.kind.drive_levels("sh_addr"),
            vec![Level::Admin, Level::Ti],
            "the setup claim drives at Admin; the release must then drive it back at Ti"
        );
    }

    /// Another tweak captured the original at Ti; this admin-floor tweak releases last, and the
    /// original must still go back at Ti, not at this tweak's own route.
    #[test]
    fn a_last_release_drives_the_original_at_the_level_it_was_captured_at() {
        let h = Harness::new();
        h.kind.seed("sh_addr", Value::Startup(StartupType::Manual));
        let shared = SharedDef {
            id: SharedId("sh".into()),
            setting: Setting::Service(SvcAddr {
                name: "sh_addr".into(),
            }),
            value: Value::Startup(StartupType::Disabled),
        };
        h.claims
            .claim(&shared, "ti_tweak", &h.kind, &ExecCx::new(Level::Ti))
            .unwrap();
        h.claims
            .claim(&shared, "demo", &h.kind, &ExecCx::new(Level::Admin))
            .unwrap();
        h.claims
            .release(&shared.id, "ti_tweak", &h.kind, &ExecCx::new(Level::Ti))
            .unwrap();

        let mut t = tweak(
            "demo",
            vec![shared_effect("sh_eff", "sh")],
            vec![opt("A", vec![("sh_eff", ModelOptValue::Claim(None))])],
        );
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![shared]);
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert_eq!(
            h.kind.drive_levels("sh_addr"),
            vec![Level::Ti, Level::Ti],
            "the capture drove at Ti; the admin releaser must still drive the original back at Ti"
        );
    }

    /// Pins pre-existing behaviour, not this change: `drive_to_captured` already routed per effect,
    /// so a `ti` step's captured value is driven back at Ti.
    #[test]
    fn a_ti_setting_is_driven_back_at_ti() {
        let h = Harness::new();
        h.kind.seed("s1", Value::Startup(StartupType::Disabled));
        let mut t = tweak("demo", vec![ti_svc_effect("s1")], vec![]);
        t.elevation = Level::Admin;
        let c = corpus(vec![t.clone()], vec![]);
        let mut captured = BTreeMap::new();
        captured.insert(EffectId("s1".into()), Value::Startup(StartupType::Manual));
        h.snapshots
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(captured),
                    journal: Vec::new(),
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        run_restore(&t, &c, &h.deps()).expect("restore succeeds");

        assert_eq!(
            h.kind.last_drive_level("s1"),
            Some(Level::Ti),
            "a ti-declared setting must be driven back at Ti"
        );
    }

    /// The core §11 invariant: `apply(option)` then `restore()` of the just-captured entry returns
    /// the mock machine to the pre-(second-)apply state. Enumerated (not sampled) over every
    /// ordered pair of a small option set -- deterministic, and touches both the pure-Settings path
    /// (Opt0<->Opt2) and the action-run/undo-then-reapply path (any pair involving Opt1).
    #[test]
    fn apply_then_restore_property() {
        fn build_tweak() -> Tweak {
            tweak(
                "demo",
                vec![svc_effect("svc", false), action_effect("act", true, true)],
                vec![
                    opt(
                        "Opt0",
                        vec![("svc", set(Value::Startup(StartupType::Manual)))],
                    ),
                    opt(
                        "Opt1",
                        vec![
                            ("svc", set(Value::Startup(StartupType::Disabled))),
                            ("act", ModelOptValue::Run(None)),
                        ],
                    ),
                    opt(
                        "Opt2",
                        vec![("svc", set(Value::Startup(StartupType::Automatic)))],
                    ),
                ],
            )
        }
        let labels = ["Opt0", "Opt1", "Opt2"];

        for &start in &labels {
            for &target in &labels {
                if start == target {
                    continue;
                }
                let h = Harness::new();
                h.kind.seed("svc", Value::Startup(StartupType::Boot)); // matches no option
                let t = build_tweak();
                let c = corpus(vec![t.clone()], vec![]);
                let deps = h.deps();

                futures_block_on(apply::apply(&t, &c, &OptLabel(start.into()), &deps))
                    .unwrap_or_else(|e| panic!("apply({start}) failed: {e}"));
                let baseline_svc = h.kind.live_value("svc");
                let baseline_act_present = *h
                    .actions
                    .presence
                    .lock()
                    .unwrap()
                    .get("act_apply")
                    .unwrap_or(&false);

                futures_block_on(apply::apply(&t, &c, &OptLabel(target.into()), &deps))
                    .unwrap_or_else(|e| panic!("apply({start} -> {target}) failed: {e}"));

                let outcome = run_restore(&t, &c, &deps)
                    .unwrap_or_else(|e| panic!("restore({start} -> {target}) failed: {e}"));
                assert!(
                    outcome.consumed.is_some(),
                    "{start} -> {target}: restore must consume the just-captured entry"
                );
                assert_eq!(
                    h.kind.live_value("svc"),
                    baseline_svc,
                    "{start} -> {target}: restored Setting must match the pre-second-apply state"
                );
                let act_present_after = *h
                    .actions
                    .presence
                    .lock()
                    .unwrap()
                    .get("act_apply")
                    .unwrap_or(&false);
                assert_eq!(
                    act_present_after, baseline_act_present,
                    "{start} -> {target}: restored action presence must match the pre-second-apply state"
                );
            }
        }
    }
}
