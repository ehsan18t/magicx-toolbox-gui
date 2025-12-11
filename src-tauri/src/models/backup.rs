//! Backup Models
//!
//! Simple type definitions for backup operations.

/// A unique identifier for a registry key (hive + key path + value name)
pub type RegistryKeyId = String;

/// Creates a unique key identifier for a registry value
pub fn make_key_id(hive: &str, key: &str, value_name: &str) -> RegistryKeyId {
    format!("{}\\{}\\{}", hive, key, value_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_key_id() {
        let key_id = make_key_id("HKLM", "SOFTWARE\\Microsoft\\Test", "ValueName");
        assert_eq!(key_id, "HKLM\\SOFTWARE\\Microsoft\\Test\\ValueName");
    }
}
