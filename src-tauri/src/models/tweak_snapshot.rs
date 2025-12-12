//! Tweak Snapshot Models
//!
//! Snapshot-based storage for registry/service state before tweak application.
//! Used for atomic rollback to the exact state before any changes were made.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Snapshot of a single registry value before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    /// Registry hive (HKCU, HKLM)
    pub hive: String,
    /// Registry key path
    pub key: String,
    /// Value name
    pub value_name: String,
    /// Value type (REG_DWORD, REG_SZ, etc.) - None if didn't exist
    pub value_type: Option<String>,
    /// The value before modification - None if didn't exist
    pub value: Option<Value>,
    /// Whether the value existed before modification
    pub existed: bool,
}

/// Snapshot of a service's state before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSnapshot {
    /// Service name
    pub name: String,
    /// Startup type before modification (disabled, manual, automatic, etc.)
    pub startup_type: String,
    /// Whether the service was running before modification
    pub was_running: bool,
}

/// Complete snapshot of system state before applying a tweak option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakSnapshot {
    /// Tweak ID this snapshot belongs to
    pub tweak_id: String,
    /// Human-readable tweak name (for reference)
    pub tweak_name: String,
    /// Which option index was applied
    pub applied_option_index: usize,
    /// Option label that was applied (for reference)
    pub applied_option_label: String,
    /// Timestamp when snapshot was created (ISO 8601)
    pub created_at: String,
    /// Windows version when snapshot was created (10 or 11)
    pub windows_version: u32,
    /// Whether SYSTEM elevation was used for this tweak
    #[serde(default)]
    pub requires_system: bool,
    /// Registry values captured before changes
    pub registry_snapshots: Vec<RegistrySnapshot>,
    /// Service states captured before changes
    pub service_snapshots: Vec<ServiceSnapshot>,
}

impl TweakSnapshot {
    /// Create a new empty snapshot for a tweak
    pub fn new(
        tweak_id: &str,
        tweak_name: &str,
        applied_option_index: usize,
        applied_option_label: &str,
        windows_version: u32,
        requires_system: bool,
    ) -> Self {
        Self {
            tweak_id: tweak_id.to_string(),
            tweak_name: tweak_name.to_string(),
            applied_option_index,
            applied_option_label: applied_option_label.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            windows_version,
            requires_system,
            registry_snapshots: Vec::new(),
            service_snapshots: Vec::new(),
        }
    }

    /// Add a registry snapshot
    pub fn add_registry_snapshot(&mut self, snapshot: RegistrySnapshot) {
        self.registry_snapshots.push(snapshot);
    }

    /// Add a service snapshot
    pub fn add_service_snapshot(&mut self, snapshot: ServiceSnapshot) {
        self.service_snapshots.push(snapshot);
    }
}
