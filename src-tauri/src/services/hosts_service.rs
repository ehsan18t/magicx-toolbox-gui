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

/// Read and parse the hosts file
pub fn read_hosts_file() -> Result<Vec<HostsEntry>, Error> {
    let hosts_path = get_hosts_path();

    if !hosts_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;

    let mut entries = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments (including MagicX markers)
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse the entry: IP whitespace domain [# comment]
        let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
        if parts.len() >= 2 {
            let ip = parts[0].to_string();
            let rest = parts[1].trim();

            // Extract domain (strip inline comments)
            let domain = if let Some(hash_pos) = rest.find('#') {
                rest[..hash_pos].trim().to_string()
            } else {
                rest.split_whitespace().next().unwrap_or("").to_string()
            };

            if !domain.is_empty() {
                entries.push(HostsEntry { ip, domain });
            }
        }
    }

    Ok(entries)
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

    // Ensure we start on a new line
    let needs_newline = !existing_content.ends_with('\n') && !existing_content.is_empty();
    let content = format!(
        "{}{}\n{}\n",
        if needs_newline { "\n" } else { "" },
        MAGICX_MARKER,
        entry_line
    );

    file.write_all(content.as_bytes())
        .map_err(|e| Error::WindowsApi(format!("Failed to write to hosts file: {}", e)))?;

    log::info!("Added hosts entry: {} -> {}", domain, ip);
    Ok(())
}

/// Remove an entry from the hosts file
pub fn remove_hosts_entry(ip: &str, domain: &str) -> Result<(), Error> {
    let hosts_path = get_hosts_path();

    if !hosts_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<&str> = Vec::new();

    // Two-pass approach: first identify which lines to remove,
    // then only remove markers whose associated entry is being removed.
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Check if this is a MagicX marker followed by the target entry
        if trimmed.starts_with(MAGICX_MARKER) {
            // Look ahead to see if the next non-empty line is the entry we're removing
            if let Some(next_line) = lines.get(i + 1) {
                let next_trimmed = next_line.trim();
                if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                    let parts: Vec<&str> = next_trimmed
                        .splitn(2, |c: char| c.is_whitespace())
                        .collect();
                    if parts.len() >= 2 {
                        let line_ip = parts[0];
                        let line_domain = parts[1].split_whitespace().next().unwrap_or("");
                        if line_ip == ip && line_domain.eq_ignore_ascii_case(domain) {
                            // Skip this marker and the next entry line
                            log::info!("Removing hosts entry: {} -> {}", domain, ip);
                            i += 2; // Skip marker + entry
                            continue;
                        }
                    }
                }
            }
            // Marker is for a different entry, keep it
            new_lines.push(lines[i]);
            i += 1;
            continue;
        }

        // Check if this is the entry we want to remove (without a preceding marker)
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
            if parts.len() >= 2 {
                let line_ip = parts[0];
                let line_domain = parts[1].split_whitespace().next().unwrap_or("");

                if line_ip == ip && line_domain.eq_ignore_ascii_case(domain) {
                    log::info!("Removing hosts entry: {} -> {}", domain, ip);
                    i += 1;
                    continue;
                }
            }
        }

        new_lines.push(lines[i]);
        i += 1;
    }

    // Write back the modified content
    let new_content = new_lines.join("\n");
    fs::write(&hosts_path, new_content.as_bytes())
        .map_err(|e| Error::WindowsApi(format!("Failed to write hosts file: {}", e)))?;

    Ok(())
}
