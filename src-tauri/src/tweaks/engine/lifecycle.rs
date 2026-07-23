//! Per-tweak concurrency (spec §8.7) and Needs-Attention assembly (ADR-0001/0002).
//!
//! **The lock.** "A per-tweak-id async lock spans check→capture→save→mutate→verify. Different
//! tweaks may run concurrently; the same tweak may not" (§8.7). `apply`'s own signature is fixed by
//! the brief to exactly `(tweak, corpus, target, deps)` — no extra handle to thread through — so
//! the lock table lives here as a process-wide static, keyed by tweak id, rather than as a `Deps`
//! field every call site (and every existing `detect`-only test) would otherwise need to grow.
//!
//! **Needs Attention.** Two distinct situations produce it, and neither is a detection verdict
//! (spec §8.4: "Needs Attention is not a detection verdict"): a live rollback that could not verify
//! every restore (ADR-0001, surfaced by `apply`'s own `Err(EngineError::RollbackReport{..})`), and
//! — the case this module scans for directly — a journal left `intended && !completed` by a
//! process that crashed between running an action and its completion mark being fsynced (spec §8.1,
//! invariant 5). [`scan_for_crash_residue`] is the recovery check a startup pass runs per entry.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use crate::tweaks::model::EffectId;
use crate::tweaks::snapshot::{Entry, Seq};

type LockTable = Mutex<HashMap<String, Arc<AsyncMutex<()>>>>;

static LOCKS: OnceLock<LockTable> = OnceLock::new();

fn locks() -> &'static LockTable {
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Acquires (creating on first use) `tweak_id`'s async lock, serializing the whole apply/restore
/// sequence for that one tweak while other tweaks proceed concurrently (spec §8.7). The map lookup
/// itself is a brief, never-held-across-`.await` `std::sync::Mutex` section; the returned owned
/// guard is what the caller actually holds across its async body.
pub(crate) async fn lock_tweak(tweak_id: &str) -> OwnedMutexGuard<()> {
    let arc = {
        let mut map = locks().lock().expect("tweak-locks mutex poisoned");
        map.entry(tweak_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    };
    arc.lock_owned().await
}

/// One tweak's snapshot entry left in a state that cannot be silently trusted (ADR-0001/0002):
/// named, exact unrecoverable items — never a guess, never a silent retry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedsAttention {
    pub tweak_id: String,
    pub seq: Seq,
    /// One line per unrecoverable item — a crash-left unmarked action, or a rollback restore that
    /// failed verification. Plain strings (not the underlying error types) so this stays trivially
    /// comparable in tests and displayable in the UI without re-deriving `PartialEq` through every
    /// wrapped `kinds`/`snapshot`/`shared_claims` error type.
    pub unrecoverable: Vec<String>,
}

impl NeedsAttention {
    /// Builds a `NeedsAttention` from a rollback's collected failures (ADR-0001) — one line per
    /// failure, in the order they occurred.
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

/// Scans one entry's journal for rows left `intended && !completed` (spec §8.1, invariant 5): the
/// exact signature a process crash between running an action and fsyncing its completion mark
/// leaves behind. `None` means the entry is not crash-residue (every intended action is marked, or
/// there is no journal at all — a pure Settings apply/restore never touches this path).
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

    /// Proves real mutual exclusion, not accidental non-preemption: a multi-thread runtime runs
    /// two `apply`-shaped tasks for the SAME tweak id concurrently, each holding the lock across a
    /// real (blocking, on its own worker thread) sleep while pushing ordered markers into a shared
    /// log. Without the lock, a second worker thread could interleave its markers into the first
    /// task's window; with it, one task's markers must appear as a contiguous block.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn per_tweak_lock_serializes_same_tweak() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let log: Arc<Mutex<Vec<(u32, &'static str)>>> = Arc::new(Mutex::new(Vec::new()));
        let id = format!("lock_test_tweak_{}", COUNTER.fetch_add(1, Ordering::SeqCst));

        async fn holder(id: String, tag: &'static str, log: Arc<Mutex<Vec<(u32, &'static str)>>>) {
            let _guard = lock_tweak(&id).await;
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
            let _g = lock_tweak("distinct_tweak_a").await;
            std::thread::sleep(Duration::from_millis(150));
        });
        let b = tokio::spawn(async {
            let _g = lock_tweak("distinct_tweak_b").await;
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
}
