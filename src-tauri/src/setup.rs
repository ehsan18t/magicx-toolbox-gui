use crate::services::backup_service;
use tauri::App;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Register the handle debug events are emitted through. Held in debug.rs rather
    // than threaded through the apply chain as a parameter -- see the note on
    // DEBUG_APP there. Must happen before anything that might emit.
    crate::debug::set_debug_app(app.handle().clone());

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
