use std::sync::Mutex;

/// AppState that can be shared across the application.
/// Currently unused but kept for potential future use (e.g., caching, shared settings).
#[allow(dead_code)]
pub struct AppState(pub Mutex<State>);

#[derive(Default)]
#[allow(dead_code)]
pub struct State {
    pub user_preferences: UserPreferences,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct UserPreferences {
    pub theme: String,
}
