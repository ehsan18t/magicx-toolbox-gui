# Tweak System — Technical Architecture

> Architecture reference for the redesigned tweak engine.
> For the YAML authoring guide, see [TWEAK_AUTHORING.md](./TWEAK_AUTHORING.md).
> For full design rationale, see the spec at
> [`docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md`](./superpowers/specs/2026-07-21-tweak-system-redesign-design.md)
> and the decision records under [`docs/adr/`](./adr/). This document is the map; the spec is the
> territory.

---

## Overview

The engine applies curated Windows tweaks from embedded YAML and keeps per-tweak snapshots so changes
can be reverted. Its defining property is **one representation per change**: apply, capture, detect, and
revert all consume the *same* typed value, so they cannot drift (spec §5, invariant 1). Correctness is
enforced by types and the build-time validator, not by convention.

### One-representation model

The old system split one change into four representations — a YAML change, a captured snapshot, a
restore op, and a detection comparison — spread across separate services that drifted. The redesign
collapses them. An `Effect` is one atomic unit of change; a `Value` is the one domain shared by capture,
apply, detect, and restore (`src-tauri/src/tweaks/model.rs`):

```
Effect  = Setting(Setting) | Shared(SharedId) | Action(ActionDef)
Setting = Registry | RegistryKey | Service | Task | Hosts | Firewall
Value   = Absent | Missing | Reg(TypedRegValue) | Startup | TaskEnabled(bool) | Present(bool)
```

- **Reversibility and detectability are typed**, not asserted: Settings are always reversible and
  detectable; Actions are reversible iff they carry `undo`, detectable iff they carry `probe`; ephemeral
  Actions are exempt from both (spec §6.4/§7).
- **The did-it-work contract is unavoidable:** every effect op returns `Result`; a failed apply/read/revert
  is `Err`, never a benign value. An unreadable state is **Unknown**, never a guess (spec §8.4, invariant 3).

---

## Layer diagram

```
YAML corpus + shared block
        │  build.rs: schema::load_corpus → validate_structural → validate_semantic (per support milestone)
        ▼
Compiled Tweak model  (surface: Vec<EffectDef>, options = flat value-maps)  — embedded as JSON
        │
        ▼
Engine (src-tauri/src/tweaks/engine/)
   lifecycle: apply · detect · restore · verify · atomic rollback
   per-tweak lock · WAL action journal · shared-claims record
        │  dispatches each Effect to →
        ▼
EffectKind modules (src-tauri/src/tweaks/kinds/)
   registry · service · task · hosts · firewall · action
   (read + apply + revert + detect co-located per kind)
        │  execute through →
        ▼
Elevation broker (src-tauri/src/services/elevation/)
   user (in-process) · admin (in-process) · system · ti (short-lived children),
   grouped multi-op execution for same-level System/TI steps
```

- The **build script** is the gatekeeper: `build.rs` `#[path]`-includes the runtime's own
  `model`/`parse`/`schema`/`validate` modules, so build-time and runtime validation are the *same code* —
  drift is a compile error. It loads every `*.yaml` in `tweaks/`, runs the structural and semantic guards
  (spec §10) over each milestone of the support matrix (`19045`, `22621`, `22631`, `26100`), and embeds
  the validated corpus as JSON.
- Each **EffectKind module** owns how one kind reads/applies/reverts a value. The trusted low-level
  primitives (`registry_service` `RegSetValueExW`, `service_control` SCM, scheduler COM, `hosts_service`,
  `firewall_service`) are reused and hardened as adopted (e.g. the `delete_key` guards against
  lone/leading/trailing-backslash parent deletion).
- The **broker** owns privilege and is reused as-is; its wire protocol already carries `Vec<BrokerOp>`,
  so consecutive same-level System/TI steps batch into one child (order-preserving) without adding UAC
  prompts.

---

## Lifecycle

A tweak is in exactly one state at a time; options are mutually exclusive (spec §8).

**Apply(option)** — acquire the per-tweak lock and detect current status (applying the active option is
a verified no-op). Capture the pre-apply `Value` of every applicable non-shared Setting; a read that
cannot read is `Err` and aborts *before touching anything*. Persist the snapshot entry atomically before
mutating, including the **WAL action journal** (the target's intended action list, unmarked). Drive each
effect to its desired value in declaration order through its kind module and the broker; verify each by
read-back (Settings) or `probe`/exit-code (Actions). Each action's completion is fsynced into the journal
after it runs — a crash between run and mark surfaces as **Needs Attention**, never a silent skip.

**Atomic rollback (ADR-0001).** Any failure restores the just-captured entry via the same path as a user
Restore — undo the journal's completed actions in reverse, then drive the captured state back. The
returned error carries both the original failure and any rollback failures. A verified full restore
consumes the entry; a rollback that cannot fully complete keeps it and surfaces **Needs Attention**
(ADR-0002). "Atomic" means *attempted atomically, with failure surfaced* — not a guaranteed all-or-nothing.

**Detect** — read each applicable, detectable, non-shared Setting once; `optional` effects map `Missing`
through `if_missing`; probeable Actions contribute their session-cached present/absent; claimed shared
settings count as matching while any claim holds. A matching option wins; at most one can match
(distinctness guard); no match ⇒ **System Default** (a computed status, never authored — ADR-0003). A
read that fails ⇒ **Unknown**, with a needs-elevation hint when that is the cause. Options needing an
unsatisfiable value on this machine are flagged **unavailable**. A full scan runs in the background at
launch; statuses stream in; an Elevate triggers a full re-scan. There is no drift-refresh in v1.

**Restore Snapshot** — the only restore action (ADR-0003). Consume the head entry: run its journal's
undos in reverse, then re-apply the target. An **option reference** is re-applied *as currently defined*
(its Settings, actions, ephemerals — ADR-0007); a **value dump** is driven back verbatim. Verify;
success consumes the entry, the next becomes head, and exhausting the history simply reads as System
Default. Failure keeps the entry; an incomplete restore ⇒ Needs Attention.

---

## Snapshots & shared claims

Snapshots live in the portable `snapshots/` directory **next to the executable**
(`SnapshotStore::open_default` → `current_exe().parent()/snapshots`; spec §11). Storage is per-tweak: one
subdirectory per tweak-id, one atomically-written file per entry, keyed by a **monotonic per-tweak
sequence** (wall-clock timestamps are display metadata only). Each entry is stamped with a schema version
and the machine's `MachineGuid`.

- **Authored-option captures store a reference** (`OptionRef(label)`), re-derived from the current corpus
  on restore; **unauthored states** (System Default, drift) store a full value dump. Both carry the WAL
  journal. Shared-referenced effects appear in **neither** — their return path is the claims record
  (ADR-0006/0007).
- **Dedup moves to head:** at most one entry per authored option (re-capture vacates the old position);
  unauthored captures are all kept (spec §8.2).
- An entry that is corrupt, wrong-schema, wrong-machine, or **dangling** (its option/tweak no longer
  exists, or the target is unavailable here) is **invalid**: kept on disk, excluded from the walk, and
  released only by explicit user consent (`discard_snapshot_entry`) — never guessed at (ADR-0002).
- **Shared claims** live in one engine-level `shared_claims.json` under the snapshots root:
  `{ shared_id → { original, claimants } }`. First claim captures the live original once and drives the
  value; further claims are verified no-ops; the last release restores the captured original,
  unconditionally and verified (external drift is overwritten by the return). Detection counts a claimed
  setting as matching for every claimant while any claim holds (spec §8.6).

---

## Elevation & execution context

See ADR-0005. Four author-declared levels (`user`/`admin`/`system`/`ti`), a per-tweak floor with
per-effect escalate-only refinement (`effective = max(floor, step)`). `user`/`admin` run in-process;
`system` duplicates winlogon's token in a fresh child; `ti` starts the TrustedInstaller service and
parent-spoofs off it. A **user-hive (HKCU) effect always runs in-process as the interactive user**,
ignoring the floor. At startup an **over-the-shoulder guard** compares the process-token SID with the
interactive session SID; on mismatch (a different admin's credentials elevated the app), User-level
tweaks are disabled to avoid writing the wrong hive. Reads run at whatever level the app currently has;
TI-protected resources legitimately deny reads and report **Unknown** until the user elevates. The app
never silently escalates.

---

## Safety model (the ADRs)

| ADR | Decision |
|---|---|
| [0001](./adr/0001-rollback-failure-is-a-first-class-state.md) | Rollback failure is a first-class, retryable **Needs Attention** state; rollback never aborts early. |
| [0002](./adr/0002-snapshot-deletion-requires-verification-or-consent.md) | A snapshot is deleted only by a verified restore, the verified startup stale-cleanup, or explicit consent — never on a failure path or uncertainty. |
| [0003](./adr/0003-system-default-is-a-computed-status.md) | System Default is a computed **status**, not a restore target; Restore Snapshot walks the history. |
| [0004](./adr/0004-value-null-is-not-a-delete-spelling.md) | `absent` is the only absence spelling; a forgotten/`null`/omitted value is a build error, never a silent delete. |
| [0005](./adr/0005-elevation-is-per-tweak-and-never-silently-escalated.md) | Elevation is declared per tweak (refinable per effect), escalate-only, never inferred or silently escalated. |
| [0006](./adr/0006-one-address-one-owner-shared-state-is-declared-and-refcounted.md) | One address, one owner corpus-wide; genuine sharing is a declared, refcounted `shared` claim. |
| [0007](./adr/0007-option-snapshots-are-references-restore-reapplies-the-current-definition.md) | Option snapshots are references; Restore re-applies the current corpus definition (updates heal restores). |

---

## Module map

```
src-tauri/src/tweaks/
  model.rs          Effect · Setting · ActionDef · Value · Tweak · Opt  (the one representation)
  schema.rs         YAML DTOs → compiled model (build-time; deny_unknown_fields)
  parse.rs          typed literals, registry-path grammar, build-expr grammar, kv_semicolon parser
  validate.rs       structural + semantic guards (spec §10), quantified per support milestone
  engine/           apply · detect · revert · lifecycle (per-tweak lock, verify, Needs Attention)
  kinds/            registry · service · task · hosts · firewall · action  (read+apply+revert+detect)
  snapshot.rs       atomic per-tweak history, seq ordering, refs vs dumps, invalid-entry handling
  shared_claims.rs  claims record: capture-once, refcount, last-release restore
  winver.rs         RtlGetVersion + UBR
src-tauri/src/commands/tweaks.rs   Tauri command surface (below)
src-tauri/build.rs                 load + validate + compile the corpus at build time
```

**Command surface** (`commands/tweaks.rs`): `get_tweaks`, `get_statuses_stream` (background scan,
streamed), `rescan_after_elevation`, `apply_tweak`, `restore_tweak`, `list_snapshot_entries`,
`discard_snapshot_entry`, `get_elevation_state`. The `*View` types translate engine results into the
frontend model — per-tweak state (Active option / System Default / Unknown / Unavailable), per-option
unavailable reasons, held-by info, and apply/restore outcomes with per-effect results.

---

## Migration status

The redesign was a hard cut: the old effect/apply/backup pipeline and every old YAML file were deleted in
the same effort (spec §12). The engine ships with an **example corpus** — `src-tauri/tweaks/examples.yaml`,
one tweak per feature — that proves the build and engine end-to-end. The real tweak corpus is re-authored
from scratch, per category, on `main`, outside this plan. There is no dual-schema layer and no snapshot
migration: old on-disk snapshots are invalidated by the schema-version bump.
