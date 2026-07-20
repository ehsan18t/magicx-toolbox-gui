# Tweak System Remediation — Roadmap

The authoritative stage-by-stage tracker for the tweak-system remediation. This is the map; the detail
lives elsewhere and is linked per stage:

- [TWEAK_SYSTEM_REVIEW.md](./TWEAK_SYSTEM_REVIEW.md) — the 47-finding audit
- [TWEAK_SYSTEM_PLAN.md](./TWEAK_SYSTEM_PLAN.md) — reasoning, crate research, contradiction resolutions, stage-2 outcome
- [adr/](./adr/) — the four design decisions taken deliberately
- [spec/profile-v1.md](./spec/profile-v1.md) — the deleted profile format, preserved for the rebuild

**Guiding priorities, in order:** correctness first, then less code, then optimized. Where these conflict,
the conflict is named rather than silently resolved.

---

## Status at a glance

| Stage | What | Status |
| --- | --- | --- |
| 0 | Critical rollback fix + `serde_yml` swap + build determinism | ✅ done (PR #31) |
| 1 | Delete: profile system + dead code (~2,830 LOC) | ✅ done (PR #31) |
| 2 | Test seams + CI (57 → 75 tests) | ✅ done (this branch) |
| 3 | Effect layer: one "did-it-work" contract + API swaps | ⏳ next |
| 3a | Elevation module (SYSTEM / TrustedInstaller) | 🔬 researching |
| 4 | Collapse duplication (detection+inspection, build schema, loader) | ▫ planned |
| 5 | Design fixes: the ADRs (Needs Attention, atomic rollback, hosts, snapshot identity) | ▫ planned |
| 6 | UI: System Default selectable, Needs Attention surfaced | ▫ planned |
| 7 | Docs rewritten against what exists | ▫ planned |
| — | YAML corpus fixes (independent) | ▫ unblocked, can start anytime |
| L | Later: profile rebuild, comprehensive logging | ▫ deferred |

**Ordering rule that is easy to get wrong:** stage 4 (collapse detection+inspection) must precede stage 5.
"Needs Attention" is a new status state; adding it while detection and inspection are still two divergent
implementations means implementing it twice — and they already disagree.

---

## ✅ Stage 0 — Critical fixes (done, PR #31)

- `apply.rs` no longer discards the rollback result and deletes the snapshot regardless. A snapshot is
  released only on a verified-complete rollback; otherwise it is kept and every unrestored resource
  reported. The decision is a pure `classify_rollback` with 6 tests. Implements ADR-0001, ADR-0002.
- `serde_yml` → `serde_yaml_bw` (RUSTSEC-2025-0068: unsound, archived). Verified byte-identical output.
- `build.rs` emits `tweaks.json` deterministically (`HashMap` → `BTreeMap`).
- CRLF enforced as a repo property.

## ✅ Stage 1 — Delete (done, PR #31)

- Profile system removed entirely (~2,169 LOC, 7 commands, `zip`/`sha2`/`hex`/`hostname`). UI kept but
  disabled. Format recorded in [spec/profile-v1.md](./spec/profile-v1.md) first.
- Dead code removed (~660 LOC): `models/backup.rs`, the second YAML pipeline in `models/tweak.rs`,
  `aliases`, 8 never-invoked Tauri commands, 4 unused `Error` variants.
- A drift guard added: embedded tweak data must deserialize into the runtime types.

## ✅ Stage 2 — Test seams + CI (done, this branch)

Tests 57 → 75, driving the **real** registry under an HKCU scratch subtree rather than a mock. `AppHandle`
removed from the apply chain (a `OnceLock` in `debug.rs`). Round-trips, hosts parsing, firewall arg
building, and the detection↔inspection divergence all pinned. Accepted gaps and two audit corrections
recorded in [TWEAK_SYSTEM_PLAN.md § Stage 2 outcome](./TWEAK_SYSTEM_PLAN.md).

---

## ⏳ Stage 3 — The effect layer's success contract

**The root-cause stage.** Six confirmed findings are one bug wearing different clothes: an error converted
into a benign-looking value, so a failed privileged operation reports success and rollback never fires.

| Site | Error becomes |
| --- | --- |
| `capture.rs:588` | access-denied → `existed: false` → **revert deletes a value that existed** |
| `detection.rs:234` | read failure → "missing" → `*_missing_is_match` → inferred MATCH |
| `ti_elevation.rs:468` | exit code returned, caller discards it |
| `firewall_service.rs:25` | netsh exit status ignored → create silently no-ops |
| `restore.rs:388` | provably-failed restore returns `Ok(())` → snapshot deleted |
| `service_ops.rs:89` | `net.exe` exit 2 treated as success |

**Goal:** one shared definition of "did it work" across the five effect services, replacing five different
ones. Until this lands, ADR-0001 and ADR-0002 are decorative.

**Tasks**
- `service_control.rs` → raw `windows-sys` (already a dependency; kills locale-dependent `sc.exe`/`net.exe`
  string parsing). Keep the 4 public signatures so callers do not change. Handle the gotchas already
  catalogued in the plan (`SERVICE_NO_CHANGE`, `QueryServiceConfigW` sizing, `STOP_PENDING` polling,
  `EnumDependentServicesW`).
- Task Scheduler → `windows` crate COM (`TASK_STATE` is a numeric enum — the actual fix for the locale class).
- `capture.rs:588` → stop mapping access-denied to `existed: false`. **Blocker for ADR-0003.**
- `winreg` stays: it already distinguishes `NotFound` from `PermissionDenied`; the 8 `.unwrap_or(false)`
  call sites coerce that away. Our bug, not the crate's.
- Firewall: locale-dependent parsing is the only remaining driver (injection is disproven for that path —
  stage 2). Lower priority; hand-roll COM or fix the exit-status check, decide when we get here.
- Collapse the elevated duplicates and the `if use_ti / else if use_system / else` triad behind one
  `Elevation` enum.

**Verify:** the HKCU round-trip harness from stage 2 covers registry; extend the parser-fixture and
nonexistent-name patterns for services/scheduler. A failed elevated op must now surface as an `Err`.

**Unblocks:** stage 5 (the ADRs need truthful failure signals), stage 4's confidence.

## ⏳ Stage 3a — Elevation module (researched; scoped)

The owner was not confident the hand-written SYSTEM/TrustedInstaller elevation (~1,294 lines of `unsafe`
FFI) is correct. A three-probe research pass (crate landscape, safety audit, design alternatives) settled
it. Detail in [TWEAK_SYSTEM_PLAN.md § Elevation](./TWEAK_SYSTEM_PLAN.md); the verdict:

**Do not replace it with an elevation crate — none worth having exists.**
- crates.io has **zero** TrustedInstaller crates (verified). The whole landscape is a few C++/Go/twinBASIC
  PoCs; the current Rust code faithfully ports the de-facto reference (`nfedera/run-as-trustedinstaller`).
- Every "SYSTEM elevation" crate solves a *different* problem — interactive UAC-to-admin, or privilege
  *dropping* — none silently duplicate winlogon's token. `runas-rs` is also **GPL-3.0** (would infect a
  shipped product). Adopting any wrapper of this technique means trusting an unaudited single author with
  the ability to run code as TrustedInstaller — an unacceptable supply-chain trade.

**The hard parts are correct — this partly refutes the audit.** Token duplication, the parent-process
spoof (attribute list sized-then-initialised, parent handle outliving `CreateProcessW`,
`DeleteProcThreadAttributeList` on all paths), and handle/`SC_HANDLE` lifetimes are all sound. No active
leak was found; the audit's "unclosed handles on error paths" is largely **not** borne out. The real
weakness there is *fragility*: ~15 manual `CloseHandle` sites where one future early-return leaks.

**The genuine defects are all one systemic mistake: shell command strings.** Every elevated op is funnelled
through `cmd.exe /c <string>` with values escaped by the broken `escape_shell_arg` (`common.rs:72`, which
mixes caret-escaping and quote-doubling — two incompatible models, both wrong inside `cmd` quotes). That
one design yields:

| Sev | Site | Effect | Review # |
| --- | --- | --- | --- |
| **critical** | `ti_elevation.rs:92` | `\"`-escape + `cmd /c` → a `"`+`&` in a PowerShell script runs a separate command as SYSTEM/TI. Author-controlled today, so a *correctness* landmine with SYSTEM blast radius; a real vuln if user input ever reaches it (profile rebuild). | #27 |
| high | `system_elevation.rs:194` | REG_SZ with a space or `%VAR%` is double-mangled and silently corrupted | #26 |
| high | `ti_elevation.rs:468` + callers `helpers.rs:772,870` | schtasks exit code discarded (`.map(\|_\| ())`) → failed elevated task = success | #8 |
| medium | `system_elevation.rs:110`, `ti_elevation.rs:398` | `GetExitCodeProcess` return ignored, `WAIT_FAILED` not distinguished from `WAIT_TIMEOUT` → possible silent `Ok(0)` | new |
| medium | `common.rs:107` | `SeDebugPrivilege` enabled process-wide and never dropped | new |

**Scope (fits inside stage 3's contract — this IS how ops apply under SYSTEM/TI):**
1. **Delete the `cmd.exe /c` design and `escape_shell_arg`.** Registry and service ops (the bulk) call
   Win32 APIs directly with **typed** args (`RegSetValueExW`/`RegDeleteValueW`,
   `ChangeServiceConfigW`/`StartServiceW`/`ControlService`) in the elevated broker — dissolving the
   injection *and* corruption classes and turning exit codes into return values. This is literally stage
   3's "did-it-work contract" at the elevated layer.
2. PowerShell (the only irreducible interpreter case) → spawn `powershell.exe` directly as argv with
   `-EncodedCommand` (base64 UTF-16LE); no shell ever parses script bytes.
3. schtasks helpers map non-zero exit → `Err` at the source, so no caller can drop it.
4. Adopt the official **`windows`** crate (0.62, Microsoft, 272M downloads) for *this module only* — not as
   an elevation crate but for `Owned<HANDLE>` (CloseHandle-on-drop) + typed `Result`s, making the correct
   cleanup structural. Coexists with `windows-sys` elsewhere. Keep `to_wide_string` (an 8-line helper
   doesn't justify `widestring`); reject `sysinfo` (heavy) for the ToolHelp lookup.
5. Do **not** move to in-process `ImpersonateLoggedOnUser`: under `panic = "abort"` a panic before
   `RevertToSelf` leaks SYSTEM onto the thread. Keep the spawn-a-broker design.

Verified caveat on the `windows` crate: `Owned`'s `Drop` still won't run on panic (same as today), but it
covers the normal + `?`-early-return paths, which is strictly more than the manual pattern.

## ▫ Stage 4 — Collapse duplication

- **detection.rs + inspection.rs → one comparison core** (−331 LOC; the highest-value item). They disagree
  in four verified ways (REG_BINARY hex, `*_missing_is_match`, vacuous-empty-option, service errors). The
  stage-2 test `values_match_disagrees_with_registry_values_match_on_binary_hex_strings` is the red marker:
  when the collapse works, that test starts failing and should be deleted, not inverted.
- **Share the build schema** (−304 LOC): move pure types to `models/tweak_schema.rs`, `build.rs` includes
  only that. Drift becomes a compile error instead of a runtime panic.
- **Kill the loader clones**: `get_tweaks_for_version` deep-clones up to 189 tweaks per call; return
  `&'static` references.

**Must precede stage 5.**

## ▫ Stage 5 — Design fixes (the ADRs)

- Rollback always attempts all five phases, never aborts early (ADR-0001).
- **Needs Attention** state: retains its snapshot, names unrestorable resources, offers Retry and explicit
  "Keep current state". Enforces the ADR-0002 invariant (snapshot deleted only by verified restore or
  explicit consent).
- `tempfile` for atomic snapshot writes (verified `MoveFileExW` + `MOVEFILE_REPLACE_EXISTING`). Drop `fs4`
  for `std::fs::File::lock`.
- Hosts file: rewrite ~120 LOC by hand (no crate survives the requirements). Fixes the BOM, line-ending
  (`join("\n")` at `hosts_service.rs:210`), and multi-hostname bugs pinned in stage 2.
- **Snapshot identity** (ADR-0003 consequence): stamp snapshots with `MachineGuid`, warn on mismatch.
  **Prerequisite:** add `#[serde(default)]` + a schema version to `TweakSnapshot` FIRST (finding #18), or
  the added field breaks every existing snapshot.

## ▫ Stage 6 — UI

- System Default made selectable whenever `has_backup` (ADR-0003). `TweakCard.svelte:176` currently only
  unstages; the dropdown entry is `disabled` and vanishes once any option is applied.
- Needs Attention surfaced with its Retry / Keep-current-state actions.

## ▫ Stage 7 — Docs

`TWEAK_SYSTEM.md`, `TWEAK_AUTHORING.md`, `APP_CONTEXT.md` updated together against final behaviour:
atomicity wording per ADR-0001 (*attempted atomically, with failure surfaced* — not a guarantee), System
Default, Needs Attention, snapshot location, the full five-phase detection algorithm.

---

## ▫ Independent track — YAML corpus

Unblocked by everything above; can start anytime. From the review:

- **`legacy_network_protocols` silently re-enables SMBv1** in two options while its name/description/info
  mention only LLMNR/WPAD/NetBIOS. **Treat as urgent** — a live security-relevant data bug needing no
  refactor.
- 98 "(Default)" options write explicit Group Policy values that stock Windows ships *absent*.
- Asymmetric toggles where option[1] does not undo everything option[0] does.
- `mouse_input_mode` options 1 and 3 are byte-identical (option 3 unreachable).
- Two gaming tweaks overwrite the same `DirectXUserGlobalSettings` string, erasing each other.

## ▫ Deferred — Later

- **Profile system rebuild** from scratch, sharing one machine-identity mechanism with the snapshot install
  ID. Read [spec/profile-v1.md](./spec/profile-v1.md) first — `.mgx` archives from 3.0.0 are recoverable
  only if the option content hash is reproduced exactly.
- **Comprehensive logging** users can attach to bug reports. `tauri-plugin-log` stays (it does file logging
  with rotation); fix the dead `TargetKind::Webview` target and add `LogDir`.

---

## Dependency ledger (net so far and planned)

| Removed | Added |
| --- | --- |
| `serde_yml` (RUSTSEC) → `serde_yaml_bw` | — |
| `zip`, `sha2`, `hex`, `hostname` (with profiles) | — |
| planned: `fs4` → `std::fs::File::lock` | planned: `windows` (COM), `tempfile` |
| planned: `reqwest` → `ureq` (−28 crates) | planned: `schemars` (build/dev) |

Elevation crate decisions are pending stage-3a research.
