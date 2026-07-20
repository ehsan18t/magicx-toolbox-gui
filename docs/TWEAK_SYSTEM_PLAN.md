# Tweak System — Remediation Plan

Phased plan reconciling four inputs:

- [TWEAK_SYSTEM_REVIEW.md](./TWEAK_SYSTEM_REVIEW.md) — 47 audit findings (41 confirmed, 5 plausible, 1 refuted)
- [docs/adr/0001–0004](./adr/) — design decisions taken deliberately, with rejected alternatives
- Crate research — 29 evaluations, 13 of them rejections
- Code-reduction sweep — 33 items, **−2,223 net LOC**, 24 high-confidence

Goals, in the owner's priority order: **correctness first, then less code, then optimized.** Where those
conflict the conflict is named rather than silently resolved.

---

## Execution order

This table is authoritative. The work-package sections further down are reference detail.

| # | Stage | Needs tests first? | Why it sits here |
| --- | --- | --- | --- |
| **0** | Critical `apply.rs` patch + `serde_yml` swap | no | Active user harm and a security advisory. Small enough not to disturb the refactor. |
| **1** | **Delete** — dead code, 9 dead commands, the profile system (~2,800 LOC) | **no** | Deletion is compiler-verified: a still-referenced item is a *compile error*, not a runtime bug. Doing this first shrinks the surface stage 2 has to cover. |
| **2** | **Tests + CI** | — | Over the now-smaller surface. Nothing below is verifiable without it. |
| **3** | **Effect layer: decouple *and* swap APIs together** | yes | One "did it work" contract implemented on `windows-sys`/COM. Splitting these is pointless — a truthful contract built on lying `sc.exe` output is still a lie. |
| **4** | **Collapse duplication** — detection+inspection, build schema, loader clones | yes | Must precede stage 5: Needs Attention is a new status state, and adding it while detection and inspection are still two divergent implementations means implementing it twice. |
| **5** | **Design fixes (ADR-0001…0004)** + snapshot schema version + install ID | yes | |
| **6** | **UI** — System Default selectable, Needs Attention surfaced | yes | |
| **7** | **Docs**, written against what exists | — | Writing them earlier means writing them twice. |
| later | Features — profile system rebuild, comprehensive logging | — | |

**The key ordering insight: delete before you test.** Writing tests first means writing tests for code you are
about to throw away — and worse, it makes dead code *look* load-bearing. Four existing tests already exist
solely to exercise methods nothing calls.

The **YAML corpus track** is independent of all of this and can slot in anywhere; the SMBv1 finding should not
wait.

---

## Contradictions resolved

The two research passes disagreed in three places. Recording the resolutions, because each one would
otherwise be re-litigated.

### 1. `windows-service` crate — REJECTED, despite a −148 LOC saving

The reduction sweep recommended adopting Mullvad's `windows-service` (healthy: 5.6M downloads, released
2026-05-08, actively maintained). The crate research **read its source** and found a disqualifying flaw:

`Service::change_config()` never passes `SERVICE_NO_CHANGE` — the string does not appear in `src/service.rs`
at all. It requires a fully-populated `ServiceInfo` and writes every field. So changing *only* the startup
type means reconstructing every other field from `query_config()` first, and **`account_password` cannot be
read back from Windows at all.** Any service running under a named account would have its password wiped
and fail to start at next boot.

**Resolution: rewrite `service_control.rs` against `windows-sys` directly.** We already depend on it with
`Win32_System_Services` enabled, `elevation/common.rs` already calls these exact APIs, and it costs zero new
dependencies. We lose the sweep's −148 LOC and gain a correctness guarantee. Correctness wins.

Both passes independently flagged the same secondary hazard: `net stop` stops dependent services, the SCM
`ControlService` path does not. Whichever route we take, `EnumDependentServicesW` must be handled or we
silently regress today's behavior.

### 2. `build.rs` type sharing — use the sweep's design, not the crate research's

Crate research proposed `#[path = "src/models/tweak.rs"] mod tweak;` and verified it compiles. But its
scratch crate did not carry `build.rs`'s **own** impls — `validate()` and `requires_admin()` at
`build.rs:1040-1151`. Inherent impls must live in the same crate as the type, so including the whole file
pulls `models`' impls into `build.rs` alongside build's own, and they collide.

**Resolution: the sweep's version.** Move pure type definitions to `src/models/tweak_schema.rs`; `tweak.rs`
re-exports them and keeps every impl block where it is; `build.rs` includes only the schema file and keeps
its own impls. Same outcome, actually compiles.

### 3. YAML parser choice — `serde_yaml_bw`, not `serde_yaml_ng`

The crate research recommended `serde_yaml_ng` and rationalized its staleness as
*"'frozen' is an acceptable and arguably desirable property"* for a build-time parser. That reasoning is
defensible in isolation, but it was reached without `serde_yaml_bw` in the candidate set. With the full
picture, only one fork is actually alive:

| Crate | Latest | Last release | State |
| --- | --- | --- | --- |
| `serde_yaml_bw` | 2.5.6 | 2026-05-02 | 6 releases since Nov 2025 — actively developed |
| `serde_yaml_ng` | 0.10.0 | 2024-05-26 | ~2 years stale |
| `serde_norway` | 0.9.42 | 2024-12-21 | ~1.5 years stale |
| `serde_yml` | 0.0.13 | 2026-05-27 | that release *is* the deprecation notice; archived + RUSTSEC |
| `serde_yaml` | 0.9.34 | 2024-03-25 | deprecated upstream |

Verified before adopting: output is byte-identical across all 189 tweaks; the `deny_unknown_fields` error
still enumerates every valid field; and `value: null` still deserializes to `None`, so ADR-0004 is unaffected.

### 4. Snapshot location — three docs are wrong, not one

Review finding #39 caught `TWEAK_SYSTEM.md` claiming snapshots live in app data when the code writes them
next to the executable. `docs/APP_CONTEXT.md:52` carries the identical wrong claim. Fix all three
(`TWEAK_SYSTEM.md`, `APP_CONTEXT.md`, and the `storage.rs` doc comment) or pick a location and move the code
— but decide which, rather than fixing prose to match a location nobody chose.

---

## What the audit is really about

Two root causes account for over half the confirmed findings.

**Root cause A — errors converted into benign-looking values.** Six sites, one bug:

| Site | Error becomes |
|---|---|
| `capture.rs:588` | access-denied → `existed: false` → **revert deletes a value that existed** |
| `detection.rs:234` | read failure → "missing" → `*_missing_is_match` → **inferred MATCH** |
| `ti_elevation.rs:468` | exit code returned, caller discards it |
| `firewall_service.rs:25` | netsh exit status ignored → create silently no-ops |
| `restore.rs:388` | provably-failed restore returns `Ok(())` → snapshot deleted |
| `service_ops.rs:89` | `net.exe` exit 2 treated as success |

ADR-0001 and ADR-0002 are both statements about failure paths. They are decorative until this is fixed —
if the effect layer reports success for a failed call, rollback never learns it failed and Needs Attention
never triggers. **This is why Phase 1 is the effect layer and not the UI.**

**Root cause B — parallel implementations that drifted.** `detection.rs` vs `inspection.rs`,
`build.rs` mirrors vs `models/tweak.rs`, three effect services with a second elevated implementation,
a dead second YAML pipeline. Collapsing these is simultaneously the bug fix and the LOC win.

---

## Phase 0 — Make change verifiable

**Nothing below this line is safe without it.** Every module we are about to rewrite has **zero tests**:
`apply.rs`, `restore.rs`, `capture.rs`, `storage.rs`, `hosts_service.rs`, `firewall_service.rs`, `batch.rs`.
The 61 existing tests cluster in `system_info_service.rs` (12), `update.rs` (10) and `elevation/common.rs` (8)
— modules we are not touching. There is no CI (`.github/` has no workflows).

- Characterization tests over current apply / rollback / capture / restore behavior, including the failure
  paths. These pin behavior *before* it changes; several will encode bugs, which is correct — they get
  updated deliberately in the phase that fixes them.
- Fixture-driven tests for hosts parsing (BOM+CRLF, no-BOM+LF, mixed endings, non-UTF-8 bytes,
  multi-hostname lines, domain-as-second-hostname, duplicates) — these are cheap and catch the whole class.
- A CI workflow: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`.

Tests do **not** grow the shipped binary — `#[cfg(test)]` is compiled out of release builds.

**Also here, because it is free:** wire up `cargo deny` or at minimum `cargo audit`. It would have caught
the item below on its own.

---

## Phase 0.5 — Land immediately, independent of everything

These have no dependencies and shipping them late costs real user harm.

| Change | Why now |
|---|---|
| **`apply.rs:156-165` — the critical patch** | Bind the rollback result; delete the snapshot only on verified success; stop `?` swallowing the real apply error. `revert_tweak:270` already has the correct shape to copy. Every day this ships is a day someone can permanently lose their original state. |
| **`serde_yml` → `serde_yaml_bw`** | **RUSTSEC-2025-0068** — affects all versions, no patch exists, repo archived. Two-line swap. Build-time-only over trusted input so not exploitable, but there is no reason to carry it. Do **not** follow serde_yml's own deprecation notice to `noyalib` — same maintainer as the archived unsound crate. See "YAML parser choice" below for why `serde_yaml_bw` and not `serde_yaml_ng`. |
| **Dead-code deletion** (−~500 LOC) | rustc already flags most of it. `models/backup.rs` (whole file, dead), the dead YAML pipeline at `tweak.rs:527-621`, seven unused helpers, three never-constructed `Error` variants, dead struct fields hidden behind `#[allow(dead_code)]` masks. Pure deletion, compiler-verified. |
| **Doc fixes that describe unchanged behavior** | `hosts_changes`/`firewall_changes` into Option Structure; document `aliases`; rewrite Common Mistake #2; delete the `value: null` appendix row and fix Example 4 to `action: delete_value` (ADR-0004). |

**Deliberately deferred:** doc fixes describing behavior we are about to change (atomicity wording, System
Default, Needs Attention). Writing those now means writing them twice.

---

## Phase 1 — The effect layer's success contract

The root-cause fix. One shared definition of "did it work" across the five effect services, replacing five
different ones plus a second elevated implementation of three of them.

- **`service_control.rs` → `windows-sys`** (see resolution #1). Keep the four public signatures identical so
  none of the ~20 call sites across 7 files change. Gotchas that must be implemented, not discovered:
  `SERVICE_NO_CHANGE` for unchanged fields and NULL (not `""`) for unchanged strings; `QueryServiceConfigW`
  double-call sizing with a **pointer-aligned** buffer; `ControlService` returns immediately so
  `STOP_PENDING` must be polled with `dwCheckPoint` progress detection; `EnumDependentServicesW` before stop;
  `ERROR_SERVICE_NOT_ACTIVE`/`ALREADY_RUNNING` are idempotent success; `SC_HANDLE` is a pointer — check
  `is_null()`, not `== 0`; RAII handle guards, remembering `panic = "abort"` means `Drop` does not run on panic.
- **Task Scheduler → `windows` crate (COM).** `IRegisteredTask::State()` returns `TASK_STATE` as a **numeric
  enum**, which is the actual fix for the locale class. `IRegisteredTaskCollection` is 1-indexed.
- **Firewall.** Judgment call: `windows_firewall` is real COM (not a netsh wrapper) but has 9 stars and one
  reverse dependency, by its own author. Since the `windows` crate arrives for Task Scheduler anyway, prefer
  hand-rolling ~250-350 LOC over a thin dependency on the firewall path.
- **`capture.rs:588`** — stop mapping every read error to `existed: false`. This is the ADR-0003 blocker.
- **`winreg` stays.** It already distinguishes `NotFound` from `PermissionDenied` via `io::ErrorKind`; eight
  `.unwrap_or(false)` call sites coerce that away. **The bug is ours.** `windows-registry` would be a
  regression on exactly this axis.
- **Collapse the elevation duplicates** (−70 LOC) and the `if use_ti / else if use_system / else` triad
  repeated at 8 sites (−56 LOC). An `Elevation` enum makes `(use_system=false, use_ti=true)` unrepresentable.

Moving service and scheduler control in-process **deletes** the `cmd.exe` command-line construction for those
paths, which is the real fix for the injection surface — no crate does this, it is an architecture change.

---

## Phase 2 — Snapshots and rollback (ADR-0001, ADR-0002)

- Rollback always attempts all five phases and never aborts early. Today a registry restore failure abandons
  services, scheduler, hosts and firewall entirely.
- **Needs Attention** state: retains its snapshot, names the unrestorable resources, offers Retry and an
  explicit "Keep current state". The invariant from ADR-0002: *a snapshot is deleted only by a verified
  restore or an explicit user decision.*
- **`tempfile` for atomic snapshot writes.** Verified to use `MoveFileExW` + `MOVEFILE_REPLACE_EXISTING`,
  the correct Windows primitive. Handle `ERROR_SHARING_VIOLATION` when antivirus holds the target open.
  Rejected: `atomicwrites` (maintenance), `atomic-write-file` (**no Windows implementation at all**, despite
  its README).
- **Drop `fs4`** — `std::fs::File::lock` is stable and this project requires 1.92. Three lines, one fewer dependency.
- **Hosts file: write it ourselves, ~120 LOC.** No crate survives contact with the requirements —
  `hostsfile` was compiled and empirically **destroys CRLF line endings**; `hostfile` and `parse-hosts` have
  no write API. Read as bytes not `String`, preserve per-line endings and BOM, handle multi-hostname lines.

*This phase adds ~150-250 lines for Needs Attention. That is the one place where "less code" loses, deliberately.*

---

## Phase 3 — Detection, status, and the parse/clone question

- **Collapse `detection.rs` + `inspection.rs` onto one comparison core** (−331 LOC, highest-value item).
  They currently disagree in four verified ways: different registry comparators (REG_BINARY hex-string form
  matches in one, mismatches in the other); `inspection` ignores `*_missing_is_match` **entirely** (zero
  occurrences — affecting 8 shipping tweaks); inverted empty-option semantics (`inspection` returns vacuous
  true, `detection` returns not-matched); different service-error handling. One core returning per-item
  results serves both — `inspection` renders them, `detection` takes `.all()`.
  `inspection`'s output is serialized straight to the frontend, so the `OptionInspection` shape must not move.
- **Kill the clones in `tweak_loader.rs`.** `get_tweaks_for_version` deep-clones up to all 189
  `TweakDefinition`s — every `String`, every nested `Vec` — **on every call**. Return `&'static` references
  and iterators; serde serializes `&T` fine so the Tauri boundary is unaffected. This is deletion, and it is
  a far bigger win than the one-time parse.
- **On the parse overhead specifically:** the build does *not* convert YAML to Rust. It emits a **436 KB JSON
  string**, `include_str!`s it, and `serde_json::from_str`s it at runtime inside a `LazyLock`. The intent was
  right; the implementation stops one step short. Measure before acting — this is a one-time cost of roughly
  1-4 ms. True const Rust data would eliminate it and halve the catalog's memory (today the binary holds both
  the JSON string *and* the parsed heap copy), but `&'static str` cannot be deserialized from YAML at build
  time, so it **reintroduces the mirror-type duplication Phase 4 deletes**. `Cow<'static, str>` bridges it at
  real complexity cost. Treat as a measured trade, not a free win. `rkyv` is not worth it at this scale.

---

## Phase 4 — Build-time schema

- **Share the schema instead of mirroring it** (−304 LOC) — see resolution #2. After this, a renamed field is
  a compile error in both consumers instead of a runtime panic behind `.expect()`.
- **Keep the hand-rolled validation.** `garde`, `validator` and `serde_valid` were all evaluated and all
  rejected on the same four blockers: no warnings concept (the build emits at least six non-fatal warnings),
  no cross-file state (duplicate IDs span files), no Vec-uniqueness rule, and no way to attach the source
  YAML filename to an error. The existing error messages are better than what a derive macro can produce.
- **`schemars`** — generate a JSON Schema from the shared types for editor autocomplete and inline validation
  across 325 KB of YAML. Verified end-to-end against the real types; it reads existing serde attributes
  (`deny_unknown_fields` → `additionalProperties: false`). CI step to regenerate and fail on `git diff --exit-code`.
- **`regex` → `regex-lite` in `[build-dependencies]`** (0 LOC, closes a real gap): the build validates patterns
  with full `regex` while the runtime executes them with `regex-lite`, so a pattern can pass validation and
  fail on the user's machine.
- **Fix the build-time gap the audit found:** the non-empty-option check counts `skip_validation` and
  version-filtered changes, so ~8 tweaks ship with permanently undetectable status.

---

## Phase 5 — UI and profiles

- **System Default as a selectable state** (ADR-0003). `TweakCard.svelte:176-180` currently only unstages;
  the dropdown entry is `disabled: true` and vanishes once any option is applied. Both controls, whenever
  `has_backup`.
- **Needs Attention** surfaced with its Retry / Keep-current-state actions.
- **Profiles last** — they reuse the apply engine, so fixing them earlier would be fixing a facade.

---

## Phase 6 — Docs, written against what exists

`TWEAK_SYSTEM.md`, `TWEAK_AUTHORING.md` and `APP_CONTEXT.md` updated together: atomicity wording per ADR-0001
(*attempted atomically, with failure surfaced* — not a guarantee), System Default per ADR-0003, Needs
Attention, snapshot location, the detection algorithm covering all five effect phases, and `aliases`.

---

## Independent track — the YAML corpus

Unblocked by everything above; slot in anywhere. From the review:

- **`legacy_network_protocols` silently re-enables SMBv1** in two options while its name, description and
  info mention only LLMNR/WPAD/NetBIOS. Treat as urgent on its own merits.
- 98 options labelled "(Default)" write explicit Group Policy values that stock Windows ships **absent**,
  leaving the machine policy-managed.
- Asymmetric toggles where option[1] does not undo everything option[0] does.
- `mouse_input_mode` options 1 and 3 declare byte-identical state, making option 3 unreachable.
- Two gaming tweaks overwrite the same composite `DirectXUserGlobalSettings` string, erasing each other.

---

## Dependency ledger

| Out | In |
|---|---|
| `fs4` (std covers it) | `windows` (COM: Task Scheduler, firewall) |
| `serde_yml` (RUSTSEC-2025-0068, archived) | `serde_yaml_ng` |
| `hostname` (one call site) | `tempfile` |
| *optional:* `reqwest` → `ureq` (**removes 28 exclusive crates**) | `schemars` (build/dev) |
| *optional:* `tauri-plugin-log` (**removes 6 crates**, incl. `rust_decimal` + `byte-unit` for a stdout logger) | `semver` (replaces a hand-rolled parser, −92 LOC, fixes pre-release comparison) |

Dropped with the profile system: `zip`, `sha2`, `hostname`, `hex`.

Also free wins: use the already-present `wmi::WMIDateTime` instead of hand-rolled CIM_DATETIME parsing
(**fixes a real timezone bug** — uptime reads 0 for any uptime shorter than the UTC offset);
`trim-paths = true` strips `C:\Users\Ehsan\...` from the shipped binary.

~~Use the already-present `hex` for the REG_BINARY compact path~~ — **void.** `hex` is profile-only and gets
dropped, so this −10 LOC item would now cost a dependency.

---

## Decisions taken

1. **`tauri-plugin-log` — KEEP.** The earlier recommendation to hand-roll ~26 lines was scoped to today's
   stdout-only usage and is wrong for where this is going: a comprehensive logging system whose output users
   attach to bug reports. The plugin already does file logging with rotation. What to fix now is the dead
   `TargetKind::Webview` target (`lib.rs:33-34`), which serializes every log record and emits it as a Tauri
   IPC event to a listener that does not exist — remove it, add `LogDir`.
2. **`reqwest` → `ureq`. Accepted.** Removes 28 exclusive crates; update commands become sync.
3. **Nine dead Tauri commands — delete** (−163 LOC).
4. **Snapshots stay next to the executable.** Rationale: they survive a Windows reinstall, which an app-data
   location would not. Fix the three docs to match (`TWEAK_SYSTEM.md`, `APP_CONTEXT.md`, the `storage.rs`
   doc comment). See "Snapshot identity" below for the consequence this creates.
5. **Profile system — delete entirely, rebuild later** (see its own section below).

## Snapshot identity — a consequence of decision 4

Because snapshots survive a Windows reinstall, they can outlive the machine they describe. A snapshot saying
"the original value was X" restored onto a fresh install writes a value that was never *that* system's
original.

Resolution: stamp each snapshot with the Windows installation identity and **warn** on mismatch. Warning, not
auto-deletion — that keeps ADR-0002 intact, and the "Keep current state" consent path already exists to
resolve it.

Use `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` — regenerated on clean install, stable across feature
updates and hardware changes, readable without admin. (Known limitation: machines cloned from one image share
a GUID. Irrelevant here, since snapshots are not transferred between machines.)

**Hard prerequisite, do not skip:** review finding #18 — `TweakSnapshot` has no schema version and its nested
structs have no `#[serde(default)]`. Adding *any* field, including this one, makes every existing snapshot
fail to deserialize. Add `#[serde(default)]` and a schema version **first**, then the install ID. Getting this
backwards destroys exactly the data ADR-0002 exists to protect.

The rejected alternative was extending the existing stale-snapshot verification to infer "this is a different
Windows" from state mismatches. Correctly judged too complex for the benefit — an explicit identifier is one
registry read.

## Profile system — delete now, rebuild later

The profile system is incomplete (`system_state` is exported and never read; per-tweak progress events are
modelled and never emitted). Rather than carry a half-built subsystem through every stage below, it is deleted
in the cleanup stage and rebuilt from scratch later.

Boundary confirmed by identifier-level grep across the crate. **Not compile-verified — `cargo check` after
deletion is the only real proof.**

**Delete — 8 files, 2,169 LOC, all confirmed profile-only:**

| File | LOC | | File | LOC |
| --- | --- | --- | --- | --- |
| `models/profile.rs` | 533 | | `services/profile/archive.rs` | 199 |
| `commands/profile.rs` | 301 | | `services/profile/mod.rs` | 67 |
| `services/profile/validation.rs` | 623 | | `services/profile/migration.rs` | 60 |
| `services/profile/export.rs` | 347 | | `services/profile/import.rs` | 39 |

Plus 5 one-line module edits (`models/mod.rs:3,10`, `services/mod.rs:5`, `commands/mod.rs:6`,
`lib.rs:119-126`) and optionally the `ProfileError` variant (`error.rs:51-52,75`, zero non-profile constructors).
7 Tauri commands go, one of which (`get_windows_version`) already has zero frontend call sites.
3 `#[cfg(test)]` modules / 6 `#[test]` fns die with them.

**Startup safety — verified, no launch-time breakage.** The only lifecycle invoke is
`ProfileManager.svelte:102-103` `onMount(() => profileStore.loadSavedProfiles())`, and `ProfileManager` renders
only under `{:else if activeTab === "profiles"}` while `activeTab` is non-persisted state defaulting to
`"overview"`. It cannot fire at launch. The two modals mounted unconditionally at `+layout.svelte:138-139`
guard every effect behind `if (isOpen)` and invoke nothing.

**Keep-but-disable — 5 frontend files, 1,729 LOC.** Cheapest coherent approach is **one choke point**: stub
the 6 `invoke()` bodies in `src/lib/api/profile.ts:206,222,232,250,262,271` and remove the
`loadSavedProfiles()` call at `ProfileManager.svelte:103`. The type block (`profile.ts:15-190`, ~195 LOC) has
no backend dependency and compiles standalone. Nothing outside those 5 files imports the store or API.
Caution: `src/lib/api/index.ts:1` is `export * from "./profile"` — keep the exports or the barrel breaks.
Three entry points need the disabled treatment: the sidebar Profiles tab
(`navigation.svelte.ts:64-71`, `isPermanent: true`), `SnapshotsView.svelte:154,162`, and
`SettingsModal.svelte:63-73`.

**Dependencies — drop all four: `zip`, `sha2`, `hostname`, and `hex`.** Every `hex::` call site is inside
profile code. `registry_value.rs` does *not* use the crate — the apparent matches are a local variable named
`hex` (`:193,:200`), error-string prose, and test names. **Consequence:** the sweep's "use the already-present
`hex` for the REG_BINARY compact path" item is void — `hex` will no longer be present, so that −10 LOC would
now cost a dependency. Dropped from the plan.

**`aliases` dies.** Exactly one read in the entire codebase: `services/profile/validation.rs:134`. After
deletion it is write-only. This also retires the "document `aliases`" doc task. It is `pub` on a `Serialize`
struct so no dead-code warning will fire — remove it deliberately or not at all.

**Nothing else is orphaned.** `export.rs`'s `capture_system_state` is a private local fn that does *not* call
`backup/capture.rs`; every service fn it uses has independent callers. `detect_tweak_state` survives with 4
non-profile callers (`apply.rs:63`, `query.rs:83,126,178`).

### Before deleting: preserve the archive spec

`.mgx` archives are **recoverable, not orphaned** — Deflate ZIPs containing `profile.json`, optional
`system_state.json`, and `manifest.json` with a real `format_version: u32` field. A rebuilt system can read
3.0.0 archives.

**But one thing is unreproducible from the archive alone:** `hash_option_content`
(`services/profile/mod.rs:26-34`) is SHA-256 over `serde_json::to_vec(option)` domain-separated by the literal
`b"profile-option-v2"`, truncated to 32 hex chars — plus a legacy field-order variant at `:41-67`. That hash is
embedded in stored profiles. **Copy `services/profile/mod.rs` and `models/profile.rs` into a
`docs/spec/profile-v1.md` note before deleting**, or the rebuild cannot resolve moved options in existing
profiles. Also note `format_version` is currently written but never checked on read
(`archive.rs:145-149`) — the rebuild should fix that.

**Note for the rebuild:** `system_state` was intended for an import-time compatibility check that was never
built. That is the same problem the snapshot install-ID solves, so the rebuilt system should share one
machine-identity mechanism rather than inventing a second.

**This removes stage 7 entirely and shrinks the blast radius of stages 2–6** — the effect-layer contract, the
detection/inspection collapse, and the ADR work no longer have to keep a second apply path working.
