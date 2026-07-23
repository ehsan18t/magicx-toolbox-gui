# Tweak System Redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the tweak effect/apply/backup layer with the typed engine specified in
`docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md` (rev 2), hard-cutting the old
pipeline and corpus at engine-green.

**Architecture:** One typed representation (`Value`) feeds apply/capture/detect/restore. Effect data
is a serializable enum; per-kind behavior lives in `kinds/*` behind an `EffectKind` trait so the
engine is testable with mocks. Lifecycle = WAL-journaled apply with atomic rollback, reference-based
snapshots (restore re-applies the current definition), refcounted shared claims, and build-time
guards quantified per supported Windows milestone.

**Tech Stack:** Rust (MSVC, Windows-only), Tauri, existing `windows`/`windows-sys` primitives,
`thiserror`, `serde`/`serde_yaml` (build-time), Svelte 5 + rune stores (minimal adaptation).
**No new dependencies** — everything here is hand-rolled or reuses existing crates.

## Plan style — deliberate absence of implementation code

Per the maintainer's instruction, tasks pin **contracts** (exact names, signatures, file paths),
**behaviors** (named test cases with exact assertions in prose), **commands**, and **spec
references** — not implementation bodies. The spec (rev 2), ADRs 0001–0007, and `CONTEXT.md` are
binding; where a task and the spec disagree, **the spec wins and the discrepancy must be reported**,
not silently reconciled. Illustrative snippets are marked *(illustrative, non-binding)*.

## Global Constraints

- **Binding references:** spec rev 2 (`docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md`), ADRs `docs/adr/0001..0007`, glossary `CONTEXT.md`. Invariants §15 (26 items) are acceptance criteria.
- **Gate (backend):** `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings && cargo test` — green before every commit. Full stack at the end: `pnpm run validate`.
- Logging via `log` crate only — never `println!`/`eprintln!`. Errors via `thiserror`, propagate with `?`. Never `let _ =` a privileged call (invariant 2).
- Scheduler tests touching live Task Scheduler COM: mark `#[ignore]` (harness STATUS_ACCESS_VIOLATION artifact), run explicitly via `cargo test -- --ignored`.
- **Git:** work stays on branch `tweak-system-redesign`; stage explicit paths only (never `git add -A`); conventional-commit titles; commit at the end of every task. Never push.
- CRLF is git-normalized on commit — Write/Edit emitting LF is fine.
- Snapshots directory: `snapshots/` **next to the executable** (portable). Machine-stamped via `MachineGuid`.
- Frontend: Svelte 5 runes only, existing UI primitives only, `svelte-autofixer` after component edits.

---

### Task 0: Commit the pinned docs

**Files:**
- Commit only (no edits): `docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md`, `docs/adr/0002..0005` (amended), `docs/adr/0006-*.md`, `docs/adr/0007-*.md`, `CONTEXT.md`, `docs/superpowers/plans/2026-07-22-tweak-system-redesign.md`

**Interfaces:** none — this locks the binding references implementation agents read.

- [ ] **Step 1:** `git status` — confirm only the files above (plus possibly `docs/ROADMAP.md`/`PROGRESS.md`, which must NOT be staged) are modified/untracked.
- [ ] **Step 2:** Stage the listed paths explicitly; commit:
  `docs(tweak-system): pin rev-2 spec, ADR amendments 0002-0005, new ADRs 0006-0007, glossary, and implementation plan`

---

### Task 1: Core model types (`tweaks/model.rs`)

**Files:**
- Create: `src-tauri/src/tweaks/mod.rs`, `src-tauri/src/tweaks/model.rs`
- Modify: `src-tauri/src/lib.rs` (register module)
- Test: unit tests inline in `model.rs` (`#[cfg(test)]`)

**Interfaces (Produces — binding names for all later tasks):**
- `enum Effect { Setting(Setting), Shared(SharedId), Action(ActionDef) }`
- `enum Setting { Registry(RegAddr), RegistryKey(KeyAddr), Service(SvcAddr), Task(TaskAddr), Hosts(HostsAddr), Firewall(RuleAddr) }`
- `enum Value { Absent, Missing, Reg(TypedRegValue), Startup(StartupType), TaskEnabled(bool), Present(bool) }`
- `enum TypedRegValue { Dword(u32), Qword(u64), Sz(String), ExpandSz(String), MultiSz(Vec<String>), Binary(Vec<u8>) }`
- `enum StartupType { Boot, System, Automatic, AutomaticDelayed, Manual, Disabled }`
- `enum Level { User, Admin, System, Ti }` · `enum RiskLevel { Low, Medium, High, Critical }`
- `struct RegAddr { hive: Hive, path: String, name: String, ty: RegType, field: Option<FieldAddr> }` · `enum Hive { Hklm, Hkcu }` · `struct FieldAddr { field: String, format: PackedFormat }` · `enum PackedFormat { KvSemicolon }`
- `struct Tweak { id, name, description, category, info, warning, requires_reboot: bool, risk_level: RiskLevel, elevation: Level, reversible: bool, surface: Vec<EffectDef>, options: Vec<Opt>, windows: Option<WindowsScope> }`
- `struct EffectDef { id: EffectId, kind: Effect, elevation: Option<Level>, optional: bool, if_missing: Option<Value>, windows: Option<WindowsScope> }`
- `struct Opt { label: String, values: BTreeMap<EffectId, OptValue> }` · `enum OptValue { Set(Value), Run, Claim, Unclaimed }` (per-value `windows` scoping carried alongside `Set`)
- `struct ActionDef { apply: Script, undo: Option<Script>, probe: Option<Script>, ephemeral: bool, shell: Shell }` plus `DeleteTree` structural variant
- `struct WindowsScope { products: Option<Vec<u8>>, build: Option<BuildExpr>, revision: Option<BuildExpr> }` · `enum BuildExpr { Exact(u32), Min(u32), Max(u32), Range(u32, u32) }`
- `struct SharedDef { id: SharedId, setting: Setting, value: Value }`
- All serde-serializable; `Value` comparison is derived equality (one comparison per kind — invariant 1).

- [ ] **Step 1:** Write failing unit tests, named exactly: `value_roundtrips_serde` (every `Value` variant serializes and deserializes equal), `value_equality_is_per_kind` (e.g. `Absent != Present(false)`, `Missing != Absent`), `model_types_are_send_sync` (compile-time assertion).
- [ ] **Step 2:** Run `cargo test -p magicx-toolbox tweaks::model` — expect FAIL (module absent).
- [ ] **Step 3:** Implement the types per spec §5/§6 compiled-model shapes. No behavior beyond derives.
- [ ] **Step 4:** Run tests — expect PASS. Run the backend gate.
- [ ] **Step 5:** Commit: `feat(tweaks): add core typed model for the redesigned engine`

---

### Task 2: Authoring-surface parsers (paths, literals, windows grammar)

**Files:**
- Create: `src-tauri/src/tweaks/parse.rs`
- Test: inline unit tests

**Interfaces:**
- Consumes: Task 1 types.
- Produces: `parse_reg_path(&str) -> Result<(Hive, String), ParseError>` · `parse_value_literal(raw: &YamlScalar, ty: RegType) -> Result<Value, ParseError>` · `parse_windows_scope(yaml) -> Result<WindowsScope, ParseError>` · `parse_packed(format: PackedFormat, live: &str) -> Result<PackedFields, ParseError>` and `serialize_packed(...)` (order-preserving upsert per spec §5.2). These are the **single** parsers shared by `build.rs` and runtime (invariant 23's round-trip rule).

- [ ] **Step 1:** Write failing table-driven tests covering exactly these accept/reject cases (spec §5.1, §6.2, §6.6):
  - Paths: accepts `HKLM\A\B` and `HKEY_LOCAL_MACHINE\A\B` (normalized equal); rejects leading `\`, trailing `\`, empty segment `A\\B`, forward slash, hive alone, non-HKLM/HKCU hives.
  - Literals: DWORD accepts `1` and `0x1` (equal); QWORD > u32::MAX; BINARY accepts `"de,ad,be,ef"` and `"de ad be ef"` (equal bytes), rejects odd-length pair; MULTI_SZ from YAML list, `[]` → empty vec; `absent` keyword → `Value::Absent` for values/fields and `Present(false)` for presence kinds; `{ literal: absent }` → `Sz("absent")`; bare `absent` on SZ **without** escape context → the reserved keyword (never the string); `null`/empty scalar → error naming `absent`.
  - Windows: `products: [10, 11]` expands to build ranges (10 → 10240..=19045, 11 → 22000..); `build: ">=26100"`, `"26100..27200"`, exact; `revision` with non-exact `build` → error; empty `windows: {}` → unconstrained.
  - Packed (`kv_semicolon`): parse `A=1;B=2;` → fields in order; upsert `B=3` preserves `A` and order; upsert new field appends; remove (`absent`) deletes only that field; garbage input (`no-separators==;;`) → `ParseError` (never a partial result).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §5.1/§5.2/§6.2/§6.6. No regex in the packed parser.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): add path, literal, windows-scope, and packed-value parsers`

---

### Task 3: Schema loading + structural build guards

**Files:**
- Create: `src-tauri/src/tweaks/schema.rs` (YAML → model; written so `build.rs` can include it via `#[path]` exactly like today's `models/tweak_schema.rs`), `src-tauri/src/tweaks/validate.rs`
- Test: `src-tauri/src/tweaks/validate.rs` inline tests + bad-YAML fixtures under `src-tauri/tweaks_fixtures/bad/`
- **Do NOT touch `build.rs` in this task.** The live `src-tauri/tweaks/*.yaml` corpus is still old-schema and would fail the new loader; `build.rs` wiring happens in Task 15 once `examples.yaml` replaces it. Loader and guards are proven here through unit tests over fixtures.

**Interfaces:**
- Consumes: Tasks 1–2.
- Produces: `load_corpus(dir) -> Result<Corpus, Vec<ValidationError>>` where `struct Corpus { tweaks: Vec<Tweak>, shared: Vec<SharedDef> }`; `validate_structural(&Corpus) -> Vec<ValidationError>`. `ValidationError` carries tweak id, effect/option id, and a message that **names the fix** (e.g. the `absent` escape, the merge/reassign playbook).

- [ ] **Step 1:** Create one minimal-bad fixture per guard, each a tiny standalone YAML, named for its violation: `dup_address_two_tweaks.yaml`, `dup_address_direct_vs_shared.yaml`, `dup_address_in_one_tweak.yaml`, `dup_shared_decls.yaml`, `whole_vs_field_mix.yaml`, `services_start_raw_registry.yaml` (canonicalization), `option_missing_setting.yaml` (coverage), `shared_entry_omitted.yaml`, `bad_path_trailing_backslash.yaml`, `null_option_value.yaml`, `reversible_flag_lies.yaml`, `ti_disabled_by_typed_effect.yaml`. Write failing tests: each fixture must produce exactly its named `ValidationError` (assert on error kind + offending id), and a good corpus (spec §6 example, adapted) must load clean.
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement loading + these guards per spec §10 (ownership/dup ban, canonicalization, coverage incl. explicit `claim`/`unclaimed`, path syntax, reversibility honesty vs computed §6.4, TI self-availability over typed effects only).
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): schema loader and structural build-time guards`

---

### Task 4: Semantic build guards (milestones, detectability, distinctness)

**Files:**
- Modify: `src-tauri/src/tweaks/validate.rs`
- Test: inline + fixtures under `src-tauri/tweaks_fixtures/bad/`

**Interfaces:**
- Consumes: Task 3.
- Produces: `validate_semantic(&Corpus, &[Milestone]) -> Vec<ValidationError>`; `const SUPPORT_MATRIX: &[Milestone]` with `struct Milestone { build: u32 }` — initial values `19045, 22621, 22631, 26100` (spec §14 default; single source of truth here).

- [ ] **Step 1:** New fixtures + failing tests: `undetectable_on_one_milestone.yaml` (an option whose only detectable effect is build-scoped out on 19045), `all_optional_option.yaml` (violates ≥1 **non-optional** detectable), `identical_on_detectable_projection.yaml` (options differ only by a probe-less action), `differ_only_by_noundo_action.yaml` (Residue rule §10), `differ_only_by_undo_action_ok.yaml` (**must pass** — legal sole distinguisher), `shared_only_distinguisher.yaml` (must fail).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per-milestone quantification over the applicable projection (spec §10): for each milestone → compute each tweak's applicable surface → run detectability, byte-distinctness, detectable-projection distinctness, non-shared-difference, and the no-undo-action rule.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): per-milestone semantic guards (detectability, distinctness, residue rule)`

---

### Task 5: Registry kind

**Files:**
- Create: `src-tauri/src/tweaks/kinds/mod.rs` (defines `EffectKind` trait + `ExecCx`), `src-tauri/src/tweaks/kinds/registry.rs`
- Modify: `src-tauri/src/services/registry_service.rs` (harden `delete_key`: reject empty child name and leading/lone backslash — the parent/hive-root deletion hazard at `registry_service.rs:332-347`)
- Test: inline unit tests + round-trip tests against real registry under a dedicated `HKCU\Software\MagicXToolboxTest` key (created/removed per test)

**Interfaces:**
- Consumes: Tasks 1–2 (`Value`, `RegAddr`, packed parsers).
- Produces — **the trait every kind implements and the engine mocks** (binding):
  `trait EffectKind: Send + Sync { fn read(&self, s: &Setting, cx: &ExecCx) -> Result<Value, Error>; fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error>; }`
  plus `struct RegistryKind` implementing it for `Setting::Registry` and `Setting::RegistryKey`.

- [ ] **Step 1:** Failing tests, real registry (HKCU test key): `read_distinguishes_notfound_typemismatch` (absent value → `Ok(Absent)`; wrong type → typed error, **never** a fake Absent — invariant 2); `drive_roundtrip_every_reg_type` (write → read equal for all six types); `drive_absent_deletes_value`; `drive_autocreates_parent_path` (deep unexisting subpath); `key_presence_roundtrip` (`Present(true)`/`Present(false)` incl. delete-only-if-created semantics at the kind level: drive `Present(false)` deletes); `field_upsert_preserves_unknown_fields` (seed `A=1;X=9;`, drive field `A`→`2`, read whole value = `A=2;X=9;`); `field_on_malformed_value_is_typed_error`; `delete_key_guards` (empty child / leading backslash → error, parent untouched).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §5.1/§5.2 (auto-create, drive-to-Absent, field mutex around read-modify-write, hardened delete path in `registry_service`).
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): registry kind with field addressing, presence, and hardened deletes`

---

### Task 6: Service and Task kinds (Missing-aware)

**Files:**
- Create: `src-tauri/src/tweaks/kinds/service.rs`, `src-tauri/src/tweaks/kinds/task.rs`
- Test: inline; round-trip against a safe real service (read-only assertions on a well-known service's startup type + write round-trip only on a test-safe target per existing `roundtrip_tests.rs` conventions); scheduler write tests `#[ignore]`d

**Interfaces:**
- Consumes: Task 5's `EffectKind` trait.
- Produces: `ServiceKind`, `TaskKind` implementing `EffectKind`. Reads return `Value::Missing` when the service/task does not exist (never an error for that case, never `Absent`); `drive(_, Missing)` is a verified no-op (invariant 12).

- [ ] **Step 1:** Failing tests: `missing_service_reads_missing` (nonexistent service name → `Ok(Missing)`); `missing_is_distinct_from_access_denied` (assert error variant differs); `drive_to_missing_is_noop_ok`; `service_startup_roundtrip` (each `StartupType` on the designated test service); `task_enable_roundtrip` (`#[ignore]`).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement wrapping existing `service_control` / scheduler COM services, per spec §5.1/§5.4.
- [ ] **Step 4:** Run tests + gate (ignored tests via `cargo test -- --ignored` once, manually) — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): service and task kinds with typed Missing`

---

### Task 7: Hosts and Firewall kinds

**Files:**
- Create: `src-tauri/src/tweaks/kinds/hosts.rs`, `src-tauri/src/tweaks/kinds/firewall.rs`
- Test: inline; hosts round-trip against a temp-redirected hosts path if the existing `hosts_service` supports injection, else against the real file with guaranteed cleanup; firewall round-trip with a uniquely-named test rule, always deleted in teardown

**Interfaces:**
- Consumes: Task 5's trait.
- Produces: `HostsKind`, `FirewallKind` implementing `EffectKind` over `Present(bool)`, wrapping `hosts_service` / `firewall_service` (spec §5.1).

- [ ] **Step 1:** Failing tests: `hosts_present_roundtrip` (add → `Present(true)`, remove → `Present(false)`); `firewall_rule_roundtrip` (create named rule from full `RuleAddr` definition → present; delete → absent); `firewall_recreate_uses_authored_definition` (documented carry-over limitation — assert recreated rule matches the authored def).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): hosts and firewall presence kinds`

---

### Task 8: Action kind (scripts, probe, undo, ephemeral)

**Files:**
- Create: `src-tauri/src/tweaks/kinds/action.rs`
- Test: inline, using trivial real PowerShell/cmd scripts (e.g. `exit 0` / `exit 1`, a probe checking a temp-file marker)

**Interfaces:**
- Consumes: Tasks 1, 5; existing broker `-EncodedCommand` path for elevated contexts, direct spawn for User/Admin in-process contexts per §9.
- Produces: `ActionKind` with `run_apply(&ActionDef, cx) -> Result<()>`, `run_undo(...)`, `run_probe(...) -> Result<bool>` (exit 0 ⇒ present/true). Bounded timeout (default from spec §14: bounded, exit-code-only; stdout captured to logs). `DeleteTree` structural action implemented here via the hardened registry delete.

- [ ] **Step 1:** Failing tests: `apply_exit0_ok_exit1_err`; `probe_polarity` (exit 0 → `true`, nonzero → `false` — and a probe **failure to spawn** is `Err`, never `false`); `timeout_kills_and_errs`; `encoded_command_carries_special_chars` (script containing quotes/newlines/`$` round-trips); `ephemeral_has_no_undo_probe` (enforced at type/validation level — assert loader rejects, cross-check Task 3 fixture list, add `ephemeral_with_undo.yaml` fixture there if missing).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §7.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): action kind with exit-code contract, probe, timeout`

---

### Task 9: Snapshot store (WAL entries, dedup, invalid classification)

**Files:**
- Create: `src-tauri/src/tweaks/snapshot.rs`
- Test: inline, against temp directories (store root is injectable; production root = exe-adjacent `snapshots/`)

**Interfaces:**
- Consumes: Task 1 types.
- Produces (binding for engine tasks):
  - `struct SnapshotStore` with `open(root: PathBuf) -> Self`
  - `push(tweak_id, NewEntry) -> Result<Seq>` — create-new atomic (temp → fsync → rename); dedup: if `NewEntry.captured` is `OptionRef(label)` matching an existing entry, that entry is removed and the new one takes a **fresh head seq** (invariant 6)
  - `head(tweak_id) -> Result<Option<Entry>>` — skips invalid entries
  - `consume(tweak_id, seq) -> Result<()>` — only after verified restore (caller-enforced)
  - `mark_completed(tweak_id, seq, action_id) -> Result<()>` — fsynced journal update (invariant 5)
  - `classify(&RawEntry, &Corpus, machine_guid, running_build) -> EntryValidity` where `enum EntryValidity { Valid, Invalid(InvalidReason) }`, `InvalidReason ∈ { Corrupt, WrongSchema, WrongMachine, DanglingRef, TargetUnavailable }`
  - `list(tweak_id) -> Result<Vec<EntrySummary>>` (valid + invalid with reasons, for UI surfacing)
  - `discard(tweak_id, seq) -> Result<()>` — the explicit-consent release (ADR-0002)
  - `enum Captured { OptionRef(String), Values(BTreeMap<EffectId, Value>) }` · `struct Entry { schema_version, machine_guid, tweak_id, seq: Seq, timestamp, captured: Captured, journal: Vec<JournalRow> }` · `struct JournalRow { action_id: EffectId, intended: bool, completed: bool }`

- [ ] **Step 1:** Failing tests: `push_is_create_new` (same seq collision → loud error, first entry intact); `dedup_moves_option_ref_to_head` (push A-ref, push B-ref, push A-ref again → exactly one A entry, head, freshest seq); `dumps_never_dedup` (two `Values` captures both kept); `seq_is_monotonic_across_reopen` (reopen store, next seq > all prior — never wall-clock derived); `journal_mark_survives_reopen` (mark → reopen → completed=true); `classify_matrix` (one case per `InvalidReason`, incl. `DanglingRef` for a label absent from the corpus and `WrongMachine` for a foreign guid); `head_skips_invalid`; `discard_removes_only_target`.
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §8.2/§8.3/§11.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): snapshot store with WAL journal, move-to-head dedup, invalid classification`

---

### Task 10: Shared claims record

**Files:**
- Create: `src-tauri/src/tweaks/shared_claims.rs`
- Test: inline, temp dirs; drive/read side goes through an injected `EffectKind` mock

**Interfaces:**
- Consumes: Tasks 1, 5 (trait), 9 (same atomic-write helpers may be shared).
- Produces: `struct ClaimsStore` with `claim(shared: &SharedDef, claimant: &TweakId, kinds, cx) -> Result<ClaimOutcome>` (first claim: read original → persist → drive to shared value → verify; later: verify no-op), `release(shared_id, claimant, kinds, cx) -> Result<ReleaseOutcome>` where `ReleaseOutcome ∈ { StillHeld(Vec<TweakId>), RestoredOriginal }`, `holders(shared_id) -> Vec<TweakId>`, `is_claimed(shared_id) -> bool`. Record persisted atomically, machine-stamped, under the snapshots root (ADR-0006).

- [ ] **Step 1:** Failing tests (mock kind): `first_claim_captures_once_and_drives`; `second_claim_is_verified_noop` (mock records exactly one drive); `early_release_leaves_value_reports_holders`; `last_release_restores_original_unconditionally` (mutate mock's live value to simulate external drift → release still drives captured original — grill Q4); `failed_restore_keeps_record_needs_attention` (mock drive errs → record persists, error surfaces); `interleaving_property` (randomized claim/release sequences from N tweaks → original restored exactly once, at the true last release).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §8.6.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): refcounted shared-claims record with capture-once/restore-last`

---

### Task 11: Engine — detect

**Files:**
- Create: `src-tauri/src/tweaks/engine/mod.rs`, `src-tauri/src/tweaks/engine/detect.rs`
- Test: inline, mock `EffectKind` (in-memory state map) — zero OS contact

**Interfaces:**
- Consumes: Tasks 1–10 contracts.
- Produces (binding for commands/frontend):
  - `struct TweakStatus { state: TweakState, unavailable: Vec<UnavailableOpt>, residues: Vec<EffectId>, has_history: bool, held_shared: Vec<HeldInfo> }`
  - `enum TweakState { Active(OptLabel), SystemDefault, Unknown(Vec<UnknownReason>) }` — `UnknownReason` carries effect id + cause (`AccessDenied | Malformed | MissingRequired`), plus a `needs_elevation: bool` hint
  - `struct UnavailableOpt { label: OptLabel, reason: String }`
  - `detect(&Tweak, &Corpus, &Deps) -> TweakStatus` where `struct Deps<'a>` bundles kinds registry, claims store, snapshot store, probe cache, running `WinVer`
  - `struct ProbeCache` — per-session, keyed `(tweak_id, effect_id)`, invalidated by that tweak's apply/restore (spec §7)

- [ ] **Step 1:** Failing tests (each a scenario over the mock): `matching_option_wins`; `at_most_one_match_holds` (guard-legal corpus, all live states enumerated); `no_match_reads_system_default`; `optional_missing_maps_if_missing` (missing service + `if_missing: Disabled` matches the disabled option); `option_needing_missing_resource_is_unavailable`; `nonoptional_missing_is_unknown`; `access_denied_is_unknown_never_sd` (invariant 3); `malformed_packed_is_unknown`; `omitted_undo_action_expectation_strict` (probe present → omitting option does NOT match); `omitted_noundo_action_residue_tolerated` (probe present → omitting option DOES match, residue listed — grill Q3); `claimed_shared_matches_all_claimants`; `scoped_out_effect_excluded` (effect out of running build → not read, not compared); `empty_applicable_surface_is_unavailable_tweak`; `probe_cache_hit_no_respawn` (mock probe counts invocations).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §8.4 (+§5.4, §6.6, §8.6).
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): detection with Unknown/Unavailable/Residue and probe cache`

---

### Task 12: Engine — apply + rollback

**Files:**
- Create: `src-tauri/src/tweaks/engine/apply.rs`, `src-tauri/src/tweaks/engine/lifecycle.rs` (per-tweak async lock map, Needs-Attention assembly)
- Test: inline, mock kinds

**Interfaces:**
- Consumes: Tasks 9–11.
- Produces: `apply(&Tweak, target: &OptLabel, &Deps) -> Result<ApplyOutcome, EngineError>`; `ApplyOutcome` reports per-effect results + fresh `TweakStatus` (grill Q1: post-op status from the operation's own verify reads). `EngineError::RollbackReport { original: Box<Error>, rollback_failures: Vec<Error> }` (invariant 20). `NeedsAttention` carries exact unrecoverable items.

- [ ] **Step 1:** Failing tests: `already_active_is_verified_noop_no_snapshot`; `capture_before_mutation` (mock ordering assertion: all reads precede first drive — invariant 4); `unreadable_capture_aborts_untouched`; `entry_persisted_with_intended_actions_before_first_drive` (invariant 5); `completion_marked_after_each_action`; `declaration_order_preserved`; `verify_mismatch_rolls_back` (mock lies on one drive → snapshot restored, both errors in `RollbackReport`); `rollback_failure_is_needs_attention_snapshot_kept` (ADR-0001/0002); `verified_rollback_consumes_entry`; `omitted_undo_action_driven_back` (live probe present + target omits + undo exists → undo ran, journal records it — grill Q3); `noundo_residue_left_in_place`; `shared_claims_processed_in_order` (claim on claiming target, release on unclaiming target — grill Q3/§8.6); `missing_target_apply_fails_typed` (resource vanished post-detect); `per_tweak_lock_serializes_same_tweak` (two concurrent applies of one tweak → strictly sequential mock ops); `crash_window_simulation` (drop mid-apply after an action ran, before mark: re-open store → journal shows intended-unmarked → lifecycle reports Needs Attention).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §8.1 (+§8.6, §5.4).
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): WAL-journaled apply pipeline with atomic rollback`

---

### Task 13: Engine — restore

**Files:**
- Create: `src-tauri/src/tweaks/engine/revert.rs`
- Test: inline, mock kinds

**Interfaces:**
- Consumes: Tasks 9–12 (restore re-uses apply's drive path — spec: revert.rs is thin).
- Produces: `restore(&Tweak, &Corpus, &Deps) -> Result<RestoreOutcome, EngineError>`; `RestoreOutcome` = fresh status + consumed-entry info + `held_shared` notices + reboot-advisory flag (dump restores).

- [ ] **Step 1:** Failing tests: `undo_runs_reverse_order_before_reapply` (journal [a,b] completed → undo order [b,a]); `option_ref_reapplies_current_definition` (mutate corpus between capture and restore → restored values are the NEW definition — ADR-0007); `option_ref_runs_actions_and_ephemerals`; `dump_drives_values_only_sets_reboot_advisory`; `captured_missing_restores_as_noop`; `verified_restore_consumes_head_next_becomes_head`; `failed_restore_keeps_entry` (invariant 8); `incomplete_restore_needs_attention_kept`; `dangling_ref_skipped_and_surfaced_not_restored`; `walk_to_empty_history_reads_system_default` (ADR-0003); `claims_recomputed_like_apply` (restoring to unclaiming state releases; drift-stomp per grill Q4 via claims store); `apply_then_restore_property` (randomized option sequences on a mock surface: after restoring the just-captured entry, mock state == pre-apply state — the core §11 property test).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §8.5.
- [ ] **Step 4:** Run tests + gate — expect PASS.
- [ ] **Step 5:** Commit: `feat(tweaks): restore as undo-then-reapply with verified consumption`

---

### Task 14: Elevation routing + Windows version runtime

**Files:**
- Create: `src-tauri/src/tweaks/engine/context.rs` (level → `ExecCx` routing, SID guard, grouped batching), `src-tauri/src/tweaks/winver.rs`
- Modify: `src-tauri/src/services/elevation/broker.rs` area — add the **multi-op caller** (public fn taking `Vec<BrokerOp>` for one child; wire protocol already supports it, `broker.rs:69-76`)
- Test: inline; routing/grouping as pure-logic unit tests; SID comparison behind an injectable probe; `winver` against injected values + one live smoke test

**Interfaces:**
- Consumes: Task 1 (`Level`), existing broker.
- Produces: `effective_level(floor: Level, step: Option<Level>) -> Level` (max, escalate-only); `route(effect: &EffectDef, tweak: &Tweak) -> ExecCx` with the **HKCU exception** (user-hive → in-process interactive user regardless of floor — ADR-0005); `group_steps(&[PlannedStep]) -> Vec<ExecGroup>` (consecutive same-level System/TI steps share one child; User/Admin never grouped; order preserved — invariant 18); `sid_mismatch() -> bool` (process token SID vs interactive session SID); `running_winver() -> WinVer { build: u32, revision: u32 }` via `RtlGetVersion` + `UBR` registry value; `WindowsScope::applies(&WinVer) -> bool`.

- [ ] **Step 1:** Failing tests: `effective_level_is_max_escalate_only` (all 16 floor×step combos); `hkcu_ignores_floor` (System-floor tweak, HKCU effect → in-process-user context); `grouping_preserves_order_and_boundaries` (sequence U,S,S,T,T,S → groups [U][S,S][T,T][S]); `admin_never_grouped_into_child`; `insufficient_elevation_two_distinct_errors` (couldn't-acquire vs acquired-but-denied are different variants); `scope_applies_matrix` (products/build/revision combos incl. revision-on-exact-build); `sid_mismatch_disables_user_level` (guard true → User-level tweaks flagged, others untouched).
- [ ] **Step 2:** Run — expect FAIL.
- [ ] **Step 3:** Implement per spec §9/§6.6; multi-op caller added beside the existing `run_one` without changing broker wire semantics.
- [ ] **Step 4:** Run tests + gate — expect PASS. Manually run one grouped System-level op end-to-end on the dev machine (elevated) and note the result in the commit body.
- [ ] **Step 5:** Commit: `feat(tweaks): execution-context routing, SID guard, grouped broker caller, winver runtime`

---

### Task 15: Example corpus + real-machine end-to-end + the hard cut

**Files:**
- Create: `src-tauri/tweaks/examples.yaml` (replaces all current YAML — one registry tweak with tri-state `absent`, one service+task tweak with `optional`/`if_missing`, one hosts+firewall tweak, one action tweak (probe+undo, plus an ephemeral), one shared pair, one packed-value tweak, one windows-scoped tweak)
- Modify: `src-tauri/build.rs` — replace the old schema/validation wiring with the new loader + `validate_structural` + `validate_semantic` over `src-tauri/tweaks/` (now containing only `examples.yaml`); `src-tauri/src/lib.rs` — remove the deleted commands from `generate_handler!` (the tweak command surface is intentionally empty until Task 16)
- Delete: all 9 existing `src-tauri/tweaks/*.yaml`, `src-tauri/src/services/backup/` (entire module), `src-tauri/src/models/tweak_schema.rs` + the `models/tweak.rs` re-exports, and the old command modules `src-tauri/src/commands/tweaks/` + `src-tauri/src/commands/backup.rs`. **No shims, no dual pipeline:** after this task the only tweak engine in the tree is the new one. The backend gate must be green with zero tweak commands registered; the frontend calls commands by string name, so its gate is unaffected until Task 17.
- Test: `src-tauri/src/tweaks/e2e_tests.rs` (real machine, HKCU-scoped where possible; service/scheduler cases follow Task 6 ignore conventions)

**Interfaces:**
- Consumes: everything prior.
- Produces: the example corpus every later doc/test references; proof the compiled pipeline works on a real machine.

- [ ] **Step 1:** Author `examples.yaml` per the list above (spec §6 example is the seed). Run `cargo build` — expect the new validator to pass it; deliberately break one option (omit a setting) → expect the named build error → revert the break.
- [ ] **Step 2:** Write failing E2E tests: `registry_tweak_full_lifecycle` (detect SD → apply A → status Active(A) → apply B → restore → Active(A) → restore → SD, snapshots consumed exactly per walk); `shared_pair_lifecycle` (apply both, revert one → other still Active, revert last → original restored); `packed_field_lifecycle` (unknown field seeded by hand survives the full cycle).
- [ ] **Step 3:** Run — expect FAIL, then make green (these tests exercise wiring, not new logic; failures here are integration defects — fix them, never weaken the assertions).
- [ ] **Step 4:** Delete the old YAML + `services/backup/` + old schema per the Delete list; fix every compile error by **removal or migration to the new engine only** (no shims). Gate green.
- [ ] **Step 5:** Commit: `feat(tweaks)!: cut over to the redesigned engine; delete legacy pipeline and corpus`

---

### Task 16: Tauri command layer

**Files:**
- Create: `src-tauri/src/commands/tweaks.rs` (one module; the old `commands/tweaks/` tree and `commands/backup.rs` were deleted in Task 15, so this is a clean create, not a migration)
- Modify: `src-tauri/src/lib.rs` (`generate_handler!` list)
- Test: command-level tests where feasible (thin layer — logic lives in the engine; assert wiring: lock reuse, status emission)

**Interfaces:**
- Consumes: engine API (Tasks 11–14).
- Produces (binding for the frontend): commands `get_tweaks() -> Vec<TweakView>` (compiled model incl. options, risk, reversible, elevation, availability), `get_statuses_stream()` — kicks the background full scan, emitting `tweak-status` Tauri events per result batch (grill Q1/Q5: incremental arrival), `apply_tweak(tweak_id, option_label) -> ApplyOutcome`, `restore_tweak(tweak_id) -> RestoreOutcome`, `list_snapshot_entries(tweak_id)`, `discard_snapshot_entry(tweak_id, seq)` (consent release), `rescan_after_elevation()`, `get_elevation_state() -> { level, sid_mismatch }`. Every command logs at entry and returns `Result<T, Error>` (CLAUDE.md rule).

- [ ] **Step 1:** Define the command signatures + `TweakView`/event payload types; wire `generate_handler!`. Write failing tests for: `statuses_emit_incrementally` (event per batch, not one blob), `apply_returns_fresh_status_no_rescan` (engine outcome reused).
- [ ] **Step 2:** Run — expect FAIL → implement → PASS. Full backend gate.
- [ ] **Step 3:** Commit: `feat(commands): new tweak command surface with streaming statuses`

---

### Task 17: Frontend minimal adaptation

**Files:**
- Modify: `src/lib/stores/` tweak store (`.svelte.ts` — accept incremental status events, new state shape), the tweak card/list components (render `Unknown` with reason + elevate hint, `Unavailable` with reason, Residue info marker, held-by info, Needs-Attention detail, 1-authored-option toggle shape, SID-mismatch notice), API layer calling the Task 16 commands
- Test: `pnpm run check && pnpm run type-check && pnpm run lint`; `svelte-autofixer` on every edited component

**Interfaces:**
- Consumes: Task 16 command + event contracts.
- Produces: a working app on the new engine — **no visual redesign, existing primitives only** (Badge/Card/Modal/Switch/Select), no snapshot-history browser (Restore = single head-walk button).

- [ ] **Step 1:** Adapt the store to the event stream + new `TweakStatus` shape (runes; getter-based access per house rules).
- [ ] **Step 2:** Render each new state functionally; wire Restore/discard/elevate flows; aria labels intact.
- [ ] **Step 3:** Run frontend gate + `svelte-autofixer`; launch the app (`pnpm tauri dev`) and manually walk one full lifecycle on the examples (apply → restore → SD; elevate → Unknowns resolve). Record what was verified in the commit body.
- [ ] **Step 4:** Commit: `feat(ui): adapt stores and tweak views to the redesigned engine states`

---

### Task 18: Docs realignment + final gate

**Files:**
- Modify: `docs/TWEAK_AUTHORING.md` (rewrite authoring guide: new schema, house style, `.reg` conventions, `absent`, `optional`/`if_missing`, `shared`, `windows`, packed fields, action contract, examples from `examples.yaml`), `docs/TWEAK_SYSTEM.md` (architecture reference → new engine), `docs/ARCHITECTURE.md` (line ~615 snapshot-location fix), `docs/ROADMAP.md` (stage status)
- Test: `pnpm run validate` (full stack)

**Interfaces:** none produced — closes the loop CLAUDE.md requires ("when tweak runtime behavior changes, update TWEAK_AUTHORING.md").

- [ ] **Step 1:** Rewrite the three docs against the spec + shipped code; every YAML fragment in the authoring guide must be copy-paste-buildable against the validator (spot-check by pasting one into a scratch fixture).
- [ ] **Step 2:** Run `pnpm run validate` — green. Run `cargo test -- --ignored` once; report results.
- [ ] **Step 3:** Commit (single docs commit per house rules): `docs(tweak-system): realign authoring guide, system reference, and architecture to the new engine`

---

## Self-review (performed at authoring time)

- **Spec coverage:** invariants 1–26 each map to named test cases (1→T1/T5; 2→T5/T8; 3→T11; 4/5→T12; 6→T9; 7→T13; 8→T9/T13; 9→T11; 10/11→T3/T4; 12→T6/T11; 13→T2; 14→T5/T8; 15→T5; 16→T3; 17→T10; 18→T12/T14; 19→T8/T11; 20→T12; 21→T9; 22→T14; 23→T4; 24→T14; 25→T12-batch semantics surface via commands (per-tweak independence is engine-level: covered by per-tweak lock tests + command wiring in T16); 26→T4/T11/T12). Grill decisions: Q1→T11/T16, Q2→T15/plan structure, Q3→T4/T11/T12, Q4→T10, Q5→T17.
- **Placeholder scan:** no TBDs; every test step names concrete cases with expected outcomes; implementation steps bind to spec sections instead of code bodies **by explicit design** (maintainer instruction) — implementers must treat spec references as normative, not optional reading.
- **Type consistency:** `EffectKind`/`ExecCx` (T5) consumed by T6–T8, T10–T14; `TweakStatus` (T11) consumed by T12–T13, T16–T17; `SnapshotStore` (T9) by T12–T13; `ClaimsStore` (T10) by T12–T13; names checked for drift.
- **Known intentional deviation from the writing-plans template:** no inline implementation code, per the user's explicit constraint. Test cases are named contracts instead of code blocks for the same reason.
