---
status: accepted
---

# Rollback failure is a first-class, retryable state

Applying a tweak is atomic in intent — if any Effect Phase fails, we roll the whole thing back from the Snapshot. But Rollback is itself a pile of Windows operations that can fail (a locked service, a protected task, access denied), so "all changes are rolled back" is a promise the system cannot unconditionally keep. Rather than pretend otherwise, a Tweak whose Rollback did not fully succeed enters **Needs Attention**: it keeps its Snapshot, names the resources that could not be restored, and offers a retry.

Rollback also never aborts early. Previously a registry restore failure abandoned the service, scheduler, hosts and firewall phases entirely; now all five always attempt and collect their failures.

## Considered Options

- **Best-effort and quiet** — attempt everything, report only "apply failed". Rejected: the machine is left half-modified and the user is never told.
- **Lock the control until resolved** — rejected because some restores genuinely cannot succeed on a given machine (a TrustedInstaller-owned service where TI elevation is failing), which would brick the tweak in the UI permanently.
- **Auto-retry at next startup, escalating elevation** — rejected as a default. Silently re-attempting registry and service writes while the user isn't watching produces failures nobody can reconstruct afterwards. Viable later as an opt-in.

## Consequences

"Atomic" in our docs now means *attempted atomically, with failure surfaced* — not *guaranteed all-or-nothing*. The authoring guide must say so plainly, because a tweak author reading "all changes are rolled back" will otherwise assume a guarantee we don't provide.
