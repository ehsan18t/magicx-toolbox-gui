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
    /// `sha256:<hex>`, published by GitHub for every asset it stores.
    #[serde(default)]
    pub digest: Option<String>,
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
    pub asset_digest: Option<String>,
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
                asset_digest: None,
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
        asset_digest: matching_asset.and_then(|a| a.digest.clone()),
    })
}

/// The repository `releasesApiUrl` in `src/lib/config/app.ts` queries.
const RELEASE_REPO: &str = "ehsan18t/magicx-toolbox-gui";

/// The whole URL must be `https://github.com/<RELEASE_REPO>/releases/download/<tag>/<asset_name>`: a
/// prefix check admits dot segments, queries and a lookalike repository.
fn is_trusted_download_url(url: &str, asset_name: &str) -> bool {
    let prefix = format!("https://github.com/{RELEASE_REPO}/releases/download/");
    let Some(rest) = url
        .get(..prefix.len())
        .filter(|head| head.eq_ignore_ascii_case(&prefix))
        .map(|_| &url[prefix.len()..])
    else {
        return false;
    };
    let plain = |s: &str| {
        !s.is_empty()
            && s != "."
            && s != ".."
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    };
    matches!(rest.split_once('/'), Some((tag, name)) if plain(tag) && name == asset_name)
}

fn sha256(bytes: &[u8]) -> Result<[u8; 32], Error> {
    use windows_sys::Win32::Security::Cryptography::{BCryptHash, BCRYPT_SHA256_ALG_HANDLE};
    let len = u32::try_from(bytes.len())
        .map_err(|_| Error::Update("The update is too large to verify".into()))?;
    let mut out = [0u8; 32];
    // SAFETY: the pseudo-handle needs no open; input and output are valid for the lengths given.
    let status = unsafe {
        BCryptHash(
            BCRYPT_SHA256_ALG_HANDLE,
            std::ptr::null(),
            0,
            bytes.as_ptr(),
            len,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    if status != 0 {
        return Err(Error::Update(format!(
            "Could not hash the update (NTSTATUS {status:#x})"
        )));
    }
    Ok(out)
}

/// `expected` is GitHub's `sha256:<hex>`; anything else is refused rather than skipped.
fn verify_digest(expected: &str, bytes: &[u8]) -> Result<(), Error> {
    let hex = expected
        .strip_prefix("sha256:")
        .filter(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| {
            Error::Update("The release publishes no usable SHA-256 for this file".into())
        })?;
    let actual: String = sha256(bytes)?.iter().map(|b| format!("{b:02x}")).collect();
    if !actual.eq_ignore_ascii_case(hex) {
        log::error!("Downloaded update does not match the release's SHA-256");
        return Err(Error::Update(
            "The downloaded update does not match the published checksum".into(),
        ));
    }
    Ok(())
}

const STAGED_PREFIX: &str = "magicx-update-";

/// Best effort: an installer still running from an earlier update holds its file open.
fn remove_stale_installers(dir: &std::path::Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("could not list earlier update files: {}", e.kind());
            return;
        }
    };
    for entry in entries.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(STAGED_PREFIX)
        {
            discard_staged(&entry.path(), "earlier");
        }
    }
}

/// Logs `what`, never the path: a staged name is guarded only by being unguessable.
fn discard_staged(path: &std::path::Path, what: &str) {
    if let Err(e) = std::fs::remove_file(path) {
        log::warn!("could not delete the {what} update file: {}", e.kind());
    }
}

/// Written under an unguessable name, then re-opened read-only with writers and deleters denied and
/// compared with the verified bytes. Holding that handle until the installer starts is what keeps
/// the file that runs identical to the one checked.
fn stage_installer(
    dir: &std::path::Path,
    bytes: &[u8],
    asset_name: &str,
) -> Result<(std::path::PathBuf, std::fs::File), Error> {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    let io_err = |what: &str, e: std::io::Error| {
        log::error!("Failed to {what} the update file: {e}");
        Error::Update(format!("Failed to save update file: {e}"))
    };
    remove_stale_installers(dir);
    let token =
        crate::services::exclusive_temp::random_hex_token().map_err(|e| io_err("name", e))?;
    let path = dir.join(format!("{STAGED_PREFIX}{token}-{asset_name}"));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(FILE_SHARE_READ)
        .open(&path)
        .map_err(|e| io_err("create", e))?;
    let written = file.write_all(bytes);
    drop(file);
    if let Err(e) = written {
        discard_staged(&path, "partial");
        return Err(io_err("write", e));
    }

    let mut held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&path)
        .map_err(|e| io_err("reopen", e))?;
    let mut on_disk = Vec::with_capacity(bytes.len());
    held.read_to_end(&mut on_disk)
        .map_err(|e| io_err("read back", e))?;
    if on_disk != bytes {
        drop(held);
        discard_staged(&path, "altered");
        return Err(Error::Update(
            "The update file changed on disk before it could be run".into(),
        ));
    }
    Ok((path, held))
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
pub async fn install_update(
    download_url: String,
    asset_name: String,
    asset_digest: Option<String>,
) -> Result<(), Error> {
    tauri::async_runtime::spawn_blocking(move || {
        install_update_in(
            crate::tweaks::engine::lifecycle::gate(),
            download_url,
            asset_name,
            asset_digest,
        )
    })
    .await?
}

fn install_update_in(
    gate: &crate::tweaks::engine::lifecycle::ApplyGate,
    download_url: String,
    asset_name: String,
    asset_digest: Option<String>,
) -> Result<(), Error> {
    log::info!("Starting update download: {:?}", asset_name);

    // Latched through the download and kept once the installer runs, until the frontend exits: the
    // broker spawns current_exe(), so no apply may run while the installer replaces this executable.
    let latch = gate
        .begin_exit()
        .map_err(|refused| Error::exit_refused(refused, "install the update"))?;

    validate_asset_name(&asset_name)?;
    if !is_trusted_download_url(&download_url, &asset_name) {
        log::error!("Rejected untrusted download URL: {:?}", download_url);
        return Err(Error::Update(
            "Download URL is not from a trusted source. Updates must come from the official GitHub repository.".into()
        ));
    }
    let Some(asset_digest) = asset_digest else {
        return Err(Error::Update(
            "The release publishes no checksum for this file, so it cannot be verified".into(),
        ));
    };

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

    verify_digest(&asset_digest, &bytes)?;
    let (download_path, held) = stage_installer(&std::env::temp_dir(), &bytes, &asset_name)?;

    log::info!("Download verified, launching installer...");

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
        // A portable build is the app itself: without this it yields to this instance's mutex.
        Command::new(&download_path)
            .arg(crate::services::single_instance::AFTER_RESTART_ARG)
            .spawn()
    };

    match result {
        Ok(_) => {
            log::info!("Installer launched successfully");
            // msiexec opens the package after spawn returns; the OS closes this handle at exit.
            std::mem::forget(held);
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
    fn download_url_must_be_this_repos_release_asset() {
        let base = format!("https://github.com/{RELEASE_REPO}/releases/download");
        assert!(is_trusted_download_url(
            &format!("{base}/v3.1.0/x.exe"),
            "x.exe"
        ));
        for url in [
            format!("{base}/v3.1.0/y.exe"),
            format!("{base}/v3.1.0/x.exe?x=1"),
            format!("{base}/../../evil/r/releases/download/v1/x.exe"),
            format!("{base}/%2e%2e/x.exe"),
            format!("{base}//x.exe"),
            format!("{base}/v1/sub/x.exe"),
            format!("https://github.com/{RELEASE_REPO}-evil/releases/download/v1/x.exe"),
            "https://objects.githubusercontent.com/x.exe".to_string(),
            "https://example.com/x.exe".to_string(),
        ] {
            assert!(!is_trusted_download_url(&url, "x.exe"), "{url} trusted");
        }
    }

    #[test]
    fn the_digest_must_match_and_be_well_formed() {
        let abc = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify_digest(abc, b"abc").is_ok());
        assert!(verify_digest(&abc.to_uppercase().replace("SHA256", "sha256"), b"abc").is_ok());
        for bad in [
            abc.replace("ba78", "0000"),
            abc.trim_start_matches("sha256:").to_string(),
            "sha256:xyz".to_string(),
            "md5:900150983cd24fb0d6963f7d28e17f72".to_string(),
        ] {
            assert!(verify_digest(&bad, b"abc").is_err(), "{bad} accepted");
        }
    }

    #[test]
    fn the_frontends_inline_case_flag_matches_any_case() {
        let pattern = regex_lite::Regex::new(r"(?i)MagicX[-_]Toolbox.*x64.*\.(exe|msi)$").unwrap();
        assert!(pattern.is_match("magicx-toolbox_3.1.0_X64-setup.EXE"));
        assert!(!pattern.is_match("MagicX-Toolbox_3.1.0_arm64.msi"));
    }

    #[test]
    fn a_staged_installer_cannot_be_rewritten_while_held() {
        const ERROR_SHARING_VIOLATION: i32 = 32;
        let dir = tempfile::tempdir().unwrap();
        let (path, held) = stage_installer(dir.path(), b"payload", "x.exe").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"payload");
        let err = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .expect_err("no writer while the installer is held");
        assert_eq!(err.raw_os_error(), Some(ERROR_SHARING_VIOLATION));
        drop(held);
    }

    #[test]
    fn staging_clears_earlier_installers_and_nothing_else() {
        let dir = tempfile::tempdir().unwrap();
        let stale = dir.path().join(format!("{STAGED_PREFIX}0123-x.exe"));
        let unrelated = dir.path().join("keep.exe");
        std::fs::write(&stale, b"old").unwrap();
        std::fs::write(&unrelated, b"other").unwrap();
        let (path, held) = stage_installer(dir.path(), b"new", "x.exe").unwrap();
        assert!(!stale.exists(), "an earlier installer was left behind");
        assert!(unrelated.exists());
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        drop(held);
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
        let ok = format!("https://github.com/{RELEASE_REPO}/releases/download/v1/x.exe");
        for (url, name, digest) in [
            ("https://example.com/x.exe", "x.exe", Some("sha256:00")),
            (ok.as_str(), "..\\x.exe", Some("sha256:00")),
            (ok.as_str(), "C:x.exe", Some("sha256:00")),
            (ok.as_str(), "x.zip", Some("sha256:00")),
            (ok.as_str(), "x.exe", None),
        ] {
            let result =
                install_update_in(&gate, url.into(), name.into(), digest.map(str::to_string));
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
        let result = install_update_in(
            &gate,
            "https://example.com/x.exe".into(),
            "x.exe".into(),
            None,
        );
        assert!(
            matches!(result, Err(Error::ApplyInFlight(_))),
            "got {result:?}"
        );
    }
}
