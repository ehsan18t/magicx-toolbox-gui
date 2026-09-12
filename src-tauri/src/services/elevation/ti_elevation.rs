//! TrustedInstaller elevation: start the TI service, then spawn the broker child with
//! TrustedInstaller.exe spoofed as its parent so it inherits the TI token.

use crate::error::Error;
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE};
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STATUS_PROCESS,
};
use windows_sys::Win32::System::SystemInformation::{
    GetSystemDirectoryW, GetSystemWindowsDirectoryW,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList, OpenProcess,
    QueryFullProcessImageNameW, UpdateProcThreadAttribute, CREATE_NO_WINDOW,
    CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_CREATE_PROCESS, PROCESS_INFORMATION, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW,
};

use super::common::{
    empty_process_info, enable_debug_privilege, hidden_startup_info, to_wide_string, wait_and_reap,
    SpawnError, ELEVATED_PROCESS_TIMEOUT_MS,
};

/// dwCurrentState values we distinguish while waiting for the service.
const SERVICE_STOPPED: u32 = 1;
const SERVICE_START_PENDING: u32 = 2;
const SERVICE_RUNNING: u32 = 4;

const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

/// Turn the Win32 codes this path actually produces into something a support engineer can act on.
/// Anything else keeps its bare number, which is still better than nothing.
fn describe_win32(code: u32) -> String {
    let name = match code {
        5 => "ERROR_ACCESS_DENIED",
        1058 => "ERROR_SERVICE_DISABLED",
        1060 => "ERROR_SERVICE_DOES_NOT_EXIST",
        1061 => "ERROR_SERVICE_CANNOT_ACCEPT_CTRL",
        1062 => "ERROR_SERVICE_NOT_ACTIVE",
        1069 => "ERROR_SERVICE_LOGON_FAILED",
        1072 => "ERROR_SERVICE_MARKED_FOR_DELETE",
        1314 => "ERROR_PRIVILEGE_NOT_HELD",
        _ => return code.to_string(),
    };
    format!("{code} ({name})")
}

/// A human-readable name for a service state code, for the timeout message.
fn describe_service_state(state: u32) -> &'static str {
    match state {
        SERVICE_STOPPED => "stopped",
        SERVICE_START_PENDING => "start-pending",
        3 => "stop-pending",
        SERVICE_RUNNING => "running",
        _ => "an unexpected state",
    }
}

/// Start the TrustedInstaller service and wait for it to be running, returning its pid.
fn start_trusted_installer_service() -> Result<u32, Error> {
    // SAFETY: Windows Service Control Manager API calls. All handles (SCM and service)
    // are closed on both success and error paths. Service status query uses properly
    // sized structures.
    unsafe {
        let scm = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open the service control manager: {}",
                describe_win32(GetLastError())
            )));
        }

        let service_name = to_wide_string("TrustedInstaller");
        let service = OpenServiceW(
            scm,
            service_name.as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        );

        if service.is_null() {
            // Read the error BEFORE closing: CloseServiceHandle overwrites the thread last-error,
            // so reading it afterwards reports the close, not the OpenServiceW that failed.
            let err = GetLastError();
            CloseServiceHandle(scm);
            return Err(Error::ServiceControl(format!(
                "Failed to open the TrustedInstaller service: {}",
                describe_win32(err)
            )));
        }

        let mut bytes_needed: u32 = 0;
        let mut status = std::mem::MaybeUninit::<SERVICE_STATUS_PROCESS>::zeroed();

        let query_result = QueryServiceStatusEx(
            service,
            SC_STATUS_PROCESS_INFO,
            status.as_mut_ptr() as *mut u8,
            std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
            &mut bytes_needed,
        );

        if query_result == 0 {
            let err = GetLastError();
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            return Err(Error::ServiceControl(format!(
                "Failed to query the TrustedInstaller service status: {}",
                describe_win32(err)
            )));
        }

        let status = status.assume_init();
        let current_state = status.dwCurrentState;

        if current_state == SERVICE_RUNNING {
            let pid = status.dwProcessId;
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            log::info!("The TrustedInstaller service is already running (pid {pid})");
            return Ok(pid);
        }

        log::info!(
            "Starting the TrustedInstaller service, currently {}",
            describe_service_state(current_state)
        );
        let start_result = StartServiceW(service, 0, ptr::null());

        if start_result == 0 {
            let err = GetLastError();
            if err != ERROR_SERVICE_ALREADY_RUNNING {
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(Error::ServiceControl(format!(
                    "Failed to start the TrustedInstaller service: {}",
                    describe_win32(err)
                )));
            }
        }

        // Poll up to 10s, querying first so a running service costs nothing. The last state and
        // query error leave the loop, so a timeout tells "stuck starting" from "never started".
        let mut last_state = current_state;
        let mut last_query_err = None;
        for attempt in 0..100 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let mut status = std::mem::MaybeUninit::<SERVICE_STATUS_PROCESS>::zeroed();
            let query_result = QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                status.as_mut_ptr() as *mut u8,
                std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut bytes_needed,
            );

            if query_result == 0 {
                last_query_err = Some(GetLastError());
                continue;
            }
            last_query_err = None;

            let status = status.assume_init();
            last_state = status.dwCurrentState;
            if last_state == SERVICE_RUNNING {
                let pid = status.dwProcessId;
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                log::info!("The TrustedInstaller service started (pid {pid})");
                return Ok(pid);
            }
        }

        CloseServiceHandle(service);
        CloseServiceHandle(scm);
        let detail = match last_query_err {
            Some(err) => format!(
                "last state {}, last status query failed: {}",
                describe_service_state(last_state),
                describe_win32(err)
            ),
            None => format!("it is still {}", describe_service_state(last_state)),
        };
        Err(Error::ServiceControl(format!(
            "The TrustedInstaller service did not start within 10s ({detail})"
        )))
    }
}

/// The image path of an open process handle, for identity verification.
///
/// # Safety
/// `handle` must be a valid process handle opened with `PROCESS_QUERY_LIMITED_INFORMATION`.
unsafe fn process_image_path(handle: HANDLE) -> Option<String> {
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len) == FALSE {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}

fn system_folder(
    query: unsafe extern "system" fn(*mut u16, u32) -> u32,
    name: &str,
) -> Result<Vec<u16>, Error> {
    let mut buf = [0u16; 1024];
    // SAFETY: `buf` is writable for the length passed.
    let len = unsafe { query(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    if len == 0 {
        // SAFETY: GetLastError only reads thread-local state.
        return Err(Error::WindowsApi(format!(
            "Failed to read the {name} folder: {}",
            describe_win32(unsafe { GetLastError() })
        )));
    }
    if len >= buf.len() {
        // Too small a buffer succeeds, returns the size needed, and sets no last-error.
        return Err(Error::WindowsApi(format!(
            "The {name} folder path needs a {len}-character buffer"
        )));
    }
    Ok(buf[..len].to_vec())
}

fn unprefixed(path: &str) -> &str {
    path.strip_prefix(r"\\?\")
        .or_else(|| path.strip_prefix(r"\??\"))
        .unwrap_or(path)
}

fn ti_image_path(windows_dir: &str) -> String {
    format!(
        r"{}\servicing\TrustedInstaller.exe",
        unprefixed(windows_dir).trim_end_matches('\\')
    )
}

fn is_ti_image(image: &str, ti_image: &str) -> bool {
    unprefixed(image).eq_ignore_ascii_case(ti_image)
}

/// Names a foreign image found at TrustedInstaller's pid by file name plus whether it sits under
/// the Windows folder. Its full path is a user profile directory as often as not, and this crosses
/// into the log and to the UI. `windows_dir` must already be unprefixed and unterminated.
fn foreign_image(image: &str, windows_dir: &str) -> String {
    let image = unprefixed(image);
    let name = image.rsplit('\\').next().unwrap_or(image);
    let under = image
        .get(..windows_dir.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(windows_dir))
        && image.as_bytes().get(windows_dir.len()) == Some(&b'\\');
    let where_it_sits = if under {
        "under the Windows folder"
    } else {
        "outside the Windows folder"
    };
    format!("{name} ({where_it_sits})")
}

/// Open TrustedInstaller with `PROCESS_CREATE_PROCESS` for the parent spoof; the caller owns the
/// handle. The SCM's pid can be recycled by now (the service stops when idle), so once the open
/// handle pins it, its image must match and the SCM must still report that pid running.
fn get_trusted_installer_handle() -> Result<HANDLE, Error> {
    enable_debug_privilege()?;
    let windows_dir =
        String::from_utf16_lossy(&system_folder(GetSystemWindowsDirectoryW, "Windows")?);
    let windows_dir = unprefixed(&windows_dir).trim_end_matches('\\');
    let ti_image = ti_image_path(windows_dir);
    // An idle-stop and restart between the start and the check moves the pid: retry once.
    match open_trusted_installer(&ti_image, windows_dir)? {
        Ok(handle) => Ok(handle),
        Err(mismatch) => {
            log::warn!("{mismatch}; retrying once");
            open_trusted_installer(&ti_image, windows_dir)?.map_err(Error::ServiceControl)
        }
    }
}

/// The TI service's (state, pid), by query only: never starts it.
fn ti_service_status() -> Result<(u32, u32), Error> {
    let name = to_wide_string("TrustedInstaller");
    // SAFETY: each handle opened here is closed before returning; the buffer fits the struct.
    unsafe {
        let scm = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        let service = if scm.is_null() {
            ptr::null_mut()
        } else {
            OpenServiceW(scm, name.as_ptr(), SERVICE_QUERY_STATUS)
        };
        let mut status = std::mem::MaybeUninit::<SERVICE_STATUS_PROCESS>::zeroed();
        let mut bytes_needed = 0;
        let queried = !service.is_null()
            && QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                status.as_mut_ptr() as *mut u8,
                std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut bytes_needed,
            ) != 0;
        let err = (!queried).then(|| GetLastError());
        if !service.is_null() {
            CloseServiceHandle(service);
        }
        if !scm.is_null() {
            CloseServiceHandle(scm);
        }
        match err {
            Some(code) => Err(Error::ServiceControl(format!(
                "TrustedInstaller verification failed: could not query the service: {}",
                describe_win32(code)
            ))),
            None => {
                let status = status.assume_init();
                Ok((status.dwCurrentState, status.dwProcessId))
            }
        }
    }
}

fn service_pid_mismatch(pinned: u32, state: u32, current: u32) -> Option<String> {
    let found = if state != SERVICE_RUNNING {
        describe_service_state(state).to_string()
    } else if current != pinned {
        format!("running as pid {current}")
    } else {
        return None;
    };
    Some(format!(
        "TrustedInstaller verification failed: pinned pid {pinned}, but the service is {found}"
    ))
}

/// One acquisition. The inner `Err` is a service-pid mismatch, which a TI restart can explain.
fn open_trusted_installer(
    ti_image: &str,
    windows_dir: &str,
) -> Result<Result<HANDLE, String>, Error> {
    let pid = start_trusted_installer_service()?;

    // SAFETY: `pid` may be stale: the open handle pins it, and its identity is verified through
    // that same handle before it is returned. Closed on every failure path, else the caller's.
    unsafe {
        let handle = OpenProcess(
            PROCESS_CREATE_PROCESS | PROCESS_QUERY_LIMITED_INFORMATION,
            FALSE,
            pid,
        );
        if handle.is_null() {
            return Err(Error::ServiceControl(format!(
                "Failed to open the TrustedInstaller process (pid {pid}): {}",
                describe_win32(GetLastError())
            )));
        }

        match process_image_path(handle) {
            Some(path) if is_ti_image(&path, ti_image) => {}
            Some(path) => {
                CloseHandle(handle);
                let image = foreign_image(&path, windows_dir);
                log::warn!("The pid the SCM reported for TrustedInstaller ({pid}) runs {image}");
                return Err(Error::ServiceControl(format!(
                    "pid {pid} runs {image}, not TrustedInstaller: the service stopped and its \
                     pid was reused, or it runs from a non-default path"
                )));
            }
            None => {
                let err = GetLastError();
                CloseHandle(handle);
                return Err(Error::ServiceControl(format!(
                    "Could not verify that pid {pid} is TrustedInstaller: {}",
                    describe_win32(err)
                )));
            }
        }

        // The pid is pinned, so the SCM still reporting it RUNNING makes this handle the service.
        match ti_service_status().map(|(state, current)| service_pid_mismatch(pid, state, current))
        {
            Ok(None) => Ok(Ok(handle)),
            Ok(Some(mismatch)) => {
                CloseHandle(handle);
                Ok(Err(mismatch))
            }
            Err(e) => {
                CloseHandle(handle);
                Err(e)
            }
        }
    }
}

/// Spawn `command_line` with TrustedInstaller.exe as its parent, so it inherits the TI token, and
/// wait for it. The broker's TI launcher; the command line is built by
/// `broker::run_elevated_broker`, never by a caller.
pub(super) fn spawn_as_trusted_installer(command_line: &str) -> Result<i32, SpawnError> {
    // Not the command line: it holds the request and response temp paths, and the response path
    // is guarded only by being unguessable; anyone who can read the log directory reads the log.
    log::info!("Spawning the broker as TrustedInstaller");

    let mut work_dir =
        system_folder(GetSystemDirectoryW, "System32").map_err(SpawnError::NoChild)?;
    work_dir.push(0);
    let ti_handle = get_trusted_installer_handle().map_err(SpawnError::NoChild)?;
    let mut command_wide = to_wide_string(command_line);

    // SAFETY: `ti_handle` closes on every path once `create` returns. The usize-aligned list
    // buffer is sized by the sizing call and outlives DeleteProcThreadAttributeList. `command_wide`
    // is owned, NUL-terminated and writable by CreateProcessW; wait_and_reap reaps `process_info`.
    unsafe {
        let mut create = || -> Result<PROCESS_INFORMATION, Error> {
            let mut attr_list_size: usize = 0;
            // First call sizes the buffer; it is documented to fail, so only the size is meaningful.
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut attr_list_size);
            if attr_list_size == 0 {
                return Err(Error::ServiceControl(format!(
                    "Failed to size the process attribute list: {}",
                    describe_win32(GetLastError())
                )));
            }

            let usize_count = attr_list_size.div_ceil(std::mem::size_of::<usize>());
            let mut attr_list_buffer: Vec<usize> = vec![0; usize_count];
            let attr_list = attr_list_buffer.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;

            if InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_list_size) == FALSE {
                return Err(Error::ServiceControl(format!(
                    "Failed to initialize the process attribute list: {}",
                    describe_win32(GetLastError())
                )));
            }

            // The list stores a pointer to this, so it must outlive every use of `attr_list`.
            let mut ti_handle_copy = ti_handle;
            let updated = UpdateProcThreadAttribute(
                attr_list,
                0,
                PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
                &mut ti_handle_copy as *mut _ as *mut _,
                std::mem::size_of::<HANDLE>(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if updated == FALSE {
                let err = GetLastError();
                DeleteProcThreadAttributeList(attr_list);
                return Err(Error::ServiceControl(format!(
                    "Failed to set the parent-process attribute: {}",
                    describe_win32(err)
                )));
            }

            let mut startup_info = STARTUPINFOEXW {
                StartupInfo: hidden_startup_info(),
                lpAttributeList: attr_list,
            };
            // With EXTENDED_STARTUPINFO_PRESENT, `cb` must span the EX struct, not just the inner
            // STARTUPINFOW, or CreateProcessW will not see the attribute list.
            startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
            let mut process_info = empty_process_info();

            // Null lpEnvironment: the child inherits this app's (the admin user's) environment.
            let created = CreateProcessW(
                ptr::null(),
                command_wide.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                FALSE,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
                ptr::null(),
                work_dir.as_ptr(),
                &startup_info.StartupInfo,
                &mut process_info,
            );
            let err = (created == FALSE).then(|| GetLastError());
            DeleteProcThreadAttributeList(attr_list);

            match err {
                Some(code) => Err(Error::ServiceControl(format!(
                    "Failed to create the broker process as TrustedInstaller: {}",
                    describe_win32(code)
                ))),
                None => Ok(process_info),
            }
        };

        let created = create();
        CloseHandle(ti_handle);
        let process_info = created.map_err(SpawnError::NoChild)?;
        wait_and_reap(
            &process_info,
            "TrustedInstaller command",
            ELEVATED_PROCESS_TIMEOUT_MS,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The codes a hardened or managed machine actually produces are the ones worth naming: a
    /// disabled service and a missing service send the user to completely different places, and
    /// "1058" alone sends them nowhere.
    #[test]
    fn the_service_errors_worth_acting_on_are_named() {
        assert_eq!(describe_win32(1058), "1058 (ERROR_SERVICE_DISABLED)");
        assert_eq!(describe_win32(1060), "1060 (ERROR_SERVICE_DOES_NOT_EXIST)");
        assert_eq!(describe_win32(5), "5 (ERROR_ACCESS_DENIED)");
        assert_eq!(describe_win32(1314), "1314 (ERROR_PRIVILEGE_NOT_HELD)");
    }

    /// An unmapped code still has to survive intact rather than being swallowed.
    #[test]
    fn an_unmapped_error_keeps_its_number() {
        assert_eq!(describe_win32(999_999), "999999");
    }

    #[test]
    fn the_service_pid_check_needs_the_pinned_pid_still_running() {
        assert_eq!(service_pid_mismatch(10, SERVICE_RUNNING, 10), None);
        let moved = service_pid_mismatch(10, SERVICE_RUNNING, 20).expect("a moved pid fails");
        assert!(moved.contains("running as pid 20"), "{moved}");
        let stopped =
            service_pid_mismatch(10, SERVICE_STOPPED, 0).expect("a stopped service fails");
        assert!(stopped.contains("stopped"), "{stopped}");
    }

    #[test]
    fn service_states_read_as_words() {
        assert_eq!(describe_service_state(SERVICE_STOPPED), "stopped");
        assert_eq!(
            describe_service_state(SERVICE_START_PENDING),
            "start-pending"
        );
        assert_eq!(describe_service_state(SERVICE_RUNNING), "running");
    }

    fn passes(image: &str, windows_dir: &str) -> bool {
        is_ti_image(image, &ti_image_path(windows_dir))
    }

    #[test]
    fn the_image_check_accepts_the_windows_folder_in_every_spelling() {
        let win = r"C:\Windows";
        assert!(passes(r"C:\Windows\servicing\TrustedInstaller.exe", win));
        assert!(passes(r"c:\windows\SERVICING\trustedinstaller.EXE", win));
        assert!(passes(
            r"\\?\C:\Windows\servicing\TrustedInstaller.exe",
            win
        ));
        assert!(passes(
            r"\??\C:\Windows\servicing\TrustedInstaller.exe",
            win
        ));
        assert!(passes(
            r"C:\Windows\servicing\TrustedInstaller.exe",
            r"C:\Windows\"
        ));
        assert!(passes(
            r"D:\WINNT\servicing\TrustedInstaller.exe",
            r"D:\WINNT"
        ));
    }

    /// A foreign image at TI's pid is diagnostic, but its path names a user. Only the file name and
    /// whether it sits under the Windows folder may be said.
    #[test]
    fn a_foreign_image_is_named_without_its_path() {
        let win = r"C:\Windows";
        for (image, expected) in [
            (
                r"C:\Users\Someone\AppData\Local\app.exe",
                "app.exe (outside the Windows folder)",
            ),
            (
                r"\\?\C:\Windows\System32\svchost.exe",
                "svchost.exe (under the Windows folder)",
            ),
            (
                r"C:\WindowsApps\thing.exe",
                "thing.exe (outside the Windows folder)",
            ),
            (
                r"c:\windows\explorer.exe",
                "explorer.exe (under the Windows folder)",
            ),
        ] {
            let got = foreign_image(image, win);
            assert_eq!(got, expected, "{image}");
            assert!(!got.contains('\\'), "{got} still carries a path");
        }
    }

    #[test]
    fn the_image_check_rejects_a_servicing_folder_outside_the_windows_folder() {
        for image in [
            r"C:\Temp\servicing\TrustedInstaller.exe",
            r"\\?\C:\Temp\servicing\TrustedInstaller.exe",
            r"\??\C:\Temp\servicing\TrustedInstaller.exe",
            r"C:\Windows\Temp\servicing\TrustedInstaller.exe",
            r"D:\Windows\servicing\TrustedInstaller.exe",
            r"C:\Windows\servicing\TrustedInstaller.exe.bak",
            r"C:\Windows\System32\svchost.exe",
        ] {
            assert!(!passes(image, r"C:\Windows"), "{image} passed");
        }
    }
}
