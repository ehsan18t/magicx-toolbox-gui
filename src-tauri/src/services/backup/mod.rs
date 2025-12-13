//! # Snapshot-Based Backup Service
//!
//! Unified option-based backup system for Windows registry tweaks with atomic
//! rollback capabilities.
//!
//! ## Module Organization
//!
//! This module is split into:
//! - `storage`: File I/O for snapshot persistence
//! - `capture`: State capture before applying tweaks
//! - `restore`: Atomic restore with rollback support
//! - `helpers`: Parsing and comparison utilities

mod capture;
mod helpers;
mod restore;
mod storage;

// Re-export public items from submodules
pub use capture::{capture_current_state, capture_snapshot};
pub use restore::restore_from_snapshot;
pub use storage::{
    delete_snapshot, get_applied_tweaks, get_snapshots_dir, load_snapshot, save_snapshot,
    snapshot_exists, update_snapshot_metadata,
};

// Internal re-exports for cross-module use
use capture::read_registry_value;
use helpers::{parse_hive, parse_value_type, task_state_matches, values_match};

use crate::error::Error;
use crate::models::{RegistryValueType, TweakDefinition, TweakOption, TweakSnapshot, TweakState};
use crate::services::{scheduler_service, service_control};
use std::fs;

// ============================================================================
// State Detection
// ============================================================================

/// Detect current state of a tweak by comparing against all options
/// Returns TweakState with current_option_index = None if no option matches
pub fn detect_tweak_state(
    tweak: &TweakDefinition,
    windows_version: u32,
) -> Result<TweakState, Error> {
    let has_snapshot = snapshot_exists(&tweak.id)?;
    let snapshot_option_index = if has_snapshot {
        load_snapshot(&tweak.id)?.map(|s| s.applied_option_index)
    } else {
        None
    };

    // Try to match current state against each option
    for (index, option) in tweak.options.iter().enumerate() {
        if option_matches_current_state(option, windows_version)? {
            return Ok(TweakState {
                tweak_id: tweak.id.clone(),
                current_option_index: Some(index),
                has_snapshot,
                snapshot_option_index,
            });
        }
    }

    // No option matches - system is in custom/default state
    Ok(TweakState {
        tweak_id: tweak.id.clone(),
        current_option_index: None,
        has_snapshot,
        snapshot_option_index,
    })
}

/// Check if all registry/service/scheduler changes in an option match current state
/// Items with skip_validation=true are excluded from this check
fn option_matches_current_state(option: &TweakOption, windows_version: u32) -> Result<bool, Error> {
    // Count only validatable changes (those without skip_validation)
    let validatable_registry: Vec<_> = option
        .registry_changes
        .iter()
        .filter(|c| !c.skip_validation && c.applies_to_version(windows_version))
        .collect();
    let validatable_services: Vec<_> = option
        .service_changes
        .iter()
        .filter(|c| !c.skip_validation)
        .collect();
    let validatable_scheduler: Vec<_> = option
        .scheduler_changes
        .iter()
        .filter(|c| !c.skip_validation)
        .collect();

    // If option has no validatable changes, it can't match
    if validatable_registry.is_empty()
        && validatable_services.is_empty()
        && validatable_scheduler.is_empty()
    {
        return Ok(false);
    }

    // Check all validatable registry values
    for change in validatable_registry {
        let (current_value, existed) = read_registry_value(
            &change.hive,
            &change.key,
            &change.value_name,
            &change.value_type,
        )?;

        if !existed {
            return Ok(false);
        }

        if !values_match(&current_value, &Some(change.value.clone())) {
            return Ok(false);
        }
    }

    // Check all validatable service states
    for change in validatable_services {
        let status = service_control::get_service_status(&change.name)?;
        let current_startup = status.startup_type;

        if current_startup != Some(change.startup) {
            return Ok(false);
        }
    }

    // Check all validatable scheduler task states
    for change in validatable_scheduler {
        // Determine expected state based on action
        let expected_state = match change.action {
            crate::models::tweak::SchedulerAction::Enable => scheduler_service::TaskState::Ready,
            crate::models::tweak::SchedulerAction::Disable => {
                scheduler_service::TaskState::Disabled
            }
            crate::models::tweak::SchedulerAction::Delete => scheduler_service::TaskState::NotFound,
        };

        // Handle pattern vs exact name
        if let Some(ref pattern) = change.task_name_pattern {
            // For patterns, find all matching tasks and check each
            let tasks = scheduler_service::find_tasks_by_pattern(&change.task_path, pattern)
                .unwrap_or_default();

            if tasks.is_empty() {
                // No tasks found - only matches if we expected deletion or ignore_not_found
                if expected_state != scheduler_service::TaskState::NotFound
                    && !change.ignore_not_found
                {
                    return Ok(false);
                }
            } else {
                // Check that all matching tasks have expected state
                for task in tasks {
                    if !task_state_matches(&task.state, &expected_state) {
                        return Ok(false);
                    }
                }
            }
        } else if let Some(ref task_name) = change.task_name {
            // Exact task name - check single task
            let current_state = scheduler_service::get_task_state(&change.task_path, task_name)
                .unwrap_or(scheduler_service::TaskState::NotFound);

            // Handle ignore_not_found for exact names
            if current_state == scheduler_service::TaskState::NotFound && change.ignore_not_found {
                // Task not found but ignore_not_found is set - consider this as matching
                continue;
            }

            if !task_state_matches(&current_state, &expected_state) {
                return Ok(false);
            }
        }
        // If neither pattern nor name, skip validation for this change
    }

    // All validatable checks passed
    Ok(true)
}

// ============================================================================
// Migration & Validation
// ============================================================================

/// Clean up old backup files (one-time migration)
pub fn cleanup_old_backups() -> Result<(), Error> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| Error::BackupFailed(format!("Failed to get executable path: {}", e)))?
        .parent()
        .ok_or_else(|| Error::BackupFailed("Failed to get executable directory".into()))?
        .to_path_buf();

    let old_backups_dir = exe_dir.join("backups");

    if old_backups_dir.exists() {
        log::info!("Cleaning up old backup files from {:?}", old_backups_dir);
        if let Ok(entries) = fs::read_dir(&old_backups_dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
        let _ = fs::remove_dir(&old_backups_dir);
        log::info!("Old backup cleanup complete");
    }

    Ok(())
}

/// Validate all snapshots on app startup
/// Removes stale snapshots where tweak was externally reverted
pub fn validate_all_snapshots() -> Result<u32, Error> {
    log::info!("Validating all snapshots on startup");

    let applied_tweaks = get_applied_tweaks()?;
    let mut removed_count = 0;

    for tweak_id in applied_tweaks {
        if let Some(snapshot) = load_snapshot(&tweak_id)? {
            // A snapshot is stale if current state matches the original snapshot state
            // (meaning the tweak was externally reverted)
            let is_stale = snapshot_matches_current_state(&snapshot)?;

            if is_stale {
                log::info!(
                    "Removing stale snapshot for '{}' - tweak was externally reverted",
                    snapshot.tweak_name
                );
                delete_snapshot(&tweak_id)?;
                removed_count += 1;
            }
        }
    }

    if removed_count > 0 {
        log::info!("Removed {} stale snapshots", removed_count);
    }

    Ok(removed_count)
}

/// Check if current registry state matches the snapshot state
/// (indicating tweak was externally reverted)
fn snapshot_matches_current_state(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    for reg in &snapshot.registry_snapshots {
        let hive = parse_hive(&reg.hive)?;
        let value_type = reg
            .value_type
            .as_ref()
            .map(|t| parse_value_type(t))
            .transpose()?
            .unwrap_or(RegistryValueType::Dword);

        let (current_value, current_exists) =
            read_registry_value(&hive, &reg.key, &reg.value_name, &value_type)?;

        // Check if current state matches snapshot state
        let matches = if !reg.existed && !current_exists {
            true
        } else if reg.existed && current_exists {
            values_match(&reg.value, &current_value)
        } else {
            false
        };

        if !matches {
            return Ok(false);
        }
    }

    // All values match snapshot - tweak was reverted
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::helpers::{parse_hive, parse_value_type, task_state_matches, values_match};
    use super::*;
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
        let result = parse_hive("HKCU");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hive_hklm() {
        let result = parse_hive("HKLM");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hive_invalid() {
        let result = parse_hive("INVALID");
        assert!(result.is_err());
    }

    // ========================================================================
    // parse_value_type tests
    // ========================================================================

    #[test]
    fn test_parse_value_type_dword() {
        let result = parse_value_type("REG_DWORD");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_value_type_string() {
        let result = parse_value_type("REG_SZ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_value_type_invalid() {
        let result = parse_value_type("INVALID");
        assert!(result.is_err());
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
