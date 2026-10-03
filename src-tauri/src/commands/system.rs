use crate::error::Result;
use crate::models::SystemReading;
use crate::services::system_info_service;

/// The live fields always; the WMI hardware read only when `with_hardware`.
#[tauri::command]
pub async fn get_system_info(with_hardware: bool) -> Result<SystemReading> {
    log::info!("Reading system information (hardware: {with_hardware})");
    tauri::async_runtime::spawn_blocking(move || {
        system_info_service::get_system_info(with_hardware)
    })
    .await?
}
