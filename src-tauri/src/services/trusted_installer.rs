//! SYSTEM Elevation Service
//!
//! Provides functionality to execute commands as SYSTEM without restarting the app.
//! Uses winlogon.exe token (not a Protected Process) and CreateProcessWithTokenW
//! to launch hidden cmd.exe processes that can modify protected registry keys.

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
    CreateProcessWithTokenW, GetCurrentProcess, OpenProcess, OpenProcessToken, CREATE_NO_WINDOW,
    LOGON_WITH_PROFILE, PROCESS_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, STARTUPINFOW,
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
        let target_len = target_wide.len() - 1;

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

/// Get SYSTEM token from winlogon.exe
fn get_system_token() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = find_process_by_name("winlogon.exe")?;
    log::debug!("Found winlogon.exe with PID: {}", pid);
    get_process_token(pid)
}

/// Execute a command as SYSTEM and wait for it to complete
/// Returns the exit code
fn execute_command_as_system(command_line: &str) -> Result<i32, Error> {
    let token = get_system_token()?;
    log::debug!("Got SYSTEM token, executing command: {}", command_line);

    // Build command line: cmd.exe /c <command>
    let full_command = format!("cmd.exe /c {}", command_line);
    let mut command_wide = to_wide_string(&full_command);

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

        let result = CreateProcessWithTokenW(
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

        CloseHandle(token);

        if result == FALSE {
            return Err(Error::ServiceControl(format!(
                "Failed to create process as SYSTEM: {}",
                GetLastError()
            )));
        }

        // Wait for the process to complete
        windows_sys::Win32::System::Threading::WaitForSingleObject(
            process_info.hProcess,
            0xFFFFFFFF, // INFINITE
        );

        // Get exit code
        let mut exit_code: u32 = 0;
        windows_sys::Win32::System::Threading::GetExitCodeProcess(
            process_info.hProcess,
            &mut exit_code,
        );

        CloseHandle(process_info.hProcess);
        CloseHandle(process_info.hThread);

        log::debug!("SYSTEM command completed with exit code: {}", exit_code);
        Ok(exit_code as i32)
    }
}

/// Get the current user's SID (for converting HKCU to HKU\<SID>)
fn get_current_user_sid() -> Result<String, Error> {
    use std::process::Command;

    // Use whoami /user to get the SID - cleaner than Windows API
    let output = Command::new("whoami")
        .args(["/user", "/fo", "csv", "/nh"])
        .output()
        .map_err(|e| Error::ServiceControl(format!("Failed to run whoami: {}", e)))?;

    if !output.status.success() {
        return Err(Error::ServiceControl("whoami failed".to_string()));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    // Output format: "DOMAIN\User","S-1-5-21-..."
    let parts: Vec<&str> = output_str.trim().split(',').collect();
    if parts.len() >= 2 {
        let sid = parts[1].trim_matches('"').to_string();
        log::debug!("Current user SID: {}", sid);
        Ok(sid)
    } else {
        Err(Error::ServiceControl(format!(
            "Failed to parse SID from whoami output: {}",
            output_str
        )))
    }
}

/// Set a registry value as SYSTEM using reg.exe
/// This bypasses normal permission checks by running reg.exe as SYSTEM
pub fn set_registry_value_as_system(
    hive: &str,
    key: &str,
    value_name: &str,
    value_type: &str,
    value_data: &str,
) -> Result<(), Error> {
    log::info!(
        "Setting registry value as SYSTEM: {}\\{}\\{} = {} ({})",
        hive,
        key,
        value_name,
        value_data,
        value_type
    );

    // For HKCU, we need to use HKU\<SID> since SYSTEM's HKCU is different
    let full_key = if hive.eq_ignore_ascii_case("HKCU") {
        let sid = get_current_user_sid()?;
        log::debug!("Converting HKCU to HKU\\{}", sid);
        format!("HKU\\{}\\{}", sid, key)
    } else {
        format!("{}\\{}", hive, key)
    };

    // Build reg.exe command
    // reg add "HKLM\Software\..." /v "ValueName" /t REG_DWORD /d 1 /f
    let command = format!(
        "reg add \"{}\" /v \"{}\" /t {} /d {} /f",
        full_key, value_name, value_type, value_data
    );

    let exit_code = execute_command_as_system(&command)?;

    if exit_code == 0 {
        log::info!("Successfully set registry value as SYSTEM");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "reg.exe failed with exit code: {}",
            exit_code
        )))
    }
}

/// Delete a registry value as SYSTEM using reg.exe
pub fn delete_registry_value_as_system(
    hive: &str,
    key: &str,
    value_name: &str,
) -> Result<(), Error> {
    log::info!(
        "Deleting registry value as SYSTEM: {}\\{}\\{}",
        hive,
        key,
        value_name
    );

    // For HKCU, we need to use HKU\<SID> since SYSTEM's HKCU is different
    let full_key = if hive.eq_ignore_ascii_case("HKCU") {
        let sid = get_current_user_sid()?;
        format!("HKU\\{}\\{}", sid, key)
    } else {
        format!("{}\\{}", hive, key)
    };

    let command = format!("reg delete \"{}\" /v \"{}\" /f", full_key, value_name);

    let exit_code = execute_command_as_system(&command)?;

    if exit_code == 0 {
        log::info!("Successfully deleted registry value as SYSTEM");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "reg.exe delete failed with exit code: {}",
            exit_code
        )))
    }
}

/// Check if SYSTEM elevation is available (running as admin)
pub fn can_use_system_elevation() -> bool {
    crate::services::system_info_service::is_running_as_admin()
}
