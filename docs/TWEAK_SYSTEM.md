# Tweak System: Technical Architecture

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

The old system split one change into four representations (a YAML change, a captured snapshot, a
restore op, and a detection comparison) spread across separate services that drifted. The redesign
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
Compiled Tweak model  (surface: Vec<EffectDef>, options = flat value-maps), embedded as JSON
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
   user (in-process) · admin (in-process) · ti (short-lived child),
   grouped multi-op execution for consecutive TI steps
```

- The **build script** is the gatekeeper: `build.rs` `#[path]`-includes the runtime's own
  `model`/`parse`/`schema`/`validate` modules, so build-time and runtime validation are the *same code*;
  drift is a compile error. It loads every `*.yaml` in `tweaks/`, runs the structural and semantic guards
  (spec §10) over each milestone of the support matrix (`19045`, `22621`, `22631`, `26100`), and embeds
  the validated corpus as JSON.
- Each **EffectKind module** owns how one kind reads/applies/reverts a value. The trusted low-level
  primitives (`registry_service` `RegSetValueExW`, `service_control` SCM, scheduler COM, `hosts_service`,
  `firewall_service`) are reused and hardened as adopted (e.g. the `delete_key` guards against
  lone/leading/trailing-backslash parent deletion).
- The **broker** owns privilege; its wire protocol carries `Vec<BrokerOp>`, so consecutive `ti` steps batch into one child (order-preserving) without adding UAC prompts.

---

## Lifecycle

A tweak is in exactly one state at a time; options are mutually exclusive (spec §8).

**Apply(option):** acquire the per-tweak lock and detect current status (applying the active option is
a verified no-op). Capture the pre-apply `Value` of every applicable non-shared Setting; a read that
cannot read is `Err` and aborts *before touching anything*. Persist the snapshot entry atomically before
mutating, including the **WAL action journal** (the target's intended action list, unmarked). Drive each
effect to its desired value in declaration order through its kind module and the broker; verify each by
read-back (Settings) or `probe`/exit-code (Actions). Each action's completion is fsynced into the journal
after it runs, and a row left planned but never confirmed complete surfaces as **Needs Attention**, never a silent skip.
The journal proves the action was planned, never that it ran; the scan covers a tweak's whole history rather than only its newest entry, and a row carries its own resolution mark, written by the same path that marks a row completed, so a verified apply or restore takes the rows it drove and verified out of the scan while leaving every row it never accounted for in it.

**Atomic rollback (ADR-0001).** Any failure restores the just-captured entry via the same path as a user
Restore: undo the journal's completed actions in reverse, then drive the captured state back. The
returned error carries both the original failure and any rollback failures.

An action whose script started and then failed (non-zero exit, timeout, failed wait) may have partly run, and a probe cannot see partial progress, so the rollback always reverses it like one that ran, even on an untouched machine: its `undo` runs (or, for a drive-back, its `apply` re-runs), verified by its probe when it has one, and only a verified reversal lets its journal row resolve. An action with no `undo` that failed partway leaves the rollback incomplete, reported as a `no_undo` item. Only an action refused before its script was spawned (`ActionNotStarted`, an unrouted level, an elevation never acquired) or never reached counts as not having run. Restore has no such gap: any undo, re-run or re-apply that fails leaves the restore unverified, so its entry is kept and Needs Attention is recorded. A verified full restore
consumes the entry; a rollback that cannot fully complete, or an elevated step whose outcome is unknown, keeps it and surfaces **Needs Attention** (ADR-0002, ADR-0005). "Atomic" means *attempted atomically, with failure surfaced*, not a guaranteed all-or-nothing.

**Needs Attention is a per-tweak record**, `snapshots/<tweak-id>/_attention.json`, written atomically and stamped with the schema version and machine guid exactly like an entry. It is deliberately not a field on an entry: dedup, a later verified rollback's `consume`, and an entry turning invalid all delete or disqualify entries, and the mark has to outlive every one of them. It is **set** by an apply that fails and keeps its snapshot, by a restore that does not fully verify, and by the startup crash-residue scan; it is **cleared** only by a fully verified apply or restore of that tweak, or by the user's own decision to keep the current state. That decision is a single backend operation which releases the record whether or not any entry is left and discards the ones that are, so a record can never outlive the last entry with no way to release it (ADR-0002). Unresolved state is durable in three more places the startup crash scan reads. An apply or restore sets `drive_open` on the entry it drives from before its first change; it covers Settings and Shared blocks only. A verified apply or restore settles it on every entry, as it clears the record, because it re-established the whole surface; a verified rollback settles only its own, and a recorded failure only the marks it added, never one an earlier crash left. Each action a rollback or restore undoes or re-runs is held in the entry's `actions_in_flight` until it finishes and verifies, and is otherwise resolved only by an operation that drives and verifies that action (in either direction) or probes it, exactly like a journal row, so settling the Settings never hides a half-undone script. When a verified outcome's `drive_open` cannot be rewritten (the entry file held open elsewhere), `settle_verified` tries to record `outcome_unrecorded` at once, with an `unrecorded` item the scan reads as the explanation for that mark; if even that write fails, the returned status carries the attention and the next launch may report the change as unfinished. The scan adds its items to an existing record of this build's rather than skipping it, and never writes over a foreign or unreadable one. `settle_verified` is the one sink for a verified outcome's bookkeeping: it resolves what the outcome drove or probed absent, settles drive marks, and clears the record only when the scan finds nothing left, otherwise recording what remains; any store failure on the way is recorded as `outcome_unrecorded`. `consume` refuses to delete an entry that still holds any mark the operation's settle left (an open drive, an unfinished step, or a row it never accounted for), so the entry survives as that mark's evidence; a verified rollback first settles its own entry's drive and the rows whose action was refused before it started, was never reached, or failed partway and was then reversed and verified. An action that ran but could not be marked complete is reversed by the rollback like any action that ran, and its row stays outstanding, so the entry is kept and the scan raises it. The last place is the journal rows a crash left planned and unconfirmed, so a verified apply or restore also **resolves those rows before it clears the record**, per row and in the entry that holds the row, through the same atomic rewrite that marks a row completed. It resolves exactly the rows whose action it drove and verified, so a restore of a captured value dump never retires an action it neither probed nor undid, and the user's own decision needs nothing extra because it discards every entry and takes their rows with them. Ordering the row marks before the record is what makes a crash between them safe: the worst case is a stale record the next clear releases, never a resolved tweak the next scan marks again. Nothing else clears the record, discarding the last entry by hand included, and a record that cannot be written is reported as itself rather than counted as an unrecovered resource. A record that cannot be *read* or *parsed* is surfaced as Needs Attention in its own right, since reporting it as "nothing to attend to" would hide a real mark behind a log line. A record stamped for another machine or another schema is never overwritten and never deleted, exactly like an entry across the same boundary; one that names no owner at all is this build's to replace and to release, or it would badge the tweak with nothing able to lift the badge. Each recorded item carries the effect id and a kind (drive, verify, outcome_unknown, action, no_undo, claim, store, crash_residue, other), so the UI can tell a retryable step from a one-way one, and, when the failure was classified, a class (access denied, not found, invalid data, busy, failed) saying why.

**Detect:** read each applicable, detectable, non-shared Setting once; `optional` effects map `Missing`
through `if_missing`; probeable Actions contribute their session-cached present/absent; claimed shared
settings count as matching while any claim holds. A matching option wins; at most one can match
(distinctness guard); no match ⇒ **System Default** (a computed status, never authored; ADR-0003). A
read that fails ⇒ **Unknown**, with a needs-elevation hint when that is the cause. Options needing an
unsatisfiable value on this machine are flagged **unavailable**. A full scan runs in the background at
launch; statuses stream in; an Elevate triggers a full re-scan. There is no drift-refresh in v1.

**Restore Snapshot:** the only restore action (ADR-0003). Consume the head entry: reverse its journal's
completed actions in reverse (undo what the apply ran, re-run what it drove back), then re-apply the target. An **option reference** is re-applied *as currently defined*
(its Settings, actions, ephemerals; ADR-0007); a **value dump** is driven back verbatim. Verify;
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
  journal. Shared-referenced effects appear in **neither**; their return path is the claims record
  (ADR-0006/0007).
- **Dedup moves to head:** at most one entry per authored option (re-capture vacates the old position);
  unauthored captures are all kept (spec §8.2).
- An entry that is corrupt, wrong-schema, wrong-machine, or **dangling** (its option/tweak no longer
  exists, or the target is unavailable here) is **invalid**: kept on disk, excluded from the walk, and
  released only by explicit user consent (`discard_snapshot_entry`), never guessed at (ADR-0002).
- **Shared claims** live in one engine-level file per machine, `shared_claims.<MachineGuid>.json`, under the snapshots root, so a portable folder used on two machines never overwrites either machine's originals. An unsuffixed `shared_claims.json` stamped for this machine is read until the first write folds it in; one stamped for another machine is never read or touched. The shape is `{ shared_id → { original, restore_level, claimants } }` (schema version 2). First claim captures the live original once and drives the value; further claims are verified no-ops; the last release restores the captured original, unconditionally and verified (external drift is overwritten by the return). `restore_level` is the highest level any claimant routed at, raised as claims arrive; the restore drives at the higher of it and the last releaser's current route (ADR-0007), so a Ti capture is never driven back at Admin. A last release that its apply then rolls back re-claims at the level it restored at, so the recorded level never drops. A version 1 file loads with `restore_level` absent: that restore falls back to the releaser's route (logged), and the next write stamps the file as version 2. A higher version, or an unreadable file, is Corrupt: never rewritten, and never read as "no claims" by any caller (detect reports Unknown, apply and restore fail). Detection counts a claimed setting as matching for every claimant while any claim holds (spec §8.6).

---

## Elevation & execution context

See ADR-0005. Three author-declared levels (`user`/`admin`/`ti`), a per-tweak floor with per-effect
escalate-only refinement (`effective = max(floor, step)`). `user`/`admin` run in-process; `ti` starts
the TrustedInstaller service and parent-spoofs off it. A run of consecutive same-level `ti` effects
shares ONE elevated child rather than one per effect. A **user-hive (HKCU) effect always runs in-process as the interactive user**,
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
| [0002](./adr/0002-snapshot-deletion-requires-verification-or-consent.md) | A snapshot is deleted only by a verified restore, a verified rollback of the entry just pushed, a dedup that supersedes a settled entry, or explicit consent; never on a failure path or uncertainty. No startup stale-cleanup is implemented. |
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
`discard_snapshot_entry`, `keep_current_state`, `get_tweak_status`, `get_elevation_state`. The `*View` types translate engine results into the
frontend model: per-tweak state (Active option / System Default / Unknown / Unavailable), per-option
unavailable reasons, held-by info, and apply/restore outcomes with per-effect results.

---

## Migration status

The redesign was a hard cut: the old effect/apply/backup pipeline and every old YAML file were deleted in
the same effort (spec §12). A demonstration corpus (`examples.yaml`, one tweak per feature) carried the
engine end-to-end until the real categories were re-authored from scratch; it has since been removed. The
shipping corpus is the nine category files in `src-tauri/tweaks/`. There is no dual-schema layer and no
snapshot migration: old on-disk snapshots are invalidated by the schema-version bump.
