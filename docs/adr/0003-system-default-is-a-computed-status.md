---
status: accepted
---

# System Default is a computed status, not a restore target; restore walks the snapshot history

A machine's setting is frequently not at its stock default before we touch it — a prior tool, a group policy, or the user got there first. In the redesigned engine, **System Default is a computed *status***: it means the live surface **matches no authored option** (the author never defined this state, or the machine drifted out of every defined one). It is **not** a selectable or restorable target — there is no "revert to System Default" action.

The **only** restore action is **Restore Snapshot**, which drives the system to the **most recent** entry in the tweak's snapshot history and, on verified success, **consumes** it (the system now matches it, so the return-point is spent). Pressing Restore repeatedly walks back through the captured history; when the last entry is popped the surface matches no option again — i.e. it simply *reads* as System Default, reached by exhausting the history rather than by a dedicated button. A future *select-a-snapshot* control may restore a specific entry; the default is always most-recent.

## Considered Options

- **Offer System Default as a selectable state that performs a revert** (the previous system's behaviour) — rejected: it made System Default behave like an authored option, gave snapshots no natural end of life, and forced a single "the original" target when the redesign keeps a full history of return-points.

- **A single snapshot ("the original"); restore always returns to it** — rejected: it discards the intermediate states the user passed through. The history preserves every state the user actually had and lets a power user step back one at a time (or delete the most-recent to skip it).

## Consequences

Detection is decoupled from the snapshot count: status is purely live-surface-vs-options, so leftover return-points never turn a clean System-Default machine into Needs Attention. `is_applied` means only "there is a history to restore from." A snapshot is released the moment the live system matches the state it holds — via a verified restore or the startup stale-cleanup (which compares **checkable Settings** only, never scripts) — so the history self-prunes without ever dropping a state the user is not currently standing on.

## Amended 2026-07-22 (tweak-system redesign, spec rev 2)

- **What "drives the system to the entry" means is defined by ADR-0007:** an entry holding an *authored option* is a **reference** — restore re-applies that option as *currently defined* (its Settings, its actions, its ephemerals); only *unauthored* states (System Default, drift) are value dumps that are driven back verbatim. Restore also runs the entry's recorded undo scripts (reverse order) before re-applying the target.
- **Statuses beyond System Default:** detection may also report **Unknown** (the surface could not be read — access denied, malformed packed value, missing non-optional resource; never a guess) and per-option **unavailable-on-this-machine** (an option needing a resource this machine lacks). Neither is authored, neither is a restore target.
- **Shape rule under computed Default:** 1 authored option renders as a toggle (Default ↔ On), ≥2 as a dropdown — the Default position is always this ADR's computed status.
