//! Power plan setting indexes, through powrprof.

use crate::error::Error;
use std::ptr;

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::System::Power::{
    PowerEnumerate, PowerGetActiveScheme, PowerReadACValueIndex, PowerReadDCValueIndex,
    PowerSetActiveScheme, PowerWriteACValueIndex, PowerWriteDCValueIndex, ACCESS_SCHEME,
};

const ERROR_NO_MORE_ITEMS: u32 = 259;

fn check(what: &str, code: u32) -> Result<(), Error> {
    match code {
        0 => Ok(()),
        code => Err(Error::win32(format!("{what} failed"), code)),
    }
}

pub fn active_scheme() -> Result<String, Error> {
    let mut scheme: *mut GUID = ptr::null_mut();
    // SAFETY: on success `scheme` points at a LocalAlloc'd GUID, copied out and freed once.
    unsafe {
        check(
            "PowerGetActiveScheme",
            PowerGetActiveScheme(ptr::null_mut(), &mut scheme),
        )?;
        let guid = *scheme;
        LocalFree(scheme.cast());
        Ok(super::guid_string(&guid))
    }
}

/// `scheme` in its canonical spelling.
pub fn scheme_exists(scheme: &str) -> Result<bool, Error> {
    let mut index = 0;
    loop {
        let mut guid = GUID::from_u128(0);
        let mut size = std::mem::size_of::<GUID>() as u32;
        // SAFETY: `guid` is a GUID-sized buffer and `size` says so.
        let code = unsafe {
            PowerEnumerate(
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ACCESS_SCHEME,
                index,
                (&mut guid as *mut GUID).cast(),
                &mut size,
            )
        };
        match code {
            ERROR_NO_MORE_ITEMS => return Ok(false),
            code => check("PowerEnumerate", code)?,
        }
        if super::guid_string(&guid) == scheme {
            return Ok(true);
        }
        index += 1;
    }
}

/// The (AC, DC) indexes of one setting in `scheme`.
pub fn read_index(scheme: &str, subgroup: &str, setting: &str) -> Result<(u32, u32), Error> {
    let (scheme, subgroup, setting) = (
        super::guid(scheme)?,
        super::guid(subgroup)?,
        super::guid(setting)?,
    );
    let (mut ac, mut dc) = (0u32, 0u32);
    // SAFETY: every pointer is to a live local.
    unsafe {
        check(
            "PowerReadACValueIndex",
            PowerReadACValueIndex(ptr::null_mut(), &scheme, &subgroup, &setting, &mut ac),
        )?;
        check(
            "PowerReadDCValueIndex",
            PowerReadDCValueIndex(ptr::null_mut(), &scheme, &subgroup, &setting, &mut dc),
        )?;
    }
    Ok((ac, dc))
}

pub fn write_index(
    scheme: &str,
    subgroup: &str,
    setting: &str,
    ac: u32,
    dc: u32,
) -> Result<(), Error> {
    let (scheme, subgroup, setting) = (
        super::guid(scheme)?,
        super::guid(subgroup)?,
        super::guid(setting)?,
    );
    // SAFETY: every pointer is to a live local.
    unsafe {
        check(
            "PowerWriteACValueIndex",
            PowerWriteACValueIndex(ptr::null_mut(), &scheme, &subgroup, &setting, ac),
        )?;
        check(
            "PowerWriteDCValueIndex",
            PowerWriteDCValueIndex(ptr::null_mut(), &scheme, &subgroup, &setting, dc),
        )
    }
}

/// A written index of the active plan takes effect only when the plan is activated again.
pub fn activate(scheme: &str) -> Result<(), Error> {
    let scheme = super::guid(scheme)?;
    // SAFETY: `scheme` is a live local.
    check("PowerSetActiveScheme", unsafe {
        PowerSetActiveScheme(ptr::null_mut(), &scheme)
    })
}
