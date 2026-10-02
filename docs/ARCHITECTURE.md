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
├── layout.svelte.ts      # Navigation pane collapsed/overlay state (sidebarStore)
├── colorScheme.svelte.ts # Accent color scheme selection
├── settings.svelte.ts    # App settings with localStorage persistence
├── logs.svelte.ts        # Logs panel lines, polling, logging settings and export
├── navigation.svelte.ts  # Tab navigation state
├── update.svelte.ts      # Update checking state
├── tweakDetailsModal.svelte.ts # Which tweak the details window shows
└── tweaks.svelte.ts      # Barrel export for tweaks system
    ├── tweaksData.svelte.ts    # System info, categories, tweaks list
    ├── tweaksLoading.svelte.ts # Loading/error state with SvelteSet/SvelteMap
    ├── tweaksPending.svelte.ts # Pending changes and reboot tracking
    └── tweaksActions.svelte.ts # Apply, revert, toggle actions
```

**Available stores:**
- `themeStore` - Theme management (light/dark/system)
- `modalStore` - Modal state (about/settings/update)
- `sidebarStore` - Navigation pane collapsed/overlay state
- `colorSchemeStore` - Accent color scheme selection
- `settingsStore` - App settings with localStorage persistence
- `logsStore` - Logs panel lines, polling, logging settings and export
- `navigationStore` - Tab navigation with navigateToTab(), navigateToCategory()
- `updateStore` - Update info and checking state
- `tweakDetailsModalStore` - Which tweak the details window shows (also the selected row)

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
├── ui/                   # Reusable primitives (Button, Badge, Card, Modal*, Select, SegmentedSwitch, FilterChips, ...)
├── tweaks/
│   ├── TweakRow.svelte          # One tweak as a full-width row: text, callouts, meta line, control
│   ├── TweakControl.svelte      # The option switch or dropdown, shared by the row and the details window
│   ├── TweakDetailsModal.svelte # Details window: header control, option comparison table, snapshot history
│   ├── SummaryPanel.svelte      # "At a glance" pane at 1400px+ content width
│   ├── GroupedTweakList.svelte  # Rows grouped by category (Favorites, Snapshots)
│   ├── AppRow.svelte            # One app item as a row
│   └── details/                 # Registry, service, scheduler, hosts, firewall change items; CommandList
├── views/                # Overview, Category, Search, Favorites, Snapshots, Settings, Manual Tests
├── layout/               # TitleBar (search box, Ctrl+K), Sidebar (navigation pane), LogsPanel, PageLayout
├── feedback/             # PendingBar (staged changes, Apply, Discard), RebootBanner, ApplyingOverlay, toasts
├── modals/               # About, Update, profile dialogs, ConfirmDialog
├── profile/              # ProfileManager
├── settings/             # ThemeToggle, ColorSchemePicker
└── shared/               # Icon, ExternalLink, MarkdownText
```

The navigation pane docks expanded at a window width of 1008px and above (the title bar toggle collapses it, and the choice is kept), shows icons only below that, and opens over the content when toggled there. The Logs panel docks under the content column, beside the navigation pane rather than under it. `src/routes/+page.svelte` renders it, and the error screens of `+page.svelte` and `+layout.svelte` render their own, so it stays reachable when loading fails. Design tokens (navy and slate neutrals, the Segoe UI Variable font, seven accent schemes per theme) live in `src/app.css`.

### Motion

Motion follows Windows 11 Fluent timing: entrances decelerate, exits accelerate and run one step faster, and nothing waits on an animation before taking input. Every timing value is a token in the `@theme static` block of `src/app.css`: durations (`fast` 100ms for hover and press, `normal` 150ms for small elements and toggles, `slow` 200ms for pages, dialogs and toasts, `slower` 300ms for progress values), delays (`reveal`, `settle`, `tooltip`, `feedback`), easing curves, travel distances and the z-index layers. Tailwind turns them into utilities such as `duration-normal`, `ease-out`, `animate-rise-in` and `z-modal`.

Use a CSS class (`animate-fade-in`, `animate-rise-in`, `animate-pop-in`, `animate-reveal`) for an element that only animates in. Use the presets in `src/lib/utils/motion.ts` (`fade`, `shift`, `pop`, `expand`, and `reflow` for `animate:`) when Svelte has to keep an element mounted for its exit or measure its height. The presets read the same tokens at runtime, and they honour the Windows "Animation effects" setting (`prefers-reduced-motion`), which Svelte's own transitions would otherwise ignore. A timer that has to match an animation reads `duration()` or `delay()` from that module, never a number.

Lists that change on every keystroke or status rescan (tweak rows, search results, log lines) never animate per item; their container fades in once when it first appears. A control repeated on every row pays nothing at mount: the segmented switch draws its selection as a CSS pseudo-element, and only the switch that changes runs `glide()` to slide it over. A per-row observer or anchor-positioned thumb measured about 75% slower page mounts.

### Browser preview

`pnpm dev` and then `/?preview` (as administrator) or `/?preview&user` (as a standard user) runs the UI in a plain browser against mocked IPC (`src/lib/preview/`, loaded by `src/hooks.client.ts` in dev builds only). Machine state comes from `fixtures.ts`. The tweaks, apps and categories come from `corpus.json`, a subset of the catalogue generated by the real command projections. After changing one of those tweaks, regenerate it with `cargo test dump_preview_corpus -- --ignored` in `src-tauri`; `preview_corpus_matches_the_catalogue` fails `cargo test` while it is stale.

---

## Backend Architecture

### Core Features

> The tweak engine was rebuilt around a single typed representation. This section summarizes it;
> [architecture/tweak/](./architecture/tweak/README.md) is the full architecture reference and
> [TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md) is the authoring guide.

#### 1. Effect-centric tweaks
- **One managed surface**: a tweak declares its `effects:` (registry value/key, service, task, hosts,
  firewall, shared, action) once; each **option** is a flat value-map over that surface.
- **Computed statuses**: "System Default" is computed when the live surface matches no option; 1 option
  renders as a System default | option switch, 2 as a segmented switch, 3 or more as a dropdown. **Unknown** (unreadable) and per-option **unavailable** are also
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
- **WAL action journal** makes "an action ran but nothing recorded it" impossible to lose silently: it
  surfaces as **Needs Attention**.
- **A snapshot is deleted only** by a verified restore, a verified rollback of the entry just pushed, a dedup that supersedes a settled entry, or explicit user consent, never on a failure path (ADR-0002). No startup stale-cleanup is implemented.

#### 4. Configuration Profile System
- **Profile export**: Export applied tweaks as shareable `.mgx` archives
- **Profile import**: Import and validate profiles before applying
- **Validation**: Pre-apply validation with warnings/errors and change preview
- **System state capture**: Optional full system state snapshot for debugging
- **Rollback support**: Automatic rollback on partial apply failure
- See [PROFILE_SYSTEM.md](./PROFILE_SYSTEM.md) for complete documentation

#### 5. Elevation Model (ADR-0005)
- **Three declared levels**: `user` / `admin` / `ti`, author-declared, never inferred. A tweak
  declares a **floor**; an effect may escalate (`effective = max(floor, step)`), never lower.
- **User-provided**: the app ships unelevated; Admin comes from launching as admin or the in-app
  **Elevate** relaunch, never silently acquired. Privileged tweaks are disabled until the user elevates.
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
is a flat value-map over it. The full schema (every effect kind, value literal, presence/shared/version
semantics, and the build guards) is documented in **[TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md)**; the
one-representation model and lifecycle in **[architecture/tweak/](./architecture/tweak/README.md)**.

```yaml
# See TWEAK_AUTHORING.md for the full schema; the eight files in src-tauri/tweaks/ are the corpus.
category: { id: ..., name: ..., icon: ..., description: ... }   # one category block per file
tweaks:
  - id: unique_tweak_id
    name: "Human Readable Name"
    description: "What this tweak does"
    risk_level: low | medium | high | critical
    elevation: user | admin | ti              # per-tweak floor; there are no requires_* flags
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

Shared-referenced effects appear in no per-tweak entry; their return path is the per-machine
`shared_claims.<MachineGuid>.json` record (ADR-0006).

## Tweak Format Examples

Worked examples for every effect kind live in
**[TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md)** (§17), which also documents the full schema. The
shipping corpus is the nine category files in [`src-tauri/tweaks/`](../src-tauri/tweaks/). In the effect-centric model there is no fixed
change-list execution order: a tweak declares its `effects:` once, and applying an option **drives each
effect to its desired value in declaration order** (capture → persist snapshot + WAL → drive → verify
per effect), with atomic rollback on any failure (see [apply-and-restore.md](./architecture/tweak/apply-and-restore.md)).

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
- Enable or disable scheduled tasks; it never creates or deletes them
- Query task state (Ready, Disabled, Running, NotFound)
- Uses the Task Scheduler COM API (`ITaskService`)

### 5. Snapshot store + shared claims (`tweaks/snapshot.rs`, `tweaks/shared_claims.rs`)
- `SnapshotStore::open_default()` - per-tweak history in the portable `snapshots/` directory **next to
  the executable** (one subdirectory per tweak-id, one atomically-written file per entry)
- Entries are references (authored options) or value dumps (unauthored states), each carrying the WAL
  action journal and stamped with the machine guid and the capturing user's SID; invalid entries
  (another machine, another account's HKCU capture, dangling) are kept, excluded, and released only by
  user consent
- `shared_claims.<MachineGuid>.json` (under the snapshots root, one per machine) - the refcounted
  claims record: capture-once, last-release restores the captured original (ADR-0006)

### 6. `profile` - Configuration Profile Export/Import
- `export_profile()` - Export applied tweaks to .mgx archive
- `import_profile()` - Read and validate profile from archive
- `apply_profile()` - Apply validated profile to system
- `validate_profile()` - Validate profile against current system
- See [PROFILE_SYSTEM.md](./PROFILE_SYSTEM.md) for complete documentation

### 7. `elevation` - TrustedInstaller privilege
- `run_ops(level, ops)` is the only entry point: a batch of typed `BrokerOp`s run in one elevated
  child, which is this same binary re-spawned with `--broker`
- TrustedInstaller comes from starting its service and spoofing that process as the child's parent,
  after verifying the opened process really is TrustedInstaller (the service stops when idle, so its
  pid can be recycled between the SCM's answer and the handle we use)
- Every op is a typed effect (registry value or key, service startup type, scheduled task). There is
  no "run this string" op, so nothing the child can be asked to do is an interpreter
- The request crosses as a file, created through `exclusive_temp` so no other process running as the
  same user can rewrite it before the elevated child reads it

### 8. `system_info_service` - System Detection
- Windows version detection (10 vs 11)
- Build number detection
- Admin privilege check
- CPU/RAM information

### 9. `logging` - On-device logger
- One `log::Log` implementation installed first thing in `run()`: every line is redacted before it reaches the in-memory session buffer (2000 lines, read by the Logs panel), the session file, or an export
- Session files in `%LOCALAPPDATA%\me.ehsankhan.magicx-toolbox\logs`, size-capped and pruned to the newest 10 within 10 MiB; "Save logs on this PC" off stops disk writes only (ADR-0010)
- The TrustedInstaller child's log lines come back inside its response and are re-logged by the app under the `helper` source
- See [architecture/logging.md](./architecture/logging.md) for the full reference

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

### Logging Operations (`commands/logging.rs`)
| Command                               | Description                                                        |
| ------------------------------------- | ------------------------------------------------------------------ |
| `get_log_tail(since)`                 | Session buffer lines after `since`, plus how many were evicted     |
| `log_frontend(level, message)`        | Record an interface error in the session log                       |
| `get_log_settings()`                  | Saving, Detailed, logs folder, file count and size, any problem    |
| `set_log_settings(persist, detailed)` | Save and apply the logging settings; returns the effective state   |
| `export_diagnostics()`                | Write one redacted diagnostics text file where the user chooses    |
| `reveal_last_export()`                | Show the last exported file in Explorer                            |
| `open_log_folder()`                   | Open the logs folder                                               |
| `delete_logs()`                       | Delete the saved session files no running process is writing      |

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
2. Runs the structural and semantic guards (spec §10) over each milestone of the support matrix:
   ownership, coverage, detectability, distinctness, reversibility honesty, path syntax, typed literals
3. Embeds the validated corpus as JSON (`OUT_DIR/corpus.json`)
4. A YAML mistake or a failed guard is a **compile error**; no runtime file I/O for tweak definitions

`build.rs` `#[path]`-includes the runtime's own `model`/`parse`/`schema`/`validate` modules, so
build-time and runtime validation are the same code, so schema drift is a compile error.

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
| `snapshots/` (next to the executable)   | Per-tweak snapshot history + per-machine claims files |
| `src-tauri/src/logging/`                | On-device logger, redaction, session files, logging settings |
| `%LOCALAPPDATA%\me.ehsankhan.magicx-toolbox\logs` | Session log files (`magicx-YYYYMMDD-HHMMSS-<pid>.log`); `logging.json` sits one folder up |

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
5. **Local logs**: logs never leave the PC on their own; personal details are redacted before any line is stored, and the user exports a diagnostics file deliberately (ADR-0010)

---

## Known Limitations

1. **Service control requires admin**: Cannot modify service startup type without elevation
2. **Some registry keys protected**: Even with admin, some keys (SAM) may be inaccessible
3. **Windows version detection**: Based on registry, may not detect all insider builds

---

## Categories

The engine ships nine category files in `src-tauri/tweaks/`: `ai`, `debloat`, `interface`, `network`,
`performance`, `privacy`, `security`, `services`, `windows_update`. Each declares its category once via the `category:`
block (`id` / `name` / `icon` / `description`), and every tweak in the file inherits it. The
demonstration corpus that carried the engine through the redesign (`examples.yaml`) was removed once
the real categories landed.
