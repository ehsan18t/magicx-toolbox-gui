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
