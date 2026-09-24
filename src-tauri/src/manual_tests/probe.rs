//! Parent-side spawn/transport probes: each changes only what the parent does around the production
//! spawn (its privilege, the child's environment and transport) and reads back what it saw; the
//! elevated child gets no test code. Thread-local, so a spawn from another view never sees a card's
//! probe; [`carry`] hands it across the command layer's `spawn_blocking` hop.

use std::cell::RefCell;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub(crate) struct JobObservation {
    pub in_job: bool,
    /// `Some(GetLastError)` when the `IsProcessInJob` query itself failed.
    pub query_error: Option<u32>,
}

#[derive(Default, Clone)]
struct Probe {
    skip_debug_privilege: bool,
    system_only_env: bool,
    transport_dir: Option<PathBuf>,
    capture_job: bool,
    job: Option<JobObservation>,
}

thread_local! {
    static PROBE: RefCell<Probe> = RefCell::default();
}

fn with<T>(f: impl FnOnce(&mut Probe) -> T) -> T {
    PROBE.with_borrow_mut(f)
}

pub(crate) fn skip_debug_privilege() -> bool {
    with(|p| p.skip_debug_privilege)
}

pub(crate) fn system_only_env() -> bool {
    with(|p| p.system_only_env)
}

pub(crate) fn transport_dir() -> Option<PathBuf> {
    with(|p| p.transport_dir.clone())
}

pub(crate) fn capture_job() -> bool {
    with(|p| p.capture_job)
}

pub(crate) fn record_job(in_job: bool, query_error: Option<u32>) {
    with(|p| {
        p.job = Some(JobObservation {
            in_job,
            query_error,
        })
    });
}

/// Runs `work` under the calling thread's probe, wherever `work` ends up running.
pub(crate) fn carry<T, F>(work: F) -> impl FnOnce() -> T + Send + 'static
where
    F: FnOnce() -> T + Send + 'static,
{
    let probe = with(|p| p.clone());
    move || {
        let _installed = install(probe);
        work()
    }
}

pub(super) fn reset() {
    with(|p| *p = Probe::default());
}

pub(super) fn observed_job() -> Option<JobObservation> {
    with(|p| p.job)
}

/// Resets the probe on drop, so a card's change never outlives its own scope even if the card
/// returns early. A release-profile panic aborts before Drop runs, but that also ends the process.
pub(super) struct ProbeGuard;

impl Drop for ProbeGuard {
    fn drop(&mut self) {
        reset();
    }
}

fn install(probe: Probe) -> ProbeGuard {
    with(|p| *p = probe);
    ProbeGuard
}

fn arm(set: impl FnOnce(&mut Probe)) -> ProbeGuard {
    let mut probe = Probe::default();
    set(&mut probe);
    install(probe)
}

pub(super) fn arm_skip_debug_privilege() -> ProbeGuard {
    arm(|p| p.skip_debug_privilege = true)
}

pub(super) fn arm_capture_job() -> ProbeGuard {
    arm(|p| p.capture_job = true)
}

pub(super) fn arm_system_only_env() -> ProbeGuard {
    arm(|p| p.system_only_env = true)
}

pub(super) fn arm_transport_dir(dir: PathBuf) -> ProbeGuard {
    arm(|p| p.transport_dir = Some(dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arming_a_probe_sets_exactly_one_knob_and_the_guard_clears_it() {
        {
            let _g = arm_skip_debug_privilege();
            assert!(skip_debug_privilege());
            assert!(!system_only_env() && !capture_job() && transport_dir().is_none());
        }
        assert!(
            !skip_debug_privilege(),
            "the guard must clear the knob on drop"
        );
    }

    #[test]
    fn a_job_observation_round_trips() {
        let _g = arm_capture_job();
        assert!(observed_job().is_none());
        record_job(true, None);
        let seen = observed_job().expect("recorded");
        assert!(seen.in_job && seen.query_error.is_none());
    }

    #[test]
    fn a_probe_is_invisible_to_other_threads_unless_carried() {
        let _g = arm_system_only_env();
        assert!(!std::thread::spawn(system_only_env).join().unwrap());
        let carried = carry(system_only_env);
        assert!(std::thread::spawn(carried).join().unwrap());
    }
}
