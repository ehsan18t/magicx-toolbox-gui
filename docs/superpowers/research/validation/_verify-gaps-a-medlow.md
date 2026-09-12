# Adversarial verification: gap proposals 13 to 33 (Medium and Low)

**Subject.** `_gaps-privacy-debloat-interface.md`, proposals **13 to 25** (Medium, 13 items) and
**26 to 33** (Low, 8 items). Proposals 1 to 12 (High) are another verifier's scope and are not
covered here.

**Stance.** Adversarial. Default REJECT under uncertainty. Every claim below was re-derived from
primary evidence rather than accepted from the proposal.

**Target platform.** Windows 11 24H2 (26100) and 25H2 (26200) primary; Windows 10 IoT Enterprise
LTSC 2021 (19044) secondary.

## Evidence method

The host machine is heavily modified by its owner, so its registry, service state and scheduled
tasks were treated as inadmissible. Only these classes of evidence were used:

1. **Shipped ADMX and ADML on 26100** under `C:\Windows\PolicyDefinitions`. Every ADMX cited below
   was parsed directly: policy name, `class`, `key`, `valueName`, `supportedOn`, `enabledValue`,
   `disabledValue` and the `elements` enum, with the matching `en-US` ADML display and help strings.
2. **String presence in shipped binaries.** A byte scan for UTF-16LE and ASCII occurrences of each
   value name across `C:\Windows\System32` (top level plus subdirectories), `C:\Windows\SystemApps`,
   `C:\Windows\ImmersiveControlPanel`, `C:\Windows\SystemResources` and `C:\Program Files\WindowsApps`.
   Presence in a **consumer** binary (the component that renders the surface) is treated as strong
   evidence the value is read; presence only in `SHCore.dll`, `assignedaccessmanagersvc.dll` or
   `DMWmiBridgeProv.dll` is treated as weak, because those hold policy-plumbing tables rather than
   feature code.
3. **Microsoft Learn Policy CSP pages**, fetched live.
4. **Upstream project sources fetched raw**: Win11Debloat `Regfiles/*.reg` (decoded from UTF-16, not
   read from the README), privacy.sexy `windows.yaml`, Sophia Script `Sophia.psm1` for Windows 11,
   Chris Titus WinUtil `config/tweaks.json`.
5. **The corpus itself**: every `key` and `name` pair in `src-tauri/tweaks/*.yaml` was scanned for
   each proposed value name to test the duplicate claims.

## Headline result

| Verdict | Count | Proposals |
|---|---|---|
| CONFIRMED | 13 | 13, 15, 18, 19, 20, 22, 23, 26, 27, 28, 29, 30, 33 |
| CORRECTED | 4 | 16, 17, 24, 32 |
| REJECTED | 4 | 14, 21, 25, 31 |
| UNRESOLVED | 0 | none stand unresolved as a verdict; 25 is rejected *because* its polarity is unresolvable without a probe |

Total 21. Exact per-proposal reasons are in the summary table at the end.

**Two defects found that would have shipped a broken tweak:**

- **Proposal 16** writes `DisableSearchHistory` to **HKLM**. The shipped 26100 `Search.admx`
  declares that policy `class="User"` (so HKCU) and `supportedOn="Win8Only"`, and the value name does
  not appear in any Windows 11 search binary. As proposed it writes to a hive the policy does not use,
  for a policy the shipped ADMX scopes to Windows 8.
- **Proposal 24** writes `MultiTaskingAltTabFilter` = `3` meaning "windows only". That is correct for
  the `Explorer\Advanced` preference but is **exactly wrong** for the Group Policy value of the same
  name, where `3` means "3 most recent tabs" and `4` means "open windows only". Same value name, two
  keys, two different enum bases. Getting the key and the value out of step silently produces the
  opposite of the user's intent.

**Three source-attribution claims in the proposal are false** and were relied on to reach the
"three independent sources" bar. See "Source-attribution failures" below.

---

## Per-proposal findings

### 13. `disable_windows_spotlight_all` (privacy, Medium) - CONFIRMED

- **Key** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, **value**
  `DisableWindowsSpotlightFeatures`, **REG_DWORD**, Disabled = `1`, stock default value-absent.
- Shipped `CloudContent.admx` on 26100: `class="User"`,
  `key="Software\Policies\Microsoft\Windows\CloudContent"`,
  `valueName="DisableWindowsSpotlightFeatures"`, `enabledValue 1` / `disabledValue 0`,
  `supportedOn="windows:SUPPORTED_Windows_10_0_NOSERVER"`. Everything in the proposal matches.
- `en-US\CloudContent.adml`, "Turn off all Windows spotlight features": "If you enable this policy
  setting, Windows spotlight on lock screen, Windows tips, Microsoft consumer features and other
  related features will be turned off. You should enable this policy setting if your goal is to
  minimize network traffic from target devices."
- Binary evidence on 26100: the value name is present in `ContentDeliveryManager.Background.dll`,
  `StartTileData.dll`, `Taskbar.dll`, `SettingsHandlers_ContentDeliveryManager.dll`,
  `CustomShellHost.exe` and `ShellAppRuntime.exe`. These are the feature consumers, not policy
  plumbing, so the value is genuinely read on the target build.
- Corroboration: privacy.sexy `windows.yaml` writes the same value name (confirmed by direct fetch).
- **Duplicate check: passes.** No `CloudContent` value in the corpus other than
  `DisableTailoredExperiencesWithDiagnosticData`, `DisableSoftLanding` and
  `DisableWindowsConsumerFeatures` (privacy.yaml 508, 817, 847), all HKLM and all different names.
- **Caveat the tweak copy must carry.** This is a master switch whose blast radius overlaps four
  shipped tweaks (`RotatingLockScreenOverlayEnabled`, `SubscribedContent-338387Enabled`,
  `SoftLandingEnabled` / `SubscribedContent-338389Enabled`, `DisableWindowsConsumerFeatures`).
  It does not clobber their values, so revert stays correct, but a user who applies this and then
  looks at those tweaks will see them still reading "not applied" while their surfaces are gone.
  Say so, or the corpus looks inconsistent.

### 14. `disable_third_party_suggestions` (privacy, Medium) - REJECTED

The facts are right; the tweak is not worth a slot.

- Facts confirmed: `CloudContent.admx` on 26100 gives `class="User"`, key
  `Software\Policies\Microsoft\Windows\CloudContent`, `valueName="DisableThirdPartySuggestions"`,
  `enabledValue 1`. ADML: "If you enable this policy, Windows spotlight features like lock screen
  spotlight, suggested apps in Start menu or Windows tips will no longer suggest apps and content
  from third-party software publishers." Binary presence in
  `ContentDeliveryManager.Background.dll`. The proposal's inverted-name warning is correct.
- **Rejection ground 1, documented inertness.** Policy CSP Experience >
  `AllowThirdPartySuggestionsInWindowsSpotlight` declares
  `Dependency Type: DependsOn, Dependency URI: User/.../Experience/AllowWindowsSpotlight,
  Dependency Allowed Value: [1]`. The setting is only meaningful while Spotlight is on. The corpus
  already ships lock-screen Spotlight suppression, and proposal 13 turns Spotlight off wholesale.
  In the corpus's own recommended configuration this value does nothing.
- **Rejection ground 2, strict subset.** Everything it suppresses is a subset of proposal 13.
- **Rejection ground 3, sourcing.** The proposal cites privacy.sexy. A raw fetch of
  privacy.sexy `windows.yaml` contains no occurrence of `DisableThirdPartySuggestions`. The claim is
  false. Only the ADMX supports this one, and the ADMX is not in dispute; the value is.
- Ship proposal 13 instead.

### 15. `disable_spotlight_desktop` (interface, Medium) - CONFIRMED

- **Key** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, **value**
  `DisableSpotlightCollectionOnDesktop`, **REG_DWORD**, Disabled = `1`, stock default value-absent.
- `CloudContent.admx` on 26100: `class="User"`, same CloudContent key, `enabledValue 1`,
  `supportedOn="windows:SUPPORTED_Windows_10_0_NOSERVER"`. ADML text is exactly as the proposal
  quotes it, including Microsoft's own typo "subsequentyly".
- Binary presence in `SettingsHandlers_ContentDeliveryManager.dll`, `StartTileData.dll`,
  `SettingsHandlers_nt.dll`, `CustomShellHost.exe`.
- Win11Debloat `Regfiles/Disable_Desktop_Spotlight.reg`, fetched and decoded, is byte-for-byte the
  proposed effect: `[HKEY_CURRENT_USER\Software\Policies\Microsoft\Windows\CloudContent]`
  `"DisableSpotlightCollectionOnDesktop"=dword:00000001`.
- **Source correction.** The proposal cites privacy.sexy. privacy.sexy does not contain this value
  name. Actual support is ADMX plus Win11Debloat, which is sufficient because ADMX is tier A.
- **Overlap note.** Also covered by proposal 13's "and other related features". Ship at most one of
  13 and 15 as a *recommended* item; the other is fine as an available control.

### 16. `disable_search_history` (privacy, Medium) - CORRECTED

The `IsDeviceSearchHistoryEnabled` half is right. The `DisableSearchHistory` half is wrong in three
independent ways and must be dropped. The stated stock default is also wrong.

**Corrected specification:**

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`
- **Value name:** `IsDeviceSearchHistoryEnabled`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Enabled (Stock Default) = **absent** (delete the value on revert)
- **Drop entirely:** the `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` `DisableSearchHistory`
  effect.

**Why the policy half is dropped:**

1. **Wrong hive.** The shipped 26100 `Search.admx` declares
   `<policy name="DisableSearchHistory" class="User" key="SOFTWARE\Policies\Microsoft\Windows\Explorer"
   valueName="DisableSearchHistory">`. `class="User"` means HKCU. The proposal writes HKLM. So does
   privacy.sexy (`keyPath: HKLM\Software\Policies\Microsoft\Windows\Explorer`, line 7496), which is
   where the proposal inherited the error; privacy.sexy's own cited reference for it is an
   admx.help mirror of the **Windows 10 2016** ADMX, not a current one.
2. **Declared Windows 8 only.** The same policy element carries `<supportedOn ref="Win8Only" />`.
   It sits in the legacy block of `Search.admx` alongside `AlwaysUseAutoLangDetection` and
   `ConnectedSearchUseWeb`, not in the `windows:SUPPORTED_Windows_10_0` block that holds
   `AllowCloudSearch` and `AllowSearchHighlights`.
3. **No Windows 11 consumer.** The literal `DisableSearchHistory` appears in exactly one shipped
   binary, `SHCore.dll` (the shell policy table). It does not appear in `SearchUx.UI.dll`,
   `windowsudk.shellcommon.dll` or any other search component. By contrast
   `IsDeviceSearchHistoryEnabled` appears in `SearchUx.UI.dll` (the 26100 taskbar search UI, in the
   view-model block next to `IsCloudSearchEnabledForMSA` and `IsCloudSearchEnabledForAAD`),
   `windowsudk.shellcommon.dll` and `AppXDeploymentExtensions.desktop.dll`.

**Why the stock default is corrected (this is the revert-correctness issue, ADR-0002 territory):**
the proposal asserts `IsDeviceSearchHistoryEnabled` is "normally present and `1`" and that the
revert option must therefore write `1`. privacy.sexy annotates this exact value with
`deleteOnRevert: 'true' # Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro
(>= 23H2)` (line 7508). That annotation comes from privacy.sexy's default-state scan of real Pro
images, and it says value-absent. Win11Debloat's `Disable_Search_History.reg` writes only
`"IsDeviceSearchHistoryEnabled"=dword:00000000` with no revert file, which is consistent. Revert
must **delete**, not write `1`. UNKNOWN 3 in the proposal resolves against the proposal.

**Also worth flagging for the proposal's rejected list:** it asserts "the shipped 26100 `Search.admx`
contains zero policy elements". That is false. `Search.admx` on 26100 is 66 KB and defines **50**
policies, including `DoNotUseWebResults` (`ConnectedSearchUseWeb`), `DisableWebSearch`,
`AllowCloudSearch`, `AllowSearchHighlights` (`EnableDynamicContentInWSB`) and
`ConfigureSearchOnTaskbarMode` (`SearchOnTaskbarMode`, `SUPPORTED_Windows_11_0_NOSERVER`). The
rejection of the legacy web-search family still stands on the `WinBlueOnly` / `RedistOnly`
`supportedOn` values, but the stated reason is wrong and should be fixed so it is not re-litigated.

### 17. `disable_cloud_content_search` (privacy, Medium) - CORRECTED

Keys, value names and types are all correct. The stock default for the two HKCU values is wrong.

**Corrected specification:**

- `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\SearchSettings` `IsMSACloudSearchEnabled`
  `REG_DWORD`, Disabled = `0`, Stock Default = **absent**
- `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\SearchSettings` `IsAADCloudSearchEnabled`
  `REG_DWORD`, Disabled = `0`, Stock Default = **absent**
- `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` `AllowCloudSearch` `REG_DWORD`,
  Disabled = `0`, Stock Default = absent

Evidence:

- Shipped 26100 `Search.admx`: `<policy name="AllowCloudSearch" class="Machine"
  key="SOFTWARE\Policies\Microsoft\Windows\Windows Search" valueName="AllowCloudSearch">` with
  `supportedOn="windows:SUPPORTED_Windows_10_0"` and an enum of `0` = Disable Cloud Search,
  `1` = Enable Cloud Search, `2` = User Selected. ADML: "Allow search and Cortana to search cloud
  sources like OneDrive and SharePoint." The proposal's `0` is correct. Note the enum has a third
  value `2`; if the corpus wants a faithful control it should offer all three, otherwise document
  that it only writes `0`.
- `AllowCloudSearch` string present in `Windows.Storage.Search.dll`, `windowsudk.shellcommon.dll`
  and `Windows.FileExplorer.Common.dll` on 26100, so it is live and not Cortana-dead despite the
  Cortana-era ADML wording.
- `IsMSACloudSearchEnabled` and `IsAADCloudSearchEnabled` present in `windowsudk.shellcommon.dll`
  and `AppXDeploymentExtensions.desktop.dll`.
- privacy.sexy writes both HKCU values with `data: "0"` and
  `deleteOnRevert: 'true' # Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro
  (>= 23H2)` (lines 7580 to 7592). That is the correction: value-absent, revert deletes.
  UNKNOWN 4 in the proposal resolves against the proposal.
- **Duplicate check: passes.** The corpus touches `SearchSettings` once, for
  `IsDynamicSearchBoxEnabled` (interface.yaml 384). Different value.

### 18. `disable_voice_activation` (privacy, Medium) - CONFIRMED

Verified exactly as written, with no corrections.

- Shipped 26100 `AppPrivacy.admx`, both policies `class="Machine"`, key
  `Software\Policies\Microsoft\Windows\AppPrivacy`, `supportedOn="windows:SUPPORTED_Windows_10_0"`,
  each with an `<enum>` of `0` = User is in control, `1` = Force Allow, `2` = Force Deny. Value names
  `LetAppsActivateWithVoice` and `LetAppsActivateWithVoiceAboveLock`. Force Deny = `2` is right and
  the polarity is not inverted.
- ADML for both quotes the "Force Deny" behaviour explicitly, including the above-lock variant.
- Binary presence in `agentactivationruntimewindows.dll` (the voice-activation runtime),
  `AarSvc.dll` and `SettingsHandlers_SpeechPrivacy.dll`.
- The suggested per-user siblings are also real: `AgentActivationEnabled` is present in
  `agentactivationruntimewindows.dll` and `SettingsHandlers_SpeechPrivacy.dll` on 26100.
- **Duplicate check: passes.** No `LetAppsActivateWithVoice*` anywhere in the corpus.

### 19. `disable_windows_backup` (privacy, Medium) - CONFIRMED, with a mandatory copy constraint

- Shipped 26100 `SettingSync.admx`: `class="Machine"`, key
  `Software\Policies\Microsoft\Windows\SettingSync`, `valueName="EnableWindowsBackup"`,
  `enabledValue 1` / `disabledValue 0`, `supportedOn="windows:SUPPORTED_Windows_10_0_NOSERVER"`.
  Proposal matches on every field.
- ADML verbatim: "If you enable this policy setting, windows backup will occur periodically. If you
  disable or do not configure this policy setting, windows backup will not take place."
- Binary presence in `SyncSettings.dll`, `CloudRestoreLauncher.dll` and `cdp.dll` on 26100.
  `CloudRestoreLauncher.dll` is the cloud-restore entry point, which is the one piece of positive
  evidence that writing `0` has an observable effect beyond confirming the documented default.
- **Duplicate check: passes.** The corpus's `disable_settings_sync` uses `DisableSettingSync` and
  `DisableSettingSyncUserOverride` under the same key; different values.
- **Copy constraint, non-negotiable.** By Microsoft's own text, `0` and absent are documented as
  behaviourally identical. The Drawbacks text must say that applying this may produce no visible
  change today and that the Windows Backup app can still be run manually. Do not describe it as
  "stops Windows backing up your files".

### 20. `disable_online_tips` (privacy, Medium) - CONFIRMED, and the proposal is right where a major upstream is wrong

- **Key** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer`, **value**
  `AllowOnlineTips`, **REG_DWORD**, Disabled = `0`, stock default value-absent.
- Shipped 26100 `ControlPanel.admx`: `class="Machine"`,
  `key="Software\Microsoft\Windows\CurrentVersion\Policies\Explorer"`, and a
  `<boolean id="CheckBox_AllowOnlineTips" valueName="AllowOnlineTips">` with
  `trueValue 1` / `falseValue 0`, `supportedOn="windows:SUPPORTED_Windows_10_0_RS3"`. The proposal
  matches exactly.
- ADML: "Enables or disables the retrieval of online tips and help for the Settings app. If disabled,
  Settings will not contact Microsoft content services to retrieve tips and help content."
- Binary presence in **`SystemSettings.dll`**, which is the Settings app itself. That is the
  consumer, so the network-egress claim is supportable.
- **Upstream defect worth recording.** privacy.sexy writes this value to
  `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` (line 30344), which is not the key in the
  shipped ADMX. Anyone cross-checking against privacy.sexy will find a conflict; the shipped ADMX
  wins and the proposal is on the correct side of it.
- **Duplicate check: passes.** The corpus's `disable_tips_and_suggestions` uses
  `SoftLandingEnabled` and `SubscribedContent-338389Enabled` under ContentDeliveryManager. Different
  key, different surface.

### 21. `hide_settings_ai_page` (debloat, Medium) - REJECTED

Facts hold. The tweak should not ship.

- Facts confirmed: shipped 26100 `ControlPanel.admx` defines `SettingsPageVisibility`, `class="Both"`,
  key `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`, with
  `<text id="SettingsPageVisibilityBox" valueName="SettingsPageVisibility" required="true" />`, so
  **REG_SZ** is correct and a REG_DWORD would be silently ignored.
  `supportedOn="windows:SUPPORTED_Windows_10_0_RS2"`. The ADML documents the `showonly:` / `hide:`
  block-list semantics and confirms "Direct navigation to a blocked page via URI, context menu in
  Explorer or other means will result in the front page of Settings being shown instead."
- The page identifier is real on 26100: `aicomponents` (lowercase) is present in `SystemSettings.dll`,
  and `AIComponents` is present in `SystemSettings.dll`, `SystemSettings.DataModel.dll`,
  `SettingsEnvironment.Desktop.dll` and `SystemSettingsAdminFlows.exe`.
- **Rejection ground 1, it changes nothing about AI.** Hiding the page does not disable Image Search,
  Content Extraction or Semantic Analysis. Those keep running. This is concealment presented as
  debloat, and it is the single clearest case of cosmetic theatre in the Medium set.
- **Rejection ground 2, it removes the user's own control surface.** The AI components page is where
  a user turns those components off. A product whose stated purpose is "give the user control over
  their machine" should not ship a switch whose only effect is to hide a control panel from them.
- **Rejection ground 3, the shared-value hazard is real and already demonstrated.** WinUtil's
  `config/tweaks.json` contains **two separate entries** writing `SettingsPageVisibility` (lines 895
  with `"Value": "hide:aicomponents"`, and 1612). That is exactly the clobber the proposal warns
  about, occurring inside a single tool. If the corpus ever wants this surface, it must be one tweak
  owning the value with a composed dropdown, as the proposal says. Given grounds 1 and 2, that
  complexity buys nothing.
- **Shared-value audit result:** this is the **only** proposal in the Medium and Low set that writes
  a multi-owner shared string. Proposals 13, 14 and 15 all live under the same `CloudContent` key but
  each owns a distinct value name, so there is no clobber, only semantic overlap. Everything else is
  a private DWORD. Proposal 24 has a related but different hazard (same value name at two keys with
  two enum bases), described in its entry.

### 22. `disable_nag_toasts` (debloat, Medium) - CONFIRMED, with corrected sourcing

- **Keys** `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.Suggested`
  and `...\Notifications\Settings\Windows.SystemToast.BackupReminder`, **value** `Enabled`,
  **REG_DWORD**, Silenced = `0`, stock default value-absent.
- Win11Debloat `Regfiles/Disable_Windows_Suggestions.reg`, fetched and decoded, writes both, exactly
  as proposed:
  `[HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.Suggested]`
  `"Enabled"=dword:00000000` and the same shape for `Windows.SystemToast.BackupReminder`, with the
  inline comments "Disable 'Suggested' app notifications (Ads for MS services)" and "Disable Windows
  Backup reminder notifications".
- Both toast identifiers exist in shipped 26100 binaries: `Windows.SystemToast.Suggested` in
  `ContentDeliveryManager.Utilities.dll`, `NotificationController.dll` and `SmartActionPlatform.dll`;
  `Windows.SystemToast.BackupReminder` in `shell32.dll`. So neither is a stale identifier.
- The `Notifications\Settings\<AppId>\Enabled` scheme is the per-app store the Settings notifications
  page writes, so the mechanism is the same one the shipped UI uses.
- **Source corrections.** The proposal cites Sophia Script for a
  `Windows.ActionCenter.SmartOptOut` sibling. A raw grep of the Windows 11 `Sophia.psm1` returns no
  `SystemToast` match at all. privacy.sexy does have per-app toast suppression but uses a **different**
  mechanism, `HKLM\SOFTWARE\Classes\AppUserModelId\<id>` plus
  `PushNotifications\Applications\<id>`, and only for `Windows.SystemToast.SecurityAndMaintenance`
  and `Windows.SystemToast.SecurityCenter`. Neither corroborates these two identifiers. Real support
  is Win11Debloat plus shipped-binary presence, which is enough for the identifiers but means this is
  community-corroborated, not Microsoft-documented. Say so in the entry.
- **UNKNOWN 10 stands and is a real implementation requirement.** The per-app keys are created lazily
  on first toast. The apply must create the key, and the status probe must treat "key absent" as the
  stock state, not as an error, or the did-it-work contract produces false failures on a fresh
  profile.
- **Duplicate check: passes**, and the framing is right: this is the surgical alternative to the
  shipped `disable_toast_notifications` (`ToastEnabled` = 0, interface.yaml 787). Present them as
  alternatives.

### 23. `disable_snap_assist` (interface, Medium) - CONFIRMED

- **Key** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, **value** `SnapAssist`,
  **REG_DWORD**, Disabled = `0`.
- Two independent projects agree on key, name, type and polarity. Sophia Script `Sophia.psm1`
  lines 1697 and 1701: `New-ItemProperty -Path HKCU:\Software\Microsoft\Windows\CurrentVersion\
  Explorer\Advanced -Name SnapAssist -PropertyType DWord -Value 0` for Disable and `-Value 1` for
  Enable. Win11Debloat `Regfiles/Disable_Snap_Assist.reg` writes `"SnapAssist"=dword:00000000`.
- Binary presence on 26100 in `twinui.dll` and `twinui.pcshell.dll` (the components that implement
  snap), `Taskbar.View.dll`, and `SettingsHandlers_nt.dll` (the Settings Multitasking page handler),
  where it sits in the same string block as `SnapFill`, `JointResize` and `MultiTaskingAltTabFilter`.
  It is a live `Explorer\Advanced` preference on the target build and it maps to a visible Settings
  toggle, which makes the effect self-verifying.
- **Source correction.** The proposal cites privacy.sexy. privacy.sexy contains no `SnapAssist`
  occurrence. Two projects plus binary plus Settings-UI mapping is still sufficient.
- **Duplicate check: passes.** The corpus's `disable_snap_flyout` writes `EnableSnapAssistFlyout`
  (interface.yaml 535), a different value for the hover-the-maximise-button flyout. The proposal's
  distinction is correct.
- **Residual, low-severity.** The proposal asserts the value is present and `1` on a stock profile
  and that revert must write `1`. That could not be established from an admissible source. If it is
  in fact absent by default, writing `1` on revert leaves a stray value but the effective behaviour
  (Snap Assist on) is identical, so this is not a harmful-revert case. Probe if convenient; do not
  block on it.

### 24. `alt_tab_hide_browser_tabs` (interface, Medium) - CORRECTED

The proposal describes only the user-preference surface and misses a shipped Group Policy that uses
the **same value name with a different enum base**. Writing the proposed `3` at the policy key would
produce the opposite of the requested behaviour.

**The two surfaces, both real on 26100:**

| Surface | Key | Value | Type | Enum |
|---|---|---|---|---|
| Group Policy (tier A) | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` | `MultiTaskingAltTabFilter` | `REG_DWORD` | `1` = windows + 20 tabs, `2` = windows + 5 tabs, `3` = windows + 3 tabs, **`4` = open windows only** |
| User preference (tier C) | `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `MultiTaskingAltTabFilter` | `REG_DWORD` | `0` = 20 tabs, `1` = 5 tabs, `2` = 3 tabs, **`3` = open windows only** |

**Corrected specification (recommended):**

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `MultiTaskingAltTabFilter`
- **Type:** `REG_DWORD`
- **Options:** Windows only = `4`; 3 most recent tabs = `3`; 5 = `2`; 20 = `1`; Stock Default = absent
- Optionally also write the `Explorer\Advanced` preference with the **0-based** enum so the Settings
  UI reflects the change. If both are written they must be kept in step; do not reuse one number
  across both keys.

Evidence:

- Shipped 26100 `Multitasking.admx`: `<policy name="BrowserAltTabBlowout" class="User"
  key="Software\Policies\Microsoft\Windows\Explorer">`,
  `supportedOn="windows:SUPPORTED_Windows_10_0_RS7"`, with
  `<enum id="AltTabFilterDropdown" valueName="MultiTaskingAltTabFilter" required="true">` and items
  `1` = `AltTabFilter_All`, `2` = `AltTabFilter_Five`, `3` = `AltTabFilter_Three`,
  `4` = `AltTabFilter_None`. `en-US\Multitasking.adml`: `AltTabFilter_None` = "Open windows only",
  `AltTabFilter_Three` = "Open windows and 3 most recent tabs in apps", and the help text
  "If this is set to show 'Open windows only', the whole feature will be disabled."
- The 0-based preference enum is pinned by Win11Debloat's four sibling reg files, fetched and
  decoded: `Hide_Tabs_In_Alt_Tab.reg` = `dword:00000003`, `Show_3_Tabs_In_Alt_Tab.reg` =
  `dword:00000002`, `Show_5_Tabs_In_Alt_Tab.reg` = `dword:00000001`, `Show_20_Tabs_In_Alt_Tab.reg` =
  `dword:00000000`, all at
  `[HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced]`.
- Binary presence on 26100 in `twinui.dll` (which implements Alt+Tab, in the `Explorer\Advanced`
  string block alongside `SnapAssist` and `StoreAppsOnTaskbar`), `SettingsHandlers_nt.dll` and
  `SHCore.dll`.
- **Source corrections.** The proposal claims "three independent sources agreeing on the enum",
  naming Win11Debloat, Sophia Script and an ElevenForum tutorial. Sophia's Windows 11 `Sophia.psm1`
  contains no `MultiTaskingAltTabFilter` occurrence. privacy.sexy and WinUtil contain none either.
  The ElevenForum page could not be retrieved through the Wayback Machine. The actual community
  evidence is one project's four sibling files, which do pin the 0-based enum internally but are one
  source, not three. The tier A ADMX is what rescues this proposal.
- Two smaller corrections: `supportedOn` is `SUPPORTED_Windows_10_0_RS7` (2004), not "Windows 11
  21H2 and later"; and the ADML says "app tabs" generically, so the copy should not assert that only
  Microsoft Edge is affected.
- **Duplicate check: passes.** No occurrence anywhere in the corpus.

### 25. `disable_widgets_lock_screen` (interface, Medium) - REJECTED, do not ship

The proposal already flags this as blocked. Verification confirms the conflict is real, adds a
second conflict, and adds a redundancy argument. All three point the same way.

- **Polarity conflict, confirmed from both sides.**
  Shipped 26100 `NewsAndInterests.admx`: `<policy name="DisableWidgetsOnLockScreen" class="Machine"
  key="SOFTWARE\Policies\Microsoft\Dsh" valueName="DisableWidgetsOnLockScreen">` with
  `<enabledValue><decimal value="0" /></enabledValue>` and
  `<disabledValue><decimal value="1" /></disabledValue>`. The ADML says "If you enable this policy
  setting, widgets will not appear on the lock screen." Enabling in gpedit therefore writes **`0`**
  to suppress widgets.
  Policy CSP NewsAndInterests, fetched live, gives the same key and value name but
  `Default Value: 0` and `Allowed values: 0 (Default) = Enabled, 1 = Disabled`, that is, **`1`**
  suppresses widgets. The two Microsoft sources prescribe opposite writes for the same intent. The
  sibling `DisableWidgetsBoard` carries the identical inversion in the ADMX, so it is systematic
  rather than a one-off typo, which makes it impossible to declare either side a clear transcription
  error.
- **Applicability conflict.** The CSP lists Applicable OS as "Windows Insider Preview" only, while
  the shipped ADMX says `SUPPORTED_Windows_11_0_22H2_NOSERVER`. Unresolved.
- **Likely redundancy, new.** The corpus already ships `disable_widgets` writing
  `HKLM\SOFTWARE\Policies\Microsoft\Dsh` `AllowNewsAndInterests` = `0` (debloat.yaml 216). The
  26100 `NewsAndInterests.adml` describes that policy as "This policy specifies whether the widgets
  feature **is allowed on the device**", `class="Machine"`, `enabledValue 1` / `disabledValue 0`.
  A device-wide "widgets not allowed" almost certainly removes the lock-screen panel too, which
  would make this proposal redundant regardless of how the polarity resolves. That is UNKNOWN 7 in
  the proposal, still open, but it now has an ADML sentence behind it.
- **Verdict rationale.** Writing the wrong value would silently leave enabled a surface the user
  asked to remove, and the corpus would report success. That is a direct did-it-work-contract
  violation. Two conflicting Microsoft sources plus a live redundancy question is not a shippable
  state. Rejected until a read-modify-observe probe on a real 26100 or 26200 machine with lock-screen
  widgets on settles it, and until `AllowNewsAndInterests` = 0 is tested against the same surface.

### 26. `taskbar_last_active_click` (interface, Low) - CONFIRMED

- **Key** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, **value**
  `LastActiveClick`, **REG_DWORD**, Focus last active = `1`, stock default value-absent.
- Binary presence on 26100 in **`Taskbar.View.dll`** (twice) and **`Taskbar.dll`**. These are the
  Windows 11 taskbar itself, not policy plumbing, so the value is read by the shipping taskbar on the
  target build. This is the decisive evidence and it is the kind the brief accepts.
- Win11Debloat `Regfiles/Enable_Last_Active_Click.reg`, fetched and decoded, writes
  `"LastActiveClick"=dword:00000001` at that key, with an unusually explicit inline description that
  matches the proposal's claimed behaviour including "the pop-up window display will still show if
  you hover your mouse over the taskbar icon".
- **Source correction.** The proposal cites Sophia Script. The Windows 11 `Sophia.psm1` contains no
  `LastActiveClick` occurrence, nor do privacy.sexy or WinUtil. The ElevenForum tutorial could not be
  retrieved through the Wayback Machine. Real support is one project plus binary presence. That is
  below the three-independent-tier-C bar, but binary presence in the consumer is stronger than any
  number of community sources for the question "does this value exist and get read", and the
  semantics are trivially self-verifying by clicking a grouped taskbar icon. Confirmed on that basis;
  label it community-corroborated in the entry.
- **Duplicate check: passes.** No occurrence in the corpus.

### 27. `explorer_expand_to_current_folder` (interface, Low) - CONFIRMED

- **Key** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, **value**
  `NavPaneExpandToCurrentFolder`, **REG_DWORD**, Expand = `1`, stock default value-absent.
- Sophia Script `Sophia.psm1` lines 3434 and 3438 write exactly this at exactly this key with
  `-Value 0` and `-Value 1`, confirming type and polarity.
- Binary presence on 26100 in `shell32.dll`, `ExplorerFrame.dll` and `Windows.UI.FileExplorer.dll`,
  all Explorer consumers.
- The Folder Options checkbox "Expand to open folder" makes the mapping self-verifying in the UI,
  as the proposal says.
- **Duplicate check: passes.**

### 28. `explorer_restore_folders_at_logon` (interface, Low) - CONFIRMED

- **Key** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, **value**
  `PersistBrowsers`, **REG_DWORD**, Restore = `1`, stock default value-absent.
- Sophia Script `Sophia.psm1` lines 6629 and 6633 write exactly this at exactly this key with
  `-Value 0` and `-Value 1`.
- Binary presence on 26100 in `shell32.dll`, `ExplorerFrame.dll` and `gpprefcl.dll`.
- Maps to the Folder Options checkbox "Restore previous folder windows at logon", so the effect is
  trivially verifiable.
- **Duplicate check: passes.**

### 29. `taskbar_full_date_time` (interface, Low) - CONFIRMED, with one addition

- **Key** `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, **value**
  `TurnOffAbbreviatedDateTimeFormat`, **REG_DWORD**, Full format = `1`, stock default value-absent.
- Shipped 26100 `Taskbar.admx`: `class="User"`, key `Software\Policies\Microsoft\Windows\Explorer`,
  `enabledValue 1` / `disabledValue 0`, `supportedOn="windows:SUPPORTED_Windows_11_0_22H2"`.
  Every field in the proposal matches.
- ADML: "This policy setting allows you to show the longer time and date format in the system tray.
  If this setting is enabled, the time format will include the AM/PM time marker and the date will
  include the year. **A reboot is required for this policy setting to take effect.**"
- Binary presence on 26100 in **`Taskbar.View.dll`** (twice) and `SettingsHandlers_DesktopTaskbar.dll`.
  The Windows 11 tray clock reads it.
- **Addition the proposal missed:** the copy must state the reboot requirement, otherwise a user
  applies it, sees nothing, and reasonably concludes the tweak is broken.
- The suggested companion `AlwaysShowNotificationIcon` is equally real: same ADMX file, same key,
  `class="User"`, `enabledValue 1`, `supportedOn="windows:SUPPORTED_Windows_11_0_22H2"`, ADML
  "Show notification bell icon ... Otherwise, the notification icon will only be shown when there's a
  special status (for example, Do Not Disturb is turned on). A reboot is required." Ship or skip both
  together, as the proposal says.
- **Duplicate check: passes.** `seconds_in_tray_clock` uses `ShowSecondsInSystemClock` under
  `Explorer\Advanced` (interface.yaml 355). Different key, different value, no collision.

### 30. `hide_recently_added_apps` (interface, Low) - CONFIRMED, applicability corrected

- **Key** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` and
  `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, **value** `HideRecentlyAddedApps`,
  **REG_DWORD**, Hidden = `1`, stock default value-absent. The proposal is correct on all of these.
- Shipped 26100 `StartMenu.admx`: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`,
  no explicit `enabledValue`, which per the ADMX schema defaults to REG_DWORD `1` enabled / `0`
  disabled. ADML: "Remove 'Recently added' list from Start Menu ... The corresponding setting will
  also be disabled in Settings."
- **Applicability corrected.** The proposal says "Windows 10 1803 and later". The ADMX says
  `SUPPORTED_Windows_10_0_RS4` (1803) but Policy CSP Start > `HideRecentlyAddedApps` says
  "Windows 10, version 1703 [10.0.15063] and later", with editions Pro, Enterprise, Education,
  IoT Enterprise / IoT Enterprise LTSC, and both Device and User scope. Use 1703.
- **Two additions the proposal missed, both from the CSP page:**
  1. "This policy requires a reboot to take effect."
  2. Microsoft's own validation steps are written against the Windows 11 Settings toggle:
     "In the Settings app, enable the Show recently added apps option ... Check that the Show
     recently added apps Settings toggle is grayed out." This settles the concern that the policy is
     a Windows 10-only surface. It is documented against the current Start.
- Binary presence on 26100 in `StartTileData.dll`, the Start app-list data model. It does not appear
  in any other shell consumer, which is worth noting but is consistent with `StartTileData.dll`
  being the component that owns the recently-added list.
- **Not subsumed by anything shipped.** The corpus's `disable_start_recommendations`
  (`Start_IrisRecommendations` = 0, interface.yaml 595) suppresses the promotional rows only;
  recently-installed apps keep appearing. This is the fallback if proposal 1
  (`HideRecommendedSection`) does not land on Pro.
- **Duplicate check: passes.**

### 31. `disable_new_app_alert` (interface, Low) - REJECTED

- Facts confirmed: shipped 26100 `WindowsExplorer.admx`, `class="Machine"`, key
  `Software\Policies\Microsoft\Windows\Explorer`, `valueName="NoNewAppAlert"`, `enabledValue 1`,
  `supportedOn="windows:SUPPORTED_Windows8"`. ADML: "Do not show the 'new application installed'
  notification ... If this group policy is enabled, no notifications will be shown."
- **Rejection ground 1, no consumer evidence.** `NoNewAppAlert` appears in exactly one shipped
  binary on 26100, `SHCore.dll`, which holds the shell policy table. It appears in no feature
  component. Compare with the other Low proposals, every one of which was found in a real consumer
  (`Taskbar.View.dll`, `twinui.pcshell.dll`, `ExplorerFrame.dll`, `StartTileData.dll`). That is not
  proof the policy is dead, since shell code can reach the table by policy id, but combined with a
  Windows 8-era `supportedOn` it means the real-world effect on 26100 is unproven.
- **Rejection ground 2, the benefit is negative.** The notification it suppresses fires rarely, and
  it is the user's only signal that an installer changed their file or protocol associations, which
  is a well-known adware and browser-hijack vector. The proposal itself concedes the
  security-awareness cost. Trading a real security signal for the removal of an occasional toast is
  a bad deal, and the corpus already has a global toast control for users who want silence.
- Do not ship. Facts recorded so it is not rediscovered.

### 32. `disable_notification_center` (interface, Low) - CORRECTED

Real and effective on 26100. Three corrections to the entry, one of them a factual claim that must
not be asserted.

**Corrected specification:**

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` **or**
  `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` (the ADMX is `class="Both"`, so machine scope
  is available and is the better choice for a machine-wide toolbox action)
- **Value name:** `DisableNotificationCenter`
- **Type:** `REG_DWORD`
- **Options:** Removed = `1`; Present (Stock Default) = absent
- **Requires a reboot.**

Evidence and corrections:

- Shipped 26100 `Taskbar.admx`: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`,
  `enabledValue 1` / `disabledValue 0`, `supportedOn="windows:SUPPORTED_Windows_10_0"`.
- Binary presence in **`Windows.UI.ActionCenter.dll`**, **`Taskbar.View.dll`** (twice) and
  **`twinui.pcshell.dll`**, that is, in the components that actually render the notification centre
  on Windows 11. This is the strongest consumer evidence in the entire Low set and it settles the
  "does it still work on 26100" question affirmatively.
- **Correction 1, an unsupported claim.** The proposal states it removes "the Notification Center
  *and* the calendar flyout". The ADML does not say that. It says: "This policy setting removes
  Notifications and Action Center from the notification area on the taskbar ... The user will be able
  to read notifications when they appear, but they won't be able to review any notifications they
  miss." On Windows 11 the calendar and the notification list share one flyout, so losing the
  calendar is plausible, but it is an inference. Do not state it as documented fact; either probe it
  or phrase it as "may also remove the calendar flyout, which shares the same panel".
- **Correction 2:** add the reboot requirement, which the ADML states explicitly.
- **Correction 3, sourcing:** WinUtil `config/tweaks.json` does contain `DisableNotificationCenter`
  (line 1012), so that citation holds. Sophia Script does not; privacy.sexy does not. The ADMX is
  tier A, so this does not matter for the verdict.
- **Duplicate check: passes.** The corpus's `disable_toast_notifications` writes `ToastEnabled` under
  `PushNotifications` (interface.yaml 803). Different key, different effect. The proposal's framing
  that the corpus currently offers nothing between "all toasts off" and "leave it alone" is accurate.
- Ship it only with a blunt warning in the copy: notification history is gone, not just quieter.

### 33. Extend `disable_suggested_content_settings` with `SubscribedContent-353698Enabled` (privacy, Low) - CONFIRMED

- **Key** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, **value**
  `SubscribedContent-353698Enabled`, **REG_DWORD**, Disabled = `0`, stock default value-absent.
- Win11Debloat `Regfiles/Disable_Windows_Suggestions.reg`, fetched and decoded, writes it in the same
  block, under the comment "Show me suggested content in the Settings app", as the exact three the
  corpus already ships:

  ```
  ; Show me suggested content in the Settings app
  [HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager]
  "SubscribedContent-338393Enabled"=dword:00000000
  "SubscribedContent-353694Enabled"=dword:00000000
  "SubscribedContent-353696Enabled"=dword:00000000
  "SubscribedContent-353698Enabled"=dword:00000000
  ```

  That is a direct match with privacy.yaml lines 691, 693 and 695, so it genuinely closes the set.
- **Independent confirmation that the subscription id is live on 26100.** The literal string
  `353698` is present in `ContentDeliveryManager.Background.dll`,
  `ContentDeliveryManager.Utilities.dll` and `Windows.Services.TargetedContent.dll`. Note that the
  full value name is never a literal in any binary, for any of these slots, because Content Delivery
  Manager composes `SubscribedContent-<id>Enabled` at runtime; `SubscribedContent-338393Enabled` and
  `SubscribedContent-338388Enabled`, both already shipped and both known-good, are equally absent as
  literals. So literal absence of the full name is not evidence against, and presence of the bare id
  is the right test. It passes.
- **Source correction.** The proposal says the slot is "corroborated by privacy.sexy and Sophia
  Script". Neither contains `353698`, nor does WinUtil. Support is Win11Debloat plus the shipped
  binaries. That is still adequate, because this is an additive extension to an existing tweak whose
  three siblings are already validated and whose failure mode is nil: if the slot is inert, writing
  `0` to it does nothing.
- **Risk: none.** Revert deletes, consistent with the three existing effects.

---

## Source-attribution failures in the original proposal

These matter because several proposals reached their claimed tier by counting sources that do not
exist. Verified by raw fetch of each project file and a literal search.

| Proposal | Claimed source | Actual |
|---|---|---|
| 14 | privacy.sexy | `DisableThirdPartySuggestions` absent from privacy.sexy `windows.yaml` |
| 15 | privacy.sexy | `DisableSpotlightCollectionOnDesktop` absent from privacy.sexy |
| 22 | Sophia Script (`Windows.ActionCenter.SmartOptOut`) | no `SystemToast` or `SmartOptOut` match in Windows 11 `Sophia.psm1`; privacy.sexy uses a different mechanism and different toast ids |
| 23 | privacy.sexy | `SnapAssist` absent from privacy.sexy |
| 24 | Sophia Script, ElevenForum | `MultiTaskingAltTabFilter` absent from Sophia, privacy.sexy and WinUtil; ElevenForum page not retrievable via Wayback |
| 26 | Sophia Script | `LastActiveClick` absent from Sophia, privacy.sexy and WinUtil |
| 33 | privacy.sexy, Sophia Script | `353698` absent from both, and from WinUtil |

In every case the verdict survived on tier A ADMX or on shipped-binary evidence, so no proposal was
rejected purely for this. But the tier labels in the original document overstate the corroboration
and should be corrected before anyone treats them as load-bearing.

## Two upstream defects found, worth recording

1. **privacy.sexy writes `AllowOnlineTips` to the wrong key.** It uses
   `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` (line 30344). The shipped 26100
   `ControlPanel.admx` says `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`. Proposal
   20 has the right key. Do not "fix" it toward privacy.sexy.
2. **privacy.sexy has an inverted-polarity bug in its search-history script.** Lines 7494 to 7508
   set `DisableSearchHistory` to `"1"` and, in the same script, `IsDeviceSearchHistoryEnabled` to
   `"1"` as well. `IsDeviceSearchHistoryEnabled` = 1 **enables** device search history, so that half
   of the script does the opposite of its stated intent. This is a caution about treating
   privacy.sexy as a semantics oracle, and it is part of why proposal 16 is corrected rather than
   confirmed.

## Shared-value collision audit (brief question 6)

Checked every Medium and Low proposal for the `SettingsPageVisibility` failure mode, a single value
that more than one tweak would want to own.

- **Proposal 21, `SettingsPageVisibility`: real, and the only one of its kind here.** Confirmed by
  the ADMX `<text>` element (list semantics, REG_SZ) and demonstrated in the wild by WinUtil, which
  writes that value from two separate entries in one config file. Rejected on other grounds anyway.
- **Proposal 24, a different hazard of the same family:** one value name, `MultiTaskingAltTabFilter`,
  living at two keys with two incompatible enum bases. Not a clobber, but a silent
  wrong-value hazard, which is arguably worse because nothing detects it. Handled in the entry.
- **Proposals 13, 14 and 15** share the `HKCU\...\Policies\...\CloudContent` key but each owns a
  distinct value name. No clobber. Semantic overlap only, which affects tweak copy and status
  presentation, not revert correctness.
- **Everything else** writes a private DWORD that no other proposal or shipped tweak touches. The
  duplicate scan across all seven corpus YAML files returned zero collisions for every proposed
  value name in this set.

## Summary table

| # | Id | Verdict | One line |
|---|---|---|---|
| 13 | `disable_windows_spotlight_all` | CONFIRMED | ADMX, ADML and consumer binaries all match; describe the overlap with four shipped tweaks |
| 14 | `disable_third_party_suggestions` | REJECTED | Documented CSP dependency makes it inert once Spotlight is off; strict subset of 13; cited source does not contain it |
| 15 | `disable_spotlight_desktop` | CONFIRMED | ADMX plus an exact Win11Debloat match; overlaps 13 |
| 16 | `disable_search_history` | CORRECTED | Drop `DisableSearchHistory` (User class not HKLM, Win8Only, no Win11 consumer); keep `IsDeviceSearchHistoryEnabled` = 0; revert deletes, does not write 1 |
| 17 | `disable_cloud_content_search` | CORRECTED | Keys and values right; the two HKCU values are absent by default, so revert deletes |
| 18 | `disable_voice_activation` | CONFIRMED | ADMX enum 0/1/2 confirmed, Force Deny = 2, no inversion |
| 19 | `disable_windows_backup` | CONFIRMED | Real control; copy must not overclaim, since 0 and absent are documented as equivalent |
| 20 | `disable_online_tips` | CONFIRMED | Key matches ADMX and the value is read by `SystemSettings.dll`; privacy.sexy's key is wrong, the proposal's is right |
| 21 | `hide_settings_ai_page` | REJECTED | Hides a page without disabling anything, and hides the page the user would use to disable it; shared-value hazard demonstrated in WinUtil |
| 22 | `disable_nag_toasts` | CONFIRMED | Both toast ids exist in shipped binaries and Win11Debloat writes both; apply must create the lazily-created keys |
| 23 | `disable_snap_assist` | CONFIRMED | Two projects plus `twinui`/`Taskbar.View` presence plus a Settings toggle; revert value is a minor open question with no harmful outcome |
| 24 | `alt_tab_hide_browser_tabs` | CORRECTED | Same value name at two keys with different enum bases; policy key wants `4` for windows-only, `Explorer\Advanced` wants `3` |
| 25 | `disable_widgets_lock_screen` | REJECTED | ADMX and CSP prescribe opposite writes, applicability also conflicts, and the shipped `AllowNewsAndInterests` = 0 may already cover it |
| 26 | `taskbar_last_active_click` | CONFIRMED | Read by `Taskbar.View.dll` and `Taskbar.dll` on 26100; community support thinner than claimed |
| 27 | `explorer_expand_to_current_folder` | CONFIRMED | Sophia plus three Explorer binaries plus a Folder Options checkbox |
| 28 | `explorer_restore_folders_at_logon` | CONFIRMED | Sophia plus `shell32`/`ExplorerFrame` plus a Folder Options checkbox |
| 29 | `taskbar_full_date_time` | CONFIRMED | ADMX exact, read by `Taskbar.View.dll`; add the documented reboot requirement |
| 30 | `hide_recently_added_apps` | CONFIRMED | ADMX plus a CSP page whose validation steps are written against Windows 11 Settings; 1703 not 1803, reboot required |
| 31 | `disable_new_app_alert` | REJECTED | No consumer-binary evidence on 26100, Windows 8-era `supportedOn`, and it suppresses a genuine file-association security signal |
| 32 | `disable_notification_center` | CORRECTED | Strongly confirmed by `Windows.UI.ActionCenter.dll`; class is Both so HKLM is available; reboot required; the calendar-flyout claim is inference, not documentation |
| 33 | extend with `SubscribedContent-353698Enabled` | CONFIRMED | Win11Debloat writes it beside the three already shipped, and the id `353698` is a literal in the 26100 Content Delivery Manager binaries |

## UNKNOWNS remaining after this pass

1. **`DisableWidgetsOnLockScreen` polarity (proposal 25).** Still unresolved and now confirmed from
   both sides: the shipped ADMX writes `0` to suppress, the CSP says `1` suppresses. Needs a
   read-modify-observe probe on 26100 or 26200 with lock-screen widgets enabled. Blocking.
2. **Whether the shipped `AllowNewsAndInterests` = 0 already removes lock-screen widgets on 24H2.**
   If yes, proposal 25 is redundant whatever the polarity turns out to be. Same probe can answer it.
3. **`SnapAssist` stock presence (proposal 23).** Whether a fresh 26100 profile has the value present
   and `1`, or absent. Non-blocking: both revert choices leave Snap Assist on, so no harmful-revert
   risk either way. It only affects whether revert deletes or writes `1`.
4. **Whether `DisableNotificationCenter` also removes the clock and calendar flyout on Windows 11
   (proposal 32).** Not documented in the ADML. Needs a live check before the copy can claim it.
5. **Observable effect of `EnableWindowsBackup` = 0 on an unmanaged consumer machine (proposal 19).**
   The value is read by `SyncSettings.dll` and `CloudRestoreLauncher.dll`, so something consumes it,
   but whether a user sees any difference (for example, the cloud-restore offer disappearing at next
   OOBE) is unverified. Affects the Drawbacks wording, not the correctness of the tweak.
6. **`MultiTaskingAltTabFilter` precedence (proposal 24).** Both the policy key and the
   `Explorer\Advanced` preference are read by shipped code. Which wins when they disagree was not
   determined. If the corpus writes only the policy value, the Settings Multitasking dropdown may
   continue to display the old preference. Cosmetic, but worth knowing before choosing whether to
   write one surface or both.
