//! Restore Snapshot (spec §8.5, +§8.6; ADR-0002/0003/0007). Thin by design: this module owns only
//! the controller sequencing -- undo the head entry's completed actions in reverse, then re-apply
//! its target -- and composes [`apply`]'s already-reviewed drive/verify primitives for every actual
//! mutation. No new drive/verify logic is written here.
//!
//! ## The steps (spec §8.5), mirrored from `apply.rs`'s own numbering
//! 0. **Lock; find the head entry.** [`lifecycle::lock_tweak`] serializes against a concurrent
//!    apply/restore of the same tweak (spec §8.7). [`SnapshotStore::head`] already skips
//!    invalid/dangling entries (ADR-0002) -- if none remain, the surface simply reads as System
//!    Default (ADR-0003): nothing to restore, nothing consumed. Every entry `head` skipped as
//!    invalid is still surfaced (via `list`) as `skipped_invalid`, never silently dropped.
//! 1. **Undo the entry's completed journal actions, in reverse order** -- these are exactly the
//!    actions that ran when the user *left* the state this entry captured (ADR-0007). Reuses
//!    [`apply::verify_reversed_probe`] verbatim for the same did-it-work discipline apply's own
//!    rollback uses; a completed no-undo action is reported un-undoable (incomplete), never fatal to
//!    the rest of the walk.
//! 2. **Re-apply the target**, re-derived from the *current* corpus (ADR-0007), never the possibly
//!    stale `tweak` parameter:
//!    - `Captured::OptionRef` -- [`apply::drive_to_captured`] drives its Settings (it re-derives
//!      internally too); a small declaration-order-preserving surface (Shared + Action effects,
//!      ephemerals included) is then driven via [`apply::drive_forward`] verbatim, exactly like a
//!      fresh apply of that option minus its own snapshot capture. Driven with `DriveCtx { journal:
//!      Journaling::None, .. }` -- no on-disk WAL entry is pushed for this drive pass (review fix,
//!      §Fix 1 below): the entry being restored is never consumed until step 4 verifies the WHOLE
//!      restore, so a crash mid-re-apply just leaves that entry on disk and the caller retries.
//!    - `Captured::Values` -- `drive_to_captured` for Settings, plus [`release_shared_claims`] for
//!      any Shared effect this tweak currently holds (review fix, see below); scripts cannot be
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
//! ## Fix 1 (post-review): no throwaway snapshot entry
//! An earlier revision pushed a throwaway `Captured::Values({})` entry purely to give
//! `drive_action`'s hardcoded `mark_completed(tweak_id, seq, ..)` call somewhere durable to write,
//! then discarded it right after driving. That was a reviewed CRITICAL: the throwaway always took
//! the *next* monotonic seq (above the real entry still on disk), so a crash between push and
//! discard -- or a failed discard, previously only `log::warn!`ed and never surfaced as a failure --
//! would let the phantom win `head()` on the next call. Its journal carried the target option's own
//! action rows, so a later restore would undo actions that were legitimately applied and "restore"
//! to an empty map; a completed action with no `undo` among them would then keep the phantom
//! forever (consume is correctly gated on empty failures), permanently masking the real
//! return-point -- an ADR-0002 stranding. The fix makes the WAL bookkeeping itself optional
//! ([`apply::Journaling`]) instead of routing around it: restore's re-apply genuinely does not need
//! per-action crash journaling (see step 2 above), so it now drives with `Journaling::None` and
//! pushes nothing at all. `option_ref_reapply_pushes_no_extra_entry` pins this directly.
//!
//! ## Fix 2 (Task 15, E2E-discovered): a Values-dump restore now also releases held shared claims
//! `Captured::Values` only ever arises when the pre-apply state matched no authored option, and
//! claiming a shared setting only ever happens by standing on a `Claim`-valued option -- so at the
//! moment this dump was captured, this tweak could not yet have been a claimant of any Shared
//! effect on its surface. Reverting to that moment must therefore give up whatever claim the
//! tweak took since (spec §8.5: "restore recomputes shared claims exactly as an ordinary apply of
//! the target state would," §8.6). Before this fix, the Values branch drove Settings back but left
//! Shared effects completely untouched (only the `Captured::OptionRef` branch's `drive_forward`
//! call ever reached them) -- a real, permanently-leaked claim (and a shared value that never
//! returns to its true original) for the — common — case of a tweak whose very first apply claims
//! a shared setting straight from a never-touched machine. [`release_shared_claims`] closes this:
//! it releases every Shared effect this tweak currently holds, mirroring `apply::drive_shared`'s
//! own `Unclaimed`-release logic minus the option lookup (there is no option to consult here).

use std::collections::BTreeSet;

use super::apply::attention_item;
use crate::tweaks::model::{
    ActionDef, Corpus, Effect, EffectDef, EffectId, OptLabel, OptValue, SharedId, Tweak,
};
use crate::tweaks::shared_claims::ReleaseOutcome;
use crate::tweaks::snapshot::{
    Attention, AttentionReason, Captured, EntrySummary, EntryValidity, JournalRow, Seq,
};
use crate::tweaks::validate::{applicable_surface, option_unavailable, Milestone};
use crate::tweaks::winver::WinVer;

use super::apply::{self, ActionPlan, DriveCtx, DriveState, EngineError};
use super::detect::{self, HeldInfo, TweakState, TweakStatus, UnavailableOpt};
use super::{context, lifecycle, Deps, Phase};

/// `restore`'s result (controller decision 3): a fresh [`TweakStatus`] computed from this
/// operation's own verify reads (grill Q1 -- no re-scan), which entry (if any) this restore
/// consumed, the reboot advisory for a Values-dump restore, and every invalid/dangling entry `head`
/// bypassed (surfaced, never silently dropped -- ADR-0002). `status.has_history`/`status.held_shared`
/// already carry the "further entry remains" / "held by" notices, mirroring `ApplyOutcome`'s own
/// shape rather than duplicating them at the top level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub status: TweakStatus,
    /// `Some(seq)` iff a valid entry existed and was restored + consumed; `None` means there was
    /// nothing to restore (ADR-0003: the surface already reads System Default).
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

fn do_restore(tweak: &Tweak, corpus: &Corpus, deps: &Deps) -> Result<RestoreOutcome, EngineError> {
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

    let mut failures: Vec<EngineError> = Vec::new();

    // Step 1: undo the entry's completed journal actions, in reverse order.
    let mut accounted = undo_journal(current_tweak, corpus, &entry.journal, deps, &mut failures);

    // Step 2 (+ step 3, folded in via `drive_forward`'s own Shared handling): re-apply the target.
    let mut reboot_advisory = false;
    let mut held_shared: Vec<HeldInfo> = Vec::new();
    let mut residues: Vec<EffectId> = Vec::new();
    let restored_label = match &entry.captured {
        Captured::OptionRef(label) => {
            let result = reapply_option_ref(current_tweak, corpus, label, milestone, &winver, deps);
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
            release_shared_claims(
                current_tweak,
                corpus,
                &milestone,
                deps,
                &mut held_shared,
                &mut failures,
            );
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
        if let Err(e) =
            deps.snapshots
                .set_attention(&current_tweak.id, deps.machine_guid, attention)
        {
            store.push(EngineError::AttentionWrite(e));
        }
        return Err(EngineError::RestoreFailed { failures, store });
    }
    // A verified restore accounts for the actions it undid and re-drove, and for no other row: a
    // `Values` head carries no re-drive, so residue it never probed stays for the next crash scan.
    apply::resolve_accounted(deps, &current_tweak.id, &accounted);
    if let Err(e) = deps
        .snapshots
        .clear_attention(&current_tweak.id, deps.machine_guid)
    {
        log::warn!(
            "tweak '{}': could not clear Needs Attention: {e}",
            current_tweak.id
        );
    }
    if let Err(e) = deps.snapshots.consume(&current_tweak.id, entry.seq) {
        // Every effect verified, so this is not a failed restore and nothing needs attention: the
        // machine is restored and only the spent return point outlived it.
        return Err(EngineError::EntryCleanup(e));
    }

    let (has_history, attention) = detect::history(&current_tweak.id, corpus, deps);
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
        consumed: Some(entry.seq),
        reboot_advisory,
        skipped_invalid,
    })
}

/// Step 1 (spec §8.5): undoes `journal`'s completed rows in reverse declaration order -- the
/// actions that ran when the user left the state now being restored to (ADR-0007). Reuses
/// [`apply::verify_reversed_probe`] verbatim for the did-it-work check; a completed action with no
/// `undo` is reported un-undoable (incomplete), never fatal to the rest of the walk. Returns the
/// rows it drove back and verified, which is what a verified restore may resolve.
fn undo_journal(
    tweak: &Tweak,
    corpus: &Corpus,
    journal: &[JournalRow],
    deps: &Deps,
    failures: &mut Vec<EngineError>,
) -> BTreeSet<EffectId> {
    let mut undone = BTreeSet::new();
    for row in journal.iter().rev().filter(|r| r.completed) {
        let Some((effect, action_def)) = find_action(tweak, &row.action_id) else {
            failures.push(EngineError::Invalid(format!(
                "completed action '{}' vanished from the surface during restore",
                row.action_id
            )));
            continue;
        };
        if is_ephemeral(action_def) {
            // Belt-and-suspenders (review fix): apply's Step 2 no longer journals an ephemeral
            // action at all, so this row should be unreachable in practice -- but an ephemeral is
            // exempt from ALL reversibility bookkeeping (spec §7, invariant 10): skip it, never
            // report it un-undoable and strand the entry.
            continue;
        }
        if !has_undo(action_def) {
            log::warn!(
                "tweak '{}': completed action '{}' has no undo -- reported un-undoable, restore incomplete",
                tweak.id, row.action_id
            );
            failures.push(EngineError::NoUndo(row.action_id.clone()));
            continue;
        }
        let cx = context::route(effect, tweak, corpus);
        match deps.actions.undo(action_def, &cx) {
            Ok(()) => {
                let before = failures.len();
                apply::verify_reversed_probe(action_def, effect, false, corpus, deps, failures);
                if failures.len() == before {
                    undone.insert(row.action_id.clone());
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
    milestone: Milestone,
    winver: &WinVer,
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
        if matches!(raw, Some(OptValue::Run(_))) {
            action_plan.push((effect.id.clone(), ActionPlan::Apply));
            continue;
        }
        if let ActionDef::Script {
            probe: Some(_),
            undo,
            ..
        } = action_def
        {
            match deps
                .probes
                .probe(action_def, &context::read_route(effect, deps.level, corpus))
            {
                Ok(true) if undo.is_some() => {
                    action_plan.push((effect.id.clone(), ActionPlan::UndoBack));
                }
                Ok(true) => residues.push(effect.id.clone()),
                Ok(false) => {}
                Err(e) => {
                    failures.push(EngineError::CaptureFailed {
                        effect: effect.id.clone(),
                        source: e,
                    });
                    plan_failed = true;
                }
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

    // No on-disk WAL entry is pushed for this drive pass (review fix, was a CRITICAL): the entry
    // being restored is never consumed until the WHOLE restore verifies (Step 4), so a crash
    // mid-re-apply just leaves that entry on disk and the caller retries the entire restore -- the
    // throwaway `Values({})` entry this used to fabricate purely to satisfy `drive_action`'s
    // hardcoded completion-mark call was itself an ADR-0002 stranding risk: it always took the NEXT
    // monotonic seq (above the real entry still on disk), so a crash between push and discard, or a
    // failed discard (previously only `log::warn!`ed, never surfaced), would let the phantom win
    // `head()` -- undoing legitimately-applied actions and, if any lacked `undo`, staying kept
    // forever, permanently masking the real return-point. `Journaling::None` removes the need for
    // any entry at all.
    let ctx = DriveCtx {
        tweak,
        corpus,
        target_opt,
        milestone,
        deps,
        journal: apply::Journaling::None,
    };
    let mut state = DriveState::default();
    if let Err(e) = apply::drive_forward(&ctx, &surface, &action_plan, &mut state) {
        failures.push(e);
    }
    let driven_actions = state.driven_actions();
    held_shared.extend(state.held_shared);

    OptionRefResult {
        failures,
        held_shared,
        residues,
        driven_actions,
    }
}

/// Releases every Shared effect on `tweak`'s applicable surface that `tweak` currently holds a
/// claim on (Fix 2, see this file's module docs) -- the `Captured::Values` restore path's
/// equivalent of what `apply::drive_shared`'s `Unclaimed` arm does for an ordinary apply, minus the
/// option lookup: a Values dump carries no target answer for a Shared effect at all, and the only
/// sound interpretation of "the state before this apply" for one is "not claiming," since claiming
/// only ever happens by standing on an authored `Claim` option. A tweak that never held a
/// particular shared id is left untouched (nothing to release).
fn release_shared_claims(
    tweak: &Tweak,
    corpus: &Corpus,
    milestone: &Milestone,
    deps: &Deps,
    held_shared: &mut Vec<HeldInfo>,
    failures: &mut Vec<EngineError>,
) {
    for effect in applicable_surface(tweak, milestone) {
        let Effect::Shared(shared_id) = &effect.kind else {
            continue;
        };
        if !currently_holds(deps, shared_id, &tweak.id) {
            continue;
        }
        let cx = context::route(effect, tweak, corpus);
        match deps.claims.release(shared_id, &tweak.id, deps.kinds, &cx) {
            Ok(ReleaseOutcome::StillHeld(holders)) => held_shared.push(HeldInfo {
                shared: shared_id.clone(),
                holders,
            }),
            Ok(ReleaseOutcome::RestoredOriginal) => {}
            Err(e) => failures.push(EngineError::Claim {
                shared: shared_id.clone(),
                source: e,
            }),
        }
    }
}

fn currently_holds(deps: &Deps, shared_id: &SharedId, tweak_id: &str) -> bool {
    deps.claims.holders(shared_id).iter().any(|h| h == tweak_id)
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

/// Mirrors `apply::is_ephemeral` (private there too -- duplicated here for the same reason
/// `find_action`/`has_undo` are: a pure lookup, not the drive/verify logic this task reuses
/// verbatim). Exempt from ALL reversibility bookkeeping (spec §7, invariant 10).
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
/// (grill Q1: no fresh `detect` re-scan) -- the fuller "authors a real value against a live Missing
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
    use crate::tweaks::kinds::{BatchFailure, EffectKind, Error as KindError, ExecCx};
    use crate::tweaks::model::{
        Level, OptValue as ModelOptValue, Probe, RiskLevel, ScopedValue, Script, Setting,
        SharedDef, SharedId, Shell, StartupType, SvcAddr, Value,
    };
    use crate::tweaks::shared_claims::ClaimsStore;
    use crate::tweaks::snapshot::{is_outstanding, InvalidReason, NewEntry, SnapshotStore};
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

    enum DrivePlan {
        Err,
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
                BatchFail::CouldNotAcquire => {
                    KindError::CouldNotAcquireElevation(Level::Ti, "mock: TI unavailable".into())
                }
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
            self.log.lock().unwrap().push(Op::Drive(key.clone()));
            self.levels.lock().unwrap().push((key.clone(), cx.level()));
            match self.drive_plan.lock().unwrap().get(&key) {
                Some(DrivePlan::Err) => Err(KindError::Backend("mock drive failure".into())),
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
        fn drive_batch(
            &self,
            items: &[(&Setting, &Value)],
            cx: &ExecCx,
        ) -> Result<(), BatchFailure> {
            self.batches.lock().unwrap().push(items.len());
            if let Some(verdict) = self.take_verdict() {
                for (setting, target) in &items[..verdict.completed.min(items.len())] {
                    self.drive(setting, target, cx)
                        .expect("a scripted completed drive must pass");
                }
                return Err(BatchFailure {
                    index: verdict.index,
                    error: verdict.fail.error(),
                    completed: verdict.completed,
                });
            }
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
                        },
                        JournalRow {
                            action_id: EffectId("b".into()),
                            intended: true,
                            completed: true,
                            resolved: false,
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

    /// Fix 1 regression (reviewed CRITICAL): an OptionRef restore that runs an action must push
    /// NO snapshot entry of its own -- no throwaway `Values({})` WAL vehicle, nothing left behind
    /// beyond consuming the entry it restored. A decoy entry proves the store's total count only
    /// ever shrinks by exactly the consumed entry, never grows.
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

    /// Fix 2 regression: an entry whose journal was populated by a REAL apply that ran an
    /// ephemeral action (not a hand-fabricated `journal: Vec::new()` entry, which is exactly why
    /// the earlier ephemeral tests missed this) must not be stranded on restore -- the ephemeral is
    /// excluded from the journal at the source, so `undo_journal` never even sees a row for it.
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
                    source: KindError::CouldNotAcquireElevation(Level::Ti, _),
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
    /// store could not release the spent entry. The machine is restored, so nothing is marked.
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

        // A handle sharing read but not delete: `consume` fails with a sharing violation.
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
        let path = h
            ._tmp
            .path()
            .join("demo")
            .join(format!("{:020}.json", seq.0));
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&path)
            .unwrap();

        let err = run_restore(&t, &c, &h.deps()).expect_err("the entry cannot be removed");
        assert!(matches!(err, EngineError::EntryCleanup(_)), "{err}");
        assert!(!err.to_string().contains("restore failed"), "{err}");
        assert_eq!(detect::detect(&t, &c, &h.deps()).attention, None);
        assert!(path.exists(), "the entry is kept, never silently lost");
        drop(held);
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
            !h.claims.is_claimed(&SharedId("sh".into())),
            "restoring to an unclaiming target must release the shared setting"
        );
        assert_eq!(
            h.kind.live_value("sh_addr"),
            Value::Startup(StartupType::Manual),
            "last release restores the captured original unconditionally"
        );
    }

    /// Fix 2 (Task 15, E2E-discovered): unlike `claims_recomputed_like_apply` above (which pushes
    /// a `Captured::OptionRef` entry directly), this pins the `Captured::Values` dump path -- the
    /// shape every tweak's very first-ever apply produces on a never-touched machine. Before the
    /// fix, restoring a Values dump drove Settings back but left a held Shared claim completely
    /// untouched (only the OptionRef branch's `drive_forward` call ever reached Shared effects),
    /// permanently leaking the claim and stranding the shared value away from its true original.
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
            !h.claims.is_claimed(&SharedId("sh".into())),
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
        assert!(!h.claims.is_claimed(&SharedId("sh".into())));
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
