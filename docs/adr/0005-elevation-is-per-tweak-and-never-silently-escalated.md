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

## Amended 2026-07-27 (over-the-shoulder guard: corrected, not removed)

The guard as first implemented disabled every per-user tweak on every machine, at every privilege
level. Three separate faults, all now fixed. The guard itself stands; its inputs were wrong.

- **The probe could never succeed.** It read the console session's user via `WTSQueryUserToken`, which
  requires `SE_TCB_NAME`. Only LocalSystem holds that privilege, so the call returned
  `ERROR_PRIVILEGE_NOT_HELD` (1314) for every real run, elevated or not. Measured at both levels on a
  developer machine. The guard's fail-closed arm then read that guaranteed failure as a mismatch and
  refused all 54 `elevation: user` tweaks. **Now:** `ProcessIdToSessionId` ->
  `WTSQuerySessionInformationW(WTSUserName/WTSDomainName)` -> `LookupAccountNameW`, which needs no
  privilege and was verified working at both levels.
- **It compared against the wrong session.** The console session's owner is not necessarily the user
  this process serves: under RDP the app can run in session 2 while another user holds the console,
  which is a legitimate configuration and not an over-the-shoulder elevation. **Now:** the comparison
  is against the owner of the process's own session.
- **It keyed on the declared floor, not on the hive.** `elevation:` states which privilege a tweak
  needs; the hive states whose state it changes. Those are independent, and 31 `admin`-floor tweaks in
  the shipped corpus drive HKCU effects, every one of which the guard let through unexamined while it
  blocked the `user`-floor ones wholesale. **Now:** the guard consumes
  `context::tweak_touches_hkcu`, built on the same `effect_is_hkcu` that routing uses, so the two
  cannot answer the question differently. A corpus-wide test asserts that agreement for every tweak.

- **The hive check covers every shape that names one.** Besides settings and shared blocks, a
  `DeleteTree` action carries a `KeyAddr` and therefore a hive. Missing it would have left an HKCU
  *subtree deletion*, the most destructive effect kind, routed by the floor and running against an
  elevated child's own hive.
- **A directory-free fallback keeps the guard from re-creating the bug it fixes.** Resolving the
  session owner's SID uses `LookupAccountNameW`, a name lookup that fails on a domain-joined machine
  with an unreachable DC or an Entra-joined machine. Falling straight to "blocked" there would repeat
  the original failure on a different population of machines. When SID resolution fails the guard
  compares SAM account names instead (`GetUserNameExW(NameSamCompatible)` against the WTS session
  owner), both of which come from local token/session state. The comparison is symmetric by
  construction: names are only ever compared with names, so a half-resolved state degrades to
  "unknown" and can never masquerade as a difference between accounts.

**Fail-closed is reaffirmed, and the reasoning is recorded here because it is not obvious.** An
unreadable SID still blocks, and it now reports a distinct state (`SidUnknown`) rather than accusing
another account. Failing *open* was considered and rejected: it buys no availability once the probe
works, and it is unrecoverable when it is wrong. The snapshot store is keyed to the machine
(`Entry` carries `machine_guid` and no user identity) while HKCU is keyed to the account, and apply
verifies a write by reading back the same hive it just wrote, so a wrong-account write is
self-confirming green with no return point. Refusing mutates nothing. See
`docs/plans/fix-hkcu-user-level-gate.md`.

**Related, deliberately unfixed:** the same machine-keyed-store/account-keyed-hive mismatch is
reachable without any elevation at all, on a multi-user machine sharing one portable install. Recorded
in `PRE_MERGE_TASKS.md` item 4 and consciously deferred; it predates this amendment and is not caused
by it.

---

## Amendment: three levels, grouped execution, and a third failure mode

Three changes to this decision, each forced by what the code and the corpus actually turned out to be.

### The SYSTEM level is gone

This ADR described four declared levels and two elevation mechanisms. The shipped corpus declares 191 `admin`, 65 `user`, 21 `ti` and **zero** `system`, and `Level::User`/`Level::Admin` never reach the broker at all. So the SYSTEM mechanism, duplicating winlogon.exe's token and spawning through `CreateProcessWithTokenW`, had no production caller and never had one.

Keeping it was not free. It was unsafe Win32 that no test exercised, and duplicating a SYSTEM token off winlogon is the most antivirus-legible thing this application could do. It is deleted, and `elevation: system` is now a build-time parse error rather than a silently unreachable declaration.

Three levels remain: `user`, `admin`, `ti`. The floor-and-escalate rule, the HKCU exception, and the over-the-shoulder guard are unchanged.

### System and TI no longer run "a fresh child per operation"

The original text said System and TI run in a fresh short-lived child **per operation**, justified by determinism: a tweak's outcome should not depend on what ran before it.

That justification survives; the per-operation reading does not. Acquiring TrustedInstaller is entirely a per-spawn cost (connect to the SCM, start and poll the service, open and verify its process, build an attribute list, cold-start this binary again, round-trip JSON through the filesystem), and the one tweak that reaches the broker has 21 elevated effects. It paid that cost 21 times to apply and up to 19 more to roll back.

A run of **consecutive same-level** effects now shares one child. Determinism is preserved by what the grouping refuses to do: it never reorders, it only groups adjacent equals, and anything that is not a same-level brokerable Setting (an in-process effect, a Shared block, an Action, an HKCU effect the routing forces in-process) splits the run. A batch still stops at its first failure, and the failing operation is still named.

### A third failure mode: the child ran, and we cannot say how far

This ADR named two failures and required that neither ever be silently downgraded to the other. The code honoured that for the two it named, and had nowhere to put a third case that genuinely occurs: a child terminated mid-batch on a timeout, a child whose wait or exit-code query failed, a child that panicked, a child that completed the batch but could not return its response, or a response that failed validation. Every one of those was reported as "could not acquire", which says the machine is unchanged.

That was load-bearing in the wrong direction. On a drive failure the engine rolls back and, if every restore verifies, consumes the snapshot entry, because ADR-0002 says a verified full restore leaves nothing for the entry to describe. Once a child has run, what it did cannot be proven, even though the rollback restores every captured setting (including any the child drove) and verifies them. The child's own account is missing or untrusted, a Service Control Manager or Task Scheduler call it made may still complete server-side after it is killed, and a child whose kill could not be confirmed may still be running; any of these can change a setting after it verified, and the entry describing it was deleted precisely then. ADR-0002 always allows keeping an entry, so an unknown outcome keeps it.

**Indeterminate** is now a first-class outcome, deliberately neither of its neighbours. Reported as "could not acquire" it claims nothing happened; reported as an operation failure it blames an operation that may have succeeded. The tweak still rolls back. What changes is that the snapshot survives, which is what surfaces the tweak as Needs Attention rather than as silently finished. This is the same principle as ADR-0001: a state that cannot be verified is surfaced, never hidden.
