# Performance & Gaming tweak validation

Rebuilt 2026-07-27. Source corpus: `src-tauri/tweaks/performance.yaml` (23 tweaks) plus two verified
additions carried in from the gap hunt, for **25 tweaks total**.

Target platform: **Windows 11 24H2 (build 26100) and newer, including 25H2**, x64. Windows 10 IoT
Enterprise LTSC 2021 is a low-priority secondary target and is called out only where behaviour
differs.

This document consolidates the original validation pass and two adversarial rounds run over it. It is
the authoritative research record for this category: verdicts, exact mechanisms, every correction the
YAML still needs, and a ready-to-paste `info` block for each tweak in the `_INFO_TEMPLATE.md` shape.

Several findings rest on direct read-only inspection of a live Windows 11 24H2 machine (build
10.0.26100.4061): registry enumeration, `fsutil behavior query`, `powercfg /list`, `Get-MMAgent`,
`Get-Service`, `Win32_DeviceGuard` WMI, and string extraction from `fsutil.exe`, `ntfs.sys`,
`mmcss.sys` and `ntoskrnl.exe`. Live-system observations are labelled tier `C` (direct measurement on
one machine) and are evidence of what a value name is, not always evidence of a factory default.
Where the machine had clearly been tweaked already, that is stated rather than treated as a default.

**A standing note on placebo.** This category attracts myths harder than any other. Every entry below
states whether measured evidence supports the performance claim. Where it does not, the drawback
"no measurable gain on modern hardware" appears verbatim in the copy. That is not editorialising the
tweak out of existence; the control is real and the user is entitled to it. It is refusing to sell an
unmeasured number as a measured one.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `optimize_visual_effects` | INCORRECT | low | Community-corroborated | Writes only the dialog's radio-button state; no effect is actually changed. Stock default must be `absent`, not 0. |
| `disable_search_indexing` | VERIFIED | medium | Microsoft-documented | none |
| `disable_background_apps` | VERIFIED | low | Microsoft-documented | none |
| `memory_prefetch_mode` | VERIFIED | medium | Microsoft-documented | Mechanism correct; the `SysMain` stock start type is an **open question** and the revert depends on it. |
| `system_responsiveness` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Info's "a value of 0 reserves none at all" is false; values below 10 clamp to 20. |
| `enable_game_mode` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Info claims `AllowAutoGameMode` is written; it is not. True stock default not established. |
| `enable_gpu_scheduling` | VERIFIED-WITH-CORRECTION | medium | Community-corroborated | "Stock Default = 2" is machine-dependent; 1 and value-absent are both real stock states. |
| `disable_fullscreen_optimizations` | INCORRECT | medium | Community-corroborated | Both options write `GameDVR_FSEBehaviorMode: 2`, so the tweak cannot round-trip. Default option is not the default. |
| `disable_multiplane_overlay` | VERIFIED-WITH-CORRECTION | medium | Community-corroborated | Missing `OverlayMinFPS` companion; build gate leaves 22632 to 26099 uncovered. |
| `variable_refresh_rate` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Settings writes `VRROptimizeEnable=0` when turned off, which matches neither option. |
| `optimizations_windowed_games` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Info contradicts the effect it writes; `SwapEffectUpgradeCache` companion unhandled. |
| `disable_gamedvr_capture` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `AllowGameDVR=0` kills all recording and broadcasting, not only background capture. |
| `ultimate_performance_power_plan` | INCORRECT | medium | Microsoft-documented | `/duplicatescheme` mints a new GUID; `/setactive` against the template GUID fails and the probe can never succeed. |
| `disable_power_throttling` | VERIFIED-WITH-CORRECTION | medium | Community-corroborated | **Pitch is inverted.** The focused app is High QoS and is never throttled; the real effect is on background, minimized and occluded work. |
| `network_throttling_index` | VERIFIED | medium | Microsoft-documented | none. The literal `10` stock default is **correct**; do not change it to `absent`. |
| `disable_storage_sense` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Not Configured is off until low disk space or manual enablement; there is no default background schedule to stop. |
| `ssd_optimize_trim` | INCORRECT | low | Microsoft-documented | Value name is `DisableDeleteNotification`, not `NtfsDisableDeleteNotify`. The tweak is a no-op as authored. |
| `ntfs_disable_lastaccess` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot` missing. On volumes over 128 GB the real-world delta is often zero; say so. |
| `disable_memory_compression` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Reboot claim overstated; the cmdlet takes effect immediately for newly compressed pages. |
| `disable_fast_startup` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | `requires_reboot: true` is wrong; the change applies at the next **shutdown**, not after a restart. |
| `disable_vbs_hvci` | VERIFIED-WITH-CORRECTION | critical | Microsoft-documented | Stock Default `enable_vbs: 1` is wrong; setting both to 0 does not stop VBS on 24H2. Conflicts with `security:enable_credential_guard`. |
| `disable_spectre_meltdown` | VERIFIED-WITH-CORRECTION | critical | Microsoft-documented | 3/3 disables only Spectre v2 and Meltdown, and overwrites any Downfall/GDS bit already set. |
| `disable_mouse_acceleration` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `requires_reboot: true` is wrong; a sign-out or a `SystemParametersInfo` call is what applies it. |
| `reserved_storage_off` **(new)** | VERIFIED | medium | Microsoft-documented | New entry. Do not promise "roughly 7 GB"; the size varies, and upgraded machines may never have had reserved storage. |
| `no_index_encrypted_files` **(new)** | VERIFIED | low | Microsoft-documented | New entry. It **is** in the shipped ADMX; the earlier "not in the ADMX" sourcing claim was wrong. |

**Tally: 25 tweaks. 6 VERIFIED, 15 VERIFIED-WITH-CORRECTION, 4 INCORRECT, 0 UNVERIFIED, 0 DISPUTED.**

Four tweaks are INCORRECT and must not ship as authored: `ssd_optimize_trim`,
`ultimate_performance_power_plan`, `optimize_visual_effects`, `disable_fullscreen_optimizations`.

## Corrections required

1. **`ssd_optimize_trim`: the registry value name is wrong, and the tweak does nothing.**
   The YAML writes `NtfsDisableDeleteNotify` under `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`.
   No such value exists. String extraction from `C:\Windows\System32\fsutil.exe` yields the value
   names `DisableDeleteNotification` and `RefsDisableDeleteNotification`, plus the display strings
   `NTFS DisableDeleteNotify` and `ReFS DisableDeleteNotify`. String extraction from
   `C:\Windows\System32\drivers\ntfs.sys` yields `DisableDeleteNotification` and
   `DisableDeleteNotificationDrain`, and no `DisableDeleteNotify` at all. The live FileSystem key on
   Windows 11 24H2 contains `DisableDeleteNotification REG_DWORD 0x0` and contains neither
   `DisableDeleteNotify` nor `NtfsDisableDeleteNotify`. Correct name for NTFS:
   `DisableDeleteNotification`. Correct name for ReFS: `RefsDisableDeleteNotification`. The value
   semantics (0 = TRIM enabled, 1 = TRIM disabled) and the "Stock Default = 0" value are both right;
   only the name is wrong. Note that `DisableDeleteNotification` is present with value 0 on a stock
   machine, so `0` is the correct default rather than `absent`.

2. **`ultimate_performance_power_plan`: the apply script cannot activate the plan it creates.**
   `powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61` creates a copy under a **new**
   GUID. On the test machine, `powercfg /list` shows the resulting plan as
   `4da59277-6cd6-4604-83d4-4afaec3840e0 (Ultimate Performance)`; the template GUID `e9a42b02-...`
   does not appear in the scheme list at all. Therefore
   `powercfg /setactive e9a42b02-d5df-448d-aa00-03f14749eb61` fails, and the `probe`, which matches
   `powercfg /getactivescheme` against that same template GUID, can never return 0. The apply must
   capture the GUID that `/duplicatescheme` prints and activate that, and must detect the case where
   the scheme already exists so a second apply does not mint another copy. Additionally, `| Out-Null`
   discards both the GUID and any error text, so a failure surfaces as success, which violates the
   project's did-it-work contract.

3. **`ultimate_performance_power_plan`: `undo` hardcodes Balanced and orphans the duplicate.**
   The undo sets `381b4222-f694-41f0-9685-ff5bb260df2e` (Balanced) rather than restoring whatever
   scheme was active before. On the test machine the active scheme is an OEM plan
   (`85d583c5-... Legion Balance Mode`), which the undo would silently discard. The undo also never
   runs `powercfg /delete` on the duplicated scheme, so repeated apply/undo cycles accumulate
   "Ultimate Performance" copies.

4. **`optimize_visual_effects`: writing `VisualFXSetting` alone changes nothing.**
   On the live machine, `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects` has
   `ValueCount = 0`: `VisualFXSetting` is **absent**. The key holds 19 subkeys (`AnimateMinMax`,
   `ComboBoxAnimation`, `ControlAnimations`, `CursorShadow`, `DragFullWindows`, `DropShadow`,
   `DWMAeroPeekEnabled`, `DWMEnabled`, `DWMSaveThumbnailEnabled`, `FontSmoothing`,
   `ListBoxSmoothScrolling`, `ListviewAlphaSelect`, `ListviewShadow`, `MenuAnimation`,
   `SelectionFade`, `TaskbarAnimations`, `Themes`, `ThumbnailsOrIcon`, `TooltipAnimation`), each
   containing only `DefaultApplied`. The actual effect state lives in
   `HKCU\Control Panel\Desktop\UserPreferencesMask` (observed `9e,1e,07,80,12,00,00,00`),
   `HKCU\Control Panel\Desktop\WindowMetrics\MinAnimate`, and
   `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\{TaskbarAnimations,
   ListviewAlphaSelect, ListviewShadow}`. Two corrections: the "Stock Default" must be `absent`, not
   `0`; and the tweak must write the per-effect values and signal the running shell (normally via
   `SystemParametersInfo`) or nothing at all changes.

5. **`disable_fullscreen_optimizations`: both options write the same `GameDVR_FSEBehaviorMode`, so the
   tweak cannot round-trip.** "Exclusive fullscreen forced" writes 2 / 1 / 1 and
   "Default (Stock Default)" writes 2 / 0 / 1. `GameDVR_FSEBehaviorMode` is the one value that is
   supposed to differ between the two recipes, and it is identical in both: a copy-paste defect.
   Selecting "Default" therefore does not revert the tweak. Separately, the "Default" option is not
   the Windows default: on the live machine, whose `GameConfigStore` had never had this tweak applied,
   the values are `GameDVR_FSEBehaviorMode = 0`, `GameDVR_HonorUserFSEBehaviorMode = 0`,
   `GameDVR_DXGIHonorFSEWindowsCompatible = 0`, `GameDVR_EFSEFeatureFlags = 0`, so the "restore"
   option writes two wrong values.

6. **`disable_spectre_meltdown`: 3/3 does not disable all mitigations, and it clobbers others.**
   KB4073119 gives `FeatureSettingsOverride=3` / `FeatureSettingsOverrideMask=3` for CVE-2017-5715
   (Spectre variant 2) and CVE-2017-5754 (Meltdown) only. Other mitigations use different bits in the
   same DWORD: KB5029778 specifies `FeatureSettingsOverride = 0x2000000` (33554432) to disable the
   Gather Data Sampling / "Downfall" mitigation, and KB4073119's "enable everything except HT-off"
   setting is `FeatureSettingsOverride=72`. Because `FeatureSettingsOverride` is a single DWORD,
   writing 3 **overwrites** a previously configured `0x2000000`, silently re-enabling the GDS
   mitigation, and the revert to `absent` deletes an administrator's deliberate setting. The option
   label "Mitigations disabled" should read "Spectre v2 and Meltdown mitigations disabled". The stock
   default (`absent` for both) is correct.

7. **`disable_spectre_meltdown`: the Meltdown half is already a no-op on modern CPUs.**
   Windows enables KVA Shadow only when `IA32_ARCH_CAPABILITIES[RDCL_NO]` is clear. Any CPU reporting
   RDCL_NO (broadly, Intel from Ice Lake / 10th generation onward, and all AMD) never had KVA Shadow
   enabled, so there is nothing to reclaim. The info text should say this rather than framing the
   payoff as merely "small to none".

8. **`disable_vbs_hvci`: the Stock Default is wrong and the disable is not reliable.**
   Microsoft's OEM guidance says a memory-integrity-enabled image sets
   `Scenarios\HypervisorEnforcedCodeIntegrity` `Enabled=1`, `WasEnabledBy=1`, and `EnabledBootId`. It
   does **not** set `DeviceGuard\EnableVirtualizationBasedSecurity`, which is a policy value that is
   absent unless configured; the YAML's "Stock Default `enable_vbs: 1`" is therefore wrong and should
   be `absent` (with `hvci_enabled: 1` correct only on clean installs that met the auto-enablement
   bar, and `absent` on upgrades). Separately, on the test machine
   `EnableVirtualizationBasedSecurity = 0` and HVCI `Enabled = 0`, yet
   `Win32_DeviceGuard.VirtualizationBasedSecurityStatus = 2` ("enabled and running") and
   `SecurityServicesRunning = {1}` (Credential Guard running). Setting these two values to 0 turns off
   memory integrity but does not stop VBS or the hypervisor. The tweak's name and its promise of
   recovered CPU-bound FPS both overstate what these two registry values achieve.

9. **`disable_vbs_hvci`: `WasEnabledBy` is not handled.**
   Per Microsoft's registry guidance, deleting `WasEnabledBy` greys out the memory integrity UI with
   "This setting is managed by your administrator". The tweak neither deletes nor restores it, so a
   user who reverts through the tweak will not necessarily see the Windows Security UI return to a
   normal state, and a user who re-enables through Windows Security may end up in a state the tweak
   cannot detect.

10. **`disable_vbs_hvci` conflicts with `security:enable_credential_guard`, and the conflict must be
    cross-referenced.** Credential Guard is a VBS-hosted feature and cannot run without VBS. A user
    who applies the security tweak and then this one ends up with `LsaCfgFlags = 1` requesting a
    feature the machine can no longer host, and neither tweak's snapshot knows about the other.
    `_cross-category.md` notes that the conflict is currently masked because
    `enable_credential_guard` writes `LsaCfgFlags` to the wrong key
    (`Control\DeviceGuard` rather than `Control\Lsa` or the `Policies\...\DeviceGuard` policy path),
    so it is a no-op today. **Fixing that key activates the conflict**, so the dependency must be
    handled in the same change that corrects the key, not deferred. Preferred fix: declare a mutual
    exclusion or dependency in the schema. Minimum fix: explicit cross-references in both `info`
    blocks and both `warning` fields.

11. **`system_responsiveness`: the info text is wrong about 0.**
    Microsoft Learn states plainly: "Values below 10 and above 100 are clamped to 20. A value of 100
    disables MMCSS." The YAML's claim that "a value of 0 reserves none at all" and that "going all the
    way to 0 is aggressive and can starve background audio" is backwards: 0 is clamped to 20, which is
    the default. The default of 20 is confirmed live (`SystemResponsiveness REG_DWORD 0x14`). The
    claim that 10 is "the documented pro-audio setting" has no tier A/B source and should be dropped.

12. **`enable_game_mode`: the info describes a value the tweak does not write.**
    The text says it "sets the GameBar `AutoGameModeEnabled` and `AllowAutoGameMode` flags", but the
    effects block writes only `AutoGameModeEnabled`. On the test machine
    `HKCU\Software\Microsoft\GameBar` contains `ShowStartupPanel`, `AutoGameModeEnabled`,
    `UseNexusForGameBarEnabled`, `UseNexusForGameMode`, and no `AllowAutoGameMode` at all. Either
    write it or stop claiming it.

13. **`ntfs_disable_lastaccess`: `requires_reboot` is missing.**
    Microsoft's `fsutil behavior` reference says of `disablelastaccess`: "You must restart your
    computer for this parameter to take effect." The YAML has no `requires_reboot: true`.

14. **`ntfs_disable_lastaccess`: state the zero-delta case honestly.**
    In System Managed mode the NTFS driver decides at boot: last-access updates are enabled when the
    system volume is 128 GB or smaller and disabled when it is larger. On any machine with a system
    volume over 128 GB, which is nearly all of them, System Managed mode has **already** disabled the
    updates, so the tweak's real-world delta is often exactly zero. The copy must say this plainly
    rather than implying a saving that is not there.

15. **`disable_fast_startup`: `requires_reboot: true` is the wrong flag.**
    `HiberbootEnabled` governs what the **next shutdown** does. A reboot (Restart) is already a full
    cold boot and does not exercise the setting. The user needs a shutdown, not a reboot.

16. **`disable_mouse_acceleration`: `requires_reboot: true` is the wrong flag.**
    The values are `REG_SZ` under `HKCU\Control Panel\Mouse` (confirmed live: `MouseSpeed`,
    `MouseThreshold1`, `MouseThreshold2` all `REG_SZ`). They are read into the session at sign-in, or
    applied immediately by `SystemParametersInfo(SPI_SETMOUSE)`. A sign-out is sufficient; the info
    text itself says "sign out and back in", contradicting the flag.

17. **`disable_gamedvr_capture`: the policy is broader than the description.**
    `AllowGameDVR = 0` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR` disables Windows Game
    Recording **and Broadcasting** entirely, per the ADMX. The info says "The Game Bar overlay itself
    still works unless you disable it separately", which is misleading: manual recording and streaming
    are gone too, not just background capture.

18. **`variable_refresh_rate`: "Off" has two representations and only one is handled.**
    The stock state is the whole key `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` being absent
    (confirmed live). But when a user turns VRR **off** in Settings after having turned it on, Windows
    writes `VRROptimizeEnable=0` into `DirectXUserGlobalSettings` rather than removing the field. That
    state matches neither YAML option and will read as Unknown. The "Off (Stock Default)" option
    should accept both `absent` and `"0"`.

19. **`optimizations_windowed_games`: internal contradiction, and a missing companion value.**
    The info says "current builds do not expose a single clean registry value for it, so it is driven
    through the UI/API rather than a simple key", while the tweak writes exactly such a value. One of
    the two is wrong. Separately, community scripts that reproduce the Settings toggle also write
    `SwapEffectUpgradeCache=1` under `HKCU\Software\Microsoft\DirectX\GraphicsSettings`; whether that
    is required or merely a cache is not established. Also worth encoding: Microsoft states that
    turning on Auto HDR turns this on automatically and that it cannot be turned off while Auto HDR is
    on, so the tweak can be silently overridden.

20. **`disable_multiplane_overlay`: missing `OverlayMinFPS`, and a build-range gap.**
    The commonly published pre-24H2 recipe pairs `OverlayTestMode=5` with `OverlayMinFPS=0` under
    `HKLM\SOFTWARE\Microsoft\Windows\Dwm`. The YAML writes only `OverlayTestMode`. Also, the two
    effects are gated `<=22631` and `>=26100`, which leaves builds 22632 to 26099 with neither effect
    applied. That range is Insider-only in practice, but the gate should be `<26100` / `>=26100`.

21. **`enable_gpu_scheduling`: "Stock Default = 2" is not a safe universal default.**
    `HwSchMode` has three meaningful states: 1 (off), 2 (on), and absent-or-0 (let the system and
    driver decide). On the test Windows 11 24H2 machine the value is present and equal to 2. Multiple
    tier C references state that the default is 1 (off) and that OEMs may ship it on. Because the real
    stock state varies by OEM image and by GPU driver, hardcoding 2 as "Stock Default" will write an
    incorrect "restore" on machines whose factory state was 1 or absent.

22. **`disable_memory_compression`: the reboot claim is overstated.**
    `Disable-MMAgent -MemoryCompression` takes effect for newly compressed pages immediately; the
    existing compression store drains over time or at the next boot. `requires_reboot: true` is
    defensible as a "fully applied" signal but the info's flat "Requires a reboot" is not accurate.

23. **`network_throttling_index`: no YAML defect; correct the framing only, and keep the literal 10.**
    Microsoft's current MMCSS Learn page documents `SystemProfile` and `SystemResponsiveness` and says
    nothing about `NetworkThrottlingIndex`, but Microsoft KB 948066 documents the value directly: the
    same key, the default of 10 packets per millisecond, `FFFFFFFF` to disable, and a reboot required.
    The mechanism is still shipping on the target build (`NetworkThrottlingIndex` present in
    `mmcss.sys` on 26100.4061) and the default is present in the registry
    (`NetworkThrottlingIndex REG_DWORD 0xa`) alongside `SystemResponsiveness = 20`. **The literal `10`
    stock default is correct and must not become `absent`; writing `absent` would delete a value
    Windows ships and would itself be the revert bug.** What remains is scope: the cap only binds
    while an MMCSS multimedia task is registered and only bites on a saturated fast link, so the info
    text must keep promising nothing about latency or gaming.

24. **`disable_power_throttling`: add the hardware note and the sourcing note.**
    The feature is documented by Microsoft (Windows Insider blog, tier B), and the blog states it
    requires Intel Speed Shift, "available in Intel's 6th-gen and beyond Core processors". Microsoft
    does not document the registry override, but `ntoskrnl.exe` and `ntkrla57.exe` on 26100.4061 both
    contain the strings `PowerThrottling` and `PowerThrottlingOff` and no other System32 binary does,
    and four independent tier C sources spanning 2017 to the present agree on key, subkey, name, type
    and polarity. The default (`absent`, key not present) is confirmed live and is correct. The tweak
    should carry an applicability note that it is inert on hardware without Speed Shift class support,
    and should describe the override as community-corroborated rather than documented.

25. **`disable_power_throttling`: the description and info text state the effect backwards.**
    The YAML `description` reads "Stop Windows from clocking down foreground apps to save power" and
    the `info` repeats it ("stops Windows from **down-clocking foreground apps**"; "Foreground
    applications are **allowed to run at full clocks instead of being throttled**"). Microsoft's
    Quality of Service reference gives the classification the throttling acts on: for window-owning
    processes, **In Focus = High**, Visible = Medium, Minimized or Fully Occluded = Low, and any
    process playing audio is High. The focused foreground application is therefore already High QoS
    and is already exempt from EcoQoS down-clocking on a stock machine, so the promised benefit cannot
    occur. `PowerThrottlingOff = 1` removes throttling from **background, minimized and occluded**
    work. Rewrite the description and the copy around background work and delete the foreground claim
    entirely; do not repeat the old framing. This is a text defect only; the registry mechanism is
    correct.

26. **`disable_storage_sense`: the text asserts a default background schedule that does not exist.**
    The YAML `description` is "Stop scheduled automatic disk cleanup runs" and the `info` says Storage
    Sense "automatically runs scheduled disk cleanups" and that applying the tweak means "Windows
    stops running cleanup passes in the background". Microsoft's Storage Policy CSP documents the Not
    Configured state as "Storage Sense is turned off until the user runs into low disk space or the
    user enables it manually", and documents the cadence when it is on as "during low free disk
    space". There is no default recurring schedule to stop. The genuine benefit is that the machine
    policy pins Storage Sense off and prevents anything, including the user or a later Windows setup
    pass, turning it on; state it that way. The registry mechanism, the 0 semantics and the `absent`
    stock default are all correct.

27. **`reserved_storage_off` (new): three framing corrections before it ships.**
    (a) The "roughly 7 GB" figure is unsourced and does not appear on the cmdlet page. Reserved storage
    size varies with installed language packs and optional features. State it as "typically several
    GB, varies by configuration" and read the actual figure from the system rather than promising a
    number. (b) A machine may never have had reserved storage at all: Microsoft documents it as
    enabled automatically on new PCs with 1903 preinstalled and on clean installs, and **not** enabled
    when upgrading from an earlier version. On a large population of machines this reclaims nothing,
    and the probe must report "not applicable" rather than claiming success. This is a common
    upgrade-path outcome, not an LTSC quirk. (c) This is a **disk-space** control filed under
    `performance`, not a speed tweak, and that honesty has to survive into the shipped copy, because
    everything else in this file promises speed.

28. **`no_index_encrypted_files` (new): the sourcing claim in the gap proposal was wrong.**
    The proposal said "Not present in the shipped ADMX under that value name; written directly per the
    STIG check text". It **is** in the shipped ADMX on 26100 as policy
    `AllowIndexingEncryptedStoresOrItems`, class Machine, key
    `SOFTWARE\Policies\Microsoft\Windows\Windows Search`, `enabledValue` 1 / `disabledValue` 0. The
    reason it looked absent is an encoding trap: `Search.admx` ships as **UTF-16LE** while almost every
    other ADMX is UTF-8, so a byte-level UTF-8 grep misses every string in it. This raises the tweak's
    confidence from STIG-derived to Microsoft-documented. Two things the proposal omitted and the copy
    must carry: the shipped ADML warns that enabling or disabling this setting causes the index to be
    **rebuilt completely**; and the tweak is a no-op for anyone who has applied
    `performance:disable_search_indexing`, because there is then no index at all.

## New in this revision

Two tweaks are added to this category, both carried in from the gap hunt and re-verified in
`_verify-gaps-b-medlow.md`.

**`reserved_storage_off` (CONFIRMED, with a framing correction).** Microsoft Learn documents
`Set-WindowsReservedStorageState` in the DISM PowerShell module, with `-State` taking "either Disabled
or Enabled", and documents the exact failure mode when servicing is in flight. The paired
`Get-WindowsReservedStorageState` gives a real probe. Nothing else in the corpus touches reserved
storage, and it is the largest single reclaimable allocation on a stock 24H2 image. The framing
correction is that this is a disk-space control, the size is variable rather than "roughly 7 GB", and
upgraded machines may never have had reserved storage in the first place. See correction 27.

**`no_index_encrypted_files` (CORRECTED).** The mechanism, values, type and stock default were all
right in the proposal; the sourcing claim was wrong. The policy is in the shipped 26100 `Search.admx`,
and the shipped `Search.adml` explain text confirms the effective default from the horse's mouth:
"This policy setting is not configured by default. If you do not configure this policy setting, the
local setting, configured through Control Panel, will be used. By default, the Control Panel setting
is set to not index encrypted content." See correction 28.

Both are `VERIFIED`. Neither is sold as a speed tweak.

## Rejected performance myths

These were evaluated and rejected. They are recorded here with one-line reasons so nobody re-adds
them. **Do not reintroduce any of these without new measured evidence at build 26100 or later.**

| Candidate | Reason for rejection |
|---|---|
| Timer resolution forcing, HPET, `bcdedit /set useplatformclock`, `disabledynamictick` | Since Windows 11 22H2 a process requesting a high timer resolution no longer changes the global resolution, so the classic mechanism does not do what the guides claim on 26100. |
| `Win32PrioritySeparation` | A foreground/background quantum hint with no reproducible gaming benefit and no measured evidence at 26100. |
| QoS bandwidth reservation (`NonBestEffortLimit`, the "20 percent reserved bandwidth" claim) | The premise is false: the reservation only applies to applications that explicitly request QoS. |
| Pagefile disabling or fixed sizing | Causes hard failures in applications that commit large amounts of memory, and provides no measured gain. |
| Standby list clearing (`EmptyStandbyList` and friends) | Discards a cache the OS then has to refill; the measured effect is negative on average. |
| `LargeSystemCache` | Server-oriented, deprecated behaviour on client SKUs. |
| Prefetch and Superfetch registry disabling | The corpus already exposes the supported control, the `SysMain` service, in `memory_prefetch_mode`. Poking the registry values duplicates it less safely. |
| TCP registry packs (`TcpAckFrequency`, `TCPNoDelay`, `TcpWindowSize`, autotuning off, "gaming netsh" bundles) | Per-interface, undocumented, and repeatedly shown not to reduce game latency on modern stacks. |
| MMCSS `SystemProfile\Tasks\Games` tuning (`GPU Priority` = 8, `Scheduling Category` = High, `SFIO Priority` = High) | No published measurement of a frame-time or latency improvement on 26100; the stock values are already the tuned ones for most fields, so the circulated "fix" mostly rewrites defaults. Fails the measured-evidence bar. |
| MSI mode (`MSISupported`) and interrupt affinity | The registry path is per-device-instance (`PCI\VEN_...\Device Parameters\...`), so it is not expressible as a portable tweak, and setting it on a device whose driver does not support MSI can produce an unbootable system. Modern GPUs already default to MSI. |
| `ntfs_disable_8dot3` | The load-bearing Microsoft performance claim is not on the cited page, no measured benefit was found, and the real stock default on 26100 is `2` (per-volume), not the `0` or `absent` the proposal assumed, which makes the revert a hazard on top of an unmeasured benefit. |

Also previously rejected and recorded in `_gaps-security-performance-network.md` section C.1, listed
here so they are not rediscovered: `DisablePagingExecutive`, `SvcHostSplitThresholdInKB`,
`NtfsMemoryUsage`, `TdrDelay` / `TdrDdiDelay`, PCIe ASPM and USB 3 link power management off,
processor `IDLEDISABLE` and core parking overrides, `powercfg /overlaysetactive`,
`ClearPageFileAtShutdown`, and `Ndu` / `DusmSvc` disabling.

## Tweak entries

### `optimize_visual_effects` Visual effects for performance

**Verdict:** INCORRECT

**Mechanism:**

As authored, one effect, `visual_fx`:

- Key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects`
- Value name: `VisualFXSetting`
- Type: `REG_DWORD`
- Option "Best performance": `visual_fx: 2`
- Option "Let Windows choose (Stock Default)": `visual_fx: 0`

`VisualFXSetting` records which radio button is selected in the Performance Options dialog
(0 = let Windows choose, 1 = best appearance, 2 = best performance, 3 = custom). It is a record of the
choice, not the mechanism that implements it. On the live 24H2 machine the `VisualEffects` key has
`ValueCount = 0` (so `VisualFXSetting` is absent) and 19 subkeys each holding only `DefaultApplied`.

**CORRECTED mechanism.** The stock default is `absent`, not `0`, and the tweak must write the values
that actually drive the effects and then signal the running shell:

| Key | Value | Type | Best performance | Stock |
|---|---|---|---|---|
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects` | `VisualFXSetting` | `REG_DWORD` | `2` | `absent` |
| `HKCU\Control Panel\Desktop` | `UserPreferencesMask` | `REG_BINARY` | animation bits cleared | observed stock `9e,1e,07,80,12,00,00,00` |
| `HKCU\Control Panel\Desktop\WindowMetrics` | `MinAnimate` | `REG_SZ` | `"0"` | `"1"` |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarAnimations` | `REG_DWORD` | `0` | `1` |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `ListviewAlphaSelect` | `REG_DWORD` | `0` | `1` |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `ListviewShadow` | `REG_DWORD` | `0` | `1` |

plus a `SystemParametersInfo` broadcast (`SPI_SETUIEFFECTS` / `SPI_SETANIMATION` and
`SPIF_SENDCHANGE`) so the running shell picks the change up. Without that broadcast the values apply
only at the next sign-in.

**WRONG mechanism, as currently authored, for reference:** a single `VisualFXSetting` write with a
stock default of `0`. This changes nothing on the machine, and the `0` default writes a value where
none existed, so a later revert leaves the machine in a state it was never in.

**Corrections needed:** see correction 4. Two concrete errors: the Stock Default must be `absent`, not
`0`; and the tweak must write the per-effect values and signal the shell, or it is a no-op. Until
both land, this tweak must not ship: the user applies it, sees no change, and concludes either that
the app is broken or that the tweak is placebo.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off desktop animations and shadows so menus and windows appear instantly.**

      ## What it does
      Applies the "Adjust for best performance" visual profile. It sets `VisualFXSetting` to 2 and
      writes the values that actually drive the effects: `UserPreferencesMask` and `MinAnimate`
      under `Control Panel\Desktop`, plus `TaskbarAnimations`, `ListviewAlphaSelect` and
      `ListviewShadow` under Explorer's `Advanced` key.

      ## Benefits
      - **Instant menus**: no fade or slide before a menu or window appears
      - **Helps weak GPUs**: the shell stops compositing effects it does not need to
      - **Better over RDP**: animations are the worst thing to send down a remote session

      ## Drawbacks
      - **Flatter desktop**: no fades, slides, or drop shadows anywhere in the shell
      - **No FPS change**: this is shell chrome; it does not touch in-game frame rates
      - **No measured gain**: no measurable gain on modern hardware; the animations cost close to nothing on a current
        GPU, so the benefit is how it feels, not what it measures

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately for most effects, fully after sign-out or restarting Explorer
      - **Reverting**: restores the previous values from the snapshot, including removing
        `VisualFXSetting` if it was not present before

      ## Recommendation
      Worth it on an older or low-end machine, over Remote Desktop, or if you simply prefer an
      instant, no-frills desktop. Leave it alone on modern hardware if you like the polished look;
      you are trading appearance for a difference you will feel but not measure.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [SystemParametersInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow)
      - [fsutil behavior (pattern for tool-written registry values)](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior)
```

**Sources:**

1. SystemParametersInfoW, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow (tier A; the API the Performance Options dialog calls so the running shell applies the change)
2. "Modify Performance Options and Visual Effects via Registry", https://mattwv.wordpress.com/2016/06/01/modify-performance-options-and-visual-effects-via-registry/ (tier D; used only for the 0/1/2/3 enumeration, which the live machine could not confirm because the value is absent)
3. "adjust for best performance via group policy", Microsoft-hosted forum thread stating that the registry setting alone does not trigger the API call the system needs to make, https://social.technet.microsoft.com/Forums/ie/en-US/39d6e2c3-0c5a-4fe1-98af-9a881d619187/adjust-for-best-performance-via-group-policy (tier D)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `VisualEffects` key `ValueCount = 0`, 19 subkeys each holding only `DefaultApplied`; `UserPreferencesMask = 9e,1e,07,80,12,00,00,00`, `MinAnimate = 1`, `TaskbarAnimations = 1`, `ListviewAlphaSelect = 1`, `ListviewShadow = 1` (tier C)

### `disable_search_indexing` Windows Search indexing

**Verdict:** VERIFIED

**Mechanism:** one effect, `wsearch`, a service start-type change.

- Service short name: `WSearch` (Windows Search)
- Option "Disabled": `wsearch: disabled` (start type Disabled, 4)
- Option "Automatic, Delayed (Stock Default)": `wsearch: automatic_delayed` (start type Automatic, 2,
  with `DelayedAutoStart = 1`)

Confirmed live on Windows 11 24H2: `HKLM\SYSTEM\CurrentControlSet\Services\WSearch` has `Start = 2`
and `DelayedAutoStart = 1`, exactly matching the YAML's stock default, with no start or stop triggers
registered. `WSearch` hosts `SearchIndexer.exe`, which builds the property store and full-text index
used by Start menu search, File Explorer search, and applications that query the Windows Search API
such as classic Outlook. No reboot is required; stopping and disabling the service takes effect
immediately.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the background file indexer, cutting idle disk and CPU activity.**

      ## What it does
      Disables the `WSearch` service, which continuously catalogs your files so Start and File
      Explorer searches return instantly. With it stopped, Windows no longer builds or maintains
      the search index.

      ## Benefits
      - **Less idle I/O**: no background indexing while you are not using the PC
      - **Frees CPU**: indexing spikes noticeably on large libraries
      - **Redundant with Everything**: no reason to index twice if you use a third-party search

      ## Drawbacks
      - **Slow file search**: Explorer scans on demand instead of reading a prebuilt index
      - **Start menu search**: finding apps and settings becomes slower and less complete
      - **Outlook search breaks**: classic Outlook depends on the Windows index
      - **No measured gain**: no measurable gain on modern hardware; under 20 MB written per day and under 1 percent
        idle CPU on a healthy SSD machine

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the service stops on apply
      - **Reverting**: restores the previous start type from the snapshot
      - On Windows 11 the taskbar Search flyout is more tightly coupled to this service than it was
        on Windows 10, and can return nothing rather than merely returning results slowly

      ## Recommendation
      Worth it on a hard drive or a low-RAM machine, or if you already search with Everything. On a
      healthy SSD with adequate RAM, leave it enabled; the gain does not justify losing search.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Windows Search overview](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
      - [Guidelines for disabling system services](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
```

**Sources:**

1. Windows Search overview, https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview (tier A)
2. Security guidelines for disabling system services, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
3. Windows Search (WSearch) service defaults in Windows 11, https://revertservice.com/11/wsearch/ (tier C)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `WSearch Start = 2`, `DelayedAutoStart = 1`, no triggers (tier C)

### `disable_background_apps` UWP background apps

**Verdict:** VERIFIED

**Mechanism:** one effect, `run_in_background`.

- Key: `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`
- Value name: `LetAppsRunInBackground`
- Type: `REG_DWORD`
- Option "Force denied": `2`
- Option "User in control (Stock Default)": `absent`

This is the registry backing of the "Let Windows apps run in the background" policy from
`AppPrivacy.admx` (Computer Configuration > Administrative Templates > Windows Components > App
Privacy). Enumeration: 0 = user is in control, 1 = Force Allow, 2 = Force Deny. Force Deny blocks
packaged (UWP / Store) apps from registering and running background tasks; Win32 desktop applications
are unaffected. On the live 24H2 machine the `AppPrivacy` policy key is absent, confirming that the
correct stock default is value-absent rather than a written 0. No reboot is required, though affected
apps pick up the change on their next background-task registration.

**Corrections needed:** `none`

Note, recorded rather than actioned: `_cross-category.md` observes that this tweak's framing is
performance while its mechanism is an `AppPrivacy` policy, the same key family as
`privacy:disable_app_diagnostics`. Either placement is arguable. No change required.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Store and UWP apps waking to refresh in the background.**

      ## What it does
      Sets the `LetAppsRunInBackground` policy to Force Deny (2), which blocks packaged Store and
      UWP apps from registering or running background tasks. Classic Win32 desktop programs are
      not affected.

      ## Benefits
      - **Less idle CPU**: apps stop waking themselves while you are not using them
      - **Less network chatter**: no background fetches from Store apps
      - **Better battery**: the clearest win, on laptops and handhelds

      ## Drawbacks
      - **Notifications stop**: push and toast alerts from Mail, Calendar and Store messengers stop
        or are delayed
      - **Live tiles freeze**: packaged apps no longer update their tiles
      - **Search side effects**: Microsoft notes Cortana and Search "might not function as expected"
        under Force Deny
      - **Settings greyed out**: the per-app background permission UI is locked once the policy is set

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately; each app picks it up at its next background-task registration
      - **Reverting**: removes the policy value, returning control to the user

      ## Recommendation
      Good for most people who do not live inside Store apps and want less background activity, and
      close to mandatory on a battery-limited machine. Skip it if you rely on real-time alerts from
      Mail or a Store messenger.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Privacy Policy CSP (LetAppsRunInBackground)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy)
      - [Manage connections from Windows to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**

1. Privacy Policy CSP, `LetAppsRunInBackground`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
2. Manage connections from Windows to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. `AppPrivacy.admx` (`LetAppsRunInBackground`, values 0/1/2), https://github.com/Harvester57/W10-ADMX/blob/master/AppPrivacy.admx (tier A mirror of the shipped ADMX)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` absent (tier C)

### `memory_prefetch_mode` SysMain (SuperFetch) prefetching

**Verdict:** VERIFIED (with an unresolved stock default, see below)

**Mechanism:** one effect, `sysmain`, a service start-type change.

- Service short name: `SysMain` (formerly `SuperFetch`)
- Option "Disabled": `sysmain: disabled` (start type Disabled, 4)
- Option "Automatic (Stock Default)": `sysmain: automatic` (start type Automatic, 2)

`SysMain` is the memory management agent service. It profiles application launch patterns and
prepopulates the standby list so frequently used binaries and data are already in RAM. It also drives
ReadyBoost and, historically, prelaunch. Disabling it stops the profiling, the prefetch reads, and the
associated standby-list warming. No reboot is required; stopping the service takes effect immediately,
though the standby list is only rebuilt over subsequent use.

**Open question, carried forward.** The `SysMain` stock start type **could not be verified** and
remains open. The YAML claims `automatic`, which matches every published reference, but the
26100.4061 machine available to this pass reports `START_TYPE : 4 DISABLED` with
`ERROR_CONTROL : 0 IGNORE`. Stock `SysMain` is `ERROR_CONTROL : 1 NORMAL`, so that machine has almost
certainly been modified by a prior tweaking tool and is not usable as evidence either way. This is the
one service default in the file that could not be observed on an untouched image. Confirm it against a
clean 24H2 image before trusting the revert path, because a wrong start type restores the service to a
state the machine never shipped with. For contrast, `WSearch` was observable and is confirmed correct.

**Corrections needed:** none to the mechanism. **The stock start type must be confirmed on a clean
Windows 11 24H2 image before this tweak's revert path can be trusted.** Track it as a blocking item,
not a nice-to-have: it is the revert value, which `_harmful-revert.md` treats as the highest-stakes
class of defect in the corpus.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off the SysMain preloader, a targeted fix for the case where it pins your disk at 100 percent.**

      ## What it does
      Disables `SysMain` (formerly SuperFetch), the service that studies which apps you launch and
      preloads them into spare RAM so they open faster. With it off, Windows stops that background
      profiling and pre-caching, along with the disk and memory activity that comes with it.

      ## Benefits
      - **Stops disk thrashing**: the one clear win, when SysMain itself is the process pinning
        the disk at 100 percent
      - **No background profiling**: no launch-pattern tracking and no standby-list warming
      - **Quieter idle**: less disk activity on a spinning drive that is already struggling

      ## Drawbacks
      - **Slower cold launches**: applications open from disk instead of from a warmed cache
      - **ReadyBoost stops working**: irrelevant on any modern machine, but it does stop
      - **No measured gain**: no measurable gain on modern hardware; on an SSD with adequate RAM, SysMain's cost is
        negligible and no published measurement shows an FPS benefit from disabling it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the service stops on apply
      - **Reverting**: restores the previous start type from the snapshot
      - This is a different feature from memory compression; disabling `SysMain` does not disable
        compression, which has its own tweak

      ## Recommendation
      Leave it enabled unless you have confirmed SysMain itself is causing sustained 100 percent disk
      usage on your machine. It is a targeted fix for one failure mode, not a general speedup.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Disable-MMAgent (the MMAgent feature set SysMain drives)](https://learn.microsoft.com/en-us/powershell/module/mmagent/disable-mmagent)
      - [Guidelines for disabling system services](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
```

**Sources:**

1. Disable-MMAgent, which documents that SysMain / MMAgent covers application launch prefetching, prelaunch, page combining, and memory compression as separable features, https://learn.microsoft.com/en-us/powershell/module/mmagent/disable-mmagent (tier A)
2. Security guidelines for disabling system services, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
3. "How to Configure SuperFetch (SysMain) in Windows", https://www.ninjaone.com/blog/how-to-configure-superfetch-in-windows/ (tier C)
4. Direct inspection, Windows 11 24H2 build 26100.4061: service present, `START_TYPE : 4 DISABLED`, `ERROR_CONTROL : 0 IGNORE`, i.e. already modified and not usable as evidence of the default (tier C)

### `system_responsiveness` MMCSS reserved CPU (System Responsiveness)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `responsiveness`.

- Key: `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`
- Value name: `SystemResponsiveness`
- Type: `REG_DWORD`
- Option "Reduced (10)": `10`
- Option "Default (20) (Stock Default)": `20`
- `requires_reboot: true` (correct)

Microsoft Learn documents this exactly: the key "contains a REG_DWORD value named
SystemResponsiveness that determines the percentage of CPU resources that should be guaranteed to
low-priority tasks. For example, if this value is 20, then 20% of CPU resources are reserved for
low-priority tasks." Values not divisible by 10 round down; **values below 10 and above 100 are
clamped to 20**; 100 disables MMCSS entirely (the driver returns `STATUS_SERVER_DISABLED`). The
reservation is only in force while a thread has registered an MMCSS task via
`AvSetMmThreadCharacteristics`, which in practice means while audio or video is playing. Confirmed
live on 24H2: `SystemResponsiveness REG_DWORD 0x14` (20), present by default, so the literal `20`
stock default is correct and must not become `absent`.

**Corrections needed:** see correction 11. The info text's "a value of 0 reserves none at all" and
"going all the way to 0 is aggressive and can starve background audio" are backwards: 0 is clamped to
20, which is the default. Rewrite: values below 10 clamp to 20, 100 disables MMCSS. Drop the unsourced
claim that 10 is "the documented pro-audio setting"; no tier A or B source says so.

**Ready-to-paste info block:**

```yaml
    info: |
      **Lowers the CPU share MMCSS holds back for background work while audio or video is playing.**

      ## What it does
      Sets `SystemResponsiveness` to 10. Microsoft documents this value as the percentage of CPU
      guaranteed to low-priority tasks while a multimedia thread is registered with the Multimedia
      Class Scheduler Service. The default is 20, so this halves the reservation.

      ## Benefits
      - **More headroom**: multimedia threads get a larger share under contention
      - **Audio workstations**: the one setup where the reservation is worth tuning at all
      - **Fully reversible**: a single documented DWORD with a documented default

      ## Drawbacks
      - **Background stutter**: less CPU guaranteed to low-priority work can surface as glitches in
        a background stream or recording under heavy load
      - **Playback only**: the reservation does nothing unless something has registered
        an MMCSS task
      - **No measured gain**: no measurable gain on modern hardware; no published measurement shows an FPS or frame-time
        effect from this value

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: after reboot
      - **Reverting**: restores the previous value from the snapshot; the shipped default is 20 and
        the value is present out of the box, not absent
      - Values below 10 and above 100 are clamped back to 20, so setting 0 does not "reserve nothing";
        it gives you the default. A value of 100 disables MMCSS entirely.

      ## Recommendation
      Worth setting on an audio production machine where you want the audio engine to hold priority
      under load. Gamers should skip it: MMCSS governs multimedia thread priority, not your game's
      render thread, and there is no measurement showing a frame rate effect.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Multimedia Class Scheduler Service](https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service)
      - [AvSetMmThreadCharacteristicsW](https://learn.microsoft.com/en-us/windows/win32/api/avrt/nf-avrt-avsetmmthreadcharacteristicsw)
```

**Sources:**

1. Multimedia Class Scheduler Service, https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service (tier A; the `SystemResponsiveness` semantics, the clamping rule, and the 100-disables-MMCSS behaviour)
2. AvSetMmThreadCharacteristicsW, https://learn.microsoft.com/en-us/windows/win32/api/avrt/nf-avrt-avsetmmthreadcharacteristicsw (tier A; establishes that the reservation only binds while a task is registered)
3. Direct inspection, Windows 11 24H2 build 26100.4061: `SystemResponsiveness = 0x14` present out of the box (tier C)

### `enable_game_mode` Windows Game Mode

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `auto_game_mode`.

- Key: `HKCU\Software\Microsoft\GameBar`
- Value name: `AutoGameModeEnabled`
- Type: `REG_DWORD`
- Option "On (Stock Default)": `1`
- Option "Off": `0`

`AutoGameModeEnabled` is the registry backing of Settings > Gaming > Game Mode. When on, Windows
applies a scheduling and resource profile to a detected game process: background work such as Windows
Update and indexing is deprioritised, and the game gets more consistent CPU and GPU scheduling.
Windows also stores per-game profile data alongside it, visible on the live machine as the
`Win32_AutoGameModeDefaultProfile` and `Win32_GameModeRelatedProcesses` binary values in
`HKCU\System\GameConfigStore`. Takes effect at the next game launch; no reboot. On the live machine
`HKCU\Software\Microsoft\GameBar` contains `ShowStartupPanel`, `AutoGameModeEnabled` (0, because the
user had turned it off), `UseNexusForGameBarEnabled` and `UseNexusForGameMode`, and notably **not**
`AllowAutoGameMode`.

**Corrections needed:** see correction 12. The info claims `AllowAutoGameMode` is written; it is not,
and that value is absent on a live 24H2 machine. Either write it or stop claiming it; the copy below
drops the claim. Separately, the "Stock Default = 1" could not be established: the value was present
but user-modified on the test machine, and no tier A or B statement of the factory value was found. It
may be value-absent on a clean install until the user visits the Settings page. Confirm on a clean
image before trusting the revert.

**Ready-to-paste info block:**

```yaml
    info: |
      **Lets Windows push background work out of the way while you play, smoothing your 1 percent lows.**

      ## What it does
      Enables Windows Game Mode by setting the Game Bar value `AutoGameModeEnabled` to 1. When
      Windows detects a game, it deprioritises background work such as Windows Update and indexing
      and gives the game more consistent CPU and GPU scheduling.

      ## Benefits
      - **Steadier frame times**: the reported benefit is better 1 percent lows, not higher average FPS
      - **Fewer mid-game hitches**: background updates and indexing are held off during play
      - **Zero risk**: a per-user toggle that mirrors the Settings switch exactly

      ## Drawbacks
      - **Background work slows**: a download, compile, or encode running alongside the game gets less
      - **Occasional title conflicts**: a few games have historically behaved worse with it on
      - **No measured gain**: no measurable gain on modern hardware; no tier A or B source publishes a benchmark, and
        community measurements of the average-FPS effect land inside run-to-run noise

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: at the next game launch
      - **Reverting**: restores the previous value from the snapshot
      - Windows keeps per-game profile data under `HKCU\System\GameConfigStore`, which this tweak
        does not touch

      ## Recommendation
      Leave it on. It is the safest tweak in this category and the failure mode is a single title
      behaving oddly, which you can then handle per game. Turn it off if you stream or encode while
      playing and want those processes to keep their share.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Game Mode in Windows](https://support.microsoft.com/en-us/windows/game-mode-in-windows-2ffe7e17-2c1b-c0a1-c5cb-ecb98efd41ae)
      - [How to turn Game Mode on or off](https://www.majorgeeks.com/content/page/how_to_turn_on_or_off_game_mode_in_windows_10.html)
```

**Sources:**

1. Game Mode in Windows, https://support.microsoft.com/en-us/windows/game-mode-in-windows-2ffe7e17-2c1b-c0a1-c5cb-ecb98efd41ae (tier A; documents the feature and what it prioritises, but not the registry value)
2. "How to Turn On or Off Game Mode in Windows 10 & 11", https://www.majorgeeks.com/content/page/how_to_turn_on_or_off_game_mode_in_windows_10.html (tier D; cited for the value name and 0/1 semantics only)
3. Direct inspection, Windows 11 24H2 build 26100.4061: `HKCU\Software\Microsoft\GameBar` value list, `AllowAutoGameMode` absent (tier C)

### `enable_gpu_scheduling` Hardware-accelerated GPU scheduling (HAGS)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `hw_sch_mode`.

- Key: `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers`
- Value name: `HwSchMode`
- Type: `REG_DWORD`
- Option "Enabled (Stock Default)": `2`
- Option "Disabled": `1`
- `requires_reboot: true` (correct)

`HwSchMode` has three meaningful states: **1 = off, 2 = on, and 0 or value-absent = let the system and
driver decide**. It is the registry backing of Settings > System > Display > Graphics > Default
graphics settings > Hardware-accelerated GPU scheduling. HAGS moves GPU work submission and
frame-queue management from the CPU-side WDDM scheduler to a hardware scheduling processor on the GPU,
reducing the CPU-side cost of submitting work. Requires Windows 10 2004 (19041) or later and a WDDM
2.7 or newer driver; the toggle is hidden in Settings on unsupported hardware. Confirmed live on
Windows 11 24H2: `HwSchMode REG_DWORD 0x2` present, alongside `DxgKrnlVersion 0x11007` (WDDM 3.1
class).

**Corrections needed:** see correction 21. "Stock Default = 2" is not universally true. Tier C
references state the default is 1 (off) and that OEMs may ship it on; the one machine measured had 2.
Because the real stock state varies by OEM image and GPU driver, hardcoding 2 as the restore value
writes an incorrect "stock" on machines whose factory state was 1 or absent. The safest shape is a
third option or an `absent` default representing "let the system decide", and the tweak should not
claim to restore a value it cannot know. Confirm on a clean 24H2 image.

**Ready-to-paste info block:**

```yaml
    info: |
      **Hands GPU frame scheduling to the GPU itself, and unlocks NVIDIA Frame Generation.**

      ## What it does
      Sets `HwSchMode` to 2, enabling Hardware-Accelerated GPU Scheduling. Frame-queue management
      moves from the CPU-side Windows scheduler to a dedicated processor on the GPU, trimming some
      of the CPU overhead of submitting GPU work.

      ## Benefits
      - **Enables Frame Generation**: NVIDIA DLSS Frame Generation will not run without it, which
        is the strongest reason to turn it on
      - **Less CPU work**: helps marginally on a CPU-limited system
      - **Mirrors Settings exactly**: the same toggle Windows exposes, applied from here

      ## Drawbacks
      - **Slightly more VRAM**: the GPU-side scheduler has its own footprint
      - **VR stutter reports**: some VR users have seen regressions with it enabled
      - **No measured gain**: no measurable gain on modern hardware; outside Frame Generation, published average-FPS
        testing puts the effect inside run-to-run noise

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, on a GPU with a WDDM 2.7 or newer driver
      - **Takes effect**: after reboot
      - **Reverting**: restores the previous value from the snapshot
      - The toggle is hidden in Settings on hardware that does not support it, and GPU driver
        installers can rewrite this value

      ## Recommendation
      Turn it on if you want DLSS Frame Generation or you are CPU-limited; that is a concrete reason.
      If you are on VR or you have no specific problem to solve, leave it where your machine shipped:
      you will not measure the difference.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [Hardware-accelerated GPU scheduling (Windows support)](https://support.microsoft.com/en-us/windows/hardware-accelerated-gpu-scheduling-4f5ba4a0-4b26-4f4e-8b4b-0a1e9a2b4d55)
      - [Should you enable hardware-accelerated GPU scheduling?](https://www.pcworld.com/article/2339130/should-you-enable-hardware-accelerated-gpu-scheduling-in-windows-11.html)
```

**Sources:**

1. "How to Enable Hardware-Accelerated GPU Scheduling in Windows 10 and 11", https://www.howtogeek.com/756935/how-to-enable-hardware-accelerated-gpu-scheduling-in-windows-11/ (tier C; key path, value name, and the 1/2 semantics)
2. "Should you enable hardware-accelerated GPU scheduling in Windows 11?", https://www.pcworld.com/article/2339130/should-you-enable-hardware-accelerated-gpu-scheduling-in-windows-11.html (tier C; independent measurement discussion, and the default-off claim)
3. Direct inspection, Windows 11 24H2 build 26100.4061: `HwSchMode = 0x2` present, `DxgKrnlVersion = 0x11007` (tier C)

### `disable_fullscreen_optimizations` Fullscreen optimizations (force exclusive)

**Verdict:** INCORRECT

**Mechanism:**

As authored, three effects, all under `HKCU\System\GameConfigStore`, all `REG_DWORD`:

| Effect id | Value name | "Exclusive fullscreen forced" | "Default (Stock Default)" |
|---|---|---|---|
| `fse_behavior` | `GameDVR_FSEBehaviorMode` | `2` | `2` |
| `honor_user_fse` | `GameDVR_HonorUserFSEBehaviorMode` | `1` | `0` |
| `dxgi_honor_fse` | `GameDVR_DXGIHonorFSEWindowsCompatible` | `1` | `1` |

**The defect is visible in the table.** `GameDVR_FSEBehaviorMode` is `2` in both options, so the one
value that is supposed to distinguish "force exclusive" from "leave alone" is identical in both. The
tweak cannot round-trip: selecting "Default" does not revert it.

**CORRECTED mechanism, provisional.** On the live machine, whose `GameConfigStore` had never had this
tweak applied, the observed values are:

| Value name | Observed stock | Type |
|---|---|---|
| `GameDVR_FSEBehaviorMode` | `0` | `REG_DWORD` |
| `GameDVR_HonorUserFSEBehaviorMode` | `0` | `REG_DWORD` |
| `GameDVR_DXGIHonorFSEWindowsCompatible` | `0` | `REG_DWORD` |
| `GameDVR_EFSEFeatureFlags` | `0` | `REG_DWORD` |

So the honest stock-default representation is `absent` for all three, or 0 / 0 / 0, and the "force
exclusive" option must write a `GameDVR_FSEBehaviorMode` that differs from the stock value. Which
value that is could not be settled: the two tier C/D sources found disagree, one saying
`GameDVR_FSEBehaviorMode` 0 = high-impact games only and 1 = all fullscreen games, the other saying
1 = enable and 2 = disable. Microsoft documents none of the three values.

**WRONG mechanism, as currently authored, for reference:** both options writing `fse_behavior: 2`,
with a "Stock Default" of 2 / 0 / 1 that does not match any observed stock machine.

Fullscreen Optimizations itself is the DWM path that runs a game which requested exclusive fullscreen
as a borderless flip-model window, so that Alt-Tab, overlays, Auto HDR, and VRR keep working. The
`GameConfigStore` values are the community-canonical way to force the legacy exclusive path instead.
The key exists on Windows 10 1703 and later and on all Windows 11 builds, per-user. No reboot; the
values are read when a game starts. Note that the equivalent per-game setting in the executable's
compatibility properties is targeted, whereas this is a global per-user override applied to every
game, which is the wrong granularity for a per-title workaround.

**Corrections needed:** see correction 5. Two defects. First, both options write
`GameDVR_FSEBehaviorMode: 2`, so the tweak cannot round-trip; this alone makes it unshippable.
Second, the "Default (Stock Default)" option writes two values that are not the Windows default. **Do
not ship this tweak until the clean-install values and the real semantics of `GameDVR_FSEBehaviorMode`
are established on an untouched 24H2 image.** The info block below is written for the corrected form
and is provisional on that.

**Ready-to-paste info block:**

```yaml
    info: |
      **Forces games out of the modern compositing path into legacy exclusive fullscreen.**

      ## What it does
      Writes the `GameConfigStore` values `GameDVR_FSEBehaviorMode`,
      `GameDVR_HonorUserFSEBehaviorMode` and `GameDVR_DXGIHonorFSEWindowsCompatible` so games that
      ask for exclusive fullscreen get it, instead of being run as a borderless flip-model window
      by the Desktop Window Manager.

      ## Benefits
      - **Troubleshooting lever**: for a specific title that stutters or tears in borderless mode
      - **Bypasses the compositor**: the game gets direct control of the display
      - **Per-user and instant**: no reboot, and it applies at the next game launch

      ## Drawbacks
      - **Breaks modern features**: Auto HDR, variable refresh rate, and fast Alt-Tab all depend on
        the compositing path this bypasses
      - **Alt-Tab flicker**: black-screen flashes on task switch are a common result
      - **Wrong granularity**: this is a global override applied to every game, where the per-game
        compatibility setting is targeted
      - **No measured gain**: no measurable gain on modern hardware; on current flip-model DWM the exclusive path is
        usually equal to or worse than the optimised one, and no measurement shows a general win

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: at the next game launch
      - **Reverting**: restores the previous values from the snapshot
      - These values are undocumented by Microsoft and community sources disagree on what each
        number means, so treat the behaviour as title-dependent and verify it yourself

      ## Recommendation
      Only for troubleshooting one specific game that stutters in borderless mode, and only after
      trying that game's own fullscreen setting first. Everyone else should leave it off; you are
      trading Auto HDR, VRR, and fast Alt-Tab for an effect nobody has measured.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [Optimizations for windowed games in Windows 11](https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11)
      - [Microsoft Q&A: enable or disable full-screen optimizations](https://learn.microsoft.com/en-us/answers/questions/4079856/enable-or-disable-full-screen-optimizations-on-win)
```

**Sources:**

1. Optimizations for windowed games in Windows 11, https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11 (tier A; documents the modern presentation path these values bypass, and what depends on it)
2. Microsoft Q&A, "enable or disable Full-screen optimizations on Windows 11 (REGEDIT)", https://learn.microsoft.com/en-us/answers/questions/4079856/enable-or-disable-full-screen-optimizations-on-win (tier D; the answer covers `GameDVR_FSEBehavior` only, and the question about the three values in this tweak is explicitly left unanswered)
3. "Fullscreen Optimizations and HDR", DeepWiki index of shoober420/windows11-scripts, https://deepwiki.com/shoober420/windows11-scripts/5.3-fullscreen-optimizations-and-hdr (tier D; a tweak-script index, used only to show that community recipes disagree)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `GameDVR_FSEBehaviorMode = 0`, `GameDVR_HonorUserFSEBehaviorMode = 0`, `GameDVR_DXGIHonorFSEWindowsCompatible = 0`, `GameDVR_EFSEFeatureFlags = 0` on a machine that had never had this tweak applied (tier C)

### `disable_multiplane_overlay` Multi-plane overlay (MPO)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two build-gated effects.

| Effect id | Key | Value name | Type | Build gate | "Overlays disabled" | "Enabled (Stock Default)" |
|---|---|---|---|---|---|---|
| `overlay_test_mode` | `HKLM\SOFTWARE\Microsoft\Windows\Dwm` | `OverlayTestMode` | `REG_DWORD` | `<=22631` | `5` | `absent` |
| `disable_overlays` | `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers` | `DisableOverlays` | `REG_DWORD` | `>=26100` | `1` | `absent` |

`requires_reboot: true` (correct).

MPO lets DWM hand independent content layers (a video, a game, the desktop) to the display
controller's hardware overlay planes instead of compositing them into one buffer, which saves GPU work
and power. On certain GPU, driver, and mixed-refresh multi-monitor combinations it produced flicker,
black flashes, and frame-time spikes. `OverlayTestMode=5` is the long-standing DWM override; on
Windows 11 24H2 and 25H2 the graphics stack no longer reads it, and the equivalent control moved to
`DisableOverlays` under `GraphicsDrivers`. Both values are absent by default, confirmed live on 24H2:
the `Dwm` key contains no `OverlayTestMode` and no `OverlayMinFPS`, and `GraphicsDrivers` contains no
`DisableOverlays`. The build-gated design is correct in principle and the split point is right.

**Corrections needed:** see correction 20. Two fixes. Add the `OverlayMinFPS = 0` (`REG_DWORD`,
`HKLM\SOFTWARE\Microsoft\Windows\Dwm`) companion to the pre-24H2 path, which the commonly published
recipe pairs with `OverlayTestMode=5`. And close the build gap: the gates `<=22631` and `>=26100`
leave builds 22632 to 26099 with neither effect applied, so the first gate should be `<26100`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off DWM hardware overlays, the fix for MPO screen flicker and frame-time spikes.**

      ## What it does
      Disables Multi-Plane Overlay, the Desktop Window Manager feature that hands video and game
      layers to the display controller's hardware overlay planes. On builds before 24H2 it sets the
      Dwm `OverlayTestMode` to 5; on 24H2 and newer it sets `DisableOverlays` under
      `GraphicsDrivers`, because the older key is no longer read.

      ## Benefits
      - **Stops MPO flicker**: the black flashes and screen flicker that hit some GPU and
        multi-monitor combinations
      - **Stops MPO micro-stutter**: frame-time spikes traceable to overlay plane switching
      - **Build-aware**: applies the value your Windows build actually reads

      ## Drawbacks
      - **Higher GPU load**: DWM now composites everything itself, raising GPU use and power draw
      - **Breaks in-game overlays**: on 24H2 and newer, `DisableOverlays` turns off all hardware
        overlays, which is reported to break Discord, NVIDIA and AMD overlays
      - **No measured gain**: no measurable gain on modern hardware; this is a bug workaround, not a speedup; if you
        do not have the flicker, disabling overlays only costs you

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (uses `DisableOverlays`); older builds use the Dwm
        key instead
      - **Takes effect**: after reboot
      - **Reverting**: removes the value, which is the stock state on both paths
      - Verify with `dxdiag`: MPO showing as "Not Supported" or `MaxPlanes: 1` confirms it took

      ## Recommendation
      Use it only if you actually see MPO flicker or micro-stutter and a GPU driver update did not
      fix it; recent drivers resolved most of these bugs, so update first. If you have no flicker,
      leave overlays on, especially on 24H2 and newer where turning them off can break your Discord
      or GPU overlay.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [What is Multi-Plane Overlay (MPO), NVIDIA KB 5157](https://nvidia.custhelp.com/app/answers/detail/a_id/5157)
      - [MPO fix moved to the GraphicsDrivers path on newer builds](https://github.com/RedDot-3ND7355/MPO-GPU-FIX/issues/26)
```

**Sources:**

1. "Windows 11 25H2 MPO fix moved to GraphicsDrivers path", MPO-GPU-FIX issue 26, https://github.com/RedDot-3ND7355/MPO-GPU-FIX/issues/26 (tier D; states that `OverlayTestMode` and `OverlayMinFps` are ignored on 25H2 and that `DisableOverlays` under `GraphicsDrivers` is the working key)
2. "A Complete Guide to Disable Windows MPO in Win11 24H2", https://www.minitool.com/news/disable-windows-mpo.html (tier D; the `OverlayTestMode` plus `OverlayMinFPS` pairing)
3. What is Multi-Plane Overlay (MPO) in Windows 11, NVIDIA knowledge base answer 5157, https://nvidia.custhelp.com/app/answers/detail/a_id/5157 (tier C; referenced but returned HTTP 403 to every fetch attempt in this pass, so the vendor's current position is recorded as unverified)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `Dwm` key contains no `OverlayTestMode` or `OverlayMinFPS`; `GraphicsDrivers` contains no `DisableOverlays` (tier C)

### `variable_refresh_rate` Variable refresh rate for windowed games

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `vrr_optimize`, addressing a single field inside a packed string.

- Key: `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`
- Value name: `DirectXUserGlobalSettings`
- Type: `REG_SZ`
- Field: `VRROptimizeEnable`, format `kv_semicolon`
- Option "Enabled": `"1"`
- Option "Off (Stock Default)": field removed (`absent`)

`DirectXUserGlobalSettings` is a single `REG_SZ` holding several semicolon-delimited `Name=Value;`
pairs written by Settings > System > Display > Graphics > Default graphics settings. Observed field
set across sources:
`HighPerfAdapter=<VEN&DEV&SUBSYS>;VRROptimizeEnable=0;AutoHDREnable=1;SwapEffectUpgradeEnable=1`.
`VRROptimizeEnable=1` extends OS-level adaptive sync to DirectX 11 and windowed titles that do not
drive VRR natively. The `kv_semicolon` field addressing and the field name are both correct as
authored, and the engine parses and upserts a single field while preserving the others. Requires a
WDDM 2.6 or newer driver and a display and GPU that actually support adaptive sync; it is a no-op on a
fixed-refresh panel. Per-user, no reboot. On the live 24H2 machine the entire
`HKCU\Software\Microsoft\DirectX` key is absent, so the stock state genuinely is "no value at all".

**Corrections needed:** see correction 18. "Off" has two representations and only one is handled. A
machine that never touched the setting has the field absent; a machine where the user turned VRR on
and then off has `VRROptimizeEnable=0` written into the string. The second state matches neither
option and will read as Unknown. The "Off (Stock Default)" option should accept both `absent` and
`"0"`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Extends adaptive sync to DX11 and borderless games so motion stays smooth with less tearing.**

      ## What it does
      Sets the `VRROptimizeEnable` field inside the packed `DirectXUserGlobalSettings` string, which
      is the same value Settings > Display > Graphics writes. It extends the OS-level variable
      refresh rate path to DirectX 11 and windowed games that do not drive adaptive sync themselves.

      ## Benefits
      - **Less tearing**: in windowed and DX11 titles that would not otherwise get adaptive sync
      - **Works alongside G-Sync**: it complements the driver-level toggle rather than replacing it
      - **Free where supported**: no measurable cost on hardware that has VRR

      ## Drawbacks
      - **Needs VRR hardware**: on a fixed-refresh display it does nothing at all
      - **Frame limiter interactions**: some titles have reported oddities when combined with a cap
      - **No FPS change**: this is a smoothness and tearing setting, not a frame rate one

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, with a WDDM 2.6 or newer driver and an
        adaptive-sync capable display
      - **Takes effect**: at the next game launch
      - **Reverting**: restores the previous field value from the snapshot, leaving the other fields
        in the shared string untouched
      - This value shares one registry string with Auto HDR and the windowed-games optimisation, so
        it is edited field by field rather than overwritten

      ## Recommendation
      Turn it on if you have a FreeSync or G-Sync compatible display and play DX11 or borderless
      titles; it is one of the few genuinely free wins in this category. There is nothing to gain on
      a fixed-refresh monitor.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Optimizations for windowed games in Windows 11](https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11)
      - [Enable or disable variable refresh rate for games in Windows 11](https://www.elevenforum.com/t/enable-or-disable-variable-refresh-rate-for-games-in-windows-11.12052/)
```

**Sources:**

1. Optimizations for windowed games in Windows 11, https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11 (tier A; documents the shared Default graphics settings page and the Auto HDR interaction)
2. "Enable or Disable Variable Refresh Rate for Games in Windows 11", https://www.elevenforum.com/t/enable-or-disable-variable-refresh-rate-for-games-in-windows-11.12052/ (tier C; states the value is the `REG_SZ` `DirectXUserGlobalSettings` and that `VRROptimizeEnable` is 0 or 1 inside it)
3. Winhance issue 363, "VRR setting enabled/disabled in Windows Settings is not correctly read", https://github.com/memstechtips/Winhance/issues/363 (tier D; documents both the shared-string clobbering hazard and the `VRROptimizeEnable=0` off-state)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `HKCU\Software\Microsoft\DirectX` key absent (tier C)

### `optimizations_windowed_games` Optimizations for windowed games

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `swap_effect_upgrade`, addressing a single field inside a packed string.

- Key: `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`
- Value name: `DirectXUserGlobalSettings`
- Type: `REG_SZ`
- Field: `SwapEffectUpgradeEnable`, format `kv_semicolon`
- Option "Enabled": `"1"`
- Option "Off (Stock Default)": field removed (`absent`)
- Tweak-level gate: `windows: { build: ">=22621" }` (correct)

Microsoft: "Optimizations for windowed games improves gaming performance for DirectX 10 and DirectX 11
games running in windowed and borderless windowed modes." It upgrades the legacy blt-model
presentation path to the modern flip model (DirectFlip), which lowers present latency and unlocks Auto
HDR and VRR for windowed titles. Windows 11 22H2 (22621) and later; per-user, no reboot; off by
default, and the entire `UserGpuPreferences` key is absent on a clean machine (confirmed live).
Microsoft states that turning on Auto HDR turns this on automatically, and that it cannot be turned
off while Auto HDR is on, so the tweak can be silently overridden.

**Corrections needed:** see correction 19. The info text says current builds "do not expose a single
clean registry value for it, so it is driven through the UI/API rather than a simple key", which
directly contradicts the value the tweak writes; one of the two is wrong and the copy below removes
the claim. Add the Auto HDR interaction to the cautions. And establish whether
`HKCU\Software\Microsoft\DirectX\GraphicsSettings\SwapEffectUpgradeCache` must also be written for the
Settings UI and the runtime to agree, or whether it is only a cache; community scripts that reproduce
the Settings toggle write it alongside.

**Ready-to-paste info block:**

```yaml
    info: |
      **Routes borderless DX10 and DX11 games through DirectFlip for lower input latency.**

      ## What it does
      Sets the `SwapEffectUpgradeEnable` field inside the packed `DirectXUserGlobalSettings` string,
      the same value the Settings > Display > Graphics toggle writes. It upgrades the legacy
      blt-model presentation path to the modern flip model, which is what lowers present latency.

      ## Benefits
      - **Lower input latency**: close to exclusive fullscreen, for borderless DX10 and DX11 titles
      - **Keeps the perks**: fast Alt-Tab, Auto HDR and VRR all keep working
      - **Microsoft-supported**: a documented Windows feature, not a community recipe

      ## Drawbacks
      - **Occasional tearing**: Microsoft acknowledges this and suggests matching frame rate to
        refresh rate or enabling V-Sync
      - **Rare per-title glitches**: a minority of games render incorrectly on the flip path
      - **Can be overridden**: turning on Auto HDR force-enables this and you cannot turn it off
        while Auto HDR is on, so it may read as drifted

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (the feature arrived in 22H2)
      - **Takes effect**: at the next game launch
      - **Reverting**: restores the previous field value from the snapshot, leaving the other fields
        in the shared string untouched
      - This value shares one registry string with Auto HDR and variable refresh rate, so it is
        edited field by field rather than overwritten

      ## Recommendation
      Turn it on if you play DX10 or DX11 games in borderless or windowed mode; this is the closest
      thing in this category to a free latency win, and it is a first-party feature. Turn it off for
      a specific title that tears or renders wrongly with it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Optimizations for windowed games in Windows 11](https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11)
      - [Windows 11 22H2 enables new gaming features](https://www.thurrott.com/windows/windows-11/273204/windows-11-version-22h2-enables-new-gaming-features)
```

**Sources:**

1. Optimizations for windowed games in Windows 11, https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11 (tier A; the feature description, the tearing caveat, and the Auto HDR interaction)
2. "Windows 11 Version 22H2 Enables New Gaming Features", https://www.thurrott.com/windows/windows-11/273204/windows-11-version-22h2-enables-new-gaming-features (tier C; the 22H2 introduction and the default-off state)
3. "Fullscreen Optimizations and HDR", DeepWiki index of shoober420/windows11-scripts, https://deepwiki.com/shoober420/windows11-scripts/5.3-fullscreen-optimizations-and-hdr (tier D; documents `SwapEffectUpgradeCache=1` under `HKCU\Software\Microsoft\DirectX\GraphicsSettings` being written alongside the string)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `HKCU\Software\Microsoft\DirectX` key absent (tier C)

### `disable_gamedvr_capture` Xbox Game Bar background capture (Game DVR)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** three effects.

| Effect id | Key | Value name | Type | "Disabled" | "Enabled (Stock Default)" |
|---|---|---|---|---|---|
| `gamedvr_enabled` | `HKCU\System\GameConfigStore` | `GameDVR_Enabled` | `REG_DWORD` | `0` | `1` |
| `app_capture_enabled` | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `AppCaptureEnabled` | `REG_DWORD` | `0` | `1` |
| `allow_gamedvr_policy` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR` | `AllowGameDVR` | `REG_DWORD` | `0` | `absent` |

`AppCaptureEnabled` is the per-user toggle for Game Bar capture; `GameDVR_Enabled` is the per-user
Game DVR flag in the game config store; `AllowGameDVR` is the machine policy from `GameDVR.admx`,
"Enables or disables Windows Game Recording and Broadcasting", policy path Computer Configuration >
Administrative Templates > Windows Components > Windows Game Recording and Broadcasting, disabled
value `0x00000000`. The ADMX ships from the Windows 10 RTM administrative templates onward and applies
to all Windows 11 builds. No reboot; the change takes effect for the next game session. On the live
24H2 machine the `AllowGameDVR` policy value and the HKCU `AppCaptureEnabled` value both exist but had
been set by the user, so they are not evidence of the defaults; the stock defaults of 1 / 1 / absent
are consistent with tier C reporting but were not independently confirmed.

**Corrections needed:** see correction 17. The info's claim that "The Game Bar overlay itself still
works unless you disable it separately" is misleading: `AllowGameDVR = 0` disables Windows Game
Recording **and Broadcasting** entirely, so manual recording and streaming are gone too, not just
background capture, and the Settings UI is greyed out. Also worth flagging in the copy: the third
effect is a machine-wide policy mixed into a tweak whose other two effects are per-user, so on a
multi-user machine one user applying this affects everyone. The clean-install values of
`GameDVR_Enabled` and `AppCaptureEnabled` remain unconfirmed.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off Game DVR, removing the always-on background recorder from every game session.**

      ## What it does
      Sets the per-user `GameDVR_Enabled` and `AppCaptureEnabled` values to 0 and applies the
      machine policy `AllowGameDVR = 0`. The policy is the Windows Game Recording and Broadcasting
      setting, so it removes recording and streaming, not only the background capture buffer.

      ## Benefits
      - **No background recorder**: a small constant overhead is gone from every game session
      - **Steadier lows**: where the effect shows up, it shows up here rather than in
        average FPS
      - **Machine-wide**: the policy holds regardless of what any user toggles in Settings

      ## Drawbacks
      - **Loses instant replay**: the "record the last 30 seconds" clip feature stops working
      - **Loses manual capture**: the policy disables all Game Recording and
        Broadcasting, and greys out the Settings page
      - **Affects every user**: one effect is a machine policy inside an otherwise per-user tweak
      - **No measured gain**: no measurable gain on modern hardware; the overhead removed is small and no published
        benchmark isolates it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: at the next game session
      - **Reverting**: restores the per-user values and removes the machine policy
      - The Game Bar overlay itself is a separate feature; this removes its capture capability, not
        the overlay

      ## Recommendation
      Worth applying if you never use Game Bar clips and want the recorder out of the way. Skip it if
      you ever hit "record that", because this takes manual capture with it, not just the background
      buffer.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [AllowGameDVR policy (GameDVR.admx)](https://admx.help/?Category=Windows_10_2016&Policy=Microsoft.Policies.GameDVR::AllowGameDVR)
      - [DISA STIG V-220845, Game Recording and Broadcasting must be disabled](https://www.stigviewer.com/stigs/microsoft_windows_10/2025-02-25/finding/V-220845)
```

**Sources:**

1. "Enables or disables Windows Game Recording and Broadcasting" (GameDVR.admx policy reference), https://admx.help/?Category=Windows_10_2016&Policy=Microsoft.Policies.GameDVR::AllowGameDVR (tier A, official ADMX mirror)
2. GameDVR.admx source, https://github.com/mxk/windows-secure-group-policy/blob/main/PolicyDefinitions/GameDVR.admx (tier A mirror)
3. DISA STIG V-220845, Windows 10, Game Recording and Broadcasting must be disabled, https://www.stigviewer.com/stigs/microsoft_windows_10/2025-02-25/finding/V-220845 (tier B)
4. Direct inspection, Windows 11 24H2 build 26100.4061: policy key and `AppCaptureEnabled` present but user-modified (tier C)

### `ultimate_performance_power_plan` Ultimate Performance power plan

**Verdict:** INCORRECT

**Mechanism:**

Two effects. The first is a private state marker:

- Key: `HKCU\Software\MagicXToolbox\State`, value name `UltimatePerformance`, type `REG_DWORD`
- Option "Ultimate Performance": `state: 1`, `ultimate_scheme: run`
- Option "Balanced (Stock Default)": `state: 0`

The second, `ultimate_scheme`, is a `shell: powershell` action.

**WRONG mechanism, as currently authored, verbatim:**

```powershell
# apply
powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 | Out-Null
powercfg /setactive e9a42b02-d5df-448d-aa00-03f14749eb61

# undo
powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e

# probe
if ((powercfg /getactivescheme) -match 'e9a42b02-d5df-448d-aa00-03f14749eb61') { exit 0 } else { exit 1 }
```

`e9a42b02-d5df-448d-aa00-03f14749eb61` is the well-known GUID of the hidden Ultimate Performance
scheme **template**. `powercfg /duplicatescheme <guid>` materialises a copy of that template as a
selectable scheme and prints the copy's GUID, and **the copy receives a new GUID, not the template's**.
Measured on the test machine: after duplication the plan exists as
`4da59277-6cd6-4604-83d4-4afaec3840e0 (Ultimate Performance)` and the template GUID does not appear in
`powercfg /list` at all. So `/setactive e9a42b02-...` fails, and the probe, which matches
`/getactivescheme` against that same template GUID, can never return 0. `| Out-Null` then discards
both the GUID and any error text, so the failure surfaces as success, which violates the project's
did-it-work contract.

**CORRECTED mechanism, verbatim:**

```powershell
# apply
$line = (powercfg /list) | Where-Object { $_ -match 'Ultimate Performance' } | Select-Object -First 1
if (-not $line) {
  $out = powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61
  if ($LASTEXITCODE -ne 0) { Write-Error ($out -join "`n"); exit 1 }
  $line = $out -join ' '
}
$guid = [regex]::Match($line, '[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}').Value
if (-not $guid) { Write-Error 'Could not determine the Ultimate Performance scheme GUID'; exit 1 }
powercfg /setactive $guid
if ($LASTEXITCODE -ne 0) { exit 1 }

# undo
# $PriorScheme is the GUID captured from `powercfg /getactivescheme` BEFORE apply ran,
# stored in the snapshot. Never hardcode Balanced.
powercfg /setactive $PriorScheme
if ($LASTEXITCODE -ne 0) { exit 1 }
$dup = (powercfg /list) | Where-Object { $_ -match 'Ultimate Performance' } | Select-Object -First 1
if ($dup) {
  $g = [regex]::Match($dup, '[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}').Value
  if ($g) { powercfg /delete $g }
}

# probe
# Compare against the GUID recorded at apply time. $AppliedScheme comes from the snapshot.
$active = powercfg /getactivescheme
if ($active -match [regex]::Escape($AppliedScheme)) { exit 0 } else { exit 1 }
```

Note on the probe: matching the **name** "Ultimate Performance" instead of the recorded GUID is
locale-dependent and breaks on a localised Windows, so the GUID comparison is the correct form. The
name match in the apply path is a pragmatic already-exists check and should be paired with a GUID
check where possible.

The scheme itself disables core parking, raises the minimum processor state, and removes most idle
power transitions. The template exists on Windows 10 1803 and later and on Windows 11, on all client
SKUs (the duplication trick works outside Pro for Workstations). Microsoft does not expose it on
battery-powered systems by design, and on Modern Standby hardware `powercfg /list` may show only
Balanced, in which case the plan cannot be selected without further changes. Takes effect immediately;
no reboot, and the YAML correctly sets none.

**Corrections needed:** see corrections 2 and 3. Three fixes, all required before this ships:
capture and activate the GUID that `/duplicatescheme` prints, and detect the already-exists case;
stop swallowing output and errors with `| Out-Null`, because that violates the did-it-work contract;
and have `undo` restore the previously active scheme captured at apply time rather than hardcoding
Balanced, since the test machine's active scheme was an OEM plan
(`85d583c5-... Legion Balance Mode`) that Balanced would silently replace. The undo should also delete
the duplicated scheme so repeated apply/undo cycles do not accumulate copies.

**Ready-to-paste info block:**

```yaml
    info: |
      **Activates the hidden Ultimate Performance power plan, cutting CPU idle transitions on a desktop.**

      ## What it does
      Duplicates the hidden Ultimate Performance scheme template with `powercfg /duplicatescheme`
      and activates the copy. The plan disables core parking, raises the minimum processor state,
      and removes most idle power transitions.

      ## Benefits
      - **Less jitter**: the CPU spends less time ramping cores back up from low-power states
      - **No core parking**: all cores stay available for bursty loads
      - **Instant and reversible**: no reboot, and your previous plan is restored on revert

      ## Drawbacks
      - **Higher idle power**: more heat and louder fans, continuously
      - **Bad on battery**: materially worse runtime on any laptop or handheld
      - **Duplicates High Performance**: the delta against the stock High Performance plan is
        small
      - **No measured gain**: no measurable gain on modern hardware; no average-FPS improvement in any published
        measurement; the honest claim is reduced micro-latency, not more frames

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, desktops on AC power
      - **Takes effect**: immediately
      - **Reverting**: restores the power scheme that was active before, and deletes the duplicated
        plan
      - On Modern Standby hardware the scheme list may show only Balanced, in which case this plan
        cannot be selected

      ## Recommendation
      Worth trying on a desktop on AC power where you want minimum latency and do not mind the extra
      heat and power. On a laptop, or if you already run High Performance, skip it: you are paying
      real power for a difference you are unlikely to measure.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Powercfg command-line options](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options)
      - [PowerDuplicateScheme function](https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-powerduplicatescheme)
```

**Sources:**

1. Powercfg command-line options, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A; `/duplicatescheme`, `/setactive`, `/delete`, `/getactivescheme`)
2. PowerDuplicateScheme function, https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-powerduplicatescheme (tier A; confirms the duplicate receives a newly generated GUID)
3. "Add and Activate Ultimate Performance Profile does not actually Activate", ChrisTitusTech/winutil issue 1260, https://github.com/ChrisTitusTech/winutil/issues/1260 (tier D; an independent report of exactly this defect)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `powercfg /list` shows `4da59277-6cd6-4604-83d4-4afaec3840e0 (Ultimate Performance)` and no `e9a42b02-...` scheme; active scheme was an OEM plan `85d583c5-...` (tier C)

### `disable_power_throttling` CPU power throttling

**Verdict:** VERIFIED-WITH-CORRECTION (mechanism community-corroborated, not Microsoft-documented; the
YAML's user-facing text states the effect backwards and must be rewritten)

**Mechanism:** one effect, `power_throttling_off`.

- Key: `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling` (the subkey must be created; it
  does not exist on a stock machine)
- Value name: `PowerThrottlingOff`
- Type: `REG_DWORD`
- Option "Throttling off": `1`
- Option "Windows managed (Stock Default)": `absent`
- `requires_reboot: true` (correct)

Key path, subkey, value name, type, polarity and the `absent` stock default are all **correct as
authored**. Power Throttling is a Windows 10 1709 feature that places processes Windows judges to be
background or unimportant into the CPU's most energy-efficient operating points. Microsoft does not
document the system-wide registry override, but it is not folklore: direct inspection of
`C:\Windows\System32\ntoskrnl.exe` on 26100.4061 finds the literal UTF-16 strings `PowerThrottling` and
`PowerThrottlingOff` inside the kernel image (also in the LA57 variant `ntkrla57.exe`, and in no other
System32 binary out of 5,418 scanned). The kernel power manager therefore reads a value by exactly
this name on the target build, and four independent tier C write-ups spanning 2017 to the present
agree on key, subkey, name, type and polarity: 1 disables power throttling system-wide, 0 enables it.

**The pitch is INVERTED and must be rewritten.** Microsoft's Quality of Service reference documents the
classification the throttling acts on. For window-owning processes: **In Focus = High**, Visible =
Medium, Minimized or Fully Occluded = Low. A process playing audio is High regardless of window state.
The focused foreground application is therefore **already High QoS and already exempt from EcoQoS
down-clocking on a stock machine**, so the YAML's promised benefit cannot occur. What
`PowerThrottlingOff = 1` actually removes is throttling of **background, minimized and occluded**
work. The registry mechanism is real; the advertised reason for using it is not.

**Corrections needed:** see corrections 24 and 25.

1. **Rewrite the `description` and the `info` text.** Delete "Stop Windows from clocking down
   foreground apps to save power", "stops Windows from down-clocking foreground apps", and
   "Foreground applications are allowed to run at full clocks instead of being throttled". Replace
   with the accurate statement: power throttling applies to background, minimized and occluded
   processes; the focused foreground window is already classified High QoS and is not throttled; this
   value removes the throttling of background work. **Do not repeat the old framing anywhere.**
2. **Add the hardware note.** The feature requires Intel Speed Shift, "available in Intel's 6th-gen
   and beyond Core processors", or an equivalent hardware-managed P-state implementation. On hardware
   without it the value is inert, and the tweak has no `windows` or hardware gate to say so.
3. **Describe the override as community-corroborated**, not Microsoft-documented. Microsoft documents
   the feature and the user-facing controls (the power slider and the per-app "Managed by Windows"
   toggle under Settings > System > Power & battery), not this registry value. There is no contract
   that a future build keeps honouring it, even though 26100 clearly does read it.

No registry defect. Do not change the key, the value name, the type, or the `absent` stock default.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows down-clocking your background work, so minimized jobs keep running at full speed.**

      ## What it does
      Creates the `PowerThrottling` subkey and sets `PowerThrottlingOff` to 1, which turns off
      Windows Power Throttling system-wide. Windows classifies minimized and occluded processes as
      low quality-of-service and parks them in the CPU's most efficient operating points; this
      removes that behaviour.

      ## Benefits
      - **Background jobs unthrottled**: a compile, encode, download or server-style workload in a
        minimized or covered window is no longer clocked down
      - **Window state irrelevant**: work does not slow down just because you switched away
        from it
      - **Cleanly reversible**: removing the value hands the decision back to Windows

      ## Drawbacks
      - **Worse battery life**: the throttling exists to save power, and removing it costs runtime
      - **More heat**: the CPU holds higher operating points, and fans respond, for work you are not
        watching
      - **Nothing for foreground**: Windows already classifies the focused window as high
        priority and never power-throttles it, so this does not speed up what you are using
      - **No measured gain**: no measurable gain on modern hardware; on a desktop already running a high-performance
        power plan, almost no throttling happens to remove

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, on hardware with Intel Speed Shift (6th generation
        Core or newer) or an equivalent hardware P-state implementation; it is inert without it
      - **Takes effect**: after reboot
      - **Reverting**: removes the value, which is the stock state; the subkey does not exist on an
        untouched machine
      - Efficiency-core scheduling on hybrid CPUs is not affected by this value, so it is not a fix
        for "my game ran on an E-core"

      ## Recommendation
      Worth it on a plugged-in machine that runs long background jobs you keep minimized. Skip it on
      battery, and skip it if the goal is a faster foreground app: Windows was never throttling that
      one.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [Quality of Service (the In Focus = High classification)](https://learn.microsoft.com/en-us/windows/win32/procthread/quality-of-service)
      - [Introducing Power Throttling, Windows Insider Blog](https://blogs.windows.com/windows-insider/2017/04/18/introducing-power-throttling/)
```

**Sources:**

1. Quality of Service, Win32 process and thread reference, https://learn.microsoft.com/en-us/windows/win32/procthread/quality-of-service (tier A; the QoS classification table giving In Focus = High, Visible = Medium, Minimized or Fully Occluded = Low, which is what refutes the foreground claim)
2. "Introducing Power Throttling", Windows Insider Blog, https://blogs.windows.com/windows-insider/2017/04/18/introducing-power-throttling/ (tier B; documents the feature and the Speed Shift requirement, not the registry value)
3. Direct binary inspection, Windows 11 24H2 build 26100.4061: `ntoskrnl.exe` and `ntkrla57.exe` contain the UTF-16 strings `PowerThrottling` and `PowerThrottlingOff`; a sweep of 5,418 binaries under `System32` found the name in no other module (tier C, primary measurement)
4. Direct registry inspection, Windows 11 24H2 build 26100.4061: `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling` key absent, confirming the `absent` stock default (tier C, primary measurement)
5. "How to Enable or Disable Power Throttling in Windows 10", TenForums, https://www.tenforums.com/tutorials/99445-how-enable-disable-power-throttling-windows-10-a.html (tier C; Windows 10 1709 era, the registry value)
6. "How To Disable Power Throttling in Windows 11", Winaero, https://winaero.com/disable-power-throttling/ (tier C; Windows 11 era, independent author)
7. "How to Enable or Disable Power Throttling in Windows", NinjaOne, https://www.ninjaone.com/blog/enable-or-disable-power-throttling-in-windows/ (tier C; endpoint-management vendor documentation, independent of the enthusiast sites)
8. "How to Enable, Disable, and Configure Power Throttling in Windows 10", WinBuzzer, https://winbuzzer.com/2020/07/28/how-to-enable-disable-and-configure-power-throttling-in-windows-10-xcxwbt/ (tier C; fourth independent origin, agrees on key, name, type, and polarity)

### `network_throttling_index` Multimedia network throttling

**Verdict:** VERIFIED

**Mechanism:** one effect, `throttling_index`.

- Key: `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`
- Value name: `NetworkThrottlingIndex`
- Type: `REG_DWORD`
- Option "Disabled (0xffffffff)": `4294967295`
- Option "Default (10) (Stock Default)": `10`
- `requires_reboot: true` (correct)

While an MMCSS multimedia task is registered, the networking stack limits the rate at which
non-multimedia packets are indicated up the stack. Microsoft KB 948066 documents this key and value
directly: the throttle exists "because multimedia programs require more resources", the default is 10
packets per millisecond, network throttling "can be completely turned off by setting the value to
FFFFFFFF (hexadecimal)", and a restart is required after changing it. The mechanism dates to Windows
Vista, when the throttle caused network throughput to collapse during audio playback.

**The literal `10` stock default is CORRECT and must not become `absent`.** Windows genuinely ships
`NetworkThrottlingIndex = 10` and `SystemResponsiveness = 20`, both documented in KB 948066 and both
confirmed present out of the box on 26100.4061 (`NetworkThrottlingIndex REG_DWORD 0xa`). Writing
`absent` here would delete a value Windows ships and would itself be the revert bug that
`_harmful-revert.md` warns about in its "guard against the inverse error" section, which names this
exact value. The MMCSS reader is also still shipping: the UTF-16 strings `NetworkThrottlingIndex`,
`SystemResponsiveness` and `SystemProfile` appear inside `C:\Windows\System32\drivers\mmcss.sys` on
26100.4061 and in no other System32 binary out of 5,418 scanned.

**Corrections needed:** `none` on the YAML. See correction 23 for the framing. The key path, value
name, `REG_DWORD` type, the `0xFFFFFFFF` disable value, the literal `10` stock default and
`requires_reboot: true` are all correct and confirmed against a tier A Microsoft KB plus direct
measurement on 26100.4061. Keep the honest framing of the benefit: the cap is real and still present,
but it only binds while an MMCSS multimedia task is registered and only matters on a link fast enough
to be saturated, so it must not be sold as a general throughput or latency improvement.

**Ready-to-paste info block:**

```yaml
    info: |
      **Lifts the legacy 10-packet-per-millisecond network cap that applies while media is playing.**

      ## What it does
      Sets `NetworkThrottlingIndex` to 0xFFFFFFFF, which Microsoft documents as completely turning
      off the Multimedia Class Scheduler throttle. The shipped default is 10, meaning roughly 10
      packets per millisecond while a multimedia task is registered.

      ## Benefits
      - **Removes a ceiling**: on a very fast link saturated during audio or video playback
      - **Documented switch**: Microsoft's own KB gives this value and the disable constant
      - **Cleanly reversible**: the shipped default is a known number, not a guess

      ## Drawbacks
      - **Can hurt playback**: the cap exists to protect glitch-free audio and video under network and
        CPU contention, and removing it can reintroduce that glitching
      - **Not latency**: this has nothing to do with ping, jitter, or packet scheduling for a
        game's own traffic, and it is one of the most reliably mis-sold tweaks in circulation
      - **No measured gain**: no measurable gain on modern hardware; RSS and receive-side coalescing made the Vista-era
        failure mode largely historical, and the tweak is repeated far more often than it is measured

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: after reboot
      - **Reverting**: writes 10 back, which is the value Windows ships; this value is present out of
        the box, not absent
      - The cap only binds while something has registered a multimedia task with MMCSS, so it does
        nothing when no audio or video is playing

      ## Recommendation
      Only worth it if you are saturating a very fast link while media plays and you have measured
      the cap biting. Everyone else should leave it at 10; it will not improve your ping or your
      in-game latency, and removing it can cost you smooth playback under load.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [KB 948066, throttling mechanism for network performance](https://www.freelists.org/post/thin/KB-How-to-use-the-throttling-mechanism-to-control-network-performance-in-Windows-Vista)
      - [Multimedia Class Scheduler Service](https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service)
```

**Sources:**

1. Microsoft KB 948066, "How to use the throttling mechanism to control network performance in Windows Vista" (tier A; documents `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\NetworkThrottlingIndex`, the 10 packets-per-millisecond default, `FFFFFFFF` to disable, and the reboot requirement). The article has been retired from support.microsoft.com; its text is quoted verbatim in the contemporaneous TechNet mailing-list archive at https://www.freelists.org/post/thin/KB-How-to-use-the-throttling-mechanism-to-control-network-performance-in-Windows-Vista
2. Multimedia Class Scheduler Service, https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service (tier A; documents the `SystemProfile` key and `SystemResponsiveness`, and is silent on `NetworkThrottlingIndex`, cited for the key's ownership rather than for the value)
3. Direct binary inspection, Windows 11 24H2 build 26100.4061: `C:\Windows\System32\drivers\mmcss.sys` contains the UTF-16 strings `NetworkThrottlingIndex`, `SystemResponsiveness` and `SystemProfile`; a sweep of 5,418 `System32` binaries found `NetworkThrottlingIndex` in no other module (tier C, primary measurement)
4. Direct registry inspection, Windows 11 24H2 build 26100.4061: `NetworkThrottlingIndex REG_DWORD 0xa` and `SystemResponsiveness REG_DWORD 20` both present out of the box (tier C, primary measurement)
5. TIBCO knowledge article on MMCSS-induced UDP packet loss in Windows Vista and 7, https://support.tibco.com/external/article/90880/ (tier C; independent vendor documentation of the original failure mode and of the same key and values)

### `disable_storage_sense` Storage Sense auto-cleanup

**Verdict:** VERIFIED-WITH-CORRECTION (mechanism correct; the info text describes a stock behaviour
that Microsoft's own documentation contradicts)

**Mechanism:** one effect, `storage_sense_global`.

- Key: `HKLM\SOFTWARE\Policies\Microsoft\Windows\StorageSense`
- Value name: `AllowStorageSenseGlobal`
- Type: `REG_DWORD`
- Option "Disabled": `0`
- Option "Allowed (Stock Default)": `absent`

Confirmed against the shipped `C:\Windows\PolicyDefinitions\StorageSense.admx` on build 26100:
`key="Software\Policies\Microsoft\Windows\StorageSense" valueName="AllowStorageSenseGlobal"`.
Microsoft's Policy CSP documents three states: Enabled (Storage Sense on, cadence "during low free
disk space", the user cannot disable it), Disabled (Storage Sense off, the user cannot enable it), and
Not Configured, the default: "Storage Sense is turned off until the user runs into low disk space or
the user enables it manually", with the user able to configure it in Settings. Windows 10 1903 and
later, and all Windows 11 builds. Microsoft lists the policy for Pro, Enterprise, Education, Windows
SE, and IoT Enterprise at device scope; Home is not in the supported-editions list although the
registry value is read on Home in practice, so treat Home as unverified. No reboot. Confirmed live on
24H2: the `StorageSense` policy key is absent, so the `absent` stock default is correct.

**Corrections needed:** see correction 26. None on the registry mechanism: key, value name, type, the
0 semantics and the `absent` stock default are all correct. **The `info` and `description` text must
be corrected.** Drop the claim that Storage Sense runs "scheduled" cleanups in the background by
default; Microsoft documents the unconfigured state as off until low disk space or manual enablement,
and the default cadence when on as "during low free disk space", not a schedule. Reframe the benefit
as making the off state permanent and un-flippable, so a future Windows setup pass, a user, or an OEM
image cannot turn Storage Sense on later. That is a real benefit; it is a lockout, not the removal of
running background work. Keep the existing Home caveat. Note also that the stock state genuinely
varies: on the 26100.4061 test machine the per-user `StoragePolicy` key had `01` = 1, meaning Storage
Sense had been switched on for that profile, so neither "always off" nor "always on" can be asserted
as the shipped state.

**Ready-to-paste info block:**

```yaml
    info: |
      **Locks Storage Sense off so nothing can turn automatic file deletion back on.**

      ## What it does
      Sets the machine policy `AllowStorageSenseGlobal` to 0. Microsoft documents this state as
      Storage Sense off with the user unable to enable it. Left unconfigured, Storage Sense is off
      until you hit low disk space or switch it on yourself, and when on it runs during low free
      disk space rather than on a schedule.

      ## Benefits
      - **Permanent lockout**: neither a user, an OEM image, nor a later Windows setup pass can turn
        it on
      - **No surprise deletions**: Downloads, temp files and the Recycle Bin stay exactly as you
        leave them
      - **Documented policy**: a shipped ADMX setting with a documented Not Configured state

      ## Drawbacks
      - **You manage space**: Windows will not reclaim anything automatically, even under
        pressure
      - **Settings greyed out**: the Storage Sense controls are locked, which can look like breakage
      - **No speed change**: no performance change of any kind; this is a behaviour control, not a speed tweak, and it
        has no effect on FPS or responsiveness

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, Pro, Enterprise, Education and IoT Enterprise;
        Home reads the value in practice but is not in Microsoft's supported list
      - **Takes effect**: immediately
      - **Reverting**: removes the policy value, returning control to the user
      - The per-user Storage Sense state lives elsewhere, under
        `HKCU\...\StorageSense\Parameters\StoragePolicy`; the machine policy overrides it, so this
        tweak deliberately does not touch it

      ## Recommendation
      Worth applying if you have ever lost a file to an automatic cleanup, or if you want the
      decision pinned so nothing can flip it later. Leave Storage Sense available if you are short on
      disk space and would rather Windows handle it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Storage Policy CSP (AllowStorageSenseGlobal)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-storage)
      - [Group Policy Search, Allow Storage Sense](https://gpsearch.azurewebsites.net/default.aspx?policyid=14518)
```

**Sources:**

1. Storage Policy CSP, `AllowStorageSenseGlobal`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-storage (tier A; the Enabled / Disabled / Not Configured wording, the "during low free disk space" cadence, and the edition list)
2. Shipped ADMX, `C:\Windows\PolicyDefinitions\StorageSense.admx` on build 26100.4061: `key="Software\Policies\Microsoft\Windows\StorageSense" valueName="AllowStorageSenseGlobal"` (tier A, primary)
3. Group Policy Search, "Allow Storage Sense", https://gpsearch.azurewebsites.net/default.aspx?policyid=14518 (tier A)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `HKLM\SOFTWARE\Policies\Microsoft\Windows\StorageSense` key absent; per-user `HKCU\Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy` present with `01` = 1 (tier C)

### `ssd_optimize_trim` SSD TRIM (delete notification)

**Verdict:** INCORRECT

**Mechanism:**

**WRONG mechanism, as currently authored:**

- Key: `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`
- Value name: **`NtfsDisableDeleteNotify`** (does not exist)
- Type: `REG_DWORD`
- Option "TRIM enabled (Stock Default)": `0`
- Option "TRIM disabled": `1`

The value name does not exist, so the tweak writes a value that nothing reads, in either direction.
Applying "TRIM disabled" would not disable TRIM; applying "TRIM enabled" would not enable it; and the
status check reports the tweak's state from a value Windows ignores.

**CORRECTED mechanism:**

| Key | Value name | Type | TRIM enabled (stock) | TRIM disabled |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` | `DisableDeleteNotification` | `REG_DWORD` | `0` (present, not absent) | `1` |
| `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` | `RefsDisableDeleteNotification` | `REG_DWORD` | `0` | `1` (optional second effect, for ReFS coverage) |

Evidence, in increasing order of strength:

- The live FileSystem key on Windows 11 24H2 contains `DisableDeleteNotification REG_DWORD 0x0`. It
  does not contain `NtfsDisableDeleteNotify`, and it does not contain `DisableDeleteNotify` either.
  For comparison, the same key does contain `NtfsDisableLastAccessUpdate`,
  `NtfsDisable8dot3NameCreation`, `NtfsMemoryUsage`, and `RefsDisableLastAccessUpdate`, so the `Ntfs`
  prefix convention is real, just not used for this setting.
- String extraction from `C:\Windows\System32\fsutil.exe` yields exactly five matches for
  `DeleteNotif`: `DisableDeleteNotify`, `DisableDeleteNotification`, `RefsDisableDeleteNotification`,
  `ReFS DisableDeleteNotify`, and `NTFS DisableDeleteNotify`. The first is the command-line keyword;
  the last two are the display strings printed by `fsutil behavior query`; the middle two are the
  registry value names.
- String extraction from `C:\Windows\System32\drivers\ntfs.sys`, the component that actually reads the
  setting, yields only `DisableDeleteNotification` and `DisableDeleteNotificationDrain`.
  `DisableDeleteNotify` does not appear in the driver at all.

The value semantics (0 = TRIM enabled, 1 = TRIM disabled) and the stock default of `0` are both right;
only the name is wrong. Note the default is a **present** `0`, not `absent`, so the revert must write
0 rather than delete the value. Microsoft states that enabling or disabling delete notification does
not require a restart and takes effect at the next unmap command. TRIM is only meaningful on SSD and
NVMe media, and only if the device reports support for it. Microsoft: "For systems using NTFS, trim is
enabled by default unless an administrator disables it."

**Corrections needed:** see correction 1.

1. **Rename the value to `DisableDeleteNotification`.** This is the whole defect. Until it lands, the
   tweak is a no-op and must not ship.
2. Keep the stock default at `0` (present), not `absent`.
3. Consider adding `RefsDisableDeleteNotification` as a second effect if ReFS coverage is wanted.
4. **Remove or implement the Retrim claim.** The info text says the tweak "verifies the scheduled
   optimization task issues a Retrim on detected SSDs rather than a classic defragmentation". Nothing
   in the effects block does that. The relevant task is `\Microsoft\Windows\Defrag\ScheduledDefrag`.
   The copy below removes the claim.
5. Treat the "TRIM disabled" option with care: it is an actively harmful setting to expose, because
   turning TRIM off on an SSD degrades sustained write performance and endurance. Microsoft's stated
   use case for disabling it is a specific class of device that regresses with delete notification on,
   which is rare on consumer hardware.

**Ready-to-paste info block:**

```yaml
    info: |
      **Confirms TRIM is on, so your SSD keeps its sustained write speed and endurance.**

      ## What it does
      Reads and, if needed, repairs `DisableDeleteNotification` under the FileSystem control key.
      With it at 0, Windows tells the drive which blocks are free when files are deleted, which is
      what lets the SSD controller manage free space efficiently.

      ## Benefits
      - **Sustained write speed**: an SSD without TRIM slows down as it fills and stays slow
      - **Better endurance**: the controller does less write amplification when it knows what is free
      - **Repairs broken machines**: fixes the case where a previous tool or image turned TRIM off

      ## Drawbacks
      - **Usually nothing broken**: Windows enables TRIM by default, so on a healthy system
        this confirms a state you already had
      - **No measured gain**: no measurable gain on modern hardware; this is a health check, not a speed boost; expect
        no change if your defaults were already correct
      - **Nothing for HDDs**: TRIM is meaningless outside SSD and NVMe media

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, on SSD or NVMe drives that report TRIM support
      - **Takes effect**: immediately, at the next delete; no restart is required
      - **Reverting**: restores the previous value from the snapshot
      - The opposite setting, turning TRIM off, has a real and negative long-term effect on an SSD;
        Microsoft's only stated reason for it is a rare class of device that regresses with it on

      ## Recommendation
      Safe to apply as a health check on any SSD system. Never select the TRIM-disabled option unless
      your drive vendor has specifically told you to; you would be trading long-term write
      performance for nothing.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [fsutil behavior (disabledeletenotify)](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior)
      - [Optimize-Volume (ReTrim)](https://learn.microsoft.com/en-us/powershell/module/storage/optimize-volume)
```

**Sources:**

1. fsutil behavior, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior (tier A; documents `disabledeletenotify` semantics, the NTFS default, and the no-restart-required behaviour, while conspicuously not naming a registry value for it as it does for `disable8dot3`, `allowextchar`, `disablelastaccess` and `memoryusage`)
2. Optimize-Volume, https://learn.microsoft.com/en-us/powershell/module/storage/optimize-volume (tier A; the `-ReTrim` operation and the fact that scheduled optimization retrims SSDs rather than defragmenting them)
3. Direct inspection, Windows 11 24H2 build 26100.4061: `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` contains `DisableDeleteNotification = 0x0` and no `NtfsDisableDeleteNotify`; `fsutil behavior query disabledeletenotify` reports `NTFS DisableDeleteNotify = 0` and `ReFS DisableDeleteNotify = 0` (tier C)
4. String extraction from `C:\Windows\System32\fsutil.exe` and `C:\Windows\System32\drivers\ntfs.sys`, build 26100.4061 (tier C, direct measurement)

### `ntfs_disable_lastaccess` NTFS last-access timestamp updates

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `disable_lastaccess`.

- Key: `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`
- Value name: `NtfsDisableLastAccessUpdate`
- Type: `REG_DWORD`
- Option "Disabled (user managed)": `0x80000001`
- Option "System managed (Stock Default)": `0x80000002`
- `requires_reboot` is **missing** and must be added

Microsoft's `fsutil behavior` reference names this value directly: "The `disablelastaccess` parameter
reduces the impact of logging updates to the Last Access Time stamp on files and directories ... This
parameter updates the `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem\NtfsDisableLastAccessUpdate`
registry key." Since Windows 10 1803 the value encodes two independent bits plus a high sentinel bit:

| Value | Meaning |
|---|---|
| `0x80000000` | User Managed, last-access updates enabled |
| `0x80000001` | User Managed, last-access updates disabled |
| `0x80000002` | System Managed, last-access updates enabled |
| `0x80000003` | System Managed, last-access updates disabled |

In System Managed mode the NTFS driver decides at boot: updates are enabled when the system volume is
128 GB or smaller and disabled when it is larger. Confirmed live on 24H2:
`NtfsDisableLastAccessUpdate REG_DWORD 0x80000002` present, and
`fsutil behavior query disablelastaccess` returns
`DisableLastAccess = 2 (System Managed, Last Access Time Updates ENABLED)`. So the stock default value
is correct and the value is present rather than absent. Microsoft: "You must restart your computer for
this parameter to take effect."

**Corrections needed:** see corrections 13 and 14.

1. **Add `requires_reboot: true`.** Microsoft states a restart is required and the YAML has no flag.
2. **State the zero-delta case honestly.** On a system volume larger than 128 GB, which is nearly all
   of them, System Managed mode has already disabled the updates, so the tweak's real-world delta is
   often exactly zero. The copy must say that plainly, not imply a saving that is not there.

The value, key, type, semantics, and stock default are all correct as authored.

**Ready-to-paste info block:**

```yaml
    info: |
      **Pins NTFS last-access timestamp updates off instead of leaving the choice to Windows.**

      ## What it does
      Sets `NtfsDisableLastAccessUpdate` to 0x80000001, which is User Managed mode with updates
      disabled. Windows normally ships 0x80000002, System Managed with updates enabled, and then
      decides at boot based on the size of your system volume.

      ## Benefits
      - **Fewer metadata writes**: no "last read" stamp update per file read
      - **Deterministic**: your choice holds regardless of volume size or a later disk change
      - **Documented switch**: the same value `fsutil behavior set disablelastaccess` writes

      ## Drawbacks
      - **Breaks last-access tooling**: Microsoft warns this "can affect programs such as Backup and
        Remote Storage", and forensic and archiving tools rely on it too
      - **Overrides Windows**: a later change of volume layout will not be reflected
      - **No measured gain**: no measurable gain on modern hardware; on any system volume larger than 128 GB, System
        Managed mode has already disabled these updates, so the real-world change is usually exactly
        zero

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: after reboot
      - **Reverting**: writes 0x80000002 back, which is the value Windows ships; this value is
        present out of the box, not absent
      - NTFS defers on-disk last-access writes by up to an hour and answers queries from memory, so
        the write volume being saved is smaller than intuition suggests

      ## Recommendation
      Safe, but do not expect a difference: if your system volume is over 128 GB, Windows already
      stopped these updates. Skip it entirely if you run backup, archiving, or forensic software that
      reads last-access times.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [fsutil behavior (disablelastaccess)](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior)
      - [The Last Access updates are almost back (the four-state encoding and the 128 GB rule)](https://dfir.ru/2018/12/08/the-last-access-updates-are-almost-back/)
```

**Sources:**

1. fsutil behavior, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior (tier A; names the registry value, gives the restart requirement, and gives the Backup and Remote Storage warning)
2. "The Last Access updates are almost back", My DFIR Blog, https://dfir.ru/2018/12/08/the-last-access-updates-are-almost-back/ (tier C; documents the four-state encoding and the 128 GB system volume rule)
3. "Daily Blog #557: Changes in the NtfsDisableLastAccessUpdate key", Hacking Exposed Computer Forensics Blog, https://www.hecfblog.com/2018/12/daily-blog-557-changes-in.html (tier C; the 12/6/18 update confirms the 128 GB threshold)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `NtfsDisableLastAccessUpdate = 0x80000002`, `fsutil behavior query disablelastaccess` returns `DisableLastAccess = 2 (System Managed, Last Access Time Updates ENABLED)` (tier C)

### `disable_memory_compression` RAM memory compression

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two effects. The first is a private state marker:

- Key: `HKCU\Software\MagicXToolbox\State`, value name `MemoryCompression`, type `REG_DWORD`
- Option "Disabled": `state: 1`, `memory_compression: run`
- Option "Enabled (Stock Default)": `state: 0`

The second, `memory_compression`, is a `shell: powershell` action, **verbatim as authored and correct
as authored**:

```powershell
# apply
Disable-MMAgent -MemoryCompression

# undo
Enable-MMAgent -MemoryCompression

# probe
if ((Get-MMAgent).MemoryCompression -eq $false) { exit 0 } else { exit 1 }
```

Windows compresses cold pages in the standby list rather than paging them to disk, storing them in the
Memory Compression process's working set. `Disable-MMAgent` is the documented cmdlet for turning off
memory compression among the MMAgent features (application launch prefetching, operation recorder API,
page combining, application prelaunching, memory compression), and `Get-MMAgent` reports the current
state. The apply, undo and probe are all correct against the documented cmdlet surface, and the probe
genuinely detects the applied state. The MMAgent module ships in-box on Windows 10 and 11, all client
SKUs; the cmdlets require an elevated session, which matches `elevation: admin`. Verified live:
`Get-MMAgent` on the 24H2 machine returned `MemoryCompression : False` (the tweak had been applied
there), confirming the probe's shape.

**Corrections needed:** see correction 22.

1. **The reboot claim is overstated.** `Disable-MMAgent -MemoryCompression` takes effect for newly
   compressed pages immediately; the existing compression store drains over time or at the next boot.
   `requires_reboot: true` is defensible as a "fully applied" signal, but the info's flat "Requires a
   reboot" is not accurate and the copy below states it precisely.
2. The "Enabled (Stock Default)" option lists only `state: 0` and does not list the action, relying on
   the engine convention that an unselected undo-carrying action has its `undo` driven. That
   convention **is** confirmed in the engine (`src-tauri/src/tweaks/engine/apply.rs`, recorded in
   `_cross-category.md` section 2.4 as "not a defect"), so this is correct by design. Recorded so a
   future review does not re-raise it.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off RAM page compression, trading memory headroom for a little less CPU work.**

      ## What it does
      Runs `Disable-MMAgent -MemoryCompression`. Windows normally compresses infrequently used pages
      in RAM, via the Memory Compression process, to fit more data before it has to page to disk.
      With it off, those pages are simply paged out instead.

      ## Benefits
      - **No compression cost**: the compress and decompress work disappears
      - **Simpler memory behaviour**: pages go to the page file rather than into a compressed store
      - **Documented cmdlet**: a supported Microsoft switch with a matching enable command

      ## Drawbacks
      - **Worse under pressure**: on 8 or 16 GB, removing the cushion causes more paging to disk
        and more hard faults, which hurts rather than helps
      - **Earlier out-of-memory**: with a small or absent page file, you hit the ceiling earlier
      - **No measured gain**: no measurable gain on modern hardware; the Memory Compression process normally uses very
        little CPU, and this is one of the most commonly recommended and most commonly harmful tweaks
        in circulation

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, sensibly only with 32 GB of RAM or more
      - **Takes effect**: immediately for newly compressed pages; the existing compressed store
        drains over time or at the next reboot
      - **Reverting**: runs `Enable-MMAgent -MemoryCompression`, restoring the shipped behaviour
      - This is a different MMAgent feature from SysMain prefetching; disabling the SysMain service
        does not disable compression

      ## Recommendation
      Only on a high-RAM machine where you have actually proven compression is costing you something
      measurable. On a typical 8 to 16 GB machine, leave it enabled: disabling it will make things
      worse, not better.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Disable-MMAgent](https://learn.microsoft.com/en-us/powershell/module/mmagent/disable-mmagent)
      - [Enable-MMAgent](https://learn.microsoft.com/en-us/powershell/module/mmagent/enable-mmagent)
```

**Sources:**

1. Disable-MMAgent, https://learn.microsoft.com/en-us/powershell/module/mmagent/disable-mmagent (tier A)
2. Enable-MMAgent, https://learn.microsoft.com/en-us/powershell/module/mmagent/enable-mmagent (tier A)
3. Get-MMAgent, https://learn.microsoft.com/en-us/powershell/module/mmagent/get-mmagent (tier A; the probe surface)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `Get-MMAgent` returns `MemoryCompression : False` (tier C)

### `disable_fast_startup` Fast Startup (hiberboot)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one effect, `hiberboot`.

- Key: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power`
- Value name: `HiberbootEnabled`
- Type: `REG_DWORD`
- Option "Disabled": `0`
- Option "Enabled (Stock Default)": `1`
- `requires_reboot: true` is **wrong**; see corrections

Fast Startup (hiberboot) turns Shut Down into a hybrid operation: user sessions are logged off, then
the kernel session and loaded drivers are hibernated to `hiberfil.sys` and restored on the next
power-on. `HiberbootEnabled = 0` makes Shut Down a true full shutdown, so drivers and kernel state are
reinitialised from scratch each start. It is the registry backing of Control Panel > Power Options >
Choose what the power buttons do > "Turn on fast startup". Windows 8 and later, all client SKUs. The
setting only exists when hibernation is available: `powercfg /h off` removes `hiberfil.sys` and makes
Fast Startup unavailable regardless of this value. Confirmed live on Windows 11 24H2:
`HiberbootEnabled REG_DWORD 0x1` present, so the literal `1` stock default is correct.

**Corrections needed:** see correction 15. `requires_reboot: true` misdescribes the requirement. The
value governs what the **next shutdown** does; a Restart is already a full cold boot and does not
exercise the setting, which is exactly why "restart to fix it" often works when "shut down and power
on" does not. The flag should either be dropped or the UI text should say "takes effect at your next
shutdown". The copy below says the latter.

**Ready-to-paste info block:**

```yaml
    info: |
      **Makes Shut Down a real shutdown, so drivers and kernel state start fresh every time.**

      ## What it does
      Sets `HiberbootEnabled` to 0. Fast Startup normally turns Shut Down into a partial
      hibernation: your session logs off but the kernel and loaded drivers are saved to
      `hiberfil.sys` and restored on the next power-on. With it off, every start reinitialises
      everything.

      ## Benefits
      - **Clears driver state**: GPU and USB quirks that only a true cold boot resolves
      - **Safer for dual-boot**: a hibernated NTFS volume mounted read-write by another OS can be
        corrupted, and this removes that hazard
      - **Honest shutdown**: no surprise when a "shutdown" did not actually reset anything

      ## Drawbacks
      - **Slower cold boot**: a handful of seconds on a modern NVMe machine, more on older hardware
      - **Stability, not speed**: this changes no frame rate and no responsiveness
      - **No measured gain**: no measurable gain on modern hardware; the benefit is the class of problem it prevents,
        not anything you can benchmark

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: at your next shutdown, not after a restart; a Restart is already a full
        cold boot and does not exercise this setting
      - **Reverting**: writes 1 back, which is the value Windows ships; this value is present out of
        the box, not absent
      - If hibernation has been removed with `powercfg /h off`, Fast Startup is already unavailable
        regardless of this value

      ## Recommendation
      Turn it off if you dual-boot, or if you are chasing driver and GPU oddities that clear after a
      real power cycle. If your machine boots cleanly and you value the few seconds, leave it on.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Turn on or off Fast Startup in Windows 11](https://www.elevenforum.com/t/turn-on-or-off-fast-startup-in-windows-11.1212/)
      - [Disabling Windows Fast Startup (HP Wolf Security)](https://support.hpwolf.com/s/article/Disabling-Windows-10-Fast-Startup)
```

**Sources:**

1. "Turn On or Off Fast Startup in Windows 11", https://www.elevenforum.com/t/turn-on-or-off-fast-startup-in-windows-11.1212/ (tier C; key path, value name, and the 0/1 semantics)
2. "Disabling Windows 10 Fast Startup", HP Wolf Security support, https://support.hpwolf.com/s/article/Disabling-Windows-10-Fast-Startup (tier C, independent vendor documentation)
3. Powercfg command-line options, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A; `powercfg /h off` and its effect on the availability of Fast Startup)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `HiberbootEnabled = 0x1` present (tier C)

### `disable_vbs_hvci` Virtualization-Based Security / Memory Integrity (HVCI)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two effects.

| Effect id | Key | Value name | Type | "Disabled" | "Enabled (Stock Default)" |
|---|---|---|---|---|---|
| `enable_vbs` | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `EnableVirtualizationBasedSecurity` | `REG_DWORD` | `0` | `1` (**wrong**, should be `absent`) |
| `hvci_enabled` | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` | `Enabled` | `REG_DWORD` | `0` | `1` (correct only on a clean install that met the auto-enablement bar) |

`requires_reboot: true` (correct). `risk_level: critical` (correct).

VBS uses the hypervisor to create an isolated virtual trust level that hosts the secure kernel. HVCI,
branded Memory Integrity, runs kernel-mode code integrity validation inside that isolated environment,
so an attacker who compromises the normal kernel still cannot load unsigned or tampered kernel code.
Microsoft documents these exact key paths and value names, so the mechanism as authored is correct;
the defect is in the stock default and in what the two values can actually achieve.

Microsoft's OEM guidance states memory integrity is on by default on clean installs of Windows 11 on
hardware meeting the bar (Intel 8th generation or later from 22H2, AMD Zen 2 or later, Qualcomm 8180
or later, 8 GB RAM, 64 GB SSD, compatible drivers, virtualization enabled in firmware) and on all
Secured-core PCs, and that "Auto-enablement pertains only to clean installs, not upgrades of existing
devices."

**Corrections needed:** see corrections 8, 9 and 10.

1. **The Stock Default `enable_vbs: 1` is wrong.** Microsoft's OEM recommended image configuration
   sets `Scenarios\HypervisorEnforcedCodeIntegrity` `Enabled=1`, `WasEnabledBy=1` and `EnabledBootId`.
   It does **not** set `DeviceGuard\EnableVirtualizationBasedSecurity`, which is a policy value that is
   absent unless configured. The stock default should be `absent`. `hvci_enabled: 1` is right on a
   clean install that met the bar and wrong (absent) on an upgrade, so it cannot be a single fixed
   literal either.
2. **Setting both values to 0 does not reliably stop VBS.** Measured on the test machine:
   `EnableVirtualizationBasedSecurity = 0` and HVCI `Enabled = 0` while
   `Win32_DeviceGuard.VirtualizationBasedSecurityStatus = 2` ("enabled and running") and
   `SecurityServicesRunning = {1}` (Credential Guard running). Anything else that needs the hypervisor
   (Hyper-V, WSL2, Windows Sandbox, Credential Guard, Smart App Control, Application Guard) keeps it
   launched; fully stopping it needs `bcdedit /set hypervisorlaunchtype off` as well, which has its
   own consequences. The tweak's name and its promise of recovered CPU-bound FPS both overstate what
   these two values achieve.
3. **`WasEnabledBy` is not handled.** Per Microsoft's registry guidance, deleting it greys out the
   memory integrity UI with "This setting is managed by your administrator". The tweak neither deletes
   nor restores it, so a user who reverts will not necessarily see Windows Security return to normal,
   and a user who re-enables through Windows Security may reach a state the tweak cannot detect.
4. **Cross-reference the conflict with `security:enable_credential_guard`.** Credential Guard is
   VBS-hosted and cannot run without VBS, so applying the security tweak and then this one leaves
   `LsaCfgFlags = 1` requesting a feature the machine can no longer host. The conflict is currently
   masked because `enable_credential_guard` writes `LsaCfgFlags` to the wrong key, and **fixing that
   key activates it**. Handle the dependency in the same change. Minimum: explicit cross-references in
   both `info` blocks and both `warning` fields.
5. If memory integrity was enabled with UEFI lock (`Locked = 1`), the registry change will not take
   effect until Secure Boot is disabled from firmware. Add this to the cautions.

On the benefit: the CPU-bound performance recovery is real, and Microsoft acknowledges the cost
directly ("Older processors rely on an emulation of these features, called Restricted User Mode, and
will have a bigger impact on performance"). The cost is largest on processors without Mode-Based
Execution Control (Intel pre-Kaby Lake) or Guest Mode Execute Trap (AMD pre-Zen 2). Published gaming
benchmarks in the 5 to 15 percent range for 1 percent lows come from technical press rather than tier
A or B sources, so the number is indicative, not established, and the copy must not present it as
measured fact.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off Memory Integrity to recover CPU-bound frame times, at a critical cost to security.**

      ## What it does
      Sets `EnableVirtualizationBasedSecurity` and the `HypervisorEnforcedCodeIntegrity` `Enabled`
      value to 0. HVCI validates kernel-mode code inside a hypervisor-isolated environment, and that
      validation is what costs CPU time on memory and driver operations.

      ## Benefits
      - **Recovers CPU headroom**: the largest effect is on 1 percent lows in CPU-limited games
      - **Older CPUs benefit**: Microsoft states processors without the newer virtualization
        features fall back to emulation and "will have a bigger impact on performance"
      - **Fixes driver blocks**: some older drivers cannot load at all with memory integrity on

      ## Drawbacks
      - **Critical security downgrade**: you lose hypervisor-enforced protection against kernel-mode
        malware and malicious or vulnerable drivers
      - **Breaks Credential Guard**: it is VBS-hosted, so do not combine this with the Credential
        Guard tweak in the security category; the two contradict each other
      - **Anti-cheat and policy**: some anti-cheat systems require memory integrity, and every
        enterprise or compliance policy does
      - **VBS keeps running**: measured on 24H2, the hypervisor stays running for Hyper-V,
        WSL2, Sandbox and Credential Guard, so the recovered performance is less than advertised

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer on hardware meeting the VBS bar
      - **Takes effect**: after reboot
      - **Reverting**: restores the previous values from the snapshot
      - If memory integrity was enabled with UEFI lock, this will not take effect until Secure Boot
        is disabled from firmware
      - The published 5 to 15 percent figures come from technical press, not Microsoft, so treat them
        as indicative rather than measured

      ## Recommendation
      Only consider this on a personal gaming machine where you knowingly accept a weaker security
      posture for CPU headroom, and only after confirming your anti-cheat allows it. Never on a work,
      shared, or sensitive machine, and never alongside Credential Guard.

      ## Evidence
      - **Risk**: critical
      - **Confidence**: Microsoft-documented
      - [Enable memory integrity (the exact registry values)](https://learn.microsoft.com/en-us/windows/security/hardware-security/enable-virtualization-based-protection-of-code-integrity)
      - [Memory integrity and VBS enablement, OEM guidance](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/oem-hvci-enablement)
```

**Sources:**

1. Enable memory integrity, https://learn.microsoft.com/en-us/windows/security/hardware-security/enable-virtualization-based-protection-of-code-integrity (tier A; gives the exact `reg add` commands for `EnableVirtualizationBasedSecurity`, `RequirePlatformSecurityFeatures`, `Locked`, `Scenarios\HypervisorEnforcedCodeIntegrity\Enabled`, and `WasEnabledBy`)
2. Memory integrity and VBS enablement (OEM guidance), https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/oem-hvci-enablement (tier A; default enablement rules, hardware bar, the clean-install-only statement, and the recommended image configuration `Enabled=1`, `WasEnabledBy=1`, `EnabledBootId`)
3. Windows 11 STIG V-253371, virtualization-based protection of code integrity must be enabled, https://www.stigviewer.com/stigs/microsoft_windows_11/2025-05-15/finding/V-253371 (tier B)
4. Direct inspection, Windows 11 24H2 build 26100.4061: `EnableVirtualizationBasedSecurity = 0`, HVCI `Enabled = 0`, `Locked = 0`, yet `VirtualizationBasedSecurityStatus = 2` and `SecurityServicesRunning = {1}` (tier C)

### `disable_spectre_meltdown` Spectre / Meltdown CPU mitigations

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two effects, both under
`HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management`, both `REG_DWORD`.

| Effect id | Value name | "Mitigations disabled" | "Enabled (Stock Default)" |
|---|---|---|---|
| `feature_settings_override` | `FeatureSettingsOverride` | `3` | `absent` |
| `feature_settings_override_mask` | `FeatureSettingsOverrideMask` | `3` | `absent` |

`requires_reboot: true` (correct). `risk_level: critical` (correct).

KB4073119 gives exactly these two `reg add` commands with data 3 and 3 to disable the mitigations for
CVE-2017-5715 (Spectre variant 2, branch target injection) and CVE-2017-5754 (Meltdown, rogue data
cache load). The mask selects which bits of the override are honoured, and the article states that "a
value of 3 is accurate for `FeatureSettingsOverrideMask` for both the 'enable' and 'disable'
settings". So the pair as authored is correct and still current; no Microsoft statement retiring it
was found. Windows 10 and Windows 11, all SKUs, on any machine that received the January 2018 or later
servicing updates. The stock default of `absent` for both is right: these are override values that
only exist once an administrator writes them. On the test machine both were present with value 3, but
only because the tweak had already been applied there.

**Corrections needed:** see corrections 6 and 7.

1. **Relabel the option.** "Mitigations disabled" oversells it. 3/3 disables only the Spectre v2 and
   Meltdown mitigations; MDS, TAA, L1TF, SSBD and Gather Data Sampling have separate bits and are
   untouched. The label should read "Spectre v2 and Meltdown mitigations disabled".
2. **Add a caution about overwriting other bits.** `FeatureSettingsOverride` is one DWORD, so writing
   3 clears any previously configured bit, including the `0x2000000` (33554432) value KB5029778
   specifies for disabling the Gather Data Sampling / "Downfall" mitigation, and KB4073119's own
   `FeatureSettingsOverride=72`. Writing 3 silently re-enables the GDS mitigation, and the revert to
   `absent` deletes an administrator's deliberate setting.
3. **State that the Meltdown half is already a no-op on modern CPUs.** Windows enables KVA Shadow only
   when `IA32_ARCH_CAPABILITIES[RDCL_NO]` is clear. Any CPU reporting RDCL_NO (broadly, Intel from Ice
   Lake / 10th generation onward, and all AMD) never had KVA Shadow enabled, so there is nothing to
   reclaim. Say this rather than framing the payoff as merely "small to none".

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Spectre v2 and Meltdown mitigations to reclaim CPU throughput on older processors.**

      ## What it does
      Writes `FeatureSettingsOverride` and `FeatureSettingsOverrideMask` as 3, the exact pair
      Microsoft's KB4073119 gives for disabling the mitigations for CVE-2017-5715 (Spectre variant 2)
      and CVE-2017-5754 (Meltdown). Nothing else is changed.

      ## Benefits
      - **Reclaims syscall throughput**: the mitigations cost most on syscall-heavy and
        context-switch-heavy workloads
      - **Older CPUs benefit**: machines relying on the software retpoline or IBRS/IBPB paths pay
        the biggest penalty
      - **Microsoft-documented switch**: the same two values Microsoft's own KB publishes

      ## Drawbacks
      - **Critical security downgrade**: re-exposes the machine to side-channel attacks that read
        memory across process and virtual machine boundaries
      - **Two CVEs only**: MDS, TAA, L1TF, SSBD and Downfall use different bits and stay on,
        so this is not "mitigations off"
      - **Clobbers other settings**: this is one DWORD, so writing 3 overwrites any Downfall
        mitigation override an administrator had configured
      - **No measured gain**: no measurable gain on modern hardware; on any CPU reporting RDCL_NO, roughly Intel 10th
        generation onward and all AMD, Meltdown protection was never enabled, so half of this tweak
        has nothing to reclaim

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, with the largest effect on older CPUs lacking
        silicon-level fixes
      - **Takes effect**: after reboot
      - **Reverting**: removes both values, which is the stock state
      - Some benchmarks regress after disabling these, so measure your own workload rather than
        assuming a gain

      ## Recommendation
      Hard to justify on modern hardware: the benefit is small or absent and the risk is real. Only
      an option on an isolated, older, personal machine where you knowingly accept the exposure.
      Never on a shared, work, or virtualization-hosting machine.

      ## Evidence
      - **Risk**: critical
      - **Confidence**: Microsoft-documented
      - [KB4073119, Windows client guidance for speculative execution side-channel vulnerabilities](https://support.microsoft.com/en-us/topic/kb4073119-windows-client-guidance-for-it-pros-to-protect-against-silicon-based-microarchitectural-and-speculative-execution-side-channel-vulnerabilities-35820a8a-ae13-1299-88cc-357f104f5b11)
      - [KVA Shadow: Mitigating Meltdown on Windows (MSRC)](https://www.microsoft.com/en-us/msrc/blog/2018/03/kva-shadow-mitigating-meltdown-on-windows)
```

**Sources:**

1. KB4073119, Windows client guidance for IT Pros to protect against silicon-based microarchitectural and speculative execution side-channel vulnerabilities, https://support.microsoft.com/en-us/topic/kb4073119-windows-client-guidance-for-it-pros-to-protect-against-silicon-based-microarchitectural-and-speculative-execution-side-channel-vulnerabilities-35820a8a-ae13-1299-88cc-357f104f5b11 (tier A; the two values, the data 3 / mask 3 pair, and the `FeatureSettingsOverride=72` variant)
2. KVA Shadow: Mitigating Meltdown on Windows, Microsoft Security Response Center, https://www.microsoft.com/en-us/msrc/blog/2018/03/kva-shadow-mitigating-meltdown-on-windows (tier A; KVA Shadow is enabled only when RDCL_NO is clear)
3. KB5029778, how to manage the vulnerability associated with CVE-2022-40982 ("Downfall" / Gather Data Sampling), which documents `FeatureSettingsOverride = 0x2000000` (tier A, referenced via https://www.elevenforum.com/t/kb5029778-how-to-manage-cve-2022-40982-downfall-cpu-vulnerability.17350/ as a tier D mirror)
4. Direct inspection, Windows 11 24H2 build 26100.4061: both values present with data 3, already tweaked, so not evidence of a default (tier C)

### `disable_mouse_acceleration` Mouse acceleration (Enhance Pointer Precision)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** three effects, all under `HKCU\Control Panel\Mouse`, all **`REG_SZ`**.

| Effect id | Value name | Type | "Acceleration off" | "Default (Stock Default)" |
|---|---|---|---|---|
| `mouse_speed` | `MouseSpeed` | `REG_SZ` | `"0"` | `"1"` |
| `mouse_threshold1` | `MouseThreshold1` | `REG_SZ` | `"0"` | `"6"` |
| `mouse_threshold2` | `MouseThreshold2` | `REG_SZ` | `"0"` | `"10"` |

`requires_reboot: true` is **wrong**; see corrections.

These are the classic Windows pointer ballistics parameters, documented since the NT era.
`MouseThreshold1` and `MouseThreshold2` set the pixel-per-interrupt thresholds at which the system
changes the mouse-to-cursor ratio: if movement exceeds `MouseThreshold1` and `MouseSpeed` is greater
than 0, the cursor moves at twice normal speed; if it exceeds `MouseThreshold2` and `MouseSpeed` is 2,
at four times. Setting all three to 0 gives a strict 1:1 mapping. This is the registry backing of the
"Enhance pointer precision" checkbox. The `REG_SZ` typing is confirmed live on Windows 11 24H2, all
three present as `REG_SZ`, which is worth stressing because writing them as `REG_DWORD` would be
silently ignored; the YAML has the typing right.

**Corrections needed:** see correction 16. `requires_reboot: true` is wrong. The values are read into
the session at sign-in, or applied immediately by `SystemParametersInfo(SPI_SETMOUSE)`. A sign-out is
sufficient, and the info text itself already says "sign out and back in", so the flag and the text
disagree. Separately, the default values `1` / `6` / `10` are consistent across sources including
Microsoft's own legacy KB, but could not be confirmed on a stock machine because the test machine was
already tweaked; confirm on a clean image.

**Ready-to-paste info block:**

```yaml
    info: |
      **Makes pointer movement 1:1 with the mouse, so the same motion always moves the same distance.**

      ## What it does
      Sets `MouseSpeed`, `MouseThreshold1` and `MouseThreshold2` to 0. These are the pointer
      ballistics values behind the "Enhance pointer precision" checkbox; with all three at 0 Windows
      stops changing the mouse-to-cursor ratio based on how fast you move.

      ## Benefits
      - **Predictable aim**: muscle memory maps to a fixed physical distance, which matters in
        aim-heavy shooters
      - **Immediately noticeable**: one of the few tweaks in this category you can feel the moment it
        applies
      - **Cleanly reversible**: three string values with well-known defaults

      ## Drawbacks
      - **More desk movement**: crossing a large or multi-monitor desktop takes a bigger physical
        motion
      - **Often already bypassed**: many games read raw input and ignore Windows ballistics entirely,
        so this may change nothing in-game and only affect the desktop cursor
      - **Not exhaustive**: `SmoothMouseXCurve` and `SmoothMouseYCurve` in the same key define the
        modern acceleration curve and are not touched, so a machine with a custom curve is not fully
        neutralised

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: after sign-out, or immediately if the tool signals the change to Windows
      - **Reverting**: restores the previous values from the snapshot
      - These values are strings, not numbers, in the registry; a tool that writes them as DWORDs is
        silently ignored by Windows

      ## Recommendation
      Strongly recommended for gamers, especially in shooters, where 1:1 movement is a real
      advantage. If you only use the mouse for desktop work across a large screen and like the
      accelerated feel, leave it on.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Q149228, How to Disable Mouse Acceleration (Microsoft KB archive)](https://jeffpar.github.io/kbarchive/kb/149/Q149228/)
      - [SystemParametersInfoW (SPI_SETMOUSE)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow)
```

**Sources:**

1. Q149228, "How to Disable Mouse Acceleration", Microsoft KB archive, https://jeffpar.github.io/kbarchive/kb/149/Q149228/ (tier A content, archived mirror; documents `MouseSpeed`, `MouseThreshold1`, `MouseThreshold2` and the 0-to-disable procedure)
2. SystemParametersInfoW, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow (tier A; `SPI_SETMOUSE` is what applies these values to a running session)
3. "Turn On or Off Enhance Pointer Precision in Windows", https://www.tenforums.com/tutorials/101691-turn-off-enhance-pointer-precision-windows.html (tier C; gives the 1 / 6 / 10 defaults)
4. Direct inspection, Windows 11 24H2 build 26100.4061: all three values present as `REG_SZ` (tier C)

### `reserved_storage_off` Reserved storage (new in this revision)

**Verdict:** VERIFIED

**Mechanism:** a `shell: powershell` action plus a private state marker.

- State marker: `HKCU\Software\MagicXToolbox\State`, value name `ReservedStorage`, type `REG_DWORD`

Action, verbatim:

```powershell
# apply
Set-WindowsReservedStorageState -State Disabled
if ($LASTEXITCODE -ne 0) { exit 1 }

# undo
Set-WindowsReservedStorageState -State Enabled
if ($LASTEXITCODE -ne 0) { exit 1 }

# probe
$state = (Get-WindowsReservedStorageState | Out-String)
if ($state -match 'Disabled') { exit 0 } else { exit 1 }
```

Microsoft Learn documents `Set-WindowsReservedStorageState` in the DISM PowerShell module, with
`-State` taking "either Disabled or Enabled", and documents the failure mode verbatim: "This command
line option is only supported for online Windows images. If reserved storage is in use, it may not be
disabled, and the following error is returned: *This operation is not supported when reserved storage
is in use. Please wait for any servicing operations to complete and then try again later.*" The paired
`Get-WindowsReservedStorageState` gives a real probe.

Reserved storage is enabled automatically on new PCs with 1903 preinstalled and on clean installs, and
is **not** enabled when upgrading from an earlier version. On the LTSC/IoT research image it was
already `Disabled`. Nothing else in the corpus touches reserved storage.

**Corrections needed:** see correction 27. Three framing corrections, all before it ships.

1. **Do not promise "roughly 7 GB".** That figure is unsourced and does not appear on the cmdlet page.
   Reserved storage size varies with installed language packs and optional features. Say "typically
   several GB, varies by configuration" and read the actual figure from the system.
2. **Handle the not-applicable case.** A machine upgraded from an earlier Windows version may never
   have had reserved storage. The probe must report "not applicable" rather than claiming success.
   This is a common upgrade-path outcome, not an LTSC quirk.
3. **Do not sell it as a speed tweak.** This is a disk-space control filed under `performance`, and
   that honesty has to survive into the shipped copy, because everything else in this file promises
   speed.

One open item before shipping: the exact property name returned by `Get-WindowsReservedStorageState`
should be confirmed against the cmdlet's output on 26100 so the probe can compare a property rather
than string-matching the formatted output. The `Out-String` form above is the safe interim.

**Ready-to-paste info block:**

```yaml
    info: |
      **Frees the disk space Windows sets aside for updates, at the cost of update headroom.**

      ## What it does
      Runs `Set-WindowsReservedStorageState -State Disabled`. Reserved storage is a block Windows
      sets aside so feature updates, temporary files and system caches always have room; disabling
      it returns that space to you.

      ## Benefits
      - **Reclaims several GB**: the largest single reclaimable allocation on a stock install
      - **Supported switch**: a first-party DISM cmdlet with a matching enable command, not a
        registry poke
      - **Helps small drives**: on a 128 GB device the reclaimed space is a meaningful fraction

      ## Drawbacks
      - **Updates can fail**: with the reserve gone, an update on a nearly full disk can fail
        or demand you free space manually
      - **Often nothing there**: reserved storage is only enabled on clean installs
        and preinstalled 1903-and-later PCs, never on upgrades, so many machines never had it
      - **No speed change**: no performance change of any kind; this is disk space, not speed; it will not make
        anything faster

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, unless a servicing operation is in progress, in which case
        Windows refuses and the tweak reports the failure rather than claiming success
      - **Reverting**: runs `Set-WindowsReservedStorageState -State Enabled`
      - The amount reclaimed varies with your installed language packs and optional features, so
        check the reported figure rather than expecting a fixed number

      ## Recommendation
      Worth it on a small drive where several GB actually matters and you are willing to manage free
      space around feature updates yourself. On a roomy drive, leave it alone; you are trading update
      reliability for space you are not short of.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Set-WindowsReservedStorageState](https://learn.microsoft.com/en-us/powershell/module/dism/set-windowsreservedstoragestate)
      - [Get-WindowsReservedStorageState](https://learn.microsoft.com/en-us/powershell/module/dism/get-windowsreservedstoragestate)
```

**Sources:**

1. Set-WindowsReservedStorageState, DISM PowerShell module, https://learn.microsoft.com/en-us/powershell/module/dism/set-windowsreservedstoragestate (tier A; the `-State` parameter values, the online-images-only restriction, and the reserved-storage-in-use error text)
2. Get-WindowsReservedStorageState, https://learn.microsoft.com/en-us/powershell/module/dism/get-windowsreservedstoragestate (tier A; the probe surface)
3. "What's new in Windows 10 Enterprise LTSC 2021", Reserved storage section, https://learn.microsoft.com/en-us/windows/whats-new/ltsc/whats-new-windows-10-2021 (tier A; the clean-install-and-preinstalled-only enablement rule, and that upgrades do not get it)
4. Live `Get-WindowsReservedStorageState` on build 26100.4061, and on the LTSC/IoT research image where it reported `Disabled` (tier C)

### `no_index_encrypted_files` Indexing of encrypted files (moved)

**Moved to `security.md`.** This control is about the confidentiality of encrypted file contents, not
performance, and it was assigned to two rebuild agents in error. The authoritative entry, with its
mechanism, corrections and ready-to-paste info block, lives in `security.md`. Nothing is lost here.

## Merge candidates

Retained from the original pass. No merge is recommended without a deliberate decision; each entry
states what granularity would be lost.

**Group 1: MMCSS SystemProfile knobs.** `system_responsiveness` and `network_throttling_index` write
two values in the same key
(`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`), both belong to MMCSS,
both are legacy, both require a reboot, and neither has a measured modern benefit. Proposed shape: one
tweak, "MMCSS legacy tuning", with options "Default (20 / 10)", "Foreground-favoured (10 / 10)", and
"Legacy network cap removed (20 / 0xffffffff)". Granularity lost: a user could no longer remove the
network cap while leaving responsiveness at 20 and also lower responsiveness, since those become
mutually exclusive. Given that neither knob has a demonstrated effect, that loss is cheap. If
independent control is wanted, keep them separate.

**Group 2: DirectX global graphics defaults.** `variable_refresh_rate` and
`optimizations_windowed_games` address two fields of one packed `REG_SZ`
(`HKCU\Software\Microsoft\DirectX\UserGpuPreferences\DirectXUserGlobalSettings`), are set from the
same Settings page, and interact: Microsoft states Auto HDR force-enables the windowed-games
optimisation. Proposed shape: one tweak, "Default graphics settings", with options "Both off (Stock
Default)", "Windowed optimisations on", "Windowed optimisations plus VRR on". Granularity lost: VRR
alone without windowed optimisations becomes unreachable, which matters for a user on a fixed-refresh
display who wants only the latency path. Since the engine already has a `kv_semicolon`
read-modify-write path behind a process-wide mutex, the safer answer is probably to keep them
separate.

**Group 3: CPU security traded for throughput.** `disable_vbs_hvci` and `disable_spectre_meltdown` are
both `risk_level: critical` and both trade protection for CPU throughput, and a user reasoning about
"how much security am I giving up for frames" evaluates them together. Proposed shape: one tweak, "CPU
security mitigations", with options "All on (Stock Default)", "Memory integrity off", "Memory
integrity and Spectre/Meltdown mitigations off". Granularity lost: disabling Spectre/Meltdown
mitigations while keeping memory integrity on becomes unreachable, which is unusual but legitimate on
an older CPU with a compatible driver set. **Recommendation: do not merge.** The two recovery paths
differ enough (one can be undone from Windows Security, the other only from the registry) that
combining them makes the revert story worse.

No merge is proposed for `optimize_visual_effects`, `ssd_optimize_trim`, `ntfs_disable_lastaccess`,
`disable_storage_sense`, `reserved_storage_off`, or `no_index_encrypted_files`. Despite several
touching "disk and shell housekeeping" they are unrelated subsystems, and a user would combine rather
than choose between them.

## Open questions

These could not be established and need a genuinely clean Windows install (ideally a Windows 11 24H2
image untouched by any tweaking tool). Every item below is a **revert value or a semantics question**,
which `_harmful-revert.md` ranks as the corpus's most damaging defect class, so none of them is
cosmetic.

1. **`memory_prefetch_mode`:** the true clean-install start type of `SysMain` on 26100. The YAML
   claims `automatic`, which matches every published reference, but the 26100.4061 machine reports
   `START_TYPE : 4 DISABLED` with `ERROR_CONTROL : 0 IGNORE`, and stock `SysMain` is
   `ERROR_CONTROL : 1 NORMAL`, so that machine has almost certainly been modified and is not usable as
   evidence either way. **This remains open.** For contrast, `WSearch` was observable and is confirmed
   correct: `AUTO_START (DELAYED)` on 26100.4061 with no start or stop triggers, exactly matching the
   YAML's `automatic_delayed`.
2. **`disable_fullscreen_optimizations`:** the true clean-install values of `GameDVR_FSEBehaviorMode`,
   `GameDVR_HonorUserFSEBehaviorMode` and `GameDVR_DXGIHonorFSEWindowsCompatible`, and their real
   semantics. Observed 0 / 0 / 0 on one machine; two tier C/D sources give contradictory enumerations;
   Microsoft documents none of them. Until this is settled the tweak should not ship.
3. **`enable_game_mode`:** whether `HKCU\Software\Microsoft\GameBar\AutoGameModeEnabled` is present
   with value 1 on a clean install, or absent until the user visits Settings > Gaming > Game Mode. The
   "Stock Default = 1" depends on this.
4. **`enable_gpu_scheduling`:** whether `HwSchMode` is present on a clean Windows 11 24H2 install and
   with what value, and whether OEM images differ. Tier C sources say the default is off (1); the one
   machine measured had 2.
5. **`disable_vbs_hvci`:** on a clean Windows 11 24H2 install that met the auto-enablement bar, which
   of `DeviceGuard\EnableVirtualizationBasedSecurity`,
   `Scenarios\HypervisorEnforcedCodeIntegrity\Enabled` and `WasEnabledBy` are actually present, and
   with what values. Microsoft's OEM article gives the recommended image configuration but not what a
   retail clean install produces.
6. **`disable_gamedvr_capture`:** clean-install values of `GameDVR_Enabled` and `AppCaptureEnabled`.
   Assumed 1 / 1 from tier C only.
7. **`disable_mouse_acceleration`:** clean-install values of `MouseSpeed`, `MouseThreshold1`,
   `MouseThreshold2`. Assumed 1 / 6 / 10 from a tier A archive plus tier C, not observed on a stock
   machine.
8. **`optimizations_windowed_games`:** whether
   `HKCU\Software\Microsoft\DirectX\GraphicsSettings\SwapEffectUpgradeCache` must also be written for
   the Settings UI and the runtime to agree, or whether it is only a cache.
9. **`disable_multiplane_overlay`:** NVIDIA's own position. Their knowledge base article on MPO
   (answer id 5157) returned HTTP 403 to every fetch attempt. Someone should open it in a browser and
   record whether NVIDIA still recommends the registry workaround or considers the underlying bug
   fixed in current drivers.
10. **`reserved_storage_off`:** the exact property name returned by `Get-WindowsReservedStorageState`
    on 26100, so the probe can compare a property rather than string-matching formatted output.

Resolved since the original pass, recorded so they are not re-raised: whether the engine runs an
action's `undo` when the user selects an option that omits that action. It does;
`src-tauri/src/tweaks/engine/apply.rs` drives the undo when a target option omits an undo-carrying
action whose probe currently reads present. This closes the question for both
`disable_memory_compression` and `ultimate_performance_power_plan`.
