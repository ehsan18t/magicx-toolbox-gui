use serde::{Deserialize, Serialize};

/// Represents a registry value that can be of different types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegistryValue {
    DWord(u32),
    String(String),
    Binary(Vec<u8>),
    MultiString(Vec<String>),
    QWord(u64),
}

impl RegistryValue {
    pub fn as_dword(&self) -> Option<u32> {
        match self {
            RegistryValue::DWord(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            RegistryValue::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            RegistryValue::Binary(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_multi_string(&self) -> Option<&[String]> {
        match self {
            RegistryValue::MultiString(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_qword(&self) -> Option<u64> {
        match self {
            RegistryValue::QWord(v) => Some(*v),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            RegistryValue::DWord(_) => "DWORD",
            RegistryValue::String(_) => "STRING",
            RegistryValue::Binary(_) => "BINARY",
            RegistryValue::MultiString(_) => "MULTI_STRING",
            RegistryValue::QWord(_) => "QWORD",
        }
    }
}

/// Information about a registry key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryKeyInfo {
    pub hive: String,
    pub key: String,
    pub subkey_count: u32,
    pub value_count: u32,
}

/// A registry key and its values (for backup/restore)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryKeySnapshot {
    pub hive: String,
    pub key: String,
    pub values: Vec<(String, RegistryValue)>,
}
