//! Tauri commands for app items (ADR-0009). Removal and install run under the same per-id
//! lifecycle lock as an apply, so close, restart and update wait for them.

use rayon::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::apps::{self, install_route, AppPresence, AppsState, InstallRoute, Machine};
use crate::commands::logging::log_outcome;
use crate::commands::tweaks::{
    blocking, compute_availability, current_app_level, next_status_stamp, run_locked, Availability,
};
use crate::error::{Error, Result};
use crate::tweaks::compiled_apps;
use crate::tweaks::engine::context::{self, RealSidProbe, SidCheck};
use crate::tweaks::engine::lifecycle;
use crate::tweaks::model::{AppDef, AppSource, InstallSource, Level, RiskLevel};
use crate::tweaks::validate::scope_admits;
use crate::tweaks::winver::{running_winver, WinVer};

const OUT_OF_SCOPE: &str = "Not available on this Windows build";
const OTHER_ACCOUNT: &str =
    "Another account elevated this app, so this check would read that account's install";

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct InstallView {
    pub kind: InstallKind,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum InstallKind {
    Store,
    Winget,
    StorePage,
}

/// `appx` removes for every account; `script` acts on the running account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum AppSourceKind {
    Appx,
    Script,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct AppView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub info: Option<String>,
    pub warning: Option<String>,
    pub category: String,
    pub risk: RiskLevel,
    pub source: AppSourceKind,
    pub install: Option<InstallView>,
    pub remove_availability: Availability,
    pub install_availability: Availability,
    /// False when this Windows build is outside the app's scope; the UI hides it unless asked.
    pub supported: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct AppStatusView {
    pub app_id: String,
    pub presence: AppPresence,
    pub install_route: InstallRoute,
    pub stamp: u64,
}

fn in_scope(app: &AppDef, winver: &WinVer) -> bool {
    scope_admits(app.windows.as_ref(), &winver.to_milestone())
}

/// Script items read and change the running account's profile; AppX removal is machine-wide.
fn is_script(app: &AppDef) -> bool {
    matches!(app.source, AppSource::Script { .. })
}

fn remove_availability(app: &AppDef, level: Level, sid: SidCheck) -> Availability {
    app_wording(compute_availability(
        is_script(app),
        Level::Admin,
        level,
        sid,
        None,
    ))
}

/// An install lands in the running account, so another account's elevation must block it.
fn install_availability(level: Level, sid: SidCheck) -> Availability {
    app_wording(compute_availability(true, Level::User, level, sid, None))
}

fn app_wording(availability: Availability) -> Availability {
    let reason = |s: &str| s.to_string();
    match availability {
        Availability::NeedsElevation { .. } => Availability::NeedsElevation {
            reason: reason("Restart the app as administrator to remove apps."),
        },
        Availability::SidMismatch { .. } => Availability::SidMismatch {
            reason: reason(
                "Another account elevated this app, so it would change that account's apps. \
                 Restart it under your own account.",
            ),
        },
        Availability::SidUnknown { .. } => Availability::SidUnknown {
            reason: reason(
                "This app could not confirm which account owns this session, so app changes \
                 stay off rather than risk changing the wrong account.",
            ),
        },
        other => other,
    }
}

pub(super) fn app_view(app: &AppDef, level: Level, sid: SidCheck, winver: &WinVer) -> AppView {
    AppView {
        id: app.id.clone(),
        name: app.name.clone(),
        description: app.description.clone(),
        info: app.info.clone(),
        warning: app.warning.clone(),
        category: app.category.clone(),
        risk: app.risk_level,
        supported: in_scope(app, winver),
        source: if is_script(app) {
            AppSourceKind::Script
        } else {
            AppSourceKind::Appx
        },
        install: app.install.as_ref().map(|i| match i {
            InstallSource::Store(id) => InstallView {
                kind: InstallKind::Store,
                id: id.clone(),
            },
            InstallSource::Winget(id) => InstallView {
                kind: InstallKind::Winget,
                id: id.clone(),
            },
            InstallSource::StorePage(id) => InstallView {
                kind: InstallKind::StorePage,
                id: id.clone(),
            },
        }),
        remove_availability: remove_availability(app, level, sid),
        install_availability: install_availability(level, sid),
    }
}

fn route(app: &AppDef, m: &dyn Machine) -> InstallRoute {
    install_route(
        app.install.as_ref(),
        m.winget_available(),
        m.store_available(),
    )
}

/// One status per app, skipping any mid-removal or mid-install: its presence is changing.
fn scan(
    apps: &[AppDef],
    m: &dyn Machine,
    winver: &WinVer,
    sid: SidCheck,
    is_locked: &(dyn Fn(&str) -> bool + Sync),
) -> Vec<AppStatusView> {
    m.invalidate();
    let (winget, store) = (m.winget_available(), m.store_available());
    // Build the index once up front: concurrent first lookups would each spawn an enumeration.
    if apps
        .iter()
        .any(|a| in_scope(a, winver) && matches!(a.source, AppSource::Appx(_)))
    {
        if let Err(e) = m.appx(&[]) {
            log::warn!("app scan: {e}");
        }
    }
    apps.par_iter()
        .filter_map(|app| {
            // Stamped before the check: a removal that locks after it stamps higher and wins.
            let stamp = next_status_stamp();
            if is_locked(&app.id) {
                log::debug!("skipping app {} in the scan: a change is in flight", app.id);
                return None;
            }
            let unknown = |reason: &str| AppPresence::Unknown {
                reason: reason.to_string(),
                needs_elevation: false,
            };
            let presence = if !in_scope(app, winver) {
                unknown(OUT_OF_SCOPE)
            } else if is_script(app) && sid.blocks_hkcu() {
                unknown(OTHER_ACCOUNT)
            } else {
                apps::presence(app, m)
            };
            Some(AppStatusView {
                app_id: app.id.clone(),
                presence,
                install_route: install_route(app.install.as_ref(), winget, store),
                stamp,
            })
        })
        .collect()
}

fn refusal(app: &AppDef, availability: Availability, winver: &WinVer) -> Result<()> {
    if !in_scope(app, winver) {
        return Err(Error::AppUnavailable(format!(
            "{}: {OUT_OF_SCOPE}.",
            app.name
        )));
    }
    match availability {
        Availability::Available => Ok(()),
        Availability::NeedsElevation { reason }
        | Availability::SidMismatch { reason }
        | Availability::SidUnknown { reason }
        | Availability::ElevationPathUnavailable { reason } => Err(Error::AppUnavailable(reason)),
    }
}

async fn gate(app: &'static AppDef, installing: bool) -> Result<()> {
    blocking(move || {
        let (level, sid) = (current_app_level(), context::sid_check(&RealSidProbe));
        let availability = if installing {
            install_availability(level, sid)
        } else {
            remove_availability(app, level, sid)
        };
        refusal(app, availability, &running_winver())
    })
    .await
}

fn find_app(app_id: &str) -> Result<&'static AppDef> {
    compiled_apps()
        .iter()
        .find(|a| a.id == app_id)
        .ok_or_else(|| Error::NotFound(format!("app '{app_id}'")))
}

/// Runs a removal or install under the app's lock, with the taskbar showing it.
async fn change_tracked(
    handle: AppHandle,
    app: &'static AppDef,
    change: fn(&AppDef, &dyn Machine) -> Result<AppPresence>,
) -> Result<AppStatusView> {
    let host = handle.clone();
    let work = run_locked(&app.id, move || {
        let stamp = next_status_stamp();
        let state = handle.state::<AppsState>();
        let m = state.machine();
        let presence = change(app, &m)?;
        Ok(AppStatusView {
            app_id: app.id.clone(),
            presence,
            install_route: route(app, &m),
            stamp,
        })
    });
    crate::taskbar::track(&host, work).await
}

/// `remove_app`'s whole path for any definition, so the test build's manual test runs it too.
pub(crate) async fn remove_gated(handle: AppHandle, app: &'static AppDef) -> Result<AppStatusView> {
    gate(app, false).await?;
    change_tracked(handle, app, apps::remove).await
}

/// `install_app`'s whole path, as [`remove_gated`].
pub(crate) async fn install_gated(
    handle: AppHandle,
    app: &'static AppDef,
) -> Result<AppStatusView> {
    gate(app, true).await?;
    change_tracked(handle, app, apps::install).await
}

#[tauri::command]
pub async fn get_apps() -> Result<Vec<AppView>> {
    log::info!("get_apps: building the app item view for the UI");
    blocking(|| {
        let (level, sid) = (current_app_level(), context::sid_check(&RealSidProbe));
        let winver = running_winver();
        Ok(compiled_apps()
            .iter()
            .map(|a| app_view(a, level, sid, &winver))
            .collect())
    })
    .await
}

#[tauri::command]
pub async fn get_app_statuses(app: AppHandle) -> Result<Vec<AppStatusView>> {
    log::info!("get_app_statuses: scanning app presence");
    blocking(move || {
        let state = app.state::<AppsState>();
        Ok(scan(
            compiled_apps(),
            &state.machine(),
            &running_winver(),
            context::sid_check(&RealSidProbe),
            &lifecycle::is_locked,
        ))
    })
    .await
}

/// For the outcome line: what the app was left as.
fn presence_summary(view: &AppStatusView) -> String {
    match view.presence {
        AppPresence::Installed {
            provisioned_only: false,
        } => "installed",
        AppPresence::Installed {
            provisioned_only: true,
        } => "provisioned only",
        AppPresence::Absent => "absent",
        AppPresence::Unknown { .. } => "unknown",
    }
    .to_string()
}

#[tauri::command]
pub async fn remove_app(app: AppHandle, app_id: String) -> Result<AppStatusView> {
    log::info!("remove_app: '{app_id}'");
    let started = std::time::Instant::now();
    let result = async { remove_gated(app, find_app(&app_id)?).await }.await;
    log_outcome(
        &format!("remove '{app_id}'"),
        started,
        &result,
        presence_summary,
    );
    result
}

#[tauri::command]
pub async fn install_app(app: AppHandle, app_id: String) -> Result<AppStatusView> {
    log::info!("install_app: '{app_id}'");
    let started = std::time::Instant::now();
    let result = async { install_gated(app, find_app(&app_id)?).await }.await;
    log_outcome(
        &format!("install '{app_id}'"),
        started,
        &result,
        presence_summary,
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::fake::{lookup, FakeMachine};
    use crate::tweaks::model::{BuildExpr, WindowsScope};

    fn app(id: &str, source: AppSource) -> AppDef {
        AppDef {
            id: id.into(),
            name: id.into(),
            description: "d".into(),
            category: "apps".into(),
            info: None,
            warning: None,
            risk_level: RiskLevel::Low,
            windows: None,
            source,
            install: Some(InstallSource::Store("9NBLGGH4R32N".into())),
        }
    }

    fn appx() -> AppSource {
        AppSource::Appx(vec!["Vendor.App".into()])
    }

    fn script() -> AppSource {
        AppSource::Script {
            probe: "p".into(),
            remove: "r".into(),
            timeout: None,
        }
    }

    fn win11() -> WinVer {
        WinVer {
            build: 26100,
            revision: 0,
        }
    }

    fn win10_only(mut def: AppDef) -> AppDef {
        def.windows = Some(WindowsScope {
            products: None,
            build: Some(BuildExpr::Max(19045)),
            revision: None,
        });
        def
    }

    #[test]
    fn remove_needs_admin_and_a_script_removal_needs_the_session_owner() {
        let appx_app = app("a", appx());
        let script_app = app("s", script());
        assert!(matches!(
            remove_availability(&appx_app, Level::User, SidCheck::SameUser),
            Availability::NeedsElevation { .. }
        ));
        assert_eq!(
            remove_availability(&appx_app, Level::Admin, SidCheck::DifferentUser),
            Availability::Available
        );
        assert!(matches!(
            remove_availability(&script_app, Level::Admin, SidCheck::DifferentUser),
            Availability::SidMismatch { .. }
        ));
        assert!(matches!(
            remove_availability(&script_app, Level::Admin, SidCheck::Undetermined),
            Availability::SidUnknown { .. }
        ));
        assert_eq!(
            remove_availability(&script_app, Level::Admin, SidCheck::SameUser),
            Availability::Available
        );
    }

    #[test]
    fn install_runs_unelevated_but_never_for_another_account() {
        assert_eq!(
            install_availability(Level::User, SidCheck::SameUser),
            Availability::Available
        );
        assert_eq!(
            install_availability(Level::Admin, SidCheck::SameUser),
            Availability::Available
        );
        assert!(matches!(
            install_availability(Level::Admin, SidCheck::DifferentUser),
            Availability::SidMismatch { .. }
        ));
        assert!(matches!(
            install_availability(Level::Admin, SidCheck::Undetermined),
            Availability::SidUnknown { .. }
        ));
    }

    #[test]
    fn the_gate_refuses_unavailable_and_out_of_scope_apps() {
        let def = app("a", appx());
        assert!(refusal(&def, Availability::Available, &win11()).is_ok());
        let refused = refusal(
            &def,
            remove_availability(&def, Level::User, SidCheck::SameUser),
            &win11(),
        );
        assert!(matches!(refused, Err(Error::AppUnavailable(_))));
        assert!(matches!(
            refusal(&win10_only(def), Availability::Available, &win11()),
            Err(Error::AppUnavailable(_))
        ));
    }

    #[test]
    fn a_status_scan_skips_locked_ids_and_invalidates_first() {
        let apps = [app("busy", appx()), app("idle", appx())];
        let m = FakeMachine {
            elevated: true,
            store: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(true, false))]);
        let statuses = scan(&apps, &m, &win11(), SidCheck::SameUser, &|id: &str| {
            id == "busy"
        });
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].app_id, "idle");
        assert_eq!(
            statuses[0].presence,
            AppPresence::Installed {
                provisioned_only: false
            }
        );
        assert_eq!(statuses[0].install_route, InstallRoute::StorePage);
        assert_eq!(m.invalidated(), 1);
    }

    #[test]
    fn an_out_of_scope_app_reads_unknown_without_probing() {
        let apps = [win10_only(app("old", script()))];
        let m = FakeMachine::default();
        let shown = scan(&apps, &m, &win11(), SidCheck::SameUser, &|_: &str| false);
        assert!(matches!(
            &shown[0].presence,
            AppPresence::Unknown { reason, .. } if reason == OUT_OF_SCOPE
        ));
        assert!(m.ran().is_empty(), "an out-of-scope probe must not run");
    }

    #[test]
    fn app_view_carries_the_wire_shape() {
        let mut def = app("a", appx());
        def.install = Some(InstallSource::StorePage("9NBLGGH4R32N".into()));
        let json = serde_json::to_value(app_view(&def, Level::User, SidCheck::SameUser, &win11()))
            .unwrap();
        assert_eq!(
            json["install"],
            serde_json::json!({ "kind": "store_page", "id": "9NBLGGH4R32N" })
        );
        assert_eq!(json["risk"], "Low");
        assert_eq!(json["source"], "appx");
        assert_eq!(json["remove_availability"]["state"], "needs_elevation");
        assert_eq!(json["install_availability"]["state"], "available");
        assert_eq!(json["supported"], true);
        let old = win10_only(app("old", script()));
        assert!(!app_view(&old, Level::User, SidCheck::SameUser, &win11()).supported);
    }

    #[test]
    fn a_script_item_reads_unknown_when_another_account_elevated_the_app() {
        let apps = [app("od", script()), app("pkg", appx())];
        let m = FakeMachine {
            elevated: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(true, false))]);
        let statuses = scan(&apps, &m, &win11(), SidCheck::DifferentUser, &|_: &str| {
            false
        });
        let od = statuses.iter().find(|s| s.app_id == "od").unwrap();
        assert!(matches!(
            &od.presence,
            AppPresence::Unknown { reason, .. } if reason == OTHER_ACCOUNT
        ));
        assert!(m.ran().is_empty(), "the per-user probe must not run");
        let pkg = statuses.iter().find(|s| s.app_id == "pkg").unwrap();
        assert!(matches!(pkg.presence, AppPresence::Installed { .. }));
    }
}
