# Copilot Instructions

**CRITICAL: These instructions are mandatory. Ignoring them will result in suboptimal code quality.**

## Validation

**ALWAYS run `bun run validate` before committing.** This runs format, lint, type-check, svelte-check, cargo fmt, and cargo clippy.

## Documentation Priority

When modifying the tweak system, **update `TWEAK_AUTHORING.md` immediately**. It's the definitive guide for tweak authors.

## Commits

- Commit by **TASK**, not by file
- Use clear messages: `feat(ui): add toggle` or `fix(theme): persist preference`
- Don't group unrelated changes

## Dependency Guidelines

Before adding ANY npm package or cargo crate:

1. Research 3+ alternatives
2. Evaluate: maintenance (<6 months updates), stability (1.0+), performance, bundle size, security
3. Avoid: unmaintained packages, excessive dependencies, pre-1.0 for production
4. Prefer: MIT/Apache-2.0/BSD licenses

---

# Frontend (Svelte)

**Scope:** All files under `src/`

## Key Rules

- Use **Svelte 5 runes** (`$state`, `$derived`, `$effect`); avoid legacy `export let`
- Use **Tailwind CSS v4** utility classes
- Aliases: `@/*` → `src/*`, `$lib` for library exports
- External links: use `ExternalLink` component (opens via Tauri shell)
- Drag regions: `drag-enable` for title bar, `drag-disable` for interactive elements

## Store Pattern

Stores use `.svelte.ts` files with getter-based access. See existing stores in `src/lib/stores/` for patterns.

## UI Components

Use primitives from `$lib/components/ui`: `Button`, `Badge`, `Card`, `Modal`, `IconButton`, `Switch`, `Select`, `Spinner`

For tweak details: `$lib/components/tweak-details` has `RegistryChangeItem`, `ServiceChangeItem`, `SchedulerChangeItem`, `CommandList`

## Patterns

- State: `$state`, `$derived`; `onMount` for browser-only work
- CSS: Tailwind class order; use `@/app.css` variables
- Imports: prefer barrel exports from `$lib/index.ts`
- Accessibility: include aria labels; maintain focus styles
- Performance: debounce expensive handlers; prefer CSS transitions

## Avoid

- Global styles outside `src/app.css`
- Direct `fetch` to local files (use Tauri commands)
- `$store` subscription syntax with rune-based stores
- Blocking UI with long-running calls

---

# Backend (Rust/Tauri)

**Scope:** All files under `src-tauri/`

## Architecture

```
src-tauri/src/
├── commands/       # Tauri command handlers
├── models/         # Data structures (tweak.rs, system.rs)
├── services/       # Business logic (registry, scheduler, backup, etc.)
├── lib.rs          # App entry, command registration
├── error.rs        # Custom error types
├── state.rs        # AppState
└── setup.rs        # Init hooks
```

## Key Rules

- Target MSVC toolchain on Windows
- Commands go in `commands/*.rs`, register in `lib.rs` via `generate_handler!`
- Use `thiserror` for errors; propagate with `?`
- Lock mutexes minimally

## Logging

Use `log` crate macros (never `println!`):

- `log::info!` for command entry points
- `log::debug!`/`log::trace!` for internal operations
- `log::error!` before returning errors

## YAML Tweaks

Tweaks defined in `src-tauri/tweaks/`. See `TWEAK_AUTHORING.md` for full schema and examples.

Key points:

- YAML compiled at build time via `build.rs`
- Options array model: 2 options = toggle, 3+ = dropdown
- `skip_validation: true` excludes item from status checks

## Adding Commands

1. Create handler in `commands/*.rs` with `#[tauri::command]`
2. Return `Result<T, Error>`
3. Add logging at entry
4. Register in `lib.rs`

## Avoid

- Blocking main thread
- Plugins without capability permissions
- `println!`/`eprintln!` (use `log` macros)
- Hardcoded paths

---

# Testing & Validation

- **Full stack:** `bun run validate` from project root
- **Frontend only:** `bun run lint && bun run type-check`
- **Backend only:** `cargo check && cargo clippy` in `src-tauri`
