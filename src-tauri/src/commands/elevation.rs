//! Elevation Commands
//!
//! Commands for elevating the application to TrustedInstaller privileges.

use crate::error::{Error, Result};
use crate::services::trusted_installer;
use tauri::AppHandle;

/// Restart the application with TrustedInstaller privileges
#[tauri::command]
pub async fn restart_as_trusted_installer(_app: AppHandle) -> Result<()> {
    log::info!("Command: restart_as_trusted_installer");

    // Check if running as admin first
    if !crate::services::system_info_service::is_running_as_admin() {
        log::warn!("Cannot elevate to TrustedInstaller without admin privileges");
        return Err(Error::RequiresAdmin);
    }

    trusted_installer::restart_as_trusted_installer()
}

/// Check if elevation to TrustedInstaller is possible
#[tauri::command]
pub async fn can_elevate_to_trusted_installer() -> Result<bool> {
    // Can only elevate if running as admin
    Ok(crate::services::system_info_service::is_running_as_admin())
}
