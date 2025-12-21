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
use crate::services::{scheduler_service, service_control};

/// Validate a profile against current system state.
pub fn validate_profile(
    profile: &ConfigurationProfile,
    available_tweaks: &[TweakDefinition],
    windows_version: u32,
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
        match validate_selection(selection, available_tweaks, windows_version) {
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
) -> SelectionResult {
    let mut selection_warnings = Vec::new();

    // Find the tweak
    let tweak = match available_tweaks.iter().find(|t| t.id == selection.tweak_id) {
        Some(t) => t,
        None => {
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

    // Resolve option index
    let resolved_option_index = if selection.selected_option_index < tweak.options.len() {
        selection.selected_option_index
    } else {
        return SelectionResult::Invalid {
            error: ValidationError {
                tweak_id: selection.tweak_id.clone(),
                code: ErrorCode::InvalidOptionIndex,
                message: format!(
                    "Option index {} is out of bounds (tweak has {} options)",
                    selection.selected_option_index,
                    tweak.options.len()
                ),
            },
        };
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
        changes,
    };

    SelectionResult::Valid {
        change_preview: Box::new(change_preview),
        selection_warnings,
    }
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
