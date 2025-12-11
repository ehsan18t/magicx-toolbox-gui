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
// These must stay in sync with runtime types, but that's intentional:
// if you change the model, you'll get a compile error here reminding you to update.
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[allow(clippy::upper_case_acronyms)]
enum RegistryHive {
    HKCU,
    HKLM,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
enum RegistryValueType {
    #[serde(rename = "REG_DWORD")]
    DWord,
    #[serde(rename = "REG_SZ")]
    String,
    #[serde(rename = "REG_EXPAND_SZ")]
    ExpandString,
    #[serde(rename = "REG_BINARY")]
    Binary,
    #[serde(rename = "REG_MULTI_SZ")]
    MultiString,
    #[serde(rename = "REG_QWORD")]
    QWord,
}

/// Option for multi-state tweaks (displayed as dropdown in UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TweakOption {
    label: String,
    value: serde_json::Value,
    #[serde(default)]
    is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryChange {
    hive: RegistryHive,
    key: String,
    value_name: String,
    value_type: RegistryValueType,
    enable_value: serde_json::Value,
    #[serde(default)]
    disable_value: Option<serde_json::Value>,
    #[serde(default)]
    windows_versions: Option<Vec<u32>>,
    /// Multi-state options (if present, displayed as dropdown instead of toggle)
    #[serde(default)]
    options: Option<Vec<TweakOption>>,
}

/// Windows service startup type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ServiceStartupType {
    Disabled,
    Manual,
    Automatic,
    Boot,
    System,
}

/// Single service change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceChange {
    name: String,
    enable_startup: ServiceStartupType,
    disable_startup: ServiceStartupType,
    #[serde(default)]
    stop_on_disable: bool,
    #[serde(default)]
    start_on_enable: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct TweakDefinitionRaw {
    id: String,
    name: String,
    description: String,
    risk_level: RiskLevel,
    #[serde(default)]
    requires_reboot: bool,
    #[serde(default)]
    requires_admin: bool,
    registry_changes: Vec<RegistryChange>,
    #[serde(default)]
    service_changes: Option<Vec<ServiceChange>>,
    #[serde(default)]
    info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TweakDefinition {
    id: String,
    name: String,
    description: String,
    category: String,
    risk_level: RiskLevel,
    #[serde(default)]
    requires_reboot: bool,
    #[serde(default)]
    requires_admin: bool,
    registry_changes: Vec<RegistryChange>,
    #[serde(default)]
    service_changes: Option<Vec<ServiceChange>>,
    #[serde(default)]
    info: Option<String>,
}

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

        let content = fs::read_to_string(&path)?;
        let tweak_file: TweakFile = serde_yml::from_str(&content).map_err(|e| {
            format!(
                "Failed to parse {}: {}",
                path.file_name().unwrap().to_string_lossy(),
                e
            )
        })?;

        let category_id = tweak_file.category.id.clone();
        categories.push(tweak_file.category);

        for raw in tweak_file.tweaks {
            let tweak = TweakDefinition {
                id: raw.id.clone(),
                name: raw.name,
                description: raw.description,
                category: category_id.clone(),
                risk_level: raw.risk_level,
                requires_reboot: raw.requires_reboot,
                requires_admin: raw.requires_admin,
                registry_changes: raw.registry_changes,
                service_changes: raw.service_changes,
                info: raw.info,
            };
            tweaks.insert(raw.id, tweak);
        }
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
