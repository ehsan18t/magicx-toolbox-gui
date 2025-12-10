use crate::error::Result;
use crate::services::backup_service;
use serde::Serialize;

/// Backup information for frontend display
#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub tweak_id: String,
    pub tweak_name: String,
    pub created_at: String,
}

/// Check if a backup exists for a tweak
#[tauri::command]
pub fn has_backup(tweak_id: String) -> Result<bool> {
    backup_service::backup_exists(&tweak_id)
}

/// List all available backups
#[tauri::command]
pub fn list_backups() -> Result<Vec<String>> {
    backup_service::list_backups()
}

/// Get backup information
#[tauri::command]
pub fn get_backup_info(tweak_id: String) -> Result<Option<BackupInfo>> {
    let backup = backup_service::load_backup(&tweak_id)?;
    Ok(backup.map(|b| BackupInfo {
        tweak_id: b.tweak_id,
        tweak_name: b.tweak_name,
        created_at: b.created_at,
    }))
}

/// Restore a tweak from its backup
#[tauri::command]
pub fn restore_from_backup(tweak_id: String) -> Result<()> {
    backup_service::restore_from_backup(&tweak_id)
}

/// Delete a backup
#[tauri::command]
pub fn delete_backup(tweak_id: String) -> Result<()> {
    backup_service::delete_backup(&tweak_id)
}
