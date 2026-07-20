use crate::error::Error;
use crate::models::RegistryHive;
use std::io;
use winreg::enums::*;
use winreg::types::{FromRegValue, ToRegValue};
use winreg::RegKey;
use winreg::RegValue;
use winreg::HKEY;

/// Format hive name for display
fn hive_name(hive: &RegistryHive) -> &'static str {
    match hive {
        RegistryHive::Hkcu => "HKCU",
        RegistryHive::Hklm => "HKLM",
    }
}

/// Get registry hive key
fn get_hive_key(hive: &RegistryHive) -> Result<HKEY, Error> {
    match hive {
        RegistryHive::Hkcu => Ok(HKEY_CURRENT_USER),
        RegistryHive::Hklm => Ok(HKEY_LOCAL_MACHINE),
    }
}

/// Classify a subkey-open failure: a *missing key* is `RegistryKeyNotFound`, anything else is
/// `RegistryAccessDenied`.
///
/// This is the single source of truth for that distinction. Every key-open path routes through it,
/// so a NotFound can never again be silently folded into AccessDenied — the bug that made
/// `delete_value` reject deletes of an already-absent key. The apply/revert/broker "did-it-work"
/// idempotency shims treat `RegistryKeyNotFound` as "already gone → success", so mislabelling it as
/// AccessDenied turned a no-op delete into a hard failure.
fn classify_open_error(e: &io::Error, not_found_ctx: &str) -> Error {
    if e.kind() == io::ErrorKind::NotFound {
        Error::RegistryKeyNotFound(not_found_ctx.to_string())
    } else {
        Error::RegistryAccessDenied(e.to_string())
    }
}

/// Open a subkey for reading, classifying a missing key via [`classify_open_error`].
fn open_read_key(hive: &RegistryHive, key_path: &str, value_name: &str) -> Result<RegKey, Error> {
    let hive_key = get_hive_key(hive)?;
    RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| classify_open_error(&e, &format!("{}\\{}", key_path, value_name)))
}

/// Read a typed value. An absent *value* maps to `None`; an absent *key* is an error (via the open).
fn read_typed<T: FromRegValue>(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    type_label: &str,
) -> Result<Option<T>, Error> {
    log::trace!(
        "Reading {} {}\\{}\\{}",
        type_label,
        hive_name(hive),
        key_path,
        value_name
    );
    let reg_key = open_read_key(hive, key_path, value_name)?;
    match reg_key.get_value::<T, _>(value_name) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to read {} from {}: {}",
            type_label, value_name, e
        ))),
    }
}

/// Read a DWORD value from registry
pub fn read_dword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<u32>, Error> {
    read_typed(hive, key_path, value_name, "DWORD")
}

/// Read a String value from registry
pub fn read_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<String>, Error> {
    read_typed(hive, key_path, value_name, "String")
}

/// Read a multi-string value from registry
pub fn read_multi_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<Vec<String>>, Error> {
    read_typed(hive, key_path, value_name, "MultiString")
}

/// Read a QWORD (u64) value from registry
pub fn read_qword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<u64>, Error> {
    read_typed(hive, key_path, value_name, "QWORD")
}

/// Read binary data from registry (raw bytes, regardless of the stored value type)
pub fn read_binary(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<Vec<u8>>, Error> {
    log::trace!(
        "Reading Binary {}\\{}\\{}",
        hive_name(hive),
        key_path,
        value_name
    );
    let reg_key = open_read_key(hive, key_path, value_name)?;
    match reg_key.get_raw_value(value_name) {
        Ok(v) => Ok(Some(v.bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to read Binary from {}: {}",
            value_name, e
        ))),
    }
}

/// Check if write access is allowed for the given hive.
/// HKLM modifications require admin privileges.
fn require_write_access(hive: &RegistryHive) -> Result<(), Error> {
    use crate::services::system_info_service::is_running_as_admin;
    if matches!(hive, RegistryHive::Hklm) && !is_running_as_admin() {
        log::warn!("HKLM modification requires admin privileges");
        return Err(Error::RequiresAdmin);
    }
    Ok(())
}

/// Enforce write access, then create-or-open the target subkey for writing.
///
/// Every setter and `create_key` shares this prologue, so admin-gating and the create-subkey open
/// live in exactly one place.
fn open_write_key(hive: &RegistryHive, key_path: &str) -> Result<RegKey, Error> {
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;
    let (reg_key, _) = RegKey::predef(hive_key)
        .create_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;
    Ok(reg_key)
}

/// Set a value winreg encodes natively via `set_value` (DWORD / QWORD / String / MultiString).
fn set_typed<T: ToRegValue>(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &T,
    type_label: &str,
) -> Result<(), Error> {
    log::debug!(
        "Setting {} {}\\{}\\{}",
        type_label,
        hive_name(hive),
        key_path,
        value_name
    );
    let reg_key = open_write_key(hive, key_path)?;
    reg_key.set_value(value_name, value).map_err(|e| {
        Error::RegistryOperation(format!("Failed to set {} {}: {}", type_label, value_name, e))
    })?;
    log::trace!("{} value set successfully", type_label);
    Ok(())
}

/// Set a DWORD value in registry
pub fn set_dword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: u32,
) -> Result<(), Error> {
    set_typed(hive, key_path, value_name, &value, "DWORD")
}

/// Set a String value in registry
pub fn set_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &str,
) -> Result<(), Error> {
    set_typed(hive, key_path, value_name, &value, "String")
}

/// Set a multi-string value in registry
pub fn set_multi_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &[String],
) -> Result<(), Error> {
    set_typed(hive, key_path, value_name, &value.to_vec(), "MultiString")
}

/// Set a QWORD (u64) value in registry
pub fn set_qword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: u64,
) -> Result<(), Error> {
    set_typed(hive, key_path, value_name, &value, "QWORD")
}

/// Set a value with an explicit (non-native) `vtype` via `set_raw_value`.
fn set_raw(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    vtype: winreg::enums::RegType,
    bytes: Vec<u8>,
    type_label: &str,
) -> Result<(), Error> {
    log::debug!(
        "Setting {} {}\\{}\\{}",
        type_label,
        hive_name(hive),
        key_path,
        value_name
    );
    let reg_key = open_write_key(hive, key_path)?;
    let reg_value = RegValue { vtype, bytes };
    reg_key.set_raw_value(value_name, &reg_value).map_err(|e| {
        Error::RegistryOperation(format!("Failed to set {} {}: {}", type_label, value_name, e))
    })?;
    log::trace!("{} value set successfully", type_label);
    Ok(())
}

/// Set an expandable string value in registry
pub fn set_expand_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &str,
) -> Result<(), Error> {
    set_raw(
        hive,
        key_path,
        value_name,
        REG_EXPAND_SZ,
        encode_utf16_registry_string(value),
        "ExpandString",
    )
}

/// Set binary data in registry
pub fn set_binary(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &[u8],
) -> Result<(), Error> {
    set_raw(
        hive,
        key_path,
        value_name,
        REG_BINARY,
        value.to_vec(),
        "Binary",
    )
}

fn encode_utf16_registry_string(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// Delete a registry value
pub fn delete_value(hive: &RegistryHive, key_path: &str, value_name: &str) -> Result<(), Error> {
    log::debug!(
        "Deleting value {}\\{}\\{}",
        hive_name(hive),
        key_path,
        value_name
    );
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;

    // A missing key here must surface as RegistryKeyNotFound (not AccessDenied): the caller's
    // idempotency shim treats "already absent" as success, so this is how a no-op delete stays a
    // no-op. See [`classify_open_error`].
    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| classify_open_error(&e, &format!("{}\\{}", key_path, value_name)))?;

    reg_key.delete_value(value_name).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            Error::RegistryKeyNotFound(format!("{}\\{}", key_path, value_name))
        } else {
            Error::RegistryOperation(format!("Failed to delete {}: {}", value_name, e))
        }
    })?;

    log::trace!("Value deleted successfully");
    Ok(())
}

/// Delete a registry key and all its subkeys recursively
pub fn delete_key(hive: &RegistryHive, key_path: &str) -> Result<(), Error> {
    log::debug!("Deleting key {}\\{}", hive_name(hive), key_path);
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;

    // Need to open parent key and delete the child
    // Split path into parent and child
    let (parent_path, child_name) = match key_path.rsplit_once('\\') {
        Some((parent, child)) => (parent, child),
        None => {
            // No parent - trying to delete a top-level key (not allowed)
            return Err(Error::RegistryOperation(
                "Cannot delete top-level registry key".into(),
            ));
        }
    };

    let parent_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(parent_path, KEY_WRITE)
        .map_err(|e| classify_open_error(&e, &format!("Parent key not found: {}", parent_path)))?;

    // delete_subkey_all deletes the key and all subkeys recursively
    parent_key.delete_subkey_all(child_name).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            Error::RegistryKeyNotFound(key_path.to_string())
        } else {
            Error::RegistryOperation(format!("Failed to delete key {}: {}", key_path, e))
        }
    })?;

    log::trace!("Key deleted successfully");
    Ok(())
}

/// Check if a registry key exists
pub fn key_exists(hive: &RegistryHive, key_path: &str) -> Result<bool, Error> {
    let hive_key = get_hive_key(hive)?;
    match RegKey::predef(hive_key).open_subkey_with_flags(key_path, KEY_READ) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Error::RegistryAccessDenied(e.to_string())),
    }
}

/// Check if a registry value exists
pub fn value_exists(hive: &RegistryHive, key_path: &str, value_name: &str) -> Result<bool, Error> {
    let hive_key = get_hive_key(hive)?;
    let reg_key = match RegKey::predef(hive_key).open_subkey_with_flags(key_path, KEY_READ) {
        Ok(k) => k,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(Error::RegistryAccessDenied(e.to_string())),
    };

    // Try to get any value - if it exists, return true
    match reg_key.get_raw_value(value_name) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to check value {}: {}",
            value_name, e
        ))),
    }
}

/// Create a registry key without setting any value
pub fn create_key(hive: &RegistryHive, key_path: &str) -> Result<(), Error> {
    log::debug!("Creating key {}\\{}", hive_name(hive), key_path);
    // create_subkey creates the key if it doesn't exist, or opens it if it does
    open_write_key(hive, key_path)?;
    log::trace!("Key created successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exists_hkcu() {
        // Test with known HKCU key
        let result = key_exists(
            &RegistryHive::Hkcu,
            "Software\\Microsoft\\Windows\\CurrentVersion",
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn deleting_a_value_under_a_missing_key_reports_not_found_not_access_denied() {
        // Regression for the did-it-work idempotency contract: apply/revert/broker treat a
        // RegistryKeyNotFound on delete as "already absent → success". Before the fix, delete_value
        // folded the missing-key open error into RegistryAccessDenied, so those shims saw a hard
        // failure and aborted a no-op delete. HKCU needs no admin, so this runs everywhere.
        let err = delete_value(
            &RegistryHive::Hkcu,
            "Software\\MagicxToolboxTests\\wp1_definitely_absent_key",
            "AnyValue",
        )
        .expect_err("deleting a value under a nonexistent key must be an error");
        assert!(
            matches!(err, Error::RegistryKeyNotFound(_)),
            "expected RegistryKeyNotFound, got {err:?}"
        );
    }
}
