//! # Elevation Service (TrustedInstaller)
//!
//! Executes privileged operations on Windows via the **broker**: the main app spawns this binary
//! with a TrustedInstaller token (`--broker`), and the child runs the same effect services the
//! unelevated path uses. No shell command strings; results cross back typed.
//!
//! - `level`: the `Elevation` enum, the single dispatch value for the apply chain
//! - `broker`: the wire protocol, the executor, the `--broker` entrypoint, and `run_ops`
//! - `common`: Win32 helpers for the spawn path
//! - `ti_elevation`: TrustedInstaller parent-process spoof + spawn
//!
//! The only way in is [`run_ops`]: a batch of typed [`BrokerOp`]s at one [`Elevation`].

mod broker;
mod common;
mod level;
mod ti_elevation;

pub use broker::{run_broker, run_ops, BrokerOp, BrokerOpError};
pub use level::Elevation;
