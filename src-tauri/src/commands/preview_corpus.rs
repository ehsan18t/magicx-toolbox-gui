//! `src/lib/preview/corpus.json`: the catalogue subset the `?preview` harness serves, built by the
//! same projections `get_categories`, `get_tweaks` and `get_apps` return.
//! Regenerate after a YAML change: `cargo test dump_preview_corpus -- --ignored`.

use std::path::PathBuf;

use serde_json::{json, Value};

use super::apps::app_view;
use super::tweaks::{category_view, effect_display_name, tweak_view};
use crate::tweaks::engine::context::SidCheck;
use crate::tweaks::model::Level;
use crate::tweaks::winver::WinVer;
use crate::tweaks::{compiled_apps, compiled_corpus};

// fixtures.ts and index.ts name ids from these lists; keep them in step.
const CATEGORIES: &[&str] = &[
    "debloat",
    "interface",
    "performance",
    "privacy",
    "security",
    "windows_update",
];
const TWEAKS: &[&str] = &[
    "disable_scoobe_nag",
    "disable_nag_toasts",
    "disable_widgets",
    "disable_web_search_start",
    "disable_auto_install_sponsored_apps",
    "disable_welcome_experience",
    "disable_edge_sidebar",
    "disable_start_suggestions",
    "disable_edge_startup_boost",
    "alt_tab_hide_browser_tabs",
    "taskbar_search_mode_win11",
    "disable_mouse_acceleration",
    "system_responsiveness",
    "disable_vbs_hvci",
    "disable_spectre_meltdown",
    "enable_gpu_scheduling",
    "variable_refresh_rate",
    "ultimate_performance_power_plan",
    "disable_search_indexing",
    "memory_prefetch_mode",
    "optimize_visual_effects",
    "disable_fullscreen_optimizations",
    "disable_ceip_tasks",
    "disable_compat_appraiser",
    "disable_diagnostic_data",
    "disable_wer_service",
    "disable_location_tracking",
    "disable_activity_history",
    "disable_consumer_features",
    "disable_explorer_cloud_recommendations",
    "disable_language_list_access",
    "disable_lockscreen_spotlight_ads",
    "disable_remote_registry",
    "enable_credential_guard",
    "defender_cloud_protection",
    "asr_block_office_script_vectors",
    "asr_block_lsass_theft",
    "enable_controlled_folder_access",
    "powershell_module_transcript_logging",
    "block_vulnerable_drivers",
    "remove_smbv1",
    "disable_admin_shares",
    "kernel_dma_protection",
    "lock_on_inactivity",
    "enable_network_protection",
    "block_update_pipeline",
    "windows_update_mode",
    "update_feature_control",
    "defer_feature_updates",
];
const APPS: &[&str] = &[
    "teams_consumer",
    "clipchamp",
    "quick_assist",
    "bing_news",
    "bing_weather",
    "solitaire",
    "get_help",
    "getstarted_tips",
    "feedback_hub",
    "phone_link",
    "outlook_new",
    "xbox_game_bar",
    "onedrive",
];

/// The build `fixtures.ts` reports.
const WINVER: WinVer = WinVer {
    build: 26200,
    revision: 6584,
};

fn pick<'a, T>(items: &'a [T], ids: &[&str], id: fn(&T) -> &str, what: &str) -> Vec<&'a T> {
    for want in ids {
        assert!(
            items.iter().any(|i| id(i) == *want),
            "preview {what} '{want}' is not in the catalogue"
        );
    }
    items.iter().filter(|i| ids.contains(&id(i))).collect()
}

/// Availability and support depend on the machine; the preview computes its own.
fn without(view: impl serde::Serialize, keys: &[&str]) -> Value {
    let mut v = serde_json::to_value(view).unwrap();
    let obj = v.as_object_mut().unwrap();
    for k in keys {
        obj.remove(*k);
    }
    v
}

fn generate() -> Value {
    let corpus = compiled_corpus();
    let (level, sid) = (Level::Admin, SidCheck::SameUser);
    let categories: Vec<_> = pick(&corpus.categories, CATEGORIES, |c| &c.id, "category")
        .into_iter()
        .map(category_view)
        .collect();
    let tweaks: Vec<_> = pick(&corpus.tweaks, TWEAKS, |t| &t.id, "tweak")
        .into_iter()
        .map(|t| {
            let view = tweak_view(t, corpus, &WINVER, level, sid);
            let mut v = without(view, &["availability", "supported"]);
            v["surface"] = t
                .surface
                .iter()
                .map(|e| json!({ "id": e.id, "name": effect_display_name(e) }))
                .collect();
            v
        })
        .collect();
    let apps: Vec<_> = pick(compiled_apps(), APPS, |a| &a.id, "app")
        .into_iter()
        .map(|a| {
            let view = app_view(a, level, sid, &WINVER);
            without(
                view,
                &["remove_availability", "install_availability", "supported"],
            )
        })
        .collect();
    json!({ "categories": categories, "tweaks": tweaks, "apps": apps })
}

fn corpus_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/lib/preview/corpus.json")
}

#[test]
#[ignore = "writes src/lib/preview/corpus.json"]
fn dump_preview_corpus() {
    let text = serde_json::to_string_pretty(&generate()).unwrap();
    std::fs::write(corpus_path(), text.replace('\n', "\r\n") + "\r\n").unwrap();
}

#[test]
fn preview_corpus_matches_the_catalogue() {
    let committed = std::fs::read_to_string(corpus_path()).unwrap();
    let committed: Value = serde_json::from_str(&committed).unwrap();
    assert!(
        committed == generate(),
        "src/lib/preview/corpus.json is stale: run `cargo test dump_preview_corpus -- --ignored`"
    );
}
