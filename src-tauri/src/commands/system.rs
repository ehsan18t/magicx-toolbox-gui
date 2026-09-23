use crate::error::Result;
use crate::models::SystemInfo;
use crate::services::system_info_service;

/// Get system information (Windows version, admin status, etc.)
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo> {
    log::info!("Reading system information");
    // WMI queries block; off the async worker.
    tauri::async_runtime::spawn_blocking(system_info_service::get_system_info).await?
}
