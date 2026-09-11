# CLAUDE.md

Guidance for Claude Code (and other agents) working in this repo.

MagicX Toolbox is a **Windows-only** system-optimization app: a Tauri + Rust backend and a
Svelte 5 + Tailwind CSS v4 frontend. It applies curated Windows tweaks from embedded YAML and keeps
snapshots so changes can be reverted. Because it is Windows-only, cross-platform dependency bloat is
pure cost.

## Code style

- Write no more code than the task needs — keep it small and optimized. "Small" is never an excuse to
  cut what's genuinely required; use what's necessary.

## Comments — hard rules

Every comment is re-read on every pass over the file, so it must earn its tokens. **The test: a comment
is justified only when omitting it would cost more than including it** — a wrong edit, or a fact
re-derived from scratch. Its bytes are paid on every read; the saving is paid once. A line that stops
someone reintroducing `GetVersionEx` clears that bar. A paragraph restating the function name never
does. "Mildly useful" is below the bar.

- **Budget, enforced.** One line is the default, three lines the hard cap, a module `//!` header five.
  Over budget means compress or delete, never wrap onto more lines. Length is the real cost here, not
  count: a few long blocks outweigh all the short ones put together.
- **Fragments, not prose.** `// TerminateProcess fails with error 5 on an already-exited process.` — not
  "We check the error code after TerminateProcess because if the child has already exited the call
  fails with ERROR_ACCESS_DENIED, which we then treat as success rather than a real failure…". Cut every word that is not the fact: no "note that", "worth
  naming", "deliberately", "belt-and-suspenders", "this is the whole story about", and no "see X for
  why" pointer the reader already reaches from the type.
- **Prefer a name.** If a comment explains *what* something is, rename the item and delete the comment.
  The identifier is already in the file, so it costs nothing extra.
- **Rustdoc is not ceremony.** `///` only where the signature is genuinely ambiguous. Never a
  `# Errors` / `# Returns` / `# Arguments` section restating the types: `Result<T, Error>` already says
  it can fail and the `Error` enum already lists how. Most `pub` items in a binary crate need no doc at
  all; `pub` here is a visibility modifier, not a published API.
- **Banned: history framing.** No comment narrates this repo's past: `previously`, `used to`, `the old
  code`, `changed from`, `no longer`, `until this fix`, `(review fix)`, `now that we`, plus dates,
  shas, PR or issue numbers, "Stage N" / "WP" labels, before/after pairs. That is the commit message's
  job. **But keep the fact** — most history comments here guard a real trap, so rewrite present-tense
  rather than deleting: `// the old code parsed the localized "Status:" line` becomes `// Read the exit
  code: the "Status:" text is translated.`
- **The only four reasons to comment at all:**
  1. **A Windows behaviour the code cannot show** — an API that lies, a locale trap, a flag with a
     non-obvious effect. (`RtlGetVersion`, never `GetVersionEx`: the compat shim under-reports.)
  2. **A deliberate non-choice** — the obvious alternative reintroduces a defect. Name both.
     (`serde_yaml_bw`, not `serde_yml`: RUSTSEC-2025-0068.)
  3. **A safety-contract link** — cite the ADR (`ADR-0002`) so a refactor sees what it holds up.
  4. **Non-obvious code that must not be simplified**, because clarity would cost stability or
     performance. Say which of the two.
- **Everything else is deleted:** restating the code, section banners, commented-out code,
  `TODO`/`FIXME` (fix it or file it in `docs/KNOWN_ISSUES.md`), and any comment on a private helper
  whose name and signature already say it.

Applies to `.rs`, `.ts`, `.svelte`, tweak YAML, `Cargo.toml`, and CI alike. Bring any comment you touch
into compliance in the same change.

## Docs style

- **Never use em dashes (—) in user-facing docs** (`docs/*.md`, `TWEAK_AUTHORING.md`, READMEs, and any
  author-facing or end-user text). Use a comma, colon, parentheses, semicolon, or a separate sentence.

## The gate — run before every commit

- **Full stack:** `pnpm run validate` (prettier, tsc, svelte-check, `cargo fmt --check`, clippy
  `-D warnings`, eslint).
- **Backend only:** `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
- **Frontend only:** `pnpm run check && pnpm run type-check && pnpm run lint`

Fix every issue the gate reports, whether or not your change caused it. Two scheduler tests that
activate the live Task Scheduler COM service are `#[ignore]`d (they race libtest's per-test
thread-churn into a STATUS_ACCESS_VIOLATION — a harness artifact, not a code defect); run them
explicitly with `cargo test -- --ignored`.

## Git

- **Never push.** The maintainer does all pushes; commit locally only.
- **Never `git add -A` / `git add .`.** `PROGRESS.md` at the repo root is untracked scratch and
  `docs/ROADMAP.md` is often mid-edit — `-A` sweeps both in. Always stage explicit paths.
- **Commit by task, not by file.** Conventional-commit titles (`fix(registry): …`, `feat(ui): …`),
  imperative mood, **no internal labels** ("WP", "Stage N", "wip"). Don't group unrelated changes.
- **Batch docs.** Don't commit `ROADMAP.md` / status docs after each work-package; fold them into one
  docs commit at the end of a stage.
- **CRLF is enforced** (`.gitattributes eol=crlf`, `rustfmt.toml newline_style = "Windows"`, and a CI
  job asserts every tracked text file is CRLF). The Edit/Write tools emit LF; git normalizes to CRLF
  on commit, so committing is fine. To discard an LF-only working-tree diff, `git checkout -- <file>`.
- Multi-step work: branch per unit → `git merge --squash` onto `main` with a proper message (or
  commit directly on `main` with explicit staging). `git rebase -i` is not available here.

## Backend (Rust / Tauri) — `src-tauri/`

- Windows / MSVC. Commands live in `commands/*.rs` (`#[tauri::command]`, return `Result<T, Error>`,
  log at entry) and register in `lib.rs` via `generate_handler!`.
- Errors: `thiserror`, propagate with `?`. Logging: the `log` crate **only** — never
  `println!` / `eprintln!`. Lock mutexes minimally; don't block the main thread or hardcode paths.
- **Privileged operations run through the typed elevation broker** (`services/elevation/`), never by
  composing shell strings: the app re-spawns itself under a SYSTEM / TrustedInstaller token and runs
  typed `BrokerOp`s through the same effect services. Registry via `RegSetValueExW`, services via
  `windows-sys` SCM, scheduler via `windows` COM. `BrokerOp` carries no script variant: PowerShell
  runs only through the `action` effect kind (`tweaks/kinds/action.rs`), never through the broker.
- **The "did-it-work" contract:** a failed privileged or effect operation must surface as `Err`, never
  a benign-looking value. Registry reads must distinguish *not-found* from *access-denied*. Never
  `let _ =` a privileged call.

## Frontend (Svelte 5) — `src/`

- **Svelte 5 runes** (`$state`, `$derived`, `$effect`) — never legacy `export let`, and never `$store`
  subscription syntax with rune stores. Stores are `.svelte.ts` files with getter-based access
  (`src/lib/stores/`).
- Tailwind CSS v4 utility classes; no global styles outside `src/app.css`. Aliases: `$lib`,
  `@/*` → `src/*`; prefer barrel exports from `$lib/index.ts`.
- Reuse the UI primitives in `$lib/components/ui` (`Button`, `Badge`, `Card`, `Modal`, `Select`,
  `Switch`, `Spinner`, …) before building new ones. Any new icon must be imported in `Icon.svelte`.
- No direct `fetch` to local files — go through Tauri commands. External links use the `ExternalLink`
  component. Include aria labels, keep focus styles, and don't block the UI on long-running calls.
- After editing a component, the official Svelte MCP `svelte-autofixer` is the expected check.

## Safety model & ADRs — `docs/adr/`

- **Apply is atomic *in intent*, not guaranteed.** A failed phase rolls the tweak back from the
  snapshot; a rollback that cannot fully complete surfaces as **Needs Attention** rather than hiding
  it (ADR-0001). "Atomic" means *attempted atomically, with failure surfaced*.
- **A snapshot is deleted only by a verified restore or an explicit user decision**
  (`keep_current_state`) — never on a failure path. `let _ = restore(...)` is a bug (ADR-0002).
- **System Default is a selectable state = a Revert** whenever a snapshot exists (ADR-0003).
- Snapshots: one JSON file per tweak in a portable `snapshots/` directory next to the executable;
  written atomically (temp file + rename); stamped with a schema version and the machine's
  `MachineGuid`.

## Tweak system / YAML

- Tweaks are YAML in `src-tauri/tweaks/`, compiled at build time by `build.rs` (the schema types are
  shared with the runtime via `src/tweaks/{model,parse,schema,validate}.rs`, so drift is a compile
  error). Category is **per file**, not per tweak: one `category:` header per YAML file. 1 authored
  option → toggle, 2+ → dropdown; you never author "System Default", it is the computed state when
  the live surface matches no option. `optional: true` (with an optional `if_missing:`) tolerates a
  *missing* resource at capture and detect; it does not weaken the post-apply verify.
- **When tweak runtime behavior changes, update `docs/TWEAK_AUTHORING.md`** — it is the authoritative
  author guide. `docs/TWEAK_SYSTEM.md` is the architecture reference and `docs/ROADMAP.md` is the
  stage tracker.

## Dependencies

Before adding any crate or npm package: research alternatives; prefer maintained (updated within
~6 months), ≥1.0, permissively licensed (MIT / Apache-2.0 / BSD); avoid heavy or unmaintained deps.
