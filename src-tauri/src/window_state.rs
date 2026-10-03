//! Restores the main window's size, position and maximized state across launches.

use tauri::plugin::TauriPlugin;
use tauri::Runtime;
use tauri_plugin_window_state::StateFlags;

// Not VISIBLE: the window starts hidden and shows when its page loads (main_window.rs).
const FLAGS: StateFlags = StateFlags::SIZE
    .union(StateFlags::POSITION)
    .union(StateFlags::MAXIMIZED);

/// Saves on RunEvent::Exit, before the process lets go of the single-instance mutex, so a relaunch
/// (which waits for its predecessor to exit) always reads the final state.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_window_state::Builder::new()
        .with_state_flags(FLAGS)
        .build()
}
