//! Windows Service Control Manager operations (typed, via `windows-sys`).
//!
//! Replaces the previous `sc.exe` / `net.exe` / `reg.exe` string-parsing implementation, which was
//! locale-dependent and swallowed real failures. State goes through the SCM
//! (`QueryServiceStatusEx`, `ChangeServiceConfigW`); the startup type is read from the service's
//! typed `Start` registry value (a numeric DWORD, so locale-free).
//!
//! `panic = "abort"` in release means `Drop` does not run on a panic, but the `ScHandle` guard
//! still covers the normal and `?`-early-return paths, strictly better than manual
//! `CloseServiceHandle`.

use crate::error::Error;
use crate::models::{RegistryHive, ServiceStartupType};
use crate::services::registry_service;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Services::{
    ChangeServiceConfigW, CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx,
    SC_HANDLE, SC_STATUS_PROCESS_INFO, SERVICE_STATUS_PROCESS,
};

// --- Win32 constants (stable ABI values; defined locally to avoid version-specific import churn) ---
const SC_MANAGER_CONNECT: u32 = 0x0001;
const SERVICE_QUERY_STATUS: u32 = 0x0004;
const SERVICE_CHANGE_CONFIG: u32 = 0x0002;
const SERVICE_NO_CHANGE: u32 = 0xffff_ffff;

// dwCurrentState values
const SVC_STOPPED: u32 = 1;
const SVC_START_PENDING: u32 = 2;
const SVC_STOP_PENDING: u32 = 3;
const SVC_RUNNING: u32 = 4;
const SVC_CONTINUE_PENDING: u32 = 5;
const SVC_PAUSE_PENDING: u32 = 6;
const SVC_PAUSED: u32 = 7;

const ERROR_SERVICE_DOES_NOT_EXIST: u32 = 1060;

/// Service running state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    StartPending,
    StopPending,
    Paused,
    PausePending,
    ContinuePending,
    Unknown,
}

/// Service status information.
#[derive(Debug, Clone)]
#[allow(dead_code)] // name field reserved for future use
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
    pub startup_type: Option<ServiceStartupType>,
    /// Whether the service exists in the Service Control Manager.
    pub exists: bool,
}

/// RAII guard that closes an `SC_HANDLE` on drop (normal and `?`-early-return paths).
struct ScHandle(SC_HANDLE);

impl Drop for ScHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: handle is non-null and owned by this guard.
            unsafe { CloseServiceHandle(self.0) };
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Open the SCM and a service handle. `Ok(None)` means the service does not exist; `Err` is a real
/// SCM/open failure. The returned SCM guard is kept alive alongside the service guard.
fn open_service(name: &str, access: u32) -> Result<Option<(ScHandle, ScHandle)>, Error> {
    // SAFETY: standard SCM open sequence; both handles are wrapped in RAII guards.
    unsafe {
        let scm = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(Error::win32("OpenSCManager failed", GetLastError()));
        }
        let scm = ScHandle(scm);

        let wname = wide(name);
        let svc = OpenServiceW(scm.0, wname.as_ptr(), access);
        if svc.is_null() {
            let err = GetLastError();
            if err == ERROR_SERVICE_DOES_NOT_EXIST {
                return Ok(None);
            }
            return Err(Error::win32(format!("OpenService '{name}' failed"), err));
        }
        Ok(Some((scm, ScHandle(svc))))
    }
}

fn state_from_dword(s: u32) -> ServiceState {
    match s {
        SVC_STOPPED => ServiceState::Stopped,
        SVC_START_PENDING => ServiceState::StartPending,
        SVC_STOP_PENDING => ServiceState::StopPending,
        SVC_RUNNING => ServiceState::Running,
        SVC_CONTINUE_PENDING => ServiceState::ContinuePending,
        SVC_PAUSE_PENDING => ServiceState::PausePending,
        SVC_PAUSED => ServiceState::Paused,
        _ => ServiceState::Unknown,
    }
}

fn start_type_dword(t: &ServiceStartupType) -> u32 {
    match t {
        ServiceStartupType::Boot => 0,
        ServiceStartupType::System => 1,
        ServiceStartupType::Automatic => 2,
        ServiceStartupType::Manual => 3,
        ServiceStartupType::Disabled => 4,
    }
}

/// Query a service's current `dwCurrentState` via `QueryServiceStatusEx`.
fn query_current_state(svc: SC_HANDLE) -> Result<u32, Error> {
    // SAFETY: `svc` is a valid service handle with SERVICE_QUERY_STATUS access; the buffer is a
    // correctly-sized, zeroed SERVICE_STATUS_PROCESS.
    unsafe {
        let mut status: SERVICE_STATUS_PROCESS = std::mem::zeroed();
        let mut needed: u32 = 0;
        let ok = QueryServiceStatusEx(
            svc,
            SC_STATUS_PROCESS_INFO,
            &mut status as *mut SERVICE_STATUS_PROCESS as *mut u8,
            std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
            &mut needed,
        );
        if ok == 0 {
            return Err(Error::win32("QueryServiceStatusEx failed", GetLastError()));
        }
        Ok(status.dwCurrentState)
    }
}

/// Read a service's startup type from its typed `Start` registry value (locale-free).
fn read_startup_type(service_name: &str) -> Option<ServiceStartupType> {
    let key = format!("System\\CurrentControlSet\\Services\\{}", service_name);
    match registry_service::read_dword(&RegistryHive::Hklm, &key, "Start") {
        Ok(Some(v)) => ServiceStartupType::from_registry_value(v),
        _ => None,
    }
}

/// Get the current status of a Windows service.
pub fn get_service_status(service_name: &str) -> Result<ServiceStatus, Error> {
    let (_scm, svc) = match open_service(service_name, SERVICE_QUERY_STATUS)? {
        None => {
            return Ok(ServiceStatus {
                name: service_name.to_string(),
                state: ServiceState::Unknown,
                startup_type: None,
                exists: false,
            })
        }
        Some(pair) => pair,
    };

    let state = state_from_dword(query_current_state(svc.0)?);
    let startup_type = read_startup_type(service_name);

    Ok(ServiceStatus {
        name: service_name.to_string(),
        state,
        startup_type,
        exists: true,
    })
}

/// Set the startup type of a Windows service.
pub fn set_service_startup(
    service_name: &str,
    startup_type: &ServiceStartupType,
) -> Result<(), Error> {
    // Preserve prior behavior: skip if already disabled.
    if matches!(startup_type, ServiceStartupType::Disabled) {
        if let Ok(true) = is_service_disabled(service_name) {
            log::info!(
                "Service '{}' is already disabled, skipping config.",
                service_name
            );
            return Ok(());
        }
    }

    let (_scm, svc) = open_service(service_name, SERVICE_CHANGE_CONFIG)?.ok_or_else(|| {
        Error::win32(
            format!("service '{service_name}' does not exist"),
            crate::error::win32::SERVICE_DOES_NOT_EXIST,
        )
    })?;

    log::info!(
        "Setting service '{}' startup to {:?}",
        service_name,
        startup_type
    );

    // SAFETY: `svc` has SERVICE_CHANGE_CONFIG access. SERVICE_NO_CHANGE leaves every field but the
    // start type untouched; all string params are NULL (not empty) to mean "unchanged".
    unsafe {
        let ok = ChangeServiceConfigW(
            svc.0,
            SERVICE_NO_CHANGE,
            start_type_dword(startup_type),
            SERVICE_NO_CHANGE,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
        );
        if ok == 0 {
            return Err(Error::win32(
                format!("failed to set service '{service_name}' startup"),
                GetLastError(),
            ));
        }
    }

    log::debug!(
        "Successfully set service '{}' startup to {:?}",
        service_name,
        startup_type
    );
    Ok(())
}

/// Check if a service is disabled.
pub fn is_service_disabled(service_name: &str) -> Result<bool, Error> {
    Ok(get_service_status(service_name)?.startup_type == Some(ServiceStartupType::Disabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_service_reports_absent() {
        let s = get_service_status("MagicXNoSuchService_zzq").unwrap();
        assert!(!s.exists);
        assert_eq!(s.state, ServiceState::Unknown);
        assert_eq!(s.startup_type, None);
    }

    #[test]
    fn known_service_reports_present_with_startup() {
        // "Schedule" (Task Scheduler) exists on every Windows edition.
        let s = get_service_status("Schedule").unwrap();
        assert!(s.exists);
        assert!(
            s.startup_type.is_some(),
            "expected a readable Start value for Schedule"
        );
    }

    #[test]
    fn start_type_dword_round_trips_through_from_registry_value() {
        for t in [
            ServiceStartupType::Boot,
            ServiceStartupType::System,
            ServiceStartupType::Automatic,
            ServiceStartupType::Manual,
            ServiceStartupType::Disabled,
        ] {
            let n = start_type_dword(&t);
            assert_eq!(ServiceStartupType::from_registry_value(n), Some(t));
        }
    }
}
