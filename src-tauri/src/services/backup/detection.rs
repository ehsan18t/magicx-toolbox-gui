//! State Detection and Validation
//!
//! Functions for detecting tweak states and validating snapshots:
//! - Tweak state detection by comparing against option configurations
//! - Snapshot validation to detect externally reverted tweaks
//! - Migration utilities for old backup formats

use crate::error::Error;
use crate::models::{
    RegistryAction, RegistryValueType, TweakDefinition, TweakOption, TweakSnapshot, TweakState,
};
use crate::services::{registry_service, scheduler_service, service_control};
use rayon::prelude::*;
use std::fs;

use super::capture::read_registry_value;
use super::helpers::{parse_hive, parse_value_type, task_state_matches, values_match};
use super::storage::{delete_snapshot, get_applied_tweaks, load_snapshot, snapshot_exists};

// ============================================================================
// State Detection
// ============================================================================

/// Result of a match check: whether it matched and whether status was inferred
#[derive(Debug, Clone, Copy)]
struct MatchResult {
    /// Whether the option matches current system state
    matches: bool,
    /// Whether the match was inferred from missing items (via missing_is_match)
    /// rather than detected from actual values
    inferred: bool,
}

impl MatchResult {
    fn matched() -> Self {
        Self {
            matches: true,
            inferred: false,
        }
    }

    fn not_matched() -> Self {
        Self {
            matches: false,
            inferred: false,
        }
    }
}

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
        let result = option_matches_current_state(option, windows_version)?;
        if result.matches {
            return Ok(TweakState {
                tweak_id: tweak.id.clone(),
                current_option_index: Some(index),
                has_snapshot,
                snapshot_option_index,
                status_inferred: result.inferred,
            });
        }
    }

    // No option matches - system is in custom/default state
    Ok(TweakState {
        tweak_id: tweak.id.clone(),
        current_option_index: None,
        has_snapshot,
        snapshot_option_index,
        status_inferred: false,
    })
}

/// Check if all registry/service/scheduler changes in an option match current state
/// Items with skip_validation=true are excluded from this check
/// Uses parallel iteration for registry, service, and scheduler checks
fn option_matches_current_state(
    option: &TweakOption,
    windows_version: u32,
) -> Result<MatchResult, Error> {
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
        return Ok(MatchResult::not_matched());
    }

    // Use rayon to check all three categories in parallel using nested joins
    // Each category check is independent and can run concurrently
    let ((registry_result, services_result), scheduler_result) = rayon::join(
        || {
            rayon::join(
                || check_registry_matches(&validatable_registry, option.registry_missing_is_match),
                || check_services_match(&validatable_services, option.service_missing_is_match),
            )
        },
        || check_scheduler_matches(&validatable_scheduler, option.scheduler_missing_is_match),
    );

    let registry_result = registry_result?;
    let services_result = services_result?;
    let scheduler_result = scheduler_result?;

    // All must match for the option to match
    if registry_result.matches && services_result.matches && scheduler_result.matches {
        // Status is inferred if ANY component was inferred
        let inferred =
            registry_result.inferred || services_result.inferred || scheduler_result.inferred;
        Ok(MatchResult {
            matches: true,
            inferred,
        })
    } else {
        Ok(MatchResult::not_matched())
    }
}

/// Check if all registry values match expected state (parallelized)
/// When missing_is_match is true, missing registry keys/values are treated as matching
fn check_registry_matches(
    validatable_registry: &[&crate::models::RegistryChange],
    missing_is_match: bool,
) -> Result<MatchResult, Error> {
    if validatable_registry.is_empty() {
        return Ok(MatchResult::matched());
    }

    // Use parallel iteration for registry checks, collect results
    // Each result is (matched, was_inferred)
    let results: Vec<Result<(bool, bool), Error>> = validatable_registry
        .par_iter()
        .map(|change| {
            match change.action {
                RegistryAction::Set => {
                    // For Set action, check if the value matches
                    let value_type = match &change.value_type {
                        Some(vt) => vt,
                        None => return Ok((false, false)), // Invalid config
                    };
                    let expected_value = match &change.value {
                        Some(v) => v,
                        None => return Ok((false, false)), // Invalid config
                    };

                    let (current_value, existed) = read_registry_value(
                        &change.hive,
                        &change.key,
                        &change.value_name,
                        value_type,
                    )?;

                    if !existed {
                        // Item doesn't exist - check missing_is_match flag
                        if missing_is_match {
                            return Ok((true, true)); // Inferred match
                        }
                        return Ok((false, false));
                    }

                    Ok((
                        values_match(&current_value, &Some(expected_value.clone())),
                        false,
                    ))
                }
                RegistryAction::DeleteValue => {
                    // For DeleteValue, check that the value doesn't exist
                    let exists = registry_service::value_exists(
                        &change.hive,
                        &change.key,
                        &change.value_name,
                    )
                    .unwrap_or(false);
                    Ok((!exists, false))
                }
                RegistryAction::DeleteKey => {
                    // For DeleteKey, check that the key doesn't exist
                    let exists =
                        registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);
                    Ok((!exists, false))
                }
                RegistryAction::CreateKey => {
                    // For CreateKey, check that the key exists
                    let exists =
                        registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);
                    if !exists && missing_is_match {
                        // Key doesn't exist but missing_is_match - treat as inferred match
                        return Ok((true, true));
                    }
                    Ok((exists, false))
                }
            }
        })
        .collect();

    // Check if all results match and track if any were inferred
    let mut any_inferred = false;
    for result in results {
        let (matched, inferred) = result?;
        if !matched {
            return Ok(MatchResult::not_matched());
        }
        if inferred {
            any_inferred = true;
        }
    }

    Ok(MatchResult {
        matches: true,
        inferred: any_inferred,
    })
}

/// Check if all services match expected state (parallelized)
/// When missing_is_match is true, services that don't exist are treated as matching
fn check_services_match(
    validatable_services: &[&crate::models::ServiceChange],
    missing_is_match: bool,
) -> Result<MatchResult, Error> {
    if validatable_services.is_empty() {
        return Ok(MatchResult::matched());
    }

    // Use parallel iterator and collect results
    // Each result is (matched, was_inferred)
    let results: Vec<Result<(bool, bool), Error>> = validatable_services
        .par_iter()
        .map(|change| {
            let status = service_control::get_service_status(&change.name)?;

            // Check if service exists
            if !status.exists {
                // Service doesn't exist - check missing_is_match flag
                if missing_is_match {
                    return Ok((true, true)); // Inferred match
                }
                return Ok((false, false));
            }

            let current_startup = status.startup_type;
            Ok((current_startup == Some(change.startup), false))
        })
        .collect();

    // Check if all results match and track if any were inferred
    let mut any_inferred = false;
    for result in results {
        let (matched, inferred) = result?;
        if !matched {
            return Ok(MatchResult::not_matched());
        }
        if inferred {
            any_inferred = true;
        }
    }

    Ok(MatchResult {
        matches: true,
        inferred: any_inferred,
    })
}

/// Check if all scheduler tasks match expected state (parallelized)
/// When missing_is_match is true, tasks that don't exist are treated as matching
fn check_scheduler_matches(
    validatable_scheduler: &[&crate::models::SchedulerChange],
    missing_is_match: bool,
) -> Result<MatchResult, Error> {
    if validatable_scheduler.is_empty() {
        return Ok(MatchResult::matched());
    }

    // Process scheduler changes in parallel.
    //
    // NOTE: The scheduler_service uses the `schtasks.exe` CLI tool via std::process::Command,
    // which is thread-safe and does not share COM apartments between threads.
    // Therefore, it is safe to parallelize these checks.
    // Each result is (matched, was_inferred)
    let results: Vec<Result<(bool, bool), Error>> = validatable_scheduler
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
                    // No tasks found - check various conditions
                    if expected_state == scheduler_service::TaskState::NotFound {
                        // Expected deletion, no tasks found = match
                        return Ok((true, false));
                    }
                    if change.ignore_not_found {
                        // Tasks not found but ignore_not_found is set
                        return Ok((true, false));
                    }
                    if missing_is_match {
                        // Tasks not found but missing_is_match is set
                        return Ok((true, true)); // Inferred match
                    }
                    return Ok((false, false));
                } else {
                    // Check that all matching tasks have expected state
                    for task in tasks {
                        if !task_state_matches(&task.state, &expected_state) {
                            return Ok((false, false));
                        }
                    }
                }
            } else if let Some(ref task_name) = change.task_name {
                // Exact task name - check single task
                let current_state = scheduler_service::get_task_state(&change.task_path, task_name)
                    .unwrap_or(scheduler_service::TaskState::NotFound);

                if current_state == scheduler_service::TaskState::NotFound {
                    // Task not found
                    if change.ignore_not_found {
                        return Ok((true, false));
                    }
                    if missing_is_match {
                        return Ok((true, true)); // Inferred match
                    }
                    // Only match if we expected deletion
                    if expected_state == scheduler_service::TaskState::NotFound {
                        return Ok((true, false));
                    }
                    return Ok((false, false));
                }

                if !task_state_matches(&current_state, &expected_state) {
                    return Ok((false, false));
                }
            }
            // If neither pattern nor name, skip validation for this change (implicitly match)

            Ok((true, false))
        })
        .collect();

    // Check if all results match and track if any were inferred
    let mut any_inferred = false;
    for result in results {
        let (matched, inferred) = result?;
        if !matched {
            return Ok(MatchResult::not_matched());
        }
        if inferred {
            any_inferred = true;
        }
    }

    Ok(MatchResult {
        matches: true,
        inferred: any_inferred,
    })
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
