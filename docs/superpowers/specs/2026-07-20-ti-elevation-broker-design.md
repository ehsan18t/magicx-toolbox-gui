# Elevation Redesign — The Elevated Effect Broker

*Design spec. Status: proposed (awaiting review). Date: 2026-07-20.*

Supersedes the mechanism sketched in [ROADMAP.md § Stage 3a](../../ROADMAP.md) and
[TWEAK_SYSTEM_PLAN.md § Phase 1](../../TWEAK_SYSTEM_PLAN.md). It does **not** overturn that
research — it agrees with every major finding and makes the crossing mechanism concrete. See
[§ Comparison with Stage 3a](#comparison-with-stage-3a).

---

## Problem

The elevation module (~1,294 lines of `unsafe` FFI across `common.rs`, `system_elevation.rs`,
`ti_elevation.rs`, `service_ops.rs`) carries three pains that are **one knot**:

1. **Duplication.** Registry / service / scheduler each exist in up to three appliers — normal
   (native API), SYSTEM, TI — plus an `if use_ti / else if use_system / else` triad repeated at
   ~8 call sites.
2. **Unsafe sprawl.** ~15 manual `CloseHandle` sites; one future early-return leaks.
3. **Shell-string fragility.** Every elevated op is built into a `cmd.exe /c "<string>"` and
   escaped by the broken `escape_shell_arg` (`common.rs:72`), which mixes caret-escaping and
   quote-doubling. This yields a **critical** injection landmine (`ti_elevation.rs:92`), REG_SZ
   corruption (`system_elevation.rs:194`), and discarded exit codes (`ti_elevation.rs:468`).

The root of all three is the shell-string design: because the only way to "run something
elevated" is to spawn a shell as SYSTEM/TI, every op degrades to a string, every string needs
escaping, and every result degrades to an exit code that callers drop.

## Constraints (non-negotiable)

- **Correctness first**, then less code, then optimized (the project's stated order).
- **`panic = "abort"`** (release profile) — `Drop` does not run on panic. This forbids in-process
  `ImpersonateLoggedOnUser`: a panic before `RevertToSelf` leaks SYSTEM/TI onto the calling thread.
- **Keep SYSTEM and TI as two tiers** (least privilege). A `requires_system` tweak must not be run
  at TrustedInstaller.
- **The hard FFI is already correct** — token duplication (winlogon), the TI parent-process spoof
  (attribute list sized-then-initialised, parent handle outliving `CreateProcessW`,
  `DeleteProcThreadAttributeList` on all paths), and handle/`SC_HANDLE` lifetimes. The research
  found no active leak. **Reuse this code to spawn the broker; do not rewrite it.**
- No elevation crate: crates.io has zero TrustedInstaller crates; every "SYSTEM elevation" crate
  solves a different problem, and adopting one means trusting an unaudited author with code-as-TI.

---

## Core idea

> **The broker is nothing more than the effect services, running in an elevated process.**

There is no second, typed-Win32 reimplementation of registry/service/scheduler for the elevated
path. The elevated path runs **the same effect-service code** the unelevated path runs — it just
runs it inside a child process that holds the SYSTEM or TI token.

This works because the effect libraries are token-relative:
- `winreg` opening `HKLM\...\Services\WaaSMedicSvc` succeeds or fails based on **the process's
  token**. In a broker running as TI, the same `winreg` call that fails unelevated now succeeds.
- `windows-sys` service control (`OpenServiceW`/`ChangeServiceConfigW`/`ControlService`) is
  likewise gated by the caller's token.
- Task Scheduler COM is the same.

So "elevate this operation" becomes "run this operation in a process with the right token," not
"rebuild this operation as a shell string." The shell disappears.

## Architecture

```
┌─ main app (Administrator) ─────────────┐        ┌─ broker child (SYSTEM or TI) ──────────┐
│ apply engine                           │        │  magicx-toolbox.exe --broker           │
│   for each effect op:                  │        │                                        │
│     match elevation {                  │        │  read Request (BrokerOp[]) from stdin  │
│       None        => run in-proc ──────┼──┐     │  for each op: run the SAME effect fn ──┼─┐
│       System | TI  => serialize op ────┼─ │ ─▶  │  winreg / service_control / scheduler  │ │
│     }                                  │  │     │  write Response (OpResult[]) to stdout ◀┼─┘
│   spawn broker with the tier's token ──┼──┘     └────────────────────────────────────────┘
│   (winlogon dup | TI parent-spoof)     │              ▲ token chosen by how it's spawned
└────────────────────────────────────────┘
```

- **Broker entrypoint:** a `--broker` subcommand of the existing app binary (no second exe to
  ship/sign; re-exec self). It reads a `Request` from **stdin**, executes, writes a `Response` to
  **stdout**, exits. Stateless — one spawn per apply-batch.
- **Spawn strategy (the only tricky FFI, reused as-is):**
  - `System` → duplicate winlogon's token → `CreateProcessWithTokenW(token, …, "magicx-toolbox.exe --broker")`.
  - `TrustedInstaller` → start the TI service, get its PID, parent-process spoof → `CreateProcessW`.
  - Both wrapped in the `windows` crate's `Owned<HANDLE>` so cleanup is structural on the normal
    and `?`-early-return paths (still not on panic — same as today, but strictly better than manual
    `CloseHandle`).
- **No shell anywhere.** stdin/stdout carry structured bytes; `powershell.exe` is spawned by the
  broker as argv (never through `cmd`).

### The `Elevation` enum (replaces the triad and the two booleans)

```rust
enum Elevation { None, System, TrustedInstaller }
```

Derived once from the tweak's declared `requires_system` / `requires_ti`. Makes the illegal state
`(use_system=false, use_ti=true)` unrepresentable and collapses the ~8 `if use_ti / else if …`
sites to a single `match`.

### Request / response protocol

Internal, versioned, `serde`-encoded (JSON is fine — small, human-debuggable; the broker is our
own code, not an untrusted boundary). Sketch:

```rust
struct Request  { version: u32, ops: Vec<BrokerOp> }
struct Response { results: Vec<OpResult> }         // positional, 1:1 with ops

enum BrokerOp {
    RegSet   { hive: Hive, key: String, name: String, value: TypedRegValue },
    RegDelete{ hive: Hive, key: String, name: String },
    RegDeleteKey { hive: Hive, key: String },
    RegCreateKey { hive: Hive, key: String },
    SvcConfig{ name: String, startup: StartupType },
    SvcControl{ name: String, action: StartStop },
    Scheduler{ path: String, target: TaskTarget, action: TaskAction },
    Powershell{ encoded: String },   // base64 UTF-16LE, spawned as -EncodedCommand
    RawCmd   { command: String },    // author's literal pre/post_command, cmd /c <one argv elem>
}

enum OpResult { Ok, Err { message: String }, ExitCode(i32) }
```

`TypedRegValue` carries the real type (`Dword(u32)`, `Qword(u64)`, `Sz(String)`, `ExpandSz`,
`Binary(Vec<u8>)`, `MultiSz(Vec<String>)`) — so REG_SZ with spaces or `%VAR%` and REG_BINARY blobs
cross the boundary as **typed data**, never as a mangled string. This dissolves the corruption class.

### Broker scope

The broker handles only the ops that actually need SYSTEM/TI: **registry, service, scheduler**, and
the **command / PowerShell hooks** when the tweak declares elevation. **Hosts and firewall stay in
their own services** (`hosts_service` file I/O, `firewall_service` via `netsh` argv) — they require
admin, not SYSTEM/TI, and the Stage-2 audit already cleared the firewall argv path of injection.

### The two irreducible interpreter cases

- **PowerShell** (`pre/post_powershell`): broker spawns `powershell.exe -NoProfile -NonInteractive
  -EncodedCommand <base64 UTF-16LE>` as argv. No shell parses script bytes → the critical injection
  landmine is gone even for script content.
- **Author `pre/post_commands`**: these are *inherently* `cmd.exe` scripts written by the tweak
  author. Broker runs the author's whole string as a single argv element to `cmd.exe /c` — no
  `escape_shell_arg`, because we are not composing a command *around* untrusted data; the string
  *is* the author's command. Remains author-controlled exactly as today. (If profile rebuild ever
  routes user input here, that is a separate gate — noted, not solved here.)

## Result / error contract ("did it work")

Every op returns a real result. `RegSet` returns the `RegSetValueExW`/`winreg` status; `SvcControl`
polls `STOP_PENDING` and distinguishes `ERROR_SERVICE_NOT_ACTIVE` (idempotent success) from failure;
`Powershell`/`RawCmd` return the true process exit code. The broker serializes these into
`OpResult`s; the main app maps any `Err`/non-zero into the effect layer's `Err`, which is what makes
ADR-0001 (rollback) and ADR-0002 (snapshot retention) finally load-bearing at the elevated layer.
**No caller can discard an exit code**, because there is no bare `i32` to discard — the crossing is
typed end to end.

## Unsafe surface after the change

- **Kept (correct, reused):** the two spawn primitives — winlogon token dup and TI parent-spoof —
  plus `enable_debug_privilege`, `find_process_by_name`, `start_trusted_installer_service`. Wrapped
  in `Owned<HANDLE>`.
- **Deleted:** `escape_shell_arg`, all `cmd.exe /c` string construction for registry/service/
  scheduler, both `execute_command_as_*` string executors, `service_ops.rs`'s function-pointer
  indirection, the per-op `CreateProcess*` + wait/timeout/exit-code tails (now one place: the broker
  spawn + wait).
- **`SeDebugPrivilege`:** enable it in the broker child, not process-wide-and-forever in the main
  app (`common.rs:107`).

## Testing

- The broker request→response is pure data: unit-test op serialization and the effect functions
  directly (the Stage-2 HKCU-scratch harness already drives real registry unelevated; those same
  functions are what the broker runs).
- Broker round-trip: a test that spawns `--broker` **without** elevation (identity token) and feeds
  a request hitting the HKCU scratch subtree — exercises the full serialize → spawn → execute →
  deserialize path with no admin needed.
- Elevated end-to-end (service/scheduler under real SYSTEM/TI) stays a manual/CI-elevated check, as
  today — the honest ceiling without an elevated runner.

---

## Comparison with Stage 3a

The two designs **agree on every load-bearing decision**. "Reconsider from scratch" converged back
onto the Stage-3a research because that research was sound. The new design differs only in making
the mechanism concrete and in one structural simplification.

| Dimension | Stage 3a (as written) | This design | Delta |
|---|---|---|---|
| Use an elevation crate? | No — none worth having | No | **same** |
| Rewrite the token-dup / parent-spoof FFI? | No — it's correct | No | **same** |
| Shell strings for reg/svc/sched? | Delete them | Delete them | **same** |
| PowerShell | `-EncodedCommand` argv | `-EncodedCommand` argv | **same** |
| `windows` crate for `Owned<HANDLE>` | Adopt (this module only) | Adopt (spawn only) | **same** |
| In-proc impersonation? | No (`panic=abort` leak) | No (same reason) | **same** |
| Tiers | SYSTEM + TI kept | SYSTEM + TI kept | **same** |
| **How ops run elevated** | "call Win32 APIs directly with typed args **in the elevated broker**" — reads as a *second, raw-Win32 reimplementation* of each op | **Run the existing effect-service code** (`winreg`/`windows-sys`/COM) unchanged, inside the broker process | **new: one implementation, not two.** The broker reuses the unelevated ops; strictly less code and one behavior to test. |
| **How the request crosses** | unspecified (per-op spawn? args?) | **stateless structured stdin/stdout**, one spawn per apply-batch, `--broker` subcommand of the same exe | **new: concrete + stateless.** No named-pipe EoP surface, no lifecycle. |
| **Stage-3 ↔ TI seam** | bundled ambiguously into Phase 1 | **explicit:** Stage 3 builds the effect services + `Elevation` seam; TI stage wraps them in the broker | **new: no double implementation of the collapse.** |

**Bottom line:** this is Stage 3a, de-risked. The single insight that changes the code size is
*"the broker is the effect services in an elevated process,"* which removes the implied second
implementation and ties the elevation work cleanly to Stage 3's effect-layer contract.

## Rejected alternatives

- **Persistent broker + named-pipe IPC.** A long-lived TI-privileged pipe server is a standing
  elevation-of-privilege target (must ACL to the caller SID, validate every message) and adds
  lifecycle/crash-recovery. More moving parts — fights the goal. Only justified by high-frequency
  elevated ops, which user-initiated tweaks are not.
- **Per-op spawn passing typed argv (no structured protocol).** Kills injection too, but `argv` is
  awkward for `REG_BINARY`/`REG_MULTI_SZ`/multi-op batches, costs N spawns per tweak, and needs
  stdout parsing for real results anyway — i.e. it becomes this design with a worse encoding.
- **In-process `ImpersonateLoggedOnUser`.** Fewer processes, but `panic = "abort"` means a panic
  before `RevertToSelf` leaks SYSTEM/TI onto the thread. Rejected on safety.
- **Collapse to a single TI-only tier.** Simplest core, but every elevated op would run at max
  privilege and always pay the TI-service-start cost. Rejected to preserve least privilege.

## Scope & sequencing (the Stage-3 ↔ TI seam)

To avoid implementing the elevation collapse twice:

1. **Stage 3 (effect layer)** builds the effect services (`service_control` → `windows-sys`,
   scheduler → COM, the `capture.rs:588` fix, the `winreg` `.unwrap_or(false)` fixes) with the
   shared "did-it-work" contract, **and** introduces the `Elevation` enum plus a thin
   `ElevatedExecutor` seam. The elevated arm may temporarily keep today's mechanism behind the seam,
   but must stop swallowing its exit codes.
2. **TI stage (this design)** replaces the elevated arm with the broker — a drop-in behind the same
   seam, running the effect services Stage 3 already built. No rework of the collapse.

## Implementation notes (as built)

Deviations and decisions taken while implementing this design, recorded for accuracy:

- **Transport is temp files, not stdin/stdout.** The main app writes the `Request` to a temp file
  and passes `--broker <req> <resp>` as argv; the broker reads the request file and writes the
  response file. This avoids setting up inherited pipe handles across `CreateProcessWithTokenW` /
  the parent-process spoof (fiddly, error-prone FFI) while preserving every essential property: no
  shell parses anything, and the request *data* never appears on a command line — only our own
  generated temp-file paths do. One spawn per op batch, stateless.
- **The wrappers keep their signatures; only their bodies changed.** `run_command_as_*`,
  `run_powershell_as_*`, `run_schtasks_as_*`, `set_service_startup_as_*`, `stop/start_service_as_*`,
  and `set/delete_registry_value_as_system` now build a typed `BrokerOp` and call
  `run_elevated_broker`. `helpers.rs` was touched only to pass the typed `ServiceStartupType`
  (instead of the sc-string) — the apply chain is otherwise unchanged.
- **HKCU writes never go through the broker.** HKCU is the user's own hive and is always writable
  directly; running as SYSTEM would target SYSTEM's HKCU, not the user's. So a `requires_system`
  tweak's HKCU change writes directly (equivalent to the old `HKU\<SID>` reg.exe path), and only
  HKLM under `use_system` uses the broker. This sidesteps needing SID→`HKU` handling in the broker.
- **schtasks is transitional.** schtasks still crosses as a `RawCmd` (`cmd /c schtasks …`) inside the
  broker, so `escape_shell_arg` survives for that one path. Typed `Scheduler` ops exist in the
  protocol; wiring `helpers.rs`'s scheduler path to them (removing `escape_shell_arg` entirely) is
  the clean follow-up.
- **Deleted with the shell-string design:** `service_ops.rs`, both `execute_command_as_*` cmd
  wrappers, `get_current_user_sid`, `validate_registry_path`, `RegistryValue::reg_exe_data`. The
  token-dup / parent-spoof primitives (`spawn_as_*`) were kept unchanged — the research found them
  correct — and now launch the broker instead of `cmd.exe`.
- **Test ceiling.** The broker executor, the file-transport entrypoint, and the `None` path are
  tested unelevated against HKCU scratch. The real SYSTEM/TI *spawn* is not unit-tested: under
  `cargo test`, `current_exe()` is the test-harness binary, not the app binary that carries the
  `--broker` entrypoint. This matches the pre-existing elevation code, which never had spawn tests;
  the spawn reuses the unchanged, proven primitives.
