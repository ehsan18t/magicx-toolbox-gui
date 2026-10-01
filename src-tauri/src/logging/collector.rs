//! The broker child's logger and the parent's re-log of its lines. The child keeps this crate's
//! records unredacted for its response; the parent redacts them when it re-logs (ADR-0010).

use std::sync::{Mutex, TryLockError};

use log::{Level, LevelFilter};

use super::pipeline::{guarded, lock, Source};

pub const MAX_LINES: usize = 200;
pub const MAX_LINE: usize = 512;
const HELPER_TARGET: &str = "app_lib::helper";

struct Kept {
    lines: Vec<String>,
    dropped: usize,
}

impl Kept {
    fn push(&mut self, line: String) {
        if self.lines.len() < MAX_LINES {
            self.lines.push(line);
        } else {
            self.dropped += 1;
        }
    }

    fn take(&mut self) -> Vec<String> {
        let mut lines = std::mem::take(&mut self.lines);
        let dropped = std::mem::take(&mut self.dropped);
        if dropped > 0 {
            lines.pop();
            lines.push(format!(
                "WARN app_lib::logging: {} more lines were not kept",
                dropped + 1
            ));
        }
        lines
    }
}

static KEPT: Mutex<Kept> = Mutex::new(Kept {
    lines: Vec::new(),
    dropped: 0,
});

struct Collector;

static COLLECTOR: Collector = Collector;

fn keeps(level: Level, target: &str, max: LevelFilter) -> bool {
    level <= max && target.starts_with("app_lib")
}

impl log::Log for Collector {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        keeps(metadata.level(), metadata.target(), log::max_level())
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            guarded(|| {
                let line = format!("{} {}: {}", record.level(), record.target(), record.args());
                lock(&KEPT).push(clean(&line));
            });
        }
    }

    fn flush(&self) {}
}

/// The child's logger, at Info until its request names the level.
pub fn install() {
    if log::set_logger(&COLLECTOR).is_ok() {
        set_detailed(false);
    }
}

pub fn set_detailed(detailed: bool) {
    log::set_max_level(if detailed {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });
}

pub fn drain() -> Vec<String> {
    lock(&KEPT).take()
}

/// For the panic hook: a panic inside `log` holds the lock, so never wait on it.
pub fn try_drain() -> Vec<String> {
    match KEPT.try_lock() {
        Ok(mut kept) => kept.take(),
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner().take(),
        Err(TryLockError::WouldBlock) => Vec::new(),
    }
}

/// Control characters become spaces; the result is cut to [`MAX_LINE`] bytes on a char boundary.
pub fn clean(line: &str) -> String {
    let mut out = String::with_capacity(line.len().min(MAX_LINE));
    for c in line.chars() {
        let c = if c.is_control() { ' ' } else { c };
        if out.len() + c.len_utf8() > MAX_LINE {
            break;
        }
        out.push(c);
    }
    out
}

/// At most [`MAX_LINES`], each cleaned. A line that is not "LEVEL app_lib…: msg" keeps its text at Info.
fn helper_lines(lines: &[String]) -> Vec<(Level, String, String)> {
    lines
        .iter()
        .take(MAX_LINES)
        .map(|raw| {
            let line = clean(raw);
            let parsed = line.split_once(' ').and_then(|(level, rest)| {
                let (target, msg) = rest.split_once(": ")?;
                let ours = target.starts_with("app_lib") && !target.contains(' ');
                let level: Level = level.parse().ok().filter(|_| ours)?;
                Some((level, target.to_owned(), msg.to_owned()))
            });
            parsed.unwrap_or_else(|| (Level::Info, HELPER_TARGET.to_owned(), line))
        })
        .collect()
}

/// Called by the parent only for a response that validated.
pub fn relog(batch: u64, lines: &[String]) {
    for (level, target, msg) in helper_lines(lines) {
        super::push(level, Source::Helper, &target, &format!("#{batch} {msg}"));
    }
}

pub fn relog_panic(batch: u64, lines: &[String], panic: &str) {
    relog(batch, lines);
    super::push(
        Level::Error,
        Source::Helper,
        HELPER_TARGET,
        &format!("#{batch} panicked: {}", clean(panic)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_this_crate_at_the_requested_level_is_kept() {
        assert!(keeps(Level::Info, "app_lib::x", LevelFilter::Info));
        assert!(!keeps(Level::Debug, "app_lib::x", LevelFilter::Info));
        assert!(keeps(Level::Debug, "app_lib::x", LevelFilter::Debug));
        assert!(!keeps(Level::Error, "tauri::x", LevelFilter::Debug));
    }

    #[test]
    fn the_child_keeps_at_most_the_cap_and_says_how_many_it_dropped() {
        let mut kept = Kept {
            lines: Vec::new(),
            dropped: 0,
        };
        for n in 0..MAX_LINES + 9 {
            kept.push(n.to_string());
        }
        let lines = kept.take();
        assert_eq!(lines.len(), MAX_LINES);
        assert_eq!(lines[MAX_LINES - 2], (MAX_LINES - 2).to_string());
        assert!(lines[MAX_LINES - 1].contains("10 more lines"), "{lines:?}");
        assert!(kept.take().is_empty());
    }

    #[test]
    fn a_held_lock_never_blocks_the_panic_drain() {
        let _held = lock(&KEPT);
        assert!(try_drain().is_empty());
    }

    #[test]
    fn a_line_is_cleaned_and_capped_on_a_char_boundary() {
        let line = clean(&format!("a\r\nb\u{1b}[2J\u{7}{}", "é".repeat(MAX_LINE)));
        assert!(line.len() <= MAX_LINE);
        assert!(!line.chars().any(char::is_control), "{line:?}");
        assert!(line.starts_with("a  b [2J "), "{line:?}");
    }

    #[test]
    fn a_forged_log_is_capped_and_its_control_characters_stripped() {
        let forged: Vec<String> = (0..MAX_LINES * 3)
            .map(|n| {
                format!(
                    "DEBUG app_lib::x: {n}\r\nERROR app_lib::y: injected{}",
                    "z".repeat(900)
                )
            })
            .collect();
        let lines = helper_lines(&forged);
        assert_eq!(lines.len(), MAX_LINES);
        for (level, target, msg) in &lines {
            assert_eq!((*level, target.as_str()), (Level::Debug, "app_lib::x"));
            assert!(msg.len() <= MAX_LINE);
            assert!(!msg.chars().any(char::is_control), "{msg:?}");
        }
    }

    #[test]
    fn a_line_that_names_no_level_or_another_crate_is_kept_at_info() {
        for raw in [
            "just text",
            "LOUD app_lib::x: m",
            "ERROR tauri::x: m",
            "ERROR app_lib x: m",
        ] {
            let (level, target, msg) = helper_lines(&[raw.to_owned()]).remove(0);
            assert_eq!(
                (level, target.as_str(), msg.as_str()),
                (Level::Info, HELPER_TARGET, raw)
            );
        }
    }
}
