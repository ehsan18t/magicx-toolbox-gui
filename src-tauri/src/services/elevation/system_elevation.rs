//! SYSTEM Elevation Functions
//!
//! Execute commands with SYSTEM privileges by impersonating winlogon.exe.
//! Includes registry operations and service control.

use crate::error::Error;
use crate::models::{RegistryHive, RegistryValueType, ServiceStartupType};
use std::ptr;

use super::broker::{run_one, BrokerOp};
use super::Elevation;

use super::common::{
    enable_debug_privilege, find_process_by_name, get_process_token, to_wide_string, wait_and_reap,
    CloseHandle, CreateProcessWithTokenW, GetLastError, CREATE_NO_WINDOW, FALSE, HANDLE,
    LOGON_WITH_PROFILE, PROCESS_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW, SW_HIDE,
};

/// Get SYSTEM token from winlogon.exe
fn get_system_token() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let pid = find_process_by_name("winlogon.exe")?;
    log::debug!("Found winlogon.exe with PID: {}", pid);
    get_process_token(pid)
}

/// Spawn a raw command line as SYSTEM (no `cmd.exe` wrapper) and wait for it to complete.
/// Returns the exit code. This is the broker launcher; `execute_command_as_system` wraps a shell
/// command in `cmd.exe /c` and delegates here.
pub(super) fn spawn_as_system(command_line: &str) -> Result<i32, Error> {
    let token = get_system_token()?;
    log::debug!("Got SYSTEM token, spawning: {}", command_line);

    let mut command_wide = to_wide_string(command_line);

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

        wait_and_reap(&process_info, "SYSTEM command")
    }
}

/// Set a registry value as SYSTEM via the elevated broker (typed `RegSetValueExW`, no reg.exe).
/// The typed value crosses to the broker as data, dissolving the injection and REG_SZ-corruption
/// classes the old `reg add` + `escape_shell_arg` path carried.
pub fn set_registry_value_as_system(
    hive: RegistryHive,
    key: &str,
    value_name: &str,
    value_type: RegistryValueType,
    value: serde_json::Value,
) -> Result<(), Error> {
    run_one(
        Elevation::System,
        BrokerOp::RegSet {
            hive,
            key: key.to_string(),
            value_name: value_name.to_string(),
            value_type,
            value,
        },
    )
}

/// Delete a registry value as SYSTEM via the elevated broker (typed `RegDeleteValueW`, no reg.exe).
/// An absent value is reported as success by the broker.
pub fn delete_registry_value_as_system(
    hive: RegistryHive,
    key: &str,
    value_name: &str,
) -> Result<(), Error> {
    run_one(
        Elevation::System,
        BrokerOp::RegDeleteValue {
            hive,
            key: key.to_string(),
            value_name: value_name.to_string(),
        },
    )
}

/// Check if SYSTEM elevation is available (running as admin)
pub fn can_use_system_elevation() -> bool {
    crate::services::system_info_service::is_running_as_admin()
}

/// Execute an arbitrary command as SYSTEM (via the elevated broker; `cmd /c` inside it).
pub fn run_command_as_system(command: &str) -> Result<i32, Error> {
    log::info!("Running command as SYSTEM: {}", command);
    run_one(
        Elevation::System,
        BrokerOp::RawCmd {
            command: command.to_string(),
        },
    )
    .map(|()| 0)
}

/// Set a Windows service startup type as SYSTEM (typed `ChangeServiceConfigW`, via the broker).
pub fn set_service_startup_as_system(
    service_name: &str,
    startup: &ServiceStartupType,
) -> Result<(), Error> {
    run_one(
        Elevation::System,
        BrokerOp::SvcSetStartup {
            name: service_name.to_string(),
            startup: *startup,
        },
    )
}

/// Stop a Windows service as SYSTEM (typed `ControlService`, via the broker).
pub fn stop_service_as_system(service_name: &str) -> Result<(), Error> {
    run_one(
        Elevation::System,
        BrokerOp::SvcStop {
            name: service_name.to_string(),
        },
    )
}

/// Start a Windows service as SYSTEM (typed `StartServiceW`, via the broker).
pub fn start_service_as_system(service_name: &str) -> Result<(), Error> {
    run_one(
        Elevation::System,
        BrokerOp::SvcStart {
            name: service_name.to_string(),
        },
    )
}
