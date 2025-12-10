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

/// Category definition loaded from YAML file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub order: i32,
}

/// Complete tweak file structure with category metadata and tweaks
#[derive(Debug, Clone, Deserialize)]
pub struct TweakFile {
    pub category: CategoryDefinition,
    pub tweaks: Vec<TweakDefinitionRaw>,
}

/// Raw tweak definition as loaded from YAML (without category field)
#[derive(Debug, Clone, Deserialize)]
pub struct TweakDefinitionRaw {
    pub id: String,
    pub name: String,
    pub description: String,
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
    pub category: String, // Now a dynamic string instead of enum
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
    /// Create from raw definition and category id
    pub fn from_raw(raw: TweakDefinitionRaw, category_id: &str) -> Self {
        TweakDefinition {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            category: category_id.to_string(),
            risk_level: raw.risk_level,
            requires_reboot: raw.requires_reboot,
            requires_admin: raw.requires_admin,
            registry_changes: raw.registry_changes,
            info: raw.info,
        }
    }

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

/// Result of applying or reverting a tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakResult {
    pub success: bool,
    pub message: String,
    pub requires_reboot: bool,
}
