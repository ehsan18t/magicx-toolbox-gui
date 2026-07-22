//! `EffectKind` for `Setting::Service` (spec §5.1/§5.4). Wraps the low-level `service_control` SCM
//! primitive for the primary start type, plus a direct registry read/write of the
//! `DelayedAutostart` DWORD for the `AutomaticDelayed` distinction `service_control`'s typed
//! `ServiceStartupType` does not carry — exactly the technique `service_control::read_startup_type`
//! already uses for the primary type itself ("locale-free ... via the typed `Start` registry
//! value"); the SCM's own `ChangeServiceConfig2W`/`QueryServiceConfig2W` persist to this exact
//! registry value, so a direct typed read/write is equivalent, without a second Win32 FFI surface
//! for one DWORD.
//!
//! `Missing` (spec §5.4): a service the SCM does not know about reads `Ok(Value::Missing)`, never
//! an error; driving *to* `Missing` is a verified no-op (the engine never installs/uninstalls a
//! service); driving a real `Startup` value at a service that turns out missing is the typed
//! [`Error::ResourceMissing`] — never a silent skip (invariant 12).

use crate::error::Error as BackendError;
use crate::models::{RegistryHive, ServiceStartupType};
use crate::services::registry_service;
use crate::services::service_control::{self, ServiceStatus};
use crate::tweaks::model::{Setting, StartupType, SvcAddr, Value};

use super::{guard_level, map_backend_error, EffectKind, Error, ExecCx};

/// `DelayedAutostart` lives beside the primary `Start` value under the same service registry key.
const DELAYED_VALUE: &str = "DelayedAutostart";

/// `EffectKind` for `Setting::Service`.
pub struct ServiceKind;

impl EffectKind for ServiceKind {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        // Reads never escalate (spec invariant 24) -- `cx` is unused here on purpose.
        match s {
            Setting::Service(addr) => read_service(addr),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Task(_)
            | Setting::Hosts(_)
            | Setting::Firewall(_) => Err(Error::Invalid("ServiceKind cannot read this Setting")),
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        match s {
            Setting::Service(addr) => drive_service(addr, target),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Task(_)
            | Setting::Hosts(_)
            | Setting::Firewall(_) => Err(Error::Invalid("ServiceKind cannot drive this Setting")),
        }
    }
}

fn service_key(name: &str) -> String {
    format!("System\\CurrentControlSet\\Services\\{name}")
}

/// `true` only when the value is exactly `1` — absent or `0` both mean "not delayed" (Windows'
/// own default), so a merely-missing value never fabricates delayed-start.
fn delayed_autostart(name: &str) -> bool {
    matches!(
        registry_service::read_dword(&RegistryHive::Hklm, &service_key(name), DELAYED_VALUE),
        Ok(Some(1))
    )
}

fn set_delayed_autostart(name: &str, delayed: bool) -> Result<(), Error> {
    registry_service::set_dword(
        &RegistryHive::Hklm,
        &service_key(name),
        DELAYED_VALUE,
        u32::from(delayed),
    )
    .map_err(map_backend_error)
}

fn old_startup(t: StartupType) -> ServiceStartupType {
    match t {
        StartupType::Boot => ServiceStartupType::Boot,
        StartupType::System => ServiceStartupType::System,
        StartupType::Automatic | StartupType::AutomaticDelayed => ServiceStartupType::Automatic,
        StartupType::Manual => ServiceStartupType::Manual,
        StartupType::Disabled => ServiceStartupType::Disabled,
    }
}

fn new_startup(t: ServiceStartupType) -> StartupType {
    match t {
        ServiceStartupType::Boot => StartupType::Boot,
        ServiceStartupType::System => StartupType::System,
        // AutomaticDelayed is resolved by the caller (read_service); the primitive's own type
        // has no delayed variant, so this half always yields plain Automatic.
        ServiceStartupType::Automatic => StartupType::Automatic,
        ServiceStartupType::Manual => StartupType::Manual,
        ServiceStartupType::Disabled => StartupType::Disabled,
    }
}

/// The exists/backend-error/startup-type decision, isolated from the SCM call itself so the
/// Missing/error distinction (invariant 2/12) is unit-testable without a real service. Never
/// resolves `AutomaticDelayed` — that needs the second, real registry read `read_service` layers
/// on afterward.
fn map_status(name: &str, result: Result<ServiceStatus, BackendError>) -> Result<Value, Error> {
    let status = result.map_err(map_backend_error)?;
    if !status.exists {
        return Ok(Value::Missing);
    }
    status
        .startup_type
        .map(new_startup)
        .map(Value::Startup)
        .ok_or_else(|| {
            Error::Backend(format!(
                "service '{name}' exists but its startup type could not be read"
            ))
        })
}

fn read_service(addr: &SvcAddr) -> Result<Value, Error> {
    let value = map_status(&addr.name, service_control::get_service_status(&addr.name))?;
    Ok(match value {
        Value::Startup(StartupType::Automatic) if delayed_autostart(&addr.name) => {
            Value::Startup(StartupType::AutomaticDelayed)
        }
        other => other,
    })
}

fn drive_service(addr: &SvcAddr, target: &Value) -> Result<(), Error> {
    match target {
        // The engine never installs/uninstalls a service (spec §5.4, invariant 12): a defined
        // no-op regardless of whether the service currently exists.
        Value::Missing => Ok(()),
        Value::Startup(st) => {
            if map_status(&addr.name, service_control::get_service_status(&addr.name))?
                == Value::Missing
            {
                return Err(Error::ResourceMissing(format!(
                    "service '{}' does not exist",
                    addr.name
                )));
            }
            service_control::set_service_startup(&addr.name, &old_startup(*st))
                .map_err(map_backend_error)?;
            // Always set the flag explicitly, for every target type -- not just Automatic/
            // AutomaticDelayed. Leaving it untouched for e.g. a drive to Disabled would strand a
            // stale `1` from an earlier AutomaticDelayed drive: invisible to our own read path
            // (which only inspects the flag when the *current* type is Automatic), but a real,
            // lingering change to the machine that a later drive to Automatic would silently
            // inherit. Driving must always leave the registry in exactly the driven-to state.
            set_delayed_autostart(&addr.name, *st == StartupType::AutomaticDelayed)
        }
        _ => Err(Error::Invalid(
            "a service can only be driven to Startup or Missing",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_control::ServiceState;
    use crate::tweaks::model::Level;

    fn user_cx() -> ExecCx {
        ExecCx::new(Level::User)
    }

    /// A name that certainly does not exist -- no elevation, no real resource needed (controller
    /// decision 4).
    const NO_SUCH_SERVICE: &str = "MagicXNoSuchService_5F3F1D2E-6A4B-4C9E-9B0A-6B6E6C7D8E9F";

    #[test]
    fn missing_service_reads_missing() {
        let cx = user_cx();
        let setting = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        assert_eq!(ServiceKind.read(&setting, &cx).unwrap(), Value::Missing);
    }

    #[test]
    fn error_can_never_collapse_into_missing() {
        // A nonexistent service collapses to Ok(Missing) -- never an error.
        let missing = map_status(
            "irrelevant",
            Ok(ServiceStatus {
                name: "irrelevant".to_string(),
                state: ServiceState::Unknown,
                startup_type: None,
                exists: false,
            }),
        );
        assert!(matches!(missing, Ok(Value::Missing)), "got {missing:?}");

        // Any backend Err is `?`-propagated before the Missing-producing branch is reachable, so
        // it can never be silently downgraded to "the resource doesn't exist" (invariant 2). This
        // does NOT prove a real OS access-denied surfaces as `Error::AccessDenied`: neither
        // `service_control` nor `scheduler_service` has a distinguished access-denied variant the
        // way `registry_service::classify_open_error` does -- their real failures arrive as
        // `ServiceControl(String)`/`CommandExecution(String)` and land in `map_backend_error`'s
        // `Backend` catch-all, indistinguishable from e.g. RPC-unavailable or a locked service
        // database. `RequiresAdmin` below is the one typed "insufficient rights" signal that
        // exists at this layer today; closing that real gap is deferred to the detection task that
        // builds the needs-elevation hint, not fixed here.
        let denied = map_status("irrelevant", Err(BackendError::RequiresAdmin));
        assert!(
            matches!(denied, Err(Error::AccessDenied(_))),
            "got {denied:?}"
        );
    }

    #[test]
    fn drive_to_missing_is_noop_ok() {
        let cx = user_cx();
        let setting = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        ServiceKind
            .drive(&setting, &Value::Missing, &cx)
            .expect("driving a service to Missing must be a no-op success");
    }

    #[test]
    fn drive_real_value_at_missing_resource_is_typed_error() {
        let cx = user_cx();
        let setting = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        let err = ServiceKind
            .drive(&setting, &Value::Startup(StartupType::Manual), &cx)
            .expect_err("driving a real value at a missing service must be a typed error");
        assert!(matches!(err, Error::ResourceMissing(_)), "got {err:?}");
    }

    #[test]
    fn drive_rejects_system_and_ti_levels_for_now() {
        let setting = Setting::Service(SvcAddr {
            name: NO_SUCH_SERVICE.to_string(),
        });
        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = ServiceKind
                .drive(&setting, &Value::Startup(StartupType::Manual), &cx)
                .expect_err("this build cannot yet route System/Ti through the broker");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    #[test]
    fn known_service_reads_a_real_startup_value() {
        // "Schedule" (Task Scheduler) exists on every Windows edition -- the same reference
        // service service_control.rs's own tests use. Read-only: no mutation, no elevation.
        let cx = user_cx();
        let setting = Setting::Service(SvcAddr {
            name: "Schedule".to_string(),
        });
        let value = ServiceKind
            .read(&setting, &cx)
            .expect("Schedule must be readable");
        assert!(matches!(value, Value::Startup(_)), "got {value:?}");
    }

    #[test]
    fn startup_type_conversions_cover_every_variant() {
        // Boot/System are only legal on driver services (ChangeServiceConfigW rejects them for a
        // normal Win32 service), so they can never be safely exercised against a real test
        // service -- pinned here as a pure conversion instead, with no live service touched.
        for st in [
            StartupType::Boot,
            StartupType::System,
            StartupType::Automatic,
            StartupType::AutomaticDelayed,
            StartupType::Manual,
            StartupType::Disabled,
        ] {
            let recovered = new_startup(old_startup(st));
            // AutomaticDelayed collapses to plain Automatic through the primitive's 5-value
            // domain; read_service alone resolves it back via the delayed-autostart registry bit.
            let expected = if st == StartupType::AutomaticDelayed {
                StartupType::Automatic
            } else {
                st
            };
            assert_eq!(recovered, expected, "{st:?} -> {recovered:?}");
        }
    }

    #[test]
    #[ignore = "writes a real service's startup type; needs admin -- run with `cargo test -- --ignored` while elevated"]
    fn service_startup_roundtrip() {
        // RemoteRegistry: present on every Windows SKU (core NT service), governs only *remote*
        // registry access, so nothing on this machine depends on it -- safe to toggle
        // transiently. Boot/System are excluded (see startup_type_conversions_cover_every_variant
        // above): they are only valid for driver services.
        const NAME: &str = "RemoteRegistry";
        let setting = Setting::Service(SvcAddr {
            name: NAME.to_string(),
        });
        let cx = ExecCx::new(Level::Admin);

        let original = match ServiceKind
            .read(&setting, &cx)
            .expect("RemoteRegistry must exist")
        {
            Value::Startup(st) => st,
            other => panic!("expected Value::Startup, got {other:?}"),
        };
        let _restore = RestoreServiceStartup {
            name: NAME.to_string(),
            original,
        };

        for target in [
            StartupType::Manual,
            StartupType::Disabled,
            StartupType::Automatic,
            StartupType::AutomaticDelayed,
            original,
        ] {
            ServiceKind
                .drive(&setting, &Value::Startup(target), &cx)
                .unwrap_or_else(|e| panic!("drive to {target:?} failed: {e}"));
            assert_eq!(
                ServiceKind.read(&setting, &cx).unwrap(),
                Value::Startup(target),
                "roundtrip mismatch for {target:?}"
            );
        }
    }

    /// Restores a real service's startup type on drop, even on panic (the test profile unwinds),
    /// so a failed assertion never leaves a live service permanently mutated.
    struct RestoreServiceStartup {
        name: String,
        original: StartupType,
    }
    impl Drop for RestoreServiceStartup {
        fn drop(&mut self) {
            let setting = Setting::Service(SvcAddr {
                name: self.name.clone(),
            });
            let cx = ExecCx::new(Level::Admin);
            // Cleanup only -- the one accepted `let _` exception (a Drop-guard restoring state).
            // Note: drive_service's DelayedAutostart write is unconditional, so restoring to a
            // non-delayed original writes an explicit `0` rather than removing the value even if
            // it was absent before this test ran. That is functionally identical to absent (both
            // our read path and Windows treat 0/absent the same) -- not the residue regression
            // fixed above, just a harmless, expected byte-level difference.
            let _ = ServiceKind.drive(&setting, &Value::Startup(self.original), &cx);
        }
    }
}
