use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a registry key (hive + key path + value name)
pub type RegistryKeyId = String;

/// Creates a unique key identifier for a registry value
pub fn make_key_id(hive: &str, key: &str, value_name: &str) -> RegistryKeyId {
    format!("{}\\{}\\{}", hive, key, value_name)
}

// ============================================================================
// Atomic Operation Types
// ============================================================================

/// Snapshot of a registry value before modification (for rollback)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSnapshot {
    /// Registry hive
    pub hive: String,
    /// Registry key path
    pub key: String,
    /// Value name
    pub value_name: String,
    /// Value type
    pub value_type: String,
    /// Value before modification (None = didn't exist)
    pub value_before: Option<serde_json::Value>,
    /// Whether the key existed before
    pub key_existed: bool,
}

/// Result of verifying a single registry change
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// Key identifier
    pub key_id: RegistryKeyId,
    /// Whether the value matches expected
    pub matches: bool,
    /// Expected value
    pub expected: Option<serde_json::Value>,
    /// Actual value found
    pub actual: Option<serde_json::Value>,
}

/// Report from a rollback operation
#[derive(Debug, Clone, Default)]
pub struct RollbackReport {
    /// Number of keys successfully rolled back
    pub succeeded: usize,
    /// Number of keys that failed to rollback
    pub failed: usize,
    /// Details of failures
    pub failures: Vec<(RegistryKeyId, String)>,
    /// Whether all rollbacks succeeded
    pub all_succeeded: bool,
}

/// Parses a key ID back into components
pub fn parse_key_id(key_id: &RegistryKeyId) -> Option<(String, String, String)> {
    let parts: Vec<&str> = key_id.splitn(3, '\\').collect();
    if parts.len() >= 3 {
        // The key path may contain backslashes, so we need to handle that
        let hive = parts[0].to_string();
        // Find the second backslash to split key and value_name
        if let Some(first_sep) = key_id.find('\\') {
            let rest = &key_id[first_sep + 1..];
            if let Some(last_sep) = rest.rfind('\\') {
                let key = rest[..last_sep].to_string();
                let value_name = rest[last_sep + 1..].to_string();
                return Some((hive, key, value_name));
            }
        }
    }
    None
}

/// A single entry in the registry baseline - the "golden" original value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    /// Registry hive (HKCU or HKLM)
    pub hive: String,
    /// Registry key path
    pub key: String,
    /// Value name within the key
    pub value_name: String,
    /// Original value type (REG_DWORD, REG_SZ, etc.)
    pub value_type: String,
    /// Original value data (None = value didn't exist)
    pub original_value: Option<serde_json::Value>,
    /// Whether the key itself existed
    pub key_existed: bool,
    /// ISO 8601 timestamp when this baseline was captured
    pub captured_at: String,
    /// Which tweak first triggered this baseline capture
    pub captured_by_tweak: String,
}

/// The global registry baseline - stores first-seen (original) values
/// This file is NEVER overwritten once a key is recorded
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryBaseline {
    /// Version for migration purposes
    pub version: u32,
    /// ISO 8601 timestamp of first creation
    pub created_at: String,
    /// ISO 8601 timestamp of last update
    pub updated_at: String,
    /// Map of key_id -> BaselineEntry
    pub entries: HashMap<RegistryKeyId, BaselineEntry>,
}

impl RegistryBaseline {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new() -> Self {
        let now = chrono::Local::now().to_rfc3339();
        RegistryBaseline {
            version: Self::CURRENT_VERSION,
            created_at: now.clone(),
            updated_at: now,
            entries: HashMap::new(),
        }
    }

    /// Check if a key has a baseline entry
    pub fn has_entry(&self, key_id: &RegistryKeyId) -> bool {
        self.entries.contains_key(key_id)
    }

    /// Get a baseline entry
    pub fn get_entry(&self, key_id: &RegistryKeyId) -> Option<&BaselineEntry> {
        self.entries.get(key_id)
    }

    /// Add a new baseline entry (only if not already present)
    /// Returns true if entry was added, false if it already existed
    pub fn add_entry(&mut self, key_id: RegistryKeyId, entry: BaselineEntry) -> bool {
        use std::collections::hash_map::Entry;

        if let Entry::Vacant(e) = self.entries.entry(key_id) {
            e.insert(entry);
            self.updated_at = chrono::Local::now().to_rfc3339();
            true
        } else {
            false
        }
    }

    /// Remove a baseline entry (only used during recovery/cleanup)
    pub fn remove_entry(&mut self, key_id: &RegistryKeyId) -> Option<BaselineEntry> {
        let result = self.entries.remove(key_id);
        if result.is_some() {
            self.updated_at = chrono::Local::now().to_rfc3339();
        }
        result
    }
}

/// Information about an applied tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedTweakInfo {
    /// Tweak ID
    pub tweak_id: String,
    /// Human-readable tweak name
    pub tweak_name: String,
    /// ISO 8601 timestamp when applied
    pub applied_at: String,
    /// Windows version when applied
    pub windows_version: u32,
    /// List of registry key IDs this tweak modified
    pub modified_keys: Vec<RegistryKeyId>,
}

/// Tracks which tweaks are using a particular registry key
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyOwnership {
    /// List of tweak IDs that have modified this key
    pub tweak_ids: Vec<String>,
}

impl KeyOwnership {
    pub fn new() -> Self {
        KeyOwnership {
            tweak_ids: Vec::new(),
        }
    }

    /// Add a tweak to the ownership list
    pub fn add_tweak(&mut self, tweak_id: &str) {
        if !self.tweak_ids.contains(&tweak_id.to_string()) {
            self.tweak_ids.push(tweak_id.to_string());
        }
    }

    /// Remove a tweak from the ownership list
    /// Returns true if the list is now empty
    pub fn remove_tweak(&mut self, tweak_id: &str) -> bool {
        self.tweak_ids.retain(|id| id != tweak_id);
        self.tweak_ids.is_empty()
    }

    /// Get the reference count (number of tweaks using this key)
    pub fn ref_count(&self) -> usize {
        self.tweak_ids.len()
    }

    /// Check if a tweak owns this key
    pub fn is_owned_by(&self, tweak_id: &str) -> bool {
        self.tweak_ids.contains(&tweak_id.to_string())
    }
}

/// Global state tracking all applied tweaks and key ownership
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TweakState {
    /// Version for migration purposes
    pub version: u32,
    /// ISO 8601 timestamp of last update
    pub updated_at: String,
    /// Map of tweak_id -> AppliedTweakInfo for currently applied tweaks
    pub applied_tweaks: HashMap<String, AppliedTweakInfo>,
    /// Map of key_id -> KeyOwnership for reference counting
    pub key_ownership: HashMap<RegistryKeyId, KeyOwnership>,
}

impl TweakState {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new() -> Self {
        TweakState {
            version: Self::CURRENT_VERSION,
            updated_at: chrono::Local::now().to_rfc3339(),
            applied_tweaks: HashMap::new(),
            key_ownership: HashMap::new(),
        }
    }

    /// Check if a tweak is currently applied
    pub fn is_tweak_applied(&self, tweak_id: &str) -> bool {
        self.applied_tweaks.contains_key(tweak_id)
    }

    /// Get info about an applied tweak
    pub fn get_applied_tweak(&self, tweak_id: &str) -> Option<&AppliedTweakInfo> {
        self.applied_tweaks.get(tweak_id)
    }

    /// Record that a tweak has been applied
    pub fn record_applied(&mut self, info: AppliedTweakInfo) {
        let tweak_id = info.tweak_id.clone();

        // Update key ownership for all modified keys
        for key_id in &info.modified_keys {
            self.key_ownership
                .entry(key_id.clone())
                .or_default()
                .add_tweak(&tweak_id);
        }

        self.applied_tweaks.insert(tweak_id, info);
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// Record that a tweak has been reverted
    /// Returns list of key_ids that are no longer referenced by any tweak
    pub fn record_reverted(&mut self, tweak_id: &str) -> Vec<RegistryKeyId> {
        let mut orphaned_keys = Vec::new();

        if let Some(info) = self.applied_tweaks.remove(tweak_id) {
            // Update key ownership for all modified keys
            for key_id in &info.modified_keys {
                if let Some(ownership) = self.key_ownership.get_mut(key_id) {
                    if ownership.remove_tweak(tweak_id) {
                        // No more tweaks reference this key
                        orphaned_keys.push(key_id.clone());
                    }
                }
            }

            // Clean up empty ownership entries
            for key_id in &orphaned_keys {
                self.key_ownership.remove(key_id);
            }

            self.updated_at = chrono::Local::now().to_rfc3339();
        }

        orphaned_keys
    }

    /// Get keys that would be orphaned if a tweak is reverted (read-only, doesn't modify state)
    pub fn get_orphaned_keys_if_reverted(&self, tweak_id: &str) -> Vec<RegistryKeyId> {
        let mut orphaned_keys = Vec::new();

        if let Some(info) = self.applied_tweaks.get(tweak_id) {
            for key_id in &info.modified_keys {
                if let Some(ownership) = self.key_ownership.get(key_id) {
                    // Key would be orphaned if this tweak is the only one referencing it
                    if ownership.ref_count() == 1
                        && ownership.tweak_ids.contains(&tweak_id.to_string())
                    {
                        orphaned_keys.push(key_id.clone());
                    }
                }
            }
        }

        orphaned_keys
    }

    /// Get the reference count for a key
    pub fn get_key_ref_count(&self, key_id: &RegistryKeyId) -> usize {
        self.key_ownership
            .get(key_id)
            .map(|o| o.ref_count())
            .unwrap_or(0)
    }

    /// Get all tweaks that modify a specific key
    pub fn get_tweaks_for_key(&self, key_id: &RegistryKeyId) -> Vec<String> {
        self.key_ownership
            .get(key_id)
            .map(|o| o.tweak_ids.clone())
            .unwrap_or_default()
    }

    /// Get all applied tweak IDs
    pub fn get_applied_tweak_ids(&self) -> Vec<String> {
        self.applied_tweaks.keys().cloned().collect()
    }
}

/// Conflict information when multiple tweaks affect the same key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConflict {
    /// The registry key that has a conflict
    pub key_id: RegistryKeyId,
    /// Human-readable key path
    pub key_path: String,
    /// Tweaks that modify this key
    pub conflicting_tweaks: Vec<String>,
    /// Current value in registry
    pub current_value: Option<serde_json::Value>,
    /// Baseline (original) value
    pub baseline_value: Option<serde_json::Value>,
}

/// Result of conflict detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    /// Whether there are any conflicts
    pub has_conflicts: bool,
    /// List of key conflicts
    pub conflicts: Vec<KeyConflict>,
    /// Warning message if any
    pub warning: Option<String>,
}

/// Status of a specific registry key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStatus {
    /// Key matches baseline (original value)
    AtBaseline,
    /// Key is modified by tweaks
    Modified {
        by_tweaks: Vec<String>,
        ref_count: usize,
    },
    /// Key was changed externally (not by our tweaks)
    ExternallyModified,
    /// Key status unknown (no baseline)
    Unknown,
}

/// Recovery action to take for a corrupted/inconsistent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Restore key to baseline value
    RestoreBaseline,
    /// Re-read current value as new baseline
    CaptureNewBaseline,
    /// Remove stale tweak state
    RemoveStaleTweakState,
    /// Remove orphaned baseline entry
    RemoveOrphanedBaseline,
}

/// A recovery suggestion for fixing inconsistent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySuggestion {
    /// Description of the issue
    pub issue: String,
    /// Suggested action
    pub action: RecoveryAction,
    /// Affected key ID (if applicable)
    pub key_id: Option<RegistryKeyId>,
    /// Affected tweak ID (if applicable)
    pub tweak_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_key_id() {
        let key_id = make_key_id("HKLM", "SOFTWARE\\Microsoft\\Test", "ValueName");
        assert_eq!(key_id, "HKLM\\SOFTWARE\\Microsoft\\Test\\ValueName");
    }

    #[test]
    fn test_parse_key_id() {
        let key_id = "HKLM\\SOFTWARE\\Microsoft\\Test\\ValueName".to_string();
        let parsed = parse_key_id(&key_id);
        assert!(parsed.is_some());
        let (hive, key, value) = parsed.unwrap();
        assert_eq!(hive, "HKLM");
        assert_eq!(key, "SOFTWARE\\Microsoft\\Test");
        assert_eq!(value, "ValueName");
    }

    #[test]
    fn test_key_ownership_ref_counting() {
        let mut ownership = KeyOwnership::new();
        assert_eq!(ownership.ref_count(), 0);

        ownership.add_tweak("tweak1");
        assert_eq!(ownership.ref_count(), 1);

        ownership.add_tweak("tweak2");
        assert_eq!(ownership.ref_count(), 2);

        // Adding same tweak again shouldn't increase count
        ownership.add_tweak("tweak1");
        assert_eq!(ownership.ref_count(), 2);

        // Remove one tweak
        let empty = ownership.remove_tweak("tweak1");
        assert!(!empty);
        assert_eq!(ownership.ref_count(), 1);

        // Remove last tweak
        let empty = ownership.remove_tweak("tweak2");
        assert!(empty);
        assert_eq!(ownership.ref_count(), 0);
    }

    #[test]
    fn test_baseline_add_entry() {
        let mut baseline = RegistryBaseline::new();
        let entry = BaselineEntry {
            hive: "HKLM".to_string(),
            key: "SOFTWARE\\Test".to_string(),
            value_name: "Value".to_string(),
            value_type: "REG_DWORD".to_string(),
            original_value: Some(serde_json::json!(1)),
            key_existed: true,
            captured_at: chrono::Local::now().to_rfc3339(),
            captured_by_tweak: "test_tweak".to_string(),
        };

        let key_id = make_key_id("HKLM", "SOFTWARE\\Test", "Value");

        // First add should succeed
        assert!(baseline.add_entry(key_id.clone(), entry.clone()));
        assert!(baseline.has_entry(&key_id));

        // Second add should fail (already exists)
        assert!(!baseline.add_entry(key_id.clone(), entry));
    }

    #[test]
    fn test_tweak_state_record_applied() {
        let mut state = TweakState::new();
        let key1 = make_key_id("HKLM", "SOFTWARE\\Test", "Value1");
        let key2 = make_key_id("HKLM", "SOFTWARE\\Test", "Value2");

        let info = AppliedTweakInfo {
            tweak_id: "tweak1".to_string(),
            tweak_name: "Test Tweak".to_string(),
            applied_at: chrono::Local::now().to_rfc3339(),
            windows_version: 11,
            modified_keys: vec![key1.clone(), key2.clone()],
        };

        state.record_applied(info);

        assert!(state.is_tweak_applied("tweak1"));
        assert_eq!(state.get_key_ref_count(&key1), 1);
        assert_eq!(state.get_key_ref_count(&key2), 1);
    }

    #[test]
    fn test_tweak_state_overlapping_keys() {
        let mut state = TweakState::new();
        let shared_key = make_key_id("HKLM", "SOFTWARE\\Test", "SharedValue");

        // Apply first tweak
        let info1 = AppliedTweakInfo {
            tweak_id: "tweak1".to_string(),
            tweak_name: "Tweak 1".to_string(),
            applied_at: chrono::Local::now().to_rfc3339(),
            windows_version: 11,
            modified_keys: vec![shared_key.clone()],
        };
        state.record_applied(info1);
        assert_eq!(state.get_key_ref_count(&shared_key), 1);

        // Apply second tweak that shares the key
        let info2 = AppliedTweakInfo {
            tweak_id: "tweak2".to_string(),
            tweak_name: "Tweak 2".to_string(),
            applied_at: chrono::Local::now().to_rfc3339(),
            windows_version: 11,
            modified_keys: vec![shared_key.clone()],
        };
        state.record_applied(info2);
        assert_eq!(state.get_key_ref_count(&shared_key), 2);

        // Revert first tweak - key should still be referenced
        let orphaned = state.record_reverted("tweak1");
        assert!(orphaned.is_empty()); // Not orphaned yet
        assert_eq!(state.get_key_ref_count(&shared_key), 1);

        // Revert second tweak - key is now orphaned
        let orphaned = state.record_reverted("tweak2");
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0], shared_key);
        assert_eq!(state.get_key_ref_count(&shared_key), 0);
    }
}
