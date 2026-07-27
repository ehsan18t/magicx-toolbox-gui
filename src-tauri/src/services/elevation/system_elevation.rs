//! SYSTEM elevation: duplicate winlogon.exe's token and spawn the broker child with it.

use crate::error::Error;
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE};
use windows_sys::Win32::System::Threading::{
    CreateProcessWithTokenW, CREATE_NO_WINDOW, LOGON_WITH_PROFILE,
};

use super::common::{
    empty_process_info, enable_debug_privilege, find_process_by_name, get_process_token,
    hidden_startup_info, to_wide_string, wait_and_reap,
};

fn get_system_token() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = find_process_by_name("winlogon.exe")?;
    log::debug!("Found winlogon.exe with PID: {}", pid);
    get_process_token(pid)
}

/// Spawn `command_line` as SYSTEM and wait for it, returning its exit code. The broker's SYSTEM
/// launcher; the command line is built by `broker::run_elevated_broker`, never by a caller.
pub(super) fn spawn_as_system(command_line: &str) -> Result<i32, Error> {
    let token = get_system_token()?;
    log::debug!("Got SYSTEM token, spawning: {}", command_line);

    let mut command_wide = to_wide_string(command_line);

    // SAFETY: `token` is a primary token from get_process_token and is closed on every path.
    // `command_wide` is NUL-terminated and outlives the call, which CreateProcessW* may mutate
    // in place. `process_info`'s handles are reaped by wait_and_reap.
    unsafe {
        let startup_info = hidden_startup_info();
        let mut process_info = empty_process_info();

        let created = CreateProcessWithTokenW(
            token,
            LOGON_WITH_PROFILE,
            ptr::null(),
            command_wide.as_mut_ptr(),
            CREATE_NO_WINDOW,
            ptr::null(),
            ptr::null(),
            &startup_info,
            &mut process_info,
        );
        // Read the error before CloseHandle, which overwrites the thread's last-error.
        let err = (created == FALSE).then(|| GetLastError());
        CloseHandle(token);

        match err {
            Some(code) => Err(Error::ServiceControl(format!(
                "Failed to create process as SYSTEM: {code}"
            ))),
            None => wait_and_reap(&process_info, "SYSTEM command"),
        }
    }
}

/// Check if SYSTEM elevation is available (running as admin)
pub fn can_use_system_elevation() -> bool {
    crate::services::system_info_service::is_running_as_admin()
}
