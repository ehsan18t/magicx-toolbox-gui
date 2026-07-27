//! Win32 helpers shared by the SYSTEM and TrustedInstaller spawn paths.
//!
//! Every failure path here captures `GetLastError` BEFORE closing any handle: `CloseHandle`
//! overwrites the thread's last-error, so reading it afterwards reports the close, not the call
//! that actually failed.

use crate::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, DuplicateTokenEx, LookupPrivilegeValueW, SecurityImpersonation,
    TokenPrimary, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_ALL_ACCESS, TOKEN_DUPLICATE, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, STARTUPINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const STARTF_USESHOWWINDOW: u32 = 0x00000001;
/// `AdjustTokenPrivileges` reports a privilege it could not grant through this, not a FALSE return.
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;

/// How long to wait on a spawned elevated child before treating it as hung.
pub(super) const ELEVATED_PROCESS_TIMEOUT_MS: u32 = 30_000;

/// Convert a Rust string to a null-terminated wide string.
pub(super) fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// A hidden-window `STARTUPINFOW`, the shape both spawn paths want.
pub(super) fn hidden_startup_info() -> STARTUPINFOW {
    // SAFETY: STARTUPINFOW is a plain C struct of integers, pointers, and a handle triple; an
    // all-zero value is the documented "no overrides" state.
    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE as u16;
    si
}

/// A zeroed `PROCESS_INFORMATION` for `CreateProcess*` to fill in.
pub(super) fn empty_process_info() -> PROCESS_INFORMATION {
    // SAFETY: pure out-param; every field is written by a successful CreateProcess*.
    unsafe { std::mem::zeroed() }
}

/// Enable `SeDebugPrivilege` for the current process. Required to open winlogon (for its SYSTEM
/// token) and the TrustedInstaller service process (to spoof it as a parent).
pub(super) fn enable_debug_privilege() -> Result<(), Error> {
    // SAFETY: standard OpenProcessToken/LookupPrivilegeValueW/AdjustTokenPrivileges sequence; the
    // token handle is closed on every path and `tp` is fully initialized before use.
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(win_err("OpenProcessToken"));
        }

        let privilege_name = to_wide_string("SeDebugPrivilege");
        let mut luid: LUID = std::mem::zeroed();
        if LookupPrivilegeValueW(ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
            return Err(close_then(token, win_err("LookupPrivilegeValue")));
        }

        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0] = LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        };

        if AdjustTokenPrivileges(token, FALSE, &tp, 0, ptr::null_mut(), ptr::null_mut()) == FALSE {
            return Err(close_then(token, win_err("AdjustTokenPrivileges")));
        }

        // AdjustTokenPrivileges succeeds even when it granted nothing; only the last-error says so.
        let partial = GetLastError() == ERROR_NOT_ALL_ASSIGNED;
        CloseHandle(token);
        if partial {
            return Err(Error::WindowsApi(
                "SeDebugPrivilege not available, admin rights required".to_string(),
            ));
        }

        log::trace!("Enabled SeDebugPrivilege");
        Ok(())
    }
}

/// Find a process ID by executable name (case-insensitive).
pub(super) fn find_process_by_name(target_name: &str) -> Result<u32, Error> {
    // SAFETY: the snapshot handle is closed on every path, and PROCESSENTRY32W is only read after
    // a successful Process32FirstW/NextW filled it in.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(win_err("CreateToolhelp32Snapshot"));
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) == FALSE {
            return Err(close_then(snapshot, win_err("Process32FirstW")));
        }

        loop {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..len]).eq_ignore_ascii_case(target_name) {
                let pid = entry.th32ProcessID;
                CloseHandle(snapshot);
                log::trace!("Found {} with PID {}", target_name, pid);
                return Ok(pid);
            }
            if Process32NextW(snapshot, &mut entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot);
        Err(Error::WindowsApi(format!(
            "Process not found: {}",
            target_name
        )))
    }
}

/// Duplicate a process's token as a primary token the caller owns and must close.
pub(super) fn get_process_token(pid: u32) -> Result<HANDLE, Error> {
    // SAFETY: the process and source-token handles are closed on every path; `dup_token` is
    // written by a successful DuplicateTokenEx and handed to the caller.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
        if process.is_null() {
            return Err(win_err(&format!("OpenProcess for PID {pid}")));
        }

        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut token) == FALSE {
            return Err(close_then(process, win_err("OpenProcessToken")));
        }

        let mut dup_token: HANDLE = ptr::null_mut();
        let ok = DuplicateTokenEx(
            token,
            TOKEN_ALL_ACCESS,
            ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut dup_token,
        );
        let err = (ok == FALSE).then(|| win_err("DuplicateTokenEx"));
        CloseHandle(token);
        CloseHandle(process);

        match err {
            Some(e) => Err(e),
            None => {
                log::trace!("Got duplicated token from PID {}", pid);
                Ok(dup_token)
            }
        }
    }
}

/// Wrap the current thread's last Win32 error. Call this BEFORE any `CloseHandle`.
fn win_err(what: &str) -> Error {
    // SAFETY: GetLastError only reads thread-local state.
    Error::WindowsApi(format!("{what} failed: {}", unsafe { GetLastError() }))
}

/// Close `handle` and return `err` unchanged, so the caller cannot accidentally re-read the
/// last-error after the close has already overwritten it.
///
/// # Safety
/// `handle` must be a valid, owned handle that is not used again.
unsafe fn close_then(handle: HANDLE, err: Error) -> Error {
    CloseHandle(handle);
    err
}

/// Wait for a spawned elevated process to finish, reap it, and return its real exit code.
///
/// The three wait outcomes stay distinct, so a wait failure can never masquerade as a completed
/// process with exit code 0, which the broker would then read as success: the process exited (its
/// exit code, with the `GetExitCodeProcess` BOOL checked rather than assumed), it hung (terminate
/// and report a timeout), or the wait itself failed (report it, never `Ok(0)`).
///
/// # Safety
/// `pi` must hold valid process and thread handles from a successful `CreateProcess*`. Both handles
/// are closed on every return path.
pub(super) unsafe fn wait_and_reap(pi: &PROCESS_INFORMATION, label: &str) -> Result<i32, Error> {
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, TerminateProcess, WaitForSingleObject,
    };
    const WAIT_OBJECT_0: u32 = 0x0000_0000;
    const WAIT_TIMEOUT: u32 = 0x0000_0102;

    let wait_result = WaitForSingleObject(pi.hProcess, ELEVATED_PROCESS_TIMEOUT_MS);

    // Every branch reads its error before reaping, then reaps exactly once.
    let outcome = if wait_result == WAIT_TIMEOUT {
        log::warn!("{label} timed out after {ELEVATED_PROCESS_TIMEOUT_MS}ms");
        TerminateProcess(pi.hProcess, 1);
        Err(Error::ServiceControl(format!(
            "{label} timed out after {ELEVATED_PROCESS_TIMEOUT_MS}ms"
        )))
    } else if wait_result != WAIT_OBJECT_0 {
        // WAIT_FAILED (0xFFFF_FFFF) or any unexpected value: do NOT fall through to a bogus Ok(0).
        Err(Error::ServiceControl(format!(
            "{label} wait failed (result {wait_result:#x}): {}",
            GetLastError()
        )))
    } else {
        let mut exit_code: u32 = 0;
        if GetExitCodeProcess(pi.hProcess, &mut exit_code) == FALSE {
            Err(Error::ServiceControl(format!(
                "{label} exit-code query failed: {}",
                GetLastError()
            )))
        } else {
            log::debug!("{label} completed with exit code: {exit_code}");
            Ok(exit_code as i32)
        }
    };

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    outcome
}
