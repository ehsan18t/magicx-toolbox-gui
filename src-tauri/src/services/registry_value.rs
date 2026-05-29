use crate::error::Error;
use crate::models::{RegistryHive, RegistryValueType};
use crate::services::{registry_service, trusted_installer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryValue {
    Dword(u32),
    Qword(u64),
    String(String),
    ExpandString(String),
    MultiString(Vec<String>),
    Binary(Vec<u8>),
}

impl RegistryValue {
    fn to_json(&self) -> serde_json::Value {
        match self {
            RegistryValue::Dword(value) => serde_json::json!(value),
            RegistryValue::Qword(value) => serde_json::json!(value),
            RegistryValue::String(value) | RegistryValue::ExpandString(value) => {
                serde_json::json!(value)
            }
            RegistryValue::MultiString(value) => serde_json::json!(value),
            RegistryValue::Binary(value) => serde_json::json!(value),
        }
    }

    fn reg_exe_data(&self) -> String {
        match self {
            RegistryValue::Dword(value) => value.to_string(),
            RegistryValue::Qword(value) => value.to_string(),
            RegistryValue::String(value) | RegistryValue::ExpandString(value) => {
                format!("\"{}\"", value)
            }
            RegistryValue::MultiString(value) => value.join("\\0"),
            RegistryValue::Binary(value) => value
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>(),
        }
    }
}

pub fn parse_registry_value(
    value_type: &RegistryValueType,
    value: &serde_json::Value,
) -> Result<RegistryValue, Error> {
    match value_type {
        RegistryValueType::Dword => parse_u64(value, value_type).and_then(|parsed| {
            u32::try_from(parsed)
                .map(RegistryValue::Dword)
                .map_err(|_| {
                    Error::ValidationError(format!("REG_DWORD value {} exceeds u32 range", value))
                })
        }),
        RegistryValueType::Qword => parse_u64(value, value_type).map(RegistryValue::Qword),
        RegistryValueType::String => parse_string(value, value_type).map(RegistryValue::String),
        RegistryValueType::ExpandString => {
            parse_string(value, value_type).map(RegistryValue::ExpandString)
        }
        RegistryValueType::MultiString => parse_multi_string(value).map(RegistryValue::MultiString),
        RegistryValueType::Binary => parse_binary(value).map(RegistryValue::Binary),
    }
}

pub fn registry_values_match(
    value_type: &RegistryValueType,
    current_value: &Option<serde_json::Value>,
    expected_value: &Option<serde_json::Value>,
) -> Result<bool, Error> {
    match (current_value, expected_value) {
        (None, None) => Ok(true),
        (Some(current), Some(expected)) => {
            let normalized_current = parse_registry_value(value_type, current)?.to_json();
            let normalized_expected = parse_registry_value(value_type, expected)?.to_json();
            Ok(normalized_current == normalized_expected)
        }
        _ => Ok(false),
    }
}

pub fn write_registry_json_value(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
    value: &serde_json::Value,
    use_system: bool,
) -> Result<(), Error> {
    let parsed = parse_registry_value(value_type, value)?;

    if use_system {
        return write_registry_json_value_as_system(hive, key, value_name, value_type, &parsed);
    }

    match parsed {
        RegistryValue::Dword(value) => registry_service::set_dword(hive, key, value_name, value),
        RegistryValue::Qword(value) => registry_service::set_qword(hive, key, value_name, value),
        RegistryValue::String(value) => registry_service::set_string(hive, key, value_name, &value),
        RegistryValue::ExpandString(value) => {
            registry_service::set_expand_string(hive, key, value_name, &value)
        }
        RegistryValue::MultiString(value) => {
            registry_service::set_multi_string(hive, key, value_name, &value)
        }
        RegistryValue::Binary(value) => registry_service::set_binary(hive, key, value_name, &value),
    }
}

fn write_registry_json_value_as_system(
    hive: &RegistryHive,
    key: &str,
    value_name: &str,
    value_type: &RegistryValueType,
    value: &RegistryValue,
) -> Result<(), Error> {
    trusted_installer::set_registry_value_as_system(
        hive.as_str(),
        key,
        value_name,
        value_type.as_str(),
        &value.reg_exe_data(),
    )
}

fn parse_u64(value: &serde_json::Value, value_type: &RegistryValueType) -> Result<u64, Error> {
    value.as_u64().ok_or_else(|| {
        Error::ValidationError(format!(
            "Expected unsigned integer for {} registry value, got: {}",
            value_type.as_str(),
            value
        ))
    })
}

fn parse_string(
    value: &serde_json::Value,
    value_type: &RegistryValueType,
) -> Result<String, Error> {
    value.as_str().map(str::to_string).ok_or_else(|| {
        Error::ValidationError(format!(
            "Expected string for {} registry value, got: {}",
            value_type.as_str(),
            value
        ))
    })
}

fn parse_multi_string(value: &serde_json::Value) -> Result<Vec<String>, Error> {
    let arr = value.as_array().ok_or_else(|| {
        Error::ValidationError(format!(
            "Expected array of strings for REG_MULTI_SZ registry value, got: {}",
            value
        ))
    })?;

    arr.iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str().map(str::to_string).ok_or_else(|| {
                Error::ValidationError(format!(
                    "REG_MULTI_SZ item [{}] must be a string, got: {}",
                    index, item
                ))
            })
        })
        .collect()
}

fn parse_binary(value: &serde_json::Value) -> Result<Vec<u8>, Error> {
    if let Some(arr) = value.as_array() {
        return arr
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let parsed = item.as_u64().ok_or_else(|| {
                    Error::ValidationError(format!(
                        "REG_BINARY array item [{}] must be an integer (0-255), got: {}",
                        index, item
                    ))
                })?;

                u8::try_from(parsed).map_err(|_| {
                    Error::ValidationError(format!(
                        "REG_BINARY array item [{}] value {} exceeds byte range (0-255)",
                        index, parsed
                    ))
                })
            })
            .collect();
    }

    let hex = value.as_str().ok_or_else(|| {
        Error::ValidationError(format!(
            "Expected array of bytes or hex string for REG_BINARY registry value, got: {}",
            value
        ))
    })?;

    parse_binary_hex_string(hex)
}

fn parse_binary_hex_string(value: &str) -> Result<Vec<u8>, Error> {
    let tokens: Vec<String> = if value.contains(',') {
        value
            .split(',')
            .map(|token| token.trim().to_string())
            .collect()
    } else {
        let compact: String = value.chars().filter(|c| !c.is_ascii_whitespace()).collect();
        if !compact.len().is_multiple_of(2) {
            return Err(Error::ValidationError(format!(
                "REG_BINARY hex string must contain an even number of digits, got {}",
                compact.len()
            )));
        }
        compact
            .as_bytes()
            .chunks(2)
            .map(|chunk| String::from_utf8_lossy(chunk).to_string())
            .collect()
    };

    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let token = token
                .strip_prefix("0x")
                .or_else(|| token.strip_prefix("0X"))
                .unwrap_or(token);

            if token.is_empty() || token.len() > 2 {
                return Err(Error::ValidationError(format!(
                    "Invalid REG_BINARY byte [{}]: '{}'",
                    index, token
                )));
            }

            u8::from_str_radix(token, 16).map_err(|_| {
                Error::ValidationError(format!("Invalid REG_BINARY byte [{}]: '{}'", index, token))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_binary_from_comma_separated_hex_string() {
        let parsed = parse_registry_value(&RegistryValueType::Binary, &json!("00,A0,ff")).unwrap();

        assert_eq!(parsed, RegistryValue::Binary(vec![0, 160, 255]));
    }

    #[test]
    fn parses_binary_from_byte_array() {
        let parsed =
            parse_registry_value(&RegistryValueType::Binary, &json!([0, 160, 255])).unwrap();

        assert_eq!(parsed, RegistryValue::Binary(vec![0, 160, 255]));
    }

    #[test]
    fn rejects_binary_array_item_outside_byte_range() {
        let err = parse_registry_value(&RegistryValueType::Binary, &json!([256])).unwrap_err();

        assert!(err.to_string().contains("0-255"));
    }

    #[test]
    fn rejects_invalid_binary_hex_token() {
        let err = parse_registry_value(&RegistryValueType::Binary, &json!("00,GG")).unwrap_err();

        assert!(err.to_string().contains("Invalid REG_BINARY byte"));
    }

    #[test]
    fn matches_authored_binary_string_to_read_byte_array() {
        let matches = registry_values_match(
            &RegistryValueType::Binary,
            &Some(json!([0, 160, 255])),
            &Some(json!("00,A0,FF")),
        )
        .unwrap();

        assert!(matches);
    }
}
