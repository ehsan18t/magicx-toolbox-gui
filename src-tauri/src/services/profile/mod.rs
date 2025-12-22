//! Profile Service Module
//!
//! Handles configuration profile export, import, validation, and application.

pub mod archive;
mod export;
mod import;
pub mod migration;
mod validation;

use sha2::{Digest, Sha256};

use crate::models::TweakOption;

// Re-export main functions
pub use export::export_profile;
pub use import::{apply_profile, import_profile};
pub use validation::validate_profile;

/// Compute a hash of option content for schema change detection.
///
/// This hash is used to detect when a tweak's option definition has changed
/// between when a profile was created and when it's being imported.
/// Uses first 32 characters (128 bits) for good collision resistance.
pub fn hash_option_content(option: &TweakOption) -> String {
    let mut hasher = Sha256::new();

    // Hash registry changes
    for change in &option.registry_changes {
        hasher.update(change.hive.as_str().as_bytes());
        hasher.update(change.key.as_bytes());
        hasher.update(change.value_name.as_bytes());
        if let Some(ref v) = change.value {
            hasher.update(format!("{:?}", v).as_bytes());
        }
    }

    // Hash service changes
    for service in &option.service_changes {
        hasher.update(service.name.as_bytes());
        hasher.update(service.startup.as_str().as_bytes());
    }

    // Hash scheduler changes
    for task in &option.scheduler_changes {
        hasher.update(task.task_path.as_bytes());
        if let Some(ref name) = task.task_name {
            hasher.update(name.as_bytes());
        }
        hasher.update(task.action.as_str().as_bytes());
    }

    // Use first 32 characters (128 bits) for good collision resistance
    hex::encode(hasher.finalize())[..32].to_string()
}
