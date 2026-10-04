use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::{install_route, AppPresence, InstallRoute, Machine};
use crate::error::Error;
use crate::services::appx_index::AppxLookup;
use crate::tweaks::kinds::action::ACTION_TIMEOUT;
use crate::tweaks::model::{AppDef, AppSource, InstallSource};
use crate::tweaks::validate::{is_appx_name, is_store_id, is_winget_id};

/// For a removal whose item sets no `timeout`, AppX included.
const REMOVE_TIMEOUT: Duration = Duration::from_secs(600);
const INSTALL_TIMEOUT: Duration = Duration::from_secs(1800);

/// `$_.Exception.HResult` names the deployment failure (0x80073CFA and friends); 0 is remapped so
/// a failure can never read as success.
const EXIT_ON_ERROR: &str = "} catch {\n    $h = $_.Exception.HResult\n    if ($h -eq 0) { $h = 1 }\n    exit $h\n}\nexit 0\n";

/// Never touches the index cache: callers invalidate first, since failures are cached too.
pub(crate) fn presence(app: &AppDef, m: &dyn Machine) -> AppPresence {
    let presence = match &app.source {
        AppSource::Appx(packages) => appx_presence(m.appx(packages), m.elevated()),
        AppSource::Script { probe, .. } => script_presence(
            m.powershell(probe, ACTION_TIMEOUT).map(|r| r.code),
            m.elevated(),
        ),
    };
    if let AppPresence::Unknown { reason, .. } = &presence {
        log::warn!("app '{}': presence unknown: {reason}", app.id);
    }
    presence
}

fn appx_presence(lookup: Result<AppxLookup, Error>, elevated: bool) -> AppPresence {
    match lookup {
        Ok(l) if l.registered => AppPresence::Installed {
            provisioned_only: false,
        },
        Ok(l) if l.provisioned => AppPresence::Installed {
            provisioned_only: true,
        },
        Ok(_) if elevated => AppPresence::Absent,
        // Unelevated, only this account's packages are listed: a miss proves nothing.
        Ok(_) => AppPresence::Unknown {
            reason: "Without administrator rights only this account's apps can be checked.".into(),
            needs_elevation: true,
        },
        Err(e) => {
            log::warn!("app presence: {e}");
            AppPresence::Unknown {
                reason: "The installed app list could not be read.".into(),
                needs_elevation: false,
            }
        }
    }
}

fn script_presence(exit: Result<i32, Error>, elevated: bool) -> AppPresence {
    match exit {
        Ok(0) => AppPresence::Installed {
            provisioned_only: false,
        },
        Ok(2) => AppPresence::Absent,
        Ok(code) => AppPresence::Unknown {
            reason: format!("The presence check ended with {}.", describe_exit(code)),
            needs_elevation: !elevated,
        },
        Err(e) => {
            log::warn!("app presence check: {e}");
            AppPresence::Unknown {
                reason: "The presence check did not finish.".into(),
                needs_elevation: false,
            }
        }
    }
}

/// Small codes in decimal, HRESULTs in hex.
fn describe_exit(code: i32) -> String {
    if (0..=0xFFFF).contains(&code) {
        format!("exit code {code}")
    } else {
        format!("error 0x{:08X}", code as u32)
    }
}

pub(crate) fn remove(app: &AppDef, m: &dyn Machine) -> Result<AppPresence, Error> {
    let (body, timeout) = match &app.source {
        AppSource::Appx(packages) => (appx_remove_script(packages)?, REMOVE_TIMEOUT),
        AppSource::Script {
            remove, timeout, ..
        } => (
            remove.clone(),
            timeout.map_or(REMOVE_TIMEOUT, |s| Duration::from_secs(s.into())),
        ),
    };
    let run = m.powershell(&body, timeout).map_err(|e| {
        log::error!("removing app '{}': {e}", app.id);
        Error::AppFailed(format!("Removing {} did not finish: {e}", app.name))
    })?;
    let code = run.code;
    if code != 0 {
        run.warn_on_failure(&format!("removing app '{}'", app.id));
        return Err(Error::AppFailed(format!(
            "Removing {} failed ({}).",
            app.name,
            describe_exit(code)
        )));
    }
    m.invalidate();
    match presence(app, m) {
        AppPresence::Absent => Ok(AppPresence::Absent),
        AppPresence::Installed { .. } if matches!(app.source, AppSource::Appx(_)) => {
            Err(Error::AppFailed(format!(
                "{} is still registered after the removal. Another signed-in account may still hold it: sign that account out, then check again.",
                app.name
            )))
        }
        AppPresence::Installed { .. } => Err(Error::AppFailed(format!(
            "{} is still installed after its removal reported success.",
            app.name
        ))),
        AppPresence::Unknown { reason, .. } => Err(Error::AppFailed(format!(
            "{} was removed, but its absence could not be confirmed: {reason}",
            app.name
        ))),
    }
}

pub(crate) fn install(
    app: &AppDef,
    m: &dyn Machine,
    cancel: &AtomicBool,
) -> Result<AppPresence, Error> {
    let route = install_route(
        app.install.as_ref(),
        m.winget_available(),
        m.store_available(),
    );
    let (id, source) = match (&app.install, route) {
        (Some(InstallSource::Store(id)), InstallRoute::Winget) => (id, "msstore"),
        (Some(InstallSource::Winget(id)), InstallRoute::Winget) => (id, "winget"),
        _ => {
            return Err(Error::AppUnavailable(format!(
                "{} cannot be installed by this app on this PC.",
                app.name
            )))
        }
    };
    if cancel.load(Ordering::SeqCst) {
        return Err(cancelled(app));
    }
    let script = winget_install_script(id, source)?;
    let run = match m.powershell_until(&script, INSTALL_TIMEOUT, cancel) {
        Ok(run) => run,
        Err(e) => {
            stop_store_install(app, (source == "msstore").then_some(id.as_str()), m)?;
            if cancel.load(Ordering::SeqCst) {
                return verify_cancel(app, m);
            }
            log::error!("installing app '{}': {e}", app.id);
            return Err(Error::AppFailed(format!(
                "Installing {} did not finish: {e}",
                app.name
            )));
        }
    };
    let code = run.code;
    if code != 0 {
        run.warn_on_failure(&format!("installing app '{}' with winget", app.id));
        return Err(Error::AppFailed(format!(
            "Installing {} failed: winget ended with {}.",
            app.name,
            describe_exit(code)
        )));
    }
    m.invalidate();
    match presence(app, m) {
        installed @ AppPresence::Installed { .. } => Ok(installed),
        AppPresence::Absent => Err(Error::AppFailed(format!(
            "winget finished, but {} is still not installed.",
            app.name
        ))),
        AppPresence::Unknown { reason, .. } => Err(Error::AppFailed(format!(
            "{} was installed, but its presence could not be confirmed: {reason}",
            app.name
        ))),
    }
}

fn cancelled(app: &AppDef) -> Error {
    Error::AppCancelled(format!("Installing {} was cancelled.", app.name))
}

/// After winget is killed, by a cancel or the timeout: a Store install runs on in the Store's own
/// service, so it is cancelled there too.
fn stop_store_install(app: &AppDef, store_id: Option<&str>, m: &dyn Machine) -> Result<(), Error> {
    if let Some(id) = store_id {
        let run = m
            .powershell(&store_cancel_script(id)?, ACTION_TIMEOUT)
            .map_err(|e| {
                log::error!("cancelling the Store install of app '{}': {e}", app.id);
                Error::AppFailed(format!(
                    "winget was stopped, but the Microsoft Store may still be installing {}: {e}",
                    app.name
                ))
            })?;
        if run.code != 0 {
            run.warn_on_failure(&format!("cancelling the Store install of app '{}'", app.id));
            return Err(Error::AppFailed(format!(
                "winget was stopped, but the Microsoft Store may still be installing {} ({}).",
                app.name,
                describe_exit(run.code)
            )));
        }
    }
    Ok(())
}

/// Whatever presence reads after a cancel is the outcome; an install that finished first stands.
fn verify_cancel(app: &AppDef, m: &dyn Machine) -> Result<AppPresence, Error> {
    log::info!("install of app '{}' cancelled", app.id);
    m.invalidate();
    match presence(app, m) {
        installed @ AppPresence::Installed { .. } => Ok(installed),
        AppPresence::Absent => Err(cancelled(app)),
        AppPresence::Unknown { reason, .. } => Err(Error::AppFailed(format!(
            "Installing {} was cancelled, but whether part of it remains could not be confirmed: {reason}",
            app.name
        ))),
    }
}

/// `Cancel` throws E_BOUNDS (0x8000000B) when nothing is queued. Exit 3: still queued after the wait.
const STORE_CANCEL_SCRIPT: &str = r"$ErrorActionPreference = 'Stop'
try {
    $null = [Windows.ApplicationModel.Store.Preview.InstallControl.AppInstallManager,Windows.ApplicationModel.Store.Preview,ContentType=WindowsRuntime]
    $m = [Windows.ApplicationModel.Store.Preview.InstallControl.AppInstallManager]::new()
    try { $m.Cancel('{id}') } catch { if ($_.Exception.InnerException.HResult -ne 0x8000000B) { throw } }
    $deadline = (Get-Date).AddSeconds(20)
    while (@($m.AppInstallItems | Where-Object { $_.ProductId -eq '{id}' -and $_.GetCurrentStatus().InstallState -notin 'Completed', 'Canceled', 'Error' }).Count) {
        if ((Get-Date) -gt $deadline) { exit 3 }
        Start-Sleep -Milliseconds 500
    }
} catch {
    $e = $_.Exception
    while ($e.InnerException) { $e = $e.InnerException }
    $h = $e.HResult
    if ($h -eq 0) { $h = 1 }
    exit $h
}
exit 0
";

fn store_cancel_script(id: &str) -> Result<String, Error> {
    if !is_store_id(id) {
        return Err(Error::AppFailed(format!(
            "{id:?} is not a valid install id."
        )));
    }
    Ok(STORE_CANCEL_SCRIPT.replace("{id}", id))
}

/// Bundle pass, then main, then provisioned, per package. Every name is re-checked against the
/// build-time charset: it is embedded verbatim between single quotes.
fn appx_remove_script(packages: &[String]) -> Result<String, Error> {
    let mut script = String::from("$ErrorActionPreference = 'Stop'\ntry {\n");
    for name in packages {
        if !is_appx_name(name) {
            return Err(Error::AppFailed(format!(
                "{name:?} is not a valid package name."
            )));
        }
        script.push_str(&format!(
            "    Get-AppxPackage -AllUsers -Name '{name}' -PackageTypeFilter Bundle | Remove-AppxPackage -AllUsers\n\
             \x20   Get-AppxPackage -AllUsers -Name '{name}' | Remove-AppxPackage -AllUsers\n\
             \x20   Get-AppxProvisionedPackage -Online | Where-Object DisplayName -eq '{name}' | Remove-AppxProvisionedPackage -Online\n"
        ));
    }
    script.push_str(EXIT_ON_ERROR);
    Ok(script)
}

fn winget_install_script(id: &str, source: &str) -> Result<String, Error> {
    let valid = match source {
        "msstore" => is_store_id(id),
        _ => is_winget_id(id),
    };
    if !valid {
        return Err(Error::AppFailed(format!(
            "{id:?} is not a valid install id."
        )));
    }
    Ok(format!(
        "$ErrorActionPreference = 'Stop'\ntry {{\n\
         \x20   $winget = Join-Path $env:LOCALAPPDATA 'Microsoft\\WindowsApps\\winget.exe'\n\
         \x20   & $winget install --id '{id}' -e --source {source} --accept-source-agreements --accept-package-agreements\n\
         \x20   exit $LASTEXITCODE\n{EXIT_ON_ERROR}"
    ))
}

#[cfg(test)]
mod tests {
    use super::super::fake::{lookup, FakeMachine};
    use super::*;
    use crate::tweaks::model::RiskLevel;

    fn app(source: AppSource, install: Option<InstallSource>) -> AppDef {
        AppDef {
            id: "feedback_hub".into(),
            name: "Feedback Hub".into(),
            description: "d".into(),
            category: "apps".into(),
            info: None,
            warning: None,
            risk_level: RiskLevel::Low,
            windows: None,
            source,
            install,
        }
    }

    fn appx_app() -> AppDef {
        app(
            AppSource::Appx(vec!["Microsoft.WindowsFeedbackHub".into()]),
            Some(InstallSource::Store("9NBLGGH4R32N".into())),
        )
    }

    fn script_app(timeout: Option<u32>) -> AppDef {
        app(
            AppSource::Script {
                probe: "probe-body".into(),
                remove: "remove-body".into(),
                timeout,
            },
            None,
        )
    }

    fn installed() -> AppPresence {
        AppPresence::Installed {
            provisioned_only: false,
        }
    }

    #[test]
    fn script_exit_codes_map_to_presence() {
        for elevated in [true, false] {
            assert_eq!(script_presence(Ok(0), elevated), installed());
            assert_eq!(script_presence(Ok(2), elevated), AppPresence::Absent);
            for code in [1, 3, -1, 0x8007_0005_u32 as i32] {
                let p = script_presence(Ok(code), elevated);
                assert!(
                    matches!(&p, AppPresence::Unknown { needs_elevation, .. } if *needs_elevation == !elevated),
                    "{code}: {p:?}"
                );
            }
            let timed_out = Err(Error::CommandExecution(
                "action exceeded its 30s timeout and was terminated".into(),
            ));
            assert!(matches!(
                script_presence(timed_out, elevated),
                AppPresence::Unknown { .. }
            ));
        }
    }

    #[test]
    fn the_probe_runs_at_the_fixed_action_timeout() {
        let m = FakeMachine::default().with_exits(vec![Ok(2)]);
        assert_eq!(presence(&script_app(Some(900)), &m), AppPresence::Absent);
        assert_eq!(m.ran(), vec![("probe-body".to_string(), ACTION_TIMEOUT)]);
    }

    #[test]
    fn elevated_appx_presence_reports_provisioned_only() {
        assert_eq!(appx_presence(Ok(lookup(true, true)), true), installed());
        assert_eq!(
            appx_presence(Ok(lookup(false, true)), true),
            AppPresence::Installed {
                provisioned_only: true
            }
        );
        assert_eq!(
            appx_presence(Ok(lookup(false, false)), true),
            AppPresence::Absent
        );
    }

    #[test]
    fn unelevated_fallback_never_reads_a_miss_as_absent() {
        assert_eq!(appx_presence(Ok(lookup(true, false)), false), installed());
        assert!(matches!(
            appx_presence(Ok(lookup(false, false)), false),
            AppPresence::Unknown {
                needs_elevation: true,
                ..
            }
        ));
    }

    #[test]
    fn an_index_failure_is_unknown_never_absent() {
        for elevated in [true, false] {
            let p = appx_presence(Err(Error::CommandExecution("winrt".into())), elevated);
            assert!(
                matches!(
                    p,
                    AppPresence::Unknown {
                        needs_elevation: false,
                        ..
                    }
                ),
                "{p:?}"
            );
        }
    }

    #[test]
    fn a_verified_removal_reads_absent_after_invalidating() {
        let m = FakeMachine {
            elevated: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(false, false))])
        .with_exits(vec![Ok(0)]);
        assert_eq!(remove(&appx_app(), &m).unwrap(), AppPresence::Absent);
        assert_eq!(m.invalidated(), 1);
        assert_eq!(m.ran()[0].1, REMOVE_TIMEOUT);
    }

    #[test]
    fn a_removal_that_still_reads_installed_is_an_error() {
        let m = FakeMachine {
            elevated: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(true, false))])
        .with_exits(vec![Ok(0)]);
        let err = remove(&appx_app(), &m).unwrap_err().to_string();
        assert!(err.contains("Another signed-in account"), "{err}");

        let m = FakeMachine::default().with_exits(vec![Ok(0), Ok(0)]);
        let err = remove(&script_app(None), &m).unwrap_err().to_string();
        assert!(err.contains("still installed"), "{err}");

        let m = FakeMachine {
            elevated: true,
            ..Default::default()
        }
        .with_appx(vec![Err("winrt".into())])
        .with_exits(vec![Ok(0)]);
        let err = remove(&appx_app(), &m).unwrap_err().to_string();
        assert!(err.contains("could not be confirmed"), "{err}");
    }

    #[test]
    fn a_failed_removal_names_its_hresult_and_skips_verification() {
        let m = FakeMachine::default().with_exits(vec![Ok(0x8007_3CFA_u32 as i32)]);
        let err = remove(&appx_app(), &m).unwrap_err().to_string();
        assert!(err.contains("0x80073CFA"), "{err}");
        assert_eq!(m.invalidated(), 0);
    }

    #[test]
    fn a_script_removal_runs_its_own_body_and_timeout() {
        let m = FakeMachine::default().with_exits(vec![Ok(0), Ok(2)]);
        assert_eq!(
            remove(&script_app(Some(600)), &m).unwrap(),
            AppPresence::Absent
        );
        let ran = m.ran();
        assert_eq!(
            ran[0],
            ("remove-body".to_string(), Duration::from_secs(600))
        );
        assert_eq!(ran[1].0, "probe-body");
    }

    #[test]
    fn the_appx_script_embeds_only_validated_literals() {
        let names = vec![
            "Clipchamp.Clipchamp".to_string(),
            "Vendor.App-2".to_string(),
        ];
        let script = appx_remove_script(&names).unwrap();
        let quoted: Vec<&str> = script.split('\'').skip(1).step_by(2).collect();
        assert!(
            quoted
                .iter()
                .all(|q| *q == "Stop" || names.iter().any(|n| n == q)),
            "{quoted:?}"
        );
        for name in &names {
            assert!(script.contains(&format!(
                "Get-AppxPackage -AllUsers -Name '{name}' -PackageTypeFilter Bundle | Remove-AppxPackage -AllUsers"
            )));
            assert!(script.contains(&format!(
                "Get-AppxPackage -AllUsers -Name '{name}' | Remove-AppxPackage -AllUsers"
            )));
            assert!(script.contains(&format!(
                "Where-Object DisplayName -eq '{name}' | Remove-AppxProvisionedPackage -Online"
            )));
        }
        assert!(script.contains("exit $h") && script.contains("$_.Exception.HResult"));

        for bad in ["x'; Remove-Item C:\\ -Recurse; '", "", "a b"] {
            assert!(appx_remove_script(&[bad.to_string()]).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn install_runs_winget_on_the_store_source_and_verifies() {
        let m = FakeMachine {
            elevated: true,
            winget: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(true, false))])
        .with_exits(vec![Ok(0)]);
        assert_eq!(
            install(&appx_app(), &m, &AtomicBool::new(false)).unwrap(),
            installed()
        );
        let (script, timeout) = &m.ran()[0];
        assert!(script.contains(
            "install --id '9NBLGGH4R32N' -e --source msstore --accept-source-agreements --accept-package-agreements"
        ));
        assert_eq!(*timeout, INSTALL_TIMEOUT);
        assert_eq!(m.invalidated(), 1);
    }

    #[test]
    fn install_uses_the_winget_source_for_a_winget_id() {
        let mut def = appx_app();
        def.install = Some(InstallSource::Winget("Vendor.App".into()));
        let m = FakeMachine {
            winget: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(true, false))])
        .with_exits(vec![Ok(0)]);
        install(&def, &m, &AtomicBool::new(false)).unwrap();
        assert!(m.ran()[0]
            .0
            .contains("--id 'Vendor.App' -e --source winget"));
    }

    #[test]
    fn install_refuses_any_route_but_winget_without_running_anything() {
        for (install_source, winget, store) in [
            (
                Some(InstallSource::StorePage("9NBLGGH4R32N".into())),
                true,
                true,
            ),
            (
                Some(InstallSource::Store("9NBLGGH4R32N".into())),
                false,
                true,
            ),
            (None, true, true),
        ] {
            let mut def = appx_app();
            def.install = install_source;
            let m = FakeMachine {
                winget,
                store,
                ..Default::default()
            };
            assert!(matches!(
                install(&def, &m, &AtomicBool::new(false)),
                Err(Error::AppUnavailable(_))
            ));
            assert!(m.ran().is_empty());
        }
    }

    #[test]
    fn an_install_that_does_not_verify_is_an_error() {
        let m = FakeMachine {
            elevated: true,
            winget: true,
            ..Default::default()
        }
        .with_appx(vec![Ok(lookup(false, false))])
        .with_exits(vec![Ok(0)]);
        let err = install(&appx_app(), &m, &AtomicBool::new(false))
            .unwrap_err()
            .to_string();
        assert!(err.contains("still not installed"), "{err}");

        let m = FakeMachine {
            winget: true,
            ..Default::default()
        }
        .with_exits(vec![Ok(0x8A15_0014_u32 as i32)]);
        let err = install(&appx_app(), &m, &AtomicBool::new(false))
            .unwrap_err()
            .to_string();
        assert!(err.contains("0x8A150014"), "{err}");
    }

    fn killed(
        answers: Vec<Result<i32, String>>,
        appx: Vec<Result<AppxLookup, String>>,
    ) -> FakeMachine {
        FakeMachine {
            elevated: true,
            winget: true,
            cancels_in_run: true,
            ..Default::default()
        }
        .with_appx(appx)
        .with_exits(answers)
    }

    #[test]
    fn a_cancelled_store_install_is_cancelled_in_the_store_and_verified_absent() {
        let m = killed(
            vec![Err("killed".into()), Ok(0)],
            vec![Ok(lookup(false, false))],
        );
        let err = install(&appx_app(), &m, &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, Error::AppCancelled(_)), "{err:?}");
        let ran = m.ran();
        assert!(ran[1].0.contains("$m.Cancel('9NBLGGH4R32N')"));
        assert!(ran[1].0.contains("$_.ProductId -eq '9NBLGGH4R32N'"));
        assert_eq!(ran[1].1, ACTION_TIMEOUT);
        assert_eq!(m.invalidated(), 1);
    }

    #[test]
    fn a_cancel_that_lands_after_the_install_finished_reads_installed() {
        let m = killed(
            vec![Err("killed".into()), Ok(0)],
            vec![Ok(lookup(true, false))],
        );
        assert_eq!(
            install(&appx_app(), &m, &AtomicBool::new(false)).unwrap(),
            installed()
        );
    }

    #[test]
    fn a_store_cancel_that_fails_says_the_store_may_still_be_installing() {
        let m = killed(vec![Err("killed".into()), Ok(3)], vec![]);
        let err = install(&appx_app(), &m, &AtomicBool::new(false))
            .unwrap_err()
            .to_string();
        assert!(err.contains("may still be installing"), "{err}");
        assert_eq!(m.invalidated(), 0);
    }

    #[test]
    fn a_cancelled_winget_source_install_skips_the_store() {
        let mut def = appx_app();
        def.install = Some(InstallSource::Winget("Vendor.App".into()));
        let m = killed(vec![Err("killed".into())], vec![Ok(lookup(false, false))]);
        let err = install(&def, &m, &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, Error::AppCancelled(_)), "{err:?}");
        assert_eq!(m.ran().len(), 1);
    }

    #[test]
    fn a_timed_out_store_install_is_cancelled_in_the_store_and_reported_unfinished() {
        let m = FakeMachine {
            winget: true,
            ..Default::default()
        }
        .with_exits(vec![Err("timeout".into()), Ok(0)]);
        let err = install(&appx_app(), &m, &AtomicBool::new(false))
            .unwrap_err()
            .to_string();
        assert!(err.contains("did not finish"), "{err}");
        assert!(m.ran()[1].0.contains("$m.Cancel('9NBLGGH4R32N')"));
    }

    #[test]
    fn a_cancel_before_winget_starts_runs_nothing() {
        let m = killed(vec![], vec![]);
        let err = install(&appx_app(), &m, &AtomicBool::new(true)).unwrap_err();
        assert!(matches!(err, Error::AppCancelled(_)), "{err:?}");
        assert!(m.ran().is_empty());
    }

    #[test]
    fn the_store_cancel_script_embeds_only_a_validated_id() {
        assert!(store_cancel_script("9NBLGGH4R32N")
            .unwrap()
            .contains("Cancel('9NBLGGH4R32N')"));
        assert!(store_cancel_script("x'); Remove-Item -Recurse C:/; ('").is_err());
    }
}
