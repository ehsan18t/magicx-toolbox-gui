use crate::error::{Error, Result};
use crate::models::{ConflictReport, RecoverySuggestion};
use crate::services::backup_service;
use serde::Serialize;

/// Backup information for frontend display
#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub tweak_id: String,
    pub tweak_name: String,
    pub applied_at: String,
    pub windows_version: u32,
    pub modified_keys_count: usize,
}

/// Check if a tweak is recorded as applied in our state
#[tauri::command]
pub fn has_backup(tweak_id: String) -> Result<bool> {
    backup_service::is_tweak_applied(&tweak_id)
}

/// List all applied tweaks (tracked by our backup system)
#[tauri::command]
pub fn list_backups() -> Result<Vec<String>> {
    backup_service::get_applied_tweaks()
}

/// Get backup information for a tweak
#[tauri::command]
pub fn get_backup_info(tweak_id: String) -> Result<Option<BackupInfo>> {
    let state = backup_service::load_tweak_state()?;
    Ok(state.get_applied_tweak(&tweak_id).map(|info| BackupInfo {
        tweak_id: info.tweak_id.clone(),
        tweak_name: info.tweak_name.clone(),
        applied_at: info.applied_at.clone(),
        windows_version: info.windows_version,
        modified_keys_count: info.modified_keys.len(),
    }))
}

/// Restore a specific key to its baseline value (emergency recovery)
#[tauri::command]
pub fn restore_key_to_baseline(key_id: String) -> Result<bool> {
    log::info!("Restoring key {} to baseline", key_id);
    backup_service::restore_key_to_baseline(&key_id)
}

/// Get conflict report for a tweak
#[tauri::command]
pub fn get_tweak_conflicts(tweak_id: String) -> Result<ConflictReport> {
    let state = backup_service::load_tweak_state()?;

    // Get keys for this tweak
    let info = state.get_applied_tweak(&tweak_id).ok_or_else(|| {
        Error::BackupFailed(format!("Tweak '{}' is not currently applied", tweak_id))
    })?;

    let keys: Vec<(String, String, String)> = info
        .modified_keys
        .iter()
        .filter_map(crate::models::parse_key_id)
        .collect();

    backup_service::detect_conflicts(&tweak_id, &keys)
}

/// Run diagnostics on the backup system
#[tauri::command]
pub fn run_backup_diagnostics() -> Result<Vec<RecoverySuggestion>> {
    backup_service::run_diagnostics()
}

/// Migrate legacy backup files to new format
#[tauri::command]
pub fn migrate_legacy_backups() -> Result<usize> {
    backup_service::migrate_legacy_backups()
}

/// Reset all backup state (emergency recovery only)
#[tauri::command]
pub fn reset_backup_state() -> Result<()> {
    log::warn!("Resetting all backup state via command");
    backup_service::reset_all_state()
}

/// Get the baseline entry for a registry key
#[tauri::command]
pub fn get_baseline_entry(key_id: String) -> Result<Option<crate::models::BaselineEntry>> {
    backup_service::get_baseline_value(&key_id)
}

/// Get status of the backup system
#[derive(Debug, Clone, Serialize)]
pub struct BackupSystemStatus {
    pub total_applied_tweaks: usize,
    pub total_baseline_entries: usize,
    pub total_tracked_keys: usize,
}

#[tauri::command]
pub fn get_backup_system_status() -> Result<BackupSystemStatus> {
    let state = backup_service::load_tweak_state()?;
    let baseline = backup_service::load_baseline()?;

    Ok(BackupSystemStatus {
        total_applied_tweaks: state.applied_tweaks.len(),
        total_baseline_entries: baseline.entries.len(),
        total_tracked_keys: state.key_ownership.len(),
    })
}
