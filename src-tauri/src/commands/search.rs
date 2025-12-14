//! Fuzzy Search Commands - High-performance concurrent fuzzy search for tweaks
//!
//! Uses nucleo-matcher (same engine as fzf) with rayon for parallel execution.

use crate::error::Result;
use crate::services::{system_info_service, tweak_loader};
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use rayon::prelude::*;
use serde::Serialize;

/// A search result containing the tweak ID, match score, and matched indices
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// The tweak ID
    pub tweak_id: String,
    /// Fuzzy match score (higher is better)
    pub score: u32,
    /// Category ID for navigation
    pub category_id: String,
    /// Match indices for highlighting (character positions in the combined search text)
    pub match_indices: Vec<u32>,
}

/// Perform fuzzy search across all tweaks
///
/// Searches tweak name, description, and info fields concurrently.
/// Results are sorted by score (highest first).
#[tauri::command]
pub async fn fuzzy_search_tweaks(query: String) -> Result<Vec<SearchResult>> {
    log::debug!("Command: fuzzy_search_tweaks('{}')", query);

    // Empty query returns empty results
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Get all tweaks for current Windows version
    let windows_info = system_info_service::get_windows_info()?;
    let version = windows_info.version_number();
    let tweaks = tweak_loader::get_tweaks_for_version(version)?;

    let query = query.to_lowercase();

    // Process tweaks in parallel using rayon
    let mut results: Vec<SearchResult> = tweaks
        .par_iter()
        .filter_map(|(tweak_id, tweak)| {
            // Create thread-local matcher for each parallel task
            let mut matcher = Matcher::new(Config::DEFAULT);

            // Build searchable text: name + description + info
            // Weight name more heavily by including it multiple times
            let search_text = format!(
                "{} {} {} {}",
                tweak.name,
                tweak.name, // Double weight for name
                tweak.description,
                tweak.info.as_deref().unwrap_or("")
            )
            .to_lowercase();

            // Convert to UTF-32 for nucleo
            let haystack: Vec<char> = search_text.chars().collect();

            let haystack_str = Utf32Str::Unicode(&haystack);

            // Use Pattern for better multi-word matching
            let pattern = Pattern::new(
                &query,
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );

            // Get match with indices
            let mut indices = Vec::new();
            let score = pattern.indices(haystack_str, &mut matcher, &mut indices);

            score.map(|s| SearchResult {
                tweak_id: tweak_id.clone(),
                score: s,
                category_id: tweak.category_id.clone(),
                match_indices: indices,
            })
        })
        .collect();

    // Sort by score descending (highest first)
    results.sort_by(|a, b| b.score.cmp(&a.score));

    log::debug!(
        "Fuzzy search '{}' returned {} results",
        query,
        results.len()
    );

    Ok(results)
}
