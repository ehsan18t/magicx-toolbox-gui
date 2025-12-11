use crate::error::Error;
use crate::models::{
    CpuInfo, GpuInfo, HardwareInfo, MemoryInfo, MotherboardInfo, SystemInfo, WindowsInfo,
};
use serde::Deserialize;
use std::env;
use winreg::enums::*;
use winreg::RegKey;
use wmi::WMIConnection;

// WMI query structs
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Processor")]
#[serde(rename_all = "PascalCase")]
struct Win32Processor {
    name: Option<String>,
    number_of_cores: Option<u32>,
    number_of_logical_processors: Option<u32>,
    architecture: Option<u16>,
    max_clock_speed: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_VideoController")]
#[serde(rename_all = "PascalCase")]
struct Win32VideoController {
    name: Option<String>,
    adapter_ram: Option<u64>,
    driver_version: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_PhysicalMemory")]
#[serde(rename_all = "PascalCase")]
struct Win32PhysicalMemory {
    capacity: Option<u64>,
    speed: Option<u32>,
    #[serde(rename = "SMBIOSMemoryType")]
    smbios_memory_type: Option<u16>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_BaseBoard")]
#[serde(rename_all = "PascalCase")]
struct Win32BaseBoard {
    manufacturer: Option<String>,
    product: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_BIOS")]
#[serde(rename_all = "PascalCase")]
struct Win32Bios {
    #[serde(rename = "SMBIOSBIOSVersion")]
    smbios_bios_version: Option<String>,
}

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
        .unwrap_or_else(|_| "0".to_string());

    // Windows 11 starts at build 22000. This is a stable, well-documented threshold.
    // Both Windows 10 and 11 report CurrentMajorVersionNumber=10, so we use build number
    // to distinguish them. This threshold is for released versions and is unlikely to change.
    let build: u32 = build_number.parse().unwrap_or(0);
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

/// Get hardware information using WMI queries
fn get_hardware_info() -> HardwareInfo {
    log::debug!("Gathering hardware information via WMI");

    // Initialize WMI connection
    let wmi_con = match WMIConnection::new() {
        Ok(con) => con,
        Err(e) => {
            log::warn!("Failed to create WMI connection: {}", e);
            return HardwareInfo::default();
        }
    };

    let cpu = get_cpu_info(&wmi_con);
    let gpu = get_gpu_info(&wmi_con);
    let memory = get_memory_info(&wmi_con);
    let motherboard = get_motherboard_info(&wmi_con);

    HardwareInfo {
        cpu,
        gpu,
        memory,
        motherboard,
    }
}

/// Get CPU information from WMI
fn get_cpu_info(wmi_con: &WMIConnection) -> CpuInfo {
    let query: Vec<Win32Processor> = match wmi_con.query() {
        Ok(results) => results,
        Err(e) => {
            log::warn!("Failed to query CPU info: {}", e);
            return CpuInfo::default();
        }
    };

    if let Some(cpu) = query.first() {
        let architecture = match cpu.architecture {
            Some(0) => "x86".to_string(),
            Some(9) => "x64".to_string(),
            Some(12) => "ARM64".to_string(),
            _ => "Unknown".to_string(),
        };

        CpuInfo {
            name: cpu.name.clone().unwrap_or_else(|| "Unknown".to_string()),
            cores: cpu.number_of_cores.unwrap_or(0),
            threads: cpu.number_of_logical_processors.unwrap_or(0),
            architecture,
            max_clock_mhz: cpu.max_clock_speed.unwrap_or(0),
        }
    } else {
        CpuInfo::default()
    }
}

/// Get GPU information from WMI
fn get_gpu_info(wmi_con: &WMIConnection) -> Vec<GpuInfo> {
    let query: Vec<Win32VideoController> = match wmi_con.query() {
        Ok(results) => results,
        Err(e) => {
            log::warn!("Failed to query GPU info: {}", e);
            return vec![];
        }
    };

    query
        .into_iter()
        .filter(|gpu| {
            // Filter out virtual/basic display adapters
            let name = gpu.name.as_deref().unwrap_or("");
            !name.to_lowercase().contains("basic")
                && !name.to_lowercase().contains("microsoft")
                && !name.is_empty()
        })
        .map(|gpu| {
            let memory_bytes = gpu.adapter_ram.unwrap_or(0);
            let memory_gb = memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

            GpuInfo {
                name: gpu.name.unwrap_or_else(|| "Unknown".to_string()),
                memory_gb: (memory_gb * 10.0).round() / 10.0, // Round to 1 decimal
                driver_version: gpu.driver_version.unwrap_or_else(|| "Unknown".to_string()),
            }
        })
        .collect()
}

/// Get memory information from WMI
fn get_memory_info(wmi_con: &WMIConnection) -> MemoryInfo {
    let query: Vec<Win32PhysicalMemory> = match wmi_con.query() {
        Ok(results) => results,
        Err(e) => {
            log::warn!("Failed to query memory info: {}", e);
            return MemoryInfo::default();
        }
    };

    if query.is_empty() {
        return MemoryInfo::default();
    }

    let total_bytes: u64 = query.iter().filter_map(|m| m.capacity).sum();
    let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let slots_used = query.len() as u32;

    // Get speed from first stick (they're usually the same)
    let speed_mhz = query.first().and_then(|m| m.speed).unwrap_or(0);

    // Convert SMBIOS memory type to readable string
    // Reference: https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.4.0.pdf
    let memory_type = query
        .first()
        .and_then(|m| m.smbios_memory_type)
        .map(|t| match t {
            20 => "DDR".to_string(),
            21 => "DDR2".to_string(),
            24 => "DDR3".to_string(),
            26 => "DDR4".to_string(),
            34 => "DDR5".to_string(),
            _ => format!("Type {}", t),
        })
        .unwrap_or_else(|| "Unknown".to_string());

    MemoryInfo {
        total_gb: (total_gb * 10.0).round() / 10.0,
        speed_mhz,
        memory_type,
        slots_used,
    }
}

/// Get motherboard information from WMI
fn get_motherboard_info(wmi_con: &WMIConnection) -> MotherboardInfo {
    // Get baseboard info
    let baseboard_query: Vec<Win32BaseBoard> = wmi_con.query().unwrap_or_default();

    let (manufacturer, product) = if let Some(board) = baseboard_query.first() {
        (
            board
                .manufacturer
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            board
                .product
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
        )
    } else {
        ("Unknown".to_string(), "Unknown".to_string())
    };

    // Get BIOS version
    let bios_query: Vec<Win32Bios> = wmi_con.query().unwrap_or_default();
    let bios_version = bios_query
        .first()
        .and_then(|b| b.smbios_bios_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    MotherboardInfo {
        manufacturer,
        product,
        bios_version,
    }
}

/// Get full system information
pub fn get_system_info() -> Result<SystemInfo, Error> {
    log::debug!("Gathering system information");
    let windows = get_windows_info()?;
    let computer_name = env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let username = env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let is_admin = is_running_as_admin();
    let hardware = get_hardware_info();

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
        hardware,
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
