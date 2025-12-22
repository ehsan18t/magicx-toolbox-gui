# 🔄 Comprehensive Backup/Restore System Redesign Plan

## Executive Summary

The current "backup" system is actually a **per-tweak snapshot system** designed for atomic rollback of individual tweaks. It is **not** a user-facing import/export system for migrating configurations between machines or Windows versions. This plan addresses the fundamental architectural gaps and proposes a complete redesign.

---

## 1. Current State Analysis

### 1.1 What the Current System Does

| Component               | Implementation                                 | Location                    |
| ----------------------- | ---------------------------------------------- | --------------------------- |
| **Per-Tweak Snapshots** | JSON files storing pre-apply state             | `snapshots/{tweak_id}.json` |
| **Atomic Rollback**     | Restore exact registry/service/scheduler state | restore.rs                  |
| **State Detection**     | Compare current state against tweak options    | detection.rs                |
| **Stale Cleanup**       | Remove snapshots if external changes detected  | `validate_all_snapshots()`  |
| **UI "Snapshots" View** | List tweaks with `has_backup=true`             | SnapshotsView.svelte        |

### 1.2 Critical Gaps Identified

| Gap                              | Impact                                     | Example                                                                  |
| -------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------ |
| **No Cross-Machine Export**      | Cannot migrate configuration to new PC     | User reinstalls Windows, loses all tweaks                                |
| **No Windows Version Awareness** | Win10→Win11 migration breaks               | `TaskbarAl` registry key doesn't exist on Win10                          |
| **No Schema Versioning**         | Tweak IDs/options change = data corruption | Renaming `disable_telemetry` to `telemetry_control` breaks old snapshots |
| **No Selective Restore**         | All-or-nothing                             | User wants only privacy tweaks from backup                               |
| **No Portable Archive**          | Individual JSON files in app folder        | Cannot email backup to friend                                            |
| **No Feedback/Progress**         | Silent success/failure                     | Restore of 50 tweaks shows no progress                                   |
| **No Validation Before Restore** | Blindly applies changes                    | Tries to enable a service that doesn't exist on target machine           |
| **No Conflict Detection**        | Overwrites without warning                 | Target machine already has different customization                       |

### 1.3 Current Data Structures

```rust
// TweakSnapshot (per-tweak, internal)
pub struct TweakSnapshot {
    pub tweak_id: String,
    pub tweak_name: String,
    pub applied_option_index: usize,
    pub applied_option_label: String,
    pub created_at: String,
    pub windows_version: u32,
    pub requires_system: bool,
    pub original_option_index: Option<usize>,
    pub registry_snapshots: Vec<RegistrySnapshot>,
    pub service_snapshots: Vec<ServiceSnapshot>,
    pub scheduler_snapshots: Vec<SchedulerSnapshot>,
}
```

**Problem**: This captures the *original state before applying*, not the *desired configuration*. If you export this and import on another machine, you're importing "what it used to be" not "what I want it to be".

---

## 2. Proposed Architecture

### 2.1 Conceptual Separation

| Concept                   | Purpose                              | Storage                   | User-Facing? |
| ------------------------- | ------------------------------------ | ------------------------- | ------------ |
| **Internal Snapshots**    | Atomic rollback for single tweak     | `snapshots/{id}.json`     | No (hidden)  |
| **Configuration Profile** | User's desired tweak selections      | `profiles/{name}.mgx`     | Yes          |
| **System Backup**         | Full system state before any changes | `backups/{timestamp}.mgx` | Yes          |

### 2.2 New Data Model

```rust
// ============================================================================
// CONFIGURATION PROFILE (User's Intent - What they WANT)
// ============================================================================

/// Schema version for forward/backward compatibility
pub const PROFILE_SCHEMA_VERSION: u32 = 1;

/// A user-exportable configuration profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProfile {
    /// Schema version for migration support
    pub schema_version: u32,

    /// Profile metadata
    pub metadata: ProfileMetadata,

    /// Tweak selections (tweak_id → selected option index)
    /// Only stores tweaks that differ from system default
    pub selections: Vec<TweakSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// User-provided profile name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// When profile was created
    pub created_at: String,
    /// When profile was last modified
    pub modified_at: String,
    /// App version that created this profile
    pub app_version: String,
    /// Source Windows version (10 or 11)
    pub source_windows_version: u32,
    /// Source Windows build number
    pub source_windows_build: u32,
    /// Unique machine identifier (optional, for sync features)
    pub source_machine_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakSelection {
    /// Tweak ID (stable identifier)
    pub tweak_id: String,
    /// Selected option index at time of export
    pub selected_option_index: usize,
    /// Option label for human reference (informational only)
    pub selected_option_label: String,
    /// Hash of tweak definition for detecting schema changes
    pub tweak_definition_hash: String,
}

// ============================================================================
// SYSTEM STATE SNAPSHOT (What the System HAS - for validation/preview)
// ============================================================================

/// Complete system state snapshot for validation before restore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStateSnapshot {
    pub schema_version: u32,
    pub metadata: SnapshotMetadata,
    /// Raw registry values relevant to known tweaks
    pub registry_state: Vec<RegistryValueState>,
    /// Service configurations
    pub service_state: Vec<ServiceState>,
    /// Scheduled task states
    pub scheduler_state: Vec<SchedulerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub created_at: String,
    pub app_version: String,
    pub windows_version: u32,
    pub windows_build: u32,
    pub machine_name: String,
}

// ============================================================================
// IMPORT VALIDATION & PREVIEW
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

#[derive(Debug, Clone, Serialize)]
pub struct ValidationWarning {
    pub tweak_id: String,
    pub code: WarningCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum WarningCode {
    /// Tweak schema has changed since export
    TweakSchemaChanged,
    /// Windows version mismatch (may still work)
    WindowsVersionMismatch,
    /// Registry key doesn't exist but can be created
    RegistryKeyMissing,
    /// Tweak is already at desired state
    AlreadyApplied,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub tweak_id: String,
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum ErrorCode {
    /// Tweak ID no longer exists in app
    TweakNotFound,
    /// Option index out of bounds
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
}

#[derive(Debug, Clone, Serialize)]
pub struct TweakChangePreview {
    pub tweak_id: String,
    pub tweak_name: String,
    pub category_id: String,
    /// Current option on target system (None if unknown)
    pub current_option_index: Option<usize>,
    pub current_option_label: Option<String>,
    /// Desired option from profile
    pub target_option_index: usize,
    pub target_option_label: String,
    /// Whether this tweak can be applied
    pub applicable: bool,
    /// Reason if not applicable
    pub skip_reason: Option<String>,
    /// Risk level of this specific change
    pub risk_level: RiskLevel,
    /// Detailed changes that would be made
    pub changes: Vec<ChangeDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeDetail {
    pub change_type: ChangeType,
    pub description: String,
    pub current_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ChangeType {
    Registry,
    Service,
    ScheduledTask,
    Command,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationStats {
    /// Total tweaks in profile
    pub total_tweaks: usize,
    /// Tweaks that can be applied
    pub applicable_tweaks: usize,
    /// Tweaks that will be skipped
    pub skipped_tweaks: usize,
    /// Tweaks already at desired state
    pub already_applied: usize,
    /// Tweaks with warnings
    pub tweaks_with_warnings: usize,
}
```

### 2.3 Archive Format

Use a **single binary archive** (`.mgx` extension) containing:

```
profile.mgx (ZIP archive)
├── manifest.json          # ProfileManifest with version, checksums
├── profile.json           # ConfigurationProfile
├── system_state.json      # Optional: SystemStateSnapshot at export time
└── signatures/            # Optional: integrity signatures
    └── sha256.txt
```

**Why ZIP?**:
- Standard format with wide tooling support
- Built-in compression
- Can add files without breaking backward compatibility
- Rust has excellent `zip` crate support

---

## 3. Feature Specifications

### 3.1 Export Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        EXPORT WIZARD                            │
├─────────────────────────────────────────────────────────────────┤
│ Step 1: Select Tweaks                                           │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ ☑ Select All Applied Tweaks (23)                            │ │
│ │ ☐ Include System Default Selections                         │ │
│ ├─────────────────────────────────────────────────────────────┤ │
│ │ Privacy (5 tweaks)                                          │ │
│ │   ☑ Disable Telemetry            [Disabled]                 │ │
│ │   ☑ Disable Activity History     [Disabled]                 │ │
│ │   ☑ Disable Advertising ID       [Disabled]                 │ │
│ │   ...                                                        │ │
│ │ Performance (8 tweaks)                                       │ │
│ │   ☑ Icon Cache Size              [4 MB]                     │ │
│ │   ☑ Disable SuperFetch           [Disabled]                 │ │
│ │   ...                                                        │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Step 2: Profile Details                                         │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Name: [My Gaming Setup                              ]       │ │
│ │ Description: [Optimized for gaming, privacy-focused ]       │ │
│ │                                                              │ │
│ │ ☑ Include system state snapshot (for conflict detection)    │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                              [Cancel]  [Export to File]         │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Import Flow with Validation

```
┌─────────────────────────────────────────────────────────────────┐
│                        IMPORT WIZARD                            │
├─────────────────────────────────────────────────────────────────┤
│ Step 1: Load Profile                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 📁 My Gaming Setup.mgx                                      │ │
│ │ Created: Dec 20, 2025 • Windows 11 (22631) • v3.0.0         │ │
│ │ 23 tweak selections                                         │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Step 2: Validation Results                                      │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ ✅ 20 tweaks ready to apply                                 │ │
│ │ ⚠️  2 tweaks with warnings                                  │ │
│ │ ❌ 1 tweak cannot be applied                                │ │
│ │                                                             │ │
│ │ ──────────────────────────────────────────────────────────  │ │
│ │ ⚠️ Disable Cortana                                          │ │
│ │    Windows version mismatch (Win11→Win10)                   │ │
│ │    Registry key may not exist, will be created              │ │
│ │    [Apply Anyway] [Skip]                                    │ │
│ │                                                             │ │
│ │ ❌ Disable Windows Search Highlights                       │ │
│ │    This tweak requires Windows 11                           │ │
│ │    [Skip - Cannot Apply]                                    │ │
│ │                                                             │ │
│ │ ──────────────────────────────────────────────────────────  │ │
│ │ ℹ️ 5 tweaks are already at desired state                   │ │
│ │    [Include] [Skip Already Applied]                         │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Step 3: Review Changes                                          │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ This will make the following changes:                       │ │
│ │                                                              │ │
│ │ Registry: 47 values                                          │ │
│ │ Services: 8 changes                                          │ │
│ │ Scheduled Tasks: 12 changes                                  │ │
│ │                                                              │ │
│ │ ☑ Create system restore point before applying               │ │
│ │ ☐ Apply changes that require reboot immediately             │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     [Cancel]  [Back]  [Apply 20 Tweaks]         │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 Progress Feedback During Apply

```
┌─────────────────────────────────────────────────────────────────┐
│                     APPLYING PROFILE                            │
├─────────────────────────────────────────────────────────────────┤
│ Applying "My Gaming Setup"                                      │
│                                                                 │
│ ████████████████████░░░░░░░░░░░░  12/20 tweaks                 │
│                                                                 │
│ ✅ Disable Telemetry                                            │
│ ✅ Disable Activity History                                     │
│ ✅ Disable Advertising ID                                       │
│ ⏳ Icon Cache Size                                              │
│    Setting HKLM\SOFTWARE\Microsoft\Windows\...                 │
│ ⬜ Disable SuperFetch                                           │
│ ⬜ ...                                                          │
│                                                                 │
│ Time elapsed: 00:12                                             │
│ Estimated remaining: 00:08                                      │
├─────────────────────────────────────────────────────────────────┤
│                              [Cancel]                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Schema Evolution Strategy

### 4.1 Tweak ID Stability

**Problem**: If `disable_telemetry` is renamed to `telemetry_control`, old profiles break.

**Solution**: Tweak ID Aliasing System

```rust
// In tweak.rs - add alias support to YAML schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakDefinition {
    pub id: String,
    /// Previous IDs this tweak was known by (for migration)
    #[serde(default)]
    pub aliases: Vec<String>,
    // ... rest of fields
}

// During import, resolve aliases:
fn resolve_tweak_id(profile_tweak_id: &str, tweaks: &[TweakDefinition]) -> Option<&TweakDefinition> {
    tweaks.iter().find(|t| {
        t.id == profile_tweak_id || t.aliases.contains(&profile_tweak_id.to_string())
    })
}
```

### 4.2 Option Index Stability

**Problem**: If options are reordered or added, option index 1 might mean something different.

**Solution**: Content-Based Option Matching

```rust
/// Generate a hash of option's effective changes
fn hash_option(option: &TweakOption) -> String {
    let mut hasher = Sha256::new();

    // Hash registry changes (sorted for stability)
    for change in option.registry_changes.iter().sorted_by_key(|c| (&c.hive, &c.key, &c.value_name)) {
        hasher.update(format!("{}:{}:{}:{:?}",
            change.hive.as_str(), change.key, change.value_name, change.value));
    }

    // Hash service changes
    for change in option.service_changes.iter().sorted_by_key(|c| &c.name) {
        hasher.update(format!("{}:{}", change.name, change.startup.as_str()));
    }

    hex::encode(hasher.finalize())
}

/// Find matching option by content hash, falling back to index
fn resolve_option_index(
    tweak: &TweakDefinition,
    profile_index: usize,
    profile_hash: Option<&str>,
) -> Result<usize, OptionResolutionError> {
    // If we have a hash, try to match by content first
    if let Some(hash) = profile_hash {
        for (idx, option) in tweak.options.iter().enumerate() {
            if hash_option(option) == hash {
                return Ok(idx);
            }
        }
    }

    // Fall back to index if in bounds
    if profile_index < tweak.options.len() {
        Ok(profile_index)
    } else {
        Err(OptionResolutionError::IndexOutOfBounds {
            tweak_id: tweak.id.clone(),
            requested: profile_index,
            available: tweak.options.len(),
        })
    }
}
```

### 4.3 Schema Version Migration

```rust
/// Migrate profile from older schema version to current
fn migrate_profile(profile: &mut ConfigurationProfile) -> Result<Vec<MigrationNote>, MigrationError> {
    let mut notes = Vec::new();

    match profile.schema_version {
        1 => {
            // Current version, no migration needed
        }
        0 => {
            // Hypothetical v0 -> v1 migration
            // e.g., rename fields, update tweak IDs, etc.
            profile.schema_version = 1;
            notes.push(MigrationNote::SchemaUpgraded { from: 0, to: 1 });
        }
        v => {
            return Err(MigrationError::UnsupportedVersion(v));
        }
    }

    Ok(notes)
}
```

---

## 5. Windows Version Compatibility

### 5.1 Version-Aware Validation

```rust
/// Check if a tweak selection is compatible with target Windows version
fn check_windows_compatibility(
    tweak: &TweakDefinition,
    option_index: usize,
    source_version: u32,
    target_version: u32,
) -> CompatibilityResult {
    let option = &tweak.options[option_index];

    // Check each change type
    let mut incompatible_changes = Vec::new();
    let mut warnings = Vec::new();

    for change in &option.registry_changes {
        match &change.windows_versions {
            Some(versions) if !versions.contains(&target_version) => {
                // This change won't apply on target version
                if versions.contains(&source_version) {
                    // It was meant for source version only
                    incompatible_changes.push(IncompatibleChange::Registry {
                        key: format!("{}\\{}", change.hive.as_str(), change.key),
                        reason: format!("Only applies to Windows {}", versions.iter().join(", ")),
                    });
                }
            }
            Some(versions) if !versions.contains(&source_version) => {
                // This change applies to target but didn't apply on source
                warnings.push(CompatibilityWarning::NewChange {
                    description: format!("Registry change for {} only applies on Windows {}",
                        change.key, target_version),
                });
            }
            _ => {} // Universal change or matching versions
        }
    }

    // Check services exist on target
    for service in &option.service_changes {
        if !service_exists_on_version(&service.name, target_version) {
            incompatible_changes.push(IncompatibleChange::Service {
                name: service.name.clone(),
                reason: format!("Service does not exist on Windows {}", target_version),
            });
        }
    }

    CompatibilityResult {
        fully_compatible: incompatible_changes.is_empty(),
        incompatible_changes,
        warnings,
    }
}
```

### 5.2 Version-Specific Tweak Filtering

```rust
/// Filter profile selections to only those applicable on target version
fn filter_for_target_version(
    selections: &[TweakSelection],
    tweaks: &[TweakDefinition],
    target_version: u32,
) -> (Vec<TweakSelection>, Vec<SkippedSelection>) {
    let mut applicable = Vec::new();
    let mut skipped = Vec::new();

    for selection in selections {
        if let Some(tweak) = tweaks.iter().find(|t| t.id == selection.tweak_id) {
            if tweak.applies_to_version(target_version) {
                applicable.push(selection.clone());
            } else {
                skipped.push(SkippedSelection {
                    selection: selection.clone(),
                    reason: SkipReason::WindowsVersionIncompatible {
                        required: tweak.applicable_versions(),
                        current: target_version,
                    },
                });
            }
        } else {
            skipped.push(SkippedSelection {
                selection: selection.clone(),
                reason: SkipReason::TweakNotFound,
            });
        }
    }

    (applicable, skipped)
}
```

---

## 6. Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)

| Task                               | Files                                         | Priority |
| ---------------------------------- | --------------------------------------------- | -------- |
| Define new data models             | `src-tauri/src/models/profile.rs`             | P0       |
| Implement profile serialization    | `src-tauri/src/services/profile/`             | P0       |
| Add ZIP archive handling           | `src-tauri/src/services/archive.rs`           | P0       |
| Schema version migration framework | `src-tauri/src/services/profile/migration.rs` | P0       |
| Add tweak definition hashing       | `src-tauri/src/services/profile/hashing.rs`   | P1       |

### Phase 2: Validation System (Week 2-3)

| Task                                 | Files                                             | Priority |
| ------------------------------------ | ------------------------------------------------- | -------- |
| Profile validation logic             | `src-tauri/src/services/profile/validation.rs`    | P0       |
| Windows version compatibility checks | `src-tauri/src/services/profile/compatibility.rs` | P0       |
| Change preview generation            | `src-tauri/src/services/profile/preview.rs`       | P0       |
| Tweak ID alias resolution            | tweak.rs                                          | P1       |

### Phase 3: Export Feature (Week 3-4)

| Task                         | Files                                             | Priority |
| ---------------------------- | ------------------------------------------------- | -------- |
| Tauri commands for export    | `src-tauri/src/commands/profile.rs`               | P0       |
| Export wizard UI             | `src/lib/components/profile/ExportWizard.svelte`  | P0       |
| Tweak selection component    | `src/lib/components/profile/TweakSelector.svelte` | P0       |
| File save dialog integration | `src/lib/api/profile.ts`                          | P0       |

### Phase 4: Import Feature (Week 4-5)

| Task                       | Files                                                 | Priority |
| -------------------------- | ----------------------------------------------------- | -------- |
| Tauri commands for import  | `src-tauri/src/commands/profile.rs`                   | P0       |
| Import wizard UI           | `src/lib/components/profile/ImportWizard.svelte`      | P0       |
| Validation results display | `src/lib/components/profile/ValidationResults.svelte` | P0       |
| Batch apply with progress  | `src/lib/components/profile/ApplyProgress.svelte`     | P0       |

### Phase 5: Polish & Edge Cases (Week 5-6)

| Task                                        | Files                                              | Priority |
| ------------------------------------------- | -------------------------------------------------- | -------- |
| Progress streaming from backend             | Event system                                       | P0       |
| Error recovery and rollback                 | `src-tauri/src/services/profile/apply.rs`          | P0       |
| Profile management UI (list saved profiles) | `src/lib/components/profile/ProfileManager.svelte` | P1       |
| Quick export/import from Snapshots view     | SnapshotsView.svelte                               | P1       |
| Documentation                               | `docs/PROFILE_SYSTEM.md`                           | P1       |

---

## 7. API Design

### 7.1 Tauri Commands

```rust
// Export commands
#[tauri::command]
async fn export_profile(
    name: String,
    description: Option<String>,
    tweak_ids: Vec<String>,
    include_system_state: bool,
) -> Result<Vec<u8>, Error>;

#[tauri::command]
async fn export_profile_to_file(
    name: String,
    description: Option<String>,
    tweak_ids: Vec<String>,
    include_system_state: bool,
    file_path: String,
) -> Result<(), Error>;

// Import commands
#[tauri::command]
async fn validate_profile(profile_data: Vec<u8>) -> Result<ProfileValidation, Error>;

#[tauri::command]
async fn validate_profile_file(file_path: String) -> Result<ProfileValidation, Error>;

#[tauri::command]
async fn apply_profile(
    profile_data: Vec<u8>,
    skip_tweak_ids: Vec<String>,
    create_restore_point: bool,
) -> Result<ApplyResult, Error>;

// Progress events (emitted during apply)
// "profile:progress" -> ProfileProgressEvent
// "profile:tweak_complete" -> TweakCompleteEvent
// "profile:error" -> ProfileErrorEvent
```

### 7.2 Frontend API

```typescript
// src/lib/api/profile.ts

export interface ProfileValidation {
  is_valid: boolean;
  is_partially_applicable: boolean;
  warnings: ValidationWarning[];
  errors: ValidationError[];
  preview: TweakChangePreview[];
  stats: ValidationStats;
}

export async function exportProfile(options: ExportOptions): Promise<Blob>;
export async function exportProfileToFile(options: ExportOptions, filePath: string): Promise<void>;
export async function validateProfile(data: ArrayBuffer): Promise<ProfileValidation>;
export async function validateProfileFile(filePath: string): Promise<ProfileValidation>;
export async function applyProfile(data: ArrayBuffer, options: ApplyOptions): Promise<ApplyResult>;

// Progress events
export function onProfileProgress(callback: (event: ProfileProgressEvent) => void): UnlistenFn;
```

---

## 8. File Structure Changes

```
src-tauri/src/
├── models/
│   ├── mod.rs
│   ├── tweak.rs           # Add 'aliases' field
│   ├── tweak_snapshot.rs  # Keep for internal snapshots
│   └── profile.rs         # NEW: Profile models
│
├── services/
│   ├── mod.rs
│   ├── backup/            # Keep for internal snapshots
│   └── profile/           # NEW: Profile system
│       ├── mod.rs
│       ├── archive.rs     # ZIP handling
│       ├── export.rs      # Export logic
│       ├── import.rs      # Import logic
│       ├── validation.rs  # Validation engine
│       ├── compatibility.rs # Version checks
│       ├── migration.rs   # Schema migration
│       └── hashing.rs     # Content hashing
│
├── commands/
│   ├── mod.rs
│   ├── backup.rs          # Keep for internal snapshots
│   └── profile.rs         # NEW: Profile commands

src/lib/
├── api/
│   ├── tweaks.ts
│   └── profile.ts         # NEW: Profile API
│
├── components/
│   ├── profile/           # NEW: Profile components
│   │   ├── ExportWizard.svelte
│   │   ├── ImportWizard.svelte
│   │   ├── TweakSelector.svelte
│   │   ├── ValidationResults.svelte
│   │   ├── ApplyProgress.svelte
│   │   └── index.ts
│   └── views/
│       └── SnapshotsView.svelte  # Add export/import buttons
│
├── stores/
│   └── profile.svelte.ts  # NEW: Profile state
│
└── types/
    └── index.ts           # Add Profile types
```

---

## 9. Migration from Current System

### 9.1 Backward Compatibility

- **Internal snapshots**: Keep working exactly as they do now
- **"Snapshots" view**: Rename to "Applied Tweaks" or add "Profiles" as separate view
- **Existing `.json` snapshot files**: No migration needed, they serve different purpose

### 9.2 User Education

Add tooltips/help:
- "**Applied Tweaks**: Individual tweaks you've applied. Each can be reverted to its original state."
- "**Profiles**: Saved configurations you can export and import across machines."

---

## 10. Security Considerations

| Concern                     | Mitigation                                                                                           |
| --------------------------- | ---------------------------------------------------------------------------------------------------- |
| Malicious profile injection | Profiles only contain tweak IDs and option indices; actual changes come from app's tweak definitions |
| Profile tampering           | Add SHA-256 checksum in manifest; warn if mismatch                                                   |
| Privilege escalation        | Profile import respects same permission model as manual apply                                        |
| Data leakage                | System state snapshot is optional; sensitive data not included by default                            |

---

## 11. Testing Strategy

### Unit Tests
- Profile serialization roundtrip
- Schema migration for all version pairs
- Validation logic edge cases
- Option hash stability

### Integration Tests
- Export → Import on same machine
- Export → Import across Windows versions (mocked)
- Partial apply with skip list
- Progress event ordering

### Manual Test Cases
1. Export profile with 20 tweaks → Import on fresh Windows 11
2. Export from Win10 → Validate on Win11 (check warnings)
3. Export → Modify tweak YAML (change options) → Import (test hash resolution)
4. Export → Rename tweak ID → Add alias → Import (test alias resolution)
5. Import during apply → Cancel → Verify partial state

---

## 12. Future Enhancements (Post-MVP)

| Feature               | Description                                     | Priority |
| --------------------- | ----------------------------------------------- | -------- |
| Cloud Sync            | Sync profiles via GitHub Gist / OneDrive        | P2       |
| Profile Library       | Community-shared profiles                       | P3       |
| Diff View             | Compare two profiles                            | P2       |
| Scheduled Apply       | Apply profile on schedule (e.g., "gaming mode") | P3       |
| Profile Templates     | Pre-built profiles (Gaming, Privacy, Minimal)   | P2       |
| Windows Restore Point | Automatic restore point before batch apply      | P1       |

---

This plan provides a complete roadmap for transforming the current per-tweak snapshot system into a robust, user-facing configuration management system with proper validation, version compatibility, and feedback mechanisms.