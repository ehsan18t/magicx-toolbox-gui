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

**Store Structure:**
```
src/lib/stores/
├── index.ts              # Barrel export for all stores
├── theme.svelte.ts       # Theme management (light/dark/system)
├── modal.svelte.ts       # Modal state (about/settings/update)
├── layout.svelte.ts      # Sidebar expanded/pinned state (sidebarStore)
├── colorScheme.svelte.ts # Accent color scheme selection
├── settings.svelte.ts    # App settings with localStorage persistence
├── debug.svelte.ts       # Debug panel and logging state
├── navigation.svelte.ts  # Tab navigation state
├── update.svelte.ts      # Update checking state
├── systemElevation.svelte.ts # SYSTEM elevation mode
├── tweakDetailsModal.svelte.ts # Tweak details modal state
└── tweaks.svelte.ts      # Barrel export for tweaks system
    ├── tweaksData.svelte.ts    # System info, categories, tweaks list
    ├── tweaksLoading.svelte.ts # Loading/error state with SvelteSet/SvelteMap
    ├── tweaksPending.svelte.ts # Pending changes and reboot tracking
    └── tweaksActions.svelte.ts # Apply, revert, toggle actions
```

**Available stores:**
- `themeStore` - Theme management (light/dark/system)
- `modalStore` - Modal state (about/settings/update)
- `sidebarStore` - Sidebar expanded/pinned state
- `colorSchemeStore` - Accent color scheme selection
- `settingsStore` - App settings with localStorage persistence
- `debugState` - Debug panel and logging state
- `navigationStore` - Tab navigation with navigateToTab(), navigateToCategory()
- `updateStore` - Update info and checking state
- `systemElevationStore` - SYSTEM elevation mode
- `tweakDetailsModalStore` - Tweak details modal state

**Tweaks system stores:**
- `systemStore` - Windows system info (.info getter)
- `categoriesStore` - Category definitions (.list, .map)
- `tweaksStore` - Tweak definitions with status (.list, .byCategory, .stats)
- `loadingStore` - Per-tweak loading state (SvelteSet-based)
- `errorStore` - Per-tweak error messages (SvelteMap-based)
- `pendingChangesStore` - Staged changes before apply (SvelteMap-based)
- `pendingRebootStore` - Tweaks requiring reboot (SvelteSet-based)
- `filterStore` - Search and filter state

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

> The tweak engine was rebuilt around a single typed representation. This section summarizes it;
> [TWEAK_SYSTEM.md](./TWEAK_SYSTEM.md) is the full architecture reference and
> [TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md) is the authoring guide.

#### 1. Effect-centric tweaks
- **One managed surface**: a tweak declares its `effects:` (registry value/key, service, task, hosts,
  firewall, shared, action) once; each **option** is a flat value-map over that surface.
- **Computed statuses**: "System Default" is computed when the live surface matches no option; 1 option
  renders as a toggle, ≥2 as a dropdown. **Unknown** (unreadable) and per-option **unavailable** are also
  computed, never authored.
- **Windows scoping**: `windows: { products, build, revision }` at tweak/effect/option-value level.

#### 2. Typed effects (one representation)
- Apply, capture, detect, and revert all consume the *same* typed `Value`, so they cannot drift.
- Each `EffectKind` module co-locates read/apply/revert/detect and wraps the reused low-level primitives
  (registry `RegSetValueExW`, service SCM, scheduler COM, hosts, firewall).
- Reversibility and detectability are **typed**: Settings always; Actions iff they carry `undo`/`probe`.

#### 3. Snapshot history + WAL
- **Per-tweak history**: one atomically-written entry per capture, ordered by a monotonic sequence.
  Authored-option captures are stored as references (re-applied from the current corpus); unauthored
  states are value dumps.
- **WAL action journal** makes "an action ran but nothing recorded it" impossible to lose silently — it
  surfaces as **Needs Attention**.
- **A snapshot is deleted only** by a verified restore, the verified startup stale-cleanup, or explicit
  user consent — never on a failure path (ADR-0002).

#### 4. Configuration Profile System
- **Profile export**: Export applied tweaks as shareable `.mgx` archives
- **Profile import**: Import and validate profiles before applying
- **Validation**: Pre-apply validation with warnings/errors and change preview
- **System state capture**: Optional full system state snapshot for debugging
- **Rollback support**: Automatic rollback on partial apply failure
- See [PROFILE_SYSTEM.md](./PROFILE_SYSTEM.md) for complete documentation

#### 5. Elevation Model (ADR-0005)
- **Four declared levels**: `user` / `admin` / `system` / `ti`, author-declared, never inferred. A tweak
  declares a **floor**; an effect may escalate (`effective = max(floor, step)`), never lower.
- **User-provided**: the app ships unelevated; Admin comes from launching as admin or the in-app
  **Elevate** relaunch — never silently acquired. Privileged tweaks are disabled until the user elevates.
- **HKCU exception**: a user-hive effect always runs in-process as the interactive user, and a
  token-SID/session-SID mismatch disables User-level tweaks (over-the-shoulder guard).
- **Reads run at the current level**: TI-protected resources deny reads and report **Unknown** with a
  needs-elevation hint until the user elevates.

#### 6. Risk Levels
```yaml
risk_levels:
  low: Safe, no system impact
  medium: May affect system behavior
  high: Significant impact, changes important features
  critical: Can break system functionality
```

---

## Data Model

The tweak schema is **effect-centric** and defined by the compiled model in
`src-tauri/src/tweaks/model.rs`. A tweak declares its managed surface once (`effects:`) and each option
is a flat value-map over it. The full schema — every effect kind, value literal, presence/shared/version
semantics, and the build guards — is documented in **[TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md)**; the
one-representation model and lifecycle in **[TWEAK_SYSTEM.md](./TWEAK_SYSTEM.md)**.

```yaml
# See TWEAK_AUTHORING.md for the full schema; src-tauri/tweaks/examples.yaml is the reference corpus.
category: { id: ..., name: ..., icon: ..., description: ... }   # one category block per file
tweaks:
  - id: unique_tweak_id
    name: "Human Readable Name"
    description: "What this tweak does"
    risk_level: low | medium | high | critical
    elevation: user | admin | system | ti     # per-tweak floor (never requires_admin/_system/_ti)
    reversible: true | false                   # declared and build-checked against the computed value
    requires_reboot: false                     # optional
    effects:                                   # the managed surface, declared once
      - id: some_flag
        registry: { key: 'HKCU\Software\...', name: SomeValue, type: REG_DWORD }
    options:                                   # only the real states; "System Default" is computed
      - label: "On"
        values: { some_flag: 1 }
      - label: "Off"
        values: { some_flag: absent }          # `absent` is the only absence spelling
```

### Snapshot entry

Per-tweak history entries live under `snapshots/<tweak-id>/` next to the executable
(`src-tauri/src/tweaks/snapshot.rs`):

```rust
struct SnapshotEntry {
    schema_version: u32,
    machine_guid: Option<String>,   // MachineGuid stamp; a wrong-machine entry is invalid
    tweak_id: String,
    seq: u64,                       // monotonic per-tweak sequence (ordering; wall-clock is display only)
    timestamp: String,              // display metadata
    captured: Captured,             // OptionRef(label) for authored options, or Values(map) for dumps
    journal: Vec<(EffectId, ActionMark)>,   // WAL: intended → completed, per action
}
```

Shared-referenced effects appear in no per-tweak entry — their return path is the `shared_claims.json`
record (ADR-0006).

## Tweak Format Examples

Worked examples for every effect kind live in the reference corpus
[`src-tauri/tweaks/examples.yaml`](../src-tauri/tweaks/examples.yaml), and the schema is documented in
**[TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md)**. In the effect-centric model there is no fixed
change-list execution order: a tweak declares its `effects:` once, and applying an option **drives each
effect to its desired value in declaration order** (capture → persist snapshot + WAL → drive → verify
per effect), with atomic rollback on any failure (see [TWEAK_SYSTEM.md](./TWEAK_SYSTEM.md)).

---

## Backend Services

### 1. Tweak engine (`tweaks/`) - Loading & execution
- The compiled, build-time-validated corpus is embedded at build time; runtime access via
  `tweaks::compiled_corpus()`
- The engine (`tweaks/engine/`) owns the apply/detect/restore lifecycle; the `tweaks/kinds/` modules
  are the per-kind effect executors that wrap the reused low-level primitives below

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

### 5. Snapshot store + shared claims (`tweaks/snapshot.rs`, `tweaks/shared_claims.rs`)
- `SnapshotStore::open_default()` - per-tweak history in the portable `snapshots/` directory **next to
  the executable** (one subdirectory per tweak-id, one atomically-written file per entry)
- Entries are references (authored options) or value dumps (unauthored states), each carrying the WAL
  action journal; invalid/dangling entries are kept, excluded, and released only by user consent
- `shared_claims.json` (under the snapshots root) - the refcounted claims record: capture-once,
  last-release restores the captured original (ADR-0006)

### 6. `profile` - Configuration Profile Export/Import
- `export_profile()` - Export applied tweaks to .mgx archive
- `import_profile()` - Read and validate profile from archive
- `apply_profile()` - Apply validated profile to system
- `validate_profile()` - Validate profile against current system
- See [PROFILE_SYSTEM.md](./PROFILE_SYSTEM.md) for complete documentation

### 7. `trusted_installer` - SYSTEM Elevation & PowerShell
- Execute commands as SYSTEM via winlogon.exe token
- Registry writes as SYSTEM for protected keys
- PowerShell execution: `run_powershell()`, `run_powershell_as_system()`
- Schtasks execution: `run_schtasks_as_system()`

### 8. `system_info_service` - System Detection
- Windows version detection (10 vs 11)
- Build number detection
- Admin privilege check
- CPU/RAM information

---

## Commands (Tauri IPC)

### Tweak Operations (`commands/tweaks.rs`)
| Command                    | Description                                                              |
| -------------------------- | ------------------------------------------------------------------------ |
| `get_tweaks()`             | List all tweaks from the compiled model                                  |
| `get_statuses_stream()`    | Start the background scan; per-tweak statuses stream in as events        |
| `rescan_after_elevation()` | Full re-scan after the user elevates (Unknowns become readable)          |
| `apply_tweak(id, option)`  | Apply an option: capture snapshot + WAL, drive + verify, rollback on failure |
| `restore_tweak(id)`        | Restore the head snapshot entry (undo journal, then re-apply the target) |
| `get_elevation_state()`    | Current elevation level and SID-mismatch status                          |

### Snapshot Operations
| Command                         | Description                                                     |
| ------------------------------- | -------------------------------------------------------------- |
| `list_snapshot_entries(id)`     | List a tweak's snapshot history (valid and invalid entries)    |
| `discard_snapshot_entry(...)`   | Release an invalid/dangling entry by explicit user consent (ADR-0002) |

### Profile Operations
| Command              | Description                              |
| -------------------- | ---------------------------------------- |
| `profile_export()`   | Export applied tweaks to .mgx archive    |
| `profile_import()`   | Import and validate profile from archive |
| `profile_validate()` | Validate profile against current system  |
| `profile_apply()`    | Apply validated profile to system        |

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
1. Loads every `*.yaml` in the `tweaks/` directory (`schema::load_corpus`)
2. Runs the structural and semantic guards (spec §10) over each milestone of the support matrix —
   ownership, coverage, detectability, distinctness, reversibility honesty, path syntax, typed literals
3. Embeds the validated corpus as JSON (`OUT_DIR/corpus.json`)
4. A YAML mistake or a failed guard is a **compile error**; no runtime file I/O for tweak definitions

`build.rs` `#[path]`-includes the runtime's own `model`/`parse`/`schema`/`validate` modules, so
build-time and runtime validation are the same code — schema drift is a compile error.

### Generated Output
- The validated corpus embedded as JSON, deserialized once via `tweaks::compiled_corpus()`

---

## File Locations

| Path                                      | Purpose                                               |
| ----------------------------------------- | ----------------------------------------------------- |
| `src-tauri/tweaks/*.yaml`               | Tweak definitions (the example corpus; the full corpus is re-authored on `main`) |
| `src-tauri/src/tweaks/`                 | Tweak engine (model, schema, parse, validate, engine, kinds, snapshot, shared_claims) |
| `src-tauri/src/commands/tweaks.rs`      | Tauri command surface for tweaks                      |
| `src-tauri/src/services/`               | Reused low-level primitives + the elevation broker    |
| `src-tauri/src/models/`                 | Data structures                                       |
| `snapshots/` (next to the executable)   | Per-tweak snapshot history + `shared_claims.json`     |

### Tweak Engine Module Structure

```
src-tauri/src/tweaks/
├── model.rs          # Effect · Setting · ActionDef · Value · Tweak · Opt (the one representation)
├── schema.rs         # YAML DTOs → compiled model (build-time)
├── parse.rs          # typed literals, registry-path + build-expr grammars, kv_semicolon parser
├── validate.rs       # structural + semantic guards (spec §10), per support milestone
├── engine/           # apply · detect · revert · lifecycle
├── kinds/            # registry · service · task · hosts · firewall · action
├── snapshot.rs       # atomic per-tweak history
└── shared_claims.rs  # refcounted shared-claims record
```

---

## Security Considerations

1. **Elevation**: privileged tweaks are disabled until the user elevates; the app never silently escalates (ADR-0005)
2. **Snapshot integrity**: a snapshot is deleted only by a verified restore or explicit consent; invalid entries are kept and surfaced (ADR-0002)
3. **Atomic rollback**: any apply failure restores the captured state; an incomplete rollback surfaces as **Needs Attention**, never hidden (ADR-0001)
4. **No remote code**: All tweaks are compiled into the binary; no external downloads

---

## Known Limitations

1. **Service control requires admin**: Cannot modify service startup type without elevation
2. **Some registry keys protected**: Even with admin, some keys (SAM) may be inaccessible
3. **Windows version detection**: Based on registry, may not detect all insider builds

---

## Categories

The engine currently ships the **example corpus** only (`src-tauri/tweaks/examples.yaml`, category
`Examples`) — one tweak per feature, proving the build and engine end-to-end. The real category set is
re-authored from scratch, per category, on `main`, outside this plan (spec §12). Categories are declared
per file via the `category:` block (`id` / `name` / `icon` / `description`).
