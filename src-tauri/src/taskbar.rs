//! Taskbar progress for long jobs (app removal and install, the update download). A taskbar call
//! that fails is logged and never fails the job.

use std::future::Future;
use std::sync::{Mutex, PoisonError};
use tauri::window::{ProgressBarState, ProgressBarStatus};
use tauri::{AppHandle, Manager, UserAttentionType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bar {
    Busy,
    Percent(u64),
    Failed,
    Clear,
}

#[derive(Debug, Default)]
struct Jobs {
    running: usize,
    failed: bool,
    // Recorded from focus events, so no getter runs under the lock.
    focused: bool,
}

impl Jobs {
    fn begin(&mut self) -> Option<Bar> {
        self.running += 1;
        (self.running == 1).then(|| {
            self.failed = false;
            Bar::Busy
        })
    }

    // A failure the focused window already shows leaves no red bar behind.
    fn end(&mut self, ok: bool) -> Option<Bar> {
        self.running = self.running.saturating_sub(1);
        self.failed |= !ok;
        if self.running > 0 {
            return None;
        }
        if self.failed && !self.focused {
            return Some(Bar::Failed);
        }
        self.failed = false;
        Some(Bar::Clear)
    }

    fn progress(&self, percent: u64) -> Option<Bar> {
        (self.running == 1).then_some(Bar::Percent(percent.min(100)))
    }

    fn focus_changed(&mut self, focused: bool) -> Option<Bar> {
        self.focused = focused;
        (focused && self.running == 0 && std::mem::take(&mut self.failed)).then_some(Bar::Clear)
    }
}

static JOBS: Mutex<Jobs> = Mutex::new(Jobs {
    running: 0,
    failed: false,
    focused: false,
});

// Shown under the JOBS lock so two jobs' updates cannot land out of order. Only setters run under
// it (they post to the event loop); a getter would wait on the main thread, which `on_focus` may hold.
fn show(app: &AppHandle, next: impl FnOnce(&mut Jobs) -> Option<Bar>) {
    let window = app.get_webview_window("main");
    let mut jobs = JOBS.lock().unwrap_or_else(PoisonError::into_inner);
    let (Some(bar), Some(window)) = (next(&mut jobs), window) else {
        return;
    };
    let (status, progress) = match bar {
        Bar::Busy => (ProgressBarStatus::Indeterminate, None),
        Bar::Percent(p) => (ProgressBarStatus::Normal, Some(p)),
        Bar::Failed => (ProgressBarStatus::Error, Some(100)),
        Bar::Clear => (ProgressBarStatus::None, None),
    };
    let state = ProgressBarState {
        status: Some(status),
        progress,
    };
    if let Err(e) = window.set_progress_bar(state) {
        log::warn!("taskbar progress not set: {e}");
    }
}

fn end(app: &AppHandle, ok: bool) {
    let mut background = false;
    show(app, |jobs| {
        background = !jobs.focused;
        jobs.end(ok)
    });
    if !background {
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.request_user_attention(Some(UserAttentionType::Informational)) {
            log::warn!("taskbar attention not requested: {e}");
        }
    }
}

/// Calls `end` once: with the recorded outcome, or as a failure when dropped early or by a panic.
struct EndGuard<F: FnMut(bool)> {
    end: F,
    ok: bool,
}

impl<F: FnMut(bool)> Drop for EndGuard<F> {
    fn drop(&mut self) {
        (self.end)(self.ok);
    }
}

/// Runs `work` with the taskbar busy, then flashes the taskbar button if the window is in the background.
pub async fn track<T, E>(
    app: &AppHandle,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    show(app, Jobs::begin);
    let mut guard = EndGuard {
        end: |ok| end(app, ok),
        ok: false,
    };
    let result = work.await;
    guard.ok = result.is_ok();
    result
}

/// Determinate progress for the one running job; ignored while jobs overlap.
pub fn progress(app: &AppHandle, percent: u64) {
    show(app, |jobs| jobs.progress(percent));
}

/// Clears a failure's red bar once the user returns to the window.
pub fn on_focus(app: &AppHandle, focused: bool) {
    show(app, |jobs| jobs.focus_changed(focused));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_clears_only_when_the_last_job_ends() {
        let mut jobs = Jobs::default();
        assert_eq!(jobs.begin(), Some(Bar::Busy));
        assert_eq!(jobs.begin(), None);
        assert_eq!(jobs.end(true), None);
        assert_eq!(jobs.end(true), Some(Bar::Clear));
        assert_eq!(jobs.running, 0);
    }

    #[test]
    fn an_overlapping_failure_shows_when_the_batch_ends_in_the_background() {
        let mut jobs = Jobs::default();
        jobs.begin();
        jobs.begin();
        assert_eq!(jobs.end(false), None);
        assert_eq!(jobs.end(true), Some(Bar::Failed));
        assert_eq!(jobs.focus_changed(true), Some(Bar::Clear));
        assert_eq!(jobs.focus_changed(true), None);
    }

    #[test]
    fn a_failure_in_the_focused_window_clears_at_once() {
        let mut jobs = Jobs::default();
        jobs.focus_changed(true);
        jobs.begin();
        assert_eq!(jobs.end(false), Some(Bar::Clear));
        assert_eq!(jobs.focus_changed(true), None);
    }

    #[test]
    fn focus_arriving_mid_job_is_seen_when_the_job_fails() {
        let mut jobs = Jobs::default();
        jobs.begin();
        assert_eq!(jobs.focus_changed(true), None);
        assert_eq!(jobs.end(false), Some(Bar::Clear));
    }

    #[test]
    fn a_new_job_replaces_a_shown_failure() {
        let mut jobs = Jobs::default();
        jobs.begin();
        jobs.end(false);
        assert_eq!(jobs.begin(), Some(Bar::Busy));
        assert_eq!(
            jobs.focus_changed(true),
            None,
            "focus must not clear a running job"
        );
        assert_eq!(jobs.end(true), Some(Bar::Clear));
    }

    #[test]
    fn progress_shows_only_for_a_sole_job() {
        let mut jobs = Jobs::default();
        assert_eq!(jobs.progress(10), None);
        jobs.begin();
        assert_eq!(jobs.progress(150), Some(Bar::Percent(100)));
        jobs.begin();
        assert_eq!(jobs.progress(10), None);
    }

    #[test]
    fn the_guard_ends_a_job_exactly_once() {
        let mut ended = Vec::new();
        drop(EndGuard {
            end: |ok| ended.push(ok),
            ok: false,
        });
        let mut guard = EndGuard {
            end: |ok| ended.push(ok),
            ok: false,
        };
        guard.ok = true;
        drop(guard);
        assert_eq!(ended, [false, true]);
    }

    #[test]
    fn a_panic_ends_the_job_as_a_failure() {
        let ended = std::sync::Mutex::new(None);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = EndGuard {
                end: |ok| *ended.lock().unwrap() = Some(ok),
                ok: false,
            };
            panic!("job panicked");
        }));
        assert_eq!(*ended.lock().unwrap(), Some(false));
    }
}
