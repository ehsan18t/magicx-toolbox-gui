//! # Elevation Services (SYSTEM and TrustedInstaller)
//!
//! Executes privileged operations on Windows via the **broker**: the main app spawns this binary
//! with a SYSTEM or TrustedInstaller token (`--broker`), and the child runs the same effect
//! services the unelevated path uses — no shell command strings, results cross back typed.
//!
//! ## Module Organization
//!
//! - `level`: the `Elevation` enum — the single dispatch value for the apply chain
//! - `broker`: the elevated effect broker (protocol, executor, `--broker` entrypoint,
//!   `run_elevated_broker`, `run_ops` — the redesigned engine's grouped-execution caller, spec §9)
//! - `common`: shared utilities, constants, and Windows API imports
//! - `system_elevation`: SYSTEM token duplication (winlogon.exe) + spawn
//! - `ti_elevation`: TrustedInstaller parent-process spoof + spawn
//!
//! ## Usage
//!
//! ```ignore
//! if can_use_system_elevation() {
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

// Re-export the broker entrypoint (called from the `--broker` subcommand in lib.rs) and the
// multi-op caller (Task 14: `tweaks::kinds` routes System/TI drives through `run_ops`, which needs
// `BrokerOp`/`BrokerOpError` at this boundary too).
pub use broker::{run_broker, run_ops, BrokerOp, BrokerOpError};

// Re-export SYSTEM elevation functions
pub use system_elevation::{can_use_system_elevation, set_registry_value_as_system};
