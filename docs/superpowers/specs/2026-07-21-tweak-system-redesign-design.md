# Tweak System Redesign — Design

Status: **approved (design)** · Date: 2026-07-21 · Supersedes the effect/apply/backup layer described in
`docs/TWEAK_SYSTEM.md`.

## 1. Context & motivation

The current tweak system splits a single registry/service/task change into **four separate
representations** — a YAML change, a captured snapshot, a restore op, and a detection comparison — spread
across `services/*_service.rs`, `backup/capture.rs`, `backup/restore.rs`, and `backup/compare.rs`. Those
representations drift, and the drift is where correctness dies: state can be lost on a failed rollback, a
revert can silently do nothing, and detection can disagree with what was applied.

This redesign rebuilds the effect-execution layer and the lifecycle around it so that correctness is
enforced by **types and the compiler**, not by convention. It is a first-principles rebuild, not a
patch-list — the rules below stand on their own merit.

## 2. Goals & non-goals

**Goals**
- One representation per change: apply, capture, detect, and revert consume the *same* typed value.
- Reversibility and detectability are **typed properties**, so the system never claims an operation it
  cannot perform (the ADR-0001 honest-failure principle made structural).
- The did-it-work contract is unavoidable: a failed effect surfaces as `Err`, never a benign value.
- Build-time validation proves the properties that today only fail at runtime (undetectable tweaks,
  unreachable options, tweaks that fight over the same state).

**Non-goals**
- Not rewriting the Svelte frontend. The Tauri command contract adapts as needed but the UI model
  (status, the System-Default *status*, Restore Snapshot, Needs Attention) is preserved.
- Not migrating the full ~189-tweak corpus in this effort (see §12). The engine ships with a small
  converted starter set; the rest is follow-up.
- No dual-schema compatibility layer. Clean break (see §12).

## 3. Decisions (the spine)

These were settled during brainstorming and are fixed for this design:

| Decision | Choice |
|---|---|
| **Scope** | Executors + shared lifecycle + snapshot format + YAML schema + `build.rs`. Frontend adapts. |
| **Core model** | Declarative typed **Settings** (desired value per option + runtime-captured value) with one representation feeding apply/detect/restore; imperative **Actions** as an explicit escape hatch. |
| **Escape hatch** | Actions carry a required `apply` and **optional** `undo` and `probe`. Absent `undo` ⇒ typed non-reversible; absent `probe` ⇒ typed non-detectable. Honest one-way behavior, never silent. |
| **Engine structure** | Hybrid: effect **data** is a serializable enum (exhaustiveness, atomic ordering, snapshots for free); each kind's **behavior** lives in its own deep module behind an `EffectKind` trait. |
| **Migration** | Engine + schema first with a small converted starter set; full corpus later; clean break; old on-disk snapshots invalidated by a schema-version bump. |

## 4. Architecture overview

```
YAML corpus  ──build.rs──▶  compiled Tweak model  (data: Vec<Opt>, each = value-map over a shared surface)
                                     │
                                     ▼
                        Engine  (lifecycle: apply · detect · restore · verify · atomic rollback · per-tweak lock)
                                     │  dispatches each Effect (enum) to →
                                     ▼
        EffectKind modules  (registry / service / task / hosts / firewall / action — read+apply+revert+detect co-located)
                                     │  execute through →
                                     ▼
                    Execution context  (existing typed elevation broker: user / admin / system / TI)
```

- The **Engine** owns lifecycle, ordering, verification, rollback, and the per-tweak lock.
- Each **EffectKind module** owns how *one kind* reads/applies/reverts a value.
- The **broker** owns privilege. It is reused as-is (it already replaced shell-string elevation with a
  typed, verified path); it gains a grouped-execution entry point (§9).
- **Snapshots** are serialized captured `Value`s — the same type as the corpus — so a capture and its
  restore are one code path, not two.

## 5. The core abstraction — `Effect`

An `Effect` is one atomic unit of change, in one of two categories:

- **`Setting`** — declarative, always reversible. Addresses a piece of typed system state and names a
  desired value. Kinds: `Registry` (a typed value at hive\key\name), `Service` (a service's startup
  type), `Task` (a scheduled task's enabled-state), `Hosts` (a hosts-file entry — present/absent), and
  `Firewall` (a firewall rule — present/absent). Operations: **read** → current value, **apply** →
  drive to desired, **revert** → drive to the captured value. *Detect is read + compare.* All four share one
  address+value type and one comparison, so they cannot disagree.
- **`Action`** — imperative, optionally reversible. A `cmd`/`powershell` script or a structural registry
  op (key create / key-tree delete). Carries `apply` (required), `undo` (optional), `probe` (optional).

```rust
enum Effect  { Setting(Setting), Action(Action) }
enum Setting { Registry(RegAddr), Service(SvcAddr), Task(TaskAddr), Hosts(HostsAddr), Firewall(RuleAddr) }
enum Value   { Absent, Reg(TypedRegValue), Startup(StartupType), TaskEnabled(bool), Present(bool) }
```

`TypedRegValue` covers DWORD / QWORD / SZ / EXPAND_SZ / MULTI_SZ / BINARY. `StartupType` covers
Boot / System / Automatic / AutomaticDelayed / Manual / Disabled. A service or task that does not exist
is its own read state (`NotInstalled` / `NotFound`), distinct from any value and from a read error. A
`Hosts` / `Firewall` address (a specific `ip↔domain` mapping, resp. a named rule) reads as `Present(bool)`,
so add/remove and create/delete are ordinary drive-to-value operations — reversible and detectable by
construction, exactly like the other Settings.

**First-class kinds:** Registry / Service / Task / **Hosts / Firewall** settings and `cmd` / `powershell` /
key actions. Hosts and firewall stay **dedicated typed kinds** (as they are today — `HostsChange`,
`FirewallChange` with real services), *not* demoted to free-form scripts: modelling them as Settings keeps
their reversibility and detectability enforced by the type system rather than by hand-written `undo` /
`probe` scripts. Their low-level services (`hosts_service`, `firewall_service`) are reused and wrapped by
`kinds/hosts.rs` / `kinds/firewall.rs`, the same way the registry/service/task primitives are.

## 6. Data model & YAML schema (effect-centric)

The schema moves from *option-centric* (each option re-lists its own changes) to **effect-centric**: a
tweak declares its **managed state surface once**, and every option supplies values over that surface.
This makes it structurally impossible for two options to touch inconsistent address sets — the root of the
class where one option forgets a setting another sets, two options collide on shared state, or two options
end up byte-identical.

```yaml
id: disable_telemetry
name: "Telemetry"
risk_level: low
elevation: admin            # per-Tweak FLOOR: user | admin | system | ti (declared, never inferred)
reversible: true            # computed: all effects are Settings, Actions with `undo`, or ephemeral
                            #   Actions (§7) — none one-way here. Build-checked against the real value.

effects:                    # the managed surface — declared ONCE
  - id: allow_telemetry     # Registry Setting (HKLM) — runs at the admin floor
    registry: { hive: HKLM, key: "...\\DataCollection", name: AllowTelemetry, type: REG_DWORD }
  - id: tailored_ads        # Registry Setting (HKCU) — ignores the floor, runs in-process as the real user (§9)
    registry: { hive: HKCU, key: "...\\AdvertisingInfo", name: Enabled, type: REG_DWORD }
  - id: diagtrack           # Service Setting
    service: { name: DiagTrack }
    elevation: ti           # per-STEP escalate: effective = max(admin floor, ti) = ti; the rest stay admin
  - id: block_vortex        # Hosts Setting (typed kind — present/absent, reversible by construction)
    hosts: { ip: 0.0.0.0, domain: vortex.data.microsoft.com }
  - id: flush_dns           # ephemeral Action — transient side-effect, no undo/probe, exempt from
                            #   reversibility & detectability (never makes the tweak one-way)
    command:
      shell: powershell
      ephemeral: true
      apply: "Clear-DnsClientCache"

options:                    # ONLY the real states we offer — "System Default" is NOT authored
  - label: "Telemetry Off"
    values:
      allow_telemetry: 0
      tailored_ads: 0
      diagtrack: disabled
      block_vortex: present        # add the hosts entry (revert removes it)
      flush_dns: run               # run the action on apply for this option
```

**"System Default" is a computed status, never an authored option.** Authors list only the real states a
tweak offers; the app shows **System Default** when the live surface matches *none* of them. There is no
`is_default` option and no declared stock/default values anywhere. Consequently the UI states are
*authored options + 1 implicit System Default*, which shifts the shape rule: **1 authored option ⇒ toggle**
(Default ↔ On), **≥2 authored ⇒ dropdown** (Default / A / B …). The **Default** position is the computed
status (§8), not a selectable target: the user leaves it by applying an option and returns toward it via
**Restore Snapshot** (§8), never a "revert to Default" action.

Compiled model:

```rust
struct Tweak     { id, name, risk_level, elevation: Level, reversible: bool, surface: Vec<EffectDef>, options: Vec<Opt> }
struct EffectDef { id, kind: Effect, elevation: Option<Level> }   // step level; effective = max(tweak floor, this)
enum   Level     { User, Admin, System, Ti }   // declared, trusted, never inferred (§9, ADR-0005)
struct Opt       { label, values: Map<EffectId, OptValue> }   // only non-default states; System Default is computed
enum   OptValue  { Set(Value), Run }   // Settings carry a Value; an Action entry is Run or omitted
```

**Consequences:**
- **No authored defaults at all.** Because the default state is computed (no option matches), there are
  no declared stock/default values anywhere, so an author cannot guess a stock default wrong, and two
  tweaks cannot disagree about one. Returning toward a prior state is handled by **Restore Snapshot**,
  which walks the captured return-point history (§8), never a hand-authored default. (The display label for this computed state is cosmetic — "System
  Default" is just the default name.)
- **Build-checkable structure.** Because the surface is explicit and each option is a value-map over it,
  `build.rs` can prove no two options are byte-identical, no option omits a managed setting, and no two
  tweaks manage the same address with option values that fight each other. See §10.
- **Reversibility is computed.** `reversible` = every effect is a Setting, an Action with an `undo`, **or
  an `ephemeral` Action** (transient side-effects don't count against it — §7). Build-checked against the
  declared flag and surfaced to the UI, so a one-way tweak is labelled up front, not discovered at restore
  time.

**Windows-version applicability.** An effect (or a specific value) may be scoped by OS **major version
and/or build number** — `windows: { version: 11, build_min: 22621, build_max: 26099 }`, every field
optional: a bare `version: 11` scopes to a whole major line, while a `build_min` / `build_max` range gates
a specific feature window (e.g. 22H2 → 23H2). On a version/build where it does not apply it is **excluded
entirely** — not applied, not read, not counted — and the detectability guard (§10) is evaluated *per
supported build milestone* (the declared support matrix, §14), so an all-scoped-out option cannot ship
undetectable. The running build is read via `RtlGetVersion` (**not** `GetVersionEx`, which the
compatibility shim under-reports on an unmanifested process). There is **no `skip_validation` flag** — it is
removed. Detectability is instead a typed property: **Settings are always detectable; Actions are
detectable iff they declare a `probe`.** (A blanket apply-but-do-not-validate flag was the direct cause of
the undetectable-tweak class in the old system and has no replacement by design.)

**Tweak metadata.** Beyond `id` / `name` / `risk_level` / `reversible`, a tweak carries the descriptive
and UI fields the current model has — `description`, `category`, optional `info` / `warning`, and an
optional `requires_reboot: bool` (default `false`, surfaced to the UI). `requires_reboot` is optional with
a default, matching the code and keeping the authoring docs honest. `risk_level ∈ {low, medium, high}` is
advisory and feeds the disclosure guard (§10). It also carries its declared **`elevation`** floor
(`user | admin | system | ti`) plus an optional per-effect `elevation:` that can escalate a single step
above the floor (§9).

## 7. Action contract (the escape hatch)

Actions exist for free-form scripts that cannot be expressed as declarative settings (loops,
conditionals, multi-step logic). The contract:

- **`apply` is required**; **`undo`, `probe`, and `ephemeral` are optional and independent.**
  - `undo` present ⇒ that action reverts cleanly; absent ⇒ that action is one-way.
  - `probe` present ⇒ that action contributes to detection; absent ⇒ it does not.
  - `ephemeral: true` ⇒ a **transient side-effect that changes no persistent state** (flush DNS, clear the
    icon/thumbnail cache, restart Explorer, `gpupdate /force`): it runs on apply, takes **no** `undo` or
    `probe`, and is **exempt** from the reversibility and detectability computations.
- **Whether an action needs `undo` follows one question — *can the change it makes persistently alter or
  break the system?*** Changing a service's start type can (→ it needs `undo`); flushing DNS or clearing a
  cache cannot (→ `ephemeral`, no `undo`). A single **non-ephemeral** no-`undo` action makes the whole
  tweak `reversible: false`; on restore, all Settings and undo-carrying actions still revert, only the
  genuinely one-way actions cannot, and *those* are surfaced as Needs Attention. "Partial" never means
  "nothing reverts," and an ephemeral action never makes a tweak one-way.
- **`probe` is state-based, not history-based, and exists because scripts have no readable state.** A
  Setting is verified by reading it back directly; a `cmd` / `powershell` Action produces an effect the
  engine cannot introspect, so the author supplies a `probe` that answers *"is the state this action
  produces currently present?"* — never *"did the script run."* The same probe serves both roles: apply-time
  (the did-it-work check for that action) and detect-time (its present/absent contribution). (Structural key
  Actions need no author probe — key existence is read directly, a system-provided probe.)

**Authoring ergonomics — scripts may be inline or filed, author's choice per action:**

```yaml
# short → inline YAML block scalar (keeps newlines, readable)
apply: |
  Get-AppxPackage -AllUsers *SomePkg* | ForEach-Object {
    Remove-AppxPackage -Package $_.PackageFullName -AllUsers
  }

# substantial → external file, embedded by build.rs (real editor tooling / linting / testing)
apply: { file: "scripts/debloat_apps.ps1" }
undo:  { file: "scripts/restore_apps.ps1" }
probe: { file: "scripts/probe_apps.ps1" }
```

**Result contract is an exit code (locale-independent, no stdout string-parsing):**
- `apply` / `undo` → exit `0` = success, non-zero = failure (feeds the did-it-work contract).
- `probe` → exit `0` = the produced state is **present** (applied/on), non-zero = **absent** (not-applied).
  The same probe serves both roles: at **apply** time it is the action's did-it-work check (non-zero after
  `apply` ⇒ the effect isn't confirmed ⇒ fail + rollback); at **detect** time it is the present/absent
  contribution to option-match — never a Needs-Attention verdict (detection isn't a verdict, §8).

All scripts run through the broker's existing `-EncodedCommand` path (base64 of the whole script block),
so **size, loops, quotes, and special characters carry no shell-escaping risk regardless of length**.
Structural registry key ops (create / delete-tree) are Actions: `create_key` gets a system-provided
inverse (delete the created key); `delete_key` on a subtree is one-way unless the author supplies an
`undo` (e.g. a `.reg` restore).

## 8. Lifecycle

**Apply(target option)** — one ordered, fail-closed pipeline. A tweak is in **exactly one state at a
time** — options are mutually exclusive.
0. **Acquire the per-tweak lock** and read current status (Detect, below). Applying the already-active
   option is a verified no-op.
1. **Capture the pre-apply state as a snapshot.** Before mutating, read the current `Value` of every
   applicable **Setting** on the surface (through the elevation each needs) and push it onto the tweak's
   snapshot **history** — the return-point for *"where the system was before this apply."* (Script Actions
   carry no captured state; only `actions_run` is recorded.) If the pre-apply state **is an authored
   option**, its snapshot **dedups** — one entry per option, replace/rename the existing one, never a
   duplicate; if it is **System Default or any other non-defined state**, the entry is **kept** (never
   deduped — we never drop a state the user actually had). All Settings are read **before any mutation**, so
   the return-point is complete; a read that cannot read returns `Err` and aborts before touching anything —
   never a fake "absent".
2. **Persist the new snapshot atomically** (temp → fsync → rename) *before* mutating — create-new, so a lost
   race is a loud error, never a silent overwrite of an existing return-point.
3. **Drive each effect to its desired value in declaration order**, via its EffectKind module through the
   broker. Drive-to-value is **idempotent** — a setting already at target is written-and-verified, not
   skipped blindly, so retries are safe. **Inapplicable effects** (scoped out of the running OS version)
   are skipped entirely.
4. **Verify (did-it-work)** — after each Setting, read back and compare to desired; mismatch → `Err`.
   Actions verify via `probe` if present, else the broker's typed exit code.

**Atomic rollback (ADR-0001, structural).** Any failure ⇒ **restore the snapshot just captured in step 1**
(the pre-apply state) via the same drive-to-value path as a user Restore — so there is no separate "restore"
branch that can silently return success. Restoring the pre-apply state is correct no matter how far the
apply progressed: drive-to-value is absolute, so partial progress needs no per-step tracking. The returned
error carries **both** the original failure and any rollback failures, so a swallowed rollback result cannot
happen. **On a verified full restore** the system now matches that snapshot, so the just-captured snapshot
is **consumed** (removed) — you are back exactly where you started the attempt, with prior history intact. A
rollback that **cannot fully complete** ⇒ **Needs Attention**, snapshot **kept** (ADR-0001/0002 — a
return-point survives any *incomplete* restore, released only on a verified match, never on uncertainty).
Non-reversible Actions that already ran are reported as un-undoable.

**Detect.** Read each detectable Setting once and compare the live surface to each option under one typed
compare (probeable Actions contribute their applied/not result). **A matching option always wins:** if the
live surface matches an authored option, that option is shown as selected. At most one option can match,
because the build guard makes option value-maps mutually distinct on the **detectable** surface (§10). **If
no option matches, the status is System Default** — "the author never defined this state, or the surface
has drifted out of every defined one." Detection is therefore **decoupled from the snapshot history**:
System Default is a *status*, never a restore target, so return-point snapshots may still exist (they only
drive the Restore button). `build.rs` guarantees ≥1 detectable effect per option, so an option is always
distinguishable from System Default and from its siblings. `is_applied` means simply *"the tweak has a
snapshot history to restore from."* **Needs Attention is not a detection verdict** — it is the separate
outcome of a rollback/restore that could not fully complete (§8 rollback, ADR-0001).

**Restore Snapshot (the only restore action, per ADR-0003).** There is **no "revert to System Default."**
The sole restore action drives the system to the **most recent** snapshot (drive-to-value; run `undo` for
undo-carrying actions; verify). On success the system now matches that snapshot, so it is **consumed**
(removed) and the next-most-recent becomes the head — pressing Restore again walks back through the history,
and popping the last entry leaves the surface matching no option (which simply *reads* as System Default).
Any failure ⇒ `Err` ⇒ the snapshot is **kept** for retry; a snapshot is deleted **only** on a verified match
(a fully-verified restore or the startup stale-cleanup) or an explicit `keep_current_state` decision
(ADR-0002). A future *select-a-snapshot* control may restore a specific entry; the default is always
most-recent.

**Concurrency.** A per-tweak-id async lock spans the whole check→capture→save→mutate→verify sequence, and
`save` uses create-new semantics so a lost race is a loud error, not silent loss of a captured return-point.
Different tweaks may apply concurrently; the same tweak may not.

**Batch.** Batch apply/restore runs tweaks **independently** — one tweak's failure never aborts the others
— and reports counts **per tweak** (`applied N/total`), never per sub-operation, so the failed count can
never exceed the total.

## 9. Elevation & execution context

See ADR-0005 for the full decision. The app ships unelevated (`asInvoker`); Admin is **user-provided, never
silently acquired** — the user raises the whole app to Admin by launching as administrator or clicking the
in-app **Elevate** button (a one-time `restart_as_admin` UAC relaunch). While the app is not Admin, Tweaks
needing Admin / System / TI are **disabled** in the UI; status is still detected read-only (reads never need
elevation), only *changing* is blocked.

**Four declared levels — `User / Admin / System / TI`** (the successor to today's
`requires_admin` / `requires_system` / `requires_ti` flags; `None` is renamed **`User`** to say what it
means). The level is **author-declared, trusted, and never inferred** (a build-time guess is wrong on some
machine — ADR-0005). It maps to *execution context*, not to a single "elevated child":

- **User** — runs **in-process as the real interactive user**. Per-user state (HKCU) must use this so it
  lands in the *user's* hive, never System's or TI's.
- **Admin** — runs **in-process in the already-elevated app** (an HKLM write is a direct `RegSetValueExW`);
  no child is spawned. Admin is a **persistent property of the process** once granted — it is *not*
  established or released per Tweak.
- **System** — runs in a **fresh short-lived child** launched from winlogon.exe's **duplicated token**.
- **TI** — runs in a **fresh short-lived child** launched by **starting the TrustedInstaller service and
  parent-process-spoofing off it** — a heavier path than System, and one that *depends on the TI service
  being startable* (see the failure split below).

From an already-Admin process, System and TI **add no further UAC prompts** — they are reached through the
existing typed broker (token duplication for System; service-start + parent spoof for TI).

**Declared per Tweak, refinable per step — the floor is the Tweak, a step may escalate.** A Tweak declares
a **floor** level for all its steps; a step may declare its own level, and its **effective level is the
higher of the two** — `max(tweak_floor, step_level)`. A step escalates above the floor for a resource that
needs more, but never lowers it: with a floor of `Admin` and one service step marked `TI`, only that
service runs as `TI` and every other step runs at `Admin`; with a floor of `TI`, a step marked `Admin`
still runs at `TI` (the floor wins). **Each step then routes to the context its effective level needs:**
`User` / `Admin` run in-process (as the user, resp. in the elevated app); `System` / `TI` run in their
child. The **one exception is correctness, not privilege:** a **user-hive (HKCU) effect ignores the floor**
and always runs in-process as the real user — never routed to a System/TI child even under a System/TI
floor — because it must land in the *user's* hive, not System's or TI's.

**Grouped execution is a spawn optimization for the child levels only.** Consecutive same-level System/TI
steps are batched into **one** child (the broker wire protocol already carries a `Vec` of ops, but the
multi-op caller is **net-new wiring** — today every op spawns its own child). `User` / `Admin` steps are
in-process and are never grouped into a child. Batching changes the process count, not the number of UAC
prompts — there is at most one, the initial Elevate.

**Insufficient / unavailable elevation fails cleanly — and names two distinct cases.** The engine **never
silently escalates**: the level shown to the user equals the level used. Both cases surface as a distinct
**insufficient-elevation** failure (abort + Rollback), never a benign value:
- **Couldn't *acquire* the declared level** — the token could not be assumed at all: the TI path could not
  **start the TrustedInstaller service** (disabled/blocked), `SeDebugPrivilege` was denied, or winlogon was
  not found. This is environmental or self-inflicted, *not* a mis-declared level.
  - **Self-deadlock guard (this is a debloat tool):** a Tweak that **disables the TrustedInstaller service**
    is rejected at build — it would break the app's own TI path for every later TI Tweak (§10).
- **Acquired the level but the operation was still access-denied** — the declared level is genuinely too low
  on this machine; the remedy is for the author to correct the declaration (the required privilege varies by
  machine, so no build guess is right everywhere).

## 10. `build.rs` compile-time guards

Because the surface is explicit data, the validator can prove properties that today only fail at runtime.
Each becomes a build error:
- **Detectability** — every option has ≥1 detectable effect on every supported Windows version.
- **Reachability & detectable distinctness** — no two options share a byte-identical value-map, *and* no
  two options are identical on their **detectable projection** (the Settings plus probeable Actions —
  non-`probe` Actions contribute nothing to detection). This closes the gap where two options differing
  only by a non-detectable Action would build clean yet be indistinguishable at detect time, breaking
  "at most one option can match" (§8). Every option also covers every managed setting.
- **Cross-tweak ownership** — no two tweaks manage the same registry address / service / task with option
  values that would fight each other. Composite string keys (e.g. `DirectXUserGlobalSettings`) are modeled
  as structured multi-field settings so co-ownership is safe.
- **TI self-availability** — no Tweak sets the `TrustedInstaller` service to Disabled (or deletes it):
  disabling the app's own elevation path would break every later TI Tweak (§9).
- **Reversibility & disclosure** — declared `reversible` equals computed (`ephemeral` actions don't count
  against it); a setting re-enabling an insecure state cannot be silently `risk_level: low`.
- **Typed-value validation with the runtime parser** — REG_BINARY hex (and every typed value) is validated
  at build with the same code the runtime uses, so build and runtime cannot disagree.

Schema types stay shared between `build.rs` and the runtime (`models/tweak_schema.rs`), extended for the
new model, so drift remains a compile error.

## 11. Module layout, errors & testing

**Layout — consolidate each kind's behavior into one deep module** (the co-location the hybrid structure
buys). Today a kind's logic is smeared across `services/*_service.rs` + `backup/{capture,restore,compare,
detection,inspection}.rs`; those collapse into `kinds/*` + `engine/detect.rs`.

```
src-tauri/src/tweaks/
  model.rs            Effect · Setting · Action · Value · Tweak · Opt   (the one representation)
  engine/
    apply.rs          apply pipeline + atomic rollback
    detect.rs         detection
    revert.rs         thin — reuses apply's drive-to-value
    lifecycle.rs      per-tweak lock, verify, Needs Attention assembly
  kinds/
    registry.rs       RegistrySetting read/apply/revert (value ops) + key Actions
    service.rs        ServiceSetting
    task.rs           TaskSetting
    hosts.rs          HostsSetting     (wraps hosts_service; present/absent)
    firewall.rs       FirewallSetting  (wraps firewall_service; present/absent)
    action.rs         command/script Actions (apply / undo / probe)
  snapshot.rs         atomic storage, schema version + machine guid, corrupt-state handling
  elevation/          existing broker + grouped-execution entry point
```

The trusted low-level Windows primitives are **reused**: `registry_service` (RegSetValueExW),
`service_control` (SCM), scheduler COM, `hosts_service`, `firewall_service`, and the broker. The kind
modules wrap them and absorb the drift-prone capture/restore/compare logic. Primitives are **hardened as they are adopted** — e.g. the
`delete_key` empty-parent guard (a trailing/lone backslash must never delete the parent key) is fixed when
the key-delete Action wraps it, rather than carried forward unexamined.

**Snapshots.** A per-tweak **history** of return-points in a portable `snapshots/` directory **next to the
executable** (portable-app design — resolves the current code/doc location disagreement in favour of
next-to-exe). Each entry: `{ schema_version, machine_guid, tweak_id, captured_option, values:
Map<EffectId, Value>, actions_run, irreversible_actions_run, seq }`. A captured state that **is** an
authored option is stored as the **option reference** (`captured_option`), its `values` re-derived from the
corpus — so a schema change only risks the **non-defined captures** (System Default and drifted states),
which hold real `values` and are the sole migration-sensitive entries. **Dedup:** at most one entry per
authored option (replace/rename on re-capture); non-defined captures are all kept. **Startup stale-cleanup:**
an entry whose held state matches the current live system is removed — the comparison uses only **checkable
Settings** (registry / service / task / hosts / firewall), never cmd/powershell Actions (a script has no
reliable state to diff), and a System-Default capture contains no scripts anyway, so it is always fully
checkable. Atomic write for **both** the entry and the metadata update, `serde` defaults on nested structs
(older snapshots still deserialize), shared-lock reads. An entry that is corrupt/unparseable, carries an
older `schema_version`, or whose `machine_guid` ≠ the current machine is treated as **no valid snapshot** —
never mistaken for a valid prior state.

**Error handling.** One `thiserror` type; every effect op returns `Result`; registry reads stay typed
(not-found ≠ access-denied ≠ type-mismatch). There is structurally nowhere to `let _ =` a privileged
call because revert shares apply's path — no swallow-and-`Ok` branch exists to write. `NeedsAttention` is
a first-class outcome carrying the exact unrecoverable items.

**Testing.**
- **Round-trip per kind** (extend `backup/roundtrip_tests.rs`): read → apply → verify → restore → verify
  against the real registry / service / task on CI.
- **Core invariant as a property test:** apply(any option) then restore the just-captured snapshot ⇒
  machine back at the pre-apply state.
- **Engine tests with a mock `EffectKind`** (in-memory state): exercise ordering, atomic rollback, and
  Needs-Attention assembly with zero OS contact — possible only because kinds sit behind the trait.
- **Validator regression tests:** feed known-bad YAML (all-undetectable, identical options, conflicting
  owners) and assert the build rejects them.

## 12. Migration plan

- **Engine + schema first.** Build `tweaks/model.rs`, the engine, the kind modules, and the new
  `build.rs` validation. Old `services/*_service.rs` primitives are retained and wrapped; old
  `backup/*` modules are removed as their logic is absorbed.
- **Starter set.** Convert a small, representative set of tweaks (one per effect kind + one action tweak)
  to the new schema to prove the engine end-to-end.
- **Corpus later.** The remaining ~189 tweaks migrate in follow-up work, fixing known content problems as
  each tweak is touched (all-skip_validation/undetectable tweaks, options that erase each other,
  conflicting stock assumptions, undisclosed insecure options).
- **Clean break.** No dual-schema adapter. Old on-disk user snapshots are invalidated by a
  schema-version bump; they are transient runtime state, so users simply re-detect.

## 13. Failure classes this design eliminates

Grouped by the mechanism that prevents them — first-principles, not a prior finding list:

- **Structural (impossible to express or reach):** one typed representation means capture/apply/detect/
  restore cannot drift; rollback and Restore share one drive-to-value path, so no separate restore branch
  can silently "succeed"; a read error is `Err`, never a fake absent; key create/delete are Actions, never
  mis-run as value ops; no stock/default value is authored, so a wrong default cannot exist; the per-tweak
  lock removes the apply race; each snapshot is written **create-new**, so a capture can't overwrite an
  existing return-point in the history.
- **Build-time (rejected before shipping):** every option has a detectable effect per supported OS
  version; no two options are byte-identical; every option covers the managed surface; no two tweaks
  fight over the same address; declared `reversible` / `risk_level` match reality; typed values validate
  with the runtime parser.
- **Finished / absorbed:** atomic snapshot writes (both save and metadata) with schema/machine stamping
  and serde defaults; the typed elevation broker's grouped execution replaces per-service duplication;
  batch counts are per-tweak.
- **Docs realigned as the schema changes** (`docs/TWEAK_SYSTEM.md`, `docs/TWEAK_AUTHORING.md`): rollback
  scope, `requires_reboot` optionality, snapshot location, and the new `elevation` declaration (§9).
- **Hosts & firewall kept as first-class typed kinds** (not demoted to scripts): modelling them as
  Settings preserves their reversibility and detectability through the type system, wrapping the existing
  `hosts_service` / `firewall_service` (see §5).

## 14. Decisions deferred to the implementation plan

Each has a recommended default so it is not blocking:
- **Recommended default level per resource (advisory only).** §9 makes the level author-declared — a
  per-Tweak **floor** plus an optional per-step `elevation:` that escalates a single step (`max(floor,
  step)`) — **trusted and never inferred** (ADR-0005). Authoring *guidance* (a suggested starting level per
  kind/resource) may ship to help authors, but it never sets the stored level and never feeds a silent
  escalation.
- **Action working directory / timeout / output capture.** Default: no working-dir assumption, a bounded
  timeout, exit-code-only interpretation (stdout captured only for logging).
- **Task "enabled" granularity.** Default: model as enabled/disabled boolean; richer task definition
  (triggers/actions) is out of scope for the first engine.
- **Supported-build matrix.** Build-number scoping (§6) checks applicability against a declared set of
  supported build milestones (e.g. Win10 22H2 `19045`; Win11 22H2 `22621`, 23H2 `22631`, 24H2 `26100`). The
  exact representation, and how the detectability guard iterates it, are finalized in the plan; default: an
  explicit milestone list the validator loops over.
- **`requires_reboot` granularity.** Default: a per-tweak flag; per-option reboot needs are deferred.
- **Profile/import layer.** Out of scope for this engine; it drives the same apply/restore entry points and
  identifies options by label within a tweak.

## 15. Normative invariants

The non-negotiable rules the implementation and its tests must uphold. Each maps to the section that
details it; these are the traceable acceptance criteria.

1. **One representation.** A change's desired value, captured pre-apply value, current reading, and detection
   comparison use one typed `Value` and one comparison per kind (§5).
2. **Did-it-work.** Every effect op returns `Result`; a failed apply/read/revert surfaces as `Err`, never a
   benign value; no `let _ =` on a privileged call (§8/§11).
3. **Each apply captures the pre-apply state** as a return-point in the snapshot history, before any
   mutation — authored-option captures dedup (one per option), System-Default/non-defined captures are all
   kept (§8/§11).
4. **The pre-apply state is captured before any mutation;** an unreadable value aborts before mutating (§8).
5. **Options are mutually exclusive** — a tweak is in exactly one authored option *or* System Default at
   any time (§6/§8).
6. **A matching option always wins** over System Default in detection; at most one option can match; no
   match ⇒ System Default (§8/§10).
7. **System Default is a computed *status*, never authored and never a restore target** — it means "matches
   no defined option." The only restore action is **Restore Snapshot** (most recent) (§6/§8, ADR-0003).
8. **Reversibility is typed and computed** — `reversible` = all effects are Settings, Actions with `undo`,
   or `ephemeral` Actions; one **non-ephemeral** no-`undo` action makes the tweak one-way, surfaced
   *before* apply; revert still reverts everything else (§7).
9. **Detectability is typed** — Settings always detectable, Actions detectable iff they declare a `probe`;
   there is no `skip_validation` flag (§6/§10).
10. **Effects apply in declaration order;** elevation batching is order-preserving (§8/§9).
11. **`probe` and read-back verification are state-based,** used identically at apply-time (did-it-work) and
    detect-time (§7/§8).
12. **A snapshot is deleted only on a verified match** — a fully-verified restore, or the startup
    stale-cleanup where the live system matches the held state (checkable Settings only), or an explicit
    `keep_current_state` — never on uncertainty (§8/§11, ADR-0002).
13. **Rollback restores the just-captured pre-apply snapshot** via the same drive-to-value path as Restore;
    a verified full restore consumes that snapshot, an incomplete one is Needs Attention with it kept, and
    the error carries both the original and rollback failures (§8, ADR-0001).
14. **Snapshots are portable and machine-bound** — stored next to the exe; a corrupt / wrong-schema /
    wrong-machine snapshot is "no valid snapshot", never a valid prior (§11).
15. **Inapplicable effects** (OS-version- or build-scoped-out) are excluded from apply, detect, and the
    detectability guard for that version/build (§6/§8/§10).
16. **Batch operations are per-tweak-independent** and report per-tweak counts (§8).
17. **Elevation is declared, never inferred** — a per-Tweak **floor** (`User / Admin / System / TI`); a
    step's effective level is `max(floor, step)`, escalate-only. `User` / `Admin` run in-process (a
    user-hive/HKCU effect *always* runs in-process as the real user, ignoring the floor); `System` / `TI`
    run in a fresh child. The engine never silently escalates, and "couldn't acquire the level" is a
    distinct failure from "acquired but access-denied" (§9, ADR-0005).
