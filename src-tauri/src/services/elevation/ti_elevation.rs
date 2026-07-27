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
    UpdateProcThreadAttribute, CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT,
    EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_CREATE_PROCESS,
    PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW,
};

use super::common::{
    empty_process_info, enable_debug_privilege, hidden_startup_info, to_wide_string, wait_and_reap,
};

/// dwCurrentState value for a running service.
const SERVICE_RUNNING: u32 = 4;
const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

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
                "Failed to open SCManager: {}",
                GetLastError()
            )));
        }

        let service_name = to_wide_string("TrustedInstaller");
        let service = OpenServiceW(
            scm,
            service_name.as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        );

        if service.is_null() {
            CloseServiceHandle(scm);
            return Err(Error::ServiceControl(format!(
                "Failed to open TrustedInstaller service: {}",
                GetLastError()
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
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            return Err(Error::ServiceControl(format!(
                "Failed to query TrustedInstaller status: {}",
                GetLastError()
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
                    "Failed to start TrustedInstaller: {}",
                    err
                )));
            }
        }

        // Wait for service to be running (poll for up to 10 seconds)
        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(100));

            let mut status = std::mem::MaybeUninit::<SERVICE_STATUS_PROCESS>::zeroed();
            let query_result = QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                status.as_mut_ptr() as *mut u8,
                std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut bytes_needed,
            );

            if query_result != 0 {
                let status = status.assume_init();
                if status.dwCurrentState == SERVICE_RUNNING {
                    let pid = status.dwProcessId;
                    CloseServiceHandle(service);
                    CloseServiceHandle(scm);
                    log::debug!("TrustedInstaller started with PID: {}", pid);
                    return Ok(pid);
                }
            }
        }

        CloseServiceHandle(service);
        CloseServiceHandle(scm);
        Err(Error::ServiceControl(
            "Timeout waiting for TrustedInstaller to start".to_string(),
        ))
    }
}

/// Open the TrustedInstaller process with `PROCESS_CREATE_PROCESS`, the access the parent spoof
/// needs. The returned handle is owned by the caller.
fn get_trusted_installer_handle() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = start_trusted_installer_service()?;

    // SAFETY: `pid` came from the SCM's own status for a running service.
    unsafe {
        let handle = OpenProcess(PROCESS_CREATE_PROCESS, FALSE, pid);
        if handle.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open TrustedInstaller process: {}",
                GetLastError()
            )));
        }
        Ok(handle)
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
                return Err(Error::ServiceControl(
                    "Failed to get attribute list size".to_string(),
                ));
            }

            let usize_count = attr_list_size.div_ceil(std::mem::size_of::<usize>());
            let mut attr_list_buffer: Vec<usize> = vec![0; usize_count];
            let attr_list = attr_list_buffer.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;

            if InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_list_size) == FALSE {
                return Err(Error::ServiceControl(format!(
                    "Failed to initialize attribute list: {}",
                    GetLastError()
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
                    "Failed to set parent process attribute: {err}"
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
                    "Failed to create process as TrustedInstaller: {code}"
                ))),
                None => wait_and_reap(&process_info, "TrustedInstaller command"),
            }
        };

        let result = spawn();
        CloseHandle(ti_handle);
        result
    }
}
