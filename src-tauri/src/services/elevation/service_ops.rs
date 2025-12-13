//! Common Elevated Service Operations
//!
//! Generic helpers for service management that work with different elevation levels.
//! This reduces code duplication between SYSTEM and TrustedInstaller elevation modules.

use crate::error::Error;

use super::common::escape_shell_arg;

/// Elevation level for service operations
#[derive(Debug, Clone, Copy)]
pub enum ElevationLevel {
    /// Run as SYSTEM (via winlogon.exe token)
    System,
    /// Run as TrustedInstaller (via parent process spoofing)
    TrustedInstaller,
}

impl ElevationLevel {
    /// Get a human-readable label for logging
    pub fn label(&self) -> &'static str {
        match self {
            ElevationLevel::System => "SYSTEM",
            ElevationLevel::TrustedInstaller => "TrustedInstaller",
        }
    }
}

/// Type alias for command execution functions
pub type CommandExecutor = fn(&str) -> Result<i32, Error>;

/// Set a Windows service startup type using elevated privileges
///
/// # Arguments
/// * `service_name` - Name of the Windows service
/// * `startup_type` - SC startup type string (disabled, demand, auto, etc.)
/// * `elevation` - Which elevation level to use
/// * `execute` - The command execution function for the elevation level
pub fn set_service_startup_elevated(
    service_name: &str,
    startup_type: &str,
    elevation: ElevationLevel,
    execute: CommandExecutor,
) -> Result<(), Error> {
    log::info!(
        "Setting service '{}' startup to '{}' as {}",
        service_name,
        startup_type,
        elevation.label()
    );

    let escaped_name = escape_shell_arg(service_name);
    let command = format!("sc config \"{}\" start= {}", escaped_name, startup_type);
    let exit_code = execute(&command)?;

    if exit_code == 0 {
        log::info!("Successfully set service startup as {}", elevation.label());
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "sc config failed with exit code: {}",
            exit_code
        )))
    }
}

/// Stop a Windows service using elevated privileges
///
/// # Arguments
/// * `service_name` - Name of the Windows service
/// * `elevation` - Which elevation level to use
/// * `execute` - The command execution function for the elevation level
pub fn stop_service_elevated(
    service_name: &str,
    elevation: ElevationLevel,
    execute: CommandExecutor,
) -> Result<(), Error> {
    log::info!(
        "Stopping service '{}' as {}",
        service_name,
        elevation.label()
    );

    let escaped_name = escape_shell_arg(service_name);
    let command = format!("net stop \"{}\"", escaped_name);
    let exit_code = execute(&command)?;

    // net stop returns 0 on success, 2 if already stopped
    if exit_code == 0 || exit_code == 2 {
        log::info!(
            "Service stopped (or was already stopped) as {}",
            elevation.label()
        );
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "net stop failed with exit code: {}",
            exit_code
        )))
    }
}

/// Start a Windows service using elevated privileges
///
/// # Arguments
/// * `service_name` - Name of the Windows service
/// * `elevation` - Which elevation level to use
/// * `execute` - The command execution function for the elevation level
pub fn start_service_elevated(
    service_name: &str,
    elevation: ElevationLevel,
    execute: CommandExecutor,
) -> Result<(), Error> {
    log::info!(
        "Starting service '{}' as {}",
        service_name,
        elevation.label()
    );

    let escaped_name = escape_shell_arg(service_name);
    let command = format!("net start \"{}\"", escaped_name);
    let exit_code = execute(&command)?;

    // net start returns 0 on success, 2 if already running
    if exit_code == 0 || exit_code == 2 {
        log::info!(
            "Service started (or was already running) as {}",
            elevation.label()
        );
        Ok(())
    } else {
        Err(Error::ServiceControl(format!(
            "net start failed with exit code: {}",
            exit_code
        )))
    }
}
