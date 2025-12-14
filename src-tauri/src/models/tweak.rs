//! Tweak model definitions for the unified option-based tweak system.
//!
//! Every tweak has an array of options, each containing its own registry changes,
//! service changes, and commands. `is_toggle: true` renders as a switch (2 options),
//! otherwise as a dropdown.

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

/// Registry hive types
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

impl SchedulerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulerAction::Enable => "enable",
            SchedulerAction::Disable => "disable",
            SchedulerAction::Delete => "delete",
        }
    }
}

// ============================================================================
// CORE STRUCTURES
// ============================================================================

/// Category definition from YAML header
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct RegistryChange {
    pub hive: RegistryHive,
    pub key: String,
    pub value_name: String,
    pub value_type: RegistryValueType,
    /// Target value for this registry entry
    pub value: serde_json::Value,
    /// Optional Windows version filter [10], [11], or [10, 11]
    #[serde(default)]
    pub windows_versions: Option<Vec<u32>>,
    /// If true, skip this change for tweak status validation and ignore failures during apply
    #[serde(default)]
    pub skip_validation: bool,
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

/// Single service modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A single option within a tweak - contains all changes for that state
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl TweakOption {
    /// Get registry changes filtered for a specific Windows version
    pub fn get_registry_changes_for_version(&self, version: u32) -> Vec<&RegistryChange> {
        self.registry_changes
            .iter()
            .filter(|change| change.applies_to_version(version))
            .collect()
    }

    /// Check if this option has any effective changes for the given Windows version
    pub fn has_changes_for_version(&self, version: u32) -> bool {
        let has_registry = self
            .registry_changes
            .iter()
            .any(|c| c.applies_to_version(version));
        let has_services = !self.service_changes.is_empty();
        let has_scheduler = !self.scheduler_changes.is_empty();
        let has_commands = !self.pre_commands.is_empty() || !self.post_commands.is_empty();
        let has_powershell = !self.pre_powershell.is_empty() || !self.post_powershell.is_empty();
        has_registry || has_services || has_scheduler || has_commands || has_powershell
    }
}

/// Raw tweak definition as loaded from YAML (before category assignment)
#[derive(Debug, Clone, Deserialize)]
pub struct TweakDefinitionRaw {
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
    /// If true, display as toggle switch (must have exactly 2 options)
    /// If false, display as dropdown
    #[serde(default)]
    pub is_toggle: bool,
    /// Array of available states/options
    pub options: Vec<TweakOption>,
}

/// Complete tweak definition with category assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// If true, display as toggle switch (must have exactly 2 options)
    /// If false, display as dropdown
    #[serde(default)]
    pub is_toggle: bool,
    /// Array of available states/options
    pub options: Vec<TweakOption>,
    /// Category this tweak belongs to
    #[serde(default)]
    pub category_id: String,
}

impl TweakDefinition {
    /// Create from raw definition and category id
    pub fn from_raw(raw: TweakDefinitionRaw, category_id: &str) -> Self {
        TweakDefinition {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            info: raw.info,
            risk_level: raw.risk_level,
            requires_admin: raw.requires_admin,
            requires_system: raw.requires_system,
            requires_ti: raw.requires_ti,
            requires_reboot: raw.requires_reboot,
            is_toggle: raw.is_toggle,
            options: raw.options,
            category_id: category_id.to_string(),
        }
    }

    /// Validate tweak structure
    pub fn validate(&self) -> Result<(), String> {
        if self.options.is_empty() {
            return Err(format!("Tweak '{}' must have at least 1 option", self.id));
        }
        if self.is_toggle && self.options.len() != 2 {
            return Err(format!(
                "Toggle tweak '{}' must have exactly 2 options, found {}",
                self.id,
                self.options.len()
            ));
        }
        Ok(())
    }

    /// Get all unique registry keys across all options (for state detection)
    pub fn all_registry_keys(&self) -> Vec<(RegistryHive, String, String)> {
        let mut keys = Vec::new();
        for option in &self.options {
            for change in &option.registry_changes {
                let key = (change.hive, change.key.clone(), change.value_name.clone());
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }

    /// Get all unique service names across all options
    pub fn all_service_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for option in &self.options {
            for change in &option.service_changes {
                if !names.contains(&change.name) {
                    names.push(change.name.clone());
                }
            }
        }
        names
    }

    /// Check if this tweak applies to a given Windows version
    /// (has at least one option with changes for that version)
    pub fn applies_to_version(&self, version: u32) -> bool {
        self.options
            .iter()
            .any(|opt| opt.has_changes_for_version(version))
    }

    /// Get all Windows versions this tweak applies to
    /// Returns empty Vec if applies to all versions
    pub fn applicable_versions(&self) -> Vec<u32> {
        let mut versions = std::collections::HashSet::new();
        let mut has_specific = false;

        for option in &self.options {
            for change in &option.registry_changes {
                if let Some(v) = &change.windows_versions {
                    if !v.is_empty() {
                        has_specific = true;
                        versions.extend(v.iter());
                    }
                }
            }
        }

        if !has_specific {
            return Vec::new();
        }

        let mut result: Vec<_> = versions.into_iter().collect();
        result.sort();
        result
    }
}

/// YAML file structure with category and tweaks
#[derive(Debug, Clone, Deserialize)]
pub struct TweakFile {
    pub category: CategoryDefinition,
    pub tweaks: Vec<TweakDefinitionRaw>,
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
    /// Error message if state detection failed (tweak still returned but with unknown state)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
            value_type: RegistryValueType::Dword,
            value: serde_json::json!(value),
            windows_versions,
            skip_validation: false,
        }
    }

    fn make_option(registry_changes: Vec<RegistryChange>) -> TweakOption {
        TweakOption {
            label: "Test".to_string(),
            registry_changes,
            service_changes: Vec::new(),
            scheduler_changes: Vec::new(),
            pre_commands: Vec::new(),
            post_commands: Vec::new(),
            pre_powershell: Vec::new(),
            post_powershell: Vec::new(),
        }
    }

    fn make_tweak(is_toggle: bool, options: Vec<TweakOption>) -> TweakDefinition {
        TweakDefinition {
            id: "test_tweak".to_string(),
            name: "Test Tweak".to_string(),
            description: "A test tweak".to_string(),
            info: None,
            risk_level: RiskLevel::Low,
            requires_admin: false,
            requires_system: false,
            requires_ti: false,
            requires_reboot: false,
            is_toggle,
            options,
            category_id: "test".to_string(),
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

    #[test]
    fn test_toggle_validation() {
        let tweak = make_tweak(
            true,
            vec![
                make_option(vec![make_registry_change(1, None)]),
                make_option(vec![make_registry_change(0, None)]),
            ],
        );
        assert!(tweak.validate().is_ok());

        let tweak = make_tweak(true, vec![make_option(vec![make_registry_change(1, None)])]);
        assert!(tweak.validate().is_err());
    }

    #[test]
    fn test_dropdown_validation() {
        let tweak = make_tweak(
            false,
            vec![
                make_option(vec![make_registry_change(1, None)]),
                make_option(vec![make_registry_change(2, None)]),
                make_option(vec![make_registry_change(3, None)]),
            ],
        );
        assert!(tweak.validate().is_ok());
    }

    #[test]
    fn test_all_registry_keys() {
        let tweak = make_tweak(
            false,
            vec![
                make_option(vec![
                    make_registry_change(1, None),
                    RegistryChange {
                        hive: RegistryHive::Hklm,
                        key: "SOFTWARE\\Other".to_string(),
                        value_name: "Other".to_string(),
                        value_type: RegistryValueType::Dword,
                        value: serde_json::json!(1),
                        windows_versions: None,
                        skip_validation: false,
                    },
                ]),
                make_option(vec![make_registry_change(2, None)]),
            ],
        );
        let keys = tweak.all_registry_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_applicable_versions() {
        // Universal tweak
        let tweak = make_tweak(
            true,
            vec![
                make_option(vec![make_registry_change(1, None)]),
                make_option(vec![make_registry_change(0, None)]),
            ],
        );
        assert!(tweak.applicable_versions().is_empty());

        // Version-specific tweak
        let tweak = make_tweak(
            true,
            vec![
                make_option(vec![make_registry_change(1, Some(vec![11]))]),
                make_option(vec![make_registry_change(0, Some(vec![11]))]),
            ],
        );
        assert_eq!(tweak.applicable_versions(), vec![11]);
    }
}
