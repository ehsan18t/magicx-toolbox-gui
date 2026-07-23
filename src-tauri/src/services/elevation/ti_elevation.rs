//! TrustedInstaller Elevation Functions
//!
//! Execute commands with TrustedInstaller privileges using parent process spoofing.
//! `spawn_as_trusted_installer` is the broker's TI launcher (`broker.rs` calls it internally); the
//! typed per-op TI wrappers this file used to also expose were the old apply pipeline's direct
//! call surface and are gone with it (spec §12) — the redesigned engine routes every System/TI
//! drive through the broker's generic `run_ops` (`services::elevation::run_ops`) instead.

use crate::error::Error;
use std::ptr;

use super::common::{
    enable_debug_privilege, to_wide_string, wait_and_reap, CloseHandle, CloseServiceHandle,
    CreateProcessW, DeleteProcThreadAttributeList, GetLastError, InitializeProcThreadAttributeList,
    OpenProcess, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    UpdateProcThreadAttribute, CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT,
    ERROR_SERVICE_ALREADY_RUNNING, EXTENDED_STARTUPINFO_PRESENT, FALSE, HANDLE,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_CREATE_PROCESS, PROCESS_INFORMATION,
    PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO,
    SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START, SERVICE_STATUS_PROCESS,
    STARTF_USESHOWWINDOW, STARTUPINFOEXW, STARTUPINFOW, SW_HIDE,
};

// ============================================================================
// TRUSTEDINSTALLER ELEVATION
// ============================================================================

/// Start the TrustedInstaller service and wait for it to be running
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

/// Get a handle to the TrustedInstaller process with PROCESS_CREATE_PROCESS access
fn get_trusted_installer_handle() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = start_trusted_installer_service()?;

    // SAFETY: OpenProcess is called with a valid PID obtained from the service.
    // The returned handle is owned by the caller and must be closed.
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

/// Spawn a raw command line as TrustedInstaller via parent-process spoofing (no `cmd.exe` wrapper).
/// This creates a process with TrustedInstaller.exe as its parent, inheriting the TI token.
/// `execute_command_as_trusted_installer` wraps a shell command in `cmd.exe /c` and delegates here.
pub(super) fn spawn_as_trusted_installer(command_line: &str) -> Result<i32, Error> {
    log::info!("Spawning as TrustedInstaller: {}", command_line);

    let ti_handle = get_trusted_installer_handle()?;

    let mut command_wide = to_wide_string(command_line);

    // SAFETY: Windows API calls for parent process spoofing. This creates a process
    // with TrustedInstaller.exe as parent, inheriting its privileges. All handles
    // and attribute lists are properly cleaned up.
    unsafe {
        // Initialize the attribute list for parent process spoofing
        let mut attr_list_size: usize = 0;

        // First call to get the required size
        InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut attr_list_size);

        if attr_list_size == 0 {
            CloseHandle(ti_handle);
            return Err(Error::ServiceControl(
                "Failed to get attribute list size".to_string(),
            ));
        }

        // Allocate memory for attribute list.
        // Use an aligned element type (usize) to ensure proper alignment.
        let usize_count = attr_list_size.div_ceil(std::mem::size_of::<usize>());
        let mut attr_list_buffer: Vec<usize> = vec![0; usize_count];
        let attr_list = attr_list_buffer.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;

        // Initialize the attribute list
        if InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_list_size) == FALSE {
            CloseHandle(ti_handle);
            return Err(Error::ServiceControl(format!(
                "Failed to initialize attribute list: {}",
                GetLastError()
            )));
        }

        // Set parent process attribute
        let mut ti_handle_copy = ti_handle;
        if UpdateProcThreadAttribute(
            attr_list,
            0,
            PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
            &mut ti_handle_copy as *mut _ as *mut _,
            std::mem::size_of::<HANDLE>(),
            ptr::null_mut(),
            ptr::null_mut(),
        ) == FALSE
        {
            DeleteProcThreadAttributeList(attr_list);
            CloseHandle(ti_handle);
            return Err(Error::ServiceControl(format!(
                "Failed to set parent process attribute: {}",
                GetLastError()
            )));
        }

        // Set up STARTUPINFOEXW with hidden window
        let startup_info = STARTUPINFOEXW {
            StartupInfo: STARTUPINFOW {
                cb: std::mem::size_of::<STARTUPINFOEXW>() as u32,
                lpReserved: ptr::null_mut(),
                lpDesktop: ptr::null_mut(),
                lpTitle: ptr::null_mut(),
                dwX: 0,
                dwY: 0,
                dwXSize: 0,
                dwYSize: 0,
                dwXCountChars: 0,
                dwYCountChars: 0,
                dwFillAttribute: 0,
                dwFlags: STARTF_USESHOWWINDOW,
                wShowWindow: SW_HIDE as u16,
                cbReserved2: 0,
                lpReserved2: ptr::null_mut(),
                hStdInput: ptr::null_mut(),
                hStdOutput: ptr::null_mut(),
                hStdError: ptr::null_mut(),
            },
            lpAttributeList: attr_list,
        };

        let mut process_info = PROCESS_INFORMATION {
            hProcess: ptr::null_mut(),
            hThread: ptr::null_mut(),
            dwProcessId: 0,
            dwThreadId: 0,
        };

        // Create process with TrustedInstaller as parent
        let result = CreateProcessW(
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

        DeleteProcThreadAttributeList(attr_list);
        CloseHandle(ti_handle);

        if result == FALSE {
            return Err(Error::ServiceControl(format!(
                "Failed to create process as TrustedInstaller: {}",
                GetLastError()
            )));
        }

        wait_and_reap(&process_info, "TrustedInstaller command")
    }
}
