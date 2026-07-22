//! `EffectKind` for `Setting::Hosts` (spec §5.1). Wraps the low-level `hosts_service` primitive:
//! `Present(true)` ensures the `(ip, domain)` entry exists (adding it if absent), `Present(false)`
//! ensures it does not (removing it if present) — both idempotent no-ops otherwise, exactly
//! `hosts_service::add_hosts_entry`/`remove_hosts_entry`'s own existing behavior. Unlike
//! Service/Task there is no `Missing` state here: a hosts entry is never "not installed", so
//! driving `Present(true)` is itself the creation the engine performs (mirrors
//! `RegistryKind::drive_key`, not `ServiceKind`/`TaskKind`).

use crate::services::hosts_service;
use crate::tweaks::model::{HostsAddr, Setting, Value};

use super::{guard_level, map_backend_error, EffectKind, Error, ExecCx};

/// `EffectKind` for `Setting::Hosts`.
pub struct HostsKind;

impl EffectKind for HostsKind {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        // Reads never escalate (spec invariant 24) -- `cx` is unused here on purpose.
        match s {
            Setting::Hosts(addr) => read_hosts(addr),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Service(_)
            | Setting::Task(_)
            | Setting::Firewall(_) => Err(Error::Invalid("HostsKind cannot read this Setting")),
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        match s {
            Setting::Hosts(addr) => drive_hosts(addr, target),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Service(_)
            | Setting::Task(_)
            | Setting::Firewall(_) => Err(Error::Invalid("HostsKind cannot drive this Setting")),
        }
    }
}

fn read_hosts(addr: &HostsAddr) -> Result<Value, Error> {
    let exists = hosts_service::entry_exists(&addr.ip, &addr.domain).map_err(map_backend_error)?;
    Ok(Value::Present(exists))
}

fn drive_hosts(addr: &HostsAddr, target: &Value) -> Result<(), Error> {
    match target {
        Value::Present(true) => {
            hosts_service::add_hosts_entry(&addr.ip, &addr.domain, None).map_err(map_backend_error)
        }
        Value::Present(false) => {
            hosts_service::remove_hosts_entry(&addr.ip, &addr.domain).map_err(map_backend_error)
        }
        _ => Err(Error::Invalid(
            "a hosts entry can only be driven to Present(bool)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::Level;

    fn user_cx() -> ExecCx {
        ExecCx::new(Level::User)
    }

    /// `.invalid` is IANA-reserved (RFC 2606) and guaranteed never to resolve or appear in a real
    /// hosts file (Task 7 controller decision 4) -- safe even though `read` touches the real file.
    const NO_SUCH_DOMAIN: &str = "magicx-toolbox-test-does-not-exist-5f3f1d2e.invalid";

    fn addr(domain: &str) -> HostsAddr {
        HostsAddr {
            ip: "0.0.0.0".to_string(),
            domain: domain.to_string(),
        }
    }

    #[test]
    fn missing_hosts_entry_reads_present_false() {
        // Reads the real hosts file (harmless, no admin needed) but the domain certainly is not in
        // it, so this runs by default and carries the primary coverage for the not-found path
        // (controller decision 4) without any write.
        let cx = user_cx();
        let setting = Setting::Hosts(addr(NO_SUCH_DOMAIN));
        assert_eq!(
            HostsKind.read(&setting, &cx).unwrap(),
            Value::Present(false)
        );
    }

    #[test]
    fn drive_rejects_system_and_ti_levels_for_now() {
        // guard_level fires before any hosts_service call -- safe to run by default.
        let setting = Setting::Hosts(addr(NO_SUCH_DOMAIN));
        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = HostsKind
                .drive(&setting, &Value::Present(true), &cx)
                .expect_err("this build cannot yet route System/Ti through the broker");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    #[test]
    #[ignore = "writes the real hosts file; needs admin -- run with `cargo test -- --ignored` while elevated"]
    fn hosts_present_roundtrip() {
        const DOMAIN: &str = "magicx-toolbox-test-7f2c9a41-roundtrip.invalid";
        let setting = Setting::Hosts(addr(DOMAIN));
        let cx = ExecCx::new(Level::Admin);
        let _cleanup = RemoveHostsEntry {
            ip: "0.0.0.0".to_string(),
            domain: DOMAIN.to_string(),
        };

        HostsKind
            .drive(&setting, &Value::Present(true), &cx)
            .unwrap();
        assert_eq!(HostsKind.read(&setting, &cx).unwrap(), Value::Present(true));

        HostsKind
            .drive(&setting, &Value::Present(false), &cx)
            .unwrap();
        assert_eq!(
            HostsKind.read(&setting, &cx).unwrap(),
            Value::Present(false)
        );
    }

    /// Removes a hosts entry on drop, even on panic, so a failed assertion never leaves the real
    /// hosts file mutated (controller decision 4).
    struct RemoveHostsEntry {
        ip: String,
        domain: String,
    }
    impl Drop for RemoveHostsEntry {
        fn drop(&mut self) {
            // Cleanup only -- the one accepted `let _` exception (a Drop-guard restoring state).
            let _ = hosts_service::remove_hosts_entry(&self.ip, &self.domain);
        }
    }
}
