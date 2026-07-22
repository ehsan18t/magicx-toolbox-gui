//! The `EffectKind` contract (spec §5): one trait every address kind implements, so the engine
//! (a later task) can treat Registry/RegistryKey/Service/Task/Hosts/Firewall uniformly and mock
//! any of them in tests. Task 5 implemented `Setting::Registry` and `Setting::RegistryKey` (see
//! [`registry`]); this task adds `Setting::Service` and `Setting::Task` (see [`service`] and
//! [`task`]); Task 7 adds Hosts/Firewall.
//!
//! ## `ExecCx` and the broker seam (read this before wiring System/TI in a later task)
//!
//! `ExecCx` carries the effective elevation [`Level`] for one operation. Per spec invariant 24,
//! *reads* always run at the current, in-process level — there is no elevated read op in the
//! broker protocol (`services::elevation::broker::BrokerOp` has no `RegRead*` variant; a read
//! never needs a fresh child process), so a kind's `read` must never gate on `cx.level()`.
//! *Drives* (writes) do need to escalate for `Level::System`/`Level::Ti`: "User/Admin in-process,
//! System/TI in fresh children." This build implements only the in-process half — every kind's
//! `drive` must reject `System`/`Ti` with a typed [`Error`], never a silent no-op or a fake
//! success.
//!
//! `ExecCx`'s field stays private, constructed only through [`ExecCx::new`], specifically so a
//! later task can grow it without breaking this task's callers or touching the trait below. That
//! task's shape, concretely:
//! - Add a field to `ExecCx` (e.g. a broker handle/sender) behind a new constructor (say
//!   `ExecCx::with_broker(level, handle)`), leaving today's `ExecCx::new` as the no-broker case.
//! - In each kind's `drive`, replace the `System`/`Ti` error arm with: build the matching
//!   `BrokerOp` — the registry kind's ops already exist verbatim in the broker protocol
//!   (`RegSet` / `RegDeleteValue` / `RegDeleteKey` / `RegCreateKey`, spec §5.1) — and hand it to
//!   the `ExecCx`'s broker handle, which calls `services::elevation::broker::run_elevated_broker`.
//! - Precondition for that task: `services/elevation/mod.rs` re-exports only `run_broker` and
//!   `run_scheduler_op` from its private `broker` submodule today. Reaching `BrokerOp` /
//!   `run_elevated_broker` from `tweaks::kinds` needs new re-exports there (or `pub(crate) mod
//!   broker`) — out of scope here since it touches a file outside this task's boundary.
//! - None of the above changes `EffectKind`'s signature — only `ExecCx`'s internals and each
//!   kind's `System`/`Ti` branch change.

pub mod registry;
pub mod service;
pub mod task;

use crate::error::Error as BackendError;
use crate::tweaks::model::{Level, RegType, Setting, Value};
use crate::tweaks::parse::ParseError;

/// Errors an [`EffectKind`] can return. Every case a caller must act on differently gets its own
/// variant (spec invariant 2): a missing *value* is not an error at all (`read` returns
/// `Ok(Value::Absent)`), but a missing *key*, a denied operation, a type mismatch, and a malformed
/// packed value must never collapse into one another or into an opaque string.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The registry key itself does not exist. A plain value `read` absorbs this into
    /// `Ok(Value::Absent)` (a missing key trivially means a missing value); this variant exists
    /// for the call paths where that collapse does not apply.
    #[error("registry key not found: {0}")]
    KeyNotFound(String),

    /// The operation was denied — insufficient rights, including "this build cannot yet reach
    /// the privilege it needs".
    #[error("registry access denied: {0}")]
    AccessDenied(String),

    /// The value exists but its stored type does not match what this address declares.
    #[error("{path}\\{name} is stored as {actual:?}, not the declared {expected:?}")]
    TypeMismatch {
        path: String,
        name: String,
        expected: RegType,
        actual: RegType,
    },

    /// A packed value's live string could not be parsed by its declared format — never guessed
    /// at, never partially rewritten (spec §5.2).
    #[error("{path}\\{name} is not a valid packed value: {source}")]
    MalformedPacked {
        path: String,
        name: String,
        #[source]
        source: ParseError,
    },

    /// `cx`'s elevation level has no in-process implementation yet (see the module docs above).
    #[error("{0:?} elevation is not yet routed by this build")]
    UnsupportedLevel(Level),

    /// The addressed service or task does not exist, but the caller asked to drive it to a real
    /// (non-`Missing`) value. The engine never installs or uninstalls services/tasks (spec §5.4,
    /// invariant 12), so this is a typed refusal, never a silent no-op — distinct from driving
    /// *to* `Missing`, which is a defined no-op (`Ok(())`) regardless of whether the resource
    /// exists.
    #[error("{0}")]
    ResourceMissing(String),

    /// A caller routed a `Setting`/`Value` this kind does not own to it — an engine dispatch bug,
    /// not a runtime condition. Kept typed rather than a panic: this trait also runs inside the
    /// elevated broker process, where a panic would abort an entire batch.
    #[error("{0}")]
    Invalid(&'static str),

    /// Anything else the backing primitive reported (registry, service, or task -- kept
    /// kind-neutral since `map_backend_error` routes all three kinds through this variant).
    #[error("operation failed: {0}")]
    Backend(String),
}

/// Execution context an [`EffectKind`] runs under. See the module docs for the broker seam a
/// later task attaches without changing [`EffectKind`] itself.
pub struct ExecCx {
    level: Level,
}

impl ExecCx {
    pub fn new(level: Level) -> Self {
        Self { level }
    }

    pub fn level(&self) -> Level {
        self.level
    }
}

/// One address kind's read/drive behavior (spec §5). The contract every kind implements; the
/// engine (a later task) dispatches on `Setting`'s variant and mocks this trait in its own tests.
pub trait EffectKind: Send + Sync {
    /// The current value at `s`'s address. Never guesses: an unreadable or unparseable state is
    /// a typed `Err`, not a fabricated `Value` (invariant 3).
    fn read(&self, s: &Setting, cx: &ExecCx) -> Result<Value, Error>;

    /// Drives `s`'s address to `target`.
    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error>;
}

// --- helpers shared by the service and task kinds ------------------------------------------------
//
// The registry kind (`registry.rs`) predates these and keeps its own private copies so Task 5's
// already-reviewed file stays untouched; these exist once here because two new kinds need them.

/// `Level::User`/`Level::Admin` run in-process; `System`/`Ti` need the elevation broker, which
/// this build does not wire up yet (see the module docs above).
fn guard_level(cx: &ExecCx) -> Result<(), Error> {
    match cx.level() {
        Level::User | Level::Admin => Ok(()),
        other => Err(Error::UnsupportedLevel(other)),
    }
}

/// Backend-error fallback for kinds whose primitive exposes no richer typed distinction than this
/// (service/task): a declared "requires admin" signal becomes our typed [`Error::AccessDenied`];
/// anything else is the least-specific [`Error::Backend`] bucket. Never produces `Value::Missing`
/// — that is exclusively the caller's job when the resource genuinely does not exist (invariant
/// 2), so a backend error here can never be confused with an absent resource.
fn map_backend_error(e: BackendError) -> Error {
    match e {
        BackendError::RequiresAdmin => {
            Error::AccessDenied("requires administrator privileges".to_string())
        }
        other => Error::Backend(other.to_string()),
    }
}
