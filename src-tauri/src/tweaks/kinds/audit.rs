//! `EffectKind` for `Setting::Audit`: one outcome flag (success or failure) of an advanced
//! audit policy subcategory. Needs `admin` for reads and drives; no broker op carries it.

use std::sync::{Mutex, PoisonError};

use crate::error::Error as BackendError;
use crate::services::audit_service;
use crate::tweaks::model::{AuditEvent, Setting, Value};

use super::{guard_level, map_backend_error, EffectKind, Error, ExecCx};

const SUCCESS: u32 = 1;
const FAILURE: u32 = 2;
const NONE: u32 = 4;

fn flag(event: AuditEvent) -> u32 {
    match event {
        AuditEvent::Success => SUCCESS,
        AuditEvent::Failure => FAILURE,
    }
}

/// The writes that set `event` to `on` from the `current` flags. A written 0 means "unchanged", so
/// no auditing is written as NONE; clearing a flag clears both first, which holds whether a write
/// replaces the flags or adds to them.
fn writes(current: u32, event: AuditEvent, on: bool) -> Vec<u32> {
    let current = current & (SUCCESS | FAILURE);
    let target = if on {
        current | flag(event)
    } else {
        current & !flag(event)
    };
    if target == current {
        Vec::new()
    } else if target == 0 {
        vec![NONE]
    } else if current & !target != 0 {
        vec![NONE, target]
    } else {
        vec![target]
    }
}

pub trait AuditApi: Send + Sync {
    fn query(&self, subcategory: &str) -> Result<u32, BackendError>;
    fn set(&self, subcategory: &str, flags: u32) -> Result<(), BackendError>;
}

pub struct Native;

impl AuditApi for Native {
    fn query(&self, subcategory: &str) -> Result<u32, BackendError> {
        audit_service::query(subcategory)
    }

    fn set(&self, subcategory: &str, flags: u32) -> Result<(), BackendError> {
        audit_service::set(subcategory, flags)
    }
}

/// A flag's drive rewrites its whole subcategory in up to two writes, so drives must not
/// interleave, and a read must not see the cleared state between them.
static READ_MODIFY_WRITE: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    READ_MODIFY_WRITE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

pub struct AuditKind<'a>(pub &'a dyn AuditApi);

impl EffectKind for AuditKind<'_> {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        let Setting::Audit(addr) = s else {
            return Err(Error::Invalid("AuditKind cannot read this Setting"));
        };
        let _rmw = lock();
        let flags = self.0.query(&addr.subcategory).map_err(map_backend_error)?;
        Ok(Value::Audited(flags & flag(addr.event) != 0))
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        let Setting::Audit(addr) = s else {
            return Err(Error::Invalid("AuditKind cannot drive this Setting"));
        };
        let Value::Audited(on) = target else {
            return Err(Error::Invalid(
                "an audit flag can only be driven to audited or not",
            ));
        };
        let _rmw = lock();
        let current = self.0.query(&addr.subcategory).map_err(map_backend_error)?;
        for (step, flags) in writes(current, addr.event, *on).into_iter().enumerate() {
            let Err(e) = self.0.set(&addr.subcategory, flags) else {
                continue;
            };
            // The first write cleared the flag this effect does not own; rollback restores only
            // owned flags, so put the subcategory back here.
            if step > 0 {
                if let Err(undo) = self.0.set(&addr.subcategory, current & (SUCCESS | FAILURE)) {
                    log::warn!(
                        "could not put audit subcategory {} back after a failed write: {undo}",
                        addr.subcategory
                    );
                }
            }
            return Err(map_backend_error(e));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::win32;
    use crate::tweaks::model::{AuditAddr, Level};

    /// One subcategory. `additive` models a write that adds flags instead of replacing them; both
    /// models clear everything on NONE and ignore a written 0.
    struct Fake {
        live: Mutex<u32>,
        written: Mutex<Vec<u32>>,
        additive: bool,
        fail: Option<u32>,
        /// Fails the write with this (1-based) number.
        fail_write: Option<usize>,
    }

    impl Fake {
        fn holding(flags: u32, additive: bool) -> Self {
            Self {
                live: Mutex::new(flags),
                written: Mutex::new(Vec::new()),
                additive,
                fail: None,
                fail_write: None,
            }
        }
    }

    impl AuditApi for Fake {
        fn query(&self, _: &str) -> Result<u32, BackendError> {
            match self.fail {
                Some(code) => Err(BackendError::win32("fake", code)),
                None => Ok(*self.live.lock().unwrap()),
            }
        }

        fn set(&self, _: &str, flags: u32) -> Result<(), BackendError> {
            let mut written = self.written.lock().unwrap();
            written.push(flags);
            if self.fail_write == Some(written.len()) {
                return Err(BackendError::win32("fake", win32::ACCESS_DENIED));
            }
            drop(written);
            let mut live = self.live.lock().unwrap();
            *live = match flags {
                0 => *live,
                NONE => 0,
                f if self.additive => *live | f,
                f => f,
            };
            Ok(())
        }
    }

    const LOGON: &str = "0cce9215-69ae-11d9-bed3-505054503030";

    fn audit(event: AuditEvent) -> Setting {
        Setting::Audit(AuditAddr {
            subcategory: LOGON.into(),
            event,
            label: "Logon".into(),
        })
    }

    fn admin() -> ExecCx {
        ExecCx::new(Level::Admin)
    }

    #[test]
    fn a_flag_reads_only_its_own_bit() {
        let api = Fake::holding(SUCCESS, false);
        let kind = AuditKind(&api);
        assert_eq!(
            kind.read(&audit(AuditEvent::Success), &admin()).unwrap(),
            Value::Audited(true)
        );
        assert_eq!(
            kind.read(&audit(AuditEvent::Failure), &admin()).unwrap(),
            Value::Audited(false)
        );
    }

    #[test]
    fn writes_never_send_zero_and_clear_before_narrowing() {
        use AuditEvent::{Failure, Success};
        assert_eq!(writes(0, Success, true), vec![SUCCESS]);
        assert_eq!(writes(SUCCESS, Failure, true), vec![SUCCESS | FAILURE]);
        assert_eq!(writes(SUCCESS, Success, false), vec![NONE]);
        assert_eq!(
            writes(SUCCESS | FAILURE, Failure, false),
            vec![NONE, SUCCESS]
        );
        assert_eq!(writes(SUCCESS, Success, true), Vec::<u32>::new());
        assert_eq!(writes(NONE, Success, false), Vec::<u32>::new());
    }

    /// Capture both flags of a subcategory with no auditing, enable both, then restore the
    /// capture: the "no auditing" state must come back, which a written 0 would not do.
    #[test]
    fn no_auditing_round_trips_through_capture_and_restore() {
        for additive in [false, true] {
            let api = Fake::holding(0, additive);
            let kind = AuditKind(&api);
            let (success, failure) = (audit(AuditEvent::Success), audit(AuditEvent::Failure));
            let captured = [
                kind.read(&success, &admin()).unwrap(),
                kind.read(&failure, &admin()).unwrap(),
            ];

            kind.drive(&success, &Value::Audited(true), &admin())
                .unwrap();
            kind.drive(&failure, &Value::Audited(true), &admin())
                .unwrap();
            assert_eq!(*api.live.lock().unwrap(), SUCCESS | FAILURE);

            kind.drive(&success, &captured[0], &admin()).unwrap();
            kind.drive(&failure, &captured[1], &admin()).unwrap();
            assert_eq!(*api.live.lock().unwrap(), 0, "additive: {additive}");
            assert!(!api.written.lock().unwrap().contains(&0));
            assert_eq!(kind.read(&success, &admin()).unwrap(), captured[0]);
            assert_eq!(kind.read(&failure, &admin()).unwrap(), captured[1]);
        }
    }

    #[test]
    fn driving_one_flag_keeps_the_other() {
        for additive in [false, true] {
            let api = Fake::holding(SUCCESS | FAILURE, additive);
            let kind = AuditKind(&api);
            kind.drive(
                &audit(AuditEvent::Failure),
                &Value::Audited(false),
                &admin(),
            )
            .unwrap();
            assert_eq!(*api.live.lock().unwrap(), SUCCESS, "additive: {additive}");
        }
    }

    #[test]
    fn a_failed_second_write_puts_the_flag_it_does_not_own_back() {
        for additive in [false, true] {
            let mut api = Fake::holding(SUCCESS | FAILURE, additive);
            api.fail_write = Some(2);
            let err = AuditKind(&api)
                .drive(
                    &audit(AuditEvent::Failure),
                    &Value::Audited(false),
                    &admin(),
                )
                .unwrap_err();
            assert!(matches!(err, Error::AccessDenied(_)), "{err:?}");
            assert_eq!(
                *api.written.lock().unwrap(),
                [NONE, SUCCESS, SUCCESS | FAILURE]
            );
            assert_eq!(
                *api.live.lock().unwrap(),
                SUCCESS | FAILURE,
                "additive: {additive}"
            );
        }
    }

    #[test]
    fn an_unelevated_read_needs_elevation_and_ti_is_refused() {
        let mut api = Fake::holding(0, false);
        api.fail = Some(win32::PRIVILEGE_NOT_HELD);
        assert!(matches!(
            AuditKind(&api).read(&audit(AuditEvent::Success), &ExecCx::new(Level::User)),
            Err(Error::AccessDenied(_))
        ));
        assert!(matches!(
            AuditKind(&api).drive(
                &audit(AuditEvent::Success),
                &Value::Audited(true),
                &ExecCx::new(Level::Ti)
            ),
            Err(Error::UnsupportedLevel(Level::Ti))
        ));
        assert!(api.written.lock().unwrap().is_empty());
    }

    #[test]
    #[ignore = "reads the live audit policy; needs admin -- run with `cargo test -- --ignored` while elevated"]
    fn reads_the_logon_subcategory() {
        let kind = AuditKind(&Native);
        let success = kind.read(&audit(AuditEvent::Success), &admin());
        let failure = kind.read(&audit(AuditEvent::Failure), &admin());
        println!("Logon subcategory: success {success:?}, failure {failure:?}");
        assert!(matches!(success, Ok(Value::Audited(_))));
        assert!(matches!(failure, Ok(Value::Audited(_))));
    }
}
