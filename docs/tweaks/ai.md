# AI & Copilot tweaks

This category collects the Windows AI controls in one place: Recall (snapshot policy, component policy and the optional feature), Click to Do, the Copilot app and its taskbar button, the AI features inside Notepad, Paint and Microsoft Edge, and the `WSAIFabricSvc` AI broker service. The primary platform is Windows 11 24H2 (build 26100) and newer, including 25H2 (26200); Windows 10 IoT Enterprise LTSC 2021 (build 19044) is a secondary target, and only two tweaks here do anything there (the Copilot taskbar policy on 19044.3758 or newer, and the Edge policies wherever a current Edge is installed). Most tweaks are machine-wide policy values that the snapshot restores exactly; the exceptions are two script actions (Copilot app removal and the Recall optional feature) and one service start-type change. Several Recall and Click to Do controls only have a visible effect on Copilot+ PCs, and most of the WindowsAI and Paint policies are documented for Pro, Enterprise, Education and IoT Enterprise, not Home.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Disable Windows Recall snapshots](#disable-windows-recall-snapshots) | `disable_recall_snapshots` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Recall feature component](#recall-feature-component) | `remove_recall_component` | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable the Click to Do overlay](#disable-the-click-to-do-overlay) | `disable_click_to_do` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the Copilot app](#remove-the-copilot-app) | `remove_copilot_app` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable the Recall optional feature](#disable-the-recall-optional-feature) | `remove_recall_feature` | Switch (2 options) | medium | admin | yes | INCORRECT (corrected form ships) |
| [Hide the Copilot taskbar button](#hide-the-copilot-taskbar-button) | `disable_copilot_taskbar` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Notepad AI features](#disable-notepad-ai-features) | `disable_notepad_ai` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Paint AI features](#disable-paint-ai-features) | `disable_paint_ai` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off Edge AI features](#turn-off-edge-ai-features) | `disable_edge_ai_features` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Set the AI Fabric service to Manual start](#set-the-ai-fabric-service-to-manual-start) | `disable_ai_fabric_service` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |

"Switch" means one or two authored options, shown as a segmented switch; "Dropdown" means three or more. The app adds the computed **System Default** state to either while it is the live state (the state when the machine matches no authored option; selecting it restores the snapshot).

## Tweaks

### Disable Windows Recall snapshots

`disable_recall_snapshots` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: build >= 26100 (Microsoft's real floor is 26100.3915, see below) · Reversible: yes

**Locks Recall's screen-snapshot capture off for every user on the PC, and deletes the snapshots already saved.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `recall` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`, value `DisableAIDataAnalysis`, `REG_DWORD` |

| Option | `recall` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` (value deleted) |

System Default: shown when `DisableAIDataAnalysis` holds anything other than `1` or absent (for example `0` written by another tool or a GPO); selecting it restores the value captured in the snapshot. Stock Windows has no value here, which reads as the Enabled option (Microsoft documents the default as 0, snapshots may be saved subject to user opt-in).

#### How it works

This is the Group Policy "Turn off saving snapshots for use with Recall" (Windows Components > Windows AI), exposed in MDM as Policy CSP `WindowsAI/DisableAIDataAnalysis`, defined in `WindowsCopilot.admx`. Microsoft documents it as Format `int`, Default Value 0, with 0 meaning snapshots may be saved and 1 meaning snapshots are not saved. When the policy is enabled, Windows stops Recall from saving new snapshots and, per Microsoft, also deletes any snapshots already stored on the device.

The ADMX declares the policy with `class="Both"`, so it exists at machine scope (`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`) and at user scope (`HKCU\Software\Policies\Microsoft\Windows\WindowsAI`). When both are set, Computer Configuration wins for Recall enforcement, which is why the tweak writes the HKLM value: it is the stronger of the two and covers every account. It is a real policy-hive value, not a preference.

Recall itself only exists on a Copilot+ PC meeting the Secured-core standard: a 40 TOPS NPU, 16 GB RAM, 8 logical processors, 256 GB storage, device encryption or BitLocker, and Windows Hello Enhanced Sign-in Security with a biometric enrolled. On any other hardware the policy has no feature to act on. Microsoft also states that on managed (commercial) devices snapshots are not enabled by default and saving them requires individual user opt-in consent, so on such a device the policy mostly locks in the existing state.

The Policy CSP lists applicability as Windows 11 24H2 with KB5055627 (build 10.0.26100.3915) and later. The app's `windows:` grammar does not support revision scoping (any `revision:` is a build error), and once it does it will only allow a revision against a single exact build, which would exclude 25H2. So the tweak is gated on the build line (`>= 26100`) and the revision floor is stated in the copy. On a 26100 build below revision 3915 the write succeeds but Windows ignores it.

Supported editions per Microsoft: Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. Home is not listed; the tweak carries no edition gate, so on Home the write is expected to be inert.

#### Benefits
- No new Recall snapshots are written to local storage.
- Snapshots already saved on the device are deleted.
- Machine-wide: applies to every account, not only the one that applied it.

#### Drawbacks
- Recall's timeline and its search stop working for every user on the PC.
- Deletion is permanent: reverting restores the policy value, never the deleted snapshots.
- Inert on hardware that is not a Copilot+ PC, and on 26100 builds below 26100.3915.
- Not read on Home, per Microsoft's edition list.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 build 26100.3915 (KB5055627) and newer, including 25H2; Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC; only meaningful on Copilot+ hardware. Not present on Windows 10 LTSC 2021 (the tweak is hidden there by its build gate).
- **Takes effect**: after a reboot, which also lets the snapshot deletion settle.
- **Reverting**: System Default or the Enabled option removes or restores the policy value from the snapshot. The snapshots Windows deleted when the policy was applied do not come back.

#### Interactions
- [Recall feature component](#recall-feature-component) (`AllowRecallEnablement`) lives under the same key and is the stronger lever: it removes the component rather than stopping capture. With the component set to Available, this snapshot policy still decides whether snapshots are saved.
- [Disable the Recall optional feature](#disable-the-recall-optional-feature) disables the Recall optional feature through DISM; the research names this snapshot policy as its documented, reversible alternative.
- [Disable the Click to Do overlay](#disable-the-click-to-do-overlay) does not cover Click to Do inside Recall; this tweak (or the component tweak) is needed for that surface.
- No other tweak writes `DisableAIDataAnalysis`.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The mechanism (key, value, type, polarity, HKLM scope) is correct; the research established that Microsoft's applicability floor is 26100.3915, not the whole 26100 line, that Home is not a supported edition, and that applying deletes stored snapshots permanently. All three facts are stated in the shipped copy; the gate stays at the build line because the `windows:` grammar does not support revision scoping.
- **Confidence**: Microsoft-documented. The Policy CSP page gives the format, default, allowed values, applicability, editions and scopes; the Manage Recall page documents the deletion, the hardware prerequisites and the managed-device opt-in default.
- **Reasoning**: The policy-hive audit checked the ADMX class and confirmed that writing HKLM for a `class="Both"` policy is legitimate and the stronger choice, since Computer Configuration wins for Recall enforcement. No source contradicted the value or its polarity. The open point is enforcement on Home, which Microsoft does not claim.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any Copilot+ PC where you want certainty that Recall never captures the screen, and accept that existing snapshots are deleted with it. If you use Recall's timeline, leave it alone: this removes the choice for every user rather than offering it. On non-Copilot+ hardware it is harmless and does nothing visible.

#### Sources
1. Policy CSP - WindowsAI, `DisableAIDataAnalysis`: Format `int`, Default Value 0, allowed values 0 and 1, applicability Windows 11 24H2 with KB5055627 (10.0.26100.3915) and later, editions Pro / Enterprise / Education / IoT Enterprise / IoT Enterprise LTSC, Device and User scope, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
2. Manage Recall for Windows clients: Copilot+ and Secured-core prerequisites, snapshot deletion when the policy is enabled, managed-device default and user opt-in consent, https://learn.microsoft.com/en-us/windows/client-management/manage-recall (tier A)
3. Shipped `WindowsCopilot.admx`: `DisableAIDataAnalysis` declared `class="Both"`, Computer Configuration wins (repository policy-hive audit, internal analysis)

### Recall feature component

`remove_recall_component` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: build >= 26100 (Microsoft's real floor is 26100.3915) · Reversible: yes

**Strips the Recall component off the device instead of just switching it off.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `allow_recall` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`, value `AllowRecallEnablement`, `REG_DWORD` |

| Option | `allow_recall` |
|---|---|
| Removed | `0` |
| Disabled | `absent` (value deleted; the policy's "not configured" state) |
| Available | `1` |

System Default: shown when `AllowRecallEnablement` holds a value other than 0, 1 or absent; selecting it restores the snapshot. Stock Windows has no value here, which reads as the Disabled option: Microsoft documents that with the policy not configured, the Recall component is present on the device in a disabled state.

#### How it works

This is the Group Policy "Allow Recall to be enabled", Policy CSP `WindowsAI/AllowRecallEnablement`, defined in `WindowsCopilot.admx` with `class="Machine"`, so HKLM is the only valid hive and there is no per-user version. Microsoft documents three states. Set to 0, the Recall component is disabled, its bits are removed from the device, saved snapshots are deleted, and a restart is required. Not configured, end users have the Recall component in a disabled state. Set to 1, Recall becomes available, and whether it saves snapshots is then governed by the separate `DisableAIDataAnalysis` policy.

That is why this tweak is a three-option dropdown rather than a toggle: deleting the value does not make Recall available, only an explicit 1 does. It is the strongest Recall lever Windows offers because it removes the feature payload instead of gating it.

Restoring Recall after removal needs the Available option and, on some commercial devices, an explicit re-provision with `Enable-WindowsOptionalFeature -Online -FeatureName "Recall"`, which this tweak does not run.

As with the snapshot policy, Microsoft's applicability is Windows 11 24H2 with KB5055627 (26100.3915) and later; the tweak's gate is the build line because the `windows:` grammar does not support revision scoping, and earlier 26100 builds ignore the value. Editions: Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC; not Home. Recall itself only exists on Copilot+ PCs.

#### Benefits
- The Recall payload is removed from disk, not merely switched off.
- No leftover component that a setting or update can quietly switch back on.
- Saved Recall snapshots are deleted along with the component.
- The Available option gives a real, documented path back to Recall.

#### Drawbacks
- Getting Recall back after Removed takes the Available option and may also need `Enable-WindowsOptionalFeature -Online -FeatureName "Recall"`.
- Deleted snapshots do not return under any option.
- The stock (Disabled, not configured) state does not hand Recall back: a revert to it leaves Recall present but disabled.
- Inert on non-Copilot+ hardware, on Home, and on 26100 builds below 26100.3915.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 build 26100.3915 (KB5055627) and newer, including 25H2; Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC; Copilot+ hardware. Not present on Windows 10 LTSC 2021.
- **Takes effect**: after a reboot, which Microsoft requires for the component removal.
- **Reverting**: System Default restores the captured value from the snapshot. On a stock machine that is the not-configured state, where Recall is present but disabled. Reverting never restores the payload removed by Removed on its own, and never restores deleted snapshots.

#### Interactions
- [Disable Windows Recall snapshots](#disable-windows-recall-snapshots) writes `DisableAIDataAnalysis` under the same key. With this tweak at Available, the snapshot policy still decides whether snapshots are saved; with this tweak at Removed, the snapshot policy has nothing to act on.
- [Disable the Recall optional feature](#disable-the-recall-optional-feature) disables the same feature at the DISM optional-feature level. Removed here plus that tweak is redundant but harmless. If this tweak is set to Removed, restoring Recall may require re-enabling the optional feature, which that tweak's revert only does when Recall was enabled before it ran.
- [Disable the Click to Do overlay](#disable-the-click-to-do-overlay) does not cover Click to Do inside Recall; removing Recall removes that surface.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The key, value and type are correct. The research established that the not-configured state (value absent) leaves Recall disabled, not available, so a literal 1 is required for Available; the shipped dropdown has three options (0, absent, 1) that match Microsoft's three documented states, and the risk is medium because the apply deletes snapshots and removes the payload.
- **Confidence**: Microsoft-documented. The Policy CSP page defines all three states, device-only scope and the restart; Manage Recall documents re-provisioning.
- **Reasoning**: The cross-cutting harmful-revert audit singled this policy out: a two-option shape whose second option writes `absent` while claiming "Available" would promise something Windows does not do. Microsoft's own wording ("end users will have the Recall component in a disabled state") settled it. The policy-hive audit confirmed `class="Machine"` and HKLM.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Pick Removed if you want Recall gone rather than held off and you are confident you will not want it back. If there is any chance you will try Recall later, use the [snapshot policy](#disable-windows-recall-snapshots) instead, which is a clean on-off. Use Available only if you want Recall offered and understand the snapshot policy still governs capture.

#### Sources
1. Policy CSP - WindowsAI, `AllowRecallEnablement`: three documented states, device scope only, restart required on disable, snapshot deletion, applicability Windows 11 24H2 with KB5055627 (10.0.26100.3915) and later, editions Pro / Enterprise / Education / IoT Enterprise / IoT Enterprise LTSC, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
2. Manage Recall for Windows clients: re-provisioning via `Enable-WindowsOptionalFeature -Online -FeatureName "Recall"` on commercial devices, Copilot+ hardware prerequisites, https://learn.microsoft.com/en-us/windows/client-management/manage-recall (tier A)
3. Repository harmful-revert audit: not-configured leaves Recall disabled, so the Available state needs an explicit 1 (internal analysis)
4. Shipped `WindowsCopilot.admx`: `AllowRecallEnablement` declared `class="Machine"` (repository policy-hive audit, internal analysis)

### Disable the Click to Do overlay

`disable_click_to_do` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: build >= 26100 · Reversible: yes

**Removes the Click to Do overlay, so nothing screenshots and analyzes your screen on demand.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `click_to_do` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`, value `DisableClickToDo`, `REG_DWORD` |

| Option | `click_to_do` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` (value deleted) |

System Default: shown when `DisableClickToDo` holds a value other than 1 or absent (for example 0); selecting it restores the snapshot. Stock Windows has no value, which reads as Enabled (documented Default Value 0, Click to Do enabled).

#### How it works

This is the Group Policy "Disable Click to Do" (Windows Components > Windows AI), Policy CSP `WindowsAI/DisableClickToDo`, in `WindowsCopilot.admx`. Microsoft documents Default Value 0 and value 1 to disable, and states that when the policy is enabled "the Click to Do component and entry points won't be available to users". Click to Do takes an on-demand full-screen capture and runs its analysis (Summarize, Rewrite and other actions) locally on the device, so this policy removes a capture surface rather than blocking a cloud upload.

The ADMX declares the policy `class="Both"` (Computer and User Configuration); Computer Configuration wins, so the HKLM write covers every account. A per-user switch also exists at Settings > Privacy & security > Click to Do.

Two limits matter. Microsoft states the policy does not affect Click to Do inside Recall, so the Recall tweaks are needed for that surface. And the Policy CSP applicability column for this policy still reads "Windows Insider Preview", the only Microsoft statement on availability the research found, so Microsoft's own table does not guarantee the behaviour on a given retail 24H2 or 25H2 build.

Click to Do needs a Copilot+ PC (40 TOPS NPU, 16 GB RAM, 8 logical processors, 256 GB storage) or an eligible Cloud PC. The `>= 26100` build gate admits every 24H2 machine; on the great majority of them the value is inert. Editions: Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC.

#### Benefits
- The on-demand full-screen capture and its entry points are removed.
- Machine-wide, unlike the per-user Settings switch.
- The entry points go away rather than showing a greyed-out switch.

#### Drawbacks
- Summarize, Rewrite and the other Click to Do actions become unavailable.
- Does not disable Click to Do inside Recall.
- Inert without a Copilot+ PC or an eligible Cloud PC.
- Not a cloud block: the analysis was on-device to begin with.
- Microsoft's policy table still lists the policy under Windows Insider Preview.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer on a Copilot+ PC or eligible Cloud PC; Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC. Not present on Windows 10 LTSC 2021.
- **Takes effect**: immediately; sign out and back in if an entry point is still visible. No Microsoft source documents a reboot, and the same setting is user-togglable in Settings without one.
- **Reverting**: System Default or Enabled restores the captured value from the snapshot.

#### Interactions
- [Disable Windows Recall snapshots](#disable-windows-recall-snapshots) and [Recall feature component](#recall-feature-component) share the `WindowsAI` key and cover Click to Do inside Recall, which this tweak does not.
- [Remove the Copilot app](#remove-the-copilot-app) does not touch `Microsoft.Windows.Ai.Copilot.Provider`, the component behind Click to Do and right-click AI actions.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The value, polarity and hive are correct. The research established that the real gate is Copilot+ hardware rather than a build number, that Microsoft's applicability column still says Windows Insider Preview, and that no reboot is documented; the shipped tweak carries `requires_reboot: false` and states the hardware and Insider caveats in its copy.
- **Confidence**: Microsoft-documented (Policy CSP and Manage Click to Do).
- **Reasoning**: The policy-hive audit confirmed the ADMX class is Both and that Computer Configuration wins, so the HKLM write is correct. The open question is retail-build behaviour, since Microsoft's table has not moved the policy out of Insider Preview.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a Copilot+ PC if you have no use for the overlay, and pair it with the Recall tweaks, because it does not cover Click to Do inside Recall. On other hardware it is harmless and changes nothing you can see.

#### Sources
1. Policy CSP - WindowsAI, `DisableClickToDo`: Default Value 0, value 1 disables, Device and User scope, ADMX key under `SOFTWARE\Policies\`, applicability column "Windows Insider Preview", editions Pro / Enterprise / Education / IoT Enterprise / IoT Enterprise LTSC, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
2. Manage Click to Do for Windows clients: entry points removed when enabled, on-device analysis, no effect on Click to Do inside Recall, Copilot+ and Cloud PC prerequisites, the Settings > Privacy & security > Click to Do per-user switch, https://learn.microsoft.com/en-us/windows/client-management/manage-click-to-do (tier A)
3. Shipped `WindowsCopilot.admx`: `DisableClickToDo` declared `class="Both"`, Computer Configuration wins (repository policy-hive audit, internal analysis)

### Remove the Copilot app

`remove_copilot_app` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: build >= 26100 · Reversible: yes

**Uninstalls the Copilot app so it stops appearing in Start, on the taskbar, and in your program list.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry (app state marker) | `HKCU\Software\MagicXToolbox\Debloat`, value `Copilot`, `REG_DWORD` |
| `app` | action (PowerShell, timeout 600 s) | apply: `Get-AppxPackage -AllUsers 'Microsoft.Copilot' \| Remove-AppxPackage -AllUsers`, then `Get-AppxProvisionedPackage -Online \| Where-Object { $_.PackageName -like 'Microsoft.Copilot*' } \| Remove-AppxProvisionedPackage -Online`; undo: `winget install --id 9NHT9RB2F4HD -e --accept-source-agreements --accept-package-agreements`; probe: exits 0 (removed) when neither an installed `Microsoft.Copilot` package for any user nor a provisioned `Microsoft.Copilot*` package exists, otherwise exits 1 |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run (apply the removal; probe must report removed) |
| Installed | `0` | not run (probe must report present, so selecting it from Removed runs the undo, a winget install) |

System Default: shown whenever the marker does not match an option, which includes every stock machine (the marker value does not exist until the app writes it) and any machine where the marker and the actual package state disagree. Selecting it restores the snapshot: the marker is put back as captured and the action's undo runs if the snapshot recorded the app as present. Stock Windows 11 24H2 and 25H2 ship the Copilot app installed and provisioned.

#### How it works

On Windows 11 24H2 and newer, Copilot is an ordinary Microsoft Store app with package family name `Microsoft.Copilot_8wekyb3d8bbwe`. The apply removes it two ways. `Remove-AppxPackage -AllUsers` uninstalls it for every existing account. `Remove-AppxProvisionedPackage -Online` removes the staged copy Windows installs into each new user profile; without that second step, the app returns for any new account and can be re-provisioned by feature updates. Microsoft's Manage Windows Copilot page documents removing `Microsoft.Copilot` as a supported route, and the Store product `9NHT9RB2F4HD` ("Microsoft Copilot on Windows") resolves to the same package family, so apply, probe and undo agree on one identity. `winget show --id 9NHT9RB2F4HD --source msstore` resolved in the research with publisher Microsoft Corporation, so the undo is executable.

The removal does not touch `Microsoft.Windows.Ai.Copilot.Provider`, the separate component behind Click to Do and right-click AI actions.

A second Store listing, `XP9CXNGPPJ97XX` "Microsoft Copilot", now exists. Microsoft's Store catalog returns it with no package family name at all, which indicates a non-Appx delivery path; this tweak targets only the Appx app and does not remove that one. Copilot's packaging has changed repeatedly (sidebar, then PWA, then a WebView2 build, then a native WinUI app, and in 2026 a WebView-based build that bundles its own copy of Edge), so the target needs re-checking with each release.

The marker value `HKCU\Software\MagicXToolbox\Debloat\Copilot` is the app's own record of which option was chosen; it makes the two options detectably distinct. Because it lives in HKCU, the app's different-account guard applies: if the app is elevated with another account's credentials, the tweak is disabled rather than writing the wrong user's hive.

On Enterprise, Education and IoT Enterprise running 24H2 or later, Microsoft's managed alternative is the `RemoveMicrosoftCopilotApp` policy, which only acts when both Copilot and Microsoft 365 Copilot are installed, the app was not user-installed, and it has not been launched in the last 28 days. The inbox-app removal policy `RemoveDefaultMicrosoftStorePackages` (ApplicationManagement CSP) also lists `Copilot`.

#### Benefits
- The app is uninstalled for all users, not just hidden.
- Removing the provisioned package stops it appearing for new accounts.
- Nothing left to launch or update in the background.
- A documented, Microsoft-supported removal route.

#### Drawbacks
- No Copilot assistant from Start, the taskbar or the app list.
- A feature update can re-provision the app, so it may need applying again.
- The second, non-Appx Store listing for Copilot is not covered.
- Copilot's packaging is a moving target.
- Reinstalling (the Installed option or a revert) needs winget and an internet connection; if the install cannot complete, the revert surfaces as Needs Attention.
- The probe treats "the package query returned nothing" as removed. The repository's probe-fail-open audit notes that `Get-AppxPackage -AllUsers` errors are non-terminating, so a failed query (for example when the probe runs without elevation) evaluates the same as "not installed" and reports Removed.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer (the app also exists on 23H2 / 22631, below this project's gate). The app is not present on Windows 10 LTSC 2021.
- **Takes effect**: immediately; no reboot.
- **Reverting**: System Default restores the captured marker and, when the snapshot recorded the app as present, runs the winget reinstall. The reinstall comes from the Microsoft Store, so it needs connectivity.

#### Interactions
- [Hide the Copilot taskbar button](#hide-the-copilot-taskbar-button) only hides the entry point; with this tweak applied it has nothing left to hide. Microsoft names removing `Microsoft.Copilot` as a supported replacement for that deprecated policy.
- [Disable the Click to Do overlay](#disable-the-click-to-do-overlay) governs the separate `Microsoft.Windows.Ai.Copilot.Provider` surface, which this tweak leaves alone.
- The same HKCU marker key (`HKCU\Software\MagicXToolbox\Debloat`) is used by [Disable the Recall optional feature](#disable-the-recall-optional-feature) and by debloat tweaks, each under its own value name; there is no overlap.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The package identity and undo are correct. The research established that removing only the installed package leaves the provisioned copy staged (so the app returns for new profiles and after feature updates) and that a probe checking only installed packages would report Removed while the provisioned copy remains; the shipped apply removes both and the shipped probe checks both.
- **Confidence**: Microsoft-documented (Manage Windows Copilot, Policy CSP, `Remove-AppxProvisionedPackage` reference), plus direct Store catalog lookups for both product ids.
- **Reasoning**: Store catalog lookups tied `9NHT9RB2F4HD` to `Microsoft.Copilot_8wekyb3d8bbwe`; the second listing's missing package family name is the basis for the non-Appx caveat. The 24H2 re-scope audit kept the tweak as the primary Copilot control. The probe-fail-open audit's finding (a failed `Get-AppxPackage` reads as removed) still applies to the shipped probe shape, which has no `$ErrorActionPreference = 'Stop'` / `catch` guard.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you do not use Copilot; it is the cleanest way to get rid of the app and costs nothing beyond losing the assistant. Leave it if you use Copilot at all, since getting it back depends on the Store being reachable. Re-check it after each feature update.

#### Sources
1. Microsoft Store catalog, product `9NHT9RB2F4HD`: title "Microsoft Copilot on Windows", package family name `Microsoft.Copilot_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NHT9RB2F4HD?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy CSP - WindowsAI, `RemoveMicrosoftCopilotApp` (managed alternative and its preconditions) and `TurnOffWindowsCopilot` (deprecated), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
3. Manage Windows Copilot: the 24H2 Store-app model and removing `Microsoft.Copilot` as a supported route, https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot (tier A)
4. `Remove-AppxProvisionedPackage` cmdlet reference, https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)
5. Microsoft Store catalog, product `XP9CXNGPPJ97XX` "Microsoft Copilot": no package family name returned (tier A, primary observation)
6. Policy-based inbox app removal, `RemoveDefaultMicrosoftStorePackages`, `Copilot` in the supported list, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (from the repository's 24H2 re-scope audit)
7. "New Copilot for Windows 11 includes a full Microsoft Edge package", Windows Latest, packaging history only, https://www.windowslatest.com/2026/04/05/new-copilot-for-windows-11-includes-a-full-microsoft-edge-package-uses-more-ram/ (tier D)
8. Repository probe-fail-open audit: non-terminating `Get-AppxPackage` errors make this probe shape report Removed when the query fails (internal analysis)

### Disable the Recall optional feature

`remove_recall_feature` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: build >= 26100 · Reversible: yes

**Turns off the Recall optional Windows feature at the component level, below the Settings toggle.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry (app state marker) | `HKCU\Software\MagicXToolbox\Debloat`, value `RecallFeature`, `REG_DWORD` |
| `feature` | action (PowerShell, timeout 900 s) | apply: reads the current state of optional feature `Recall` (`Unknown` if unreadable), stores it as `REG_SZ` `PreApplyState` under `HKLM\SOFTWARE\MagicXToolbox\RecallFeature`, then runs `DISM.exe /Online /Disable-Feature /FeatureName:Recall /NoRestart`, treating exit codes 0 and 3010 as success; undo: exits 0 without doing anything unless `PreApplyState` is `Enabled`, otherwise runs `DISM.exe /Online /Enable-Feature /FeatureName:Recall /NoRestart` (0 and 3010 are success); probe: exits 0 (applied) when the feature state is `Disabled` or `DisabledWithPayloadRemoved`, otherwise 1 |

| Option | `state` | `feature` |
|---|---|---|
| Disabled | `1` | run (probe must report disabled) |
| Enabled | `0` | not run (probe must report not disabled, so selecting it from Disabled runs the undo) |

System Default: shown whenever the marker does not match an option, which includes every stock machine (no marker exists until the app writes it). Selecting it restores the snapshot, running the undo when needed. On stock Windows 11 24H2 x64 the `Recall` feature entry is enumerated on every install; on a machine that is not a Copilot+ PC with Recall on, its state is typically `Disabled` or `DisabledWithPayloadRemoved` (observed on IoT Enterprise LTSC 2024, 26100.4061).

#### How it works

Windows 11 24H2 enumerates an optional feature literally named `Recall` (`Get-WindowsOptionalFeature -Online`). The entry exists on ordinary x64 installs, including SKUs where the Recall experience is never offered; what is Copilot+ gated is the experience, not the entry. `DISM /Disable-Feature` turns the feature off at the servicing level, below the user-facing Settings toggle, so paths that only flip that setting cannot bring Recall back.

Without `/Remove`, DISM leaves the payload on disk; the tweak does not pass `/Remove`, so the result is state `Disabled`, not `DisabledWithPayloadRemoved`. The probe accepts both, because both mean "off", and a machine that already has the payload removed is more thoroughly disabled than this tweak would make it.

With `/NoRestart`, DISM returns 3010 (`ERROR_SUCCESS_REBOOT_REQUIRED`) when the change succeeded and needs a restart, which is the normal result for disabling a feature; the script maps it to exit 0 so the app's exit-code contract (0 is success) reads it correctly.

The undo is conditional. Apply records the feature's state before it ran in `HKLM\SOFTWARE\MagicXToolbox\RecallFeature\PreApplyState`, and undo only re-enables Recall when that recorded state is `Enabled`. That stops a revert from installing Recall on a machine that never had it; an unconditional enable would also fail with 0x800F081F where the payload is not staged, because no `/Source` is given. If the payload has ever been removed, re-enabling needs a DISM `/Source`.

The marker `HKCU\Software\MagicXToolbox\Debloat\RecallFeature` records the chosen option. It lives in HKCU, so the different-account guard applies. The `PreApplyState` value is written by the script, not owned as a tweak effect, and is not removed by the undo.

#### Benefits
- Disables the feature itself, not the setting on top of it.
- A Settings flip alone cannot bring Recall back.
- Complements the Recall snapshot and component policies.
- Revert only re-enables Recall where it was enabled before.
- Works on any edition where the feature entry exists; it does not depend on policy support.

#### Drawbacks
- The payload stays on disk, only inactive.
- The feature entry exists everywhere, but the Recall experience needs Copilot+ hardware, so on most PCs applying changes nothing visible.
- Revert is conditional: on a machine where Recall was already disabled, reverting correctly does nothing.
- A reboot is needed to complete the change.
- DISM can take minutes; the action allows up to 900 seconds.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, x64; the feature entry is present even on SKUs that never offer Recall. Not present on Windows 10 LTSC 2021.
- **Takes effect**: after a reboot; DISM reports a pending restart on success.
- **Reverting**: System Default restores the snapshot. The undo re-enables the feature only if the recorded pre-apply state was `Enabled`; otherwise it leaves the feature disabled. Choosing the Enabled option directly on a machine where Recall was never enabled runs that same no-op undo, so the feature stays disabled and the probe cannot confirm the Enabled state; use System Default rather than Enabled on such a machine.

#### Interactions
- [Recall feature component](#recall-feature-component) with Removed also removes the Recall bits through policy; applying both is redundant but consistent. Bringing Recall back after both may need this tweak reverted (on a machine where it was enabled) plus the component policy set to Available.
- [Disable Windows Recall snapshots](#disable-windows-recall-snapshots) is the documented, reversible alternative that stops capture without touching the component.
- Shares the HKCU marker key with [Remove the Copilot app](#remove-the-copilot-app) under a different value name.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The research required that the probe accept both `Disabled` and `DisabledWithPayloadRemoved`, that DISM exit 3010 count as success, that the undo branch on the recorded pre-apply state, and that the copy not claim the payload is removed. The shipped action meets all four. The research did not re-verify the shipped script.
- **Confidence**: Microsoft-documented for DISM semantics (`/Disable-Feature`, `/Enable-Feature`, `/Remove`, `/Source`, `/NoRestart`, feature states), plus direct feature enumeration on 26100.4061.
- **Reasoning**: The feature name `Recall` was confirmed by enumeration; the state `DisabledWithPayloadRemoved` was observed on a non-Copilot+ SKU; the 3010 behaviour is standard DISM with `/NoRestart`. The probe-fail-open audit lists this tweak among the probes with safe polarity: an unreadable feature state leaves `$s` empty and the probe exits 1 (not applied) rather than reporting success.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Use it on a Copilot+ PC where Recall is present and you want it off at the component level. On any other machine prefer the [Recall snapshot policy](#disable-windows-recall-snapshots), which is documented, reversible and does not depend on optional-feature state.

#### Sources
1. `Get-WindowsOptionalFeature -Online` on Windows 11 26100.4061 IoT Enterprise LTSC: FeatureName `Recall`, State `DisabledWithPayloadRemoved` (tier A, primary observation)
2. `Disable-WindowsOptionalFeature` cmdlet reference: feature states and `-Remove` semantics, https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature (tier A)
3. DISM operating system package servicing command-line options: `/Disable-Feature`, `/Enable-Feature`, `/Remove`, `/Source`, `/NoRestart`, https://learn.microsoft.com/en-us/windows/deployment/usmt/dism-operating-system-package-servicing-command-line-options (tier A)
4. Policy CSP - WindowsAI, `DisableAIDataAnalysis` as the documented reversible alternative, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
5. microsoft/winget-cli issue 2707: DISM exit code 3010 with `/NoRestart`, https://github.com/microsoft/winget-cli/issues/2707 (tier C)
6. "Completely uninstall Recall feature on Windows 11", Pureinfotech, DISM command shape only, https://pureinfotech.com/uninstall-recall-windows-11/ (tier D)

### Hide the Copilot taskbar button

`disable_copilot_taskbar` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Removes the Copilot button from your taskbar without touching anything else.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `copilot_off` | registry | `HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot`, value `TurnOffWindowsCopilot`, `REG_DWORD` |

| Option | `copilot_off` |
|---|---|
| Hidden | `1` |
| Shown | `absent` (value deleted) |

System Default: shown when the value holds anything other than 1 or absent (for example 0, the ADMX's disabled value); selecting it restores the snapshot. Stock Windows has no value, which reads as Shown.

#### How it works

This is the user-scope Group Policy "Turn off Windows Copilot", Policy CSP `WindowsAI/TurnOffWindowsCopilot`. The shipped `WindowsCopilot.admx` on 26100.4061 declares it `class="User"`, key `SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot`, value `TurnOffWindowsCopilot`, enabled 1 and disabled 0. The Policy CSP marks Device scope not applicable, so there is no HKLM equivalent and the per-user HKCU write is the only correct one. Microsoft's documented effect is that the Copilot icon does not appear on the taskbar. The shell still reads the value on 26100: the string is present in `Taskbar.dll`, `CustomShellHost.exe`, `ShellAppRuntime.exe`, `TransmogProvider.dll` and `assignedaccessmanagersvc.dll`.

Since the September and October 2024 updates, Copilot on 24H2 and newer is an ordinary Store app rather than the old sidebar, so this policy hides the shell entry point and leaves the app installed. Win+C and the Copilot hardware key now open the Microsoft 365 Copilot prompt box (from the May 2025 optional preview onward), a different surface this policy does not govern.

Microsoft has deprecated the policy: Manage Windows Copilot says AppLocker "should be used instead of the Turn Off Windows Copilot legacy policy setting and its MDM equivalent, TurnOffWindowsCopilot. The policy is subject to near-term deprecation", and the Policy CSP page says "This policy is deprecated and may be removed in a future release". Deprecated is not removed; the ADMX still ships and the shell still reads the value on 26100. The supported replacements are an AppLocker publisher rule for `MICROSOFT.COPILOT` or removing the `Microsoft.Copilot` app.

Applicability per the Policy CSP: Windows 10 21H2 (19044.3758) and later, Windows 10 22H2 with KB5032278, Windows 11 22H2 with KB5030310, and Windows 11 23H2 and later. This is one of the two tweaks in the category that apply to Windows 10 IoT Enterprise LTSC 2021 (the other is the Edge policies tweak), given build 19044.3758 or newer. Editions: Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC; the ADMX `supportedOn` is `SUPPORTED_Windows_11_0_NOSERVER_ENTERPRISE_EDUCATION_PRO_SANDBOX`, so Home is excluded and the write is expected to be inert there.

Because the value is in HKCU, it applies to the current user only and is subject to the app's different-account guard. It still needs administrator rights: `HKCU\Software\Policies` grants the user read access only (SYSTEM and Administrators have full control), so an unelevated write is refused.

#### Benefits
- Cleaner taskbar with the Copilot button gone.
- Removing the value restores the button exactly.

#### Drawbacks
- Copilot is still installed and can be opened from Search, Start or Edge.
- Deprecated by Microsoft and subject to near-term removal.
- Inert on Home.
- Does not govern Win+C or the Copilot hardware key.
- Per-user only: other accounts on the PC keep the button.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 (with KB5030310) and newer, including 24H2 and 25H2; Windows 10 21H2 build 19044.3758 and newer (so Windows 10 IoT Enterprise LTSC 2021 on a current cumulative update); Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after signing out and back in (the tweak does not flag a reboot for this per-user cosmetic change).
- **Reverting**: System Default or Shown restores the captured value.

#### Interactions
- [Remove the Copilot app](#remove-the-copilot-app) is the primary Copilot control on 24H2 and newer; this tweak is the cosmetic supplement.
- The debloat tweak `disable_web_search_start` removes the Copilot entry in the Start search flyout, a different surface.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The registry effect is correct exactly as authored (key, value, type, polarity, HKCU scope, absent default). The research established that the policy does not govern Win+C, is a no-op on Home, and is deprecated; the shipped copy states all three and makes no Win+C claim.
- **Confidence**: Microsoft-documented, with the shipped ADMX and a binary string check on 26100.4061.
- **Reasoning**: The policy-hive audit confirmed `class="User"` and HKCU. The 24H2 re-scope audit found the policy still read on current builds despite the deprecation banner and recommended keeping it as a supplement to app removal. The open question is timing: Microsoft gives no date for removal.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you just want the button out of the way; it costs nothing and is instantly undone. If you want Copilot actually gone on 24H2 or newer, use [Remove the Copilot app](#remove-the-copilot-app) instead. Expect this one to stop working in a future release.

#### Sources
1. Shipped `WindowsCopilot.admx` on Windows 11 26100.4061: `TurnOffWindowsCopilot`, `class="User"`, key `SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot`, enabled 1 / disabled 0, `supportedOn` `SUPPORTED_Windows_11_0_NOSERVER_ENTERPRISE_EDUCATION_PRO_SANDBOX` (tier A)
2. Policy CSP - WindowsAI, `TurnOffWindowsCopilot`: deprecation note, User-only scope, editions, applicable OS list including Windows 10 21H2 19044.3758, "The Copilot icon won't appear on the taskbar either", https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
3. Manage Windows Copilot: AppLocker replacement and near-term deprecation, the 24H2 Store-app model, the AppLocker publisher rule for `MICROSOFT.COPILOT`, the `Remove-AppxPackage` route and the May 2025 Win+C prompt-box behaviour, https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot (tier A)
4. Binary inspection on Windows 11 26100.4061: `TurnOffWindowsCopilot` as a UTF-16 string in `Taskbar.dll`, `CustomShellHost.exe`, `ShellAppRuntime.exe`, `TransmogProvider.dll` and `assignedaccessmanagersvc.dll` (tier A, primary measurement)

### Disable Notepad AI features

`disable_notepad_ai` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Turns off Copilot, Rewrite and Summarize inside Notepad, leaving a plain text editor.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `notepad_ai` | registry | `HKLM\SOFTWARE\Policies\WindowsNotepad`, value `DisableAIFeatures`, `REG_DWORD` |

| Option | `notepad_ai` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` (value deleted) |

System Default: shown when the value holds anything other than 1 or absent (for example 0); selecting it restores the snapshot. Stock Windows has no value, which reads as Enabled.

#### How it works

The Store version of Notepad checks `DisableAIFeatures` under `HKLM\SOFTWARE\Policies\WindowsNotepad` and, when it is 1, removes its AI surfaces (Copilot, Rewrite and Summarize). The key path has no `Microsoft` component; that is correct. Inside `Notepad.exe` 11.2604.5.0 the strings `SOFTWARE\Policies\WindowsNotepad` and `DisableAIFeatures` are adjacent literals in one string block, followed by the `0` and `1` literals, and Microsoft Learn says: "To disable AI features in Notepad, set the DisableAIFeatures registry value to 1 under `HKLM:\SOFTWARE\Policies\WindowsNotepad`."

One value covers every AI surface. `Notepad.exe` contains no `DisableRewrite`, `DisableSummarize` or `DisableCopilot` strings, and Learn lists exactly one policy.

The Group Policy setting is named `DisableAIFeaturesInNotepad`, but the registry value Notepad reads is `DisableAIFeatures`; writing the policy name as a value would be silently ignored. `WindowsNotepad.admx` ships out of band in `WindowsNotepadAdminTemplates.cab`, not in the inbox ADMX set, so no ADMX on 26100 contains the value. Writing the registry value directly works whether or not the ADMX is installed.

Notepad updates through the Microsoft Store independently of Windows. The value is honoured from Notepad 11.2503.16.0; a fully patched 26100 or 26200 machine can still carry an older Notepad that ignores it. Windows 10 LTSC 2021 ships the classic Win32 Notepad with no AI surface, and the tweak is gated to Windows 11.

#### Benefits
- Copilot, Rewrite and Summarize disappear from Notepad's menus and toolbar.
- Machine-wide: every account on the PC.
- Removes the entry points that ask you to sign in to use AI.

#### Drawbacks
- All or nothing: there is no per-feature switch.
- Notepad versions before 11.2503.16.0 ignore the value.
- Notepad only; other apps' AI features are unaffected.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 and newer with Notepad 11.2503.16.0 or later.
- **Takes effect**: the next time Notepad starts.
- **Reverting**: System Default or Enabled restores the captured value.

#### Interactions
None known. No other tweak writes under `HKLM\SOFTWARE\Policies\WindowsNotepad`.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the unusual key path was attacked and confirmed.
- **Confidence**: Microsoft-documented, and confirmed by the literal strings in the shipped Notepad binary.
- **Reasoning**: The adversarial pass attacked the key path (no `Microsoft` component), looked for per-feature values, checked the ADMX name against the registry value name, and explained the ADMX's absence from `PolicyDefinitions`. Every attack resolved in favour of the shipped value.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want Notepad to stay a fast, plain text editor. Skip it if you use Rewrite or Summarize, since the switch takes all of them.

#### Sources
1. Manage Notepad: "set the DisableAIFeatures registry value to 1 under `HKLM:\SOFTWARE\Policies\WindowsNotepad`", the app version floor and the single-policy list, https://learn.microsoft.com/en-us/windows/client-management/manage-notepad (tier A)
2. Strings in `Notepad.exe` 11.2604.5.0 (`Microsoft.WindowsNotepad_11.2604.5.0_x64__8wekyb3d8bbwe`): key and value adjacent, no per-feature value names (tier A, product artifact)
3. Search of `C:\Windows\PolicyDefinitions` on 26100: no inbox ADMX contains `DisableAIFeatures`, consistent with `WindowsNotepad.admx` shipping in `WindowsNotepadAdminTemplates.cab` (tier A, primary observation)
4. Policy CSP - WindowsAI (cited by the tweak's evidence list), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai

### Disable Paint AI features

`disable_paint_ai` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: build >= 22621 · Reversible: yes

**Removes Cocreator, Generative fill and Image Creator from Paint, leaving the classic drawing tools.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `paint_cocreator` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint`, value `DisableCocreator`, `REG_DWORD` |
| `paint_generative_fill` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint`, value `DisableGenerativeFill`, `REG_DWORD` |
| `paint_image_creator` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint`, value `DisableImageCreator`, `REG_DWORD` |

| Option | `paint_cocreator` | `paint_generative_fill` | `paint_image_creator` |
|---|---|---|---|
| Disabled | `1` | `1` | `1` |
| Enabled | `absent` | `absent` | `absent` |

System Default: shown when the three values do not all match one option (for example only one of them set, or any set to 0); selecting it restores the snapshot. Stock Windows has none of the values, which reads as Enabled (documented Default Value 0 for each).

#### How it works

These are three Microsoft policies shipped in `WindowsCopilot.admx` on 26100, each `class="Machine"`, each on `Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, each with `enabledValue` 1 and `disabledValue` 0, each `supportedOn` `SUPPORTED_Windows_11_0_22H2`. Policy CSP - WindowsAI gives the same key, value names, ADMX file, Format `int`, Default Value 0, and allowed values 0 (enabled, default) and 1 (disabled), with Device scope only (User explicitly not supported). Paint reads them and hides the matching AI tool when a value is 1. The ADML confirms the polarity: "If this policy is enabled, Cocreator functionality will not be accessible in the Paint app."

The key is under `CurrentVersion\Policies`, not `SOFTWARE\Policies`; that is where Microsoft defines these policies, and both the ADMX and the CSP agree on it.

The Policy CSP lists applicability as Windows 11 22H2 (10.0.22621.4870) and later, and Windows 11 24H2 (10.0.26100.3360) and later. The tweak's gate is `build >= 22621`, so on 26100 builds below revision 3360 the values may be ignored. Editions: Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC. Windows 10 is outside the `supportedOn` range and outside the gate.

#### Benefits
- The AI buttons leave Paint's toolbar.
- Machine-wide; Microsoft documents these as device-scope policies.
- Removes the surfaces that ask you to sign in or spend AI credits.

#### Drawbacks
- Cocreator, Generative fill and Image Creator all go together.
- Paint only; image generation elsewhere in Windows is unaffected.
- Not available on Windows 10.
- Not listed for Home in Microsoft's edition table.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 (22621.4870) and newer, and 24H2 from 26100.3360; Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC.
- **Takes effect**: the next time Paint starts.
- **Reverting**: System Default or Enabled restores the three captured values.

#### Interactions
None known. No other tweak writes under the Paint policy key.

#### Validation
- **Verdict**: VERIFIED. No correction needed; the research also upheld leaving out two values that circulate online (see [Considered and not shipped](#considered-and-not-shipped)).
- **Confidence**: Microsoft-documented, from two independent tier A sources (the shipped ADMX/ADML and the Policy CSP) that agree on key, names, type and polarity.
- **Reasoning**: The adversarial pass attacked existence, polarity (the `Disable*` naming is not inverted: 1 disables in both sources) and duplication. One verification limit: the value names were not found in any binary under `C:\Program Files\WindowsApps` on the inspection machine because Paint was not installed there; the ADMX plus CSP pair makes that immaterial.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use Paint for quick drawing and cropping and have no interest in image generation. Skip it if you use Cocreator or Generative fill, since all three tools are controlled together.

#### Sources
1. Shipped `WindowsCopilot.admx` and `en-US\WindowsCopilot.adml` on 26100: the three policies, `class="Machine"`, key, enabled 1 / disabled 0, `supportedOn` `SUPPORTED_Windows_11_0_22H2` (tier A)
2. Policy CSP - WindowsAI: per-value registry key and value name, ADMX file, Format `int`, Default 0, allowed values, Device scope only, applicability 22621.4870 and 26100.3360, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
3. Manage Paint (cited by the tweak's evidence list), https://learn.microsoft.com/en-us/windows/client-management/manage-paint
4. Search of all ADMX on 26100 and the complete WindowsAI CSP policy list for `DisableGenerativeErase` and `DisableRemoveBackground`: absent from both (tier A, primary observation)

### Turn off Edge AI features

`disable_edge_ai_features` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Strips Copilot and Bing Chat entry points out of Microsoft Edge and stops AI reading your pages and history.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `edge_history_ai_search` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `EdgeHistoryAISearchEnabled`, `REG_DWORD` |
| `edge_ntp_bing_chat` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `NewTabPageBingChatEnabled`, `REG_DWORD` |
| `edge_copilot_page_context` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `CopilotPageContext`, `REG_DWORD` |
| `edge_entra_copilot_page_context` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `EdgeEntraCopilotPageContext`, `REG_DWORD` |
| `edge_compose_inline` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `ComposeInlineEnabled`, `REG_DWORD` |

| Option | `edge_history_ai_search` | `edge_ntp_bing_chat` | `edge_copilot_page_context` | `edge_entra_copilot_page_context` | `edge_compose_inline` |
|---|---|---|---|---|---|
| Off (three values are work-account only) | `0` | `0` | `0` | `0` | `0` |
| On | `absent` | `absent` | `absent` | `absent` | `absent` |

System Default: shown when the five values do not all match one option (for example an organisation's policy sets only some of them, or sets one to 1); selecting it restores the snapshot. Stock Windows has none of these values, which reads as On (Edge's own defaults apply).

#### How it works

Microsoft Edge reads mandatory policies from `HKLM\SOFTWARE\Policies\Microsoft\Edge` (each policy page lists `Path (Mandatory): SOFTWARE\Policies\Microsoft\Edge`). All five values exist, are `REG_DWORD`, and appear in none of Edge's deprecated or obsolete tables. What 0 does, and the minimum Edge version for each:

| Value | 0 means | Minimum Edge | Profiles |
|---|---|---|---|
| `EdgeHistoryAISearchEnabled` | AI-powered history search off; history search returns to exact matching | 138 | all |
| `NewTabPageBingChatEnabled` | all Bing Chat entry points removed from the new tab page | 117 | all |
| `CopilotPageContext` | Copilot cannot read page content | 124 | Microsoft Entra ID only |
| `EdgeEntraCopilotPageContext` | Copilot cannot read page content or history | 130 | Microsoft Entra ID only |
| `ComposeInlineEnabled` | inline Rewrite unavailable | 115 | Microsoft Entra ID only |

Microsoft states the Entra restriction in the policy text: on `CopilotPageContext`, "This policy applies only to Microsoft Entra ID profiles in Microsoft Edge. It doesn't apply to Microsoft account (MSA) profiles"; on `ComposeInlineEnabled`, "This policy applies only to Microsoft Entra accounts and doesn't apply to Microsoft accounts"; and `EdgeEntraCopilotPageContext` lists "Applies to a profile that is signed in with a Microsoft account: No". On a consumer profile these three writes succeed, the app reports the tweak applied, and Edge ignores them. The option label says so, because the app cannot detect it: it verifies the registry, not Edge's interpretation of it.

Edge updates independently of Windows, so the Edge version floors cannot be expressed as a `windows:` gate; an older Edge accepts the write and ignores the values it does not know. The same independence makes this tweak usable on Windows 10 IoT Enterprise LTSC 2021 wherever a current Edge is installed, which is why it has no Windows gate.

Setting any Edge policy makes Edge show "managed by your organization" on its settings page. These policies do not remove Copilot's own toolbar button; Microsoft's `EdgeCopilotEnabled` policy is not supported on Windows.

#### Benefits
- The Bing Chat entry points leave the new tab page.
- History search goes back to plain exact matching, without AI.
- On Entra (work or school) profiles, Copilot loses access to page content and history and inline Rewrite goes away.
- Machine-wide: every Edge profile on the PC.

#### Drawbacks
- Three of the five values do nothing on a personal Microsoft account profile, so on a consumer PC only the history-search and new-tab values change anything.
- Edge shows "managed by your organization".
- Older Edge builds ignore some values; `EdgeHistoryAISearchEnabled` needs Edge 138.
- AI history search and inline Rewrite are lost.
- The Copilot toolbar button stays.

#### Applies to, takes effect, reverting
- **Applies to**: any supported Windows with Microsoft Edge installed, including Windows 10 IoT Enterprise LTSC 2021, subject to the per-value Edge minimums above and to the Entra-only restriction on three values.
- **Takes effect**: after restarting Edge.
- **Reverting**: System Default or On restores the five captured values; the managed banner clears once no Edge policies remain (other Edge tweaks in the app also write to this key).

#### Interactions
- Other tweaks write different values under the same key: `disable_edge_first_run` (`HideFirstRunExperience`), `disable_edge_startup_boost` (`StartupBoostEnabled`, `BackgroundModeEnabled`), `disable_edge_sidebar` (`HubsSidebarEnabled`, `EdgeCollectionsEnabled`), `disable_edge_telemetry` (`PersonalizationReportingEnabled`, `Edge3PSerpTelemetryEnabled`) and the SmartScreen tweak (`SmartScreenEnabled`). None collides with these five; any of them keeps the managed banner visible.
- `disable_edge_sidebar` removes only the sidebar panel, not these AI features; its copy notes the Copilot toolbar button needs `Microsoft365CopilotChatIconEnabled`, which no tweak writes.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. All five values, the key and the type are correct. The research established that `CopilotPageContext`, `EdgeEntraCopilotPageContext` and `ComposeInlineEnabled` apply only to Microsoft Entra ID profiles, that each value has its own Edge version floor, and that `EdgeCopilotEnabled` must not be written on Windows; the shipped tweak names the Entra limit in its option label and copy, lists the floors, and does not write `EdgeCopilotEnabled`.
- **Confidence**: Microsoft-documented (the Edge policy reference for each value).
- **Reasoning**: The research weighed two shapes: ship only the two consumer-effective values, or ship all five with the Entra limit stated. The shipped tweak takes the second, on the grounds that a real control exists for Entra users and the corpus handles Copilot+-gated tweaks the same way (honest copy rather than removal). The open question the research left is whether an Edge version check is needed; the tweak has none, so an old Edge silently ignores newer values.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want Edge to stop offering AI in the browsing surface, and expect a visible change on a personal account only from the history search and new tab page values. On a work or school PC with Entra profiles all five take effect, and this is the strongest in-browser AI control available here.

#### Sources
1. Microsoft Edge Browser Policy Documentation (index; `CopilotCDPPageContext` marked obsolete), https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies (tier A)
2. `EdgeHistoryAISearchEnabled`: Edge 138+, 0 gives exact-match history search, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgehistoryaisearchenabled (tier A)
3. `NewTabPageBingChatEnabled`: Edge 117+, 0 removes all Bing Chat entry points from the new tab page, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#newtabpagebingchatenabled (tier A)
4. `CopilotPageContext`: Edge 124+, Entra ID profiles only, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#copilotpagecontext (tier A)
5. `EdgeEntraCopilotPageContext`: Edge 130+, not applicable to Microsoft account profiles, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgeentracopilotpagecontext (tier A)
6. `ComposeInlineEnabled`: Edge 115+, Entra accounts only, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#composeinlineenabled (tier A)
7. `EdgeCopilotEnabled`: Windows not supported, no Windows registry section, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgecopilotenabled (tier A)

### Set the AI Fabric service to Manual start

`disable_ai_fabric_service` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: build >= 26100 (service present from about 26100.3624) · Reversible: yes

**Stops the Windows AI broker service from starting with the machine, leaving it available on demand.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry (app state marker) | `HKCU\Software\MagicXToolbox\State`, value `AiFabricService`, `REG_DWORD` |
| `ai_fabric` | service start type | `WSAIFabricSvc`; `optional: true` (no `if_missing`) |

| Option | `state` | `ai_fabric` |
|---|---|---|
| Manual | `1` | `manual` (Start = 3) |
| Automatic | `0` | `automatic` (Start = 2) |

System Default: shown whenever the marker does not match the service state, which includes every stock machine (Automatic, but no marker yet); selecting it restores the snapshot. Stock Windows 11 24H2 and 25H2 ship `WSAIFabricSvc` as plain Automatic: not delayed-start and not trigger-started.

#### How it works

`WSAIFabricSvc` is a shared svchost service (`svchost.exe -k WSAIFabricSvcGroup -p`, type 0x20) running as `NT AUTHORITY\LocalService`, depending on `RPCSS` and `BrokerInfrastructure`, hosted in `C:\Windows\System32\WSAIFabricHost.dll`. Its display name resolves to the literal string `WSAIFabricSvc`, and its description to "Provides support to communicate with AIFabric in Local service context over COM." It is the COM broker that Windows AI workloads talk to. Setting it to Manual stops the boot-time start; because it is a demand-startable COM broker, anything that genuinely needs it can still start it. Manual rather than Disabled is intentional: Disabled would block callers outright, and 25H2 users report the service re-enabling itself after reboot when disabled.

The service arrived in 24H2 servicing: `WSAIFabricHost.dll` on 26100.4061 carries FileVersion 10.0.26100.3624 (May 2025), and a Win11Debloat issue from June 2025, before 25H2 shipped, reports it on 24H2. 26100 builds below roughly 26100.3624 do not have it, and the exact first revision is unknown, so the service effect is `optional: true`: on a machine without it the service reads Missing, both options are shown as unavailable, and nothing is written. The engine never installs or removes services.

The stock start type is Automatic, from three lines of evidence independent of any single machine's configuration: Win11Debloat's undo file restores `Start` = 2, a Microsoft Q&A 25H2 thread recommends `Set-Service WSAIFabricSvc -StartupType Manual` to stop it "starting automatically after reboot", and 25H2 users report it re-enabling after being disabled. `sc qtriggerinfo` shows no triggers and there is no `DelayedAutostart` value, so the Automatic option is a plain Automatic.

The memory issue that often motivates this tweak is hardware-specific. On NPU-equipped and Copilot+ PCs, reports show `WorkloadsSessionHost` processes holding gigabytes (a Q&A thread on 26200.8894 measured 8 instances at 2,449 MB private and 4,454 MB working set, dropping to zero after stopping the service; ElevenForum reports 5 to 8 GB alongside an AMD NPU interaction). On a non-NPU 24H2 machine, no `WorkloadsSessionHost*` file exists anywhere under `C:\Windows`, none runs even while the service runs, and `WSAIFabricHost.dll` contains no such strings. On ordinary hardware this tweak only keeps an idle broker out of startup.

The marker in HKCU records the chosen option and is subject to the app's different-account guard.

#### Benefits
- One fewer service started at boot.
- On Copilot+ and NPU machines, avoids the AI workload hosts that have been measured holding several gigabytes, as long as nothing starts the service on demand.
- Manual keeps the service startable, so nothing that needs it is blocked.

#### Drawbacks
- No measurable memory or CPU saving on PCs without an NPU.
- Using a Windows AI feature can start the service on demand anyway.
- Unavailable on 24H2 builds below roughly 26100.3624, where the service does not exist.
- Changing the start type does not stop the running service; the effect arrives at the next boot.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 from about build 26100.3624, and 25H2. The service does not exist on Windows 10 (the tweak is hidden there by its build gate).
- **Takes effect**: after a reboot, since only the start type changes.
- **Reverting**: System Default restores the captured start type (Automatic on a stock machine) and the captured marker.

#### Interactions
None known. `WSAIFabricSvc` appears in no other tweak, including the services category.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The service and the Manual value are correct. The research established that the service exists on 24H2 (from about 26100.3624), not only on 25H2, so the gate is the 26100 line plus a clean skip where it is absent; that its display name is literally `WSAIFabricSvc`; and that the multi-gigabyte memory benefit applies only to NPU and Copilot+ hardware. The shipped tweak uses a `>= 26100` gate with an optional service effect and scopes the memory claim to NPU PCs.
- **Confidence**: Microsoft-documented for service start-type semantics and from shipped product artifacts (DLL version resource, MUI strings, service registration); the Automatic default is community-corroborated.
- **Reasoning**: The adversarial pass refuted a 25H2-only gate with the shipped 26100 DLL, refuted the universal memory claim with a negative binary search, and rejected two service catalogs (`batcmd.com`, `revertservice.com`) that give Manual as the default: they publish the identical wrong value, neither tracks 25H2, and every other line of evidence contradicts them. Open questions: the exact first 26100 revision with the service, and no clean 26200 image was inspected for the default (it is strongly indicated by the 25H2 reports).
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying on a Copilot+ or NPU-equipped PC where AI workload hosts are consuming real memory. On an ordinary PC it is a harmless way to keep an unused AI broker out of startup, but do not expect a difference in Task Manager.

#### Sources
1. `C:\Windows\System32\WSAIFabricHost.dll` version resource on 26100.4061: FileVersion 10.0.26100.3624, FileDescription `WSAIFabricSvc`, May 2025 (tier A, product artifact)
2. `C:\Windows\System32\en-US\WSAIFabricHost.dll.mui` string table: display name `WSAIFabricSvc` and the description (tier A, product artifact)
3. Service registration under `HKLM\SYSTEM\CurrentControlSet\Services\WSAIFabricSvc` on 26100.4061: ImagePath, ObjectName, Type, DependOnService (tier A, primary observation; not used as evidence of the default start type)
4. `sc qtriggerinfo WSAIFabricSvc` (no triggers) and no `DelayedAutostart` value (tier A, primary observation)
5. Negative search for `WorkloadsSessionHost*` under `C:\Windows` and for related strings inside `WSAIFabricHost.dll` (tier A, primary observation)
6. Win11Debloat `Regfiles/Disable_AI_Service_Auto_Start.reg` (Start 3) and `Regfiles/Undo/Enable_AI_Service_Auto_Start.reg` (Start 2), https://github.com/Raphire/Win11Debloat (tier C)
7. Win11Debloat issue 265 (June 2025, service on 24H2, Manual rather than Disabled) and issue 497, https://github.com/Raphire/Win11Debloat/issues/265 (tier C)
8. Microsoft Q&A question 5955428 on 26200.8894: memory measurement and the `Set-Service ... -StartupType Manual` remedy (tier C)
9. ElevenForum thread 45675 (via the Wayback Machine): 5 to 8 GB on NPU hardware, service re-enabling itself when disabled (tier C)
10. microsoft/onnxruntime-genai issue 2013, AMD NPU interaction, https://github.com/microsoft/onnxruntime-genai/issues/2013 (tier C)
11. Set-Service, https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/set-service (cited by the tweak's evidence list)
12. Service start types, https://learn.microsoft.com/en-us/windows/win32/services/service-installation-and-configuration (cited by the tweak's evidence list)

## Considered and not shipped

These are individual values the research examined for tweaks in this category and rejected; none is written by any tweak.

- **`EdgeCopilotEnabled`** (proposed as an extra value for [Turn off Edge AI features](#turn-off-edge-ai-features), to remove Copilot in Edge entirely). Microsoft's policy page lists Windows: Not supported and macOS: Not supported, with Android and iOS 123+ only, and has no Windows registry section, so writing it on Windows does nothing. Source: https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgecopilotenabled (tier A).
- **`CopilotCDPPageContext`** (Edge). Marked "(obsolete)" on the Edge policy index and superseded by `EdgeEntraCopilotPageContext`, which the Edge tweak writes. Source: https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies (tier A).
- **`DisableGenerativeErase` and `DisableRemoveBackground`** (Paint, circulating in Win11Debloat). Present in no ADMX on 26100 and in no Microsoft Learn page, including the complete WindowsAI CSP policy list; single-source, so not real policies as far as the evidence shows. [Disable Paint AI features](#disable-paint-ai-features) writes only the three documented values.
