use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

/// Global debug mode flag
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// The handle used to emit debug events to the frontend, set once during setup. Held here rather
/// than threaded through the apply chain: `AppHandle` is `AppHandle<Wry>` and cannot be constructed
/// outside a running app, so any signature carrying it is uncallable from a test.
static DEBUG_APP: OnceLock<AppHandle> = OnceLock::new();

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

/// Register the handle used to emit debug events. Called once, during setup.
pub fn set_debug_app(app: AppHandle) {
    let _ = DEBUG_APP.set(app);
}

/// A no-op when debug mode is off, and when no handle is registered (the state under `cargo test`).
pub fn emit_debug_log(level: DebugLevel, message: &str, context: Option<&str>) {
    if !is_debug_enabled() {
        return;
    }

    let Some(app) = DEBUG_APP.get() else {
        return;
    };

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
    ($msg:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Info, $msg, None)
    };
    ($msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Info, $msg, Some($ctx))
    };
}

#[macro_export]
macro_rules! debug_warn {
    ($msg:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Warn, $msg, None)
    };
    ($msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Warn, $msg, Some($ctx))
    };
}

#[macro_export]
macro_rules! debug_error {
    ($msg:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Error, $msg, None)
    };
    ($msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Error, $msg, Some($ctx))
    };
}

#[macro_export]
macro_rules! debug_success {
    ($msg:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Success, $msg, None)
    };
    ($msg:expr, $ctx:expr) => {
        $crate::debug::emit_debug_log($crate::debug::DebugLevel::Success, $msg, Some($ctx))
    };
}
