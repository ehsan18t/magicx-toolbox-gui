# MagicX Toolbox Architecture

> Comprehensive guide for understanding the project's capabilities, data flow, and design decisions.

## Project Overview

**MagicX Toolbox** is a Windows system tweaking application built with:
- **Backend**: Rust + Tauri 2.0
- **Frontend**: Svelte 5 + TypeScript + Tailwind CSS 4
- **Target**: Windows 10 and Windows 11

The application allows users to apply, revert, and manage Windows registry tweaks and service configurations through a modern GUI.

---

## Frontend Architecture

### Store Pattern (Svelte 5 Runes)

Stores use Svelte 5 runes (`.svelte.ts` files) with getter-based reactive access:

```typescript
// Store definition pattern
let state = $state<T>(initialValue);

export const store = {
  get value() { return state; },
  get derived() { return computedValue; },
  action() { state = newValue; }
};

// Component usage - direct access, no $ prefix
import { store } from "$lib/stores/store.svelte";

const derived = $derived(store.value);
```

**Available stores:**
- `themeStore` - Theme management (light/dark/system)
- `modalStore` - Modal state (about/settings/update)
- `sidebarState` - Sidebar expanded/pinned state
- `colorSchemeStore` - Accent color scheme selection
- `settingsStore` - App settings with localStorage persistence
- `tweakDetailsModalStore` - Tweak details modal state
- `debugState` - Debug panel and logging state

### UI Components

Reusable UI primitives in `$lib/components/ui/`:
- `Button` - Primary, secondary, danger, ghost variants
- `Badge` - Status indicators
- `Card` - Content containers
- `Modal`, `ModalHeader`, `ModalBody`, `ModalFooter` - Dialog system
- `IconButton` - Icon-only buttons with tooltips
- `Switch` - Boolean toggles
- `Select` - Dropdown selection
- `SearchInput` - Search with icon
- `Spinner` - Loading indicator

### Component Structure

```
src/lib/components/
├── ui/                   # Reusable primitives
│   ├── Button.svelte
│   ├── Badge.svelte
│   ├── Card.svelte
│   ├── Modal.svelte, ModalHeader.svelte, ModalBody.svelte, ModalFooter.svelte
│   ├── IconButton.svelte
│   ├── Switch.svelte
│   ├── Select.svelte
│   ├── SearchInput.svelte
│   ├── Spinner.svelte
│   └── index.ts          # Barrel exports
├── tweak-details/        # Tweak detail sub-components
│   ├── RegistryChangeItem.svelte
│   ├── ServiceChangeItem.svelte
│   ├── SchedulerChangeItem.svelte
│   ├── CommandList.svelte
│   └── index.ts
├── AboutModal.svelte     # App info modal
├── SettingsModal.svelte  # App settings
├── UpdateModal.svelte    # Update management
├── TweakCard.svelte      # Individual tweak display
├── TweakDetailsModal.svelte # Tweak details view
├── Sidebar.svelte        # Navigation sidebar
├── TitleBar.svelte       # Custom window titlebar
└── ...
```

---

## Backend Architecture

### Core Features

#### 1. Registry Tweaks
- **Binary tweaks**: Toggle between ON (`enable_value`) and OFF (`disable_value`) states
- **Multi-state tweaks**: Dropdown selection from multiple options (e.g., icon cache sizes: 500KB, 4MB, 8MB)
- **Windows version filtering**: Tweaks can be filtered to apply only on Windows 10, 11, or both

#### 2. Service Control
- **Tweak-level services**: Apply/revert service configuration when applying/reverting a tweak
- **Per-option services**: Multi-state tweaks can have different service configurations per option
- **Service operations**: Set startup type (Disabled, Manual, Automatic), stop/start services

#### 3. Snapshot-Based Backup System
- **Atomic snapshots**: Before applying any tweak, the current registry state is captured
- **Automatic restoration**: Reverting a tweak restores from snapshot, not from predefined values
- **Stale snapshot cleanup**: On app startup, snapshots are validated; stale ones are removed
- **Failure handling**: If apply/revert fails, snapshots are cleaned up to prevent orphaned state

#### 4. Permissions Model
- **Admin detection**: App detects if running as administrator
- **Per-tweak admin requirement**: Each tweak specifies `requires_admin: true/false`
- **HKCU vs HKLM**: HKCU registry keys don't require admin; HKLM keys typically do
- **Service operations**: Always require administrator privileges

#### 5. Risk Levels
```yaml
risk_levels:
  low: Safe, no system impact
  medium: May affect system behavior
  high: Significant impact, changes important features
  critical: Can break system functionality
```

---

## Data Model

### Tweak Definition (YAML → Rust → TypeScript)

All tweaks use a **unified option-based model** where each tweak has an `options` array:

```yaml
tweaks:
  - id: unique_tweak_id
    name: "Human Readable Name"
    description: "What this tweak does"
    risk_level: low | medium | high | critical
    requires_admin: true | false
    requires_system: true | false  # Requires SYSTEM elevation
    requires_reboot: true | false
    is_toggle: true | false  # true = switch UI (2 options), false = dropdown
    info: "Optional detailed explanation"

    options:  # Array of available states (2 for toggle, 2+ for dropdown)
      - label: "Option Display Name"
        registry_changes:
          - hive: HKCU | HKLM
            key: "Registry\\Path"
            value_name: "ValueName"
            value_type: REG_DWORD | REG_SZ | REG_BINARY | REG_QWORD | ...
            value: <target value>
            windows_versions: [10, 11]  # Optional filter
        service_changes:
          - name: "ServiceName"
            startup: disabled | manual | automatic | boot | system
            stop_service: true | false
            start_service: true | false
        scheduler_changes:
          - task_path: "\\Microsoft\\Windows\\..."
            task_name: "TaskName"
            action: enable | disable | delete
        pre_commands: []      # Shell commands before changes
        pre_powershell: []    # PowerShell before changes
        post_commands: []     # Shell commands after changes
        post_powershell: []   # PowerShell after changes
```

### Execution Order

When applying an option, changes execute in this order:
1. `pre_commands` → 2. `pre_powershell` → 3. `registry_changes` → 4. `service_changes` → 5. `scheduler_changes` → 6. `post_commands` → 7. `post_powershell`

### Snapshot Structure

```rust
struct TweakSnapshot {
    tweak_id: String,
    tweak_name: String,
    applied_option_index: usize,
    applied_option_label: String,
    created_at: String,
    windows_version: u32,
    requires_system: bool,
    registry_snapshots: Vec<RegistrySnapshot>,
    service_snapshots: Vec<ServiceSnapshot>,
    scheduler_snapshots: Vec<SchedulerSnapshot>,
}

struct RegistrySnapshot {
    hive: String,
    key: String,
    value_name: String,
    value_type: Option<String>,
    value: Option<serde_json::Value>,
    existed: bool,
}

struct ServiceSnapshot {
    name: String,
    startup_type: String,
    was_running: bool,
}

struct SchedulerSnapshot {
    task_path: String,
    task_name: String,
    original_state: String,  // "Ready", "Disabled", "NotFound"
}
```

## Tweak Format Examples

All tweaks use the **unified option-based model** where each tweak has an `options` array.

### 1. Simple Toggle Tweak
A standard toggle (`is_toggle: true`) with two options (enabled/disabled).

```yaml
- id: disable_game_dvr
  name: "Disable Game DVR"
  description: "Disables Windows Game Recording and Broadcasting features."
  risk_level: low
  requires_admin: false
  requires_system: false
  requires_reboot: false
  is_toggle: true
  options:
    - label: "Disabled"
      registry_changes:
        - hive: HKCU
          key: "System\\GameConfigStore"
          value_name: "GameDVR_Enabled"
          value_type: REG_DWORD
          value: 0
    - label: "Enabled"
      registry_changes:
        - hive: HKCU
          key: "System\\GameConfigStore"
          value_name: "GameDVR_Enabled"
          value_type: REG_DWORD
          value: 1
```

### 2. Multi-State Tweak (Dropdown)
A tweak with multiple choices (`is_toggle: false`).

```yaml
- id: icon_cache_size
  name: "Icon Cache Size"
  description: "Increase icon cache size to prevent icon corruption."
  risk_level: low
  is_toggle: false
  options:
    - label: "Standard (500KB)"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "500"
    - label: "Medium (2MB)"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "2048"
    - label: "Large (4MB)"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: REG_SZ
          value: "4096"
```

### 3. Service Management Tweak
A tweak that manages Windows services.

```yaml
- id: disable_print_spooler
  name: "Disable Print Spooler"
  description: "Disables printing services. Useful if you don't use a printer."
  risk_level: medium
  requires_admin: true
  is_toggle: true
  options:
    - label: "Disabled"
      service_changes:
        - name: "Spooler"
          startup: disabled
    - label: "Enabled"
      service_changes:
        - name: "Spooler"
          startup: automatic
```

### 4. Hybrid Tweak (Registry + Services + Scheduler)
Combines registry changes with service and scheduler management.

```yaml
- id: disable_diag_track
  name: "Disable Telemetry"
  description: "Disables Windows telemetry and data collection."
  risk_level: medium
  requires_admin: true
  is_toggle: true
  options:
    - label: "Disabled"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection"
          value_name: "AllowTelemetry"
          value_type: REG_DWORD
          value: 0
      service_changes:
        - name: "DiagTrack"
          startup: disabled
      scheduler_changes:
        - task_path: "\\Microsoft\\Windows\\Application Experience"
          task_name: "Microsoft Compatibility Appraiser"
          action: disable
    - label: "Enabled"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection"
          value_name: "AllowTelemetry"
          value_type: REG_DWORD
          value: 3
      service_changes:
        - name: "DiagTrack"
          startup: automatic
      scheduler_changes:
        - task_path: "\\Microsoft\\Windows\\Application Experience"
          task_name: "Microsoft Compatibility Appraiser"
          action: enable
```

### 5. Version-Specific Tweak
Applies different changes based on Windows version.

```yaml
- id: taskbar_alignment
  name: "Taskbar Alignment"
  description: "Align taskbar icons to the left (Windows 11 only)."
  risk_level: low
  is_toggle: true
  options:
    - label: "Left"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
          value_name: "TaskbarAl"
          value_type: REG_DWORD
          value: 0
          windows_versions: [11]
    - label: "Center"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
          value_name: "TaskbarAl"
          value_type: REG_DWORD
          value: 1
          windows_versions: [11]
```

### 6. System-Protected Tweak (TrustedInstaller)
Requires `requires_system: true` to modify protected keys.

```yaml
- id: disable_defender
  name: "Disable Windows Defender"
  description: "Completely disables Windows Defender Antivirus."
  risk_level: critical
  requires_admin: true
  requires_system: true
  requires_reboot: true
  is_toggle: true
  options:
    - label: "Disabled"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Policies\\Microsoft\\Windows Defender"
          value_name: "DisableAntiSpyware"
          value_type: REG_DWORD
          value: 1
    - label: "Enabled"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Policies\\Microsoft\\Windows Defender"
          value_name: "DisableAntiSpyware"
          value_type: REG_DWORD
          value: 0
```

### 7. Command-Based Tweak with PowerShell
Runs shell commands and PowerShell scripts before or after applying changes.

```yaml
- id: remove_onedrive
  name: "Remove OneDrive"
  description: "Uninstalls OneDrive and removes integration."
  risk_level: high
  is_toggle: true
  options:
    - label: "Removed"
      pre_commands:
        - "taskkill /f /im OneDrive.exe"
      pre_powershell:
        - "Stop-Process -Name 'OneDrive' -Force -ErrorAction SilentlyContinue"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
          value_name: "OneDrive"
          value_type: REG_SZ
          value: ""
      post_powershell:
        - "Start-Process '%SystemRoot%\\SysWOW64\\OneDriveSetup.exe' -ArgumentList '/uninstall' -Wait"
    - label: "Installed"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
          value_name: "OneDrive"
          value_type: REG_SZ
          value: "\"C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe\" /background"

```

### Execution Order

When applying an option, changes are executed in this order:
1. `pre_commands` (shell commands)
2. `pre_powershell` (PowerShell scripts)
3. `registry_changes`
4. `service_changes`
5. `scheduler_changes`
6. `post_commands` (shell commands)
7. `post_powershell` (PowerShell scripts)

---

## Backend Services

### 1. `tweak_loader` - Tweak Loading
- Loads tweaks from compiled binary (embedded at build time)
- Build-time YAML parsing via `build.rs`
- Runtime access via `get_tweak()`, `get_all_tweaks()`

### 2. `registry_service` - Registry Operations
- Read/write registry values (DWORD, SZ, BINARY, etc.)
- Delete registry values
- Create registry keys
- Windows API via `winreg` crate

### 3. `service_control` - Windows Service Management
- Get/set service startup type
- Start/stop services
- Query service status
- Uses Windows SC (Service Control Manager) API

### 4. `scheduler_service` - Task Scheduler Management
- Enable/disable/delete scheduled tasks
- Query task state (Ready, Disabled, Running, NotFound)
- Uses Windows `schtasks.exe` CLI

### 5. `backup_service` - Snapshot Management
- `capture_snapshot()` - Capture current state before changes
- `save_snapshot()` / `load_snapshot()` - Persist to JSON files
- `restore_from_snapshot()` - Restore original state
- `validate_all_snapshots()` - Startup cleanup of stale snapshots
- Storage: `snapshots/` directory next to executable (portable app design)

### 6. `trusted_installer` - SYSTEM Elevation & PowerShell
- Execute commands as SYSTEM via winlogon.exe token
- Registry writes as SYSTEM for protected keys
- PowerShell execution: `run_powershell()`, `run_powershell_as_system()`
- Schtasks execution: `run_schtasks_as_system()`

### 7. `system_info_service` - System Detection
- Windows version detection (10 vs 11)
- Build number detection
- Admin privilege check
- CPU/RAM information

---

## Commands (Tauri IPC)

### Tweak Operations
| Command                         | Description                                               |
| ------------------------------- | --------------------------------------------------------- |
| `apply_tweak(id)`               | Apply tweak (toggle ON), or toggle OFF if already applied |
| `revert_tweak(id)`              | Revert to original state using snapshot or disable_value  |
| `apply_tweak_option(id, index)` | Apply specific option for multi-state tweak               |
| `get_tweak_status(id)`          | Check if tweak is currently applied                       |
| `get_all_tweaks_with_status()`  | Get all tweaks with their current statuses                |

### Backup Operations
| Command                 | Description                        |
| ----------------------- | ---------------------------------- |
| `has_snapshot(id)`      | Check if snapshot exists for tweak |
| `cleanup_old_backups()` | Remove orphaned backup files       |
| `validate_snapshots()`  | Validate and clean stale snapshots |

### System Operations
| Command               | Description                                   |
| --------------------- | --------------------------------------------- |
| `get_system_info()`   | Get Windows version, admin status, build info |
| `get_categories()`    | Get all tweak categories                      |
| `toggle_debug_mode()` | Enable/disable debug logging                  |

---

## Error Handling

```rust
enum Error {
    RegistryOperation(String),  // Registry read/write failures
    WindowsApi(String),         // Win32 API errors
    RequiresAdmin,              // Operation needs elevation
    IoOperation(String),        // File I/O errors
    ServiceControl(String),     // Service operation failures
    UnsupportedWindowsVersion,  // Tweak not available for this Windows
}
```

All operations return `Result<T, Error>` propagated to frontend.

---

## Build System

### `build.rs` - Compile-Time Processing
1. Reads YAML files from `tweaks/` directory
2. Parses category and tweak definitions
3. Embeds as static Rust code in binary
4. No runtime file I/O for tweak definitions

### Generated Output
- Categories and tweaks embedded as static `&[u8]` JSON
- Loaded once at runtime via `tweak_loader`

---

## File Locations

| Path                                      | Purpose                                               |
| ----------------------------------------- | ----------------------------------------------------- |
| `src-tauri/tweaks/*.yaml`                 | Tweak definitions (7 categories, 76 tweaks)           |
| `src-tauri/src/commands/`                 | Tauri command handlers                                |
| `src-tauri/src/commands/tweaks/`          | Tweak commands (split into query/apply/batch/helpers) |
| `src-tauri/src/services/`                 | Business logic services                               |
| `src-tauri/src/models/`                   | Data structures                                       |
| `%APPDATA%/com.magicx.toolbox/snapshots/` | Snapshot JSON files                                   |

### Backend Commands Module Structure

```
src-tauri/src/commands/tweaks/
├── mod.rs      # Module exports
├── query.rs    # Status and listing commands (get_*, get_tweak_status)
├── apply.rs    # Single tweak operations (apply_tweak, revert_tweak)
├── batch.rs    # Batch operations (batch_apply_tweaks, batch_revert_tweaks)
└── helpers.rs  # Internal utilities (registry/service/scheduler operations)
```

---

## Security Considerations

1. **Elevation**: App detects admin status; operations fail gracefully if lacking permissions
2. **Snapshot validation**: Prevents applying to non-existent backups
3. **Rollback on failure**: Registry changes are rolled back if service operations fail
4. **No remote code**: All tweaks are compiled into the binary; no external downloads

---

## Known Limitations

1. **Service control requires admin**: Cannot modify service startup type without elevation
2. **Some registry keys protected**: Even with admin, some keys (SAM) may be inaccessible
3. **Windows version detection**: Based on registry, may not detect all insider builds

---

## Categories

| ID             | Name           | Tweaks | Description                    |
| -------------- | -------------- | ------ | ------------------------------ |
| privacy        | Privacy        | 20     | Disable telemetry and tracking |
| performance    | Performance    | 13     | Optimize system speed          |
| ui             | UI/UX          | 14     | Customize appearance           |
| security       | Security       | 11     | Improve security settings      |
| services       | Services       | 6      | Manage Windows services        |
| gaming         | Gaming         | 6      | Gaming optimizations           |
| windows_update | Windows Update | 5      | Control update behavior        |
