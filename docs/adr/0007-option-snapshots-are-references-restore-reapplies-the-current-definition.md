---
status: accepted
---

# Option snapshots are references; Restore re-applies the current definition

A pre-apply capture whose state **is an authored option** is stored as a **reference**
(`captured_option: <label>`), not a value dump. Restoring it means **re-applying that option exactly
as the *current* corpus defines it** — drive its Settings, run its actions (ephemerals included),
verify its probes — after first running the entry's recorded **undo** scripts in reverse order (the
actions that ran when the user left that state). Only **unauthored** states — System Default and
drifted states, which exist nowhere but on that machine — are captured as full value dumps and driven
back verbatim (their scripts cannot be re-run; the UI carries the standing reboot-may-be-needed
advisory).

The app is the source of truth for authored states. A corpus update that changes an option therefore
*intentionally heals* what old references restore to — restore produces the option as it is defined
today, coherently, never a half-old fabricated state.

**The bookkeeping that makes this safe:**

- **WAL action journal.** The snapshot entry is persisted *before* mutation and includes the target
  option's intended action list; each action's completion is durably marked after it runs. A crash
  between run and mark surfaces as Needs Attention — an action can never run unrecorded.
- **Dedup moves to head.** At most one entry per authored option; re-capturing an option-state gives
  the entry a fresh head position under a **monotonic per-tweak sequence** (wall-clock is display
  metadata only). Restore therefore always walks the most recent journey; unauthored captures are
  never deduped.
- **Dangling references are invalid, not guessed.** If the referenced option or tweak no longer
  exists, or the target is unavailable on this machine/build, the entry is excluded from the walk,
  surfaced, and released only by explicit consent (ADR-0002 amendment).
- **Shared-referenced effects appear in no per-tweak snapshot** — their return path is exclusively
  the claims record (ADR-0006), so two tweaks' snapshots can never fight over one address.

## Considered Options

- **Always store captured values verbatim; restore drives bytes back** — rejected: on a corpus
  update it restores stale values the current definition no longer means (or worse, a mix), it turns
  option renames into silent state corruption instead of a detectable dangling reference, and it
  duplicates authored data that has exactly one legitimate source.
- **References with a stored value-hash that invalidates on corpus change** — rejected: strictly
  worse than re-applying (the entry dies on every corpus touch instead of healing) while keeping all
  the reference bookkeeping.

## Consequences

- Restore of an option reference is a *full re-apply*, so probed actions verify meaningfully and
  ephemeral activators run — a restored state is a working state, not just written bytes.
- **Accepted edge:** if an option's surface *grew* since an older dump was captured, walking all the
  way back cannot return the new effect to a pre-history value that was never captured. Documented
  as a known, rare, bounded limitation.
- Undo scripts belong to the entry that was captured when they ran: entry E records the actions of
  the apply that created E, and restoring E runs exactly those undos before re-applying E's state.
