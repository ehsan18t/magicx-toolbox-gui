//! Per-tweak apply/restore locks (spec §8.7), the exit latch, and the crash-residue scan.
//!
//! One process-wide [`ApplyGate`] holds the locks and the exit state under a single mutex, so a
//! committed exit refuses every later lock and no exit commits while a lock is held.
//! [`scan_for_crash_residue`] flags journal rows a crash mid-apply left `intended && !completed`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use crate::tweaks::model::EffectId;
use crate::tweaks::snapshot::{Entry, Seq};

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

/// One tweak's snapshot entry that cannot be silently trusted (ADR-0001/0002): exact
/// unrecoverable items, never a guess and never a silent retry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedsAttention {
    pub tweak_id: String,
    pub seq: Seq,
    /// One line per unrecoverable item (a crash-left unmarked action or a failed rollback
    /// restore). Plain strings keep it comparable in tests and displayable in the UI.
    pub unrecoverable: Vec<String>,
}

impl NeedsAttention {
    /// One line per rollback failure (ADR-0001), in the order they occurred.
    pub fn from_rollback_failures(
        tweak_id: &str,
        seq: Seq,
        failures: &[impl std::fmt::Display],
    ) -> Self {
        Self {
            tweak_id: tweak_id.to_string(),
            seq,
            unrecoverable: failures.iter().map(ToString::to_string).collect(),
        }
    }
}

/// Flags rows left `intended && !completed` (spec §8.1, invariant 5), the mark of a crash between
/// running an action and fsyncing its completion. `None` when every intended action is marked or
/// there is no journal (a pure Settings apply/restore).
pub fn scan_for_crash_residue(tweak_id: &str, entry: &Entry) -> Option<NeedsAttention> {
    let unmarked: Vec<EffectId> = entry
        .journal
        .iter()
        .filter(|row| row.intended && !row.completed)
        .map(|row| row.action_id.clone())
        .collect();
    if unmarked.is_empty() {
        return None;
    }
    Some(NeedsAttention {
        tweak_id: tweak_id.to_string(),
        seq: entry.seq,
        unrecoverable: unmarked
            .into_iter()
            .map(|id| {
                format!(
                    "action '{id}' ran but its completion was never durably marked -- the process \
                     likely crashed mid-apply"
                )
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::snapshot::{Captured, JournalRow};
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
        }]);

        let flagged = scan_for_crash_residue("demo", &entry).expect("must flag crash residue");
        assert_eq!(flagged.tweak_id, "demo");
        assert_eq!(flagged.seq, Seq(7));
        assert_eq!(flagged.unrecoverable.len(), 1);
        assert!(flagged.unrecoverable[0].contains("flush_dns"));
    }

    #[test]
    fn completed_journal_is_not_crash_residue() {
        let entry = entry_with_journal(vec![JournalRow {
            action_id: EffectId("flush_dns".into()),
            intended: true,
            completed: true,
        }]);
        assert!(scan_for_crash_residue("demo", &entry).is_none());
    }

    #[test]
    fn empty_journal_is_not_crash_residue() {
        let entry = entry_with_journal(Vec::new());
        assert!(scan_for_crash_residue("demo", &entry).is_none());
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
