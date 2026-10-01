//! One GUI instance per session: the engine's locks are process-local, so a second instance would
//! race the first over the same snapshot and shared-claims files.

use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, HANDLE, WAIT_ABANDONED,
    WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

/// Passed by restart-as-admin and a downloaded update, whose predecessor still holds the mutex
/// while it exits.
pub const AFTER_RESTART_ARG: &str = "--after-restart";
const MUTEX_NAME: &str = "Local\\MagicXToolbox.Gui";
const HANDOFF_WAIT: Duration = Duration::from_secs(15);
const DENIED_RETRY: Duration = Duration::from_millis(100);

pub enum Instance {
    First(InstanceGuard),
    AlreadyRunning,
}

pub struct InstanceGuard {
    handle: Option<HANDLE>,
    note: Option<String>,
}

impl InstanceGuard {
    /// A warning raised before the log plugin exists, for the setup hook to log.
    pub fn take_note(&mut self) -> Option<String> {
        self.note.take()
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle {
            // SAFETY: `handle` is a mutex this thread owns, released and closed exactly once.
            unsafe {
                ReleaseMutex(handle);
                CloseHandle(handle);
            }
        }
    }
}

pub fn acquire(after_restart: bool) -> Instance {
    acquire_named(MUTEX_NAME, after_restart, HANDOFF_WAIT)
}

fn owned(handle: HANDLE) -> Instance {
    Instance::First(InstanceGuard {
        handle: Some(handle),
        note: None,
    })
}

fn unguarded(note: String) -> Instance {
    Instance::First(InstanceGuard {
        handle: None,
        note: Some(note),
    })
}

fn acquire_named(name: &str, after_restart: bool, handoff: Duration) -> Instance {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let deadline = Instant::now() + handoff;
    loop {
        // SAFETY: `wide` is NUL-terminated and outlives the call; null attributes are the default.
        let (handle, err) = unsafe {
            let handle = CreateMutexW(std::ptr::null(), 1, wide.as_ptr());
            (handle, GetLastError())
        };
        if handle.is_null() {
            if err != ERROR_ACCESS_DENIED {
                return unguarded(format!(
                    "single-instance mutex unavailable (error {err}); starting without it"
                ));
            }
            // An elevated or other-user instance's mutex denies this token, and it is still running.
            if !after_restart {
                return Instance::AlreadyRunning;
            }
            // Restart as administrator from a standard account: denied until the predecessor exits.
            if Instant::now() >= deadline {
                return unguarded(
                    "previous instance kept the single-instance mutex past the restart handoff; \
                     starting without it"
                        .into(),
                );
            }
            std::thread::sleep(DENIED_RETRY);
            continue;
        }
        if err != ERROR_ALREADY_EXISTS {
            return owned(handle);
        }
        if !after_restart {
            // SAFETY: `handle` is valid and not owned by this thread; closed exactly once.
            unsafe { CloseHandle(handle) };
            return Instance::AlreadyRunning;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        let remaining_ms = u32::try_from(remaining.as_millis()).unwrap_or(u32::MAX);
        // SAFETY: `handle` is a valid mutex handle owned by this function.
        let waited = unsafe { WaitForSingleObject(handle, remaining_ms) };
        if waited == WAIT_OBJECT_0 || waited == WAIT_ABANDONED {
            return owned(handle);
        }
        // SAFETY: `handle` is valid and not owned by this thread; closed exactly once.
        unsafe { CloseHandle(handle) };
        // The predecessor is exiting and refuses applies, so a window beats no window.
        return unguarded(format!(
            "previous instance did not exit within the restart handoff (wait {waited}); \
             starting without the single-instance mutex"
        ));
    }
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

    fn owns_mutex(instance: &Instance) -> bool {
        matches!(
            instance,
            Instance::First(InstanceGuard {
                handle: Some(_),
                ..
            })
        )
    }

    /// Contends from another thread (mutex ownership is per thread) and reports when it returned.
    fn contend(name: &str, after_restart: bool) -> std::thread::JoinHandle<(bool, Instant)> {
        let name = name.to_owned();
        std::thread::spawn(move || {
            let instance = acquire_named(&name, after_restart, Duration::from_secs(10));
            (owns_mutex(&instance), Instant::now())
        })
    }

    #[test]
    fn a_second_launch_finds_the_first() {
        let name = unique_name();
        let first = acquire_named(&name, false, HANDOFF_WAIT);
        assert!(owns_mutex(&first));
        let contender = name.clone();
        let second = std::thread::spawn(move || {
            matches!(
                acquire_named(&contender, false, HANDOFF_WAIT),
                Instance::AlreadyRunning
            )
        });
        assert!(second.join().unwrap());
        drop(first);
    }

    #[test]
    fn a_restarted_launch_takes_over_once_the_first_exits() {
        let name = unique_name();
        let first = acquire_named(&name, false, HANDOFF_WAIT);
        let second = contend(&name, true);
        std::thread::sleep(Duration::from_millis(300));
        let released_at = Instant::now();
        drop(first);
        let (owns, acquired_at) = second.join().unwrap();
        assert!(owns);
        assert!(acquired_at >= released_at, "took over before the release");
    }

    #[test]
    fn a_restarted_launch_starts_anyway_when_the_handoff_expires() {
        let name = unique_name();
        let first = acquire_named(&name, false, HANDOFF_WAIT);
        let contender = name.clone();
        let second = std::thread::spawn(move || {
            match acquire_named(&contender, true, Duration::from_millis(200)) {
                Instance::First(mut guard) => guard.handle.is_none() && guard.take_note().is_some(),
                Instance::AlreadyRunning => false,
            }
        });
        assert!(second.join().unwrap());
        drop(first);
    }

    #[test]
    fn a_restarted_launch_waits_out_a_mutex_its_token_is_denied() {
        use windows_sys::Win32::Security::{
            InitializeAcl, InitializeSecurityDescriptor, SetSecurityDescriptorDacl, ACL,
            ACL_REVISION, SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR,
        };
        const SECURITY_DESCRIPTOR_REVISION: u32 = 1;
        let name = unique_name();
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut acl = [0u64; 4];
        let mut sd = SECURITY_DESCRIPTOR::default();
        // SAFETY: an empty DACL denies every open; only the creating handle keeps access. Both
        // buffers outlive CreateMutexW, and `acl` is 8-byte aligned.
        let held = unsafe {
            let acl = acl.as_mut_ptr().cast::<ACL>();
            assert_ne!(InitializeAcl(acl, 32, ACL_REVISION), 0);
            let psd = (&mut sd as *mut SECURITY_DESCRIPTOR).cast();
            assert_ne!(
                InitializeSecurityDescriptor(psd, SECURITY_DESCRIPTOR_REVISION),
                0
            );
            assert_ne!(SetSecurityDescriptorDacl(psd, 1, acl, 0), 0);
            let sa = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: psd,
                bInheritHandle: 0,
            };
            CreateMutexW(&sa, 1, wide.as_ptr())
        };
        assert!(!held.is_null());

        let plain = name.clone();
        let refused = std::thread::spawn(move || {
            matches!(
                acquire_named(&plain, false, HANDOFF_WAIT),
                Instance::AlreadyRunning
            )
        });
        assert!(refused.join().unwrap(), "the DACL must deny a plain launch");

        let second = contend(&name, true);
        std::thread::sleep(Duration::from_millis(300));
        let released_at = Instant::now();
        // SAFETY: `held` is this thread's mutex, released and closed exactly once.
        unsafe {
            ReleaseMutex(held);
            CloseHandle(held);
        }
        let (owns, acquired_at) = second.join().unwrap();
        assert!(owns);
        assert!(acquired_at >= released_at, "took over before the release");
    }
}
