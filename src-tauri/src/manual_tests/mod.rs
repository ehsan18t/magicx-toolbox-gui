//! Manual Tests for the `test-build` feature: real-machine checks run from the app's UI.
//! To add a test, write its `fn(&Ctx) -> Verdict` and add one entry to [`TESTS`].
//! Production paths only: nothing here runs inside the elevated broker child.

mod cases;
mod errors;
mod runner;

pub use runner::{cancel, record_batch, run, Ctx, ManualTestReport, Verdict};

use serde::Serialize;

pub struct ManualTest {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Plain words for the user: what running it changes on this PC.
    pub changes: &'static str,
    /// Puts the Run button behind a confirmation.
    pub changes_system: bool,
    /// `Some(default)` shows a duration field, in minutes.
    pub minutes: Option<u32>,
    pub run: fn(&Ctx) -> Verdict,
}

pub const TESTS: &[ManualTest] = &[
    ManualTest {
        id: "baseline_block_update_pipeline",
        title: "Baseline: Block the Windows Update pipeline",
        description: "Reads every effect of block_update_pipeline through the engine's own reads and keeps it as a baseline.",
        changes: "Nothing. Read-only.",
        changes_system: false,
        minutes: None,
        run: cases::baseline,
    },
    ManualTest {
        id: "ti_batch_timing",
        title: "TrustedInstaller batch timing",
        description: "Applies block_update_pipeline, times each TrustedInstaller batch against the 30 s budget, restores it and compares with the baseline.",
        changes: "Blocks Windows Update (services, tasks, policies) for a few seconds, then restores every effect from the snapshot.",
        changes_system: true,
        minutes: None,
        run: cases::ti_batch_timing,
    },
    ManualTest {
        id: "waasmedic_watch",
        title: "WaaSMedic watch",
        description: "Applies block_update_pipeline, polls every effect once a minute for drift, then restores it and compares with the baseline.",
        changes: "Blocks Windows Update for the whole watch (default 15 minutes), then restores every effect from the snapshot.",
        changes_system: true,
        minutes: Some(15),
        run: cases::waasmedic_watch,
    },
    ManualTest {
        id: "waasmedic_task_read",
        title: "WaaSMedic task read",
        description: "Reads the PerformRemediation and DeferredWork tasks through the scheduler service as this process.",
        changes: "Nothing. Read-only.",
        changes_system: false,
        minutes: None,
        run: cases::waasmedic_task_read,
    },
];

#[derive(Debug, Clone, Serialize)]
pub struct ManualTestView {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub changes: &'static str,
    pub changes_system: bool,
    pub minutes: Option<u32>,
}

pub fn list() -> Vec<ManualTestView> {
    TESTS
        .iter()
        .map(|t| ManualTestView {
            id: t.id,
            title: t.title,
            description: t.description,
            changes: t.changes,
            changes_system: t.changes_system,
            minutes: t.minutes,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::TESTS;
    use std::collections::HashSet;

    #[test]
    fn the_registry_lists_the_initial_tests_with_unique_ids() {
        let ids: Vec<&str> = TESTS.iter().map(|t| t.id).collect();
        for id in [
            "baseline_block_update_pipeline",
            "ti_batch_timing",
            "waasmedic_watch",
            "waasmedic_task_read",
        ] {
            assert!(ids.contains(&id), "{id} is missing");
        }
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
    }

    #[test]
    fn no_user_facing_text_uses_an_em_dash() {
        for t in TESTS {
            for text in [t.title, t.description, t.changes] {
                assert!(!text.contains('\u{2014}'), "{}: {text}", t.id);
            }
        }
    }

    #[test]
    fn only_the_tests_that_change_the_system_ask_for_confirmation() {
        for t in TESTS {
            let read_only = t.changes == "Nothing. Read-only.";
            assert_eq!(t.changes_system, !read_only, "{}", t.id);
        }
    }
}
