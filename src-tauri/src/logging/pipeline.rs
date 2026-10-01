//! Level check, format, redact, cap, ring, file. The global logger is a thin wrapper over this, so
//! tests drive it without `log::set_logger`. Nothing here may call `log::`.

use std::cell::Cell;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use log::{Level, LevelFilter};

use super::files::FileSink;
use super::redact::{cut_end, Redactor};

pub const RING_CAPACITY: usize = 2000;
pub const RECORD_CAP: usize = 4096;
/// Cut before redaction, so one huge message cannot stall every logging thread; the cut backs off
/// to a separator so no identifier fragment slips past redaction.
const INPUT_CAP: usize = 16 * 1024;
const MARKER_ROOM: usize = 40;

thread_local! {
    pub(super) static IN_LOGGER: Cell<bool> = const { Cell::new(false) };
}

/// A record raised while this thread is already inside the logger is dropped, never recursed into.
pub(super) fn guarded(f: impl FnOnce()) {
    if IN_LOGGER.get() {
        return;
    }
    IN_LOGGER.set(true);
    f();
    IN_LOGGER.set(false);
}

pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    App,
    Ui,
    Helper,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::App => "app",
            Source::Ui => "ui",
            Source::Helper => "helper",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub seq: u64,
    pub ts: String,
    pub level: Level,
    pub source: Source,
    pub target: String,
    pub msg: String,
}

fn prefix(ts: &str, level: Level, source: Source, target: &str) -> String {
    format!(
        "{ts} {:<5} {:<6} {target}: ",
        level.as_str(),
        source.as_str()
    )
}

impl Entry {
    /// Continuation lines are indented with a tab.
    pub fn line(&self) -> String {
        let mut line = prefix(&self.ts, self.level, self.source, &self.target);
        for (n, part) in self.msg.split('\n').enumerate() {
            if n > 0 {
                line.push_str("\n\t");
            }
            line.push_str(part.strip_suffix('\r').unwrap_or(part));
        }
        line.push('\n');
        line
    }
}

fn timestamp() -> String {
    chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3f%:z")
        .to_string()
}

fn floor_boundary(s: &str, max: usize) -> usize {
    if max >= s.len() {
        return s.len();
    }
    let mut at = max;
    while !s.is_char_boundary(at) {
        at -= 1;
    }
    at
}

#[derive(Default)]
pub struct Ring {
    entries: VecDeque<Entry>,
    last: u64,
}

impl Ring {
    fn push(&mut self, mut entry: Entry) -> u64 {
        self.last += 1;
        entry.seq = self.last;
        if self.entries.len() == RING_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        self.last
    }

    /// Entries after `since`, and how many seqs after it were evicted before they could be read.
    pub fn since(&self, since: u64) -> (Vec<Entry>, u64) {
        let skipped = self
            .entries
            .front()
            .map_or(0, |e| e.seq.saturating_sub(since.saturating_add(1)));
        let start = self.entries.partition_point(|e| e.seq <= since);
        (self.entries.range(start..).cloned().collect(), skipped)
    }
}

/// Lock order: file before ring. A writer never holds both.
pub struct Pipeline {
    level: AtomicUsize,
    redactor: Redactor,
    ring: Mutex<Ring>,
    file: Mutex<FileSink>,
    echo: bool,
}

impl Pipeline {
    pub fn new(redactor: Redactor, level: LevelFilter, echo: bool) -> Self {
        Self {
            level: AtomicUsize::new(level as usize),
            redactor,
            ring: Mutex::default(),
            file: Mutex::default(),
            echo,
        }
    }

    pub fn level(&self) -> LevelFilter {
        LevelFilter::iter()
            .nth(self.level.load(Ordering::Relaxed))
            .unwrap_or(LevelFilter::Info)
    }

    pub fn set_level(&self, level: LevelFilter) {
        self.level.store(level as usize, Ordering::Relaxed);
    }

    /// Other crates' records only at Warn and above. UI and helper lines skip that filter: their
    /// target says nothing about where they came from.
    pub fn enabled(&self, level: Level, target: &str, source: Source) -> bool {
        level <= self.level()
            && (source != Source::App || level <= Level::Warn || target.starts_with("app_lib"))
    }

    pub fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.level(), record.target(), Source::App) {
            guarded(|| {
                let msg = record.args().to_string();
                self.ingest(record.level(), Source::App, record.target(), &msg);
            });
        }
    }

    pub fn push(&self, level: Level, source: Source, target: &str, msg: &str) {
        if self.enabled(level, target, source) {
            guarded(|| self.ingest(level, source, target, msg));
        }
    }

    pub fn redact(&self, text: &str) -> String {
        self.redactor.redact(text)
    }

    fn ingest(&self, level: Level, source: Source, target: &str, raw: &str) {
        let ts = timestamp();
        let room = RECORD_CAP.saturating_sub(prefix(&ts, level, source, target).len());
        let entry = Entry {
            seq: 0,
            ts,
            level,
            source,
            target: target.to_owned(),
            msg: self.prepare(raw, room),
        };
        let line = entry.line();
        if self.echo {
            // The one sanctioned stderr write: debug builds echo for `pnpm tauri dev`.
            eprint!("{line}");
        }
        let seq = lock(&self.ring).push(entry);
        self.write(seq, level, &line);
    }

    fn prepare(&self, raw: &str, room: usize) -> String {
        let input = raw.get(..cut_end(raw, INPUT_CAP)).unwrap_or_default();
        let msg = self.redactor.redact(input);
        let cut = raw.len() - input.len();
        if cut == 0 && msg.len() <= room {
            return msg;
        }
        let keep = floor_boundary(&msg, room.saturating_sub(MARKER_ROOM));
        let dropped = msg.len() - keep + cut;
        format!(
            "{}… [truncated {dropped} bytes]",
            msg.get(..keep).unwrap_or_default()
        )
    }

    fn write(&self, seq: u64, level: Level, line: &str) {
        let failure = lock(&self.file).write(seq, level, line);
        if let Some(failure) = failure {
            self.note(failure);
        }
    }

    /// A line about the logger itself: ring only, never through `log::`.
    fn note(&self, msg: String) {
        let entry = Entry {
            seq: 0,
            ts: timestamp(),
            level: Level::Error,
            source: Source::App,
            target: "app_lib::logging".into(),
            msg,
        };
        if self.echo {
            eprint!("{}", entry.line());
        }
        lock(&self.ring).push(entry);
    }

    /// Opens a new session file. With `backfill`, the ring so far is copied in first and writers
    /// skip every seq up to that point: nothing is lost or written twice.
    pub fn attach(&self, dir: &Path, header: &str, backfill: bool) {
        let mut file = lock(&self.file);
        if let Err(failure) = file.open(dir, header) {
            drop(file);
            self.note(failure);
            return;
        }
        self.attach_opened(file, backfill);
    }

    fn attach_opened(&self, mut file: MutexGuard<'_, FileSink>, backfill: bool) {
        let ring = lock(&self.ring);
        file.attached_through = ring.last;
        let backlog: Vec<(Level, String)> = if backfill {
            ring.entries.iter().map(|e| (e.level, e.line())).collect()
        } else {
            Vec::new()
        };
        drop(ring);
        let failure = backlog
            .iter()
            .find_map(|(level, line)| file.append(*level, line));
        drop(file);
        if let Some(failure) = failure {
            self.note(failure);
        }
    }

    /// Closes the file and clears its error: turning saving off is the reset a broken sink waits for.
    pub fn detach(&self) {
        let mut file = lock(&self.file);
        file.close();
        file.error = None;
    }

    pub fn file_state(&self) -> (bool, Option<String>) {
        let file = lock(&self.file);
        (file.writing(), file.error.clone())
    }

    pub fn tail(&self, since: u64) -> (Vec<Entry>, u64) {
        lock(&self.ring).since(since)
    }

    pub fn ring_text(&self) -> String {
        lock(&self.ring).entries.iter().map(Entry::line).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::files;
    use super::super::redact::{tests::identity, Redactor};
    use super::*;
    use std::io::{self, Write};

    fn pipeline(user: &str) -> Pipeline {
        Pipeline::new(Redactor::new(&identity(user)), LevelFilter::Info, false)
    }

    fn log(p: &Pipeline, level: Level, target: &str, msg: &str) {
        p.log(
            &log::Record::builder()
                .level(level)
                .target(target)
                .args(format_args!("{msg}"))
                .build(),
        );
    }

    fn msgs(p: &Pipeline) -> Vec<String> {
        p.tail(0).0.into_iter().map(|e| e.msg).collect()
    }

    #[test]
    fn levels_and_the_crate_filter_decide_what_is_kept() {
        let p = pipeline("Alice");
        log(&p, Level::Debug, "app_lib::x", "debug");
        log(&p, Level::Info, "app_lib::x", "info");
        log(&p, Level::Info, "tauri::x", "tauri info");
        log(&p, Level::Warn, "tauri::x", "tauri warn");
        p.push(Level::Info, Source::Ui, "webview", "ui info");
        p.push(Level::Info, Source::Helper, "anything", "helper info");
        p.set_level(LevelFilter::Debug);
        log(&p, Level::Debug, "app_lib::x", "debug on");
        assert_eq!(
            msgs(&p),
            ["info", "tauri warn", "ui info", "helper info", "debug on"]
        );
    }

    #[test]
    fn the_ring_numbers_from_one_and_reports_what_it_evicted() {
        let p = pipeline("Alice");
        for n in 0..RING_CAPACITY + 5 {
            log(&p, Level::Info, "app_lib", &n.to_string());
        }
        let (lines, skipped) = p.tail(0);
        assert_eq!(lines.len(), RING_CAPACITY);
        assert_eq!(lines[0].seq, 6);
        assert_eq!(skipped, 5);
        let (lines, skipped) = p.tail(RING_CAPACITY as u64);
        assert_eq!((lines.len(), skipped), (5, 0));
        assert_eq!(lines[0].seq, RING_CAPACITY as u64 + 1);
        assert_eq!(p.tail(RING_CAPACITY as u64 + 5).0.len(), 0);
    }

    #[test]
    fn a_line_has_the_documented_shape_and_indents_continuations() {
        let p = pipeline("Alice");
        log(
            &p,
            Level::Info,
            "app_lib::commands::tweaks",
            "one\r\ntwo\nthree",
        );
        let line = p.tail(0).0[0].line();
        let (ts, rest) = line.split_once(' ').unwrap();
        assert!(chrono::DateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.3f%:z").is_ok());
        assert_eq!(
            rest,
            "INFO  app    app_lib::commands::tweaks: one\n\ttwo\n\tthree\n"
        );
    }

    #[test]
    fn a_long_record_is_capped_on_a_char_boundary() {
        let p = pipeline("Alice");
        let long = "é".repeat(10_000);
        log(&p, Level::Info, "app_lib", &long);
        let entry = &p.tail(0).0[0];
        assert!(entry.line().len() <= RECORD_CAP);
        assert!(entry.msg.ends_with(" bytes]"));
        let kept = entry.msg.split('…').next().unwrap();
        assert!(kept.chars().all(|c| c == 'é'));
        assert!(entry.msg.contains("[truncated "));
        let dropped: usize = entry
            .msg
            .rsplit("truncated ")
            .next()
            .unwrap()
            .trim_end_matches(" bytes]")
            .parse()
            .unwrap();
        assert_eq!(kept.len() + dropped, long.len());
    }

    #[test]
    fn the_input_cut_never_keeps_a_fragment_of_an_identifier() {
        let p = pipeline("Alice");
        let filler = r"C:\Users\Alice\AppData\Local\Temp ".repeat(INPUT_CAP / 34);
        let pad = "q".repeat(INPUT_CAP - filler.len() - 20);
        log(
            &p,
            Level::Info,
            "app_lib",
            &format!("{filler}{pad} 6f1d2c3b-aaaa-4bbb-8ccc-0123456789ab after"),
        );
        let msg = &msgs(&p)[0];
        let end = &msg[msg.len() - 80..];
        assert!(msg.starts_with("%TEMP%"), "{end}");
        assert!(!msg.contains("6f1d2c3b"), "{end}");
    }

    #[test]
    fn memory_only_writes_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = pipeline("Alice");
        for level in [Level::Error, Level::Warn, Level::Info] {
            log(&p, level, "app_lib", "kept in memory");
        }
        assert_eq!(p.tail(0).0.len(), 3);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        assert!(!p.file_state().0);
    }

    #[test]
    fn personal_details_reach_neither_the_ring_nor_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = pipeline("Alice");
        p.attach(dir.path(), "header\n", true);
        let secrets = [
            "Alice",
            "DESKTOP-K2J9",
            "S-1-5-21-111-222-333-1001",
            "6f1d2c3b-aaaa-4bbb-8ccc-0123456789ab",
            r"C:\Users\Alice",
        ];
        for secret in secrets {
            for form in [
                secret.to_string(),
                secret.replace('\\', "\\\\"),
                secret.replace('\\', "/"),
                secret.to_ascii_uppercase(),
            ] {
                log(&p, Level::Error, "app_lib", &format!("saw {form} here"));
                log(&p, Level::Info, "app_lib", &format!("{form:?}"));
            }
        }
        let files = files::session_files(dir.path());
        assert_eq!(files.len(), 1);
        let bytes = std::fs::read(&files[0].path).unwrap().to_ascii_lowercase();
        let ring = msgs(&p).join("\n").to_ascii_lowercase();
        for secret in secrets {
            let secret = secret.to_ascii_lowercase();
            for form in [
                secret.clone(),
                secret.replace('\\', "\\\\"),
                secret.replace('\\', "/"),
            ] {
                assert!(!ring.contains(&form), "{form} in the ring");
                assert!(
                    !bytes.windows(form.len()).any(|w| w == form.as_bytes()),
                    "{form} in the file"
                );
            }
        }
    }

    #[test]
    fn attaching_while_a_writer_is_mid_record_loses_and_duplicates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let p = pipeline("Alice");
        log(&p, Level::Info, "app_lib", "before");
        // A writer that has pushed to the ring but not yet reached the file.
        let pending = Entry {
            seq: 0,
            ts: timestamp(),
            level: Level::Info,
            source: Source::App,
            target: "app_lib".into(),
            msg: "in flight".into(),
        };
        let line = pending.line();
        let seq = lock(&p.ring).push(pending);
        p.attach(dir.path(), "", true);
        p.write(seq, Level::Info, &line);
        log(&p, Level::Info, "app_lib", "after");
        let text = std::fs::read_to_string(&files::session_files(dir.path())[0].path).unwrap();
        for msg in ["before", "in flight", "after"] {
            assert_eq!(text.matches(msg).count(), 1, "{msg}:\n{text}");
        }
    }

    #[test]
    fn a_fresh_file_without_backfill_starts_empty() {
        let dir = tempfile::tempdir().unwrap();
        let p = pipeline("Alice");
        log(&p, Level::Info, "app_lib", "earlier");
        p.attach(dir.path(), "", false);
        log(&p, Level::Info, "app_lib", "later");
        let text = std::fs::read_to_string(&files::session_files(dir.path())[0].path).unwrap();
        assert!(!text.contains("earlier"));
        assert!(text.contains("later"));
    }

    struct Broken;

    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("disk full"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_write_failure_leaves_one_ring_line_and_goes_memory_only() {
        let p = pipeline("Alice");
        lock(&p.file).start(Box::new(Broken));
        for _ in 0..3 {
            log(&p, Level::Error, "app_lib", "record");
        }
        let entries = p.tail(0).0;
        let failures: Vec<_> = entries
            .iter()
            .filter(|e| e.target == "app_lib::logging")
            .collect();
        assert_eq!(failures.len(), 1);
        assert!(failures[0].msg.contains("disk full"));
        assert_eq!(entries.len(), 4);
        let (writing, error) = p.file_state();
        assert!(!writing);
        assert!(error.is_some());
        p.detach();
        assert_eq!(p.file_state(), (false, None));
    }

    #[test]
    fn a_record_logged_from_inside_the_logger_is_dropped() {
        let p = pipeline("Alice");
        guarded(|| log(&p, Level::Error, "app_lib", "nested"));
        assert!(p.tail(0).0.is_empty());
        log(&p, Level::Error, "app_lib", "outside");
        assert_eq!(msgs(&p), ["outside"]);
    }
}
