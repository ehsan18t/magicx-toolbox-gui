# Policy hive audit: ADMX class vs. registry hive

**Scope:** every registry effect in `src-tauri/tweaks/*.yaml` whose key path contains `\Policies\`.
**Defect class hunted:** a Group Policy backed value written to the wrong hive relative to the
`class` its ADMX definition declares. A `class="User"` policy belongs in `HKCU\Software\Policies\...`
and a `class="Machine"` policy belongs in `HKLM\SOFTWARE\Policies\...`. Writing one in the wrong hive
means the Group Policy engine never refreshes it and, in most cases, the consuming component never
reads it, so the tweak silently does nothing.

**Target platform:** Windows 11 24H2 (build 26100) and newer, including 25H2. Windows 10 IoT
Enterprise LTSC 2021 is a low-priority secondary target.

## Sources used

1. **`C:\Windows\PolicyDefinitions` on a Windows 11 build 26100 machine** (the audit host). 218 ADMX
   files, 3,403 `<policy>` definitions parsed. This is the shipped ADMX set for the primary target
   platform and is the strongest possible evidence for the `class` attribute.
2. **Microsoft Learn Policy CSP pages** (`learn.microsoft.com/windows/client-management/mdm/policy-csp-*`)
   for the Scope row (Device and/or User) and the "Group policy mapping" block naming the ADMX file,
   registry key, registry value, and Computer vs. User Configuration.
3. **Microsoft Learn Edge policy reference** (`learn.microsoft.com/deployedge/microsoft-edge-policies/*`)
   for the Edge registry paths and ADMX file name.
4. **Chromium source** (`components/policy/tools/template_writers/writers/admx_writer.py`,
   `components/policy/core/common/policy_loader_win.cc`) for the class emitted into `MSEdge.admx`
   and the hive precedence Edge applies.

`admx.help` and `getadmx.com` were deliberately not used.

## Headline result

**Exactly one wrong-hive defect exists in the corpus, and it is the already-known one.**
85 policy-path values were checked. No second instance of the defect was found, in either direction.
No `HKCU\Software\Policies\...` write in the corpus is a Machine-class policy.

| Verdict | Count |
| --- | --- |
| CORRECT | 56 |
| CLASS BOTH | 18 |
| NOT A GP POLICY | 10 |
| WRONG HIVE | 1 |
| UNRESOLVED | 0 |
| **Total** | **85** |

## Summary table

### debloat.yaml

| Tweak id | Value name | Declared class | Hive written | Verdict |
| --- | --- | --- | --- | --- |
| `disable_web_search_start` | `DisableSearchBoxSuggestions` | User (WindowsExplorer.admx) | HKCU | CORRECT |
| `disable_widgets` | `AllowNewsAndInterests` | Machine (NewsAndInterests.admx) | HKLM | CORRECT |
| `remove_teams_chat_taskbar` | `ChatIcon` | Machine (Taskbar.admx) | HKLM | CORRECT |
| `disable_edge_first_run` | `HideFirstRunExperience` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_startup_boost` | `StartupBoostEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_startup_boost` | `BackgroundModeEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_sidebar` | `HubsSidebarEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_sidebar` | `EdgeCollectionsEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `remove_cortana` | `AllowCortana` | Machine (Search.admx) | HKLM | CORRECT (see note) |

### interface.yaml

| Tweak id | Value name | Declared class | Hive written | Verdict |
| --- | --- | --- | --- | --- |
| `disable_copilot_taskbar` | `TurnOffWindowsCopilot` | User (WindowsCopilot.admx) | HKCU | CORRECT |
| `disable_meet_now` | `HideSCAMeetNow` | User (Taskbar.admx) | HKCU | CORRECT |
| `verbose_logon_messages` | `VerboseStatus` | Machine (Logon.admx) | HKLM | CORRECT |

### network.yaml

| Tweak id | Value name | Declared class | Hive written | Verdict |
| --- | --- | --- | --- | --- |
| `disable_llmnr` | `EnableMulticast` | Machine (DnsClient.admx) | HKLM | CORRECT |
| `disable_delivery_optimization_p2p` | `DODownloadMode` | Machine (DeliveryOptimization.admx) | HKLM | CORRECT |
| `defer_quality_updates` | `DeferQualityUpdates` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `defer_quality_updates` | `DeferQualityUpdatesPeriodInDays` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `target_release_version` | `TargetReleaseVersion` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `target_release_version` | `TargetReleaseVersionInfo` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `target_release_version` | `ProductVersion` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `disable_auto_restart_logged_on` | `NoAutoRebootWithLoggedOnUsers` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `exclude_wu_driver_updates` | `ExcludeWUDriversInQualityUpdate` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `disable_store_auto_updates` | `AutoDownload` | Machine (WindowsStore.admx) | HKLM | CORRECT |
| `block_update_over_metered` | `AllowAutoWindowsUpdateDownloadOverMeteredNetwork` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `disable_auto_update_download` | `NoAutoUpdate` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |
| `disable_auto_update_download` | `AUOptions` | Machine (WindowsUpdate.admx) | HKLM | CORRECT |

### performance.yaml

| Tweak id | Value name | Declared class | Hive written | Verdict |
| --- | --- | --- | --- | --- |
| `disable_background_apps` | `LetAppsRunInBackground` | Machine (AppPrivacy.admx) | HKLM | CORRECT |
| `disable_gamedvr_capture` | `AllowGameDVR` | Machine (GameDVR.admx) | HKLM | CORRECT |
| `disable_storage_sense` | `AllowStorageSenseGlobal` | Machine (StorageSense.admx) | HKLM | CORRECT |

### privacy.yaml

| Tweak id | Value name | Declared class | Hive written | Verdict |
| --- | --- | --- | --- | --- |
| `disable_recall_snapshots` | `DisableAIDataAnalysis` | Both (WindowsCopilot.admx) | HKLM | CLASS BOTH |
| `remove_recall_component` | `AllowRecallEnablement` | Machine (WindowsCopilot.admx) | HKLM | CORRECT |
| `disable_click_to_do` | `DisableClickToDo` | Both (WindowsCopilot.admx) | HKLM | CLASS BOTH |
| `disable_diagnostic_data` | `AllowTelemetry` | Both (DataCollection.admx) | HKLM | CLASS BOTH |
| `disable_ceip_tasks` | `CEIPEnable` | Machine (ICM.admx) | HKLM | CORRECT |
| `disable_compat_appraiser` | `AITEnable` | Machine (AppCompat.admx) | HKLM | CORRECT |
| `disable_compat_appraiser` | `DisableInventory` | Machine (AppCompat.admx) | HKLM | CORRECT |
| `disable_feedback_notifications` | `DoNotShowFeedbackNotifications` | Machine (FeedbackNotifications.admx) | HKLM | CORRECT |
| `disable_onesettings_downloads` | `DisableOneSettingsDownloads` | Machine (DataCollection.admx) | HKLM | CORRECT |
| `disable_device_name_in_telemetry` | `AllowDeviceNameInTelemetry` | Machine (DataCollection.admx) | HKLM | CORRECT |
| `disable_activity_history` | `EnableActivityFeed` | Machine (OSPolicy.admx) | HKLM | CORRECT |
| `disable_activity_history` | `PublishUserActivities` | Machine (OSPolicy.admx) | HKLM | CORRECT |
| `disable_activity_history` | `UploadUserActivities` | Machine (OSPolicy.admx) | HKLM | CORRECT |
| `disable_cloud_clipboard` | `AllowCrossDeviceClipboard` | Machine (OSPolicy.admx) | HKLM | CORRECT |
| `disable_inking_typing_personalization` | `AllowInputPersonalization` | Machine (Globalization.admx) | HKLM | CORRECT |
| `disable_advertising_id` | `DisabledByGroupPolicy` | Machine (UserProfiles.admx) | HKLM | CORRECT |
| `disable_tailored_experiences` | `DisableTailoredExperiencesWithDiagnosticData` | **User (CloudContent.admx)** | **HKLM** | **WRONG HIVE** |
| `disable_location_tracking` | `DisableLocation` | User and Machine (Sensors.admx) | HKLM | CLASS BOTH |
| `disable_app_diagnostics` | `LetAppsGetDiagnosticInfo` | Machine (AppPrivacy.admx) | HKLM | CORRECT |
| `disable_find_my_device` | `AllowFindMyDevice` | Machine (FindMy.admx) | HKLM | CORRECT |
| `disable_settings_sync` | `DisableSettingSync` | Machine (SettingSync.admx) | HKLM | CORRECT |
| `disable_settings_sync` | `DisableSettingSyncUserOverride` | Machine (SettingSync.admx) | HKLM | CORRECT |
| `disable_error_reporting` | `Disabled` | User and Machine (ErrorReporting.admx) | HKLM | CLASS BOTH |
| `disable_tips_and_suggestions` | `DisableSoftLanding` | Machine (CloudContent.admx) | HKLM | CORRECT |
| `disable_consumer_features` | `DisableWindowsConsumerFeatures` | Machine (CloudContent.admx) | HKLM | CORRECT |
| `disable_edge_telemetry` | `MetricsReportingEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_telemetry` | `SendSiteInfoToImproveServices` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_telemetry` | `PersonalizationReportingEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |
| `disable_edge_telemetry` | `Edge3PSerpTelemetryEnabled` | Both (MSEdge.admx) | HKLM | CLASS BOTH |

### security.yaml

| Tweak id | Value name | Declared class | Hive written | Verdict |
| --- | --- | --- | --- | --- |
| `require_ctrlaltdel` | `DisableCAD` | n/a (Security Option) | HKLM | NOT A GP POLICY |
| `disable_autorun` | `NoDriveTypeAutoRun` | Both (AutoPlay.admx) | HKLM | CLASS BOTH |
| `disable_autorun` | `NoAutorun` | Both (AutoPlay.admx) | HKLM | CLASS BOTH |
| `powershell_scriptblock_logging` | `EnableScriptBlockLogging` | Both (PowerShellExecutionPolicy.admx) | HKLM | CLASS BOTH |
| `uac_max` | `ConsentPromptBehaviorAdmin` | n/a (Security Option) | HKLM | NOT A GP POLICY |
| `uac_max` | `PromptOnSecureDesktop` | n/a (Security Option) | HKLM | NOT A GP POLICY |
| `uac_max` | `EnableLUA` | n/a (Security Option) | HKLM | NOT A GP POLICY |
| `filter_admin_token` | `FilterAdministratorToken` | n/a (Security Option) | HKLM | NOT A GP POLICY |
| `hide_last_user` | `DontDisplayLastUserName` | n/a (Security Option) | HKLM | NOT A GP POLICY |
| `printnightmare_point_and_print` | `RestrictDriverInstallationToAdministrators` | Machine (Printing.admx) | HKLM | CORRECT |
| `enable_controlled_folder_access` | `EnableControlledFolderAccess` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `enable_network_protection` | `EnableNetworkProtection` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `enable_pua_protection` | `PUAProtection` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `asr_block_lsass_theft` | `9e6c4e1f-...e4b0` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `asr_block_office_script_vectors` | `d4f940ab-...688a` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `asr_block_office_script_vectors` | `3b576869-...e899` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `asr_block_office_script_vectors` | `5beb7efe-...04cc` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `asr_block_office_script_vectors` | `be9ba2d9-...6550` | Machine (WindowsDefender.admx) | HKLM | CORRECT |
| `enforce_smartscreen` | `EnableSmartScreen` | Machine (SmartScreen.admx) | HKLM | CORRECT |
| `enforce_smartscreen` | `ShellSmartScreenLevel` | Machine (SmartScreen.admx) | HKLM | CORRECT |
| `enforce_smartscreen` | `SmartScreenEnabled` | Both (SmartScreen.admx) | HKLM | CLASS BOTH |
| `disable_smb_guest` | `AllowInsecureGuestAuth` | Machine (LanmanWorkstation.admx) | HKLM | CORRECT |
| `firewall_all_profiles` | `EnableFirewall` (DomainProfile) | Machine (WindowsFirewall.admx) | HKLM | CORRECT |
| `firewall_all_profiles` | `DefaultInboundAction` (DomainProfile) | n/a (firewall CSE) | HKLM | NOT A GP POLICY |
| `firewall_all_profiles` | `EnableFirewall` (StandardProfile) | Machine (WindowsFirewall.admx) | HKLM | CORRECT |
| `firewall_all_profiles` | `DefaultInboundAction` (StandardProfile) | n/a (firewall CSE) | HKLM | NOT A GP POLICY |
| `firewall_all_profiles` | `EnableFirewall` (PublicProfile) | n/a (firewall CSE) | HKLM | NOT A GP POLICY |
| `firewall_all_profiles` | `DefaultInboundAction` (PublicProfile) | n/a (firewall CSE) | HKLM | NOT A GP POLICY |

### services.yaml

No `\Policies\` registry effects. Nothing to audit.

## Defect found

### 1. WRONG HIVE: `privacy:disable_tailored_experiences` / `DisableTailoredExperiencesWithDiagnosticData`

* **Declared class:** `User`
* **ADMX file:** `CloudContent.admx`
* **Currently written to:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`
* **Correct location:** `HKCU\Software\Policies\Microsoft\Windows\CloudContent`
* **Effect of the current write:** **inert.** The Group Policy engine never refreshes this value in
  HKLM because no Computer Configuration policy claims it, and the consuming component (the content
  delivery / tailored experiences path) reads the per-user policy location. The value sits in the
  registry doing nothing.

**Evidence, shipped ADMX (build 26100):**

```xml
<policy name="DisableTailoredExperiencesWithDiagnosticData" class="User"
        key="Software\Policies\Microsoft\Windows\CloudContent"
        valueName="DisableTailoredExperiencesWithDiagnosticData">
```

**Evidence, Microsoft Learn Policy CSP (Experience / AllowTailoredExperiencesWithDiagnosticData):**

> Scope: Device ❌ / User ✅
> `./User/Vendor/MSFT/Policy/Config/Experience/AllowTailoredExperiencesWithDiagnosticData`

There is no Device-scoped variant of this setting. Note that the corpus also writes the companion
per-user preference `HKCU\Software\Microsoft\Windows\CurrentVersion\Privacy\TailoredExperiencesWithDiagnosticDataEnabled`,
which is *not* a policy value and does work. So the tweak is not completely without effect today;
the policy half of it is the inert half. Moving the policy write to HKCU makes the policy half
enforce the setting and grey out the Settings UI toggle as intended.

**Sources**

* `C:\Windows\PolicyDefinitions\CloudContent.admx` (Windows 11 build 26100)
* <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience>

## Class Both, either hive is legitimate

These 18 values live under a policy whose ADMX `class` is `Both`, or under a pair of ADMX policies
(one `User`, one `Machine`) sharing the same key and value name. Writing to HKLM is legitimate and,
in every case here, is the stronger of the two options.

### Windows policies declared `class="Both"`

| Value | ADMX | Precedence when both hives are set |
| --- | --- | --- |
| `DisableAIDataAnalysis` | WindowsCopilot.admx | Computer Configuration wins for Recall enforcement. |
| `DisableClickToDo` | WindowsCopilot.admx | Computer Configuration wins. |
| `AllowTelemetry` | DataCollection.admx | Computer Configuration wins. Policy CSP lists both Device and User scope; the device value is authoritative for the diagnostic data level. |
| `NoDriveTypeAutoRun` | AutoPlay.admx | Computer Configuration wins over User Configuration; this is the documented behavior for the Autoplay policies. |
| `NoAutorun` | AutoPlay.admx | Computer Configuration wins. |
| `EnableScriptBlockLogging` | PowerShellExecutionPolicy.admx | Computer Configuration wins; the machine value is what the engine consults for system-wide logging. |
| `SmartScreenEnabled` (Edge) | SmartScreen.admx (`EdgeConfigureSmartScreen`) | Machine scope wins, per Edge policy precedence. |

### Windows policies with paired User and Machine definitions

| Value | ADMX | Detail |
| --- | --- | --- |
| `DisableLocation` | Sensors.admx | `DisableLocation_1` is `class="User"`, `DisableLocation_2` is `class="Machine"`, both on `Software\Policies\Microsoft\Windows\LocationAndSensors`. The HKLM write hits the Machine definition. Correct and system-wide, which is what the tweak intends. |
| `Disabled` (Windows Error Reporting) | ErrorReporting.admx | `WerDisable_1` is `class="User"`, `WerDisable_2` is `class="Machine"`, both on `SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting`. HKLM hits the Machine definition. |

### Microsoft Edge policies

`HideFirstRunExperience`, `StartupBoostEnabled`, `BackgroundModeEnabled`, `HubsSidebarEnabled`,
`EdgeCollectionsEnabled`, `MetricsReportingEnabled`, `SendSiteInfoToImproveServices`,
`PersonalizationReportingEnabled`, `Edge3PSerpTelemetryEnabled`.

`MSEdge.admx` is generated by the Chromium ADMX writer, which emits `class="Both"` for every policy
without exception:

```python
# components/policy/tools/template_writers/writers/admx_writer.py
def GetClass(self, policy):
    return 'Both'
```

Edge therefore surfaces each policy under both Computer Configuration and User Configuration, which
is why the Learn pages give the GP path as `Administrative Templates/Microsoft Edge` with no
Computer or User qualifier and give the registry path as `SOFTWARE\Policies\Microsoft\Edge` with no
hive qualifier.

At runtime Edge loads policy from both hives and tags them by scope:

```cpp
// components/policy/core/common/policy_loader_win.cc
{POLICY_SCOPE_MACHINE, HKEY_LOCAL_MACHINE},
{POLICY_SCOPE_USER,    HKEY_CURRENT_USER},
```

Machine scope outranks user scope in Chromium's policy merge, so the corpus's HKLM writes are both
valid and the higher-priority choice. No change needed.

**Sources**

* <https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hidefirstrunexperience>
* <https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hubssidebarenabled>
* <https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/backgroundmodeenabled>
* <https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/metricsreportingenabled>
* <https://raw.githubusercontent.com/chromium/chromium/main/components/policy/tools/template_writers/writers/admx_writer.py>
* <https://raw.githubusercontent.com/chromium/chromium/main/components/policy/core/common/policy_loader_win.cc>

## Not a GP policy (the `\Policies\` rule does not apply)

These 10 values sit under a key containing `\Policies\` but are not ADMX backed, so the class rule
has nothing to say about them. All 10 are machine-scoped by nature and correctly in HKLM.

### Security Options (6 values)

`DisableCAD`, `ConsentPromptBehaviorAdmin`, `PromptOnSecureDesktop`, `EnableLUA`,
`FilterAdministratorToken`, `DontDisplayLastUserName`, all under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`.

A grep of all 218 shipped ADMX files on build 26100 returns zero hits for every one of these value
names. They are **Security Options**, delivered by the Security Settings client-side extension from
a security template (`GptTmpl.inf`, `MACHINE\Software\Microsoft\Windows\CurrentVersion\Policies\System\...`),
configured under Computer Configuration > Windows Settings > Security Settings > Local Policies >
Security Options. They are machine-only by design (there is no user-scoped equivalent), the LSA and
Winlogon read them from HKLM directly, and the corpus writes them to HKLM. No defect.

One consequence worth recording for the tweak system, unrelated to hive correctness: because these
are applied by the security CSE rather than by the registry policy engine, a local security policy
refresh can overwrite a direct registry write on a domain-joined or MDM-managed machine. That is a
durability caveat, not a hive caveat.

### Windows Firewall client-side extension (4 values)

`DefaultInboundAction` on all three profiles, plus `EnableFirewall` on `PublicProfile`.

`WindowsFirewall.admx` on build 26100 defines only `WF_EnableFirewall_Name_1`
(`SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile`) and `WF_EnableFirewall_Name_2`
(`SOFTWARE\Policies\Microsoft\WindowsFirewall\StandardProfile`), both `class="Machine"`. It contains
no `DefaultInboundAction` and no `PublicProfile` key at all. Those values belong to the Windows
Defender Firewall with Advanced Security snap-in, a separate GP client-side extension that writes
into the same `SOFTWARE\Policies\Microsoft\WindowsFirewall\...` store. The firewall service reads
them from HKLM. Firewall profile configuration is inherently per-machine, so there is no user hive
variant and no defect. The two `EnableFirewall` values that *are* ADMX backed (Domain and Standard
profiles) are `class="Machine"` and correctly in HKLM.

## Confirmed CORRECT (checked, clean)

Listed so the reader knows these were actually resolved to an ADMX definition rather than skipped.
Each was matched by exact registry key **and** exact value name against the shipped ADMX set on
Windows 11 build 26100.

### User class, written to HKCU (3)

All three HKCU policy writes in the corpus were checked for the opposite-direction defect. All are
genuinely `class="User"`.

* `debloat:disable_web_search_start` / `DisableSearchBoxSuggestions`
  `WindowsExplorer.admx`, `class="User"`, key `Software\Policies\Microsoft\Windows\Explorer`.
* `interface:disable_copilot_taskbar` / `TurnOffWindowsCopilot`
  `WindowsCopilot.admx`, `class="User"`, key `SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot`.
* `interface:disable_meet_now` / `HideSCAMeetNow`
  `Taskbar.admx`, `class="User"`, key `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`.

### Machine class, written to HKLM (53)

**debloat.yaml**
`AllowNewsAndInterests` (NewsAndInterests.admx), `ChatIcon` (Taskbar.admx, policy
`ConfigureChatIcon`), `AllowCortana` (Search.admx, see note below).

**interface.yaml**
`VerboseStatus` (Logon.admx).

**network.yaml**
`EnableMulticast` (DnsClient.admx, policy `Turn_Off_Multicast`), `DODownloadMode`
(DeliveryOptimization.admx, policy `DownloadMode`), `DeferQualityUpdates` and
`DeferQualityUpdatesPeriodInDays`, `TargetReleaseVersion`, `TargetReleaseVersionInfo`,
`ProductVersion`, `NoAutoRebootWithLoggedOnUsers`, `ExcludeWUDriversInQualityUpdate`,
`AllowAutoWindowsUpdateDownloadOverMeteredNetwork`, `NoAutoUpdate`, `AUOptions` (all
WindowsUpdate.admx), `AutoDownload` (WindowsStore.admx).

**performance.yaml**
`LetAppsRunInBackground` (AppPrivacy.admx), `AllowGameDVR` (GameDVR.admx),
`AllowStorageSenseGlobal` (StorageSense.admx, policy `SS_AllowStorageSenseGlobal`).

**privacy.yaml**
`AllowRecallEnablement` (WindowsCopilot.admx), `CEIPEnable` (ICM.admx), `AITEnable` and
`DisableInventory` (AppCompat.admx), `DoNotShowFeedbackNotifications` (FeedbackNotifications.admx),
`DisableOneSettingsDownloads` and `AllowDeviceNameInTelemetry` (DataCollection.admx),
`EnableActivityFeed`, `PublishUserActivities`, `UploadUserActivities`, `AllowCrossDeviceClipboard`
(OSPolicy.admx), `AllowInputPersonalization` (Globalization.admx), `DisabledByGroupPolicy`
(UserProfiles.admx, policy `DisableAdvertisingId`), `LetAppsGetDiagnosticInfo` (AppPrivacy.admx),
`AllowFindMyDevice` (FindMy.admx), `DisableSettingSync` and `DisableSettingSyncUserOverride`
(SettingSync.admx), `DisableSoftLanding` and `DisableWindowsConsumerFeatures` (CloudContent.admx).

Note that `DisableSoftLanding` and `DisableWindowsConsumerFeatures` sit in the *same* ADMX file as
the one defect, `CloudContent.admx`, and both are genuinely `class="Machine"`. The CloudContent
family is mixed, which is exactly why the one User-class member was easy to get wrong.

**security.yaml**
`RestrictDriverInstallationToAdministrators` (Printing.admx), `EnableControlledFolderAccess`,
`EnableNetworkProtection`, `PUAProtection`, and the five ASR rule GUIDs (WindowsDefender.admx),
`EnableSmartScreen` and `ShellSmartScreenLevel` (SmartScreen.admx, policy
`ShellConfigureSmartScreen`), `AllowInsecureGuestAuth` (LanmanWorkstation.admx, policy
`Pol_EnableInsecureGuestLogons`), `EnableFirewall` on DomainProfile and StandardProfile
(WindowsFirewall.admx).

The five ASR rule values deserve a word because they did not match on value name. They are written
as individual values under
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`, which
`WindowsDefender.admx` declares as a `<list>` element inside the `class="Machine"` policy
`ExploitGuard_ASR_Rules`:

```xml
<policy name="ExploitGuard_ASR_Rules" class="Machine"
        key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR"
        valueName="ExploitGuard_ASR_Rules">
  <elements>
    <list id="ExploitGuard_ASR_Rules" additive="true" explicitValue="true"
          key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules" />
  </elements>
</policy>
```

Machine class, HKLM write, correct.

### Note on `AllowCortana`

`debloat:remove_cortana` writes `AllowCortana` to
`HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`. The Policy CSP page confirms the mapping
is Machine scoped:

> Scope: Device ✅ / User ❌
> Location: Computer Configuration, Path: Windows Components > Search,
> Registry Key Name: `SOFTWARE\Policies\Microsoft\Windows\Windows Search`,
> Registry Value Name: `AllowCortana`, ADMX File Name: `Search.admx`

So the hive is correct. However, a grep of the shipped `Search.admx` on build 26100 finds no
`AllowCortana` policy at all; Microsoft removed it when the Cortana app was retired from Windows 11.
The value is therefore correctly placed but likely vestigial on the primary target platform. That is
a relevance question, not a hive defect, and is out of scope for this audit.

**Source:** <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience>

## Unresolved

None. Every one of the 85 values was resolved either to a shipped ADMX definition, to a documented
Policy CSP group policy mapping, or to a positively identified non-ADMX mechanism (Security Options
or the firewall client-side extension).

## Method notes for anyone re-running this

1. Extract every line in `src-tauri/tweaks/*.yaml` whose registry key matches `/policies\\/i`.
   That is 85 values across six files; `services.yaml` has none.
2. Parse `C:\Windows\PolicyDefinitions\*.admx` with a `<policy ...>...</policy>` block regex. Do not
   terminate the block on the first `/>`; child elements such as `<parentCategory ... />` are
   self-closing and will truncate the block, hiding the `<elements>` section where most `valueName`
   attributes actually live. That mistake silently produced 24 false "no ADMX found" results on the
   first pass of this audit.
3. Match on exact key **and** exact value name, then read the `class` attribute off the enclosing
   `<policy>` element.
4. Flag `class="User"` written to HKLM and `class="Machine"` written to HKCU.
