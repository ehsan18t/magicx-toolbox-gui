# Effect-Layer Audit — registry / services / scheduler / cmd / powershell / TI broker

**Date:** 2026-07-20
**Scope:** the privileged effect layer — `services/elevation/*` (broker, SYSTEM/TI spawn, common) and the
effect services it drives (`registry_service`, `registry_value`, `service_control`, `scheduler_service`),
plus the capture/restore call sites that consume them.
**Method:** three partitioned finder lenses (correctness + simplification), then adversarial skeptics on
every correctness/safety finding (prove-real; broker silent-success run at max effort, prove-*safe*).
**Result:** 7 correctness/safety findings, **all CONFIRMED**; 5 dedup wins; 2 design footguns; 2 caller-side
leaks; 1 hardening note. A broad clean bill on the parts the recent refactor already fixed.

The headline: the refactor dissolved the *loud* classes (PowerShell injection, REG_SZ corruption, locale
parsing). What remains is a **subtler second layer of the same "did-it-work" theme** — the failure signal
exists but is dropped or misclassified at a few specific seams. And the duplication originally suspected is
real and **causally linked to at least two of the bugs**: the copy-pasted `read_*` fns are why `delete_value`
drifted (A1), and the duplicated spawn tail is where the C2 fix has to land. Simplify and fix are the same
work here, not competing.

---

## Confirmed correctness / safety findings (ranked)

Verification tag: **proof-confirmed** = skeptic reproduced/traced it against the code.

### 1. C1 — the broker can report SUCCESS for a privileged op that never ran  ·  HIGH · proof-confirmed (prove-safe)
`elevation/broker.rs:241` (+ `:216-219`, `main.rs`)

`run_elevated_broker` infers success solely from "a response file is readable + parseable." Three facts
combine:
- **No request↔response binding exists** anywhere in the elevation module — no nonce, request-id,
  timestamp, or arity check. `into_single` (`broker.rs:89`) just pops the *last* result.
- **The child exit code is discarded on the Ok path** — `spawn.and_then(|exit| read(resp_path))`; `exit`
  appears only inside `.map_err` (runs only when the *read* fails). `run_broker` deliberately returns
  `0` vs `2/3/4/5` to signal transport success/failure, and that signal is thrown away.
- **The resp path is predictable and never pre-cleared** — `magicx-broker-<pid>-<seq>-resp.json`, `seq`
  resets to 0 each process start; the only `remove_file` is *after* the read, best-effort.

**Repro (rare conjunction, but no guard prevents it):** PID reuse across app runs + a leftover
`…-<pid>-0-resp.json` (parent aborted under `panic="abort"` before cleanup, or best-effort remove failed) +
the new broker's `write(resp_path)` fails (exit 5) without overwriting → parent reads the *stale*
`{"results":["Ok"]}` → `Ok(())` for a SYSTEM/TI op that never executed.

**Fix:** pre-clear `resp_path` (or `OpenOptions::create_new`/`O_EXCL`) **before** spawn; **gate on
`exit == 0`** before trusting the file (the value is already in hand); ideally echo a per-request nonce in
the response and verify it. This also subsumes C2.

### 2. A1 — DeleteValue against an absent key fails loud across apply / revert / broker  ·  HIGH (fails-closed) · proof-confirmed
`registry_service.rs:434`

`delete_value`'s key-open arm folds `io::ErrorKind::NotFound` into `Error::RegistryAccessDenied`, while
`delete_key` (`:470`) correctly distinguishes them. All three "deleting an absent thing is success" shims —
broker `delete_ok` (`broker.rs:101`), direct apply (`helpers.rs:354`), revert (`restore.rs:182`) — match
**only** `RegistryKeyNotFound`, so the mislabeled `AccessDenied` propagates as a hard failure. A DeleteValue
whose target key isn't present on that machine aborts the apply; a revert of a "didn't exist originally"
value fails when its containing key was removed. (Only the absent-*key* sub-case leaks; absent-*value* is
handled right. Fails loud, not silent — a reviewer could argue Medium on that basis; kept High for breadth.)

**Fix:** mirror `delete_key`'s branch (`if e.kind()==NotFound { RegistryKeyNotFound } else { AccessDenied }`).
Permanently eliminated by the shared open helper in **S1**.

### 3. C3 — schtasks task names containing `& | < > ^ %` are corrupted  ·  MEDIUM · proof-confirmed
`elevation/common.rs:77` → `backup/restore.rs:300`

`escape_shell_arg` prepends carets (an *unquoted*-context escape) and doubles `%`, but the value is placed
**inside** `cmd`'s double-quotes (`/Change /TN "{escaped}" /Enable` → `cmd /c schtasks …`). Inside double
quotes carets are inert, so `Games & Apps\T1` → `Games ^& Apps\T1` reaches schtasks with a literal caret →
"task not found". Fails closed. Lives inside the known "schtasks still shell-string" gap, but it's a concrete
corruption of a *legitimate* task name, not merely the general gap.

**Fix:** migrate the schtasks path to a typed `BrokerOp::Scheduler`/argv like the rest and retire
`escape_shell_arg` (the documented follow-up). Interim: stop caret/percent-escaping for the quoted context.

### 4. B1 — start_service reports Ok without polling to RUNNING  ·  MEDIUM · proof-confirmed
`service_control.rs:283`

`StartServiceW` only queues the start (→ `START_PENDING`); it does not block until `RUNNING`. `start_service`
returns `Ok` on the queued start with no poll — while `stop_service` *does* poll (`wait_for_stop`), and the
codebase's own `start_trusted_installer_service` (`ti_elevation.rs:194`) polls for `SERVICE_RUNNING`. So a
service that enters `START_PENDING` then fails async init is reported as success. Fail-open and asymmetric.

**Fix:** after `StartServiceW` succeeds, poll `query_current_state` to `SVC_RUNNING` (Err on `STOPPED`/
timeout); keep `ERROR_SERVICE_ALREADY_RUNNING` idempotent.

### 5. A2 — capture aborts snapshotting a non-DWORD DeleteValue with no declared type  ·  MEDIUM · proof-confirmed
`backup/capture.rs:146`

`value_type` is explicitly optional and "ignored for delete" per the schema, so authors omit it. Capture
defaults the omitted type to `DWORD` and reads with it. Reading an existing REG_SZ/BINARY/QWORD value with a
DWORD reader returns `ERROR_BAD_FILE_TYPE` (222) → `Error::RegistryOperation` (skeptic corrected the finder's
"InvalidData" label) → `classify_read_result` propagates it (correctly *not* coerced to absent) → capture
aborts the apply and the rollback value is never recorded.

**Fix:** for capture (goal = record whatever is there), read the raw value generically
(`get_raw_value` → capture vtype + bytes); only Set-verification needs the declared type.

### 6. C2 — WAIT_FAILED / unchecked GetExitCodeProcess → bogus Ok(0)  ·  MEDIUM · proof-confirmed (prove-safe)
`system_elevation.rs:97,113` (mirror `ti_elevation.rs:371,387`)

Only `WAIT_TIMEOUT` (0x102) is special-cased; `WAIT_FAILED` (0xFFFFFFFF) and everything else fall through as
"completed." `GetExitCodeProcess`'s BOOL is unchecked and `exit_code` defaults to 0, so a wait failure returns
`Ok(0)`. Today the response-file read partially masks it (compounds C1); its real damage is that it **defeats
the natural C1 fix** — you can't gate on `exit == 0` while the spawn fabricates `0` on failure.

**Fix (land with C1):** match `wait_result` — `WAIT_OBJECT_0` proceed, `0x102` timeout-Err, else Err with
`GetLastError()`; check `GetExitCodeProcess`'s BOOL.

### 7. B2 — CoInitializeEx is never balanced by CoUninitialize  ·  LOW–MEDIUM (skeptic-downgraded) · proof-confirmed
`scheduler_service.rs:100`

No `CoUninitialize` exists anywhere; `task_service()` calls `CoInitializeEx` per-op, and scheduler ops run
**in-process** on the `Elevation::None` path (`detection.rs:195`, `compare.rs:327`, `capture.rs:549`,
`restore.rs:311`). On the main thread COM is already up (→ `S_FALSE`, no crash), so this is a slow per-thread
init/apartment imbalance, not a functional failure — hence the downgrade. The discarded HRESULT also conflates
`RPC_E_CHANGED_MODE` (must *not* uninit) with `S_OK` (must uninit).

**Fix:** init COM once per process/thread behind a guard, **or** capture the HRESULT and `CoUninitialize`
only on `S_OK`/`S_FALSE` (skip `RPC_E_CHANGED_MODE`), on the same thread.

---

## Simplification / dedup wins (finder-claims — concrete, low-risk; validate the "too much duplicate code" read)

- **S1 — registry read layer (highest value).** `read_dword/read_string/read_multi_string/read_binary/
  read_qword` (`registry_service.rs:17`, ~185 LOC) are five copies of the same open-key + NotFound/AccessDenied
  classification + `get_value` match. Collapse to one `open_read_key` + generic `read_typed<T: FromRegValue>`
  (~60 LOC). **This is the root cause of A1** — one audited classifier instead of six hand-copied ones. Reuse
  the same open helper (with an access-flag arg) for write/delete so NotFound is never conflated again.
- **S2 — registry writes.** Six `set_*` fns repeat the `require_write_access` + `create_subkey_with_flags` +
  `map_err` prologue (`registry_service.rs:231`) → `open_write_key` + `set_typed<T: ToRegValue>`. Also delete
  the test-local `key_exists` clone (`:541`) and call `super::key_exists`.
- **S3 — capture arms.** `Set`|`DeleteValue` and `DeleteKey`|`CreateKey` are byte-identical pairs
  (`capture.rs:121`), and `capture_current_state` (`:383`) re-implements the body a third time → `|` patterns
  + one shared `registry_value_snapshot` helper.
- **S4 — scheduler.** `enable_task`/`disable_task` are byte-identical but for `VARIANT_TRUE`/`FALSE`
  (`scheduler_service.rs:134`) → `set_task_enabled(bool)`; the `task_service()→GetFolder→GetTask` triplet
  repeats 4× → `resolve_task` helper (caller picks the not-found policy).
- **S5 — elevation spawn.** The ~35-line wait/timeout/terminate/GetExitCode/close-handles tail is duplicated
  verbatim between `spawn_as_system` and `spawn_as_trusted_installer` → `wait_and_reap(&PROCESS_INFORMATION)`
  in `common.rs`. **This is where the C2 fix lands** — one site instead of two divergent ones.

## Design footguns (notes)

- **D1 — vestigial `Result<i32>`.** `run_command_as_*`/`run_powershell_as_*`/`run_schtasks_as_*` all end in
  `.map(|()| 0)`, so callers' `exit_code != 0` checks (`helpers.rs:63`, `restore.rs:303`) are **dead code** —
  the real signal is `Ok`/`Err`. Change to `Result<()>`, drop the dead checks, fix three stale doc sites
  (`system_elevation.rs:28`, `ti_elevation.rs:248`, `mod.rs:19`'s `cmd /c cmd /c echo` example).
- **D2 — did-it-work hidden in a tuple.** `apply_action_to_pattern` returns `(success, error_count, errors)`
  (`scheduler_service.rs:240`); the sole caller checks it, but any future `let (_,_,_) = …?` treats
  all-tasks-failed as success. Return `Err` when `error_count == tasks.len()`, or a typed outcome.

## Caller-side did-it-work leaks (off-slice — own pass)

- **N1** — `start_service` results discarded with `let _ =` at `helpers.rs:606/609/612` and
  `restore.rs:269/271`. Even after B1, start failures are swallowed at these call sites.
- **N2** — `capture.rs:425` `if let Ok(matching_tasks) = find_tasks_by_pattern(…)` swallows a scheduler read
  error in `capture_current_state` (existed-state silently omitted).

## Hardening (low)

- **H1** — `enable_debug_privilege` (`common.rs:92`) enables `SeDebugPrivilege` process-wide and never drops
  it. Low risk (host is admin), but scoping it to the token-dup window is cheap.

---

## Clean bill (verified solid — do not touch)

- **PowerShell** `-EncodedCommand` is correct UTF-16LE + RFC-4648 base64; no lingering escaped `-Command` in
  the elevated path. RawCmd/PowerShell inputs are build-time author-trusted (`helpers.rs:59`) — **no injection.**
- **Broker command-line quoting** quotes the exe + both temp paths — spaces in the exe/`%TEMP%` path are safe.
- **Firewall** composes netsh via `Command::args` (argv, no shell) — not the broker RawCmd path.
- **Services** — `ScHandle` RAII (bound to `_scm`/`_s`, not `_`), single `open_service` helper, dependent-stop
  fails-closed (`ERROR_DEPENDENT_SERVICES_RUNNING`), `EnumDependentServicesW` two-call sizing correct.
- **Spawn** — `STARTUPINFOEXW`/attribute-list setup correct; token/process/thread handles +
  `DeleteProcThreadAttributeList` released on every non-panic error path (acceptable under `panic="abort"`).
- **Value-type fidelity** — EXPAND_SZ single-null, MULTI_SZ, BINARY hex round-trip, DWORD overflow guard,
  `%VAR%` preserved unexpanded.
- **HKCU/HKLM routing** — only HKLM+system goes through the broker; HKLM+!admin is rejected, not silently
  written; no re-spawn recursion.
- **Capture-path did-it-work reads** — `classify_read_result` + `key_exists` correctly propagate
  `AccessDenied` (the ADR-0003 hazard is guarded in the capture path).
- **Scheduler not-found** — `is_not_found` distinguishes `0x80070002/3` from real COM failures; broker
  fails-closed (no response file written on abort → parent's read errors).

---

## Recommended sequencing (a focused "effect-layer hardening" pass)

Correctness and dedup are the same work — sequence so each dedup carries its bug fix:

1. **S1 + A1** — collapse the `read_*` layer into `open_read_key` + `read_typed<T>`; the shared open helper
   fixes `delete_value` by construction. (Highest value: −125 LOC *and* kills a High finding.)
2. **C1 + C2 + S5** — extract `wait_and_reap`, gate the broker on `exit == 0`, pre-clear the resp path, add a
   per-request nonce. Fixes the only silent-success class and its compounding wait bug in one place.
3. **A2** — capture reads raw values generically.
4. **B1** — poll `start_service` to RUNNING.
5. **C3** — migrate schtasks to a typed `BrokerOp::Scheduler`/argv; retire `escape_shell_arg`.
6. **B2 + S4** — COM init guard + scheduler dedup.
7. Sweep **D1/D2** (tighten return types) and **N1/N2** (caller-side `let _ =`), **H1** hardening.

This is a natural **Stage 3b** — it closes the remaining did-it-work seams the Stage-3 contract was meant to
guarantee, and it delivers the duplication cut originally asked for. All 7 correctness items are regression-
testable on the existing HKCU scratch harness except the broker spawn (the accepted cargo-test gap).
