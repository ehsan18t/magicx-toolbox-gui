//! `EffectKind` for `Setting::Power`: the AC and DC indexes of one power plan setting, as one
//! value. An authored address reads and drives the active plan; a drive back to a captured value
//! goes to the plan it was read from (`Setting::pinned_to`). Runs in-process at `user` or `admin`;
//! no broker op carries it.

use crate::error::Error as BackendError;
use crate::services::power_service;
use crate::tweaks::model::{PlanTag, PowerAddr, Setting, Value};

use super::{guard_level, map_backend_error, EffectKind, Error, ExecCx};

pub trait PowerApi: Send + Sync {
    fn active(&self) -> Result<String, BackendError>;
    fn exists(&self, scheme: &str) -> Result<bool, BackendError>;
    fn read(&self, scheme: &str, addr: &PowerAddr) -> Result<(u32, u32), BackendError>;
    fn write(&self, scheme: &str, addr: &PowerAddr, ac: u32, dc: u32) -> Result<(), BackendError>;
    fn activate(&self, scheme: &str) -> Result<(), BackendError>;
}

pub struct Native;

impl PowerApi for Native {
    fn active(&self) -> Result<String, BackendError> {
        power_service::active_scheme()
    }

    fn exists(&self, scheme: &str) -> Result<bool, BackendError> {
        power_service::scheme_exists(scheme)
    }

    fn read(&self, scheme: &str, addr: &PowerAddr) -> Result<(u32, u32), BackendError> {
        power_service::read_index(scheme, &addr.subgroup, &addr.setting)
    }

    fn write(&self, scheme: &str, addr: &PowerAddr, ac: u32, dc: u32) -> Result<(), BackendError> {
        power_service::write_index(scheme, &addr.subgroup, &addr.setting, ac, dc)
    }

    fn activate(&self, scheme: &str) -> Result<(), BackendError> {
        power_service::activate(scheme)
    }
}

pub struct PowerKind<'a>(pub &'a dyn PowerApi);

impl PowerKind<'_> {
    /// The plan `addr` names, or `None` when a pinned plan has been deleted since.
    fn scheme(&self, addr: &PowerAddr) -> Result<Option<String>, Error> {
        match &addr.scheme {
            None => self.0.active().map(Some).map_err(map_backend_error),
            Some(pinned) => match self.0.exists(pinned).map_err(map_backend_error)? {
                true => Ok(Some(pinned.clone())),
                false => Ok(None),
            },
        }
    }
}

impl EffectKind for PowerKind<'_> {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        let Setting::Power(addr) = s else {
            return Err(Error::Invalid("PowerKind cannot read this Setting"));
        };
        let Some(scheme) = self.scheme(addr)? else {
            return Ok(Value::Missing);
        };
        let (ac, dc) = self.0.read(&scheme, addr).map_err(map_backend_error)?;
        Ok(Value::PowerIndex {
            ac,
            dc,
            plan: PlanTag(Some(scheme.into())),
        })
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        let Setting::Power(addr) = s else {
            return Err(Error::Invalid("PowerKind cannot drive this Setting"));
        };
        let Value::PowerIndex { ac, dc, .. } = target else {
            return Err(Error::Invalid(
                "a power setting can only be driven to an AC and DC index",
            ));
        };
        let Some(scheme) = self.scheme(addr)? else {
            return Err(Error::ResourceMissing(format!(
                "power plan {} no longer exists",
                addr.scheme.as_deref().unwrap_or_default()
            )));
        };
        self.0
            .write(&scheme, addr, *ac, *dc)
            .map_err(map_backend_error)?;
        // Activating a pinned plan that is not the active one would switch the user's plan.
        if scheme == self.0.active().map_err(map_backend_error)? {
            self.0.activate(&scheme).map_err(map_backend_error)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    /// Plans by GUID, each holding the one setting under test, plus which plan is active.
    pub(crate) struct Plans {
        pub(crate) plans: Mutex<BTreeMap<String, (u32, u32)>>,
        pub(crate) active: Mutex<String>,
        pub(crate) activations: Mutex<Vec<String>>,
        pub(crate) fail: Option<u32>,
    }

    impl Plans {
        pub(crate) fn new(plans: &[(&str, (u32, u32))], active: &str) -> Self {
            Self {
                plans: Mutex::new(plans.iter().map(|(g, v)| (g.to_string(), *v)).collect()),
                active: Mutex::new(active.into()),
                activations: Mutex::new(Vec::new()),
                fail: None,
            }
        }

        pub(crate) fn holding(&self, scheme: &str) -> Option<(u32, u32)> {
            self.plans.lock().unwrap().get(scheme).copied()
        }

        fn fail(&self) -> Result<(), BackendError> {
            self.fail
                .map_or(Ok(()), |code| Err(BackendError::win32("fake", code)))
        }
    }

    impl PowerApi for Plans {
        fn active(&self) -> Result<String, BackendError> {
            self.fail()?;
            Ok(self.active.lock().unwrap().clone())
        }

        fn exists(&self, scheme: &str) -> Result<bool, BackendError> {
            Ok(self.holding(scheme).is_some())
        }

        fn read(&self, scheme: &str, _: &PowerAddr) -> Result<(u32, u32), BackendError> {
            self.fail()?;
            Ok(self
                .holding(scheme)
                .expect("the kind reads only existing plans"))
        }

        fn write(&self, scheme: &str, _: &PowerAddr, ac: u32, dc: u32) -> Result<(), BackendError> {
            self.fail()?;
            self.plans.lock().unwrap().insert(scheme.into(), (ac, dc));
            Ok(())
        }

        fn activate(&self, scheme: &str) -> Result<(), BackendError> {
            self.activations.lock().unwrap().push(scheme.into());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::Plans;
    use super::*;
    use crate::error::win32;
    use crate::tweaks::model::Level;

    const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
    const HIGH: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";

    /// Allow wake timers (SUB_SLEEP, RTCWAKE).
    fn wake_timers() -> Setting {
        Setting::Power(PowerAddr {
            subgroup: "238c9fa8-0aad-41ed-83f4-97be242c8f20".into(),
            setting: "bd3b718a-0680-4d9d-8ab2-e1d2b4ac806d".into(),
            label: "Allow wake timers".into(),
            scheme: None,
        })
    }

    fn index(ac: u32, dc: u32) -> Value {
        Value::PowerIndex {
            ac,
            dc,
            plan: PlanTag(None),
        }
    }

    fn cx(level: Level) -> ExecCx {
        ExecCx::new(level)
    }

    #[test]
    fn capture_drive_verify_and_restore_round_trip_on_the_active_plan() {
        let api = Plans::new(&[(BALANCED, (2, 0))], BALANCED);
        let kind = PowerKind(&api);
        let s = wake_timers();
        let captured = kind.read(&s, &cx(Level::User)).unwrap();
        assert_eq!(captured, index(2, 0));
        assert!(
            matches!(&captured, Value::PowerIndex { plan: PlanTag(Some(p)), .. } if &**p == BALANCED)
        );

        kind.drive(&s, &index(0, 0), &cx(Level::Admin)).unwrap();
        assert_eq!(kind.read(&s, &cx(Level::User)).unwrap(), index(0, 0));

        let back = s.pinned_to(&captured);
        kind.drive(&back, &captured, &cx(Level::User)).unwrap();
        assert_eq!(kind.read(&back, &cx(Level::User)).unwrap(), captured);
        assert_eq!(*api.activations.lock().unwrap(), [BALANCED, BALANCED]);
    }

    #[test]
    fn a_restore_after_a_plan_switch_goes_back_to_the_captured_plan() {
        let api = Plans::new(&[(BALANCED, (2, 0)), (HIGH, (1, 1))], BALANCED);
        let kind = PowerKind(&api);
        let s = wake_timers();
        let captured = kind.read(&s, &cx(Level::User)).unwrap();
        kind.drive(&s, &index(0, 0), &cx(Level::Admin)).unwrap();
        *api.active.lock().unwrap() = HIGH.into();
        api.activations.lock().unwrap().clear();

        let back = s.pinned_to(&captured);
        kind.drive(&back, &captured, &cx(Level::Admin)).unwrap();

        assert_eq!(api.holding(BALANCED), Some((2, 0)));
        assert_eq!(
            api.holding(HIGH),
            Some((1, 1)),
            "the active plan is untouched"
        );
        assert!(api.activations.lock().unwrap().is_empty(), "no plan switch");
        assert_eq!(kind.read(&back, &cx(Level::User)).unwrap(), captured);
        assert_eq!(kind.read(&s, &cx(Level::User)).unwrap(), index(1, 1));
    }

    #[test]
    fn a_deleted_captured_plan_reads_missing_and_refuses_a_drive() {
        let api = Plans::new(&[(BALANCED, (2, 0)), (HIGH, (0, 0))], BALANCED);
        let kind = PowerKind(&api);
        let s = wake_timers();
        let captured = kind.read(&s, &cx(Level::User)).unwrap();
        api.plans.lock().unwrap().remove(BALANCED);
        *api.active.lock().unwrap() = HIGH.into();

        let back = s.pinned_to(&captured);
        assert_eq!(kind.read(&back, &cx(Level::User)).unwrap(), Value::Missing);
        assert!(matches!(
            kind.drive(&back, &captured, &cx(Level::Admin)),
            Err(Error::ResourceMissing(_))
        ));
        assert_eq!(api.holding(HIGH), Some((0, 0)));
    }

    #[test]
    fn a_denied_call_needs_elevation_and_a_missing_setting_is_not_found() {
        let s = wake_timers();
        for (code, denied) in [(win32::ACCESS_DENIED, true), (win32::FILE_NOT_FOUND, false)] {
            let mut api = Plans::new(&[(BALANCED, (0, 0))], BALANCED);
            api.fail = Some(code);
            let read = PowerKind(&api).read(&s, &cx(Level::User));
            let drive = PowerKind(&api).drive(&s, &index(0, 0), &cx(Level::User));
            if denied {
                assert!(matches!(read, Err(Error::AccessDenied(_))));
                assert!(matches!(drive, Err(Error::AccessDenied(_))));
            } else {
                assert!(matches!(read, Err(Error::NotFound(_))));
            }
        }
    }

    #[test]
    fn drive_refuses_ti_and_foreign_values_before_writing() {
        let api = Plans::new(&[(BALANCED, (1, 1))], BALANCED);
        let kind = PowerKind(&api);
        let s = wake_timers();
        assert!(matches!(
            kind.drive(&s, &index(0, 0), &cx(Level::Ti)),
            Err(Error::UnsupportedLevel(Level::Ti))
        ));
        assert!(matches!(
            kind.drive(&s, &Value::Present(true), &cx(Level::User)),
            Err(Error::Invalid(_))
        ));
        assert_eq!(api.holding(BALANCED), Some((1, 1)));
    }

    #[test]
    #[ignore = "reads the live active power scheme; run with `cargo test -- --ignored`"]
    fn reads_the_active_schemes_wake_timer_setting() {
        let value = PowerKind(&Native)
            .read(&wake_timers(), &cx(Level::User))
            .expect("the active scheme's wake timer setting is readable as any user");
        println!("active scheme, Allow wake timers: {value:?}");
        assert!(matches!(
            value,
            Value::PowerIndex {
                plan: PlanTag(Some(_)),
                ..
            }
        ));
    }
}
