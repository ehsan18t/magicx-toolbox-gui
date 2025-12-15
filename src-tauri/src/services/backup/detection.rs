//! State Detection and Validation
//!
//! Functions for detecting tweak states and validating snapshots:
//! - Tweak state detection by comparing against option configurations
//! - Snapshot validation to detect externally reverted tweaks
//! - Migration utilities for old backup formats

use crate::error::Error;
use crate::models::{RegistryValueType, TweakDefinition, TweakOption, TweakSnapshot, TweakState};
use crate::services::{scheduler_service, service_control};
use rayon::prelude::*;
use std::fs;

use super::capture::read_registry_value;
use super::helpers::{parse_hive, parse_value_type, task_state_matches, values_match};
use super::storage::{delete_snapshot, get_applied_tweaks, load_snapshot, snapshot_exists};

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
/// Uses parallel iteration for registry, service, and scheduler checks
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

    // Use rayon to check all three categories in parallel using nested joins
    // Each category check is independent and can run concurrently
    let ((registry_ok, services_ok), scheduler_ok) = rayon::join(
        || {
            rayon::join(
                || check_registry_matches(&validatable_registry),
                || check_services_match(&validatable_services),
            )
        },
        || check_scheduler_matches(&validatable_scheduler),
    );

    Ok(registry_ok? && services_ok? && scheduler_ok?)
}

/// Check if all registry values match expected state (parallelized)
fn check_registry_matches(
    validatable_registry: &[&crate::models::RegistryChange],
) -> Result<bool, Error> {
    if validatable_registry.is_empty() {
        return Ok(true);
    }

    // Use parallel iteration for registry checks, collect results
    let results: Vec<Result<bool, Error>> = validatable_registry
        .par_iter()
        .map(|change| {
            let (current_value, existed) = read_registry_value(
                &change.hive,
                &change.key,
                &change.value_name,
                &change.value_type,
            )?;

            if !existed {
                return Ok(false);
            }

            Ok(values_match(&current_value, &Some(change.value.clone())))
        })
        .collect();

    // Check if all results are Ok(true)
    for result in results {
        if !result? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Check if all services match expected state (parallelized)
fn check_services_match(
    validatable_services: &[&crate::models::ServiceChange],
) -> Result<bool, Error> {
    if validatable_services.is_empty() {
        return Ok(true);
    }

    // Use parallel iterator and collect results
    let results: Vec<Result<bool, Error>> = validatable_services
        .par_iter()
        .map(|change| {
            let status = service_control::get_service_status(&change.name)?;
            let current_startup = status.startup_type;
            Ok(current_startup == Some(change.startup))
        })
        .collect();

    // Check if all results are Ok(true)
    for result in results {
        if !result? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Check if all scheduler tasks match expected state (parallelized)
fn check_scheduler_matches(
    validatable_scheduler: &[&crate::models::SchedulerChange],
) -> Result<bool, Error> {
    if validatable_scheduler.is_empty() {
        return Ok(true);
    }

    // Process scheduler changes in parallel.
    //
    // NOTE: The scheduler_service uses the `schtasks.exe` CLI tool via std::process::Command,
    // which is thread-safe and does not share COM apartments between threads.
    // Therefore, it is safe to parallelize these checks.
    let results: Vec<Result<bool, Error>> = validatable_scheduler
        .par_iter()
        .map(|change| {
            // Determine expected state based on action
            let expected_state = match change.action {
                crate::models::tweak::SchedulerAction::Enable => {
                    scheduler_service::TaskState::Ready
                }
                crate::models::tweak::SchedulerAction::Disable => {
                    scheduler_service::TaskState::Disabled
                }
                crate::models::tweak::SchedulerAction::Delete => {
                    scheduler_service::TaskState::NotFound
                }
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
                if current_state == scheduler_service::TaskState::NotFound
                    && change.ignore_not_found
                {
                    // Task not found but ignore_not_found is set - consider this as matching
                    return Ok(true);
                }

                if !task_state_matches(&current_state, &expected_state) {
                    return Ok(false);
                }
            }
            // If neither pattern nor name, skip validation for this change (implicitly match)

            Ok(true)
        })
        .collect();

    // Check if all results are Ok(true)
    for result in results {
        if !result? {
            return Ok(false);
        }
    }

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

/// Validate all snapshots on app startup (parallelized)
/// Removes stale snapshots where tweak was externally reverted
pub fn validate_all_snapshots() -> Result<u32, Error> {
    log::info!("Validating all snapshots on startup");

    let applied_tweaks = get_applied_tweaks()?;

    // Use parallel iteration to check all snapshots concurrently
    let stale_tweaks: Vec<String> = applied_tweaks
        .par_iter()
        .filter_map(|tweak_id| {
            match load_snapshot(tweak_id) {
                Ok(Some(snapshot)) => {
                    // A snapshot is stale if current state matches the original snapshot state
                    // (meaning the tweak was externally reverted)
                    match snapshot_matches_current_state(&snapshot) {
                        Ok(true) => {
                            log::info!(
                                "Found stale snapshot for '{}' - tweak was externally reverted",
                                snapshot.tweak_name
                            );
                            Some(tweak_id.clone())
                        }
                        Ok(false) => None,
                        Err(e) => {
                            log::warn!("Error checking snapshot for {}: {}", tweak_id, e);
                            None
                        }
                    }
                }
                Ok(None) => None,
                Err(e) => {
                    log::warn!("Error loading snapshot for {}: {}", tweak_id, e);
                    None
                }
            }
        })
        .collect();

    // Delete stale snapshots (sequential to avoid file system race conditions)
    let removed_count = stale_tweaks.len() as u32;
    for tweak_id in stale_tweaks {
        if let Err(e) = delete_snapshot(&tweak_id) {
            log::warn!("Failed to delete stale snapshot for {}: {}", tweak_id, e);
        }
    }

    if removed_count > 0 {
        log::info!("Removed {} stale snapshots", removed_count);
    }

    Ok(removed_count)
}

/// Check if current registry state matches the snapshot state (parallelized)
/// (indicating tweak was externally reverted)
fn snapshot_matches_current_state(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    if snapshot.registry_snapshots.is_empty() {
        return Ok(true);
    }

    // Use parallel iteration to check all registry values and collect results
    let results: Vec<Result<bool, Error>> = snapshot
        .registry_snapshots
        .par_iter()
        .map(|reg| {
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

            Ok(matches)
        })
        .collect();

    // Check if all results match
    for result in results {
        if !result? {
            return Ok(false);
        }
    }

    Ok(true)
}
