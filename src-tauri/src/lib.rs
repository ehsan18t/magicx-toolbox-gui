mod commands;
pub mod debug;
mod error;
mod models;
mod services;
mod setup;
mod state;

pub use debug::{emit_debug_log, is_debug_enabled, set_debug_enabled, DebugLevel, DebugLogEntry};
pub use error::Error;
pub use models::*;
use state::AppState;
use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    // Log to console in debug builds only
                    Target::new(TargetKind::Stdout),
                    // Log to webview console for frontend debugging
                    Target::new(TargetKind::Webview),
                ])
                // In debug mode: show debug level and above
                // In release mode: show warn level and above
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Warn
                })
                // More verbose for our own crate
                .level_for(
                    "app_lib",
                    if cfg!(debug_assertions) {
                        log::LevelFilter::Trace
                    } else {
                        log::LevelFilter::Info
                    },
                )
                // Use colored output format with ANSI colors
                .format(|out, message, record| {
                    // Color codes for different log levels
                    let color = match record.level() {
                        log::Level::Error => "\x1b[31m", // Red
                        log::Level::Warn => "\x1b[33m",  // Yellow
                        log::Level::Info => "\x1b[32m",  // Green
                        log::Level::Debug => "\x1b[36m", // Cyan
                        log::Level::Trace => "\x1b[35m", // Magenta
                    };
                    let reset = "\x1b[0m";

                    out.finish(format_args!(
                        "{}[{}][{}]{}[{}] {}",
                        color,
                        chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                        record.target(),
                        reset,
                        record.level(),
                        message
                    ))
                })
                .build(),
        )
        .manage(AppState(Default::default()))
        .setup(|app| {
            log::info!("Application starting...");
            log::debug!("Debug logging enabled");
            setup::setup(app)
        })
        .invoke_handler(tauri::generate_handler![
            commands::general::greet,
            commands::general::get_greetings,
            commands::general::clear_greetings,
            commands::general::update_theme,
            commands::general::get_theme,
            commands::system::get_system_info,
            commands::tweaks::get_categories,
            commands::tweaks::get_available_tweaks,
            commands::tweaks::get_available_tweaks_for_version,
            commands::tweaks::get_tweaks_by_category,
            commands::tweaks::get_tweak,
            commands::tweaks::get_tweak_status,
            commands::tweaks::apply_tweak,
            commands::tweaks::revert_tweak,
            commands::tweaks::batch_apply_tweaks,
            commands::debug::set_debug_mode,
            commands::debug::get_debug_mode,
            // Backup commands
            commands::backup::has_backup,
            commands::backup::list_backups,
            commands::backup::get_backup_info,
            commands::backup::restore_key_to_baseline,
            commands::backup::get_tweak_conflicts,
            commands::backup::run_backup_diagnostics,
            commands::backup::migrate_legacy_backups,
            commands::backup::reset_backup_state,
            commands::backup::get_baseline_entry,
            commands::backup::get_backup_system_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
