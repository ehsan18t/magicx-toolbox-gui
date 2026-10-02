//! On-device logger: every record is redacted before the ring, the Logs panel, the session file or
//! an export sees it. Opting out of saving stops disk writes only; the session ring keeps working.

pub mod collector;
pub mod files;
pub mod panic;
pub mod pipeline;
pub mod redact;
pub mod settings;

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use log::{Level, LevelFilter};

use pipeline::{lock, Entry, Pipeline, Source};
use redact::{Identity, Redactor};
use settings::Settings;

const UI_TARGET: &str = "webview";
const UI_PER_SECOND: u32 = 20;

static PIPELINE: OnceLock<Pipeline> = OnceLock::new();
static CONTROL: Mutex<Control> = Mutex::new(Control {
    data_dir: None,
    persist: false,
    detailed: false,
    error: None,
    facts: String::new(),
    last_export: None,
});
static UI_RATE: Mutex<RateLimit> = Mutex::new(RateLimit {
    window: None,
    count: 0,
    dropped: 0,
});

/// What the settings, the session file and the exports agree on. Lock order: this, then the
/// pipeline's own locks.
struct Control {
    data_dir: Option<PathBuf>,
    persist: bool,
    detailed: bool,
    error: Option<String>,
    facts: String,
    last_export: Option<PathBuf>,
}

impl Control {
    fn logs_dir(&self) -> Option<PathBuf> {
        self.data_dir.as_ref().map(|d| d.join("logs"))
    }

    fn header(&self) -> String {
        format!(
            "{}Detailed logging: {}\n\n",
            self.facts,
            if self.detailed { "on" } else { "off" }
        )
    }
}

struct Global;

impl log::Log for Global {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        PIPELINE
            .get()
            .is_some_and(|p| p.enabled(metadata.level(), metadata.target(), Source::App))
    }

    fn log(&self, record: &log::Record<'_>) {
        if let Some(p) = PIPELINE.get() {
            p.log(record);
        }
    }

    fn flush(&self) {}
}

fn level_for(detailed: bool) -> LevelFilter {
    if detailed || cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

fn set_detailed(p: &Pipeline, detailed: bool) {
    p.set_level(level_for(detailed));
    log::set_max_level(level_for(detailed));
}

/// Whether Debug records are kept: Detailed logging, or any debug build.
pub fn detailed() -> bool {
    PIPELINE
        .get()
        .is_some_and(|p| p.level() >= LevelFilter::Debug)
}

/// Switches Detailed for this session without saving it; returns the previous setting.
#[cfg(feature = "test-build")]
pub fn set_detailed_unsaved(detailed: bool) -> bool {
    let mut c = lock(&CONTROL);
    if let Some(p) = PIPELINE.get() {
        set_detailed(p, detailed);
    }
    std::mem::replace(&mut c.detailed, detailed)
}

/// Memory-only until [`start`]. The identity is read here, before `set_logger`: the reads log on
/// failure, and nothing may be computed lazily inside the logger.
pub fn install() {
    let redactor = Redactor::new(&Identity::from_machine());
    let pipeline = Pipeline::new(redactor, level_for(false), cfg!(debug_assertions));
    if PIPELINE.set(pipeline).is_ok() && log::set_logger(&Global).is_ok() {
        log::set_max_level(level_for(false));
    }
}

/// Reads the settings and attaches the session file. `data_dir` is the app-local data folder.
pub fn start(data_dir: PathBuf) {
    let Some(p) = PIPELINE.get() else { return };
    let loaded = settings::load(&data_dir.join(settings::FILE_NAME));
    let effective = settings::with_overrides(loaded.settings, std::env::args_os().skip(1));
    let logs = data_dir.join("logs");
    let facts = facts(p, &logs);
    let mut c = lock(&CONTROL);
    c.persist = effective.persist;
    c.detailed = effective.detailed;
    c.error = loaded.error;
    c.facts = facts;
    c.data_dir = Some(data_dir);
    set_detailed(p, c.detailed);
    if files::checked(&logs).is_ok() {
        files::remove_plugin_files(&logs);
        if !c.persist {
            files::retain(&logs, None, &files::pid_is_live);
        }
    }
    if c.persist {
        p.attach(&logs, &c.header(), true);
    }
}

fn facts(p: &Pipeline, logs: &Path) -> String {
    let winver = crate::tweaks::winver::running_winver();
    format!(
        "MagicX Toolbox {} session log\nWindows build {}.{} {}, {}\nElevated: {}, pid {}\nLogs folder: {}\n",
        env!("CARGO_PKG_VERSION"),
        winver.build,
        winver.revision,
        edition(),
        machine_arch(),
        yes_no(crate::services::system_info_service::is_running_as_admin()),
        std::process::id(),
        p.redact(&logs.display().to_string()),
    )
}

pub fn yes_no(on: bool) -> &'static str {
    if on {
        "yes"
    } else {
        "no"
    }
}

pub fn edition() -> String {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", KEY_READ)
        .and_then(|key| key.get_value::<String, _>("EditionID"))
        .unwrap_or_else(|_| "unknown edition".into())
}

/// Machine and process architecture: an x64 build under ARM64 emulation reports both.
pub fn machine_arch() -> String {
    use windows_sys::Win32::System::SystemInformation::{
        IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64, IMAGE_FILE_MACHINE_I386,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, IsWow64Process2};
    let (mut process, mut native) = (0u16, 0u16);
    // SAFETY: the pseudo-handle needs no closing; both out-params are live locals.
    let ok = unsafe { IsWow64Process2(GetCurrentProcess(), &mut process, &mut native) } != 0;
    let native = match (ok, native) {
        (false, _) => "unknown".to_string(),
        (true, IMAGE_FILE_MACHINE_AMD64) => "x64".to_string(),
        (true, IMAGE_FILE_MACHINE_ARM64) => "arm64".to_string(),
        (true, IMAGE_FILE_MACHINE_I386) => "x86".to_string(),
        (true, other) => format!("0x{other:04x}"),
    };
    format!("machine {native}, process {}", std::env::consts::ARCH)
}

/// For a source the crate filter must not judge by target: interface and helper lines.
pub fn push(level: Level, source: Source, target: &str, msg: &str) {
    if let Some(p) = PIPELINE.get() {
        p.push(level, source, target, msg);
    }
}

fn record_panic(msg: &str) {
    push(Level::Error, Source::App, "app_lib::panic", msg);
}

struct RateLimit {
    window: Option<Instant>,
    count: u32,
    dropped: u64,
}

impl RateLimit {
    /// Whether to keep this message, and how many the last window dropped, reported once.
    fn admit(&mut self, now: Instant) -> (bool, u64) {
        let mut report = 0;
        if self
            .window
            .is_none_or(|w| now.duration_since(w) >= Duration::from_secs(1))
        {
            report = std::mem::take(&mut self.dropped);
            self.window = Some(now);
            self.count = 0;
        }
        if self.count < UI_PER_SECOND {
            self.count += 1;
            (true, report)
        } else {
            self.dropped += 1;
            (false, report)
        }
    }

    /// What a finished window dropped, once: a burst followed by silence is still reported.
    fn take_expired(&mut self, now: Instant) -> u64 {
        if self
            .window
            .is_some_and(|w| now.duration_since(w) >= Duration::from_secs(1))
        {
            std::mem::take(&mut self.dropped)
        } else {
            0
        }
    }
}

fn report_ui_drops(dropped: u64) {
    if dropped > 0 {
        push(
            Level::Warn,
            Source::Ui,
            UI_TARGET,
            &format!(
                "{dropped} interface messages were dropped (more than {UI_PER_SECOND} a second)"
            ),
        );
    }
}

pub fn push_ui(level: Level, msg: &str) {
    let (admit, dropped) = lock(&UI_RATE).admit(Instant::now());
    report_ui_drops(dropped);
    if admit {
        push(level, Source::Ui, UI_TARGET, msg);
    }
}

fn flush_ui_drops() {
    let dropped = lock(&UI_RATE).take_expired(Instant::now());
    report_ui_drops(dropped);
}

pub fn tail(since: u64) -> (Vec<Entry>, u64) {
    flush_ui_drops();
    PIPELINE.get().map(|p| p.tail(since)).unwrap_or_default()
}

/// The effective state, for the settings view and the export header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub persist: bool,
    pub detailed: bool,
    pub folder: Option<PathBuf>,
    pub writing: bool,
    pub error: Option<String>,
    pub files: u32,
    pub bytes: u64,
}

fn status_of(c: &Control) -> Status {
    let (writing, sink_error) = PIPELINE.get().map(Pipeline::file_state).unwrap_or_default();
    let folder = c.logs_dir();
    let listed = folder
        .as_deref()
        .filter(|d| files::checked(d).is_ok())
        .map(files::session_files)
        .unwrap_or_default();
    let missing = c
        .data_dir
        .is_none()
        .then(|| "The logs folder could not be located, so logs are kept in memory only.".into());
    let error = [c.error.clone(), sink_error, missing]
        .into_iter()
        .flatten()
        .reduce(|a, b| format!("{a} {b}"));
    Status {
        persist: c.persist,
        detailed: c.detailed,
        folder,
        writing,
        error,
        files: u32::try_from(listed.len()).unwrap_or(u32::MAX),
        bytes: listed.iter().map(|f| f.bytes).sum(),
    }
}

pub fn status() -> Status {
    status_of(&lock(&CONTROL))
}

/// Saves the choice and applies it now. Saving on opens a new session file; off closes it.
pub fn set(persist: bool, detailed: bool) -> Status {
    let mut c = lock(&CONTROL);
    let Some(p) = PIPELINE.get() else {
        return status_of(&c);
    };
    c.detailed = detailed;
    set_detailed(p, detailed);
    c.error = c
        .data_dir
        .as_ref()
        .and_then(|d| settings::save(&d.join(settings::FILE_NAME), Settings { persist, detailed }).err())
        .map(|e| format!("The logging settings could not be saved ({e}), so they last only until the app closes."));
    if persist != c.persist {
        c.persist = persist;
        match c.logs_dir() {
            Some(logs) if persist => p.attach(&logs, &c.header(), true),
            _ => p.detach(),
        }
    }
    status_of(&c)
}

/// The settings `restart_as_admin` hands the elevated instance.
pub fn restart_args() -> [String; 2] {
    let c = lock(&CONTROL);
    settings::override_args(Settings {
        persist: c.persist,
        detailed: c.detailed,
    })
}

/// Deletes every session file whose process has ended, this one's included once closed, and
/// starts a fresh file when saving is on.
pub fn delete_logs() -> std::io::Result<Status> {
    let c = lock(&CONTROL);
    let (Some(p), Some(logs)) = (PIPELINE.get(), c.logs_dir()) else {
        return Ok(status_of(&c));
    };
    p.detach();
    let own = std::process::id();
    let mut failed = None;
    if files::checked(&logs).is_ok() {
        for file in files::session_files(&logs) {
            if file.pid != own && files::pid_is_live(file.pid) {
                continue;
            }
            if let Err(e) = std::fs::remove_file(&file.path) {
                failed.get_or_insert(e);
            }
        }
    }
    if c.persist {
        p.attach(&logs, &c.header(), false);
    }
    match failed {
        Some(e) => Err(e),
        None => Ok(status_of(&c)),
    }
}

/// The heading for this session's ring when the files do not hold all of it: saving off, or the
/// file stopped (write failure, hard cap, failed attach).
fn unsaved_heading(persist: bool, writing: bool) -> Option<&'static str> {
    match (persist, writing) {
        (false, _) => Some("this session (not saved to disk)"),
        (true, false) => Some("this session (not fully saved to disk)"),
        (true, true) => None,
    }
}

/// `header`, then every kept session file oldest first, then this session's ring when the files do
/// not hold all of it. The files are already redacted; the header is redacted here. Streamed after
/// CONTROL is released: an export can run to tens of MiB.
pub fn export(dest: &Path, header: &str) -> io::Result<()> {
    flush_ui_drops();
    let Some(p) = PIPELINE.get() else {
        return Err(io::Error::other("the logger is not running"));
    };
    let (sessions, ring) = {
        let c = lock(&CONTROL);
        let sessions = c
            .logs_dir()
            .filter(|d| files::checked(d).is_ok())
            .map(|d| files::session_files(&d))
            .unwrap_or_default();
        let ring =
            unsaved_heading(c.persist, p.file_state().0).map(|heading| (heading, p.ring_text()));
        (sessions, ring)
    };
    write_export(dest, &p.redact(header), &sessions, ring)?;
    lock(&CONTROL).last_export = Some(dest.to_path_buf());
    Ok(())
}

fn write_export(
    dest: &Path,
    header: &str,
    sessions: &[files::SessionFile],
    ring: Option<(&str, String)>,
) -> io::Result<()> {
    let mut out = BufWriter::new(File::create(dest)?);
    out.write_all(header.as_bytes())?;
    for file in sessions {
        let name = file.path.file_name().unwrap_or_default().to_string_lossy();
        writeln!(out, "\n===== {name} =====")?;
        copy_session_file(&file.path, &mut out)?;
    }
    if let Some((heading, text)) = ring {
        writeln!(out, "\n===== {heading} =====")?;
        out.write_all(text.as_bytes())?;
    }
    out.flush()
}

/// A read failure is written into the export; a write failure is the export's own error.
fn copy_session_file(path: &Path, out: &mut impl Write) -> io::Result<()> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return writeln!(out, "(could not be read: {e})"),
    };
    let mut buf = vec![0; 64 * 1024];
    loop {
        match file.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => out.write_all(&buf[..n])?,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return writeln!(out, "(could not be read: {e})"),
        }
    }
}

pub fn last_export() -> Option<PathBuf> {
    lock(&CONTROL).last_export.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_rate_limit_reports_what_it_dropped_once() {
        let mut rate = RateLimit {
            window: None,
            count: 0,
            dropped: 0,
        };
        let t0 = Instant::now();
        for _ in 0..UI_PER_SECOND {
            assert_eq!(rate.admit(t0), (true, 0));
        }
        assert_eq!(rate.admit(t0), (false, 0));
        assert_eq!(rate.admit(t0 + Duration::from_millis(500)), (false, 0));
        let t1 = t0 + Duration::from_secs(1);
        assert_eq!(rate.admit(t1), (true, 2));
        assert_eq!(rate.admit(t1), (true, 0));
    }

    #[test]
    fn a_burst_followed_by_silence_is_still_reported() {
        let mut rate = RateLimit {
            window: None,
            count: 0,
            dropped: 0,
        };
        let t0 = Instant::now();
        for _ in 0..UI_PER_SECOND + 3 {
            rate.admit(t0);
        }
        assert_eq!(rate.take_expired(t0 + Duration::from_millis(500)), 0);
        assert_eq!(rate.take_expired(t0 + Duration::from_secs(1)), 3);
        assert_eq!(rate.take_expired(t0 + Duration::from_secs(2)), 0);
        assert_eq!(rate.admit(t0 + Duration::from_secs(2)), (true, 0));
    }

    #[test]
    fn the_export_carries_the_ring_whenever_the_file_does_not_hold_it_all() {
        assert!(unsaved_heading(false, false).is_some());
        assert_eq!(
            unsaved_heading(true, false),
            Some("this session (not fully saved to disk)")
        );
        assert_eq!(unsaved_heading(true, true), None);
    }

    #[test]
    fn the_streamed_export_has_the_documented_layout() {
        let dir = tempfile::tempdir().unwrap();
        let session = |name: &str, text: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, text).unwrap();
            files::SessionFile {
                path,
                stamp: String::new(),
                pid: 0,
                bytes: 0,
            }
        };
        let mut sessions = vec![
            session("magicx-20260101-120000-1.log", "one\n"),
            session("magicx-20260101-120001-2.log", "two é\n"),
        ];
        sessions.push(files::SessionFile {
            path: dir.path().join("magicx-20260101-120002-3.log"),
            ..sessions[0].clone()
        });
        let dest = dir.path().join("export.txt");
        write_export(
            &dest,
            "header\n",
            &sessions,
            Some(("this session (not saved to disk)", "ring\n".into())),
        )
        .unwrap();
        let text = std::fs::read_to_string(&dest).unwrap();
        let (head, missing) = text.split_once("(could not be read: ").unwrap();
        assert_eq!(
            head,
            "header\n\n===== magicx-20260101-120000-1.log =====\none\n\n===== magicx-20260101-120001-2.log =====\ntwo é\n\n===== magicx-20260101-120002-3.log =====\n"
        );
        assert!(
            missing.ends_with(")\n\n===== this session (not saved to disk) =====\nring\n"),
            "{missing}"
        );
    }
}
