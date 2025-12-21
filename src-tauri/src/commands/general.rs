use crate::error::{Error, Result};
use tauri::Manager;

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
