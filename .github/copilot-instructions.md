# Copilot Instructions - Common

## Commits

- Always commit related changes together.
- Keep commits task-scoped (one task can touch multiple files). Avoid dumping unrelated changes together.
- Use clear messages, e.g., `feat(ui): add settings drawer toggle` or `fix(theme): persist system preference on init`.
- Always commit after each logical change; avoid large uncommitted work.

# Copilot Instructions – Svelte Frontend

## Scope

- Applies to all files under `src/`, including Svelte components, routes, styles, and config that affect the frontend build.

## Expectations

- Prefer Svelte 5 runes APIs; avoid legacy `export let` unless interacting with legacy components.
- Keep components SSR-safe, but remember this project uses `adapter-static` with `prerender=true` and `ssr=false`.
- Use Tailwind CSS (v4) utility classes; avoid inline styles unless necessary.
- Keep aliases consistent: `@/*` maps to `src/*` and `$lib` for library exports.
- For external links in the UI, use the `ExternalLink` component (it opens via Tauri shell).
- Use `themeStore` from `$lib/stores/theme` for theme toggling; do not reimplement theme persistence.
- Respect window drag regions: elements inside the title bar that need interaction must have `drag-disable`.

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

## Testing & linting

- Run `bun run validate` (format, lint, check) before committing frontend changes.

---

# Copilot Instructions – Rust (Tauri Backend)

## Scope

- Applies to `src-tauri/` (Rust code, build.rs, tauri.conf.json) and Tauri capability configs.

## Project Architecture

The backend follows a layered architecture:

```
src-tauri/src/
├── commands/       # Tauri command handlers (thin layer calling services)
│   ├── backup.rs   # Backup management commands
│   ├── debug.rs    # Debug mode toggling
│   ├── general.rs  # General app commands (greet, theme)
│   ├── system.rs   # System info commands
│   └── tweaks.rs   # Tweak operations (apply, revert, status)
├── models/         # Data structures and types
│   ├── registry.rs # Registry-related types
│   ├── system.rs   # System info types
│   └── tweak.rs    # Tweak definitions, categories, registry changes
├── services/       # Business logic layer
│   ├── backup_service.rs      # Backup creation/restoration
│   ├── registry_service.rs    # Windows registry operations
│   ├── system_info_service.rs # Windows version detection
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

Tweaks are defined in YAML files under `src-tauri/tweaks/`. Each file contains:

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
    requires_reboot: false
    info: "Optional documentation"
    registry_changes:
      - hive: HKLM # HKCU or HKLM
        key: "System\\..."
        value_name: "Start"
        value_type: "REG_DWORD" # REG_DWORD | REG_SZ | REG_EXPAND_SZ | REG_BINARY
        enable_value: 4 # Value when tweak is applied
        disable_value: 2 # Value when tweak is reverted
        windows_versions: [10] # Optional: [10], [11], or [10, 11]
```

### Adding New Tweaks

1. Add to existing category YAML file or create new file in `tweaks/`.
2. Use unique `id` across all tweaks.
3. Set appropriate `risk_level` based on impact.
4. Use `windows_versions` on individual registry changes for version-specific behavior.
5. Test with both apply and revert operations.

### Tweak Loader Behavior

- Auto-discovers all `.yaml` files in `tweaks/` directory.
- Filters registry changes by current Windows version at runtime.
- Caches loaded tweaks in memory.

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

- HKLM writes require admin privileges; the service checks and returns `Error::RequiresAdmin`.
- All operations are logged at trace/debug level.

## Backup System

Before applying tweaks, the system creates JSON backups:

- Location: `backups/` directory next to executable (portable app design).
- Format: `{tweak_id}.json` containing original registry values.
- Used for reverting tweaks to original state.

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
