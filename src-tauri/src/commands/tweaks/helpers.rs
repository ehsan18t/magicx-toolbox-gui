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
    RegistryAction, RegistryHive, RegistryValueType, SchedulerAction, TweakDefinition, TweakOption,
};
use crate::services::{
    firewall_service, hosts_service, registry_service, scheduler_service, service_control,
    trusted_installer,
};
use tauri::AppHandle;

// ============================================================================
// Command Execution
// ============================================================================

/// Run a shell command (as user, SYSTEM, or TrustedInstaller)
pub fn run_command(cmd: &str, use_system: bool, use_ti: bool) -> Result<()> {
    let elevation_label = if use_ti {
        " as TrustedInstaller"
    } else if use_system {
        " as SYSTEM"
    } else {
        ""
    };
    log::info!("Running command{}: {}", elevation_label, cmd);

    if use_ti {
        // TrustedInstaller has highest privilege
        match trusted_installer::run_command_as_ti(cmd) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    log::warn!("Command (TI) returned exit code {}: {}", exit_code, cmd);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!(
                "TrustedInstaller command failed: {}",
                e
            ))),
        }
    } else if use_system {
        match trusted_installer::run_command_as_system(cmd) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    log::warn!("Command (SYSTEM) returned exit code {}: {}", exit_code, cmd);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!(
                "SYSTEM command failed: {}",
                e
            ))),
        }
    } else {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = std::process::Command::new("cmd")
            .raw_arg(format!("/C {}", cmd))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| Error::CommandExecution(e.to_string()))?;

        if !output.status.success() {
            log::warn!(
                "Command failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }
}

/// Run a PowerShell command (as user, SYSTEM, or TrustedInstaller)
pub fn run_powershell_command(cmd: &str, use_system: bool, use_ti: bool) -> Result<()> {
    let elevation_label = if use_ti {
        " as TrustedInstaller"
    } else if use_system {
        " as SYSTEM"
    } else {
        ""
    };
    log::info!("Running PowerShell{}: {}", elevation_label, cmd);

    if use_ti {
        // TrustedInstaller has highest privilege
        match trusted_installer::run_powershell_as_ti(cmd) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    log::warn!("PowerShell (TI) returned exit code {}", exit_code);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!(
                "PowerShell (TrustedInstaller) failed: {}",
                e
            ))),
        }
    } else if use_system {
        match trusted_installer::run_powershell_as_system(cmd) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    log::warn!("PowerShell (SYSTEM) returned exit code {}", exit_code);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!(
                "PowerShell (SYSTEM) failed: {}",
                e
            ))),
        }
    } else {
        match trusted_installer::run_powershell(cmd) {
            Ok(ps_result) => {
                if ps_result.exit_code != 0 {
                    log::warn!(
                        "PowerShell returned exit code {}: {}",
                        ps_result.exit_code,
                        ps_result.stderr
                    );
                }
                if !ps_result.stdout.is_empty() {
                    log::debug!("PowerShell stdout: {}", ps_result.stdout);
                }
                if !ps_result.stderr.is_empty() && ps_result.exit_code != 0 {
                    log::debug!("PowerShell stderr: {}", ps_result.stderr);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!("PowerShell failed: {}", e))),
        }
    }
}

// ============================================================================
// Registry Operations
// ============================================================================

/// Read a registry value
pub fn read_registry_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
) -> Result<Option<serde_json::Value>> {
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
        Ok(v) => Ok(v),
        Err(Error::RegistryKeyNotFound(_)) => Ok(None),
        Err(e) => {
            log::debug!(
                "Could not read {}\\{}\\{}: {}",
                hive.as_str(),
                key,
                value_name,
                e
            );
            Ok(None)
        }
    }
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
    if use_system {
        return write_registry_value_as_system(hive, key, value_name, value_type, value);
    }

    match value_type {
        RegistryValueType::Dword => {
            let v = value.as_u64().ok_or_else(|| {
                Error::ValidationError(format!(
                    "Expected u64 for DWORD registry value, got: {}",
                    value
                ))
            })?;
            registry_service::set_dword(hive, key, value_name, v as u32)?;
        }
        RegistryValueType::String | RegistryValueType::ExpandString => {
            let v = value.as_str().ok_or_else(|| {
                Error::ValidationError(format!(
                    "Expected string for {} registry value, got: {}",
                    value_type.as_str(),
                    value
                ))
            })?;
            registry_service::set_string(hive, key, value_name, v)?;
        }
        RegistryValueType::Binary => {
            let arr = value.as_array().ok_or_else(|| {
                Error::ValidationError(format!(
                    "Expected array for BINARY registry value, got: {}",
                    value
                ))
            })?;
            let binary: Vec<u8> = arr
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u8))
                .collect();
            registry_service::set_binary(hive, key, value_name, &binary)?;
        }
        RegistryValueType::Qword => {
            let v = value.as_u64().ok_or_else(|| {
                Error::ValidationError(format!(
                    "Expected u64 for QWORD registry value, got: {}",
                    value
                ))
            })?;
            registry_service::set_qword(hive, key, value_name, v)?;
        }
        RegistryValueType::MultiString => {
            return Err(Error::ValidationError(
                "MultiString registry values are not supported for write operations".into(),
            ));
        }
    }

    Ok(())
}

/// Write a registry value as SYSTEM
fn write_registry_value_as_system(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
    value: &serde_json::Value,
) -> Result<()> {
    let value_data = match value_type {
        RegistryValueType::Dword | RegistryValueType::Qword => {
            value.as_u64().map(|v| v.to_string())
        }
        RegistryValueType::String | RegistryValueType::ExpandString => {
            value.as_str().map(|s| format!("\"{}\"", s))
        }
        _ => {
            log::warn!("SYSTEM elevation not supported for {:?}", value_type);
            return Ok(());
        }
    };

    if let Some(data) = value_data {
        trusted_installer::set_registry_value_as_system(
            hive.as_str(),
            key,
            value_name,
            value_type.as_str(),
            &data,
        )?;
    }

    Ok(())
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
    app: &AppHandle,
    tweak: &TweakDefinition,
    option: &TweakOption,
    windows_version: u32,
) -> Result<()> {
    // Step 1: Apply registry changes (already has internal rollback on failure)
    apply_registry_changes(app, tweak, option, windows_version)?;

    // Step 2: Apply service changes - fail-fast, return error for full rollback
    if let Err(e) = apply_service_changes_atomic(option, tweak.requires_system, tweak.requires_ti) {
        log::error!("Service changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 3: Apply scheduler changes - fail-fast, return error for full rollback
    if let Err(e) =
        apply_scheduler_changes_atomic(app, option, tweak.requires_system, tweak.requires_ti)
    {
        log::error!("Scheduler changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 4: Apply hosts file changes - fail-fast, return error for full rollback
    if let Err(e) = apply_hosts_changes_atomic(app, option) {
        log::error!("Hosts file changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 5: Apply firewall changes - fail-fast, return error for full rollback
    if let Err(e) = apply_firewall_changes_atomic(app, option) {
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
    app: &AppHandle,
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
                app,
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
fn apply_service_changes_atomic(
    option: &TweakOption,
    use_system: bool,
    use_ti: bool,
) -> Result<()> {
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

        let elevation = if use_ti {
            " (TrustedInstaller)"
        } else if use_system {
            " (SYSTEM)"
        } else {
            ""
        };

        let start_type = change.startup.to_sc_start_type();

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
                elevation,
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
        } else if use_ti {
            trusted_installer::set_service_startup_as_ti(&change.name, start_type)
        } else if use_system {
            trusted_installer::set_service_startup_as_system(&change.name, start_type)
        } else {
            service_control::set_service_startup(&change.name, &change.startup)
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

        // 2) Stop if desired
        if desired_stop && !matches!(current_state, Some(service_control::ServiceState::Stopped)) {
            if use_ti {
                let _ = trusted_installer::stop_service_as_ti(&change.name);
            } else if use_system {
                let _ = trusted_installer::stop_service_as_system(&change.name);
            } else {
                let _ = service_control::stop_service(&change.name);
            }
        }

        // 3) Start if explicitly requested
        if desired_start && !matches!(current_state, Some(service_control::ServiceState::Running)) {
            if use_ti {
                let _ = trusted_installer::start_service_as_ti(&change.name);
            } else if use_system {
                let _ = trusted_installer::start_service_as_system(&change.name);
            } else {
                let _ = service_control::start_service(&change.name);
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
fn apply_scheduler_changes_atomic(
    app: &AppHandle,
    option: &TweakOption,
    use_system: bool,
    use_ti: bool,
) -> Result<()> {
    let use_elevated = use_ti || use_system;

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

        let elevation_str = if use_ti {
            " (TrustedInstaller)"
        } else if use_system {
            " (SYSTEM)"
        } else {
            ""
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
            apply_scheduler_pattern(app, change, use_elevated, use_ti, &flags_str)?;
        } else {
            apply_scheduler_exact(app, change, use_elevated, use_ti, &flags_str)?;
        }
    }
    Ok(())
}

/// Apply scheduler change for a pattern match
fn apply_scheduler_pattern(
    app: &AppHandle,
    change: &crate::models::SchedulerChange,
    use_elevated: bool,
    use_ti: bool,
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

    if use_elevated {
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
            let escaped_path = trusted_installer::escape_shell_arg(&full_path);
            let schtasks_args = match change.action {
                SchedulerAction::Enable => format!("/Change /TN \"{}\" /Enable", escaped_path),
                SchedulerAction::Disable => format!("/Change /TN \"{}\" /Disable", escaped_path),
                SchedulerAction::Delete => format!("/Delete /TN \"{}\" /F", escaped_path),
            };

            let result = if use_ti {
                trusted_installer::run_schtasks_as_ti(&schtasks_args)
            } else {
                trusted_installer::run_schtasks_as_system(&schtasks_args)
            }
            .map(|_| ())
            .map_err(|e| Error::CommandExecution(e.to_string()));

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
                    app,
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
                app,
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
    app: &AppHandle,
    change: &crate::models::SchedulerChange,
    use_elevated: bool,
    use_ti: bool,
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

    let result = if use_elevated {
        let escaped_path = trusted_installer::escape_shell_arg(&full_path);
        let schtasks_args = match change.action {
            SchedulerAction::Enable => format!("/Change /TN \"{}\" /Enable", escaped_path),
            SchedulerAction::Disable => format!("/Change /TN \"{}\" /Disable", escaped_path),
            SchedulerAction::Delete => format!("/Delete /TN \"{}\" /F", escaped_path),
        };
        if use_ti {
            trusted_installer::run_schtasks_as_ti(&schtasks_args)
        } else {
            trusted_installer::run_schtasks_as_system(&schtasks_args)
        }
        .map(|_| ())
        .map_err(|e| Error::CommandExecution(e.to_string()))
    } else {
        scheduler_service::apply_scheduler_change(&change.task_path, task_name, change.action)
    };

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
            app,
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
fn apply_hosts_changes_atomic(app: &AppHandle, option: &TweakOption) -> Result<()> {
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
                app,
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
fn apply_firewall_changes_atomic(app: &AppHandle, option: &TweakOption) -> Result<()> {
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

            emit_debug_log(
                app,
                DebugLevel::Info,
                &format!("Firewall: {}", details),
                None,
            );
        }
    }

    Ok(())
}
