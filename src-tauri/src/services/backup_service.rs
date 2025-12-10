//! Bulletproof Backup Service
//!
//! This service manages registry backups with a focus on safety and reliability.
//! Key features:
//! - Global baseline: Original values are captured once and never overwritten
//! - Reference counting: Tracks how many tweaks modify each registry key
//! - Atomic operations: Changes are rolled back on failure
//! - Conflict detection: Warns when multiple tweaks affect the same key

use crate::error::Error;
use crate::models::{
    make_key_id, AppliedTweakInfo, BaselineEntry, ConflictReport, KeyConflict, KeyStatus,
    RecoveryAction, RecoverySuggestion, RegistryBaseline, RegistryKeyId, TweakState,
};
use crate::services::registry_service;
use std::fs;
use std::path::PathBuf;

// File names for our state files
const BASELINE_FILE: &str = "registry_baseline.json";
const TWEAK_STATE_FILE: &str = "tweak_state.json";
const BACKUP_DIR: &str = "backups";

// ============================================================================
// File Path Management
// ============================================================================

/// Get the backups directory path (next to executable for portability)
pub fn get_backups_dir() -> Result<PathBuf, Error> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| Error::BackupFailed(format!("Failed to get executable path: {}", e)))?
        .parent()
        .ok_or_else(|| Error::BackupFailed("Failed to get executable directory".into()))?
        .to_path_buf();

    let backups_dir = exe_dir.join(BACKUP_DIR);

    // Create directory if it doesn't exist
    if !backups_dir.exists() {
        fs::create_dir_all(&backups_dir).map_err(|e| {
            Error::BackupFailed(format!("Failed to create backups directory: {}", e))
        })?;
        log::debug!("Created backups directory at {:?}", backups_dir);
    }

    Ok(backups_dir)
}

fn get_baseline_path() -> Result<PathBuf, Error> {
    Ok(get_backups_dir()?.join(BASELINE_FILE))
}

fn get_tweak_state_path() -> Result<PathBuf, Error> {
    Ok(get_backups_dir()?.join(TWEAK_STATE_FILE))
}

// ============================================================================
// Baseline Management
// ============================================================================

/// Load the registry baseline from disk
pub fn load_baseline() -> Result<RegistryBaseline, Error> {
    let path = get_baseline_path()?;

    if !path.exists() {
        log::debug!("No baseline file found, creating new baseline");
        return Ok(RegistryBaseline::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to read baseline: {}", e)))?;

    let baseline: RegistryBaseline = serde_json::from_str(&content)
        .map_err(|e| Error::BackupFailed(format!("Failed to parse baseline: {}", e)))?;

    log::trace!("Loaded baseline with {} entries", baseline.entries.len());

    Ok(baseline)
}

/// Save the registry baseline to disk
pub fn save_baseline(baseline: &RegistryBaseline) -> Result<(), Error> {
    let path = get_baseline_path()?;

    let json = serde_json::to_string_pretty(baseline)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize baseline: {}", e)))?;

    fs::write(&path, json)
        .map_err(|e| Error::BackupFailed(format!("Failed to write baseline: {}", e)))?;

    log::trace!("Saved baseline with {} entries", baseline.entries.len());
    Ok(())
}

/// Capture baseline values for registry keys (only captures if not already in baseline)
/// Returns the list of key IDs that were newly captured
pub fn capture_baseline_for_keys(
    tweak_id: &str,
    keys: &[(String, String, String, String)], // (hive, key, value_name, value_type)
) -> Result<Vec<RegistryKeyId>, Error> {
    log::debug!(
        "Capturing baseline for {} keys (tweak: {})",
        keys.len(),
        tweak_id
    );

    let mut baseline = load_baseline()?;
    let mut newly_captured = Vec::new();
    let now = chrono::Local::now().to_rfc3339();

    for (hive, key, value_name, value_type) in keys {
        let key_id = make_key_id(hive, key, value_name);

        // Skip if already in baseline
        if baseline.has_entry(&key_id) {
            log::trace!("Baseline already exists for {}", key_id);
            continue;
        }

        // Read current value from registry
        let (original_value, key_existed) = read_current_value(hive, key, value_name, value_type)?;

        let entry = BaselineEntry {
            hive: hive.clone(),
            key: key.clone(),
            value_name: value_name.clone(),
            value_type: value_type.clone(),
            original_value,
            key_existed,
            captured_at: now.clone(),
            captured_by_tweak: tweak_id.to_string(),
        };

        baseline.add_entry(key_id.clone(), entry);
        newly_captured.push(key_id.clone());
        log::trace!("Captured baseline for {}", key_id);
    }

    if !newly_captured.is_empty() {
        save_baseline(&baseline)?;
        log::info!(
            "Captured {} new baseline entries for tweak '{}'",
            newly_captured.len(),
            tweak_id
        );
    }

    Ok(newly_captured)
}

/// Read the current value of a registry key
fn read_current_value(
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

    // Try to read the value
    let result =
        match value_type {
            "REG_DWORD" => registry_service::read_dword(&hive_enum, key, value_name)
                .map(|v| serde_json::json!(v)),
            "REG_SZ" | "REG_EXPAND_SZ" => {
                registry_service::read_string(&hive_enum, key, value_name)
                    .map(|v| serde_json::json!(v))
            }
            "REG_BINARY" => registry_service::read_binary(&hive_enum, key, value_name)
                .map(|v| serde_json::json!(v)),
            "REG_QWORD" => registry_service::read_qword(&hive_enum, key, value_name)
                .map(|v| serde_json::json!(v)),
            _ => Err(Error::BackupFailed(format!(
                "Unsupported value type: {}",
                value_type
            ))),
        };

    match result {
        Ok(value) => Ok((Some(value), true)),
        Err(Error::RegistryKeyNotFound(_)) => Ok((None, false)),
        Err(e) => {
            // Log the error but don't fail - treat as non-existent
            log::warn!("Failed to read {}\\{}\\{}: {}", hive, key, value_name, e);
            Ok((None, false))
        }
    }
}

// ============================================================================
// Tweak State Management
// ============================================================================

/// Load the tweak state from disk
pub fn load_tweak_state() -> Result<TweakState, Error> {
    let path = get_tweak_state_path()?;

    if !path.exists() {
        log::debug!("No tweak state file found, creating new state");
        return Ok(TweakState::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to read tweak state: {}", e)))?;

    let state: TweakState = serde_json::from_str(&content)
        .map_err(|e| Error::BackupFailed(format!("Failed to parse tweak state: {}", e)))?;

    log::trace!(
        "Loaded tweak state with {} applied tweaks",
        state.applied_tweaks.len()
    );

    Ok(state)
}

/// Save the tweak state to disk
pub fn save_tweak_state(state: &TweakState) -> Result<(), Error> {
    let path = get_tweak_state_path()?;

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize tweak state: {}", e)))?;

    fs::write(&path, json)
        .map_err(|e| Error::BackupFailed(format!("Failed to write tweak state: {}", e)))?;

    log::trace!(
        "Saved tweak state with {} applied tweaks",
        state.applied_tweaks.len()
    );
    Ok(())
}

/// Check if a tweak is currently applied according to our state
pub fn is_tweak_applied(tweak_id: &str) -> Result<bool, Error> {
    let state = load_tweak_state()?;
    Ok(state.is_tweak_applied(tweak_id))
}

/// Get list of all applied tweak IDs
pub fn get_applied_tweaks() -> Result<Vec<String>, Error> {
    let state = load_tweak_state()?;
    Ok(state.get_applied_tweak_ids())
}

// ============================================================================
// Apply/Revert Operations with Atomic Rollback
// ============================================================================

/// Result of a pre-flight check before applying a tweak
#[derive(Debug)]
#[allow(dead_code)]
pub struct PreflightResult {
    /// Keys that need baseline capture
    pub needs_baseline: Vec<RegistryKeyId>,
    /// Conflicts with other applied tweaks
    pub conflicts: Vec<KeyConflict>,
    /// Whether it's safe to proceed
    pub can_proceed: bool,
    /// Warning message if any
    pub warning: Option<String>,
}

/// Perform pre-flight checks before applying a tweak
pub fn preflight_check(
    tweak_id: &str,
    keys: &[(String, String, String, String)], // (hive, key, value_name, value_type)
) -> Result<PreflightResult, Error> {
    log::debug!("Running preflight check for tweak '{}'", tweak_id);

    let baseline = load_baseline()?;
    let state = load_tweak_state()?;

    let mut needs_baseline = Vec::new();
    let mut conflicts = Vec::new();

    for (hive, key, value_name, _value_type) in keys {
        let key_id = make_key_id(hive, key, value_name);

        // Check if we need to capture baseline
        if !baseline.has_entry(&key_id) {
            needs_baseline.push(key_id.clone());
        }

        // Check for conflicts with other applied tweaks
        let other_tweaks = state.get_tweaks_for_key(&key_id);
        if !other_tweaks.is_empty() && !other_tweaks.contains(&tweak_id.to_string()) {
            let current_value = read_current_value(hive, key, value_name, "REG_DWORD")
                .ok()
                .and_then(|(v, _)| v);
            let baseline_value = baseline
                .get_entry(&key_id)
                .and_then(|e| e.original_value.clone());

            conflicts.push(KeyConflict {
                key_id: key_id.clone(),
                key_path: format!("{}\\{}\\{}", hive, key, value_name),
                conflicting_tweaks: other_tweaks,
                current_value,
                baseline_value,
            });
        }
    }

    let has_conflicts = !conflicts.is_empty();
    let warning = if has_conflicts {
        Some(format!(
            "This tweak shares {} registry key(s) with other applied tweaks. Reverting order matters.",
            conflicts.len()
        ))
    } else {
        None
    };

    log::debug!(
        "Preflight result: {} need baseline, {} conflicts, can_proceed={}",
        needs_baseline.len(),
        conflicts.len(),
        true // We allow proceeding even with conflicts, but warn the user
    );

    Ok(PreflightResult {
        needs_baseline,
        conflicts,
        can_proceed: true, // We always allow proceed but warn
        warning,
    })
}

/// Record that a tweak has been successfully applied
/// This should be called AFTER all registry changes succeed
pub fn record_tweak_applied(
    tweak_id: &str,
    tweak_name: &str,
    windows_version: u32,
    modified_keys: Vec<RegistryKeyId>,
) -> Result<(), Error> {
    log::debug!("Recording tweak '{}' as applied", tweak_id);

    let mut state = load_tweak_state()?;

    let info = AppliedTweakInfo {
        tweak_id: tweak_id.to_string(),
        tweak_name: tweak_name.to_string(),
        applied_at: chrono::Local::now().to_rfc3339(),
        windows_version,
        modified_keys,
    };

    state.record_applied(info);
    save_tweak_state(&state)?;

    log::info!("Recorded tweak '{}' as applied", tweak_id);
    Ok(())
}

/// Record that a tweak has been reverted
/// Returns list of key_ids that should have their values restored to baseline
pub fn record_tweak_reverted(tweak_id: &str) -> Result<Vec<RegistryKeyId>, Error> {
    log::debug!("Recording tweak '{}' as reverted", tweak_id);

    let mut state = load_tweak_state()?;
    let orphaned_keys = state.record_reverted(tweak_id);
    save_tweak_state(&state)?;

    log::info!(
        "Recorded tweak '{}' as reverted, {} keys orphaned",
        tweak_id,
        orphaned_keys.len()
    );

    Ok(orphaned_keys)
}

/// Get keys that would be orphaned if a tweak is reverted (read-only, doesn't modify state)
/// This is used to preview the revert operation before actually performing it
pub fn get_orphaned_keys_for_tweak(tweak_id: &str) -> Result<Vec<RegistryKeyId>, Error> {
    let state = load_tweak_state()?;
    Ok(state.get_orphaned_keys_if_reverted(tweak_id))
}

/// Check if a key can be restored to baseline (ref_count == 0 after revert)
#[allow(dead_code)]
pub fn should_restore_to_baseline(key_id: &RegistryKeyId, tweak_id: &str) -> Result<bool, Error> {
    let state = load_tweak_state()?;

    // Get current ref count
    let current_count = state.get_key_ref_count(key_id);

    // If only this tweak references it, we can restore
    if current_count == 1 {
        let tweaks = state.get_tweaks_for_key(key_id);
        return Ok(tweaks.len() == 1 && tweaks[0] == tweak_id);
    }

    // If ref_count > 1, other tweaks are using this key
    Ok(false)
}

/// Get the baseline value for a key
pub fn get_baseline_value(key_id: &RegistryKeyId) -> Result<Option<BaselineEntry>, Error> {
    let baseline = load_baseline()?;
    Ok(baseline.get_entry(key_id).cloned())
}

/// Restore a registry key to its baseline value
pub fn restore_key_to_baseline(key_id: &RegistryKeyId) -> Result<bool, Error> {
    let baseline = load_baseline()?;

    let entry = match baseline.get_entry(key_id) {
        Some(e) => e,
        None => {
            log::warn!("No baseline found for key {}", key_id);
            return Ok(false);
        }
    };

    let hive_enum = match entry.hive.as_str() {
        "HKCU" => crate::models::RegistryHive::HKCU,
        "HKLM" => crate::models::RegistryHive::HKLM,
        _ => return Err(Error::BackupFailed(format!("Unknown hive: {}", entry.hive))),
    };

    match &entry.original_value {
        Some(value) => {
            log::debug!("Restoring {} to baseline value", key_id);

            match entry.value_type.as_str() {
                "REG_DWORD" => {
                    if let Some(v) = value.as_u64() {
                        registry_service::set_dword(
                            &hive_enum,
                            &entry.key,
                            &entry.value_name,
                            v as u32,
                        )?;
                    }
                }
                "REG_SZ" | "REG_EXPAND_SZ" => {
                    if let Some(v) = value.as_str() {
                        registry_service::set_string(&hive_enum, &entry.key, &entry.value_name, v)?;
                    }
                }
                "REG_BINARY" => {
                    if let Some(arr) = value.as_array() {
                        let binary: Vec<u8> = arr
                            .iter()
                            .filter_map(|v| v.as_u64().map(|u| u as u8))
                            .collect();
                        registry_service::set_binary(
                            &hive_enum,
                            &entry.key,
                            &entry.value_name,
                            &binary,
                        )?;
                    }
                }
                "REG_QWORD" => {
                    if let Some(v) = value.as_u64() {
                        registry_service::set_qword(&hive_enum, &entry.key, &entry.value_name, v)?;
                    }
                }
                _ => {
                    log::warn!("Unsupported value type for restore: {}", entry.value_type);
                    return Ok(false);
                }
            }

            log::info!("Restored {} to baseline value", key_id);
            Ok(true)
        }
        None => {
            // Value didn't exist originally - for safety, we don't delete it
            log::debug!(
                "Key {} had no original value, skipping delete for safety",
                key_id
            );
            Ok(false)
        }
    }
}

// ============================================================================
// Conflict Detection
// ============================================================================

/// Detect conflicts between a tweak and other applied tweaks
pub fn detect_conflicts(
    tweak_id: &str,
    keys: &[(String, String, String)], // (hive, key, value_name)
) -> Result<ConflictReport, Error> {
    let state = load_tweak_state()?;
    let baseline = load_baseline()?;

    let mut conflicts = Vec::new();

    for (hive, key, value_name) in keys {
        let key_id = make_key_id(hive, key, value_name);
        let other_tweaks: Vec<_> = state
            .get_tweaks_for_key(&key_id)
            .into_iter()
            .filter(|id| id != tweak_id)
            .collect();

        if !other_tweaks.is_empty() {
            let current_value = read_current_value(hive, key, value_name, "REG_DWORD")
                .ok()
                .and_then(|(v, _)| v);
            let baseline_value = baseline
                .get_entry(&key_id)
                .and_then(|e| e.original_value.clone());

            conflicts.push(KeyConflict {
                key_id: key_id.clone(),
                key_path: format!("{}\\{}\\{}", hive, key, value_name),
                conflicting_tweaks: other_tweaks,
                current_value,
                baseline_value,
            });
        }
    }

    let has_conflicts = !conflicts.is_empty();
    let warning = if has_conflicts {
        Some("Multiple tweaks modify the same registry keys. Be careful with revert order.".into())
    } else {
        None
    };

    Ok(ConflictReport {
        has_conflicts,
        conflicts,
        warning,
    })
}

/// Get the status of a specific registry key
#[allow(dead_code)]
pub fn get_key_status(key_id: &RegistryKeyId) -> Result<KeyStatus, Error> {
    let baseline = load_baseline()?;
    let state = load_tweak_state()?;

    // Check if key is in baseline
    let baseline_entry = match baseline.get_entry(key_id) {
        Some(entry) => entry,
        None => return Ok(KeyStatus::Unknown),
    };

    // Check if any tweaks modify this key
    let tweaks = state.get_tweaks_for_key(key_id);

    if tweaks.is_empty() {
        // No tweaks modify this key, check if it matches baseline
        let current = read_current_value(
            &baseline_entry.hive,
            &baseline_entry.key,
            &baseline_entry.value_name,
            &baseline_entry.value_type,
        )?;

        if current.0 == baseline_entry.original_value {
            return Ok(KeyStatus::AtBaseline);
        } else {
            return Ok(KeyStatus::ExternallyModified);
        }
    }

    // Key is modified by tweaks
    Ok(KeyStatus::Modified {
        by_tweaks: tweaks,
        ref_count: state.get_key_ref_count(key_id),
    })
}

// ============================================================================
// Recovery and Diagnostics
// ============================================================================

/// Run diagnostics and suggest recovery actions
pub fn run_diagnostics() -> Result<Vec<RecoverySuggestion>, Error> {
    log::info!("Running backup system diagnostics");

    let baseline = load_baseline()?;
    let state = load_tweak_state()?;
    let mut suggestions = Vec::new();

    // Check for tweaks in state that reference keys not in baseline
    for (tweak_id, info) in &state.applied_tweaks {
        for key_id in &info.modified_keys {
            if !baseline.has_entry(key_id) {
                suggestions.push(RecoverySuggestion {
                    issue: format!(
                        "Tweak '{}' references key '{}' which has no baseline",
                        tweak_id, key_id
                    ),
                    action: RecoveryAction::CaptureNewBaseline,
                    key_id: Some(key_id.clone()),
                    tweak_id: Some(tweak_id.clone()),
                });
            }
        }
    }

    // Check for orphaned baseline entries (keys not referenced by any tweak but modified)
    for (key_id, entry) in &baseline.entries {
        let ref_count = state.get_key_ref_count(key_id);
        if ref_count == 0 {
            // Read current value
            let current = read_current_value(
                &entry.hive,
                &entry.key,
                &entry.value_name,
                &entry.value_type,
            )?;

            // If current doesn't match baseline, it might be externally modified
            if current.0 != entry.original_value {
                suggestions.push(RecoverySuggestion {
                    issue: format!(
                        "Key '{}' is not at baseline but no tweak claims ownership",
                        key_id
                    ),
                    action: RecoveryAction::RestoreBaseline,
                    key_id: Some(key_id.clone()),
                    tweak_id: None,
                });
            }
        }
    }

    log::info!("Diagnostics found {} issues", suggestions.len());
    Ok(suggestions)
}

/// Reset all state (use with caution - for recovery only)
pub fn reset_all_state() -> Result<(), Error> {
    log::warn!("Resetting all backup state - this should only be used for recovery!");

    let baseline_path = get_baseline_path()?;
    let state_path = get_tweak_state_path()?;

    if baseline_path.exists() {
        fs::remove_file(&baseline_path)
            .map_err(|e| Error::BackupFailed(format!("Failed to delete baseline: {}", e)))?;
        log::info!("Deleted baseline file");
    }

    if state_path.exists() {
        fs::remove_file(&state_path)
            .map_err(|e| Error::BackupFailed(format!("Failed to delete tweak state: {}", e)))?;
        log::info!("Deleted tweak state file");
    }

    Ok(())
}

// ============================================================================
// Legacy Compatibility - Migration from old backup format
// ============================================================================

/// Check for and migrate old-style backups
pub fn migrate_legacy_backups() -> Result<usize, Error> {
    log::info!("Checking for legacy backup files to migrate");

    let backups_dir = get_backups_dir()?;
    let mut migrated_count = 0;

    // Look for old-style .json files (excluding our new state files)
    for entry in fs::read_dir(&backups_dir).map_err(|e| Error::BackupFailed(e.to_string()))? {
        let entry = entry.map_err(|e| Error::BackupFailed(e.to_string()))?;
        let filename = entry.file_name().to_string_lossy().to_string();

        // Skip our new state files
        if filename == BASELINE_FILE || filename == TWEAK_STATE_FILE {
            continue;
        }

        // Check if it's an old backup file
        if filename.ends_with(".json") {
            log::debug!("Found potential legacy backup: {}", filename);

            // Try to read and migrate
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(legacy) = serde_json::from_str::<LegacyTweakBackup>(&content) {
                    log::info!("Migrating legacy backup for tweak '{}'", legacy.tweak_id);

                    // Add entries to baseline
                    let mut baseline = load_baseline()?;
                    let now = chrono::Local::now().to_rfc3339();

                    for entry in &legacy.entries {
                        let key_id =
                            make_key_id(entry.hive.as_str(), &entry.key, &entry.value_name);

                        if !baseline.has_entry(&key_id) {
                            let baseline_entry = BaselineEntry {
                                hive: entry.hive.as_str().to_string(),
                                key: entry.key.clone(),
                                value_name: entry.value_name.clone(),
                                value_type: format!("{:?}", entry.value_type),
                                original_value: entry.original_value.clone(),
                                key_existed: entry.key_existed,
                                captured_at: now.clone(),
                                captured_by_tweak: legacy.tweak_id.clone(),
                            };

                            baseline.add_entry(key_id, baseline_entry);
                        }
                    }

                    save_baseline(&baseline)?;
                    migrated_count += 1;

                    // Rename old file to .migrated
                    let new_path = entry.path().with_extension("json.migrated");
                    let _ = fs::rename(entry.path(), new_path);
                }
            }
        }
    }

    if migrated_count > 0 {
        log::info!("Migrated {} legacy backup files", migrated_count);
    }

    Ok(migrated_count)
}

// Legacy backup format for migration
#[derive(Debug, serde::Deserialize)]
struct LegacyTweakBackup {
    tweak_id: String,
    #[allow(dead_code)]
    tweak_name: String,
    #[allow(dead_code)]
    created_at: String,
    #[allow(dead_code)]
    windows_version: String,
    entries: Vec<LegacyBackupEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct LegacyBackupEntry {
    hive: crate::models::RegistryHive,
    key: String,
    value_name: String,
    value_type: crate::models::RegistryValueType,
    original_value: Option<serde_json::Value>,
    key_existed: bool,
}

// ============================================================================
// Backward Compatibility Functions (for existing code that expects old API)
// ============================================================================

/// Check if a backup exists for a tweak (now checks tweak state)
#[allow(dead_code)]
pub fn backup_exists(tweak_id: &str) -> Result<bool, Error> {
    let state = load_tweak_state()?;
    Ok(state.is_tweak_applied(tweak_id))
}

/// List all backup tweak IDs (now returns applied tweaks)
#[allow(dead_code)]
pub fn list_backups() -> Result<Vec<String>, Error> {
    get_applied_tweaks()
}

// ============================================================================
// Atomic Operation Helpers
// ============================================================================

use crate::models::{RollbackReport, ValueSnapshot, VerifyResult};

/// Capture current values for a list of registry changes before modification.
/// This creates snapshots that can be used for rollback if any write fails.
pub fn capture_snapshots(
    changes: &[&crate::models::RegistryChange],
) -> Result<Vec<ValueSnapshot>, Error> {
    log::debug!("Capturing {} registry snapshots", changes.len());

    let mut snapshots = Vec::with_capacity(changes.len());

    for change in changes {
        let hive_str = change.hive.as_str().to_string();
        let value_type_str = format!("{:?}", change.value_type).replace('"', "");

        // Use the existing read_current_value function which returns (Option<Value>, bool)
        let (value_before, key_existed) =
            read_current_value(&hive_str, &change.key, &change.value_name, &value_type_str)?;

        snapshots.push(ValueSnapshot {
            hive: hive_str,
            key: change.key.clone(),
            value_name: change.value_name.clone(),
            value_type: value_type_str,
            value_before,
            key_existed,
        });
    }

    log::debug!("Captured {} snapshots successfully", snapshots.len());
    Ok(snapshots)
}

/// Verify that all registry changes were applied correctly.
/// Reads back the values and compares with expected values.
pub fn verify_changes(
    changes: &[&crate::models::RegistryChange],
    is_enable: bool,
) -> Result<Vec<VerifyResult>, Error> {
    log::debug!("Verifying {} registry changes", changes.len());

    let mut results = Vec::with_capacity(changes.len());

    for change in changes {
        let hive_str = change.hive.as_str().to_string();
        let value_type_str = format!("{:?}", change.value_type).replace('"', "");

        // Get expected value
        let expected = if is_enable {
            Some(change.enable_value.clone())
        } else {
            change.disable_value.clone()
        };

        // Read actual value
        let (actual, _) =
            read_current_value(&hive_str, &change.key, &change.value_name, &value_type_str)?;

        let key_id = make_key_id(change.hive.as_str(), &change.key, &change.value_name);

        let matches = compare_values(&expected, &actual);

        if !matches {
            log::warn!(
                "Verification failed for {}: expected {:?}, got {:?}",
                key_id,
                expected,
                actual
            );
        }

        results.push(VerifyResult {
            key_id,
            matches,
            expected,
            actual,
        });
    }

    let failed_count = results.iter().filter(|r| !r.matches).count();
    if failed_count > 0 {
        log::warn!("{} of {} verifications failed", failed_count, results.len());
    } else {
        log::debug!("All {} verifications passed", results.len());
    }

    Ok(results)
}

/// Compare expected value with actual value from registry
fn compare_values(
    expected: &Option<serde_json::Value>,
    actual: &Option<serde_json::Value>,
) -> bool {
    match (expected, actual) {
        (Some(exp), Some(act)) => {
            // Handle numeric comparison (DWORD/QWORD values may come as different integer types)
            if let (Some(exp_num), Some(act_num)) = (exp.as_u64(), act.as_u64()) {
                return exp_num == act_num;
            }
            if let (Some(exp_num), Some(act_num)) = (exp.as_i64(), act.as_i64()) {
                return exp_num == act_num;
            }
            // String comparison
            if let (Some(exp_str), Some(act_str)) = (exp.as_str(), act.as_str()) {
                return exp_str == act_str;
            }
            // Array comparison (for binary values)
            if let (Some(exp_arr), Some(act_arr)) = (exp.as_array(), act.as_array()) {
                return exp_arr == act_arr;
            }
            // Fall back to direct comparison
            exp == act
        }
        (None, None) => true,
        _ => false,
    }
}

/// Rollback changes using captured snapshots.
/// Attempts to restore all values to their pre-modification state.
pub fn rollback_from_snapshots(snapshots: &[ValueSnapshot]) -> RollbackReport {
    log::info!("Rolling back {} registry changes", snapshots.len());

    let mut succeeded = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for snapshot in snapshots {
        let hive = match snapshot.hive.as_str() {
            "HKCU" => crate::models::RegistryHive::HKCU,
            "HKLM" => crate::models::RegistryHive::HKLM,
            _ => {
                log::error!("Unknown hive in snapshot: {}", snapshot.hive);
                failed += 1;
                let key_id = make_key_id(&snapshot.hive, &snapshot.key, &snapshot.value_name);
                failures.push((key_id, format!("Unknown hive '{}'", snapshot.hive)));
                continue;
            }
        };

        let result = restore_snapshot_value(&hive, snapshot);
        let key_id = make_key_id(&snapshot.hive, &snapshot.key, &snapshot.value_name);

        match result {
            Ok(()) => {
                succeeded += 1;
                log::trace!(
                    "Rolled back {}\\{}\\{}",
                    snapshot.hive,
                    snapshot.key,
                    snapshot.value_name
                );
            }
            Err(e) => {
                failed += 1;
                log::error!("Rollback failed for {}: {}", key_id, e);
                failures.push((key_id, e.to_string()));
            }
        }
    }

    let all_succeeded = failed == 0;
    if all_succeeded {
        log::info!(
            "Rollback completed successfully: {} changes restored",
            succeeded
        );
    } else {
        log::error!(
            "Rollback partially failed: {} succeeded, {} failed",
            succeeded,
            failed
        );
    }

    RollbackReport {
        succeeded,
        failed,
        failures,
        all_succeeded,
    }
}

/// Restore a single registry value from a snapshot
fn restore_snapshot_value(
    hive: &crate::models::RegistryHive,
    snapshot: &ValueSnapshot,
) -> Result<(), Error> {
    match &snapshot.value_before {
        Some(value) => {
            // Restore to original value
            if let Some(dword) = value.as_u64() {
                super::registry_service::set_dword(
                    hive,
                    &snapshot.key,
                    &snapshot.value_name,
                    dword as u32,
                )?;
            } else if let Some(qword) = value.as_u64() {
                super::registry_service::set_qword(
                    hive,
                    &snapshot.key,
                    &snapshot.value_name,
                    qword,
                )?;
            } else if let Some(string) = value.as_str() {
                super::registry_service::set_string(
                    hive,
                    &snapshot.key,
                    &snapshot.value_name,
                    string,
                )?;
            } else if let Some(arr) = value.as_array() {
                let bytes: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                super::registry_service::set_binary(
                    hive,
                    &snapshot.key,
                    &snapshot.value_name,
                    &bytes,
                )?;
            } else {
                return Err(Error::BackupFailed(format!(
                    "Unsupported value type for restore: {:?}",
                    value
                )));
            }
        }
        None => {
            // Value didn't exist before - delete it
            // Note: For now we log this but don't actually delete,
            // as deleting registry values can be risky
            log::debug!(
                "Value {}\\{}\\{} didn't exist before, skipping deletion",
                snapshot.hive,
                snapshot.key,
                snapshot.value_name
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_backups_dir() {
        let result = get_backups_dir();
        assert!(result.is_ok());
    }
}
