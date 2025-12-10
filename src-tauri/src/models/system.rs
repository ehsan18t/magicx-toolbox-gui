use serde::{Deserialize, Serialize};

/// Windows version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsInfo {
    /// Windows version: "10" or "11"
    pub version: String,
    /// Build number (e.g., 22621 for Windows 11)
    pub build: u32,
    /// Edition: "Home", "Pro", "Enterprise", etc.
    pub edition: String,
    /// Architecture: "x64", "x86", "ARM64"
    pub architecture: String,
    /// Whether the current process is running as administrator
    pub is_admin: bool,
}

impl WindowsInfo {
    pub fn is_windows_10(&self) -> bool {
        self.version == "10"
    }

    pub fn is_windows_11(&self) -> bool {
        self.version == "11"
    }

    pub fn is_64bit(&self) -> bool {
        self.architecture == "x64" || self.architecture == "ARM64"
    }

    pub fn display_version(&self) -> String {
        format!("Windows {} (Build {})", self.version, self.build)
    }
}

/// System information (could extend WindowsInfo later)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub windows_info: WindowsInfo,
    /// Additional system info can be added here
    pub total_ram_gb: Option<u32>,
    pub processor_count: Option<u32>,
}

impl From<WindowsInfo> for SystemInfo {
    fn from(windows_info: WindowsInfo) -> Self {
        SystemInfo {
            windows_info,
            total_ram_gb: None,
            processor_count: None,
        }
    }
}
