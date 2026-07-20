//! Firewall service for managing Windows Firewall rules.
//!
//! Uses netsh advfirewall commands to create, delete, and query firewall rules.
//! Requires administrator privileges.

use crate::error::Error;
use crate::models::tweak::{
    FirewallChange, FirewallOperation, FirewallProtocol, FirewallRuleAction,
};
use std::process::Command;

/// Check if a firewall rule exists by name.
///
/// Keys on netsh's **exit status**, not on the localized "No rules match the specified criteria"
/// text: `netsh advfirewall firewall show rule name=X` exits 0 when the rule exists and non-zero
/// when it does not. This is locale-independent and, unlike the old text check, no longer lets a
/// genuine netsh failure masquerade as "rule exists" — which previously made `create` silently
/// no-op (a failed existence probe was read as `!contains("No rules match") == true`).
pub fn rule_exists(name: &str) -> Result<bool, Error> {
    let output = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={}", name),
        ])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to query firewall rule: {}", e)))?;

    Ok(output.status.success())
}

/// Apply a firewall change
pub fn apply_firewall_change(change: &FirewallChange) -> Result<(), Error> {
    match change.operation {
        FirewallOperation::Create => create_firewall_rule(change),
        FirewallOperation::Delete => delete_firewall_rule(&change.name),
    }
}

/// Create a new firewall rule
pub fn create_firewall_rule(change: &FirewallChange) -> Result<(), Error> {
    // Check if rule already exists
    if rule_exists(&change.name)? {
        log::debug!("Firewall rule already exists: {}", change.name);
        return Ok(());
    }

    let args = build_create_rule_args(change)?;

    // Execute netsh command
    let output = Command::new("netsh")
        .args(&args)
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to create firewall rule: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::CommandExecution(format!(
            "Failed to create firewall rule '{}': {} {}",
            change.name, stdout, stderr
        )));
    }

    log::info!("Created firewall rule: {}", change.name);
    Ok(())
}

/// Build the `netsh advfirewall firewall add rule` argument vector for a change.
///
/// Split out from `create_firewall_rule` so the argument construction can be tested
/// without executing netsh. Executing it would need admin and would mutate the real
/// machine-wide firewall -- including on CI, where the runner IS elevated.
fn build_create_rule_args(change: &FirewallChange) -> Result<Vec<String>, Error> {
    // Validate required fields for create operation
    let direction = change.direction.ok_or_else(|| {
        Error::ValidationError(format!(
            "Firewall rule '{}' requires 'direction' field for create operation",
            change.name
        ))
    })?;

    let action = change.action.ok_or_else(|| {
        Error::ValidationError(format!(
            "Firewall rule '{}' requires 'action' field for create operation",
            change.name
        ))
    })?;

    // Build the netsh command
    let mut args = vec![
        "advfirewall".to_string(),
        "firewall".to_string(),
        "add".to_string(),
        "rule".to_string(),
        format!("name={}", change.name),
        format!("dir={}", direction.as_str()),
        format!(
            "action={}",
            match action {
                FirewallRuleAction::Block => "block",
                FirewallRuleAction::Allow => "allow",
            }
        ),
    ];

    // Add optional fields
    if let Some(protocol) = &change.protocol {
        match protocol {
            FirewallProtocol::Any => args.push("protocol=any".to_string()),
            FirewallProtocol::Tcp => args.push("protocol=tcp".to_string()),
            FirewallProtocol::Udp => args.push("protocol=udp".to_string()),
            FirewallProtocol::Icmpv4 => args.push("protocol=icmpv4".to_string()),
            FirewallProtocol::Icmpv6 => args.push("protocol=icmpv6".to_string()),
        }
    }

    if let Some(program) = &change.program {
        args.push(format!("program={}", program));
    }

    if let Some(service) = &change.service {
        args.push(format!("service={}", service));
    }

    if let Some(remote_addresses) = &change.remote_addresses {
        args.push(format!("remoteip={}", remote_addresses.join(",")));
    }

    if let Some(remote_ports) = &change.remote_ports {
        args.push(format!("remoteport={}", remote_ports));
    }

    if let Some(local_ports) = &change.local_ports {
        args.push(format!("localport={}", local_ports));
    }

    if let Some(description) = &change.description {
        args.push(format!("description={}", description));
    }

    // Enable the rule by default
    args.push("enable=yes".to_string());

    Ok(args)
}

/// Delete a firewall rule by name
pub fn delete_firewall_rule(name: &str) -> Result<(), Error> {
    // Check if rule exists first
    if !rule_exists(name)? {
        log::debug!("Firewall rule does not exist: {}", name);
        return Ok(());
    }

    let output = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "delete",
            "rule",
            &format!("name={}", name),
        ])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to delete firewall rule: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::CommandExecution(format!(
            "Failed to delete firewall rule '{}': {} {}",
            name, stdout, stderr
        )));
    }

    log::info!("Deleted firewall rule: {}", name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tweak::{FirewallDirection, FirewallOperation};

    fn change(name: &str) -> FirewallChange {
        FirewallChange {
            name: name.to_string(),
            operation: FirewallOperation::Create,
            direction: Some(FirewallDirection::Outbound),
            action: Some(FirewallRuleAction::Block),
            protocol: None,
            program: None,
            service: None,
            remote_addresses: None,
            remote_ports: None,
            local_ports: None,
            description: None,
            skip_validation: false,
        }
    }

    #[test]
    fn a_minimal_rule_produces_the_expected_netsh_arguments() {
        let args = build_create_rule_args(&change("Block Telemetry")).unwrap();
        assert_eq!(
            args,
            vec![
                "advfirewall",
                "firewall",
                "add",
                "rule",
                "name=Block Telemetry",
                "dir=out",
                "action=block",
                "enable=yes",
            ]
        );
    }

    #[test]
    fn direction_and_action_are_required_for_create() {
        let mut c = change("No Direction");
        c.direction = None;
        assert!(build_create_rule_args(&c).is_err());

        let mut c = change("No Action");
        c.action = None;
        assert!(build_create_rule_args(&c).is_err());
    }

    #[test]
    fn optional_fields_appear_only_when_set_and_addresses_are_comma_joined() {
        let mut c = change("Full");
        c.protocol = Some(FirewallProtocol::Tcp);
        c.program = Some(r"C:\Program Files\App\app.exe".to_string());
        c.service = Some("diagtrack".to_string());
        c.remote_addresses = Some(vec!["157.56.0.0/16".into(), "168.62.0.0/16".into()]);
        c.remote_ports = Some("80,443".to_string());
        c.local_ports = Some("1-1024".to_string());
        c.description = Some("blocked by us".to_string());

        let args = build_create_rule_args(&c).unwrap();
        assert!(args.contains(&"protocol=tcp".to_string()));
        assert!(args.contains(&r"program=C:\Program Files\App\app.exe".to_string()));
        assert!(args.contains(&"service=diagtrack".to_string()));
        assert!(args.contains(&"remoteip=157.56.0.0/16,168.62.0.0/16".to_string()));
        assert!(args.contains(&"remoteport=80,443".to_string()));
        assert!(args.contains(&"localport=1-1024".to_string()));
        assert!(args.contains(&"description=blocked by us".to_string()));

        // A rule with nothing optional set must not carry empty placeholders.
        let minimal = build_create_rule_args(&change("Minimal")).unwrap();
        assert!(!minimal.iter().any(|a| a.starts_with("protocol=")));
        assert!(!minimal.iter().any(|a| a.starts_with("program=")));
        assert!(!minimal.iter().any(|a| a.starts_with("description=")));
    }

    /// The audit flagged YAML-authored firewall strings as a command-injection
    /// surface. This pins why they are NOT: the args go to `Command::new("netsh")`
    /// via `.args()`, which builds the argv directly through CreateProcessW. No
    /// shell is involved, so `&`, `|`, `"` and newlines are inert data.
    ///
    /// What they DO risk is netsh's own `key=value` parsing, so each value must stay
    /// in exactly ONE argv element -- never split, never merged.
    #[test]
    fn hostile_characters_in_a_rule_name_stay_in_a_single_argument() {
        let hostile = r#"evil" & calc.exe & echo "pwned"#;
        let args = build_create_rule_args(&change(hostile)).unwrap();

        let name_args: Vec<&String> = args.iter().filter(|a| a.starts_with("name=")).collect();
        assert_eq!(name_args.len(), 1, "the name was split across arguments");
        assert_eq!(
            name_args[0],
            &format!("name={}", hostile),
            "the name was altered or truncated"
        );

        // Nothing hostile leaked into a separate argument that netsh would read as
        // another key=value pair.
        assert_eq!(
            args.len(),
            8,
            "unexpected extra arguments were produced: {:?}",
            args
        );
    }

    #[test]
    fn a_newline_in_a_description_does_not_create_a_second_argument() {
        let mut c = change("Newline");
        c.description = Some("first\r\nsecond".to_string());
        let args = build_create_rule_args(&c).unwrap();
        let descs: Vec<&String> = args
            .iter()
            .filter(|a| a.starts_with("description="))
            .collect();
        assert_eq!(descs.len(), 1);
        assert_eq!(descs[0], "description=first\r\nsecond");
    }

    #[test]
    fn a_nonexistent_rule_is_reported_absent_not_present() {
        // Locale-free: netsh exits non-zero for a name that matches no rule, so rule_exists must
        // return Ok(false) — never Ok(true) (which would make create silently no-op) and never
        // Err. This also empirically confirms the exit-code contract on the running machine.
        let exists = rule_exists("MagicXNoSuchFirewallRule_zzq").unwrap();
        assert!(!exists, "a non-existent rule must be reported as absent");
    }
}
