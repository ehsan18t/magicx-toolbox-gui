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
