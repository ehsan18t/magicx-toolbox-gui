//! TrustedInstaller Elevation Service
//!
//! Provides functionality to restart the application with TrustedInstaller privileges.
//! This allows modifying protected registry keys and system files.

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
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus, StartServiceW,
    SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, SERVICE_START, SERVICE_STATUS, SERVICE_STOPPED,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessAsUserW, GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, STARTUPINFOW,
};

const TRUSTED_INSTALLER_SERVICE: &str = "TrustedInstaller";
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

/// Convert a Rust string to a null-terminated wide string
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Convert u16 to lowercase for case-insensitive comparison
fn char_to_lower(c: u16) -> u16 {
    if c >= b'A' as u16 && c <= b'Z' as u16 {
        c + 32
    } else {
        c
    }
}

/// Enable SeDebugPrivilege for the current process
fn enable_debug_privilege() -> Result<(), Error> {
    unsafe {
        let mut token_handle: HANDLE = ptr::null_mut();
        let process = GetCurrentProcess();

        if OpenProcessToken(
            process,
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        ) == FALSE
        {
            return Err(Error::ServiceControl(format!(
                "Failed to open process token: {}",
                GetLastError()
            )));
        }

        let privilege_name = to_wide_string("SeDebugPrivilege");
        let mut luid = LUID {
            LowPart: 0,
            HighPart: 0,
        };

        if LookupPrivilegeValueW(ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
            CloseHandle(token_handle);
            return Err(Error::ServiceControl(format!(
                "Failed to lookup SeDebugPrivilege: {}",
                GetLastError()
            )));
        }

        let privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        if AdjustTokenPrivileges(
            token_handle,
            FALSE,
            &privileges,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        ) == FALSE
        {
            CloseHandle(token_handle);
            return Err(Error::ServiceControl(format!(
                "Failed to adjust token privileges: {}",
                GetLastError()
            )));
        }

        CloseHandle(token_handle);
        log::debug!("SeDebugPrivilege enabled successfully");
        Ok(())
    }
}

/// Start the TrustedInstaller service if not running
fn start_trusted_installer_service() -> Result<(), Error> {
    unsafe {
        let sc_manager = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        if sc_manager.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open SC Manager: {}",
                GetLastError()
            )));
        }

        let service_name = to_wide_string(TRUSTED_INSTALLER_SERVICE);
        let service = OpenServiceW(
            sc_manager,
            service_name.as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        );

        if service.is_null() {
            CloseServiceHandle(sc_manager);
            return Err(Error::ServiceControl(format!(
                "Failed to open TrustedInstaller service: {}",
                GetLastError()
            )));
        }

        // Check if already running
        let mut status = SERVICE_STATUS {
            dwServiceType: 0,
            dwCurrentState: 0,
            dwControlsAccepted: 0,
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        };

        if QueryServiceStatus(service, &mut status) != FALSE
            && status.dwCurrentState != SERVICE_STOPPED
        {
            log::debug!("TrustedInstaller service is already running");
            CloseServiceHandle(service);
            CloseServiceHandle(sc_manager);
            return Ok(());
        }

        // Start the service
        if StartServiceW(service, 0, ptr::null()) == FALSE {
            let error = GetLastError();
            CloseServiceHandle(service);
            CloseServiceHandle(sc_manager);

            // ERROR_SERVICE_ALREADY_RUNNING = 1056
            if error != 1056 {
                return Err(Error::ServiceControl(format!(
                    "Failed to start TrustedInstaller service: {}",
                    error
                )));
            }
        }

        CloseServiceHandle(service);
        CloseServiceHandle(sc_manager);

        log::info!("TrustedInstaller service started");

        // Give the service time to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(())
    }
}

/// Find the TrustedInstaller.exe process ID
fn find_trusted_installer_pid() -> Result<u32, Error> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() || snapshot == INVALID_HANDLE_VALUE {
            return Err(Error::ServiceControl(format!(
                "Failed to create process snapshot: {}",
                GetLastError()
            )));
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; 260],
        };

        if Process32FirstW(snapshot, &mut entry) == FALSE {
            CloseHandle(snapshot);
            return Err(Error::ServiceControl("No processes found".into()));
        }

        let target_name = to_wide_string("TrustedInstaller.exe");
        let target_len = target_name.len() - 1; // Exclude null terminator

        loop {
            // Compare process name (case insensitive)
            let process_name: Vec<u16> = entry
                .szExeFile
                .iter()
                .take_while(|&&c| c != 0)
                .copied()
                .collect();

            if process_name.len() == target_len {
                let matches = process_name
                    .iter()
                    .zip(target_name.iter())
                    .all(|(&a, &b)| char_to_lower(a) == char_to_lower(b));

                if matches {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot);
                    log::info!("Found TrustedInstaller.exe with PID: {}", pid);
                    return Ok(pid);
                }
            }

            if Process32NextW(snapshot, &mut entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot);
        Err(Error::ServiceControl(
            "TrustedInstaller.exe process not found".into(),
        ))
    }
}

/// Get the token from TrustedInstaller process
fn get_trusted_installer_token() -> Result<HANDLE, Error> {
    // First, start the service
    start_trusted_installer_service()?;

    // Enable debug privilege
    enable_debug_privilege()?;

    // Find the process
    let pid = find_trusted_installer_pid()?;

    unsafe {
        // Open the process
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
        if process.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open TrustedInstaller process: {}",
                GetLastError()
            )));
        }

        // Get the process token
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut token) == FALSE {
            CloseHandle(process);
            return Err(Error::ServiceControl(format!(
                "Failed to open TrustedInstaller token: {}",
                GetLastError()
            )));
        }

        // Duplicate the token as a primary token
        let mut new_token: HANDLE = ptr::null_mut();
        if DuplicateTokenEx(
            token,
            TOKEN_ALL_ACCESS,
            ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut new_token,
        ) == FALSE
        {
            CloseHandle(token);
            CloseHandle(process);
            return Err(Error::ServiceControl(format!(
                "Failed to duplicate TrustedInstaller token: {}",
                GetLastError()
            )));
        }

        CloseHandle(token);
        CloseHandle(process);

        log::info!("Successfully obtained TrustedInstaller token");
        Ok(new_token)
    }
}

/// Restart the current application as TrustedInstaller
pub fn restart_as_trusted_installer() -> Result<(), Error> {
    log::info!("Attempting to restart as TrustedInstaller");

    // Get our executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| Error::ServiceControl(format!("Failed to get executable path: {}", e)))?;

    let exe_path_wide = to_wide_string(exe_path.to_string_lossy().as_ref());

    // Get TrustedInstaller token
    let token = get_trusted_installer_token()?;

    unsafe {
        // Setup startup info
        let startup_info = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
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
            dwFlags: 0,
            wShowWindow: 0,
            cbReserved2: 0,
            lpReserved2: ptr::null_mut(),
            hStdInput: ptr::null_mut(),
            hStdOutput: ptr::null_mut(),
            hStdError: ptr::null_mut(),
        };

        let mut process_info = PROCESS_INFORMATION {
            hProcess: ptr::null_mut(),
            hThread: ptr::null_mut(),
            dwProcessId: 0,
            dwThreadId: 0,
        };

        // Create the new process with TrustedInstaller token
        let result = CreateProcessAsUserW(
            token,
            exe_path_wide.as_ptr(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            FALSE,
            0,
            ptr::null(),
            ptr::null(),
            &startup_info,
            &mut process_info,
        );

        CloseHandle(token);

        if result == FALSE {
            return Err(Error::ServiceControl(format!(
                "Failed to create process as TrustedInstaller: {}",
                GetLastError()
            )));
        }

        // Close handles to the new process
        CloseHandle(process_info.hProcess);
        CloseHandle(process_info.hThread);

        log::info!(
            "Successfully launched new process as TrustedInstaller (PID: {})",
            process_info.dwProcessId
        );

        // Exit the current process
        std::process::exit(0);
    }
}

/// Check if we might be running as TrustedInstaller (heuristic check)
#[allow(dead_code)]
pub fn is_elevated() -> bool {
    // For now, just check if we're admin - TrustedInstaller detection is complex
    crate::services::system_info_service::is_running_as_admin()
}
