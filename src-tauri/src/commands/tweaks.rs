use crate::error::{Error, Result};
use crate::models::{TweakDefinition, TweakStatus};
use crate::services::{tweak_loader, system_info_service, registry_service};
use std::collections::HashMap;

/// Get all available tweaks filtered by Windows version
#[tauri::command]
pub async fn get_available_tweaks() -> Result<HashMap<String, TweakDefinition>> {
    let system_info = system_info_service::get_windows_info()?;
    let tweaks = tweak_loader::get_tweaks_for_version(&system_info.version)?;
    Ok(tweaks)
}

/// Get tweaks by category
#[tauri::command]
pub async fn get_tweaks_by_category(category: String) -> Result<HashMap<String, TweakDefinition>> {
    let system_info = system_info_service::get_windows_info()?;
    let mut category_tweaks = tweak_loader::get_tweaks_by_category(&category)?;
    
    // Filter by Windows version
    category_tweaks.retain(|_, tweak| tweak.applies_to_version(&system_info.version));
    
    Ok(category_tweaks)
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
    
    let system_info = system_info_service::get_windows_info()?;
    
    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(&system_info.version) {
        return Err(Error::UnsupportedWindowsVersion);
    }
    
    // Check registry to see if tweak is applied
    let changes = tweak.get_changes_for_version(&system_info.version)
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
pub async fn apply_tweak(tweak_id: String) -> Result<()> {
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;
    
    let system_info = system_info_service::get_windows_info()?;
    
    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(&system_info.version) {
        return Err(Error::UnsupportedWindowsVersion);
    }
    
    // Check if admin required and not running as admin
    if tweak.requires_admin && !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }
    
    let changes = tweak.get_changes_for_version(&system_info.version)
        .ok_or_else(|| Error::UnsupportedWindowsVersion)?;
    
    // TODO: Create backup before applying
    
    // Apply all registry changes
    for change in changes {
        apply_registry_change(change)?;
    }
    
    Ok(())
}

/// Revert a tweak (restore to disable values or from backup)
#[tauri::command]
pub async fn revert_tweak(tweak_id: String) -> Result<()> {
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;
    
    let system_info = system_info_service::get_windows_info()?;
    
    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(&system_info.version) {
        return Err(Error::UnsupportedWindowsVersion);
    }
    
    // Check if admin required and not running as admin
    if tweak.requires_admin && !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }
    
    let changes = tweak.get_changes_for_version(&system_info.version)
        .ok_or_else(|| Error::UnsupportedWindowsVersion)?;
    
    // Restore all registry changes
    for change in changes {
        if let Some(disable_value) = &change.disable_value {
            revert_registry_change(change, disable_value)?;
        }
    }
    
    Ok(())
}

/// Apply multiple tweaks at once
#[tauri::command]
pub async fn batch_apply_tweaks(tweak_ids: Vec<String>) -> Result<()> {
    let system_info = system_info_service::get_windows_info()?;
    
    if !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }
    
    // TODO: Create single backup point
    
    // Apply all tweaks
    for tweak_id in tweak_ids {
        apply_tweak(tweak_id).await?;
    }
    
    Ok(())
}

/// Check if a tweak is currently applied by reading registry values
fn check_tweak_applied(changes: &[crate::models::RegistryChange]) -> Result<bool> {
    if changes.is_empty() {
        return Ok(false);
    }
    
    // Check first registry change to determine if applied
    let first_change = &changes[0];
    
    let current_value = match first_change.value_type {
        crate::models::RegistryValueType::DWord => {
            registry_service::read_dword(&first_change.hive, &first_change.key, &first_change.value_name)?
                .map(|v| serde_json::json!(v))
        }
        crate::models::RegistryValueType::String => {
            registry_service::read_string(&first_change.hive, &first_change.key, &first_change.value_name)?
                .map(|v| serde_json::json!(v))
        }
        crate::models::RegistryValueType::Binary => {
            registry_service::read_binary(&first_change.hive, &first_change.key, &first_change.value_name)?
                .map(|v| serde_json::json!(v))
        }
        _ => None,
    };
    
    if let Some(current) = current_value {
        Ok(current == first_change.enable_value)
    } else {
        Ok(false)
    }
}

/// Apply a single registry change
fn apply_registry_change(change: &crate::models::RegistryChange) -> Result<()> {
    match change.value_type {
        crate::models::RegistryValueType::DWord => {
            if let Some(value) = change.enable_value.as_u64() {
                registry_service::set_dword(&change.hive, &change.key, &change.value_name, value as u32)?;
            }
        }
        crate::models::RegistryValueType::String => {
            if let Some(value) = change.enable_value.as_str() {
                registry_service::set_string(&change.hive, &change.key, &change.value_name, value)?;
            }
        }
        crate::models::RegistryValueType::Binary => {
            if let Some(value) = change.enable_value.as_array() {
                let binary: Vec<u8> = value.iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                registry_service::set_binary(&change.hive, &change.key, &change.value_name, &binary)?;
            }
        }
        _ => {}
    }
    
    Ok(())
}

/// Revert a registry change to disable value
fn revert_registry_change(change: &crate::models::RegistryChange, disable_value: &serde_json::Value) -> Result<()> {
    match change.value_type {
        crate::models::RegistryValueType::DWord => {
            if let Some(value) = disable_value.as_u64() {
                registry_service::set_dword(&change.hive, &change.key, &change.value_name, value as u32)?;
            }
        }
        crate::models::RegistryValueType::String => {
            if let Some(value) = disable_value.as_str() {
                registry_service::set_string(&change.hive, &change.key, &change.value_name, value)?;
            }
        }
        crate::models::RegistryValueType::Binary => {
            if let Some(value) = disable_value.as_array() {
                let binary: Vec<u8> = value.iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                registry_service::set_binary(&change.hive, &change.key, &change.value_name, &binary)?;
            }
        }
        _ => {}
    }
    
    Ok(())
}
