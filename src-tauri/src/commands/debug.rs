use crate::debug::set_debug_enabled;

/// Enable or disable debug mode
#[tauri::command]
pub fn set_debug_mode(enabled: bool) {
    set_debug_enabled(enabled);
    log::info!(
        "Debug mode {}",
        if enabled { "enabled" } else { "disabled" }
    );
}
