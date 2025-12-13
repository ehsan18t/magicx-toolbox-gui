use crate::error::{Error, Result};
use crate::state::AppState;
use tauri::Manager;

#[tauri::command]
pub fn update_theme(new_theme: String, state: tauri::State<AppState>) -> Result<()> {
    let mut state = state.0.lock().map_err(|_| Error::StateLock)?;
    state.user_preferences.theme = new_theme;
    Ok(())
}

#[tauri::command]
pub fn get_theme(state: tauri::State<AppState>) -> Result<String> {
    let state = state.0.lock().map_err(|_| Error::StateLock)?;
    Ok(state.user_preferences.theme.clone())
}

/// Show the main window. Called by frontend when it's ready to display.
#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                log::debug!("Main window already visible - ignoring duplicate show request");
                return Ok(());
            }
            Ok(false) => {
                window
                    .show()
                    .map_err(|e| Error::WindowsApi(e.to_string()))?;
                log::info!("Main window shown (frontend signal)");
            }
            Err(e) => {
                log::warn!(
                    "Failed to check window visibility: {} - attempting to show anyway",
                    e
                );
                window
                    .show()
                    .map_err(|e| Error::WindowsApi(e.to_string()))?;
                log::info!("Main window shown (despite visibility check failure)");
            }
        }
    }
    Ok(())
}
