use crate::error::Result;
use crate::services::backup_service;
use serde::Serialize;

/// Backup information for frontend display
#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub tweak_id: String,
    pub tweak_name: String,
    pub applied_at: String,
    pub windows_version: u32,
    pub registry_values_count: usize,
    pub service_snapshots_count: usize,
    pub scheduler_snapshots_count: usize,
}

/// Check if a tweak has a snapshot (is applied)
#[tauri::command]
pub fn has_backup(tweak_id: String) -> Result<bool> {
    backup_service::snapshot_exists(&tweak_id)
}

/// List all applied tweaks (by listing snapshot files)
#[tauri::command]
pub fn list_backups() -> Result<Vec<String>> {
    backup_service::get_applied_tweaks()
}

/// Get backup information for a tweak
#[tauri::command]
pub fn get_backup_info(tweak_id: String) -> Result<Option<BackupInfo>> {
    match backup_service::load_snapshot(&tweak_id)? {
        Some(snapshot) => Ok(Some(BackupInfo {
            tweak_id: snapshot.tweak_id,
            tweak_name: snapshot.tweak_name,
            applied_at: snapshot.created_at,
            windows_version: snapshot.windows_version,
            registry_values_count: snapshot.registry_snapshots.len(),
            service_snapshots_count: snapshot.service_snapshots.len(),
            scheduler_snapshots_count: snapshot.scheduler_snapshots.len(),
        })),
        None => Ok(None),
    }
}

/// Validate all snapshots on app startup
/// Removes stale snapshots where current registry state matches the snapshot state
/// Returns the number of stale snapshots removed
#[tauri::command]
pub fn validate_snapshots() -> Result<u32> {
    backup_service::validate_all_snapshots()
}
