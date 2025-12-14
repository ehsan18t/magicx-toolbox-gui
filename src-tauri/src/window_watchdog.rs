use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Show window after timeout if it hasn't been shown already.
///
/// This watchdog provides a failsafe mechanism to ensure the window always becomes visible,
/// even if the frontend fails to mount or signal readiness. This prevents the app from
/// getting stuck in a hidden state running in the background.
///
/// # Arguments
/// * `app_handle` - Handle to the Tauri application
/// * `window_label` - Label of the window to watch (e.g., "main")
/// * `timeout_secs` - Number of seconds to wait before forcing window visibility
///
/// # Note
/// This function should be called from a spawned async task (e.g., `tauri::async_runtime::spawn`).
/// It uses `tokio::time::sleep` internally for non-blocking async sleep.
pub async fn start_window_watchdog(app_handle: AppHandle, window_label: &str, timeout_secs: u64) {
    let label = window_label.to_string();

    // Async sleep - doesn't block the thread pool
    tokio::time::sleep(Duration::from_secs(timeout_secs)).await;

    if let Some(window) = app_handle.get_webview_window(&label) {
        // Check if window is still hidden
        match window.is_visible() {
            Ok(false) => {
                log::warn!(
                    "Window '{}' still hidden after {}s timeout - forcing visibility",
                    label,
                    timeout_secs
                );
                if let Err(e) = window.show() {
                    log::error!("Failed to show window after timeout: {}", e);
                } else {
                    log::info!("Window '{}' shown via watchdog fallback", label);
                }
            }
            Ok(true) => {
                log::debug!("Window '{}' already visible - watchdog not needed", label);
            }
            Err(e) => {
                log::error!("Failed to check window visibility: {}", e);
            }
        }
    } else {
        log::error!("Window '{}' not found for watchdog check", label);
    }
}
