//! Tweak loader service - provides access to pre-compiled tweak definitions.
//!
//! Tweaks are compiled from YAML files at build time by `build.rs`.
//! This eliminates runtime file I/O and YAML parsing for instant loading.

use crate::error::Error;
use crate::generated_tweaks::{CATEGORIES, TWEAKS};
use crate::models::{CategoryDefinition, TweakDefinition};

/// Load all categories (pre-compiled at build time).
///
/// Categories are sorted by their `order` field.
pub fn load_all_categories() -> Result<&'static [CategoryDefinition], Error> {
    log::debug!(
        "Returning {} pre-compiled categories",
        crate::generated_tweaks::CATEGORY_COUNT
    );
    Ok(CATEGORIES.as_slice())
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
pub fn get_tweaks_for_version(version: u32) -> Result<Vec<&'static TweakDefinition>, Error> {
    log::debug!("Getting tweaks for Windows version: {}", version);
    let total = TWEAKS.len();

    // Borrow from the compiled-in map instead of deep-cloning up to 189 definitions per call.
    let filtered: Vec<&'static TweakDefinition> = TWEAKS
        .values()
        .filter(|tweak| tweak.applies_to_version(version))
        .collect();

    log::info!(
        "Filtered tweaks for Windows {}: {} of {} applicable",
        version,
        filtered.len(),
        total
    );
    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the build.rs <-> models/tweak.rs type mirror.
    ///
    /// build.rs parses the YAML with its own hand-written copy of these types and
    /// serializes the result to tweaks.json; the runtime types here deserialize it.
    /// The two are maintained by hand and drift silently: a field renamed on one
    /// side only, or added to the build mirror but not to `TweakDefinition` (which
    /// is `deny_unknown_fields`), produces a panic inside a `LazyLock` on the first
    /// tweak lookup -- i.e. at runtime, on a user's machine, not at compile time.
    ///
    /// Touching either side runs this, so the drift surfaces here instead.
    #[test]
    fn embedded_tweak_data_deserializes_into_the_runtime_types() {
        // Forcing the LazyLock is the whole point: this is where the .expect() lives.
        let tweak_count = TWEAKS.len();
        let category_count = CATEGORIES.len();

        assert!(tweak_count > 0, "no tweaks were compiled into the binary");
        assert!(
            category_count > 0,
            "no categories were compiled into the binary"
        );
        assert_eq!(
            category_count,
            crate::generated_tweaks::CATEGORY_COUNT,
            "CATEGORY_COUNT disagrees with the embedded category data"
        );

        // Every tweak must satisfy the invariant build.rs validates, so a build-time
        // rule that stops being enforced does not pass unnoticed.
        for (id, tweak) in TWEAKS.iter() {
            assert!(
                tweak.options.len() >= 2,
                "tweak '{}' has {} option(s); the minimum is 2",
                id,
                tweak.options.len()
            );
            assert_eq!(id, &tweak.id, "map key and tweak.id disagree");
            assert!(
                !tweak.category_id.is_empty(),
                "tweak '{}' has no category_id",
                id
            );
        }
    }
}
