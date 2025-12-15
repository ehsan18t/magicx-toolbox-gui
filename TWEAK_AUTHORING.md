# Tweak Authoring Guide

> **The definitive guide to writing tweaks for MagicX Toolbox**
>
> This document covers everything a tweak author needs to know: YAML schema, execution order, error handling, state detection, and best practices.

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [YAML File Structure](#yaml-file-structure)
4. [Category Definition](#category-definition)
5. [Tweak Definition](#tweak-definition)
6. [Options Array](#options-array)
7. [Change Types](#change-types)
   - [Registry Changes](#registry-changes)
   - [Service Changes](#service-changes)
   - [Scheduler Changes](#scheduler-changes)
   - [Shell Commands](#shell-commands)
   - [PowerShell Commands](#powershell-commands)
8. [Execution Order & Atomicity](#execution-order--atomicity)
9. [Error Handling Behavior](#error-handling-behavior)
10. [The `skip_validation` Flag](#the-skip_validation-flag)
11. [State Detection](#state-detection)
12. [Snapshot & Revert System](#snapshot--revert-system)
13. [Windows Version Filtering](#windows-version-filtering)
14. [Complete Examples](#complete-examples)
15. [Best Practices](#best-practices)
16. [Common Mistakes](#common-mistakes)
17. [Build-Time Validation](#build-time-validation)
18. [Testing Your Tweaks](#testing-your-tweaks)
19. [Troubleshooting](#troubleshooting)

---

## Overview

Tweaks are defined in YAML files under `src-tauri/tweaks/`. Each file contains:
- One **category** (a grouping for UI display)
- Multiple **tweaks** (the actual settings modifications)

Tweaks use a **unified option-based model** where every tweak has an `options` array. Each option defines a complete state with all the changes needed to achieve that state.

### Key Concepts

| Concept              | Description                                                       |
| -------------------- | ----------------------------------------------------------------- |
| **Option**           | A complete state for a tweak (e.g., "Enabled", "Disabled", "4MB") |
| **Toggle Tweak**     | Has exactly 2 options, displays as an on/off switch               |
| **Dropdown Tweak**   | Has 3+ options, displays as a dropdown selector                   |
| **Snapshot**         | Captured original state before applying a tweak (for reverting)   |
| **Atomic Execution** | Registry, services, and scheduler changes are all-or-nothing      |

---

## Quick Start

Here's a minimal tweak that disables a registry setting:

```yaml
category:
  id: example
  name: "Example Category"
  description: "Example tweaks for demonstration"
  icon: "mdi:cog"
  order: 99

tweaks:
  - id: disable_example_feature
    name: "Disable Example Feature"
    description: "A simple toggle tweak"
    risk_level: low
    requires_reboot: false
    is_toggle: true
    options:
      - label: "Disabled"
        registry_changes:
          - hive: HKCU
            key: "Software\\Example\\Feature"
            value_name: "Enabled"
            value_type: "REG_DWORD"
            value: 0
      - label: "Enabled"
        registry_changes:
          - hive: HKCU
            key: "Software\\Example\\Feature"
            value_name: "Enabled"
            value_type: "REG_DWORD"
            value: 1
```

---

## YAML File Structure

Each YAML file must have exactly this structure:

```yaml
category:
  # Category definition (one per file)

tweaks:
  # Array of tweak definitions
  - id: tweak_one
    # ...
  - id: tweak_two
    # ...
```

**Rules:**
- One category per file
- Unlimited tweaks per file (group related tweaks together)
- File must be in `src-tauri/tweaks/` directory
- File extension must be `.yaml`
- Tweaks are compiled at build time (no runtime YAML parsing)

---

## Category Definition

Categories group related tweaks in the UI.

```yaml
category:
  id: string              # Required: Unique identifier (snake_case)
  name: string            # Required: Display name in UI
  description: string     # Required: Category description
  icon: string            # Required: MDI icon name (e.g., "mdi:shield-lock")
  order: number           # Optional: Sort order (lower = higher in list, default: 0)
```

### Category Field Details

| Field         | Type   | Required | Description                                                                              |
| ------------- | ------ | -------- | ---------------------------------------------------------------------------------------- |
| `id`          | string | ✅        | Unique identifier across all categories. Use `snake_case`.                               |
| `name`        | string | ✅        | Display name shown in sidebar and headers.                                               |
| `description` | string | ✅        | Brief description of what this category contains.                                        |
| `icon`        | string | ✅        | [Material Design Icons](https://pictogrammers.com/library/mdi/) name with `mdi:` prefix. |
| `order`       | number | ❌        | Sort priority. Categories with lower numbers appear first. Default: `0`.                 |

### Example Categories

```yaml
# Privacy category - appears first
category:
  id: privacy
  name: "Privacy"
  description: "Reduce telemetry, tracking, and data collection"
  icon: "mdi:shield-lock"
  order: 1

# Gaming category - appears later
category:
  id: gaming
  name: "Gaming"
  description: "Optimize Windows for gaming performance"
  icon: "mdi:gamepad-variant"
  order: 6
```

---

## Tweak Definition

Each tweak defines a configurable Windows setting.

```yaml
- id: string                    # Required: Unique identifier
  name: string                  # Required: Display name
  description: string           # Required: Short description
  info: string                  # Optional: Detailed documentation
  risk_level: low|medium|high|critical  # Required: Impact level
  requires_admin: boolean       # Optional: Needs admin privileges
  requires_system: boolean      # Optional: Needs SYSTEM elevation (implies admin)
  requires_ti: boolean          # Optional: Needs TrustedInstaller (implies system & admin)
  requires_reboot: boolean      # Required: Needs restart to take effect
  is_toggle: boolean            # Required: true = switch UI, false = dropdown
  options: []                   # Required: Array of option definitions
```

### Tweak Field Details

| Field             | Type    | Required | Default | Description                                                         |
| ----------------- | ------- | -------- | ------- | ------------------------------------------------------------------- |
| `id`              | string  | ✅        | -       | Unique identifier across ALL tweaks in ALL files. Use `snake_case`. |
| `name`            | string  | ✅        | -       | Display name shown in the tweak card.                               |
| `description`     | string  | ✅        | -       | One-line description shown under the name.                          |
| `info`            | string  | ❌        | -       | Extended documentation shown in info popup.                         |
| `risk_level`      | enum    | ✅        | -       | One of: `low`, `medium`, `high`, `critical`.                        |
| `requires_admin`  | boolean | ❌        | `false` | Requires running as Administrator. Auto-inferred if system/ti set.  |
| `requires_system` | boolean | ❌        | `false` | Requires SYSTEM elevation. Auto-inferred if ti is set.              |
| `requires_ti`     | boolean | ❌        | `false` | Requires TrustedInstaller elevation (for WaaSMedicSvc, etc.)        |
| `requires_reboot` | boolean | ✅        | `false` | Changes require restart to fully apply.                             |
| `is_toggle`       | boolean | ✅        | `false` | `true` = 2-option switch, `false` = dropdown.                       |
| `options`         | array   | ✅        | -       | Array of available states for this tweak.                           |

### Risk Levels Explained

| Level      | Color  | When to Use                                                  |
| ---------- | ------ | ------------------------------------------------------------ |
| `low`      | Green  | Safe changes that can be easily reverted. No system impact.  |
| `medium`   | Yellow | Changes that may require restart or have minor side effects. |
| `high`     | Orange | Significant system impact. Could cause issues if misused.    |
| `critical` | Red    | Could break Windows. Only for advanced users.                |

### Privilege Requirements

The permission system uses a hierarchical model: **TrustedInstaller > System > Admin > None**

Lower permissions are automatically inferred from higher ones:
- `requires_ti: true` → automatically sets `requires_system: true` and `requires_admin: true`
- `requires_system: true` → automatically sets `requires_admin: true`

**Only specify the highest permission level needed:**

```yaml
# No special privileges (HKCU changes only)
# Just omit all permission flags - they default to false

# Administrator required (HKLM changes)
requires_admin: true

# SYSTEM elevation required (protected registry keys/services)
requires_system: true
# Note: requires_admin is automatically inferred, don't specify it

# TrustedInstaller required (heavily protected services like WaaSMedicSvc)
requires_ti: true
# Note: Both requires_system and requires_admin are automatically inferred
```

**When is `requires_admin: true` needed?**
- Any changes to HKLM (HKEY_LOCAL_MACHINE) registry keys
- Changes that affect all users on the system

**When is `requires_system: true` needed?**
- Protected registry keys (e.g., under `SYSTEM\CurrentControlSet\Services\`)
- Protected scheduled tasks
- Some Windows services that resist changes
- Generally: If normal admin elevation fails, try SYSTEM elevation

**When is `requires_ti: true` needed?**
- Services owned by TrustedInstaller that cannot be modified even as SYSTEM
- Primary example: **WaaSMedicSvc** (Windows Update Medic Service)
- Any resource with an ACL that grants access only to TrustedInstaller
- Generally: If SYSTEM elevation still fails with "Access Denied", use TrustedInstaller

---

## Options Array

Every tweak must have an `options` array. Each option represents a complete state.

### Toggle Tweaks (2 Options)

For `is_toggle: true`, you must have **exactly 2 options**:

```yaml
is_toggle: true
options:
  - label: "Option A"      # Index 0 - Shown when switch is ON
    # ... changes for option A
  - label: "Option B"      # Index 1 - Shown when switch is OFF
    # ... changes for option B
```

**Convention:** For toggle tweaks:
- `options[0]` = The "tweaked" state (what the user wants to achieve)
- `options[1]` = The "default/normal" state

### Dropdown Tweaks (3+ Options)

For `is_toggle: false`, you can have any number of options (minimum 1):

```yaml
is_toggle: false
options:
  - label: "500 KB (Default)"
    # ... changes
  - label: "2 MB"
    # ... changes
  - label: "4 MB"
    # ... changes
  - label: "8 MB"
    # ... changes
```

### Option Structure

```yaml
options:
  - label: string                # Required: Display name for this option
    registry_changes: []         # Optional: Registry modifications
    service_changes: []          # Optional: Windows service changes
    scheduler_changes: []        # Optional: Task Scheduler changes
    pre_commands: []             # Optional: Shell commands BEFORE changes
    pre_powershell: []           # Optional: PowerShell BEFORE changes
    post_commands: []            # Optional: Shell commands AFTER changes
    post_powershell: []          # Optional: PowerShell AFTER changes
```

---

## Change Types

### Registry Changes

Modify Windows Registry values.

```yaml
registry_changes:
  - hive: HKCU | HKLM           # Required: Registry hive
    key: string                  # Required: Registry key path (no hive prefix)
    value_name: string           # Required: Value name (use "" for default)
    value_type: string           # Required: REG_DWORD, REG_SZ, etc.
    value: any                   # Required: Target value
    windows_versions: [10, 11]   # Optional: Filter by Windows version
    skip_validation: boolean     # Optional: Exclude from status check
```

#### Registry Field Details

| Field              | Type    | Required | Description                                                                |
| ------------------ | ------- | -------- | -------------------------------------------------------------------------- |
| `hive`             | enum    | ✅        | `HKCU` (Current User) or `HKLM` (Local Machine).                           |
| `key`              | string  | ✅        | Path without hive. Use `\\` for separators.                                |
| `value_name`       | string  | ✅        | Name of the value. Empty string `""` for default value.                    |
| `value_type`       | enum    | ✅        | Registry value type (see table below).                                     |
| `value`            | any     | ✅        | The value to set. Type depends on `value_type`.                            |
| `windows_versions` | array   | ❌        | Only apply on specific Windows versions.                                   |
| `skip_validation`  | boolean | ❌        | Default `false`. See [skip_validation section](#the-skip_validation-flag). |

#### Registry Value Types

| Type            | YAML Syntax             | Description                  |
| --------------- | ----------------------- | ---------------------------- |
| `REG_DWORD`     | `value: 1`              | 32-bit integer               |
| `REG_QWORD`     | `value: 12345678900`    | 64-bit integer               |
| `REG_SZ`        | `value: "string"`       | String value                 |
| `REG_EXPAND_SZ` | `value: "%PATH%"`       | Expandable string            |
| `REG_BINARY`    | `value: [0, 1, 2, 255]` | Binary data (array of bytes) |
| `REG_MULTI_SZ`  | *(Limited support)*     | Multi-string (read-only)     |

#### Registry Examples

```yaml
# DWORD (integer)
- hive: HKCU
  key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
  value_name: "HideFileExt"
  value_type: "REG_DWORD"
  value: 0

# String
- hive: HKLM
  key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer"
  value_name: "Max Cached Icons"
  value_type: "REG_SZ"
  value: "4096"

# Empty string (default value)
- hive: HKCU
  key: "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae9a9}\\InprocServer32"
  value_name: ""
  value_type: "REG_SZ"
  value: ""

# Binary data
- hive: HKLM
  key: "System\\CurrentControlSet\\Services\\Example"
  value_name: "Config"
  value_type: "REG_BINARY"
  value: [0, 1, 2, 3, 255]

# Delete a value (set to null)
- hive: HKCU
  key: "Software\\Example"
  value_name: "DeleteMe"
  value_type: "REG_SZ"
  value: null

# Windows 11 only
- hive: HKCU
  key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
  value_name: "TaskbarAl"
  value_type: "REG_DWORD"
  value: 0
  windows_versions: [11]
```

#### HKCU vs HKLM

| Hive   | Full Name          | Requires Admin | Affects           |
| ------ | ------------------ | -------------- | ----------------- |
| `HKCU` | HKEY_CURRENT_USER  | ❌ No           | Current user only |
| `HKLM` | HKEY_LOCAL_MACHINE | ✅ Yes          | All users         |

---

### Service Changes

Modify Windows service startup behavior.

```yaml
service_changes:
  - name: string                 # Required: Service name
    startup: string              # Required: Startup type
    stop_service: boolean        # Optional: Stop after changing (default: false)
    start_service: boolean       # Optional: Start after changing (default: false)
    skip_validation: boolean     # Optional: Exclude from status check
```

#### Service Field Details

| Field             | Type    | Required | Default | Description                                                  |
| ----------------- | ------- | -------- | ------- | ------------------------------------------------------------ |
| `name`            | string  | ✅        | -       | Service name (not display name). Use `sc query` to find.     |
| `startup`         | enum    | ✅        | -       | One of: `disabled`, `manual`, `automatic`, `boot`, `system`. |
| `stop_service`    | boolean | ❌        | `false` | Stop the service after changing startup type.                |
| `start_service`   | boolean | ❌        | `false` | Start the service after changing startup type.               |
| `skip_validation` | boolean | ❌        | `false` | See [skip_validation section](#the-skip_validation-flag).    |

#### Service Startup Types

| Type        | Registry Value | Description                        |
| ----------- | -------------- | ---------------------------------- |
| `disabled`  | 4              | Service cannot start               |
| `manual`    | 3              | Service starts only when requested |
| `automatic` | 2              | Service starts at boot             |
| `boot`      | 0              | Kernel driver (boot-start)         |
| `system`    | 1              | Kernel driver (system-start)       |

**Note:** When setting `startup: disabled`, the service is automatically stopped (equivalent to `stop_service: true`).

#### Service Examples

```yaml
# Disable and stop a service
- name: "DiagTrack"
  startup: disabled

# Enable and start a service
- name: "Spooler"
  startup: automatic
  start_service: true

# Disable a stubborn service (skip validation because it re-enables itself)
- name: "wuauserv"
  startup: disabled
  skip_validation: true
```

---

### Scheduler Changes

Modify Windows Task Scheduler tasks. Supports both exact task names and regex patterns for bulk operations.

```yaml
scheduler_changes:
  # Option 1: Exact task name
  - task_path: string            # Required: Task folder path
    task_name: string            # Required (if no pattern): Exact task name
    action: string               # Required: enable, disable, or delete
    skip_validation: boolean     # Optional: Exclude from status check
    ignore_not_found: boolean    # Optional: Don't error if task doesn't exist

  # Option 2: Pattern matching (regex)
  - task_path: string            # Required: Task folder path
    task_name_pattern: string    # Required (if no name): Regex pattern to match task names
    action: string               # Required: enable, disable, or delete
    skip_validation: boolean     # Optional: Exclude from status check
    ignore_not_found: boolean    # Optional: Don't error if no tasks match
```

#### Scheduler Field Details

| Field               | Type    | Required | Description                                                                    |
| ------------------- | ------- | -------- | ------------------------------------------------------------------------------ |
| `task_path`         | string  | ✅        | Folder path in Task Scheduler. Use `\\` prefix.                                |
| `task_name`         | string  | ⚠️        | Exact name of the scheduled task. **Required if `task_name_pattern` not set.** |
| `task_name_pattern` | string  | ⚠️        | Regex pattern to match multiple tasks. **Required if `task_name` not set.**    |
| `action`            | enum    | ✅        | `enable`, `disable`, or `delete`.                                              |
| `skip_validation`   | boolean | ❌        | Default `false`. See [skip_validation section](#the-skip_validation-flag).     |
| `ignore_not_found`  | boolean | ❌        | Default `false`. See [ignore_not_found section](#the-ignore_not_found-flag).   |

> **Note:** You must specify either `task_name` OR `task_name_pattern`, but not both.

#### Scheduler Actions

| Action    | Description                              |
| --------- | ---------------------------------------- |
| `enable`  | Enable a disabled task                   |
| `disable` | Disable a task (can be re-enabled later) |
| `delete`  | Permanently remove a task                |

**Warning:** `delete` is irreversible. The task cannot be restored by reverting the tweak. Use `disable` unless you're certain.

#### The `ignore_not_found` Flag

Controls behavior when a task (or all tasks matching a pattern) doesn't exist:

| Scenario                          | `ignore_not_found: false` (default) | `ignore_not_found: true`        |
| --------------------------------- | ----------------------------------- | ------------------------------- |
| Exact task doesn't exist          | ❌ Error (rollback)                  | ⚠️ Warning (continue)            |
| Pattern matches no tasks          | ❌ Error (rollback)                  | ⚠️ Warning (continue)            |
| Status detection (task not found) | ❌ Doesn't match                     | ✅ Matches (treated as expected) |

**Use case:** Optional tasks that may not exist on all Windows versions or editions.

#### Pattern Matching with `task_name_pattern`

Use regex patterns to target multiple tasks in a folder:

```yaml
# Match all tasks containing these keywords
scheduler_changes:
  - task_path: "\\Microsoft\\Windows\\UpdateOrchestrator"
    task_name_pattern: "USO|MusNotification|Reboot|Refresh"
    action: disable
    ignore_not_found: true   # Some tasks may not exist on all systems
```

The pattern uses Rust regex syntax. Common patterns:
- `|` - Alternation (OR): `"Task1|Task2|Task3"`
- `.` - Any character: `"Schedule.*"`
- `^` - Start of name: `"^Backup"`
- `$` - End of name: `"Report$"`
- `.*` - Match anything: `"Microsoft.*Update"`

#### Behavior Matrix (Scheduler)

| Flag Combination         | Task Not Found       | Action Failed        | Status Detection       |
| ------------------------ | -------------------- | -------------------- | ---------------------- |
| Default (both false)     | ❌ Error → rollback   | ❌ Error → rollback   | ✅ Included             |
| `ignore_not_found: true` | ✅ Warning → continue | ❌ Error → rollback   | ✅ Matches if not found |
| `skip_validation: true`  | ❌ Error → rollback   | ✅ Warning → continue | ❌ Excluded             |
| Both `true`              | ✅ Warning → continue | ✅ Warning → continue | ❌ Excluded             |

#### Finding Task Paths

Open Task Scheduler (`taskschd.msc`) and navigate to the task. The path is shown in the folder tree.

Common paths:
- `\Microsoft\Windows\Customer Experience Improvement Program`
- `\Microsoft\Windows\Application Experience`
- `\Microsoft\Windows\UpdateOrchestrator`
- `\Microsoft\Windows\WindowsUpdate`

#### Scheduler Examples

```yaml
# Exact task name - disable single task
- task_path: "\\Microsoft\\Windows\\Customer Experience Improvement Program"
  task_name: "Consolidator"
  action: disable

# Pattern matching - disable all update-related tasks in a folder
- task_path: "\\Microsoft\\Windows\\UpdateOrchestrator"
  task_name_pattern: "Schedule|USO|MusNotification|Reboot"
  action: disable
  ignore_not_found: true   # Not all tasks exist on every system

# Optional task - don't fail if it doesn't exist
- task_path: "\\Microsoft\\Windows\\Application Experience"
  task_name: "MareBackup"
  action: disable
  ignore_not_found: true

# Delete tasks permanently (use with caution)
- task_path: "\\Microsoft\\Windows\\Application Experience"
  task_name: "ProgramDataUpdater"
  action: delete
```

---

### Shell Commands

Run shell commands via `cmd.exe`.

```yaml
pre_commands:      # Run BEFORE registry/service/scheduler changes
  - "ipconfig /flushdns"
  - "net stop SomeService"

post_commands:     # Run AFTER registry/service/scheduler changes
  - "taskkill /f /im explorer.exe"
  - "start explorer.exe"
```

**Execution:**
- Commands run via `cmd.exe /C <command>`
- If `requires_system: true`, commands run as SYSTEM
- If `requires_ti: true`, commands run as TrustedInstaller
- Working directory is the executable directory

**See [Error Handling](#error-handling-behavior) for failure behavior.**

---

### PowerShell Commands

Run PowerShell scripts.

```yaml
pre_powershell:    # Run BEFORE registry/service/scheduler changes
  - "Stop-Process -Name 'explorer' -Force"

post_powershell:   # Run AFTER registry/service/scheduler changes
  - "Get-AppxPackage *xbox* | Remove-AppxPackage"
```

**Execution:**
- Commands run via PowerShell (or as SYSTEM if `requires_system: true`)
- If `requires_ti: true`, PowerShell commands run as TrustedInstaller
- Can span multiple lines if needed (use YAML multiline syntax)
- Exit code 0 = success, non-zero = failure

**See [Error Handling](#error-handling-behavior) for failure behavior.**

#### PowerShell Example with Exit Code

```yaml
post_powershell:
  - |
    $path='\\Microsoft\\Windows\\UpdateOrchestrator\\'
    $fail=0
    Get-ScheduledTask -TaskPath $path -ErrorAction SilentlyContinue |
    Where-Object {$_.TaskName -match 'Schedule Scan|USO'} |
    ForEach-Object {
        try {
            Disable-ScheduledTask -InputObject $_ -ErrorAction Stop
        } catch {
            $fail=1
        }
    }
    exit $fail
```

---

## Execution Order & Atomicity

When applying an option, changes execute in this **exact order**:

```
1. pre_commands         ← Shell commands (cmd.exe)
2. pre_powershell       ← PowerShell commands
3. registry_changes     ← Registry modifications      ┐
4. service_changes      ← Windows service changes     │ ATOMIC
5. scheduler_changes    ← Task Scheduler changes      ┘
6. post_commands        ← Shell commands (cmd.exe)
7. post_powershell      ← PowerShell commands
```

### What "Atomic" Means

Steps 3, 4, and 5 (registry, services, scheduler) are **atomic**:
- If **ANY** of these steps fails, **ALL** changes are rolled back
- Rollback uses the snapshot captured before step 1
- You get either complete success or complete rollback

### What's NOT Atomic

- `pre_commands` and `pre_powershell` run **before** the snapshot is used
- `post_commands` and `post_powershell` run **after** atomic changes
- These command steps have different failure behavior (see next section)

---

## Error Handling Behavior

**CRITICAL: Understanding which errors are fatal vs. non-fatal is essential for writing robust tweaks.**

| Step                | Fatal on Error? | Triggers Rollback?         | Notes                               |
| ------------------- | --------------- | -------------------------- | ----------------------------------- |
| `pre_commands`      | ✅ **YES**       | ❌ No (nothing applied yet) | Aborts before any changes           |
| `pre_powershell`    | ✅ **YES**       | ❌ No (nothing applied yet) | Aborts before any changes           |
| `registry_changes`  | ✅ **YES**       | ✅ **YES**                  | Rolls back all registry changes     |
| `service_changes`   | ✅ **YES**       | ✅ **YES**                  | Rolls back everything from snapshot |
| `scheduler_changes` | ✅ **YES**       | ✅ **YES**                  | Rolls back everything from snapshot |
| `post_commands`     | ❌ **NO**        | ❌ No                       | Logged as warning, continues        |
| `post_powershell`   | ❌ **NO**        | ❌ No                       | Logged as warning, continues        |

### Key Insights

1. **Pre-commands are fail-fast**: If `pre_commands` or `pre_powershell` fail, the operation aborts immediately. No changes are made.

2. **Core changes are atomic**: If registry, service, or scheduler changes fail, everything rolls back to the original state.

3. **Post-commands are non-fatal**: If `post_commands` or `post_powershell` fail, the tweak is still considered "applied". Errors are logged but don't abort.

### Practical Implications

```yaml
options:
  - label: "Apply"
    # Use pre_commands for REQUIRED setup that must succeed
    pre_commands:
      - "net stop RequiredService"  # If this fails, abort

    registry_changes:
      - # ... these are atomic

    # Use post_commands for OPTIONAL cleanup that can fail safely
    post_commands:
      - "taskkill /f /im explorer.exe"   # OK if fails
      - "start explorer.exe"             # OK if fails
```

---

## The `skip_validation` Flag

The `skip_validation` flag is a **powerful feature** for handling items that behave unexpectedly.

### What It Does

When `skip_validation: true` is set on a registry, service, or scheduler change:

1. **Status Detection**: The item is **excluded** from tweak status checks
2. **Atomic Rollback**: Failures for this item **don't trigger** a full rollback
3. **Execution**: The change is **still attempted**, but failures are logged as warnings

### When to Use It

**Use Case 1: Self-Healing Services**

Some Windows services (like `wuauserv`) re-enable themselves when you open Settings or certain apps. Without `skip_validation`, your tweak would show as "not applied" whenever this happens.

```yaml
service_changes:
  - name: "wuauserv"
    startup: disabled
    skip_validation: true    # Don't let this service control tweak status
  - name: "WaaSMedicSvc"
    startup: disabled        # This one is reliable
```

**Use Case 2: Protected Resources**

Some resources might fail to change due to permissions, but you don't want to abort the entire tweak.

```yaml
scheduler_changes:
  - task_path: "\\Microsoft\\Windows\\UpdateOrchestrator"
    task_name: "ScheduleScan"
    action: disable
    skip_validation: true    # Might fail on some systems, but continue
```

**Use Case 3: Version-Specific Items**

When combining changes that might not exist on all systems.

### How Status Detection Works with skip_validation

```
Tweak Status Detection:
1. Get all registry_changes where skip_validation != true
2. Get all service_changes where skip_validation != true
3. Check if ALL of these match the option's expected values
4. If yes → option is "applied"
5. If no → check next option
```

**Example:**
```yaml
options:
  - label: "Disabled"
    registry_changes:
      - hive: HKLM
        key: "..."
        value_name: "A"
        value: 0            # This is checked for status
    service_changes:
      - name: "ServiceA"
        startup: disabled   # This is checked for status
      - name: "wuauserv"
        startup: disabled
        skip_validation: true   # This is NOT checked for status
```

Even if `wuauserv` re-enables itself, the tweak shows as "Disabled" as long as registry value `A` and `ServiceA` are correct.

---

## State Detection

The app detects which option is currently active by comparing system state against all options.

### Detection Algorithm

```
For each option in tweak.options:
  1. Filter registry_changes by:
     - skip_validation == false
     - applies_to_version(current_windows)
  2. Filter service_changes by:
     - skip_validation == false
  3. If no validatable changes remain → skip option
  4. For each validatable registry change:
     - Read current registry value
     - Compare with option's expected value
     - If mismatch → option doesn't match, try next
  5. For each validatable service change:
     - Get current service startup type
     - Compare with option's expected startup
     - If mismatch → option doesn't match, try next
  6. If all checks pass → this option is current
```

### Special Cases

1. **No Match (System Default)**: If no option matches current state, UI shows "System Default" placeholder

2. **Empty Options**: If an option has no validatable changes (all filtered out), it cannot be detected as current

3. **Value Comparison**: Handles numeric type variations (DWORD as u32 or u64)

---

## Snapshot & Revert System

Before applying any tweak, the app captures a **snapshot** of the original state.

### What Gets Captured

- All registry values that will be modified
- All service startup types and running states
- All scheduled task states (enabled/disabled)

### Snapshot Storage

- Location: `snapshots/` directory next to the executable
- Format: `{tweak_id}.json`
- One snapshot per tweak (not per option)

### Revert Behavior

When reverting:
1. Load snapshot for the tweak
2. Restore all registry values to original
3. Restore all service states to original
4. Restore all scheduler states to original
5. Delete the snapshot

### Snapshot Lifecycle

```
First Apply:
  1. Capture snapshot (original pre-tweak state)
  2. Save snapshot to disk
  3. Apply option changes
  4. On FAILURE: Restore snapshot, delete it

Switching Options (snapshot exists):
  1. Capture current system state (pre-apply state)
  2. Apply new option changes
  3. On SUCCESS: Update snapshot metadata (option index/label)
  4. On FAILURE: Restore to pre-apply state (previous option)
     - Original snapshot is PRESERVED
     - Tweak stays at the previous option

Revert:
  1. Load snapshot
  2. Restore original pre-tweak state
  3. Delete snapshot
```

### Rollback Behavior

The rollback system handles failures differently based on context:

| Scenario                    | Failure Behavior                          | Result                                                       |
| --------------------------- | ----------------------------------------- | ------------------------------------------------------------ |
| **First apply fails**       | Restore from original snapshot, delete it | System returns to original state, tweak not applied          |
| **Switching options fails** | Restore from pre-apply state              | System stays at previous option, original snapshot preserved |
| **Revert**                  | Restore from original snapshot            | System returns to original pre-tweak state                   |

**Key Insight:** When switching between options (e.g., "Disabled" → "Enabled"), if the apply fails midway, the system rolls back to the **previous option state**, not the original pre-tweak state. This prevents losing your current configuration due to a partial failure.

### Snapshot Contents

The snapshot stores:
- `applied_option_index` / `applied_option_label`: Which option was last successfully applied
- `registry_snapshots`: Original registry values before first apply
- `service_snapshots`: Original service states before first apply
- `scheduler_snapshots`: Original task states before first apply

**Important:** The registry/service/scheduler values in the snapshot represent the **original pre-tweak state**, not the current option's state. The snapshot metadata (option index/label) is updated when switching options successfully.

---

## Windows Version Filtering

Filter changes to apply only on specific Windows versions.

```yaml
registry_changes:
  - hive: HKCU
    key: "Software\\Example"
    value_name: "Win10Only"
    value_type: "REG_DWORD"
    value: 1
    windows_versions: [10]       # Only Windows 10

  - hive: HKCU
    key: "Software\\Example"
    value_name: "Win11Only"
    value_type: "REG_DWORD"
    value: 1
    windows_versions: [11]       # Only Windows 11

  - hive: HKCU
    key: "Software\\Example"
    value_name: "BothVersions"
    value_type: "REG_DWORD"
    value: 1
    windows_versions: [10, 11]   # Both versions (same as omitting)
```

### Behavior

- **Omitted `windows_versions`**: Change applies to all versions
- **Empty array `[]`**: Change applies to all versions
- **`[10]`**: Only Windows 10
- **`[11]`**: Only Windows 11
- **`[10, 11]`**: Both versions (explicit)

### Version Detection

The app detects Windows version at runtime:
- Windows 10 → version `10`
- Windows 11 → version `11`

---

## Complete Examples

### Example 1: Simple Toggle (Registry Only)

```yaml
- id: show_file_extensions
  name: "Show File Extensions"
  description: "Always show file extensions in File Explorer"
  risk_level: low
  requires_admin: false
  requires_reboot: false
  requires_system: false
  is_toggle: true
  options:
    - label: "Extensions Visible"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
          value_name: "HideFileExt"
          value_type: "REG_DWORD"
          value: 0
    - label: "Extensions Hidden (Default)"
      registry_changes:
        - hive: HKCU
          key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
          value_name: "HideFileExt"
          value_type: "REG_DWORD"
          value: 1
```

### Example 2: Complex Toggle (Registry + Services + Scheduler + PowerShell)

```yaml
- id: disable_windows_update_complete
  name: "Disable Windows Update (Complete)"
  description: "Completely disable the Windows Update service and automatic updates"
  risk_level: high
  requires_admin: true
  requires_reboot: false
  requires_system: false
  is_toggle: true
  info: |
    Stops and disables the Windows Update service.
    Updates must be installed manually.
    Use with caution - security updates will not be installed automatically.
  options:
    - label: "Windows Update Disabled"
      post_powershell:
        - |
          $path='\\Microsoft\\Windows\\UpdateOrchestrator\\'
          $fail=0
          Get-ScheduledTask -TaskPath $path -ErrorAction SilentlyContinue |
          Where-Object {$_.TaskName -match 'Schedule Scan|USO|MusNotification|Reboot|Refresh'} |
          ForEach-Object {
              try {
                  Disable-ScheduledTask -InputObject $_ -ErrorAction Stop
                  if ((Get-ScheduledTask -InputObject $_).State -ne 'Disabled') { $fail=1 }
              } catch { $fail=1 }
          }
          exit $fail
      registry_changes:
        - hive: HKLM
          key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU"
          value_name: "AUOptions"
          value_type: "REG_DWORD"
          value: 1
        - hive: HKLM
          key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU"
          value_name: "NoAutoUpdate"
          value_type: "REG_DWORD"
          value: 1
        - hive: HKLM
          key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate"
          value_name: "SetDisableUXWUAccess"
          value_type: "REG_DWORD"
          value: 1
      service_changes:
        - name: "wuauserv"
          startup: disabled
          skip_validation: true     # Re-enables itself when Settings opens
        - name: "WaaSMedicSvc"
          startup: disabled
        - name: "UsoSvc"
          startup: disabled

    - label: "Windows Update Enabled"
      post_powershell:
        - |
          $path='\\Microsoft\\Windows\\UpdateOrchestrator\\'
          Get-ScheduledTask -TaskPath $path -ErrorAction SilentlyContinue |
          Where-Object {$_.TaskName -match 'Schedule Scan|USO|MusNotification|Reboot|Refresh'} |
          ForEach-Object { Enable-ScheduledTask -InputObject $_ -ErrorAction SilentlyContinue }
      registry_changes:
        - hive: HKLM
          key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU"
          value_name: "AUOptions"
          value_type: "REG_DWORD"
          value: 2
        - hive: HKLM
          key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU"
          value_name: "NoAutoUpdate"
          value_type: "REG_DWORD"
          value: 0
        - hive: HKLM
          key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate"
          value_name: "SetDisableUXWUAccess"
          value_type: "REG_DWORD"
          value: 0
      service_changes:
        - name: "wuauserv"
          startup: manual
          skip_validation: true
        - name: "WaaSMedicSvc"
          startup: manual
        - name: "UsoSvc"
          startup: automatic
```

### Example 3: Dropdown Tweak (Multi-Option)

```yaml
- id: icon_cache_size
  name: "Icon Cache Size"
  description: "Increase icon cache to prevent corruption"
  risk_level: low
  requires_admin: true
  requires_reboot: false
  requires_system: false
  is_toggle: false
  info: "Larger cache prevents icon corruption but uses more disk space"
  options:
    - label: "500 KB (Default)"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: "REG_SZ"
          value: "500"

    - label: "2 MB"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: "REG_SZ"
          value: "2048"

    - label: "4 MB"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: "REG_SZ"
          value: "4096"

    - label: "8 MB"
      registry_changes:
        - hive: HKLM
          key: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"
          value_name: "Max Cached Icons"
          value_type: "REG_SZ"
          value: "8192"
```

### Example 4: Windows 11 Only Tweak

```yaml
- id: classic_context_menu_win11
  name: "Enable Classic Context Menu (Windows 11)"
  description: "Use classic right-click context menu in Windows 11"
  risk_level: low
  requires_admin: true
  requires_reboot: false
  requires_system: false
  is_toggle: true
  options:
    - label: "Classic Menu Enabled"
      registry_changes:
        - hive: HKCU
          key: "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae9a9}\\InprocServer32"
          value_name: ""
          value_type: "REG_SZ"
          value: ""
          windows_versions: [11]

    - label: "Modern Menu (Default)"
      registry_changes:
        - hive: HKCU
          key: "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae9a9}\\InprocServer32"
          value_name: ""
          value_type: "REG_SZ"
          value: null
          windows_versions: [11]
```

### Example 5: Service-Only Tweak

```yaml
- id: disable_print_spooler
  name: "Disable Print Spooler"
  description: "Disable printing if you don't use a printer"
  risk_level: medium
  requires_admin: true
  requires_reboot: false
  requires_system: false
  is_toggle: true
  info: "Disable if you don't use printing. Spooler service can be a security risk."
  options:
    - label: "Disabled"
      registry_changes: []        # No registry changes
      service_changes:
        - name: "Spooler"
          startup: disabled

    - label: "Enabled"
      registry_changes: []
      service_changes:
        - name: "Spooler"
          startup: automatic
          start_service: true
```

### Example 6: Scheduler-Only Tweak

```yaml
- id: disable_telemetry_tasks
  name: "Disable Telemetry Tasks"
  description: "Disable telemetry-related scheduled tasks"
  risk_level: low
  requires_admin: true
  requires_reboot: false
  requires_system: false
  is_toggle: true
  options:
    - label: "Tasks Disabled"
      scheduler_changes:
        - task_path: "\\Microsoft\\Windows\\Customer Experience Improvement Program"
          task_name: "Consolidator"
          action: disable
        - task_path: "\\Microsoft\\Windows\\Customer Experience Improvement Program"
          task_name: "UsbCeip"
          action: disable
        - task_path: "\\Microsoft\\Windows\\Application Experience"
          task_name: "Microsoft Compatibility Appraiser"
          action: disable

    - label: "Tasks Enabled"
      scheduler_changes:
        - task_path: "\\Microsoft\\Windows\\Customer Experience Improvement Program"
          task_name: "Consolidator"
          action: enable
        - task_path: "\\Microsoft\\Windows\\Customer Experience Improvement Program"
          task_name: "UsbCeip"
          action: enable
        - task_path: "\\Microsoft\\Windows\\Application Experience"
          task_name: "Microsoft Compatibility Appraiser"
          action: enable
```

### Example 7: Command-Based Tweak

```yaml
- id: flush_dns_cache
  name: "Flush DNS Cache"
  description: "Clear DNS resolver cache"
  risk_level: low
  requires_admin: true
  requires_reboot: false
  requires_system: false
  is_toggle: true
  info: "Use this to resolve DNS issues or after changing DNS servers"
  options:
    - label: "Flush Now"
      registry_changes: []
      post_commands:
        - "ipconfig /flushdns"

    - label: "Default"
      registry_changes: []
      # No-op option (nothing to do)
```

---

## Best Practices

### 1. Always Provide Both States

Every option should have a clear counterpart:

```yaml
# ✅ Good: Clear toggle states
options:
  - label: "Disabled"
    # ... disable changes
  - label: "Enabled"
    # ... enable changes

# ❌ Bad: One-way operation
options:
  - label: "Delete Everything"
    post_commands:
      - "del /q C:\\*"
  # No way to restore!
```

### 2. Use Descriptive Labels

Labels should clearly indicate the state:

```yaml
# ✅ Good
- label: "Telemetry Disabled"
- label: "Game Mode Enabled"
- label: "4 MB Cache"

# ❌ Bad
- label: "Option 1"
- label: "Click Me"
- label: "Yes"
```

### 3. Set Appropriate Risk Levels

Be honest about risks:

```yaml
# Low: Can be safely toggled anytime
risk_level: low

# Medium: Might need restart, minor side effects
risk_level: medium

# High: Could break functionality
risk_level: high

# Critical: Could prevent boot or major issues
risk_level: critical
```

### 4. Use `info` for Documentation

Explain complex tweaks:

```yaml
info: |
  This tweak disables the Windows Update service entirely.

  **Effects:**
  - No automatic updates
  - Must manually download and install updates

  **Risks:**
  - Missing security patches
  - Software compatibility issues

  **When to use:**
  - On isolated systems
  - When testing specific Windows versions
```

### 5. Use `skip_validation` Wisely

Only use when necessary:

```yaml
# ✅ Good: Known problematic service
- name: "wuauserv"
  startup: disabled
  skip_validation: true   # Known to re-enable itself

# ❌ Bad: Using everywhere "just in case"
- name: "SomeService"
  startup: disabled
  skip_validation: true   # Why? Document the reason!
```

### 6. Order Options Logically

```yaml
# ✅ Good: Applied state first for toggles
options:
  - label: "Disabled"        # Index 0 = "tweaked" state
  - label: "Enabled"         # Index 1 = "normal" state

# For dropdowns, order by value or usability
options:
  - label: "500 KB (Default)"
  - label: "2 MB"
  - label: "4 MB"
  - label: "8 MB"
```

### 7. Test on Both Windows 10 and 11

Even if you target one version:

```yaml
# Explicitly mark version-specific changes
windows_versions: [11]

# Or ensure graceful handling on both versions
```

---

## Common Mistakes

### 1. Forgetting `requires_admin` for HKLM

```yaml
# ❌ Wrong
requires_admin: false
registry_changes:
  - hive: HKLM    # HKLM always needs admin!
    # ...

# ✅ Correct
requires_admin: true
registry_changes:
  - hive: HKLM
    # ...
```

### 2. Wrong Option Count for Toggle

```yaml
# ❌ Wrong: Toggle with 3 options
is_toggle: true
options:
  - label: "Low"
  - label: "Medium"
  - label: "High"

# ✅ Correct: Use dropdown for 3+ options
is_toggle: false
options:
  - label: "Low"
  - label: "Medium"
  - label: "High"
```

### 3. Using Backslash Incorrectly

```yaml
# ❌ Wrong: Single backslash (YAML escape)
key: "Software\Microsoft\Windows"

# ✅ Correct: Double backslash
key: "Software\\Microsoft\\Windows"
```

### 4. Relying on Post-Commands for Critical Work

```yaml
# ❌ Bad: Critical operation in post_commands (non-fatal)
post_commands:
  - "important_command_that_must_succeed"

# ✅ Better: Use pre_commands (fatal) or registry/service changes (atomic)
pre_commands:
  - "important_command_that_must_succeed"
```

### 5. Not Handling Restore State

```yaml
# ❌ Bad: Only one option, can't restore
options:
  - label: "Delete User Data"
    post_commands:
      - "del /q %USERPROFILE%\\AppData\\..."

# ✅ Better: Provide restore option or don't use destructive commands
options:
  - label: "Clear Cache"
    registry_changes:
      - # Reversible registry change instead
```

### 6. Misunderstanding skip_validation

```yaml
# ❌ Wrong: skip_validation doesn't mean "optional"
- name: "SomeService"
  startup: disabled
  skip_validation: true   # Still tries to disable!

# skip_validation means:
# - Still attempts the change
# - Failures don't rollback
# - Status detection ignores this item
```

---

## Build-Time Validation

The tweak system includes a **strict validation engine** that runs at build time. All YAML files are validated for structural and semantic correctness before compilation. Errors are reported with file names and tweak IDs for easy identification.

### What Gets Validated

| Check                            | Type    | Description                                                                 |
| -------------------------------- | ------- | --------------------------------------------------------------------------- |
| **Unknown Fields**               | Error   | Typos in field names are caught (e.g., `require_admin` vs `requires_admin`) |
| **Duplicate Tweak IDs**          | Error   | Each tweak must have a unique ID across all files                           |
| **Duplicate Category IDs**       | Error   | Each category must have a unique ID across all files                        |
| **Tweak ID Format**              | Error   | IDs must be snake_case (lowercase letters, digits, underscores)             |
| **Toggle Option Count**          | Error   | Tweaks with `is_toggle: true` must have exactly 2 options                   |
| **Duplicate Option Labels**      | Error   | Option labels must be unique within a tweak (case-insensitive)              |
| **Empty Options**                | Error   | Each option must have at least one change (registry, service, etc.)         |
| **Windows Versions**             | Error   | Only `10` and `11` are valid values                                         |
| **Registry Value Types**         | Error   | Values must match their declared `value_type`                               |
| **Registry Key/Value Names**     | Error   | Registry `key` cannot be empty                                              |
| **Service Names**                | Error   | Service `name` cannot be empty                                              |
| **Scheduler Task Path**          | Error   | `task_path` cannot be empty                                                 |
| **Scheduler Task Name**          | Error   | `task_name` or `task_name_pattern` cannot be empty                          |
| **Scheduler Mutual Exclusivity** | Error   | Cannot set both `task_name` and `task_name_pattern`                         |
| **Regex Patterns**               | Error   | `task_name_pattern` values must be valid regex                              |
| **Empty Registry Value Name**    | Warning | Empty `value_name` targets the default value (may be intentional)           |
| **HKLM Without Admin**           | Warning | HKLM registry changes should have `requires_admin: true`                    |

### Errors vs Warnings

- **Errors** are fatal and will fail the build. These represent invalid configurations that would cause runtime failures.
- **Warnings** are non-fatal and show during build output with `⚠`. These represent potentially problematic configurations that may be intentional.

### Registry Value Type Rules

| `value_type`    | Expected Value                       | Example                        |
| --------------- | ------------------------------------ | ------------------------------ |
| `REG_DWORD`     | Integer (0 to 4294967295)            | `value: 1`                     |
| `REG_QWORD`     | Integer (64-bit)                     | `value: 9223372036854775807`   |
| `REG_SZ`        | String                               | `value: "text"`                |
| `REG_EXPAND_SZ` | String (with environment variables)  | `value: "%USERPROFILE%\\path"` |
| `REG_MULTI_SZ`  | Array of strings                     | `value: ["a", "b"]`            |
| `REG_BINARY`    | Array of bytes (0-255) or hex string | `value: [0, 1, 255]`           |

### Common Validation Errors

#### Unknown Field

```
[privacy.yaml] Parse error: unknown field `require_admin`, expected one of `id`, `name`, ...
```

**Fix:** Check for typos in field names. Use `requires_admin`, not `require_admin`.

#### Invalid REG_DWORD Value

```
[privacy.yaml] Tweak 'my_tweak': option 'Enabled' registry change 'MyValue': REG_DWORD value -1 out of range (0..4294967295)
```

**Fix:** REG_DWORD is unsigned. Use `0` to `4294967295`. To "delete" a value, use PowerShell:

```yaml
post_powershell:
  - "Remove-ItemProperty -Path 'HKCU:\\Path' -Name 'Value' -ErrorAction SilentlyContinue"
```

#### REG_SZ with null value

```
[ui.yaml] Tweak 'my_tweak': option 'Default' registry change 'Value': REG_SZ requires string value, got null
```

**Fix:** Registry values can't be `null`. To delete a key, use PowerShell:

```yaml
post_powershell:
  - "Remove-Item -Path 'HKCU:\\Path\\To\\Key' -Recurse -Force -ErrorAction SilentlyContinue"
```

#### Scheduler Mutual Exclusivity

```
[windows_update.yaml] Tweak 'my_tweak': option 'Disabled' scheduler change: cannot specify both 'task_name' and 'task_name_pattern' (mutually exclusive)
```

**Fix:** Use either `task_name` for a single task OR `task_name_pattern` for multiple tasks:

```yaml
# Single task
scheduler_changes:
  - task_path: "\\Microsoft\\Windows\\Task"
    task_name: "SpecificTask"
    action: disable

# Multiple tasks by pattern
scheduler_changes:
  - task_path: "\\Microsoft\\Windows\\Task"
    task_name_pattern: "Task1|Task2|Task3"
    action: disable
```

#### Empty Option

```
[privacy.yaml] Tweak 'my_tweak': option 'Enabled' has no changes (registry, service, scheduler, or commands)
```

**Fix:** Each option must do something. Add at least one of:
- `registry_changes`
- `service_changes`
- `scheduler_changes`
- `pre_commands` / `post_commands`
- `pre_powershell` / `post_powershell`

#### Invalid Tweak ID Format

```
[privacy.yaml] Tweak 'MyTweak': tweak ID must be snake_case (lowercase letters, digits, underscores only)
```

**Fix:** Use snake_case for tweak IDs:

```yaml
# Wrong
- id: MyTweak
- id: my-tweak
- id: myTweak123

# Correct
- id: my_tweak
- id: disable_telemetry
- id: set_pagefile_size_4gb
```

#### Duplicate Option Labels

```
[privacy.yaml] Tweak 'my_tweak': duplicate option label 'Disabled' (case-insensitive)
```

**Fix:** Each option label must be unique within a tweak:

```yaml
# Wrong
options:
  - label: "Disabled"
    ...
  - label: "disabled"  # Duplicate (case-insensitive)!

# Correct
options:
  - label: "Disabled"
    ...
  - label: "Enabled"
```

#### Duplicate Category IDs

```
[my_tweaks.yaml] Duplicate category ID 'privacy' (already defined in privacy.yaml)
```

**Fix:** Each YAML file must define a unique category ID:

```yaml
# In my_tweaks.yaml - wrong if privacy.yaml already uses 'privacy'
category:
  id: privacy  # Duplicate!

# Correct - use a unique ID
category:
  id: my_custom_privacy
```

### Common Warnings

Warnings don't fail the build but indicate potential issues:

#### HKLM Without Admin

```
⚠ [privacy.yaml] Tweak 'my_tweak': contains HKLM registry changes but requires_admin is false (should be true)
```

**Fix:** HKLM changes require admin privileges:

```yaml
- id: my_tweak
  requires_admin: true  # Add this!
  options:
    - label: "Enabled"
      registry_changes:
        - hive: HKLM  # This requires admin
          key: "SOFTWARE\\..."
```

#### Empty Value Name

```
⚠ [ui.yaml] Tweak 'my_tweak': option 'Enabled' registry change '': value_name is empty (targeting default value)
```

This is a warning because targeting the default value (`(Default)`) is sometimes intentional. If not intentional, specify a value name:

```yaml
registry_changes:
  - hive: HKCU
    key: "Software\\MyApp"
    value_name: "MySetting"  # Not empty
    value_type: "REG_SZ"
    value: "my value"
```

---

## Testing Your Tweaks

### 1. Validate YAML Syntax

```powershell
# The build will fail if YAML is invalid
cd src-tauri
cargo build
```

### 2. Check in Dev Mode

```powershell
bun run dev
# Navigate to your category and test:
# - Can you see the tweak?
# - Does the toggle/dropdown work?
# - Does status detection work?
```

### 3. Test Apply and Revert

1. Apply the tweak
2. Verify changes took effect (check registry, services, etc.)
3. Revert the tweak
4. Verify original state is restored

### 4. Test Error Cases

1. What if a registry key doesn't exist?
2. What if a service is protected?
3. What happens with wrong permissions?

### 5. Test on Clean System

Some changes only work on fresh installs or after updates.

---

## Troubleshooting

### Tweak Doesn't Appear

1. Check YAML syntax (compile errors in terminal)
2. Verify `id` is unique across all files
3. Check `windows_versions` filter isn't excluding your version
4. Rebuild: `cargo build`

### Status Detection Wrong

1. Check if values match exactly (case-sensitive for strings)
2. Verify `skip_validation` is used appropriately
3. Check registry key path is correct (use regedit to verify)
4. Test with debug mode enabled

### Tweak Fails to Apply

1. Check `requires_admin` is set correctly
2. Try `requires_system: true` for protected resources
3. Check Windows Event Viewer for errors
4. Enable debug mode for detailed logs

### Revert Doesn't Work

1. Check if snapshot was captured (look in `snapshots/` folder)
2. Verify original state was readable
3. Some changes (like `delete` scheduler action) can't be reverted

### Service Won't Stay Disabled

1. Use `skip_validation: true` for self-healing services
2. Consider additional registry changes to prevent re-enabling
3. Some services are protected by Windows and will always re-enable

---

## Appendix: Value Type Reference

| YAML Type | Registry Type | Example YAML           | Notes                       |
| --------- | ------------- | ---------------------- | --------------------------- |
| Integer   | REG_DWORD     | `value: 1`             | 32-bit, 0 to 4294967295     |
| Integer   | REG_QWORD     | `value: 9876543210`    | 64-bit                      |
| String    | REG_SZ        | `value: "text"`        | Plain string                |
| String    | REG_EXPAND_SZ | `value: "%PATH%"`      | Expandable environment vars |
| Array     | REG_BINARY    | `value: [0, 255, 128]` | Byte array                  |
| Null      | (delete)      | `value: null`          | Deletes the value           |

---

## Appendix: Service Name Reference

Common service names for tweaks:

| Service Name       | Display Name               | Notes             |
| ------------------ | -------------------------- | ----------------- |
| `DiagTrack`        | Connected User Experiences | Telemetry         |
| `dmwappushservice` | WAP Push Message           | Win10 only        |
| `wuauserv`         | Windows Update             | Self-healing      |
| `WaaSMedicSvc`     | Windows Update Medic       | Repairs WU        |
| `UsoSvc`           | Update Orchestrator        | Schedules updates |
| `Spooler`          | Print Spooler              | Printing          |
| `SysMain`          | SysMain                    | Superfetch        |
| `WSearch`          | Windows Search             | Indexing          |

Use `sc query state= all` to list all services.

---

## Appendix: Scheduler Path Reference

Common task paths:

| Path                                                         | Contains             |
| ------------------------------------------------------------ | -------------------- |
| `\Microsoft\Windows\Customer Experience Improvement Program` | Telemetry tasks      |
| `\Microsoft\Windows\Application Experience`                  | Compatibility tasks  |
| `\Microsoft\Windows\UpdateOrchestrator`                      | Windows Update tasks |
| `\Microsoft\Windows\Windows Defender`                        | Defender tasks       |
| `\Microsoft\Windows\WindowsUpdate`                           | Legacy update tasks  |

Use Task Scheduler (`taskschd.msc`) to explore tasks.

---

*Last updated: December 2024*
*This document should be updated whenever the tweak system changes.*
