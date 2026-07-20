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

    /// Documents a KNOWN DIVERGENCE, and is written to pass against it.
    ///
    /// There are two comparison implementations in the codebase and they disagree:
    ///
    /// - `detection.rs` compares through `registry_value::registry_values_match`,
    ///   which normalises both sides by their declared `RegistryValueType` first.
    /// - `inspection.rs` compares through `values_match` above, which is raw
    ///   `serde_json` equality with an integer-width fallback and knows nothing
    ///   about registry types.
    ///
    /// So a REG_BINARY value authored in the documented `"00,A0,FF"` hex form
    /// MATCHES in detection and MISMATCHES in inspection: the tweak card reads
    /// "applied" while the details modal lists a mismatch for the same value.
    /// `registry_value.rs`'s own test asserts the normalising form is correct, so
    /// `values_match` is the wrong one.
    ///
    /// The 2026-05-29 plan's Task 1 was supposed to unify these -- its acceptance
    /// criterion required REG_BINARY to "apply, restore, export/import, and detect
    /// consistently" -- but the unification reached `detection.rs` and stopped.
    ///
    /// This test pins the CURRENT behaviour so the divergence is visible and cannot
    /// widen unnoticed. When the two comparison paths are collapsed onto one core,
    /// this test SHOULD start failing: that is the signal the collapse worked, and
    /// it should then be deleted rather than adjusted.
    #[test]
    fn values_match_disagrees_with_registry_values_match_on_binary_hex_strings() {
        let byte_array = Some(serde_json::json!([0, 160, 255]));
        let hex_string = Some(serde_json::json!("00,A0,FF"));

        // The registry-aware comparison correctly treats these as the same bytes.
        assert!(
            crate::services::registry_value::registry_values_match(
                &RegistryValueType::Binary,
                &byte_array,
                &hex_string,
            )
            .unwrap(),
            "registry_values_match should normalise both forms to the same bytes"
        );

        // The type-blind comparison does not, because it only sees an array and a string.
        assert!(
            !values_match(&byte_array, &hex_string),
            "values_match unexpectedly handles REG_BINARY hex strings -- if this now \
             passes, the two comparison paths have been unified and this test should \
             be deleted rather than inverted"
        );
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
