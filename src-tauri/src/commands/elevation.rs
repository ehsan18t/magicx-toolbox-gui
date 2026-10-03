//! Restarting the app with administrator privileges.

use crate::error::{Error, Result};
use crate::tweaks::engine::lifecycle::{self, ApplyGate};
use tauri::Manager;

#[tauri::command]
pub async fn restart_as_admin(app: tauri::AppHandle) -> Result<()> {
    log::info!("Restart as admin requested");
    let owner = match app.get_webview_window("main").map(|w| w.hwnd()) {
        Some(Ok(hwnd)) => hwnd.0 as isize,
        Some(Err(e)) => {
            log::warn!("no window to own the UAC prompt: {e}");
            0
        }
        None => 0,
    };
    // "runas" blocks until the UAC prompt is answered, so it runs on the blocking pool.
    tauri::async_runtime::spawn_blocking(move || {
        let launch = || {
            crate::window_state::save(&app);
            launch_elevated(owner)
        };
        restart_as_admin_in(lifecycle::gate(), launch, || app.exit(0))
    })
    .await?
}

// Latched before the UAC prompt, not re-checked after it: by then the elevated instance is
// running, and refusing would leave two. An apply started during the prompt is refused instead.
fn restart_as_admin_in(
    gate: &ApplyGate,
    launch: impl FnOnce() -> Result<()>,
    exit: impl FnOnce(),
) -> Result<()> {
    let latch = gate
        .begin_exit()
        .map_err(|refused| Error::exit_refused(refused, "restart as administrator"))?;
    launch()?;
    log::info!("New admin instance started, exiting current instance");
    // `exit` only queues the exit, so the latch must outlive this command.
    latch.keep_until_exit();
    exit();
    Ok(())
}

/// ShellExecuteExW "runas" raises the UAC prompt; owned by the window, it opens in front of it.
fn launch_elevated(owner: isize) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide = |s: &OsStr| -> Vec<u16> { s.encode_wide().chain(std::iter::once(0)).collect() };
    let exe = std::env::current_exe()
        .map_err(|e| Error::from_io("Could not find the app's executable", &e))?;
    // The elevated instance may run under another account, whose own settings file would apply.
    let [persist, detailed] = crate::logging::restart_args();
    let params = format!(
        "{} {persist} {detailed}",
        crate::services::single_instance::after_restart_arg()
    );
    let (verb, file, params) = (
        wide(OsStr::new("runas")),
        wide(exe.as_os_str()),
        wide(OsStr::new(&params)),
    );
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC,
        hwnd: owner as _,
        lpVerb: verb.as_ptr(),
        lpFile: file.as_ptr(),
        lpParameters: params.as_ptr(),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    // SAFETY: every string is NUL-terminated and outlives the call; `info` is sized as declared.
    if unsafe { ShellExecuteExW(&mut info) } == 0 {
        // SAFETY: reads this thread's last error, set by the failed call above.
        return Err(launch_error(unsafe { GetLastError() }));
    }
    Ok(())
}

fn launch_error(code: u32) -> Error {
    use windows_sys::Win32::Foundation::ERROR_CANCELLED;
    if code == ERROR_CANCELLED {
        Error::ElevationDeclined
    } else {
        Error::win32("Could not restart as administrator", code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::lifecycle::AppExiting;

    #[test]
    fn a_declined_uac_prompt_is_its_own_error() {
        assert!(matches!(launch_error(1223), Error::ElevationDeclined));
        assert!(matches!(launch_error(5), Error::Win32 { code: 5, .. }));
    }

    #[tokio::test]
    async fn failed_launch_releases_the_latch() {
        let gate = ApplyGate::default();
        let result = restart_as_admin_in(
            &gate,
            || Err(Error::ElevationDeclined),
            || panic!("must not exit"),
        );
        assert!(
            matches!(result, Err(Error::ElevationDeclined)),
            "got {result:?}"
        );
        assert!(gate.lock_tweak("t").await.is_ok());
    }

    #[tokio::test]
    async fn launched_restart_keeps_the_latch_and_exits() {
        let gate = ApplyGate::default();
        let mut exited = false;
        restart_as_admin_in(&gate, || Ok(()), || exited = true).expect("launch succeeds");
        assert!(exited);
        assert_eq!(gate.lock_tweak("t").await.err(), Some(AppExiting::Final));
    }

    #[tokio::test]
    async fn refused_under_an_apply_without_prompting() {
        let gate = ApplyGate::default();
        let _guard = gate.lock_tweak("t").await.expect("no exit pending");
        let result = restart_as_admin_in(
            &gate,
            || panic!("must not prompt"),
            || panic!("must not exit"),
        );
        assert!(
            matches!(result, Err(Error::ApplyInFlight(_))),
            "got {result:?}"
        );
    }
}
