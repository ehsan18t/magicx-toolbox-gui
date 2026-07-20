//! Tweak model definitions for the unified option-based tweak system.
//!
//! Every tweak has an array of options, each containing its own registry changes,
//! service changes, and commands. `is_toggle: true` renders as a switch (2 options),
//! otherwise as a dropdown.
//!
//! The YAML-schema type definitions (`RiskLevel`, `RegistryChange`, `TweakDefinition`,
//! etc.) live in [`crate::models::tweak_schema`] and are shared with `build.rs` so
//! both consume the exact same type instead of hand-mirrored copies. This module
//! re-exports them (so existing `models::tweak::Foo` paths keep working) and holds
//! the runtime-only impls for those types, plus the runtime-only status/result types
//! (`TweakState`, `TweakResult`, `TweakStatus`) that build.rs never sees.

use serde::{Deserialize, Serialize};

// Re-export the shared schema types so they remain reachable as `models::tweak::*` (and, via
// `models/mod.rs`'s `pub use tweak::*`, as `models::*`). `models/mod.rs` must therefore NOT also
// glob `tweak_schema` directly, or every schema name becomes ambiguous through two globs.
pub use crate::models::tweak_schema::*;

// ============================================================================
// IMPLS FOR SHARED SCHEMA TYPES
// ============================================================================

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

impl RegistryHive {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryHive::Hkcu => "HKCU",
            RegistryHive::Hklm => "HKLM",
        }
    }
}

impl RegistryValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryValueType::Dword => "REG_DWORD",
            RegistryValueType::Qword => "REG_QWORD",
            RegistryValueType::String => "REG_SZ",
            RegistryValueType::ExpandString => "REG_EXPAND_SZ",
            RegistryValueType::MultiString => "REG_MULTI_SZ",
            RegistryValueType::Binary => "REG_BINARY",
        }
    }
}

impl RegistryAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryAction::Set => "set",
            RegistryAction::DeleteValue => "delete_value",
            RegistryAction::DeleteKey => "delete_key",
            RegistryAction::CreateKey => "create_key",
        }
    }
}

impl ServiceStartupType {
    /// Convert to Windows SC command start type string
    pub fn to_sc_start_type(&self) -> &'static str {
        match self {
            ServiceStartupType::Disabled => "disabled",
            ServiceStartupType::Manual => "demand",
            ServiceStartupType::Automatic => "auto",
            ServiceStartupType::Boot => "boot",
            ServiceStartupType::System => "system",
        }
    }

    /// Convert from Windows registry Start value (DWORD)
    pub fn from_registry_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(ServiceStartupType::Boot),
            1 => Some(ServiceStartupType::System),
            2 => Some(ServiceStartupType::Automatic),
            3 => Some(ServiceStartupType::Manual),
            4 => Some(ServiceStartupType::Disabled),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceStartupType::Disabled => "disabled",
            ServiceStartupType::Manual => "manual",
            ServiceStartupType::Automatic => "automatic",
            ServiceStartupType::Boot => "boot",
            ServiceStartupType::System => "system",
        }
    }
}

impl SchedulerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulerAction::Enable => "enable",
            SchedulerAction::Disable => "disable",
            SchedulerAction::Delete => "delete",
        }
    }
}

impl RegistryChange {
    /// Check if this registry change applies to a given Windows version
    pub fn applies_to_version(&self, version: u32) -> bool {
        match &self.windows_versions {
            None => true,
            Some(versions) if versions.is_empty() => true,
            Some(versions) => versions.contains(&version),
        }
    }
}

impl HostsAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            HostsAction::Add => "add",
            HostsAction::Remove => "remove",
        }
    }
}

impl FirewallDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallDirection::Inbound => "in",
            FirewallDirection::Outbound => "out",
        }
    }
}

impl FirewallRuleAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallRuleAction::Block => "block",
            FirewallRuleAction::Allow => "allow",
        }
    }
}

impl FirewallProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallProtocol::Any => "any",
            FirewallProtocol::Tcp => "tcp",
            FirewallProtocol::Udp => "udp",
            FirewallProtocol::Icmpv4 => "icmpv4",
            FirewallProtocol::Icmpv6 => "icmpv6",
        }
    }
}

impl FirewallOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallOperation::Create => "create",
            FirewallOperation::Delete => "delete",
        }
    }
}

impl TweakOption {
    /// Check if this option has any effective changes for the given Windows version
    pub fn has_changes_for_version(&self, version: u32) -> bool {
        let has_registry = self
            .registry_changes
            .iter()
            .any(|c| c.applies_to_version(version));
        let has_services = !self.service_changes.is_empty();
        let has_scheduler = !self.scheduler_changes.is_empty();
        let has_hosts = !self.hosts_changes.is_empty();
        let has_firewall = !self.firewall_changes.is_empty();
        let has_commands = !self.pre_commands.is_empty() || !self.post_commands.is_empty();
        let has_powershell = !self.pre_powershell.is_empty() || !self.post_powershell.is_empty();
        has_registry
            || has_services
            || has_scheduler
            || has_hosts
            || has_firewall
            || has_commands
            || has_powershell
    }
}

impl TweakDefinition {
    /// Check if this tweak applies to a given Windows version
    /// (has at least one option with changes for that version)
    pub fn applies_to_version(&self, version: u32) -> bool {
        self.options
            .iter()
            .any(|opt| opt.has_changes_for_version(version))
    }

    /// The privilege level this tweak's operations run at, derived from its declared flags.
    pub fn elevation(&self) -> crate::services::elevation::Elevation {
        crate::services::elevation::Elevation::from_flags(self.requires_system, self.requires_ti)
    }
}

// ============================================================================
// STATUS/RESULT TYPES
// ============================================================================

/// Current state of a tweak in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakState {
    pub tweak_id: String,
    /// Index of matching option, or None if no match (System Default)
    pub current_option_index: Option<usize>,
    /// True if a snapshot exists (tweak was applied by this app)
    pub has_snapshot: bool,
    /// The option index from snapshot (if exists)
    #[serde(default)]
    pub snapshot_option_index: Option<usize>,
    /// True if the status was inferred from missing items (via missing_is_match flag)
    /// rather than detected from actual registry/service values
    #[serde(default)]
    pub status_inferred: bool,
}

/// Result of applying or reverting a tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakResult {
    pub success: bool,
    pub message: String,
    pub requires_reboot: bool,
    /// List of (tweak_id, error_message) for failed operations in batch mode
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<(String, String)>,
}

/// Status of a specific tweak (returned to frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakStatus {
    pub tweak_id: String,
    /// Whether the tweak has been applied by this app (has snapshot)
    pub is_applied: bool,
    /// When the tweak was last applied (if snapshot exists)
    pub last_applied: Option<String>,
    /// Whether a snapshot exists for reverting
    pub has_backup: bool,
    /// Index of current matching option, or None if System Default
    pub current_option_index: Option<usize>,
    /// The original option index from the snapshot, if one exists.
    /// - None: No snapshot exists (tweak was never applied)
    /// - Some(None): Snapshot exists but original state was unknown (didn't match any option)
    /// - Some(Some(i)): Snapshot exists and original state matched option i
    ///
    /// Used by frontend to show "Default" segment when original state was unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_original_option_index: Option<Option<usize>>,
    /// True if the status was inferred from missing items (via missing_is_match flag)
    /// rather than detected from actual registry/service values
    #[serde(default)]
    pub status_inferred: bool,
    /// Error message if state detection failed (tweak still returned but with unknown state)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// True when the last revert did not fully succeed and the snapshot was kept for retry
    /// (ADR-0001 "Needs Attention").
    #[serde(default)]
    pub needs_attention: bool,
    /// Resources a partial revert could not restore (empty unless `needs_attention`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unrestorable_resources: Vec<String>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry_change(value: i32, windows_versions: Option<Vec<u32>>) -> RegistryChange {
        RegistryChange {
            hive: RegistryHive::Hkcu,
            key: "SOFTWARE\\Test".to_string(),
            value_name: "Value".to_string(),
            action: RegistryAction::Set,
            value_type: Some(RegistryValueType::Dword),
            value: Some(serde_json::json!(value)),
            windows_versions,
            skip_validation: false,
        }
    }

    #[test]
    fn test_registry_change_applies_to_version() {
        let change = make_registry_change(1, None);
        assert!(change.applies_to_version(10));
        assert!(change.applies_to_version(11));

        let change = make_registry_change(1, Some(vec![11]));
        assert!(!change.applies_to_version(10));
        assert!(change.applies_to_version(11));

        let change = make_registry_change(1, Some(vec![10, 11]));
        assert!(change.applies_to_version(10));
        assert!(change.applies_to_version(11));
    }
}
