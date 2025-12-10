use crate::error::Result;
use crate::models::SystemInfo;
use crate::services::system_info_service;

/// Get system information (Windows version, admin status, etc.)
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo> {
    let info = system_info_service::get_system_info()?;
    Ok(info)
}
