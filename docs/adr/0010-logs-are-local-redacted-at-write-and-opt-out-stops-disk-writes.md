---
status: accepted
---

# Logs stay on the device, are redacted when written, and opting out stops disk writes only

A support report is only as good as the log behind it, and a Windows tweaking app's log is full of personal detail: the user's profile path, account and computer names, the account SID, the MachineGuid, script output that quotes any of them. The app also changes protected settings through a TrustedInstaller child that, until now, logged nothing the user could ever see.

We keep one on-device logger (`src-tauri/src/logging/`) with these rules:

- **Logs never leave the PC on their own.** Nothing is uploaded. The only way a log leaves is Export diagnostics, which the user starts, and which writes one plain-text file to a place the user picks.
- **Redaction happens before any sink.** Every line is redacted once, as it is recorded, before it reaches the in-memory session buffer, the Logs panel, the session file or an export. Exports do not redact a second time: the files they copy are already redacted. The identity the redactor looks for (paths, names, SID, MachineGuid) is gathered once, before the logger is installed, so nothing is computed inside it.
- **Opting out stops disk writes only.** "Save logs on this PC" is on by default. Turning it off closes the session file and writes no new ones; the session buffer and the Logs panel keep working until the app closes, and Export diagnostics then carries this session from memory. Existing files are kept until the user deletes them or retention prunes them.
- **The TrustedInstaller child's log lines travel in its response.** The child never writes a log file. It keeps its own lines in memory (at the level the parent asks for, capped at 200 lines of 512 bytes) and returns them inside the response JSON, or inside a panic report if it panics. The parent re-logs them, redacted by its own pipeline, only after the response or report has passed the version and nonce checks.
- **File order is the writer lock's order.** Lines reach the session file in the order writers take the file lock, which can differ by a few lines from the sequence numbers the session buffer assigns. When saving starts, the buffer so far is copied into the new file and writers skip every line already copied, so no line is lost or written twice.

This amends the broker's rule that an op's error text never crosses back to the parent. That text can quote the registry key, the value or the path an op touched, so the child still never puts it in `OpFailure`; it now logs it at Debug only, which crosses only when Detailed logging is on, and the parent redacts it like any other helper line.

## Considered Options

- **Opt-in logging (off until the user enables it)**: rejected. The first failure on a user's machine would then have no record, and asking the user to reproduce a one-off failure after enabling logging rarely works.
- **Opting out stops all logging, memory included**: rejected. The Logs panel is how a user sees what just failed; turning it off with the disk switch costs that without making anything more private, since the memory buffer never leaves the process.
- **Redact at export time**: rejected. The panel, the session files and a copied line would all hold raw values, and a user who attaches a session file directly would share them.
- **A log file written by the TrustedInstaller child**: rejected. The file would be created as TrustedInstaller in a folder the unelevated app also writes, which moves the transport's ACL question rather than settling it, and it would need its own retention.
- **Upload or crash-report service**: rejected. Nothing in the app talks to a server about the user's machine, and a report is the user's to send.

## Consequences

- A user who reports a problem exports one file and can read it before attaching it; the export toast says to check it.
- Redaction covers known values and patterns only. A personal detail outside them, such as a name in a file path that is not a user profile, can still appear, which is why the user is told to check the file.
- Helper lines from a child that is killed (an antivirus kill, the 30 second timeout) or that ran but could not write its response file are still lost: only the parent's exit-code line records those (KNOWN_ISSUES issue 3).
- The broker wire version is 4: the request carries `detailed`, the response carries `log`, and a failed op carries its numeric Windows error code (`win32`), which is not personal. Exit codes are unchanged, and a panic is still outcome-unknown whatever its report says.
