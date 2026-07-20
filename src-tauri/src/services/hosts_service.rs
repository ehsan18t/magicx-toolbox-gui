//! Hosts file service for managing entries in the Windows hosts file.
//!
//! The hosts file is located at C:\Windows\System32\drivers\etc\hosts
//! and requires administrator privileges to modify.

use crate::error::Error;
use crate::models::tweak::{HostsAction, HostsChange};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Marker comment to identify entries managed by MagicX Toolbox
const MAGICX_MARKER: &str = "# MagicX Toolbox";

/// Get the path to the Windows hosts file
fn get_hosts_path() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

/// Represents a single entry in the hosts file
#[derive(Debug, Clone)]
pub struct HostsEntry {
    pub ip: String,
    pub domain: String,
}

/// Parse hosts-file text into entries.
///
/// Split out from `read_hosts_file` so the parsing rules can be tested without
/// touching `C:\Windows\System32\drivers\etc\hosts`, which needs admin and is
/// machine-global.
pub fn parse_hosts_lines(content: &str) -> Vec<HostsEntry> {
    let mut entries = Vec::new();

    // A UTF-8 BOM is glued to the first line; strip it so the first IP isn't corrupted.
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments (including MagicX markers)
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // A line maps one IP to one or more hostnames: `IP host1 [host2 ...] [# comment]`.
        // Strip any inline comment, then emit an entry per hostname so `entry_exists` sees every
        // one (not just the first) and removal can target a single hostname.
        let code = match trimmed.find('#') {
            Some(pos) => trimmed[..pos].trim_end(),
            None => trimmed,
        };
        let mut tokens = code.split_whitespace();
        if let Some(ip) = tokens.next() {
            for domain in tokens {
                entries.push(HostsEntry {
                    ip: ip.to_string(),
                    domain: domain.to_string(),
                });
            }
        }
    }

    entries
}

/// Read and parse the hosts file
pub fn read_hosts_file() -> Result<Vec<HostsEntry>, Error> {
    let hosts_path = get_hosts_path();

    if !hosts_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;

    Ok(parse_hosts_lines(&content))
}

/// Check if a specific hosts entry exists
pub fn entry_exists(ip: &str, domain: &str) -> Result<bool, Error> {
    let entries = read_hosts_file()?;
    Ok(entries
        .iter()
        .any(|e| e.ip == ip && e.domain.eq_ignore_ascii_case(domain)))
}

/// Apply a hosts change
pub fn apply_hosts_change(change: &HostsChange) -> Result<(), Error> {
    match change.action {
        HostsAction::Add => add_hosts_entry(&change.ip, &change.domain, change.comment.as_deref()),
        HostsAction::Remove => remove_hosts_entry(&change.ip, &change.domain),
    }
}

/// Add an entry to the hosts file
pub fn add_hosts_entry(ip: &str, domain: &str, comment: Option<&str>) -> Result<(), Error> {
    // First check if it already exists
    if entry_exists(ip, domain)? {
        log::debug!("Hosts entry already exists: {} -> {}", domain, ip);
        return Ok(());
    }

    let hosts_path = get_hosts_path();

    // Read existing content
    let existing_content = if hosts_path.exists() {
        fs::read_to_string(&hosts_path)
            .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?
    } else {
        String::new()
    };

    // Build new entry
    let entry_line = if let Some(c) = comment {
        format!("{}\t{}\t# {}", ip, domain, c)
    } else {
        format!("{}\t{}", ip, domain)
    };

    // Append to file with marker
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to open hosts file for writing: {}", e)))?;

    // Ensure we start on a new line. Windows hosts files are CRLF, so append CRLF (not LF).
    let needs_newline = !existing_content.ends_with('\n') && !existing_content.is_empty();
    let content = format!(
        "{}{}\r\n{}\r\n",
        if needs_newline { "\r\n" } else { "" },
        MAGICX_MARKER,
        entry_line
    );

    file.write_all(content.as_bytes())
        .map_err(|e| Error::WindowsApi(format!("Failed to write to hosts file: {}", e)))?;

    log::info!("Added hosts entry: {} -> {}", domain, ip);
    Ok(())
}

/// What removing `(ip, domain)` does to a single hosts line.
enum LineEdit {
    /// Not a matching entry line — keep it verbatim.
    Unchanged,
    /// The removed hostname was the only one on the line — drop the whole line.
    Dropped,
    /// Other hostnames remain — the line rewritten without the target hostname.
    Rewritten(String),
}

/// Decide what to do with one hosts line when removing `(ip, domain)`.
///
/// Matches only on the IP plus the hostname list (case-insensitive), preserves any inline comment,
/// and — crucially — keeps the line's other hostnames instead of deleting a shared line wholesale.
fn edit_hosts_line(raw: &str, ip: &str, domain: &str) -> LineEdit {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return LineEdit::Unchanged;
    }

    let (code, comment) = match trimmed.find('#') {
        Some(pos) => (trimmed[..pos].trim_end(), Some(trimmed[pos..].to_string())),
        None => (trimmed, None),
    };

    let mut tokens = code.split_whitespace();
    let line_ip = match tokens.next() {
        Some(t) => t,
        None => return LineEdit::Unchanged,
    };
    if line_ip != ip {
        return LineEdit::Unchanged;
    }

    let hosts: Vec<&str> = tokens.collect();
    if !hosts.iter().any(|h| h.eq_ignore_ascii_case(domain)) {
        return LineEdit::Unchanged; // this IP line doesn't carry the target hostname
    }

    let remaining: Vec<&str> = hosts
        .into_iter()
        .filter(|h| !h.eq_ignore_ascii_case(domain))
        .collect();
    if remaining.is_empty() {
        return LineEdit::Dropped;
    }

    let mut rebuilt = format!("{}\t{}", line_ip, remaining.join(" "));
    if let Some(c) = comment {
        rebuilt.push(' ');
        rebuilt.push_str(&c);
    }
    LineEdit::Rewritten(rebuilt)
}

/// Remove `(ip, domain)` from hosts-file text, returning new text with CRLF line endings.
///
/// A line mapping the IP to several hostnames keeps its other hostnames; only when the removed
/// hostname was the last one is the whole line dropped, along with a MagicX marker directly above
/// it. Foreign comments and blank lines are preserved. A leading UTF-8 BOM is stripped.
pub fn remove_entry_from_hosts(content: &str, ip: &str, domain: &str) -> String {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());

    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];

        // A MagicX marker is dropped only when the entry line right below it is fully removed.
        if raw.trim().starts_with(MAGICX_MARKER) {
            match lines
                .get(i + 1)
                .map(|next| edit_hosts_line(next, ip, domain))
            {
                Some(LineEdit::Dropped) => {
                    i += 2; // drop marker + entry line
                    continue;
                }
                Some(LineEdit::Rewritten(new_line)) => {
                    out.push(raw.to_string()); // keep marker
                    out.push(new_line);
                    i += 2;
                    continue;
                }
                _ => {
                    out.push(raw.to_string());
                    i += 1;
                    continue;
                }
            }
        }

        match edit_hosts_line(raw, ip, domain) {
            LineEdit::Dropped => {}
            LineEdit::Rewritten(new_line) => out.push(new_line),
            LineEdit::Unchanged => out.push(raw.to_string()),
        }
        i += 1;
    }

    let mut result = out.join("\r\n");
    if !result.is_empty() {
        result.push_str("\r\n");
    }
    result
}

/// Remove an entry from the hosts file
pub fn remove_hosts_entry(ip: &str, domain: &str) -> Result<(), Error> {
    let hosts_path = get_hosts_path();

    if !hosts_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;

    let new_content = remove_entry_from_hosts(&content, ip, domain);
    log::info!("Removing hosts entry: {} -> {}", domain, ip);

    fs::write(&hosts_path, new_content.as_bytes())
        .map_err(|e| Error::WindowsApi(format!("Failed to write hosts file: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The parser must not care about line endings. Windows hosts files are CRLF,
    /// but a file touched by another tool can end up mixed.
    #[test]
    fn parses_crlf_lf_and_mixed_line_endings_identically() {
        let lf = "0.0.0.0 a.example\n127.0.0.1 b.example\n";
        let crlf = "0.0.0.0 a.example\r\n127.0.0.1 b.example\r\n";
        let mixed = "0.0.0.0 a.example\r\n127.0.0.1 b.example\n";

        for (label, text) in [("lf", lf), ("crlf", crlf), ("mixed", mixed)] {
            let e = parse_hosts_lines(text);
            assert_eq!(e.len(), 2, "{label}: wrong entry count");
            assert_eq!(e[0].ip, "0.0.0.0", "{label}");
            assert_eq!(e[0].domain, "a.example", "{label}");
            assert_eq!(e[1].domain, "b.example", "{label}");
        }
    }

    #[test]
    fn ignores_comments_blank_lines_and_our_own_marker() {
        let text = concat!(
            "# Copyright (c) 1993-2009 Microsoft Corp.\r\n",
            "\r\n",
            "       \r\n",
            "# 102.54.94.97     rhino.acme.com          # source server\r\n",
            "# MagicX Toolbox\r\n",
            "0.0.0.0 telemetry.example\r\n"
        );
        let e = parse_hosts_lines(text);
        assert_eq!(e.len(), 1, "only the real entry should parse");
        assert_eq!(e[0].domain, "telemetry.example");
    }

    #[test]
    fn strips_inline_comments_from_the_domain() {
        let e = parse_hosts_lines("0.0.0.0 telemetry.example # blocked by us\r\n");
        assert_eq!(e.len(), 1);
        assert_eq!(
            e[0].domain, "telemetry.example",
            "the trailing comment leaked into the domain"
        );
    }

    #[test]
    fn tolerates_tabs_and_repeated_spaces_as_separators() {
        let e = parse_hosts_lines("0.0.0.0\t\ttelemetry.example\r\n127.0.0.1    local.example\r\n");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].domain, "telemetry.example");
        assert_eq!(e[1].domain, "local.example");
    }

    /// A hosts line may map one IP to several hostnames; every one is now a distinct entry, so
    /// `entry_exists` sees them all and removal can target a single hostname.
    #[test]
    fn every_hostname_on_a_multi_host_line_is_parsed() {
        let e = parse_hosts_lines("127.0.0.1 alpha.example beta.example gamma.example\r\n");
        assert_eq!(e.len(), 3);
        assert_eq!(e[0].domain, "alpha.example");
        assert_eq!(e[1].domain, "beta.example");
        assert_eq!(e[2].domain, "gamma.example");
        assert!(e.iter().all(|x| x.ip == "127.0.0.1"));
    }

    /// A leading UTF-8 BOM is stripped, so the first entry's IP is intact.
    #[test]
    fn a_utf8_bom_is_stripped_from_the_first_entry() {
        let e = parse_hosts_lines("\u{feff}0.0.0.0 telemetry.example\r\n127.0.0.1 ok.example\r\n");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].ip, "0.0.0.0", "the BOM must not corrupt the first IP");
        assert_eq!(e[0].domain, "telemetry.example");
        assert_eq!(e[1].ip, "127.0.0.1");
    }

    #[test]
    fn lines_without_a_hostname_are_skipped() {
        let e = parse_hosts_lines("0.0.0.0\r\n\r\n0.0.0.0   \r\n");
        assert!(e.is_empty(), "an IP with no hostname is not an entry");
    }

    #[test]
    fn removing_one_hostname_keeps_the_others_on_the_line() {
        let out = remove_entry_from_hosts(
            "127.0.0.1 alpha.example beta.example gamma.example\r\n",
            "127.0.0.1",
            "beta.example",
        );
        let entries = parse_hosts_lines(&out);
        assert_eq!(entries.len(), 2, "only beta should be removed");
        assert!(entries.iter().any(|e| e.domain == "alpha.example"));
        assert!(entries.iter().any(|e| e.domain == "gamma.example"));
        assert!(!entries.iter().any(|e| e.domain == "beta.example"));
    }

    #[test]
    fn removing_the_last_hostname_drops_the_line_and_its_marker() {
        let content = concat!(
            "# MagicX Toolbox\r\n",
            "0.0.0.0\ttelemetry.example\r\n",
            "127.0.0.1 keep.example\r\n"
        );
        let out = remove_entry_from_hosts(content, "0.0.0.0", "telemetry.example");
        assert!(
            !out.contains("MagicX Toolbox"),
            "the marker should be dropped with its only entry"
        );
        assert!(!out.contains("telemetry.example"));
        assert!(out.contains("keep.example"));
    }

    #[test]
    fn removal_normalizes_to_crlf() {
        // LF input (e.g. mangled by a previous buggy write) must come back out as CRLF.
        let out = remove_entry_from_hosts(
            "0.0.0.0 a.example\n0.0.0.0 b.example\n",
            "0.0.0.0",
            "a.example",
        );
        assert!(out.contains("b.example"));
        assert!(out.matches("\r\n").count() > 0);
        assert_eq!(
            out.matches('\n').count(),
            out.matches("\r\n").count(),
            "every LF must be part of a CRLF"
        );
    }
}
