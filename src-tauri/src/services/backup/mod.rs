//! # Snapshot-Based Backup Service
//!
//! Unified option-based backup system for Windows registry tweaks with atomic
//! rollback capabilities.
//!
//! ## Module Organization
//!
//! - `storage`: File I/O for snapshot persistence
//! - `capture`: State capture before applying tweaks
//! - `restore`: Atomic restore with rollback support
//! - `detection`: State detection and snapshot validation
//! - `helpers`: Parsing and comparison utilities

mod capture;
mod detection;
mod helpers;
mod restore;
mod storage;

// Re-export public items from submodules
pub use capture::{capture_current_state, capture_snapshot};
pub use detection::{cleanup_old_backups, detect_tweak_state, validate_all_snapshots};
pub use restore::restore_from_snapshot;
pub use storage::{
    delete_snapshot, get_applied_tweaks, get_snapshots_dir, load_snapshot, save_snapshot,
    snapshot_exists, update_snapshot_metadata,
};

#[cfg(test)]
mod tests {
    use super::helpers::{parse_hive, parse_value_type, task_state_matches, values_match};
    use crate::services::scheduler_service;
    use serde_json::json;

    // ========================================================================
    // values_match tests
    // ========================================================================

    #[test]
    fn test_values_match_both_none() {
        assert!(values_match(&None, &None));
    }

    #[test]
    fn test_values_match_one_none() {
        assert!(!values_match(&Some(json!(1)), &None));
        assert!(!values_match(&None, &Some(json!(1))));
    }

    #[test]
    fn test_values_match_equal_dwords() {
        let a = Some(json!(42u32));
        let b = Some(json!(42u32));
        assert!(values_match(&a, &b));
    }

    #[test]
    fn test_values_match_different_dwords() {
        let a = Some(json!(1));
        let b = Some(json!(0));
        assert!(!values_match(&a, &b));
    }

    #[test]
    fn test_values_match_equal_strings() {
        let a = Some(json!("test"));
        let b = Some(json!("test"));
        assert!(values_match(&a, &b));
    }

    #[test]
    fn test_values_match_different_strings() {
        let a = Some(json!("test1"));
        let b = Some(json!("test2"));
        assert!(!values_match(&a, &b));
    }

    #[test]
    fn test_values_match_numeric_coercion() {
        let a = Some(json!(1i64));
        let b = Some(json!(1u64));
        assert!(values_match(&a, &b));
    }

    // ========================================================================
    // parse_hive tests
    // ========================================================================

    #[test]
    fn test_parse_hive_hkcu() {
        assert!(parse_hive("HKCU").is_ok());
    }

    #[test]
    fn test_parse_hive_hklm() {
        assert!(parse_hive("HKLM").is_ok());
    }

    #[test]
    fn test_parse_hive_invalid() {
        assert!(parse_hive("INVALID").is_err());
    }

    // ========================================================================
    // parse_value_type tests
    // ========================================================================

    #[test]
    fn test_parse_value_type_dword() {
        assert!(parse_value_type("REG_DWORD").is_ok());
    }

    #[test]
    fn test_parse_value_type_string() {
        assert!(parse_value_type("REG_SZ").is_ok());
    }

    #[test]
    fn test_parse_value_type_invalid() {
        assert!(parse_value_type("INVALID").is_err());
    }

    // ========================================================================
    // task_state_matches tests
    // ========================================================================

    #[test]
    fn test_task_state_matches_same() {
        assert!(task_state_matches(
            &scheduler_service::TaskState::Ready,
            &scheduler_service::TaskState::Ready
        ));
    }

    #[test]
    fn test_task_state_matches_running_ready() {
        assert!(task_state_matches(
            &scheduler_service::TaskState::Running,
            &scheduler_service::TaskState::Ready
        ));
    }

    #[test]
    fn test_task_state_matches_disabled() {
        assert!(!task_state_matches(
            &scheduler_service::TaskState::Disabled,
            &scheduler_service::TaskState::Ready
        ));
    }
}
