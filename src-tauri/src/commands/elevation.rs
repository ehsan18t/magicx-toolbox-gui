//! Elevation Commands
//!
//! Commands for SYSTEM elevation to modify protected registry keys.

use crate::error::Result;
use crate::services::trusted_installer;

/// Apply a registry change as SYSTEM
#[tauri::command]
pub async fn apply_registry_as_system(
    hive: String,
    key: String,
    value_name: String,
    value_type: String,
    value_data: String,
) -> Result<()> {
    log::info!(
        "Command: apply_registry_as_system - {}\\{}\\{}",
        hive,
        key,
        value_name
    );

    trusted_installer::set_registry_value_as_system(
        &hive,
        &key,
        &value_name,
        &value_type,
        &value_data,
    )
}

/// Delete a registry value as SYSTEM
#[tauri::command]
pub async fn delete_registry_as_system(
    hive: String,
    key: String,
    value_name: String,
) -> Result<()> {
    log::info!(
        "Command: delete_registry_as_system - {}\\{}\\{}",
        hive,
        key,
        value_name
    );

    trusted_installer::delete_registry_value_as_system(&hive, &key, &value_name)
}

/// Check if SYSTEM elevation is available
#[tauri::command]
pub async fn can_use_system_elevation() -> Result<bool> {
    Ok(trusted_installer::can_use_system_elevation())
}
