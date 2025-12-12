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
        })),
        None => Ok(None),
    }
}

/// Get status of the backup system
#[derive(Debug, Clone, Serialize)]
pub struct BackupSystemStatus {
    pub total_applied_tweaks: usize,
    pub snapshots_dir: String,
}

#[tauri::command]
pub fn get_backup_system_status() -> Result<BackupSystemStatus> {
    let applied = backup_service::get_applied_tweaks()?;
    let snapshots_dir = backup_service::get_snapshots_dir()?
        .to_string_lossy()
        .to_string();

    Ok(BackupSystemStatus {
        total_applied_tweaks: applied.len(),
        snapshots_dir,
    })
}

/// Clean up old backup files (migration from old format)
#[tauri::command]
pub fn cleanup_old_backups() -> Result<()> {
    backup_service::cleanup_old_backups()
}

/// Validate all snapshots on app startup
/// Removes stale snapshots where current registry state matches the snapshot state
/// Returns the number of stale snapshots removed
#[tauri::command]
pub fn validate_snapshots() -> Result<u32> {
    backup_service::validate_all_snapshots()
}
