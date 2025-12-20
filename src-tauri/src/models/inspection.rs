use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMismatch {
    pub hive: String,
    pub key: String,
    pub value_name: String,
    pub expected_value: Option<serde_json::Value>,
    pub actual_value: Option<serde_json::Value>,
    pub value_type: Option<String>,
    pub description: String,
    pub is_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMismatch {
    pub name: String,
    pub expected_startup: String,
    pub actual_startup: Option<String>,
    pub description: String,
    pub is_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionInspection {
    pub option_index: usize,
    pub label: String,
    pub is_current: bool,
    pub is_pending: bool,
    pub registry_results: Vec<RegistryMismatch>,
    pub service_results: Vec<ServiceMismatch>,
    pub all_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakInspection {
    pub tweak_id: String,
    pub options: Vec<OptionInspection>,
    /// Index of the option that fully matches, if any
    pub matched_option_index: Option<usize>,
}
