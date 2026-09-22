use tauri::{App, Manager};

use crate::commands::tweaks::TweakEngineState;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Register the handle debug events are emitted through. Held in debug.rs rather
    // than threaded through the apply chain as a parameter -- see the note on
    // DEBUG_APP there. Must happen before anything that might emit.
    crate::debug::set_debug_app(app.handle().clone());

    // SnapshotStore/ClaimsStore/ProbeCache: app-lifetime singletons, never re-opened per call.
    let tweak_state = TweakEngineState::new()?;
    // Crash-interrupted apply carry-forward (spec §8.1 invariant 5): flags any snapshot entry left
    // `intended && !completed` by a process that crashed mid-apply, before the frontend ever asks.
    tweak_state.scan_startup_crash_residue();
    app.manage(tweak_state);

    Ok(())
}
