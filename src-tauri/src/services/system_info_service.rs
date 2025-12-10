use crate::error::Error;
use crate::models::{SystemInfo, WindowsInfo};
use std::env;
use winreg::enums::*;
use winreg::RegKey;

/// Retrieve Windows version information
pub fn get_windows_info() -> Result<WindowsInfo, Error> {
    let (version, build) = get_windows_version_and_build()?;
    let edition = get_windows_edition()?;
    let architecture = get_system_architecture();
    let is_admin = is_running_as_admin();

    Ok(WindowsInfo {
        version,
        build,
        edition,
        architecture,
        is_admin,
    })
}

/// Get full system information
pub fn get_system_info() -> Result<SystemInfo, Error> {
    let windows_info = get_windows_info()?;

    let total_ram_gb = None; // Can be enhanced later with WMI
    let processor_count = get_processor_count();

    Ok(SystemInfo {
        windows_info,
        total_ram_gb,
        processor_count,
    })
}

/// Get Windows version (10 or 11) and build number using winreg
fn get_windows_version_and_build() -> Result<(String, u32), Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    // Read CurrentBuildNumber
    let build_str: String = key
        .get_value("CurrentBuildNumber")
        .unwrap_or_else(|_| "19045".to_string());

    let build: u32 = build_str.parse().unwrap_or(19045);

    // Determine Windows version based on build number
    // Windows 11 starts at build 22000
    let version = if build >= 22000 {
        "11".to_string()
    } else {
        "10".to_string()
    };

    Ok((version, build))
}

/// Get Windows edition from registry
fn get_windows_edition() -> Result<String, Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    let edition: String = key
        .get_value("EditionID")
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(edition)
}

/// Get system architecture
fn get_system_architecture() -> String {
    // Check PROCESSOR_ARCHITECTURE environment variable
    env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "x64".to_string())
}

/// Check if running as administrator
/// Uses a simple heuristic: try to open a protected registry key
pub fn is_running_as_admin() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    // Try to open SYSTEM key with write access - only admins can do this
    hklm.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control", KEY_WRITE)
        .is_ok()
}

/// Get processor count from environment
fn get_processor_count() -> Option<u32> {
    env::var("NUMBER_OF_PROCESSORS")
        .ok()
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_windows_info() {
        let result = get_windows_info();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.version == "10" || info.version == "11");
        assert!(info.build > 0);
    }

    #[test]
    fn test_get_architecture() {
        let arch = get_system_architecture();
        assert!(!arch.is_empty());
    }
}
