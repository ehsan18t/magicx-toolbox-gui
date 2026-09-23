//! One GUI instance per session: the engine's locks are process-local, so a second instance would
//! race the first over the same snapshot and shared-claims files.

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, HANDLE, WAIT_ABANDONED,
    WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

/// Passed by restart-as-admin, whose predecessor still holds the mutex while it exits.
pub const AFTER_RESTART_ARG: &str = "--after-restart";
const MUTEX_NAME: &str = "Local\\MagicXToolbox.Gui";
const HANDOFF_WAIT_MS: u32 = 15_000;

pub enum Instance {
    First(InstanceGuard),
    AlreadyRunning,
}

pub struct InstanceGuard(Option<HANDLE>);

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.0 {
            // SAFETY: `handle` is a mutex this thread owns, released and closed exactly once.
            unsafe {
                ReleaseMutex(handle);
                CloseHandle(handle);
            }
        }
    }
}

pub fn acquire(after_restart: bool) -> Instance {
    acquire_named(MUTEX_NAME, after_restart)
}

fn acquire_named(name: &str, after_restart: bool) -> Instance {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is NUL-terminated and outlives the call; null attributes are the default.
    let (handle, err) = unsafe {
        let handle = CreateMutexW(std::ptr::null(), 1, wide.as_ptr());
        (handle, GetLastError())
    };
    if handle.is_null() {
        // An elevated instance's mutex denies a non-elevated opener, which is still "running".
        if err == ERROR_ACCESS_DENIED {
            return Instance::AlreadyRunning;
        }
        log::warn!("single-instance mutex unavailable (error {err}); starting without it");
        return Instance::First(InstanceGuard(None));
    }
    if err != ERROR_ALREADY_EXISTS {
        return Instance::First(InstanceGuard(Some(handle)));
    }
    if after_restart {
        // SAFETY: `handle` is a valid mutex handle owned by this function.
        let waited = unsafe { WaitForSingleObject(handle, HANDOFF_WAIT_MS) };
        if waited == WAIT_OBJECT_0 || waited == WAIT_ABANDONED {
            return Instance::First(InstanceGuard(Some(handle)));
        }
        log::warn!("previous instance did not exit within the restart handoff (wait {waited})");
    }
    // SAFETY: `handle` is valid and not owned by this thread; closed exactly once.
    unsafe { CloseHandle(handle) };
    Instance::AlreadyRunning
}

/// Brings the running instance's window forward; the launching process holds foreground rights.
pub fn focus_running_instance(window_title: &str) {
    let wide: Vec<u16> = window_title
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: `wide` is NUL-terminated; a null class name matches any class.
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), wide.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_name() -> String {
        format!(
            "Local\\MagicXToolbox.Test.{}.{:?}",
            std::process::id(),
            std::time::SystemTime::now()
        )
    }

    #[test]
    fn a_second_launch_finds_the_first() {
        let name = unique_name();
        let first = acquire_named(&name, false);
        assert!(matches!(first, Instance::First(InstanceGuard(Some(_)))));

        // Mutex ownership is per thread, so the contender must be another thread.
        let contender = name.clone();
        let second = std::thread::spawn(move || {
            matches!(acquire_named(&contender, false), Instance::AlreadyRunning)
        });
        assert!(second.join().unwrap());
        drop(first);
    }

    #[test]
    fn a_restarted_launch_takes_over_once_the_first_exits() {
        let name = unique_name();
        let first = acquire_named(&name, false);
        let successor = name.clone();
        let second = std::thread::spawn(move || {
            matches!(acquire_named(&successor, true), Instance::First(_))
        });
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(first);
        assert!(second.join().unwrap());
    }
}
