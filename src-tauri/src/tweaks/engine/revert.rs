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
//!
//! `EngineError` is reused as-is (its variant set is closed to this file -- apply.rs's own body is
//! untouched): a restore failure is reported via the existing `RollbackReport{original,
//! rollback_failures}` shape, which structurally matches restore's own failure bundle (undo
//! failures + re-apply failures) even though its `Display` wording ("apply failed...") was written
//! for apply's rollback. Documented here rather than silently reused.

use crate::tweaks::kinds::ExecCx;
use crate::tweaks::model::{
    ActionDef, Corpus, Effect, EffectDef, EffectId, OptLabel, OptValue, SharedId, Tweak,
};
use crate::tweaks::shared_claims::ReleaseOutcome;
use crate::tweaks::snapshot::{Captured, EntrySummary, EntryValidity, JournalRow, Seq};
use crate::tweaks::validate::{applicable_surface, option_unavailable, Milestone};
use crate::tweaks::winver::WinVer;

use super::apply::{self, ActionPlan, DriveCtx, DriveState, EngineError};
use super::detect::{self, HeldInfo, TweakState, TweakStatus, UnavailableOpt};
use super::{lifecycle, Deps};

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
    let _guard = lifecycle::lock_tweak(&tweak.id).await;
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

    let cx = ExecCx::new(deps.level);
    let mut failures: Vec<EngineError> = Vec::new();

    // Step 1: undo the entry's completed journal actions, in reverse order.
    undo_journal(current_tweak, &entry.journal, &cx, deps, &mut failures);

    // Step 2 (+ step 3, folded in via `drive_forward`'s own Shared handling): re-apply the target.
    let mut reboot_advisory = false;
    let mut held_shared: Vec<HeldInfo> = Vec::new();
    let mut residues: Vec<EffectId> = Vec::new();
    let restored_label = match &entry.captured {
        Captured::OptionRef(label) => {
            let result =
                reapply_option_ref(current_tweak, corpus, label, milestone, &winver, &cx, deps);
            failures.extend(result.failures);
            held_shared = result.held_shared;
            residues = result.residues;
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
                &milestone,
                &cx,
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
    if failures.is_empty() {
        if let Err(e) = deps.snapshots.consume(&current_tweak.id, entry.seq) {
            failures.push(EngineError::SnapshotWrite(e));
        }
    }
    if !failures.is_empty() {
        let original = failures.remove(0);
        return Err(EngineError::RollbackReport {
            original: Box::new(original),
            rollback_failures: failures,
        });
    }

    let has_history = deps
        .snapshots
        .head(
            &current_tweak.id,
            corpus,
            deps.machine_guid,
            milestone.build,
        )
        .map(|e| e.is_some())
        .unwrap_or_else(|e| {
            log::warn!(
                "tweak '{}': snapshot history unreadable after restore: {e}",
                current_tweak.id
            );
            false
        });

    let status = TweakStatus {
        state: restored_label.map_or(TweakState::SystemDefault, TweakState::Active),
        unavailable: unavailable_options(current_tweak, &milestone),
        residues,
        has_history,
        held_shared,
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
/// `undo` is reported un-undoable (incomplete), never fatal to the rest of the walk.
fn undo_journal(
    tweak: &Tweak,
    journal: &[JournalRow],
    cx: &ExecCx,
    deps: &Deps,
    failures: &mut Vec<EngineError>,
) {
    for row in journal.iter().rev().filter(|r| r.completed) {
        let Some(action_def) = find_action(tweak, &row.action_id) else {
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
            failures.push(EngineError::Invalid(format!(
                "action '{}' ran and cannot be undone (no undo script) -- restore is incomplete",
                row.action_id
            )));
            continue;
        }
        match deps.actions.undo(action_def, cx) {
            Ok(()) => {
                apply::verify_reversed_probe(action_def, &row.action_id, false, cx, deps, failures)
            }
            Err(e) => failures.push(EngineError::ActionFailed {
                effect: row.action_id.clone(),
                source: e,
            }),
        }
    }
}

/// What re-applying an OptionRef target's non-Setting effects produced -- bundled so
/// [`reapply_option_ref`] stays under clippy's argument-count lint.
struct OptionRefResult {
    failures: Vec<EngineError>,
    held_shared: Vec<HeldInfo>,
    residues: Vec<EffectId>,
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
    cx: &ExecCx,
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
            match deps.probes.probe(action_def, cx) {
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
    held_shared.extend(state.held_shared);

    OptionRefResult {
        failures,
        held_shared,
        residues,
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
    milestone: &Milestone,
    cx: &ExecCx,
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
        match deps.claims.release(shared_id, &tweak.id, deps.kinds, cx) {
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

fn find_action<'a>(tweak: &'a Tweak, effect_id: &EffectId) -> Option<&'a ActionDef> {
    tweak
        .surface
        .iter()
        .find(|e| &e.id == effect_id)
        .and_then(|e| match &e.kind {
            Effect::Action(a) => Some(a),
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
    use crate::tweaks::kinds::{EffectKind, Error as KindError};
    use crate::tweaks::model::{
        Level, OptValue as ModelOptValue, RiskLevel, ScopedValue, Script, Setting, SharedDef,
        SharedId, Shell, StartupType, SvcAddr, Value,
    };
    use crate::tweaks::shared_claims::ClaimsStore;
    use crate::tweaks::snapshot::{InvalidReason, NewEntry, SnapshotStore};
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
    }

    #[derive(Default)]
    struct MockKind {
        log: Log,
        live: Mutex<HashMap<String, Value>>,
        drive_plan: Mutex<HashMap<String, DrivePlan>>,
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
        fn live_value(&self, name: &str) -> Value {
            self.live
                .lock()
                .unwrap()
                .get(name)
                .cloned()
                .unwrap_or(Value::Absent)
        }
    }

    impl EffectKind for MockKind {
        fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, KindError> {
            let key = setting_key(s);
            self.log.lock().unwrap().push(Op::Read(key.clone()));
            Ok(self.live_value(&key))
        }

        fn drive(&self, s: &Setting, target: &Value, _cx: &ExecCx) -> Result<(), KindError> {
            let key = setting_key(s);
            self.log.lock().unwrap().push(Op::Drive(key.clone()));
            match self.drive_plan.lock().unwrap().get(&key) {
                Some(DrivePlan::Err) => Err(KindError::Backend("mock drive failure".into())),
                None => {
                    self.live.lock().unwrap().insert(key, target.clone());
                    Ok(())
                }
            }
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
    }
    impl MockActions {
        fn new(log: Log, presence: Presence) -> Self {
            Self {
                log,
                presence,
                ..Default::default()
            }
        }
    }
    impl ActionRunner for MockActions {
        fn apply(&self, action: &ActionDef, _cx: &ExecCx) -> Result<(), KindError> {
            let key = action_key(action);
            self.log.lock().unwrap().push(Op::RunApply(key.clone()));
            self.presence.lock().unwrap().insert(key, true);
            Ok(())
        }
        fn undo(&self, action: &ActionDef, _cx: &ExecCx) -> Result<(), KindError> {
            let key = action_key(action);
            self.log.lock().unwrap().push(Op::RunUndo(key.clone()));
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
                probe: probe.then(|| Script(format!("{id}_probe"))),
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
                        },
                        JournalRow {
                            action_id: EffectId("b".into()),
                            intended: true,
                            completed: true,
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
                    }],
                },
                &c,
                Some("test-guid"),
                19045,
            )
            .unwrap();

        let err = run_restore(&t, &c, &h.deps()).expect_err("an un-undoable action must fail");
        assert!(matches!(err, EngineError::RollbackReport { .. }));

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
        assert!(matches!(err, EngineError::RollbackReport { .. }));
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
