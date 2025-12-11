//! TrustedInstaller Elevation Service
//!
//! Provides functionality to restart the application with SYSTEM privileges.
//! Uses winlogon.exe token (not a Protected Process) and launches in user's session.
//!
//! Note: Full TrustedInstaller elevation is complex due to session 0 isolation.
//! This implementation elevates to SYSTEM which works for most protected registry keys.

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
    CreateProcessAsUserW, GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, STARTUPINFOW,
};

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

/// Find a process ID by name
fn find_process_by_name(target_name: &str) -> Result<u32, Error> {
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

        let target_wide = to_wide_string(target_name);
        let target_len = target_wide.len() - 1; // Exclude null terminator

        loop {
            let process_name: Vec<u16> = entry
                .szExeFile
                .iter()
                .take_while(|&&c| c != 0)
                .copied()
                .collect();

            if process_name.len() == target_len {
                let matches = process_name
                    .iter()
                    .zip(target_wide.iter())
                    .all(|(&a, &b)| char_to_lower(a) == char_to_lower(b));

                if matches {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot);
                    log::info!("Found {} with PID: {}", target_name, pid);
                    return Ok(pid);
                }
            }

            if Process32NextW(snapshot, &mut entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot);
        Err(Error::ServiceControl(format!(
            "{} process not found",
            target_name
        )))
    }
}

/// Get a duplicated token from a process
fn get_process_token(pid: u32) -> Result<HANDLE, Error> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
        if process.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open process {}: {}",
                pid,
                GetLastError()
            )));
        }

        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut token) == FALSE {
            CloseHandle(process);
            return Err(Error::ServiceControl(format!(
                "Failed to open process token: {}",
                GetLastError()
            )));
        }

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
                "Failed to duplicate token: {}",
                GetLastError()
            )));
        }

        CloseHandle(token);
        CloseHandle(process);

        Ok(new_token)
    }
}

/// Restart the current application with SYSTEM privileges
/// Uses winlogon.exe token (runs as SYSTEM in user's session, not protected)
pub fn restart_as_trusted_installer() -> Result<(), Error> {
    log::info!("Attempting to restart as SYSTEM (via winlogon.exe)");

    // Enable debug privilege
    enable_debug_privilege()?;

    // Find winlogon.exe - it runs as SYSTEM in the user's session and is NOT a PPL
    let pid = find_process_by_name("winlogon.exe")?;

    // Get token from winlogon
    let token = get_process_token(pid)?;
    log::info!("Successfully obtained SYSTEM token from winlogon.exe");

    // Get our executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| Error::ServiceControl(format!("Failed to get executable path: {}", e)))?;

    let exe_path_wide = to_wide_string(exe_path.to_string_lossy().as_ref());

    unsafe {
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
                "Failed to create process as SYSTEM: {}",
                GetLastError()
            )));
        }

        CloseHandle(process_info.hProcess);
        CloseHandle(process_info.hThread);

        log::info!(
            "Successfully launched new process as SYSTEM (PID: {})",
            process_info.dwProcessId
        );

        std::process::exit(0);
    }
}

/// Check if we might be running with elevated privileges
#[allow(dead_code)]
pub fn is_elevated() -> bool {
    crate::services::system_info_service::is_running_as_admin()
}
