//! Shared YAML-schema type definitions for the tweak system.
//!
//! These are the pure data definitions consumed by BOTH `build.rs` (build-time YAML
//! parsing/validation and JSON generation) and the runtime crate (`models::tweak`,
//! deserializing the embedded JSON). Previously these ~19 types were hand-mirrored
//! in both places and could drift silently; now there is exactly one definition of
//! each, and a field rename is a compile error on both sides.
//!
//! This file contains ONLY type definitions (derives + serde attributes + fields).
//! `impl` blocks stay with their respective consumers: runtime-only impls remain in
//! `models/tweak.rs`, build-only impls (validation, `requires_admin`, etc.) remain
//! in `build.rs`.

use serde::{Deserialize, Serialize};

// ============================================================================
// ENUMS
// ============================================================================

/// Risk level for a tweak indicating potential impact
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Safe to apply/revert without issues
    Low,
    /// May require restart or have minor side effects
    Medium,
    /// Could significantly impact system, requires caution
    High,
    /// Could break Windows, should only be used by advanced users
    Critical,
}

/// Registry hive types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RegistryHive {
    #[serde(rename = "HKCU")]
    Hkcu,
    #[serde(rename = "HKLM")]
    Hklm,
}

/// Registry value types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RegistryValueType {
    #[serde(rename = "REG_DWORD")]
    Dword,
    #[serde(rename = "REG_QWORD")]
    Qword,
    #[serde(rename = "REG_SZ")]
    String,
    #[serde(rename = "REG_EXPAND_SZ")]
    ExpandString,
    #[serde(rename = "REG_MULTI_SZ")]
    MultiString,
    #[serde(rename = "REG_BINARY")]
    Binary,
}

/// Action to perform on a registry key/value
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum RegistryAction {
    /// Set a registry value (default behavior)
    #[default]
    Set,
    /// Delete a specific registry value
    DeleteValue,
    /// Delete an entire registry key and all subkeys
    DeleteKey,
    /// Create a registry key without setting any value
    CreateKey,
}

/// Windows service startup type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStartupType {
    /// Service is disabled (cannot be started)
    Disabled,
    /// Service must be started manually
    Manual,
    /// Service starts automatically at boot
    Automatic,
    /// Kernel device driver (boot-start)
    Boot,
    /// Kernel device driver (system-start)
    System,
}

/// Action to perform on a scheduled task
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SchedulerAction {
    /// Enable a disabled scheduled task
    Enable,
    /// Disable a scheduled task
    Disable,
    /// Delete/unregister a scheduled task
    Delete,
}

// ============================================================================
// CORE STRUCTURES
// ============================================================================

/// Category definition from YAML header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub order: i32,
}

/// Single registry modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryChange {
    pub hive: RegistryHive,
    pub key: String,
    /// Value name (empty string targets the default value)
    #[serde(default)]
    pub value_name: String,
    /// Action to perform: set, delete_value, delete_key, create_key
    /// Defaults to "set" for backward compatibility
    #[serde(default)]
    pub action: RegistryAction,
    /// Value type - required for "set" action, ignored for delete/create actions
    #[serde(default)]
    pub value_type: Option<RegistryValueType>,
    /// Target value - required for "set" action, ignored for delete/create actions
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    /// Optional Windows version filter [10], [11], or [10, 11]
    #[serde(default)]
    pub windows_versions: Option<Vec<u32>>,
    /// If true, skip this change for tweak status validation and ignore failures during apply
    #[serde(default)]
    pub skip_validation: bool,
}

/// Single service modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceChange {
    /// Service name (e.g., "DiagTrack", "Spooler")
    pub name: String,
    /// Target startup type
    pub startup: ServiceStartupType,
    /// Stop the service after changing startup type
    #[serde(default)]
    pub stop_service: bool,
    /// Start the service after changing startup type
    #[serde(default)]
    pub start_service: bool,
    /// If true, skip this change for tweak status validation and ignore failures during apply
    #[serde(default)]
    pub skip_validation: bool,
}

/// Single scheduled task modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerChange {
    /// Task path (e.g., "\\Microsoft\\Windows\\Customer Experience Improvement Program")
    pub task_path: String,
    /// Exact task name (e.g., "Consolidator"). Mutually exclusive with task_name_pattern.
    #[serde(default)]
    pub task_name: Option<String>,
    /// Regex pattern to match multiple task names (e.g., "USO|Reboot|Refresh").
    /// Mutually exclusive with task_name. All matching tasks will have the action applied.
    #[serde(default)]
    pub task_name_pattern: Option<String>,
    /// Action to perform on the task(s)
    pub action: SchedulerAction,
    /// If true, skip this change for tweak status validation and ignore failures during apply
    #[serde(default)]
    pub skip_validation: bool,
    /// If true, don't error if task/path not found (useful for optional tasks)
    #[serde(default)]
    pub ignore_not_found: bool,
}

/// Action to perform on a hosts file entry
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum HostsAction {
    /// Add entry (or ensure it exists)
    Add,
    /// Remove entry if it exists
    Remove,
}

/// Single hosts file modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostsChange {
    /// IP address to map (e.g., "127.0.0.1", "0.0.0.0")
    pub ip: String,
    /// Domain/hostname to block or redirect (e.g., "telemetry.microsoft.com")
    pub domain: String,
    /// Action to perform: add or remove
    pub action: HostsAction,
    /// Optional comment to add after the entry (for documentation)
    #[serde(default)]
    pub comment: Option<String>,
    /// If true, skip this change for tweak status validation
    #[serde(default)]
    pub skip_validation: bool,
}

/// Direction for firewall rules
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FirewallDirection {
    /// Inbound traffic
    Inbound,
    /// Outbound traffic
    Outbound,
}

/// Action for firewall rules
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FirewallRuleAction {
    /// Block traffic
    Block,
    /// Allow traffic
    Allow,
}

/// Protocol for firewall rules
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FirewallProtocol {
    /// Any protocol
    Any,
    /// TCP only
    Tcp,
    /// UDP only
    Udp,
    /// ICMP
    Icmpv4,
    /// ICMPv6
    Icmpv6,
}

/// Firewall change operation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FirewallOperation {
    /// Create a new firewall rule
    Create,
    /// Delete an existing firewall rule by name
    Delete,
}

/// Single firewall rule modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirewallChange {
    /// Unique rule name (e.g., "Block DiagTrack Telemetry")
    pub name: String,
    /// Operation to perform: create or delete
    pub operation: FirewallOperation,
    /// Direction: inbound or outbound (required for create)
    #[serde(default)]
    pub direction: Option<FirewallDirection>,
    /// Action: block or allow (required for create)
    #[serde(default)]
    pub action: Option<FirewallRuleAction>,
    /// Protocol to match (defaults to any)
    #[serde(default)]
    pub protocol: Option<FirewallProtocol>,
    /// Program/executable path to match (optional)
    #[serde(default)]
    pub program: Option<String>,
    /// Service name to match (optional)
    #[serde(default)]
    pub service: Option<String>,
    /// Remote addresses to match (optional, e.g., "157.56.0.0/16")
    #[serde(default)]
    pub remote_addresses: Option<Vec<String>>,
    /// Remote ports to match (optional, e.g., "80,443")
    #[serde(default)]
    pub remote_ports: Option<String>,
    /// Local ports to match (optional)
    #[serde(default)]
    pub local_ports: Option<String>,
    /// Description for the rule
    #[serde(default)]
    pub description: Option<String>,
    /// If true, skip this change for tweak status validation
    #[serde(default)]
    pub skip_validation: bool,
}

/// A single option within a tweak - contains all changes for that state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TweakOption {
    /// Display label (e.g., "Enabled", "Disabled", "4MB")
    pub label: String,
    /// Registry modifications for this option
    #[serde(default)]
    pub registry_changes: Vec<RegistryChange>,
    /// Service modifications for this option
    #[serde(default)]
    pub service_changes: Vec<ServiceChange>,
    /// Scheduled task modifications for this option
    #[serde(default)]
    pub scheduler_changes: Vec<SchedulerChange>,
    /// Hosts file modifications for this option
    #[serde(default)]
    pub hosts_changes: Vec<HostsChange>,
    /// Firewall rule modifications for this option
    #[serde(default)]
    pub firewall_changes: Vec<FirewallChange>,
    /// Shell commands (cmd.exe) to run BEFORE applying changes
    #[serde(default)]
    pub pre_commands: Vec<String>,
    /// Shell commands (cmd.exe) to run AFTER applying changes
    #[serde(default)]
    pub post_commands: Vec<String>,
    /// PowerShell commands to run BEFORE applying changes (after pre_commands)
    #[serde(default)]
    pub pre_powershell: Vec<String>,
    /// PowerShell commands to run AFTER applying changes (after post_commands)
    #[serde(default)]
    pub post_powershell: Vec<String>,
    /// If true, treat missing registry keys/values as matching this option.
    /// Used for tweaks that modify registry entries which may not exist on all Windows editions.
    /// When a registry key/value doesn't exist and this flag is set, the option is considered
    /// a match (status is inferred rather than detected from actual values).
    #[serde(default)]
    pub registry_missing_is_match: bool,
    /// If true, treat missing services as matching this option.
    /// Used for tweaks that disable services which may not exist on all Windows editions.
    /// When a service doesn't exist and this flag is set, the option is considered
    /// a match (status is inferred rather than detected).
    #[serde(default)]
    pub service_missing_is_match: bool,
    /// If true, treat missing scheduled tasks as matching this option.
    /// Used for tweaks that disable tasks which may not exist on all Windows editions.
    /// When a task doesn't exist and this flag is set, the option is considered
    /// a match (status is inferred rather than detected).
    #[serde(default)]
    pub scheduler_missing_is_match: bool,
}

/// Complete tweak definition with category assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TweakDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub info: Option<String>,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub requires_system: bool,
    /// If true, run as TrustedInstaller (for protected services like WaaSMedicSvc)
    #[serde(default)]
    pub requires_ti: bool,
    #[serde(default)]
    pub requires_reboot: bool,
    /// If true, force dropdown display even for 2 options (default: false)
    /// By default, 2 options = toggle, 3+ options = dropdown
    #[serde(default)]
    pub force_dropdown: bool,
    /// Array of available states/options
    pub options: Vec<TweakOption>,
    /// Category this tweak belongs to
    #[serde(default)]
    pub category_id: String,
}
