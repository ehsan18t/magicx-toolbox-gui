use tauri::App;

pub fn setup(_app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging for debug builds
    #[cfg(debug_assertions)]
    {
        use tauri_plugin_log::{Builder as LogBuilder, Target, TargetKind};

        let log_plugin = LogBuilder::new()
            .targets([
                Target::new(TargetKind::Webview),
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::LogDir { file_name: None }),
            ])
            .level(log::LevelFilter::Info)
            .build();
        _app.handle().plugin(log_plugin)?;
    }

    Ok(())
}
