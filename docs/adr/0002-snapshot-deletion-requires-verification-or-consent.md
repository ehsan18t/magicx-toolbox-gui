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
