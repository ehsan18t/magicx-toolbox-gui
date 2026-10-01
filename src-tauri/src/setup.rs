use tauri::{App, Manager};

use crate::apps::AppsState;
use crate::commands::tweaks::TweakEngineState;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    match app.path().app_local_data_dir() {
        Ok(dir) => crate::logging::start(dir),
        Err(e) => log::warn!("app data folder unavailable ({e}); logs stay in memory"),
    }

    // SnapshotStore/ClaimsStore/ProbeCache: app-lifetime singletons, never re-opened per call.
    let tweak_state = TweakEngineState::new()?;
    // Crash-interrupted apply carry-forward (spec §8.1 invariant 5): flags any snapshot entry left
    // `intended && !completed` by a process that crashed mid-apply, before the frontend ever asks.
    tweak_state.scan_startup_crash_residue();
    app.manage(tweak_state);
    app.manage(AppsState::new());

    Ok(())
}
