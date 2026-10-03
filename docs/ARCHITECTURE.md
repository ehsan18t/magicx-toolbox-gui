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

**Stores** (`src/lib/stores/`, one exported object per module, imported from the module itself):

| Module | Store | Holds |
| --- | --- | --- |
| `tweaksData` | `tweaksStore`, `categoriesStore` | The tweak model, categories, and live per-tweak statuses from the `tweak-status` stream |
| `tweakActions` | `tweakActionsStore` | Apply, restore and keep-current-state (single and batch), with per-tweak running and error state |
| `tweaksPending` | `pendingChangesStore`, `pendingRebootStore` | Staged changes, and tweaks waiting for a restart |
| `apps` | `appsStore` | App items, their presence, and Remove, Install and Get in Store (ADR-0009) |
| `boot` | `bootStore` | The launch sequence: the tweak model, the status stream, system info and elevation start together; app presence follows the model |
| `system` | `systemStore` | Windows and hardware info. The last good hardware read is cached across launches and painted at once under freshly read Windows fields; every launch then rereads the hardware (WMI) in the background and replaces the cache only when that read fully succeeds. With nothing cached, the card waits for the full read |
| `elevation` | `elevationStore` | The app's elevation ceiling, whether it runs as admin, and Restart as admin |
| `favorites` | `favoritesStore` | Starred tweak ids |
| `snapshotHistory` | `createSnapshotHistory()` | One details window's snapshot entries and discard |
| `profile` | `profileStore` | Profile export, import and apply |
| `navigation` | `navigationStore` | The current page or category |
| `search`, `pageFilter` | `searchStore`, `pageFilterStore` | Global fuzzy search, and the in-page filter of a list page |
| `detailsModal` | `tweakDetailsModalStore`, `appDetailsModalStore` | Which item the shared details window shows |
| `modal`, `confirm`, `toast` | `modalStore`, `confirmStore`, `toastStore` | The open dialog, the shared confirmation dialog, toasts (`toastStore.failure` logs and toasts an error) |
| `settings`, `theme`, `colorScheme`, `sidebar` | `settingsStore`, `themeStore`, `colorSchemeStore`, `sidebarStore` | Persisted preferences and the navigation pane state |
| `logs` | `logsStore` | The Logs panel, logging settings and diagnostics export |
| `diagnostics` (plain module) | `diagnosticsHeader`, `diagnosticsFacts`, `versionLabel` | The bug-report header and the facts About lists, read from the app info, system and elevation stores |
| `update`, `appInfo` | `updateStore`, `appInfoStore` | Update checks and installs, and the app's version facts |
| `manualTests` | `manualTestsStore` | The Manual Tests view (test build only) |

Tauri calls live in `src/lib/api/`, one module per command group. They are called from the stores, from the app shell (`App.svelte`, for system events) and from the logger (`utils/logger.ts`).

The types those calls send and receive (command arguments and results, event payloads, channel messages) are generated from the Rust serde types by ts-rs into `src/lib/types/generated/` whenever `cargo test` runs, and `src/lib/types/index.ts` re-exports them under their frontend names. After changing a serialized Rust type, run `pnpm run test:rust` (`cargo test` with the `test-build` feature, which the manual-test types need; `pnpm run validate` runs it before type-checking) and commit the regenerated files; CI fails when they are stale. The backend error shape (`{ code, message }`, hand-serialized in `error.rs`) is the exception: its codes stay listed by hand in `BACKEND_ERROR_CODES`.

### UI Components

Reusable primitives live in `$lib/components/ui/` and are exported from its barrel: buttons and links (`Button`, `IconButton`, `LinkButton`, `ExternalLink`), surfaces (`Card`, `SectionCard`, `PanelSection`, `Callout`, `CodeBlock`, `EmptyState`, `IconTile`, `SettingRow`), text (`PanelHeading`, `InlineCode`, `HighlightedText`, `MetaItem`), inputs (`Switch`, `SegmentedSwitch`, `Select`, `Checkbox`, `ToggleChip`, `SearchInput`, `TextField`, `TextArea`), dialogs (`Modal`, `ModalHeader`, `ModalTitle`, `ModalBody`, `ModalFooter`), and status (`Badge`, `Count`, `Dot`, `Meter`, `ProgressBar`, `ActivityBar`, `Spinner`, `Skeleton`, `SkeletonList`). Their class recipes are in `ui/variants.ts`, and the barrel exports the ones other folders style their own elements with (`button`, `card`, `rowButton`, `indicator`, `META_LINE` and the like). Import a folder through its barrel: ESLint rejects a deep path such as `$lib/components/ui/variants` from outside the folder. Class merging goes through `$lib/utils/cn` only (ESLint rejects importing `tailwind-merge` or `tailwind-variants` directly); its `cn` and `tv` mirror the custom theme names in `src/app.css`, which `cn.test.ts` enforces. Design data lives in `$lib/design`: the icon registry (`icons.ts`, where a new icon is registered), `ICON_SIZE` (`size.ts`), tone maps (`tone.ts`), shared surface classes (`surface.ts`) and the `HEADING` type scale (`type.ts`).

### Component Structure

```
src/lib/components/
├── ui/        # Primitives (see above)
├── items/     # Tweak and app rows: TweakRow, AppRow, TweakControl, GroupedTweakList, Restore buttons
│   └── details/   # Tweak and app details windows: change matrix, scripts, snapshot history
├── views/     # Overview, Category, Search, Favorites, Snapshots, Profiles, Settings, Manual Tests
│   └── settings/  # Settings-only controls: the accent colour picker
├── layout/    # TitleBar (search box, Ctrl+K), Sidebar (nav item, footer), LogsPanel, PageLayout, SummaryPanel, AppliedMeter
├── feedback/  # PendingBar, PendingReviewModal, RebootBanner, ApplyingOverlay, LoadError, NoMatches, toasts
├── modals/    # About, Update, ConfirmDialog, ConfirmHost (mounts the shared confirmation)
│   └── profile/   # Profile export and import dialogs
└── shared/    # Icon, MarkdownText
```

The navigation pane docks expanded at a window width of 1008px and above (the title bar toggle collapses it, and the choice is kept), shows icons only below that, and opens over the content when toggled there. If the interface fails before it mounts, `src/main.ts` replaces the loading screen with a plain-page error and a Close button (no Svelte or app CSS needed), and an outer `svelte:boundary` in `src/App.svelte` catches a failure in the title bar, overlay or toasts with a Retry and Close screen; both say inline when the window cannot close. The Logs panel docks under the content column, beside the navigation pane rather than under it. `src/Workspace.svelte` renders it, and the boot error screen in `src/App.svelte` renders its own, so it stays reachable when loading fails. Design tokens (navy and slate neutrals, the Segoe UI Variable font, seven accent schemes per theme) live in `src/app.css`.

### Motion

Motion follows Windows 11 Fluent timing: entrances decelerate, exits accelerate and run one step faster, and nothing waits on an animation before taking input. Every timing value is a token in the `@theme static` block of `src/app.css`: durations (`fast` 100ms for hover and press, `normal` 150ms for small elements and toggles, `slow` 200ms for pages, dialogs and toasts, `slower` 300ms for progress values), delays (`reveal`, `settle`, `tooltip`, `feedback`), easing curves, travel distances and the z-index layers. Tailwind turns them into utilities such as `duration-normal`, `ease-out`, `animate-rise-in` and `z-modal`.

Use a CSS class (`animate-fade-in`, `animate-rise-in`, `animate-pop-in`, `animate-reveal`) for an element that only animates in. Use the presets in `src/lib/utils/motion.ts` (`fade`, `shift`, `pop`, `expand`, and `reflow` for `animate:`) when Svelte has to keep an element mounted for its exit or measure its height. The presets read the same tokens at runtime, and they honour the Windows "Animation effects" setting (`prefers-reduced-motion`), which Svelte's own transitions would otherwise ignore. A timer that has to match an animation reads `duration()` or `delay()` from that module, never a number.

Lists that change on every keystroke or status rescan (tweak rows, search results, log lines) never animate per item; their container fades in once when it first appears. A control repeated on every row pays nothing at mount: the segmented switch draws its selection as a CSS pseudo-element, and only the switch that changes runs `glide()` to slide it over. A per-row observer or anchor-positioned thumb measured about 75% slower page mounts.

### Browser preview

`pnpm dev` and then `/?preview` (as administrator) or `/?preview&user` (as a standard user) runs the UI in a plain browser against mocked IPC (`src/lib/preview/`, loaded by `src/main.ts` in dev builds only). Machine state comes from `fixtures.ts`. The tweaks, apps and categories come from `corpus.json`, a subset of the catalogue generated by the real command projections. After changing one of those tweaks, regenerate it with `cargo test dump_preview_corpus -- --ignored` in `src-tauri`; `preview_corpus_matches_the_catalogue` fails `cargo test` while it is stale.

---

## Backend Architecture

### Core Features

> The tweak engine was rebuilt around a single typed representation. This section summarizes it; [architecture/tweak/](./architecture/tweak/README.md) is the full architecture reference and [TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md) is the authoring guide.

#### 1. Effect-centric tweaks
- **One managed surface**: a tweak declares its `effects:` (registry value/key, service, task, hosts, firewall, shared, action) once; each **option** is a flat value-map over that surface.
- **Computed statuses**: "System Default" is computed when the live surface matches no option; 1 option renders as a System default | option switch, 2 as a segmented switch, 3 or more as a dropdown. **Unknown** (unreadable) and per-option **unavailable** are also computed, never authored.
- **Windows scoping**: `windows: { products, build, revision }` at tweak/effect/option-value level.

#### 2. Typed effects (one representation)
- Apply, capture, detect, and revert all consume the *same* typed `Value`, so they cannot drift.
- Each `EffectKind` module co-locates read/apply/revert/detect and wraps the reused low-level primitives (registry `RegSetValueExW`, service SCM, scheduler COM, hosts, firewall).
- Reversibility and detectability are **typed**: Settings always; Actions iff they carry `undo`/`probe`.

#### 3. Snapshot history + WAL
- **Per-tweak history**: one atomically-written entry per capture, ordered by a monotonic sequence. Authored-option captures are stored as references (re-applied from the current corpus); unauthored states are value dumps.
- **WAL action journal** makes "an action ran but nothing recorded it" impossible to lose silently: it surfaces as **Needs Attention**.
- **A snapshot is deleted only** by a verified restore, a verified rollback of the entry just pushed, a dedup that supersedes a settled entry, or explicit user consent, never on a failure path (ADR-0002). No startup stale-cleanup is implemented.

#### 4. Configuration Profile System
- No backend yet: every profile call rejects (see [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) #4); the v1 archive format is kept in [spec/profile-v1.md](./spec/profile-v1.md) for the rebuild.

#### 5. Elevation Model (ADR-0005)
- **Three declared levels**: `user` / `admin` / `ti`, author-declared, never inferred. A tweak declares a **floor**; an effect may escalate (`effective = max(floor, step)`), never lower.
- **User-provided**: the app ships unelevated; Admin comes from launching as admin or the in-app **Elevate** relaunch, never silently acquired. Privileged tweaks are disabled until the user elevates.
- **HKCU exception**: a user-hive effect always runs in-process as the interactive user, and a token-SID/session-SID mismatch disables User-level tweaks (over-the-shoulder guard).
- **Reads run at the current level**: TI-protected resources deny reads and report **Unknown** with a needs-elevation hint until the user elevates.

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

The tweak schema is **effect-centric** and defined by the compiled model in `src-tauri/src/tweaks/model.rs`. A tweak declares its managed surface once (`effects:`) and each option is a flat value-map over it. The full schema (every effect kind, value literal, presence/shared/version semantics, and the build guards) is documented in **[TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md)**; the one-representation model and lifecycle in **[architecture/tweak/](./architecture/tweak/README.md)**.

```yaml
# See TWEAK_AUTHORING.md for the full schema; the nine files in src-tauri/tweaks/ are the corpus.
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

Per-tweak history entries live under `snapshots/<tweak-id>/` next to the executable (`src-tauri/src/tweaks/snapshot.rs`):

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

Shared-referenced effects appear in no per-tweak entry; their return path is the per-machine `shared_claims.<MachineGuid>.json` record (ADR-0006).

## Tweak Format Examples

Worked examples for every effect kind live in **[TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md)** (§17), which also documents the full schema. The shipping corpus is the nine category files in [`src-tauri/tweaks/`](../src-tauri/tweaks/). In the effect-centric model there is no fixed change-list execution order: a tweak declares its `effects:` once, and applying an option **drives each effect to its desired value in declaration order** (capture → persist snapshot + WAL → drive → verify per effect), with atomic rollback on any failure (see [apply-and-restore.md](./architecture/tweak/apply-and-restore.md)).

---

## Backend Services

### 1. Tweak engine (`tweaks/`) - Loading & execution
- The compiled, build-time-validated corpus is embedded at build time; runtime access via `tweaks::compiled_corpus()`
- The engine (`tweaks/engine/`) owns the apply/detect/restore lifecycle; the `tweaks/kinds/` modules are the per-kind effect executors that wrap the reused low-level primitives below

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
- `SnapshotStore::open_default()` - per-tweak history in the portable `snapshots/` directory **next to the executable** (one subdirectory per tweak-id, one atomically-written file per entry)
- Entries are references (authored options) or value dumps (unauthored states), each carrying the WAL action journal and stamped with the machine guid and the capturing user's SID; invalid entries (another machine, another account's HKCU capture, dangling) are kept, excluded, and released only by user consent
- `shared_claims.<MachineGuid>.json` (under the snapshots root, one per machine) - the refcounted claims record: capture-once, last-release restores the captured original (ADR-0006)

### 6. `profile` - Configuration Profile Export/Import
- No backend yet (see [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) #4 and [spec/profile-v1.md](./spec/profile-v1.md)).

### 7. `elevation` - TrustedInstaller privilege
- `run_ops(level, ops)` is the only entry point: a batch of typed `BrokerOp`s run in one elevated child, which is this same binary re-spawned with `--broker`
- TrustedInstaller comes from starting its service and spoofing that process as the child's parent, after verifying the opened process really is TrustedInstaller (the service stops when idle, so its pid can be recycled between the SCM's answer and the handle we use)
- Every op is a typed effect (registry value or key, service startup type, scheduled task). There is no "run this string" op, so nothing the child can be asked to do is an interpreter
- The request crosses as a file, created through `exclusive_temp` so no other process running as the same user can rewrite it before the elevated child reads it

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

### 10. Window state and taskbar (`window_state.rs`, `taskbar.rs`)
- The main window's size, position and maximized state persist through `tauri-plugin-window-state` (restored before the hidden window is shown, saved on exit and before a relaunch); the update download reports its progress in the Update modal and on the taskbar button.

### 11. Self-update (`commands/update.rs`)
- The app ships only as the portable `magicx-toolbox.exe`; `tauri.conf.json` has `bundle.active: false`, so `tauri build` emits the exe and no MSI or NSIS installer. A per-machine installer would put the exe in `Program Files`, where an unelevated app cannot write its `snapshots/` folder.
- `check_for_update` reads the GitHub release list of `RELEASE_REPO` (the one repository constant, also used to check download URLs) and offers the newest release (pre-releases only when the setting allows) whose assets include `magicx-toolbox.exe`, matched case-insensitively by name; a release without it is skipped. The name lives in one constant, `PORTABLE_ASSET`, in the backend.
- `install_update` takes the exit latch first (no apply may run while the exe is replaced), accepts only that asset from `https://github.com/ehsan18t/magicx-toolbox-gui/releases/download/<tag>/`, and refuses a release without GitHub's SHA-256 digest. It creates `<exe>.new` beside the running exe before downloading, so a folder the app cannot write to fails at once with `UPDATE_FOLDER_READ_ONLY` (the Update modal then links to the releases page). The download gets a time budget scaled to its size (at least 16 KiB/s, never under five minutes), so a slow connection still finishes. After the digest matches, it deletes a stale `<exe>.old` (one it cannot delete fails the update, naming the file), renames the running exe to `<exe>.old` (Windows allows renaming a running image), renames `<exe>.new` to the exe's name, saves the window state and starts the new exe with `--after-restart=<pid>` from its folder, then exits by itself. Both renames stay in one folder, so they are atomic on one volume; a single replace is not possible because Windows will not overwrite a running exe. Any failure discards `<exe>.new`, and any failure after the first rename moves `<exe>.old` back and returns an error; if even that fails, the error names the file to rename by hand.
- `remove_update_leftovers()` deletes `<exe>.old` and any interrupted `<exe>.new` once the new version's window has first shown, in one attempt; a failure is logged and retried on the following start. Until then the previous version stays beside it, so a release that cannot start is not a dead end: its startup failure dialog names `<exe>.old` and how to rename it back.
- The new version runs from the same folder, so it reads the same `snapshots/` history (ADR-0008).

---

## Commands (Tauri IPC)

Commands live in `src-tauri/src/commands/<group>.rs`, one module per group (tweaks, apps, system, logging, elevation, update, general, manual tests). The `generate_handler!` list in `src-tauri/src/lib.rs` is the complete set of registered commands; the frontend reaches them through the matching module in `src/lib/api/`. [architecture/tweak/commands-and-ui.md](./architecture/tweak/commands-and-ui.md) describes the tweak commands and their events.

---

## Error Handling

Every command returns `Result<T, Error>`, propagated to the frontend. The `Error` enum (`thiserror`) is in `src-tauri/src/error.rs`, which is the list of failure kinds.

---

## Build System

### `build.rs` - Compile-Time Processing
1. Loads every `*.yaml` in the `tweaks/` directory (`schema::load_corpus`)
2. Runs the structural and semantic guards over each milestone of the support matrix: ownership, coverage, detectability, distinctness, reversibility honesty, path syntax, typed literals
3. Embeds the validated corpus as JSON (`OUT_DIR/corpus.json`)
4. A YAML mistake or a failed guard is a **compile error**; no runtime file I/O for tweak definitions

`build.rs` `#[path]`-includes the runtime's own `model`/`parse`/`schema`/`validate` modules, so build-time and runtime validation are the same code, so schema drift is a compile error.

### Generated Output
- The validated corpus embedded as JSON, deserialized once via `tweaks::compiled_corpus()`

---

## File Locations

| Path                                      | Purpose                                               |
| ----------------------------------------- | ----------------------------------------------------- |
| `src-tauri/tweaks/*.yaml`               | Tweak and app definitions (one category per file) |
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
├── validate.rs       # structural + semantic guards, per support milestone
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
6. **Verified updates**: an update installs only `magicx-toolbox.exe` from this repository's release downloads, and only when its SHA-256 matches the digest GitHub publishes for it

---

## Known Limitations

1. **Service control requires admin**: Cannot modify service startup type without elevation
2. **Some registry keys protected**: Even with admin, some keys (SAM) may be inaccessible
3. **Windows version detection**: Based on registry, may not detect all insider builds

---

## Categories

The engine ships nine category files in `src-tauri/tweaks/`: `ai`, `debloat`, `interface`, `network`, `performance`, `privacy`, `security`, `services`, `windows_update`. Each declares its category once via the `category:` block (`id` / `name` / `icon` / `description`), and every tweak in the file inherits it. The demonstration corpus that carried the engine through the redesign (`examples.yaml`) was removed once the real categories landed.
