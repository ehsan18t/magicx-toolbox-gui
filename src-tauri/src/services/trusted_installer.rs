//! TrustedInstaller Elevation Service
//!
//! Provides functionality to restart the application with TrustedInstaller privileges.
//! Uses the Scheduled Task approach since TrustedInstaller.exe is a Protected Process Light (PPL)
//! which prevents direct token access.

use crate::error::Error;
use std::process::Command;

/// Restart the current application as TrustedInstaller using a scheduled task
pub fn restart_as_trusted_installer() -> Result<(), Error> {
    log::info!("Attempting to restart as TrustedInstaller using scheduled task");

    // Get our executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| Error::ServiceControl(format!("Failed to get executable path: {}", e)))?;

    let exe_path_str = exe_path.to_string_lossy().to_string();
    let task_name = "MagicXToolboxElevation";

    // Create a scheduled task that runs as TrustedInstaller
    // The task runs immediately and then is deleted
    let create_result = Command::new("schtasks")
        .args([
            "/Create",
            "/TN",
            task_name,
            "/TR",
            &exe_path_str,
            "/SC",
            "ONCE",
            "/ST",
            "00:00",
            "/RU",
            "NT SERVICE\\TrustedInstaller",
            "/RL",
            "HIGHEST",
            "/F", // Force overwrite if exists
        ])
        .output()
        .map_err(|e| Error::ServiceControl(format!("Failed to create scheduled task: {}", e)))?;

    if !create_result.status.success() {
        let stderr = String::from_utf8_lossy(&create_result.stderr);
        log::error!("Failed to create scheduled task: {}", stderr);
        return Err(Error::ServiceControl(format!(
            "Failed to create scheduled task: {}",
            stderr
        )));
    }

    log::info!("Created scheduled task: {}", task_name);

    // Run the task immediately
    let run_result = Command::new("schtasks")
        .args(["/Run", "/TN", task_name])
        .output()
        .map_err(|e| Error::ServiceControl(format!("Failed to run scheduled task: {}", e)))?;

    if !run_result.status.success() {
        let stderr = String::from_utf8_lossy(&run_result.stderr);
        log::error!("Failed to run scheduled task: {}", stderr);

        // Clean up the task even if run failed
        let _ = Command::new("schtasks")
            .args(["/Delete", "/TN", task_name, "/F"])
            .output();

        return Err(Error::ServiceControl(format!(
            "Failed to run scheduled task: {}",
            stderr
        )));
    }

    log::info!("Scheduled task started successfully");

    // Delete the task (clean up)
    let delete_result = Command::new("schtasks")
        .args(["/Delete", "/TN", task_name, "/F"])
        .output();

    if let Err(e) = delete_result {
        log::warn!("Failed to delete scheduled task: {}", e);
    }

    log::info!("Successfully launched new process as TrustedInstaller via scheduled task");

    // Exit the current process
    std::process::exit(0);
}

/// Check if we might be running as TrustedInstaller (heuristic check)
#[allow(dead_code)]
pub fn is_elevated() -> bool {
    // For now, just check if we're admin - TrustedInstaller detection is complex
    crate::services::system_info_service::is_running_as_admin()
}
