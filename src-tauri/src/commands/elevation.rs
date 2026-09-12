//! Elevation Commands
//!
//! Restarting the app with administrator privileges.

use crate::error::{Error, Result};
use crate::tweaks::engine::lifecycle::{self, ApplyGate};

#[tauri::command]
pub async fn restart_as_admin(app: tauri::AppHandle) -> Result<()> {
    log::info!("Restart as admin requested");
    // ShellExecuteW "runas" blocks until the UAC prompt is answered, so it runs on the blocking pool.
    tauri::async_runtime::spawn_blocking(move || {
        restart_as_admin_in(lifecycle::gate(), launch_elevated, || app.exit(0))
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

/// Uses ShellExecuteW with the "runas" verb, which shows the UAC prompt.
fn launch_elevated() -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe_path = std::env::current_exe()
        .map_err(|e| Error::WindowsApi(format!("Failed to get executable path: {}", e)))?;

    let exe_path_wide: Vec<u16> = OsStr::new(&exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let runas: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: ShellExecuteW is called with valid null-terminated wide strings.
    // The operation is "runas" which triggers UAC elevation. Return value > 32
    // indicates success.
    unsafe {
        let result = ShellExecuteW(
            ptr::null_mut(),
            runas.as_ptr(),
            exe_path_wide.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        );

        // ShellExecuteW returns a value > 32 on success
        if result as usize <= 32 {
            return Err(Error::WindowsApi(format!(
                "Failed to restart as admin, error code: {}",
                result as usize
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::lifecycle::AppExiting;

    #[tokio::test]
    async fn failed_launch_releases_the_latch() {
        let gate = ApplyGate::default();
        let result = restart_as_admin_in(
            &gate,
            || Err(Error::WindowsApi("declined".into())),
            || panic!("must not exit"),
        );
        assert!(
            matches!(result, Err(Error::WindowsApi(_))),
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
