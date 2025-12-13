//! Helper Functions
//!
//! Utility functions for parsing registry types and values.

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

/// Compare two JSON values for equality (handles numeric type variations)
pub fn values_match(a: &Option<serde_json::Value>, b: &Option<serde_json::Value>) -> bool {
    match (a, b) {
        (Some(va), Some(vb)) => {
            // Handle numeric type variations (i64 vs u64)
            if let (Some(na), Some(nb)) = (va.as_i64(), vb.as_i64()) {
                return na == nb;
            }
            if let (Some(na), Some(nb)) = (va.as_u64(), vb.as_u64()) {
                return na == nb;
            }
            // Standard comparison
            va == vb
        }
        (None, None) => true,
        _ => false,
    }
}

/// Check if two scheduler task states match (considers Ready/Running as equivalent)
pub fn task_state_matches(
    current: &scheduler_service::TaskState,
    expected: &scheduler_service::TaskState,
) -> bool {
    match (current, expected) {
        // Ready and Running are both "enabled" states
        (scheduler_service::TaskState::Ready, scheduler_service::TaskState::Ready) => true,
        (scheduler_service::TaskState::Running, scheduler_service::TaskState::Ready) => true,
        (scheduler_service::TaskState::Ready, scheduler_service::TaskState::Running) => true,
        // Exact matches for other states
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
    fn test_values_match() {
        assert!(values_match(
            &Some(serde_json::json!(1)),
            &Some(serde_json::json!(1))
        ));
        assert!(!values_match(
            &Some(serde_json::json!(1)),
            &Some(serde_json::json!(2))
        ));
        assert!(values_match(&None, &None));
        assert!(!values_match(&Some(serde_json::json!(1)), &None));
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
