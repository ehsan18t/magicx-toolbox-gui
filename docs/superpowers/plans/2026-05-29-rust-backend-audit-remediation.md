# Rust Backend Audit Remediation Implementation Plan

> ## STATUS: SUPERSEDED — 2026-07-20
>
> Most of this plan was executed (see commits `08fa2e6`, `522d9e1`, `5131569`, `2621dda`), but the
> checkboxes were never updated, so it reads as pending when it is not. They are left as-is rather than
> ticked, because ticking boxes without re-verifying each one would be worse than leaving them ambiguous.
>
> Superseded by [TWEAK_SYSTEM_PLAN.md](../../TWEAK_SYSTEM_PLAN.md), informed by
> [TWEAK_SYSTEM_REVIEW.md](../../TWEAK_SYSTEM_REVIEW.md).
>
> **Carried forward — verified still outstanding:**
>
> - **Task 1 is incomplete.** Its acceptance criterion was *"`REG_BINARY` authored values apply, restore,
>   export/import, and **detect** consistently."* `detection.rs:243` uses the normalized
>   `registry_value::registry_values_match`, but `inspection.rs:156` still uses `helpers::values_match`
>   (raw JSON equality). A `REG_BINARY` value authored in the supported `"00,A0,FF"` hex form therefore
>   matches in detection and mismatches in inspection. The unification stopped one file short. Addressed
>   by the detection/inspection collapse in the new plan's stage 4.
>
> **No longer applicable:**
>
> - **Tasks 2, 3 and 6** concern the profile system, which is being deleted and rebuilt. See
>   "Profile system — delete now, rebuild later" in the new plan.
>
> **Still binding — its contributor rules outlived the plan:**
>
> - *"Do not silently ignore privileged operation failures"* is exactly the rule that
>   `let _ = restore_from_snapshot(..)` violated in `apply.rs`. Fixed in `881ad57`.
> - *"ALWAYS run `bun run validate` before committing"* — still the gate.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the impactful Rust backend correctness issues found in the audit, then update the docs so future humans and AI agents can understand the app and avoid reintroducing drift.

**Architecture:** Keep the tweak/profile systems feature-complete by sharing one apply, registry conversion, snapshot, and validation path instead of maintaining partial duplicate implementations. Documentation changes are part of the acceptance criteria for every behavior change.

**Tech Stack:** Rust/Tauri, Svelte frontend integration where needed, YAML tweak definitions, Bun validation, Cargo tests/clippy, Markdown docs.

---

## File Structure

- Modify: `src-tauri/src/commands/tweaks/helpers.rs`
  - Command execution semantics, shared apply helpers if kept in this module.
- Modify or create: `src-tauri/src/services/registry_value.rs`
  - Single Rust module for registry JSON parsing, typed writes, typed reads, and comparison normalization.
- Modify: `src-tauri/src/services/profile/import.rs`
  - Remove duplicate partial apply logic and call the shared tweak apply path.
- Modify: `src-tauri/src/services/profile/validation.rs`
  - Implement hash fallback and permission validation.
- Modify: `src-tauri/src/services/profile/mod.rs`
  - Replace incomplete option hashing with canonical full-option hashing.
- Modify: `src-tauri/src/services/backup/detection.rs`
  - Validate stale snapshots across all captured resource types.
- Modify: `src-tauri/src/services/system_info_service.rs`
  - Add lightweight runtime context helpers.
- Modify: `src-tauri/src/commands/profile.rs`
  - Use lightweight Windows/admin context.
- Modify: `src-tauri/src/commands/tweaks/apply.rs`
  - Use lightweight context and strict pre-command semantics.
- Modify: `src-tauri/src/commands/tweaks/batch.rs`
  - Avoid repeated full WMI system-info calls.
- Modify: `docs/TWEAK_AUTHORING.md`
  - Keep tweak authoring behavior aligned with runtime.
- Modify: `docs/TWEAK_SYSTEM.md`
  - Update architecture notes to current code after fixes.
- Modify: `docs/PROFILE_SYSTEM.md`
  - Update profile guarantees, restore-point wording, hash fallback, validation behavior, and system-state behavior.
- Create: `docs/APP_CONTEXT.md`
  - Human/AI context document listing app purpose, features, architecture, safety model, commands, tweak/profile/snapshot systems, and validation commands.

## Task 1: Registry Value Normalization

- [ ] Write tests for `REG_BINARY` array values, comma-separated hex strings, whitespace-tolerant hex strings, invalid bytes, and equality between read byte arrays and authored hex strings.
- [ ] Add a focused registry value helper module that parses authored JSON into typed registry values.
- [ ] Update manual apply, profile apply, snapshot restore, and detection to use the same helper.
- [ ] Decide and document `REG_MULTI_SZ` support: either implement write support or make build-time validation reject authored write attempts.
- [ ] Run `cargo test registry` and `cargo clippy -- -D warnings`.

## Task 2: Profile Apply Parity

- [ ] Write tests proving profile apply handles every tweak option change type: registry, service, scheduler exact name, scheduler pattern, hosts, firewall, and command skip warnings.
- [ ] Remove `services/profile/import.rs` duplicate `apply_tweak_changes` logic.
- [ ] Extract or reuse the normal tweak apply engine so profile apply uses the same atomic rollback and elevation behavior as manual apply.
- [ ] Preserve the profile security rule intentionally: if profile apply skips pre/post commands, surface that as validation/apply warnings and document the difference clearly.
- [ ] Run profile import/apply tests and targeted tweak apply tests.

## Task 3: Restore Point and Snapshot Safety

- [ ] Write tests for profile apply with an existing internal snapshot to prove it is not overwritten.
- [ ] Replace misleading `create_restore_point` behavior with one of two explicit paths:
  - `create_internal_backup`: temporary rollback data stored separately from manual snapshots.
  - Real Windows restore point using `SRSetRestorePoint`, if implementing the planned feature now.
- [ ] Update frontend/API naming only if the public option changes.
- [ ] Update `PROFILE_SYSTEM.md` to distinguish Windows restore points, internal snapshots, and profile rollback data.

## Task 4: Stale Snapshot Validation

- [ ] Write tests for registry-only, service-only, scheduler-only, hosts-only, firewall-only, and mixed snapshots.
- [ ] Update `snapshot_matches_current_state` to compare every captured snapshot category.
- [ ] If a resource cannot be queried safely, keep the snapshot and log a warning instead of deleting it.
- [ ] Run backup service tests and startup validation tests.

## Task 5: Pre-Command Failure Semantics

- [ ] Write tests for shell and PowerShell helpers returning errors on non-zero exit codes.
- [ ] Change `run_command` and `run_powershell_command` to return `Err` for non-zero exits.
- [ ] Keep post-command behavior nonfatal in `apply.rs` by logging those returned errors without rollback.
- [ ] Update `TWEAK_AUTHORING.md` only if the final behavior intentionally differs from the documented fail-fast contract.

## Task 6: Profile Hash Fallback and Permission Validation

- [ ] Write tests for option reorder, option insertion, hash match, hash mismatch, and invalid hash cases.
- [ ] Implement canonical full-option hashing that includes all fields affecting behavior.
- [ ] Resolve out-of-range or moved options by matching stored hash before failing validation.
- [ ] Add admin/SYSTEM/TI validation warnings/errors before apply.
- [ ] Update `PROFILE_SYSTEM.md` with the actual hash and permission validation behavior.

## Task 7: Lightweight Runtime Context

- [ ] Add a lightweight context helper for Windows version, build number, and admin status without WMI hardware/device enumeration.
- [ ] Replace `get_system_info()` calls in profile/tweak apply, batch apply, batch revert, and `get_windows_version`.
- [ ] Keep full `get_system_info()` for the UI command that really displays full system details.
- [ ] Add timing-safe tests where possible, and verify no WMI-heavy calls remain in hot paths.

## Task 8: Documentation Synchronization

- [ ] Audit `docs/TWEAK_AUTHORING.md` against current/fixed runtime behavior:
  - `REG_BINARY` accepted formats.
  - `REG_MULTI_SZ` actual support.
  - pre/post command failure semantics.
  - profile apply command-skip behavior if kept.
  - scheduler pattern and `ignore_not_found` behavior.
- [ ] Audit `docs/TWEAK_SYSTEM.md`:
  - Correct the profile archive description.
  - Update module map and execution flow if shared apply/profile code moved.
  - Clarify what is atomic and what is best-effort.
- [ ] Audit `docs/PROFILE_SYSTEM.md`:
  - Correct option content-hash fallback behavior.
  - Correct permission validation behavior.
  - Correct `system_state.json` behavior.
  - Correct restore-point wording.
  - Clarify checksums as integrity checks, not cryptographic trust/signature.
- [ ] Run a docs-only review for stale claims after all code changes land.

## Task 9: App Context Document

- [ ] Create `docs/APP_CONTEXT.md`.
- [ ] Include a concise app overview: MagicX Toolbox is a Windows/Tauri/Svelte app for applying reversible system tweaks from embedded YAML definitions.
- [ ] Document major systems:
  - Tweak definitions and option model.
  - Registry/service/scheduler/hosts/firewall changes.
  - Command and PowerShell hooks.
  - Elevation model: user, admin, SYSTEM, TrustedInstaller.
  - Snapshot, rollback, stale cleanup.
  - Profile export/import archive.
  - System inspection/debug commands.
  - Update flow.
  - Frontend stores and main UI primitives.
- [ ] Include safety rules for future contributors:
  - Prefer existing apply/validation helpers.
  - Do not duplicate system-change logic.
  - Do not silently ignore privileged operation failures.
  - Always update `TWEAK_AUTHORING.md` when tweak behavior changes.
  - Always run `bun run validate` before commit.
- [ ] Add links to `TWEAK_AUTHORING.md`, `TWEAK_SYSTEM.md`, `PROFILE_SYSTEM.md`, `.github/copilot-instructions.md`, and key Rust modules.

## Task 10: Final Validation

- [ ] Run `bun install` if `node_modules/.bin/prettier` is missing.
- [ ] Run `bun run validate`.
- [ ] Fix all formatter, lint, type, Svelte, Rust fmt, and clippy issues.
- [ ] Run any new targeted tests added by this plan.
- [ ] Review `git diff` for unrelated churn.
- [ ] Commit by task with clear messages.

## Acceptance Criteria

- `REG_BINARY` authored values apply, restore, export/import, and detect consistently.
- Profile apply cannot silently skip supported core change types.
- Existing internal snapshots are never overwritten by profile apply rollback setup.
- Stale snapshot cleanup does not delete valid non-registry snapshots.
- Pre-command non-zero exit codes abort before core system changes.
- Profile validation resolves moved options by hash and reports permission problems before apply.
- Hot tweak/profile paths avoid full WMI system-info collection.
- `TWEAK_AUTHORING.md`, `TWEAK_SYSTEM.md`, and `PROFILE_SYSTEM.md` match the implementation.
- `docs/APP_CONTEXT.md` gives future AI/human contributors enough app context to work safely.
- `bun run validate` passes.
