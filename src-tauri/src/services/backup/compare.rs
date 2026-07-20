//! Shared comparison core for `detection` and `inspection`.
//!
//! Both the tweak-card status (detection) and the details modal (inspection) answer the same
//! question — "does this option's declared state match the current system?" — and used to answer it
//! with two independent, drifting implementations. This is the one core they now share.
//!
//! `compare_option` produces the full per-item results (the `*Mismatch` structs the frontend
//! renders). Detection reduces them to a boolean via [`OptionComparison::all_match`]; inspection
//! renders them directly.
//!
//! The four historical divergences are resolved here, in one place:
//! - **Registry comparator:** the typed, `RegistryValueType`-aware `registry_values_match` (so a
//!   `REG_BINARY` authored as `"00,A0,FF"` matches a stored `[0,160,255]`).
//! - **`*_missing_is_match`:** honored (a missing item counts as an *inferred* match).
//! - **Empty option:** an option with zero validatable items does NOT match (nothing to confirm).
//! - **Service query errors:** propagated (as registry/hosts/firewall queries already were).

use crate::error::Error;
use crate::models::inspection::{
    FirewallMismatch, HostsMismatch, RegistryMismatch, SchedulerMismatch, ServiceMismatch,
};
use crate::models::tweak::{FirewallOperation, HostsAction, SchedulerAction};
use crate::models::{RegistryAction, TweakOption};
use crate::services::{
    firewall_service, hosts_service, registry_service, registry_value, scheduler_service,
    service_control,
};

use super::capture::read_registry_value;
use super::helpers::task_state_matches;

/// The full per-item comparison of one option against current system state.
pub struct OptionComparison {
    pub registry: Vec<RegistryMismatch>,
    pub service: Vec<ServiceMismatch>,
    pub scheduler: Vec<SchedulerMismatch>,
    pub hosts: Vec<HostsMismatch>,
    pub firewall: Vec<FirewallMismatch>,
    /// True if any validatable item matched only because a `*_missing_is_match` flag treated a
    /// missing item as a match (rather than an actual-value match). Drives `status_inferred`.
    pub inferred: bool,
}

impl OptionComparison {
    /// Whether the option matches current state: at least one validatable (non-`skip_validation`)
    /// item AND every validatable item matches. An option with zero validatable items cannot be the
    /// current state — there is nothing to confirm — so it does not match.
    pub fn all_match(&self) -> bool {
        let validatable: Vec<bool> = std::iter::empty()
            .chain(
                self.registry
                    .iter()
                    .filter(|r| !r.skip_validation)
                    .map(|r| r.is_match),
            )
            .chain(
                self.service
                    .iter()
                    .filter(|s| !s.skip_validation)
                    .map(|s| s.is_match),
            )
            .chain(
                self.scheduler
                    .iter()
                    .filter(|s| !s.skip_validation)
                    .map(|s| s.is_match),
            )
            .chain(
                self.hosts
                    .iter()
                    .filter(|h| !h.skip_validation)
                    .map(|h| h.is_match),
            )
            .chain(
                self.firewall
                    .iter()
                    .filter(|f| !f.skip_validation)
                    .map(|f| f.is_match),
            )
            .collect();

        !validatable.is_empty() && validatable.iter().all(|&m| m)
    }
}

/// Compare one option against current system state, building the per-item result lists.
pub fn compare_option(
    option: &TweakOption,
    windows_version: u32,
) -> Result<OptionComparison, Error> {
    let mut inferred = false;
    let registry = compare_registry(option, windows_version, &mut inferred)?;
    let service = compare_service(option, &mut inferred)?;
    let scheduler = compare_scheduler(option, &mut inferred)?;
    let hosts = compare_hosts(option)?;
    let firewall = compare_firewall(option)?;
    Ok(OptionComparison {
        registry,
        service,
        scheduler,
        hosts,
        firewall,
        inferred,
    })
}

/// Record that a non-`skip_validation` item matched via a `*_missing_is_match` flag.
fn note_inferred(inferred: &mut bool, skip_validation: bool) {
    if !skip_validation {
        *inferred = true;
    }
}

fn compare_registry(
    option: &TweakOption,
    windows_version: u32,
    inferred: &mut bool,
) -> Result<Vec<RegistryMismatch>, Error> {
    let missing_is_match = option.registry_missing_is_match;
    let mut results = Vec::new();

    for change in &option.registry_changes {
        if !change.applies_to_version(windows_version) {
            continue;
        }

        let path = format!("{}\\{}", change.hive.as_str(), change.key);
        let value_label = if change.value_name.is_empty() {
            "(Default)".to_string()
        } else {
            change.value_name.clone()
        };

        let mismatch = match change.action {
            RegistryAction::Set => {
                let (value_type, expected_val) = match (&change.value_type, &change.value) {
                    (Some(vt), Some(v)) => (vt, v),
                    _ => continue, // Invalid config: nothing to compare.
                };

                let (current_val, existed) =
                    read_registry_value(&change.hive, &change.key, &change.value_name, value_type)?;

                let is_match = if !existed {
                    if missing_is_match {
                        note_inferred(inferred, change.skip_validation);
                        true
                    } else {
                        false
                    }
                } else {
                    registry_value::registry_values_match(
                        value_type,
                        &current_val,
                        &Some(expected_val.clone()),
                    )
                    .unwrap_or(false)
                };

                RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: change.value_name.clone(),
                    expected_value: Some(expected_val.clone()),
                    actual_value: if existed { current_val } else { None },
                    value_type: Some(value_type.as_str().to_string()),
                    description: format!("Set {} to {:?}", value_label, expected_val),
                    is_match,
                    skip_validation: change.skip_validation,
                }
            }
            RegistryAction::DeleteValue => {
                let exists =
                    registry_service::value_exists(&change.hive, &change.key, &change.value_name)
                        .unwrap_or(false);
                RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: change.value_name.clone(),
                    expected_value: None,
                    actual_value: exists.then(|| serde_json::json!("Exists")),
                    value_type: None,
                    description: format!("Delete value {}", value_label),
                    is_match: !exists,
                    skip_validation: change.skip_validation,
                }
            }
            RegistryAction::DeleteKey => {
                let exists =
                    registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);
                RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: String::new(),
                    expected_value: None,
                    actual_value: exists.then(|| serde_json::json!("Exists")),
                    value_type: None,
                    description: format!("Delete key {}", path),
                    is_match: !exists,
                    skip_validation: change.skip_validation,
                }
            }
            RegistryAction::CreateKey => {
                let exists =
                    registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);
                let is_match = if !exists && missing_is_match {
                    note_inferred(inferred, change.skip_validation);
                    true
                } else {
                    exists
                };
                RegistryMismatch {
                    hive: change.hive.as_str().to_string(),
                    key: change.key.clone(),
                    value_name: String::new(),
                    expected_value: Some(serde_json::json!("Exists")),
                    actual_value: exists.then(|| serde_json::json!("Exists")),
                    value_type: None,
                    description: format!("Create key {}", path),
                    is_match,
                    skip_validation: change.skip_validation,
                }
            }
        };

        results.push(mismatch);
    }

    Ok(results)
}

fn compare_service(
    option: &TweakOption,
    inferred: &mut bool,
) -> Result<Vec<ServiceMismatch>, Error> {
    let missing_is_match = option.service_missing_is_match;
    let mut results = Vec::new();

    for change in &option.service_changes {
        // A query failure propagates (as it did in detection); a service that genuinely does not
        // exist is Ok(status) with exists == false, which the missing_is_match flag can absorb.
        let status = service_control::get_service_status(&change.name)?;

        let is_match = if !status.exists {
            if missing_is_match {
                note_inferred(inferred, change.skip_validation);
                true
            } else {
                false
            }
        } else {
            status.startup_type == Some(change.startup)
        };

        results.push(ServiceMismatch {
            name: change.name.clone(),
            expected_startup: format!("{:?}", change.startup),
            actual_startup: status.startup_type.map(|s| format!("{:?}", s)),
            description: format!("Set startup to {:?}", change.startup),
            is_match,
            skip_validation: change.skip_validation,
        });
    }

    Ok(results)
}

fn task_state_label(state: &scheduler_service::TaskState) -> Option<String> {
    match state {
        scheduler_service::TaskState::Ready => Some("Ready".to_string()),
        scheduler_service::TaskState::Disabled => Some("Disabled".to_string()),
        scheduler_service::TaskState::Running => Some("Running".to_string()),
        scheduler_service::TaskState::NotFound => None,
        scheduler_service::TaskState::Unknown(s) => Some(s.clone()),
    }
}

fn compare_scheduler(
    option: &TweakOption,
    inferred: &mut bool,
) -> Result<Vec<SchedulerMismatch>, Error> {
    let missing_is_match = option.scheduler_missing_is_match;
    let mut results = Vec::new();

    for change in &option.scheduler_changes {
        let (expected_state, expected_label) = match change.action {
            SchedulerAction::Enable => (scheduler_service::TaskState::Ready, "Ready"),
            SchedulerAction::Disable => (scheduler_service::TaskState::Disabled, "Disabled"),
            SchedulerAction::Delete => (scheduler_service::TaskState::NotFound, "Deleted"),
        };

        if let Some(pattern) = &change.task_name_pattern {
            let tasks = scheduler_service::find_tasks_by_pattern(&change.task_path, pattern)
                .unwrap_or_default();

            if tasks.is_empty() {
                // No matching tasks: a match if we expected deletion, or the caller opted to ignore
                // absence, or the option infers a match from absence.
                let expected_absent = expected_state == scheduler_service::TaskState::NotFound;
                let is_match = expected_absent || change.ignore_not_found || missing_is_match;
                if is_match && missing_is_match && !expected_absent && !change.ignore_not_found {
                    note_inferred(inferred, change.skip_validation);
                }
                results.push(SchedulerMismatch {
                    task_path: change.task_path.clone(),
                    task_name: String::new(),
                    expected_state: expected_label.to_string(),
                    actual_state: None,
                    description: format!("{:?} task (pattern: {})", change.action, pattern),
                    is_match,
                    skip_validation: change.skip_validation,
                });
            } else {
                for task in tasks {
                    results.push(SchedulerMismatch {
                        task_path: change.task_path.clone(),
                        task_name: task.name.clone(),
                        expected_state: expected_label.to_string(),
                        actual_state: task_state_label(&task.state),
                        description: format!("{:?} task (pattern: {})", change.action, pattern),
                        is_match: task_state_matches(&task.state, &expected_state),
                        skip_validation: change.skip_validation,
                    });
                }
            }
        } else if let Some(task_name) = &change.task_name {
            let current = scheduler_service::get_task_state(&change.task_path, task_name)
                .unwrap_or(scheduler_service::TaskState::NotFound);

            let is_match = if current == scheduler_service::TaskState::NotFound {
                let expected_absent = expected_state == scheduler_service::TaskState::NotFound;
                let m = change.ignore_not_found || missing_is_match || expected_absent;
                if m && missing_is_match && !expected_absent && !change.ignore_not_found {
                    note_inferred(inferred, change.skip_validation);
                }
                m
            } else {
                task_state_matches(&current, &expected_state)
            };

            results.push(SchedulerMismatch {
                task_path: change.task_path.clone(),
                task_name: task_name.clone(),
                expected_state: expected_label.to_string(),
                actual_state: task_state_label(&current),
                description: format!("{:?} task", change.action),
                is_match,
                skip_validation: change.skip_validation,
            });
        }
        // Neither task_name nor task_name_pattern: nothing to compare.
    }

    Ok(results)
}

fn compare_hosts(option: &TweakOption) -> Result<Vec<HostsMismatch>, Error> {
    let mut results = Vec::new();

    for change in &option.hosts_changes {
        let exists = hosts_service::entry_exists(&change.ip, &change.domain)?;
        let expected_exists = matches!(change.action, HostsAction::Add);
        let description = if expected_exists {
            format!("Add hosts entry {} -> {}", change.domain, change.ip)
        } else {
            format!("Remove hosts entry {} -> {}", change.domain, change.ip)
        };
        results.push(HostsMismatch {
            ip: change.ip.clone(),
            domain: change.domain.clone(),
            expected_exists,
            actual_exists: exists,
            description,
            is_match: exists == expected_exists,
            skip_validation: change.skip_validation,
        });
    }

    Ok(results)
}

fn compare_firewall(option: &TweakOption) -> Result<Vec<FirewallMismatch>, Error> {
    let mut results = Vec::new();

    for change in &option.firewall_changes {
        let exists = firewall_service::rule_exists(&change.name)?;
        let expected_exists = matches!(change.operation, FirewallOperation::Create);
        let description = if expected_exists {
            format!("Create firewall rule '{}'", change.name)
        } else {
            format!("Delete firewall rule '{}'", change.name)
        };
        results.push(FirewallMismatch {
            name: change.name.clone(),
            expected_exists,
            actual_exists: exists,
            description,
            is_match: exists == expected_exists,
            skip_validation: change.skip_validation,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(is_match: bool, skip_validation: bool) -> RegistryMismatch {
        RegistryMismatch {
            hive: "HKCU".into(),
            key: "K".into(),
            value_name: "V".into(),
            expected_value: None,
            actual_value: None,
            value_type: None,
            description: String::new(),
            is_match,
            skip_validation,
        }
    }

    fn comparison(registry: Vec<RegistryMismatch>) -> OptionComparison {
        OptionComparison {
            registry,
            service: vec![],
            scheduler: vec![],
            hosts: vec![],
            firewall: vec![],
            inferred: false,
        }
    }

    #[test]
    fn a_matching_validatable_item_makes_the_option_match() {
        assert!(comparison(vec![reg(true, false)]).all_match());
    }

    #[test]
    fn one_validatable_mismatch_fails_the_option() {
        assert!(!comparison(vec![reg(true, false), reg(false, false)]).all_match());
    }

    #[test]
    fn skip_validation_items_are_excluded_from_the_verdict() {
        // A failing skip_validation item does not fail the option.
        assert!(comparison(vec![reg(true, false), reg(false, true)]).all_match());
    }

    #[test]
    fn an_option_with_no_validatable_items_does_not_match() {
        // The reconciled empty-option semantics: nothing to confirm -> not matched. (Inspection
        // previously returned vacuous-true here via `.all()` over an empty iterator.)
        assert!(!comparison(vec![]).all_match());
        assert!(!comparison(vec![reg(false, true)]).all_match());
    }
}
