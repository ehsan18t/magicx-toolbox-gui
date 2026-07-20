//! Tweak Snapshot Models
//!
//! Snapshot-based storage for registry/service state before tweak application.
//! Used for atomic rollback to the exact state before any changes were made.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current on-disk snapshot schema version. Additive fields guarded by `#[serde(default)]` don't
/// require a bump; bump only when the meaning of an existing field changes. Snapshots written
/// before versioning existed deserialize as 0.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

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

/// Snapshot of a scheduled task's state before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerSnapshot {
    /// Task path (e.g., "\\Microsoft\\Windows\\Customer Experience Improvement Program")
    pub task_path: String,
    /// Task name (e.g., "Consolidator")
    pub task_name: String,
    /// Task state before modification ("Ready", "Disabled", "NotFound")
    pub original_state: String,
}

/// Snapshot of a hosts file entry before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsSnapshot {
    /// IP address
    pub ip: String,
    /// Domain name
    pub domain: String,
    /// Whether the entry existed before modification
    pub existed: bool,
}

/// Snapshot of a firewall rule before modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallSnapshot {
    /// Rule name
    pub name: String,
    /// Whether the rule existed before modification
    pub existed: bool,
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
    /// On-disk schema version (0 = a snapshot written before versioning existed).
    #[serde(default)]
    pub schema_version: u32,
    /// The machine this snapshot was captured on (MachineGuid), if it was readable. A mismatch at
    /// load time means the snapshot came from a different machine, so `load_snapshot` warns.
    #[serde(default)]
    pub machine_guid: Option<String>,
    /// Set when a revert of this tweak did not fully succeed (ADR-0001). The snapshot is kept so the
    /// user can retry; the snapshot is released only by a fully-verified revert or an explicit
    /// "keep current state" decision (ADR-0002).
    #[serde(default)]
    pub needs_attention: bool,
    /// Human-readable descriptions of the resources a partial revert could not restore.
    #[serde(default)]
    pub unrestorable_resources: Vec<String>,
    /// Whether SYSTEM elevation was used for this tweak
    #[serde(default)]
    pub requires_system: bool,
    /// Which option index matched the original state before any changes.
    /// None means original state was unknown (didn't match any defined option).
    /// Used by frontend to show "Default" segment in segmented switch.
    #[serde(default)]
    pub original_option_index: Option<usize>,
    /// Registry values captured before changes
    pub registry_snapshots: Vec<RegistrySnapshot>,
    /// Service states captured before changes
    pub service_snapshots: Vec<ServiceSnapshot>,
    /// Scheduled task states captured before changes
    #[serde(default)]
    pub scheduler_snapshots: Vec<SchedulerSnapshot>,
    /// Hosts file entries captured before changes
    #[serde(default)]
    pub hosts_snapshots: Vec<HostsSnapshot>,
    /// Firewall rules captured before changes
    #[serde(default)]
    pub firewall_snapshots: Vec<FirewallSnapshot>,
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
        original_option_index: Option<usize>,
    ) -> Self {
        Self {
            tweak_id: tweak_id.to_string(),
            tweak_name: tweak_name.to_string(),
            applied_option_index,
            applied_option_label: applied_option_label.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            windows_version,
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            machine_guid: crate::services::system_info_service::machine_guid(),
            needs_attention: false,
            unrestorable_resources: Vec::new(),
            requires_system,
            original_option_index,
            registry_snapshots: Vec::new(),
            service_snapshots: Vec::new(),
            scheduler_snapshots: Vec::new(),
            hosts_snapshots: Vec::new(),
            firewall_snapshots: Vec::new(),
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

    /// Add a scheduler snapshot
    pub fn add_scheduler_snapshot(&mut self, snapshot: SchedulerSnapshot) {
        self.scheduler_snapshots.push(snapshot);
    }

    /// Add a hosts snapshot
    pub fn add_hosts_snapshot(&mut self, snapshot: HostsSnapshot) {
        self.hosts_snapshots.push(snapshot);
    }

    /// Add a firewall snapshot
    pub fn add_firewall_snapshot(&mut self, snapshot: FirewallSnapshot) {
        self.firewall_snapshots.push(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_snapshot_stamps_the_current_schema_version() {
        let s = TweakSnapshot::new("t", "T", 0, "opt", 11, false, None);
        assert_eq!(s.schema_version, SNAPSHOT_SCHEMA_VERSION);
    }

    #[test]
    fn a_pre_versioning_snapshot_still_deserializes_with_defaults() {
        // Snapshots written before schema_version/machine_guid existed must keep loading. The
        // additive #[serde(default)] fields make the migration a compile-time-safe no-op (finding #18).
        let json = r#"{
            "tweak_id": "t", "tweak_name": "T", "applied_option_index": 0,
            "applied_option_label": "opt", "created_at": "2020-01-01T00:00:00Z",
            "windows_version": 11, "registry_snapshots": [], "service_snapshots": []
        }"#;
        let s: TweakSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(
            s.schema_version, 0,
            "missing version defaults to 0 (pre-versioning)"
        );
        assert_eq!(s.machine_guid, None);
        assert_eq!(s.tweak_id, "t");
    }

    #[test]
    fn schema_version_and_machine_guid_round_trip() {
        let mut s = TweakSnapshot::new("t", "T", 0, "opt", 11, false, None);
        s.machine_guid = Some("ABC-123".to_string());
        let back: TweakSnapshot =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back.schema_version, SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(back.machine_guid.as_deref(), Some("ABC-123"));
    }
}
