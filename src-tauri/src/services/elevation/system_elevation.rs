//! SYSTEM Elevation Functions
//!
//! Execute commands with SYSTEM privileges by impersonating winlogon.exe.
//! Includes registry operations and service control.

use crate::error::Error;
use std::ptr;

use super::common::{
    enable_debug_privilege, escape_shell_arg, find_process_by_name, get_process_token,
    to_wide_string, validate_registry_path, CloseHandle, CreateProcessWithTokenW, GetLastError,
    CREATE_NO_WINDOW, ELEVATED_PROCESS_TIMEOUT_MS, FALSE, HANDLE, LOGON_WITH_PROFILE,
    PROCESS_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW, SW_HIDE,
};

/// Get SYSTEM token from winlogon.exe
fn get_system_token() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = find_process_by_name("winlogon.exe")?;
    log::debug!("Found winlogon.exe with PID: {}", pid);
    get_process_token(pid)
}

/// Execute a command as SYSTEM and wait for it to complete
/// Returns the exit code
pub fn execute_command_as_system(command_line: &str) -> Result<i32, Error> {
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

    const CREATE_NO_WINDOW_FLAG: u32 = 0x08000000;

    // Use whoami /user to get the SID - cleaner than Windows API
    let output = Command::new("whoami")
        .args(["/user", "/fo", "csv", "/nh"])
        .creation_flags(CREATE_NO_WINDOW_FLAG)
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
