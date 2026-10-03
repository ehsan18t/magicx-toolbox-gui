//! Enables or disables one privilege on this process's token.

use crate::error::{win32, Error};
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// `AdjustTokenPrivileges` reports a privilege it could not grant through this, not a FALSE return.
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;

/// `Ok(true)` when the call changed the privilege's state. A privilege the token does not hold
/// fails as `PRIVILEGE_NOT_HELD` on enable.
pub(crate) fn adjust(name: &str, enable: bool) -> Result<bool, Error> {
    // SAFETY: standard OpenProcessToken/LookupPrivilegeValueW/AdjustTokenPrivileges sequence; the
    // token handle is closed on every path after the last-error is read, `tp` is fully initialized
    // and `previous` is sized.
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(Error::win32("OpenProcessToken failed", GetLastError()));
        }
        let result = adjust_token(token, name, enable);
        CloseHandle(token);
        result
    }
}

/// # Safety
/// `token` must be an open token handle with adjust and query access.
unsafe fn adjust_token(token: HANDLE, name: &str, enable: bool) -> Result<bool, Error> {
    let privilege_name = super::wide(name);
    let mut luid: LUID = std::mem::zeroed();
    if LookupPrivilegeValueW(ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
        return Err(Error::win32(
            format!("LookupPrivilegeValue({name}) failed"),
            GetLastError(),
        ));
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
        return Err(Error::win32(
            format!("AdjustTokenPrivileges({name}) failed"),
            GetLastError(),
        ));
    }
    if enable && GetLastError() == ERROR_NOT_ALL_ASSIGNED {
        return Err(Error::win32(
            format!("{name} is not held by this process"),
            win32::PRIVILEGE_NOT_HELD,
        ));
    }
    log::trace!("{name} enabled = {enable}");
    // PreviousState lists only the privileges whose state the call actually changed.
    Ok(previous.PrivilegeCount > 0)
}
