//! Update commands for checking and installing app updates from GitHub Releases

use crate::Error;
use serde::{Deserialize, Serialize};
use std::io::Read;
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
pub fn check_for_update(app: tauri::AppHandle, config: UpdateConfig) -> Result<UpdateInfo, Error> {
    log::info!("Checking for updates from GitHub...");

    let current_version = app.package_info().version.to_string();
    log::debug!("Current version: {}", current_version);

    // Fetch latest release from GitHub API. ureq surfaces a non-2xx status as `Err(Status(..))`,
    // so the rate-limit (403) and no-releases (404) cases are handled in the error arm.
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let response = match agent
        .get(&config.releases_api_url)
        .set("User-Agent", "MagicX-Toolbox-Updater")
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(403, resp)) => {
            let remaining = resp.header("x-ratelimit-remaining").unwrap_or("unknown");
            log::warn!("GitHub API rate limit. Remaining: {}", remaining);
            return Err(Error::Update(
                "GitHub API rate limit exceeded. Please try again later.".into(),
            ));
        }
        Err(ureq::Error::Status(404, _)) => {
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
        Err(ureq::Error::Status(code, _)) => {
            return Err(Error::Update(format!(
                "GitHub API returned status: {}",
                code
            )));
        }
        Err(ureq::Error::Transport(t)) => {
            log::error!("Failed to fetch releases: {}", t);
            return Err(Error::Update(
                "Failed to fetch update info. Please check your internet connection.".into(),
            ));
        }
    };

    let release: GitHubRelease = response.into_json().map_err(|e| {
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

// Each prefix ends in '/': without it "magicx-toolbox-evil" matches too.
const ALLOWED_DOWNLOAD_PREFIXES: &[&str] = &[
    "https://github.com/ehsan18t/magicx-toolbox/",
    "https://objects.githubusercontent.com/",
];

fn is_trusted_download_url(url: &str) -> bool {
    ALLOWED_DOWNLOAD_PREFIXES
        .iter()
        .any(|prefix| url.starts_with(prefix))
}

// Windows opens the device for a reserved stem whatever the extension ("NUL.exe", "COM1 .msi").
fn is_reserved_device(stem: &str) -> bool {
    let b = stem.trim_end().as_bytes();
    let numbered =
        |p: &[u8]| b.len() == 4 && b[..3].eq_ignore_ascii_case(p) && b[3].is_ascii_digit();
    ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|d| b.eq_ignore_ascii_case(d.as_bytes()))
        || numbered(b"COM")
        || numbered(b"LPT")
}

fn validate_asset_name(name: &str) -> Result<(), Error> {
    // ASCII only: bidi/zero-width characters disguise a name and superscript digits open COM¹.
    // ':' is a drive prefix or an ADS; `join` replaces the whole base on "C:x.exe".
    let allowed = |c: char| matches!(c, ' '..='~') && !r#"<>:"/\|?*"#.contains(c);
    let stem = name.split('.').next().unwrap_or_default();
    if name.contains("..") || !name.chars().all(allowed) || is_reserved_device(stem) {
        log::error!("Rejected invalid asset name: {:?}", name);
        return Err(Error::Update("Invalid asset name".into()));
    }

    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if !matches!(extension.to_lowercase().as_str(), "exe" | "msi") {
        log::error!("Rejected unsupported file type: {}", extension);
        return Err(Error::Update(
            "Unsupported installer type. Only .exe and .msi files are allowed.".into(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn install_update(download_url: String, asset_name: String) -> Result<(), Error> {
    tauri::async_runtime::spawn_blocking(move || {
        install_update_in(
            crate::tweaks::engine::lifecycle::gate(),
            download_url,
            asset_name,
        )
    })
    .await?
}

fn install_update_in(
    gate: &crate::tweaks::engine::lifecycle::ApplyGate,
    download_url: String,
    asset_name: String,
) -> Result<(), Error> {
    log::info!("Starting update download: {:?}", asset_name);

    // Latched through the download and kept once the installer runs, until the frontend exits: the
    // broker spawns current_exe(), so no apply may run while the installer replaces this executable.
    let latch = gate
        .begin_exit()
        .map_err(|refused| Error::exit_refused(refused, "install the update"))?;

    // Security: Validate download URL is from trusted source
    if !is_trusted_download_url(&download_url) {
        log::error!("Rejected untrusted download URL: {:?}", download_url);
        return Err(Error::Update(
            "Download URL is not from a trusted source. Updates must come from the official GitHub repository.".into()
        ));
    }

    validate_asset_name(&asset_name)?;

    let temp_dir = std::env::temp_dir();
    let download_path = temp_dir.join(&asset_name);

    log::debug!("Downloading to: {:?}", download_path);

    // Download the file. ureq returns Err on a non-2xx status, so a failed download is caught here.
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for downloads
        .build();

    let response = agent
        .get(&download_url)
        .set("User-Agent", "MagicX-Toolbox-Updater")
        .call()
        .map_err(|e| {
            log::error!("Failed to download update: {}", e);
            Error::Update(format!("Failed to download update: {}", e))
        })?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| {
            log::error!("Failed to read download: {}", e);
            Error::Update(format!("Failed to read downloaded data: {}", e))
        })?;

    // Write to temp file
    std::fs::write(&download_path, &bytes).map_err(|e| {
        log::error!("Failed to write update file: {}", e);
        Error::Update(format!("Failed to save update file: {}", e))
    })?;

    log::info!("Download complete, launching installer...");

    let extension = download_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let result = if extension.eq_ignore_ascii_case("msi") {
        crate::services::system32::SystemTool::Msiexec
            .command()
            .map_err(|e| std::io::Error::other(e.to_string()))
            .and_then(|mut msiexec| {
                msiexec
                    .arg("/i")
                    .arg(&download_path)
                    .arg("/passive")
                    .spawn()
            })
    } else {
        Command::new(&download_path).spawn()
    };

    match result {
        Ok(_) => {
            log::info!("Installer launched successfully");
            latch.keep_until_exit();
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

    #[test]
    fn bare_installer_names_are_accepted() {
        for name in [
            "x.exe",
            "MagicX-Toolbox_3.1.0_x64-setup.exe",
            "App 3.1.0.MSI",
            "x.ExE",
            "CONSOLE.exe",
            "NULL.exe",
            "COM10.exe",
            "LPTX.msi",
        ] {
            assert!(validate_asset_name(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn device_and_disguised_names_are_rejected() {
        for name in [
            "x.exe.",
            "x.exe ",
            ".exe",
            "NUL.exe",
            "con.exe",
            "Prn.msi",
            "AUX.exe",
            "COM0.exe",
            "com1.exe",
            "LPT1.msi",
            "lpt9.exe",
            "NUL .exe",
            "CON.tar.exe",
            "COM\u{b9}.exe",
            "LPT\u{b2}.msi",
            "COM\u{b3}.exe",
            "x\u{202e}exe.msi",
            "x\u{200b}.exe",
            "x\u{feff}.exe",
            "\u{e9}.exe",
        ] {
            assert!(
                matches!(validate_asset_name(name), Err(Error::Update(_))),
                "{name:?} accepted"
            );
        }
    }

    #[test]
    fn download_url_must_sit_under_the_repo_directory() {
        assert!(is_trusted_download_url(
            "https://github.com/ehsan18t/magicx-toolbox/releases/download/v3.1.0/x.exe"
        ));
        assert!(is_trusted_download_url(
            "https://objects.githubusercontent.com/x"
        ));
        for url in [
            "https://github.com/ehsan18t/magicx-toolbox-evil/releases/download/v1/x.exe",
            "https://github.com/ehsan18t/magicx-toolbox",
            "https://example.com/x.exe",
        ] {
            assert!(!is_trusted_download_url(url), "{url} trusted");
        }
    }

    #[test]
    fn non_bare_asset_names_are_rejected() {
        for name in [
            "",
            r"C:\x.exe",
            "/x.exe",
            r"..\x.exe",
            "..",
            "sub/x.exe",
            r"sub\x.exe",
            "C:x.exe",
            r"\\server\share\x.exe",
            r"\\?\C:\x.exe",
            "x.exe:stream",
            "a.txt:s.exe",
            "x<.exe",
            "x|.exe",
            "x?.exe",
            "x*.exe",
            "x\".exe",
            "x>.exe",
            "x\u{1}.exe",
            "x.zip",
        ] {
            assert!(
                matches!(validate_asset_name(name), Err(Error::Update(_))),
                "{name:?} accepted"
            );
        }
    }

    #[tokio::test]
    async fn rejected_install_leaves_the_latch_released() {
        let gate = crate::tweaks::engine::lifecycle::ApplyGate::default();
        for (url, name) in [
            ("https://example.com/x.exe", "x.exe"),
            ("https://github.com/ehsan18t/magicx-toolbox/x", "..\\x.exe"),
            ("https://github.com/ehsan18t/magicx-toolbox/x", "C:x.exe"),
            ("https://github.com/ehsan18t/magicx-toolbox/x", "x.zip"),
        ] {
            let result = install_update_in(&gate, url.into(), name.into());
            assert!(matches!(result, Err(Error::Update(_))), "got {result:?}");
        }
        drop(
            gate.begin_exit()
                .expect("latch released after every rejection"),
        );
        assert!(gate.lock_tweak("t").await.is_ok());
    }

    #[tokio::test]
    async fn install_is_refused_under_an_apply() {
        let gate = crate::tweaks::engine::lifecycle::ApplyGate::default();
        let _guard = gate.lock_tweak("t").await.expect("no exit pending");
        let result = install_update_in(&gate, "https://example.com/x.exe".into(), "x.exe".into());
        assert!(
            matches!(result, Err(Error::ApplyInFlight(_))),
            "got {result:?}"
        );
    }
}
