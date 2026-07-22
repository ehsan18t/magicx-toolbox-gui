//! Snapshot store: the persistence backbone the restore/rollback safety story depends on (spec
//! §8.2/§8.3/§11; ADR-0002). Pure storage — no Windows effect APIs, no elevation. One JSON file
//! per entry inside a per-tweak subdirectory: a single corrupt or huge history can never make
//! another tweak's history unreadable, and `discard`/dedup ever touch exactly the one file they
//! target instead of rewriting a shared blob.
//!
//! **Seq, never wall-clock** (invariant 6). The next seq is the max of what's actually on disk
//! (`scan_max_seq`, filenames only — corruption in one file's *content* can never block issuing a
//! seq) and a best-effort cache (`_seq.json`) that remembers the high-water mark even after a
//! dedup vacates the highest entry. Losing the cache costs a directory scan, never correctness.
//!
//! **Invalid entries are never deleted by this module** (ADR-0002). `classify` is the only
//! gatekeeper: it returns a verdict, `head`/`list` act on it, and `push`'s dedup only ever removes
//! an existing entry that itself `classify`s `Valid` against the caller's own corpus/machine/build
//! — a foreign-machine or otherwise-invalid entry that happens to parse and share the label is
//! left untouched. `discard` (explicit consent) and `consume` (caller-verified restore) are the
//! only other removal paths.

use crate::tweaks::model::{Corpus, EffectId, Value};
use crate::tweaks::validate::{option_unavailable, Milestone};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Bump on any breaking change to `Entry`'s shape; `classify` treats a mismatch as `WrongSchema`
/// rather than guessing an upgrade path (spec §8.3/§11, invariant 21).
const SCHEMA_VERSION: u32 = 1;

const SEQ_CACHE_FILE: &str = "_seq.json";

/// Monotonic per-tweak sequence number (spec §8.2) — never derived from wall-clock. Orders a
/// tweak's history; `head`/`consume`/`discard`/`mark_completed` address entries by this alone.
/// **Identity, never defaultable**: a missing `seq` must fail `Entry` deserialization outright
/// (see `Entry`'s field-level `#[serde(default)]` note) — silently defaulting it would let content
/// decide which file a write targets, exactly the bug `mark_completed`/`rewrite_entry` now guard
/// against by taking `seq` as a trusted parameter instead of reading it back off the entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Seq(pub u64);

/// What one entry captured (spec §8.3). Authored-option captures store the reference only —
/// restore re-derives from the *current* corpus (ADR-0007). Unauthored states (System Default,
/// drift) store the full value map because they exist nowhere else. Never deduped against each
/// other; only a repeated `OptionRef` dedups (spec §8.2, invariant 6). Identity data, not
/// defaultable — see `Entry`'s field-level `#[serde(default)]` note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Captured {
    OptionRef(String),
    Values(BTreeMap<EffectId, Value>),
}

/// One row of the WAL action journal (spec §8.1, invariant 5): `intended` is persisted before any
/// mutation; `completed` flips durably only once that action has actually run. Mandatory, not
/// defaultable, for the same reason as `Entry`'s identity fields: `action_id` is itself an
/// identity key `mark_completed` matches on, and `intended`/`completed` are the WAL state the
/// whole durability guarantee is about — a missing field here must be `Corrupt`, never a guess.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalRow {
    pub action_id: EffectId,
    pub intended: bool,
    pub completed: bool,
}

/// One persisted snapshot entry (spec §8.3).
///
/// `#[serde(default)]` is applied **only** to `schema_version`/`machine_guid` — spec §11's
/// forward-compat intent is that a genuinely old external file (predating one of these fields, or
/// a future additive field on either) still loads, matching the proven pattern in
/// `services/backup/storage.rs`'s `TweakSnapshot`. It is deliberately NOT a blanket
/// container-level default: `seq`, `tweak_id`, `captured`, and `journal` are identity data, and
/// silently defaulting any of them would be worse than refusing to parse. Concretely, a blanket
/// default once let a `seq`-less entry deserialize as `Seq(0)`, and `mark_completed` trusted that
/// content-derived value to pick which file to rewrite — exactly the "content decides which file
/// gets written" failure `mark_completed`/`rewrite_entry` are built to rule out. An entry missing
/// any of the four mandatory fields is correctly `Corrupt`, never guessed at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(default)]
    pub schema_version: u32,
    /// `None` when the machine's identity couldn't be read at capture time — never guessed, and
    /// `classify` never compares a known guid against an unknown one either (mirrors the proven
    /// "identity unknown, skip the check" handling in `services::backup::storage`).
    #[serde(default)]
    pub machine_guid: Option<String>,
    pub tweak_id: String,
    pub seq: Seq,
    /// Display metadata only — never used for ordering or comparison (spec §8.2: clocks skew).
    pub timestamp: String,
    pub captured: Captured,
    pub journal: Vec<JournalRow>,
}

/// What a caller pushes (spec §8.1 step 2). The store stamps `schema_version`, `machine_guid`,
/// `tweak_id`, `seq`, and `timestamp` itself — callers can never mis-stamp an entry.
#[derive(Debug, Clone)]
pub struct NewEntry {
    pub captured: Captured,
    pub journal: Vec<JournalRow>,
}

/// Raw bytes read from one entry file, tagged with the `Seq` recovered from its filename — kept
/// separate from parsing so a corrupt payload never hides *which* entry is corrupt (spec §8.3).
#[derive(Debug, Clone)]
pub struct RawEntry {
    pub seq: Seq,
    pub bytes: Vec<u8>,
}

/// Why an entry cannot be a restore target (spec §8.3, ADR-0002). Never a deletion trigger by
/// itself — `discard` is the only caller-driven removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum InvalidReason {
    /// Unparseable JSON, or JSON missing/mistyping `schema_version` itself.
    Corrupt,
    /// Parseable, but `schema_version` doesn't match this build's `SCHEMA_VERSION`.
    WrongSchema,
    /// `machine_guid` doesn't match the running machine (only checked when both sides are known).
    WrongMachine,
    /// The entry's tweak, or (for an `OptionRef`) its option label, is no longer in the corpus.
    DanglingRef,
    /// The referenced option's tweak is scoped out of the running Windows build.
    TargetUnavailable,
}

/// The result of classifying one entry (spec §8.3, ADR-2). Never a deletion trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EntryValidity {
    Valid,
    Invalid(InvalidReason),
}

/// One `list` row for UI surfacing (spec §8.3): full detail when the payload parsed, `None` fields
/// when it was too corrupt to parse — it still carries a `seq` and a reason so the UI can offer
/// `discard`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntrySummary {
    pub seq: Seq,
    pub validity: EntryValidity,
    pub timestamp: Option<String>,
    pub captured: Option<Captured>,
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("I/O error in snapshot store: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not determine the executable directory")]
    ExeDir,

    #[error("snapshot seq {seq:?} for tweak '{tweak_id}' already exists")]
    SeqCollision { tweak_id: String, seq: Seq },

    #[error("no snapshot entry seq {seq:?} for tweak '{tweak_id}'")]
    NotFound { tweak_id: String, seq: Seq },

    #[error("snapshot entry for tweak '{tweak_id}' seq {seq:?} is corrupt")]
    Corrupt { tweak_id: String, seq: Seq },

    #[error("action '{action_id}' is not in the journal for tweak '{tweak_id}' seq {seq:?}")]
    UnknownJournalAction {
        tweak_id: String,
        seq: Seq,
        action_id: String,
    },
}

/// Portable, per-tweak, atomic-write snapshot history (spec §11). One subdirectory per tweak-id
/// under `root`, one JSON file per entry named by its `Seq`.
#[derive(Debug, Clone)]
pub struct SnapshotStore {
    root: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct SeqCache {
    last: u64,
}

impl SnapshotStore {
    /// Opens a store rooted at `root` (created lazily, per-tweak, on first `push`). Tests always
    /// pass a temp dir here; production uses `open_default`.
    pub fn open(root: PathBuf) -> Self {
        Self { root }
    }

    /// Production root: the portable `snapshots/` directory next to the executable (spec §11).
    pub fn open_default() -> Result<Self, SnapshotError> {
        let exe = std::env::current_exe()?;
        let dir = exe.parent().ok_or(SnapshotError::ExeDir)?;
        Ok(Self::open(dir.join("snapshots")))
    }

    fn tweak_dir(&self, tweak_id: &str) -> PathBuf {
        self.root.join(tweak_id)
    }

    /// Create-new push (spec §8.1/§11, invariant 6): allocates the next monotonic seq, dedups an
    /// `OptionRef` re-capture to the head, and writes atomically. A lost create-new race surfaces
    /// as `SeqCollision`, never a silent overwrite.
    ///
    /// `corpus`/`machine_guid`/`running_build` are the same classification context `head`/`list`
    /// take (spec §8.3, ADR-0002): dedup only ever removes an existing entry that `classify`s
    /// `Valid` for *this* machine/build/corpus — a foreign-machine, dangling, or otherwise-invalid
    /// entry that happens to parse and share the label is left on disk untouched, exactly like any
    /// other invalid entry, released only by `discard`. `machine_guid` also stamps the new entry
    /// (the store no longer reads the OS registry itself — the caller reads it once and passes the
    /// same value everywhere, matching `head`/`list`/`classify`'s own contract).
    pub fn push(
        &self,
        tweak_id: &str,
        new_entry: NewEntry,
        corpus: &Corpus,
        machine_guid: Option<&str>,
        running_build: u32,
    ) -> Result<Seq, SnapshotError> {
        let dir = self.tweak_dir(tweak_id);
        fs::create_dir_all(&dir)?;

        // Dedup (spec §8.2, invariant 6): a re-captured OptionRef moves to head — remove the old
        // entry with the same label first, but ONLY if it currently classifies `Valid` (ADR-0002):
        // an entry that merely parses and shares the label — foreign machine, dangling, wrong
        // schema, scoped out — is never treated as "the prior capture of this option" by this
        // store; it stays on disk, surfaced via `list`, released only by `discard`. Values dumps
        // never dedup.
        if let Captured::OptionRef(label) = &new_entry.captured {
            for raw in read_raw_entries(&dir)? {
                let (validity, parsed) =
                    classify_and_parse(&raw, corpus, machine_guid, running_build);
                let Some(existing) = parsed else {
                    continue; // Corrupt/WrongSchema: no well-typed entry to compare against
                };
                if validity != EntryValidity::Valid {
                    continue;
                }
                if matches!(&existing.captured, Captured::OptionRef(l) if l == label) {
                    fs::remove_file(entry_path(&dir, raw.seq))?;
                    log::debug!(
                        "tweak '{tweak_id}': dedup removed seq {:?} for option '{label}'",
                        raw.seq
                    );
                }
            }
        }

        let seq = next_seq(&dir)?;
        let entry = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: machine_guid.map(str::to_string),
            tweak_id: tweak_id.to_string(),
            seq,
            timestamp: chrono::Local::now().to_rfc3339(),
            captured: new_entry.captured,
            journal: new_entry.journal,
        };
        write_entry_create_new(&dir, &entry)?;
        // Best-effort: losing this hint only costs a directory scan on the next push, never
        // correctness — `scan_max_seq` always recovers the true high-water mark from disk.
        if let Err(e) = write_seq_cache(&dir, seq) {
            log::warn!("tweak '{tweak_id}': failed to persist seq cache: {e}");
        }
        log::debug!("tweak '{tweak_id}': pushed snapshot entry seq {seq:?}");
        Ok(seq)
    }

    /// The highest-seq *valid* entry (spec §8.3) — invalid entries are skipped, never guessed at.
    pub fn head(
        &self,
        tweak_id: &str,
        corpus: &Corpus,
        machine_guid: Option<&str>,
        running_build: u32,
    ) -> Result<Option<Entry>, SnapshotError> {
        let dir = self.tweak_dir(tweak_id);
        let mut raws = read_raw_entries(&dir)?;
        raws.sort_by_key(|r| std::cmp::Reverse(r.seq));
        for raw in &raws {
            let (validity, parsed) = classify_and_parse(raw, corpus, machine_guid, running_build);
            if validity == EntryValidity::Valid {
                return Ok(parsed);
            }
        }
        Ok(None)
    }

    /// Every entry, valid and invalid, oldest first — for UI surfacing (spec §8.3, ADR-0002).
    pub fn list(
        &self,
        tweak_id: &str,
        corpus: &Corpus,
        machine_guid: Option<&str>,
        running_build: u32,
    ) -> Result<Vec<EntrySummary>, SnapshotError> {
        let dir = self.tweak_dir(tweak_id);
        let mut raws = read_raw_entries(&dir)?;
        raws.sort_by_key(|r| r.seq);
        Ok(raws
            .iter()
            .map(|raw| {
                let (validity, parsed) =
                    classify_and_parse(raw, corpus, machine_guid, running_build);
                EntrySummary {
                    seq: raw.seq,
                    validity,
                    timestamp: parsed.as_ref().map(|e| e.timestamp.clone()),
                    captured: parsed.map(|e| e.captured),
                }
            })
            .collect())
    }

    /// Removes the entry after a verified restore (caller-enforced, ADR-0002). Mechanically
    /// identical to `discard`; kept as a separate method because the two release paths carry
    /// different caller obligations the store itself cannot check.
    pub fn consume(&self, tweak_id: &str, seq: Seq) -> Result<(), SnapshotError> {
        remove_entry(&self.tweak_dir(tweak_id), tweak_id, seq)
    }

    /// Removes the entry on explicit user consent (ADR-0002) — the only release for an entry
    /// `classify` marked invalid.
    pub fn discard(&self, tweak_id: &str, seq: Seq) -> Result<(), SnapshotError> {
        remove_entry(&self.tweak_dir(tweak_id), tweak_id, seq)
    }

    /// Durably flips one journal row's `completed` bit (spec §8.1, invariant 5): an atomic
    /// rewrite of the whole entry, fsynced, so the mark survives a crash immediately after.
    ///
    /// `seq` — the parameter, i.e. the filename this method read the entry from — is the ONLY
    /// trusted identity for where the rewrite lands; the entry's own (content-derived) `seq` field
    /// is never used to pick a write path, and is overwritten to match before the rewrite so the
    /// file's content can't drift from its own filename. Content must never decide which file gets
    /// written.
    pub fn mark_completed(
        &self,
        tweak_id: &str,
        seq: Seq,
        action_id: &EffectId,
    ) -> Result<(), SnapshotError> {
        let dir = self.tweak_dir(tweak_id);
        let path = entry_path(&dir, seq);
        let bytes = fs::read(&path).map_err(|e| io_to_not_found(e, tweak_id, seq))?;
        let mut entry: Entry =
            serde_json::from_slice(&bytes).map_err(|_| SnapshotError::Corrupt {
                tweak_id: tweak_id.to_string(),
                seq,
            })?;

        let row = entry
            .journal
            .iter_mut()
            .find(|r| r.action_id == *action_id)
            .ok_or_else(|| SnapshotError::UnknownJournalAction {
                tweak_id: tweak_id.to_string(),
                seq,
                action_id: action_id.to_string(),
            })?;
        row.completed = true;
        entry.seq = seq; // self-heal: the file's own content must agree with its trusted filename

        rewrite_entry(&dir, seq, &entry)?;
        log::debug!("tweak '{tweak_id}': marked action '{action_id}' completed at seq {seq:?}");
        Ok(())
    }
}

/// Classifies one raw entry against the current corpus/machine/build (spec §8.3, ADR-0002). Pure:
/// never deletes, never mutates — the only gatekeeper `head`/`list` defer to for what counts as a
/// usable restore target.
pub fn classify(
    raw: &RawEntry,
    corpus: &Corpus,
    machine_guid: Option<&str>,
    running_build: u32,
) -> EntryValidity {
    classify_and_parse(raw, corpus, machine_guid, running_build).0
}

/// `classify`'s implementation, threading the parsed `Entry` through so `head`/`list` don't parse
/// twice. `None` only for `Corrupt`/`WrongSchema`, where no well-typed `Entry` exists at all.
fn classify_and_parse(
    raw: &RawEntry,
    corpus: &Corpus,
    machine_guid: Option<&str>,
    running_build: u32,
) -> (EntryValidity, Option<Entry>) {
    let json: serde_json::Value = match serde_json::from_slice(&raw.bytes) {
        Ok(v) => v,
        Err(_) => return (EntryValidity::Invalid(InvalidReason::Corrupt), None),
    };
    match json
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(v) if v == u64::from(SCHEMA_VERSION) => {}
        Some(_) => return (EntryValidity::Invalid(InvalidReason::WrongSchema), None),
        None => return (EntryValidity::Invalid(InvalidReason::Corrupt), None),
    }
    let entry: Entry = match serde_json::from_value(json) {
        Ok(e) => e,
        Err(_) => return (EntryValidity::Invalid(InvalidReason::Corrupt), None),
    };

    // Only compared when both sides are known — an unreadable guid on either side means
    // "identity unknown", never guessed into a false mismatch.
    if let (Some(entry_guid), Some(current)) = (entry.machine_guid.as_deref(), machine_guid) {
        if entry_guid != current {
            return (
                EntryValidity::Invalid(InvalidReason::WrongMachine),
                Some(entry),
            );
        }
    }

    let Some(tweak) = corpus.tweaks.iter().find(|t| t.id == entry.tweak_id) else {
        return (
            EntryValidity::Invalid(InvalidReason::DanglingRef),
            Some(entry),
        );
    };
    if let Captured::OptionRef(label) = &entry.captured {
        let Some(matched) = tweak.options.iter().find(|o| &o.label.0 == label) else {
            return (
                EntryValidity::Invalid(InvalidReason::DanglingRef),
                Some(entry),
            );
        };
        let milestone = Milestone {
            build: running_build,
        };
        // Real option-level applicability (spec §8.3/§8.4), not just the tweak's own `windows:`
        // scope: an option can be unavailable purely because its *own* per-value scope excludes
        // this milestone even when the tweak itself admits it (reuses validate.rs's
        // `applicable_surface`/`applicable_value`, which also folds in the tweak-level scope).
        if option_unavailable(tweak, matched, &milestone) {
            return (
                EntryValidity::Invalid(InvalidReason::TargetUnavailable),
                Some(entry),
            );
        }
    }
    (EntryValidity::Valid, Some(entry))
}

fn entry_path(dir: &Path, seq: Seq) -> PathBuf {
    dir.join(format!("{:020}.json", seq.0))
}

/// Filenames only — never reads content, so one unrelated unreadable file can never block
/// allocating the next seq (spec §8.2: robust to a partially-written history).
fn scan_max_seq(dir: &Path) -> Result<u64, SnapshotError> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut max = 0u64;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if let Some(seq) = entry
            .file_name()
            .to_str()
            .and_then(|n| n.strip_suffix(".json"))
            .and_then(|stem| stem.parse::<u64>().ok())
        {
            max = max.max(seq);
        }
    }
    Ok(max)
}

fn read_seq_cache(dir: &Path) -> Option<u64> {
    let bytes = fs::read(dir.join(SEQ_CACHE_FILE)).ok()?;
    serde_json::from_slice::<SeqCache>(&bytes)
        .ok()
        .map(|c| c.last)
}

fn write_seq_cache(dir: &Path, seq: Seq) -> Result<(), SnapshotError> {
    let json = serde_json::to_vec(&SeqCache { last: seq.0 }).expect("SeqCache always serializes");
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(&json)?;
    tmp.as_file().sync_all()?;
    tmp.persist(dir.join(SEQ_CACHE_FILE))
        .map_err(|e| SnapshotError::Io(e.error))?;
    Ok(())
}

/// Next monotonic seq (spec §8.2, invariant 6): max of what's actually on disk and the persisted
/// cache, so a dedup-vacated head can never reissue an old number and a lost/corrupt cache
/// self-heals from the directory. Never wall-clock derived.
fn next_seq(dir: &Path) -> Result<Seq, SnapshotError> {
    let scan_max = scan_max_seq(dir)?;
    let cached = read_seq_cache(dir).unwrap_or(0);
    Ok(Seq(scan_max.max(cached) + 1))
}

/// All entry files currently on disk, full content. IO failure here is a genuine failure (never
/// "no history") and must propagate — a read that cannot distinguish corrupt-vs-IO-failure must
/// not silently treat an IO failure as "no snapshot".
fn read_raw_entries(dir: &Path) -> Result<Vec<RawEntry>, SnapshotError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(stem) = name.to_str().and_then(|n| n.strip_suffix(".json")) else {
            continue;
        };
        let Ok(seq) = stem.parse::<u64>() else {
            continue; // not a seq-named entry file (e.g. the seq cache) — not history
        };
        let bytes = fs::read(entry.path())?;
        out.push(RawEntry {
            seq: Seq(seq),
            bytes,
        });
    }
    Ok(out)
}

/// Atomic create-new write (spec §8.1/§11, invariant 6): temp file in the same directory, fsynced,
/// then `persist_noclobber` — Windows `MoveFileExW` *without* `MOVEFILE_REPLACE_EXISTING`, so a
/// seq collision is a loud `Err` and whatever was already at that seq is left untouched.
fn write_entry_create_new(dir: &Path, entry: &Entry) -> Result<(), SnapshotError> {
    let json = serde_json::to_vec_pretty(entry).expect("Entry always serializes");
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(&json)?;
    tmp.as_file().sync_all()?;
    tmp.persist_noclobber(entry_path(dir, entry.seq))
        .map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                SnapshotError::SeqCollision {
                    tweak_id: entry.tweak_id.clone(),
                    seq: entry.seq,
                }
            } else {
                SnapshotError::Io(e.error)
            }
        })?;
    Ok(())
}

/// Atomic in-place rewrite of an existing entry (spec §8.1, invariant 5) — used only by
/// `mark_completed`, where overwriting the current seq's file is exactly the intent.
///
/// `seq` is taken as an explicit, caller-trusted parameter and is the ONLY thing that decides the
/// write path — never `entry.seq`. Content must never decide which file gets written: an entry
/// whose own `seq` field is missing/wrong (e.g. a stray `#[serde(default)]` letting it read back
/// as `Seq(0)`) must not silently redirect a rewrite to some other file.
fn rewrite_entry(dir: &Path, seq: Seq, entry: &Entry) -> Result<(), SnapshotError> {
    let json = serde_json::to_vec_pretty(entry).expect("Entry always serializes");
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(&json)?;
    tmp.as_file().sync_all()?;
    tmp.persist(entry_path(dir, seq))
        .map_err(|e| SnapshotError::Io(e.error))?;
    Ok(())
}

fn remove_entry(dir: &Path, tweak_id: &str, seq: Seq) -> Result<(), SnapshotError> {
    fs::remove_file(entry_path(dir, seq)).map_err(|e| io_to_not_found(e, tweak_id, seq))
}

fn io_to_not_found(e: std::io::Error, tweak_id: &str, seq: Seq) -> SnapshotError {
    if e.kind() == std::io::ErrorKind::NotFound {
        SnapshotError::NotFound {
            tweak_id: tweak_id.to_string(),
            seq,
        }
    } else {
        SnapshotError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::{
        BuildExpr, Effect, EffectDef, Hive, KeyAddr, Level, Opt, OptLabel, OptValue, RiskLevel,
        ScopedValue, Setting, Tweak, WindowsScope,
    };

    /// A stand-in for the engine's real machine guid — tests just need *a* consistent value across
    /// calls in the same test, never the real OS registry (see `push`'s doc: it no longer reads it).
    const GUID: &str = "test-guid";

    fn store(dir: &Path) -> SnapshotStore {
        SnapshotStore::open(dir.to_path_buf())
    }

    /// The one effect every test `Opt` covers, so `tweak()`'s surface is never trivially empty —
    /// `option_unavailable`'s "empty surface ⇒ unavailable" branch would otherwise make every
    /// option unavailable regardless of scope, which isn't what most tests are exercising.
    fn effect_def(id: &str) -> EffectDef {
        EffectDef {
            id: EffectId(id.to_string()),
            kind: Effect::Setting(Setting::RegistryKey(KeyAddr {
                hive: Hive::Hkcu,
                path: "Software\\MagicXTest".to_string(),
            })),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    /// An option covering `effect_def("eff1")`, with `value_windows` as that value's own
    /// per-option-value scope (spec §6.6's third scoping level) — `None` is fully available.
    fn opt_scoped(label: &str, value_windows: Option<WindowsScope>) -> Opt {
        let mut values = BTreeMap::new();
        values.insert(
            EffectId("eff1".into()),
            OptValue::Set(ScopedValue {
                value: Value::Present(true),
                windows: value_windows,
            }),
        );
        Opt {
            label: OptLabel(label.to_string()),
            values,
        }
    }

    fn opt(label: &str) -> Opt {
        opt_scoped(label, None)
    }

    fn tweak(id: &str, windows: Option<WindowsScope>, options: Vec<Opt>) -> Tweak {
        Tweak {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            category: "misc".to_string(),
            info: None,
            warning: None,
            requires_reboot: false,
            risk_level: RiskLevel::Low,
            elevation: Level::User,
            reversible: true,
            surface: vec![effect_def("eff1")],
            options,
            windows,
        }
    }

    fn corpus(tweaks: Vec<Tweak>) -> Corpus {
        Corpus {
            categories: Vec::new(),
            tweaks,
            shared: Vec::new(),
        }
    }

    /// For pushes that never touch dedup (`Captured::Values`) — the classification context is
    /// unused on that path, so an empty corpus/no-guid/build-0 is a valid stand-in.
    fn empty_corpus() -> Corpus {
        corpus(Vec::new())
    }

    fn values_entry() -> NewEntry {
        NewEntry {
            captured: Captured::Values(BTreeMap::new()),
            journal: Vec::new(),
        }
    }

    fn option_ref_entry(label: &str) -> NewEntry {
        NewEntry {
            captured: Captured::OptionRef(label.to_string()),
            journal: Vec::new(),
        }
    }

    fn read_entry_direct(dir: &Path, seq: Seq) -> Entry {
        let bytes = fs::read(entry_path(dir, seq)).expect("entry file exists");
        serde_json::from_slice(&bytes).expect("entry parses")
    }

    #[test]
    fn push_is_create_new() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();

        let entry_a = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: None,
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t1".into(),
            captured: Captured::Values(BTreeMap::new()),
            journal: Vec::new(),
        };
        write_entry_create_new(&dir, &entry_a).expect("first write succeeds");

        let mut entry_b = entry_a.clone();
        entry_b.timestamp = "t2-different".into();
        let err = write_entry_create_new(&dir, &entry_b).expect_err("seq collision must be loud");
        assert!(matches!(
            err,
            SnapshotError::SeqCollision { .. } | SnapshotError::Io(_)
        ));

        let on_disk = read_entry_direct(&dir, Seq(1));
        assert_eq!(on_disk.timestamp, "t1", "the first entry must stay intact");
    }

    #[test]
    fn dedup_moves_option_ref_to_head() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let c = corpus(vec![tweak("demo", None, vec![opt("A"), opt("B")])]);

        let seq_a1 = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 10240)
            .unwrap();
        let seq_b = s
            .push("demo", option_ref_entry("B"), &c, Some(GUID), 10240)
            .unwrap();
        let seq_a2 = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 10240)
            .unwrap();

        assert!(
            seq_a2 > seq_b && seq_b > seq_a1,
            "each push takes a fresh, higher seq"
        );

        let dir = tmp.path().join("demo");
        let raws = read_raw_entries(&dir).unwrap();
        assert_eq!(
            raws.len(),
            2,
            "the stale A entry is removed, B is untouched"
        );
        let mut seqs: Vec<u64> = raws.iter().map(|r| r.seq.0).collect();
        seqs.sort_unstable();
        assert_eq!(seqs, vec![seq_b.0, seq_a2.0]);

        let head_entry = read_entry_direct(&dir, seq_a2);
        assert_eq!(head_entry.captured, Captured::OptionRef("A".into()));
    }

    #[test]
    fn push_dedup_only_removes_a_currently_valid_matching_entry() {
        // ADR-0002 / invariant 21 (portable store): move `snapshots/` to another machine and
        // re-apply the same option. The old entry parses fine and its label matches, but it must
        // classify `WrongMachine` here — dedup must never delete it just because it parses and
        // shares a label. Only a currently-`Valid` match may be removed.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        let c = corpus(vec![tweak("demo", None, vec![opt("A")])]);

        let foreign = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("foreign-machine".into()),
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t0".into(),
            captured: Captured::OptionRef("A".into()),
            journal: Vec::new(),
        };
        write_entry_create_new(&dir, &foreign).unwrap();

        let s = store(tmp.path());
        let new_seq = s
            .push("demo", option_ref_entry("A"), &c, Some("here-guid"), 10240)
            .expect("push must succeed even though a foreign entry exists");

        let foreign_still_there = read_entry_direct(&dir, Seq(1));
        assert_eq!(
            foreign_still_there, foreign,
            "a foreign-machine entry must never be silently deleted by dedup"
        );

        let raws = read_raw_entries(&dir).unwrap();
        assert_eq!(
            raws.len(),
            2,
            "the new capture sits beside the untouched foreign entry, not deduped against it"
        );
        assert_ne!(new_seq, Seq(1));
        let new_entry = read_entry_direct(&dir, new_seq);
        assert_eq!(new_entry.captured, Captured::OptionRef("A".into()));
        assert_eq!(new_entry.machine_guid.as_deref(), Some("here-guid"));
    }

    #[test]
    fn dumps_never_dedup() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let c = empty_corpus();

        s.push("demo", values_entry(), &c, None, 0).unwrap();
        s.push("demo", values_entry(), &c, None, 0).unwrap();

        let dir = tmp.path().join("demo");
        let raws = read_raw_entries(&dir).unwrap();
        assert_eq!(
            raws.len(),
            2,
            "two Values dumps are both kept, never deduped"
        );
    }

    #[test]
    fn seq_is_monotonic_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let c = empty_corpus();

        let s1 = SnapshotStore::open(root.clone());
        let seq1 = s1.push("demo", values_entry(), &c, None, 0).unwrap();
        let seq2 = s1.push("demo", values_entry(), &c, None, 0).unwrap();
        drop(s1);

        let s2 = SnapshotStore::open(root);
        let seq3 = s2.push("demo", values_entry(), &c, None, 0).unwrap();

        assert!(
            seq3 > seq2 && seq2 > seq1,
            "seq keeps increasing across reopen, never derived from wall-clock"
        );
    }

    #[test]
    fn seq_recovers_when_cache_file_missing() {
        // Robustness (spec §8.2): losing the `_seq.json` hint must not reissue an old seq.
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let c = empty_corpus();
        let seq1 = s.push("demo", values_entry(), &c, None, 0).unwrap();

        fs::remove_file(tmp.path().join("demo").join(SEQ_CACHE_FILE)).unwrap();

        let seq2 = s.push("demo", values_entry(), &c, None, 0).unwrap();
        assert!(
            seq2 > seq1,
            "directory scan recovers the true high-water mark"
        );
    }

    #[test]
    fn journal_mark_survives_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let action = EffectId("act1".into());

        let s1 = SnapshotStore::open(root.clone());
        let seq = s1
            .push(
                "demo",
                NewEntry {
                    captured: Captured::Values(BTreeMap::new()),
                    journal: vec![JournalRow {
                        action_id: action.clone(),
                        intended: true,
                        completed: false,
                    }],
                },
                &empty_corpus(),
                None,
                0,
            )
            .unwrap();
        s1.mark_completed("demo", seq, &action).unwrap();
        drop(s1);

        let _s2 = SnapshotStore::open(root); // reopen to prove durability, not a different code path
        let entry = read_entry_direct(&tmp.path().join("demo"), seq);
        assert!(
            entry.journal[0].completed,
            "the completion mark must survive a store reopen"
        );
    }

    #[test]
    fn mark_completed_errors_on_unknown_action() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let seq = s
            .push("demo", values_entry(), &empty_corpus(), None, 0)
            .unwrap(); // no journal rows at all

        let err = s
            .mark_completed("demo", seq, &EffectId("nope".into()))
            .unwrap_err();
        assert!(matches!(err, SnapshotError::UnknownJournalAction { .. }));
    }

    #[test]
    fn mark_completed_refuses_an_entry_missing_seq_and_never_writes_seq_zero() {
        // Fix B regression: `seq` must be mandatory. A blanket `#[serde(default)]` on `Entry`
        // previously let a `seq`-less file deserialize as `Seq(0)`, and `rewrite_entry` trusted
        // that content-derived value as the write path — silently overwriting whatever lived at
        // seq 0. `seq` must now come only from the trusted caller parameter/filename, never from
        // content, and a missing `seq` field must refuse to parse at all.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        let action = EffectId("act1".into());

        // A legitimate, unrelated entry at seq 0 — the file that must stay untouched.
        let sentinel = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("g".into()),
            tweak_id: "demo".into(),
            seq: Seq(0),
            timestamp: "sentinel".into(),
            captured: Captured::Values(BTreeMap::new()),
            journal: Vec::new(),
        };
        write_entry_create_new(&dir, &sentinel).unwrap();

        // The real target, at seq 5, written with its "seq" field stripped out entirely.
        let target = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("g".into()),
            tweak_id: "demo".into(),
            seq: Seq(5),
            timestamp: "t5".into(),
            captured: Captured::Values(BTreeMap::new()),
            journal: vec![JournalRow {
                action_id: action.clone(),
                intended: true,
                completed: false,
            }],
        };
        let mut json = serde_json::to_value(&target).unwrap();
        json.as_object_mut().unwrap().remove("seq");
        fs::write(entry_path(&dir, Seq(5)), serde_json::to_vec(&json).unwrap()).unwrap();

        let s = store(tmp.path());
        let err = s.mark_completed("demo", Seq(5), &action).unwrap_err();
        assert!(
            matches!(err, SnapshotError::Corrupt { .. }),
            "a seq-less entry must refuse to deserialize, not silently default to Seq(0): got {err:?}"
        );

        let sentinel_after = read_entry_direct(&dir, Seq(0));
        assert_eq!(
            sentinel_after, sentinel,
            "mark_completed must never write to a file other than the trusted seq it read from"
        );
    }

    #[test]
    fn classify_matrix() {
        let guid_here = "guid-here";
        let t = tweak("demo", None, vec![opt("A")]);
        let c = corpus(vec![t]);

        let valid_entry = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some(guid_here.into()),
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t".into(),
            captured: Captured::OptionRef("A".into()),
            journal: Vec::new(),
        };

        // Valid.
        let raw = RawEntry {
            seq: Seq(1),
            bytes: serde_json::to_vec(&valid_entry).unwrap(),
        };
        assert_eq!(
            classify(&raw, &c, Some(guid_here), 10240),
            EntryValidity::Valid
        );

        // Corrupt: unparseable JSON.
        let raw = RawEntry {
            seq: Seq(2),
            bytes: b"{ not json".to_vec(),
        };
        assert_eq!(
            classify(&raw, &c, Some(guid_here), 10240),
            EntryValidity::Invalid(InvalidReason::Corrupt)
        );

        // WrongSchema.
        let mut wrong_schema = valid_entry.clone();
        wrong_schema.schema_version = SCHEMA_VERSION + 1;
        let raw = RawEntry {
            seq: Seq(3),
            bytes: serde_json::to_vec(&wrong_schema).unwrap(),
        };
        assert_eq!(
            classify(&raw, &c, Some(guid_here), 10240),
            EntryValidity::Invalid(InvalidReason::WrongSchema)
        );

        // WrongMachine: a foreign guid.
        let mut foreign = valid_entry.clone();
        foreign.machine_guid = Some("some-other-machine".into());
        let raw = RawEntry {
            seq: Seq(4),
            bytes: serde_json::to_vec(&foreign).unwrap(),
        };
        assert_eq!(
            classify(&raw, &c, Some(guid_here), 10240),
            EntryValidity::Invalid(InvalidReason::WrongMachine)
        );

        // DanglingRef: label absent from the corpus.
        let mut dangling = valid_entry.clone();
        dangling.captured = Captured::OptionRef("Ghost".into());
        let raw = RawEntry {
            seq: Seq(5),
            bytes: serde_json::to_vec(&dangling).unwrap(),
        };
        assert_eq!(
            classify(&raw, &c, Some(guid_here), 10240),
            EntryValidity::Invalid(InvalidReason::DanglingRef)
        );

        // TargetUnavailable: the tweak's own windows scope excludes the running build.
        let scoped_tweak = tweak(
            "scoped",
            Some(WindowsScope {
                products: None,
                build: Some(BuildExpr::Exact(26100)),
                revision: None,
            }),
            vec![opt("A")],
        );
        let c2 = corpus(vec![scoped_tweak]);
        let mut unavailable = valid_entry.clone();
        unavailable.tweak_id = "scoped".into();
        let raw = RawEntry {
            seq: Seq(6),
            bytes: serde_json::to_vec(&unavailable).unwrap(),
        };
        assert_eq!(
            classify(&raw, &c2, Some(guid_here), 19045),
            EntryValidity::Invalid(InvalidReason::TargetUnavailable)
        );
    }

    #[test]
    fn target_unavailable_reaches_option_level_scope_not_just_tweak_level() {
        // Fix 2: the tweak itself carries no `windows` restriction (would classify Valid under a
        // tweak-level-only check), but this specific option's own per-value scope excludes the
        // running build — `classify` must still call it `TargetUnavailable`.
        let restrictive = opt_scoped(
            "A",
            Some(WindowsScope {
                products: None,
                build: Some(BuildExpr::Exact(26100)),
                revision: None,
            }),
        );
        let t = tweak("demo", None, vec![restrictive]); // tweak-level windows: None
        let c = corpus(vec![t]);

        let entry = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("g".into()),
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t".into(),
            captured: Captured::OptionRef("A".into()),
            journal: Vec::new(),
        };
        let raw = RawEntry {
            seq: Seq(1),
            bytes: serde_json::to_vec(&entry).unwrap(),
        };

        assert_eq!(
            classify(&raw, &c, Some("g"), 19045),
            EntryValidity::Invalid(InvalidReason::TargetUnavailable),
            "an option's own per-value scope must be checked, not just the tweak's"
        );
    }

    #[test]
    fn option_available_if_any_covered_effect_survives_even_if_another_is_scoped_out() {
        // Fix A regression: unavailable means NO covered effect survives, not "any covered effect
        // is scoped out". An option driving two effects — one unconditional, one option-scoped to
        // a single build — must still classify Valid on a build where only the second is excluded;
        // the engine just skips the inapplicable one. The earlier (buggy) predicate used `.any()`
        // over "is scoped out", which a single-effect option can't distinguish from the correct
        // "none survive" — this test needs two effects to tell them apart.
        let mut values = BTreeMap::new();
        values.insert(
            EffectId("eff1".into()),
            OptValue::Set(ScopedValue {
                value: Value::Present(true),
                windows: None, // survives on every build
            }),
        );
        values.insert(
            EffectId("eff2".into()),
            OptValue::Set(ScopedValue {
                value: Value::Present(true),
                windows: Some(WindowsScope {
                    products: None,
                    build: Some(BuildExpr::Exact(26100)),
                    revision: None,
                }), // scoped out on 19045
            }),
        );
        let opt_a = Opt {
            label: OptLabel("A".into()),
            values,
        };
        let t = Tweak {
            id: "demo".into(),
            name: "demo".into(),
            description: String::new(),
            category: "misc".into(),
            info: None,
            warning: None,
            requires_reboot: false,
            risk_level: RiskLevel::Low,
            elevation: Level::User,
            reversible: true,
            surface: vec![effect_def("eff1"), effect_def("eff2")],
            options: vec![opt_a],
            windows: None,
        };
        let c = corpus(vec![t]);

        let entry = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("g".into()),
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t".into(),
            captured: Captured::OptionRef("A".into()),
            journal: Vec::new(),
        };
        let raw = RawEntry {
            seq: Seq(1),
            bytes: serde_json::to_vec(&entry).unwrap(),
        };

        assert_eq!(
            classify(&raw, &c, Some("g"), 19045),
            EntryValidity::Valid,
            "eff1 still survives on 19045 even though eff2 (option-scoped to 26100) does not"
        );
    }

    #[test]
    fn head_skips_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        let t = tweak("demo", None, vec![opt("A")]);
        let c = corpus(vec![t]);

        let valid = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("g".into()),
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t1".into(),
            captured: Captured::OptionRef("A".into()),
            journal: Vec::new(),
        };
        write_entry_create_new(&dir, &valid).unwrap();

        let mut invalid = valid.clone();
        invalid.seq = Seq(2); // the head by seq, but a dangling ref — must be skipped
        invalid.captured = Captured::OptionRef("Ghost".into());
        write_entry_create_new(&dir, &invalid).unwrap();

        let s = store(tmp.path());
        let head = s
            .head("demo", &c, Some("g"), 10240)
            .unwrap()
            .expect("a valid entry exists below the invalid head");
        assert_eq!(head.seq, Seq(1));
        assert_eq!(head.captured, Captured::OptionRef("A".into()));
    }

    #[test]
    fn discard_removes_only_target() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let c = empty_corpus();
        let seq1 = s.push("demo", values_entry(), &c, None, 0).unwrap();
        let seq2 = s.push("demo", values_entry(), &c, None, 0).unwrap();
        let seq3 = s.push("demo", values_entry(), &c, None, 0).unwrap();

        s.discard("demo", seq2).unwrap();

        let dir = tmp.path().join("demo");
        let mut remaining: Vec<u64> = read_raw_entries(&dir)
            .unwrap()
            .iter()
            .map(|r| r.seq.0)
            .collect();
        remaining.sort_unstable();
        assert_eq!(remaining, vec![seq1.0, seq3.0]);
    }

    #[test]
    fn consume_removes_the_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let seq = s
            .push("demo", values_entry(), &empty_corpus(), None, 0)
            .unwrap();

        s.consume("demo", seq).unwrap();
        assert!(read_raw_entries(&tmp.path().join("demo"))
            .unwrap()
            .is_empty());

        let err = s.consume("demo", seq).unwrap_err();
        assert!(matches!(err, SnapshotError::NotFound { .. }));
    }

    #[test]
    fn open_default_resolves_exe_adjacent_snapshots_dir() {
        let store = SnapshotStore::open_default().unwrap();
        let exe_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        assert_eq!(store.root, exe_dir.join("snapshots"));
    }
}
