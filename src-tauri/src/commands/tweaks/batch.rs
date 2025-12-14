//! Batch Commands - Batch apply/revert operations for multiple tweaks

use super::apply::{apply_tweak, revert_tweak};
use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::TweakResult;
use crate::services::system_info_service;
use tauri::AppHandle;

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
    let mut partial_success_count = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for (tweak_id, option_index) in &operations {
        let result = Box::pin(apply_tweak(app.clone(), tweak_id.clone(), *option_index)).await;

        match result {
            Ok(res) => {
                if res.success {
                    success_count += 1;
                } else {
                    // Partial success - apply rolled back but record failure
                    partial_success_count += 1;
                    // Collect inner failures
                    for (id, msg) in res.failures {
                        failures.push((id, msg));
                    }
                    if failures.is_empty() {
                        // No inner failures but still failed
                        failures.push((tweak_id.clone(), res.message));
                    }
                }
                if res.requires_reboot {
                    requires_reboot = true;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                log::warn!(
                    "Failed to apply tweak '{}' option {}: {}",
                    tweak_id,
                    option_index,
                    error_msg
                );
                failures.push((tweak_id.clone(), error_msg));
            }
        }
    }

    let failure_count = failures.len();
    let message = if failure_count > 0 {
        format!(
            "Applied {}/{} tweaks ({} failed, {} partial)",
            success_count,
            operations.len(),
            failure_count,
            partial_success_count
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
        failures,
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
    let mut partial_success_count = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for tweak_id in &tweak_ids {
        let result = Box::pin(revert_tweak(app.clone(), tweak_id.clone())).await;

        match result {
            Ok(res) => {
                if res.success {
                    success_count += 1;
                } else {
                    // Partial success - some operations failed, snapshot kept for retry
                    partial_success_count += 1;
                    // Collect inner failures
                    for (id, msg) in res.failures {
                        failures.push((id, msg));
                    }
                    if failures.is_empty() {
                        // No inner failures but still failed
                        failures.push((tweak_id.clone(), res.message));
                    }
                }
                if res.requires_reboot {
                    requires_reboot = true;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                log::warn!("Failed to revert tweak '{}': {}", tweak_id, error_msg);
                failures.push((tweak_id.clone(), error_msg));
            }
        }
    }

    let failure_count = failures.len();
    let message = if failure_count > 0 {
        format!(
            "Reverted {}/{} tweaks ({} failed, {} partial)",
            success_count,
            tweak_ids.len(),
            failure_count,
            partial_success_count
        )
    } else {
        format!("Reverted {} tweaks", success_count)
    };

    Ok(TweakResult {
        success: failure_count == 0,
        message,
        requires_reboot,
        failures,
    })
}
