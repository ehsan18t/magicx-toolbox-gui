use crate::error::Error;
use crate::models::{SystemInfo, WindowsInfo};
use std::ffi::CString;
use std::mem;
use windows::Win32::Foundation::*;
use windows::Win32::System::Registry::*;
use windows::Win32::System::SystemInformation::*;

/// Retrieve Windows version information
pub fn get_windows_info() -> Result<WindowsInfo, Error> {
    let (version, build) = get_windows_version_and_build()?;
    let edition = get_windows_edition()?;
    let architecture = get_system_architecture()?;
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

    let total_ram_gb = get_total_memory_gb();
    let processor_count = get_processor_count();

    Ok(SystemInfo {
        windows_info,
        total_ram_gb,
        processor_count,
    })
}

/// Get Windows version (10 or 11) and build number
fn get_windows_version_and_build() -> Result<(String, u32), Error> {
    unsafe {
        // Read from registry: HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion
        let hklm = RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            CString::new("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
                .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?
                .as_ptr() as *const u8,
            0,
            KEY_READ,
            &mut std::ptr::null_mut(),
        )
        .map_err(|_| Error::WindowsApi("Failed to open registry key".to_string()))?;

        let (major, minor, build) = get_os_version_info()?;

        // Determine version based on major/minor/build
        let version = if major == 10 {
            if build >= 22000 {
                "11".to_string()
            } else {
                "10".to_string()
            }
        } else {
            "10".to_string()
        };

        Ok((version, build))
    }
}

/// Get OS version info using Windows API
#[cfg(target_os = "windows")]
fn get_os_version_info() -> Result<(u32, u32, u32), Error> {
    // Try Windows 10+ version info first
    if let Ok(version_str) = read_registry_string(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        "CurrentVersion",
    ) {
        // For Windows 10+, we need to read the ReleaseId and CurrentBuildNumber
        let build_str = read_registry_string(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentBuildNumber",
        )
        .unwrap_or_else(|_| "19045".to_string());

        if let Ok(build) = build_str.parse::<u32>() {
            return Ok((10, 0, build));
        }
    }

    // Fallback
    Ok((10, 0, 19045))
}

/// Read a string value from registry
fn read_registry_string(hive: HKEY, key_path: &str, value_name: &str) -> Result<String, Error> {
    unsafe {
        let key_cstr =
            CString::new(key_path).map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;
        let value_cstr =
            CString::new(value_name).map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

        let mut hkey = std::ptr::null_mut();
        RegOpenKeyExA(hive, key_cstr.as_ptr() as *const u8, 0, KEY_READ, &mut hkey)
            .map_err(|_| Error::WindowsApi("Failed to open registry".to_string()))?;

        let mut value_len = 1024u32;
        let mut buffer = vec![0u8; value_len as usize];

        let result = RegQueryValueExA(
            hkey,
            value_cstr.as_ptr() as *const u8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buffer.as_mut_ptr(),
            &mut value_len,
        );

        RegCloseKey(hkey);

        result
            .ok()
            .map_err(|_| Error::WindowsApi("Failed to read registry value".to_string()))?;

        buffer.truncate(value_len as usize);
        String::from_utf8(buffer).map_err(|e| Error::WindowsApi(e.to_string()))
    }
}

/// Get Windows edition from registry
fn get_windows_edition() -> Result<String, Error> {
    read_registry_string(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        "EditionID",
    )
    .or_else(|_| Ok("Unknown".to_string()))
}

/// Get system architecture
fn get_system_architecture() -> Result<String, Error> {
    Ok("x64".to_string()) // Most Windows 10/11 systems are x64
}

/// Check if running as administrator
pub fn is_running_as_admin() -> bool {
    unsafe {
        use windows::Win32::Security::*;

        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = 0u32;
        let mut size = mem::size_of::<u32>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );

        CloseHandle(token);

        result.is_ok() && elevation != 0
    }
}

/// Get total system memory in GB
fn get_total_memory_gb() -> Option<u32> {
    // This would require WMI or other system APIs
    // For now, return None - can be enhanced later
    None
}

/// Get processor count
fn get_processor_count() -> Option<u32> {
    unsafe {
        let mut system_info = mem::zeroed();
        GetSystemInfo(&mut system_info);
        Some(system_info.dwNumberOfProcessors)
    }
}
