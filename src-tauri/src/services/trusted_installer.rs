//! Elevation Services (SYSTEM and TrustedInstaller)
//!
//! Provides functionality to execute commands with elevated privileges:
//! - SYSTEM: Uses winlogon.exe token impersonation via CreateProcessWithTokenW
//! - TrustedInstaller: Uses parent process spoofing with TrustedInstaller.exe
//!
//! TrustedInstaller is more powerful than SYSTEM - it owns protected system files,
//! registry keys, and services like WaaSMedicSvc that even SYSTEM cannot modify.

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
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STATUS_PROCESS,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, CreateProcessWithTokenW, DeleteProcThreadAttributeList, GetCurrentProcess,
    InitializeProcThreadAttributeList, OpenProcess, OpenProcessToken, UpdateProcThreadAttribute,
    CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, LOGON_WITH_PROFILE,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_CREATE_PROCESS, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW,
    STARTUPINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const STARTF_USESHOWWINDOW: u32 = 0x00000001;
/// Timeout for waiting on elevated processes (30 seconds)
const ELEVATED_PROCESS_TIMEOUT_MS: u32 = 30_000;

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

/// Escape a string for safe use in shell commands.
/// This prevents command injection by escaping special characters.
///
/// For Windows cmd.exe, we need to:
/// 1. Escape existing double quotes by doubling them
/// 2. Escape special shell characters: & | < > ^ %
fn escape_shell_arg(s: &str) -> String {
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
fn validate_registry_path(path: &str) -> Result<(), Error> {
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
fn enable_debug_privilege() -> Result<(), Error> {
    // SAFETY: Windows API calls for privilege management. All handles are properly
    // closed and pointers are validated. GetCurrentProcess returns a pseudo-handle
    // that doesn't need closing.
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
    // SAFETY: Windows API calls for process enumeration. Snapshot handle is closed
    // on all code paths. PROCESSENTRY32W struct is properly sized and initialized.
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
    // SAFETY: Windows API calls for token duplication. All handles are properly
    // closed on error paths and the duplicated token is returned for caller cleanup.
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

    // SAFETY: Windows API calls for creating a process with impersonation token.
    // Process and thread handles are closed after waiting for completion.
    // Token handle is closed after use. The command_wide buffer remains valid
    // throughout the CreateProcessAsUserW call.
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
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE as u16,
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

        // Wait for the process to complete (with timeout)
        let wait_result = windows_sys::Win32::System::Threading::WaitForSingleObject(
            process_info.hProcess,
            ELEVATED_PROCESS_TIMEOUT_MS,
        );

        // Check if we timed out (WAIT_TIMEOUT = 0x102)
        if wait_result == 0x102 {
            log::warn!(
                "SYSTEM command timed out after {}ms",
                ELEVATED_PROCESS_TIMEOUT_MS
            );
            // Terminate the hung process
            windows_sys::Win32::System::Threading::TerminateProcess(process_info.hProcess, 1);
            CloseHandle(process_info.hProcess);
            CloseHandle(process_info.hThread);
            return Err(Error::ServiceControl(format!(
                "SYSTEM command timed out after {}ms",
                ELEVATED_PROCESS_TIMEOUT_MS
            )));
        }

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
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // Use whoami /user to get the SID - cleaner than Windows API
    let output = Command::new("whoami")
        .args(["/user", "/fo", "csv", "/nh"])
        .creation_flags(CREATE_NO_WINDOW)
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

    // Validate inputs to prevent command injection
    validate_registry_path(key)?;
    validate_registry_path(value_name)?;

    // For HKCU, we need to use HKU\<SID> since SYSTEM's HKCU is different
    let full_key = if hive.eq_ignore_ascii_case("HKCU") {
        let sid = get_current_user_sid()?;
        log::debug!("Converting HKCU to HKU\\{}", sid);
        format!("HKU\\{}\\{}", sid, key)
    } else {
        format!("{}\\{}", hive, key)
    };

    // Escape shell arguments to prevent command injection
    let escaped_key = escape_shell_arg(&full_key);
    let escaped_value_name = escape_shell_arg(value_name);
    let escaped_value_data = escape_shell_arg(value_data);

    // Build reg.exe command with escaped arguments
    // reg add "HKLM\Software\..." /v "ValueName" /t REG_DWORD /d 1 /f
    let command = format!(
        "reg add \"{}\" /v \"{}\" /t {} /d {} /f",
        escaped_key, escaped_value_name, value_type, escaped_value_data
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

    // Validate inputs to prevent command injection
    validate_registry_path(key)?;
    validate_registry_path(value_name)?;

    // For HKCU, we need to use HKU\<SID> since SYSTEM's HKCU is different
    let full_key = if hive.eq_ignore_ascii_case("HKCU") {
        let sid = get_current_user_sid()?;
        format!("HKU\\{}\\{}", sid, key)
    } else {
        format!("{}\\{}", hive, key)
    };

    // Escape shell arguments to prevent command injection
    let escaped_key = escape_shell_arg(&full_key);
    let escaped_value_name = escape_shell_arg(value_name);

    let command = format!(
        "reg delete \"{}\" /v \"{}\" /f",
        escaped_key, escaped_value_name
    );

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

/// Execute an arbitrary command as SYSTEM
/// This is useful for running pre/post commands that need SYSTEM privileges
pub fn run_command_as_system(command: &str) -> Result<i32, Error> {
    log::info!("Running command as SYSTEM: {}", command);
    execute_command_as_system(command)
}

/// Set a Windows service startup type as SYSTEM using sc.exe
/// This bypasses normal permission checks for protected services
pub fn set_service_startup_as_system(service_name: &str, startup_type: &str) -> Result<(), Error> {
    log::info!(
        "Setting service '{}' startup to '{}' as SYSTEM",
        service_name,
        startup_type
    );

    let command = format!("sc config \"{}\" start= {}", service_name, startup_type);
    let exit_code = execute_command_as_system(&command)?;

    if exit_code == 0 {
        log::info!("Successfully set service startup as SYSTEM");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "sc config failed with exit code: {}",
            exit_code
        )))
    }
}

/// Stop a Windows service as SYSTEM using net stop
pub fn stop_service_as_system(service_name: &str) -> Result<(), Error> {
    log::info!("Stopping service '{}' as SYSTEM", service_name);

    let command = format!("net stop \"{}\"", service_name);
    let exit_code = execute_command_as_system(&command)?;

    // net stop returns 0 on success, 2 if already stopped
    if exit_code == 0 || exit_code == 2 {
        log::info!("Service stopped (or was already stopped) as SYSTEM");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "net stop failed with exit code: {}",
            exit_code
        )))
    }
}

/// Start a Windows service as SYSTEM using net start
pub fn start_service_as_system(service_name: &str) -> Result<(), Error> {
    log::info!("Starting service '{}' as SYSTEM", service_name);

    let command = format!("net start \"{}\"", service_name);
    let exit_code = execute_command_as_system(&command)?;

    // net start returns 0 on success, 2 if already running
    if exit_code == 0 || exit_code == 2 {
        log::info!("Service started (or was already running) as SYSTEM");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "net start failed with exit code: {}",
            exit_code
        )))
    }
}

// ============================================================================
// POWERSHELL EXECUTION
// ============================================================================

/// Result of a PowerShell command execution
#[derive(Debug)]
pub struct PowerShellResult {
    /// Exit code from PowerShell
    pub exit_code: i32,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Whether the command was successful (exit code 0)
    pub success: bool,
}

/// Execute a PowerShell command as the current user
/// Uses -NoProfile and -ExecutionPolicy Bypass for reliability
pub fn run_powershell(script: &str) -> Result<PowerShellResult, Error> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    log::info!("Running PowerShell command: {}", script);

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to execute PowerShell: {}", e)))?;

    let result = PowerShellResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    };

    if result.success {
        log::debug!("PowerShell command succeeded");
        if !result.stdout.is_empty() {
            log::trace!("PowerShell stdout: {}", result.stdout.trim());
        }
    } else {
        log::warn!(
            "PowerShell command failed with exit code {}: {}",
            result.exit_code,
            result.stderr.trim()
        );
    }

    Ok(result)
}

/// Execute a PowerShell command as SYSTEM
/// The command is wrapped and executed via CreateProcessWithTokenW
pub fn run_powershell_as_system(script: &str) -> Result<i32, Error> {
    log::info!("Running PowerShell command as SYSTEM: {}", script);

    // Escape double quotes in the script for the command line
    let escaped_script = script.replace('"', "\\\"");

    // Build the full command to run PowerShell as SYSTEM
    let command = format!(
        "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command \"{}\"",
        escaped_script
    );

    let exit_code = execute_command_as_system(&command)?;

    if exit_code == 0 {
        log::info!("PowerShell command as SYSTEM completed successfully");
    } else {
        log::warn!(
            "PowerShell command as SYSTEM failed with exit code: {}",
            exit_code
        );
    }

    Ok(exit_code)
}

/// Run a scheduled task command as SYSTEM (for protected tasks)
pub fn run_schtasks_as_system(args: &str) -> Result<i32, Error> {
    log::info!("Running schtasks as SYSTEM: {}", args);
    let command = format!("schtasks {}", args);
    execute_command_as_system(&command)
}

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
        let mut status_buffer = [0u8; std::mem::size_of::<SERVICE_STATUS_PROCESS>()];
        let mut bytes_needed: u32 = 0;

        let query_result = QueryServiceStatusEx(
            service,
            SC_STATUS_PROCESS_INFO,
            status_buffer.as_mut_ptr(),
            status_buffer.len() as u32,
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

        let status = &*(status_buffer.as_ptr() as *const SERVICE_STATUS_PROCESS);
        let current_state = status.dwCurrentState;

        // If already running, return the PID
        // SERVICE_RUNNING = 4
        if current_state == 4 {
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
            // ERROR_SERVICE_ALREADY_RUNNING = 1056
            if err != 1056 {
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

            let query_result = QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                status_buffer.as_mut_ptr(),
                status_buffer.len() as u32,
                &mut bytes_needed,
            );

            if query_result != 0 {
                let status = &*(status_buffer.as_ptr() as *const SERVICE_STATUS_PROCESS);
                if status.dwCurrentState == 4 {
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

/// Execute a command as TrustedInstaller using parent process spoofing
/// This creates a process with TrustedInstaller.exe as its parent,
/// causing it to inherit the TI token.
fn execute_command_as_trusted_installer(command_line: &str) -> Result<i32, Error> {
    log::info!("Executing command as TrustedInstaller: {}", command_line);

    let ti_handle = get_trusted_installer_handle()?;

    // Build command line: cmd.exe /c <command>
    let full_command = format!("cmd.exe /c {}", command_line);
    let mut command_wide = to_wide_string(&full_command);

    // SAFETY: Windows API calls for parent process spoofing. This creates a process
    // with TrustedInstaller.exe as parent, inheriting its privileges. All handles
    // and attribute lists are properly cleaned up. The command_wide and ti_handle_copy
    // remain valid throughout the CreateProcessW call.
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

        // Allocate memory for attribute list
        let attr_list_buffer = vec![0u8; attr_list_size];
        let attr_list = attr_list_buffer.as_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;

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

        // Wait for the process to complete (with timeout)
        let wait_result = windows_sys::Win32::System::Threading::WaitForSingleObject(
            process_info.hProcess,
            ELEVATED_PROCESS_TIMEOUT_MS,
        );

        // Check if we timed out (WAIT_TIMEOUT = 0x102)
        if wait_result == 0x102 {
            log::warn!(
                "TrustedInstaller command timed out after {}ms",
                ELEVATED_PROCESS_TIMEOUT_MS
            );
            // Terminate the hung process
            windows_sys::Win32::System::Threading::TerminateProcess(process_info.hProcess, 1);
            CloseHandle(process_info.hProcess);
            CloseHandle(process_info.hThread);
            return Err(Error::ServiceControl(format!(
                "TrustedInstaller command timed out after {}ms",
                ELEVATED_PROCESS_TIMEOUT_MS
            )));
        }

        // Get exit code
        let mut exit_code: u32 = 0;
        windows_sys::Win32::System::Threading::GetExitCodeProcess(
            process_info.hProcess,
            &mut exit_code,
        );

        CloseHandle(process_info.hProcess);
        CloseHandle(process_info.hThread);

        log::debug!(
            "TrustedInstaller command completed with exit code: {}",
            exit_code
        );
        Ok(exit_code as i32)
    }
}

/// Set a Windows service startup type as TrustedInstaller
/// This is needed for protected services like WaaSMedicSvc
pub fn set_service_startup_as_ti(service_name: &str, startup_type: &str) -> Result<(), Error> {
    log::info!(
        "Setting service '{}' startup to '{}' as TrustedInstaller",
        service_name,
        startup_type
    );

    let command = format!("sc config \"{}\" start= {}", service_name, startup_type);
    let exit_code = execute_command_as_trusted_installer(&command)?;

    if exit_code == 0 {
        log::info!("Successfully set service startup as TrustedInstaller");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "sc config failed with exit code: {}",
            exit_code
        )))
    }
}

/// Stop a Windows service as TrustedInstaller
pub fn stop_service_as_ti(service_name: &str) -> Result<(), Error> {
    log::info!("Stopping service '{}' as TrustedInstaller", service_name);

    let command = format!("net stop \"{}\"", service_name);
    let exit_code = execute_command_as_trusted_installer(&command)?;

    // net stop returns 0 on success, 2 if already stopped
    if exit_code == 0 || exit_code == 2 {
        log::info!("Service stopped (or was already stopped) as TrustedInstaller");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "net stop failed with exit code: {}",
            exit_code
        )))
    }
}

/// Start a Windows service as TrustedInstaller
pub fn start_service_as_ti(service_name: &str) -> Result<(), Error> {
    log::info!("Starting service '{}' as TrustedInstaller", service_name);

    let command = format!("net start \"{}\"", service_name);
    let exit_code = execute_command_as_trusted_installer(&command)?;

    // net start returns 0 on success, 2 if already running
    if exit_code == 0 || exit_code == 2 {
        log::info!("Service started (or was already running) as TrustedInstaller");
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "net start failed with exit code: {}",
            exit_code
        )))
    }
}

/// Run an arbitrary command as TrustedInstaller
pub fn run_command_as_ti(command: &str) -> Result<i32, Error> {
    log::info!("Running command as TrustedInstaller: {}", command);
    execute_command_as_trusted_installer(command)
}

/// Run a PowerShell command as TrustedInstaller
pub fn run_powershell_as_ti(script: &str) -> Result<i32, Error> {
    log::info!("Running PowerShell command as TrustedInstaller: {}", script);

    let escaped_script = script.replace('"', "\\\"");
    let command = format!(
        "powershell.exe -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -Command \"{}\"",
        escaped_script
    );

    execute_command_as_trusted_installer(&command)
}

/// Run schtasks as TrustedInstaller (for protected tasks)
pub fn run_schtasks_as_ti(args: &str) -> Result<i32, Error> {
    log::info!("Running schtasks as TrustedInstaller: {}", args);
    let command = format!("schtasks {}", args);
    execute_command_as_trusted_installer(&command)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // escape_shell_arg tests
    // ========================================================================

    #[test]
    fn test_escape_shell_arg_no_special_chars() {
        let result = escape_shell_arg("normaltext123");
        assert_eq!(result, "normaltext123");
    }

    #[test]
    fn test_escape_shell_arg_with_ampersand() {
        let result = escape_shell_arg("foo & bar");
        assert_eq!(result, "foo ^& bar");
    }

    #[test]
    fn test_escape_shell_arg_with_pipe() {
        let result = escape_shell_arg("foo | bar");
        assert_eq!(result, "foo ^| bar");
    }

    #[test]
    fn test_escape_shell_arg_with_angle_brackets() {
        let result = escape_shell_arg("foo < bar > baz");
        assert_eq!(result, "foo ^< bar ^> baz");
    }

    #[test]
    fn test_escape_shell_arg_with_caret() {
        let result = escape_shell_arg("foo ^ bar");
        assert_eq!(result, "foo ^^ bar");
    }

    #[test]
    fn test_escape_shell_arg_with_percent() {
        let result = escape_shell_arg("100%");
        assert_eq!(result, "100%%");
    }

    #[test]
    fn test_escape_shell_arg_with_double_quotes() {
        let result = escape_shell_arg("say \"hello\"");
        assert_eq!(result, "say \"\"hello\"\"");
    }

    #[test]
    fn test_escape_shell_arg_mixed_special_chars() {
        let result = escape_shell_arg("a & b | c > d");
        assert_eq!(result, "a ^& b ^| c ^> d");
    }

    // ========================================================================
    // validate_registry_path tests
    // ========================================================================

    #[test]
    fn test_validate_registry_path_valid() {
        let result = validate_registry_path("Software\\Microsoft\\Windows");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_registry_path_with_spaces() {
        let result = validate_registry_path("Software\\Microsoft\\Windows NT");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_registry_path_with_guid() {
        let result = validate_registry_path("Software\\{12345678-1234-1234-1234-123456789012}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_registry_path_with_underscore_hyphen() {
        let result = validate_registry_path("Software\\My_App-Name");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_registry_path_invalid_ampersand() {
        let result = validate_registry_path("Software\\Bad & Path");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_registry_path_invalid_pipe() {
        let result = validate_registry_path("Software\\Bad|Path");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_registry_path_invalid_semicolon() {
        let result = validate_registry_path("Software;Command");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_registry_path_invalid_angle_brackets() {
        let result = validate_registry_path("Software\\<script>");
        assert!(result.is_err());
    }
}
