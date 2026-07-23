//! Hosts file service for managing entries in the Windows hosts file.
//!
//! The hosts file is located at C:\Windows\System32\drivers\etc\hosts
//! and requires administrator privileges to modify.

use crate::error::Error;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

use windows_sys::core::PWSTR;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_LOCK_VIOLATION, ERROR_NOT_ALL_ASSIGNED,
    ERROR_SHARING_VIOLATION, HANDLE, LUID,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW,
    SetNamedSecurityInfoW, SDDL_REVISION_1, SE_FILE_OBJECT,
};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, EqualSid, LookupPrivilegeValueW, ACL, DACL_SECURITY_INFORMATION,
    LUID_AND_ATTRIBUTES, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
    SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::Storage::FileSystem::{ReplaceFileW, REPLACEFILE_WRITE_THROUGH};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Bounded retry for `ReplaceFileW` against a transient sharing/lock violation — empirically
/// confirmed against the real hosts file during Task 7 hardening: a filter driver (commonly AV
/// real-time protection watching this specific, classic-malware-target file) can hold it open for
/// a moment, failing the very next replace attempt with `ERROR_SHARING_VIOLATION`, yet an
/// immediate retry succeeds. `REPLACE_RETRY_ATTEMPTS` short, fixed-delay attempts (well under a
/// second total) absorb that without turning a routine, momentary AV scan into a user-facing
/// failure; any other error, or the retries running out, still surfaces as a typed `Err`. This
/// entire window is *before* the swap (see `replace_hosts_file_atomically`): the replacement file
/// already carries the correct content and security by the time the first attempt runs, so a
/// retry can only ever repeat the one, whole, already-correct swap — never double-apply anything.
const REPLACE_RETRY_ATTEMPTS: u32 = 5;
const REPLACE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

/// Marker comment to identify entries managed by MagicX Toolbox
const MAGICX_MARKER: &str = "# MagicX Toolbox";

/// Get the path to the Windows hosts file
fn get_hosts_path() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

/// Null-terminated UTF-16 encoding of a path, for the `PCWSTR` params the file-replace APIs take.
/// Widens straight from `OsStr` (never through a lossy `&str`/`to_string_lossy` step), matching
/// this codebase's existing `wide`/`to_wide_string` helpers (`service_control.rs`,
/// `elevation/common.rs`) but path-typed since both inputs here are always `Path`s.
fn wide_path(p: &Path) -> Vec<u16> {
    p.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Null-terminated UTF-16 encoding of a plain string (the one privilege name this file widens).
fn wide_str(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// A call-unique (not just process-unique) temp filename in the hosts directory. Process id alone
/// is not enough: spec invariant 25 makes batch operations per-tweak-independent, and `rayon`
/// (already a dependency) means two threads in this one process can really call
/// `replace_hosts_file_atomically` concurrently and collide on one path otherwise.
static TMP_COUNTER: AtomicU32 = AtomicU32::new(0);
fn unique_tmp_name() -> String {
    format!(
        "magicx-hosts-{}-{}.tmp",
        std::process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// The hosts file's owner + DACL, captured up front so the *replacement* file can be made correct
/// before it ever becomes `hosts` — never restored on the real file after the fact (code review
/// Fix 2: a post-swap fix-up leaves a window where either a capture failure or a restore failure
/// strands the real file mis-owned, permanently and silently).
///
/// `ReplaceFileW` preserves *some* of the original security descriptor on its own, but Task 7
/// hardening empirically found it does NOT preserve the owner: a fresh temp file (owned by
/// whichever elevated account is running) would carry that ownership through the swap — silently
/// handing this file to that specific admin account instead of leaving it owned by
/// `BUILTIN\Administrators` — and owner-driven ACL inheritance can then graft extra ACEs on top.
/// Applying the captured security to the replacement *before* the swap (`apply_security`, called
/// from `replace_hosts_file_atomically` before `ReplaceFileW`) closes that gap by construction:
/// whatever `ReplaceFileW` does or doesn't carry over stops mattering, because the file it swaps
/// in already has the right answer.
///
/// `sd` is the security-descriptor buffer Windows allocated for this; `owner`/`dacl` point
/// *inside* it, so neither may outlive `sd`, which is freed via `LocalFree` on drop.
struct SecurityCapture {
    sd: PSECURITY_DESCRIPTOR,
    owner: PSID,
    dacl: *mut ACL,
}

impl Drop for SecurityCapture {
    fn drop(&mut self) {
        if !self.sd.is_null() {
            // SAFETY: `sd` was allocated by `GetNamedSecurityInfoW`, which documents `LocalFree`
            // as the correct release for its `ppSecurityDescriptor` output.
            unsafe {
                LocalFree(self.sd);
            }
        }
    }
}

/// Reads `path`'s current owner + DACL. `None` means we could not determine them — the caller's
/// contract (`replace_hosts_file_atomically`) is to fail closed on `None` for an existing file,
/// never to proceed with an irreversible swap whose permission side effect it could not correct.
fn capture_security(path: &Path) -> Option<SecurityCapture> {
    let wide = wide_path(path);
    let mut owner: PSID = ptr::null_mut();
    let mut dacl: *mut ACL = ptr::null_mut();
    let mut sd: PSECURITY_DESCRIPTOR = ptr::null_mut();

    // SAFETY: `wide` is valid and NUL-terminated for the whole call; the group/SACL out-params we
    // don't want back are null, exactly as MSDN documents for "don't retrieve this".
    let err = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut sd,
        )
    };
    if err != 0 || sd.is_null() {
        return None;
    }
    Some(SecurityCapture { sd, owner, dacl })
}

/// Best-effort: enables `SeRestorePrivilege` on the current process token. Reassigning an
/// arbitrary captured owner (`apply_security`, below) can require it when that owner isn't a group
/// the token already belongs to — it has worked without this on this machine only because the
/// captured owner happens to be the freely-assignable `BUILTIN\Administrators`, which is not
/// guaranteed on every machine. Not enabling it here is never itself a failure:
/// `apply_security`'s own `SetNamedSecurityInfoW` call afterward is the real, typed pass/fail gate
/// (code review: "proceed only if the assignment succeeds anyway — otherwise the typed error").
fn try_enable_restore_privilege() {
    // SAFETY: standard token-privilege-adjustment sequence; the token handle is closed before
    // returning, on every path.
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            log::debug!(
                "OpenProcessToken failed while enabling SeRestorePrivilege: {}",
                GetLastError()
            );
            return;
        }

        let name = wide_str("SeRestorePrivilege");
        let mut luid: LUID = std::mem::zeroed();
        if LookupPrivilegeValueW(ptr::null(), name.as_ptr(), &mut luid) == 0 {
            log::debug!(
                "LookupPrivilegeValueW(SeRestorePrivilege) failed: {}",
                GetLastError()
            );
            CloseHandle(token);
            return;
        }

        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0] = LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        };

        let adjusted = AdjustTokenPrivileges(token, 0, &tp, 0, ptr::null_mut(), ptr::null_mut());
        let err = GetLastError();
        CloseHandle(token);

        if adjusted == 0 {
            log::debug!("AdjustTokenPrivileges(SeRestorePrivilege) failed: {err}");
        } else if err == ERROR_NOT_ALL_ASSIGNED {
            log::debug!("SeRestorePrivilege is not held by this token; proceeding without it");
        } else {
            log::trace!("SeRestorePrivilege enabled");
        }
    }
}

/// Applies a previously `capture_security`d owner + DACL onto `path`.
///
/// A privileged call, never silently discarded. Called on the *temp* file before the swap (see
/// `replace_hosts_file_atomically`), so a failure here costs nothing — the real hosts file has not
/// been touched yet, and the caller deletes the temp file and bails.
fn apply_security(path: &Path, capture: &SecurityCapture) -> Result<(), Error> {
    let wide = wide_path(path);
    // SAFETY: `wide` is valid and NUL-terminated; `capture.owner`/`capture.dacl` are still valid
    // because `capture.sd` (which they point into) has not been freed yet.
    let err = unsafe {
        SetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            capture.owner,
            ptr::null_mut(),
            capture.dacl,
            ptr::null(),
        )
    };
    if err != 0 {
        return Err(Error::WindowsApi(format!(
            "Failed to apply the captured owner/permissions to {} (error {})",
            path.display(),
            err
        )));
    }
    Ok(())
}

/// Best-effort human-readable SDDL string for an already-captured security descriptor — used only
/// for the error log in `verify_security`, never for the actual comparison (which compares raw
/// SID/ACL bytes instead).
fn sddl_string(sd: PSECURITY_DESCRIPTOR) -> Option<String> {
    let mut wide_sddl: PWSTR = ptr::null_mut();
    let mut len: u32 = 0;
    // SAFETY: `sd` is a live, valid security descriptor owned by the caller's `SecurityCapture`;
    // the buffer this allocates is freed via `LocalFree` below.
    let ok = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut wide_sddl,
            &mut len,
        )
    };
    if ok == 0 || wide_sddl.is_null() {
        return None;
    }
    // SAFETY: `wide_sddl` is valid for `len` UTF-16 code units per the call's own contract.
    let s =
        unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(wide_sddl, len as usize)) };
    // SAFETY: `wide_sddl` was allocated by the conversion call above via LocalAlloc, which
    // documents `LocalFree` as the correct release.
    unsafe {
        LocalFree(wide_sddl.cast());
    }
    Some(s.trim_end_matches('\0').to_string())
}

/// Cheap post-swap sanity check. The replacement file already carried `expected`'s owner/DACL
/// before the swap (`apply_security`), so a fresh read of `path` should always match — this is not
/// the mechanism that makes the result correct (that's `apply_security`, pre-swap), only a check
/// that nothing else altered it during the swap itself. Never returns `Ok` on a mismatch.
fn verify_security(path: &Path, expected: &SecurityCapture) -> Result<(), Error> {
    let actual = capture_security(path).ok_or_else(|| {
        Error::WindowsApi(format!(
            "could not re-read {} after replacing it, to verify its owner/permissions",
            path.display()
        ))
    })?;

    // SAFETY: both SIDs come from a successful GetNamedSecurityInfoW and are still valid (neither
    // capture has been dropped yet).
    let owner_matches = unsafe { EqualSid(expected.owner, actual.owner) } != 0;
    // SAFETY: both ACL pointers come from a successful GetNamedSecurityInfoW; `AclSize` is the
    // documented total byte length of the ACL structure starting at that same pointer.
    let dacl_matches = unsafe {
        let a = std::slice::from_raw_parts(
            expected.dacl.cast::<u8>(),
            usize::from((*expected.dacl).AclSize),
        );
        let b = std::slice::from_raw_parts(
            actual.dacl.cast::<u8>(),
            usize::from((*actual.dacl).AclSize),
        );
        a == b
    };
    if owner_matches && dacl_matches {
        return Ok(());
    }

    let expected_sddl = sddl_string(expected.sd).unwrap_or_else(|| "<unavailable>".to_string());
    log::error!(
        "{} was replaced but its owner/permissions no longer match what was applied to the \
         replacement just before the swap; expected SDDL: {expected_sddl}",
        path.display()
    );
    Err(Error::WindowsApi(format!(
        "{} was replaced but its owner/permissions do not match what was applied before the swap \
         — see the error log for the expected SDDL",
        path.display()
    )))
}

/// Replaces the hosts file's entire content atomically, with its owner/DACL already correct by
/// construction — never fixed up after the fact.
///
/// Critical section: `hosts` is a real system file (`C:\Windows\System32\drivers\etc\hosts`) that
/// networking depends on, so a crash, full disk, or an AV product locking the file mid-write must
/// never leave it truncated, and a permission mistake here must never be silent (code review Fix
/// 2). The sequence: capture the original owner/DACL *first* — failing closed if that fails, never
/// touching the real file; write the new content to a temp file; apply the captured security to
/// *that temp file*; only then swap it in via `ReplaceFileW`. Every step before the swap can fail
/// without consequence (the real file is untouched until the swap), and the swap itself only ever
/// installs an already-correct file, so there is no window where the real file exists with the
/// wrong owner for `ReplaceFileW`'s retries (or anything else) to observe. A plain
/// temp-file-then-rename (`MoveFileExW`/`fs::rename`, what this codebase's own snapshot writer
/// uses in `backup/storage.rs`) would be worse still — it hands the *source* file's ACL to the
/// destination entirely, not merely the owner — which is exactly why this primitive needs its own
/// path instead of reusing `tempfile::NamedTempFile::persist`.
fn replace_hosts_file_atomically(new_content: &str) -> Result<(), Error> {
    let hosts_path = get_hosts_path();

    // A machine with no hosts file yet has nothing to preserve or lose: there is no pre-existing
    // security descriptor (or content) at risk, so this one case is intentionally a plain,
    // non-atomic create rather than the capture/apply/swap sequence below — nothing to lose means
    // nothing for atomicity to protect.
    if !hosts_path.exists() {
        return fs::write(&hosts_path, new_content.as_bytes())
            .map_err(|e| Error::WindowsApi(format!("Failed to create hosts file: {}", e)));
    }

    // Capture BEFORE writing anything, and fail closed: an irreversible swap whose permission side
    // effect we could not correct is worse than not swapping at all.
    let security = capture_security(&hosts_path).ok_or_else(|| {
        Error::WindowsApi(
            "could not capture the hosts file's current owner/permissions; refusing to replace \
             it without a way to preserve them"
                .to_string(),
        )
    })?;

    let dir = hosts_path
        .parent()
        .ok_or_else(|| Error::WindowsApi("hosts file path has no parent directory".to_string()))?;
    // Same directory as the target: ReplaceFileW (like any atomic replace) requires same-volume
    // source/destination.
    let tmp_path = dir.join(unique_tmp_name());

    if let Err(e) = fs::write(&tmp_path, new_content.as_bytes()) {
        let _ = fs::remove_file(&tmp_path); // best-effort: don't leave our own temp file behind
        return Err(Error::WindowsApi(format!(
            "Failed to write temp hosts file: {}",
            e
        )));
    }

    // Apply the captured security to the temp file *before* the swap (see the function doc
    // comment) — a failure here costs nothing, so clean up and bail without ever touching the
    // real file.
    try_enable_restore_privilege();
    if let Err(e) = apply_security(&tmp_path, &security) {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    let replaced = wide_path(&hosts_path);
    let replacement = wide_path(&tmp_path);

    let mut last_err = 0u32;
    for attempt in 1..=REPLACE_RETRY_ATTEMPTS {
        // SAFETY: `replaced`/`replacement` are valid, NUL-terminated wide strings kept alive for
        // the whole call. No backup file is requested and the exclude/reserved parameters are
        // null, per MSDN's documented contract for `ReplaceFileW`.
        let ok = unsafe {
            ReplaceFileW(
                replaced.as_ptr(),
                replacement.as_ptr(),
                ptr::null(),
                REPLACEFILE_WRITE_THROUGH,
                ptr::null(),
                ptr::null(),
            )
        };
        if ok != 0 {
            // The replacement already carried the correct security (`apply_security`, above) --
            // this is a cheap sanity check, never the mechanism that makes it correct.
            return verify_security(&hosts_path, &security);
        }

        // SAFETY: GetLastError is always safe to call; it just reads thread-local state.
        last_err = unsafe { GetLastError() };
        let transient = matches!(last_err, ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION);
        if !transient || attempt == REPLACE_RETRY_ATTEMPTS {
            break;
        }
        log::debug!(
            "ReplaceFile on the hosts file hit a transient sharing/lock violation (attempt {attempt}/{REPLACE_RETRY_ATTEMPTS}); retrying"
        );
        std::thread::sleep(REPLACE_RETRY_DELAY);
    }

    let _ = fs::remove_file(&tmp_path); // best-effort: don't leave our own temp file behind
    Err(Error::WindowsApi(format!(
        "ReplaceFile failed while writing the hosts file after {REPLACE_RETRY_ATTEMPTS} attempt(s) (error {last_err})"
    )))
}

/// Represents a single entry in the hosts file
#[derive(Debug, Clone)]
pub struct HostsEntry {
    pub ip: String,
    pub domain: String,
}

/// Parse hosts-file text into entries.
///
/// Split out from `read_hosts_file` so the parsing rules can be tested without
/// touching `C:\Windows\System32\drivers\etc\hosts`, which needs admin and is
/// machine-global.
pub fn parse_hosts_lines(content: &str) -> Vec<HostsEntry> {
    let mut entries = Vec::new();

    // A UTF-8 BOM is glued to the first line; strip it so the first IP isn't corrupted.
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments (including MagicX markers)
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // A line maps one IP to one or more hostnames: `IP host1 [host2 ...] [# comment]`.
        // Strip any inline comment, then emit an entry per hostname so `entry_exists` sees every
        // one (not just the first) and removal can target a single hostname.
        let code = match trimmed.find('#') {
            Some(pos) => trimmed[..pos].trim_end(),
            None => trimmed,
        };
        let mut tokens = code.split_whitespace();
        if let Some(ip) = tokens.next() {
            for domain in tokens {
                entries.push(HostsEntry {
                    ip: ip.to_string(),
                    domain: domain.to_string(),
                });
            }
        }
    }

    entries
}

/// Read and parse the hosts file
pub fn read_hosts_file() -> Result<Vec<HostsEntry>, Error> {
    let hosts_path = get_hosts_path();

    if !hosts_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;

    Ok(parse_hosts_lines(&content))
}

/// Check if a specific hosts entry exists
pub fn entry_exists(ip: &str, domain: &str) -> Result<bool, Error> {
    let entries = read_hosts_file()?;
    Ok(entries
        .iter()
        .any(|e| e.ip == ip && e.domain.eq_ignore_ascii_case(domain)))
}

/// Computes the new hosts-file text after appending `(ip, domain)` (with an optional comment),
/// tagged with the MagicX marker.
///
/// Pure and independent of the real file — mirrors `remove_entry_from_hosts` — so the CRLF/marker/
/// newline-padding logic is unit-testable without touching
/// `C:\Windows\System32\drivers\etc\hosts`. Splitting this out is also what lets `add_hosts_entry`
/// go through the same atomic whole-file replace as removal, instead of a separate append-mode
/// file handle: one write path in this file to reason about instead of two (see
/// `replace_hosts_file_atomically`).
fn add_entry_to_hosts(content: &str, ip: &str, domain: &str, comment: Option<&str>) -> String {
    let entry_line = if let Some(c) = comment {
        format!("{}\t{}\t# {}", ip, domain, c)
    } else {
        format!("{}\t{}", ip, domain)
    };

    // Ensure we start on a new line. Windows hosts files are CRLF, so pad with CRLF (not LF).
    let needs_newline = !content.ends_with('\n') && !content.is_empty();
    format!(
        "{}{}{}\r\n{}\r\n",
        content,
        if needs_newline { "\r\n" } else { "" },
        MAGICX_MARKER,
        entry_line
    )
}

/// Add an entry to the hosts file
pub fn add_hosts_entry(ip: &str, domain: &str, comment: Option<&str>) -> Result<(), Error> {
    // First check if it already exists
    if entry_exists(ip, domain)? {
        log::debug!("Hosts entry already exists: {} -> {}", domain, ip);
        return Ok(());
    }

    let hosts_path = get_hosts_path();
    let existing_content = if hosts_path.exists() {
        fs::read_to_string(&hosts_path)
            .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?
    } else {
        String::new()
    };

    let new_content = add_entry_to_hosts(&existing_content, ip, domain, comment);
    replace_hosts_file_atomically(&new_content)?;

    log::info!("Added hosts entry: {} -> {}", domain, ip);
    Ok(())
}

/// What removing `(ip, domain)` does to a single hosts line.
enum LineEdit {
    /// Not a matching entry line — keep it verbatim.
    Unchanged,
    /// The removed hostname was the only one on the line — drop the whole line.
    Dropped,
    /// Other hostnames remain — the line rewritten without the target hostname.
    Rewritten(String),
}

/// Decide what to do with one hosts line when removing `(ip, domain)`.
///
/// Matches only on the IP plus the hostname list (case-insensitive), preserves any inline comment,
/// and — crucially — keeps the line's other hostnames instead of deleting a shared line wholesale.
fn edit_hosts_line(raw: &str, ip: &str, domain: &str) -> LineEdit {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return LineEdit::Unchanged;
    }

    let (code, comment) = match trimmed.find('#') {
        Some(pos) => (trimmed[..pos].trim_end(), Some(trimmed[pos..].to_string())),
        None => (trimmed, None),
    };

    let mut tokens = code.split_whitespace();
    let line_ip = match tokens.next() {
        Some(t) => t,
        None => return LineEdit::Unchanged,
    };
    if line_ip != ip {
        return LineEdit::Unchanged;
    }

    let hosts: Vec<&str> = tokens.collect();
    if !hosts.iter().any(|h| h.eq_ignore_ascii_case(domain)) {
        return LineEdit::Unchanged; // this IP line doesn't carry the target hostname
    }

    let remaining: Vec<&str> = hosts
        .into_iter()
        .filter(|h| !h.eq_ignore_ascii_case(domain))
        .collect();
    if remaining.is_empty() {
        return LineEdit::Dropped;
    }

    let mut rebuilt = format!("{}\t{}", line_ip, remaining.join(" "));
    if let Some(c) = comment {
        rebuilt.push(' ');
        rebuilt.push_str(&c);
    }
    LineEdit::Rewritten(rebuilt)
}

/// Remove `(ip, domain)` from hosts-file text, returning new text with CRLF line endings.
///
/// A line mapping the IP to several hostnames keeps its other hostnames; only when the removed
/// hostname was the last one is the whole line dropped, along with a MagicX marker directly above
/// it. Foreign comments and blank lines are preserved. A leading UTF-8 BOM is stripped.
pub fn remove_entry_from_hosts(content: &str, ip: &str, domain: &str) -> String {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());

    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];

        // A MagicX marker is dropped only when the entry line right below it is fully removed.
        if raw.trim().starts_with(MAGICX_MARKER) {
            match lines
                .get(i + 1)
                .map(|next| edit_hosts_line(next, ip, domain))
            {
                Some(LineEdit::Dropped) => {
                    i += 2; // drop marker + entry line
                    continue;
                }
                Some(LineEdit::Rewritten(new_line)) => {
                    out.push(raw.to_string()); // keep marker
                    out.push(new_line);
                    i += 2;
                    continue;
                }
                _ => {
                    out.push(raw.to_string());
                    i += 1;
                    continue;
                }
            }
        }

        match edit_hosts_line(raw, ip, domain) {
            LineEdit::Dropped => {}
            LineEdit::Rewritten(new_line) => out.push(new_line),
            LineEdit::Unchanged => out.push(raw.to_string()),
        }
        i += 1;
    }

    let mut result = out.join("\r\n");
    if !result.is_empty() {
        result.push_str("\r\n");
    }
    result
}

/// Remove an entry from the hosts file
pub fn remove_hosts_entry(ip: &str, domain: &str) -> Result<(), Error> {
    let hosts_path = get_hosts_path();

    if !hosts_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;

    let new_content = remove_entry_from_hosts(&content, ip, domain);
    log::info!("Removing hosts entry: {} -> {}", domain, ip);

    replace_hosts_file_atomically(&new_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The parser must not care about line endings. Windows hosts files are CRLF,
    /// but a file touched by another tool can end up mixed.
    #[test]
    fn parses_crlf_lf_and_mixed_line_endings_identically() {
        let lf = "0.0.0.0 a.example\n127.0.0.1 b.example\n";
        let crlf = "0.0.0.0 a.example\r\n127.0.0.1 b.example\r\n";
        let mixed = "0.0.0.0 a.example\r\n127.0.0.1 b.example\n";

        for (label, text) in [("lf", lf), ("crlf", crlf), ("mixed", mixed)] {
            let e = parse_hosts_lines(text);
            assert_eq!(e.len(), 2, "{label}: wrong entry count");
            assert_eq!(e[0].ip, "0.0.0.0", "{label}");
            assert_eq!(e[0].domain, "a.example", "{label}");
            assert_eq!(e[1].domain, "b.example", "{label}");
        }
    }

    #[test]
    fn ignores_comments_blank_lines_and_our_own_marker() {
        let text = concat!(
            "# Copyright (c) 1993-2009 Microsoft Corp.\r\n",
            "\r\n",
            "       \r\n",
            "# 102.54.94.97     rhino.acme.com          # source server\r\n",
            "# MagicX Toolbox\r\n",
            "0.0.0.0 telemetry.example\r\n"
        );
        let e = parse_hosts_lines(text);
        assert_eq!(e.len(), 1, "only the real entry should parse");
        assert_eq!(e[0].domain, "telemetry.example");
    }

    #[test]
    fn strips_inline_comments_from_the_domain() {
        let e = parse_hosts_lines("0.0.0.0 telemetry.example # blocked by us\r\n");
        assert_eq!(e.len(), 1);
        assert_eq!(
            e[0].domain, "telemetry.example",
            "the trailing comment leaked into the domain"
        );
    }

    #[test]
    fn tolerates_tabs_and_repeated_spaces_as_separators() {
        let e = parse_hosts_lines("0.0.0.0\t\ttelemetry.example\r\n127.0.0.1    local.example\r\n");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].domain, "telemetry.example");
        assert_eq!(e[1].domain, "local.example");
    }

    /// A hosts line may map one IP to several hostnames; every one is now a distinct entry, so
    /// `entry_exists` sees them all and removal can target a single hostname.
    #[test]
    fn every_hostname_on_a_multi_host_line_is_parsed() {
        let e = parse_hosts_lines("127.0.0.1 alpha.example beta.example gamma.example\r\n");
        assert_eq!(e.len(), 3);
        assert_eq!(e[0].domain, "alpha.example");
        assert_eq!(e[1].domain, "beta.example");
        assert_eq!(e[2].domain, "gamma.example");
        assert!(e.iter().all(|x| x.ip == "127.0.0.1"));
    }

    /// A leading UTF-8 BOM is stripped, so the first entry's IP is intact.
    #[test]
    fn a_utf8_bom_is_stripped_from_the_first_entry() {
        let e = parse_hosts_lines("\u{feff}0.0.0.0 telemetry.example\r\n127.0.0.1 ok.example\r\n");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].ip, "0.0.0.0", "the BOM must not corrupt the first IP");
        assert_eq!(e[0].domain, "telemetry.example");
        assert_eq!(e[1].ip, "127.0.0.1");
    }

    #[test]
    fn lines_without_a_hostname_are_skipped() {
        let e = parse_hosts_lines("0.0.0.0\r\n\r\n0.0.0.0   \r\n");
        assert!(e.is_empty(), "an IP with no hostname is not an entry");
    }

    /// Code review Fix 2 item 3: process id alone collides across threads in one process; the
    /// counter must make every call distinct regardless.
    #[test]
    fn unique_tmp_name_differs_across_calls() {
        let a = unique_tmp_name();
        let b = unique_tmp_name();
        assert_ne!(
            a, b,
            "concurrent callers must not collide on the same temp filename"
        );
    }

    #[test]
    fn add_entry_appends_with_marker_and_crlf() {
        let out = add_entry_to_hosts("", "0.0.0.0", "telemetry.example", None);
        assert_eq!(out, "# MagicX Toolbox\r\n0.0.0.0\ttelemetry.example\r\n");
    }

    #[test]
    fn add_entry_includes_comment_when_given() {
        let out = add_entry_to_hosts("", "0.0.0.0", "telemetry.example", Some("blocked by us"));
        assert_eq!(
            out,
            "# MagicX Toolbox\r\n0.0.0.0\ttelemetry.example\t# blocked by us\r\n"
        );
    }

    #[test]
    fn add_entry_pads_a_newline_before_appending_to_content_missing_one() {
        let out = add_entry_to_hosts("127.0.0.1 localhost", "0.0.0.0", "x.example", None);
        assert_eq!(
            out,
            "127.0.0.1 localhost\r\n# MagicX Toolbox\r\n0.0.0.0\tx.example\r\n"
        );
    }

    #[test]
    fn add_entry_does_not_double_pad_content_that_already_ends_in_a_newline() {
        let out = add_entry_to_hosts("127.0.0.1 localhost\r\n", "0.0.0.0", "x.example", None);
        assert_eq!(
            out,
            "127.0.0.1 localhost\r\n# MagicX Toolbox\r\n0.0.0.0\tx.example\r\n"
        );
    }

    #[test]
    fn add_entry_on_empty_content_has_no_leading_newline() {
        let out = add_entry_to_hosts("", "0.0.0.0", "x.example", None);
        assert!(out.starts_with("# MagicX Toolbox\r\n"), "{out:?}");
    }

    #[test]
    fn removing_one_hostname_keeps_the_others_on_the_line() {
        let out = remove_entry_from_hosts(
            "127.0.0.1 alpha.example beta.example gamma.example\r\n",
            "127.0.0.1",
            "beta.example",
        );
        let entries = parse_hosts_lines(&out);
        assert_eq!(entries.len(), 2, "only beta should be removed");
        assert!(entries.iter().any(|e| e.domain == "alpha.example"));
        assert!(entries.iter().any(|e| e.domain == "gamma.example"));
        assert!(!entries.iter().any(|e| e.domain == "beta.example"));
    }

    #[test]
    fn removing_the_last_hostname_drops_the_line_and_its_marker() {
        let content = concat!(
            "# MagicX Toolbox\r\n",
            "0.0.0.0\ttelemetry.example\r\n",
            "127.0.0.1 keep.example\r\n"
        );
        let out = remove_entry_from_hosts(content, "0.0.0.0", "telemetry.example");
        assert!(
            !out.contains("MagicX Toolbox"),
            "the marker should be dropped with its only entry"
        );
        assert!(!out.contains("telemetry.example"));
        assert!(out.contains("keep.example"));
    }

    #[test]
    fn removal_normalizes_to_crlf() {
        // LF input (e.g. mangled by a previous buggy write) must come back out as CRLF.
        let out = remove_entry_from_hosts(
            "0.0.0.0 a.example\n0.0.0.0 b.example\n",
            "0.0.0.0",
            "a.example",
        );
        assert!(out.contains("b.example"));
        assert!(out.matches("\r\n").count() > 0);
        assert_eq!(
            out.matches('\n').count(),
            out.matches("\r\n").count(),
            "every LF must be part of a CRLF"
        );
    }
}
