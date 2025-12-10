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
        RegistryHive::HKCU => "HKCU",
        RegistryHive::HKLM => "HKLM",
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
    let reg_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::RegistryKeyNotFound(key_path.to_string())
            } else {
                Error::RegistryAccessDenied(e.to_string())
            }
        })?;

    match reg_key.get_raw_value(value_name) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Error::RegistryOperation(e.to_string())),
    }
}

/// Get registry hive key
fn get_hive_key(hive: &RegistryHive) -> Result<HKEY, Error> {
    match hive {
        RegistryHive::HKCU => Ok(HKEY_CURRENT_USER),
        RegistryHive::HKLM => Ok(HKEY_LOCAL_MACHINE),
    }
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
    let hive_key = get_hive_key(hive)?;

    // For HKLM, we need write permissions which typically require admin
    if matches!(hive, RegistryHive::HKLM) {
        use crate::services::system_info_service::is_running_as_admin;
        if !is_running_as_admin() {
            log::warn!("HKLM write requires admin privileges");
            return Err(Error::RequiresAdmin);
        }
    }

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
    let hive_key = get_hive_key(hive)?;

    // For HKLM, we need write permissions which typically require admin
    if matches!(hive, RegistryHive::HKLM) {
        use crate::services::system_info_service::is_running_as_admin;
        if !is_running_as_admin() {
            log::warn!("HKLM write requires admin privileges");
            return Err(Error::RequiresAdmin);
        }
    }

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
    let hive_key = get_hive_key(hive)?;

    // For HKLM, we need write permissions which typically require admin
    if matches!(hive, RegistryHive::HKLM) {
        use crate::services::system_info_service::is_running_as_admin;
        if !is_running_as_admin() {
            log::warn!("HKLM write requires admin privileges");
            return Err(Error::RequiresAdmin);
        }
    }

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

/// Delete a registry value
pub fn delete_value(hive: &RegistryHive, key_path: &str, value_name: &str) -> Result<(), Error> {
    log::debug!(
        "Deleting value {}\\{}\\{}",
        hive_name(hive),
        key_path,
        value_name
    );
    let hive_key = get_hive_key(hive)?;

    // For HKLM, we need write permissions which typically require admin
    if matches!(hive, RegistryHive::HKLM) {
        use crate::services::system_info_service::is_running_as_admin;
        if !is_running_as_admin() {
            log::warn!("HKLM delete requires admin privileges");
            return Err(Error::RequiresAdmin);
        }
    }

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

/// Create a registry key
pub fn create_key(hive: &RegistryHive, key_path: &str) -> Result<(), Error> {
    log::debug!("Creating key {}\\{}", hive_name(hive), key_path);
    let hive_key = get_hive_key(hive)?;

    // For HKLM, we need write permissions which typically require admin
    if matches!(hive, RegistryHive::HKLM) {
        use crate::services::system_info_service::is_running_as_admin;
        if !is_running_as_admin() {
            log::warn!("HKLM create requires admin privileges");
            return Err(Error::RequiresAdmin);
        }
    }

    RegKey::predef(hive_key)
        .create_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| Error::RegistryAccessDenied(e.to_string()))?;

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
            &RegistryHive::HKCU,
            "Software\\Microsoft\\Windows\\CurrentVersion",
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
