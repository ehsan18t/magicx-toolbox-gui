//! Primitive-facing Windows types (hive, value, startup, action) spoken by the `services`
//! primitives and the elevation broker wire protocol. Kept separate from `tweaks::model`
//! (`Hive`/`RegType`, the compiled-corpus form); the `From` impls below translate between the two.

use serde::{Deserialize, Serialize};

use crate::tweaks::model::{FwAction, FwDirection, FwProtocol, Hive, RegType};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum SchedulerAction {
    Enable,
    Disable,
}

impl SchedulerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulerAction::Enable => "enable",
            SchedulerAction::Disable => "disable",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum HostsAction {
    Add,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum FirewallDirection {
    Inbound,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum FirewallRuleAction {
    Block,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum FirewallProtocol {
    Any,
    Tcp,
    Udp,
    Icmpv4,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum FirewallOperation {
    Create,
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

impl From<Hive> for RegistryHive {
    fn from(hive: Hive) -> Self {
        match hive {
            Hive::Hklm => Self::Hklm,
            Hive::Hkcu => Self::Hkcu,
        }
    }
}

impl From<RegType> for RegistryValueType {
    fn from(ty: RegType) -> Self {
        match ty {
            RegType::Dword => Self::Dword,
            RegType::Qword => Self::Qword,
            RegType::Sz => Self::String,
            RegType::ExpandSz => Self::ExpandString,
            RegType::MultiSz => Self::MultiString,
            RegType::Binary => Self::Binary,
        }
    }
}

impl From<FwDirection> for FirewallDirection {
    fn from(d: FwDirection) -> Self {
        match d {
            FwDirection::Inbound => Self::Inbound,
            FwDirection::Outbound => Self::Outbound,
        }
    }
}

impl From<FwAction> for FirewallRuleAction {
    fn from(a: FwAction) -> Self {
        match a {
            FwAction::Block => Self::Block,
            FwAction::Allow => Self::Allow,
        }
    }
}

impl From<FwProtocol> for FirewallProtocol {
    fn from(p: FwProtocol) -> Self {
        match p {
            FwProtocol::Any => Self::Any,
            FwProtocol::Tcp => Self::Tcp,
            FwProtocol::Udp => Self::Udp,
            FwProtocol::Icmpv4 => Self::Icmpv4,
            FwProtocol::Icmpv6 => Self::Icmpv6,
        }
    }
}
