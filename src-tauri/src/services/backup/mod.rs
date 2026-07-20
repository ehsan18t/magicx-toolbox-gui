//! # Snapshot-Based Backup Service
//!
//! Unified option-based backup system for Windows registry tweaks with atomic
//! rollback capabilities.
//!
//! ## Module Organization
//!
//! - `storage`: File I/O for snapshot persistence
//! - `capture`: State capture before applying tweaks
//! - `restore`: Atomic restore with rollback support
//! - `detection`: State detection and snapshot validation
//! - `inspection`: Per-item mismatch report for the UI
//! - `compare`: The shared option-vs-current comparison core (detection + inspection)
//! - `helpers`: Parsing and comparison utilities

#[cfg(test)]
mod roundtrip_tests;

mod capture;
mod compare;
mod detection;
mod helpers;
pub mod inspection;
pub mod restore;
pub mod storage;

// Re-export public items from submodules
pub use capture::{capture_current_state, capture_snapshot, read_registry_value};
pub use detection::{detect_tweak_state, validate_all_snapshots};
pub use inspection::inspect_tweak;
pub use restore::{restore_from_snapshot, RestoreResult};
pub use storage::{
    delete_snapshot, get_applied_tweaks, load_snapshot, mark_needs_attention, save_snapshot,
    snapshot_exists, update_snapshot_metadata,
};
