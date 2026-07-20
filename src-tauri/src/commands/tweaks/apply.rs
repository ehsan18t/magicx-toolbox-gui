//! Apply Commands - Single tweak apply/revert operations

use super::helpers::{apply_all_changes_atomically, run_command, run_powershell_command};
use crate::debug::{emit_debug_log, is_debug_enabled, DebugLevel};
use crate::error::{Error, Result};
use crate::models::TweakResult;
use crate::services::{backup_service, system_info_service, tweak_loader};

/// Outcome of the automatic rollback that follows a failed apply.
///
/// Rollback is itself a sequence of Windows operations that can fail. The
/// difference between "the machine is back where it started" and "the machine is
/// half-changed and the snapshot is the only way back" is decided entirely here,
/// so the decision is kept separate from the I/O and unit-tested.
///
/// See `docs/adr/0001-rollback-failure-is-a-first-class-state.md` and
/// `docs/adr/0002-snapshot-deletion-requires-verification-or-consent.md`.
#[derive(Debug, PartialEq, Eq)]
enum RollbackOutcome {
    /// Nothing had been captured, so there was nothing to roll back to.
    NothingToRollBack,
    /// Every captured resource was verifiably restored.
    Verified,
    /// The machine may be partly changed. These operations could not be restored.
    Incomplete(Vec<String>),
}

/// Classify a rollback attempt. `None` means no rollback was attempted.
///
/// Only `Verified` and `NothingToRollBack` permit releasing a snapshot.
fn classify_rollback(rollback: Option<Result<backup_service::RestoreResult>>) -> RollbackOutcome {
    match rollback {
        None => RollbackOutcome::NothingToRollBack,
        Some(Ok(result)) if result.success => RollbackOutcome::Verified,
        // `success` is derived from `failures.is_empty()`, but do not lean on that
        // invariant: a restore reporting failure without detail is still not verified.
        Some(Ok(result)) if result.failures.is_empty() => {
            RollbackOutcome::Incomplete(vec!["restore reported failure without detail".into()])
        }
        Some(Ok(result)) => RollbackOutcome::Incomplete(result.failures),
        // A hard error aborts the restore partway -- the registry phase returns Err
        // and the service/scheduler/hosts/firewall phases never run -- so this is
        // strictly worse than a collected per-item failure.
        Some(Err(e)) => RollbackOutcome::Incomplete(vec![format!("restore aborted: {}", e)]),
    }
}

/// Apply a specific option for a tweak
///
/// For toggle tweaks (is_toggle: true):
/// - option_index 0 = first option (usually "Enabled" or "On")
/// - option_index 1 = second option (usually "Disabled" or "Off")
///
/// For dropdown tweaks (is_toggle: false):
/// - option_index corresponds to the options array index
#[tauri::command]
pub async fn apply_tweak(tweak_id: String, option_index: usize) -> Result<TweakResult> {
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
    let runtime = system_info_service::get_runtime_context()?;
    let version = runtime.windows_version();

    log::debug!(
        "Applying option '{}' for '{}' on Windows {}",
        option.label,
        tweak.name,
        version
    );

    // Check admin if required
    if tweak.requires_admin && !runtime.is_admin {
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
        // Capture original state - pass current_option_index so we know if original was unknown
        let original_option_index = current_state.current_option_index;
        let snapshot =
            backup_service::capture_snapshot(&tweak, option_index, version, original_option_index)?;
        backup_service::save_snapshot(&snapshot)?;
        log::info!(
            "Captured original snapshot for '{}' with {} registry values, {} services (original_option_index={:?})",
            tweak.name,
            snapshot.registry_snapshots.len(),
            snapshot.service_snapshots.len(),
            original_option_index
        );
        None
    };

    // Step 2: Run pre_commands if defined (non-reversible, fail-fast)
    for cmd in &option.pre_commands {
        if let Err(e) = run_command(cmd, tweak.elevation()) {
            log::error!("Pre-command failed, aborting: {}", e);
            if !is_switching_options {
                if let Err(del_err) = backup_service::delete_snapshot(&tweak_id) {
                    log::warn!(
                        "Failed to delete snapshot for '{}' after pre-command failure: {}",
                        tweak_id,
                        del_err
                    );
                }
            }
            return Err(Error::CommandExecution(format!(
                "Pre-command failed: {}",
                e
            )));
        }
    }

    // Step 3: Run pre_powershell if defined (non-reversible, fail-fast)
    for ps_cmd in &option.pre_powershell {
        if let Err(e) = run_powershell_command(ps_cmd, tweak.elevation()) {
            log::error!("Pre-PowerShell command failed, aborting: {}", e);
            if !is_switching_options {
                if let Err(del_err) = backup_service::delete_snapshot(&tweak_id) {
                    log::warn!(
                        "Failed to delete snapshot for '{}' after pre-PowerShell failure: {}",
                        tweak_id,
                        del_err
                    );
                }
            }
            return Err(Error::CommandExecution(format!(
                "Pre-PowerShell failed: {}",
                e
            )));
        }
    }

    // Steps 4-6: Apply all core changes ATOMICALLY
    if let Err(e) = apply_all_changes_atomically(&tweak, option, version) {
        log::error!("Failed to apply changes for '{}': {}", tweak.name, e);

        // Roll back based on context. The result is deliberately NOT discarded:
        // rollback can itself fail, and when it does the machine is left partly
        // changed with the snapshot as the only remaining route back (ADR-0002).
        let rollback = match pre_apply_state {
            Some(ref previous_option_state) => {
                log::warn!("Rolling back to previous option state (switching options failed)...");
                Some(backup_service::restore_from_snapshot(previous_option_state))
            }
            None => backup_service::load_snapshot(&tweak_id)?.map(|snapshot| {
                log::warn!("Rolling back ALL changes to original state (first apply failed)...");
                backup_service::restore_from_snapshot(&snapshot)
            }),
        };

        let rollback_failures = match classify_rollback(rollback) {
            RollbackOutcome::NothingToRollBack | RollbackOutcome::Verified => {
                // The machine is provably back at its pre-apply state. On a first
                // apply the snapshot now describes a state we are no longer in, so
                // it is safe to release. A delete failure must never mask the real
                // apply error, so it is logged rather than propagated with `?`.
                if !is_switching_options {
                    if let Err(del_err) = backup_service::delete_snapshot(&tweak_id) {
                        log::warn!(
                            "Failed to delete snapshot for '{}' after a verified rollback: {}",
                            tweak_id,
                            del_err
                        );
                    }
                }
                return Err(e);
            }
            RollbackOutcome::Incomplete(failures) => failures,
        };

        // Rollback did not fully succeed. Keep the snapshot -- switching options
        // preserves the original by design, and on a first apply it is now the only
        // way back to the user's original state.
        log::error!(
            "Rollback for '{}' was incomplete: {} operation(s) could not be restored. \
             Snapshot kept so the revert can be retried.",
            tweak.name,
            rollback_failures.len()
        );

        if is_debug_enabled() {
            emit_debug_log(
                DebugLevel::Error,
                &format!("Rollback incomplete: {}", tweak.name),
                Some(&format!(
                    "{} operation(s) not restored - snapshot kept for retry",
                    rollback_failures.len()
                )),
            );
        }

        // Report the apply failure AND every resource left in a changed state.
        // Returning Ok(success: false) mirrors revert_tweak's partial-failure shape
        // so the UI can surface the detail instead of a bare error string.
        let incomplete_count = rollback_failures.len();
        let mut failures: Vec<(String, String)> =
            vec![(tweak_id.clone(), format!("apply failed: {}", e))];
        failures.extend(
            rollback_failures
                .into_iter()
                .map(|msg| (tweak_id.clone(), format!("rollback: {}", msg))),
        );

        return Ok(TweakResult {
            success: false,
            message: format!(
                "Apply failed and rollback was incomplete: {} operation(s) could not be \
                 restored. The snapshot has been kept so you can retry reverting.",
                incomplete_count
            ),
            requires_reboot: false,
            failures,
        });
    }

    // Step 7: If switching options succeeded, update the snapshot metadata
    if is_switching_options {
        backup_service::update_snapshot_metadata(&tweak_id, option_index, &option.label)?;
    }

    // Step 8: Run post_commands (non-fatal, no rollback)
    for cmd in &option.post_commands {
        if let Err(e) = run_command(cmd, tweak.elevation()) {
            log::warn!("Post-command failed (non-fatal): {}", e);
        }
    }

    // Step 9: Run post_powershell (non-fatal, no rollback)
    for ps_cmd in &option.post_powershell {
        if let Err(e) = run_powershell_command(ps_cmd, tweak.elevation()) {
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
pub async fn revert_tweak(tweak_id: String) -> Result<TweakResult> {
    log::info!("Command: revert_tweak({})", tweak_id);

    let tweak = tweak_loader::get_tweak(&tweak_id)?.ok_or_else(|| {
        log::error!("Tweak not found: {}", tweak_id);
        Error::NotFound(format!("Tweak '{}'", tweak_id))
    })?;

    let runtime = system_info_service::get_runtime_context()?;

    // Check admin if required
    if tweak.requires_admin && !runtime.is_admin {
        log::warn!("Tweak '{}' requires admin, but running as user", tweak.name);
        return Err(Error::RequiresAdmin);
    }

    // Load snapshot
    let snapshot = backup_service::load_snapshot(&tweak_id)?
        .ok_or_else(|| Error::BackupFailed("No snapshot found for this tweak".into()))?;

    log::info!(
        "Reverting '{}' from option '{}' (snapshot from {}, requires_system={})",
        tweak.name,
        snapshot.applied_option_label,
        snapshot.created_at,
        snapshot.requires_system
    );

    if is_debug_enabled() {
        emit_debug_log(
            DebugLevel::Info,
            &format!("Reverting: {}", tweak.name),
            Some(&format!(
                "{} registry, {} services, {} tasks",
                snapshot.registry_snapshots.len(),
                snapshot.service_snapshots.len(),
                snapshot.scheduler_snapshots.len()
            )),
        );
    }

    // Restore from snapshot - now returns RestoreResult with failure details
    let restore_result = backup_service::restore_from_snapshot(&snapshot)?;

    // Only delete snapshot if ALL operations succeeded
    // This allows the user to retry the revert if some operations failed
    if restore_result.success {
        backup_service::delete_snapshot(&tweak_id)?;
        log::info!(
            "Successfully reverted '{}' (snapshot deleted){}",
            tweak.name,
            if tweak.requires_reboot {
                " - reboot required"
            } else {
                ""
            }
        );

        if is_debug_enabled() {
            emit_debug_log(
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
    } else {
        // Partial success - some operations failed but snapshot is kept for retry
        log::warn!(
            "Partial revert for '{}': {} failures (snapshot kept for retry)",
            tweak.name,
            restore_result.failures.len()
        );

        if is_debug_enabled() {
            emit_debug_log(
                DebugLevel::Warn,
                &format!("Partial revert: {}", tweak.name),
                Some(&format!(
                    "{} failures - snapshot kept for retry",
                    restore_result.failures.len()
                )),
            );
        }

        // Convert failures to (tweak_id, error) format for TweakResult
        let failures: Vec<(String, String)> = restore_result
            .failures
            .into_iter()
            .map(|msg| (tweak_id.clone(), msg))
            .collect();

        // Return partial success with failure details
        // The snapshot is preserved so user can retry
        Ok(TweakResult {
            success: false,
            message: format!(
                "Partial revert: {} operations failed. Snapshot kept for retry.",
                failures.len()
            ),
            requires_reboot: tweak.requires_reboot,
            failures,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::backup_service::RestoreResult;

    /// Regression guard. `apply_tweak` used to do:
    ///
    /// ```ignore
    /// let _ = backup_service::restore_from_snapshot(&snapshot);
    /// backup_service::delete_snapshot(&tweak_id)?;
    /// ```
    ///
    /// discarding the rollback result and deleting the snapshot unconditionally.
    /// A partially-failed rollback therefore left the machine half-changed with the
    /// user's original state gone and no indication anything was wrong.
    ///
    /// Every case below that is NOT `Verified` / `NothingToRollBack` must keep the
    /// snapshot. That is the whole contract of ADR-0002.
    #[test]
    fn only_a_verified_rollback_permits_releasing_the_snapshot() {
        let releasable = |outcome: &RollbackOutcome| {
            matches!(
                outcome,
                RollbackOutcome::NothingToRollBack | RollbackOutcome::Verified
            )
        };

        assert!(releasable(&classify_rollback(None)));
        assert!(releasable(&classify_rollback(Some(Ok(RestoreResult {
            success: true,
            failures: vec![],
        })))));

        assert!(!releasable(&classify_rollback(Some(Ok(RestoreResult {
            success: false,
            failures: vec!["service DiagTrack could not be restored".into()],
        })))));
        assert!(!releasable(&classify_rollback(Some(Err(
            crate::error::Error::BackupFailed("registry restore aborted".into())
        )))));
    }

    #[test]
    fn no_rollback_attempted_is_not_an_incomplete_rollback() {
        assert_eq!(classify_rollback(None), RollbackOutcome::NothingToRollBack);
    }

    #[test]
    fn a_clean_restore_is_verified() {
        assert_eq!(
            classify_rollback(Some(Ok(RestoreResult {
                success: true,
                failures: vec![],
            }))),
            RollbackOutcome::Verified
        );
    }

    #[test]
    fn collected_restore_failures_are_reported_verbatim() {
        assert_eq!(
            classify_rollback(Some(Ok(RestoreResult {
                success: false,
                failures: vec!["service Spooler".into(), "task ScheduleScan".into()],
            }))),
            RollbackOutcome::Incomplete(vec!["service Spooler".into(), "task ScheduleScan".into()])
        );
    }

    /// `success` is derived from `failures.is_empty()`, so this state should not
    /// occur -- but treating an unexplained failure as "verified" would delete the
    /// snapshot, so it must still resolve to `Incomplete`.
    #[test]
    fn failure_without_detail_is_still_incomplete() {
        let outcome = classify_rollback(Some(Ok(RestoreResult {
            success: false,
            failures: vec![],
        })));
        match outcome {
            RollbackOutcome::Incomplete(failures) => assert_eq!(failures.len(), 1),
            other => panic!("expected Incomplete, got {:?}", other),
        }
    }

    /// A hard `Err` is worse than collected failures: the registry phase returns
    /// early and the service/scheduler/hosts/firewall phases never run at all.
    #[test]
    fn an_aborted_restore_is_incomplete_and_says_so() {
        let outcome = classify_rollback(Some(Err(crate::error::Error::BackupFailed(
            "hive locked".into(),
        ))));
        match outcome {
            RollbackOutcome::Incomplete(failures) => {
                assert_eq!(failures.len(), 1);
                assert!(
                    failures[0].contains("restore aborted"),
                    "message should identify this as an aborted restore, got: {}",
                    failures[0]
                );
                assert!(failures[0].contains("hive locked"), "underlying cause lost");
            }
            other => panic!("expected Incomplete, got {:?}", other),
        }
    }
}
