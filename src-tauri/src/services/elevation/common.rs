//! Win32 helpers for the TrustedInstaller spawn path.
//!
//! Every failure path here captures `GetLastError` BEFORE closing any handle: `CloseHandle`
//! overwrites the thread's last-error, so reading it afterwards reports the close, not the call
//! that actually failed.

use super::broker::AcquireReason;
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
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

const STARTF_USESHOWWINDOW: u32 = 0x00000001;
/// `AdjustTokenPrivileges` reports a privilege it could not grant through this, not a FALSE return.
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;

/// How long to wait on a spawned elevated child before treating it as hung.
pub(super) const ELEVATED_PROCESS_TIMEOUT_MS: u32 = 30_000;

/// How long to wait for a child we terminated to actually die before saying so.
const TERMINATE_GRACE_MS: u32 = 5_000;

const WAIT_OBJECT_0: u32 = 0x0000_0000;
const WAIT_TIMEOUT: u32 = 0x0000_0102;

/// Why a spawn returned no exit code, split on whether the child process ever existed.
#[derive(Debug)]
pub(super) enum SpawnError {
    /// Failed before `CreateProcessW` created a child: nothing ran.
    NoChild(AcquireReason, Error),
    /// A child was created, so ops may have run.
    ChildRan(Error),
}

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

/// (holders, enabled by the first holder): concurrent TI spawns share one enable.
static DEBUG_PRIVILEGE: std::sync::Mutex<(usize, bool)> = std::sync::Mutex::new((0, false));

/// `SeDebugPrivilege`, enabled while held; required to open the TrustedInstaller process. Left
/// enabled, every later child (action scripts, installers) inherits it.
pub(super) struct DebugPrivilege(());

impl DebugPrivilege {
    pub(super) fn enable() -> Result<Self, SpawnError> {
        let mut state = DEBUG_PRIVILEGE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.0 == 0 {
            state.1 = adjust_debug_privilege(true)?;
        }
        state.0 += 1;
        Ok(Self(()))
    }
}

impl Drop for DebugPrivilege {
    fn drop(&mut self) {
        let mut state = DEBUG_PRIVILEGE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.0 -= 1;
        if state.0 == 0 && std::mem::take(&mut state.1) {
            if let Err(SpawnError::NoChild(_, e) | SpawnError::ChildRan(e)) =
                adjust_debug_privilege(false)
            {
                log::warn!("could not disable SeDebugPrivilege again: {e}");
            }
        }
    }
}

/// Sets `SeDebugPrivilege` on this process's token; `Ok(true)` when the call changed its state.
#[cfg(feature = "test-build")]
pub(crate) fn set_debug_privilege(enable: bool) -> Result<bool, Error> {
    adjust_debug_privilege(enable)
        .map_err(|(SpawnError::NoChild(_, e) | SpawnError::ChildRan(e))| e)
}

fn adjust_debug_privilege(enable: bool) -> Result<bool, SpawnError> {
    // SAFETY: standard OpenProcessToken/LookupPrivilegeValueW/AdjustTokenPrivileges sequence; the
    // token handle is closed on every path, `tp` is fully initialized and `previous` is sized.
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(spawn_failed(win_err("OpenProcessToken")));
        }

        let privilege_name = to_wide_string("SeDebugPrivilege");
        let mut luid: LUID = std::mem::zeroed();
        if LookupPrivilegeValueW(ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
            return Err(spawn_failed(close_then(
                token,
                win_err("LookupPrivilegeValue"),
            )));
        }

        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0] = LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: if enable { SE_PRIVILEGE_ENABLED } else { 0 },
        };

        let mut previous: TOKEN_PRIVILEGES = std::mem::zeroed();
        let mut previous_len = 0u32;
        if AdjustTokenPrivileges(
            token,
            FALSE,
            &tp,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            &mut previous,
            &mut previous_len,
        ) == FALSE
        {
            return Err(spawn_failed(close_then(
                token,
                win_err("AdjustTokenPrivileges"),
            )));
        }

        // AdjustTokenPrivileges succeeds even when it granted nothing; only the last-error says so.
        let partial = GetLastError() == ERROR_NOT_ALL_ASSIGNED;
        CloseHandle(token);
        if enable && partial {
            return Err(SpawnError::NoChild(
                AcquireReason::DebugPrivilegeStripped,
                Error::WindowsApi("SeDebugPrivilege not available, admin rights required".into()),
            ));
        }

        log::trace!("SeDebugPrivilege enabled = {enable}");
        // PreviousState lists only the privileges whose state the call actually changed.
        Ok(previous.PrivilegeCount > 0)
    }
}

pub(super) fn spawn_failed(e: Error) -> SpawnError {
    SpawnError::NoChild(AcquireReason::SpawnFailed, e)
}

/// Wrap the current thread's last Win32 error. Call this BEFORE any `CloseHandle`.
pub(super) fn win_err(what: &str) -> Error {
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

/// Terminate the child after `failure`, saying whether it may still run. `TerminateProcess` only
/// starts termination and fails with error 5 on an exited child, so a wait on `h` confirms death
/// either way. Safety: `h` must be a valid process handle.
unsafe fn terminate_after(h: HANDLE, label: &str, failure: String) -> Error {
    let terminate_err = (TerminateProcess(h, 1) == FALSE).then(|| GetLastError());
    let grace_ms = if terminate_err.is_some() {
        0
    } else {
        TERMINATE_GRACE_MS
    };
    let unconfirmed = |detail: String| {
        format!("could not be confirmed dead ({detail}); it may still be modifying the system")
    };
    let (outcome, dead) = match (WaitForSingleObject(h, grace_ms), terminate_err) {
        (WAIT_OBJECT_0, None) => ("was terminated".to_owned(), true),
        (WAIT_OBJECT_0, Some(_)) => ("had already exited".to_owned(), true),
        (_, Some(code)) => (
            unconfirmed(format!("TerminateProcess failed: {code}")),
            false,
        ),
        (WAIT_TIMEOUT, None) => (
            unconfirmed(format!(
                "still running {TERMINATE_GRACE_MS}ms after being terminated"
            )),
            false,
        ),
        (other, None) => (
            unconfirmed(format!(
                "confirming the kill failed (result {other:#x}): {}",
                GetLastError()
            )),
            false,
        ),
    };
    // One line for the whole kill, since the broker records only its classification.
    let message = format!("{label} {failure} and {outcome}");
    if dead {
        log::warn!("{message}");
    } else {
        log::error!("{message}");
    }
    Error::ServiceControl(message)
}

/// Wait up to `timeout_ms` for a spawned child, reap it, and return its real exit code; a failed
/// wait never falls through to `Ok(0)`, which the broker reads as success. Safety: `pi` holds valid
/// handles from a successful `CreateProcess*`; both are closed on every path.
pub(super) unsafe fn wait_and_reap(
    pi: &PROCESS_INFORMATION,
    label: &str,
    timeout_ms: u32,
) -> Result<i32, SpawnError> {
    // The timeout excludes sleep and hibernate (Windows 8+), so no awake-time accounting.
    let outcome = match WaitForSingleObject(pi.hProcess, timeout_ms) {
        WAIT_OBJECT_0 => {
            let mut exit_code: u32 = 0;
            if GetExitCodeProcess(pi.hProcess, &mut exit_code) == FALSE {
                // Signaled, so the child has exited: nothing to terminate.
                Err(Error::ServiceControl(format!(
                    "{label} exit-code query failed: {}",
                    GetLastError()
                )))
            } else {
                // The exit code is logged where it is interpreted, in `broker::run_elevated_broker`.
                Ok(exit_code as i32)
            }
        }
        WAIT_TIMEOUT => Err(terminate_after(
            pi.hProcess,
            label,
            format!("timed out after {timeout_ms}ms"),
        )),
        other => {
            let failure = format!("wait failed (result {other:#x}): {}", GetLastError());
            Err(terminate_after(pi.hProcess, label, failure))
        }
    };

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    outcome.map_err(SpawnError::ChildRan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE};

    #[test]
    fn a_refused_terminate_on_an_exited_child_reports_it_exited() {
        let root = std::env::var_os("SystemRoot").expect("SystemRoot is set");
        let cmd = std::path::Path::new(&root).join(r"System32\cmd.exe");
        let mut child = std::process::Command::new(cmd)
            .args(["/c", "exit", "0"])
            .spawn()
            .expect("spawn a test child");
        child.wait().expect("the child exits");
        // SAFETY: the handle is checked, used once, then closed.
        let message = unsafe {
            // No PROCESS_TERMINATE right: TerminateProcess fails with error 5.
            let h = OpenProcess(PROCESS_SYNCHRONIZE, FALSE, child.id());
            assert!(!h.is_null(), "open the exited child");
            let message = terminate_after(h, "test child", "timed out".into()).to_string();
            CloseHandle(h);
            message
        };
        assert!(
            message.ends_with("test child timed out and had already exited"),
            "{message}"
        );
    }

    #[test]
    #[ignore = "needs an elevated token holding SeDebugPrivilege; mutates this process's token"]
    fn the_debug_privilege_is_off_again_once_the_last_holder_drops() {
        let changed = |enable| {
            adjust_debug_privilege(enable)
                .map_err(|(SpawnError::NoChild(_, e) | SpawnError::ChildRan(e))| e)
                .unwrap()
        };
        changed(false);
        {
            let _outer = DebugPrivilege::enable().unwrap();
            drop(DebugPrivilege::enable().unwrap());
            assert!(!changed(true), "a nested holder's drop disabled it");
        }
        assert!(
            !changed(false),
            "still enabled after the last holder dropped"
        );
    }
}
