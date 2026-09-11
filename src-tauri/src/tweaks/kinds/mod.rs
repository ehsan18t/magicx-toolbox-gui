//! The `EffectKind` contract (spec §5): one trait every address kind implements, so the engine can
//! treat Registry/RegistryKey/Service/Task/Hosts/Firewall uniformly and mock any of them in tests.
//!
//! ## Where the elevation decision lives
//!
//! `ExecCx` carries the effective [`Level`] for one operation, and only *drives* consult it.
//! Reads always run in-process at whatever level the app holds (spec invariant 24): the broker
//! protocol has no read op, because a read never needs a fresh child. So a kind's `read` must
//! never gate on `cx.level()`.
//!
//! The System/Ti routing itself is NOT here. `engine::AllKinds::drive` owns it: at `User`/`Admin`
//! it delegates to the kind's own `drive` below, and at `System`/`Ti` it never calls that `drive`
//! at all, translating the `Setting`/`Value` into `BrokerOp`s (`to_broker_op`/`to_broker_ops` in
//! `registry.rs`/`service.rs`/`task.rs`) and submitting them through `elevation::run_ops` in one
//! child. Translation sits beside the address shape it understands; dispatch sits above every
//! kind. Each `drive` here still rejects `System`/`Ti` itself, so a kind called directly can never
//! silently escalate. Hosts/Firewall have no `BrokerOp`, so they reach that rejection at those
//! levels.

pub mod action;
pub mod firewall;
pub mod hosts;
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

    /// The operation was denied for want of rights. For a routed System/TI drive this also covers
    /// "the child ran, but the op inside it failed", which is why it reads as the broad denial
    /// rather than a registry-specific one.
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

    /// A packed value's live string could not be parsed by its declared format: never guessed
    /// at, never partially rewritten (spec §5.2).
    #[error("{path}\\{name} is not a valid packed value: {source}")]
    MalformedPacked {
        path: String,
        name: String,
        #[source]
        source: ParseError,
    },

    /// `cx`'s level has no routing for this `Setting`: a field-addressed registry write, or a
    /// Hosts/Firewall effect, at `System`/`Ti`, neither of which the broker translation covers.
    #[error("{0:?} elevation is not yet routed by this build")]
    UnsupportedLevel(Level),

    /// Nothing ran: the TI service would not start, `SeDebugPrivilege` was denied, the child was
    /// never created, or it refused its request (unreadable, unparseable, or a different build).
    /// Unlike [`Error::AccessDenied`], where the child ran and an operation was refused.
    #[error("could not acquire {0:?} elevation: {1}")]
    CouldNotAcquireElevation(Level, String),

    /// The elevated child may have run ops: it timed out, its wait or exit-code query failed, it
    /// panicked, or its response was lost or invalid. Rolls back like its neighbours, but the
    /// snapshot survives (ADR-0002, ADR-0005).
    #[error(
        "{0:?} elevation ran but its outcome is unknown, so the machine may have changed: {1}"
    )]
    ElevatedOutcomeUnknown(Level, String),

    /// The addressed service or task does not exist, but the caller asked to drive it to a real
    /// (non-`Missing`) value. The engine never installs or uninstalls services/tasks (spec §5.4),
    /// so this is a typed refusal, never a silent no-op. Driving *to* `Missing` is the defined
    /// no-op instead, whether or not the resource exists.
    #[error("{0}")]
    ResourceMissing(String),

    /// A caller routed a `Setting`/`Value` this kind does not own to it: an engine dispatch bug,
    /// not a runtime condition. Typed rather than a panic because this trait also runs inside the
    /// elevated broker process, where a panic would abort an entire batch.
    #[error("{0}")]
    Invalid(&'static str),

    /// Anything else the backing primitive reported, kept kind-neutral since `map_backend_error`
    /// routes registry, service, and task through this one variant.
    #[error("operation failed: {0}")]
    Backend(String),

    /// An Action's script ran to completion but reported failure via its exit code -- the sole,
    /// locale-independent `apply`/`undo` signal (spec §7). Carries the code rather than collapsing
    /// it into an opaque string, so a caller can log or display it.
    #[error("action exited with code {0}")]
    ActionFailed(i32),

    /// An Action's script process could not be spawned, or was killed after exceeding its bounded
    /// timeout (spec §14). Distinct from [`Error::ActionFailed`]: this means "we could not learn
    /// the answer," which must never present as a benign `Ok(false)`/success (invariant 2) --
    /// exactly the same principle `ActionFailed` vs. this variant draws for `probe`.
    #[error("{0}")]
    ActionExecFailed(String),
}

/// Execution context an [`EffectKind`] runs under. See the module docs for where the elevation
/// decision is made (not here).
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

/// One address kind's read/drive behavior (spec §5). The engine dispatches on `Setting`'s variant
/// and mocks this trait in its own tests.
pub trait EffectKind: Send + Sync {
    /// The current value at `s`'s address. Never guesses: an unreadable or unparseable state is
    /// a typed `Err`, not a fabricated `Value` (invariant 3).
    fn read(&self, s: &Setting, cx: &ExecCx) -> Result<Value, Error>;

    /// Drives `s`'s address to `target`.
    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error>;

    /// Drives a run of Settings that all routed to the same level, in declaration order, stopping
    /// at the first failure exactly as a sequence of [`Self::drive`] calls would.
    ///
    /// The default body IS that sequence, so any implementation that does not override this keeps
    /// the previous behaviour byte for byte -- which is what lets every mock stay a `drive` mock.
    /// [`crate::tweaks::engine::AllKinds`] overrides it to put a whole run of TrustedInstaller
    /// steps through ONE elevated child instead of one child per effect.
    fn drive_batch(&self, items: &[(&Setting, &Value)], cx: &ExecCx) -> Result<(), BatchFailure> {
        for (index, (setting, target)) in items.iter().enumerate() {
            self.drive(setting, target, cx)
                .map_err(|error| BatchFailure { index, error })?;
        }
        Ok(())
    }
}

/// Which item of a [`EffectKind::drive_batch`] slice failed, and why.
///
/// The index is structural rather than a message: the caller holds the effect ids that produced the
/// slice, and needs to name the failing one to roll back correctly. A batch stops at its first
/// failure, so exactly one of these is ever produced.
#[derive(Debug)]
pub struct BatchFailure {
    pub index: usize,
    pub error: Error,
}

// --- helpers shared by the service and task kinds ------------------------------------------------

/// `User`/`Admin` run in-process; `System`/`Ti` are routed to the broker one layer up, so reaching
/// this kind's own `drive` at those levels means the routing was bypassed (see the module docs).
fn guard_level(cx: &ExecCx) -> Result<(), Error> {
    match cx.level() {
        Level::User | Level::Admin => Ok(()),
        other => Err(Error::UnsupportedLevel(other)),
    }
}

/// Backend-error fallback for kinds whose primitive exposes no richer typed distinction than this
/// (service/task): a declared "requires admin" signal becomes our typed [`Error::AccessDenied`];
/// anything else is the least-specific [`Error::Backend`] bucket. Never produces `Value::Missing`:
/// that is exclusively the caller's job when the resource genuinely does not exist (invariant
/// 2), so a backend error here can never be confused with an absent resource.
fn map_backend_error(e: BackendError) -> Error {
    match e {
        BackendError::RequiresAdmin => {
            Error::AccessDenied("requires administrator privileges".to_string())
        }
        other => Error::Backend(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "Could not acquire the level" and "acquired it, but access denied" stay two distinct typed
    /// failures, distinguishable by variant rather than by string-matching a message.
    #[test]
    fn insufficient_elevation_two_distinct_errors() {
        let could_not_acquire = Error::CouldNotAcquireElevation(
            Level::Ti,
            "TrustedInstaller service would not start".into(),
        );
        let acquired_but_denied =
            Error::AccessDenied("policy denies this key even as SYSTEM".into());

        assert!(matches!(
            could_not_acquire,
            Error::CouldNotAcquireElevation(..)
        ));
        assert!(matches!(acquired_but_denied, Error::AccessDenied(..)));
        // Genuinely distinguishable, not just differently-worded instances of one variant.
        assert_ne!(
            std::mem::discriminant(&could_not_acquire),
            std::mem::discriminant(&acquired_but_denied)
        );
    }

    #[test]
    fn an_unknown_outcome_names_the_level_before_the_detail() {
        let err = Error::ElevatedOutcomeUnknown(Level::Ti, "timed out".into());
        assert_eq!(
            err.to_string(),
            "Ti elevation ran but its outcome is unknown, so the machine may have changed: timed out"
        );
    }
}
