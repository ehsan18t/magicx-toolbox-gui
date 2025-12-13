# Copilot Instructions - Common

**CRITICAL: These instructions are mandatory and must be followed on every task. Ignoring them will result in suboptimal code quality and project standards violations.**

## Documentation Priority

**HIGHEST PRIORITY**: Whenever the tweak system is modified (execution order, error handling, atomicity, new fields, behavior changes, etc.), you **MUST** update `TWEAK_AUTHORING.md` immediately. This is the definitive guide for tweak authors and must always reflect the current implementation.

Changes that require documentation updates:

- Adding new fields to registry_changes, service_changes, scheduler_changes
- Modifying execution order or atomicity behavior
- Changing error handling (fatal vs. non-fatal)
- Adding new change types or actions
- Modifying state detection logic
- Changing snapshot/revert behavior

## Commits

- **Commit by TASK, not by FILE.** One task may involve multiple files.
- **Do NOT group unrelated changes.** If you have fixed a bug and added a feature, commit them separately.
- **Partially stage files if needed.** If a file contains changes for two different tasks, select only the relevant lines/hunks for the current commit.
- Use clear messages, e.g., `feat(ui): add settings drawer toggle` or `fix(theme): persist system preference on init`.
- Always commit after each logical change; avoid large uncommitted work.

## Dependency Selection Guidelines

**MANDATORY: Follow this process when adding ANY new package (npm) or crate (cargo).**

### Before Adding a Dependency

1. **Identify the need clearly** - What specific functionality is required?
2. **Research alternatives** - Always compare at least 3 options when available
3. **Evaluate each option** using these criteria:

| Criteria          | Priority | Description                                                           |
| ----------------- | -------- | --------------------------------------------------------------------- |
| **Maintenance**   | Critical | Last update <6 months, active issue resolution, responsive maintainer |
| **Stability**     | Critical | Major version 1.0+, minimal breaking changes, semantic versioning     |
| **Performance**   | High     | Benchmarks, memory footprint, startup impact                          |
| **Bundle Size**   | High     | Tree-shakeable, minimal transitive dependencies                       |
| **Security**      | High     | No known CVEs, audit history, trusted maintainers                     |
| **Compatibility** | High     | Works with current Rust/Node versions, platform support (Windows)     |
| **Documentation** | Medium   | API docs, examples, TypeScript types (for npm)                        |
| **Community**     | Medium   | GitHub stars, download count, Stack Overflow presence                 |

### Decision Matrix Template

When proposing a new dependency, document:

```
## Dependency Proposal: [purpose]

### Options Evaluated:
1. [Package A] - [version]
   - ✅ Pros: ...
   - ❌ Cons: ...
   - 📊 Last updated: [date], Downloads: [count], Size: [KB]

2. [Package B] - [version]
   - ✅ Pros: ...
   - ❌ Cons: ...
   - 📊 Last updated: [date], Downloads: [count], Size: [KB]

### Recommendation: [Package X] because [reasons]
```

### Red Flags - Avoid Dependencies That:

- Haven't been updated in >12 months (unless feature-complete like `lodash`)
- Have unresolved security vulnerabilities
- Pull in excessive transitive dependencies
- Are maintained by a single person with no backup maintainers
- Have license incompatibilities (prefer MIT, Apache-2.0, BSD)
- Are pre-1.0 without a clear roadmap (for production use)

### Project Goals Alignment

All dependencies must support our core goals:

- **Best Performance**: Minimal runtime overhead, efficient algorithms
- **Maximum Stability**: Battle-tested, widely used, backward compatible
- **Small Bundle**: Prefer focused libraries over kitchen-sink frameworks

### Current Approved Dependencies

See `package.json` (frontend) and `src-tauri/Cargo.toml` (backend) for the current vetted dependency list. Any additions must go through the evaluation process above.

# Copilot Instructions – Svelte Frontend

**MANDATORY: Follow all rules in this section for every frontend change.**

## Scope

- Applies to all files under `src/`, including Svelte components, routes, styles, and config that affect the frontend build.

## Expectations

- Prefer Svelte 5 runes APIs; avoid legacy `export let` unless interacting with legacy components.
- Keep components SSR-safe, but remember this project uses `adapter-static` with `prerender=true` and `ssr=false`.
- Use Tailwind CSS (v4) utility classes; avoid inline styles unless necessary.
- Keep aliases consistent: `@/*` maps to `src/*` and `$lib` for library exports.
- For external links in the UI, use the `ExternalLink` component (it opens via Tauri shell).
- Respect window drag regions: elements inside the title bar that need interaction must have `drag-disable`.

## Store Pattern (Svelte 5 Runes)

Stores are implemented using Svelte 5 runes (`.svelte.ts` files) with getter-based access:

```typescript
// Store definition (e.g., theme.svelte.ts)
let currentTheme = $state<Theme>("system");

export const themeStore = {
  get current() { return currentTheme; },
  get isDark() { return currentTheme === "dark"; },
  set(theme: Theme) { currentTheme = theme; },
  toggle() { /* ... */ }
};

// Component usage - NO $ prefix, use getter directly
<script lang="ts">
  import { themeStore } from "$lib/stores/theme.svelte";

  // Access via getter (reactive automatically)
  const isDark = $derived(themeStore.isDark);
</script>

{#if themeStore.current === "dark"}
  <Icon icon="moon" />
{/if}
```

**Store Structure:**

```
src/lib/stores/
├── index.ts              # Barrel export for all stores
├── theme.svelte.ts       # Theme management (light/dark/system)
├── modal.svelte.ts       # Modal state (about/settings/update)
├── layout.svelte.ts      # Sidebar state (sidebarStore)
├── colorScheme.svelte.ts # Accent color scheme selection
├── settings.svelte.ts    # App settings with localStorage persistence
├── debug.svelte.ts       # Debug panel and logging state
├── navigation.svelte.ts  # Tab navigation state
├── update.svelte.ts      # Update checking state
├── systemElevation.svelte.ts # SYSTEM elevation mode
├── tweakDetailsModal.svelte.ts # Tweak details modal state
└── tweaks.svelte.ts      # Barrel for tweaks system
    ├── tweaksData.svelte.ts    # systemStore, categoriesStore, tweaksStore
    ├── tweaksLoading.svelte.ts # loadingStore, errorStore (SvelteSet/SvelteMap)
    ├── tweaksPending.svelte.ts # pendingChangesStore, pendingRebootStore
    └── tweaksActions.svelte.ts # filterStore, apply/revert/toggle actions
```

**Available stores:**

- `themeStore` - Theme management (light/dark/system)
- `modalStore` - Modal state (about/settings/update)
- `sidebarStore` - Sidebar expanded/pinned state
- `colorSchemeStore` - Accent color scheme
- `settingsStore` - App settings with localStorage persistence
- `debugState` - Debug panel and logging state
- `navigationStore` - Tab navigation with navigateToTab(), navigateToCategory()
- `updateStore` - Update info and checking state
- `systemElevationStore` - SYSTEM elevation mode
- `tweakDetailsModalStore` - Tweak details modal state

**Tweaks system stores (from tweaks.svelte.ts barrel):**

- `systemStore` - Windows system info (.info getter)
- `categoriesStore` - Category definitions (.list, .map getters)
- `tweaksStore` - Tweak list with status (.list, .byCategory, .stats getters)
- `loadingStore` - Per-tweak loading state (SvelteSet-based, .isLoading(), .isAnyLoading)
- `errorStore` - Per-tweak error messages (SvelteMap-based, .getError(), .hasAnyError)
- `pendingChangesStore` - Staged changes before apply (SvelteMap-based)
- `pendingRebootStore` - Tweaks requiring reboot (SvelteSet-based)
- `filterStore` - Search and filter state

**Using SvelteSet/SvelteMap for reactivity:**

```typescript
import { SvelteSet, SvelteMap } from "svelte/reactivity";

// Do NOT wrap with $state - they are already reactive
const loadingTweaks = new SvelteSet<string>();
const errors = new SvelteMap<string, string>();

export const loadingStore = {
  start(tweakId: string) { loadingTweaks.add(tweakId); },
  stop(tweakId: string) { loadingTweaks.delete(tweakId); },
  isLoading(tweakId: string) { return loadingTweaks.has(tweakId); },
  get isAnyLoading() { return loadingTweaks.size > 0; },
};
```

## UI Primitives

Use the UI primitives from `$lib/components/ui` for consistency:

```typescript
import { Button, Badge, Card, Modal, ModalHeader, ModalBody, ModalFooter,
         IconButton, Switch, Select, SearchInput, Spinner } from "$lib/components/ui";
```

**Guidelines:**

- Use `Button` with `variant` prop: `"primary"`, `"secondary"`, `"danger"`, `"ghost"`
- Use `Badge` for status indicators with `variant` prop
- Use `Card` for content containers
- Use `Modal` + `ModalHeader` + `ModalBody` + `ModalFooter` for dialogs
- Use `IconButton` for icon-only buttons with tooltips
- Use `Switch` for boolean toggles
- Use `Spinner` for loading states

## Tweak Details Sub-Components

Use sub-components from `$lib/components/tweak-details` when displaying tweak changes:

```typescript
import { RegistryChangeItem, ServiceChangeItem, SchedulerChangeItem, CommandList } from "$lib/components/tweak-details";
```

- `RegistryChangeItem` - Displays a single registry change with path, value, badges
- `ServiceChangeItem` - Displays a service change with name and startup type
- `SchedulerChangeItem` - Displays a scheduler task change with action
- `CommandList` - Displays pre/post commands or PowerShell scripts

## Build / run / test

- Preferred commands: `bun run dev` for local, `bun run validate` before commits (format, lint, check).
- Routing: SPA fallback is configured; don't add SSR-only patterns. Assets served from `static/`.
- Keep `vite.config.ts` define constants in sync with `APP_CONFIG`.

## Patterns to follow

- State: prefer `$state` and `$derived` where applicable; use `onMount` for browser-only work.
- Forms & events: use native form submit handlers and prevent default when calling Tauri commands.
- CSS: use Tailwind class order; rely on `@/app.css` variables for colors.
- Imports: use `$lib/index.ts` barrel exports where possible.
- Accessibility: include labels/aria where needed; ensure focus styles remain visible; keyboard operability for title bar buttons.
- Performance: avoid heavy work in `onMount`; debounce expensive handlers; prefer CSS transitions over JS when possible.
- Drag regions: `drag-enable` for the bar, `drag-disable` for clickable controls.
- External resources: open external URLs through `ExternalLink`; avoid raw `window.open`.

## Tauri Command Integration

- Use `invoke()` from `@tauri-apps/api/core` to call Rust commands.
- Commands are defined in `src-tauri/src/commands/` and registered in `lib.rs`.
- Handle errors gracefully; Rust errors are returned as strings in the rejection.
- For tweak operations, use the API functions from `src/lib/api/tweaks.ts`.

## Avoid

- Adding new global styles outside `src/app.css` unless justified.
- Using direct `fetch` to local files; use Tauri commands via `@tauri-apps/api` when talking to backend.
- Adding new NPM dependencies without confirming compatibility with Vite/SvelteKit/Tauri.
- Blocking the UI with long-running calls; offload via Tauri commands instead.
- Using `$store` subscription syntax with rune-based stores (they don't have `subscribe`).

## Testing & linting

- Run `bun run validate` (format, lint, check) before committing frontend changes.

---

# Copilot Instructions – Rust (Tauri Backend)

**MANDATORY: Follow all rules in this section for every backend change.**

## Scope

- Applies to `src-tauri/` (Rust code, build.rs, tauri.conf.json) and Tauri capability configs.

## Project Architecture

The backend follows a layered architecture:

```
src-tauri/src/
├── commands/           # Tauri command handlers (thin layer calling services)
│   ├── backup.rs       # Backup management commands
│   ├── debug.rs        # Debug mode toggling
│   ├── elevation.rs    # SYSTEM elevation commands
│   ├── general.rs      # General app commands (greet, theme)
│   ├── system.rs       # System info commands
│   ├── update.rs       # Update commands
│   └── tweaks/         # Tweak commands (modular structure)
│       ├── mod.rs      # Module exports
│       ├── query.rs    # Status/listing (get_categories, get_tweak_status, etc.)
│       ├── apply.rs    # Single tweak ops (apply_tweak, revert_tweak)
│       ├── batch.rs    # Batch operations (batch_apply_tweaks, batch_revert_tweaks)
│       └── helpers.rs  # Internal utilities (registry/service/scheduler ops)
├── models/             # Data structures and types
│   ├── registry.rs     # Registry-related types
│   ├── system.rs       # System info types
│   └── tweak.rs        # Tweak definitions, categories, registry changes
├── services/           # Business logic layer
│   ├── backup_service.rs      # Backup creation/restoration
│   ├── registry_service.rs    # Windows registry operations
│   ├── scheduler_service.rs   # Windows Task Scheduler operations
│   ├── service_control.rs     # Windows service management
│   ├── system_info_service.rs # Windows version detection
│   ├── trusted_installer.rs   # SYSTEM elevation & PowerShell execution
│   └── tweak_loader.rs        # YAML tweak file discovery and parsing
├── lib.rs          # App entry, plugin setup, command registration
├── error.rs        # Custom error types with thiserror
├── state.rs        # AppState for shared mutable state
├── setup.rs        # App initialization hooks
└── debug.rs        # Debug logging utilities
```

## Expectations

- Target MSVC toolchain on Windows; avoid GNU-only crates. Keep dependencies compatible with Tauri 2.
- Commands live under `src-tauri/src/commands`; register them in `lib.rs` via `generate_handler!`.
- Use `AppState` for shared state; lock mutexes minimally and avoid long-held locks.
- Use `thiserror` for error types; propagate errors with `?` operator.
- Keep capability and permission changes in `src-tauri/capabilities/*.json` aligned with Tauri plugin usage.

## Logging System

The app uses `tauri-plugin-log` with level-based filtering:

- **Debug builds**: Console + Webview output, `Debug` level for dependencies, `Trace` for `app_lib`.
- **Release builds**: `Warn` level only (no debug noise in production).

### Logging Guidelines

```rust
// Use appropriate log levels:
log::trace!("Detailed execution flow");     // Very verbose, internal steps
log::debug!("Useful debugging info");       // Development debugging
log::info!("High-level events");            // Significant operations
log::warn!("Potential issues");             // Recoverable problems
log::error!("Errors that need attention");  // Failures
```

- Log command entry points with `log::info!`.
- Log registry operations with paths using `log::debug!` or `log::trace!`.
- Log errors before returning them.
- Never use `println!` or `eprintln!` – always use `log` crate macros.

## YAML Tweak System

Tweaks are defined in YAML files under `src-tauri/tweaks/`. Each file contains a category and array of tweaks.
All tweaks use a **unified option-based model** where each tweak has an `options` array.

### File Structure

```yaml
category:
  id: privacy # Unique category identifier
  name: "Privacy" # Display name
  description: "..." # Category description
  icon: "🔒" # Emoji icon
  order: 1 # Sort order in UI

tweaks:
  - id: disable_telemetry
    name: "Disable Telemetry"
    description: "..."
    risk_level: low # low | medium | high | critical
    requires_admin: true
    requires_system: false # Requires SYSTEM elevation for protected keys
    requires_ti: false # Requires TrustedInstaller (for WaaSMedicSvc, etc.)
    requires_reboot: false
    is_toggle: true # true = 2 options (toggle switch), false = dropdown
    info: "Optional documentation"

    options:
      - label: "Disabled" # First option (index 0)
        registry_changes:
          - hive: HKLM
            key: "SOFTWARE\\..."
            value_name: "AllowTelemetry"
            value_type: REG_DWORD
            value: 0
            windows_versions: [10, 11] # Optional filter
            skip_validation: false # Optional: if true, ignore this for status check & failure
        service_changes:
          - name: "DiagTrack"
            startup: disabled
            skip_validation: false # Optional: if true, ignore for status check & failure
        scheduler_changes:
          - task_path: "\\Microsoft\\Windows\\Application Experience"
            task_name: "Microsoft Compatibility Appraiser"
            action: disable # enable | disable | delete
            skip_validation: false # Optional: if true, ignore for status check & failure
            ignore_not_found: false # Optional: if true, ignore if task doesn't exist
          # Or use pattern matching for multiple tasks:
          - task_path: "\\Microsoft\\Windows\\UpdateOrchestrator"
            task_name_pattern: "USO|MusNotification|Reboot" # Regex pattern
            action: disable
            ignore_not_found: true # Some tasks may not exist on all systems
        pre_commands: [] # Shell commands before changes
        pre_powershell: [] # PowerShell before changes (after pre_commands)
        post_commands: [] # Shell commands after changes
        post_powershell: [] # PowerShell after changes (after post_commands)

      - label: "Enabled" # Second option (index 1)
        registry_changes:
          - hive: HKLM
            key: "SOFTWARE\\..."
            value_name: "AllowTelemetry"
            value_type: REG_DWORD
            value: 3
        service_changes:
          - name: "DiagTrack"
            startup: automatic
```

### skip_validation Flag

The `skip_validation` flag can be added to `registry_changes`, `service_changes`, and `scheduler_changes`. When `true`:

1. **Status Detection**: The item is excluded from tweak status checks (determining if a tweak is "applied" or not)
2. **Atomic Rollback**: Failures for this item won't trigger a full rollback
3. **Execution**: The change is still attempted, but failures are logged as warnings and execution continues

**Use case**: Windows Update service (`wuauserv`) re-enables itself when Settings app opens. With `skip_validation: true`, the tweak status won't flip to "not applied" just because this service changed.

### Execution Order

When applying an option, changes are executed in this order:

1. `pre_commands` (shell)
2. `pre_powershell` (PowerShell)
3. `registry_changes`
4. `service_changes`
5. `scheduler_changes`
6. `post_commands` (shell)
7. `post_powershell` (PowerShell)

### Adding New Tweaks

1. Add to existing category YAML file or create new file in `tweaks/`.
2. Use unique `id` across all tweaks.
3. Set appropriate `risk_level` based on impact.
4. Set `is_toggle: true` for 2-option tweaks, `false` for dropdowns.
5. Use `windows_versions` on individual registry changes for version-specific behavior.
6. Test with both apply and revert operations.

### Tweak Loader Behavior

- YAML files are compiled at build time (no runtime parsing).
- Filters registry changes by current Windows version at runtime.
- Tweaks are embedded in the binary via `build.rs`.

## Registry Operations

All registry operations go through `registry_service.rs`:

```rust
// Reading values
registry_service::read_dword(&hive, &key, &value_name)?;
registry_service::read_string(&hive, &key, &value_name)?;
registry_service::read_binary(&hive, &key, &value_name)?;

// Writing values (checks admin for HKLM)
registry_service::set_dword(&hive, &key, &value_name, value)?;
registry_service::set_string(&hive, &key, &value_name, value)?;
registry_service::set_binary(&hive, &key, &value_name, &bytes)?;
```

## Scheduler Operations

Scheduled task management via `scheduler_service.rs`:

```rust
scheduler_service::get_task_state(task_path, task_name)?;
scheduler_service::enable_task(task_path, task_name)?;
scheduler_service::disable_task(task_path, task_name)?;
scheduler_service::delete_task(task_path, task_name)?;
```

## PowerShell Execution

PowerShell commands via `trusted_installer.rs`:

```rust
// As current user
trusted_installer::run_powershell(script)?;

// As SYSTEM (for protected operations)
trusted_installer::run_powershell_as_system(script)?;
```

- HKLM writes require admin privileges; the service checks and returns `Error::RequiresAdmin`.
- All operations are logged at trace/debug level.

## Backup System

Before applying tweaks, the system creates JSON snapshots:

- Location: `snapshots/` directory next to executable (portable app design).
- Format: `{tweak_id}.json` containing original registry, service, and scheduled task states.
- Used for reverting tweaks to original state.
- Captures state BEFORE any changes are made for reliable rollback.

## Build / toolchain / env

- Use `rustup default stable-x86_64-pc-windows-msvc`. Ensure VS Build Tools with C++ workload installed.
- Verify WebView2 runtime exists on Windows.
- Commands: `cargo check` inside `src-tauri` for backend; `bun run validate` for full stack.

## Patterns to follow

- Use `tauri::Builder::default()` chaining; keep plugins initialization explicit in `lib.rs`.
- When adding new commands, define request structs with `serde::Deserialize` if more than 2 args.
- For blocking IO/CPU, use `tauri::async_runtime::spawn_blocking`.
- Align plugin usage with capabilities and update `tauri.conf.json` if window behavior changes.
- Keep error messages user-safe; avoid leaking file paths in user-facing strings.

## Adding New Commands

1. Create handler function in appropriate `commands/*.rs` file.
2. Use `#[tauri::command]` attribute.
3. Return `Result<T, Error>` using the custom error type.
4. Add logging at entry and for significant operations.
5. Register in `lib.rs` `generate_handler!` macro.

Example:

```rust
#[tauri::command]
pub fn my_command(arg: String) -> Result<ResponseType, Error> {
    log::info!("my_command called with: {}", arg);
    // ... implementation
    Ok(response)
}
```

## Avoid

- Blocking the main thread with long-running work.
- Introducing plugins without updating capabilities permissions.
- Using unstable Rust features.
- Writing to filesystem without validating paths.
- Using `println!`/`eprintln!` instead of `log` macros.
- Hardcoding paths; use relative paths from executable location.

## Testing & linting

- Run `cargo check` in `src-tauri` for type checking.
- Run `cargo clippy` for linting (configured in `clippy.toml`).
- Run `cargo fmt` for formatting (configured in `rustfmt.toml`).
- Full validation: `bun run validate` from project root.
