//! Firewall service for managing Windows Firewall rules.
//!
//! Uses netsh advfirewall commands to create, delete, and query firewall rules.
//! Requires administrator privileges.

use crate::error::Error;
use crate::models::tweak::{
    FirewallChange, FirewallOperation, FirewallProtocol, FirewallRuleAction,
};
use std::process::Command;

/// Check if a firewall rule exists by name
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If the rule doesn't exist, netsh returns "No rules match the specified criteria"
    Ok(!stdout.contains("No rules match the specified criteria"))
}

/// Check the current state of a firewall change
#[allow(dead_code)]
pub fn check_firewall_change_status(change: &FirewallChange) -> Result<bool, Error> {
    let exists = rule_exists(&change.name)?;

    match change.operation {
        FirewallOperation::Create => Ok(exists),
        FirewallOperation::Delete => Ok(!exists),
    }
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

/// Get details about an existing firewall rule
#[allow(dead_code)]
pub fn get_rule_details(name: &str) -> Result<Option<String>, Error> {
    let output = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={}", name),
            "verbose",
        ])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to query firewall rule: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("No rules match the specified criteria") {
        return Ok(None);
    }

    Ok(Some(stdout.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rule_query_format() {
        // This is a unit test placeholder
        // Actual testing requires elevated privileges
    }
}
