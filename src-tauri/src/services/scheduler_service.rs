//! Windows Task Scheduler service for managing scheduled tasks.
//!
//! Uses the Task Scheduler 2.0 COM API (via the `windows` crate) rather than parsing
//! `schtasks.exe` text output. `IRegisteredTask::State()` returns a numeric `TASK_STATE`, which is
//! the actual fix for the locale class: the old code parsed the localized "Status:" line, so it
//! silently misread state on non-English Windows.
//!
//! Supports both exact task names and regex patterns for matching multiple tasks.

use crate::error::Error;
use crate::models::tweak::SchedulerAction;
use regex_lite::Regex;

use windows::core::BSTR;
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::TaskScheduler::{ITaskService, TaskScheduler, TASK_ENUM_HIDDEN};
use windows::Win32::System::Variant::VARIANT;

// TASK_STATE numeric values (the locale-free source of truth).
const TASK_STATE_DISABLED: i32 = 1;
const TASK_STATE_QUEUED: i32 = 2;
const TASK_STATE_READY: i32 = 3;
const TASK_STATE_RUNNING: i32 = 4;

// "Not found" HRESULTs (ERROR_FILE_NOT_FOUND / ERROR_PATH_NOT_FOUND as HRESULT).
const HRESULT_FILE_NOT_FOUND: u32 = 0x8007_0002;
const HRESULT_PATH_NOT_FOUND: u32 = 0x8007_0003;

/// State of a scheduled task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    /// Task is enabled and ready to run.
    Ready,
    /// Task is disabled.
    Disabled,
    /// Task is currently running.
    Running,
    /// Task was not found.
    NotFound,
    /// Unknown state.
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

/// Represents a task found in a folder.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub name: String,
    pub state: TaskState,
}

fn com_err(e: windows::core::Error) -> Error {
    Error::CommandExecution(format!("Task Scheduler COM error: {}", e))
}

fn is_not_found(e: &windows::core::Error) -> bool {
    let code = e.code().0 as u32;
    code == HRESULT_FILE_NOT_FOUND || code == HRESULT_PATH_NOT_FOUND
}

/// Map the numeric `TASK_STATE` to our locale-free `TaskState`. A queued task is enabled (an
/// instance is merely waiting), so it is treated as `Ready` for detection purposes.
fn task_state_from_com(state: i32) -> TaskState {
    match state {
        TASK_STATE_DISABLED => TaskState::Disabled,
        TASK_STATE_READY | TASK_STATE_QUEUED => TaskState::Ready,
        TASK_STATE_RUNNING => TaskState::Running,
        other => TaskState::Unknown(format!("TASK_STATE({})", other)),
    }
}

/// Connect to the local Task Scheduler service. COM is initialized on the current thread if it is
/// not already; an existing initialization (any apartment) is tolerated.
fn task_service() -> Result<ITaskService, Error> {
    // SAFETY: standard Task Scheduler 2.0 connect sequence.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let service: ITaskService =
            CoCreateInstance(&TaskScheduler, None, CLSCTX_ALL).map_err(com_err)?;
        service
            .Connect(
                &VARIANT::default(),
                &VARIANT::default(),
                &VARIANT::default(),
                &VARIANT::default(),
            )
            .map_err(com_err)?;
        Ok(service)
    }
}

/// Get the current state of a scheduled task.
pub fn get_task_state(task_path: &str, task_name: &str) -> Result<TaskState, Error> {
    // SAFETY: interface pointers are owned by RAII wrappers from the `windows` crate.
    unsafe {
        let service = task_service()?;
        let folder = match service.GetFolder(&BSTR::from(task_path)) {
            Ok(f) => f,
            Err(e) if is_not_found(&e) => return Ok(TaskState::NotFound),
            Err(e) => return Err(com_err(e)),
        };
        let task = match folder.GetTask(&BSTR::from(task_name)) {
            Ok(t) => t,
            Err(e) if is_not_found(&e) => return Ok(TaskState::NotFound),
            Err(e) => return Err(com_err(e)),
        };
        Ok(task_state_from_com(task.State().map_err(com_err)?.0))
    }
}

/// Enable a scheduled task.
pub fn enable_task(task_path: &str, task_name: &str) -> Result<(), Error> {
    log::info!("Enabling scheduled task: {}\\{}", task_path, task_name);
    // SAFETY: as above.
    unsafe {
        let service = task_service()?;
        let folder = service.GetFolder(&BSTR::from(task_path)).map_err(com_err)?;
        let task = folder.GetTask(&BSTR::from(task_name)).map_err(com_err)?;
        task.SetEnabled(VARIANT_TRUE).map_err(com_err)?;
    }
    Ok(())
}

/// Disable a scheduled task.
pub fn disable_task(task_path: &str, task_name: &str) -> Result<(), Error> {
    log::info!("Disabling scheduled task: {}\\{}", task_path, task_name);
    // SAFETY: as above.
    unsafe {
        let service = task_service()?;
        let folder = service.GetFolder(&BSTR::from(task_path)).map_err(com_err)?;
        let task = folder.GetTask(&BSTR::from(task_name)).map_err(com_err)?;
        task.SetEnabled(VARIANT_FALSE).map_err(com_err)?;
    }
    Ok(())
}

/// Delete a scheduled task. A task (or folder) that is already gone is treated as success.
pub fn delete_task(task_path: &str, task_name: &str) -> Result<(), Error> {
    log::info!("Deleting scheduled task: {}\\{}", task_path, task_name);
    // SAFETY: as above.
    unsafe {
        let service = task_service()?;
        let folder = match service.GetFolder(&BSTR::from(task_path)) {
            Ok(f) => f,
            Err(e) if is_not_found(&e) => return Ok(()),
            Err(e) => return Err(com_err(e)),
        };
        match folder.DeleteTask(&BSTR::from(task_name), 0) {
            Ok(()) => Ok(()),
            Err(e) if is_not_found(&e) => Ok(()),
            Err(e) => Err(com_err(e)),
        }
    }
}

/// Apply a scheduler change based on the action type.
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

/// List all tasks directly in a folder path. A missing folder yields an empty list.
pub fn list_tasks_in_folder(task_path: &str) -> Result<Vec<TaskInfo>, Error> {
    // SAFETY: as above; the collection is 1-indexed per the Task Scheduler contract.
    unsafe {
        let service = task_service()?;
        let folder = match service.GetFolder(&BSTR::from(task_path)) {
            Ok(f) => f,
            Err(e) if is_not_found(&e) => return Ok(Vec::new()),
            Err(e) => return Err(com_err(e)),
        };
        let tasks = folder.GetTasks(TASK_ENUM_HIDDEN.0).map_err(com_err)?;
        let count = tasks.Count().map_err(com_err)?;

        let mut result = Vec::with_capacity(count.max(0) as usize);
        for i in 1..=count {
            let item = tasks.get_Item(&VARIANT::from(i)).map_err(com_err)?;
            let name = item.Name().map_err(com_err)?.to_string();
            let state = task_state_from_com(item.State().map_err(com_err)?.0);
            result.push(TaskInfo { name, state });
        }
        Ok(result)
    }
}

/// Find tasks matching a regex pattern in a folder.
pub fn find_tasks_by_pattern(task_path: &str, pattern: &str) -> Result<Vec<TaskInfo>, Error> {
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

/// Apply action to multiple tasks found by pattern.
/// Returns `(success_count, error_count, errors)`.
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
            Ok(()) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(format!("{}\\{}: {}", task_path, task.name, e));
            }
        }
    }

    Ok((success_count, error_count, errors))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_state_from_com_maps_numeric_states() {
        assert_eq!(task_state_from_com(TASK_STATE_DISABLED), TaskState::Disabled);
        assert_eq!(task_state_from_com(TASK_STATE_READY), TaskState::Ready);
        assert_eq!(task_state_from_com(TASK_STATE_QUEUED), TaskState::Ready);
        assert_eq!(task_state_from_com(TASK_STATE_RUNNING), TaskState::Running);
        assert!(matches!(task_state_from_com(0), TaskState::Unknown(_)));
    }

    #[test]
    fn task_state_from_str_parses_known_states() {
        assert_eq!(TaskState::from_str("Ready"), TaskState::Ready);
        assert_eq!(TaskState::from_str("Disabled"), TaskState::Disabled);
        assert_eq!(TaskState::from_str("Running"), TaskState::Running);
        assert_eq!(TaskState::from_str("  READY  "), TaskState::Ready);
    }

    #[test]
    fn nonexistent_task_reports_not_found() {
        // The root folder exists; the task does not.
        let s = get_task_state("\\", "MagicXNoSuchTask_zzq").unwrap();
        assert_eq!(s, TaskState::NotFound);
    }

    #[test]
    fn listing_root_folder_succeeds() {
        // Exercises Connect -> GetFolder -> GetTasks -> Count/Item/Name/State without needing a
        // specific task to exist. Root may hold zero or more tasks; either way it must not error.
        let tasks = list_tasks_in_folder("\\").expect("listing root tasks folder should succeed");
        // Every returned task has a non-empty name and a mapped state.
        for t in &tasks {
            assert!(!t.name.is_empty());
        }
    }
}
