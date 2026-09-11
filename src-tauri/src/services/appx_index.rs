//! One enumeration of installed Appx packages, shared by every probe that asks about one.
//!
//! Asking about a single package costs the same as listing every package: measured, a
//! `Get-AppxPackage -AllUsers -Name X` is 424ms and a bare `Get-AppxPackage -AllUsers` is 416ms,
//! because the cost is the module load plus the WinRT enumeration, and both are paid per process.
//! So a corpus that asks about a dozen packages one spawn at a time pays that dozen times over for
//! an answer one spawn already contained.
//!
//! This builds the answer once and hands it to every asker. The whole enumeration, per-user and
//! provisioned together, is a single PowerShell run.

use std::collections::HashSet;
use std::sync::Mutex;

use crate::error::Error;
use crate::services::system32::SystemTool;

/// Lists every installed package name, per-user and provisioned, one per line. `-AllUsers` needs
/// admin; without it the call fails rather than silently reporting only the current user's
/// packages, which would read as "absent" for anything installed for someone else. That failure
/// surfaces as `Err`, never as an empty set, so a probe reports "cannot tell" instead of a
/// confident wrong answer.
const ENUMERATE: &str = r#"
$ErrorActionPreference = 'Stop'
Get-AppxPackage -AllUsers | ForEach-Object { $_.Name }
Get-AppxProvisionedPackage -Online | ForEach-Object { $_.DisplayName }
"#;

/// Lazily-built set of installed package names, lowercased for case-insensitive lookup.
///
/// The build result is cached including its failure: a machine where the enumeration cannot run
/// would otherwise retry the same failing 400ms spawn once per asking probe. [`Self::invalidate`]
/// clears it so a later sweep re-observes, which is what makes a removal visible after an apply.
#[derive(Default)]
pub struct AppxIndex {
    cached: Mutex<Option<Result<HashSet<String>, String>>>,
}

impl AppxIndex {
    /// Whether any of `packages` is installed. Names are compared case-insensitively, matching how
    /// Windows treats package names and how the corpus spells them.
    pub fn any_installed(&self, packages: &[String]) -> Result<bool, Error> {
        let mut guard = self.cached.lock().unwrap_or_else(|e| e.into_inner());
        let built = guard.get_or_insert_with(|| enumerate().map_err(|e| e.to_string()));
        match built {
            Ok(names) => Ok(packages
                .iter()
                .any(|p| names.contains(&p.to_ascii_lowercase()))),
            Err(msg) => Err(Error::CommandExecution(format!(
                "could not enumerate installed packages: {msg}"
            ))),
        }
    }

    /// Drops the cached enumeration so the next ask re-observes the machine. Called wherever the
    /// probe cache is invalidated, since removing an app is exactly what makes this stale.
    pub fn invalidate(&self) {
        *self.cached.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

fn enumerate() -> Result<HashSet<String>, Error> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let utf16: Vec<u8> = ENUMERATE
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let output = SystemTool::PowerShell
        .command()?
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &base64(&utf16),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| Error::CommandExecution(format!("failed to spawn PowerShell: {e}")))?;

    if !output.status.success() {
        return Err(Error::CommandExecution(format!(
            "package enumeration exited with {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_ascii_lowercase)
        .collect())
}

/// Standard base64 (RFC 4648), for `-EncodedCommand`'s UTF-16LE payload.
fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
        assert_eq!(base64(b""), "");
    }

    /// A cached failure must surface as `Err`, never as "no packages installed": an empty set would
    /// make every `AppxAbsent` probe report the app as already removed.
    #[test]
    fn an_enumeration_failure_is_an_error_not_an_empty_set() {
        let index = AppxIndex::default();
        *index.cached.lock().unwrap() = Some(Err("winrt unavailable".to_string()));

        let err = index
            .any_installed(&["Microsoft.GetHelp".to_string()])
            .expect_err("a failed enumeration must not read as absent");
        assert!(err.to_string().contains("winrt unavailable"), "got {err}");
    }

    #[test]
    fn lookup_is_case_insensitive_and_invalidation_clears_the_cache() {
        let index = AppxIndex::default();
        *index.cached.lock().unwrap() = Some(Ok(HashSet::from(["microsoft.gethelp".to_string()])));

        assert!(index.any_installed(&["Microsoft.GetHelp".into()]).unwrap());
        assert!(!index.any_installed(&["Microsoft.Absent".into()]).unwrap());
        assert!(
            index
                .any_installed(&["Microsoft.Absent".into(), "MICROSOFT.GETHELP".into()])
                .unwrap(),
            "any of the listed packages counts, and case must not matter"
        );

        index.invalidate();
        assert!(
            index.cached.lock().unwrap().is_none(),
            "invalidate must drop the cached enumeration so the next ask re-observes"
        );
    }
}
