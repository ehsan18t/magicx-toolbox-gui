# Tweak System - Technical Architecture

> Internal reference for the unified option-based tweak system.
> For YAML authoring guide, see [TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md).

*Updated: 2026-03-19*

---

## Overview

Every tweak is an array of **options** (minimum 2). Each option defines the complete set of system changes for that state. The system detects which option matches current state, captures snapshots for rollback, and applies changes atomically where possible.

### Key Design Decisions

| Decision              | Choice                                   | Rationale                                                  |
| --------------------- | ---------------------------------------- | ---------------------------------------------------------- |
| Migration Strategy    | Big-bang rewrite                         | Types deeply intertwined                                   |
| Default Detection     | Match against options                    | No `is_default` flag; compare current state to all options |
| Unmatched State       | "System Default" placeholder             | Non-selectable indicator when no option matches            |
| Revert Behavior       | From snapshot                            | Restore to captured state before tweak was applied         |
| Option Identification | Array index                              | Stable array order in YAML                                 |
| Build-time validation | `#[serde(deny_unknown_fields)]`          | Catches YAML typos at compile time                         |
| Elevation hierarchy   | User → Admin → SYSTEM → TrustedInstaller | `requires_ti` implies system & admin                       |

---

## Architecture

### YAML → Binary Pipeline

```
tweaks/*.yaml
    ↓  (build.rs parses + validates at compile time)
    ↓  Mirror types with #[serde(deny_unknown_fields)]
OUT_DIR/tweaks.json + categories.json
    ↓  (include_str! embeds at compile time)
generated_tweaks.rs → LazyLock<HashMap<String, TweakDefinition>>
```

Build-time validation catches:
- Unknown fields (typos)
- Missing required fields
- Invalid value types vs declared registry types (e.g., string for REG_DWORD)
- DWORD/QWORD range overflow
- Invalid regex in `task_name_pattern`
- Mutual exclusivity (`task_name` XOR `task_name_pattern`)
- Empty options (no changes at all)
- Duplicate tweak/category IDs
- Firewall create without direction/action

### Change Types

Each option can contain any combination of:

| Type                | Applied By                              | Detected By           | Snapshot                     | Restore                  |
| ------------------- | --------------------------------------- | --------------------- | ---------------------------- | ------------------------ |
| `registry_changes`  | `registry_service` or SYSTEM elevation  | Read + compare values | Value + existed flag         | Set/delete value         |
| `service_changes`   | `service_control` or SYSTEM elevation   | Query startup type    | Startup type + running state | Set startup + start/stop |
| `scheduler_changes` | `scheduler_service` or SYSTEM elevation | Query task state      | Task state per name/pattern  | Enable/disable task      |
| `hosts_changes`     | `hosts_service` (file I/O)              | Check entry existence | Entry existed flag           | Add/remove entry         |
| `firewall_changes`  | `firewall_service` (netsh)              | Check rule existence  | Rule existed flag            | Delete rule (create N/A) |

### Execution Order

```
pre_commands (cmd.exe)
  → pre_powershell (PowerShell)
    → registry_changes (atomic with rollback)
      → service_changes (atomic with rollback)
        → scheduler_changes (atomic with rollback)
          → hosts_changes (atomic with rollback)
            → firewall_changes (atomic with rollback)
  → post_commands (cmd.exe)
    → post_powershell (PowerShell)
```

If any atomic change phase fails, all completed changes in that phase are rolled back.

### Elevation Hierarchy

```
requires_ti: true    → TrustedInstaller (parent process spoofing via TI service)
                       Implies requires_system and requires_admin
requires_system: true → SYSTEM (winlogon.exe token duplication)
                       Implies requires_admin
requires_admin: true  → Administrator (standard elevation)
```

Build.rs infers the hierarchy: if `requires_ti` is set, `requires_system` and `requires_admin` are automatically set to true.

### State Detection

Parallel (rayon) comparison of current system state against each option:

1. Filter out `skip_validation: true` items
2. Filter registry changes by `windows_versions`
3. Check registry, services, scheduler, hosts, firewall in parallel
4. Handle `*_missing_is_match` flags for Windows edition compatibility
5. Handle `ignore_not_found` for optional scheduled tasks
6. Handle `task_name_pattern` regex for bulk scheduler operations
7. First option where ALL validatable changes match = current state

If no option matches → "System Default" (unmatched state).

### Snapshot System

- **First apply**: Capture pre-change state → save as snapshot
- **Option switch**: Capture current state → apply atomically → rollback on failure
- **Revert**: Restore from original snapshot → delete snapshot
- **Stale detection**: On startup, validate all snapshots; remove if externally reverted

Snapshots include: registry values, service states, scheduler task states, hosts entries, firewall rules.

Storage: JSON files in app data directory with file locking (fs4).

### Profile System

- Export: ZIP archive of snapshot files + metadata
- Import: Validate against current tweak definitions, handle renamed tweaks via `aliases`
- `aliases` field on TweakDefinition maps old IDs to current ID for migration

---

## Module Map

### Models (`src-tauri/src/models/`)

| File                | Purpose                                                             |
| ------------------- | ------------------------------------------------------------------- |
| `tweak.rs`          | Core types: TweakDefinition, TweakOption, all change types, enums   |
| `tweak_snapshot.rs` | Snapshot types: Registry/Service/Scheduler/Hosts/Firewall snapshots |
| `inspection.rs`     | Inspection types: Mismatch details for UI display                   |

### Commands (`src-tauri/src/commands/tweaks/`)

| File         | Purpose                                                                         |
| ------------ | ------------------------------------------------------------------------------- |
| `apply.rs`   | `apply_tweak`, `revert_tweak` — orchestrates snapshot + apply + rollback        |
| `query.rs`   | `get_tweak_status`, `get_all_tweak_statuses` (parallel), `get_tweak_inspection` |
| `helpers.rs` | `apply_all_changes_atomically`, per-type apply functions                        |

### Backup (`src-tauri/src/services/backup/`)

| File            | Purpose                                                                                  |
| --------------- | ---------------------------------------------------------------------------------------- |
| `detection.rs`  | `detect_tweak_state`, `option_matches_current_state`, stale snapshot validation          |
| `capture.rs`    | `capture_snapshot`, `capture_current_state` (parallel)                                   |
| `restore.rs`    | `restore_from_snapshot` (atomic registry, best-effort services/scheduler/hosts/firewall) |
| `inspection.rs` | `inspect_tweak` — detailed mismatch report for UI                                        |
| `storage.rs`    | File I/O with locking                                                                    |
| `helpers.rs`    | Parsing utilities, value comparison                                                      |

### Services (`src-tauri/src/services/`)

| File                   | Purpose                                      |
| ---------------------- | -------------------------------------------- |
| `registry_service.rs`  | Windows Registry read/write operations       |
| `service_control.rs`   | Windows Service query/start/stop/set-startup |
| `scheduler_service.rs` | Task Scheduler query/enable/disable/delete   |
| `hosts_service.rs`     | Hosts file entry management                  |
| `firewall_service.rs`  | Firewall rule management via netsh           |
| `elevation/`           | SYSTEM and TrustedInstaller elevation        |

### Build (`src-tauri/build.rs`)

Mirror types of `tweak.rs` with `#[serde(deny_unknown_fields)]` for compile-time YAML validation. Must stay in sync — see comments at top of file.

---

## UI Behavior

Automatically determined by option count:
- **2 options** → Toggle switch (unless `force_dropdown: true`)
- **3+ options** → Dropdown/segmented control

"System Default" is shown as a non-selectable indicator when current state doesn't match any option. "Revert" button appears only when a snapshot exists.
