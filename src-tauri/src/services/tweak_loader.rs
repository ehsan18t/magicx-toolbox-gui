//! Tweak loader service - provides access to pre-compiled tweak definitions.
//!
//! Tweaks are compiled from YAML files at build time by `build.rs`.
//! This eliminates runtime file I/O and YAML parsing for instant loading.

use crate::error::Error;
use crate::generated_tweaks::{CATEGORIES, TWEAKS};
use crate::models::{CategoryDefinition, TweakDefinition};
use std::collections::HashMap;

/// Load all categories (pre-compiled at build time).
///
/// Categories are sorted by their `order` field.
pub fn load_all_categories() -> Result<Vec<CategoryDefinition>, Error> {
    log::debug!(
        "Returning {} pre-compiled categories",
        crate::generated_tweaks::CATEGORY_COUNT
    );
    Ok(CATEGORIES.clone())
}

/// Load all tweaks (pre-compiled at build time).
///
/// Returns a HashMap for O(1) lookup by tweak ID.
#[allow(dead_code)]
pub fn load_all_tweaks() -> Result<HashMap<String, TweakDefinition>, Error> {
    log::debug!(
        "Returning {} pre-compiled tweaks",
        crate::generated_tweaks::TWEAK_COUNT
    );
    Ok(TWEAKS.clone())
}

/// Get a specific tweak by ID.
///
/// This is O(1) lookup from the pre-compiled HashMap.
pub fn get_tweak(tweak_id: &str) -> Result<Option<TweakDefinition>, Error> {
    log::trace!("Looking up tweak: {}", tweak_id);
    let result = TWEAKS.get(tweak_id).cloned();
    if result.is_none() {
        log::debug!("Tweak not found: {}", tweak_id);
    }
    Ok(result)
}

/// Filter tweaks by Windows version (u32: 10 or 11).
///
/// Returns only tweaks that have registry changes applicable to the given version.
pub fn get_tweaks_for_version(version: u32) -> Result<HashMap<String, TweakDefinition>, Error> {
    log::debug!("Getting tweaks for Windows version: {}", version);
    let total = TWEAKS.len();

    let filtered: HashMap<_, _> = TWEAKS
        .iter()
        .filter(|(_, tweak)| tweak.applies_to_version(version))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    log::info!(
        "Filtered tweaks for Windows {}: {} of {} applicable",
        version,
        filtered.len(),
        total
    );
    Ok(filtered)
}

/// Filter tweaks by category.
pub fn get_tweaks_by_category(category: &str) -> Result<HashMap<String, TweakDefinition>, Error> {
    log::debug!("Getting tweaks for category: {}", category);

    let filtered: HashMap<_, _> = TWEAKS
        .iter()
        .filter(|(_, tweak)| tweak.category.eq_ignore_ascii_case(category))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    log::debug!("Found {} tweaks in category '{}'", filtered.len(), category);
    Ok(filtered)
}
