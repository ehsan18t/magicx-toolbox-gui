//! Profile Import/Apply
//!
//! Import and apply configuration profiles.

use std::path::Path;

use crate::error::Error;
use crate::models::{ConfigurationProfile, ProfileValidation, TweakDefinition};
use crate::services::profile::archive::read_profile_archive;
use crate::services::profile::validation::validate_profile;

/// Import and validate a profile from a file.
pub fn import_profile(
    path: &Path,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
) -> Result<(ConfigurationProfile, ProfileValidation), Error> {
    log::info!("Importing profile from {}", path.display());

    // Read the archive
    let mut contents = read_profile_archive(path)?;

    // Migrate if needed
    let migration_notes =
        crate::services::profile::migration::migrate_profile(&mut contents.profile)?;

    if !migration_notes.is_empty() {
        log::info!(
            "Profile migrated with {} notes: {:?}",
            migration_notes.len(),
            migration_notes
        );
    }

    // Validate
    let validation = validate_profile(&contents.profile, available_tweaks, windows_version)?;

    Ok((contents.profile, validation))
}
