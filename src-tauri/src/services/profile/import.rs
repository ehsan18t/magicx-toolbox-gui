//! Profile Import/Apply
//!
//! Import and apply configuration profiles.

use std::path::Path;

use crate::error::Error;
use crate::models::{
    ApplyFailure, ApplyOptions, ConfigurationProfile, ProfileApplyResult, ProfileValidation,
    TweakDefinition,
};
use crate::services::backup::{
    capture_snapshot, load_snapshot, restore_from_snapshot, save_snapshot,
};
use crate::services::profile::archive::read_profile_archive;
use crate::services::profile::validation::validate_profile;
use crate::services::{registry_service, scheduler_service, service_control};

/// Import and validate a profile from a file.
pub fn import_profile(
    path: &Path,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
) -> Result<(ConfigurationProfile, ProfileValidation), Error> {
    log::info!("Importing profile from {}", path.display());

    // Read the archive
    let mut contents = read_profile_archive(path)?;

    // Migrate if needed
    let migration_notes =
        crate::services::profile::migration::migrate_profile(&mut contents.profile)?;

    if !migration_notes.is_empty() {
        log::info!(
            "Profile migrated with {} notes: {:?}",
            migration_notes.len(),
            migration_notes
        );
    }

    // Validate
    let validation = validate_profile(&contents.profile, available_tweaks, windows_version)?;

    Ok((contents.profile, validation))
}

/// Apply a profile to the system.
pub fn apply_profile(
    profile: &ConfigurationProfile,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
    options: &ApplyOptions,
) -> Result<ProfileApplyResult, Error> {
    log::info!("Applying profile '{}'", profile.metadata.name);

    // First validate
    let validation = validate_profile(profile, available_tweaks, windows_version)?;

    let mut applied_count = 0;
    let mut skipped_count = 0;
    let mut failures = Vec::new();
    let mut backup_tweak_ids = Vec::new();
    let mut reboot_required_tweaks = Vec::new();

    for preview in &validation.preview {
        // Skip if in skip list
        if options.skip_tweak_ids.contains(&preview.tweak_id) {
            log::debug!("Skipping tweak '{}' (in skip list)", preview.tweak_id);
            skipped_count += 1;
            continue;
        }

        // Skip if already applied and option is set
        if preview.already_applied && options.skip_already_applied {
            log::debug!(
                "Skipping tweak '{}' (already at target state)",
                preview.tweak_id
            );
            skipped_count += 1;
            continue;
        }

        // Skip if not applicable
        if !preview.applicable {
            log::debug!(
                "Skipping tweak '{}' (not applicable: {:?})",
                preview.tweak_id,
                preview.skip_reason
            );
            skipped_count += 1;
            continue;
        }

        // Find the tweak definition
        let tweak = match available_tweaks.iter().find(|t| t.id == preview.tweak_id) {
            Some(t) => t,
            None => {
                log::warn!("Tweak '{}' not found during apply", preview.tweak_id);
                skipped_count += 1;
                continue;
            }
        };

        // Create backup if requested
        if options.create_restore_point {
            match capture_snapshot(tweak, preview.target_option_index, windows_version, None) {
                Ok(snapshot) => {
                    if let Err(e) = save_snapshot(&snapshot) {
                        log::warn!("Failed to save backup for '{}': {}", tweak.id, e);
                    } else {
                        backup_tweak_ids.push(tweak.id.clone());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to create backup for '{}': {}", tweak.id, e);
                }
            }
        }

        // Apply the tweak - with bounds check
        let option = match tweak.options.get(preview.target_option_index) {
            Some(opt) => opt,
            None => {
                log::error!(
                    "Option index {} out of bounds for tweak '{}' (has {} options)",
                    preview.target_option_index,
                    tweak.id,
                    tweak.options.len()
                );
                failures.push(ApplyFailure {
                    tweak_id: tweak.id.clone(),
                    tweak_name: tweak.name.clone(),
                    error: format!(
                        "Option index {} is out of bounds",
                        preview.target_option_index
                    ),
                    was_rolled_back: false,
                });
                continue;
            }
        };
        match apply_tweak_changes(tweak, option, windows_version) {
            Ok(()) => {
                log::info!(
                    "Applied tweak '{}' option '{}' successfully",
                    tweak.id,
                    option.label
                );
                applied_count += 1;

                // Track if this tweak requires reboot
                if tweak.requires_reboot {
                    reboot_required_tweaks.push(tweak.id.clone());
                }
            }
            Err(e) => {
                log::error!("Failed to apply tweak '{}': {}", tweak.id, e);

                // Try to rollback if we have a backup
                let was_rolled_back = if backup_tweak_ids.contains(&tweak.id) {
                    if let Ok(Some(snapshot)) = load_snapshot(&tweak.id) {
                        match restore_from_snapshot(&snapshot) {
                            Ok(_) => {
                                log::info!("Rolled back tweak '{}'", tweak.id);
                                true
                            }
                            Err(rb_err) => {
                                log::error!("Failed to rollback '{}': {}", tweak.id, rb_err);
                                false
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                failures.push(ApplyFailure {
                    tweak_id: tweak.id.clone(),
                    tweak_name: tweak.name.clone(),
                    error: e.to_string(),
                    was_rolled_back,
                });
            }
        }
    }

    let success = failures.is_empty();
    let requires_reboot = !reboot_required_tweaks.is_empty();

    Ok(ProfileApplyResult {
        success,
        applied_count,
        skipped_count,
        failed_count: failures.len(),
        failures,
        requires_reboot,
        reboot_required_tweaks,
    })
}

/// Apply tweak changes (registry, services, scheduler).
fn apply_tweak_changes(
    tweak: &TweakDefinition,
    option: &crate::models::TweakOption,
    windows_version: u32,
) -> Result<(), Error> {
    // Apply registry changes
    for change in &option.registry_changes {
        if !change.applies_to_version(windows_version) {
            continue;
        }

        match change.action {
            crate::models::RegistryAction::Set => {
                let value_type = change.value_type.as_ref().ok_or_else(|| {
                    Error::ValidationError("Set action requires value_type".into())
                })?;
                let value = change
                    .value
                    .as_ref()
                    .ok_or_else(|| Error::ValidationError("Set action requires value".into()))?;

                write_registry_value(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    value_type,
                    value,
                )?;
            }
            crate::models::RegistryAction::DeleteValue => {
                // Ignore not found errors
                let _ =
                    registry_service::delete_value(&change.hive, &change.key, &change.value_name);
            }
            crate::models::RegistryAction::DeleteKey => {
                // Ignore not found errors
                let _ = registry_service::delete_key(&change.hive, &change.key);
            }
            crate::models::RegistryAction::CreateKey => {
                registry_service::create_key(&change.hive, &change.key)?;
            }
        }
    }

    // Apply service changes
    for service_change in &option.service_changes {
        service_control::set_service_startup(&service_change.name, &service_change.startup)?;
    }

    // Apply scheduler changes
    for task_change in &option.scheduler_changes {
        if let Some(ref task_name) = task_change.task_name {
            match task_change.action {
                crate::models::SchedulerAction::Enable => {
                    if let Err(e) =
                        scheduler_service::enable_task(&task_change.task_path, task_name)
                    {
                        log::warn!(
                            "Failed to enable task '{}\\{}': {}",
                            task_change.task_path,
                            task_name,
                            e
                        );
                    }
                }
                crate::models::SchedulerAction::Disable => {
                    if let Err(e) =
                        scheduler_service::disable_task(&task_change.task_path, task_name)
                    {
                        log::warn!(
                            "Failed to disable task '{}\\{}': {}",
                            task_change.task_path,
                            task_name,
                            e
                        );
                    }
                }
                crate::models::SchedulerAction::Delete => {
                    if let Err(e) =
                        scheduler_service::delete_task(&task_change.task_path, task_name)
                    {
                        log::warn!(
                            "Failed to delete task '{}\\{}': {}",
                            task_change.task_path,
                            task_name,
                            e
                        );
                    }
                }
            }
        }
    }

    // Note: Commands are not run in profile apply to avoid security issues
    // Users should be aware that pre/post commands won't execute
    if !option.pre_commands.is_empty() || !option.post_commands.is_empty() {
        log::warn!(
            "Tweak '{}' has commands that won't be executed in profile apply",
            tweak.id
        );
    }

    Ok(())
}

/// Write a registry value.
fn write_registry_value(
    hive: &crate::models::RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &crate::models::RegistryValueType,
    value: &serde_json::Value,
) -> Result<(), Error> {
    use crate::models::RegistryValueType;

    match value_type {
        RegistryValueType::Dword => {
            let v = value.as_u64().ok_or_else(|| {
                Error::ValidationError(format!("Expected u64 for DWORD, got: {}", value))
            })?;
            registry_service::set_dword(hive, key, value_name, v as u32)?;
        }
        RegistryValueType::String | RegistryValueType::ExpandString => {
            let v = value.as_str().ok_or_else(|| {
                Error::ValidationError(format!(
                    "Expected string for {}, got: {}",
                    value_type.as_str(),
                    value
                ))
            })?;
            registry_service::set_string(hive, key, value_name, v)?;
        }
        RegistryValueType::Binary => {
            let arr = value.as_array().ok_or_else(|| {
                Error::ValidationError(format!("Expected array for BINARY, got: {}", value))
            })?;
            let binary: Vec<u8> = arr
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u8))
                .collect();
            registry_service::set_binary(hive, key, value_name, &binary)?;
        }
        RegistryValueType::Qword => {
            let v = value.as_u64().ok_or_else(|| {
                Error::ValidationError(format!("Expected u64 for QWORD, got: {}", value))
            })?;
            registry_service::set_qword(hive, key, value_name, v)?;
        }
        RegistryValueType::MultiString => {
            return Err(Error::ValidationError(
                "MultiString registry values are not supported".into(),
            ));
        }
    }

    Ok(())
}
