use crate::error::Error;
use crate::models::{SystemInfo, WindowsInfo};
use std::env;
use winreg::enums::*;
use winreg::RegKey;

/// Retrieve Windows version information
pub fn get_windows_info() -> Result<WindowsInfo, Error> {
    log::trace!("Reading Windows version info from registry");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    // Read product name
    let product_name: String = key
        .get_value("ProductName")
        .unwrap_or_else(|_| "Windows".to_string());

    // Read display version (e.g., "23H2")
    let display_version: String = key
        .get_value("DisplayVersion")
        .unwrap_or_else(|_| "".to_string());

    // Read build number
    let build_number: String = key
        .get_value("CurrentBuildNumber")
        .unwrap_or_else(|_| "19045".to_string());

    let build: u32 = build_number.parse().unwrap_or(19045);
    let is_windows_11 = build >= 22000;
    let version_string = if is_windows_11 {
        "11".to_string()
    } else {
        "10".to_string()
    };

    log::info!(
        "Detected Windows {} (build {}, {})",
        version_string,
        build_number,
        display_version
    );

    Ok(WindowsInfo {
        product_name,
        display_version,
        build_number,
        is_windows_11,
        version_string,
    })
}

/// Get full system information
pub fn get_system_info() -> Result<SystemInfo, Error> {
    log::debug!("Gathering system information");
    let windows = get_windows_info()?;
    let computer_name = env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let username = env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let is_admin = is_running_as_admin();

    log::debug!(
        "System info: computer={}, user={}, admin={}",
        computer_name,
        username,
        is_admin
    );

    Ok(SystemInfo {
        windows,
        computer_name,
        username,
        is_admin,
    })
}

/// Check if running as administrator
/// Uses a simple heuristic: try to open a protected registry key
pub fn is_running_as_admin() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    // Try to open SYSTEM key with write access - only admins can do this
    let is_admin = hklm
        .open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control", KEY_WRITE)
        .is_ok();
    log::trace!("Admin check: {}", is_admin);
    is_admin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_windows_info() {
        let result = get_windows_info();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.version_string == "10" || info.version_string == "11");
        assert!(!info.build_number.is_empty());
    }
}
