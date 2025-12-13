//! Update commands for checking and installing app updates from GitHub Releases

use crate::Error;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// GitHub Release asset information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// GitHub Release information from API
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields available for future use/debugging
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub html_url: String,
    pub assets: Vec<GitHubAsset>,
}

/// Update information returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Whether an update is available
    pub available: bool,
    /// Current app version
    pub current_version: String,
    /// Latest version available (if update available)
    pub latest_version: Option<String>,
    /// Release notes for the update
    pub release_notes: Option<String>,
    /// Download URL for the update asset
    pub download_url: Option<String>,
    /// When the update was published
    pub published_at: Option<String>,
    /// Asset file name
    pub asset_name: Option<String>,
    /// Asset size in bytes
    pub asset_size: Option<u64>,
}

/// Update check configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfig {
    /// GitHub releases API URL
    pub releases_api_url: String,
    /// Regex pattern to match asset name
    pub asset_pattern: String,
}

/// Parse semantic version string to tuple for comparison
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let version = version.trim_start_matches('v');
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        // Handle pre-release suffixes like "0-beta"
        let patch_str = parts[2].split('-').next().unwrap_or(parts[2]);
        let patch = patch_str.parse().ok()?;
        Some((major, minor, patch))
    } else if parts.len() == 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor, 0))
    } else {
        None
    }
}

/// Compare two versions, returns true if latest > current
fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(curr), Some(lat)) => lat > curr,
        _ => false,
    }
}

/// Check for available updates from GitHub Releases
///
/// This command fetches the latest release from GitHub and checks if it's newer
/// than the current version. It also finds the appropriate asset based on the
/// provided regex pattern.
#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    config: UpdateConfig,
) -> Result<UpdateInfo, Error> {
    log::info!("Checking for updates from GitHub...");

    let current_version = app.package_info().version.to_string();
    log::debug!("Current version: {}", current_version);

    // Fetch latest release from GitHub API
    let client = reqwest::Client::builder()
        .user_agent("MagicX-Toolbox-Updater")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Update(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(&config.releases_api_url)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch releases: {}", e);
            if e.is_timeout() {
                Error::Update("Request timed out. Please check your internet connection.".into())
            } else if e.is_connect() {
                Error::Update("Failed to connect. Please check your internet connection.".into())
            } else {
                Error::Update(format!("Failed to fetch update info: {}", e))
            }
        })?;

    // Handle rate limiting
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        log::warn!("GitHub API rate limit. Remaining: {}", remaining);
        return Err(Error::Update(
            "GitHub API rate limit exceeded. Please try again later.".into(),
        ));
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        log::warn!("No releases found");
        return Ok(UpdateInfo {
            available: false,
            current_version,
            latest_version: None,
            release_notes: None,
            download_url: None,
            published_at: None,
            asset_name: None,
            asset_size: None,
        });
    }

    if !response.status().is_success() {
        return Err(Error::Update(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let release: GitHubRelease = response.json().await.map_err(|e| {
        log::error!("Failed to parse release JSON: {}", e);
        Error::Update("Failed to parse update information".into())
    })?;

    log::debug!("Latest release: {}", release.tag_name);

    // Parse asset pattern regex
    let asset_regex = regex_lite::Regex::new(&config.asset_pattern).map_err(|e| {
        log::error!("Invalid asset pattern regex: {}", e);
        Error::Update(format!("Invalid asset pattern: {}", e))
    })?;

    // Find matching asset
    let matching_asset = release
        .assets
        .iter()
        .find(|asset| asset_regex.is_match(&asset.name));

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let is_update_available = is_newer_version(&current_version, &latest_version);

    log::info!(
        "Update check complete: current={}, latest={}, available={}",
        current_version,
        latest_version,
        is_update_available
    );

    Ok(UpdateInfo {
        available: is_update_available,
        current_version,
        latest_version: Some(latest_version),
        release_notes: release.body,
        download_url: matching_asset.map(|a| a.browser_download_url.clone()),
        published_at: release.published_at,
        asset_name: matching_asset.map(|a| a.name.clone()),
        asset_size: matching_asset.map(|a| a.size),
    })
}

/// Allowed GitHub repository prefixes for update downloads
/// This prevents downloading from untrusted sources
const ALLOWED_DOWNLOAD_PREFIXES: &[&str] = &[
    "https://github.com/ehsan18t/magicx-toolbox",
    "https://objects.githubusercontent.com/",
];

/// Validate that a download URL is from a trusted source
fn is_trusted_download_url(url: &str) -> bool {
    ALLOWED_DOWNLOAD_PREFIXES
        .iter()
        .any(|prefix| url.starts_with(prefix))
}

/// Download and install an update
///
/// Downloads the update asset to a temporary location and launches the installer.
/// The app should exit after calling this to allow the installer to complete.
#[tauri::command]
pub async fn install_update(download_url: String, asset_name: String) -> Result<(), Error> {
    log::info!("Starting update download: {}", asset_name);

    // Security: Validate download URL is from trusted source
    if !is_trusted_download_url(&download_url) {
        log::error!("Rejected untrusted download URL: {}", download_url);
        return Err(Error::Update(
            "Download URL is not from a trusted source. Updates must come from the official GitHub repository.".into()
        ));
    }

    // Validate asset name to prevent path traversal
    if asset_name.contains("..") || asset_name.contains('/') || asset_name.contains('\\') {
        log::error!("Rejected invalid asset name: {}", asset_name);
        return Err(Error::Update("Invalid asset name".into()));
    }

    // Validate file extension
    let extension = std::path::Path::new(&asset_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if !matches!(extension.to_lowercase().as_str(), "exe" | "msi") {
        log::error!("Rejected unsupported file type: {}", extension);
        return Err(Error::Update(
            "Unsupported installer type. Only .exe and .msi files are allowed.".into(),
        ));
    }

    // Get temp directory
    let temp_dir = std::env::temp_dir();
    let download_path = temp_dir.join(&asset_name);

    log::debug!("Downloading to: {:?}", download_path);

    // Download the file
    let client = reqwest::Client::builder()
        .user_agent("MagicX-Toolbox-Updater")
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for downloads
        .build()
        .map_err(|e| Error::Update(format!("Failed to create HTTP client: {}", e)))?;

    let response = client.get(&download_url).send().await.map_err(|e| {
        log::error!("Failed to download update: {}", e);
        Error::Update(format!("Failed to download update: {}", e))
    })?;

    if !response.status().is_success() {
        return Err(Error::Update(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let bytes = response.bytes().await.map_err(|e| {
        log::error!("Failed to read download: {}", e);
        Error::Update(format!("Failed to read downloaded data: {}", e))
    })?;

    // Write to temp file
    std::fs::write(&download_path, &bytes).map_err(|e| {
        log::error!("Failed to write update file: {}", e);
        Error::Update(format!("Failed to save update file: {}", e))
    })?;

    log::info!("Download complete, launching installer...");

    // Launch the installer
    // For .exe installers, just run them
    // For .msi installers, use msiexec
    let extension = download_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let result = if extension.eq_ignore_ascii_case("msi") {
        Command::new("msiexec")
            .args(["/i", download_path.to_str().unwrap_or(""), "/passive"])
            .spawn()
    } else {
        Command::new(&download_path).spawn()
    };

    match result {
        Ok(_) => {
            log::info!("Installer launched successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to launch installer: {}", e);
            Err(Error::Update(format!("Failed to launch installer: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // parse_version tests
    // ========================================================================

    #[test]
    fn test_parse_version_three_parts() {
        assert_eq!(parse_version("3.0.0"), Some((3, 0, 0)));
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        assert_eq!(parse_version("v3.0.0"), Some((3, 0, 0)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_two_parts() {
        assert_eq!(parse_version("3.0"), Some((3, 0, 0)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
    }

    #[test]
    fn test_parse_version_with_prerelease() {
        // Should strip pre-release suffix from patch
        assert_eq!(parse_version("3.0.0-beta"), Some((3, 0, 0)));
        assert_eq!(parse_version("1.2.3-rc.1"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version("abc.def.ghi"), None);
        assert_eq!(parse_version("1"), None);
    }

    // ========================================================================
    // is_newer_version tests
    // ========================================================================

    #[test]
    fn test_is_newer_version_major() {
        assert!(is_newer_version("2.0.0", "3.0.0"));
        assert!(!is_newer_version("3.0.0", "2.0.0"));
    }

    #[test]
    fn test_is_newer_version_minor() {
        assert!(is_newer_version("3.0.0", "3.1.0"));
        assert!(!is_newer_version("3.1.0", "3.0.0"));
    }

    #[test]
    fn test_is_newer_version_patch() {
        assert!(is_newer_version("3.0.0", "3.0.1"));
        assert!(!is_newer_version("3.0.1", "3.0.0"));
    }

    #[test]
    fn test_is_newer_version_equal() {
        assert!(!is_newer_version("3.0.0", "3.0.0"));
    }

    #[test]
    fn test_is_newer_version_with_v_prefix() {
        assert!(is_newer_version("3.0.0", "v3.1.0"));
        assert!(is_newer_version("v3.0.0", "3.1.0"));
    }
}
