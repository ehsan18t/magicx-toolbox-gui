use crate::error::Error;
use crate::models::{
    CpuInfo, DeviceInfo, DiskInfo, GpuInfo, HardwareInfo, LiveSystemInfo, MachineHardware,
    MemoryInfo, MotherboardInfo, SystemReading, WindowsInfo,
};
use crate::tweaks::winver::running_winver;
use serde::Deserialize;
use std::env;
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
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

const CURRENT_VERSION_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";

/// The running Windows version and uptime, from the registry and `RtlGetVersion`: no WMI.
pub fn get_windows_info() -> Result<WindowsInfo, Error> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(CURRENT_VERSION_KEY, KEY_READ)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;
    let build = running_winver().build;
    let is_windows_11 = build >= 22000;
    let registry_name: String = key
        .get_value("ProductName")
        .unwrap_or_else(|_| "Windows".to_string());
    // SAFETY: no arguments; it cannot fail.
    let uptime_seconds = unsafe { GetTickCount64() } / 1000;
    let info = WindowsInfo {
        product_name: product_name(&registry_name, build),
        display_version: key.get_value("DisplayVersion").unwrap_or_default(),
        build_number: build.to_string(),
        is_windows_11,
        version_string: if is_windows_11 { "11" } else { "10" }.to_string(),
        uptime_seconds,
        install_date: key
            .get_value::<u32, _>("InstallDate")
            .ok()
            .and_then(install_date_iso),
    };
    log::info!(
        "Detected Windows {} (build {}, {}), uptime={}s",
        info.version_string,
        info.build_number,
        info.display_version,
        uptime_seconds
    );
    Ok(info)
}

/// `ProductName` still reads "Windows 10" on Windows 11, so the build decides the major.
fn product_name(registry_name: &str, build: u32) -> String {
    let name = registry_name
        .strip_prefix("Microsoft ")
        .unwrap_or(registry_name);
    if build >= 22000 {
        name.replacen("Windows 10", "Windows 11", 1)
    } else {
        name.to_string()
    }
}

/// `InstallDate` is Unix seconds (UTC).
fn install_date_iso(unix_seconds: u32) -> Option<String> {
    chrono::DateTime::from_timestamp(i64::from(unix_seconds), 0)
        .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

fn wmi_failed(what: &'static str) -> impl Fn(wmi::WMIError) -> Error {
    move |e| Error::WindowsApi(format!("WMI {what}: {e}"))
}

fn connect() -> Result<WMIConnection, Error> {
    WMIConnection::new().map_err(wmi_failed("connection"))
}

fn joined<T>(
    handle: std::thread::ScopedJoinHandle<'_, Result<T, Error>>,
    what: &str,
) -> Result<T, Error> {
    handle
        .join()
        .map_err(|_| Error::WindowsApi(format!("the {what} read panicked")))?
}

/// One class query failing (damaged WMI repository, some VMs) costs only its own fields.
fn or_default<T: Default>(read: Result<T, Error>, partial: &mut bool) -> T {
    read.unwrap_or_else(|e| {
        log::warn!("{e}; showing its default");
        *partial = true;
        T::default()
    })
}

/// Every WMI read, on three threads that each own a COM apartment and a connection: one
/// connection per query costs more in setup than the queries take. Only a failed connection or
/// a panicked thread fails the read.
fn read_machine_hardware() -> Result<MachineHardware, Error> {
    let start = std::time::Instant::now();
    let (cimv2, display, storage) = std::thread::scope(|s| {
        let cimv2 = s.spawn(|| -> Result<_, Error> {
            let con = connect()?;
            let mut partial = false;
            let read = (
                or_default(get_cpu_info(&con), &mut partial),
                or_default(get_memory_info(&con), &mut partial),
                or_default(get_motherboard_info(&con), &mut partial),
                or_default(get_network_info(&con), &mut partial),
                or_default(get_device_info(&con), &mut partial),
            );
            Ok((read, partial))
        });
        let display = s.spawn(|| -> Result<_, Error> {
            let con = connect()?;
            let mut partial = false;
            let gpu = or_default(get_gpu_info(&con), &mut partial);
            Ok(((gpu, get_monitor_info(&con)), partial))
        });
        let storage = s.spawn(|| -> Result<_, Error> {
            let mut partial = false;
            let disks = or_default(get_disk_info(&connect()?), &mut partial);
            Ok((disks, partial))
        });
        (
            joined(cimv2, "CPU, memory, board and network"),
            joined(display, "GPU and monitor"),
            joined(storage, "disk"),
        )
    });
    let ((cpu, memory, motherboard, network, device), cimv2_partial) = cimv2?;
    let ((gpu, monitors), display_partial) = display?;
    let (disks, storage_partial) = storage?;
    log::debug!("Hardware info gathered in {:?}", start.elapsed());

    let total_storage_gb: f64 = disks.iter().map(|d| d.size_gb).sum();
    Ok(MachineHardware {
        hardware: HardwareInfo {
            cpu,
            gpu,
            monitors,
            memory,
            motherboard,
            disks,
            network,
            total_storage_gb,
        },
        device,
        partial: cimv2_partial || display_partial || storage_partial,
    })
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
fn get_monitor_info(wmi_con: &WMIConnection) -> Vec<crate::models::MonitorInfo> {
    log::debug!("Gathering monitor info via Nested EnumDisplayDevices + WMI + PnP");

    let monitor_names = get_all_monitor_names(wmi_con);
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
fn get_all_monitor_names(wmi_con: &WMIConnection) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

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
fn get_network_info(wmi_con: &WMIConnection) -> Result<Vec<crate::models::NetworkInfo>, Error> {
    let query: Vec<Win32NetworkAdapterConfiguration> =
        wmi_con.query().map_err(wmi_failed("network adapters"))?;

    Ok(query
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
        .collect())
}

/// Get CPU information from WMI
fn get_cpu_info(wmi_con: &WMIConnection) -> Result<CpuInfo, Error> {
    let query: Vec<Win32Processor> = wmi_con.query().map_err(wmi_failed("processor"))?;

    Ok(if let Some(cpu) = query.first() {
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
    })
}

/// Get GPU information from WMI
fn get_gpu_info(wmi_con: &WMIConnection) -> Result<Vec<GpuInfo>, Error> {
    let query: Vec<Win32VideoController> =
        wmi_con.query().map_err(wmi_failed("video controllers"))?;

    Ok(query
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
        .collect())
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
fn get_memory_info(wmi_con: &WMIConnection) -> Result<MemoryInfo, Error> {
    let query: Vec<Win32PhysicalMemory> = wmi_con.query().map_err(wmi_failed("physical memory"))?;

    if query.is_empty() {
        return Ok(MemoryInfo::default());
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

    Ok(MemoryInfo {
        total_gb: (total_gb * 10.0).round() / 10.0,
        speed_mhz,
        memory_type,
        slots_used,
    })
}

/// Get motherboard information from WMI
fn get_motherboard_info(wmi_con: &WMIConnection) -> Result<MotherboardInfo, Error> {
    let baseboard_query: Vec<Win32BaseBoard> = wmi_con.query().map_err(wmi_failed("baseboard"))?;

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

    let bios_query: Vec<Win32Bios> = wmi_con.query().map_err(wmi_failed("BIOS"))?;
    let bios_version = bios_query
        .first()
        .and_then(|b| b.smbios_bios_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(MotherboardInfo {
        manufacturer,
        product,
        bios_version,
    })
}

/// Get disk drive information using MSFT_PhysicalDisk for reliable SSD/HDD detection
/// Falls back to Win32_DiskDrive if storage namespace is unavailable
fn get_disk_info(wmi_con: &WMIConnection) -> Result<Vec<DiskInfo>, Error> {
    log::trace!("Querying MSFT_PhysicalDisk from storage namespace");

    // Try MSFT_PhysicalDisk first (more reliable for SSD/HDD detection)
    if let Ok(storage_con) = WMIConnection::with_namespace_path("Root\\Microsoft\\Windows\\Storage")
    {
        let query: Vec<MsftPhysicalDisk> = storage_con.query().unwrap_or_default();
        if !query.is_empty() {
            return Ok(query
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
                .collect());
        }
    }

    // Fallback to Win32_DiskDrive
    log::trace!("Falling back to Win32_DiskDrive");
    let disk_query: Vec<Win32DiskDrive> = wmi_con.query().map_err(wmi_failed("disk drives"))?;

    Ok(disk_query
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
        .collect())
}

/// Get device information from Win32_ComputerSystem
fn get_device_info(wmi_con: &WMIConnection) -> Result<DeviceInfo, Error> {
    let query: Vec<Win32ComputerSystem> = wmi_con.query().map_err(wmi_failed("computer system"))?;

    Ok(if let Some(cs) = query.first() {
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
    })
}

/// The live fields, plus the WMI hardware read when `with_hardware`; a failed connection is `Err`.
pub fn get_system_info(with_hardware: bool) -> Result<SystemReading, Error> {
    let live = LiveSystemInfo {
        windows: get_windows_info()?,
        computer_name: env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string()),
        username: env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string()),
        is_admin: is_running_as_admin(),
    };
    Ok(SystemReading {
        live,
        machine: with_hardware.then(read_machine_hardware).transpose()?,
    })
}

/// Whether this process holds an elevated token, via `TokenElevation`. Not a `KEY_WRITE` probe of
/// an HKLM key: any error there (low memory, missing key) reads as "not admin", falsely gating
/// every admin-floor tweak and every HKLM write.
pub fn is_running_as_admin() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // SAFETY: `GetCurrentProcess` returns a pseudo-handle that never needs closing; the token
    // handle is closed on every path; `elevation` is a live local sized to what the callee writes.
    let is_admin = unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
            log::warn!("admin check: OpenProcessToken failed; assuming not elevated");
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            std::ptr::addr_of_mut!(elevation).cast(),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        ) != FALSE;
        CloseHandle(token);
        if !ok {
            log::warn!(
                "admin check: GetTokenInformation(TokenElevation) failed; assuming not elevated"
            );
            return false;
        }
        elevation.TokenIsElevated != 0
    };
    log::trace!("Admin check: {}", is_admin);
    is_admin
}

/// The machine's stable identity — `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid`.
///
/// Returns `None` if it can't be read; callers treat that as "identity unknown" and skip the
/// machine-mismatch check rather than failing.
pub fn machine_guid() -> Option<String> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\Cryptography", KEY_READ)
        .and_then(|key| key.get_value::<String, _>("MachineGuid"))
        .inspect_err(|e| log::warn!("could not read MachineGuid: {e}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_info_reads_without_wmi() {
        let info = get_windows_info().expect("CurrentVersion is readable");
        assert!(info.version_string == "10" || info.version_string == "11");
        assert!(info.build_number.parse::<u32>().unwrap() > 0);
        assert!(info.uptime_seconds > 0);
    }

    #[test]
    fn product_name_takes_the_major_from_the_build() {
        assert_eq!(
            product_name("Windows 10 IoT Enterprise LTSC 2024", 26100),
            "Windows 11 IoT Enterprise LTSC 2024"
        );
        assert_eq!(product_name("Windows 10 Pro", 19045), "Windows 10 Pro");
        assert_eq!(
            product_name("Microsoft Windows 10 Pro", 22631),
            "Windows 11 Pro"
        );
    }

    #[test]
    fn a_failed_class_falls_back_to_its_default_and_marks_the_read_partial() {
        let mut partial = false;
        assert_eq!(or_default(Ok(3u32), &mut partial), 3);
        assert!(!partial);
        let failed: Result<u32, Error> = Err(Error::WindowsApi("WMI processor: x".into()));
        assert_eq!(or_default(failed, &mut partial), 0);
        assert!(partial);
        assert_eq!(or_default(Ok(5u32), &mut partial), 5);
        assert!(partial, "a later success does not clear it");
    }

    #[test]
    fn install_date_is_utc_iso() {
        assert_eq!(
            install_date_iso(0x6823_885c).as_deref(),
            Some("2025-05-13T17:58:52Z")
        );
    }
}
