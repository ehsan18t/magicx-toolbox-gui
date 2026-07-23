//! Shared-claims record: the runtime realization of ADR-0006 (spec §8.6). Several tweaks may
//! legitimately drive one address to one corpus-declared value (spec §6.5's `shared` block); this
//! module refcounts that sharing so the address is captured once, driven once, and restored once —
//! exactly at the true last release — no matter how many tweaks claim and release it in what order.
//!
//! **Persistence**: one engine-level, atomically-written, machine-stamped JSON file
//! (`shared_claims.json`) directly under the snapshots root — a single `shared_id -> record` map,
//! per spec §8.6's literal shape, not one-file-per-entry like [`super::snapshot`]. A claims record
//! is only ever useful as a whole (the engine needs "is anything claimed right now" cheaply), and
//! unlike a snapshot history there is nothing to keep once a shared id's last release verifies —
//! the entry is removed outright, so one small file is the natural fit.
//!
//! **Corrupt/wrong-schema is a hard error, not a skip.** [`super::snapshot`]'s `classify` treats an
//! unreadable *history* entry as merely unavailable as a restore target — there are always other
//! entries, and nothing is lost by ignoring one. Here there is exactly one copy of each captured
//! original; silently treating a corrupt file as "no claims" would let a first-claim re-capture
//! whatever the live value has drifted to as a fabricated "original," permanently losing the real
//! one (invariant 2, invariant 17). So parse failure or a schema mismatch surfaces as
//! [`ClaimsError::Corrupt`] and every operation refuses rather than guesses.
//!
//! **Wrong machine is different and safe to treat as empty**: mirrors the snapshot store's stance
//! (a copied-elsewhere `snapshots/` directory). A record stamped for another machine describes
//! claims that were never actually driven *here*, so there is no real original to protect — this
//! machine's live value has never been touched by any claim, and reading it fresh for a first claim
//! is correct, not a guess. A subsequent write on this machine naturally replaces the foreign
//! content; there is nothing on this machine's actual system state that record could ever have
//! restored.
//!
//! **Lock assumption (caller-held, not this module's job)**: every public method here does its own
//! read-modify-write of the whole file with no cached in-memory state between calls, which is the
//! shape a later task's claims-record lock can wrap soundly (spec §8.7: "claim ops serialized behind
//! the claims-record lock"). This build does not add that lock itself — callers must serialize their
//! own claim/release calls for now, exactly as the task brief specifies.

use crate::tweaks::kinds::{EffectKind, Error as KindError, ExecCx};
use crate::tweaks::model::{Setting, SharedDef, SharedId, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const SCHEMA_VERSION: u32 = 1;
const CLAIMS_FILE: &str = "shared_claims.json";

/// Distinguishes a genuinely new capture from a verified no-op (controller decision: the engine
/// needs to tell these apart, e.g. to decide whether to log "now enforced" vs "already enforced").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimOutcome {
    /// This was the first claimant: the live original was captured and the address driven.
    Captured,
    /// Another claimant already held this address; verified the driven value still holds.
    AlreadyHeld,
}

/// The result of one release (spec §8.6). `StillHeld` is INFO, never a failure — the releasing
/// tweak's own outcome is unaffected by other tweaks still claiming the address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseOutcome {
    /// Other claimants remain; the address was left alone.
    StillHeld(Vec<String>),
    /// This was the last claimant: the captured original was driven back, verified.
    RestoredOriginal,
}

#[derive(Debug, thiserror::Error)]
pub enum ClaimsError {
    #[error("I/O error in shared-claims record: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not determine the executable directory")]
    ExeDir,

    /// Unparseable JSON, or a `schema_version` this build does not recognize — never guessed past,
    /// see the module docs for why this is a hard error here (unlike snapshot history entries).
    #[error("shared-claims record is corrupt or from an incompatible schema version")]
    Corrupt,

    /// A read/drive through the injected [`EffectKind`] failed.
    #[error(transparent)]
    Kind(#[from] KindError),

    /// A drive succeeded without error but the read-back did not match — did-it-work (invariant 2).
    #[error("shared '{shared_id}' drove to {expected:?} but read back {actual:?}")]
    VerifyMismatch {
        shared_id: String,
        expected: Value,
        actual: Value,
    },

    /// `release` was called for a claimant that is not actually a current holder of `shared_id` —
    /// a caller bug, surfaced rather than silently accepted as a no-op (invariant 2).
    #[error("'{claimant}' does not currently hold shared claim '{shared_id}'")]
    NotHeld { shared_id: String, claimant: String },
}

/// One shared address's persisted claim state (spec §8.6): the captured original, the address
/// itself (so `release` — which takes only a [`SharedId`], never a [`SharedDef`] — can still drive
/// back to it), and the current claimant set. `claimants` preserves claim order (a `Vec`, not a
/// set) purely for a stable, readable "held by" message; membership is checked linearly, which is
/// fine at the tiny N a shared address realistically has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ClaimRecord {
    setting: Setting,
    original: Value,
    claimants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaimsFile {
    schema_version: u32,
    machine_guid: Option<String>,
    records: BTreeMap<String, ClaimRecord>,
}

/// Refcounted claims store (spec §8.6, ADR-0006). See the module docs for persistence shape, the
/// corrupt-vs-wrong-machine distinction, and the caller-held-lock assumption.
#[derive(Debug, Clone)]
pub struct ClaimsStore {
    root: PathBuf,
    machine_guid: Option<String>,
}

impl ClaimsStore {
    /// Opens a store rooted at `root` (the same snapshots root [`super::snapshot::SnapshotStore`]
    /// uses — ADR-0006), stamping every write with `machine_guid`. Tests pass a temp dir and a
    /// synthetic guid; production uses [`Self::open_default`].
    pub fn open(root: PathBuf, machine_guid: Option<String>) -> Self {
        Self { root, machine_guid }
    }

    /// Production root: the same portable `snapshots/` directory next to the executable that
    /// [`super::snapshot::SnapshotStore::open_default`] uses, stamped with the real machine guid.
    pub fn open_default() -> Result<Self, ClaimsError> {
        let exe = std::env::current_exe()?;
        let dir = exe.parent().ok_or(ClaimsError::ExeDir)?;
        Ok(Self::open(
            dir.join("snapshots"),
            crate::services::system_info_service::machine_guid(),
        ))
    }

    fn file_path(&self) -> PathBuf {
        self.root.join(CLAIMS_FILE)
    }

    /// Loads the current record map. Missing file = no claims yet (`Ok(empty)`); a
    /// machine-mismatched file is treated the same way (see module docs); a file that exists but
    /// fails to parse, or carries a schema version this build does not know, is a hard
    /// [`ClaimsError::Corrupt`] — never silently treated as "no claims" (that would risk a
    /// first-claim re-capturing drifted state as a fabricated original).
    fn load(&self) -> Result<BTreeMap<String, ClaimRecord>, ClaimsError> {
        let path = self.file_path();
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let bytes = fs::read(&path)?;
        let file: ClaimsFile = serde_json::from_slice(&bytes).map_err(|_| ClaimsError::Corrupt)?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(ClaimsError::Corrupt);
        }
        if let (Some(file_guid), Some(current)) =
            (file.machine_guid.as_deref(), self.machine_guid.as_deref())
        {
            if file_guid != current {
                log::warn!(
                    "shared-claims record was captured on a different machine (MachineGuid {file_guid} != {current}); \
                     ignoring its content rather than restoring a foreign original onto this machine"
                );
                return Ok(BTreeMap::new());
            }
        }
        Ok(file.records)
    }

    /// Atomic whole-file rewrite (temp file in the same directory, fsynced, then renamed onto the
    /// final path) — mirrors the pattern in [`super::snapshot`]; kept as a private copy here rather
    /// than reaching into that module's private helpers, per this task's instruction not to touch
    /// snapshot.rs. A consolidation carry-forward: the two atomic-write bodies are now duplicated
    /// across `snapshot.rs` and this file.
    fn save(&self, records: BTreeMap<String, ClaimRecord>) -> Result<(), ClaimsError> {
        fs::create_dir_all(&self.root)?;
        let file = ClaimsFile {
            schema_version: SCHEMA_VERSION,
            machine_guid: self.machine_guid.clone(),
            records,
        };
        let json = serde_json::to_vec_pretty(&file).expect("ClaimsFile always serializes");
        let mut tmp = tempfile::NamedTempFile::new_in(&self.root)?;
        tmp.write_all(&json)?;
        tmp.as_file().sync_all()?;
        tmp.persist(self.file_path())
            .map_err(|e| ClaimsError::Io(e.error))?;
        Ok(())
    }

    /// Claims `shared` on behalf of `claimant` (spec §8.6).
    ///
    /// First claimant: reads the live value through `kinds` (the captured original), persists it
    /// with the claimant *before* driving (so a crash between persist and drive leaves a correct,
    /// recoverable record — see below), drives to `shared.value`, then verifies the read-back. A
    /// failed drive/verify here still leaves the just-persisted record in place: the true original
    /// is the valuable, hard-to-recover data, and rolling it back on failure would force a retry to
    /// re-read a system that may now be in a partially-mutated state, capturing the wrong "original".
    ///
    /// Later claimants: a verified no-op — the address is already driven, so this never drives
    /// again; it only confirms the value still holds and adds the claimant.
    pub fn claim(
        &self,
        shared: &SharedDef,
        claimant: &str,
        kinds: &dyn EffectKind,
        cx: &ExecCx,
    ) -> Result<ClaimOutcome, ClaimsError> {
        let mut records = self.load()?;
        let key = shared.id.0.clone();

        match records.get_mut(&key) {
            None => {
                let original = kinds.read(&shared.setting, cx)?;
                records.insert(
                    key.clone(),
                    ClaimRecord {
                        setting: shared.setting.clone(),
                        original,
                        claimants: vec![claimant.to_string()],
                    },
                );
                self.save(records)?;

                kinds.drive(&shared.setting, &shared.value, cx)?;
                let after = kinds.read(&shared.setting, cx)?;
                if after != shared.value {
                    return Err(ClaimsError::VerifyMismatch {
                        shared_id: key,
                        expected: shared.value.clone(),
                        actual: after,
                    });
                }
                log::info!("shared '{key}': first claim by '{claimant}' captured original and drove to the shared value");
                Ok(ClaimOutcome::Captured)
            }
            Some(record) => {
                let already = record.claimants.iter().any(|c| c == claimant);
                if !already {
                    record.claimants.push(claimant.to_string());
                    self.save(records)?;
                }

                let current = kinds.read(&shared.setting, cx)?;
                if current != shared.value {
                    return Err(ClaimsError::VerifyMismatch {
                        shared_id: key,
                        expected: shared.value.clone(),
                        actual: current,
                    });
                }
                log::debug!("shared '{key}': claim by '{claimant}' is a verified no-op");
                Ok(ClaimOutcome::AlreadyHeld)
            }
        }
    }

    /// Releases `claimant`'s hold on `shared_id` (spec §8.6). Removes the claimant; if others
    /// remain, the address is left alone and this reports `StillHeld` — info, not failure. If this
    /// was the last claimant, drives back to the captured original **unconditionally** (grill Q4:
    /// a revert means "give me the captured state back," so any external drift is overwritten, not
    /// read-and-skipped), verifies, and only then removes the record. A failed restore here returns
    /// before any save, so the on-disk record — original and claimant list both — is left exactly
    /// as it was before this call: nothing is lost, and a retry (by the same claimant) behaves
    /// identically (ADR-0002-consistent: never delete on an unverified restore).
    pub fn release(
        &self,
        shared_id: &SharedId,
        claimant: &str,
        kinds: &dyn EffectKind,
        cx: &ExecCx,
    ) -> Result<ReleaseOutcome, ClaimsError> {
        let mut records = self.load()?;
        let key = shared_id.0.clone();

        let Some(record) = records.get_mut(&key) else {
            return Err(ClaimsError::NotHeld {
                shared_id: key,
                claimant: claimant.to_string(),
            });
        };
        let before = record.claimants.len();
        record.claimants.retain(|c| c != claimant);
        if record.claimants.len() == before {
            return Err(ClaimsError::NotHeld {
                shared_id: key,
                claimant: claimant.to_string(),
            });
        }

        if !record.claimants.is_empty() {
            let holders = record.claimants.clone();
            self.save(records)?;
            log::info!("shared '{key}': '{claimant}' released; still held by {holders:?}");
            return Ok(ReleaseOutcome::StillHeld(holders));
        }

        // Last release: unconditional restore. `records` here holds the claimant already removed
        // in memory but NOT YET persisted -- if the drive/verify below fails, we return without
        // calling `self.save`, so the durable file still shows the pre-release state untouched.
        let setting = record.setting.clone();
        let original = record.original.clone();
        kinds.drive(&setting, &original, cx)?;
        let after = kinds.read(&setting, cx)?;
        if after != original {
            return Err(ClaimsError::VerifyMismatch {
                shared_id: key,
                expected: original,
                actual: after,
            });
        }

        records.remove(&key);
        self.save(records)?;
        log::info!("shared '{key}': last release by '{claimant}' restored the captured original");
        Ok(ReleaseOutcome::RestoredOriginal)
    }

    /// Current claimants of `shared_id`, in claim order; empty if unclaimed. Read-only and
    /// best-effort: an unreadable/corrupt record conservatively reports "no holders" (logged) rather
    /// than propagating an error this method's signature has no room for — the safety-critical
    /// paths are `claim`/`release`, which do return `Result`.
    pub fn holders(&self, shared_id: &SharedId) -> Vec<String> {
        match self.load() {
            Ok(records) => records
                .get(&shared_id.0)
                .map(|r| r.claimants.clone())
                .unwrap_or_default(),
            Err(e) => {
                log::warn!("shared-claims record unreadable while querying holders: {e}");
                Vec::new()
            }
        }
    }

    /// Whether any claimant currently holds `shared_id`.
    pub fn is_claimed(&self, shared_id: &SharedId) -> bool {
        !self.holders(shared_id).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::{Hive, Level, RegAddr, RegType, TypedRegValue};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Mutex;

    fn shared_def() -> SharedDef {
        SharedDef {
            id: SharedId("telemetry_off".into()),
            setting: crate::tweaks::model::Setting::Registry(RegAddr {
                hive: Hive::Hklm,
                path: "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection".into(),
                name: "AllowTelemetry".into(),
                ty: RegType::Dword,
                field: None,
            }),
            value: Value::Reg(TypedRegValue::Dword(0)),
        }
    }

    fn original_value() -> Value {
        Value::Reg(TypedRegValue::Dword(1))
    }

    /// In-memory `EffectKind` mock (brief requirement: the claim/release drive/read side goes
    /// through an injected mock, never a real kind). Tracks drive-call count and can be told to
    /// fail every subsequent drive, for the failed-restore test.
    struct MockKind {
        current: Mutex<Value>,
        drive_calls: AtomicU32,
        fail_drives: AtomicBool,
    }

    impl MockKind {
        fn new(initial: Value) -> Self {
            Self {
                current: Mutex::new(initial),
                drive_calls: AtomicU32::new(0),
                fail_drives: AtomicBool::new(false),
            }
        }

        fn live(&self) -> Value {
            self.current.lock().unwrap().clone()
        }

        fn set_fail(&self, fail: bool) {
            self.fail_drives.store(fail, Ordering::SeqCst);
        }
    }

    impl EffectKind for MockKind {
        fn read(
            &self,
            _s: &crate::tweaks::model::Setting,
            _cx: &ExecCx,
        ) -> Result<Value, KindError> {
            Ok(self.current.lock().unwrap().clone())
        }

        fn drive(
            &self,
            _s: &crate::tweaks::model::Setting,
            target: &Value,
            _cx: &ExecCx,
        ) -> Result<(), KindError> {
            self.drive_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_drives.load(Ordering::SeqCst) {
                return Err(KindError::Backend("mock drive failure".into()));
            }
            *self.current.lock().unwrap() = target.clone();
            Ok(())
        }
    }

    fn store(dir: &Path) -> ClaimsStore {
        ClaimsStore::open(dir.to_path_buf(), Some("test-guid".into()))
    }

    fn cx() -> ExecCx {
        ExecCx::new(Level::User)
    }

    #[test]
    fn first_claim_captures_once_and_drives() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        let outcome = s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();

        assert_eq!(outcome, ClaimOutcome::Captured);
        assert_eq!(
            mock.live(),
            shared.value,
            "first claim must drive to the shared value"
        );
        assert_eq!(mock.drive_calls.load(Ordering::SeqCst), 1);
        assert_eq!(s.holders(&shared.id), vec!["tweak_a".to_string()]);
        assert!(s.is_claimed(&shared.id));
    }

    #[test]
    fn second_claim_is_verified_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();
        let outcome = s.claim(&shared, "tweak_b", &mock, &cx()).unwrap();

        assert_eq!(outcome, ClaimOutcome::AlreadyHeld);
        assert_eq!(
            mock.drive_calls.load(Ordering::SeqCst),
            1,
            "a later claim must verify, never drive again"
        );
        let mut holders = s.holders(&shared.id);
        holders.sort();
        assert_eq!(holders, vec!["tweak_a".to_string(), "tweak_b".to_string()]);
    }

    #[test]
    fn early_release_leaves_value_reports_holders() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();
        s.claim(&shared, "tweak_b", &mock, &cx()).unwrap();

        let outcome = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap();

        assert_eq!(
            outcome,
            ReleaseOutcome::StillHeld(vec!["tweak_b".to_string()])
        );
        assert_eq!(
            mock.live(),
            shared.value,
            "the value must be left alone while a claimant remains"
        );
        assert!(s.is_claimed(&shared.id));
    }

    #[test]
    fn last_release_restores_original_unconditionally() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();
        // Simulate external drift after the claim -- the release must overwrite this, not
        // read-and-skip on it (grill Q4).
        *mock.current.lock().unwrap() = Value::Reg(TypedRegValue::Dword(999));

        let outcome = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap();

        assert_eq!(outcome, ReleaseOutcome::RestoredOriginal);
        assert_eq!(
            mock.live(),
            original_value(),
            "unconditional restore must overwrite the drift"
        );
        assert_eq!(
            mock.drive_calls.load(Ordering::SeqCst),
            2,
            "release must actually call drive (1 from the claim + 1 from this release) -- \
             asserting only the end state would also pass a buggy 'skip drive if current already \
             looks fine' implementation"
        );
        assert!(!s.is_claimed(&shared.id));
        assert!(s.holders(&shared.id).is_empty());
    }

    #[test]
    fn failed_restore_keeps_record_needs_attention() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();
        mock.set_fail(true);

        let err = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap_err();

        assert!(matches!(err, ClaimsError::Kind(_)));
        assert!(
            s.is_claimed(&shared.id),
            "the record must be kept, not deleted, on a failed restore"
        );
        assert_eq!(s.holders(&shared.id), vec!["tweak_a".to_string()]);
    }

    /// A corrupt/unparseable claims file must never fall through to "no claims" -- doing so would
    /// let `claim` read the already-driven live value and persist it as a fresh "original,"
    /// permanently losing the real one. Both `claim` and `release` must refuse outright.
    #[test]
    fn corrupt_claims_file_is_hard_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(CLAIMS_FILE), b"{ not valid json").unwrap();

        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        let claim_err = s.claim(&shared, "tweak_a", &mock, &cx()).unwrap_err();
        assert!(matches!(claim_err, ClaimsError::Corrupt));

        let release_err = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap_err();
        assert!(matches!(release_err, ClaimsError::Corrupt));

        assert_eq!(
            mock.drive_calls.load(Ordering::SeqCst),
            0,
            "a corrupt record must never let claim/release re-capture or re-drive the live value"
        );
    }

    /// A record stamped for a different machine must be treated as no record at all (mirrors
    /// `SnapshotStore`'s WrongMachine stance) -- this machine must never drive a foreign record's
    /// captured "original" onto its own live value.
    #[test]
    fn foreign_machine_guid_treated_as_no_record() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = shared_def();

        // A *valid*, well-formed record -- just stamped for a different machine (e.g. a portable
        // snapshots/ directory copied from elsewhere).
        let mut records = BTreeMap::new();
        records.insert(
            shared.id.0.clone(),
            ClaimRecord {
                setting: shared.setting.clone(),
                original: Value::Reg(TypedRegValue::Dword(777)), // the foreign "original"
                claimants: vec!["foreign_tweak".to_string()],
            },
        );
        let foreign = ClaimsFile {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("foreign-machine".into()),
            records,
        };
        std::fs::write(
            tmp.path().join(CLAIMS_FILE),
            serde_json::to_vec_pretty(&foreign).unwrap(),
        )
        .unwrap();

        let s = ClaimsStore::open(tmp.path().to_path_buf(), Some("here-machine".into()));
        let mock = MockKind::new(original_value()); // this machine's real live value, never 777

        assert!(
            s.holders(&shared.id).is_empty(),
            "a foreign-machine record must not surface its claimant on this machine"
        );
        assert!(!s.is_claimed(&shared.id));

        // A release "by" the foreign claimant must find no record here, and above all must never
        // drive the foreign original (777) onto this machine's live value.
        let err = s
            .release(&shared.id, "foreign_tweak", &mock, &cx())
            .unwrap_err();
        assert!(matches!(err, ClaimsError::NotHeld { .. }));
        assert_eq!(
            mock.drive_calls.load(Ordering::SeqCst),
            0,
            "the foreign record's captured original must never be driven onto this machine"
        );
        assert_eq!(mock.live(), original_value());

        // A fresh claim on this machine proceeds normally, capturing THIS machine's live value --
        // not the foreign record's -- exactly as if no record existed.
        let outcome = s.claim(&shared, "tweak_here", &mock, &cx()).unwrap();
        assert_eq!(outcome, ClaimOutcome::Captured);
        assert_eq!(mock.live(), shared.value);
        assert_eq!(s.holders(&shared.id), vec!["tweak_here".to_string()]);
    }

    /// `release` must refuse -- never drive, never treat as a benign no-op -- both when the shared
    /// id has no record at all and when a record exists but this specific claimant never held it.
    #[test]
    fn release_by_non_holder_is_not_held() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        let err = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap_err();
        assert!(matches!(err, ClaimsError::NotHeld { .. }));

        s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();
        let err = s.release(&shared.id, "tweak_b", &mock, &cx()).unwrap_err();
        assert!(matches!(err, ClaimsError::NotHeld { .. }));

        assert_eq!(
            mock.drive_calls.load(Ordering::SeqCst),
            1,
            "neither NotHeld case may drive -- only the earlier legitimate claim's single drive"
        );
        assert!(
            s.is_claimed(&shared.id),
            "the real holder's claim must be untouched by either failed release attempt"
        );
        assert_eq!(s.holders(&shared.id), vec!["tweak_a".to_string()]);
    }

    /// The crown-jewel property test (brief): for every legal claim/release interleaving among N
    /// claimants sharing one address, the captured original is restored exactly once, at the true
    /// last release, and the live value tracks `shared.value` throughout the claimed window.
    ///
    /// Deterministic by construction: rather than sampling with a PRNG, this exhaustively
    /// enumerates every valid interleaving of N claim/release pairs (each claimant must claim
    /// before it releases; otherwise the two event streams interleave freely) via backtracking, so
    /// "many interleavings" means literally all of them for the chosen N, not one arbitrarily
    /// chosen order.
    #[test]
    fn interleaving_property() {
        #[derive(Clone, Copy, Debug)]
        enum Event {
            Claim(usize),
            Release(usize),
        }

        fn enumerate(n: usize) -> Vec<Vec<Event>> {
            fn backtrack(
                n: usize,
                claimed: &mut [bool],
                held: &mut [bool],
                cur: &mut Vec<Event>,
                out: &mut Vec<Vec<Event>>,
            ) {
                if cur.len() == 2 * n {
                    out.push(cur.clone());
                    return;
                }
                for i in 0..n {
                    if !claimed[i] {
                        claimed[i] = true;
                        held[i] = true;
                        cur.push(Event::Claim(i));
                        backtrack(n, claimed, held, cur, out);
                        cur.pop();
                        held[i] = false;
                        claimed[i] = false;
                    }
                }
                for i in 0..n {
                    if held[i] {
                        held[i] = false;
                        cur.push(Event::Release(i));
                        backtrack(n, claimed, held, cur, out);
                        cur.pop();
                        held[i] = true;
                    }
                }
            }
            let mut out = Vec::new();
            backtrack(
                n,
                &mut vec![false; n],
                &mut vec![false; n],
                &mut Vec::new(),
                &mut out,
            );
            out
        }

        let n = 3;
        let sequences = enumerate(n);
        // (2n)! / 2^n valid interleavings for independent claim/release pairs -- 90 for n=3.
        assert_eq!(sequences.len(), 90);

        for seq in &sequences {
            let tmp = tempfile::tempdir().unwrap();
            let s = store(tmp.path());
            let shared = shared_def();
            let mock = MockKind::new(original_value());
            let mut held_count = 0usize;
            let mut captures = 0usize;
            let mut restores = 0usize;

            for ev in seq {
                match *ev {
                    Event::Claim(i) => {
                        let outcome = s
                            .claim(&shared, &format!("tweak_{i}"), &mock, &cx())
                            .unwrap();
                        held_count += 1;
                        let expected = if held_count == 1 {
                            captures += 1;
                            ClaimOutcome::Captured
                        } else {
                            ClaimOutcome::AlreadyHeld
                        };
                        assert_eq!(outcome, expected, "sequence {seq:?}");
                        assert_eq!(mock.live(), shared.value, "sequence {seq:?}");
                    }
                    Event::Release(i) => {
                        let outcome = s
                            .release(&shared.id, &format!("tweak_{i}"), &mock, &cx())
                            .unwrap();
                        held_count -= 1;
                        if held_count == 0 {
                            assert_eq!(
                                outcome,
                                ReleaseOutcome::RestoredOriginal,
                                "sequence {seq:?}"
                            );
                            restores += 1;
                            assert_eq!(mock.live(), original_value(), "sequence {seq:?}");
                        } else {
                            assert!(
                                matches!(outcome, ReleaseOutcome::StillHeld(_)),
                                "sequence {seq:?}"
                            );
                            assert_eq!(mock.live(), shared.value, "sequence {seq:?}");
                        }
                    }
                }
            }
            // A sequence may contain several disjoint claim/release "windows" (e.g. 0 and 1 fully
            // release before 2 ever claims) -- each window gets exactly one capture and exactly one
            // restore, at its own true last release (already enforced step-by-step above). Across
            // the whole sequence that means captures and restores must match; a mismatch would mean
            // some window's original was restored more than once, or not at all.
            assert_eq!(
                captures, restores,
                "every captured window must be restored exactly once, for {seq:?}"
            );
        }
    }
}
