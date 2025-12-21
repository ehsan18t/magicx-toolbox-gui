//! Profile Commands
//!
//! Tauri commands for the profile export/import system.

use std::path::PathBuf;

use crate::error::Error;
use crate::models::{
    ApplyOptions, ConfigurationProfile, ExportOptions, ProfileApplyResult, ProfileValidation,
    TweakDefinition,
};
use crate::services::profile::{apply_profile, export_profile, import_profile, validate_profile};
use crate::services::{system_info_service, tweak_loader};

/// Export a configuration profile to a file.
#[tauri::command]
pub fn profile_export(
    file_path: String,
    name: String,
    description: Option<String>,
    tweak_ids: Vec<String>,
    include_system_state: bool,
) -> Result<(), Error> {
    log::info!("Exporting profile '{}' to '{}'", name, file_path);

    let system_info = system_info_service::get_system_info()?;
    let windows_version = system_info.windows.version_number();
    let windows_build: u32 = system_info.windows.build_number.parse().unwrap_or(0);
    let tweaks = tweak_loader::get_tweaks_for_version(windows_version)?;
    let available_tweaks: Vec<TweakDefinition> = tweaks.values().cloned().collect();

    let options = ExportOptions {
        name,
        description,
        tweak_ids,
        include_system_state,
    };

    let path = PathBuf::from(file_path);
    export_profile(
        &path,
        &available_tweaks,
        &options,
        windows_version,
        windows_build,
    )?;

    Ok(())
}

/// Import and validate a profile from a file.
#[tauri::command]
pub fn profile_import(
    file_path: String,
) -> Result<(ConfigurationProfile, ProfileValidation), Error> {
    log::info!("Importing profile from '{}'", file_path);

    let system_info = system_info_service::get_system_info()?;
    let windows_version = system_info.windows.version_number();
    let tweaks = tweak_loader::get_tweaks_for_version(windows_version)?;
    let available_tweaks: Vec<TweakDefinition> = tweaks.values().cloned().collect();
    let path = PathBuf::from(file_path);

    import_profile(&path, &available_tweaks, windows_version)
}

/// Validate a profile against the current system.
#[tauri::command]
pub fn profile_validate(profile: ConfigurationProfile) -> Result<ProfileValidation, Error> {
    log::info!("Validating profile '{}'", profile.metadata.name);

    let system_info = system_info_service::get_system_info()?;
    let windows_version = system_info.windows.version_number();
    let tweaks = tweak_loader::get_tweaks_for_version(windows_version)?;
    let available_tweaks: Vec<TweakDefinition> = tweaks.values().cloned().collect();

    validate_profile(&profile, &available_tweaks, windows_version)
}

/// Apply a validated profile to the system.
#[tauri::command]
pub fn profile_apply(
    profile: ConfigurationProfile,
    skip_tweak_ids: Vec<String>,
    skip_already_applied: bool,
    create_restore_point: bool,
) -> Result<ProfileApplyResult, Error> {
    log::info!("Applying profile '{}'", profile.metadata.name);

    let system_info = system_info_service::get_system_info()?;
    let windows_version = system_info.windows.version_number();
    let tweaks = tweak_loader::get_tweaks_for_version(windows_version)?;
    let available_tweaks: Vec<TweakDefinition> = tweaks.values().cloned().collect();

    let options = ApplyOptions {
        skip_tweak_ids,
        skip_already_applied,
        create_restore_point,
    };

    apply_profile(&profile, &available_tweaks, windows_version, &options)
}

/// Get the current Windows version for the UI.
#[tauri::command]
pub fn get_windows_version() -> Result<u32, Error> {
    let system_info = system_info_service::get_system_info()?;
    Ok(system_info.windows.version_number())
}
