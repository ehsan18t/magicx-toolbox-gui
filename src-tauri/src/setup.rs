use crate::services::backup_service;
use tauri::App;

pub fn setup(_app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Validate all snapshots on startup
    // This removes stale snapshots where the tweak was externally reverted
    log::info!("Validating snapshots on startup...");
    match backup_service::validate_all_snapshots() {
        Ok(removed) => {
            if removed > 0 {
                log::info!("Removed {} stale snapshots", removed);
            } else {
                log::debug!("All snapshots are valid");
            }
        }
        Err(e) => {
            log::warn!("Failed to validate snapshots: {}", e);
            // Don't fail app startup for this
        }
    }

    Ok(())
}
