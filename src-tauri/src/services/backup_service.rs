//! Snapshot-Based Backup Service
//!
//! Simple, atomic backup system for registry tweaks.
//! Key features:
//! - Capture complete state before applying any tweak
//! - Atomic rollback on failure
//! - Simple snapshot-based restore

use crate::error::Error;
use crate::models::{RegistrySnapshot, ServiceSnapshot, TweakDefinition, TweakSnapshot};
use crate::services::{registry_service, trusted_installer};
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

/// Capture complete state of all registry keys affected by a tweak
pub fn capture_snapshot(
    tweak: &TweakDefinition,
    windows_version: u32,
) -> Result<TweakSnapshot, Error> {
    log::info!("Capturing snapshot for tweak '{}'", tweak.name);

    let mut snapshot = TweakSnapshot::new(
        &tweak.id,
        &tweak.name,
        windows_version,
        tweak.requires_system,
    );

    // Capture registry values
    for change in &tweak.registry_changes {
        let hive_str = change.hive.as_str();
        let value_type_str = change.value_type.as_str();

        let (value, existed) =
            read_registry_value(hive_str, &change.key, &change.value_name, value_type_str)?;

        let reg_snapshot = RegistrySnapshot {
            hive: hive_str.to_string(),
            key: change.key.clone(),
            value_name: change.value_name.clone(),
            value_type: value_type_str.to_string(),
            value,
            existed,
        };

        snapshot.registry_snapshots.push(reg_snapshot);
        log::trace!(
            "Captured: {}\\{}\\{} = {:?} (existed: {})",
            hive_str,
            change.key,
            change.value_name,
            snapshot.registry_snapshots.last().unwrap().value,
            existed
        );
    }

    // Capture service states
    if let Some(ref service_changes) = tweak.service_changes {
        for sc in service_changes {
            let service_snapshot = capture_service_state(&sc.name)?;
            snapshot.service_snapshots.push(service_snapshot);
        }
    }

    log::info!(
        "Captured {} registry values and {} services for '{}'",
        snapshot.registry_snapshots.len(),
        snapshot.service_snapshots.len(),
        tweak.name
    );

    Ok(snapshot)
}

/// Capture current service state
fn capture_service_state(service_name: &str) -> Result<ServiceSnapshot, Error> {
    use crate::services::service_control;

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

/// Read a registry value (returns value and whether it existed)
fn read_registry_value(
    hive: &str,
    key: &str,
    value_name: &str,
    value_type: &str,
) -> Result<(Option<serde_json::Value>, bool), Error> {
    let hive_enum = match hive {
        "HKCU" => crate::models::RegistryHive::HKCU,
        "HKLM" => crate::models::RegistryHive::HKLM,
        _ => return Err(Error::BackupFailed(format!("Unknown hive: {}", hive))),
    };

    let result = match value_type {
        "REG_DWORD" => registry_service::read_dword(&hive_enum, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        "REG_SZ" | "REG_EXPAND_SZ" => registry_service::read_string(&hive_enum, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        "REG_BINARY" => registry_service::read_binary(&hive_enum, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        "REG_QWORD" => registry_service::read_qword(&hive_enum, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        _ => Err(Error::BackupFailed(format!(
            "Unsupported value type: {}",
            value_type
        ))),
    };

    match result {
        Ok(Some(value)) => Ok((Some(value), true)),
        Ok(None) => Ok((None, false)),
        Err(Error::RegistryKeyNotFound(_)) => Ok((None, false)),
        Err(e) => {
            log::warn!("Failed to read {}\\{}\\{}: {}", hive, key, value_name, e);
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

/// Restore all registry values from snapshot (atomic - all or nothing)
pub fn restore_from_snapshot(snapshot: &TweakSnapshot) -> Result<(), Error> {
    log::info!(
        "Restoring from snapshot for tweak '{}'",
        snapshot.tweak_name
    );

    // First, collect all write operations we need to do
    let mut operations: Vec<RestoreOperation> = Vec::new();

    for reg in &snapshot.registry_snapshots {
        let hive_enum = match reg.hive.as_str() {
            "HKCU" => crate::models::RegistryHive::HKCU,
            "HKLM" => crate::models::RegistryHive::HKLM,
            _ => {
                return Err(Error::BackupFailed(format!("Unknown hive: {}", reg.hive)));
            }
        };

        operations.push(RestoreOperation {
            hive: hive_enum,
            key: reg.key.clone(),
            value_name: reg.value_name.clone(),
            value_type: reg.value_type.clone(),
            value: reg.value.clone(),
            existed: reg.existed,
        });
    }

    // Execute all operations, tracking what we've done for rollback
    let mut completed: Vec<(RestoreOperation, Option<serde_json::Value>, bool)> = Vec::new();

    for op in &operations {
        // Capture current value for rollback
        let (current_value, current_exists) =
            read_registry_value(op.hive.as_str(), &op.key, &op.value_name, &op.value_type)?;

        // Execute the restore (use SYSTEM elevation if configured)
        match execute_restore_operation(op, snapshot.requires_system) {
            Ok(()) => {
                completed.push((op.clone(), current_value, current_exists));
            }
            Err(e) => {
                log::error!(
                    "Restore failed for {}\\{}\\{}: {}",
                    op.hive.as_str(),
                    op.key,
                    op.value_name,
                    e
                );

                // Rollback everything we've done
                rollback_operations(&completed);

                return Err(Error::BackupFailed(format!(
                    "Failed to restore registry value, rolled back {} changes: {}",
                    completed.len(),
                    e
                )));
            }
        }
    }

    log::info!(
        "Successfully restored {} registry values from snapshot",
        completed.len()
    );

    Ok(())
}

#[derive(Clone)]
struct RestoreOperation {
    hive: crate::models::RegistryHive,
    key: String,
    value_name: String,
    value_type: String,
    value: Option<serde_json::Value>,
    existed: bool,
}

/// Execute a single restore operation
fn execute_restore_operation(op: &RestoreOperation, use_system: bool) -> Result<(), Error> {
    if !op.existed {
        // Value didn't exist - delete it
        log::debug!(
            "Deleting {}\\{}\\{} (didn't exist originally)",
            op.hive.as_str(),
            op.key,
            op.value_name
        );

        if use_system {
            // Use SYSTEM elevation for delete
            match trusted_installer::delete_registry_value_as_system(
                op.hive.as_str(),
                &op.key,
                &op.value_name,
            ) {
                Ok(()) => Ok(()),
                Err(_) => Ok(()), // Ignore delete failures (key may already be gone)
            }
        } else {
            match registry_service::delete_value(&op.hive, &op.key, &op.value_name) {
                Ok(()) => Ok(()),
                Err(Error::RegistryKeyNotFound(_)) => Ok(()), // Already gone
                Err(e) => Err(e),
            }
        }
    } else if let Some(value) = &op.value {
        // Restore the original value
        log::debug!(
            "Restoring {}\\{}\\{} = {:?} (use_system: {})",
            op.hive.as_str(),
            op.key,
            op.value_name,
            value,
            use_system
        );

        if use_system {
            // Use SYSTEM elevation via reg.exe
            let value_data = match op.value_type.as_str() {
                "REG_DWORD" | "REG_QWORD" => value.as_u64().map(|v| v.to_string()),
                "REG_SZ" | "REG_EXPAND_SZ" => value.as_str().map(|s| format!("\"{}\"", s)),
                _ => {
                    log::warn!("SYSTEM elevation not supported for {}", op.value_type);
                    return Ok(());
                }
            };

            if let Some(data) = value_data {
                log::info!(
                    "[SYSTEM] Restoring {}: {}\\{}\\{} = {}",
                    op.value_type,
                    op.hive.as_str(),
                    op.key,
                    op.value_name,
                    data
                );
                trusted_installer::set_registry_value_as_system(
                    op.hive.as_str(),
                    &op.key,
                    &op.value_name,
                    &op.value_type,
                    &data,
                )?;
            }
            Ok(())
        } else {
            // Normal registry writes
            match op.value_type.as_str() {
                "REG_DWORD" => {
                    if let Some(v) = value.as_u64() {
                        registry_service::set_dword(&op.hive, &op.key, &op.value_name, v as u32)?;
                    }
                }
                "REG_SZ" | "REG_EXPAND_SZ" => {
                    if let Some(v) = value.as_str() {
                        registry_service::set_string(&op.hive, &op.key, &op.value_name, v)?;
                    }
                }
                "REG_BINARY" => {
                    if let Some(arr) = value.as_array() {
                        let binary: Vec<u8> = arr
                            .iter()
                            .filter_map(|v| v.as_u64().map(|u| u as u8))
                            .collect();
                        registry_service::set_binary(&op.hive, &op.key, &op.value_name, &binary)?;
                    }
                }
                "REG_QWORD" => {
                    if let Some(v) = value.as_u64() {
                        registry_service::set_qword(&op.hive, &op.key, &op.value_name, v)?;
                    }
                }
                _ => {
                    return Err(Error::BackupFailed(format!(
                        "Unsupported value type: {}",
                        op.value_type
                    )));
                }
            }
            Ok(())
        }
    } else {
        // Existed but was null? Shouldn't happen, but delete to be safe
        log::warn!(
            "Value existed but was None: {}\\{}\\{}",
            op.hive.as_str(),
            op.key,
            op.value_name
        );
        let _ = registry_service::delete_value(&op.hive, &op.key, &op.value_name);
        Ok(())
    }
}

/// Rollback completed operations on failure
fn rollback_operations(completed: &[(RestoreOperation, Option<serde_json::Value>, bool)]) {
    log::warn!("Rolling back {} registry operations", completed.len());

    for (op, original_value, original_existed) in completed.iter().rev() {
        let rollback_op = RestoreOperation {
            hive: op.hive.clone(),
            key: op.key.clone(),
            value_name: op.value_name.clone(),
            value_type: op.value_type.clone(),
            value: original_value.clone(),
            existed: *original_existed,
        };

        // Rollback uses normal registry writes (if SYSTEM was needed, we probably can't rollback anyway)
        if let Err(e) = execute_restore_operation(&rollback_op, false) {
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

/// Detect current state of a tweak by reading registry values
/// Returns: (is_applied, current_option_index)
pub fn detect_tweak_state(tweak: &TweakDefinition) -> Result<(bool, Option<usize>), Error> {
    if tweak.registry_changes.is_empty() {
        return Ok((false, None));
    }

    let mut all_match_enabled = true;
    let mut detected_option: Option<usize> = None;

    // Read all current values and compare
    for change in &tweak.registry_changes {
        let (current_value, _existed) = read_registry_value(
            change.hive.as_str(),
            &change.key,
            &change.value_name,
            change.value_type.as_str(),
        )?;

        // Check if matches enable_value
        if !values_match(&current_value, &Some(change.enable_value.clone())) {
            all_match_enabled = false;
        }

        // Check multi-state options
        if let Some(ref options) = change.options {
            for (idx, opt) in options.iter().enumerate() {
                if values_match(&current_value, &Some(opt.value.clone())) {
                    detected_option = Some(idx);
                    break;
                }
            }
        }
    }

    Ok((all_match_enabled, detected_option))
}

/// Compare two JSON values for equality (handles numeric type variations)
fn values_match(a: &Option<serde_json::Value>, b: &Option<serde_json::Value>) -> bool {
    match (a, b) {
        (Some(va), Some(vb)) => {
            // Direct equality works for most cases
            if va == vb {
                return true;
            }
            // Fallback: numeric comparison (i64/u64 may differ in JSON)
            if let (Some(na), Some(nb)) = (va.as_i64(), vb.as_i64()) {
                return na == nb;
            }
            false
        }
        (None, None) => true,
        _ => false,
    }
}

// ============================================================================
// Migration
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

        // Remove all files in old backups directory
        if let Ok(entries) = fs::read_dir(&old_backups_dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }

        // Try to remove the directory itself
        let _ = fs::remove_dir(&old_backups_dir);

        log::info!("Old backup cleanup complete");
    }

    Ok(())
}

// ============================================================================
// Snapshot Validation
// ============================================================================

/// Validate a single snapshot against current registry state
/// Returns true if snapshot is still valid (current state differs from snapshot state)
/// Returns false if snapshot is stale (current state matches snapshot state)
pub fn validate_snapshot(snapshot: &TweakSnapshot) -> Result<bool, Error> {
    log::debug!("Validating snapshot for tweak '{}'", snapshot.tweak_name);

    // Check each registry value in the snapshot
    for reg in &snapshot.registry_snapshots {
        let (current_value, current_exists) =
            read_registry_value(&reg.hive, &reg.key, &reg.value_name, &reg.value_type)?;

        // If current state matches snapshot state, tweak was externally reverted
        let snapshot_matches_current = if !reg.existed && !current_exists {
            // Both don't exist - matches
            true
        } else if reg.existed && current_exists {
            // Both exist - compare values
            values_match(&reg.value, &current_value)
        } else {
            // One exists, one doesn't - doesn't match
            false
        };

        if !snapshot_matches_current {
            // Current state differs from snapshot - snapshot is valid
            log::trace!(
                "Snapshot valid: {}\\{}\\{} differs from snapshot",
                reg.hive,
                reg.key,
                reg.value_name
            );
            return Ok(true);
        }
    }

    // All values match snapshot - tweak was reverted externally, snapshot is stale
    log::info!(
        "Snapshot stale for '{}': current state matches snapshot state",
        snapshot.tweak_name
    );
    Ok(false)
}

/// Validate all snapshots on app startup
/// Removes stale snapshots where current registry state matches the snapshot state
pub fn validate_all_snapshots() -> Result<u32, Error> {
    log::info!("Validating all snapshots on startup");

    let applied_tweaks = get_applied_tweaks()?;
    let mut removed_count = 0;

    for tweak_id in applied_tweaks {
        if let Some(snapshot) = load_snapshot(&tweak_id)? {
            let is_valid = validate_snapshot(&snapshot)?;

            if !is_valid {
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
    } else {
        log::debug!("All snapshots are valid");
    }

    Ok(removed_count)
}
