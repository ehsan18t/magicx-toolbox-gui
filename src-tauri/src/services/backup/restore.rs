//! Restore Operations
//!
//! Functions for restoring system state from snapshots:
//! - Atomic restore with rollback on failure
//! - Registry value restoration (normal and elevated)
//! - Service and scheduler state restoration (with SYSTEM elevation support)

use crate::error::Error;
use crate::models::{
    RegistryHive, RegistryValueType, SchedulerSnapshot, ServiceSnapshot, TweakSnapshot,
};
use crate::services::{registry_service, scheduler_service, service_control, trusted_installer};

use super::capture::read_registry_value;
use super::helpers::{parse_hive, parse_value_type};

/// Result of a restore operation with detailed failure information
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Whether all restore operations succeeded
    pub success: bool,
    /// List of failures (empty if success is true)
    pub failures: Vec<String>,
}

/// Restore all registry/service values from snapshot
///
/// Registry operations are atomic (rollback on failure).
/// Service and scheduler operations use SYSTEM elevation when needed and collect
/// failures rather than stopping early, allowing partial restore.
///
/// Returns a RestoreResult with details about what succeeded/failed.
pub fn restore_from_snapshot(snapshot: &TweakSnapshot) -> Result<RestoreResult, Error> {
    log::info!(
        "Restoring from snapshot for tweak '{}' (was option '{}', requires_system={})",
        snapshot.tweak_name,
        snapshot.applied_option_label,
        snapshot.requires_system
    );

    let mut failures: Vec<String> = Vec::new();

    // Phase 1: Restore registry values (atomic - all or nothing)
    let mut completed_registry: Vec<(RegistryRestoreOp, Option<serde_json::Value>, bool)> =
        Vec::new();

    for reg in &snapshot.registry_snapshots {
        let hive = parse_hive(&reg.hive)?;

        // Capture current value for rollback
        let value_type = reg
            .value_type
            .as_ref()
            .map(|t| parse_value_type(t))
            .transpose()?
            .unwrap_or(RegistryValueType::Dword);

        let (current_value, current_exists) =
            read_registry_value(&hive, &reg.key, &reg.value_name, &value_type)?;

        let op = RegistryRestoreOp {
            hive,
            key: reg.key.clone(),
            value_name: reg.value_name.clone(),
            value_type: reg.value_type.clone(),
            value: reg.value.clone(),
            existed: reg.existed,
        };

        match execute_registry_restore(&op, snapshot.requires_system) {
            Ok(()) => {
                completed_registry.push((op, current_value, current_exists));
            }
            Err(e) => {
                log::error!(
                    "Restore failed for {}\\{}\\{}: {}",
                    reg.hive,
                    reg.key,
                    reg.value_name,
                    e
                );
                rollback_registry_operations(&completed_registry);
                return Err(Error::BackupFailed(format!(
                    "Failed to restore registry, rolled back {} changes: {}",
                    completed_registry.len(),
                    e
                )));
            }
        }
    }

    // Phase 2: Restore service states (collect failures, don't stop)
    for svc in &snapshot.service_snapshots {
        if let Err(e) = restore_service_state(svc, snapshot.requires_system) {
            let msg = format!("Service '{}': {}", svc.name, e);
            log::error!("Failed to restore service: {}", msg);
            failures.push(msg);
        }
    }

    // Phase 3: Restore scheduled task states (with SYSTEM elevation if needed)
    for task in &snapshot.scheduler_snapshots {
        if let Err(e) = restore_scheduler_state(task, snapshot.requires_system) {
            let msg = format!("Task '{}\\{}': {}", task.task_path, task.task_name, e);
            log::error!("Failed to restore task: {}", msg);
            failures.push(msg);
        }
    }

    let success = failures.is_empty();

    if success {
        log::info!(
            "Successfully restored {} registry, {} services, {} tasks",
            completed_registry.len(),
            snapshot.service_snapshots.len(),
            snapshot.scheduler_snapshots.len()
        );
    } else {
        log::warn!(
            "Restore completed with {} failures out of {} registry, {} services, {} tasks",
            failures.len(),
            snapshot.registry_snapshots.len(),
            snapshot.service_snapshots.len(),
            snapshot.scheduler_snapshots.len()
        );
    }

    Ok(RestoreResult { success, failures })
}

#[derive(Clone)]
struct RegistryRestoreOp {
    hive: RegistryHive,
    key: String,
    value_name: String,
    value_type: Option<String>,
    value: Option<serde_json::Value>,
    existed: bool,
}

/// Execute a single registry restore operation
fn execute_registry_restore(op: &RegistryRestoreOp, use_system: bool) -> Result<(), Error> {
    if !op.existed {
        // Value didn't exist - delete it
        log::debug!(
            "Deleting {}\\{}\\{} (didn't exist originally)",
            op.hive.as_str(),
            op.key,
            op.value_name
        );

        if use_system {
            let _ = trusted_installer::delete_registry_value_as_system(
                op.hive.as_str(),
                &op.key,
                &op.value_name,
            );
        } else {
            let _ = registry_service::delete_value(&op.hive, &op.key, &op.value_name);
        }
        Ok(())
    } else if let (Some(value), Some(value_type)) = (&op.value, &op.value_type) {
        // Restore the original value
        log::debug!(
            "Restoring {}\\{}\\{} = {:?}",
            op.hive.as_str(),
            op.key,
            op.value_name,
            value
        );

        if use_system {
            restore_registry_with_system(&op.hive, &op.key, &op.value_name, value_type, value)
        } else {
            restore_registry_normal(&op.hive, &op.key, &op.value_name, value_type, value)
        }
    } else {
        log::warn!(
            "Skipping restore for {}\\{}\\{}: existed but no value/type",
            op.hive.as_str(),
            op.key,
            op.value_name
        );
        Ok(())
    }
}

fn restore_registry_normal(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &str,
    value: &serde_json::Value,
) -> Result<(), Error> {
    match value_type {
        "REG_DWORD" => {
            if let Some(v) = value.as_u64() {
                registry_service::set_dword(hive, key, value_name, v as u32)?;
            }
        }
        "REG_SZ" | "REG_EXPAND_SZ" => {
            if let Some(v) = value.as_str() {
                registry_service::set_string(hive, key, value_name, v)?;
            }
        }
        "REG_BINARY" => {
            if let Some(arr) = value.as_array() {
                let binary: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                registry_service::set_binary(hive, key, value_name, &binary)?;
            }
        }
        "REG_QWORD" => {
            if let Some(v) = value.as_u64() {
                registry_service::set_qword(hive, key, value_name, v)?;
            }
        }
        _ => {
            return Err(Error::BackupFailed(format!(
                "Unsupported value type: {}",
                value_type
            )));
        }
    }
    Ok(())
}

fn restore_registry_with_system(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &str,
    value: &serde_json::Value,
) -> Result<(), Error> {
    let value_data = match value_type {
        "REG_DWORD" | "REG_QWORD" => value.as_u64().map(|v| v.to_string()),
        "REG_SZ" | "REG_EXPAND_SZ" => value.as_str().map(|s| format!("\"{}\"", s)),
        _ => {
            log::warn!("SYSTEM elevation not supported for {}", value_type);
            return Ok(());
        }
    };

    if let Some(data) = value_data {
        trusted_installer::set_registry_value_as_system(
            hive.as_str(),
            key,
            value_name,
            value_type,
            &data,
        )?;
    }
    Ok(())
}

fn restore_service_state(snapshot: &ServiceSnapshot, use_system: bool) -> Result<(), Error> {
    log::debug!(
        "Restoring service '{}' to startup='{}', was_running={}",
        snapshot.name,
        snapshot.startup_type,
        snapshot.was_running
    );

    let startup = match snapshot.startup_type.as_str() {
        "disabled" => crate::models::ServiceStartupType::Disabled,
        "manual" => crate::models::ServiceStartupType::Manual,
        "automatic" => crate::models::ServiceStartupType::Automatic,
        "boot" => crate::models::ServiceStartupType::Boot,
        "system" => crate::models::ServiceStartupType::System,
        _ => {
            log::warn!("Unknown startup type: {}", snapshot.startup_type);
            return Ok(());
        }
    };

    // Set startup type (with SYSTEM elevation if needed)
    if use_system {
        let start_type = startup.to_sc_start_type();
        trusted_installer::set_service_startup_as_system(&snapshot.name, start_type)?;
    } else {
        service_control::set_service_startup(&snapshot.name, &startup)?;
    }

    // Start/stop the service (best effort - don't fail if this part fails)
    if snapshot.was_running {
        if use_system {
            let _ = trusted_installer::start_service_as_system(&snapshot.name);
        } else {
            let _ = service_control::start_service(&snapshot.name);
        }
    } else if use_system {
        let _ = trusted_installer::stop_service_as_system(&snapshot.name);
    } else {
        let _ = service_control::stop_service(&snapshot.name);
    }

    log::info!(
        "Restored service '{}' to startup '{}'",
        snapshot.name,
        snapshot.startup_type
    );
    Ok(())
}

fn restore_scheduler_state(snapshot: &SchedulerSnapshot, use_system: bool) -> Result<(), Error> {
    let task_path = format!("{}\\{}", snapshot.task_path, snapshot.task_name);
    log::debug!(
        "Restoring scheduled task '{}' to state: {} (use_system={})",
        task_path,
        snapshot.original_state,
        use_system
    );

    match snapshot.original_state.as_str() {
        "Ready" | "Running" => {
            // Task was enabled, re-enable it
            if use_system {
                let escaped_path = trusted_installer::escape_shell_arg(&task_path);
                let args = format!("/Change /TN \"{}\" /Enable", escaped_path);
                let exit_code = trusted_installer::run_schtasks_as_system(&args)?;
                if exit_code != 0 {
                    return Err(Error::CommandExecution(format!(
                        "schtasks enable failed with exit code {}",
                        exit_code
                    )));
                }
                log::info!("Enabled scheduled task (SYSTEM): {}", task_path);
            } else {
                scheduler_service::enable_task(&snapshot.task_path, &snapshot.task_name)?;
            }
        }
        "Disabled" => {
            // Task was disabled, ensure it's disabled
            if use_system {
                let escaped_path = trusted_installer::escape_shell_arg(&task_path);
                let args = format!("/Change /TN \"{}\" /Disable", escaped_path);
                let exit_code = trusted_installer::run_schtasks_as_system(&args)?;
                if exit_code != 0 {
                    return Err(Error::CommandExecution(format!(
                        "schtasks disable failed with exit code {}",
                        exit_code
                    )));
                }
                log::info!("Disabled scheduled task (SYSTEM): {}", task_path);
            } else {
                scheduler_service::disable_task(&snapshot.task_path, &snapshot.task_name)?;
            }
        }
        "NotFound" => {
            // Task didn't exist before - we can't restore a deleted task
            // This is expected if the tweak was a "delete" action
            log::info!(
                "Task '{}' was not found before tweak, cannot restore (expected for delete actions)",
                task_path
            );
        }
        _ => {
            log::warn!(
                "Unknown scheduler state '{}' for task '{}', skipping restore",
                snapshot.original_state,
                task_path
            );
        }
    }

    Ok(())
}

/// Rollback completed registry operations on failure
fn rollback_registry_operations(
    completed: &[(RegistryRestoreOp, Option<serde_json::Value>, bool)],
) {
    log::warn!("Rolling back {} registry operations", completed.len());

    for (op, original_value, original_existed) in completed.iter().rev() {
        let rollback_op = RegistryRestoreOp {
            hive: op.hive,
            key: op.key.clone(),
            value_name: op.value_name.clone(),
            value_type: op.value_type.clone(),
            value: original_value.clone(),
            existed: *original_existed,
        };

        if let Err(e) = execute_registry_restore(&rollback_op, false) {
            log::error!(
                "Failed to rollback {}\\{}\\{}: {}",
                op.hive.as_str(),
                op.key,
                op.value_name,
                e
            );
        }
    }
}
