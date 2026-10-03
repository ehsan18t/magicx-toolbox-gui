//! One GUI instance per session: the engine's locks are process-local, so a second instance would
//! race the first over the same snapshot and shared-claims files.

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStringExt;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, FILETIME, HANDLE, HWND,
    INVALID_HANDLE_VALUE, LPARAM, WAIT_ABANDONED, WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, GetProcessTimes, OpenProcess, ReleaseMutex,
    WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

const AFTER_RESTART_ARG: &str = "--after-restart=";
const MUTEX_NAME: &str = "Local\\MagicXToolbox.Gui";
const PREDECESSOR_WAIT: Duration = Duration::from_secs(30);
const EXITING_WAIT: Duration = Duration::from_secs(5);
const DENIED_RETRY: Duration = Duration::from_millis(100);
const WINDOW_CLASS: &str = "Tauri Window";
const WINDOW_TITLE: &str = "MagicX Toolbox";

/// For a relaunch of this exe: the new instance waits for this process to exit.
pub fn after_restart_arg() -> String {
    format!("{AFTER_RESTART_ARG}{}", std::process::id())
}

#[derive(Debug, PartialEq, Eq)]
pub enum Launch {
    Plain,
    AfterRestart(u32),
}

/// argv[1] only, where both relaunch paths put it. Without a usable PID it is a plain launch.
pub fn classify(args: &[OsString]) -> Launch {
    args.get(1)
        .and_then(|a| a.to_str())
        .and_then(|a| a.strip_prefix(AFTER_RESTART_ARG))
        .and_then(|pid| pid.parse().ok())
        .filter(|&pid| pid != 0)
        .map_or(Launch::Plain, Launch::AfterRestart)
}

pub enum Instance {
    First(InstanceGuard),
    /// The running instance's window was brought forward, or is still starting.
    Focused,
    /// Another instance holds the mutex but showed no window in time.
    Unresponsive,
}

pub struct InstanceGuard {
    handle: Option<HANDLE>,
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

/// A plain launch brings a running instance forward and exits; with no window to find, the holder
/// is starting or exiting, so this one waits briefly for the mutex. A relaunch never focuses the
/// predecessor: it waits for that process to exit, then for the mutex.
pub fn acquire(launch: Launch) -> Instance {
    match launch {
        Launch::Plain => acquire_named(MUTEX_NAME, EXITING_WAIT, focus_running_instance),
        Launch::AfterRestart(pid) => {
            if !wait_for_exit(pid, PREDECESSOR_WAIT, created(own_process())) {
                log::warn!(
                    "the previous instance (pid {pid}) is still running after the restart handoff"
                );
            }
            acquire_named(MUTEX_NAME, EXITING_WAIT, || false)
        }
    }
}

fn own_process() -> HANDLE {
    // SAFETY: the pseudo-handle needs no closing.
    unsafe { GetCurrentProcess() }
}

/// The process's creation time in FILETIME ticks, or `None` when it cannot be read.
fn created(process: HANDLE) -> Option<u64> {
    let [mut creation, mut exit, mut kernel, mut user] = [FILETIME::default(); 4];
    // SAFETY: `process` is open with query rights; the four out-params are live locals.
    let ok =
        unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) } != 0;
    ok.then(|| (u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime))
}

/// False only when the process is still running at `timeout`. One already gone, or created after
/// `started_by` (its PID was reused, so it is not the predecessor), counts as exited.
fn wait_for_exit(pid: u32, timeout: Duration, started_by: Option<u64>) -> bool {
    // SAFETY: a failed open returns null and is not used; an opened handle is closed exactly once.
    unsafe {
        let process = OpenProcess(
            PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        );
        if process.is_null() {
            return true;
        }
        let reused = matches!(
            (created(process), started_by),
            (Some(theirs), Some(ours)) if theirs > ours
        );
        let exited = reused || WaitForSingleObject(process, millis(timeout)) == WAIT_OBJECT_0;
        CloseHandle(process);
        exited
    }
}

fn millis(d: Duration) -> u32 {
    u32::try_from(d.as_millis()).unwrap_or(u32::MAX)
}

fn owned(handle: HANDLE) -> Instance {
    Instance::First(InstanceGuard {
        handle: Some(handle),
    })
}

fn unguarded(note: String) -> Instance {
    log::warn!("{note}");
    Instance::First(InstanceGuard { handle: None })
}

/// `found_window` reports (and focuses) the holder's window, which settles the launch at once.
fn acquire_named(name: &str, wait: Duration, found_window: impl Fn() -> bool) -> Instance {
    let wide = super::wide(name);
    let deadline = Instant::now() + wait;
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
            // An elevated or other-user instance holds it: this token cannot open it to wait on.
            if found_window() {
                return Instance::Focused;
            }
            if Instant::now() >= deadline {
                return Instance::Unresponsive;
            }
            std::thread::sleep(DENIED_RETRY);
            continue;
        }
        if err != ERROR_ALREADY_EXISTS {
            return owned(handle);
        }
        let settled = if found_window() {
            Instance::Focused
        } else {
            let remaining = deadline.saturating_duration_since(Instant::now());
            // SAFETY: `handle` is a valid mutex handle owned by this function.
            let waited = unsafe { WaitForSingleObject(handle, millis(remaining)) };
            if waited == WAIT_OBJECT_0 || waited == WAIT_ABANDONED {
                return owned(handle);
            }
            log::warn!("another instance held the single-instance mutex with no window to show");
            Instance::Unresponsive
        };
        // SAFETY: `handle` is valid and not owned by this thread; closed exactly once.
        unsafe { CloseHandle(handle) };
        return settled;
    }
}

/// False when the running instance has no window yet or any more.
fn focus_running_instance() -> bool {
    let Some(hwnd) = running_window() else {
        return false;
    };
    // SAFETY: `hwnd` came from EnumWindows; a window destroyed since only fails these calls.
    unsafe {
        // Hidden: still starting, and it shows itself once its page loads.
        if IsWindowVisible(hwnd) == 0 {
            return true;
        }
        // SW_RESTORE un-maximizes a maximized window, so only a minimized one gets it.
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        SetForegroundWindow(hwnd);
    }
    true
}

struct Search {
    own: u32,
    pids: Vec<u32>,
    by_exe: HWND,
    by_title: HWND,
}

/// The Tauri window of another process running this exe; failing that (a renamed copy), another
/// process's Tauri window with this app's title. A title alone matches any app's window.
fn running_window() -> Option<HWND> {
    let pids = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.file_name().map(processes_named))
        .unwrap_or_default();
    let mut search = Search {
        own: std::process::id(),
        pids,
        by_exe: std::ptr::null_mut(),
        by_title: std::ptr::null_mut(),
    };
    // SAFETY: `search` outlives EnumWindows, which calls `visit` synchronously on this thread.
    unsafe { EnumWindows(Some(visit), &mut search as *mut Search as LPARAM) };
    [search.by_exe, search.by_title]
        .into_iter()
        .find(|hwnd| !hwnd.is_null())
}

fn text_of(buf: &[u16], len: i32) -> OsString {
    OsString::from_wide(&buf[..usize::try_from(len).unwrap_or(0).min(buf.len())])
}

unsafe extern "system" fn visit(hwnd: HWND, lparam: LPARAM) -> i32 {
    // SAFETY: `lparam` is the `Search` `running_window` passed, alive for the enumeration.
    let search = unsafe { &mut *(lparam as *mut Search) };
    let (mut pid, mut class, mut title) = (0, [0u16; 64], [0u16; 64]);
    // SAFETY: every out-param is a live local sized as passed.
    let (class_len, title_len) = unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
        (
            GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32),
            GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32),
        )
    };
    if pid == search.own || text_of(&class, class_len) != WINDOW_CLASS {
        return 1;
    }
    if search.pids.contains(&pid) {
        search.by_exe = hwnd;
        return 0;
    }
    if search.by_title.is_null() && text_of(&title, title_len) == WINDOW_TITLE {
        search.by_title = hwnd;
    }
    1
}

/// Other processes whose image has this file name, compared case-insensitively.
fn processes_named(exe: &OsStr) -> Vec<u32> {
    let own = std::process::id();
    let wanted = exe.to_string_lossy().to_lowercase();
    let mut pids = Vec::new();
    // SAFETY: the snapshot handle is checked and closed exactly once; `entry` is sized as required.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return pids;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut more = Process32FirstW(snapshot, &mut entry) != 0;
        while more {
            let name = &entry.szExeFile;
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            if entry.th32ProcessID != own
                && OsString::from_wide(&name[..len])
                    .to_string_lossy()
                    .to_lowercase()
                    == wanted
            {
                pids.push(entry.th32ProcessID);
            }
            more = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
    }
    pids
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

    fn outcome(instance: &Instance) -> &'static str {
        match instance {
            Instance::First(InstanceGuard { handle: Some(_) }) => "owned",
            Instance::First(_) => "unguarded",
            Instance::Focused => "focused",
            Instance::Unresponsive => "unresponsive",
        }
    }

    /// Contends from another thread (mutex ownership is per thread) and reports when it returned.
    fn contend(
        name: &str,
        wait: Duration,
        window: bool,
    ) -> std::thread::JoinHandle<(&'static str, Instant)> {
        let name = name.to_owned();
        std::thread::spawn(move || {
            let instance = acquire_named(&name, wait, || window);
            (outcome(&instance), Instant::now())
        })
    }

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_restart_argument_names_the_predecessor() {
        assert_eq!(
            classify(&args(&["app.exe", &after_restart_arg(), "--log-persist=1"])),
            Launch::AfterRestart(std::process::id())
        );
        for plain in [
            &["app.exe"][..],
            &["app.exe", "--after-restart"],
            &["app.exe", "--after-restart="],
            &["app.exe", "--after-restart=x"],
            &["app.exe", "--after-restart=0"],
            &["app.exe", "--after-restarted=5"],
            &["app.exe", "x", "--after-restart=5"],
        ] {
            assert_eq!(classify(&args(plain)), Launch::Plain, "{plain:?}");
        }
    }

    #[test]
    fn a_second_launch_with_a_window_to_focus_exits_at_once() {
        let name = unique_name();
        let first = acquire_named(&name, Duration::ZERO, || false);
        assert_eq!(outcome(&first), "owned");
        let started = Instant::now();
        let (outcome, _) = contend(&name, Duration::from_secs(10), true)
            .join()
            .unwrap();
        assert_eq!(outcome, "focused");
        assert!(started.elapsed() < Duration::from_secs(5), "it waited");
        drop(first);
    }

    #[test]
    fn a_launch_finding_no_window_takes_over_once_the_holder_exits() {
        let name = unique_name();
        let first = acquire_named(&name, Duration::ZERO, || false);
        let second = contend(&name, Duration::from_secs(10), false);
        std::thread::sleep(Duration::from_millis(300));
        let released_at = Instant::now();
        drop(first);
        let (outcome, acquired_at) = second.join().unwrap();
        assert_eq!(outcome, "owned");
        assert!(acquired_at >= released_at, "took over before the release");
    }

    #[test]
    fn a_holder_that_never_exits_is_reported_never_bypassed() {
        let name = unique_name();
        let first = acquire_named(&name, Duration::ZERO, || false);
        let (outcome, _) = contend(&name, Duration::from_millis(200), false)
            .join()
            .unwrap();
        assert_eq!(outcome, "unresponsive");
        drop(first);
    }

    fn short_lived_child() -> std::process::Child {
        crate::services::system32::SystemTool::Cmd
            .command()
            .unwrap()
            .args(["/c", "ping -n 3 127.0.0.1 >nul"])
            .spawn()
            .unwrap()
    }

    #[test]
    fn the_predecessor_is_waited_for_by_its_process() {
        let mut child = short_lived_child();
        assert!(wait_for_exit(child.id(), Duration::from_secs(10), None));
        assert!(
            child.try_wait().unwrap().is_some(),
            "returned before the exit"
        );
        assert!(!wait_for_exit(
            std::process::id(),
            Duration::from_millis(50),
            None
        ));
    }

    /// A PID naming a process younger than this one was reused: no 30 s wait on a stranger.
    #[test]
    fn a_reused_pid_is_not_waited_for() {
        let mut child = short_lived_child();
        let ours = created(own_process());
        assert!(ours.is_some());
        assert!(wait_for_exit(child.id(), Duration::from_secs(10), ours));
        assert!(child.try_wait().unwrap().is_none(), "it waited");
        child.wait().unwrap();
    }

    #[test]
    fn a_launch_waits_out_a_mutex_its_token_is_denied() {
        use windows_sys::Win32::Security::{
            InitializeAcl, InitializeSecurityDescriptor, SetSecurityDescriptorDacl, ACL,
            ACL_REVISION, SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR,
        };
        const SECURITY_DESCRIPTOR_REVISION: u32 = 1;
        let name = unique_name();
        let wide = crate::services::wide(&name);
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

        let (outcome, _) = contend(&name, Duration::from_secs(10), true)
            .join()
            .unwrap();
        assert_eq!(outcome, "focused", "a window settles a denied launch too");

        let second = contend(&name, Duration::from_secs(10), false);
        std::thread::sleep(Duration::from_millis(300));
        let released_at = Instant::now();
        // SAFETY: `held` is this thread's mutex, released and closed exactly once.
        unsafe {
            ReleaseMutex(held);
            CloseHandle(held);
        }
        let (outcome, acquired_at) = second.join().unwrap();
        assert_eq!(outcome, "owned");
        assert!(acquired_at >= released_at, "took over before the release");
    }
}
