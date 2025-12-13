use crate::error::Error;
use crate::models::RegistryHive;
use std::io;
use winreg::enums::*;
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

/// Read a DWORD value from registry
pub fn read_dword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<u32>, Error> {
    log::trace!(
        "Reading DWORD {}\\{}\\{}",
        hive_name(hive),
        key_path,
        value_name
    );
    let hive_key = get_hive_key(hive)?;
    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::RegistryKeyNotFound(format!("{}\\{}", key_path, value_name))
            } else {
                Error::RegistryAccessDenied(e.to_string())
            }
        })?;

    match reg_key.get_value::<u32, _>(value_name) {
        Ok(v) => {
            log::trace!("Read DWORD value: {}", v);
            Ok(Some(v))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            log::trace!("DWORD value not found");
            Ok(None)
        }
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to read DWORD from {}: {}",
            value_name, e
        ))),
    }
}

/// Read a String value from registry
pub fn read_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<String>, Error> {
    log::trace!(
        "Reading String {}\\{}\\{}",
        hive_name(hive),
        key_path,
        value_name
    );
    let hive_key = get_hive_key(hive)?;
    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::RegistryKeyNotFound(format!("{}\\{}", key_path, value_name))
            } else {
                Error::RegistryAccessDenied(e.to_string())
            }
        })?;

    match reg_key.get_value::<String, _>(value_name) {
        Ok(v) => {
            log::trace!("Read String value: {}", v);
            Ok(Some(v))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            log::trace!("String value not found");
            Ok(None)
        }
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to read String from {}: {}",
            value_name, e
        ))),
    }
}

/// Read binary data from registry
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
    let hive_key = get_hive_key(hive)?;
    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::RegistryKeyNotFound(format!("{}\\{}", key_path, value_name))
            } else {
                Error::RegistryAccessDenied(e.to_string())
            }
        })?;

    match reg_key.get_raw_value(value_name) {
        Ok(v) => {
            log::trace!("Read Binary value ({} bytes)", v.bytes.len());
            Ok(Some(v.bytes))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            log::trace!("Binary value not found");
            Ok(None)
        }
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to read Binary from {}: {}",
            value_name, e
        ))),
    }
}

/// Read a QWORD (u64) value from registry
pub fn read_qword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
) -> Result<Option<u64>, Error> {
    log::trace!(
        "Reading QWORD {}\\{}\\{}",
        hive_name(hive),
        key_path,
        value_name
    );
    let hive_key = get_hive_key(hive)?;
    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::RegistryKeyNotFound(format!("{}\\{}", key_path, value_name))
            } else {
                Error::RegistryAccessDenied(e.to_string())
            }
        })?;

    match reg_key.get_value::<u64, _>(value_name) {
        Ok(v) => {
            log::trace!("Read QWORD value: {}", v);
            Ok(Some(v))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            log::trace!("QWORD value not found");
            Ok(None)
        }
        Err(e) => Err(Error::RegistryOperation(format!(
            "Failed to read QWORD from {}: {}",
            value_name, e
        ))),
    }
}

/// Get registry hive key
fn get_hive_key(hive: &RegistryHive) -> Result<HKEY, Error> {
    match hive {
        RegistryHive::Hkcu => Ok(HKEY_CURRENT_USER),
        RegistryHive::Hklm => Ok(HKEY_LOCAL_MACHINE),
    }
}

/// Check if write access is allowed for the given hive
/// HKLM modifications require admin privileges
fn require_write_access(hive: &RegistryHive) -> Result<(), Error> {
    use crate::services::system_info_service::is_running_as_admin;
    if matches!(hive, RegistryHive::Hklm) && !is_running_as_admin() {
        log::warn!("HKLM modification requires admin privileges");
        return Err(Error::RequiresAdmin);
    }
    Ok(())
}

/// Set a DWORD value in registry
pub fn set_dword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: u32,
) -> Result<(), Error> {
    log::debug!(
        "Setting DWORD {}\\{}\\{} = {}",
        hive_name(hive),
        key_path,
        value_name,
        value
    );
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;

    let (reg_key, _) = RegKey::predef(hive_key)
        .create_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    reg_key.set_value(value_name, &value).map_err(|e| {
        Error::RegistryOperation(format!("Failed to set DWORD {}: {}", value_name, e))
    })?;

    log::trace!("DWORD value set successfully");
    Ok(())
}

/// Set a String value in registry
pub fn set_string(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &str,
) -> Result<(), Error> {
    log::debug!(
        "Setting String {}\\{}\\{} = {}",
        hive_name(hive),
        key_path,
        value_name,
        value
    );
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;

    let (reg_key, _) = RegKey::predef(hive_key)
        .create_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    reg_key.set_value(value_name, &value).map_err(|e| {
        Error::RegistryOperation(format!("Failed to set String {}: {}", value_name, e))
    })?;

    log::trace!("String value set successfully");
    Ok(())
}

/// Set binary data in registry
pub fn set_binary(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: &[u8],
) -> Result<(), Error> {
    log::debug!(
        "Setting Binary {}\\{}\\{} ({} bytes)",
        hive_name(hive),
        key_path,
        value_name,
        value.len()
    );
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;

    let (reg_key, _) = RegKey::predef(hive_key)
        .create_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    let reg_value = RegValue {
        vtype: REG_BINARY,
        bytes: value.to_vec(),
    };
    reg_key.set_raw_value(value_name, &reg_value).map_err(|e| {
        Error::RegistryOperation(format!("Failed to set Binary {}: {}", value_name, e))
    })?;

    log::trace!("Binary value set successfully");
    Ok(())
}

/// Set a QWORD (u64) value in registry
pub fn set_qword(
    hive: &RegistryHive,
    key_path: &str,
    value_name: &str,
    value: u64,
) -> Result<(), Error> {
    log::debug!(
        "Setting QWORD {}\\{}\\{} = {}",
        hive_name(hive),
        key_path,
        value_name,
        value
    );
    require_write_access(hive)?;
    let hive_key = get_hive_key(hive)?;

    let (reg_key, _) = RegKey::predef(hive_key)
        .create_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

    reg_key.set_value(value_name, &value).map_err(|e| {
        Error::RegistryOperation(format!("Failed to set QWORD {}: {}", value_name, e))
    })?;

    log::trace!("QWORD value set successfully");
    Ok(())
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

    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if a registry key exists (test utility)
    fn key_exists(hive: &RegistryHive, key_path: &str) -> Result<bool, Error> {
        let hive_key = get_hive_key(hive)?;
        match RegKey::predef(hive_key).open_subkey_with_flags(key_path, KEY_READ) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Error::RegistryAccessDenied(e.to_string())),
        }
    }

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
}
