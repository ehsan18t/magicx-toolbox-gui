//! Windows Task Scheduler service for managing scheduled tasks.
//!
//! Provides functionality to enable, disable, and delete scheduled tasks
//! using the Windows `schtasks.exe` command-line tool.
//!
//! Supports both exact task names and regex patterns for matching multiple tasks.

use crate::error::Error;
use crate::models::tweak::SchedulerAction;
use regex::Regex;
use std::os::windows::process::CommandExt;
use std::process::Command;

/// CREATE_NO_WINDOW flag to prevent console window from appearing
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
        .creation_flags(CREATE_NO_WINDOW)
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
        .creation_flags(CREATE_NO_WINDOW)
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
        .creation_flags(CREATE_NO_WINDOW)
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
        .creation_flags(CREATE_NO_WINDOW)
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

/// Represents a task found in a folder
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub name: String,
    pub state: TaskState,
}

/// List all tasks in a folder path
pub fn list_tasks_in_folder(task_path: &str) -> Result<Vec<TaskInfo>, Error> {
    let path = task_path.trim_end_matches('\\');
    log::debug!("Listing tasks in folder: {}", path);

    let output = Command::new("schtasks")
        .args(["/Query", "/TN", &format!("{}\\", path), "/FO", "LIST", "/V"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to execute schtasks: {}", e)))?;

    // If folder doesn't exist, return empty list
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not exist") || stderr.contains("cannot find") {
            log::debug!("Task folder not found: {}", path);
            return Ok(Vec::new());
        }
        return Err(Error::CommandExecution(format!(
            "Failed to list tasks in '{}': {}",
            path, stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tasks = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_state: Option<TaskState> = None;

    // Parse the LIST /V output which contains multiple tasks
    // Each task has "TaskName:" and "Status:" fields
    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("TaskName:") {
            // Save previous task if we have one
            if let (Some(name), Some(state)) = (current_name.take(), current_state.take()) {
                tasks.push(TaskInfo { name, state });
            }

            // Extract task name (full path, we just want the name part)
            let full_name = line.strip_prefix("TaskName:").unwrap_or("").trim();
            // Get just the task name from full path like \Microsoft\Windows\Folder\TaskName
            if let Some(name) = full_name.rsplit('\\').next() {
                if !name.is_empty() {
                    current_name = Some(name.to_string());
                }
            }
        } else if line.starts_with("Status:") {
            let state_str = line.strip_prefix("Status:").unwrap_or("").trim();
            current_state = Some(TaskState::from_str(state_str));
        }
    }

    // Don't forget the last task
    if let (Some(name), Some(state)) = (current_name, current_state) {
        tasks.push(TaskInfo { name, state });
    }

    log::debug!("Found {} tasks in folder '{}'", tasks.len(), path);
    Ok(tasks)
}

/// Find tasks matching a regex pattern in a folder
pub fn find_tasks_by_pattern(task_path: &str, pattern: &str) -> Result<Vec<TaskInfo>, Error> {
    log::debug!(
        "Finding tasks matching pattern '{}' in '{}'",
        pattern,
        task_path
    );

    let regex = Regex::new(pattern).map_err(|e| {
        Error::CommandExecution(format!("Invalid regex pattern '{}': {}", pattern, e))
    })?;

    let all_tasks = list_tasks_in_folder(task_path)?;
    let matching: Vec<TaskInfo> = all_tasks
        .into_iter()
        .filter(|t| regex.is_match(&t.name))
        .collect();

    log::debug!(
        "Found {} tasks matching pattern '{}' in '{}'",
        matching.len(),
        pattern,
        task_path
    );

    Ok(matching)
}

/// Apply action to multiple tasks found by pattern
/// Returns (success_count, error_count, errors)
pub fn apply_action_to_pattern(
    task_path: &str,
    pattern: &str,
    action: SchedulerAction,
    ignore_not_found: bool,
) -> Result<(usize, usize, Vec<String>), Error> {
    let tasks = find_tasks_by_pattern(task_path, pattern)?;

    if tasks.is_empty() {
        if ignore_not_found {
            log::warn!(
                "No tasks found matching pattern '{}' in '{}' (ignore_not_found=true)",
                pattern,
                task_path
            );
            return Ok((0, 0, Vec::new()));
        } else {
            return Err(Error::CommandExecution(format!(
                "No tasks found matching pattern '{}' in '{}'",
                pattern, task_path
            )));
        }
    }

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for task in tasks {
        log::info!(
            "Applying {:?} to task '{}\\{}'",
            action,
            task_path,
            task.name
        );

        match apply_scheduler_change(task_path, &task.name, action) {
            Ok(()) => {
                success_count += 1;
            }
            Err(e) => {
                error_count += 1;
                let err_msg = format!("{}\\{}: {}", task_path, task.name, e);
                errors.push(err_msg);
            }
        }
    }

    Ok((success_count, error_count, errors))
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
