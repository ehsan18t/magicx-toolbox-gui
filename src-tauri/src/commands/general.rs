use crate::error::{Error, Result};
use crate::state::AppState;

#[tauri::command]
pub fn update_theme(new_theme: String, state: tauri::State<AppState>) -> Result<()> {
    let mut state = state.0.lock().map_err(|_| Error::StateLock)?;
    state.user_preferences.theme = new_theme;
    Ok(())
}

#[tauri::command]
pub fn get_theme(state: tauri::State<AppState>) -> Result<String> {
    let state = state.0.lock().map_err(|_| Error::StateLock)?;
    Ok(state.user_preferences.theme.clone())
}
