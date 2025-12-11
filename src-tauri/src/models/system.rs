use serde::{Deserialize, Serialize};

/// Windows version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsInfo {
    /// Product name (e.g., "Windows 11 Pro")
    pub product_name: String,
    /// Display version (e.g., "23H2")
    pub display_version: String,
    /// Build number as string (e.g., "22631")
    pub build_number: String,
    /// Whether this is Windows 11 (build >= 22000)
    pub is_windows_11: bool,
    /// Version string: "10" or "11" for tweak filtering
    pub version_string: String,
}

impl WindowsInfo {
    pub fn is_windows_10(&self) -> bool {
        !self.is_windows_11
    }

    pub fn display_version_full(&self) -> String {
        format!("{} (Build {})", self.product_name, self.build_number)
    }

    /// Get version as u32 (10 or 11) for registry change filtering
    pub fn version_number(&self) -> u32 {
        if self.is_windows_11 {
            11
        } else {
            10
        }
    }
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    /// CPU name (e.g., "Intel Core i7-12700K")
    pub name: String,
    /// Number of physical cores
    pub cores: u32,
    /// Number of logical processors (threads)
    pub threads: u32,
    /// CPU architecture (e.g., "x64")
    pub architecture: String,
    /// Maximum clock speed in MHz
    pub max_clock_mhz: u32,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    /// GPU name (e.g., "NVIDIA GeForce RTX 3080")
    pub name: String,
    /// GPU memory in GB
    pub memory_gb: f64,
    /// Driver version
    pub driver_version: String,
}

/// Memory (RAM) information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    /// Total physical memory in GB
    pub total_gb: f64,
    /// Memory speed in MHz
    pub speed_mhz: u32,
    /// Memory type (e.g., "DDR4", "DDR5")
    pub memory_type: String,
    /// Number of memory sticks
    pub slots_used: u32,
}

/// Motherboard information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MotherboardInfo {
    /// Manufacturer (e.g., "ASUS", "MSI", "Gigabyte")
    pub manufacturer: String,
    /// Product name/model
    pub product: String,
    /// BIOS version
    pub bios_version: String,
}

/// Hardware information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub gpu: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub motherboard: MotherboardInfo,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub windows: WindowsInfo,
    pub computer_name: String,
    pub username: String,
    pub is_admin: bool,
    pub hardware: HardwareInfo,
}
