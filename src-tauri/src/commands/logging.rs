//! The Logs panel and Settings > Diagnostics. `get_log_tail` and `log_frontend` do not log at entry:
//! the panel polls the first, and the second is itself a log line.

use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::commands::tweaks::{blocking, TweakEngineState};
use crate::error::{Error, Result};
use crate::logging::{self, pipeline::Entry, pipeline::Source, yes_no, Status};

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct LogLineView {
    pub seq: u64,
    pub ts: String,
    pub level: LogLevel,
    pub source: Source,
    pub target: String,
    pub msg: String,
}

impl From<Entry> for LogLineView {
    fn from(e: Entry) -> Self {
        Self {
            seq: e.seq,
            ts: e.ts,
            level: match e.level {
                log::Level::Error => LogLevel::Error,
                log::Level::Warn => LogLevel::Warn,
                log::Level::Info => LogLevel::Info,
                log::Level::Debug => LogLevel::Debug,
                log::Level::Trace => LogLevel::Trace,
            },
            source: e.source,
            target: e.target,
            msg: e.msg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct LogTail {
    pub lines: Vec<LogLineView>,
    /// Lines after `since` that left the in-memory buffer before they could be read.
    pub skipped: u64,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct LogSettingsView {
    pub persist: bool,
    pub detailed: bool,
    pub folder: String,
    pub writing: bool,
    pub error: Option<String>,
    pub files: u32,
    pub bytes: u64,
}

impl From<Status> for LogSettingsView {
    fn from(s: Status) -> Self {
        Self {
            persist: s.persist,
            detailed: s.detailed,
            folder: s
                .folder
                .map(|f| f.display().to_string())
                .unwrap_or_default(),
            writing: s.writing,
            error: s.error,
            files: s.files,
            bytes: s.bytes,
        }
    }
}

#[tauri::command]
pub async fn get_log_tail(since: u64) -> Result<LogTail> {
    let (lines, skipped) = logging::tail(since);
    Ok(LogTail {
        lines: lines.into_iter().map(Into::into).collect(),
        skipped,
    })
}

#[tauri::command]
pub async fn log_frontend(level: String, message: String) -> Result<()> {
    let level = match level.as_str() {
        "error" => log::Level::Error,
        "warn" => log::Level::Warn,
        _ => log::Level::Info,
    };
    logging::push_ui(level, &message);
    Ok(())
}

#[tauri::command]
pub async fn get_log_settings() -> Result<LogSettingsView> {
    log::info!("get_log_settings");
    blocking(|| Ok(logging::status().into())).await
}

#[tauri::command]
pub async fn set_log_settings(persist: bool, detailed: bool) -> Result<LogSettingsView> {
    log::info!("set_log_settings: persist {persist}, detailed {detailed}");
    blocking(move || Ok(logging::set(persist, detailed).into())).await
}

#[tauri::command]
pub async fn delete_logs() -> Result<LogSettingsView> {
    log::info!("delete_logs");
    blocking(|| {
        logging::delete_logs()
            .map(Into::into)
            .map_err(|e| Error::from_io("deleting the log files", &e))
    })
    .await
}

#[tauri::command]
pub async fn open_log_folder(app: AppHandle) -> Result<()> {
    log::info!("open_log_folder");
    blocking(move || {
        let folder = logging::status()
            .folder
            .ok_or_else(|| Error::NotFound("the logs folder".into()))?;
        crate::logging::files::prepare_dir(&folder)
            .map_err(|e| Error::from_io("opening the logs folder", &e))?;
        app.opener()
            .open_path(folder.display().to_string(), None::<&str>)
            .map_err(|e| Error::WindowsApi(format!("opening the logs folder: {e}")))
    })
    .await
}

#[tauri::command]
pub async fn reveal_last_export(app: AppHandle) -> Result<()> {
    log::info!("reveal_last_export");
    blocking(move || {
        let path = logging::last_export()
            .ok_or_else(|| Error::NotFound("an exported diagnostics file".into()))?;
        app.opener()
            .reveal_item_in_dir(path)
            .map_err(|e| Error::WindowsApi(format!("showing the exported file: {e}")))
    })
    .await
}

/// `None` when the save dialog is cancelled.
#[tauri::command]
pub async fn export_diagnostics(app: AppHandle) -> Result<Option<String>> {
    log::info!("export_diagnostics");
    blocking(move || {
        let name = format!(
            "magicx-diagnostics-{}.txt",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );
        let Some(picked) = app
            .dialog()
            .file()
            .set_file_name(name)
            .add_filter("Text", &["txt"])
            .blocking_save_file()
        else {
            log::info!("export_diagnostics: cancelled");
            return Ok(None);
        };
        let path = picked
            .into_path()
            .map_err(|e| Error::ValidationError(format!("the chosen location: {e}")))?;
        logging::export(&path, &diagnostics_header(&app))
            .map_err(|e| Error::from_io("writing the diagnostics file", &e))?;
        log::info!("diagnostics exported to {}", path.display());
        Ok(Some(path.display().to_string()))
    })
    .await
}

/// The one line every apply, restore, keep-current-state, removal and install leaves. Error text is
/// the frontend's own, which never carries a registry or hosts value.
pub(crate) fn log_outcome<T>(
    what: &str,
    started: Instant,
    result: &Result<T>,
    done: impl FnOnce(&T) -> String,
) {
    let (level, line) = outcome_line(what, started.elapsed().as_millis(), result, done);
    log::log!(level, "{line}");
}

fn outcome_line<T>(
    what: &str,
    ms: u128,
    result: &Result<T>,
    done: impl FnOnce(&T) -> String,
) -> (log::Level, String) {
    match result {
        Ok(value) => (
            log::Level::Info,
            format!("{what}: {} in {ms} ms", done(value)),
        ),
        Err(
            e @ (Error::TweakUnavailable(_)
            | Error::AppUnavailable(_)
            | Error::ApplyInFlight(_)
            | Error::AppExiting(_)
            | Error::NotFound(_)),
        ) => (log::Level::Warn, format!("{what} refused: {}", e.code())),
        Err(Error::AppCancelled(_)) => (log::Level::Info, format!("{what}: cancelled in {ms} ms")),
        Err(e) => (
            log::Level::Warn,
            format!("{what} failed: {}: {e}", e.code()),
        ),
    }
}

fn diagnostics_header(app: &AppHandle) -> String {
    use crate::tweaks::engine::context::{sid_check, RealSidProbe};
    let winver = crate::tweaks::winver::running_winver();
    let status = logging::status();
    let mut out = format!(
        "MagicX Toolbox diagnostics\nCreated: {}\nApp version: {}\nWindows build: {}.{} {}\nArchitecture: {}\nElevated: {}\nAccount check: {:?}\nSaving to disk: {}\nDetailed logging: {}\n",
        chrono::Local::now().to_rfc3339(),
        env!("CARGO_PKG_VERSION"),
        winver.build,
        winver.revision,
        logging::edition(),
        logging::machine_arch(),
        yes_no(crate::services::system_info_service::is_running_as_admin()),
        sid_check(&RealSidProbe),
        if status.persist { "on" } else { "off" },
        if status.detailed { "on" } else { "off" },
    );
    if let Some(error) = &status.error {
        out.push_str(&format!("Logging problem: {error}\n"));
    }
    if let Some(state) = app.try_state::<TweakEngineState>() {
        for line in state.diagnostics_lines() {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::engine::lifecycle::AppExiting;

    #[test]
    fn an_outcome_names_its_state_a_refusal_its_code_and_a_failure_its_message() {
        let ok: Result<&str> = Ok("absent");
        assert_eq!(
            outcome_line("remove 'app'", 12, &ok, |v| v.to_string()),
            (log::Level::Info, "remove 'app': absent in 12 ms".into())
        );
        for refused in [
            Error::TweakUnavailable("needs admin".into()),
            Error::AppUnavailable("needs admin".into()),
            Error::ApplyInFlight("retry"),
            Error::AppExiting(AppExiting::Final),
            Error::NotFound("tweak 'x'".into()),
        ] {
            let code = refused.code();
            let (level, line) = outcome_line(
                "apply 'x' -> 'On'",
                1,
                &Err::<(), _>(refused),
                |_| unreachable!(),
            );
            assert_eq!(level, log::Level::Warn);
            assert_eq!(line, format!("apply 'x' -> 'On' refused: {code}"));
        }
        let cancelled: Result<()> = Err(Error::AppCancelled("Installing X was cancelled.".into()));
        assert_eq!(
            outcome_line("install 'x'", 7, &cancelled, |_| unreachable!()),
            (log::Level::Info, "install 'x': cancelled in 7 ms".into())
        );
        let failed: Result<()> = Err(Error::AppFailed("Removing X failed (exit code 1).".into()));
        assert_eq!(
            outcome_line("remove 'x'", 1, &failed, |_| unreachable!()),
            (
                log::Level::Warn,
                "remove 'x' failed: APP_FAILED: Removing X failed (exit code 1).".into()
            )
        );
    }
}
