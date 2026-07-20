//! # Elevation Services (SYSTEM and TrustedInstaller)
//!
//! Executes privileged operations on Windows via the **broker**: the main app spawns this binary
//! with a SYSTEM or TrustedInstaller token (`--broker`), and the child runs the same effect
//! services the unelevated path uses — no shell command strings, results cross back typed.
//!
//! ## Module Organization
//!
//! - `level`: the `Elevation` enum — the single dispatch value for the apply chain
//! - `broker`: the elevated effect broker (protocol, executor, `--broker` entrypoint, `run_elevated_broker`)
//! - `common`: shared utilities, constants, and Windows API imports
//! - `system_elevation`: SYSTEM token duplication (winlogon.exe) + spawn, and the SYSTEM wrappers
//! - `ti_elevation`: TrustedInstaller parent-process spoof + spawn, and the TI wrappers
//!
//! ## Usage
//!
//! ```ignore
//! if can_use_system_elevation() {
//!     run_command_as_system("cmd /c echo hello")?;
//!     set_registry_value_as_system(
//!         RegistryHive::Hklm,
//!         "SOFTWARE\\Test",
//!         "Value",
//!         RegistryValueType::Dword,
//!         serde_json::json!(1),
//!     )?;
//! }
//! ```

mod broker;
mod common;
mod level;
mod system_elevation;
mod ti_elevation;

// Re-export the elevation level enum (the single dispatch value for the apply chain)
pub use level::Elevation;

// Re-export the broker entrypoint (called from the `--broker` subcommand in lib.rs) and the typed
// scheduler op. The broker protocol types stay internal to this module — the elevated wrappers
// build them.
pub use broker::{run_broker, run_scheduler_op};

// Re-export SYSTEM elevation functions
pub use system_elevation::{
    can_use_system_elevation, delete_registry_value_as_system, run_command_as_system,
    set_registry_value_as_system, set_service_startup_as_system, start_service_as_system,
    stop_service_as_system,
};

// Re-export TrustedInstaller elevation functions
pub use ti_elevation::{
    run_command_as_ti, run_powershell, run_powershell_as_system, run_powershell_as_ti,
    set_service_startup_as_ti, start_service_as_ti, stop_service_as_ti,
};
