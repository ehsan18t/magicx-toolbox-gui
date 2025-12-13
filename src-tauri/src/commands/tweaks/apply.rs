//! Apply Commands - Single tweak apply/revert operations

use super::helpers::{apply_all_changes_atomically, run_command, run_powershell_command};
use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::TweakResult;
use crate::services::{backup_service, system_info_service, tweak_loader};
use tauri::AppHandle;

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
        Error::NotFound(format!("Tweak '{}'", tweak_id))
    })?;

    // Validate option_index
    if option_index >= tweak.options.len() {
        return Err(Error::ValidationError(format!(
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
            failures: Vec::new(),
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
    let is_switching_options = backup_service::snapshot_exists(&tweak_id)?;
    let pre_apply_state = if is_switching_options {
        log::info!(
            "Switching options for '{}': capturing current state for potential rollback",
            tweak.name
        );
        Some(backup_service::capture_current_state(&tweak, version)?)
    } else {
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
        if let Err(e) = run_command(cmd, tweak.requires_system, tweak.requires_ti) {
            log::error!("Pre-command failed, aborting: {}", e);
            return Err(Error::CommandExecution(format!(
                "Pre-command failed: {}",
                e
            )));
        }
    }

    // Step 3: Run pre_powershell if defined (non-reversible, fail-fast)
    for ps_cmd in &option.pre_powershell {
        if let Err(e) = run_powershell_command(ps_cmd, tweak.requires_system, tweak.requires_ti) {
            log::error!("Pre-PowerShell command failed, aborting: {}", e);
            return Err(Error::CommandExecution(format!(
                "Pre-PowerShell failed: {}",
                e
            )));
        }
    }

    // Steps 4-6: Apply all core changes ATOMICALLY
    if let Err(e) = apply_all_changes_atomically(&app, &tweak, option, version) {
        log::error!("Failed to apply changes for '{}': {}", tweak.name, e);

        // Rollback based on context
        if let Some(ref current_state) = pre_apply_state {
            log::warn!("Rolling back to previous option state (switching options failed)...");
            let _ = backup_service::restore_from_snapshot(current_state);
        } else {
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

    // Step 8: Run post_commands (non-fatal, no rollback)
    for cmd in &option.post_commands {
        if let Err(e) = run_command(cmd, tweak.requires_system, tweak.requires_ti) {
            log::warn!("Post-command failed (non-fatal): {}", e);
        }
    }

    // Step 9: Run post_powershell (non-fatal, no rollback)
    for ps_cmd in &option.post_powershell {
        if let Err(e) = run_powershell_command(ps_cmd, tweak.requires_system, tweak.requires_ti) {
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
        failures: Vec::new(),
    })
}

/// Revert a tweak to its original state (restore from snapshot)
#[tauri::command]
pub async fn revert_tweak(app: AppHandle, tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: revert_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::NotFound(format!("Tweak '{}'", tweak_id))
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
        failures: Vec::new(),
    })
}
