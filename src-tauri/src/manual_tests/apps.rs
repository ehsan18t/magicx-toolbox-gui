use std::sync::LazyLock;
use std::time::Instant;

use super::errors;
use super::runner::{Ctx, Verdict};
use crate::apps::{self, AppPresence, InstallRoute, Machine};
use crate::services::system_info_service;
use crate::tweaks::model::{AppDef, AppSource, InstallSource, RiskLevel};

/// Not a corpus item: the test owns its definition so it runs on any build.
static FEEDBACK_HUB: LazyLock<AppDef> = LazyLock::new(|| AppDef {
    id: "manual_test_feedback_hub".into(),
    name: "Feedback Hub".into(),
    description: "Manual test target.".into(),
    category: "manual_tests".into(),
    info: None,
    warning: None,
    risk_level: RiskLevel::Low,
    windows: None,
    source: AppSource::Appx(vec!["Microsoft.WindowsFeedbackHub".into()]),
    install: Some(InstallSource::Store("9NBLGGH4R32N".into())),
});

pub fn feedback_hub_round_trip(cx: &Ctx) -> Verdict {
    let app: &'static AppDef = &FEEDBACK_HUB;
    if !system_info_service::is_running_as_admin() {
        return Verdict::fail("Refused: the app is not elevated; restart it as administrator.");
    }
    let m = cx.host.apps().machine();
    let route = apps::install_route(
        app.install.as_ref(),
        m.winget_available(),
        m.store_available(),
    );
    cx.info(format!("install route = {route:?}"));
    if route != InstallRoute::Winget {
        return Verdict::fail(
            "Refused: winget is not available for this account, so Feedback Hub could not be put back. Nothing was removed.",
        );
    }
    m.invalidate();
    let before = apps::presence(app, &m);
    cx.info(format!("presence before = {before:?}"));
    if !matches!(before, AppPresence::Installed { .. }) {
        return Verdict::fail(
            "Refused: Feedback Hub is not installed here. Install it from the Microsoft Store, then run this again.",
        );
    }

    let t0 = Instant::now();
    let removed = cx.host.remove_app(app);
    let remove_ms = t0.elapsed().as_millis();
    match &removed {
        Ok(status) => cx.info(format!("remove -> {:?} ({remove_ms} ms)", status.presence)),
        Err(e) => {
            cx.error(format!("remove failed: {}", errors::app(e)));
            return Verdict::fail(format!("Removal failed after {remove_ms} ms: {e}"));
        }
    }

    let t1 = Instant::now();
    let installed = cx.host.install_app(app);
    let install_ms = t1.elapsed().as_millis();
    match &installed {
        Ok(status) => {
            cx.info(format!(
                "install -> {:?} ({install_ms} ms)",
                status.presence
            ));
            Verdict::pass(format!(
                "Removed in {remove_ms} ms and verified absent; reinstalled through winget in {install_ms} ms and verified installed."
            ))
        }
        Err(e) => {
            cx.error(format!("install failed: {}", errors::app(e)));
            Verdict::fail(format!(
                "Removal verified, but the reinstall failed after {install_ms} ms: {e}. Install Feedback Hub from the Microsoft Store to put it back."
            ))
        }
    }
}
