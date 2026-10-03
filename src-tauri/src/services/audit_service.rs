//! Advanced audit policy subcategories, through advapi32's system audit policy API.

use crate::error::Error;
use crate::services::privilege;
use std::ptr;
use std::sync::{Mutex, PoisonError};

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Security::Authentication::Identity::{
    AuditFree, AuditQuerySystemPolicy, AuditSetSystemPolicy, AUDIT_POLICY_INFORMATION,
};

const SECURITY_PRIVILEGE: &str = "SeSecurityPrivilege";

/// Serializes enable and restore of the process-wide privilege, so one caller's restore cannot
/// disable it under another's call.
static PRIVILEGE: Mutex<()> = Mutex::new(());

/// Both audit calls need `SeSecurityPrivilege` enabled: an administrator holds it disabled.
fn with_security_privilege<T>(call: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    let _held = PRIVILEGE.lock().unwrap_or_else(PoisonError::into_inner);
    let enabled_here = privilege::adjust(SECURITY_PRIVILEGE, true)?;
    let result = call();
    if enabled_here {
        if let Err(e) = privilege::adjust(SECURITY_PRIVILEGE, false) {
            log::warn!("could not disable {SECURITY_PRIVILEGE} again: {e}");
        }
    }
    result
}

/// The subcategory's raw `AuditingInformation` flags.
pub fn query(subcategory: &str) -> Result<u32, Error> {
    let guid = super::guid(subcategory)?;
    with_security_privilege(|| {
        let mut policy: *mut AUDIT_POLICY_INFORMATION = ptr::null_mut();
        // SAFETY: on success `policy` points at one entry, read and then freed once by AuditFree.
        unsafe {
            if !AuditQuerySystemPolicy(&guid, 1, &mut policy) {
                return Err(Error::win32(
                    "AuditQuerySystemPolicy failed",
                    GetLastError(),
                ));
            }
            let flags = (*policy).AuditingInformation;
            AuditFree(policy.cast());
            Ok(flags)
        }
    })
}

/// Sets the subcategory's raw `AuditingInformation` flags.
pub fn set(subcategory: &str, flags: u32) -> Result<(), Error> {
    let entry = AUDIT_POLICY_INFORMATION {
        AuditSubCategoryGuid: super::guid(subcategory)?,
        AuditingInformation: flags,
        ..Default::default()
    };
    with_security_privilege(|| {
        // SAFETY: `entry` is one fully initialized local.
        unsafe {
            if AuditSetSystemPolicy(&entry, 1) {
                Ok(())
            } else {
                Err(Error::win32("AuditSetSystemPolicy failed", GetLastError()))
            }
        }
    })
}
