//! Shared low-level Windows types (Task 15 pre-work): the primitive-facing hive/value/startup/
//! action types that `services::{registry_service, registry_value, service_control,
//! scheduler_service, hosts_service, firewall_service}` and the elevation broker speak, extracted
//! verbatim from the deleted option-centric `models::tweak_schema`/`models::tweak` so those
//! primitives (spec §11: "the trusted low-level primitives are reused") keep compiling once the
//! rest of the old YAML schema is gone. This file is deliberately NOT the new engine's own model
//! (`tweaks::model`) — `RegistryHive`/`RegistryValueType`/etc. here are what the OS-facing
//! primitives and the broker wire protocol use; `tweaks::model::{Hive, RegType, ...}` is the
//! compiled-corpus representation the kinds translate to/from (see e.g. `tweaks/kinds/registry.rs`'s
//! `old_hive`/`old_type` conversions). Kept separate on purpose, same reasoning `tweaks/model.rs`'s
//! own module docs already give for `Hive`/`RegType` vs. these.

use serde::{Deserialize, Serialize};

/// Registry hive types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RegistryHive {
    #[serde(rename = "HKCU")]
    Hkcu,
    #[serde(rename = "HKLM")]
    Hklm,
}

impl RegistryHive {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryHive::Hkcu => "HKCU",
            RegistryHive::Hklm => "HKLM",
        }
    }
}

/// Registry value types.
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

/// Windows service startup type.
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

/// Action to perform on a scheduled task.
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

impl SchedulerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulerAction::Enable => "enable",
            SchedulerAction::Disable => "disable",
            SchedulerAction::Delete => "delete",
        }
    }
}

/// Action to perform on a hosts file entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum HostsAction {
    /// Add entry (or ensure it exists)
    Add,
    /// Remove entry if it exists
    Remove,
}

impl HostsAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            HostsAction::Add => "add",
            HostsAction::Remove => "remove",
        }
    }
}

/// Single hosts file modification — the shape `services::hosts_service::apply_hosts_change` acts
/// on.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Direction for firewall rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FirewallDirection {
    /// Inbound traffic
    Inbound,
    /// Outbound traffic
    Outbound,
}

impl FirewallDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallDirection::Inbound => "in",
            FirewallDirection::Outbound => "out",
        }
    }
}

/// Action for firewall rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FirewallRuleAction {
    /// Block traffic
    Block,
    /// Allow traffic
    Allow,
}

impl FirewallRuleAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallRuleAction::Block => "block",
            FirewallRuleAction::Allow => "allow",
        }
    }
}

/// Protocol for firewall rules.
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

/// Firewall change operation type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FirewallOperation {
    /// Create a new firewall rule
    Create,
    /// Delete an existing firewall rule by name
    Delete,
}

impl FirewallOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirewallOperation::Create => "create",
            FirewallOperation::Delete => "delete",
        }
    }
}

/// Single firewall rule modification — the shape `services::firewall_service::create_firewall_rule`
/// acts on (spec §11: `firewall_service` is a reused, wrapped primitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
