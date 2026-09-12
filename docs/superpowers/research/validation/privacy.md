# Privacy and telemetry tweak validation

Rebuilt 2026-07-27. Source corpus: `src-tauri/tweaks/privacy.yaml` (29 tweaks as authored) plus the
verified gap additions. Target platform: **Windows 11 24H2 (build 26100) and newer, including 25H2**.
Windows 10 IoT Enterprise LTSC 2021 is a low-priority secondary target and is called out per entry
where a mechanism does not reach it.

This document consolidates four earlier passes: the original category validation, two adversarial
re-attacks, the adjudications in `_verification.md` (which override this document wherever they
differ), the hive audit in `_policy-hive-audit.md`, and the revert-safety findings in
`_harmful-revert.md`. It then folds in the gap proposals that survived adversarial verification in
`_verify-gaps-a-high.md`, `_verify-gaps-a-medlow.md` and `_verify-gaps-b-medlow.md`.

**Three tweaks have left this category.** `disable_recall_snapshots`, `remove_recall_component` and
`disable_click_to_do` are all Windows AI / Copilot+ controls under
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`. They move to the new `ai` category and are
validated in `ai.md`. Nothing about them is repeated here; the WindowsAI key does not appear in this
document again.

**Entry count.** 29 authored, minus the 3 moved to `ai`, equals 26 carried forward. Eight new tweaks
are added and one existing tweak (`disable_suggested_content_settings`) is extended with a fourth
value rather than duplicated. That is **34 entries** in this document, covering nine additions.

**On citations.** Shipped ADMX, ADML and binary-resource evidence is tier A but has no URL by nature,
so those sources appear as plain bullets. Every URL cited here was reached during one of the passes
above.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `disable_diagnostic_data` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `info` promises an edition-dependent floor (0 on Enterprise) that no option implements |
| `disable_ceip_tasks` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `\Microsoft\Windows\Autochk\Proxy` uncovered; `info` overstates the task effect; shipped task enabled-state unresolved |
| `disable_compat_appraiser` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `ProgramDataUpdater` does not exist on 26100 and is not optional; `Microsoft Compatibility Appraiser Exp` uncovered |
| `disable_feedback_notifications` | VERIFIED | low | Microsoft-documented | none |
| `disable_feedback_frequency` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | "Never" requires `PeriodInNanoSeconds` = 0, not `absent` |
| `disable_onesettings_downloads` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | No `windows` gate for the Windows 11 21H2+ floor; the "can silently re-toggle telemetry" headline is unsourced |
| `disable_device_name_in_telemetry` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `info` claims a privacy gain that cannot occur; the write is behaviourally identical to the shipped default |
| `disable_activity_history` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot: true` is contradicted by all three shipped ADML help strings |
| `disable_cloud_clipboard` | VERIFIED | low | Microsoft-documented | none |
| `disable_online_speech_recognition` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | "On (Stock Default) = 1" fabricates a consent; revert must delete the value |
| `disable_inking_typing_personalization` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `AllowInputPersonalization` is the online-speech policy, not an inking control; consent-derived literals used as defaults |
| `disable_advertising_id` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Incomplete: missing the documented `HKLM\...\AdvertisingInfo\Enabled` = 0 |
| `disable_tailored_experiences` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | The policy is `class="User"`; the HKLM write is in the wrong hive |
| `disable_location_tracking` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Missing the documented `LetAppsAccessLocation` = 2; ConsentStore write is tier C only |
| `disable_app_diagnostics` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Scope claim overstated (packaged apps only); ConsentStore write is tier C only |
| `disable_find_my_device` | VERIFIED | medium | Microsoft-documented | none |
| `disable_settings_sync` | VERIFIED | low | Microsoft-documented | none |
| `disable_error_reporting` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Policy-key path unconfirmed at tier A; `info` overstates the dump-suppression effect |
| `disable_suggested_content_settings` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Stock default must be `absent`; extend with `SubscribedContent-353698Enabled` |
| `disable_start_app_suggestions` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Missing `SubscribedContent-338388Enabled` and the Windows 11 Recommended-section control |
| `disable_app_launch_tracking` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | "On (Stock Default) = 1" is wrong; the value is absent on 26100 |
| `disable_lockscreen_spotlight_ads` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Needs a split default: keep `RotatingLockScreenOverlayEnabled` = 1, change 338387 to `absent` |
| `disable_tips_and_suggestions` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `DisableSoftLanding` is Enterprise / Education / IoT only and is silently ignored on Home and Pro |
| `disable_consumer_features` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `info` describes a `SilentInstalledAppsEnabled` effect that is not in the effects list |
| `disable_edge_telemetry` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `MetricsReportingEnabled` and `SendSiteInfoToImproveServices` are obsolete and dead after Edge 88 |
| `disable_language_list_access` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | "Allowed (Stock Default) = 0" should be `absent` |
| `disable_explorer_cloud_recommendations` | VERIFIED | low | Microsoft-documented | none (new) |
| `disable_app_device_inventory` | VERIFIED | low | Microsoft-documented | none (new); the value really is spelled `DisableAPISamping` |
| `disable_search_history` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Drop `DisableSearchHistory` entirely; revert must delete, not write 1 |
| `disable_cloud_content_search` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Both HKCU stock defaults are `absent`; `AllowCloudSearch` has a third enum value 2 |
| `disable_voice_activation` | VERIFIED | low | Microsoft-documented | none (new) |
| `disable_windows_backup` | VERIFIED | low | Microsoft-documented | none (new), but the copy constraint below is mandatory |
| `disable_online_tips` | VERIFIED | low | Microsoft-documented | none (new); privacy.sexy's key for this value is wrong and must not be copied |
| `disable_wer_service` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Drop the CPU claim; the shipped start type is unresolved |

Tally: 9 VERIFIED, 25 VERIFIED-WITH-CORRECTION, 0 UNVERIFIED, 0 DISPUTED, 0 INCORRECT.

## Corrections required

1. **`disable_diagnostic_data`**: the `info` says the tweak pushes the level "to the minimum your edition allows" and that Enterprise, Education and IoT / LTSC "can reach the Security level (0)". The tweak offers one non-default option and it writes 1. No option can reach 0 on any edition. Either add a Security (0) option gated to those SKUs, or correct the text. The `info` block below takes the second route.
2. **`disable_ceip_tasks`**: a CEIP uploader task is uncovered. `\Microsoft\Windows\Autochk\Proxy` sits outside the CEIP task folder, but Microsoft's shipped description for it, resource -102 in `%SystemRoot%\System32\acproxy.dll`, reads "This task collects and uploads autochk SQM data if opted-in to the Microsoft Customer Experience Improvement Program." Add it as a task effect, `optional: true` with `if_missing: disabled`.
3. **`disable_ceip_tasks`**: the `info` overstates the task effects. `wsqmcons.exe,-107` for `Consolidator` says "**If the user has consented** to participate ... this job collects and sends usage data to Microsoft", and `usbceip.dll,-602` for `UsbCeip` says "**If the user has not consented** ... this task does not do anything." On a machine that never opted in, disabling these tasks stops nothing. Credit the `CEIPEnable` policy effect as the part that does the work.
4. **`disable_ceip_tasks`**: the shipped enabled-state of `Consolidator`, `UsbCeip` and `KernelCeipTask` is UNRESOLVED, and the "Enabled (Stock Default)" option asserts an answer. Enabling or disabling a task rewrites the `<Enabled>` element in its own XML under `C:\Windows\System32\Tasks`, so no live machine is evidence of shipped state. Until a clean 26100 image settles it, the safe revert restores the pre-apply state from the snapshot rather than writing a literal "enabled".
5. **`disable_compat_appraiser`**: `task_progdata` must be marked `optional: true` with `if_missing: disabled`. `ProgramDataUpdater` does not exist on build 26100; a full enumeration of `\Microsoft\Windows\Application Experience\` returns `MareBackup`, `Microsoft Compatibility Appraiser`, `Microsoft Compatibility Appraiser Exp`, `PcaPatchDbTask`, `SdbinstMergeDbTask` and `StartupAppTask` only. As authored the tweak names a task that is absent on its primary platform.
6. **`disable_compat_appraiser`**: add `\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser Exp` as an `optional: true` task effect. It exists and is Ready on 26100 and runs appraiser work the tweak otherwise leaves alone.
7. **`disable_feedback_frequency`**: the "Never" option is wrong. Microsoft's table gives Never as `PeriodInNanoSeconds` = 0 **and** `NumberOfSIUFInPeriod` = 0. The YAML writes `num_siuf: 0, period_ns: absent`, which is not a documented combination. Set `period_ns: 0`.
8. **`disable_onesettings_downloads`**: missing applicability gate. Policy CSP gives the applicable OS as "Windows 11, version 21H2 [10.0.22000] and later". Add `windows: { build: ">=22000" }`, or state the floor in `info`.
9. **`disable_onesettings_downloads`**: the headline benefit is unsourced. Microsoft describes OneSettings only as a configuration-download service; nothing supports "can silently re-toggle telemetry". Blocking it does not reduce diagnostic data collection, which is governed by `AllowTelemetry` and the Connected User Experiences and Telemetry service. Move Microsoft's warning that dependent apps "may stop working" out of a parenthetical and into the drawbacks.
10. **`disable_device_name_in_telemetry`**: the `info` promises a gain that cannot occur. The shipped `DataCollection.adml` says "If you disable or do not configure this policy setting, then device name will not be sent to Microsoft as part of Windows diagnostic data", and Policy CSP gives Default Value 0. Writing 0 and leaving the value absent are behaviourally identical on every build in range. Rewrite the benefit as pinning against a later MDM profile or administrative template, and consider renaming the option from "Excluded".
11. **`disable_device_name_in_telemetry`**, for future maintainers: do **not** "correct" the value name to `AllowDeviceNameInDiagnosticData` to match the Policy CSP heading. The shipped `DataCollection.admx` defines policy `AllowDeviceNameInDiagnosticData` with `valueName="AllowDeviceNameInTelemetry"`. The YAML uses the registry value name, which is correct; changing it produces a silently ignored write.
12. **`disable_activity_history`**: `requires_reboot: true` is contradicted three times. The shipped `en-US\OSPolicy.adml` on 26100 closes `EnableActivityFeed_Help`, `PublishUserActivities_Help` and `UploadUserActivities_Help` with the identical sentence "Policy change takes effect immediately". Set `requires_reboot: false` and delete the `info` sentence "a reboot helps it take full effect".
13. **`disable_online_speech_recognition`**: `HasAccepted` has no shipped default of 1. Microsoft documents only the 0 direction, and the value materialises when the user makes a consent choice in OOBE or in Settings. The stock representation is **absent**. The revert must **delete** the value. A revert that writes 1 manufactures a cloud-speech consent the user never granted; if the engine cannot express "restore to absent" for this effect it needs a delete-capable revert.
14. **`disable_inking_typing_personalization`**: `AllowInputPersonalization` is mis-attributed. It is the Group Policy "Allow users to enable online speech recognition services" from **`Globalization.admx`**, at `HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization`. The hive is right; the tweak it sits in is not. Either move the effect to `disable_online_speech_recognition`, or replace it with the genuine inking policy, `AllowLinguisticDataCollection` ("Improve inking and typing recognition") from **`TextInput.admx`** at `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\TextInput`. The two are not equivalent and the choice must be deliberate.
15. **`disable_inking_typing_personalization`**: `accepted_privacy: 1` and `harvest_contacts: 1` as "Stock Default" are consent-derived values, not fixed Windows defaults. Same defect class as correction 13.
16. **`disable_advertising_id`**: the tweak is *incomplete*, not incorrect. Microsoft documents `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo\Enabled = 0` alongside the policy value, and the tweak must add it. Keeping the HKCU write as a per-user supplement is defensible; relying on it alone is not. No Microsoft source states the HKCU value is inert, so do not remove it.
17. **`disable_tailored_experiences`**: `CloudContent.admx` declares `DisableTailoredExperiencesWithDiagnosticData` with `class="User"`, and Policy CSP `Experience/AllowTailoredExperiencesWithDiagnosticData` is User-scoped with Device explicitly unsupported. Move the policy effect to `HKCU\Software\Policies\Microsoft\Windows\CloudContent`. Do **not** conflate it with the tweak's other effect: `HKCU\...\CurrentVersion\Privacy\TailoredExperiencesWithDiagnosticDataEnabled` is the Settings-UI value and a separate knob. Both can legitimately exist.
18. **`disable_tailored_experiences`**: the "On (Stock Default)" option writes a literal 1 to `TailoredExperiencesWithDiagnosticDataEnabled`, which is consent-derived rather than a fixed default.
19. **`disable_location_tracking`**: Microsoft's documented pair for this outcome is `DisableLocation` = 1 **and** `LetAppsAccessLocation` = 2 under `HKLM\Software\Policies\Microsoft\Windows\AppPrivacy`. The YAML omits the second and substitutes an undocumented ConsentStore write. Add the policy value and label the ConsentStore write community-sourced.
20. **`disable_app_diagnostics`**: the `info` claim "Apps can no longer inspect diagnostic details about other processes on your system" is too broad. `LetAppsGetDiagnosticInfo` governs Windows (UWP / packaged) apps only. Win32 processes are unaffected.
21. **`disable_error_reporting`**: the `info` says disabling WER "suppresses local crash logs and dumps that Reliability Monitor and third-party support tools use". Kernel crash dumps (`MEMORY.DMP`) are governed by `HKLM\SYSTEM\CurrentControlSet\Control\CrashControl` and are unaffected. What is suppressed is the WER report queue, user-mode `LocalDumps` and Reliability Monitor entries. Separately, the `Software\Policies\Microsoft\Windows\Windows Error Reporting` path is not confirmed at tier A; verify against `ErrorReporting.admx` before treating it as such.
22. **`disable_suggested_content_settings`**: "On (Stock Default)" writes 1 to every value. None of them exists in the shipped `C:\Users\Default\NTUSER.DAT` on 26100, and privacy.sexy independently annotates them "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)". The default option must be `absent`.
23. **`disable_suggested_content_settings`**: extend the effect set with a fourth value, `SubscribedContent-353698Enabled`, REG_DWORD, Off = 0, Stock Default = `absent`. Do not create a second tweak for it. Win11Debloat writes it in the same block as the three already shipped, and the bare id `353698` is present in `ContentDeliveryManager.Background.dll`, `ContentDeliveryManager.Utilities.dll` and `Windows.Services.TargetedContent.dll` on 26100.
24. **`disable_start_app_suggestions`**: coverage gap. Add `SubscribedContent-338388Enabled` = 0 with a stock default of `absent` for the Windows 10 surface, and add `Start_IrisRecommendations` = 0 or the tier A `HideRecommendedSection` = 1 for the Windows 11 Recommended section the display name implies. The "Stock Default = 1" on `SystemPaneSuggestionsEnabled` itself is **correct** and must not be swept to `absent`: it ships present with data 1 in the Default user hive on 26100.
25. **`disable_app_launch_tracking`**: "On (Stock Default) = 1" is wrong. `Start_TrackProgs` is absent from every hive loaded under `HKEY_USERS` on 26100, including a near-pristine second profile, and Microsoft's instruction is to "**Create** a REG_DWORD registry setting named `Start_TrackProgs`". The default option must write `absent`.
26. **`disable_lockscreen_spotlight_ads`**: needs a split default, not one answer. `RotatingLockScreenOverlayEnabled` is present with data 1 in the shipped Default user hive on 26100 and keeps its literal; `SubscribedContent-338387Enabled` is absent and must change to `absent`.
27. **`disable_tips_and_suggestions`**: `DisableSoftLanding` (Policy CSP `Experience/AllowWindowsTips`) is documented as not supported on Pro and, by omission, not on Home. It is honored on Enterprise, Education and IoT Enterprise / IoT Enterprise LTSC only. Add the SKU caveat the sibling `disable_consumer_features` already carries.
28. **`disable_consumer_features`**: the `info` states twice that the tweak sets the per-user `SilentInstalledAppsEnabled` to 0 and builds its Home/Pro recommendation on that. No such effect exists in its effects list. Either add `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager\SilentInstalledAppsEnabled` = 0, or remove every reference to it and state that the tweak is inert on Home and Pro.
29. **`disable_edge_telemetry`**: `MetricsReportingEnabled` and `SendSiteInfoToImproveServices` both carry Microsoft's banner "OBSOLETE: This policy is obsolete and doesn't work after Microsoft Edge version 88", supported versions 77 to 88. Half the tweak does nothing on every currently shipping Edge. Remove both effects, or gate them behind an Edge version of 88 or lower.
30. **`disable_edge_telemetry`**: the `info` says `MetricsReportingEnabled` "is independent of Windows `AllowTelemetry`". Microsoft states the opposite: the policy was replaced by Allow Telemetry on Windows 10, and unconfigured Edge "defaults to the Windows diagnostic data setting".
31. **`disable_language_list_access`**: "Allowed (Stock Default)" writes 0, but Microsoft's instruction is to "Create a **new** REG_DWORD registry setting named `HttpAcceptLanguageOptOut`", which means it is absent by default. Change the default option to `absent`. Behaviourally 0 and absent are equivalent here, so the risk is cosmetic, but a default must be the real default.
32. **`disable_search_history`** (new): drop the `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer\DisableSearchHistory` half entirely. The shipped 26100 `Search.admx` declares it `class="User"` (so HKLM is the wrong hive), carries `<supportedOn ref="Win8Only" />`, and the literal appears in exactly one shipped binary, `SHCore.dll`, the shell policy table, and in no search component. Keep only `IsDeviceSearchHistoryEnabled`, and the revert must **delete** it, not write 1.
33. **`disable_cloud_content_search`** (new): the two HKCU values are absent by default, so the revert deletes them rather than writing 1. Also note that `AllowCloudSearch` is an enum with a third value: 0 = Disable Cloud Search, 1 = Enable Cloud Search, 2 = User Selected. A faithful control offers all three; otherwise document that the tweak only writes 0.
34. **`disable_app_device_inventory`** (new): the value name is `DisableAPISamping`, not `DisableAPISampling`. The misspelling is Microsoft's and is genuine: a search for the correct spelling across every ADMX on 26100 returns nothing. Also do not write the ADMX `name=` attributes (`TurnOffAPISamping`, `TurnOffInstallTracing`, `TurnOffApplicationFootprint`, `TurnOffWin32AppBackup`) into the registry; they are policy names, not value names.
35. **`disable_windows_backup`** (new), copy constraint, non-negotiable: by Microsoft's own ADML text, 0 and absent are documented as behaviourally identical. The drawbacks must say that applying this may produce no visible change today and that the Windows Backup app can still be run manually. Do not describe it as "stops Windows backing up your files".
36. **`disable_wer_service`** (new): drop the proposal's "stops WER consuming CPU on a crash" claim; it is unmeasured. The honest benefit is that crash reports stop being generated and uploaded. The shipped default start type is **unresolved**: the proposal says Manual, but no authoritative source confirms it, so the revert must restore the pre-apply start type from the snapshot rather than writing a literal.
37. **Corpus-wide evidence caveat, not a tweak defect.** Corrections 22, 25 and 26 rest partly on reads of the validation machine's `HKEY_USERS` hives and its shipped `C:\Users\Default\NTUSER.DAT`. That machine is heavily modified by its owner. The Default-hive reads are the stronger of the two, since that hive is the template for new profiles, but any correction resting on a live `HKEY_USERS` read should be re-confirmed on a clean 26100 image before a YAML change is made on its authority. Claims sourced to `C:\Windows\PolicyDefinitions`, to shipped binary resource strings, or to Microsoft Learn are unaffected.

## New in this revision

**Moved out.** `disable_recall_snapshots`, `remove_recall_component` and `disable_click_to_do` now
live in the `ai` category. Their corrections (the 26100.3915 servicing floor, the "Available" option
that does not make Recall available, the missing Copilot+ hardware gate) travel with them.

**Eight new tweaks, all adversarially verified.**

| Tweak | Verified in | Core mechanism |
|---|---|---|
| `disable_explorer_cloud_recommendations` | `_verify-gaps-a-high.md` entry 2 | `DisableGraphRecentItems`, stops File Explorer requesting cloud file metadata |
| `disable_app_device_inventory` | `_verify-gaps-a-high.md` entry 4 | Four 24H2-era AppCompat collectors, each read by its own named binary |
| `disable_search_history` | `_verify-gaps-a-medlow.md` item 16 | `IsDeviceSearchHistoryEnabled` only, policy half dropped |
| `disable_cloud_content_search` | `_verify-gaps-a-medlow.md` item 17 | `AllowCloudSearch` plus the two per-user cloud search values |
| `disable_voice_activation` | `_verify-gaps-a-medlow.md` item 18 | `LetAppsActivateWithVoice` and the above-lock variant, Force Deny = 2 |
| `disable_windows_backup` | `_verify-gaps-a-medlow.md` item 19 | `EnableWindowsBackup` = 0, a lock rather than a behaviour change |
| `disable_online_tips` | `_verify-gaps-a-medlow.md` item 20 | `AllowOnlineTips` = 0, stops the Settings app fetching help content |
| `disable_wer_service` | `_verify-gaps-b-medlow.md` item 25 | `WerSvc` service, the third leg of a job the corpus had two-thirds done |

**One extension, not a new tweak.** `disable_suggested_content_settings` gains a fourth value,
`SubscribedContent-353698Enabled`. See correction 23.

**Two upstream defects recorded so they are not re-litigated.** privacy.sexy writes
`DisableSearchHistory` to HKLM (the shipped ADMX says `class="User"`) and writes `AllowOnlineTips` to
`HKLM\SOFTWARE\Policies\Microsoft\Windows\System` (the shipped ADMX says
`Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`). In both cases the shipped ADMX wins
and this corpus is on the correct side.

## Tweak entries

### `disable_diagnostic_data` Diagnostic data level

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value `AllowTelemetry`,
`REG_DWORD`.

| Option | Value |
|---|---|
| Required only | `1` |
| Windows default (Stock Default) | `absent` |

This is the Group Policy "Allow Diagnostic Data" (older label "Allow Telemetry") from
`DataCollection.admx`. Microsoft's allowed values are 0 (Security), 1 (Basic / Required, the
framework default), and 3 (Full / Optional). Value 2 (Enhanced) exists only on Windows 10 1809 and
earlier and Windows Server 2016 / 2019. If the policy is not configured, "the device will send
required diagnostic data and the end user can choose whether to send optional diagnostic data from
the Settings app", which makes `absent` the correct stock default. Value 0 is honored only on
Enterprise, Education, IoT Core and Server; Microsoft states plainly that "Using this setting on
other devices is equivalent to setting the value of 1", so writing 1 is correct and portable.

**Corrections needed:** see correction 1. The `info` describes an edition-aware minimum the tweak
does not implement, because no option writes 0. The block below states the real behaviour instead.

**Ready-to-paste info block:**

```yaml
    info: |
      **Caps Windows diagnostic data at the Required level and takes the choice away from the Settings app.**

      ## What it does
      Writes the `AllowTelemetry` policy value as 1 under `Policies\Microsoft\Windows\DataCollection`,
      the Group Policy "Allow Diagnostic Data". Optional diagnostic data stops being sent and the
      Settings toggle that would raise it again is greyed out for every user on the machine.

      ## Benefits
      - **Less outbound data**: optional diagnostics, including usage and inking samples, stop flowing
      - **Locked down**: no user or app can raise the level back to Optional from Settings
      - **Portable**: 1 is honored on every edition, unlike the Security level

      ## Drawbacks
      - **Not zero**: required diagnostic data still flows; only Enterprise, Education and IoT honor 0
      - **Service still runs**: this lowers what `DiagTrack` sends, it does not stop the service
      - **Settings greyed out**: the diagnostic data control becomes unusable until you revert
      - **Update diagnostics**: Microsoft loses one input for diagnosing failed updates on your device

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1507 and later
      - **Takes effect**: immediately, the Settings page reflects it on next open
      - **Reverting**: deletes the policy value, restoring the Windows default and the user's choice
      - Windows Insider builds override the level upward regardless of this policy

      ## Recommendation
      Apply it on any personal machine; the cost is diagnostic reach that Microsoft, not you, benefits
      from. Skip it if you are actively working with Microsoft support on an update or crash issue.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - System (AllowTelemetry)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system)
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
```

**Sources:**
1. Policy CSP - System (AllowTelemetry), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)
2. Configure Windows diagnostic data in your organization, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` on build 26100 (tier A, shipped ADMX)

### `disable_ceip_tasks` Customer Experience Improvement Program

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry value plus three scheduled tasks.

| Effect | Target | Type | Off | Stock Default |
|---|---|---|---|---|
| `ceip_enable` | `HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows` `CEIPEnable` | `REG_DWORD` | `0` | `absent` |
| `task_consolidator` | `\Microsoft\Windows\Customer Experience Improvement Program\Consolidator` | task | disabled | enabled |
| `task_usbceip` | `\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip` | task | disabled | enabled |
| `task_kernelceip` | `\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipTask` (`optional: true`, `if_missing: disabled`) | task | disabled | enabled |

Uncovered task that must be added: `\Microsoft\Windows\Autochk\Proxy`, `optional: true`,
`if_missing: disabled`.

The shipped `ICM.admx` on 26100 defines policy `CEIPEnable` with
`key="Software\Policies\Microsoft\SQMClient\Windows"`, `valueName="CEIPEnable"`, `enabledValue` 0 and
`disabledValue` 1. The YAML's polarity is exactly the ADMX enabled value, and `absent` is correct for
unconfigured. The same value appears again inside the `InternetManagement_RestrictCommunication`
policy's `enabledList` at the same key with the same 0, which is independent corroboration inside the
shipped file. The registry effect is correct as authored.

**Corrections needed:** see corrections 2, 3 and 4. The `ceip_enable` effect, its polarity and its
`absent` default need no change. The `optional: true` / `if_missing: disabled` handling on
`task_kernelceip` is the right design regardless of how the presence question resolves.

**Ready-to-paste info block:**

```yaml
    info: |
      **Opts the machine out of the legacy Customer Experience Improvement Program and stops its upload tasks.**

      ## What it does
      Sets the `CEIPEnable` policy value to 0, the Group Policy "Turn off Windows Customer Experience
      Improvement Program", and disables the CEIP scheduled tasks (`Consolidator`, `UsbCeip`, and
      `KernelCeipTask` where present) that collect and upload SQM usage data.

      ## Benefits
      - **Opts out by policy**: the `CEIPEnable` value is the part that actually stops participation
      - **Fewer wake-ups**: recurring upload tasks stop being scheduled
      - **Survives updates**: the policy value holds even when a feature update re-enables the tasks

      ## Drawbacks
      - **Often already off**: Microsoft's own task descriptions say these tasks do nothing unless you
        consented to CEIP, so on a machine that never opted in the task half changes nothing
      - **Not telemetry**: modern diagnostic data flows through `DiagTrack`, which this does not touch
      - **One task uncovered**: `\Microsoft\Windows\Autochk\Proxy` is also a CEIP SQM uploader and is
        not currently included

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 22H2
      - **Takes effect**: immediately, the tasks stop on apply
      - **Reverting**: restores the previous policy value and task states from the snapshot
      - `KernelCeipTask` is a Windows 7 / 8 era task and is usually absent on modern builds

      ## Recommendation
      Apply it; the cost is nil and the policy opt-out is durable. Do not treat it as a telemetry
      control: if reducing outbound diagnostics is the goal, use the diagnostic data level tweak.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [CEIPEnable](https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable)
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. CEIPEnable, https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\ICM.admx` on build 26100 (tier A, shipped ADMX)
3. Shipped task description strings `wsqmcons.exe,-107` (`Consolidator`), `usbceip.dll,-602` (`UsbCeip`), `acproxy.dll,-102` (`Autochk\Proxy`) (tier A, shipped binary resources; evidence of what each task is, not of its default state)
4. Manage connections from Windows components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### `disable_compat_appraiser` Compatibility Appraiser tasks

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two registry values under `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat` plus
three scheduled tasks under `\Microsoft\Windows\Application Experience\`.

| Effect | Target | Type | Off | Stock Default |
|---|---|---|---|---|
| `ait_enable` | `AITEnable` | `REG_DWORD` | `0` | `absent` |
| `disable_inventory` | `DisableInventory` | `REG_DWORD` | `1` | `absent` |
| `task_appraiser` | `Microsoft Compatibility Appraiser` | task | disabled | enabled |
| `task_progdata` | `ProgramDataUpdater` | task | disabled | enabled |
| `task_startup` | `StartupAppTask` | task | disabled | enabled |

The shipped `AppCompat.admx` on 26100 defines `AppCompatTurnOffApplicationImpactTelemetry` writing
`AITEnable` with `enabledValue` 0, and `AppCompatTurnOffProgramInventory` writing `DisableInventory`
with no explicit value pair, meaning the ADMX default of enabled = 1. Both polarities in the YAML are
correct. DISA STIG finding V-253385 independently specifies `DisableInventory` = 1 at that key.

**Corrected task set.** A full enumeration of `\Microsoft\Windows\Application Experience\` on 26100
returns exactly six tasks: `MareBackup`, `Microsoft Compatibility Appraiser`,
`Microsoft Compatibility Appraiser Exp`, `PcaPatchDbTask`, `SdbinstMergeDbTask`, `StartupAppTask`.

Wrong as authored: `task_progdata` targets `ProgramDataUpdater`, which does not exist on the primary
platform and carries neither `optional: true` nor `if_missing:`. Missing entirely:
`Microsoft Compatibility Appraiser Exp`, which exists, is Ready, and runs appraiser work.

**Corrections needed:** see corrections 5 and 6. The registry half needs no change. The performance
claim in the current `info` ("the heaviest telemetry task and often causes noticeable CPU and disk
spikes") is widely reported but no tier A or B measurement supports it, so it is stated below as an
expectation rather than a fact.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the Windows upgrade-readiness appraiser and its inventory collection, including the recurring CompatTelRunner workload.**

      ## What it does
      Sets the `AITEnable` and `DisableInventory` policy values under
      `Policies\Microsoft\Windows\AppCompat`, turning off Application Impact Telemetry and the
      Application Compatibility Program Inventory, then disables the Application Experience scheduled
      tasks that run `CompatTelRunner.exe` and feed the inventory.

      ## Benefits
      - **No appraiser runs**: the recurring `CompatTelRunner.exe` workload stops being scheduled
      - **Inventory off**: installed-application inventory stops being gathered and reported
      - **Policy-backed**: both values are shipped Group Policy settings, not undocumented keys

      ## Drawbacks
      - **Upgrade gating**: appraiser data feeds Microsoft's safeguard-hold logic, so compatibility
        problems are less likely to be caught before a feature update
      - **Restored by updates**: feature updates re-enable the tasks; the policy values persist
      - **Load claim unproven**: the CPU and disk relief is widely reported but not measured by any
        Microsoft or benchmark source

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 22H2
      - **Takes effect**: immediately, the tasks stop on apply
      - **Reverting**: restores the previous policy values and task states from the snapshot
      - Do not extend this by deleting or renaming `CompatTelRunner.exe`; servicing restores it and
        removal has historically broken Windows updates

      ## Recommendation
      Apply it on a machine you keep on its current Windows version. Hold off if you are about to run
      a feature upgrade and want Microsoft's compatibility checks working in your favour.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [DISA STIG Windows 11 V-253385, Application Compatibility Program Inventory](https://www.stigviewer.com/stigs/microsoft_windows_11/2022-06-24/finding/V-253385)
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\AppCompat.admx` on build 26100 (tier A, shipped ADMX)
2. DISA STIG, Windows 11, V-253385, https://www.stigviewer.com/stigs/microsoft_windows_11/2022-06-24/finding/V-253385 (tier B)
3. `Get-ScheduledTask -TaskPath '\Microsoft\Windows\Application Experience\'` and `schtasks /query` on build 26100 (tier A, shipped OS state for task presence)

### `disable_feedback_notifications` Feedback request notifications

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value
`DoNotShowFeedbackNotifications`, `REG_DWORD`. Off writes `1`; "On (Stock Default)" is `absent`.

Microsoft documents this verbatim: "Create a REG_DWORD registry setting named
**DoNotShowFeedbackNotifications** in
**HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows\DataCollection** with a value of 1 (one)",
equivalent to enabling Computer Configuration > Administrative Templates > Windows Components > Data
Collection and Preview Builds > "Do not show feedback notifications". It suppresses Feedback Hub
survey prompts machine-wide. No SKU gate is documented.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows interrupting you with Feedback Hub survey prompts.**

      ## What it does
      Sets the `DoNotShowFeedbackNotifications` policy value under
      `Policies\Microsoft\Windows\DataCollection`, the Group Policy "Do not show feedback
      notifications". Feedback requests stop appearing for every user on the machine.

      ## Benefits
      - **No interruptions**: the periodic "How likely are you to recommend Windows" prompts stop
      - **Machine-wide**: one policy covers every account, unlike the per-user frequency setting
      - **Zero functional cost**: nothing depends on the prompts

      ## Drawbacks
      - **Not telemetry**: this hides prompts only; diagnostic data collection is unchanged
      - **Insiders lose a channel**: on a Windows Insider machine you stop being asked for the
        feedback the program exists to gather

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 all supported versions
      - **Takes effect**: immediately
      - **Reverting**: deletes the policy value, restoring the Windows default
      - You can still open Feedback Hub and submit feedback yourself at any time

      ## Recommendation
      Apply it on any machine that is not enrolled in the Windows Insider Program. If you are an
      Insider and want to be asked, leave it alone.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.16 Feedback and diagnostics](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Shipped `DataCollection.admx` on Windows 11 build 26100, policy "Do not show feedback notifications"
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.16 Feedback and diagnostics, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` on build 26100 (tier A, shipped ADMX)

### `disable_feedback_frequency` Feedback prompt frequency

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** `HKCU\Software\Microsoft\Siuf\Rules`, two `REG_DWORD` values.

| Option | `NumberOfSIUFInPeriod` | `PeriodInNanoSeconds` |
|---|---|---|
| Never | `0` | `0` |
| Automatic (Stock Default) | `absent` | `absent` |

**Wrong as authored:** the "Never" option writes `num_siuf: 0` with `period_ns: absent`, which is not
one of the five states Microsoft documents. Microsoft's published table is exact:

| Setting | `PeriodInNanoSeconds` | `NumberOfSIUFInPeriod` |
|---|---|---|
| Automatically | delete the value | delete the value |
| Never | 0 | 0 |
| Always | 100000000 | delete the value |
| Once a day | 864000000000 | 1 |
| Once a week | 6048000000000 | 1 |

**Corrections needed:** see correction 7. Set `period_ns: 0` in the "Never" option.

**Ready-to-paste info block:**

```yaml
    info: |
      **Sets your account's Windows feedback prompt schedule to never.**

      ## What it does
      Writes `NumberOfSIUFInPeriod` and `PeriodInNanoSeconds` as 0 under `HKCU\Software\Microsoft\Siuf\Rules`,
      the pair Microsoft documents for the "Never" option of Settings > Privacy and security >
      Diagnostics and feedback > Feedback frequency.

      ## Benefits
      - **No prompts**: Windows stops asking this account for feedback
      - **No admin needed**: it is a per-user setting, so it works without elevation
      - **Documented pair**: (0, 0) is Microsoft's own mapping for "Never"

      ## Drawbacks
      - **Current user only**: other accounts on the machine keep their own schedule
      - **Redundant with the policy**: if the machine-wide feedback notification policy is already
        applied, this adds nothing
      - **A user can undo it**: nothing locks the Settings control

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 all supported versions
      - **Takes effect**: immediately
      - **Reverting**: deletes both values, returning the schedule to Automatic
      - Microsoft's table lists both values as REG_DWORD yet gives "Once a day" and "Once a week" data
        that exceeds the 32-bit range; that inconsistency is Microsoft's and does not affect the 0 pair

      ## Recommendation
      Apply it if you cannot or do not want to set a machine-wide policy. If you have admin rights,
      the machine-wide feedback notification tweak is the more durable choice.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.16 Feedback and diagnostics](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [privacy.sexy, which writes the same `Siuf\Rules` pair](https://github.com/undergroundwires/privacy.sexy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.16 Feedback and diagnostics, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A, the five-state mapping table)
2. privacy.sexy, which writes the same `HKCU\Software\Microsoft\Siuf\Rules` pair, https://github.com/undergroundwires/privacy.sexy (tier C, corroboration only)

### `disable_onesettings_downloads` OneSettings config downloads

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value
`DisableOneSettingsDownloads`, `REG_DWORD`. Blocked writes `1`; "Allowed (Stock Default)" is `absent`.

The shipped `DataCollection.admx` on 26100 defines it as `class="Machine"` at
`key="Software\Policies\Microsoft\Windows\DataCollection"` with `enabledValue` 1 and `disabledValue`
0, and the ADML explain string matches the Policy CSP text word for word. Key, value name, type,
polarity and the `absent` default are all correct. Microsoft: "If you enable this policy, Windows
won't attempt to connect with the OneSettings Service." The connections article describes OneSettings
as the service "used by Windows components and apps, such as the telemetry service, to dynamically
update their configuration" and warns "If you turn off this service, apps using this service may stop
working."

**Corrections needed:** see corrections 8 and 9. No `windows:` gate for the documented Windows 11
21H2 floor, and the current `info` sells a telemetry control the policy does not deliver.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows periodically downloading configuration from Microsoft's OneSettings service.**

      ## What it does
      Sets `DisableOneSettingsDownloads` to 1 under `Policies\Microsoft\Windows\DataCollection`, the
      Group Policy "Disable OneSettings Downloads". Windows no longer connects to OneSettings, the
      service that lets Windows components and apps update their configuration between updates.

      ## Benefits
      - **No remote config**: components keep the configuration they shipped with
      - **Fewer connections**: one recurring outbound call to Microsoft stops
      - **Experiment opt-out**: staged feature flags delivered this way stop arriving

      ## Drawbacks
      - **Apps may break**: Microsoft's own warning is that apps using this service "may stop working"
      - **Not a telemetry control**: diagnostic data volume is unchanged; that is governed by the
        diagnostic data level and the Connected User Experiences and Telemetry service
      - **No staged fixes**: configuration-only mitigations Microsoft ships this way will not reach you

      ## Good to know
      - **Applies to**: Windows 11 21H2 (build 22000) and newer, per Microsoft's applicability table
      - **Takes effect**: immediately
      - **Reverting**: deletes the policy value, restoring the Windows default
      - On Windows 10 the value is written but is outside Microsoft's documented applicability range

      ## Recommendation
      Apply it if you want your local configuration to stay put and you accept that some Microsoft
      apps may misbehave. Skip it on a machine where Store or Microsoft 365 apps must be reliable.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - System (DisableOneSettingsDownloads)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system)
      - [Manage connections from Windows components to Microsoft services, 31 Services Configuration](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Policy CSP - System (DisableOneSettingsDownloads), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)
2. Manage connections from Windows components to Microsoft services, section 31 Services Configuration, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` and `en-US\DataCollection.adml` on build 26100 (tier A, shipped ADMX and ADML)

### `disable_device_name_in_telemetry` Device name in telemetry

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value
`AllowDeviceNameInTelemetry`, `REG_DWORD`. "Excluded" writes `0`; "Windows default (Stock Default)" is
`absent`.

The shipped `DataCollection.admx` on 26100 defines policy `AllowDeviceNameInDiagnosticData` as
`class="Machine"` with `valueName="AllowDeviceNameInTelemetry"`, `enabledValue` 1, `disabledValue` 0,
supported from `SUPPORTED_Windows_10_0_RS4`. Note the deliberate mismatch between the policy name and
the value name; the YAML uses the value name, which is the correct one.

The shipped ADML string `AllowDeviceNameInDiagnosticData_Explain` reads: "This policy allows the
device name to be sent to Microsoft as part of Windows diagnostic data. If you disable or do not
configure this policy setting, then device name will not be sent to Microsoft as part of Windows
diagnostic data." Policy CSP gives Default Value 0. Writing 0 and leaving the value absent are
therefore behaviourally identical on every build in range: this tweak pins a default, it does not
change behaviour.

**Corrections needed:** see corrections 10 and 11.

**Ready-to-paste info block:**

```yaml
    info: |
      **Pins the device-name exclusion so nothing can later start attaching your computer name to diagnostic data.**

      ## What it does
      Writes `AllowDeviceNameInTelemetry` as 0 under `Policies\Microsoft\Windows\DataCollection`, the
      Group Policy "Allow device name to be sent in Windows diagnostic data". Windows already excludes
      the device name when the policy is unset, so this makes the existing state explicit and
      unchangeable by a later configuration.

      ## Benefits
      - **Defends the default**: an MDM profile or administrative template cannot flip it to 1 later
      - **Explicit state**: the exclusion is recorded rather than implied
      - **No cost**: nothing on the machine depends on the device name being sent

      ## Drawbacks
      - **No change today**: the shipped default already excludes the device name, so you will not see
        or measure any difference after applying it
      - **Narrow**: it removes one field, not the diagnostic data itself
      - **Managed machines**: on a domain or Intune-managed device, a policy pushed later still wins

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later
      - **Takes effect**: immediately
      - **Reverting**: deletes the policy value; behaviour is unchanged because the default is the same
      - The registry value is `AllowDeviceNameInTelemetry` even though the policy is called
        `AllowDeviceNameInDiagnosticData`; changing the value name to match the policy name breaks it

      ## Recommendation
      Apply it if you manage the machine's configuration deliberately and want the exclusion locked in.
      If you want a visible reduction in what leaves the machine, use the diagnostic data level tweak
      instead; this one changes nothing on a stock install.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - System (AllowDeviceNameInDiagnosticData)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system)
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
```

**Sources:**
1. Policy CSP - System (AllowDeviceNameInDiagnosticData), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` and `en-US\DataCollection.adml` on build 26100 (tier A, shipped ADMX and ADML). This replaces an earlier admx.help citation, which is an unreachable third-party mirror and not an acceptable source for this corpus.

### `disable_activity_history` Activity History and Timeline

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, three `REG_DWORD` values.

| Effect | Value name | Disabled | Stock Default |
|---|---|---|---|
| `enable_feed` | `EnableActivityFeed` | `0` | `absent` |
| `publish_activities` | `PublishUserActivities` | `0` | `absent` |
| `upload_activities` | `UploadUserActivities` | `0` | `absent` |

The shipped `OSPolicy.admx` on 26100 defines all three as `class="Machine"` policies at
`key="Software\Policies\Microsoft\Windows\System"`, each with `enabledValue` 1 and `disabledValue` 0.
Policy CSP gives Default Value 1 for all three, which makes `absent` the correct stock default.

**Wrong as authored:** `requires_reboot: true`. The shipped `en-US\OSPolicy.adml` on 26100 closes all
three help strings (`EnableActivityFeed_Help`, `PublishUserActivities_Help`,
`UploadUserActivities_Help`) with the identical sentence "Policy change takes effect immediately".
There is no documented reboot, sign-out or Explorer-restart requirement for any of them.

**Corrections needed:** see correction 12. The key, all three value names, the type, the polarity and
the `absent` default are correct.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows building an activity feed of what you do and publishing it to your account.**

      ## What it does
      Sets `EnableActivityFeed`, `PublishUserActivities` and `UploadUserActivities` to 0 under
      `Policies\Microsoft\Windows\System`, the three OS Policies group policies behind Activity
      History. The local activity feed stops being written and no activities are published or
      uploaded.

      ## Benefits
      - **No local feed**: Windows stops recording an activity trail for the machine
      - **No upload**: activities cannot be published to your Microsoft or work account
      - **Policy-locked**: the Settings toggles follow the policy and cannot be flipped back

      ## Drawbacks
      - **Small effect on Windows 11**: Timeline was removed and consumer cloud upload of activity
        history was retired, so most of what this blocks is already dormant
      - **Timeline lost on Windows 10**: if you still use Timeline, it stops working
      - **Existing data stays**: activities already stored are not deleted by this tweak

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1803 and later
      - **Takes effect**: immediately, per Microsoft's own help text for all three policies
      - **Reverting**: deletes all three values, restoring the Windows default
      - Deleting stored activities is a separate action in Settings and is not covered here

      ## Recommendation
      Apply it on Windows 11; the cost is essentially nil and it prevents the feature being switched
      back on. On Windows 10, skip it if you actually use Timeline.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.22 Activity History](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - Privacy (UploadUserActivities)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.22 Activity History, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (UploadUserActivities), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\OSPolicy.admx` and `en-US\OSPolicy.adml` on build 26100 (tier A, shipped ADMX and ADML)

### `disable_cloud_clipboard` Cross-device cloud clipboard

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, value `AllowCrossDeviceClipboard`,
`REG_DWORD`. Disabled writes `0`; "Enabled (Stock Default)" is `absent`.

Microsoft documents `AllowCrossDeviceClipboard` with GP friendly name "Allow Clipboard
synchronization across devices", GP path System / OS Policies, ADMX `OSPolicy.admx`, supported values
0 (Not allowed) and 1 (default, Allowed), most restricted value 0. The policy governs whether "an item
copied to the clipboard is uploaded to the cloud so that other devices can access it". Local clipboard
history is a different setting (`AllowClipboardHistory`) and is untouched, which the YAML gets right.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Keeps everything you copy on this machine instead of syncing it to Microsoft's cloud.**

      ## What it does
      Sets `AllowCrossDeviceClipboard` to 0 under `Policies\Microsoft\Windows\System`, the Group Policy
      "Allow Clipboard synchronization across devices". Copied items stop being uploaded for other
      signed-in devices to read.

      ## Benefits
      - **Secrets stay local**: passwords and tokens pasted through the clipboard stop leaving the PC
      - **Win+V still works**: local clipboard history is a separate setting and is unaffected
      - **Machine-wide**: covers every account on the device

      ## Drawbacks
      - **No cross-device paste**: copying on this PC and pasting on another signed-in PC stops working
      - **Phone Link paste**: clipboard sharing with a paired phone stops as well

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later
      - **Takes effect**: immediately
      - **Reverting**: deletes the policy value, restoring the Windows default
      - Local clipboard history (`AllowClipboardHistory`) is deliberately left alone

      ## Recommendation
      Apply it on any machine where you paste credentials, keys or personal data. Leave it alone only
      if you actively paste between multiple Windows devices signed into the same account.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 30 Cloud Clipboard](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - System (AllowCrossDeviceClipboard)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 30 Cloud Clipboard, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - System (AllowCrossDeviceClipboard), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)

### `disable_online_speech_recognition` Online speech recognition

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** `HKCU\Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy`, value
`HasAccepted`, `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `0` |
| On (Stock Default) | **`absent`** (the revert must delete the value) |

**Wrong as authored:** the "On (Stock Default)" option writes `1`. This is the single most damaging
defect in the category. Microsoft documents only the disable direction ("Create a REG_DWORD registry
setting named **HasAccepted** ... with a value of 0 (zero)"), no Microsoft source gives the value a
shipped default of 1, and the CSP text confirms the OS position is that the control is deferred to the
user. The value materialises only when the user makes a consent choice in OOBE or in Settings. On a
machine where online speech recognition was never accepted, reverting this tweak **turns cloud speech
on and fabricates a consent the user never granted**.

**Corrections needed:** see correction 13. If the engine's state model cannot express "restore to
absent" for this effect, it needs a delete-capable revert rather than a value write. See also
correction 14: the machine-wide lock for this same feature, `AllowInputPersonalization`, currently
sits in `disable_inking_typing_personalization` and belongs here.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops your voice being sent to Microsoft's cloud speech service for recognition.**

      ## What it does
      Writes `HasAccepted` as 0 under `HKCU\Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy`,
      the registry backing of Settings > Privacy and security > Speech > Online speech recognition.
      Dictation and voice interaction fall back to on-device recognition.

      ## Benefits
      - **No voice upload**: audio stops being sent to Microsoft Speech services for this account
      - **On-device speech keeps working**: local dictation and voice typing still function
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **Worse dictation**: cloud recognition is more accurate and covers more languages
      - **Per user**: every account on the machine needs it applied separately
      - **Apps that require it**: voice features in apps built on the Microsoft cloud speech stack stop
        responding

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later
      - **Takes effect**: immediately, the Settings toggle reflects it at once
      - **Reverting**: deletes the value so Windows returns to asking you, rather than recording a
        consent you may never have given
      - The machine-wide equivalent is the "Allow users to enable online speech recognition services"
        policy, which locks the choice for every account

      ## Recommendation
      Apply it unless you dictate regularly and value the accuracy. If you share the machine, apply it
      per account or use the machine-wide policy instead.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.6 Speech](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - Privacy (AllowInputPersonalization)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.6 Speech, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (AllowInputPersonalization), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. `_harmful-revert.md`, confirmed documentation-backed harmful revert (project record)

### `disable_inking_typing_personalization` Inking and typing personalization

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** five `REG_DWORD` effects.

| Effect | Key | Value name | Disabled | Stock Default as authored |
|---|---|---|---|---|
| `accepted_privacy` | `HKCU\Software\Microsoft\Personalization\Settings` | `AcceptedPrivacyPolicy` | `0` | `1` |
| `restrict_text` | `HKCU\Software\Microsoft\InputPersonalization` | `RestrictImplicitTextCollection` | `1` | `0` |
| `restrict_ink` | `HKCU\Software\Microsoft\InputPersonalization` | `RestrictImplicitInkCollection` | `1` | `0` |
| `harvest_contacts` | `HKCU\Software\Microsoft\InputPersonalization\TrainedDataStore` | `HarvestContacts` | `0` | `1` |
| `allow_input_policy` | `HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization` | `AllowInputPersonalization` | `0` | `absent` |

The two `RestrictImplicit*Collection` values are documented by Microsoft as the registry equivalent of
turning off "Improve inking and typing", each set to 1 under
`HKEY_CURRENT_USER\Software\Microsoft\InputPersonalization`. Those are correct.
`AcceptedPrivacyPolicy` and `HarvestContacts` are real values used by the same subsystem but are not
Microsoft-documented; they rest on community references.

**Wrong as authored, effect 5.** `AllowInputPersonalization` is **not** an inking control. Policy CSP
`Privacy/AllowInputPersonalization` maps to the Group Policy "Allow users to enable online speech
recognition services", under Control Panel > Regional and Language Options, defined in
**`Globalization.admx`**. The hive and key path (`HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization`)
are correct for that policy, so this is not a hive defect; the effect is in the wrong tweak. Setting it
to 0 silently disables cloud speech recognition machine-wide, which is not what this tweak's name or
description leads a user to expect.

**The genuine inking policy**, if machine-wide inking control is the intent, is
`AllowLinguisticDataCollection` ("Improve inking and typing recognition") from **`TextInput.admx`**, at
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\TextInput`.

**Corrections needed:** see corrections 14 and 15.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows collecting your typing and handwriting to build a personal language model.**

      ## What it does
      Sets `RestrictImplicitTextCollection` and `RestrictImplicitInkCollection` to 1 under
      `HKCU\Software\Microsoft\InputPersonalization`, the documented registry form of turning off
      "Improve inking and typing", and clears the related personalization consent and contact-harvest
      values so the local dictionary stops being fed.

      ## Benefits
      - **No implicit collection**: typed and handwritten text stops being sampled into the model
      - **No contact harvesting**: names from your contacts stop being added to the local dictionary
      - **Per user and reversible**: nothing is deleted, collection simply stops

      ## Drawbacks
      - **Weaker prediction**: autocorrect and text suggestions degrade over time without new samples
      - **Handwriting accuracy**: pen input recognition improves less on your own handwriting
      - **Two values undocumented**: `AcceptedPrivacyPolicy` and `HarvestContacts` are community-sourced,
        not documented by Microsoft
      - **Existing model stays**: what has already been learned is not erased

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 all supported versions
      - **Takes effect**: immediately
      - **Reverting**: restores the previous values from the snapshot
      - The related machine-wide policy in this tweak governs online speech recognition, not inking;
        see the online speech recognition tweak

      ## Recommendation
      Apply it if you type or write anything sensitive and do not lean on Windows text prediction.
      Leave it alone if you use pen input heavily and want recognition to keep improving.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.21 Inking and Typing](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - Privacy (AllowInputPersonalization)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.21 Inking and Typing, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (AllowInputPersonalization), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\Globalization.admx` (policy `AllowInputPersonalization`) and `TextInput.admx` (policy `AllowLinguisticDataCollection`) on build 26100 (tier A, shipped ADMX)

### `disable_advertising_id` Advertising ID

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** three values, one more than the tweak currently writes.

| Effect | Key | Value name | Type | Disabled | Stock Default |
|---|---|---|---|---|---|
| `adv_policy` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo` | `DisabledByGroupPolicy` | `REG_DWORD` | `1` | `absent` |
| **missing** | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo` | `Enabled` | `REG_DWORD` | `0` | `1` |
| `adv_enabled` | `HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo` | `Enabled` | `REG_DWORD` | `0` | `1` |

Microsoft documents exactly two values for turning off the advertising ID, and both are in HKLM:
`Enabled` = 0 under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo`, and
`DisabledByGroupPolicy` = 1 under `HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo`
(Computer Configuration > Administrative Templates > System > User Profiles > "Turn off the
advertising ID"). Policy CSP `Privacy/DisableAdvertisingId` is Device-scoped only.

**The tweak is incomplete, not incorrect.** No Microsoft source states the HKCU value is inert, and the
advertising ID is genuinely a per-user identity, so "the OS ignores HKCU" is not established. Keep the
HKCU write as a per-user supplement and add the documented machine value.

**Corrections needed:** see correction 16.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off the per-user advertising ID that Store apps use to track you across sessions.**

      ## What it does
      Sets `DisabledByGroupPolicy` to 1 under `Policies\Microsoft\Windows\AdvertisingInfo`, the Group
      Policy "Turn off the advertising ID", and writes the companion `Enabled` value as 0 so the
      identifier is no longer issued to apps. Windows resets the existing ID.

      ## Benefits
      - **No stable ad identity**: Store apps lose the identifier used to correlate you across sessions
      - **Policy-locked**: the Settings toggle is greyed out and cannot be turned back on by a user
      - **Applies to new accounts**: the machine-level value covers profiles created later

      ## Drawbacks
      - **Ads remain**: this makes advertising less targeted, it does not block or reduce it
      - **Store app quirks**: a small number of apps that expect an advertising ID behave oddly
      - **Settings locked**: while applied, users cannot change the setting themselves

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 all supported versions
      - **Takes effect**: immediately
      - **Reverting**: removes the policy value and restores the previous per-user value from the snapshot
      - Microsoft documents the companion `Enabled` value under HKLM; the per-user copy is the one the
        Settings toggle writes and both are worth setting

      ## Recommendation
      Apply it on every machine. There is no functional cost and the identifier exists only to serve
      advertisers. Skip it only if you are debugging a Store app that depends on the ID.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.1 General](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - Privacy (DisableAdvertisingId)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.1 General, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (DisableAdvertisingId), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. `_verification.md`, privacy claim 1, PARTLY CONFIRMED (project adjudication record)

### `disable_tailored_experiences` Tailored experiences

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** two separate knobs. They are related but they are not the same control and
must not be conflated.

| Effect | Corrected key | Value name | Type | Off | Stock Default |
|---|---|---|---|---|---|
| `tailored_policy` | **`HKCU\Software\Policies\Microsoft\Windows\CloudContent`** | `DisableTailoredExperiencesWithDiagnosticData` | `REG_DWORD` | `1` | `absent` |
| `tailored_user` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Privacy` | `TailoredExperiencesWithDiagnosticDataEnabled` | `REG_DWORD` | `0` | consent-derived, currently `1` |

**Wrong as authored:** `tailored_policy` is written to
`HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`. The shipped `CloudContent.admx` declares it
`class="User"`:

```xml
<policy name="DisableTailoredExperiencesWithDiagnosticData" class="User"
        key="Software\Policies\Microsoft\Windows\CloudContent"
        valueName="DisableTailoredExperiencesWithDiagnosticData">
```

Policy CSP `Experience/AllowTailoredExperiencesWithDiagnosticData` is User-scoped with Device
explicitly not supported, and its Group Policy mapping gives Location = User Configuration. Because the
policy is `class="User"`, the Group Policy engine will never write or refresh an HKLM copy. Do not
assert the HKLM write is inert (that is not documented); assert that it is in the wrong hive.

The second effect, `TailoredExperiencesWithDiagnosticDataEnabled`, is the Settings UI value under
`CurrentVersion\Privacy`. It is a separate, undocumented control and both can legitimately exist.

**Corrections needed:** see corrections 17 and 18.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows using your diagnostic data to target the tips, ads and recommendations it shows you.**

      ## What it does
      Sets the user-scope `DisableTailoredExperiencesWithDiagnosticData` policy under
      `Software\Policies\Microsoft\Windows\CloudContent` and clears the matching Settings value under
      `CurrentVersion\Privacy`. Windows stops personalising lock screen content, tips and consumer
      feature suggestions using your diagnostic data.

      ## Benefits
      - **Untargeted suggestions**: recommendations stop being shaped by what your device reports
      - **Two layers**: the policy pins it and the Settings value matches, so the UI agrees
      - **No functional loss**: nothing stops working, only the targeting changes

      ## Drawbacks
      - **Suggestions do not stop**: they become generic rather than absent
      - **Depends on Spotlight**: Microsoft documents this policy as requiring Windows Spotlight to be
        allowed; with Spotlight fully disabled it is moot
      - **Per user**: the policy is user-scoped, so each account needs it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later
      - **Takes effect**: immediately
      - **Reverting**: removes the policy value and restores the previous Settings value from the snapshot
      - This is a User Configuration policy; writing it to HKLM does nothing the Group Policy engine
        will refresh

      ## Recommendation
      Apply it on any personal machine. It is a pure personalisation opt-out with no cost. Skip it only
      if you genuinely find the tailored suggestions useful.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Experience (AllowTailoredExperiencesWithDiagnosticData)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
      - [Manage connections from Windows components to Microsoft services, 25 Personalized Experiences](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Policy CSP - Experience (AllowTailoredExperiencesWithDiagnosticData), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100, `class="User"` (tier A, shipped ADMX)
3. `_policy-hive-audit.md`, the only WRONG HIVE finding in `privacy.yaml` (project record)

### `disable_location_tracking` Location tracking

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** three values, one more than the tweak currently writes.

| Effect | Key | Value name | Type | Off | Stock Default |
|---|---|---|---|---|---|
| `location_policy` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors` | `DisableLocation` | `REG_DWORD` | `1` | `absent` |
| **missing** | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` | `LetAppsAccessLocation` | `REG_DWORD` | `2` (Force Deny) | `absent` |
| `location_consent` | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location` | `Value` | `REG_SZ` | `Deny` | `Allow` |

Microsoft documents `DisableLocation` exactly: "Create a REG_DWORD registry setting named
**DisableLocation** in **HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows\LocationAndSensors**
with a value of 1 (one)", equivalent to "Turn off location". Microsoft's documented companion for
blocking apps specifically is `LetAppsAccessLocation` = 2 under `AppPrivacy`, which this tweak does not
write. The ConsentStore `Value` is the device-level consent record read by the Capability Access
Manager, with accepted strings `Allow` and `Deny`; it is real and widely used but is not
Microsoft-documented, so it is tier C.

**Corrections needed:** see correction 19.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off the Windows location platform so neither Windows nor any app can get a position fix.**

      ## What it does
      Sets `DisableLocation` to 1 under `Policies\Microsoft\Windows\LocationAndSensors`, the Group
      Policy "Turn off location", and denies location at the device consent store. The location
      service stops resolving positions and the Settings toggle is greyed out.

      ## Benefits
      - **Hard off**: this is the device-level switch, not a per-app permission
      - **Covers every account**: no per-user configuration needed
      - **Nothing to leak**: apps cannot request a position they can no longer obtain

      ## Drawbacks
      - **Automatic time zone breaks**: it fails silently and your clock can drift after travel
      - **Find My Device stops**: locating a lost machine stops working regardless of its own setting
      - **Weather, Maps and location-aware apps**: all lose their position and fall back to manual entry
      - **Settings locked**: with the policy present a user cannot re-enable location from the UI

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 all supported versions
      - **Takes effect**: immediately, though the Settings page needs reopening to redraw
      - **Reverting**: removes the policy and restores the previous consent value from the snapshot
      - The consent-store half is community-sourced; the policy half is Microsoft-documented

      ## Recommendation
      Apply it on a desktop that never needs a position fix. Do not apply it on a laptop you travel
      with, where automatic time zone and Find My Device are worth more than the privacy gain.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.2 Location](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - Privacy (LetAppsAccessLocation)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.2 Location, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (LetAppsAccessLocation, Force Deny = 2), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Microsoft Q&A, "Location Services Disabled and Grayed out in Windows 11 24H2", https://learn.microsoft.com/en-us/answers/questions/3930072/location-services-disabled-and-grayed-out-in-windo (tier D, cited only for the unresolved report that the consent-store write alone may not flip the UI state on 24H2)

### `disable_app_diagnostics` App diagnostic access

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two effects.

| Effect | Key | Value name | Type | Denied | Stock Default |
|---|---|---|---|---|---|
| `appdiag_policy` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` | `LetAppsGetDiagnosticInfo` | `REG_DWORD` | `2` (Force Deny) | `absent` |
| `appdiag_consent` | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\appDiagnostics` | `Value` | `REG_SZ` | `Deny` | `Allow` |

`LetAppsGetDiagnosticInfo` is Policy CSP `Privacy/LetAppsGetDiagnosticInfo`, "Let Windows apps get
diagnostic information about other apps", with three settings: User is in control (0), Force Allow (1),
Force Deny (2). With Force Deny, "Windows apps aren't allowed to get diagnostic information about other
apps and employees in your organization can't change it". Microsoft notes that an app open when the
policy is applied must be restarted. The ConsentStore write is real and used in practice but is not
Microsoft-documented, so it is tier C.

**Corrections needed:** see correction 20. The scope claim in the current `info` is too broad.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks Store apps from reading diagnostic details about the other apps running on your PC.**

      ## What it does
      Sets `LetAppsGetDiagnosticInfo` to 2 (Force Deny) under `Policies\Microsoft\Windows\AppPrivacy`
      and denies the capability at the device consent store. Packaged Windows apps lose access to the
      process, package and user-name information the App Diagnostics capability exposes.

      ## Benefits
      - **Closes a cross-app channel**: packaged apps can no longer enumerate what else you run
      - **User name protected**: the capability also exposes the account name alongside process details
      - **Locked**: with Force Deny, no user or app can grant the capability back

      ## Drawbacks
      - **Packaged apps only**: Win32 processes are completely unaffected, so Task Manager, Process
        Explorer and any classic tool still see everything
      - **Breaks packaged monitors**: Store-delivered diagnostic, accessibility or monitoring apps lose
        their data
      - **Apps need restarting**: an app already running when the policy applies keeps its old state
        until relaunched

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later
      - **Takes effect**: immediately, but affected apps must be restarted
      - **Reverting**: removes the policy and restores the previous consent value from the snapshot
      - The consent-store half is community-sourced; the policy half is Microsoft-documented

      ## Recommendation
      Apply it on a machine where you install Store apps you do not fully trust. Skip it if you rely on
      a packaged monitoring or accessibility tool that needs this capability.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Privacy (LetAppsGetDiagnosticInfo)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Policy CSP - Privacy (LetAppsGetDiagnosticInfo), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
2. Manage connections from Windows components to Microsoft services, sections 18.2 and 18.15 (the AppPrivacy Force Deny value 2), https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### `disable_find_my_device` Find My Device

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\FindMyDevice`, value `AllowFindMyDevice`,
`REG_DWORD`. Off writes `0`; "On (Stock Default)" is `absent`.

Microsoft documents it exactly: "You can also create a new REG_DWORD registry setting
**HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\FindMyDevice\AllowFindMyDevice** to **0 (zero)**",
equivalent to disabling Computer Configuration > Administrative Templates > Windows Components > Find
My Device > "Turn On/Off Find My Device". With it off, Windows stops the periodic location reporting
that lets you locate the device from your Microsoft account.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows periodically reporting your device's location to your Microsoft account.**

      ## What it does
      Sets `AllowFindMyDevice` to 0 under `Policies\Microsoft\FindMyDevice`, the Group Policy "Turn
      On/Off Find My Device". The recurring location report that powers device tracking from
      account.microsoft.com stops.

      ## Benefits
      - **No location beacon**: a recurring outbound location report tied to your account ends
      - **Machine-wide**: the policy covers every account on the device
      - **Reversible**: deleting the value restores the feature exactly

      ## Drawbacks
      - **Lost device is lost**: you cannot locate or remotely lock a stolen laptop from your account
      - **Security regression**: for a portable machine this costs more than it gains
      - **Redundant with location off**: disabling the location platform already breaks Find My Device

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later
      - **Takes effect**: immediately
      - **Reverting**: deletes the policy value, restoring the Windows default
      - The feature only works with a Microsoft account and a functioning location service in the
        first place

      ## Recommendation
      Apply it on a desktop that never leaves the house. Do not apply it on a laptop or tablet; the
      ability to find or lock a stolen device is worth more than this privacy gain.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 5 Find My Device](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - Experience (AllowFindMyDevice)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 5 Find My Device, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Experience (AllowFindMyDevice), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)

### `disable_settings_sync` Settings sync

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync`, two `REG_DWORD` values.

| Effect | Value name | Disabled | Stock Default |
|---|---|---|---|
| `disable_sync` | `DisableSettingSync` | `2` | `absent` |
| `disable_sync_override` | `DisableSettingSyncUserOverride` | `1` | `absent` |

Microsoft documents this pair verbatim: "Create a REG_DWORD registry setting named
**DisableSettingSync** in **HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows\SettingSync** with a
value of 2 (two) and another named **DisableSettingSyncUserOverride** in the same key with a value of 1
(one)", equivalent to enabling "Do not sync" with the "Allow users to turn syncing on" checkbox left
unchecked. The non-obvious 2 is confirmed: 2 means disabled, not 0.

**Corrections needed:** `none`. `requires_reboot: true` is not documented as required but is a
defensible conservative choice for a policy that affects sign-in-time state.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops your settings, themes and saved passwords being copied to your Microsoft account.**

      ## What it does
      Writes `DisableSettingSync` = 2 and `DisableSettingSyncUserOverride` = 1 under
      `Policies\Microsoft\Windows\SettingSync`, the Group Policy "Do not sync" with the user override
      removed. Settings roaming stops and the user cannot turn it back on.

      ## Benefits
      - **Nothing roams**: settings, themes, language preferences and saved passwords stay on this PC
      - **Cannot be re-enabled**: the override value locks the Settings control
      - **Machine-wide**: covers every account on the device

      ## Drawbacks
      - **No configuration roaming**: a second Windows device no longer inherits your setup
      - **Settings control locked**: users see a greyed-out sync section until you revert
      - **Partial on Windows 11**: the user-facing surface is now Windows Backup, whose scope overlaps
        but is not identical to legacy sync

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 8.1 through Windows 10
      - **Takes effect**: after reboot
      - **Reverting**: deletes both values, restoring sync and user control
      - `DisableSettingSync` = 2 is correct and deliberate; 0 does not mean disabled here

      ## Recommendation
      Apply it on a single-device setup or any machine where saved passwords should not leave the box.
      Skip it if you run several Windows devices and want your configuration to follow you.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 21 Sync your settings](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Shipped `SettingSync.admx` on Windows 11 build 26100
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 21 Sync your settings, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\SettingSync.admx` on build 26100 (tier A, shipped ADMX)

### `disable_error_reporting` Windows Error Reporting

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two `REG_DWORD` values, both named `Disabled`.

| Effect | Key | Disabled | Stock Default |
|---|---|---|---|
| `wer_disabled` | `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `1` | `absent` |
| `wer_policy` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting` | `1` | `absent` |

Microsoft's WER Settings reference states that WER settings live under
`HKEY_CURRENT_USER\Software\Microsoft\Windows\Windows Error Reporting` or the HKLM equivalent, and
documents `Disabled` as `REG_DWORD` with 0 = Enabled (default) and 1 = Disabled. The non-policy write
matches that exactly. The policy-key write corresponds to the Group Policy "Disable Windows Error
Reporting"; the value name and semantics are the same, but the
`Software\Policies\Microsoft\Windows\Windows Error Reporting` path is not confirmed at tier A in any
pass so far.

**Corrections needed:** see correction 21.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops crash and error reports being generated and sent to Microsoft.**

      ## What it does
      Writes `Disabled` = 1 to both the product and the policy Windows Error Reporting keys under HKLM.
      WER stops queueing reports, stops uploading them, and stops writing the user-mode artefacts that
      Reliability Monitor reads.

      ## Benefits
      - **No crash uploads**: WER reports can carry heap contents and file paths, and they stop leaving
      - **Quieter machine**: crash-report prompts and background upload attempts stop
      - **Two layers**: the product value and the policy value are both set

      ## Drawbacks
      - **Local diagnostics lost**: the WER report queue, user-mode `LocalDumps` and Reliability Monitor
        entries all go quiet, which is what support processes usually ask for first
      - **Less useful with telemetry already reduced**: at the Required diagnostic level, crash dumps
        are not sent anyway, so the added privacy is smaller than it looks
      - **Third-party handlers**: some crash handlers query WER state and behave differently

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows Vista through Windows 10
      - **Takes effect**: immediately
      - **Reverting**: deletes both values, restoring the Windows default
      - Kernel crash dumps (`MEMORY.DMP`) are governed by `SYSTEM\CurrentControlSet\Control\CrashControl`
        and are **not** affected by this tweak

      ## Recommendation
      Apply it on a stable machine you do not troubleshoot. Do not apply it while you are chasing
      crashes or instability; you will be throwing away your own first diagnostic.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [WER Settings](https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings)
      - [Configure Windows diagnostic data in your organization (crash dump table by level)](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
```

**Sources:**
1. WER Settings, https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings (tier A)
2. Configure Windows diagnostic data in your organization, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)

### `disable_suggested_content_settings` Suggested content in Settings

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated for the ID mapping, Microsoft-documented
for the surface)

**Corrected mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, now
**four** `REG_DWORD` values, all Off = `0`, all Stock Default = **`absent`**.

| Effect | Value name | Off | Stock Default |
|---|---|---|---|
| `content_338393` | `SubscribedContent-338393Enabled` | `0` | `absent` |
| `content_353694` | `SubscribedContent-353694Enabled` | `0` | `absent` |
| `content_353696` | `SubscribedContent-353696Enabled` | `0` | `absent` |
| `content_353698` (**new**) | `SubscribedContent-353698Enabled` | `0` | `absent` |

**Wrong as authored:** "On (Stock Default)" writes `1` to all three existing values, and the fourth
value is missing.

Four genuinely independent tier C sources, with different authors, countries of origin and eras,
describe the first three IDs as a group as the registry backing of "Show me suggested content in the
Settings app", with 1 = shown and 0 = hidden, agreeing on key path, value names, type and semantics
with no disagreement. Microsoft publishes no `SubscribedContent-NNNNNN` mapping and structurally
cannot: the prefix is a literal inside the shipped `ContentDeliveryManager.Utilities.dll` and
`ContentDeliveryManager.Background.dll` with the numeric ID appended at runtime from the subscription
identifier. That the surface exists and is separately controllable is confirmed at tier A by the shipped
`DisableWindowsSpotlightOnSettings` policy in `CloudContent.admx`.

**The fourth value.** Win11Debloat's `Regfiles/Disable_Windows_Suggestions.reg` writes
`SubscribedContent-353698Enabled` in the same block, under the same "Show me suggested content in the
Settings app" comment, as the three already shipped. The bare id `353698` is present in
`ContentDeliveryManager.Background.dll`, `ContentDeliveryManager.Utilities.dll` and
`Windows.Services.TargetedContent.dll` on 26100. The full value name is never a literal in any binary
for any of these slots, including the known-good ones, so literal absence of the full name is not
evidence against it; presence of the bare id is the right test and it passes. Neither privacy.sexy,
Sophia Script nor WinUtil contains `353698`, so the earlier claim that they corroborate it was wrong.
Support is Win11Debloat plus the shipped binaries, which is adequate for an additive extension whose
failure mode is nil: if the slot were inert, writing 0 to it would do nothing.

**Corrections needed:** see corrections 22 and 23.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the promotional cards Windows shows inside the Settings app.**

      ## What it does
      Sets four `SubscribedContent-*Enabled` values to 0 under the per-user Content Delivery Manager
      key. The Settings app stops requesting and rendering the suggested-content cards shown on the
      Home, System and Personalization pages.

      ## Benefits
      - **Cleaner Settings**: promotional cards and app pitches disappear from Settings pages
      - **Less fetching**: the content subscriptions stop being requested for this account
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **Cosmetic only**: nothing about your privacy posture changes, only what Settings displays
      - **Undocumented IDs**: Microsoft publishes no mapping for these subscription IDs, so the grouping
        rests on independent community sources rather than documentation
      - **Reset by feature updates**: these values are widely reported to return to their default state
        after a major Windows update

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1803 and later
      - **Takes effect**: immediately, reopen Settings to see it
      - **Reverting**: deletes the values, which is the state a fresh profile is in
      - The Microsoft-documented alternative is the `DisableWindowsSpotlightOnSettings` policy, which is
        broader and not what this tweak uses

      ## Recommendation
      Apply it; there is no downside beyond losing advertisements. Skip it only if you want Microsoft's
      Settings suggestions.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [privacy.sexy, "Disable suggested content in Settings app"](https://github.com/undergroundwires/privacy.sexy)
      - [Win11Debloat, Disable_Windows_Suggestions.reg](https://github.com/Raphire/Win11Debloat)
      - [Manage connections from Windows components to Microsoft services, 25 Personalized Experiences](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100: none of the values is present (primary observation, tier A with the caveat in correction 37)
2. Shipped `CloudContent.admx` on build 26100: policy `DisableWindowsSpotlightOnSettings`, class User, key `Software\Policies\Microsoft\Windows\CloudContent` (tier A, shipped ADMX)
3. String evidence on build 26100: the bare id `353698` in `ContentDeliveryManager.Background.dll`, `ContentDeliveryManager.Utilities.dll` and `Windows.Services.TargetedContent.dll` (tier A, product artifact)
4. privacy.sexy, "Disable suggested content in Settings app", writing exactly the three original values as REG_DWORD 0 with `deleteOnRevert` and the annotation "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)", https://github.com/undergroundwires/privacy.sexy (tier C)
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, which writes all four values under the "Show me suggested content in the Settings app" comment, https://github.com/Raphire/Win11Debloat (tier C)
6. Sophia Script for Windows 11, function `SettingsSuggestedContent`, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
7. Brink, TenForums tutorial 100541 and ElevenForum tutorial 3791 (counted as one origin, same author), https://www.elevenforum.com/t/enable-or-disable-suggested-content-in-settings-in-windows-11.3791/ (tier C)
8. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, listing 338393, 353694, 353696 and 353698, https://github.com/Biswa96/WinLight (tier C)
9. Manage connections from Windows components to Microsoft services, section 25 Personalized Experiences, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### `disable_start_app_suggestions` Start menu app suggestions

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated)

**Mechanism as authored:** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`,
value `SystemPaneSuggestionsEnabled`, `REG_DWORD`. Off writes `0`; "On (Stock Default)" writes `1`.

**The literal 1 default is correct here and must not be swept to `absent`.** This value is one of the
few in the key that Windows genuinely seeds: it is present with data 1 in the shipped
`C:\Users\Default\NTUSER.DAT` on build 26100. A string scan of 4051 shipped `System32` and
`SystemApps` modules on 26100 found the value name in `ContentDeliveryManager.Background.dll`, so it is
still read.

**Missing companions.**

| Value | Key | Off | Stock Default | Surface |
|---|---|---|---|---|
| `SubscribedContent-338388Enabled` | same ContentDeliveryManager key | `0` | `absent` | the Windows 10 Start suggestions toggle, which writes both values |
| `Start_IrisRecommendations` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `0` | unknown | the Windows 11 Recommended section |
| `HideRecommendedSection` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` | `1` | `absent` | the Windows 11 Recommended section, tier A, shipped in `StartMenu.admx` |

The Windows 11 wording "Show recommendations for tips, app promotions and more" is a **different**
surface from `SystemPaneSuggestionsEnabled`. Sophia Script, which tracks 25H2 and later, has dropped
both `SystemPaneSuggestionsEnabled` and 338388 and uses `Start_IrisRecommendations` alone.

**Corrections needed:** see correction 24. This tweak also overlaps `disable_start_suggestions` in
`debloat.yaml` and the two are merge candidates.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops promoted apps appearing among your Start menu entries.**

      ## What it does
      Sets `SystemPaneSuggestionsEnabled` to 0 under the per-user Content Delivery Manager key, the
      long-standing registry backing of "Occasionally show suggestions in Start". Content Delivery
      Manager stops injecting promoted app tiles into the Start app list.

      ## Benefits
      - **No promoted apps**: Store pitches stop appearing in the Start app list
      - **Still read on 24H2**: the value is present in the shipped Content Delivery Manager binary, so
        it is a live control and not a legacy leftover
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **Does not clear the Recommended section**: on Windows 11 that panel is a different surface and
        needs its own control
      - **Partial on Windows 10**: the Settings toggle also writes a companion subscription value that
        this tweak does not, so some suggestions can persist
      - **Reset by feature updates**: major updates are widely reported to restore the default

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later
      - **Takes effect**: immediately in most cases; signing out guarantees it
      - **Reverting**: writes the value back to 1, which is genuinely how Windows ships it
      - Unlike the `SubscribedContent-*` values, this one really is seeded by Windows, so its stock
        default is a literal 1 and not value-absent

      ## Recommendation
      Apply it on any machine; there is no cost. If your goal is a clean Windows 11 Start menu, pair it
      with the tweak that hides the Recommended section.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Brink, "Turn On or Off App Suggestions in Start in Windows 10"](https://www.tenforums.com/tutorials/24117-turn-off-app-suggestions-start-windows-10-a.html)
      - [Win11Debloat, Disable_Windows_Suggestions.reg](https://github.com/Raphire/Win11Debloat)
      - [Disassembler0, Win10-Initial-Setup-Script](https://github.com/Disassembler0/Win10-Initial-Setup-Script)
```

**Sources:**
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100: `SystemPaneSuggestionsEnabled` present as `REG_DWORD` 1 (primary observation)
2. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100: `SystemPaneSuggestionsEnabled` in `ContentDeliveryManager.Background.dll`, `Start_IrisRecommendations` in `StartTileData.dll` (tier A, product artifact)
3. Shipped `StartMenu.admx` on build 26100: policy `HideRecommendedSection`, key `Software\Policies\Microsoft\Windows\Explorer` (tier A, shipped ADMX)
4. Brink, "Turn On or Off App Suggestions in Start in Windows 10", TenForums tutorial 24117, which records that `SystemPaneSuggestionsEnabled` "has changed to" `SubscribedContent-338388Enabled`, https://www.tenforums.com/tutorials/24117-turn-off-app-suggestions-start-windows-10-a.html (tier C)
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C)
6. Disassembler0, Win10-Initial-Setup-Script, `DisableAppSuggestions`, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C)
7. hellzerg, Optimizer, `OptimizeHelper.cs`, https://github.com/hellzerg/optimizer (tier C)
8. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C)

### `disable_app_launch_tracking` App launch tracking

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value
`Start_TrackProgs`, `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `0` |
| On (Stock Default) | **`absent`** |

**Wrong as authored:** "On (Stock Default)" writes `1`. `Start_TrackProgs` is absent from every hive
loaded under `HKEY_USERS` on build 26100, including a second, near-pristine profile whose
`Explorer\Advanced` key holds a single value; if Windows seeded the value at profile creation it would
be there. Microsoft's own instruction is to "**Create** a REG_DWORD registry setting named
`Start_TrackProgs`", the phrasing used for values that do not already exist. Windows treats absent and
1 identically for this value, so nothing user-visible breaks, but the revert leaves the profile in a
state it was never in.

**Corrections needed:** see correction 25. The key, value name, type, polarity and `elevation: user`
are all correct.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows recording which applications you launch to rank Start and search results.**

      ## What it does
      Sets `Start_TrackProgs` to 0 under `Explorer\Advanced`, the documented registry form of "Let
      Windows track app launches to improve Start and search results". Windows stops accumulating the
      launch counts behind the Most used list and part of search ranking.

      ## Benefits
      - **No launch history**: the record of which apps you open stops being kept
      - **Microsoft-documented**: this is the exact value Microsoft names for the Settings toggle
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **Most used list degrades**: the Start menu section that lists your frequent apps stops updating
      - **Search ranking**: results are ordered less usefully for your own habits
      - **Local only**: this data never left the machine, so the privacy gain is against local snooping,
        not against Microsoft

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later
      - **Takes effect**: immediately, Explorer reads it on the next Start interaction
      - **Reverting**: deletes the value, which is how a fresh profile ships; Windows treats absent as
        tracking enabled
      - The value name is `Start_TrackProgs`, with that exact casing and underscore

      ## Recommendation
      Apply it if a shared or observed machine makes the local record of what you run worth removing.
      If you like the Most used list, skip it; the data does not leave your PC.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.1 General](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Enumeration of every hive under `HKEY_USERS` on Windows 11 build 26100: the value is absent in all six
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.1 General, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Enumeration of every hive under `HKEY_USERS` on build 26100, `Start_TrackProgs` absent in all six (primary observation; see correction 37 for the provenance caveat)

### `disable_lockscreen_spotlight_ads` Lock screen ads and fun facts

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated)

**Corrected mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, two
`REG_DWORD` values with **different** stock defaults.

| Effect | Value name | Off | Stock Default |
|---|---|---|---|
| `rotating_lock` | `RotatingLockScreenOverlayEnabled` | `0` | `1` (correct as authored, keep it) |
| `content_338387` | `SubscribedContent-338387Enabled` | `0` | **`absent`** |

**Wrong as authored:** the "On (Stock Default)" option writes an explicit 1 to both. In the shipped
`C:\Users\Default\NTUSER.DAT` on build 26100, `RotatingLockScreenOverlayEnabled` is present with data 1
(alongside `RotatingLockScreenEnabled` = 1), while `SubscribedContent-338387Enabled` is absent, in
common with every other `SubscribedContent-*Enabled` value. This needs a split default, not one answer.

`RotatingLockScreenOverlayEnabled` is confirmed against shipped code: a string scan of 4051 `System32`
and `SystemApps` modules on 26100 found the value name in `Windows.UI.Immersive.dll`, the component
that renders the lock screen, and in `ContentDeliveryManager.Utilities.dll`. Four independent tier C
sources identify it as the "Get fun facts, tips, tricks and more on your lock screen" option and pair
it with 338387 as the lock-screen tip and ad subscription, agreeing on key, both value names, type and
polarity.

**Corrections needed:** see correction 26.

**Ready-to-paste info block:**

```yaml
    info: |
      **Keeps the Windows Spotlight lock screen pictures but strips the ads, tips and "fun facts" text off them.**

      ## What it does
      Sets `RotatingLockScreenOverlayEnabled` and `SubscribedContent-338387Enabled` to 0 under the
      per-user Content Delivery Manager key. The Spotlight wallpaper rotation continues; the overlaid
      promotional text and tip cards stop being fetched and drawn.

      ## Benefits
      - **Wallpapers kept**: unlike the Spotlight policy, this leaves the daily images working
      - **No lock screen ads**: promoted apps and offers stop appearing over the picture
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **You lose the trivia**: the "fun facts" and photo-location captions go with the ads
      - **Undocumented ID**: the 338387 subscription slot has no Microsoft mapping and rests on
        independent community sources
      - **Reset by feature updates**: these values are widely reported to return after a major update

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later
      - **Takes effect**: at the next lock screen
      - **Reverting**: restores the overlay value to 1 and deletes the subscription value, which is how
        a fresh profile ships
      - The Microsoft-documented alternatives (`DisableWindowsSpotlightFeatures`,
        `DisableCloudOptimizedContent`) also kill the wallpapers, which is why this tweak does not use them

      ## Recommendation
      Apply it if you like the Spotlight photographs and not the advertising over them. If you do not
      want Spotlight at all, use the broader Spotlight policy instead.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Brink, "Enable or Disable Facts, Tips, and Tricks on Lock Screen in Windows 11"](https://www.elevenforum.com/t/enable-or-disable-facts-tips-and-tricks-on-lock-screen-in-windows-11.7079/)
      - [Win11Debloat, Disable_Lockscreen_Tips.reg](https://github.com/Raphire/Win11Debloat)
      - [Manage connections from Windows components to Microsoft services, 25 Personalized Experiences](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100: `RotatingLockScreenOverlayEnabled` present as `REG_DWORD` 1, `SubscribedContent-338387Enabled` absent (primary observation)
2. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100: `RotatingLockScreenOverlayEnabled` in `Windows.UI.Immersive.dll` and `ContentDeliveryManager.Utilities.dll` (tier A, product artifact)
3. Brink, ElevenForum tutorial 7079, whose .reg files write both values together and which states "check (on - default)", https://www.elevenforum.com/t/enable-or-disable-facts-tips-and-tricks-on-lock-screen-in-windows-11.7079/ (tier C)
4. Win11Debloat (Raphire), `Regfiles/Disable_Lockscreen_Tips.reg` and its undo file, https://github.com/Raphire/Win11Debloat (tier C)
5. hellzerg, Optimizer, `OptimizeHelper.cs` (independent origin), https://github.com/hellzerg/optimizer (tier C)
6. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C)
7. Disassembler0, Win10-Initial-Setup-Script (2016 era, independent origin), https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C)

### `disable_tips_and_suggestions` Windows tips and suggestions

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** three effects.

| Effect | Key | Value name | Type | Off | Stock Default |
|---|---|---|---|---|---|
| `soft_landing_policy` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent` | `DisableSoftLanding` | `REG_DWORD` | `1` | `absent` |
| `soft_landing_pref` | `HKCU\...\ContentDeliveryManager` | `SoftLandingEnabled` | `REG_DWORD` | `0` | `1` as authored |
| `content_338389` | `HKCU\...\ContentDeliveryManager` | `SubscribedContent-338389Enabled` | `REG_DWORD` | `0` | `1` as authored |

The policy half is fully confirmed at tier A. Policy CSP `Experience/AllowWindowsTips` maps to Group
Policy Name `DisableSoftLanding`, Friendly Name "Do not show Windows tips", Location Computer
Configuration, Registry Key Name `Software\Policies\Microsoft\Windows\CloudContent`, ADMX
`CloudContent.admx`. Default Value 1 (tips enabled), 0 disables. It has a documented dependency on
`AllowWindowsSpotlight` = 1. The two per-user values are the Content Delivery Manager backing of "Get
tips, tricks, and suggestions as you use Windows" and are community-sourced only.

**The applicability is the correction that matters.** The Policy CSP editions row for
`AllowWindowsTips` reads **not supported on Pro**; supported on Enterprise, Education and IoT Enterprise
/ IoT Enterprise LTSC. Home is not listed at all. On Home and Pro the `DisableSoftLanding` write is
silently ignored and only the two per-user values do anything.

**Corrections needed:** see correction 27. Note also that `SubscribedContent-338389Enabled` is a
`SubscribedContent-*` value and so is very likely absent on a fresh profile; its stock default carries
the same doubt recorded in correction 22 and should be checked against a clean image before the
literal 1 is trusted.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the tip, trick and suggestion notifications Windows pops while you work.**

      ## What it does
      Sets the `DisableSoftLanding` policy under `Policies\Microsoft\Windows\CloudContent` and clears
      the two per-user Content Delivery Manager values behind "Get tips, tricks, and suggestions as you
      use Windows". The tip toasts stop appearing.

      ## Benefits
      - **No tip toasts**: the "Did you know" and "Try this" notifications stop
      - **Two layers**: on supported editions the policy pins it, and the per-user values cover the rest
      - **No functional loss**: nothing depends on the tips

      ## Drawbacks
      - **Policy inert on Home and Pro**: Microsoft documents `DisableSoftLanding` as unsupported on Pro
        and does not list Home, so on the two most common editions only the per-user values do anything
      - **Genuinely useful tips too**: occasional pointers to new features go with the promotions
      - **Reset by feature updates**: the per-user values are widely reported to return after a major update

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later; the policy half is
        honored only on Enterprise, Education and IoT Enterprise / IoT Enterprise LTSC
      - **Takes effect**: immediately
      - **Reverting**: removes the policy value and restores the previous per-user values from the snapshot
      - Microsoft documents this policy as depending on Windows Spotlight being allowed

      ## Recommendation
      Apply it if the notifications distract you. On Home or Pro, expect only the per-user half to work,
      which is still enough to stop the toasts for your account.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Experience (AllowWindowsTips)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
      - [AskVG, registry tweaks to disable ads, suggestions and tips](https://www.askvg.com/registry-tweaks-to-disable-ads-suggestions-and-tips-in-windows-10/)
```

**Sources:**
1. Policy CSP - Experience (AllowWindowsTips), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100, policy `DisableSoftLanding`, `class="Machine"` (tier A, shipped ADMX)
3. AskVG, "Registry Tweaks to Disable Ads, Suggestions and Tips in Windows 10", https://www.askvg.com/registry-tweaks-to-disable-ads-suggestions-and-tips-in-windows-10/ (tier C)

### `disable_consumer_features` Windows consumer features

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect. `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value
`DisableWindowsConsumerFeatures`, `REG_DWORD`. Disabled writes `1`; "Enabled (Stock Default)" is
`absent`.

Policy CSP `Experience/AllowWindowsConsumerFeatures` maps to Group Policy Name
`DisableWindowsConsumerFeatures`, Friendly Name "Turn off Microsoft consumer experiences", Location
Computer Configuration, Registry Key Name `Software\Policies\Microsoft\Windows\CloudContent`, ADMX
`CloudContent.admx`. Microsoft describes the scope as "experiences that are typically for consumers
only, such as Start suggestions, Membership notifications, Post-OOBE app install and redirect tiles".
Default Value 1 (allowed), most restricted 0. It has a documented dependency on `AllowWindowsSpotlight`
= 1. Editions: not supported on Pro, and Home is not listed; supported on Enterprise, Education, IoT
Enterprise and IoT Enterprise LTSC.

**The `info` describes an effect that is not in the effects list.** The current text states twice that
the tweak sets the per-user `SilentInstalledAppsEnabled` to 0 and builds its Home/Pro recommendation on
that. There is exactly one effect and it is the Enterprise-only policy.

**Corrections needed:** see correction 28. The block below takes the "state it plainly" route; if the
`SilentInstalledAppsEnabled` effect is added instead, the drawbacks must be rewritten again.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off Microsoft consumer experiences: post-setup app installs, promoted tiles and membership nags.**

      ## What it does
      Sets `DisableWindowsConsumerFeatures` to 1 under `Policies\Microsoft\Windows\CloudContent`, the
      Group Policy "Turn off Microsoft consumer experiences". On editions that honor it, Windows stops
      silently installing promoted apps and stops showing consumer suggestion surfaces.

      ## Benefits
      - **No silent app installs**: the post-OOBE promoted app installs stop
      - **Fewer nags**: membership notifications and redirect tiles stop appearing
      - **Durable**: a policy value outlasts the per-user content settings a feature update can reset

      ## Drawbacks
      - **Inert on Home and Pro**: Microsoft documents the policy as unsupported on Pro and does not
        list Home, so on those editions this tweak does nothing at all
      - **Useful suggestions go too**: on supported editions, legitimate Store recommendations stop
      - **Depends on Spotlight**: Microsoft documents this policy as requiring Windows Spotlight allowed

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later, on Enterprise,
        Education, IoT Enterprise and IoT Enterprise LTSC only
      - **Takes effect**: after reboot
      - **Reverting**: deletes the policy value, restoring the Windows default
      - Prior to Windows 10 1803 this policy had User scope; on current builds it is Device scope

      ## Recommendation
      Apply it on Enterprise, Education or IoT. On Home or Pro it will not do anything, so use the
      per-user Start and suggested-content tweaks instead.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Experience (AllowWindowsConsumerFeatures)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
      - [Manage connections from Windows components to Microsoft services, 25 Personalized Experiences](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Policy CSP - Experience (AllowWindowsConsumerFeatures), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100, `class="Machine"` (tier A, shipped ADMX)

### `disable_edge_telemetry` Microsoft Edge telemetry

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** two live values under `HKLM\SOFTWARE\Policies\Microsoft\Edge`, both
`REG_DWORD`, Off = `0`, Stock Default = `absent`.

| Effect | Value name | Status |
|---|---|---|
| `edge_personalization` | `PersonalizationReportingEnabled` | live, Edge 80 and later |
| `edge_3pserp` | `Edge3PSerpTelemetryEnabled` | live, Edge 120 and later, dynamic policy refresh |

**Wrong as authored, two dead effects.** Both carry Microsoft's banner "OBSOLETE: This policy is
obsolete and doesn't work after Microsoft Edge version 88", supported versions Windows 77 to 88:

| Effect | Value name | Status |
|---|---|---|
| `edge_metrics` | `MetricsReportingEnabled` | OBSOLETE, dead after Edge 88 |
| `edge_siteinfo` | `SendSiteInfoToImproveServices` | OBSOLETE, dead after Edge 88 |

The registry paths and value names are correct; the policies simply are not read by any Edge shipping
today. Both pages still list `MSEdge.admx`, which is why they still appear in policy tooling and in
tweak scripts. They will show as "applied" to any registry-presence probe while doing nothing.

`PersonalizationReportingEnabled` = 0 prevents Microsoft collecting browsing history, favourites,
collections and usage for personalising ads, search, news and Edge, and users cannot override it;
Microsoft notes it "isn't available for child accounts or enterprise accounts".
`Edge3PSerpTelemetryEnabled` = 0 stops Edge capturing searches performed on third-party search
providers.

**Corrections needed:** see corrections 29 and 30. On the replacement question: `DiagnosticData` is the
successor to `MetricsReportingEnabled`, but Microsoft scopes it to Windows 7, Windows 8 and macOS, so
on Windows 10 and 11 there is no Edge-side equivalent left and the Windows `AllowTelemetry` tweak is
the correct lever.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Edge using your browsing for personalization and stops it reporting your third-party searches.**

      ## What it does
      Sets `PersonalizationReportingEnabled` and `Edge3PSerpTelemetryEnabled` to 0 under
      `Policies\Microsoft\Edge`. Edge stops sending browsing history, favourites, collections and usage
      for ad, search and news personalization, and stops capturing searches you run on Google or other
      third-party engines.

      ## Benefits
      - **No browsing profile**: history and favourites stop feeding Microsoft's personalization
      - **Third-party searches private**: a surface no Windows-level telemetry tweak covers
      - **Users cannot override**: both are policies, so the browser settings follow them

      ## Drawbacks
      - **Only if Edge is installed**: on a machine without Edge the whole tweak is inert
      - **Less relevant Edge content**: news feed, search suggestions and offers get less useful
      - **Not a full Edge telemetry off switch**: Edge's diagnostic level on Windows follows the Windows
        diagnostic data setting, not an Edge policy

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; needs Edge 80 or later, and Edge 120 or later for the
        third-party search value
      - **Takes effect**: immediately for the search value; restart Edge for the personalization value
      - **Reverting**: deletes both values, restoring Edge's defaults
      - Two older Edge policies often bundled with this (`MetricsReportingEnabled`,
        `SendSiteInfoToImproveServices`) are marked obsolete by Microsoft and do nothing after Edge 88,
        so they are deliberately not written here

      ## Recommendation
      Apply it if you use Edge at all, even occasionally. If Edge is not your browser and you have
      removed it, skip it; there is nothing to configure.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft Edge policy: PersonalizationReportingEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/personalizationreportingenabled)
      - [Microsoft Edge policy: Edge3PSerpTelemetryEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/edge3pserptelemetryenabled)
      - [Microsoft Edge policy: MetricsReportingEnabled (marked obsolete)](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/metricsreportingenabled)
```

**Sources:**
1. Microsoft Edge policy: PersonalizationReportingEnabled, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/personalizationreportingenabled (tier A)
2. Microsoft Edge policy: Edge3PSerpTelemetryEnabled, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/edge3pserptelemetryenabled (tier A)
3. Microsoft Edge policy: MetricsReportingEnabled, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/metricsreportingenabled (tier A, carries the OBSOLETE banner)
4. Microsoft Edge policy: SendSiteInfoToImproveServices, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/sendsiteinfotoimproveservices (tier A, carries the OBSOLETE banner)

### `disable_language_list_access` Website access to language list

**Verdict:** VERIFIED-WITH-CORRECTION

**Corrected mechanism:** `HKCU\Control Panel\International\User Profile`, value
`HttpAcceptLanguageOptOut`, `REG_DWORD`.

| Option | Value |
|---|---|
| Blocked | `1` |
| Allowed (Stock Default) | **`absent`** |

**Wrong as authored:** "Allowed (Stock Default)" writes `0`. Microsoft's instruction is to "Create a
**new** REG_DWORD registry setting named **HttpAcceptLanguageOptOut** in **HKEY_CURRENT_USER\Control
Panel\International\User Profile** with a value of 1", which means the value does not exist by default.
Behaviourally 0 and absent are equivalent here, so the risk is cosmetic, but a default must be the real
default.

**Corrections needed:** see correction 31.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows handing your full language list to web content, trimming one browser fingerprinting signal.**

      ## What it does
      Writes `HttpAcceptLanguageOptOut` = 1 under `HKCU\Control Panel\International\User Profile`, the
      documented registry form of turning off "Let websites provide locally relevant content by
      accessing my language list". Windows stops exposing the configured list, which in practice trims
      the `Accept-Language` header it contributes.

      ## Benefits
      - **Less entropy**: one identifying signal is removed from the fingerprinting surface
      - **Zero cost**: nothing on the machine depends on the list being shared
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **Narrower than it sounds**: modern browsers set `Accept-Language` themselves, so this mostly
        affects Windows-integrated web content and legacy paths
      - **Auto language selection**: some multilingual sites stop picking your preferred language
      - **Small gain**: one signal among many; it will not defeat fingerprinting on its own

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later
      - **Takes effect**: immediately; browsers pick it up on next launch
      - **Reverting**: deletes the value, which is the state a fresh profile is in
      - Microsoft's own wording is "create a new REG_DWORD", which is why the stock state is
        value-absent rather than a written 0

      ## Recommendation
      Apply it; the cost is one convenience feature that most people never notice. Skip it if you rely
      on sites auto-selecting a non-default language.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 18.1 General](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [privacy.sexy, which writes the same `HttpAcceptLanguageOptOut` value](https://github.com/undergroundwires/privacy.sexy)
```

**Sources:**
1. Manage connections from Windows components to Microsoft services, section 18.1 General, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. privacy.sexy, which writes the same value at the same key, https://github.com/undergroundwires/privacy.sexy (tier C, corroboration only)

### `disable_explorer_cloud_recommendations` File Explorer cloud recommendations

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `DisableGraphRecentItems`,
`REG_DWORD`. Disabled writes `1`; "Enabled (Stock Default)" is `absent`.

The shipped `Explorer.admx` on 26100:

```xml
<policy name="DisableGraphRecentItems" class="Machine"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="DisableGraphRecentItems">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2_NOSERVER" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

`class="Machine"` confirms HKLM, and `enabledValue` 1 confirms that 1 means off. The value name is read
by four shipped binaries: `System32\shell32.dll`, `System32\windows.storage.dll`,
`SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\MicrosoftGraphRecentItemsManager.dll` and
`SystemApps\MicrosoftWindows.Client.FileExp_cw5n1h2txyewy\FileExplorerExtensions.dll`. The third is
named for exactly the subsystem the policy governs.

**This is a network control, not a display toggle.** The 26100 ADML: "Turning off this setting will
prevent File Explorer from requesting cloud file metadata and displaying it in the homepage and other
views in File Explorer. Any insights and files available based on account activity will be stopped in
views such as Recent, Recommended, Favorites, Details pane, etc."

Not a duplicate: `disable_recent_files` writes `ShowRecent` / `ShowFrequent` under `Explorer\Advanced`
(the local MRU) and `remove_home_nav_pane` hides the Home node. Neither stops the Graph request.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops File Explorer calling Microsoft's cloud for file recommendations and account-based insights.**

      ## What it does
      Sets `DisableGraphRecentItems` to 1 under `Policies\Microsoft\Windows\Explorer`. File Explorer
      stops requesting cloud file metadata from Microsoft Graph, so Recommended, Recent, Favorites and
      the details pane no longer show items sourced from your account activity.

      ## Benefits
      - **Stops the request, not just the display**: no Graph call is made at all
      - **Cleaner Home**: the Recommended strip stops showing OneDrive and SharePoint suggestions
      - **Machine-wide**: one policy covers every account on the device

      ## Drawbacks
      - **No cloud recommendations**: if you work from OneDrive or SharePoint, the shortcuts to recently
        touched cloud files disappear
      - **Details pane loses insights**: activity information about a selected cloud file stops appearing
      - **Windows 11 only**: the policy does not exist on Windows 10 or LTSC 2021, where it is inert

      ## Good to know
      - **Applies to**: Windows 11 22H2 and newer, client editions only
      - **Takes effect**: immediately; restart File Explorer to redraw Home
      - **Reverting**: deletes the policy value, restoring the Windows default
      - Local recent files are a separate setting and are not affected by this tweak

      ## Recommendation
      Apply it on a machine that does not use OneDrive or SharePoint for daily work, where the cloud
      calls buy you nothing. Skip it if Explorer Home's Recommended list is part of your workflow.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Shipped `Explorer.admx` and `en-US\Explorer.adml` on Windows 11 build 26100
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\Explorer.admx` and `en-US\Explorer.adml` on build 26100 (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: `DisableGraphRecentItems` present in `shell32.dll`, `windows.storage.dll`, `MicrosoftGraphRecentItemsManager.dll` and `FileExplorerExtensions.dll` (tier A, product artifact)
3. Manage connections from Windows components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A, context for the Explorer cloud-content surface)

### `disable_app_device_inventory` App and device inventory collectors

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat`, four `REG_DWORD` values, each
Disabled = `1`, each Stock Default = `absent`.

| Value name | Collector | Read by |
|---|---|---|
| `DisableInstallTracing` | Install Tracing, which tracks application installs | `System32\installmon.dll` |
| `DisableAPISamping` | API Sampling, sampled collection of APIs used at runtime | `System32\apisampling.dll` |
| `DisableApplicationFootprint` | sampled collection of registry and file usage | `System32\appfootprint.dll`, `System32\pcasvc.dll` |
| `DisableWin32AppBackup` | the compatibility scan over backed-up applications | `System32\aeinv.dll`, `System32\aemarebackup.dll`, `System32\appraiser.dll` |

All four are in the shipped `AppDeviceInventory.admx` on 26100, each `class="Machine"`, each on
`Software\Policies\Microsoft\Windows\AppCompat`, each
`supportedOn ref="windows:SUPPORTED_Windows_11_0_24H2"`, each with `enabledValue` 1 and `disabledValue`
0. Microsoft left the intent comments in the file:

```xml
<policy name="TurnOffAPISamping" class="Machine"
        key="Software\Policies\Microsoft\Windows\AppCompat" valueName="DisableAPISamping">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_24H2" />
  <!-- "Enabled" here means we are turning off API Sampling. -->
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

**The misspelling is genuine.** The value name is `DisableAPISamping`. A search for the correct
spelling `DisableAPISampling` across every ADMX on 26100 returns nothing. Do not "fix" it. Equally, do
not write the ADMX `name=` attributes (`TurnOffInstallTracing`, `TurnOffAPISamping`,
`TurnOffApplicationFootprint`, `TurnOffWin32AppBackup`) into the registry; those are policy names.

Not a duplicate of `disable_compat_appraiser`, which writes `AITEnable` and `DisableInventory` under
the same key. Those are the legacy Application Impact Telemetry and inventory switches; these four are
a distinct 24H2-era collection family with their own "App and Device Inventory" policy category.

**Corrections needed:** `none`. See correction 34 for the two spelling traps that must survive into the
YAML.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off the four app and device inventory collectors Microsoft added in Windows 11 24H2.**

      ## What it does
      Sets `DisableInstallTracing`, `DisableAPISamping`, `DisableApplicationFootprint` and
      `DisableWin32AppBackup` to 1 under `Policies\Microsoft\Windows\AppCompat`. Install tracing, API
      sampling, registry and file usage sampling, and the compatibility scan over backed-up apps all
      stop running.

      ## Benefits
      - **Four collectors off**: each one is a separate sampling agent with its own shipped binary
      - **Newest surface**: this family is new in 24H2 and is not covered by older telemetry tweaks
      - **Policy-backed**: all four are shipped Group Policy settings, not undocumented keys

      ## Drawbacks
      - **App restore scan**: `DisableWin32AppBackup` turns off the compatibility scan that runs when
        restoring applications from Windows Backup
      - **Fewer compatibility diagnostics**: if you hit an app compatibility problem, Microsoft has less
        to work with
      - **24H2 and newer only**: on Windows 10 or LTSC 2021 the values are written but nothing reads them

      ## Good to know
      - **Applies to**: Windows 11 24H2 (build 26100) and newer only
      - **Takes effect**: immediately
      - **Reverting**: deletes all four values, restoring the Windows default
      - The value really is spelled `DisableAPISamping`; the misspelling is Microsoft's and correcting
        it produces a value nothing reads

      ## Recommendation
      Apply it on any 24H2 or newer machine you keep on its current apps. Skip it, or at least skip the
      backup value, if you plan to restore your applications from Windows Backup on a new PC.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Configure Windows diagnostic data in your organization](https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization)
      - Shipped `AppDeviceInventory.admx` and `en-US\AppDeviceInventory.adml` on Windows 11 build 26100
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\AppDeviceInventory.admx` and `en-US\AppDeviceInventory.adml` on build 26100 (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: each value name present in its own named collector binary, `installmon.dll`, `apisampling.dll`, `appfootprint.dll` / `pcasvc.dll`, `aeinv.dll` / `aemarebackup.dll` / `appraiser.dll` (tier A, product artifact)
3. Configure Windows diagnostic data in your organization, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A, context for the appraiser and inventory family)

### `disable_search_history` Device search history

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision, community-corroborated)

**Corrected mechanism:** one value only.

| Key | Value name | Type | Disabled | Stock Default |
|---|---|---|---|---|
| `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings` | `IsDeviceSearchHistoryEnabled` | `REG_DWORD` | `0` | **`absent`**, the revert deletes the value |

**Wrong as originally proposed, and it must not be shipped:** a second effect writing
`DisableSearchHistory` = 1 to `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`. Drop it entirely.
It fails three independent ways:

1. **Wrong hive.** The shipped 26100 `Search.admx` declares
   `<policy name="DisableSearchHistory" class="User" key="SOFTWARE\Policies\Microsoft\Windows\Explorer" valueName="DisableSearchHistory">`.
   `class="User"` means HKCU. privacy.sexy writes HKLM, which is where the proposal inherited the
   error; privacy.sexy's own cited reference is an admx.help mirror of the Windows 10 2016 ADMX.
2. **Declared Windows 8 only.** The same policy element carries `<supportedOn ref="Win8Only" />` and
   sits in the legacy block of `Search.admx` alongside `AlwaysUseAutoLangDetection` and
   `ConnectedSearchUseWeb`, not in the `windows:SUPPORTED_Windows_10_0` block.
3. **No Windows 11 consumer.** The literal `DisableSearchHistory` appears in exactly one shipped
   binary, `SHCore.dll`, the shell policy table, and in no search component.

By contrast `IsDeviceSearchHistoryEnabled` appears in `SearchUx.UI.dll` (the 26100 taskbar search UI,
in the view-model block next to `IsCloudSearchEnabledForMSA` and `IsCloudSearchEnabledForAAD`),
`windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll`.

**The revert must delete.** privacy.sexy annotates this exact value with
`deleteOnRevert: 'true' # Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)`,
from its default-state scan of real Pro images. Win11Debloat's `Disable_Search_History.reg` writes only
`"IsDeviceSearchHistoryEnabled"=dword:00000000` with no revert file, which is consistent.

**Corrections needed:** see correction 32.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows keeping a history of what you have searched for on this device.**

      ## What it does
      Sets `IsDeviceSearchHistoryEnabled` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`, the value the 26100 taskbar
      search UI reads. Windows stops recording your search terms and stops replaying them as recent
      searches.

      ## Benefits
      - **No search trail**: past queries stop being stored for this account
      - **Cleaner search box**: the recent-searches list stops appearing when you open search
      - **No admin needed**: it is a per-user setting

      ## Drawbacks
      - **No recent searches**: repeating a previous query means typing it again
      - **Local only**: this is a local history, so the gain is against someone using your PC, not
        against Microsoft
      - **Not Microsoft-documented**: the value is confirmed inside the shipped search UI binary and by
        independent tools, but Microsoft publishes no page for it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1903 and later
      - **Takes effect**: immediately, reopen search to see it
      - **Reverting**: deletes the value, which is the state a fresh profile is in
      - The old `DisableSearchHistory` group policy is a Windows 8 era setting that no Windows 11 search
        component reads, so it is deliberately not written here

      ## Recommendation
      Apply it on a shared or observed machine, or if a search history you never asked for bothers you.
      Skip it if you re-run the same searches often and value the shortcut.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [privacy.sexy, device search history](https://github.com/undergroundwires/privacy.sexy)
      - [Win11Debloat, Disable_Search_History.reg](https://github.com/Raphire/Win11Debloat)
      - Shipped `SearchUx.UI.dll` on Windows 11 build 26100 contains the value name
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\Search.admx` on build 26100, showing `DisableSearchHistory` as `class="User"` with `supportedOn ref="Win8Only"` (tier A, shipped ADMX; this is the evidence for dropping the policy half)
2. String evidence on build 26100: `IsDeviceSearchHistoryEnabled` in `SearchUx.UI.dll`, `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll`; `DisableSearchHistory` only in `SHCore.dll` (tier A, product artifact)
3. privacy.sexy, which writes the value with `deleteOnRevert` and the annotation "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)", https://github.com/undergroundwires/privacy.sexy (tier C)
4. Win11Debloat (Raphire), `Regfiles/Disable_Search_History.reg`, https://github.com/Raphire/Win11Debloat (tier C)

### `disable_cloud_content_search` Cloud content in search

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Corrected mechanism:** three `REG_DWORD` values.

| Key | Value name | Disabled | Stock Default |
|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` | `AllowCloudSearch` | `0` | `absent` |
| `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\SearchSettings` | `IsMSACloudSearchEnabled` | `0` | **`absent`** |
| `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\SearchSettings` | `IsAADCloudSearchEnabled` | `0` | **`absent`** |

Keys, value names and types are all correct as proposed. **Wrong as originally proposed:** the two
HKCU values were described as "normally present", so the revert would write 1. privacy.sexy writes both
with `deleteOnRevert: 'true' # Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro
(>= 23H2)`. The revert must delete.

The shipped 26100 `Search.admx` defines
`<policy name="AllowCloudSearch" class="Machine" key="SOFTWARE\Policies\Microsoft\Windows\Windows Search" valueName="AllowCloudSearch">`
with `supportedOn="windows:SUPPORTED_Windows_10_0"` and an **enum of three values**: 0 = Disable Cloud
Search, 1 = Enable Cloud Search, 2 = User Selected. The ADML reads "Allow search and Cortana to search
cloud sources like OneDrive and SharePoint." Writing 0 is correct; the third value 2 exists and a
faithful control would offer it.

The string `AllowCloudSearch` is present in `Windows.Storage.Search.dll`, `windowsudk.shellcommon.dll`
and `Windows.FileExplorer.Common.dll` on 26100, so it is live despite the Cortana-era ADML wording.
`IsMSACloudSearchEnabled` and `IsAADCloudSearchEnabled` are present in `windowsudk.shellcommon.dll` and
`AppXDeploymentExtensions.desktop.dll`. Duplicate check passes: the corpus touches `SearchSettings`
once, for `IsDynamicSearchBoxEnabled` in `interface.yaml`, a different value.

**Corrections needed:** see correction 33.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops taskbar search querying your OneDrive, SharePoint and Outlook content in the cloud.**

      ## What it does
      Sets the `AllowCloudSearch` policy to 0 under `Policies\Microsoft\Windows\Windows Search` and
      clears the two per-user cloud search values under `SearchSettings`. Search stops sending your
      typed query to Microsoft to match against account content and returns local results only.

      ## Benefits
      - **Queries stay local**: what you type in the search box stops being sent to Microsoft
      - **Both account types**: personal Microsoft accounts and work or school accounts are both covered
      - **Faster, quieter search**: no network round trip on each keystroke-driven query

      ## Drawbacks
      - **No cloud file results**: OneDrive, SharePoint and Outlook items stop appearing in taskbar search
      - **Work machines**: on a managed device this can remove a feature colleagues rely on
      - **Only two of three states offered**: `AllowCloudSearch` also has a "User Selected" value (2)
        that this tweak does not expose

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later for the policy,
        1903 and later for the per-user values
      - **Takes effect**: immediately
      - **Reverting**: deletes all three values, which is the state a fresh install and profile are in
      - Local file and app search is completely unaffected

      ## Recommendation
      Apply it on a personal machine where taskbar search is for launching apps and finding local files.
      Skip it if you deliberately search your OneDrive or work content from the taskbar.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Search (AllowCloudSearch)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search)
      - [privacy.sexy, personal cloud content search](https://github.com/undergroundwires/privacy.sexy)
      - Shipped `Search.admx` on Windows 11 build 26100
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\Search.admx` on build 26100, policy `AllowCloudSearch`, `class="Machine"`, three-value enum (tier A, shipped ADMX)
2. Policy CSP - Search (AllowCloudSearch), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search (tier A)
3. String evidence on build 26100: `AllowCloudSearch` in `Windows.Storage.Search.dll`, `windowsudk.shellcommon.dll`, `Windows.FileExplorer.Common.dll`; the two HKCU value names in `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll` (tier A, product artifact)
4. privacy.sexy, "Disable personal cloud content search in taskbar", with `deleteOnRevert` on both HKCU values, https://github.com/undergroundwires/privacy.sexy (tier C, cited for the stock-default correction)

### `disable_voice_activation` Voice activation and wake words

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`, two `REG_DWORD` values.

| Value name | Blocked | Stock Default |
|---|---|---|
| `LetAppsActivateWithVoice` | `2` (Force Deny) | `absent` |
| `LetAppsActivateWithVoiceAboveLock` | `2` (Force Deny) | `absent` |

Enum for both: 0 = User is in control, 1 = Force Allow, 2 = Force Deny.

The shipped 26100 `AppPrivacy.admx` defines both as `class="Machine"` at
`key="Software\Policies\Microsoft\Windows\AppPrivacy"`, `supportedOn="windows:SUPPORTED_Windows_10_0"`,
each with the three-value enum above. The ADML for both quotes the Force Deny behaviour explicitly,
including the above-lock variant. Force Deny = 2 is right and the polarity is not inverted. The value
names are present in `agentactivationruntimewindows.dll` (the voice-activation runtime), `AarSvc.dll`
and `SettingsHandlers_SpeechPrivacy.dll` on 26100.

Optional per-user siblings, also real and present in
`agentactivationruntimewindows.dll` and `SettingsHandlers_SpeechPrivacy.dll`:
`AgentActivationEnabled` and `AgentActivationOnLockScreenEnabled` under
`HKCU\Software\Microsoft\Speech_OneCore\Settings\VoiceActivation\UserPreferenceForAllApps`.

Duplicate check passes: no `LetAppsActivateWithVoice*` value appears anywhere else in the corpus.
`disable_online_speech_recognition` covers recognition, not wake-word activation, which is a separate
always-listening surface.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks apps from listening for a wake word, including above the lock screen.**

      ## What it does
      Sets `LetAppsActivateWithVoice` and `LetAppsActivateWithVoiceAboveLock` to 2 (Force Deny) under
      `Policies\Microsoft\Windows\AppPrivacy`. No app can register for voice activation, so the
      always-listening wake-word path is closed for every account on the device.

      ## Benefits
      - **Nothing listens for a keyword**: the always-on activation path is denied outright
      - **Lock screen covered**: the above-lock variant is set as well, closing the stronger exposure
      - **Locked**: with Force Deny, users and apps cannot grant the capability back

      ## Drawbacks
      - **Wake words stop working**: any voice assistant you actually want, including third-party ones,
        stops responding to its keyword
      - **Above-lock convenience**: hands-free use while the PC is locked ends
      - **Users cannot re-enable**: the Settings controls are locked while the policy is applied

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later
      - **Takes effect**: immediately; an app already running may need restarting
      - **Reverting**: deletes both values, restoring user control
      - Push-to-talk and manually invoked voice input are unaffected

      ## Recommendation
      Apply it on any machine with a microphone where you do not use a wake word. Skip it if you rely on
      hands-free voice activation, including for accessibility.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Privacy (LetAppsActivateWithVoice)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
      - [Manage connections from Windows components to Microsoft services, 18.6 Speech](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Shipped `AppPrivacy.admx` on Windows 11 build 26100
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\AppPrivacy.admx` and `en-US\AppPrivacy.adml` on build 26100, both policies `class="Machine"` with the 0 / 1 / 2 enum (tier A, shipped ADMX and ADML)
2. Policy CSP - Privacy (LetAppsActivateWithVoice family), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. String evidence on build 26100: both value names in `agentactivationruntimewindows.dll`, `AarSvc.dll` and `SettingsHandlers_SpeechPrivacy.dll` (tier A, product artifact)
4. privacy.sexy, for the per-user `AgentActivationEnabled` siblings, https://github.com/undergroundwires/privacy.sexy (tier C)

### `disable_windows_backup` Windows Backup and cloud restore

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync`, value `EnableWindowsBackup`,
`REG_DWORD`. Disabled writes `0`; "Not configured (Stock Default)" is `absent`.

The shipped 26100 `SettingSync.admx` defines it `class="Machine"` at
`key="Software\Policies\Microsoft\Windows\SettingSync"`, `valueName="EnableWindowsBackup"`,
`enabledValue` 1 and `disabledValue` 0, `supportedOn="windows:SUPPORTED_Windows_10_0_NOSERVER"`. The
ADML is verbatim: "If you enable this policy setting, windows backup will occur periodically. If you
disable or do not configure this policy setting, windows backup will not take place." The value name is
present in `SyncSettings.dll`, `CloudRestoreLauncher.dll` and `cdp.dll` on 26100;
`CloudRestoreLauncher.dll` is the cloud-restore entry point.

Not a duplicate: `disable_settings_sync` writes `DisableSettingSync` and
`DisableSettingSyncUserOverride` under the same key. Those govern legacy roaming settings sync;
`EnableWindowsBackup` governs the newer Windows Backup and cloud restore pipeline.

**Corrections needed:** `none` on the mechanism, but the copy constraint in correction 35 is
non-negotiable: 0 and absent are documented as behaviourally identical, so this is a lock, not a
behaviour change.

**Ready-to-paste info block:**

```yaml
    info: |
      **Locks periodic Windows Backup off so nothing can switch it on later.**

      ## What it does
      Sets `EnableWindowsBackup` to 0 under `Policies\Microsoft\Windows\SettingSync`, the policy behind
      periodic Windows Backup and cloud restore. Microsoft's own text says backup does not take place
      when the policy is disabled or unconfigured, so this pins the existing state.

      ## Benefits
      - **Cannot be turned on later**: an OOBE flow, a Settings nudge or a servicing update cannot start
        periodic backup behind your back
      - **Explicit state**: the machine records a decision rather than relying on a default
      - **Machine-wide**: covers every account on the device

      ## Drawbacks
      - **No visible change today**: periodic backup does not take place when the policy is unset either,
        so you will not see a difference after applying it
      - **Manual backup still possible**: the Windows Backup app can still be run by hand on an unmanaged
        machine, so this is not a hard block
      - **No cloud restore**: you give up the automatic restore path on a future reinstall or new PC

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 client editions
      - **Takes effect**: immediately
      - **Reverting**: deletes the policy value, restoring the Windows default
      - This is a different feature from settings sync, which has its own policy values in the same key

      ## Recommendation
      Apply it if you back up deliberately with your own tooling and do not want a Microsoft-managed copy
      of your setup. Skip it if cloud restore on a future PC is something you would actually use.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services, 21 Sync your settings](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Shipped `SettingSync.admx` and `en-US\SettingSync.adml` on Windows 11 build 26100
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\SettingSync.admx` and `en-US\SettingSync.adml` on build 26100 (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: `EnableWindowsBackup` in `SyncSettings.dll`, `CloudRestoreLauncher.dll` and `cdp.dll` (tier A, product artifact)
3. Manage connections from Windows components to Microsoft services, section 21 Sync your settings, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### `disable_online_tips` Settings app online tips

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer`, value
`AllowOnlineTips`, `REG_DWORD`. Disabled writes `0`; "Enabled (Stock Default)" is `absent`.

The shipped 26100 `ControlPanel.admx` defines the policy `class="Machine"` at
`key="Software\Microsoft\Windows\CurrentVersion\Policies\Explorer"` with a
`<boolean id="CheckBox_AllowOnlineTips" valueName="AllowOnlineTips">` carrying `trueValue` 1 and
`falseValue` 0, `supportedOn="windows:SUPPORTED_Windows_10_0_RS3"`. The ADML: "Enables or disables the
retrieval of online tips and help for the Settings app. If disabled, Settings will not contact
Microsoft content services to retrieve tips and help content." The value name is present in
`SystemSettings.dll`, the Settings app itself, which is the consumer, so the network claim is
supportable.

**Note the key.** It lives under `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`, not
under `Software\Policies\...`. privacy.sexy writes this value to
`HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, which is not the key in the shipped ADMX. The
shipped ADMX wins; do not copy privacy.sexy here.

Not a duplicate: `disable_tips_and_suggestions` targets the Content Delivery Manager notification tips.
`AllowOnlineTips` is the Settings-app help-content fetch, a different surface and different traffic.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the Settings app fetching help and tip content from Microsoft over the network.**

      ## What it does
      Sets `AllowOnlineTips` to 0 under `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`,
      the Group Policy "Allow Online Tips". Settings stops contacting Microsoft content services and
      shows only the help text that ships with Windows.

      ## Benefits
      - **Concrete traffic reduction**: the Settings app stops making content-service requests
      - **Read by the Settings app itself**: the value name is present in `SystemSettings.dll`
      - **Machine-wide**: covers every account on the device

      ## Drawbacks
      - **Stale help text**: Settings pages show only the locally shipped help, which can be out of date
      - **No updated guidance**: fixes and clarifications Microsoft publishes after release do not appear
      - **Small surface**: this is one app's help fetch, not a system-wide network control

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later
      - **Takes effect**: immediately
      - **Reverting**: deletes the value, restoring the Windows default
      - The key is under `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`, not under
        `Software\Policies`; several tweak tools get this wrong

      ## Recommendation
      Apply it on any machine where you would rather Settings did not talk to Microsoft to render a
      help panel. Skip it if you lean on the in-Settings help links.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - Shipped `ControlPanel.admx` and `en-US\ControlPanel.adml` on Windows 11 build 26100
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\ControlPanel.admx` and `en-US\ControlPanel.adml` on build 26100 (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: `AllowOnlineTips` present in `SystemSettings.dll` (tier A, product artifact)
3. Manage connections from Windows components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
4. privacy.sexy, recorded here as the upstream that writes this value to the wrong key, https://github.com/undergroundwires/privacy.sexy (tier C, cited as a defect record, not as support)

### `disable_wer_service` Windows Error Reporting service

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:** service `WerSvc`, display name "Windows Error Reporting Service", backed by
`System32\WerSvc.dll`.

| Option | Start type |
|---|---|
| Disabled | `Disabled` |
| Stock Default | **unresolved**, restore the pre-apply start type from the snapshot |

`WerSvc` is present on 26100. It has no SCM dependents and no dependencies, so disabling it does not
cascade to any other service. This is the third leg of a job the corpus had two-thirds done: it already
writes the WER `Disabled` policy value in `privacy.yaml` and disables the
`\Microsoft\Windows\Windows Error Reporting\QueueReporting` scheduled task in `services.yaml`, whose own
info copy already calls disabling the service "a natural pairing". Genuinely absent from the corpus
until now.

**Corrections needed:** see correction 36. Two things must not ship as proposed. The claim that this
"stops WER consuming CPU on a crash" is unmeasured and must be dropped; the honest benefit is that
crash reports stop being generated and uploaded. And the shipped default start type is unresolved: the
proposal says Manual, but no authoritative source confirms it, so the revert must restore the pre-apply
start type from the snapshot rather than writing a literal.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the Windows Error Reporting service, so crash reports are not collected or uploaded at all.**

      ## What it does
      Sets the `WerSvc` service to Disabled. Windows Error Reporting stops running, so crash and hang
      reports are not gathered, queued or sent. This is the service-level companion to the WER policy
      value and the WER queue-reporting scheduled task.

      ## Benefits
      - **Reports never created**: nothing is queued to upload, rather than queued and then blocked
      - **No dependents**: `WerSvc` has no SCM dependencies in either direction, so nothing else breaks
      - **Completes the set**: pairs with the WER policy value and the queue-reporting task

      ## Drawbacks
      - **Crash history goes quiet**: Reliability Monitor's crash history and the "Windows has recovered
        from an unexpected shutdown" dialogs stop appearing
      - **Diagnostics lost first**: on an unstable machine you have removed your first diagnostic
      - **Third-party handlers**: some crash handlers query WER state and behave differently
      - **No speed gain**: this is a privacy and noise reduction, not a performance tweak

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 all supported versions
      - **Takes effect**: immediately, the service stops on apply
      - **Reverting**: restores the previous start type from the snapshot
      - The shipped default start type is not confirmed by any authoritative source, which is exactly
        why the revert reads it from the snapshot rather than writing a fixed value

      ## Recommendation
      Apply it on a stable machine where you have already disabled Windows Error Reporting by policy and
      want the service gone too. Do not apply it while troubleshooting crashes.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [WER Settings](https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings)
      - [Security guidelines for disabling system services](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
```

**Sources:**
1. WER Settings, https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings (tier A, for what WER does)
2. Live service presence and SCM dependency graph on build 26100: `WerSvc` present, `WerSvc.dll` in `System32`, no dependents and no dependencies (tier A for existence only, not for default start type)
3. `src-tauri/tweaks/services.yaml`, `task_wer_queuereporting`, whose info copy already names `WerSvc` as the natural pairing (project record, dedupe and cross-reference)
4. WinUtil, privacy.sexy and Sophia Script, all in agreement on `WerSvc` as the correct service short name, https://github.com/undergroundwires/privacy.sexy (tier C)

