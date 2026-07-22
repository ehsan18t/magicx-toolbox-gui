# Tweak System Redesign — Design

Status: **approved (design, rev 2)** · Date: 2026-07-21 · Revised: 2026-07-22 · Supersedes the
effect/apply/backup layer described in `docs/TWEAK_SYSTEM.md`.

Rev 2 folds in the full design-review Q&A: presence semantics, shared ownership, restore-is-reapply,
the `.reg`-aligned authoring conventions, the merged registry paths, the new Windows-version model, and
the Unknown status. The resolved-decision log is Appendix A; ADR-0006 and ADR-0007 record the two new
architectural decisions, and ADR-0002/0003/0004/0005 carry amendments.

## 1. Context & motivation

The current tweak system splits a single registry/service/task change into **four separate
representations** — a YAML change, a captured snapshot, a restore op, and a detection comparison — spread
across `services/*_service.rs` and `services/backup/{capture,restore,compare}.rs`. Those representations
drift, and the drift is where correctness dies: state can be lost on a failed rollback, a revert can
silently do nothing, and detection can disagree with what was applied.

This redesign rebuilds the effect-execution layer and the lifecycle around it so that correctness is
enforced by **types and the compiler**, not by convention. It is a first-principles rebuild, not a
patch-list — the rules below stand on their own merit.

## 2. Goals & non-goals

**Goals**
- One representation per change: apply, capture, detect, and revert consume the *same* typed value.
- Reversibility and detectability are **typed properties**, so the system never claims an operation it
  cannot perform (the ADR-0001 honest-failure principle made structural).
- The did-it-work contract is unavoidable: a failed effect surfaces as `Err`, never a benign value —
  and that extends to detection: an unreadable state is **Unknown**, never a guess.
- Build-time validation proves the properties that today only fail at runtime (undetectable tweaks,
  unreachable options, tweaks that fight over the same state).

**Non-goals**
- Not rewriting the Svelte frontend. The Tauri command contract adapts as needed but the UI model is
  preserved and extended: per-tweak status (authored option / System Default / **Unknown**), per-option
  **unavailable-on-this-machine**, Residue markers, Restore Snapshot, Needs Attention — via minimal
  functional adaptation with existing UI primitives only (§12).
- Not shipping the full corpus in the first engine drop. The engine ships with a small **rewritten**
  starter set; the remaining corpus is **redesigned** (not mechanically converted) as follow-up (§12).
- No dual-schema compatibility layer and no snapshot migration. The app is unreleased; existing
  installs are test-only and disposable (§12).

## 3. Decisions (the spine)

| Decision | Choice |
|---|---|
| **Scope** | Executors + shared lifecycle + snapshot format + YAML schema + `build.rs`. Frontend adapts. |
| **Core model** | Declarative typed **Settings** (desired value per option + runtime-captured value) with one representation feeding apply/detect/restore; imperative **Actions** as an explicit escape hatch. |
| **Escape hatch** | Actions carry a required `apply` and **optional** `undo` / `probe` / `ephemeral`. Absent `undo` ⇒ typed non-reversible; absent `probe` ⇒ typed non-detectable. Honest one-way behavior, never silent. |
| **Engine structure** | Hybrid: effect **data** is a serializable enum; each kind's **behavior** lives in its own deep module behind an `EffectKind` trait. |
| **Presence** | Machine-local resource absence is a typed, capture-only **Missing** state with author-declared meaning (`optional` / `if_missing`); the engine never installs or uninstalls resources (§5.4). |
| **Ownership** | **One address, one owner**, corpus-wide, enforced at build; genuine sharing goes through a declared **`shared`** setting with claim/release refcounting (§6.5, ADR-0006). |
| **Snapshots** | Authored-option captures are **references** re-applied from the current corpus; only unauthored states are value dumps. Restore = undo recorded actions, then re-apply the target (§8, ADR-0007). |
| **Authoring surface** | `.reg`-aligned literals, merged `HKLM\…` paths, reserved `absent` keyword at value/key/field depth, field addressing for packed values (§6). |
| **Versioning** | `windows: { products, build, revision }` at tweak/effect/value level; H-names dropped; guards quantified per declared support milestone (§6.6, §10). |
| **Migration** | Clean break; corpus rewritten from scratch with correct ownership boundaries; no snapshot converter (unreleased app). |

## 4. Architecture overview

```
YAML corpus + shared block ──build.rs──▶ compiled Tweak model (surface: Vec<EffectDef>, options = value-maps)
                                                  │
                                                  ▼
                       Engine (lifecycle: apply · detect · restore · verify · atomic rollback
                               · per-tweak lock · WAL action journal · shared-claims record)
                                                  │  dispatches each Effect to →
                                                  ▼
        EffectKind modules (registry / registry_key / service / task / hosts / firewall / action
                            — read+apply+revert+detect co-located)
                                                  │  execute through →
                                                  ▼
                     Execution context (existing typed elevation broker: user / admin / system / TI,
                                        grouped multi-op children for System/TI)
```

- The **Engine** owns lifecycle, ordering, verification, rollback, the per-tweak lock, the WAL action
  journal, and the shared-claims record.
- Each **EffectKind module** owns how *one kind* reads/applies/reverts a value.
- The **broker** owns privilege. It is reused as-is and gains the grouped-execution caller (§9) — the
  wire protocol already carries `Vec<BrokerOp>`.
- **Snapshots** are per-tweak histories: option **references** plus value **dumps** for unauthored
  states (§8, §11).

## 5. The core abstraction — `Effect`

An `Effect` is one atomic unit of change:

```rust
enum Effect  { Setting(Setting), Shared(SharedId), Action(Action) }
enum Setting { Registry(RegAddr), RegistryKey(KeyAddr), Service(SvcAddr),
               Task(TaskAddr), Hosts(HostsAddr), Firewall(RuleAddr) }
enum Value   { Absent, Missing, Reg(TypedRegValue), Startup(StartupType),
               TaskEnabled(bool), Present(bool) }
```

### 5.1 Settings (declarative, reversible by construction)

| kind | addresses | value domain |
|---|---|---|
| `Registry` | full path `HIVE\key`, value name, type, optional `field`+`format` | typed literal / `Absent` |
| `RegistryKey` | full path `HIVE\key` | `Present(bool)` |
| `Service` | service name | `StartupType` (Boot / System / Automatic / AutomaticDelayed / Manual / Disabled) |
| `Task` | exact task path | `TaskEnabled(bool)` |
| `Hosts` | `ip` + `domain` pair | `Present(bool)` |
| `Firewall` | named rule + full rule definition | `Present(bool)` |

Operations: **read** → current value, **apply** → drive to desired, **revert** → drive to the captured
value; *detect is read + compare*. All four share one address+value type and one comparison, so they
cannot disagree. Addresses are **exact names only** — no patterns, no wildcard task names: patterns
break address identity, which the ownership guard (§10) and snapshot keys depend on.

**Registry specifics:**
- Paths are single merged strings: `key: 'HKLM\SOFTWARE\…'`. v1 hives: **HKLM and HKCU** only
  (short and long spellings accepted, normalized at build); leading/trailing backslashes, empty
  segments, and forward slashes are build errors. House style: plain or single-quoted YAML scalars, so
  backslashes are written singly, exactly as regedit shows them.
- A value write **auto-creates missing parent keys** (standard `RegCreateKeyEx` behavior). Restore
  drives values back; an empty key husk left behind is accepted and documented — harmless and normal
  on Windows.
- Key existence as a *switch* is the `RegistryKey` Setting: pre-existence is **captured** like any
  value, so revert deletes the key only if we created it. There is **no `create_key` Action and no
  `delete_value` Action** — both are subsumed by Settings (`Absent` deletes a value; `Present(false)`
  removes a key we created). The one structural Action that remains is **delete-tree** (§7), one-way
  unless the author supplies `undo`.
- `Absent` is the drivable "does not exist" state for registry **values and packed fields**; the
  presence kinds (`RegistryKey`, `Hosts`, `Firewall`) model the same idea as `Present(false)`. The
  authoring keyword `absent` is one word for both: it compiles to `Value::Absent` on values/fields
  and to `Present(false)` on presence kinds — one spelling for authors, one typed comparison per
  kind. Capture of a nonexistent value = `Absent`.

### 5.2 Packed-value field addressing

Some registry values pack several independent knobs into one string
(`DirectXUserGlobalSettings` = `SwapEffectUpgradeEnable=1;VRROptimizeEnable=0;`). A Registry Setting
may address **one field** of such a value:

```yaml
registry: { key: 'HKCU\Software\Microsoft\DirectX\UserGpuPreferences',
            name: DirectXUserGlobalSettings, type: REG_SZ,
            field: SwapEffectUpgradeEnable, format: kv_semicolon }
```

- v1 ships exactly one format, `kv_semicolon` (`Name=Value;` pairs), as a deterministic parser —
  **no regex anywhere in the write path**.
- Apply parses the live string, **upserts only the addressed field, preserving unknown fields and
  their order**, and re-serializes. Capture/detect/restore operate on the field's value like any other
  typed value; `absent` removes the field.
- A live string the parser cannot understand is a **typed read error** → the tweak reads as Unknown —
  never a guess, never a destructive rewrite.
- Field writes are read-modify-write, so the registry kind serializes them behind one process-wide
  mutex (the per-tweak lock does not cover two tweaks addressing different fields of one value).

### 5.3 Shared references

An effect may be `shared: <id>`, referencing a corpus-level shared setting (§6.5). Its option values
are `claim` / `unclaimed`; its lifecycle is the claims record (§8.6), not the per-tweak snapshot.

### 5.4 Presence — the `Missing` state

`Missing` means **the addressed resource does not exist on this machine** (service not installed, task
not found). It is distinct from `Absent` (a state we can drive to) and from a read error:

- `Missing` is **capture-only**: an option can never author it, and **driving to `Missing` is a defined
  no-op** — the engine never installs or uninstalls services/tasks.
- An effect that may legitimately be missing declares `optional: true`, optionally with
  `if_missing: <value>` — "on a machine where this resource is absent, detection treats this effect as
  reading `<value>`" (e.g. a missing service counts as `disabled`).
- An option whose desired value for a Missing resource ≠ its `if_missing` meaning (e.g. *enable* a
  service that is not installed) is shown **unavailable on this machine** at detect time; apply is
  never offered. If a resource vanishes between detect and apply, the apply fails typed
  (resource-missing) and rolls back — never a silent skip.
- A **non**-optional effect reading `Missing` is a typed error → the tweak reads as Unknown.

### 5.5 Actions (imperative escape hatch)

A `cmd`/`powershell` script or the structural **delete-tree**. Contract in §7.

## 6. Data model & YAML schema (effect-centric)

The schema is **effect-centric**: a tweak declares its **managed state surface once**, and every option
supplies values over that surface — one flat map, one table row per option. This makes it structurally
impossible for two options to touch inconsistent address sets.

```yaml
id: windows_update_control
name: "Windows Update"
description: "Controls automatic update behavior."
category: system
risk_level: high              # low | medium | high | critical
elevation: admin              # per-Tweak FLOOR: user | admin | system | ti (declared, never inferred)
reversible: true              # computed & build-checked (§6.4)

effects:                      # the managed surface — declared ONCE
  - id: no_auto_update
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU', name: NoAutoUpdate, type: REG_DWORD }
  - id: au_options
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU', name: AUOptions, type: REG_DWORD }
  - id: update_service
    service: { name: wuauserv }
  - id: medic_task
    task: { path: '\Microsoft\Windows\WaaSMedic\PerformRemediation' }

options:                      # ONLY the real states we offer — "System Default" is NOT authored
  - label: "Fully Disabled"
    values:
      no_auto_update: 1
      au_options: absent      # tri-state policy: the value is removed
      update_service: disabled
      medic_task: disabled
  - label: "Notify Only"
    values:
      no_auto_update: 1
      au_options: 0x2         # decimal or 0x hex, as regedit shows it
      update_service: manual
      medic_task: enabled
```

### 6.1 Statuses and the shape rule

**"System Default" is a computed status, never an authored option** (ADR-0003): the app shows it when
the live surface matches *none* of the authored options. **Unknown** is the status when the surface
cannot be read (§8.4). UI shape: **1 authored option ⇒ toggle** (Default ↔ On), **≥2 authored ⇒
dropdown** (Default / A / B …). The Default position is a computed status, not a selectable target;
the user returns toward prior states only via **Restore Snapshot**.

### 6.2 Value literals (`.reg`-aligned)

- Types use the real names: `REG_DWORD`, `REG_QWORD`, `REG_SZ`, `REG_EXPAND_SZ`, `REG_MULTI_SZ`,
  `REG_BINARY`.
- `REG_DWORD`/`REG_QWORD`: decimal or `0x` hex. `REG_SZ`/`REG_EXPAND_SZ`: plain strings.
- `REG_BINARY`: `.reg` hex-pair form — `"90,12,03,80"` (commas or spaces).
- `REG_MULTI_SZ`: a real YAML string list; `[]` clears it (deliberately better than `.reg`'s
  unwritable `hex(7):`).
- **`absent` is one reserved keyword** meaning "does not exist," accepted uniformly at all three
  depths — value, key, field — and for **every** value type including `REG_SZ` / `REG_EXPAND_SZ`.
  A bare `absent` is *always* the keyword; that is what keeps deletion expressible everywhere
  (invariant 13). A string value whose literal content is the word `absent` uses the escape
  `{ literal: absent }`, which is the escape's entire purpose. The same reservation rule covers
  positional keywords (`present`, `claim`, `run`, …) in their own positions.
  A `null` or empty option entry (`some_effect:` with nothing after it) is a **build error naming
  `absent`** — a *forgotten* value must never become a silent delete, which is ADR-0004's actual
  principle; a deliberately typed `absent` is not a forgotten value.
- **No `-` delete spellings and no `.reg` import** — the YAML schema is the single source of truth. A
  dev-side `.reg → YAML` scaffolding tool may exist later, outside the engine.
- Kind-specific keywords: services take `disabled | manual | automatic | automatic_delayed | boot |
  system`; tasks `enabled | disabled`; presence kinds `present | absent`; shared `claim | unclaimed`;
  actions `run` (or the entry is omitted).

### 6.3 Coverage

Every option covers **every Setting effect** on the surface — the build guard rejects a hole. Action
entries are `run` or omitted (omitted = this option does not run it). Shared entries are always
explicit: `claim` or `unclaimed` — omission is a build error, so sharing is always a visible decision.

### 6.4 Computed properties

- `reversible` = every effect is a Setting, an Action with `undo`, or an `ephemeral` Action.
  Build-checked against the declared flag; a one-way tweak is labelled before apply, never discovered
  at restore time.
- Detectability is **typed**: Settings are always detectable; Actions iff they declare `probe`.
  **There is no `skip_validation` flag, no `ignore_not_found`, and no `task_name_pattern`** — their
  historical jobs are replaced by typed presence (`optional`/`if_missing`, §5.4), typed version
  scoping (§6.6), and exact-name addressing (§5.1).
- Tweak metadata: `description`, `category`, optional `info` / `warning`, optional
  `requires_reboot: bool` (default `false`), `risk_level ∈ {low, medium, high, critical}` (advisory).

### 6.5 The corpus-level `shared` block

```yaml
shared:
  - id: telemetry_off
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection', name: AllowTelemetry, type: REG_DWORD }
    value: 0                  # THE shared value — declared here, never per-option
```

Tweaks reference it as an effect (`- id: telemetry, shared: telemetry_off`) and options say `claim`
or `unclaimed`. The shared block declares the single target value, so claiming tweaks **cannot
disagree by construction**; two tweaks wanting *different* values on one address is a build error
whose message names both tweaks and the merge/reassign playbook. Runtime semantics: §8.6.

### 6.6 Windows-version applicability

```yaml
windows:
  products: [10, 11]          # exact set membership; sugar over build ranges (10 = 10240..19045, 11 = >=22000)
  build: ">=26100"            # N | >=N | <=N | A..B   (inclusive), against the major build
  revision: ">=2314"          # same grammar, against the UBR (the part after the dot)
```

- Every field optional; omitted = unconstrained; axes AND together. `revision` is only legal when
  `build` pins a **single** build (revision counters reset per build line — a cross-build revision
  range is a build error). H-names (22H2, …) do not exist in the schema.
- Allowed at **tweak level** (whole tweak scoped), **effect level**, and **per-option-value** level.
- Runtime reads the build via `RtlGetVersion` (never `GetVersionEx`) and the revision via the
  `UBR` registry value. A scoped-out effect is **excluded entirely** — not applied, not read, not
  counted. A tweak whose applicable surface is empty on the running build is shown **unavailable,
  with the reason**.
- Build guards evaluate every property per milestone of the declared support matrix (§10).

### 6.7 House style

Addresses in one-line flow form; everything else block form; plain/single-quoted path scalars
(single backslashes). **No sugar in v1**: no YAML anchors, no key grouping, no inline values — one way
to write everything. Revisit only if authoring the real corpus proves repetition pain.

## 7. Action contract (the escape hatch)

Actions exist for free-form scripts that cannot be expressed as declarative settings, plus the one
structural op left: **delete-tree** (one-way unless the author supplies `undo`, e.g. a `.reg` restore).

- **`apply` is required**; **`undo`, `probe`, and `ephemeral` are optional and independent.**
  - `undo` present ⇒ that action reverts cleanly; absent ⇒ that action is one-way.
  - `probe` present ⇒ contributes to detection; absent ⇒ it does not.
  - `ephemeral: true` ⇒ a transient side-effect changing no persistent state (flush DNS, restart
    Explorer, `gpupdate /force`): runs on apply, takes **no** `undo`/`probe`, exempt from the
    reversibility and detectability computations. An ephemeral action never makes a tweak one-way.
- The deciding question for `undo`: *can the change persistently alter or break the system?* Yes → it
  needs `undo` (or the tweak is honestly one-way); no → `ephemeral`.
- A single non-ephemeral no-`undo` action makes the whole tweak `reversible: false`, surfaced up
  front. On restore, everything else still reverts; only the genuinely one-way actions cannot, and
  those surface as **Needs Attention**. "Partial" never means "nothing reverts."
- **`probe` is state-based, never history-based**: it answers *"is the state this action produces
  currently present?"* — the same probe is the apply-time did-it-work check and the detect-time
  present/absent contribution. Probe results are **cached per session** and refreshed after an
  apply/restore of that tweak or on explicit refresh — detection must not re-spawn PowerShell per
  status poll (§14).
- Scripts may be inline (YAML block scalar) or filed (`apply: { file: scripts/x.ps1 }`, embedded by
  `build.rs`). Result contract is the exit code (locale-independent): `apply`/`undo` 0 = success;
  `probe` 0 = present, non-zero = absent. All scripts run through the broker's `-EncodedCommand`
  path, so size, loops, quotes, and special characters carry no escaping risk.

## 8. Lifecycle

A tweak is in **exactly one state at a time**; options are mutually exclusive.

### 8.1 Apply(target option)

0. **Acquire the per-tweak lock**, detect current status. Applying the already-active option is a
   verified no-op (no snapshot pushed).
1. **Capture the pre-apply state.** Read the current `Value` of every applicable non-shared Setting on
   the surface (through the elevation each needs — §9); `Missing` is a legal capture for `optional`
   effects. A read that cannot read returns `Err` and **aborts before touching anything** — never a
   fake value. Shared-referenced effects are excluded — their lifecycle is the claims record (§8.6).
2. **Persist the snapshot entry atomically before mutating** (temp → fsync → rename, create-new so a
   lost race is a loud error). The entry is a reference if the pre-apply state is an authored option,
   else a value dump (§8.3) — and it includes the **WAL action journal**: the target option's intended
   action list, all unmarked.
3. **Drive each effect to its desired value in declaration order** via its kind module through the
   broker. Drive-to-value is **idempotent**; inapplicable (version-scoped-out) effects are skipped;
   `Missing` targets follow §5.4. An **omitted undo-carrying probeable Action** whose probe reads
   *present* is driven too: its `undo` runs, verified by the probe reading absent, and is recorded in
   the journal — actions participate in drive-to-state when they can; a no-undo action's Residue is
   left in place and disclosed (§8.4). Shared claims are processed in order like any effect (§8.6). After
   each Action runs, its **completion mark is fsynced into the journal** — on crash, an
   intended-but-unmarked action is *unknown* and the tweak surfaces as **Needs Attention**, never a
   silent skip.
4. **Verify (did-it-work)** — after each Setting, read back and compare; mismatch → `Err`. Actions
   verify via `probe` if present, else the broker's typed exit code.

**Atomic rollback (ADR-0001, structural).** Any failure ⇒ restore the entry captured in step 1 via the
same path as a user Restore (§8.5) — undo the journal's completed actions in reverse order, then drive
the captured state back. The returned error carries **both** the original failure and any rollback
failures. On a verified full restore the just-captured entry is **consumed**; a rollback that cannot
fully complete ⇒ **Needs Attention**, entry kept (ADR-0002). Non-reversible actions that already ran
are reported as un-undoable.

### 8.2 Snapshot history & dedup

Entries are ordered by a **monotonic per-tweak sequence number** (wall-clock timestamps are display
metadata and tiebreak nothing — clocks skew). **Dedup:** at most one entry per authored option — a
re-capture of an option-state **moves that entry to the head** (fresh seq; the old position is
vacated), so Restore always returns through the most recent journey, never a stale position. System
Default and drifted (non-defined) captures are **all kept** — we never drop a state the user actually
had. (Automated SD dedup via the startup verifier is future work — §14; unbounded growth is accepted
for now and bounded in practice by profiles.)

### 8.3 What an entry holds

`{ schema_version, machine_guid, tweak_id, seq, timestamp, captured: OptionRef(label) | Values(map),
   journal: [(action_id, intended|completed)] }`

- **Authored-option captures store the reference only** — restore re-derives from the *current*
  corpus (ADR-0007): the app is the source of truth for authored states, and corpus updates
  intentionally heal what a reference restores to.
- **Unauthored states (System Default, drift) store the full value map** — they exist nowhere else.
  Scripts have no captured state; only the journal is recorded. Shared-referenced effects appear in
  neither (claims record instead).
- An entry that is corrupt, wrong-schema, wrong-machine, **dangling** (its option label or tweak no
  longer exists), or whose target option is unavailable on this machine/build is **invalid**: kept on
  disk, excluded from the walk, surfaced in the UI with an explicit discard affordance — released only
  by user consent (ADR-0002). Never guessed at, never silently deleted.

### 8.4 Detect

Read each applicable, detectable, non-shared Setting once; `optional` effects map `Missing` through
`if_missing`; probeable Actions contribute their cached present/absent; claimed shared settings count
as matching while any claim is held (§8.6). **A matching option always wins; at most one option can
match** (guaranteed by the distinctness guard, §10); **no match ⇒ System Default**. A read that fails
(access denied, malformed packed value, non-optional Missing) ⇒ status **Unknown** — never System
Default, never a guess, with a needs-elevation hint when that is the cause (§9). Options that need an
unsatisfiable value on this machine are flagged **unavailable**. Detection is decoupled from snapshot
history; `is_applied` means only "a history exists to restore from." **Needs Attention is not a
detection verdict** — it is the outcome of an operation that could not fully complete.

**Omitted probeable Actions — the expectation splits on `undo`.** An option that **runs** a probeable
action expects its probe *present*; an option that **omits** it nominally expects *absent*. For an
**undo-carrying** action the expectation is strict — a present reading disqualifies the omitting
option (and apply drives the state, §8.1). For a **no-undo** action, the permanent product of a
one-way run from an earlier state is **Residue**: it never disqualifies the omitting option's match —
the option matches on the rest of its projection — and it is **disclosed** as an info marker on the
active option (the probe tells us it lingers; the tweak is already labelled `reversible: false`).
The distinctness guard closes the loophole this tolerance would otherwise open (§10).

**Cadence.** A full scan starts in the background at launch — the UI renders immediately and statuses
arrive incrementally (frontend stores accept streaming/batched updates; the transport is a plan
detail). After an apply/restore, that tweak's status comes from the operation's own verify reads — no
re-scan. Elevation triggers an automatic full re-scan (Unknowns become readable). **There is no
drift-refresh mechanism in v1** — no watchers, no manual refresh; state changed outside the app is
observed at the next launch or post-Elevate scan.

### 8.5 Restore Snapshot (the only restore action, ADR-0003 + ADR-0007)

Consume the head entry:

1. **Undo** the entry's journal — run `undo` for each completed undo-carrying action, in reverse
   order (these are the actions that ran when the user *left* the state being returned to).
2. **Re-apply the target:**
   - **Option reference** → re-apply that option exactly as *currently defined*: drive its Settings,
     run its actions (ephemerals included), verify probes — a full apply of the target state, minus
     snapshot capture.
   - **Value dump** → drive each stored value back (a captured `Missing` is a defined no-op). Scripts
     cannot be re-run from a dump; if the surface pairs Settings with ephemeral activators, the UI
     shows the standing advisory that a reboot/logoff may be needed for full effect.
3. **Verify.** Success ⇒ the entry is **consumed**; the next-most-recent becomes head; popping the
   last entry leaves the surface matching no option, which simply *reads* as System Default. Any
   failure ⇒ `Err`, entry **kept** for retry; an incomplete restore ⇒ **Needs Attention** (ADR-0001/2).

Restore recomputes shared claims exactly as an ordinary apply of the target state would (§8.6). The
accepted edge (ADR-0007): if an option's surface *grew* since an older dump was captured, walking all
the way back cannot return the new effect to a pre-history value that was never captured — accepted
and documented; in practice rare, and bounded by the corpus rewrite discipline.

### 8.6 Shared claims (runtime)

One engine-level, machine-stamped, atomically-written **claims record** in the snapshots directory:
`{ shared_id → { original: Value, claimants: [tweak_id] } }`.

- **First claim** (corpus-wide): capture the live original **once**, drive to the declared shared
  value, record the claimant.
- **Further claims**: verified no-op; claimant added.
- **Release** (any transition — an ordinary apply **or** a restore — whose target state does not
  claim): claimant removed; the value is left alone while other claimants remain — the releasing
  tweak's result reports *"held by \<tweaks\>"* as info, not failure. **Last release**: drive the
  value back to the captured original, verify, release the record (a verified restore —
  ADR-0002-consistent) — **unconditionally**: an externally-drifted shared value is overwritten by
  the return, exactly as any restore drives captured values over drift. Only Apply ever captures
  drift; a revert means "give me the captured state back."
- Detection: a claimed shared setting matches every claiming option while the claim set is non-empty;
  `unclaimed` entries are excluded from that option's detectable projection (§10 requires options to
  differ on something non-shared).
- Claim operations are serialized behind the claims-record lock; per-tweak locks stay sufficient for
  everything else because addresses are otherwise single-owner (ADR-0006).

### 8.7 Concurrency & batch

A per-tweak-id async lock spans check→capture→save→mutate→verify. Different tweaks may run
concurrently; the same tweak may not. Packed-value field writes serialize behind the registry-kind
mutex; claim ops behind the claims lock. Snapshot saves use create-new semantics. **Batch** runs
tweaks independently — one failure never aborts the rest — and reports **per-tweak** counts.

## 9. Elevation & execution context

See ADR-0005 (as amended). The app ships unelevated (`asInvoker`); Admin is user-provided (launch as
admin or the in-app **Elevate** relaunch), never silently acquired. Four declared levels —
**`User / Admin / System / TI`** — author-declared, trusted, never inferred. A Tweak declares a
**floor**; a step may declare its own level; **effective = max(floor, step)**, escalate-only.

- **User** — in-process as the interactive user (per-user state / HKCU must land in the user's hive).
- **Admin** — in-process in the elevated app; a persistent property of the process once granted.
- **System** — fresh short-lived child from winlogon's duplicated token.
- **TI** — fresh short-lived child via starting the TrustedInstaller service + parent spoofing.

**HKCU exception:** a user-hive effect ignores the floor and always runs in-process as the real user —
never routed to a System/TI child. **Over-the-shoulder guard (new):** at startup the app compares its
process token SID with the interactive session's user SID; on mismatch (a *different* admin's
credentials elevated the app), **User-level (HKCU-touching) tweaks are disabled with a clear message**
— otherwise every HKCU write, read-back, and detection would target the wrong account's hive while
reporting green.

**Reads run at whatever level the app currently has.** Most state is world-readable, so detection
works unelevated; TI-protected resources (WaaSMedic-class keys/tasks) legitimately deny reads — those
tweaks read as **Unknown** with a needs-elevation hint until the user elevates. This replaces the old
"reads never need elevation" claim, which was false exactly for the resources that motivate the TI
path. Reads never trigger elevation; the app never silently escalates for anything.

**Grouped execution (in scope for v1):** consecutive same-level System/TI steps are batched into
**one** child per group — the broker wire protocol already carries `Vec<BrokerOp>`; the multi-op
caller is the net-new wiring, order-preserving. User/Admin steps are in-process and never grouped.
Batching changes the process count, never the prompt count (at most one — the initial Elevate).

**Insufficient elevation names two distinct failures** (both abort + rollback, never a benign value):
*couldn't acquire the level* (TI service unstartable, `SeDebugPrivilege` denied, winlogon absent —
environmental) vs. *acquired but access-denied* (the declaration is genuinely too low on this machine;
the author corrects it). The TI self-availability guard (§10) keeps the app from disabling its own TI
path.

## 10. `build.rs` compile-time guards

Every guard is evaluated **per milestone of the declared support matrix** (an explicit list of
supported builds, e.g. `19045; 22621; 22631; 26100` — exact values finalized in the plan, §14),
over each milestone's **applicable projection** (effects/values in scope for that build):

- **Detectability** — every option has ≥1 **non-optional** detectable effect on every supported
  milestone where the tweak applies. (Optional effects may be Missing on a given machine; an option
  must stay distinguishable without them.)
- **Reachability & distinctness** — per milestone: no two options byte-identical, no two options
  identical on their **detectable projection**, and every pair of options differs on at least one
  **non-shared** effect (claimed shared values are held by other tweaks too, so they cannot be the
  sole distinguisher). A pair may be distinguished solely by an **undo-carrying** probeable Action
  (strict expectations keep matches exclusive), but **never solely by a no-undo Action** — its
  Residue tolerance (§8.4) would let both options match once the action has run.
- **Coverage** — every option covers every Setting effect; shared entries are explicit
  `claim`/`unclaimed`; Action entries are `run` or omitted.
- **Ownership (ADR-0006)** — every address appears **at most once corpus-wide**, counting direct
  effects and `shared` declarations together: a direct effect and a shared declaration on one address,
  two shared declarations on one address, or two effects on one address within one tweak are all
  build errors. A packed value is whole-owned **xor** field-addressed; each field is owned once.
  **Kind canonicalization:** a raw registry effect under `…\Services\<X>\Start` or a task's registry
  storage path is a build error — use the Service/Task kind, so ownership cannot be dodged via a
  second address space.
- **Path syntax** — hive whitelist (HKLM/HKCU, both spellings), no leading/trailing backslash, no
  empty segments, no forward slashes. (This also front-loads the historical `delete_key`
  trailing-backslash hazard as a build rejection; the runtime guard in the registry kind is hardened
  regardless: an empty child name or hive-root deletion is a typed error.)
- **TI self-availability** — no *typed* Service/Registry effect disables or removes the
  TrustedInstaller service. Scripts are statically opaque; the guard's claim is scoped honestly to
  typed effects, and script review guidance carries the residual (§14).
- **Reversibility honesty** — declared `reversible` equals computed (§6.4).
- **Typed-value validation with the runtime parser** — every literal (REG_BINARY hex-pairs included)
  parses with the same code the runtime uses, so build and runtime cannot disagree. `windows:`
  grammar validated (revision requires a pinned build); shared references resolve; shared values are
  type-legal for their kind; the `absent`-literal escape rule enforced (§6.2).

Removed from the guard list (kept as **review guidance**, not build errors, until a decidable
mechanism exists): the risk-disclosure rule ("a setting re-enabling an insecure state cannot be
`risk_level: low`") — `build.rs` has no insecure-state oracle; inventing one silently would be
unreviewed policy.

Schema types stay shared between `build.rs` and the runtime (`models/tweak_schema.rs`), so drift
remains a compile error.

## 11. Module layout, errors & testing

```
src-tauri/src/tweaks/
  model.rs            Effect · Setting · Action · Value · Tweak · Opt   (the one representation)
  engine/
    apply.rs          apply pipeline + WAL journal + atomic rollback
    detect.rs         detection (incl. Unknown, unavailable options, probe cache)
    revert.rs         thin — undo-journal + re-apply / drive-to-value
    lifecycle.rs      per-tweak lock, verify, Needs Attention assembly
  kinds/
    registry.rs       value ops + field parser + auto-create + key presence + delete-tree hardening
    service.rs        ServiceSetting (Missing-aware)
    task.rs           TaskSetting (Missing-aware)
    hosts.rs          HostsSetting     (wraps hosts_service; present/absent)
    firewall.rs       FirewallSetting  (wraps firewall_service; present/absent)
    action.rs         command/script Actions (apply / undo / probe / ephemeral)
  snapshot.rs         atomic storage, seq ordering, refs vs dumps, invalid-entry handling
  shared_claims.rs    claims record: capture-once, refcount, last-release restore
  elevation/          existing broker + grouped-execution caller + SID-mismatch guard
```

The trusted low-level primitives are **reused**: `registry_service` (RegSetValueExW),
`service_control` (SCM), scheduler COM, `hosts_service`, `firewall_service`, and the broker (today's
capture/restore/compare logic lives in `services/backup/*` and is absorbed). Primitives are hardened
as adopted — the `delete_key` guards (empty child name, lone/leading/trailing backslash must never
delete a parent or hive root) land when the registry kind wraps them.

**Snapshots** live in the portable `snapshots/` directory next to the executable (code and three of
four docs already agree; `docs/ARCHITECTURE.md` is corrected as part of the doc realignment).
Atomic write for entries, metadata, and the claims record; `serde` defaults on nested structs;
shared-lock reads; invalid entries per §8.3.

**Error handling.** One `thiserror` type; every effect op returns `Result`; registry reads stay typed
(not-found ≠ access-denied ≠ type-mismatch ≠ malformed-packed-value ≠ resource-missing). Revert shares
apply's path, so no swallow-and-`Ok` branch exists to write. `NeedsAttention` carries the exact
unrecoverable items; `Unknown` carries the exact unreadable effects and why.

**Testing.**
- **Round-trip per kind** against the real registry/service/task on CI: read → apply → verify →
  restore → verify. Field-addressing gets parser round-trip + malformed-input tests (unknown fields
  preserved, order preserved, garbage → typed error).
- **Core invariant as a property test:** apply(any option) then restore ⇒ machine at the pre-apply
  state (dump case) / the option state (ref case).
- **Engine tests with a mock `EffectKind`:** ordering, rollback, Needs-Attention assembly, **WAL
  crash-recovery** (kill between action-run and completion-mark ⇒ Needs Attention, never silent),
  dedup move-to-head, invalid-entry exclusion.
- **Shared claims property tests:** arbitrary claim/release interleavings ⇒ original restored exactly
  once, on the last release; detection honest throughout.
- **Validator regression tests:** known-bad YAML for every §10 guard (duplicate addresses,
  shared+direct overlap, all-optional options, per-milestone identical options, bad paths, revision
  without pinned build, null/empty option values) — each must fail the build.

## 12. Migration plan

- **Engine + schema first.** `tweaks/model.rs`, the engine, kind modules, `shared_claims.rs`, and the
  new `build.rs` validation. `services/*_service.rs` primitives are retained and wrapped.
- **Hard cut, no coexistence.** When the engine + example set is green, the old pipeline dies in the
  same effort: `services/backup/*`, the old tweak-command internals, and **every existing YAML file**
  are deleted. At no point do two engines live in the tree.
- **Example set, not a starter corpus.** A small authored set of examples (one per effect kind, one
  action, one shared group, one packed value, one presence case) exists solely to prove the build and
  the engine end-to-end.
- **Merge at engine-green.** The branch merges to `main` when engine + examples + the full gate pass —
  never held for corpus parity (the app is unreleased; a long-lived branch is the real risk).
- **Corpus rewritten later, on `main`, outside this plan.** The ~189 existing tweak entries are
  research input only: the corpus is re-authored from scratch, per category, with correct ownership
  boundaries, presence declarations, and version scoping. Known content defects (undetectable tweaks,
  colliding owners, wrong stock assumptions, undisclosed insecure options) die in the rewrite.
- **Frontend: minimal functional adaptation, inside the plan** (the hard cut deletes the old
  commands): the new command contract, stores accepting incrementally-arriving statuses, and every
  new state — Unknown, Unavailable, Residue, held-by info, Needs-Attention detail, the 1-option
  toggle, the SID-mismatch notice — rendered with the existing UI primitives. No visual redesign; no
  snapshot-history browser (Restore stays a single head-walk button; select-a-snapshot remains
  future).
- **Clean break, no converters.** The app is unreleased; test installs are disposable. Old on-disk
  snapshots are invalidated by the schema-version bump and treated as no valid snapshot.
- **Docs realigned in the same effort:** `docs/TWEAK_SYSTEM.md`, `docs/TWEAK_AUTHORING.md` (schema,
  house style, presence/shared/versioning semantics, script guidance), `docs/ARCHITECTURE.md`
  (snapshot location), and the ADR set (0002/0003/0004/0005 amendments; 0006/0007 new).

## 13. Failure classes this design eliminates

- **Structural (impossible to express or reach):** one typed representation — capture/apply/detect/
  restore cannot drift; rollback and Restore share one path; a read error is `Err` and detection says
  **Unknown**, never a guess; no stock/default value is authored; the per-tweak lock plus single-owner
  addresses remove write races; snapshots are create-new; `create_key`/`delete_value` cannot destroy
  pre-existing state because they no longer exist — presence is captured, deletion is a driven value.
- **Machine-local reality (typed, not assumed):** resources that may not exist are `optional` with
  declared meaning; unsatisfiable options are unavailable, not failing; TI-protected reads are
  Unknown, not fake-System-Default; a differently-elevated process cannot silently write the wrong
  HKCU hive (SID guard).
- **Cross-tweak (owned, not mediated):** every address has one owner; genuine sharing is a declared,
  refcounted claim — reverting one tweak can no longer flip another to System Default; different
  target values on one address cannot ship.
- **Crash windows (journaled):** the WAL action journal makes "an action ran but nothing recorded it"
  impossible to lose silently — it surfaces as Needs Attention.
- **Build-time (rejected before shipping):** every §10 guard, per supported milestone.
- **Update-time (healing by design):** option snapshots are references, so corpus updates change what
  a restore produces *intentionally and coherently* — never a half-old, half-new fabricated state
  (ADR-0007).

## 14. Decisions deferred to the implementation plan

Each with a recommended default so nothing blocks:

- **Action working directory / timeout / output capture.** Default: no working-dir assumption, a
  bounded timeout, exit-code-only interpretation (stdout captured for logging).
- **Task definition richness.** Default: enabled/disabled boolean only; triggers/actions out of scope.
- **Support-matrix milestone values.** Default: an explicit list (e.g. Win10 22H2 `19045`; Win11
  `22621`, `22631`, `26100`) the validator loops over; exact set finalized in the plan.
- **Probe cache policy details.** Default: per-session cache keyed by tweak, invalidated by that
  tweak's apply/restore and by explicit refresh.
- **`requires_reboot` granularity.** Default: per-tweak flag; per-option needs deferred.
- **SD-snapshot dedup automation.** Future: the startup verifier already prunes entries matching the
  live system; extending it to fold duplicate SD dumps is deferred (growth accepted; profiles bound
  it in practice).
- **Authoring lint pack.** Script-content heuristics (e.g. `sc config TrustedInstaller`), insecure-
  state disclosure list — review guidance until a curated, reviewable mechanism exists.
- **Profile/import layer.** Out of scope; it drives the same apply/restore entry points and
  identifies options by label within a tweak.

## 15. Normative invariants

The non-negotiable rules the implementation and its tests must uphold; each maps to the section that
details it.

1. **One representation.** A change's desired value, captured value, current reading, and detection
   comparison use one typed `Value` and one comparison per kind (§5).
2. **Did-it-work.** Every effect op returns `Result`; a failed apply/read/revert surfaces as `Err`,
   never a benign value; no `let _ =` on a privileged call (§8/§11).
3. **Unknown, never a guess.** An unreadable or unparseable state (access denied, malformed packed
   value, non-optional Missing) is status **Unknown** — never System Default, never a fabricated
   value (§8.4).
4. **Capture before mutation.** Every apply captures the pre-apply state (references for authored
   options, dumps for unauthored states) before any mutation; an unreadable value aborts pre-mutation
   (§8.1/§8.3).
5. **WAL journal.** The intended action list is persisted before mutation; each completion is marked
   durably after it runs; on crash, intended-but-unmarked ⇒ Needs Attention, never a silent skip
   (§8.1).
6. **Dedup moves to head.** At most one entry per authored option; re-capture takes a fresh head
   position (monotonic seq — never wall-clock ordering); unauthored captures are all kept (§8.2).
7. **Restore = undo + re-apply.** Consume the head: run the entry's completed undos in reverse, then
   re-apply the target — option refs from the *current* corpus definition (actions and ephemerals
   included), dumps by driving stored values (§8.5, ADR-0007).
8. **Snapshot release rules.** An entry is deleted only on a verified restore, the startup
   stale-cleanup's verified match (checkable Settings only), or explicit user consent — never on
   uncertainty; invalid/dangling entries are kept, excluded, surfaced, and released only by consent
   (§8.3, ADR-0002).
9. **Options are mutually exclusive**; a matching option always wins; at most one can match; no match
   ⇒ System Default — a computed status, never authored, never a restore target (§6.1/§8.4, ADR-0003).
10. **Reversibility is typed and computed** — Settings, undo-carrying Actions, and ephemeral Actions;
    one non-ephemeral no-`undo` action makes the tweak one-way, surfaced before apply; everything
    else still reverts (§6.4/§7).
11. **Detectability is typed** — Settings always; Actions iff `probe`; there is no `skip_validation`
    flag (§6.4).
12. **Presence is typed.** `Missing` is capture-only with author-declared meaning
    (`optional`/`if_missing`); driving to `Missing` is a defined no-op; the engine never installs or
    uninstalls resources; unsatisfiable options are unavailable-on-this-machine, not silently skipped
    (§5.4).
13. **`absent` is the only absence spelling**, uniform at value/key/field depth and across every
    value type: a bare `absent` is always the keyword, `{ literal: … }` is the escape for a string
    whose content is that word, and a null/empty/omitted value is a build error naming `absent`
    (§6.2, ADR-0004 as amended).
14. **No unconditional structural inverses.** Parent keys auto-create on apply; key existence is a
    captured `Present(bool)` Setting; `create_key`/`delete_value` Actions do not exist; delete-tree
    is explicitly one-way unless the author supplies `undo` (§5.1/§7).
15. **Field ops preserve what they don't own** — unknown fields and order survive every write; a
    malformed live string is a typed error; field writes are serialized (§5.2).
16. **One address, one owner** corpus-wide (direct xor shared, once; whole-value xor field-addressed;
    canonicalized kinds close the alias routes) (§10, ADR-0006).
17. **Shared claims refcount.** The shared block declares the single target value; the first claim
    captures the original once; the last release restores it verified and **unconditionally**
    (external drift is overwritten by the return); detection counts claimed
    settings for every claimant; releasing early reports "held by", not failure (§8.6, ADR-0006).
18. **Effects apply in declaration order**; grouped System/TI batching is order-preserving (§8.1/§9).
19. **Probe and read-back are state-based**, identical at apply-time and detect-time; probe results
    are session-cached and invalidated by that tweak's mutations (§7/§8.4).
20. **Rollback restores the just-captured entry** via the same path as Restore; the error carries both
    the original and rollback failures; verified ⇒ consumed, incomplete ⇒ Needs Attention with the
    entry kept (§8.1, ADR-0001).
21. **Snapshots are portable and machine-bound** — next to the exe, create-new, atomically written
    (entries, metadata, claims record); corrupt/wrong-schema/wrong-machine/dangling ⇒ "no valid
    snapshot", never a valid prior (§8.3/§11).
22. **Version scoping is total.** `windows { products / build / revision }` per §6.6; scoped-out
    effects are excluded from apply, detect, and guards; an empty applicable surface makes the tweak
    unavailable with a reason; the running build comes from `RtlGetVersion` + `UBR` (§6.6).
23. **Guards are quantified per supported milestone** over the applicable projection — detectability
    (≥1 non-optional detectable effect), distinctness (byte, detectable-projection, and non-shared),
    coverage, ownership, path syntax, reversibility honesty, typed-value round-trip (§10).
24. **Elevation is declared, never inferred**; effective = max(floor, step), escalate-only; User/Admin
    in-process, System/TI in fresh children; an HKCU effect always runs as the interactive user, and a
    token-SID/session-SID mismatch disables User-level tweaks; reads run at the current level and
    surface Unknown when denied; "couldn't acquire" and "acquired but denied" are distinct failures
    (§9, ADR-0005 as amended).
25. **Batch operations are per-tweak-independent** and report per-tweak counts (§8.7).
26. **Omitted probeable Actions split on `undo`.** An option omitting a probeable Action nominally
    expects its probe absent: strict for undo-carrying actions — and apply drives the state by
    running `undo` — tolerant for no-undo actions, whose **Residue** never disqualifies a match, is
    always disclosed, and can never be an option pair's sole distinguisher (§8.1/§8.4/§10).

## Appendix A — Resolved-decision log (2026-07-22 review)

| Topic | Resolution | Where |
|---|---|---|
| Restore semantics | Undo recorded actions (reverse), then re-apply target; option refs re-derive from current corpus | §8.5, ADR-0007 |
| Dangling snapshot refs | Invalid: kept, excluded, surfaced, consent-only release | §8.3 |
| Surface-growth edge | Accepted + documented | §8.5 |
| Action bookkeeping | WAL journal: intent pre-mutation, durable completion marks, crash ⇒ Needs Attention | §8.1 |
| Resource presence | Typed `Missing`, `optional` + `if_missing`, unavailable options, no install/uninstall | §5.4 |
| Task name patterns | Removed — exact names only (corpus rewritten) | §5.1 |
| Value deletion | Authored `absent` keyword (value/key/field); `delete_value` retired; ADR-0004 amended | §6.2 |
| `create_key` data loss | Auto-create parents + `RegistryKey` presence Setting; `create_key` retired | §5.1 |
| Packed values | Field addressing, `kv_semicolon` parser, no regex; whole xor field ownership | §5.2, §10 |
| Detect read failure | **Unknown** status; reads at current level; needs-elevation hint | §8.4, §9 |
| Windows versioning | `products` (set, sugar) / `build` / `revision` grammar; H-names dropped; tweak/effect/value scoping | §6.6 |
| Cross-tweak sharing | One address one owner + corpus-level `shared` block with claim/release refcount | §6.5, §8.6, ADR-0006 |
| Over-the-shoulder UAC | Token-SID vs session-SID guard; User-level tweaks disabled on mismatch | §9 |
| Migration | None needed — unreleased app; corpus rewritten from scratch | §12 |
| `risk_level` | All four kept: low/medium/high/critical | §6.4 |
| Duplicate addresses | Total ban incl. shared/direct overlap, per-tweak dupes, mode mixing | §10 |
| Registry paths | Merged `HKLM\…` single string; HKLM/HKCU v1; path-syntax build rules | §5.1, §10 |
| `.reg` conventions | Types/hex/binary/multi-sz literals adopted; `-` spellings and `.reg` import rejected | §6.2 |
| Grouped execution | **Stays in v1** (elevation is a foundational pillar; no deferral) | §9 |
| House style | Flow-form addresses, block elsewhere, no sugar in v1 | §6.7 |
| Detection cadence | Background launch scan, statuses stream in; post-op status from verify reads; full rescan on Elevate; no drift refresh in v1 | §8.4 |
| Omitted probeable Actions | Expected-absent split on `undo`; drive-back via `undo` on apply; Residue disclosed, never a sole distinguisher | §8.1, §8.4, §10 |
| Shared release vs drift | Last release restores the captured original unconditionally | §8.6 |
| Cutover | Hard cut deletes old pipeline + all old YAML at engine-green; examples only; merge at engine-green; corpus later on `main` | §12 |
| Frontend scope | Minimal functional adaptation in-plan; no visual redesign; no history browser | §2, §12 |
