---
status: accepted
---

# Elevation is user-provided, declared per Tweak (refinable per step), and never silently escalated

The app ships unelevated (`asInvoker`). Admin is provided by the *user* — by launching as administrator,
or by an in-app **Elevate** button that relaunches once via `restart_as_admin`; the app never self-elevates
silently. While the process is not admin, Tweaks needing Admin, System, or TrustedInstaller are **disabled**
in the UI. Their status is still detected read-only (reads never need elevation), so the user sees the
current state but can only change a privileged Tweak after choosing to elevate.

There are four declared levels — `User / Admin / System / TI` (`User` is the renamed `None`, and the
successor to today's `requires_admin` / `requires_system` / `requires_ti` flags). Elevation is **declared
per Tweak, refinable per step**: a Tweak declares a **floor** level, and each effect's effective level is
`max(floor, its own declared level)` — a step may escalate above the floor (a mostly-Admin Tweak marking
one service `TI`) but never lowers it. The one exception is correctness, not privilege: a user-hive (HKCU)
effect ignores the floor and always runs in-process as the real user. The levels map to execution context, not to one "elevated child":

- **Admin is a persistent property of the process** — once the user elevates, the whole app stays Admin
  for its lifetime; it is *not* established or released per Tweak, and Admin ops run **in-process** (no
  child).
- **System and TI run in a fresh short-lived child per operation**, so a Tweak's outcome never depends on
  what ran before it (a mis-declared Tweak fails deterministically, never "accidentally works"). System is
  reached by duplicating winlogon.exe's token; TI by **starting the TrustedInstaller service and
  parent-process-spoofing off it** — distinct mechanisms with distinct prerequisites.
- A per-user (HKCU) effect **always** runs in-process as the real user — even inside a System/TI Tweak —
  so it never lands in the elevated account's hive.

The declared level is **trusted, not inferred or validated** at build time, because the privilege a given
resource actually needs is a property of the *machine*, not the tweak (a service observed needing Admin on
one PC and TrustedInstaller on another). Two failures are distinguished, both surfaced as a named
**insufficient-elevation** error (abort + Rollback), never a silent escalation:

- **Couldn't acquire the declared level** — the token could not be assumed at all: the TI service would not
  start, `SeDebugPrivilege` was denied, or winlogon was not found. Environmental or self-inflicted, not a
  mis-declared level. Because TI depends on starting the TrustedInstaller service, a Tweak that **disables**
  that service is rejected at build — it would break the app's own TI path for every later TI Tweak.
- **Acquired the level but the operation was still access-denied** — the declaration is genuinely too low
  for this machine; the author corrects it.

## Considered Options

- **Require admin at launch (manifest `requireAdministrator`)** — rejected: it prompts on every launch even
  for read-only browsing and locks non-admin users out entirely, when detection needs no elevation at all.
- **Lazy self-elevation to admin on the first privileged Apply** — rejected: elevation must be a deliberate,
  visible user choice, not something the app initiates mid-flow.
- **Sticky/global elevation reused across Tweaks** — rejected: it makes a Tweak's success depend on what ran
  before it, so a mis-declared Tweak "accidentally works" in a batch yet fails alone. Per-Tweak scoping makes
  failure deterministic.
- **Build-time inference or validation of the required level** — rejected: the required privilege varies by
  machine, so any static guess is wrong somewhere.
- **Escalate on access-denied (Admin → System → TI)** — rejected: it would run an operation at a higher
  privilege than the user was shown, breaking the rule that the elevation displayed equals the elevation
  used. Viable later only as an explicit, visible choice.

## Consequences

A Tweak's required Elevation Level is part of the compiled model and surfaced to the frontend, which uses it
to disable privileged Tweaks until the app is elevated. "Insufficient elevation" is a distinct, named
failure, not a generic error; the remedy is for the user to elevate or for the author to correct the
declaration — never a silent workaround. Batching a Tweak's operations into one elevated child process (the
broker wire protocol already supports it) is a process-spawn optimization only; there is at most one
elevation prompt — the elevation itself — so nothing here trades away UAC prompts.

## Amended 2026-07-22 (tweak-system redesign, spec rev 2)

- **Reads run at whatever level the app currently has** — this supersedes the parenthetical above
  that "reads never need elevation," and equally the Considered-Options rationale that "detection
  needs no elevation at all." Most state is world-readable, so unelevated detection works; but
  TI-protected resources (WaaSMedic-class keys and tasks) legitimately deny reads, and those
  tweaks report the **Unknown** status with a needs-elevation hint until the user elevates. Detection
  never guesses, never shows a fake System Default, and reads never trigger elevation.
- **The TI self-availability build rejection is scoped to *typed* effects.** The sentence above
  ("a Tweak that disables that service is rejected at build") holds for Service/Registry Settings;
  script Actions are statically opaque, so a script-based TI-disable cannot be build-rejected —
  script review guidance carries that residual. The guard's claim is honest, not categorical.
- **Over-the-shoulder guard.** "In-process as the real user" fails when a *different* admin account's
  credentials elevated the app (the relaunched process's HKCU is that admin's hive, and every write,
  read-back verification, and detection would agree on the wrong target). At startup the app compares
  its process-token SID with the interactive session's user SID; on mismatch, **User-level
  (HKCU-touching) tweaks are disabled with a clear message**. This closes the blind spot the original
  text shared with the design.
- **Grouped execution is committed for v1** (not deferred): consecutive same-level System/TI steps
  batch into one child via the existing multi-op wire protocol; the multi-op caller is the net-new
  wiring, order-preserving. User/Admin steps stay in-process and are never grouped.
