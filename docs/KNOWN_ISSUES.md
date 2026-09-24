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
| 2   | Broker transport falls back to user TEMP where SystemTemp is missing | only there, and only as a forced false failure | 2026-09-12 |
| 3   | Nothing inside the broker child is observable   | only as thin support detail after a failure  | 2026-09-12 |

---

## 2. Broker transport falls back to user TEMP where SystemTemp is missing

**What is already guarded.** The request cannot be substituted anywhere: `run_elevated_broker` (`services/elevation/broker.rs`) passes the request file's identity on the TrustedInstaller child's command line, and `read_own_request` runs only the file with that identity, read through the same handle (`EXIT_FOREIGN_REQUEST` otherwise). The request and response are exchanged through `%SystemRoot%\SystemTemp` (`system_temp`), which grants only SYSTEM and Administrators, so the unelevated side of the account can neither read nor replace the response. `a_real_trusted_installer_child_serves_only_the_request_it_was_given` (ignored; needs an elevated run after `cargo build`) drives the real TrustedInstaller child through that folder, and passes on Windows 11 24H2 (build 26100).

**The residual.** Where `SystemTemp` is missing, or refuses the request file, the transport falls back to the parent's `%TEMP%` and logs that it did. There the response is created in a directory the unelevated side owns, so a same-user process that learns its path can overwrite it between the child's exit and the parent's read. The in-process read-back turns a forged success into a verify mismatch and a rollback, so the cost is a false failure, never a false success. The request stays bound by its identity either way.

**What is unknown.** Whether `SystemTemp` exists with the same ACL on 19045, 22621 and 22631. The Manual Tests case F62 checks exactly that on a real machine; run it once per supported build. If a build lacks it, the options are to create it with the same ACL at install time, or to accept the fallback there.

## 3. Nothing inside the broker child is observable

**The gap.** The TrustedInstaller child has no logger of its own. An op failure crosses back as `OpFailure.message` in the response, but a transport failure or a panic reaches the parent only as an exit code, so for exactly the failures where the child writes no response there is nothing to read but that number. The parent names each exit code it is given (`describe_broker_exit` in `services/elevation/broker.rs`) and records the level and the classification alongside it, which is as far as the parent alone can see. What it deliberately does not record is the op's own message: it can name a registry key or the data written to it, so an operation refused inside the child is logged by position, and what the operation actually complained about is readable nowhere.

**Candidate fixes, and why each waits.** Carrying structured log lines back inside the response costs a `WIRE_VERSION` bump and still says nothing about the two cases that matter, because a child that panics or cannot write its response writes no response at all. A log file of the child's own does cover them. Two places could hold it: beside the response, which inherits issue 2's directory decision and so is best done after it; or the app's own log directory, which is independent of issue 2 but puts a TrustedInstaller-owned file in a directory the unelevated app also writes, so the ACL question moves rather than disappears. Either way the file is created as TrustedInstaller and needs the `CREATE_NEW` and reparse-point guards the response write already uses, and either way a real elevated run is what settles it.

**The order.** Not forced, but cheaper in one direction: settling issue 2 first leaves the transport directory holding both files under a single decision, so the guard is built once.
