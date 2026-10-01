//! `ActionKind`: apply/undo/probe for `Action` effects (spec §5.5/§7); not an `EffectKind`, as Actions are not `Setting`s.
//! Apply/undo [`guard_level`] and reject `Ti` (no `BrokerOp` carries a script); probe is a read and never gates on level.
//! A missing `undo` or `probe` is `Error::Invalid`, never `Ok`. The exit code is the only success signal; output is logged, never parsed.
//! `kill()` reaches one pid: every child runs in a [`KillOnCloseJob`], so the timeout bounds the whole tree and the pipe drains end.
//! A Cmd script reaches `%TEMP%` through [`ExclusiveTempFile`] (user-writable), held until the child exits.

use std::io::Read;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use regex_lite::Regex;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

use crate::services::exclusive_temp::ExclusiveTempFile;
use crate::services::system32::SystemTool;
use crate::tweaks::model::{ActionDef, Probe, Shell};

use super::registry::RegistryKind;
use super::{guard_level, Error, ExecCx};

/// Bound for every probe, and for an apply/undo whose action sets no `timeout` (spec §14).
pub(crate) const ACTION_TIMEOUT: Duration = Duration::from_secs(30);

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
            ActionDef::Script {
                apply,
                shell,
                timeout,
                ..
            } => {
                guard_level(cx)?;
                run_and_require_zero(*shell, &apply.0, script_timeout(*timeout), cx)
            }
            ActionDef::DeleteTree { key, .. } => RegistryKind.delete_tree(key, cx),
        }
    }

    /// Runs `action`'s `undo`. Absent `undo` is a typed error, never a silent no-op (spec §7: a
    /// no-`undo` action is honestly one-way).
    pub fn run_undo(&self, action: &ActionDef, cx: &ExecCx) -> Result<(), Error> {
        match action {
            ActionDef::Script {
                undo: Some(undo),
                shell,
                timeout,
                ..
            } => {
                guard_level(cx)?;
                run_and_require_zero(*shell, &undo.0, script_timeout(*timeout), cx)
            }
            ActionDef::Script { undo: None, .. } => Err(Error::Invalid(
                "this action has no undo script -- it is one-way (spec §7)",
            )),
            ActionDef::DeleteTree {
                undo: Some(undo), ..
            } => {
                guard_level(cx)?;
                run_and_require_zero(Shell::PowerShell, &undo.0, ACTION_TIMEOUT, cx)
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
            } => Ok(run_script(*shell, &body.0, ACTION_TIMEOUT)?.code == 0),
            // `engine::detect` answers the native probe forms itself; reaching this arm is an engine
            // routing bug.
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

fn script_timeout(seconds: Option<u32>) -> Duration {
    seconds.map_or(ACTION_TIMEOUT, |s| Duration::from_secs(s.into()))
}

fn run_and_require_zero(
    shell: Shell,
    body: &str,
    timeout: Duration,
    cx: &ExecCx,
) -> Result<(), Error> {
    let run = run_script(shell, body, timeout)?;
    let id = cx.effect().map_or("?", |e| e.0.as_str());
    run.warn_on_failure(&format!("action '{id}'"));
    match run.code {
        0 => Ok(()),
        code => Err(Error::ActionFailed(code)),
    }
}

/// End of a script's output kept for the log when it fails.
const TAIL_BYTES: usize = 2048;

/// A script that ran to its end. `code` is the sole success signal; `tail` is for the log only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptRun {
    pub code: i32,
    /// The end of stderr, or of stdout when stderr is empty.
    pub tail: String,
}

impl ScriptRun {
    pub(crate) fn warn_on_failure(&self, who: &str) {
        if self.code != 0 {
            let output = if self.tail.is_empty() {
                "(no output)"
            } else {
                &self.tail
            };
            log::warn!("{who} exited with {}; output:\n{output}", self.code);
        }
    }
}

/// powershell.exe under `-EncodedCommand` writes stderr as CLIXML whatever `-OutputFormat` says:
/// keep the plain text and the error strings, drop the progress records.
fn readable(stream: &str) -> String {
    if !stream.contains("#< CLIXML") {
        return stream.to_string();
    }
    let block = Regex::new(r"(?s)<Objs[^>]*>.*?</Objs>").expect("constant pattern");
    let string = Regex::new(r#"(?s)<S S="[^"]*">(.*?)</S>"#).expect("constant pattern");
    let escape = Regex::new(r"_x([0-9A-Fa-f]{4})_").expect("constant pattern");
    let mut out = String::new();
    let mut last = 0;
    for objs in block.find_iter(stream) {
        out.push_str(stream.get(last..objs.start()).unwrap_or_default());
        for s in string.captures_iter(objs.as_str()) {
            out.push_str(&s[1]);
        }
        last = objs.end();
    }
    out.push_str(stream.get(last..).unwrap_or_default());
    let out = escape.replace_all(&out, |c: &regex_lite::Captures<'_>| {
        u32::from_str_radix(&c[1], 16)
            .ok()
            .and_then(char::from_u32)
            .map(String::from)
            .unwrap_or_default()
    });
    out.replace("#< CLIXML", "")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .trim()
        .to_string()
}

/// Starts past a separator: a fragment of a SID or path would slip past redaction.
fn tail_of(text: &str) -> &str {
    let at = crate::logging::redact::cut_start(text, text.len().saturating_sub(TAIL_BYTES));
    text.get(at..).unwrap_or_default().trim_start()
}

/// Runs one script body to completion or until `timeout` kills it. `code` is the sole,
/// locale-independent success signal (spec §7); output is never interpreted.
pub(crate) fn run_script(shell: Shell, body: &str, timeout: Duration) -> Result<ScriptRun, Error> {
    match shell {
        Shell::PowerShell => wait_with_timeout(spawn_powershell(body)?, timeout),
        Shell::Cmd => {
            let file =
                ExclusiveTempFile::create("magicx-action", "cmd", "action script", body.as_bytes())
                    .map_err(|e| Error::ActionNotStarted(format!("temp script: {e}")))?;
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
        SystemTool::PowerShell,
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
    spawn_command(SystemTool::Cmd, &["/c", &path])
}

fn spawn_command(tool: SystemTool, args: &[&str]) -> Result<Child, Error> {
    let cmd = tool
        .command()
        .map_err(|e| Error::ActionNotStarted(e.to_string()))?;
    spawn(cmd, args)
}

fn spawn(mut cmd: Command, args: &[&str]) -> Result<Child, Error> {
    cmd.args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.spawn().map_err(|e| {
        let program = cmd.get_program().to_string_lossy();
        Error::ActionNotStarted(format!("failed to spawn {program}: {e}"))
    })
}

/// Polls `child` until it exits or `timeout` passes, then kills and reaps it (std has no process
/// timeout, spec §14). The pipes drain on threads, so a chatty script cannot block on a full pipe.
fn wait_with_timeout(mut child: Child, timeout: Duration) -> Result<ScriptRun, Error> {
    // Bind `child` into its kill-on-close job before anything else, so a fast-spawning grandchild
    // cannot start outside it. Either step failing kills the child: fail closed, never unmonitored.
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
                // Symmetric with the timeout arm: a failed wait must not leave the child running.
                let _ = child.kill();
                let _ = child.wait();
                break Err(Error::ActionExecFailed(format!(
                    "failed to wait on action process: {e}"
                )));
            }
        }
    };

    // Close the job before joining the drain threads: it kills any descendant still holding the
    // inherited stdout/stderr pipe, so the pipe reaches EOF and the joins cannot hang past the bound.
    drop(job);

    let joined = |t: Option<thread::JoinHandle<String>>| {
        t.map(|t| t.join().unwrap_or_default()).unwrap_or_default()
    };
    let (stdout, stderr) = (joined(out), joined(err));
    let shown = if stderr.is_empty() { &stdout } else { &stderr };
    status.map(|s| ScriptRun {
        code: s.code().unwrap_or(-1),
        tail: tail_of(shown).to_string(),
    })
}

/// A Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`: the child and every descendant die when
/// the last handle closes, on any exit path including unwind. Bounds the whole tree, not one pid.
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

/// Drains a pipe on a background thread, logs it at Debug and returns it decoded: raw CLIXML glues
/// `_x000D_` to names and defeats redaction. Never parsed for success (spec §7/§14).
fn drain_to_log(
    mut pipe: impl Read + Send + 'static,
    stream: &'static str,
) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        if pipe.read_to_end(&mut buf).is_err() {
            return String::new();
        }
        let text = readable(String::from_utf8_lossy(&buf).trim());
        if !text.is_empty() {
            log::debug!("action {stream}: {text}");
        }
        text
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
    use crate::tweaks::kinds::EffectKind;
    use crate::tweaks::model::{Hive, KeyAddr, Level, Probe, Script, Setting, Value};
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
            timeout: None,
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

    /// Probe shape: presence of a temp-file marker.
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
        let missing = std::env::temp_dir().join("definitely-not-a-real-executable-98213.exe");
        let err = spawn(Command::new(missing), &[]).expect_err("a nonexistent program must fail");
        assert!(matches!(err, Error::ActionNotStarted(_)), "got {err:?}");
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
    fn apply_and_undo_honor_the_actions_own_timeout() {
        let cx = user_cx();
        let mut action = script_action(
            "Start-Sleep -Seconds 10",
            Some("Start-Sleep -Seconds 10"),
            None,
            false,
            Shell::PowerShell,
        );
        if let ActionDef::Script { timeout, .. } = &mut action {
            *timeout = Some(1);
        }
        for (label, result) in [
            ("apply", ActionKind.run_apply(&action, &cx)),
            ("undo", ActionKind.run_undo(&action, &cx)),
        ] {
            let err = result.expect_err("a script outliving its own timeout must fail");
            assert!(
                matches!(&err, Error::ActionExecFailed(m) if m.contains("1s")),
                "{label}: got {err:?}"
            );
        }
        assert_eq!(script_timeout(None), ACTION_TIMEOUT);
    }

    #[test]
    fn encoded_command_carries_special_chars() {
        // Quotes, a literal `$`, and a newline between two statements -- none of it composed into
        // a shell command line (spec §7), so nothing here needs escaping.
        let body = r#"$s = 'a $b "c" d'
if ($s -eq 'a $b "c" d') { exit 0 } else { exit 1 }"#;
        let code = run_script(Shell::PowerShell, body, ACTION_TIMEOUT)
            .expect("script must run to completion")
            .code;
        assert_eq!(
            code, 0,
            "special characters must round-trip through the encoded command intact"
        );
    }

    #[test]
    fn cmd_shell_runs_via_a_temp_script_file() {
        assert_eq!(
            run_script(Shell::Cmd, "exit 0", ACTION_TIMEOUT)
                .unwrap()
                .code,
            0
        );
        assert_eq!(
            run_script(Shell::Cmd, "exit 3", ACTION_TIMEOUT)
                .unwrap()
                .code,
            3
        );
    }

    #[test]
    fn a_failing_script_keeps_the_end_of_its_error_output() {
        let run = run_script(
            Shell::PowerShell,
            "Write-Output 'ignored'; [Console]::Error.WriteLine('boom'); exit 3",
            ACTION_TIMEOUT,
        )
        .unwrap();
        assert_eq!((run.code, run.tail.as_str()), (3, "boom"));
        let run = run_script(
            Shell::PowerShell,
            "Write-Error 'a <b> & c'; exit 5",
            ACTION_TIMEOUT,
        )
        .unwrap();
        assert_eq!(run.code, 5);
        assert!(run.tail.contains("a <b> & c"), "{}", run.tail);
        assert!(!run.tail.contains("<Objs"), "{}", run.tail);
        let run = run_script(Shell::Cmd, "@echo only stdout\r\n@exit 4", ACTION_TIMEOUT).unwrap();
        assert_eq!((run.code, run.tail.as_str()), (4, "only stdout"));
    }

    #[test]
    fn clixml_error_output_is_made_readable() {
        let raw = "#< CLIXML\r\nraw line\r\n<Objs Version=\"1.1.0.1\"><Obj S=\"progress\" RefId=\"0\"><AV>Preparing modules</AV></Obj><S S=\"Error\">bad &lt;x&gt;_x000D__x000A_</S><S S=\"Error\">  + Id : E_x000D__x000A_</S></Objs>";
        assert_eq!(readable(raw), "raw line\r\nbad <x>\r\n  + Id : E");
        assert_eq!(readable("plain"), "plain");
    }

    #[test]
    fn the_output_tail_keeps_no_fragment_of_an_identifier() {
        let text = format!("S-1-5-21-111-222-333-1001\r\n{}", "w ".repeat(1012));
        assert_eq!(tail_of(&text), "w ".repeat(1012));
    }

    #[test]
    fn logged_error_output_is_decoded() {
        let raw = "#< CLIXML\r\n<Objs Version=\"1.1.0.1\"><S S=\"Error\">no Alice_x000D__x000A_</S></Objs>";
        let text = drain_to_log(std::io::Cursor::new(raw.as_bytes().to_vec()), "stderr")
            .join()
            .unwrap();
        assert_eq!(text, "no Alice");
    }

    #[test]
    fn the_output_tail_is_cut_on_a_char_boundary() {
        let text = format!("head{}", "é".repeat(TAIL_BYTES));
        let tail = tail_of(&text);
        assert!(tail.len() <= TAIL_BYTES);
        assert!(tail.chars().all(|c| c == 'é'));
        assert_eq!(tail_of("short"), "short");
    }

    // The temp-script guards themselves (exclusive create, the held share-mode lock, delete on
    // drop) are pinned in `services::exclusive_temp`, which now owns them.

    #[test]
    fn drive_rejects_system_and_ti_for_script_actions() {
        let action = script_action("exit 0", Some("exit 0"), None, false, Shell::PowerShell);
        let level = Level::Ti;
        let cx = ExecCx::new(level);
        let err = ActionKind
            .run_apply(&action, &cx)
            .expect_err("this build cannot yet route Ti through the broker");
        assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        let err = ActionKind
            .run_undo(&action, &cx)
            .expect_err("this build cannot yet route Ti through the broker");
        assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
    }

    #[test]
    fn probe_never_gates_on_level() {
        // Mirrors registry.rs's `read_runs_in_process_regardless_of_declared_level`: probe is a
        // read, so (unlike apply/undo) it must not reject Ti.
        let action = script_action("exit 0", None, Some("exit 0"), false, Shell::PowerShell);
        for level in [Level::User, Level::Admin, Level::Ti] {
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
        let child = format!("{}\\Child", scratch.path);
        registry_service::set_dword(&RegistryHive::Hkcu, &child, "Flag", 1).unwrap();
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

    /// The `DeleteTree` counterpart of `drive_rejects_system_and_ti_for_script_actions`: apply (via
    /// `RegistryKind::delete_tree`) and undo both reject Ti.
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

        let level = Level::Ti;
        let cx = ExecCx::new(level);
        let err = ActionKind
            .run_apply(&apply_action, &cx)
            .expect_err("delete-tree apply must reject Ti exactly like a raw RegistryKey effect");
        assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");

        let err = ActionKind
            .run_undo(&undo_action, &cx)
            .expect_err("delete-tree undo must reject Ti");
        assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
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
