use crate::error::Error;
use crate::models::{CategoryDefinition, TweakDefinition, TweakFile};
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

/// Discover all YAML files in the tweaks directory
fn discover_yaml_files() -> Result<Vec<PathBuf>, Error> {
    let tweaks_dir = get_tweaks_dir()?;
    let mut yaml_files = Vec::new();

    let entries = fs::read_dir(&tweaks_dir)
        .map_err(|e| Error::WindowsApi(format!("Failed to read tweaks directory: {}", e)))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        yaml_files.push(path);
                    }
                }
            }
        }
    }

    Ok(yaml_files)
}

/// Load a single tweak file and parse its structure
fn load_tweak_file(file_path: &PathBuf) -> Result<TweakFile, Error> {
    let content = fs::read_to_string(file_path).map_err(|e| {
        Error::WindowsApi(format!("Failed to read tweak file {:?}: {}", file_path, e))
    })?;

    let tweak_file: TweakFile = serde_yaml::from_str(&content)
        .map_err(|e| Error::WindowsApi(format!("Failed to parse YAML {:?}: {}", file_path, e)))?;

    Ok(tweak_file)
}

/// Load all categories from YAML files
pub fn load_all_categories() -> Result<Vec<CategoryDefinition>, Error> {
    let yaml_files = discover_yaml_files()?;
    let mut categories = Vec::new();

    for file_path in yaml_files {
        match load_tweak_file(&file_path) {
            Ok(tweak_file) => {
                categories.push(tweak_file.category);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load {:?}: {}", file_path, e);
            }
        }
    }

    // Sort categories by order
    categories.sort_by_key(|c| c.order);

    Ok(categories)
}

/// Load all tweaks from all YAML files (auto-discovery)
pub fn load_all_tweaks() -> Result<HashMap<String, TweakDefinition>, Error> {
    let yaml_files = discover_yaml_files()?;
    let mut tweaks = HashMap::new();

    for file_path in yaml_files {
        match load_tweak_file(&file_path) {
            Ok(tweak_file) => {
                let category_id = &tweak_file.category.id;
                for raw_tweak in tweak_file.tweaks {
                    let tweak = TweakDefinition::from_raw(raw_tweak, category_id);
                    tweaks.insert(tweak.id.clone(), tweak);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load {:?}: {}", file_path, e);
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
        .filter(|(_, tweak)| tweak.category.to_lowercase() == category.to_lowercase())
        .collect();

    Ok(filtered)
}
