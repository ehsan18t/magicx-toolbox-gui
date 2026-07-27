# Adversarial verification: the 12 High-value gap proposals

Verifies the twelve proposals ranked **High** in `_gaps-privacy-debloat-interface.md`. Written to
refute, not to confirm: each entry was attacked on existence, key/name/type accuracy, value polarity,
duplication, applicability, shared-value collision, and whether the benefit is real.

**Verification platform.** Local box is Windows 11 **26100.4061**, `DisplayVersion` 24H2,
`EditionID` `IoTEnterpriseS` (IoT Enterprise LTSC 2024). Per the brief the machine's own registry,
service start types and task state are **not** treated as evidence of any Windows default. What is
used as evidence:

- `C:\Windows\PolicyDefinitions\*.admx` and `en-US\*.adml` (describe the product, not this machine)
- Presence of a value-name string inside a shipped Windows binary, which proves the OS has code that
  looks the name up. Absence is weaker evidence but is reported honestly.
- Microsoft Learn (Policy CSP, Edge policy reference, Start policy settings, Notepad management)
- The corpus YAML itself, for the duplication check

**Binary sweep coverage.** 8,194 binaries under `C:\Windows\System32` (depth 3) and
`C:\Windows\SystemApps` (depth 4), 5,207 under `C:\Program Files\WindowsApps`, plus 1,359 in
shell-related `C:\Windows\WinSxS` component directories. Both UTF-16LE and ASCII encodings.

---

## Verdict summary

| # | Proposed id | Verdict | One-line reason |
|---|---|---|---|
| 1 | `disable_start_recommended_section` | **CONFIRMED** | ADMX + CSP + read by `StartMenu.dll` and `SHCore.dll` on 26100; the SE `supportedOn` is cosmetic |
| 2 | `disable_explorer_cloud_recommendations` | **CONFIRMED** | ADMX exact; read by `shell32.dll`, `windows.storage.dll`, `MicrosoftGraphRecentItemsManager.dll` |
| 3 | `disable_account_notifications` | **CONFIRMED** | ADMX exact (class User); read by `windowsudk.shellcommon.dll`; Learn documents the notification classes |
| 4 | `disable_app_device_inventory` | **CONFIRMED** | All four names exact incl. the `DisableAPISamping` misspelling; each read by its own collector binary |
| 5 | `disable_ai_fabric_service` | **CORRECTED** | The service **does exist on 26100**. The `build >= 26200` gate and the display name are both wrong |
| 6 | `disable_notepad_ai` | **CONFIRMED** | Key and value found verbatim inside `Notepad.exe` 11.2604.5.0; Learn confirms hive and semantics |
| 7 | `disable_paint_ai` | **CONFIRMED** | ADMX + Policy CSP agree on all three; the two extra Win11Debloat values are correctly rejected |
| 8 | `disable_settings_account_ads` | **CORRECTED** | Real, but applicability is Windows 11 21H2+ (not Win10 2004) and it is a **no-op on Pro and Home** |
| 9 | `disable_phone_companion_start` | **CONFIRMED** | `Start\Companions` and `IsEnabled` both read by `StartMenu.dll` on 26100, plus 3 independent sources for hive and PFN; `requires_reboot` should be false |
| 10 | `disable_drag_tray` | **CORRECTED** | Value is real and correct, but the feature was **renamed Drop Tray** and now has a Settings toggle; one cited source is bogus |
| 11 | `hide_unsupported_hardware_notice` | **CONFIRMED** | ADMX exact; read by `shell32.dll` (watermark) and `AboutSettingsHandlers.dll` (Settings banner) |
| 12 | `disable_edge_ai_features` | **CORRECTED** | Three of the five are documented **Entra-only** and change nothing on a consumer MSA profile |

**Score: 8 confirmed as written, 4 corrected, 0 rejected.** No proposal was found to be fabricated,
which is itself notable. The three findings that matter most are #5 (wrong applicability gate, would
skip the tweak on the primary target platform), #12 (three values that would satisfy the write,
report success, and produce no user-visible change on the machines this product targets), and #10
(a cited source that does not support the claim at all).

---

## Cross-cutting findings

### Duplication check: the proposer's claim holds

The claim that none of the twelve duplicates existing corpus surface was independently re-verified by
parsing all seven files in `src-tauri/tweaks/` and extracting every registry, service and task effect
(293 effects). Results:

- `HideRecommendedSection`, `DisableGraphRecentItems`, `DisableAccountNotifications`,
  `DisableConsumerAccountStateContent`, `HideUnsupportedHardwareNotifications`, `DragTrayEnabled`,
  `Start\Companions`, `WSAIFabricSvc`, the Notepad key, the Paint key: **zero** occurrences anywhere
  in the corpus.
- `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat` is already used by `disable_compat_appraiser`
  (`privacy.yaml:191,193`) but only for `AITEnable` and `DisableInventory`. Different value names.
- `HKLM\SOFTWARE\Policies\Microsoft\Edge` is already used by five tweaks (`disable_edge_first_run`,
  `disable_edge_startup_boost`, `disable_edge_sidebar`, `disable_edge_telemetry`,
  `enforce_smartscreen`) but none of the five proposed AI value names appear.
- `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent` is already used by three tweaks but not for
  `DisableConsumerAccountStateContent`.

Verdict: the non-duplicate claim is **verified for all twelve**.

### Shared-value collision risk: none among these twelve

The proposer flagged `SettingsPageVisibility` (proposal 21, Medium, not in this batch) as a single
shared `REG_SZ` that two tweaks would clobber. That failure mode requires two owners of the **same
value**, not the same key.

The engine snapshots and restores per value, not per key: `BrokerOp` carries `value_name` through
`SetValue` and `DeleteValue` (`src-tauri/src/services/elevation/broker.rs:32,40,137`), and
`registry_service.rs` opens per value name. So sharing a key with another tweak is not a collision.

Every one of the twelve writes value names that no other corpus tweak owns, and none of the twelve
writes a composed list-semantics string. **No collision risk in this batch.** The keys that are
shared with existing tweaks (`AppCompat`, `Edge`, `CloudContent`, `Policies\System`) are safe under
per-value snapshotting.

One adjacent caution, not a collision: proposal 3 suggests also writing the non-policy sibling
`HKCU\...\Explorer\Advanced` `Start_AccountNotifications`. That value **is** real (found in
`StartDocked.dll`), but it is the value the Settings UI itself writes. If the tweak owns it, a user
toggling the Settings switch silently drifts the tweak out of its recorded state. Ship the policy as
the single effect, or accept the drift and say so.

### "Stock Default = absent" is correct in every case, and for two of them it is the only defined state

Every one of these policies is `absent` when not configured, so the proposed revert option is right
throughout. Two need an extra note:

`HideRecommendedSection` and `HideUnsupportedHardwareNotifications` are the only two whose ADMX
`<policy>` element declares **no `enabledValue` and no `disabledValue`**:

```xml
<policy name="HideUnsupportedHardwareNotifications" class="Machine"
        key="Software\Microsoft\Windows\CurrentVersion\Policies\System"
        valueName="HideUnsupportedHardwareNotifications">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_NOSERVER" />
</policy>
```

For such a policy there is no Microsoft-defined "0" state at all. The revert **must** be `absent`;
writing an explicit `0` is undefined behaviour that no source covers. The proposals already say
`absent`, which is correct, but the YAML must not be "helpfully" changed to write `0` later.

### The 24H2 ADML `supportedOn` strings, resolved

Pulled from the shipped `en-US\Windows.adml` on 26100, because two proposals cite these labels:

| Token | Shipped display string |
|---|---|
| `SUPPORTED_Windows_11_0_SE` | Windows 11 SE |
| `SUPPORTED_Windows_11_0_22H2_NOSERVER` | At least Windows 11 Version 22H2 |
| `SUPPORTED_Windows_11_0_24H2` | At least Windows 11 Version 24H2 |
| `SUPPORTED_Windows_11_0_NOSERVER` | At least Windows 11 |
| `SUPPORTED_Windows_10_0_20H1_NOSERVER` | At least Windows 10 Version 2004 |
| `SUPPORTED_Windows_10_0_RS7` | At least Windows Server 2016, Windows 10 Version 1909 |

Note the last row: the proposal for #8 glossed `SUPPORTED_Windows_10_0_RS7` as "Windows 10 2004".
The shipped string says 1909. See entry 8.

---

## Entry 1. `disable_start_recommended_section` (interface)

**Verdict: CONFIRMED.** Ship as proposed, and **drop the PolicyManager fallback**.

**Mechanism as proposed:** `HKLM` and `HKCU` `SOFTWARE\Policies\Microsoft\Windows\Explorer`,
`HideRecommendedSection`, `REG_DWORD`, Hidden = `1`, Stock Default = `absent`.

**Attack 1, does it do anything on 26100?** Yes. The literal string `HideRecommendedSection` is
present in two shipped binaries on this 26100 machine:

- `C:\Windows\SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\StartMenu.dll`
- `C:\Windows\System32\SHCore.dll`

In `StartMenu.dll` it sits inside a contiguous run of Start policy value names:

```
...\CurrentVersion\Themes\Personalize | ColorPrevalence | HideAppList |
NoStartMenuMorePrograms | HideRecommendedSection | DisableContextMenus | ...
```

`HideAppList`, `NoStartMenuMorePrograms` and `DisableContextMenus` are all known-working Start
policies. `HideRecommendedSection` is read by the same code path.

**Attack 2, the Windows 11 SE `supportedOn` problem.** The proposal raised this as a risk and
proposed an MDM-cache fallback. Resolved against it. The shipped `StartMenu.admx` on 26100 does say:

```xml
<policy name="HideRecommendedSection" class="Both"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="HideRecommendedSection">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_SE" />
</policy>
```

`supportedOn` is a **gpedit display annotation only**. It controls the "Requirements:" line in the
policy editor. It does not gate what the OS reads, and the OS binary that reads the value is
`StartMenu.dll`, which is on a non-SE IoT Enterprise image here. Microsoft's own Policy CSP
contradicts the ADMX and is authoritative on editions:

> Editions: Pro / Enterprise / Education / IoT Enterprise / IoT Enterprise LTSC (all supported).
> Applicable OS: Windows 11, version 22H2 [10.0.22621] and later.
> Registry Key Name: `Software\Policies\Microsoft\Windows\Explorer`
> Registry Value Name: `HideRecommendedSection`
> Location: Computer and User Configuration
> Allowed values: `0 (Default)` Recommended section shown; `1` Recommended section hidden.

**Attack 3, polarity.** Not inverted. CSP is explicit: `1` hides. Matches the proposal.

**Attack 4, duplicate?** No. `disable_start_recommendations` writes `Start_IrisRecommendations`
(`interface.yaml:595`) under `Explorer\Advanced`. Different key, different value, different scope.
The proposer's distinction (promo rows versus the whole section) is accurate.

**Correction to make:** the proposal's fallback suggestion, writing
`HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Start`, is **unnecessary and should not ship**.
It is the MDM policy cache, which the proposal's own rejected list correctly bans as an authoring
surface. The direct policy value is read on Pro-class editions. UNKNOWN #6 is resolved.

**Applicability:** Windows 11 22H2 (22621) and later, so all of 26100 and 26200, all editions listed
above. Not applicable to LTSC 2021 (Windows 10 Start layout).

**Reboot:** none, but `StartMenuExperienceHost.exe` must restart (sign out or kill the process) for
the layout to reflow. The proposal did not mention this; the YAML should.

**Sources:** `C:\Windows\PolicyDefinitions\StartMenu.admx` and `en-US\StartMenu.adml` on 26100
(tier A); Policy CSP - Start > HideRecommendedSection,
https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start (tier A);
binary string evidence in `StartMenu.dll` and `SHCore.dll` on 26100.4061 (tier A, product artifact).

---

## Entry 2. `disable_explorer_cloud_recommendations` (privacy)

**Verdict: CONFIRMED.** Ship exactly as proposed.

**Mechanism as proposed:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`,
`DisableGraphRecentItems`, `REG_DWORD`, Disabled = `1`, Stock Default = `absent`.

**Attack 1, existence.** Confirmed twice over. Shipped `Explorer.admx` on 26100:

```xml
<policy name="DisableGraphRecentItems" class="Machine"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="DisableGraphRecentItems">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2_NOSERVER" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

And the value name is read by three shipped binaries:

- `C:\Windows\System32\shell32.dll`
- `C:\Windows\System32\windows.storage.dll`
- `C:\Windows\SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\MicrosoftGraphRecentItemsManager.dll`
- `C:\Windows\SystemApps\MicrosoftWindows.Client.FileExp_cw5n1h2txyewy\FileExplorerExtensions.dll`

The third of those is named for exactly the subsystem the policy claims to govern. This is about as
strong as binary evidence gets.

**Attack 2, key/name/type/polarity.** All exact. `class="Machine"` means HKLM only, which the
proposal has right (it does not propose an HKCU write). `enabledValue` `1` confirms `1` = off.

**Attack 3, is the claimed effect real or cosmetic?** Real, and the proposal's framing is accurate.
The 26100 ADML is a network claim, not a display claim:

> Turning off this setting will prevent File Explorer from requesting cloud file metadata and
> displaying it in the homepage and other views in File Explorer. Any insights and files available
> based on account activity will be stopped in views such as Recent, Recommended, Favorites, Details
> pane, etc.

**Attack 4, duplicate?** No. `disable_recent_files` writes `ShowRecent` / `ShowFrequent`
(`interface.yaml:266,268`) under `Explorer\Advanced`, which is the local MRU. `remove_home_nav_pane`
hides the Home node. Neither stops the Graph request. Confirmed distinct.

**Applicability:** Windows 11 22H2 and later, client only, machine scope. Not applicable to LTSC 2021.

**Sources:** `Explorer.admx` / `en-US\Explorer.adml` on 26100 (tier A); binary string evidence above
(tier A, product artifact).

---

## Entry 3. `disable_account_notifications` (debloat)

**Verdict: CONFIRMED.** Ship the policy value. Treat the `Start_AccountNotifications` sibling as
optional and read the caution.

**Mechanism as proposed:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`,
`DisableAccountNotifications`, `REG_DWORD`, Disabled = `1`, Stock Default = `absent`.

**Attack 1, existence and exactness.** Shipped `AccountNotifications.admx` on 26100:

```xml
<policy name="DisableAccountNotifications" class="User"
        key="SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications"
        valueName="DisableAccountNotifications">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_20H1_NOSERVER" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

`class="User"` confirms HKCU. Key path, value name, type and polarity all match the proposal exactly.
The value name also appears in `C:\Windows\System32\windowsudk.shellcommon.dll`, so shipped shell
code looks it up.

**Attack 2, is the described effect real?** Yes, and the proposal understated nothing. The 26100
ADML enumerates the notification classes verbatim:

> This policy allows you to prevent Windows from displaying notifications to Microsoft account (MSA)
> and local users in Start (user tile). Notifications include getting users to: reauthenticate;
> backup their device; manage cloud storage quotas as well as manage their Microsoft 365 or XBOX
> subscription. [...] No reboots or service restarts are required for this policy setting to take
> effect.

Microsoft Learn's Start policy settings page repeats the same list and maps it to
`./User/Vendor/MSFT/Policy/Config/Notifications/DisableAccountNotifications`, GPO path
User Configuration > Administrative Templates > Windows Components > Account Notifications.

**Attack 3, duplicate?** No. The corpus's `disable_scoobe_nag` (`debloat.yaml:124`) writes
`ScoobeSystemSettingEnabled` under `UserProfileEngagement`, and the ContentDeliveryManager tweaks
cover subscribed-content slots. Neither touches the user-tile notification channel.

**Correction / caution to encode:** `requires_reboot` must be **false**. The ADML states plainly that
no reboot or service restart is needed. Also, if the tweak additionally writes the non-policy sibling
`HKCU\...\Explorer\Advanced` `Start_AccountNotifications` (confirmed present in `StartDocked.dll`),
be aware that this is the value the Settings toggle owns. A user flipping that switch in Settings
will silently desynchronise the tweak from its snapshot. Prefer the policy value alone.

**Applicability:** Windows 10 2004 and later, client only, user scope. Applies to 26100, 26200 and
LTSC 2021.

**Sources:** `AccountNotifications.admx` / `.adml` on 26100 (tier A); Microsoft Learn, Start menu
policy settings > Disable Account Notifications,
https://learn.microsoft.com/en-us/windows/configuration/start/policy-settings (tier A); binary string
evidence in `windowsudk.shellcommon.dll` (tier A, product artifact).

---

## Entry 4. `disable_app_device_inventory` (privacy)

**Verdict: CONFIRMED.** This is the best-evidenced proposal in the batch. Ship as written.

**Mechanism as proposed:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat`, four `REG_DWORD`
values, each Disabled = `1`, each Stock Default = `absent`.

**Attack 1, do the names exist and is the odd spelling real?** Yes, and yes. All four are in the
shipped `AppDeviceInventory.admx` on 26100, each `class="Machine"`, each on key
`Software\Policies\Microsoft\Windows\AppCompat`, each `supportedOn ref="windows:SUPPORTED_Windows_11_0_24H2"`,
each `enabledValue` `1` / `disabledValue` `0`. Microsoft even left the intent comments in the file:

```xml
<policy name="TurnOffAPISamping" class="Machine"
        key="Software\Policies\Microsoft\Windows\AppCompat" valueName="DisableAPISamping">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_24H2" />
  <!-- "Enabled" here means we are turning off API Sampling. -->
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

The `DisableAPISamping` misspelling is genuine: a search for `DisableAPISampling` (correct spelling)
across every ADMX on 26100 returns **nothing**. The proposal's warning is right and must survive into
the YAML.

Note the ADMX `name=` attributes differ from the value names (`TurnOffInstallTracing`,
`TurnOffAPISamping`, `TurnOffApplicationFootprint`, `TurnOffWin32AppBackup`). Do not write those into
the registry.

**Attack 2, does anything actually read them?** Yes, and each one is read by a binary named for its
own collector. This is the decisive result:

| Value name | Shipped binary that contains the string |
|---|---|
| `DisableInstallTracing` | `System32\installmon.dll` |
| `DisableAPISamping` | `System32\apisampling.dll` |
| `DisableApplicationFootprint` | `System32\appfootprint.dll`, `System32\pcasvc.dll` |
| `DisableWin32AppBackup` | `System32\aeinv.dll`, `System32\aemarebackup.dll`, `System32\appraiser.dll` |

Four distinct collectors, four distinct binaries, all present on 26100. These are live controls, not
dead policy names.

**Attack 3, polarity.** All four are `enabledValue` `1`, and the ADML help text for each confirms
"If you enable this policy, X will not be run." `1` = collector off. No inversion.

**Attack 4, duplicate?** No. `disable_compat_appraiser` (`privacy.yaml:191,193`) writes `AITEnable`
and `DisableInventory` under the same key. Different value names, and per-value snapshotting means no
interference. The proposer's claim that these are a distinct 24H2-era collection family is confirmed
by the dedicated `AppDeviceInventory` category and the 24H2-only `supportedOn`.

**Applicability:** Windows 11 24H2 and later, machine scope. Gate `build >= 26100`. **Not** present on
LTSC 2021 or on pre-24H2 Windows 11.

**Downside, stated honestly:** `DisableWin32AppBackup` turns off the compatibility scan over
backed-up applications, which is the scan that runs when restoring apps from Windows Backup. The
proposal flagged this. Keep it in the tweak copy.

**Sources:** `AppDeviceInventory.admx` / `en-US\AppDeviceInventory.adml` on 26100 (tier A); binary
string evidence in the seven collector binaries above (tier A, product artifact).

---

## Entry 5. `disable_ai_fabric_service` (debloat)

**Verdict: CORRECTED.** The service is real, but the proposal's applicability is **wrong in a way
that would make the tweak skip the primary target platform**, and the display name is wrong.

**What the proposal claims:** service `WSAIFabricSvc`, display name "Windows AI Fabric Service",
Windows 11 25H2 (26200) and later only, "the service does not exist on 26100 or LTSC 2021, so the
tweak must be gated `build >= 26200`".

**Refutation.** `WSAIFabricSvc` **exists on Windows 11 24H2 build 26100**. The service is registered
on this 26100.4061 machine and its host binary is a Microsoft-shipped 26100 file:

```
C:\Windows\System32\WSAIFabricHost.dll
  FileVersion    : 10.0.26100.3624 (WinBuild.160101.0800)
  ProductVersion : 10.0.26100.3624
  FileDescription: WSAIFabricSvc
  (timestamp May 2025)
```

Service registration:

```
ImagePath   : %systemroot%\system32\svchost.exe -k WSAIFabricSvcGroup -p
ObjectName  : NT AUTHORITY\LocalService
Type        : 0x20 (shared svchost process)
DependOnService : RPCSS, BrokerInfrastructure
Description : @%systemroot%\system32\WSAIFabricHost.dll,-103
DisplayName : @%systemroot%\system32\WSAIFabricHost.dll,-102
```

The registration and the shipped DLL are product artifacts, not owner modifications. Nobody
hand-authors a `WSAIFabricHost.dll` with a Microsoft version resource. The file version pins the
arrival: the service came to 24H2 in servicing around **26100.3624**, not only in 25H2.

**Correction 1, the display name.** Resolving the MUI resource
(`System32\en-US\WSAIFabricHost.dll.mui`) gives:

> DisplayName: `WSAIFabricSvc`
> Description: `Provides support to communicate with AIFabric in Local service context over COM.`

The display name is literally `WSAIFabricSvc`, **not** "Windows AI Fabric Service". If the corpus
matches or displays on the friendly name, it will not match.

**Correction 2, the gate.** `build >= 26200` is wrong. Correct gate is `build >= 26100` **plus a
presence check that fails cleanly when the service is absent**, because early 26100 servicing levels
below roughly 26100.3624 will not have it. The corpus's service effect must treat "service not
registered" as a skip, not an error.

**Attack on the claimed benefit.** The proposal's justification is the `WorkloadsSessionHost.exe`
memory story ("eight processes, multi-gigabyte resident"). It does not generalise:

- A recursive search of all of `C:\Windows` for `WorkloadsSessionHost*` on this 26100.4061 host
  returns **nothing**, and zero such processes run even though `WSAIFabricSvc` itself is running.
- `WSAIFabricHost.dll` does **not** contain the strings `WorkloadsSessionHost`, `WorkloadSession` or
  `SessionHost` in either encoding. It does contain `AIFabric` and `Workloads`.
- The reports that do exist are all from NPU-equipped or Copilot+ machines. A Microsoft Q&A thread on
  26200.8894 measured 8 instances at 2,449 MB private / 4,454 MB working set, dropping to zero after
  `Stop-Service WSAIFabricSvc -Force`; ElevenForum threads report 5 to 8 GB alongside an AMD NPU
  interaction tracked in `microsoft/onnxruntime-genai` issue 2013.

So the **service is universal but the memory symptom is NPU / Copilot+ specific**. The tweak copy
must not promise a multi-gigabyte win on ordinary hardware. On a non-NPU 24H2 machine this is a
"stop a COM broker for AI workloads from auto-starting" tweak, which is a legitimate control under
the corpus's inclusion principle but is not the resource reclaim the proposal advertises.

**The default start type, now resolved well enough to write the revert option.** The proposal's
"Automatic (Stock Default)" holds. Three lines of evidence, none of which is this machine's own
configured state:

1. **Win11Debloat's own undo file** writes it back to Automatic. `Regfiles/Disable_AI_Service_Auto_Start.reg`
   sets `"Start"=dword:00000003` and `Regfiles/Undo/Enable_AI_Service_Auto_Start.reg` sets
   `"Start"=dword:00000002`. A project's restore-to-default artifact is a statement about the default.
2. The Microsoft Q&A 25H2 thread's remedy is `Set-Service WSAIFabricSvc -StartupType Manual` with the
   note that this "prevents the service from starting automatically after reboot", which only makes
   sense if the shipped state is Automatic.
3. ElevenForum users on 25H2 report the service re-enabling itself after reboot when disabled.

It is also **not** trigger-started (`sc qtriggerinfo` reports no registered triggers) and **not**
delayed-auto (no `DelayedAutostart` value in the service key). So the revert option is a plain
Automatic.

**Counter-evidence, weighed and rejected.** `batcmd.com` and `revertservice.com`, both cited by the
proposal, state the 24H2 start type is **Manual**. They are contradicted by every other line of
evidence, they publish the identical wrong value, and neither tracks 25H2 at all. Two auto-generated
service catalogs carrying the same wrong figure is the signature of a common upstream, so they count
as **one derivative source, and it is wrong**. Drop both from the proposal's Sources line.

Setting Manual rather than Disabled remains the right applied value.

**Duplicate?** No. `WSAIFabricSvc` appears nowhere in `services.yaml` or any other corpus file.

**Sources:** shipped `C:\Windows\System32\WSAIFabricHost.dll` version resource and
`en-US\WSAIFabricHost.dll.mui` string table on 26100.4061 (tier A, product artifact); service
registration under `HKLM\SYSTEM\CurrentControlSet\Services\WSAIFabricSvc` (used as evidence of
**existence and of registration shape**, not of the owner's configured start type); negative binary
search for `WorkloadsSessionHost` across `C:\Windows`; Win11Debloat
`Regfiles/Disable_AI_Service_Auto_Start.reg` and `Regfiles/Undo/Enable_AI_Service_Auto_Start.reg`
(tier C, read as raw `.reg` bodies); Win11Debloat issue 265 (2025-06-25, predates 25H2 GA, reports
the service on 24H2, and the maintainer confirms the action is Manual rather than Disabled) and issue
497 (tier C); Microsoft Q&A 5955428 on 26200.8894 (tier C); ElevenForum thread 45675 via the Wayback
Machine (tier C). Explicitly **rejected as sources**: `batcmd.com` and `revertservice.com`, which are
one derivative source and state a start type contradicted by all other evidence.

---

## Entry 6. `disable_notepad_ai` (debloat)

**Verdict: CONFIRMED.** The unusual key path is correct exactly as proposed.

**Mechanism as proposed:** `HKLM\SOFTWARE\Policies\WindowsNotepad`, `DisableAIFeatures`, `REG_DWORD`,
Disabled = `1`, Stock Default = `absent`.

**Attack 1, that key path looks wrong.** It looks wrong (no `Microsoft` component) and it is right.
The literal bytes inside the shipped Notepad binary settle it. From
`C:\Program Files\WindowsApps\Microsoft.WindowsNotepad_11.2604.5.0_x64__8wekyb3d8bbwe\Notepad\Notepad.exe`:

```
SOFTWARE\Policies\WindowsNotepad | DisableAIFeatures | ... | 0 | 1 |
```

The key path and the value name are adjacent literals in the same string block, followed by the `0`
and `1` literals. Microsoft Learn agrees:

> To disable AI features in Notepad, set the DisableAIFeatures registry value to 1 under
> `HKLM:\SOFTWARE\Policies\WindowsNotepad`.

**Attack 2, is there a separate value per feature (Rewrite, Summarize, Copilot)?** No, and this is
worth recording so it is not re-litigated. `Notepad.exe` contains **no** `DisableRewrite`,
`DisableSummarize` or `DisableCopilot` strings, and Learn's policy section lists exactly one policy.
One value covers all AI surfaces. The proposal is right.

**Attack 3, name mismatch trap.** The Group Policy / ADMX policy is named
`DisableAIFeaturesInNotepad`; the **registry value** is `DisableAIFeatures`. Writing the ADMX name
would be silently ignored. The proposal cites the ADMX name in its Sources line but proposes the
correct registry name; make sure the YAML uses `DisableAIFeatures`.

**Attack 4, why is it not in `C:\Windows\PolicyDefinitions`?** Because `WindowsNotepad.admx` ships
out of band in `WindowsNotepadAdminTemplates.cab`, not in the inbox ADMX set. Confirmed by search: no
ADMX on 26100 contains `DisableAIFeatures`. This is not evidence against the tweak; registry-only
application works regardless of whether the ADMX is installed.

**Applicability, with a real gate the proposal understated:** Windows 11 22H2 and later **and Notepad
app version 11.2503.16.0 or later**. Notepad updates through the Store independently of the OS build,
so a fully patched 26100 or 26200 machine can still carry a pre-11.2503 Notepad on which the value is
inert. Gate `products: [11]` as proposed, and say in the copy that an out-of-date Notepad will ignore
it. Inert on LTSC 2021 (classic Win32 Notepad, no AI surface).

**Duplicate?** No. `Notepad` appears nowhere in the corpus.

**Sources:** Microsoft Learn, Manage Notepad,
https://learn.microsoft.com/en-us/windows/client-management/manage-notepad (tier A); binary string
evidence in `Notepad.exe` 11.2604.5.0 (tier A, product artifact).

---

## Entry 7. `disable_paint_ai` (debloat)

**Verdict: CONFIRMED.** All three values exact. The proposal's own rejection of the two extra
Win11Debloat values is also correct and is upheld.

**Mechanism as proposed:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint`,
`DisableCocreator` / `DisableGenerativeFill` / `DisableImageCreator`, `REG_DWORD`, each Disabled = `1`,
each Stock Default = `absent`.

**Attack 1, existence.** All three in the shipped `WindowsCopilot.admx` on 26100, each
`class="Machine"`, each on `Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, each
`enabledValue` `1` / `disabledValue` `0`, each `supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"`.

Policy CSP - WindowsAI independently gives, for each of the three, `Registry Key Name`
`Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, the matching `Registry Value Name`,
`ADMX File Name` `WindowsCopilot.admx`, Format `int`, Default Value `0`, and allowed values
`0 (Default)` enabled / `1` disabled. Scope is Device only (User explicitly not supported), which
confirms HKLM.

Applicable OS on the CSP pages: "Windows 11, version 22H2 [10.0.22621.4870] and later" and
"Windows 11, version 24H2 [10.0.26100.3360] and later". Editions Pro / Enterprise / Education /
IoT Enterprise / IoT Enterprise LTSC, all supported. The proposal's applicability line is accurate.

**Attack 2, polarity.** Not inverted. Both ADMX `enabledValue` and the CSP allowed-values table give
`1` = disabled. ADML confirms: "If this policy is enabled, Cocreator functionality will not be
accessible in the Paint app."

**Attack 3, the two extra values.** Upheld as rejected. `DisableGenerativeErase` and
`DisableRemoveBackground` appear in **no** ADMX on 26100 and on **no** Microsoft Learn page (the
complete WindowsAI CSP policy list does not contain them). Single-source (Win11Debloat only). Do not
ship them. Note also that neither string appears in any binary under `C:\Program Files\WindowsApps`,
though Paint was not installed on this machine so that is not decisive either way.

**Attack 4, duplicate?** No. `Paint` appears nowhere in the corpus.

**Note on verification limits:** the three value names were **not** found in any binary under
`C:\Program Files\WindowsApps` on this machine, because the Paint app is not installed here. The
ADMX + CSP pair is tier A on both sides, so this does not weaken the verdict.

**Sources:** `WindowsCopilot.admx` / `en-US\WindowsCopilot.adml` on 26100 (tier A); Policy CSP -
WindowsAI, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai
(tier A).

---

## Entry 8. `disable_settings_account_ads` (debloat)

**Verdict: CORRECTED.** The mechanism is real and exact. The applicability is wrong in two places,
and the "High" ranking is not defensible for the product's likely audience.

**Mechanism as proposed:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`,
`DisableConsumerAccountStateContent`, `REG_DWORD`, Disabled = `1`, Stock Default = `absent`.
**Key, value name, type and polarity all verified correct.**

Shipped `CloudContent.admx` on 26100, `class="Machine"`, key
`Software\Policies\Microsoft\Windows\CloudContent`, `enabledValue` `1` / `disabledValue` `0`. The
value name also appears in `System32\windowsudk.shellcommon.dll` and
`System32\wbem\DMWmiBridgeProv.dll`, so shipped code reads it.

**Correction 1, applicability floor.** The proposal says "Windows 10 2004
(`SUPPORTED_Windows_10_0_RS7`) and later". Two problems. The shipped 26100 `Windows.adml` renders
`SUPPORTED_Windows_10_0_RS7` as **"At least Windows Server 2016, Windows 10 Version 1909"**, not
2004. And Microsoft's Policy CSP page states the applicable OS as **"Windows 11, version 21H2
[10.0.22000] and later"**. The two Microsoft sources disagree; the CSP is the narrower and more
current claim. **Use Windows 11 21H2 and later.** The practical consequence is that the LTSC 2021
(Windows 10 19044) case the proposal leans on is not actually supported by the CSP.

**Correction 2, the CSP allowed-values table is misleading and must not be copied.** The Policy CSP
page for this policy reads:

> Allowed values: `0 (Default)` Disabled. `1` Enabled.

Read naively that says `1` **enables** the content, which would invert the tweak. It does not. That
table describes the **policy** state, not the feature state, and it contradicts the same page's own
description block ("If you enable this policy, Windows experiences [...] will instead present the
default fallback content"). The ADMX `enabledValue` of `1` is decisive. `1` = account-state content
**off**. The proposal has this right; the risk is a future editor "fixing" it against the CSP table.

**Correction 3, the edition gate is confirmed and it is fatal to the "High" ranking.** Policy CSP
editions for `DisableConsumerAccountStateContent`:

> Editions: **Pro NOT supported**; Enterprise, Education, IoT Enterprise / IoT Enterprise LTSC
> supported. Scope: Device only.

The proposal flagged this. Verification confirms it, and confirms it is the same gate the corpus
already lives with: `AllowWindowsConsumerFeatures` (the CSP side of the already-shipped
`disable_consumer_features`, `privacy.yaml:847`) carries the identical "Pro not supported" marking.
So this is not a novel problem, but it does mean the tweak is a **no-op on Pro and Home**, which is
the majority of consumer Windows 11 installs. Ranking it High alongside items that work everywhere
overstates it. The tweak copy must say plainly that Pro and Home will not honour it, as the proposal
itself recommended.

**Duplicate?** No. The corpus's three CloudContent tweaks write
`DisableTailoredExperiencesWithDiagnosticData`, `DisableSoftLanding` and
`DisableWindowsConsumerFeatures`. Different values. The distinction the proposer drew (consumer
features versus account-state cards) is supported by the two policies having separate ADML
descriptions and separate client components.

**Corrected applicability line:** Windows 11 21H2 (22000) and later, machine scope, Enterprise /
Education / IoT Enterprise / IoT Enterprise LTSC only. Silently ignored on Pro and Home.

**Sources:** `CloudContent.admx` / `en-US\CloudContent.adml` on 26100 (tier A); Policy CSP -
Experience > DisableConsumerAccountStateContent,
https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A);
`en-US\Windows.adml` `supportedOn` string table on 26100 (tier A); binary string evidence in
`windowsudk.shellcommon.dll` and `DMWmiBridgeProv.dll`.

---

## Entry 9. `disable_phone_companion_start` (debloat)

**Verdict: CONFIRMED**, on stronger evidence than the proposal offered. Two details need care.

**Mechanism as proposed:**
`HKCU\Software\Microsoft\Windows\CurrentVersion\Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe`,
`IsEnabled`, `REG_DWORD`, Disabled = `0`, Stock Default = `absent`.

**Attack 1, does the key exist as a product concept?** Yes. The proposal offered only community
sources. First-party binary evidence is available and is better. The key path literal
`Software\Microsoft\Windows\CurrentVersion\Start\Companions` is present in two shipped Start binaries
on 26100:

- `SystemApps\Microsoft.Windows.StartMenuExperienceHost_cw5n1h2txyewy\StartDocked.dll`
- `SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\StartMenu.dll`

**Attack 2, is `IsEnabled` the right value name under that key?** Yes. In `StartMenu.dll` the key
path literal is followed immediately by the per-companion value-name run, which contains `IsEnabled`:

```
Software\Microsoft\Windows\CurrentVersion\Start\Companions | com.microsoft.startmenucompanion |
CompanionSettingName | CompanionSettingExtendedName | DetailedSettingsLaunchUri |
DetailedSettingsLaunchName | RefreshCardLaunchUri | OverridesPackage | DefaultState | Icon |
IsEnabled | IsAvailable | DetailedSettingsSettingName | CompanionVersion |
AdaptiveCardsSchema_MAJOR | AdaptiveCardsSchema_MINOR | ... | Enabled | Disabled | Unavailable |
%ls\%ls
```

The trailing `%ls\%ls` format string is the `<CompanionsKey>\<PFN>` subkey composition, and
`Enabled | Disabled | Unavailable` is the tri-state this code logs. `IsEnabled` is a real per-companion
value under a per-PFN subkey. The proposal's shape is correct.

**Attack 3, is the PFN right, and is the hive right?** Not provable from the binary, but
independently corroborated three times over. `StartDocked.dll` does **not** contain the literal
`Microsoft.YourPhone_8wekyb3d8bbwe`, which is expected because the subkey is created from the
companion app's own registration rather than hardcoded in the shell, and the hive cannot be read off
a string. Three genuinely independent sources agree on `HKEY_CURRENT_USER` plus that exact PFN:

1. Win11Debloat `Regfiles/Disable_Phone_Link_In_Start.reg`, read as the raw `.reg` body:
   `[HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe]`
   with `"IsEnabled"=dword:00000000`.
2. ElevenForum tutorial 26919 (via the Wayback Machine), identical `.reg` body, plus a build
   attribution: the feature starts at build **26100.3915 (24H2)** and 22631.5262 (23H2).
3. Microsoft Q&A 5510106, whose answer walks the user to the same key and value and where a
   participant reports the key on 26100.4770.

**Attack 4, is `0` the right applied value and `absent` the right revert?** Yes, with a caveat the
proposal missed. The binary carries a `DefaultState` value name alongside `IsEnabled`, which implies
the shell falls back to a package-declared default when `IsEnabled` is absent, consistent with Stock
Default = `absent`. Note that Win11Debloat's undo writes `1` rather than deleting the value, so the
corpus's `absent` revert and Win11Debloat's revert are **not** the same end state. `absent` is the
more faithful one. Also note the revert restores "whatever the package declares", not necessarily
"shown", so the copy should not promise the panel comes back if Phone Link has since been removed.

**Attack 5, is this a hidden hack or a Settings-backed toggle?** It is Settings-backed, which the
proposal did not mention and which matters. Microsoft's support article "Mobile device in Start menu"
documents **Personalization > Start > Show mobile device in Start**. This value is the backing store
for that switch. Practical consequence: like the `Start_AccountNotifications` case in entry 3, a user
flipping the Settings toggle will silently desynchronise the tweak from its snapshot. Say so in the
copy, or accept the drift knowingly.

**Attack 6, duplicate?** No. `Companions` and `YourPhone` appear nowhere in the corpus as registry
effects. `remove_phone_link` removes the appx, which is a different and (per `_rescope-24h2.md`) less
reliable action. The proposer's reasoning stands.

**Correction: `requires_reboot` should be false.** The proposal says "Requires sign-out or restart to
take effect". Neither the ElevenForum tutorial nor the Win11Debloat file specifies a sign-out step,
and ElevenForum reliably calls one out when it is needed. The Microsoft Q&A "restart your PC" line is
troubleshooting advice for a panel that failed to appear, not a stated requirement.

**Applicability:** Windows 11 24H2 build **26100.3915** and later (and 23H2 22631.5262), user scope.
Not present on LTSC 2021. Also inert wherever Phone Link is not installed, which includes IoT
Enterprise LTSC images, so the tweak must treat a missing key as a clean no-op rather than an error.

**Sources:** binary string evidence in `StartDocked.dll` and `StartMenu.dll` on 26100.4061 (tier A,
product artifact); Microsoft support article "Mobile device in Start menu",
https://support.microsoft.com/en-us/windows/mobile-device-in-start-menu-21676d6a-3bc3-439a-aaa3-7463b91cda79
(tier A, for the Settings surface); Win11Debloat `Regfiles/Disable_Phone_Link_In_Start.reg` (tier C,
raw `.reg` body); ElevenForum tutorial 26919 via the Wayback Machine (tier C); Microsoft Q&A 5510106
(tier C). GeekRewind was examined and **not** counted: it reads as a rewrite of the ElevenForum
tutorial. TheWindowsClub and NinjaOne are JavaScript-gated and their served HTML contains neither
`Companions` nor `IsEnabled`, so the proposal's NinjaOne citation corroborates nothing.

---

## Entry 10. `disable_drag_tray` (interface)

**Verdict: CORRECTED.** The key, value name, type and polarity are all right. Two things are wrong:
one of the proposal's four cited sources does not support the claim at all, and the feature has since
been **renamed and given a Settings toggle**, which changes what the tweak should be called and how
it should be described.

**Mechanism as proposed:** `HKCU\Software\Microsoft\Windows\CurrentVersion\CDP`, `DragTrayEnabled`,
`REG_DWORD`, Disabled = `0`, Stock Default = `absent`. **All verified correct.**

**Attack 1, is the key really `CurrentVersion\CDP`?** This was the main doubt, because `CDP` is the
Connected Devices Platform key whose established per-user tenants are authorisation policies
(`RomeSdkChannelUserAuthzPolicy`, `NearShareChannelUserAuthzPolicy`, `CdpUserSettingsVersion`,
`EnableRemoteLaunchToast`), and a UI affordance toggle is not an obvious neighbour. It is nonetheless
correct, and it is specifically **not** `Explorer\Advanced` and not a Shell key. Three independent
sources carry the identical literal:

```
[HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\CDP]
"DragTrayEnabled"=dword:00000000
```

1. Win11Debloat `Regfiles/Disable_Share_Drag_Tray.reg` (raw `.reg` body; undo writes `1`).
2. ElevenForum tutorial 40485 via the Wayback Machine, carrying both `dword:00000001` (on) and
   `dword:00000000` (off) bodies.
3. MajorGeeks, firsthand and useful precisely because the author narrates creating the value:
   "Navigate to `Computer\HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\CDP` ... Name it
   `DragTrayEnabled` ... type 0 ... To undo these changes and enable the drag tray, just type 1", and
   "I had to create it from scratch while testing this". That last sentence independently confirms
   **Stock Default = `absent`**, which is exactly the detail the corpus's revert depends on.

AskVG is a fourth, partially independent source with the same key, value and polarity.

**Attack 2, the local binary sweep found nothing. Does that refute it?** No. The literal
`DragTrayEnabled` (and the substrings `DragTray` and `DropTray`) was searched in both UTF-16LE and
ASCII across 8,194 binaries under `System32` and `SystemApps`, 5,207 under
`C:\Program Files\WindowsApps`, and 1,359 shell / CDP / share / Explorer related `WinSxS` component
directories. **Zero hits.** But this machine is **26100.4061** and the feature ships at
**26100.4202**, so the code is genuinely not on this image. The negative result is what the sources
predict, and it doubles as independent corroboration of the default: on 26100.4061 the `CDP` key
exists with its five established values and **no** `DragTrayEnabled`.

**Correction 1: one of the proposal's cited sources is bogus and must be removed.** The proposal
cites "Pureinfotech, How to disable Drag Tray (Drop Tray) sharing UI on Windows 11". That page never
mentions `DragTrayEnabled` or the `CDP` key. It documents a completely different mechanism, a
feature-flag override at
`HKLM\SYSTEM\ControlSet001\Control\FeatureManagement\Overrides\14\3895955085` with `EnabledState` and
`EnabledStateOptions` plus a reboot, or the Settings app. Counting it would have been a false
corroboration, and it would be a dangerous thing to ship: feature-management override IDs are
build-specific and are not a stable authoring surface. **Strike it from the Sources line.** The claim
still clears the three-independent-source bar without it.

**Correction 2: the feature was renamed, and it now has a Settings toggle.** Per ElevenForum's
changelog, at builds **26100.8328 (24H2)**, 26200.8328 (25H2) and 28000.2179 (26H1), "Drag Tray has
been now renamed to Drop Tray", and its Settings home moved from System > Nearby sharing to
**System > Multitasking**. A Settings toggle has existed since 26100.7309 / 26200.7309. Two
consequences for the corpus:

- The tweak should be named for **Drop Tray**, with "Drag Tray" as an alias in the description, or
  users on current servicing will not recognise it.
- This is no longer a hidden hack; it is the backing store for a supported Settings switch. That is
  good for reliability and bad for drift: a user toggling it in Settings desynchronises the snapshot,
  the same caution as entries 3 and 9.

**Correction 3: build attribution.** The proposal's 26100.4202 is right for the 24H2 GA channel
(ElevenForum). AskVG's 26200.5518 (KB5055625, April 2025) is the earlier Dev-channel debut. Both are
true and they are not in conflict.

**Attack 3, polarity and reboot.** `0` = off, `1` = on, absent = on. No restart required; changes are
described as instant, and neither the ElevenForum tutorial nor AskVG specifies a sign-out.

**Duplicate?** No. `DragTray` and `CDP` appear nowhere in the corpus.

**Applicability:** Windows 11 24H2 at 26100.4202 or later, and 25H2. User scope. Not present on LTSC
2021. The corpus has no revision-level gate, so `build >= 26100` plus copy that says early 26100
servicing levels have nothing to disable is the best available, exactly as the proposal notes.

**Sources:** Win11Debloat `Regfiles/Disable_Share_Drag_Tray.reg` (tier C, raw `.reg` body);
ElevenForum tutorial 40485 via the Wayback Machine (tier C, also the source for the rename and the
Settings relocation); MajorGeeks "How To Disable Drag Tray" (tier C, firsthand, and the source for
value-absent being the default); AskVG (tier C, Dev-channel build attribution); negative binary
search across 14,760 shipped binaries on 26100.4061 (product artifact, consistent with the sources).
**Rejected as a source: Pureinfotech**, which describes an unrelated feature-management override.

---

## Entry 11. `hide_unsupported_hardware_notice` (interface)

**Verdict: CONFIRMED.** Ship as proposed.

**Mechanism as proposed:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`,
`HideUnsupportedHardwareNotifications`, `REG_DWORD`, Hidden = `1`, Stock Default = `absent`.

**Attack 1, existence.** Shipped `ControlPanel.admx` on 26100, `class="Machine"`, key
`Software\Microsoft\Windows\CurrentVersion\Policies\System`, valueName exactly as proposed,
`supportedOn ref="windows:SUPPORTED_Windows_11_0_NOSERVER"` ("At least Windows 11").

**Attack 2, does anything read it, and does it cover both surfaces the proposal claims?** Yes, and
the binary evidence maps one-to-one onto the two claimed surfaces:

- `C:\Windows\System32\shell32.dll` (the desktop watermark)
- `C:\Windows\System32\AboutSettingsHandlers.dll` (Settings > System > About, the banner)

The proposal claimed "desktop watermark plus the Settings > System > About banner". The two binaries
that contain the value name are precisely the watermark host and the About page handler. Claim
verified.

**Attack 3, ADML semantics.** Unambiguous:

> This policy controls messages which are shown when Windows is running on a device that does not
> meet the minimum system requirements for this OS version. If you enable this policy setting, these
> messages will never appear on desktop or in the Settings app.

**Attack 4, is the benefit real or theatre?** Real but purely cosmetic, and the proposal already says
so. The important thing is the negative claim the proposal makes: it does **not** change update
eligibility or servicing behaviour. Nothing found contradicts that, and the value is read only by
presentation-layer binaries (shell32 and a Settings page handler), which supports it. Keep that
sentence in the copy so nobody reads the tweak as "makes my PC supported".

**Attack 5, the revert.** As noted in the cross-cutting section, this policy declares **no**
`disabledValue`. There is no defined `0` state. Revert must be `absent`. The proposal has this right.

**Attack 6, shared key.** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` is a
high-traffic key (UAC values, legal notice, and more). Under per-value snapshotting this is not a
collision. No corpus tweak writes this value name.

**Applicability:** all Windows 11 client editions, so 26100 and 26200. Machine scope. Not applicable
to LTSC 2021 (Windows 10).

**Sources:** `ControlPanel.admx` / `en-US\ControlPanel.adml` on 26100 (tier A); binary string
evidence in `shell32.dll` and `AboutSettingsHandlers.dll` (tier A, product artifact).

---

## Entry 12. `disable_edge_ai_features` (debloat)

**Verdict: CORRECTED.** All five values exist and none is deprecated, but **three of the five are
documented as applying only to Microsoft Entra ID profiles and do nothing on a Microsoft account**.
Shipping them as-is on consumer machines would produce a successful write with no observable effect.

**Mechanism as proposed:** `HKLM\SOFTWARE\Policies\Microsoft\Edge`, five `REG_DWORD` values, each
Off = `0`, each Stock Default = `absent`. The base key is correct for all five (each Edge policy page
lists `Path (Mandatory): SOFTWARE\Policies\Microsoft\Edge`), and all five are `REG_DWORD`. None
appears in Edge's deprecated or obsolete policy tables.

**The finding, per value:**

| Value | Exists | Type | `0` means | Real gate |
|---|---|---|---|---|
| `EdgeHistoryAISearchEnabled` | Yes | REG_DWORD | AI history search off, exact-match only | Edge 138+ |
| `NewTabPageBingChatEnabled` | Yes | REG_DWORD | all Bing Chat entry points removed from NTP | Edge 117+ |
| `CopilotPageContext` | Yes | REG_DWORD | Copilot cannot read page content | Edge 124+, **Entra profiles only** |
| `EdgeEntraCopilotPageContext` | Yes | REG_DWORD | Copilot cannot read page content or history | Edge 130+, **Entra profiles only** |
| `ComposeInlineEnabled` | Yes | REG_DWORD | Rewrite unavailable | Edge 115+, **Entra profiles only** |

The Entra restriction is stated by Microsoft in the policy text itself, for example on
`CopilotPageContext`:

> This policy applies only to Microsoft Entra ID profiles in Microsoft Edge. It doesn't apply to
> Microsoft account (MSA) profiles.

and on `ComposeInlineEnabled`:

> This policy applies only to Microsoft Entra accounts and doesn't apply to Microsoft accounts.

and `EdgeEntraCopilotPageContext` lists "Applies to a profile that is signed in with a Microsoft
account: **No**".

**Why this matters more than a footnote.** `CLAUDE.md` states the did-it-work contract: a failed
privileged or effect operation must surface as `Err`, never a benign-looking value. These three do
not fail, they succeed and do nothing. That is the harder version of the same problem: the user
applies a tweak called "turn off Edge Copilot page context", the app reports Applied, and Copilot in
Edge carries on reading pages, because the profile is an MSA. Options, in order of preference:

1. **Ship only the two that work on consumer profiles** (`EdgeHistoryAISearchEnabled`,
   `NewTabPageBingChatEnabled`). Cleanest, and preserves the tweak's honesty.
2. Ship all five but make the Entra limitation explicit in the tweak copy and in Drawbacks.
3. Do not use `skip_validation: true` to paper over it. That hides the problem instead of solving it.

**A second correction: reject the proposal's suggested third option.** The proposal says
"`EdgeCopilotEnabled` = 0 is also documented and removes Copilot in Edge entirely; that would make a
good third option value on this tweak." It would not. Microsoft's page for `EdgeCopilotEnabled` lists
**Windows: Not supported**, macOS: Not supported, Android and iOS 123+ only, and the page carries no
Windows registry settings section at all. Writing it to `HKLM\SOFTWARE\Policies\Microsoft\Edge` on
Windows does nothing. **Do not ship it.**

Also noted while checking: `CopilotCDPPageContext` is marked "(obsolete)" on the Edge policy index,
superseded by `EdgeEntraCopilotPageContext`. Do not add it.

**Attack, duplicate?** No. The corpus's five existing Edge tweaks write `HideFirstRunExperience`,
`StartupBoostEnabled`, `BackgroundModeEnabled`, `HubsSidebarEnabled`, `EdgeCollectionsEnabled`,
`MetricsReportingEnabled`, `SendSiteInfoToImproveServices`, `PersonalizationReportingEnabled`,
`Edge3PSerpTelemetryEnabled` and `SmartScreenEnabled`. None of the five proposed values collides. The
proposer's reasoning about `HubsSidebarEnabled` removing only the panel is correct.

**Applicability:** wherever Edge is installed, machine scope, with the per-value Edge version floors
in the table above. The proposal did not carry Edge version gates; they should be in the copy, since
a machine on Edge 120 will ignore `EdgeHistoryAISearchEnabled`.

**Known side effect the proposal correctly flagged:** setting any Edge policy makes Edge show
"managed by your organization" on its settings page. The corpus's existing Edge tweaks already do
this, so it is not new, but keep it in the copy.

**Sources:** Microsoft Edge Browser Policy Documentation pages for `copilotpagecontext`,
`edgeentracopilotpagecontext`, `edgehistoryaisearchenabled`, `composeinlineenabled`,
`newtabpagebingchatenabled` and `edgecopilotenabled` under
https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/ (tier A).

---

## UNKNOWNS remaining after this pass

1. **The exact 26100 servicing revision at which `WSAIFabricSvc` first appears.** Evidence here bounds
   it at or below 26100.3624 (the host DLL's file version), and Win11Debloat issue 265 from June 2025
   predates 25H2 GA. The lower bound is unknown, so the tweak needs a **presence check** rather than a
   revision gate.
2. **`WSAIFabricSvc` on 25H2 specifically.** The Automatic default is well corroborated on 24H2 and
   strongly indicated on 25H2, but no clean 26200 image was inspected. Low risk; worth one probe
   before shipping.
3. **Entry 8's applicability floor.** The shipped ADML (`At least Windows Server 2016, Windows 10
   Version 1909`) and the Policy CSP (`Windows 11, version 21H2`) disagree. Both cover the primary
   target, so this only matters if LTSC 2021 support is claimed for it. Use the CSP's narrower claim.
4. **Entry 1 on Windows 11 Home.** The Policy CSP edition list omits Home, as it does for most
   policies. Whether `StartMenu.dll` honours the value on Home was not established; the binary
   evidence is edition-independent, which is suggestive but not proof.
5. **Entry 12's Edge version floors in practice.** Microsoft documents per-policy minimum Edge
   versions (115, 117, 124, 130, 138). The corpus has no Edge version gate, so a machine on an older
   Edge will accept the write and ignore the policy. Decide whether that is acceptable or whether the
   tweak needs an Edge version probe.
6. **Entry 10's rename timing.** "Drag Tray" to "Drop Tray" at 26100.8328 rests on a single
   ElevenForum changelog line. The registry value name is unaffected either way, so this is a copy
   question, not a correctness one.

## Items from the original UNKNOWNS list that this pass resolved

- **UNKNOWN 6, `HideRecommendedSection` on Pro.** Resolved. The Policy CSP lists Pro as supported and
  the value is read by `StartMenu.dll` on a non-SE 26100 image. The `SUPPORTED_Windows_11_0_SE`
  annotation is a gpedit display string, not an enforcement gate. **The PolicyManager fallback effect
  is not required and should not be shipped.**
- **UNKNOWN 5, `WSAIFabricSvc` stock start type.** Resolved to **Automatic (`Start` = 2), plain, not
  delayed, not trigger-started**, on the strength of Win11Debloat's own undo artifact plus two
  independent 25H2 reports. The larger finding is that the premise of the question was wrong: the
  service is not 25H2-only, so it cannot be gated `build >= 26200`.
- **Not previously listed, `DragTrayEnabled`.** Resolved to correct as proposed, with the caveat that
  one of the proposal's four cited sources (Pureinfotech) documents an entirely different mechanism
  and must be struck.

## What a second pass should do differently

Two of the four corrections came from reading the **actual artifact** rather than the description of
it: the `.reg` bodies, the ADMX `<policy>` element, the MUI string table, the value name inside the
shipped binary. The proposal's errors clustered where it relied on prose about an artifact
(a forum's name for a service, a policy page's summary, a blog's title) rather than the artifact
itself. The binary-string sweep in particular was cheap and decisive: it converted five entries from
"Microsoft documents it" to "the OS demonstrably reads it", and it is the technique that turned up
entry 5's wrong applicability gate.
