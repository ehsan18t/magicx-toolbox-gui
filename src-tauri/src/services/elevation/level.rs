//! Elevation level: the single value `broker.rs` dispatches on to choose in-process versus a fresh
//! elevated child.
//!
//! One enum rather than the `(use_system, use_ti)` boolean pair this replaced, which could express
//! the nonsense state `(false, true)` and expanded into a three-way `if` at every call site.

/// The privilege level an operation runs at. `TrustedInstaller` is strictly higher than `System`,
/// which is strictly higher than `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    /// Run in-process. Engine callers only ever construct `System`/`TrustedInstaller`, since a
    /// `User`/`Admin` drive never reaches the broker (see `tweaks::engine::to_elevation`); this
    /// variant exists for `broker.rs`'s own dispatch and to exercise `run_ops` without a spawn.
    #[allow(dead_code)]
    None,
    /// Run as SYSTEM (winlogon token duplication).
    System,
    /// Run as TrustedInstaller (parent-process spoof off the TI service).
    TrustedInstaller,
}

impl Elevation {
    /// Whether this level needs elevation (SYSTEM or TrustedInstaller).
    pub fn is_elevated(self) -> bool {
        !matches!(self, Elevation::None)
    }
}
