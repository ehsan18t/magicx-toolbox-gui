//! Elevation level — the single value the broker's own dispatch (`broker.rs`) uses to choose
//! in-process vs. a fresh elevated child.
//!
//! Every privileged operation used to be routed by a pair of booleans `(use_system, use_ti)`
//! threaded through the apply chain and expanded at ~6 sites into
//! `if use_ti { .. } else if use_system { .. } else { .. }`. That pair can express the nonsense
//! state `(use_system = false, use_ti = true)`. Collapsing it to one enum makes that state
//! unrepresentable and turns every dispatch into a single `match`.

/// The privilege level an operation runs at. `TrustedInstaller` is strictly higher than `System`,
/// which is strictly higher than `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    /// Run in-process — no elevation (`broker.rs`'s own general-purpose dispatch still branches
    /// on this; the redesigned engine's callers only ever construct `System`/`TrustedInstaller`
    /// here, since a `User`/`Admin`-level drive never reaches the broker at all — see
    /// `tweaks::engine::to_elevation`).
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
