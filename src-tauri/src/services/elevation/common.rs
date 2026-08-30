//! Win32 helpers for the TrustedInstaller spawn path.
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
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, TerminateProcess, WaitForSingleObject,
    PROCESS_INFORMATION, STARTUPINFOW,
};
use windows_sys::Win32::System::WindowsProgramming::QueryUnbiasedInterruptTime;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

const STARTF_USESHOWWINDOW: u32 = 0x00000001;
/// `AdjustTokenPrivileges` reports a privilege it could not grant through this, not a FALSE return.
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;

/// How long to wait on a spawned elevated child before treating it as hung. Charged in awake time
/// only (see [`wait_awake`]), never wall-clock.
pub(super) const ELEVATED_PROCESS_TIMEOUT_MS: u32 = 30_000;

/// How long to wait for a child we terminated to actually die before saying so.
const TERMINATE_GRACE_MS: u32 = 5_000;

/// One blocking slice of the timeout budget. Small enough that a suspend/resume is noticed
/// promptly, large enough that the loop costs nothing on the common path.
const WAIT_SLICE_MS: u32 = 1_000;

const WAIT_OBJECT_0: u32 = 0x0000_0000;
const WAIT_TIMEOUT: u32 = 0x0000_0102;

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

/// Unbiased interrupt time in milliseconds: time the machine has spent awake, excluding sleep and
/// hibernation. Returns `None` if the clock is unavailable, which callers treat as "no better
/// information than the biased clock".
unsafe fn unbiased_now_ms() -> Option<u64> {
    let mut t: u64 = 0;
    // QueryUnbiasedInterruptTime reports 100-nanosecond intervals.
    (QueryUnbiasedInterruptTime(&mut t) != FALSE).then_some(t / 10_000)
}

/// Wait on `h`, charging only awake time against `budget_ms`.
///
/// `WaitForSingleObject`'s own timeout is biased: suspended time counts against it. Closing a laptop
/// lid for thirty seconds mid-apply therefore reports a perfectly healthy elevated child as hung,
/// terminates it, and rolls the tweak back. Waiting in slices and charging the budget from the
/// unbiased clock makes the timeout mean "the child ran for this long without finishing", which is
/// the only thing it was ever supposed to mean.
///
/// Returns a raw `WaitForSingleObject` result, so callers keep the same three-outcome shape.
///
/// # Safety
/// `h` must be a valid process handle.
unsafe fn wait_awake(h: HANDLE, budget_ms: u32) -> u32 {
    let Some(start) = unbiased_now_ms() else {
        // No unbiased clock: one biased wait is strictly better than looping forever.
        return WaitForSingleObject(h, budget_ms);
    };
    loop {
        let result = WaitForSingleObject(h, WAIT_SLICE_MS);
        if result != WAIT_TIMEOUT {
            return result;
        }
        // A failed clock read is charged as a full budget rather than spinning: conservative, and
        // it guarantees the loop terminates.
        let awake_ms =
            unbiased_now_ms().map_or(u64::from(budget_ms), |now| now.saturating_sub(start));
        if awake_ms >= u64::from(budget_ms) {
            return WAIT_TIMEOUT;
        }
    }
}

/// Terminate a hung child and confirm it is actually gone, returning the error to report.
///
/// `TerminateProcess` only *initiates* termination. Returning before the process has died lets the
/// engine roll back from the snapshot while an elevated child is still writing the very keys being
/// restored, and then delete that snapshot because the restore verified clean (ADR-0002). So the
/// two cases are reported differently: a confirmed kill is an ordinary timeout, an unconfirmed one
/// says plainly that the machine may still be changing under us.
///
/// # Safety
/// `h` must be a valid process handle opened with `PROCESS_TERMINATE`.
unsafe fn terminate_and_confirm(h: HANDLE, label: &str) -> Error {
    let terminate_err = (TerminateProcess(h, 1) == FALSE).then(|| GetLastError());
    let confirmed_dead =
        terminate_err.is_none() && WaitForSingleObject(h, TERMINATE_GRACE_MS) == WAIT_OBJECT_0;

    if confirmed_dead {
        return Error::ServiceControl(format!(
            "{label} timed out after {ELEVATED_PROCESS_TIMEOUT_MS}ms of running time and was terminated"
        ));
    }
    let detail = match terminate_err {
        Some(code) => format!("TerminateProcess failed: {code}"),
        None => format!("still running {TERMINATE_GRACE_MS}ms after being terminated"),
    };
    log::error!("{label}: could not confirm the elevated child died ({detail})");
    Error::ServiceControl(format!(
        "{label} timed out after {ELEVATED_PROCESS_TIMEOUT_MS}ms of running time and could not be confirmed dead ({detail}); it may still be modifying the system"
    ))
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
    let wait_result = wait_awake(pi.hProcess, ELEVATED_PROCESS_TIMEOUT_MS);

    // Every branch reads its error before reaping, then reaps exactly once.
    let outcome = if wait_result == WAIT_TIMEOUT {
        log::warn!("{label} timed out after {ELEVATED_PROCESS_TIMEOUT_MS}ms of running time");
        Err(terminate_and_confirm(pi.hProcess, label))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use std::process::{Command, Stdio};

    #[test]
    fn the_unbiased_clock_is_available_and_never_goes_backwards() {
        // SAFETY: pure out-param read of a kernel clock.
        let a = unsafe { unbiased_now_ms() }.expect("QueryUnbiasedInterruptTime must be available");
        std::thread::sleep(std::time::Duration::from_millis(25));
        // SAFETY: as above.
        let b = unsafe { unbiased_now_ms() }.expect("QueryUnbiasedInterruptTime must be available");
        assert!(b >= a, "unbiased clock went backwards: {a} -> {b}");
    }

    /// The slice loop has to keep both halves of the old single-call behaviour: return as soon as
    /// the child exits, and still report `WAIT_TIMEOUT` once the budget is genuinely spent. Only
    /// the accounting changed, from wall-clock to awake time.
    #[test]
    fn wait_awake_returns_on_exit_and_still_times_out() {
        let mut quick = Command::new("cmd.exe")
            .args(["/c", "exit", "0"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn a short-lived child");
        // SAFETY: the handle is owned by `quick` and outlives the wait.
        let result = unsafe { wait_awake(quick.as_raw_handle() as HANDLE, 30_000) };
        assert_eq!(result, WAIT_OBJECT_0, "a child that exits must signal");
        let _ = quick.wait();

        let mut slow = Command::new("cmd.exe")
            .args(["/c", "ping", "-n", "6", "127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn a longer-lived child");
        // A budget below one slice still spends a whole slice, then finds the budget exhausted.
        // SAFETY: the handle is owned by `slow` and outlives the wait.
        let result = unsafe { wait_awake(slow.as_raw_handle() as HANDLE, 500) };
        assert_eq!(result, WAIT_TIMEOUT, "a child still running must time out");
        let _ = slow.kill();
        let _ = slow.wait();
    }

    /// A terminated child must be confirmed dead before the caller is told the batch is over,
    /// because the caller's next move is to roll back from the snapshot.
    #[test]
    fn terminate_and_confirm_reports_an_ordinary_timeout_once_the_child_is_gone() {
        let mut child = Command::new("cmd.exe")
            .args(["/c", "ping", "-n", "30", "127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn a long-lived child");
        // SAFETY: the handle is owned by `child` and outlives the call.
        let err = unsafe { terminate_and_confirm(child.as_raw_handle() as HANDLE, "test child") };
        let message = err.to_string();
        assert!(
            message.contains("was terminated"),
            "a confirmed kill must not warn about a live process: {message}"
        );
        assert!(
            !message.contains("may still be modifying"),
            "a confirmed kill must not warn about a live process: {message}"
        );
        let _ = child.wait();
    }
}
