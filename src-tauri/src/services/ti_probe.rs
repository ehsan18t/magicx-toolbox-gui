//! Whether this machine can reach TrustedInstaller at all, answered before the user clicks.
//!
//! `validate.rs` already refuses a corpus that would disable the TrustedInstaller service itself,
//! but that is a build-time check on our own YAML. It sees nothing done by a third-party hardening
//! baseline, a debloat script, or group policy, and CIS and STIG baselines routinely disable the
//! service outright.
//!
//! Without this probe such a machine shows a perfectly healthy card: detect never escalates, so
//! every read succeeds, and `compute_availability` only asked whether the app itself was elevated.
//! The user clicks, waits ten seconds for the service poll, and gets a rollback and a toast that
//! disappears. Asking the SCM up front turns that into a sentence on the card.
//!
//! Deliberately cheap and read-only: open the SCM, open the service for query, read its start type.
//! No start attempt, no `SeDebugPrivilege`, nothing that could itself fail for a second reason.

use std::ptr;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceConfigW, QUERY_SERVICE_CONFIGW,
    SC_MANAGER_CONNECT, SERVICE_DISABLED, SERVICE_QUERY_CONFIG,
};

const ERROR_SERVICE_DOES_NOT_EXIST: u32 = 1060;

/// Why an elevated apply cannot work on this machine, or `None` when it can.
///
/// The string is user-facing and names the machine's condition, not a Win32 code: these three
/// cases send a user to three different places, and "elevation failed" sends them nowhere.
pub type Unavailable = Option<String>;

/// Cached because it answers a question about machine configuration, and it is consulted once per
/// tweak per sweep across a 256-tweak corpus. Changing the TrustedInstaller service's start type
/// mid-session is a deliberate administrative act; a restart to pick it up is the right cost.
static PROBE: OnceLock<Unavailable> = OnceLock::new();

/// Whether TrustedInstaller can be reached, with the reason it cannot.
pub fn trusted_installer_blocked() -> &'static Unavailable {
    PROBE.get_or_init(probe)
}

fn probe() -> Unavailable {
    // SAFETY: plain SCM queries. Both handles are closed on every path, and the config buffer is
    // sized by the API's own first call before it is read.
    unsafe {
        let scm = OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            log::warn!(
                "could not open the SCM to probe TrustedInstaller: {}",
                GetLastError()
            );
            // Not knowing is not the same as knowing it is broken. Stay out of the way and let the
            // apply report the real error if there is one.
            return None;
        }

        let name: Vec<u16> = "TrustedInstaller\0".encode_utf16().collect();
        let service = OpenServiceW(scm, name.as_ptr(), SERVICE_QUERY_CONFIG);
        if service.is_null() {
            let err = GetLastError();
            CloseServiceHandle(scm);
            return if err == ERROR_SERVICE_DOES_NOT_EXIST {
                Some(
                    "The Windows Modules Installer (TrustedInstaller) service is not present on \
                     this PC, so tweaks that need it cannot run here."
                        .to_string(),
                )
            } else {
                log::warn!("could not open the TrustedInstaller service to probe it: {err}");
                None
            };
        }

        let mut needed: u32 = 0;
        // Documented to fail with the required size; only `needed` is meaningful here.
        QueryServiceConfigW(service, ptr::null_mut(), 0, &mut needed);
        if needed == 0 {
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            return None;
        }

        let mut buf = vec![0u8; needed as usize];
        let ok = QueryServiceConfigW(
            service,
            buf.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW,
            needed,
            &mut needed,
        );
        let start_type =
            (ok != 0).then(|| (*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW)).dwStartType);
        CloseServiceHandle(service);
        CloseServiceHandle(scm);

        match start_type {
            Some(SERVICE_DISABLED) => Some(
                "The Windows Modules Installer (TrustedInstaller) service is disabled on this PC, \
                 so tweaks that need it cannot run. Set its startup type to Manual to enable them."
                    .to_string(),
            ),
            _ => None,
        }
    }
}
