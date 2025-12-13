//! TrustedInstaller Elevation Functions
//!
//! Execute commands with TrustedInstaller privileges using parent process spoofing.
//! Also includes PowerShell execution and scheduled task commands.

use crate::error::Error;
use std::ptr;

use super::common::{
    enable_debug_privilege, to_wide_string, CloseHandle, CloseServiceHandle, CreateProcessW,
    DeleteProcThreadAttributeList, GetLastError, InitializeProcThreadAttributeList, OpenProcess,
    OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW, UpdateProcThreadAttribute,
    CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, ELEVATED_PROCESS_TIMEOUT_MS,
    EXTENDED_STARTUPINFO_PRESENT, FALSE, HANDLE, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_CREATE_PROCESS, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PARENT_PROCESS,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STATUS_PROCESS, STARTF_USESHOWWINDOW, STARTUPINFOEXW, STARTUPINFOW, SW_HIDE,
};

// Re-export execute_command_as_system for use in run_powershell_as_system
use super::system_elevation::execute_command_as_system;

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
    const CREATE_NO_WINDOW_FLAG: u32 = 0x08000000;

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
        .creation_flags(CREATE_NO_WINDOW_FLAG)
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
pub fn execute_command_as_trusted_installer(command_line: &str) -> Result<i32, Error> {
    log::info!("Executing command as TrustedInstaller: {}", command_line);

    let ti_handle = get_trusted_installer_handle()?;

    // Build command line: cmd.exe /c <command>
    let full_command = format!("cmd.exe /c {}", command_line);
    let mut command_wide = to_wide_string(&full_command);

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
