//! Restores the main window's size, position and maximized state across launches.

use std::sync::mpsc;
use std::time::Duration;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

// Not VISIBLE: the window starts hidden and shows when its page loads (main_window.rs).
const FLAGS: StateFlags = StateFlags::SIZE
    .union(StateFlags::POSITION)
    .union(StateFlags::MAXIMIZED);

const SAVE_WAIT: Duration = Duration::from_secs(2);

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_window_state::Builder::new()
        .with_state_flags(FLAGS)
        .build()
}

/// Before a relaunch: the new instance reads the state before this one exits and the plugin saves.
pub fn save(app: &AppHandle) {
    // On the main thread: the plugin holds its cache lock across window getters, which off it wait on
    // the main thread, whose Moved/Resized handler waits on that lock. From the main thread it runs inline.
    let (tx, rx) = mpsc::channel();
    let handle = app.clone();
    let queued = app.run_on_main_thread(move || {
        if let Err(unsent) = tx.send(handle.save_window_state(FLAGS)) {
            if let Err(e) = unsent.0 {
                log::warn!("could not save the window size and position: {e}");
            }
        }
    });
    if let Err(e) = queued {
        log::warn!("could not queue the window size and position save: {e}");
        return;
    }
    match rx.recv_timeout(SAVE_WAIT) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => log::warn!("could not save the window size and position: {e}"),
        Err(_) => log::warn!("window size and position save still pending; relaunching anyway"),
    }
}
