//! Snapshot Storage Operations
//!
//! File I/O functions for snapshot persistence:
//! - Directory and path management
//! - Save, load, delete snapshots
//! - List applied tweaks

use crate::error::Error;
use crate::models::TweakSnapshot;
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

/// Save a snapshot to disk atomically.
///
/// The snapshot is the only record of the machine's original state, so a crash mid-write must never
/// leave a truncated or half-written file. Write to a temp file in the same directory, then
/// atomically rename it over the target (`NamedTempFile::persist` = `MoveFileExW` +
/// `MOVEFILE_REPLACE_EXISTING` on Windows). The replace is atomic, so no lock is needed and the last
/// writer wins with a complete file.
pub fn save_snapshot(snapshot: &TweakSnapshot) -> Result<(), Error> {
    let dir = get_snapshots_dir()?;
    let path = dir.join(format!("{}.json", snapshot.tweak_id));

    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| Error::BackupFailed(format!("Failed to serialize snapshot: {}", e)))?;

    let mut tmp = tempfile::NamedTempFile::new_in(&dir)
        .map_err(|e| Error::BackupFailed(format!("Failed to create temp snapshot file: {}", e)))?;
    tmp.write_all(json.as_bytes())
        .map_err(|e| Error::BackupFailed(format!("Failed to write snapshot: {}", e)))?;
    tmp.persist(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to persist snapshot: {}", e)))?;

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

    // Exclusive lock (std::fs::File::lock) for the read-modify-write, released when `file` drops.
    file.lock()
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

    // Warn (don't block) if the snapshot came from a different machine: its captured "original
    // state" describes another system, so restoring it here could target the wrong values.
    if let (Some(snap_guid), Some(current)) = (
        snapshot.machine_guid.as_deref(),
        crate::services::system_info_service::machine_guid(),
    ) {
        if snap_guid != current {
            log::warn!(
                "Snapshot for tweak '{}' was captured on a different machine (MachineGuid {} != {}); \
                 restoring it may target the wrong state",
                tweak_id,
                snap_guid,
                current
            );
        }
    }

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

/// Record that a tweak's revert did not fully succeed, so its status surfaces as Needs Attention
/// (ADR-0001). Only the flag and the unrestorable list are set; the snapshot's restore data is left
/// intact so a later retry still has the original values.
pub fn mark_needs_attention(tweak_id: &str, unrestorable: Vec<String>) -> Result<(), Error> {
    if let Some(mut snapshot) = load_snapshot(tweak_id)? {
        snapshot.needs_attention = true;
        snapshot.unrestorable_resources = unrestorable;
        let count = snapshot.unrestorable_resources.len();
        save_snapshot(&snapshot)?;
        log::info!(
            "Marked tweak '{}' as Needs Attention ({} unrestorable resource(s))",
            tweak_id,
            count
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_snapshot_atomically_replaces_an_existing_snapshot() {
        // Regression: the atomic write must *replace* an existing snapshot, not fail on
        // "file already exists" (a Windows rename hazard) — and it must leave a complete file.
        let id = format!("__wp2_atomic_test_{}", std::process::id());

        let mut v1 = TweakSnapshot::new(&id, "T", 0, "v1", 11, false, None);
        v1.applied_option_label = "v1".to_string();
        save_snapshot(&v1).unwrap();
        assert_eq!(
            load_snapshot(&id).unwrap().unwrap().applied_option_label,
            "v1"
        );

        let mut v2 = TweakSnapshot::new(&id, "T", 1, "v2", 11, false, None);
        v2.applied_option_label = "v2".to_string();
        save_snapshot(&v2).unwrap();
        assert_eq!(
            load_snapshot(&id).unwrap().unwrap().applied_option_label,
            "v2"
        );

        delete_snapshot(&id).unwrap();
    }

    #[test]
    fn mark_needs_attention_flags_the_snapshot_and_keeps_the_restore_data() {
        // ADR-0001: a partial revert marks the kept snapshot as Needs Attention without touching the
        // restore data, so a retry still has the original values.
        let id = format!("__wp5_needs_attention_{}", std::process::id());
        let snap = TweakSnapshot::new(&id, "T", 0, "opt", 11, false, None);
        assert!(
            !snap.needs_attention,
            "a fresh snapshot is not Needs Attention"
        );
        save_snapshot(&snap).unwrap();

        mark_needs_attention(&id, vec!["Service 'X': access denied".to_string()]).unwrap();

        let loaded = load_snapshot(&id).unwrap().unwrap();
        assert!(loaded.needs_attention);
        assert_eq!(
            loaded.unrestorable_resources,
            vec!["Service 'X': access denied".to_string()]
        );
        assert_eq!(loaded.tweak_id, id, "restore identity is preserved");

        delete_snapshot(&id).unwrap();
    }
}
