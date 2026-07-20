//! Per-item mismatch report for the UI.
//!
//! Renders the shared comparison core ([`super::compare`]) into the serialized `TweakInspection`
//! the details modal consumes. The comparison logic itself lives in `compare`, shared with
//! `detection` — this module only maps its results into the frontend shape.

use crate::error::Error;
use crate::models::{OptionInspection, TweakDefinition, TweakInspection, TweakOption};
use rayon::prelude::*;

/// Inspect a tweak: for every option, the per-item match status vs current system state.
pub fn inspect_tweak(
    tweak: &TweakDefinition,
    windows_version: u32,
    current_option_index: Option<usize>,
    pending_option_index: Option<usize>,
) -> Result<TweakInspection, Error> {
    // Options are independent; inspect them in parallel.
    let options: Vec<OptionInspection> = tweak
        .options
        .par_iter()
        .enumerate()
        .map(|(index, option)| {
            inspect_option(
                index,
                option,
                windows_version,
                current_option_index == Some(index),
                pending_option_index == Some(index),
            )
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let matched_option_index = options.iter().position(|opt| opt.all_match);

    Ok(TweakInspection {
        tweak_id: tweak.id.clone(),
        options,
        matched_option_index,
    })
}

fn inspect_option(
    index: usize,
    option: &TweakOption,
    windows_version: u32,
    is_current: bool,
    is_pending: bool,
) -> Result<OptionInspection, Error> {
    let comparison = super::compare::compare_option(option, windows_version)?;
    let all_match = comparison.all_match();

    Ok(OptionInspection {
        option_index: index,
        label: option.label.clone(),
        is_current,
        is_pending,
        registry_results: comparison.registry,
        service_results: comparison.service,
        scheduler_results: comparison.scheduler,
        hosts_results: comparison.hosts,
        firewall_results: comparison.firewall,
        all_match,
    })
}
