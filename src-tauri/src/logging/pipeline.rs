//! Level check, format, redact, cap, ring, file. The global logger is a thin wrapper over this, so
//! tests drive it without `log::set_logger`. Nothing here may call `log::`.

use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use log::{Level, LevelFilter};

use super::files::FileSink;
use super::redact::Redactor;

pub const RING_CAPACITY: usize = 2000;
pub const RECORD_CAP: usize = 4096;
/// Cut before redaction, so one huge message cannot stall every logging thread; the cut backs off
/// to a separator so no identifier fragment slips past redaction.
const INPUT_CAP: usize = 8 * 1024;
const MARKER_ROOM: usize = 40;

thread_local! {
    pub(super) static IN_LOGGER: Cell<bool> = const { Cell::new(false) };
}

struct InLogger;

impl Drop for InLogger {
    fn drop(&mut self) {
        IN_LOGGER.set(false);
    }
}

/// A record raised while this thread is already inside the logger is dropped, never recursed into.
/// The flag resets on unwind too: a caught panic must not silence the thread. Every path that takes
/// the ring or file lock runs in here, so the panic hook skips instead of re-locking on this thread.
pub(super) fn guarded<R>(f: impl FnOnce() -> R) -> Option<R> {
    if IN_LOGGER.get() {
        return None;
    }
    IN_LOGGER.set(true);
    let _reset = InLogger;
    Some(f())
}

/// Debug builds echo for `pnpm tauri dev`; `eprint!` panics when stderr is gone.
fn echo(line: &str) {
    let _ = std::io::Write::write_all(&mut std::io::stderr(), line.as_bytes());
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

const LEVEL_WIDTH: usize = 5;
const SOURCE_WIDTH: usize = 6;

fn prefix_len(ts: &str, level: Level, source: Source, target: &str) -> usize {
    ts.len()
        + level.as_str().len().max(LEVEL_WIDTH)
        + source.as_str().len().max(SOURCE_WIDTH)
        + target.len()
        + 5
}

impl Entry {
    /// Continuation lines are indented with a tab.
    pub fn line(&self) -> String {
        let newlines = self.msg.bytes().filter(|&b| b == b'\n').count();
        let mut line = String::with_capacity(
            prefix_len(&self.ts, self.level, self.source, &self.target)
                + self.msg.len()
                + newlines
                + 1,
        );
        let _ = write!(
            line,
            "{} {:<LEVEL_WIDTH$} {:<SOURCE_WIDTH$} {}: ",
            self.ts,
            self.level.as_str(),
            self.source.as_str(),
            self.target
        );
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

    pub fn cut_start(&self, text: &str, at: usize) -> usize {
        self.redactor.cut_start(text, at)
    }

    fn ingest(&self, level: Level, source: Source, target: &str, raw: &str) {
        let ts = timestamp();
        // The trailing newline is part of the line.
        let room = RECORD_CAP.saturating_sub(prefix_len(&ts, level, source, target) + 1);
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
            echo(&line);
        }
        let seq = lock(&self.ring).push(entry);
        self.write(seq, level, &line);
    }

    /// `room` is for the formatted line, where each `\n` becomes `\n\t`.
    fn prepare(&self, raw: &str, room: usize) -> String {
        let input = raw
            .get(..self.redactor.cut_end(raw, INPUT_CAP))
            .unwrap_or_default();
        let msg = self.redactor.redact(input);
        let cut = raw.len() - input.len();
        let width = |c: char| if c == '\n' { 2 } else { c.len_utf8() };
        if cut == 0 && msg.chars().map(width).sum::<usize>() <= room {
            return msg;
        }
        let budget = room.saturating_sub(MARKER_ROOM);
        let (mut used, mut keep) = (0, 0);
        for (i, c) in msg.char_indices() {
            used += width(c);
            if used > budget {
                break;
            }
            keep = i + c.len_utf8();
        }
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
            echo(&entry.line());
        }
        lock(&self.ring).push(entry);
    }

    /// Opens a new session file. With `backfill`, the ring so far is copied in first and writers
    /// skip every seq up to that point: nothing is lost or written twice.
    pub fn attach(&self, dir: &Path, header: &str, backfill: bool) {
        guarded(|| {
            let mut file = lock(&self.file);
            if let Err(failure) = file.open(dir, header) {
                drop(file);
                self.note(failure);
                return;
            }
            self.attach_opened(file, backfill);
        });
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
        guarded(|| {
            let mut file = lock(&self.file);
            file.close();
            file.error = None;
        });
    }

    pub fn file_state(&self) -> FileState {
        guarded(|| {
            let file = lock(&self.file);
            FileState {
                writing: file.writing(),
                soft_capped: file.soft_capped(),
                error: file.error.clone(),
            }
        })
        .unwrap_or_default()
    }

    pub fn tail(&self, since: u64) -> (Vec<Entry>, u64) {
        guarded(|| lock(&self.ring).since(since)).unwrap_or_default()
    }

    pub fn ring_text(&self) -> String {
        guarded(|| lock(&self.ring).entries.iter().map(Entry::line).collect()).unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileState {
    pub writing: bool,
    /// Past the soft cap: Info and Debug lines reach the ring only.
    pub soft_capped: bool,
    pub error: Option<String>,
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
    fn a_multi_line_record_is_capped_on_the_formatted_line() {
        let p = pipeline("Alice");
        log(&p, Level::Info, "app_lib", &"a\n".repeat(RECORD_CAP));
        let line = p.tail(0).0[0].line();
        assert!(line.len() <= RECORD_CAP, "{}", line.len());
        assert!(line.ends_with(" bytes]\n"));
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
    fn a_panic_inside_the_logger_leaves_the_thread_able_to_log() {
        let p = pipeline("Alice");
        let caught = std::panic::catch_unwind(|| guarded(|| panic!("inside")));
        assert!(caught.is_err());
        log(&p, Level::Error, "app_lib", "after");
        assert_eq!(msgs(&p), ["after"]);
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
        assert!(!p.file_state().writing);
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
        let state = p.file_state();
        assert!(!state.writing);
        assert!(state.error.is_some());
        p.detach();
        assert_eq!(p.file_state(), FileState::default());
    }

    #[test]
    fn the_prefix_length_matches_the_formatted_prefix() {
        for level in [
            Level::Error,
            Level::Warn,
            Level::Info,
            Level::Debug,
            Level::Trace,
        ] {
            for source in [Source::App, Source::Ui, Source::Helper] {
                let entry = Entry {
                    seq: 0,
                    ts: timestamp(),
                    level,
                    source,
                    target: "app_lib::x".into(),
                    msg: String::new(),
                };
                let len = prefix_len(&entry.ts, level, source, &entry.target);
                assert_eq!(entry.line().len(), len + 1, "{level} {source:?}");
            }
        }
    }

    #[test]
    fn the_input_cut_never_keeps_the_first_word_of_a_spaced_name() {
        let p = pipeline("John Smith");
        let unit = r"C:\Users\John Smith\AppData\Local\Temp ";
        let filler = unit.repeat((INPUT_CAP - 100) / unit.len());
        let pad = "q".repeat(INPUT_CAP - 6 - filler.len());
        let raw = format!("{filler}{pad} John Smith after");
        assert_eq!(&raw[INPUT_CAP..INPUT_CAP + 5], "Smith");
        log(&p, Level::Info, "app_lib", &raw);
        let msg = &msgs(&p)[0];
        assert!(msg.starts_with("%TEMP%"), "{msg}");
        assert!(!msg.contains("John"), "{}", &msg[msg.len() - 60..]);
    }

    #[test]
    fn the_soft_cap_is_reported() {
        let p = pipeline("Alice");
        lock(&p.file).start(Box::new(io::sink()));
        assert!(!p.file_state().soft_capped);
        let big = "x".repeat(files::SOFT_CAP as usize + 1);
        lock(&p.file).append(Level::Info, &big);
        let state = p.file_state();
        assert!(state.writing && state.soft_capped, "{state:?}");
    }

    struct Witness(std::sync::Arc<std::sync::atomic::AtomicBool>);

    impl Write for Witness {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Drop for Witness {
        fn drop(&mut self) {
            self.0.store(IN_LOGGER.get(), Ordering::SeqCst);
        }
    }

    /// The panic hook skips while IN_LOGGER is set; unset, a panic under the file lock would hang it.
    #[test]
    fn the_file_lock_is_only_held_inside_the_logger_guard() {
        let dir = tempfile::tempdir().unwrap();
        let p = pipeline("Alice");
        for reopen in [false, true] {
            let seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            lock(&p.file).start(Box::new(Witness(seen.clone())));
            if reopen {
                p.attach(dir.path(), "", false);
            } else {
                p.detach();
            }
            assert!(seen.load(Ordering::SeqCst), "reopen {reopen}");
            assert!(!IN_LOGGER.get());
        }
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
