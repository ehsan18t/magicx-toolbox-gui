//! Windows Service Control Manager operations
//!
//! Provides functions for managing Windows services:
//! - Getting service status and startup type
//! - Starting and stopping services
//! - Changing service startup type (disabled, manual, automatic)

use crate::error::Error;
use crate::models::ServiceStartupType;
use std::process::Command;

/// Service running state
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

impl ServiceState {
    fn from_sc_output(state_str: &str) -> Self {
        match state_str.trim().to_uppercase().as_str() {
            "RUNNING" => ServiceState::Running,
            "STOPPED" => ServiceState::Stopped,
            "START_PENDING" => ServiceState::StartPending,
            "STOP_PENDING" => ServiceState::StopPending,
            "PAUSED" => ServiceState::Paused,
            "PAUSE_PENDING" => ServiceState::PausePending,
            "CONTINUE_PENDING" => ServiceState::ContinuePending,
            _ => ServiceState::Unknown,
        }
    }
}

/// Service status information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
    pub startup_type: Option<ServiceStartupType>,
}

/// Get the current status of a Windows service
pub fn get_service_status(service_name: &str) -> Result<ServiceStatus, Error> {
    // Query service state using sc.exe
    let output = Command::new("sc")
        .args(["query", service_name])
        .output()
        .map_err(|e| {
            Error::ServiceControl(format!("Failed to query service '{}': {}", service_name, e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse state from output
    let state = parse_service_state(&stdout).unwrap_or(ServiceState::Unknown);

    // Get startup type from registry
    let startup_type = get_service_startup_type(service_name).ok();

    Ok(ServiceStatus {
        name: service_name.to_string(),
        state,
        startup_type,
    })
}

/// Parse service state from sc query output
fn parse_service_state(output: &str) -> Option<ServiceState> {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("STATE") {
            // Format: "STATE              : 4  RUNNING"
            if let Some(state_part) = line.split(':').nth(1) {
                // Extract the text state (e.g., "RUNNING" from "4  RUNNING")
                let parts: Vec<&str> = state_part.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(ServiceState::from_sc_output(parts[1]));
                }
            }
        }
    }
    None
}

/// Get the startup type of a service from registry
fn get_service_startup_type(service_name: &str) -> Result<ServiceStartupType, Error> {
    let key_path = format!("System\\CurrentControlSet\\Services\\{}", service_name);

    let output = Command::new("reg")
        .args(["query", &format!("HKLM\\{}", key_path), "/v", "Start"])
        .output()
        .map_err(|e| {
            Error::ServiceControl(format!("Failed to query service startup type: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse REG_DWORD value from output
    for line in stdout.lines() {
        if line.contains("Start") && line.contains("REG_DWORD") {
            if let Some(value_str) = line.split_whitespace().last() {
                // Parse hex value (e.g., "0x4")
                let value = if value_str.starts_with("0x") {
                    u32::from_str_radix(&value_str[2..], 16).ok()
                } else {
                    value_str.parse::<u32>().ok()
                };

                if let Some(v) = value {
                    return ServiceStartupType::from_registry_value(v).ok_or_else(|| {
                        Error::ServiceControl(format!("Unknown startup type value: {}", v))
                    });
                }
            }
        }
    }

    Err(Error::ServiceControl(format!(
        "Could not determine startup type for service '{}'",
        service_name
    )))
}

/// Set the startup type of a Windows service
pub fn set_service_startup(
    service_name: &str,
    startup_type: &ServiceStartupType,
) -> Result<(), Error> {
    let start_type = startup_type.to_sc_start_type();

    log::info!(
        "Setting service '{}' startup to '{}'",
        service_name,
        start_type
    );

    let output = Command::new("sc")
        .args(["config", service_name, &format!("start={}", start_type)])
        .output()
        .map_err(|e| {
            Error::ServiceControl(format!(
                "Failed to configure service '{}': {}",
                service_name, e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::ServiceControl(format!(
            "Failed to set service '{}' startup to '{}': {}",
            service_name, start_type, stderr
        )));
    }

    log::debug!(
        "Successfully set service '{}' startup to '{}'",
        service_name,
        start_type
    );
    Ok(())
}

/// Start a Windows service
pub fn start_service(service_name: &str) -> Result<(), Error> {
    log::info!("Starting service '{}'", service_name);

    let output = Command::new("net")
        .args(["start", service_name])
        .output()
        .map_err(|e| {
            Error::ServiceControl(format!("Failed to start service '{}': {}", service_name, e))
        })?;

    // net start returns success even if already running
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore "already started" errors
        if !stderr.contains("already been started") && !stderr.contains("2182") {
            return Err(Error::ServiceControl(format!(
                "Failed to start service '{}': {}",
                service_name, stderr
            )));
        }
    }

    log::debug!(
        "Service '{}' started (or was already running)",
        service_name
    );
    Ok(())
}

/// Stop a Windows service
pub fn stop_service(service_name: &str) -> Result<(), Error> {
    log::info!("Stopping service '{}'", service_name);

    let output = Command::new("net")
        .args(["stop", service_name])
        .output()
        .map_err(|e| {
            Error::ServiceControl(format!("Failed to stop service '{}': {}", service_name, e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore "not started" errors
        if !stderr.contains("not started") && !stderr.contains("3521") {
            return Err(Error::ServiceControl(format!(
                "Failed to stop service '{}': {}",
                service_name, stderr
            )));
        }
    }

    log::debug!(
        "Service '{}' stopped (or was already stopped)",
        service_name
    );
    Ok(())
}

/// Check if a service is currently running
#[allow(dead_code)]
pub fn is_service_running(service_name: &str) -> Result<bool, Error> {
    let status = get_service_status(service_name)?;
    Ok(status.state == ServiceState::Running)
}

/// Check if a service is disabled
#[allow(dead_code)]
pub fn is_service_disabled(service_name: &str) -> Result<bool, Error> {
    let status = get_service_status(service_name)?;
    Ok(status.startup_type == Some(ServiceStartupType::Disabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service_state_running() {
        let output = r#"
SERVICE_NAME: wuauserv
        TYPE               : 20  WIN32_SHARE_PROCESS
        STATE              : 4  RUNNING
                                (STOPPABLE, NOT_PAUSABLE, ACCEPTS_PRESHUTDOWN)
        WIN32_EXIT_CODE    : 0  (0x0)
        "#;
        assert_eq!(parse_service_state(output), Some(ServiceState::Running));
    }

    #[test]
    fn test_parse_service_state_stopped() {
        let output = r#"
SERVICE_NAME: wuauserv
        TYPE               : 20  WIN32_SHARE_PROCESS
        STATE              : 1  STOPPED
        WIN32_EXIT_CODE    : 0  (0x0)
        "#;
        assert_eq!(parse_service_state(output), Some(ServiceState::Stopped));
    }
}
