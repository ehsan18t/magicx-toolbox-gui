---
status: accepted
---

# A Snapshot is deleted only by verified restore or explicit user decision

A Snapshot is the only record of what a machine looked like before we touched it; once it is gone the Original State is unrecoverable. So there are exactly two things that may delete one: a **restore that verifiably succeeded**, or a **deliberate user decision** to accept the current state and let it go. Nothing else — not a failure path, not an error, not a convenience cleanup.

This was violated in practice: a failed first apply discarded the rollback result and deleted the Snapshot unconditionally, so a partial rollback left the machine half-changed with no way back and no indication anything was wrong.

## Consequences

- Startup stale-snapshot cleanup stays, but only because it already satisfies the rule: it deletes only when every captured resource verifiably matches the Original State, and preserves the Snapshot whenever a resource cannot be checked. A cleanup that deleted on *uncertainty* would violate this ADR.
- Any code path that deletes a Snapshot must first inspect a restore result. Discarding that result — `let _ = restore(...)` — is a bug by definition, not a style choice.
- "Keep current state" exists in the UI specifically to give consent a place to live. Without it, users stuck in Needs Attention would have no legitimate way to release a Snapshot.
- Snapshots form a per-tweak **history** of return-points, not a single file; each is released independently under the same rule. A restore, and the startup stale-cleanup, match a held state against the live system using only **checkable Settings** — registry, service, task, hosts, firewall — never cmd/powershell Actions, which have no reliable state to diff (a System-Default capture holds no scripts anyway). Restoring the most-recent snapshot is the only restore action, and a verified match consumes it (see ADR-0003).

## Amended 2026-07-22 (tweak-system redesign, spec rev 2)

- **Invalid entries are released only by consent.** An entry that is corrupt, wrong-schema, wrong-machine, or **dangling** (it references an option or tweak the current corpus no longer defines, or a target unavailable on this machine/build) is treated as *no valid snapshot* for restore purposes — but it is **kept on disk, excluded from the walk, and surfaced in the UI with an explicit discard affordance**. Deleting it silently would be deletion on uncertainty, which this ADR forbids.
- **The shared-claims record follows the same rule.** The original value captured at the first claim of a shared setting (ADR-0006) is restored, verified, and only then released — on the *last* claim's release. An unverifiable restore keeps the record and surfaces Needs Attention, exactly like a per-tweak snapshot.
- **The checkable-Settings-only match above is scoped to the startup stale-cleanup.** A *restore* of an authored-option reference is now a full re-apply (ADR-0007), so its verification legitimately includes Action probes — state-based PowerShell checks — not Settings alone. The cleanup keeps the stricter Settings-only comparison because it deletes without a user action in flight.
