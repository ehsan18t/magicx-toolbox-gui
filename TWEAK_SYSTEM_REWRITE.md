# Tweak System Rewrite - Technical Specification

> Complete architectural rewrite to unified option-based tweak system

## Executive Summary

Replace the current dual-mode (binary/multi-state) tweak system with a **unified option-based model** where every tweak has an array of options, each containing its own registry changes, service changes, and commands.

---

## Design Decisions

| Decision              | Choice                       | Rationale                                                                |
| --------------------- | ---------------------------- | ------------------------------------------------------------------------ |
| Migration Strategy    | Big-bang                     | Types are deeply intertwined; incremental would create confusion         |
| Default Detection     | Match against options        | No `is_default` flag; detect by comparing current state to all options   |
| Unmatched State       | "System Default" placeholder | If current state doesn't match any option, show non-selectable indicator |
| Revert Behavior       | From snapshot                | Always restore to captured state before tweak was applied                |
| Service-only Tweaks   | Supported                    | `registry_changes: []` is valid within an option                         |
| Option Identification | Array index                  | No `id` field; use stable array order in YAML                            |

---

## New YAML Schema

### Complete Structure

```yaml
category:
  id: string              # Unique category ID (e.g., "privacy")
  name: string            # Display name (e.g., "Privacy")
  description: string     # Category description
  icon: string            # Emoji icon
  order: number           # Sort order in UI

tweaks:
  - id: string                    # Unique tweak ID (e.g., "disable_telemetry")
    name: string                  # Display name
    description: string           # Short description
    info: string?                 # Optional detailed documentation
    risk_level: low | medium | high | critical
    requires_admin: boolean       # Needs administrator privileges
    requires_system: boolean      # Needs TrustedInstaller (SYSTEM) elevation
    requires_reboot: boolean      # Requires restart to take effect
    is_toggle: boolean            # true = switch UI (2 options), false = dropdown UI

    options:                      # Array of available states
      - label: string             # Display label (e.g., "Enabled", "Disabled", "4MB")
        registry_changes:         # Registry modifications for this option
          - hive: HKCU | HKLM
            key: string
            value_name: string
            value_type: REG_DWORD | REG_SZ | REG_BINARY | REG_QWORD | REG_EXPAND_SZ | REG_MULTI_SZ
            value: any            # Target value (number, string, or byte array)
            windows_versions: [10, 11]?  # Optional: filter by Windows version
        service_changes:          # Service modifications for this option
          - name: string          # Service name (e.g., "DiagTrack")
            startup: disabled | manual | automatic | boot | system
            stop_service: boolean?   # Stop service after changing startup
            start_service: boolean?  # Start service after changing startup
        scheduler_changes:        # Scheduled task modifications for this option
          - task_path: string     # Task path (e.g., "\\Microsoft\\Windows\\Application Experience")
            task_name: string     # Task name (e.g., "Microsoft Compatibility Appraiser")
            action: enable | disable | delete
        pre_commands: string[]?   # Shell commands to run BEFORE changes
        pre_powershell: string[]? # PowerShell commands BEFORE changes (after pre_commands)
        post_commands: string[]?  # Shell commands to run AFTER changes
        post_powershell: string[]? # PowerShell commands AFTER changes (after post_commands)
```

### Execution Order

When applying an option, changes are executed in this specific order:
1. `pre_commands` - Shell (cmd.exe) commands
2. `pre_powershell` - PowerShell commands
3. `registry_changes` - Registry modifications
4. `service_changes` - Windows service changes
5. `scheduler_changes` - Task Scheduler changes
6. `post_commands` - Shell (cmd.exe) commands
7. `post_powershell` - PowerShell commands

### Toggle Tweak Example (2 Options)

```yaml
- id: disable_telemetry
  name: "Disable Telemetry"
  description: "Prevents Windows from collecting diagnostic data"
  risk_level: medium
  requires_admin: true
  requires_system: false
  requires_reboot: false
  is_toggle: true

  options:
    - label: "Disabled"           # Option 0 - Telemetry OFF
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection"
          value_name: "AllowTelemetry"
          value_type: REG_DWORD
          value: 0
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\DataCollection"
          value_name: "AllowTelemetry"
          value_type: REG_DWORD
          value: 0
      service_changes:
        - name: "DiagTrack"
          startup: disabled
          stop_service: true

    - label: "Enabled"            # Option 1 - Telemetry ON (Windows default)
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection"
          value_name: "AllowTelemetry"
          value_type: REG_DWORD
          value: 3
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\DataCollection"
          value_name: "AllowTelemetry"
          value_type: REG_DWORD
          value: 3
      service_changes:
        - name: "DiagTrack"
          startup: automatic
          start_service: true
```

### Multi-State Dropdown Example

```yaml
- id: icon_cache_size
  name: "Icon Cache Size"
  description: "Increase icon cache to prevent corruption"
  risk_level: low
  requires_admin: true
  requires_system: false
  requires_reboot: false
  is_toggle: false

  options:
    - label: "500 KB (Default)"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "500"

    - label: "2 MB"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "2048"

    - label: "4 MB"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "4096"

    - label: "8 MB"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "8192"
```

### Service-Only Tweak Example

```yaml
- id: disable_print_spooler
  name: "Disable Print Spooler"
  description: "Disable printing if you don't use a printer"
  risk_level: medium
  requires_admin: true
  requires_system: false
  requires_reboot: false
  is_toggle: true

  options:
    - label: "Disabled"
      registry_changes: []        # No registry changes
      service_changes:
        - name: "Spooler"
          startup: disabled
          stop_service: true

    - label: "Enabled"
      registry_changes: []
      service_changes:
        - name: "Spooler"
          startup: automatic
          start_service: true
```

### Command-Based Tweak Example

```yaml
- id: flush_dns_cache
  name: "Flush DNS Cache"
  description: "Clear DNS resolver cache"
  risk_level: low
  requires_admin: true
  requires_system: false
  requires_reboot: false
  is_toggle: true

  options:
    - label: "Flush Now"
      registry_changes: []
      service_changes: []
      post_commands:
        - "ipconfig /flushdns"

    - label: "Default"            # No-op option
      registry_changes: []
      service_changes: []
```

### Version-Specific Registry Changes

```yaml
- id: taskbar_alignment
  name: "Left-Align Taskbar"
  description: "Move taskbar icons to the left (Windows 11 only)"
  risk_level: low
  requires_admin: false
  requires_system: false
  requires_reboot: false
  is_toggle: true

  options:
    - label: "Left"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
          value_name: "TaskbarAl"
          value_type: REG_DWORD
          value: 0
          windows_versions: [11]  # Only on Windows 11

    - label: "Center"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
          value_name: "TaskbarAl"
          value_type: REG_DWORD
          value: 1
          windows_versions: [11]
```

---

## Rust Models

### File: `src-tauri/src/models/tweak.rs`

```rust
use serde::{Deserialize, Serialize};

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RegistryHive {
    #[serde(rename = "HKCU")]
    Hkcu,
    #[serde(rename = "HKLM")]
    Hklm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStartupType {
    Disabled,
    Manual,
    Automatic,
    Boot,
    System,
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

/// Single registry modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryChange {
    pub hive: RegistryHive,
    pub key: String,
    pub value_name: String,
    pub value_type: RegistryValueType,
    pub value: serde_json::Value,
    /// Optional Windows version filter [10], [11], or [10, 11]
    #[serde(default)]
    pub windows_versions: Option<Vec<u32>>,
}

/// Single service modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChange {
    pub name: String,
    pub startup: ServiceStartupType,
    #[serde(default)]
    pub stop_service: bool,
    #[serde(default)]
    pub start_service: bool,
}

/// A single option within a tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakOption {
    pub label: String,
    #[serde(default)]
    pub registry_changes: Vec<RegistryChange>,
    #[serde(default)]
    pub service_changes: Vec<ServiceChange>,
    #[serde(default)]
    pub pre_commands: Vec<String>,
    #[serde(default)]
    pub post_commands: Vec<String>,
}

/// Complete tweak definition (YAML → Rust)
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
    #[serde(default)]
    pub requires_reboot: bool,
    #[serde(default)]
    pub is_toggle: bool,
    pub options: Vec<TweakOption>,
    /// Populated at load time from category
    #[serde(default)]
    pub category_id: String,
}

/// YAML file structure
#[derive(Debug, Clone, Deserialize)]
pub struct TweakFile {
    pub category: CategoryDefinition,
    pub tweaks: Vec<TweakDefinition>,
}

// ============================================================================
// HELPER IMPLEMENTATIONS
// ============================================================================

impl RegistryChange {
    /// Check if this change applies to the given Windows version
    pub fn applies_to_version(&self, version: u32) -> bool {
        match &self.windows_versions {
            None => true,
            Some(versions) if versions.is_empty() => true,
            Some(versions) => versions.contains(&version),
        }
    }
}

impl TweakDefinition {
    /// Validate tweak has correct number of options for toggle
    pub fn validate(&self) -> Result<(), String> {
        if self.is_toggle && self.options.len() != 2 {
            return Err(format!(
                "Toggle tweak '{}' must have exactly 2 options, found {}",
                self.id,
                self.options.len()
            ));
        }
        if self.options.is_empty() {
            return Err(format!("Tweak '{}' must have at least 1 option", self.id));
        }
        Ok(())
    }

    /// Get all registry changes across all options (for state detection)
    pub fn all_registry_keys(&self) -> Vec<(RegistryHive, String, String)> {
        let mut keys = Vec::new();
        for option in &self.options {
            for change in &option.registry_changes {
                let key = (change.hive.clone(), change.key.clone(), change.value_name.clone());
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }

    /// Get all service names across all options
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
}

impl RegistryHive {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryHive::Hkcu => "HKCU",
            RegistryHive::Hklm => "HKLM",
        }
    }
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

impl ServiceStartupType {
    pub fn to_sc_start_type(&self) -> &'static str {
        match self {
            ServiceStartupType::Disabled => "disabled",
            ServiceStartupType::Manual => "demand",
            ServiceStartupType::Automatic => "auto",
            ServiceStartupType::Boot => "boot",
            ServiceStartupType::System => "system",
        }
    }
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
```

### File: `src-tauri/src/models/tweak_snapshot.rs`

```rust
use serde::{Deserialize, Serialize};
use super::tweak::{RegistryHive, RegistryValueType, ServiceStartupType};

/// Complete snapshot of state before applying a tweak option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakSnapshot {
    /// Tweak ID this snapshot belongs to
    pub tweak_id: String,
    /// Human-readable tweak name
    pub tweak_name: String,
    /// Which option index was applied (for reference)
    pub applied_option_index: usize,
    /// Timestamp when snapshot was created
    pub created_at: String,
    /// Windows version when snapshot was created
    pub windows_version: u32,
    /// Whether SYSTEM elevation was used
    pub used_system_elevation: bool,
    /// Registry values before changes
    pub registry_snapshots: Vec<RegistrySnapshot>,
    /// Service states before changes
    pub service_snapshots: Vec<ServiceSnapshot>,
}

/// Snapshot of a single registry value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub hive: String,
    pub key: String,
    pub value_name: String,
    /// Original value type (if existed)
    pub value_type: Option<String>,
    /// Original value (if existed)
    pub value: Option<serde_json::Value>,
    /// Whether the value existed before
    pub existed: bool,
}

/// Snapshot of a service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSnapshot {
    pub name: String,
    pub startup_type: String,
    pub was_running: bool,
}

impl TweakSnapshot {
    pub fn new(
        tweak_id: String,
        tweak_name: String,
        applied_option_index: usize,
        windows_version: u32,
        used_system_elevation: bool,
    ) -> Self {
        Self {
            tweak_id,
            tweak_name,
            applied_option_index,
            created_at: chrono::Utc::now().to_rfc3339(),
            windows_version,
            used_system_elevation,
            registry_snapshots: Vec::new(),
            service_snapshots: Vec::new(),
        }
    }
}
```

---

## State Detection Logic

### Matching Current State to Options

```rust
/// Result of detecting current tweak state
#[derive(Debug, Clone, Serialize)]
pub struct TweakState {
    /// Index of matching option, or None if no match (System Default)
    pub current_option_index: Option<usize>,
    /// True if a snapshot exists (tweak was applied by this app)
    pub has_snapshot: bool,
    /// The option index from snapshot (if exists)
    pub snapshot_option_index: Option<usize>,
}

/// Detect which option matches the current system state
pub fn detect_tweak_state(
    tweak: &TweakDefinition,
    windows_version: u32,
) -> Result<TweakState, Error> {
    let has_snapshot = snapshot_exists(&tweak.id)?;
    let snapshot_option_index = if has_snapshot {
        load_snapshot(&tweak.id)?.map(|s| s.applied_option_index)
    } else {
        None
    };

    // Try to match current state against each option
    for (index, option) in tweak.options.iter().enumerate() {
        if option_matches_current_state(option, windows_version)? {
            return Ok(TweakState {
                current_option_index: Some(index),
                has_snapshot,
                snapshot_option_index,
            });
        }
    }

    // No option matches - system is in custom/default state
    Ok(TweakState {
        current_option_index: None,
        has_snapshot,
        snapshot_option_index,
    })
}

/// Check if all registry/service changes in an option match current state
fn option_matches_current_state(
    option: &TweakOption,
    windows_version: u32,
) -> Result<bool, Error> {
    // Check all registry values
    for change in &option.registry_changes {
        if !change.applies_to_version(windows_version) {
            continue; // Skip version-filtered changes
        }

        let current_value = read_registry_value(&change.hive, &change.key, &change.value_name)?;

        match current_value {
            None => return Ok(false), // Value doesn't exist, no match
            Some(val) => {
                if !values_equal(&val, &change.value, &change.value_type) {
                    return Ok(false);
                }
            }
        }
    }

    // Check all service states
    for change in &option.service_changes {
        let current_startup = get_service_startup_type(&change.name)?;
        if current_startup != change.startup {
            return Ok(false);
        }
    }

    // All checks passed - this option matches
    Ok(true)
}
```

---

## Apply/Revert Flow

### Apply Tweak Option

```rust
pub fn apply_tweak(
    tweak_id: &str,
    option_index: usize,
) -> Result<TweakResult, Error> {
    let tweak = get_tweak(tweak_id)?;
    let option = tweak.options.get(option_index)
        .ok_or(Error::InvalidOptionIndex)?;

    // 1. Capture current state as snapshot
    let snapshot = capture_snapshot(&tweak, option_index)?;
    save_snapshot(&snapshot)?;

    // 2. Run pre-commands
    for cmd in &option.pre_commands {
        run_shell_command(cmd)?;
    }

    // 3. Apply registry changes
    for change in &option.registry_changes {
        if !change.applies_to_version(get_windows_version()) {
            continue;
        }
        write_registry_value(
            &change.hive,
            &change.key,
            &change.value_name,
            &change.value_type,
            &change.value,
            tweak.requires_system,
        )?;
    }

    // 4. Apply service changes
    for change in &option.service_changes {
        set_service_startup(&change.name, change.startup)?;
        if change.stop_service {
            stop_service(&change.name)?;
        }
        if change.start_service {
            start_service(&change.name)?;
        }
    }

    // 5. Run post-commands
    for cmd in &option.post_commands {
        run_shell_command(cmd)?;
    }

    Ok(TweakResult {
        success: true,
        message: format!("Applied '{}'", option.label),
        requires_reboot: tweak.requires_reboot,
    })
}
```

### Revert Tweak (From Snapshot)

```rust
pub fn revert_tweak(tweak_id: &str) -> Result<TweakResult, Error> {
    let snapshot = load_snapshot(tweak_id)?
        .ok_or(Error::NoSnapshot)?;

    // 1. Restore registry values
    for reg_snap in &snapshot.registry_snapshots {
        if reg_snap.existed {
            // Restore original value
            write_registry_value(
                &reg_snap.hive,
                &reg_snap.key,
                &reg_snap.value_name,
                &reg_snap.value_type.as_ref().unwrap(),
                &reg_snap.value.as_ref().unwrap(),
                snapshot.used_system_elevation,
            )?;
        } else {
            // Delete the value (it didn't exist before)
            delete_registry_value(
                &reg_snap.hive,
                &reg_snap.key,
                &reg_snap.value_name,
            )?;
        }
    }

    // 2. Restore service states
    for svc_snap in &snapshot.service_snapshots {
        let startup = parse_startup_type(&svc_snap.startup_type)?;
        set_service_startup(&svc_snap.name, startup)?;

        if svc_snap.was_running {
            start_service(&svc_snap.name)?;
        } else {
            stop_service(&svc_snap.name)?;
        }
    }

    // 3. Delete snapshot
    delete_snapshot(tweak_id)?;

    Ok(TweakResult {
        success: true,
        message: "Reverted to original state".to_string(),
        requires_reboot: false, // Snapshot tracks this
    })
}
```

---

## Frontend Types

### File: `src/lib/types/index.ts`

```typescript
// ============================================================================
// ENUMS
// ============================================================================

export type RiskLevel = 'low' | 'medium' | 'high' | 'critical';
export type RegistryHive = 'HKCU' | 'HKLM';
export type RegistryValueType = 'REG_DWORD' | 'REG_QWORD' | 'REG_SZ' | 'REG_EXPAND_SZ' | 'REG_MULTI_SZ' | 'REG_BINARY';
export type ServiceStartupType = 'disabled' | 'manual' | 'automatic' | 'boot' | 'system';

// ============================================================================
// CORE TYPES
// ============================================================================

export interface RegistryChange {
  hive: RegistryHive;
  key: string;
  valueName: string;
  valueType: RegistryValueType;
  value: unknown;
  windowsVersions?: number[];
}

export interface ServiceChange {
  name: string;
  startup: ServiceStartupType;
  stopService?: boolean;
  startService?: boolean;
}

export interface TweakOption {
  label: string;
  registryChanges: RegistryChange[];
  serviceChanges: ServiceChange[];
  preCommands: string[];
  postCommands: string[];
}

export interface Tweak {
  id: string;
  name: string;
  description: string;
  info?: string;
  riskLevel: RiskLevel;
  requiresAdmin: boolean;
  requiresSystem: boolean;
  requiresReboot: boolean;
  isToggle: boolean;
  options: TweakOption[];
  categoryId: string;
}

export interface Category {
  id: string;
  name: string;
  description: string;
  icon: string;
  order: number;
}

// ============================================================================
// STATE TYPES
// ============================================================================

export interface TweakState {
  /** Index of matching option, or null if no match (System Default) */
  currentOptionIndex: number | null;
  /** True if a snapshot exists (tweak was applied by this app) */
  hasSnapshot: boolean;
  /** The option index from snapshot (if exists) */
  snapshotOptionIndex?: number;
}

export interface TweakWithState extends Tweak {
  state: TweakState;
}

// ============================================================================
// API TYPES
// ============================================================================

export interface TweakResult {
  success: boolean;
  message: string;
  requiresReboot: boolean;
}

export interface ApplyTweakRequest {
  tweakId: string;
  optionIndex: number;
}
```

---

## Frontend UI Behavior

### Toggle Tweaks (`is_toggle: true`)

```
┌─────────────────────────────────────────────────────────┐
│  🔒 Disable Telemetry                              [●○] │
│  Prevents Windows from collecting diagnostic data       │
│  ─────────────────────────────────────────────────────  │
│  Status: Disabled ✓                                     │
│  [Revert to Original]                                   │
└─────────────────────────────────────────────────────────┘

States:
- Switch OFF (left)  = options[1] (Enabled/Default)
- Switch ON (right)  = options[0] (Disabled/Applied)
- Grayed out         = No match (System Default), click to apply

Convention: For toggles, options[0] is the "tweaked" state, options[1] is "normal"
```

### Dropdown Tweaks (`is_toggle: false`)

```
┌─────────────────────────────────────────────────────────┐
│  📦 Icon Cache Size                          [4 MB  ▼] │
│  Increase icon cache to prevent corruption              │
│  ─────────────────────────────────────────────────────  │
│  ┌─────────────────────────┐                            │
│  │ ○ System Default        │  ← Shown if no match       │
│  │ ─────────────────────── │                            │
│  │ ○ 500 KB (Default)      │                            │
│  │ ○ 2 MB                  │                            │
│  │ ● 4 MB                  │  ← Currently selected      │
│  │ ○ 8 MB                  │                            │
│  └─────────────────────────┘                            │
│  [Revert to Original]                                   │
└─────────────────────────────────────────────────────────┘

- "System Default" is NOT a real option - it's a placeholder
- Selecting any real option triggers apply_tweak(id, index)
- "Revert" button only appears if has_snapshot = true
```

---

## Files to Modify

### Backend (Rust)

| File                                       | Action      | Changes                               |
| ------------------------------------------ | ----------- | ------------------------------------- |
| `src-tauri/src/models/tweak.rs`            | **REWRITE** | New unified schema                    |
| `src-tauri/src/models/tweak_snapshot.rs`   | **REWRITE** | Add `applied_option_index`, simplify  |
| `src-tauri/src/models/mod.rs`              | Update      | Re-export new types                   |
| `src-tauri/build.rs`                       | **REWRITE** | Parse new YAML schema                 |
| `src-tauri/src/services/tweak_loader.rs`   | Update      | Adapt to new types                    |
| `src-tauri/src/services/backup_service.rs` | **REWRITE** | New state detection, capture, restore |
| `src-tauri/src/commands/tweaks.rs`         | **REWRITE** | Unified apply/revert logic            |

### Frontend (TypeScript/Svelte)

| File                                        | Action      | Changes                                |
| ------------------------------------------- | ----------- | -------------------------------------- |
| `src/lib/types/index.ts`                    | **REWRITE** | New interfaces matching Rust           |
| `src/lib/api/tweaks.ts`                     | Update      | New API signatures                     |
| `src/lib/stores/tweaks.ts`                  | Update      | State management for new model         |
| `src/lib/components/TweakCard.svelte`       | **REWRITE** | Toggle vs dropdown based on `isToggle` |
| `src/lib/components/CategorySection.svelte` | Update      | Handle new state structure             |

### YAML Tweaks

| File                                   | Action      | Tweaks Count |
| -------------------------------------- | ----------- | ------------ |
| `src-tauri/tweaks/privacy.yaml`        | **MIGRATE** | ~20 tweaks   |
| `src-tauri/tweaks/performance.yaml`    | **MIGRATE** | ~13 tweaks   |
| `src-tauri/tweaks/services.yaml`       | **MIGRATE** | ~6 tweaks    |
| `src-tauri/tweaks/windows_update.yaml` | **MIGRATE** | ~5 tweaks    |
| `src-tauri/tweaks/security.yaml`       | **MIGRATE** | ~11 tweaks   |
| `src-tauri/tweaks/gaming.yaml`         | **MIGRATE** | ~6 tweaks    |
| `src-tauri/tweaks/ui.yaml`             | **MIGRATE** | ~14 tweaks   |

---

## Implementation Order

### Phase 1: Core Models (Rust)
1. Rewrite `models/tweak.rs` with new schema
2. Rewrite `models/tweak_snapshot.rs`
3. Update `models/mod.rs`

### Phase 2: Build System
4. Rewrite `build.rs` to parse new YAML structure
5. Test compilation with a single test YAML file

### Phase 3: Services
6. Update `services/tweak_loader.rs`
7. Rewrite `services/backup_service.rs` (state detection + snapshot)
8. Test with cargo check

### Phase 4: Commands
9. Rewrite `commands/tweaks.rs` (apply, revert, status)
10. Test backend with Tauri dev

### Phase 5: YAML Migration
11. Migrate all 7 YAML files to new format
12. Validate all tweaks compile

### Phase 6: Frontend
13. Rewrite `types/index.ts`
14. Update `api/tweaks.ts`
15. Update `stores/tweaks.ts`
16. Rewrite `TweakCard.svelte`
17. Update `CategorySection.svelte`

### Phase 7: Testing & Polish
18. End-to-end testing of apply/revert
19. State detection validation
20. UI polish and error handling

---

## Risk Mitigation

1. **Backup existing code** - Create git branch before starting
2. **Test incrementally** - Compile after each phase
3. **Single YAML first** - Test with one category before migrating all
4. **Snapshot compatibility** - Old snapshots will be invalid; add migration or clear on first run

---

## Open Questions

1. **Empty option arrays** - If all registry_changes have `windows_versions` that don't match current OS, the option becomes no-op. Is this acceptable?

2. **Command failures** - If `pre_commands` fail, should we abort? Current design: yes, abort before registry changes.

3. **Partial failures** - If some registry writes succeed but one fails, should we rollback? Current design: yes, use snapshot to rollback.

4. **Dropdown "Apply" trigger** - Does selecting a dropdown option immediately apply, or stage for batch apply? Current design: immediate apply (matches current behavior).

---

*Document created: 2024-12-12*
*Status: Ready for implementation*
