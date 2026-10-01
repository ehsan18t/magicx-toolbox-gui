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
use std::io::Read;
use std::process::{Child, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::Error;
use crate::services::system32::SystemTool;

/// Lists every installed package name, per-user (`R:`) and provisioned (`P:`), one per line.
/// `-AllUsers` needs admin; without it the call fails rather than silently reporting only the
/// current user's packages, which would read as "absent" for anything installed for someone else.
/// That failure surfaces as `Err`, never as an empty set.
const ENUMERATE: &str = r#"
$ErrorActionPreference = 'Stop'
Get-AppxPackage -AllUsers | ForEach-Object { 'R:' + $_.Name }
Get-AppxProvisionedPackage -Online | ForEach-Object { 'P:' + $_.DisplayName }
"#;

/// The unelevated fallback: only this account's packages, so a miss proves nothing.
const ENUMERATE_CURRENT_USER: &str = r#"
$ErrorActionPreference = 'Stop'
Get-AppxPackage | ForEach-Object { 'R:' + $_.Name }
"#;

/// A normal run takes about 400ms; past this the spawn is treated as hung and killed.
const ENUMERATE_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

/// Lowercased package names.
#[derive(Debug, Default)]
struct Packages {
    registered: HashSet<String>,
    provisioned: HashSet<String>,
}

type Built = Result<Packages, String>;

/// Whether any asked-about package is registered for a user, and whether any is provisioned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppxLookup {
    pub registered: bool,
    pub provisioned: bool,
}

/// Lazily-built set of installed package names, lowercased for case-insensitive lookup.
///
/// The build result is cached including its failure: a machine where the enumeration cannot run
/// would otherwise retry the same failing 400ms spawn once per asking probe. [`Self::invalidate`]
/// clears it so a later sweep re-observes, which is what makes a removal visible after an apply.
pub struct AppxIndex {
    all_users: bool,
    cache: Mutex<Cache>,
}

impl Default for AppxIndex {
    fn default() -> Self {
        Self {
            all_users: true,
            cache: Mutex::default(),
        }
    }
}

#[derive(Default)]
struct Cache {
    generation: u64,
    built: Option<Arc<Built>>,
}

impl AppxIndex {
    /// This account's packages only, with no provisioned list: works without admin.
    pub fn current_user() -> Self {
        Self {
            all_users: false,
            ..Self::default()
        }
    }

    /// Whether any of `packages` is installed. Names are compared case-insensitively, matching how
    /// Windows treats package names and how the corpus spells them.
    pub fn any_installed(&self, packages: &[String]) -> Result<bool, Error> {
        self.lookup(packages).map(|l| l.registered || l.provisioned)
    }

    pub fn lookup(&self, packages: &[String]) -> Result<AppxLookup, Error> {
        let script = if self.all_users {
            ENUMERATE
        } else {
            ENUMERATE_CURRENT_USER
        };
        match &*self.built(|| enumerate(script)) {
            Ok(names) => {
                let any = |set: &HashSet<String>| {
                    packages
                        .iter()
                        .any(|p| set.contains(&p.to_ascii_lowercase()))
                };
                Ok(AppxLookup {
                    registered: any(&names.registered),
                    provisioned: any(&names.provisioned),
                })
            }
            Err(msg) => Err(Error::CommandExecution(format!(
                "could not enumerate installed packages: {msg}"
            ))),
        }
    }

    /// Drops the cached enumeration so the next ask re-observes the machine. Called wherever the
    /// probe cache is invalidated, since removing an app is exactly what makes this stale.
    pub fn invalidate(&self) {
        let mut cache = self.lock();
        cache.generation += 1;
        cache.built = None;
    }

    fn lock(&self) -> MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Enumerates outside the lock so a slow spawn never blocks `invalidate`; a result that an
    /// `invalidate` overtook is returned to its asker but not cached.
    fn built(&self, enumerate: impl FnOnce() -> Result<Packages, Error>) -> Arc<Built> {
        let generation = {
            let cache = self.lock();
            if let Some(built) = &cache.built {
                return Arc::clone(built);
            }
            cache.generation
        };
        let built = Arc::new(enumerate().map_err(|e| e.to_string()));
        let mut cache = self.lock();
        if cache.generation == generation && cache.built.is_none() {
            cache.built = Some(Arc::clone(&built));
        }
        built
    }
}

fn enumerate(script: &str) -> Result<Packages, Error> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let child = SystemTool::PowerShell
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
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::CommandExecution(format!("failed to spawn PowerShell: {e}")))?;
    let (code, stdout, stderr) = output_within(child, ENUMERATE_TIMEOUT)?;

    if code != 0 {
        return Err(Error::CommandExecution(format!(
            "package enumeration exited with {code}: {}",
            String::from_utf8_lossy(&stderr).trim()
        )));
    }

    Ok(parse_packages(&String::from_utf8_lossy(&stdout)))
}

fn parse_packages(stdout: &str) -> Packages {
    let mut packages = Packages::default();
    for line in stdout.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix("R:") {
            packages.registered.insert(name.to_ascii_lowercase());
        } else if let Some(name) = line.strip_prefix("P:") {
            packages.provisioned.insert(name.to_ascii_lowercase());
        }
    }
    packages
}

/// Exit code, stdout and stderr of `child`, or `Err` after killing it once `timeout` passes.
/// Pipes drain on threads so a full pipe buffer cannot stall the child past the bound.
fn output_within(mut child: Child, timeout: Duration) -> Result<(i32, Vec<u8>, Vec<u8>), Error> {
    fn drain(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
        thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut buf);
            }
            buf
        })
    }
    let out = drain(child.stdout.take());
    let err = drain(child.stderr.take());

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < timeout => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Error::CommandExecution(format!(
                    "package enumeration exceeded {}s and was terminated",
                    timeout.as_secs()
                )));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Error::CommandExecution(format!(
                    "failed to wait on package enumeration: {e}"
                )));
            }
        }
    };
    let joined = |t: thread::JoinHandle<Vec<u8>>| t.join().unwrap_or_default();
    Ok((status.code().unwrap_or(-1), joined(out), joined(err)))
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
        index.lock().built = Some(Arc::new(Err("winrt unavailable".to_string())));

        let err = index
            .any_installed(&["Microsoft.GetHelp".to_string()])
            .expect_err("a failed enumeration must not read as absent");
        assert!(err.to_string().contains("winrt unavailable"), "got {err}");
    }

    fn registered(names: &[&str]) -> Packages {
        Packages {
            registered: names.iter().map(|n| n.to_string()).collect(),
            provisioned: HashSet::new(),
        }
    }

    #[test]
    fn lookup_is_case_insensitive_and_invalidation_clears_the_cache() {
        let index = AppxIndex::default();
        index.lock().built = Some(Arc::new(Ok(registered(&["microsoft.gethelp"]))));

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
            index.lock().built.is_none(),
            "invalidate must drop the cached enumeration so the next ask re-observes"
        );
    }

    #[test]
    fn enumeration_runs_unlocked_and_an_overtaken_result_is_not_cached() {
        let index = AppxIndex::default();
        let built = index.built(|| {
            // Deadlocks if the cache lock were held across the enumeration.
            index.invalidate();
            Ok(registered(&["stale"]))
        });
        assert!(matches!(&*built, Ok(names) if names.registered.contains("stale")));
        assert!(
            index.lock().built.is_none(),
            "a result an invalidate overtook must not be cached"
        );

        index.built(|| Ok(Packages::default()));
        assert!(index.lock().built.is_some());
    }

    #[test]
    fn output_lines_split_into_registered_and_provisioned() {
        let packages =
            parse_packages("R:Microsoft.GetHelp\r\nP:Clipchamp.Clipchamp\r\nnoise\r\n\r\n");
        let index = AppxIndex::default();
        index.lock().built = Some(Arc::new(Ok(packages)));
        let lookup = |n: &str| index.lookup(&[n.to_string()]).unwrap();
        assert_eq!(
            lookup("microsoft.gethelp"),
            AppxLookup {
                registered: true,
                provisioned: false
            }
        );
        assert_eq!(
            lookup("CLIPCHAMP.clipchamp"),
            AppxLookup {
                registered: false,
                provisioned: true
            }
        );
        assert!(index
            .any_installed(&["Clipchamp.Clipchamp".into()])
            .unwrap());
        assert!(!index.any_installed(&["noise".into()]).unwrap());
    }

    #[test]
    fn a_hung_spawn_times_out_as_an_error() {
        use std::os::windows::process::CommandExt;
        let child = SystemTool::PowerShell
            .command()
            .unwrap()
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep 30",
            ])
            .creation_flags(0x0800_0000)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("PowerShell must spawn");
        let err = output_within(child, Duration::from_millis(200))
            .expect_err("a spawn past its bound must be an error, never an empty output");
        assert!(err.to_string().contains("terminated"), "got {err}");
    }
}
