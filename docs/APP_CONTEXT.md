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

Tweaks live in `src-tauri/tweaks/*.yaml` and are compiled at build time by `src-tauri/build.rs`. Each tweak has at least two options. Each option is a complete target state and can include:

- Registry changes: set/delete/create key, typed values including `REG_DWORD`, `REG_QWORD`, `REG_SZ`, `REG_EXPAND_SZ`, `REG_BINARY`, and `REG_MULTI_SZ`.
- Windows service changes: startup type plus optional start/stop.
- Task Scheduler changes: exact task names or regex patterns.
- Hosts file changes.
- Firewall rules via `netsh`.
- Shell and PowerShell pre/post hooks.

See `docs/TWEAK_AUTHORING.md` for the authoring contract.

## Apply And Safety Model

Manual tweak apply is handled by `src-tauri/src/commands/tweaks/apply.rs` and `helpers.rs`.

- First apply captures an internal snapshot in a portable `snapshots/` folder next to the executable (written atomically: temp file + rename).
- Switching options captures the current state for failure rollback, then updates snapshot metadata on success.
- Core changes apply in order: registry, services, scheduler, hosts, firewall. Each phase is atomic *in intent*; a failed phase rolls the whole tweak back from the snapshot.
- Revert restores all five phases and collects failures; the snapshot is released only on a fully verified restore. A partial revert enters "Needs Attention" — the snapshot is kept and the user can retry or explicitly "keep current state" (ADR-0001 / ADR-0002).
- Pre-command and pre-PowerShell failures abort before core changes; post-hook failures are logged and do not roll back successful core changes.
- `requires_admin` / `requires_system` / `requires_ti` determine elevation; privileged operations run through the typed broker (no shell strings) and a failed privileged op surfaces as an error.

Do not duplicate system-change application logic. New profile/batch paths should reuse the same apply engine or shared helpers.

## Snapshot System

Snapshots are one JSON file per tweak in a `snapshots/` directory next to the executable (portable). They record registry values, service states, scheduled task states, hosts entries, and firewall rules, plus a schema version, the capturing machine's `MachineGuid` (a load-time mismatch warns), and a Needs-Attention marker with the unrestorable-resource list. Writes are atomic (temp file + rename); the metadata read-modify-write path takes an exclusive `std::fs::File::lock`.

On startup, stale snapshot validation removes a snapshot only when every captured resource is verifiably back at the original state. If a resource cannot be checked safely, the snapshot is preserved.

## Profile System

The profile system (`.mgx` export/import) was removed in the current build and its UI is disabled. A rebuild is planned for later, sharing one machine-identity mechanism with the snapshot install ID; the v1 archive format is preserved in `docs/spec/profile-v1.md` so existing archives remain recoverable. Windows restore-point creation is not implemented.

## Backend Map

- `src-tauri/src/models/`: shared Rust data models.
- `src-tauri/src/commands/`: Tauri command handlers.
- `src-tauri/src/commands/tweaks/`: tweak query/apply/batch commands.
- `src-tauri/src/services/backup/`: snapshot capture, restore (all-phase), storage (atomic), detection, inspection.
- `src-tauri/src/services/registry_value.rs`: canonical registry JSON parsing, writing, and comparison.
- `src-tauri/src/services/elevation/`: SYSTEM and TrustedInstaller execution.
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
- Run `bun run validate` before committing.

## Validation

- Full stack: `bun run validate`
- Frontend only: `bun run lint && bun run type-check`
- Backend only: `cd src-tauri && cargo check && cargo clippy --all-targets --all-features -- -D warnings`
