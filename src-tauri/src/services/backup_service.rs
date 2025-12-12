//! Snapshot-Based Backup Service
//!
//! Unified option-based backup system for registry tweaks.
//! Key features:
//! - Capture complete state before applying any tweak option
//! - Atomic rollback on failure
//! - State detection by matching current state against all options

use crate::error::Error;
use crate::models::{
    RegistryHive, RegistrySnapshot, RegistryValueType, SchedulerSnapshot, ServiceSnapshot,
    TweakDefinition, TweakOption, TweakSnapshot, TweakState,
};
use crate::services::{registry_service, scheduler_service, service_control, trusted_installer};
use std::fs;
use std::path::PathBuf;

const SNAPSHOTS_DIR: &str = "snapshots";

// ============================================================================
// File Path Management
// ============================================================================

/// Get the snapshots directory path (next to executable for portability)
pub fn get_snapshots_dir() -> Result<PathBuf, Error> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| Error::BackupFailed(format!("Failed to get executable path: {}", e)))?
        .parent()
        .ok_or_else(|| Error::BackupFailed("Failed to get executable directory".into()))?
        .to_path_buf();

    let snapshots_dir = exe_dir.join(SNAPSHOTS_DIR);

    // Create directory if it doesn't exist
    if !snapshots_dir.exists() {
        fs::create_dir_all(&snapshots_dir).map_err(|e| {
            Error::BackupFailed(format!("Failed to create snapshots directory: {}", e))
        })?;
        log::debug!("Created snapshots directory at {:?}", snapshots_dir);
    }

    Ok(snapshots_dir)
}

fn get_snapshot_path(tweak_id: &str) -> Result<PathBuf, Error> {
    Ok(get_snapshots_dir()?.join(format!("{}.json", tweak_id)))
}

// ============================================================================
// Snapshot Operations
// ============================================================================

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
        let task_snapshot =
            capture_scheduler_state(&task_change.task_path, &task_change.task_name)?;
        snapshot.add_scheduler_snapshot(task_snapshot);
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

/// Capture current service state
fn capture_service_state(service_name: &str) -> Result<ServiceSnapshot, Error> {
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
fn capture_scheduler_state(task_path: &str, task_name: &str) -> Result<SchedulerSnapshot, Error> {
    let state = scheduler_service::get_task_state(task_path, task_name)?;

    Ok(SchedulerSnapshot {
        task_path: task_path.to_string(),
        task_name: task_name.to_string(),
        original_state: state.as_str().to_string(),
    })
}

/// Read a registry value (returns value and whether it existed)
fn read_registry_value(
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

/// Save snapshot to disk
pub fn save_snapshot(snapshot: &TweakSnapshot) -> Result<(), Error> {
    let path = get_snapshot_path(&snapshot.tweak_id)?;

    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize snapshot: {}", e)))?;

    fs::write(&path, json)
        .map_err(|e| Error::BackupFailed(format!("Failed to write snapshot: {}", e)))?;

    log::debug!("Saved snapshot to {:?}", path);
    Ok(())
}

/// Load snapshot for a tweak
pub fn load_snapshot(tweak_id: &str) -> Result<Option<TweakSnapshot>, Error> {
    let path = get_snapshot_path(tweak_id)?;

    if !path.exists() {
        log::debug!("No snapshot found for tweak '{}'", tweak_id);
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to read snapshot: {}", e)))?;

    let snapshot: TweakSnapshot = serde_json::from_str(&content)
        .map_err(|e| Error::BackupFailed(format!("Failed to parse snapshot: {}", e)))?;

    log::debug!("Loaded snapshot for tweak '{}'", tweak_id);
    Ok(Some(snapshot))
}

/// Check if a snapshot exists for a tweak
pub fn snapshot_exists(tweak_id: &str) -> Result<bool, Error> {
    let path = get_snapshot_path(tweak_id)?;
    Ok(path.exists())
}

/// Delete snapshot after successful revert
pub fn delete_snapshot(tweak_id: &str) -> Result<(), Error> {
    let path = get_snapshot_path(tweak_id)?;

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| Error::BackupFailed(format!("Failed to delete snapshot: {}", e)))?;
        log::debug!("Deleted snapshot for tweak '{}'", tweak_id);
    }

    Ok(())
}

/// Get list of all applied tweak IDs (by listing snapshot files)
pub fn get_applied_tweaks() -> Result<Vec<String>, Error> {
    let dir = get_snapshots_dir()?;
    let mut tweaks = Vec::new();

    if dir.exists() {
        for entry in fs::read_dir(&dir).map_err(|e| Error::BackupFailed(e.to_string()))? {
            let entry = entry.map_err(|e| Error::BackupFailed(e.to_string()))?;
            let filename = entry.file_name().to_string_lossy().to_string();

            if filename.ends_with(".json") {
                let tweak_id = filename.trim_end_matches(".json").to_string();
                tweaks.push(tweak_id);
            }
        }
    }

    Ok(tweaks)
}

// ============================================================================
// Restore Operations
// ============================================================================

/// Restore all registry/service values from snapshot (atomic - all or nothing)
pub fn restore_from_snapshot(snapshot: &TweakSnapshot) -> Result<(), Error> {
    log::info!(
        "Restoring from snapshot for tweak '{}' (was option '{}')",
        snapshot.tweak_name,
        snapshot.applied_option_label
    );

    // Restore registry values
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
                    "Failed to restore, rolled back {} changes: {}",
                    completed_registry.len(),
                    e
                )));
            }
        }
    }

    // Restore service states
    for svc in &snapshot.service_snapshots {
        if let Err(e) = restore_service_state(svc) {
            log::warn!("Failed to restore service '{}': {}", svc.name, e);
            // Continue with other services - service restore failures are non-fatal
        }
    }

    // Restore scheduled task states
    for task in &snapshot.scheduler_snapshots {
        if let Err(e) = restore_scheduler_state(task) {
            log::warn!(
                "Failed to restore task '{}\\{}': {}",
                task.task_path,
                task.task_name,
                e
            );
            // Continue with other tasks - task restore failures are non-fatal
        }
    }

    log::info!(
        "Successfully restored {} registry values, {} services, and {} tasks",
        completed_registry.len(),
        snapshot.service_snapshots.len(),
        snapshot.scheduler_snapshots.len()
    );

    Ok(())
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

fn restore_service_state(snapshot: &ServiceSnapshot) -> Result<(), Error> {
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

    service_control::set_service_startup(&snapshot.name, &startup)?;

    if snapshot.was_running {
        let _ = service_control::start_service(&snapshot.name);
    } else {
        let _ = service_control::stop_service(&snapshot.name);
    }

    Ok(())
}

fn restore_scheduler_state(snapshot: &SchedulerSnapshot) -> Result<(), Error> {
    let task_path = format!("{}\\{}", snapshot.task_path, snapshot.task_name);
    log::debug!(
        "Restoring scheduled task '{}' to state: {}",
        task_path,
        snapshot.original_state
    );

    match snapshot.original_state.as_str() {
        "Ready" | "Running" => {
            // Task was enabled, re-enable it
            scheduler_service::enable_task(&snapshot.task_path, &snapshot.task_name)?;
        }
        "Disabled" => {
            // Task was disabled, ensure it's disabled
            scheduler_service::disable_task(&snapshot.task_path, &snapshot.task_name)?;
        }
        "NotFound" => {
            // Task didn't exist before - we can't restore a deleted task
            // This is expected if the tweak was a "delete" action
            log::info!(
                "Task '{}' was not found before tweak, cannot restore",
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

/// Check if all registry/service changes in an option match current state
/// Items with skip_validation=true are excluded from this check
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

    // If option has no validatable changes, it can't match
    if validatable_registry.is_empty() && validatable_services.is_empty() {
        return Ok(false);
    }

    // Check all validatable registry values
    for change in validatable_registry {
        let (current_value, existed) = read_registry_value(
            &change.hive,
            &change.key,
            &change.value_name,
            &change.value_type,
        )?;

        if !existed {
            return Ok(false);
        }

        if !values_match(&current_value, &Some(change.value.clone())) {
            return Ok(false);
        }
    }

    // Check all validatable service states
    for change in validatable_services {
        let status = service_control::get_service_status(&change.name)?;
        let current_startup = status.startup_type;

        if current_startup != Some(change.startup) {
            return Ok(false);
        }
    }

    // All validatable checks passed
    Ok(true)
}

/// Compare two JSON values for equality (handles numeric type variations)
fn values_match(a: &Option<serde_json::Value>, b: &Option<serde_json::Value>) -> bool {
    match (a, b) {
        (Some(va), Some(vb)) => {
            if va == vb {
                return true;
            }
            // Numeric comparison fallback
            if let (Some(na), Some(nb)) = (va.as_i64(), vb.as_i64()) {
                return na == nb;
            }
            if let (Some(na), Some(nb)) = (va.as_u64(), vb.as_u64()) {
                return na == nb;
            }
            false
        }
        (None, None) => true,
        _ => false,
    }
}

// ============================================================================
// Helper Parsers
// ============================================================================

fn parse_hive(hive: &str) -> Result<RegistryHive, Error> {
    match hive {
        "HKCU" => Ok(RegistryHive::Hkcu),
        "HKLM" => Ok(RegistryHive::Hklm),
        _ => Err(Error::BackupFailed(format!("Unknown hive: {}", hive))),
    }
}

fn parse_value_type(value_type: &str) -> Result<RegistryValueType, Error> {
    match value_type {
        "REG_DWORD" => Ok(RegistryValueType::Dword),
        "REG_QWORD" => Ok(RegistryValueType::Qword),
        "REG_SZ" => Ok(RegistryValueType::String),
        "REG_EXPAND_SZ" => Ok(RegistryValueType::ExpandString),
        "REG_MULTI_SZ" => Ok(RegistryValueType::MultiString),
        "REG_BINARY" => Ok(RegistryValueType::Binary),
        _ => Err(Error::BackupFailed(format!(
            "Unknown value type: {}",
            value_type
        ))),
    }
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

/// Validate all snapshots on app startup
/// Removes stale snapshots where tweak was externally reverted
pub fn validate_all_snapshots() -> Result<u32, Error> {
    log::info!("Validating all snapshots on startup");

    let applied_tweaks = get_applied_tweaks()?;
    let mut removed_count = 0;

    for tweak_id in applied_tweaks {
        if let Some(snapshot) = load_snapshot(&tweak_id)? {
            // A snapshot is stale if current state matches the original snapshot state
            // (meaning the tweak was externally reverted)
            let is_stale = snapshot_matches_current_state(&snapshot)?;

            if is_stale {
                log::info!(
                    "Removing stale snapshot for '{}' - tweak was externally reverted",
                    snapshot.tweak_name
                );
                delete_snapshot(&tweak_id)?;
                removed_count += 1;
            }
        }
    }

    if removed_count > 0 {
        log::info!("Removed {} stale snapshots", removed_count);
    }

    Ok(removed_count)
}

/// Check if current registry state matches the snapshot state
/// (indicating tweak was externally reverted)
fn snapshot_matches_current_state(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    for reg in &snapshot.registry_snapshots {
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

        if !matches {
            return Ok(false);
        }
    }

    // All values match snapshot - tweak was reverted
    Ok(true)
}
