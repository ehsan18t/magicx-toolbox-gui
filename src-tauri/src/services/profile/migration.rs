//! Profile Migration
//!
//! Handles migration of configuration profiles across schema versions.

use crate::error::Error;
use crate::models::{ConfigurationProfile, PROFILE_SCHEMA_VERSION};
use serde::Serialize;

/// Result of a migration operation
#[derive(Debug, Clone, Serialize)]
pub struct MigrationNote {
    pub from_version: u32,
    pub to_version: u32,
    pub message: String,
}

/// Migrate a profile to the current schema version
pub fn migrate_profile(profile: &mut ConfigurationProfile) -> Result<Vec<MigrationNote>, Error> {
    let mut notes = Vec::new();

    // If schema version is newer than supported, return error
    if profile.schema_version > PROFILE_SCHEMA_VERSION {
        return Err(Error::ProfileError(format!(
            "Profile schema version {} is newer than supported version {}",
            profile.schema_version, PROFILE_SCHEMA_VERSION
        )));
    }

    // If schema version matches, no migration needed
    if profile.schema_version == PROFILE_SCHEMA_VERSION {
        return Ok(notes);
    }

    // Migration loop (v0 -> v1, v1 -> v2, etc.)
    // Note: Currently we only have v1, so this is forward-looking structure
    while profile.schema_version < PROFILE_SCHEMA_VERSION {
        match profile.schema_version {
            0 => {
                // Hypothetical v0 to v1 migration
                // For now, we just bump the version as v1 is the initial stable version
                // In real scenario, we would transform fields here
                profile.schema_version = 1;
                notes.push(MigrationNote {
                    from_version: 0,
                    to_version: 1,
                    message: "Upgraded profile from v0 to v1".to_string(),
                });
            }
            v => {
                // This shouldn't happen if loop condition is correct and we handle all versions
                return Err(Error::ProfileError(format!(
                    "Unsupported profile schema version: {}",
                    v
                )));
            }
        }
    }

    Ok(notes)
}
