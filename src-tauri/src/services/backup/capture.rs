//! Snapshot Capture Operations
//!
//! Functions for capturing system state before applying tweaks:
//! - Registry value snapshots
//! - Service state snapshots
//! - Scheduled task state snapshots

use crate::error::Error;
use crate::models::{
    RegistryHive, RegistrySnapshot, RegistryValueType, SchedulerSnapshot, ServiceSnapshot,
    TweakDefinition, TweakSnapshot,
};
use crate::services::{registry_service, scheduler_service, service_control};

/// Capture complete state before applying a tweak option
pub fn capture_snapshot(
    tweak: &TweakDefinition,
    option_index: usize,
    windows_version: u32,
) -> Result<TweakSnapshot, Error> {
    let option = tweak
        .options
        .get(option_index)
        .ok_or_else(|| Error::BackupFailed(format!("Invalid option index: {}", option_index)))?;

    log::info!(
        "Capturing snapshot for tweak '{}' option '{}' (index {})",
        tweak.name,
        option.label,
        option_index
    );

    let mut snapshot = TweakSnapshot::new(
        &tweak.id,
        &tweak.name,
        option_index,
        &option.label,
        windows_version,
        tweak.requires_system,
    );

    // Capture registry values for this option
    for change in &option.registry_changes {
        // Skip if version doesn't apply
        if !change.applies_to_version(windows_version) {
            continue;
        }

        let (value, existed) = read_registry_value(
            &change.hive,
            &change.key,
            &change.value_name,
            &change.value_type,
        )?;

        let reg_snapshot = RegistrySnapshot {
            hive: change.hive.as_str().to_string(),
            key: change.key.clone(),
            value_name: change.value_name.clone(),
            value_type: if existed {
                Some(change.value_type.as_str().to_string())
            } else {
                None
            },
            value,
            existed,
        };

        snapshot.add_registry_snapshot(reg_snapshot);
        log::trace!(
            "Captured: {}\\{}\\{} (existed: {})",
            change.hive.as_str(),
            change.key,
            change.value_name,
            existed
        );
    }

    // Capture service states for this option
    for sc in &option.service_changes {
        let service_snapshot = capture_service_state(&sc.name)?;
        snapshot.add_service_snapshot(service_snapshot);
    }

    // Capture scheduled task states for this option
    for task_change in &option.scheduler_changes {
        // Handle pattern matching vs exact task name
        if let Some(ref pattern) = task_change.task_name_pattern {
            // Pattern-based: capture state for all matching tasks
            let matching_tasks =
                scheduler_service::find_tasks_by_pattern(&task_change.task_path, pattern)?;

            if matching_tasks.is_empty() {
                // If ignore_not_found, just skip capturing
                if task_change.ignore_not_found {
                    log::debug!(
                        "No tasks found matching pattern '{}' in '{}' (ignore_not_found)",
                        pattern,
                        task_change.task_path
                    );
                    continue;
                }
                // Otherwise, still continue - we don't fail snapshot capture
                log::warn!(
                    "No tasks found matching pattern '{}' in '{}'",
                    pattern,
                    task_change.task_path
                );
                continue;
            }

            for task in matching_tasks {
                let task_snapshot = SchedulerSnapshot {
                    task_path: task_change.task_path.clone(),
                    task_name: task.name.clone(),
                    original_state: task.state.as_str().to_string(),
                };
                snapshot.add_scheduler_snapshot(task_snapshot);
                log::trace!(
                    "Captured pattern task: {}\\{} (state: {})",
                    task_change.task_path,
                    task.name,
                    task.state.as_str()
                );
            }
        } else if let Some(ref task_name) = task_change.task_name {
            // Exact task name: capture single task state
            let task_snapshot = capture_scheduler_state(&task_change.task_path, task_name)?;
            snapshot.add_scheduler_snapshot(task_snapshot);
        } else {
            // Neither pattern nor name specified - skip with warning
            log::warn!("Scheduler change has neither task_name nor task_name_pattern, skipping");
        }
    }

    log::info!(
        "Captured {} registry values, {} services, and {} tasks for '{}'",
        snapshot.registry_snapshots.len(),
        snapshot.service_snapshots.len(),
        snapshot.scheduler_snapshots.len(),
        tweak.name
    );

    Ok(snapshot)
}

/// Capture CURRENT system state for ALL items across ALL options of a tweak.
/// Used for rollback when switching between options - restores to the state
/// BEFORE the current apply operation started (not the original pre-tweak state).
pub fn capture_current_state(
    tweak: &TweakDefinition,
    windows_version: u32,
) -> Result<TweakSnapshot, Error> {
    log::info!(
        "Capturing current state for tweak '{}' (all options)",
        tweak.name
    );

    // We create a snapshot but option_index/label don't matter here since this is temporary
    let mut snapshot = TweakSnapshot::new(
        &tweak.id,
        &tweak.name,
        usize::MAX, // Marker for "current state" snapshot
        "_current_state_",
        windows_version,
        tweak.requires_system,
    );

    // Use HashSet to avoid duplicates across options
    use std::collections::HashSet;
    let mut captured_registry: HashSet<String> = HashSet::new();
    let mut captured_services: HashSet<String> = HashSet::new();
    let mut captured_tasks: HashSet<String> = HashSet::new();

    // Iterate ALL options to capture all potentially affected items
    for option in &tweak.options {
        // Capture registry values
        for change in &option.registry_changes {
            if !change.applies_to_version(windows_version) {
                continue;
            }

            let key_id = format!(
                "{}\\{}\\{}",
                change.hive.as_str(),
                change.key,
                change.value_name
            );
            if captured_registry.contains(&key_id) {
                continue;
            }
            captured_registry.insert(key_id);

            let (value, existed) = read_registry_value(
                &change.hive,
                &change.key,
                &change.value_name,
                &change.value_type,
            )?;

            let reg_snapshot = RegistrySnapshot {
                hive: change.hive.as_str().to_string(),
                key: change.key.clone(),
                value_name: change.value_name.clone(),
                value_type: if existed {
                    Some(change.value_type.as_str().to_string())
                } else {
                    None
                },
                value,
                existed,
            };

            snapshot.add_registry_snapshot(reg_snapshot);
        }

        // Capture service states
        for sc in &option.service_changes {
            if captured_services.contains(&sc.name) {
                continue;
            }
            captured_services.insert(sc.name.clone());

            let service_snapshot = capture_service_state(&sc.name)?;
            snapshot.add_service_snapshot(service_snapshot);
        }

        // Capture scheduled task states
        for task_change in &option.scheduler_changes {
            if let Some(ref pattern) = task_change.task_name_pattern {
                // Pattern-based matching
                let matching_tasks =
                    scheduler_service::find_tasks_by_pattern(&task_change.task_path, pattern)?;

                for task in matching_tasks {
                    let task_id = format!("{}\\{}", task_change.task_path, task.name);
                    if captured_tasks.contains(&task_id) {
                        continue;
                    }
                    captured_tasks.insert(task_id);

                    let task_snapshot = SchedulerSnapshot {
                        task_path: task_change.task_path.clone(),
                        task_name: task.name.clone(),
                        original_state: task.state.as_str().to_string(),
                    };
                    snapshot.add_scheduler_snapshot(task_snapshot);
                }
            } else if let Some(ref task_name) = task_change.task_name {
                let task_id = format!("{}\\{}", task_change.task_path, task_name);
                if captured_tasks.contains(&task_id) {
                    continue;
                }
                captured_tasks.insert(task_id);

                // Don't fail if task doesn't exist during current state capture
                match capture_scheduler_state(&task_change.task_path, task_name) {
                    Ok(task_snapshot) => {
                        snapshot.add_scheduler_snapshot(task_snapshot);
                    }
                    Err(e) => {
                        log::debug!(
                            "Could not capture state for task {}\\{}: {} (may not exist)",
                            task_change.task_path,
                            task_name,
                            e
                        );
                    }
                }
            }
        }
    }

    log::info!(
        "Captured current state: {} registry values, {} services, {} tasks for '{}'",
        snapshot.registry_snapshots.len(),
        snapshot.service_snapshots.len(),
        snapshot.scheduler_snapshots.len(),
        tweak.name
    );

    Ok(snapshot)
}

/// Capture current service state
pub(crate) fn capture_service_state(service_name: &str) -> Result<ServiceSnapshot, Error> {
    let status = service_control::get_service_status(service_name)?;
    let startup_type = status
        .startup_type
        .map(|t| format!("{:?}", t).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ServiceSnapshot {
        name: service_name.to_string(),
        startup_type,
        was_running: status.state == service_control::ServiceState::Running,
    })
}

/// Capture current scheduled task state
pub(crate) fn capture_scheduler_state(
    task_path: &str,
    task_name: &str,
) -> Result<SchedulerSnapshot, Error> {
    let state = scheduler_service::get_task_state(task_path, task_name)?;

    Ok(SchedulerSnapshot {
        task_path: task_path.to_string(),
        task_name: task_name.to_string(),
        original_state: state.as_str().to_string(),
    })
}

/// Read a registry value (returns value and whether it existed)
pub(crate) fn read_registry_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
) -> Result<(Option<serde_json::Value>, bool), Error> {
    let result = match value_type {
        RegistryValueType::Dword => registry_service::read_dword(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        RegistryValueType::String | RegistryValueType::ExpandString => {
            registry_service::read_string(hive, key, value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
        RegistryValueType::Binary => registry_service::read_binary(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        RegistryValueType::Qword => registry_service::read_qword(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        RegistryValueType::MultiString => registry_service::read_string(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
    };

    match result {
        Ok(Some(value)) => Ok((Some(value), true)),
        Ok(None) => Ok((None, false)),
        Err(Error::RegistryKeyNotFound(_)) => Ok((None, false)),
        Err(e) => {
            log::warn!(
                "Failed to read {}\\{}\\{}: {}",
                hive.as_str(),
                key,
                value_name,
                e
            );
            Ok((None, false))
        }
    }
}
