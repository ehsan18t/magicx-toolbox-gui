use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

/// Global debug mode flag
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Debug log level
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DebugLevel {
    Info,
    Warn,
    Error,
    Success,
}

/// Debug log entry sent to frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugLogEntry {
    pub timestamp: String,
    pub level: DebugLevel,
    pub message: String,
    pub context: Option<String>,
}

/// Enable or disable debug mode
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Check if debug mode is enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::SeqCst)
}

/// Send a debug log to the frontend via Tauri event
pub fn emit_debug_log(app: &AppHandle, level: DebugLevel, message: &str, context: Option<&str>) {
    if !is_debug_enabled() {
        return;
    }

    let entry = DebugLogEntry {
        timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        level,
        message: message.to_string(),
        context: context.map(|s| s.to_string()),
    };

    // Emit to frontend
    let _ = app.emit("debug-log", entry);
}

/// Convenience macros for debug logging
#[macro_export]
macro_rules! debug_info {
    ($app:expr, $msg:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Info, $msg, None)
    };
    ($app:expr, $msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Info, $msg, Some($ctx))
    };
}

#[macro_export]
macro_rules! debug_warn {
    ($app:expr, $msg:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Warn, $msg, None)
    };
    ($app:expr, $msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Warn, $msg, Some($ctx))
    };
}

#[macro_export]
macro_rules! debug_error {
    ($app:expr, $msg:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Error, $msg, None)
    };
    ($app:expr, $msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Error, $msg, Some($ctx))
    };
}

#[macro_export]
macro_rules! debug_success {
    ($app:expr, $msg:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Success, $msg, None)
    };
    ($app:expr, $msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($app, $crate::debug::DebugLevel::Success, $msg, Some($ctx))
    };
}
