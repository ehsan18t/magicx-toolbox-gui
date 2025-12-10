use crate::debug::{is_debug_enabled, set_debug_enabled};

/// Enable or disable debug mode
#[tauri::command]
pub fn set_debug_mode(enabled: bool) {
    set_debug_enabled(enabled);
    log::info!(
        "Debug mode {}",
        if enabled { "enabled" } else { "disabled" }
    );
}

/// Get current debug mode status
#[tauri::command]
pub fn get_debug_mode() -> bool {
    is_debug_enabled()
}
