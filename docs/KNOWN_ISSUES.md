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
| 4   | An older build cannot read what this one resolved | only where two builds share one folder     | 2026-09-12 |
| 5   | Atomic writes are not flushed through a power loss | only on power loss right after a write     | 2026-09-23 |
| 6   | Update installer is saved predictably and run unverified | yes, to a same-user process during an update | 2026-09-23 |

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

## 4. An older build cannot read what this one resolved

Unresolved state has four durable forms. Needs Attention lives in `snapshots/<tweak-id>/_attention.json` (`_attention.<SID>.json` for a tweak that touches HKCU); a journal row an operation has accounted for carries `resolved: true` inside its own entry; an apply or restore a crash interrupted leaves `drive_open: true` on its entry; and an action undo or re-run it interrupted stays in the entry's `actions_in_flight` (`tweaks/snapshot.rs`). Builds before the record know none of them. The immediately previous build (the one that introduced the record, `resolved` and `record_unreadable`) knows the first two and not the last two. The snapshots directory is portable and sits next to the executable, so any build dropped into that folder drives the same history.

**What happens then.** Nothing breaks, and apart from the dedup case below no snapshot entry is lost. The entry walk skips every file whose name is not a sequence number, so the record is invisible to the older build rather than corrupting it, and a record this build cannot use is surfaced as Needs Attention in its own right instead of reading as a clean tweak. A wrong-schema or foreign-machine record is never overwritten and never deleted: this build refuses to record over one and logs that it refused, so neither build can silently destroy the other's mark. The row mark is the softer case, because an unknown JSON field is simply ignored: the older build's crash scan raises a row this build already resolved, and any entry that build rewrites to mark an action completed drops the `resolved` flag on every row in that entry, so this build raises it again afterwards. The new marks fail the other way, and the immediately previous build shows all of it: its scan ignores `drive_open` and `actions_in_flight`, so an interruption this build marked raises nothing there; any entry it rewrites (marking an action completed, resolving a row) drops both fields; and its dedup, which keeps only an entry with an outstanding row, deletes a Settings-only entry whose drive a crash interrupted as soon as the same option is captured again, so that crash is lost for good. A record with the `outcome_unrecorded` reason or an `unrecorded` item does not parse there either, so that build surfaces it as an unreadable record. An item's `class` (access denied, not found, busy and the like) is the soft case: an older build ignores the unknown field, so the record still loads and the item reads by its message alone, and any record that build rewrites drops the field. What the user sees either way is a badge out of step with reality, in both directions: a tweak this build marked shows nothing under the older one, and a tweak the older one applies or restores successfully still shows Needs Attention the next time this build reads it.

**Two residuals in the same marks, both accepted for now.** When the record already carries an `unrecorded` item (an outcome that verified but could not settle its drive mark), the startup scan treats every open drive mark as explained by it, so a later crash that leaves a new drive mark is not added as its own item: the tweak stays badged, but the text still says the last operation ended in a verified state. And Keep current state settles drive marks and unfinished steps before it releases the record, but not outstanding journal rows, so if one of its discards then fails, the next launch raises that entry's rows again. That second one predates the drive mark.

**The way out already exists in the UI:** Keep current state is a single backend operation that releases the record whether or not any entry is left and discards the ones that are (another account's HKCU entries excepted), which takes their rows with them, and a fully verified apply or restore under this build resolves the rows it accounted for.

**The shared-claims record moved, and only pre-release builds of this branch are affected.** This build keeps it in `snapshots/shared_claims.<MachineGuid>.json` (schema version 2). It reads an unsuffixed `shared_claims.json` stamped for this machine, version 1 or 2, and folds it into its own file on its next write, deleting the old one; a version 1 record restores at the releasing tweak's own route, with a log line. A build that knows only the unsuffixed file then finds no record, so a first claim there would capture the already-driven value as a fabricated original, and a release there reports the claim as not held. No released build reads either file, since `main` has no shared claims, so the exposure is limited to development builds sharing one folder.

**The fix.** Nothing in any mark: this resolves when the older build is gone, and downgrades are not a supported flow. Cross-build agreement would need the marks to live somewhere an older build already parses, and for the record that is exactly the entry field this design moved away from, because entry releases kept dropping it.

## 5. Atomic writes are not flushed through a power loss

Every snapshot entry, journal mark, drive mark, `_seq.json` and `_attention.json` is written as a temp file, `sync_all`, then renamed over the target (`write_atomic` and `rewrite_entry` in `tweaks/snapshot.rs`). The rename itself is neither `MOVEFILE_WRITE_THROUGH` nor followed by a flush of the directory, so after a power loss NTFS can come back with the old file in place: the new content was on disk, but the rename that published it was not.

**What happens then.** A mark written just before a change can be lost while the change it guards survives. The registry value or service start type is set, but the drive mark, the in-flight action or the completed row that should say so is gone, and the next launch raises nothing for it or raises a row that did in fact finish. A crash of the app alone is not affected, since the rename reached the file system before the process died; only a power loss or an OS crash shortly after a write can do this. The pattern predates the drive mark and applies to every write in the store.

**The fix.** Rename with `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)` (or `SetFileInformationByHandle` with `FileRenameInfoEx` and a flush of the directory handle) in the one atomic-write helper, and measure the cost: every apply and restore writes several marks, so a slower write is paid on each.


## 6. The update installer is saved predictably and run unverified

**Where.** `install_update_in` in `commands/update.rs`. The download goes to `std::env::temp_dir().join(&asset_name)`, is written with `std::fs::write`, and is launched by path: `SystemTool::Msiexec` with `/i` for an `.msi`, `Command::new(&download_path)` otherwise.

**The root cause.** Three independent gaps in the same few lines:

- The file name is the release asset name, known in advance, in the user's `%TEMP%`. `std::fs::write` opens with create-or-truncate, follows whatever already sits at that name (a reparse point included), and asks for no sharing restriction, so another process can hold its own write handle across ours.
- Between the write and the launch nothing holds the file, so its content can change before the installer starts, and the launch resolves the path again.
- Nothing checks that what runs is what was downloaded or what was published: there is no hash check, and the releases are unsigned (`certificateThumbprint` is `null` in `tauri.conf.json`), so there is no signature to check either.

When the app runs elevated (after Restart as administrator), the installer inherits the elevated token, so whatever controls the file content at launch runs elevated. The same code is on `main`; the redesign branch added URL and asset-name validation and an absolute `msiexec` path but left the file handling unchanged.

**Also in this flow, lower weight.** `is_trusted_download_url` compares raw string prefixes. A URL containing `..` segments passes it and is normalized by the HTTP client afterwards, the `objects.githubusercontent.com/` prefix admits any repository's assets, and redirects are not re-checked. The URL arrives through the IPC arguments, so this is reachable only from a compromised webview.

**Fix.**

1. Create the file exclusively under an unguessable name: `exclusive_temp::unique_temp_path`, `create_new`, and a `FILE_SHARE_READ`-only handle, keeping the asset's extension (neither `msiexec` nor the NSIS installer cares about the rest of the name). `ExclusiveTempFile` deletes its file on drop, which would delete the installer while it runs, so add a way to keep the file, or open it directly the same way.
2. After writing, close the write handle, reopen the file read-only with `FILE_SHARE_READ` only (no writer or deleter can then open it), read it back through that handle, compare it with the downloaded bytes, and keep the handle open until the installer process has started. A read-only, share-read handle should not block the image loader, which opens the file for read and execute; confirm that on the manual run below, since a sharing violation there would stop every update.
3. Check integrity against the release: the GitHub release API returns a `digest` (`sha256:<hex>`) for each asset. Add it to `GitHubAsset`, carry it through `UpdateInfo`, and compare it with a SHA-256 of the bytes (CNG `BCryptHash` through `windows-sys`). Refuse to install when it is missing or differs. Signing the releases later would allow `WinVerifyTrust` on top.
4. Parse the URL instead of prefix-matching it: require `https`, host `github.com`, and a path under `/ehsan18t/magicx-toolbox/releases/download/`; reject `..` segments; and re-check the host after each redirect, or turn redirects off and follow them manually through the same check.

**How to verify.** Unit tests for the URL check (dot segments, other repositories, lookalike hosts) and for the digest comparison (match, mismatch, missing). The file handling needs one manual update from an older build to a newer release, run once unelevated and once after Restart as administrator.
