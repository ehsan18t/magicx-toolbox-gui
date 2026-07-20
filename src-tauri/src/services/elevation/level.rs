//! Elevation level — the single value the apply chain uses to choose a privilege level.
//!
//! Every privileged operation used to be routed by a pair of booleans `(use_system, use_ti)`
//! threaded through the apply chain and expanded at ~6 sites into
//! `if use_ti { .. } else if use_system { .. } else { .. }`. That pair can express the nonsense
//! state `(use_system = false, use_ti = true)`. Collapsing it to one enum makes that state
//! unrepresentable and turns every dispatch into a single `match`.

/// The privilege level an operation runs at.
///
/// Derived once from a tweak's declared flags via [`Elevation::from_flags`]. `TrustedInstaller`
/// is strictly higher than `System`, which is strictly higher than `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    /// Run as the current user — no elevation.
    None,
    /// Run as SYSTEM (winlogon token duplication).
    System,
    /// Run as TrustedInstaller (parent-process spoof off the TI service).
    TrustedInstaller,
}

impl Elevation {
    /// Derive the level from a tweak's declared `requires_system` / `requires_ti` flags.
    ///
    /// `requires_ti` implies `requires_system` (and admin) in the tweak-system hierarchy, so an
    /// inconsistent `(requires_system = false, requires_ti = true)` still resolves to
    /// `TrustedInstaller` rather than a hybrid.
    pub fn from_flags(requires_system: bool, requires_ti: bool) -> Self {
        if requires_ti {
            Elevation::TrustedInstaller
        } else if requires_system {
            Elevation::System
        } else {
            Elevation::None
        }
    }

    /// Whether this level needs elevation (SYSTEM or TrustedInstaller).
    pub fn is_elevated(self) -> bool {
        !matches!(self, Elevation::None)
    }

    /// Human-readable label for logging: `"User"`, `"SYSTEM"`, or `"TrustedInstaller"`.
    pub fn label(self) -> &'static str {
        match self {
            Elevation::None => "User",
            Elevation::System => "SYSTEM",
            Elevation::TrustedInstaller => "TrustedInstaller",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_flags_maps_the_three_reachable_states() {
        assert_eq!(Elevation::from_flags(false, false), Elevation::None);
        assert_eq!(Elevation::from_flags(true, false), Elevation::System);
        assert_eq!(Elevation::from_flags(true, true), Elevation::TrustedInstaller);
    }

    #[test]
    fn requires_ti_without_system_still_resolves_to_ti() {
        // The nonsense input a bool pair allowed must never produce a hybrid.
        assert_eq!(Elevation::from_flags(false, true), Elevation::TrustedInstaller);
    }

    #[test]
    fn is_elevated_is_false_only_for_none() {
        assert!(!Elevation::None.is_elevated());
        assert!(Elevation::System.is_elevated());
        assert!(Elevation::TrustedInstaller.is_elevated());
    }
}
