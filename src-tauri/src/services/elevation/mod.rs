//! # Elevation Services (SYSTEM and TrustedInstaller)
//!
//! Provides functionality to execute commands with elevated privileges on Windows.
//!
//! ## Module Organization
//!
//! - `common`: Shared utilities, constants, and Windows API imports
//! - `system_elevation`: SYSTEM-level elevation (via winlogon.exe token)
//! - `ti_elevation`: TrustedInstaller-level elevation (via parent process spoofing)
//!
//! ## Usage
//!
//! ```ignore
//! // Check if elevation is available
//! if can_use_system_elevation() {
//!     // Run command as SYSTEM
//!     run_command_as_system("cmd /c echo hello")?;
//!
//!     // Set registry value as SYSTEM
//!     set_registry_value_as_system("HKLM", "SOFTWARE\\Test", "Value", "REG_DWORD", "1")?;
//! }
//! ```

mod common;
mod system_elevation;
mod ti_elevation;

// Re-export common utilities
pub use common::{escape_shell_arg, validate_registry_path};

// Re-export SYSTEM elevation functions
pub use system_elevation::{
    can_use_system_elevation, delete_registry_value_as_system, run_command_as_system,
    set_registry_value_as_system, set_service_startup_as_system, start_service_as_system,
    stop_service_as_system,
};

// Re-export TrustedInstaller elevation functions
pub use ti_elevation::{
    run_command_as_ti, run_powershell, run_powershell_as_system, run_powershell_as_ti,
    run_schtasks_as_system, run_schtasks_as_ti, set_service_startup_as_ti, start_service_as_ti,
    stop_service_as_ti, PowerShellResult,
};
