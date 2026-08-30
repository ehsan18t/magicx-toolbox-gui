//! TrustedInstaller elevation: start the TI service, then spawn the broker child with
//! TrustedInstaller.exe spoofed as its parent so it inherits the TI token.

use crate::error::Error;
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE};
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STATUS_PROCESS,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList, OpenProcess,
    QueryFullProcessImageNameW, UpdateProcThreadAttribute, CREATE_NO_WINDOW,
    CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_CREATE_PROCESS, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW,
};

use super::common::{
    empty_process_info, enable_debug_privilege, hidden_startup_info, to_wide_string, wait_and_reap,
};

/// dwCurrentState values we distinguish while waiting for the service.
const SERVICE_STOPPED: u32 = 1;
const SERVICE_START_PENDING: u32 = 2;
const SERVICE_RUNNING: u32 = 4;

const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

/// The image the TrustedInstaller service must actually be running, checked before its handle is
/// used as a spawn parent.
const TI_IMAGE_SUFFIX: &str = r"\servicing\TrustedInstaller.exe";

/// Turn the Win32 codes this path actually produces into something a support engineer can act on.
/// Anything else keeps its bare number, which is still better than nothing.
fn describe_win32(code: u32) -> String {
    let name = match code {
        5 => "ERROR_ACCESS_DENIED",
        1058 => "ERROR_SERVICE_DISABLED",
        1060 => "ERROR_SERVICE_DOES_NOT_EXIST",
        1061 => "ERROR_SERVICE_CANNOT_ACCEPT_CTRL",
        1062 => "ERROR_SERVICE_NOT_ACTIVE",
        1069 => "ERROR_SERVICE_LOGON_FAILED",
        1072 => "ERROR_SERVICE_MARKED_FOR_DELETE",
        1314 => "ERROR_PRIVILEGE_NOT_HELD",
        _ => return code.to_string(),
    };
    format!("{code} ({name})")
}

/// A human-readable name for a service state code, for the timeout message.
fn describe_service_state(state: u32) -> &'static str {
    match state {
        SERVICE_STOPPED => "stopped",
        SERVICE_START_PENDING => "start-pending",
        3 => "stop-pending",
        SERVICE_RUNNING => "running",
        _ => "an unexpected state",
    }
}

/// Start the TrustedInstaller service and wait for it to be running, returning its pid.
fn start_trusted_installer_service() -> Result<u32, Error> {
    // SAFETY: Windows Service Control Manager API calls. All handles (SCM and service)
    // are closed on both success and error paths. Service status query uses properly
    // sized structures.
    unsafe {
        // Open Service Control Manager
        let scm = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open the service control manager: {}",
                describe_win32(GetLastError())
            )));
        }

        let service_name = to_wide_string("TrustedInstaller");
        let service = OpenServiceW(
            scm,
            service_name.as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        );

        if service.is_null() {
            // Read the error BEFORE closing: CloseServiceHandle overwrites the thread last-error,
            // so reading it afterwards reports the close, not the OpenServiceW that failed.
            let err = GetLastError();
            CloseServiceHandle(scm);
            return Err(Error::ServiceControl(format!(
                "Failed to open the TrustedInstaller service: {}",
                describe_win32(err)
            )));
        }

        // Check current status
        let mut bytes_needed: u32 = 0;
        let mut status = std::mem::MaybeUninit::<SERVICE_STATUS_PROCESS>::zeroed();

        let query_result = QueryServiceStatusEx(
            service,
            SC_STATUS_PROCESS_INFO,
            status.as_mut_ptr() as *mut u8,
            std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
            &mut bytes_needed,
        );

        if query_result == 0 {
            let err = GetLastError();
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            return Err(Error::ServiceControl(format!(
                "Failed to query the TrustedInstaller service status: {}",
                describe_win32(err)
            )));
        }

        let status = status.assume_init();
        let current_state = status.dwCurrentState;

        // If already running, return the PID
        if current_state == SERVICE_RUNNING {
            let pid = status.dwProcessId;
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            log::debug!("TrustedInstaller already running with PID: {}", pid);
            return Ok(pid);
        }

        // Start the service
        log::debug!("Starting TrustedInstaller service...");
        let start_result = StartServiceW(service, 0, ptr::null());

        if start_result == 0 {
            let err = GetLastError();
            if err != ERROR_SERVICE_ALREADY_RUNNING {
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(Error::ServiceControl(format!(
                    "Failed to start the TrustedInstaller service: {}",
                    describe_win32(err)
                )));
            }
        }

        // Poll for up to 10 seconds. Query first and sleep only between attempts: a service that
        // is already running by the time StartServiceW returns should cost nothing. The last
        // observed state is carried out of the loop so a timeout can say what the service was
        // actually doing, which is the difference between "stuck starting" and "never started".
        let mut last_state = SERVICE_STOPPED;
        let mut last_query_err = None;
        for attempt in 0..100 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let mut status = std::mem::MaybeUninit::<SERVICE_STATUS_PROCESS>::zeroed();
            let query_result = QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                status.as_mut_ptr() as *mut u8,
                std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut bytes_needed,
            );

            if query_result == 0 {
                last_query_err = Some(GetLastError());
                continue;
            }

            let status = status.assume_init();
            last_state = status.dwCurrentState;
            if last_state == SERVICE_RUNNING {
                let pid = status.dwProcessId;
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                log::debug!("TrustedInstaller started with PID: {pid}");
                return Ok(pid);
            }
        }

        CloseServiceHandle(service);
        CloseServiceHandle(scm);
        let detail = match last_query_err {
            Some(err) => format!(
                "last state {}, last status query failed: {}",
                describe_service_state(last_state),
                describe_win32(err)
            ),
            None => format!("it is still {}", describe_service_state(last_state)),
        };
        Err(Error::ServiceControl(format!(
            "The TrustedInstaller service did not start within 10s ({detail})"
        )))
    }
}

/// The image path of an open process handle, for identity verification.
///
/// # Safety
/// `handle` must be a valid process handle opened with `PROCESS_QUERY_LIMITED_INFORMATION`.
unsafe fn process_image_path(handle: HANDLE) -> Option<String> {
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len) == FALSE {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}

/// Open the TrustedInstaller process with `PROCESS_CREATE_PROCESS`, the access the parent spoof
/// needs, and verify it really is TrustedInstaller before handing the handle back. The returned
/// handle is owned by the caller.
///
/// The verification is not paranoia. `start_trusted_installer_service` reads a pid out of the SCM
/// and closes its handles before returning it, and the service stops itself when idle, so the pid
/// can be dead and recycled by the time it is opened. Spoofing a recycled pid as
/// `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` would make the broker child inherit THAT process's token
/// instead, silently, at whatever privilege it happens to hold. Checking the image closes the
/// window between the SCM's answer and the handle we actually use.
fn get_trusted_installer_handle() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = start_trusted_installer_service()?;

    // SAFETY: `pid` came from the SCM's own status for a running service; the handle is closed on
    // every failure path below and otherwise owned by the caller.
    unsafe {
        let handle = OpenProcess(
            PROCESS_CREATE_PROCESS | PROCESS_QUERY_LIMITED_INFORMATION,
            FALSE,
            pid,
        );
        if handle.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open the TrustedInstaller process (pid {pid}): {}",
                describe_win32(GetLastError())
            )));
        }

        match process_image_path(handle) {
            Some(path)
                if path
                    .to_ascii_lowercase()
                    .ends_with(&TI_IMAGE_SUFFIX.to_ascii_lowercase()) =>
            {
                Ok(handle)
            }
            Some(path) => {
                CloseHandle(handle);
                Err(Error::ServiceControl(format!(
                    "pid {pid} is {path}, not TrustedInstaller: the service stopped and the pid was reused"
                )))
            }
            None => {
                let err = GetLastError();
                CloseHandle(handle);
                Err(Error::ServiceControl(format!(
                    "Could not verify that pid {pid} is TrustedInstaller: {}",
                    describe_win32(err)
                )))
            }
        }
    }
}

/// Spawn `command_line` with TrustedInstaller.exe as its parent, so it inherits the TI token, and
/// wait for it. The broker's TI launcher; the command line is built by
/// `broker::run_elevated_broker`, never by a caller.
pub(super) fn spawn_as_trusted_installer(command_line: &str) -> Result<i32, Error> {
    log::info!("Spawning as TrustedInstaller: {}", command_line);

    let ti_handle = get_trusted_installer_handle()?;
    let mut command_wide = to_wide_string(command_line);

    // SAFETY: `ti_handle` is closed on every path. The attribute list buffer is usize-aligned and
    // sized by the API's own first (sizing) call, and is deleted before its backing Vec drops.
    // `command_wide` is NUL-terminated and outlives the call, which CreateProcessW may mutate in
    // place. `process_info`'s handles are reaped by wait_and_reap.
    unsafe {
        let mut spawn = || -> Result<i32, Error> {
            let mut attr_list_size: usize = 0;
            // First call sizes the buffer; it is documented to fail, so only the size is meaningful.
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut attr_list_size);
            if attr_list_size == 0 {
                return Err(Error::ServiceControl(format!(
                    "Failed to size the process attribute list: {}",
                    describe_win32(GetLastError())
                )));
            }

            let usize_count = attr_list_size.div_ceil(std::mem::size_of::<usize>());
            let mut attr_list_buffer: Vec<usize> = vec![0; usize_count];
            let attr_list = attr_list_buffer.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;

            if InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_list_size) == FALSE {
                return Err(Error::ServiceControl(format!(
                    "Failed to initialize the process attribute list: {}",
                    describe_win32(GetLastError())
                )));
            }

            let mut ti_handle_copy = ti_handle;
            let updated = UpdateProcThreadAttribute(
                attr_list,
                0,
                PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
                &mut ti_handle_copy as *mut _ as *mut _,
                std::mem::size_of::<HANDLE>(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if updated == FALSE {
                let err = GetLastError();
                DeleteProcThreadAttributeList(attr_list);
                return Err(Error::ServiceControl(format!(
                    "Failed to set the parent-process attribute: {}",
                    describe_win32(err)
                )));
            }

            let mut startup_info = STARTUPINFOEXW {
                StartupInfo: hidden_startup_info(),
                lpAttributeList: attr_list,
            };
            // With EXTENDED_STARTUPINFO_PRESENT, `cb` must span the EX struct, not just the inner
            // STARTUPINFOW, or CreateProcessW will not see the attribute list.
            startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
            let mut process_info = empty_process_info();

            let created = CreateProcessW(
                ptr::null(),
                command_wide.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                FALSE,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
                ptr::null(),
                ptr::null(),
                &startup_info.StartupInfo,
                &mut process_info,
            );
            let err = (created == FALSE).then(|| GetLastError());
            DeleteProcThreadAttributeList(attr_list);

            match err {
                Some(code) => Err(Error::ServiceControl(format!(
                    "Failed to create the broker process as TrustedInstaller: {}",
                    describe_win32(code)
                ))),
                None => wait_and_reap(&process_info, "TrustedInstaller command"),
            }
        };

        let result = spawn();
        CloseHandle(ti_handle);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The codes a hardened or managed machine actually produces are the ones worth naming: a
    /// disabled service and a missing service send the user to completely different places, and
    /// "1058" alone sends them nowhere.
    #[test]
    fn the_service_errors_worth_acting_on_are_named() {
        assert_eq!(describe_win32(1058), "1058 (ERROR_SERVICE_DISABLED)");
        assert_eq!(describe_win32(1060), "1060 (ERROR_SERVICE_DOES_NOT_EXIST)");
        assert_eq!(describe_win32(5), "5 (ERROR_ACCESS_DENIED)");
        assert_eq!(describe_win32(1314), "1314 (ERROR_PRIVILEGE_NOT_HELD)");
    }

    /// An unmapped code still has to survive intact rather than being swallowed.
    #[test]
    fn an_unmapped_error_keeps_its_number() {
        assert_eq!(describe_win32(999_999), "999999");
    }

    #[test]
    fn service_states_read_as_words() {
        assert_eq!(describe_service_state(SERVICE_STOPPED), "stopped");
        assert_eq!(
            describe_service_state(SERVICE_START_PENDING),
            "start-pending"
        );
        assert_eq!(describe_service_state(SERVICE_RUNNING), "running");
    }

    /// The parent-spoof identity check is a suffix match on the image path, so it has to be
    /// case-insensitive (Windows paths are) and must not match a lookalike that merely contains
    /// the name somewhere.
    #[test]
    fn the_image_check_matches_case_insensitively_and_only_at_the_end() {
        let matches = |path: &str| {
            path.to_ascii_lowercase()
                .ends_with(&TI_IMAGE_SUFFIX.to_ascii_lowercase())
        };
        assert!(matches(r"C:\Windows\servicing\TrustedInstaller.exe"));
        assert!(matches(r"c:\windows\SERVICING\trustedinstaller.EXE"));
        assert!(
            !matches(r"C:\Temp\TrustedInstaller.exe"),
            "the servicing directory is part of the identity"
        );
        assert!(
            !matches(r"C:\Windows\servicing\TrustedInstaller.exe.bak"),
            "a suffix match must anchor at the end"
        );
        assert!(!matches(r"C:\Windows\System32\svchost.exe"));
    }
}
