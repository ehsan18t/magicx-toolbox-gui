//! Update commands: check GitHub Releases, then replace the running portable exe in place.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The one asset a release ships: the portable exe an update swaps in.
const PORTABLE_ASSET: &str = "magicx-toolbox.exe";

fn is_portable_asset(name: &str) -> bool {
    name.eq_ignore_ascii_case(PORTABLE_ASSET)
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    /// `sha256:<hex>`, published by GitHub for every asset it stores.
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub assets: Vec<GitHubAsset>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub published_at: Option<String>,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
    /// GitHub's `sha256:<hex>`; an update without one is not installed.
    pub asset_digest: Option<String>,
    /// The offered release is marked pre-release on GitHub.
    pub prerelease: bool,
}

impl UpdateInfo {
    fn none(current_version: String) -> Self {
        Self {
            available: false,
            current_version,
            latest_version: None,
            release_notes: None,
            download_url: None,
            published_at: None,
            asset_name: None,
            asset_size: None,
            asset_digest: None,
            prerelease: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfig {
    /// Offer releases GitHub marks pre-release; off, only stable releases count.
    #[serde(default)]
    pub include_prereleases: bool,
}

const RELEASES_PER_PAGE: u32 = 30;

fn strip_v_prefix(version: &str) -> &str {
    version.trim_start_matches(['v', 'V'])
}

/// `major.minor.patch[-pre]`; an empty `pre` is a release, which outranks every pre-release of it.
#[derive(Debug)]
struct Version {
    core: (u64, u64, u64),
    pre: Vec<String>,
}

fn parse_version(version: &str) -> Option<Version> {
    let version = strip_v_prefix(version.trim());
    let version = version.split('+').next().unwrap_or(version);
    let (core, pre) = version.split_once('-').unwrap_or((version, ""));
    let num = |s: &str| s.parse::<u64>().ok();
    let core = match core.split('.').collect::<Vec<_>>()[..] {
        [major, minor, patch] => (num(major)?, num(minor)?, num(patch)?),
        [major, minor] => (num(major)?, num(minor)?, 0),
        _ => return None,
    };
    let pre = pre
        .split('.')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    Some(Version { core, pre })
}

/// SemVer precedence: numeric identifiers compare as numbers and sort before alphanumeric ones.
impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.core
            .cmp(&other.core)
            .then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => {
                    for (a, b) in self.pre.iter().zip(&other.pre) {
                        let order = match (a.parse::<u64>(), b.parse::<u64>()) {
                            (Ok(x), Ok(y)) => x.cmp(&y),
                            (Ok(_), Err(_)) => Ordering::Less,
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => a.cmp(b),
                        };
                        if order != Ordering::Equal {
                            return order;
                        }
                    }
                    self.pre.len().cmp(&other.pre.len())
                }
            })
    }
}

// Equality follows `cmp`, not the strings: `rc.01` and `rc.1` are one version.
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(curr), Some(lat)) => lat > curr,
        _ => false,
    }
}

/// The newest published release that ships the portable exe, skipping drafts, and pre-releases
/// unless they are wanted. A `-pre` tag counts as a pre-release even when GitHub's flag is unset.
fn newest_release(
    releases: Vec<GitHubRelease>,
    include_prereleases: bool,
) -> Option<GitHubRelease> {
    releases
        .into_iter()
        .filter(|r| !r.draft && portable_asset(&r.assets).is_some())
        .filter_map(|mut r| {
            let v = parse_version(&r.tag_name)?;
            r.prerelease |= !v.pre.is_empty();
            (include_prereleases || !r.prerelease).then_some((v, r))
        })
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, r)| r)
}

#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle, config: UpdateConfig) -> Result<UpdateInfo> {
    log::info!("Checking for updates from GitHub...");
    let current_version = app.package_info().version.to_string();
    tauri::async_runtime::spawn_blocking(move || check_for_update_in(current_version, config))
        .await?
}

fn check_for_update_in(current_version: String, config: UpdateConfig) -> Result<UpdateInfo> {
    log::debug!("Current version: {}", current_version);

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(30)))
        .http_status_as_error(false)
        .build()
        .into();

    // `/releases/latest` never returns a pre-release, so read the list and choose here.
    let list_url = format!(
        "https://api.github.com/repos/{RELEASE_REPO}/releases?per_page={RELEASES_PER_PAGE}"
    );
    let mut response = agent
        .get(&list_url)
        .header("User-Agent", "MagicX-Toolbox-Updater")
        .call()
        .map_err(|e| {
            log::error!("Failed to fetch releases: {}", e);
            Error::Update(
                "Failed to fetch update info. Please check your internet connection.".into(),
            )
        })?;

    match response.status().as_u16() {
        200..=299 => {}
        403 => {
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
        // The list endpoint answers an empty repo with `[]`; 404 means the repo or URL is wrong.
        404 => {
            log::error!("Releases endpoint not found: {}", list_url);
            return Err(Error::Update(
                "Could not find the release feed. Please try again later.".into(),
            ));
        }
        code => {
            return Err(Error::Update(format!(
                "GitHub API returned status: {}",
                code
            )));
        }
    }

    let releases: Vec<GitHubRelease> = response.body_mut().read_json().map_err(|e| {
        log::error!("Failed to parse release JSON: {}", e);
        Error::Update("Failed to parse update information".into())
    })?;
    let Some(release) = newest_release(releases, config.include_prereleases) else {
        log::info!("Update check complete: no published release to offer");
        return Ok(UpdateInfo::none(current_version));
    };

    log::debug!("Latest release: {}", release.tag_name);

    let matching_asset = portable_asset(&release.assets);

    let latest_version = strip_v_prefix(&release.tag_name).to_string();
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
        prerelease: release.prerelease,
    })
}

fn portable_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    assets.iter().find(|asset| is_portable_asset(&asset.name))
}

/// The one repository updates are read from and downloaded from.
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

fn sha256(bytes: &[u8]) -> Result<[u8; 32]> {
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
fn verify_digest(expected: &str, bytes: &[u8]) -> Result<()> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub downloaded: u64,
    /// Bytes; `None` when the server sends no size.
    pub total: Option<u64>,
}

impl DownloadProgress {
    fn percent(&self) -> Option<u64> {
        self.total
            .filter(|&total| total > 0)
            .map(|total| (u128::from(self.downloaded) * 100 / u128::from(total)).min(100) as u64)
    }
}

const UNKNOWN_SIZE_STEP: u64 = 256 * 1024;

/// Reports a download only when its whole percent changes (with no known size, each 256 KiB).
struct ProgressMeter {
    now: DownloadProgress,
    last_step: Option<u64>,
}

impl ProgressMeter {
    fn new(total: Option<u64>) -> Self {
        Self {
            now: DownloadProgress {
                downloaded: 0,
                total,
            },
            last_step: None,
        }
    }

    fn advance(&mut self, read: u64) -> Option<DownloadProgress> {
        self.now.downloaded = self.now.downloaded.saturating_add(read);
        let step = self
            .now
            .percent()
            .unwrap_or(self.now.downloaded / UNKNOWN_SIZE_STEP);
        (self.last_step != Some(step)).then(|| {
            self.last_step = Some(step);
            self.now
        })
    }
}

#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    download_url: String,
    asset_name: String,
    asset_digest: Option<String>,
    asset_size: Option<u64>,
    on_progress: tauri::ipc::Channel<DownloadProgress>,
) -> Result<()> {
    let exe = std::env::current_exe()
        .map_err(|e| Error::Update(format!("Could not find the running app: {e}")))?;
    let host = app.clone();
    let mut channel = Some(on_progress);
    let report = move |progress: DownloadProgress| {
        if let Some(Err(e)) = channel.as_ref().map(|c| c.send(progress)) {
            log::warn!("update progress not delivered, no more will be sent: {e}");
            channel = None;
        }
        if let Some(percent) = progress.percent() {
            crate::taskbar::progress(&host, percent);
        }
    };
    let saver = app.clone();
    let exiter = app.clone();
    let work = tauri::async_runtime::spawn_blocking(move || {
        install_update_in(
            crate::tweaks::engine::lifecycle::gate(),
            &UpdatePaths::beside(exe),
            Download {
                url: download_url,
                asset_name,
                digest: asset_digest,
                size: asset_size,
            },
            report,
            || crate::window_state::save(&saver),
            || exiter.exit(0),
        )
    });
    crate::taskbar::track(&app, async { work.await? }).await
}

struct Download {
    url: String,
    asset_name: String,
    digest: Option<String>,
    size: Option<u64>,
}

fn install_update_in(
    gate: &crate::tweaks::engine::lifecycle::ApplyGate,
    paths: &UpdatePaths,
    download: Download,
    on_progress: impl FnMut(DownloadProgress),
    before_launch: impl FnOnce(),
    exit: impl FnOnce(),
) -> Result<()> {
    log::info!("Starting update download: {:?}", download.asset_name);

    // Latched through the download and kept until exit: the broker spawns current_exe(), so no
    // apply may run while that file is being replaced.
    let latch = gate
        .begin_exit()
        .map_err(|refused| Error::exit_refused(refused, "install the update"))?;

    if !is_portable_asset(&download.asset_name) {
        log::error!("Rejected asset: {:?}", download.asset_name);
        return Err(Error::Update(
            "The release file is not the app's portable exe".into(),
        ));
    }
    if !is_trusted_download_url(&download.url, &download.asset_name) {
        log::error!("Rejected untrusted download URL: {:?}", download.url);
        return Err(Error::Update(
            "Download URL is not from a trusted source. Updates must come from the official GitHub repository.".into()
        ));
    }
    let Some(digest) = download.digest else {
        return Err(Error::Update(
            "The release publishes no checksum for this file, so it cannot be verified".into(),
        ));
    };

    // Created before the download, so a folder the app cannot write to fails at once.
    let file =
        std::fs::File::create(&paths.staged).map_err(|e| fs_error("create the update file", &e))?;
    let staged = fetch(&download.url, download.size, on_progress).and_then(|bytes| {
        verify_digest(&digest, &bytes)?;
        write_staged(file, &bytes)
    });
    if let Err(e) = staged {
        discard(&paths.staged);
        return Err(e);
    }

    log::info!("Download verified, replacing the app");
    before_launch();
    let dir = paths.exe.parent().unwrap_or(Path::new("."));
    replace_and_launch(paths, |exe| {
        // Without it the new instance yields to this one's single-instance mutex.
        Command::new(exe)
            .arg(crate::services::single_instance::after_restart_arg())
            .current_dir(dir)
            .spawn()
            .map(drop)
    })?;
    log::info!("Updated app started, exiting this instance");
    // `exit` only queues the exit, so the latch must outlive this command.
    latch.keep_until_exit();
    exit();
    Ok(())
}

fn fetch(
    url: &str,
    size: Option<u64>,
    mut on_progress: impl FnMut(DownloadProgress),
) -> Result<Vec<u8>> {
    // ureq returns Err on a non-2xx status, so a failed download is caught here.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(download_budget(size)))
        .build()
        .into();

    let response = agent
        .get(url)
        .header("User-Agent", "MagicX-Toolbox-Updater")
        .call()
        .map_err(|e| {
            log::error!("Failed to download update: {}", e);
            Error::Update(format!("Failed to download update: {}", e))
        })?;

    let body = response.into_body();
    let mut meter = ProgressMeter::new(body.content_length().or(size));
    // `into_reader` is unbounded; `read_to_vec` would cap the exe at 10 MB.
    let mut reader = body.into_reader();
    let mut bytes = Vec::new();
    let mut chunk = vec![0u8; 64 * 1024];
    if let Some(progress) = meter.advance(0) {
        on_progress(progress);
    }
    loop {
        let read = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => read,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                log::error!("Failed to read download: {}", e);
                return Err(Error::Update(format!(
                    "Failed to read downloaded data: {}",
                    e
                )));
            }
        };
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(progress) = meter.advance(read as u64) {
            on_progress(progress);
        }
    }
    Ok(bytes)
}

// Scaled to the size, not an idle timeout: ureq offers those only through its unstable `unversioned` API.
fn download_budget(size: Option<u64>) -> std::time::Duration {
    const MIN_BYTES_PER_SEC: u64 = 16 * 1024;
    const FLOOR_SECS: u64 = 300;
    const UNKNOWN_SIZE_SECS: u64 = 1800;
    std::time::Duration::from_secs(size.map_or(UNKNOWN_SIZE_SECS, |s| {
        (s / MIN_BYTES_PER_SEC).max(FLOOR_SECS)
    }))
}

/// The running exe and its two siblings. Kept in one folder so both renames stay on one volume.
#[derive(Debug)]
struct UpdatePaths {
    exe: PathBuf,
    /// The verified download, before it takes the exe's name.
    staged: PathBuf,
    /// The replaced exe, deleted at the next start.
    old: PathBuf,
}

impl UpdatePaths {
    fn beside(exe: PathBuf) -> Self {
        let sibling = |suffix: &str| {
            let mut name = exe.clone().into_os_string();
            name.push(suffix);
            PathBuf::from(name)
        };
        Self {
            staged: sibling(".new"),
            old: sibling(".old"),
            exe,
        }
    }
}

const ERROR_WRITE_PROTECT: i32 = 19;

fn fs_error(what: &str, e: &std::io::Error) -> Error {
    log::error!("Failed to {what}: {e}");
    if e.kind() == std::io::ErrorKind::PermissionDenied
        || e.raw_os_error() == Some(ERROR_WRITE_PROTECT)
    {
        Error::UpdateFolderReadOnly
    } else {
        Error::Update(format!("Could not {what}: {e}"))
    }
}

/// Logs the failure, never the path.
fn discard(path: &Path) {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            log::warn!("could not delete the update file: {}", e.kind());
        }
        _ => {}
    }
}

// No hold-open guard against a swap after the write: whoever can write this folder can already
// replace the exe itself.
fn write_staged(mut file: std::fs::File, bytes: &[u8]) -> Result<()> {
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| fs_error("save the update", &e))
}

/// Moves the running exe aside and the staged file into its place, then starts it. Any failure
/// puts the previous exe back and discards the staged file before returning.
fn replace_and_launch(
    paths: &UpdatePaths,
    launch: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<()> {
    // A stale or locked `.old` would fail the rename below as if the folder were read-only.
    match std::fs::remove_file(&paths.old) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            discard(&paths.staged);
            log::error!("Failed to remove the previous update's leftover: {e}");
            return Err(Error::Update(format!(
                "{} is left over from the last update and could not be removed ({e}). Delete it, then try again.",
                file_name(&paths.old)
            )));
        }
        _ => {}
    }
    // Two renames, not one replace: Windows renames a running exe but will not overwrite it
    // (MOVEFILE_REPLACE_EXISTING fails on a mapped image). Nothing runs between the two.
    if let Err(e) = std::fs::rename(&paths.exe, &paths.old) {
        discard(&paths.staged);
        return Err(fs_error("move the running app aside", &e));
    }
    if let Err(e) = std::fs::rename(&paths.staged, &paths.exe) {
        return Err(roll_back(paths, fs_error("move the update into place", &e)));
    }
    if let Err(e) = launch(&paths.exe) {
        log::error!("Failed to start the updated app: {e}");
        return Err(roll_back(
            paths,
            Error::Update(format!("Could not start the updated app: {e}")),
        ));
    }
    Ok(())
}

/// `cause` when the previous exe is back in place; otherwise an error saying how to restore it.
fn roll_back(paths: &UpdatePaths, cause: Error) -> Error {
    match std::fs::rename(&paths.old, &paths.exe) {
        Ok(()) => {
            discard(&paths.staged);
            cause
        }
        Err(e) => {
            log::error!("Failed to restore the previous app after a failed update: {e}");
            Error::Update(format!(
                "The update failed and the previous version could not be put back ({e}). Rename {} to {} to restore it.",
                file_name(&paths.old),
                file_name(&paths.exe)
            ))
        }
    }
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// For a startup failure report: how to go back to the version the last update replaced.
pub fn previous_version_hint() -> Option<String> {
    let paths = UpdatePaths::beside(std::env::current_exe().ok()?);
    paths.old.is_file().then(|| {
        format!(
            "The previous version is still at {}. To go back to it, delete {} and rename {} to {}.",
            paths.old.display(),
            file_name(&paths.exe),
            file_name(&paths.old),
            file_name(&paths.exe)
        )
    })
}

/// Deletes the exe an update replaced, and any interrupted download. Called once the window has
/// shown, so a release that cannot start keeps the version to go back to.
pub fn remove_update_leftovers() {
    match std::env::current_exe() {
        Ok(exe) => remove_leftovers(&UpdatePaths::beside(exe)),
        Err(e) => log::warn!("could not look for files left by an update: {e}"),
    }
}

/// One attempt; a file still held is retried at the next start.
fn remove_leftovers(paths: &UpdatePaths) {
    discard(&paths.staged);
    match std::fs::remove_file(&paths.old) {
        Ok(()) => log::info!("Removed the app version replaced by the last update"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => log::warn!(
            "could not delete the replaced app version, will retry next start: {}",
            e.kind()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_budget_scales_with_size_above_a_floor() {
        let secs = |size| download_budget(size).as_secs();
        assert_eq!(secs(Some(1024)), 300);
        assert_eq!(secs(Some(16 * 1024 * 600)), 600);
        assert_eq!(secs(None), 1800);
    }

    fn core(v: &str) -> Option<(u64, u64, u64)> {
        parse_version(v).map(|v| v.core)
    }

    #[test]
    fn test_parse_version_three_parts() {
        assert_eq!(core("3.0.0"), Some((3, 0, 0)));
        assert_eq!(core("1.2.3"), Some((1, 2, 3)));
        assert_eq!(core("10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        assert_eq!(core("v3.0.0"), Some((3, 0, 0)));
        assert_eq!(core("v1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_two_parts() {
        assert_eq!(core("3.0"), Some((3, 0, 0)));
        assert_eq!(core("1.2"), Some((1, 2, 0)));
    }

    #[test]
    fn test_parse_version_with_prerelease() {
        assert_eq!(core("3.0.0-beta"), Some((3, 0, 0)));
        assert_eq!(
            parse_version("1.2.3-rc.1").map(|v| v.pre),
            Some(vec!["rc".into(), "1".into()])
        );
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version("abc.def.ghi"), None);
        assert_eq!(parse_version("1"), None);
    }

    #[test]
    fn prerelease_ordering_follows_semver() {
        let ordered = [
            "3.0.0",
            "3.1.0-alpha",
            "3.1.0-alpha.1",
            "3.1.0-alpha.beta",
            "3.1.0-beta",
            "3.1.0-beta.2",
            "3.1.0-beta.11",
            "3.1.0-rc.1",
            "3.1.0",
        ];
        for pair in ordered.windows(2) {
            assert!(
                is_newer_version(pair[0], pair[1]),
                "{} < {}",
                pair[0],
                pair[1]
            );
            assert!(
                !is_newer_version(pair[1], pair[0]),
                "{} > {}",
                pair[1],
                pair[0]
            );
        }
    }

    fn release(tag: &str, prerelease: bool, draft: bool) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag.into(),
            body: None,
            published_at: None,
            assets: vec![asset(PORTABLE_ASSET)],
            prerelease,
            draft,
        }
    }

    #[test]
    fn a_release_without_the_portable_exe_is_never_offered() {
        let bare = GitHubRelease {
            assets: vec![asset("magicx-toolbox-setup.msi")],
            ..release("v3.2.0", false, false)
        };
        assert_eq!(
            newest_release(vec![release("v3.0.0", false, false), bare], false).map(|r| r.tag_name),
            Some("v3.0.0".into())
        );
    }

    #[test]
    fn newest_release_skips_prereleases_unless_wanted_and_always_skips_drafts() {
        let list = || {
            vec![
                release("v3.0.0", false, false),
                release("v3.1.0-beta.1", true, false),
                release("v3.2.0", false, true),
            ]
        };
        assert_eq!(
            newest_release(list(), false).map(|r| r.tag_name),
            Some("v3.0.0".into())
        );
        assert_eq!(
            newest_release(list(), true).map(|r| r.tag_name),
            Some("v3.1.0-beta.1".into())
        );
        assert!(newest_release(Vec::new(), true).is_none());
    }

    #[test]
    fn a_pre_release_tag_counts_as_prerelease_without_githubs_flag() {
        let list = || {
            vec![
                release("v3.0.0", false, false),
                release("v3.1.0-rc.1", false, false),
            ]
        };
        assert_eq!(
            newest_release(list(), false).map(|r| r.tag_name),
            Some("v3.0.0".into())
        );
        assert!(newest_release(list(), true).is_some_and(|r| r.prerelease));
    }

    #[test]
    fn equality_agrees_with_ordering() {
        let a = parse_version("3.1.0-rc.01").unwrap();
        let b = parse_version("3.1.0-rc.1").unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);
        assert_eq!(a, b);
        assert_ne!(a, parse_version("3.1.0-rc.2").unwrap());
    }

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

    fn asset(name: &str) -> GitHubAsset {
        GitHubAsset {
            name: name.into(),
            browser_download_url: String::new(),
            size: 0,
            digest: None,
        }
    }

    #[test]
    fn only_the_portable_exe_is_offered() {
        for name in ["magicx-toolbox.exe", "MagicX-Toolbox.EXE"] {
            assert!(is_portable_asset(name), "{name} rejected");
        }
        for name in [
            "",
            "magicx-toolbox",
            "magicx-toolbox.exe.sha256",
            "magicx-toolbox.exe ",
            "magicx-toolbox.msi",
            "magicx_toolbox.exe",
            "MagicX-Toolbox_3.1.0_x64-setup.exe",
            "MagicX-Toolbox_3.1.0_x64_en-US.msi",
            "old-magicx-toolbox.exe",
            r"..\magicx-toolbox.exe",
            "sub/magicx-toolbox.exe",
            "magicx-toolbox.exe:stream",
        ] {
            assert!(!is_portable_asset(name), "{name:?} accepted");
        }
        let assets = [
            asset("MagicX-Toolbox_3.1.0_x64-setup.exe"),
            asset("Magicx-Toolbox.exe"),
        ];
        assert_eq!(
            portable_asset(&assets).map(|a| a.name.as_str()),
            Some("Magicx-Toolbox.exe")
        );
        assert!(portable_asset(&assets[..1]).is_none());
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

    /// A temp folder holding `magicx-toolbox.exe` ("old") and its staged update ("new").
    fn staged_update() -> (tempfile::TempDir, UpdatePaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = UpdatePaths::beside(dir.path().join(PORTABLE_ASSET));
        std::fs::write(&paths.exe, b"old").unwrap();
        std::fs::write(&paths.staged, b"new").unwrap();
        (dir, paths)
    }

    #[test]
    fn siblings_keep_the_exe_name() {
        let paths = UpdatePaths::beside(PathBuf::from(r"D:\Tools\MagicX.exe"));
        assert_eq!(paths.staged, PathBuf::from(r"D:\Tools\MagicX.exe.new"));
        assert_eq!(paths.old, PathBuf::from(r"D:\Tools\MagicX.exe.old"));
    }

    #[test]
    fn a_swap_puts_the_update_in_place_and_keeps_the_old_exe_aside() {
        let (_dir, paths) = staged_update();
        let mut launched = None;
        replace_and_launch(&paths, |exe| {
            launched = Some(std::fs::read(exe)?);
            Ok(())
        })
        .unwrap();
        assert_eq!(launched.as_deref(), Some(&b"new"[..]));
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"new");
        assert_eq!(std::fs::read(&paths.old).unwrap(), b"old");
        assert!(!paths.staged.exists());
    }

    #[test]
    fn a_failed_launch_puts_the_previous_exe_back() {
        let (_dir, paths) = staged_update();
        let result = replace_and_launch(&paths, |_| Err(std::io::Error::other("no")));
        assert!(matches!(result, Err(Error::Update(_))), "got {result:?}");
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"old");
        assert!(!paths.old.exists());
        assert!(!paths.staged.exists());
    }

    #[test]
    fn a_failed_move_into_place_puts_the_previous_exe_back() {
        let (_dir, paths) = staged_update();
        std::fs::remove_file(&paths.staged).unwrap();
        let result = replace_and_launch(&paths, |_| panic!("nothing to launch"));
        assert!(matches!(result, Err(Error::Update(_))), "got {result:?}");
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"old");
        assert!(!paths.old.exists());
    }

    #[test]
    fn a_running_exe_can_be_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let paths = UpdatePaths::beside(dir.path().join("cmd.exe"));
        let system = crate::services::system32::system_dir().unwrap();
        std::fs::copy(system.join("cmd.exe"), &paths.exe).unwrap();
        std::fs::write(&paths.staged, b"new").unwrap();
        let mut running = Command::new(&paths.exe)
            .args(["/d", "/c", "ping", "-n", "5", "127.0.0.1"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let swapped = replace_and_launch(&paths, |_| Ok(()));
        running.kill().unwrap();
        running.wait().unwrap();
        swapped.unwrap();
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"new");
        assert!(paths.old.exists());
    }

    #[test]
    fn leftovers_are_removed_and_nothing_else() {
        let (dir, paths) = staged_update();
        std::fs::write(&paths.old, b"older").unwrap();
        let unrelated = dir.path().join("snapshots.json");
        std::fs::write(&unrelated, b"keep").unwrap();
        remove_leftovers(&paths);
        assert!(!paths.old.exists());
        assert!(!paths.staged.exists());
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"old");
        assert!(unrelated.exists());
        remove_leftovers(&paths);
    }

    #[test]
    fn a_stale_previous_exe_is_replaced_by_the_swap() {
        let (_dir, paths) = staged_update();
        std::fs::write(&paths.old, b"older").unwrap();
        replace_and_launch(&paths, |_| Ok(())).unwrap();
        assert_eq!(std::fs::read(&paths.old).unwrap(), b"old");
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"new");
    }

    #[test]
    fn a_previous_exe_that_cannot_be_removed_is_named_and_the_download_discarded() {
        let (_dir, paths) = staged_update();
        std::fs::create_dir(&paths.old).unwrap();
        std::fs::write(paths.old.join("held"), b"x").unwrap();
        let result = replace_and_launch(&paths, |_| panic!("nothing to launch"));
        assert!(
            matches!(&result, Err(Error::Update(m)) if m.contains("magicx-toolbox.exe.old")),
            "got {result:?}"
        );
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"old");
        assert!(!paths.staged.exists());
    }

    #[test]
    fn a_folder_the_app_cannot_write_is_its_own_error() {
        const ERROR_ACCESS_DENIED: i32 = 5;
        for code in [ERROR_ACCESS_DENIED, ERROR_WRITE_PROTECT] {
            let e = std::io::Error::from_raw_os_error(code);
            assert!(
                matches!(fs_error("x", &e), Error::UpdateFolderReadOnly),
                "{code}"
            );
        }
        let full = std::io::Error::from_raw_os_error(112);
        assert!(matches!(fs_error("x", &full), Error::Update(_)));
    }

    #[test]
    fn an_unwritable_folder_fails_before_the_download() {
        let gate = crate::tweaks::engine::lifecycle::ApplyGate::default();
        let dir = tempfile::tempdir().unwrap();
        let paths = UpdatePaths::beside(dir.path().join("missing").join(PORTABLE_ASSET));
        let result = install_update_in(
            &gate,
            &paths,
            download(
                &format!("https://github.com/{RELEASE_REPO}/releases/download/v1/{PORTABLE_ASSET}"),
                PORTABLE_ASSET,
                Some("sha256:00"),
            ),
            |_| panic!("no download may start"),
            || {},
            || {},
        );
        assert!(result.is_err(), "got {result:?}");
    }

    fn download(url: &str, name: &str, digest: Option<&str>) -> Download {
        Download {
            url: url.into(),
            asset_name: name.into(),
            digest: digest.map(str::to_string),
            size: None,
        }
    }

    #[tokio::test]
    async fn rejected_install_leaves_the_latch_released() {
        let gate = crate::tweaks::engine::lifecycle::ApplyGate::default();
        let (_dir, paths) = staged_update();
        let base = format!("https://github.com/{RELEASE_REPO}/releases/download/v1");
        let ok = format!("{base}/{PORTABLE_ASSET}");
        let setup = format!("{base}/MagicX-Toolbox_3.1.0_x64-setup.exe");
        for (url, name, digest) in [
            (
                "https://example.com/magicx-toolbox.exe",
                PORTABLE_ASSET,
                Some("sha256:00"),
            ),
            (ok.as_str(), r"..\magicx-toolbox.exe", Some("sha256:00")),
            (
                setup.as_str(),
                "MagicX-Toolbox_3.1.0_x64-setup.exe",
                Some("sha256:00"),
            ),
            (ok.as_str(), PORTABLE_ASSET, None),
        ] {
            let result = install_update_in(
                &gate,
                &paths,
                download(url, name, digest),
                |_| {},
                || {},
                || panic!("a rejected update never exits"),
            );
            assert!(matches!(result, Err(Error::Update(_))), "got {result:?}");
        }
        assert_eq!(std::fs::read(&paths.exe).unwrap(), b"old");
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
            &UpdatePaths::beside(PathBuf::from(PORTABLE_ASSET)),
            download("https://example.com/x.exe", "x.exe", None),
            |_| {},
            || {},
            || {},
        );
        assert!(
            matches!(result, Err(Error::ApplyInFlight(_))),
            "got {result:?}"
        );
    }

    #[test]
    fn percent_is_known_only_with_a_size() {
        let at = |downloaded, total| DownloadProgress { downloaded, total }.percent();
        assert_eq!(at(50, Some(200)), Some(25));
        assert_eq!(at(300, Some(200)), Some(100));
        assert_eq!(at(199, Some(200)), Some(99), "floored, never rounded up");
        assert_eq!(at(u64::MAX, Some(u64::MAX)), Some(100));
        assert_eq!(at(10, Some(0)), None);
        assert_eq!(at(10, None), None);
    }

    #[test]
    fn the_meter_reports_each_whole_percent_once() {
        let mut meter = ProgressMeter::new(Some(1000));
        assert_eq!(
            meter.advance(0),
            Some(DownloadProgress {
                downloaded: 0,
                total: Some(1000)
            })
        );
        assert_eq!(meter.advance(5), None);
        assert_eq!(meter.advance(5).map(|p| p.downloaded), Some(10));
        assert_eq!(meter.advance(990).and_then(|p| p.percent()), Some(100));
        assert_eq!(meter.advance(0), None);
    }

    #[test]
    fn without_a_size_the_meter_reports_each_step() {
        let mut meter = ProgressMeter::new(None);
        assert!(meter.advance(0).is_some());
        assert_eq!(meter.advance(UNKNOWN_SIZE_STEP - 1), None);
        assert_eq!(
            meter.advance(1).map(|p| p.downloaded),
            Some(UNKNOWN_SIZE_STEP)
        );
    }
}
