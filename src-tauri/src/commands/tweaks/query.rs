//! Query Commands - Status and listing operations for tweaks

use crate::error::Result;
use crate::models::{CategoryDefinition, TweakDefinition, TweakStatus};
use crate::services::{backup_service, system_info_service, tweak_loader};
use rayon::prelude::*;

/// Get all available categories (auto-discovered from YAML files)
#[tauri::command]
pub async fn get_categories() -> Result<Vec<CategoryDefinition>> {
    log::debug!("Command: get_categories");
    let categories = tweak_loader::load_all_categories()?;
    log::debug!("Returning {} categories", categories.len());
    Ok(categories)
}

/// Get all available tweaks filtered by current Windows version
#[tauri::command]
pub async fn get_available_tweaks() -> Result<Vec<TweakDefinition>> {
    log::debug!("Command: get_available_tweaks");
    let windows_info = system_info_service::get_windows_info()?;
    let version = windows_info.version_number();
    log::debug!("Windows version detected: {}", version);

    let tweaks = tweak_loader::get_tweaks_for_version(version)?;
    log::debug!("Returning {} tweaks for Windows {}", tweaks.len(), version);
    Ok(tweaks.into_values().collect())
}

/// Get all available tweaks filtered by specified Windows version
#[tauri::command]
pub async fn get_available_tweaks_for_version(version: u32) -> Result<Vec<TweakDefinition>> {
    log::debug!("Command: get_available_tweaks_for_version({})", version);
    let tweaks = tweak_loader::get_tweaks_for_version(version)?;
    log::debug!("Returning {} tweaks for Windows {}", tweaks.len(), version);
    Ok(tweaks.into_values().collect())
}

/// Get tweaks by category
#[tauri::command]
pub async fn get_tweaks_by_category(category: String) -> Result<Vec<TweakDefinition>> {
    log::debug!("Command: get_tweaks_by_category({})", category);
    let windows_info = system_info_service::get_windows_info()?;
    let mut category_tweaks = tweak_loader::get_tweaks_by_category(&category)?;

    // Filter by Windows version
    let version = windows_info.version_number();
    category_tweaks.retain(|_, tweak| tweak.applies_to_version(version));
    log::debug!(
        "Returning {} tweaks in category '{}'",
        category_tweaks.len(),
        category
    );

    Ok(category_tweaks.into_values().collect())
}

/// Get a specific tweak by ID
#[tauri::command]
pub async fn get_tweak(tweak_id: String) -> Result<Option<TweakDefinition>> {
    log::debug!("Command: get_tweak({})", tweak_id);
    let tweak = tweak_loader::get_tweak(&tweak_id)?;
    if tweak.is_some() {
        log::trace!("Found tweak: {}", tweak_id);
    } else {
        log::debug!("Tweak not found: {}", tweak_id);
    }
    Ok(tweak)
}

/// Get status of a specific tweak
/// Returns current_option_index = None if system state doesn't match any defined option
#[tauri::command]
pub async fn get_tweak_status(tweak_id: String) -> Result<TweakStatus> {
    log::trace!("Command: get_tweak_status({})", tweak_id);
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| crate::error::Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;

    let windows_info = system_info_service::get_windows_info()?;
    let version = windows_info.version_number();

    // Detect current state by matching against all options
    let state = backup_service::detect_tweak_state(&tweak, version)?;

    // Get last applied timestamp from snapshot if exists
    let last_applied = backup_service::load_snapshot(&tweak_id)?.map(|s| s.created_at);

    log::trace!(
        "Tweak {} status: current_option={:?}, has_snapshot={}",
        tweak_id,
        state.current_option_index,
        state.has_snapshot
    );

    Ok(TweakStatus {
        tweak_id,
        is_applied: state.current_option_index == Some(0),
        last_applied,
        has_backup: state.has_snapshot,
        current_option_index: state.current_option_index,
        error: None,
    })
}

/// Get status of all tweaks (parallelized for performance)
#[tauri::command]
pub async fn get_all_tweak_statuses() -> Result<Vec<TweakStatus>> {
    log::debug!("Command: get_all_tweak_statuses");
    let windows_info = system_info_service::get_windows_info()?;
    let version = windows_info.version_number();

    let tweaks = tweak_loader::get_tweaks_for_version(version)?;

    // Use rayon parallel iterator for concurrent status detection
    // This is a CPU-bound + IO-bound task that benefits from parallelization
    let statuses: Vec<TweakStatus> = tweaks
        .into_par_iter()
        .map(|(id, tweak)| {
            match backup_service::detect_tweak_state(&tweak, version) {
                Ok(state) => {
                    let last_applied = backup_service::load_snapshot(&id)
                        .ok()
                        .flatten()
                        .map(|s| s.created_at);

                    TweakStatus {
                        tweak_id: id,
                        is_applied: state.current_option_index == Some(0),
                        last_applied,
                        has_backup: state.has_snapshot,
                        current_option_index: state.current_option_index,
                        error: None,
                    }
                }
                Err(e) => {
                    log::warn!("Failed to detect state for tweak {}: {}", id, e);
                    // Return tweak with error state instead of dropping it
                    // This ensures frontend sees all tweaks and can show error indicator
                    TweakStatus {
                        tweak_id: id,
                        is_applied: false,
                        last_applied: None,
                        has_backup: false,
                        current_option_index: None,
                        error: Some(format!("State detection failed: {}", e)),
                    }
                }
            }
        })
        .collect();

    log::debug!("Returning {} tweak statuses", statuses.len());
    Ok(statuses)
}
