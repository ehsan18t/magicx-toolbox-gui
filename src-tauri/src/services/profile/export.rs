//! Profile Export
//!
//! Export configuration profiles from current tweak state.

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::Error;
use crate::models::{
    ConfigurationProfile, ExportOptions, ProfileMetadata, RegistryValueState, SchedulerState,
    ServiceState, SnapshotMetadata, SystemStateSnapshot, TweakDefinition, TweakSelection,
    PROFILE_SCHEMA_VERSION,
};
use crate::services::backup::detect_tweak_state;
use crate::services::profile::archive::write_profile_archive;
use crate::services::{registry_service, scheduler_service, service_control};

/// Export a configuration profile.
pub fn export_profile(
    path: &Path,
    available_tweaks: &[TweakDefinition],
    options: &ExportOptions,
    windows_version: u32,
    windows_build: u32,
) -> Result<(), Error> {
    log::info!("Exporting profile '{}' to {}", options.name, path.display());

    let app_version = env!("CARGO_PKG_VERSION").to_string();

    // Build selections from applied tweaks
    let selections = build_selections(available_tweaks, &options.tweak_ids, windows_version)?;

    if selections.is_empty() {
        return Err(Error::ProfileError(
            "No applied tweaks to export".to_string(),
        ));
    }

    // Create metadata
    let metadata = ProfileMetadata::new(
        options.name.clone(),
        options.description.clone(),
        app_version.clone(),
        windows_version,
        windows_build,
    );

    // Build profile
    let profile = ConfigurationProfile::new(metadata, selections);

    // Capture system state if requested
    let system_state = if options.include_system_state {
        Some(capture_system_state(
            available_tweaks,
            &app_version,
            windows_version,
            windows_build,
        )?)
    } else {
        None
    };

    // Write the archive
    write_profile_archive(path, &profile, system_state.as_ref())?;

    log::info!(
        "Profile exported successfully with {} tweaks",
        profile.selections.len()
    );
    Ok(())
}

/// Build tweak selections from current state.
fn build_selections(
    available_tweaks: &[TweakDefinition],
    tweak_ids: &[String],
    windows_version: u32,
) -> Result<Vec<TweakSelection>, Error> {
    let mut selections = Vec::new();

    for tweak in available_tweaks {
        // Skip if tweak_ids is specified and this tweak isn't in it
        if !tweak_ids.is_empty() && !tweak_ids.contains(&tweak.id) {
            continue;
        }

        // Skip if not applicable to this Windows version
        if !tweak.applies_to_version(windows_version) {
            continue;
        }

        // Get current state
        let state = match detect_tweak_state(tweak, windows_version) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to detect state for '{}': {}", tweak.id, e);
                continue;
            }
        };

        // Only include if at a known option (not default/unknown)
        if let Some(option_index) = state.current_option_index {
            let option = &tweak.options[option_index];
            let option_hash = hash_option_content(option);

            selections.push(TweakSelection {
                tweak_id: tweak.id.clone(),
                selected_option_index: option_index,
                selected_option_label: option.label.clone(),
                option_content_hash: Some(option_hash),
                category_id: Some(tweak.category_id.clone()),
            });
        }
    }

    Ok(selections)
}

/// Compute a hash of option content for schema change detection.
fn hash_option_content(option: &crate::models::TweakOption) -> String {
    let mut hasher = Sha256::new();

    // Hash registry changes
    for change in &option.registry_changes {
        hasher.update(change.hive.as_str().as_bytes());
        hasher.update(change.key.as_bytes());
        hasher.update(change.value_name.as_bytes());
        if let Some(ref v) = change.value {
            hasher.update(format!("{:?}", v).as_bytes());
        }
    }

    // Hash service changes
    for service in &option.service_changes {
        hasher.update(service.name.as_bytes());
        hasher.update(service.startup.as_str().as_bytes());
    }

    // Hash scheduler changes
    for task in &option.scheduler_changes {
        hasher.update(task.task_path.as_bytes());
        if let Some(ref name) = task.task_name {
            hasher.update(name.as_bytes());
        }
        hasher.update(task.action.as_str().as_bytes());
    }

    hex::encode(hasher.finalize())[..16].to_string()
}

/// Capture current system state for the profile.
fn capture_system_state(
    available_tweaks: &[TweakDefinition],
    app_version: &str,
    windows_version: u32,
    windows_build: u32,
) -> Result<SystemStateSnapshot, Error> {
    log::debug!("Capturing system state snapshot");

    let machine_name = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let metadata = SnapshotMetadata {
        created_at: chrono::Local::now().to_rfc3339(),
        app_version: app_version.to_string(),
        windows_version,
        windows_build,
        machine_name,
    };

    let mut registry_state = Vec::new();
    let mut service_state = Vec::new();
    let mut scheduler_state = Vec::new();

    // Collect state from all tweaks
    for tweak in available_tweaks {
        if !tweak.applies_to_version(windows_version) {
            continue;
        }

        for option in &tweak.options {
            // Capture registry state
            for change in &option.registry_changes {
                if !change.applies_to_version(windows_version) {
                    continue;
                }

                let (exists, value) = read_registry_value(
                    &change.hive,
                    &change.key,
                    &change.value_name,
                    change.value_type.as_ref(),
                );

                registry_state.push(RegistryValueState {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: change.value_name.clone(),
                    value_type: change.value_type.as_ref().map(|vt| vt.as_str().to_string()),
                    value,
                    exists,
                });
            }

            // Capture service state
            for service_change in &option.service_changes {
                if let Ok(status) = service_control::get_service_status(&service_change.name) {
                    service_state.push(ServiceState {
                        name: service_change.name.clone(),
                        startup_type: status
                            .startup_type
                            .map(|st| st.as_str().to_string())
                            .unwrap_or_else(|| "Unknown".to_string()),
                        is_running: status.state == service_control::ServiceState::Running,
                        exists: true,
                    });
                } else {
                    service_state.push(ServiceState {
                        name: service_change.name.clone(),
                        startup_type: "Unknown".to_string(),
                        is_running: false,
                        exists: false,
                    });
                }
            }

            // Capture scheduler state
            for task_change in &option.scheduler_changes {
                if let Some(ref task_name) = task_change.task_name {
                    let state =
                        scheduler_service::get_task_state(&task_change.task_path, task_name)
                            .unwrap_or(scheduler_service::TaskState::NotFound);

                    scheduler_state.push(SchedulerState {
                        task_path: task_change.task_path.clone(),
                        task_name: task_name.clone(),
                        state: format!("{:?}", state),
                        exists: state != scheduler_service::TaskState::NotFound,
                    });
                }
            }
        }
    }

    // Deduplicate entries using a HashSet-based approach
    // (dedup_by only removes consecutive duplicates, which doesn't work for unsorted data)
    let registry_state = deduplicate_registry_state(registry_state);
    let service_state = deduplicate_service_state(service_state);
    let scheduler_state = deduplicate_scheduler_state(scheduler_state);

    Ok(SystemStateSnapshot {
        schema_version: PROFILE_SCHEMA_VERSION,
        metadata,
        registry_state,
        service_state,
        scheduler_state,
    })
}

/// Deduplicate registry state entries by (hive, key, value_name)
fn deduplicate_registry_state(entries: Vec<RegistryValueState>) -> Vec<RegistryValueState> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for entry in entries {
        let key = (
            entry.hive.clone(),
            entry.key.clone(),
            entry.value_name.clone(),
        );
        if seen.insert(key) {
            result.push(entry);
        }
    }

    result
}

/// Deduplicate service state entries by name
fn deduplicate_service_state(entries: Vec<ServiceState>) -> Vec<ServiceState> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for entry in entries {
        if seen.insert(entry.name.clone()) {
            result.push(entry);
        }
    }

    result
}

/// Deduplicate scheduler state entries by (task_path, task_name)
fn deduplicate_scheduler_state(entries: Vec<SchedulerState>) -> Vec<SchedulerState> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for entry in entries {
        let key = (entry.task_path.clone(), entry.task_name.clone());
        if seen.insert(key) {
            result.push(entry);
        }
    }

    result
}

/// Read a registry value using the expected type, returning (exists, value).
fn read_registry_value(
    hive: &crate::models::RegistryHive,
    key: &str,
    value_name: &str,
    value_type: Option<&crate::models::RegistryValueType>,
) -> (bool, Option<serde_json::Value>) {
    use crate::models::RegistryValueType;

    // If we know the type, read with that type
    if let Some(vt) = value_type {
        match vt {
            RegistryValueType::Dword => match registry_service::read_dword(hive, key, value_name) {
                Ok(Some(v)) => (true, Some(serde_json::Value::Number(v.into()))),
                Ok(None) => (false, None),
                Err(_) => (false, None),
            },
            RegistryValueType::String | RegistryValueType::ExpandString => {
                match registry_service::read_string(hive, key, value_name) {
                    Ok(Some(v)) => (true, Some(serde_json::Value::String(v))),
                    Ok(None) => (false, None),
                    Err(_) => (false, None),
                }
            }
            RegistryValueType::Binary => {
                match registry_service::read_binary(hive, key, value_name) {
                    Ok(Some(v)) => {
                        let arr: Vec<serde_json::Value> = v
                            .into_iter()
                            .map(|b| serde_json::Value::Number(b.into()))
                            .collect();
                        (true, Some(serde_json::Value::Array(arr)))
                    }
                    Ok(None) => (false, None),
                    Err(_) => (false, None),
                }
            }
            RegistryValueType::Qword => match registry_service::read_qword(hive, key, value_name) {
                Ok(Some(v)) => (true, Some(serde_json::json!(v))),
                Ok(None) => (false, None),
                Err(_) => (false, None),
            },
            RegistryValueType::MultiString => {
                // Not supported, just check if value exists
                match registry_service::value_exists(hive, key, value_name) {
                    Ok(true) => (true, None),
                    _ => (false, None),
                }
            }
        }
    } else {
        // No type specified, try DWORD as default (most common)
        match registry_service::read_dword(hive, key, value_name) {
            Ok(Some(v)) => (true, Some(serde_json::Value::Number(v.into()))),
            Ok(None) => (false, None),
            Err(_) => {
                // Fallback: just check if value exists
                match registry_service::value_exists(hive, key, value_name) {
                    Ok(true) => (true, None),
                    _ => (false, None),
                }
            }
        }
    }
}
