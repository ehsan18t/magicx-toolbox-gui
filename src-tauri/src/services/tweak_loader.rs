use crate::error::Error;
use crate::models::TweakDefinition;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the tweaks directory path
fn get_tweaks_dir() -> Result<PathBuf, Error> {
    // First, try to find tweaks directory relative to executable (production)
    if let Ok(exe_path) = std::env::current_exe() {
        let tweaks_dir = exe_path.parent().map(|p| p.join("tweaks"));

        if let Some(dir) = tweaks_dir {
            if dir.exists() {
                return Ok(dir);
            }
        }
    }

    // Try resources directory for bundled apps
    if let Ok(exe_path) = std::env::current_exe() {
        // On Windows, resources might be in a resources folder
        let resources_dir = exe_path
            .parent()
            .map(|p| p.join("resources").join("tweaks"));

        if let Some(dir) = resources_dir {
            if dir.exists() {
                return Ok(dir);
            }
        }
    }

    // For development, use CARGO_MANIFEST_DIR or current working directory
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_tweaks = PathBuf::from(&manifest_dir).join("tweaks");
        if dev_tweaks.exists() {
            return Ok(dev_tweaks);
        }
    }

    // Try current working directory (for development)
    if let Ok(cwd) = std::env::current_dir() {
        // Check if we're in the project root
        let cwd_tweaks = cwd.join("src-tauri").join("tweaks");
        if cwd_tweaks.exists() {
            return Ok(cwd_tweaks);
        }

        // Check if we're already in src-tauri
        let cwd_tweaks = cwd.join("tweaks");
        if cwd_tweaks.exists() {
            return Ok(cwd_tweaks);
        }
    }

    Err(Error::WindowsApi(
        "Could not find tweaks directory".to_string(),
    ))
}

/// Load all tweaks from YAML files
pub fn load_all_tweaks() -> Result<HashMap<String, TweakDefinition>, Error> {
    let mut tweaks = HashMap::new();

    let tweaks_dir = get_tweaks_dir()?;

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
            match load_tweaks_from_file(file_path.to_str().unwrap()) {
                Ok(category_tweaks) => {
                    for tweak in category_tweaks {
                        tweaks.insert(tweak.id.clone(), tweak);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load {}: {}", category_file, e);
                }
            }
        }
    }

    if tweaks.is_empty() {
        return Err(Error::WindowsApi(format!(
            "No tweaks loaded. Tweaks directory: {:?}",
            get_tweaks_dir().ok()
        )));
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
