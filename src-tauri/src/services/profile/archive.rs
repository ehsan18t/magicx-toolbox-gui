//! Profile Archive Handling
//!
//! Read and write .mgx profile archives (ZIP format).

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::error::Error;
use crate::models::{
    ConfigurationProfile, ProfileManifest, SystemStateSnapshot, PROFILE_SCHEMA_VERSION,
};

/// Contents of a profile archive.
#[allow(dead_code)]
pub struct ProfileArchiveContents {
    pub profile: ConfigurationProfile,
    pub system_state: Option<SystemStateSnapshot>,
}

/// Write a profile archive to a file.
pub fn write_profile_archive(
    path: &Path,
    profile: &ConfigurationProfile,
    system_state: Option<&SystemStateSnapshot>,
) -> Result<(), Error> {
    log::info!("Writing profile archive to {}", path.display());

    let file = File::create(path)
        .map_err(|e| Error::ProfileError(format!("Failed to create archive file: {}", e)))?;

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    // Serialize and write profile.json
    let profile_json = serde_json::to_string_pretty(profile)
        .map_err(|e| Error::ProfileError(format!("Failed to serialize profile: {}", e)))?;
    let profile_checksum = compute_checksum(profile_json.as_bytes());

    zip.start_file("profile.json", options)
        .map_err(|e| Error::ProfileError(format!("Failed to create profile.json: {}", e)))?;
    zip.write_all(profile_json.as_bytes())
        .map_err(|e| Error::ProfileError(format!("Failed to write profile.json: {}", e)))?;

    // Write system_state.json if provided
    let (includes_system_state, system_state_checksum) = if let Some(state) = system_state {
        let state_json = serde_json::to_string_pretty(state)
            .map_err(|e| Error::ProfileError(format!("Failed to serialize system state: {}", e)))?;
        let checksum = compute_checksum(state_json.as_bytes());

        zip.start_file("system_state.json", options).map_err(|e| {
            Error::ProfileError(format!("Failed to create system_state.json: {}", e))
        })?;
        zip.write_all(state_json.as_bytes()).map_err(|e| {
            Error::ProfileError(format!("Failed to write system_state.json: {}", e))
        })?;

        (true, Some(checksum))
    } else {
        (false, None)
    };

    // Create and write manifest
    let manifest = ProfileManifest {
        format_version: PROFILE_SCHEMA_VERSION,
        profile_checksum,
        includes_system_state,
        system_state_checksum,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| Error::ProfileError(format!("Failed to serialize manifest: {}", e)))?;

    zip.start_file("manifest.json", options)
        .map_err(|e| Error::ProfileError(format!("Failed to create manifest.json: {}", e)))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| Error::ProfileError(format!("Failed to write manifest.json: {}", e)))?;

    zip.finish()
        .map_err(|e| Error::ProfileError(format!("Failed to finalize archive: {}", e)))?;

    log::info!("Profile archive written successfully");
    Ok(())
}

/// Maximum allowed profile archive size (10 MB)
const MAX_PROFILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum allowed uncompressed file size within archive (10 MB)
/// Protects against ZIP bombs (small compressed files that expand to huge sizes)
const MAX_UNCOMPRESSED_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Read a file from the archive with size limit protection.
/// Returns an error if the uncompressed size exceeds the limit.
fn read_archive_file_safe(zip: &mut ZipArchive<File>, name: &str) -> Result<String, Error> {
    let mut file = zip
        .by_name(name)
        .map_err(|e| Error::ProfileError(format!("Missing {}: {}", name, e)))?;

    // Check uncompressed size to protect against ZIP bombs
    if file.size() > MAX_UNCOMPRESSED_FILE_SIZE {
        return Err(Error::ProfileError(format!(
            "File {} exceeds maximum allowed size: {} bytes (max {} bytes)",
            name,
            file.size(),
            MAX_UNCOMPRESSED_FILE_SIZE
        )));
    }

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| Error::ProfileError(format!("Failed to read {}: {}", name, e)))?;

    Ok(contents)
}

/// Read a profile archive from a file.
pub fn read_profile_archive(path: &Path) -> Result<ProfileArchiveContents, Error> {
    log::info!("Reading profile archive from {}", path.display());

    // Check file size before opening
    let metadata = std::fs::metadata(path)
        .map_err(|e| Error::ProfileError(format!("Failed to read file metadata: {}", e)))?;

    if metadata.len() > MAX_PROFILE_SIZE {
        return Err(Error::ProfileError(format!(
            "Profile file too large: {} bytes (max {} bytes)",
            metadata.len(),
            MAX_PROFILE_SIZE
        )));
    }

    let file = File::open(path)
        .map_err(|e| Error::ProfileError(format!("Failed to open archive: {}", e)))?;

    let mut zip = ZipArchive::new(file)
        .map_err(|e| Error::ProfileError(format!("Invalid archive: {}", e)))?;

    // Read manifest (with ZIP bomb protection)
    let manifest: ProfileManifest = {
        let contents = read_archive_file_safe(&mut zip, "manifest.json")?;
        serde_json::from_str(&contents)
            .map_err(|e| Error::ProfileError(format!("Invalid manifest: {}", e)))?
    };

    // Read profile (with ZIP bomb protection)
    let profile: ConfigurationProfile = {
        let contents = read_archive_file_safe(&mut zip, "profile.json")?;

        // Verify checksum
        let checksum = compute_checksum(contents.as_bytes());
        if checksum != manifest.profile_checksum {
            return Err(Error::ProfileError("Profile checksum mismatch".to_string()));
        }

        serde_json::from_str(&contents)
            .map_err(|e| Error::ProfileError(format!("Invalid profile: {}", e)))?
    };

    // Read system state if present (with ZIP bomb protection)
    let system_state = if manifest.includes_system_state {
        let contents = read_archive_file_safe(&mut zip, "system_state.json")?;

        // Verify checksum if present
        if let Some(ref expected) = manifest.system_state_checksum {
            let checksum = compute_checksum(contents.as_bytes());
            if checksum != *expected {
                return Err(Error::ProfileError(
                    "System state checksum mismatch".to_string(),
                ));
            }
        }

        Some(
            serde_json::from_str(&contents)
                .map_err(|e| Error::ProfileError(format!("Invalid system state: {}", e)))?,
        )
    } else {
        None
    };

    Ok(ProfileArchiveContents {
        profile,
        system_state,
    })
}

/// Compute SHA-256 checksum of data.
fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
