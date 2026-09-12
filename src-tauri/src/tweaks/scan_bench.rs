//! Timing harness for the corpus-wide detect sweep.
//!
//! Wraps the real `EffectKind`/`ProbeSource` seams in a decorator that records call count and
//! elapsed time per work category, so the production path is measured exactly as it runs. Nothing
//! in `detect.rs` is modified or mocked.
//!
//! Run with: `cargo test --release scan_sweep_timing -- --ignored --nocapture`

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::tweaks::engine::{ActionRunner, AllKinds, Deps, ProbeCache, ProbeSource, RealProbe};
use crate::tweaks::kinds::{EffectKind, Error as KindError, ExecCx};
use crate::tweaks::model::{ActionDef, Setting, Value};

#[derive(Default, Clone, Copy)]
struct Stat {
    calls: u32,
    total: Duration,
    max: Duration,
}

#[derive(Default)]
struct Stats(Mutex<HashMap<&'static str, Stat>>);

impl Stats {
    fn record(&self, label: &'static str, elapsed: Duration) {
        let mut map = self.0.lock().unwrap();
        let e = map.entry(label).or_default();
        e.calls += 1;
        e.total += elapsed;
        e.max = e.max.max(elapsed);
    }

    fn rows(&self) -> Vec<(&'static str, Stat)> {
        let mut rows: Vec<_> = self
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        rows.sort_by_key(|(_, s)| std::cmp::Reverse(s.total));
        rows
    }
}

fn setting_label(s: &Setting) -> &'static str {
    match s {
        Setting::Registry(_) => "registry value",
        Setting::RegistryKey(_) => "registry key",
        Setting::Service(_) => "service (SCM)",
        Setting::Task(_) => "scheduled task (COM)",
        Setting::Hosts(_) => "hosts file",
        Setting::Firewall(_) => "firewall",
    }
}

struct TimedKinds<'a>(&'a Stats);

impl EffectKind for TimedKinds<'_> {
    fn read(&self, s: &Setting, cx: &ExecCx) -> Result<Value, KindError> {
        let t = Instant::now();
        let r = AllKinds.read(s, cx);
        self.0.record(setting_label(s), t.elapsed());
        r
    }
    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), KindError> {
        AllKinds.drive(s, target, cx)
    }
}

struct TimedProbes<'a>(&'a Stats);

impl ProbeSource for TimedProbes<'_> {
    fn probe(&self, action: &ActionDef, cx: &ExecCx) -> Result<bool, KindError> {
        let t = Instant::now();
        let r = RealProbe.probe(action, cx);
        self.0
            .record("action probe (spawns a process)", t.elapsed());
        r
    }
}

struct NoActions;
impl ActionRunner for NoActions {
    fn apply(&self, _: &ActionDef, _: &ExecCx) -> Result<(), KindError> {
        unreachable!("detect never applies")
    }
    fn undo(&self, _: &ActionDef, _: &ExecCx) -> Result<(), KindError> {
        unreachable!("detect never undoes")
    }
}

#[test]
#[ignore = "timing harness: touches the live registry/SCM/COM. Run with --ignored --nocapture"]
fn scan_sweep_timing() {
    use crate::services::system_info_service;
    use crate::tweaks::compiled_corpus;
    use crate::tweaks::engine::detect;
    use crate::tweaks::model::Level;
    use crate::tweaks::shared_claims::ClaimsStore;
    use crate::tweaks::snapshot::SnapshotStore;
    use crate::tweaks::winver::running_winver;

    let stats = Stats::default();
    let kinds = TimedKinds(&stats);
    let probes = TimedProbes(&stats);
    let claims = ClaimsStore::open_default().expect("claims store");
    let snapshots = SnapshotStore::open_default().expect("snapshot store");
    let probe_cache = ProbeCache::new();

    let deps = Deps {
        kinds: &kinds,
        probes: &probes,
        actions: &NoActions,
        claims: &claims,
        snapshots: &snapshots,
        probe_cache: &probe_cache,
        machine_guid: None,
        level: if system_info_service::is_running_as_admin() {
            Level::Admin
        } else {
            Level::User
        },
        running: running_winver(),
    };

    let corpus = compiled_corpus();
    let mut per_tweak: Vec<(String, Duration)> = Vec::with_capacity(corpus.tweaks.len());

    // Serial first, so per-category attribution is not distorted by workers overlapping.
    let sweep = Instant::now();
    for tweak in &corpus.tweaks {
        let t = Instant::now();
        let _ = detect::detect(tweak, corpus, &deps);
        per_tweak.push((tweak.id.clone(), t.elapsed()));
    }
    let total = sweep.elapsed();

    // Snapshot the per-category attribution BEFORE the parallel pass: the two passes share one
    // `Stats`, and letting the second one accumulate into it would double every count and make the
    // totals exceed the serial wall-clock they are meant to explain.
    let categories = stats.rows();

    // Then the shape production actually runs. A warm ProbeCache would make this meaningless, so
    // it gets a cold one; everything else is shared with the serial pass above.
    let cold_cache = ProbeCache::new();
    let par_deps = Deps {
        probe_cache: &cold_cache,
        ..deps
    };
    let par = Instant::now();
    corpus.tweaks.par_iter().for_each(|tweak| {
        let _ = detect::detect(tweak, corpus, &par_deps);
    });
    let par_total = par.elapsed();

    println!(
        "\n===== SWEEP: {} tweaks =====\n  serial:   {:.2?}\n  parallel: {:.2?}  ({:.1}x on {} threads)\n",
        corpus.tweaks.len(),
        total,
        par_total,
        total.as_secs_f64() / par_total.as_secs_f64(),
        rayon::current_num_threads(),
    );

    println!("--- by work category (what the time is actually spent on) ---");
    println!(
        "{:<34} {:>7} {:>11} {:>10} {:>10}",
        "category", "calls", "total", "mean", "max"
    );
    for (label, s) in &categories {
        println!(
            "{:<34} {:>7} {:>11.2?} {:>10.2?} {:>10.2?}",
            label,
            s.calls,
            s.total,
            s.total / s.calls.max(1),
            s.max
        );
    }
    let measured: Duration = categories.iter().map(|(_, s)| s.total).sum();
    println!(
        "\nmeasured in the seams: {:.2?} of {:.2?} ({:.0}%); the rest is pure engine logic",
        measured,
        total,
        measured.as_secs_f64() / total.as_secs_f64() * 100.0
    );

    per_tweak.sort_by_key(|(_, d)| std::cmp::Reverse(*d));
    println!("\n--- slowest 15 tweaks ---");
    for (id, d) in per_tweak.iter().take(15) {
        println!("{:>10.2?}  {id}", d);
    }
    let slow: Duration = per_tweak.iter().take(15).map(|(_, d)| *d).sum();
    println!(
        "\ntop 15 of {} tweaks account for {:.2?} ({:.0}% of the sweep)",
        per_tweak.len(),
        slow,
        slow.as_secs_f64() / total.as_secs_f64() * 100.0
    );
}
