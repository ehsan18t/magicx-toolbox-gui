//! Windows Task Scheduler service for managing scheduled tasks.
//!
//! Provides functionality to enable, disable, and delete scheduled tasks
//! using the Windows `schtasks.exe` command-line tool.

use crate::error::Error;
use crate::models::tweak::SchedulerAction;
use std::process::Command;

/// State of a scheduled task
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    /// Task is enabled and ready to run
    Ready,
    /// Task is disabled
    Disabled,
    /// Task is currently running
    Running,
    /// Task was not found
    NotFound,
    /// Unknown state
    Unknown(String),
}

impl TaskState {
    pub fn as_str(&self) -> &str {
        match self {
            TaskState::Ready => "Ready",
            TaskState::Disabled => "Disabled",
            TaskState::Running => "Running",
            TaskState::NotFound => "NotFound",
            TaskState::Unknown(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "ready" => TaskState::Ready,
            "disabled" => TaskState::Disabled,
            "running" => TaskState::Running,
            _ => TaskState::Unknown(s.to_string()),
        }
    }
}

/// Get the full task path combining path and name
fn get_full_task_path(task_path: &str, task_name: &str) -> String {
    let path = task_path.trim_end_matches('\\');
    if path.is_empty() {
        format!("\\{}", task_name)
    } else {
        format!("{}\\{}", path, task_name)
    }
}

/// Get the current state of a scheduled task
pub fn get_task_state(task_path: &str, task_name: &str) -> Result<TaskState, Error> {
    let full_path = get_full_task_path(task_path, task_name);
    log::debug!("Getting state of scheduled task: {}", full_path);

    let output = Command::new("schtasks")
        .args(["/Query", "/TN", &full_path, "/FO", "LIST", "/V"])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to execute schtasks: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if task doesn't exist
        if stderr.contains("does not exist") || stderr.contains("cannot find") {
            log::debug!("Scheduled task not found: {}", full_path);
            return Ok(TaskState::NotFound);
        }
        return Err(Error::CommandExecution(format!(
            "Failed to query task '{}': {}",
            full_path, stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the output to find the Status line
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Status:") {
            let state_str = line.strip_prefix("Status:").unwrap_or("").trim();
            let state = TaskState::from_str(state_str);
            log::debug!("Task '{}' state: {:?}", full_path, state);
            return Ok(state);
        }
    }

    log::warn!("Could not parse state for task: {}", full_path);
    Ok(TaskState::Unknown("Could not parse state".to_string()))
}

/// Enable a scheduled task
pub fn enable_task(task_path: &str, task_name: &str) -> Result<(), Error> {
    let full_path = get_full_task_path(task_path, task_name);
    log::info!("Enabling scheduled task: {}", full_path);

    let output = Command::new("schtasks")
        .args(["/Change", "/TN", &full_path, "/Enable"])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to execute schtasks: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::CommandExecution(format!(
            "Failed to enable task '{}': {}",
            full_path, stderr
        )));
    }

    log::info!("Successfully enabled scheduled task: {}", full_path);
    Ok(())
}

/// Disable a scheduled task
pub fn disable_task(task_path: &str, task_name: &str) -> Result<(), Error> {
    let full_path = get_full_task_path(task_path, task_name);
    log::info!("Disabling scheduled task: {}", full_path);

    let output = Command::new("schtasks")
        .args(["/Change", "/TN", &full_path, "/Disable"])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to execute schtasks: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::CommandExecution(format!(
            "Failed to disable task '{}': {}",
            full_path, stderr
        )));
    }

    log::info!("Successfully disabled scheduled task: {}", full_path);
    Ok(())
}

/// Delete a scheduled task
pub fn delete_task(task_path: &str, task_name: &str) -> Result<(), Error> {
    let full_path = get_full_task_path(task_path, task_name);
    log::info!("Deleting scheduled task: {}", full_path);

    let output = Command::new("schtasks")
        .args(["/Delete", "/TN", &full_path, "/F"])
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to execute schtasks: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If task doesn't exist, consider it a success
        if stderr.contains("does not exist") || stderr.contains("cannot find") {
            log::debug!("Task already deleted or not found: {}", full_path);
            return Ok(());
        }
        return Err(Error::CommandExecution(format!(
            "Failed to delete task '{}': {}",
            full_path, stderr
        )));
    }

    log::info!("Successfully deleted scheduled task: {}", full_path);
    Ok(())
}

/// Apply a scheduler change based on the action type
pub fn apply_scheduler_change(
    task_path: &str,
    task_name: &str,
    action: SchedulerAction,
) -> Result<(), Error> {
    match action {
        SchedulerAction::Enable => enable_task(task_path, task_name),
        SchedulerAction::Disable => disable_task(task_path, task_name),
        SchedulerAction::Delete => delete_task(task_path, task_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_full_task_path() {
        assert_eq!(
            get_full_task_path("\\Microsoft\\Windows\\Test", "TaskName"),
            "\\Microsoft\\Windows\\Test\\TaskName"
        );
        assert_eq!(
            get_full_task_path("\\Microsoft\\Windows\\Test\\", "TaskName"),
            "\\Microsoft\\Windows\\Test\\TaskName"
        );
        assert_eq!(get_full_task_path("", "TaskName"), "\\TaskName");
    }

    #[test]
    fn test_task_state_from_str() {
        assert_eq!(TaskState::from_str("Ready"), TaskState::Ready);
        assert_eq!(TaskState::from_str("Disabled"), TaskState::Disabled);
        assert_eq!(TaskState::from_str("Running"), TaskState::Running);
        assert_eq!(TaskState::from_str("  READY  "), TaskState::Ready);
    }
}
