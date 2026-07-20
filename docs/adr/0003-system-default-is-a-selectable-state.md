---
status: accepted
---

# System Default is a selectable state, and selecting it is a Revert

A machine's setting is frequently not at its Stock Default before we touch it — a prior tool, a group policy, or the user got there first. That Original State is a legitimate destination, so **System Default is offered as a selectable state whenever a Snapshot exists**, in both the two-option control and the dropdown, and choosing it performs a Revert: restore the Original State, then release the Snapshot on verified success.

Previously the affordance was rendered but inert. The two-option control showed a clickable "Default" segment whose handler only unstaged a pending change and never restored anything, while the dropdown showed a permanently disabled "System Default" entry that also vanished as soon as any option was applied. The two controls disagreed with each other and neither one worked.

## Considered Options

- **Restore but keep the Snapshot** (a distinct operation from Revert) — so that cycling System Default ↔ an Option never re-captures, and the first good capture is kept forever. Genuinely attractive, because re-capture is not currently reliable: an access-denied registry read is recorded as "this value did not exist", which would trade a good Snapshot for a corrupt one.

  Rejected because it makes System Default behave unlike every other state and leaves Snapshots with no natural end of life. The re-capture hazard is a capture bug and is being fixed on its own merits; we would rather fix it than design around it.

- **Keep the Snapshot only when the Original State matched no Option** — rejected as a special case that is hard to explain and hard to test.

## Consequences

Once at System Default there is nothing left to restore, so releasing the Snapshot loses nothing — the next Apply captures afresh. This depends on capture being correct, which makes the access-denied capture bug a blocker for this change rather than an independent cleanup.
