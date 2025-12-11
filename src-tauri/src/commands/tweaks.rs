use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{
    make_key_id, CategoryDefinition, RegistryKeyId, TweakDefinition, TweakResult, TweakStatus,
};
use crate::services::{
    backup_service, registry_service, service_control, system_info_service, tweak_loader,
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

    // Check for multi-state tweaks and detect current option
    let current_option_index = detect_current_option(&changes)?;

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

    // Pre-capture: Take snapshots of all values BEFORE any modifications
    // This ensures we have accurate values for rollback even if baseline capture partially failed
    let snapshots = match backup_service::capture_snapshots(&changes) {
        Ok(s) => {
            log::debug!("Pre-captured {} snapshots for rollback", s.len());
            s
        }
        Err(e) => {
            log::warn!("Failed to capture pre-apply snapshots: {}", e);
            if is_debug_enabled() {
                emit_debug_log(
                    &app,
                    DebugLevel::Warn,
                    &format!("Failed to capture snapshots for {}: {}", tweak.name, e),
                    Some("Rollback may be incomplete if apply fails"),
                );
            }
            Vec::new() // Continue without snapshots - we still have baseline
        }
    };

    // Apply all registry changes with rollback on failure
    let mut applied_changes: Vec<RegistryKeyId> = Vec::new();
    let mut applied_count = 0usize;
    let mut rollback_needed = false;
    let mut error_msg: Option<String> = None;

    for change in &changes {
        let key_id = make_key_id(change.hive.as_str(), &change.key, &change.value_name);

        match apply_registry_change(&app, change, &tweak.name) {
            Ok(()) => {
                applied_changes.push(key_id);
                applied_count += 1;
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

    // If any change failed, rollback using captured snapshots
    if rollback_needed {
        log::warn!(
            "Rolling back {} changes for tweak '{}'",
            applied_count,
            tweak.name
        );

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Warn,
                &format!("Rolling back changes for: {}", tweak.name),
                Some(&format!("{} changes to revert", applied_count)),
            );
        }

        // Use snapshot-based rollback for accuracy (only rollback what was applied)
        let snapshots_to_rollback: Vec<_> = snapshots.into_iter().take(applied_count).collect();
        if !snapshots_to_rollback.is_empty() {
            let report = backup_service::rollback_from_snapshots(&snapshots_to_rollback);

            if !report.all_succeeded {
                log::error!(
                    "Rollback partially failed: {} succeeded, {} failed",
                    report.succeeded,
                    report.failed
                );
                if is_debug_enabled() {
                    for (key_id, error) in &report.failures {
                        emit_debug_log(
                            &app,
                            DebugLevel::Error,
                            &format!("Rollback failed for {}: {}", key_id, error),
                            None,
                        );
                    }
                }

                return Err(Error::BackupFailed(format!(
                    "Failed to apply tweak '{}': {}. Rollback partially failed ({} of {} keys restored). Manual intervention may be required.",
                    tweak.name,
                    error_msg.unwrap_or_default(),
                    report.succeeded,
                    report.succeeded + report.failed
                )));
            }
        } else {
            // Fallback to baseline-based rollback if no snapshots
            for key_id in &applied_changes {
                if let Err(e) = backup_service::restore_key_to_baseline(key_id) {
                    log::error!("Failed to rollback key {}: {}", key_id, e);
                }
            }
        }

        return Err(Error::BackupFailed(format!(
            "Failed to apply tweak '{}': {}. Changes have been rolled back.",
            tweak.name,
            error_msg.unwrap_or_default()
        )));
    }

    // Post-verification: Confirm all changes were actually written
    let verification = backup_service::verify_changes(&changes, true);
    match verification {
        Ok(results) => {
            let failed: Vec<_> = results.iter().filter(|r| !r.matches).collect();
            if !failed.is_empty() {
                log::warn!(
                    "Post-verification: {} of {} changes may not have been applied correctly",
                    failed.len(),
                    results.len()
                );
                if is_debug_enabled() {
                    for result in &failed {
                        emit_debug_log(
                            &app,
                            DebugLevel::Warn,
                            &format!(
                                "Verification mismatch for {}: expected {:?}, got {:?}",
                                result.key_id, result.expected, result.actual
                            ),
                            Some("Value may not have been written correctly"),
                        );
                    }
                }
                // Note: We don't fail here - the write may have succeeded but read-back differs
                // (e.g., Windows normalizing values). We log the warning for investigation.
            } else {
                log::debug!(
                    "Post-verification: All {} changes verified successfully",
                    results.len()
                );
            }
        }
        Err(e) => {
            log::warn!("Post-verification failed: {}", e);
            // Don't fail the operation - verification is informational
        }
    }

    // Apply service changes if any
    if let Some(ref service_changes) = tweak.service_changes {
        for sc in service_changes {
            log::info!(
                "Applying service change for '{}': {} -> {:?}",
                tweak.name,
                sc.name,
                sc.enable_startup
            );

            // Stop service first if required
            if sc.stop_on_disable {
                if let Err(e) = service_control::stop_service(&sc.name) {
                    log::warn!("Failed to stop service '{}': {}", sc.name, e);
                    // Continue anyway - service might already be stopped
                }
            }

            // Set startup type
            if let Err(e) = service_control::set_service_startup(&sc.name, &sc.enable_startup) {
                log::error!("Failed to set service '{}' startup: {}", sc.name, e);
                return Err(Error::ServiceControl(format!(
                    "Failed to configure service '{}': {}",
                    sc.name, e
                )));
            }
        }
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

    // Check if tweak applies to current Windows version
    if !tweak.applies_to_version(version) {
        return Err(Error::UnsupportedWindowsVersion);
    }

    // Check if admin required
    if tweak.requires_admin && !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }

    let changes = tweak.get_changes_for_version(version);
    if changes.is_empty() {
        return Err(Error::WindowsApi(
            "No registry changes for this Windows version".to_string(),
        ));
    }

    // Get the first change (multi-state typically has one key)
    let change = changes
        .first()
        .ok_or_else(|| Error::WindowsApi("No registry changes available".to_string()))?;

    // Get options
    let options = change
        .options
        .as_ref()
        .ok_or_else(|| Error::WindowsApi("Tweak does not have multiple options".to_string()))?;

    // Validate option index
    if option_index >= options.len() {
        return Err(Error::WindowsApi(format!(
            "Invalid option index: {} (max: {})",
            option_index,
            options.len() - 1
        )));
    }

    let option = &options[option_index];
    log::debug!(
        "Applying option '{}' (index {}) for '{}'",
        option.label,
        option_index,
        tweak.name
    );

    // Debug log
    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Applying option '{}' for: {}", option.label, tweak.name),
            None,
        );
    }

    // Capture baseline
    let keys_info = vec![(
        change.hive.as_str().to_string(),
        change.key.clone(),
        change.value_name.clone(),
        format!("{:?}", change.value_type).replace("\"", ""),
    )];

    let _ = backup_service::capture_baseline_for_keys(&tweak_id, &keys_info);

    // Apply the option value
    write_registry_value(&app, change, &option.value, "Setting", &tweak.name)?;

    // Record as applied
    let key_id = make_key_id(change.hive.as_str(), &change.key, &change.value_name);
    if let Err(e) =
        backup_service::record_tweak_applied(&tweak_id, &tweak.name, version, vec![key_id])
    {
        log::warn!("Failed to record tweak state: {}", e);
    }

    log::info!(
        "Successfully applied option '{}' for tweak '{}'",
        option.label,
        tweak.name
    );

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Success,
            &format!("Applied option '{}': {}", option.label, tweak.name),
            if tweak.requires_reboot {
                Some("Reboot required")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Applied '{}' for {}", option.label, tweak.name),
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

    // Pre-capture snapshots for rollback in case of failure
    let snapshots = match backup_service::capture_snapshots(&changes) {
        Ok(s) => {
            log::debug!("Pre-captured {} snapshots for revert rollback", s.len());
            s
        }
        Err(e) => {
            log::warn!("Failed to capture pre-revert snapshots: {}", e);
            Vec::new()
        }
    };

    // Get list of orphaned keys (ref_count goes to 0) WITHOUT recording yet
    // We'll only record after successful revert
    let orphaned_keys = match backup_service::get_orphaned_keys_for_tweak(&tweak_id) {
        Ok(keys) => keys,
        Err(e) => {
            log::warn!("Could not determine orphaned keys: {}", e);
            Vec::new()
        }
    };
    log::debug!(
        "Tweak '{}' revert: {} keys will be orphaned",
        tweak.name,
        orphaned_keys.len()
    );

    // Track revert progress for rollback
    let mut reverted_count = 0usize;
    let mut revert_error: Option<String> = None;

    // For each registry change:
    // - If key is orphaned (no other tweaks use it): restore to baseline
    // - If key is still used by other tweaks: apply disable_value (or skip if none)
    for change in &changes {
        let key_id = make_key_id(change.hive.as_str(), &change.key, &change.value_name);

        let result: std::result::Result<(), Error> = (|| {
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
            Ok(())
        })();

        if let Err(e) = result {
            revert_error = Some(e.to_string());
            break;
        }
        reverted_count += 1;
    }

    // If any revert operation failed, rollback to restore original state
    if let Some(error) = revert_error {
        log::warn!(
            "Revert failed for tweak '{}' at change {}/{}",
            tweak.name,
            reverted_count + 1,
            changes.len()
        );

        if is_debug_enabled() {
            emit_debug_log(
                &app,
                DebugLevel::Warn,
                &format!("Rolling back failed revert for: {}", tweak.name),
                Some(&format!("{} changes to restore", reverted_count)),
            );
        }

        // Use snapshot-based rollback to restore original state
        let snapshots_to_rollback: Vec<_> = snapshots.into_iter().take(reverted_count).collect();
        if !snapshots_to_rollback.is_empty() {
            let report = backup_service::rollback_from_snapshots(&snapshots_to_rollback);

            if !report.all_succeeded {
                log::error!(
                    "Revert rollback partially failed: {} succeeded, {} failed",
                    report.succeeded,
                    report.failed
                );
                if is_debug_enabled() {
                    for (key_id, err) in &report.failures {
                        emit_debug_log(
                            &app,
                            DebugLevel::Error,
                            &format!("Rollback failed for {}: {}", key_id, err),
                            None,
                        );
                    }
                }

                return Err(Error::BackupFailed(format!(
                    "Failed to revert tweak '{}': {}. Rollback partially failed ({} of {} restored). Manual intervention may be required.",
                    tweak.name,
                    error,
                    report.succeeded,
                    report.succeeded + report.failed
                )));
            }
        }

        return Err(Error::BackupFailed(format!(
            "Failed to revert tweak '{}': {}. Changes have been rolled back.",
            tweak.name, error
        )));
    }

    // Revert service changes if any (restore original startup type and start services)
    if let Some(ref service_changes) = tweak.service_changes {
        for sc in service_changes {
            log::info!(
                "Reverting service change for '{}': {} -> {:?}",
                tweak.name,
                sc.name,
                sc.disable_startup
            );

            // Set startup type back to original
            if let Err(e) = service_control::set_service_startup(&sc.name, &sc.disable_startup) {
                log::warn!("Failed to restore service '{}' startup: {}", sc.name, e);
                // Continue anyway - don't fail the whole revert
            }

            // Start service if required
            if sc.start_on_enable {
                if let Err(e) = service_control::start_service(&sc.name) {
                    log::warn!("Failed to start service '{}': {}", sc.name, e);
                    // Continue anyway - service might need manual start
                }
            }
        }
    }

    // All reverts succeeded - now record the tweak as reverted in state
    if let Err(e) = backup_service::record_tweak_reverted(&tweak_id) {
        log::warn!(
            "Failed to record tweak revert state for '{}': {}",
            tweak.name,
            e
        );
        // Don't fail - the registry changes were successful
    }

    // Post-verification: Confirm changes were reverted correctly
    let verification = backup_service::verify_changes(&changes, false);
    match verification {
        Ok(results) => {
            let failed: Vec<_> = results.iter().filter(|r| !r.matches).collect();
            if !failed.is_empty() {
                log::warn!(
                    "Revert verification: {} of {} changes may not match expected values",
                    failed.len(),
                    results.len()
                );
                // Note: Don't fail - some keys may have been skipped intentionally
            } else {
                log::debug!(
                    "Revert verification: All {} changes verified",
                    results.len()
                );
            }
        }
        Err(e) => {
            log::trace!("Revert verification failed: {}", e);
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

/// Check if a tweak is currently applied by reading registry values.
/// Returns true only if ALL registry changes match their enable_value.
fn check_tweak_applied(changes: &[&crate::models::RegistryChange]) -> Result<bool> {
    if changes.is_empty() {
        return Ok(false);
    }

    // Check ALL registry changes - tweak is applied only if all values match
    for change in changes {
        let current_value = match change.value_type {
            crate::models::RegistryValueType::DWord => {
                registry_service::read_dword(&change.hive, &change.key, &change.value_name)?
                    .map(|v| serde_json::json!(v))
            }
            crate::models::RegistryValueType::String
            | crate::models::RegistryValueType::ExpandString => {
                registry_service::read_string(&change.hive, &change.key, &change.value_name)?
                    .map(|v| serde_json::json!(v))
            }
            crate::models::RegistryValueType::Binary => {
                registry_service::read_binary(&change.hive, &change.key, &change.value_name)?
                    .map(|v| serde_json::json!(v))
            }
            crate::models::RegistryValueType::QWord => {
                registry_service::read_qword(&change.hive, &change.key, &change.value_name)?
                    .map(|v| serde_json::json!(v))
            }
            crate::models::RegistryValueType::MultiString => {
                // MultiString is not commonly used, treat as not matching
                log::trace!("MultiString type not supported in status check");
                None
            }
        };

        match current_value {
            Some(current) if current == change.enable_value => {
                // This change matches, continue checking others
                continue;
            }
            _ => {
                // Value doesn't match or couldn't be read - tweak is not fully applied
                return Ok(false);
            }
        }
    }

    // All changes match their enable_value
    Ok(true)
}

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

/// Detect which option is currently active for multi-state tweaks.
/// Returns None for binary tweaks or if no matching option is found.
fn detect_current_option(changes: &[&crate::models::RegistryChange]) -> Result<Option<usize>> {
    // Only check the first change for multi-state (usually there's just one)
    let first_change = match changes.first() {
        Some(c) => c,
        None => return Ok(None),
    };

    // Check if this is a multi-state tweak
    let options = match &first_change.options {
        Some(opts) if opts.len() > 1 => opts,
        _ => return Ok(None), // Binary tweak or no options
    };

    // Read current value
    let current_value = match read_current_value(first_change)? {
        Some(v) => v,
        None => return Ok(None),
    };

    // Find which option matches
    for (index, option) in options.iter().enumerate() {
        if current_value == option.value {
            return Ok(Some(index));
        }
    }

    // No matching option found (might be a custom value)
    Ok(None)
}

/// Write a registry value based on type (unified helper for apply/revert)
fn write_registry_value(
    app: &AppHandle,
    change: &crate::models::RegistryChange,
    value: &serde_json::Value,
    operation: &str,
    tweak_name: &str,
) -> Result<()> {
    let hive_name = change.hive.as_str();
    let full_path = format!("{}\\{}\\{}", hive_name, change.key, change.value_name);

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
                        &format!("{} String: {} = \"{}\"", operation, full_path, v),
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

/// Apply a single registry change
fn apply_registry_change(
    app: &AppHandle,
    change: &crate::models::RegistryChange,
    tweak_name: &str,
) -> Result<()> {
    write_registry_value(app, change, &change.enable_value, "Setting", tweak_name)
}

/// Revert a registry change to disable value
fn revert_registry_change(
    app: &AppHandle,
    change: &crate::models::RegistryChange,
    disable_value: &serde_json::Value,
    tweak_name: &str,
) -> Result<()> {
    write_registry_value(app, change, disable_value, "Reverting", tweak_name)
}
