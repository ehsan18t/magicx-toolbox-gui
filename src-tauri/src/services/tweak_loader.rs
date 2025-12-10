use crate::error::Error;
use crate::models::TweakDefinition;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Load all tweaks from YAML files
pub fn load_all_tweaks() -> Result<HashMap<String, TweakDefinition>, Error> {
    let mut tweaks = HashMap::new();

    // Try to load from app resources directory
    if let Ok(tweaks_data) = load_tweaks_from_resources() {
        tweaks.extend(tweaks_data);
    }

    Ok(tweaks)
}

/// Load tweaks from a specific YAML file
pub fn load_tweaks_from_file(file_path: &str) -> Result<Vec<TweakDefinition>, Error> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read tweak file: {}", e)))?;

    let tweaks: Vec<TweakDefinition> = serde_yaml::from_str(&content)
        .map_err(|e| Error::WindowsApi(format!("Failed to parse YAML: {}", e)))?;

    Ok(tweaks)
}

/// Load tweaks from embedded resources or file system
fn load_tweaks_from_resources() -> Result<HashMap<String, TweakDefinition>, Error> {
    let mut tweaks = HashMap::new();

    // Try to find tweaks directory in app directory
    let exe_path = std::env::current_exe()
        .map_err(|e| Error::WindowsApi(format!("Failed to get exe path: {}", e)))?;

    let tweaks_dir = exe_path
        .parent()
        .ok_or_else(|| Error::WindowsApi("Could not determine app directory".to_string()))?
        .join("tweaks");

    if tweaks_dir.exists() {
        for category_file in [
            "privacy.yaml",
            "performance.yaml",
            "ui.yaml",
            "security.yaml",
            "services.yaml",
            "gaming.yaml",
        ] {
            let file_path = tweaks_dir.join(category_file);
            if file_path.exists() {
                if let Ok(category_tweaks) = load_tweaks_from_file(file_path.to_str().unwrap()) {
                    for tweak in category_tweaks {
                        tweaks.insert(tweak.id.clone(), tweak);
                    }
                }
            }
        }
    }

    Ok(tweaks)
}

/// Get a specific tweak by ID
pub fn get_tweak(tweak_id: &str) -> Result<Option<TweakDefinition>, Error> {
    let tweaks = load_all_tweaks()?;
    Ok(tweaks.get(tweak_id).cloned())
}

/// Filter tweaks by Windows version
pub fn get_tweaks_for_version(version: &str) -> Result<HashMap<String, TweakDefinition>, Error> {
    let all_tweaks = load_all_tweaks()?;

    let filtered = all_tweaks
        .into_iter()
        .filter(|(_, tweak)| tweak.applies_to_version(version))
        .collect();

    Ok(filtered)
}

/// Filter tweaks by category
pub fn get_tweaks_by_category(category: &str) -> Result<HashMap<String, TweakDefinition>, Error> {
    let all_tweaks = load_all_tweaks()?;

    let filtered = all_tweaks
        .into_iter()
        .filter(|(_, tweak)| tweak.category.to_string().to_lowercase() == category.to_lowercase())
        .collect();

    Ok(filtered)
}
