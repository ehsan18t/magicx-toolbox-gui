use tauri::App;

pub fn setup(_app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Logging is configured in lib.rs via tauri_plugin_log
    // Additional setup tasks can be added here
    Ok(())
}
