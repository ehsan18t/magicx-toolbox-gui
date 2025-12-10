use crate::error::Error;
use crate::models::{RegistryChange, RegistryHive, RegistryValueType};
use crate::services::registry_service;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Backup entry for a single registry value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub hive: RegistryHive,
    pub key: String,
    pub value_name: String,
    pub value_type: RegistryValueType,
    pub original_value: Option<serde_json::Value>,
    pub key_existed: bool,
}

/// Complete backup for a tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakBackup {
    pub tweak_id: String,
    pub tweak_name: String,
    pub created_at: String,
    pub windows_version: String,
    pub entries: Vec<BackupEntry>,
}

/// Get the backups directory path (in root directory as portable app)
pub fn get_backups_dir() -> Result<PathBuf, Error> {
    let exe_path = std::env::current_exe()
        .map_err(|e| Error::BackupFailed(format!("Failed to get exe path: {}", e)))?;

    let root = exe_path.parent().ok_or_else(|| {
        Error::BackupFailed("Could not determine executable parent directory".to_string())
    })?;

    let backups_dir = root.join("backups");
    log::trace!("Backups directory: {:?}", backups_dir);

    // Create backups directory if it doesn't exist
    if !backups_dir.exists() {
        log::debug!("Creating backups directory: {:?}", backups_dir);
        fs::create_dir_all(&backups_dir).map_err(|e| {
            Error::BackupFailed(format!("Failed to create backups directory: {}", e))
        })?;
    }

    Ok(backups_dir)
}

/// Get backup file path for a tweak
pub fn get_backup_path(tweak_id: &str) -> Result<PathBuf, Error> {
    let backups_dir = get_backups_dir()?;
    Ok(backups_dir.join(format!("{}.json", tweak_id)))
}

/// Check if a backup exists for a tweak
pub fn backup_exists(tweak_id: &str) -> Result<bool, Error> {
    let backup_path = get_backup_path(tweak_id)?;
    let exists = backup_path.exists();
    log::trace!("Backup exists for '{}': {}", tweak_id, exists);
    Ok(exists)
}

/// Read current registry values for a set of changes (for backup)
fn read_current_values(changes: &[&RegistryChange]) -> Result<Vec<BackupEntry>, Error> {
    let mut entries = Vec::new();
    log::trace!(
        "Reading current values for {} registry changes",
        changes.len()
    );

    for change in changes {
        // Check if key exists first
        let key_existed = registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);
        log::trace!(
            "Registry key {:?}\\{} exists: {}",
            change.hive,
            change.key,
            key_existed
        );

        let original_value = if key_existed {
            match change.value_type {
                RegistryValueType::DWord => {
                    registry_service::read_dword(&change.hive, &change.key, &change.value_name)?
                        .map(|v| serde_json::json!(v))
                }
                RegistryValueType::String | RegistryValueType::ExpandString => {
                    registry_service::read_string(&change.hive, &change.key, &change.value_name)?
                        .map(|v| serde_json::json!(v))
                }
                RegistryValueType::Binary => {
                    registry_service::read_binary(&change.hive, &change.key, &change.value_name)?
                        .map(|v| serde_json::json!(v))
                }
                _ => None,
            }
        } else {
            None
        };

        entries.push(BackupEntry {
            hive: change.hive.clone(),
            key: change.key.clone(),
            value_name: change.value_name.clone(),
            value_type: change.value_type.clone(),
            original_value,
            key_existed,
        });
    }

    log::trace!("Read {} backup entries", entries.len());
    Ok(entries)
}

/// Create a backup before applying a tweak
pub fn create_tweak_backup(
    tweak_id: &str,
    tweak_name: &str,
    windows_version: u32,
    changes: &[&RegistryChange],
) -> Result<String, Error> {
    log::info!("Creating backup for tweak '{}' ({})", tweak_id, tweak_name);
    let entries = read_current_values(changes)?;

    let backup = TweakBackup {
        tweak_id: tweak_id.to_string(),
        tweak_name: tweak_name.to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        windows_version: windows_version.to_string(),
        entries,
    };

    let backup_path = get_backup_path(tweak_id)?;
    let json = serde_json::to_string_pretty(&backup)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize backup: {}", e)))?;

    fs::write(&backup_path, json)
        .map_err(|e| Error::BackupFailed(format!("Failed to write backup file: {}", e)))?;

    log::debug!("Backup saved to {:?}", backup_path);
    Ok(backup_path.to_string_lossy().to_string())
}

/// Load a backup from disk
pub fn load_backup(tweak_id: &str) -> Result<Option<TweakBackup>, Error> {
    let backup_path = get_backup_path(tweak_id)?;

    if !backup_path.exists() {
        log::trace!("No backup found at {:?}", backup_path);
        return Ok(None);
    }

    log::trace!("Loading backup from {:?}", backup_path);
    let content = fs::read_to_string(&backup_path)
        .map_err(|e| Error::BackupFailed(format!("Failed to read backup file: {}", e)))?;

    let backup: TweakBackup = serde_json::from_str(&content)
        .map_err(|e| Error::BackupFailed(format!("Failed to parse backup file: {}", e)))?;

    log::debug!(
        "Loaded backup for tweak '{}' created at {}",
        backup.tweak_id,
        backup.created_at
    );
    Ok(Some(backup))
}

/// Restore registry values from a backup
pub fn restore_from_backup(tweak_id: &str) -> Result<(), Error> {
    log::info!("Restoring from backup for tweak '{}'", tweak_id);
    let backup = load_backup(tweak_id)?
        .ok_or_else(|| Error::BackupFailed(format!("No backup found for tweak: {}", tweak_id)))?;

    log::debug!("Backup has {} entries to restore", backup.entries.len());
    let mut restored_count = 0;
    let mut skipped_count = 0;

    for entry in &backup.entries {
        match &entry.original_value {
            Some(value) => {
                log::trace!(
                    "Restoring {:?}\\{}\\{} to original value",
                    entry.hive,
                    entry.key,
                    entry.value_name
                );
                // Restore to original value
                match entry.value_type {
                    RegistryValueType::DWord => {
                        if let Some(v) = value.as_u64() {
                            registry_service::set_dword(
                                &entry.hive,
                                &entry.key,
                                &entry.value_name,
                                v as u32,
                            )?;
                            restored_count += 1;
                        }
                    }
                    RegistryValueType::String | RegistryValueType::ExpandString => {
                        if let Some(v) = value.as_str() {
                            registry_service::set_string(
                                &entry.hive,
                                &entry.key,
                                &entry.value_name,
                                v,
                            )?;
                            restored_count += 1;
                        }
                    }
                    RegistryValueType::Binary => {
                        if let Some(arr) = value.as_array() {
                            let binary: Vec<u8> = arr
                                .iter()
                                .filter_map(|v| v.as_u64().map(|u| u as u8))
                                .collect();
                            registry_service::set_binary(
                                &entry.hive,
                                &entry.key,
                                &entry.value_name,
                                &binary,
                            )?;
                            restored_count += 1;
                        }
                    }
                    _ => {
                        skipped_count += 1;
                    }
                }
            }
            None => {
                // Value didn't exist before - we could delete it but for safety we skip
                log::trace!(
                    "Skipping {:?}\\{}\\{} (no original value)",
                    entry.hive,
                    entry.key,
                    entry.value_name
                );
                skipped_count += 1;
            }
        }
    }

    log::info!(
        "Restore complete for '{}': {} restored, {} skipped",
        tweak_id,
        restored_count,
        skipped_count
    );
    Ok(())
}

/// Delete a backup file
pub fn delete_backup(tweak_id: &str) -> Result<(), Error> {
    let backup_path = get_backup_path(tweak_id)?;

    if backup_path.exists() {
        log::debug!(
            "Deleting backup for tweak '{}' at {:?}",
            tweak_id,
            backup_path
        );
        fs::remove_file(&backup_path)
            .map_err(|e| Error::BackupFailed(format!("Failed to delete backup: {}", e)))?;
    } else {
        log::trace!("No backup to delete for tweak '{}'", tweak_id);
    }

    Ok(())
}

/// List all backup tweak IDs
pub fn list_backups() -> Result<Vec<String>, Error> {
    let backups_dir = get_backups_dir()?;

    let mut backups = Vec::new();

    if backups_dir.exists() {
        for entry in fs::read_dir(&backups_dir).map_err(|e| Error::BackupFailed(e.to_string()))? {
            let entry = entry.map_err(|e| Error::BackupFailed(e.to_string()))?;

            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    // Extract tweak_id from filename (remove .json)
                    backups.push(filename.trim_end_matches(".json").to_string());
                }
            }
        }
    }

    log::trace!("Found {} backups in {:?}", backups.len(), backups_dir);
    Ok(backups)
}

/// Get backup info without loading full backup
pub fn get_backup_info(tweak_id: &str) -> Result<Option<(String, String)>, Error> {
    let backup = load_backup(tweak_id)?;
    Ok(backup.map(|b| (b.tweak_name, b.created_at)))
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
