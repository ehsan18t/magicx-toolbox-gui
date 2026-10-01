---
status: accepted
---

# One build line per snapshots folder; downgrades are not supported

The `snapshots/` folder is portable and sits next to the executable, so any build dropped into that folder drives the same history. Unresolved state lives in several durable forms, each added as the engine learned what a crash can leave behind: the per-tweak (or, for HKCU tweaks, per-account) Needs Attention record, a journal row's `resolved` mark, an entry's `drive_open` mark, its `actions_in_flight` set, the attention items' `class` and `entries`, and the per-machine shared-claims file `shared_claims.<MachineGuid>.json`. A build older than the one that introduced a form does not know it.

We do not make older builds agree with newer ones. A folder is written by one build line moving forward, and running an older build over a folder a newer one has written is unsupported.

## Considered Options

- **Keep every mark where an older build already parses it**, for example as fields on the entry: rejected. The record deliberately left the entry because dedup, a verified rollback's `consume` and an entry turning invalid all delete or disqualify entries, and the mark has to outlive every one of them. Moving it back to satisfy an older reader reintroduces the loss the record exists to prevent.
- **A migration or compatibility layer in each new build that also writes the old forms**: rejected. It doubles every write path for a flow nobody ships: the released 3.0.0-r1 uses a different layout (flat `snapshots/<tweak-id>.json`) that this engine intentionally ignores, so the only builds that ever share this folder layout are development builds of this engine.

## Consequences

An older build over a newer folder shows badges out of step with reality, in both directions: it ignores marks it does not know, and any entry it rewrites drops the fields it does not know. Its dedup can delete a Settings-only entry whose drive a crash interrupted. A build that knows only the unsuffixed `shared_claims.json` finds no claims record at all, so a first claim there would capture an already-driven value as the original.

What still holds across builds: the entry walk skips every file whose name is not a sequence number, so unknown files are invisible rather than corrupting; a record this build cannot parse surfaces as Needs Attention in its own right; a wrong-schema or foreign-machine record is never overwritten and never deleted; and Keep current state releases the record and discards the entries whatever wrote them, so the user always has a way out.
