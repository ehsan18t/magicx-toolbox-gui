//! Profile Models
//!
//! Data structures for configuration profiles - user-exportable tweak selections
//! that can be imported across machines and Windows versions.

use serde::{Deserialize, Serialize};

/// Current schema version for profiles
pub const PROFILE_SCHEMA_VERSION: u32 = 1;

/// File extension for profile archives
pub const PROFILE_EXTENSION: &str = "mgx";

/// MIME type for profile archives
pub const PROFILE_MIME_TYPE: &str = "application/x-magicx-profile";

// ============================================================================
// CONFIGURATION PROFILE (User's Intent - What they WANT)
// ============================================================================

/// A user-exportable configuration profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProfile {
    /// Schema version for migration support
    pub schema_version: u32,

    /// Profile metadata
    pub metadata: ProfileMetadata,

    /// Tweak selections (tweak_id → selected option)
    /// Only stores tweaks that differ from system default
    pub selections: Vec<TweakSelection>,
}

impl ConfigurationProfile {
    /// Create a new profile with the given metadata and selections
    pub fn new(metadata: ProfileMetadata, selections: Vec<TweakSelection>) -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            metadata,
            selections,
        }
    }

    /// Check if this profile needs migration
    pub fn needs_migration(&self) -> bool {
        self.schema_version < PROFILE_SCHEMA_VERSION
    }
}

/// Profile metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// User-provided profile name
    pub name: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// When profile was created (ISO 8601)
    pub created_at: String,
    /// When profile was last modified (ISO 8601)
    pub modified_at: String,
    /// App version that created this profile
    pub app_version: String,
    /// Source Windows version (10 or 11)
    pub source_windows_version: u32,
    /// Source Windows build number
    pub source_windows_build: u32,
    /// Unique machine identifier (optional, for sync features)
    #[serde(default)]
    pub source_machine_id: Option<String>,
}

impl ProfileMetadata {
    /// Create new metadata with current timestamp
    pub fn new(
        name: String,
        description: Option<String>,
        app_version: String,
        windows_version: u32,
        windows_build: u32,
    ) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            name,
            description,
            created_at: now.clone(),
            modified_at: now,
            app_version,
            source_windows_version: windows_version,
            source_windows_build: windows_build,
            source_machine_id: None,
        }
    }
}

/// A single tweak selection in a profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakSelection {
    /// Tweak ID (stable identifier)
    pub tweak_id: String,
    /// Selected option index at time of export
    pub selected_option_index: usize,
    /// Option label for human reference (informational only)
    pub selected_option_label: String,
    /// Hash of the option's content for detecting schema changes
    #[serde(default)]
    pub option_content_hash: Option<String>,
    /// Category ID for grouping in UI
    #[serde(default)]
    pub category_id: Option<String>,
}

// ============================================================================
// ARCHIVE MANIFEST
// ============================================================================

/// Manifest file in the archive root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManifest {
    /// Archive format version
    pub format_version: u32,
    /// Checksum of profile.json
    pub profile_checksum: String,
    /// Whether system_state.json is included
    pub includes_system_state: bool,
    /// Checksum of system_state.json (if included)
    #[serde(default)]
    pub system_state_checksum: Option<String>,
}

// ============================================================================
// SYSTEM STATE SNAPSHOT (What the System HAS - for validation/preview)
// ============================================================================

/// Complete system state snapshot for validation before restore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStateSnapshot {
    /// Schema version
    pub schema_version: u32,
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    /// Raw registry values relevant to known tweaks
    pub registry_state: Vec<RegistryValueState>,
    /// Service configurations
    pub service_state: Vec<ServiceState>,
    /// Scheduled task states
    pub scheduler_state: Vec<SchedulerState>,
}

/// Metadata for a system state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// When snapshot was created (ISO 8601)
    pub created_at: String,
    /// App version
    pub app_version: String,
    /// Windows version (10 or 11)
    pub windows_version: u32,
    /// Windows build number
    pub windows_build: u32,
    /// Machine hostname
    pub machine_name: String,
}

/// State of a single registry value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryValueState {
    /// Registry hive (HKCU, HKLM)
    pub hive: String,
    /// Key path
    pub key: String,
    /// Value name
    pub value_name: String,
    /// Value type (REG_DWORD, etc.)
    pub value_type: Option<String>,
    /// Current value
    pub value: Option<serde_json::Value>,
    /// Whether the value exists
    pub exists: bool,
}

/// State of a Windows service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    /// Service name
    pub name: String,
    /// Current startup type
    pub startup_type: String,
    /// Whether service is running
    pub is_running: bool,
    /// Whether service exists
    pub exists: bool,
}

/// State of a scheduled task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerState {
    /// Task path
    pub task_path: String,
    /// Task name
    pub task_name: String,
    /// Current state (Ready, Disabled, NotFound)
    pub state: String,
    /// Whether task exists
    pub exists: bool,
}

// ============================================================================
// VALIDATION TYPES
// ============================================================================

/// Result of validating a profile against current system
#[derive(Debug, Clone, Serialize)]
pub struct ProfileValidation {
    /// Overall validity status
    pub is_valid: bool,
    /// Profile can be partially applied
    pub is_partially_applicable: bool,
    /// Warnings that don't prevent import
    pub warnings: Vec<ValidationWarning>,
    /// Errors that prevent import of specific tweaks
    pub errors: Vec<ValidationError>,
    /// Preview of what would change
    pub preview: Vec<TweakChangePreview>,
    /// Summary statistics
    pub stats: ValidationStats,
}

/// A validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Affected tweak ID
    pub tweak_id: String,
    /// Warning code
    pub code: WarningCode,
    /// Human-readable message
    pub message: String,
}

/// Warning codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WarningCode {
    /// Tweak schema has changed since export
    TweakSchemaChanged,
    /// Windows version mismatch (may still work)
    WindowsVersionMismatch,
    /// Registry key doesn't exist but can be created
    RegistryKeyMissing,
    /// Tweak is already at desired state
    AlreadyApplied,
    /// Option resolved by hash instead of index
    OptionResolvedByHash,
    /// Tweak resolved by alias instead of primary ID
    TweakResolvedByAlias,
}

/// A validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Affected tweak ID
    pub tweak_id: String,
    /// Error code
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
}

/// Error codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Tweak ID no longer exists in app
    TweakNotFound,
    /// Option index out of bounds and no hash match
    InvalidOptionIndex,
    /// Service doesn't exist on this system
    ServiceNotFound,
    /// Scheduled task doesn't exist on this system
    TaskNotFound,
    /// Registry hive inaccessible
    RegistryAccessDenied,
    /// Tweak requires permissions not available
    InsufficientPermissions,
    /// Windows version incompatible
    WindowsVersionIncompatible,
    /// Profile schema version too new
    SchemaVersionTooNew,
    /// Archive is corrupted or invalid
    InvalidArchive,
    /// Checksum mismatch
    ChecksumMismatch,
}

/// Preview of what would change for a single tweak
#[derive(Debug, Clone, Serialize)]
pub struct TweakChangePreview {
    /// Tweak ID
    pub tweak_id: String,
    /// Tweak name for display
    pub tweak_name: String,
    /// Category ID
    pub category_id: String,
    /// Original Tweak ID from profile (if resolved by alias)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_tweak_id: Option<String>,
    /// Current option on target system (None if unknown)
    pub current_option_index: Option<usize>,
    /// Current option label
    pub current_option_label: Option<String>,
    /// Desired option from profile
    pub target_option_index: usize,
    /// Target option label
    pub target_option_label: String,
    /// Whether this tweak can be applied
    pub applicable: bool,
    /// Reason if not applicable
    pub skip_reason: Option<String>,
    /// Risk level of this tweak
    pub risk_level: String,
    /// Whether this is already at desired state
    pub already_applied: bool,
    /// Whether this tweak has commands that will be skipped during profile apply
    #[serde(default)]
    pub has_skipped_commands: bool,
    /// Detailed changes that would be made
    pub changes: Vec<ChangeDetail>,
}

/// Detail of a single change
#[derive(Debug, Clone, Serialize)]
pub struct ChangeDetail {
    /// Type of change
    pub change_type: ChangeType,
    /// Human-readable description
    pub description: String,
    /// Current value (if applicable)
    pub current_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
}

/// Types of changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Registry,
    Service,
    ScheduledTask,
    Command,
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Default)]
pub struct ValidationStats {
    /// Total tweaks in profile
    pub total_tweaks: usize,
    /// Tweaks that can be applied
    pub applicable_tweaks: usize,
    /// Tweaks that will be skipped due to errors
    pub skipped_tweaks: usize,
    /// Tweaks already at desired state
    pub already_applied: usize,
    /// Tweaks with warnings
    pub tweaks_with_warnings: usize,
}

// ============================================================================
// APPLY RESULT
// ============================================================================

/// Result of applying a profile
#[derive(Debug, Clone, Serialize)]
pub struct ProfileApplyResult {
    /// Overall success
    pub success: bool,
    /// Number of tweaks successfully applied
    pub applied_count: usize,
    /// Number of tweaks skipped
    pub skipped_count: usize,
    /// Number of tweaks that failed
    pub failed_count: usize,
    /// Details of failures
    pub failures: Vec<ApplyFailure>,
    /// Whether any applied tweak requires reboot
    pub requires_reboot: bool,
    /// Tweaks that require reboot
    pub reboot_required_tweaks: Vec<String>,
}

/// Details of a failed tweak application
#[derive(Debug, Clone, Serialize)]
pub struct ApplyFailure {
    /// Tweak ID
    pub tweak_id: String,
    /// Tweak name
    pub tweak_name: String,
    /// Error message
    pub error: String,
    /// Whether partial changes were made (and rolled back)
    pub was_rolled_back: bool,
}

// ============================================================================
// PROGRESS EVENTS
// ============================================================================

/// Progress event during profile application
#[derive(Debug, Clone, Serialize)]
pub struct ProfileProgressEvent {
    /// Current tweak being processed
    pub current_tweak_id: String,
    /// Current tweak name
    pub current_tweak_name: String,
    /// Current step within the tweak
    pub current_step: String,
    /// Progress (0-100) for current tweak
    pub tweak_progress: u8,
    /// Overall progress (0-100)
    pub overall_progress: u8,
    /// Number of tweaks completed
    pub completed: usize,
    /// Total tweaks to process
    pub total: usize,
}

/// Event when a tweak completes
#[derive(Debug, Clone, Serialize)]
pub struct TweakCompleteEvent {
    /// Tweak ID
    pub tweak_id: String,
    /// Whether it succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

// ============================================================================
// EXPORT OPTIONS
// ============================================================================

/// Options for exporting a profile
#[derive(Debug, Clone, Deserialize)]
pub struct ExportOptions {
    /// Profile name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Tweak IDs to include (empty = all applied)
    pub tweak_ids: Vec<String>,
    /// Whether to include system state snapshot
    pub include_system_state: bool,
}

/// Options for applying a profile
#[derive(Debug, Clone, Deserialize)]
pub struct ApplyOptions {
    /// Tweak IDs to skip
    #[serde(default)]
    pub skip_tweak_ids: Vec<String>,
    /// Whether to create a Windows restore point
    #[serde(default)]
    pub create_restore_point: bool,
    /// Whether to skip tweaks already at desired state
    #[serde(default = "default_true")]
    pub skip_already_applied: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ApplyOptions {
    fn default() -> Self {
        Self {
            skip_tweak_ids: Vec::new(),
            create_restore_point: false,
            skip_already_applied: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let metadata = ProfileMetadata::new(
            "Test Profile".to_string(),
            Some("A test profile".to_string()),
            "3.0.0".to_string(),
            11,
            22631,
        );

        let selections = vec![TweakSelection {
            tweak_id: "disable_telemetry".to_string(),
            selected_option_index: 0,
            selected_option_label: "Disabled".to_string(),
            option_content_hash: None,
            category_id: Some("privacy".to_string()),
        }];

        let profile = ConfigurationProfile::new(metadata, selections);

        assert_eq!(profile.schema_version, PROFILE_SCHEMA_VERSION);
        assert_eq!(profile.selections.len(), 1);
        assert!(!profile.needs_migration());
    }

    #[test]
    fn test_needs_migration() {
        let mut profile = ConfigurationProfile {
            schema_version: 0,
            metadata: ProfileMetadata::new(
                "Old Profile".to_string(),
                None,
                "2.0.0".to_string(),
                10,
                19045,
            ),
            selections: Vec::new(),
        };

        assert!(profile.needs_migration());

        profile.schema_version = PROFILE_SCHEMA_VERSION;
        assert!(!profile.needs_migration());
    }
}
