//! Restore Operations
//!
//! Functions for restoring system state from snapshots:
//! - Atomic restore with rollback on failure
//! - Registry value restoration (normal and elevated)
//! - Service and scheduler state restoration (with SYSTEM elevation support)

use crate::error::Error;
use crate::models::{
    FirewallSnapshot, HostsSnapshot, RegistryHive, RegistrySnapshot, SchedulerAction,
    SchedulerSnapshot, ServiceSnapshot, TweakSnapshot,
};
use crate::services::{
    firewall_service, hosts_service, registry_service, registry_value, service_control,
    trusted_installer,
};

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

    // ADR-0001: rollback never aborts early. Every phase is attempted and its failures collected,
    // so one failed phase can't abandon the rest — and nothing is rolled back on a partial restore
    // (that would un-restore already-restored values). The caller keeps the snapshot whenever any
    // failure remains, so the user can retry (Needs Attention).
    let mut failures: Vec<String> = Vec::new();

    // Phase 1: Restore registry values
    for reg in &snapshot.registry_snapshots {
        if let Err(e) = restore_one_registry(reg, snapshot.requires_system) {
            let msg = format!(
                "Registry '{}\\{}\\{}': {}",
                reg.hive, reg.key, reg.value_name, e
            );
            log::error!("Failed to restore registry: {}", msg);
            failures.push(msg);
        }
    }

    // Phase 2: Restore service states
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

    // Phase 4: Restore hosts file entries (collect failures)
    for host in &snapshot.hosts_snapshots {
        if let Err(e) = restore_hosts_state(host) {
            let msg = format!("Hosts '{}->{}': {}", host.ip, host.domain, e);
            log::error!("Failed to restore hosts entry: {}", msg);
            failures.push(msg);
        }
    }

    // Phase 5: Restore firewall rules (collect failures)
    for fw in &snapshot.firewall_snapshots {
        if let Err(e) = restore_firewall_state(fw) {
            let msg = format!("Firewall '{}': {}", fw.name, e);
            log::error!("Failed to restore firewall rule: {}", msg);
            failures.push(msg);
        }
    }

    let success = failures.is_empty();

    if success {
        log::info!(
            "Successfully restored {} registry, {} services, {} tasks, {} hosts, {} firewall",
            snapshot.registry_snapshots.len(),
            snapshot.service_snapshots.len(),
            snapshot.scheduler_snapshots.len(),
            snapshot.hosts_snapshots.len(),
            snapshot.firewall_snapshots.len()
        );
    } else {
        log::warn!(
            "Restore completed with {} failures out of {} registry, {} services, {} tasks, {} hosts, {} firewall",
            failures.len(),
            snapshot.registry_snapshots.len(),
            snapshot.service_snapshots.len(),
            snapshot.scheduler_snapshots.len(),
            snapshot.hosts_snapshots.len(),
            snapshot.firewall_snapshots.len()
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

/// Restore a single registry value from its snapshot.
fn restore_one_registry(reg: &RegistrySnapshot, use_system: bool) -> Result<(), Error> {
    let hive = parse_hive(&reg.hive)?;
    let op = RegistryRestoreOp {
        hive,
        key: reg.key.clone(),
        value_name: reg.value_name.clone(),
        value_type: reg.value_type.clone(),
        value: reg.value.clone(),
        existed: reg.existed,
    };
    execute_registry_restore(&op, use_system)
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
            trusted_installer::delete_registry_value_as_system(op.hive, &op.key, &op.value_name)?;
        } else {
            match registry_service::delete_value(&op.hive, &op.key, &op.value_name) {
                Ok(()) => {}
                Err(Error::RegistryKeyNotFound(_)) => {
                    // Already absent (key/value missing) - treat as restored
                }
                Err(e) => return Err(e),
            }
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
    let value_type = parse_value_type(value_type)?;
    registry_value::write_registry_json_value(hive, key, value_name, &value_type, value, false)
}

fn restore_registry_with_system(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &str,
    value: &serde_json::Value,
) -> Result<(), Error> {
    let value_type = parse_value_type(value_type)?;
    registry_value::write_registry_json_value(hive, key, value_name, &value_type, value, true)
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
        trusted_installer::set_service_startup_as_system(&snapshot.name, &startup)?;
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

    // One typed op for both elevations (no schtasks string): SYSTEM runs it in the broker,
    // otherwise in-process.
    let level = if use_system {
        trusted_installer::Elevation::System
    } else {
        trusted_installer::Elevation::None
    };

    match snapshot.original_state.as_str() {
        "Ready" | "Running" => {
            // Task was enabled — re-enable it.
            trusted_installer::run_scheduler_op(
                level,
                &snapshot.task_path,
                &snapshot.task_name,
                SchedulerAction::Enable,
            )?;
            log::info!("Enabled scheduled task: {}", task_path);
        }
        "Disabled" => {
            // Task was disabled — ensure it stays disabled.
            trusted_installer::run_scheduler_op(
                level,
                &snapshot.task_path,
                &snapshot.task_name,
                SchedulerAction::Disable,
            )?;
            log::info!("Disabled scheduled task: {}", task_path);
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

fn restore_hosts_state(snapshot: &HostsSnapshot) -> Result<(), Error> {
    if snapshot.existed {
        // Entry existed before - ensure it exists now
        let currently_exists = hosts_service::entry_exists(&snapshot.ip, &snapshot.domain)?;
        if !currently_exists {
            hosts_service::add_hosts_entry(&snapshot.ip, &snapshot.domain, None)?;
            log::info!(
                "Restored hosts entry: {} -> {}",
                snapshot.domain,
                snapshot.ip
            );
        }
    } else {
        // Entry didn't exist before - remove it if present
        let currently_exists = hosts_service::entry_exists(&snapshot.ip, &snapshot.domain)?;
        if currently_exists {
            hosts_service::remove_hosts_entry(&snapshot.ip, &snapshot.domain)?;
            log::info!(
                "Removed hosts entry: {} -> {} (didn't exist originally)",
                snapshot.domain,
                snapshot.ip
            );
        }
    }
    Ok(())
}

fn restore_firewall_state(snapshot: &FirewallSnapshot) -> Result<(), Error> {
    if snapshot.existed {
        // Rule existed before - we can't fully recreate it without storing the full rule config
        // Just log a warning if it's missing now
        let currently_exists = firewall_service::rule_exists(&snapshot.name)?;
        if !currently_exists {
            log::warn!(
                "Firewall rule '{}' existed before but is now missing; cannot recreate without original rule config",
                snapshot.name
            );
        }
    } else {
        // Rule didn't exist before - delete it if present
        let currently_exists = firewall_service::rule_exists(&snapshot.name)?;
        if currently_exists {
            firewall_service::delete_firewall_rule(&snapshot.name)?;
            log::info!(
                "Deleted firewall rule '{}' (didn't exist originally)",
                snapshot.name
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_phase_does_not_abort_the_remaining_phases() {
        // ADR-0001: rollback attempts all five phases and collects failures. A registry restore that
        // fails must not abandon the service phase — previously the registry phase rolled back and
        // returned Err, abandoning services / scheduler / hosts / firewall entirely.
        let mut snap = TweakSnapshot::new("__wp4_test", "T", 0, "opt", 11, false, None);

        // A registry op that fails immediately (unknown hive), before touching the registry.
        snap.registry_snapshots.push(RegistrySnapshot {
            hive: "BOGUS_HIVE".to_string(),
            key: "Software\\X".to_string(),
            value_name: "V".to_string(),
            value_type: Some("REG_DWORD".to_string()),
            value: Some(serde_json::json!(1)),
            existed: true,
        });
        // A service op for a service that does not exist — this later phase must still be attempted.
        snap.service_snapshots.push(ServiceSnapshot {
            name: "MagicXNoSuchService_wp4".to_string(),
            startup_type: "manual".to_string(),
            was_running: false,
        });

        let result = restore_from_snapshot(&snap).unwrap();
        assert!(!result.success);
        assert_eq!(
            result.failures.len(),
            2,
            "both phases must be attempted and both fail: {:?}",
            result.failures
        );
        assert!(result.failures.iter().any(|f| f.starts_with("Registry")));
        assert!(result.failures.iter().any(|f| f.starts_with("Service")));
    }
}
