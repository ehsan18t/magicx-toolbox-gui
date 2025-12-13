//! Tweak Commands - Unified Option-Based Model
//!
//! Every tweak has an `options` array. Each option contains:
//! - `label`: Display name (e.g., "Enabled", "Disabled", "High Performance")
//! - `registry_changes`: Registry modifications for this option
//! - `service_changes`: Service startup changes for this option
//! - `scheduler_changes`: Windows Task Scheduler changes for this option
//! - `pre_commands` / `post_commands`: Shell commands to run
//! - `pre_powershell` / `post_powershell`: PowerShell commands to run
//!
//! Execution order when applying an option:
//! 1. pre_commands (shell)
//! 2. pre_powershell (PowerShell)
//! 3. registry_changes
//! 4. service_changes
//! 5. scheduler_changes
//! 6. post_commands (shell)
//! 7. post_powershell (PowerShell)
//!
//! Tweaks with `is_toggle: true` have exactly 2 options and render as a toggle switch.
//! Tweaks with `is_toggle: false` render as a dropdown selector.

use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::{
    CategoryDefinition, RegistryHive, RegistryValueType, SchedulerAction, TweakDefinition,
    TweakOption, TweakResult, TweakStatus,
};
use crate::services::{
    backup_service, registry_service, scheduler_service, service_control, system_info_service,
    trusted_installer, tweak_loader,
};
use tauri::AppHandle;

// ============================================================================
// Query Commands
// ============================================================================

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
        .ok_or_else(|| Error::WindowsApi(format!("Tweak not found: {}", tweak_id)))?;

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
    })
}

/// Get status of all tweaks
#[tauri::command]
pub async fn get_all_tweak_statuses() -> Result<Vec<TweakStatus>> {
    log::debug!("Command: get_all_tweak_statuses");
    let windows_info = system_info_service::get_windows_info()?;
    let version = windows_info.version_number();

    let tweaks = tweak_loader::get_tweaks_for_version(version)?;
    let mut statuses = Vec::new();

    for (id, tweak) in tweaks {
        let state = backup_service::detect_tweak_state(&tweak, version)?;
        let last_applied = backup_service::load_snapshot(&id)?.map(|s| s.created_at);

        statuses.push(TweakStatus {
            tweak_id: id,
            is_applied: state.current_option_index == Some(0),
            last_applied,
            has_backup: state.has_snapshot,
            current_option_index: state.current_option_index,
        });
    }

    log::debug!("Returning {} tweak statuses", statuses.len());
    Ok(statuses)
}

// ============================================================================
// Apply Commands
// ============================================================================

/// Apply a specific option for a tweak
///
/// For toggle tweaks (is_toggle: true):
/// - option_index 0 = first option (usually "Enabled" or "On")
/// - option_index 1 = second option (usually "Disabled" or "Off")
///
/// For dropdown tweaks (is_toggle: false):
/// - option_index corresponds to the options array index
#[tauri::command]
pub async fn apply_tweak(
    app: AppHandle,
    tweak_id: String,
    option_index: usize,
) -> Result<TweakResult> {
    log::info!(
        "Command: apply_tweak({}, option_index={})",
        tweak_id,
        option_index
    );

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    // Validate option_index
    if option_index >= tweak.options.len() {
        return Err(Error::WindowsApi(format!(
            "Invalid option index {} for tweak '{}' (has {} options)",
            option_index,
            tweak.name,
            tweak.options.len()
        )));
    }

    let option = &tweak.options[option_index];
    let system_info = system_info_service::get_system_info()?;
    let version = system_info.windows.version_number();

    log::debug!(
        "Applying option '{}' for '{}' on Windows {}",
        option.label,
        tweak.name,
        version
    );

    // Check admin if required
    if tweak.requires_admin && !system_info.is_admin {
        log::warn!("Tweak '{}' requires admin, but running as user", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    // Check if already at this option
    let current_state = backup_service::detect_tweak_state(&tweak, version)?;
    if current_state.current_option_index == Some(option_index) {
        log::info!(
            "Tweak '{}' is already at option '{}', skipping",
            tweak.name,
            option.label
        );
        return Ok(TweakResult {
            success: true,
            message: format!("Already at option: {}", option.label),
            requires_reboot: false,
        });
    }

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Applying: {} → {}", tweak.name, option.label),
            None,
        );
    }

    // Step 1: Snapshot handling
    // - If no snapshot exists: capture original state (for future revert)
    // - If snapshot exists: capture CURRENT state (for rollback on failure)
    let is_switching_options = backup_service::snapshot_exists(&tweak_id)?;
    let pre_apply_state = if is_switching_options {
        // Switching options: capture current state BEFORE making changes
        // This allows rollback to the PREVIOUS option state, not original pre-tweak state
        log::info!(
            "Switching options for '{}': capturing current state for potential rollback",
            tweak.name
        );
        Some(backup_service::capture_current_state(&tweak, version)?)
    } else {
        // First apply: capture original state for the snapshot
        let snapshot = backup_service::capture_snapshot(&tweak, option_index, version)?;
        backup_service::save_snapshot(&snapshot)?;
        log::info!(
            "Captured original snapshot for '{}' with {} registry values, {} services",
            tweak.name,
            snapshot.registry_snapshots.len(),
            snapshot.service_snapshots.len()
        );
        None
    };

    // Step 2: Run pre_commands if defined (non-reversible, fail-fast)
    for cmd in &option.pre_commands {
        if let Err(e) = run_command(cmd, tweak.requires_system) {
            log::error!("Pre-command failed, aborting: {}", e);
            return Err(Error::CommandExecution(format!(
                "Pre-command failed: {}",
                e
            )));
        }
    }

    // Step 3: Run pre_powershell if defined (non-reversible, fail-fast)
    for ps_cmd in &option.pre_powershell {
        if let Err(e) = run_powershell_command(ps_cmd, tweak.requires_system) {
            log::error!("Pre-PowerShell command failed, aborting: {}", e);
            return Err(Error::CommandExecution(format!(
                "Pre-PowerShell failed: {}",
                e
            )));
        }
    }

    // Steps 4-6: Apply all core changes ATOMICALLY (registry, services, scheduler)
    // If any step fails, rollback ALL previously successful steps
    if let Err(e) = apply_all_changes_atomically(&app, &tweak, option, version) {
        log::error!("Failed to apply changes for '{}': {}", tweak.name, e);

        // Rollback based on context:
        // - If switching options: restore to pre-apply state (previous option's state)
        // - If first apply: restore to original snapshot
        if let Some(ref current_state) = pre_apply_state {
            // Switching options: restore to the state BEFORE this apply started
            log::warn!("Rolling back to previous option state (switching options failed)...");
            let _ = backup_service::restore_from_snapshot(current_state);
            // Keep the original snapshot intact - the tweak is still "applied" at previous option
        } else {
            // First apply: restore from original snapshot and delete it
            if let Some(snapshot) = backup_service::load_snapshot(&tweak_id)? {
                log::warn!("Rolling back ALL changes to original state (first apply failed)...");
                let _ = backup_service::restore_from_snapshot(&snapshot);
            }
            backup_service::delete_snapshot(&tweak_id)?;
        }

        return Err(e);
    }

    // Step 7: If switching options succeeded, update the snapshot metadata
    if is_switching_options {
        backup_service::update_snapshot_metadata(&tweak_id, option_index, &option.label)?;
    }

    // Step 7: Run post_commands (non-fatal, no rollback)
    for cmd in &option.post_commands {
        if let Err(e) = run_command(cmd, tweak.requires_system) {
            log::warn!("Post-command failed (non-fatal): {}", e);
        }
    }

    // Step 8: Run post_powershell (non-fatal, no rollback)
    for ps_cmd in &option.post_powershell {
        if let Err(e) = run_powershell_command(ps_cmd, tweak.requires_system) {
            log::warn!("Post-PowerShell command failed (non-fatal): {}", e);
        }
    }

    log::info!(
        "Successfully applied '{}' → '{}'{}",
        tweak.name,
        option.label,
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
            &format!("Applied: {} → {}", tweak.name, option.label),
            if tweak.requires_reboot {
                Some("Reboot required")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Applied: {} → {}", tweak.name, option.label),
        requires_reboot: tweak.requires_reboot,
    })
}

/// Revert a tweak to its original state (restore from snapshot)
#[tauri::command]
pub async fn revert_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: revert_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::WindowsApi(format!("Tweak not found: {}", tweak_id))
    })?;

    let system_info = system_info_service::get_system_info()?;

    // Check admin if required
    if tweak.requires_admin && !system_info.is_admin {
        log::warn!("Tweak '{}' requires admin, but running as user", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    // Load snapshot
    let snapshot = backup_service::load_snapshot(&tweak_id)?
        .ok_or_else(|| Error::BackupFailed("No snapshot found for this tweak".into()))?;

    log::info!(
        "Reverting '{}' from option '{}' (snapshot from {})",
        tweak.name,
        snapshot.applied_option_label,
        snapshot.created_at
    );

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Reverting: {}", tweak.name),
            Some(&format!(
                "{} registry values, {} services",
                snapshot.registry_snapshots.len(),
                snapshot.service_snapshots.len()
            )),
        );
    }

    // Restore from snapshot
    backup_service::restore_from_snapshot(&snapshot)?;

    // Delete snapshot after successful restore
    backup_service::delete_snapshot(&tweak_id)?;

    log::info!(
        "Successfully reverted '{}'{}",
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
            &format!("Reverted: {}", tweak.name),
            if tweak.requires_reboot {
                Some("Reboot required")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: true,
        message: format!("Reverted: {}", tweak.name),
        requires_reboot: tweak.requires_reboot,
    })
}

/// Batch apply multiple tweak options
/// Input: Vec of (tweak_id, option_index) tuples
#[tauri::command]
pub async fn batch_apply_tweaks(
    app: AppHandle,
    operations: Vec<(String, usize)>,
) -> Result<TweakResult> {
    log::info!(
        "Command: batch_apply_tweaks({} operations)",
        operations.len()
    );

    let system_info = system_info_service::get_system_info()?;

    if !system_info.is_admin {
        log::warn!("Batch apply requires admin privileges");
        return Err(Error::RequiresAdmin);
    }

    if is_debug_enabled() {
        emit_debug_log(
            &app,
            DebugLevel::Info,
            &format!("Batch applying {} tweaks", operations.len()),
            None,
        );
    }

    let mut requires_reboot = false;
    let mut success_count = 0;
    let mut failure_count = 0;

    for (tweak_id, option_index) in &operations {
        let result = Box::pin(apply_tweak(app.clone(), tweak_id.clone(), *option_index)).await;

        match result {
            Ok(res) => {
                success_count += 1;
                if res.requires_reboot {
                    requires_reboot = true;
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to apply tweak '{}' option {}: {}",
                    tweak_id,
                    option_index,
                    e
                );
                failure_count += 1;
            }
        }
    }

    let message = if failure_count > 0 {
        format!(
            "Applied {}/{} tweaks ({} failed)",
            success_count,
            operations.len(),
            failure_count
        )
    } else {
        format!("Successfully applied {} tweaks", success_count)
    };

    log::info!(
        "Batch apply completed: {}{}",
        message,
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
            &message,
            if requires_reboot {
                Some("Reboot required")
            } else {
                None
            },
        );
    }

    Ok(TweakResult {
        success: failure_count == 0,
        message,
        requires_reboot,
    })
}

/// Batch revert multiple tweaks
#[tauri::command]
pub async fn batch_revert_tweaks(app: AppHandle, tweak_ids: Vec<String>) -> Result<TweakResult> {
    log::info!("Command: batch_revert_tweaks({} tweaks)", tweak_ids.len());

    let system_info = system_info_service::get_system_info()?;

    if !system_info.is_admin {
        return Err(Error::RequiresAdmin);
    }

    let mut requires_reboot = false;
    let mut success_count = 0;

    for tweak_id in &tweak_ids {
        let result = Box::pin(revert_tweak(app.clone(), tweak_id.clone())).await;

        match result {
            Ok(res) => {
                success_count += 1;
                if res.requires_reboot {
                    requires_reboot = true;
                }
            }
            Err(e) => {
                log::warn!("Failed to revert tweak '{}': {}", tweak_id, e);
            }
        }
    }

    Ok(TweakResult {
        success: true,
        message: format!("Reverted {} tweaks", success_count),
        requires_reboot,
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Run a shell command (as user or SYSTEM)
fn run_command(cmd: &str, use_system: bool) -> Result<()> {
    log::info!(
        "Running command{}: {}",
        if use_system { " as SYSTEM" } else { "" },
        cmd
    );

    if use_system {
        match trusted_installer::run_command_as_system(cmd) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    log::warn!("Command returned exit code {}: {}", exit_code, cmd);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!(
                "SYSTEM command failed: {}",
                e
            ))),
        }
    } else {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = std::process::Command::new("cmd")
            .raw_arg(format!("/C {}", cmd))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| Error::CommandExecution(e.to_string()))?;

        if !output.status.success() {
            log::warn!(
                "Command failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }
}

/// Apply all registry changes for an option atomically
fn apply_registry_changes(
    app: &AppHandle,
    tweak: &TweakDefinition,
    option: &TweakOption,
    windows_version: u32,
) -> Result<()> {
    let mut applied: Vec<(RegistryHive, String, String, Option<serde_json::Value>)> = Vec::new();

    for change in &option.registry_changes {
        // Skip if not for this Windows version
        if !change.applies_to_version(windows_version) {
            continue;
        }

        let full_path = format!(
            "{}\\{}\\{}",
            change.hive.as_str(),
            change.key,
            change.value_name
        );

        // Read current value for rollback (only for validatable changes)
        let current = if !change.skip_validation {
            read_registry_value(
                &change.hive,
                &change.key,
                &change.value_name,
                &change.value_type,
            )?
        } else {
            None
        };

        log::debug!(
            "Setting{} {} = {:?} (was {:?})",
            if change.skip_validation {
                " (skip_validation)"
            } else {
                ""
            },
            full_path,
            change.value,
            current
        );

        // Write new value
        if let Err(e) = write_registry_value(
            &change.hive,
            &change.key,
            &change.value_name,
            &change.value_type,
            &change.value,
            tweak.requires_system,
        ) {
            if change.skip_validation {
                // For skip_validation items, log warning but continue
                log::warn!(
                    "Failed to write {} (skip_validation, continuing): {}",
                    full_path,
                    e
                );
                continue;
            }

            log::error!("Failed to write {}: {}", full_path, e);

            // Rollback applied changes
            for (hive, key, value_name, original) in applied.iter().rev() {
                if let Some(val) = original {
                    // Restore original value - guess type from value
                    let _ = restore_value(hive, key, value_name, val);
                } else {
                    // Delete the value we created
                    let _ = registry_service::delete_value(hive, key, value_name);
                }
            }

            return Err(e);
        }

        // Only track for rollback if NOT skip_validation
        if !change.skip_validation {
            applied.push((
                change.hive,
                change.key.clone(),
                change.value_name.clone(),
                current,
            ));
        }

        if is_debug_enabled() {
            emit_debug_log(
                app,
                DebugLevel::Info,
                &format!(
                    "Set{}{}",
                    if change.skip_validation { " [sv]" } else { "" },
                    full_path
                ),
                Some(&format!("{:?}", change.value)),
            );
        }
    }

    log::debug!("Applied {} registry changes", applied.len());
    Ok(())
}

/// Apply ALL core changes atomically: registry, services, scheduler
/// If any step fails, caller is responsible for full rollback from snapshot
fn apply_all_changes_atomically(
    app: &AppHandle,
    tweak: &TweakDefinition,
    option: &TweakOption,
    windows_version: u32,
) -> Result<()> {
    // Step 1: Apply registry changes (already has internal rollback on failure)
    apply_registry_changes(app, tweak, option, windows_version)?;

    // Step 2: Apply service changes - fail-fast, return error for full rollback
    if let Err(e) = apply_service_changes_atomic(option, tweak.requires_system, tweak.requires_ti) {
        log::error!("Service changes failed, need full rollback: {}", e);
        return Err(e);
    }

    // Step 3: Apply scheduler changes - fail-fast, return error for full rollback
    if let Err(e) =
        apply_scheduler_changes_atomic(app, option, tweak.requires_system, tweak.requires_ti)
    {
        log::error!("Scheduler changes failed, need full rollback: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Apply all service changes for an option atomically (fail on first error for non-skip_validation items)
fn apply_service_changes_atomic(
    option: &TweakOption,
    use_system: bool,
    use_ti: bool,
) -> Result<()> {
    for change in &option.service_changes {
        // Check current startup/state so we can skip redundant work
        let status = match service_control::get_service_status(&change.name) {
            Ok(status) => Some(status),
            Err(e) => {
                log::warn!(
                    "Could not query service '{}' status (continuing anyway): {}",
                    change.name,
                    e
                );
                None
            }
        };

        let current_startup = status.as_ref().and_then(|s| s.startup_type);
        let current_state = status.as_ref().map(|s| &s.state);

        let desired_stop =
            change.startup == crate::models::ServiceStartupType::Disabled || change.stop_service;
        // Only start when explicitly requested and not also requesting a stop
        let desired_start = change.start_service && !desired_stop;

        let startup_matches = current_startup == Some(change.startup);

        let elevation = if use_ti {
            " (TrustedInstaller)"
        } else if use_system {
            " (SYSTEM)"
        } else {
            ""
        };

        let start_type = change.startup.to_sc_start_type();

        // 1) Startup config (skip if already matching)
        if startup_matches {
            log::info!(
                "Service '{}' already at startup '{:?}', skipping config",
                change.name,
                change.startup
            );
        } else {
            log::info!(
                "Setting service{}{} '{}' startup to {:?}",
                elevation,
                if change.skip_validation {
                    " (skip_validation)"
                } else {
                    ""
                },
                change.name,
                change.startup
            );
        }

        let result = if startup_matches {
            Ok(())
        } else if use_ti {
            // TrustedInstaller elevation for protected services like WaaSMedicSvc
            trusted_installer::set_service_startup_as_ti(&change.name, start_type)
        } else if use_system {
            // SYSTEM elevation
            trusted_installer::set_service_startup_as_system(&change.name, start_type)
        } else {
            // Normal admin elevation
            service_control::set_service_startup(&change.name, &change.startup)
        };

        if let Err(e) = result {
            if change.skip_validation {
                // For skip_validation items, log warning but continue
                log::warn!(
                    "Failed to set service '{}' startup (skip_validation, continuing): {}",
                    change.name,
                    e
                );
                continue;
            }
            return Err(Error::ServiceControl(format!(
                "Failed to set service '{}' startup: {}",
                change.name, e
            )));
        }

        // 2) Stop if desired (skip if already stopped)
        if desired_stop && !matches!(current_state, Some(service_control::ServiceState::Stopped)) {
            if use_ti {
                let _ = trusted_installer::stop_service_as_ti(&change.name);
            } else if use_system {
                let _ = trusted_installer::stop_service_as_system(&change.name);
            } else {
                let _ = service_control::stop_service(&change.name);
            }
        }

        // 3) Start if explicitly requested (skip if already running)
        if desired_start && !matches!(current_state, Some(service_control::ServiceState::Running)) {
            if use_ti {
                let _ = trusted_installer::start_service_as_ti(&change.name);
            } else if use_system {
                let _ = trusted_installer::start_service_as_system(&change.name);
            } else {
                let _ = service_control::start_service(&change.name);
            }
        }

        // If we got here and everything was already aligned, say so once.
        if startup_matches
            && (!desired_stop
                || matches!(current_state, Some(service_control::ServiceState::Stopped)))
            && (!desired_start
                || matches!(current_state, Some(service_control::ServiceState::Running)))
        {
            log::info!(
                "Service '{}' already matches requested startup/state; skipping",
                change.name
            );
        }
    }
    Ok(())
}

/// Apply all scheduler changes for an option atomically (fail on first error for non-skip_validation items)
fn apply_scheduler_changes_atomic(
    app: &AppHandle,
    option: &TweakOption,
    use_system: bool,
    use_ti: bool,
) -> Result<()> {
    // Determine which elevation to use
    let use_elevated = use_ti || use_system;

    for change in &option.scheduler_changes {
        // Determine if using pattern or exact name
        let is_pattern = change.task_name_pattern.is_some();
        let identifier = if let Some(ref pattern) = change.task_name_pattern {
            pattern.clone()
        } else if let Some(ref name) = change.task_name {
            name.clone()
        } else {
            log::error!("Scheduler change has neither task_name nor task_name_pattern");
            return Err(Error::CommandExecution(
                "Scheduler change requires either task_name or task_name_pattern".to_string(),
            ));
        };

        let elevation_str = if use_ti {
            " (TrustedInstaller)"
        } else if use_system {
            " (SYSTEM)"
        } else {
            ""
        };

        let flags_str = {
            let mut flags = Vec::new();
            if change.skip_validation {
                flags.push("skip_validation");
            }
            if change.ignore_not_found {
                flags.push("ignore_not_found");
            }
            if is_pattern {
                flags.push("pattern");
            }
            if flags.is_empty() {
                String::new()
            } else {
                format!(" ({})", flags.join(", "))
            }
        };

        log::info!(
            "Applying scheduler change{}{}: {}\\{} → {:?}",
            elevation_str,
            flags_str,
            change.task_path,
            identifier,
            change.action
        );

        // Handle pattern vs exact name
        if is_pattern {
            let pattern = change.task_name_pattern.as_ref().unwrap();

            // For pattern matching, we need to find all matching tasks and apply action to each
            if use_elevated {
                // When using elevated elevation, we need to list tasks first, then apply to each
                let tasks = scheduler_service::find_tasks_by_pattern(&change.task_path, pattern)?;

                if tasks.is_empty() {
                    if change.ignore_not_found {
                        log::warn!(
                            "No tasks found matching pattern '{}' in '{}' (ignore_not_found)",
                            pattern,
                            change.task_path
                        );
                        continue;
                    } else if change.skip_validation {
                        log::warn!(
                            "No tasks found matching pattern '{}' in '{}' (skip_validation)",
                            pattern,
                            change.task_path
                        );
                        continue;
                    } else {
                        return Err(Error::CommandExecution(format!(
                            "No tasks found matching pattern '{}' in '{}'",
                            pattern, change.task_path
                        )));
                    }
                }

                // Apply action to each matching task with elevated elevation
                for task in tasks {
                    let full_path = format!("{}\\{}", change.task_path, task.name);
                    let schtasks_args = match change.action {
                        SchedulerAction::Enable => {
                            format!("/Change /TN \"{}\" /Enable", full_path)
                        }
                        SchedulerAction::Disable => {
                            format!("/Change /TN \"{}\" /Disable", full_path)
                        }
                        SchedulerAction::Delete => {
                            format!("/Delete /TN \"{}\" /F", full_path)
                        }
                    };

                    let result = if use_ti {
                        trusted_installer::run_schtasks_as_ti(&schtasks_args)
                    } else {
                        trusted_installer::run_schtasks_as_system(&schtasks_args)
                    }
                    .map(|_| ())
                    .map_err(|e| Error::CommandExecution(e.to_string()));

                    if let Err(e) = result {
                        if change.skip_validation {
                            log::warn!(
                                "Failed to apply scheduler change for '{}' (skip_validation, continuing): {}",
                                full_path,
                                e
                            );
                            continue;
                        }
                        return Err(Error::CommandExecution(format!(
                            "Failed to apply scheduler change for '{}': {}",
                            full_path, e
                        )));
                    }

                    if is_debug_enabled() {
                        emit_debug_log(
                            app,
                            DebugLevel::Info,
                            &format!(
                                "Scheduler{}: {} → {:?}",
                                flags_str, full_path, change.action
                            ),
                            None,
                        );
                    }
                }
            } else {
                // Use scheduler_service's pattern matching function
                let (success_count, error_count, errors) =
                    scheduler_service::apply_action_to_pattern(
                        &change.task_path,
                        pattern,
                        change.action,
                        change.ignore_not_found,
                    )?;

                if error_count > 0 {
                    if change.skip_validation {
                        log::warn!(
                            "Pattern '{}': {} succeeded, {} failed (skip_validation): {:?}",
                            pattern,
                            success_count,
                            error_count,
                            errors
                        );
                    } else {
                        return Err(Error::CommandExecution(format!(
                            "Pattern '{}': {} succeeded, {} failed: {:?}",
                            pattern, success_count, error_count, errors
                        )));
                    }
                }

                if is_debug_enabled() {
                    emit_debug_log(
                        app,
                        DebugLevel::Info,
                        &format!(
                            "Scheduler{}: {}\\[{}] → {:?} ({} tasks)",
                            flags_str, change.task_path, pattern, change.action, success_count
                        ),
                        None,
                    );
                }
            }
        } else {
            // Exact task name
            let task_name = change.task_name.as_ref().unwrap();
            let full_path = format!("{}\\{}", change.task_path, task_name);

            let result = if use_elevated {
                // Use schtasks via trusted_installer for elevated operations
                let schtasks_args = match change.action {
                    SchedulerAction::Enable => {
                        format!("/Change /TN \"{}\" /Enable", full_path)
                    }
                    SchedulerAction::Disable => {
                        format!("/Change /TN \"{}\" /Disable", full_path)
                    }
                    SchedulerAction::Delete => {
                        format!("/Delete /TN \"{}\" /F", full_path)
                    }
                };
                if use_ti {
                    trusted_installer::run_schtasks_as_ti(&schtasks_args)
                } else {
                    trusted_installer::run_schtasks_as_system(&schtasks_args)
                }
                .map(|_| ())
                .map_err(|e| Error::CommandExecution(e.to_string()))
            } else {
                // Use scheduler_service directly
                scheduler_service::apply_scheduler_change(
                    &change.task_path,
                    task_name,
                    change.action,
                )
            };

            // Handle errors with flag-based behavior
            if let Err(e) = result {
                let is_not_found = e.to_string().contains("does not exist")
                    || e.to_string().contains("cannot find");

                if is_not_found && change.ignore_not_found {
                    // Task not found but ignore_not_found is set - continue
                    log::warn!(
                        "Task '{}' not found (ignore_not_found, continuing)",
                        full_path
                    );
                    continue;
                } else if change.skip_validation {
                    // Any error with skip_validation - log and continue
                    log::warn!(
                        "Failed to apply scheduler change for '{}' (skip_validation, continuing): {}",
                        full_path,
                        e
                    );
                    continue;
                } else {
                    return Err(Error::CommandExecution(format!(
                        "Failed to apply scheduler change for '{}': {}",
                        full_path, e
                    )));
                }
            }

            if is_debug_enabled() {
                emit_debug_log(
                    app,
                    DebugLevel::Info,
                    &format!(
                        "Scheduler{}: {} → {:?}",
                        flags_str, full_path, change.action
                    ),
                    None,
                );
            }
        }
    }
    Ok(())
}

/// Run a PowerShell command (as user or SYSTEM)
fn run_powershell_command(cmd: &str, use_system: bool) -> Result<()> {
    log::info!(
        "Running PowerShell{}: {}",
        if use_system { " as SYSTEM" } else { "" },
        cmd
    );

    if use_system {
        // run_powershell_as_system returns Result<i32, Error> (exit code only)
        match trusted_installer::run_powershell_as_system(cmd) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    log::warn!("PowerShell (SYSTEM) returned exit code {}", exit_code);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!(
                "PowerShell (SYSTEM) failed: {}",
                e
            ))),
        }
    } else {
        // run_powershell returns Result<PowerShellResult, Error>
        match trusted_installer::run_powershell(cmd) {
            Ok(ps_result) => {
                if ps_result.exit_code != 0 {
                    log::warn!(
                        "PowerShell returned exit code {}: {}",
                        ps_result.exit_code,
                        ps_result.stderr
                    );
                }
                if !ps_result.stdout.is_empty() {
                    log::debug!("PowerShell stdout: {}", ps_result.stdout);
                }
                if !ps_result.stderr.is_empty() && ps_result.exit_code != 0 {
                    log::debug!("PowerShell stderr: {}", ps_result.stderr);
                }
                Ok(())
            }
            Err(e) => Err(Error::CommandExecution(format!("PowerShell failed: {}", e))),
        }
    }
}

/// Read a registry value
fn read_registry_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
) -> Result<Option<serde_json::Value>> {
    let result = match value_type {
        RegistryValueType::Dword => registry_service::read_dword(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        RegistryValueType::String | RegistryValueType::ExpandString => {
            registry_service::read_string(hive, key, value_name)
                .map(|v| v.map(|val| serde_json::json!(val)))
        }
        RegistryValueType::Binary => registry_service::read_binary(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        RegistryValueType::Qword => registry_service::read_qword(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
        RegistryValueType::MultiString => registry_service::read_string(hive, key, value_name)
            .map(|v| v.map(|val| serde_json::json!(val))),
    };

    match result {
        Ok(v) => Ok(v),
        Err(Error::RegistryKeyNotFound(_)) => Ok(None),
        Err(e) => {
            log::debug!(
                "Could not read {}\\{}\\{}: {}",
                hive.as_str(),
                key,
                value_name,
                e
            );
            Ok(None)
        }
    }
}

/// Write a registry value
fn write_registry_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
    value: &serde_json::Value,
    use_system: bool,
) -> Result<()> {
    if use_system {
        return write_registry_value_as_system(hive, key, value_name, value_type, value);
    }

    match value_type {
        RegistryValueType::Dword => {
            if let Some(v) = value.as_u64() {
                registry_service::set_dword(hive, key, value_name, v as u32)?;
            }
        }
        RegistryValueType::String | RegistryValueType::ExpandString => {
            if let Some(v) = value.as_str() {
                registry_service::set_string(hive, key, value_name, v)?;
            }
        }
        RegistryValueType::Binary => {
            if let Some(arr) = value.as_array() {
                let binary: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect();
                registry_service::set_binary(hive, key, value_name, &binary)?;
            }
        }
        RegistryValueType::Qword => {
            if let Some(v) = value.as_u64() {
                registry_service::set_qword(hive, key, value_name, v)?;
            }
        }
        RegistryValueType::MultiString => {
            log::warn!("MultiString not supported for write");
        }
    }

    Ok(())
}

/// Write a registry value as SYSTEM
fn write_registry_value_as_system(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
    value: &serde_json::Value,
) -> Result<()> {
    let value_data = match value_type {
        RegistryValueType::Dword | RegistryValueType::Qword => {
            value.as_u64().map(|v| v.to_string())
        }
        RegistryValueType::String | RegistryValueType::ExpandString => {
            value.as_str().map(|s| format!("\"{}\"", s))
        }
        _ => {
            log::warn!("SYSTEM elevation not supported for {:?}", value_type);
            return Ok(());
        }
    };

    if let Some(data) = value_data {
        trusted_installer::set_registry_value_as_system(
            hive.as_str(),
            key,
            value_name,
            value_type.as_str(),
            &data,
        )?;
    }

    Ok(())
}

/// Restore a value (guess type from JSON value)
fn restore_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value: &serde_json::Value,
) -> Result<()> {
    if let Some(v) = value.as_u64() {
        // Could be DWORD or QWORD - use DWORD if it fits
        if v <= u32::MAX as u64 {
            registry_service::set_dword(hive, key, value_name, v as u32)?;
        } else {
            registry_service::set_qword(hive, key, value_name, v)?;
        }
    } else if let Some(v) = value.as_str() {
        registry_service::set_string(hive, key, value_name, v)?;
    } else if let Some(arr) = value.as_array() {
        let binary: Vec<u8> = arr
            .iter()
            .filter_map(|v| v.as_u64().map(|u| u as u8))
            .collect();
        registry_service::set_binary(hive, key, value_name, &binary)?;
    }
    Ok(())
}
