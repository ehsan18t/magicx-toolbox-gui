use crate::error::Error;
use crate::models::{
    CpuInfo, DeviceInfo, DiskInfo, GpuInfo, HardwareInfo, MemoryInfo, MotherboardInfo, SystemInfo,
    WindowsInfo,
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
    video_processor: Option<String>,
    current_refresh_rate: Option<u32>,
    video_mode_description: Option<String>,
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

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_DiskDrive")]
#[serde(rename_all = "PascalCase")]
struct Win32DiskDrive {
    model: Option<String>,
    size: Option<String>,
    media_type: Option<String>,
    interface_type: Option<String>,
}

/// MSFT_PhysicalDisk from storage namespace for reliable SSD/HDD detection
#[derive(Deserialize, Debug)]
#[serde(rename = "MSFT_PhysicalDisk")]
#[serde(rename_all = "PascalCase")]
struct MsftPhysicalDisk {
    friendly_name: Option<String>,
    size: Option<u64>,
    media_type: Option<u16>,    // 0=Unspecified, 3=HDD, 4=SSD, 5=SCM
    bus_type: Option<u16>,      // 11=SATA, 17=NVMe
    health_status: Option<u16>, // 0=Healthy, 1=Warning, 2=Unhealthy
}

/// Win32_OperatingSystem for uptime, install date, and name
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_OperatingSystem")]
#[serde(rename_all = "PascalCase")]
struct Win32OperatingSystem {
    caption: Option<String>,
    last_boot_up_time: Option<String>,
    install_date: Option<String>,
}

/// Win32_ComputerSystem for device manufacturer/model
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystem")]
#[serde(rename_all = "PascalCase")]
struct Win32ComputerSystem {
    manufacturer: Option<String>,
    model: Option<String>,
    system_type: Option<String>,
    #[serde(rename = "PCSystemType")]
    pc_system_type: Option<u16>, // 1=Desktop, 2=Mobile, 3=Workstation, etc.
}

/// Retrieve Windows version information
pub fn get_windows_info() -> Result<WindowsInfo, Error> {
    log::trace!("Reading Windows version info from registry");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    // Read product name (Legacy/Fallback)
    let registry_product_name: String = key
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

    let build: u32 = build_number.parse().unwrap_or(0);
    // Is Windows 11? (Build >= 22000)
    let is_windows_11 = build >= 22000;
    let version_string = if is_windows_11 {
        "11".to_string()
    } else {
        "10".to_string()
    };

    // Get uptime, install date, and caption from WMI
    let (uptime_seconds, install_date, os_caption) = get_os_info();

    // Use WMI caption as product name (more accurate for LTSC/IoT) or fallback to registry
    let mut product_name = os_caption.unwrap_or(registry_product_name);

    if product_name.starts_with("Microsoft ") {
        product_name = product_name.replacen("Microsoft ", "", 1);
    }

    log::info!(
        "Detected Windows {} (build {}, {}), uptime={}s",
        version_string,
        build_number,
        display_version,
        uptime_seconds
    );

    Ok(WindowsInfo {
        product_name,
        display_version,
        build_number,
        is_windows_11,
        version_string,
        uptime_seconds,
        install_date,
    })
}

/// Get uptime, install date, and caption from Win32_OperatingSystem
fn get_os_info() -> (u64, Option<String>, Option<String>) {
    let wmi_con = match WMIConnection::new() {
        Ok(con) => con,
        Err(e) => {
            log::warn!("Failed to create WMI connection for OS info: {}", e);
            return (0, None, None);
        }
    };

    let query: Vec<Win32OperatingSystem> = wmi_con.query().unwrap_or_default();
    if let Some(os) = query.first() {
        // Parse WMI datetime format: "20240115123456.000000+000"
        let uptime_seconds = os
            .last_boot_up_time
            .as_ref()
            .map(|boot_time| parse_wmi_datetime_to_uptime(boot_time))
            .unwrap_or(0);

        // Convert install date to ISO 8601
        let install_date = os
            .install_date
            .as_ref()
            .map(|d| parse_wmi_datetime_to_iso(d));

        let caption = os.caption.clone();

        (uptime_seconds, install_date, caption)
    } else {
        (0, None, None)
    }
}

/// Parse WMI datetime format to uptime in seconds
fn parse_wmi_datetime_to_uptime(wmi_datetime: &str) -> u64 {
    // WMI format: "20240115123456.123456+000"
    // Extract: YYYYMMDDHHMMSS
    if wmi_datetime.len() < 14 {
        return 0;
    }

    let year: i32 = wmi_datetime[0..4].parse().unwrap_or(0);
    let month: u32 = wmi_datetime[4..6].parse().unwrap_or(1);
    let day: u32 = wmi_datetime[6..8].parse().unwrap_or(1);
    let hour: u32 = wmi_datetime[8..10].parse().unwrap_or(0);
    let min: u32 = wmi_datetime[10..12].parse().unwrap_or(0);
    let sec: u32 = wmi_datetime[12..14].parse().unwrap_or(0);

    // Calculate seconds since boot using simple date arithmetic
    use std::time::{SystemTime, UNIX_EPOCH};

    // Convert boot time to approximate Unix timestamp
    // This is a simplified calculation - for display purposes only
    let days_since_epoch = (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100
        + (year - 1601) / 400
        + days_before_month(month, is_leap_year(year))
        + day as i32
        - 1;
    let boot_secs =
        days_since_epoch as u64 * 86400 + hour as u64 * 3600 + min as u64 * 60 + sec as u64;

    // Get current time
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    now_secs.saturating_sub(boot_secs)
}

fn days_before_month(month: u32, leap: bool) -> i32 {
    let days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let base = days
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(0);
    if leap && month > 2 {
        base + 1
    } else {
        base
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Parse WMI datetime to ISO 8601 format
fn parse_wmi_datetime_to_iso(wmi_datetime: &str) -> String {
    if wmi_datetime.len() < 14 {
        return wmi_datetime.to_string();
    }
    format!(
        "{}-{}-{}T{}:{}:{}",
        &wmi_datetime[0..4],
        &wmi_datetime[4..6],
        &wmi_datetime[6..8],
        &wmi_datetime[8..10],
        &wmi_datetime[10..12],
        &wmi_datetime[12..14]
    )
}

/// Get hardware information using WMI queries (parallelized with connection reuse)
/// Uses 3 threads instead of 7 to reduce COM initialization overhead:
/// - Thread 1: Fast cimv2 queries (CPU, Memory, Motherboard, Network) - same connection
/// - Thread 2: Slow cimv2 queries (GPU, Monitors) - WinAPI/registry intensive
/// - Thread 3: Storage namespace queries (Disks) - different WMI namespace
fn get_hardware_info() -> HardwareInfo {
    log::debug!("Gathering hardware information via WMI (3-thread hybrid)");

    use std::thread;
    let start = std::time::Instant::now();

    // Run WMI queries in parallel using scoped threads with connection reuse
    let (cpu, memory, motherboard, network, gpu, monitors, disks) = thread::scope(|s| {
        // Thread 1: Fast cimv2 queries - reuse single connection for 4 queries
        let fast_cimv2_handle = s.spawn(|| match WMIConnection::new() {
            Ok(con) => (
                get_cpu_info(&con),
                get_memory_info(&con),
                get_motherboard_info(&con),
                get_network_info(&con),
            ),
            Err(e) => {
                log::debug!("WMI connection failed for fast cimv2 queries: {}", e);
                (
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                )
            }
        });

        // Thread 2: Slow cimv2 queries (GPU + Monitors) - WinAPI/registry heavy
        let slow_cimv2_handle = s.spawn(|| match WMIConnection::new() {
            Ok(con) => (get_gpu_info(&con), get_monitor_info(&con)),
            Err(e) => {
                log::debug!("WMI connection failed for slow cimv2 queries: {}", e);
                (Default::default(), Default::default())
            }
        });

        // Thread 3: Storage namespace - uses different WMI namespace internally
        let storage_handle = s.spawn(|| match WMIConnection::new() {
            Ok(con) => get_disk_info(&con),
            Err(e) => {
                log::debug!("WMI connection failed for disk info: {}", e);
                Default::default()
            }
        });

        // Wait for all threads to complete
        let (cpu, memory, motherboard, network) = fast_cimv2_handle.join().unwrap_or_default();
        let (gpu, monitors) = slow_cimv2_handle.join().unwrap_or_default();
        let disks = storage_handle.join().unwrap_or_default();

        (cpu, memory, motherboard, network, gpu, monitors, disks)
    });

    log::debug!("Hardware info gathered in {:?}", start.elapsed());

    // Calculate total storage
    let total_storage_gb: f64 = disks.iter().map(|d| d.size_gb).sum();

    HardwareInfo {
        cpu,
        gpu,
        monitors,
        memory,
        motherboard,
        disks,
        network,
        total_storage_gb,
    }
}

use std::mem::size_of;
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsExW, DEVMODEW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, ENUM_CURRENT_SETTINGS,
};

#[derive(Deserialize, Debug)]
#[serde(rename = "WmiMonitorID")]
#[serde(rename_all = "PascalCase")]
struct WmiMonitorID {
    user_friendly_name: Option<Vec<u16>>,
    instance_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_PnPEntity")]
#[serde(rename_all = "PascalCase")]
struct Win32PnPEntity {
    name: Option<String>,
    device_id: Option<String>,
    class_guid: Option<String>,
}

/// Get monitor information using WinAPI (EnumDisplayDevices/EnumDisplaySettings) + WMI
fn get_monitor_info(_wmi_con: &WMIConnection) -> Vec<crate::models::MonitorInfo> {
    log::debug!("Gathering monitor info via Nested EnumDisplayDevices + WMI + PnP");

    let monitor_names = get_all_monitor_names();
    let mut monitors = Vec::new();
    let mut adapter_index = 0;

    // SAFETY: Windows EnumDisplayDevicesW and EnumDisplaySettingsW calls.
    // Structures are zeroed before use and properly sized. Loop terminates when
    // EnumDisplayDevicesW returns false, indicating end of device list.
    unsafe {
        loop {
            let mut adapter_device: DISPLAY_DEVICEW = std::mem::zeroed();
            adapter_device.cb = size_of::<DISPLAY_DEVICEW>() as u32;

            // Enum Adapter
            if EnumDisplayDevicesW(std::ptr::null(), adapter_index, &mut adapter_device, 0) == 0 {
                // If we've checked a reasonable number of adapters and they are failing, break.
                // But usually 0 returns false means end of list.
                break;
            }

            let device_name_raw = &adapter_device.DeviceName;
            let len = device_name_raw
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(device_name_raw.len());
            let device_name = String::from_utf16_lossy(&device_name_raw[0..len]);
            let state_flags = adapter_device.StateFlags;

            // Log adapter info
            log::debug!(
                "Adapter {}: {} (Flags: 0x{:X})",
                adapter_index,
                device_name,
                state_flags
            );

            if (state_flags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0 {
                // Get Settings for this Adapter
                let mut dev_mode: DEVMODEW = std::mem::zeroed();
                dev_mode.dmSize = size_of::<DEVMODEW>() as u16;
                // We use EnumDisplaySettingsExW to get the current mode for this adapter.
                // Note: If multiple monitors are on one adapter, they might share this or have specific modes?
                // Typically Extended Desktop = Separate Adapters (Sources).
                let mut resolution = "Unknown".to_string();
                let mut refresh_rate = 60;

                if EnumDisplaySettingsExW(
                    adapter_device.DeviceName.as_ptr(),
                    ENUM_CURRENT_SETTINGS,
                    &mut dev_mode,
                    0,
                ) != 0
                {
                    let width = dev_mode.dmPelsWidth;
                    let height = dev_mode.dmPelsHeight;
                    refresh_rate = dev_mode.dmDisplayFrequency;
                    resolution = format!("{}x{}", width, height);
                }

                // Inner Loop: Enum Monitors on this Adapter
                let mut monitor_index = 0;
                loop {
                    let mut monitor_device: DISPLAY_DEVICEW = std::mem::zeroed();
                    monitor_device.cb = size_of::<DISPLAY_DEVICEW>() as u32;

                    if EnumDisplayDevicesW(
                        adapter_device.DeviceName.as_ptr(),
                        monitor_index,
                        &mut monitor_device,
                        0,
                    ) == 0
                    {
                        break;
                    }

                    // Found a monitor on this adapter
                    let mon_id_raw = &monitor_device.DeviceID;
                    let len = mon_id_raw
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(mon_id_raw.len());
                    let device_id = String::from_utf16_lossy(&mon_id_raw[0..len]);
                    let mon_flags = monitor_device.StateFlags;

                    log::debug!(
                        "  -> Monitor {}: {} (Flags: 0x{:X})",
                        monitor_index,
                        device_id,
                        mon_flags
                    );

                    if (mon_flags & DISPLAY_DEVICE_ACTIVE) != 0 {
                        let mut name = "Generic Monitor".to_string();
                        let hardware_id = device_id.split('\\').nth(1).unwrap_or("");

                        // Match Name
                        if let Some(wmi_match) = monitor_names.iter().find_map(|(k, v)| {
                            if (!hardware_id.is_empty() && k.contains(hardware_id))
                                || k.contains(&device_id)
                                || device_id.contains(k)
                            {
                                Some(v.clone())
                            } else {
                                None
                            }
                        }) {
                            name = wmi_match;
                        } else {
                            // Fallback
                            let device_string_raw = &monitor_device.DeviceString;
                            let len = device_string_raw
                                .iter()
                                .position(|&c| c == 0)
                                .unwrap_or(device_string_raw.len());
                            let monitor_name_w = &device_string_raw[0..len];
                            let monitor_name = String::from_utf16_lossy(monitor_name_w);
                            if !monitor_name.trim().is_empty() {
                                name = monitor_name;
                            }
                        }

                        log::debug!("  -> Added: {} - {} @ {}Hz", name, resolution, refresh_rate);
                        monitors.push(crate::models::MonitorInfo {
                            name: name.clone(),
                            resolution: resolution.clone(),
                            refresh_rate,
                        });
                    } else {
                        log::debug!("  -> Monitor is not active (StateFlags & DISPLAY_DEVICE_ACTIVE == 0). Skipping.");
                    }

                    monitor_index += 1;
                }

                if monitor_index == 0 {
                    log::debug!(
                        "  -> Active Adapter but no monitors enumerated via EnumDisplayDevices?"
                    );
                }
            }

            adapter_index += 1;
        }
    }

    monitors
}

/// Get map of unique ID -> Friendly Name from WmiMonitorID and Win32_PnPEntity
fn get_all_monitor_names() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    let wmi_con = match WMIConnection::new() {
        Ok(con) => con,
        Err(_) => return map,
    };

    // 1. Try WmiMonitorID (Best source for Model Names via EDID)
    if let Ok(wmi_monitor_con) = WMIConnection::with_namespace_path("root\\wmi") {
        let query: Vec<WmiMonitorID> = wmi_monitor_con.query().unwrap_or_default();
        for mon in query {
            if let Some(raw) = mon.user_friendly_name {
                let chars: Vec<u16> = raw.into_iter().filter(|&c| c != 0).collect();
                let name = String::from_utf16_lossy(&chars);
                if !name.trim().is_empty() {
                    map.insert(mon.instance_name, name);
                }
            }
        }
    }

    // 2. Try Win32_PnPEntity as fallback (e.g. "Integrated Monitor")
    // Filter for Monitor class GUID: {4d36e96e-e325-11ce-bfc1-08002be10318}
    let pnp_query: Vec<Win32PnPEntity> = wmi_con.query().unwrap_or_default();
    for pnp in pnp_query {
        if let (Some(class_guid), Some(dev_id), Some(name)) =
            (pnp.class_guid, pnp.device_id, pnp.name)
        {
            if class_guid.eq_ignore_ascii_case("{4d36e96e-e325-11ce-bfc1-08002be10318}") {
                // Only add if not already present (WmiMonitorID is better)
                // We use check against map keys containing the hardware ID
                // But PnPEntity DeviceID IS the InstanceID usually.
                map.entry(dev_id).or_insert(name);
            }
        }
    }

    map
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_NetworkAdapterConfiguration")]
#[serde(rename_all = "PascalCase")]
struct Win32NetworkAdapterConfiguration {
    description: Option<String>,
    #[serde(rename = "MACAddress")]
    mac_address: Option<String>,
    #[serde(rename = "IPAddress")]
    ip_address: Option<Vec<String>>,
    #[serde(rename = "IPEnabled")]
    ip_enabled: Option<bool>,
    #[serde(rename = "DHCPEnabled")]
    dhcp_enabled: Option<bool>,
}

/// Get network information from WMI
fn get_network_info(wmi_con: &WMIConnection) -> Vec<crate::models::NetworkInfo> {
    let query: Vec<Win32NetworkAdapterConfiguration> = match wmi_con.query() {
        Ok(results) => results,
        Err(e) => {
            log::warn!("Failed to query network info: {}", e);
            return vec![];
        }
    };

    query
        .into_iter()
        .filter(|adapter| adapter.ip_enabled.unwrap_or(false))
        .map(|adapter| {
            // Get first IPv4 address (usually the main one)
            let ip_address = adapter
                .ip_address
                .as_ref()
                .and_then(|ips| ips.first())
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            crate::models::NetworkInfo {
                name: adapter
                    .description
                    .unwrap_or_else(|| "Unknown Adapter".to_string()),
                mac_address: adapter.mac_address.unwrap_or_else(|| "Unknown".to_string()),
                ip_address,
                dhcp_enabled: adapter.dhcp_enabled.unwrap_or(false),
            }
        })
        .collect()
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
            let driver_desc = gpu.name.as_deref().unwrap_or("");
            let wmi_ram = gpu.adapter_ram.unwrap_or(0);

            // Try to get 64-bit VRAM size from Registry (fixes 4GB cap)
            let memory_bytes = get_gpu_vram_from_registry(driver_desc).unwrap_or(wmi_ram);

            // Better precision: use proper rounding only at display time
            let memory_gb = if memory_bytes > 0 {
                // Convert to GB with better precision
                let gb = memory_bytes as f64 / 1_073_741_824.0; // 1024^3
                                                                // Round to 2 decimal places for better accuracy
                (gb * 100.0).round() / 100.0
            } else {
                0.0
            };

            GpuInfo {
                name: gpu.name.unwrap_or_else(|| "Unknown".to_string()),
                memory_gb,
                driver_version: gpu.driver_version.unwrap_or_else(|| "Unknown".to_string()),
                processor: gpu.video_processor.unwrap_or_else(String::new),
                refresh_rate: gpu.current_refresh_rate.unwrap_or(0),
                video_mode: gpu.video_mode_description.unwrap_or_else(String::new),
            }
        })
        .collect()
}

/// Helper: Get GPU VRAM size from Registry (handles value > 4GB)
fn get_gpu_vram_from_registry(driver_desc: &str) -> Option<u64> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let video_class = hklm
        .open_subkey(
            "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}",
        )
        .ok()?;

    for key_name in video_class.enum_keys().map(|x| x.unwrap_or_default()) {
        if let Ok(sub_key) = video_class.open_subkey(&key_name) {
            // Check if this subkey matches the driver description
            let desc: String = sub_key.get_value("DriverDesc").unwrap_or_default();
            if desc == driver_desc {
                // Try reading HardwareInformation.qwMemorySize (QWORD, 64-bit)
                if let Ok(qw_size) = sub_key.get_value::<u64, _>("HardwareInformation.qwMemorySize")
                {
                    return Some(qw_size);
                }

                // Fallback: Try HardwareInformation.MemorySize (DWORD or Binary)
                // Note: Binary values might need distinct handling, but typical fallback is DWORD
                if let Ok(dw_size) = sub_key.get_value::<u32, _>("HardwareInformation.MemorySize") {
                    return Some(dw_size as u64);
                }
            }
        }
    }
    None
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

/// Get disk drive information using MSFT_PhysicalDisk for reliable SSD/HDD detection
/// Falls back to Win32_DiskDrive if storage namespace is unavailable
fn get_disk_info(wmi_con: &WMIConnection) -> Vec<DiskInfo> {
    log::trace!("Querying MSFT_PhysicalDisk from storage namespace");

    // Try MSFT_PhysicalDisk first (more reliable for SSD/HDD detection)
    if let Ok(storage_con) = WMIConnection::with_namespace_path("Root\\Microsoft\\Windows\\Storage")
    {
        let query: Vec<MsftPhysicalDisk> = storage_con.query().unwrap_or_default();
        if !query.is_empty() {
            return query
                .into_iter()
                .map(|disk| {
                    let model = disk
                        .friendly_name
                        .unwrap_or_else(|| "Unknown Drive".to_string());
                    let size_gb = disk
                        .size
                        .map(|s| {
                            let gb = s as f64 / 1_073_741_824.0;
                            (gb * 100.0).round() / 100.0
                        })
                        .unwrap_or(0.0);

                    // MediaType: 0=Unspecified, 3=HDD, 4=SSD, 5=SCM
                    let drive_type = match disk.media_type {
                        Some(3) => "HDD".to_string(),
                        Some(4) => "SSD".to_string(),
                        Some(5) => "SCM".to_string(), // Storage Class Memory (e.g., Intel Optane)
                        _ => "Unknown".to_string(),
                    };

                    // BusType: 7=USB, 10=SAS, 11=SATA, 17=NVMe
                    let interface_type = match disk.bus_type {
                        Some(7) => "USB".to_string(),
                        Some(10) => "SAS".to_string(),
                        Some(11) => "SATA".to_string(),
                        Some(17) => "NVMe".to_string(),
                        _ => "Unknown".to_string(),
                    };

                    // HealthStatus: 0=Healthy, 1=Warning, 2=Unhealthy
                    let health_status = disk.health_status.map(|h| match h {
                        0 => "Healthy".to_string(),
                        1 => "Warning".to_string(),
                        2 => "Unhealthy".to_string(),
                        _ => "Unknown".to_string(),
                    });

                    log::debug!(
                        "Disk (MSFT): model={}, size_gb={:.2}, type={}, interface={}, health={:?}",
                        model,
                        size_gb,
                        drive_type,
                        interface_type,
                        health_status
                    );

                    DiskInfo {
                        model,
                        size_gb,
                        drive_type,
                        interface_type,
                        health_status,
                    }
                })
                .collect();
        }
    }

    // Fallback to Win32_DiskDrive
    log::trace!("Falling back to Win32_DiskDrive");
    let disk_query: Vec<Win32DiskDrive> = wmi_con.query().unwrap_or_default();

    disk_query
        .into_iter()
        .map(|disk| {
            let model = disk.model.unwrap_or_else(|| "Unknown Drive".to_string());
            let size_gb = disk
                .size
                .and_then(|s| s.parse::<u64>().ok())
                .map(|bytes| {
                    let gb = bytes as f64 / 1_073_741_824.0;
                    (gb * 100.0).round() / 100.0
                })
                .unwrap_or(0.0);

            // Best effort drive type detection from Win32_DiskDrive
            let drive_type = disk
                .media_type
                .map(|mt| {
                    if mt.contains("SSD") {
                        "SSD".to_string()
                    } else if mt.contains("Fixed hard disk") {
                        "HDD".to_string() // May be wrong for SSDs
                    } else {
                        mt
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let interface_type = disk.interface_type.unwrap_or_else(|| "Unknown".to_string());

            log::debug!(
                "Disk (Win32): model={}, size_gb={:.2}, type={}, interface={}",
                model,
                size_gb,
                drive_type,
                interface_type
            );

            DiskInfo {
                model,
                size_gb,
                drive_type,
                interface_type,
                health_status: None,
            }
        })
        .collect()
}

/// Get device information from Win32_ComputerSystem
fn get_device_info(wmi_con: &WMIConnection) -> DeviceInfo {
    let query: Vec<Win32ComputerSystem> = wmi_con.query().unwrap_or_default();

    if let Some(cs) = query.first() {
        let manufacturer = cs
            .manufacturer
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let model = cs.model.clone().unwrap_or_else(|| "Unknown".to_string());
        let system_type = cs
            .system_type
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        // PCSystemType: 1=Desktop, 2=Mobile, 3=Workstation, 4=Enterprise Server, etc.
        let pc_type = match cs.pc_system_type {
            Some(1) => "Desktop".to_string(),
            Some(2) => "Laptop".to_string(),
            Some(3) => "Workstation".to_string(),
            Some(4) => "Enterprise Server".to_string(),
            Some(5) => "SOHO Server".to_string(),
            Some(6) => "Appliance PC".to_string(),
            Some(7) => "Performance Server".to_string(),
            Some(8) => "Slate/Tablet".to_string(),
            _ => "Unknown".to_string(),
        };

        log::debug!(
            "Device info: manufacturer={}, model={}, type={}",
            manufacturer,
            model,
            pc_type
        );

        DeviceInfo {
            manufacturer,
            model,
            system_type,
            pc_type,
        }
    } else {
        DeviceInfo::default()
    }
}

/// Get full system information
pub fn get_system_info() -> Result<SystemInfo, Error> {
    log::debug!("Gathering system information");
    let windows = get_windows_info()?;
    let computer_name = env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let username = env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let is_admin = is_running_as_admin();

    // Get hardware and device info using the same WMI connection
    let wmi_con = WMIConnection::new().ok();
    let hardware = get_hardware_info();
    let device = wmi_con.as_ref().map(get_device_info).unwrap_or_default();

    log::debug!(
        "System info: computer={}, user={}, admin={}, device={}",
        computer_name,
        username,
        is_admin,
        device.model
    );

    Ok(SystemInfo {
        windows,
        computer_name,
        username,
        is_admin,
        hardware,
        device,
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

    // ========================================================================
    // is_leap_year tests
    // ========================================================================

    #[test]
    fn test_is_leap_year_divisible_by_4_not_100() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2016));
    }

    #[test]
    fn test_is_leap_year_divisible_by_100_not_400() {
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(2200));
    }

    #[test]
    fn test_is_leap_year_divisible_by_400() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(1600));
        assert!(is_leap_year(2400));
    }

    #[test]
    fn test_is_leap_year_not_divisible_by_4() {
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2019));
        assert!(!is_leap_year(2021));
    }

    // ========================================================================
    // days_before_month tests
    // ========================================================================

    #[test]
    fn test_days_before_month_january() {
        assert_eq!(days_before_month(1, false), 0);
        assert_eq!(days_before_month(1, true), 0);
    }

    #[test]
    fn test_days_before_month_march_non_leap() {
        // Jan(31) + Feb(28) = 59
        assert_eq!(days_before_month(3, false), 59);
    }

    #[test]
    fn test_days_before_month_march_leap() {
        // Jan(31) + Feb(29) = 60
        assert_eq!(days_before_month(3, true), 60);
    }

    #[test]
    fn test_days_before_month_december() {
        // 31+28+31+30+31+30+31+31+30+31+30 = 334 (non-leap)
        assert_eq!(days_before_month(12, false), 334);
        assert_eq!(days_before_month(12, true), 335);
    }

    // ========================================================================
    // parse_wmi_datetime_to_iso tests
    // ========================================================================

    #[test]
    fn test_parse_wmi_datetime_to_iso_valid() {
        let wmi = "20241213123456.000000+000";
        let iso = parse_wmi_datetime_to_iso(wmi);
        assert_eq!(iso, "2024-12-13T12:34:56");
    }

    #[test]
    fn test_parse_wmi_datetime_to_iso_short_input() {
        let wmi = "2024";
        let iso = parse_wmi_datetime_to_iso(wmi);
        // Should return as-is when too short
        assert_eq!(iso, "2024");
    }

    #[test]
    fn test_parse_wmi_datetime_to_iso_midnight() {
        let wmi = "20240101000000.000000+000";
        let iso = parse_wmi_datetime_to_iso(wmi);
        assert_eq!(iso, "2024-01-01T00:00:00");
    }
}
