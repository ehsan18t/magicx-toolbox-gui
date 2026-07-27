//! `ActionKind`: apply/undo/probe for imperative `Action` effects (spec §5.5/§7), meaning free-form
//! `cmd`/`powershell` scripts plus the one structural op, `DeleteTree`. Not an `EffectKind` impl,
//! because Actions are not `Setting`s (see `kinds/mod.rs` on the `Effect`/`Setting`/`Action` split).
//!
//! ## Level gating mirrors every other kind's read/drive split
//! `run_apply`/`run_undo` mutate state, so they [`guard_level`] and reject `System`/`Ti`: no
//! `BrokerOp` carries a script, so there is nothing to route them to. `run_probe` only observes
//! state (spec §7: "state-based, never history-based"), so like every `read` it never gates on
//! `cx.level()` and always runs in-process.
//!
//! ## No-undo / no-probe contract
//! `run_undo` without an `undo`, and `run_probe` without a `probe` (which includes every
//! `DeleteTree`, since it has no `probe` field), return `Error::Invalid`, never `Ok(())`/
//! `Ok(false)`. The engine is expected to check presence before calling; if it does not, the call
//! must still never lie about having done something.
//!
//! ## `DeleteTree` reuses the hardened registry delete, verbatim
//! `run_apply` on `DeleteTree` drives `Setting::RegistryKey(key)` to `Value::Present(false)`
//! through [`RegistryKind`]'s own `drive`, guards included, rather than reimplementing a delete.
//! Its `undo` has no `shell` field on the model, unlike `Script`, so it always runs as PowerShell,
//! which can do `reg import` and anything `cmd` can via `cmd /c`. A fixed choice, stated here
//! rather than left as a silent guess.
//!
//! ## Timeout, and the process tree it actually bounds
//! Rust's std has no built-in process timeout, so [`wait_with_timeout`] hand-rolls one: poll
//! `Child::try_wait` up to a bound, then `kill()` + `wait()` (reaping so no orphan remains) on
//! expiry — a typed [`Error::ActionExecFailed`], never a benign value (spec §14, invariant 2).
//! stdout/stderr are drained on background threads throughout (`log::debug!` only, never parsed
//! for success). `kill()` alone only reaches the immediate child's pid, though — a script that
//! spawns a detached grandchild inheriting the piped stdout/stderr handle could otherwise keep
//! that drain thread's `read_to_end` blocked well past the bound. Every child is therefore bound
//! into a fresh [`KillOnCloseJob`] (a Windows Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`)
//! right after spawn; closing it (every exit path — success, timeout, or wait-error — does, before
//! joining the drain threads) terminates the whole tree, so "bounded" is a real guarantee on
//! everything the action started, not just what we can see the pid of.
//!
//! ## Hardening the Cmd temp-script path against local tampering
//! A Cmd script must reach disk (`cmd.exe` has no `-EncodedCommand` equivalent), and `%TEMP%` is
//! writable by every process running as this user. [`ExclusiveTempFile`] is what makes that safe;
//! see its module docs for the three guards. The value is kept alive until the child has exited,
//! so the lock covers the whole execution.
//!
//! ## Encoding
//! [`base64_encode`] is local to this file. The broker had an identical copy until its only caller,
//! a PowerShell op with no producer, was removed; this is now the single one.

use std::io::Read;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

use crate::services::exclusive_temp::ExclusiveTempFile;
use crate::tweaks::model::{ActionDef, Probe, Setting, Shell, Value};

use super::registry::RegistryKind;
use super::{guard_level, EffectKind, Error, ExecCx};

/// Bounded default for every script this kind runs (spec §14: "a bounded timeout"). Generous
/// enough for a real tweak script (a service restart, `gpupdate /force`, a handful of registry
/// writes) without ever hanging an apply/detect pass indefinitely.
const ACTION_TIMEOUT: Duration = Duration::from_secs(30);

/// `Child::try_wait` polling granularity — coarse enough to be cheap, fine enough that a timeout
/// error fires close to the bound rather than one whole interval late.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `apply`/`undo`/`probe` for `ActionDef` (spec §7). Not an `EffectKind` — see the module docs.
pub struct ActionKind;

impl ActionKind {
    /// Runs `action`'s `apply`. Exit 0 is success; non-zero is a typed [`Error::ActionFailed`]
    /// (spec §7).
    pub fn run_apply(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), Error> {
        match action {
            ActionDef::Script { apply, shell, .. } => {
                guard_level(cx)?;
                run_and_require_zero(*shell, &apply.0)
            }
            ActionDef::DeleteTree { key, .. } => {
                // `RegistryKind::drive` runs its own `guard_level`, rejecting System/Ti exactly as
                // a raw `RegistryKey` effect would -- not duplicated here.
                RegistryKind.drive(
                    &Setting::RegistryKey(key.clone()),
                    &Value::Present(false),
                    cx,
                )
            }
        }
    }

    /// Runs `action`'s `undo`. Absent `undo` is a typed error, never a silent no-op (spec §7: a
    /// no-`undo` action is honestly one-way).
    pub fn run_undo(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), Error> {
        match action {
            ActionDef::Script {
                undo: Some(undo),
                shell,
                ..
            } => {
                guard_level(cx)?;
                run_and_require_zero(*shell, &undo.0)
            }
            ActionDef::Script { undo: None, .. } => Err(Error::Invalid(
                "this action has no undo script -- it is one-way (spec §7)",
            )),
            ActionDef::DeleteTree {
                undo: Some(undo), ..
            } => {
                guard_level(cx)?;
                run_and_require_zero(Shell::PowerShell, &undo.0)
            }
            ActionDef::DeleteTree { undo: None, .. } => Err(Error::Invalid(
                "this delete-tree has no undo script -- it is one-way unless the author supplies one (spec §7)",
            )),
        }
    }

    /// Reads whether `action`'s produced state is currently present (spec §7: "state-based, never
    /// history-based" — the same check apply-time did-it-work and detect-time detection share).
    /// Exit 0 = present (`Ok(true)`); non-zero = absent (`Ok(false)`). A probe that cannot be run
    /// at all — fails to spawn, or times out — is `Err`, never `Ok(false)` (invariant 2): "can't
    /// tell" must never read as "absent". Never gates on `cx`'s level — see the module docs.
    pub fn run_probe(&self, action: &ActionDef, _cx: &ExecCx) -> Result<bool, Error> {
        match action {
            ActionDef::Script {
                probe: Some(Probe::Script(body)),
                shell,
                ..
            } => Ok(run_script(*shell, &body.0, ACTION_TIMEOUT)? == 0),
            // The native probe forms never reach here: `engine::detect` answers them itself, since
            // only it holds the shared state they read. Reaching this arm is an engine routing bug.
            ActionDef::Script { probe: Some(_), .. } => Err(Error::Invalid(
                "a native probe must be answered by the engine, not run as a script",
            )),
            ActionDef::Script { probe: None, .. } => Err(Error::Invalid(
                "this action has no probe -- it never contributes to detection (spec §7)",
            )),
            ActionDef::DeleteTree { .. } => Err(Error::Invalid(
                "delete-tree has no probe by type -- it never contributes to detection (spec §7)",
            )),
        }
    }
}

fn run_and_require_zero(shell: Shell, body: &str) -> Result<(), Error> {
    match run_script(shell, body, ACTION_TIMEOUT)? {
        0 => Ok(()),
        code => Err(Error::ActionFailed(code)),
    }
}

/// Runs one script body to completion (or until `timeout` kills it), returning its raw exit code —
/// the sole, locale-independent success/failure signal (spec §7). Never interprets stdout.
fn run_script(shell: Shell, body: &str, timeout: Duration) -> Result<i32, Error> {
    match shell {
        Shell::PowerShell => wait_with_timeout(spawn_powershell(body)?, timeout),
        Shell::Cmd => {
            let file = ExclusiveTempFile::create("magicx-action", "cmd", body.as_bytes())
                .map_err(|e| Error::ActionExecFailed(format!("temp script: {e}")))?;
            wait_with_timeout(spawn_cmd(file.path())?, timeout)
            // `file` drops here, after the child has fully exited: the share-mode lock holds for
            // the whole execution and the temp `.cmd` is deleted only once cmd.exe is done.
        }
    }
}

/// Runs `body` via `powershell.exe -EncodedCommand` (base64 of UTF-16LE, spec §7): the script
/// never appears on any command line, so quotes/newlines/`$`/length carry no escaping risk.
fn spawn_powershell(body: &str) -> Result<Child, Error> {
    let utf16: Vec<u8> = body.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = base64_encode(&utf16);
    spawn_command(
        "powershell.exe",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &encoded,
        ],
    )
}

/// Runs `script_path` via `cmd.exe /c <path>`. `cmd.exe` has no `-EncodedCommand` equivalent, so
/// the escaping-free guarantee comes from a different mechanism here: the script body never
/// appears on the command line at all (only our own generated temp path does) — it lives solely in
/// the file `script_path` names.
fn spawn_cmd(script_path: &Path) -> Result<Child, Error> {
    let path = script_path.to_string_lossy();
    spawn_command("cmd.exe", &["/c", &path])
}

fn spawn_command(program: &str, args: &[&str]) -> Result<Child, Error> {
    Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::ActionExecFailed(format!("failed to spawn {program}: {e}")))
}

/// Waits for `child`, killing + reaping it if `timeout` elapses first — std has no built-in
/// process timeout (spec §14). stdout/stderr are drained concurrently on background threads
/// (`log::debug!` only, spec §14's "captured for logging") so a chatty script can never deadlock
/// against a full OS pipe buffer while the loop below only polls exit status.
fn wait_with_timeout(mut child: Child, timeout: Duration) -> Result<i32, Error> {
    // Bind `child` (and, transitively, anything it later spawns) into a fresh kill-on-close job
    // right away, before anything else -- minimizing the window in which a fast-spawning
    // grandchild could start outside the container (Fix 3). Either step failing kills the process
    // immediately rather than leaving it running unmonitored: fail closed, matching the "did-it-
    // work" contract's honest-failure principle.
    let job = match KillOnCloseJob::new() {
        Ok(job) => job,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    if let Err(e) = job.assign(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(e);
    }

    let out = child.stdout.take().map(|p| drain_to_log(p, "stdout"));
    let err = child.stderr.take().map(|p| drain_to_log(p, "stderr"));

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if start.elapsed() < timeout => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                // Timeout exceeded: kill + wait so no orphaned process remains, then report a
                // typed error -- "can't tell" must never read as a benign exit code.
                let _ = child.kill();
                let _ = child.wait();
                break Err(Error::ActionExecFailed(format!(
                    "action exceeded its {}s timeout and was terminated",
                    timeout.as_secs()
                )));
            }
            Err(e) => {
                // Symmetric with the timeout arm above: a failed wait must not leave the child
                // running/unreaped either (Fix 2).
                let _ = child.kill();
                let _ = child.wait();
                break Err(Error::ActionExecFailed(format!(
                    "failed to wait on action process: {e}"
                )));
            }
        }
    };

    // Close the job *before* joining the drain threads below, on every path: closing the last
    // handle to a JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE job terminates any descendant the immediate
    // child left behind (e.g. a detached grandchild holding the inherited stdout/stderr pipe
    // open), which is what lets that pipe finally reach EOF so the joins below can't hang past the
    // bound (Fix 3). Dropping it after the joins instead would defeat the whole point.
    drop(job);

    if let Some(t) = out {
        let _ = t.join();
    }
    if let Some(t) = err {
        let _ = t.join();
    }
    status.map(|s| s.code().unwrap_or(-1))
}

/// A Windows Job Object configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`: every process
/// assigned to it (the spawned child, and anything *it* spawns — descendants inherit job
/// membership by default) is terminated the moment the job's last handle closes, on any exit path,
/// even across a panic unwind (`Drop` runs regardless). This is what makes the bounded timeout a
/// real guarantee on the whole process tree, not just the immediate pid (Fix 3).
struct KillOnCloseJob(HANDLE);

impl KillOnCloseJob {
    fn new() -> Result<Self, Error> {
        // SAFETY: `CreateJobObjectW` is a plain FFI call; both pointer arguments are `null`
        // (anonymous job, default security), which is documented as legal. The returned handle is
        // checked for null before use.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(Error::ActionExecFailed(format!(
                "failed to create job object: {}",
                unsafe { GetLastError() }
            )));
        }

        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `job` was just created and checked non-null; `info` is a valid, correctly-sized
        // stack value of exactly the type `JobObjectExtendedLimitInformation` expects.
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(info).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            let code = unsafe { GetLastError() };
            // SAFETY: `job` is a valid handle we just created and hold the only reference to;
            // nothing has been assigned to it yet.
            unsafe { CloseHandle(job) };
            return Err(Error::ActionExecFailed(format!(
                "failed to configure job object: {code}"
            )));
        }
        Ok(Self(job))
    }

    /// Assigns `child` (and, transitively, anything it later spawns) to this job.
    fn assign(&self, child: &Child) -> Result<(), Error> {
        // SAFETY: `self.0` is a valid job handle from `new`; `child.as_raw_handle()` is a valid
        // process handle owned by `child`, alive for at least the duration of this call.
        let ok = unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as HANDLE) };
        if ok == 0 {
            return Err(Error::ActionExecFailed(format!(
                "failed to bind the action process to its job object: {}",
                unsafe { GetLastError() }
            )));
        }
        Ok(())
    }
}

impl Drop for KillOnCloseJob {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid job handle created by `new`, closed exactly once here.
        // `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` means this also terminates anything still running
        // inside the job -- the entire reason this type exists.
        unsafe { CloseHandle(self.0) };
    }
}

/// Drains a pipe to the `log` crate on a background thread -- output is captured for diagnostics
/// only, never parsed for success (spec §7/§14: the exit code is the sole signal).
fn drain_to_log(
    mut pipe: impl Read + Send + 'static,
    stream: &'static str,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        if pipe.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
            log::debug!("action {stream}: {}", String::from_utf8_lossy(&buf).trim());
        }
    })
}

/// Standard base64 (RFC 4648), small enough not to justify a dependency.
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RegistryHive;
    use crate::services::registry_service;
    use crate::tweaks::model::{Hive, KeyAddr, Level, Probe, Script};
    use std::sync::atomic::{AtomicU32, Ordering};

    static SCRATCH_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn user_cx() -> ExecCx {
        ExecCx::new(Level::User)
    }

    fn script_action(
        apply: &str,
        undo: Option<&str>,
        probe: Option<&str>,
        ephemeral: bool,
        shell: Shell,
    ) -> ActionDef {
        ActionDef::Script {
            apply: Script(apply.to_string()),
            undo: undo.map(|s| Script(s.to_string())),
            probe: probe.map(|s| Probe::Script(Script(s.to_string()))),
            ephemeral,
            shell,
        }
    }

    #[test]
    fn apply_exit0_ok_exit1_err() {
        let cx = user_cx();
        let ok = script_action("exit 0", None, None, false, Shell::PowerShell);
        ActionKind
            .run_apply(&ok, &cx)
            .expect("exit 0 must be success");

        let fail = script_action("exit 1", None, None, false, Shell::PowerShell);
        let err = ActionKind
            .run_apply(&fail, &cx)
            .expect_err("exit 1 must be a typed failure");
        assert!(matches!(err, Error::ActionFailed(1)), "got {err:?}");
    }

    #[test]
    fn probe_polarity() {
        let cx = user_cx();
        let present = script_action("exit 0", None, Some("exit 0"), false, Shell::PowerShell);
        assert!(ActionKind.run_probe(&present, &cx).unwrap());

        let absent = script_action("exit 0", None, Some("exit 1"), false, Shell::PowerShell);
        assert!(!ActionKind.run_probe(&absent, &cx).unwrap());
    }

    /// The brief's suggested probe shape: presence of a temp-file marker.
    #[test]
    fn probe_polarity_against_a_temp_file_marker() {
        let path = std::env::temp_dir().join(format!(
            "magicx-action-test-marker-{}-{}",
            std::process::id(),
            SCRATCH_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let _cleanup = Cleanup(path.clone());

        let probe_script = format!(
            "if (Test-Path '{}') {{ exit 0 }} else {{ exit 1 }}",
            path.display()
        );
        let action = script_action(
            "exit 0",
            None,
            Some(&probe_script),
            false,
            Shell::PowerShell,
        );
        let cx = user_cx();

        assert!(
            !ActionKind.run_probe(&action, &cx).unwrap(),
            "marker must read absent before it exists"
        );
        std::fs::write(&path, b"present").unwrap();
        assert!(
            ActionKind.run_probe(&action, &cx).unwrap(),
            "marker must read present once created"
        );
    }

    #[test]
    fn probe_without_probe_script_is_typed_error_not_ok_false() {
        let cx = user_cx();
        let action = script_action("exit 0", None, None, false, Shell::PowerShell);
        let err = ActionKind
            .run_probe(&action, &cx)
            .expect_err("no probe must be Err, never Ok(false)");
        assert!(matches!(err, Error::Invalid(_)), "got {err:?}");
    }

    #[test]
    fn undo_without_undo_script_is_typed_error() {
        let cx = user_cx();
        let action = script_action("exit 0", None, None, false, Shell::PowerShell);
        let err = ActionKind
            .run_undo(&action, &cx)
            .expect_err("no undo must be Err -- the action is one-way");
        assert!(matches!(err, Error::Invalid(_)), "got {err:?}");
    }

    #[test]
    fn undo_runs_and_shares_the_exit_code_contract() {
        let cx = user_cx();
        let action = script_action("exit 0", Some("exit 0"), None, false, Shell::PowerShell);
        ActionKind
            .run_undo(&action, &cx)
            .expect("undo exit 0 must succeed");

        let failing = script_action("exit 0", Some("exit 7"), None, false, Shell::PowerShell);
        let err = ActionKind
            .run_undo(&failing, &cx)
            .expect_err("undo exit 7 must be a typed failure");
        assert!(matches!(err, Error::ActionFailed(7)), "got {err:?}");
    }

    #[test]
    fn spawn_failure_is_err_never_ok() {
        let err = spawn_command("definitely-not-a-real-executable-98213.exe", &[])
            .expect_err("a nonexistent program must fail to spawn");
        assert!(matches!(err, Error::ActionExecFailed(_)), "got {err:?}");
    }

    #[test]
    fn timeout_kills_and_errs() {
        let start = Instant::now();
        let err = run_script(
            Shell::PowerShell,
            "Start-Sleep -Seconds 5",
            Duration::from_millis(300),
        )
        .expect_err("a script that outlives its timeout must be a typed error");
        assert!(matches!(err, Error::ActionExecFailed(_)), "got {err:?}");
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "the timeout must actually kill the process rather than waiting out the full sleep: took {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn encoded_command_carries_special_chars() {
        // Quotes, a literal `$`, and a newline between two statements -- none of it composed into
        // a shell command line (spec §7), so nothing here needs escaping.
        let body = r#"$s = 'a $b "c" d'
if ($s -eq 'a $b "c" d') { exit 0 } else { exit 1 }"#;
        let code = run_script(Shell::PowerShell, body, ACTION_TIMEOUT)
            .expect("script must run to completion");
        assert_eq!(
            code, 0,
            "special characters must round-trip through the encoded command intact"
        );
    }

    #[test]
    fn cmd_shell_runs_via_a_temp_script_file() {
        assert_eq!(run_script(Shell::Cmd, "exit 0", ACTION_TIMEOUT).unwrap(), 0);
        assert_eq!(run_script(Shell::Cmd, "exit 3", ACTION_TIMEOUT).unwrap(), 3);
    }

    // The temp-script guards themselves (exclusive create, the held share-mode lock, delete on
    // drop) are pinned in `services::exclusive_temp`, which now owns them.

    #[test]
    fn drive_rejects_system_and_ti_for_script_actions() {
        let action = script_action("exit 0", Some("exit 0"), None, false, Shell::PowerShell);
        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = ActionKind
                .run_apply(&action, &cx)
                .expect_err("this build cannot yet route System/Ti through the broker");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
            let err = ActionKind
                .run_undo(&action, &cx)
                .expect_err("this build cannot yet route System/Ti through the broker");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    #[test]
    fn probe_never_gates_on_level() {
        // Mirrors registry.rs's `read_runs_in_process_regardless_of_declared_level`: probe is a
        // read, so (unlike apply/undo) it must not reject System/Ti.
        let action = script_action("exit 0", None, Some("exit 0"), false, Shell::PowerShell);
        for level in [Level::User, Level::Admin, Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            assert!(
                ActionKind.run_probe(&action, &cx).unwrap(),
                "probe must not depend on level {level:?}"
            );
        }
    }

    // --- DeleteTree ------------------------------------------------------------------------

    /// A unique HKCU scratch subtree that deletes itself on drop, even on panic (mirrors
    /// registry.rs's own `Scratch`).
    struct Scratch {
        path: String,
    }
    impl Scratch {
        fn new(label: &str) -> Self {
            let n = SCRATCH_COUNTER.fetch_add(1, Ordering::SeqCst);
            Scratch {
                path: format!(
                    "Software\\MagicXToolboxTest\\kindaction_{label}_{}_{n}",
                    std::process::id()
                ),
            }
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = registry_service::delete_key(&RegistryHive::Hkcu, &self.path);
        }
    }

    #[test]
    fn delete_tree_apply_deletes_the_key_recursively() {
        let scratch = Scratch::new("apply");
        registry_service::create_key(&RegistryHive::Hkcu, &format!("{}\\Child", scratch.path))
            .unwrap();
        let key = KeyAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
        };
        let action = ActionDef::DeleteTree {
            key: key.clone(),
            undo: None,
        };

        ActionKind
            .run_apply(&action, &user_cx())
            .expect("delete-tree apply must succeed");
        assert_eq!(
            RegistryKind
                .read(&Setting::RegistryKey(key), &user_cx())
                .unwrap(),
            Value::Present(false)
        );
    }

    #[test]
    fn delete_tree_apply_on_already_absent_key_is_idempotent() {
        let scratch = Scratch::new("idempotent");
        let key = KeyAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
        };
        let action = ActionDef::DeleteTree { key, undo: None };
        ActionKind
            .run_apply(&action, &user_cx())
            .expect("deleting an already-absent tree must be a no-op success");
    }

    #[test]
    fn delete_tree_undo_absent_is_one_way() {
        let scratch = Scratch::new("no_undo");
        let key = KeyAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
        };
        let action = ActionDef::DeleteTree { key, undo: None };
        let err = ActionKind
            .run_undo(&action, &user_cx())
            .expect_err("no undo must be Err -- delete-tree is one-way without one");
        assert!(matches!(err, Error::Invalid(_)), "got {err:?}");
    }

    #[test]
    fn delete_tree_undo_runs_the_restore_script() {
        let scratch = Scratch::new("undo");
        let key = KeyAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
        };
        let restore_ps = format!("New-Item -Path 'HKCU:\\{}' -Force | Out-Null", key.path);
        let action = ActionDef::DeleteTree {
            key: key.clone(),
            undo: Some(Script(restore_ps)),
        };

        ActionKind
            .run_undo(&action, &user_cx())
            .expect("undo script must run and exit 0");
        assert_eq!(
            RegistryKind
                .read(&Setting::RegistryKey(key), &user_cx())
                .unwrap(),
            Value::Present(true)
        );
    }

    /// Fix 4: `drive_rejects_system_and_ti_for_script_actions` above only covers the `Script`
    /// variant -- this asserts the same guard for `DeleteTree`'s `run_apply` (via the reused
    /// `RegistryKind::drive`) and `run_undo` directly, rather than only through code inspection.
    #[test]
    fn delete_tree_rejects_system_and_ti_levels() {
        let scratch = Scratch::new("level_gate");
        let key = KeyAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
        };
        let apply_action = ActionDef::DeleteTree {
            key: key.clone(),
            undo: None,
        };
        let undo_action = ActionDef::DeleteTree {
            key,
            undo: Some(Script("exit 0".to_string())),
        };

        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = ActionKind.run_apply(&apply_action, &cx).expect_err(
                "delete-tree apply must reject System/Ti exactly like a raw RegistryKey effect",
            );
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");

            let err = ActionKind
                .run_undo(&undo_action, &cx)
                .expect_err("delete-tree undo must reject System/Ti");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    #[test]
    fn delete_tree_probe_is_not_reachable_honestly() {
        let scratch = Scratch::new("probe");
        let key = KeyAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
        };
        let action = ActionDef::DeleteTree { key, undo: None };
        let err = ActionKind
            .run_probe(&action, &user_cx())
            .expect_err("delete-tree has no probe by type -- must never fake true/false");
        assert!(matches!(err, Error::Invalid(_)), "got {err:?}");
    }
}
