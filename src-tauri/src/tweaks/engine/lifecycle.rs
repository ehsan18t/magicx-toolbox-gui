//! Per-tweak apply/restore locks (spec §8.7), the exit latch, and the crash-residue scan.
//!
//! One process-wide [`ApplyGate`] holds the locks and the exit state under a single mutex, so a
//! committed exit refuses every later lock and no exit commits while a lock is held.
//! [`scan_for_crash_residue`] flags the open drive, unfinished steps and outstanding rows a crash
//! left.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use crate::tweaks::model::EffectId;
use crate::tweaks::snapshot::{
    is_outstanding, Attention, AttentionItem, AttentionKind, AttentionReason, Entry, SnapshotStore,
};

/// Refused before anything was touched; a pending exit can still be called off, a final one cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AppExiting {
    #[error("The app is restarting as administrator or installing an update, so nothing was changed. If it stays open, try again.")]
    Pending,
    #[error(
        "The app is closing, so nothing was changed. Restart the app before changing settings."
    )]
    Final,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExitRefused {
    ApplyInFlight,
    Exiting(AppExiting),
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Exit {
    #[default]
    Open,
    Pending,
    Final,
}

#[derive(Default)]
struct GateState {
    locks: HashMap<String, Arc<AsyncMutex<()>>>,
    exit: Exit,
}

impl GateState {
    fn any_locked(&self) -> bool {
        self.locks.values().any(|arc| arc.try_lock().is_err())
    }

    fn refusal(&self) -> Option<AppExiting> {
        match self.exit {
            Exit::Open => None,
            Exit::Pending => Some(AppExiting::Pending),
            Exit::Final => Some(AppExiting::Final),
        }
    }
}

#[derive(Default)]
pub struct ApplyGate(Mutex<GateState>);

static GATE: OnceLock<ApplyGate> = OnceLock::new();

pub fn gate() -> &'static ApplyGate {
    GATE.get_or_init(ApplyGate::default)
}

impl ApplyGate {
    fn state(&self) -> MutexGuard<'_, GateState> {
        self.0.lock().expect("tweak-locks mutex poisoned")
    }

    /// Serializes one tweak's whole apply/restore (spec §8.7); other tweaks proceed concurrently.
    pub async fn lock_tweak(&self, tweak_id: &str) -> Result<OwnedMutexGuard<()>, AppExiting> {
        let arc = self
            .state()
            .locks
            .entry(tweak_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone();
        let guard = arc.lock_owned().await;
        // Checked while holding the lock, under the mutex `begin_exit` takes: either this sees the
        // latch or the exit sees this lock held. Checking before the await leaves a gap (stability).
        if let Some(refused) = self.state().refusal() {
            return Err(refused);
        }
        Ok(guard)
    }

    /// Starts a pending exit: every later [`Self::lock_tweak`] is refused until the latch drops or
    /// is kept (final). An exit mid-drive would leave a tweak partly applied under its snapshot.
    pub fn begin_exit(&self) -> Result<ExitLatch<'_>, ExitRefused> {
        let mut state = self.state();
        if let Some(exiting) = state.refusal() {
            return Err(ExitRefused::Exiting(exiting));
        }
        if state.any_locked() {
            return Err(ExitRefused::ApplyInFlight);
        }
        state.exit = Exit::Pending;
        Ok(ExitLatch(self))
    }

    /// Whether a window close may proceed; it then makes the exit final, which no latch drop undoes.
    /// A pending exit already rules out an apply in flight, so it never blocks the close.
    pub fn commit_close(&self) -> bool {
        let mut state = self.state();
        if state.exit == Exit::Open && state.any_locked() {
            return false;
        }
        state.exit = Exit::Final;
        true
    }
}

pub(crate) async fn lock_tweak(tweak_id: &str) -> Result<OwnedMutexGuard<()>, AppExiting> {
    gate().lock_tweak(tweak_id).await
}

/// Whether `tweak_id` is mid-apply or mid-restore. Non-blocking for the rayon detect sweep, which
/// skips such a tweak because its surface is half-driven. Racy: a best-effort filter, never a lock.
pub(crate) fn is_locked(tweak_id: &str) -> bool {
    let state = gate().state();
    state
        .locks
        .get(tweak_id)
        .is_some_and(|arc| arc.try_lock().is_err())
}

/// Calls off a pending exit on drop, so a declined UAC prompt or a failed download re-enables
/// applies. A final exit stays final.
#[must_use]
pub struct ExitLatch<'a>(&'a ApplyGate);

impl ExitLatch<'_> {
    pub fn keep_until_exit(self) {
        self.0.state().exit = Exit::Final;
    }
}

impl Drop for ExitLatch<'_> {
    fn drop(&mut self) {
        let mut state = self.0.state();
        if state.exit == Exit::Pending {
            state.exit = Exit::Open;
        }
    }
}

/// Flags an open drive, every interrupted action step and every still-outstanding row across a
/// tweak's whole history (spec §8.1, invariant 5). All are written before the work starts, so they
/// prove only that it was started and never settled -- never that it ran.
pub fn scan_for_crash_residue(entries: &[Entry]) -> Option<Attention> {
    let open_drive = entries
        .iter()
        .any(|entry| entry.drive_open)
        .then(|| AttentionItem {
            effect: None,
            kind: AttentionKind::CrashResidue,
            class: None,
            message: "a change to this tweak was never recorded as finished, so its settings may \
                      be only partly changed; the app may have stopped mid-change"
                .to_string(),
        });
    let steps: BTreeSet<&EffectId> = entries
        .iter()
        .flat_map(|entry| entry.actions_in_flight.iter())
        .collect();
    let steps = steps.into_iter().map(|id| AttentionItem {
        effect: Some(id.clone()),
        kind: AttentionKind::CrashResidue,
        class: None,
        message: format!(
            "undoing or re-running action '{id}' was never recorded as finished, so it may be \
             only partly done"
        ),
    });
    let rows = entries
        .iter()
        .flat_map(|entry| entry.journal.iter())
        .filter(|row| is_outstanding(row))
        .map(|row| AttentionItem {
            effect: Some(row.action_id.clone()),
            kind: AttentionKind::CrashResidue,
            class: None,
            message: format!(
                "action '{}' was planned but never confirmed complete, so it may or may not have \
                 run: the app stopped mid-apply",
                row.action_id
            ),
        });
    let items: Vec<AttentionItem> = open_drive.into_iter().chain(steps).chain(rows).collect();
    (!items.is_empty()).then_some(Attention {
        reason: AttentionReason::CrashResidue,
        items,
    })
}

/// One tweak's share of the startup carry-forward (spec §8.1, invariant 5), so a crash reaches the
/// UI like any other kept failure (ADR-0001) instead of living only in the log. An existing record
/// of this build's gains the items it lacks, so a crash during a retry is not hidden behind it.
pub(crate) fn record_crash_residue(snapshots: &SnapshotStore, tweak_id: &str, guid: Option<&str>) {
    let entries = match snapshots.unresolved_entries(tweak_id, guid) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("tweak '{tweak_id}': snapshot history unreadable in the crash scan: {e}");
            return;
        }
    };
    let Some(found) = scan_for_crash_residue(&entries) else {
        return;
    };
    let attention = match snapshots.attention(tweak_id, guid) {
        Ok(None) => found,
        // Never written over: it is another build's or machine's, or unparseable.
        Ok(Some(existing)) if existing.reason == AttentionReason::RecordUnreadable => return,
        Ok(Some(mut existing)) => {
            // An `Unrecorded` item already says why the drive mark is still open.
            let explained = existing
                .items
                .iter()
                .any(|i| i.kind == AttentionKind::Unrecorded);
            let before = existing.items.len();
            for item in found.items {
                let drive_mark = item.effect.is_none();
                if !(drive_mark && explained) && !existing.items.contains(&item) {
                    existing.items.push(item);
                }
            }
            if existing.items.len() == before {
                return;
            }
            existing
        }
        Err(e) => {
            log::warn!("tweak '{tweak_id}': attention record unreadable: {e}");
            return;
        }
    };
    log::error!(
        "tweak '{tweak_id}' needs attention after a crash-interrupted change ({} item(s))",
        attention.items.len()
    );
    if let Err(e) = snapshots.set_attention(tweak_id, guid, attention) {
        log::error!("tweak '{tweak_id}': could not record Needs Attention: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::EffectId;
    use crate::tweaks::snapshot::{Captured, JournalRow, Seq};
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    fn entry_with_journal(journal: Vec<JournalRow>) -> Entry {
        Entry {
            schema_version: 1,
            machine_guid: None,
            tweak_id: "demo".into(),
            seq: Seq(7),
            timestamp: "t".into(),
            captured: Captured::Values(BTreeMap::new()),
            drive_open: false,
            actions_in_flight: Default::default(),
            journal,
        }
    }

    #[test]
    fn crash_window_simulation() {
        // An action "ran" (the mock recorded it) but the process dropped before `mark_completed`
        // durably flipped its row -- reopening the store must show `intended && !completed`, and
        // the recovery scan must flag it (spec §8.1 invariant 5), never silently skip it.
        let entry = entry_with_journal(vec![JournalRow {
            action_id: EffectId("flush_dns".into()),
            intended: true,
            completed: false,
            resolved: false,
        }]);

        let flagged = scan_for_crash_residue(&[entry]).expect("must flag crash residue");
        assert_eq!(flagged.reason, AttentionReason::CrashResidue);
        assert_eq!(flagged.items.len(), 1);
        assert_eq!(flagged.items[0].effect, Some(EffectId("flush_dns".into())));
        assert!(flagged.items[0].message.contains("flush_dns"));
        // The row is written before the action is driven, so the wording must not claim it ran.
        assert!(
            !flagged.items[0].message.contains("ran but"),
            "{}",
            flagged.items[0].message
        );
    }

    #[test]
    fn completed_journal_is_not_crash_residue() {
        let entry = entry_with_journal(vec![JournalRow {
            action_id: EffectId("flush_dns".into()),
            intended: true,
            completed: true,
            resolved: false,
        }]);
        assert!(scan_for_crash_residue(&[entry]).is_none());
    }

    /// The mark a verified apply or restore leaves on the row it accounted for: read back off the
    /// entry, it takes that row out of the scan without deleting anything.
    #[test]
    fn a_resolved_row_is_not_crash_residue() {
        let entry = entry_with_journal(vec![JournalRow {
            action_id: EffectId("flush_dns".into()),
            intended: true,
            completed: false,
            resolved: true,
        }]);
        assert!(scan_for_crash_residue(&[entry]).is_none());
    }

    #[test]
    fn an_open_drive_is_crash_residue_with_no_effect_named() {
        let entry = Entry {
            drive_open: true,
            ..entry_with_journal(Vec::new())
        };
        let flagged = scan_for_crash_residue(&[entry]).expect("an open drive must be flagged");
        assert_eq!(flagged.reason, AttentionReason::CrashResidue);
        assert_eq!(flagged.items.len(), 1);
        assert_eq!(flagged.items[0].effect, None);
        assert_eq!(flagged.items[0].kind, AttentionKind::CrashResidue);
    }

    /// Two interrupted drives say one thing; each outstanding row still names its own action.
    #[test]
    fn open_drives_raise_one_item_beside_the_action_rows() {
        let row = JournalRow {
            action_id: EffectId("act1".into()),
            intended: true,
            completed: false,
            resolved: false,
        };
        let open = |journal| Entry {
            drive_open: true,
            ..entry_with_journal(journal)
        };
        let flagged = scan_for_crash_residue(&[open(vec![row]), open(Vec::new())])
            .expect("both kinds flagged");
        let effects: Vec<_> = flagged.items.iter().map(|i| i.effect.clone()).collect();
        assert_eq!(effects, vec![None, Some(EffectId("act1".into()))]);
    }

    fn crashed_store(tmp: &std::path::Path) -> SnapshotStore {
        use crate::tweaks::model::Corpus;
        use crate::tweaks::snapshot::NewEntry;
        let store = SnapshotStore::open(tmp.to_path_buf());
        let empty = Corpus {
            categories: Vec::new(),
            tweaks: Vec::new(),
            shared: Vec::new(),
        };
        let new_entry = NewEntry {
            captured: Captured::Values(BTreeMap::new()),
            journal: Vec::new(),
        };
        let seq = store.push("demo", new_entry, &empty, Some("g"), 0).unwrap();
        store.open_drive("demo", seq).unwrap();
        store
    }

    fn apply_failed() -> Attention {
        Attention {
            reason: AttentionReason::ApplyFailed,
            items: vec![AttentionItem {
                effect: Some(EffectId("s1".into())),
                kind: AttentionKind::Drive,
                class: None,
                message: "s1 could not be driven".into(),
            }],
        }
    }

    /// A crash during a retry joins the earlier failure's record instead of hiding behind it, and
    /// a second launch adds nothing more.
    #[test]
    fn a_crash_joins_an_existing_record_once() {
        let tmp = tempfile::tempdir().unwrap();
        let store = crashed_store(tmp.path());
        store
            .set_attention("demo", Some("g"), apply_failed())
            .unwrap();

        record_crash_residue(&store, "demo", Some("g"));
        record_crash_residue(&store, "demo", Some("g"));
        let record = store.attention("demo", Some("g")).unwrap().unwrap();
        assert_eq!(record.reason, AttentionReason::ApplyFailed);
        let effects: Vec<_> = record.items.iter().map(|i| i.effect.clone()).collect();
        assert_eq!(effects, vec![Some(EffectId("s1".into())), None]);
    }

    #[test]
    fn another_machines_record_is_never_joined() {
        let tmp = tempfile::tempdir().unwrap();
        let store = crashed_store(tmp.path());
        store
            .set_attention("demo", Some("other"), apply_failed())
            .unwrap();

        record_crash_residue(&store, "demo", Some("g"));
        let theirs = store.attention("demo", Some("other")).unwrap().unwrap();
        assert_eq!(theirs, apply_failed());
    }

    /// An `Unrecorded` item already explains why the mark is open, so no crash is claimed for it.
    #[test]
    fn an_unrecorded_outcome_explains_the_open_mark() {
        let tmp = tempfile::tempdir().unwrap();
        let store = crashed_store(tmp.path());
        let explained = Attention {
            reason: AttentionReason::OutcomeUnrecorded,
            items: vec![AttentionItem {
                effect: None,
                kind: AttentionKind::Unrecorded,
                class: None,
                message: "could not record it".into(),
            }],
        };
        store
            .set_attention("demo", Some("g"), explained.clone())
            .unwrap();

        record_crash_residue(&store, "demo", Some("g"));
        assert_eq!(store.attention("demo", Some("g")).unwrap(), Some(explained));
    }

    #[test]
    fn empty_journal_is_not_crash_residue() {
        let entry = entry_with_journal(Vec::new());
        assert!(scan_for_crash_residue(&[entry]).is_none());
    }

    /// Residue can sit on a superseded entry, which `head` never reaches -- the scan takes the set.
    #[test]
    fn residue_is_found_on_every_entry_not_just_the_newest() {
        let newest = entry_with_journal(vec![JournalRow {
            action_id: EffectId("done".into()),
            intended: true,
            completed: true,
            resolved: false,
        }]);
        let superseded = entry_with_journal(vec![JournalRow {
            action_id: EffectId("stale".into()),
            intended: true,
            completed: false,
            resolved: false,
        }]);
        let flagged =
            scan_for_crash_residue(&[newest, superseded]).expect("the older row still counts");
        assert_eq!(flagged.items.len(), 1);
        assert_eq!(flagged.items[0].effect, Some(EffectId("stale".into())));
    }

    /// Two same-id holders on a multi-thread runtime, each sleeping on its own worker while holding
    /// the lock: their markers must form contiguous blocks, proving real exclusion.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn per_tweak_lock_serializes_same_tweak() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let log: Arc<Mutex<Vec<(u32, &'static str)>>> = Arc::new(Mutex::new(Vec::new()));
        let id = format!("lock_test_tweak_{}", COUNTER.fetch_add(1, Ordering::SeqCst));

        async fn holder(id: String, tag: &'static str, log: Arc<Mutex<Vec<(u32, &'static str)>>>) {
            let _guard = lock_tweak(&id).await.expect("no exit pending");
            log.lock().unwrap().push((0, tag));
            // A real cross-thread window: while this task holds the guard, another worker thread
            // attempting the same tweak id's lock must genuinely block, not merely lose a race.
            std::thread::sleep(Duration::from_millis(40));
            log.lock().unwrap().push((1, tag));
        }

        let a = tokio::spawn(holder(id.clone(), "A", log.clone()));
        let b = tokio::spawn(holder(id.clone(), "B", log.clone()));
        a.await.unwrap();
        b.await.unwrap();

        let log = log.lock().unwrap();
        assert_eq!(log.len(), 4, "both holders must record both markers");
        // Serialized means one tag's two markers are adjacent -- never A-start, B-start, A-end.
        let tags: Vec<&str> = log.iter().map(|(_, t)| *t).collect();
        assert!(
            (tags[0] == tags[1] && tags[2] == tags[3]) && tags[0] != tags[2],
            "expected two contiguous same-tag pairs from strictly sequential holders, got {tags:?}"
        );
    }

    /// Different tweak ids must NOT serialize against each other (spec §8.7: "different tweaks may
    /// run concurrently") -- two distinct ids' critical sections may interleave freely.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn different_tweak_ids_run_concurrently() {
        let start = std::time::Instant::now();
        let a = tokio::spawn(async {
            let _g = lock_tweak("distinct_tweak_a")
                .await
                .expect("no exit pending");
            std::thread::sleep(Duration::from_millis(150));
        });
        let b = tokio::spawn(async {
            let _g = lock_tweak("distinct_tweak_b")
                .await
                .expect("no exit pending");
            std::thread::sleep(Duration::from_millis(150));
        });
        a.await.unwrap();
        b.await.unwrap();
        assert!(
            start.elapsed() < Duration::from_millis(280),
            "distinct tweak ids must overlap, not serialize: took {:?}",
            start.elapsed()
        );
    }

    #[tokio::test]
    async fn exit_is_refused_while_a_tweak_is_locked() {
        let gate = ApplyGate::default();
        let guard = gate.lock_tweak("t").await.expect("no exit pending");
        assert_eq!(gate.begin_exit().err(), Some(ExitRefused::ApplyInFlight));
        drop(guard);
        assert!(gate.begin_exit().is_ok());
    }

    #[tokio::test]
    async fn committed_exit_refuses_applies_until_it_drops() {
        let gate = ApplyGate::default();
        let latch = gate.begin_exit().expect("nothing locked");
        assert_eq!(gate.lock_tweak("t").await.err(), Some(AppExiting::Pending));
        drop(latch);
        assert!(gate.lock_tweak("t").await.is_ok());
    }

    #[tokio::test]
    async fn kept_exit_latch_never_lifts() {
        let gate = ApplyGate::default();
        gate.begin_exit().expect("nothing locked").keep_until_exit();
        assert_eq!(gate.lock_tweak("t").await.err(), Some(AppExiting::Final));
    }

    #[test]
    fn second_exit_is_refused_while_one_is_pending() {
        let gate = ApplyGate::default();
        let _first = gate.begin_exit().expect("nothing locked");
        assert_eq!(
            gate.begin_exit().err(),
            Some(ExitRefused::Exiting(AppExiting::Pending))
        );
    }

    #[tokio::test]
    async fn close_is_blocked_only_by_an_apply_in_flight() {
        let gate = ApplyGate::default();
        let guard = gate.lock_tweak("t").await.expect("no exit pending");
        assert!(!gate.commit_close(), "a close must not kill an apply");
        drop(guard);
        assert!(gate.commit_close());
        assert_eq!(gate.lock_tweak("t").await.err(), Some(AppExiting::Final));
    }

    #[tokio::test]
    async fn a_close_stays_committed_after_a_pending_exit_is_called_off() {
        let gate = ApplyGate::default();
        let pending = gate.begin_exit().expect("nothing locked");
        assert!(
            gate.commit_close(),
            "a pending exit already rules out an apply"
        );
        drop(pending);
        assert_eq!(gate.lock_tweak("t").await.err(), Some(AppExiting::Final));
    }
}
