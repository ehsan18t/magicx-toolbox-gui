use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRegistryChange {
    pub hive: String,
    pub key: String,
    pub value_name: String,
    pub value_type: String,
    pub enable_value: serde_json::Value,
    #[serde(default)]
    pub disable_value: Option<serde_json::Value>,
    #[serde(default)]
    pub windows_versions: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct TestTweak {
    pub id: String,
    pub name: String,
    pub registry_changes: Vec<TestRegistryChange>,
}

fn main() {
    let yaml = r#"
id: test
name: "Test"
registry_changes:
  - hive: HKLM
    key: "System\\Test"
    value_name: "Start"
    value_type: "REG_DWORD"
    enable_value: 4
    disable_value: 2
  - hive: HKCU
    key: "Software\\Test"
    value_name: "Enabled"
    value_type: "REG_DWORD"
    enable_value: 0
    windows_versions: [10]
"#;

    match serde_yaml::from_str::<TestTweak>(yaml) {
        Ok(tweak) => {
            println!("✓ Parsed successfully!");
            println!("  ID: {}", tweak.id);
            println!("  Registry changes: {}", tweak.registry_changes.len());
            for change in &tweak.registry_changes {
                println!(
                    "    - {} versions: {:?}",
                    change.key, change.windows_versions
                );
            }
        }
        Err(e) => {
            println!("✗ Parse error: {}", e);
        }
    }
}
