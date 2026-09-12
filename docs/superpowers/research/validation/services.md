# Services and Scheduled Tasks tweak validation

Validated 2026-07-27. Source corpus: `src-tauri/tweaks/services.yaml` (29 tweaks: 23 service tweaks
and 6 scheduled-task tweaks).

**Target platform.** Primary: Windows 11 24H2 (build 26100) and newer, including 25H2 (26200).
Secondary and low priority: Windows 10 IoT Enterprise LTSC 2021 (19044). Windows 10 22H2 consumer and
Windows 11 21H2 / 22H2 / 23H2 are out of scope, which is what moves three tweaks in this file from
"needs a gate" to "has no supported platform on the primary target".

This document consolidates the original validation pass and two adversarial rounds run over it. It
supersedes every earlier revision of this file. It is research only: no YAML was edited.

**Not in scope here.** `disable_ai_fabric_service` (`WSAIFabricSvc`) is a newly verified service
control that belongs to the new `ai` category, not to this file. It is cross-referenced where it
matters (see `task_autochk_proxy` and merge candidate 4) but it is not one of the 29 tweaks below.

## Method note

Microsoft does not publish a per-release default start-type table for *client* Windows. The highest
tier available for that specific fact is:

- Tier A: the Windows Server "guidelines for disabling system services" table, which covers most of
  these services on a Desktop Experience install and gives Microsoft's own default start type and
  its own stance on disabling.
- Tier A: individual service manifests reproduced verbatim in Microsoft Learn (for example
  `SCardSvr`, where the manifest attribute `start="demand"` appears in the docs).
- Tier A: shipped binary resource strings that Windows itself uses as a task or service description
  (for example `acproxy.dll,-102`). These describe the product, not one machine.
- Tier B: the CIS Microsoft Windows 11 Enterprise Benchmark, which records a per-release "Default
  Value" for each service it covers.
- Tier C: two independent clean-install dumps (Winhelponline Windows 10 Pro 22H2 and Windows 11 Pro
  23H2) plus two per-release reference sites (batcmd.com, revertservice.com).

**Evidence provenance, read this before acting.** The machine used during validation is heavily
modified by its owner and is not a stock system. Nothing about a Windows *default* may rest on it.
That specifically disqualifies its current service `Start` values and its current scheduled-task
`<Enabled>` elements, because enabling or disabling a task rewrites that element inside the task's
own XML under `C:\Windows\System32\Tasks`. Two categories of observation from that machine do
survive, and are used below:

1. **Trigger registrations** under `HKLM\SYSTEM\CurrentControlSet\Services\<name>\TriggerInfo`.
   `ChangeServiceConfigW` does not touch trigger registration, so a start-type change by this app or
   by the owner cannot have created or removed them.
2. **An empty `<Triggers />` element** in a task definition. No enable or disable operation produces
   an empty trigger set; enabling and disabling rewrite `<Enabled>`, not `<Triggers>`.

Everything else that rested on that machine's state has been withdrawn and moved to
`## Open questions needing a clean image`.

**The engine never starts or stops a service.** `src-tauri/src/tweaks/kinds/service.rs` drives a
service effect with `service_control::set_service_startup` (a `ChangeServiceConfigW` call) plus an
explicit `DelayedAutostart` companion write, and nothing else. There is no `StartServiceW` and no
`ControlService(STOP)` on any tweak path. Two consequences run through this whole document:

- Every service tweak takes effect **at the next reboot**, so every one of them needs
  `requires_reboot: true`. Twelve currently omit it.
- Driving `automatic` explicitly writes `DelayedAutostart = 0`, so restoring a service whose stock
  state is Automatic (Delayed Start) with a plain `automatic` option is an incomplete revert that
  moves the service into the boot start path.

Also relevant: `drive_service` returns `Error::ResourceMissing` when the service does not exist, and
`read_service` returns `Value::Missing`, which for a non-`optional` effect makes the engine classify
the whole tweak `TweakState::Unknown` (`UnknownCause::MissingRequired`) and refuse to apply it with
`EngineError::SurfaceUnreadable`. A tweak targeting an absent service is therefore not a quiet no-op;
it is an unusable tile.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `disable_diagtrack` | VERIFIED | medium | Microsoft-documented | Restate the `AllowTelemetry` claim in Microsoft's own terms |
| `disable_print_spooler` | VERIFIED | medium | Microsoft-documented | none |
| `disable_fax` | INCORRECT | low | Community-corroborated | `Fax` is absent on the entire primary platform; delete, or gate to `products: [10]` and mark the effect `optional` |
| `disable_program_compat_assistant` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Stock default on 26100 is Automatic (Delayed Start); the `automatic` option clears `DelayedAutostart`. Manual on LTSC 2021 |
| `disable_distributed_link_tracking` | VERIFIED | low | Microsoft-documented | none |
| `disable_retail_demo` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | `requires_reboot: true` missing |
| `disable_wallet_service` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot: true` missing |
| `disable_touch_keyboard` | INCORRECT | medium | Microsoft-documented | `TabletInputService` is absent on the entire primary platform, and the `Win + .` warning is false there |
| `disable_bluetooth` | VERIFIED | medium | Microsoft-documented | Say that `bthserv` is trigger-started (3 registrations), so "Manual" is not dormant |
| `disable_alljoyn_router` | INCORRECT | low | Microsoft-documented | AllJoyn was retired 1 October 2024; `AJRouter` is absent on the entire primary platform |
| `disable_phone_service` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot: true` missing; `PhoneSvc` is trigger-started (3 registrations) |
| `disable_ssdp_upnp` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | `requires_reboot: true` missing |
| `disable_windows_insider` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot: true` missing |
| `disable_maps_broker` | VERIFIED | low | Microsoft-documented | none |
| `disable_geolocation` | VERIFIED | medium | Microsoft-documented | Say that `lfsvc` is trigger-started (4 registrations) |
| `disable_xbox_services` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Split `XboxGipSvc` into its own selectable effect; the all-or-nothing shape makes the tweak's own advice impossible to follow |
| `disable_cdpsvc` | VERIFIED | medium | Microsoft-documented | none |
| `disable_biometrics` | VERIFIED | medium | Microsoft-documented | Say that `WbioSrvc` is trigger-started (2 registrations) |
| `disable_smartcard` | VERIFIED-WITH-CORRECTION | medium, must be **high** | Microsoft-documented | `risk_level` contradicts its own lockout warning; `requires_reboot: true` missing; `SCardSvr` and `ScDeviceEnum` are trigger-started |
| `disable_sensor_services` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot: true` missing; `SensorService` has 8 trigger registrations |
| `disable_parental_controls` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | `requires_reboot: true` missing |
| `disable_payments_nfc` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | `requires_reboot: true` missing; `SEMgrSvc` is trigger-started (2 registrations) |
| `disable_wmp_network_sharing` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Effect needs `optional: true`; the service rides the removable `Media.WindowsMediaPlayer` feature and is absent on IoT Enterprise LTSC; `requires_reboot: true` missing |
| `task_autochk_proxy` | VERIFIED | low | Microsoft-documented | Say the upload is CEIP-gated; cross-reference `privacy:disable_ceip_tasks`, which does not cover this task |
| `task_feedback_dmclient` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | `DmClient` has an empty `<Triggers />` set on 26100, so the copy overstates the first effect |
| `task_wer_queuereporting` | VERIFIED | low | Community-corroborated | none |
| `task_maps_update` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Shipped enabled-state of `MapsUpdateTask` is unestablished, so the stock-default option has no provenance; `MapsToastTask` has an empty `<Triggers />` set and a `ComHandler` action |
| `task_disk_diagnostic_datacollector` | INCORRECT | low | Community-corroborated | Shipped task has an empty `<Triggers />` set, so nothing is scheduled to run; shipped enabled-state is an open question, so the revert value has no provenance |
| `task_device_census` | INCORRECT | low | Community-corroborated | Task path does not exist; the real tasks are `Device` and `Device User` |

Tally: **10 VERIFIED, 14 VERIFIED-WITH-CORRECTION, 5 INCORRECT.** No `UNVERIFIED` and no `DISPUTED`.

## Corrections required

1. **`task_device_census` targets a task path that does not exist.** The YAML writes
   `\Microsoft\Windows\Device Information\Devicecensus`. That folder contains a task named
   **`Device`** (action `devicecensus.exe SystemCxt`, runs daily) and, on builds where it exists, a
   companion **`Device User`** (action `devicecensus.exe UserCxt`, runs at user logon).
   `Devicecensus` is the name of the *executable* (`%WinDir%\System32\devicecensus.exe`), not the
   name of a task. No source at any tier refers to a task named `Devicecensus`. The tweak does
   nothing as authored. Correct the path to `\Microsoft\Windows\Device Information\Device` and add a
   second, `optional: true` effect for `\Microsoft\Windows\Device Information\Device User`, because
   disabling only the system-context task leaves the user-context census running.

2. **`disable_smartcard` declares a risk level that contradicts its own warning.** The `warning`
   field states that on a machine using smart-card sign-in the tweak "removes the only sign-in path
   and can lock the user out", while `risk_level` is `medium`. A lockout that cannot be undone from
   inside Windows is the top of this corpus's scale. Set `risk_level: high`. This is the only tweak
   in the corpus where the declared risk contradicts its own warning text, and it currently sits in
   the same visual bucket as "breaks Bluetooth" at the moment the user decides.

3. **`disable_wmp_network_sharing` targets a service that is not part of the base OS.**
   `WMPNetworkSvc` ships with the removable `Media.WindowsMediaPlayer` optional feature, not with
   Windows itself. On Windows 11 24H2 IoT Enterprise LTSC 2024 the service has no key under
   `HKLM\SYSTEM\CurrentControlSet\Services`, `wmpnetwk.exe` is not on disk, and
   `Get-WindowsCapability -Online` reports `Media.WindowsMediaPlayer~~~~0.0.12.0 = NotPresent`.
   Because the effect is not marked `optional`, the engine reads `Value::Missing`, classifies the
   whole tweak `TweakState::Unknown` with `UnknownCause::MissingRequired`, and refuses to apply it
   with `EngineError::SurfaceUnreadable`; snapshot capture aborts with `CaptureMissingRequired`. The
   tweak is unusable, not merely inapplicable. Add `optional: true` with an `if_missing`, and drop
   the `warning` text's claim that the service "ships present with a Manual/trigger start type on
   both Windows 10 and 11", which is false as an unconditional statement.

4. **Every service tweak needs `requires_reboot: true`, and twelve omit it.** The engine issues only
   `ChangeServiceConfigW`; it never stops a running service and never starts a newly-Automatic one,
   on any path. The twelve missing the flag are `disable_fax`, `disable_retail_demo`,
   `disable_wallet_service`, `disable_alljoyn_router`, `disable_phone_service`, `disable_ssdp_upnp`,
   `disable_windows_insider`, `disable_smartcard`, `disable_sensor_services`,
   `disable_parental_controls`, `disable_payments_nfc` and `disable_wmp_network_sharing`. Without the
   flag the app reports the new state while the old behaviour is still live, which breaks the
   "did-it-work" contract from the user's point of view. It bites hardest on `disable_smartcard`,
   where `SCardSvr` and `ScDeviceEnum` are trigger-started and therefore typically already running on
   a machine with a reader attached. The six task tweaks correctly omit the flag: disabling a
   scheduled task takes effect immediately.

5. **`disable_program_compat_assistant` restores the wrong Automatic variant.** On Windows 11 24H2
   the stock start type for `PcaSvc` is **Automatic (Delayed Start, Trigger Start)**, that is
   `Start=2` with `DelayedAutostart=1`. The YAML's stock option writes plain `automatic`, and
   `drive_service` explicitly writes `DelayedAutostart = 0` for every non-delayed target, so the
   revert genuinely strips the delayed flag and moves `PcaSvc` into the boot start path. Change the
   stock option to `automatic_delayed`. On the secondary target, Windows 10 IoT Enterprise LTSC 2021,
   the stock start type is **Manual (Trigger Start)**, so a version-conditional default is required
   if that platform is kept.

6. **`disable_fax` has no supported platform on the primary target.** The `Fax` service is present
   with default Manual on Windows 10 (so it is real on LTSC 2021) and absent from Windows 11 22H2
   onward, which is the whole primary platform. As authored the tweak has no applicability gate and
   its effect is not `optional`, so on 24H2 it produces the `MissingRequired` failure described in
   correction 3 rather than a silent no-op. Either delete it, or gate it to `products: [10]` and mark
   the effect `optional: true`.

7. **`disable_touch_keyboard` has no supported platform on the primary target, and its warning is
   false there.** `TabletInputService` is present on Windows 10 (Manual, Trigger Start) and absent
   from Windows 11 22H2 onward. On 24H2 the touch keyboard, handwriting panel and the emoji picker
   are served by the Windows Input Experience host (`TextInputHost.exe`), not by a service, so the
   `warning` claim that the tweak breaks `Win + .` cannot be true on the primary platform. Microsoft
   additionally rates this service **"Do not disable"** in its own Server guidance, which is a tier A
   signal against the tweak even on the version where it applies. Delete it, or gate it to
   `products: [10]`, mark the effect `optional: true`, scope the warning to Windows 10, and surface
   Microsoft's rating.

8. **`disable_alljoyn_router` has no supported platform on the primary target.** Microsoft deprecated
   its AllJoyn implementation, explicitly including the AllJoyn Router Service, on 1 September 2023
   and lists it as **retired on 1 October 2024**, which lines up with the 24H2 release. `AJRouter` is
   present with default Manual (Trigger Start) on Windows 10 and absent on 24H2. Delete it, or gate
   it to `products: [10]` with `optional: true`, and add `requires_reboot: true` if kept.

9. **`disable_xbox_services` bundles `XboxGipSvc` with three services that have nothing to do with
   hardware.** `XblAuthManager`, `XblGameSave` and `XboxNetApiSvc` are Xbox Live account and
   networking plumbing that a non-Xbox user never needs, and Microsoft's own Server guidance rates
   two of them "Should be disabled". `XboxGipSvc` is the Xbox Accessory Management Service and is the
   only member of the group tied to physical hardware the user may own; disabling it can break Xbox
   controller behaviour in any launcher, Steam and Epic included. The YAML's own `info` tells the
   user to keep that service while disabling the rest, which the current all-or-nothing option shape
   makes impossible. Split `XboxGipSvc` into its own tweak, or give this tweak a third option that
   disables the three Xbox Live services and leaves the accessory service at Manual. Do not merge
   this tweak with anything further.

10. **`task_disk_diagnostic_datacollector` disables a task that carries no trigger.** The shipped
    definition of `\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector`
    on 26100 has **`<Triggers />`**, an empty trigger element, plus `<Hidden>true</Hidden>` and a
    `MaintenanceSettings` block (`Period` 14 days, `Deadline` 1 month). That observation survives the
    provenance rule, because no enable or disable operation produces an empty trigger set. So there
    is no weekly schedule, and the round-one source for the "weekly trigger" claim is a Windows 7
    task-reference page that does not describe the target platform. The separate claim that the task
    ships `<Enabled>false</Enabled>` is **withdrawn** (see open question 1), which means the
    "Enabled (Stock Default)" option currently has no established provenance for the literal it
    writes. Under the `_harmful-revert.md` rule a revert value must be sourced from Windows rather
    than assumed, so this option is blocked pending a clean-image check. Correct the `info` text now
    to stop describing a recurring upload the task does not have on 24H2, and do not ship the stock
    option until the shipped enabled-state is confirmed.

11. **`task_maps_update`'s stock-default option has no established provenance.** The claim that
    `MapsUpdateTask` ships `<Enabled>false</Enabled>` rested on the modified machine and is
    **withdrawn** (see open question 2). Two facts do survive and need to reach the copy. First,
    `MapsToastTask` has an empty `<Triggers />` set and a `ComHandler` action, so it only ever fires
    when `MapsBroker` invokes it; the tweak does not stop a scheduled toast because there is no
    schedule. Second, Microsoft states that "Maps is no longer preinstalled with Windows starting
    with the Windows 11, version 24H2 release", and deprecated the Maps app in May 2025 and the
    Windows Maps platform APIs on 8 April 2025, so on the primary platform both tasks sit behind an
    app that is not installed. Confirm `MapsUpdateTask`'s shipped enabled-state before trusting
    "Enabled (Stock Default)".

12. **`task_feedback_dmclient` overstates its first effect.** `\Microsoft\Windows\Feedback\Siuf\DmClient`
    has an empty `<Triggers />` set on 26100; only `DmClientOnScenarioDownload` carries a real
    trigger (a `WnfStateChangeTrigger`). Disabling `DmClient` still blocks an on-demand or
    externally-invoked run, so this is not a defect in the mechanism, but the copy's "stops the
    scheduled feedback and diagnostic transmission" framing describes a schedule this build does not
    have. Reword so the user is not told they are switching off a recurring transmission.

13. **The "Manual (Stock Default)" label understates what several tweaks take away.** Manual with a
    hardware or event trigger means the service starts itself when the trigger fires, so moving it to
    Disabled removes a behaviour the user currently has rather than merely declining to start one. On
    26100, `SensorService` has **8** trigger registrations, `lfsvc` and `SCardSvr` have **4** each,
    `bthserv`, `PhoneSvc` and `ScDeviceEnum` have **3** each, and `SEMgrSvc` and `WbioSrvc` have
    **2** each. Every affected `info` block must say the service is trigger-started, because "Manual"
    reads as dormant to a user and it is not. Affected tweaks: `disable_bluetooth`,
    `disable_geolocation`, `disable_biometrics`, `disable_smartcard`, `disable_sensor_services`,
    `disable_phone_service`, `disable_payments_nfc`.

14. **`task_autochk_proxy`'s effect is conditional on CEIP consent, and the copy does not say so.**
    Microsoft's own shipped description string for the task, resource `-102` in
    `%SystemRoot%\System32\acproxy.dll`, reads: "This task collects and uploads autochk SQM data if
    opted-in to the Microsoft Customer Experience Improvement Program." On a machine that never opted
    into CEIP the task uploads nothing, so disabling it produces no observable change. Say so in
    Drawbacks. Related, and belonging to `privacy.md` rather than to this file:
    `privacy:disable_ceip_tasks` is named for CEIP but does not cover this task, even though
    Microsoft's own string makes it a CEIP SQM uploader. That correction is recorded against the
    privacy tweak; this file's job is the cross-reference.

15. **`disable_diagtrack`'s `AllowTelemetry` wording is imprecise.** The `warning` says the policy is
    "floored to 1" on Home and Pro. Microsoft's own statement is that the "Diagnostic data off
    (Security)" level is available only on Enterprise, Education, IoT and Server editions, so on Home
    and Pro the lowest effective level is Required. Same fact, but only the second version is
    sourceable. Restate it.

16. **`disable_parental_controls` is the one tweak here whose misuse harms a third party.** No
    mechanism defect was found, but `risk_level: low` describes the operator's risk, not the
    supervised child's. Applying it on a device with a Family Safety supervised account silently
    stops enforcement with no signal to the parent. Keep the risk level if it is defined as risk to
    the machine, but the copy must state the supervision-bypass consequence plainly. It is in
    Drawbacks in the block below.

## Merge candidates

Only groups where the settings are part of one subsystem and a user would sensibly choose *between*
them rather than combine them.

### 1. Offline Maps: `disable_maps_broker` + `task_maps_update`

The `MapsBroker` service and the `MapsUpdateTask` / `MapsToastTask` pair are one subsystem, and the
YAML's own copy already tells the user to apply them together. `MapsToastTask` is invoked by
`MapsBroker` through a `ComHandler`, so with the service disabled the toast task cannot fire at all,
which makes the current split produce half-states in either order.

**Proposed shape:** one "Offline Maps" tweak with three options: "On (Stock Default)" (`MapsBroker`
Automatic Delayed, both tasks at their shipped enabled-state), "Background updates off" (`MapsBroker`
Automatic Delayed, `MapsUpdateTask` Disabled), "Off" (`MapsBroker` Disabled, both tasks Disabled).

**Granularity lost:** none, provided the three-option shape is used. Collapsing to two options would
lose the ability to keep the Maps platform available to apps while stopping only background map
downloads.

**Blocker:** the stock-default value for `MapsUpdateTask` is an open question (see below). Do not
build the merged "On (Stock Default)" option until that is settled, or the merge inherits the
unsourced revert value instead of fixing it.

### 2. Windows payment stack: `disable_wallet_service` + `disable_payments_nfc`

`WalletService` is the wallet front end and `SEMgrSvc` is the NFC secure-element manager behind
tap-to-pay. They are the two halves of one feature, both default to Manual, both are low risk, and no
user sensibly wants one without the other.

**Proposed shape:** one "Windows payments (Wallet and NFC secure element)" tweak with
Disabled / Manual (Stock Default), `requires_reboot: true`, `risk_level: low`.

**Granularity lost:** essentially none. The only case the merge removes is a user who wants the
wallet UI available while the secure element is unreachable, which is not a coherent configuration.

### 3. Local-network media discovery: `disable_ssdp_upnp` + `disable_wmp_network_sharing`

`WMPNetworkSvc` serves a media library over DLNA, and DLNA discovery *is* SSDP/UPnP, so `SSDPSRV` and
`upnphost` are the transport underneath it. Applying the WMP tweak alone leaves the discovery
transport up; applying the SSDP tweak alone already breaks DLNA serving regardless of the WMP tweak.
A user choosing here is really choosing how much of the local-network media surface to expose.

**Proposed shape:** one tweak with three options: "All on (Stock Default)", "Stop serving media, keep
discovery" (`WMPNetworkSvc` Disabled only), "No SSDP/UPnP at all" (all three Disabled). The
`WMPNetworkSvc` effect must be `optional: true` so the merged tweak still works on images without the
Windows Media Player Legacy feature, which includes the primary IoT Enterprise LTSC target.

**Granularity lost:** none. The three-option shape preserves both existing behaviours and adds the
sensible middle rung that the current split cannot express.

### 4. Telemetry scheduled tasks: `task_autochk_proxy` + `task_feedback_dmclient` + `task_disk_diagnostic_datacollector` + `task_device_census`

Four low-risk toggles, each disabling a scheduled upload of diagnostic data to Microsoft, none with
any user-facing function. Four separate tiles is a lot of surface for what is one decision.

**Proposed shape:** one "Telemetry scheduled tasks" tweak with "Stock Default" / "All disabled", each
task effect marked `optional: true` so a build that lacks one does not make the whole tweak Unknown.

**Granularity lost:** the ability to disable the CEIP uploads while keeping the device census, which
matters to a machine reporting into Windows Update for Business reports or Update Compliance, since
the census is the data source those reports depend on. That is a narrow enterprise case on a
consumer-facing tool; if it is worth keeping, split into "CEIP tasks" (Autochk Proxy, Disk Diagnostic
collector, Feedback SIUF) and leave `task_device_census` standalone.

**`task_wer_queuereporting` is deliberately excluded.** It is the one task in the group with a real
reason to leave enabled: a user may want Microsoft and OEMs to receive their crash dumps while
sending nothing else. Folding it in would take that choice away.

**Cross-references before building this.** `privacy:disable_ceip_tasks` covers `Consolidator` and
`UsbCeip` and is named for CEIP, so a merged CEIP tweak here would overlap it; decide which file owns
the CEIP task set before merging. And `disable_ai_fabric_service` in the new `ai` category is a
service, not a task, so it stays out of this group.

### 5. Split candidate, not a merge: `disable_xbox_services`

Going the other way. `XboxGipSvc` should be its own effect or its own tweak, for the reasons in
correction 9. The current all-or-nothing option shape contradicts the tweak's own advice.

### 6. Explicitly rejected: `disable_biometrics` + `disable_smartcard`

Both remove an alternate credential provider, so they look mergeable. **Do not merge them.** The risk
profiles are not comparable: losing Windows Hello falls back to PIN or password, whereas losing smart
card can be the only sign-in path on a managed machine. One is `medium` and the other must become
`high` (correction 2). A merged tweak would have to carry the higher risk level, which would either
over-warn on biometrics or under-warn on smart cards.

## Open questions needing a clean image

These are claims that were reported as confirmed in an earlier round, whose only evidence was the
modified validation machine. They are **withdrawn**, not disproved. Each remains plausible and each
blocks a revert value, so none may be acted on until a clean Windows 11 24H2 image confirms it.

1. **`task_disk_diagnostic_datacollector`: does the task ship disabled?** The original claim was that
   `Microsoft-Windows-DiskDiagnosticDataCollector` ships `<Enabled>false</Enabled>`, which would make
   "Enabled (Stock Default)" turn on a hidden CEIP collector that Windows shipped switched off.
   Withdrawn, because enabling or disabling a task rewrites exactly that element, so the owner may
   simply have disabled it. The corroborating observation that the sibling
   `Microsoft-Windows-DiskDiagnosticResolver` is also `<Enabled>false</Enabled>` at an untouched
   image-build timestamp is weakened by the same contamination, and file timestamps do not reliably
   survive servicing. **What survives:** the empty `<Triggers />` set, which no enable or disable
   operation produces. That is enough to establish there is no schedule; it is not enough to
   establish the shipped enabled-state.

2. **`task_maps_update`: does `MapsUpdateTask` ship disabled?** Same contamination, same reason. The
   "untouched image-build timestamp" argument is not load-bearing. Until this is settled, "Enabled
   (Stock Default)" writes a literal with no provenance.

3. **Shipped enabled-state of every other task in this file.** No clean-image baseline exists for
   `\Microsoft\Windows\Autochk\Proxy`, the two `Feedback\Siuf\DmClient*` tasks,
   `Windows Error Reporting\QueueReporting`, `Maps\MapsToastTask`, or
   `Device Information\Device` and `Device User`. Absence of an `<Enabled>` element is suggestive
   that a definition was never rewritten, since a disable operation writes
   `<Enabled>false</Enabled>` and a re-enable writes `<Enabled>true</Enabled>`, but it is a weaker
   argument than a clean image and it should not carry a revert value on its own.

4. **SKU spread on `disable_wmp_network_sharing`.** The absence of `Media.WindowsMediaPlayer` is
   confirmed on IoT Enterprise LTSC 2024. Whether retail Home and Pro 26100 images still install that
   feature by default was not verified. The `optional: true` fix is correct either way; what is
   unknown is how many machines the tweak actually reaches.

5. **`PcaSvc` on Windows 10 IoT Enterprise LTSC 2021.** The Manual (Trigger Start) default is sourced
   from Windows 10 22H2 clean-install dumps and per-release tables, not from an LTSC 2021 image
   specifically. LTSC 2021's feature baseline is 21H2, where the same tables agree, so this is a low
   risk, but if the version-conditional default in correction 5 is implemented it should be checked
   against a real LTSC image.

**How to close these cheaply.** Mount a clean 24H2 `install.wim` and read the task XML under
`Windows\System32\Tasks` offline, plus the offline `SYSTEM` hive for service `Start` values. That
gives the true shipped state without booting anything, and it answers items 1 through 3 and item 5 in
one pass. A Windows 11 24H2 evaluation VM answers item 4 as well.

## Tweak entries

### `disable_diagtrack` Connected User Experiences and Telemetry (DiagTrack)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `DiagTrack` (display name "Connected User Experiences and Telemetry"), hosted
  in the `utcsvc` svchost group as LocalSystem.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Automatic (Stock Default)" writes start type Automatic (`Start=2`, `DelayedAutostart=0`).
- Documented stock start type: **Automatic**, not delayed and not trigger-started, on every release
  checked. Microsoft's Server guidance lists it as always installed, Automatic, "No guidance".

**Corrections needed:** One, and it is wording only. The `warning` field says the `AllowTelemetry=0`
policy is "floored to 1" on Home and Pro. Restate it in Microsoft's own terms: the "Diagnostic data
off (Security)" level is available only on Enterprise, Education, IoT and Server editions, so the
lowest effective level on Home and Pro is Required. Same fact, sourceable version. The mechanism,
the stock start type and `requires_reboot: true` are all correct.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the service that collects Windows diagnostic and usage data and uploads it to Microsoft.**

      ## What it does
      Sets the Connected User Experiences and Telemetry service (`DiagTrack`) to Disabled. This is
      the component Microsoft documents as managing diagnostic data events and diagnostic logs sent
      back to Microsoft, so disabling it stops the collection and upload pipeline itself rather than
      lowering the amount of data it is allowed to send.

      ## Benefits
      - **Stops the transmitter**: the pipeline does not run, rather than running at a lower level
      - **Works on every edition**: the "Diagnostic data off" policy level is Enterprise, Education, IoT and Server only
      - **Quieter machine**: removes the background network and disk activity the uploader generates

      ## Drawbacks
      - **Feedback Hub breaks**: diagnostic submission through Feedback Hub stops working
      - **Enterprise signals stop**: Defender for Endpoint, Windows Update for Business reports and Update Compliance all read this pipeline
      - **Updates can undo it**: feature updates re-provision service configuration and are widely reported to restore it to Automatic
      - **No speed gain**: the honest benefit is privacy, not performance

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Automatic
      - **Pairs with**: the diagnostic data policy tweak in Privacy, which caps the level but cannot stop the transmitter on Home or Pro

      ## Recommendation
      Apply it on a personal machine where privacy matters more than Feedback Hub. Do not apply it on
      a machine managed by an IT department or enrolled in Defender for Endpoint, where the
      diagnostic pipeline is load-bearing.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
````

**Sources:**
1. Configure Windows diagnostic data in your organization, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
2. Manage connections from Windows operating system components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Guidelines for disabling system services in Windows Server with Desktop Experience, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
4. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)

### `disable_print_spooler` Print Spooler (Spooler)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `Spooler` (display name "Print Spooler"), `%WinDir%\System32\spoolsv.exe`,
  running as LocalSystem in its own process.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Automatic (Stock Default)" writes start type Automatic (`Start=2`).
- Documented stock start type: **Automatic**. Microsoft's Server guidance gives the same default and
  rates it "OK to disable if not a print server or a domain controller".

**Corrections needed:** `none`. `requires_reboot: true` is present and correct, the stock start type
is right, and the warning text accurately describes total loss of printing.

**Ready-to-paste info block:**

````yaml
    info: |
      **Removes all printing and shuts down the component repeatedly hit by remote-code-execution bugs.**

      ## What it does
      Sets the Print Spooler service (`Spooler`) to Disabled. The spooler queues every print job and
      mediates all interaction with printer drivers and print devices, so with it disabled Windows
      cannot talk to any printer, physical or virtual.

      ## Benefits
      - **Removes an attack surface**: the spooler has produced a sustained run of RCE and privilege-escalation flaws, PrintNightmare among them
      - **Kills the whole class**: disabling the service removes the surface outright rather than mitigating one bug
      - **One less service**: an Automatic-start service stops loading at boot

      ## Drawbacks
      - **All printing breaks**: including Microsoft Print to PDF and Print to XPS, not just physical printers
      - **Printers vanish from apps**: printer lists disappear from every application print dialog
      - **Export flows break**: label and receipt software, some PDF export paths, and some scanner suites route through the spooler
      - **Not a patch substitute**: disabling is a documented mitigation, not a replacement for installing the updates

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Automatic
      - Unsupported by Microsoft on a domain controller or a print server

      ## Recommendation
      Apply it on a machine that genuinely never prints, such as a kiosk, a server, or a hardened
      workstation. Do not apply it if you ever print or export to PDF through the print dialog, since
      there is no partial mode.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [KB5005010: restricting installation of new printer drivers](https://support.microsoft.com/en-us/topic/kb5005010-restricting-installation-of-new-printer-drivers-after-applying-the-july-6-2021-updates-31b91c02-05bc-4ada-a7ea-183b129578a7)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. KB5005010: Restricting installation of new printer drivers after applying the July 6, 2021 updates, https://support.microsoft.com/en-us/topic/kb5005010-restricting-installation-of-new-printer-drivers-after-applying-the-july-6-2021-updates-31b91c02-05bc-4ada-a7ea-183b129578a7 (tier A)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)

### `disable_fax` Fax Service (Fax)

**Verdict:** INCORRECT (on the primary platform)

**Mechanism, corrected:**
- Service short name: `Fax`, `%WinDir%\system32\fxssvc.exe`, running as NetworkService in its own
  process. It backs Windows Fax and Scan.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual**, on Windows 10 only. The service is **absent** from
  Windows 11 22H2 onward, which covers the entire primary platform, because Windows Fax and Scan
  stopped being installed by default from 22H2.
- Corrected applicability: Windows 10 IoT Enterprise LTSC 2021 only, and the effect must be
  `optional: true` because even there the service depends on the optional feature being present.

**Mechanism as authored (wrong):** the same service and options, with **no `windows` gate and no
`optional` flag**, presented as universally applicable. On 24H2 the service does not exist, so
`read_service` returns `Value::Missing`, the engine classifies the tweak `Unknown` with
`UnknownCause::MissingRequired`, and apply fails with `EngineError::SurfaceUnreadable`. It is an
unusable tile, not a quiet no-op. `requires_reboot` is also absent.

**Corrections needed:** Three. (1) The tweak has no supported platform on Windows 11 24H2 or newer;
delete it, or gate it to `windows: { products: [10] }`. (2) If kept, mark the effect `optional: true`
with an `if_missing`, because the service is carried by an optional feature rather than by the base
OS. (3) Add `requires_reboot: true`, per correction 4. The `info` text's framing as a safe,
universally applicable cleanup is misleading on the primary platform and must be scoped.

**Ready-to-paste info block** (written for the corrected, Windows 10 gated form):

````yaml
    info: |
      **Turns off the legacy fax service on a machine that has no fax hardware.**

      ## What it does
      Sets the Fax service (`Fax`) to Disabled. The service sends and receives faxes through a fax
      modem or a network fax device and is what Windows Fax and Scan talks to. Document scanning
      through other applications does not go through it.

      ## Benefits
      - **Removes a legacy service**: nothing on a modern PC uses fax
      - **Smaller service list**: one less entry to audit on a hardened machine
      - **No side effects**: scanning, printing and imaging are unaffected

      ## Drawbacks
      - **Faxing stops**: sending and receiving through Windows Fax and Scan no longer works
      - **Not present on Windows 11**: the service is absent from Windows 11 22H2 and newer, so this control does nothing there
      - **Depends on an optional feature**: if Windows Fax and Scan was never installed, there is no service to change

      ## Good to know
      - **Applies to**: Windows 10 IoT Enterprise LTSC 2021 only; the service does not exist on Windows 11 24H2 or newer
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - The tile reports Unknown rather than applying if the service is not installed

      ## Recommendation
      Apply it on a Windows 10 LTSC machine with no fax hardware, where it is free. Skip it entirely
      on Windows 11, where there is nothing to disable.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Windows 10 default services configuration, clean install](https://www.winhelponline.com/blog/windows-10-default-services-configuration/)
      - [Windows 11 default services configuration, clean install, Fax absent](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Fax service, Windows 11 per-release table](https://batcmd.com/windows/11/services/fax/)
````

**Sources:**
1. Windows 10 Default Services Configuration (Windows 10 Pro 22H2 clean install, `Fax` = Manual), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
2. Windows 11 Default Services Configuration (Windows 11 Pro 23H2 clean install, `Fax` absent), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Fax, Windows 11 service reference (21H2 Manual; 22H2, 23H2 and 24H2 recorded as "not exists"), https://batcmd.com/windows/11/services/fax/ (tier C)
4. `src-tauri/src/tweaks/kinds/service.rs`, `drive_service` and `read_service` (missing service yields `ResourceMissing` on drive and `Value::Missing` on read) (tier A, this repository)

### `disable_program_compat_assistant` Program Compatibility Assistant Service (PcaSvc)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism, corrected:**
- Service short name: `PcaSvc` (display name "Program Compatibility Assistant Service"),
  `%WinDir%\System32\pcasvc.dll`, hosted in the `LocalSystemNetworkRestricted` svchost group as
  LocalSystem.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Stock Default" must write **`automatic_delayed`** on the primary platform, that is
  `Start=2` with `DelayedAutostart=1`.
- Documented stock start type on Windows 11 21H2 through 24H2: **Automatic (Delayed Start, Trigger
  Start)**. On Windows 10, including the LTSC 2021 baseline: **Manual (Trigger Start)**.

**Mechanism as authored (wrong):** the stock option writes plain `automatic`. Because `drive_service`
always writes the `DelayedAutostart` companion value explicitly, driving to `automatic` writes
`DelayedAutostart = 0`, so the revert genuinely strips the delayed flag and moves `PcaSvc` from the
post-boot queue into the boot start path, a state the machine was never in. On Windows 10 the same
option leaves the service Automatic when stock is Manual, so it starts at every boot from then on.

**Corrections needed:** One, with two halves. Change the stock option to `automatic_delayed` for the
primary platform, and make it version-conditional (Manual on Windows 10) if the secondary platform is
kept. Separately, the `info` claim that "Windows 11 24H2 leans on `PcaSvc` more heavily" is
directionally supported by the existence of `PcaPatchDbTask` under Application Experience but is not
backed by a Microsoft statement; soften it or cite it.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the compatibility pop-ups and the background monitoring of every program launch.**

      ## What it does
      Sets the Program Compatibility Assistant service (`PcaSvc`) to Disabled. PCA watches programs
      as they install and run, detects known compatibility problems, applies automatic compatibility
      shims, and raises prompts such as "This app may not have installed correctly".

      ## Benefits
      - **No compatibility prompts**: the "may not have installed correctly" dialog stops appearing
      - **No launch monitoring**: PCA stops inspecting each program start
      - **Microsoft rates it optional**: the Server guidance marks this service "OK to disable"

      ## Drawbacks
      - **Shims stop applying**: legacy programs that quietly relied on automatic fixes may misbehave
      - **Hard to attribute**: the failure is invisible until an old program breaks, and the cause is not obvious
      - **App compat surface on 24H2**: the `PcaPatchDbTask` scheduled task maintains the compatibility database PCA consumes, so the newest builds lean on this path more

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, where the stock start type is Automatic (Delayed Start); on Windows 10 the stock start type is Manual
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot, including the delayed-start flag
      - The service is trigger-started as well as delayed, so it does start itself on demand

      ## Recommendation
      Apply it if the prompts annoy you and you do not run finicky legacy software. Leave it enabled
      if you depend on old applications, since automatic shims are exactly what keeps some of them
      working.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, PcaSvc as Auto (Delayed, Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Program Compatibility Assistant Service per-release table](https://batcmd.com/windows/11/services/pcasvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`PcaSvc`: Desktop Experience only, Automatic, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 11 Default Services Configuration (Windows 11 Pro 23H2 clean install, `PcaSvc` = Auto (Delayed, Triggered)), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Windows 10 Default Services Configuration (Windows 10 Pro 22H2 clean install, `PcaSvc` = Manual), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
4. Program Compatibility Assistant Service per-release tables, https://batcmd.com/windows/10/services/pcasvc/ and https://batcmd.com/windows/11/services/pcasvc/ (tier C)
5. `src-tauri/src/tweaks/kinds/service.rs`, `drive_service` (always writes `DelayedAutostart` explicitly for every target type) (tier A, this repository)

### `disable_distributed_link_tracking` Distributed Link Tracking Client (TrkWks)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `TrkWks` (display name "Distributed Link Tracking Client").
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Automatic (Stock Default)" writes start type Automatic (`Start=2`).
- Documented stock start type: **Automatic**, not delayed and not trigger-started. Microsoft's Server
  guidance lists it as Desktop Experience only, Automatic, "No guidance".

**Corrections needed:** `none`. `requires_reboot: true` is present, the stock start type is right,
and the mechanism matches. One framing note for the rewrite: Microsoft offers no guidance either way
for this service, so the case for disabling rests on the absence of a dependency rather than on an
endorsement. The copy should not imply Microsoft recommends it.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the service that repairs shortcuts and document links when files move between NTFS drives.**

      ## What it does
      Sets the Distributed Link Tracking Client service (`TrkWks`) to Disabled. The service tracks the
      NTFS object identifier on a file so shortcuts and embedded OLE links can be repaired when the
      target is moved or renamed within or across NTFS volumes on the same machine.

      ## Benefits
      - **One less boot service**: it is Automatic, so it loads on every start
      - **Rarely exercised**: the repair scenario needs files moved across NTFS volumes, which most people never do
      - **No dependency**: nothing else in Windows requires this service

      ## Drawbacks
      - **Links stop self-repairing**: a shortcut whose target moved will no longer find it
      - **Confusing failure**: the broken shortcut is easy to fix by hand but hard to connect to this change
      - **OLE links affected**: embedded links in documents lose the same automatic repair
      - **No Microsoft endorsement**: the Server guidance gives "No guidance" for this service, so this rests on absence of dependency

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Automatic
      - The service is not trigger-started, so on a stock machine it really is running

      ## Recommendation
      Apply it on a single-drive machine or any system where shortcuts point at fixed locations. Leave
      it enabled if you shuffle files between NTFS volumes and rely on shortcuts or linked documents
      following them.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, clean install](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Distributed Link Tracking Client per-release table](https://batcmd.com/windows/11/services/trkwks/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Distributed Link Tracking Client per-release tables, https://batcmd.com/windows/10/services/trkwks/ and https://batcmd.com/windows/11/services/trkwks/ (tier C)

### `disable_retail_demo` Retail Demo Service (RetailDemo)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short name: `RetailDemo` (display name "Retail Demo Service").
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual**, not trigger-started. Not covered by Microsoft's Windows
  Server service guidance, which is expected because Retail Demo Experience is a client-only feature.

**Corrections needed:** One. `requires_reboot: true` is missing, per correction 4. No defect in the
service name, the options or the stock start type.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the in-store demonstration mode that only ever runs on shop display units.**

      ## What it does
      Sets the Retail Demo service (`RetailDemo`) to Disabled. The service supports Retail Demo
      Experience, the scripted demonstration mode a retailer puts on a display machine. On a PC you
      own and have never placed into demo mode, it has nothing to do.

      ## Benefits
      - **No purpose on your PC**: the feature only applies to shop display units
      - **Shorter service list**: one fewer entry when auditing what can start
      - **Fully reversible**: nothing else depends on it

      ## Drawbacks
      - **Retail demo stops**: a genuine display unit would lose its demonstration mode
      - **No measurable gain**: a Manual service that is never triggered is not consuming resources, so expect no speed change
      - **Nothing visible changes**: there is no user-facing difference to look for after applying

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - This is a tidy-up, not a performance change

      ## Recommendation
      Apply it on any personally owned machine; there is no scenario where a home or work PC needs
      Retail Demo. Skip it only on an actual in-store display unit.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Windows 11 default services configuration, clean install](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Retail Demo Service per-release table](https://batcmd.com/windows/11/services/retaildemo/)
      - [Retail Demo Service defaults](https://revertservice.com/11/retaildemo/)
````

**Sources:**
1. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (`RetailDemo` = Manual), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
2. Retail Demo Service per-release tables, https://batcmd.com/windows/10/services/retaildemo/ and https://batcmd.com/windows/11/services/retaildemo/ (tier C)
3. Retail Demo Service (RetailDemo) defaults, https://revertservice.com/10/retaildemo/ and https://revertservice.com/11/retaildemo/ (tier C)

### `disable_wallet_service` Wallet Service (WalletService)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short name: `WalletService` (display name "WalletService"), hosted in the `appmodel`
  svchost group as LocalSystem.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual**. Microsoft's Server guidance lists it as Desktop Experience
  only, Manual, and rates it **"OK to disable"**.

**Corrections needed:** One. `requires_reboot: true` is missing, per correction 4. See merge
candidate 2: this belongs with `disable_payments_nfc`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the service behind Microsoft Wallet and tap-to-pay from Windows.**

      ## What it does
      Sets the Wallet service (`WalletService`) to Disabled. It backs the Windows wallet and
      tap-to-pay payment features on hardware that supports them. It has nothing to do with card
      details saved in a web browser, which keep working normally.

      ## Benefits
      - **Microsoft rates it safe**: the Server guidance marks this service "OK to disable"
      - **No hardware to serve**: a desktop with no payment hardware gets nothing from it
      - **Rarely used feature**: paying from a Windows PC is uncommon

      ## Drawbacks
      - **Wallet payments stop**: Microsoft Wallet and tap-to-pay from this device no longer work
      - **Half the stack**: the NFC secure element is a separate service, so disabling only this one leaves the other half running
      - **Nothing visible changes**: on a machine with no payment hardware there is no observable difference

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - **Pairs with**: the Payments and NFC secure element tweak, which covers the other half

      ## Recommendation
      Apply it on any machine where you do not pay from Windows, which is nearly every desktop. Leave
      it enabled if you use Microsoft Wallet or tap-to-pay on a laptop or tablet with NFC.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, clean install](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [WalletService per-release table](https://batcmd.com/windows/11/services/walletservice/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`WalletService`: Manual, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. WalletService per-release table, https://batcmd.com/windows/11/services/walletservice/ (tier C)

### `disable_touch_keyboard` Touch Keyboard and Handwriting Service (TabletInputService)

**Verdict:** INCORRECT (on the primary platform)

**Mechanism, corrected:**
- Service short name: `TabletInputService` (display name "Touch Keyboard and Handwriting Panel
  Service"), `%WinDir%\System32\TabSvc.dll`, hosted in the `LocalSystemNetworkRestricted` svchost
  group and a member of the `PlugPlay` group.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)**, on Windows 10 only. The service is
  **absent** on Windows 11 22H2, 23H2 and 24H2 in all editions, which is the entire primary platform.
- Corrected applicability: Windows 10 IoT Enterprise LTSC 2021 only, with the effect marked
  `optional: true`.

**Mechanism as authored (wrong):** the same service and options with **no `windows` gate and no
`optional` flag**, plus a `warning` claiming the tweak "breaks the on-screen touch keyboard,
handwriting/ink, and the emoji and symbol panel (Win + .)". On Windows 11 22H2 and later those
surfaces are served by the Windows Input Experience host process (`TextInputHost.exe`), not by this
service, so on the primary platform the warning describes breakage the tweak cannot cause. And
because the service is absent there, the engine marks the tweak Unknown and refuses to apply it.

**Corrections needed:** Three. (1) No supported platform on Windows 11 24H2 or newer; delete, or gate
to `windows: { products: [10] }` with `optional: true`. (2) Scope the `Win + .` warning to
Windows 10; it is a false warning on the primary platform. (3) Surface Microsoft's own rating: the
Server guidance rates `TabletInputService` **"Do not disable"**, which is a tier A signal against
this tweak even where it applies, and the current copy presents the change as a matter of taste.

**Ready-to-paste info block** (written for the corrected, Windows 10 gated form):

````yaml
    info: |
      **Turns off the touch keyboard, pen input and emoji panel on a desktop that only uses a physical keyboard.**

      ## What it does
      Sets the Touch Keyboard and Handwriting Panel service (`TabletInputService`) to Disabled. On
      Windows 10 this service provides the on-screen keyboard, pen and handwriting input, and the
      emoji and symbol panel opened with Win and period.

      ## Benefits
      - **Unused on a desktop**: a keyboard-and-mouse machine never invokes these surfaces
      - **One less service**: removes an input-stack service from the start path

      ## Drawbacks
      - **Emoji picker breaks**: Win and period stops opening the emoji and symbol panel
      - **Touch keyboard breaks**: the on-screen keyboard and the handwriting and ink panel stop working
      - **Microsoft says do not**: the Server guidance rates this service "Do not disable"
      - **Costs nothing when idle**: the service is trigger-started, so on a machine with no touch or pen it is not running anyway

      ## Good to know
      - **Applies to**: Windows 10 IoT Enterprise LTSC 2021 only; the service does not exist on Windows 11 22H2 or newer, where these features come from the Windows Input Experience host instead
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - The tile reports Unknown rather than applying if the service is not installed

      ## Recommendation
      Skip this one. Microsoft rates the service "Do not disable", a trigger-started service costs
      nothing when idle, and the emoji picker is a real loss for a small gain. If you want it anyway,
      apply it only on a Windows 10 desktop with no touch or pen and no use of Win and period.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, TabletInputService rated Do not disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, TabletInputService absent](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Touch Keyboard and Handwriting Panel Service per-release table](https://batcmd.com/windows/11/services/tabletinputservice/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`TabletInputService`: Manual, **Do not disable**), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Microsoft Q&A, "The Touch Keyboard and Handwriting Panel Service is missing when I installed Windows 11", https://learn.microsoft.com/en-us/answers/questions/4147866/the-touch-keyboard-and-handwriting-panel-service-i (tier A host, community-authored content)
3. Windows 11 Default Services Configuration (Windows 11 Pro 23H2 clean install, `TabletInputService` absent), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Touch Keyboard and Handwriting Panel Service, Windows 11 per-release table (21H2 Manual; 22H2, 23H2, 24H2 "not exists"), https://batcmd.com/windows/11/services/tabletinputservice/ (tier C)
5. Windows 10 Default Services Configuration (Windows 10 Pro 22H2 clean install, Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)

### `disable_bluetooth` Bluetooth Support Services (bthserv)

**Verdict:** VERIFIED

**Mechanism:**
- Service short names, three effects:
  - `bthserv` (Bluetooth Support Service), which handles discovery and association of remote devices.
  - `BTAGService` (Bluetooth Audio Gateway Service), the hands-free audio gateway role.
  - `BthAvctpSvc` (AVCTP service), the Audio/Video Control Transport Protocol used by Bluetooth audio
    and remote control.
- Option "Disabled" writes start type Disabled (`Start=4`) on all three.
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`) on all three.
- Documented stock start type: **Manual (Trigger Start)** for all three. `bthserv` carries **3**
  trigger registrations on 26100. Microsoft's Server guidance covers `bthserv` (Desktop Experience
  only, Manual, "OK to disable") and does not cover the two audio-path services.

**Corrections needed:** None to the mechanism, the service list, the options or the stock start type,
and `requires_reboot: true` is already present. One copy correction, per correction 13: the info must
say `bthserv` is trigger-started, because "Manual" reads as dormant and this service does start
itself when Bluetooth hardware or a pairing event appears. Setting `Start=4` does override trigger
registration, so the disable is genuinely effective.

**Ready-to-paste info block:**

````yaml
    info: |
      **Shuts down the whole Windows Bluetooth stack on a machine that has no Bluetooth hardware.**

      ## What it does
      Sets all three Bluetooth services to Disabled: `bthserv` (device discovery and pairing),
      `BTAGService` (the hands-free audio gateway) and `BthAvctpSvc` (the audio and remote-control
      transport). Together they are the Bluetooth stack, so disabling them prevents it from starting
      at all.

      ## Benefits
      - **Removes a radio surface**: nothing can bring the Bluetooth stack up, which is a hardening guarantee rather than a speed gain
      - **Microsoft rates it safe**: the Server guidance marks `bthserv` "OK to disable"
      - **Correctly grouped**: all three go together, so you do not end up with a half-working stack

      ## Drawbacks
      - **All Bluetooth breaks**: mice, keyboards, headphones, game controllers and file transfer all stop working
      - **Input lockout risk**: on a machine whose only mouse or keyboard is Bluetooth, you can be left unable to interact with it after the reboot
      - **Not dormant today**: these are trigger-started, so on a machine with a Bluetooth radio they do start themselves and you are removing live behaviour
      - **No performance gain**: on a machine with no radio they never start, so nothing is being saved

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start types from the snapshot; the stock value is Manual for all three
      - Disabled overrides trigger registration, so the stack genuinely cannot start

      ## Recommendation
      Apply it on a wired desktop with no Bluetooth radio and no wireless peripherals. Never apply it
      on a laptop, or on any machine with a Bluetooth mouse, keyboard, headset or controller, and
      confirm you have a wired input device before you reboot.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, bthserv rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, all three Manual (Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Bluetooth Support Service per-release table](https://batcmd.com/windows/11/services/bthserv/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`bthserv`: Manual, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (all three = Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. bthserv, BTAGService and BthAvctpSvc per-release tables, https://batcmd.com/windows/11/services/bthserv/ (tier C)
4. Trigger registration count for `bthserv` (3) read from `HKLM\SYSTEM\CurrentControlSet\Services\bthserv\TriggerInfo` on build 26100 (tier A for the existence of trigger registration, which `ChangeServiceConfigW` cannot create or remove; not used as evidence of any default start type)

### `disable_alljoyn_router` AllJoyn Router Service (AJRouter)

**Verdict:** INCORRECT (on the primary platform)

**Mechanism, corrected:**
- Service short name: `AJRouter` (display name "AllJoyn Router Service"), which routes AllJoyn
  messages for local AllJoyn clients.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)** on Windows 10 and on Windows 11 21H2
  through 23H2. **Absent on Windows 11 24H2 and newer.**
- Corrected applicability: Windows 10 IoT Enterprise LTSC 2021 only, with the effect marked
  `optional: true`.

**Mechanism as authored (wrong):** the same service and options with **no `windows` gate, no
`optional` flag and no `requires_reboot`**, presented as universally applicable. Microsoft deprecated
its AllJoyn implementation, explicitly including the AllJoyn Router Service, on 1 September 2023 and
lists it as **retired on 1 October 2024**, which lines up with the 24H2 release. On the primary
platform the service does not exist, so the engine marks the tweak Unknown and refuses to apply it.

**Corrections needed:** Three. (1) No supported platform on Windows 11 24H2 or newer; delete, or gate
to `windows: { products: [10] }`. (2) If kept, mark the effect `optional: true`. (3) Add
`requires_reboot: true`, per correction 4.

**Ready-to-paste info block** (written for the corrected, Windows 10 gated form):

````yaml
    info: |
      **Turns off the router for AllJoyn, an IoT device protocol Microsoft has since retired.**

      ## What it does
      Sets the AllJoyn Router service (`AJRouter`) to Disabled. AllJoyn was an open discovery and
      messaging protocol for Internet-of-Things devices, and this service routed its messages for
      local clients. Microsoft retired its AllJoyn implementation on 1 October 2024.

      ## Benefits
      - **Retired technology**: Microsoft has removed AllJoyn from Windows, so nothing new will use it
      - **Trigger-started service**: closes a listener path that would otherwise start itself on demand
      - **No dependency**: nothing in normal networking or internet use touches AllJoyn

      ## Drawbacks
      - **AllJoyn devices stop**: interop with AllJoyn-based smart devices ends, which is rare but real on older setups
      - **Not present on Windows 11**: the service is absent from Windows 11 24H2 and newer, so this control does nothing there
      - **Nothing visible changes**: on a machine with no AllJoyn devices there is no observable difference

      ## Good to know
      - **Applies to**: Windows 10 IoT Enterprise LTSC 2021 only; the service does not exist on Windows 11 24H2 or newer
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - The tile reports Unknown rather than applying if the service is not installed

      ## Recommendation
      Apply it on a Windows 10 LTSC machine with no AllJoyn smart devices, which is almost all of
      them. Skip it on Windows 11, where the service was removed with the protocol.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Features removed in Windows client, AllJoyn including the AllJoyn Router Service retired 1 October 2024](https://learn.microsoft.com/en-us/windows/whats-new/removed-features)
      - [Deprecated features in the Windows client, AllJoyn deprecated 1 September 2023](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features)
      - [AllJoyn Router Service per-release table](https://batcmd.com/windows/11/services/ajrouter/)
````

**Sources:**
1. Features and functionality removed in Windows client (AllJoyn, including the AllJoyn Router Service, retired 1 October 2024), https://learn.microsoft.com/en-us/windows/whats-new/removed-features (tier A)
2. Deprecated features in the Windows client (AllJoyn deprecated 1 September 2023), https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
3. Guidelines for disabling system services in Windows Server with Desktop Experience (`AJRouter`: Desktop Experience only, Manual, No guidance), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
4. AllJoyn Router Service, Windows 11 per-release table (24H2 "not exists"), https://batcmd.com/windows/11/services/ajrouter/ (tier C)
5. Windows 10 Default Services Configuration (`AJRouter` = Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)

### `disable_phone_service` Phone Service (PhoneSvc)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short name: `PhoneSvc` (display name "Phone Service"), which manages device telephony
  state for hardware with a cellular modem.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)**, with **3** trigger registrations on 26100.
  Microsoft's Server guidance lists it as Desktop Experience only, Manual, and rates it
  **"OK to disable"**.

**Corrections needed:** Two. (1) `requires_reboot: true` is missing, per correction 4. (2) The info
must say the service is trigger-started, per correction 13. The service name, the options and the
stock start type are correct, and the existing clarification that this is not the Phone Link app is
accurate and must survive.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the cellular telephony service on a machine that has no mobile modem.**

      ## What it does
      Sets the Phone service (`PhoneSvc`) to Disabled. It manages device telephony state, meaning the
      cellular and phone-call capability of hardware that has a mobile modem. It is not the Phone
      Link app that mirrors your Android or iPhone, which is unaffected.

      ## Benefits
      - **Microsoft rates it safe**: the Server guidance marks this service "OK to disable"
      - **No hardware to manage**: a desktop or a Wi-Fi-only laptop has no cellular radio
      - **Removes a trigger**: the service currently starts itself when a telephony event fires

      ## Drawbacks
      - **Cellular telephony stops**: on a device with a built-in modem, phone and call state handling breaks
      - **Not dormant today**: it is trigger-started with three registrations, so you are removing behaviour rather than declining to start something
      - **Nothing visible changes**: on a machine with no modem there is no observable difference

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - This does not affect Phone Link, which uses a different service

      ## Recommendation
      Apply it on a desktop or any laptop without a cellular modem. Leave it enabled on a tablet,
      always-connected laptop or any device with built-in mobile broadband.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, PhoneSvc rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, clean install](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Phone Service per-release table](https://batcmd.com/windows/11/services/phonesvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`PhoneSvc`: Manual, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Phone Service per-release table, https://batcmd.com/windows/11/services/phonesvc/ (tier C)
4. Trigger registration count for `PhoneSvc` (3) read from `HKLM\SYSTEM\CurrentControlSet\Services\PhoneSvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### `disable_ssdp_upnp` SSDP Discovery and UPnP Device Host (SSDPSRV/upnphost)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short names, two effects:
  - `SSDPSRV` (SSDP Discovery), which discovers network devices and services using the SSDP
    discovery protocol and also announces SSDP devices running on this machine.
  - `upnphost` (UPnP Device Host), running as LocalService in the
    `LocalServiceAndNoImpersonation` group, which hosts UPnP devices on this machine.
- Option "Disabled" writes start type Disabled (`Start=4`) on both.
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`) on both.
- Documented stock start type: **Manual** for both, and unusually for this file, neither is
  trigger-started. Microsoft's Server guidance lists both as Desktop Experience only, Manual, and
  rates both **"OK to disable"**.

**Corrections needed:** One. `requires_reboot: true` is missing, per correction 4, and it matters
here because both services can be running at apply time on a machine that casts or streams. See merge
candidate 3: this belongs with `disable_wmp_network_sharing`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows discovering and advertising devices over SSDP and UPnP on your local network.**

      ## What it does
      Sets the SSDP Discovery service (`SSDPSRV`) and the UPnP Device Host (`upnphost`) to Disabled.
      The first discovers and announces devices using the SSDP protocol; the second hosts UPnP
      devices on this machine. Together they are how Windows finds and advertises itself to smart
      TVs, media renderers and UPnP-capable routers.

      ## Benefits
      - **Smaller local footprint**: your PC stops announcing itself to everything on the LAN
      - **Microsoft rates both safe**: the Server guidance marks both services "OK to disable"
      - **Known-risky protocol**: UPnP has a long history of local-network exposure issues

      ## Drawbacks
      - **Cast to Device breaks**: streaming media from Windows to a TV stops working
      - **DLNA discovery breaks**: UPnP and DLNA devices no longer appear, in either direction
      - **Router auto-config breaks**: applications that open ports through UPnP, including some games and torrent clients, must be configured by hand
      - **Smart-home discovery**: some smart-home apps that find devices over SSDP stop seeing them

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start types from the snapshot; the stock value is Manual for both
      - Neither service is trigger-started, so on a machine that casts they are genuinely running

      ## Recommendation
      Apply it on a security-focused machine that does not cast, stream or rely on UPnP port
      forwarding. Leave it enabled if you use Cast to Device, DLNA, or a game or client that opens
      ports through the router automatically.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, both rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, both Manual](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [SSDP Discovery per-release table](https://batcmd.com/windows/11/services/ssdpsrv/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`SSDPSRV` and `upnphost`: Manual, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (both = Manual, neither trigger-started), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. SSDP Discovery and UPnP Device Host per-release tables, https://batcmd.com/windows/11/services/ssdpsrv/ and https://batcmd.com/windows/11/services/upnphost/ (tier C)

### `disable_windows_insider` Windows Insider Service (wisvc)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short name: `wisvc` (display name "Windows Insider Service"), which provides
  infrastructure support for the Windows Insider Program: enrolment, channel selection and delivery
  of preview builds.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)**. Microsoft's Server guidance lists it as
  **always installed**, Manual, and rates it **"OK to disable"**.

**Corrections needed:** One. `requires_reboot: true` is missing, per correction 4. The service name,
the options and the stock start type are correct, and the statement that normal Windows Update
servicing does not depend on this service is accurate.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the service that enrols this PC in Windows Insider preview builds.**

      ## What it does
      Sets the Windows Insider service (`wisvc`) to Disabled. It handles enrolment in the Windows
      Insider Program, channel selection, and delivery of preview builds. Normal stable-channel
      Windows Update servicing does not go through it.

      ## Benefits
      - **Microsoft rates it safe**: the Server guidance marks this service "OK to disable"
      - **No function on stable**: it does nothing on a machine kept on the retail channel
      - **Blocks accidental enrolment**: nobody can put this machine on a preview build without reverting first

      ## Drawbacks
      - **Insider enrolment breaks**: you cannot join the program or receive preview builds
      - **Silent failure in Settings**: the Insider Program page will fail to enrol without explaining why
      - **Nothing visible changes**: on a stable-channel machine there is no observable difference

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - Revert this first if you later decide to join the Insider Program

      ## Recommendation
      Apply it on any machine you keep on stable Windows, which is nearly all of them. Leave it
      enabled if you are an Insider or plan to become one.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, wisvc rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, clean install](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Windows Insider Service per-release table](https://batcmd.com/windows/11/services/wisvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`wisvc`: always installed, Manual, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Windows Insider Service per-release table, https://batcmd.com/windows/11/services/wisvc/ (tier C)

### `disable_maps_broker` Downloaded Maps Manager (MapsBroker)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `MapsBroker` (display name "Downloaded Maps Manager"),
  `%SystemRoot%\System32\moshost.dll`, running as NetworkService. Microsoft's description says it "is
  started on-demand by application accessing downloaded maps" and that "Disabling this service will
  prevent apps from accessing maps".
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Automatic (Delayed) (Stock Default)" writes `automatic_delayed`, that is `Start=2` with
  `DelayedAutostart=1`.
- Documented stock start type: **Automatic (Delayed Start)**, which is exactly what the YAML writes.
  Microsoft's Server guidance lists it as Desktop Experience only, Automatic, and rates it
  **"OK to disable"**.

**Corrections needed:** `none`. `requires_reboot: true` is present, and this is one of only two
tweaks in the file that correctly uses `automatic_delayed`. See merge candidate 1: it belongs with
`task_maps_update`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the offline-maps download service on a PC that never uses offline maps.**

      ## What it does
      Sets the Downloaded Maps Manager service (`MapsBroker`) to Disabled. It downloads offline maps
      and serves them to any application that uses the Windows Maps platform. Microsoft's own
      description warns that disabling it prevents apps from accessing maps.

      ## Benefits
      - **Removes a delayed-autostart service**: it loads shortly after every boot, so this genuinely cuts post-boot work
      - **Microsoft rates it safe**: the Server guidance marks this service "OK to disable"
      - **Feature is winding down**: Microsoft deprecated the Maps app and the Windows Maps platform APIs in 2025

      ## Drawbacks
      - **Offline maps stop**: downloading and updating offline map data no longer works
      - **Map-using apps fail**: applications built on the Windows Maps platform may error rather than degrade gracefully
      - **Not a big saving**: the service is idle once loaded, so expect a small change, not a noticeable one

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021; Maps is no longer preinstalled from 24H2
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Automatic (Delayed Start)
      - **Pairs with**: the offline Maps scheduled task tweak, which covers the background download side

      ## Recommendation
      Apply it if you do not use offline maps or the Maps app, which on 24H2 is most people, since
      Maps is no longer preinstalled. Leave it enabled if you keep downloaded map regions for offline
      navigation.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, MapsBroker rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Deprecated features in the Windows client, Maps app and Maps platform APIs](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features)
      - [Windows 11 default services configuration, MapsBroker as Auto (Delayed)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`MapsBroker`: Automatic, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Deprecated features in the Windows client (Maps app deprecated May 2025; Windows UWP Map control and Maps platform APIs deprecated 8 April 2025), https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
3. Resources for deprecated features ("Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release"), https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources (tier A)
4. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (`MapsBroker` = Auto (Delayed)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)

### `disable_geolocation` Geolocation Service (lfsvc)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `lfsvc` (display name "Geolocation Service"), which monitors the current
  location of the system and manages geofences, feeding the OS location provider that applications
  and Windows features query.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)**, with **4** trigger registrations on 26100.
  Microsoft's Server guidance lists it as Desktop Experience only, Manual, and rates it
  **"OK to disable"**.

**Corrections needed:** None to the mechanism, the options, the stock start type or `requires_reboot`,
which is already true. One copy correction, per correction 13: the info must say the service is
trigger-started, since "Manual" implies dormancy and `lfsvc` starts itself whenever an application or
Windows feature requests location. The existing warning list (apps, Find My Device, automatic time
zone, location-based weather) is accurate and must survive intact.

**Ready-to-paste info block:**

````yaml
    info: |
      **Removes system-wide location resolution, so nothing on the PC can work out where you are.**

      ## What it does
      Sets the Geolocation service (`lfsvc`) to Disabled. It monitors the device's location and
      manages geofences, and it is the provider every application and Windows feature queries for
      location. Disabling it removes the provider rather than denying access to it, which is stronger
      than the Location privacy toggle.

      ## Benefits
      - **Stronger than the toggle**: the provider is gone, so there is nothing for an app to be granted
      - **Microsoft rates it safe**: the Server guidance marks this service "OK to disable"
      - **Removes a live trigger**: with four trigger registrations, this service currently starts itself on demand

      ## Drawbacks
      - **All app location breaks**: every application that asks Windows where you are gets nothing
      - **Find My Device breaks**: Microsoft documents that feature as depending on device location, so a lost machine cannot be located
      - **Automatic time zone breaks**: if you travel, the clock stops following you; the Auto Time Zone Updater depends on this service
      - **Weather and maps degrade**: location-based weather and any map that centres on you stop working

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - The service is trigger-started, so on a machine that uses location it is genuinely running today

      ## Recommendation
      Apply it on a stationary desktop where privacy matters and nothing needs to know where the
      machine is. Leave it enabled on a laptop you travel with, where automatic time zone and Find My
      Device are worth more than the privacy gain.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, lfsvc rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Manage connections from Windows components to Microsoft services, Find My Device](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Windows 11 default services configuration, lfsvc as Manual (Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`lfsvc`: Manual, OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Manage connections from Windows operating system components to Microsoft services, Find My Device section, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (`lfsvc` = Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Trigger registration count for `lfsvc` (4) read from `HKLM\SYSTEM\CurrentControlSet\Services\lfsvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### `disable_xbox_services` Xbox Live Services (XblAuthManager et al.)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short names, four effects:
  - `XblAuthManager` (Xbox Live Auth Manager), authentication and authorization for Xbox Live.
  - `XblGameSave` (Xbox Live Game Save), synchronisation of Xbox Live save data.
  - `XboxNetApiSvc` (Xbox Live Networking Service), the Xbox Live networking API.
  - `XboxGipSvc` (Xbox Accessory Management Service), `%WinDir%\System32\XboxGipSvc.dll`, LocalSystem
    in the `netsvcs` group, which manages connected Xbox accessories.
- Option "Disabled" writes start type Disabled (`Start=4`) on all four.
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`) on all four.
- Documented stock start type: **Manual** for all four. `XblGameSave` and `XboxGipSvc` are
  additionally trigger-started; `XblAuthManager` and `XboxNetApiSvc` are plain Manual. Microsoft's
  Server guidance covers `XblAuthManager` and `XblGameSave` (Desktop Experience only, Manual, rated
  **"Should be disabled"**). CIS Microsoft Windows 11 Enterprise Benchmark item 5.42 recommends
  `XboxGipSvc` be set to Disabled and records its default as "Windows 10 R1709 and newer: Manual".

**Corrections needed:** One, and it is structural. `XboxGipSvc` must become independently selectable,
either as its own tweak or as a third option that disables only the three Xbox Live services. It is
the only member of the group tied to physical hardware the user may own, and disabling it can break
Xbox controller behaviour in any launcher, Steam and Epic included. The tweak's own copy already
tells the user to keep that service while disabling the rest, and the current all-or-nothing shape
makes that impossible to follow. Do not merge this tweak with anything else; split it. On the
evidence tier, note honestly that raw gamepad input is delivered by the kernel-mode GIP and XUSB
drivers rather than by this service, and that CIS records the documented impact narrowly as
"Connected Xbox accessories may not function"; the broader controller breakage is reported at
community tier. Because this is a safety warning, it stays in Drawbacks at full strength.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off Xbox sign-in, cloud saves and networking on a PC that never uses Xbox or Game Pass.**

      ## What it does
      Sets four services to Disabled: `XblAuthManager` (Xbox Live sign-in), `XblGameSave` (cloud save
      sync), `XboxNetApiSvc` (Xbox Live networking) and `XboxGipSvc` (Xbox Accessory Management, which
      handles connected Xbox controllers and accessories).

      ## Benefits
      - **Microsoft recommends two of them**: the Server guidance rates `XblAuthManager` and `XblGameSave` "Should be disabled"
      - **CIS recommends a third**: the Windows 11 Enterprise Benchmark sets `XboxGipSvc` to Disabled at Level 1
      - **Dead weight without Xbox**: on a PC that never touches the Xbox app or Game Pass, none of the four does anything useful

      ## Drawbacks
      - **Xbox controllers can break system-wide**: `XboxGipSvc` handles Xbox accessories, and disabling it can stop controllers working in any launcher, Steam and Epic included
      - **Xbox app and Game Pass break**: sign-in, the store front end and PC Game Pass all stop working
      - **Cloud saves stop**: Xbox Live save synchronisation no longer runs, so progress stays local
      - **Game Bar loses features**: sign-in and capture features that depend on Xbox Live stop working

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start types from the snapshot; the stock value is Manual for all four
      - If you use an Xbox controller, do not apply this until the accessory service can be kept separately

      ## Recommendation
      Apply it on a PC that never uses Xbox, Game Pass or an Xbox controller. Do not apply it if you
      game with an Xbox controller in any launcher, because the accessory service is bundled in and
      cannot currently be kept on its own.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, XblAuthManager and XblGameSave rated Should be disabled](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [CIS Windows 11 Enterprise Benchmark 5.42, Xbox Accessory Management Service](https://www.tenable.com/audits/items/CIS_MS_Windows_11_Enterprise_Level_1_v1.0.0.audit:00bba2f087d462b5662f1987510165b6)
      - [Xbox Accessory Management Service per-release table](https://batcmd.com/windows/11/services/xboxgipsvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`XblAuthManager`, `XblGameSave`: Manual, Should be disabled), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. CIS Microsoft Windows 11 Enterprise Benchmark, item 5.42 "Ensure 'Xbox Accessory Management Service (XboxGipSvc)' is set to 'Disabled'" (Impact: "Connected Xbox accessories may not function"; Default Value: Windows 10 R1709 and newer: Manual), https://www.tenable.com/audits/items/CIS_MS_Windows_11_Enterprise_Level_1_v1.0.0.audit:00bba2f087d462b5662f1987510165b6 (tier B)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (all four present, defaults as stated), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Xbox Accessory Management Service, Windows 11 per-release table, https://batcmd.com/windows/11/services/xboxgipsvc/ (tier C)

### `disable_cdpsvc` Connected Devices Platform Service (CDPSvc)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `CDPSvc` (display name "Connected Devices Platform Service"),
  `%SystemRoot%\System32\CDPSvc.dll`, running as LocalService. It is the cross-device fabric behind
  Nearby Sharing, Phone Link, cross-device clipboard and handoff.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Automatic (Delayed) (Stock Default)" writes `automatic_delayed`, that is `Start=2` with
  `DelayedAutostart=1`.
- Documented stock start type: **Automatic (Delayed Start, Trigger Start)**, which is what the YAML
  writes. Microsoft's Server guidance lists it as Desktop Experience only, Automatic, "No guidance".

**Corrections needed:** `none`. `requires_reboot: true` is present and the `automatic_delayed` stock
option is correct. One thing worth surfacing in the copy: the companion per-user service `CDPUserSvc_*`
is Automatic and is not touched by this tweak, so disabling only the system service leaves the
per-user service running and failing to reach its backend. That is not a YAML defect but it explains
what the user will see.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the background sync that keeps this PC in step with your other Microsoft-linked devices.**

      ## What it does
      Sets the Connected Devices Platform service (`CDPSvc`) to Disabled. It is the cross-device
      fabric behind Nearby Sharing, Phone Link, the shared clipboard and the "resume on other
      devices" handoff, and it maintains background connections to keep those in sync.

      ## Benefits
      - **Cuts background connections**: this is a genuinely network-active service, not an idle one
      - **Removes a delayed-autostart load**: it starts shortly after every boot
      - **No value alone**: a single-device user gets nothing from cross-device sync

      ## Drawbacks
      - **Phone Link breaks**: your phone no longer connects to this PC
      - **Nearby Sharing breaks**: sending files to a nearby Windows device stops working
      - **Shared clipboard breaks**: copy on one device and paste on another no longer works, and handoff between devices stops
      - **Per-user half stays up**: the companion `CDPUserSvc` service keeps running and simply fails to reach its backend

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Automatic (Delayed Start)
      - Microsoft gives no guidance either way for this service, so the case rests on absence of dependency

      ## Recommendation
      Apply it on a single-device machine that does not use Phone Link, Nearby Sharing or the shared
      clipboard. Leave it enabled if you move between a phone and this PC, since those features have
      no fallback.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, CDPSvc as Auto (Delayed, Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Connected Devices Platform Service defaults](https://revertservice.com/11/cdpsvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 Default Services Configuration (`CDPSvc` = Auto (Delayed, Triggered); `CDPUserSvc_*` = Auto), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
3. Windows 11 Default Services Configuration (`CDPSvc` = Auto (Delayed, Triggered)), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Connected Devices Platform Service (CDPSvc) defaults, https://revertservice.com/11/cdpsvc/ (tier C)

### `disable_biometrics` Windows Biometric Service (WbioSrvc)

**Verdict:** VERIFIED

**Mechanism:**
- Service short name: `WbioSrvc` (display name "Windows Biometric Service"), which gives client
  applications the ability to capture, compare, manipulate and store biometric data without direct
  access to biometric hardware or samples. It is what Windows Hello fingerprint and facial
  recognition sign-in depend on.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)**, with **2** trigger registrations on 26100.
  Microsoft's Server guidance lists it as Desktop Experience only, Manual, "No guidance".

**Corrections needed:** None to the mechanism, the options, the stock start type or `requires_reboot`,
which is already true. One copy correction, per correction 13: say the service is trigger-started, so
a user with a fingerprint reader understands they are removing live behaviour rather than declining a
dormant one.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off Windows Hello fingerprint and face sign-in on a PC that logs in with a PIN or password.**

      ## What it does
      Sets the Windows Biometric service (`WbioSrvc`) to Disabled. It manages fingerprint readers and
      infrared cameras and brokers biometric samples to applications, and it is the service Windows
      Hello fingerprint and face sign-in run on top of.

      ## Benefits
      - **Hardware you may not have**: on a desktop with no reader or IR camera it manages nothing
      - **Removes a credential path**: one fewer way to authenticate to the machine, which is a hardening choice
      - **Fully reversible**: Windows falls back to PIN or password rather than locking you out

      ## Drawbacks
      - **Fingerprint sign-in breaks**: the reader stops being an option at the sign-in screen
      - **Face sign-in breaks**: Windows Hello facial recognition stops working
      - **Hello for Business paths**: a biometric-gated credential provider stops working, so verify a PIN or password works before applying
      - **Not dormant today**: the service is trigger-started, so on a machine with biometric hardware it starts itself

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - Microsoft gives no guidance either way for this service

      ## Recommendation
      Apply it if you sign in only with a password or PIN and have no fingerprint reader or IR camera.
      Leave it enabled if you use any biometric unlock, since the convenience loss is immediate and
      the gain is negligible.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, WbioSrvc as Manual (Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Windows Biometric Service per-release table](https://batcmd.com/windows/11/services/wbiosrvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (`WbioSrvc` = Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Windows Biometric Service per-release table, https://batcmd.com/windows/11/services/wbiosrvc/ (tier C)
4. Trigger registration count for `WbioSrvc` (2) read from `HKLM\SYSTEM\CurrentControlSet\Services\WbioSrvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### `disable_smartcard` Smart Card Services (SCardSvr)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short names, three effects:
  - `SCardSvr` (Smart Cards for Windows service, formerly the Smart Card Resource Manager).
    Microsoft documents it as providing "the basic infrastructure for all other smart card components
    as it manages smart card readers and application interactions on the computer".
  - `ScDeviceEnum` (Smart Card Device Enumeration Service), which creates software device nodes for
    all smart card readers accessible to a given session.
  - `SCPolicySvc` (Smart Card Removal Policy), which enforces the removal policy, for example locking
    the desktop when a card is withdrawn.
- Option "Disabled" writes start type Disabled (`Start=4`) on all three.
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`) on all three.
- Documented stock start type on client Windows: `SCardSvr` **Manual (Trigger Start)**, confirmed at
  tier A because Microsoft Learn reproduces the service manifest carrying `start="demand"`;
  `ScDeviceEnum` **Manual (Trigger Start)**; `SCPolicySvc` **Manual**, not trigger-started. On 26100
  `SCardSvr` has **4** trigger registrations and `ScDeviceEnum` has **3**. Note a genuine platform
  difference: Microsoft's Windows *Server* guidance lists `SCardSvr` as already Disabled on Server.
  That is a server-specific default; the YAML's client-side Manual is correct.

**Corrections needed:** Three. (1) **`risk_level` must be `high`, not `medium`.** The tweak's own
`warning` says it "removes the only sign-in path and can lock the user out", and the mechanism
supports that: `SCardSvr` is the resource manager every other smart card component depends on, so
with it disabled the smart card credential provider has no path to the card, and on a machine whose
only sign-in path is a smart card there is no interactive sign-in left and no way to revert from
inside Windows. This is the only tweak in the corpus whose declared risk contradicts its own warning.
(2) `requires_reboot: true` is missing, per correction 4, and it matters more here than anywhere
else: `SCardSvr` and `ScDeviceEnum` are trigger-started, so on a machine with a reader attached they
are typically already running at apply time, the engine never stops them, and without the flag the
app reports smart card support as disabled while the card still works. The user discovers the real
state at the next boot, which is the worst possible moment. (3) The copy should note that
`SCPolicySvc` is what locks the desktop on card removal, so on a card-using machine disabling it is a
security regression rather than a hardening step.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off smart card support on a consumer PC that has no card reader and no card-based sign-in.**

      ## What it does
      Sets three services to Disabled: `SCardSvr` (the resource manager every other smart card
      component depends on), `ScDeviceEnum` (which enumerates readers into a session) and
      `SCPolicySvc` (which enforces the lock-on-card-removal policy). Together they are smart card
      support.

      ## Benefits
      - **Hardware you do not have**: on a home PC with no reader, three services manage nothing
      - **Removes a credential path**: no card-based authentication surface remains
      - **Removes live triggers**: `SCardSvr` and `ScDeviceEnum` are trigger-started, so they do start themselves today

      ## Drawbacks
      - **Can lock you out**: on a machine that signs in with a smart card, this removes the only sign-in path and cannot be reverted from inside Windows
      - **CAC, PIV and YubiKey PIV break**: card-based logon of every kind stops working
      - **Enterprise VPN and certificate auth break**: any VPN or certificate authentication that reads a card stops
      - **Removal policy stops**: with `SCPolicySvc` disabled the desktop no longer locks when a card is withdrawn, which is a security regression on a card-using machine

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service; a card can keep working until then
      - **Reverting**: restores the previous start types from the snapshot; the stock value is Manual for all three
      - Confirm a working password or PIN sign-in before applying, and never apply it remotely

      ## Recommendation
      Apply it only on a personal PC with no card reader and no card-based sign-in. Never apply it on
      a work, managed or enterprise machine, or anywhere smart card, CAC, PIV or YubiKey PIV sign-in
      is in use, because the lockout is not recoverable from inside Windows.

      ## Evidence
      - **Risk**: high
      - **Confidence**: Microsoft-documented
      - [Smart Cards for Windows Service, with the shipped service manifest](https://learn.microsoft.com/en-us/windows/security/identity-protection/smart-cards/smart-card-smart-cards-for-windows-service)
      - [Guidelines for disabling system services in Windows Server](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, clean install](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
````

**Sources:**
1. Smart Cards for Windows Service (service manifest with `start="demand"`, and the description of its role as the base infrastructure for all smart card components), https://learn.microsoft.com/en-us/windows/security/identity-protection/smart-cards/smart-card-smart-cards-for-windows-service (tier A)
2. Guidelines for disabling system services in Windows Server with Desktop Experience (`SCardSvr` already Disabled on Server, `ScDeviceEnum` Manual / OK to disable, `SCPolicySvc` Manual / No guidance), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
3. Windows 10 Default Services Configuration (`SCardSvr` and `ScDeviceEnum` = Manual (Triggered), `SCPolicySvc` = Manual), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
4. Windows 11 Default Services Configuration (same three defaults), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
5. Trigger registration counts for `SCardSvr` (4) and `ScDeviceEnum` (3) read from `HKLM\SYSTEM\CurrentControlSet\Services\<name>\TriggerInfo` on build 26100 (tier A for trigger existence only)

### `disable_sensor_services` Sensor Services (SensorService)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short names, three effects:
  - `SensorService` (Sensor Service), which manages sensors on the device and the sensor
    functionality Windows exposes to applications.
  - `SensrSvc` (Sensor Monitoring Service), `%WinDir%\system32\sensrsvc.dll`, running as LocalService
    in the `LocalServiceAndNoImpersonation` group. Its documented example is adapting screen
    brightness to lighting conditions.
  - `SensorDataService` (Sensor Data Service), `SensorDataService.exe`, LocalSystem, which delivers
    data from a variety of sensors.
- Option "Disabled" writes start type Disabled (`Start=4`) on all three.
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`) on all three.
- Documented stock start type: **Manual (Trigger Start)** for all three, on both the primary and the
  secondary platform. `SensorService` carries **8** trigger registrations on 26100, the most of any
  service in this file. Microsoft's Server guidance covers all three as Desktop Experience only,
  Manual, and rates **all three "OK to disable"**. A prior suspicion that `SensrSvc` had been removed
  is not supported: it is present on 21H2 through 24H2 with no "not exists" entry in any table.

**Corrections needed:** Two. (1) `requires_reboot: true` is missing, per correction 4. (2) The info
must say `SensorService` is trigger-started with eight registrations, per correction 13. No
correction to the service list or the defaults. Worth adding to the copy: determining whether a
machine truly has no sensors is not obvious to a typical user, so the text should say how to check
rather than leaving it to guesswork.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the sensor stack on a desktop that has no ambient-light or orientation sensors.**

      ## What it does
      Sets three services to Disabled: `SensorService` (which manages sensors and exposes them to
      apps), `SensrSvc` (which monitors sensors and adapts system state, for example screen
      brightness) and `SensorDataService` (which delivers the sensor data itself).

      ## Benefits
      - **Microsoft rates all three safe**: the Server guidance marks every one of them "OK to disable"
      - **Hardware you may not have**: a tower desktop typically has no light, orientation or motion sensors
      - **Removes many triggers**: `SensorService` alone carries eight trigger registrations, so it does start itself today

      ## Drawbacks
      - **Auto-brightness breaks**: adaptive screen brightness stops responding to ambient light
      - **Screen rotation breaks**: automatic rotation on a tablet or convertible stops working
      - **Sensor-aware apps break**: anything reading orientation, light or motion gets nothing
      - **Hard to be sure**: an attached monitor or a laptop lid sensor is easy to overlook, so check Device Manager under Sensors before applying

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start types from the snapshot; the stock value is Manual for all three
      - On a genuinely sensorless machine these never start, so expect a guarantee rather than a speed gain

      ## Recommendation
      Apply it on a desktop you have confirmed has no sensors listed in Device Manager. Leave it
      enabled on any laptop, tablet or convertible, where auto-brightness and rotation are worth more
      than removing three idle services.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Guidelines for disabling system services in Windows Server, all three rated OK to disable](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
      - [Windows 11 default services configuration, all three Manual (Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Sensor Monitoring Service per-release table](https://batcmd.com/windows/11/services/sensrsvc/)
````

**Sources:**
1. Guidelines for disabling system services in Windows Server with Desktop Experience (`SensorService`, `SensrSvc`, `SensorDataService`: all Manual, all OK to disable), https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (all three = Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Sensor Monitoring Service, Windows 11 per-release table (Manual on 21H2, 22H2, 23H2 and 24H2, all editions, no "not exists" entry), https://batcmd.com/windows/11/services/sensrsvc/ (tier C)
4. Trigger registration count for `SensorService` (8) read from `HKLM\SYSTEM\CurrentControlSet\Services\SensorService\TriggerInfo` on build 26100 (tier A for trigger existence only)

### `disable_parental_controls` Family Safety Monitor (WpcMonSvc)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short name: `WpcMonSvc`, display name "Parental Controls". It enforces the Microsoft Family
  Safety restrictions applied to an account: screen time limits, app and content filtering, and
  activity reporting.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual**, not trigger-started. Not covered by Microsoft's Windows
  Server service guidance, which is expected for a consumer family feature.

**Corrections needed:** Two. (1) `requires_reboot: true` is missing, per correction 4. (2) Per
correction 16, the copy must state the supervision-bypass consequence plainly. This is the one tweak
in this file whose misuse harms a third party rather than the operator: applying it on a device with
a supervised child account silently stops enforcement, and the parent sees limits stop working with
no obvious cause. The mechanism, the options and the stock start type are correct.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the Family Safety monitor on an account that is not supervised by Microsoft Family.**

      ## What it does
      Sets the Family Safety Monitor service (`WpcMonSvc`, shown as Parental Controls) to Disabled. It
      is what enforces Microsoft Family Safety restrictions on this device: screen time limits, app
      and content filtering, and activity reporting.

      ## Benefits
      - **No account to supervise**: on an adult or standalone account it enforces nothing
      - **Not trigger-started**: unlike most services here it is plain Manual, so this is a clean removal
      - **Fully reversible**: nothing else depends on it

      ## Drawbacks
      - **Supervision stops silently**: on a device with a supervised child account this defeats parental controls, and the parent gets no notification that enforcement stopped
      - **Screen time limits break**: time restrictions are no longer applied on this device
      - **Content filtering breaks**: app and web content limits stop being enforced
      - **Activity reporting breaks**: the parent's activity report for this device goes quiet

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - The consequence of misuse here falls on someone other than the person applying it

      ## Recommendation
      Apply it on an adult or standalone account that has never used Microsoft Family Safety. Never
      apply it on a device with a supervised child account, where it is a supervision bypass rather
      than a tidy-up.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Windows 11 default services configuration, WpcMonSvc as Manual](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Windows 10 default services configuration, WpcMonSvc as Manual](https://www.winhelponline.com/blog/windows-10-default-services-configuration/)
      - [Parental Controls per-release table](https://batcmd.com/windows/11/services/wpcmonsvc/)
````

**Sources:**
1. Windows 11 Default Services Configuration (`WpcMonSvc` = Manual), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
2. Windows 10 Default Services Configuration (`WpcMonSvc` = Manual), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
3. Parental Controls per-release tables, https://batcmd.com/windows/11/services/wpcmonsvc/ (tier C)

### `disable_payments_nfc` Payments and NFC/SE Manager (SEMgrSvc)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Service short name: `SEMgrSvc` (display name "Payments and NFC/SE Manager"), which manages the NFC
  secure element used for tap-to-pay and secure wallet operations on devices with NFC hardware.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual (Trigger Start)**, with **2** trigger registrations on 26100.
  Not covered by Microsoft's Windows Server service guidance.

**Corrections needed:** Two. (1) `requires_reboot: true` is missing, per correction 4. (2) The info
must say the service is trigger-started, per correction 13. See merge candidate 2: this belongs with
`disable_wallet_service`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the NFC secure-element manager on a device with no tap-to-pay hardware.**

      ## What it does
      Sets the Payments and NFC/SE Manager service (`SEMgrSvc`) to Disabled. It manages the hardware
      secure element that NFC tap-to-pay and secure wallet operations run through. It is the hardware
      half of the Windows payment stack; the Wallet service is the front end.

      ## Benefits
      - **Hardware you may not have**: a desktop has no NFC radio and no secure element
      - **Removes a payment surface**: nothing can reach the secure element after this
      - **Removes a live trigger**: the service is trigger-started, so it does start itself on demand today

      ## Drawbacks
      - **Tap-to-pay breaks**: NFC payments from this device stop working
      - **Secure wallet breaks**: wallet features that use the secure element stop working
      - **Half the stack**: the Wallet service is separate, so disabling only this one leaves the front end running
      - **Nothing visible changes**: on a machine without NFC there is no observable difference

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual
      - **Pairs with**: the Wallet service tweak, which covers the other half

      ## Recommendation
      Apply it on any desktop, and on any laptop without NFC. Leave it enabled if you tap to pay from
      this device or use a secure-element wallet.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Windows 11 default services configuration, SEMgrSvc as Manual (Triggered)](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Windows 10 default services configuration, SEMgrSvc as Manual (Triggered)](https://www.winhelponline.com/blog/windows-10-default-services-configuration/)
      - [Payments and NFC/SE Manager per-release table](https://batcmd.com/windows/11/services/semgrsvc/)
````

**Sources:**
1. Windows 11 Default Services Configuration (`SEMgrSvc` = Manual (Triggered)), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
2. Windows 10 Default Services Configuration (`SEMgrSvc` = Manual (Triggered)), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
3. Payments and NFC/SE Manager per-release table, https://batcmd.com/windows/11/services/semgrsvc/ (tier C)
4. Trigger registration count for `SEMgrSvc` (2) read from `HKLM\SYSTEM\CurrentControlSet\Services\SEMgrSvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### `disable_wmp_network_sharing` Windows Media Player Network Sharing (WMPNetworkSvc)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism, corrected:**
- Service short name: `WMPNetworkSvc` (display name "Windows Media Player Network Sharing Service"),
  `%ProgramFiles%\Windows Media Player\wmpnetwk.exe`, running as NetworkService in its own process. It
  shares Windows Media Player libraries to other networked players and media devices over UPnP.
- Option "Disabled" writes start type Disabled (`Start=4`).
- Option "Manual (Stock Default)" writes start type Manual (`Start=3`).
- Documented stock start type: **Manual**, with no trigger registration, **where the service exists**.
- Corrected applicability: the service is **not part of the base OS**. It is delivered by the
  removable `Media.WindowsMediaPlayer` optional feature (Windows Media Player Legacy). On
  Windows 11 24H2 IoT Enterprise LTSC 2024 there is no key under
  `HKLM\SYSTEM\CurrentControlSet\Services\WMPNetworkSvc`, `wmpnetwk.exe` is not on disk, and
  `Get-WindowsCapability -Online` reports `Media.WindowsMediaPlayer~~~~0.0.12.0 = NotPresent`. The
  effect must therefore carry `optional: true` with an `if_missing`.

**Mechanism as authored (wrong):** the same service and options with **no `optional` flag and no
`requires_reboot`**, plus a `warning` asserting that the service "ships present with a Manual/trigger
start type on both Windows 10 and 11". That statement is false as an unconditional claim, and because
the effect is required rather than optional, the engine reads `Value::Missing`, classifies the tweak
`TweakState::Unknown` with `UnknownCause::MissingRequired`, refuses apply with
`EngineError::SurfaceUnreadable`, and aborts snapshot capture with `CaptureMissingRequired`. On the
primary IoT Enterprise LTSC target the tweak is unusable, not merely inapplicable.

**Corrections needed:** Three. (1) Add `optional: true` with an `if_missing` value, per correction 3.
(2) Drop the unconditional "ships present on both Windows 10 and 11" claim from the `warning` and
scope it to images where Windows Media Player Legacy is installed. (3) Add `requires_reboot: true`,
per correction 4. Also drop the trigger-start hedge: where the service exists it is plain Manual with
no trigger registration. See merge candidate 3: this belongs with `disable_ssdp_upnp`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows Media Player serving your media library to other devices on the network.**

      ## What it does
      Sets the Windows Media Player Network Sharing service (`WMPNetworkSvc`) to Disabled. The service
      publishes your Windows Media Player library over DLNA and UPnP so smart TVs and other renderers
      can play from it. Local playback in the player is unaffected.

      ## Benefits
      - **Closes a network listener**: the service opens a listener that serves media to the LAN
      - **Nothing else depends on it**: no other Windows feature needs library sharing
      - **Legacy component**: Microsoft deprecated the legacy DRM services this player relies on in December 2024

      ## Drawbacks
      - **DLNA serving stops**: streaming your library to a TV or other renderer no longer works
      - **Discovery is separate**: DLNA also needs the SSDP and UPnP services, so this tweak alone leaves the discovery transport up
      - **May not be installed**: the service ships with the removable Windows Media Player Legacy feature, so it is absent on Enterprise LTSC and IoT Enterprise LTSC images and on any machine where that feature was removed

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021, but only where Windows Media Player Legacy is installed
      - **Takes effect**: after reboot, because changing a start type never stops an already running service
      - **Reverting**: restores the previous start type from the snapshot; the stock value is Manual where the service exists
      - **Pairs with**: the SSDP and UPnP tweak, which removes the transport DLNA rides on

      ## Recommendation
      Apply it if you do not stream your media library from this PC to other devices. Leave it enabled
      if you use DLNA sharing from Windows Media Player to a TV or a network player.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Deprecated features in the Windows client, legacy DRM services used by Windows Media Player](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features)
      - [Windows 11 default services configuration, WMPNetworkSvc as Manual](https://www.winhelponline.com/blog/windows-11-default-services-configuration/)
      - [Windows Media Player Network Sharing Service per-release table](https://batcmd.com/windows/11/services/wmpnetworksvc/)
````

**Sources:**
1. Direct inspection of Windows 11 24H2 build 26100 (IoT Enterprise LTSC 2024): no `HKLM\SYSTEM\CurrentControlSet\Services\WMPNetworkSvc` key, no `wmpnetwk.exe` on disk, `Get-WindowsCapability -Online` reports `Media.WindowsMediaPlayer~~~~0.0.12.0 = NotPresent` (tier A for absence, which is a shipped-image fact rather than a configuration default)
2. Deprecated features in the Windows client (legacy DRM services used by Windows Media Player deprecated December 2024), https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps (`WMPNetworkSvc` = Manual), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C, describing images with the Windows Media Player feature installed)
4. Windows Media Player Network Sharing Service, Windows 11 per-release table, https://batcmd.com/windows/11/services/wmpnetworksvc/ (tier C, same caveat)
5. `src-tauri/src/tweaks/kinds/service.rs` and the engine's `UnknownCause::MissingRequired` / `EngineError::SurfaceUnreadable` path (tier A, this repository)

### `task_autochk_proxy` Autochk Proxy task

**Verdict:** VERIFIED

**Mechanism:**
- Full task path: `\Microsoft\Windows\Autochk\Proxy`.
- Option "Disabled" sets the task Disabled.
- Option "Enabled (Stock Default)" sets the task Enabled.
- Action: `rundll32.exe acproxy.dll,PerformAutochkOperations`, fired from a boot trigger. Microsoft's
  own shipped description string for the task, resource `-102` in `%SystemRoot%\System32\acproxy.dll`,
  reads: "This task collects and uploads autochk SQM data if opted-in to the Microsoft Customer
  Experience Improvement Program." That is the telemetry upload of disk-check data, not the disk
  check itself, so the YAML's claim that boot-time `autochk` and `chkdsk` are unaffected is correct.

**Corrections needed:** One, and it is honesty rather than mechanism. The upload is gated on CEIP
consent by Microsoft's own description, so on a machine that never opted into CEIP the task uploads
nothing and disabling it produces no observable change. Say so in Drawbacks. Cross-reference, and not
a correction to this file: `privacy:disable_ceip_tasks` is named for CEIP but does not cover this
task, even though Microsoft's own string makes it a CEIP SQM uploader; that correction belongs to
`privacy.md`. The shipped enabled-state of this task is an open question (open question 3), so the
"Enabled (Stock Default)" literal needs a clean-image check before it is fully trusted, in common
with every other task tweak in this file.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the scheduled upload of disk-check telemetry to Microsoft, without touching disk checking itself.**

      ## What it does
      Disables the Autochk Proxy scheduled task (`\Microsoft\Windows\Autochk\Proxy`). Microsoft's own
      description for it reads "This task collects and uploads autochk SQM data if opted-in to the
      Microsoft Customer Experience Improvement Program". It runs
      `rundll32.exe acproxy.dll,PerformAutochkOperations` from a boot trigger.

      ## Benefits
      - **One less upload**: removes a boot-time diagnostic transmission to Microsoft
      - **Disk checking unaffected**: `autochk` and `chkdsk` are a different mechanism and keep working
      - **No functional cost**: nothing on the machine reads the data this task sends

      ## Drawbacks
      - **Nothing changes without CEIP**: Microsoft's description gates the upload on CEIP opt-in, so on a machine that never opted in you will see no difference
      - **Updates can re-enable it**: tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates
      - **Partial coverage**: this is one of several CEIP and census tasks, so disabling it alone does not stop the rest

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: immediately, since a scheduled task changes state as soon as it is disabled
      - **Reverting**: restores the previous enabled state from the snapshot
      - **Pairs with**: the Customer Experience Improvement Program tweak in Privacy, which sets the CEIP policy this task honours

      ## Recommendation
      Apply it if you want fewer diagnostic uploads leaving the machine; there is no functional
      downside. Skip it if you deliberately participate in CEIP for enterprise reporting.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [CEIPEnable, the Customer Experience Improvement Program control](https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable)
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
````

**Sources:**
1. Shipped task description resource `%SystemRoot%\System32\acproxy.dll,-102`: "This task collects and uploads autochk SQM data if opted-in to the Microsoft Customer Experience Improvement Program" (tier A, shipped binary resource; evidence of what the task is, not of its default enabled state)
2. CEIPEnable, https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable (tier A)
3. Windows-10-Hardening, `components/scheduled_tasks.bat` (CEIP task group, identical path `\Microsoft\Windows\Autochk\Proxy`), https://github.com/aghorler/Windows-10-Hardening/blob/master/components/scheduled_tasks.bat (tier D, corroborating the path form only)
4. windows10fixup `Fixup.ps1` (independent tooling using the identical path), https://github.com/iDigitalFlame/windows10fixup (tier D, corroborating the path form only)

### `task_feedback_dmclient` Feedback (SIUF) DmClient tasks

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Full task paths, two effects:
  - `\Microsoft\Windows\Feedback\Siuf\DmClient`
  - `\Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload`
- Option "Disabled" sets both tasks Disabled.
- Option "Enabled (Stock Default)" sets both tasks Enabled.
- SIUF is Microsoft's internal name for the feedback subsystem, and the binary these tasks drive,
  `dmclient.exe`, is identified as the "Microsoft Feedback SIUF Deployment Manager Client".
  `DmClientOnScenarioDownload` carries the description "Update SIUF...", that is it refreshes
  feedback scenario configuration, and it holds a real `WnfStateChangeTrigger`. `DmClient` has an
  **empty `<Triggers />` set** on 26100.

**Corrections needed:** One. The copy overstates the first effect. With an empty trigger set,
disabling `DmClient` blocks an on-demand or externally-invoked run but stops nothing that was
scheduled, so "stops the scheduled feedback and diagnostic transmission" describes a schedule this
build does not have. Only `DmClientOnScenarioDownload` suppresses an event-driven run. The paths, the
option shape and the immediate-effect assumption are all correct. The shipped enabled-state of both
tasks remains an open question (open question 3): the absence of an `<Enabled>` element in their
definitions is suggestive that they were never rewritten but is not a clean-image confirmation.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the Windows Feedback tasks that send feedback and diagnostic data to Microsoft.**

      ## What it does
      Disables the two Windows Feedback (SIUF) scheduled tasks, `DmClient` and
      `DmClientOnScenarioDownload`. They drive `dmclient.exe`, the Microsoft Feedback SIUF Deployment
      Manager Client. The second refreshes the feedback scenario configuration from an event trigger;
      the first has no trigger on current builds and runs only when something invokes it.

      ## Benefits
      - **Removes a feedback channel**: nothing on the machine depends on these uploads
      - **Blocks the scenario refresh**: `DmClientOnScenarioDownload` is the one with a live trigger, and it stops
      - **No functional cost**: neither task provides a user-facing feature

      ## Drawbacks
      - **Feedback Hub degrades**: submissions and the scenario-driven feedback prompts may stop working properly
      - **Less than it sounds**: `DmClient` carries no trigger on Windows 11 24H2, so disabling it stops nothing that was scheduled
      - **Updates can re-enable it**: tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates
      - **Overlaps with the telemetry service**: with `DiagTrack` disabled these tasks have no working upload path anyway

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: immediately, since a scheduled task changes state as soon as it is disabled
      - **Reverting**: restores the previous enabled state from the snapshot
      - **Pairs with**: the DiagTrack telemetry service tweak, which removes the transport these tasks would use

      ## Recommendation
      Apply it if you do not use Feedback Hub, since there is nothing to lose. Leave it alone if you
      actively submit feedback to Microsoft and want the attached diagnostics to arrive.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
      - [dmclient.exe identified as the Microsoft Feedback SIUF Deployment Manager Client](https://strontic.github.io/xcyclopedia/library/dmclient.exe-7E90BC5211DC802DEDFA345E234B0F95.html)
````

**Sources:**
1. Shipped task definitions on Windows 11 24H2 build 26100: `DmClient` has `<Triggers />` (empty), `DmClientOnScenarioDownload` has a `WnfStateChangeTrigger` (tier A for the trigger observation, which no enable or disable operation can produce; the enabled state of these files is not used as evidence)
2. STRONTIC xcyclopedia, `dmclient.exe` identified as "Microsoft Feedback SIUF Deployment Manager Client", https://strontic.github.io/xcyclopedia/library/dmclient.exe-7E90BC5211DC802DEDFA345E234B0F95.html (tier C)
3. Optimize-Offline `ScheduledTasks.json` (task `DmClientOnScenarioDownload`, description "Update SIUF..."), https://github.com/DrEmpiricism/Optimize-Offline/blob/master/Content/Additional/Setup/ScheduledTasks.json (tier D, corroborating the path form only)
4. windows10fixup `Fixup.ps1` (both full paths in the form the YAML uses), https://github.com/iDigitalFlame/windows10fixup (tier D, corroborating the path form only)

### `task_wer_queuereporting` Windows Error Reporting QueueReporting task

**Verdict:** VERIFIED

**Mechanism:**
- Full task path: `\Microsoft\Windows\Windows Error Reporting\QueueReporting`.
- Option "Disabled" sets the task Disabled.
- Option "Enabled (Stock Default)" sets the task Enabled.
- Windows Error Reporting queues crash reports that could not be sent immediately, and this task
  drains that queue by uploading the queued reports to Microsoft. Microsoft documents crash reporting
  and crash dumps as a diagnostic channel "managed by Windows Error Reporting", distinct from the
  Connected User Experiences and Telemetry component, so disabling the task stops the queued upload
  while leaving local error logging in place. That matches the YAML.

**Corrections needed:** `none`. The path, the option shape and the immediate-effect assumption are
correct. The shipped enabled-state is an open question in common with the other task tweaks (open
question 3). Keep this tweak out of any merged telemetry-task group, per merge candidate 4: it is the
one task here that a user may rationally want to keep.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops queued crash reports being uploaded to Microsoft, while local crash logs stay on your PC.**

      ## What it does
      Disables the Windows Error Reporting QueueReporting scheduled task
      (`\Microsoft\Windows\Windows Error Reporting\QueueReporting`). Windows Error Reporting queues
      crash reports that could not be sent at the time of the crash, and this task drains that queue
      by uploading them.

      ## Benefits
      - **Crash data stays local**: reports can contain application and system state, and none of it leaves the machine
      - **Local logging intact**: `.wer` reports and Reliability Monitor keep working
      - **Separate from telemetry**: this is a distinct channel from the diagnostic data pipeline, so it is a real additional gain

      ## Drawbacks
      - **Microsoft stops hearing about your crashes**: that removes one input into the bug you are actually hitting getting fixed
      - **Support cases suffer**: a support case that expects Windows Error Reporting submissions has nothing to reference
      - **Updates can re-enable it**: tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: immediately, since a scheduled task changes state as soon as it is disabled
      - **Reverting**: restores the previous enabled state from the snapshot
      - **Pairs with**: the Windows Error Reporting service tweak in Privacy, which stops reports being generated in the first place

      ## Recommendation
      Apply it if you would rather crash contents stayed on your machine, since local diagnostics are
      unaffected. Leave it enabled if you are chasing a recurring crash and want Microsoft or an OEM
      to receive the reports.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Configure Windows diagnostic data in your organization, crash reporting as a distinct channel](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
      - [Windows Error Reporting overview](https://learn.microsoft.com/en-us/windows/win32/wer/windows-error-reporting)
````

**Sources:**
1. Configure Windows diagnostic data in your organization (crash reporting and crash dumps managed by Windows Error Reporting, described as a channel distinct from the telemetry component), https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
2. Windows Error Reporting, https://learn.microsoft.com/en-us/windows/win32/wer/windows-error-reporting (tier A, for the queue and upload model rather than for the task path)
3. Windows-10-Hardening, `components/scheduled_tasks.bat` (identical full path), https://github.com/aghorler/Windows-10-Hardening/blob/master/components/scheduled_tasks.bat (tier D, corroborating the path form only)
4. windows10fixup `Fixup.ps1` (identical path), https://github.com/iDigitalFlame/windows10fixup (tier D, corroborating the path form only)

### `task_maps_update` Offline Maps update tasks

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
- Full task paths, two effects:
  - `\Microsoft\Windows\Maps\MapsUpdateTask`, which downloads updates to offline maps in the
    background.
  - `\Microsoft\Windows\Maps\MapsToastTask`, which raises the notification about them.
- Option "Disabled" sets both tasks Disabled.
- Option "Enabled (Stock Default)" sets both tasks Enabled.
- Both paths exist verbatim on Windows 11 24H2 build 26100. `MapsToastTask` has an **empty
  `<Triggers />` set** and a `ComHandler` action, so it only ever fires when `MapsBroker` invokes it;
  it has no schedule of its own.
- Microsoft states that "Maps is no longer preinstalled with Windows starting with the Windows 11,
  version 24H2 release", and deprecated the Maps app in May 2025 and the Windows Maps platform APIs
  on 8 April 2025, so on the primary platform both tasks sit behind an app that is not installed.

**Corrections needed:** Two. (1) The "Enabled (Stock Default)" option for the `maps_update` effect
has **no established provenance**. The earlier claim that `MapsUpdateTask` ships
`<Enabled>false</Enabled>` rested on the modified validation machine and is withdrawn; see open
question 2. Under the `_harmful-revert.md` rule a revert value must be established from evidence
about Windows rather than assumed, so this literal is blocked pending a clean-image check. If the
withdrawn finding is confirmed, the stock option must write `{ maps_update: disabled, maps_toast:
enabled }`, and the defect bites hardest on the System Default path, where ADR-0003 makes System
Default a selectable state the app would write incorrectly. (2) The copy overstates the second
effect: `MapsToastTask` has no trigger of its own, so the tweak does not stop a scheduled toast. Add
the 24H2 context that Maps is no longer preinstalled. See merge candidate 1: this belongs with
`disable_maps_broker`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the background download of offline map updates and the notification about them.**

      ## What it does
      Disables the two offline Maps scheduled tasks, `MapsUpdateTask` (which downloads offline map
      updates in the background) and `MapsToastTask` (which raises the notification). The toast task
      has no schedule of its own; it fires only when the Downloaded Maps Manager service invokes it.

      ## Benefits
      - **No background downloads**: map data is large, so this saves real bandwidth on a metered link
      - **No update notifications**: the toast about new map data stops appearing
      - **Pairs with the service**: it is the task half of the same subsystem as Downloaded Maps Manager

      ## Drawbacks
      - **Offline maps stop updating**: downloaded regions go stale until you refresh them by hand
      - **Manual updates still need the service**: you can only trigger an update while the Downloaded Maps Manager service is enabled
      - **Little to stop on 24H2**: Maps is no longer preinstalled from Windows 11 24H2, so on a clean install there may be no app behind these tasks
      - **Updates can re-enable it**: tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: immediately, since a scheduled task changes state as soon as it is disabled
      - **Reverting**: restores the previous enabled state from the snapshot
      - **Pairs with**: the Downloaded Maps Manager service tweak, which covers the service half

      ## Recommendation
      Apply it if you do not use offline maps, which on 24H2 covers most people since Maps is no
      longer preinstalled. Leave it enabled if you keep downloaded map regions current for offline
      navigation.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Resources for deprecated features, Maps no longer preinstalled from Windows 11 24H2](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources)
      - [Deprecated features in the Windows client, Maps app and Maps platform APIs](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features)
````

**Sources:**
1. Shipped task definitions on Windows 11 24H2 build 26100: `MapsToastTask` has an empty `<Triggers />` set and a `ComHandler` action (tier A for the trigger observation only; the enabled state of `MapsUpdateTask` observed on that machine is withdrawn and is not used)
2. Resources for deprecated features ("Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release"), https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources (tier A)
3. Deprecated features in the Windows client (Maps app deprecated May 2025; Windows UWP Map control and Maps platform APIs deprecated 8 April 2025), https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
4. windows-cleaner `Disable-Scheduled-Tasks.ps1` (both full paths in the form the YAML uses), https://github.com/markulie/windows-cleaner/blob/main/Disable-Scheduled-Tasks.ps1 (tier D, corroborating the path form only)
5. windows10fixup `Fixup.ps1` (identical paths), https://github.com/iDigitalFlame/windows10fixup (tier D, corroborating the path form only)

### `task_disk_diagnostic_datacollector` DiskDiagnosticDataCollector task

**Verdict:** INCORRECT

**Mechanism:**
- Full task path:
  `\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector`. The path itself
  is confirmed exactly as the YAML writes it, in a folder alongside the separate
  `Microsoft-Windows-DiskDiagnosticResolver` task, which this tweak correctly leaves alone.
- Option "Disabled" sets the task Disabled.
- Option "Enabled (Stock Default)" sets the task Enabled.
- The task's own description is "The Windows Disk Diagnostic reports general disk and system
  information to Microsoft for users participating in the Customer Experience Program", and its
  action is `%windir%\system32\rundll32.exe dfdts.dll,DfdGetDefaultPolicyAndSMART`.
- Decisive observation: the shipped definition on 26100 has **`<Triggers />`**, an empty trigger
  element, plus `<Hidden>true</Hidden>` and a `MaintenanceSettings` block (`Period` 14 days,
  `Deadline` 1 month). There is no weekly trigger and no trigger of any kind. This observation
  survives the provenance rule because no enable or disable operation produces an empty trigger set;
  those operations rewrite `<Enabled>`, not `<Triggers>`.

**Corrections needed:** Two, and read the evidence caveat carefully.

1. **The task has no trigger, so the copy is wrong about what it stops.** The round-one claim that
   "its default trigger is weekly" came from a Windows 7 task-reference page and does not describe
   the target platform. On 26100 the only scheduling hint is the `MaintenanceSettings` block, which
   makes the task a candidate for Automatic Maintenance rather than a scheduled job. The `info` must
   stop describing a recurring upload that does not exist on 24H2 or later.

2. **The stock-default option has no established provenance, so it must not ship as written.** The
   earlier claim that the task ships `<Enabled>false</Enabled>`, which would have made
   "Enabled (Stock Default)" turn on a hidden CEIP collector Windows shipped switched off, rested on
   the modified validation machine and is **withdrawn**; see open question 1. That means the current
   literal is neither confirmed right nor confirmed wrong: it is unsourced. Under the
   `_harmful-revert.md` rule, a revert value must be established from documentation, shipped ADMX, or
   a known-clean image of the target build, never assumed. Because ADR-0003 makes System Default a
   selectable state whenever a snapshot exists, an unsourced literal here is a live risk on the exact
   path a cautious user takes. Do not ship the stock option until a clean 26100 image settles it.

Recommendation to the maintainer: either drop the tweak, or keep it and gate the stock option on the
clean-image check. Do not extend it to `Microsoft-Windows-DiskDiagnosticResolver`, which is the task
that surfaces the failing-disk warning, and whose shipped enabled-state is equally unestablished.

**Ready-to-paste info block** (honest about what is and is not known; do not ship the stock option
until open question 1 is closed):

````yaml
    info: |
      **Stops the hidden task that reports drive SMART data to Microsoft, without touching failing-disk warnings.**

      ## What it does
      Disables the DiskDiagnosticDataCollector scheduled task
      (`\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector`). Its own
      description says it "reports general disk and system information to Microsoft for users
      participating in the Customer Experience Program", and it runs
      `rundll32.exe dfdts.dll,DfdGetDefaultPolicyAndSMART`.

      ## Benefits
      - **Drive data stays local**: SMART attributes and disk information are not sent to Microsoft
      - **Failing-disk warnings intact**: the separate DiskDiagnosticResolver task is what warns you, and this tweak does not touch it
      - **Hidden task made explicit**: the task is marked hidden in Task Scheduler, so this surfaces a control you would otherwise not see

      ## Drawbacks
      - **Nothing was scheduled**: on Windows 11 24H2 this task carries no trigger at all, only maintenance settings, so disabling it stops no recurring run
      - **CEIP-gated anyway**: the description ties the reporting to Customer Experience Program participation
      - **Updates can re-enable it**: tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021
      - **Takes effect**: immediately, since a scheduled task changes state as soon as it is disabled
      - **Reverting**: restores the previous enabled state from the snapshot
      - Do not disable the separate DiskDiagnosticResolver task, which is what warns you about a failing drive

      ## Recommendation
      Apply it if you want the control to be explicit, but expect no observable change, because the
      task has no trigger on current builds. Skip it if you are looking for a difference you can
      measure.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [CEIPEnable, the Customer Experience Improvement Program control](https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable)
      - [Microsoft-Windows-DiskDiagnosticDataCollector task reference](https://windows.fyicenter.com/4257_Microsoft-Windows-DiskDiagnosticDataCollector_Scheduled_Task_on_Windows_7.html)
````

**Sources:**
1. Shipped task definition on Windows 11 24H2 build 26100: `Microsoft-Windows-DiskDiagnosticDataCollector` contains `<Hidden>true</Hidden>`, a `MaintenanceSettings` block and, decisively, an empty `<Triggers />` element (tier A for the trigger observation only, because no enable or disable operation produces an empty trigger set; the `<Enabled>` value read on that machine is withdrawn and is not used)
2. "Microsoft-Windows-DiskDiagnosticDataCollector" scheduled task reference (full path, folder, verbatim task description and action; its weekly-trigger claim describes Windows 7 and does not hold on 24H2), https://windows.fyicenter.com/4257_Microsoft-Windows-DiskDiagnosticDataCollector_Scheduled_Task_on_Windows_7.html (tier C)
3. "Microsoft-Windows-DiskDiagnosticResolver" scheduled task reference (separate task, SMART predictive failure warning), https://windows.fyicenter.com/4384_Microsoft-Windows-DiskDiagnosticResolver_Scheduled_Task_on_Windows_8.html (tier C)
4. CEIPEnable, https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable (tier A, for the Customer Experience Improvement Program framing)
5. Windows-10-Hardening, `components/scheduled_tasks.bat` (identical full path), https://github.com/aghorler/Windows-10-Hardening/blob/master/components/scheduled_tasks.bat (tier D, corroborating the path form only)

### `task_device_census` Devicecensus task

**Verdict:** INCORRECT

**Mechanism, corrected:**
- Full task path: `\Microsoft\Windows\Device Information\Device`, whose action is
  `devicecensus.exe SystemCxt` and which runs daily.
- Second effect needed, `optional: true` because it is absent on some builds:
  `\Microsoft\Windows\Device Information\Device User`, whose action is `devicecensus.exe UserCxt` and
  which runs at user logon.
- Option "Disabled" would set both tasks Disabled; option "Enabled (Stock Default)" would set them
  Enabled.
- `devicecensus.exe` inventories the device and reports hardware and configuration data used for
  update eligibility and targeting. That part of the YAML's description is accurate.

**Mechanism as authored (wrong):** `\Microsoft\Windows\Device Information\Devicecensus`. **That task
does not exist.** `Devicecensus` is the name of the *executable*
(`%WinDir%\System32\devicecensus.exe`), not the name of a task. Two independent references that walk
the Task Scheduler tree describe the `Device Information` folder as containing a task called
`Device`, and independent tooling that disables the census uses
`\Microsoft\Windows\Device Information\Device`. No source found at any tier refers to a task named
`Devicecensus`. As authored the tweak does nothing, and the "Enabled (Stock Default)" revert has
nothing to restore.

**Corrections needed:** One, with two parts. Change the path to
`\Microsoft\Windows\Device Information\Device`, and add a second, `optional: true` effect for
`\Microsoft\Windows\Device Information\Device User`, because disabling only the system-context task
leaves the user-context census running at every logon. Until this is fixed the tweak is inert. The
shipped enabled-state of both tasks is an open question in common with the rest of this file (open
question 3).

**Ready-to-paste info block** (written for the corrected path):

````yaml
    info: |
      **Stops the daily inventory that reports your hardware and configuration to Microsoft.**

      ## What it does
      Disables the device census scheduled tasks under `\Microsoft\Windows\Device Information\`:
      `Device`, which runs `devicecensus.exe SystemCxt` daily, and `Device User`, which runs
      `devicecensus.exe UserCxt` at logon. They inventory the machine and report hardware and
      configuration data used for update eligibility and targeting.

      ## Benefits
      - **Stops a daily upload**: this is one of the few telemetry tasks with a real recurring schedule
      - **Covers both contexts**: the system and the user census both stop, so nothing runs at logon either
      - **No user-facing feature**: nothing you interact with depends on the census

      ## Drawbacks
      - **Update targeting degrades**: eligibility signals go stale, so feature update offers may be less well matched to this machine
      - **Enterprise reporting breaks**: Windows Update for Business reports and Update Compliance read census data, so those go quiet
      - **Updates can re-enable it**: tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021; the `Device User` task is not present on every build
      - **Takes effect**: immediately, since a scheduled task changes state as soon as it is disabled
      - **Reverting**: restores the previous enabled state from the snapshot
      - This does not block Windows Update; it only removes an input to how updates are targeted

      ## Recommendation
      Apply it if you want less hardware and configuration data leaving the machine and accept
      slightly less precise update targeting. Leave it enabled on a machine that reports into Windows
      Update for Business reports or Update Compliance.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [What is devicecensus.exe on Windows, walking the Device Information task folder](https://www.ghacks.net/2019/09/23/what-is-devicecensus-exe-on-windows-10-and-why-does-it-need-internet-connectivity/)
      - [What is Device Census or devicecensus.exe in Windows](https://www.majorgeeks.com/content/page/what_is_devicecensus_exe.html)
````

**Sources:**
1. gHacks, "What is devicecensus.exe on Windows 10" (walks Task Scheduler Library > Microsoft > Windows > Device Information and states the folder contains "a single task called Device", with `devicecensus.exe` as its action), https://www.ghacks.net/2019/09/23/what-is-devicecensus-exe-on-windows-10-and-why-does-it-need-internet-connectivity/ (tier C)
2. MajorGeeks, "What is Device Census or devicecensus.exe in Windows 10?" (same navigation path, task named `Device`, "Multiple triggers defined"), https://www.majorgeeks.com/content/page/what_is_devicecensus_exe.html (tier C)
3. windows10fixup `Fixup.ps1` (disables `Microsoft\Windows\Device Information\Device`; no `Devicecensus` task name appears anywhere in the script), https://github.com/iDigitalFlame/windows10fixup (tier D, corroborating the path form used by working tooling)
4. Direct enumeration of `C:\Windows\System32\Tasks\Microsoft\Windows\Device Information` on build 26100 returns `Device` and `Device User`, and no `Devicecensus` (tier A for task presence, which is an image fact rather than a configuration default)
