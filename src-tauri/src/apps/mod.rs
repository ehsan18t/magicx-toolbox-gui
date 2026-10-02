//! App items (ADR-0009): presence, removal and install of curated apps. No options, no snapshot.

mod run;

pub(crate) use run::{install, presence, remove};

use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;

use crate::error::Error;
use crate::services::appx_index::{AppxIndex, AppxLookup};
use crate::services::system_info_service;
use crate::tweaks::kinds::action::{run_script, ScriptRun};
use crate::tweaks::model::{InstallSource, Shell};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AppPresence {
    Installed {
        provisioned_only: bool,
    },
    Absent,
    Unknown {
        reason: String,
        needs_elevation: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallRoute {
    Winget,
    StorePage,
    None,
}

/// Everything presence, removal and install touch on the machine.
pub(crate) trait Machine: Sync {
    fn elevated(&self) -> bool;
    fn appx(&self, packages: &[String]) -> Result<AppxLookup, Error>;
    fn invalidate(&self);
    fn powershell(&self, body: &str, timeout: Duration) -> Result<ScriptRun, Error>;
    fn winget_available(&self) -> bool;
    fn store_available(&self) -> bool;
}

/// App-lifetime state. The index scope is fixed at startup: elevation never changes in-process.
pub struct AppsState {
    elevated: bool,
    appx: AppxIndex,
}

impl AppsState {
    pub fn new() -> Self {
        let elevated = system_info_service::is_running_as_admin();
        Self {
            elevated,
            appx: if elevated {
                AppxIndex::default()
            } else {
                AppxIndex::current_user()
            },
        }
    }

    pub(crate) fn machine(&self) -> RealMachine<'_> {
        RealMachine(self)
    }
}

pub(crate) struct RealMachine<'a>(&'a AppsState);

impl Machine for RealMachine<'_> {
    fn elevated(&self) -> bool {
        self.0.elevated
    }

    fn appx(&self, packages: &[String]) -> Result<AppxLookup, Error> {
        self.0.appx.lookup(packages)
    }

    fn invalidate(&self) {
        self.0.appx.invalidate();
    }

    fn powershell(&self, body: &str, timeout: Duration) -> Result<ScriptRun, Error> {
        run_script(Shell::PowerShell, body, timeout)
            .map_err(|e| Error::CommandExecution(e.to_string()))
    }

    fn winget_available(&self) -> bool {
        // An app execution alias: a reparse point `metadata` cannot always follow.
        winget_path().is_some_and(|p| std::fs::symlink_metadata(p).is_ok())
    }

    fn store_available(&self) -> bool {
        use winreg::enums::{HKEY_CLASSES_ROOT, KEY_READ};
        winreg::RegKey::predef(HKEY_CLASSES_ROOT)
            .open_subkey_with_flags("ms-windows-store", KEY_READ)
            .is_ok()
    }
}

/// The running account's winget, the same path the generated install script runs.
fn winget_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|dir| PathBuf::from(dir).join(r"Microsoft\WindowsApps\winget.exe"))
}

pub(crate) fn install_route(
    install: Option<&InstallSource>,
    winget: bool,
    store: bool,
) -> InstallRoute {
    match install {
        Some(InstallSource::Store(_) | InstallSource::Winget(_)) if winget => InstallRoute::Winget,
        Some(InstallSource::Store(_) | InstallSource::StorePage(_)) if store => {
            InstallRoute::StorePage
        }
        _ => InstallRoute::None,
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    /// Answers are consumed in order; the last one repeats.
    #[derive(Default)]
    pub(crate) struct FakeMachine {
        pub elevated: bool,
        pub winget: bool,
        pub store: bool,
        pub appx: Mutex<VecDeque<Result<AppxLookup, String>>>,
        pub exits: Mutex<VecDeque<Result<i32, String>>>,
        pub scripts: Mutex<Vec<(String, Duration)>>,
        pub invalidations: AtomicUsize,
    }

    fn next<T: Clone>(queue: &Mutex<VecDeque<Result<T, String>>>) -> Result<T, Error> {
        let mut queue = queue.lock().unwrap();
        let answer = if queue.len() > 1 {
            queue.pop_front()
        } else {
            queue.front().cloned()
        };
        answer
            .expect("the fake was asked something it has no answer for")
            .map_err(Error::CommandExecution)
    }

    pub(crate) fn lookup(registered: bool, provisioned: bool) -> AppxLookup {
        AppxLookup {
            registered,
            provisioned,
        }
    }

    impl FakeMachine {
        pub fn with_appx(self, answers: Vec<Result<AppxLookup, String>>) -> Self {
            *self.appx.lock().unwrap() = answers.into();
            self
        }

        pub fn with_exits(self, answers: Vec<Result<i32, String>>) -> Self {
            *self.exits.lock().unwrap() = answers.into();
            self
        }

        pub fn ran(&self) -> Vec<(String, Duration)> {
            self.scripts.lock().unwrap().clone()
        }

        pub fn invalidated(&self) -> usize {
            self.invalidations.load(Ordering::SeqCst)
        }
    }

    impl Machine for FakeMachine {
        fn elevated(&self) -> bool {
            self.elevated
        }
        fn appx(&self, _packages: &[String]) -> Result<AppxLookup, Error> {
            next(&self.appx)
        }
        fn invalidate(&self) {
            self.invalidations.fetch_add(1, Ordering::SeqCst);
        }
        fn powershell(&self, body: &str, timeout: Duration) -> Result<ScriptRun, Error> {
            self.scripts
                .lock()
                .unwrap()
                .push((body.to_string(), timeout));
            next(&self.exits).map(|code| ScriptRun {
                code,
                tail: String::new(),
            })
        }
        fn winget_available(&self) -> bool {
            self.winget
        }
        fn store_available(&self) -> bool {
            self.store
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_route_matrix() {
        let store = InstallSource::Store("9NBLGGH4R32N".into());
        let winget = InstallSource::Winget("Vendor.App".into());
        let page = InstallSource::StorePage("9NBLGGH4R32N".into());
        let cases = [
            (Some(&store), true, true, InstallRoute::Winget),
            (Some(&store), true, false, InstallRoute::Winget),
            (Some(&store), false, true, InstallRoute::StorePage),
            (Some(&store), false, false, InstallRoute::None),
            (Some(&winget), true, true, InstallRoute::Winget),
            (Some(&winget), true, false, InstallRoute::Winget),
            (Some(&winget), false, true, InstallRoute::None),
            (Some(&winget), false, false, InstallRoute::None),
            (Some(&page), true, true, InstallRoute::StorePage),
            (Some(&page), false, true, InstallRoute::StorePage),
            (Some(&page), true, false, InstallRoute::None),
            (None, true, true, InstallRoute::None),
        ];
        for (install, has_winget, has_store, expected) in cases {
            assert_eq!(
                install_route(install, has_winget, has_store),
                expected,
                "{install:?} winget={has_winget} store={has_store}"
            );
        }
    }

    #[test]
    fn presence_serializes_to_the_wire_shape() {
        let json = |p: AppPresence| serde_json::to_value(p).unwrap();
        assert_eq!(
            json(AppPresence::Installed {
                provisioned_only: true
            }),
            serde_json::json!({ "state": "installed", "provisioned_only": true })
        );
        assert_eq!(
            json(AppPresence::Absent),
            serde_json::json!({ "state": "absent" })
        );
        assert_eq!(
            json(AppPresence::Unknown {
                reason: "r".into(),
                needs_elevation: true
            }),
            serde_json::json!({ "state": "unknown", "reason": "r", "needs_elevation": true })
        );
        assert_eq!(
            serde_json::to_value(InstallRoute::StorePage).unwrap(),
            "store_page"
        );
    }
}
