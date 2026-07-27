//! The elevated effect broker.
//!
//! The broker is nothing more than the effect services running in an elevated process. Instead of
//! composing `cmd.exe /c <string>` command lines and escaping values (the source of the injection
//! and REG_SZ-corruption classes), the main app serializes a list of **typed** operations, spawns
//! this broker with a SYSTEM or TrustedInstaller token, and the broker runs the very same effect
//! functions the unelevated path uses — now succeeding on protected resources because the process
//! holds the elevated token.
//!
//! Transport is a request file + a response file (paths passed as argv to `--broker`), so no shell
//! ever parses anything and every result crosses back as typed data. There is no interpreter op:
//! every variant of [`BrokerOp`] names a typed effect, so nothing the elevated child can be asked
//! to do is "run this string".

use crate::error::Error;
use crate::models::{RegistryHive, RegistryValueType, SchedulerAction, ServiceStartupType};
use crate::services::{registry_service, registry_value, scheduler_service, service_control};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

use super::Elevation;

/// One typed operation for the broker to perform in the elevated process.
///
/// Every variant must have a producer in `tweaks::kinds` (`to_broker_op`/`to_broker_ops`). A
/// variant with no producer is still reachable by anything that can hand the child a request file,
/// so it is pure attack surface — add one only together with the translation that emits it.
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

/// Broker entrypoint: read a request file, execute it, write a response file. Returns a process
/// exit code (0 = the batch was executed and a response was written; non-zero = the broker could
/// not read the request or write the response — a transport failure, distinct from op failures,
/// which are reported inside the response).
pub fn run_broker(req_path: &str, resp_path: &str) -> i32 {
    let bytes = match std::fs::read(req_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("broker: failed to read request {}: {}", req_path, e);
            return 2;
        }
    };
    let request: BrokerRequest = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("broker: failed to parse request: {}", e);
            return 3;
        }
    };

    let response = execute_request(&request);

    let out = match serde_json::to_vec(&response) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("broker: failed to serialize response: {}", e);
            return 4;
        }
    };
    if let Err(e) = std::fs::write(resp_path, out) {
        eprintln!("broker: failed to write response {}: {}", resp_path, e);
        return 5;
    }
    0
}

/// Monotonic counter mixed into the per-invocation transport nonce.
static BROKER_SEQ: AtomicU64 = AtomicU64::new(0);

/// A per-invocation transport nonce. Mixes wall-clock, a process-local counter, and the pid so two
/// invocations get distinct nonces even across a process restart that reuses our pid and resets the
/// counter — the exact conjunction that could otherwise let a stale response file be read as a
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
/// `Elevation::None` runs them in-process (the effect services already hold the needed rights).
/// `System` / `TrustedInstaller` serialize the request to a temp file and spawn
/// `<this exe> --broker <req> <resp>` under the corresponding token (reusing the winlogon token-dup
/// / TI parent-spoof primitives), then read the typed response back. No shell parses the
/// operations, and the request *data* never appears on a command line — only our controlled
/// temp-file paths do.
///
/// Trust in the response is gated three ways: the response path is pre-cleared, the child's exit
/// code must be 0 (run_broker returns 0 only *after* writing the response), and the response's
/// nonce must match the one sent — so a leftover file from a prior run can never be read as this
/// run's result.
pub fn run_elevated_broker(
    level: Elevation,
    request: &BrokerRequest,
) -> Result<BrokerResponse, Error> {
    if !level.is_elevated() {
        return Ok(execute_request(request));
    }

    let exe = std::env::current_exe()
        .map_err(|e| Error::ServiceControl(format!("current_exe failed: {}", e)))?;

    let nonce = next_nonce();
    let wire = BrokerRequest {
        nonce,
        ops: request.ops.clone(),
    };

    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let req_path = dir.join(format!("magicx-broker-{}-{:016x}-req.json", pid, nonce));
    let resp_path = dir.join(format!("magicx-broker-{}-{:016x}-resp.json", pid, nonce));

    // Never trust a leftover file at the response path.
    let _ = std::fs::remove_file(&resp_path);

    let req_json = serde_json::to_vec(&wire)
        .map_err(|e| Error::ServiceControl(format!("serialize broker request: {}", e)))?;
    std::fs::write(&req_path, &req_json)
        .map_err(|e| Error::ServiceControl(format!("write broker request: {}", e)))?;

    // Spawn "<exe>" --broker "<req>" "<resp>" directly (no cmd.exe wrapper). Paths are quoted; the
    // values are our own generated temp names, never untrusted data.
    let cmdline = format!(
        "\"{}\" --broker \"{}\" \"{}\"",
        exe.display(),
        req_path.display(),
        resp_path.display()
    );

    let spawn = match level {
        Elevation::System => super::system_elevation::spawn_as_system(&cmdline),
        Elevation::TrustedInstaller => super::ti_elevation::spawn_as_trusted_installer(&cmdline),
        Elevation::None => unreachable!("handled above"),
    };

    // Gate on the child's real exit code before trusting the file: a non-zero exit means the broker
    // did not finish writing the response, so any bytes at resp_path are stale or partial.
    let read = spawn.and_then(|exit| {
        if exit != 0 {
            return Err(Error::ServiceControl(format!(
                "broker process exited with code {} without completing (transport failure)",
                exit
            )));
        }
        std::fs::read(&resp_path)
            .map_err(|e| Error::ServiceControl(format!("broker wrote no response: {}", e)))
    });

    let _ = std::fs::remove_file(&req_path);
    let _ = std::fs::remove_file(&resp_path);

    validate_response(&read?, nonce)
}

/// The two distinct ways a multi-op batch can fail (spec §9, ADR-0005 as amended; invariant 24):
/// the elevated child could never be ACQUIRED at all (environmental -- the TI service would not
/// start, `SeDebugPrivilege` was denied, winlogon was not found, or the child failed to spawn or
/// respond) versus the child WAS acquired and ran, but at least one operation inside it failed
/// (the declaration is genuinely too low for this machine). Both abort + roll back at the call
/// site; neither is ever silently downgraded to the other or to a benign value.
#[derive(Debug, thiserror::Error)]
pub enum BrokerOpError {
    #[error("could not acquire the elevated child: {0}")]
    CouldNotAcquire(#[source] Error),
    #[error("operation failed inside the elevated child: {0}")]
    OpFailed(#[source] Error),
}

/// Runs a whole batch of operations in ONE elevated child (spec §9's grouped execution). The only
/// entry point into the broker.
pub fn run_ops(level: Elevation, ops: Vec<BrokerOp>) -> Result<(), BrokerOpError> {
    let sent = ops.len();
    let response = run_elevated_broker(level, &BrokerRequest { nonce: 0, ops })
        .map_err(BrokerOpError::CouldNotAcquire)?;
    check_response(sent, &response)
}

/// The did-it-work decision, isolated from the spawn so it is testable against a response the
/// executor would never produce. Success requires BOTH no reported failure AND a full attempt
/// count: a response that ran fewer ops than were sent, yet names no failure, is a truncated or
/// forged one, and reporting it as `Ok(())` would leave a half-applied batch looking complete.
fn check_response(sent: usize, response: &BrokerResponse) -> Result<(), BrokerOpError> {
    if let Some(failure) = &response.failure {
        return Err(BrokerOpError::OpFailed(Error::ServiceControl(format!(
            "broker op {} failed: {}",
            failure.index, failure.message
        ))));
    }
    if response.attempted != sent {
        return Err(BrokerOpError::OpFailed(Error::ServiceControl(format!(
            "broker attempted {} of {sent} ops but reported no failure",
            response.attempted
        ))));
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
        // Key present, value absent — the common "already gone" case the apply flow hits.
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

        let err = check_response(3, &clean(2))
            .expect_err("two of three attempted with no failure is not a completed batch");
        assert!(matches!(err, BrokerOpError::OpFailed(_)), "got {err:?}");
        assert!(err.to_string().contains("2 of 3"), "got {err}");

        let err = check_response(3, &clean(0))
            .expect_err("a response claiming nothing ran must never be Ok");
        assert!(matches!(err, BrokerOpError::OpFailed(_)), "got {err:?}");
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
            matches!(err, BrokerOpError::OpFailed(_)),
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
        assert!(matches!(err, BrokerOpError::OpFailed(_)), "got {err:?}");
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

    // Deliberately NOT a `#[test]`: `run_ops(Elevation::System, ...)` respawns
    // `std::env::current_exe()` with `--broker <req> <resp>` (see `run_elevated_broker` above),
    // and only `main.rs` (via `run_broker_if_requested`) understands that flag -- under `cargo
    // test`, `current_exe()` is the libtest harness binary, which rejects `--broker` as an
    // unrecognized argument and exits non-zero. That is a structural property of the respawn
    // design (pre-existing, shared by every `*_as_system`/`*_as_ti` wrapper, not something Task 14
    // introduced) and cannot be made to pass from inside this test binary.
    //
    // The real end-to-end path (Task 14's grouped-execution smoke test) was instead verified
    // manually against the actually-built `magicx-toolbox.exe`: a `BrokerRequest` with this exact
    // grouped write-then-delete-then-delete-key shape (`RegSet` -> `RegDeleteValue` ->
    // `RegDeleteKey`, one child, spec §9) was serialized to a file and fed to
    // `magicx-toolbox.exe --broker <req> <resp>` directly (elevated Administrator shell); the
    // response reported `["Ok","Ok","Ok"]`, and an independent PowerShell `Test-Path` against
    // `HKLM:\SOFTWARE\MagicXToolboxTest\...` confirmed no residue afterward. See the Task 14
    // report for the full transcript.
}
