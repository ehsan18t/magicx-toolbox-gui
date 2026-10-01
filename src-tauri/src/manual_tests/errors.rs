//! Typed error chains for the report: variant, Win32 code, classification, acquire reason. Broker
//! and script text is reduced to its Win32 code, since it can carry temp paths and the nonce.

use crate::error::Error;
use crate::tweaks::engine::apply::EngineError;
use crate::tweaks::engine::{failure_class, failure_code};
use crate::tweaks::kinds::Error as KindError;
use crate::tweaks::shared_claims::ClaimsError;
use crate::tweaks::snapshot::SnapshotError;

/// Panic-free: this also runs on the failed-restore path.
fn win32_code(text: &str) -> String {
    regex_lite::Regex::new(r"(?i)(?:Windows error|os error|HRESULT)\s*(0x[0-9a-f]+|\d+)")
        .ok()
        .and_then(|re| re.captures(text)?.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "none reported".into())
}

pub fn kind(e: &KindError) -> String {
    match e {
        KindError::NotFound(d) => format!("KindError::NotFound: {d}"),
        KindError::AccessDenied(d) => format!("KindError::AccessDenied: {d}"),
        KindError::TypeMismatch { .. } | KindError::MalformedPacked { .. } => {
            format!("KindError: {e}")
        }
        KindError::UnsupportedLevel(level) => format!("KindError::UnsupportedLevel({level:?})"),
        KindError::CouldNotAcquireElevation(level, reason, detail) => format!(
            "KindError::CouldNotAcquireElevation level={level:?} acquire_reason={reason:?} ({reason}) win32={}",
            win32_code(detail)
        ),
        KindError::ElevatedOutcomeUnknown(level, detail) => format!(
            "KindError::ElevatedOutcomeUnknown level={level:?} win32={}",
            win32_code(detail)
        ),
        KindError::ElevatedOpFailed(level, class) => {
            format!("KindError::ElevatedOpFailed level={level:?} class={class:?} ({class})")
        }
        KindError::ResourceMissing(d) => format!("KindError::ResourceMissing: {d}"),
        KindError::Invalid(d) => format!("KindError::Invalid: {d}"),
        KindError::Backend(d) => format!("KindError::Backend win32={}: {d}", win32_code(d)),
        KindError::ActionFailed(code) => format!("KindError::ActionFailed exit_code={code}"),
        KindError::ActionExecFailed(d) => {
            format!("KindError::ActionExecFailed win32={}", win32_code(d))
        }
        KindError::ActionNotStarted(d) => {
            format!("KindError::ActionNotStarted win32={}", win32_code(d))
        }
    }
}

pub fn snapshot(e: &SnapshotError) -> String {
    match e {
        SnapshotError::Io(io) => format!(
            "SnapshotError::Io kind={:?} win32={}",
            io.kind(),
            io.raw_os_error()
                .map_or_else(|| "none reported".into(), |c| c.to_string())
        ),
        other => format!("SnapshotError: {other}"),
    }
}

fn claims(e: &ClaimsError) -> String {
    match e {
        ClaimsError::Kind(k) => kind(k),
        ClaimsError::Io(io) => format!(
            "ClaimsError::Io kind={:?} win32={}",
            io.kind(),
            io.raw_os_error()
                .map_or_else(|| "none reported".into(), |c| c.to_string())
        ),
        other => format!("ClaimsError: {other}"),
    }
}

/// One line per error in the chain, nested failures indented under their report.
pub fn engine(e: &EngineError) -> Vec<String> {
    let mut out = vec![format!(
        "failure code {} class {:?}",
        failure_code(e).as_str(),
        failure_class(e)
    )];
    engine_into(e, 0, &mut out);
    out
}

fn engine_into(e: &EngineError, depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    let line = match e {
        EngineError::CaptureFailed { effect, source } => {
            format!("CaptureFailed effect={effect}: {}", kind(source))
        }
        EngineError::DriveFailed { effect, source } => {
            format!("DriveFailed effect={effect}: {}", kind(source))
        }
        EngineError::ActionFailed { effect, source } => {
            format!("ActionFailed effect={effect}: {}", kind(source))
        }
        EngineError::SnapshotWrite(s) => format!("SnapshotWrite: {}", snapshot(s)),
        EngineError::JournalMark { effect, source } => {
            format!("JournalMark effect={effect}: {}", snapshot(source))
        }
        EngineError::EntryCleanup(s) => format!("EntryCleanup: {}", snapshot(s)),
        EngineError::AttentionWrite(s) => format!("AttentionWrite: {}", snapshot(s)),
        EngineError::Claim { shared, source } => {
            format!("Claim shared={shared}: {}", claims(source))
        }
        EngineError::RollbackReport {
            original,
            rollback_failures,
            outcome_unknown,
            store,
        } => {
            out.push(format!(
                "{pad}RollbackReport outcome_unknown={outcome_unknown} rollback_failures={} store_failures={}",
                rollback_failures.len(),
                store.len()
            ));
            out.push(format!("{pad}  original:"));
            engine_into(original, depth + 2, out);
            for f in rollback_failures {
                out.push(format!("{pad}  rollback failure:"));
                engine_into(f, depth + 2, out);
            }
            for f in store {
                out.push(format!("{pad}  store failure:"));
                engine_into(f, depth + 2, out);
            }
            return;
        }
        EngineError::RestoreFailed { failures, store } => {
            out.push(format!(
                "{pad}RestoreFailed failures={} store_failures={}",
                failures.len(),
                store.len()
            ));
            for f in failures.iter().chain(store) {
                engine_into(f, depth + 1, out);
            }
            return;
        }
        other => format!("{other:?}"),
    };
    out.push(format!("{pad}{line}"));
}

pub fn app(e: &Error) -> String {
    match e {
        Error::Win32 { context, code } => format!("Error::Win32 code={code:#x}: {context}"),
        Error::TweakFailed { code, message } => {
            format!("Error::TweakFailed code={}: {message}", code.as_str())
        }
        other => format!("Error::{}: {other}", other.code()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::elevation::{AcquireReason, OpFailureClass};
    use crate::tweaks::model::{EffectId, Level};

    #[test]
    fn broker_detail_is_reduced_to_its_win32_code() {
        let e = KindError::CouldNotAcquireElevation(
            Level::Ti,
            AcquireReason::SpawnFailed,
            r"CreateProcessW C:\Users\x\AppData\Local\Temp\req-9f3a.json (Windows error 0x5)"
                .into(),
        );
        let line = kind(&e);
        assert!(
            line.contains("SpawnFailed") && line.contains("win32=0x5"),
            "{line}"
        );
        assert!(!line.contains("Temp") && !line.contains("9f3a"), "{line}");
    }

    #[test]
    fn a_rollback_report_names_every_nested_failure() {
        let e = EngineError::RollbackReport {
            original: Box::new(EngineError::DriveFailed {
                effect: EffectId("a".into()),
                source: KindError::ElevatedOpFailed(Level::Ti, OpFailureClass::AccessDenied),
            }),
            rollback_failures: vec![EngineError::ResourceMissing(EffectId("b".into()))],
            outcome_unknown: false,
            store: Vec::new(),
        };
        let lines = engine(&e).join("\n");
        assert!(lines.contains("DriveFailed effect=a"), "{lines}");
        assert!(lines.contains("class=AccessDenied"), "{lines}");
        assert!(lines.contains("ResourceMissing"), "{lines}");
    }
}
