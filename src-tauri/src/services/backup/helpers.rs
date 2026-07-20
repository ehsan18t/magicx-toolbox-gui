//! Helper Functions
//!
//! Utility functions for parsing registry types and comparing scheduler task states.

use crate::error::Error;
use crate::models::{RegistryHive, RegistryValueType};
use crate::services::scheduler_service;

/// Parse hive string to RegistryHive enum
pub fn parse_hive(hive: &str) -> Result<RegistryHive, Error> {
    match hive {
        "HKCU" => Ok(RegistryHive::Hkcu),
        "HKLM" => Ok(RegistryHive::Hklm),
        _ => Err(Error::BackupFailed(format!("Unknown hive: {}", hive))),
    }
}

/// Parse value type string to RegistryValueType enum
pub fn parse_value_type(value_type: &str) -> Result<RegistryValueType, Error> {
    match value_type {
        "REG_DWORD" => Ok(RegistryValueType::Dword),
        "REG_QWORD" => Ok(RegistryValueType::Qword),
        "REG_SZ" => Ok(RegistryValueType::String),
        "REG_EXPAND_SZ" => Ok(RegistryValueType::ExpandString),
        "REG_MULTI_SZ" => Ok(RegistryValueType::MultiString),
        "REG_BINARY" => Ok(RegistryValueType::Binary),
        _ => Err(Error::BackupFailed(format!(
            "Unknown value type: {}",
            value_type
        ))),
    }
}

/// Check if two scheduler task states match (considers Ready/Running as equivalent).
pub fn task_state_matches(
    current: &scheduler_service::TaskState,
    expected: &scheduler_service::TaskState,
) -> bool {
    match (current, expected) {
        // Ready and Running are both "enabled" states.
        (scheduler_service::TaskState::Ready, scheduler_service::TaskState::Ready) => true,
        (scheduler_service::TaskState::Running, scheduler_service::TaskState::Ready) => true,
        (scheduler_service::TaskState::Ready, scheduler_service::TaskState::Running) => true,
        // Exact matches for other states.
        (a, b) => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hive() {
        assert!(matches!(parse_hive("HKCU"), Ok(RegistryHive::Hkcu)));
        assert!(matches!(parse_hive("HKLM"), Ok(RegistryHive::Hklm)));
        assert!(parse_hive("INVALID").is_err());
    }

    #[test]
    fn test_parse_value_type() {
        assert!(matches!(
            parse_value_type("REG_DWORD"),
            Ok(RegistryValueType::Dword)
        ));
        assert!(matches!(
            parse_value_type("REG_SZ"),
            Ok(RegistryValueType::String)
        ));
        assert!(parse_value_type("INVALID").is_err());
    }

    #[test]
    fn test_task_state_matches() {
        assert!(task_state_matches(
            &scheduler_service::TaskState::Ready,
            &scheduler_service::TaskState::Ready
        ));
        assert!(task_state_matches(
            &scheduler_service::TaskState::Running,
            &scheduler_service::TaskState::Ready
        ));
        assert!(!task_state_matches(
            &scheduler_service::TaskState::Disabled,
            &scheduler_service::TaskState::Ready
        ));
    }
}
