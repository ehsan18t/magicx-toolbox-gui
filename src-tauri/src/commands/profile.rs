//! Profile Commands
//!
//! Tauri commands for the profile export/import system.

use std::path::PathBuf;
use tauri::Manager;

use crate::error::Error;
use crate::models::{
    ApplyOptions, ConfigurationProfile, ExportOptions, ProfileApplyResult, ProfileValidation,
    TweakDefinition,
};
use crate::services::profile::{apply_profile, export_profile, import_profile, validate_profile};
use crate::services::{system_info_service, tweak_loader};

/// Get the profile directory for storing/reading profiles.
/// Returns an error if the app data directory cannot be determined.
pub fn get_profile_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    app_handle
        .path()
        .app_data_dir()
        .map(|p| p.join("profiles"))
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

#[tauri::command]
pub async fn get_saved_profiles(
    app_handle: tauri::AppHandle,
    custom_path: Option<String>,
) -> Result<Vec<crate::models::ProfileMetadata>, String> {
    let profile_dir = if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        get_profile_dir(&app_handle)?
    };

    if !profile_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();

    let entries = std::fs::read_dir(profile_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("mgx") {
            // Try to read metadata from the archive
            match crate::services::profile::archive::read_profile_archive(&path) {
                Ok(contents) => {
                    profiles.push(contents.profile.metadata);
                }
                Err(e) => {
                    // Log but continue - don't fail the entire list for one corrupted file
                    log::warn!("Failed to read profile '{}': {}", path.display(), e);
                }
            }
        }
    }

    // Sort by date created (descending)
    profiles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(profiles)
}

#[tauri::command]
pub async fn delete_saved_profile(
    app_handle: tauri::AppHandle,
    name: String,
    custom_path: Option<String>,
) -> Result<(), String> {
    let profile_dir = if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        get_profile_dir(&app_handle)?
    };

    // Sanitize filename to prevent directory traversal
    let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "");
    let filename = format!("{}.mgx", safe_name);
    let path = profile_dir.join(filename);

    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn profile_export(
    _app_handle: tauri::AppHandle,
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
