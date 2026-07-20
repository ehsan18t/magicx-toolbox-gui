//! State Detection and Validation
//!
//! Functions for detecting tweak states and validating snapshots:
//! - Tweak state detection by comparing against option configurations
//! - Snapshot validation to detect externally reverted tweaks
//! - Migration utilities for old backup formats

use crate::error::Error;
use crate::models::{RegistryValueType, TweakDefinition, TweakSnapshot, TweakState};
use crate::services::{
    firewall_service, hosts_service, registry_value, scheduler_service, service_control,
};
use rayon::prelude::*;

use super::capture::read_registry_value;
use super::helpers::{parse_hive, parse_value_type, task_state_matches};
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

    // Try to match current state against each option (shared comparison core).
    for (index, option) in tweak.options.iter().enumerate() {
        let comparison = super::compare::compare_option(option, windows_version)?;
        if comparison.all_match() {
            return Ok(TweakState {
                tweak_id: tweak.id.clone(),
                current_option_index: Some(index),
                has_snapshot,
                snapshot_option_index,
                status_inferred: comparison.inferred,
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

// ============================================================================
// Migration & Validation
// ============================================================================

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

/// Check if current system state matches the snapshot state
/// (indicating tweak was externally reverted).
///
/// If a captured resource cannot be verified, return the error so startup cleanup
/// preserves the snapshot instead of deleting rollback data.
fn snapshot_matches_current_state(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    let has_any_snapshot = !snapshot.registry_snapshots.is_empty()
        || !snapshot.service_snapshots.is_empty()
        || !snapshot.scheduler_snapshots.is_empty()
        || !snapshot.hosts_snapshots.is_empty()
        || !snapshot.firewall_snapshots.is_empty();

    if !has_any_snapshot {
        return Ok(false);
    }

    Ok(registry_snapshots_match(snapshot)?
        && service_snapshots_match(snapshot)?
        && scheduler_snapshots_match(snapshot)?
        && hosts_snapshots_match(snapshot)?
        && firewall_snapshots_match(snapshot)?)
}

fn registry_snapshots_match(snapshot: &TweakSnapshot) -> Result<bool, Error> {
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

            if !reg.existed && !current_exists {
                return Ok(true);
            }

            if reg.existed && current_exists {
                return registry_value::registry_values_match(
                    &value_type,
                    &current_value,
                    &reg.value,
                );
            }

            Ok(false)
        })
        .collect();

    all_match(results)
}

fn service_snapshots_match(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    for service in &snapshot.service_snapshots {
        let status = service_control::get_service_status(&service.name)?;
        let current_startup = status
            .startup_type
            .map(|startup| startup.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let current_running = status.state == service_control::ServiceState::Running;

        if current_startup != service.startup_type || current_running != service.was_running {
            return Ok(false);
        }
    }

    Ok(true)
}

fn scheduler_snapshots_match(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    for task in &snapshot.scheduler_snapshots {
        let current_state = scheduler_service::get_task_state(&task.task_path, &task.task_name)?;
        let expected_state = scheduler_service::TaskState::from_str(&task.original_state);

        if !task_state_matches(&current_state, &expected_state) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn hosts_snapshots_match(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    for host in &snapshot.hosts_snapshots {
        let current_exists = hosts_service::entry_exists(&host.ip, &host.domain)?;

        if current_exists != host.existed {
            return Ok(false);
        }
    }

    Ok(true)
}

fn firewall_snapshots_match(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    for firewall in &snapshot.firewall_snapshots {
        let current_exists = firewall_service::rule_exists(&firewall.name)?;

        if current_exists != firewall.existed {
            return Ok(false);
        }
    }

    Ok(true)
}

fn all_match(results: Vec<Result<bool, Error>>) -> Result<bool, Error> {
    for result in results {
        if !result? {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ServiceSnapshot;

    #[test]
    fn service_only_snapshot_is_not_stale_when_original_service_state_differs() {
        let mut snapshot = empty_snapshot();
        snapshot.service_snapshots.push(ServiceSnapshot {
            name: "__mgx_missing_service_for_stale_snapshot_test__".to_string(),
            startup_type: "manual".to_string(),
            was_running: false,
        });

        let matches = snapshot_matches_current_state(&snapshot).unwrap();

        assert!(!matches);
    }

    #[test]
    fn empty_snapshot_is_not_stale() {
        let snapshot = empty_snapshot();

        let matches = snapshot_matches_current_state(&snapshot).unwrap();

        assert!(!matches);
    }

    fn empty_snapshot() -> TweakSnapshot {
        TweakSnapshot::new("test_tweak", "Test Tweak", 0, "Apply", 11, false, None)
    }
}
