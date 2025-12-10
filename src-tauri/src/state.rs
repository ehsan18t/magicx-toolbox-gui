use std::sync::Mutex;

// AppState that can be shared across the application
pub struct AppState(pub Mutex<State>);

#[derive(Default)]
pub struct State {
    pub greetings: Vec<String>,
    pub user_preferences: UserPreferences,
}

#[derive(Default)]
pub struct UserPreferences {
    pub theme: String,
}
