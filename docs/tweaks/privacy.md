# Privacy & Telemetry tweaks

This category covers Windows diagnostic data and telemetry, the legacy Customer Experience Improvement Program and compatibility appraiser, cloud sync and cloud lookups (clipboard, settings, backup, search, File Explorer), advertising and personalization identifiers, content-suggestion surfaces (Settings, Start, lock screen, tips), app capability locks (location, app diagnostics, voice activation), Windows Error Reporting and two Microsoft Edge policies. The primary platform is Windows 11 24H2 (build 26100) and newer, including 25H2; the secondary platform is Windows 10 IoT Enterprise LTSC 2021 (build 19044), and each entry says where a mechanism does not reach it. No tweak in this file carries a `windows:` build gate, so every tweak is offered on every supported build; where a policy is only read on newer builds, the value is still written on older ones and simply has no effect there. Several tweaks author a single option on purpose: where the stock state is consent-derived or unresolved (a scheduled task's enabled state, a service start type, a value that only appears once the user answers a prompt), the way back is the snapshot rather than an invented literal.

A note on "System Default" for every entry below: it is not an option anyone authors. The app shows System Default when the live machine state matches none of the tweak's options, and selecting it restores the tweak's snapshot, meaning the exact values captured before the first apply (including "value did not exist", which restores by deleting the value). A toggle (one authored option) moves between that option and System Default; a dropdown lists System Default alongside its authored options.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Limit diagnostic data to the Required level](#limit-diagnostic-data-to-the-required-level) | `disable_diagnostic_data` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable the Customer Experience Improvement Program](#disable-the-customer-experience-improvement-program) | `disable_ceip_tasks` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Compatibility Appraiser tasks](#disable-compatibility-appraiser-tasks) | `disable_compat_appraiser` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off feedback request notifications](#turn-off-feedback-request-notifications) | `disable_feedback_notifications` | Switch (2 options) | low | admin | no | VERIFIED |
| [Set feedback prompt frequency to Never](#set-feedback-prompt-frequency-to-never) | `disable_feedback_frequency` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Block OneSettings config downloads](#block-onesettings-config-downloads) | `disable_onesettings_downloads` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Exclude the device name from telemetry](#exclude-the-device-name-from-telemetry) | `disable_device_name_in_telemetry` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Activity History and Timeline](#disable-activity-history-and-timeline) | `disable_activity_history` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable the cross-device cloud clipboard](#disable-the-cross-device-cloud-clipboard) | `disable_cloud_clipboard` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off online speech recognition](#turn-off-online-speech-recognition) | `disable_online_speech_recognition` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable inking and typing personalization](#disable-inking-and-typing-personalization) | `disable_inking_typing_personalization` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off the advertising ID](#turn-off-the-advertising-id) | `disable_advertising_id` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off tailored experiences](#turn-off-tailored-experiences) | `disable_tailored_experiences` | Switch | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off location tracking](#turn-off-location-tracking) | `disable_location_tracking` | Switch | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Block app diagnostic access](#block-app-diagnostic-access) | `disable_app_diagnostics` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off Find My Device](#turn-off-find-my-device) | `disable_find_my_device` | Switch (2 options) | medium | admin | no | VERIFIED |
| [Disable settings sync](#disable-settings-sync) | `disable_settings_sync` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Disable Windows Error Reporting](#disable-windows-error-reporting) | `disable_error_reporting` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off suggested content in Settings](#turn-off-suggested-content-in-settings) | `disable_suggested_content_settings` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off Start menu app suggestions](#turn-off-start-menu-app-suggestions) | `disable_start_app_suggestions` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off app launch tracking](#turn-off-app-launch-tracking) | `disable_app_launch_tracking` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off lock screen ads and fun facts](#turn-off-lock-screen-ads-and-fun-facts) | `disable_lockscreen_spotlight_ads` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off Windows tips and suggestions](#turn-off-windows-tips-and-suggestions) | `disable_tips_and_suggestions` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Windows consumer features](#disable-windows-consumer-features) | `disable_consumer_features` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off Microsoft Edge telemetry](#turn-off-microsoft-edge-telemetry) | `disable_edge_telemetry` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block website access to your language list](#block-website-access-to-your-language-list) | `disable_language_list_access` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable File Explorer cloud recommendations](#disable-file-explorer-cloud-recommendations) | `disable_explorer_cloud_recommendations` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable app and device inventory collectors](#disable-app-and-device-inventory-collectors) | `disable_app_device_inventory` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off device search history](#turn-off-device-search-history) | `disable_search_history` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable cloud content in search](#disable-cloud-content-in-search) | `disable_cloud_content_search` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block voice activation and wake words](#block-voice-activation-and-wake-words) | `disable_voice_activation` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Windows Backup and cloud restore](#disable-windows-backup-and-cloud-restore) | `disable_windows_backup` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Settings app online tips](#disable-settings-app-online-tips) | `disable_online_tips` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable the Windows Error Reporting service](#disable-the-windows-error-reporting-service) | `disable_wer_service` | Switch | medium | admin | no | VERIFIED-WITH-CORRECTION |

## Tweaks

### Limit diagnostic data to the Required level

`disable_diagnostic_data` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Caps Windows diagnostic data at the Required level and takes the choice away from the Settings app.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `allow_telemetry` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value `AllowTelemetry`, `REG_DWORD` |

| Option | `allow_telemetry` |
|---|---|
| Required only | `1` |
| User's choice | `absent` |

System Default appears only if the value holds something other than 1 or nothing (for example 0 or 3 written by another tool or an MDM profile); selecting it restores the snapshot. The stock Windows state is value-absent, which is exactly the "User's choice" option.

#### How it works

`AllowTelemetry` is the Group Policy "Allow Diagnostic Data" (older label "Allow Telemetry") from `DataCollection.admx`, and the Policy CSP setting `System/AllowTelemetry`. It sets the ceiling on what the Connected User Experiences and Telemetry service (`DiagTrack`) collects and uploads. Microsoft's allowed values are 0 (Security), 1 (Basic, now called Required, the framework default), and 3 (Full, now called Optional). Value 2 (Enhanced) exists only on Windows 10 1809 and earlier and on Windows Server 2016 and 2019. When the policy is not configured, Microsoft states that "the device will send required diagnostic data and the end user can choose whether to send optional diagnostic data from the Settings app"; that is why value-absent is the stock state and why the second option is labelled "User's choice".

Writing 1 removes the user's ability to raise the level to Optional: the Settings > Privacy and security > Diagnostics and feedback toggle is greyed out for every account. The tweak deliberately writes 1 and not 0. Value 0 (Security) is honoured only on Enterprise, Education, IoT Core and Server, and Microsoft states plainly that "using this setting on other devices is equivalent to setting the value of 1", so 1 is the portable floor that means the same thing on every edition, including Home and Pro. No option in this tweak reaches 0 on any edition.

The ADMX declares the policy `class="Both"`, so it could legally be written under HKCU as well; the hive audit found that the Computer Configuration (HKLM) value wins and is the authoritative one for the diagnostic data level, which is the hive this tweak writes.

#### Benefits
- Optional diagnostic data, including usage and inking samples, stops being sent.
- No user or app can raise the level back to Optional from Settings while the policy is present.
- Value 1 has the same meaning on every edition, so the tweak behaves identically on Home, Pro, Enterprise and LTSC.

#### Drawbacks
- Required diagnostic data still flows; this is a cap, not an off switch. Only Enterprise, Education and IoT honour the Security level (0), and this tweak does not write it.
- The `DiagTrack` service keeps running; this lowers what it sends and does not stop it.
- The diagnostic data control in Settings is greyed out until you revert.
- Microsoft loses one input for diagnosing failed updates on your device.
- Windows Insider builds override the level upward regardless of this policy.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1507 and later, every edition (with value 1). Covers LTSC 2021.
- **Takes effect**: immediately; the Settings page reflects it the next time it is opened.
- **Reverting**: choosing "User's choice" deletes the value, returning to the Windows default and giving the Settings toggle back to the user. System Default restores whatever the snapshot captured, which on a stock machine is also value-absent.

#### Interactions
- `disable_diagtrack` (Services, "Disable User Experiences and Telemetry (DiagTrack)") stops the service that transmits diagnostic data; this tweak only lowers the level it collects. The two are complementary.
- `disable_error_reporting` and `disable_wer_service`: at the Required level, crash dumps are not sent anyway, which reduces the extra privacy those two add.
- `disable_device_name_in_telemetry` and `disable_onesettings_downloads` write other values under the same `DataCollection` policy key; each owns a different value, so there is no conflict.
- `disable_edge_telemetry`: Edge on Windows follows this Windows diagnostic data setting for its own diagnostic level.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The mechanism was correct; the correction was that value 1 is the only level this tweak reaches, so no edition-dependent "minimum" (value 0) is implemented.
- **Confidence**: Microsoft-documented. Key, value name, value meanings and the unconfigured behaviour all come from Policy CSP and Microsoft's diagnostic data configuration page, and the shipped `DataCollection.admx` on build 26100 matches.
- **Reasoning**: the adversarial pass attacked the claim that the tweak pushes the level "to the minimum your edition allows"; it failed because no option writes 0, so the claim was withdrawn and the tweak is described as a portable cap at Required. The value, key, hive (Computer Configuration wins over User) and the absent default all survived.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any personal machine; the cost is diagnostic reach that benefits Microsoft rather than you. Skip it while you are actively working with Microsoft support on an update or crash issue. If you want the Security level on Enterprise, Education or IoT, that needs a value this tweak does not write.

#### Sources
1. Policy CSP - System (AllowTelemetry), allowed values, edition behaviour of value 0, and the unconfigured default, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)
2. Configure Windows diagnostic data in your organization, the diagnostic data levels and what each sends, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` on build 26100, the policy definition and its `class="Both"` declaration (tier A, shipped ADMX)

### Disable the Customer Experience Improvement Program

`disable_ceip_tasks` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Opts the machine out of the legacy Customer Experience Improvement Program and stops its upload tasks.**

#### What it changes

| Effect | Kind | Target | Presence |
|---|---|---|---|
| `ceip_enable` | registry | `HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows`, value `CEIPEnable`, `REG_DWORD` | required |
| `task_consolidator` | task | `\Microsoft\Windows\Customer Experience Improvement Program\Consolidator` | required |
| `task_usbceip` | task | `\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip` | required |
| `task_kernelceip` | task | `\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipTask` | `optional: true`, `if_missing: disabled` |

| Option | `ceip_enable` | `task_consolidator` | `task_usbceip` | `task_kernelceip` |
|---|---|---|---|---|
| Disabled | `0` | disabled | disabled | disabled |

This is a toggle: the other position is System Default, which restores the snapshot (the previous `CEIPEnable` value, usually absent, and each task's previous enabled state). The stock state of `CEIPEnable` is value-absent. The shipped enabled state of the three tasks is unresolved: enabling or disabling a task rewrites the `<Enabled>` element in its own XML under `C:\Windows\System32\Tasks`, so no live machine is evidence of the shipped state, which is why no "Enabled" option is authored.

#### How it works

`CEIPEnable` is the Group Policy "Turn off Windows Customer Experience Improvement Program" from `ICM.admx`, defined at `Software\Policies\Microsoft\SQMClient\Windows` with `enabledValue` 0 and `disabledValue` 1. So 0 is the policy's "Enabled" state, meaning CEIP is turned off. The same value appears a second time in the shipped `ICM.admx`, inside the `InternetManagement_RestrictCommunication` policy's enabled list, at the same key with the same 0, which corroborates both key and polarity from within the shipped file.

The scheduled tasks are the Software Quality Metrics (SQM) collectors and uploaders. Microsoft's own task descriptions, read from the shipped binaries, make the task half conditional: `wsqmcons.exe,-107` for `Consolidator` says "If the user has consented to participate ... this job collects and sends usage data to Microsoft", and `usbceip.dll,-602` for `UsbCeip` says "If the user has not consented ... this task does not do anything." The policy value is therefore the part that actually stops participation; disabling the tasks removes the scheduled wake-ups. `KernelCeipTask` is a Windows 7 and 8 era task; an enumeration of the CEIP task folder on build 26100 returned only `Consolidator` and `UsbCeip`, so the effect is optional and a machine without it counts as disabled for detection (and applying is a verified no-op for that effect).

#### Benefits
- The `CEIPEnable` policy opts the machine out of CEIP; that is the part that does the work.
- Recurring upload tasks stop being scheduled.
- The policy value persists even when a feature update re-enables the tasks.

#### Drawbacks
- On a machine that never opted in to CEIP, Microsoft's task descriptions say the tasks already do nothing, so the task half changes nothing there.
- This is not a modern telemetry control: diagnostic data flows through `DiagTrack`, which this tweak does not touch.
- `\Microsoft\Windows\Autochk\Proxy` is also a CEIP SQM uploader (its shipped description `acproxy.dll,-102` reads "This task collects and uploads autochk SQM data if opted-in to the Microsoft Customer Experience Improvement Program"), but it sits outside the CEIP task folder and is not part of this tweak.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 22H2. `KernelCeipTask` is usually absent on modern builds and is skipped there.
- **Takes effect**: immediately; the tasks are disabled on apply.
- **Reverting**: System Default restores the previous policy value and each task's previous enabled state from the snapshot; it does not force the tasks to "enabled" unless that is what was captured.

#### Interactions
- `task_autochk_proxy` (Services, "Disable Autochk Proxy task") covers the Autochk CEIP uploader this tweak leaves alone, and honours the `CEIPEnable` policy this tweak sets.
- `task_disk_diagnostic_datacollector` (Services) is another task whose reporting is tied to CEIP participation.
- `disable_diagnostic_data` is the lever for modern diagnostic data; this tweak is not a substitute for it.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The registry effect was correct from the start; the corrections were that the task half only matters on a machine that consented to CEIP, that the tasks' shipped enabled state is unresolved (so revert restores the snapshot rather than writing "enabled"), and that the Autochk Proxy uploader lives outside this task folder.
- **Confidence**: Microsoft-documented. `CEIPEnable` is documented on Microsoft Learn and in the shipped `ICM.admx`; the task behaviour comes from Microsoft's own shipped task description strings.
- **Reasoning**: the adversarial pass challenged the benefit of disabling the tasks (it survives only as "fewer wake-ups", since the tasks are consent-gated), and the presence of `KernelCeipTask` on 26100 (absent there, handled by `optional`). The polarity and key survived, corroborated twice inside the shipped ADMX. A separate verification pass suggested adding `\Microsoft\Windows\PI\Sqm-Tasks` to this tweak; that task is not part of it. Open question: the shipped enabled state of the tasks on a clean 26100 image.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it; the cost is nil and the policy opt-out is durable. Do not treat it as a telemetry control: if reducing outbound diagnostics is the goal, use the diagnostic data level tweak and consider the DiagTrack service tweak.

#### Sources
1. CEIPEnable, the policy value and its meaning, https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\ICM.admx` on build 26100, key, value name and polarity, plus the second occurrence under `InternetManagement_RestrictCommunication` (tier A, shipped ADMX)
3. Shipped task description strings `wsqmcons.exe,-107` (`Consolidator`), `usbceip.dll,-602` (`UsbCeip`) and `acproxy.dll,-102` (`Autochk\Proxy`), what each task does and that it is consent-gated; not evidence of default state (tier A, shipped binary resources)
4. Manage connections from Windows components to Microsoft services, context for Windows data flows, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
5. Enumeration of the CEIP task folder on build 26100, showing only `Consolidator` and `UsbCeip` (tier A for presence)

### Disable Compatibility Appraiser tasks

`disable_compat_appraiser` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the Windows upgrade-readiness appraiser and its inventory collection, including the recurring CompatTelRunner workload.**

#### What it changes

| Effect | Kind | Target | Presence |
|---|---|---|---|
| `ait_enable` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat`, value `AITEnable`, `REG_DWORD` | required |
| `disable_inventory` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat`, value `DisableInventory`, `REG_DWORD` | required |
| `task_appraiser` | task | `\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser` | required |
| `task_appraiser_exp` | task | `\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser Exp` | `optional: true`, `if_missing: disabled` |
| `task_progdata` | task | `\Microsoft\Windows\Application Experience\ProgramDataUpdater` | `optional: true`, `if_missing: disabled` |
| `task_startup` | task | `\Microsoft\Windows\Application Experience\StartupAppTask` | required |

| Option | `ait_enable` | `disable_inventory` | `task_appraiser` | `task_appraiser_exp` | `task_progdata` | `task_startup` |
|---|---|---|---|---|---|---|
| Disabled | `0` | `1` | disabled | disabled | disabled | disabled |

This is a toggle: the other position is System Default, which restores the snapshot. The stock state of both policy values is value-absent; the shipped enabled state of the tasks is not fixed by any source, so no "Enabled" option is authored and the snapshot carries the way back.

#### How it works

Both registry values are shipped Group Policy settings in `AppCompat.admx` on build 26100. `AppCompatTurnOffApplicationImpactTelemetry` writes `AITEnable` with `enabledValue` 0, turning off Application Impact Telemetry. `AppCompatTurnOffProgramInventory` writes `DisableInventory` with no explicit value pair, meaning the ADMX default of 1 when enabled, turning off the Application Compatibility Program Inventory; DISA STIG finding V-253385 independently specifies `DisableInventory` = 1 at the same key.

The scheduled tasks under `\Microsoft\Windows\Application Experience\` run `CompatTelRunner.exe` and feed the inventory. A full enumeration of that folder on build 26100 returns exactly six tasks: `MareBackup`, `Microsoft Compatibility Appraiser`, `Microsoft Compatibility Appraiser Exp`, `PcaPatchDbTask`, `SdbinstMergeDbTask` and `StartupAppTask`. `ProgramDataUpdater` does not exist on 26100 and survives here only for the Windows 10 surface, hence `optional`. `Microsoft Compatibility Appraiser Exp` exists and is Ready on 26100 and runs appraiser work, so it is included, also as optional because it is not present on every build. `SdbinstMergeDbTask` is application-compatibility shim database maintenance, not telemetry, and is deliberately left enabled; `PcaPatchDbTask` and `MareBackup` are not part of this tweak.

The appraiser's data feeds Microsoft's upgrade-readiness and safeguard-hold logic: it is how Windows Update learns that an installed app or driver would break on the next feature update.

#### Benefits
- The recurring `CompatTelRunner.exe` appraiser workload stops being scheduled.
- Installed-application inventory stops being gathered and reported.
- Both registry values are shipped Group Policy settings, not undocumented keys, and they persist across feature updates.

#### Drawbacks
- Appraiser data feeds Microsoft's safeguard-hold logic, so compatibility problems are less likely to be caught before a feature update.
- Feature updates re-enable the tasks; the policy values persist, and the tweak then reads as System Default until reapplied.
- The CPU and disk relief is widely reported but is not measured by any Microsoft or benchmark source.
- Do not extend this by deleting or renaming `CompatTelRunner.exe`: servicing restores it, and removing it has broken Windows updates.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 22H2. On 26100 `ProgramDataUpdater` is absent and is skipped.
- **Takes effect**: immediately; the tasks are disabled on apply.
- **Reverting**: System Default restores the previous policy values (normally absent) and each task's previous enabled state from the snapshot.

#### Interactions
- `disable_app_device_inventory` writes four other values under the same `AppCompat` policy key (the 24H2 collector family). Different values, no conflict; the two together cover both the legacy and the 24H2 inventory surfaces.
- `DisableWin32AppBackup` in `disable_app_device_inventory` also governs the compatibility scan performed by `appraiser.dll` and `aemarebackup.dll` over backed-up apps.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The registry half was correct; the corrections were about the task set: `ProgramDataUpdater` does not exist on 26100 (so it must be optional), and `Microsoft Compatibility Appraiser Exp` exists and runs appraiser work.
- **Confidence**: Microsoft-documented for the policy values (shipped ADMX, with DISA STIG corroboration); task presence is from direct enumeration on build 26100.
- **Reasoning**: the adversarial pass enumerated the task folder and found one targeted task missing and one appraiser task uncovered; both are handled. The performance claim was attacked and downgraded to "widely reported, not measured". A later gap-verification pass rejected a separate task-only appraiser tweak as a duplicate of this one and noted `PcaPatchDbTask` and `MareBackup` as further candidates; they are not included. The tasks' shipped enabled state remains open.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine you keep on its current Windows version. Hold off if you are about to run a feature upgrade and want Microsoft's compatibility checks working in your favour.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\AppCompat.admx` on build 26100, both policy definitions and polarities (tier A, shipped ADMX)
2. DISA STIG, Windows 11, V-253385, Application Compatibility Program Inventory, independent specification of `DisableInventory` = 1, https://www.stigviewer.com/stigs/microsoft_windows_11/2022-06-24/finding/V-253385 (tier B)
3. `Get-ScheduledTask -TaskPath '\Microsoft\Windows\Application Experience\'` and `schtasks /query` on build 26100, the six tasks present (tier A, shipped OS state for task presence)
4. Configure Windows diagnostic data in your organization, context for the appraiser and inventory data, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)

### Turn off feedback request notifications

`disable_feedback_notifications` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows interrupting you with Feedback Hub survey prompts.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `feedback_notif` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value `DoNotShowFeedbackNotifications`, `REG_DWORD` |

| Option | `feedback_notif` |
|---|---|
| Off | `1` |
| On | `absent` |

System Default appears only if the value holds something other than 1 or nothing; selecting it restores the snapshot. The stock Windows state is value-absent ("On").

#### How it works

Microsoft documents this verbatim: "Create a REG_DWORD registry setting named DoNotShowFeedbackNotifications in HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows\DataCollection with a value of 1 (one)", equivalent to enabling Computer Configuration > Administrative Templates > Windows Components > Data Collection and Preview Builds > "Do not show feedback notifications". It suppresses the Feedback Hub survey prompts ("How likely are you to recommend Windows...") machine-wide. It is a machine-class policy, so one value covers every account. No SKU gate is documented. Diagnostic data collection is untouched; only the prompts go.

#### Benefits
- The periodic feedback survey prompts stop.
- One policy covers every account, unlike the per-user frequency setting.
- Nothing depends on the prompts, so there is no functional cost.

#### Drawbacks
- It hides prompts only; diagnostic data collection is unchanged.
- On a Windows Insider machine you stop being asked for the feedback the program exists to gather.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and all supported Windows 10 versions including LTSC 2021; every edition.
- **Takes effect**: immediately.
- **Reverting**: "On" deletes the value, restoring the Windows default. You can still open Feedback Hub and submit feedback yourself at any time while it is applied.

#### Interactions
- `disable_feedback_frequency` sets the same outcome per user through the Settings value; with this policy applied, that tweak adds nothing.
- `task_feedback_dmclient` (Services) disables the SIUF feedback download tasks.
- Shares the `DataCollection` policy key with `disable_diagnostic_data`, `disable_onesettings_downloads` and `disable_device_name_in_telemetry`; each owns a different value.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented: the exact key, value and data are published by Microsoft and match the shipped ADMX.
- **Reasoning**: key, value name, type, polarity and the absent default all match Microsoft's own instruction. The hive audit confirmed it is a machine-class policy written to HKLM. The research names the defining ADMX as `DataCollection.admx`; the hive audit names `FeedbackNotifications.admx`; both agree on class and key.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine that is not enrolled in the Windows Insider Program. If you are an Insider and want to be asked, leave it alone.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.16 Feedback and diagnostics, the exact registry instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` on build 26100, policy "Do not show feedback notifications" (tier A, shipped ADMX)

### Set feedback prompt frequency to Never

`disable_feedback_frequency` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Sets your account's Windows feedback prompt schedule to never.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `num_siuf` | registry | `HKCU\Software\Microsoft\Siuf\Rules`, value `NumberOfSIUFInPeriod`, `REG_DWORD` |
| `period_ns` | registry | `HKCU\Software\Microsoft\Siuf\Rules`, value `PeriodInNanoSeconds`, `REG_DWORD` |

| Option | `num_siuf` | `period_ns` |
|---|---|---|
| Never | `0` | `0` |
| Automatic | `absent` | `absent` |

System Default appears when the pair matches neither row, for example when Settings has set "Always", "Once a day" or "Once a week"; selecting it restores the snapshot. The stock Windows state is both values absent ("Automatically").

#### How it works

Settings > Privacy and security > Diagnostics and feedback > Feedback frequency stores its choice in these two per-user values. Microsoft publishes the full mapping:

| Setting | `PeriodInNanoSeconds` | `NumberOfSIUFInPeriod` |
|---|---|---|
| Automatically | delete the value | delete the value |
| Never | 0 | 0 |
| Always | 100000000 | delete the value |
| Once a day | 864000000000 | 1 |
| Once a week | 6048000000000 | 1 |

"Never" is the pair (0, 0); writing 0 to only one of the two is not one of the five published states, so the tweak writes both. Microsoft's table lists both values as `REG_DWORD` yet gives "Once a day" and "Once a week" data that exceeds the 32-bit range; that inconsistency is Microsoft's and does not affect the 0 pair this tweak writes. Because the values are in HKCU, they apply to the signed-in account only and are written in-process as the interactive user.

#### Benefits
- Windows stops asking this account for feedback.
- It is a per-user setting, so it works without elevation.
- (0, 0) is Microsoft's own documented mapping for "Never".

#### Drawbacks
- Current user only: other accounts keep their own schedule.
- Redundant if the machine-wide feedback notification policy is already applied.
- Nothing locks the Settings control, so a user can undo it from Settings.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and all supported Windows 10 versions including LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Automatic" deletes both values, returning the schedule to Automatically. System Default restores whatever pair the snapshot captured.

#### Interactions
- `disable_feedback_notifications` is the machine-wide, locked equivalent; with it applied this tweak adds nothing.
- `task_feedback_dmclient` (Services) covers the SIUF download tasks.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that "Never" requires `PeriodInNanoSeconds` = 0 as well as `NumberOfSIUFInPeriod` = 0; the shipped option writes the documented pair.
- **Confidence**: Microsoft-documented: the five-state mapping table is Microsoft's.
- **Reasoning**: the adversarial pass compared the option against Microsoft's table and found the half-written pair undocumented; the shipped (0, 0) pair survives. The 32-bit overflow in Microsoft's table was examined and found irrelevant to the 0 pair.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you cannot or do not want to set a machine-wide policy. If you have admin rights, the machine-wide feedback notification tweak is the more durable choice.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.16 Feedback and diagnostics, the five-state mapping table, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. privacy.sexy, which writes the same `HKCU\Software\Microsoft\Siuf\Rules` pair, https://github.com/undergroundwires/privacy.sexy (tier C, corroboration only)

### Block OneSettings config downloads

`disable_onesettings_downloads` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows periodically downloading configuration from Microsoft's OneSettings service.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `onesettings` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value `DisableOneSettingsDownloads`, `REG_DWORD` |

| Option | `onesettings` |
|---|---|
| Blocked | `1` |
| Allowed | `absent` |

System Default appears only if the value holds something other than 1 or nothing (for example 0); selecting it restores the snapshot. The stock Windows state is value-absent ("Allowed").

#### How it works

This is the Group Policy "Disable OneSettings Downloads", defined in the shipped `DataCollection.admx` on 26100 as `class="Machine"` at `Software\Policies\Microsoft\Windows\DataCollection` with `enabledValue` 1 and `disabledValue` 0; the ADML explain text matches the Policy CSP text word for word. Microsoft: "If you enable this policy, Windows won't attempt to connect with the OneSettings Service." Microsoft describes OneSettings as the service "used by Windows components and apps, such as the telemetry service, to dynamically update their configuration", and warns "If you turn off this service, apps using this service may stop working."

This is a remote-configuration control, not a telemetry control: it does not change how much diagnostic data is collected, which is governed by `AllowTelemetry` and the Connected User Experiences and Telemetry service. What it stops is configuration and feature-flag delivery between updates.

The Policy CSP applicability for this setting is Windows 11 21H2 (build 22000) and later. The tweak has no build gate, so on Windows 10 (including LTSC 2021) the value is still written, but that is outside Microsoft's documented applicability and nothing is documented to read it there.

#### Benefits
- Windows components keep the configuration they shipped with.
- One recurring outbound connection to Microsoft stops.
- Staged feature flags and experiments delivered this way stop arriving.

#### Drawbacks
- Microsoft's own warning: apps using the service "may stop working".
- Not a telemetry control: diagnostic data volume is unchanged.
- Configuration-only mitigations Microsoft ships through OneSettings will not reach the machine.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 21H2 (build 22000) and newer per Microsoft, so all of the primary platform. On Windows 10 and LTSC 2021 the value is written but is outside the documented range.
- **Takes effect**: immediately.
- **Reverting**: "Allowed" deletes the value, restoring the Windows default.

#### Interactions
- Shares the `DataCollection` policy key with `disable_diagnostic_data`, `disable_feedback_notifications` and `disable_device_name_in_telemetry`; each owns a different value.
- For reducing diagnostic data, use `disable_diagnostic_data` and `disable_diagtrack` (Services); this tweak does not do that.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The mechanism was correct; the corrections were that the documented floor is Windows 11 21H2 and that the policy is a configuration-download control, not a way to stop Windows re-toggling telemetry (nothing sources that claim).
- **Confidence**: Microsoft-documented: Policy CSP, the connections article and the shipped ADMX and ADML all agree.
- **Reasoning**: the adversarial pass attacked the headline "can silently re-toggle telemetry" benefit and found no source; it was dropped. Key, value, type, polarity and the absent default survived. Open point: the research asked for either a `windows:` gate at build 22000 or a stated floor; the tweak states the floor and has no gate.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want the local configuration to stay put and you accept that some Microsoft apps may misbehave. Skip it on a machine where Store or Microsoft 365 apps must be reliable.

#### Sources
1. Policy CSP - System (DisableOneSettingsDownloads), behaviour and the Windows 11 21H2 applicability, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)
2. Manage connections from Windows components to Microsoft services, section 31 Services Configuration, what OneSettings is and the "may stop working" warning, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` and `en-US\DataCollection.adml` on build 26100 (tier A, shipped ADMX and ADML)

### Exclude the device name from telemetry

`disable_device_name_in_telemetry` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Pins the device-name exclusion so nothing can later start attaching your computer name to diagnostic data.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `device_name` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection`, value `AllowDeviceNameInTelemetry`, `REG_DWORD` |

| Option | `device_name` |
|---|---|
| Excluded | `0` |
| Included | `absent` |

System Default appears if the value holds 1 (the device name is being sent, typically set by an MDM profile or administrative template); selecting it restores the snapshot. The stock Windows state is value-absent. Note that the "Included" label names the option that deletes the value, which in practice also excludes the device name, because that is Windows' default when the policy is not configured.

#### How it works

The shipped `DataCollection.admx` on 26100 defines policy `AllowDeviceNameInDiagnosticData` ("Allow device name to be sent in Windows diagnostic data") as `class="Machine"` with `valueName="AllowDeviceNameInTelemetry"`, `enabledValue` 1, `disabledValue` 0, supported from Windows 10 1803 (RS4). The policy name and the registry value name differ on purpose; the value name is the one Windows reads, and writing a value called `AllowDeviceNameInDiagnosticData` would be silently ignored.

The shipped ADML explain string reads: "This policy allows the device name to be sent to Microsoft as part of Windows diagnostic data. If you disable or do not configure this policy setting, then device name will not be sent to Microsoft as part of Windows diagnostic data." Policy CSP gives Default Value 0. Writing 0 and leaving the value absent are therefore behaviourally identical on every build in range: the tweak pins the existing default so a later configuration cannot flip it to 1, rather than changing what is sent today.

#### Benefits
- Defends the default: records the exclusion explicitly so a later MDM profile or administrative template writing 1 is visible as a change from the pinned state.
- The exclusion is recorded rather than implied.
- Nothing on the machine depends on the device name being sent.

#### Drawbacks
- No visible change on a stock install: the shipped default already excludes the device name.
- Narrow: it governs one field, not the diagnostic data itself.
- On a domain or Intune-managed device, a policy pushed later still wins.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1803 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "Included" deletes the value; behaviour is unchanged because the default is the same.

#### Interactions
- Shares the `DataCollection` policy key with `disable_diagnostic_data`, `disable_feedback_notifications` and `disable_onesettings_downloads`; each owns a different value.
- For a visible reduction in what leaves the machine, `disable_diagnostic_data` is the relevant tweak.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The mechanism was correct; the correction was that writing 0 produces no privacy gain over the unset default, so the tweak is described as pinning a default, not changing behaviour.
- **Confidence**: Microsoft-documented: Policy CSP and the shipped ADMX and ADML.
- **Reasoning**: the adversarial pass attacked the claimed privacy gain using Microsoft's own ADML text and the CSP default, and the claim fell; the value-name mismatch was checked against the ADMX and the shipped value name survived. The cross-cutting inclusion review keeps controls like this when the copy states plainly that there is no visible change today.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you manage the machine's configuration deliberately and want the exclusion recorded. If you want a visible reduction in what leaves the machine, use the diagnostic data level tweak; this one changes nothing on a stock install.

#### Sources
1. Policy CSP - System (AllowDeviceNameInDiagnosticData), Default Value 0, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\DataCollection.admx` and `en-US\DataCollection.adml` on build 26100, the value name and the unconfigured behaviour (tier A, shipped ADMX and ADML)
3. Configure Windows diagnostic data in your organization, context for what diagnostic data carries, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)

### Disable Activity History and Timeline

`disable_activity_history` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows building an activity feed of what you do and publishing it to your account.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `enable_feed` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, value `EnableActivityFeed`, `REG_DWORD` |
| `publish_activities` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, value `PublishUserActivities`, `REG_DWORD` |
| `upload_activities` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, value `UploadUserActivities`, `REG_DWORD` |

| Option | `enable_feed` | `publish_activities` | `upload_activities` |
|---|---|---|---|
| Disabled | `0` | `0` | `0` |
| Enabled | `absent` | `absent` | `absent` |

System Default appears when the three values are mixed (for example only one of them set) or hold 1; selecting it restores the snapshot. The stock Windows state is all three absent.

#### How it works

The shipped `OSPolicy.admx` on 26100 defines all three as `class="Machine"` policies at `Software\Policies\Microsoft\Windows\System`, each with `enabledValue` 1 and `disabledValue` 0. Policy CSP gives Default Value 1 (allowed) for all three, so absent means the feature is on. `EnableActivityFeed` controls whether Windows records the local activity feed at all; `PublishUserActivities` controls whether activities are published; `UploadUserActivities` controls whether they are uploaded to the account in the cloud. The Settings > Privacy and security > Activity history toggles follow the policy and cannot be flipped back while it is set.

The shipped `en-US\OSPolicy.adml` on 26100 closes all three help strings (`EnableActivityFeed_Help`, `PublishUserActivities_Help`, `UploadUserActivities_Help`) with the identical sentence "Policy change takes effect immediately", so no reboot or sign-out is needed. On Windows 11, Timeline was removed and consumer cloud upload of activity history was retired, so much of what this blocks is already dormant; on Windows 10, Timeline is the visible feature that stops working.

#### Benefits
- Windows stops recording an activity trail on the machine.
- Activities cannot be published or uploaded to your Microsoft or work account.
- The Settings toggles are locked by policy.

#### Drawbacks
- Small effect on Windows 11, where Timeline and consumer cloud upload are already gone.
- On Windows 10, Timeline stops working.
- Activities already stored are not deleted; clearing them is a separate action in Settings.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1803 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately, per Microsoft's own help text for all three policies.
- **Reverting**: "Enabled" deletes all three values, restoring the Windows default. Stored history that was never recorded while the policy was set is not recovered.

#### Interactions
- Shares the `Policies\Microsoft\Windows\System` key with `disable_cloud_clipboard`; different values, no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The key, value names, type, polarity and absent default were all correct; the correction was that the policies take effect immediately (per all three shipped ADML help strings), so no reboot is required.
- **Confidence**: Microsoft-documented: the connections article, Policy CSP and the shipped ADMX and ADML.
- **Reasoning**: the adversarial pass attacked the reboot requirement against the shipped ADML and it fell three times over; everything else survived. The hive audit confirmed all three as machine-class values written to HKLM.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on Windows 11; the cost is essentially nil and it prevents the feature being switched back on. On Windows 10, skip it if you actually use Timeline.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.22 Activity History, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (UploadUserActivities and siblings), Default Value 1, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\OSPolicy.admx` and `en-US\OSPolicy.adml` on build 26100, definitions and "Policy change takes effect immediately" (tier A, shipped ADMX and ADML)

### Disable the cross-device cloud clipboard

`disable_cloud_clipboard` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Keeps everything you copy on this machine instead of syncing it to Microsoft's cloud.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `cross_device_clip` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, value `AllowCrossDeviceClipboard`, `REG_DWORD` |

| Option | `cross_device_clip` |
|---|---|
| Disabled | `0` |
| Enabled | `absent` |

System Default appears if the value holds 1 (explicitly allowed); selecting it restores the snapshot. The stock Windows state is value-absent, which behaves as allowed.

#### How it works

Microsoft documents `AllowCrossDeviceClipboard` with Group Policy friendly name "Allow Clipboard synchronization across devices", under System > OS Policies, in `OSPolicy.admx`. Supported values are 0 (Not allowed) and 1 (Allowed, the default); the most restricted value is 0. The policy governs whether "an item copied to the clipboard is uploaded to the cloud so that other devices can access it". Setting it to 0 stops the upload and greys out the cloud sync part of Settings > System > Clipboard. Local clipboard history (Win+V) is a different policy, `AllowClipboardHistory`, and is deliberately not touched.

#### Benefits
- Passwords, tokens and personal data that pass through the clipboard stop leaving the PC.
- Local clipboard history (Win+V) keeps working.
- One machine policy covers every account.

#### Drawbacks
- Copying on this PC and pasting on another PC signed into the same account stops working.
- Clipboard sharing with a paired phone through Phone Link stops as well.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes the value, restoring the Windows default and the user's own Settings choice.

#### Interactions
- Shares the `Policies\Microsoft\Windows\System` key with `disable_activity_history`; different values.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented: Policy CSP and the connections article give the value, meanings and default.
- **Reasoning**: key, value, type, polarity and absent default all match Microsoft's documentation; the hive audit confirmed a machine-class policy in HKLM. The separation from local clipboard history was checked and holds.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine where you paste credentials, keys or personal data. Leave it alone only if you actively paste between several Windows devices signed into the same account, or use Phone Link clipboard sharing.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 30 Cloud Clipboard, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - System (AllowCrossDeviceClipboard), values, default and scope, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-system (tier A)

### Turn off online speech recognition

`disable_online_speech_recognition` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops your voice being sent to Microsoft's cloud speech service for recognition.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `speech_accepted` | registry | `HKCU\Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy`, value `HasAccepted`, `REG_DWORD` |

| Option | `speech_accepted` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default appears when the value holds 1, which is what Windows writes once a user has accepted online speech recognition in OOBE or Settings; selecting it restores the snapshot. The stock state is value-absent: the value only materialises when the user answers the consent prompt.

#### How it works

`HasAccepted` is the per-user registry backing of Settings > Privacy and security > Speech > Online speech recognition. Microsoft documents only the disable direction: "Create a REG_DWORD registry setting named HasAccepted ... with a value of 0 (zero)". With 0, dictation and voice interaction fall back to on-device recognition and no audio is sent to Microsoft Speech services for this account.

The "On" option deletes the value rather than writing 1. No Microsoft source gives `HasAccepted` a shipped default of 1, and the Policy CSP text confirms the operating system defers this control to the user. Writing 1 would record a cloud-speech consent the user never gave, which on a machine where online speech was never accepted would turn cloud speech on; deleting the value returns Windows to asking the user.

The machine-wide lock for the same feature is the Group Policy "Allow users to enable online speech recognition services" (`AllowInputPersonalization` under `HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization`, from `Globalization.admx`). In this corpus that policy value is written by the inking and typing tweak, not by this one.

#### Benefits
- Audio stops being sent to Microsoft's cloud speech service for this account.
- On-device dictation and voice typing keep working.
- Per-user setting, no elevation needed.

#### Drawbacks
- Dictation accuracy drops: cloud recognition is more accurate and covers more languages.
- Every account on the machine needs it applied separately.
- Voice features in apps built on the Microsoft cloud speech stack stop responding.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately; the Settings toggle reflects it at once.
- **Reverting**: "On" deletes the value so Windows returns to asking you, rather than recording a consent. System Default restores the snapshot, which puts back a 1 only if you had genuinely accepted before the first apply.

#### Interactions
- `disable_inking_typing_personalization` writes `AllowInputPersonalization` = 0, the machine-wide policy that disables online speech recognition for every account. With that tweak applied, this per-user value is moot and the Settings control is locked.
- `disable_voice_activation` covers wake-word activation, a separate always-listening surface this tweak does not touch.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that the stock state is value-absent, not 1, so the "On" option must delete the value; the shipped option does.
- **Confidence**: Microsoft-documented for the disable direction and for the consent-deferred position; the absent stock state follows from both.
- **Reasoning**: this was the highest-priority revert-safety finding in the category: a revert that writes 1 fabricates consent. The cross-cutting revert audit confirmed it on documentation alone, without needing a clean image. The key, value, type and 0 polarity survived every pass.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it unless you dictate regularly and value the accuracy. On a shared machine, apply it per account, or use the inking and typing tweak, which also sets the machine-wide speech policy.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.6 Speech, the `HasAccepted` = 0 instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (AllowInputPersonalization), the machine-wide online speech policy and the user-deferred default, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Cross-cutting revert audit (`_harmful-revert.md`): writing 1 on revert fabricates consent; documentation-backed (project record)

### Disable inking and typing personalization

`disable_inking_typing_personalization` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows collecting your typing and handwriting to build a personal language model, and locks online speech recognition off machine-wide.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `accepted_privacy` | registry | `HKCU\Software\Microsoft\Personalization\Settings`, value `AcceptedPrivacyPolicy`, `REG_DWORD` |
| `restrict_text` | registry | `HKCU\Software\Microsoft\InputPersonalization`, value `RestrictImplicitTextCollection`, `REG_DWORD` |
| `restrict_ink` | registry | `HKCU\Software\Microsoft\InputPersonalization`, value `RestrictImplicitInkCollection`, `REG_DWORD` |
| `harvest_contacts` | registry | `HKCU\Software\Microsoft\InputPersonalization\TrainedDataStore`, value `HarvestContacts`, `REG_DWORD` |
| `allow_input_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization`, value `AllowInputPersonalization`, `REG_DWORD` |

| Option | `accepted_privacy` | `restrict_text` | `restrict_ink` | `harvest_contacts` | `allow_input_policy` |
|---|---|---|---|---|---|
| Disabled | `0` | `1` | `1` | `0` | `0` |

This is a toggle: the other position is System Default, which restores the snapshot of all five values. There is no authored "Enabled" option because `AcceptedPrivacyPolicy` and `HarvestContacts` are consent-derived (they depend on what the user agreed to), not fixed Windows defaults. `AllowInputPersonalization` is absent on a stock machine.

The four HKCU effects run in-process as the interactive user even though the tweak's floor is admin (the HKLM policy needs it), so they land in your own hive.

#### How it works

Microsoft documents `RestrictImplicitTextCollection` = 1 and `RestrictImplicitInkCollection` = 1 under `HKEY_CURRENT_USER\Software\Microsoft\InputPersonalization` as the registry form of turning off "Improve inking and typing" (Settings > Privacy and security > Inking and typing personalization). With them set, typed and handwritten text stops being sampled into the local personal dictionary and language model. `AcceptedPrivacyPolicy` (the personalization consent flag) and `HarvestContacts` (adding contact names to the local dictionary) are real values used by the same subsystem, but they are not Microsoft-documented; they rest on community references.

The fifth effect is not an inking control. `AllowInputPersonalization` under `HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization` is the Group Policy "Allow users to enable online speech recognition services", under Control Panel > Regional and Language Options, defined in `Globalization.admx`, and it is Policy CSP `Privacy/AllowInputPersonalization`. The key and hive are correct for that policy (machine class). Setting it to 0 disables cloud speech recognition for every account on the machine and locks the Settings control. The genuine machine-wide inking policy is a different one, `AllowLinguisticDataCollection` ("Improve inking and typing recognition", `TextInput.admx`, at `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\TextInput`), which this tweak does not write.

#### Benefits
- Typed and handwritten text stops being sampled into the personalization model.
- Contact names stop being added to the local dictionary.
- Online speech recognition is locked off for every account by the machine policy.
- Nothing is deleted; collection simply stops.

#### Drawbacks
- Autocorrect and text prediction degrade over time without new samples.
- Pen input recognition improves less on your own handwriting.
- `AcceptedPrivacyPolicy` and `HarvestContacts` are community-sourced, not Microsoft-documented.
- What has already been learned is not erased.
- Side effect that the name does not suggest: cloud dictation stops working machine-wide because of the speech policy.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and all supported Windows 10 versions including LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: System Default restores all five values from the snapshot, including deleting `AllowInputPersonalization` if it was absent before.

#### Interactions
- `disable_online_speech_recognition` sets the per-user speech consent; this tweak's `AllowInputPersonalization` = 0 is the machine-wide lock for the same feature and overrides it.
- `disable_diagnostic_data`: Optional diagnostic data includes inking samples, which the Required cap also stops sending.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The two `RestrictImplicit*` values are correct; the corrections were that `AllowInputPersonalization` is the online speech recognition policy (from `Globalization.admx`), not an inking control, and that the consent-derived values cannot have an authored stock default.
- **Confidence**: Microsoft-documented for the two `RestrictImplicit*` values and for what `AllowInputPersonalization` controls; community-corroborated for `AcceptedPrivacyPolicy` and `HarvestContacts`.
- **Reasoning**: the adversarial pass traced `AllowInputPersonalization` through Policy CSP to its ADMX and found it governs speech; the adjudication confirmed it and said either to move it to the speech tweak or to replace it with `AllowLinguisticDataCollection`, and that the choice must be deliberate. The tweak keeps the speech policy here and says so. The consent-derived defaults are handled by making the tweak a single-option toggle.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you type or write anything sensitive and do not lean on Windows text prediction or cloud dictation. Leave it alone if you use pen input heavily and want recognition to keep improving, or if anyone on the machine relies on online dictation.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.21 Inking and Typing, the `RestrictImplicit*` instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (AllowInputPersonalization), the online speech recognition policy, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Shipped `C:\Windows\PolicyDefinitions\Globalization.admx` (policy `AllowInputPersonalization`) and `TextInput.admx` (policy `AllowLinguisticDataCollection`) on build 26100 (tier A, shipped ADMX)

### Turn off the advertising ID

`disable_advertising_id` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off the per-user advertising ID that Store apps use to track you across sessions.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `adv_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo`, value `DisabledByGroupPolicy`, `REG_DWORD` |
| `adv_machine_enabled` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo`, value `Enabled`, `REG_DWORD` |
| `adv_enabled` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo`, value `Enabled`, `REG_DWORD` |

| Option | `adv_policy` | `adv_machine_enabled` | `adv_enabled` |
|---|---|---|---|
| Disabled | `1` | `0` | `0` |

This is a toggle: the other position is System Default, which restores the snapshot. The research gives the stock state as `DisabledByGroupPolicy` absent and both `Enabled` values at 1; the tweak authors a single option, so the way back is the snapshot. The HKCU effect runs as the interactive user.

#### How it works

Microsoft documents exactly two values for turning off the advertising ID, both in HKLM: `Enabled` = 0 under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo`, and `DisabledByGroupPolicy` = 1 under `HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo`, the Group Policy Computer Configuration > Administrative Templates > System > User Profiles > "Turn off the advertising ID" (`UserProfiles.admx`, machine class). Policy CSP `Privacy/DisableAdvertisingId` is Device-scoped only. With the policy set, the advertising ID is no longer issued to apps, the existing ID is reset, and the Settings > Privacy and security > General toggle is greyed out.

The third effect, `Enabled` under the per-user `AdvertisingInfo` key, is the value the Settings toggle itself writes. No Microsoft source states that the HKCU value is inert, and the advertising ID is genuinely a per-user identity, so it is written as a per-user supplement alongside the two documented machine values.

#### Benefits
- Store apps lose the stable identifier used to correlate you across sessions.
- The Settings toggle is locked and cannot be turned back on by a user.
- The machine-level values cover profiles created later.

#### Drawbacks
- Ads remain: this makes advertising less targeted, it does not block or reduce it.
- A small number of Store apps that expect an advertising ID can behave oddly.
- Users cannot change the setting themselves while it is applied.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and all supported Windows 10 versions including LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: System Default removes the policy value (if it was absent before) and restores the previous machine and per-user `Enabled` values from the snapshot.

#### Interactions
- `disable_tailored_experiences` covers a different personalization channel (diagnostic-data-driven tips and ads); the two are complementary.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that the tweak must also write the Microsoft-documented `HKLM\...\CurrentVersion\AdvertisingInfo\Enabled` = 0; it does.
- **Confidence**: Microsoft-documented for both HKLM values; the HKCU value is the Settings backing and is kept as a supplement.
- **Reasoning**: the adjudication record found the "HKCU is ignored" claim partly confirmed at most: nothing documents it as inert, so it was not removed, and the missing documented machine value was added. The hive audit confirmed `DisabledByGroupPolicy` is machine class in HKLM.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on every machine. There is no functional cost and the identifier exists only to serve advertisers. Skip it only if you are debugging a Store app that depends on the ID.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.1 General, both HKLM values, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (DisableAdvertisingId), Device scope, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Adjudication record (`_verification.md`, privacy claim 1, partly confirmed) (project record)

### Turn off tailored experiences

`disable_tailored_experiences` · Switch · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows using your diagnostic data to target the tips, ads and recommendations it shows you.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `tailored_user` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Privacy`, value `TailoredExperiencesWithDiagnosticDataEnabled`, `REG_DWORD` |
| `tailored_policy` | registry | `HKCU\Software\Policies\Microsoft\Windows\CloudContent`, value `DisableTailoredExperiencesWithDiagnosticData`, `REG_DWORD` |

| Option | `tailored_user` | `tailored_policy` |
|---|---|---|
| Off | `0` | `1` |

This is a toggle: the other position is System Default, which restores the snapshot. The policy value is absent on a stock machine; the Settings value is consent-derived (it reflects the user's OOBE answer), so no "On" option is authored.

#### How it works

Two related but separate knobs. `DisableTailoredExperiencesWithDiagnosticData` is the Group Policy "Do not use diagnostic data for tailored experiences". The shipped `CloudContent.admx` on 26100 declares it `class="User"` at `Software\Policies\Microsoft\Windows\CloudContent`, and Policy CSP `Experience/AllowTailoredExperiencesWithDiagnosticData` is User-scoped with Device explicitly not supported and a Group Policy location under User Configuration. So the per-user policy hive (`HKCU\Software\Policies`) is the only place the Group Policy engine writes and refreshes it, and that is where the tweak writes it. With it set, Windows stops personalising lock screen content, tips and consumer feature suggestions using your diagnostic data, and the Settings toggle follows the policy.

`TailoredExperiencesWithDiagnosticDataEnabled` under `CurrentVersion\Privacy` is the value behind the Settings > Privacy and security > Diagnostics and feedback > Tailored experiences toggle. It is undocumented and is not a policy value; it is written as well so the UI state agrees with the policy.

Microsoft documents this policy as depending on Windows Spotlight being allowed; with Spotlight fully disabled, there is nothing left for it to tailor.

#### Benefits
- Recommendations stop being shaped by what the device reports.
- The policy pins the setting and the Settings value matches, so the UI agrees.
- Nothing stops working; only the targeting changes.
- Per-user, so no elevation is needed.

#### Drawbacks
- Suggestions do not stop; they become generic.
- Moot when Windows Spotlight is fully disabled.
- User-scoped: each account needs it applied separately.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: System Default removes the policy value (if absent before) and restores the previous Settings value from the snapshot.

#### Interactions
- `disable_windows_spotlight_all` (Debloat, "Turn off all Windows Spotlight features") disables Spotlight wholesale, which makes this tweak moot. It writes a different value under the same per-user `CloudContent` policy key.
- `disable_tips_and_suggestions` and `disable_consumer_features` write other `CloudContent` policy values in HKLM; no overlap in values.
- `disable_diagnostic_data` reduces the diagnostic data that tailoring would use.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was the hive: the policy is `class="User"`, so it belongs under `HKCU\Software\Policies`, which is where the tweak writes it; and the Settings value is consent-derived, so no literal default is authored.
- **Confidence**: Microsoft-documented for the policy (Policy CSP and shipped ADMX); the Settings value is undocumented.
- **Reasoning**: this was the only wrong-hive finding in the corpus-wide policy hive audit. The adjudication confirmed the scope and deliberately left open whether an HKLM copy would be inert (not documented either way); the tweak avoids the question by writing the correct hive. The two knobs were checked as distinct and both kept.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any personal machine; it is a pure personalisation opt-out with no cost. Skip it only if you find the tailored suggestions useful.

#### Sources
1. Policy CSP - Experience (AllowTailoredExperiencesWithDiagnosticData), User scope only, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100, `class="User"` (tier A, shipped ADMX)
3. Manage connections from Windows components to Microsoft services, section 25 Personalized Experiences, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
4. Corpus-wide policy hive audit (`_policy-hive-audit.md`), the wrong-hive finding and its fix (project record)

### Turn off location tracking

`disable_location_tracking` · Switch · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off the Windows location platform so neither Windows nor any app can get a position fix.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `location_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors`, value `DisableLocation`, `REG_DWORD` |
| `location_apps_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`, value `LetAppsAccessLocation`, `REG_DWORD` |
| `location_consent` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location`, value `Value`, `REG_SZ` |

| Option | `location_policy` | `location_apps_policy` | `location_consent` |
|---|---|---|---|
| Off | `1` | `2` (Force Deny) | `"Deny"` |

This is a toggle: the other position is System Default, which restores the snapshot. The stock state is both policy values absent and the consent store at `"Allow"`, but the consent value reflects the user's choice, so no "On" option is authored.

#### How it works

Microsoft documents `DisableLocation` exactly: "Create a REG_DWORD registry setting named DisableLocation in HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows\LocationAndSensors with a value of 1 (one)", equivalent to the Group Policy "Turn off location". In `Sensors.admx` there are two definitions on that key, one user-class and one machine-class; the HKLM write hits the machine definition and applies system-wide. With it set, the location service stops resolving positions for Windows and for apps, and Settings > Privacy and security > Location is greyed out.

Microsoft's documented companion for blocking apps specifically is `LetAppsAccessLocation` = 2 (Force Deny) under `AppPrivacy`, Policy CSP `Privacy/LetAppsAccessLocation`, with 0 = user in control, 1 = Force Allow, 2 = Force Deny. The tweak writes both.

The third value is the device-level consent record in the Capability Access Manager's ConsentStore, accepted strings `Allow` and `Deny`. It is real and widely used but is not Microsoft-documented (community-sourced). A Microsoft Q&A report on 24H2 suggests that the consent-store write alone may not flip the UI state; here it is backed by the two policy values, so the outcome does not depend on it.

#### Benefits
- Device-level off switch, not a per-app permission.
- Covers every account without per-user configuration.
- Apps cannot request a position they can no longer obtain.

#### Drawbacks
- Automatic time zone fails silently, so the clock can be wrong after travel.
- Find My Device stops locating the machine regardless of its own setting.
- Weather, Maps and other location-aware apps lose their position and fall back to manual entry.
- With the policy present, a user cannot re-enable location from Settings.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and all supported Windows 10 versions including LTSC 2021.
- **Takes effect**: immediately; the Settings page needs reopening to redraw.
- **Reverting**: System Default removes the policy values that were absent before and restores the previous consent value from the snapshot.

#### Interactions
- `disable_find_my_device`: disabling location already breaks Find My Device, so with this applied that tweak is redundant.
- `disable_geolocation` (Services, "Disable Geolocation Service (lfsvc)") stops the service that provides location; complementary to this policy lock.
- `disable_app_diagnostics` and `disable_voice_activation` write other values under the same `AppPrivacy` policy key; `disable_background_apps` (Performance) does too. Each owns a different value.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that Microsoft's documented pair is `DisableLocation` = 1 plus `LetAppsAccessLocation` = 2, and the ConsentStore write is community-sourced only; the tweak writes both documented values and keeps the consent write as a supplement.
- **Confidence**: Microsoft-documented for both policy values; the consent-store value is community-corroborated.
- **Reasoning**: the adversarial pass found the documented app-access policy missing and flagged the consent write as tier C; both are addressed. The hive audit confirmed the HKLM write of `DisableLocation` hits the machine-class definition. Open question: whether the consent-store write alone changes the 24H2 Settings state (a tier D report only).
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a desktop that never needs a position fix. Do not apply it on a laptop you travel with, where automatic time zone and Find My Device are worth more than the privacy gain.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.2 Location, the `DisableLocation` instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Privacy (LetAppsAccessLocation, Force Deny = 2), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. Microsoft Q&A, "Location Services Disabled and Grayed out in Windows 11 24H2", cited only for the unresolved report that the consent-store write alone may not flip the UI state on 24H2, https://learn.microsoft.com/en-us/answers/questions/3930072/location-services-disabled-and-grayed-out-in-windo (tier D)

### Block app diagnostic access

`disable_app_diagnostics` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Blocks Store apps from reading diagnostic details about the other apps running on your PC.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `appdiag_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`, value `LetAppsGetDiagnosticInfo`, `REG_DWORD` |
| `appdiag_consent` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\appDiagnostics`, value `Value`, `REG_SZ` |

| Option | `appdiag_policy` | `appdiag_consent` |
|---|---|---|
| Denied | `2` (Force Deny) | `"Deny"` |

This is a toggle: the other position is System Default, which restores the snapshot. The stock state is the policy absent and the consent store at `"Allow"`, but the consent value reflects the user's choice, so no "Allowed" option is authored.

#### How it works

`LetAppsGetDiagnosticInfo` is Policy CSP `Privacy/LetAppsGetDiagnosticInfo`, "Let Windows apps get diagnostic information about other apps" (`AppPrivacy.admx`, machine class), with three settings: 0 = User is in control, 1 = Force Allow, 2 = Force Deny. With Force Deny, "Windows apps aren't allowed to get diagnostic information about other apps and employees in your organization can't change it". The App Diagnostics capability exposes process, package and user-name information about other running apps; the policy governs Windows (UWP / packaged) apps only. Win32 processes are not subject to it, so Task Manager, Process Explorer and any classic tool still see everything. Microsoft notes that an app open when the policy is applied must be restarted for it to take effect.

The ConsentStore `Value` is the device-level consent record read by the Capability Access Manager. It is used in practice but is not Microsoft-documented (community-sourced).

#### Benefits
- Packaged apps can no longer enumerate what else you run.
- The capability also exposes the account name, which is protected too.
- With Force Deny, no user or app can grant the capability back.

#### Drawbacks
- Packaged apps only: Win32 processes are completely unaffected.
- Store-delivered diagnostic, accessibility or monitoring apps lose their data.
- Apps already running keep their old state until relaunched.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately for new app launches; running apps must be restarted.
- **Reverting**: System Default removes the policy (if absent before) and restores the previous consent value from the snapshot.

#### Interactions
- `disable_location_tracking`, `disable_voice_activation` and `disable_background_apps` (Performance) write other values under the same `AppPrivacy` key; each owns a different value.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was scope: the policy governs packaged apps only, not "apps" in general.
- **Confidence**: Microsoft-documented for the policy; community-corroborated for the consent-store value.
- **Reasoning**: the adversarial pass attacked the claim that apps "can no longer inspect diagnostic details about other processes" and narrowed it to packaged apps; the key, value, enum and polarity survived. The hive audit confirmed a machine-class policy in HKLM.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine where you install Store apps you do not fully trust. Skip it if you rely on a packaged monitoring or accessibility tool that needs this capability.

#### Sources
1. Policy CSP - Privacy (LetAppsGetDiagnosticInfo), the enum, Force Deny behaviour and restart note, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
2. Manage connections from Windows components to Microsoft services, sections 18.2 and 18.15, the AppPrivacy Force Deny value 2, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### Turn off Find My Device

`disable_find_my_device` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows periodically reporting your device's location to your Microsoft account.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `fmd` | registry | `HKLM\SOFTWARE\Policies\Microsoft\FindMyDevice`, value `AllowFindMyDevice`, `REG_DWORD` |

| Option | `fmd` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default appears if the value holds 1 (explicitly allowed); selecting it restores the snapshot. The stock Windows state is value-absent ("On").

#### How it works

Microsoft documents it exactly: "You can also create a new REG_DWORD registry setting HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\FindMyDevice\AllowFindMyDevice to 0 (zero)", equivalent to disabling Computer Configuration > Administrative Templates > Windows Components > Find My Device > "Turn On/Off Find My Device" (`FindMy.admx`, machine class), and Policy CSP `Experience/AllowFindMyDevice`. With it off, Windows stops the periodic location reporting that lets you locate the device from account.microsoft.com, and the Settings > Privacy and security > Find my device control is locked. The feature only works with a Microsoft account and a working location service in the first place.

#### Benefits
- A recurring outbound location report tied to your account ends.
- The policy covers every account on the device.
- Deleting the value restores the feature exactly.

#### Drawbacks
- You cannot locate or remotely lock a lost or stolen device from your account.
- For a portable machine this is a security regression that costs more than it gains.
- Redundant if the location platform is already disabled.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "On" deletes the value, restoring the Windows default.

#### Interactions
- `disable_location_tracking` already breaks Find My Device by turning off the location platform.
- `disable_geolocation` (Services) stops the location service the feature depends on.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented: the exact key, value and data are published by Microsoft.
- **Reasoning**: key, value, type, polarity and absent default all match Microsoft's instruction; the hive audit confirmed a machine-class policy in HKLM. The medium risk reflects the lost-device consequence, not any uncertainty about the mechanism.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a desktop that never leaves the house. Do not apply it on a laptop or tablet: being able to find or lock a stolen device is worth more than this privacy gain.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 5 Find My Device, the exact registry instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP - Experience (AllowFindMyDevice), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)

### Disable settings sync

`disable_settings_sync` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops your settings, themes and saved passwords being copied to your Microsoft account.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `disable_sync` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync`, value `DisableSettingSync`, `REG_DWORD` |
| `disable_sync_override` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync`, value `DisableSettingSyncUserOverride`, `REG_DWORD` |

| Option | `disable_sync` | `disable_sync_override` |
|---|---|---|
| Disabled | `2` | `1` |
| Enabled | `absent` | `absent` |

System Default appears when the pair matches neither row (for example `DisableSettingSync` = 2 with the user override allowed); selecting it restores the snapshot. The stock Windows state is both values absent.

#### How it works

Microsoft documents this pair verbatim: "Create a REG_DWORD registry setting named DisableSettingSync in HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows\SettingSync with a value of 2 (two) and another named DisableSettingSyncUserOverride in the same key with a value of 1 (one)". That is the Group Policy "Do not sync" (`SettingSync.admx`, machine class) with the "Allow users to turn syncing on" checkbox left unchecked. The non-obvious 2 is correct: in this policy 2 means disabled, and 0 does not. The override value removes the user's ability to turn sync back on from Settings.

With both set, settings roaming stops: settings, themes, language preferences and saved passwords stay on this PC. On Windows 11 the user-facing surface for this is Windows Backup, whose scope overlaps legacy sync but is not identical; the Windows Backup and cloud restore pipeline is governed by a separate value in the same key (see Interactions).

#### Benefits
- Settings, themes, language preferences and saved passwords do not roam.
- The override value locks the Settings control.
- One machine policy covers every account.

#### Drawbacks
- A second Windows device no longer inherits your setup.
- Users see a greyed-out sync section until you revert.
- On Windows 11 the Windows Backup surface overlaps but is not identical to legacy sync.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 8.1 through Windows 10, so LTSC 2021 is covered.
- **Takes effect**: after reboot. Microsoft does not document a reboot requirement; the tweak declares one as a conservative choice for a policy that affects sign-in-time state.
- **Reverting**: "Enabled" deletes both values, restoring sync and user control.

#### Interactions
- `disable_windows_backup` writes `EnableWindowsBackup` in the same `SettingSync` key; different value, different feature (Windows Backup and cloud restore rather than legacy roaming sync). The two are complementary.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the reboot flag was reviewed and accepted as defensible though undocumented.
- **Confidence**: Microsoft-documented: the exact pair and data are published by Microsoft and match the shipped ADMX.
- **Reasoning**: the counter-intuitive value 2 was attacked and confirmed against Microsoft's text; the hive audit confirmed both values are machine class in HKLM.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a single-device setup, or on any machine where saved passwords should not leave the box. Skip it if you run several Windows devices and want your configuration to follow you.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 21 Sync your settings, the exact pair, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\SettingSync.admx` on build 26100 (tier A, shipped ADMX)

### Disable Windows Error Reporting

`disable_error_reporting` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops crash and error reports being generated and sent to Microsoft.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `wer_disabled` | registry | `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting`, value `Disabled`, `REG_DWORD` |
| `wer_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting`, value `Disabled`, `REG_DWORD` |

| Option | `wer_disabled` | `wer_policy` |
|---|---|---|
| Disabled | `1` | `1` |
| Enabled | `absent` | `absent` |

System Default appears when the two values disagree or hold 0; selecting it restores the snapshot. The stock Windows state is both values absent (WER enabled).

#### How it works

Microsoft's WER Settings reference states that WER settings live under `HKEY_CURRENT_USER\Software\Microsoft\Windows\Windows Error Reporting` or the HKLM equivalent, and documents `Disabled` as `REG_DWORD` with 0 = Enabled (the default) and 1 = Disabled. The first effect writes exactly that at machine level. The second writes the same value name under the policy key, the Group Policy "Disable Windows Error Reporting": `ErrorReporting.admx` defines it twice on `SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting`, once user-class (`WerDisable_1`) and once machine-class (`WerDisable_2`), and the HKLM write hits the machine definition.

With WER disabled, it stops queueing reports, stops uploading them, and stops writing the user-mode artefacts that Reliability Monitor reads: the WER report queue, user-mode `LocalDumps` and Reliability Monitor entries all go quiet. WER reports can carry heap contents and file paths, which is the privacy motive. Kernel crash dumps (`MEMORY.DMP`) are governed by `HKLM\SYSTEM\CurrentControlSet\Control\CrashControl` and are not affected.

#### Benefits
- Crash uploads, which can carry heap contents and file paths, stop leaving the machine.
- Crash-report prompts and background upload attempts stop.
- Both the product value and the policy value are set.

#### Drawbacks
- Local diagnostics are lost: the WER queue, user-mode `LocalDumps` and Reliability Monitor entries, which support processes usually ask for first.
- At the Required diagnostic data level crash dumps are not sent anyway, so the extra privacy is smaller than it looks.
- Some third-party crash handlers query WER state and behave differently.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows Vista through Windows 10, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes both values, restoring the Windows default. Reports that were never generated while disabled cannot be recovered.

#### Interactions
- `disable_wer_service` disables the `WerSvc` service itself; together with this tweak, reports are neither permitted nor processed.
- `task_wer_queuereporting` (Services, "Disable Windows Error Reporting QueueReporting task") disables the queued-report upload task.
- `disable_diagnostic_data`: at Required, crash dumps are already not sent.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections were that kernel crash dumps are not affected (only the WER queue, user-mode `LocalDumps` and Reliability Monitor entries), and that the policy-key path needed confirming against `ErrorReporting.admx`.
- **Confidence**: Microsoft-documented for the product value (WER Settings); the policy path and class come from the shipped `ErrorReporting.admx` as recorded in the hive audit.
- **Reasoning**: the adversarial pass narrowed the dump-suppression claim. The category research leaves the policy path unconfirmed at tier A, while the corpus-wide hive audit, parsing the shipped `ErrorReporting.admx`, records `WerDisable_1` (user) and `WerDisable_2` (machine) on exactly this key; this page follows the hive audit.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a stable machine you do not troubleshoot. Do not apply it while you are chasing crashes or instability; you would be throwing away your own first diagnostic.

#### Sources
1. WER Settings, the `Disabled` value, its locations and meanings, https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings (tier A)
2. Configure Windows diagnostic data in your organization, crash dump behaviour by diagnostic level, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
3. Corpus-wide policy hive audit (`_policy-hive-audit.md`), `ErrorReporting.admx` `WerDisable_1` (user) and `WerDisable_2` (machine) on the policy key (project record, from shipped ADMX)

### Turn off suggested content in Settings

`disable_suggested_content_settings` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Removes the promotional cards Windows shows inside the Settings app.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `content_338393` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `SubscribedContent-338393Enabled`, `REG_DWORD` |
| `content_353694` | registry | same key, value `SubscribedContent-353694Enabled`, `REG_DWORD` |
| `content_353696` | registry | same key, value `SubscribedContent-353696Enabled`, `REG_DWORD` |
| `content_353698` | registry | same key, value `SubscribedContent-353698Enabled`, `REG_DWORD` |

| Option | `content_338393` | `content_353694` | `content_353696` | `content_353698` |
|---|---|---|---|---|
| Off | `0` | `0` | `0` | `0` |
| On | `absent` | `absent` | `absent` | `absent` |

System Default appears when the four values are mixed or hold 1 (for example after toggling the Settings control, which writes explicit values); selecting it restores the snapshot. The stock state is all four absent: none exists in the shipped Default user hive on build 26100.

#### How it works

Content Delivery Manager is the per-user Windows component that fetches and schedules promotional and suggestion content. It keys each content slot by a numeric subscription id and composes the value name `SubscribedContent-<id>Enabled` at runtime; 1 means the slot is shown and 0 means it is hidden. The Settings app asks Content Delivery Manager for these slots to render the suggested-content cards on its Home, System and Personalization pages. Setting the four slots to 0 stops them being requested and rendered for this account. This is the registry backing of "Show me suggested content in the Settings app".

Microsoft publishes no mapping from subscription ids to surfaces and structurally cannot: the prefix is a literal inside the shipped `ContentDeliveryManager.Utilities.dll` and `ContentDeliveryManager.Background.dll`, and the numeric id is appended at runtime. The grouping rests on four independent community sources (different authors, countries and eras) that agree on key, value names, type and semantics for the first three ids. The fourth id, 353698, is written by Win11Debloat in the same block under the same comment, and the bare id `353698` is present in `ContentDeliveryManager.Background.dll`, `ContentDeliveryManager.Utilities.dll` and `Windows.Services.TargetedContent.dll` on 26100. The full value name is never a literal in any binary for any of these slots (including the known-good ones), so presence of the bare id is the right test. That the Settings suggested-content surface exists and is separately controllable is confirmed at tier A by the shipped `DisableWindowsSpotlightOnSettings` policy in `CloudContent.admx`, which is broader and is not what this tweak uses.

#### Benefits
- Promotional cards and app pitches disappear from Settings pages.
- The content subscriptions stop being requested for this account.
- Per-user setting, no elevation needed.

#### Drawbacks
- Cosmetic: your privacy posture does not change, only what Settings displays.
- The subscription ids are undocumented by Microsoft; the grouping rests on independent community sources and binary evidence.
- Major Windows updates are widely reported to reset these values.
- If an id were inert, writing 0 to it would do nothing; the failure mode is nil.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1803 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately; reopen Settings to see it.
- **Reverting**: "On" deletes the four values, which is the state a fresh profile is in.

#### Interactions
- Other Content Delivery Manager tweaks own different values under the same key: `disable_start_app_suggestions`, `disable_lockscreen_spotlight_ads`, `disable_tips_and_suggestions`, and in Debloat `disable_start_suggestions`, `disable_auto_install_sponsored_apps` and `disable_welcome_experience`. No value is shared.
- `disable_windows_spotlight_all` (Debloat) turns off Spotlight features wholesale; its surfaces overlap without touching these values.
- `disable_settings_account_ads` (Debloat, "Turn off account upsell cards in Settings") covers a different Settings card surface through a CloudContent policy.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (community-corroborated for the id mapping, Microsoft-documented for the surface). The corrections were that the stock state is value-absent (none of the values is in the Default user hive), and that the set includes a fourth slot, 353698.
- **Confidence**: Community-corroborated. No Microsoft mapping exists; four independent community sources plus shipped binary evidence support it.
- **Reasoning**: the adversarial pass checked the Default hive for the stock state and the shipped binaries for the ids. It also caught a false attribution (privacy.sexy, Sophia Script and WinUtil do not contain 353698); support for that slot is Win11Debloat plus the binaries, judged adequate for an additive value with no failure mode. The absent-default reading rests partly on a heavily modified validation machine; the Default-hive read is the stronger evidence and privacy.sexy's default-state annotation agrees.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it; there is no downside beyond losing the advertisements. Skip it only if you want Microsoft's Settings suggestions.

#### Sources
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100: none of the four values present (tier A, primary observation, with the clean-image caveat)
2. Shipped `CloudContent.admx` on build 26100, policy `DisableWindowsSpotlightOnSettings`, class User, confirming the surface (tier A, shipped ADMX)
3. String evidence on build 26100: bare id `353698` in `ContentDeliveryManager.Background.dll`, `ContentDeliveryManager.Utilities.dll` and `Windows.Services.TargetedContent.dll` (tier A, product artifact)
4. privacy.sexy, "Disable suggested content in Settings app", the three original values as REG_DWORD 0 with `deleteOnRevert` and the annotation "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)", https://github.com/undergroundwires/privacy.sexy (tier C)
5. Win11Debloat, `Regfiles/Disable_Windows_Suggestions.reg`, all four values under "Show me suggested content in the Settings app", https://github.com/Raphire/Win11Debloat (tier C)
6. Sophia Script for Windows 11, function `SettingsSuggestedContent`, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
7. Brink, ElevenForum tutorial 3791 (and TenForums tutorial 100541, same author), https://www.elevenforum.com/t/enable-or-disable-suggested-content-in-settings-in-windows-11.3791/ (tier C)
8. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, listing all four ids, https://github.com/Biswa96/WinLight (tier C)
9. Manage connections from Windows components to Microsoft services, section 25 Personalized Experiences, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### Turn off Start menu app suggestions

`disable_start_app_suggestions` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops promoted apps appearing among your Start menu entries.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `pane_suggestions` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `SystemPaneSuggestionsEnabled`, `REG_DWORD` |

| Option | `pane_suggestions` |
|---|---|
| Off | `0` |
| On | `1` |

System Default appears only if the value is missing or holds something other than 0 or 1; selecting it restores the snapshot. The stock Windows state is a literal 1: unlike the `SubscribedContent-*` values, this value is genuinely seeded, present with data 1 in the shipped Default user hive on build 26100.

#### How it works

`SystemPaneSuggestionsEnabled` is the long-standing per-user registry backing of "Occasionally show suggestions in Start". Content Delivery Manager reads it and, when it is 0, stops injecting promoted app tiles into the Start app list. A string scan of 4051 shipped `System32` and `SystemApps` modules on 26100 found the value name in `ContentDeliveryManager.Background.dll`, so it is still a live control, not a legacy leftover.

It does not cover every Start suggestion surface. On Windows 10 the Settings toggle also writes a companion value, `SubscribedContent-338388Enabled` (Brink records that `SystemPaneSuggestionsEnabled` "has changed to" that value), which this tweak does not write. On Windows 11 the Recommended section and its "Show recommendations for tips, app promotions and more" wording are different surfaces again, controlled by `Start_IrisRecommendations` (read by `StartTileData.dll`) and by the `HideRecommendedSection` policy in `StartMenu.admx`. Sophia Script, which tracks 25H2 and later, has dropped both `SystemPaneSuggestionsEnabled` and 338388 in favour of `Start_IrisRecommendations`.

#### Benefits
- Store pitches stop appearing in the Start app list.
- The value is present in the shipped Content Delivery Manager binary on 24H2, so it is a live control.
- Per-user setting, no elevation needed.

#### Drawbacks
- Does not clear the Windows 11 Recommended section, which needs its own control.
- Partial on Windows 10, where the companion 338388 value is not written by this tweak.
- Major updates are widely reported to restore the default.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately in most cases; signing out guarantees it.
- **Reverting**: "On" writes the value back to 1, which is how Windows ships it.

#### Interactions
- `disable_start_suggestions` (Debloat, "Turn off Start menu app promotions") writes the companion `SubscribedContent-338388Enabled`; apply both for full coverage of the Start suggestion slot.
- `disable_start_recommendations` (Interface, "Turn off promoted recommendations in Start") writes `Start_IrisRecommendations`, and `disable_start_recommended_section` (Interface, "Hide the Recommended section in Start") sets `HideRecommendedSection`; those are the Windows 11 Recommended-section controls.
- Other Content Delivery Manager tweaks own different values under the same key.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (community-corroborated). The correction was coverage: the value alone does not cover the Windows 10 companion slot or the Windows 11 Recommended section; the literal 1 stock default was confirmed correct and must not be changed to absent.
- **Confidence**: Community-corroborated for the value's meaning; the stock default and the value's continued use are established from the shipped Default hive and shipped binaries.
- **Reasoning**: the adversarial pass tested whether the 1 default was an assumption (it is not: the Default hive seeds it) and whether the value is still read on 24H2 (it is). It flagged the missing companions; those surfaces are covered by the separate Debloat and Interface tweaks named above rather than by this one. The research also noted overlap with `disable_start_suggestions` as a merge candidate.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine; there is no cost. For a clean Windows 11 Start menu, pair it with the Start promotions tweak in Debloat and the Recommended-section tweaks in Interface.

#### Sources
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100: `SystemPaneSuggestionsEnabled` present as `REG_DWORD` 1 (primary observation, Default-hive read; clean-image confirmation pending)
2. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100: `SystemPaneSuggestionsEnabled` in `ContentDeliveryManager.Background.dll`, `Start_IrisRecommendations` in `StartTileData.dll` (tier A, product artifact)
3. Shipped `StartMenu.admx` on build 26100, policy `HideRecommendedSection` (tier A, shipped ADMX)
4. Brink, "Turn On or Off App Suggestions in Start in Windows 10", TenForums tutorial 24117, https://www.tenforums.com/tutorials/24117-turn-off-app-suggestions-start-windows-10-a.html (tier C)
5. Win11Debloat, `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C)
6. Disassembler0, Win10-Initial-Setup-Script, `DisableAppSuggestions`, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C)
7. hellzerg, Optimizer, `OptimizeHelper.cs`, https://github.com/hellzerg/optimizer (tier C)
8. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C)

### Turn off app launch tracking

`disable_app_launch_tracking` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows recording which applications you launch to rank Start and search results.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `track_progs` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `Start_TrackProgs`, `REG_DWORD` |

| Option | `track_progs` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default appears if the value holds 1 (which Settings writes when the toggle is flipped back on); selecting it restores the snapshot. The stock state is value-absent, which Windows treats the same as 1 (tracking on).

#### How it works

`Start_TrackProgs` is the value Microsoft names for Settings > Privacy and security > General > "Let Windows track app launches to improve Start and search results". Microsoft's instruction is to "Create a REG_DWORD registry setting named Start_TrackProgs" with 0, the phrasing used for values that do not exist by default. Explorer reads it on the next Start interaction; at 0 Windows stops accumulating the per-app launch counts behind the Most used list and part of search ranking. The value name must be exactly `Start_TrackProgs`, with that casing and underscore.

On build 26100 the value was absent from every hive loaded under `HKEY_USERS`, including a near-pristine second profile whose `Explorer\Advanced` key held a single value, so absent is the stock state. Absent and 1 behave identically. This data is local: it ranks your own Start and search results and is not sent anywhere by this setting.

#### Benefits
- The record of which apps you open stops being kept.
- Microsoft-documented value for the Settings toggle.
- Per-user setting, no elevation needed.

#### Drawbacks
- The Start menu Most used list stops updating.
- Search results are ordered less usefully for your own habits.
- The data never left the machine, so the privacy gain is against local snooping, not against Microsoft.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately; Explorer reads it on the next Start interaction.
- **Reverting**: "On" deletes the value, which is how a fresh profile ships.

#### Interactions
- Shares the `Explorer\Advanced` key with Interface tweaks such as `disable_start_recommendations` (`Start_IrisRecommendations`); different values.
- `hide_recently_added_apps` and similar Start layout tweaks (Interface) change what Start shows rather than what Windows records.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that the stock state is value-absent, not 1; the shipped "On" option deletes the value.
- **Confidence**: Microsoft-documented for the value and its meaning; the absent default is from a hive enumeration on 26100 plus Microsoft's "Create" wording.
- **Reasoning**: key, value, type, polarity and user elevation survived. The absent default rests partly on a heavily modified validation machine (the research asks for re-confirmation on a clean image), but Microsoft's "Create" instruction points the same way and absent and 1 behave identically, so nothing user-visible depends on it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if a shared or observed machine makes the local record of what you run worth removing. If you like the Most used list, skip it; the data does not leave your PC.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.1 General, the `Start_TrackProgs` instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Enumeration of every hive under `HKEY_USERS` on build 26100: value absent in all six (primary observation, with the clean-image caveat)

### Turn off lock screen ads and fun facts

`disable_lockscreen_spotlight_ads` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Keeps the Windows Spotlight lock screen pictures but strips the ads, tips and "fun facts" text off them.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `rotating_lock` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `RotatingLockScreenOverlayEnabled`, `REG_DWORD` |
| `content_338387` | registry | same key, value `SubscribedContent-338387Enabled`, `REG_DWORD` |

| Option | `rotating_lock` | `content_338387` |
|---|---|---|
| Off | `0` | `0` |
| On | `1` | `absent` |

System Default appears when the pair matches neither row (for example 338387 explicitly 1 after using the Settings toggle); selecting it restores the snapshot. The stock state is split: `RotatingLockScreenOverlayEnabled` is present with data 1 in the shipped Default user hive on 26100 (alongside `RotatingLockScreenEnabled` = 1), while `SubscribedContent-338387Enabled` is absent, like every other `SubscribedContent-*` value.

#### How it works

`RotatingLockScreenOverlayEnabled` is the per-user value behind "Get fun facts, tips, tricks and more on your lock screen" (Settings > Personalization > Lock screen, when the background is Windows Spotlight). A string scan of 4051 shipped modules on 26100 found it in `Windows.UI.Immersive.dll`, the component that renders the lock screen, and in `ContentDeliveryManager.Utilities.dll`. Subscription slot 338387 is the lock-screen tip and ad content subscription; four independent community sources pair the two values and agree on key, names, type and polarity. With both at 0, the Spotlight wallpaper rotation continues and the overlaid promotional text and tip cards stop being fetched and drawn.

The Microsoft-documented alternatives, the `DisableWindowsSpotlightFeatures` and `DisableCloudOptimizedContent` policies, also remove the Spotlight wallpapers, which is why this tweak uses the per-user values instead.

#### Benefits
- The daily Spotlight images keep working, unlike with the Spotlight policies.
- Promoted apps and offers stop appearing over the picture.
- Per-user setting, no elevation needed.

#### Drawbacks
- The "fun facts" and photo-location captions go with the ads.
- The 338387 slot has no Microsoft mapping and rests on community sources.
- Major updates are widely reported to restore these values.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later, so LTSC 2021 is covered. Only meaningful when the lock screen background is Windows Spotlight.
- **Takes effect**: at the next lock screen.
- **Reverting**: "On" writes the overlay value back to 1 and deletes the subscription value, which is how a fresh profile ships.

#### Interactions
- `disable_windows_spotlight_all` (Debloat) turns off Spotlight entirely, including the pictures; with it applied this tweak's surface is gone, although this tweak still reads its own values independently.
- `disable_tailored_experiences` changes whether lock screen content is personalised from diagnostic data.
- Other Content Delivery Manager tweaks own different values under the same key.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (community-corroborated). The correction was the split stock default: the overlay value is seeded with 1, the subscription value is absent; the shipped "On" option writes exactly that.
- **Confidence**: Community-corroborated for the meaning of the two values; the stock state and the renderer's use of the overlay value are established from the shipped Default hive and binaries.
- **Reasoning**: the adversarial pass read the Default hive and scanned shipped modules; both values survived, and the single "write 1 to both" revert was replaced by the split. The Default-hive read is the stronger evidence, but the research still asks for clean-image confirmation of the absent default.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you like the Spotlight photographs but not the advertising over them. If you do not want Spotlight at all, use the broader Spotlight tweak in Debloat instead.

#### Sources
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100: `RotatingLockScreenOverlayEnabled` present as `REG_DWORD` 1, `SubscribedContent-338387Enabled` absent (primary observation, Default-hive read; clean-image confirmation pending)
2. String scan of 4051 shipped modules on build 26100: `RotatingLockScreenOverlayEnabled` in `Windows.UI.Immersive.dll` and `ContentDeliveryManager.Utilities.dll` (tier A, product artifact)
3. Brink, "Enable or Disable Facts, Tips, and Tricks on Lock Screen in Windows 11", ElevenForum tutorial 7079, whose .reg files write both values and state "check (on - default)", https://www.elevenforum.com/t/enable-or-disable-facts-tips-and-tricks-on-lock-screen-in-windows-11.7079/ (tier C)
4. Win11Debloat, `Regfiles/Disable_Lockscreen_Tips.reg` and its undo file, https://github.com/Raphire/Win11Debloat (tier C)
5. hellzerg, Optimizer, `OptimizeHelper.cs`, https://github.com/hellzerg/optimizer (tier C)
6. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C)
7. Disassembler0, Win10-Initial-Setup-Script, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C)
8. Manage connections from Windows components to Microsoft services, section 25 Personalized Experiences, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### Turn off Windows tips and suggestions

`disable_tips_and_suggestions` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the tip, trick and suggestion notifications Windows pops while you work.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `soft_landing_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value `DisableSoftLanding`, `REG_DWORD` |
| `soft_landing_pref` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `SoftLandingEnabled`, `REG_DWORD` |
| `content_338389` | registry | same HKCU key, value `SubscribedContent-338389Enabled`, `REG_DWORD` |

| Option | `soft_landing_policy` | `soft_landing_pref` | `content_338389` |
|---|---|---|---|
| Off | `1` | `0` | `0` |

This is a toggle: the other position is System Default, which restores the snapshot. The policy is absent on a stock machine; the stock state of the two per-user values is not settled (`SubscribedContent-338389Enabled` is very likely absent on a fresh profile, like the other `SubscribedContent-*` values), so no "On" option is authored. The two HKCU effects run as the interactive user.

#### How it works

The policy half is Policy CSP `Experience/AllowWindowsTips`, which maps to Group Policy `DisableSoftLanding`, friendly name "Do not show Windows tips", Computer Configuration, key `Software\Policies\Microsoft\Windows\CloudContent`, `CloudContent.admx` (machine class). CSP Default Value 1 (tips on), 0 disables; in the registry, `DisableSoftLanding` = 1 turns tips off. It has a documented dependency on `AllowWindowsSpotlight` = 1.

The applicability is the important part: the Policy CSP editions row for `AllowWindowsTips` says it is not supported on Pro and does not list Home; it is honoured on Enterprise, Education and IoT Enterprise / IoT Enterprise LTSC only. On Home and Pro the policy write is silently ignored.

The two per-user values are the Content Delivery Manager backing of "Get tips, tricks, and suggestions as you use Windows" (Settings > System > Notifications > Additional settings); they are community-sourced. On Home and Pro they are what actually stops the tip toasts for your account.

#### Benefits
- The "Did you know" and "Try this" notifications stop.
- On supported editions the policy pins it machine-wide, and the per-user values cover everything else.
- Nothing depends on the tips.

#### Drawbacks
- The policy half is inert on Home and Pro.
- Occasional genuinely useful pointers to new features go with the promotions.
- The per-user values are widely reported to return after a major update.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later. The policy half is honoured only on Enterprise, Education and IoT Enterprise / IoT Enterprise LTSC (so it does work on LTSC 2021); the per-user half works on every edition.
- **Takes effect**: immediately.
- **Reverting**: System Default removes the policy value (if absent before) and restores the previous per-user values from the snapshot.

#### Interactions
- `disable_online_tips` stops the Settings app fetching help content; a different surface and different traffic.
- `disable_windows_spotlight_all` (Debloat) turns off Windows tips as part of all Spotlight features.
- `disable_consumer_features` and `disable_settings_account_ads` (Debloat) write other values under the same HKLM `CloudContent` policy key.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was the edition gate: `DisableSoftLanding` is honoured only on Enterprise, Education and IoT, so on Home and Pro only the per-user values act.
- **Confidence**: Microsoft-documented for the policy and its edition applicability; community-corroborated for the two per-user values.
- **Reasoning**: the adversarial pass read the Policy CSP editions row and found the SKU gate; the key, value and polarity survived and the hive audit confirmed a machine-class policy in HKLM. Open question: the stock state of the per-user values on a clean image.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if the notifications distract you. On Home or Pro, expect only the per-user half to work, which is still enough to stop the toasts for your account.

#### Sources
1. Policy CSP - Experience (AllowWindowsTips), mapping, default, Spotlight dependency and editions, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100, policy `DisableSoftLanding`, `class="Machine"` (tier A, shipped ADMX)
3. AskVG, "Registry Tweaks to Disable Ads, Suggestions and Tips in Windows 10", the per-user values, https://www.askvg.com/registry-tweaks-to-disable-ads-suggestions-and-tips-in-windows-10/ (tier C)

### Disable Windows consumer features

`disable_consumer_features` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off Microsoft consumer experiences (post-setup app installs, promoted tiles and membership nags) on the editions that honour it.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `consumer_features_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value `DisableWindowsConsumerFeatures`, `REG_DWORD` |

| Option | `consumer_features_policy` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` |

System Default appears only if the value holds something other than 1 or nothing (for example 0); selecting it restores the snapshot. The stock Windows state is value-absent.

#### How it works

Policy CSP `Experience/AllowWindowsConsumerFeatures` maps to Group Policy `DisableWindowsConsumerFeatures`, friendly name "Turn off Microsoft consumer experiences", Computer Configuration, key `Software\Policies\Microsoft\Windows\CloudContent`, `CloudContent.admx` (machine class). Microsoft describes the scope as "experiences that are typically for consumers only, such as Start suggestions, Membership notifications, Post-OOBE app install and redirect tiles". CSP Default Value 1 (allowed), most restricted 0; in the registry, 1 turns the experiences off. It has a documented dependency on `AllowWindowsSpotlight` = 1.

Editions: the policy is not supported on Pro and Home is not listed; it is honoured on Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC only. On Home and Pro this tweak writes the value and nothing reads it, so it does nothing at all there. The tweak has exactly one effect; it does not write the per-user `SilentInstalledAppsEnabled` value that stops sponsored app installs on Home and Pro.

#### Benefits
- On supported editions, the silent post-OOBE promoted app installs stop.
- Membership notifications and redirect tiles stop appearing.
- A policy value outlasts the per-user content settings that feature updates can reset.

#### Drawbacks
- Inert on Home and Pro, the two most common editions.
- On supported editions, legitimate Store recommendations go too.
- Depends on Windows Spotlight being allowed.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later, on Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC only; LTSC 2021 is an IoT Enterprise LTSC edition and is covered.
- **Takes effect**: after reboot.
- **Reverting**: "Enabled" deletes the policy value, restoring the Windows default.

#### Interactions
- `disable_auto_install_sponsored_apps` (Debloat, "Turn off auto-installed sponsored apps") writes the per-user `SilentInstalledAppsEnabled`, `PreInstalledAppsEnabled` and `OemPreInstalledAppsEnabled` values; that is the Home and Pro route to the same outcome.
- `disable_start_app_suggestions` and `disable_suggested_content_settings` are per-user suggestion controls that work on every edition.
- `disable_tips_and_suggestions` and `disable_settings_account_ads` (Debloat) write other values under the same `CloudContent` policy key.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that the tweak's single effect is the Enterprise-class policy only, so on Home and Pro it is inert; it does not write the per-user sponsored-app value.
- **Confidence**: Microsoft-documented: Policy CSP gives the mapping, scope, default, dependency and editions, and the shipped ADMX matches.
- **Reasoning**: the adversarial pass compared the described behaviour with the effects list and found the per-user value was never written; the tweak is described as policy-only. Key, value, polarity and the machine class survived. The reboot requirement is declared by the tweak; the research does not document one either way.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on Enterprise, Education or IoT. On Home or Pro it does nothing, so use the per-user Start, suggested-content and sponsored-app tweaks instead.

#### Sources
1. Policy CSP - Experience (AllowWindowsConsumerFeatures), mapping, scope, default, Spotlight dependency and editions, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100, `class="Machine"` (tier A, shipped ADMX)
3. Manage connections from Windows components to Microsoft services, section 25 Personalized Experiences, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### Turn off Microsoft Edge telemetry

`disable_edge_telemetry` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Edge using your browsing for personalization and stops it reporting your third-party searches.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `edge_personalization` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `PersonalizationReportingEnabled`, `REG_DWORD` |
| `edge_3pserp` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `Edge3PSerpTelemetryEnabled`, `REG_DWORD` |

| Option | `edge_personalization` | `edge_3pserp` |
|---|---|---|
| Off | `0` | `0` |
| On | `absent` | `absent` |

System Default appears when the two values are mixed or hold 1; selecting it restores the snapshot. The stock state is both absent (Edge defaults).

#### How it works

Both are Microsoft Edge browser policies from `MSEdge.admx`, read by Edge from `HKLM\SOFTWARE\Policies\Microsoft\Edge`. The ADMX defines them for both User and Machine; the hive audit found HKLM legitimate and the stronger of the two, since machine-scope Edge policy wins.

`PersonalizationReportingEnabled` = 0 (Edge 80 and later) prevents Microsoft collecting browsing history, favourites, collections and usage to personalise ads, search, news and Edge itself, and users cannot override it. Microsoft notes it "isn't available for child accounts or enterprise accounts". `Edge3PSerpTelemetryEnabled` = 0 (Edge 120 and later, with dynamic policy refresh) stops Edge capturing searches you run on Google or other third-party search engines.

This is not a full Edge telemetry off switch. The older `MetricsReportingEnabled` and `SendSiteInfoToImproveServices` policies carry Microsoft's banner "OBSOLETE: This policy is obsolete and doesn't work after Microsoft Edge version 88" (supported 77 to 88), so they are not written; they would appear "applied" to any registry check while doing nothing. On Windows 10 and 11 Edge's diagnostic data level follows the Windows diagnostic data setting when unconfigured, so the Windows `AllowTelemetry` tweak is the lever for that.

#### Benefits
- History and favourites stop feeding Microsoft's personalization.
- Third-party searches stop being captured, a surface no Windows-level telemetry tweak covers.
- Both are policies, so Edge's own settings follow them.

#### Drawbacks
- Inert on a machine without Edge.
- Edge's news feed, search suggestions and offers become less relevant.
- Edge diagnostic data still follows the Windows diagnostic data setting.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, wherever Edge is installed; Edge 80 or later for the personalization value and Edge 120 or later for the third-party search value.
- **Takes effect**: immediately for the search value (dynamic refresh); restart Edge for the personalization value.
- **Reverting**: "On" deletes both values, restoring Edge's defaults.

#### Interactions
- `disable_diagnostic_data` governs Edge's diagnostic level on Windows 10 and 11.
- Other Edge tweaks in the corpus (`disable_edge_first_run`, `disable_edge_startup_boost` and `disable_edge_sidebar` in Debloat, `disable_edge_ai_features` in AI) write other values under the same Edge policy key.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that `MetricsReportingEnabled` and `SendSiteInfoToImproveServices` are obsolete and do nothing after Edge 88, so only the two live policies are written, and that Edge's diagnostic level is tied to the Windows setting, not independent of it.
- **Confidence**: Microsoft-documented: each policy page on Microsoft Learn gives the value, meaning and supported versions.
- **Reasoning**: the adversarial pass and the adjudication both confirmed the obsolete banners. On a successor for Edge diagnostic control, the category research says `DiagnosticData` is scoped to Windows 7, Windows 8 and macOS, while the adjudication calls it the live replacement from Edge 111; either way this tweak does not write it and relies on the Windows setting.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use Edge at all, even occasionally. If Edge is not installed, skip it; there is nothing to configure.

#### Sources
1. Microsoft Edge policy: PersonalizationReportingEnabled, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/personalizationreportingenabled (tier A)
2. Microsoft Edge policy: Edge3PSerpTelemetryEnabled, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/edge3pserptelemetryenabled (tier A)
3. Microsoft Edge policy: MetricsReportingEnabled, carrying the OBSOLETE banner, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/metricsreportingenabled (tier A)
4. Microsoft Edge policy: SendSiteInfoToImproveServices, carrying the OBSOLETE banner, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/sendsiteinfotoimproveservices (tier A)

### Block website access to your language list

`disable_language_list_access` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows handing your full language list to web content, trimming one browser fingerprinting signal.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `lang_optout` | registry | `HKCU\Control Panel\International\User Profile`, value `HttpAcceptLanguageOptOut`, `REG_DWORD` |

| Option | `lang_optout` |
|---|---|
| Blocked | `1` |
| Allowed | `absent` |

System Default appears if the value holds 0 (Settings may write an explicit 0 when the toggle is turned back on); selecting it restores the snapshot. The stock state is value-absent; 0 and absent behave the same.

#### How it works

This is the documented registry form of turning off Settings > Privacy and security > General > "Let websites provide locally relevant content by accessing my language list". Microsoft's instruction is to "Create a new REG_DWORD registry setting named HttpAcceptLanguageOptOut in HKEY_CURRENT_USER\Control Panel\International\User Profile with a value of 1", so the value does not exist by default. With it set, Windows stops exposing the configured language list to web content, which in practice trims the `Accept-Language` header that Windows contributes.

Modern browsers set `Accept-Language` themselves, so the effect is mostly on Windows-integrated web content and legacy paths rather than on your main browser.

#### Benefits
- One identifying signal is removed from the fingerprinting surface.
- Nothing on the machine depends on the list being shared.
- Per-user setting, no elevation needed.

#### Drawbacks
- Narrower than it sounds, because modern browsers set the header themselves.
- Some multilingual sites stop auto-selecting your preferred language.
- One signal among many; it does not defeat fingerprinting on its own.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1703 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately; browsers pick it up on next launch.
- **Reverting**: "Allowed" deletes the value, the state a fresh profile is in.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction was that the stock state is value-absent, not 0; the shipped "Allowed" option deletes the value. Behaviourally 0 and absent are equivalent.
- **Confidence**: Microsoft-documented: key, value, data and the "create a new" wording are Microsoft's.
- **Reasoning**: the adversarial pass read Microsoft's "Create a new" wording as evidence the value is absent by default; key, value, type and polarity survived. The fingerprinting benefit is stated narrowly because browsers set the header independently.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it; the cost is one convenience that most people never notice. Skip it if you rely on sites auto-selecting a non-default language.

#### Sources
1. Manage connections from Windows components to Microsoft services, section 18.1 General, the `HttpAcceptLanguageOptOut` instruction, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. privacy.sexy, which writes the same value at the same key, https://github.com/undergroundwires/privacy.sexy (tier C, corroboration only)

### Disable File Explorer cloud recommendations

`disable_explorer_cloud_recommendations` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops File Explorer calling Microsoft's cloud for file recommendations and account-based insights.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `graph_recent_items` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `DisableGraphRecentItems`, `REG_DWORD` |

| Option | `graph_recent_items` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` |

System Default appears only if the value holds something other than 1 or nothing (for example 0); selecting it restores the snapshot. The stock state is value-absent.

#### How it works

The shipped `Explorer.admx` on 26100 defines `DisableGraphRecentItems` as `class="Machine"` at `Software\Policies\Microsoft\Windows\Explorer`, `supportedOn` Windows 11 22H2 and later, client only (`SUPPORTED_Windows_11_0_22H2_NOSERVER`), `enabledValue` 1 and `disabledValue` 0; so 1 turns the feature off. The value name is read by four shipped binaries: `System32\shell32.dll`, `System32\windows.storage.dll`, `SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\MicrosoftGraphRecentItemsManager.dll` and `SystemApps\MicrosoftWindows.Client.FileExp_cw5n1h2txyewy\FileExplorerExtensions.dll`; the third is named for exactly the subsystem the policy governs.

It is a network control, not a display toggle. The 26100 ADML: "Turning off this setting will prevent File Explorer from requesting cloud file metadata and displaying it in the homepage and other views in File Explorer. Any insights and files available based on account activity will be stopped in views such as Recent, Recommended, Favorites, Details pane, etc." The request to Microsoft Graph is not made at all. Local recent files are a separate setting and are not affected.

The tweak has no build gate. On Windows 10 and LTSC 2021 the policy does not exist and the value is written but ignored.

#### Benefits
- Stops the Graph request itself, not just the display.
- The Home page's Recommended strip stops showing OneDrive and SharePoint suggestions.
- One machine policy covers every account.

#### Drawbacks
- If you work from OneDrive or SharePoint, shortcuts to recently touched cloud files disappear.
- The Details pane stops showing activity information for a selected cloud file.
- Inert on Windows 10 and LTSC 2021.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 and newer, client editions only, so all of the primary platform; not LTSC 2021.
- **Takes effect**: immediately; restart File Explorer to redraw Home.
- **Reverting**: "Enabled" deletes the policy value, restoring the Windows default.

#### Interactions
- `disable_recent_files` (Interface, "Hide recent files and folders in Explorer") writes `ShowRecent` / `ShowFrequent`, the local recent list; it does not stop the Graph request.
- `remove_home_nav_pane` (Interface) hides the Home node; it does not stop the Graph request either.
- `disable_start_recommended_section` (Interface) uses `HideRecommendedSection` under a `Policies\Microsoft\Windows\Explorer` key; a different value.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented: shipped ADMX and ADML on 26100, with binary evidence of four consumers.
- **Reasoning**: the adversarial pass attacked existence (ADMX and four binaries), key, name, type and polarity (all exact; machine class means HKLM only), whether the effect is real or cosmetic (the ADML describes stopping the request), and duplication (distinct from the local recent-files and Home-node tweaks). All survived.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine that does not use OneDrive or SharePoint for daily work, where the cloud calls buy you nothing. Skip it if Explorer Home's Recommended list is part of your workflow.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\Explorer.admx` and `en-US\Explorer.adml` on build 26100, the definition and the "prevent File Explorer from requesting cloud file metadata" text (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: `DisableGraphRecentItems` in `shell32.dll`, `windows.storage.dll`, `MicrosoftGraphRecentItemsManager.dll` and `FileExplorerExtensions.dll` (tier A, product artifact)
3. Manage connections from Windows components to Microsoft services, context for the Explorer cloud-content surface, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### Disable app and device inventory collectors

`disable_app_device_inventory` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off the four app and device inventory collectors Microsoft added in Windows 11 24H2.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `install_tracing` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat`, value `DisableInstallTracing`, `REG_DWORD` |
| `api_sampling` | registry | same key, value `DisableAPISamping`, `REG_DWORD` |
| `application_footprint` | registry | same key, value `DisableApplicationFootprint`, `REG_DWORD` |
| `win32_app_backup` | registry | same key, value `DisableWin32AppBackup`, `REG_DWORD` |

| Option | `install_tracing` | `api_sampling` | `application_footprint` | `win32_app_backup` |
|---|---|---|---|---|
| Disabled | `1` | `1` | `1` | `1` |
| Enabled | `absent` | `absent` | `absent` | `absent` |

System Default appears when the four values are mixed (for example only some set) or hold 0; selecting it restores the snapshot. The stock state is all four absent.

#### How it works

All four are in the shipped `AppDeviceInventory.admx` on 26100, in their own "App and Device Inventory" policy category, each `class="Machine"`, each on `Software\Policies\Microsoft\Windows\AppCompat`, each `supportedOn` Windows 11 24H2, each with `enabledValue` 1 and `disabledValue` 0. The ADML help for each reads "If you enable this policy, X will not be run", and Microsoft left an intent comment in the file: `"Enabled" here means we are turning off API Sampling.` So 1 turns each collector off.

| Value name | Collector | Read by (on 26100) |
|---|---|---|
| `DisableInstallTracing` | Install Tracing, which tracks application installs | `System32\installmon.dll` |
| `DisableAPISamping` | API Sampling, sampled collection of APIs used at runtime | `System32\apisampling.dll` |
| `DisableApplicationFootprint` | sampled collection of registry and file usage | `System32\appfootprint.dll`, `System32\pcasvc.dll` |
| `DisableWin32AppBackup` | the compatibility scan over backed-up applications | `System32\aeinv.dll`, `System32\aemarebackup.dll`, `System32\appraiser.dll` |

Two spelling traps. The value name really is `DisableAPISamping`: a search for the correct spelling `DisableAPISampling` across every ADMX on 26100 returns nothing, so correcting it would produce a value nothing reads. And the ADMX policy names (`TurnOffInstallTracing`, `TurnOffAPISamping`, `TurnOffApplicationFootprint`, `TurnOffWin32AppBackup`) are not registry value names and must not be written.

The tweak has no build gate. On Windows 10, LTSC 2021 and pre-24H2 Windows 11 the values are written but nothing reads them.

#### Benefits
- Four separate sampling agents, each with its own shipped binary, are turned off.
- This 24H2-era family is not covered by older telemetry tweaks.
- All four are shipped Group Policy settings, not undocumented keys.

#### Drawbacks
- `DisableWin32AppBackup` turns off the compatibility scan that runs when restoring applications from Windows Backup.
- Microsoft has less to work with if you hit an app compatibility problem.
- Inert before 24H2: on Windows 10 and LTSC 2021 nothing reads the values.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 (build 26100) and newer only; not LTSC 2021 or pre-24H2 Windows 11.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes all four values, restoring the Windows default.

#### Interactions
- `disable_compat_appraiser` writes `AITEnable` and `DisableInventory` under the same `AppCompat` key; those are the legacy Application Impact Telemetry and inventory switches. Different values, no interference; the two tweaks together cover both families.
- `disable_windows_backup` locks periodic Windows Backup off; with it applied, the app-restore scan disabled here matters less.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the misspelled value name was confirmed genuine.
- **Confidence**: Microsoft-documented: shipped ADMX and ADML on 26100, with binary evidence that each value is read by its own collector.
- **Reasoning**: the adversarial pass attacked the odd spelling (genuine), whether anything reads the values (each by a binary named for its collector, the decisive result), polarity (ADML confirms 1 = collector off) and duplication with the appraiser tweak (distinct family). All survived; the research rated this the best-evidenced addition in its batch. The verification pass recommended a build gate at 26100; the tweak has none, so it is offered but inert on older builds.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any 24H2 or newer machine you keep on its current apps. Skip it if you plan to restore your applications from Windows Backup on a new PC, since the restore compatibility scan goes with it.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\AppDeviceInventory.admx` and `en-US\AppDeviceInventory.adml` on build 26100, all four definitions, polarity and the intent comment (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: each value name in its own collector binary, `installmon.dll`, `apisampling.dll`, `appfootprint.dll` / `pcasvc.dll`, `aeinv.dll` / `aemarebackup.dll` / `appraiser.dll` (tier A, product artifact)
3. Configure Windows diagnostic data in your organization, context for the appraiser and inventory family, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)

### Turn off device search history

`disable_search_history` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows keeping a history of what you have searched for on this device.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `device_search_history` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`, value `IsDeviceSearchHistoryEnabled`, `REG_DWORD` |

| Option | `device_search_history` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default appears if the value holds 1 (Settings writes 1 when the toggle is turned on explicitly); selecting it restores the snapshot. The stock state is value-absent.

#### How it works

`IsDeviceSearchHistoryEnabled` is the per-user value behind Settings > Privacy and security > Search permissions > "Search history on this device". On 26100 it appears in `SearchUx.UI.dll` (the taskbar search UI, in the view-model block next to `IsCloudSearchEnabledForMSA` and `IsCloudSearchEnabledForAAD`), `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll`. At 0, Windows stops recording your search terms and stops replaying them as recent searches when the search box opens. privacy.sexy annotates the value "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)" from its default-state scan of real Pro images, which is why "On" deletes it.

The older `DisableSearchHistory` Group Policy is deliberately not written. The shipped 26100 `Search.admx` declares it `class="User"` (so HKLM would be the wrong hive), marks it `supportedOn ref="Win8Only"` in the legacy block of the file, and its name appears in exactly one shipped binary, `SHCore.dll` (the shell policy table), and in no search component.

Microsoft publishes no page for `IsDeviceSearchHistoryEnabled`; the value is confirmed by the shipped search UI binary and by independent tools.

#### Benefits
- Past queries stop being stored for this account.
- The recent-searches list stops appearing when you open search.
- Per-user setting, no elevation needed.

#### Drawbacks
- Repeating a previous query means typing it again.
- Local history only, so the gain is against someone using your PC, not against Microsoft.
- Not Microsoft-documented.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1903 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately; reopen search to see it.
- **Reverting**: "On" deletes the value, the state a fresh profile is in.

#### Interactions
- `disable_cloud_content_search` writes two other values under the same `SearchSettings` key; `disable_search_highlights` (Interface) writes `IsDynamicSearchBoxEnabled` there. Each owns a different value.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (community-corroborated). The corrections were that only `IsDeviceSearchHistoryEnabled` does anything on current Windows (the `DisableSearchHistory` policy is user-class, Windows 8 only and unread), and that the stock state is value-absent, so "On" deletes rather than writing 1.
- **Confidence**: Community-corroborated: no Microsoft documentation, but shipped binary evidence plus two independent tools.
- **Reasoning**: the adversarial pass rejected the policy half on three independent grounds (wrong hive, Windows 8 only, no Windows 11 consumer) and corrected the default from privacy.sexy's default-state annotation. It also recorded that privacy.sexy's own search-history script writes `IsDeviceSearchHistoryEnabled` = 1 in a script meant to disable it, an inverted-polarity bug, so privacy.sexy is used here only for the default state, not as a semantics oracle.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a shared or observed machine, or if a search history you never asked for bothers you. Skip it if you re-run the same searches often and value the shortcut.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\Search.admx` on build 26100, `DisableSearchHistory` as `class="User"` with `supportedOn ref="Win8Only"`, the evidence for not writing the policy (tier A, shipped ADMX)
2. String evidence on build 26100: `IsDeviceSearchHistoryEnabled` in `SearchUx.UI.dll`, `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll`; `DisableSearchHistory` only in `SHCore.dll` (tier A, product artifact)
3. privacy.sexy, the value with `deleteOnRevert` and the "Missing by default" annotation, https://github.com/undergroundwires/privacy.sexy (tier C)
4. Win11Debloat, `Regfiles/Disable_Search_History.reg`, writing only this value as 0, https://github.com/Raphire/Win11Debloat (tier C)

### Disable cloud content in search

`disable_cloud_content_search` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops taskbar search querying your OneDrive, SharePoint and Outlook content in the cloud.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `allow_cloud_search` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`, value `AllowCloudSearch`, `REG_DWORD` |
| `msa_cloud_search` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`, value `IsMSACloudSearchEnabled`, `REG_DWORD` |
| `aad_cloud_search` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`, value `IsAADCloudSearchEnabled`, `REG_DWORD` |

| Option | `allow_cloud_search` | `msa_cloud_search` | `aad_cloud_search` |
|---|---|---|---|
| Disabled | `0` | `0` | `0` |
| Enabled | `absent` | `absent` | `absent` |

System Default appears when the three values are mixed or hold other data, including `AllowCloudSearch` = 1 (Enable) or 2 (User Selected), which this tweak does not author; selecting it restores the snapshot. The stock state is all three absent. The two HKCU effects run as the interactive user.

#### How it works

The shipped 26100 `Search.admx` defines `AllowCloudSearch` as `class="Machine"` at `SOFTWARE\Policies\Microsoft\Windows\Windows Search`, `supportedOn` Windows 10, with an enum of three values: 0 = Disable Cloud Search, 1 = Enable Cloud Search, 2 = User Selected. The ADML reads "Allow search and Cortana to search cloud sources like OneDrive and SharePoint." Despite the Cortana-era wording, the string is present in `Windows.Storage.Search.dll`, `windowsudk.shellcommon.dll` and `Windows.FileExplorer.Common.dll` on 26100, so it is live. The policy is Policy CSP `Search/AllowCloudSearch`.

`IsMSACloudSearchEnabled` (personal Microsoft account) and `IsAADCloudSearchEnabled` (work or school account) are the per-user values behind Settings > Privacy and security > Search permissions > Cloud content search; they are present in `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll`. privacy.sexy writes both with `deleteOnRevert` and the annotation "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)", which is why "Enabled" deletes them.

With all three at 0, search stops sending your typed query to Microsoft to match against account content and returns local results only. Local file and app search is unaffected. Only two of the policy's three states are offered: "User Selected" (2) is not.

#### Benefits
- What you type in the search box stops being sent to Microsoft for account-content matching.
- Personal Microsoft accounts and work or school accounts are both covered.
- No network round trip for cloud results.

#### Drawbacks
- OneDrive, SharePoint and Outlook items stop appearing in taskbar search.
- On a managed device this can remove a feature colleagues rely on.
- The "User Selected" (2) state of `AllowCloudSearch` is not offered.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; Windows 10 1709 and later for the policy and 1903 and later for the per-user values, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes all three values, the state a fresh install and profile are in.

#### Interactions
- `disable_search_history` and `disable_search_highlights` (Interface) write other values under the same `SearchSettings` key.
- `no_index_encrypted_files` (Security) writes a different value under the same `Windows Search` policy key.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. Keys, names and types were correct; the corrections were that both HKCU values are absent by default (so "Enabled" deletes them) and that `AllowCloudSearch` has a third value, 2 = User Selected, which the tweak does not expose.
- **Confidence**: Microsoft-documented for the policy (Policy CSP and shipped ADMX); the per-user values are established by shipped binaries and privacy.sexy's default-state data.
- **Reasoning**: the adversarial pass checked the ADMX enum, binary presence for all three values, the stock state and duplication (the corpus touches `SearchSettings` elsewhere only for different values). All survived after the default correction.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a personal machine where taskbar search is for launching apps and finding local files. Skip it if you deliberately search OneDrive or work content from the taskbar.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\Search.admx` on build 26100, `AllowCloudSearch`, `class="Machine"`, three-value enum (tier A, shipped ADMX)
2. Policy CSP - Search (AllowCloudSearch), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search (tier A)
3. String evidence on build 26100: `AllowCloudSearch` in `Windows.Storage.Search.dll`, `windowsudk.shellcommon.dll`, `Windows.FileExplorer.Common.dll`; both HKCU value names in `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll` (tier A, product artifact)
4. privacy.sexy, "Disable personal cloud content search in taskbar", with `deleteOnRevert` on both HKCU values, https://github.com/undergroundwires/privacy.sexy (tier C, cited for the stock-default correction)

### Block voice activation and wake words

`disable_voice_activation` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Blocks apps from listening for a wake word, including above the lock screen.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `voice_activation` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`, value `LetAppsActivateWithVoice`, `REG_DWORD` |
| `voice_activation_lock` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`, value `LetAppsActivateWithVoiceAboveLock`, `REG_DWORD` |

| Option | `voice_activation` | `voice_activation_lock` |
|---|---|---|
| Blocked | `2` (Force Deny) | `2` (Force Deny) |
| Allowed | `absent` | `absent` |

System Default appears when the two values are mixed or hold 0 (User in control) or 1 (Force Allow); selecting it restores the snapshot. The stock state is both absent, meaning the user is in control.

#### How it works

The shipped 26100 `AppPrivacy.admx` defines both as `class="Machine"` at `Software\Policies\Microsoft\Windows\AppPrivacy`, `supportedOn` Windows 10, each with the enum 0 = User is in control, 1 = Force Allow, 2 = Force Deny; the ADML for both describes the Force Deny behaviour explicitly, including the above-lock variant. They are the Policy CSP `Privacy/LetAppsActivateWithVoice` family. With Force Deny, no app can register for voice activation, so the always-listening wake-word path is closed for every account, and the Settings > Privacy and security > Voice activation controls are locked. The value names are present in `agentactivationruntimewindows.dll` (the voice-activation runtime), `AarSvc.dll` and `SettingsHandlers_SpeechPrivacy.dll` on 26100.

Push-to-talk and manually invoked voice input are not wake-word activation and are unaffected. Per-user siblings exist (`AgentActivationEnabled` and `AgentActivationOnLockScreenEnabled` under `HKCU\Software\Microsoft\Speech_OneCore\Settings\VoiceActivation\UserPreferenceForAllApps`); the machine policy overrides them, and this tweak does not write them.

#### Benefits
- Nothing can listen for a keyword; the always-on activation path is denied outright.
- The above-lock variant closes the stronger exposure while the PC is locked.
- With Force Deny, users and apps cannot grant the capability back.

#### Drawbacks
- Any voice assistant you actually want, including third-party ones, stops responding to its keyword.
- Hands-free use while the PC is locked ends.
- Users cannot re-enable it from Settings while the policy is applied.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately; an app already running may need restarting.
- **Reverting**: "Allowed" deletes both values, restoring user control.

#### Interactions
- `disable_online_speech_recognition` covers cloud recognition, not wake-word activation; the two are complementary.
- `disable_location_tracking`, `disable_app_diagnostics` and `disable_background_apps` (Performance) write other values under the same `AppPrivacy` key.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented: shipped ADMX and ADML with the enum, Policy CSP, and binary evidence of the voice-activation runtime reading both values.
- **Reasoning**: the adversarial pass checked the enum and polarity (Force Deny = 2, not inverted), binary consumers and duplication (no other `LetAppsActivateWithVoice*` in the corpus). All survived.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine with a microphone where you do not use a wake word. Skip it if you rely on hands-free voice activation, including for accessibility.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\AppPrivacy.admx` and `en-US\AppPrivacy.adml` on build 26100, both policies `class="Machine"` with the 0 / 1 / 2 enum (tier A, shipped ADMX and ADML)
2. Policy CSP - Privacy (LetAppsActivateWithVoice family), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
3. String evidence on build 26100: both value names in `agentactivationruntimewindows.dll`, `AarSvc.dll` and `SettingsHandlers_SpeechPrivacy.dll` (tier A, product artifact)
4. privacy.sexy, for the per-user `AgentActivationEnabled` siblings, https://github.com/undergroundwires/privacy.sexy (tier C)

### Disable Windows Backup and cloud restore

`disable_windows_backup` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Locks periodic Windows Backup off so nothing can switch it on later.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `enable_windows_backup` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync`, value `EnableWindowsBackup`, `REG_DWORD` |

| Option | `enable_windows_backup` |
|---|---|
| Disabled | `0` |
| Enabled | `absent` |

System Default appears if the value holds 1 (periodic backup explicitly enabled by policy); selecting it restores the snapshot. The stock state is value-absent. Note that the "Enabled" label means "policy not configured", which by Microsoft's text also means periodic backup does not take place.

#### How it works

The shipped 26100 `SettingSync.admx` defines `EnableWindowsBackup` as `class="Machine"` at `Software\Policies\Microsoft\Windows\SettingSync`, `enabledValue` 1 and `disabledValue` 0, `supportedOn` Windows 10 and later, client only. The ADML is verbatim: "If you enable this policy setting, windows backup will occur periodically. If you disable or do not configure this policy setting, windows backup will not take place." So 0 and absent are documented as behaviourally identical: this tweak records an explicit "disabled" decision so that an OOBE flow, a Settings nudge or a servicing change cannot start periodic backup, rather than changing what happens today.

The value name is present in `SyncSettings.dll`, `CloudRestoreLauncher.dll` and `cdp.dll` on 26100; `CloudRestoreLauncher.dll` is the cloud-restore entry point, which is the positive evidence that the value governs the Windows Backup and cloud restore pipeline. On an unmanaged machine the Windows Backup app can still be run by hand, so this is not a hard block on backing up.

#### Benefits
- Periodic backup cannot be turned on later behind your back.
- The machine records a decision rather than relying on a default.
- One machine policy covers every account.

#### Drawbacks
- No visible change today: periodic backup does not take place when the policy is unset either.
- The Windows Backup app can still be run manually, so this is not a hard block.
- You give up the automatic cloud restore path on a future reinstall or new PC.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 client editions, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes the policy value, restoring the Windows default.

#### Interactions
- `disable_settings_sync` writes `DisableSettingSync` and `DisableSettingSyncUserOverride` in the same `SettingSync` key; those govern legacy roaming settings sync, a different feature. The two are complementary.
- `disable_app_device_inventory`'s `DisableWin32AppBackup` turns off the compatibility scan used when restoring apps from Windows Backup.

#### Validation
- **Verdict**: VERIFIED. No correction was needed on the mechanism; the verification attached a mandatory copy constraint: say that there may be no visible change today and that manual backup remains possible.
- **Confidence**: Microsoft-documented: shipped ADMX and ADML on 26100, with binary evidence.
- **Reasoning**: the adversarial pass matched every field against the ADMX, found the consumers, confirmed it is distinct from settings sync, and forbade describing it as "stops Windows backing up your files", since 0 and absent are documented as equivalent.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you back up deliberately with your own tooling and do not want a Microsoft-managed copy of your setup. Skip it if cloud restore on a future PC is something you would actually use.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\SettingSync.admx` and `en-US\SettingSync.adml` on build 26100, definition and the "will not take place" text (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: `EnableWindowsBackup` in `SyncSettings.dll`, `CloudRestoreLauncher.dll` and `cdp.dll` (tier A, product artifact)
3. Manage connections from Windows components to Microsoft services, section 21 Sync your settings, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### Disable Settings app online tips

`disable_online_tips` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the Settings app fetching help and tip content from Microsoft over the network.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `allow_online_tips` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer`, value `AllowOnlineTips`, `REG_DWORD` |

| Option | `allow_online_tips` |
|---|---|
| Disabled | `0` |
| Enabled | `absent` |

System Default appears if the value holds 1; selecting it restores the snapshot. The stock state is value-absent.

#### How it works

The shipped 26100 `ControlPanel.admx` defines the "Allow Online Tips" policy as `class="Machine"` at `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`, with a boolean `CheckBox_AllowOnlineTips` writing `AllowOnlineTips`, `trueValue` 1 and `falseValue` 0, supported from Windows 10 1709 (RS3). The ADML: "Enables or disables the retrieval of online tips and help for the Settings app. If disabled, Settings will not contact Microsoft content services to retrieve tips and help content." The value name is present in `SystemSettings.dll`, the Settings app itself, so the consumer is confirmed.

The key is the legacy policy location under `CurrentVersion\Policies\Explorer`, not under `Software\Policies`. privacy.sexy writes this value to `HKLM\SOFTWARE\Policies\Microsoft\Windows\System`, which is not the key in the shipped ADMX; the ADMX location is the one Settings honours.

#### Benefits
- The Settings app stops making content-service requests.
- The value is read by `SystemSettings.dll`, the Settings app itself.
- One machine policy covers every account.

#### Drawbacks
- Settings pages show only the locally shipped help text, which can be out of date.
- Fixes and clarifications Microsoft publishes after release do not appear.
- One app's help fetch, not a system-wide network control.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later, so LTSC 2021 is covered.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes the value, restoring the Windows default.

#### Interactions
- `disable_tips_and_suggestions` targets the Content Delivery Manager tip notifications, a different surface and different traffic.
- `disable_suggested_content_settings` removes promotional cards inside Settings; this tweak removes the online help content. Complementary.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented: shipped ADMX and ADML on 26100, with the Settings binary as the consumer.
- **Reasoning**: the adversarial pass matched key, value and polarity to the ADMX, confirmed the consumer, and recorded that a major upstream (privacy.sexy) uses the wrong key; the shipped ADMX wins.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine where you would rather Settings did not contact Microsoft to render a help panel. Skip it if you lean on the in-Settings help links.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\ControlPanel.admx` and `en-US\ControlPanel.adml` on build 26100, the key, value and help text (tier A, shipped ADMX and ADML)
2. String evidence on build 26100: `AllowOnlineTips` in `SystemSettings.dll` (tier A, product artifact)
3. Manage connections from Windows components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
4. privacy.sexy, recorded as the upstream that writes this value to the wrong key, https://github.com/undergroundwires/privacy.sexy (tier C, a defect record, not support)

### Disable the Windows Error Reporting service

`disable_wer_service` · Switch · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the Windows Error Reporting service, so crash reports are not collected or uploaded at all.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `wer_service` | service | `WerSvc` ("Windows Error Reporting Service", `System32\WerSvc.dll`), start type |

| Option | `wer_service` |
|---|---|
| Disabled | `disabled` |

This is a toggle: the other position is System Default, which restores the service's previous start type from the snapshot. The shipped default start type is unresolved (a verification proposal said Manual, but no authoritative source confirms it), which is why no "Enabled" option with a fixed start type is authored.

#### How it works

The service effect sets the start type through the Service Control Manager. `WerSvc` is the Windows Error Reporting Service; with it disabled, crash and hang reports are not gathered, queued or sent. `WerSvc` is present on 26100 and has no SCM dependents and no dependencies, so disabling it does not cascade to any other service.

This is the service-level companion to the WER `Disabled` values (`disable_error_reporting`) and the `QueueReporting` scheduled task (`task_wer_queuereporting` in Services); the WER policy stops reports being permitted, the task stops queued reports being uploaded, and this stops the service that processes them.

#### Benefits
- Reports are never created, rather than created and then blocked.
- No dependents in either direction, so nothing else breaks.
- Completes the set with the WER policy value and the queue-reporting task.

#### Drawbacks
- Reliability Monitor's crash history and the "Windows has recovered from an unexpected shutdown" dialogs go quiet, as does `Get-WindowsErrorReporting`.
- On an unstable machine you have removed your first diagnostic.
- Some third-party crash handlers query WER state and behave differently.
- No speed gain: this is a privacy and noise reduction, not a performance tweak.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and all supported Windows 10 versions including LTSC 2021.
- **Takes effect**: immediately; the start type changes on apply.
- **Reverting**: System Default restores the previous start type from the snapshot rather than writing a fixed value.

#### Interactions
- `disable_error_reporting` sets the WER `Disabled` product and policy values; the natural pairing.
- `task_wer_queuereporting` (Services, "Disable Windows Error Reporting QueueReporting task") stops queued reports being uploaded; its own description names this service as its pairing.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections were that the "stops WER consuming CPU on a crash" claim is unmeasured and is not made, and that the shipped start type is unresolved, so revert restores from the snapshot.
- **Confidence**: Microsoft-documented for what WER does; service presence and the empty dependency graph were read live on 26100.
- **Reasoning**: the verification confirmed the service exists, is genuinely absent elsewhere in the corpus, and has no dependents; it removed the performance claim and refused to assert a default start type. Open question: the shipped start type on a clean image.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a stable machine where you have already disabled Windows Error Reporting by policy and want the service stopped too. Do not apply it while troubleshooting crashes.

#### Sources
1. WER Settings, what Windows Error Reporting does, https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings (tier A)
2. Security guidelines for disabling system services in Windows Server, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (from the tweak's own evidence list; not tiered in the research)
3. Live service presence and SCM dependency graph on build 26100: `WerSvc` present, `WerSvc.dll` in `System32`, no dependents and no dependencies (tier A for existence only, not for default start type)
4. WinUtil, privacy.sexy and Sophia Script, all agreeing on `WerSvc` as the service short name, https://github.com/undergroundwires/privacy.sexy (tier C)

## Considered and not shipped

### Recall snapshots, Recall component removal, Click to Do

`disable_recall_snapshots`, `remove_recall_component` and `disable_click_to_do` are Windows AI and Copilot+ controls under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`. They were researched alongside this category and moved to the AI category, where they ship and are documented.

### Turn off all Windows Spotlight features

`disable_windows_spotlight_all` (`DisableWindowsSpotlightFeatures` under `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`) was verified as a privacy proposal and ships in the Debloat category ("Turn off all Windows Spotlight features"). It is a master switch whose surfaces overlap `disable_lockscreen_spotlight_ads`, `disable_tips_and_suggestions` and `disable_consumer_features` without writing their values, so those tweaks can read as not applied while their surfaces are already gone.

### Turn off third-party suggestions in Spotlight

`disable_third_party_suggestions` would have written `DisableThirdPartySuggestions` = 1 under the per-user `CloudContent` policy key (`CloudContent.admx`, `class="User"`), which stops Spotlight surfaces suggesting apps and content from third-party publishers. The facts checked out; it was rejected because Policy CSP `Experience/AllowThirdPartySuggestionsInWindowsSpotlight` depends on Spotlight being allowed (so it does nothing once Spotlight is off), because everything it suppresses is a strict subset of "Turn off all Windows Spotlight features", and because its cited community source (privacy.sexy) does not contain the value. Source: shipped `CloudContent.admx` on build 26100 (tier A) and Policy CSP - Experience, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### Separate compatibility-appraiser and CEIP task tweaks

Proposed `task_compatibility_appraiser` and `task_ceip` tweaks were rejected as duplicates: `disable_compat_appraiser` and `disable_ceip_tasks` already own those tasks, and a second tweak over the same task would let two tweaks drive it to contradictory states. The genuinely new tasks were to be folded into the existing tweaks instead. `Microsoft Compatibility Appraiser Exp` is included in `disable_compat_appraiser`; `PcaPatchDbTask`, `MareBackup` and `\Microsoft\Windows\PI\Sqm-Tasks` are not currently included anywhere. `SdbinstMergeDbTask` (shim database maintenance) and `\Microsoft\Windows\PI\Secure-Boot-Update` (Secure Boot DBX revocation delivery) must not be disabled. Source: task-folder enumeration on build 26100 (tier A for presence).

### Search history Group Policy (`DisableSearchHistory`)

The search history proposal originally also wrote `DisableSearchHistory` = 1 to `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`. That half was dropped: the shipped 26100 `Search.admx` declares it `class="User"` (HKLM is the wrong hive) and `supportedOn ref="Win8Only"`, and the name appears only in `SHCore.dll`, in no search component. `disable_search_history` ships with `IsDeviceSearchHistoryEnabled` alone. Source: shipped `Search.admx` and string evidence on build 26100 (tier A).

### Obsolete Edge metrics policies

`MetricsReportingEnabled` and `SendSiteInfoToImproveServices` under `HKLM\SOFTWARE\Policies\Microsoft\Edge` are not written by `disable_edge_telemetry` because Microsoft marks both "OBSOLETE: This policy is obsolete and doesn't work after Microsoft Edge version 88". Sources: https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/metricsreportingenabled and https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/sendsiteinfotoimproveservices (tier A).
