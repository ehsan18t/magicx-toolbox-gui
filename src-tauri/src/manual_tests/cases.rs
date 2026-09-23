use std::fmt;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::errors;
use super::runner::{arm_batches, take_batches, BatchRecord, Ctx, Verdict};
use crate::commands::tweaks::{build_deps, find_tweak};
use crate::error::{win32, Error};
use crate::services::{scheduler_service, system_info_service};
use crate::tweaks::compiled_corpus;
use crate::tweaks::engine::{context, detect, lifecycle, user_facing_failure, Phase};
use crate::tweaks::model::{Effect, Tweak, Value};
use crate::tweaks::validate::applicable_surface;

const TWEAK_ID: &str = "block_update_pipeline";
const OPTION: &str = "Blocked";
/// The broker parent's wait on one TrustedInstaller child.
const TI_BUDGET_MS: u128 = 30_000;
const POLL: Duration = Duration::from_secs(60);
const RESTORE_HELP: &str = "The engine kept the snapshot: open the 'Block the Windows Update pipeline' card, which shows Needs Attention, and restore from there. Do not delete the snapshot.";

/// The last baseline read, kept for comparison by the tests that apply.
static REMEMBERED: Mutex<Option<Vec<Reading>>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Reading {
    effect: String,
    value: Result<Value, String>,
}

fn show(v: &Result<Value, String>) -> String {
    match v {
        Ok(v) => format!("{v:?}"),
        Err(e) => format!("unreadable ({e})"),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Difference {
    effect: String,
    expected: String,
    actual: String,
}

impl fmt::Display for Difference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: expected {}, actual {}",
            self.effect, self.expected, self.actual
        )
    }
}

/// Every effect whose value is not the same readable value on both sides.
pub(super) fn compare(expected: &[Reading], actual: &[Reading]) -> Vec<Difference> {
    let mut out: Vec<Difference> = expected
        .iter()
        .filter_map(|e| {
            let a = actual.iter().find(|a| a.effect == e.effect);
            let same = matches!((&e.value, a.map(|a| &a.value)), (Ok(x), Some(Ok(y))) if x == y);
            (!same).then(|| Difference {
                effect: e.effect.clone(),
                expected: show(&e.value),
                actual: a.map_or_else(|| "not read".into(), |a| show(&a.value)),
            })
        })
        .collect();
    out.extend(
        actual
            .iter()
            .filter(|a| !expected.iter().any(|e| e.effect == a.effect))
            .map(|a| Difference {
                effect: a.effect.clone(),
                expected: "not in the baseline".into(),
                actual: show(&a.value),
            }),
    );
    out
}

pub(super) fn refusal(
    elevated: bool,
    entries: usize,
    attention: bool,
    locked: bool,
) -> Option<String> {
    if !elevated {
        Some("the app is not elevated; restart it as administrator".into())
    } else if attention {
        Some(format!(
            "'{TWEAK_ID}' shows Needs Attention; resolve it on the tweak card first"
        ))
    } else if entries > 0 {
        Some(format!(
            "'{TWEAK_ID}' already has {entries} snapshot entr{}; restore it from the tweak card first",
            if entries == 1 { "y" } else { "ies" }
        ))
    } else if locked {
        Some(format!("'{TWEAK_ID}' is being changed right now"))
    } else {
        None
    }
}

fn target(cx: &Ctx) -> Result<&'static Tweak, Verdict> {
    find_tweak(compiled_corpus(), TWEAK_ID).map_err(|e| {
        cx.error(errors::app(&e));
        Verdict::fail(format!("'{TWEAK_ID}' is not in this build's corpus"))
    })
}

fn read_surface(cx: &Ctx, tweak: &Tweak, label: &str) -> Vec<Reading> {
    let started = Instant::now();
    cx.info(format!("{label}: reading every effect"));
    let deps = build_deps(cx.host.engine());
    let corpus = compiled_corpus();
    let mut out = Vec::new();
    for effect in applicable_surface(tweak, &deps.running.to_milestone()) {
        let t0 = Instant::now();
        let value = match &effect.kind {
            Effect::Setting(s) => deps
                .kinds
                .read(s, &context::read_route(effect, deps.level, corpus))
                .map_err(|e| errors::kind(&e)),
            _ => Err("not a setting; this test reads settings only".into()),
        };
        let line = format!(
            "{label} {} = {} ({} ms)",
            effect.id,
            show(&value),
            t0.elapsed().as_millis()
        );
        if value.is_ok() {
            cx.info(line);
        } else {
            cx.error(line);
        }
        out.push(Reading {
            effect: effect.id.0.clone(),
            value,
        });
    }
    cx.info(format!(
        "{label}: {} effects read in {} ms",
        out.len(),
        started.elapsed().as_millis()
    ));
    out
}

fn preconditions(cx: &Ctx) -> Result<(), Verdict> {
    let t0 = Instant::now();
    cx.info("preconditions: checking");
    let elevated = system_info_service::is_running_as_admin();
    cx.info(format!("precondition elevated = {elevated}"));
    let deps = build_deps(cx.host.engine());
    let entries = match deps.snapshots.list(
        TWEAK_ID,
        compiled_corpus(),
        deps.machine_guid,
        deps.running.build,
    ) {
        Ok(list) => list.len(),
        Err(e) => {
            cx.error(format!(
                "precondition snapshot entries unreadable: {}",
                errors::snapshot(&e)
            ));
            return Err(Verdict::fail(
                "Refused, nothing was changed: the snapshot store could not be read",
            ));
        }
    };
    cx.info(format!("precondition snapshot entries = {entries}"));
    let attention = detect::attention(TWEAK_ID, &deps);
    cx.info(format!(
        "precondition needs attention = {}",
        attention.as_ref().map_or_else(
            || "none".to_string(),
            |a| format!("{:?} with {} item(s)", a.reason, a.items.len())
        )
    ));
    let locked = lifecycle::is_locked(TWEAK_ID);
    cx.info(format!("precondition apply in flight = {locked}"));
    if let Some(why) = refusal(elevated, entries, attention.is_some(), locked) {
        cx.error(format!("preconditions: refused: {why}"));
        return Err(Verdict::fail(format!(
            "Refused, nothing was changed: {why}"
        )));
    }
    cx.info(format!(
        "preconditions: passed ({} ms)",
        t0.elapsed().as_millis()
    ));
    Ok(())
}

/// Preconditions, then a fully readable baseline, remembered for later runs.
fn capture_baseline(cx: &Ctx, tweak: &Tweak) -> Result<Vec<Reading>, Verdict> {
    preconditions(cx)?;
    let baseline = read_surface(cx, tweak, "baseline");
    let unreadable: Vec<&str> = baseline
        .iter()
        .filter(|r| r.value.is_err())
        .map(|r| r.effect.as_str())
        .collect();
    if !unreadable.is_empty() {
        return Err(Verdict::fail(format!(
            "Refused, nothing was changed: the baseline could not read {}",
            unreadable.join(", ")
        )));
    }
    let mut remembered = REMEMBERED.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(earlier) = remembered.as_ref() {
        let diffs = compare(earlier, &baseline);
        cx.info(format!(
            "baseline vs the remembered baseline: {} difference(s)",
            diffs.len()
        ));
        for d in diffs {
            cx.info(format!("  remembered {d}"));
        }
    }
    *remembered = Some(baseline.clone());
    Ok(baseline)
}

const BATCH_WINDOW: &str = "elevated batches are recorded process-wide while this test runs; one from a tweak changed elsewhere in the app during this window would also be listed";

fn batch_line(i: usize, b: &BatchRecord) -> String {
    format!(
        "{:?} batch {}: {} ops in {} ms, {}, {}% of the {TI_BUDGET_MS} ms broker wait",
        b.level,
        i + 1,
        b.ops,
        b.elapsed_ms,
        if b.ok { "ok" } else { "FAILED" },
        b.elapsed_ms.saturating_mul(100) / TI_BUDGET_MS
    )
}

fn log_batches(cx: &Ctx, phase: &str, batches: &[BatchRecord]) {
    cx.info(format!(
        "{phase}: {} elevated batch(es); {BATCH_WINDOW}",
        batches.len()
    ));
    for (i, b) in batches.iter().enumerate() {
        let line = format!("{phase}: {}", batch_line(i, b));
        if b.ok {
            cx.info(line);
        } else {
            cx.error(line);
        }
    }
}

struct Applied {
    verified: bool,
    batches: Vec<BatchRecord>,
    wall_ms: u128,
}

/// `Err` only when the apply was refused before the engine touched anything.
fn apply(cx: &Ctx, tweak: &'static Tweak) -> Result<Applied, Verdict> {
    cx.info(format!("apply: '{OPTION}' through the apply_tweak path"));
    arm_batches();
    let t0 = Instant::now();
    let result = cx.host.apply(tweak, OPTION);
    let wall_ms = t0.elapsed().as_millis();
    let batches = take_batches();
    log_batches(cx, "apply", &batches);
    let verified = match result {
        Err(e) => {
            cx.error(format!("apply: refused: {}", errors::app(&e)));
            return Err(Verdict::fail(format!(
                "Apply refused, nothing was changed: {e}"
            )));
        }
        Ok(Err(e)) => {
            cx.error(format!("apply: failed after {wall_ms} ms"));
            for line in errors::engine(&e) {
                cx.error(format!("  {line}"));
            }
            cx.error(format!(
                "apply: user-facing text: {}",
                user_facing_failure(Phase::Apply, &e)
            ));
            false
        }
        Ok(Ok(outcome)) => {
            for r in &outcome.effects {
                cx.info(format!("apply verdict {}: {:?}", r.effect, r.kind));
            }
            cx.info(format!(
                "apply: verified, state {:?}, whole apply {wall_ms} ms",
                outcome.status.state
            ));
            true
        }
    };
    Ok(Applied {
        verified,
        batches,
        wall_ms,
    })
}

fn restore(cx: &Ctx, tweak: &'static Tweak) -> bool {
    cx.info("restore: through the restore_tweak path");
    arm_batches();
    let t0 = Instant::now();
    let result = cx.host.restore(tweak);
    let wall_ms = t0.elapsed().as_millis();
    let batches = take_batches();
    log_batches(cx, "restore", &batches);
    match result {
        Err(e) => {
            cx.error(format!("restore: refused: {}", errors::app(&e)));
            false
        }
        Ok(Err(e)) => {
            cx.error(format!("restore: failed after {wall_ms} ms"));
            for line in errors::engine(&e) {
                cx.error(format!("  {line}"));
            }
            cx.error(format!(
                "restore: user-facing text: {}",
                user_facing_failure(Phase::Restore, &e)
            ));
            false
        }
        Ok(Ok(o)) => {
            cx.info(format!(
                "restore: verified in {wall_ms} ms, state {:?}, consumed {:?}, reboot advisory {}, skipped invalid entries {}",
                o.status.state,
                o.consumed,
                o.reboot_advisory,
                o.skipped_invalid.len()
            ));
            true
        }
    }
}

/// Reads the surface again and logs every effect as expected vs actual.
fn compare_with_baseline(
    cx: &Ctx,
    tweak: &Tweak,
    baseline: &[Reading],
    label: &str,
) -> Vec<Difference> {
    let now = read_surface(cx, tweak, label);
    let diffs = compare(baseline, &now);
    for b in baseline {
        match diffs.iter().find(|d| d.effect == b.effect) {
            Some(d) => cx.error(format!("compare {d}: DIFFERS")),
            None => cx.info(format!(
                "compare {}: expected {}, actual {}: match",
                b.effect,
                show(&b.value),
                show(&b.value)
            )),
        }
    }
    diffs
}

fn restore_failed(restored: bool, diffs: &[Difference], extra: Vec<String>) -> Verdict {
    let why = match (restored, diffs.len()) {
        (false, 0) => "the restore did not verify".to_string(),
        (false, n) => {
            format!("the restore did not verify and {n} effect(s) differ from the baseline")
        }
        (true, n) => format!("the restore verified but {n} effect(s) differ from the baseline"),
    };
    let mut details: Vec<String> = diffs.iter().map(ToString::to_string).collect();
    details.push(RESTORE_HELP.into());
    details.extend(extra);
    Verdict::fail(format!("RESTORE FAILED: {why}. {RESTORE_HELP}")).with_details(details)
}

/// The engine rolled a failed apply back itself; this checks what it left.
fn apply_failed(cx: &Ctx, tweak: &Tweak, baseline: &[Reading]) -> Verdict {
    let diffs = compare_with_baseline(cx, tweak, baseline, "after the failed apply");
    let attention = detect::attention(TWEAK_ID, &build_deps(cx.host.engine()));
    cx.info(format!(
        "after the failed apply: needs attention = {}",
        attention.is_some()
    ));
    if diffs.is_empty() && attention.is_none() {
        return Verdict::fail(
            "Apply failed; the engine rolled it back and every effect matches the baseline. The error chain is in the log.",
        );
    }
    restore_failed(
        false,
        &diffs,
        vec![
            "The apply failed and its rollback did not return every effect to the baseline.".into(),
        ],
    )
}

pub fn baseline(cx: &Ctx) -> Verdict {
    let tweak = match target(cx) {
        Ok(t) => t,
        Err(v) => return v,
    };
    let readings = read_surface(cx, tweak, "baseline");
    let unreadable = readings.iter().filter(|r| r.value.is_err()).count();
    let details = readings
        .iter()
        .map(|r| format!("{} = {}", r.effect, show(&r.value)))
        .collect();
    *REMEMBERED.lock().unwrap_or_else(|p| p.into_inner()) = Some(readings.clone());
    Verdict::info(format!(
        "{} effects read, {unreadable} unreadable; kept as the baseline",
        readings.len()
    ))
    .with_details(details)
}

pub fn ti_batch_timing(cx: &Ctx) -> Verdict {
    let tweak = match target(cx) {
        Ok(t) => t,
        Err(v) => return v,
    };
    let baseline = match capture_baseline(cx, tweak) {
        Ok(b) => b,
        Err(v) => return v,
    };
    let applied = match apply(cx, tweak) {
        Ok(a) => a,
        Err(v) => return v,
    };
    if !applied.verified {
        return apply_failed(cx, tweak, &baseline);
    }
    read_surface(cx, tweak, "applied");
    let restored = restore(cx, tweak);
    let diffs = compare_with_baseline(cx, tweak, &baseline, "after restore");
    if !restored || !diffs.is_empty() {
        return restore_failed(restored, &diffs, Vec::new());
    }
    let mut details: Vec<String> = applied
        .batches
        .iter()
        .enumerate()
        .map(|(i, b)| batch_line(i, b))
        .collect();
    details.push(format!("Note: {BATCH_WINDOW}."));
    let slowest = applied.batches.iter().map(|b| b.elapsed_ms).max();
    Verdict::pass(format!(
        "Apply and restore verified, every effect matches the baseline. {} elevated batch(es) on apply, slowest {} of the {TI_BUDGET_MS} ms broker wait; whole apply {} ms.",
        applied.batches.len(),
        slowest.map_or_else(|| "none".into(), |ms| format!("{ms} ms")),
        applied.wall_ms
    ))
    .with_details(details)
}

pub fn waasmedic_watch(cx: &Ctx) -> Verdict {
    let tweak = match target(cx) {
        Ok(t) => t,
        Err(v) => return v,
    };
    let baseline = match capture_baseline(cx, tweak) {
        Ok(b) => b,
        Err(v) => return v,
    };
    let applied = match apply(cx, tweak) {
        Ok(a) => a,
        Err(v) => return v,
    };
    if !applied.verified {
        return apply_failed(cx, tweak, &baseline);
    }
    let reference = read_surface(cx, tweak, "applied");
    let minutes = cx.minutes;
    cx.info(format!(
        "watch: polling every {} s for {minutes} min; Cancel ends the watch early and still restores",
        POLL.as_secs()
    ));
    let watch = Instant::now();
    let mut drift: Vec<String> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut polls = 0;
    for i in 1..=minutes {
        if !cx.sleep(POLL) {
            cx.info(format!(
                "watch: cancelled after {} s",
                watch.elapsed().as_secs()
            ));
            break;
        }
        polls = i;
        let now = read_surface(cx, tweak, &format!("poll {i}/{minutes}"));
        let at = chrono::Local::now().format("%H:%M:%S");
        for d in compare(&reference, &now) {
            let line = format!(
                "{} at {at} (+{} s): applied {}, now {}",
                d.effect,
                watch.elapsed().as_secs(),
                d.expected,
                d.actual
            );
            cx.error(format!("DRIFT {line}"));
            let key = (d.effect, d.actual);
            if !seen.contains(&key) {
                seen.push(key);
                drift.push(line);
            }
        }
        cx.info(format!(
            "poll {i}/{minutes}: {} drifted effect(s) so far",
            drift.len()
        ));
    }
    let restored = restore(cx, tweak);
    let diffs = compare_with_baseline(cx, tweak, &baseline, "after restore");
    if !restored || !diffs.is_empty() {
        let mut extra = vec![format!("Drift seen during the watch: {}", drift.len())];
        extra.extend(drift);
        return restore_failed(restored, &diffs, extra);
    }
    if drift.is_empty() {
        Verdict::info(format!(
            "No drift in {polls} poll(s); restore verified and every effect matches the baseline."
        ))
    } else {
        Verdict::info(format!(
            "Drift found: {} change(s) away from the applied value in {polls} poll(s); restore verified and every effect matches the baseline.",
            drift.len()
        ))
        .with_details(drift)
    }
}

fn is_access_denied(e: &Error) -> bool {
    match e {
        Error::Win32 { code, .. } => *code == win32::ACCESS_DENIED || *code == 0x8007_0005,
        Error::RegistryAccessDenied(_) | Error::RequiresAdmin => true,
        _ => false,
    }
}

pub fn waasmedic_task_read(cx: &Ctx) -> Verdict {
    cx.info(format!(
        "elevated = {}",
        system_info_service::is_running_as_admin()
    ));
    let mut denied = Vec::new();
    let mut failed = Vec::new();
    let mut details = Vec::new();
    for path in [
        r"\Microsoft\Windows\WaaSMedic\PerformRemediation",
        r"\Microsoft\Windows\WaaSMedic\DeferredWork",
    ] {
        let Some((folder, name)) = path.rsplit_once('\\') else {
            continue;
        };
        let t0 = Instant::now();
        match scheduler_service::get_task_state(folder, name) {
            Ok(state) => {
                let line = format!("{path}: {state:?}");
                cx.info(format!("{line} ({} ms)", t0.elapsed().as_millis()));
                details.push(line);
            }
            Err(e) => {
                let line = format!("{path}: {}", errors::app(&e));
                cx.error(format!("{line} ({} ms)", t0.elapsed().as_millis()));
                details.push(line);
                if is_access_denied(&e) {
                    denied.push(path);
                } else {
                    failed.push(path);
                }
            }
        }
    }
    if !denied.is_empty() {
        Verdict::fail(format!("Access denied reading {}", denied.join(", "))).with_details(details)
    } else if !failed.is_empty() {
        Verdict::fail(format!("Could not read {}", failed.join(", "))).with_details(details)
    } else {
        Verdict::pass("Both WaaSMedic tasks read without access denied.").with_details(details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::StartupType;

    fn r(effect: &str, value: Result<Value, &str>) -> Reading {
        Reading {
            effect: effect.into(),
            value: value.map_err(Into::into),
        }
    }

    #[test]
    fn an_identical_reading_has_no_difference() {
        let a = vec![r("svc", Ok(Value::Startup(StartupType::Manual)))];
        assert!(compare(&a, &a.clone()).is_empty());
    }

    #[test]
    fn every_difference_is_reported_once_per_effect() {
        let baseline = vec![
            r("svc", Ok(Value::Startup(StartupType::Manual))),
            r("task", Ok(Value::TaskEnabled(true))),
            r("gone", Ok(Value::Missing)),
            r("same", Ok(Value::Absent)),
        ];
        let now = vec![
            r("svc", Ok(Value::Startup(StartupType::Disabled))),
            r("task", Err("KindError::AccessDenied")),
            r("same", Ok(Value::Absent)),
            r("extra", Ok(Value::Absent)),
        ];
        let effects: Vec<String> = compare(&baseline, &now)
            .into_iter()
            .map(|d| d.effect)
            .collect();
        assert_eq!(effects, ["svc", "task", "gone", "extra"]);
    }

    #[test]
    fn an_unreadable_baseline_never_matches() {
        let a = vec![r("x", Err("denied"))];
        assert_eq!(compare(&a, &a.clone()).len(), 1);
    }

    #[test]
    fn each_precondition_refuses_on_its_own() {
        assert_eq!(refusal(true, 0, false, false), None);
        assert!(refusal(false, 0, false, false)
            .unwrap()
            .contains("not elevated"));
        assert!(refusal(true, 1, false, false)
            .unwrap()
            .contains("1 snapshot entry"));
        assert!(refusal(true, 2, false, false)
            .unwrap()
            .contains("2 snapshot entries"));
        assert!(refusal(true, 0, true, false)
            .unwrap()
            .contains("Needs Attention"));
        assert!(refusal(true, 0, false, true)
            .unwrap()
            .contains("being changed"));
    }

    #[test]
    fn a_denied_task_read_is_told_apart_from_other_failures() {
        assert!(is_access_denied(&Error::win32("x", 0x8007_0005)));
        assert!(is_access_denied(&Error::win32("x", win32::ACCESS_DENIED)));
        assert!(!is_access_denied(&Error::win32("x", 0x8007_0002)));
    }
}
