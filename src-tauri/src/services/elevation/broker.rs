//! The elevated effect broker.
//!
//! The broker is nothing more than the effect services running in an elevated process. Instead of
//! composing `cmd.exe /c <string>` command lines and escaping values (the source of the injection
//! and REG_SZ-corruption classes), the main app serializes a list of **typed** operations, spawns
//! this broker with a SYSTEM or TrustedInstaller token, and the broker runs the very same effect
//! functions the unelevated path uses, now succeeding on protected resources because the process
//! holds the elevated token.
//!
//! Transport is a request file + a response file (paths passed as argv to `--broker`), so no shell
//! ever parses anything and every result crosses back as typed data. There is no interpreter op:
//! every variant of [`BrokerOp`] names a typed effect, so nothing the elevated child can be asked
//! to do is "run this string".

use crate::error::Error;
use crate::models::{RegistryHive, RegistryValueType, SchedulerAction, ServiceStartupType};
use crate::services::exclusive_temp::{self, ExclusiveTempFile};
use crate::services::{registry_service, registry_value, scheduler_service, service_control};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ};

use super::common::SpawnError;
use super::Elevation;

/// One typed operation for the broker to perform in the elevated process.
///
/// Every variant must have a producer in `tweaks::kinds` (`to_broker_op`/`to_broker_ops`). A
/// variant with no producer is still reachable by anything that can hand the child a request file,
/// so it is pure attack surface: add one only together with the translation that emits it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrokerOp {
    /// Set a typed registry value.
    RegSet {
        hive: RegistryHive,
        key: String,
        value_name: String,
        value_type: RegistryValueType,
        value: serde_json::Value,
    },
    /// Delete a registry value (absent value is success).
    RegDeleteValue {
        hive: RegistryHive,
        key: String,
        value_name: String,
    },
    /// Delete a registry key recursively (absent key is success).
    RegDeleteKey { hive: RegistryHive, key: String },
    /// Create an empty registry key.
    RegCreateKey { hive: RegistryHive, key: String },
    /// Set a service's startup type.
    SvcSetStartup {
        name: String,
        startup: ServiceStartupType,
    },
    /// Enable / disable / delete a scheduled task.
    Scheduler {
        task_path: String,
        task_name: String,
        action: SchedulerAction,
    },
}

/// A batch of operations for one broker invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrokerRequest {
    /// Transport nonce. Assigned freshly by [`run_elevated_broker`] at send time and echoed back in
    /// the response, so a stale or foreign response file (e.g. a leftover from a prior run at a
    /// reused pid) is detected rather than read as a fresh success. Callers may leave it 0.
    #[serde(default)]
    pub nonce: u64,
    pub ops: Vec<BrokerOp>,
}

/// The op a batch died on, by position in the request's `ops`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpFailure {
    pub index: usize,
    pub message: String,
}

/// The broker's typed response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrokerResponse {
    /// Echoes the request's [`BrokerRequest::nonce`] so the parent can reject a stale/foreign file.
    #[serde(default)]
    pub nonce: u64,
    /// How many ops the child attempted. A batch stops at its first failure, so this is below the
    /// request's length exactly when `failure` is set. [`check_response`] requires the two to agree,
    /// which is what stops a truncated or forged response from reading as a completed batch.
    pub attempted: usize,
    /// The first op that failed. `None` alongside a full `attempted` count is the only success.
    pub failure: Option<OpFailure>,
}

/// Map registry "not found" into success for delete operations (deleting an absent thing is done).
fn delete_ok(result: Result<(), Error>) -> Result<(), Error> {
    match result {
        Err(Error::RegistryKeyNotFound(_)) => Ok(()),
        other => other,
    }
}

/// Execute one operation using the effect services, at whatever privilege this process holds. In
/// the broker child that is the elevated token; under `Elevation::None` it is the app's own.
pub fn execute_op(op: &BrokerOp) -> Result<(), Error> {
    match op {
        BrokerOp::RegSet {
            hive,
            key,
            value_name,
            value_type,
            value,
        } => registry_value::write_registry_json_value(hive, key, value_name, value_type, value),
        BrokerOp::RegDeleteValue {
            hive,
            key,
            value_name,
        } => delete_ok(registry_service::delete_value(hive, key, value_name)),
        BrokerOp::RegDeleteKey { hive, key } => delete_ok(registry_service::delete_key(hive, key)),
        BrokerOp::RegCreateKey { hive, key } => registry_service::create_key(hive, key),
        BrokerOp::SvcSetStartup { name, startup } => {
            service_control::set_service_startup(name, startup)
        }
        BrokerOp::Scheduler {
            task_path,
            task_name,
            action,
        } => scheduler_service::apply_scheduler_change(task_path, task_name, *action),
    }
}

/// Execute ops in declaration order, stopping at the first failure. A batch is not a set of
/// independent attempts: a later op can depend on an earlier one having actually happened (Service's
/// `SvcSetStartup` plus its `DelayedAutostart` companion write), and running the companion after the
/// primary failed would leave the registry in a state the in-process `drive_service`, which
/// `?`-aborts on the same failure, never produces.
pub fn execute_request(request: &BrokerRequest) -> BrokerResponse {
    let mut attempted = 0;
    let mut failure = None;
    for (index, op) in request.ops.iter().enumerate() {
        attempted += 1;
        if let Err(e) = execute_op(op) {
            failure = Some(OpFailure {
                index,
                message: e.to_string(),
            });
            break;
        }
    }
    BrokerResponse {
        nonce: request.nonce,
        attempted,
        failure,
    }
}

// Transport-failure exit codes. The exit code is the child's ONLY channel: it runs before the
// logger is initialized and is spawned with no console and no inherited stderr, so anything it
// wrote would be discarded. `describe_broker_exit` turns each one back into a phrase in the
// parent, where the error can actually reach the user.
const EXIT_UNREADABLE_REQUEST: i32 = 2;
const EXIT_UNPARSEABLE_REQUEST: i32 = 3;
const EXIT_UNSERIALIZABLE_RESPONSE: i32 = 4;
const EXIT_UNWRITABLE_RESPONSE: i32 = 5;
/// A panic inside the child. Distinct from the catch-all so a bug in our own executor is never
/// reported as an external kill: under `panic = "abort"` the two are otherwise indistinguishable,
/// and "crashed or was terminated" sends a support engineer looking at antivirus instead of at us.
const EXIT_PANICKED: i32 = 6;

fn describe_broker_exit(code: i32) -> &'static str {
    match code {
        EXIT_UNREADABLE_REQUEST => "could not read the request file",
        EXIT_UNPARSEABLE_REQUEST => "request file was not valid JSON",
        EXIT_UNSERIALIZABLE_RESPONSE => "could not serialize the response",
        EXIT_UNWRITABLE_RESPONSE => "could not write the response file",
        EXIT_PANICKED => "panicked while executing the batch",
        _ => "crashed or was terminated before writing a response",
    }
}

/// Broker entrypoint: read a request file, execute it, write a response file. Returns a process
/// exit code. 0 means the batch was executed and a response was written; non-zero is a transport
/// failure, distinct from op failures, which are reported inside the response.
pub fn run_broker(req_path: &str, resp_path: &str) -> i32 {
    install_panic_exit_hook();

    let Ok(bytes) = std::fs::read(req_path) else {
        return EXIT_UNREADABLE_REQUEST;
    };
    let Ok(request) = serde_json::from_slice::<BrokerRequest>(&bytes) else {
        return EXIT_UNPARSEABLE_REQUEST;
    };
    let Ok(out) = serde_json::to_vec(&execute_request(&request)) else {
        return EXIT_UNSERIALIZABLE_RESPONSE;
    };
    if write_response(resp_path, &out).is_err() {
        return EXIT_UNWRITABLE_RESPONSE;
    }
    0
}

/// Give a panicking child a distinct exit code.
///
/// The child runs before the logger is initialised, with no console and no inherited stderr, so the
/// exit code is its only channel. Release builds are `panic = "abort"`, under which a panic in
/// `execute_op` produces an abort the parent can only report as its catch-all, "crashed or was
/// terminated before writing a response" -- indistinguishable from an antivirus kill. The hook runs
/// before the abort, so exiting from it claims a code of our own. The message goes to stderr for
/// the documented manual-verification run from an elevated shell, where a console does exist.
fn install_panic_exit_hook() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("broker panicked: {info}");
        std::process::exit(EXIT_PANICKED);
    }));
}

/// Write the response, refusing to follow anything already at that path.
///
/// The parent only *reserves* the response name; the child creates it. `std::fs::write` is
/// `CREATE_ALWAYS`, which happily follows a symlink or junction planted at that name, so the write
/// would land wherever the reparse point pointed -- as TrustedInstaller. The 128 random bits in the
/// name are what make that hard to aim today, which is a reason not to rely on them alone.
/// `CREATE_NEW` makes anything already there a hard failure, and `FILE_FLAG_OPEN_REPARSE_POINT`
/// makes that true for a dangling symlink `CREATE_NEW` would otherwise follow.
fn write_response(resp_path: &str, out: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .share_mode(FILE_SHARE_READ)
        .open(resp_path)?;
    file.write_all(out)?;
    file.flush()
}

/// Whether a non-zero broker exit means nothing ran, or means we cannot know.
///
/// Exit 2 and 3 are returned before `execute_request` is ever called, so the machine is provably
/// untouched. Everything else, including the catch-all for a terminated or antivirus-killed child,
/// happened at or after the point where operations begin, so the honest answer is that we do not
/// know how far it got. Exits 4 and 5 in particular mean the batch ran to completion and only the
/// response was lost, which is the furthest thing from "nothing happened".
fn classify_exit(code: i32, detail: Error) -> BrokerOpError {
    match code {
        EXIT_UNREADABLE_REQUEST | EXIT_UNPARSEABLE_REQUEST => {
            BrokerOpError::CouldNotAcquire(detail)
        }
        _ => BrokerOpError::Indeterminate(detail),
    }
}

fn classify_spawn(e: SpawnError) -> BrokerOpError {
    match e {
        SpawnError::NoChild(e) => BrokerOpError::CouldNotAcquire(e),
        SpawnError::ChildRan(e) => BrokerOpError::Indeterminate(e),
    }
}

/// Monotonic counter mixed into the per-invocation transport nonce.
static BROKER_SEQ: AtomicU64 = AtomicU64::new(0);

/// A per-invocation transport nonce. Mixes wall-clock, a process-local counter, and the pid so two
/// invocations get distinct nonces even across a process restart that reuses our pid and resets the
/// counter: the exact conjunction that could otherwise let a stale response file be read as a
/// fresh success.
fn next_nonce() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seq = BROKER_SEQ.fetch_add(1, Ordering::SeqCst);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos.rotate_left(17)
        ^ seq.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ ((std::process::id() as u64) << 32)
}

/// Parse a broker response, rejecting it unless its nonce matches the one we sent. This is what
/// turns a stale or foreign response file into a hard error instead of a silent success.
fn validate_response(resp_bytes: &[u8], expected_nonce: u64) -> Result<BrokerResponse, Error> {
    let resp: BrokerResponse = serde_json::from_slice(resp_bytes)
        .map_err(|e| Error::ServiceControl(format!("parse broker response: {}", e)))?;
    if resp.nonce != expected_nonce {
        return Err(Error::ServiceControl(format!(
            "broker response nonce mismatch (sent {:#018x}, got {:#018x}): stale or foreign response",
            expected_nonce, resp.nonce
        )));
    }
    Ok(resp)
}

/// Run a batch of typed operations at the given elevation.
///
/// `Elevation::None` runs them in-process. `System`/`TrustedInstaller` write the request to a temp
/// file, spawn `<this exe> --broker <req> <resp>` under the corresponding token, and read the typed
/// response back. No shell parses anything, and the request *data* never reaches a command line;
/// only our own generated paths do.
///
/// ## The request file is the thing an attacker would want
///
/// The child reads it as SYSTEM or TrustedInstaller, so whoever controls its bytes controls what
/// runs at that level. `%TEMP%` is writable by every process running as this user, and the spawn
/// window is long (acquiring the TI token alone can take seconds), so "write it and hope" is not a
/// defence. It goes through [`ExclusiveTempFile`], which keeps a `FILE_SHARE_READ`-only write
/// handle open across the whole spawn: the child can read it, nothing else can rewrite it.
///
/// ## The response file is guarded, but not equally
///
/// The child creates it, so the parent cannot hold it open across the spawn the way it holds the
/// request. What does guard it: the parent only ever reserves an unguessable name, the child
/// creates it with `CREATE_NEW` plus `FILE_FLAG_OPEN_REPARSE_POINT` (so a pre-planted file,
/// junction or symlink is a hard failure rather than an arbitrary write as TrustedInstaller), the
/// child's exit code gates reading it at all, and the nonce must match the one sent.
///
/// What remains open, stated precisely: a same-user process that learns the path can overwrite the
/// response in the window between the child's exit and the parent's read, and forge a success. The
/// nonce does not stop that and was never meant to; it is a staleness guard. It is not even a
/// secret from such an attacker, since the request file is deliberately `FILE_SHARE_READ` so the
/// child can read it, and it carries both the nonce and the op count.
///
/// That is bounded, not unbounded. Every driven effect is verified by an in-process read-back the
/// forger cannot touch (`engine::apply`), so a forged success degrades into a verify mismatch and a
/// rollback, never silently-wrong machine state. The cost is a false failure, not a false success.
///
/// An inherited pipe would close it, and cannot be built here: the TrustedInstaller spawn sets
/// `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS`, under which handle inheritance is sourced from the
/// attribute parent rather than from us. A named pipe with a random name and a restrictive DACL is
/// the shape that would work, and it is a larger change than this comment once implied.
pub fn run_elevated_broker(
    level: Elevation,
    request: &BrokerRequest,
) -> Result<BrokerResponse, BrokerOpError> {
    if !level.is_elevated() {
        return Ok(execute_request(request));
    }

    let exe = std::env::current_exe().map_err(|e| {
        BrokerOpError::CouldNotAcquire(Error::ServiceControl(format!("current_exe failed: {e}")))
    })?;

    let nonce = next_nonce();
    let wire = BrokerRequest {
        nonce,
        ops: request.ops.clone(),
    };
    let req_json = serde_json::to_vec(&wire).map_err(|e| {
        BrokerOpError::CouldNotAcquire(Error::ServiceControl(format!(
            "serialize broker request: {e}"
        )))
    })?;

    let req_file =
        ExclusiveTempFile::create("magicx-broker", "req.json", &req_json).map_err(|e| {
            BrokerOpError::CouldNotAcquire(Error::ServiceControl(format!(
                "write broker request: {e}"
            )))
        })?;
    let resp_path =
        exclusive_temp::unique_temp_path("magicx-broker", "resp.json").map_err(|e| {
            BrokerOpError::CouldNotAcquire(Error::ServiceControl(format!(
                "reserve broker response path: {e}"
            )))
        })?;

    // Spawn "<exe>" --broker "<req>" "<resp>" directly (no cmd.exe wrapper). Paths are quoted; the
    // values are our own generated temp names, never untrusted data.
    let cmdline = format!(
        "\"{}\" --broker \"{}\" \"{}\"",
        exe.display(),
        req_file.path().display(),
        resp_path.display()
    );

    let spawn = match level {
        Elevation::TrustedInstaller => super::ti_elevation::spawn_as_trusted_installer(&cmdline),
        Elevation::None => unreachable!("handled above"),
    };

    // A non-zero exit means the response is stale or partial. How far the child got decides whether
    // the machine is untouched, so every failure is classified, never collapsed.
    let read = spawn.map_err(classify_spawn).and_then(|exit| {
        if exit != 0 {
            return Err(classify_exit(
                exit,
                Error::ServiceControl(format!(
                    "broker process exited with code {} ({}) without completing",
                    exit,
                    describe_broker_exit(exit)
                )),
            ));
        }
        // Exit 0 means the batch ran AND the response was written, so a read failure here is
        // about the response, not about whether anything happened.
        std::fs::read(&resp_path).map_err(|e| {
            BrokerOpError::Indeterminate(Error::ServiceControl(format!(
                "broker completed but its response could not be read: {e}"
            )))
        })
    });

    drop(req_file); // releases the share-mode lock and deletes the request
    let _ = std::fs::remove_file(&resp_path);

    // A response that fails validation came from a child that ran: the ops happened, the answer is
    // untrustworthy.
    validate_response(&read?, nonce).map_err(BrokerOpError::Indeterminate)
}

/// How a batch fails (spec §9, ADR-0005 as amended; invariant 24). Each aborts and rolls back at
/// the call site; none is ever downgraded to another or to a benign value.
#[derive(Debug, thiserror::Error)]
pub enum BrokerOpError {
    /// Nothing ran: the TrustedInstaller service would not start, `SeDebugPrivilege` was denied, no
    /// child was created, or it could not read its request. The machine is unchanged.
    #[error("could not acquire the elevated child: {0}")]
    CouldNotAcquire(#[source] Error),
    /// The child ran and an operation inside it failed.
    ///
    /// `index` is the failing op's position in the slice handed to [`run_ops`], carried
    /// structurally rather than only in the message so a caller that submitted a batch on behalf
    /// of several effects can name which one failed.
    #[error("operation failed inside the elevated child: {source}")]
    OpFailed {
        index: Option<usize>,
        #[source]
        source: Error,
    },
    /// A child was created, so ops may have run: a timeout, a failed wait or exit-code query, a
    /// panic, or a lost or invalid response. As `CouldNotAcquire`, a verified rollback would delete
    /// the snapshot (ADR-0002); as `OpFailed`, it would blame an op that may have succeeded.
    #[error("the elevated child ran but its outcome is unknown: {0}")]
    Indeterminate(#[source] Error),
}

impl BrokerOpError {
    /// The failing op's position, when the child named one.
    pub fn failed_op_index(&self) -> Option<usize> {
        match self {
            BrokerOpError::OpFailed { index, .. } => *index,
            BrokerOpError::CouldNotAcquire(_) | BrokerOpError::Indeterminate(_) => None,
        }
    }
}

/// Runs a whole batch of operations in ONE elevated child (spec §9's grouped execution). The only
/// entry point into the broker.
pub fn run_ops(level: Elevation, ops: Vec<BrokerOp>) -> Result<(), BrokerOpError> {
    let sent = ops.len();
    let response = run_elevated_broker(level, &BrokerRequest { nonce: 0, ops })?;
    check_response(sent, &response)
}

/// The did-it-work decision, isolated from the spawn so it is testable against a response the
/// executor would never produce. Success requires BOTH no reported failure AND a full attempt
/// count: a response that ran fewer ops than were sent, yet names no failure, is a truncated or
/// forged one, and reporting it as `Ok(())` would leave a half-applied batch looking complete.
fn check_response(sent: usize, response: &BrokerResponse) -> Result<(), BrokerOpError> {
    if let Some(failure) = &response.failure {
        return Err(BrokerOpError::OpFailed {
            index: Some(failure.index),
            source: Error::ServiceControl(format!(
                "broker op {} failed: {}",
                failure.index, failure.message
            )),
        });
    }
    // Fewer ops attempted than sent, yet no failure named: the executor cannot produce that, so
    // the response is truncated or forged. Some prefix of the batch may well have run, which makes
    // this indeterminate rather than a named operation failure.
    if response.attempted != sent {
        return Err(BrokerOpError::Indeterminate(Error::ServiceControl(
            format!(
                "broker attempted {} of {sent} ops but reported no failure",
                response.attempted
            ),
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SCRATCH_COUNTER: AtomicU32 = AtomicU32::new(0);

    /// A unique HKCU scratch key that deletes itself on drop, for parallel-safe isolation.
    struct Scratch {
        key: String,
    }
    impl Scratch {
        fn new() -> Self {
            let n = SCRATCH_COUNTER.fetch_add(1, Ordering::SeqCst);
            let key = format!(
                "Software\\MagicXToolboxTest\\broker_{}_{}",
                std::process::id(),
                n
            );
            Scratch { key }
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = registry_service::delete_key(&RegistryHive::Hkcu, &self.key);
        }
    }

    #[test]
    fn request_round_trips_through_json() {
        let req = BrokerRequest {
            nonce: 0xDEAD_BEEF,
            ops: vec![
                BrokerOp::RegSet {
                    hive: RegistryHive::Hklm,
                    key: "Software\\X".into(),
                    value_name: "V".into(),
                    value_type: RegistryValueType::Dword,
                    value: serde_json::json!(1),
                },
                BrokerOp::SvcSetStartup {
                    name: "Spooler".into(),
                    startup: ServiceStartupType::Manual,
                },
            ],
        };
        let json = serde_json::to_vec(&req).unwrap();
        let back: BrokerRequest = serde_json::from_slice(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn executor_sets_and_deletes_a_registry_value() {
        let scratch = Scratch::new();

        let set = BrokerOp::RegSet {
            hive: RegistryHive::Hkcu,
            key: scratch.key.clone(),
            value_name: "Flag".into(),
            value_type: RegistryValueType::Dword,
            value: serde_json::json!(7),
        };
        assert!(execute_op(&set).is_ok());
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "Flag").unwrap(),
            Some(7)
        );

        let del = BrokerOp::RegDeleteValue {
            hive: RegistryHive::Hkcu,
            key: scratch.key.clone(),
            value_name: "Flag".into(),
        };
        assert!(execute_op(&del).is_ok());
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "Flag").unwrap(),
            None
        );
    }

    #[test]
    fn deleting_an_absent_value_is_success() {
        let scratch = Scratch::new();
        // Key present, value absent: the common "already gone" case the apply flow hits.
        assert!(execute_op(&BrokerOp::RegCreateKey {
            hive: RegistryHive::Hkcu,
            key: scratch.key.clone(),
        })
        .is_ok());
        let del = BrokerOp::RegDeleteValue {
            hive: RegistryHive::Hkcu,
            key: scratch.key.clone(),
            value_name: "NeverExisted".into(),
        };
        assert!(execute_op(&del).is_ok());
    }

    /// A response is only a success when it names no failure AND accounts for every op sent.
    /// A short-but-clean response (what a truncated or forged file looks like) must not pass.
    #[test]
    fn check_response_requires_a_full_attempt_count_not_just_an_absent_failure() {
        let clean = |attempted| BrokerResponse {
            nonce: 1,
            attempted,
            failure: None,
        };
        check_response(3, &clean(3)).expect("all three attempted, none failed");

        // A short-but-clean response is Indeterminate, not OpFailed. The executor cannot produce
        // it, so the response is truncated or forged -- and a prefix of the batch may well have
        // run, which is exactly the state that must not be reported as a named operation failure
        // (which would blame an op that may have succeeded) nor as "nothing happened" (which would
        // let the caller consume the snapshot).
        let err = check_response(3, &clean(2))
            .expect_err("two of three attempted with no failure is not a completed batch");
        assert!(
            matches!(err, BrokerOpError::Indeterminate(_)),
            "got {err:?}"
        );
        assert!(err.to_string().contains("2 of 3"), "got {err}");

        let err = check_response(3, &clean(0))
            .expect_err("a response claiming nothing ran must never be Ok");
        assert!(
            matches!(err, BrokerOpError::Indeterminate(_)),
            "got {err:?}"
        );
    }

    #[test]
    fn check_response_surfaces_the_failing_op_index() {
        let response = BrokerResponse {
            nonce: 1,
            attempted: 2,
            failure: Some(OpFailure {
                index: 1,
                message: "denied".into(),
            }),
        };
        let err = check_response(3, &response).expect_err("a named failure must fail the batch");
        assert!(err.to_string().contains("op 1"), "got {err}");
        assert!(err.to_string().contains("denied"), "got {err}");
    }

    #[test]
    fn execute_request_reports_per_op_outcomes() {
        let scratch = Scratch::new();
        let req = BrokerRequest {
            nonce: 0,
            ops: vec![
                BrokerOp::RegCreateKey {
                    hive: RegistryHive::Hkcu,
                    key: scratch.key.clone(),
                },
                BrokerOp::RegSet {
                    hive: RegistryHive::Hkcu,
                    key: scratch.key.clone(),
                    value_name: "N".into(),
                    value_type: RegistryValueType::Dword,
                    value: serde_json::json!(3),
                },
            ],
        };
        let resp = execute_request(&req);
        assert_eq!((resp.attempted, resp.failure), (2, None));
    }

    #[test]
    fn run_broker_reads_request_and_writes_response() {
        // The file-transport contract: read a request file, execute, write a response file.
        let scratch = Scratch::new();
        let dir = std::env::temp_dir();
        let seq = SCRATCH_COUNTER.fetch_add(1, Ordering::SeqCst);
        let req_path = dir.join(format!(
            "magicx-brokertest-{}-{}-req.json",
            std::process::id(),
            seq
        ));
        let resp_path = dir.join(format!(
            "magicx-brokertest-{}-{}-resp.json",
            std::process::id(),
            seq
        ));

        let req = BrokerRequest {
            nonce: 0,
            ops: vec![
                BrokerOp::RegCreateKey {
                    hive: RegistryHive::Hkcu,
                    key: scratch.key.clone(),
                },
                BrokerOp::RegSet {
                    hive: RegistryHive::Hkcu,
                    key: scratch.key.clone(),
                    value_name: "Flag".into(),
                    value_type: RegistryValueType::Dword,
                    value: serde_json::json!(9),
                },
            ],
        };
        std::fs::write(&req_path, serde_json::to_vec(&req).unwrap()).unwrap();

        let code = run_broker(req_path.to_str().unwrap(), resp_path.to_str().unwrap());
        assert_eq!(code, 0);

        let resp: BrokerResponse =
            serde_json::from_slice(&std::fs::read(&resp_path).unwrap()).unwrap();
        assert_eq!((resp.attempted, resp.failure), (2, None));
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "Flag").unwrap(),
            Some(9)
        );

        let _ = std::fs::remove_file(&req_path);
        let _ = std::fs::remove_file(&resp_path);
    }

    #[test]
    fn run_elevated_broker_none_runs_in_process() {
        // Elevation::None takes the in-process path (no spawn), exercising the dispatch wrapper.
        let scratch = Scratch::new();
        let req = BrokerRequest {
            nonce: 0,
            ops: vec![BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: scratch.key.clone(),
                value_name: "N".into(),
                value_type: RegistryValueType::Dword,
                value: serde_json::json!(5),
            }],
        };
        let resp = run_elevated_broker(Elevation::None, &req).unwrap();
        assert_eq!((resp.attempted, resp.failure), (1, None));
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "N").unwrap(),
            Some(5)
        );
    }

    #[test]
    fn execute_request_echoes_the_request_nonce() {
        let resp = execute_request(&BrokerRequest {
            nonce: 0xABCD_1234,
            ops: vec![],
        });
        assert_eq!(resp.nonce, 0xABCD_1234);
        assert_eq!((resp.attempted, resp.failure), (0, None));
    }

    #[test]
    fn a_response_with_a_mismatched_nonce_is_rejected() {
        // A stale/foreign response file carries a different nonce than the one we sent: the guard
        // that stops a leftover file from being read as this invocation's success.
        let stale = serde_json::to_vec(&clean_response(111, 1)).unwrap();
        let err = validate_response(&stale, 222).expect_err("mismatched nonce must be rejected");
        assert!(matches!(err, Error::ServiceControl(_)), "got {err:?}");
    }

    #[test]
    fn a_response_with_the_expected_nonce_is_accepted() {
        let good = serde_json::to_vec(&clean_response(222, 1)).unwrap();
        let resp = validate_response(&good, 222).expect("matching nonce must validate");
        assert_eq!((resp.attempted, resp.failure), (1, None));
    }

    fn clean_response(nonce: u64, attempted: usize) -> BrokerResponse {
        BrokerResponse {
            nonce,
            attempted,
            failure: None,
        }
    }

    #[test]
    fn next_nonce_values_are_distinct() {
        assert_ne!(next_nonce(), next_nonce());
    }

    #[test]
    fn run_ops_executes_a_batch_in_one_call_unelevated() {
        // Elevation::None never spawns a child (run_elevated_broker's own early return), so this
        // exercises run_ops's batching/checking logic with zero elevation.
        let scratch = Scratch::new();
        let ops = vec![
            BrokerOp::RegCreateKey {
                hive: RegistryHive::Hkcu,
                key: scratch.key.clone(),
            },
            BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: scratch.key.clone(),
                value_name: "N".into(),
                value_type: RegistryValueType::Dword,
                value: serde_json::json!(11),
            },
        ];
        run_ops(Elevation::None, ops).expect("unelevated run_ops must succeed in-process");
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "N").unwrap(),
            Some(11)
        );
    }

    #[test]
    fn run_ops_reports_an_op_failure_as_opfailed_never_a_benign_ok() {
        let scratch = Scratch::new();
        let bad = vec![BrokerOp::RegSet {
            hive: RegistryHive::Hkcu,
            key: scratch.key.clone(),
            value_name: "N".into(),
            value_type: RegistryValueType::Dword,
            value: serde_json::json!("not-a-number"),
        }];
        let err = run_ops(Elevation::None, bad)
            .expect_err("a malformed DWORD value must fail the op, never silently succeed");
        assert!(
            matches!(err, BrokerOpError::OpFailed { .. }),
            "an in-process op failure is OpFailed, never CouldNotAcquire; got {err:?}"
        );
    }

    /// CRITICAL fix: a batch must STOP at the first failing op, never run a later one anyway. Two
    /// `RegSet`s against the same scratch key: the first with an unparseable value (fails), the
    /// second with a perfectly good one. Asserted via the EXECUTED STATE (a real read-back of the
    /// second value name), not just the response shape: if this regressed, `"Second"` would exist.
    #[test]
    fn execute_request_stops_at_the_first_failing_op_never_running_the_rest() {
        let scratch = Scratch::new();
        let req = BrokerRequest {
            nonce: 0,
            ops: vec![
                BrokerOp::RegSet {
                    hive: RegistryHive::Hkcu,
                    key: scratch.key.clone(),
                    value_name: "Bad".into(),
                    value_type: RegistryValueType::Dword,
                    value: serde_json::json!("not-a-number"), // fails to parse
                },
                BrokerOp::RegSet {
                    hive: RegistryHive::Hkcu,
                    key: scratch.key.clone(),
                    value_name: "Second".into(),
                    value_type: RegistryValueType::Dword,
                    value: serde_json::json!(99),
                },
            ],
        };

        let resp = execute_request(&req);
        assert_eq!(
            resp.attempted, 1,
            "must stop after the first failing op; the second is never attempted"
        );
        assert_eq!(resp.failure.map(|f| f.index), Some(0));
        assert_second_value_was_never_written(&scratch.key);
    }

    /// The `run_ops` half of the same fix: the whole batch fails, naming the first op, and (via
    /// the executed state, mirroring the test above) the second op never ran.
    #[test]
    fn run_ops_names_the_first_failing_op_and_never_runs_the_rest() {
        let scratch = Scratch::new();
        let ops = vec![
            BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: scratch.key.clone(),
                value_name: "Bad".into(),
                value_type: RegistryValueType::Dword,
                value: serde_json::json!("not-a-number"),
            },
            BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: scratch.key.clone(),
                value_name: "Second".into(),
                value_type: RegistryValueType::Dword,
                value: serde_json::json!(99),
            },
        ];

        let err = run_ops(Elevation::None, ops)
            .expect_err("the first op's failure must fail the whole batch");
        assert!(matches!(err, BrokerOpError::OpFailed { .. }), "got {err:?}");
        assert!(
            err.to_string().contains("op 0"),
            "must name the failing op's index; got {err}"
        );
        assert_second_value_was_never_written(&scratch.key);
    }

    /// "Second" absent (never written) is true whether the SCRATCH KEY itself was never created
    /// (`RegistryKeyNotFound` -- the first op failed before any write at all) or the key exists but
    /// the value doesn't (`Ok(None)`) -- both mean "the second op never ran."
    fn assert_second_value_was_never_written(key: &str) {
        match registry_service::read_dword(&RegistryHive::Hkcu, key, "Second") {
            Ok(None) => {}
            Err(Error::RegistryKeyNotFound(_)) => {}
            other => panic!(
                "the second op's effect must never be applied once the first op failed, got {other:?}"
            ),
        }
    }

    // There is deliberately no elevated end-to-end test here, and there cannot be one:
    // `run_ops(Elevation::System, ..)` respawns `current_exe()` with `--broker`, but under `cargo
    // test` that is the libtest harness binary, which rejects the flag and exits non-zero. The
    // respawn design makes this structural, not a gap in these tests.
    //
    // Verify the real path against the built `magicx-toolbox.exe` instead: serialize a
    // `BrokerRequest` to a file, run `magicx-toolbox.exe --broker <req> <resp>` from an elevated
    // shell, and check both the response and the machine state it claims to have produced.

    /// The child creates the response at a path the parent only reserved. `std::fs::write` was
    /// `CREATE_ALWAYS`, which follows a reparse point planted at that name and would land the write
    /// wherever it pointed, as TrustedInstaller. `CREATE_NEW` has to refuse anything already there.
    #[test]
    fn the_response_write_refuses_a_pre_planted_path() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "magicx-broker-test-preplant-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);

        std::fs::write(&path, b"planted by someone else").expect("plant a file");
        let planted = write_response(path.to_str().unwrap(), b"{}");
        assert!(
            planted.is_err(),
            "an existing file at the response path must be a hard failure"
        );
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"planted by someone else",
            "the planted file must be left untouched, never truncated"
        );

        std::fs::remove_file(&path).expect("clear the path");
        write_response(path.to_str().unwrap(), b"{}").expect("a clean path must still work");
        assert_eq!(std::fs::read(&path).unwrap(), b"{}");
        let _ = std::fs::remove_file(&path);
    }

    /// A non-zero broker exit has to say whether the machine could have changed, because the
    /// caller's next decision is whether the snapshot still describes anything (ADR-0002).
    ///
    /// Exits 2 and 3 happen before `execute_request` is ever called, so nothing ran. Exits 4 and 5
    /// happen only after it returns, meaning the whole batch ran and only the response was lost --
    /// the furthest thing from "nothing happened". A panic or an outright kill lands in between and
    /// is unknowable, which is the honest answer rather than a convenient one.
    #[test]
    fn a_broker_exit_says_whether_anything_could_have_run() {
        let detail = || Error::ServiceControl("detail".into());

        for code in [EXIT_UNREADABLE_REQUEST, EXIT_UNPARSEABLE_REQUEST] {
            assert!(
                matches!(
                    classify_exit(code, detail()),
                    BrokerOpError::CouldNotAcquire(_)
                ),
                "exit {code} happens before any op runs"
            );
        }

        for code in [
            EXIT_UNSERIALIZABLE_RESPONSE,
            EXIT_UNWRITABLE_RESPONSE,
            EXIT_PANICKED,
            // The catch-all: killed mid-batch, e.g. by a security product.
            42,
        ] {
            assert!(
                matches!(
                    classify_exit(code, detail()),
                    BrokerOpError::Indeterminate(_)
                ),
                "exit {code} happens at or after the point where ops begin"
            );
        }
    }

    /// Only a spawn that created no child ran nothing. Every failure after `CreateProcessW` keeps
    /// the snapshot (ADR-0002), and its error says whether the child may still be running.
    #[test]
    fn only_a_spawn_that_created_no_child_is_could_not_acquire() {
        use super::super::common::wait_and_reap;
        use std::os::windows::io::AsRawHandle;
        use std::process::{Child, Command, Stdio};
        use windows_sys::Win32::Foundation::{FALSE, HANDLE, WAIT_OBJECT_0};
        use windows_sys::Win32::System::Threading::{
            OpenProcess, WaitForSingleObject, PROCESS_INFORMATION,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
        };

        /// Kills the child on drop, so a failing row cannot leak it.
        struct Spawned(Child);
        impl Drop for Spawned {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        fn spawn(exe: &str, args: &[&str]) -> Spawned {
            let root = std::env::var_os("SystemRoot").expect("SystemRoot is set");
            let path = std::path::Path::new(&root).join("System32").join(exe);
            Spawned(
                Command::new(path)
                    .args(args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("spawn a test child"),
            )
        }
        fn handles(child: &Spawned, rights: u32) -> PROCESS_INFORMATION {
            let pid = child.0.id();
            // SAFETY: OpenProcess only reads its arguments; the handles are checked below.
            let pi = unsafe {
                PROCESS_INFORMATION {
                    hProcess: OpenProcess(rights, FALSE, pid),
                    // wait_and_reap only closes hThread; any owned handle stands in.
                    hThread: OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid),
                    dwProcessId: pid,
                    dwThreadId: 0,
                }
            };
            assert!(!pi.hProcess.is_null() && !pi.hThread.is_null());
            pi
        }

        let no_child = classify_spawn(SpawnError::NoChild(Error::ServiceControl(
            "CreateProcessW failed".into(),
        )));
        assert!(
            matches!(no_child, BrokerOpError::CouldNotAcquire(_)),
            "{no_child:?}"
        );

        let exits = spawn("cmd.exe", &["/c", "exit", "7"]);
        let full = PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE;
        // SAFETY: both handles are owned by `handles` and closed by wait_and_reap.
        let code = unsafe { wait_and_reap(&handles(&exits, full), "test child", 30_000) };
        assert!(
            matches!(code, Ok(7)),
            "a child that exits reports its code: {code:?}"
        );

        struct PostSpawn {
            kind: &'static str,
            hangs: bool,
            rights: u32,
            timeout_ms: u32,
            says: &'static str,
            dies: bool,
        }
        let rows = [
            PostSpawn {
                kind: "confirmed kill",
                hangs: true,
                rights: PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
                timeout_ms: 200,
                says: "timed out after 200ms and was terminated",
                dies: true,
            },
            // No PROCESS_TERMINATE: TerminateProcess fails.
            PostSpawn {
                kind: "unconfirmed kill",
                hangs: true,
                rights: PROCESS_SYNCHRONIZE,
                timeout_ms: 200,
                says: "timed out after 200ms and could not be confirmed dead (TerminateProcess failed: 5); it may still be modifying the system",
                dies: false,
            },
            // No SYNCHRONIZE: both the wait and the kill's confirming wait fail.
            PostSpawn {
                kind: "failed wait",
                hangs: true,
                rights: PROCESS_TERMINATE,
                timeout_ms: 30_000,
                says: "wait failed (result 0xffffffff): 5 and could not be confirmed dead",
                dies: true,
            },
            // No query right: the wait succeeds, GetExitCodeProcess fails.
            PostSpawn {
                kind: "failed exit-code query",
                hangs: false,
                rights: PROCESS_SYNCHRONIZE,
                timeout_ms: 30_000,
                says: "exit-code query failed",
                dies: true,
            },
        ];
        for row in rows {
            let child = if row.hangs {
                spawn("PING.EXE", &["-n", "30", "127.0.0.1"])
            } else {
                spawn("cmd.exe", &["/c", "exit", "0"])
            };
            // SAFETY: both handles are owned by `handles` and closed by wait_and_reap.
            let err = unsafe {
                wait_and_reap(&handles(&child, row.rights), "test child", row.timeout_ms)
            }
            .expect_err(row.kind);
            let grace_ms = if row.dies { 5_000 } else { 0 };
            // SAFETY: `child` owns this handle and outlives the wait.
            let dead = unsafe { WaitForSingleObject(child.0.as_raw_handle() as HANDLE, grace_ms) }
                == WAIT_OBJECT_0;

            let classified = classify_spawn(err);
            assert!(
                matches!(&classified, BrokerOpError::Indeterminate(e) if e.to_string().contains(row.says)),
                "{}: {classified:?}",
                row.kind
            );
            assert_eq!(
                dead, row.dies,
                "{}: whether the child outlives the error",
                row.kind
            );
        }
    }
}
