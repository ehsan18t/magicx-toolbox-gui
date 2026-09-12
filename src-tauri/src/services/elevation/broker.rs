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
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
#[serde(deny_unknown_fields)]
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

/// Bump on any change to the wire types or exit codes (`the_wire_format_is_pinned`): after an
/// update, parent and child are separate builds.
const WIRE_VERSION: u32 = 1;

/// A batch of operations for one broker invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BrokerRequest {
    pub version: u32,
    /// Transport nonce. Assigned freshly by [`run_elevated_broker`] at send time and echoed back in
    /// the response, so a stale or foreign response file (e.g. a leftover from a prior run at a
    /// reused pid) is detected rather than read as a fresh success.
    #[serde(default)]
    pub nonce: u64,
    pub ops: Vec<BrokerOp>,
}

impl BrokerRequest {
    pub fn new(ops: Vec<BrokerOp>) -> Self {
        Self {
            version: WIRE_VERSION,
            nonce: 0,
            ops,
        }
    }
}

/// The op a batch died on, by position in the request's `ops`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OpFailure {
    pub index: usize,
    pub message: String,
}

/// The broker's typed response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BrokerResponse {
    pub version: u32,
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
        version: WIRE_VERSION,
        nonce: request.nonce,
        attempted,
        failure,
    }
}

enum WireReject {
    Malformed(String),
    Version(Option<u64>),
}

/// Object form only: a derived struct also accepts a JSON array, which has no field names or
/// version key. The version is read first, so another build reports as a mismatch, not malformed.
fn parse_wire<T: DeserializeOwned>(
    bytes: &[u8],
    version: impl Fn(&T) -> u32,
) -> Result<T, WireReject> {
    let malformed = |e: serde_json::Error| WireReject::Malformed(e.to_string());
    let object: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(bytes).map_err(malformed)?;
    let declared = object.get("version").and_then(serde_json::Value::as_u64);
    if declared != Some(WIRE_VERSION.into()) {
        return Err(WireReject::Version(declared));
    }
    let parsed: T = serde_json::from_value(object.into()).map_err(malformed)?;
    match version(&parsed) {
        WIRE_VERSION => Ok(parsed),
        other => Err(WireReject::Version(Some(other.into()))),
    }
}

const DIFFERENT_BUILD: &str =
    "the elevated helper is a different build of the app; restart the app and try again";

// Transport-failure exit codes, the child's only channel; `describe_broker_exit` names each.
// A nothing-ran code must be a whole 32-bit value no kill or crash picks: `TerminateProcess` takes
// any code and CRT `abort()` exits 3. Pinned with the wire format (`WIRE_VERSION`).
const EXIT_UNREADABLE_REQUEST: i32 = 0x204D_5801;
const EXIT_UNPARSEABLE_REQUEST: i32 = 0x204D_5802;
const EXIT_WIRE_VERSION_MISMATCH: i32 = 0x204D_5803;
const EXIT_UNSERIALIZABLE_RESPONSE: i32 = 4;
const EXIT_UNWRITABLE_RESPONSE: i32 = 5;
/// A panic inside the child. Distinct from the catch-all so a bug in our own executor is never
/// reported as an external kill: under `panic = "abort"` the two are otherwise indistinguishable,
/// and "crashed or was terminated" sends a support engineer looking at antivirus instead of at us.
const EXIT_PANICKED: i32 = 6;

fn describe_broker_exit(code: i32) -> &'static str {
    match code {
        EXIT_UNREADABLE_REQUEST => {
            "could not read its request (bad arguments or an unreadable file)"
        }
        EXIT_UNPARSEABLE_REQUEST => "request file was not a valid request",
        EXIT_WIRE_VERSION_MISMATCH => DIFFERENT_BUILD,
        EXIT_UNSERIALIZABLE_RESPONSE => "could not serialize the response",
        EXIT_UNWRITABLE_RESPONSE => "could not write the response file",
        EXIT_PANICKED => "panicked while executing the batch",
        _ => "crashed or was terminated before writing a response",
    }
}

pub fn malformed_argv_exit_code() -> i32 {
    EXIT_UNREADABLE_REQUEST
}

/// The real child's entrypoint. The panic hook is process-wide, so tests call [`serve_request`].
pub fn run_broker(req_path: &str, resp_path: &str) -> i32 {
    install_panic_exit_hook();
    serve_request(req_path, resp_path)
}

/// Read a request file, execute it, write a response file. 0 means the batch ran and a response
/// was written; non-zero is a transport failure, distinct from op failures inside the response.
fn serve_request(req_path: &str, resp_path: &str) -> i32 {
    let Ok(bytes) = std::fs::read(req_path) else {
        return EXIT_UNREADABLE_REQUEST;
    };
    let request = match parse_wire(&bytes, |r: &BrokerRequest| r.version) {
        Ok(request) => request,
        Err(WireReject::Version(_)) => return EXIT_WIRE_VERSION_MISMATCH,
        Err(WireReject::Malformed(_)) => return EXIT_UNPARSEABLE_REQUEST,
    };
    let Ok(out) = serde_json::to_vec(&execute_request(&request)) else {
        return EXIT_UNSERIALIZABLE_RESPONSE;
    };
    if write_response(resp_path, &out).is_err() {
        return EXIT_UNWRITABLE_RESPONSE;
    }
    0
}

/// Under release `panic = "abort"` a child panic otherwise reads as the catch-all exit, like an
/// antivirus kill; the hook runs first and claims `EXIT_PANICKED`. The exit code is the child's
/// only channel; stderr serves the manual run from an elevated shell.
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

/// Only the request-side exits precede `execute_request` and prove nothing ran; any other non-zero
/// code, a kill included, may have run ops.
fn classify_exit(code: i32, detail: Error) -> BrokerOpError {
    match code {
        EXIT_UNREADABLE_REQUEST | EXIT_UNPARSEABLE_REQUEST | EXIT_WIRE_VERSION_MISMATCH => {
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
/// turns a stale or foreign response file into a hard error instead of a silent success. Every
/// rejection is `Indeterminate`: only a child that ran writes a response.
fn validate_response(
    resp_bytes: &[u8],
    expected_nonce: u64,
) -> Result<BrokerResponse, BrokerOpError> {
    let rejected = |msg: String| BrokerOpError::Indeterminate(Error::ServiceControl(msg));
    let resp = parse_wire(resp_bytes, |r: &BrokerResponse| r.version).map_err(|e| {
        rejected(match e {
            WireReject::Malformed(e) => format!("parse broker response: {e}"),
            WireReject::Version(got) => format!(
                "broker response wire version is {}, expected {WIRE_VERSION}: {DIFFERENT_BUILD}",
                got.map_or_else(|| "missing".to_owned(), |v| v.to_string())
            ),
        })
    })?;
    if resp.nonce != expected_nonce {
        // Never the two values: this string crosses IPC to the UI and into the log.
        return Err(rejected(
            "broker response nonce mismatch: stale or foreign response".to_owned(),
        ));
    }
    Ok(resp)
}

/// Run a batch of typed operations at the given elevation.
///
/// `Elevation::None` runs them in-process. `TrustedInstaller` writes the request to a temp file,
/// spawns `<this exe> --broker <req> <resp>` under that token, and reads the typed response back. No shell parses anything, and the request *data* never reaches a command line;
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
fn run_elevated_broker(
    level: Elevation,
    ops: Vec<BrokerOp>,
    spawn: impl FnOnce(&str) -> Result<i32, SpawnError>,
) -> Result<BrokerResponse, BrokerOpError> {
    if !level.is_elevated() {
        return Ok(execute_request(&BrokerRequest::new(ops)));
    }

    let exe = std::env::current_exe().map_err(|e| {
        BrokerOpError::CouldNotAcquire(Error::ServiceControl(format!("current_exe failed: {e}")))
    })?;

    let nonce = next_nonce();
    let wire = BrokerRequest {
        nonce,
        ..BrokerRequest::new(ops)
    };
    let req_json = serde_json::to_vec(&wire).map_err(|e| {
        BrokerOpError::CouldNotAcquire(Error::ServiceControl(format!(
            "serialize broker request: {e}"
        )))
    })?;

    let req_file =
        ExclusiveTempFile::create("magicx-broker", "req.json", "broker request", &req_json)
            .map_err(|e| {
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
    let resp_guard = exclusive_temp::TempPathGuard::new(resp_path, "broker response");

    // Spawn "<exe>" --broker "<req>" "<resp>" directly (no cmd.exe wrapper). Paths are quoted; the
    // values are our own generated temp names, never untrusted data.
    let cmdline = format!(
        "\"{}\" --broker \"{}\" \"{}\"",
        exe.display(),
        req_file.path().display(),
        resp_guard.path().display()
    );

    let spawned = match level {
        Elevation::TrustedInstaller => spawn(&cmdline),
        Elevation::None => unreachable!("handled above"),
    };

    // A non-zero exit means the response is stale or partial. How far the child got decides whether
    // the machine is untouched, so every failure is classified, never collapsed.
    let read = spawned.map_err(classify_spawn).and_then(|exit| {
        if exit != 0 {
            // The only place that owns both the code and its meaning, and both are ours.
            let why = describe_broker_exit(exit);
            log::warn!("The broker child exited with {exit:#x}: {why}");
            return Err(classify_exit(
                exit,
                Error::ServiceControl(format!("broker process exited with code {exit:#x}: {why}")),
            ));
        }
        // Exit 0 means the batch ran AND the response was written, so a read failure here is
        // about the response, not about whether anything happened.
        std::fs::read(resp_guard.path()).map_err(|e| {
            BrokerOpError::Indeterminate(Error::ServiceControl(format!(
                "broker completed but its response could not be read: {e}"
            )))
        })
    });

    validate_response(&read?, nonce)
}

/// How a batch fails (spec §9, ADR-0005 as amended; invariant 24). Each aborts and rolls back at
/// the call site; none is ever downgraded to another or to a benign value.
#[derive(Debug, thiserror::Error)]
pub enum BrokerOpError {
    /// Nothing ran: the TrustedInstaller service would not start, `SeDebugPrivilege` was denied, no
    /// child was created, or it refused its request (unreadable, unparseable, or from a different
    /// build) before any op. The machine is unchanged.
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
    run_ops_with(level, ops, super::ti_elevation::spawn_as_trusted_installer)
}

/// `spawn` runs the child's command line: the TrustedInstaller launch, or a test's fake child.
fn run_ops_with(
    level: Elevation,
    ops: Vec<BrokerOp>,
    spawn: impl FnOnce(&str) -> Result<i32, SpawnError>,
) -> Result<(), BrokerOpError> {
    let sent = ops.len();
    let outcome =
        run_elevated_broker(level, ops, spawn).and_then(|response| check_response(sent, &response));
    if let Err(e) = &outcome {
        log_failure(level, e);
    }
    outcome
}

/// The parent's record of a failed elevated batch: the level and what the failure means for the
/// machine, built only from values the parent itself produced. An error's own text can carry a
/// path, a nonce, or the registry key an op wrote, so none of it crosses into a log line.
fn failure_summary(level: Elevation, e: &BrokerOpError) -> String {
    let what = match e {
        BrokerOpError::CouldNotAcquire(_) => {
            "could not acquire the child, so nothing ran".to_owned()
        }
        BrokerOpError::OpFailed {
            index: Some(index), ..
        } => format!("operation {index} was refused in the child"),
        BrokerOpError::OpFailed { index: None, .. } => {
            "an operation was refused in the child".to_owned()
        }
        BrokerOpError::Indeterminate(_) => {
            "the child ran but its outcome is unknown, so the machine may have changed".to_owned()
        }
    };
    format!("{level:?} broker batch failed: {what}")
}

/// `None` for an in-process batch, which is not the broker's to report: its caller sees the same
/// error directly.
fn failure_log_level(level: Elevation, e: &BrokerOpError) -> Option<log::Level> {
    if !level.is_elevated() {
        return None;
    }
    Some(match e {
        BrokerOpError::Indeterminate(_) => log::Level::Error,
        _ => log::Level::Warn,
    })
}

fn log_failure(level: Elevation, e: &BrokerOpError) {
    if let Some(at) = failure_log_level(level, e) {
        log::log!(at, "{}", failure_summary(level, e));
    }
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
            version: WIRE_VERSION,
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
            version: WIRE_VERSION,
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
            version: WIRE_VERSION,
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
            version: WIRE_VERSION,
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
    fn serve_request_reads_request_and_writes_response() {
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
            version: WIRE_VERSION,
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

        let code = serve_request(req_path.to_str().unwrap(), resp_path.to_str().unwrap());
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
        let ops = vec![BrokerOp::RegSet {
            hive: RegistryHive::Hkcu,
            key: scratch.key.clone(),
            value_name: "N".into(),
            value_type: RegistryValueType::Dword,
            value: serde_json::json!(5),
        }];
        let resp =
            run_elevated_broker(Elevation::None, ops, |_| panic!("None never spawns")).unwrap();
        assert_eq!((resp.attempted, resp.failure), (1, None));
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "N").unwrap(),
            Some(5)
        );
    }

    #[test]
    fn execute_request_echoes_the_request_nonce() {
        let resp = execute_request(&BrokerRequest {
            version: WIRE_VERSION,
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
        assert!(
            matches!(err, BrokerOpError::Indeterminate(Error::ServiceControl(_))),
            "got {err:?}"
        );
    }

    #[test]
    fn a_response_with_the_expected_nonce_is_accepted() {
        let good = serde_json::to_vec(&clean_response(222, 1)).unwrap();
        let resp = validate_response(&good, 222).expect("matching nonce must validate");
        assert_eq!((resp.attempted, resp.failure), (1, None));
    }

    fn clean_response(nonce: u64, attempted: usize) -> BrokerResponse {
        BrokerResponse {
            version: WIRE_VERSION,
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
            version: WIRE_VERSION,
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
        assert_value_never_written(&scratch.key, "Second");
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
        assert_value_never_written(&scratch.key, "Second");
    }

    /// An absent key and an absent value both mean the op never ran.
    fn assert_value_never_written(key: &str, name: &str) {
        match registry_service::read_dword(&RegistryHive::Hkcu, key, name) {
            Ok(None) => {}
            Err(Error::RegistryKeyNotFound(_)) => {}
            other => panic!("{name} must never be written, got {other:?}"),
        }
    }

    /// One `RegSet` of `Flag` under `key`, as the JSON a parent would write.
    fn request_json(key: &str) -> serde_json::Value {
        serde_json::to_value(BrokerRequest {
            version: WIRE_VERSION,
            nonce: 0,
            ops: vec![BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: key.into(),
                value_name: "Flag".into(),
                value_type: RegistryValueType::Dword,
                value: serde_json::json!(1),
            }],
        })
        .unwrap()
    }

    fn temp_path(kind: &str) -> std::path::PathBuf {
        let seq = SCRATCH_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        std::env::temp_dir().join(format!("magicx-brokertest-{pid}-{seq}-{kind}.json"))
    }

    /// `serve_request` on raw request bytes: its exit code and whether it wrote a response.
    fn run_broker_on(request: &[u8]) -> (i32, bool) {
        let (req_path, resp_path) = (temp_path("req"), temp_path("resp"));
        std::fs::write(&req_path, request).unwrap();
        let code = serve_request(req_path.to_str().unwrap(), resp_path.to_str().unwrap());
        let responded = resp_path.exists();
        let _ = std::fs::remove_file(&req_path);
        let _ = std::fs::remove_file(&resp_path);
        (code, responded)
    }

    fn to_bytes(json: &serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(json).unwrap()
    }

    /// The child refuses a request it cannot fully understand before running any op, with an exit
    /// the parent reads as nothing-ran.
    fn assert_refused_before_any_op(case: &str, request: &[u8], key: &str, exit: i32) {
        let (code, responded) = run_broker_on(request);
        assert_eq!(code, exit, "{case}");
        assert!(
            matches!(
                classify_exit(code, Error::ServiceControl("detail".into())),
                BrokerOpError::CouldNotAcquire(_)
            ),
            "{case}: exit {code} must classify as nothing-ran"
        );
        assert!(!responded, "{case}: no response may be written");
        assert_value_never_written(key, "Flag");
    }

    #[test]
    fn a_request_with_an_unknown_field_is_refused_before_any_op() {
        let scratch = Scratch::new();
        let mut top = request_json(&scratch.key);
        top["future"] = serde_json::json!(1);
        let unparseable = EXIT_UNPARSEABLE_REQUEST;
        assert_refused_before_any_op(
            "top-level field",
            &to_bytes(&top),
            &scratch.key,
            unparseable,
        );

        let mut in_op = request_json(&scratch.key);
        in_op["ops"][0]["RegSet"]["future"] = serde_json::json!(1);
        let in_op = to_bytes(&in_op);
        assert_refused_before_any_op("field inside an op", &in_op, &scratch.key, unparseable);
    }

    #[test]
    fn a_request_that_is_not_our_json_object_is_refused_before_any_op() {
        let scratch = Scratch::new();
        let full = to_bytes(&request_json(&scratch.key));
        let op = request_json(&scratch.key)["ops"][0].clone();
        for (case, bytes) in [
            ("not JSON", b"not json".to_vec()),
            ("truncated", full[..full.len() / 2].to_vec()),
            (
                "array, other version",
                to_bytes(&serde_json::json!([2, 0, [op.clone()]])),
            ),
            (
                "array, our version",
                to_bytes(&serde_json::json!([WIRE_VERSION, 0, [op]])),
            ),
        ] {
            assert_refused_before_any_op(case, &bytes, &scratch.key, EXIT_UNPARSEABLE_REQUEST);
        }
    }

    #[test]
    fn an_unreadable_request_file_is_refused_before_any_op() {
        let (missing, resp_path) = (temp_path("missing"), temp_path("resp"));
        let code = serve_request(missing.to_str().unwrap(), resp_path.to_str().unwrap());
        assert_eq!(code, EXIT_UNREADABLE_REQUEST);
        assert!(!resp_path.exists());
    }

    /// `run_broker`'s hook would exit the whole test run on the next failing assert, unnamed.
    #[test]
    fn a_test_side_broker_run_installs_no_panic_hook() {
        assert_eq!(run_broker_on(b"{}").0, EXIT_WIRE_VERSION_MISMATCH);
        assert!(std::panic::catch_unwind(|| panic!("probe")).is_err());
    }

    #[test]
    fn a_request_with_an_unknown_variant_is_refused_before_any_op() {
        let scratch = Scratch::new();
        let mut op = request_json(&scratch.key);
        let body = op["ops"][0]["RegSet"].take();
        op["ops"][0] = serde_json::json!({ "RegFuture": body });
        let unparseable = EXIT_UNPARSEABLE_REQUEST;
        assert_refused_before_any_op("op variant", &to_bytes(&op), &scratch.key, unparseable);

        let mut hive = request_json(&scratch.key);
        hive["ops"][0]["RegSet"]["hive"] = serde_json::json!("HKFUTURE");
        assert_refused_before_any_op("hive variant", &to_bytes(&hive), &scratch.key, unparseable);
    }

    #[test]
    fn a_request_from_another_wire_version_is_refused_before_any_op() {
        let scratch = Scratch::new();
        let mut newer = request_json(&scratch.key);
        newer["version"] = serde_json::json!(u32::MAX);
        let mismatch = EXIT_WIRE_VERSION_MISMATCH;
        assert_refused_before_any_op("newer version", &to_bytes(&newer), &scratch.key, mismatch);

        let mut unversioned = request_json(&scratch.key);
        unversioned.as_object_mut().unwrap().remove("version");
        let unversioned = to_bytes(&unversioned);
        assert_refused_before_any_op("no version", &unversioned, &scratch.key, mismatch);
    }

    #[test]
    fn a_response_that_is_not_exactly_ours_is_outcome_unknown() {
        let good = || serde_json::to_value(clean_response(222, 1)).unwrap();
        let full = to_bytes(&good());
        let mut cases = vec![
            ("not JSON", b"not json".to_vec()),
            ("truncated", full[..full.len() / 2].to_vec()),
            (
                "array, other version",
                to_bytes(&serde_json::json!([2, 222, 1, null])),
            ),
            (
                "array, our version",
                to_bytes(&serde_json::json!([WIRE_VERSION, 222, 1, null])),
            ),
        ];

        let mut top = good();
        top["future"] = serde_json::json!(1);
        cases.push(("top-level field", to_bytes(&top)));

        let mut in_failure = good();
        in_failure["failure"] = serde_json::json!({ "index": 0, "message": "x", "future": 1 });
        cases.push(("field inside the failure", to_bytes(&in_failure)));

        let mut newer = good();
        newer["version"] = serde_json::json!(u32::MAX);
        cases.push(("newer version", to_bytes(&newer)));

        let mut unversioned = good();
        unversioned.as_object_mut().unwrap().remove("version");
        cases.push(("no version", to_bytes(&unversioned)));

        for (case, bytes) in cases {
            let got = validate_response(&bytes, 222);
            assert!(
                matches!(got, Err(BrokerOpError::Indeterminate(_))),
                "{case}: {got:?}"
            );
        }
    }

    #[test]
    fn every_nothing_ran_exit_carries_the_full_distinctive_prefix() {
        for code in [
            EXIT_UNREADABLE_REQUEST,
            EXIT_UNPARSEABLE_REQUEST,
            EXIT_WIRE_VERSION_MISMATCH,
            malformed_argv_exit_code(),
        ] {
            assert_eq!(code as u32 >> 16, 0x204D, "{code:#x}");
        }
    }

    #[test]
    fn the_wire_format_is_pinned() {
        // Changing any byte below, or any exit code, needs a WIRE_VERSION bump.
        const REQUEST: &str = r#"{"version":1,"nonce":7,"ops":[{"RegSet":{"hive":"HKLM","key":"K","value_name":"V","value_type":"REG_DWORD","value":1}},{"RegDeleteValue":{"hive":"HKCU","key":"K","value_name":"V"}},{"RegDeleteKey":{"hive":"HKCU","key":"K"}},{"RegCreateKey":{"hive":"HKCU","key":"K"}},{"SvcSetStartup":{"name":"S","startup":"manual"}},{"Scheduler":{"task_path":"P","task_name":"T","action":"disable"}}]}"#;
        const RESPONSE: &str =
            r#"{"version":1,"nonce":7,"attempted":2,"failure":{"index":1,"message":"denied"}}"#;
        assert_eq!(WIRE_VERSION, 1);
        assert_eq!(
            [
                EXIT_UNREADABLE_REQUEST,
                EXIT_UNPARSEABLE_REQUEST,
                EXIT_WIRE_VERSION_MISMATCH,
                EXIT_UNSERIALIZABLE_RESPONSE,
                EXIT_UNWRITABLE_RESPONSE,
                EXIT_PANICKED,
            ],
            [0x204D_5801, 0x204D_5802, 0x204D_5803, 4, 5, 6]
        );

        let (hive, key, value_name) = (RegistryHive::Hkcu, "K".to_owned(), "V".to_owned());
        let request = BrokerRequest {
            nonce: 7,
            ..BrokerRequest::new(vec![
                BrokerOp::RegSet {
                    hive: RegistryHive::Hklm,
                    key: key.clone(),
                    value_name: value_name.clone(),
                    value_type: RegistryValueType::Dword,
                    value: serde_json::json!(1),
                },
                BrokerOp::RegDeleteValue {
                    hive,
                    key: key.clone(),
                    value_name,
                },
                BrokerOp::RegDeleteKey {
                    hive,
                    key: key.clone(),
                },
                BrokerOp::RegCreateKey { hive, key },
                BrokerOp::SvcSetStartup {
                    name: "S".into(),
                    startup: ServiceStartupType::Manual,
                },
                BrokerOp::Scheduler {
                    task_path: "P".into(),
                    task_name: "T".into(),
                    action: SchedulerAction::Disable,
                },
            ])
        };
        let response = BrokerResponse {
            version: WIRE_VERSION,
            nonce: 7,
            attempted: 2,
            failure: Some(OpFailure {
                index: 1,
                message: "denied".into(),
            }),
        };
        assert_eq!(serde_json::to_string(&request).unwrap(), REQUEST);
        assert_eq!(serde_json::to_string(&response).unwrap(), RESPONSE);
        assert_eq!(
            serde_json::from_str::<BrokerRequest>(REQUEST).unwrap(),
            request
        );
        assert_eq!(
            serde_json::from_str::<BrokerResponse>(RESPONSE).unwrap(),
            response
        );
    }

    // The fake child covers the parent side only: the real child cannot run under `cargo test`, where
    // `current_exe()` is the libtest harness and rejects `--broker`. Verify it with the built exe:
    // `magicx-toolbox.exe --broker <req> <resp>` from an elevated shell, then check the machine state.

    /// The argv the child sees for `cmdline`.
    fn child_argv(cmdline: &str) -> Vec<std::ffi::OsString> {
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::UI::Shell::CommandLineToArgvW;

        let wide = super::super::common::to_wide_string(cmdline);
        let mut argc = 0;
        // SAFETY: `wide` is NUL-terminated; each entry is a NUL-terminated string inside the one
        // allocation, freed once all are copied out.
        unsafe {
            let argv = CommandLineToArgvW(wide.as_ptr(), &mut argc);
            assert!(!argv.is_null(), "CommandLineToArgvW failed on {cmdline}");
            let args = (0..argc as usize)
                .map(|i| {
                    let arg = *argv.add(i);
                    let len = (0..).take_while(|&j| *arg.add(j) != 0).count();
                    std::ffi::OsString::from_wide(std::slice::from_raw_parts(arg, len))
                })
                .collect();
            LocalFree(argv.cast());
            args
        }
    }

    /// Runs `ops` through the whole elevated parent side with `child` in place of the
    /// TrustedInstaller spawn, then asserts the request and response files are gone.
    fn run_with_child(
        ops: Vec<BrokerOp>,
        child: impl FnOnce(&str, &str) -> Result<i32, SpawnError>,
    ) -> Result<(), BrokerOpError> {
        run_with_child_paths(ops, child).0
    }

    /// [`run_with_child`], also handing back the two temp paths the child was given.
    fn run_with_child_paths(
        ops: Vec<BrokerOp>,
        child: impl FnOnce(&str, &str) -> Result<i32, SpawnError>,
    ) -> (Result<(), BrokerOpError>, [String; 2]) {
        let mut handed = None;
        let result = run_ops_with(Elevation::TrustedInstaller, ops, |cmdline| {
            let argv = child_argv(cmdline);
            assert_eq!(
                std::path::Path::new(&argv[0]),
                std::env::current_exe().unwrap()
            );
            let crate::Launch::Broker { req, resp } = crate::classify_launch(&argv) else {
                panic!("the child would not start as the broker: {cmdline}");
            };
            handed = Some([req.to_owned(), resp.to_owned()]);
            child(req, resp)
        });
        let handed = handed.expect("the elevated arm must spawn a child");
        for path in &handed {
            assert!(!std::path::Path::new(path).exists(), "left behind: {path}");
        }
        (result, handed)
    }

    fn serve(req: &str, resp: &str) -> Result<i32, SpawnError> {
        Ok(serve_request(req, resp))
    }

    /// Parses the request as the real child does, then writes a success response, edited by
    /// `edit`, without running any op.
    fn respond_with(
        edit: impl FnOnce(&BrokerRequest, &mut serde_json::Value),
    ) -> impl FnOnce(&str, &str) -> Result<i32, SpawnError> {
        move |req, resp| {
            let bytes = std::fs::read(req).unwrap();
            let Ok(request) = parse_wire(&bytes, |r: &BrokerRequest| r.version) else {
                panic!("the child refused the request");
            };
            let mut body =
                serde_json::to_value(clean_response(request.nonce, request.ops.len())).unwrap();
            edit(&request, &mut body);
            write_response(resp, &to_bytes(&body)).unwrap();
            Ok(0)
        }
    }

    fn scratch_ops(key: &str) -> Vec<BrokerOp> {
        vec![
            BrokerOp::RegCreateKey {
                hive: RegistryHive::Hkcu,
                key: key.into(),
            },
            BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: key.into(),
                value_name: "Flag".into(),
                value_type: RegistryValueType::Dword,
                value: serde_json::json!(9),
            },
        ]
    }

    #[test]
    fn the_elevated_arm_round_trips_through_a_fake_child() {
        let scratch = Scratch::new();
        run_with_child(scratch_ops(&scratch.key), serve).expect("a clean batch is Ok");
        assert_eq!(
            registry_service::read_dword(&RegistryHive::Hkcu, &scratch.key, "Flag").unwrap(),
            Some(9)
        );
    }

    #[test]
    fn a_child_that_exits_before_any_op_is_could_not_acquire() {
        let scratch = Scratch::new();
        for code in [
            EXIT_UNREADABLE_REQUEST,
            EXIT_UNPARSEABLE_REQUEST,
            EXIT_WIRE_VERSION_MISMATCH,
            malformed_argv_exit_code(),
        ] {
            let got = run_with_child(scratch_ops(&scratch.key), |_, _| Ok(code));
            assert!(
                matches!(got, Err(BrokerOpError::CouldNotAcquire(_))),
                "exit {code:#x}: {got:?}"
            );
        }
    }

    /// The child ran the batch and left a valid success response, which must not rescue the exit.
    #[test]
    fn a_child_that_exits_nonzero_after_running_is_outcome_unknown() {
        let scratch = Scratch::new();
        for code in [
            EXIT_UNSERIALIZABLE_RESPONSE,
            EXIT_UNWRITABLE_RESPONSE,
            EXIT_PANICKED,
            1,
            3,
            0xC000_0409_u32 as i32,
        ] {
            let got = run_with_child(scratch_ops(&scratch.key), |req, resp| {
                assert_eq!(serve_request(req, resp), 0);
                Ok(code)
            });
            assert!(
                matches!(got, Err(BrokerOpError::Indeterminate(_))),
                "exit {code:#x}: {got:?}"
            );
        }
    }

    #[test]
    fn a_spawn_error_says_whether_a_child_was_created() {
        let scratch = Scratch::new();
        let detail = |msg: &str| Error::ServiceControl(msg.into());

        let got = run_with_child(scratch_ops(&scratch.key), |_, _| {
            Err(SpawnError::NoChild(detail("CreateProcessW failed")))
        });
        assert!(
            matches!(got, Err(BrokerOpError::CouldNotAcquire(_))),
            "{got:?}"
        );

        let got = run_with_child(scratch_ops(&scratch.key), |req, resp| {
            assert_eq!(serve_request(req, resp), 0);
            Err(SpawnError::ChildRan(detail("timed out")))
        });
        assert!(
            matches!(got, Err(BrokerOpError::Indeterminate(_))),
            "{got:?}"
        );
    }

    /// A detail carrying one of everything a log line must never print: a path, a transport nonce,
    /// and a registry key with its data.
    const HOSTILE_DETAIL: &str =
        "C:\\Users\\Someone\\AppData\\Local\\Temp\\magicx-broker-resp.json \
         nonce 0x0123456789abcdef HKLM\\Software\\Policies\\Secret = 1";

    /// Summarises `e` and fails if any part of [`HOSTILE_DETAIL`] survived into the line.
    fn hostile_summary(e: BrokerOpError) -> String {
        let summary = failure_summary(Elevation::TrustedInstaller, &e);
        for leaked in [
            "C:\\Users",
            "magicx-broker",
            "0x0123456789abcdef",
            "HKLM",
            "Secret",
        ] {
            assert!(!summary.contains(leaked), "{summary} names {leaked}");
        }
        summary
    }

    /// What a support engineer gets for a failed elevated batch, and what they must never get: the
    /// two temp paths the parent generated for this very run.
    #[test]
    fn a_failed_elevated_batch_is_summarised_without_any_path() {
        let scratch = Scratch::new();
        let (got, handed) = run_with_child_paths(scratch_ops(&scratch.key), |_, _| {
            Ok(EXIT_UNREADABLE_REQUEST)
        });
        let err = got.expect_err("a nothing-ran exit fails the batch");

        let summary = failure_summary(Elevation::TrustedInstaller, &err);
        assert!(summary.contains("TrustedInstaller"), "{summary}");
        assert!(summary.contains("nothing ran"), "{summary}");
        for path in handed {
            assert!(!summary.contains(&path), "{summary} names {path}");
        }
    }

    /// Each classification reads as itself, and none of them echoes the detail it carries.
    #[test]
    fn no_classification_echoes_the_detail_behind_it() {
        let detail = || Error::ServiceControl(HOSTILE_DETAIL.to_owned());

        let acquire = hostile_summary(BrokerOpError::CouldNotAcquire(detail()));
        assert!(acquire.contains("nothing ran"), "{acquire}");

        let unknown = hostile_summary(BrokerOpError::Indeterminate(detail()));
        assert!(unknown.contains("outcome is unknown"), "{unknown}");

        let op = hostile_summary(BrokerOpError::OpFailed {
            index: Some(2),
            source: detail(),
        });
        assert!(op.contains("operation 2"), "{op}");

        let unplaced = hostile_summary(BrokerOpError::OpFailed {
            index: None,
            source: detail(),
        });
        assert!(unplaced.contains("an operation was refused"), "{unplaced}");
    }

    /// Only an elevated batch is the broker's to record, and only an unknown outcome is an error:
    /// the other two say plainly what happened to the machine.
    #[test]
    fn only_an_elevated_failure_is_recorded_and_only_an_unknown_outcome_is_an_error() {
        let detail = || Error::ServiceControl(HOSTILE_DETAIL.to_owned());
        let at = |level, e: BrokerOpError| failure_log_level(level, &e);

        assert_eq!(
            at(Elevation::None, BrokerOpError::Indeterminate(detail())),
            None
        );
        assert_eq!(
            at(
                Elevation::TrustedInstaller,
                BrokerOpError::Indeterminate(detail())
            ),
            Some(log::Level::Error)
        );
        assert_eq!(
            at(
                Elevation::TrustedInstaller,
                BrokerOpError::CouldNotAcquire(detail())
            ),
            Some(log::Level::Warn)
        );
        assert_eq!(
            at(
                Elevation::TrustedInstaller,
                BrokerOpError::OpFailed {
                    index: None,
                    source: detail(),
                }
            ),
            Some(log::Level::Warn)
        );
    }

    #[test]
    fn a_response_the_parent_cannot_trust_is_outcome_unknown() {
        type Child = Box<dyn FnOnce(&str, &str) -> Result<i32, SpawnError>>;
        let scratch = Scratch::new();
        let cases: [(&str, Child); 5] = [
            ("missing", Box::new(|_: &str, _: &str| Ok(0))),
            (
                "garbage",
                Box::new(|_: &str, resp: &str| {
                    write_response(resp, b"\x00 not a response").unwrap();
                    Ok(0)
                }),
            ),
            (
                "wrong nonce",
                Box::new(respond_with(|r, body| {
                    body["nonce"] = serde_json::json!(r.nonce ^ 1)
                })),
            ),
            (
                "short attempted count",
                Box::new(respond_with(|r, body| {
                    body["attempted"] = serde_json::json!(r.ops.len() - 1)
                })),
            ),
            (
                "wrong version",
                Box::new(respond_with(|_, body| {
                    body["version"] = serde_json::json!(WIRE_VERSION + 1)
                })),
            ),
        ];
        let trusted: Vec<String> = cases
            .into_iter()
            .filter_map(|(case, child)| {
                let got = run_with_child(scratch_ops(&scratch.key), child);
                let unknown = matches!(got, Err(BrokerOpError::Indeterminate(_)));
                (!unknown).then(|| format!("{case}: {got:?}"))
            })
            .collect();
        assert!(trusted.is_empty(), "{trusted:#?}");
    }

    #[test]
    fn an_op_failure_in_the_child_names_that_op() {
        let scratch = Scratch::new();
        let mut ops = scratch_ops(&scratch.key);
        for (value_name, value) in [("Bad", "not-a-number".into()), ("Second", 1.into())] {
            ops.push(BrokerOp::RegSet {
                hive: RegistryHive::Hkcu,
                key: scratch.key.clone(),
                value_name: value_name.into(),
                value_type: RegistryValueType::Dword,
                value,
            });
        }
        let err = run_with_child(ops, serve).expect_err("a failing op fails the batch");
        assert!(matches!(err, BrokerOpError::OpFailed { .. }), "{err:?}");
        assert_eq!(err.failed_op_index(), Some(2));
        assert_value_never_written(&scratch.key, "Second");
    }

    /// Every `BrokerOp` variant and every value of the enums inside one. Never executed: the
    /// service and scheduler ops would change the real machine.
    fn every_op() -> Vec<BrokerOp> {
        use RegistryHive::{Hkcu, Hklm};
        let key = "Software\\MagicX \"quoted\" \u{e9}\\K".to_owned();
        let values = [
            (RegistryValueType::Dword, serde_json::json!(u32::MAX)),
            (RegistryValueType::Qword, serde_json::json!(u64::MAX)),
            (RegistryValueType::String, serde_json::json!("text")),
            (
                RegistryValueType::ExpandString,
                serde_json::json!("%SystemRoot%\\x"),
            ),
            (RegistryValueType::MultiString, serde_json::json!(["a", ""])),
            (RegistryValueType::Binary, serde_json::json!([0, 255])),
        ];
        let mut ops: Vec<BrokerOp> = values
            .into_iter()
            .zip([Hkcu, Hklm].into_iter().cycle())
            .map(|((value_type, value), hive)| BrokerOp::RegSet {
                hive,
                key: key.clone(),
                value_name: String::new(),
                value_type,
                value,
            })
            .collect();
        for hive in [Hkcu, Hklm] {
            ops.extend([
                BrokerOp::RegDeleteValue {
                    hive,
                    key: key.clone(),
                    value_name: "V".into(),
                },
                BrokerOp::RegDeleteKey {
                    hive,
                    key: key.clone(),
                },
                BrokerOp::RegCreateKey {
                    hive,
                    key: key.clone(),
                },
            ]);
        }
        use ServiceStartupType::{Automatic, Boot, Disabled, Manual, System};
        ops.extend([Disabled, Manual, Automatic, Boot, System].map(|startup| {
            BrokerOp::SvcSetStartup {
                name: "S".into(),
                startup,
            }
        }));
        ops.extend(
            [SchedulerAction::Enable, SchedulerAction::Disable].map(|action| BrokerOp::Scheduler {
                task_path: "\\Microsoft\\Windows\\P".into(),
                task_name: "T".into(),
                action,
            }),
        );
        ops
    }

    #[test]
    fn every_broker_op_crosses_from_the_parent_to_the_child_intact() {
        let sent = every_op();
        // No wildcard: a new variant fails to compile here until `every_op` sends it.
        let variants: std::collections::HashSet<u8> = sent
            .iter()
            .map(|op| match op {
                BrokerOp::RegSet { .. } => 0,
                BrokerOp::RegDeleteValue { .. } => 1,
                BrokerOp::RegDeleteKey { .. } => 2,
                BrokerOp::RegCreateKey { .. } => 3,
                BrokerOp::SvcSetStartup { .. } => 4,
                BrokerOp::Scheduler { .. } => 5,
            })
            .collect();
        assert_eq!(variants.len(), 6);

        let expected = sent.clone();
        run_with_child(sent, respond_with(|r, _| assert_eq!(r.ops, expected)))
            .expect("the child parsed exactly what the parent sent");
    }

    /// The child creates the response at a path the parent only reserved. `CREATE_ALWAYS` follows a
    /// reparse point planted there and writes wherever it points, as TrustedInstaller; `CREATE_NEW`
    /// must refuse anything already there.
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

    /// Only a nothing-ran exit lets the caller consume the snapshot (ADR-0002); any code a kill,
    /// crash or panic can produce must stay outcome-unknown.
    #[test]
    fn a_broker_exit_says_whether_anything_could_have_run() {
        let detail = || Error::ServiceControl("detail".into());

        for code in [
            EXIT_UNREADABLE_REQUEST,
            EXIT_UNPARSEABLE_REQUEST,
            EXIT_WIRE_VERSION_MISMATCH,
        ] {
            assert!(
                matches!(
                    classify_exit(code, detail()),
                    BrokerOpError::CouldNotAcquire(_)
                ),
                "exit {code} happens before any op runs"
            );
        }

        assert!(
            matches!(
                classify_exit(malformed_argv_exit_code(), detail()),
                BrokerOpError::CouldNotAcquire(_)
            ),
            "a malformed --broker argv exits before any op runs"
        );

        for code in [
            EXIT_UNSERIALIZABLE_RESPONSE,
            EXIT_UNWRITABLE_RESPONSE,
            EXIT_PANICKED,
            // Killed mid-batch (`TerminateProcess` takes any code), CRT `abort()`, a fail-fast.
            1,
            2,
            3,
            42,
            0xC000_0409_u32 as i32,
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
