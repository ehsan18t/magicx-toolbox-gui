//! Session files: one per process run, `magicx-YYYYMMDD-HHMMSS-<pid>.log`, size-capped and
//! retention-pruned. Runs inside the logger: nothing here may call `log::`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, TimeDelta};
use log::Level;

pub const SOFT_CAP: u64 = 2 * 1024 * 1024;
pub const HARD_CAP: u64 = 4 * 1024 * 1024;
pub const KEEP_FILES: usize = 10;
pub const KEEP_BYTES: u64 = 10 * 1024 * 1024;
const SOFT_MARKER: &str =
    "----- this file passed 2 MiB: only warnings and errors are recorded from here -----\n";
const HARD_MARKER: &str = "----- this file reached 4 MiB: nothing more is recorded in it -----\n";

#[derive(Default)]
pub struct FileSink {
    out: Option<Box<dyn Write + Send>>,
    pub(super) attached_through: u64,
    written: u64,
    soft: bool,
    pub(super) error: Option<String>,
}

impl FileSink {
    /// Creates a new session file in `dir` and writes `header`; prunes old files once it exists.
    pub fn open(&mut self, dir: &Path, header: &str) -> Result<(), String> {
        self.close();
        let (file, path) = create_session_file(dir, Local::now(), std::process::id())
            .map_err(|e| self.fail(format!("the log file could not be created ({e})")))?;
        retain(dir, Some(&path), &pid_is_live);
        self.start(Box::new(file));
        match self.raw(header.as_bytes()) {
            Some(failure) => Err(failure),
            None => Ok(()),
        }
    }

    pub(super) fn start(&mut self, out: Box<dyn Write + Send>) {
        self.out = Some(out);
        self.written = 0;
        self.soft = false;
        self.error = None;
    }

    pub fn close(&mut self) {
        self.out = None;
    }

    pub fn writing(&self) -> bool {
        self.out.is_some()
    }

    /// `Some` once, on the write that broke the sink: the line for the ring.
    pub fn write(&mut self, seq: u64, level: Level, line: &str) -> Option<String> {
        if seq <= self.attached_through {
            return None;
        }
        self.append(level, line)
    }

    pub fn append(&mut self, level: Level, line: &str) -> Option<String> {
        self.out.as_ref()?;
        let len = line.len() as u64;
        if !self.soft && self.written + len > SOFT_CAP {
            self.soft = true;
            if let Some(failure) = self.raw(SOFT_MARKER.as_bytes()) {
                return Some(failure);
            }
        }
        if self.soft && level > Level::Warn {
            return None;
        }
        if self.written + len > HARD_CAP {
            let failure = self.raw(HARD_MARKER.as_bytes());
            self.out = None;
            return failure;
        }
        self.raw(line.as_bytes())
    }

    fn raw(&mut self, bytes: &[u8]) -> Option<String> {
        let result = self.out.as_mut()?.write_all(bytes);
        match result {
            Ok(()) => {
                self.written += bytes.len() as u64;
                None
            }
            Err(e) => Some(self.fail(format!("the log file could not be written ({e})"))),
        }
    }

    fn fail(&mut self, why: String) -> String {
        self.out = None;
        let message = format!(
            "{why}, so logs are kept in memory only until saving is turned off and on again or the app restarts"
        );
        self.error = Some(message.clone());
        message
    }
}

/// `write(true)`, not append: the offset is the sink's own, kept under its mutex.
fn create_session_file(dir: &Path, now: DateTime<Local>, pid: u32) -> io::Result<(File, PathBuf)> {
    prepare_dir(dir)?;
    let mut at = now;
    for _ in 0..60 {
        let path = dir.join(session_name(at, pid));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => at += TimeDelta::seconds(1),
            Err(e) => return Err(e),
        }
    }
    Err(io::ErrorKind::AlreadyExists.into())
}

pub fn session_name(at: DateTime<Local>, pid: u32) -> String {
    format!("magicx-{}-{pid}.log", at.format("%Y%m%d-%H%M%S"))
}

pub fn prepare_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    checked(dir)
}

/// A junction or symlink could point the writes, or the deletes, at another folder.
pub fn checked(dir: &Path) -> io::Result<()> {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
    if fs::symlink_metadata(dir)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(io::Error::other("the logs folder is a link or junction"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFile {
    pub path: PathBuf,
    pub stamp: String,
    pub pid: u32,
    pub bytes: u64,
}

/// `(stamp, pid)` for an exact session-file name, `None` for anything else.
fn parse_session_name(name: &str) -> Option<(&str, u32)> {
    let rest = name.strip_prefix("magicx-")?.strip_suffix(".log")?;
    let (stamp, pid) = rest.rsplit_once('-')?;
    let (date, time) = stamp.split_once('-')?;
    let digits = |s: &str, len: Option<usize>| {
        len.is_none_or(|n| s.len() == n) && !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
    };
    if !(digits(date, Some(8)) && digits(time, Some(6)) && digits(pid, None)) {
        return None;
    }
    Some((stamp, pid.parse().ok()?))
}

/// Oldest first.
pub fn session_files(dir: &Path) -> Vec<SessionFile> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<SessionFile> = entries
        .flatten()
        .filter_map(|entry| {
            let meta = entry.metadata().ok().filter(|m| m.is_file())?;
            let name = entry.file_name();
            let (stamp, pid) = parse_session_name(name.to_str()?)?;
            Some(SessionFile {
                path: entry.path(),
                stamp: stamp.to_string(),
                pid,
                bytes: meta.len(),
            })
        })
        .collect();
    files.sort_by(|a, b| (&a.stamp, a.pid).cmp(&(&b.stamp, b.pid)));
    files
}

/// Keeps the newest [`KEEP_FILES`] within [`KEEP_BYTES`]; never deletes `current` or a file whose
/// process is still running.
pub fn retain(dir: &Path, current: Option<&Path>, is_live: &dyn Fn(u32) -> bool) {
    let (mut count, mut total) = (0, 0);
    for file in session_files(dir).iter().rev() {
        count += 1;
        total += file.bytes;
        let over = count > KEEP_FILES || total > KEEP_BYTES;
        if over && Some(file.path.as_path()) != current && !is_live(file.pid) {
            let _ = fs::remove_file(&file.path);
        }
    }
}

/// The files tauri-plugin-log wrote into this same folder.
pub fn remove_plugin_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let plugin = name == "magicx-toolbox.log"
            || (name.starts_with("magicx-toolbox_") && name.ends_with(".log"))
            || (name.starts_with("magicx-toolbox") && name.ends_with(".log.bak"));
        if plugin && entry.file_type().is_ok_and(|t| t.is_file()) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// A pid this token cannot open still runs. A reused pid also reads as live: its file is kept.
pub fn pid_is_live(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, STILL_ACTIVE,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: the handle is checked for null and closed exactly once; `code` is a live local.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return GetLastError() == ERROR_ACCESS_DENIED;
        }
        let mut code = 0u32;
        let read = GetExitCodeProcess(process, &mut code) != 0;
        CloseHandle(process);
        !read || code == STILL_ACTIVE as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(dir: &Path, name: &str, bytes: usize) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, vec![b'x'; bytes]).unwrap();
        path
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }

    fn session(n: u32, pid: u32) -> String {
        format!("magicx-20260101-1200{n:02}-{pid}.log")
    }

    #[test]
    fn only_exact_session_names_parse() {
        assert_eq!(
            parse_session_name("magicx-20261002-140305-4242.log"),
            Some(("20261002-140305", 4242))
        );
        for name in [
            "magicx-20261002-140305.log",
            "magicx-2026102-140305-1.log",
            "magicx-20261002-140305-1.log.bak",
            "magicx-20261002-140305-x1.log",
            "magicx-toolbox.log",
            "notes.txt",
        ] {
            assert_eq!(parse_session_name(name), None, "{name}");
        }
    }

    #[test]
    fn retention_keeps_the_newest_ten() {
        let dir = tempfile::tempdir().unwrap();
        for n in 0..13 {
            touch(dir.path(), &session(n, 100 + n), 10);
        }
        touch(dir.path(), "notes.txt", 10);
        touch(dir.path(), "magicx-old.log", 10);
        retain(dir.path(), None, &|_| false);
        let left = names(dir.path());
        assert_eq!(left.len(), 12);
        assert!(left.contains(&"notes.txt".to_string()));
        assert!(left.contains(&"magicx-old.log".to_string()));
        for n in 0..3 {
            assert!(!left.contains(&session(n, 100 + n)), "{n}");
        }
    }

    #[test]
    fn retention_caps_the_total_size_and_spares_live_and_current_files() {
        let dir = tempfile::tempdir().unwrap();
        let mib = 1024 * 1024;
        let current = touch(dir.path(), &session(9, 1), 4 * mib);
        touch(dir.path(), &session(8, 2), 4 * mib);
        touch(dir.path(), &session(7, 3), 4 * mib);
        touch(dir.path(), &session(6, 4), 4 * mib);
        touch(dir.path(), &session(5, 5), mib);
        retain(dir.path(), Some(&current), &|pid| pid == 4);
        assert_eq!(
            names(dir.path()),
            vec![session(6, 4), session(8, 2), session(9, 1)]
        );
    }

    #[test]
    fn the_old_plugin_files_are_removed_by_exact_name() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "magicx-toolbox.log",
            "magicx-toolbox_2026-01-01_12-00-00.log",
            "magicx-toolbox.log.bak",
            "magicx-toolbox_1.log.bak",
            "magicx-toolbox.txt",
            "other.log",
            &session(1, 1),
        ] {
            touch(dir.path(), name, 1);
        }
        remove_plugin_files(dir.path());
        assert_eq!(
            names(dir.path()),
            vec![
                session(1, 1),
                "magicx-toolbox.txt".into(),
                "other.log".into()
            ]
        );
    }

    #[test]
    fn this_process_is_live_and_an_unused_pid_is_not() {
        assert!(pid_is_live(std::process::id()));
        assert!(!pid_is_live(0xFFFF_FFF0));
    }

    #[test]
    fn a_second_file_in_the_same_second_takes_the_next_name() {
        let dir = tempfile::tempdir().unwrap();
        let now = Local::now();
        let (_, first) = create_session_file(dir.path(), now, 7).unwrap();
        let (_, second) = create_session_file(dir.path(), now, 7).unwrap();
        assert_ne!(first, second);
        assert_eq!(session_files(dir.path()).len(), 2);
    }

    #[test]
    fn the_file_takes_only_warnings_after_the_soft_cap_and_stops_at_the_hard_cap() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cap.log");
        let mut sink = FileSink::default();
        sink.start(Box::new(File::create(&path).unwrap()));
        let line = format!("{}\n", "a".repeat(1023));
        for _ in 0..2048 {
            assert_eq!(sink.append(Level::Info, &line), None);
        }
        assert_eq!(sink.append(Level::Info, "info past soft\n"), None);
        assert_eq!(sink.append(Level::Warn, "warn past soft\n"), None);
        while sink.writing() {
            sink.append(Level::Error, &line);
        }
        assert_eq!(sink.append(Level::Error, "after hard\n"), None);
        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("info past soft"));
        assert!(text.contains("warn past soft"));
        assert!(!text.contains("after hard"));
        assert_eq!(text.matches(SOFT_MARKER).count(), 1);
        assert!(text.ends_with(HARD_MARKER));
        assert!(text.len() as u64 <= HARD_CAP + HARD_MARKER.len() as u64);
    }
}
