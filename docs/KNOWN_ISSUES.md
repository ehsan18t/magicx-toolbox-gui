# Known Issues

Defects that are **confirmed to exist and are meant to be fixed**, just not yet.

An entry here is an open bug with its diagnosis already done: what goes wrong, whether it can bite today, and what the fix looks like. The point is that nobody re-derives the analysis when the work is picked up. Entries leave this file by being **fixed**.

This is not a place to park things we have decided not to do. A deliberate "we are not fixing this, and here is why" is an architecture decision and belongs in `docs/adr/`, where the reasoning is durable and reviewable. If an entry below turns out to be something we accept rather than fix, write the ADR and delete the entry. Wanted improvements that are not defects live in `ROADMAP.md`.

| #   | Issue                                                                          | Bites today?                                               | Found      |
| --- | ------------------------------------------------------------------------------ | ---------------------------------------------------------- | ---------- |
| 2   | Broker transport falls back to user TEMP where SystemTemp is missing           | only there, and only as a forced false failure             | 2026-09-12 |
| 3   | A broker child that is killed or cannot write its response loses its log lines | only as thin support detail after a failure                | 2026-09-12 |
| 4   | Profiles have no backend: every profile call rejects                           | yes, the Profiles page and both profile dialogs do nothing | 2026-10-03 |

---

## 2. Broker transport falls back to user TEMP where SystemTemp is missing

**What is already guarded.** The request cannot be substituted anywhere: `run_elevated_broker` (`services/elevation/broker.rs`) passes the request file's identity on the TrustedInstaller child's command line, and `read_own_request` runs only the file with that identity, read through the same handle (`EXIT_FOREIGN_REQUEST` otherwise). The request and response are exchanged through `%SystemRoot%\SystemTemp` (`system_temp`), which grants only SYSTEM and Administrators, so the unelevated side of the account can neither read nor replace the response. `a_real_trusted_installer_child_serves_only_the_request_it_was_given` (ignored; needs an elevated run after `cargo build`) drives the real TrustedInstaller child through that folder, and passes on Windows 11 24H2 (build 26100).

**The residual.** Where `SystemTemp` is missing, or refuses the request file, the transport falls back to the parent's `%TEMP%` and logs that it did. There the response is created in a directory the unelevated side owns, so a same-user process that learns its path can overwrite it between the child's exit and the parent's read. The in-process read-back turns a forged success into a verify mismatch and a rollback, so the cost is a false failure, never a false success. The request stays bound by its identity either way.

**What is unknown.** Whether `SystemTemp` exists with the same ACL on 19045, 22621 and 22631. The Manual Tests case `systemtemp_transport` checks exactly that on a real machine; run it once per supported build. If a build lacks it, the options are to create it with the same ACL at install time, or to accept the fallback there.

## 3. A broker child that is killed or cannot write its response loses its log lines

**What is already covered.** The TrustedInstaller child keeps its own log lines in memory and returns them inside its response (`BrokerResponse.log`); the parent re-logs them under the `helper` source, tagged with the batch number, once the response passes the version and nonce checks (ADR-0010). A failed op also returns its numeric Windows error code (`OpFailure.win32`), and with Detailed logging on, the op's own error text comes back as a Debug line the parent redacts. A child that panics after reading its request writes a panic report (`PanicReport`) with its lines and the panic message to the response path; the parent re-logs both and still treats the batch as outcome-unknown. The Manual Tests case `ti_helper_log` checks the round trip on a real machine.

**The residual.** Two cases still return nothing but an exit code, because the child writes no file in either: a child that is killed (an antivirus kill, or the parent's termination at the 30 second timeout), and one that ran but could not write its response file (`EXIT_UNWRITABLE_RESPONSE`, `EXIT_UNSERIALIZABLE_RESPONSE`). For those the parent names the exit code (`describe_broker_exit` in `services/elevation/broker.rs`) in its own line, and whatever the child logged is lost. A child that refuses its request also returns only an exit code, but it has run nothing to report.

**Candidate fix, and why it waits.** Only a log the child writes as it goes survives a kill. That file would be created as TrustedInstaller, need the `CREATE_NEW` and reparse-point guards the response write already uses, and live either beside the response, which inherits issue 2's directory decision, or in the app's logs folder, which the unelevated app also writes, so the ACL question moves rather than disappears. ADR-0010 rejected a child log file for the common case; for these two cases the cost is still open, and a real elevated run is what settles it.

**The order.** Settle issue 2 first: the transport directory then holds the request, the response and any such file under a single decision.

## 4. Profiles have no backend: every profile call rejects

**What happens.** The profile commands were removed while the format is rebuilt (`docs/spec/profile-v1.md`). `src/lib/api/profile.ts` keeps the frontend surface but rejects every call at one choke point, so no `invoke()` reaches an unregistered command. The Profiles page shows a "Profiles are being rebuilt" notice, keeps New profile disabled, and does not load saved profiles on mount, since that load can only fail. The export wizard (`modals/profile/ProfileExportModal.svelte`) has no opener, so it is kept but parked unmounted. Open folder and Reset folder still trigger a load, which fails and shows the error on the page.

**The fix.** Once the backend returns: register the commands and drop the rejections in `api/profile.ts`, load saved profiles when `ProfilesView` mounts, remove the notice, enable New profile, and mount `ProfileExportModal` in `App.svelte` again with New profile as its opener.
