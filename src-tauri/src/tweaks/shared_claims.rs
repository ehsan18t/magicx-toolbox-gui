//! Shared-claims record (ADR-0006): refcounts a shared address so it is captured once and restored
//! at the true last release. One atomically written file per machine, `shared_claims.<guid>.json`,
//! so a portable `snapshots/` folder never overwrites another machine's originals.
//! Unreadable or unknown-schema is [`ClaimsError::Corrupt`], never "no claims": each original
//! exists once, and a first claim would re-capture drifted state as a fabricated original.

use crate::tweaks::kinds::{EffectKind, Error as KindError, ExecCx};
use crate::tweaks::model::{effective_level, Level, Setting, SharedDef, SharedId, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

// v1 records carry no `restore_level`; they load with it absent. Any higher version is Corrupt.
const SCHEMA_VERSION: u32 = 2;
const LEGACY_CLAIMS_FILE: &str = "shared_claims.json";

/// Serializes each whole-file read-modify-write. `lifecycle::lock_tweak` is per tweak, so two
/// claimants of one address would otherwise interleave and drop an update or a captured original.
static CLAIMS_LOCK: Mutex<()> = Mutex::new(());

/// Takes [`CLAIMS_LOCK`], recovering from poisoning.
///
/// A poisoned lock means an earlier claim operation panicked. The file is written atomically
/// (temp + rename), so the record on disk is still a whole, valid document either way, and the next
/// operation re-reads it from scratch. Refusing to proceed would strand every later claim for the
/// life of the process without protecting anything.
fn lock_claims() -> MutexGuard<'static, ()> {
    CLAIMS_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A genuinely new capture vs. a verified no-op (e.g. to log "now enforced" vs "already enforced").
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
    /// This was the last claimant: the captured original was driven back, verified, at this level.
    RestoredOriginal(Level),
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

/// `setting` is stored because `release` gets only a [`SharedId`]; `claimants` is in claim order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ClaimRecord {
    setting: Setting,
    original: Value,
    /// Highest level any claimant routed at; the restore never drives below it. `None`: a v1 record.
    #[serde(default)]
    restore_level: Option<Level>,
    claimants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaimsFile {
    schema_version: u32,
    machine_guid: Option<String>,
    records: BTreeMap<String, ClaimRecord>,
}

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
        match &self.machine_guid {
            Some(guid) => self.root.join(format!("shared_claims.{guid}.json")),
            None => self.root.join(LEGACY_CLAIMS_FILE),
        }
    }

    /// The unsuffixed file, read only while this machine has no file of its own.
    fn legacy_path(&self) -> Option<PathBuf> {
        self.machine_guid
            .as_ref()
            .map(|_| self.root.join(LEGACY_CLAIMS_FILE))
    }

    fn read_file(path: &Path) -> Result<Option<ClaimsFile>, ClaimsError> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let file: ClaimsFile = serde_json::from_slice(&bytes).map_err(|_| ClaimsError::Corrupt)?;
        if !(1..=SCHEMA_VERSION).contains(&file.schema_version) {
            return Err(ClaimsError::Corrupt);
        }
        Ok(Some(file))
    }

    fn is_ours(&self, file: &ClaimsFile) -> bool {
        match (file.machine_guid.as_deref(), self.machine_guid.as_deref()) {
            (Some(stamped), Some(current)) => stamped == current,
            _ => true,
        }
    }

    fn load(&self) -> Result<BTreeMap<String, ClaimRecord>, ClaimsError> {
        if let Some(file) = Self::read_file(&self.file_path())? {
            if !self.is_ours(&file) {
                return Err(ClaimsError::Corrupt);
            }
            return Ok(file.records);
        }
        let Some(legacy) = self.legacy_path() else {
            return Ok(BTreeMap::new());
        };
        match Self::read_file(&legacy)? {
            Some(file) if self.is_ours(&file) => Ok(file.records),
            Some(_) => {
                log::info!("shared-claims: ignoring another machine's legacy claims file");
                Ok(BTreeMap::new())
            }
            None => Ok(BTreeMap::new()),
        }
    }

    fn save(&self, records: BTreeMap<String, ClaimRecord>) -> Result<(), ClaimsError> {
        fs::create_dir_all(&self.root)?;
        let file = ClaimsFile {
            schema_version: SCHEMA_VERSION,
            machine_guid: self.machine_guid.clone(),
            records,
        };
        let json = serde_json::to_vec_pretty(&file).expect("ClaimsFile always serializes");
        super::snapshot::durable_write(&self.root, &self.file_path(), &json, true)?;
        // This machine's legacy file is now folded in; a failed delete is harmless because the
        // per-machine file shadows it from here on.
        if let Some(legacy) = self.legacy_path() {
            if matches!(Self::read_file(&legacy), Ok(Some(ref f)) if self.is_ours(f)) {
                if let Err(e) = fs::remove_file(&legacy) {
                    log::warn!("shared-claims: could not remove the migrated legacy file: {e}");
                }
            }
        }
        Ok(())
    }

    /// First claim persists the original before driving and keeps it on a failed drive (a retry would
    /// capture half-mutated state). Later claims verify only, never drive; every claim raises
    /// `restore_level` to its own route.
    pub fn claim(
        &self,
        shared: &SharedDef,
        claimant: &str,
        kinds: &dyn EffectKind,
        cx: &ExecCx,
    ) -> Result<ClaimOutcome, ClaimsError> {
        let _guard = lock_claims();
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
                        restore_level: Some(cx.level()),
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
                if record.restore_level.is_none() {
                    log::warn!(
                        "shared '{key}': v1 record has no restore level; stamping the claimant's level {:?}",
                        cx.level()
                    );
                }
                let raised = Some(effective_level(cx.level(), record.restore_level));
                if !already || record.restore_level != raised {
                    if !already {
                        record.claimants.push(claimant.to_string());
                    }
                    record.restore_level = raised;
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

    /// Last release drives the original back unconditionally, verifies, then drops the record; a failed
    /// restore saves nothing (ADR-0002). Runs at `max(restore_level, cx)`: `cx` alone is too low after
    /// a Ti capture, and it counts because the current corpus may route the block higher (ADR-0007).
    pub fn release(
        &self,
        shared_id: &SharedId,
        claimant: &str,
        kinds: &dyn EffectKind,
        cx: &ExecCx,
    ) -> Result<ReleaseOutcome, ClaimsError> {
        let _guard = lock_claims();
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
        if record.restore_level.is_none() {
            log::warn!(
                "shared '{key}': v1 record has no restore level; restoring at the releaser's level {:?}",
                cx.level()
            );
        }
        let cx = &ExecCx::new(effective_level(cx.level(), record.restore_level));
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
        Ok(ReleaseOutcome::RestoredOriginal(cx.level()))
    }

    /// Current claimants of `shared_id`, in claim order. An unreadable record is an error, never
    /// "no holders": callers decide whether to release from this answer.
    pub fn holders(&self, shared_id: &SharedId) -> Result<Vec<String>, ClaimsError> {
        let _guard = lock_claims();
        Ok(self
            .load()?
            .get(&shared_id.0)
            .map(|r| r.claimants.clone())
            .unwrap_or_default())
    }

    // Takes no lock itself: `holders` does, and the mutex is not reentrant.
    pub fn is_claimed(&self, shared_id: &SharedId) -> Result<bool, ClaimsError> {
        Ok(!self.holders(shared_id)?.is_empty())
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

    /// In-memory `EffectKind` mock, never a real kind. Tracks drive-call count and can be told to
    /// fail every subsequent drive, for the failed-restore test.
    struct MockKind {
        current: Mutex<Value>,
        drive_calls: AtomicU32,
        drive_levels: Mutex<Vec<Level>>,
        fail_drives: AtomicBool,
    }

    impl MockKind {
        fn new(initial: Value) -> Self {
            Self {
                current: Mutex::new(initial),
                drive_calls: AtomicU32::new(0),
                drive_levels: Mutex::new(Vec::new()),
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
            cx: &ExecCx,
        ) -> Result<(), KindError> {
            self.drive_calls.fetch_add(1, Ordering::SeqCst);
            self.drive_levels.lock().unwrap().push(cx.level());
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
        assert_eq!(s.holders(&shared.id).unwrap(), vec!["tweak_a".to_string()]);
        assert!(s.is_claimed(&shared.id).unwrap());
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
        let mut holders = s.holders(&shared.id).unwrap();
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
        assert!(s.is_claimed(&shared.id).unwrap());
    }

    #[test]
    fn last_release_restores_original_unconditionally() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &cx()).unwrap();
        // Simulate external drift after the claim -- the release must overwrite this, not
        // read-and-skip on it.
        *mock.current.lock().unwrap() = Value::Reg(TypedRegValue::Dword(999));

        let outcome = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap();

        assert!(matches!(outcome, ReleaseOutcome::RestoredOriginal(_)));
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
        assert!(!s.is_claimed(&shared.id).unwrap());
        assert!(s.holders(&shared.id).unwrap().is_empty());
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
            s.is_claimed(&shared.id).unwrap(),
            "the record must be kept, not deleted, on a failed restore"
        );
        assert_eq!(s.holders(&shared.id).unwrap(), vec!["tweak_a".to_string()]);
    }

    /// A corrupt/unparseable claims file must never fall through to "no claims" -- doing so would
    /// let `claim` read the already-driven live value and persist it as a fresh "original,"
    /// permanently losing the real one. Both `claim` and `release` must refuse outright.
    #[test]
    fn corrupt_claims_file_is_hard_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(LEGACY_CLAIMS_FILE), b"{ not valid json").unwrap();

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

    /// Another machine's record is invisible here, never driven here, and never overwritten.
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
                restore_level: Some(Level::Admin),
                claimants: vec!["foreign_tweak".to_string()],
            },
        );
        let foreign = ClaimsFile {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("foreign-machine".into()),
            records,
        };
        std::fs::write(
            tmp.path().join(LEGACY_CLAIMS_FILE),
            serde_json::to_vec_pretty(&foreign).unwrap(),
        )
        .unwrap();

        let s = ClaimsStore::open(tmp.path().to_path_buf(), Some("here-machine".into()));
        let mock = MockKind::new(original_value()); // this machine's real live value, never 777

        assert!(
            s.holders(&shared.id).unwrap().is_empty(),
            "a foreign-machine record must not surface its claimant on this machine"
        );
        assert!(!s.is_claimed(&shared.id).unwrap());

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
        assert_eq!(
            s.holders(&shared.id).unwrap(),
            vec!["tweak_here".to_string()]
        );

        // ADR-0006: the other machine's originals survive this machine's writes.
        let kept: ClaimsFile =
            serde_json::from_slice(&std::fs::read(tmp.path().join(LEGACY_CLAIMS_FILE)).unwrap())
                .unwrap();
        assert_eq!(kept.machine_guid.as_deref(), Some("foreign-machine"));
        assert_eq!(kept.records[&shared.id.0].claimants, vec!["foreign_tweak"]);
        let back_home = ClaimsStore::open(tmp.path().to_path_buf(), Some("foreign-machine".into()));
        assert_eq!(
            back_home.holders(&shared.id).unwrap(),
            vec!["foreign_tweak".to_string()]
        );
    }

    #[test]
    fn machines_sharing_a_folder_keep_separate_records() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = shared_def();
        let a = ClaimsStore::open(tmp.path().to_path_buf(), Some("machine-a".into()));
        let b = ClaimsStore::open(tmp.path().to_path_buf(), Some("machine-b".into()));
        let mock_a = MockKind::new(original_value());
        let mock_b = MockKind::new(original_value());

        a.claim(&shared, "tweak_a", &mock_a, &cx()).unwrap();
        b.claim(&shared, "tweak_b", &mock_b, &cx()).unwrap();

        assert_eq!(a.holders(&shared.id).unwrap(), vec!["tweak_a".to_string()]);
        let outcome = a.release(&shared.id, "tweak_a", &mock_a, &cx()).unwrap();
        assert!(matches!(outcome, ReleaseOutcome::RestoredOriginal(_)));
        assert_eq!(mock_a.live(), original_value());
        assert_eq!(b.holders(&shared.id).unwrap(), vec!["tweak_b".to_string()]);
    }

    /// A file that cannot be read is an error to every caller, never an empty holder list.
    #[test]
    fn holders_of_a_corrupt_record_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        std::fs::write(s.file_path(), b"{ not valid json").unwrap();

        assert!(matches!(
            s.holders(&shared_def().id),
            Err(ClaimsError::Corrupt)
        ));
        assert!(s.is_claimed(&shared_def().id).is_err());
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
            s.is_claimed(&shared.id).unwrap(),
            "the real holder's claim must be untouched by either failed release attempt"
        );
        assert_eq!(s.holders(&shared.id).unwrap(), vec!["tweak_a".to_string()]);
    }

    /// The crown-jewel property test: for every legal claim/release interleaving among N
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
                                ReleaseOutcome::RestoredOriginal(Level::User),
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

    /// Two tweaks may apply at once (`lifecycle` locks per tweak id, and its own test asserts
    /// different ids overlap), so two claimants of one address race the read-modify-write of the
    /// single claims file. Without [`CLAIMS_LOCK`] the losing writer's claimant is silently dropped,
    /// which leaks a refcount and, on the first claim, can lose the captured original outright.
    #[test]
    fn concurrent_claims_never_lose_a_holder() {
        const CLAIMANTS: usize = 8;

        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        std::thread::scope(|scope| {
            for i in 0..CLAIMANTS {
                let (s, shared, mock) = (&s, &shared, &mock);
                scope.spawn(move || {
                    s.claim(shared, &format!("tweak_{i}"), mock, &cx())
                        .expect("a concurrent claim must not fail");
                });
            }
        });

        let holders = s.holders(&shared.id).unwrap();
        assert_eq!(
            holders.len(),
            CLAIMANTS,
            "every concurrent claimant must survive the read-modify-write: {holders:?}"
        );
        assert_eq!(
            mock.drive_calls.load(Ordering::SeqCst),
            1,
            "only the first claimant drives; the rest are verified no-ops"
        );
    }

    /// The mirror of the claim race: every claimant releasing concurrently must still reach a
    /// single, verified last release that restores the captured original exactly once.
    #[test]
    fn concurrent_releases_restore_the_original_exactly_once() {
        const CLAIMANTS: usize = 8;

        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        for i in 0..CLAIMANTS {
            s.claim(&shared, &format!("tweak_{i}"), &mock, &cx())
                .expect("claim");
        }

        std::thread::scope(|scope| {
            for i in 0..CLAIMANTS {
                let (s, shared, mock) = (&s, &shared, &mock);
                scope.spawn(move || {
                    s.release(&shared.id, &format!("tweak_{i}"), mock, &cx())
                        .expect("a concurrent release must not fail");
                });
            }
        });

        assert!(
            !s.is_claimed(&shared.id).unwrap(),
            "the last release must remove the record"
        );
        assert_eq!(
            mock.live(),
            original_value(),
            "the captured original must be restored"
        );
    }

    fn at(level: Level) -> ExecCx {
        ExecCx::new(level)
    }

    fn write_raw(dir: &Path, json: &serde_json::Value) {
        std::fs::write(
            dir.join(LEGACY_CLAIMS_FILE),
            serde_json::to_vec_pretty(json).unwrap(),
        )
        .unwrap();
    }

    /// A Ti-protected original captured at Ti must go back at Ti even when an admin-floor claimant
    /// releases last: at Admin the drive is access-denied on every retry.
    #[test]
    fn last_release_drives_at_the_recorded_level_not_the_releasers() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &at(Level::Ti)).unwrap();
        s.claim(&shared, "tweak_b", &mock, &at(Level::Admin))
            .unwrap();
        s.release(&shared.id, "tweak_a", &mock, &at(Level::Ti))
            .unwrap();
        let outcome = s
            .release(&shared.id, "tweak_b", &mock, &at(Level::Admin))
            .unwrap();

        assert_eq!(outcome, ReleaseOutcome::RestoredOriginal(Level::Ti));
        assert_eq!(
            *mock.drive_levels.lock().unwrap(),
            vec![Level::Ti, Level::Ti]
        );
    }

    #[test]
    fn a_higher_later_claim_raises_the_recorded_level() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let shared = shared_def();
        let mock = MockKind::new(original_value());

        s.claim(&shared, "tweak_a", &mock, &at(Level::Admin))
            .unwrap();
        s.claim(&shared, "tweak_b", &mock, &at(Level::Ti)).unwrap();
        s.release(&shared.id, "tweak_b", &mock, &at(Level::Ti))
            .unwrap();
        s.release(&shared.id, "tweak_a", &mock, &at(Level::Admin))
            .unwrap();

        assert_eq!(
            *mock.drive_levels.lock().unwrap(),
            vec![Level::Admin, Level::Ti],
            "the claim drove at Admin; the restore must use the highest level any claimant routed"
        );
    }

    /// A v1 record carries no level: it loads, restores at the releaser's level, and the next write
    /// stamps the file as the current schema.
    #[test]
    fn a_v1_file_loads_and_is_rewritten_as_current() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = shared_def();
        write_raw(
            tmp.path(),
            &serde_json::json!({
                "schema_version": 1,
                "machine_guid": "test-guid",
                "records": { "telemetry_off": {
                    "setting": shared.setting,
                    "original": original_value(),
                    "claimants": ["tweak_a", "tweak_b"],
                }},
            }),
        );
        let s = store(tmp.path());
        let mock = MockKind::new(shared.value.clone());

        s.release(&shared.id, "tweak_a", &mock, &at(Level::Admin))
            .unwrap();
        let rewritten: serde_json::Value =
            serde_json::from_slice(&std::fs::read(s.file_path()).unwrap()).unwrap();
        assert_eq!(rewritten["schema_version"], SCHEMA_VERSION);
        assert!(
            !tmp.path().join(LEGACY_CLAIMS_FILE).exists(),
            "this machine's legacy file is folded into its own file"
        );
        assert_eq!(SCHEMA_VERSION, 2);
        assert_eq!(s.holders(&shared.id).unwrap(), vec!["tweak_b".to_string()]);

        let outcome = s
            .release(&shared.id, "tweak_b", &mock, &at(Level::Admin))
            .unwrap();
        assert!(matches!(outcome, ReleaseOutcome::RestoredOriginal(_)));
        assert_eq!(mock.live(), original_value());
        assert_eq!(*mock.drive_levels.lock().unwrap(), vec![Level::Admin]);
    }

    /// A file from a newer build is refused, and above all never overwritten with this schema.
    #[test]
    fn a_newer_schema_file_is_refused_and_never_rewritten() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = shared_def();
        write_raw(
            tmp.path(),
            &serde_json::json!({
                "schema_version": SCHEMA_VERSION + 1,
                "machine_guid": "test-guid",
                "records": {},
            }),
        );
        let before = std::fs::read(tmp.path().join(LEGACY_CLAIMS_FILE)).unwrap();
        let s = store(tmp.path());
        let mock = MockKind::new(original_value());

        let claim_err = s.claim(&shared, "tweak_a", &mock, &cx()).unwrap_err();
        assert!(matches!(claim_err, ClaimsError::Corrupt));
        let release_err = s.release(&shared.id, "tweak_a", &mock, &cx()).unwrap_err();
        assert!(matches!(release_err, ClaimsError::Corrupt));

        assert_eq!(
            std::fs::read(tmp.path().join(LEGACY_CLAIMS_FILE)).unwrap(),
            before
        );
        assert_eq!(mock.drive_calls.load(Ordering::SeqCst), 0);
    }
}
