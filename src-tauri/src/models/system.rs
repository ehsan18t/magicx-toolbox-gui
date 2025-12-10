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

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub windows: WindowsInfo,
    pub computer_name: String,
    pub username: String,
    pub is_admin: bool,
}
