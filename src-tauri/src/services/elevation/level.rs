//! Elevation level: the single value `broker.rs` dispatches on to choose in-process versus a fresh
//! elevated child.

/// The privilege level an operation runs at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    /// Run in-process. Engine callers only ever construct `TrustedInstaller`, since a
    /// `User`/`Admin` drive never reaches the broker (see `tweaks::engine::to_elevation`); this
    /// variant exists for `broker.rs`'s own dispatch and to exercise `run_ops` without a spawn.
    #[allow(dead_code)]
    None,
    /// Run as TrustedInstaller (parent-process spoof off the TI service).
    TrustedInstaller,
}

impl Elevation {
    /// Whether this level needs elevation.
    pub fn is_elevated(self) -> bool {
        !matches!(self, Elevation::None)
    }
}
