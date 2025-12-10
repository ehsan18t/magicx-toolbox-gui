use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{
    make_key_id, CategoryDefinition, RegistryKeyId, TweakDefinition, TweakResult, TweakStatus,
};
use crate::services::{backup_service, registry_service, system_info_service, tweak_loader};
use tauri::AppHandle;

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

/// Get status of a specific tweak (applied or not)
#[tauri::command]
pub async fn get_tweak_status(tweak_id: String) -> Result<TweakStatus> {
    log::trace!("Command: get_tweak_status({})", tweak_id);
    let tweak = tweak_loader::get_tweak(&tweak_id)?
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;

    let windows_info = system_info_service::get_windows_info()?;
    let version = windows_info.version_number();

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(version) {
        log::debug!("Tweak {} not applicable to Windows {}", tweak_id, version);
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check registry to see if tweak is applied
    let changes = tweak.get_changes_for_version(version);
    if changes.is_empty() {
        log::debug!(
            "Tweak {} has no registry changes for Windows {}",
            tweak_id,
            version
        );
        return Err(Error::UnsupportedWindowsVersion);
    }

    let is_applied = check_tweak_applied(&changes)?;

    // Check our state tracking instead of just backup existence
    let has_backup = backup_service::is_tweak_applied(&tweak_id).unwrap_or(false);

    // Get last applied timestamp if available
    let last_applied = backup_service::load_tweak_state().ok().and_then(|state| {
        state
            .get_applied_tweak(&tweak_id)
            .map(|info| info.applied_at.clone())
    });

    log::trace!(
        "Tweak {} status: applied={}, has_backup={}, last_applied={:?}",
        tweak_id,
        is_applied,
        has_backup,
        last_applied
    );

    Ok(TweakStatus {
        tweak_id,
        is_applied,
        last_applied,
        has_backup,
    })
}

/// Apply a tweak (set enable values in registry)
#[tauri::command]
pub async fn apply_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: apply_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    let system_info = system_info_service::get_system_info()?;
    let version = system_info.windows.version_number();
    log::debug!("Applying '{}' on Windows {}", tweak.name, version);

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(version) {
        log::warn!(
            "Tweak '{}' not supported on Windows {}",
            tweak.name,
            version
        );
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check if admin required and not running as admin
    if tweak.requires_admin && !system_info.is_admin {
        log::warn!("Tweak '{}' requires admin privileges", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak.get_changes_for_version(version);
    if changes.is_empty() {
        log::warn!(
            "Tweak '{}' has no changes for Windows {}",
            tweak.name,
            version
        );
        return Err(Error::UnsupportedWindowsVersion);
    }

    log::debug!(
        "Tweak '{}' has {} registry changes to apply",
        tweak.name,
        changes.len()
    );

    // Debug: Log tweak application start
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Applying tweak: {} ({})", tweak.name, tweak_id),
            Some(&format!("{} registry changes to apply", changes.len())),
        );
    }

    // Prepare key info for baseline capture
    let keys_info: Vec<(String, String, String, String)> = changes
        .iter()
        .map(|c| {
            (
                c.hive.as_str().to_string(),
                c.key.clone(),
                c.value_name.clone(),
                format!("{:?}", c.value_type).replace("\"", ""),
            )
        })
        .collect();

    // Run preflight check to detect conflicts
    let preflight = backup_service::preflight_check(&tweak_id, &keys_info)?;

    if !preflight.conflicts.is_empty() {
        log::warn!(
            "Tweak '{}' has {} conflicts with other applied tweaks",
            tweak.name,
            preflight.conflicts.len()
        );
        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Warn,
                &format!(
                    "Tweak '{}' shares registry keys with: {:?}",
                    tweak.name,
                    preflight
                        .conflicts
                        .iter()
                        .flat_map(|c| c.conflicting_tweaks.clone())
                        .collect::<Vec<_>>()
                ),
                Some("Revert order matters for shared keys"),
            );
        }
    }

    // Capture baseline values for any keys not already captured
    log::debug!("Capturing baseline for tweak '{}'", tweak.name);
    if let Err(e) = backup_service::capture_baseline_for_keys(&tweak_id, &keys_info) {
        log::warn!("Failed to capture baseline for '{}': {}", tweak.name, e);
        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Warn,
                &format!("Failed to capture baseline for {}: {}", tweak.name, e),
                Some("Continuing without full baseline"),
            );
        }
    } else {
        log::debug!("Baseline captured for tweak '{}'", tweak.name);
        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Success,
                &format!("Baseline captured for: {}", tweak.name),
                None,
            );
        }
    }

    // Apply all registry changes with rollback on failure
    let mut applied_changes: Vec<RegistryKeyId> = Vec::new();
    let mut rollback_needed = false;
    let mut error_msg: Option<String> = None;

    for change in &changes {
        let key_id = make_key_id(change.hive.as_str(), &change.key, &change.value_name);

        match apply_registry_change(&app, change, &tweak.name) {
            Ok(()) => {
                applied_changes.push(key_id);
            }
            Err(e) => {
                log::error!(
                    "Failed to apply registry change for '{}': {}",
                    tweak.name,
                    e
                );
                rollback_needed = true;
                error_msg = Some(e.to_string());
                break;
            }
        }
    }

    // If any change failed, rollback all applied changes
    if rollback_needed {
        log::warn!(
            "Rolling back {} changes for tweak '{}'",
            applied_changes.len(),
            tweak.name
        );

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Warn,
                &format!("Rolling back changes for: {}", tweak.name),
                Some(&format!("{} changes to revert", applied_changes.len())),
            );
        }

        // Try to restore each applied key to its baseline value
        for key_id in &applied_changes {
            if let Err(e) = backup_service::restore_key_to_baseline(key_id) {
                log::error!("Failed to rollback key {}: {}", key_id, e);
            }
        }

        return Err(Error::BackupFailed(format!(
            "Failed to apply tweak '{}': {}. Changes have been rolled back.",
            tweak.name,
            error_msg.unwrap_or_default()
        )));
    }

    // Record the tweak as applied in our state
    if let Err(e) =
        backup_service::record_tweak_applied(&tweak_id, &tweak.name, version, applied_changes)
    {
        log::warn!("Failed to record tweak state for '{}': {}", tweak.name, e);
    }

    log::info!(
        "Successfully applied tweak '{}'{}",
        tweak.name,
        if tweak.requires_reboot {
            " (reboot required)"
        } else {
            ""
        }
    );

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

/// Revert a tweak (restore to baseline or disable values with reference counting)
#[tauri::command]
pub async fn revert_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: revert_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    let system_info = system_info_service::get_system_info()?;
    let version = system_info.windows.version_number();
    log::debug!("Reverting '{}' on Windows {}", tweak.name, version);

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(version) {
        log::warn!(
            "Tweak '{}' not supported on Windows {}",
            tweak.name,
            version
        );
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check if admin required and not running as admin
    if tweak.requires_admin && !system_info.is_admin {
        log::warn!("Tweak '{}' requires admin privileges", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak.get_changes_for_version(version);
    if changes.is_empty() {
        log::warn!(
            "Tweak '{}' has no changes for Windows {}",
            tweak.name,
            version
        );
        return Err(Error::UnsupportedWindowsVersion);
    }

    log::debug!(
        "Tweak '{}' has {} registry changes to revert",
        tweak.name,
        changes.len()
    );

    // Debug: Log tweak revert start
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Reverting tweak: {} ({})", tweak.name, tweak_id),
            Some(&format!("{} registry changes to revert", changes.len())),
        );
    }

    // First, record the revert to get list of orphaned keys (ref_count goes to 0)
    let orphaned_keys = backup_service::record_tweak_reverted(&tweak_id)?;
    log::debug!(
        "Tweak '{}' revert: {} keys orphaned (will be restored to baseline)",
        tweak.name,
        orphaned_keys.len()
    );

    // For each registry change:
    // - If key is orphaned (no other tweaks use it): restore to baseline
    // - If key is still used by other tweaks: apply disable_value (or skip if none)
    for change in &changes {
        let key_id = make_key_id(change.hive.as_str(), &change.key, &change.value_name);

        if orphaned_keys.contains(&key_id) {
            // This key is no longer used by any tweak - restore to baseline
            log::debug!("Restoring key {} to baseline (orphaned)", key_id);

            if is_debug_enabled() {
                emit_debug_log(
                    &app,
                    DebugLevel::Info,
                    &format!(
                        "Restoring to baseline: {}\\{}\\{}",
                        change.hive.as_str(),
                        change.key,
                        change.value_name
                    ),
                    Some("No other tweaks use this key"),
                );
            }

            match backup_service::restore_key_to_baseline(&key_id) {
                Ok(restored) => {
                    if restored {
                        log::debug!("Restored {} to baseline", key_id);
                    } else {
                        log::debug!(
                            "Could not restore {} to baseline (no baseline or no original value)",
                            key_id
                        );
                        // Fallback to disable_value if available
                        if let Some(disable_value) = &change.disable_value {
                            revert_registry_change(&app, change, disable_value, &tweak.name)?;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to restore {} to baseline: {}", key_id, e);
                    // Fallback to disable_value if available
                    if let Some(disable_value) = &change.disable_value {
                        revert_registry_change(&app, change, disable_value, &tweak.name)?;
                    }
                }
            }
        } else {
            // Other tweaks still use this key - apply disable_value only
            // This keeps the key modified but moves it toward a "less aggressive" state
            if let Some(disable_value) = &change.disable_value {
                log::debug!(
                    "Key {} still used by other tweaks, applying disable_value",
                    key_id
                );

                if is_debug_enabled() {
                    let other_tweaks = backup_service::load_tweak_state()
                        .map(|s| s.get_tweaks_for_key(&key_id))
                        .unwrap_or_default();
                    emit_debug_log(
                        &app,
                        DebugLevel::Warn,
                        &format!("Key {} still used by other tweaks", key_id),
                        Some(&format!(
                            "Used by: {:?}. Applying disable value instead of baseline.",
                            other_tweaks
                        )),
                    );
                }

                revert_registry_change(&app, change, disable_value, &tweak.name)?;
            } else {
                log::debug!(
                    "Key {} has no disable_value, skipping (other tweaks still use it)",
                    key_id
                );
            }
        }
    }

    log::info!(
        "Successfully reverted tweak '{}'{}",
        tweak.name,
        if tweak.requires_reboot {
            " (reboot required)"
        } else {
            ""
        }
    );

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
    log::info!("Command: batch_apply_tweaks({} tweaks)", tweak_ids.len());
    log::debug!("Batch tweak IDs: {:?}", tweak_ids);

    let system_info = system_info_service::get_system_info()?;

    if !system_info.is_admin {
        log::warn!("Batch apply requires admin privileges");
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

    // Each tweak creates its own backup via apply_tweak, allowing individual restore
    let mut requires_reboot = false;

    // Apply all tweaks
    for tweak_id in &tweak_ids {
        let result = apply_tweak(app.clone(), tweak_id.clone()).await?;
        if result.requires_reboot {
            requires_reboot = true;
        }
    }

    log::info!(
        "Batch apply completed: {} tweaks{}",
        tweak_ids.len(),
        if requires_reboot {
            " (reboot required)"
        } else {
            ""
        }
    );

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
fn check_tweak_applied(changes: &[&crate::models::RegistryChange]) -> Result<bool> {
    if changes.is_empty() {
        return Ok(false);
    }

    // Check first registry change to determine if applied
    let first_change = changes[0];

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
