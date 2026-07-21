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
