//! `EffectKind` for `Setting::Task` (spec §5.1/§5.4). Wraps the Task Scheduler COM primitive
//! (`scheduler_service`), splitting the address's single exact path into the COM API's separate
//! folder + task name.
//!
//! `Missing` (spec §5.4): a task the scheduler does not know about (missing folder or missing
//! task -- `scheduler_service::get_task_state` already collapses both to `TaskState::NotFound`,
//! exactly as the registry kind collapses "missing key" into "missing value") reads
//! `Ok(Value::Missing)`, never an error; driving *to* `Missing` is a verified no-op; driving a
//! real `TaskEnabled` value at a task that turns out missing is the typed
//! [`Error::ResourceMissing`] (invariant 12).
//!
//! Test-safety note: every call into `scheduler_service` activates live Task Scheduler COM, which
//! this repo already knows races libtest's per-test thread churn into a STATUS_ACCESS_VIOLATION
//! (see `scheduler_service.rs`'s own `#[ignore]`d tests, which ignore even read-only queries for
//! exactly this reason). Only the `Value::Missing` drive arm (a pure no-op — no COM call at all)
//! and the pure `map_task_state` decision function run by default here; everything else that
//! reaches `scheduler_service` is `#[ignore]`d, matching that file's own convention.

use crate::error::Error as BackendError;
use crate::models::SchedulerAction;
use crate::services::elevation::BrokerOp;
use crate::services::scheduler_service::{self, TaskState};
use crate::tweaks::model::{Setting, TaskAddr, Value};

use super::{guard_level, map_backend_error, EffectKind, Error, ExecCx};

/// `EffectKind` for `Setting::Task`.
pub struct TaskKind;

impl EffectKind for TaskKind {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        match s {
            Setting::Task(addr) => read_task(addr),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Service(_)
            | Setting::Hosts(_)
            | Setting::Firewall(_) => Err(Error::Invalid("TaskKind cannot read this Setting")),
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        match s {
            Setting::Task(addr) => drive_task(addr, target),
            Setting::Registry(_)
            | Setting::RegistryKey(_)
            | Setting::Service(_)
            | Setting::Hosts(_)
            | Setting::Firewall(_) => Err(Error::Invalid("TaskKind cannot drive this Setting")),
        }
    }
}

/// Splits an exact task path (spec §5.1: "exact task path — no patterns") into the COM API's
/// separate folder + name, e.g. `\Microsoft\Windows\DiskCleanup\SilentCleanup` ->
/// (`\Microsoft\Windows\DiskCleanup`, `SilentCleanup`).
fn split_task_path(path: &str) -> Result<(String, &str), Error> {
    if !path.starts_with('\\') {
        return Err(Error::Invalid(
            "a task path must be absolute (start with '\\')",
        ));
    }
    let idx = path.rfind('\\').expect("checked above: contains '\\'");
    let folder = if idx == 0 {
        "\\".to_string()
    } else {
        path[..idx].to_string()
    };
    let name = &path[idx + 1..];
    if name.is_empty() {
        return Err(Error::Invalid(
            "a task path must name a task, not just a folder",
        ));
    }
    Ok((folder, name))
}

/// The not-found/backend-error/state decision, isolated from the COM call itself so the
/// Missing/error distinction (invariant 2/12) is unit-testable without touching live COM.
fn map_task_state(result: Result<TaskState, BackendError>) -> Result<Value, Error> {
    match result.map_err(map_backend_error)? {
        TaskState::NotFound => Ok(Value::Missing),
        TaskState::Disabled => Ok(Value::TaskEnabled(false)),
        TaskState::Ready | TaskState::Running => Ok(Value::TaskEnabled(true)),
        TaskState::Unknown(s) => Err(Error::Backend(format!("unrecognized task state: {s}"))),
    }
}

fn read_task(addr: &TaskAddr) -> Result<Value, Error> {
    let (folder, name) = split_task_path(&addr.path)?;
    map_task_state(scheduler_service::get_task_state(&folder, name))
}

fn drive_task(addr: &TaskAddr, target: &Value) -> Result<(), Error> {
    match target {
        // The engine never installs/uninstalls a task (spec §5.4, invariant 12): a defined no-op
        // regardless of whether the task currently exists.
        Value::Missing => Ok(()),
        Value::TaskEnabled(enabled) => {
            let (folder, name) = split_task_path(&addr.path)?;
            if map_task_state(scheduler_service::get_task_state(&folder, name))? == Value::Missing {
                return Err(Error::ResourceMissing(format!(
                    "scheduled task '{}' does not exist",
                    addr.path
                )));
            }
            let result = if *enabled {
                scheduler_service::enable_task(&folder, name)
            } else {
                scheduler_service::disable_task(&folder, name)
            };
            result.map_err(map_backend_error)
        }
        _ => Err(Error::Invalid(
            "a task can only be driven to TaskEnabled or Missing",
        )),
    }
}

/// Translates a System/TI-level task drive into the broker's typed op (spec §9): mirrors
/// `drive_task` mechanically, minus the existence pre-check `drive_task` performs in-process (that
/// read runs at the CURRENT level -- invariant 24 -- so it is not repeated here; an already-missing
/// task simply reports its own COM error through the broker rather than the typed
/// `ResourceMissing` the in-process path gives). Driving to `Missing` is the same no-op it always
/// is (spec §5.4).
pub(crate) fn to_broker_ops(s: &Setting, target: &Value) -> Result<Vec<BrokerOp>, Error> {
    let Setting::Task(addr) = s else {
        return Err(Error::Invalid("TaskKind cannot drive this Setting"));
    };
    match target {
        Value::Missing => Ok(Vec::new()),
        Value::TaskEnabled(enabled) => {
            let (folder, name) = split_task_path(&addr.path)?;
            Ok(vec![BrokerOp::Scheduler {
                task_path: folder,
                task_name: name.to_string(),
                action: if *enabled {
                    SchedulerAction::Enable
                } else {
                    SchedulerAction::Disable
                },
            }])
        }
        _ => Err(Error::Invalid(
            "a task can only be driven to TaskEnabled or Missing",
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

    #[test]
    fn split_task_path_handles_root_and_nested() {
        assert_eq!(split_task_path("\\Foo").unwrap(), ("\\".to_string(), "Foo"));
        assert_eq!(
            split_task_path("\\Microsoft\\Windows\\DiskCleanup\\SilentCleanup").unwrap(),
            (
                "\\Microsoft\\Windows\\DiskCleanup".to_string(),
                "SilentCleanup"
            )
        );
    }

    #[test]
    fn split_task_path_rejects_malformed_paths() {
        assert!(
            matches!(split_task_path("Foo"), Err(Error::Invalid(_))),
            "must require a leading backslash"
        );
        assert!(
            matches!(split_task_path("\\Foo\\"), Err(Error::Invalid(_))),
            "must reject an empty task name"
        );
    }

    #[test]
    fn missing_task_reads_missing() {
        // Pure: no real COM activation (controller decision 4 -- no elevation, no real resource).
        let missing = map_task_state(Ok(TaskState::NotFound));
        assert!(matches!(missing, Ok(Value::Missing)), "got {missing:?}");
    }

    #[test]
    fn error_can_never_collapse_into_missing() {
        let missing = map_task_state(Ok(TaskState::NotFound));
        assert!(matches!(missing, Ok(Value::Missing)), "got {missing:?}");

        // Any backend Err is `?`-propagated before the Missing-producing branch is reachable --
        // see service.rs's `error_can_never_collapse_into_missing` for the fuller note. This does
        // NOT prove a real access-denied surfaces distinctly: `scheduler_service` has no
        // distinguished access-denied variant either (its COM failures all land in
        // `CommandExecution`, then `map_backend_error`'s `Backend` catch-all). Deferred to the
        // detection task, not fixed here.
        let denied = map_task_state(Err(BackendError::RequiresAdmin));
        assert!(
            matches!(denied, Err(Error::AccessDenied(_))),
            "got {denied:?}"
        );
    }

    #[test]
    fn drive_to_missing_is_noop_ok() {
        // The Missing arm never touches scheduler_service -- no COM activation, safe by default.
        let cx = user_cx();
        let setting = Setting::Task(TaskAddr {
            path: "\\MagicXNoSuchFolder_5F3F1D2E\\NoSuchTask_6A4B4C9E".to_string(),
        });
        TaskKind
            .drive(&setting, &Value::Missing, &cx)
            .expect("driving a task to Missing must be a no-op success");
    }

    /// Pure translation, no COM activation at all (spec §9) -- `to_broker_ops` never calls
    /// `scheduler_service`, unlike `drive_task`.
    #[test]
    fn to_broker_ops_translates_enabled_and_missing() {
        let setting = Setting::Task(TaskAddr {
            path: "\\Microsoft\\Windows\\DiskCleanup\\SilentCleanup".to_string(),
        });

        let enable = to_broker_ops(&setting, &Value::TaskEnabled(true)).unwrap();
        assert_eq!(enable.len(), 1);
        match &enable[0] {
            BrokerOp::Scheduler {
                task_path,
                task_name,
                action,
            } => {
                assert_eq!(task_path, "\\Microsoft\\Windows\\DiskCleanup");
                assert_eq!(task_name, "SilentCleanup");
                assert_eq!(*action, SchedulerAction::Enable);
            }
            other => panic!("expected Scheduler, got {other:?}"),
        }

        let disable = to_broker_ops(&setting, &Value::TaskEnabled(false)).unwrap();
        match &disable[0] {
            BrokerOp::Scheduler { action, .. } => assert_eq!(*action, SchedulerAction::Disable),
            other => panic!("expected Scheduler, got {other:?}"),
        }

        let missing = to_broker_ops(&setting, &Value::Missing)
            .expect("driving to Missing must translate, never error");
        assert!(missing.is_empty(), "Missing is a no-op -- no ops to run");
    }

    #[test]
    fn drive_rejects_system_and_ti_levels_for_now() {
        // guard_level fires before any COM call -- safe to run by default.
        let setting = Setting::Task(TaskAddr {
            path: "\\Irrelevant".to_string(),
        });
        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = TaskKind
                .drive(&setting, &Value::TaskEnabled(true), &cx)
                .expect_err("this build cannot yet route System/Ti through the broker");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    #[test]
    #[ignore = "activates live Task Scheduler COM; races libtest thread-churn -- run with --ignored"]
    fn drive_real_value_at_missing_resource_is_typed_error() {
        let cx = user_cx();
        let setting = Setting::Task(TaskAddr {
            path: "\\MagicXNoSuchFolder_5F3F1D2E\\NoSuchTask_6A4B4C9E".to_string(),
        });
        let err = TaskKind
            .drive(&setting, &Value::TaskEnabled(true), &cx)
            .expect_err("driving a real value at a missing task must be a typed error");
        assert!(matches!(err, Error::ResourceMissing(_)), "got {err:?}");
    }

    #[test]
    #[ignore = "writes real Task Scheduler config; needs admin, activates live COM -- run with `cargo test -- --ignored` while elevated"]
    fn task_enable_roundtrip() {
        // SilentCleanup: built-in disk-cleanup maintenance task present on every edition, gated
        // behind an idle/maintenance window -- toggling it transiently has no visible effect.
        const PATH: &str = "\\Microsoft\\Windows\\DiskCleanup\\SilentCleanup";
        let setting = Setting::Task(TaskAddr {
            path: PATH.to_string(),
        });
        let cx = ExecCx::new(Level::Admin);

        let original = match TaskKind
            .read(&setting, &cx)
            .expect("designated task must exist")
        {
            Value::TaskEnabled(b) => b,
            other => panic!("expected Value::TaskEnabled, got {other:?}"),
        };
        let _restore = RestoreTaskEnabled {
            path: PATH.to_string(),
            original,
        };

        for target in [!original, original] {
            TaskKind
                .drive(&setting, &Value::TaskEnabled(target), &cx)
                .unwrap_or_else(|e| panic!("drive to {target} failed: {e}"));
            assert_eq!(
                TaskKind.read(&setting, &cx).unwrap(),
                Value::TaskEnabled(target)
            );
        }
    }

    /// Restores a real task's enabled state on drop, even on panic, so a failed assertion never
    /// leaves a live task permanently mutated.
    struct RestoreTaskEnabled {
        path: String,
        original: bool,
    }
    impl Drop for RestoreTaskEnabled {
        fn drop(&mut self) {
            let setting = Setting::Task(TaskAddr {
                path: self.path.clone(),
            });
            let cx = ExecCx::new(Level::Admin);
            // Cleanup only -- the one accepted `let _` exception (a Drop-guard restoring state).
            let _ = TaskKind.drive(&setting, &Value::TaskEnabled(self.original), &cx);
        }
    }
}
