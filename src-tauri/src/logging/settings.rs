use std::ffi::OsString;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "logging.json";
const PERSIST_ARG: &str = "--log-persist=";
const DETAILED_ARG: &str = "--log-detailed=";

/// A hand-edited file may omit a field: it takes its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub persist: bool,
    pub detailed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            persist: true,
            detailed: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Loaded {
    pub settings: Settings,
    pub error: Option<String>,
}

/// A missing file is the defaults and is not created. A file that cannot be read keeps this session
/// in memory and is left as it is until the user changes a setting.
pub fn load(path: &Path) -> Loaded {
    let memory_only = |why: String| Loaded {
        settings: Settings {
            persist: false,
            detailed: false,
        },
        error: Some(format!(
            "{FILE_NAME} {why}, so logs are kept in memory only. Change a setting to replace it."
        )),
    };
    match std::fs::read(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Loaded {
            settings: Settings::default(),
            error: None,
        },
        Err(e) => memory_only(format!("could not be read ({e})")),
        // Notepad saves "UTF-8 with BOM", which serde_json rejects.
        Ok(bytes) => {
            match serde_json::from_slice(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes)) {
                Ok(settings) => Loaded {
                    settings,
                    error: None,
                },
                Err(e) => memory_only(format!("is not valid ({e})")),
            }
        }
    }
}

/// Temp file and rename: a failed save leaves the old file whole.
pub fn save(path: &Path, settings: Settings) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_vec_pretty(&settings).map_err(std::io::Error::other)?;
    crate::tweaks::snapshot::durable_write(dir, path, &json, true)
}

/// `--log-persist=0|1` and `--log-detailed=0|1`, for this session only: an elevation under another
/// account reads that account's settings file, not the user's.
pub fn with_overrides(
    mut settings: Settings,
    args: impl IntoIterator<Item = OsString>,
) -> Settings {
    for arg in args {
        let Some(arg) = arg.to_str() else { continue };
        let flag = |prefix: &str| match arg.strip_prefix(prefix) {
            Some("1") => Some(true),
            Some("0") => Some(false),
            _ => None,
        };
        if let Some(on) = flag(PERSIST_ARG) {
            settings.persist = on;
        }
        if let Some(on) = flag(DETAILED_ARG) {
            settings.detailed = on;
        }
    }
    settings
}

pub fn override_args(settings: Settings) -> [String; 2] {
    [
        format!("{PERSIST_ARG}{}", u8::from(settings.persist)),
        format!("{DETAILED_ARG}{}", u8::from(settings.detailed)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::fs::OpenOptionsExt;

    #[test]
    fn a_missing_file_is_the_defaults_and_is_not_created() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        assert_eq!(
            load(&path),
            Loaded {
                settings: Settings {
                    persist: true,
                    detailed: false
                },
                error: None
            }
        );
        assert!(!path.exists());
    }

    #[test]
    fn a_corrupt_file_runs_memory_only_and_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        std::fs::write(&path, b"{ not json").unwrap();
        let loaded = load(&path);
        assert!(!loaded.settings.persist);
        assert!(loaded.error.is_some());
        assert_eq!(std::fs::read(&path).unwrap(), b"{ not json");
    }

    #[test]
    fn a_hand_edited_file_with_a_bom_or_a_missing_field_loads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        for (text, persist, detailed) in [
            (&b"{\"persist\":false}"[..], false, false),
            (b"{\"detailed\":true}", true, true),
            (
                b"\xEF\xBB\xBF{\"persist\":false,\"detailed\":true}",
                false,
                true,
            ),
            (b"\xEF\xBB\xBF{}", true, false),
        ] {
            std::fs::write(&path, text).unwrap();
            assert_eq!(
                load(&path),
                Loaded {
                    settings: Settings { persist, detailed },
                    error: None
                },
                "{}",
                String::from_utf8_lossy(text)
            );
        }
    }

    #[test]
    fn a_failed_save_keeps_the_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        save(&path, Settings::default()).unwrap();
        let before = std::fs::read(&path).unwrap();
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        let wanted = Settings {
            persist: false,
            detailed: true,
        };
        assert!(save(&path, wanted).is_err());
        drop(held);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        save(&path, wanted).unwrap();
        assert_eq!(load(&path).settings, wanted);
    }

    #[test]
    fn a_saved_file_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join(FILE_NAME);
        let wanted = Settings {
            persist: false,
            detailed: true,
        };
        save(&path, wanted).unwrap();
        assert_eq!(load(&path).settings, wanted);
    }

    #[test]
    fn overrides_apply_and_round_trip_through_the_restart_args() {
        let base = Settings::default();
        let args = |list: &[&str]| list.iter().map(OsString::from).collect::<Vec<_>>();
        assert_eq!(
            with_overrides(
                base,
                args(&[
                    "--after-restart=1234",
                    "--log-persist=0",
                    "--log-detailed=1"
                ])
            ),
            Settings {
                persist: false,
                detailed: true
            }
        );
        assert_eq!(
            with_overrides(base, args(&["--log-persist=yes", "--log-detailed"])),
            base
        );
        let wanted = Settings {
            persist: false,
            detailed: true,
        };
        assert_eq!(
            with_overrides(base, override_args(wanted).map(OsString::from)),
            wanted
        );
    }
}
