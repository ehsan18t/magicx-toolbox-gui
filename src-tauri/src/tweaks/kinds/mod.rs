//! The `EffectKind` contract (spec §5): one trait every address kind implements, so the engine
//! (a later task) can treat Registry/RegistryKey/Service/Task/Hosts/Firewall uniformly and mock
//! any of them in tests. Task 5 implemented `Setting::Registry` and `Setting::RegistryKey` (see
//! [`registry`]); this task adds `Setting::Service` and `Setting::Task` (see [`service`] and
//! [`task`]); Task 7 adds Hosts/Firewall.
//!
//! ## `ExecCx` and the broker seam (Task 14: System/TI routing is wired; placement decision below)
//!
//! `ExecCx` carries the effective elevation [`Level`] for one operation. Per spec invariant 24,
//! *reads* always run at the current, in-process level — there is no elevated read op in the
//! broker protocol (`services::elevation::broker::BrokerOp` has no `RegRead*` variant; a read
//! never needs a fresh child process), so a kind's `read` must never gate on `cx.level()`.
//! *Drives* (writes) do need to escalate for `Level::System`/`Level::Ti`: "User/Admin in-process,
//! System/TI in fresh children."
//!
//! **Placement:** each kind's own `drive` here is unchanged and still rejects `System`/`Ti` with
//! [`Error::UnsupportedLevel`] when called directly — that stays true and is still worth pinning
//! (the in-process kind never silently escalates on its own). The routing decision lives one layer
//! up, in `engine::AllKinds::drive` (`tweaks/engine/mod.rs`): for `Level::User`/`Level::Admin` it
//! delegates here exactly as before; for `Level::System`/`Level::Ti` it never reaches this file's
//! `drive` at all — it instead translates the `Setting`/`Value` into `BrokerOp`(s) (`to_broker_op`/
//! `to_broker_ops` in `registry.rs`/`service.rs`/`task.rs`) and submits them through
//! `services::elevation::run_ops` in one child. This keeps `EffectKind`'s signature and every
//! kind's own in-process behavior untouched, and keeps the translation colocated with the address
//! shape it understands (registry/service/task each know their own `Setting` variant's fields).
//! Hosts/Firewall have no corresponding `BrokerOp` in this build, so `AllKinds` still falls through
//! to their in-process `drive` at `System`/`Ti` too, which correctly still rejects it.

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

    /// `cx`'s elevation level has no routing at all for this `Setting` (see the module docs
    /// above) -- e.g. a field-addressed registry write, or a Hosts/Firewall effect, at
    /// `System`/`Ti`, neither of which this build's broker translation covers.
    #[error("{0:?} elevation is not yet routed by this build")]
    UnsupportedLevel(Level),

    /// Could not ACQUIRE the declared elevation level at all (spec §9, ADR-0005 as amended;
    /// invariant 24): the TI service would not start, `SeDebugPrivilege` was denied, winlogon was
    /// not found, or the elevated child failed to spawn or respond. Environmental, not a
    /// mis-declared level -- distinct from [`Error::AccessDenied`], which (for a routed System/TI
    /// drive) means the child WAS acquired and ran, but the operation itself was still denied.
    #[error("could not acquire {0:?} elevation: {1}")]
    CouldNotAcquireElevation(Level, String),

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

/// `Level::User`/`Level::Admin` run in-process here; `System`/`Ti` are routed through the
/// elevation broker one layer up, by `engine::AllKinds::drive` -- this kind's own `drive` (called
/// directly, bypassing that routing) still rejects them itself (see the module docs above).
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec §9/ADR-0005 (amended), invariant 24: "couldn't acquire the level" and "acquired but
    /// access-denied" are two DISTINCT typed failures, never collapsed into one generic error.
    /// `CouldNotAcquireElevation` is the environmental case (TI service unstartable, winlogon
    /// absent, ...); `AccessDenied` (pre-existing) is reused for "acquired the child, but the
    /// operation itself was still denied" -- distinguishable by variant, never by string-matching.
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
}
