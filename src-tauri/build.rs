//! Build script that compiles YAML tweak definitions into Rust code at compile time.
//!
//! This eliminates runtime YAML parsing and file I/O, making tweak loading instant.
//! When YAML files change, Cargo automatically rebuilds thanks to `rerun-if-changed`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

// ============================================================================
// Mirror types from models/tweak.rs for build-time parsing
// These must stay in sync with runtime types.
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryDefinition {
    id: String,
    name: String,
    description: String,
    icon: String,
    #[serde(default)]
    order: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum RegistryHive {
    #[serde(rename = "HKCU")]
    Hkcu,
    #[serde(rename = "HKLM")]
    Hklm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum RegistryValueType {
    #[serde(rename = "REG_DWORD")]
    Dword,
    #[serde(rename = "REG_QWORD")]
    Qword,
    #[serde(rename = "REG_SZ")]
    String,
    #[serde(rename = "REG_EXPAND_SZ")]
    ExpandString,
    #[serde(rename = "REG_MULTI_SZ")]
    MultiString,
    #[serde(rename = "REG_BINARY")]
    Binary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
enum ServiceStartupType {
    Disabled,
    Manual,
    Automatic,
    Boot,
    System,
}

/// Single registry modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryChange {
    hive: RegistryHive,
    key: String,
    value_name: String,
    value_type: RegistryValueType,
    value: serde_json::Value,
    #[serde(default)]
    windows_versions: Option<Vec<u32>>,
}

/// Single service modification within an option
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceChange {
    name: String,
    startup: ServiceStartupType,
    #[serde(default)]
    stop_service: bool,
    #[serde(default)]
    start_service: bool,
}

/// A single option within a tweak - contains all changes for that state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TweakOption {
    label: String,
    #[serde(default)]
    registry_changes: Vec<RegistryChange>,
    #[serde(default)]
    service_changes: Vec<ServiceChange>,
    #[serde(default)]
    pre_commands: Vec<String>,
    #[serde(default)]
    post_commands: Vec<String>,
}

/// Raw tweak definition as loaded from YAML
#[derive(Debug, Clone, Deserialize)]
struct TweakDefinitionRaw {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    info: Option<String>,
    risk_level: RiskLevel,
    #[serde(default)]
    requires_admin: bool,
    #[serde(default)]
    requires_system: bool,
    #[serde(default)]
    requires_reboot: bool,
    #[serde(default)]
    is_toggle: bool,
    options: Vec<TweakOption>,
}

/// Complete tweak definition with category assignment (for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TweakDefinition {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    info: Option<String>,
    risk_level: RiskLevel,
    #[serde(default)]
    requires_admin: bool,
    #[serde(default)]
    requires_system: bool,
    #[serde(default)]
    requires_reboot: bool,
    #[serde(default)]
    is_toggle: bool,
    options: Vec<TweakOption>,
    category_id: String,
}

/// YAML file structure with category and tweaks
#[derive(Debug, Clone, Deserialize)]
struct TweakFile {
    category: CategoryDefinition,
    tweaks: Vec<TweakDefinitionRaw>,
}

// ============================================================================
// Build script main
// ============================================================================

fn main() {
    // Standard Tauri build
    tauri_build::build();

    // Generate tweak data from YAML files
    if let Err(e) = generate_tweak_data() {
        panic!("Failed to generate tweak data: {}", e);
    }
}

fn generate_tweak_data() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let tweaks_dir = Path::new(&manifest_dir).join("tweaks");
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    // Tell Cargo to rerun if any YAML file changes
    println!("cargo:rerun-if-changed=tweaks/");
    for entry in fs::read_dir(&tweaks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    // Collect all categories and tweaks
    let mut categories: Vec<CategoryDefinition> = Vec::new();
    let mut tweaks: HashMap<String, TweakDefinition> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();

    for entry in fs::read_dir(&tweaks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path)?;

        let tweak_file: TweakFile = match serde_yml::from_str(&content) {
            Ok(tf) => tf,
            Err(e) => {
                errors.push(format!("Failed to parse {}: {}", file_name, e));
                continue;
            }
        };

        let category_id = tweak_file.category.id.clone();
        categories.push(tweak_file.category);

        for raw in tweak_file.tweaks {
            // Validate tweak structure
            if raw.options.is_empty() {
                errors.push(format!(
                    "Tweak '{}' in {} must have at least 1 option",
                    raw.id, file_name
                ));
                continue;
            }
            if raw.is_toggle && raw.options.len() != 2 {
                errors.push(format!(
                    "Toggle tweak '{}' in {} must have exactly 2 options, found {}",
                    raw.id,
                    file_name,
                    raw.options.len()
                ));
                continue;
            }

            let tweak = TweakDefinition {
                id: raw.id.clone(),
                name: raw.name,
                description: raw.description,
                info: raw.info,
                risk_level: raw.risk_level,
                requires_admin: raw.requires_admin,
                requires_system: raw.requires_system,
                requires_reboot: raw.requires_reboot,
                is_toggle: raw.is_toggle,
                options: raw.options,
                category_id: category_id.clone(),
            };
            tweaks.insert(raw.id, tweak);
        }
    }

    // Report any validation errors
    if !errors.is_empty() {
        for error in &errors {
            println!("cargo:warning={}", error);
        }
        return Err(format!("{} validation error(s) in YAML files", errors.len()).into());
    }

    // Sort categories by order
    categories.sort_by_key(|c| c.order);

    // Write JSON files separately (avoids escaping issues)
    let categories_json_path = out_path.join("categories.json");
    let tweaks_json_path = out_path.join("tweaks.json");

    fs::write(&categories_json_path, serde_json::to_string(&categories)?)?;
    fs::write(&tweaks_json_path, serde_json::to_string(&tweaks)?)?;

    // Generate Rust code that includes the JSON files
    let generated_code = format!(
        r#"// AUTO-GENERATED FILE - DO NOT EDIT
// Generated from YAML files in tweaks/ directory at build time.
// To modify tweaks, edit the YAML files and rebuild.

use std::collections::HashMap;
use std::sync::LazyLock;
use crate::models::{{CategoryDefinition, TweakDefinition}};

/// Raw JSON string of categories (embedded at compile time)
pub const CATEGORIES_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/categories.json"));

/// Raw JSON string of tweaks (embedded at compile time)
pub const TWEAKS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/tweaks.json"));

/// Pre-compiled categories loaded from YAML at build time.
/// Sorted by `order` field.
pub static CATEGORIES: LazyLock<Vec<CategoryDefinition>> = LazyLock::new(|| {{
    serde_json::from_str(CATEGORIES_JSON).expect("Failed to parse embedded categories JSON")
}});

/// Pre-compiled tweaks loaded from YAML at build time.
/// HashMap for O(1) lookup by tweak ID.
pub static TWEAKS: LazyLock<HashMap<String, TweakDefinition>> = LazyLock::new(|| {{
    serde_json::from_str(TWEAKS_JSON).expect("Failed to parse embedded tweaks JSON")
}});

/// Number of categories compiled into the binary
pub const CATEGORY_COUNT: usize = {category_count};

/// Number of tweaks compiled into the binary
pub const TWEAK_COUNT: usize = {tweak_count};
"#,
        category_count = categories.len(),
        tweak_count = tweaks.len(),
    );

    // Write the generated Rust file
    let rust_path = out_path.join("generated_tweaks.rs");
    let mut file = fs::File::create(&rust_path)?;
    file.write_all(generated_code.as_bytes())?;

    println!(
        "cargo:warning=Generated {} categories and {} tweaks from YAML files",
        categories.len(),
        tweaks.len()
    );

    Ok(())
}
