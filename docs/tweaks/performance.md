# Performance & Gaming tweaks

This category covers system responsiveness, memory, storage, power and gaming or latency controls: shell visual effects, the search indexer and SysMain services, MMCSS tuning, the DirectX and DWM presentation settings, Game Mode and Game DVR, power plans and throttling, NTFS and SSD behaviour, and the two CPU security mitigations people disable for throughput. The supported platform is Windows 11 24H2 (build 26100) and newer, primary, and Windows 10 IoT Enterprise LTSC 2021 (build 19044), secondary; only one tweak here carries a build gate that excludes LTSC 2021, and one other switches registry surface by build. This category attracts performance myths more than any other, so every entry says plainly whether measured evidence supports the performance claim; where it does not, the entry says "no measurable gain on modern hardware" rather than selling an unmeasured number. Everything here is reversible from the snapshot the app takes before the first apply.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Set visual effects to best performance](#set-visual-effects-to-best-performance) | `optimize_visual_effects` | Switch (2 options) | low | none | no | INCORRECT (corrected form ships) |
| [Disable Windows Search indexing](#disable-windows-search-indexing) | `disable_search_indexing` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Block UWP background apps](#block-uwp-background-apps) | `disable_background_apps` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable SysMain (SuperFetch) prefetching](#disable-sysmain-superfetch-prefetching) | `memory_prefetch_mode` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Halve the MMCSS reserved CPU (System Responsiveness)](#halve-the-mmcss-reserved-cpu-system-responsiveness) | `system_responsiveness` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn on Windows Game Mode](#turn-on-windows-game-mode) | `enable_game_mode` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hardware-accelerated GPU scheduling (HAGS)](#hardware-accelerated-gpu-scheduling-hags) | `enable_gpu_scheduling` | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Force exclusive fullscreen in games](#force-exclusive-fullscreen-in-games) | `disable_fullscreen_optimizations` | Switch (2 options) | medium | none | no | INCORRECT (open: value semantics unsettled) |
| [Disable multi-plane overlay (MPO)](#disable-multi-plane-overlay-mpo) | `disable_multiplane_overlay` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Variable refresh rate for windowed games](#variable-refresh-rate-for-windowed-games) | `variable_refresh_rate` | Dropdown (3 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn on optimizations for windowed games](#turn-on-optimizations-for-windowed-games) | `optimizations_windowed_games` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable Xbox Game Bar capture (Game DVR)](#disable-xbox-game-bar-capture-game-dvr) | `disable_gamedvr_capture` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Activate the Ultimate Performance power plan](#activate-the-ultimate-performance-power-plan) | `ultimate_performance_power_plan` | Switch | medium | admin | no | INCORRECT (corrected form ships) |
| [Disable CPU power throttling](#disable-cpu-power-throttling) | `disable_power_throttling` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Lift the multimedia network throttling cap](#lift-the-multimedia-network-throttling-cap) | `network_throttling_index` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Storage Sense auto-cleanup](#disable-storage-sense-auto-cleanup) | `disable_storage_sense` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable SSD TRIM (delete notification)](#enable-ssd-trim-delete-notification) | `ssd_optimize_trim` | Switch (2 options) | low | admin | no | INCORRECT (corrected form ships) |
| [Pin NTFS last-access updates off](#pin-ntfs-last-access-updates-off) | `ntfs_disable_lastaccess` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable RAM memory compression](#disable-ram-memory-compression) | `disable_memory_compression` | Switch | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Fast Startup (hiberboot)](#disable-fast-startup-hiberboot) | `disable_fast_startup` | Switch (2 options) | low | admin | no (next shutdown) | VERIFIED-WITH-CORRECTION |
| [Disable VBS and Memory Integrity (HVCI)](#disable-vbs-and-memory-integrity-hvci) | `disable_vbs_hvci` | Switch (2 options) | critical | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Spectre / Meltdown CPU mitigations](#disable-spectre--meltdown-cpu-mitigations) | `disable_spectre_meltdown` | Switch (2 options) | critical | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable mouse acceleration (Enhance Pointer Precision)](#disable-mouse-acceleration-enhance-pointer-precision) | `disable_mouse_acceleration` | Switch (2 options) | low | none | no (sign-out) | VERIFIED-WITH-CORRECTION |
| [Turn off reserved storage](#turn-off-reserved-storage) | `reserved_storage_off` | Switch | medium | admin | no | VERIFIED |

Elevation `none` means the tweak runs as the signed-in user (the YAML level `user`). Any per-user (HKCU) effect inside an `admin` tweak still runs as the signed-in user, so it lands in your own hive. A tweak with two options renders as a two-position switch; three or more render as a dropdown. In every tweak, **System Default** is never an authored option: it is the status the app shows when the live machine matches none of the options, and selecting it restores the snapshot the app took before the tweak was first applied.

## Tweaks

### Set visual effects to best performance

`optimize_visual_effects` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off desktop fades and shadows so menus and windows appear instantly.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `visual_fx` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects`, value `VisualFXSetting`, `REG_DWORD` |
| `user_preferences_mask` | registry | `HKCU\Control Panel\Desktop`, value `UserPreferencesMask`, `REG_BINARY` |
| `listview_alpha_select` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `ListviewAlphaSelect`, `REG_DWORD` |
| `listview_shadow` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `ListviewShadow`, `REG_DWORD` |
| `refresh_shell` | action (cmd, ephemeral) | `rundll32.exe user32.dll,UpdatePerUserSystemParameters 1, True` |

| Option | `visual_fx` | `user_preferences_mask` | `listview_alpha_select` | `listview_shadow` | `refresh_shell` |
|---|---|---|---|---|---|
| Best performance | `2` | `90,12,03,80,10,00,00,00` | `0` | `0` | run |
| Full visual effects | `absent` | `9e,1e,07,80,12,00,00,00` | `1` | `1` | run |

System Default appears when the four registry values match neither row, for example after you tick individual boxes in the Performance Options dialog (a "Custom" profile). On the one 24H2 machine inspected, the stock state was `VisualFXSetting` absent, `UserPreferencesMask` = `9e,1e,07,80,12,00,00,00`, `ListviewAlphaSelect` = 1 and `ListviewShadow` = 1, which is exactly the "Full visual effects" row, so an untouched machine normally reads as "Full visual effects".

#### How it works

The Performance Options dialog (System Properties > Advanced > Performance > Settings) records which radio button you picked in `VisualFXSetting` (0 = let Windows choose, 1 = best appearance, 2 = best performance, 3 = custom). That value is only a record of the choice: on a live 24H2 machine the `VisualEffects` key held no values at all and 19 subkeys (`AnimateMinMax`, `ComboBoxAnimation`, `CursorShadow`, `DropShadow`, `ListviewAlphaSelect`, `MenuAnimation`, `SelectionFade`, `TaskbarAnimations`, `TooltipAnimation` and others) each holding only `DefaultApplied`. The state the shell actually reads lives elsewhere: `UserPreferencesMask` is a packed bitmask under `Control Panel\Desktop` whose bits drive menu fades and slides, combo-box and list-box animation, tooltip fades, cursor shadow, selection fade and similar UI effects; `ListviewAlphaSelect` controls the translucent selection rectangle in Explorer list views; `ListviewShadow` controls the drop shadow under desktop icon labels. This tweak writes all four so the change is real, then runs `rundll32.exe user32.dll,UpdatePerUserSystemParameters` to ask the running session to reload the per-user parameters. That is not the `SystemParametersInfo` broadcast (`SPI_SETUIEFFECTS` / `SPI_SETANIMATION` with `SPIF_SENDCHANGE`) the research specified, and no cited source confirms that the `rundll32` call reloads a running session; if it does not, the values apply at the next sign-in. The refresh action is ephemeral: it changes no persistent state, so it takes no part in detecting which option is active. Everything is per-user (HKCU), needs no elevation, and applies on every Windows edition. Two effects of the full profile, window minimise and maximise animation (`MinAnimate`) and taskbar animation (`TaskbarAnimations`), are deliberately not written here because another tweak owns those addresses.

#### Benefits
- Menus, tooltips and list selections appear instantly, with no fade or slide.
- The shell stops compositing effects it does not need, which helps weak or integrated GPUs.
- Remote Desktop sessions feel noticeably snappier, since animations are costly to send over the wire.

#### Drawbacks
- The desktop looks flatter: no fades or drop shadows in menus, list views or tooltips.
- It is not the whole profile: window and taskbar animations stay on unless you also apply "Turn off window animations" in the Interface category.
- No FPS change: this is shell chrome and does not touch in-game frame rates.
- No measurable gain on modern hardware; the animations cost close to nothing on a current GPU, so the benefit is how it feels, not what it measures.
- The "Best performance" bitmask replaces the whole `UserPreferencesMask`, so any other bit you had customised in that mask is overwritten until you revert.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: possibly immediately for some effects, if the unverified parameter refresh works; reliably after sign-out.
- **Reverting**: System Default restores the snapshot, including deleting `VisualFXSetting` if it was absent before. Choosing "Full visual effects" instead writes the observed stock mask and list-view values.

#### Interactions
`interface:disable_ui_animations` ("Turn off window animations") owns `MinAnimate` and `TaskbarAnimations`, the two remaining pieces of the "Adjust for best performance" profile. Apply both for the complete profile. The two tweaks share no address, so they never conflict.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The research found that `VisualFXSetting` on its own changes nothing and that its stock state is absent rather than 0. The shipped tweak writes the values the shell reads (`UserPreferencesMask`, `ListviewAlphaSelect`, `ListviewShadow`, with `MinAnimate` and `TaskbarAnimations` owned by `interface:disable_ui_animations`), and its full-effects option deletes `VisualFXSetting`. One part differs from the research's form: the session refresh is a `rundll32` call to `UpdatePerUserSystemParameters`, not the `SystemParametersInfo` broadcast the research specified, and its effect is unverified.
- **Confidence**: community-corroborated, backed by direct inspection. `SystemParametersInfo` is Microsoft-documented as the API the dialog uses, but the registry layout is not documented by Microsoft; the value meanings come from a community write-up and a Microsoft-hosted forum thread, and the stock values from inspection of one 24H2 machine.
- **Reasoning**: the key finding (the radio-button value is a record, not the mechanism) is directly observable on a live machine and matches the forum statement that the registry value alone does not trigger the API call. The exact "Best performance" bitmask is not given by any cited source; it is the value the tweak ships and should be treated as the app's own recipe. Open question: whether the `rundll32` refresh applies the change to a running session.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth it on an older or low-end machine, over Remote Desktop, or if you simply prefer an instant, no-frills desktop. Pair it with "Turn off window animations" for the full effect. Leave it alone on modern hardware if you like the polished look; you are trading appearance for a difference you will feel but not measure.

#### Sources
1. SystemParametersInfoW, Microsoft Learn: the API the Performance Options dialog calls so the running shell applies the change, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow (tier A; the tweak itself does not call this API)
2. fsutil behavior, Microsoft Learn: cited in the tweak as the pattern for tool-written registry values, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior (tier A)
3. "Modify Performance Options and Visual Effects via Registry": the 0/1/2/3 meaning of `VisualFXSetting`, https://mattwv.wordpress.com/2016/06/01/modify-performance-options-and-visual-effects-via-registry/ (tier D)
4. "adjust for best performance via group policy", Microsoft-hosted forum thread: the registry value alone does not trigger the API call the system needs, https://social.technet.microsoft.com/Forums/ie/en-US/39d6e2c3-0c5a-4fe1-98af-9a881d619187/adjust-for-best-performance-via-group-policy (tier D)
5. Direct inspection of Windows 11 24H2 build 26100.4061: `VisualEffects` holds no values and 19 `DefaultApplied` subkeys; `UserPreferencesMask` = `9e,1e,07,80,12,00,00,00`, `ListviewAlphaSelect` = 1, `ListviewShadow` = 1 (tier C)

### Disable Windows Search indexing

`disable_search_indexing` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops the background file indexer, cutting idle disk and CPU activity.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `wsearch` | service | `WSearch` (Windows Search) start type |

| Option | `wsearch` |
|---|---|
| Disabled | `disabled` (start type 4) |
| Automatic, Delayed | `automatic_delayed` (start type 2 with `DelayedAutostart` = 1) |

System Default appears when `WSearch` is in any other start type, such as Manual or plain Automatic. Stock Windows 11 24H2 ships `WSearch` as Automatic (Delayed Start) with no triggers, confirmed on a live machine, so an untouched machine reads as "Automatic, Delayed".

#### How it works

`WSearch` hosts `SearchIndexer.exe`, which crawls your libraries and user folders and builds the property store and full-text index that Start menu search, File Explorer search and any application using the Windows Search API (classic Outlook is the prominent one) query. With the service disabled, nothing maintains the index: Explorer falls back to scanning folders on demand, and Start search finds apps and settings more slowly and less completely. On Windows 11 the taskbar Search flyout is more tightly coupled to this service than on Windows 10 and can return nothing at all rather than merely returning results slowly. The app changes the start type through the Service Control Manager and writes the `DelayedAutostart` companion value to match; it never deletes or installs the service. This is a service setting, so it applies on every edition.

#### Benefits
- No background indexing I/O while you are not using the PC.
- Frees the CPU spikes that indexing causes on large libraries.
- Avoids indexing twice if you already use a third-party search tool such as Everything.

#### Drawbacks
- File search in Explorer becomes a slow on-demand scan.
- Start menu search for apps and settings becomes slower and less complete, and the Windows 11 taskbar flyout can return nothing.
- Classic Outlook search breaks, because it depends on the Windows index.
- No measurable gain on modern hardware; on a healthy SSD machine indexing writes under 20 MB per day and uses under 1 percent idle CPU.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: the start type is written immediately, but the app does not stop a running service, so the indexer keeps running until the next restart; the tweak is flagged as needing a reboot for that reason.
- **Reverting**: System Default restores the start type captured in the snapshot. "Automatic, Delayed" writes the confirmed stock start type. The index rebuilds from scratch after the service returns, which takes a while on a large profile.

#### Interactions
`security:no_index_encrypted_files` ("Do not index encrypted files") sets a Windows Search policy that is moot while this tweak has the indexer disabled, because there is no index at all. `privacy:` tweaks that set Windows Search policies (such as cloud search) are likewise irrelevant while the service is off.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Microsoft-documented. The service and its role are on Microsoft Learn; the stock start type was confirmed live.
- **Reasoning**: the service name, its role and the Automatic (Delayed Start) default were all confirmed by direct inspection on 26100.4061 (`Start` = 2, `DelayedAutoStart` = 1, no triggers) as well as by documentation, so both the apply and the revert value are grounded.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth it on a hard drive, a low-RAM machine, or if you already search with Everything. On a healthy SSD with adequate RAM, leave it enabled; the gain does not justify losing search.

#### Sources
1. Windows Search overview, Microsoft Learn: what the service and index do, https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview (tier A)
2. Guidelines for disabling system services, Microsoft Learn: the service inventory and disabling guidance, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
3. Windows Search (WSearch) service defaults in Windows 11: the default start type, https://revertservice.com/11/wsearch/ (tier C)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `WSearch` `Start` = 2, `DelayedAutoStart` = 1, no triggers (tier C)

### Block UWP background apps

`disable_background_apps` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Store and UWP apps waking to refresh in the background.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `run_in_background` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy`, value `LetAppsRunInBackground`, `REG_DWORD` |

| Option | `run_in_background` |
|---|---|
| Force denied | `2` |
| User in control | `absent` |

System Default appears only if the value holds something else, such as 0 (explicit user control) or 1 (Force Allow) written by another tool or a Group Policy. On a stock machine the `AppPrivacy` policy key does not exist, confirmed live, so an untouched machine reads as "User in control".

#### How it works

This is the registry backing of the Group Policy "Let Windows apps run in the background" (Computer Configuration > Administrative Templates > Windows Components > App Privacy, from `AppPrivacy.admx`), also exposed as the Privacy Policy CSP `LetAppsRunInBackground`. Its values are 0 = user in control, 1 = Force Allow, 2 = Force Deny. Under Force Deny the platform refuses to let packaged (Store and UWP) apps register or run background tasks, so they no longer wake to sync, fetch or refresh live tiles while you are not using them. Classic Win32 desktop programs are not affected. Because the value sits in the Policies hive, the per-app background permission switches in Settings are greyed out while it is set. Microsoft notes that Cortana and Search "might not function as expected" under Force Deny.

#### Benefits
- Less idle CPU: packaged apps stop waking themselves.
- Less network chatter from background fetches.
- Better battery life, the clearest win on laptops and handhelds.

#### Drawbacks
- Push and toast notifications from Mail, Calendar and Store messengers stop or are delayed.
- Live tiles and widgets backed by packaged apps stop updating.
- Microsoft notes Cortana and Search "might not function as expected" under Force Deny.
- The per-app background permission UI in Settings is locked while the policy is set.
- It is a machine policy, so it applies to every user account on the PC.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: immediately; each app picks it up at its next background-task registration.
- **Reverting**: "User in control" deletes the policy value, returning control to Settings. System Default restores whatever the snapshot captured.

#### Interactions
`privacy:disable_app_diagnostics` writes to the same `AppPrivacy` policy key family (a different value), so the two never conflict. None known otherwise.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Microsoft-documented (Policy CSP, the shipped ADMX, and Microsoft's connection-management guidance).
- **Reasoning**: the key, value, type and 0/1/2 enumeration all come from tier A sources, and the absent stock state was confirmed live. The adversarial policy-hive audit confirmed the policy is Machine class and belongs under HKLM, which is where the tweak writes it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Good for most people who do not live inside Store apps and want less background activity, and close to mandatory on a battery-limited machine. Skip it if you rely on real-time alerts from Mail or a Store messenger.

#### Sources
1. Privacy Policy CSP, `LetAppsRunInBackground`, Microsoft Learn: values and behaviour, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy (tier A)
2. Manage connections from Windows to Microsoft services, Microsoft Learn: the policy in Microsoft's own traffic-reduction guidance, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. `AppPrivacy.admx`, mirror of the shipped ADMX: `LetAppsRunInBackground` with values 0/1/2, https://github.com/Harvester57/W10-ADMX/blob/master/AppPrivacy.admx (tier A mirror)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` absent (tier C)

### Disable SysMain (SuperFetch) prefetching

`memory_prefetch_mode` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the SysMain preloader, a targeted fix for the case where it pins your disk at 100 percent.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `sysmain` | service | `SysMain` (formerly SuperFetch) start type |

| Option | `sysmain` |
|---|---|
| Disabled | `disabled` (start type 4) |
| Automatic | `automatic` (start type 2) |

System Default appears when `SysMain` is in any other start type (Manual, Automatic Delayed). The published stock start type is Automatic, which every reference agrees on, but it has not been confirmed on a clean 24H2 image; see Validation.

#### How it works

`SysMain` is the memory management agent service. It profiles which applications you launch and when, and prepopulates the standby list so frequently used binaries and data are already in RAM before you ask for them. It also drives ReadyBoost and has historically driven application prelaunch. The same MMAgent feature set also includes page combining and memory compression, but those are separately switchable through `Disable-MMAgent`, and disabling the service does not turn compression off. Disabling `SysMain` stops the launch profiling, the prefetch reads and the standby-list warming at once; the standby list is then filled only by normal use. The one real-world failure mode it fixes is `SysMain` itself pinning a struggling disk (usually a hard drive) at 100 percent. The app sets the start type through the Service Control Manager.

#### Benefits
- Stops disk thrashing when `SysMain` itself is the process pinning the disk at 100 percent.
- No launch-pattern tracking and no standby-list warming.
- Quieter idle on a spinning drive that is already struggling.

#### Drawbacks
- Applications cold-launch from disk instead of from a warmed cache.
- ReadyBoost stops working (irrelevant on any modern machine).
- No measurable gain on modern hardware; on an SSD with adequate RAM, `SysMain`'s cost is negligible and no published measurement shows an FPS benefit from disabling it.
- Microsoft recommends leaving it on, and the "Automatic" restore value is the published default rather than one observed on a clean image.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: the start type is written immediately, but the app does not stop a running service, so `SysMain` keeps running until the next restart; the tweak is flagged as needing a reboot for that reason. Choosing Automatic likewise starts the service only at the next boot.
- **Reverting**: System Default restores the start type captured in the snapshot, which is the safest path because it is your machine's real prior state. "Automatic" writes the published default; treat it as best effort, since it has not been confirmed on a clean 24H2 image.

#### Interactions
`disable_memory_compression` ("Disable RAM memory compression") is a separate MMAgent feature; applying one does not apply the other. None known otherwise.

#### Validation
- **Verdict**: VERIFIED, with one open question: the clean-install start type of `SysMain` on 26100.
- **Confidence**: Microsoft-documented for the feature and the service; the stock start type is community-published.
- **Reasoning**: the mechanism is uncontroversial. The only machine available to the research reported `SysMain` as Disabled with error control 0 (Ignore), whereas stock `SysMain` uses error control 1 (Normal), which shows that machine had already been modified by another tool and cannot serve as evidence of the default. The revert value is the class of defect the research ranks most damaging, which is why the tweak carries a warning and why restoring from the snapshot is preferred.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Leave it enabled unless you have confirmed that `SysMain` itself is causing sustained 100 percent disk usage on your machine. It is a targeted fix for one failure mode, not a general speedup.

#### Sources
1. Disable-MMAgent, Microsoft Learn: the MMAgent features (launch prefetching, prelaunch, page combining, memory compression) as separable switches, https://learn.microsoft.com/en-us/powershell/module/mmagent/disable-mmagent (tier A)
2. Guidelines for disabling system services, Microsoft Learn: the service inventory, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
3. "How to Configure SuperFetch (SysMain) in Windows", NinjaOne: the service's role and the Automatic default, https://www.ninjaone.com/blog/how-to-configure-superfetch-in-windows/ (tier C)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `START_TYPE 4 DISABLED`, `ERROR_CONTROL 0 IGNORE`, already modified and not evidence of the default (tier C)

### Halve the MMCSS reserved CPU (System Responsiveness)

`system_responsiveness` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Lowers the CPU share MMCSS holds back for background work while audio or video is playing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `responsiveness` | registry | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`, value `SystemResponsiveness`, `REG_DWORD` |

| Option | `responsiveness` |
|---|---|
| Reduced (10) | `10` |
| Reserve 20 percent | `20` |

System Default appears when the value is anything other than 10 or 20 (for example 0 or 100 from another tweaking tool). Windows ships the value present with data 20, confirmed live, so an untouched machine reads as "Reserve 20 percent".

#### How it works

The Multimedia Class Scheduler Service (MMCSS, `mmcss.sys`) boosts the priority of threads that register as multimedia tasks through `AvSetMmThreadCharacteristics`, such as the audio engine and media playback. Microsoft documents `SystemResponsiveness` as "the percentage of CPU resources that should be guaranteed to low-priority tasks": at 20, MMCSS holds back 20 percent of CPU time for non-multimedia work while a multimedia thread is registered. Setting 10 halves that reservation, giving registered multimedia threads more headroom under contention. The rules are strict: values not divisible by 10 round down, values below 10 and above 100 are clamped to 20 (so writing 0 gives you the default, not "reserve nothing"), and 100 disables MMCSS entirely. The reservation only exists while some thread has registered an MMCSS task, which in practice means while audio or video is playing. The value is read at boot, and it is not a policy value.

#### Benefits
- Registered multimedia threads get a larger CPU share under heavy load.
- Useful on audio workstations, the one setup where the reservation is worth tuning.
- A single documented DWORD with a documented default, cleanly reversible.

#### Drawbacks
- Less CPU guaranteed to low-priority work can show up as glitches in a background stream or recording under heavy load.
- It does nothing unless something has registered an MMCSS task.
- No measurable gain on modern hardware; no published measurement shows an FPS or frame-time effect from this value.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after a reboot.
- **Reverting**: "Reserve 20 percent" writes 20, the shipped value; the value is never deleted, because Windows ships it present. System Default restores the snapshot.

#### Interactions
`network_throttling_index` ("Lift the multimedia network throttling cap") writes a sibling value under the same `SystemProfile` key. They are independent; the research considered merging them and recommended keeping them separate so each can be set on its own.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the behaviour of low values: anything below 10 clamps to 20, so 0 is the default rather than "no reservation", and 100 disables MMCSS.
- **Confidence**: Microsoft-documented.
- **Reasoning**: the value semantics, clamping and the 100-disables rule are quoted from Microsoft Learn; the 20 default was confirmed present out of the box on 26100.4061, and `mmcss.sys` on that build still contains the value name. The claim that 10 is a "documented pro-audio setting" has no tier A or B source and is not made here.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth setting on an audio production machine where you want the audio engine to hold priority under load. Gamers should skip it: MMCSS governs multimedia thread priority, not your game's render thread, and there is no measurement showing a frame rate effect.

#### Sources
1. Multimedia Class Scheduler Service, Microsoft Learn: `SystemResponsiveness` semantics, the clamping rule and 100-disables-MMCSS, https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service (tier A)
2. AvSetMmThreadCharacteristicsW, Microsoft Learn: the reservation binds only while a task is registered, https://learn.microsoft.com/en-us/windows/win32/api/avrt/nf-avrt-avsetmmthreadcharacteristicsw (tier A)
3. Direct inspection of Windows 11 24H2 build 26100.4061: `SystemResponsiveness` = 0x14 present out of the box (tier C)

### Turn on Windows Game Mode

`enable_game_mode` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Lets Windows push background work out of the way while you play, smoothing your 1 percent lows.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `auto_game_mode` | registry | `HKCU\Software\Microsoft\GameBar`, value `AutoGameModeEnabled`, `REG_DWORD` |

| Option | `auto_game_mode` |
|---|---|
| On | `1` |
| Off | `0` |

System Default appears when the value is absent or holds anything else. Whether a clean install has the value present with 1, or absent until you first visit Settings > Gaming > Game Mode, is not established.

#### How it works

`AutoGameModeEnabled` is the per-user registry backing of the Settings > Gaming > Game Mode switch. When it is on and Windows detects a game process, it applies a scheduling and resource profile to that process: background work such as Windows Update and indexing is deprioritised or held off, and the game gets more consistent CPU and GPU scheduling. Windows keeps per-game profile data (for example `Win32_AutoGameModeDefaultProfile` and `Win32_GameModeRelatedProcesses`) under `HKCU\System\GameConfigStore`, which this tweak does not touch. The value is read when a game starts. The reported benefit is steadier frame times (better 1 percent lows), not a higher average frame rate.

#### Benefits
- Steadier frame times; the reported gain is in 1 percent lows, not average FPS.
- Fewer mid-game hitches from background updates and indexing.
- A per-user switch that mirrors the Settings toggle exactly.

#### Drawbacks
- A download, compile or encode running alongside the game gets less CPU.
- A few games have historically behaved worse with it on.
- No measurable gain on modern hardware; no tier A or B source publishes a benchmark, and community measurements of the average-FPS effect land inside run-to-run noise.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: at the next game launch.
- **Reverting**: System Default restores the snapshot, which is the only way back to an absent value. "Off" writes 0 rather than deleting it.

#### Interactions
`debloat:remove_xbox_game_bar` uninstalls the Game Bar overlay app but does not touch Game Mode; the two are independent. `disable_gamedvr_capture` and `disable_fullscreen_optimizations` write other values in `GameConfigStore`, not this one.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that Game Mode is driven by `AutoGameModeEnabled` alone; a companion value, `AllowAutoGameMode`, sometimes cited alongside it, does not exist on a live 24H2 machine and is not written.
- **Confidence**: community-corroborated. Microsoft documents the feature but not the registry value.
- **Reasoning**: the value name and 0/1 meaning come from a community source and were observed on a live 24H2 machine (where the user had turned it off). The clean-install state remains an open question; because both options are real user choices and neither claims to be the stock state, the open question affects only what an untouched machine displays, not what the tweak writes.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Leave it on. It is the safest tweak in this category, and the failure mode is a single title behaving oddly, which you can then handle per game. Turn it off if you stream or encode while playing and want those processes to keep their share.

#### Sources
1. Game Mode in Windows, Microsoft Support: the feature and what it prioritises (not the registry value), https://support.microsoft.com/en-us/windows/game-mode-in-windows-2ffe7e17-2c1b-c0a1-c5cb-ecb98efd41ae (tier A)
2. "How to Turn On or Off Game Mode in Windows 10 & 11", MajorGeeks: the value name and 0/1 semantics, https://www.majorgeeks.com/content/page/how_to_turn_on_or_off_game_mode_in_windows_10.html (tier D)
3. Direct inspection of Windows 11 24H2 build 26100.4061: `HKCU\Software\Microsoft\GameBar` holds `ShowStartupPanel`, `AutoGameModeEnabled`, `UseNexusForGameBarEnabled`, `UseNexusForGameMode`, and no `AllowAutoGameMode` (tier C)

### Hardware-accelerated GPU scheduling (HAGS)

`enable_gpu_scheduling` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Hands GPU frame scheduling to the GPU itself, and unlocks NVIDIA Frame Generation.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hw_sch_mode` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers`, value `HwSchMode`, `REG_DWORD` |

| Option | `hw_sch_mode` |
|---|---|
| Enabled | `2` |
| Disabled | `1` |
| Let the system decide | `absent` |

System Default appears only for an unusual value such as an explicit 0. The factory state varies by OEM image and GPU driver: the machine inspected had 2, community sources report 1 as the Windows default, and value-absent is a real shipped state on some images, which is why all three are offered and none is called the default.

#### How it works

`HwSchMode` backs Settings > System > Display > Graphics > Default graphics settings > Hardware-accelerated GPU scheduling. Its states are 1 = off, 2 = on, and 0 or absent = let the system and driver decide. With HAGS on, GPU work submission and frame-queue management move from the CPU-side WDDM scheduler to a dedicated scheduling processor on the GPU, trimming CPU overhead for submitting GPU work. It requires Windows 10 2004 (build 19041) or later and a WDDM 2.7 or newer driver; the Settings toggle is hidden on unsupported hardware, where the value has no effect. The graphics kernel reads it at boot. GPU driver installers can rewrite it. NVIDIA DLSS Frame Generation will not run with HAGS off, which is the strongest concrete reason to enable it.

#### Benefits
- Enables NVIDIA DLSS Frame Generation, which requires HAGS.
- Marginally less CPU work per frame on a CPU-limited system.
- Mirrors the Settings toggle exactly, plus an explicit "let the system decide" state.

#### Drawbacks
- Slightly more VRAM for the GPU-side scheduler.
- Some VR users report stutter regressions with it on.
- No measurable gain on modern hardware; outside Frame Generation, published average-FPS testing puts the effect inside run-to-run noise.
- Neither "Enabled" nor "Disabled" is guaranteed to be how your machine shipped.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, on a GPU with a WDDM 2.7 or newer driver.
- **Takes effect**: after a reboot.
- **Reverting**: System Default restores the value captured in the snapshot, which is the correct way back to your factory state. If you are unsure of that state, prefer the snapshot or Settings over picking an option. A GPU driver reinstall may rewrite the value independently.

#### Interactions
None known in the corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found that no single value is a universal stock default (1, 2 and absent all occur), so the tweak offers "Let the system decide" (absent) instead of claiming one literal as the default.
- **Confidence**: community-corroborated.
- **Reasoning**: the key, value and 1/2 semantics are consistent across independent community sources and observed live (`HwSchMode` = 2 alongside a WDDM 3.1 class `DxgKrnlVersion`). The factory value on a clean 24H2 install, and how OEM images differ, remains open.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Turn it on if you want DLSS Frame Generation or you are CPU-limited; that is a concrete reason. If you use VR or have no specific problem to solve, leave it where your machine shipped: you will not measure the difference.

#### Sources
1. Hardware-accelerated GPU scheduling, Microsoft Support: the feature, cited in the tweak, https://support.microsoft.com/en-us/windows/hardware-accelerated-gpu-scheduling-4f5ba4a0-4b26-4f4e-8b4b-0a1e9a2b4d55
2. "How to Enable Hardware-Accelerated GPU Scheduling in Windows 10 and 11", How-To Geek: key path, value name and 1/2 semantics, https://www.howtogeek.com/756935/how-to-enable-hardware-accelerated-gpu-scheduling-in-windows-11/ (tier C)
3. "Should you enable hardware-accelerated GPU scheduling in Windows 11?", PCWorld: measurement discussion and the default-off claim, https://www.pcworld.com/article/2339130/should-you-enable-hardware-accelerated-gpu-scheduling-in-windows-11.html (tier C)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `HwSchMode` = 0x2 present, `DxgKrnlVersion` = 0x11007 (tier C)

### Force exclusive fullscreen in games

`disable_fullscreen_optimizations` · Switch (2 options) · Risk: medium · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Forces games out of the modern compositing path into legacy exclusive fullscreen.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `fse_behavior` | registry | `HKCU\System\GameConfigStore`, value `GameDVR_FSEBehaviorMode`, `REG_DWORD` |
| `honor_user_fse` | registry | `HKCU\System\GameConfigStore`, value `GameDVR_HonorUserFSEBehaviorMode`, `REG_DWORD` |
| `dxgi_honor_fse` | registry | `HKCU\System\GameConfigStore`, value `GameDVR_DXGIHonorFSEWindowsCompatible`, `REG_DWORD` |

| Option | `fse_behavior` | `honor_user_fse` | `dxgi_honor_fse` |
|---|---|---|---|
| Exclusive fullscreen forced | `2` | `1` | `1` |
| Fullscreen optimizations on | `0` | `0` | `0` |

System Default appears for any other combination (for example values another tool wrote). On an untouched 24H2 machine all three were present with data 0 (along with `GameDVR_EFSEFeatureFlags` = 0), which is the "Fullscreen optimizations on" row.

#### How it works

Fullscreen Optimizations is the Desktop Window Manager path that runs a game which asked for exclusive fullscreen as a borderless flip-model window instead, so that fast Alt-Tab, overlays, Auto HDR and variable refresh rate keep working while latency stays close to true exclusive mode. The three `GameConfigStore` values are the community-canonical global switch for forcing the legacy exclusive path instead; DirectX reads them per user when a game starts. Microsoft documents none of the three, and community sources disagree on what each number means: one says `GameDVR_FSEBehaviorMode` 0 = high-impact games only and 1 = all fullscreen games, another says 1 = enable and 2 = disable. The tweak's "forced" recipe (2 / 1 / 1) matches one of the two conflicting community recipes (the one where 2 = disable), and its other option writes the values observed on an untouched machine. This is a global per-user override applied to every game, whereas the per-game "Disable fullscreen optimizations" checkbox in an executable's compatibility properties targets one title.

#### Benefits
- A troubleshooting lever for a specific title that stutters or tears in borderless mode.
- The game bypasses the compositor and gets direct control of the display.
- Per-user, instant, no reboot.

#### Drawbacks
- Breaks Auto HDR, variable refresh rate and fast Alt-Tab, which all depend on the compositing path this bypasses.
- Black-screen flicker on task switch is a common result.
- Wrong granularity: a global override for every game, where the per-game compatibility setting is targeted.
- No measurable gain on modern hardware; on current flip-model DWM the exclusive path is usually equal to or worse than the optimised one, and no measurement shows a general win.
- Undocumented values with disputed meanings; verify the behaviour yourself per title.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the `GameConfigStore` key exists from Windows 10 1703 onward).
- **Takes effect**: at the next game launch.
- **Reverting**: "Fullscreen optimizations on" writes the observed stock 0 / 0 / 0. System Default restores the snapshot.

#### Interactions
`disable_gamedvr_capture` writes `GameDVR_Enabled` in the same `GameConfigStore` key (a different value). `variable_refresh_rate` and `optimizations_windowed_games` configure features this tweak disables in practice (VRR and the flip-model path), so forcing exclusive fullscreen undermines both. Windows' own per-game profile data in `GameConfigStore` is untouched.

#### Validation
- **Verdict**: INCORRECT as researched. The round-trip defect is fixed, but the research's condition for shipping (clean-install values and the meaning of each number on an untouched 24H2 image) is still unmet. The research required the two options to differ on `GameDVR_FSEBehaviorMode` and the restore option to write the observed stock values; the shipped options are 2 / 1 / 1 and 0 / 0 / 0. The meaning of each number remains disputed, because the sources conflict.
- **Confidence**: community-corroborated only in the loose sense; the sources that exist disagree, and Microsoft documents none of the values.
- **Reasoning**: the stock values rest on direct inspection of one machine that had never had the tweak applied. The research made shipping conditional on establishing the clean-install values and the semantics of each number on an untouched 24H2 image. That condition is still unmet. The tweak ships with a warning that says so, and as a per-title troubleshooting tool rather than a recommendation.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Only for troubleshooting one specific game that stutters in borderless mode, and only after trying that game's own fullscreen setting and the per-executable compatibility checkbox first. Everyone else should leave it off; you are trading Auto HDR, VRR and fast Alt-Tab for an effect nobody has measured.

#### Sources
1. Optimizations for windowed games in Windows 11, Microsoft Support: the modern presentation path these values bypass and what depends on it, https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11 (tier A)
2. Microsoft Q&A, "enable or disable Full-screen optimizations on Windows 11 (REGEDIT)": covers `GameDVR_FSEBehavior` only and leaves the question about these three values unanswered, https://learn.microsoft.com/en-us/answers/questions/4079856/enable-or-disable-full-screen-optimizations-on-win (tier D)
3. "Fullscreen Optimizations and HDR", DeepWiki index of shoober420/windows11-scripts: shows that community recipes disagree, https://deepwiki.com/shoober420/windows11-scripts/5.3-fullscreen-optimizations-and-hdr (tier D)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `GameDVR_FSEBehaviorMode` = 0, `GameDVR_HonorUserFSEBehaviorMode` = 0, `GameDVR_DXGIHonorFSEWindowsCompatible` = 0, `GameDVR_EFSEFeatureFlags` = 0 on a machine never tweaked this way (tier C)

### Disable multi-plane overlay (MPO)

`disable_multiplane_overlay` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds (the surface switches at build 26100) · Reversible: yes

**Turns off DWM hardware overlays, the fix for MPO screen flicker and frame-time spikes.**

#### What it changes

| Effect | Kind | Target | Build gate |
|---|---|---|---|
| `overlay_test_mode` | registry | `HKLM\SOFTWARE\Microsoft\Windows\Dwm`, value `OverlayTestMode`, `REG_DWORD` | 26099 and below |
| `overlay_min_fps` | registry | `HKLM\SOFTWARE\Microsoft\Windows\Dwm`, value `OverlayMinFPS`, `REG_DWORD` | 26099 and below |
| `disable_overlays` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers`, value `DisableOverlays`, `REG_DWORD` | 26100 and above |

| Option | `overlay_test_mode` | `overlay_min_fps` | `disable_overlays` |
|---|---|---|---|
| Overlays disabled | `5` | `0` | `1` |
| Enabled | `absent` | `absent` | `absent` |

Only the effects in scope for your build are written or read: on 24H2 and newer that is `DisableOverlays` alone; on LTSC 2021 (19044) it is the `Dwm` pair. System Default appears if the in-scope values hold anything else. All three values are absent on a stock machine (confirmed live on 24H2), so an untouched machine reads as "Enabled".

#### How it works

Multi-Plane Overlay lets the Desktop Window Manager hand independent content layers (a video, a game swap chain, the desktop) to the display controller's hardware overlay planes instead of compositing them into one buffer, which saves GPU work and power. On certain GPU, driver and mixed-refresh multi-monitor combinations it produced flicker, black flashes and frame-time spikes. The long-standing override is `OverlayTestMode` = 5 under the `Dwm` key, commonly paired with `OverlayMinFPS` = 0. On Windows 11 24H2 and 25H2 the graphics stack no longer reads the `Dwm` pair, and the working control is `DisableOverlays` = 1 under `GraphicsDrivers`, which turns off all hardware overlays rather than just MPO. The tweak therefore uses effect-level build gates that meet exactly at 26100, so every supported build gets the value it reads. Both paths are read at boot. You can confirm the result in `dxdiag`, where MPO showing as not supported or `MaxPlanes: 1` means it took.

#### Benefits
- Stops the black flashes and flicker that hit some GPU and multi-monitor combinations.
- Stops frame-time spikes traceable to overlay plane switching.
- Build-aware: applies the value your Windows build actually reads.

#### Drawbacks
- DWM composites everything itself, raising GPU load and power draw.
- On 24H2 and newer, `DisableOverlays` turns off all hardware overlays, which is reported to break Discord, NVIDIA and AMD in-game overlays.
- No measurable gain on modern hardware; this is a bug workaround, not a speedup, and without the flicker it only costs you.
- Undocumented by Microsoft.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build; 24H2 and newer use `DisableOverlays`, older builds (LTSC 2021) use the `Dwm` pair.
- **Takes effect**: after a reboot.
- **Reverting**: "Enabled" deletes the values, which is the stock state on both paths. System Default restores the snapshot.

#### Interactions
None known in the corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research added the `OverlayMinFPS` = 0 companion that the published pre-24H2 recipe pairs with `OverlayTestMode` = 5, and required the two build ranges to meet at 26100 with no gap; the shipped gates are 26099-and-below and 26100-and-above.
- **Confidence**: community-corroborated.
- **Reasoning**: the move from the `Dwm` key to `GraphicsDrivers` on newer builds rests on a tier D issue report plus a tier D guide; the absent stock state was confirmed live. NVIDIA's own knowledge-base article returned HTTP 403 to every fetch attempt, so the vendor's current position is unverified, and whether current drivers have fixed the underlying bug remains open.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Use it only if you actually see MPO flicker or micro-stutter and a GPU driver update did not fix it; recent drivers resolved most of these bugs, so update first. If you have no flicker, leave overlays on, especially on 24H2 and newer where turning them off can break your Discord or GPU overlay.

#### Sources
1. "Windows 11 25H2 MPO fix moved to GraphicsDrivers path", MPO-GPU-FIX issue 26: `OverlayTestMode` and `OverlayMinFps` ignored on newer builds, `DisableOverlays` under `GraphicsDrivers` works, https://github.com/RedDot-3ND7355/MPO-GPU-FIX/issues/26 (tier D)
2. "A Complete Guide to Disable Windows MPO in Win11 24H2", MiniTool: the `OverlayTestMode` plus `OverlayMinFPS` pairing, https://www.minitool.com/news/disable-windows-mpo.html (tier D)
3. What is Multi-Plane Overlay (MPO) in Windows 11, NVIDIA KB 5157: referenced, but returned HTTP 403 to every fetch, so unverified, https://nvidia.custhelp.com/app/answers/detail/a_id/5157 (tier C)
4. Direct inspection of Windows 11 24H2 build 26100.4061: no `OverlayTestMode` or `OverlayMinFPS` under `Dwm`, no `DisableOverlays` under `GraphicsDrivers` (tier C)

### Variable refresh rate for windowed games

`variable_refresh_rate` · Dropdown (3 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Extends adaptive sync to DX11 and borderless games so motion stays smooth with less tearing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `vrr_optimize` | registry field | `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, value `DirectXUserGlobalSettings` (`REG_SZ`), field `VRROptimizeEnable` in `kv_semicolon` format |

| Option | `vrr_optimize` |
|---|---|
| Enabled | `"1"` |
| Off | `"0"` |
| Driver decides | field `absent` |

System Default appears only when the field holds some other value. An unparseable string reads as Unknown and is not rewritten. On a stock machine the whole `HKCU\Software\Microsoft\DirectX` key is absent, confirmed live, so an untouched machine reads as "Driver decides".

#### How it works

`DirectXUserGlobalSettings` is one `REG_SZ` holding several semicolon-delimited `Name=Value;` pairs written by Settings > System > Display > Graphics > Default graphics settings, for example `HighPerfAdapter=...;VRROptimizeEnable=0;AutoHDREnable=1;SwapEffectUpgradeEnable=1`. `VRROptimizeEnable=1` extends the OS-level variable refresh rate path to DirectX 11 and windowed or borderless games that do not drive adaptive sync themselves; DirectX reads it per user when a game starts. The app edits only this field: it reads the live string, upserts or removes `VRROptimizeEnable`, and writes the string back with every other field and their order preserved, under a process-wide lock so two tweaks writing different fields never race. "Off" has two real representations: Settings writes `VRROptimizeEnable=0` once you have ever turned the feature on and then off, while a machine that never touched it has no field at all. Both are offered as options so neither state reads as Unknown. It needs a WDDM 2.6 or newer driver and an adaptive-sync capable display and GPU; on a fixed-refresh panel it does nothing.

#### Benefits
- Less tearing in windowed and DX11 titles that would not otherwise get adaptive sync.
- Complements the driver-level G-Sync or FreeSync setting rather than replacing it.
- No measurable cost on hardware that has VRR.

#### Drawbacks
- Needs VRR hardware; on a fixed-refresh display it does nothing.
- Some titles have reported oddities when combined with a frame cap.
- No FPS change: this is a smoothness and tearing setting, not a frame rate one.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, with a WDDM 2.6 or newer driver and an adaptive-sync display. Behaviour on LTSC 2021 was not separately researched.
- **Takes effect**: at the next game launch.
- **Reverting**: System Default restores the field captured in the snapshot, leaving the other fields in the string untouched. "Driver decides" removes the field.

#### Interactions
`optimizations_windowed_games` edits the `SwapEffectUpgradeEnable` field of the same string, and Auto HDR (set in Settings) lives there too. Because both tweaks address fields rather than the whole value, they coexist safely. `disable_fullscreen_optimizations` forces the exclusive path that bypasses the compositor, which undermines OS-level VRR for affected games.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found that "off" has two representations (field absent, and `VRROptimizeEnable=0` written by Settings after an on-then-off); the tweak offers both.
- **Confidence**: community-corroborated.
- **Reasoning**: Microsoft documents the Settings page and the Auto HDR interaction; the value, field name and 0/1 meaning come from community sources, including an issue report documenting both the shared-string clobbering hazard and the 0 off-state. The absent stock key was confirmed live. Field-level editing avoids the clobbering hazard.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Turn it on if you have a FreeSync or G-Sync compatible display and play DX11 or borderless titles; it is one of the few genuinely free wins in this category. There is nothing to gain on a fixed-refresh monitor.

#### Sources
1. Optimizations for windowed games in Windows 11, Microsoft Support: the shared Default graphics settings page and the Auto HDR interaction, https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11 (tier A)
2. "Enable or Disable Variable Refresh Rate for Games in Windows 11", ElevenForum: the `REG_SZ` `DirectXUserGlobalSettings` and the 0/1 field, https://www.elevenforum.com/t/enable-or-disable-variable-refresh-rate-for-games-in-windows-11.12052/ (tier C)
3. Winhance issue 363, "VRR setting enabled/disabled in Windows Settings is not correctly read": the shared-string clobbering hazard and the `VRROptimizeEnable=0` off-state, https://github.com/memstechtips/Winhance/issues/363 (tier D)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `HKCU\Software\Microsoft\DirectX` absent (tier C)

### Turn on optimizations for windowed games

`optimizations_windowed_games` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 22621 and newer (Windows 11 22H2+) · Reversible: yes

**Routes borderless DX10 and DX11 games through DirectFlip for lower input latency.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `swap_effect_upgrade` | registry field | `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, value `DirectXUserGlobalSettings` (`REG_SZ`), field `SwapEffectUpgradeEnable` in `kv_semicolon` format |

| Option | `swap_effect_upgrade` |
|---|---|
| Enabled | `"1"` |
| Off | field `absent` |

System Default appears when the field holds another value, for example `SwapEffectUpgradeEnable=0` written by Settings. The feature is off by default and the whole `UserGpuPreferences` key is absent on a clean machine (confirmed live), so an untouched machine reads as "Off".

#### How it works

Microsoft: "Optimizations for windowed games improves gaming performance for DirectX 10 and DirectX 11 games running in windowed and borderless windowed modes." It upgrades the legacy blt-model presentation path those games use to the modern flip model (DirectFlip), which lowers present latency to near exclusive-fullscreen levels and lets Auto HDR and VRR work in windowed titles. The field is the same one the Settings > System > Display > Graphics > Default graphics settings toggle writes, and it is read when a game starts. The app edits only this field of the packed string and preserves the rest. Microsoft states that turning on Auto HDR turns this on automatically and that it cannot be turned off while Auto HDR is on, so Windows can override the "Off" state. Community scripts that reproduce the Settings toggle also write `SwapEffectUpgradeCache` = 1 under `HKCU\Software\Microsoft\DirectX\GraphicsSettings`; this tweak does not, and whether that value is required or merely a cache is not established. The feature arrived in Windows 11 22H2, so the tweak is gated to build 22621 and newer and is not listed on LTSC 2021.

#### Benefits
- Lower input latency, close to exclusive fullscreen, for borderless DX10 and DX11 titles.
- Keeps fast Alt-Tab, Auto HDR and VRR working.
- A documented first-party Windows feature.

#### Drawbacks
- Occasional tearing; Microsoft acknowledges it and suggests matching frame rate to refresh rate or enabling V-Sync.
- A minority of games render incorrectly on the flip path.
- Auto HDR force-enables it and blocks turning it off, so the tweak may read as drifted while Auto HDR is on.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 (build 22621) and newer, including all 24H2 and 25H2 builds; unavailable on LTSC 2021.
- **Takes effect**: at the next game launch.
- **Reverting**: "Off" removes the field; System Default restores the field captured in the snapshot. Other fields in the string are never touched.

#### Interactions
`variable_refresh_rate` edits the `VRROptimizeEnable` field of the same string; Auto HDR (set in Settings) lives there too and overrides this one. Field-level editing lets both tweaks coexist. `disable_fullscreen_optimizations` pushes games to the exclusive path instead.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research confirmed that the feature is backed by a single registry field (contradicting an older claim that it could only be driven through the UI), and added the Auto HDR override to the cautions.
- **Confidence**: community-corroborated; the feature is Microsoft-documented, the field name is community-sourced.
- **Reasoning**: the feature description, tearing caveat and Auto HDR interaction are tier A; the 22H2 introduction and default-off state are tier C; the absent key was confirmed live. Open question: whether `SwapEffectUpgradeCache` must be written for the Settings UI and the runtime to agree.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Turn it on if you play DX10 or DX11 games in borderless or windowed mode; this is the closest thing in this category to a free latency win, and it is a first-party feature. Turn it off for a specific title that tears or renders wrongly with it.

#### Sources
1. Optimizations for windowed games in Windows 11, Microsoft Support: the feature, the tearing caveat and the Auto HDR interaction, https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11 (tier A)
2. "Windows 11 Version 22H2 Enables New Gaming Features", Thurrott: the 22H2 introduction and the default-off state, https://www.thurrott.com/windows/windows-11/273204/windows-11-version-22h2-enables-new-gaming-features (tier C)
3. "Fullscreen Optimizations and HDR", DeepWiki index of shoober420/windows11-scripts: `SwapEffectUpgradeCache` = 1 written alongside the string, https://deepwiki.com/shoober420/windows11-scripts/5.3-fullscreen-optimizations-and-hdr (tier D)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `HKCU\Software\Microsoft\DirectX` absent (tier C)

### Disable Xbox Game Bar capture (Game DVR)

`disable_gamedvr_capture` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off Game DVR, removing the always-on background recorder from every game session.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `gamedvr_enabled` | registry | `HKCU\System\GameConfigStore`, value `GameDVR_Enabled`, `REG_DWORD` |
| `app_capture_enabled` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR`, value `AppCaptureEnabled`, `REG_DWORD` |
| `allow_gamedvr_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR`, value `AllowGameDVR`, `REG_DWORD` |

| Option | `gamedvr_enabled` | `app_capture_enabled` | `allow_gamedvr_policy` |
|---|---|---|---|
| Disabled | `0` | `0` | `0` |
| Enabled | `1` | `1` | `absent` |

System Default appears for any other combination. The stock values are taken to be 1 / 1 / absent from community reporting; they were not independently confirmed on a clean image, so an untouched machine may read as System Default if Windows ships either per-user value absent.

#### How it works

Three layers control Game DVR. `AppCaptureEnabled` is the per-user switch for Game Bar capture (Settings > Gaming > Captures). `GameDVR_Enabled` is the per-user Game DVR flag in the game configuration store. `AllowGameDVR` is the machine policy from `GameDVR.admx`, "Enables or disables Windows Game Recording and Broadcasting" (Computer Configuration > Administrative Templates > Windows Components > Windows Game Recording and Broadcasting). Setting the policy to 0 disables Windows game recording and broadcasting entirely, not only the background "record what happened" buffer: manual recording and streaming are gone too, and the Settings capture page is greyed out. The per-user values keep the recorder off for your account even if the policy is later removed. The Game Bar overlay itself (Win+G) is a separate app and still opens; it just cannot capture. The policy is Machine class, so it lives in HKLM and affects every user; the two per-user values land in your own hive even though the tweak runs elevated.

#### Benefits
- Removes the small constant overhead of the background recorder from every game session.
- Where the effect shows up at all, it shows up in steadier lows rather than average FPS.
- The machine policy holds regardless of what any user toggles in Settings.

#### Drawbacks
- Loses instant replay ("record the last 30 seconds").
- Loses manual capture and broadcasting too; the policy disables all Game Recording and Broadcasting and greys out the Settings page.
- One effect is a machine policy inside an otherwise per-user tweak, so it affects every user of the PC.
- No measurable gain on modern hardware; the overhead removed is small and no published benchmark isolates it.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition (the ADMX ships from the Windows 10 RTM templates onward).
- **Takes effect**: at the next game session.
- **Reverting**: "Enabled" writes 1 to both per-user values and deletes the machine policy. System Default restores the snapshot, which is the more faithful path if your per-user values were absent before.

#### Interactions
`debloat:remove_xbox_game_bar` uninstalls the overlay app; this tweak disables capture but keeps the overlay. `disable_fullscreen_optimizations` writes other values in `HKCU\System\GameConfigStore`. `enable_game_mode` is unaffected.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that `AllowGameDVR` = 0 removes all recording and broadcasting, not only background capture, and that describing the overlay as still working is misleading, because capture through it is gone.
- **Confidence**: Microsoft-documented for the policy (ADMX and a DISA STIG); the per-user values are community-corroborated.
- **Reasoning**: the policy key, value and disabled value are quoted from the ADMX. The adversarial policy-hive audit confirmed `AllowGameDVR` is Machine class and correctly written under HKLM. On the inspected machine both the policy and `AppCaptureEnabled` had been set by the user, so the clean-install values of the two per-user values remain open.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying if you never use Game Bar clips and want the recorder out of the way. Skip it if you ever hit "record that", because this takes manual capture with it, not just the background buffer.

#### Sources
1. "Enables or disables Windows Game Recording and Broadcasting", GameDVR.admx policy reference: key, value and disabled value, https://admx.help/?Category=Windows_10_2016&Policy=Microsoft.Policies.GameDVR::AllowGameDVR (tier A, official ADMX mirror)
2. GameDVR.admx source: the policy definition, https://github.com/mxk/windows-secure-group-policy/blob/main/PolicyDefinitions/GameDVR.admx (tier A mirror)
3. DISA STIG V-220845, Windows 10, "Game Recording and Broadcasting must be disabled": the policy as a hardening baseline, https://www.stigviewer.com/stigs/microsoft_windows_10/2025-02-25/finding/V-220845 (tier B)
4. Direct inspection of Windows 11 24H2 build 26100.4061: policy key and `AppCaptureEnabled` present but user-modified (tier C)

### Activate the Ultimate Performance power plan

`ultimate_performance_power_plan` · Switch · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Activates the hidden Ultimate Performance power plan, cutting CPU idle transitions on a desktop.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `ultimate_scheme` | action (PowerShell, with undo and probe) | `powercfg` scheme duplication and activation; bookkeeping in `HKLM\SOFTWARE\MagicXToolbox\PowerPlan` (`AppliedScheme`, `PriorScheme`) |

| Option | `ultimate_scheme` |
|---|---|
| Ultimate Performance | run apply |

The action works as follows. **Apply** reads the currently active scheme GUID (`powercfg /getactivescheme`); reuses the GUID it recorded on an earlier apply if that scheme still exists; otherwise runs `powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61` and records the new GUID it prints as `AppliedScheme`. It never reuses a plan it did not create, even one named "Ultimate Performance". It records the prior scheme as `PriorScheme` (unless it is the same plan), then runs `powercfg /setactive` on the Ultimate GUID; any failure exits non-zero. **Undo** does nothing if no `AppliedScheme` is recorded. Otherwise, if the created plan still exists: when it is the active plan, it activates `PriorScheme` (or Balanced, `381b4222-f694-41f0-9685-ff5bb260df2e`, if that plan was deleted since); a plan you switched to yourself is left active. It then deletes the created plan with `powercfg /delete`, failing if either step fails, and clears both bookkeeping values. **Probe** reports the plan as present only when `AppliedScheme` is recorded and `powercfg /getactivescheme` shows it active.

System Default appears whenever the plan this tweak created is not the active one, which includes any machine the tweak has never touched and a machine where you switched plans since; selecting it after applying restores the snapshot, which runs the undo. Windows' stock active plan is Balanced, or an OEM plan on many laptops and prebuilt desktops.

#### How it works

The Ultimate Performance scheme is a hidden power-plan template (GUID `e9a42b02-d5df-448d-aa00-03f14749eb61`) present since Windows 10 1803. The plan disables core parking, raises the minimum processor state and removes most idle power transitions, so cores do not spend time ramping back up from low-power states. The template is not selectable directly: `powercfg /duplicatescheme` materialises a copy, and the copy receives a newly generated GUID, not the template's (confirmed by Microsoft's `PowerDuplicateScheme` documentation and on a live machine, where the copy appeared as `4da59277-...` and the template GUID never appeared in `powercfg /list`). That is why the tweak captures the printed GUID, and why it records the plan that was active before so the undo returns you to your real prior plan (including an OEM plan) instead of assuming Balanced. The probe compares GUIDs rather than plan names, so it works on a localised Windows, and the apply never looks plans up by name: a copy you made yourself (the research machine had one, `4da59277-...`) is neither reused nor deleted. Microsoft does not expose the plan on battery-powered systems by design, and on Modern Standby hardware `powercfg /list` may show only Balanced, in which case the plan cannot be selected. The duplication works on all client editions, not only Pro for Workstations.

#### Benefits
- Less jitter from cores ramping up out of low-power states.
- No core parking; all cores stay available for bursty loads.
- Instant and reversible: no reboot, and your previous plan is restored on revert.

#### Drawbacks
- Higher idle power, more heat and louder fans, continuously.
- Materially worse battery runtime on any laptop or handheld.
- Small delta against the stock High Performance plan.
- No measurable gain on modern hardware; no average-FPS improvement in any published measurement; the honest claim is reduced micro-latency, not more frames.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build; sensible on desktops on AC power. May be unavailable on Modern Standby hardware.
- **Takes effect**: immediately.
- **Reverting**: turning the switch off (System Default) runs the undo: if the created plan is still active, the plan active before the apply is reactivated (Balanced if that plan has since been deleted), and the created Ultimate plan is deleted. Changes you made to the created plan's settings are lost when it is deleted. A failed delete surfaces as Needs Attention.

#### Interactions
Network tweaks that edit power settings (`network:disable_usb_selective_suspend`, `network:disable_wake_timers` and similar) apply to whichever scheme is active when they run; if you apply them while Ultimate is active, they change only the duplicated plan, which reverting this tweak deletes, so apply or re-check them after choosing your plan.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The research found that activating the template GUID can never work (the copy gets a new GUID), and required the apply to capture the new GUID and reuse an existing copy, and the undo to restore the previously active plan rather than Balanced and to delete the copy. The shipped action does all of that, reusing only a copy it created itself. Apply, activation and delete failures all exit non-zero.
- **Confidence**: Microsoft-documented (`powercfg` and `PowerDuplicateScheme` on Microsoft Learn).
- **Reasoning**: the new-GUID behaviour is documented and was reproduced on a live machine; an independent community bug report describes the same failure in another tool. The adversarial probe audit lists this probe among those with the safe polarity: only the proven success path exits 0, and any failure reports "not applied".
- **Tested**: Build validation (schema, ownership and conflict checks); apply, probe, undo and probe again run under Windows PowerShell 5.1 on build 26100 with a user-made "Ultimate Performance" copy present: the tweak created and deleted its own copy, left the user's copy in place and reactivated the prior OEM plan.

#### Recommendation
Worth trying on a desktop on AC power where you want minimum latency and do not mind the extra heat and power. On a laptop, or if you already run High Performance, skip it: you are paying real power for a difference you are unlikely to measure.

#### Sources
1. Powercfg command-line options, Microsoft Learn: `/duplicatescheme`, `/setactive`, `/delete`, `/getactivescheme`, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A)
2. PowerDuplicateScheme function, Microsoft Learn: the duplicate receives a newly generated GUID, https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-powerduplicatescheme (tier A)
3. "Add and Activate Ultimate Performance Profile does not actually Activate", ChrisTitusTech/winutil issue 1260: an independent report of the template-GUID defect, https://github.com/ChrisTitusTech/winutil/issues/1260 (tier D)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `powercfg /list` shows `4da59277-6cd6-4604-83d4-4afaec3840e0 (Ultimate Performance)` and no `e9a42b02-...` scheme; the active scheme was an OEM plan (tier C)

### Disable CPU power throttling

`disable_power_throttling` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows down-clocking your background work, so minimized jobs keep running at full speed.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `power_throttling_off` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling`, value `PowerThrottlingOff`, `REG_DWORD` (the subkey is created) |

| Option | `power_throttling_off` |
|---|---|
| Throttling off | `1` |
| Windows managed | `absent` |

System Default appears only if the value holds something else, such as 0. The `PowerThrottling` subkey does not exist on a stock machine (confirmed live), so an untouched machine reads as "Windows managed".

#### How it works

Power Throttling (Windows 10 1709 and later) places processes Windows judges unimportant into the CPU's most energy-efficient operating points (EcoQoS). The classification it acts on is documented in Microsoft's Quality of Service reference: for window-owning processes, in focus = High, visible = Medium, minimized or fully occluded = Low, and any process playing audio is High. The focused foreground application is therefore already High QoS and is never power-throttled; what this value removes is the throttling of background, minimized and occluded work. The kernel power manager reads `PowerThrottlingOff` at boot: `ntoskrnl.exe` on 26100.4061 contains the strings `PowerThrottling` and `PowerThrottlingOff`, and no other System32 binary does. Setting it to 1 turns power throttling off system-wide. The feature depends on hardware-managed P-states (Intel Speed Shift, 6th-generation Core and newer, or an equivalent), so on hardware without it the value is inert. It does not change how the scheduler places threads on efficiency cores of hybrid CPUs. Microsoft documents the feature and its user-facing controls (the power mode slider and per-app settings), not this registry override.

#### Benefits
- A compile, encode, download or server-style job in a minimized or covered window is no longer clocked down.
- Work does not slow down just because you switched away from it.
- Removing the value hands the decision straight back to Windows.

#### Drawbacks
- Worse battery life; the throttling exists to save power.
- More heat and fan noise for work you are not watching.
- Nothing for the foreground app, which Windows never throttles.
- No measurable gain on modern hardware; on a desktop already running a high-performance power plan, almost no throttling happens to remove.
- Not a fix for "my game ran on an E-core".

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, on hardware with Intel Speed Shift or an equivalent hardware P-state implementation; inert without it.
- **Takes effect**: after a reboot.
- **Reverting**: "Windows managed" deletes the value, which is the stock state. The empty `PowerThrottling` subkey may remain, which is harmless. System Default restores the snapshot.

#### Interactions
`ultimate_performance_power_plan` keeps cores at high performance states regardless, so combined with it this tweak has even less left to remove. None conflicting.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected what the setting affects: power throttling applies to background, minimized and occluded processes, and the focused foreground window is already High QoS and never throttled; the registry mechanism itself needed no change.
- **Confidence**: community-corroborated. Microsoft documents the feature (Insider blog) and the QoS classification (Learn), not the registry value.
- **Reasoning**: four independent community sources spanning 2017 to the present agree on key, subkey, name, type and polarity, and binary inspection shows the 26100 kernel reads a value by exactly that name. There is no Microsoft contract that future builds keep honouring the override.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth it on a plugged-in machine that runs long background jobs you keep minimized. Skip it on battery, and skip it if the goal is a faster foreground app: Windows was never throttling that one.

#### Sources
1. Quality of Service, Microsoft Learn: the QoS classification (in focus = High, visible = Medium, minimized or occluded = Low), https://learn.microsoft.com/en-us/windows/win32/procthread/quality-of-service (tier A)
2. "Introducing Power Throttling", Windows Insider Blog: the feature and the Speed Shift requirement, https://blogs.windows.com/windows-insider/2017/04/18/introducing-power-throttling/ (tier B)
3. Direct binary inspection of Windows 11 24H2 build 26100.4061: `ntoskrnl.exe` and `ntkrla57.exe` contain `PowerThrottling` and `PowerThrottlingOff`; no other of 5,418 System32 binaries does (tier C)
4. Direct registry inspection of Windows 11 24H2 build 26100.4061: `PowerThrottling` key absent (tier C)
5. "How to Enable or Disable Power Throttling in Windows 10", TenForums: the registry value, https://www.tenforums.com/tutorials/99445-how-enable-disable-power-throttling-windows-10-a.html (tier C)
6. "How To Disable Power Throttling in Windows 11", Winaero: independent Windows 11 confirmation, https://winaero.com/disable-power-throttling/ (tier C)
7. "How to Enable or Disable Power Throttling in Windows", NinjaOne: endpoint-management vendor documentation, https://www.ninjaone.com/blog/enable-or-disable-power-throttling-in-windows/ (tier C)
8. "How to Enable, Disable, and Configure Power Throttling in Windows 10", WinBuzzer: a fourth independent origin, https://winbuzzer.com/2020/07/28/how-to-enable-disable-and-configure-power-throttling-in-windows-10-xcxwbt/ (tier C)

### Lift the multimedia network throttling cap

`network_throttling_index` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Lifts the legacy 10-packet-per-millisecond network cap that applies while media is playing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `throttling_index` | registry | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`, value `NetworkThrottlingIndex`, `REG_DWORD` |

| Option | `throttling_index` |
|---|---|
| Disabled (0xffffffff) | `0xffffffff` (4294967295) |
| Throttled (index 10) | `10` |

System Default appears for any other value (for example a different index another tool wrote). Windows ships the value present with data 10 (KB 948066, confirmed live), so an untouched machine reads as "Throttled (index 10)".

#### How it works

While a thread has registered an MMCSS multimedia task (audio or video playback), the networking stack limits the rate at which non-multimedia packets are indicated up the stack, to protect glitch-free playback under network and CPU contention. Microsoft KB 948066 documents the key and value directly: the default is 10 packets per millisecond, throttling "can be completely turned off by setting the value to FFFFFFFF (hexadecimal)", and a restart is required. The mechanism dates to Windows Vista, when the throttle collapsed network throughput during audio playback; receive-side scaling and receive-side coalescing on modern NICs made that failure mode largely historical. The reader still ships: `mmcss.sys` on 26100.4061 contains `NetworkThrottlingIndex`. The cap only binds while a multimedia task is registered and only matters on a link fast enough to be saturated. It has nothing to do with ping, jitter or the scheduling of a game's own packets.

#### Benefits
- Removes a ceiling on a very fast link saturated during audio or video playback.
- A documented switch: Microsoft's own KB gives the value and the disable constant.
- The shipped default is a known number, so the revert is exact.

#### Drawbacks
- Can reintroduce the playback glitching the cap exists to prevent.
- Not a latency tweak: it does nothing for ping, jitter or game traffic, and is one of the most reliably mis-sold tweaks in circulation.
- No measurable gain on modern hardware; the tweak is repeated far more often than it is measured.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after a reboot.
- **Reverting**: "Throttled (index 10)" writes 10 back, the value Windows ships; the value is never deleted, because deleting a value Windows ships present would itself be a revert bug. System Default restores the snapshot.

#### Interactions
`system_responsiveness` writes `SystemResponsiveness` under the same `SystemProfile` key. Setting `SystemResponsiveness` to 100 disables MMCSS entirely, in which case this cap never binds either.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research confirmed the literal 10 revert value is right and must not become `absent`.
- **Confidence**: Microsoft-documented (KB 948066, preserved verbatim in a contemporaneous archive after its retirement from support.microsoft.com).
- **Reasoning**: key, value, type, the `0xFFFFFFFF` constant, the default of 10 and the reboot requirement all come from the KB and were confirmed on 26100.4061, both in the registry (`0xa`) and in `mmcss.sys`. The cross-cutting revert audit names this value explicitly as one that genuinely ships seeded and must keep its literal.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Only worth it if you are saturating a very fast link while media plays and you have measured the cap biting. Everyone else should leave it at 10; it will not improve your ping or your in-game latency, and removing it can cost you smooth playback under load.

#### Sources
1. Microsoft KB 948066, "How to use the throttling mechanism to control network performance in Windows Vista": the key, the 10 default, `FFFFFFFF` to disable, and the reboot requirement; retired from Microsoft's site and quoted verbatim here, https://www.freelists.org/post/thin/KB-How-to-use-the-throttling-mechanism-to-control-network-performance-in-Windows-Vista (tier A)
2. Multimedia Class Scheduler Service, Microsoft Learn: the `SystemProfile` key's ownership (silent on this value), https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service (tier A)
3. Direct binary inspection of Windows 11 24H2 build 26100.4061: `mmcss.sys` contains `NetworkThrottlingIndex`, `SystemResponsiveness` and `SystemProfile`; no other of 5,418 System32 binaries contains `NetworkThrottlingIndex` (tier C)
4. Direct registry inspection of Windows 11 24H2 build 26100.4061: `NetworkThrottlingIndex` = 0xa and `SystemResponsiveness` = 20 present out of the box (tier C)
5. TIBCO knowledge article on MMCSS-induced UDP packet loss in Windows Vista and 7: independent documentation of the original failure mode and the same key, https://support.tibco.com/external/article/90880/ (tier C)

### Disable Storage Sense auto-cleanup

`disable_storage_sense` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Locks Storage Sense off so nothing can turn automatic file deletion back on.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `storage_sense_global` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\StorageSense`, value `AllowStorageSenseGlobal`, `REG_DWORD` |

| Option | `storage_sense_global` |
|---|---|
| Disabled | `0` |
| Allowed | `absent` |

System Default appears only if the policy holds 1 (Enabled, forced on). The policy key is absent on a stock machine (confirmed live), so an untouched machine reads as "Allowed".

#### How it works

`AllowStorageSenseGlobal` is the "Allow Storage Sense" policy from the shipped `StorageSense.admx` (confirmed on build 26100), also exposed as the Storage Policy CSP. Microsoft documents three states. Enabled: Storage Sense is on, runs "during low free disk space", and the user cannot turn it off. Disabled (0, what this tweak writes): Storage Sense is off and the user cannot turn it on. Not Configured (absent): "Storage Sense is turned off until the user runs into low disk space or the user enables it manually", and the user controls it in Settings. There is no default recurring schedule to stop: the benefit is a lockout that stops anything, including you, an OEM image or a later Windows setup pass, from turning automatic cleanup of Downloads, temporary files and the Recycle Bin on. The per-user Storage Sense state lives under `HKCU\Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy`; the machine policy overrides it, so the tweak leaves it alone. Microsoft lists the policy for Pro, Enterprise, Education, Windows SE and IoT Enterprise; Home is not in the supported list, although the value is read on Home in practice.

#### Benefits
- A permanent lockout: neither a user, an OEM image nor a later setup pass can turn it on.
- No surprise deletions from Downloads, temp folders or the Recycle Bin.
- A shipped, documented ADMX policy.

#### Drawbacks
- Windows will not reclaim anything automatically, even under disk pressure.
- The Storage Sense controls in Settings are greyed out, which can look like breakage.
- No performance change of any kind; this is a behaviour control, not a speed tweak.
- A machine policy: it applies to every user on the PC.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build; Pro, Enterprise, Education and IoT Enterprise are documented, Home reads the value in practice but is unverified.
- **Takes effect**: immediately.
- **Reverting**: "Allowed" deletes the policy, returning control to Settings; your per-user Storage Sense setting, which the tweak never touched, takes over again. System Default restores the snapshot.

#### Interactions
None known in the corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the description of the unconfigured state: Storage Sense is off until low disk space or manual enablement and runs during low free space, not on a default schedule; the benefit is the lockout.
- **Confidence**: Microsoft-documented (Policy CSP, shipped ADMX, Group Policy Search).
- **Reasoning**: key, value, type, the 0 semantics and the absent stock state are all tier A and confirmed live; the adversarial policy-hive audit confirmed it is Machine class under HKLM. The shipped per-user state genuinely varies (on the inspected machine it had been switched on for that profile), which is why the tweak manages only the policy.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying if you have ever lost a file to an automatic cleanup, or if you want the decision pinned so nothing can flip it later. Leave Storage Sense available if you are short on disk space and would rather Windows handle it.

#### Sources
1. Storage Policy CSP, `AllowStorageSenseGlobal`, Microsoft Learn: the Enabled, Disabled and Not Configured wording, the cadence and the edition list, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-storage (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\StorageSense.admx`, build 26100.4061: the key and value name (tier A, primary)
3. Group Policy Search, "Allow Storage Sense": the policy entry, https://gpsearch.azurewebsites.net/default.aspx?policyid=14518 (tier A)
4. Direct inspection of Windows 11 24H2 build 26100.4061: policy key absent; per-user `StoragePolicy` present with `01` = 1 (tier C)

### Enable SSD TRIM (delete notification)

`ssd_optimize_trim` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Confirms TRIM is on, so your SSD keeps its sustained write speed and endurance.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `disable_delete_notify` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`, value `DisableDeleteNotification`, `REG_DWORD` |

| Option | `disable_delete_notify` |
|---|---|
| TRIM enabled | `0` |
| TRIM disabled | `1` |

System Default appears only if the value is absent or holds something else. Windows ships it present with data 0 (confirmed live), so a healthy machine reads as "TRIM enabled".

#### How it works

When a file is deleted, NTFS can send the storage device a delete notification (ATA TRIM or NVMe Deallocate/unmap) naming the blocks that are now free, which lets the SSD controller reclaim them in the background instead of copying stale data during garbage collection. `DisableDeleteNotification` = 0 enables this, 1 disables it; it is the same value `fsutil behavior set disabledeletenotify` writes, and `fsutil behavior query disabledeletenotify` reports it as `NTFS DisableDeleteNotify`. That display string is not the registry name: string extraction from `fsutil.exe` and from `ntfs.sys` (the driver that reads the setting) shows the value names are `DisableDeleteNotification` for NTFS and `RefsDisableDeleteNotification` for ReFS, and no `NtfsDisableDeleteNotify` value exists. Microsoft states "for systems using NTFS, trim is enabled by default unless an administrator disables it" and that the change takes effect at the next unmap command without a restart. TRIM is meaningful only on SSD and NVMe media that report support for it. The tweak covers NTFS only; the ReFS value is not managed. Separately from this value, Windows' scheduled drive optimisation sends a periodic retrim to SSDs.

#### Benefits
- Keeps sustained write speed; an SSD without TRIM slows as it fills and stays slow.
- Better endurance: less write amplification when the controller knows what is free.
- Repairs a machine where a previous tool or image turned TRIM off.

#### Drawbacks
- On a healthy system it confirms a state you already had.
- No measurable gain on modern hardware; this is a health check, not a speed boost.
- Meaningless for hard drives.
- The "TRIM disabled" option is actively harmful on normal SSDs; Microsoft's only stated reason for it is a rare class of device that regresses with delete notification on.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, on SSD or NVMe drives that report TRIM support; NTFS volumes only.
- **Takes effect**: immediately, at the next delete.
- **Reverting**: System Default restores the snapshot. "TRIM enabled" writes 0, the shipped value; it never deletes the value, because Windows ships it present.

#### Interactions
None known in the corpus.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The research found that the registry value is `DisableDeleteNotification`; `NTFS DisableDeleteNotify` is only fsutil's display label, and a value named `NtfsDisableDeleteNotify` is read by nothing. The shipped effect writes `DisableDeleteNotification`.
- **Confidence**: Microsoft-documented for the behaviour and default; the value name is established by direct inspection of the shipped binaries and registry.
- **Reasoning**: the live FileSystem key holds `DisableDeleteNotification` = 0 and no `NtfsDisableDeleteNotify`, `ntfs.sys` contains only `DisableDeleteNotification` and `DisableDeleteNotificationDrain`, and `fsutil.exe` contains both the value names and the display strings. That is three independent measurements agreeing. The ReFS value `RefsDisableDeleteNotification` is not managed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Safe to apply as a health check on any SSD system. Never select "TRIM disabled" unless your drive vendor has specifically told you to; you would be trading long-term write performance for nothing.

#### Sources
1. fsutil behavior, Microsoft Learn: `disabledeletenotify` semantics, the NTFS default and the no-restart behaviour, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior (tier A)
2. Optimize-Volume, Microsoft Learn: the ReTrim operation and scheduled retrim of SSDs, https://learn.microsoft.com/en-us/powershell/module/storage/optimize-volume (tier A)
3. Direct inspection of Windows 11 24H2 build 26100.4061: FileSystem key holds `DisableDeleteNotification` = 0 and no `NtfsDisableDeleteNotify`; `fsutil behavior query disabledeletenotify` reports `NTFS DisableDeleteNotify = 0` and `ReFS DisableDeleteNotify = 0` (tier C)
4. String extraction from `fsutil.exe` and `drivers\ntfs.sys`, build 26100.4061: the real value names (tier C)

### Pin NTFS last-access updates off

`ntfs_disable_lastaccess` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Pins NTFS last-access timestamp updates off instead of leaving the choice to Windows.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `disable_lastaccess` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`, value `NtfsDisableLastAccessUpdate`, `REG_DWORD` |

| Option | `disable_lastaccess` |
|---|---|
| Disabled (user managed) | `0x80000001` |
| System managed | `0x80000002` |

System Default appears for the other two encodings, `0x80000000` (user managed, updates enabled) and `0x80000003` (system managed, updates disabled), or a legacy 0 or 1. Windows ships `0x80000002` (confirmed live), so an untouched machine reads as "System managed".

#### How it works

NTFS can update a file's last-access timestamp each time it is read. Microsoft's `fsutil behavior` reference names this exact value: `disablelastaccess` "updates the `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem\NtfsDisableLastAccessUpdate` registry key". Since Windows 10 1803 the value encodes a management mode and a state with a high sentinel bit:

| Value | Meaning |
|---|---|
| `0x80000000` | User Managed, last-access updates enabled |
| `0x80000001` | User Managed, last-access updates disabled |
| `0x80000002` | System Managed, last-access updates enabled |
| `0x80000003` | System Managed, last-access updates disabled |

In System Managed mode the NTFS driver decides at boot: updates are enabled when the system volume is 128 GB or smaller and disabled when it is larger. So on nearly every machine with a system volume over 128 GB, System Managed mode has already turned updates off, and this tweak's real-world delta is often exactly zero; what it adds is that your choice holds regardless of volume size. NTFS also defers on-disk last-access writes by up to an hour and answers queries from memory, so the write volume at stake is smaller than intuition suggests. The value is read at boot; Microsoft states a restart is required.

#### Benefits
- No per-read "last accessed" metadata write.
- Deterministic: your choice holds regardless of volume size or a later disk change.
- The same value `fsutil behavior set disablelastaccess` writes.

#### Drawbacks
- Microsoft warns this "can affect programs such as Backup and Remote Storage"; forensic and archiving tools rely on last-access times too.
- Overrides Windows' boot-time decision, so a later volume change is not reflected.
- No visible change on any system volume larger than 128 GB, where System Managed mode already disabled the updates.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the four-state encoding exists from Windows 10 1803 onward).
- **Takes effect**: after a reboot.
- **Reverting**: "System managed" writes `0x80000002`, the value Windows ships; the value is never deleted. System Default restores the snapshot.

#### Interactions
None known in the corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that a reboot is required and that on volumes over 128 GB the change is usually invisible.
- **Confidence**: Microsoft-documented for the value and restart requirement; the four-state encoding and 128 GB rule are community-corroborated.
- **Reasoning**: Microsoft names the value and the restart requirement; two independent forensics sources document the encoding and threshold; the stock `0x80000002` was confirmed live along with fsutil's "System Managed, Last Access Time Updates ENABLED". Under the corpus inclusion rule, the control belongs even though its visible effect is often nil; the entry says so instead of hiding it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying if you want last-access updates pinned off by your own decision rather than left to Windows to re-evaluate at every boot. Skip it if you run backup, archiving or forensic software that reads last-access times.

#### Sources
1. fsutil behavior, Microsoft Learn: names the value, gives the restart requirement and the Backup and Remote Storage warning, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/fsutil-behavior (tier A)
2. "The Last Access updates are almost back", My DFIR Blog: the four-state encoding and the 128 GB rule, https://dfir.ru/2018/12/08/the-last-access-updates-are-almost-back/ (tier C)
3. "Daily Blog #557: Changes in the NtfsDisableLastAccessUpdate key", Hacking Exposed Computer Forensics Blog: confirms the 128 GB threshold, https://www.hecfblog.com/2018/12/daily-blog-557-changes-in.html (tier C)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `NtfsDisableLastAccessUpdate` = 0x80000002; `fsutil behavior query disablelastaccess` returns `DisableLastAccess = 2 (System Managed, Last Access Time Updates ENABLED)` (tier C)

### Disable RAM memory compression

`disable_memory_compression` · Switch · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off RAM page compression, trading memory headroom for a little less CPU work.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `memory_compression` | action (PowerShell, with undo and probe) | apply `Disable-MMAgent -MemoryCompression`; undo `Enable-MMAgent -MemoryCompression`; probe `(Get-MMAgent).MemoryCompression -eq $false` |

| Option | `memory_compression` |
|---|---|
| Disabled | run apply |

Every cmdlet call uses `-ErrorAction Stop` and exits non-zero on failure, so a failed call never reads as success. System Default appears while memory compression is on, which is how Windows ships; selecting it after applying restores the snapshot, which runs the undo. Detection reads `Get-MMAgent` itself, so a machine where compression was already off reads as "Disabled" for every account and has nothing to revert.

#### How it works

When memory is under pressure, the Windows memory manager compresses infrequently used pages and keeps them in the working set of the Memory Compression process instead of writing them to the page file, so more data fits in RAM before disk paging starts. `Disable-MMAgent -MemoryCompression` is the documented switch for this one MMAgent feature (the others are application launch prefetching, the operation recorder API, page combining and application prelaunch); `Get-MMAgent` reports its state, which is what the probe reads. The cmdlets ship in-box on Windows 10 and 11, all client editions, and need an elevated session. With compression off, cold pages go straight to the page file: on a machine that rarely hits memory pressure that changes little, while on 8 or 16 GB it means more hard faults and more disk paging. The change applies immediately to newly trimmed pages; the existing compressed store drains over time or at the next boot, which is why the tweak flags a reboot for the full effect.

#### Benefits
- The compress and decompress CPU work disappears.
- Simpler memory behaviour: cold pages go to the page file rather than a compressed store.
- A supported Microsoft cmdlet with a matching enable command.

#### Drawbacks
- On 8 or 16 GB, removing the cushion causes more paging to disk and more hard faults, which hurts rather than helps.
- With a small or absent page file, you hit out-of-memory earlier.
- No measurable gain on modern hardware; the Memory Compression process normally uses very little CPU, and this is one of the most commonly recommended and most commonly harmful tweaks in circulation.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition; sensible only with 32 GB of RAM or more.
- **Takes effect**: immediately for newly compressed pages; fully after a reboot, when the existing store is gone.
- **Reverting**: turning the switch off (System Default) after applying runs `Enable-MMAgent -MemoryCompression`, restoring the shipped behaviour.

#### Interactions
`memory_prefetch_mode` ("Disable SysMain (SuperFetch) prefetching") is a different MMAgent feature; disabling the SysMain service does not disable compression, and this tweak does not touch SysMain.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the timing: the cmdlet takes effect immediately for newly compressed pages, and the reboot only drains the existing store.
- **Confidence**: Microsoft-documented (`Disable-MMAgent`, `Enable-MMAgent`, `Get-MMAgent`).
- **Reasoning**: apply, undo and probe map one-to-one onto the documented cmdlets, and `Get-MMAgent` returned `MemoryCompression : False` on a machine where it had been applied, confirming the probe. The adversarial probe audit lists this probe among those with the safe polarity.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Only on a high-RAM machine where you have actually proven compression is costing you something measurable. On a typical 8 to 16 GB machine, leave it enabled: disabling it will make things worse, not better.

#### Sources
1. Disable-MMAgent, Microsoft Learn: the switch and the MMAgent feature list, https://learn.microsoft.com/en-us/powershell/module/mmagent/disable-mmagent (tier A)
2. Enable-MMAgent, Microsoft Learn: the undo, https://learn.microsoft.com/en-us/powershell/module/mmagent/enable-mmagent (tier A)
3. Get-MMAgent, Microsoft Learn: the probe surface, https://learn.microsoft.com/en-us/powershell/module/mmagent/get-mmagent (tier A)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `Get-MMAgent` returns `MemoryCompression : False` after the tweak (tier C)

### Disable Fast Startup (hiberboot)

`disable_fast_startup` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no (takes effect at the next shutdown) · Windows: all supported builds · Reversible: yes

**Makes Shut Down a real shutdown, so drivers and kernel state start fresh every time.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hiberboot` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power`, value `HiberbootEnabled`, `REG_DWORD` |

| Option | `hiberboot` |
|---|---|
| Disabled | `0` |
| Enabled | `1` |

System Default appears if the value is absent or holds something else. Windows ships it present with data 1 (confirmed live), so an untouched machine reads as "Enabled".

#### How it works

Fast Startup (hiberboot, Windows 8 and later) turns Shut Down into a hybrid operation: user sessions are logged off, then the kernel session and loaded drivers are hibernated to `hiberfil.sys` and restored at the next power-on, which is faster than a cold boot. `HiberbootEnabled` = 0 makes Shut Down a true full shutdown, so drivers and kernel state are initialised from scratch every start. It is the registry backing of Control Panel > Power Options > Choose what the power buttons do > "Turn on fast startup". The value governs what the next shutdown does; a Restart is always a full cold boot and never uses hiberboot, which is exactly why "restart fixes it" often works when "shut down and power on" does not, and why this tweak does not ask for a reboot. Fast Startup exists only when hibernation is available: `powercfg /h off` removes `hiberfil.sys` and makes Fast Startup unavailable regardless of this value. A hibernated NTFS volume mounted read-write by another operating system can be corrupted, which is the dual-boot hazard this removes.

#### Benefits
- Clears driver state: GPU and USB quirks that only a true cold boot resolves.
- Safer for dual-boot, since the Windows volume is no longer left hibernated.
- An honest shutdown: powering off actually resets everything.

#### Drawbacks
- A slower cold boot: a handful of seconds on a modern NVMe machine, more on older hardware.
- Stability, not speed: it changes no frame rate and no responsiveness.
- No measurable gain on modern hardware; the benefit is the class of problem it prevents, not anything you can benchmark.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition, where hibernation is available.
- **Takes effect**: at your next shutdown. A Restart does not exercise this setting.
- **Reverting**: "Enabled" writes 1, the value Windows ships; the value is never deleted. System Default restores the snapshot.

#### Interactions
`network:disable_hibernation` ("Disable hibernation") runs `powercfg /hibernate off`, which removes `hiberfil.sys` and makes Fast Startup unavailable regardless of this value. With hibernation disabled, this tweak has no effect.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the change applies at the next shutdown, not after a restart, so the tweak does not request a reboot.
- **Confidence**: community-corroborated for the registry value; the `powercfg /h off` interaction is Microsoft-documented.
- **Reasoning**: the key, value and 0/1 semantics agree across independent community and vendor sources, and the stock value of 1 was confirmed live.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Turn it off if you dual-boot, or if you are chasing driver and GPU oddities that clear after a real power cycle. If your machine boots cleanly and you value the few seconds, leave it on.

#### Sources
1. "Turn On or Off Fast Startup in Windows 11", ElevenForum: key path, value name and 0/1 semantics, https://www.elevenforum.com/t/turn-on-or-off-fast-startup-in-windows-11.1212/ (tier C)
2. "Disabling Windows 10 Fast Startup", HP Wolf Security support: independent vendor documentation, https://support.hpwolf.com/s/article/Disabling-Windows-10-Fast-Startup (tier C)
3. Powercfg command-line options, Microsoft Learn: `powercfg /h off` and its effect on Fast Startup availability, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `HiberbootEnabled` = 0x1 present (tier C)

### Disable VBS and Memory Integrity (HVCI)

`disable_vbs_hvci` · Switch (2 options) · Risk: critical · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off Memory Integrity to recover CPU-bound frame times, at a critical cost to security.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `enable_vbs` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard`, value `EnableVirtualizationBasedSecurity`, `REG_DWORD` |
| `hvci_enabled` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity`, value `Enabled`, `REG_DWORD` |

| Option | `enable_vbs` | `hvci_enabled` |
|---|---|---|
| Disabled | `0` | `0` |
| Memory integrity enabled | `absent` | `1` |

System Default appears for any other combination, which includes the common case of an upgraded machine where both values are absent (memory integrity never auto-enabled). On a clean Windows 11 install on qualifying hardware, Microsoft's recommended configuration has `Scenarios\HypervisorEnforcedCodeIntegrity\Enabled` = 1 (plus `WasEnabledBy` and `EnabledBootId`) and no `EnableVirtualizationBasedSecurity`, which is the "Memory integrity enabled" row.

#### How it works

Virtualization-Based Security (VBS) uses the hypervisor to create an isolated virtual trust level that hosts a secure kernel. Hypervisor-enforced Code Integrity (HVCI), branded Memory Integrity in Windows Security, runs kernel-mode code integrity checks inside that isolated environment, so even an attacker who compromises the normal kernel cannot load unsigned or tampered kernel code. That validation costs CPU time on memory and driver operations; Microsoft states processors without Mode-Based Execution Control (Intel before Kaby Lake) or Guest Mode Execute Trap (AMD before Zen 2) fall back to an emulation, Restricted User Mode, which "will have a bigger impact on performance". Microsoft's memory-integrity article gives these exact key paths and value names. `EnableVirtualizationBasedSecurity` is a configuration value that is absent unless someone sets it; the enabled state is therefore that value absent plus `Enabled` = 1. Setting both to 0 turns memory integrity off at the next boot, but it does not stop the hypervisor or VBS itself: on the inspected 24H2 machine, with both values at 0, `Win32_DeviceGuard` still reported VBS "enabled and running" with Credential Guard running, because Hyper-V, WSL2, Windows Sandbox, Credential Guard and similar features keep the hypervisor launched. Fully stopping it would need `bcdedit /set hypervisorlaunchtype off`, which this tweak does not do. If memory integrity was enabled with UEFI lock (`Locked` = 1), the registry change has no effect until Secure Boot is disabled in firmware. The tweak does not touch `WasEnabledBy`, the value Windows Security uses to decide whether the toggle is user-controlled. Microsoft auto-enables memory integrity only on clean installs of Windows 11 on hardware meeting the bar (Intel 8th generation or later from 22H2, AMD Zen 2 or later, Qualcomm 8180 or later, 8 GB RAM, 64 GB SSD, compatible drivers, virtualization enabled in firmware) and on Secured-core PCs; "auto-enablement pertains only to clean installs, not upgrades of existing devices."

#### Benefits
- Recovers CPU headroom, with the largest effect on 1 percent lows in CPU-limited games.
- Older CPUs benefit most, per Microsoft's own statement about Restricted User Mode.
- Lets some older drivers load that memory integrity blocks.

#### Drawbacks
- A critical security downgrade: no hypervisor-enforced protection against kernel-mode malware and malicious or vulnerable drivers.
- VBS keeps running for Hyper-V, WSL2, Sandbox and Credential Guard, so the recovered performance is less than advertised.
- Some anti-cheat systems require memory integrity, and every enterprise or compliance baseline does.
- The Windows Security memory integrity page may still read as administrator-managed after a revert, because `WasEnabledBy` is not handled.
- The published 5 to 15 percent gains come from technical press, not Microsoft; treat them as indicative.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build; memory integrity is on by default only on clean Windows 11 installs meeting the hardware bar. Has no effect under UEFI lock.
- **Takes effect**: after a reboot.
- **Reverting**: System Default restores the snapshot, which is the right path on an upgraded machine where both values were absent. "Memory integrity enabled" deletes `EnableVirtualizationBasedSecurity` and sets `Enabled` = 1, which turns memory integrity on even if your machine never had it; check driver compatibility before choosing it.

#### Interactions
Directly conflicts with `security:enable_credential_guard` ("Enable Credential Guard"). Credential Guard is hosted by VBS; that tweak sets `LsaCfgFlags` under `Control\Lsa` and requires platform security features, and this tweak removes the protection it relies on. Never apply both; neither snapshot knows about the other. The research considered merging this tweak with `disable_spectre_meltdown` into one "CPU security mitigations" control and recommended against it, because the two revert paths differ (memory integrity can also be changed from Windows Security; the Spectre overrides only from the registry).

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found that `EnableVirtualizationBasedSecurity` is absent on a stock machine (so the enabled state deletes it rather than writing 1), that zeroing the two values stops memory integrity but not VBS, that `WasEnabledBy` and UEFI lock affect the result, and that the tweak conflicts with Credential Guard.
- **Confidence**: Microsoft-documented (the memory-integrity article gives the exact values; the OEM article gives the defaults).
- **Reasoning**: key paths and names are tier A; the VBS-still-running observation is a direct WMI measurement. Open question: which of the three values a retail clean 24H2 install on qualifying hardware actually contains; Microsoft documents the recommended image configuration, not the retail result.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Only consider this on a personal gaming machine where you knowingly accept a weaker security posture for CPU headroom, and only after confirming your anti-cheat allows it. Never on a work, shared or sensitive machine, and never alongside Credential Guard.

#### Sources
1. Enable memory integrity, Microsoft Learn: the exact `reg add` commands for `EnableVirtualizationBasedSecurity`, `RequirePlatformSecurityFeatures`, `Locked`, `Scenarios\HypervisorEnforcedCodeIntegrity\Enabled` and `WasEnabledBy`, https://learn.microsoft.com/en-us/windows/security/hardware-security/enable-virtualization-based-protection-of-code-integrity (tier A)
2. Memory integrity and VBS enablement, OEM guidance, Microsoft Learn: default enablement rules, the hardware bar, the clean-install-only statement and the recommended image configuration, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/oem-hvci-enablement (tier A)
3. Windows 11 STIG V-253371, virtualization-based protection of code integrity must be enabled: the hardening baseline, https://www.stigviewer.com/stigs/microsoft_windows_11/2025-05-15/finding/V-253371 (tier B)
4. Direct inspection of Windows 11 24H2 build 26100.4061: `EnableVirtualizationBasedSecurity` = 0, HVCI `Enabled` = 0, `Locked` = 0, yet `VirtualizationBasedSecurityStatus` = 2 and `SecurityServicesRunning` = {1} (tier C)

### Disable Spectre / Meltdown CPU mitigations

`disable_spectre_meltdown` · Switch (2 options) · Risk: critical · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes the Spectre v2 and Meltdown mitigations to reclaim CPU throughput on older processors.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `feature_settings_override` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management`, value `FeatureSettingsOverride`, `REG_DWORD` |
| `feature_settings_override_mask` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management`, value `FeatureSettingsOverrideMask`, `REG_DWORD` |

| Option | `feature_settings_override` | `feature_settings_override_mask` |
|---|---|---|
| Spectre v2 and Meltdown mitigations disabled | `3` | `3` |
| Enabled | `absent` | `absent` |

System Default appears whenever an administrator has configured some other override, for example `FeatureSettingsOverride` = 0x2000000 (the Downfall setting) or 72. Both values are absent on a stock machine, so an untouched machine reads as "Enabled".

#### How it works

Windows mitigates speculative-execution side channels in the kernel. CVE-2017-5715 (Spectre variant 2, branch target injection) is mitigated with retpoline or IBRS/IBPB, and CVE-2017-5754 (Meltdown, rogue data cache load) with KVA Shadow, which separates kernel and user page tables and costs most on syscall-heavy and context-switch-heavy workloads. KB4073119 gives exactly this pair to disable those two mitigations: `FeatureSettingsOverride` = 3 and `FeatureSettingsOverrideMask` = 3, where the mask selects which bits of the override are honoured ("a value of 3 is accurate for FeatureSettingsOverrideMask for both the enable and disable settings"). Other mitigations (MDS, TAA, L1TF, SSBD, Gather Data Sampling) use other bits of the same DWORD and stay on, so this is not "all mitigations off". Because the override is one DWORD, writing 3 overwrites any other bit an administrator had set, including the 0x2000000 KB5029778 specifies for disabling the Downfall mitigation, and reverting to absent deletes such a setting. Windows enables KVA Shadow only on CPUs that do not report `RDCL_NO` in `IA32_ARCH_CAPABILITIES`; any CPU that does (broadly, Intel 10th generation / Ice Lake onward and all AMD) never had it enabled, so the Meltdown half has nothing to reclaim there. The values are read at boot and apply on Windows 10 and 11, all editions, with the January 2018 or later updates.

#### Benefits
- Reclaims syscall and context-switch throughput on affected CPUs.
- Older CPUs relying on software retpoline, IBRS/IBPB or KVA Shadow gain the most.
- The same two values Microsoft's own KB publishes.

#### Drawbacks
- A critical security downgrade: re-exposes side-channel attacks that read memory across process and virtual machine boundaries.
- Only two CVEs: MDS, TAA, L1TF, SSBD and Downfall use other bits and stay on.
- Overwrites any other mitigation override already in the DWORD, and reverting deletes it.
- No measurable gain on modern hardware; on any CPU reporting `RDCL_NO` Meltdown protection was never enabled, so half of this tweak has nothing to reclaim.
- Some benchmarks regress after disabling these; measure your own workload.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition; the largest effect is on older CPUs without silicon-level fixes.
- **Takes effect**: after a reboot.
- **Reverting**: "Enabled" deletes both values, the stock state, which also deletes any unrelated override that was there before. System Default restores the snapshot, which puts back whatever was there, including an administrator's own override. On a managed machine, check the current value before applying and prefer System Default when reverting.

#### Interactions
`disable_vbs_hvci` is the other critical CPU-security-for-throughput trade in this category; the research recommended keeping them separate. No other tweak in the corpus writes `FeatureSettingsOverride`.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that 3/3 disables only the Spectre v2 and Meltdown mitigations (the option label says so), that it overwrites other bits such as the Downfall setting, and that the Meltdown half is a no-op on CPUs reporting `RDCL_NO`.
- **Confidence**: Microsoft-documented (KB4073119, KB5029778, the MSRC KVA Shadow post).
- **Reasoning**: the pair and its scope are quoted from KB4073119, with no Microsoft statement retiring it; the RDCL_NO rule is from MSRC. The inspected machine had both values at 3 only because the tweak had been applied, so it is not evidence of a default; the absent default rests on the KB's description of these as administrator overrides.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Hard to justify on modern hardware: the benefit is small or absent and the risk is real. Only an option on an isolated, older, personal machine where you knowingly accept the exposure. Never on a shared, work or virtualization-hosting machine.

#### Sources
1. KB4073119, Windows client guidance for IT Pros to protect against speculative execution side-channel vulnerabilities: the two values, the 3/3 pair and the `FeatureSettingsOverride` = 72 variant, https://support.microsoft.com/en-us/topic/kb4073119-windows-client-guidance-for-it-pros-to-protect-against-silicon-based-microarchitectural-and-speculative-execution-side-channel-vulnerabilities-35820a8a-ae13-1299-88cc-357f104f5b11 (tier A)
2. KVA Shadow: Mitigating Meltdown on Windows, MSRC: KVA Shadow is enabled only when RDCL_NO is clear, https://www.microsoft.com/en-us/msrc/blog/2018/03/kva-shadow-mitigating-meltdown-on-windows (tier A)
3. KB5029778, managing CVE-2022-40982 (Downfall / Gather Data Sampling): `FeatureSettingsOverride` = 0x2000000, via a mirror, https://www.elevenforum.com/t/kb5029778-how-to-manage-cve-2022-40982-downfall-cpu-vulnerability.17350/ (tier A content, tier D mirror)
4. Direct inspection of Windows 11 24H2 build 26100.4061: both values present with data 3 after the tweak, not evidence of a default (tier C)

### Disable mouse acceleration (Enhance Pointer Precision)

`disable_mouse_acceleration` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no (sign out) · Windows: all supported builds · Reversible: yes

**Makes pointer movement 1:1 with the mouse, so the same motion always moves the same distance.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `mouse_speed` | registry | `HKCU\Control Panel\Mouse`, value `MouseSpeed`, `REG_SZ` |
| `mouse_threshold1` | registry | `HKCU\Control Panel\Mouse`, value `MouseThreshold1`, `REG_SZ` |
| `mouse_threshold2` | registry | `HKCU\Control Panel\Mouse`, value `MouseThreshold2`, `REG_SZ` |

| Option | `mouse_speed` | `mouse_threshold1` | `mouse_threshold2` |
|---|---|---|---|
| Acceleration off | `"0"` | `"0"` | `"0"` |
| Acceleration on | `"1"` | `"6"` | `"10"` |

System Default appears for any other combination. The Windows default is taken to be 1 / 6 / 10 (Microsoft's legacy KB and community sources agree), so an untouched machine normally reads as "Acceleration on"; this was not observed on a clean image.

#### How it works

These are the classic Windows pointer ballistics parameters, the registry backing of the "Enhance pointer precision" checkbox in Mouse Properties > Pointer Options. `MouseThreshold1` and `MouseThreshold2` are movement thresholds: if the mouse moves more than `MouseThreshold1` in one interval and `MouseSpeed` is above 0, the cursor moves at twice normal distance; if it exceeds `MouseThreshold2` and `MouseSpeed` is 2, at four times. With all three at 0, the mouse-to-cursor ratio never changes with speed: a strict 1:1 mapping. The values are strings (`REG_SZ`) in the registry, confirmed live; a tool that writes them as DWORDs is silently ignored. Windows reads them into the session at sign-in, or applies them immediately when a program calls `SystemParametersInfo(SPI_SETMOUSE)`; this tweak writes the registry only, so sign out and back in to feel the change. `SmoothMouseXCurve` and `SmoothMouseYCurve` in the same key define the modern acceleration curve and are not touched, so a machine with a custom curve is not fully neutralised. Many games read raw input and ignore Windows ballistics entirely, so the in-game effect depends on the title.

#### Benefits
- Predictable aim: muscle memory maps to a fixed physical distance, which matters in aim-heavy shooters.
- One of the few tweaks in this category you can feel.
- Three string values with well-known defaults, cleanly reversible.

#### Drawbacks
- Crossing a large or multi-monitor desktop takes a bigger physical motion.
- Games that read raw input already bypass Windows ballistics, so this may only change the desktop cursor.
- `SmoothMouseXCurve` and `SmoothMouseYCurve` are not touched, so a custom curve is not neutralised.
- Needs a sign-out to take effect.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after you sign out and back in.
- **Reverting**: "Acceleration on" writes the defaults 1 / 6 / 10; System Default restores the snapshot, which is more faithful if you had custom values. Sign out afterwards.

#### Interactions
None known in the corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that a sign-out, not a reboot, applies these values, so the tweak does not request a reboot.
- **Confidence**: Microsoft-documented (the archived Microsoft KB Q149228 and the `SPI_SETMOUSE` reference).
- **Reasoning**: value names, the 0-to-disable procedure and the `REG_SZ` typing are tier A and were confirmed live. The 1 / 6 / 10 defaults are consistent across Microsoft's legacy KB and a community source, but the inspected machine was already tweaked, so the clean-install values remain open.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Strongly recommended for gamers, especially in shooters, where 1:1 movement is a real advantage. If you only use the mouse for desktop work across a large screen and like the accelerated feel, leave it on.

#### Sources
1. Q149228, "How to Disable Mouse Acceleration", Microsoft KB archive: the three values and the 0-to-disable procedure, https://jeffpar.github.io/kbarchive/kb/149/Q149228/ (tier A content, archived mirror)
2. SystemParametersInfoW, Microsoft Learn: `SPI_SETMOUSE` applies these values to a running session, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow (tier A)
3. "Turn On or Off Enhance Pointer Precision in Windows", TenForums: the 1 / 6 / 10 defaults, https://www.tenforums.com/tutorials/101691-turn-off-enhance-pointer-precision-windows.html (tier C)
4. Direct inspection of Windows 11 24H2 build 26100.4061: all three values present as `REG_SZ` (tier C)

### Turn off reserved storage

`reserved_storage_off` · Switch · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Frees the disk space Windows sets aside for updates, at the cost of update headroom.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `reserved_storage` | action (PowerShell, with undo and probe, 300-second timeout) | apply `Set-WindowsReservedStorageState -State Disabled`; undo `Set-WindowsReservedStorageState -State Enabled`; probe: the `ReservedStorageState` property of `Get-WindowsReservedStorageState` is `Disabled` |

| Option | `reserved_storage` |
|---|---|
| Reserved storage off | run apply |

Every cmdlet call uses `-ErrorAction Stop` and exits non-zero on failure. System Default appears while reserved storage is on; selecting it after applying restores the snapshot, which runs the undo. Detection reads the live state, so a machine that never had reserved storage (an upgraded install) reads as "Reserved storage off" for every account, and nothing is turned on by a revert.

#### How it works

Reserved storage (Windows 10 1903 and later) is a block of disk space Windows sets aside so that feature updates, temporary files and system caches always have room, even when the disk is nearly full. `Set-WindowsReservedStorageState` in the DISM PowerShell module turns it off or on for the running (online) image, and `Get-WindowsReservedStorageState` reports it; the probe reads its `ReservedStorageState` property, an enum that is not translated. Microsoft documents the failure mode verbatim: if reserved storage is in use, it "may not be disabled", and the cmdlet returns "This operation is not supported when reserved storage is in use. Please wait for any servicing operations to complete and then try again later." In that case the apply fails and the tweak reports the failure rather than a false success. Reserved storage is enabled automatically only on new PCs with 1903 or later preinstalled and on clean installs; it is not enabled when upgrading from an earlier version, so many machines never had it and there is nothing to reclaim. Its size varies with installed language packs and optional features, typically several GB. The action has a 300-second timeout because DISM servicing calls can run long.

#### Benefits
- Reclaims several GB, the largest single reclaimable allocation on a stock install.
- A first-party DISM cmdlet with a matching enable command, not a registry poke.
- On a 128 GB device the reclaimed space is a meaningful fraction.

#### Drawbacks
- With the reserve gone, an update on a nearly full disk can fail or demand that you free space manually.
- Often nothing to reclaim: upgraded machines never had reserved storage.
- No performance change of any kind; this is disk space, not speed.
- Refused while a servicing operation is in flight.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition; only useful where reserved storage was enabled (clean installs and preinstalled machines). On the LTSC/IoT image inspected it was already disabled.
- **Takes effect**: immediately, unless a servicing operation is in progress, in which case Windows refuses and the tweak reports the failure.
- **Reverting**: turning the switch off (System Default) after applying runs `Set-WindowsReservedStorageState -State Enabled`. A machine that never had reserved storage reads as "Reserved storage off" before any apply, so it is never turned on by a revert.

#### Interactions
None; nothing else in the corpus touches reserved storage.

#### Validation
- **Verdict**: VERIFIED. Carried in from the gap hunt and confirmed by the adversarial pass, with framing corrections: the size is variable (no fixed "roughly 7 GB"), upgraded machines may never have had it, and it is a disk-space control, not a speed tweak.
- **Confidence**: Microsoft-documented (both cmdlets and the enablement rule on Microsoft Learn).
- **Reasoning**: the cmdlet, its parameter values and its in-use failure mode are quoted from Microsoft Learn, and the clean-install-only rule from Microsoft's LTSC 2021 what's-new page. The property the probe reads, `ReservedStorageState` (type `Microsoft.Dism.Commands.ReservedStorageState`), was confirmed on build 26100.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth it on a small drive where several GB actually matters and you are willing to manage free space around feature updates yourself. On a roomy drive, leave it alone; you are trading update reliability for space you are not short of.

#### Sources
1. Set-WindowsReservedStorageState, Microsoft Learn: the `-State` values, the online-images-only restriction and the in-use error, https://learn.microsoft.com/en-us/powershell/module/dism/set-windowsreservedstoragestate (tier A)
2. Get-WindowsReservedStorageState, Microsoft Learn: the probe surface, https://learn.microsoft.com/en-us/powershell/module/dism/get-windowsreservedstoragestate (tier A)
3. What's new in Windows 10 Enterprise LTSC 2021, Reserved storage section, Microsoft Learn: enabled on clean installs and preinstalled PCs, not on upgrades, https://learn.microsoft.com/en-us/windows/whats-new/ltsc/whats-new-windows-10-2021 (tier A)
4. Live `Get-WindowsReservedStorageState` on build 26100.4061 and on the LTSC/IoT research image, where it reported `Disabled` (tier C)

## Considered and not shipped

### Do not index encrypted files (`no_index_encrypted_files`)

Moved to the Security Hardening category, where it ships as [Do not index encrypted files](security.md#do-not-index-encrypted-files). It sets the machine policy `AllowIndexingEncryptedStoresOrItems` = 0 under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`, which stops Windows Search extracting and storing the contents of EFS-encrypted files in its unencrypted index database. The July 2026 performance research carried it in from the gap hunt, then reassigned it: the control is about keeping encrypted content confidential, not about speed, so its authoritative entry lives with the security research. That research confirmed the policy is in the shipped `Search.admx` on 26100 (the file is UTF-16LE, which is why a plain UTF-8 search misses it), that the shipped default already does not index encrypted content, and that changing the setting triggers a complete index rebuild. It has one link back to this category: it does nothing while [Disable Windows Search indexing](#disable-windows-search-indexing) has the indexer turned off.

### Rejected performance myths

The research also evaluated and rejected these common performance tweaks. None ships, and none should be added back without new measured evidence at build 26100 or later.

| Candidate | Why it is not shipped |
|---|---|
| Timer resolution forcing, HPET, `bcdedit /set useplatformclock`, `disabledynamictick` | Since Windows 11 22H2 a process requesting a high timer resolution no longer changes the global resolution, so the classic mechanism does not do what the guides claim on 26100. |
| `Win32PrioritySeparation` | A foreground/background quantum hint with no reproducible gaming benefit and no measured evidence at 26100. |
| QoS bandwidth reservation (`NonBestEffortLimit`, the "20 percent reserved bandwidth" claim) | The premise is false: the reservation applies only to applications that explicitly request QoS. |
| Disabling or fixing the size of the page file | Causes hard failures in applications that commit large amounts of memory, with no measured gain. |
| Standby list clearing (`EmptyStandbyList` and similar) | Discards a cache the OS then has to refill; the measured effect is negative on average. |
| `LargeSystemCache` | Server-oriented, deprecated behaviour on client editions. |
| Prefetch and Superfetch registry values | Duplicates the supported control, the `SysMain` service, less safely; see [Disable SysMain (SuperFetch) prefetching](#disable-sysmain-superfetch-prefetching). |
| TCP registry packs (`TcpAckFrequency`, `TCPNoDelay`, `TcpWindowSize`, autotuning off, "gaming netsh" bundles) | Per-interface, undocumented, and repeatedly shown not to reduce game latency on modern stacks. |
| MMCSS `SystemProfile\Tasks\Games` tuning (`GPU Priority` = 8, `Scheduling Category` = High, `SFIO Priority` = High) | No published measurement of a frame-time or latency gain on 26100; most stock values are already the tuned ones. |
| MSI mode (`MSISupported`) and interrupt affinity | Per-device-instance registry paths cannot be expressed as a portable tweak, and forcing MSI on a driver that does not support it can make a system unbootable; modern GPUs already default to MSI. |
| Disabling 8.3 short-name creation (`ntfs_disable_8dot3`) | The cited Microsoft performance claim is not on the cited page, no measured benefit was found, and the real 26100 default is 2 (per-volume), which makes the revert a hazard. |
| `DisablePagingExecutive`, `SvcHostSplitThresholdInKB`, `NtfsMemoryUsage`, `TdrDelay` / `TdrDdiDelay`, PCIe ASPM and USB 3 link power management off, processor `IDLEDISABLE` and core-parking overrides, `powercfg /overlaysetactive`, `ClearPageFileAtShutdown`, `Ndu` / `DusmSvc` disabling | Rejected in the earlier gap review for the same reasons: no measured benefit, and in several cases real stability or power costs. |
