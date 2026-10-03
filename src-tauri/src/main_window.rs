//! The main window outside its page: the first show, a startup failure report, and the shutdown
//! block reason while a change is in flight.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{AppHandle, Manager, Runtime, Webview};
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Shutdown::{ShutdownBlockReasonCreate, ShutdownBlockReasonDestroy};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_OK, MB_SETFOREGROUND,
};

use crate::tweaks::engine::lifecycle;

const LABEL: &str = "main";
const SHUTDOWN_REASON: &str = "A tweak or app is still being changed.";

static SHOWN: AtomicBool = AtomicBool::new(false);
static LOG_FOLDER: OnceLock<Option<PathBuf>> = OnceLock::new();
static APP: OnceLock<AppHandle> = OnceLock::new();

/// WebView2 completes a failed navigation too, so a page that never loads still shows its window.
pub fn on_page_load<R: Runtime>(webview: &Webview<R>, payload: &PageLoadPayload<'_>) {
    if payload.event() != PageLoadEvent::Finished || SHOWN.swap(true, Ordering::SeqCst) {
        return;
    }
    match webview.window().show() {
        Ok(()) => log::info!("main window shown"),
        Err(e) => {
            SHOWN.store(false, Ordering::SeqCst);
            log::error!("could not show the main window: {e}");
        }
    }
}

/// Release builds abort on a panic, so before the window shows this box is the only sign of one.
pub fn install_startup_failure_report() {
    let logged = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        logged(info);
        if !SHOWN.load(Ordering::SeqCst) {
            report_startup_failure(
                info.payload_as_str()
                    .unwrap_or("An unexpected error occurred."),
            );
        }
    }));
}

/// Read once, after the session file is attached: the panic hook must not take the logger's locks.
pub fn remember_log_folder() {
    let status = crate::logging::status();
    if LOG_FOLDER
        .set(status.folder.filter(|_| status.writing))
        .is_err()
    {
        log::warn!("the log folder was already recorded");
    }
}

pub fn report_startup_failure(cause: &str) {
    let mut text = format!("MagicX Toolbox could not start.\n\n{cause}");
    if let Some(Some(folder)) = LOG_FOLDER.get() {
        text.push_str(&format!(
            "\n\nThe session log in {} has the details.",
            folder.display()
        ));
    }
    if let Some(hint) = crate::commands::update::previous_version_hint() {
        text.push_str(&format!("\n\n{hint}"));
    }
    message_box(&text);
}

pub fn report_already_running() {
    message_box(
        "MagicX Toolbox is already running but did not show its window. Wait a moment and try \
         again, or end it in Task Manager.",
    );
}

/// A debug build logs the text instead, which the pipeline echoes to stderr.
fn message_box(text: &str) {
    if cfg!(debug_assertions) {
        log::error!("{text}");
        return;
    }
    let (text, title) = (
        crate::services::wide(text),
        crate::services::wide("MagicX Toolbox"),
    );
    // SAFETY: both strings are NUL-terminated and outlive the call; a null owner is allowed.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
        );
    }
}

/// Names the in-flight change on the shutdown screen while any tweak or app lock is held.
pub fn track_busy(app: &AppHandle) {
    if APP.set(app.clone()).is_err() {
        log::warn!("the shutdown block reason is already tracked");
        return;
    }
    lifecycle::on_busy_change(queue_block_reason);
}

fn queue_block_reason() {
    let Some(app) = APP.get() else { return };
    let handle = app.clone();
    if let Err(e) = app.run_on_main_thread(move || sync_block_reason(&handle)) {
        log::warn!("could not queue the shutdown block reason: {e}");
    }
}

/// Main thread only: ShutdownBlockReasonCreate fails off the window's own thread. Reads the gate
/// itself, so the order in which updates were queued does not matter.
fn sync_block_reason(app: &AppHandle) {
    static BLOCKING: AtomicBool = AtomicBool::new(false);
    let busy = lifecycle::gate().busy();
    if busy == BLOCKING.load(Ordering::SeqCst) {
        return;
    }
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let hwnd = match window.hwnd() {
        Ok(hwnd) => hwnd.0,
        Err(e) => {
            log::warn!("no window for the shutdown block reason: {e}");
            return;
        }
    };
    let reason = crate::services::wide(SHUTDOWN_REASON);
    // SAFETY: `hwnd` is this thread's live window; `reason` is NUL-terminated and outlives the call.
    let ok = unsafe {
        if busy {
            ShutdownBlockReasonCreate(hwnd, reason.as_ptr())
        } else {
            ShutdownBlockReasonDestroy(hwnd)
        }
    };
    if ok == 0 {
        // SAFETY: reads this thread's last error, set by the failed call above.
        let code = unsafe { GetLastError() };
        log::warn!("could not update the shutdown block reason (error {code})");
    } else {
        BLOCKING.store(busy, Ordering::SeqCst);
    }
}
