//! Helper Functions - Internal utilities for tweak operations
//!
//! Contains:
//! - Command execution (shell, PowerShell)
//! - Registry read/write operations
//! - Service change application
//! - Scheduler change application
//! - Hosts file change application
//! - Firewall rule change application
//! - Atomic change orchestration

use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{
    RegistryAction, RegistryHive, RegistryValueType, TweakDefinition, TweakOption,
};
use crate::services::elevation::Elevation;
use crate::services::{
    firewall_service, hosts_service, registry_service, registry_value, scheduler_service,
    service_control, trusted_installer,
};

// ============================================================================
// Command Execution
// ============================================================================

/// Run a shell command (as user, SYSTEM, or TrustedInstaller)
pub fn run_command(cmd: &str, elevation: Elevation) -> Result<()> {
    let label_suffix = if elevation.is_elevated() {
        format!(" as {}", elevation.label())
    } else {
        String::new()
    };
    log::info!("Running command{}: {}", label_suffix, cmd);

    match elevation {
        Elevation::None => {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let output = std::process::Command::new("cmd")
                .raw_arg(format!("/C {}", cmd))
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| Error::CommandExecution(e.to_string()))?;

            if !output.status.success() {
                return Err(Error::CommandExecution(format!(
                    "Command failed with exit code {}: {}",
                    output.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            Ok(())
        }
        // SYSTEM and TrustedInstaller share the same executor signature; only the executor
        // and the label differ.
        elevated => {
            let execute: fn(&str) -> std::result::Result<(), Error> = match elevated {
                Elevation::TrustedInstaller => trusted_installer::run_command_as_ti,
                _ => trusted_installer::run_command_as_system,
            };
            execute(cmd).map_err(|e| {
                Error::CommandExecution(format!("{} command failed: {}", elevated.label(), e))
            })
        }
    }
}

/// Run a PowerShell command (as user, SYSTEM, or TrustedInstaller)
pub fn run_powershell_command(cmd: &str, elevation: Elevation) -> Result<()> {
    let label_suffix = if elevation.is_elevated() {
        format!(" as {}", elevation.label())
    } else {
        String::new()
    };
    log::info!("Running PowerShell{}: {}", label_suffix, cmd);

    match elevation {
        Elevation::None => match trusted_installer::run_powershell(cmd) {
            Ok(ps_result) => {
                if ps_result.exit_code != 0 {
                    return Err(Error::CommandExecution(format!(
                        "PowerShell failed with exit code {}: {}",
                        ps_result.exit_code, ps_result.stderr
                    )));
                }
                if !ps_result.stdout.is_empty() {
                    log::debug!("PowerShell stdout: {}", ps_result.stdout);
                }
                if !ps_result.stderr.is_empty() {
                    log::debug!("PowerShell stderr: {}", ps_result.stderr);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!("PowerShell failed: {}", e))),
        },
        // SYSTEM and TrustedInstaller share the same executor signature.
        elevated => {
            let execute: fn(&str) -> std::result::Result<(), Error> = match elevated {
                Elevation::TrustedInstaller => trusted_installer::run_powershell_as_ti,
                _ => trusted_installer::run_powershell_as_system,
            };
            execute(cmd).map_err(|e| {
                Error::CommandExecution(format!("PowerShell ({}) failed: {}", elevated.label(), e))
            })
        }
    }
}

// ============================================================================
// Registry Operations
// ============================================================================

/// Read a registry value
/// Read a registry value, returning None if it doesn't exist.
/// Delegates to the canonical implementation in backup::capture.
pub fn read_registry_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
) -> Result<Option<serde_json::Value>> {
    let (value, _existed) =
        crate::services::backup::read_registry_value(hive, key, value_name, value_type)?;
    Ok(value)
}

/// Write a registry value
fn write_registry_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
    value: &serde_json::Value,
    use_system: bool,
) -> Result<()> {
    registry_value::write_registry_json_value(hive, key, value_name, value_type, value, use_system)
}

/// Restore a value (guess type from JSON value)
fn restore_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value: &serde_json::Value,
) -> Result<()> {
    if let Some(v) = value.as_u64() {
        if v <= u32::MAX as u64 {
            registry_service::set_dword(hive, key, value_name, v as u32)?;
        } else {
            registry_service::set_qword(hive, key, value_name, v)?;
        }
    } else if let Some(v) = value.as_str() {
        registry_service::set_string(hive, key, value_name, v)?;
    } else if let Some(arr) = value.as_array() {
        let binary: Vec<u8> = arr
            .iter()
            .filter_map(|v| v.as_u64().map(|u| u as u8))
            .collect();
        registry_service::set_binary(hive, key, value_name, &binary)?;
    }
    Ok(())
}

// ============================================================================
// Atomic Change Application
// ============================================================================

/// Apply ALL core changes atomically: registry, services, scheduler, hosts, firewall
/// If any step fails, caller is responsible for full rollback from snapshot
pub fn apply_all_changes_atomically(
    tweak: &TweakDefinition,
    option: &TweakOption,
    windows_version: u32,
) -> Result<()> {
    // Step 1: Apply registry changes (already has internal rollback on failure)
    apply_registry_changes(tweak, option, windows_version)?;

    // Step 2: Apply service changes - fail-fast, return error for full rollback
    if let Err(e) = apply_service_changes_atomic(option, tweak.elevation()) {
        log::error!("Service changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 3: Apply scheduler changes - fail-fast, return error for full rollback
    if let Err(e) = apply_scheduler_changes_atomic(option, tweak.elevation()) {
        log::error!("Scheduler changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 4: Apply hosts file changes - fail-fast, return error for full rollback
    if let Err(e) = apply_hosts_changes_atomic(option) {
        log::error!("Hosts file changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 5: Apply firewall changes - fail-fast, return error for full rollback
    if let Err(e) = apply_firewall_changes_atomic(option) {
        log::error!("Firewall changes failed, need full rollback: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Rollback info for atomic registry operations
enum RegistryRollback {
    /// Restore a value that was set (delete if None, restore if Some)
    RestoreValue {
        hive: RegistryHive,
        key: String,
        value_name: String,
        original: Option<serde_json::Value>,
    },
    /// Recreate a key that was deleted.
    /// NOTE: This is best-effort only - subkeys and values within the deleted key cannot be
    /// restored. This is acceptable because delete_key is typically used to remove keys that
    /// were created by the opposite option (e.g., context menu CLSID entries).
    RecreateKey { hive: RegistryHive, key: String },
    /// Delete a key that was created
    DeleteKey { hive: RegistryHive, key: String },
}

/// Apply all registry changes for an option atomically
fn apply_registry_changes(
    tweak: &TweakDefinition,
    option: &TweakOption,
    windows_version: u32,
) -> Result<()> {
    let mut rollbacks: Vec<RegistryRollback> = Vec::new();

    for change in &option.registry_changes {
        // Skip if not for this Windows version
        if !change.applies_to_version(windows_version) {
            continue;
        }

        let full_path = format!(
            "{}\\{}{}",
            change.hive.as_str(),
            change.key,
            if change.value_name.is_empty() {
                String::new()
            } else {
                format!("\\{}", change.value_name)
            }
        );

        let result = match change.action {
            RegistryAction::Set => {
                // Set action - write a value
                let value_type = match &change.value_type {
                    Some(vt) => vt,
                    None => {
                        log::error!("Set action requires value_type: {}", full_path);
                        return Err(Error::ValidationError(
                            "Set action requires value_type".into(),
                        ));
                    }
                };
                let value = match &change.value {
                    Some(v) => v,
                    None => {
                        log::error!("Set action requires value: {}", full_path);
                        return Err(Error::ValidationError("Set action requires value".into()));
                    }
                };

                // Read current value for rollback (only for validatable changes)
                let current = if !change.skip_validation {
                    read_registry_value(&change.hive, &change.key, &change.value_name, value_type)?
                } else {
                    None
                };

                log::debug!(
                    "Setting{} {} = {:?} (was {:?})",
                    if change.skip_validation {
                        " (skip_validation)"
                    } else {
                        ""
                    },
                    full_path,
                    value,
                    current
                );

                let write_result = write_registry_value(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    value_type,
                    value,
                    tweak.requires_system,
                );

                if write_result.is_ok() && !change.skip_validation {
                    rollbacks.push(RegistryRollback::RestoreValue {
                        hive: change.hive,
                        key: change.key.clone(),
                        value_name: change.value_name.clone(),
                        original: current,
                    });
                }

                write_result
            }

            RegistryAction::DeleteValue => {
                log::debug!(
                    "Deleting value{} {}",
                    if change.skip_validation {
                        " (skip_validation)"
                    } else {
                        ""
                    },
                    full_path
                );

                // Read current value for rollback
                let current = if !change.skip_validation {
                    // Try to detect type and read - use DWORD as default
                    let value_type = change.value_type.unwrap_or(RegistryValueType::Dword);
                    read_registry_value(&change.hive, &change.key, &change.value_name, &value_type)?
                } else {
                    None
                };

                let delete_result =
                    registry_service::delete_value(&change.hive, &change.key, &change.value_name);

                // Treat not-found as success for delete operations
                let result = match delete_result {
                    Err(Error::RegistryKeyNotFound(_)) => Ok(()),
                    other => other,
                };

                if result.is_ok() && !change.skip_validation && current.is_some() {
                    rollbacks.push(RegistryRollback::RestoreValue {
                        hive: change.hive,
                        key: change.key.clone(),
                        value_name: change.value_name.clone(),
                        original: current,
                    });
                }

                result
            }

            RegistryAction::DeleteKey => {
                log::debug!(
                    "Deleting key{} {}",
                    if change.skip_validation {
                        " (skip_validation)"
                    } else {
                        ""
                    },
                    full_path
                );

                // Check if key exists for rollback tracking
                let key_existed = if !change.skip_validation {
                    registry_service::key_exists(&change.hive, &change.key).unwrap_or(false)
                } else {
                    false
                };

                let delete_result = registry_service::delete_key(&change.hive, &change.key);

                // Treat not-found as success for delete operations
                let result = match delete_result {
                    Err(Error::RegistryKeyNotFound(_)) => Ok(()),
                    other => other,
                };

                if result.is_ok() && !change.skip_validation && key_existed {
                    rollbacks.push(RegistryRollback::RecreateKey {
                        hive: change.hive,
                        key: change.key.clone(),
                    });
                }

                result
            }

            RegistryAction::CreateKey => {
                log::debug!(
                    "Creating key{} {}",
                    if change.skip_validation {
                        " (skip_validation)"
                    } else {
                        ""
                    },
                    full_path
                );

                // Check if key already exists for rollback
                let key_existed = if !change.skip_validation {
                    registry_service::key_exists(&change.hive, &change.key).unwrap_or(false)
                } else {
                    false
                };

                let create_result = registry_service::create_key(&change.hive, &change.key);

                if create_result.is_ok() && !change.skip_validation && !key_existed {
                    rollbacks.push(RegistryRollback::DeleteKey {
                        hive: change.hive,
                        key: change.key.clone(),
                    });
                }

                create_result
            }
        };

        // Handle errors
        if let Err(e) = result {
            if change.skip_validation {
                log::warn!(
                    "Failed {:?} on {} (skip_validation, continuing): {}",
                    change.action,
                    full_path,
                    e
                );
                continue;
            }

            log::error!("Failed {:?} on {}: {}", change.action, full_path, e);

            // Rollback applied changes
            for rollback in rollbacks.iter().rev() {
                match rollback {
                    RegistryRollback::RestoreValue {
                        hive,
                        key,
                        value_name,
                        original,
                    } => {
                        if let Some(val) = original {
                            let _ = restore_value(hive, key, value_name, val);
                        } else {
                            let _ = registry_service::delete_value(hive, key, value_name);
                        }
                    }
                    RegistryRollback::RecreateKey { hive, key } => {
                        // Best effort - just create the key (values are lost)
                        let _ = registry_service::create_key(hive, key);
                    }
                    RegistryRollback::DeleteKey { hive, key } => {
                        let _ = registry_service::delete_key(hive, key);
                    }
                }
            }

            return Err(e);
        }

        // Debug logging
        if is_debug_enabled() {
            let action_str = match change.action {
                RegistryAction::Set => format!("Set {:?}", change.value),
                RegistryAction::DeleteValue => "Deleted value".to_string(),
                RegistryAction::DeleteKey => "Deleted key".to_string(),
                RegistryAction::CreateKey => "Created key".to_string(),
            };
            emit_debug_log(
                DebugLevel::Info,
                &format!(
                    "{}{} {}",
                    if change.skip_validation { "[sv] " } else { "" },
                    action_str,
                    full_path
                ),
                None,
            );
        }
    }

    log::debug!("Applied {} registry changes", rollbacks.len());
    Ok(())
}

/// Apply all service changes for an option atomically
fn apply_service_changes_atomic(option: &TweakOption, elevation: Elevation) -> Result<()> {
    for change in &option.service_changes {
        let status = match service_control::get_service_status(&change.name) {
            Ok(status) => Some(status),
            Err(e) => {
                log::warn!(
                    "Could not query service '{}' status (continuing anyway): {}",
                    change.name,
                    e
                );
                None
            }
        };

        let current_startup = status.as_ref().and_then(|s| s.startup_type);
        let current_state = status.as_ref().map(|s| &s.state);

        let desired_stop =
            change.startup == crate::models::ServiceStartupType::Disabled || change.stop_service;
        let desired_start = change.start_service && !desired_stop;

        let startup_matches = current_startup == Some(change.startup);

        let elevation_label = if elevation.is_elevated() {
            format!(" ({})", elevation.label())
        } else {
            String::new()
        };

        // 1) Startup config
        if startup_matches {
            log::info!(
                "Service '{}' already at startup '{:?}', skipping config",
                change.name,
                change.startup
            );
        } else {
            log::info!(
                "Setting service{}{} '{}' startup to {:?}",
                elevation_label,
                if change.skip_validation {
                    " (skip_validation)"
                } else {
                    ""
                },
                change.name,
                change.startup
            );
        }

        let result = if startup_matches {
            Ok(())
        } else {
            match elevation {
                Elevation::TrustedInstaller => {
                    trusted_installer::set_service_startup_as_ti(&change.name, &change.startup)
                }
                Elevation::System => {
                    trusted_installer::set_service_startup_as_system(&change.name, &change.startup)
                }
                Elevation::None => {
                    service_control::set_service_startup(&change.name, &change.startup)
                }
            }
        };

        if let Err(e) = result {
            if change.skip_validation {
                log::warn!(
                    "Failed to set service '{}' startup (skip_validation, continuing): {}",
                    change.name,
                    e
                );
                continue;
            }
            return Err(Error::ServiceControl(format!(
                "Failed to set service '{}' startup: {}",
                change.name, e
            )));
        }

        // 2) Stop if desired. A failed stop is a real failure (honoring skip_validation) — not a
        // silently-swallowed no-op.
        if desired_stop && !matches!(current_state, Some(service_control::ServiceState::Stopped)) {
            let stop_result = match elevation {
                Elevation::TrustedInstaller => trusted_installer::stop_service_as_ti(&change.name),
                Elevation::System => trusted_installer::stop_service_as_system(&change.name),
                Elevation::None => service_control::stop_service(&change.name),
            };
            if let Err(e) = stop_result {
                if change.skip_validation {
                    log::warn!(
                        "Failed to stop service '{}' (skip_validation, continuing): {}",
                        change.name,
                        e
                    );
                } else {
                    return Err(Error::ServiceControl(format!(
                        "Failed to stop service '{}': {}",
                        change.name, e
                    )));
                }
            }
        }

        // 3) Start if explicitly requested. Likewise surfaced, so a service that fails to start is
        // not reported as a successful apply.
        if desired_start && !matches!(current_state, Some(service_control::ServiceState::Running)) {
            let start_result = match elevation {
                Elevation::TrustedInstaller => trusted_installer::start_service_as_ti(&change.name),
                Elevation::System => trusted_installer::start_service_as_system(&change.name),
                Elevation::None => service_control::start_service(&change.name),
            };
            if let Err(e) = start_result {
                if change.skip_validation {
                    log::warn!(
                        "Failed to start service '{}' (skip_validation, continuing): {}",
                        change.name,
                        e
                    );
                } else {
                    return Err(Error::ServiceControl(format!(
                        "Failed to start service '{}': {}",
                        change.name, e
                    )));
                }
            }
        }

        if startup_matches
            && (!desired_stop
                || matches!(current_state, Some(service_control::ServiceState::Stopped)))
            && (!desired_start
                || matches!(current_state, Some(service_control::ServiceState::Running)))
        {
            log::info!(
                "Service '{}' already matches requested startup/state; skipping",
                change.name
            );
        }
    }
    Ok(())
}

/// Apply all scheduler changes for an option atomically
fn apply_scheduler_changes_atomic(option: &TweakOption, elevation: Elevation) -> Result<()> {
    for change in &option.scheduler_changes {
        let is_pattern = change.task_name_pattern.is_some();
        let identifier = if let Some(ref pattern) = change.task_name_pattern {
            pattern.clone()
        } else if let Some(ref name) = change.task_name {
            name.clone()
        } else {
            log::error!("Scheduler change has neither task_name nor task_name_pattern");
            return Err(Error::CommandExecution(
                "Scheduler change requires either task_name or task_name_pattern".to_string(),
            ));
        };

        let elevation_str = if elevation.is_elevated() {
            format!(" ({})", elevation.label())
        } else {
            String::new()
        };

        let flags_str = {
            let mut flags = Vec::new();
            if change.skip_validation {
                flags.push("skip_validation");
            }
            if change.ignore_not_found {
                flags.push("ignore_not_found");
            }
            if is_pattern {
                flags.push("pattern");
            }
            if flags.is_empty() {
                String::new()
            } else {
                format!(" ({})", flags.join(", "))
            }
        };

        log::info!(
            "Applying scheduler change{}{}: {}\\{} → {:?}",
            elevation_str,
            flags_str,
            change.task_path,
            identifier,
            change.action
        );

        if is_pattern {
            apply_scheduler_pattern(change, elevation, &flags_str)?;
        } else {
            apply_scheduler_exact(change, elevation, &flags_str)?;
        }
    }
    Ok(())
}

/// Apply scheduler change for a pattern match
fn apply_scheduler_pattern(
    change: &crate::models::SchedulerChange,
    elevation: Elevation,
    flags_str: &str,
) -> Result<()> {
    let pattern = match change.task_name_pattern.as_deref() {
        Some(p) => p,
        None => {
            return Err(Error::ValidationError(format!(
                "Scheduler change in '{}' expected task_name_pattern but it was missing",
                change.task_path
            )))
        }
    };

    if elevation.is_elevated() {
        // Resolve matching task names (read-only), then apply each through the typed op below.
        let tasks = scheduler_service::find_tasks_by_pattern(&change.task_path, pattern)?;

        if tasks.is_empty() {
            if change.ignore_not_found || change.skip_validation {
                log::warn!(
                    "No tasks found matching pattern '{}' in '{}' ({})",
                    pattern,
                    change.task_path,
                    if change.ignore_not_found {
                        "ignore_not_found"
                    } else {
                        "skip_validation"
                    }
                );
                return Ok(());
            } else {
                return Err(Error::CommandExecution(format!(
                    "No tasks found matching pattern '{}' in '{}'",
                    pattern, change.task_path
                )));
            }
        }

        for task in tasks {
            let full_path = format!("{}\\{}", change.task_path, task.name);
            let result = trusted_installer::run_scheduler_op(
                elevation,
                &change.task_path,
                &task.name,
                change.action,
            );

            if let Err(e) = result {
                if change.skip_validation {
                    log::warn!(
                        "Failed to apply scheduler change for '{}' (skip_validation, continuing): {}",
                        full_path,
                        e
                    );
                    continue;
                }
                return Err(Error::CommandExecution(format!(
                    "Failed to apply scheduler change for '{}': {}",
                    full_path, e
                )));
            }

            if is_debug_enabled() {
                emit_debug_log(
                    DebugLevel::Info,
                    &format!(
                        "Scheduler{}: {} → {:?}",
                        flags_str, full_path, change.action
                    ),
                    None,
                );
            }
        }
    } else {
        let (success_count, error_count, errors) = scheduler_service::apply_action_to_pattern(
            &change.task_path,
            pattern,
            change.action,
            change.ignore_not_found,
        )?;

        if error_count > 0 {
            if change.skip_validation {
                log::warn!(
                    "Pattern '{}': {} succeeded, {} failed (skip_validation): {:?}",
                    pattern,
                    success_count,
                    error_count,
                    errors
                );
            } else {
                return Err(Error::CommandExecution(format!(
                    "Pattern '{}': {} succeeded, {} failed: {:?}",
                    pattern, success_count, error_count, errors
                )));
            }
        }

        if is_debug_enabled() {
            emit_debug_log(
                DebugLevel::Info,
                &format!(
                    "Scheduler{}: {}\\[{}] → {:?} ({} tasks)",
                    flags_str, change.task_path, pattern, change.action, success_count
                ),
                None,
            );
        }
    }
    Ok(())
}

/// Apply scheduler change for an exact task name
fn apply_scheduler_exact(
    change: &crate::models::SchedulerChange,
    elevation: Elevation,
    flags_str: &str,
) -> Result<()> {
    let task_name = match change.task_name.as_deref() {
        Some(n) => n,
        None => {
            return Err(Error::ValidationError(format!(
                "Scheduler change in '{}' expected task_name but it was missing",
                change.task_path
            )))
        }
    };
    let full_path = format!("{}\\{}", change.task_path, task_name);

    // One typed op for every elevation: unelevated runs it in-process, SYSTEM/TI run it inside the
    // broker. No schtasks string, so a task name with cmd metacharacters can't be corrupted (C3).
    let result =
        trusted_installer::run_scheduler_op(elevation, &change.task_path, task_name, change.action);

    if let Err(e) = result {
        let is_not_found =
            e.to_string().contains("does not exist") || e.to_string().contains("cannot find");

        if is_not_found && change.ignore_not_found {
            log::warn!(
                "Task '{}' not found (ignore_not_found, continuing)",
                full_path
            );
            return Ok(());
        } else if change.skip_validation {
            log::warn!(
                "Failed to apply scheduler change for '{}' (skip_validation, continuing): {}",
                full_path,
                e
            );
            return Ok(());
        } else {
            return Err(Error::CommandExecution(format!(
                "Failed to apply scheduler change for '{}': {}",
                full_path, e
            )));
        }
    }

    if is_debug_enabled() {
        emit_debug_log(
            DebugLevel::Info,
            &format!(
                "Scheduler{}: {} → {:?}",
                flags_str, full_path, change.action
            ),
            None,
        );
    }

    Ok(())
}

// ============================================================================
// Hosts File Operations
// ============================================================================

/// Apply all hosts file changes atomically
fn apply_hosts_changes_atomic(option: &TweakOption) -> Result<()> {
    if option.hosts_changes.is_empty() {
        return Ok(());
    }

    log::debug!("Applying {} hosts file changes", option.hosts_changes.len());

    for change in &option.hosts_changes {
        let action_str = change.action.as_str();
        let entry_desc = format!("{} → {}", change.domain, change.ip);

        log::debug!("Hosts change: {} {}", action_str, entry_desc);

        let result = hosts_service::apply_hosts_change(change);

        if let Err(e) = result {
            if change.skip_validation {
                log::warn!(
                    "Failed to apply hosts change for '{}' (skip_validation, continuing): {}",
                    entry_desc,
                    e
                );
                continue;
            } else {
                return Err(Error::CommandExecution(format!(
                    "Failed to apply hosts change for '{}': {}",
                    entry_desc, e
                )));
            }
        }

        if is_debug_enabled() {
            emit_debug_log(
                DebugLevel::Info,
                &format!("Hosts: {} {}", action_str, entry_desc),
                None,
            );
        }
    }

    Ok(())
}

// ============================================================================
// Firewall Operations
// ============================================================================

/// Apply all firewall rule changes atomically
fn apply_firewall_changes_atomic(option: &TweakOption) -> Result<()> {
    if option.firewall_changes.is_empty() {
        return Ok(());
    }

    log::debug!(
        "Applying {} firewall rule changes",
        option.firewall_changes.len()
    );

    for change in &option.firewall_changes {
        let op_str = change.operation.as_str();
        log::debug!("Firewall change: {} rule '{}'", op_str, change.name);

        let result = firewall_service::apply_firewall_change(change);

        if let Err(e) = result {
            if change.skip_validation {
                log::warn!(
                    "Failed to apply firewall change for '{}' (skip_validation, continuing): {}",
                    change.name,
                    e
                );
                continue;
            } else {
                return Err(Error::CommandExecution(format!(
                    "Failed to apply firewall change for '{}': {}",
                    change.name, e
                )));
            }
        }

        if is_debug_enabled() {
            let details = match change.operation {
                crate::models::FirewallOperation::Create => {
                    let dir = change.direction.map(|d| d.as_str()).unwrap_or("?");
                    let act = change.action.map(|a| a.as_str()).unwrap_or("?");
                    format!("{} {} {}", dir, act, change.name)
                }
                crate::models::FirewallOperation::Delete => {
                    format!("delete {}", change.name)
                }
            };

            emit_debug_log(DebugLevel::Info, &format!("Firewall: {}", details), None);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_returns_error_on_nonzero_exit_code() {
        let err = run_command("exit /b 7", Elevation::None).unwrap_err();

        assert!(err.to_string().contains("exit code 7"));
    }

    #[test]
    fn powershell_returns_error_on_nonzero_exit_code() {
        let err = run_powershell_command("exit 7", Elevation::None).unwrap_err();

        assert!(err.to_string().contains("exit code 7"));
    }
}
