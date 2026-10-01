# Test matrix and signing plan

Status: proposal, awaiting review. Nothing here is implemented beyond the hosted CI matrix already in `.github/workflows/ci.yml`.

This document covers the three things the elevation work could not close in code, because each needs hardware, money, or a decision that is yours to make.

## 1. What CI proves today, and what it cannot

`ci.yml` now runs `validate` and the previously-skipped ignored tests across `windows-2022` and `windows-2025`. That is every Windows image GitHub hosts, and both are **Server** SKUs (builds 20348 and 26100).

The gap this leaves is specific, not vague. `block_update_pipeline` is the only tweak that reaches the elevation broker, and it targets Windows Update scheduled tasks whose set differs between client builds. `windows_update.yaml` says so directly on the comment above `wu_sih`: "Windows 10 era, absent on 24H2, present on 19045". A Server runner has a third task set again. So the one code path that most needs coverage is the one CI structurally cannot exercise.

Nothing in CI has ever spawned a real elevated child either. A GitHub runner runs as an administrator, so this is not impossible in principle, but no test attempts it.

### Proposed client matrix

| Runner | Build | Why it earns its place |
|---|---|---|
| Windows 10 22H2 client | 19045 | The build where `sih` and `sihboot` exist. Proves the `optional` path and the older task set. |
| Windows 11 24H2 client | 26100 | The build where they do not. Proves the absent-optional no-op that used to abort the whole tweak. |
| Windows 11 client, hardened | any | TrustedInstaller disabled by a CIS or STIG baseline. Proves the new pre-click availability probe on a machine that actually has the condition. |

Two of these can be VMs. The third is a VM plus a hardening baseline applied once.

### What you would need to provide

1. **A machine or hypervisor that can host two or three always-on VMs.** They idle almost all the time; they only need to be reachable when a workflow runs.
2. **A GitHub self-hosted runner registered on each**, with labels such as `win-client-19045`, `win-client-26100`, `win-hardened`. Registration is a copy-paste command from the repo's Actions settings.
3. **A decision about trust.** Self-hosted runners must not be used on public-repo pull requests from forks, because a fork's PR can run arbitrary code on your machine. If this repo is or becomes public, gate the self-hosted jobs on `github.event_name != 'pull_request'` or on a label only you can apply.

Once those exist the workflow change is small: add a job with `runs-on: [self-hosted, win-client-26100]` running the same two steps, plus a genuinely elevated test that applies and reverts `block_update_pipeline` and asserts the snapshot is consumed.

I can write that job as soon as you tell me the labels. Until then it would be a job that never runs, which is worse than no job.

## 2. Code signing

`src-tauri/tauri.conf.json` sets `"certificateThumbprint": null` with an empty `timestampUrl`. The binary is unsigned.

That matters more here than for a typical app, because of what this one does. The elevation path enables `SeDebugPrivilege`, opens a SYSTEM service process with `PROCESS_CREATE_PROCESS`, and sets `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` to spoof TrustedInstaller as the parent of a child it then runs privileged code in. From a signed, known publisher that reads as a system utility. From an unsigned binary it is close to a textbook parent-PID-spoofing signature, and it is exactly what behavioural detection is tuned to score.

Grouped execution helps here for a reason that has nothing to do with speed: applying that tweak used to be 21 process creations with the same signature in a few seconds, and repetition is what trips scoring heuristics. It is now one.

### What signing needs from you

1. **An OV or EV code-signing certificate.** OV is cheaper and enough for SmartScreen reputation to build over time. EV gets instant SmartScreen reputation and is typically required for kernel-adjacent work, which this is not. OV is the reasonable choice.
2. **Certificate storage.** As of 2023 all new code-signing certificates must live on an HSM or hardware token, so a CI runner cannot simply hold a `.pfx`. In practice this means either signing on a machine with the token attached, or using a cloud signing service the CA offers.
3. **The config change**, once the certificate exists: `certificateThumbprint` plus a `timestampUrl`. Timestamping is not optional; without it every signature expires with the certificate.

Order matters. Signing first makes the next two items cheaper to interpret, because an unsigned binary being blocked tells you nothing about whether the code is at fault.

## 3. Three assumptions nobody has measured

Each is a short experiment, and each could delete code or change a decision.

**Is the broker child already inside TrustedInstaller's job object?** Under `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` a child can inherit the attribute parent's job. If it does, adding a job object of our own is redundant, and if that job carries a kill-on-close limit then TrustedInstaller stopping could take our child with it. Run `IsProcessInJob` against a real spawned child. Thirty minutes. Until it is answered, no job-object work should start, which is why none did.

**Is `SeDebugPrivilege` actually required?** `get_trusted_installer_handle` enables it before touching the SCM. An Administrator token may be able to open TrustedInstaller.exe for `PROCESS_CREATE_PROCESS` without it on a stock machine. If it can, dropping the call removes a group-policy dependency that hardened and domain-joined machines routinely strip, and removes a large part of the detection signature. One hour, and the result either deletes code or documents why the code is needed.

**What does real security software do to the spawn?** The failure shapes to look for are `OpenProcess` returning `ERROR_ACCESS_DENIED` after a successful privilege enable, and `CreateProcessW` failing with 5 or 1314. Those are now decoded and named, so the remaining step is to observe which actually occur on a machine with a mainstream product installed, and confirm the message sends a user to the right place. Half a day, and it wants the hardened VM from section 1 anyway.

## 4. What is deliberately not on this list

**A named-pipe broker transport.** The response file is the weaker half of the transport, and the honest residual is written into `broker.rs`: a same-user process that learns the path can overwrite the response between the child's exit and the parent's read. An inherited anonymous pipe cannot fix it, because `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` sources handle inheritance from the attribute parent rather than from us. A named pipe with a random name and a restrictive DACL would, and it is a real change to a working privileged path.

It is not on this list because the residual is bounded and the bound is verified rather than assumed. Every driven effect is read back in-process by a path the forger cannot touch, so a forged response degrades into a verify mismatch and a rollback. The cost is a false failure, never a false success. Meanwhile the arbitrary-write shape that pre-planting could have produced is closed: the child now creates the response with `CREATE_NEW` and `FILE_FLAG_OPEN_REPARSE_POINT`.

Revisit it if the threat model changes, or if a future `BrokerOp` ever makes a forged success worse than a spurious rollback.

**Recovering from a power loss mid-apply.** Closing the window, relaunching as administrator, and installing an update are all refused while a tweak is applying. A power cut is not refusable, and it leaves a snapshot entry describing a partly-applied tweak that startup recovery does not surface, because recovery scans per-Action journal rows and a Settings-only tweak writes none. Closing that needs a drive-started marker in the snapshot schema, which is a schema change and separate work.
