use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{TweakDefinition, TweakResult, TweakStatus};
use crate::services::{registry_service, system_info_service, tweak_loader};
use tauri::AppHandle;

/// Get all available tweaks filtered by current Windows version
#[tauri::command]
pub async fn get_available_tweaks() -> Result<Vec<TweakDefinition>> {
    let windows_info = system_info_service::get_windows_info()?;
    let tweaks = tweak_loader::get_tweaks_for_version(&windows_info.version_string)?;
    Ok(tweaks.into_values().collect())
}

/// Get all available tweaks filtered by specified Windows version
#[tauri::command]
pub async fn get_available_tweaks_for_version(version: String) -> Result<Vec<TweakDefinition>> {
    let tweaks = tweak_loader::get_tweaks_for_version(&version)?;
    Ok(tweaks.into_values().collect())
}

/// Get tweaks by category
#[tauri::command]
pub async fn get_tweaks_by_category(category: String) -> Result<Vec<TweakDefinition>> {
    let windows_info = system_info_service::get_windows_info()?;
    let mut category_tweaks = tweak_loader::get_tweaks_by_category(&category)?;

    // Filter by Windows version
    category_tweaks.retain(|_, tweak| tweak.applies_to_version(&windows_info.version_string));

    Ok(category_tweaks.into_values().collect())
}

/// Get a specific tweak by ID
#[tauri::command]
pub async fn get_tweak(tweak_id: String) -> Result<Option<TweakDefinition>> {
    let tweak = tweak_loader::get_tweak(&tweak_id)?;
    Ok(tweak)
}

/// Get status of a specific tweak (applied or not)
#[tauri::command]
pub async fn get_tweak_status(tweak_id: String) -> Result<TweakStatus> {
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;

    let windows_info = system_info_service::get_windows_info()?;

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(&windows_info.version_string) {
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check registry to see if tweak is applied
    let changes = tweak
        .get_changes_for_version(&windows_info.version_string)
        .ok_or_else(|| Error::UnsupportedWindowsVersion)?;

    let is_applied = check_tweak_applied(changes)?;

    Ok(TweakStatus {
        tweak_id,
        is_applied,
        last_applied: None, // TODO: implement tracking
        has_backup: false,  // TODO: check if backup exists
    })
}

/// Apply a tweak (set enable values in registry)
#[tauri::command]
pub async fn apply_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;

    let system_info = system_info_service::get_system_info()?;

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(&system_info.windows.version_string) {
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check if admin required and not running as admin
    if tweak.requires_admin && !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak
        .get_changes_for_version(&system_info.windows.version_string)
        .ok_or_else(|| Error::UnsupportedWindowsVersion)?;

    // Debug: Log tweak application start
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Applying tweak: {} ({})", tweak.name, tweak_id),
            Some(&format!("{} registry changes to apply", changes.len())),
        );
    }

    // TODO: Create backup before applying

    // Apply all registry changes
    for change in changes {
        apply_registry_change(&app, change, &tweak.name)?;
    }

    // Debug: Log success
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Success,
            &format!("Successfully applied: {}", tweak.name),
            if tweak.requires_reboot {
                Some("Reboot required for changes to take effect")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Successfully applied: {}", tweak.name),
        requires_reboot: tweak.requires_reboot,
    })
}

/// Revert a tweak (restore to disable values or from backup)
#[tauri::command]
pub async fn revert_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;

    let system_info = system_info_service::get_system_info()?;

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(&system_info.windows.version_string) {
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check if admin required and not running as admin
    if tweak.requires_admin && !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak
        .get_changes_for_version(&system_info.windows.version_string)
        .ok_or_else(|| Error::UnsupportedWindowsVersion)?;

    // Debug: Log tweak revert start
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Reverting tweak: {} ({})", tweak.name, tweak_id),
            Some(&format!("{} registry changes to revert", changes.len())),
        );
    }

    // Restore all registry changes
    for change in changes {
        if let Some(disable_value) = &change.disable_value {
            revert_registry_change(&app, change, disable_value, &tweak.name)?;
        }
    }

    // Debug: Log success
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Success,
            &format!("Successfully reverted: {}", tweak.name),
            if tweak.requires_reboot {
                Some("Reboot required for changes to take effect")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Successfully reverted: {}", tweak.name),
        requires_reboot: tweak.requires_reboot,
    })
}

/// Apply multiple tweaks at once
#[tauri::command]
pub async fn batch_apply_tweaks(app: AppHandle, tweak_ids: Vec<String>) -> Result<TweakResult> {
    let system_info = system_info_service::get_system_info()?;

    if !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }

    // Debug: Log batch apply start
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Batch applying {} tweaks", tweak_ids.len()),
            Some(&tweak_ids.join(", ")),
        );
    }

    // TODO: Create single backup point
    let mut requires_reboot = false;

    // Apply all tweaks
    for tweak_id in &tweak_ids {
        let result = apply_tweak(app.clone(), tweak_id.clone()).await?;
        if result.requires_reboot {
            requires_reboot = true;
        }
    }

    // Debug: Log batch success
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Success,
            &format!("Batch apply completed: {} tweaks", tweak_ids.len()),
            if requires_reboot {
                Some("Reboot required")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Successfully applied {} tweaks", tweak_ids.len()),
        requires_reboot,
    })
}

/// Check if a tweak is currently applied by reading registry values
fn check_tweak_applied(changes: &[crate::models::RegistryChange]) -> Result<bool> {
    if changes.is_empty() {
        return Ok(false);
    }

    // Check first registry change to determine if applied
    let first_change = &changes[0];

    let current_value = match first_change.value_type {
        crate::models::RegistryValueType::DWord => registry_service::read_dword(
            &first_change.hive,
            &first_change.key,
            &first_change.value_name,
        )?
        .map(|v| serde_json::json!(v)),
        crate::models::RegistryValueType::String => registry_service::read_string(
            &first_change.hive,
            &first_change.key,
            &first_change.value_name,
        )?
        .map(|v| serde_json::json!(v)),
        crate::models::RegistryValueType::Binary => registry_service::read_binary(
            &first_change.hive,
            &first_change.key,
            &first_change.value_name,
        )?
        .map(|v| serde_json::json!(v)),
        _ => None,
    };

    if let Some(current) = current_value {
        Ok(current == first_change.enable_value)
    } else {
        Ok(false)
    }
}

/// Apply a single registry change
fn apply_registry_change(
    app: &AppHandle,
    change: &crate::models::RegistryChange,
    tweak_name: &str,
) -> Result<()> {
    let hive_name = match change.hive {
        crate::models::RegistryHive::HKCU => "HKCU",
        crate::models::RegistryHive::HKLM => "HKLM",
    };
    let full_path = format!("{}\\{}\\{}", hive_name, change.key, change.value_name);

    match change.value_type {
        crate::models::RegistryValueType::DWord => {
            if let Some(value) = change.enable_value.as_u64() {
                // Debug: Log registry change
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("Setting DWORD: {} = {}", full_path, value),
                        Some(tweak_name),
                    );
                }
                registry_service::set_dword(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    value as u32,
                )?;
            }
        }
        crate::models::RegistryValueType::String => {
            if let Some(value) = change.enable_value.as_str() {
                // Debug: Log registry change
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("Setting String: {} = \"{}\"", full_path, value),
                        Some(tweak_name),
                    );
                }
                registry_service::set_string(&change.hive, &change.key, &change.value_name, value)?;
            }
        }
        crate::models::RegistryValueType::Binary => {
            if let Some(value) = change.enable_value.as_array() {
                let binary: Vec<u8> = value
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                // Debug: Log registry change
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("Setting Binary: {} ({} bytes)", full_path, binary.len()),
                        Some(tweak_name),
                    );
                }
                registry_service::set_binary(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    &binary,
                )?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Revert a registry change to disable value
fn revert_registry_change(
    app: &AppHandle,
    change: &crate::models::RegistryChange,
    disable_value: &serde_json::Value,
    tweak_name: &str,
) -> Result<()> {
    let hive_name = match change.hive {
        crate::models::RegistryHive::HKCU => "HKCU",
        crate::models::RegistryHive::HKLM => "HKLM",
    };
    let full_path = format!("{}\\{}\\{}", hive_name, change.key, change.value_name);

    match change.value_type {
        crate::models::RegistryValueType::DWord => {
            if let Some(value) = disable_value.as_u64() {
                // Debug: Log registry change
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("Reverting DWORD: {} = {}", full_path, value),
                        Some(tweak_name),
                    );
                }
                registry_service::set_dword(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    value as u32,
                )?;
            }
        }
        crate::models::RegistryValueType::String => {
            if let Some(value) = disable_value.as_str() {
                // Debug: Log registry change
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("Reverting String: {} = \"{}\"", full_path, value),
                        Some(tweak_name),
                    );
                }
                registry_service::set_string(&change.hive, &change.key, &change.value_name, value)?;
            }
        }
        crate::models::RegistryValueType::Binary => {
            if let Some(value) = disable_value.as_array() {
                let binary: Vec<u8> = value
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                // Debug: Log registry change
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("Reverting Binary: {} ({} bytes)", full_path, binary.len()),
                        Some(tweak_name),
                    );
                }
                registry_service::set_binary(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    &binary,
                )?;
            }
        }
        _ => {}
    }

    Ok(())
}
