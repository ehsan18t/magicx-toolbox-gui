# MagicX Toolbox Backend Architecture

> Comprehensive guide for LLMs to understand the project's backend capabilities, data flow, and design decisions.

## Project Overview

**MagicX Toolbox** is a Windows system tweaking application built with:
- **Backend**: Rust + Tauri 2.0
- **Frontend**: Svelte 5 + TypeScript + Tailwind CSS 4
- **Target**: Windows 10 and Windows 11

The application allows users to apply, revert, and manage Windows registry tweaks and service configurations through a modern GUI.

---

## Core Features

### 1. Registry Tweaks
- **Binary tweaks**: Toggle between ON (`enable_value`) and OFF (`disable_value`) states
- **Multi-state tweaks**: Dropdown selection from multiple options (e.g., icon cache sizes: 500KB, 4MB, 8MB)
- **Windows version filtering**: Tweaks can be filtered to apply only on Windows 10, 11, or both

### 2. Service Control
- **Tweak-level services**: Apply/revert service configuration when applying/reverting a tweak
- **Per-option services**: Multi-state tweaks can have different service configurations per option (new feature)
- **Service operations**: Set startup type (Disabled, Manual, Automatic), stop/start services

### 3. Snapshot-Based Backup System
- **Atomic snapshots**: Before applying any tweak, the current registry state is captured
- **Automatic restoration**: Reverting a tweak restores from snapshot, not from predefined values
- **Stale snapshot cleanup**: On app startup, snapshots are validated; stale ones are removed
- **Failure handling**: If apply/revert fails, snapshots are cleaned up to prevent orphaned state

### 4. Permissions Model
- **Admin detection**: App detects if running as administrator
- **Per-tweak admin requirement**: Each tweak specifies `requires_admin: true/false`
- **HKCU vs HKLM**: HKCU registry keys don't require admin; HKLM keys typically do
- **Service operations**: Always require administrator privileges

### 5. Risk Levels
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

```yaml
tweaks:
  - id: unique_tweak_id
    name: "Human Readable Name"
    description: "What this tweak does"
    risk_level: low | medium | high | critical
    requires_admin: true | false
    requires_reboot: true | false
    info: "Optional detailed explanation"

    # Registry changes (required)
    registry_changes:
      - hive: HKCU | HKLM
        key: "Registry\\Path"
        value_name: "ValueName"
        value_type: REG_DWORD | REG_SZ | REG_BINARY | ...
        enable_value: <value when ON>
        disable_value: <value when OFF>
        windows_versions: [10, 11]  # Optional filter

        # Multi-state options (optional)
        options:
          - label: "Option Display Name"
            value: <registry value>
            is_default: true | false
            service_changes:  # Per-option services (optional)
              - name: "ServiceName"
                startup: disabled | manual | automatic
                stop_if_disabled: true | false

    # Tweak-level service changes (optional)
    service_changes:
      - name: "ServiceName"
        enable_startup: disabled | manual | automatic
        disable_startup: disabled | manual | automatic
        stop_on_disable: true | false
        start_on_enable: true | false
```

### Snapshot Structure

```rust
struct TweakSnapshot {
    tweak_id: String,
    timestamp: String,
    registry: Vec<RegistrySnapshot>,
    services: Vec<ServiceSnapshot>,
}

struct RegistrySnapshot {
    hive: String,
    key: String,
    value_name: String,
    original_value: Option<serde_json::Value>,
    original_type: Option<String>,
}

struct ServiceSnapshot {
    name: String,
    original_startup: ServiceStartupType,
}
```

## Tweak Format Examples

### 1. Simple Binary Tweak
A standard toggle that changes a registry value between two states.

```yaml
- id: disable_game_dvr
  name: "Disable Game DVR"
  description: "Disables Windows Game Recording and Broadcasting features."
  risk_level: low
  requires_admin: false
  registry_changes:
    - hive: HKCU
      key: "System\\GameConfigStore"
      value_name: "GameDVR_Enabled"
      value_type: REG_DWORD
      enable_value: 0
      disable_value: 1
```

### 2. Multi-State Tweak (Dropdown)
A tweak that offers multiple choices instead of a simple toggle.

```yaml
- id: icon_cache_size
  name: "Icon Cache Size"
  description: "Increase icon cache size to prevent icon corruption."
  risk_level: low
  registry_changes:
    - hive: HKLM
      key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
      value_name: "Max Cached Icons"
      value_type: REG_SZ
      options:
        - label: "Standard (500KB)"
          value: "500"
          is_default: true
        - label: "Medium (2MB)"
          value: "2048"
        - label: "Large (4MB)"
          value: "4096"
```

### 3. Service Management Tweak
A tweak that primarily manages Windows services.

```yaml
- id: disable_print_spooler
  name: "Disable Print Spooler"
  description: "Disables printing services. Useful if you don't use a printer."
  risk_level: medium
  requires_admin: true
  service_changes:
    - name: "Spooler"
      enable_startup: disabled
      disable_startup: automatic
      stop_on_disable: true
      start_on_enable: true
  registry_changes: [] # Can be empty if only services are modified
```

### 4. Hybrid Tweak (Registry + Services)
Combines registry changes with service management.

```yaml
- id: disable_diag_track
  name: "Disable Telemetry"
  description: "Disables Windows telemetry and data collection."
  risk_level: medium
  requires_admin: true
  registry_changes:
    - hive: HKLM
      key: "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection"
      value_name: "AllowTelemetry"
      value_type: REG_DWORD
      enable_value: 0
      disable_value: 1
  service_changes:
    - name: "DiagTrack"
      enable_startup: disabled
      disable_startup: automatic
      stop_on_disable: true
```

### 5. Version-Specific Tweak
Applies different changes based on Windows version.

```yaml
- id: taskbar_alignment
  name: "Taskbar Alignment"
  description: "Align taskbar icons to the left (Windows 11 only)."
  risk_level: low
  registry_changes:
    - hive: HKCU
      key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
      value_name: "TaskbarAl"
      value_type: REG_DWORD
      enable_value: 0
      disable_value: 1
      windows_versions: [11] # Only applies to Windows 11
```

### 6. System-Protected Tweak (TrustedInstaller)
Requires `requires_system: true` to modify protected keys.

```yaml
- id: disable_defender
  name: "Disable Windows Defender"
  description: "Completely disables Windows Defender Antivirus."
  risk_level: critical
  requires_admin: true
  requires_system: true # Uses TrustedInstaller impersonation
  requires_reboot: true
  registry_changes:
    - hive: HKLM
      key: "SOFTWARE\\Policies\\Microsoft\\Windows Defender"
      value_name: "DisableAntiSpyware"
      value_type: REG_DWORD
      enable_value: 1
      disable_value: 0
```

### 7. Command-Based Tweak
Runs shell commands before or after applying changes.

```yaml
- id: remove_onedrive
  name: "Remove OneDrive"
  description: "Uninstalls OneDrive and removes integration."
  risk_level: high
  pre_commands:
    - "taskkill /f /im OneDrive.exe"
  post_commands:
    - "%SystemRoot%\\SysWOW64\\OneDriveSetup.exe /uninstall"
  registry_changes:
    - hive: HKCU
      key: "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
      value_name: "OneDrive"
      value_type: REG_SZ
      enable_value: "" # Remove value
      disable_value: "\"C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe\" /background"
```

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

### 4. `backup_service` - Snapshot Management
- `capture_snapshot()` - Capture current state before changes
- `save_snapshot()` / `load_snapshot()` - Persist to JSON files
- `restore_from_snapshot()` - Restore original state
- `validate_all_snapshots()` - Startup cleanup of stale snapshots
- Storage: `%APPDATA%/com.magicx.toolbox/snapshots/*.json`

### 5. `system_info_service` - System Detection
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

| Path                                      | Purpose                                     |
| ----------------------------------------- | ------------------------------------------- |
| `src-tauri/tweaks/*.yaml`                 | Tweak definitions (7 categories, 76 tweaks) |
| `src-tauri/src/commands/`                 | Tauri command handlers                      |
| `src-tauri/src/services/`                 | Business logic services                     |
| `src-tauri/src/models/`                   | Data structures                             |
| `%APPDATA%/com.magicx.toolbox/snapshots/` | Snapshot JSON files                         |

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
