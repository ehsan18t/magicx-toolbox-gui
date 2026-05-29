//! Snapshot Capture Operations
//!
//! Functions for capturing system state before applying tweaks:
//! - Registry value snapshots
//! - Service state snapshots
//! - Scheduled task state snapshots

use crate::error::Error;
use crate::models::{
    FirewallSnapshot, HostsSnapshot, RegistryAction, RegistryHive, RegistrySnapshot,
    RegistryValueType, SchedulerSnapshot, ServiceSnapshot, TweakDefinition, TweakSnapshot,
};
use crate::services::{
    firewall_service, hosts_service, registry_service, scheduler_service, service_control,
};
use rayon::prelude::*;

/// Capture complete state before applying a tweak option (parallelized)
pub fn capture_snapshot(
    tweak: &TweakDefinition,
    option_index: usize,
    windows_version: u32,
    original_option_index: Option<usize>,
) -> Result<TweakSnapshot, Error> {
    let option = tweak
        .options
        .get(option_index)
        .ok_or_else(|| Error::BackupFailed(format!("Invalid option index: {}", option_index)))?;

    log::info!(
        "Capturing snapshot for tweak '{}' option '{}' (index {}), original_option_index={:?}",
        tweak.name,
        option.label,
        option_index,
        original_option_index
    );

    let mut snapshot = TweakSnapshot::new(
        &tweak.id,
        &tweak.name,
        option_index,
        &option.label,
        windows_version,
        tweak.requires_system,
        original_option_index,
    );

    // Parallel capture: registry, services, scheduler, hosts, and firewall run concurrently
    let ((registry_result, (services_result, scheduler_result)), (hosts_result, firewall_result)) =
        rayon::join(
            || {
                rayon::join(
                    || capture_registry_snapshots(&option.registry_changes, windows_version),
                    || {
                        rayon::join(
                            || capture_service_snapshots(&option.service_changes),
                            || capture_scheduler_snapshots(&option.scheduler_changes),
                        )
                    },
                )
            },
            || {
                rayon::join(
                    || capture_hosts_snapshots(&option.hosts_changes),
                    || capture_firewall_snapshots(&option.firewall_changes),
                )
            },
        );

    // Add captured snapshots to the result
    for reg_snapshot in registry_result? {
        log::trace!(
            "Captured: {}\\{}\\{} (existed: {})",
            reg_snapshot.hive,
            reg_snapshot.key,
            reg_snapshot.value_name,
            reg_snapshot.existed
        );
        snapshot.add_registry_snapshot(reg_snapshot);
    }

    for service_snapshot in services_result? {
        snapshot.add_service_snapshot(service_snapshot);
    }

    for task_snapshot in scheduler_result? {
        snapshot.add_scheduler_snapshot(task_snapshot);
    }

    for hosts_snapshot in hosts_result? {
        snapshot.add_hosts_snapshot(hosts_snapshot);
    }

    for firewall_snapshot in firewall_result? {
        snapshot.add_firewall_snapshot(firewall_snapshot);
    }

    log::info!(
        "Captured {} registry, {} services, {} tasks, {} hosts, {} firewall for '{}'",
        snapshot.registry_snapshots.len(),
        snapshot.service_snapshots.len(),
        snapshot.scheduler_snapshots.len(),
        snapshot.hosts_snapshots.len(),
        snapshot.firewall_snapshots.len(),
        tweak.name
    );

    Ok(snapshot)
}

/// Capture registry values in parallel
fn capture_registry_snapshots(
    registry_changes: &[crate::models::RegistryChange],
    windows_version: u32,
) -> Result<Vec<RegistrySnapshot>, Error> {
    registry_changes
        .par_iter()
        .filter(|change| change.applies_to_version(windows_version))
        .map(|change| {
            match change.action {
                RegistryAction::Set => {
                    // For Set, capture the current value
                    let value_type = change.value_type.unwrap_or(RegistryValueType::Dword);
                    let (value, existed) = read_registry_value(
                        &change.hive,
                        &change.key,
                        &change.value_name,
                        &value_type,
                    )?;

                    Ok(RegistrySnapshot {
                        hive: change.hive.as_str().to_string(),
                        key: change.key.clone(),
                        value_name: change.value_name.clone(),
                        value_type: if existed {
                            Some(value_type.as_str().to_string())
                        } else {
                            None
                        },
                        value,
                        existed,
                    })
                }
                RegistryAction::DeleteValue => {
                    // For DeleteValue, capture if value exists and its current value
                    let value_type = change.value_type.unwrap_or(RegistryValueType::Dword);
                    let (value, existed) = read_registry_value(
                        &change.hive,
                        &change.key,
                        &change.value_name,
                        &value_type,
                    )?;

                    Ok(RegistrySnapshot {
                        hive: change.hive.as_str().to_string(),
                        key: change.key.clone(),
                        value_name: change.value_name.clone(),
                        value_type: if existed {
                            Some(value_type.as_str().to_string())
                        } else {
                            None
                        },
                        value,
                        existed,
                    })
                }
                RegistryAction::DeleteKey => {
                    // For DeleteKey, just note if the key existed
                    let existed =
                        registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);

                    Ok(RegistrySnapshot {
                        hive: change.hive.as_str().to_string(),
                        key: change.key.clone(),
                        value_name: String::new(), // Key-level operation, no specific value
                        value_type: None,
                        value: None,
                        existed,
                    })
                }
                RegistryAction::CreateKey => {
                    // For CreateKey, note if the key already existed
                    let existed =
                        registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);

                    Ok(RegistrySnapshot {
                        hive: change.hive.as_str().to_string(),
                        key: change.key.clone(),
                        value_name: String::new(), // Key-level operation, no specific value
                        value_type: None,
                        value: None,
                        existed,
                    })
                }
            }
        })
        .collect()
}

/// Capture service states in parallel
fn capture_service_snapshots(
    service_changes: &[crate::models::ServiceChange],
) -> Result<Vec<ServiceSnapshot>, Error> {
    service_changes
        .par_iter()
        .map(|sc| capture_service_state(&sc.name))
        .collect()
}

/// Capture scheduler task states (mixed parallel/sequential due to pattern matching)
fn capture_scheduler_snapshots(
    scheduler_changes: &[crate::models::SchedulerChange],
) -> Result<Vec<SchedulerSnapshot>, Error> {
    let mut snapshots = Vec::new();

    // Process scheduler changes - patterns need sequential handling due to find_tasks_by_pattern
    // but individual task captures can be parallelized within each pattern
    for task_change in scheduler_changes {
        if let Some(ref pattern) = task_change.task_name_pattern {
            // Pattern-based: capture state for all matching tasks
            let matching_tasks =
                scheduler_service::find_tasks_by_pattern(&task_change.task_path, pattern)?;

            if matching_tasks.is_empty() {
                if task_change.ignore_not_found {
                    log::debug!(
                        "No tasks found matching pattern '{}' in '{}' (ignore_not_found)",
                        pattern,
                        task_change.task_path
                    );
                    continue;
                }
                log::warn!(
                    "No tasks found matching pattern '{}' in '{}'",
                    pattern,
                    task_change.task_path
                );
                continue;
            }

            // Capture matching tasks in parallel
            let task_snapshots: Vec<SchedulerSnapshot> = matching_tasks
                .par_iter()
                .map(|task| {
                    log::trace!(
                        "Captured pattern task: {}\\{} (state: {})",
                        task_change.task_path,
                        task.name,
                        task.state.as_str()
                    );
                    SchedulerSnapshot {
                        task_path: task_change.task_path.clone(),
                        task_name: task.name.clone(),
                        original_state: task.state.as_str().to_string(),
                    }
                })
                .collect();
            snapshots.extend(task_snapshots);
        } else if let Some(ref task_name) = task_change.task_name {
            // Exact task name: capture single task state
            let task_snapshot = capture_scheduler_state(&task_change.task_path, task_name)?;
            snapshots.push(task_snapshot);
        } else {
            log::warn!("Scheduler change has neither task_name nor task_name_pattern, skipping");
        }
    }

    Ok(snapshots)
}

/// Capture hosts entry states
fn capture_hosts_snapshots(
    hosts_changes: &[crate::models::HostsChange],
) -> Result<Vec<HostsSnapshot>, Error> {
    hosts_changes
        .iter()
        .map(|change| {
            let existed = hosts_service::entry_exists(&change.ip, &change.domain)?;
            Ok(HostsSnapshot {
                ip: change.ip.clone(),
                domain: change.domain.clone(),
                existed,
            })
        })
        .collect()
}

/// Capture firewall rule states
fn capture_firewall_snapshots(
    firewall_changes: &[crate::models::FirewallChange],
) -> Result<Vec<FirewallSnapshot>, Error> {
    firewall_changes
        .iter()
        .map(|change| {
            let existed = firewall_service::rule_exists(&change.name)?;
            Ok(FirewallSnapshot {
                name: change.name.clone(),
                existed,
            })
        })
        .collect()
}

/// Capture CURRENT system state for ALL items across ALL options of a tweak (parallelized).
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
    // original_option_index also doesn't matter - this is just for rollback
    let mut snapshot = TweakSnapshot::new(
        &tweak.id,
        &tweak.name,
        usize::MAX, // Marker for "current state" snapshot
        "_current_state_",
        windows_version,
        tweak.requires_system,
        None, // Not relevant for temporary rollback snapshots
    );

    // Collect unique items across all options first
    use std::collections::{HashMap, HashSet};
    let mut unique_registry: HashMap<String, &crate::models::RegistryChange> = HashMap::new();
    let mut unique_services: HashSet<String> = HashSet::new();
    let mut unique_tasks: Vec<(&str, &str)> = Vec::new(); // (path, name)
    let mut unique_task_patterns: Vec<(&str, &str)> = Vec::new(); // (path, pattern)
    let mut unique_hosts: HashMap<String, (&str, &str)> = HashMap::new(); // key -> (ip, domain)
    let mut unique_firewall: HashSet<String> = HashSet::new();

    for option in &tweak.options {
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
            unique_registry.entry(key_id).or_insert(change);
        }

        for sc in &option.service_changes {
            unique_services.insert(sc.name.clone());
        }

        for task_change in &option.scheduler_changes {
            if let Some(ref pattern) = task_change.task_name_pattern {
                unique_task_patterns.push((&task_change.task_path, pattern));
            } else if let Some(ref task_name) = task_change.task_name {
                unique_tasks.push((&task_change.task_path, task_name));
            }
        }

        for hc in &option.hosts_changes {
            let key = format!("{}|{}", hc.ip, hc.domain);
            unique_hosts.entry(key).or_insert((&hc.ip, &hc.domain));
        }

        for fc in &option.firewall_changes {
            unique_firewall.insert(fc.name.clone());
        }
    }

    // Capture all categories in parallel
    let registry_changes: Vec<_> = unique_registry.values().cloned().collect();
    let service_names: Vec<_> = unique_services.iter().cloned().collect();
    let hosts_entries: Vec<_> = unique_hosts.values().cloned().collect();
    let firewall_names: Vec<_> = unique_firewall.iter().cloned().collect();

    let ((registry_result, (services_result, scheduler_result)), (hosts_result, firewall_result)) =
        rayon::join(
            || {
                rayon::join(
                    || {
                        // Parallel registry capture
                        registry_changes
                            .par_iter()
                            .map(|change| {
                                let value_type =
                                    change.value_type.unwrap_or(RegistryValueType::Dword);
                                let (value, existed) = read_registry_value(
                                    &change.hive,
                                    &change.key,
                                    &change.value_name,
                                    &value_type,
                                )?;

                                Ok(RegistrySnapshot {
                                    hive: change.hive.as_str().to_string(),
                                    key: change.key.clone(),
                                    value_name: change.value_name.clone(),
                                    value_type: if existed {
                                        Some(value_type.as_str().to_string())
                                    } else {
                                        None
                                    },
                                    value,
                                    existed,
                                })
                            })
                            .collect::<Result<Vec<_>, Error>>()
                    },
                    || {
                        rayon::join(
                            || {
                                // Parallel service capture
                                service_names
                                    .par_iter()
                                    .map(|name| capture_service_state(name))
                                    .collect::<Result<Vec<_>, Error>>()
                            },
                            || {
                                // Scheduler capture
                                let mut snapshots = Vec::new();
                                let mut captured_tasks_set: HashSet<String> = HashSet::new();

                                for (task_path, pattern) in &unique_task_patterns {
                                    if let Ok(matching_tasks) =
                                        scheduler_service::find_tasks_by_pattern(task_path, pattern)
                                    {
                                        for task in matching_tasks {
                                            let task_id = format!("{}\\{}", task_path, task.name);
                                            if !captured_tasks_set.contains(&task_id) {
                                                captured_tasks_set.insert(task_id);
                                                snapshots.push(SchedulerSnapshot {
                                                    task_path: task_path.to_string(),
                                                    task_name: task.name.clone(),
                                                    original_state: task.state.as_str().to_string(),
                                                });
                                            }
                                        }
                                    }
                                }

                                for (task_path, task_name) in &unique_tasks {
                                    let task_id = format!("{}\\{}", task_path, task_name);
                                    if !captured_tasks_set.contains(&task_id) {
                                        captured_tasks_set.insert(task_id);
                                        match capture_scheduler_state(task_path, task_name) {
                                            Ok(task_snapshot) => snapshots.push(task_snapshot),
                                            Err(e) => {
                                                log::debug!(
                                                "Could not capture state for task {}\\{}: {} (may not exist)",
                                                task_path,
                                                task_name,
                                                e
                                            );
                                            }
                                        }
                                    }
                                }

                                Ok::<_, Error>(snapshots)
                            },
                        )
                    },
                )
            },
            || {
                rayon::join(
                    || {
                        // Hosts capture
                        hosts_entries
                            .iter()
                            .map(|(ip, domain)| {
                                let existed = hosts_service::entry_exists(ip, domain)?;
                                Ok(HostsSnapshot {
                                    ip: ip.to_string(),
                                    domain: domain.to_string(),
                                    existed,
                                })
                            })
                            .collect::<Result<Vec<_>, Error>>()
                    },
                    || {
                        // Firewall capture
                        firewall_names
                            .iter()
                            .map(|name| {
                                let existed = firewall_service::rule_exists(name)?;
                                Ok(FirewallSnapshot {
                                    name: name.clone(),
                                    existed,
                                })
                            })
                            .collect::<Result<Vec<_>, Error>>()
                    },
                )
            },
        );

    // Add results to snapshot
    for reg in registry_result? {
        snapshot.add_registry_snapshot(reg);
    }
    for svc in services_result? {
        snapshot.add_service_snapshot(svc);
    }
    for task in scheduler_result? {
        snapshot.add_scheduler_snapshot(task);
    }
    for host in hosts_result? {
        snapshot.add_hosts_snapshot(host);
    }
    for fw in firewall_result? {
        snapshot.add_firewall_snapshot(fw);
    }

    log::info!(
        "Captured current state: {} registry, {} services, {} tasks, {} hosts, {} firewall for '{}'",
        snapshot.registry_snapshots.len(),
        snapshot.service_snapshots.len(),
        snapshot.scheduler_snapshots.len(),
        snapshot.hosts_snapshots.len(),
        snapshot.firewall_snapshots.len(),
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
pub fn read_registry_value(
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
        RegistryValueType::MultiString => {
            registry_service::read_multi_string(hive, key, value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
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
