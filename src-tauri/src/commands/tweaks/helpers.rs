//! Helper Functions - Internal utilities for tweak operations
//!
//! Contains:
//! - Command execution (shell, PowerShell)
//! - Registry read/write operations
//! - Service change application
//! - Scheduler change application
//! - Atomic change orchestration

use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{
    RegistryHive, RegistryValueType, SchedulerAction, TweakDefinition, TweakOption,
};
use crate::services::{registry_service, scheduler_service, service_control, trusted_installer};
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
            if let Some(v) = value.as_u64() {
                registry_service::set_dword(hive, key, value_name, v as u32)?;
            }
        }
        RegistryValueType::String | RegistryValueType::ExpandString => {
            if let Some(v) = value.as_str() {
                registry_service::set_string(hive, key, value_name, v)?;
            }
        }
        RegistryValueType::Binary => {
            if let Some(arr) = value.as_array() {
                let binary: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                registry_service::set_binary(hive, key, value_name, &binary)?;
            }
        }
        RegistryValueType::Qword => {
            if let Some(v) = value.as_u64() {
                registry_service::set_qword(hive, key, value_name, v)?;
            }
        }
        RegistryValueType::MultiString => {
            log::warn!("MultiString not supported for write");
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

/// Apply ALL core changes atomically: registry, services, scheduler
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

    Ok(())
}

/// Apply all registry changes for an option atomically
fn apply_registry_changes(
    app: &AppHandle,
    tweak: &TweakDefinition,
    option: &TweakOption,
    windows_version: u32,
) -> Result<()> {
    let mut applied: Vec<(RegistryHive, String, String, Option<serde_json::Value>)> = Vec::new();

    for change in &option.registry_changes {
        // Skip if not for this Windows version
        if !change.applies_to_version(windows_version) {
            continue;
        }

        let full_path = format!(
            "{}\\{}\\{}",
            change.hive.as_str(),
            change.key,
            change.value_name
        );

        // Read current value for rollback (only for validatable changes)
        let current = if !change.skip_validation {
            read_registry_value(
                &change.hive,
                &change.key,
                &change.value_name,
                &change.value_type,
            )?
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
            change.value,
            current
        );

        // Write new value
        if let Err(e) = write_registry_value(
            &change.hive,
            &change.key,
            &change.value_name,
            &change.value_type,
            &change.value,
            tweak.requires_system,
        ) {
            if change.skip_validation {
                log::warn!(
                    "Failed to write {} (skip_validation, continuing): {}",
                    full_path,
                    e
                );
                continue;
            }

            log::error!("Failed to write {}: {}", full_path, e);

            // Rollback applied changes
            for (hive, key, value_name, original) in applied.iter().rev() {
                if let Some(val) = original {
                    let _ = restore_value(hive, key, value_name, val);
                } else {
                    let _ = registry_service::delete_value(hive, key, value_name);
                }
            }

            return Err(e);
        }

        // Only track for rollback if NOT skip_validation
        if !change.skip_validation {
            applied.push((
                change.hive,
                change.key.clone(),
                change.value_name.clone(),
                current,
            ));
        }

        if is_debug_enabled() {
            emit_debug_log(
                app,
                DebugLevel::Info,
                &format!(
                    "Set{}{}",
                    if change.skip_validation { " [sv]" } else { "" },
                    full_path
                ),
                Some(&format!("{:?}", change.value)),
            );
        }
    }

    log::debug!("Applied {} registry changes", applied.len());
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
    let pattern = change.task_name_pattern.as_ref().unwrap();

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
            let schtasks_args = match change.action {
                SchedulerAction::Enable => format!("/Change /TN \"{}\" /Enable", full_path),
                SchedulerAction::Disable => format!("/Change /TN \"{}\" /Disable", full_path),
                SchedulerAction::Delete => format!("/Delete /TN \"{}\" /F", full_path),
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
    let task_name = change.task_name.as_ref().unwrap();
    let full_path = format!("{}\\{}", change.task_path, task_name);

    let result = if use_elevated {
        let schtasks_args = match change.action {
            SchedulerAction::Enable => format!("/Change /TN \"{}\" /Enable", full_path),
            SchedulerAction::Disable => format!("/Change /TN \"{}\" /Disable", full_path),
            SchedulerAction::Delete => format!("/Delete /TN \"{}\" /F", full_path),
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
