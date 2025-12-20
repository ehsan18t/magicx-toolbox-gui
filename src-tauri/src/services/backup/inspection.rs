use super::capture::read_registry_value;
use super::helpers::values_match;
use crate::error::Error;
use crate::models::{
    OptionInspection, RegistryAction, RegistryMismatch, SchedulerAction, SchedulerMismatch,
    ServiceMismatch, TweakDefinition, TweakInspection, TweakOption,
};
use crate::services::{registry_service, scheduler_service, service_control};
use rayon::prelude::*;

/// Inspect a tweak to find exact system state vs expected state for all options
pub fn inspect_tweak(
    tweak: &TweakDefinition,
    windows_version: u32,
    current_option_index: Option<usize>,
    pending_option_index: Option<usize>,
) -> Result<TweakInspection, Error> {
    // Inspect each option in parallel
    let options: Vec<OptionInspection> = tweak
        .options
        .par_iter()
        .enumerate()
        .map(|(index, option)| {
            inspect_option(
                index,
                option,
                windows_version,
                current_option_index == Some(index),
                pending_option_index == Some(index),
            )
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let matched_option_index = options.iter().position(|opt| opt.all_match);

    Ok(TweakInspection {
        tweak_id: tweak.id.clone(),
        options,
        matched_option_index,
    })
}

fn inspect_option(
    index: usize,
    option: &TweakOption,
    windows_version: u32,
    is_current: bool,
    is_pending: bool,
) -> Result<OptionInspection, Error> {
    // Check registry changes
    let registry_results = inspect_registry_changes(option, windows_version)?;

    // Check service changes
    let service_results = inspect_service_changes(option)?;

    // Check scheduler changes
    let scheduler_results = inspect_scheduler_changes(option)?;

    // Determine if everything matches
    let all_match =
        calculate_overall_match(&registry_results, &service_results, &scheduler_results);

    Ok(OptionInspection {
        option_index: index,
        label: option.label.clone(),
        is_current,
        is_pending,
        registry_results,
        service_results,
        scheduler_results,
        all_match,
    })
}

fn calculate_overall_match(
    registry_results: &[RegistryMismatch],
    service_results: &[ServiceMismatch],
    scheduler_results: &[SchedulerMismatch],
) -> bool {
    registry_results
        .iter()
        .filter(|r| !r.skip_validation)
        .all(|r| r.is_match)
        && service_results
            .iter()
            .filter(|s| !s.skip_validation)
            .all(|s| s.is_match)
        && scheduler_results
            .iter()
            .filter(|s| !s.skip_validation)
            .all(|s| s.is_match)
}

fn inspect_registry_changes(
    option: &TweakOption,
    windows_version: u32,
) -> Result<Vec<RegistryMismatch>, Error> {
    let mut results = Vec::new();

    for change in &option.registry_changes {
        // Skip changes not relevant to this Windows version
        if !change.applies_to_version(windows_version) {
            continue;
        }

        // We also inspect items with skip_validation, marking them as informative

        let path = format!("{}\\{}", change.hive.as_str(), change.key);
        let value_str = if change.value_name.is_empty() {
            "(Default)"
        } else {
            &change.value_name
        };

        match change.action {
            RegistryAction::Set => {
                let value_type = match &change.value_type {
                    Some(vt) => vt,
                    None => continue,
                };
                let expected_val = match &change.value {
                    Some(v) => v,
                    None => continue,
                };

                // Read current
                let (current_val, exists) =
                    read_registry_value(&change.hive, &change.key, &change.value_name, value_type)?;

                let is_match = exists && values_match(&current_val, &Some(expected_val.clone()));

                results.push(RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: change.value_name.clone(),
                    expected_value: Some(expected_val.clone()),
                    actual_value: if exists { current_val } else { None },
                    value_type: Some(value_type.as_str().to_string()),
                    description: format!("Set {} to {:?}", value_str, expected_val),
                    is_match,
                    skip_validation: change.skip_validation,
                });
            }
            RegistryAction::DeleteValue => {
                let exists =
                    registry_service::value_exists(&change.hive, &change.key, &change.value_name)
                        .unwrap_or(false);

                results.push(RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: change.value_name.clone(),
                    expected_value: None, // Expected: None/Deleted
                    actual_value: if exists {
                        Some(serde_json::json!("Exists"))
                    } else {
                        None
                    },
                    value_type: None,
                    description: format!("Delete value {}", value_str),
                    is_match: !exists,
                    skip_validation: change.skip_validation,
                });
            }
            RegistryAction::DeleteKey => {
                let exists =
                    registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);

                results.push(RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: String::new(),
                    expected_value: None,
                    actual_value: if exists {
                        Some(serde_json::json!("Exists"))
                    } else {
                        None
                    },
                    value_type: None,
                    description: format!("Delete key {}", path),
                    is_match: !exists,
                    skip_validation: change.skip_validation,
                });
            }
            RegistryAction::CreateKey => {
                let exists =
                    registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);

                results.push(RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: String::new(),
                    expected_value: Some(serde_json::json!("Exists")),
                    actual_value: if exists {
                        Some(serde_json::json!("Exists"))
                    } else {
                        None
                    },
                    value_type: None,
                    description: format!("Create key {}", path),
                    is_match: exists,
                    skip_validation: change.skip_validation,
                });
            }
        }
    }

    Ok(results)
}

fn inspect_service_changes(option: &TweakOption) -> Result<Vec<ServiceMismatch>, Error> {
    let mut results = Vec::new();

    for change in &option.service_changes {
        let status = service_control::get_service_status(&change.name).ok();
        let current_startup = status.as_ref().map(|s| s.startup_type).flatten();

        let expected_startup = change.startup;
        let is_match = current_startup == Some(expected_startup);

        results.push(ServiceMismatch {
            name: change.name.clone(),
            expected_startup: format!("{:?}", expected_startup),
            actual_startup: current_startup.map(|s| format!("{:?}", s)),
            description: format!("Set startup to {:?}", expected_startup),
            is_match,
            skip_validation: change.skip_validation,
        });
    }

    Ok(results)
}

fn inspect_scheduler_changes(option: &TweakOption) -> Result<Vec<SchedulerMismatch>, Error> {
    let mut results = Vec::new();

    for change in &option.scheduler_changes {
        // Handle pattern-based task matching
        if let Some(pattern) = &change.task_name_pattern {
            // For patterns, we need to list matching tasks and check each one
            let matching_tasks =
                scheduler_service::find_tasks_by_pattern(&change.task_path, pattern)
                    .unwrap_or_default();

            if matching_tasks.is_empty() && change.ignore_not_found {
                // Skip if no tasks found and ignore_not_found is true
                continue;
            }

            for task_info in matching_tasks {
                let actual_state = match &task_info.state {
                    scheduler_service::TaskState::Ready => Some("Ready".to_string()),
                    scheduler_service::TaskState::Disabled => Some("Disabled".to_string()),
                    scheduler_service::TaskState::Running => Some("Running".to_string()),
                    scheduler_service::TaskState::NotFound => None,
                    scheduler_service::TaskState::Unknown(s) => Some(s.clone()),
                };

                let (expected_state, is_match) = match change.action {
                    SchedulerAction::Enable => {
                        let expected = "Ready";
                        let matches = matches!(
                            task_info.state,
                            scheduler_service::TaskState::Ready
                                | scheduler_service::TaskState::Running
                        );
                        (expected, matches)
                    }
                    SchedulerAction::Disable => {
                        let expected = "Disabled";
                        let matches =
                            matches!(task_info.state, scheduler_service::TaskState::Disabled);
                        (expected, matches)
                    }
                    SchedulerAction::Delete => {
                        let expected = "Deleted";
                        let matches =
                            matches!(task_info.state, scheduler_service::TaskState::NotFound);
                        (expected, matches)
                    }
                };

                results.push(SchedulerMismatch {
                    task_path: change.task_path.clone(),
                    task_name: task_info.name,
                    expected_state: expected_state.to_string(),
                    actual_state,
                    description: format!("{:?} task (pattern: {})", change.action, pattern),
                    is_match,
                    skip_validation: change.skip_validation,
                });
            }
        } else if let Some(task_name) = &change.task_name {
            // Single task inspection
            let task_state = scheduler_service::get_task_state(&change.task_path, task_name)
                .unwrap_or(scheduler_service::TaskState::Unknown("Error".to_string()));

            // Handle not found case
            if matches!(task_state, scheduler_service::TaskState::NotFound)
                && change.ignore_not_found
            {
                continue;
            }

            let actual_state = match &task_state {
                scheduler_service::TaskState::Ready => Some("Ready".to_string()),
                scheduler_service::TaskState::Disabled => Some("Disabled".to_string()),
                scheduler_service::TaskState::Running => Some("Running".to_string()),
                scheduler_service::TaskState::NotFound => None,
                scheduler_service::TaskState::Unknown(s) => Some(s.clone()),
            };

            let (expected_state, is_match) = match change.action {
                SchedulerAction::Enable => {
                    let expected = "Ready";
                    let matches = matches!(
                        task_state,
                        scheduler_service::TaskState::Ready | scheduler_service::TaskState::Running
                    );
                    (expected, matches)
                }
                SchedulerAction::Disable => {
                    let expected = "Disabled";
                    let matches = matches!(task_state, scheduler_service::TaskState::Disabled);
                    (expected, matches)
                }
                SchedulerAction::Delete => {
                    let expected = "Deleted";
                    let matches = matches!(task_state, scheduler_service::TaskState::NotFound);
                    (expected, matches)
                }
            };

            results.push(SchedulerMismatch {
                task_path: change.task_path.clone(),
                task_name: task_name.clone(),
                expected_state: expected_state.to_string(),
                actual_state,
                description: format!("{:?} task", change.action),
                is_match,
                skip_validation: change.skip_validation,
            });
        }
        // If neither task_name nor task_name_pattern is set, skip this change
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_validation_logic() {
        let registry_success = RegistryMismatch {
            hive: "HKCU".into(),
            key: "Test".into(),
            value_name: "Val".into(),
            expected_value: None,
            actual_value: None,
            value_type: None,
            description: "Test".into(),
            is_match: true,
            skip_validation: false,
        };

        let registry_fail = RegistryMismatch {
            hive: "HKCU".into(),
            key: "Test".into(),
            value_name: "Val".into(),
            expected_value: None,
            actual_value: None,
            value_type: None,
            description: "Test".into(),
            is_match: false,
            skip_validation: false,
        };

        let registry_fail_skip = RegistryMismatch {
            hive: "HKCU".into(),
            key: "Test".into(),
            value_name: "Val".into(),
            expected_value: None,
            actual_value: None,
            value_type: None,
            description: "Test".into(),
            is_match: false,
            skip_validation: true,
        };

        // All match -> OK
        assert!(calculate_overall_match(
            &[registry_success.clone()],
            &[],
            &[]
        ));

        // One fail -> Fail
        assert!(!calculate_overall_match(
            &[registry_success.clone(), registry_fail.clone()],
            &[],
            &[]
        ));

        // Validation fail but skip_validation is true -> OK
        assert!(calculate_overall_match(
            &[registry_success.clone(), registry_fail_skip.clone()],
            &[],
            &[]
        ));

        // All skipped -> OK (technically vacuous truth)
        assert!(calculate_overall_match(
            &[registry_fail_skip.clone()],
            &[],
            &[]
        ));
    }
}
