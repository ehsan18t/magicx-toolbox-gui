use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }
}

/// Categories for organizing tweaks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TweakCategory {
    Privacy,
    Performance,
    UI,
    Security,
    Services,
    Gaming,
    #[serde(other)]
    Other(String),
}

impl std::fmt::Display for TweakCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TweakCategory::Privacy => write!(f, "Privacy"),
            TweakCategory::Performance => write!(f, "Performance"),
            TweakCategory::UI => write!(f, "UI/UX"),
            TweakCategory::Security => write!(f, "Security"),
            TweakCategory::Services => write!(f, "Services"),
            TweakCategory::Gaming => write!(f, "Gaming"),
            TweakCategory::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Windows version enum for version-specific tweaks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WindowsVersionTarget {
    #[serde(rename = "10")]
    Win10,
    #[serde(rename = "11")]
    Win11,
}

impl WindowsVersionTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowsVersionTarget::Win10 => "10",
            WindowsVersionTarget::Win11 => "11",
        }
    }
}

/// Registry hive types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RegistryHive {
    HKCU, // HKEY_CURRENT_USER
    HKLM, // HKEY_LOCAL_MACHINE
}

impl RegistryHive {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryHive::HKCU => "HKCU",
            RegistryHive::HKLM => "HKLM",
        }
    }
}

/// Single registry value change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryValueChange {
    pub key: String,
    pub value_name: String,
    pub value_type: RegistryValueType,
    pub value_data: serde_json::Value,
}

/// Registry value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RegistryValueType {
    #[serde(rename = "REG_DWORD")]
    DWord,
    #[serde(rename = "REG_SZ")]
    String,
    #[serde(rename = "REG_EXPAND_SZ")]
    ExpandString,
    #[serde(rename = "REG_BINARY")]
    Binary,
    #[serde(rename = "REG_MULTI_SZ")]
    MultiString,
    #[serde(rename = "REG_QWORD")]
    QWord,
}

/// Single registry change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryChange {
    pub hive: RegistryHive,
    pub key: String,
    pub value_name: String,
    pub value_type: RegistryValueType,
    pub enable_value: serde_json::Value,
    #[serde(default)]
    pub disable_value: Option<serde_json::Value>,
}

/// A complete tweak definition loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TweakCategory,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub requires_reboot: bool,
    #[serde(default)]
    pub requires_admin: bool,
    /// Map of Windows version to registry changes
    pub registry_changes: HashMap<String, Vec<RegistryChange>>,
    /// Additional info/documentation
    #[serde(default)]
    pub info: Option<String>,
}

impl TweakDefinition {
    /// Get registry changes for a specific Windows version
    pub fn get_changes_for_version(&self, version: &str) -> Option<&Vec<RegistryChange>> {
        self.registry_changes.get(version)
    }

    /// Check if this tweak applies to a given Windows version
    pub fn applies_to_version(&self, version: &str) -> bool {
        self.registry_changes.contains_key(version)
    }

    /// Get all applicable versions
    pub fn applicable_versions(&self) -> Vec<String> {
        self.registry_changes.keys().cloned().collect()
    }
}

/// Status of a tweak in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakStatus {
    pub tweak_id: String,
    pub is_applied: bool,
    pub last_applied: Option<String>, // ISO 8601 timestamp
    pub has_backup: bool,
}
