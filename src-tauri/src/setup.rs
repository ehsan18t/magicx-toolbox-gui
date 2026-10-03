use tauri::{App, Manager};

use crate::apps::AppsState;
use crate::commands::tweaks::TweakEngineState;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    crate::main_window::track_busy(app.handle());
    // SnapshotStore/ClaimsStore/ProbeCache: app-lifetime singletons, never re-opened per call.
    let tweak_state = TweakEngineState::new()?;
    // Crash-interrupted apply carry-forward (spec §8.1 invariant 5): flags any snapshot entry left
    // `intended && !completed` by a process that crashed mid-apply, before the frontend ever asks.
    tweak_state.scan_startup_crash_residue();
    app.manage(tweak_state);
    app.manage(AppsState::new());

    Ok(())
}
