use crate::error::Error;
use crate::models::RegistryHive;
use std::fs;
use std::path::PathBuf;

/// Get the backups directory path (in root directory as portable app)
pub fn get_backups_dir() -> Result<PathBuf, Error> {
    let exe_path = std::env::current_exe()
        .map_err(|e| Error::BackupFailed(format!("Failed to get exe path: {}", e)))?;

    let root = exe_path.parent().ok_or_else(|| {
        Error::BackupFailed("Could not determine executable parent directory".to_string())
    })?;

    let backups_dir = root.join("backups");

    // Create backups directory if it doesn't exist
    fs::create_dir_all(&backups_dir)
        .map_err(|e| Error::BackupFailed(format!("Failed to create backups directory: {}", e)))?;

    Ok(backups_dir)
}

/// Generate a timestamp string for backup filenames
fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Create a backup of a registry key
#[allow(unused_variables)]
pub fn backup_registry_key(
    hive: &RegistryHive,
    key_path: &str,
    backup_name: Option<&str>,
) -> Result<String, Error> {
    let backups_dir = get_backups_dir()?;

    // Generate backup filename with timestamp if not provided
    let filename = if let Some(name) = backup_name {
        format!("{}.json", name)
    } else {
        let timestamp = get_timestamp();
        let safe_key = key_path.replace('\\', "_");
        format!("{}_{}.json", safe_key, timestamp)
    };

    let backup_path = backups_dir.join(&filename);

    // TODO: Read registry key and its values, then serialize to JSON
    // For now, this is a stub that will be implemented with registry reading

    Ok(backup_path.to_string_lossy().to_string())
}

/// Restore a registry key from a backup
#[allow(unused_variables)]
pub fn restore_registry_key(backup_path: &str) -> Result<(), Error> {
    // TODO: Read JSON backup file and restore registry values
    // This will require reading the backup file and applying registry changes

    Ok(())
}

/// List all backups
pub fn list_backups() -> Result<Vec<String>, Error> {
    let backups_dir = get_backups_dir()?;

    let mut backups = Vec::new();

    if backups_dir.exists() {
        for entry in fs::read_dir(&backups_dir).map_err(|e| Error::BackupFailed(e.to_string()))? {
            let entry = entry.map_err(|e| Error::BackupFailed(e.to_string()))?;

            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    backups.push(filename.to_string());
                }
            }
        }
    }

    Ok(backups)
}

/// Delete a backup
pub fn delete_backup(backup_name: &str) -> Result<(), Error> {
    let backups_dir = get_backups_dir()?;
    let backup_path = backups_dir.join(backup_name);

    fs::remove_file(&backup_path)
        .map_err(|e| Error::BackupFailed(format!("Failed to delete backup: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backups_dir_creation() {
        let result = get_backups_dir();
        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(dir.exists());
    }
}
