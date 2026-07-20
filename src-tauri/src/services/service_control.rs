//! Windows Service Control Manager operations (typed, via `windows-sys`).
//!
//! Replaces the previous `sc.exe` / `net.exe` / `reg.exe` string-parsing implementation, which was
//! locale-dependent and swallowed real failures. State and control go through the SCM
//! (`QueryServiceStatusEx`, `ChangeServiceConfigW`, `StartServiceW`, `ControlService`); the startup
//! type is read from the service's typed `Start` registry value (a numeric DWORD — locale-free).
//!
//! The four public signatures (`get_service_status`, `set_service_startup`, `start_service`,
//! `stop_service`) are unchanged so callers do not move. `panic = "abort"` in release means `Drop`
//! does not run on a panic, but the `ScHandle` guard still covers the normal and `?`-early-return
//! paths — strictly better than manual `CloseServiceHandle`.

use crate::error::Error;
use crate::models::{RegistryHive, ServiceStartupType};
use crate::services::registry_service;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Services::{
    ChangeServiceConfigW, CloseServiceHandle, ControlService, EnumDependentServicesW,
    OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW, ENUM_SERVICE_STATUSW,
    SC_HANDLE, SC_STATUS_PROCESS_INFO, SERVICE_STATUS, SERVICE_STATUS_PROCESS,
};

// --- Win32 constants (stable ABI values; defined locally to avoid version-specific import churn) ---
const SC_MANAGER_CONNECT: u32 = 0x0001;
const SERVICE_QUERY_STATUS: u32 = 0x0004;
const SERVICE_CHANGE_CONFIG: u32 = 0x0002;
const SERVICE_START_ACCESS: u32 = 0x0010; // SERVICE_START
const SERVICE_STOP_ACCESS: u32 = 0x0020; // SERVICE_STOP
const SERVICE_ENUMERATE_DEPENDENTS: u32 = 0x0008;
const SERVICE_NO_CHANGE: u32 = 0xffff_ffff;
const SERVICE_CONTROL_STOP: u32 = 0x0000_0001;
const SERVICE_ACTIVE: u32 = 0x0000_0001;

// dwCurrentState values
const SVC_STOPPED: u32 = 1;
const SVC_START_PENDING: u32 = 2;
const SVC_STOP_PENDING: u32 = 3;
const SVC_RUNNING: u32 = 4;
const SVC_CONTINUE_PENDING: u32 = 5;
const SVC_PAUSE_PENDING: u32 = 6;
const SVC_PAUSED: u32 = 7;

// GetLastError codes
const ERROR_SERVICE_DOES_NOT_EXIST: u32 = 1060;
const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;
const ERROR_SERVICE_NOT_ACTIVE: u32 = 1062;
const ERROR_MORE_DATA: u32 = 234;

const STOP_TIMEOUT_MS: u128 = 30_000;
const START_TIMEOUT_MS: u128 = 30_000;

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

/// SAFETY: `p` must be a valid, NUL-terminated wide string for its whole length.
unsafe fn wide_to_string(p: *const u16) -> String {
    let mut len = 0usize;
    while *p.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
}

/// Open the SCM and a service handle. `Ok(None)` means the service does not exist; `Err` is a real
/// SCM/open failure. The returned SCM guard is kept alive alongside the service guard.
fn open_service(name: &str, access: u32) -> Result<Option<(ScHandle, ScHandle)>, Error> {
    // SAFETY: standard SCM open sequence; both handles are wrapped in RAII guards.
    unsafe {
        let scm = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(Error::ServiceControl(format!(
                "OpenSCManager failed: {}",
                GetLastError()
            )));
        }
        let scm = ScHandle(scm);

        let wname = wide(name);
        let svc = OpenServiceW(scm.0, wname.as_ptr(), access);
        if svc.is_null() {
            let err = GetLastError();
            if err == ERROR_SERVICE_DOES_NOT_EXIST {
                return Ok(None);
            }
            return Err(Error::ServiceControl(format!(
                "OpenService '{}' failed: {}",
                name, err
            )));
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
            return Err(Error::ServiceControl(format!(
                "QueryServiceStatusEx failed: {}",
                GetLastError()
            )));
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
        Error::ServiceControl(format!("Service does not exist: {}", service_name))
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
            return Err(Error::ServiceControl(format!(
                "Failed to set service '{}' startup: {}",
                service_name,
                GetLastError()
            )));
        }
    }

    log::debug!(
        "Successfully set service '{}' startup to {:?}",
        service_name,
        startup_type
    );
    Ok(())
}

/// Start a Windows service and wait until it is actually RUNNING.
///
/// `StartServiceW` only *queues* the start — the service enters START_PENDING and its return says
/// nothing about whether it reached RUNNING. Without a poll, a service that accepts the request then
/// fails async init is reported as success (finding B1). So, symmetrically with `stop_service`, we
/// poll to RUNNING; a fall-back to STOPPED or a timeout is a failure.
/// `ERROR_SERVICE_ALREADY_RUNNING` is idempotent success (and already running, so no poll needed).
pub fn start_service(service_name: &str) -> Result<(), Error> {
    let (_scm, svc) = open_service(service_name, SERVICE_START_ACCESS | SERVICE_QUERY_STATUS)?
        .ok_or_else(|| {
            Error::ServiceControl(format!("Service does not exist: {}", service_name))
        })?;

    log::info!("Starting service '{}'", service_name);

    // SAFETY: `svc` has SERVICE_START access; no start arguments.
    let already_running = unsafe {
        let ok = StartServiceW(svc.0, 0, ptr::null());
        if ok == 0 {
            let err = GetLastError();
            if err == ERROR_SERVICE_ALREADY_RUNNING {
                true
            } else {
                return Err(Error::ServiceControl(format!(
                    "Failed to start service '{}': {}",
                    service_name, err
                )));
            }
        } else {
            false
        }
    };

    if !already_running {
        wait_for_running(svc.0)?;
    }

    log::debug!("Service '{}' is running", service_name);
    Ok(())
}

/// Stop a Windows service. Stops active dependents first (as `net stop` did — `ControlService`
/// alone does not), treats an already-stopped service as success, and polls for `STOPPED`.
pub fn stop_service(service_name: &str) -> Result<(), Error> {
    let (_scm, svc) = open_service(
        service_name,
        SERVICE_STOP_ACCESS | SERVICE_QUERY_STATUS | SERVICE_ENUMERATE_DEPENDENTS,
    )?
    .ok_or_else(|| Error::ServiceControl(format!("Service does not exist: {}", service_name)))?;

    if query_current_state(svc.0)? == SVC_STOPPED {
        log::info!("Service '{}' is not running, skipping stop.", service_name);
        return Ok(());
    }

    log::info!("Stopping service '{}'", service_name);

    // Stop active dependents first (best-effort; the target stop below is authoritative).
    for dep in active_dependents(svc.0)? {
        if let Some((_s, dsvc)) = open_service(&dep, SERVICE_STOP_ACCESS | SERVICE_QUERY_STATUS)? {
            let _ = send_stop(dsvc.0);
            let _ = wait_for_stop(dsvc.0);
        }
    }

    send_stop(svc.0)?;
    wait_for_stop(svc.0)?;

    log::debug!("Service '{}' stopped", service_name);
    Ok(())
}

/// Send a STOP control. `ERROR_SERVICE_NOT_ACTIVE` is idempotent success.
fn send_stop(svc: SC_HANDLE) -> Result<(), Error> {
    // SAFETY: `svc` has SERVICE_STOP access; `status` is a zeroed out-param.
    unsafe {
        let mut status: SERVICE_STATUS = std::mem::zeroed();
        let ok = ControlService(svc, SERVICE_CONTROL_STOP, &mut status);
        if ok == 0 {
            let err = GetLastError();
            if err != ERROR_SERVICE_NOT_ACTIVE {
                return Err(Error::ServiceControl(format!(
                    "ControlService(STOP) failed: {}",
                    err
                )));
            }
        }
    }
    Ok(())
}

/// Poll until the service reports `STOPPED`, or time out.
fn wait_for_stop(svc: SC_HANDLE) -> Result<(), Error> {
    let start = std::time::Instant::now();
    loop {
        if query_current_state(svc)? == SVC_STOPPED {
            return Ok(());
        }
        if start.elapsed().as_millis() > STOP_TIMEOUT_MS {
            return Err(Error::ServiceControl(
                "Timed out waiting for service to stop".to_string(),
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

/// Classification of a `dwCurrentState` sample while waiting for a service to start.
#[derive(Debug, PartialEq, Eq)]
enum StartPoll {
    /// Reached RUNNING — the start succeeded.
    Running,
    /// Fell back to STOPPED — the start was accepted but the service failed to come up.
    Failed,
    /// START_PENDING or another transient state — keep polling.
    Pending,
}

fn classify_start_state(state: u32) -> StartPoll {
    match state {
        SVC_RUNNING => StartPoll::Running,
        SVC_STOPPED => StartPoll::Failed,
        _ => StartPoll::Pending,
    }
}

/// Poll until the service reports `RUNNING`; fail on a fall-back to `STOPPED` or a timeout.
fn wait_for_running(svc: SC_HANDLE) -> Result<(), Error> {
    let start = std::time::Instant::now();
    loop {
        match classify_start_state(query_current_state(svc)?) {
            StartPoll::Running => return Ok(()),
            StartPoll::Failed => {
                return Err(Error::ServiceControl(
                    "Service returned to STOPPED after a start request (start failed)".to_string(),
                ))
            }
            StartPoll::Pending => {}
        }
        if start.elapsed().as_millis() > START_TIMEOUT_MS {
            return Err(Error::ServiceControl(
                "Timed out waiting for service to start".to_string(),
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

/// Names of a service's currently-active dependent services.
fn active_dependents(svc: SC_HANDLE) -> Result<Vec<String>, Error> {
    // SAFETY: two-call sizing pattern; the second buffer is usize-aligned (>= pointer alignment)
    // and sized to `bytes_needed`, matching what the first call reported.
    unsafe {
        let mut bytes_needed: u32 = 0;
        let mut count: u32 = 0;

        let ok = EnumDependentServicesW(
            svc,
            SERVICE_ACTIVE,
            ptr::null_mut(),
            0,
            &mut bytes_needed,
            &mut count,
        );
        if ok != 0 {
            // Succeeded with a zero-size buffer => no dependents.
            return Ok(Vec::new());
        }
        let err = GetLastError();
        if err != ERROR_MORE_DATA {
            return Err(Error::ServiceControl(format!(
                "EnumDependentServices sizing failed: {}",
                err
            )));
        }
        if bytes_needed == 0 {
            return Ok(Vec::new());
        }

        let words = (bytes_needed as usize).div_ceil(std::mem::size_of::<usize>());
        let mut buf: Vec<usize> = vec![0usize; words];
        let entries = buf.as_mut_ptr() as *mut ENUM_SERVICE_STATUSW;

        let ok2 = EnumDependentServicesW(
            svc,
            SERVICE_ACTIVE,
            entries,
            bytes_needed,
            &mut bytes_needed,
            &mut count,
        );
        if ok2 == 0 {
            return Err(Error::ServiceControl(format!(
                "EnumDependentServices failed: {}",
                GetLastError()
            )));
        }

        let mut names = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let entry = &*entries.add(i);
            if !entry.lpServiceName.is_null() {
                names.push(wide_to_string(entry.lpServiceName as *const u16));
            }
        }
        Ok(names)
    }
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
    fn start_poll_treats_stopped_as_failure_and_running_as_success() {
        // B1 regression: StartServiceW only queues the start, so the poll must treat a fall-back to
        // STOPPED as a failed start (not a spurious success), RUNNING as success, and START_PENDING
        // as "keep waiting". (Starting a real service mutates the runner — the accepted gap — so the
        // poll *decision* is what we pin here.)
        assert_eq!(classify_start_state(SVC_RUNNING), StartPoll::Running);
        assert_eq!(classify_start_state(SVC_STOPPED), StartPoll::Failed);
        assert_eq!(classify_start_state(SVC_START_PENDING), StartPoll::Pending);
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
