//! Tweak Snapshot Models
//!
//! Simple snapshot-based storage for registry/service state before tweak application.
//! Used for atomic rollback if any operation fails.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Snapshot of a single registry value before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub hive: String,
    pub key: String,
    pub value_name: String,
    pub value_type: String,
    /// The value before modification. None = key didn't exist
    pub value: Option<Value>,
    /// Whether the key existed before modification
    pub existed: bool,
}

/// Snapshot of a service's state before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSnapshot {
    pub name: String,
    pub startup_type: String,
    pub was_running: bool,
}

/// Complete snapshot of a tweak's state before apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakSnapshot {
    pub tweak_id: String,
    pub tweak_name: String,
    pub applied_at: String,
    pub windows_version: u32,
    /// Whether this tweak requires SYSTEM elevation for restore
    #[serde(default)]
    pub requires_system: bool,
    pub registry_snapshots: Vec<RegistrySnapshot>,
    pub service_snapshots: Vec<ServiceSnapshot>,
}

impl TweakSnapshot {
    /// Create a new empty snapshot for a tweak
    pub fn new(
        tweak_id: &str,
        tweak_name: &str,
        windows_version: u32,
        requires_system: bool,
    ) -> Self {
        Self {
            tweak_id: tweak_id.to_string(),
            tweak_name: tweak_name.to_string(),
            applied_at: chrono::Local::now().to_rfc3339(),
            windows_version,
            requires_system,
            registry_snapshots: Vec::new(),
            service_snapshots: Vec::new(),
        }
    }
}
