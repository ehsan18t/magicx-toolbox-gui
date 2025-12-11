//! Elevation Commands
//!
//! Commands for SYSTEM elevation to modify protected registry keys
//! and restarting the app with admin privileges.

use crate::error::Result;
use crate::services::trusted_installer;

/// Apply a registry change as SYSTEM
#[tauri::command]
pub async fn apply_registry_as_system(
    hive: String,
    key: String,
    value_name: String,
    value_type: String,
    value_data: String,
) -> Result<()> {
    log::info!(
        "Command: apply_registry_as_system - {}\\{}\\{}",
        hive,
        key,
        value_name
    );

    trusted_installer::set_registry_value_as_system(
        &hive,
        &key,
        &value_name,
        &value_type,
        &value_data,
    )
}

/// Delete a registry value as SYSTEM
#[tauri::command]
pub async fn delete_registry_as_system(
    hive: String,
    key: String,
    value_name: String,
) -> Result<()> {
    log::info!(
        "Command: delete_registry_as_system - {}\\{}\\{}",
        hive,
        key,
        value_name
    );

    trusted_installer::delete_registry_value_as_system(&hive, &key, &value_name)
}

/// Check if SYSTEM elevation is available (i.e., running as admin)
#[tauri::command]
pub async fn can_use_system_elevation() -> Result<bool> {
    Ok(trusted_installer::can_use_system_elevation())
}

/// Restart the application with administrator privileges
/// Uses ShellExecuteW with "runas" verb to trigger UAC prompt
#[tauri::command]
pub async fn restart_as_admin(app: tauri::AppHandle) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    log::info!("Restart as admin requested");

    // Get current executable path
    let exe_path = std::env::current_exe().map_err(|e| {
        crate::error::Error::WindowsApi(format!("Failed to get executable path: {}", e))
    })?;

    let exe_path_wide: Vec<u16> = OsStr::new(&exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let runas: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let result = ShellExecuteW(
            ptr::null_mut(),
            runas.as_ptr(),
            exe_path_wide.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        );

        // ShellExecuteW returns a value > 32 on success
        if result as usize <= 32 {
            return Err(crate::error::Error::WindowsApi(format!(
                "Failed to restart as admin, error code: {}",
                result as usize
            )));
        }
    }

    log::info!("New admin instance started, exiting current instance");

    // Exit the current (non-admin) instance
    app.exit(0);

    Ok(())
}
