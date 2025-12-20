use super::capture::read_registry_value;
use super::helpers::values_match;
use crate::error::Error;
use crate::models::{
    OptionInspection, RegistryAction, RegistryMismatch, ServiceMismatch, TweakDefinition,
    TweakInspection, TweakOption,
};
use crate::services::{registry_service, service_control};
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

    // Determine if everything matches
    let all_match =
        registry_results.iter().all(|r| r.is_match) && service_results.iter().all(|s| s.is_match);

    Ok(OptionInspection {
        option_index: index,
        label: option.label.clone(),
        is_current,
        is_pending,
        registry_results,
        service_results,
        all_match,
    })
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
        });
    }

    Ok(results)
}
