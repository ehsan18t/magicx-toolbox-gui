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
//! - `helpers`: Parsing and comparison utilities

mod capture;
mod detection;
mod helpers;
pub mod inspection;
pub mod restore;
pub mod storage;

// Re-export public items from submodules
pub use capture::{capture_current_state, capture_snapshot};
pub use detection::{cleanup_old_backups, detect_tweak_state, validate_all_snapshots};
pub use inspection::inspect_tweak;
pub use restore::restore_from_snapshot;
pub use storage::{
    delete_snapshot, get_applied_tweaks, get_snapshots_dir, load_snapshot, save_snapshot,
    snapshot_exists, update_snapshot_metadata,
};
