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
- Routing: SPA fallback is configured; don’t add SSR-only patterns. Assets served from `static/`.
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

## Avoid

- Adding new global styles outside `src/app.css` unless justified.
- Using direct `fetch` to local files; use Tauri commands via `@tauri-apps/api` when talking to backend.
- Adding new NPM dependencies without confirming compatibility with Vite/SvelteKit/Tauri.
- Blocking the UI with long-running calls; offload via Tauri commands instead.

## Testing & linting

- Run `bun run validate` (format, lint, check) before committing frontend changes.

## Commits

- Keep commits task-scoped (one task can touch multiple files). Avoid dumping unrelated changes together.
- Use clear messages, e.g., `feat(ui): add settings drawer toggle` or `fix(theme): persist system preference on init`.

# Copilot Instructions – Rust (Tauri Backend)

## Scope

- Applies to `src-tauri/` (Rust code, build.rs, tauri.conf.json) and Tauri capability configs.

## Expectations

- Target MSVC toolchain on Windows; avoid GNU-only crates. Keep dependencies compatible with Tauri 2.
- Commands live under `src-tauri/src/commands`; register them in `lib.rs` via `generate_handler!`.
- Use `AppState` for shared state; lock mutexes minimally and avoid long-held locks.
- Prefer `anyhow`/`thiserror` patterns already in use for error handling.
- Keep capability and permission changes in `src-tauri/capabilities/*.json` aligned with Tauri plugin usage (shell/opener/etc.).
- For window behavior, wire changes through `tauri.conf.json` rather than hardcoding in Rust when possible.

## Build / toolchain / env

- Use `rustup default stable-x86_64-pc-windows-msvc`. Ensure VS Build Tools with C++ workload installed (provides `link.exe`).
- Verify WebView2 runtime exists on Windows; do not hard-pin unless necessary.
- Commands: `cargo check` inside `src-tauri` for backend; `bun run validate` for full stack.

## Patterns to follow

- Use `tauri::Builder::default()` chaining; keep plugins initialization explicit.
- When adding new commands, define a small request struct with `serde::Deserialize` if more than 2 args.
- Log through `log` crate; avoid `println!`.
- Keep feature flags minimal; prefer default features off unless required by the command.
- For blocking IO/CPU, use `tauri::async_runtime::spawn_blocking` or equivalent; avoid holding mutexes across awaits.
- Align plugin usage with capabilities (e.g., shell/opener) and update `tauri.conf.json` if window behavior changes.
- Keep error messages user-safe; avoid leaking file paths in user-facing strings.

## Avoid

- Blocking the main thread with long-running work; offload to async or spawn blocking as needed.
- Introducing plugins without updating capabilities permissions.
- Using unstable Rust features.
- Writing to the filesystem or shelling out without validating paths/inputs.

## Testing & linting

- Run `bun run validate` for frontend and `cargo check` in `src-tauri` for backend before committing.

## Commits

- Keep commits task-scoped (a task can span multiple files); avoid bundling unrelated backend/frontend changes in one commit.
- Use clear messages, e.g., `feat(commands): add export_logs command` or `fix(state): guard mutex poisoning on theme updates`.
