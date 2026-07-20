# Stage 3 — Effect Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the five effect services (registry, service, scheduler, hosts, firewall) one shared, truthful "did it work" contract, and introduce the `Elevation` seam — so a failed privileged operation surfaces as `Err`, and the elevated path is a single swap-point the TI broker can later fill.

**Architecture:** Replace locale-dependent shell-out (`sc.exe`/`net.exe`) and error-swallowing (`.unwrap_or(false)`, discarded exit codes) with typed Win32 (`windows-sys` for services, `windows` COM for Task Scheduler) that returns real results. Introduce `enum Elevation { None, System, TrustedInstaller }` behind a thin `ElevatedExecutor` seam; the elevated arm keeps today's mechanism temporarily but stops discarding its exit codes. The TI stage (separate branch/commit) later replaces that arm with the broker.

**Tech Stack:** Rust, `windows-sys` (already a dep, `Win32_System_Services`), `windows` crate (new, COM: `Win32_System_TaskScheduler`), `winreg` (kept), the Stage-2 real-registry test harness (`HKCU\Software\MagicXToolboxTest`).

## Global Constraints

- Rust edition floor **1.92**; CRLF is an enforced repo property.
- `panic = "abort"` on `[profile.release]` — `Drop` does **not** run on panic. Use RAII handle guards anyway (they cover normal + `?`-early-return paths), but do not rely on them for panic safety.
- **Keep the four public `service_control.rs` signatures byte-identical** — ~20 call sites across 7 files must not change.
- `winreg` **stays**. Distinguish `NotFound` from `PermissionDenied` via `io::ErrorKind`; never coerce with `.unwrap_or(false)`.
- `SC_HANDLE`/`HANDLE` are pointers — check `.is_null()`, never `== 0`.
- Idempotent successes are **not** failures: `ERROR_SERVICE_NOT_ACTIVE`, `ERROR_SERVICE_ALREADY_RUNNING`.
- `EnumDependentServicesW` must run before a stop (today's `net stop` stops dependents; `ControlService` does not — losing it is a silent regression).
- `Elevation` must make the illegal state `(use_system=false, use_ti=true)` unrepresentable.
- Tests drive the **real** registry under `HKCU\Software\MagicXToolboxTest` with per-test unique subkeys and `Drop` cleanup — no mocks. Service/scheduler end-to-end needs an elevated runner, so their honest unelevated ceiling is nonexistent-name + parser fixtures.
- Verify with `cd src-tauri && cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`. Priority order: **correctness first, then less code, then optimized.**
- Each task commits on the `stage-3-effect-layer` branch; the whole stage is squash-merged into one commit at the end. **Do not push.** Do not `git add` the untracked `docs/superpowers/specs/` TI spec — it travels with the TI commit.

---

## Work-package order (each is independently testable)

1. **WP1 — `Elevation` enum + `ElevatedExecutor` seam.** Pure refactor, no new deps, testable by construction (illegal state won't compile). Unblocks TI. Collapses the `if use_ti/else if use_system/else` triad (~8 sites).
2. **WP2 — Registry did-it-work contract.** Fix `capture.rs:588` (access-denied → `existed:false`) and the eight `.unwrap_or(false)` winreg sites. Testable **now** with the Stage-2 harness. Blocker for ADR-0003.
3. **WP3 — `service_control.rs` → `windows-sys`.** The big one. Keep the four signatures; handle the SCM gotchas. Testable via nonexistent-name + config round-trip on a benign service.
4. **WP4 — Task Scheduler → `windows` COM.** `TASK_STATE` numeric enum kills the locale class. Adds the `windows` crate.
5. **WP5 — Firewall exit-status.** Lowest priority (Stage-2 cleared its injection); just stop ignoring the `netsh` exit status.

WP1 and WP2 are the front-loaded, must-land pair (testable, TI-unblocking). WP3/WP4 are the heavy FFI. WP5 is a small correctness patch.

---

## WP1 — `Elevation` enum + `ElevatedExecutor` seam

**Files:**
- Create: `src-tauri/src/services/elevation/level.rs` (the enum + seam)
- Modify: `src-tauri/src/services/elevation/mod.rs` (export `Elevation`, `ElevatedExecutor`)
- Modify: `src-tauri/src/commands/tweaks/helpers.rs` (replace the ~8 `if use_ti/else if use_system/else` sites with a single `match elevation`)
- Modify: `src-tauri/src/models/tweak.rs` (add `fn elevation(&self) -> Elevation` deriving from `requires_system`/`requires_ti`)
- Test: co-located `#[cfg(test)]` in `level.rs`

**Interfaces:**
- Produces: `enum Elevation { None, System, TrustedInstaller }`; `impl Elevation { fn from_flags(requires_system: bool, requires_ti: bool) -> Elevation; fn label(&self) -> &'static str }`; `trait ElevatedExecutor { fn run(&self, command: &str) -> Result<i32, Error>; }` with today's SYSTEM/TI executors adapted behind it. `TweakOption`/`TweakDefinition::elevation() -> Elevation`.

- [ ] **Step 1: Write the failing test** — `from_flags` maps correctly and cannot represent `(false, true)` as anything but TrustedInstaller.

```rust
#[test]
fn from_flags_maps_the_three_reachable_states() {
    assert!(matches!(Elevation::from_flags(false, false), Elevation::None));
    assert!(matches!(Elevation::from_flags(true, false), Elevation::System));
    assert!(matches!(Elevation::from_flags(true, true), Elevation::TrustedInstaller));
    // requires_ti implies requires_system: (false, true) is nonsense input and
    // must still resolve to TrustedInstaller, never a hybrid.
    assert!(matches!(Elevation::from_flags(false, true), Elevation::TrustedInstaller));
}
```

- [ ] **Step 2: Run to verify it fails** — `cd src-tauri && cargo test from_flags_maps` → FAIL (`Elevation` undefined).
- [ ] **Step 3: Implement the enum**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation { None, System, TrustedInstaller }

impl Elevation {
    pub fn from_flags(requires_system: bool, requires_ti: bool) -> Self {
        if requires_ti { Elevation::TrustedInstaller }
        else if requires_system { Elevation::System }
        else { Elevation::None }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Elevation::None => "User",
            Elevation::System => "SYSTEM",
            Elevation::TrustedInstaller => "TrustedInstaller",
        }
    }
}
```

- [ ] **Step 4: Run to verify it passes.**
- [ ] **Step 5: Add the `ElevatedExecutor` seam** — adapt the existing `execute_command_as_system` / `execute_command_as_trusted_installer` behind one trait so callers select by `Elevation`, not by two booleans. `Elevation::None` runs unelevated (existing normal path). The bodies are unchanged today; only the dispatch is unified.
- [ ] **Step 6: Collapse the triad in `helpers.rs`** — read the file, replace each `if use_ti { ... } else if use_system { ... } else { ... }` with `match tweak.elevation() { … }`. Remove the paired `_as_ti`/`_as_system` call duplication.
- [ ] **Step 7: Run `cargo test` + `cargo clippy` — green.**
- [ ] **Step 8: Commit** — `git add src-tauri/src/services/elevation/level.rs src-tauri/src/services/elevation/mod.rs src-tauri/src/commands/tweaks/helpers.rs src-tauri/src/models/tweak.rs && git commit -m "stage3: Elevation enum + ElevatedExecutor seam; collapse the triad"`

---

## WP2 — Registry did-it-work contract

**Files:**
- Modify: `src-tauri/src/services/backup/capture.rs:588` (stop mapping access-denied → `existed: false`)
- Modify: the eight `.unwrap_or(false)` winreg sites (grep `unwrap_or(false)` under `src-tauri/src/services/` and `commands/tweaks/`)
- Modify: `src-tauri/src/services/registry_service.rs` and/or `registry_value.rs` (return `Result` distinguishing `NotFound` vs `PermissionDenied`)
- Test: extend `src-tauri/src/services/backup/roundtrip_tests.rs`

**Interfaces:**
- Produces: registry read helpers that return `Result<Option<T>, Error>` where `Ok(None)` = verified-absent (`ErrorKind::NotFound`) and `Err(_)` = could-not-determine (`PermissionDenied` or other). No boolean coercion of an access error.

- [ ] **Step 1: Write the failing test** — capturing a value under a key the process cannot read must **not** record `existed: false`. (Simulate with a read helper returning `PermissionDenied`; assert the capture path yields an error/uncertain, never a confident "absent". Access-denied on real HKLM keys is the field case; unit-test the mapping directly since CI is unelevated.)

```rust
#[test]
fn access_denied_read_is_not_recorded_as_absent() {
    // maps ErrorKind::PermissionDenied -> Err, ErrorKind::NotFound -> Ok(None)
    assert!(classify_read(std::io::ErrorKind::PermissionDenied).is_err());
    assert!(matches!(classify_read(std::io::ErrorKind::NotFound), Ok(None)));
}
```

- [ ] **Step 2: Run to verify it fails.**
- [ ] **Step 3: Implement `classify_read` and route all reads through it**; replace each `.unwrap_or(false)` with propagation.
- [ ] **Step 4: Run to verify it passes.**
- [ ] **Step 5: Regression** — the Stage-2 round-trip (`capture → apply → detect → restore`) still passes; add one asserting a never-existed value is deleted, not zeroed, when the read is a true `NotFound`.
- [ ] **Step 6: `cargo test` + `cargo clippy` green.**
- [ ] **Step 7: Commit** — `git commit -m "stage3: registry did-it-work contract; stop coercing access-denied to absent"`

---

## WP3 — `service_control.rs` → `windows-sys`

**Files:**
- Rewrite internals: `src-tauri/src/services/service_control.rs` (keep the four public signatures)
- Test: co-located `#[cfg(test)]` + nonexistent-name fixtures

**Interfaces:**
- Consumes: nothing from WP1/WP2. Independent.
- Produces: the same four public fns (query startup / set startup / start / stop) with unchanged signatures, now returning truthful `Result`s.

Concrete requirements (each becomes 1-2 TDD steps; write the test for the nonexistent-name and idempotent-success cases first, then the FFI body):
- [ ] **Open SCM + service** via `OpenSCManagerW`/`OpenServiceW`; RAII guard closing `SC_HANDLE` on drop; `.is_null()` checks.
- [ ] **Set startup type** via `ChangeServiceConfigW` passing `SERVICE_NO_CHANGE` for every field except `dwStartType`, and **NULL (not `""`)** for unchanged strings.
- [ ] **Query startup type** via `QueryServiceConfigW` double-call sizing into a **pointer-aligned** buffer.
- [ ] **Start** via `StartServiceW`; treat `ERROR_SERVICE_ALREADY_RUNNING` as success.
- [ ] **Stop** via `EnumDependentServicesW` first, then `ControlService(SERVICE_CONTROL_STOP)`; poll `SERVICE_STATUS_PROCESS.dwCurrentState` for `STOP_PENDING → STOPPED` using `dwCheckPoint` progress; treat `ERROR_SERVICE_NOT_ACTIVE` as success.
- [ ] **Tests:** nonexistent service name → typed `Err` (not panic); set→query startup round-trip on a benign, always-present service that is safe to leave unchanged (query-only where mutation needs admin); idempotent start/stop returns `Ok`.
- [ ] **Commit** — `git commit -m "stage3: service_control on windows-sys (typed, locale-free, dependents-aware)"`

*Note:* the crate research **rejected** Mullvad's `windows-service` (its `change_config` wipes `account_password`). Do not reintroduce it.

---

## WP4 — Task Scheduler → `windows` COM

**Files:**
- Rewrite internals: `src-tauri/src/services/scheduler_service.rs`
- Modify: `src-tauri/Cargo.toml` (add `windows` with `Win32_System_TaskScheduler`, `Win32_System_Com`)
- Test: co-located fixtures

**Interfaces:**
- Produces: query/enable/disable/delete by exact name and by `regex`-`lite` pattern, unchanged signatures.

- [ ] **Add the `windows` crate** (features: `Win32_System_TaskScheduler`, `Win32_System_Com`, `Win32_Foundation`). Coexists with `windows-sys`.
- [ ] **`CoInitializeEx` per call/thread**; `ITaskService::Connect`; `GetFolder(task_path)`.
- [ ] **Query state** via `IRegisteredTask::State()` → `TASK_STATE` (numeric enum — the locale fix). `IRegisteredTaskCollection` is **1-indexed**.
- [ ] **Enable/disable** via `IRegisteredTask::put_Enabled`; **delete** via `ITaskFolder::DeleteTask`.
- [ ] **Pattern path** compiles with `regex-lite` (runtime), matching `GetTasks` names.
- [ ] **Tests:** nonexistent task + `ignore_not_found` semantics; a state read against a known stock task (query-only).
- [ ] **Commit** — `git commit -m "stage3: Task Scheduler on windows COM (TASK_STATE numeric, locale-free)"`

---

## WP5 — Firewall exit-status

**Files:**
- Modify: `src-tauri/src/services/firewall_service.rs:25` (stop ignoring the `netsh` exit status)
- Test: extend the Stage-2 firewall arg-building tests with an exit-status assertion

- [ ] **Step 1:** test that a non-zero `netsh` exit surfaces as `Err` (inject via a small seam or assert the status-check branch). Stage-2 already proved the argv path is injection-safe, so this is purely the dropped-status fix.
- [ ] **Step 2-4:** map non-zero exit → `Err`; keep argv construction unchanged; green.
- [ ] **Commit** — `git commit -m "stage3: surface netsh exit status instead of silently no-opping"`

---

## Self-review notes

- **Spec coverage:** WP1↔"collapse elevation duplicates + triad"; WP2↔`capture.rs:588` + `winreg` sites; WP3↔`service_control`→`windows-sys` gotchas; WP4↔scheduler COM; WP5↔`firewall_service.rs:25`. Deferred to their own stages (not Stage 3): detection+inspection collapse (Stage 4), the ADR UI work (Stage 5), the elevated **broker** (TI stage — WP1 leaves the seam for it).
- **Not in scope:** rewriting the token-dup / parent-spoof FFI (correct per research); replacing `winreg` (it already distinguishes the error kinds).
- **The seam is the Stage-3 ↔ TI contract:** WP1's `ElevatedExecutor` is what the TI broker swaps into. Stage 3 must not delete the elevated executors outright — only wrap them — or the branch won't build.
