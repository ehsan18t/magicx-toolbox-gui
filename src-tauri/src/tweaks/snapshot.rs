//! Snapshot store (spec §8.2/§8.3/§11; ADR-0002): pure storage, one JSON file per entry under a
//! per-tweak subdirectory, so one corrupt history never hides another's. Seq comes from disk plus a
//! `_seq.json` hint, never wall-clock (invariant 6). Needs Attention is a per-tweak record
//! (`_attention.json`), not an entry field, because dedup/`consume`/`discard` all delete entries.
//! Journal residue is the other durable form of unresolved state, and it is resolved per row in the
//! entry that holds it, by the operation that accounted for that row.
//! Nothing here deletes an invalid entry, or a record this machine and build do not own.

use crate::tweaks::model::{Corpus, EffectId, Value};
use crate::tweaks::validate::{option_unavailable, Milestone};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Bump on any breaking change to `Entry`'s shape; `classify` treats a mismatch as `WrongSchema`
/// rather than guessing an upgrade path (spec §8.3/§11, invariant 21).
const SCHEMA_VERSION: u32 = 1;

const SEQ_CACHE_FILE: &str = "_seq.json";

const ATTENTION_FILE: &str = "_attention.json";

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
/// `resolved` is the exception: absent means "still outstanding", the surfacing direction, so an
/// entry written before the field existed still loads and still raises its residue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalRow {
    pub action_id: EffectId,
    pub intended: bool,
    pub completed: bool,
    /// An operation that drove and verified this action has accounted for the row, so the crash
    /// scan must not raise it again. Set only by `resolve_journal_rows`, never by driving.
    #[serde(default)]
    pub resolved: bool,
}

/// Which operation left the tweak in a state the user has to resolve (ADR-0001/0002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionReason {
    ApplyFailed,
    RestoreFailed,
    CrashResidue,
    /// The record itself could not be read, so whatever it holds is still unresolved. Never
    /// persisted: synthesized by the reader so an I/O failure cannot read as a clean tweak.
    RecordUnreadable,
}

/// What kind of step could not be verified, so the UI can tell a retryable drive from a one-way
/// action or a store failure instead of re-parsing a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionKind {
    Drive,
    Verify,
    OutcomeUnknown,
    Action,
    NoUndo,
    Claim,
    Store,
    CrashResidue,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttentionItem {
    #[serde(default)]
    pub effect: Option<EffectId>,
    pub kind: AttentionKind,
    pub message: String,
}

/// Needs Attention for one tweak, persisted in its own record so nothing that deletes an entry --
/// dedup, a later verified rollback's `consume`, an entry turning invalid -- can drop it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attention {
    pub reason: AttentionReason,
    pub items: Vec<AttentionItem>,
}

/// The attention record as it sits on disk, stamped like an `Entry` so a foreign-machine or
/// future-schema record is ignored rather than believed.
#[derive(Debug, Serialize, Deserialize)]
struct AttentionRecord {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    machine_guid: Option<String>,
    tweak_id: String,
    timestamp: String,
    attention: Attention,
}

/// What sits at the record path, and whether this build on this machine owns it.
enum RecordState {
    Absent,
    /// Claims no owner at all, so a later mark may replace it; still never deleted (ADR-0002).
    Unreadable,
    /// Another machine's or a *newer* build's: never overwritten, never removed.
    Theirs,
    Ours(Attention),
}

/// Whether a stamp belongs to a build this one must defer to. An older or missing version is this
/// build's to replace: refusing it would leave such a record unreadable, unwritable and unclearable.
fn written_by_a_newer_build(schema_version: u32) -> bool {
    schema_version > SCHEMA_VERSION
}

fn read_record(path: &Path, machine_guid: Option<&str>) -> Result<RecordState, SnapshotError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(RecordState::Absent),
        Err(e) => return Err(SnapshotError::Io(e)),
    };
    let Ok(record) = serde_json::from_slice::<AttentionRecord>(&bytes) else {
        return Ok(RecordState::Unreadable);
    };
    if written_by_a_newer_build(record.schema_version) {
        return Ok(RecordState::Theirs);
    }
    match (record.machine_guid.as_deref(), machine_guid) {
        (Some(recorded), Some(current)) if recorded != current => Ok(RecordState::Theirs),
        _ => Ok(RecordState::Ours(record.attention)),
    }
}

/// One persisted snapshot entry (spec §8.3). Only `schema_version` and `machine_guid` default when
/// absent: `seq`, `tweak_id`, `captured` and `journal` are identity data, and a defaulted `seq`
/// once let file content pick which file `mark_completed` rewrote.
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

    #[error(
        "the Needs Attention record for tweak '{tweak_id}' belongs to another machine or build"
    )]
    ForeignAttention { tweak_id: String },
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
        let mut superseded: Vec<Seq> = Vec::new();
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
                // An outstanding row is the whole durable mark for work a probe-less action left
                // undetectable, so it outlives dedup: superseding the entry would delete it.
                if existing.journal.iter().any(is_outstanding) {
                    log::warn!(
                        "tweak '{tweak_id}': entry {:?} kept by dedup -- its journal is still outstanding",
                        raw.seq
                    );
                    continue;
                }
                if matches!(&existing.captured, Captured::OptionRef(l) if l == label) {
                    superseded.push(raw.seq);
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
        // The new return point is durable before the superseded one goes (ADR-0002): a failed write
        // must never be able to leave the tweak with no entry at all.
        write_entry_create_new(&dir, &entry)?;
        for old in superseded {
            match fs::remove_file(entry_path(&dir, old)) {
                // Only a duplicate return point is left behind, which the next push dedups again --
                // far cheaper than failing an apply whose entry is already on disk.
                Err(e) => log::warn!("tweak '{tweak_id}': dedup could not remove {old:?}: {e}"),
                Ok(()) => log::debug!("tweak '{tweak_id}': dedup removed {old:?}"),
            }
        }
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

    /// Every entry whose journal is still the crash scan's business: this build's and machine's,
    /// newest first. Residue can sit on a superseded entry or on one the corpus has since made
    /// dangling, so `head`'s walk would miss it. Which *rows* are still outstanding is the reader's
    /// call, per row -- see [`JournalRow::resolved`].
    pub fn unresolved_entries(
        &self,
        tweak_id: &str,
        machine_guid: Option<&str>,
    ) -> Result<Vec<Entry>, SnapshotError> {
        let dir = self.tweak_dir(tweak_id);
        let mut raws = read_raw_entries(&dir)?;
        raws.sort_by_key(|r| std::cmp::Reverse(r.seq));
        Ok(raws
            .iter()
            .filter_map(|raw| serde_json::from_slice::<Entry>(&raw.bytes).ok())
            .filter(|e| e.schema_version == SCHEMA_VERSION)
            .filter(|e| match (e.machine_guid.as_deref(), machine_guid) {
                (Some(entry_guid), Some(current)) => entry_guid == current,
                _ => true,
            })
            .collect())
    }

    /// Tweak ids whose directory holds a Needs Attention record, so one the corpus no longer defines
    /// can be named rather than left with no card to badge and no way to release it.
    pub fn recorded_tweaks(&self) -> Result<Vec<String>, SnapshotError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() || !entry.path().join(ATTENTION_FILE).exists() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                out.push(name.to_string());
            }
        }
        out.sort_unstable();
        Ok(out)
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

    /// Durably flips one journal row's `completed` bit (spec §8.1, invariant 5). `seq` (the
    /// filename) is the only trusted write target; the content's own `seq` is overwritten to match,
    /// so content never picks which file is written.
    pub fn mark_completed(
        &self,
        tweak_id: &str,
        seq: Seq,
        action_id: &EffectId,
    ) -> Result<(), SnapshotError> {
        update_journal(&self.tweak_dir(tweak_id), tweak_id, seq, |journal| {
            let row = journal
                .iter_mut()
                .find(|r| r.action_id == *action_id)
                .ok_or_else(|| SnapshotError::UnknownJournalAction {
                    tweak_id: tweak_id.to_string(),
                    seq,
                    action_id: action_id.to_string(),
                })?;
            row.completed = true;
            Ok(())
        })?;
        log::debug!("tweak '{tweak_id}': marked action '{action_id}' completed at seq {seq:?}");
        Ok(())
    }

    /// Marks every outstanding row naming an action in `accounted` resolved, in the entry that
    /// holds it, so the crash scan stops raising it. `accounted` is what the calling operation
    /// actually drove and verified: a row it never probed or undid stays outstanding. Only a fully
    /// verified apply or restore may call it (ADR-0002), never a failure path.
    pub fn resolve_journal_rows(
        &self,
        tweak_id: &str,
        accounted: &BTreeSet<EffectId>,
        machine_guid: Option<&str>,
    ) -> Result<usize, SnapshotError> {
        if accounted.is_empty() {
            return Ok(0);
        }
        let dir = self.tweak_dir(tweak_id);
        let mut resolved = 0usize;
        for entry in self.unresolved_entries(tweak_id, machine_guid)? {
            let rows: Vec<EffectId> = entry
                .journal
                .iter()
                .filter(|r| is_outstanding(r) && accounted.contains(&r.action_id))
                .map(|r| r.action_id.clone())
                .collect();
            if rows.is_empty() {
                continue;
            }
            update_journal(&dir, tweak_id, entry.seq, |journal| {
                for row in journal.iter_mut().filter(|r| rows.contains(&r.action_id)) {
                    row.resolved = true;
                }
                Ok(())
            })?;
            log::info!(
                "tweak '{tweak_id}': resolved {} journal row(s) at seq {:?}",
                rows.len(),
                entry.seq
            );
            resolved += rows.len();
        }
        Ok(resolved)
    }

    /// This tweak's Needs Attention record (ADR-0001/0002), or `None` only when there is no record
    /// at all. A record this build cannot use is reported as [`AttentionReason::RecordUnreadable`],
    /// never believed and never read as a clean tweak: what it holds is still unresolved.
    pub fn attention(
        &self,
        tweak_id: &str,
        machine_guid: Option<&str>,
    ) -> Result<Option<Attention>, SnapshotError> {
        match read_record(&self.attention_path(tweak_id), machine_guid)? {
            RecordState::Ours(attention) => Ok(Some(attention)),
            RecordState::Absent => Ok(None),
            RecordState::Unreadable => {
                log::warn!("tweak '{tweak_id}': unreadable Needs Attention record, kept on disk");
                Ok(Some(unusable_record(
                    "the Needs Attention record could not be parsed, so anything it holds is still unresolved",
                )))
            }
            RecordState::Theirs => {
                log::warn!("tweak '{tweak_id}': Needs Attention record is another machine's or another build's, kept on disk");
                Ok(Some(unusable_record(
                    "the Needs Attention record belongs to another machine or another build, so this one cannot say whether it is resolved",
                )))
            }
        }
    }

    /// Durably records Needs Attention, replacing any earlier record. Deliberately independent of
    /// the entries: dedup, `consume`, `discard` and an entry turning invalid must not drop it.
    /// Refuses to overwrite another machine's or build's record, exactly as dedup refuses its entry.
    pub fn set_attention(
        &self,
        tweak_id: &str,
        machine_guid: Option<&str>,
        attention: Attention,
    ) -> Result<(), SnapshotError> {
        if matches!(
            read_record(&self.attention_path(tweak_id), machine_guid)?,
            RecordState::Theirs
        ) {
            log::error!("tweak '{tweak_id}': another machine's or build's Needs Attention record is on disk, so this mark was not recorded");
            return Err(SnapshotError::ForeignAttention {
                tweak_id: tweak_id.to_string(),
            });
        }
        let dir = self.tweak_dir(tweak_id);
        fs::create_dir_all(&dir)?;
        let reason = attention.reason;
        let record = AttentionRecord {
            schema_version: SCHEMA_VERSION,
            machine_guid: machine_guid.map(str::to_string),
            tweak_id: tweak_id.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            attention,
        };
        let json = serde_json::to_vec_pretty(&record).expect("AttentionRecord always serializes");
        write_atomic(&dir, ATTENTION_FILE, &json)?;
        log::warn!("tweak '{tweak_id}': recorded Needs Attention ({reason:?})");
        Ok(())
    }

    /// Releases the record, never the journal residue ([`Self::resolve_journal_rows`] resolves that
    /// per row, so a record cleared over an outstanding row is re-marked by the next crash scan).
    /// Only a verified apply or restore may call it, never a failure path (ADR-0002).
    pub fn clear_attention(
        &self,
        tweak_id: &str,
        machine_guid: Option<&str>,
    ) -> Result<(), SnapshotError> {
        self.remove_record(tweak_id, machine_guid, false)
    }

    /// Consent's release, and the only one that also removes a record stamped for another machine or
    /// build: nothing here can ever resolve that record, so refusing it badges the tweak permanently.
    pub fn clear_attention_consented(
        &self,
        tweak_id: &str,
        machine_guid: Option<&str>,
    ) -> Result<(), SnapshotError> {
        self.remove_record(tweak_id, machine_guid, true)
    }

    /// A record naming no owner is this build's to remove, exactly as it is this build's to replace.
    fn remove_record(
        &self,
        tweak_id: &str,
        machine_guid: Option<&str>,
        consented: bool,
    ) -> Result<(), SnapshotError> {
        let path = self.attention_path(tweak_id);
        match read_record(&path, machine_guid)? {
            RecordState::Absent => return Ok(()),
            RecordState::Theirs if !consented => {
                log::warn!("tweak '{tweak_id}': Needs Attention record is not this build's to clear, left on disk");
                return Ok(());
            }
            RecordState::Theirs | RecordState::Ours(_) | RecordState::Unreadable => {}
        }
        match fs::remove_file(&path) {
            Ok(()) => {
                log::debug!("tweak '{tweak_id}': cleared Needs Attention");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(SnapshotError::Io(e)),
        }
    }

    fn attention_path(&self, tweak_id: &str) -> PathBuf {
        self.tweak_dir(tweak_id).join(ATTENTION_FILE)
    }
}

/// A row a crash could have left half-done: planned, never confirmed complete, and not since
/// accounted for by an operation that drove and verified the same action.
pub fn is_outstanding(row: &JournalRow) -> bool {
    row.intended && !row.completed && !row.resolved
}

fn unusable_record(message: &str) -> Attention {
    Attention {
        reason: AttentionReason::RecordUnreadable,
        items: vec![AttentionItem {
            effect: None,
            kind: AttentionKind::Store,
            message: message.to_string(),
        }],
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
    write_atomic(dir, SEQ_CACHE_FILE, &json)
}

/// Replacing atomic write for the directory's non-entry files: temp file beside them, fsynced,
/// then renamed over. Entries use `write_entry_create_new`, which must never replace.
fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), SnapshotError> {
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    tmp.persist(dir.join(name))
        .map_err(|e| SnapshotError::Io(e.error))?;
    Ok(())
}

/// The high-water mark (spec §8.2, invariant 6): max of what's actually on disk and the persisted
/// cache, so a dedup-vacated head can never reissue an old number and a lost/corrupt cache
/// self-heals from the directory. Never wall-clock derived.
fn current_max_seq(dir: &Path) -> Result<u64, SnapshotError> {
    Ok(scan_max_seq(dir)?.max(read_seq_cache(dir).unwrap_or(0)))
}

fn next_seq(dir: &Path) -> Result<Seq, SnapshotError> {
    Ok(Seq(current_max_seq(dir)? + 1))
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

/// The one read-modify-rewrite path for a journal: `seq` (the filename) is the only trusted write
/// target, and the content's own `seq` is overwritten to match, so content never picks which file is
/// written. An `edit` that fails writes nothing.
fn update_journal(
    dir: &Path,
    tweak_id: &str,
    seq: Seq,
    edit: impl FnOnce(&mut Vec<JournalRow>) -> Result<(), SnapshotError>,
) -> Result<(), SnapshotError> {
    let bytes = fs::read(entry_path(dir, seq)).map_err(|e| io_to_not_found(e, tweak_id, seq))?;
    let mut entry: Entry = serde_json::from_slice(&bytes).map_err(|_| SnapshotError::Corrupt {
        tweak_id: tweak_id.to_string(),
        seq,
    })?;
    edit(&mut entry.journal)?;
    entry.seq = seq;
    rewrite_entry(dir, seq, &entry)
}

/// Atomic in-place rewrite at the caller-trusted `seq`, never `entry.seq` (see `update_journal`).
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
                        resolved: false,
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

    fn attention() -> Attention {
        Attention {
            reason: AttentionReason::ApplyFailed,
            items: vec![AttentionItem {
                effect: Some(EffectId("eff1".into())),
                kind: AttentionKind::OutcomeUnknown,
                message: "the elevated step's outcome is unknown".into(),
            }],
        }
    }

    /// A folder written by a build that kept the mark on the entry still loads, and its on-entry
    /// field asserts nothing: no record means no attention.
    #[test]
    fn an_older_folder_loads_and_its_on_entry_mark_is_not_attention() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        let json = format!(
            r#"{{"schema_version":{SCHEMA_VERSION},"machine_guid":"{GUID}","tweak_id":"demo","seq":3,"timestamp":"t","captured":{{"OptionRef":"A"}},"journal":[],"attention":{{"reason":"apply_failed","items":["x"]}}}}"#
        );
        fs::write(entry_path(&dir, Seq(3)), json).unwrap();
        let c = corpus(vec![tweak("demo", None, vec![opt("A")])]);

        let s = store(tmp.path());
        let head = s
            .head("demo", &c, Some(GUID), 19045)
            .unwrap()
            .expect("the older entry is still valid");
        assert_eq!(head.seq, Seq(3));
        assert_eq!(s.attention("demo", Some(GUID)).unwrap(), None);
    }

    /// ADR-0002: every entry release deletes a file the mark must outlive.
    #[test]
    fn the_record_survives_reopen_dedup_and_consume() {
        let tmp = tempfile::tempdir().unwrap();
        let c = corpus(vec![tweak("demo", None, vec![opt("A")])]);
        let s = store(tmp.path());
        let seq = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 19045)
            .unwrap();
        s.set_attention("demo", Some(GUID), attention()).unwrap();

        let reopened = store(tmp.path());
        assert_eq!(
            reopened.attention("demo", Some(GUID)).unwrap(),
            Some(attention())
        );

        let deduped = reopened
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 19045)
            .unwrap();
        assert_ne!(deduped, seq);
        reopened.consume("demo", deduped).unwrap();
        assert_eq!(reopened.head("demo", &c, Some(GUID), 19045).unwrap(), None);
        assert_eq!(
            reopened.attention("demo", Some(GUID)).unwrap(),
            Some(attention())
        );

        reopened.clear_attention("demo", Some(GUID)).unwrap();
        assert_eq!(reopened.attention("demo", Some(GUID)).unwrap(), None);
    }

    /// Never believed, and never read as clean either: this machine cannot say whether the tweak
    /// the other one marked is resolved, and answering "nothing pending" would claim it can.
    #[test]
    fn a_record_from_another_machine_is_reported_not_believed() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        s.set_attention("demo", Some("another-machine"), attention())
            .unwrap();
        assert_eq!(
            s.attention("demo", Some(GUID)).unwrap().map(|a| a.reason),
            Some(AttentionReason::RecordUnreadable)
        );
    }

    /// ADR-0002: entries are never deleted across a machine boundary, and the record follows the
    /// same rule -- a local mark may not overwrite it and a local clear may not remove it.
    #[test]
    fn a_foreign_record_is_never_overwritten_or_deleted() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        s.set_attention("demo", Some("another-machine"), attention())
            .unwrap();
        let path = tmp.path().join("demo").join(ATTENTION_FILE);
        let before = fs::read(&path).unwrap();

        let err = s
            .set_attention("demo", Some(GUID), attention())
            .expect_err("a local mark must not overwrite another machine's record");
        assert!(matches!(err, SnapshotError::ForeignAttention { .. }));

        s.clear_attention("demo", Some(GUID))
            .expect("clearing what this machine does not own is not a failure");
        assert!(path.exists(), "the foreign record must stay on disk");
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    /// Copy `snapshots/` to a second machine and the record there is one this build can neither
    /// read nor resolve. Consent is the one release left; without it the badge is permanent, which
    /// is the "no legitimate way to release" ADR-0002 forbids.
    #[test]
    fn only_consent_releases_another_machines_record() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        s.set_attention("demo", Some("another-machine"), attention())
            .unwrap();
        let path = tmp.path().join("demo").join(ATTENTION_FILE);

        s.clear_attention("demo", Some(GUID)).unwrap();
        assert!(
            path.exists(),
            "a verified apply or restore must not release another machine's record"
        );
        assert_eq!(
            s.attention("demo", Some(GUID)).unwrap().map(|a| a.reason),
            Some(AttentionReason::RecordUnreadable)
        );

        s.clear_attention_consented("demo", Some(GUID))
            .expect("consent releases what nothing on this machine can resolve");
        assert!(!path.exists());
        assert_eq!(s.attention("demo", Some(GUID)).unwrap(), None);
    }

    #[test]
    fn a_wrong_schema_record_is_left_alone_too() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        let json = format!(
            r#"{{"schema_version":{},"machine_guid":"{GUID}","tweak_id":"demo","timestamp":"t","attention":{{"reason":"apply_failed","items":[]}}}}"#,
            SCHEMA_VERSION + 1
        );
        fs::write(dir.join(ATTENTION_FILE), &json).unwrap();

        let s = store(tmp.path());
        assert_eq!(
            s.attention("demo", Some(GUID)).unwrap().map(|a| a.reason),
            Some(AttentionReason::RecordUnreadable)
        );
        assert!(s.set_attention("demo", Some(GUID), attention()).is_err());
        s.clear_attention("demo", Some(GUID)).unwrap();
        assert_eq!(fs::read_to_string(dir.join(ATTENTION_FILE)).unwrap(), json);

        s.clear_attention_consented("demo", Some(GUID))
            .expect("consent is the one release left for a record this build cannot read");
        assert!(!dir.join(ATTENTION_FILE).exists());
    }

    /// A record that will not parse is surfaced, not swallowed: as "no attention" it would hide a
    /// real mark. It names no owner, so this build may replace it and may release it -- keeping it
    /// through a clear would badge the tweak with nothing able to lift the badge.
    #[test]
    fn an_unparseable_record_reports_itself_and_is_still_releasable() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(ATTENTION_FILE), b"{ not json").unwrap();

        let s = store(tmp.path());
        let reported = s
            .attention("demo", Some(GUID))
            .unwrap()
            .expect("an unparseable record is never a clean tweak");
        assert_eq!(reported.reason, AttentionReason::RecordUnreadable);

        s.set_attention("demo", Some(GUID), attention())
            .expect("a record naming no owner may be replaced");
        assert_eq!(s.attention("demo", Some(GUID)).unwrap(), Some(attention()));

        fs::write(dir.join(ATTENTION_FILE), b"{ not json").unwrap();
        s.clear_attention("demo", Some(GUID)).unwrap();
        assert_eq!(s.attention("demo", Some(GUID)).unwrap(), None);
    }

    /// `head` stops at the newest valid entry, so the crash scan needs its own source: residue can
    /// sit on a superseded entry the head walk never reaches.
    #[test]
    fn unresolved_entries_reaches_entries_head_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let c = corpus(vec![tweak("demo", None, vec![opt("A")])]);
        let older = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 19045)
            .unwrap();
        let newer = s
            .push("demo", values_entry(), &c, Some(GUID), 19045)
            .unwrap();

        let seqs: Vec<Seq> = s
            .unresolved_entries("demo", Some(GUID))
            .unwrap()
            .iter()
            .map(|e| e.seq)
            .collect();
        assert_eq!(
            seqs,
            vec![newer, older],
            "newest first, superseded included"
        );
        assert_eq!(
            s.head("demo", &c, Some(GUID), 19045)
                .unwrap()
                .map(|e| e.seq),
            Some(newer),
            "head stops at the newest valid entry, so the older one needs its own source"
        );
    }

    #[test]
    fn unresolved_entries_skips_another_machines_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        let foreign = Entry {
            schema_version: SCHEMA_VERSION,
            machine_guid: Some("foreign-machine".into()),
            tweak_id: "demo".into(),
            seq: Seq(1),
            timestamp: "t".into(),
            captured: Captured::Values(BTreeMap::new()),
            journal: Vec::new(),
        };
        write_entry_create_new(&dir, &foreign).unwrap();
        let s = store(tmp.path());
        assert!(s.unresolved_entries("demo", Some(GUID)).unwrap().is_empty());
    }

    fn crashed_entry_for(action_id: &str) -> NewEntry {
        NewEntry {
            captured: Captured::Values(BTreeMap::new()),
            journal: vec![JournalRow {
                action_id: EffectId(action_id.into()),
                intended: true,
                completed: false,
                resolved: false,
            }],
        }
    }

    fn crashed_entry() -> NewEntry {
        crashed_entry_for("act1")
    }

    /// Every row the crash scan would still raise, across the whole history.
    fn outstanding(s: &SnapshotStore) -> Vec<EffectId> {
        s.unresolved_entries("demo", Some(GUID))
            .unwrap()
            .iter()
            .flat_map(|e| e.journal.iter())
            .filter(|r| is_outstanding(r))
            .map(|r| r.action_id.clone())
            .collect()
    }

    /// The mark lands in the entry that holds the row, so it survives a reopen, and the entry
    /// itself is untouched (ADR-0002).
    #[test]
    fn an_accounted_row_resolves_in_place_and_keeps_its_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        s.push("demo", crashed_entry(), &empty_corpus(), Some(GUID), 0)
            .unwrap();
        assert_eq!(outstanding(&s), vec![EffectId("act1".into())]);

        let resolved = s
            .resolve_journal_rows(
                "demo",
                &BTreeSet::from([EffectId("act1".into())]),
                Some(GUID),
            )
            .unwrap();
        assert_eq!(resolved, 1);
        assert!(outstanding(&store(tmp.path())).is_empty());
        assert_eq!(
            read_raw_entries(&tmp.path().join("demo")).unwrap().len(),
            1,
            "nothing was deleted to achieve it (ADR-0002)"
        );
    }

    /// Only what the operation accounted for: an unrelated row on another entry stays outstanding,
    /// and so does a crash pushed afterwards.
    #[test]
    fn a_row_the_operation_did_not_account_for_stays_outstanding() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        s.push("demo", crashed_entry(), &empty_corpus(), Some(GUID), 0)
            .unwrap();
        s.push(
            "demo",
            crashed_entry_for("act2"),
            &empty_corpus(),
            Some(GUID),
            0,
        )
        .unwrap();

        s.resolve_journal_rows(
            "demo",
            &BTreeSet::from([EffectId("act1".into())]),
            Some(GUID),
        )
        .unwrap();
        assert_eq!(outstanding(&s), vec![EffectId("act2".into())]);

        s.push("demo", crashed_entry(), &empty_corpus(), Some(GUID), 0)
            .unwrap();
        assert_eq!(
            outstanding(&s),
            vec![EffectId("act1".into()), EffectId("act2".into())],
            "a fresh crash after a resolve is still a crash"
        );
    }

    /// A probe-less action leaves detect reading the old option, so the outstanding row is the only
    /// durable mark left. Re-capturing the same label must not delete the entry that holds it: the
    /// applying operation resolves only what it drove, so nothing else would account for the row.
    #[test]
    fn dedup_keeps_an_entry_whose_journal_is_still_outstanding() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        let c = corpus(vec![tweak("demo", None, vec![opt("A")])]);
        let crashed = NewEntry {
            captured: Captured::OptionRef("A".into()),
            journal: vec![JournalRow {
                action_id: EffectId("act1".into()),
                intended: true,
                completed: false,
                resolved: false,
            }],
        };
        let first = s.push("demo", crashed, &c, Some(GUID), 19045).unwrap();

        let second = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 19045)
            .unwrap();
        assert_ne!(first, second);
        assert!(
            entry_path(&tmp.path().join("demo"), first).exists(),
            "the entry holding an outstanding row must outlive a re-capture of its own label"
        );
        assert_eq!(
            outstanding(&store(tmp.path())),
            vec![EffectId("act1".into())]
        );
    }

    /// An older or unstamped record stays this build's to read, replace and clear. Refusing it as
    /// another build's would strand it: permanently unreadable, unwritable and unclearable.
    #[test]
    fn an_older_or_unstamped_record_is_this_builds_to_replace() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(ATTENTION_FILE),
            format!(
                r#"{{"machine_guid":"{GUID}","tweak_id":"demo","timestamp":"t","attention":{{"reason":"restore_failed","items":[]}}}}"#
            ),
        )
        .unwrap();

        let s = store(tmp.path());
        assert_eq!(
            s.attention("demo", Some(GUID)).unwrap(),
            Some(Attention {
                reason: AttentionReason::RestoreFailed,
                items: Vec::new(),
            }),
            "an unstamped record still reads"
        );
        s.set_attention("demo", Some(GUID), attention())
            .expect("and may be replaced");
        s.clear_attention("demo", Some(GUID)).unwrap();
        assert_eq!(s.attention("demo", Some(GUID)).unwrap(), None);
    }

    #[test]
    fn recorded_tweaks_names_only_directories_holding_a_record() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(tmp.path());
        s.push("no_record", values_entry(), &empty_corpus(), None, 0)
            .unwrap();
        s.set_attention("marked", Some(GUID), attention()).unwrap();
        assert_eq!(s.recorded_tweaks().unwrap(), vec!["marked".to_string()]);
    }

    /// ADR-0002: the new return point is durable before the superseded one goes, so a dedup whose
    /// removal fails still leaves a restorable head.
    #[test]
    fn push_writes_the_new_entry_before_removing_the_superseded_one() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        let c = corpus(vec![tweak("demo", None, vec![opt("A")])]);
        let s = store(tmp.path());
        let first = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 19045)
            .unwrap();
        // A handle sharing read but not delete: the dedup's removal fails with a sharing violation.
        // (A read-only attribute would not do it: `remove_file` clears that itself and retries.)
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(entry_path(&dir, first))
            .unwrap();

        let second = s
            .push("demo", option_ref_entry("A"), &c, Some(GUID), 19045)
            .unwrap();
        assert!(
            entry_path(&dir, second).exists(),
            "the new entry is written"
        );
        assert_eq!(
            s.head("demo", &c, Some(GUID), 19045)
                .unwrap()
                .map(|e| e.seq),
            Some(second)
        );

        assert!(
            entry_path(&dir, first).exists(),
            "the superseded entry's removal must really have failed"
        );
        drop(held);
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
                resolved: false,
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
