# CLAUDE.md

Guidance for Claude Code (and other agents) working in this repo.

MagicX Toolbox is a **Windows-only** system-optimization app: a Tauri + Rust backend and a Svelte 5 + Tailwind CSS v4 frontend. It applies curated Windows tweaks from embedded YAML and keeps snapshots so changes can be reverted. Because it is Windows-only, cross-platform dependency bloat is pure cost.

## Code style

- Write no more code than the task needs — keep it small and optimized. "Small" is never an excuse to cut what's genuinely required; use what's necessary.
- One rule, one place: when the same decision (which items a page lists, a user-facing warning, a positioning routine) appears twice, extract it before writing it a third time. Duplicated rules drift and disagree.
- A failure the user caused or must act on reaches the user (toast, inline error, error state), not only the log. `.catch(() => {})` and log-and-return on a user action are bugs.

## Comments — hard rules

Every comment is re-read on every pass over the file, so it must earn its tokens. **The test: a comment is justified only when omitting it would cost more than including it** — a wrong edit, or a fact re-derived from scratch. "Mildly useful" is below the bar.

- **Budget.** One line by default, three lines the hard cap, a module `//!` header five. Over budget means compress or delete, never wrap onto more lines.
- **Fragments, not prose.** `// TerminateProcess fails with error 5 on an already-exited process.` Cut every word that is not the fact: no "note that", "deliberately", "belt-and-suspenders", and no "see X for why" pointer the reader already reaches from the type.
- **Prefer a name.** If a comment explains *what* something is, rename the item and delete the comment.
- **Rustdoc is not ceremony.** `///` only where the signature is genuinely ambiguous; never `# Errors` / `# Returns` / `# Arguments` sections restating the types. `pub` in this binary crate is a visibility modifier, not a published API.
- **No history framing** (`previously`, `used to`, `the old code`, `no longer`, `(review fix)`, dates, shas, PR numbers, "Stage N" / "WP" labels). That is the commit message's job. **But keep the fact**: rewrite present-tense rather than deleting (`// the old code parsed the localized "Status:" line` → `// Read the exit code: the "Status:" text is translated.`).
- **The only four reasons to comment:**
  1. **A Windows behaviour the code cannot show.** (`RtlGetVersion`, never `GetVersionEx`: the compat shim under-reports.)
  2. **A deliberate non-choice**: the obvious alternative reintroduces a defect. Name both. (`serde_yaml_bw`, not `serde_yml`: RUSTSEC-2025-0068.)
  3. **A safety-contract link**: cite the ADR (`ADR-0002`) so a refactor sees what it holds up.
  4. **Non-obvious code that must not be simplified**, because clarity would cost stability or performance. Say which.
- **Everything else is deleted:** restating the code, section banners, commented-out code, `TODO`/`FIXME` (fix it or file it in `docs/KNOWN_ISSUES.md`).

Applies to `.rs`, `.ts`, `.svelte`, tweak YAML, `Cargo.toml`, config files and CI alike. Bring any comment you touch into compliance in the same change.

## Docs style

- **Never use em dashes (—) in user-facing docs** (`docs/*.md`, `TWEAK_AUTHORING.md`, READMEs, tweak YAML text, UI copy). Use a comma, colon, parentheses, semicolon, or a separate sentence.

## The gate — run before every commit

- **Full stack:** `pnpm run validate`: prettier, tsc, svelte-check `--fail-on-warnings`, `cargo fmt --check`, clippy `-D warnings`, eslint `--max-warnings 0`, node tests (`pnpm run test`), `cargo test` (`pnpm run test:rust`). A warning fails the gate.
- **Backend only:** `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
- **Frontend only:** `pnpm run check && pnpm run type-check && pnpm run lint && pnpm run test`

Fix every issue the gate reports, whether or not your change caused it. Tests touching live system state (Task Scheduler COM, firewall, hosts file, TrustedInstaller) are `#[ignore]`d; run them elevated with `cargo test -- --ignored --skip dump_preview_corpus --test-threads=1`. Serial threads are required: the scheduler tests race libtest's per-test thread churn into a STATUS_ACCESS_VIOLATION (a harness artifact, not a code defect). `dump_preview_corpus` is skipped because it rewrites `src/lib/preview/corpus.json`; run it alone after changing a tweak the preview corpus includes.

## Testing

- Pure logic in `src/lib/utils/` (and pure helpers pulled out of stores) gets a colocated `*.test.ts` run by `node --test`; `$lib` imports resolve through `scripts/test-loader.mjs`. Rune modules (`*.svelte.ts`) and components are not unit-tested: keep logic worth testing out of them.
- Anything that sanitizes or escapes input for `{@html}` must have tests covering escaping and rejected URL schemes.
- Rust: unit tests beside the code; a behaviour change ships with a test that fails without it.

## Git

- **Never push.** The maintainer does all pushes; commit locally only.
- **Never `git add -A` / `git add .`.** `PROGRESS.md` at the repo root is untracked scratch, and `-A` sweeps it in. Always stage explicit paths.
- **Commit by task, not by file.** Conventional-commit titles (`fix(registry): …`, `feat(ui): …`), imperative mood, **no internal labels** ("WP", "Stage N", "wip"). Don't group unrelated changes.
- **Batch docs.** Don't commit status docs after each work-package; fold them into one docs commit at the end of a stage.
- **CRLF is enforced** (`.gitattributes eol=crlf`, `rustfmt.toml newline_style = "Windows"`, and a CI job asserts every tracked text file is CRLF). The Edit/Write tools emit LF; git normalizes to CRLF on commit, so committing is fine. To discard an LF-only working-tree diff, `git checkout -- <file>`.
- Multi-step work: branch per unit → `git merge --squash` onto `main` with a proper message (or commit directly on `main` with explicit staging). `git rebase -i` is not available here.

## Backend (Rust / Tauri) — `src-tauri/`

- Windows / MSVC. Commands live in `commands/*.rs` (`#[tauri::command]`, return `Result<T, Error>`, log at entry) and register in `lib.rs` via `generate_handler!`.
- Errors: `thiserror`, propagate with `?`. Logging: the `log` crate **only** — never `println!` / `eprintln!`. Lock mutexes minimally; don't block the main thread (blocking work goes through `spawn_blocking` or an async command) or hardcode paths.
- **Tauri capabilities are least-privilege:** a new plugin or plugin permission goes in `src-tauri/capabilities/*.json` with the narrowest scope that works; never a wildcard permission.
- Clippy runs its default groups with `-D warnings`. `clippy.toml` holds only thresholds of lints that are actually enabled; a threshold for a pedantic/restriction lint is dead config.
- **Privileged operations run through the typed elevation broker** (`services/elevation/`), never by composing shell strings: `admin` runs in-process in the elevated app; `ti` re-spawns the app under a TrustedInstaller token and runs typed `BrokerOp`s through the same effect services. Registry via `RegSetValueExW`, services via `windows-sys` SCM, scheduler via `windows` COM. `BrokerOp` carries no script variant: PowerShell runs only through the `action` effect kind (`tweaks/kinds/action.rs`) and app items (`apps/`, on the same runner), never through the broker.
- **The "did-it-work" contract:** a failed privileged or effect operation must surface as `Err`, never a benign-looking value. Registry reads must distinguish *not-found* from *access-denied*. Never `let _ =` a privileged call.
- A hand-written `Ord` needs a matching `PartialEq`/`Eq` (implement equality through `cmp`); deriving one and hand-writing the other breaks the `Ord` contract.

### Logging — `src-tauri/src/logging/` (ADR-0010, `docs/architecture/logging.md`)

- Redaction lives in the logger: every line is redacted before the ring, the panel, the file or an export. Don't redact at call sites, and never log values read from the user's registry, hosts file or other files (redaction knows paths and names, not data).
- `get_log_tail` (polled) and `log_frontend` (itself a log line) are the only commands exempt from log-at-entry.
- Never call `log::` from inside `logging/`: the re-entrancy guard silently drops it. Nothing there may panic.
- The stderr echo in `pipeline.rs` runs only in debug builds; it and the broker child's panic hook are the only sanctioned stderr writes.
- The broker child never redacts or writes files: its lines return in the response and the parent re-logs them only after validation. Never log the broker command line (temp paths).

## Frontend (Svelte 5) — `src/`

- **Svelte 5 runes** (`$state`, `$derived`, `$effect`) — never legacy `export let`, `on:event`, `$:`, `<slot>`, `createEventDispatcher`, and never `$store` subscription syntax with rune stores. Stores are `.svelte.ts` modules exposing state through getters (`src/lib/stores/`).
- **`$effect` is the escape hatch**, for syncing with something outside Svelte (DOM, timers, Tauri events). Derive values with `$derived` (writable `$derived` for resettable local state); react to user input in the event handler; act on an element with `{@attach}`. An effect that writes state another effect reads, or a counter bumped to signal another component, is a smell: call a function instead.
- **`{@attach}` over `use:` actions** for new code; convert an action you touch.
- A child never mutates a parent-owned object it received as a prop; it takes a callback, or the prop is `$bindable`.
- Tailwind CSS v4 utility classes; no global styles outside `src/app.css`. Aliases: `$lib`, `@/*` → `src/*`. No root `$lib/index.ts`: import components from their folder barrel (`$lib/components/<group>`; eslint rejects `$lib/components/*/*` deep imports, siblings import relatively) and stores from their own module (`$lib/stores/<name>.svelte`).
- Reuse the UI primitives in `$lib/components/ui` (`Button`, `Badge`, `Card`, `Modal`, `Select`, `Switch`, `Spinner`, …) before building new ones. Design data (icon registry, `ICON_SIZE`, tone maps) lives in `$lib/design`; any new icon must be registered in `$lib/design/icons.ts`, and tone opacity steps are named in `$lib/design/tone.ts`, never written in a caller.
- Tauri `invoke` and plugin calls (`opener`, `dialog`, `process`, `path`) live in `$lib/api`; stores and components call those wrappers. No direct `fetch` to local files. External links use the `ExternalLink` component. Don't block the UI on long-running calls.
- IPC types are generated, never hand-written: change the Rust serde type, run `cargo test --features test-build` in `src-tauri` (ts-rs writes `src/lib/types/generated/`), and commit the result; `src/lib/types/index.ts` only re-exports and adapts them. CI fails on stale bindings.
- Data read back from `localStorage` is untrusted (an older version may have written another shape): every persisted store passes a parser that validates and falls back to the default.
- After editing a component, the official Svelte MCP `svelte-autofixer` is the expected check.
- **Motion and z-index are tokens** (`@theme static` in `src/app.css`, `docs/ARCHITECTURE.md` § Motion): no literal `duration-150`, `ms`, `cubic-bezier` or `z-1000`. Enter-only motion is an `animate-*` class; exit or height motion uses `$lib/utils/motion` presets, never `svelte/transition` directly (those run on the Web Animations API, which the CSS reduced-motion override cannot reach). Never animate per item in lists that re-render on keystrokes or rescans.

### Accessibility — WCAG 2.2 AA

- Every control is reachable and operable by keyboard, has an accessible name, and keeps a visible focus style.
- A control that cannot be used right now explains why to keyboard and screen-reader users too: `aria-disabled="true"` (stays focusable, activation blocked in the handler) with the reason referenced by `aria-describedby`, never native `disabled` plus a hover-only tooltip.
- Dialogs and overlay drawers contain focus while open and return it to the opener on close; global shortcuts do nothing while a modal dialog or the applying overlay is up.
- Timed content (toasts) pauses on hover and focus; anything carrying an action button does not auto-dismiss while focused.
- Motion respects `prefers-reduced-motion` (the presets above do this).

## Safety model & ADRs — `docs/adr/`

- **Apply is atomic *in intent*, not guaranteed.** A failed phase rolls the tweak back from the snapshot; a rollback that cannot fully complete surfaces as **Needs Attention** rather than hiding it (ADR-0001). "Atomic" means *attempted atomically, with failure surfaced*.
- **A snapshot is deleted only by a verified restore or an explicit user decision** (`keep_current_state`) — never on a failure path. `let _ = restore(...)` is a bug (ADR-0002).
- **System Default is a computed status, never a target** (ADR-0003): the live surface matches no authored option. The only way back is Restore Snapshot, which walks the snapshot history. Only a 1-option tweak shows a System Default segment, so user-facing text for a 2+ option tweak says "Restore", never "pick System Default".
- Snapshots: one JSON file per tweak in a portable `snapshots/` directory next to the executable; written atomically (temp file + rename); stamped with a schema version and the machine's `MachineGuid`.

## Tweak system / YAML

- Tweaks are YAML in `src-tauri/tweaks/`, compiled at build time by `build.rs` (the schema types are shared with the runtime via `src/tweaks/{model,parse,schema,validate}.rs`, so drift is a compile error). Category is **per file**, not per tweak: one `category:` header per YAML file. 1 authored option → System default | option switch, 2 → segmented switch, 3+ → dropdown; you never author "System Default", it is the computed state when the live surface matches no option. `optional: true` (with an optional `if_missing:`) tolerates a *missing* resource at capture and detect; it does not weaken the post-apply verify.
- Every build lists tweaks and apps the running Windows build cannot run, with `supported: false`; the UI hides them unless Settings enables "Show tweaks this PC cannot run", and the backend refuses to apply them regardless.
- **Removable apps are app items** (`apps:` beside `tweaks:`, ADR-0009), never a tweak with a removal `action:`: presence plus Remove and Install, no options, no snapshot, no Restore. A script item's `probe` exits 0 installed, 2 absent; anything else is Unknown, so never `exit 2` from a `catch`.
- **When tweak runtime behavior changes, update `docs/TWEAK_AUTHORING.md`**, the authoritative author guide. `docs/architecture/tweak/` is the architecture reference: update the matching page when a component's behaviour changes.
- **When a tweak changes (effect, option, value, gate, risk), update its entry in `docs/tweaks/<category>.md` in the same commit.** The wiki is the complete per-tweak reference; the YAML stays lean. Adding a tweak adds an entry and an index row in `docs/tweaks/README.md`; removing one moves it to "Considered and not shipped", except a tweak converted to an app item, whose entry moves to its page's Apps section and whose row moves to the app index.

## Dependencies

Before adding any crate or npm package: research alternatives; prefer maintained (updated within ~6 months), ≥1.0, permissively licensed (MIT / Apache-2.0 / BSD); avoid heavy or unmaintained deps. Pin every version with a caret range, never `latest`.
