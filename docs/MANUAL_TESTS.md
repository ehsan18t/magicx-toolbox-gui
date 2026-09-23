# Manual Tests

The test build is the normal app plus a Manual Tests view: one card per test, each with a Run button, a live log, a Pass, Fail or Info result, and a Copy report button. It exists to check behaviour that only a real Windows machine can show, such as TrustedInstaller timing or whether Windows puts a change back on its own.

## Build and run

- `pnpm run build:test` builds the app with the `test-build` Cargo feature (`tauri build --features test-build`). The output lands in the usual `src-tauri/target/release` folder, so do not ship a binary from a folder you last built with `build:test`.
- `pnpm run dev:test` runs the development server with the same feature.
- Copy the built executable to the target PC, run it as administrator, and open Manual Tests in the sidebar.

A normal build (`pnpm run build`) compiles none of the manual test code. The frontend is the same in both builds: it calls `manual_tests_available`, which is the only command present in every build, and shows the view only when it returns `true`.

The Manual Tests view itself stays in the normal frontend bundle; it is hidden at runtime because the backend reports no manual tests.

Run one test at a time (the app refuses a second run while one is going). When a test finishes, press Copy report and paste the text wherever you are collecting results. The report starts with the app version, the Windows build and revision, the elevation state and the machine and process architecture, then the test id, start and finish times, the result, and every log line. The same lines are written to the app's log file with the prefix `[manual-test <id>]`. Account names and profile or temp folder paths are replaced with `<redacted>`.

## What each test changes

| Test | Changes this PC | What it does |
| :--- | :--- | :--- |
| `baseline_block_update_pipeline` | No | Reads every effect of `block_update_pipeline` through the engine's own reads, lists each live value, and keeps the reading in memory as the baseline. |
| `ti_batch_timing` | Yes, for a few seconds | Checks the preconditions, reads a baseline, applies `block_update_pipeline` through the same path as the Apply button, logs every TrustedInstaller batch (op count, milliseconds, share of the 30 s budget) and the whole apply's wall time (batches are recorded process-wide while the test runs, labelled by elevation level, so do not change other tweaks in the app meanwhile), then restores through the same path as the System Default restore and compares every effect with the baseline. Passes only if the apply verified, the restore verified and every effect matches. |
| `waasmedic_watch` | Yes, for the whole watch | Same preconditions, baseline and apply, then reads every effect once a minute for the chosen duration (15 minutes by default) and logs each change away from the applied value with a timestamp. Cancel ends the watch early. Either way it then restores and compares with the baseline. The result lists any drift (Info), or fails if the restore does not match the baseline. |
| `waasmedic_task_read` | No | Reads `\Microsoft\Windows\WaaSMedic\PerformRemediation` and `DeferredWork` through the scheduler service as the app's own process. Passes if both read without access denied. |

The two tests that change the PC refuse to start, before changing anything, if the app is not elevated, if `block_update_pipeline` already has snapshot entries, if it shows Needs Attention, or if it is being changed right now. They ask for confirmation first.

If a restore fails or leaves an effect different from the baseline, the result says so in capitals and lists every differing effect. The engine keeps the snapshot in that case: open the "Block the Windows Update pipeline" card, which shows Needs Attention, and restore from there. Never delete that snapshot by hand. Closing the app during a watch leaves the tweak applied with its snapshot in place, so the card can restore it the same way.

## Add a test

Everything lives in `src-tauri/src/manual_tests/`, which is compiled only with the `test-build` feature.

1. Write the test as a function `fn(&Ctx) -> Verdict`, usually in `cases.rs`. Log each step with `cx.info(...)` and each error with `cx.error(...)`; both reach the log file, the live view and the report. Return `Verdict::pass`, `Verdict::fail` or `Verdict::info`, optionally `.with_details(...)`. A test that loops should check `cx.sleep(...)` or `cx.cancelled()` so Cancel works.
2. Add one entry to `TESTS` in `src-tauri/src/manual_tests/mod.rs`: `id`, `title`, `description`, `changes` (plain words for the user), `changes_system` (puts Run behind a confirmation), `minutes` (`Some(default)` shows a duration field) and `run`.

The frontend lists whatever `TESTS` holds, so no frontend change is needed. Use the app's production paths only (the engine, the command layer's `apply_gated`/`restore_gated` through `cx.host`, the service modules). Do not add operations or diagnostics to the elevated broker child. For errors, log the typed chain with the helpers in `errors.rs` rather than raw broker text, which can carry temp paths and the transport nonce.

## Not included

The diagnostics that would have to run inside the elevated helper process (review items A3.4, F60 and F62/C5) are not part of the test build. The manual tests observe the helper only from the parent side: the batch recorder sits at the parent's existing "broker batch finished" log site.
