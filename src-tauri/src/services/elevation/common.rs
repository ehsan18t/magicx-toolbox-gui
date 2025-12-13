//! Common Utilities and Constants
//!
//! Shared utilities for elevation services:
//! - String conversion functions
//! - Security helpers (escaping, validation)
//! - Windows API constants

use crate::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

pub use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, LUID};
pub use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, DuplicateTokenEx, LookupPrivilegeValueW, SecurityImpersonation,
    TokenPrimary, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_ALL_ACCESS, TOKEN_DUPLICATE, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
pub use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
pub use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STATUS_PROCESS,
};
pub use windows_sys::Win32::System::Threading::{
    CreateProcessW, CreateProcessWithTokenW, DeleteProcThreadAttributeList, GetCurrentProcess,
    InitializeProcThreadAttributeList, OpenProcess, OpenProcessToken, UpdateProcThreadAttribute,
    CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, LOGON_WITH_PROFILE,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_CREATE_PROCESS, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW,
    STARTUPINFOW,
};
pub use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
pub const STARTF_USESHOWWINDOW: u32 = 0x00000001;
/// Timeout for waiting on elevated processes (30 seconds)
pub const ELEVATED_PROCESS_TIMEOUT_MS: u32 = 30_000;

// Windows Service state constants (dwCurrentState values)
// These are defined for completeness and future use
#[allow(dead_code)]
/// Service is stopped
pub const SERVICE_STOPPED: u32 = 1;
#[allow(dead_code)]
/// Service is starting
pub const SERVICE_START_PENDING: u32 = 2;
#[allow(dead_code)]
/// Service is stopping
pub const SERVICE_STOP_PENDING: u32 = 3;
/// Service is running
pub const SERVICE_RUNNING: u32 = 4;
#[allow(dead_code)]
/// Service continue is pending
pub const SERVICE_CONTINUE_PENDING: u32 = 5;
#[allow(dead_code)]
/// Service pause is pending
pub const SERVICE_PAUSE_PENDING: u32 = 6;
#[allow(dead_code)]
/// Service is paused
pub const SERVICE_PAUSED: u32 = 7;

// Windows error codes
/// The service is already running
pub const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

/// Convert a Rust string to a null-terminated wide string
pub fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Convert u16 to lowercase for case-insensitive comparison
pub fn char_to_lower(c: u16) -> u16 {
    if c >= b'A' as u16 && c <= b'Z' as u16 {
        c + 32
    } else {
        c
    }
}

/// Escape a string for safe use in shell commands.
/// This prevents command injection by escaping special characters.
///
/// For Windows cmd.exe, we need to:
/// 1. Escape existing double quotes by doubling them
/// 2. Escape special shell characters: & | < > ^ %
pub fn escape_shell_arg(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 10);
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\"\""), // Double quotes
            '&' | '|' | '<' | '>' | '^' => {
                escaped.push('^'); // Escape with caret
                escaped.push(c);
            }
            '%' => {
                escaped.push('%'); // Escape percent with percent
                escaped.push('%');
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Validate that a registry path contains only safe characters.
/// Returns an error if the path contains potentially dangerous characters.
pub fn validate_registry_path(path: &str) -> Result<(), Error> {
    // Allow alphanumeric, backslash, underscore, hyphen, period, and space
    for c in path.chars() {
        if !c.is_alphanumeric() && !matches!(c, '\\' | '_' | '-' | '.' | ' ' | '{' | '}') {
            return Err(Error::ServiceControl(format!(
                "Invalid character '{}' in registry path: {}",
                c, path
            )));
        }
    }
    Ok(())
}

/// Enable SeDebugPrivilege for the current process
pub fn enable_debug_privilege() -> Result<(), Error> {
    // SAFETY: Windows API calls for privilege management. All handles are properly
    // closed using CloseHandle in deferred manner.
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(Error::WindowsApi(format!(
                "OpenProcessToken failed: {}",
                GetLastError()
            )));
        }

        // Look up the LUID for SeDebugPrivilege
        let privilege_name = to_wide_string("SeDebugPrivilege");
        let mut luid: LUID = std::mem::zeroed();
        if LookupPrivilegeValueW(ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
            CloseHandle(token);
            return Err(Error::WindowsApi(format!(
                "LookupPrivilegeValue failed: {}",
                GetLastError()
            )));
        }

        // Build the token privileges structure
        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0] = LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        };

        // Enable the privilege
        if AdjustTokenPrivileges(token, FALSE, &tp, 0, ptr::null_mut(), ptr::null_mut()) == FALSE {
            CloseHandle(token);
            return Err(Error::WindowsApi(format!(
                "AdjustTokenPrivileges failed: {}",
                GetLastError()
            )));
        }

        // Check if we actually got the privilege
        let error = GetLastError();
        CloseHandle(token);

        // ERROR_NOT_ALL_ASSIGNED = 1300
        if error == 1300 {
            return Err(Error::WindowsApi(
                "SeDebugPrivilege not available - admin rights required".to_string(),
            ));
        }

        log::trace!("Successfully enabled SeDebugPrivilege");
        Ok(())
    }
}

/// Find a process ID by name
pub fn find_process_by_name(target_name: &str) -> Result<u32, Error> {
    let target_wide = to_wide_string(target_name);

    // SAFETY: Windows ToolHelp32 API calls for process enumeration.
    // Snapshot handle is properly closed using CloseHandle after enumeration.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(Error::WindowsApi(format!(
                "CreateToolhelp32Snapshot failed: {}",
                GetLastError()
            )));
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) == FALSE {
            CloseHandle(snapshot);
            return Err(Error::WindowsApi(format!(
                "Process32FirstW failed: {}",
                GetLastError()
            )));
        }

        loop {
            // Case-insensitive comparison without trailing nulls
            let entry_len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let target_len = target_wide.len() - 1; // Exclude null terminator

            if entry_len == target_len {
                let matches = entry.szExeFile[..entry_len]
                    .iter()
                    .zip(&target_wide[..target_len])
                    .all(|(&a, &b)| char_to_lower(a) == char_to_lower(b));

                if matches {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot);
                    log::trace!("Found {} with PID {}", target_name, pid);
                    return Ok(pid);
                }
            }

            if Process32NextW(snapshot, &mut entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot);
        Err(Error::WindowsApi(format!(
            "Process not found: {}",
            target_name
        )))
    }
}

/// Get a duplicated token from a process
pub fn get_process_token(pid: u32) -> Result<HANDLE, Error> {
    // SAFETY: Windows API calls for token duplication. Process handle is closed
    // after token duplication, duplicated token is returned for caller to manage.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
        if process.is_null() {
            return Err(Error::WindowsApi(format!(
                "OpenProcess failed for PID {}: {}",
                pid,
                GetLastError()
            )));
        }

        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut token) == FALSE {
            CloseHandle(process);
            return Err(Error::WindowsApi(format!(
                "OpenProcessToken failed: {}",
                GetLastError()
            )));
        }

        // Duplicate the token for primary use
        let mut dup_token: HANDLE = ptr::null_mut();
        if DuplicateTokenEx(
            token,
            TOKEN_ALL_ACCESS,
            ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut dup_token,
        ) == FALSE
        {
            CloseHandle(token);
            CloseHandle(process);
            return Err(Error::WindowsApi(format!(
                "DuplicateTokenEx failed: {}",
                GetLastError()
            )));
        }

        CloseHandle(token);
        CloseHandle(process);

        log::trace!("Got duplicated token from PID {}", pid);
        Ok(dup_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_shell_arg_no_special() {
        assert_eq!(escape_shell_arg("hello"), "hello");
    }

    #[test]
    fn test_escape_shell_arg_quotes() {
        assert_eq!(escape_shell_arg("hello\"world"), "hello\"\"world");
    }

    #[test]
    fn test_escape_shell_arg_ampersand() {
        assert_eq!(escape_shell_arg("a&b"), "a^&b");
    }

    #[test]
    fn test_escape_shell_arg_pipe() {
        assert_eq!(escape_shell_arg("a|b"), "a^|b");
    }

    #[test]
    fn test_escape_shell_arg_percent() {
        assert_eq!(escape_shell_arg("100%"), "100%%");
    }

    #[test]
    fn test_validate_registry_path_valid() {
        assert!(validate_registry_path("SOFTWARE\\Microsoft\\Windows").is_ok());
    }

    #[test]
    fn test_validate_registry_path_with_braces() {
        assert!(validate_registry_path("CLSID\\{12345}").is_ok());
    }

    #[test]
    fn test_validate_registry_path_invalid_char() {
        assert!(validate_registry_path("SOFTWARE;DROP TABLE").is_err());
    }
}
