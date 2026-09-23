use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::runner::{arm_batches, take_batches, BatchRecord, Ctx, Verdict};
use super::{errors, probe};
use crate::commands::tweaks::{build_deps, find_tweak};
use crate::error::{win32, Error};
use crate::services::elevation::{
    run_ops, set_debug_privilege, windows_dir, AcquireReason, BrokerOpError, Elevation,
};
use crate::services::exclusive_temp::ExclusiveTempFile;
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
    snapshot_kept(format!("RESTORE FAILED: {why}"), diffs, extra)
}

fn snapshot_kept(headline: String, diffs: &[Difference], extra: Vec<String>) -> Verdict {
    let mut details: Vec<String> = diffs.iter().map(ToString::to_string).collect();
    details.push(RESTORE_HELP.into());
    details.extend(extra);
    Verdict::fail(format!("{headline}. {RESTORE_HELP}")).with_details(details)
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
    rollback_failed(&diffs)
}

fn rollback_failed(diffs: &[Difference]) -> Verdict {
    let why = match diffs.len() {
        0 => "the apply failed and its rollback left the tweak needing attention".to_string(),
        n => format!(
            "the apply failed and its rollback left {n} effect(s) differing from the baseline"
        ),
    };
    snapshot_kept(format!("APPLY ROLLBACK FAILED: {why}"), diffs, Vec::new())
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
    apply_and_restore(cx, || Ok(None))
}

/// Baseline, apply, restore, compare. `arm` runs once the preconditions pass and its probe covers
/// the apply only.
fn apply_and_restore(
    cx: &Ctx,
    arm: impl FnOnce() -> Result<Option<probe::ProbeGuard>, Verdict>,
) -> Verdict {
    let tweak = match target(cx) {
        Ok(t) => t,
        Err(v) => return v,
    };
    let baseline = match capture_baseline(cx, tweak) {
        Ok(b) => b,
        Err(v) => return v,
    };
    let applied = match arm() {
        Ok(_armed) => apply(cx, tweak),
        Err(v) => return v,
    };
    let applied = match applied {
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

fn require_elevated(cx: &Ctx) -> Option<Verdict> {
    let elevated = system_info_service::is_running_as_admin();
    cx.info(format!("elevated = {elevated}"));
    (!elevated)
        .then(|| Verdict::fail("Refused: the app is not elevated; restart it as administrator."))
}

/// The candidate transport directory only SYSTEM and Administrators own.
fn systemtemp_dir() -> Result<PathBuf, Error> {
    windows_dir().map(|w| w.join("SystemTemp"))
}

fn probe_admin_can_write(dir: &Path) -> std::io::Result<()> {
    ExclusiveTempFile::create_in(dir, "magicx-f62", "probe", "f62 probe", b"probe").map(|_| ())
}

/// The verdict from a no-op spawn made with `SeDebugPrivilege` disabled. Only a failure to open the
/// TrustedInstaller process (`TiProcessUnverified`) points at the privilege; other failures do not.
fn interpret_debug_privilege(outcome: &Result<(), BrokerOpError>) -> Verdict {
    match outcome {
        Ok(()) => Verdict::info(
            "SeDebugPrivilege was NOT needed: the TrustedInstaller child spawned with it disabled. The enabling call and its failure reason could be removed.",
        ),
        Err(BrokerOpError::CouldNotAcquire(AcquireReason::TiProcessUnverified, e)) => {
            // TiProcessUnverified is access-denied on the open (privilege needed) OR a pid/image
            // mismatch (unrelated); the two are not typed apart, so present the detail and both.
            Verdict::info(format!(
                "Opening the TrustedInstaller process failed with SeDebugPrivilege disabled: {}. Access denied here means the privilege is required, so keep the enabling call; a pid or image mismatch is unrelated, so re-run the card.",
                errors::app(e)
            ))
        }
        Err(BrokerOpError::CouldNotAcquire(reason, e)) => Verdict::fail(format!(
            "Inconclusive: the no-op spawn could not acquire the child for another reason ({reason}: {}).",
            errors::app(e)
        )),
        Err(e) => Verdict::fail(format!("Inconclusive: the no-op batch failed unexpectedly ({e}).")),
    }
}

pub fn debug_privilege_needed(cx: &Ctx) -> Verdict {
    if let Some(v) = require_elevated(cx) {
        return v;
    }
    let was_enabled = match set_debug_privilege(false) {
        Ok(changed) => changed,
        Err(e) => {
            return Verdict::fail(format!(
                "Could not disable SeDebugPrivilege: {}",
                errors::app(&e)
            ))
        }
    };
    cx.info(format!(
        "SeDebugPrivilege enabled before this test = {was_enabled}"
    ));
    cx.info("spawning a no-op TrustedInstaller batch with SeDebugPrivilege disabled");
    let outcome = {
        let _probe = probe::arm_skip_debug_privilege();
        run_ops(Elevation::TrustedInstaller, Vec::new())
    };
    // The token is process-wide: an elevated spawn from elsewhere in the app re-enables it.
    let reenabled = !matches!(set_debug_privilege(false), Ok(false));
    if was_enabled {
        if let Err(e) = set_debug_privilege(true) {
            cx.error(format!(
                "could not re-enable SeDebugPrivilege: {}",
                errors::app(&e)
            ));
        }
    }
    match &outcome {
        Ok(()) => cx.info("no-op spawn returned Ok: the child ran with the privilege disabled"),
        Err(e) => cx.error(format!("no-op spawn returned: {e}")),
    }
    if reenabled {
        return Verdict::fail(
            "Inconclusive: SeDebugPrivilege did not stay disabled during the spawn (another elevated change in the app re-enables it). Re-run the test.",
        );
    }
    interpret_debug_privilege(&outcome)
}

pub fn child_job_object(cx: &Ctx) -> Verdict {
    if let Some(v) = require_elevated(cx) {
        return v;
    }
    cx.info("spawning a no-op TrustedInstaller batch to inspect its job membership");
    let (outcome, job) = {
        let _probe = probe::arm_capture_job();
        let outcome = run_ops(Elevation::TrustedInstaller, Vec::new());
        (outcome, probe::observed_job())
    };
    if let Err(e) = &outcome {
        cx.error(format!("no-op spawn returned: {e}"));
    }
    // The observation is taken before the child is reaped, so it stands even if the batch later
    // errored; only fall back to the spawn error when there is no observation to report.
    let Some(o) = job else {
        return match &outcome {
            Err(e) => Verdict::fail(format!(
                "Could not spawn the elevated child to inspect it: {e}"
            )),
            Ok(()) => {
                Verdict::fail("The spawn reported no job membership; the capture hook did not run.")
            }
        };
    };
    if let Some(code) = o.query_error {
        return Verdict::fail(format!("IsProcessInJob failed (Win32 {code})."));
    }
    let caveat = if outcome.is_err() {
        " (the no-op batch itself did not complete cleanly; see the log)"
    } else {
        ""
    };
    if o.in_job {
        Verdict::info(format!(
            "The broker child IS in a job object. Check whether its limits (for example kill-on-close) could end a batch early before adding a job of our own; see review F60.{caveat}"
        ))
    } else {
        Verdict::info(format!(
            "The broker child is NOT in a job object; it does not inherit TrustedInstaller's job. F60 is settled.{caveat}"
        ))
    }
}

pub fn system_only_environment(cx: &Ctx) -> Verdict {
    let mut verdict = apply_and_restore(cx, || {
        cx.info("applying under a system-only environment block; the scheduler COM calls must still work");
        Ok(Some(probe::arm_system_only_env()))
    });
    if verdict.status == super::runner::Status::Pass {
        verdict.summary = format!(
            "{} The scheduler COM calls worked without the user's environment, so a system-only block is viable (C5).",
            verdict.summary
        );
    }
    verdict
}

pub fn systemtemp_transport(cx: &Ctx) -> Verdict {
    let mut verdict = apply_and_restore(cx, || {
        let system_temp = systemtemp_dir().map_err(|e| {
            Verdict::fail(format!(
                "Could not locate the Windows folder: {}",
                errors::app(&e)
            ))
        })?;
        cx.info(format!("SystemTemp = {}", system_temp.display()));
        if !system_temp.exists() {
            return Err(Verdict::info(
                "SystemTemp does not exist on this build, so the transport cannot move here. F62 stays open.",
            ));
        }
        probe_admin_can_write(&system_temp).map_err(|e| {
            Verdict::fail(format!(
                "This process could not create a file in SystemTemp ({e}); it needs administrator rights."
            ))
        })?;
        cx.info(
            "this process can create a file in SystemTemp; routing the apply's transport there",
        );
        Ok(Some(probe::arm_transport_dir(system_temp)))
    });
    if verdict.status == super::runner::Status::Pass {
        verdict.summary = format!(
            "{} The TrustedInstaller child read and wrote in SystemTemp, so the transport can move there (F62).",
            verdict.summary
        );
    }
    verdict
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
    fn a_failed_apply_is_not_reported_as_a_failed_restore() {
        let diffs = compare(
            &[r("svc", Ok(Value::Absent))],
            &[r("svc", Ok(Value::Missing))],
        );
        for summary in [
            rollback_failed(&[]).summary,
            rollback_failed(&diffs).summary,
        ] {
            assert!(
                summary.starts_with("APPLY ROLLBACK FAILED: the apply failed"),
                "{summary}"
            );
        }
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

    #[test]
    fn a_spawn_with_the_privilege_disabled_that_works_reads_as_not_needed() {
        let v = interpret_debug_privilege(&Ok(()));
        assert!(v.summary.contains("NOT needed"), "{}", v.summary);
    }

    #[test]
    fn a_failed_ti_open_with_the_privilege_disabled_explains_both_causes() {
        let e = BrokerOpError::CouldNotAcquire(
            AcquireReason::TiProcessUnverified,
            Error::win32("open TrustedInstaller", win32::ACCESS_DENIED),
        );
        let summary = interpret_debug_privilege(&Err(e)).summary;
        assert!(summary.contains("privilege is required"), "{summary}");
        assert!(summary.contains("mismatch"), "{summary}");
    }

    #[test]
    fn a_service_failure_with_the_privilege_disabled_is_inconclusive() {
        let e = BrokerOpError::CouldNotAcquire(
            AcquireReason::TiServiceDisabled,
            Error::ServiceControl("disabled".into()),
        );
        assert!(interpret_debug_privilege(&Err(e))
            .summary
            .contains("Inconclusive"));
    }

    #[test]
    fn systemtemp_sits_under_the_real_windows_folder() {
        let dir = systemtemp_dir().expect("the Windows folder resolves");
        assert!(dir.ends_with("SystemTemp"), "{}", dir.display());
        assert!(dir.with_file_name("System32").is_dir(), "{}", dir.display());
    }
}
