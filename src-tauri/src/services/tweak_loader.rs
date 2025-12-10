use crate::error::Error;
use crate::models::{CategoryDefinition, TweakDefinition, TweakFile};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the tweaks directory path
fn get_tweaks_dir() -> Result<PathBuf, Error> {
    log::trace!("Searching for tweaks directory...");

    // First, try to find tweaks directory relative to executable (production)
    if let Ok(exe_path) = std::env::current_exe() {
        log::trace!("Executable path: {:?}", exe_path);
        let tweaks_dir = exe_path.parent().map(|p| p.join("tweaks"));

        if let Some(dir) = tweaks_dir {
            log::trace!("Checking exe-relative path: {:?}", dir);
            if dir.exists() {
                log::debug!("Found tweaks directory at exe-relative path: {:?}", dir);
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
            log::trace!("Checking resources path: {:?}", dir);
            if dir.exists() {
                log::debug!("Found tweaks directory in resources: {:?}", dir);
                return Ok(dir);
            }
        }
    }

    // For development, use CARGO_MANIFEST_DIR or current working directory
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        log::trace!("CARGO_MANIFEST_DIR: {}", manifest_dir);
        let dev_tweaks = PathBuf::from(&manifest_dir).join("tweaks");
        log::trace!("Checking CARGO_MANIFEST_DIR path: {:?}", dev_tweaks);
        if dev_tweaks.exists() {
            log::debug!(
                "Found tweaks directory via CARGO_MANIFEST_DIR: {:?}",
                dev_tweaks
            );
            return Ok(dev_tweaks);
        }
    }

    // Try current working directory (for development)
    if let Ok(cwd) = std::env::current_dir() {
        log::trace!("Current working directory: {:?}", cwd);

        // Check if we're in the project root
        let cwd_tweaks = cwd.join("src-tauri").join("tweaks");
        log::trace!("Checking project root path: {:?}", cwd_tweaks);
        if cwd_tweaks.exists() {
            log::debug!("Found tweaks directory in project root: {:?}", cwd_tweaks);
            return Ok(cwd_tweaks);
        }

        // Check if we're already in src-tauri
        let cwd_tweaks = cwd.join("tweaks");
        log::trace!("Checking cwd/tweaks path: {:?}", cwd_tweaks);
        if cwd_tweaks.exists() {
            log::debug!("Found tweaks directory in cwd: {:?}", cwd_tweaks);
            return Ok(cwd_tweaks);
        }
    }

    log::error!("Could not find tweaks directory in any expected location");
    Err(Error::WindowsApi(
        "Could not find tweaks directory".to_string(),
    ))
}

/// Discover all YAML files in the tweaks directory
fn discover_yaml_files() -> Result<Vec<PathBuf>, Error> {
    let tweaks_dir = get_tweaks_dir()?;
    log::debug!("Discovering YAML files in: {:?}", tweaks_dir);
    let mut yaml_files = Vec::new();

    let entries = fs::read_dir(&tweaks_dir)
        .map_err(|e| Error::WindowsApi(format!("Failed to read tweaks directory: {}", e)))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        log::trace!("Found YAML file: {:?}", path.file_name());
                        yaml_files.push(path);
                    }
                }
            }
        }
    }

    log::info!("Discovered {} YAML tweak files", yaml_files.len());
    Ok(yaml_files)
}

/// Load a single tweak file and parse its structure
fn load_tweak_file(file_path: &PathBuf) -> Result<TweakFile, Error> {
    log::trace!("Loading tweak file: {:?}", file_path.file_name());

    let content = fs::read_to_string(file_path).map_err(|e| {
        log::error!("Failed to read tweak file {:?}: {}", file_path, e);
        Error::WindowsApi(format!("Failed to read tweak file {:?}: {}", file_path, e))
    })?;

    let tweak_file: TweakFile = serde_yaml::from_str(&content).map_err(|e| {
        log::error!("Failed to parse YAML {:?}: {}", file_path, e);
        Error::WindowsApi(format!("Failed to parse YAML {:?}: {}", file_path, e))
    })?;

    log::debug!(
        "Loaded category '{}' with {} tweaks from {:?}",
        tweak_file.category.name,
        tweak_file.tweaks.len(),
        file_path.file_name()
    );

    Ok(tweak_file)
}

/// Load all categories from YAML files
pub fn load_all_categories() -> Result<Vec<CategoryDefinition>, Error> {
    log::debug!("Loading all categories...");
    let yaml_files = discover_yaml_files()?;
    let mut categories = Vec::new();

    for file_path in yaml_files {
        match load_tweak_file(&file_path) {
            Ok(tweak_file) => {
                log::trace!(
                    "Added category: {} ({})",
                    tweak_file.category.name,
                    tweak_file.category.id
                );
                categories.push(tweak_file.category);
            }
            Err(e) => {
                log::warn!("Skipping file {:?} due to error: {}", file_path, e);
            }
        }
    }

    // Sort categories by order
    categories.sort_by_key(|c| c.order);
    log::info!("Loaded {} categories", categories.len());

    Ok(categories)
}

/// Load all tweaks from all YAML files (auto-discovery)
pub fn load_all_tweaks() -> Result<HashMap<String, TweakDefinition>, Error> {
    log::debug!("Loading all tweaks...");
    let yaml_files = discover_yaml_files()?;
    let mut tweaks = HashMap::new();

    for file_path in yaml_files {
        match load_tweak_file(&file_path) {
            Ok(tweak_file) => {
                let category_id = &tweak_file.category.id;
                for raw_tweak in tweak_file.tweaks {
                    let tweak = TweakDefinition::from_raw(raw_tweak, category_id);
                    log::trace!("Loaded tweak: {} ({})", tweak.name, tweak.id);
                    tweaks.insert(tweak.id.clone(), tweak);
                }
            }
            Err(e) => {
                log::warn!("Skipping file {:?} due to error: {}", file_path, e);
            }
        }
    }

    if tweaks.is_empty() {
        log::error!("No tweaks loaded from any YAML files!");
        return Err(Error::WindowsApi(format!(
            "No tweaks loaded. Tweaks directory: {:?}",
            get_tweaks_dir().ok()
        )));
    }

    log::info!("Loaded {} total tweaks", tweaks.len());
    Ok(tweaks)
}

/// Get a specific tweak by ID
pub fn get_tweak(tweak_id: &str) -> Result<Option<TweakDefinition>, Error> {
    log::trace!("Looking up tweak: {}", tweak_id);
    let tweaks = load_all_tweaks()?;
    let result = tweaks.get(tweak_id).cloned();
    if result.is_none() {
        log::debug!("Tweak not found: {}", tweak_id);
    }
    Ok(result)
}

/// Filter tweaks by Windows version (u32: 10 or 11)
pub fn get_tweaks_for_version(version: u32) -> Result<HashMap<String, TweakDefinition>, Error> {
    log::debug!("Getting tweaks for Windows version: {}", version);
    let all_tweaks = load_all_tweaks()?;
    let total = all_tweaks.len();

    let filtered: HashMap<_, _> = all_tweaks
        .into_iter()
        .filter(|(_, tweak)| tweak.applies_to_version(version))
        .collect();

    log::info!(
        "Filtered tweaks for Windows {}: {} of {} applicable",
        version,
        filtered.len(),
        total
    );
    Ok(filtered)
}

/// Filter tweaks by category
pub fn get_tweaks_by_category(category: &str) -> Result<HashMap<String, TweakDefinition>, Error> {
    log::debug!("Getting tweaks for category: {}", category);
    let all_tweaks = load_all_tweaks()?;

    let filtered: HashMap<_, _> = all_tweaks
        .into_iter()
        .filter(|(_, tweak)| tweak.category.to_lowercase() == category.to_lowercase())
        .collect();

    log::debug!("Found {} tweaks in category '{}'", filtered.len(), category);
    Ok(filtered)
}
