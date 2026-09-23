# Known Issues

Defects that are **confirmed to exist and are meant to be fixed**, just not yet.

An entry here is an open bug with its diagnosis already done: what goes wrong, whether it can bite
today, and what the fix looks like. The point is that nobody re-derives the analysis when the work is
picked up. Entries leave this file by being **fixed**.

This is not a place to park things we have decided not to do. A deliberate "we are not fixing this,
and here is why" is an architecture decision and belongs in `docs/adr/`, where the reasoning is
durable and reviewable. If an entry below turns out to be something we accept rather than fix, write
the ADR and delete the entry.

| #   | Issue                                           | Bites today?                                 | Found      |
| --- | ----------------------------------------------- | -------------------------------------------- | ---------- |
| 2   | Broker response is written to user TEMP         | only as a forced false failure, never silent | 2026-09-12 |
| 3   | Nothing inside the broker child is observable   | only as thin support detail after a failure  | 2026-09-12 |

---

## 2. Broker response is written to user TEMP

**What is already guarded.** The request cannot be substituted. `run_elevated_broker` (`services/elevation/broker.rs`) puts the request file's identity (volume serial plus 128-bit file id, `exclusive_temp::file_identity`) on the TrustedInstaller child's command line, and `read_own_request` runs the request only when the file it opened has that identity, reading it through the same handle. The parent holds its `FILE_SHARE_READ`-only write handle for the whole spawn, so a matching identity means exactly the bytes the parent wrote. Anything else exits `EXIT_FOREIGN_REQUEST` before any op runs.

**The residual.** The child still creates its response in the parent's `%TEMP%`, a directory the unelevated side of the same account owns (and can redirect through `HKCU\Environment`). `CREATE_NEW` plus `FILE_FLAG_OPEN_REPARSE_POINT` guard only the last path component, so the unguessable name carries the rest. A same-user process that learns the path can overwrite the response between the child's exit and the parent's read. The in-process read-back turns a forged success into a verify mismatch and a rollback, so the cost is a false failure, never a false success.

**Candidate fix.** Exchange the response (and the request with it) through `%SystemRoot%\SystemTemp`. A medium-integrity process of the admin user holds `Administrators` only as a deny-only SID, so an ACL that grants nothing to `Users` or the user's own SID keeps it out, and because it does not own that directory it has no `FILE_DELETE_CHILD` there to rename or replace files. On the development machine (Windows 11 24H2, build 10.0.26100) the directory exists and `icacls` reports exactly `NT AUTHORITY\SYSTEM:(OI)(CI)(F)` and `BUILTIN\Administrators:(OI)(CI)(F)`. Its presence and ACL on 19045, 22621 and 22631 are unverified.

**Why it is deferred.** Getting it wrong breaks every elevated apply, and two things need a real elevated run: that the TrustedInstaller child can read and write there, and that the directory exists with that ACL on every supported Windows version (a fallback to `%TEMP%` would reopen this gap). A per-run directory under `%TEMP%` with a restrictive DACL does not work: the user owns `%TEMP%`, so `FILE_DELETE_CHILD` lets it rename or delete that directory regardless of its DACL.

## 3. Nothing inside the broker child is observable

**The gap.** The TrustedInstaller child has no logger of its own. An op failure crosses back as `OpFailure.message` in the response, but a transport failure or a panic reaches the parent only as an exit code, so for exactly the failures where the child writes no response there is nothing to read but that number. The parent names each exit code it is given (`describe_broker_exit` in `services/elevation/broker.rs`) and records the level and the classification alongside it, which is as far as the parent alone can see. What it deliberately does not record is the op's own message: it can name a registry key or the data written to it, so an operation refused inside the child is logged by position, and what the operation actually complained about is readable nowhere.

**Candidate fixes, and why each waits.** Carrying structured log lines back inside the response costs a `WIRE_VERSION` bump and still says nothing about the two cases that matter, because a child that panics or cannot write its response writes no response at all. A log file of the child's own does cover them. Two places could hold it: beside the response, which inherits issue 2's directory decision and so is best done after it; or the app's own log directory, which is independent of issue 2 but puts a TrustedInstaller-owned file in a directory the unelevated app also writes, so the ACL question moves rather than disappears. Either way the file is created as TrustedInstaller and needs the `CREATE_NEW` and reparse-point guards the response write already uses, and either way a real elevated run is what settles it.

**The order.** Not forced, but cheaper in one direction: settling issue 2 first leaves the transport directory holding both files under a single decision, so the guard is built once.
