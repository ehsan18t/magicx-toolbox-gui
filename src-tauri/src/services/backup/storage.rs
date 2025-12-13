//! Snapshot Storage Operations
//!
//! File I/O functions for snapshot persistence:
//! - Directory and path management
//! - Save, load, delete snapshots
//! - List applied tweaks

use crate::error::Error;
use crate::models::TweakSnapshot;
use fs4::fs_std::FileExt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

const SNAPSHOTS_DIR: &str = "snapshots";

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

pub(crate) fn get_snapshot_path(tweak_id: &str) -> Result<PathBuf, Error> {
    Ok(get_snapshots_dir()?.join(format!("{}.json", tweak_id)))
}

/// Save snapshot to disk with exclusive file locking
pub fn save_snapshot(snapshot: &TweakSnapshot) -> Result<(), Error> {
    let path = get_snapshot_path(&snapshot.tweak_id)?;

    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize snapshot: {}", e)))?;

    // Create/open file and acquire exclusive lock
    let file = File::create(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to create snapshot file: {}", e)))?;

    file.lock_exclusive()
        .map_err(|e| Error::BackupFailed(format!("Failed to acquire file lock: {}", e)))?;

    // Write content while holding lock
    let mut file = file;
    file.write_all(json.as_bytes())
        .map_err(|e| Error::BackupFailed(format!("Failed to write snapshot: {}", e)))?;

    // Lock is automatically released when file is dropped
    log::debug!("Saved snapshot to {:?}", path);
    Ok(())
}

/// Update the snapshot metadata (option index/label) after successfully switching options.
/// The original registry/service/scheduler values are preserved (for full revert capability).
/// Uses file locking for concurrency safety.
pub fn update_snapshot_metadata(
    tweak_id: &str,
    new_option_index: usize,
    new_option_label: &str,
) -> Result<(), Error> {
    let path = get_snapshot_path(tweak_id)?;

    if !path.exists() {
        return Err(Error::BackupFailed(format!(
            "No snapshot found for tweak '{}'",
            tweak_id
        )));
    }

    // Open file and acquire exclusive lock for atomic read-modify-write
    let file = File::options()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to open snapshot: {}", e)))?;

    file.lock_exclusive()
        .map_err(|e| Error::BackupFailed(format!("Failed to acquire file lock: {}", e)))?;

    // Read current content
    let mut content = String::new();
    let mut file = file;
    file.read_to_string(&mut content)
        .map_err(|e| Error::BackupFailed(format!("Failed to read snapshot: {}", e)))?;

    let mut snapshot: TweakSnapshot = serde_json::from_str(&content)
        .map_err(|e| Error::BackupFailed(format!("Failed to parse snapshot: {}", e)))?;

    log::debug!(
        "Updating snapshot metadata: option {} '{}' → {} '{}'",
        snapshot.applied_option_index,
        snapshot.applied_option_label,
        new_option_index,
        new_option_label
    );

    snapshot.applied_option_index = new_option_index;
    snapshot.applied_option_label = new_option_label.to_string();

    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize snapshot: {}", e)))?;

    // Truncate and rewrite while holding lock
    file.set_len(0)
        .map_err(|e| Error::BackupFailed(format!("Failed to truncate snapshot file: {}", e)))?;

    // Seek to beginning after truncate
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(0))
        .map_err(|e| Error::BackupFailed(format!("Failed to seek in snapshot file: {}", e)))?;

    file.write_all(json.as_bytes())
        .map_err(|e| Error::BackupFailed(format!("Failed to write snapshot: {}", e)))?;

    // Lock is automatically released when file is dropped
    log::info!(
        "Updated snapshot metadata for '{}' to option '{}'",
        tweak_id,
        new_option_label
    );
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
