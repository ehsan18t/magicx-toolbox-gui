mod commands;
mod error;
mod models;
mod services;
mod setup;
mod state;

pub use error::Error;
pub use models::*;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .manage(AppState(Default::default()))
        .setup(setup::setup)
        .invoke_handler(tauri::generate_handler![
            commands::general::greet,
            commands::general::get_greetings,
            commands::general::clear_greetings,
            commands::general::update_theme,
            commands::general::get_theme,
            commands::system::get_system_info,
            commands::tweaks::get_available_tweaks,
            commands::tweaks::get_tweaks_by_category,
            commands::tweaks::get_tweak,
            commands::tweaks::get_tweak_status,
            commands::tweaks::apply_tweak,
            commands::tweaks::revert_tweak,
            commands::tweaks::batch_apply_tweaks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
