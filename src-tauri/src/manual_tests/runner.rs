use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::{ManualTest, TESTS};
use crate::commands::tweaks::{
    apply_gated, get_tweak_status, restore_gated, ApplyOutcomeView, RestoreOutcomeView,
    TweakEngineState, TweakStatusEvent,
};
use crate::error::{Error, Result};
use crate::services::elevation::Elevation;
use crate::services::system_info_service;
use crate::tweaks::engine::apply::EngineError;
use crate::tweaks::model::Tweak;
use crate::tweaks::winver::running_winver;

static RUNNING: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);
/// `Some` only while a test has armed it, so a normal apply from the UI records nothing.
static BATCHES: Mutex<Option<Vec<BatchRecord>>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchRecord {
    pub level: Elevation,
    pub ops: usize,
    pub elapsed_ms: u128,
    pub ok: bool,
}

/// Called from the broker parent after every elevated batch, whichever tweak sent it.
pub fn record_batch(level: Elevation, ops: usize, elapsed_ms: u128, ok: bool) {
    let mut batches = BATCHES.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(list) = batches.as_mut() {
        list.push(BatchRecord {
            level,
            ops,
            elapsed_ms,
            ok,
        });
    }
}

pub(super) fn arm_batches() {
    *BATCHES.lock().unwrap_or_else(|p| p.into_inner()) = Some(Vec::new());
}

pub(super) fn take_batches() -> Vec<BatchRecord> {
    BATCHES
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .take()
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Verdict {
    pub status: Status,
    pub summary: String,
    pub details: Vec<String>,
}

impl Verdict {
    pub fn pass(summary: impl Into<String>) -> Self {
        Self::new(Status::Pass, summary)
    }
    pub fn fail(summary: impl Into<String>) -> Self {
        Self::new(Status::Fail, summary)
    }
    pub fn info(summary: impl Into<String>) -> Self {
        Self::new(Status::Info, summary)
    }
    fn new(status: Status, summary: impl Into<String>) -> Self {
        Self {
            status,
            summary: summary.into(),
            details: Vec::new(),
        }
    }
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ManualTestReport {
    pub test_id: &'static str,
    pub status: Status,
    pub summary: String,
    pub details: Vec<String>,
    /// The whole plain-text report the Copy button hands out.
    pub report: String,
}

#[derive(Clone, Serialize)]
struct LogEvent<'a> {
    test_id: &'a str,
    line: &'a str,
}

type Typed<T> = Result<std::result::Result<T, EngineError>>;

/// The app side of a run. Tests reach it only through this trait: naming `AppHandle` would link
/// the webview runtime into the unit-test binary, whose comctl32 v6 imports fail without a manifest.
pub trait Host {
    fn engine(&self) -> &TweakEngineState;
    fn apply(&self, tweak: &'static Tweak, option: &str) -> Typed<ApplyOutcomeView>;
    fn restore(&self, tweak: &'static Tweak) -> Typed<RestoreOutcomeView>;
    fn emit_line(&self, test_id: &str, line: &str);
}

impl Host for AppHandle {
    fn engine(&self) -> &TweakEngineState {
        tauri::Manager::state::<TweakEngineState>(self).inner()
    }

    fn apply(&self, tweak: &'static Tweak, option: &str) -> Typed<ApplyOutcomeView> {
        let outcome =
            tauri::async_runtime::block_on(apply_gated(self.clone(), tweak, option.to_string()));
        publish_status(self, tweak);
        outcome
    }

    fn restore(&self, tweak: &'static Tweak) -> Typed<RestoreOutcomeView> {
        let outcome = tauri::async_runtime::block_on(restore_gated(self.clone(), tweak));
        publish_status(self, tweak);
        outcome
    }

    fn emit_line(&self, test_id: &str, line: &str) {
        if let Err(e) = self.emit("manual-test-log", LogEvent { test_id, line }) {
            log::warn!("[manual-test {test_id}] could not stream a log line: {e}");
        }
    }
}

/// The tweak card updates only from a command's reply or a `tweak-status` event.
fn publish_status(app: &AppHandle, tweak: &Tweak) {
    let read = tauri::async_runtime::block_on(get_tweak_status(app.clone(), tweak.id.clone()));
    let emitted = read.and_then(|status| {
        let event = TweakStatusEvent {
            tweak_id: tweak.id.clone(),
            status,
        };
        Ok(app.emit("tweak-status", event)?)
    });
    if let Err(e) = emitted {
        log::warn!(
            "[manual-test] could not refresh the '{}' card: {e}",
            tweak.id
        );
    }
}

/// One running test: every line goes to the `log` crate, the view (`manual-test-log`) and the report.
pub struct Ctx {
    pub host: Box<dyn Host>,
    pub test_id: &'static str,
    pub minutes: u32,
    started: Instant,
    lines: RefCell<Vec<String>>,
    scrub: Vec<String>,
}

impl Ctx {
    pub fn info(&self, msg: impl AsRef<str>) {
        self.push(false, msg.as_ref());
    }

    pub fn error(&self, msg: impl AsRef<str>) {
        self.push(true, msg.as_ref());
    }

    fn push(&self, is_error: bool, msg: &str) {
        let msg = scrub(msg, &self.scrub);
        if is_error {
            log::error!("[manual-test {}] {msg}", self.test_id);
        } else {
            log::info!("[manual-test {}] {msg}", self.test_id);
        }
        let line = format!(
            "{} +{}ms {}{msg}",
            chrono::Local::now().format("%H:%M:%S%.3f"),
            self.started.elapsed().as_millis(),
            if is_error { "ERROR " } else { "" }
        );
        self.host.emit_line(self.test_id, &line);
        self.lines.borrow_mut().push(line);
    }

    pub fn cancelled(&self) -> bool {
        CANCEL.load(Ordering::SeqCst)
    }

    /// `false` when cancelled before `dur` ran out.
    pub fn sleep(&self, dur: Duration) -> bool {
        let until = Instant::now() + dur;
        while Instant::now() < until {
            if self.cancelled() {
                return false;
            }
            std::thread::sleep(
                Duration::from_millis(200).min(until.saturating_duration_since(Instant::now())),
            );
        }
        !self.cancelled()
    }
}

/// Strings a report must never carry: the account name and profile/temp folders.
fn scrub_list() -> Vec<String> {
    let mut list: Vec<String> = ["USERPROFILE", "TEMP", "TMP", "LOCALAPPDATA", "APPDATA"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .collect();
    if let Ok(user) = std::env::var("USERNAME") {
        if user.len() >= 3 {
            list.push(user);
        }
    }
    list.sort_by_key(|s| std::cmp::Reverse(s.len()));
    list
}

fn scrub(msg: &str, list: &[String]) -> String {
    let mut out = msg.to_string();
    for s in list {
        if !s.is_empty() {
            out = replace_ignore_case(&out, s, "<redacted>");
        }
    }
    out
}

/// A match followed by a path separator takes the rest of that path token with it. ASCII-only
/// lowercasing keeps every offset a char boundary: this runs between apply and restore, where a
/// panic aborts the release build and leaves the tweak applied.
fn replace_ignore_case(hay: &str, needle: &str, with: &str) -> String {
    let lower_hay = hay.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(hay.len());
    let mut rest = 0;
    for (at, _) in lower_hay.match_indices(&lower_needle) {
        if at < rest {
            continue;
        }
        out.push_str(hay.get(rest..at).unwrap_or_default());
        out.push_str(with);
        let mut end = at + needle.len();
        let tail = hay.get(end..).unwrap_or_default();
        if tail.starts_with(['\\', '/']) {
            end += tail
                .find(|c: char| c.is_whitespace() || "\"'(),;".contains(c))
                .unwrap_or(tail.len());
        }
        rest = end;
    }
    out.push_str(hay.get(rest..).unwrap_or_default());
    out
}

/// Machine and process architecture: an x64 build under ARM64 emulation reports both.
fn machine_arch() -> String {
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

struct RunGuard;

impl RunGuard {
    fn acquire() -> Result<Self> {
        if RUNNING.swap(true, Ordering::SeqCst) {
            return Err(Error::ValidationError(
                "another manual test is still running".into(),
            ));
        }
        Ok(RunGuard)
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        RUNNING.store(false, Ordering::SeqCst);
    }
}

pub fn cancel() {
    CANCEL.store(true, Ordering::SeqCst);
}

pub async fn run(
    app: AppHandle,
    test_id: String,
    minutes: Option<u32>,
) -> Result<ManualTestReport> {
    let test = TESTS
        .iter()
        .find(|t| t.id == test_id)
        .ok_or_else(|| Error::NotFound(format!("manual test '{test_id}'")))?;
    let guard = RunGuard::acquire()?;
    CANCEL.store(false, Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        execute(app, test, minutes)
    })
    .await
    .map_err(Error::from)
}

fn execute(app: AppHandle, test: &'static ManualTest, minutes: Option<u32>) -> ManualTestReport {
    let minutes = minutes.or(test.minutes).unwrap_or(0).clamp(1, 24 * 60);
    let winver = running_winver();
    let header = [
        format!("App version: {} (test build)", app.package_info().version),
        format!("Windows build: {}.{}", winver.build, winver.revision),
        format!(
            "Elevated: {}",
            if system_info_service::is_running_as_admin() {
                "yes"
            } else {
                "no"
            }
        ),
        format!("Architecture: {}", machine_arch()),
        format!("Test: {} ({})", test.id, test.title),
    ];
    let cx = Ctx {
        host: Box::new(app),
        test_id: test.id,
        minutes,
        started: Instant::now(),
        lines: RefCell::new(Vec::new()),
        scrub: scrub_list(),
    };
    let started_at = chrono::Local::now();
    for line in &header {
        cx.info(line);
    }
    cx.info(format!("started {}", started_at.to_rfc3339()));
    let verdict = (test.run)(&cx);
    let finished_at = chrono::Local::now();
    let verdict_line = format!(
        "result {:?}: {} ({} ms)",
        verdict.status,
        verdict.summary,
        cx.started.elapsed().as_millis()
    );
    if verdict.status == Status::Fail {
        cx.error(&verdict_line);
    } else {
        cx.info(&verdict_line);
    }
    for d in &verdict.details {
        cx.info(format!("  {d}"));
    }
    let mut report = String::from("MagicX Toolbox manual test report\n");
    for line in &header {
        report.push_str(line);
        report.push('\n');
    }
    report.push_str(&format!("Started: {}\n", started_at.to_rfc3339()));
    report.push_str(&format!("Finished: {}\n", finished_at.to_rfc3339()));
    report.push_str(&format!(
        "Result: {:?}: {}\n",
        verdict.status, verdict.summary
    ));
    for d in &verdict.details {
        report.push_str(&format!("  {d}\n"));
    }
    report.push_str("\nLog:\n");
    for line in cx.lines.borrow().iter() {
        report.push_str(line);
        report.push('\n');
    }
    ManualTestReport {
        test_id: test.id,
        status: verdict.status,
        summary: scrub(&verdict.summary, &cx.scrub),
        details: verdict
            .details
            .iter()
            .map(|d| scrub(d, &cx.scrub))
            .collect(),
        report: scrub(&report, &cx.scrub),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrubbing_is_case_insensitive_and_takes_the_whole_path() {
        let list = vec![r"C:\Users\Alice".to_string(), "Alice".to_string()];
        assert_eq!(
            scrub(r"read c:\users\alice\x and ALICE", &list),
            "read <redacted> and <redacted>"
        );
        assert_eq!(
            scrub(
                r"spawn C:\Users\Alice\Temp\req-9f3a.json (os error 5)",
                &list
            ),
            "spawn <redacted> (os error 5)"
        );
        assert_eq!(scrub("é Alice ü", &list), "é <redacted> ü");
    }

    #[test]
    fn an_unarmed_recorder_keeps_nothing() {
        let _ = take_batches();
        record_batch(Elevation::TrustedInstaller, 3, 10, true);
        assert!(take_batches().is_empty());
        arm_batches();
        record_batch(Elevation::TrustedInstaller, 3, 10, true);
        assert_eq!(
            take_batches(),
            vec![BatchRecord {
                level: Elevation::TrustedInstaller,
                ops: 3,
                elapsed_ms: 10,
                ok: true
            }]
        );
    }
}
