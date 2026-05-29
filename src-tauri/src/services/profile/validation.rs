//! Profile Validation
//!
//! Validates configuration profiles against the current system and tweak definitions.

use crate::error::Error;
use crate::models::{
    ChangeDetail, ChangeType, ConfigurationProfile, ErrorCode, ProfileValidation,
    TweakChangePreview, TweakDefinition, TweakSelection, ValidationError, ValidationStats,
    ValidationWarning, WarningCode, PROFILE_SCHEMA_VERSION,
};
use crate::services::backup::detect_tweak_state;
use crate::services::profile::option_content_hash_matches;
use crate::services::{scheduler_service, service_control, system_info_service};

/// Validate a profile against current system state.
pub fn validate_profile(
    profile: &ConfigurationProfile,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
) -> Result<ProfileValidation, Error> {
    validate_profile_with_admin_state(
        profile,
        available_tweaks,
        windows_version,
        system_info_service::is_running_as_admin(),
    )
}

fn validate_profile_with_admin_state(
    profile: &ConfigurationProfile,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
    is_admin: bool,
) -> Result<ProfileValidation, Error> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut preview = Vec::new();

    // Check schema version
    if profile.schema_version > PROFILE_SCHEMA_VERSION {
        errors.push(ValidationError {
            tweak_id: String::new(),
            code: ErrorCode::SchemaVersionTooNew,
            message: format!(
                "Profile schema version {} is newer than supported version {}",
                profile.schema_version, PROFILE_SCHEMA_VERSION
            ),
        });
        return Ok(ProfileValidation {
            is_valid: false,
            is_partially_applicable: false,
            warnings,
            errors,
            preview,
            stats: ValidationStats::default(),
        });
    }

    // Check Windows version mismatch
    if profile.metadata.source_windows_version != windows_version {
        warnings.push(ValidationWarning {
            tweak_id: String::new(),
            code: WarningCode::WindowsVersionMismatch,
            message: format!(
                "Profile was created on Windows {}, current system is Windows {}",
                profile.metadata.source_windows_version, windows_version
            ),
        });
    }

    // Validate each selection
    for selection in &profile.selections {
        match validate_selection(selection, available_tweaks, windows_version, is_admin) {
            SelectionResult::Valid {
                change_preview,
                selection_warnings,
            } => {
                warnings.extend(selection_warnings);
                preview.push(*change_preview);
            }
            SelectionResult::Invalid { error } => {
                errors.push(error);
            }
        }
    }

    // Calculate stats
    let stats = ValidationStats {
        total_tweaks: profile.selections.len(),
        applicable_tweaks: preview.iter().filter(|p| p.applicable).count(),
        skipped_tweaks: errors.len(),
        already_applied: preview.iter().filter(|p| p.already_applied).count(),
        tweaks_with_warnings: warnings.iter().filter(|w| !w.tweak_id.is_empty()).count(),
    };

    let is_valid = errors.is_empty();
    let is_partially_applicable = !preview.is_empty() && preview.iter().any(|p| p.applicable);

    Ok(ProfileValidation {
        is_valid,
        is_partially_applicable,
        warnings,
        errors,
        preview,
        stats,
    })
}

enum SelectionResult {
    Valid {
        change_preview: Box<TweakChangePreview>,
        selection_warnings: Vec<ValidationWarning>,
    },
    Invalid {
        error: ValidationError,
    },
}

fn validate_selection(
    selection: &TweakSelection,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
    is_admin: bool,
) -> SelectionResult {
    let mut selection_warnings = Vec::new();

    // Find the tweak
    let tweak = match available_tweaks.iter().find(|t| t.id == selection.tweak_id) {
        Some(t) => t,
        None => {
            // Try to find by alias
            if let Some(t) = available_tweaks
                .iter()
                .find(|t| t.aliases.contains(&selection.tweak_id))
            {
                selection_warnings.push(ValidationWarning {
                    tweak_id: selection.tweak_id.clone(),
                    code: WarningCode::TweakResolvedByAlias,
                    message: format!(
                        "Tweak '{}' was found via alias '{}'",
                        selection.tweak_id, t.id
                    ),
                });
                t
            } else {
                return SelectionResult::Invalid {
                    error: ValidationError {
                        tweak_id: selection.tweak_id.clone(),
                        code: ErrorCode::TweakNotFound,
                        message: format!(
                            "Tweak '{}' not found in current app version",
                            selection.tweak_id
                        ),
                    },
                };
            }
        }
    };

    // Check Windows version compatibility
    if !tweak.applies_to_version(windows_version) {
        return SelectionResult::Invalid {
            error: ValidationError {
                tweak_id: selection.tweak_id.clone(),
                code: ErrorCode::WindowsVersionIncompatible,
                message: format!(
                    "Tweak '{}' is not compatible with Windows {}",
                    tweak.name, windows_version
                ),
            },
        };
    }

    if (tweak.requires_admin || tweak.requires_system || tweak.requires_ti) && !is_admin {
        return SelectionResult::Invalid {
            error: ValidationError {
                tweak_id: selection.tweak_id.clone(),
                code: ErrorCode::InsufficientPermissions,
                message: format!("Tweak '{}' requires administrator privileges", tweak.name),
            },
        };
    }

    let resolved_option_index =
        match resolve_option_index(selection, tweak, &mut selection_warnings) {
            Ok(index) => index,
            Err(error) => return SelectionResult::Invalid { error },
        };

    // Get current tweak state
    let current_state = detect_tweak_state(tweak, windows_version).ok();
    let current_option_index = current_state.as_ref().and_then(|s| s.current_option_index);
    let current_option_label = current_option_index
        .and_then(|idx| tweak.options.get(idx))
        .map(|o| o.label.clone());

    // Check if already applied
    let already_applied = current_option_index == Some(resolved_option_index);

    if already_applied {
        selection_warnings.push(ValidationWarning {
            tweak_id: selection.tweak_id.clone(),
            code: WarningCode::AlreadyApplied,
            message: format!("Tweak '{}' is already at desired state", tweak.name),
        });
    }

    // Validate resources (services, tasks)
    let target_option = &tweak.options[resolved_option_index];
    let resource_errors = validate_option_resources(target_option, &selection.tweak_id);

    if !resource_errors.is_empty() {
        return SelectionResult::Invalid {
            error: resource_errors.into_iter().next().unwrap(),
        };
    }

    // Build change preview
    let changes = build_change_details(target_option, windows_version);

    let change_preview = TweakChangePreview {
        tweak_id: tweak.id.clone(),
        original_tweak_id: if tweak.id != selection.tweak_id {
            Some(selection.tweak_id.clone())
        } else {
            None
        },
        tweak_name: tweak.name.clone(),
        category_id: tweak.category_id.clone(),
        current_option_index,
        current_option_label,
        target_option_index: resolved_option_index,
        target_option_label: target_option.label.clone(),
        applicable: true,
        skip_reason: None,
        risk_level: tweak.risk_level.as_str().to_string(),
        already_applied,
        has_skipped_commands: false,
        changes,
    };

    SelectionResult::Valid {
        change_preview: Box::new(change_preview),
        selection_warnings,
    }
}

fn resolve_option_index(
    selection: &TweakSelection,
    tweak: &TweakDefinition,
    warnings: &mut Vec<ValidationWarning>,
) -> Result<usize, ValidationError> {
    let index_in_bounds = selection.selected_option_index < tweak.options.len();

    if let Some(stored_hash) = selection.option_content_hash.as_deref() {
        if index_in_bounds
            && option_content_hash_matches(
                &tweak.options[selection.selected_option_index],
                stored_hash,
            )
        {
            return Ok(selection.selected_option_index);
        }

        if let Some((hash_index, option)) = tweak
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| option_content_hash_matches(option, stored_hash))
        {
            warnings.push(ValidationWarning {
                tweak_id: selection.tweak_id.clone(),
                code: WarningCode::OptionResolvedByHash,
                message: format!(
                    "Option '{}' for tweak '{}' was resolved by content hash",
                    option.label, tweak.name
                ),
            });
            return Ok(hash_index);
        }

        if index_in_bounds {
            let target_option = &tweak.options[selection.selected_option_index];
            warnings.push(ValidationWarning {
                tweak_id: selection.tweak_id.clone(),
                code: WarningCode::TweakSchemaChanged,
                message: format!(
                    "Option '{}' for tweak '{}' has changed since profile was created",
                    target_option.label, tweak.name
                ),
            });
            return Ok(selection.selected_option_index);
        }
    } else if index_in_bounds {
        return Ok(selection.selected_option_index);
    }

    Err(ValidationError {
        tweak_id: selection.tweak_id.clone(),
        code: ErrorCode::InvalidOptionIndex,
        message: format!(
            "Option index {} is out of bounds (tweak has {} options)",
            selection.selected_option_index,
            tweak.options.len()
        ),
    })
}

fn validate_option_resources(
    option: &crate::models::TweakOption,
    tweak_id: &str,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check services exist
    for service in &option.service_changes {
        if service_control::get_service_status(&service.name).is_err() {
            errors.push(ValidationError {
                tweak_id: tweak_id.to_string(),
                code: ErrorCode::ServiceNotFound,
                message: format!("Service '{}' not found on this system", service.name),
            });
        }
    }

    // Check scheduler tasks exist (for exact names only)
    for task in &option.scheduler_changes {
        if let Some(ref task_name) = task.task_name {
            let state = scheduler_service::get_task_state(&task.task_path, task_name)
                .unwrap_or(scheduler_service::TaskState::NotFound);

            if state == scheduler_service::TaskState::NotFound && !task.ignore_not_found {
                errors.push(ValidationError {
                    tweak_id: tweak_id.to_string(),
                    code: ErrorCode::TaskNotFound,
                    message: format!(
                        "Scheduled task '{}\\{}' not found on this system",
                        task.task_path, task_name
                    ),
                });
            }
        }
    }

    errors
}

fn build_change_details(
    option: &crate::models::TweakOption,
    windows_version: u32,
) -> Vec<ChangeDetail> {
    let mut changes = Vec::new();

    // Registry changes
    for change in &option.registry_changes {
        if !change.applies_to_version(windows_version) {
            continue;
        }

        let description = match change.action {
            crate::models::RegistryAction::Set => {
                format!(
                    "Set {}\\{}\\{} to {:?}",
                    change.hive.as_str(),
                    change.key,
                    if change.value_name.is_empty() {
                        "(Default)"
                    } else {
                        &change.value_name
                    },
                    change.value
                )
            }
            crate::models::RegistryAction::DeleteValue => {
                format!(
                    "Delete value {}\\{}\\{}",
                    change.hive.as_str(),
                    change.key,
                    change.value_name
                )
            }
            crate::models::RegistryAction::DeleteKey => {
                format!("Delete key {}\\{}", change.hive.as_str(), change.key)
            }
            crate::models::RegistryAction::CreateKey => {
                format!("Create key {}\\{}", change.hive.as_str(), change.key)
            }
        };

        changes.push(ChangeDetail {
            change_type: ChangeType::Registry,
            description,
            current_value: None,
            new_value: change.value.as_ref().map(|v| format!("{:?}", v)),
        });
    }

    // Service changes
    for service in &option.service_changes {
        changes.push(ChangeDetail {
            change_type: ChangeType::Service,
            description: format!(
                "Set service '{}' startup to {}",
                service.name,
                service.startup.as_str()
            ),
            current_value: None,
            new_value: Some(service.startup.as_str().to_string()),
        });
    }

    // Scheduler changes
    for task in &option.scheduler_changes {
        let task_name = task
            .task_name
            .as_deref()
            .or(task.task_name_pattern.as_deref())
            .unwrap_or("*");

        changes.push(ChangeDetail {
            change_type: ChangeType::ScheduledTask,
            description: format!(
                "{} task {}\\{}",
                match task.action {
                    crate::models::SchedulerAction::Enable => "Enable",
                    crate::models::SchedulerAction::Disable => "Disable",
                    crate::models::SchedulerAction::Delete => "Delete",
                },
                task.task_path,
                task_name
            ),
            current_value: None,
            new_value: Some(task.action.as_str().to_string()),
        });
    }

    // Commands
    for cmd in &option.pre_commands {
        changes.push(ChangeDetail {
            change_type: ChangeType::Command,
            description: format!("Run: {}", cmd),
            current_value: None,
            new_value: None,
        });
    }

    for cmd in &option.post_commands {
        changes.push(ChangeDetail {
            change_type: ChangeType::Command,
            description: format!("Run: {}", cmd),
            current_value: None,
            new_value: None,
        });
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ConfigurationProfile, ProfileMetadata, RegistryAction, RegistryChange, RegistryHive,
        RegistryValueType, RiskLevel, TweakDefinition, TweakOption, TweakSelection,
    };
    use crate::services::profile::hash_option_content;

    #[test]
    fn command_backed_profile_tweaks_are_not_marked_as_skipped() {
        let tweak = TweakDefinition {
            id: "command_tweak".to_string(),
            name: "Command tweak".to_string(),
            description: "Uses command hooks".to_string(),
            info: None,
            aliases: Vec::new(),
            risk_level: RiskLevel::Low,
            requires_admin: false,
            requires_system: false,
            requires_ti: false,
            requires_reboot: false,
            force_dropdown: false,
            options: vec![
                TweakOption {
                    label: "Apply".to_string(),
                    pre_commands: vec!["echo apply".to_string()],
                    ..empty_option()
                },
                TweakOption {
                    label: "Restore".to_string(),
                    post_powershell: vec!["Write-Output restore".to_string()],
                    ..empty_option()
                },
            ],
            category_id: "test".to_string(),
        };
        let metadata =
            ProfileMetadata::new("Test".to_string(), None, "3.0.0".to_string(), 11, 22631);
        let profile = ConfigurationProfile::new(
            metadata,
            vec![TweakSelection {
                tweak_id: "command_tweak".to_string(),
                selected_option_index: 0,
                selected_option_label: "Apply".to_string(),
                option_content_hash: None,
                category_id: Some("test".to_string()),
            }],
        );

        let validation = validate_profile(&profile, &[tweak], 11).unwrap();

        assert!(validation.is_valid);
        assert!(!validation.preview[0].has_skipped_commands);
    }

    #[test]
    fn moved_option_is_resolved_by_content_hash() {
        let target_option = option_with_registry_value("Target", "TargetValue", 1);
        let stored_hash = hash_option_content(&target_option);
        let tweak = tweak_with_options(vec![
            option_with_registry_value("Other", "OtherValue", 2),
            target_option,
        ]);
        let profile = profile_with_selection(TweakSelection {
            tweak_id: "profile_tweak".to_string(),
            selected_option_index: 0,
            selected_option_label: "Target".to_string(),
            option_content_hash: Some(stored_hash),
            category_id: Some("test".to_string()),
        });

        let validation = validate_profile_with_admin_state(&profile, &[tweak], 11, true).unwrap();

        assert!(validation.is_valid);
        assert_eq!(validation.preview[0].target_option_index, 1);
        assert!(validation
            .warnings
            .iter()
            .any(|warning| warning.code == WarningCode::OptionResolvedByHash));
    }

    #[test]
    fn admin_required_tweak_is_invalid_without_admin_privileges() {
        let mut tweak = tweak_with_options(vec![
            option_with_registry_value("Apply", "AdminValue", 1),
            option_with_registry_value("Restore", "AdminValue", 0),
        ]);
        tweak.requires_admin = true;
        let profile = profile_with_selection(TweakSelection {
            tweak_id: "profile_tweak".to_string(),
            selected_option_index: 0,
            selected_option_label: "Apply".to_string(),
            option_content_hash: None,
            category_id: Some("test".to_string()),
        });

        let validation = validate_profile_with_admin_state(&profile, &[tweak], 11, false).unwrap();

        assert!(!validation.is_valid);
        assert_eq!(
            validation.errors[0].code,
            ErrorCode::InsufficientPermissions
        );
    }

    fn empty_option() -> TweakOption {
        TweakOption {
            label: String::new(),
            registry_changes: Vec::new(),
            service_changes: Vec::new(),
            scheduler_changes: Vec::new(),
            hosts_changes: Vec::new(),
            firewall_changes: Vec::new(),
            pre_commands: Vec::new(),
            post_commands: Vec::new(),
            pre_powershell: Vec::new(),
            post_powershell: Vec::new(),
            registry_missing_is_match: false,
            service_missing_is_match: false,
            scheduler_missing_is_match: false,
        }
    }

    fn profile_with_selection(selection: TweakSelection) -> ConfigurationProfile {
        let metadata =
            ProfileMetadata::new("Test".to_string(), None, "3.0.0".to_string(), 11, 22631);
        ConfigurationProfile::new(metadata, vec![selection])
    }

    fn tweak_with_options(options: Vec<TweakOption>) -> TweakDefinition {
        TweakDefinition {
            id: "profile_tweak".to_string(),
            name: "Profile tweak".to_string(),
            description: "Profile validation test tweak".to_string(),
            info: None,
            aliases: Vec::new(),
            risk_level: RiskLevel::Low,
            requires_admin: false,
            requires_system: false,
            requires_ti: false,
            requires_reboot: false,
            force_dropdown: false,
            options,
            category_id: "test".to_string(),
        }
    }

    fn option_with_registry_value(label: &str, value_name: &str, value: u32) -> TweakOption {
        TweakOption {
            label: label.to_string(),
            registry_changes: vec![RegistryChange {
                hive: RegistryHive::Hkcu,
                key: "Software\\MagicXTest".to_string(),
                value_name: value_name.to_string(),
                action: RegistryAction::Set,
                value_type: Some(RegistryValueType::Dword),
                value: Some(serde_json::json!(value)),
                windows_versions: None,
                skip_validation: true,
            }],
            ..empty_option()
        }
    }
}
