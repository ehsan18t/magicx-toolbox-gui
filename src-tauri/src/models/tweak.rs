use serde::{Deserialize, Serialize};

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
    /// List of registry changes (with optional windows_versions filter on each)
    pub registry_changes: Vec<RegistryChange>,
    /// List of Windows service changes (start/stop, enable/disable)
    #[serde(default)]
    pub service_changes: Option<Vec<ServiceChange>>,
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

impl RegistryValueType {
    /// Get the registry type string (e.g., "REG_DWORD")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DWord => "REG_DWORD",
            Self::String => "REG_SZ",
            Self::ExpandString => "REG_EXPAND_SZ",
            Self::Binary => "REG_BINARY",
            Self::MultiString => "REG_MULTI_SZ",
            Self::QWord => "REG_QWORD",
        }
    }
}

/// Service change for a specific option (simplified: just target state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionServiceChange {
    /// Service name (e.g., "SysMain", "DiagTrack")
    pub name: String,
    /// Target startup type when this option is selected
    pub startup: ServiceStartupType,
    /// Stop the service if startup is disabled
    #[serde(default)]
    pub stop_if_disabled: bool,
}

/// Option for multi-state tweaks (displayed as dropdown in UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakOption {
    /// Display label for the option
    pub label: String,
    /// Registry value for this option
    pub value: serde_json::Value,
    /// Whether this is the default/original Windows value
    #[serde(default)]
    pub is_default: bool,
    /// Service changes specific to this option
    #[serde(default)]
    pub service_changes: Option<Vec<OptionServiceChange>>,
}

/// Single registry change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryChange {
    pub hive: RegistryHive,
    pub key: String,
    pub value_name: String,
    pub value_type: RegistryValueType,
    /// Value when tweak is enabled (for binary tweaks)
    pub enable_value: serde_json::Value,
    /// Value when tweak is disabled (for binary tweaks)
    #[serde(default)]
    pub disable_value: Option<serde_json::Value>,
    /// Optional Windows version filter. If None/empty, applies to all versions.
    /// Examples: [10], [11], [10, 11]
    #[serde(default)]
    pub windows_versions: Option<Vec<u32>>,
    /// Multi-state options (if present, displayed as dropdown instead of toggle)
    /// When options are present, enable_value/disable_value are ignored
    #[serde(default)]
    pub options: Option<Vec<TweakOption>>,
}

impl RegistryChange {
    /// Check if this registry change applies to a given Windows version
    pub fn applies_to_version(&self, version: u32) -> bool {
        match &self.windows_versions {
            None => true, // No filter = applies to all
            Some(versions) if versions.is_empty() => true,
            Some(versions) => versions.contains(&version),
        }
    }

    /// Check if this is a multi-state tweak (has options)
    pub fn is_multi_state(&self) -> bool {
        self.options.as_ref().map_or(false, |opts| opts.len() > 1)
    }

    /// Get the default option index (0 if not specified)
    pub fn default_option_index(&self) -> Option<usize> {
        self.options
            .as_ref()
            .and_then(|opts| opts.iter().position(|o| o.is_default).or(Some(0)))
    }
}

/// Windows service startup type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Single service change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChange {
    /// Service name (e.g., "wuauserv" for Windows Update)
    pub name: String,
    /// Startup type when tweak is enabled (applied)
    pub enable_startup: ServiceStartupType,
    /// Startup type when tweak is disabled (reverted)
    pub disable_startup: ServiceStartupType,
    /// Stop the service when applying the tweak
    #[serde(default)]
    pub stop_on_disable: bool,
    /// Start the service when reverting the tweak
    #[serde(default)]
    pub start_on_enable: bool,
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
    /// List of registry changes (with optional windows_versions filter on each)
    pub registry_changes: Vec<RegistryChange>,
    /// List of Windows service changes (start/stop, enable/disable)
    #[serde(default)]
    pub service_changes: Option<Vec<ServiceChange>>,
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
            service_changes: raw.service_changes,
            info: raw.info,
        }
    }

    /// Get registry changes filtered for a specific Windows version
    pub fn get_changes_for_version(&self, version: u32) -> Vec<&RegistryChange> {
        self.registry_changes
            .iter()
            .filter(|change| change.applies_to_version(version))
            .collect()
    }

    /// Check if this tweak has any registry changes for a given Windows version
    pub fn applies_to_version(&self, version: u32) -> bool {
        self.registry_changes
            .iter()
            .any(|change| change.applies_to_version(version))
    }

    /// Get all Windows versions this tweak applies to.
    /// Returns an empty Vec if the tweak applies to ALL Windows versions (no version filtering).
    /// Returns specific versions if any registry change specifies windows_versions.
    pub fn applicable_versions(&self) -> Vec<u32> {
        let mut versions = std::collections::HashSet::new();
        let mut has_specific_versions = false;

        for change in &self.registry_changes {
            if let Some(v) = &change.windows_versions {
                if !v.is_empty() {
                    has_specific_versions = true;
                    versions.extend(v.iter());
                }
            }
            // None or empty windows_versions = applies to all versions (no filtering)
        }

        // If no specific versions were found, return empty Vec to indicate "all versions"
        if !has_specific_versions {
            return Vec::new();
        }

        let mut result: Vec<_> = versions.into_iter().collect();
        result.sort();
        result
    }
}

/// Status of a tweak in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakStatus {
    pub tweak_id: String,
    pub is_applied: bool,
    pub last_applied: Option<String>, // ISO 8601 timestamp
    pub has_backup: bool,
    /// Current selected option index for multi-state tweaks (None for binary tweaks)
    #[serde(default)]
    pub current_option_index: Option<usize>,
}

/// Result of applying or reverting a tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakResult {
    pub success: bool,
    pub message: String,
    pub requires_reboot: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a RegistryChange for testing
    fn make_registry_change(windows_versions: Option<Vec<u32>>) -> RegistryChange {
        RegistryChange {
            hive: RegistryHive::HKCU,
            key: "SOFTWARE\\Test".to_string(),
            value_name: "Value".to_string(),
            value_type: RegistryValueType::DWord,
            enable_value: serde_json::json!(1),
            disable_value: Some(serde_json::json!(0)),
            windows_versions,
            options: None,
        }
    }

    // Tests for RegistryChange::applies_to_version
    #[test]
    fn test_registry_change_applies_to_version_none() {
        let change = make_registry_change(None);
        assert!(change.applies_to_version(10));
        assert!(change.applies_to_version(11));
        assert!(change.applies_to_version(12)); // Future version
    }

    #[test]
    fn test_registry_change_applies_to_version_empty() {
        let change = make_registry_change(Some(vec![]));
        assert!(change.applies_to_version(10));
        assert!(change.applies_to_version(11));
    }

    #[test]
    fn test_registry_change_applies_to_version_specific() {
        let change = make_registry_change(Some(vec![11]));
        assert!(!change.applies_to_version(10));
        assert!(change.applies_to_version(11));
    }

    #[test]
    fn test_registry_change_applies_to_version_multiple() {
        let change = make_registry_change(Some(vec![10, 11]));
        assert!(change.applies_to_version(10));
        assert!(change.applies_to_version(11));
        assert!(!change.applies_to_version(12));
    }

    // Helper to create TweakDefinition for testing
    fn make_tweak_definition(changes: Vec<RegistryChange>) -> TweakDefinition {
        TweakDefinition {
            id: "test_tweak".to_string(),
            name: "Test Tweak".to_string(),
            description: "A test tweak".to_string(),
            category: "test".to_string(),
            risk_level: RiskLevel::Low,
            requires_reboot: false,
            requires_admin: false,
            registry_changes: changes,
            service_changes: None,
            info: None,
        }
    }

    #[test]
    fn test_tweak_applies_to_version_universal() {
        let tweak = make_tweak_definition(vec![make_registry_change(None)]);
        assert!(tweak.applies_to_version(10));
        assert!(tweak.applies_to_version(11));
    }

    #[test]
    fn test_tweak_applies_to_version_specific() {
        let tweak = make_tweak_definition(vec![make_registry_change(Some(vec![11]))]);
        assert!(!tweak.applies_to_version(10));
        assert!(tweak.applies_to_version(11));
    }

    #[test]
    fn test_applicable_versions_universal() {
        let tweak = make_tweak_definition(vec![make_registry_change(None)]);
        let versions = tweak.applicable_versions();
        assert!(versions.is_empty()); // Empty = all versions
    }

    #[test]
    fn test_applicable_versions_specific() {
        let tweak = make_tweak_definition(vec![
            make_registry_change(Some(vec![10])),
            make_registry_change(Some(vec![11])),
        ]);
        let versions = tweak.applicable_versions();
        assert_eq!(versions, vec![10, 11]);
    }

    #[test]
    fn test_get_changes_for_version() {
        let tweak = make_tweak_definition(vec![
            make_registry_change(Some(vec![10])),
            make_registry_change(Some(vec![11])),
            make_registry_change(None), // universal
        ]);
        let win10_changes = tweak.get_changes_for_version(10);
        assert_eq!(win10_changes.len(), 2); // specific + universal

        let win11_changes = tweak.get_changes_for_version(11);
        assert_eq!(win11_changes.len(), 2); // specific + universal
    }

    #[test]
    fn test_applicable_versions_mixed() {
        // Mix of universal and specific changes
        let tweak = make_tweak_definition(vec![
            make_registry_change(None),           // universal
            make_registry_change(Some(vec![11])), // Win11 only
        ]);
        // Since there's a universal change, tweak applies to all versions
        assert!(tweak.applies_to_version(10));
        assert!(tweak.applies_to_version(11));
    }
}
