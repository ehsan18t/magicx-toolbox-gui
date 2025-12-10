use crate::error::Result;
use crate::state::AppState;

/// A sample command that can be invoked from the frontend.
#[tauri::command]
pub fn greet(name: &str, state: tauri::State<AppState>) -> Result<String> {
    let mut state = state.0.lock().expect("Failed to lock state mutex");

    log::info!("Received greeting request for: {}", name);
    state.greetings.push(name.to_string());
    log::info!("Greetings recorded: {:?}", state.greetings);

    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

/// Get all recorded greetings
#[tauri::command]
pub fn get_greetings(state: tauri::State<AppState>) -> Result<Vec<String>> {
    let state = state.0.lock().expect("Failed to lock state mutex");
    Ok(state.greetings.clone())
}

/// Clear all greetings
#[tauri::command]
pub fn clear_greetings(state: tauri::State<AppState>) -> Result<()> {
    let mut state = state.0.lock().expect("Failed to lock state mutex");
    state.greetings.clear();
    log::info!("All greetings cleared");
    Ok(())
}

#[tauri::command]
pub fn update_theme(new_theme: String, state: tauri::State<AppState>) -> Result<()> {
    let mut state = state.0.lock().expect("Failed to lock state mutex");
    state.user_preferences.theme = new_theme;
    Ok(())
}

#[tauri::command]
pub fn get_theme(state: tauri::State<AppState>) -> Result<String> {
    let state = state.0.lock().expect("Failed to lock state mutex");
    Ok(state.user_preferences.theme.clone())
}
