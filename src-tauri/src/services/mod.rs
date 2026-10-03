pub mod appx_index;
pub mod elevation;
pub mod exclusive_temp;
pub mod firewall_service;
pub mod hosts_service;
pub mod registry_service;
pub mod registry_value;
pub mod scheduler_service;
pub mod service_control;
pub mod single_instance;
pub mod system32;
pub mod system_info_service;
pub mod ti_probe;

/// NUL-terminated UTF-16 for a Win32 `PCWSTR`, straight from the `OsStr` (never lossily).
pub fn wide(s: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

/// `FOLDERID_LocalAppData`, as Tauri resolves it: the environment variable can be overridden.
pub fn local_app_data() -> Result<std::path::PathBuf, String> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{FOLDERID_LocalAppData, SHGetKnownFolderPath};
    let mut path = std::ptr::null_mut();
    // SAFETY: `path` receives a CoTaskMem string, read up to its NUL and freed exactly once, even
    // on failure (where it is null, which CoTaskMemFree accepts).
    unsafe {
        let hr = SHGetKnownFolderPath(&FOLDERID_LocalAppData, 0, std::ptr::null_mut(), &mut path);
        let folder = (hr >= 0).then(|| {
            let len = (0..).take_while(|&i| *path.add(i) != 0).count();
            std::ffi::OsString::from_wide(std::slice::from_raw_parts(path, len)).into()
        });
        CoTaskMemFree(path.cast());
        folder.ok_or_else(|| format!("HRESULT {hr:#010x}"))
    }
}
