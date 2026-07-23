# Non-Issues

Things that were found, verified against the code, and **deliberately not fixed**.

This is the won't-fix register. An entry here is a decision, not a backlog item: it has been looked
at, it is understood, and the call was to leave it. Do not re-raise an entry in review, and do not
"helpfully" fix one without saying so first. If an entry should move, it moves as its own task and
gets struck from this file.

Each entry records what the problem actually is, whether it can bite today, and what the fix would
be if it is ever scheduled, so nobody has to re-derive it.

**Not the same as** `docs/ROADMAP.md` (planned work) or `PRE_MERGE_TASKS.md` (things blocking a
merge). Those are queues. This is not.

| #   | Issue                                            | Can it bite today?           | Decided    |
| --- | ------------------------------------------------ | ---------------------------- | ---------- |
| 1   | Snapshots carry no user identity                 | yes, on a multi-user machine | 2026-07-26 |
| 2   | Toggling a System Default tweak off cannot clear the staged change | yes        | 2026-07-27 |

---

## 1. Snapshots carry no user identity

**Decided:** 2026-07-26. Acknowledged, deliberately not fixed now. The remediation is a snapshot
schema change, which is out of scope for the current branch.

Found by the safety pass on the HKCU elevation-gate fix
(`docs/plans/fix-hkcu-user-level-gate.md`).

**The mismatch.** HKCU is keyed to the *account*. The snapshot store is keyed to the *machine* and
lives in a portable `<exe>/snapshots/` directory that every account on the box shares
(`snapshot.rs:189-193`). `Entry` carries `schema_version`, `machine_guid`, `tweak_id`, `seq`,
`timestamp`, `captured`, `journal` (`snapshot.rs:78-92`) and no user field. `head()` selects on
`tweak_id` + corpus + `machine_guid` + build (`snapshot.rs:268-285`), and the only identity-based
`InvalidReason` is `WrongMachine` (`snapshot.rs:118`). There is no `WrongUser`.

**Reachable today, no elevation involved.** User A applies an HKCU-touching tweak, pushing `seq 1`.
A logs off, B logs in and applies the same tweak under their own account, pushing `seq 2` into the
same directory. A logs back in and reverts: `head()` returns `seq 2` (sorted `Reverse(seq)`,
`snapshot.rs:277`), so **B's captured baseline is driven into A's hive** (`apply.rs:993`, `:1015`),
the read-back verifies because it just wrote those values, and the entry is consumed
(`revert.rs:197-201`). A's own return point is never consulted. If both captures are
`Captured::OptionRef` under the same label, B's push instead dedups A's entry away outright
(`snapshot.rs:199-210`).

**Fix if scheduled:** stamp `Entry` with the capturing token's SID and add
`InvalidReason::WrongUser`, so `classify_and_parse` (`snapshot.rs:384-410`) marks a foreign-user
entry `Invalid` for any HKCU-touching tweak. Per ADR-0002 it stays on disk with a discard affordance,
never a silent delete. Costs a `schema_version` bump and a migration decision for snapshots already
on disk.

---

## 2. Toggling a System Default tweak off cannot clear the staged change

**Decided:** 2026-07-27. Whether "off" means *cancel the staged change* or *apply the stock-default
option* is an authoring/design call, not a correctness one, so the current behavior stands.

**What you see.** Take a tweak that has never been applied, so its status is **System Default**
(ADR-0003: the live state matches none of the author's declared options). Switch it **on**: a change
stages, correctly. Switch it **off**: instead of cancelling that staged change, it stages a different
one, an apply of the stock-default option. The switch renders unchecked while the **Pending** badge
stays lit, and the pending counter never returns to zero. Committing the batch then writes the
stock-default values and creates a snapshot for a tweak the user never wanted touched.

**Why.** `stageApply` cancels only when the clicked label equals the currently active option
(`TweakCard.svelte:140-142`), but `mapStatusView` sets `activeOption` to `null` for every state
except `active` (`tweaksData.svelte.ts:97`), and System Default is one of those. So the comparison is
`offLabel === null`, never true, and the cancel path never runs. `handleSwitchChange` routes both
directions through `stageApply` (`TweakCard.svelte:158-161`); the earlier code sent "off" through
`goSystemDefault`, which unstaged first.

Note that `activeOption == null` for System Default is **correct** per ADR-0003, since there is no
active option to name. The gap is only in the cancel check.

**Fix if scheduled:** have the cancel path also fire when the tweak has no active option and the
clicked label is the stock default, i.e. when staging would be a no-op against the live state. Decide
first whether "off" from System Default should mean cancel or a real apply; the fix follows from
that answer, not the other way round.
