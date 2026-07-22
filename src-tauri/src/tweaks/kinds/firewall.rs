//! `EffectKind` for `Setting::Firewall` (spec §5.1). Wraps the low-level `firewall_service`
//! primitive: `Present(true)` recreates the rule from the *authored* `RuleAddr` definition if
//! absent, `Present(false)` deletes it if present — both idempotent no-ops otherwise, exactly
//! `firewall_service::create_firewall_rule`/`delete_firewall_rule`'s own existing behavior. Like
//! Hosts (and unlike Service/Task), there is no `Missing` state: a rule is never "not installed",
//! so driving `Present(true)` is itself the creation the engine performs.
//!
//! **Restore-fidelity limit (Task 7 controller decision 3).** Recreating a rule reproduces exactly
//! the fields `RuleAddr` carries — it is NOT guaranteed byte-identical to a pre-existing rule that
//! was deleted, because `firewall_service`/`netsh` expose rule properties this address does not
//! model (rule groups, interface types, edge traversal, security/authentication requirements,
//! per-profile domain/private/public scoping, and more). The spec's "reversible by construction"
//! (§5.1) holds relative to what the author declared, never relative to whatever Windows Firewall
//! state existed before a prior deletion this system never captured.

use crate::models::tweak::{
    FirewallChange, FirewallDirection, FirewallOperation, FirewallProtocol, FirewallRuleAction,
};
use crate::services::firewall_service;
use crate::tweaks::model::{FwAction, FwDirection, FwProtocol, RuleAddr, Setting, Value};

use super::{guard_level, map_backend_error, EffectKind, Error, ExecCx};

/// `EffectKind` for `Setting::Firewall`.
pub struct FirewallKind;

impl EffectKind for FirewallKind {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        // Reads never escalate (spec invariant 24) -- `cx` is unused here on purpose.
        match s {
            Setting::Firewall(addr) => read_firewall(addr),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Service(_)
            | Setting::Task(_)
            | Setting::Hosts(_) => Err(Error::Invalid("FirewallKind cannot read this Setting")),
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        match s {
            Setting::Firewall(addr) => drive_firewall(addr, target),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Service(_)
            | Setting::Task(_)
            | Setting::Hosts(_) => Err(Error::Invalid("FirewallKind cannot drive this Setting")),
        }
    }
}

fn read_firewall(addr: &RuleAddr) -> Result<Value, Error> {
    let exists = firewall_service::rule_exists(&addr.name).map_err(map_backend_error)?;
    Ok(Value::Present(exists))
}

fn drive_firewall(addr: &RuleAddr, target: &Value) -> Result<(), Error> {
    match target {
        // See the module doc comment: this recreates from the authored definition only -- never
        // from whatever the rule looked like before some earlier deletion.
        Value::Present(true) => firewall_service::create_firewall_rule(&to_firewall_change(addr))
            .map_err(map_backend_error),
        Value::Present(false) => {
            firewall_service::delete_firewall_rule(&addr.name).map_err(map_backend_error)
        }
        _ => Err(Error::Invalid(
            "a firewall rule can only be driven to Present(bool)",
        )),
    }
}

fn old_direction(d: FwDirection) -> FirewallDirection {
    match d {
        FwDirection::Inbound => FirewallDirection::Inbound,
        FwDirection::Outbound => FirewallDirection::Outbound,
    }
}

fn old_action(a: FwAction) -> FirewallRuleAction {
    match a {
        FwAction::Block => FirewallRuleAction::Block,
        FwAction::Allow => FirewallRuleAction::Allow,
    }
}

fn old_protocol(p: FwProtocol) -> FirewallProtocol {
    match p {
        FwProtocol::Any => FirewallProtocol::Any,
        FwProtocol::Tcp => FirewallProtocol::Tcp,
        FwProtocol::Udp => FirewallProtocol::Udp,
        FwProtocol::Icmpv4 => FirewallProtocol::Icmpv4,
        FwProtocol::Icmpv6 => FirewallProtocol::Icmpv6,
    }
}

/// `RuleAddr` -> the legacy `FirewallChange` shape `firewall_service` still speaks (same
/// translation technique `RegistryKind`/`ServiceKind` use for their own legacy primitives).
/// Always `FirewallOperation::Create`: `Value::Present(bool)` alone carries the create/delete
/// decision (see `drive_firewall`), so a `RuleAddr` only ever needs converting on the create path.
fn to_firewall_change(addr: &RuleAddr) -> FirewallChange {
    FirewallChange {
        name: addr.name.clone(),
        operation: FirewallOperation::Create,
        direction: Some(old_direction(addr.direction)),
        action: Some(old_action(addr.action)),
        protocol: addr.protocol.map(old_protocol),
        program: addr.program.clone(),
        service: addr.service.clone(),
        remote_addresses: addr.remote_addresses.clone(),
        remote_ports: addr.remote_ports.clone(),
        local_ports: addr.local_ports.clone(),
        description: addr.description.clone(),
        skip_validation: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::Level;

    fn user_cx() -> ExecCx {
        ExecCx::new(Level::User)
    }

    /// A name that certainly does not exist -- no elevation, no real resource needed (controller
    /// decision 4, same convention as `service.rs`/`task.rs`).
    const NO_SUCH_RULE: &str = "MagicXNoSuchFirewallRule_5F3F1D2E-6A4B-4C9E-9B0A-6B6E6C7D8E9F";

    fn rule_addr(name: &str) -> RuleAddr {
        RuleAddr {
            name: name.to_string(),
            direction: FwDirection::Outbound,
            action: FwAction::Block,
            protocol: Some(FwProtocol::Tcp),
            program: None,
            service: None,
            remote_addresses: None,
            remote_ports: None,
            local_ports: None,
            description: Some("MagicX Toolbox test rule".to_string()),
        }
    }

    #[test]
    fn missing_firewall_rule_reads_present_false() {
        // `rule_exists` keys on netsh's exit status when querying -- no admin needed (confirmed by
        // firewall_service's own default-run `a_nonexistent_rule_is_reported_absent_not_present`).
        let cx = user_cx();
        let setting = Setting::Firewall(rule_addr(NO_SUCH_RULE));
        assert_eq!(
            FirewallKind.read(&setting, &cx).unwrap(),
            Value::Present(false)
        );
    }

    #[test]
    fn drive_rejects_system_and_ti_levels_for_now() {
        // guard_level fires before any netsh call -- safe to run by default.
        let setting = Setting::Firewall(rule_addr(NO_SUCH_RULE));
        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = FirewallKind
                .drive(&setting, &Value::Present(true), &cx)
                .expect_err("this build cannot yet route System/Ti through the broker");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    /// Documents the restore-fidelity limit (module doc comment / `drive_firewall`): proves the
    /// half we DO guarantee -- every field the primitive can act on survives `RuleAddr` ->
    /// `FirewallChange` unaltered -- never that it matches some prior, uncaptured rule state. Pure
    /// conversion: no netsh call, safe by default.
    #[test]
    fn firewall_recreate_uses_authored_definition() {
        let addr = RuleAddr {
            name: "Full Definition Rule".to_string(),
            direction: FwDirection::Inbound,
            action: FwAction::Allow,
            protocol: Some(FwProtocol::Udp),
            program: Some(r"C:\Program Files\App\app.exe".to_string()),
            service: Some("diagtrack".to_string()),
            remote_addresses: Some(vec!["157.56.0.0/16".to_string()]),
            remote_ports: Some("80,443".to_string()),
            local_ports: Some("1-1024".to_string()),
            description: Some("blocked by us".to_string()),
        };

        let change = to_firewall_change(&addr);

        assert_eq!(change.name, addr.name);
        assert_eq!(change.operation, FirewallOperation::Create);
        assert_eq!(change.direction, Some(FirewallDirection::Inbound));
        assert_eq!(change.action, Some(FirewallRuleAction::Allow));
        assert_eq!(change.protocol, Some(FirewallProtocol::Udp));
        assert_eq!(change.program, addr.program);
        assert_eq!(change.service, addr.service);
        assert_eq!(change.remote_addresses, addr.remote_addresses);
        assert_eq!(change.remote_ports, addr.remote_ports);
        assert_eq!(change.local_ports, addr.local_ports);
        assert_eq!(change.description, addr.description);
    }

    #[test]
    #[ignore = "creates/deletes a real firewall rule; needs admin -- run with `cargo test -- --ignored` while elevated"]
    fn firewall_rule_roundtrip() {
        const NAME: &str = "MagicX Toolbox Test Rule 5F3F1D2E";
        let setting = Setting::Firewall(rule_addr(NAME));
        let cx = ExecCx::new(Level::Admin);
        let _cleanup = DeleteFirewallRule {
            name: NAME.to_string(),
        };

        FirewallKind
            .drive(&setting, &Value::Present(true), &cx)
            .unwrap();
        assert_eq!(
            FirewallKind.read(&setting, &cx).unwrap(),
            Value::Present(true)
        );

        FirewallKind
            .drive(&setting, &Value::Present(false), &cx)
            .unwrap();
        assert_eq!(
            FirewallKind.read(&setting, &cx).unwrap(),
            Value::Present(false)
        );
    }

    /// Deletes a firewall rule on drop, even on panic, so a failed assertion never leaves a real
    /// rule behind (controller decision 4).
    struct DeleteFirewallRule {
        name: String,
    }
    impl Drop for DeleteFirewallRule {
        fn drop(&mut self) {
            // Cleanup only -- the one accepted `let _` exception (a Drop-guard restoring state).
            let _ = firewall_service::delete_firewall_rule(&self.name);
        }
    }
}
