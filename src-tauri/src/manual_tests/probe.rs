//! Parent-side spawn/transport probes for the four real-machine review items (A3.4, F60, F62, C5).
//! Each probe changes only what the parent does around the production spawn (its privilege, the
//! child's environment and transport) and reads back what it saw; the elevated child gets no test
//! code. A card arms one probe, drives a real or no-op elevated batch, then disarms; the runner
//! serializes tests and [`reset`]s this slot before each run, so no card inherits another's.

use std::path::PathBuf;
use std::sync::Mutex;

/// What the F60 spawn hook saw when it asked whether the child is in a job object.
#[derive(Clone, Copy)]
pub(crate) struct JobObservation {
    pub in_job: bool,
    /// `Some(GetLastError)` when the `IsProcessInJob` query itself failed.
    pub query_error: Option<u32>,
}

#[derive(Default)]
struct Probe {
    disable_debug_privilege: bool,
    system_only_env: bool,
    transport_dir: Option<PathBuf>,
    capture_job: bool,
    job: Option<JobObservation>,
}

static PROBE: Mutex<Probe> = Mutex::new(Probe {
    disable_debug_privilege: false,
    system_only_env: false,
    transport_dir: None,
    capture_job: false,
    job: None,
});

fn lock() -> std::sync::MutexGuard<'static, Probe> {
    PROBE.lock().unwrap_or_else(|p| p.into_inner())
}

// Read by the gated hooks in `services::elevation`.

pub(crate) fn disable_debug_privilege() -> bool {
    lock().disable_debug_privilege
}

pub(crate) fn system_only_env() -> bool {
    lock().system_only_env
}

pub(crate) fn transport_dir() -> Option<PathBuf> {
    lock().transport_dir.clone()
}

pub(crate) fn capture_job() -> bool {
    lock().capture_job
}

pub(crate) fn record_job(in_job: bool, query_error: Option<u32>) {
    lock().job = Some(JobObservation {
        in_job,
        query_error,
    });
}

// Armed and read by the tests in `cases`; `reset` also runs before every test in `runner::execute`.

pub(super) fn reset() {
    *lock() = Probe::default();
}

pub(super) fn observed_job() -> Option<JobObservation> {
    lock().job
}

/// Resets the probe on drop, so a card's change never outlives its own scope even if the card
/// returns early. A release-profile panic aborts before Drop runs, but that also ends the process.
pub(super) struct ProbeGuard;

impl Drop for ProbeGuard {
    fn drop(&mut self) {
        reset();
    }
}

pub(super) fn arm_disable_debug_privilege() -> ProbeGuard {
    lock().disable_debug_privilege = true;
    ProbeGuard
}

pub(super) fn arm_capture_job() -> ProbeGuard {
    let mut p = lock();
    p.capture_job = true;
    p.job = None;
    ProbeGuard
}

pub(super) fn arm_system_only_env() -> ProbeGuard {
    lock().system_only_env = true;
    ProbeGuard
}

pub(super) fn arm_transport_dir(dir: PathBuf) -> ProbeGuard {
    lock().transport_dir = Some(dir);
    ProbeGuard
}

/// A `Name=Value\0...\0\0` UTF-16 block for `CreateProcessW` with `CREATE_UNICODE_ENVIRONMENT`.
fn env_block_from(pairs: &[(&str, String)]) -> Vec<u16> {
    let mut block = Vec::new();
    for (k, v) in pairs {
        block.extend(format!("{k}={v}").encode_utf16());
        block.push(0);
    }
    block.push(0);
    if block.len() == 1 {
        block.push(0); // an empty block is still the two-NUL terminator
    }
    block
}

/// C5: a minimal machine-only environment, dropping the user's profile and `HKCU\Environment` while
/// keeping what the Task Scheduler COM calls need (a System32 `Path`, `SystemRoot`, `ComSpec`).
pub(crate) fn system_env_block(system32: &str, windows: &str) -> Vec<u16> {
    let system32 = system32.trim_end_matches('\\');
    let windows = windows.trim_end_matches('\\');
    let drive = windows.get(..2).unwrap_or("C:");
    let path = format!("{system32};{windows};{system32}\\Wbem;{system32}\\WindowsPowerShell\\v1.0");
    let temp = format!("{windows}\\Temp"); // the SYSTEM account's real temp, always present
    let pairs: [(&str, String); 9] = [
        ("SystemRoot", windows.to_string()),
        ("windir", windows.to_string()),
        ("SystemDrive", drive.to_string()),
        ("Path", path),
        (
            "PATHEXT",
            ".COM;.EXE;.BAT;.CMD;.VBS;.JS;.WSF;.MSC".to_string(),
        ),
        ("ComSpec", format!("{system32}\\cmd.exe")),
        ("TEMP", temp.clone()),
        ("TMP", temp),
        ("OS", "Windows_NT".to_string()),
    ];
    env_block_from(&pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(block: &[u16]) -> String {
        String::from_utf16_lossy(block).replace('\u{0}', "|")
    }

    #[test]
    fn an_env_block_is_double_null_terminated() {
        let block = env_block_from(&[("A", "1".into()), ("B", "2".into())]);
        assert_eq!(decode(&block), "A=1|B=2||");
        assert_eq!(block.last(), Some(&0));
        assert_eq!(block[block.len() - 2], 0);
    }

    #[test]
    fn an_empty_env_block_is_still_two_nulls() {
        assert_eq!(env_block_from(&[]), vec![0, 0]);
    }

    #[test]
    fn the_system_env_has_a_system32_only_path_and_no_user_profile() {
        let block = decode(&system_env_block(r"C:\Windows\System32", r"C:\Windows"));
        assert!(
            block.contains(r"Path=C:\Windows\System32;C:\Windows;"),
            "{block}"
        );
        assert!(block.contains(r"SystemRoot=C:\Windows|"), "{block}");
        assert!(
            block.contains(r"ComSpec=C:\Windows\System32\cmd.exe"),
            "{block}"
        );
        assert!(block.contains(r"TEMP=C:\Windows\Temp"), "{block}");
        assert!(!block.to_lowercase().contains("users"), "{block}");
        assert!(!block.to_lowercase().contains("userprofile"), "{block}");
    }

    #[test]
    fn the_system_env_tolerates_a_trailing_separator_and_a_non_c_drive() {
        let block = decode(&system_env_block(r"D:\Windows\System32\", r"D:\Windows\"));
        assert!(block.contains(r"SystemDrive=D:|"), "{block}");
        assert!(block.contains(r"SystemRoot=D:\Windows|"), "{block}");
        assert!(!block.contains(r"System32\\"), "{block}");
    }

    #[test]
    fn arming_a_probe_sets_exactly_one_knob_and_the_guard_clears_it() {
        reset();
        {
            let _g = arm_disable_debug_privilege();
            assert!(disable_debug_privilege());
            assert!(!system_only_env() && !capture_job() && transport_dir().is_none());
        }
        assert!(
            !disable_debug_privilege(),
            "the guard must clear the knob on drop"
        );
    }

    #[test]
    fn a_job_observation_round_trips() {
        reset();
        let _g = arm_capture_job();
        assert!(observed_job().is_none());
        record_job(true, None);
        let seen = observed_job().expect("recorded");
        assert!(seen.in_job && seen.query_error.is_none());
        reset();
    }
}
