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
//! ever parses anything and every result crosses back as typed data. The only interpreter cases —
//! PowerShell and author `pre/post_commands` — are spawned directly as argv (`-EncodedCommand` and
//! `cmd /c` respectively), never by composing a command around untrusted values.

use crate::error::Error;
use crate::models::{RegistryHive, RegistryValueType, SchedulerAction, ServiceStartupType};
use crate::services::{registry_service, registry_value, scheduler_service, service_control};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

use super::Elevation;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// One typed operation for the broker to perform in the elevated process.
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
    /// Start a service.
    SvcStart { name: String },
    /// Stop a service.
    SvcStop { name: String },
    /// Enable / disable / delete a scheduled task.
    Scheduler {
        task_path: String,
        task_name: String,
        action: SchedulerAction,
    },
    /// Run a PowerShell script (spawned as `-EncodedCommand`, no shell parsing).
    Powershell { script: String },
    /// Run an author-supplied `cmd.exe` command (single argv to `cmd /c`).
    RawCmd { command: String },
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

/// The outcome of a single operation. Positional: `results[i]` corresponds to `ops[i]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpOutcome {
    Ok,
    Err(String),
}

/// The broker's typed response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrokerResponse {
    /// Echoes the request's [`BrokerRequest::nonce`] so the parent can reject a stale/foreign file.
    #[serde(default)]
    pub nonce: u64,
    pub results: Vec<OpOutcome>,
}

impl BrokerResponse {
    /// Collapse a single-op response into a `Result`. Used by the elevated wrappers, which submit
    /// exactly one op.
    pub fn into_single(mut self) -> Result<(), Error> {
        match self.results.pop() {
            Some(OpOutcome::Ok) => Ok(()),
            Some(OpOutcome::Err(msg)) => Err(Error::ServiceControl(msg)),
            None => Err(Error::ServiceControl(
                "broker returned no result for a single-op request".to_string(),
            )),
        }
    }
}

/// Map registry "not found" into success for delete operations (deleting an absent thing is done).
fn delete_ok(result: Result<(), Error>) -> Result<(), Error> {
    match result {
        Err(Error::RegistryKeyNotFound(_)) => Ok(()),
        other => other,
    }
}

/// Execute one operation using the effect services. Called inside the elevated broker process, so
/// `use_system = false`: the broker's own token already provides the privilege.
pub fn execute_op(op: &BrokerOp) -> Result<(), Error> {
    match op {
        BrokerOp::RegSet {
            hive,
            key,
            value_name,
            value_type,
            value,
        } => registry_value::write_registry_json_value(
            hive, key, value_name, value_type, value, false,
        ),
        BrokerOp::RegDeleteValue {
            hive,
            key,
            value_name,
        } => delete_ok(registry_service::delete_value(hive, key, value_name)),
        BrokerOp::RegDeleteKey { hive, key } => {
            delete_ok(registry_service::delete_key(hive, key))
        }
        BrokerOp::RegCreateKey { hive, key } => registry_service::create_key(hive, key),
        BrokerOp::SvcSetStartup { name, startup } => {
            service_control::set_service_startup(name, startup)
        }
        BrokerOp::SvcStart { name } => service_control::start_service(name),
        BrokerOp::SvcStop { name } => service_control::stop_service(name),
        BrokerOp::Scheduler {
            task_path,
            task_name,
            action,
        } => scheduler_service::apply_scheduler_change(task_path, task_name, *action),
        BrokerOp::Powershell { script } => run_powershell_encoded(script),
        BrokerOp::RawCmd { command } => run_raw_cmd(command),
    }
}

/// Execute every op, collecting per-op outcomes (never short-circuits: each op reports its own).
pub fn execute_request(request: &BrokerRequest) -> BrokerResponse {
    let results = request
        .ops
        .iter()
        .map(|op| match execute_op(op) {
            Ok(()) => OpOutcome::Ok,
            Err(e) => OpOutcome::Err(e.to_string()),
        })
        .collect();
    BrokerResponse {
        nonce: request.nonce,
        results,
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
    nanos
        .rotate_left(17)
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

/// Run a single operation at the given elevation, returning `Ok(())` on success. The elevated
/// wrappers (`*_as_ti` / `*_as_system`) submit exactly one op through this.
pub(super) fn run_one(level: Elevation, op: BrokerOp) -> Result<(), Error> {
    run_elevated_broker(
        level,
        &BrokerRequest {
            nonce: 0,
            ops: vec![op],
        },
    )?
    .into_single()
}

/// Apply a scheduler change (enable / disable / delete) at `level` via the typed `Scheduler` op.
///
/// Unelevated runs the COM op in-process; SYSTEM / TrustedInstaller run the *same* typed op inside
/// the elevated broker. No `schtasks` command string is composed, so a task name containing cmd
/// metacharacters (`& | < > ^ %`) can no longer be corrupted by shell escaping (finding C3).
pub fn run_scheduler_op(
    level: Elevation,
    task_path: &str,
    task_name: &str,
    action: SchedulerAction,
) -> Result<(), Error> {
    run_one(
        level,
        BrokerOp::Scheduler {
            task_path: task_path.to_string(),
            task_name: task_name.to_string(),
            action,
        },
    )
}

/// Run a PowerShell script via `-EncodedCommand` (base64 of UTF-16LE). No shell parses the script.
fn run_powershell_encoded(script: &str) -> Result<(), Error> {
    use std::os::windows::process::CommandExt;

    let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = base64_encode(&utf16);

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &encoded,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to run PowerShell: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::CommandExecution(format!(
            "PowerShell failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Run an author-supplied `cmd.exe` command as a single raw argument (no escaping of a value into a
/// larger command — the string IS the author's command).
fn run_raw_cmd(command: &str) -> Result<(), Error> {
    use std::os::windows::process::CommandExt;

    let output = std::process::Command::new("cmd")
        .raw_arg(format!("/c {}", command))
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to run command: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::CommandExecution(format!(
            "Command failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Standard base64 (RFC 4648) encoder — small enough not to justify a dependency.
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
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
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"M"), "TQ==");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
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
                BrokerOp::SvcStop { name: "Spooler".into() },
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
        assert_eq!(resp.results, vec![OpOutcome::Ok, OpOutcome::Ok]);
    }

    #[test]
    fn run_broker_reads_request_and_writes_response() {
        // The file-transport contract: read a request file, execute, write a response file.
        let scratch = Scratch::new();
        let dir = std::env::temp_dir();
        let seq = SCRATCH_COUNTER.fetch_add(1, Ordering::SeqCst);
        let req_path =
            dir.join(format!("magicx-brokertest-{}-{}-req.json", std::process::id(), seq));
        let resp_path =
            dir.join(format!("magicx-brokertest-{}-{}-resp.json", std::process::id(), seq));

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
        assert_eq!(resp.results, vec![OpOutcome::Ok, OpOutcome::Ok]);
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
        assert_eq!(resp.results, vec![OpOutcome::Ok]);
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
        assert!(resp.results.is_empty());
    }

    #[test]
    fn a_response_with_a_mismatched_nonce_is_rejected() {
        // A stale/foreign response file carries a different nonce than the one we sent — the guard
        // that stops a leftover file from being read as this invocation's success.
        let stale = serde_json::to_vec(&BrokerResponse {
            nonce: 111,
            results: vec![OpOutcome::Ok],
        })
        .unwrap();
        let err = validate_response(&stale, 222).expect_err("mismatched nonce must be rejected");
        assert!(matches!(err, Error::ServiceControl(_)), "got {err:?}");
    }

    #[test]
    fn a_response_with_the_expected_nonce_is_accepted() {
        let good = serde_json::to_vec(&BrokerResponse {
            nonce: 222,
            results: vec![OpOutcome::Ok],
        })
        .unwrap();
        let resp = validate_response(&good, 222).expect("matching nonce must validate");
        assert_eq!(resp.results, vec![OpOutcome::Ok]);
    }

    #[test]
    fn next_nonce_values_are_distinct() {
        assert_ne!(next_nonce(), next_nonce());
    }
}
