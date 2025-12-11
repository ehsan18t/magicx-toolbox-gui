use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{CategoryDefinition, TweakDefinition, TweakResult, TweakStatus};
use crate::services::{
    backup_service, registry_service, service_control, system_info_service, trusted_installer,
    tweak_loader,
};
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

    // Check if snapshot exists (means tweak is applied by us)
    let has_backup = backup_service::snapshot_exists(&tweak_id)?;

    // Detect current state from registry
    let (is_applied, current_option_index) = backup_service::detect_tweak_state(&tweak)?;

    // Get last applied timestamp from snapshot if exists
    let last_applied = backup_service::load_snapshot(&tweak_id)?.map(|s| s.applied_at);

    log::trace!(
        "Tweak {} status: applied={}, has_backup={}, option_index={:?}",
        tweak_id,
        is_applied,
        has_backup,
        current_option_index
    );

    Ok(TweakStatus {
        tweak_id,
        is_applied,
        last_applied,
        has_backup,
        current_option_index,
    })
}

/// Apply a tweak (set enable values in registry)
/// If a snapshot already exists (tweak is applied), this will TOGGLE it off (restore from snapshot)
#[tauri::command]
pub async fn apply_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: apply_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    let system_info = system_info_service::get_system_info()?;
    let version = system_info.windows.version_number();
    log::debug!("Processing '{}' on Windows {}", tweak.name, version);

    // Check admin if required
    if tweak.requires_admin && !system_info.is_admin {
        log::warn!("Tweak '{}' requires admin, but running as user", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    // Get version-specific changes
    let changes = tweak.get_changes_for_version(version);
    if changes.is_empty() {
        log::warn!(
            "No registry changes for Windows {} in tweak '{}'",
            version,
            tweak.name
        );
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check if snapshot exists - if so, this is a TOGGLE OFF operation
    if let Some(existing_snapshot) = backup_service::load_snapshot(&tweak_id)? {
        log::info!(
            "Snapshot exists for '{}' - toggling OFF (restoring from snapshot)",
            tweak.name
        );

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Info,
                &format!("Toggling OFF: {}", tweak.name),
                Some(&format!(
                    "Restoring {} registry values from snapshot",
                    existing_snapshot.registry_snapshots.len()
                )),
            );
        }

        // Restore registry values from snapshot
        if let Err(e) = backup_service::restore_from_snapshot(&existing_snapshot) {
            log::error!(
                "Failed to restore from snapshot for '{}': {}",
                tweak.name,
                e
            );

            // Delete the snapshot since restore failed - user likely lacks permissions
            // and keeping it would just show the tweak as "applied" when it's not
            let _ = backup_service::delete_snapshot(&tweak_id);

            return Err(e);
        }

        // Restore service states if any
        if let Some(ref service_changes) = tweak.service_changes {
            for sc in service_changes {
                log::info!("Restoring service: {} -> {:?}", sc.name, sc.disable_startup);

                // Restore startup type
                if let Err(e) = service_control::set_service_startup(&sc.name, &sc.disable_startup)
                {
                    log::warn!("Failed to restore service '{}' startup: {}", sc.name, e);
                }

                // Start service if required
                if sc.start_on_enable {
                    if let Err(e) = service_control::start_service(&sc.name) {
                        log::warn!("Failed to start service '{}': {}", sc.name, e);
                    }
                }
            }
        }

        // Delete snapshot after successful restore
        backup_service::delete_snapshot(&tweak_id)?;

        log::info!(
            "Successfully toggled OFF tweak '{}'{}",
            tweak.name,
            if tweak.requires_reboot {
                " (reboot required)"
            } else {
                ""
            }
        );

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Success,
                &format!("Successfully toggled OFF: {}", tweak.name),
                if tweak.requires_reboot {
                    Some("Reboot required")
                } else {
                    None
                },
            );
        }

        return Ok(TweakResult {
            success: true,
            message: format!("Successfully reverted: {}", tweak.name),
            requires_reboot: tweak.requires_reboot,
        });
    }

    // No snapshot exists - this is a regular APPLY operation
    log::info!("No snapshot for '{}' - applying tweak", tweak.name);

    // Step 1: Capture snapshot BEFORE making any changes
    let snapshot = backup_service::capture_snapshot(&tweak, version)?;
    backup_service::save_snapshot(&snapshot)?;
    log::info!(
        "Captured snapshot for '{}' with {} registry values",
        tweak.name,
        snapshot.registry_snapshots.len()
    );

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Captured snapshot for: {}", tweak.name),
            Some(&format!(
                "{} registry values",
                snapshot.registry_snapshots.len()
            )),
        );
    }

    // Step 2: Apply all registry changes atomically
    let mut applied_values: Vec<(crate::models::RegistrySnapshot, serde_json::Value)> = Vec::new();

    for change in &changes {
        let hive_str = change.hive.as_str();
        let full_path = format!("{}\\{}\\{}", hive_str, change.key, change.value_name);

        log::debug!("Setting {} = {:?}", full_path, change.enable_value);

        // Capture current value for potential rollback
        let current_snapshot = crate::models::RegistrySnapshot {
            hive: hive_str.to_string(),
            key: change.key.clone(),
            value_name: change.value_name.clone(),
            value_type: change.value_type.as_str().to_string(),
            value: read_current_value(change)?,
            existed: true, // We'll set the value, so it will exist
        };

        // Try to write the value
        if let Err(e) = write_registry_value(
            &app,
            change,
            &change.enable_value,
            "Setting",
            &tweak.name,
            tweak.requires_system,
        ) {
            log::error!("Failed to apply {}: {}", full_path, e);

            // Rollback all applied values
            log::warn!("Rolling back {} applied changes", applied_values.len());
            for (snap, _original) in applied_values.iter().rev() {
                let hive_enum = match snap.hive.as_str() {
                    "HKCU" => crate::models::RegistryHive::HKCU,
                    "HKLM" => crate::models::RegistryHive::HKLM,
                    _ => continue,
                };

                if let Some(ref val) = snap.value {
                    let _ = restore_value(
                        &hive_enum,
                        &snap.key,
                        &snap.value_name,
                        &snap.value_type,
                        val,
                    );
                } else {
                    let _ = registry_service::delete_value(&hive_enum, &snap.key, &snap.value_name);
                }
            }

            // Delete the snapshot since apply failed
            let _ = backup_service::delete_snapshot(&tweak_id);

            return Err(Error::RegistryOperation(format!(
                "Failed to apply '{}': {}. Rolled back {} changes.",
                tweak.name,
                e,
                applied_values.len()
            )));
        }

        applied_values.push((current_snapshot, change.enable_value.clone()));
    }

    // Step 3: Apply service changes
    if let Some(ref service_changes) = tweak.service_changes {
        for sc in service_changes {
            log::info!(
                "Applying service change: {} -> {:?}",
                sc.name,
                sc.enable_startup
            );

            // Stop service if required
            if sc.stop_on_disable {
                if let Err(e) = service_control::stop_service(&sc.name) {
                    log::warn!("Failed to stop service '{}': {}", sc.name, e);
                }
            }

            // Set startup type
            if let Err(e) = service_control::set_service_startup(&sc.name, &sc.enable_startup) {
                log::error!("Failed to set service '{}' startup: {}", sc.name, e);
                // Don't fail the whole operation for service errors
            }
        }
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

/// Apply a specific option for a multi-state tweak
#[tauri::command]
pub async fn apply_tweak_option(
    app: AppHandle,
    tweak_id: String,
    option_index: usize,
) -> Result<TweakResult> {
    log::info!(
        "Command: apply_tweak_option({}, option={})",
        tweak_id,
        option_index
    );

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    let system_info = system_info_service::get_system_info()?;
    let version = system_info.windows.version_number();

    // Check admin if required
    if tweak.requires_admin && !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak.get_changes_for_version(version);
    if changes.is_empty() {
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Capture snapshot if not already exists
    if !backup_service::snapshot_exists(&tweak_id)? {
        let snapshot = backup_service::capture_snapshot(&tweak, version)?;
        backup_service::save_snapshot(&snapshot)?;
    }

    // Apply the selected option for each registry change
    for change in &changes {
        if let Some(ref options) = change.options {
            if option_index < options.len() {
                let option = &options[option_index];

                // Apply registry value
                write_registry_value(
                    &app,
                    change,
                    &option.value,
                    "Setting option",
                    &tweak.name,
                    tweak.requires_system,
                )?;

                // Apply per-option service changes if defined
                if let Some(ref services) = option.service_changes {
                    for svc in services {
                        log::info!(
                            "Applying option service change: {} -> {:?}",
                            svc.name,
                            svc.startup
                        );

                        // Set startup type
                        if let Err(e) =
                            service_control::set_service_startup(&svc.name, &svc.startup)
                        {
                            log::warn!("Failed to set service '{}' startup: {}", svc.name, e);
                        }

                        // Stop service if startup is disabled and stop_if_disabled is set
                        if svc.stop_if_disabled
                            && svc.startup == crate::models::ServiceStartupType::Disabled
                        {
                            if let Err(e) = service_control::stop_service(&svc.name) {
                                log::warn!("Failed to stop service '{}': {}", svc.name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    log::info!(
        "Successfully applied option {} for '{}'",
        option_index,
        tweak.name
    );

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Success,
            &format!("Applied option for: {}", tweak.name),
            if tweak.requires_reboot {
                Some("Reboot required")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Applied option {} for {}", option_index, tweak.name),
        requires_reboot: tweak.requires_reboot,
    })
}

/// Revert a tweak (restore to snapshot state, or use disable_value if no snapshot)
#[tauri::command]
pub async fn revert_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: revert_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    let system_info = system_info_service::get_system_info()?;
    let version = system_info.windows.version_number();

    // Check admin if required
    if tweak.requires_admin && !system_info.is_admin {
        log::warn!("Tweak '{}' requires admin, but running as user", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak.get_changes_for_version(version);

    // Try to load existing snapshot
    let snapshot = backup_service::load_snapshot(&tweak_id)?;

    if let Some(snap) = snapshot {
        // CASE 1: We have a snapshot - restore from it
        log::info!(
            "Reverting '{}' using snapshot from {}",
            tweak.name,
            snap.applied_at
        );

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Info,
                &format!("Reverting: {}", tweak.name),
                Some(&format!(
                    "{} registry values to restore",
                    snap.registry_snapshots.len()
                )),
            );
        }

        // Restore registry values from snapshot (atomic)
        backup_service::restore_from_snapshot(&snap)?;

        // Delete snapshot after successful revert
        backup_service::delete_snapshot(&tweak_id)?;
    } else {
        // CASE 2: No snapshot exists (tweak was enabled before app was installed)
        // Capture current state FIRST, then apply disable_value from YAML
        log::info!(
            "No snapshot found for '{}' - capturing current state and using disable_value",
            tweak.name
        );

        // Capture current state so user can undo this revert later
        let snapshot = backup_service::capture_snapshot(&tweak, version)?;
        backup_service::save_snapshot(&snapshot)?;

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Info,
                &format!("Disabling: {} (no previous snapshot)", tweak.name),
                Some(&format!("{} registry values to set", changes.len())),
            );
        }

        // Apply disable_value for each registry change
        // Track if any writes fail so we can clean up
        let mut write_failed = false;
        let mut write_error: Option<Error> = None;

        for change in &changes {
            if let Some(ref disable_value) = change.disable_value {
                // Use disable_value from YAML
                if let Err(e) = write_registry_value(
                    &app,
                    change,
                    disable_value,
                    "Disabling",
                    &tweak.name,
                    tweak.requires_system,
                ) {
                    log::error!("Failed to write disable_value: {}", e);
                    write_failed = true;
                    write_error = Some(e);
                    break;
                }
            } else {
                // No disable_value - delete the key (restore to "not configured")
                log::debug!(
                    "No disable_value for {}\\{}\\{} - deleting key",
                    change.hive.as_str(),
                    change.key,
                    change.value_name
                );
                let _ =
                    registry_service::delete_value(&change.hive, &change.key, &change.value_name);
            }
        }

        // If any write failed, delete the snapshot we just created and return error
        if write_failed {
            log::error!(
                "Failed to apply disable_value for '{}' - deleting snapshot",
                tweak.name
            );
            let _ = backup_service::delete_snapshot(&tweak_id);
            return Err(write_error
                .unwrap_or_else(|| Error::RegistryOperation("Unknown error".to_string())));
        }
    }

    // Revert service changes (same for both cases)
    if let Some(ref service_changes) = tweak.service_changes {
        for sc in service_changes {
            log::info!("Reverting service: {} -> {:?}", sc.name, sc.disable_startup);

            // Restore startup type
            if let Err(e) = service_control::set_service_startup(&sc.name, &sc.disable_startup) {
                log::warn!("Failed to restore service '{}' startup: {}", sc.name, e);
            }

            // Start service if required
            if sc.start_on_enable {
                if let Err(e) = service_control::start_service(&sc.name) {
                    log::warn!("Failed to start service '{}': {}", sc.name, e);
                }
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

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Success,
            &format!("Successfully reverted: {}", tweak.name),
            if tweak.requires_reboot {
                Some("Reboot required")
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

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Batch applying {} tweaks", tweak_ids.len()),
            None,
        );
    }

    let mut requires_reboot = false;

    for tweak_id in &tweak_ids {
        let result = Box::pin(apply_tweak(app.clone(), tweak_id.clone())).await;

        if let Err(e) = result {
            log::warn!("Failed to apply tweak '{}' in batch: {}", tweak_id, e);
            // Continue with other tweaks
        } else if let Ok(res) = result {
            if res.requires_reboot {
                requires_reboot = true;
            }
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

// ============================================================================
// Helper Functions
// ============================================================================

/// Read current registry value for a change
fn read_current_value(change: &crate::models::RegistryChange) -> Result<Option<serde_json::Value>> {
    match change.value_type {
        crate::models::RegistryValueType::DWord => {
            registry_service::read_dword(&change.hive, &change.key, &change.value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
        crate::models::RegistryValueType::String
        | crate::models::RegistryValueType::ExpandString => {
            registry_service::read_string(&change.hive, &change.key, &change.value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
        crate::models::RegistryValueType::Binary => {
            registry_service::read_binary(&change.hive, &change.key, &change.value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
        crate::models::RegistryValueType::QWord => {
            registry_service::read_qword(&change.hive, &change.key, &change.value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
        crate::models::RegistryValueType::MultiString => {
            log::trace!("MultiString type not supported");
            Ok(None)
        }
    }
}

/// Write a registry value based on type
/// If use_system is true, uses SYSTEM elevation via reg.exe
fn write_registry_value(
    app: &AppHandle,
    change: &crate::models::RegistryChange,
    value: &serde_json::Value,
    operation: &str,
    tweak_name: &str,
    use_system: bool,
) -> Result<()> {
    let hive_name = change.hive.as_str();
    let full_path = format!("{}\\{}\\{}", hive_name, change.key, change.value_name);

    // If SYSTEM elevation requested, use reg.exe via trusted_installer
    if use_system {
        return write_registry_value_as_system(change, value, operation, &full_path);
    }

    match change.value_type {
        crate::models::RegistryValueType::DWord => {
            if let Some(v) = value.as_u64() {
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("{} DWORD: {} = {}", operation, full_path, v),
                        Some(tweak_name),
                    );
                }
                registry_service::set_dword(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    v as u32,
                )?;
            }
        }
        crate::models::RegistryValueType::String
        | crate::models::RegistryValueType::ExpandString => {
            if let Some(v) = value.as_str() {
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("{} String: {} = {}", operation, full_path, v),
                        Some(tweak_name),
                    );
                }
                registry_service::set_string(&change.hive, &change.key, &change.value_name, v)?;
            }
        }
        crate::models::RegistryValueType::Binary => {
            if let Some(arr) = value.as_array() {
                let binary: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!(
                            "{} Binary: {} ({} bytes)",
                            operation,
                            full_path,
                            binary.len()
                        ),
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
        crate::models::RegistryValueType::QWord => {
            if let Some(v) = value.as_u64() {
                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!("{} QWORD: {} = {}", operation, full_path, v),
                        Some(tweak_name),
                    );
                }
                registry_service::set_qword(&change.hive, &change.key, &change.value_name, v)?;
            }
        }
        crate::models::RegistryValueType::MultiString => {
            log::warn!(
                "MultiString registry type not supported for {}: {}",
                operation.to_lowercase(),
                full_path
            );
        }
    }

    Ok(())
}

/// Write a registry value as SYSTEM using reg.exe
fn write_registry_value_as_system(
    change: &crate::models::RegistryChange,
    value: &serde_json::Value,
    operation: &str,
    full_path: &str,
) -> Result<()> {
    let hive_name = change.hive.as_str();
    let value_type = change.value_type.as_str();

    // Convert value to string for reg.exe
    let value_data = match change.value_type {
        crate::models::RegistryValueType::DWord | crate::models::RegistryValueType::QWord => {
            value.as_u64().map(|v| v.to_string())
        }
        crate::models::RegistryValueType::String
        | crate::models::RegistryValueType::ExpandString => {
            value.as_str().map(|s| format!("\"{}\"", s))
        }
        _ => {
            log::warn!(
                "SYSTEM elevation not supported for type {:?}: {}",
                change.value_type,
                full_path
            );
            return Ok(());
        }
    };

    if let Some(data) = value_data {
        log::info!(
            "[SYSTEM] {} {}: {} = {}",
            operation,
            value_type,
            full_path,
            data
        );
        trusted_installer::set_registry_value_as_system(
            hive_name,
            &change.key,
            &change.value_name,
            value_type,
            &data,
        )?;
    }

    Ok(())
}

/// Restore a value to the registry
fn restore_value(
    hive: &crate::models::RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &str,
    value: &serde_json::Value,
) -> Result<()> {
    match value_type {
        "REG_DWORD" => {
            if let Some(v) = value.as_u64() {
                registry_service::set_dword(hive, key, value_name, v as u32)?;
            }
        }
        "REG_SZ" | "REG_EXPAND_SZ" => {
            if let Some(v) = value.as_str() {
                registry_service::set_string(hive, key, value_name, v)?;
            }
        }
        "REG_BINARY" => {
            if let Some(arr) = value.as_array() {
                let binary: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                registry_service::set_binary(hive, key, value_name, &binary)?;
            }
        }
        "REG_QWORD" => {
            if let Some(v) = value.as_u64() {
                registry_service::set_qword(hive, key, value_name, v)?;
            }
        }
        _ => {}
    }
    Ok(())
}
