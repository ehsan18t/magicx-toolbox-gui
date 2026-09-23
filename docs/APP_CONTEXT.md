# MagicX Toolbox App Context

This document is a compact orientation guide for humans and AI agents working on MagicX Toolbox.

## Product

MagicX Toolbox is a Windows system optimization app built with Tauri, Rust, Svelte 5, and Tailwind CSS v4. The app applies curated Windows tweaks from embedded YAML definitions and keeps rollback data so changes can be reverted.

## Main Features

- Browse tweak categories and search/favorite tweaks.
- Apply one option of a tweak at a time.
- Batch apply and batch revert tweaks.
- Detect current system state by comparing live Windows state against tweak options.
- Capture internal snapshots before first apply for rollback.
- Revert tweaks from snapshots; a partial revert enters "Needs Attention" (snapshot kept for retry or explicit keep-current-state).
- Select "System Default" to revert whenever a snapshot exists.
- Display Windows/system/hardware details.
- Check for GitHub releases and launch official installers.
- Debug panel and progress/toast feedback for long operations.

## Tweak System

Tweaks live in `src-tauri/tweaks/*.yaml` (one `category:` header per file) and are validated and compiled at build time by `src-tauri/build.rs`. The model is effect-centric: a tweak declares its managed surface once as a list of `effects:`, and each option is a value map over that surface. One authored option renders as a toggle; two or more render as a dropdown. "System Default" is never authored: it is the computed status when the live surface matches no option (ADR-0003).

Effect kinds:

- `registry`: one registry value (optionally one field of a packed value), typed as `REG_DWORD`, `REG_QWORD`, `REG_SZ`, `REG_EXPAND_SZ`, `REG_MULTI_SZ` or `REG_BINARY`; `absent` deletes it.
- `registry_key`: registry key presence.
- `service`: a service's startup type.
- `task`: a scheduled task's enabled state, by exact task path.
- `hosts`: a hosts-file entry's presence.
- `firewall`: a firewall rule's presence, with its full definition.
- `shared`: a reference to a corpus-level shared setting, refcounted across the tweaks that claim it (ADR-0006).
- `action`: a `cmd`/`powershell` script for changes no declarative kind can express, with optional `undo` and `probe`.

See `docs/TWEAK_AUTHORING.md` for the authoring contract.

## Apply And Safety Model

The Tauri commands in `src-tauri/src/commands/tweaks.rs` are a thin layer over the engine in `src-tauri/src/tweaks/engine/` (`apply.rs`, `revert.rs`, `detect.rs`).

- Apply captures every applicable effect's live value before any mutation and persists it as a snapshot entry (with a write-ahead journal of the actions it will run) in a portable `snapshots/` folder next to the executable, written atomically (temp file + rename).
- Effects are then driven and verified one by one, in the order the tweak declares them. There is no fixed per-kind phase order. Adjacent settings that route to TrustedInstaller are batched into one elevated child, on both the forward and the rollback path.
- Apply is atomic *in intent*: any drive or verify failure undoes the completed actions in reverse and drives the whole captured state back. A rollback that cannot fully complete surfaces as "Needs Attention" (ADR-0001).
- Revert undoes the snapshot entry's completed actions in reverse, then restores the captured surface. A captured value dump goes back even for an effect an OS upgrade has since scoped out; an effect the corpus no longer defines fails the restore and keeps the entry. The entry is consumed only on a fully verified restore; a partial revert keeps it and enters "Needs Attention", where the user can retry or explicitly "keep current state" (ADR-0002).
- A tweak's `elevation:` level (`user` / `admin` / `ti`) is its floor; each effect drives at the higher of the floor and its own requirement, and HKCU effects always run in-process as the interactive user. `ti` operations run through the typed broker (no shell strings), and a failed privileged op surfaces as an error.

Do not duplicate system-change application logic. New profile/batch paths should reuse the same apply engine or shared helpers.

## Snapshot System

Snapshots live in a portable `snapshots/` directory next to the executable (`src-tauri/src/tweaks/snapshot.rs`). Each tweak has its own subdirectory, `snapshots/<tweak-id>/`, holding one JSON file per history entry (named by a monotonic sequence number), a `_seq.json` sequence hint, and, when a restore or rollback could not complete, an `_attention.json` Needs Attention record (one per account, `_attention.<SID>.json`, for a tweak that touches HKCU). Each entry is stamped with a schema version, the capturing machine's `MachineGuid`, and the capturing process's user SID, and records either the option that was active or a dump of the captured values, plus the write-ahead journal of its actions. Writes are atomic (temp file + rename).

An entry with the wrong schema version, from another machine, captured by another account for a tweak that touches HKCU, or naming an option the corpus no longer defines is treated as invalid: it is skipped and surfaced, never deleted. Keep current state never releases another account's HKCU entry. An entry is deleted only by a verified restore or rollback, by dedup when the same option is captured again (a settled entry only), or by an explicit user decision ("keep current state" or discarding an entry) (ADR-0002). On startup, a crash-residue scan flags any drive or action a crash left unfinished as Needs Attention. Shared settings are tracked separately, one file per machine, in `snapshots/shared_claims.<MachineGuid>.json` (ADR-0006).

## Profile System

The profile system (`.mgx` export/import) was removed in the current build and its UI is disabled. A rebuild is planned for later, sharing one machine-identity mechanism with the snapshot install ID; the v1 archive format is preserved in `docs/spec/profile-v1.md` so existing archives remain recoverable. Windows restore-point creation is not implemented.

## Backend Map

- `src-tauri/src/models/`: shared Rust data models.
- `src-tauri/src/commands/`: Tauri command handlers.
- `src-tauri/src/commands/tweaks.rs`: tweak query/apply/revert commands.
- `src-tauri/src/tweaks/engine/`: detect, apply, revert, and per-effect execution-context routing.
- `src-tauri/src/tweaks/kinds/`: six modules covering seven of the eight effect kinds (`registry.rs` handles both `registry` and `registry_key`; `service`, `task`, `hosts`, `firewall`, `action`). `shared` references are handled by `tweaks/shared_claims.rs`.
- `src-tauri/src/tweaks/snapshot.rs`: per-tweak snapshot store (atomic writes, Needs Attention record).
- `src-tauri/src/tweaks/shared_claims.rs`: refcounted claims on shared blocks (ADR-0006).
- `src-tauri/src/services/registry_value.rs`: canonical registry JSON parsing, writing, and comparison.
- `src-tauri/src/services/elevation/`: TrustedInstaller execution through the typed broker.
- `src-tauri/src/services/system_info_service.rs`: lightweight runtime context and full WMI-backed system information.

## Frontend Map

- `src/lib/api/`: Tauri invoke wrappers.
- `src/lib/stores/*.svelte.ts`: Svelte 5 rune stores.
- `src/lib/components/ui/`: shared UI primitives.
- `src/lib/components/tweaks/`: tweak cards and detail views.
- `src/lib/components/modals/`: tweak/profile/settings/update modals.
- `src/lib/components/views/`: main app views.

Use existing UI primitives before creating new components.

## Contributor Rules

- Follow the repo `CLAUDE.md`.
- When tweak runtime behavior changes, update `docs/TWEAK_AUTHORING.md`.
- Keep types, constants, helpers, and components in focused files.
- Avoid duplicating registry/apply/validation logic.
- Do not silently ignore privileged operation failures.
- Use the lightweight runtime context for tweak/profile hot paths.
- Run `pnpm run validate` before committing.

## Validation

- Full stack: `pnpm run validate`
- Frontend only: `pnpm run lint && pnpm run type-check`
- Backend only: `cd src-tauri && cargo check && cargo clippy --all-targets --all-features -- -D warnings`
