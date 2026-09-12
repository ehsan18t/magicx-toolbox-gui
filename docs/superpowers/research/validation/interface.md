# Interface & Explorer tweak validation

Rebuilt 2026-07-27. Source corpus: `src-tauri/tweaks/interface.yaml` (36 tweaks as authored), plus
twelve verified additions drawn from the gap-hunt verification pass.

**Target platform:** Windows 11 24H2 (build 26100) and newer, including 25H2 (26200), primary.
Windows 10 IoT Enterprise LTSC 2021 (build 19044) is a low-priority secondary target. Windows 10 22H2
consumer and Windows 11 21H2 through 23H2 are out of scope.

This document consolidates the original validation pass, two adversarial verification rounds run over
it, the 24H2 re-scope, the harmful-revert audit, and the cross-category conflict review. It is the
authoritative research record for this category.

**Scope of this revision.**

- **42 live tweaks**, each with a full entry below.
- **1 tweak moved out.** `disable_copilot_taskbar` now belongs to the new `ai` category and is
  documented in `ai.md`. Its research is not repeated here; the summary table records the move so the
  reader does not think it was dropped.
- **5 tweaks recommended for deletion**, with a short entry each explaining why. No info blocks are
  written for these.
- **12 tweaks new in this revision**, all carrying a verified mechanism from the gap-hunt
  verification documents.

## A note on source tiers in this category

Most values in this file are File Explorer and shell *user preferences*, not policy. Microsoft does
not publish a reference for `HKCU\...\Explorer\Advanced` value names, so a tier A page for `Hidden`,
`HideFileExt`, `LaunchTo`, `TaskbarAl`, `TaskbarGlomLevel`, `ShowSecondsInSystemClock` and friends
does not exist. Where that is the case the entry says so and cites tier C. `VERIFIED` is used for a
shell preference only when several independent long-lived tier C references agree on key, name, type
and enum, **and** the setting is exposed in Windows Settings or Folder Options so the mechanism is
directly observable rather than inferred. Policy-backed tweaks are held to the stricter bar and are
sourced from the ADMX that ships in `C:\Windows\PolicyDefinitions` on 26100, which is Microsoft's own
artifact and identical on any 26100 install.

**Source-availability warning.** `admx.help` returned HTTP 522 for parts of this work and
`getadmx.com` is now a squatted gambling domain. `elevenforum.com` and `tenforums.com` return HTTP
403 to automated fetchers and were reached through the Wayback Machine. Where a URL is cited that is
currently unreachable to an automated client, the entry says so.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `enable_dark_mode` | VERIFIED | low | Community-corroborated | none |
| `disable_transparency` | VERIFIED | low | Community-corroborated | none |
| `disable_ui_animations` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | `MinAnimate` needs a sign-out, not an Explorer restart |
| `disable_aero_shake` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | stock default is wrong on Windows 11 (shake ships off) |
| `show_file_extensions` | VERIFIED | low | Community-corroborated | none |
| `show_hidden_files` | VERIFIED | low | Community-corroborated | none |
| `open_explorer_to_this_pc` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | option label says "Quick Access"; on Windows 11 value 2 is "Home" |
| `explorer_full_path_title` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | does not affect the address bar, title bar only |
| `disable_recent_files` | INCORRECT | low | Community-corroborated | wrong key: values live under `Explorer`, not `Explorer\Advanced` |
| `disable_start_recent_items` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | stock state is value-absent on 26100, not a written `1` |
| `disable_task_view_button` | VERIFIED | low | Community-corroborated | none |
| `seconds_in_tray_clock` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | missing build gate for the LTSC 2021 secondary target |
| `disable_search_highlights` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | missing the durable HKLM policy companion; "default 1" is likely value-absent |
| `taskbar_search_mode` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | no option is marked Stock Default; missing `SearchboxTaskbarModeCache` |
| `taskbar_ungroup_labels` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | missing `MMTaskbarGlomLevel` companion; copy carries dead 21H2/22H2 text |
| `taskbar_hover_time` | DISPUTED | low | Community-corroborated | reported non-functional since the 24H2 taskbar rewrite; "Default" option writes 400 where stock is absent |
| `taskbar_alignment_left` | VERIFIED | low | Community-corroborated | none |
| `disable_snap_flyout` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | stock state is value-absent on 26100; covers one of three snap toggles |
| `explorer_compact_view` | VERIFIED | low | Community-corroborated | none |
| `disable_start_recommendations` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | does not hide the Recommended section, only the promoted rows inside it |
| `enable_end_task_taskbar` | VERIFIED | low | Community-corroborated | none |
| `remove_gallery_nav_pane` | VERIFIED | low | Community-corroborated | none |
| `remove_home_nav_pane` | VERIFIED | low | Community-corroborated | none |
| `remove_onedrive_nav_pane` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | stock default is `1` present, not `absent` |
| `disable_toast_notifications` | VERIFIED | medium | Community-corroborated | none |
| `verbose_logon_messages` | VERIFIED | low | Microsoft-documented | none |
| `disable_startup_sound` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | shipped value on 26100 is `1`, so the "Stock Default" option turns a sound ON that the image ships off |
| `disable_accessibility_key_prompts` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `Flags` is a live bitmask; hardcoded 510/126/62 clobbers other sub-settings |
| `classic_context_menu_win11` | VERIFIED | low | Community-corroborated | none |
| `numlock_on_startup` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | HKCU-only write cannot affect the sign-in screen; stock default is not `absent` |
| `disable_start_recommended_section` | VERIFIED (new) | low | Microsoft-documented | ship without the PolicyManager fallback |
| `hide_unsupported_hardware_notice` | VERIFIED (new) | low | Microsoft-documented | none, revert must stay `absent` |
| `disable_phone_companion_start` | VERIFIED (new) | low | Community-corroborated | `requires_reboot` must be false |
| `disable_drag_tray` | VERIFIED-WITH-CORRECTION (new) | low | Community-corroborated | feature renamed "Drop Tray" at 26100.8328; one proposed source is bogus |
| `alt_tab_hide_browser_tabs` | VERIFIED-WITH-CORRECTION (new) | low | Microsoft-documented | same value name at two keys with different enum bases; policy key needs `4`, preference key needs `3` |
| `disable_snap_assist` | VERIFIED (new) | low | Community-corroborated | none |
| `explorer_expand_to_current_folder` | VERIFIED (new) | low | Community-corroborated | none |
| `explorer_restore_folders_at_logon` | VERIFIED (new) | low | Community-corroborated | none |
| `taskbar_full_date_time` | VERIFIED (new) | low | Microsoft-documented | add the documented reboot requirement |
| `hide_recently_added_apps` | VERIFIED (new) | low | Microsoft-documented | applicability is 1703, not 1803; reboot required |
| `taskbar_last_active_click` | VERIFIED (new) | low | Community-corroborated | none |
| `disable_notification_center` | VERIFIED-WITH-CORRECTION (new) | medium | Microsoft-documented | class is `Both` so HKLM is available; reboot required; the calendar-flyout claim is inference |
| `disable_copilot_taskbar` | MOVED | low | Microsoft-documented | relocated to the `ai` category; see `ai.md` |
| `disable_cortana_button` | DELETE | low | Community-corroborated | feature gone from the supported range |
| `disable_meet_now` | DELETE | low | Microsoft-documented | feature gone from the supported range |
| `disable_people_bar` | DELETE (one re-check first) | low | Community-corroborated | feature gone from the supported range, but `PeopleBand.dll` still ships |
| `disable_news_interests` | DELETE (one re-check first) | low | Microsoft-documented | LTSC 2021 applicability never confirmed |
| `disable_chat_taskbar` | DELETE | low | Community-corroborated | button removed in 23H2, below the support floor |

Live-tweak tally: 42. Deletion notes: 5. Moved out: 1.

| Verdict | Count |
|---|---|
| VERIFIED | 22 |
| VERIFIED-WITH-CORRECTION | 18 |
| INCORRECT | 1 |
| DISPUTED | 1 |
| UNVERIFIED | 0 |

## Corrections required

1. **`disable_recent_files` writes to the wrong key.** The YAML writes `ShowRecent` and `ShowFrequent`
   under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`. Both values live under
   `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer` (no `\Advanced`). As authored the tweak
   is a no-op that will nonetheless report "applied", because the status probe reads back the inert
   value it wrote. This is the single most damaging defect in the category.

2. **`disable_aero_shake` has the wrong Stock Default.** Title bar window shake is *disabled* by
   default from Windows 10 build 21277 onward and is off out of the box on every Windows 11 build
   (Settings > System > Multitasking > "Title bar window shake"). The option labelled
   "Enabled (Stock Default)" writes `DisallowShaking = 0`, which *enables* a feature the stock system
   ships disabled. Reverting therefore leaves the machine in a non-stock state.

3. **`numlock_on_startup` cannot do what its info text claims.** The info says the tweak sets
   `InitialKeyboardIndicators` under both the default user profile (used at the logon screen) and the
   user's own account. The effects list contains only the HKCU write. The sign-in screen reads
   `HKU\.DEFAULT\Control Panel\Keyboard\InitialKeyboardIndicators`, which the tweak never touches, so
   NumLock at the sign-in screen is unachievable as authored. Either add the `HKU\.DEFAULT` write
   (which needs admin elevation, and the tweak is declared `elevation: user`) or rewrite the copy.

4. **`numlock_on_startup` Stock Default is not `absent`.** Microsoft documents
   `HKCU\Control Panel\Keyboard\InitialKeyboardIndicators` as `REG_SZ` with a default of `0` and a
   documented range of `0` or `2`, and states that Windows *writes this entry at logoff and shutdown*
   from the live NumLock state. The value therefore exists on essentially every profile. Reverting to
   `absent` is not the stock state, and the tweak will un-apply itself at the next sign-out anyway.

5. **`numlock_on_startup` uses an undocumented value.** `2147483650` (0x80000002) is not in
   Microsoft's documented range. It is the widely used "preserve the high bit plus NumLock on" form
   for Windows 8 and later, but it rests on tier C and D only. The laptop numeric-keypad-overlay
   footgun is also missing from the copy.

6. **`remove_onedrive_nav_pane` Stock Default is wrong.** When OneDrive Personal is installed it
   creates `HKCU\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}` with
   `System.IsPinnedToNameSpaceTree = 1`. The stock state is `1` present, not `absent`. Reverting by
   deleting the value leaves the pin state undefined rather than restoring it.

7. **`taskbar_ungroup_labels` is missing `MMTaskbarGlomLevel`.** From Windows 11 24H2 the combining
   behaviour is split into two controls, one for the primary taskbar and one for taskbars on other
   displays. Writing only `TaskbarGlomLevel` leaves secondary monitors combined. The info copy also
   still warns about Windows 11 21H2 and 22H2, which are below the support floor; that is dead text.

8. **`taskbar_search_mode` has no Stock Default option.** All four options are unlabelled. The
   Windows 11 default is `2` (search box). Without a Stock Default the revert path has no target.

9. **`taskbar_search_mode` is missing `SearchboxTaskbarModeCache`.** Windows treats a missing cache
   value as "no user preference recorded" and re-migrates `SearchboxTaskbarMode` back to `2`.
   Deployment guidance writes the cache value alongside the mode.

10. **`taskbar_hover_time` "Default (400 ms)" writes a value instead of removing it.** The stock state
    is `ExtendedUIHoverTime` absent, confirmed by direct registry inspection on 26100.4061, with the
    shell falling back to its internal 400 ms. Writing an explicit 400 leaves a non-stock artefact
    behind after revert. This is the one unambiguous YAML defect in that tweak.

11. **`taskbar_hover_time` is reported non-functional on the primary target.** Either gate it below
    26100 (it remains correct for LTSC 2021 and Windows 11 up to 23H2) or carry an explicit warning.
    Do not present it as working on 26100+ without a fresh visual test. See the entry and open
    question 3.

12. **`disable_start_recommendations` overstates its effect.** `Start_IrisRecommendations` is the
    Settings > Personalization > Start toggle "Show recommendations for tips, shortcuts, new apps, and
    more". It suppresses the promoted content *inside* the Recommended area. It does not remove the
    Recommended section; that is `HideRecommendedSection`, now shipped separately as
    `disable_start_recommended_section`. Narrow the tweak name, description and info copy.

13. **`disable_search_highlights` is missing the policy companion.** `IsDynamicSearchBoxEnabled` is a
    per-user preference only. The durable route is the ADMX policy "Allow search highlights", which
    writes `EnableDynamicContentInWSB` under
    `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`. Also, the "On (Stock Default)" option
    writing `1` is likely wrong; the fresh-install state could not be sourced.

14. **`disable_accessibility_key_prompts` cannot use hardcoded restore values.** `Flags` is a live
    bitmask of every sub-option for that accessibility feature. 510 / 126 / 62 are the *common*
    defaults. The tweak correctly clears only bit 0x4 (`SKF_HOTKEYACTIVE`, `FKF_HOTKEYACTIVE`,
    `TKF_HOTKEYACTIVE`), but restoring a hardcoded constant overwrites any other sub-setting the user
    changed. The revert should re-set bit 0x4 on the observed value. The `REG_SZ` typing is correct.
    A sign-out requirement is also missing.

15. **`disable_ui_animations` reboot requirement is understated.** `TaskbarAnimations` picks up on an
    Explorer restart, but `MinAnimate` under `Control Panel\Desktop\WindowMetrics` is read at session
    start (or through a `SystemParametersInfo` broadcast). A registry-only write needs a sign-out. The
    info text says "Restart Explorer to apply" and `requires_reboot` is unset.

16. **`explorer_full_path_title` claims the address bar.** The name, description and info text all say
    "title bar and address bar". `CabinetState\FullPath` affects the title bar only; the address bar
    is always a breadcrumb control. On Windows 11 the title bar is the tab strip, so the visible
    effect is limited to the taskbar hover text and the Alt+Tab label.

17. **`open_explorer_to_this_pc` option label is stale.** `LaunchTo = 2` is "Home" on Windows 11 22H2
    and later. The numeric enum is correct and `2` is the default; only the label is wrong.

18. **`seconds_in_tray_clock` is missing a build gate.** `ShowSecondsInSystemClock` is honoured on
    Windows 10, and on Windows 11 only from 22H2 build 22621.1344. That is trivially satisfied on the
    primary target, so the gate now matters only for the LTSC 2021 secondary target, where the value
    does work. Add the gate or a note.

19. **`disable_startup_sound` has the Stock Default backwards on the primary target.** On Windows 11
    24H2 build 26100.4061 (IoT Enterprise LTSC 2024) the shipped value is `DisableStartupSound = 1`,
    and the `BootAnimation` key's last-write timestamp is 2024-04-01, the OS image-build date, against
    an install date of 2025-05-13. The YAML's "Enabled (Stock Default) = 0" therefore enables a boot
    chime the image ships switched off, and the "Disabled" option is a no-op the app will report as a
    successful change. Reachable through System Default under ADR-0003. Verify against a retail Home
    or Pro 26100 image before flipping the literal, because the SKU families may genuinely differ.

20. **`disable_start_recent_items` and `disable_snap_flyout` write a literal where the stock state is
    value-absent.** On 26100.4061 neither `Start_TrackDocs` nor `EnableSnapAssistFlyout` exists on a
    stock install. The shipped default user profile (`C:\Users\Default\NTUSER.DAT`) carries exactly
    one value under `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, namely
    `Start_SearchFiles`, and neither name has a `DefaultValue` declaration under
    `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder`. Both "Stock Default"
    options should write `absent`. Contrast `HideFileExt`, `Hidden` and `UseCompactMode`, which
    Windows itself declares under `Explorer\Advanced\Folder` with `DefaultValue` 1, 2 and 0; those
    three are correct as authored.

21. **`disable_snap_flyout` covers one of three snap toggles.** `twinui.dll` and
    `SettingsHandlers_nt.dll` on 26100.4061 carry `EnableSnapAssistFlyout`, `EnableSnapBar` and
    `EnableSnapAssist` as three independent value names, corresponding to the hover flyout, the
    drag-to-top layout bar and the post-snap suggestion picker. The tweak name reads as "snap layouts"
    but only the first is written, and Win+Z still opens the grid. `disable_snap_assist`, new in this
    revision, now covers the third.

22. **`alt_tab_hide_browser_tabs` must not reuse one enum across two keys.** The same value name,
    `MultiTaskingAltTabFilter`, exists at a policy key and a preference key with **different enum
    bases**. `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` needs `4` for windows-only;
    `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced` needs `3` for the same result.
    Writing `3` at the policy key produces "windows plus 3 recent tabs", the opposite of the requested
    behaviour. If both are written they must be kept in step.

23. **`disable_notification_center` needs three fixes at authoring time.** The shipped ADMX declares
    `class="Both"`, so HKLM is available and is the better choice for a machine-wide action. The ADML
    states a reboot is required. And the claim that it removes the calendar flyout is an inference
    from the Windows 11 shared panel, not documentation; phrase it as "may also remove".

24. **`taskbar_full_date_time` must state the reboot requirement.** The shipped `Taskbar.adml` says so
    verbatim. Without it a user applies the tweak, sees nothing, and concludes it is broken.

25. **`hide_recently_added_apps` applicability is 1703, not 1803.** The ADMX says
    `SUPPORTED_Windows_10_0_RS4` (1803) but Policy CSP Start > `HideRecentlyAddedApps` says
    "Windows 10, version 1703 [10.0.15063] and later". The CSP also documents a reboot requirement.

26. **`disable_start_recommended_section` must ship without the PolicyManager fallback.** Writing
    `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Start` is unnecessary; it is the MDM policy
    cache, not an authoring surface. The direct policy value is read on Pro-class editions.

27. **`disable_drag_tray` must be named for "Drop Tray".** At builds 26100.8328 (24H2), 26200.8328
    (25H2) and 28000.2179 (26H1) the feature was renamed from Drag Tray to Drop Tray and its Settings
    home moved from System > Nearby sharing to System > Multitasking. Users on current servicing will
    not recognise the old name. Separately, the Pureinfotech page originally proposed as a source
    documents an unrelated feature-management override and must not be cited.

28. **`disable_phone_companion_start` must set `requires_reboot: false`.** No source specifies a
    sign-out step. The Microsoft Q&A "restart your PC" line is troubleshooting advice for a panel that
    failed to appear, not a stated requirement.

29. **`hide_unsupported_hardware_notice` and `disable_start_recommended_section` must revert to
    `absent`, never `0`.** Both ADMX `<policy>` elements declare no `enabledValue` and no
    `disabledValue`, so there is no Microsoft-defined `0` state for `HideUnsupportedHardwareNotifications`
    at all. Writing an explicit `0` is undefined behaviour that no source covers.

30. **`disable_copilot_taskbar` has moved.** It is no longer part of this category. Its corrections
    (stale Win+C claim, missing Home SKU gate, deprecation notice) travel with it to `ai.md`.

## Recommended for deletion

Five tweaks target features that do not exist anywhere in the supported platform range. They are not
wrong; they have simply lost their platform. No info block is written for these, because nothing
should ship.

### `disable_cortana_button`

Gated `products: [10]`, writes `ShowCortanaButton` under `Explorer\Advanced`. Windows 11 never pinned
Cortana to the taskbar and does not read the value. Cortana as a standalone app was deprecated in
June 2023 and removed from Windows 10 by a Store update later that year, and LTSC editions ship no
Cortana at all. The value name does still appear in a shipped binary on 26100
(`windowsudk.shellcommon.dll`), which is why the original verdict was VERIFIED, but there is no
supported platform on which the button exists to hide. **Delete.**

### `disable_meet_now`

Gated `products: [10]`, writes the `HideSCAMeetNow` policy under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`. The mechanism is real and tier A
(TaskBar2.admx). Meet Now is a Skype entry point, and Skype was retired on 2025-05-05. Windows 11 has
no Meet Now tray control, and LTSC excludes the consumer app set. **Delete.**

### `disable_people_bar`

Gated `products: [10]`, writes `PeopleBand` under `Explorer\Advanced\People`. The subkey is correct,
which was checked specifically. My People is on Microsoft's deprecated-features list ("no longer being
developed"), announced at 1909, and was removed from Windows 11. People is on the LTSC-excluded app
list.

**One re-check before deletion.** `PeopleBand.dll` still ships on 26100, and My People has never moved
from the deprecated list to the removed-features list, so the feature is deprecated rather than
formally removed. The on-box evidence gathered so far is unusable: `PeopleBand = 0` exists on the
inspected 26100 machine, but the key was last written 2025-12-24, months after install, and the
tweak's own `products: [10]` gate means this app cannot have written it. Confirm on a clean LTSC 2021
(19044) image that the People button does not appear, then delete. **Delete after re-check.**

### `disable_news_interests`

Gated `products: [10]`, writes `ShellFeedsTaskbarViewMode` under `...\CurrentVersion\Feeds`. Key,
value, type and the 0/1/2 enum are all correct. News and Interests was a GA-channel Windows 10
rollout, and Windows 11 uses Widgets, a different mechanism already covered by `debloat:disable_widgets`.
Two further problems: since the March 2024 servicing updates the User Choice Protection Driver watches
the `Feeds` key and reverts writes from non-allowlisted processes, so the write may not stick; and the
durable control was always the `EnableFeeds` machine policy under
`HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Feeds`, which the YAML names in prose but never
writes.

**One re-check before deletion.** LTSC editions do not receive GA-channel feature rollouts and the
News app is explicitly excluded from LTSC, but nobody confirmed against a real LTSC 2021 image whether
News and Interests is present. Confirm, then delete. **Delete after re-check.**

### `disable_chat_taskbar`

Gated `products: [11]`, writes `TaskbarMn` under `Explorer\Advanced`. Microsoft's own 23H2 release
notes state that Chat was removed from the in-box Teams app and that Teams is no longer pinned to the
taskbar from Windows 11 23H2. 23H2 is below the support floor, so on every supported build `TaskbarMn`
has nothing to hide. The backing `ConfigureChatIcon` policy is separately marked deprecated. Note that
`debloat:remove_teams_chat_taskbar` targets the same vanished feature and dies with it. **Delete.**

## New in this revision

Twelve tweaks, each carrying a mechanism confirmed in the adversarial gap-hunt verification pass. Four
came from the High-value set, eight from the Medium and Low set.

| Tweak | Source verification | Verdict there | Why it earns a place |
|---|---|---|---|
| `disable_start_recommended_section` | `_verify-gaps-a-high.md` entry 1 | CONFIRMED | Removes the whole Start Recommended section, which the existing `Start_IrisRecommendations` tweak cannot do |
| `disable_phone_companion_start` | `_verify-gaps-a-high.md` entry 9 | CONFIRMED | Removes the mobile-device panel from Start, a surface with no other control in the corpus |
| `disable_drag_tray` | `_verify-gaps-a-high.md` entry 10 | CORRECTED | Turns off the Drop Tray share overlay that appears when dragging files |
| `hide_unsupported_hardware_notice` | `_verify-gaps-a-high.md` entry 11 | CONFIRMED | Clears the "system requirements not met" watermark and About banner |
| `disable_snap_assist` | `_verify-gaps-a-medlow.md` item 23 | CONFIRMED | The post-snap suggestion picker, one of the three snap toggles the corpus only partly covered |
| `alt_tab_hide_browser_tabs` | `_verify-gaps-a-medlow.md` item 24 | CORRECTED | Restores Alt+Tab to windows only, a frequent complaint with no existing control |
| `taskbar_last_active_click` | `_verify-gaps-a-medlow.md` item 26 | CONFIRMED | Click a grouped taskbar button to jump straight to the last active window |
| `explorer_expand_to_current_folder` | `_verify-gaps-a-medlow.md` item 27 | CONFIRMED | Keeps the Explorer tree in sync with the open folder |
| `explorer_restore_folders_at_logon` | `_verify-gaps-a-medlow.md` item 28 | CONFIRMED | Reopens the previous session's Explorer windows |
| `taskbar_full_date_time` | `_verify-gaps-a-medlow.md` item 29 | CONFIRMED | Full date and AM/PM in the tray clock, tier A policy |
| `hide_recently_added_apps` | `_verify-gaps-a-medlow.md` item 30 | CONFIRMED | Removes the "Recently added" list from Start, tier A policy |
| `disable_notification_center` | `_verify-gaps-a-medlow.md` item 32 | CORRECTED | The middle ground between "all toasts off" and "leave it alone" that the corpus lacked |

Two interface proposals from the same pass were **rejected and are deliberately not included**:

- **`disable_widgets_lock_screen`.** The shipped `NewsAndInterests.admx` and the Policy CSP prescribe
  opposite writes for the same intent (ADMX enables with `0`, CSP says `1` disables), the inversion is
  systematic across the sibling `DisableWidgetsBoard` policy so neither side can be called a
  transcription error, and the CSP lists the applicable OS as Insider Preview only. Writing the wrong
  value would leave enabled a surface the user asked to remove while the app reported success, which
  is a direct did-it-work-contract violation. It is also probably redundant with
  `debloat:disable_widgets`, which sets `AllowNewsAndInterests = 0` device-wide.
- **`disable_new_app_alert`.** `NoNewAppAlert` is real (WindowsExplorer.admx, `class="Machine"`,
  `supportedOn` Windows 8), but on 26100 it appears in exactly one binary, `SHCore.dll`, the shell
  policy table, and in no feature component, so its real-world effect is unproven. More importantly
  the notification it suppresses is the user's only signal that an installer changed their file or
  protocol associations, a well-known adware and browser-hijack vector. Trading a security signal for
  the removal of a rare toast is a bad deal.

A third, `disable_spotlight_desktop`, was confirmed in the same pass but is assigned to another
category document.

## Merge candidates

Conservative, same-subsystem merges only.

**Group 1: Start menu Recommended area.** `disable_start_recommendations`,
`disable_start_recommended_section`, `disable_start_recent_items` and `hide_recently_added_apps` now
form a ladder over the same UI region: promoted rows only, recent files only, recently added apps
only, and the whole section. A single "Start menu Recommended area" tweak with escalating options is
defensible. Granularity lost: `Start_TrackDocs` also drives File Explorer Recent and taskbar jump
lists, so a user who wants jump lists but not Start recommendations would lose that combination, and
`HideRecommendedSection` is a policy while the others are preferences with different revert semantics.
Merge only if the second effect of each option is clearly described.

**Group 2: File Explorer navigation pane nodes.** `remove_gallery_nav_pane`, `remove_home_nav_pane`,
`remove_onedrive_nav_pane`, all writing `System.IsPinnedToNameSpaceTree` under a per-user CLSID key.
Granularity lost: users genuinely combine these rather than choose between them, and each has a
different applicability gate (Gallery needs 22631+, Home needs Windows 11, OneDrive needs OneDrive
installed). Under the current one-choice-per-tweak model a merge would force a combinatorial option
list. Keep them separate unless the engine gains multi-select.

**Group 3: snap behaviour.** `disable_snap_flyout` (`EnableSnapAssistFlyout`) and `disable_snap_assist`
(`SnapAssist`) are two of the three Multitasking snap toggles; `EnableSnapBar` is the third and is not
in the corpus. A single "Snap window helpers" tweak with per-surface options would be honest about
what each one actually suppresses, which the current naming is not.

**Group 4: recent-activity privacy.** `disable_recent_files` and `disable_start_recent_items` suppress
the same underlying recent-activity store on different surfaces. A weaker candidate than Group 1,
because the surfaces are independently useful.

**Explicitly not merge candidates.** The taskbar button family looks like one group but is not: a user
combines these rather than choosing between them, and after the five deletions little of it remains.
`enable_dark_mode` and `disable_transparency` share a registry key but are orthogonal choices.
`disable_toast_notifications` and `disable_notification_center` are adjacent but not interchangeable:
one silences banners, the other removes the history panel and the tray entry point.

## Open questions

1. **`disable_aero_shake` stock representation on Windows 11.** Every source agrees the gesture is off
   by default from Windows 10 build 21277 and on Windows 11, but none says whether that is expressed
   as `DisallowShaking = 1` present or as the value being absent with the shell defaulting to
   disabled. That decides how the revert should be written.

2. **Fresh-install presence of several "default 1" values.** Partly resolved. `EnableSnapAssistFlyout`
   and `Start_TrackDocs` are confirmed **absent** on a stock 26100.4061 install. Still open for
   `IsDynamicSearchBoxEnabled`, `TaskbarAnimations`, `ShowTaskViewButton`, `TaskbarAl` and
   `ToastEnabled`: all five are present on the inspected live profile, but that profile has been used
   interactively and touched by tweak tooling, and none has a `DefaultValue` declaration, so it cannot
   be told apart whether Windows materialised them at first logon or the user did. `HideFileExt`,
   `Hidden` and `UseCompactMode` are settled in the YAML's favour.

3. **`taskbar_hover_time` on 24H2 and 25H2.** The sharpest open question in this file. The name
   survives in `Taskbar.dll` on 26100.4061 but sits in the ported legacy taskband string block, next
   to values known to be inert on Windows 11, while the maintained elevenforum tutorial states it
   stopped working at 24H2. Only a visual test settles it: set `ExtendedUIHoverTime` to 1, restart
   Explorer, and observe whether the thumbnail appears immediately or after the usual pause.

4. **`disable_startup_sound` SKU spread and sufficiency.** The shipped value on 26100.4061 IoT
   Enterprise LTSC 2024 is `1`, established from the untouched image-build key timestamp. Whether
   retail Home and Pro 26100 images also ship `1` decides whether the fix is "flip the literal" or
   "gate by SKU", which the `windows:` block cannot currently express since it has no edition axis.
   Separately, whether the HKLM value alone silences the chime or the per-user sound-scheme entry is
   also required is unconfirmed.

5. **`disable_recent_files` behaviour on recent builds.** One tier C source claims that on Windows 11
   builds from 22635.3930 onward, clearing `ShowRecent` *permanently clears* the Recent list rather
   than hiding it. If true on retail 24H2 and 25H2, the revert is not restorative and the tweak's
   `reversible: true` claim is wrong. Confirm before shipping the key-path fix.

6. **`open_explorer_to_this_pc` third and fourth enum values.** Sources disagree on whether `3` means
   Downloads or the personal OneDrive folder, and whether `0` is meaningful. Not load-bearing for the
   two options shipped.

7. **Which HKCU shell values Windows itself materialises at first interactive logon.** The question
   behind open question 2 and behind several "Stock Default" judgements across this file. The shipped
   `C:\Users\Default\NTUSER.DAT` on 26100.4061 is nearly empty under
   `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` (one value, `Start_SearchFiles`), so
   every other value must be written either by Explorer at first logon or by the user. Settle it once
   by creating a fresh local account on a clean 26100 image, signing in, signing out, and dumping that
   profile's `Explorer\Advanced`, `Themes\Personalize` and `PushNotifications` keys. That one dump
   resolves the stock-default question for roughly a third of this category and should be done before
   any of the `absent` corrections are applied to the YAML.

8. **`disable_snap_assist` stock representation.** Whether `SnapAssist` is present and `1` on a stock
   profile or absent could not be established from an admissible source. Either way the effective
   behaviour is identical (Snap Assist on), so this is not a harmful-revert case, but it decides
   whether the revert should write `1` or delete.

9. **`hide_unsupported_hardware_notice` apply latency.** No source states whether the desktop
   watermark clears on an Explorer restart or only at the next sign-in. The Settings About banner is
   redrawn when the page is reopened. Observe on a machine that actually shows the watermark.

10. **`alt_tab_hide_browser_tabs` apply latency.** The shipped `Multitasking.adml` states no reboot
    requirement, unlike three of the other new policy tweaks, but does not say when the change lands.
    `twinui.dll` reads the value; whether it caches it per session is unconfirmed.

## Tweak entries

### `enable_dark_mode` Dark mode

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`

| Value | Type | Dark | Light (Stock Default) |
|---|---|---|---|
| `AppsUseLightTheme` | REG_DWORD | `0` | `1` |
| `SystemUsesLightTheme` | REG_DWORD | `0` | `1` |

These are the two values behind Settings > Personalization > Colors > "Choose your mode".
`AppsUseLightTheme` drives the theme applied to apps and File Explorer content; `SystemUsesLightTheme`
drives the shell surfaces (taskbar, Start, Action Center). The polarity is inverted relative to the
tweak name, which the YAML gets right: `0` means "do not use light", that is, dark. Setting only one of
the two produces a mixed light and dark desktop; the YAML correctly writes both.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Switches Windows, the shell and your apps to the dark colour theme in one step.**

      ## What it does
      Writes `AppsUseLightTheme` and `SystemUsesLightTheme` under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`, the two values behind
      Settings > Personalization > Colors > "Choose your mode". Both are set to 0 for dark, which
      covers apps and File Explorer as well as the taskbar, Start and Action Center.

      ## Benefits
      - **Both surfaces together**: apps and shell flip in one action instead of two toggles
      - **Easier on the eyes**: less glare in dim rooms and at night
      - **No policy needed**: a plain per-user preference, no admin rights involved

      ## Drawbacks
      - **Not uniform**: some legacy dialogs, MMC snap-ins and Control Panel applets ignore the theme
      - **App restarts**: a few already-running apps only pick up the new theme when relaunched
      - **Worse in bright rooms**: dark text on light is easier to read in direct sunlight

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1809 and later
      - **Takes effect**: immediately, though some running apps need a restart
      - **Reverting**: restores the previous values from the snapshot
      - Windows updates do not reset this.

      ## Recommendation
      Apply it if you work in a dim room or simply prefer dark surfaces. Skip it if you rely on an
      app that renders badly in dark mode, since the theme is all or nothing.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Choose Dark or Light Mode for Colors in Windows 11](https://www.elevenforum.com/t/choose-dark-or-light-mode-for-colors-in-windows-11.555/)
      - [Enable Dark mode on Windows](https://pureinfotech.com/enable-dark-mode-windows-10/)
```

**Sources:**

1. Choose Dark or Light Mode for Colors in Windows 11, https://www.elevenforum.com/t/choose-dark-or-light-mode-for-colors-in-windows-11.555/ (tier C)
2. Enable Dark mode on Windows, https://pureinfotech.com/enable-dark-mode-windows-10/ (tier C)
3. Change Between Light and Dark Mode for Default App Mode, https://www.tweaknow.com/RegTweakAppModeTheme.php (tier C)

### `disable_transparency` Transparency effects

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`,
`EnableTransparency` (REG_DWORD). Off = `0`, On (Stock Default) = `1`.

This is the Settings > Personalization > Colors > "Transparency effects" toggle. Turning it off makes
the taskbar, Start menu, Settings app and flyouts render opaque instead of using the acrylic and Mica
materials. Transparency is on by default on Windows 11 and on Windows 10. Microsoft does not publish
the value name; it is confirmed by several tier C references and is directly observable in Settings.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Makes the taskbar, Start menu and flyouts solid instead of translucent.**

      ## What it does
      Sets `EnableTransparency` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`, the value behind
      Settings > Personalization > Colors > "Transparency effects". Shell surfaces stop using the
      acrylic and Mica blur materials and render flat.

      ## Benefits
      - **Cleaner contrast**: text on the taskbar and Start stops competing with the wallpaper
      - **Less compositing**: a small reduction in DWM work on very weak integrated graphics
      - **Consistent look**: surfaces no longer change appearance as the wallpaper scrolls behind them

      ## Drawbacks
      - **Purely aesthetic loss**: you give up the Fluent design look with nothing gained visually
      - **Not a performance tweak**: on any modern GPU the compositing saving is noise
      - **State can drift**: Windows also disables transparency automatically in battery-saver and
        reduced-effects modes, so what you see may not match the written value

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: immediately, within a second or two
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it if the blur makes taskbar and Start text hard to read against your wallpaper. Do not
      apply it expecting a frame-rate gain; that is not what this does.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable Transparency in Windows 11](https://winaero.com/how-to-enable-or-disable-transparency-in-windows-11/)
      - [Enable or Disable Transparency Effects in Windows 11](https://www.ninjaone.com/blog/enable-or-disable-transparency-effects-in-windows-11/)
```

**Sources:**

1. Enable or Disable Transparency in Windows 11, https://winaero.com/how-to-enable-or-disable-transparency-in-windows-11/ (tier C)
2. Enable or Disable Transparency Effects in Windows 11, https://www.ninjaone.com/blog/enable-or-disable-transparency-effects-in-windows-11/ (tier C)

### `disable_ui_animations` Window animations

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**

| Key | Value | Type | Off | On (Stock Default) |
|---|---|---|---|---|
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarAnimations` | REG_DWORD | `0` | `1` |
| `HKCU\Control Panel\Desktop\WindowMetrics` | `MinAnimate` | REG_SZ | `"0"` | `"1"` |

`MinAnimate` is the long-standing "Animate windows when minimizing and maximizing" performance option,
stored as a *string*. The REG_SZ typing is correct and load-bearing: written as a DWORD it is ignored.
`TaskbarAnimations` covers taskbar and Start menu animation, including the thumbnail preview slide and
fade. Both default to enabled. The YAML deliberately avoids `UserPreferencesMask`, which is the right
call, since that is a packed bitmask and a blind rewrite would clobber unrelated preferences.

**Corrections needed:** `requires_reboot` is unset and the info text says "Restart Explorer to apply".
`TaskbarAnimations` does pick up on an Explorer restart, but `MinAnimate` is read at session start (or
through a `SystemParametersInfo` broadcast), so a registry-only write needs a sign-out. Either mark
the tweak as needing a sign-out or state that the window-animation half will not take effect until
then.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the slide and fade animations when windows open, close, minimise and maximise.**

      ## What it does
      Sets `TaskbarAnimations` to 0 under `Explorer\Advanced` and the string value `MinAnimate` to
      "0" under `HKCU\Control Panel\Desktop\WindowMetrics`. Together these cover taskbar and Start
      animation and the minimise/maximise window transition. It deliberately does not touch
      `UserPreferencesMask`, which is a packed bitmask that a blind rewrite would corrupt.

      ## Benefits
      - **Snappier feel**: a short but real delay disappears from every window state change
      - **Helps weak hardware**: most noticeable on low-end GPUs and remote desktop sessions
      - **Targeted**: only two values, so unrelated visual preferences are left alone

      ## Drawbacks
      - **Lost motion cues**: harder to see where a window minimised to
      - **Split timing**: the taskbar half applies on an Explorer restart, the window half only
        after you sign out
      - **Overwritten by Settings**: Accessibility > Visual effects > "Animation effects" rewrites
        both values plus others, so flipping that toggle later undoes this

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after sign-out for window animations; the taskbar half applies when
        Explorer restarts
      - **Reverting**: restores the previous values from the snapshot

      ## Recommendation
      Worth it on low-end hardware or over remote desktop, where the animation delay is real. Skip it
      if you use the motion to track where windows went, which some people genuinely rely on.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable Animate Windows when Minimizing and Maximizing](https://www.tenforums.com/tutorials/126788-enable-disable-animate-windows-when-minimizing-maximizing.html)
      - [Enable or Disable Animations in the Taskbar](https://www.tenforums.com/tutorials/126795-enable-disable-animations-taskbar-windows-10-a.html)
```

**Sources:**

1. Enable or Disable Animate Windows when Minimizing and Maximizing, https://www.tenforums.com/tutorials/126788-enable-disable-animate-windows-when-minimizing-maximizing.html (tier C)
2. Disable Animate Windows when Minimizing and Maximizing in Windows 10, https://winaero.com/disable-animate-windows-when-minimizing-and-maximizing-in-windows-10/ (tier C)
3. Enable or Disable Animations in the Taskbar in Windows 10, https://www.tenforums.com/tutorials/126795-enable-disable-animations-taskbar-windows-10-a.html (tier C)

### `disable_aero_shake` Aero Shake

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `DisallowShaking`
(REG_DWORD). Disabled = `1`, Enabled = `0`.

`DisallowShaking` gates the shake-to-minimize gesture and the polarity is correct: `1` disallows
shaking. There is a policy-backed twin, `NoWindowMinimizingShortcuts` under
`HKCU\Software\Policies\Microsoft\Windows\Explorer`, written by the ADMX setting "Turn off Aero Shake
window minimising mouse gesture". The YAML uses the preference rather than the policy, which is the
right choice for a user-level reversible tweak.

**Corrected Stock Default:** the option currently labelled "Enabled (Stock Default) = 0" is not the
stock state on any supported build. Microsoft disabled the gesture by default from Windows 10 build
21277 onward, and Windows 11 ships it off (Settings > System > Multitasking > "Title bar window shake"
is off out of the box). The stock state is "shake off"; `0` turns it on.

**Wrong mechanism as currently authored:**

```
Disabled = DisallowShaking 1
Enabled (Stock Default) = DisallowShaking 0     <-- writes a non-stock state on revert
```

Whether stock is `DisallowShaking = 1` present or the value absent with the shell defaulting to
disabled is open question 1; that decides which of the two corrected shapes to use.

**Corrections needed:** the Stock Default label is attached to the wrong option. A user who applies
then reverts ends up with shake *enabled* on a machine that shipped with it disabled. Fix by relabelling
so the off state is the default and offering `0` as the explicit "turn the gesture on" choice, and
resolve open question 1 before choosing between `1` and `absent` for that default.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops windows from minimising when you grab a title bar and shake it.**

      ## What it does
      Sets `DisallowShaking` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`. That is the preference
      behind Settings > System > Multitasking > "Title bar window shake", so no other window
      behaviour changes.

      ## Benefits
      - **No accidental minimise**: dragging a window quickly no longer hides everything else
      - **Predictable dragging**: useful if you reposition windows often or use a high-DPI trackpad
      - **Fully reversible**: a single per-user value with no policy involved

      ## Drawbacks
      - **Gesture is gone**: there is no partial setting, it is on or off
      - **Often no visible change**: Windows 11 already ships the gesture off, so applying it may do
        nothing you can see
      - **Preference, not policy**: the Settings toggle stays available, so it can be turned back on

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2, where the gesture may still
        be on
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it on a machine where the gesture is still active and keeps firing by accident. If you
      deliberately use shake-to-minimise, leave it alone.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable Title Bar Window Shake in Windows 11](https://www.elevenforum.com/t/enable-or-disable-title-bar-window-shake-in-windows-11.2078/)
      - [How to Enable or Disable Aero Shake in Windows 10](https://www.tenforums.com/tutorials/4417-how-enable-disable-aero-shake-windows-10-a.html)
```

**Sources:**

1. Enable or Disable Title Bar Window Shake in Windows 11, https://www.elevenforum.com/t/enable-or-disable-title-bar-window-shake-in-windows-11.2078/ (tier C)
2. Enable Minimize Windows with Title Bar Shake in Windows 11 (Aero Shake), https://winaero.com/enable-minimize-windows-with-title-bar-shake-in-windows-11-aero-shake/ (tier C)
3. How to Enable or Disable Aero Shake in Windows 10, https://www.tenforums.com/tutorials/4417-how-enable-disable-aero-shake-windows-10-a.html (tier C)

### `show_file_extensions` File name extensions

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `HideFileExt`
(REG_DWORD). Show = `0`, Hide (Stock Default) = `1`.

`HideFileExt` is the Folder Options > View > "Hide extensions for known file types" checkbox. Windows
ships with it checked, that is `HideFileExt = 1`. Setting `0` shows the extension for every registered
file type. The default of `1` is not merely community-reported: Windows itself declares it under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder` with `DefaultValue = 1`, so
the written literal is exactly what Explorer's own Folder Options mechanism writes.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Shows the real file extension on every file, so a disguised executable cannot hide.**

      ## What it does
      Sets `HideFileExt` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the value behind Folder
      Options > View > "Hide extensions for known file types". Explorer then shows `.exe`, `.js`,
      `.scr` and every other registered extension.

      ## Benefits
      - **Anti-phishing**: `invoice.pdf.exe` stops rendering as `invoice.pdf` with a PDF icon
      - **Cheapest hardening**: one per-user value, no admin rights, no side effects
      - **Clearer file types**: you can tell a `.txt` from a `.md` at a glance

      ## Drawbacks
      - **Noisier lists**: every file name gets longer
      - **Rename hazard**: a user renaming a file can now strip the extension by accident
      - **Per user only**: other accounts on the machine are unaffected

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also every Windows 10 build
      - **Takes effect**: after restarting Explorer, or press F5 in an open window
      - **Reverting**: restores the previous value from the snapshot
      - Windows updates do not reset this.

      ## Recommendation
      Apply it on any machine you administer. The only reason to skip it is a shared family PC where
      someone is likely to rename files and lose the extension.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Show or Hide File Name Extensions in Windows 11](https://www.elevenforum.com/t/show-or-hide-file-name-extensions-for-known-file-types-in-windows-11.898/)
      - [Explorer\Advanced registry reference](https://renenyffenegger.ch/notes/Windows/registry/tree/HKEY_CURRENT_USER/Software/Microsoft/Windows/CurrentVersion/Explorer/Advanced/index)
```

**Sources:**

1. Show or Hide File Name Extensions for Known File Types in Windows 11, https://www.elevenforum.com/t/show-or-hide-file-name-extensions-for-known-file-types-in-windows-11.898/ (tier C)
2. `HKEY_CURRENT_USER\...\Explorer\Advanced` registry reference, https://renenyffenegger.ch/notes/Windows/registry/tree/HKEY_CURRENT_USER/Software/Microsoft/Windows/CurrentVersion/Explorer/Advanced/index (tier C)
3. Shipped OS state on 26100.4061: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\HideFileExt` declares `DefaultValue = 1` (tier A, product artifact)

### `show_hidden_files` Hidden files and folders

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `Hidden`
(REG_DWORD). Show = `1`, Hide (Stock Default) = `2`.

The enum is genuinely `1` and `2`, not `1` and `0`. This is a historical quirk of the Folder Options
radio group: `Hidden = 1` maps to "Show hidden files, folders, and drives" and `Hidden = 2` maps to
"Don't show hidden files, folders, and drives". Writing `0` is not a documented state and does not
reliably mean "hide". The stock value of `2` is declared by Windows itself under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder`, so the literal is correct.

Protected operating-system files are governed separately by `ShowSuperHidden` (REG_DWORD, default 0)
in the same key, corresponding to the Folder Options checkbox "Hide protected operating system files
(Recommended)". The two are independent: `Hidden = 1` reveals user-hidden items but leaves
system-hidden items (files carrying both Hidden and System attributes, such as `pagefile.sys`,
`hiberfil.sys`, `System Volume Information`) invisible. The tweak intentionally leaves
`ShowSuperHidden` alone, which is the correct and safer behaviour.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Reveals hidden files and folders in File Explorer, without exposing protected system files.**

      ## What it does
      Sets `Hidden` to 1 under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
      the Folder Options > View radio button "Show hidden files, folders, and drives". It leaves
      `ShowSuperHidden` alone, so protected OS files such as `pagefile.sys` and
      `System Volume Information` stay invisible.

      ## Benefits
      - **Reach app data**: `%AppData%`, `ProgramData` and dotfile config folders become browsable
      - **Safer than the alternative**: protected system files remain hidden, unlike the "show
        everything" route
      - **Troubleshooting**: hidden caches and leftover installer folders become visible

      ## Drawbacks
      - **Busier folders**: hidden metadata files such as `desktop.ini` clutter listings
      - **Deletion risk**: someone may remove an application's hidden data folder as "clutter"
      - **EDR noise**: several malware families flip this value, and a Sigma detection rule exists,
        so on a monitored endpoint the write may raise an alert

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also every Windows 10 build
      - **Takes effect**: after restarting Explorer, or press F5 in an open window
      - **Reverting**: restores the previous value from the snapshot
      - The value enum is 1 for show and 2 for hide; 0 is not a valid state.

      ## Recommendation
      Apply it if you edit config files or troubleshoot apps. Leave it off on a machine used by
      someone who might delete a hidden folder they do not recognise.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Explorer\Advanced registry reference](https://renenyffenegger.ch/notes/Windows/registry/tree/HKEY_CURRENT_USER/Software/Microsoft/Windows/CurrentVersion/Explorer/Advanced/index)
      - [What does the SuperHidden registry value control?](https://fleexlab.blogspot.com/2017/08/what-does-superhidden-registry-value.html)
      - [Sigma rule: Displaying Hidden Files Feature Disabled](https://detection.fyi/sigmahq/sigma/windows/registry/registry_set/registry_set_hide_file/)
```

**Sources:**

1. `HKEY_CURRENT_USER\...\Explorer\Advanced` registry reference (documents `Hidden` default 2, `ShowSuperHidden` default 0), https://renenyffenegger.ch/notes/Windows/registry/tree/HKEY_CURRENT_USER/Software/Microsoft/Windows/CurrentVersion/Explorer/Advanced/index (tier C)
2. What does the SuperHidden Registry value control?, https://fleexlab.blogspot.com/2017/08/what-does-superhidden-registry-value.html (tier C)
3. Sigma rule: Displaying Hidden Files Feature Disabled, https://detection.fyi/sigmahq/sigma/windows/registry/registry_set/registry_set_hide_file/ (tier C)
4. Shipped OS state on 26100.4061: `HKLM\...\Explorer\Advanced\Folder\Hidden` declares `DefaultValue = 2` (tier A, product artifact)

### `open_explorer_to_this_pc` Open File Explorer to This PC

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `LaunchTo`
(REG_DWORD). This PC = `1`, Home / Quick Access (Stock Default) = `2`.

`LaunchTo` is the Folder Options > General > "Open File Explorer to" dropdown. `1` is This PC and `2`
is the default landing page. The numeric default is `2` and the YAML has that right.

**Corrected option label:** on Windows 11 22H2 and later the value-2 destination is called **Home**,
not Quick Access. Quick Access survives as a pinned and recent section inside Home, and Windows 11
does not use a third enum value for Home.

**Wrong label as currently authored:**

```
"Quick Access (Stock Default)" = 2     <-- value correct, label stale on Windows 11
```

Some tier C references also mention a value `3` (variously Downloads or the personal OneDrive folder)
and a value `0`; those references disagree with each other, so neither is treated as established. See
open question 6.

**Corrections needed:** relabel the second option "Home / Quick Access" or make the label
version-aware. The value itself is correct and needs no change.

**Ready-to-paste info block:**

```yaml
    info: |
      **File Explorer opens on your drives instead of a feed of recent files.**

      ## What it does
      Sets `LaunchTo` to 1 under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
      the Folder Options > General > "Open File Explorer to" dropdown. Value 1 is This PC; the stock
      value 2 is the Home page (called Quick Access on Windows 10).

      ## Benefits
      - **Straight to drives**: no extra click to reach C:, D: or a mapped network drive
      - **Shoulder-surfing**: recent file names are not on screen the moment Explorer opens
      - **Stable landing**: This PC always looks the same, unlike a feed that reorders itself

      ## Drawbacks
      - **Loses pins**: pinned folders live in Home and are one click further away
      - **Slower for recents workflows**: if you reopen files from the recent list, this hurts
      - **Ordering matters**: pair it with hiding Home carefully, see below

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous value from the snapshot
      - If you also hide the Home node from the navigation pane, apply this one first, otherwise
        Explorer is configured to open a node you removed.

      ## Recommendation
      Apply it if you navigate by folder tree. Skip it if you work mainly from pinned folders and
      recent files, since Home is where both live.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [How to change the startup page in File Explorer on Windows 11](https://pureinfotech.com/open-file-explorer-this-pc-instead-quick-access-windows-11/)
      - [Open File Explorer To This PC By Default](https://memstechtips.com/set-file-explorer-launch-this-pc-regedit/)
```

**Sources:**

1. How to change startup page on File Explorer for Windows 11 (states `1` = This PC, `2` = Home, and that Home is the default), https://pureinfotech.com/open-file-explorer-this-pc-instead-quick-access-windows-11/ (tier C)
2. Open File Explorer To This PC By Default (Windows 10 and 11), https://memstechtips.com/set-file-explorer-launch-this-pc-regedit/ (tier C)

### `explorer_full_path_title` Full path in Explorer title

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\CabinetState`, `FullPath`
(REG_DWORD). On = `1`, Off (Stock Default) = `0`.

`FullPath` under `CabinetState` makes Explorer put the fully qualified path rather than the leaf
folder name into the window title. Key, name, type and the default of `0` are confirmed.

**Corrected scope claim:** it does **not** change the address bar. The address bar is always a
breadcrumb control and already shows the path segments. On Windows 11 the Explorer title bar is the
tab strip, so the visible payoff shrinks to the taskbar hover tooltip, the Alt+Tab label and
window-manager tooling.

**Wrong claim as currently authored:** the tweak name, description and info text all say "title bar
and address bar". The address bar half is false on every build.

**Corrections needed:** remove the address-bar claim from the name, description and info text, and
note the reduced visibility on Windows 11's tabbed Explorer.

**Ready-to-paste info block:**

```yaml
    info: |
      **Puts the full folder path in the Explorer window title instead of just the folder name.**

      ## What it does
      Sets `FullPath` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\CabinetState`. Explorer then titles
      the window `C:\Users\You\Projects\build` rather than `build`. The address bar is unaffected;
      it is a breadcrumb control and already shows the path.

      ## Benefits
      - **Disambiguates windows**: several folders named `src` stop looking identical in Alt+Tab
      - **Better taskbar hover**: the tooltip shows where each window actually is
      - **Window tooling**: scripts and window managers that read the title get the full path

      ## Drawbacks
      - **Small payoff on Windows 11**: the title bar is the tab strip, so you mostly see this in
        Alt+Tab and taskbar tooltips
      - **Truncation**: long paths get cut off awkwardly in narrow tooltips
      - **No address bar change**: despite what many guides claim, the breadcrumb does not change

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10, where the effect is directly
        visible in the title bar
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it if you regularly juggle several similarly named folders. On Windows 11 alone the gain
      is small enough that it is fine to skip.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Turn On or Off Display Full Path in Title Bar of File Explorer in Windows 11](https://www.elevenforum.com/t/turn-on-or-off-display-full-path-in-title-bar-of-file-explorer-in-windows-11.3585/)
      - [Display Full Path in Title Bar of File Explorer in Windows 10](https://www.tenforums.com/tutorials/3430-display-full-path-title-bar-file-explorer-windows-10-a.html)
```

**Sources:**

1. Turn On or Off Display Full Path in Title Bar of File Explorer in Windows 11, https://www.elevenforum.com/t/turn-on-or-off-display-full-path-in-title-bar-of-file-explorer-in-windows-11.3585/ (tier C)
2. Display Full Path in Title Bar of File Explorer in Windows 10, https://www.tenforums.com/tutorials/3430-display-full-path-title-bar-file-explorer-windows-10-a.html (tier C)

### `disable_recent_files` Recent files and folders in Explorer

**Verdict:** INCORRECT

**Mechanism (corrected):** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer` (no `\Advanced`)

| Value | Type | Off | On (Stock Default) |
|---|---|---|---|
| `ShowRecent` | REG_DWORD | `0` | `1` |
| `ShowFrequent` | REG_DWORD | `0` | `1` |

These two back the File Explorer Options > General > Privacy checkboxes "Show recently used files" and
"Show frequently used folders". The value names, types, enum and defaults are right.

**Wrong mechanism as currently authored:**

```
HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced   <-- wrong key on both effects
  ShowRecent    (REG_DWORD)
  ShowFrequent  (REG_DWORD)
```

Written under `Advanced` these are inert values Explorer never reads, so the tweak applies cleanly and
does nothing. This is exactly the failure mode that makes a corpus dangerous: the status probe reads
back what it wrote and reports success.

**Corrections needed:** change the key on **both** effects from
`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` to
`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer`. Before shipping the fix, resolve open
question 5: one tier C source reports that on Windows 11 builds from 22635.3930 onward, clearing
`ShowRecent` *permanently clears* the Recent list rather than hiding it, which would make the revert
non-restorative and the `reversible: true` claim wrong.

**Ready-to-paste info block:**

```yaml
    info: |
      **Empties the recent files and frequent folders lists from the File Explorer Home page.**

      ## What it does
      Sets `ShowRecent` and `ShowFrequent` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer`, the two File Explorer Options >
      General > Privacy checkboxes. Explorer stops surfacing what you opened and where you have been
      working.

      ## Benefits
      - **Shoulder-surfing**: file and project names are no longer on screen on a shared display
      - **Quieter Home page**: no auto-updating feed when you open a window
      - **Two checkboxes in one**: covers both the recent-files and frequent-folders lists

      ## Drawbacks
      - **Empty Home page**: unless you have pinned folders, Home shows almost nothing
      - **Slower reopening**: you lose the fastest route back to a file you just closed
      - **Not a wipe**: this hides lists, it does not clear `%AppData%\Microsoft\Windows\Recent` or
        stop apps recording their own most-recently-used lists

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous values from the snapshot
      - On some recent builds turning `ShowRecent` off is reported to clear the stored list rather
        than hide it, so previously recorded entries may not come back on revert.

      ## Recommendation
      Apply it on a machine whose screen other people see, such as a shared desk or a machine you
      present from. Skip it if you navigate primarily by recent files.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Group Policy: disable Recent Files and Frequent Folders in Explorer](https://matthewhill.uk/windows/group-policy-disable-recent-files-frequent-folder-explorer/)
      - [How to Remove Recent Files in File Explorer](https://www.ninjaone.com/blog/how-to-remove-recent-files-in-file-explorer/)
```

**Sources:**

1. Explorer Group Policy: Disable Recent Files / Frequent Folders (Group Policy Preferences recipe giving the key without `Advanced`), https://matthewhill.uk/windows/group-policy-disable-recent-files-frequent-folder-explorer/ (tier C)
2. How to Remove Recent Files in File Explorer, Windows 11 (states explicitly that the value is in the main `Explorer` key, not `Explorer\Advanced`), https://www.ninjaone.com/blog/how-to-remove-recent-files-in-file-explorer/ (tier C)

### `disable_start_recent_items` Start recent items tracking

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `Start_TrackDocs`
(REG_DWORD). Off = `0`, On (Stock Default) = **`absent`**.

`Start_TrackDocs` is the Settings > Personalization > Start toggle "Show recently opened items in
Start, Jump Lists, and File Explorer". It gates the shared recent-activity store, so clearing it
empties the Start Recommended file list, the taskbar and Start jump-list Recent sections, and
Explorer's Recent files at once. Key, name and type are confirmed at tier A: the literal string
`Start_TrackDocs` is present in `shell32.dll`, `StartTileData.dll`, `Windows.Internal.Shell.Broker.dll`,
`gpprefcl.dll` and `AssignedAccessManager.dll` on 26100.4061.

**Wrong Stock Default as currently authored:**

```
"On (Stock Default)" = Start_TrackDocs 1     <-- stock is value-absent on 26100
```

On a stock 26100.4061 install the value does not exist. The shipped default user profile
(`C:\Users\Default\NTUSER.DAT`) carries exactly one value under
`Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, namely `Start_SearchFiles`, and
`Start_TrackDocs` has no `DefaultValue` declaration under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder` (unlike `HideFileExt`,
`Hidden` and `UseCompactMode`, which do). The shell treats absence as "tracking on" and the Settings
toggle only materialises the value once the user moves it.

**Corrections needed:** the "On (Stock Default)" option should write `absent`, not `1`. Behaviourally
`1` and absent are identical to the shell, so this is a pristine-state defect rather than a
behavioural one, but it means a user who never touched this setting cannot be returned to a pristine
registry.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows recording what you opened, across Start, jump lists and File Explorer at once.**

      ## What it does
      Sets `Start_TrackDocs` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Settings >
      Personalization > Start toggle "Show recently opened items in Start, Jump Lists, and File
      Explorer". It gates the shared recent-activity store, so all three surfaces go quiet together.

      ## Benefits
      - **One switch, three surfaces**: Start Recommended files, jump lists and Explorer Recent
      - **Privacy on shared screens**: document names stop appearing where anyone can read them
      - **Settings-backed**: this is a supported toggle, not an undocumented hack

      ## Drawbacks
      - **Jump lists lose Recent**: right-clicking a taskbar icon no longer lists your documents
      - **Overlaps another tweak**: if you also hide Explorer recent files, reverting only one leaves
        the list still empty, which looks like a broken revert
      - **No history recovery**: existing entries are hidden, not archived

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous state from the snapshot, which on an untouched machine
        means removing the value rather than writing 1

      ## Recommendation
      Apply it if the shell surfacing your documents bothers you or the machine is shared. Skip it if
      you use jump lists to reopen work, which is a genuinely efficient habit.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable Recommended Files in Start, Recent Files in File Explorer, and Jump List items](https://www.elevenforum.com/t/enable-or-disable-recommended-files-in-start-recent-files-in-file-explorer-and-items-in-jump-lists-in-windows-11.1161/)
      - [Recently Opened Files: how to hide or show them](https://www.majorgeeks.com/content/page/recently_opened_files_how_to_hide_or_show_them_in_jump_listsfile_explorerand_start_menu.html)
```

**Sources:**

1. Shipped default user profile and live profile on Windows 11 24H2 build 26100.4061: `Start_TrackDocs` absent from both; `HKLM\...\Explorer\Advanced\Folder` defines `DefaultValue` for `HideFileExt`, `Hidden` and `UseCompactMode` but has no `Start_TrackDocs` entry; value name present in `shell32.dll` and `StartTileData.dll` (tier A, shipped OS state)
2. Enable or Disable Recommended Files in Start, Recent Files in File Explorer, and items in Jump Lists in Windows 11, https://www.elevenforum.com/t/enable-or-disable-recommended-files-in-start-recent-files-in-file-explorer-and-items-in-jump-lists-in-windows-11.1161/ (tier C)
3. Recently Opened Files: How To Hide or Show Them In Jump Lists, File Explorer, and Start Menu, https://www.majorgeeks.com/content/page/recently_opened_files_how_to_hide_or_show_them_in_jump_listsfile_explorerand_start_menu.html (tier C)

### `disable_task_view_button` Task View button

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
`ShowTaskViewButton` (REG_DWORD). Hidden = `0`, Shown (Stock Default) = `1`.

`ShowTaskViewButton` is the Settings > Personalization > Taskbar > "Task view" toggle on Windows 11
and the taskbar context-menu item "Show Task View button" on Windows 10. The default is `1` on both.
It hides the button only; Win+Tab, virtual desktops and the Task View surface itself are untouched.

**Corrections needed:** none. Whether the stock state is a written `1` or value-absent is part of open
question 2, but the value is present on the inspected live profile and no source contradicts the
literal, so it is left as authored pending the clean-image dump.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Task View icon from the taskbar without disabling virtual desktops.**

      ## What it does
      Sets `ShowTaskViewButton` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Settings >
      Personalization > Taskbar > "Task view" toggle. Only the button goes; Win+Tab, virtual desktops
      and timeline behaviour are unchanged.

      ## Benefits
      - **One less icon**: reclaims taskbar space next to Start and search
      - **Nothing disabled**: virtual desktops keep working exactly as before
      - **Fewer misclicks**: the button sits close to Start, which is a common mis-hit

      ## Drawbacks
      - **Keyboard only**: Win+Tab becomes the only route into Task View
      - **Mouse users lose access**: awkward on a tablet or touch device
      - **Per user only**: other accounts keep their button

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1607 and later
      - **Takes effect**: immediately on the Windows 11 taskbar; after restarting Explorer on
        Windows 10
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it if you switch desktops with Win+Ctrl+Arrow or do not use them at all. Leave it if you
      reach Task View with the mouse or use a touch device.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Add or Remove Task View Button on Taskbar in Windows 11](https://www.elevenforum.com/t/add-or-remove-task-view-button-on-taskbar-in-windows-11.1037/)
      - [Add or Remove the Task View Button on the Taskbar](https://www.ninjaone.com/blog/add-or-remove-the-task-view-button-on-the-taskbar-in-windows-11/)
```

**Sources:**

1. Add or Remove Task View Button on Taskbar in Windows 11, https://www.elevenforum.com/t/add-or-remove-task-view-button-on-taskbar-in-windows-11.1037/ (tier C)
2. Add or Remove the Task View Button on the Taskbar in Windows 11, https://www.ninjaone.com/blog/add-or-remove-the-task-view-button-on-the-taskbar-in-windows-11/ (tier C)

### `seconds_in_tray_clock` Seconds in the tray clock

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
`ShowSecondsInSystemClock` (REG_DWORD). Shown = `1`, Hidden (Stock Default) = `0`.

Makes the notification-area clock render seconds. On Windows 11 22H2 Moment 2 and later this is also
exposed as Settings > Personalization > Taskbar > Taskbar behaviours > "Show seconds in system tray
clock". The default is off. Key, name, type and enum are confirmed.

**Corrections needed:** the tweak has no `windows` constraint. On Windows 11 the value is honoured only
from 22H2 build 22621.1344 (Moment 2) onward; on 21H2 and early 22H2 the redesigned taskbar ignores it
entirely. Those builds are below the support floor, so the gate now matters only for correctness on
the Windows 10 IoT Enterprise LTSC 2021 secondary target, where the value has been honoured since 1607
and does work. Add the gate or a note so the applicability is recorded rather than assumed.

**Ready-to-paste info block:**

```yaml
    info: |
      **The taskbar clock ticks in seconds instead of only showing hours and minutes.**

      ## What it does
      Sets `ShowSecondsInSystemClock` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, matching Settings >
      Personalization > Taskbar > Taskbar behaviours > "Show seconds in system tray clock". Windows
      ships it off.

      ## Benefits
      - **Seconds at a glance**: no clock app or watch needed for short timings
      - **Useful for logging**: handy when correlating events to a wall clock
      - **Settings-backed**: a supported toggle, not a hidden hack

      ## Drawbacks
      - **Extra power use**: the clock repaints every second instead of every minute, which
        Microsoft's own Settings text calls out
      - **Wider clock**: the tray takes slightly more horizontal space
      - **Visual noise**: a constantly changing number in the corner of the screen

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; on Windows 11 the value needs build 22621.1344 or
        later, and on Windows 10 it works from 1607
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it on a desktop where you time things or want a precise clock. Skip it on a laptop you
      run on battery, since the per-second repaint is the one real cost.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Turn On or Off Show Seconds in System Tray Clock in Windows 11](https://www.elevenforum.com/t/turn-on-or-off-show-seconds-in-system-tray-clock-in-windows-11.10591/)
      - [How to Enable Seconds for the Taskbar Clock in Windows 11](https://winaero.com/how-to-enable-seconds-for-the-taskbar-clock-in-windows-11/)
```

**Sources:**

1. Turn On or Off Show Seconds in System Tray Clock in Windows 11, https://www.elevenforum.com/t/turn-on-or-off-show-seconds-in-system-tray-clock-in-windows-11.10591/ (tier C)
2. How to Enable Seconds for the Taskbar Clock in Windows 11, https://winaero.com/how-to-enable-seconds-for-the-taskbar-clock-in-windows-11/ (tier C)

### `disable_search_highlights` Search highlights

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`,
`IsDynamicSearchBoxEnabled` (REG_DWORD). Off = `0`, On (Stock Default) = `1`.

`IsDynamicSearchBoxEnabled` is the per-user "Search highlights" toggle (Settings > Privacy and
security > Search permissions > More settings on Windows 11, and the taskbar search context menu on
Windows 10). Setting it to `0` removes the rotating illustrations, seasonal artwork and trending items
from the search box and search flyout. The key path and value name are correct.

**Missing companion, the durable control:**

```
HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search
  EnableDynamicContentInWSB  (REG_DWORD)   0 = search highlights off
```

That is the ADMX policy "Allow search highlights". It survives feature updates and applies
machine-wide. The YAML does not write it.

**Corrections needed:** two items. (1) Add `EnableDynamicContentInWSB` under
`HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` as a companion effect, or document why it is
omitted; without it the per-user preference can be put back by a feature update or a Settings change.
(2) Verify whether the fresh-install state is `IsDynamicSearchBoxEnabled = 1` present or the value
absent. If absent, the "On (Stock Default)" option should be `absent`, not `1`. The fresh-install state
could not be sourced; see open question 2.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the rotating artwork and trending topics from the Windows search box.**

      ## What it does
      Sets `IsDynamicSearchBoxEnabled` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`, the "Search highlights"
      toggle. The illustrations, seasonal artwork and Microsoft-curated trending items disappear from
      the search box and the search flyout.

      ## Benefits
      - **No promoted content**: a system search box stops carrying editorial material
      - **Quieter flyout**: opening search shows your apps and files, not a daily graphic
      - **Less background fetching**: the highlight content is downloaded, and it stops

      ## Drawbacks
      - **Plainer search panel**: some people like the daily artwork
      - **Preference, not policy**: a feature update or a Settings change can put it back unless the
        matching machine policy is also set
      - **Local search unchanged**: this does not affect how well search finds your files

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 21H2 and later
      - **Takes effect**: after restarting Explorer, or the next time the search flyout is opened
      - **Reverting**: restores the previous value from the snapshot
      - The durable machine-wide equivalent is the "Allow search highlights" policy, which writes
        `EnableDynamicContentInWSB` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`.

      ## Recommendation
      Apply it. There is no functional cost, and a search box is a poor place for promoted content.
      Skip only if you actually enjoy the daily illustrations.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Disable Windows search highlights (documents the ADMX policy)](https://4sysops.com/archives/turn-off-windows-search-enhancements/)
      - [Enable or Disable Search Highlights in Windows 11](https://www.elevenforum.com/t/enable-or-disable-search-highlights-in-windows-11.5735/)
```

**Sources:**

1. Disable Windows search highlights (documents the ADMX policy and `EnableDynamicContentInWSB` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`), https://4sysops.com/archives/turn-off-windows-search-enhancements/ (tier C)
2. Enable or Disable Search Highlights in Windows 11, https://www.elevenforum.com/t/enable-or-disable-search-highlights-in-windows-11.5735/ (tier C)
3. Enable or Disable Search Highlights in Windows 10, https://www.tenforums.com/tutorials/194711-enable-disable-search-highlights-windows-10-a.html (tier C)

### `taskbar_search_mode` Taskbar search style

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`, `SearchboxTaskbarMode`
(REG_DWORD).

| Option | Value |
|---|---|
| Hidden | `0` |
| Icon only | `1` |
| Search box (**Stock Default**) | `2` |
| Icon and label | `3` |

The key, name, type and the 0/1/2/3 enum are confirmed, including that `3` is the "search icon and
label" presentation added alongside the Windows 11 22H2 Moment 2 search redesign. The Windows 11
default is `2`.

**Missing companion:**

```
HKCU\Software\Microsoft\Windows\CurrentVersion\Search
  SearchboxTaskbarModeCache  (REG_DWORD)   set to 1 alongside the mode
```

Windows treats a missing cache value as "the user has expressed no preference" and re-migrates
`SearchboxTaskbarMode` back to `2`. Deployment guidance sets the cache to `1` to prevent that.

**Wrong as currently authored:**

```
Hidden = 0 | Icon only = 1 | Search box = 2 | Icon and label = 3
                                    ^-- no option carries the Stock Default label, so revert has no
                                        target; and SearchboxTaskbarModeCache is never written
```

**Corrections needed:** three items. (1) Mark "Search box" as the Stock Default with value `2`.
(2) Write `SearchboxTaskbarModeCache = 1` as a companion so the setting is not re-migrated. (3) Value
`3` requires Windows 11 build 22621.1344 or later, which the primary target satisfies but the LTSC
2021 secondary target does not; on Windows 10 only 0, 1 and 2 are valid, so gate or note it.

**Ready-to-paste info block:**

```yaml
    info: |
      **Shrinks or removes the taskbar search box, the widest element on a stock Windows 11 taskbar.**

      ## What it does
      Writes `SearchboxTaskbarMode` under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`: 0 hides search entirely, 1 leaves a
      small icon, 2 is the full search box Windows ships, and 3 is an icon with a label. It matches
      Settings > Personalization > Taskbar > "Search".

      ## Benefits
      - **Reclaims space**: the search box is the single widest default taskbar element
      - **Four presentations**: pick how much room search gets rather than all or nothing
      - **Keyboard search unaffected**: pressing the Windows key and typing still works when hidden

      ## Drawbacks
      - **Keyboard only when hidden**: with mode 0 there is no mouse entry point to search
      - **Can be re-migrated**: without the companion cache value Windows may quietly restore mode 2,
        which reads back as a failed apply
      - **Mode 3 needs a recent build**: the icon-and-label presentation exists only on Windows 11
        22H2 build 22621.1344 and later

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; Windows 10 supports modes 0, 1 and 2 only
      - **Takes effect**: immediately, the taskbar re-lays out on its own
      - **Reverting**: restores the previous mode from the snapshot, which is the search box (2) on a
        stock machine

      ## Recommendation
      Icon only is the sweet spot for most people: it frees the space without losing the mouse route.
      Choose Hidden only if you always search with the Windows key.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Customizing search on the Windows 11 taskbar (Windows IT Pro blog)](https://techcommunity.microsoft.com/blog/windows-itpro-blog/customizing-search-on-the-windows-11-taskbar/3730314)
      - [Disabling the Windows 11 Taskbar Search Box for All Users](https://awakecoding.com/posts/disabling-the-windows-11-taskbar-search-box-for-all-users/)
      - [How to Change Windows 11 Taskbar Search Button Layout](https://geekrewind.com/how-to-change-windows-11-taskbar-search-button-layout/)
```

**Sources:**

1. Customizing search on the Windows 11 taskbar, Windows IT Pro blog, https://techcommunity.microsoft.com/blog/windows-itpro-blog/customizing-search-on-the-windows-11-taskbar/3730314 (tier B; confirms the four presentation options exist as a supported setting. The page body did not render for an automated fetch, so the value mapping was not taken from it)
2. Disabling the Windows 11 Taskbar Search Box for All Users (documents the migration behaviour and `SearchboxTaskbarModeCache`), https://awakecoding.com/posts/disabling-the-windows-11-taskbar-search-box-for-all-users/ (tier C)
3. How to Change Windows 11 Taskbar Search Button Layout (gives all four values and the default of 2), https://geekrewind.com/how-to-change-windows-11-taskbar-search-button-layout/ (tier C)

### `taskbar_ungroup_labels` Ungroup taskbar buttons

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `TaskbarGlomLevel`
(REG_DWORD).

| Option | Value |
|---|---|
| Always combine, hide labels (**Stock Default**) | `0` |
| Combine only when the taskbar is full | `1` |
| Never combine | `2` |

`TaskbarGlomLevel` is the "Combine taskbar buttons and hide labels" dropdown. The YAML's `2` for never
and `0` for the default are both correct. The Windows 11 21H2 and 22H2 taskbar rewrite ignored the
value; Microsoft restored the behaviour in 23H2 and it continues to work on 24H2 and 25H2.

**Missing companion:**

```
HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced
  MMTaskbarGlomLevel  (REG_DWORD)   same enum, governs taskbars on additional monitors
```

From Windows 11 24H2 the combining behaviour is split in two. Writing only `TaskbarGlomLevel` leaves
secondary monitors combined, which reads as a half-applied tweak on a multi-monitor desk.

**Corrections needed:** two items. (1) Add `MMTaskbarGlomLevel` as a companion effect so multi-monitor
setups behave consistently on 24H2 and later. (2) The info copy still warns about Windows 11 21H2 and
22H2, which are below the support floor; that caveat is dead text and should be cut or replaced with
the LTSC 2021 note (where the value has always worked).

**Ready-to-paste info block:**

```yaml
    info: |
      **Gives every window its own labelled taskbar button instead of stacking them under one icon.**

      ## What it does
      Sets `TaskbarGlomLevel` to 2 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the "Combine taskbar
      buttons and hide labels" dropdown. Value 0 always combines, 1 combines only when the taskbar
      fills up, and 2 never combines.

      ## Benefits
      - **One click per window**: no hovering a stacked icon and picking from a thumbnail list
      - **Readable labels**: you can tell three browser windows apart without previewing them
      - **Graceful option**: value 1 keeps labels until the taskbar runs out of room

      ## Drawbacks
      - **Taskbar fills fast**: with many windows open the buttons shrink or overflow
      - **Poor on narrow screens**: labelled buttons need horizontal space
      - **Second monitor needs its own value**: from 24H2 additional displays are governed by a
        separate setting, so writing only the main one leaves them combined

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (restored in 23H2 after being ignored on 21H2 and
        22H2); also every Windows 10 build
      - **Takes effect**: immediately, the taskbar re-lays out on its own
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it on a wide monitor where you keep fewer than a dozen windows open. On a laptop screen
      choose "combine when full" instead, or leave the default.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [How to enable Taskbar labels and never combine on Windows 11](https://pureinfotech.com/show-taskbar-labels-never-combine-windows-11/)
      - [Always or Never Combine Taskbar buttons and Hide Labels in Windows 11](https://www.elevenforum.com/t/always-or-never-combine-taskbar-buttons-and-hide-labels-in-windows-11.15135/)
      - [How to Enable Never Combine Taskbar Buttons in Windows 11 25H2](https://www.tech2geek.net/how-to-enable-never-combine-taskbar-buttons-in-windows-11-25h2/)
```

**Sources:**

1. How to enable Taskbar labels and never combine on Windows 11 (gives the 0/1/2 enum and the 23H2 requirement), https://pureinfotech.com/show-taskbar-labels-never-combine-windows-11/ (tier C)
2. Always or Never Combine Taskbar buttons and Hide Labels in Windows 11, https://www.elevenforum.com/t/always-or-never-combine-taskbar-buttons-and-hide-labels-in-windows-11.15135/ (tier C)
3. How to Enable Never Combine Taskbar Buttons in Windows 11 25H2 (confirms the value still works on 25H2 and that 24H2 split the setting in two), https://www.tech2geek.net/how-to-enable-never-combine-taskbar-buttons-in-windows-11-25h2/ (tier C)

### `taskbar_hover_time` Taskbar thumbnail delay

**Verdict:** DISPUTED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
`ExtendedUIHoverTime` (REG_DWORD), in milliseconds. Instant = `1`, stock = **value absent** with the
shell falling back to an internal 400 ms.

`ExtendedUIHoverTime` sets how long the pointer must rest on a taskbar button before the thumbnail
preview appears. It is distinct from `MouseHoverTime` under `Control Panel\Mouse`, which happens to
share the same 400 ms default.

**Why DISPUTED, and it is a genuine evidence conflict rather than Microsoft silence.** On one side the
value name has not been removed from shipping code: a sweep of 5,418 binaries under `System32` on
Windows 11 24H2 build 26100.4061 finds the UTF-16 string `ExtendedUIHoverTime` in exactly one module,
`Taskbar.dll`, which is the live Windows 11 taskbar implementation. On the other side, in that binary
the string sits inside the ported **legacy taskband string block**, adjacent to `MSTaskSwWClass`,
`TaskbandExtendedUI`, `ThumbnailLivePreviewHoverTime`, `DisablePreviewWindow` and `MMTaskbarGlomLevel`,
and `MMTaskbarGlomLevel` is a Windows 10 value that is inert on the Windows 11 primary taskbar. String
presence therefore proves the code was carried forward, not that the path is still reached. Against
it, the long-running elevenforum tutorial for this exact value, maintained and last updated May 2026,
now carries the standing note "The setting in this tutorial no longer works starting with at least
Windows 11 version 24H2", and a Microsoft Q&A thread titled "ExtendedUIHoverTime is NOT working
anymore" reports the same. The same forum's discussion thread confirms it "definitely works" on 23H2.
No source, and no measurement available here, confirms a visible hover-delay change on 26100 or 26200.

**Wrong Stock Default as currently authored:**

```
"Default (400 ms)" = ExtendedUIHoverTime 400     <-- stock is value-absent, confirmed on 26100.4061
```

Direct registry inspection on 26100.4061 shows no `ExtendedUIHoverTime` under `Explorer\Advanced`.
Writing a literal 400 on revert leaves a non-stock artefact behind.

**Corrections needed:** two items. (1) The "Default (400 ms)" option must remove the value rather than
write 400. This is the one unambiguous YAML defect. (2) Given the 26100 floor, either gate the tweak
below build 26100 (it remains correct for Windows 10 IoT Enterprise LTSC 2021 and Windows 11 up to
23H2) or carry an explicit warning that it is widely reported non-functional on 24H2 and later. Do not
present it as a working tweak on 26100+ without a fresh visual test. See open question 3.

**Ready-to-paste info block:**

```yaml
    info: |
      **Taskbar thumbnail previews appear the moment you hover instead of after a pause.**

      ## What it does
      Writes `ExtendedUIHoverTime` under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` as a millisecond delay.
      Value 1 means effectively instant. With the value absent the shell uses its built-in 400 ms.
      This is not the same as `MouseHoverTime` under `Control Panel\Mouse`, which shares the same
      400 ms default but governs general hover.

      ## Benefits
      - **Faster window switching**: removes a real 400 ms tax if you switch by hovering the taskbar
      - **Tunable**: any millisecond value works, not just the two presets
      - **Per user**: no admin rights and no policy involved

      ## Drawbacks
      - **Reported broken since 24H2**: on Windows 11 24H2 and 25H2 the value is widely reported not
        to change anything, so the write can succeed with no visible effect
      - **Twitchy at very low values**: previews fire whenever the cursor crosses the taskbar
      - **Undocumented**: Microsoft has never published this value name

      ## Good to know
      - **Applies to**: Windows 10 and Windows 11 up to 23H2 reliably; on Windows 11 24H2 (26100)
        and 25H2 the value name still exists in `Taskbar.dll` but only inside the legacy taskband
        code, and the effect is disputed
      - **Takes effect**: after sign-out or restarting Explorer
      - **Reverting**: restores the previous state from the snapshot, which on an untouched machine
        means removing the value rather than writing 400

      ## Recommendation
      Worth trying only if you are on Windows 10 or a Windows 11 build below 24H2, where it is well
      attested. On 24H2 and newer expect it to do nothing; do not spend time troubleshooting it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Change Hover Time to Show Taskbar Thumbnail Previews in Windows 11](https://www.elevenforum.com/t/change-hover-time-to-show-taskbar-thumbnail-previews-in-windows-11.6366/)
      - [ExtendedUIHoverTime is NOT working anymore (Microsoft Q&A)](https://learn.microsoft.com/en-us/answers/questions/2263244/extendeduihovertime-is-not-working-anymore)
      - [Change Taskbar Thumbnail Hover Delay in Windows 10](https://winaero.com/taskbar-thumbnail-hover-delay-windows-10/)
```

**Sources:**

1. Direct binary inspection, Windows 11 24H2 build 26100.4061: `ExtendedUIHoverTime` appears as a UTF-16 string in `C:\Windows\System32\Taskbar.dll` only, out of 5,418 `System32` binaries scanned, in a string block also containing `MSTaskSwWClass`, `TaskbandExtendedUI`, `ThumbnailLivePreviewHoverTime` and `MMTaskbarGlomLevel` (tier A, primary measurement; shows the name survives in the shipping taskbar, in legacy company)
2. Direct registry inspection, Windows 11 24H2 build 26100.4061: `ExtendedUIHoverTime` absent under `Explorer\Advanced`, confirming the stock default is value-absent, not 400 (tier A, primary measurement)
3. Change Hover Time to Show Taskbar Thumbnail Previews in Windows 11, elevenforum tutorial first published May 2022 and updated May 2026, carrying the note "The setting in this tutorial no longer works starting with at least Windows 11 version 24H2", https://www.elevenforum.com/t/change-hover-time-to-show-taskbar-thumbnail-previews-in-windows-11.6366/ (tier C; maintained long-lived reference and the strongest evidence for non-functionality)
4. Registry key: ExtendedUIHoverTime to modify hover time for showing taskbar thumbnail previews, elevenforum discussion, https://www.elevenforum.com/t/registry-key-extendeduihovertime-to-modify-hover-time-for-showing-taskbar-thumbnail-previews.24693/ (tier C; confirms it works on 23H2 22631)
5. ExtendedUIHoverTime is NOT working anymore, Microsoft Q&A, https://learn.microsoft.com/en-us/answers/questions/2263244/extendeduihovertime-is-not-working-anymore (tier D as a forum thread, hosted on Microsoft Q&A, corroborating source 3)
6. Change Taskbar Thumbnail Hover Delay in Windows 10, Winaero (documents the 400 ms fallback and the sign-out requirement), https://winaero.com/taskbar-thumbnail-hover-delay-windows-10/ (tier C)

### `taskbar_alignment_left` Left-align the taskbar

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `TaskbarAl`
(REG_DWORD). Left = `0`, Center (Stock Default) = `1`. Gated `windows: { products: [11] }`.

`TaskbarAl` (the last character is a lower-case L, not an I) is the Settings > Personalization >
Taskbar > Taskbar behaviours > "Taskbar alignment" dropdown. `0` is left, `1` is centre, and centre is
the Windows 11 default. The Windows 11 product gate is correct: the Windows 10 taskbar has no such
setting.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Moves Start and the pinned icons back to the left corner of the taskbar.**

      ## What it does
      Sets `TaskbarAl` to 0 under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
      the Settings > Personalization > Taskbar > "Taskbar alignment" dropdown. Value 1 is the centred
      Windows 11 default.

      ## Benefits
      - **Fixed target**: Start stays in the corner instead of shifting as windows open and close
      - **Muscle memory**: matches Windows 10 and every earlier release
      - **Fitts's law**: a screen corner is the easiest target to hit with a mouse

      ## Drawbacks
      - **Longer travel on wide screens**: centred icons are physically closer on an ultrawide
      - **Fights shell replacements**: ExplorerPatcher and similar tools manage alignment themselves
      - **Windows 11 only**: the value does nothing on Windows 10

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (and all Windows 11 builds)
      - **Takes effect**: immediately, the taskbar re-lays out on its own
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it if the icons moving under your cursor annoys you, which is the usual complaint. On a
      very wide monitor, leave it centred; the travel distance is a real cost.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Change Taskbar Alignment in Windows 11](https://www.elevenforum.com/t/change-taskbar-alignment-in-windows-11.12/)
      - [How to left align the Taskbar on Windows 11](https://pureinfotech.com/align-taskbar-icons-left-windows-11/)
```

**Sources:**

1. Change Taskbar Alignment in Windows 11, https://www.elevenforum.com/t/change-taskbar-alignment-in-windows-11.12/ (tier C)
2. How to left align Taskbar on Windows 11, https://pureinfotech.com/align-taskbar-icons-left-windows-11/ (tier C)

### `disable_snap_flyout` Snap layouts flyout

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
`EnableSnapAssistFlyout` (REG_DWORD). Off = `0`, On (Stock Default) = **`absent`**. Gated
`windows: { products: [11] }`.

`EnableSnapAssistFlyout` is the Settings > System > Multitasking > Snap windows > "Show snap layouts
when I hover over a window's maximise button" checkbox. Setting `0` suppresses the layout grid on
hover. Snapping by Win+Arrow, by drag-to-edge and by drag-to-top all continue to work.

**Wrong Stock Default as currently authored:**

```
"On (Stock Default)" = EnableSnapAssistFlyout 1     <-- stock is value-absent on 26100
```

On Windows 11 24H2 build 26100.4061 the value does not exist on a stock install: the shipped default
user profile carries only `Start_SearchFiles` under
`Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `EnableSnapAssistFlyout` has no
`DefaultValue` declaration under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder`
(unlike `HideFileExt`, `Hidden` and `UseCompactMode`), and the value is absent from a
fourteen-month-old interactively used profile on the same machine. The Multitasking settings handler
materialises it only when the user moves the toggle.

**Scope correction.** `twinui.dll` and `SettingsHandlers_nt.dll` on 26100.4061 carry three independent
value names: `EnableSnapAssistFlyout` (the hover flyout this tweak writes), `EnableSnapBar` (the
drag-to-top layout bar) and `EnableSnapAssist` (the post-snap suggestion picker, now covered by the new
`disable_snap_assist` tweak). A user who reads the name as "turn off snap layouts" will still get the
drag-to-top bar, and Win+Z still opens the layout grid.

**Corrections needed:** two items. (1) The "On (Stock Default)" option should write `absent`, not `1`.
(2) Rename the tweak for the one toggle it actually controls, or add `EnableSnapBar` as a second
option; the current name overclaims.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the snap layout grid from popping out when your cursor passes the maximise button.**

      ## What it does
      Sets `EnableSnapAssistFlyout` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Settings > System >
      Multitasking checkbox "Show snap layouts when I hover over a window's maximise button". Only
      that hover flyout is suppressed.

      ## Benefits
      - **No accidental popups**: the grid stops firing on incidental cursor movement near the
        window controls
      - **Snapping still works**: Win+Arrow, drag-to-edge and drag-to-top are unaffected
      - **Targeted**: it changes one checkbox, not the whole snap subsystem

      ## Drawbacks
      - **Loses the mouse route to layouts**: picking a layout by hovering is gone
      - **Only one of three switches**: the drag-to-top layout bar and the post-snap suggestion
        picker are separate settings and stay on
      - **Win+Z unchanged**: the keyboard shortcut still opens the layout grid

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (and all Windows 11 builds); Windows 10 has no
        snap-layouts flyout
      - **Takes effect**: after restarting Explorer or signing out
      - **Reverting**: restores the previous state from the snapshot, which on an untouched machine
        means removing the value rather than writing 1

      ## Recommendation
      Apply it if the flyout keeps appearing when you reach for the close button. Skip it if you
      actually pick layouts from the grid with the mouse.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [How to Disable Snap Layouts Flyout on Maximize Button in Windows 11](https://www.askvg.com/how-to-disable-snap-layouts-flyout-on-maximize-button-in-windows-11/)
      - [Enable or disable Snap Layouts in Windows 11](https://www.thewindowsclub.com/enable-snap-layouts-on-windows-11)
```

**Sources:**

1. Shipped default user profile and live profile on Windows 11 24H2 build 26100.4061: `EnableSnapAssistFlyout` absent from both, no `DefaultValue` declaration under `HKLM\...\Explorer\Advanced\Folder`; value name present in `twinui.dll` and `SettingsHandlers_nt.dll` alongside the separate `EnableSnapBar` and `EnableSnapAssist` names (tier A, shipped OS state)
2. How to Disable Snap Layouts Flyout on Maximize Button in Windows 11, https://www.askvg.com/how-to-disable-snap-layouts-flyout-on-maximize-button-in-windows-11/ (tier C)
3. Enable or disable Snap Layouts in Windows 11 when you hover over Maximize button, https://www.thewindowsclub.com/enable-snap-layouts-on-windows-11 (tier C)

### `explorer_compact_view` Explorer compact view

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `UseCompactMode`
(REG_DWORD). On = `1`, Off (Stock Default) = `0`. Gated `windows: { products: [11] }`.

`UseCompactMode` is the Folder Options > View > "Use compact mode" checkbox Windows 11 added when it
increased row padding in File Explorer for touch. Setting `1` restores the tighter Windows 10 row
spacing. The default of `0` is declared by Windows itself under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder`, so the literal is correct.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Tightens File Explorer row spacing so more files fit on screen.**

      ## What it does
      Sets `UseCompactMode` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Folder Options > View
      checkbox "Use compact mode". Windows 11 added extra row padding for touch; this restores the
      tighter Windows 10 spacing.

      ## Benefits
      - **More rows per screen**: noticeably fewer scrolls in large folders
      - **Familiar density**: matches Windows 10 and File Explorer as it looked for a decade
      - **Officially supported**: a Folder Options checkbox, not a hack

      ## Drawbacks
      - **Smaller touch targets**: harder to hit rows with a finger or pen
      - **Shrinking payoff**: on recent 24H2 and 25H2 servicing levels some users report compact
        mode reducing padding less than it used to
      - **Windows 11 only**: Windows 10 already uses the tighter spacing and ignores this value

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (and all Windows 11 builds)
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it on a keyboard and mouse machine, where the extra padding buys nothing. Leave it off on
      a tablet or a touchscreen laptop.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Turn On or Off Compact View in File Explorer in Windows 11](https://www.elevenforum.com/t/turn-on-or-off-compact-view-in-file-explorer-in-windows-11.896/)
      - [Windows 11 Enable Compact View in File Explorer](https://winaero.com/windows-11-enable-compact-view-in-file-explorer/)
```

**Sources:**

1. Turn On or Off Compact View in File Explorer in Windows 11, https://www.elevenforum.com/t/turn-on-or-off-compact-view-in-file-explorer-in-windows-11.896/ (tier C)
2. Windows 11 Enable Compact View in File Explorer, https://winaero.com/windows-11-enable-compact-view-in-file-explorer/ (tier C)
3. Shipped OS state on 26100.4061: `HKLM\...\Explorer\Advanced\Folder\UseCompactMode` declares `DefaultValue = 0` (tier A, product artifact)

### `disable_start_recommendations` Start menu recommendations

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
`Start_IrisRecommendations` (REG_DWORD). Off = `0`, On (Stock Default) = `1`. Gated
`windows: { products: [11] }`.

`Start_IrisRecommendations` is the Settings > Personalization > Start toggle "Show recommendations for
tips, shortcuts, new apps, and more". Setting `0` stops Microsoft-curated tips, shortcut suggestions
and new-app promotions appearing in the Start Recommended area.

**Corrected scope claim:** it does **not** remove the Recommended section. That is a different control,
the `HideRecommendedSection` policy DWORD under `SOFTWARE\Policies\Microsoft\Windows\Explorer`, now
shipped separately as `disable_start_recommended_section`.

**Wrong copy as currently authored:** the tweak is named "Start menu recommendations", described as
"Hide the Recommended section (recent files and suggested apps)", and its info text discusses a policy
that "has historically only collapsed the section" and an Education-environment flag. All of that
describes `HideRecommendedSection`, not `Start_IrisRecommendations`. The text has been attached to the
wrong value.

Not a duplicate of anything in `debloat`: `Start_IrisRecommendations` and
`SubscribedContent-338388Enabled` are different surfaces. The latter lives under
`HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager` and backs "Show suggestions
occasionally in Start", which is the Content Delivery Manager promoted-app channel. Neither subsumes
the other, though a user chasing a quiet Start menu will want both.

**Corrections needed:** narrow the tweak name and description to the promoted rows, remove the claim
that it hides the Recommended section, and rewrite the "Good to know" paragraph, which currently
describes a policy this tweak does not write.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Microsoft's promoted tips, shortcuts and app suggestions appearing in the Start menu.**

      ## What it does
      Sets `Start_IrisRecommendations` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Settings >
      Personalization > Start toggle "Show recommendations for tips, shortcuts, new apps, and more".
      It clears the promoted rows inside the Recommended area; the area itself stays.

      ## Benefits
      - **No promoted content**: Microsoft-curated app and feature pitches stop appearing
      - **Settings-backed**: a supported toggle rather than a policy or a hack
      - **No admin rights**: a plain per-user preference

      ## Drawbacks
      - **Section remains**: the Recommended area stays on screen and may look sparse or empty
      - **Not the only channel**: Start also carries Content Delivery Manager suggestions, which are
        a separate setting
      - **Loses genuine tips**: the occasional useful feature hint goes with the promotions

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (meaningful from the 22H2 Start redesign)
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the previous value from the snapshot
      - To remove the Recommended section entirely rather than just its promoted rows, use the
        separate policy-based tweak for that.

      ## Recommendation
      Apply it. There is no functional loss worth naming, and Start is a poor place for promotions.
      Pair it with the section-removal tweak if you want the space back too.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable Recommended Tips, Shortcuts, New Apps on the Start Menu in Windows 11](https://www.elevenforum.com/t/enable-or-disable-recommended-tips-shortcuts-new-apps-and-more-on-start-menu-in-windows-11.14346/)
      - [How to Disable Recommended in Start Menu on Windows 11](https://winaero.com/disable-recommended-start/)
```

**Sources:**

1. Enable or Disable Recommended Tips, Shortcuts, New Apps, and more on Start Menu in Windows 11, https://www.elevenforum.com/t/enable-or-disable-recommended-tips-shortcuts-new-apps-and-more-on-start-menu-in-windows-11.14346/ (tier C)
2. How to Disable Recommended in Start Menu on Windows 11 (documents `HideRecommendedSection` and `IsEducationEnvironment` as the separate section-removal route), https://winaero.com/disable-recommended-start/ (tier C)
3. How to remove Recommended section from Start menu in Windows 11, https://www.thewindowsclub.com/remove-recommended-section-from-start-menu-in-windows (tier C)

### `enable_end_task_taskbar` Taskbar End Task

**Verdict:** VERIFIED

**Mechanism:**
`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings`,
`TaskbarEndTask` (REG_DWORD). On = `1`, Off (Stock Default) = `0`. Gated `windows: { build: ">=22631" }`.

Adds an "End task" item to the right-click menu of taskbar buttons, terminating the app's process tree
directly. It is the registry backing for Settings > System > For developers > "End Task". The key path
including the `TaskbarDeveloperSettings` subkey, the value name, the type and the default of `0` are
all confirmed. The `build: ">=22631"` gate is correct and still useful, since it excludes the LTSC 2021
secondary target.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Adds "End task" to the taskbar right-click menu so you can kill a hung app in one click.**

      ## What it does
      Sets `TaskbarEndTask` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings`,
      the registry backing for Settings > System > For developers > "End Task". Choosing the item
      terminates that app's process tree.

      ## Benefits
      - **No Task Manager hunt**: kill the app from the button you are already looking at
      - **Kills the tree**: child processes go too, which Task Manager's End task also does
      - **Officially supported**: exposed as a Settings toggle, not an undocumented hack

      ## Drawbacks
      - **Unsaved work is lost**: this is a hard terminate with no save prompt
      - **Easy to reach for**: on a shared machine someone may use it instead of closing apps
        normally, which can corrupt app state
      - **One more menu item**: the right-click menu gets slightly longer

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (from build 22631)
      - **Takes effect**: after restarting Explorer or signing out
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it if you develop software or regularly deal with apps that stop responding. Leave it off
      on a family machine where someone will use it as a normal way to close programs.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable End Task in Taskbar by Right Click in Windows 11](https://www.elevenforum.com/t/enable-or-disable-end-task-in-taskbar-by-right-click-in-windows-11.14325/)
      - [Enable or Disable End Task Option In Taskbar On Windows 11](https://cloudinfra.net/enable-disable-end-task-option-in-taskbar-on-windows-11/)
```

**Sources:**

1. Enable or Disable End Task in Taskbar by Right Click in Windows 11, https://www.elevenforum.com/t/enable-or-disable-end-task-in-taskbar-by-right-click-in-windows-11.14325/ (tier C)
2. Enable/Disable End Task Option In Taskbar On Windows 11, https://cloudinfra.net/enable-disable-end-task-option-in-taskbar-on-windows-11/ (tier C)

### `remove_gallery_nav_pane` Gallery in Explorer

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Classes\CLSID\{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}`,
`System.IsPinnedToNameSpaceTree` (REG_DWORD). Hidden = `0`, Shown (Stock Default) = `absent`. Gated
`windows: { build: ">=22631" }`.

`{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}` is the Gallery shell namespace extension added in Windows 11
23H2. `System.IsPinnedToNameSpaceTree` under the per-user CLSID key overrides the machine-wide
definition and unpins the node from the navigation tree. Deleting the per-user value restores the
machine default, which is pinned.

The `absent` Stock Default is correct here, unlike the OneDrive case: the per-user CLSID key does not
exist on a clean install, and the pinned state comes from
`HKLM\...\Explorer\Advanced\NavPane\ShowGallery` (default 1) plus the machine-wide CLSID registration.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Gallery entry from the File Explorer sidebar.**

      ## What it does
      Writes `System.IsPinnedToNameSpaceTree = 0` under
      `HKCU\Software\Classes\CLSID\{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}`, the Gallery namespace
      extension. The per-user value shadows the machine-wide registration and unpins the node. Your
      photos are not touched.

      ## Benefits
      - **Shorter sidebar**: one less node above This PC
      - **Nothing deleted**: photos, folders and the Photos app are untouched
      - **Cleanly reversible**: deleting the per-user value restores the machine default

      ## Drawbacks
      - **Loses the photo view**: no quick chronological view of recent camera imports
      - **Per user only**: other accounts keep their Gallery node
      - **Windows 11 23H2 and later only**: Gallery does not exist on earlier builds

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (from build 22631)
      - **Takes effect**: after restarting Explorer
      - **Reverting**: removes the per-user value, restoring the machine default of pinned

      ## Recommendation
      Apply it if you never browse photos through Explorer. Leave it if Gallery is how you find
      recent phone or camera imports.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Add or Remove Gallery in File Explorer Navigation Pane in Windows 11](https://www.elevenforum.com/t/add-or-remove-gallery-in-file-explorer-navigation-pane-in-windows-11.14178/)
      - [How to Remove Gallery from File Explorer](https://winaero.com/remove-gallery-from-file-explorer/)
```

**Sources:**

1. Add or Remove Gallery in File Explorer Navigation Pane in Windows 11, https://www.elevenforum.com/t/add-or-remove-gallery-in-file-explorer-navigation-pane-in-windows-11.14178/ (tier C)
2. How to Remove Gallery from File Explorer, https://winaero.com/remove-gallery-from-file-explorer/ (tier C)
3. Hide Gallery from Explorer on Windows 11, https://endurtech.com/hide-gallery-from-explorer-on-windows-11/ (tier C)

### `remove_home_nav_pane` Home in Explorer

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Classes\CLSID\{f874310e-b6b7-47dc-bc84-b9e6b38f5903}`,
`System.IsPinnedToNameSpaceTree` (REG_DWORD). Hidden = `0`, Shown (Stock Default) = `absent`. Gated
`windows: { products: [11] }`.

`{f874310e-b6b7-47dc-bc84-b9e6b38f5903}` is the Home namespace extension in Windows 11 File Explorer.
Same mechanism as Gallery: the per-user value overrides the machine default and unpins the node. An
alternative route exists (deleting or renaming
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace\{f874310e-...}`); the YAML
picks the per-user reversible route, which is the right call.

On Windows 11 21H2 the node is still called Quick access and uses a different CLSID
(`{679f85cb-0220-4080-b29b-5540cc05aab6}`), so on 21H2 this tweak is a no-op. That build is below the
support floor, so the `products: [11]` gate is slightly loose but not harmful.

**Corrections needed:** none. Worth surfacing the dependency on `open_explorer_to_this_pc` as a hard
ordering constraint rather than advice.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Home entry from the File Explorer sidebar.**

      ## What it does
      Writes `System.IsPinnedToNameSpaceTree = 0` under
      `HKCU\Software\Classes\CLSID\{f874310e-b6b7-47dc-bc84-b9e6b38f5903}`, the Windows 11 Home
      namespace extension. The per-user value shadows the machine-wide registration and unpins the
      node from the navigation tree.

      ## Benefits
      - **Drive-focused sidebar**: the pane leads with This PC and your libraries
      - **No recents feed**: the pinned and recent aggregation is out of the way
      - **Cleanly reversible**: deleting the per-user value restores the machine default

      ## Drawbacks
      - **Pinned folders become hard to reach**: quick-access pins live inside Home
      - **Ordering trap**: Explorer's default landing page is Home, so removing the node without also
        setting Explorer to open on This PC leaves it configured to open something that is not there
      - **Per user only**: other accounts keep their Home node

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (22H2 and later; on 21H2 the node used a different
        identifier and this does nothing)
      - **Takes effect**: after restarting Explorer
      - **Reverting**: removes the per-user value, restoring the machine default of pinned

      ## Recommendation
      Apply it together with setting File Explorer to open on This PC. Skip it if you rely on pinned
      folders, since Home is where they live.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Add or Remove Home in Navigation Pane of File Explorer in Windows 11](https://www.elevenforum.com/t/add-or-remove-home-in-navigation-pane-of-file-explorer-in-windows-11.2449/)
      - [How to remove the File Explorer Home page on Windows 11](https://pureinfotech.com/remove-home-file-explorer-windows-11/)
```

**Sources:**

1. Add or Remove Home in Navigation Pane of File Explorer in Windows 11, https://www.elevenforum.com/t/add-or-remove-home-in-navigation-pane-of-file-explorer-in-windows-11.2449/ (tier C)
2. Remove Home in the Navigation Pane of File Explorer in Windows 11, https://www.ninjaone.com/blog/remove-home-in-the-navigation-pane-of-file-explorer/ (tier C)
3. How to remove File Explorer Home page on Windows 11, https://pureinfotech.com/remove-home-file-explorer-windows-11/ (tier C)

### `remove_onedrive_nav_pane` OneDrive in Explorer

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (corrected):** `HKCU\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}`,
`System.IsPinnedToNameSpaceTree` (REG_DWORD). Hidden = `0`, Shown (Stock Default) = **`1`**.

`{018D5C66-4533-4307-9B53-224DE2ED1FE6}` is the OneDrive Personal shell namespace extension. Unlike
Gallery and Home, this CLSID key is **created by OneDrive setup in HKCU** with
`System.IsPinnedToNameSpaceTree = 1` already present. Setting `0` unpins the node without touching the
sync client or the local folder.

**Wrong Stock Default as currently authored:**

```
"Shown (Stock Default)" = absent     <-- deletes a value OneDrive setup created
```

Reverting to `absent` removes a value that exists on any machine with OneDrive installed, leaving the
pin state undefined rather than restored.

**Corrections needed:** two items. (1) The Stock Default should be `1`, not `absent`. (2) Consider
gating the tweak on OneDrive being installed, so it does not present on machines where the CLSID does
not exist and the write would create a stray key with no effect. Note the CLSID covers OneDrive
Personal only; OneDrive for Business sync roots use per-tenant CLSIDs and are unaffected.

**Ready-to-paste info block:**

```yaml
    info: |
      **Hides the OneDrive entry from the File Explorer sidebar without uninstalling OneDrive.**

      ## What it does
      Sets `System.IsPinnedToNameSpaceTree` to 0 under
      `HKCU\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}`, the OneDrive Personal
      namespace extension that OneDrive setup creates with a value of 1. The sync client, the local
      folder and your files are untouched.

      ## Benefits
      - **Cleaner sidebar**: removes a node you do not use without uninstalling anything
      - **Sync unaffected**: files keep syncing exactly as before
      - **Reversible**: restoring the value of 1 puts the node back

      ## Drawbacks
      - **Confusing while syncing**: files keep uploading from a location you can no longer see in
        the pane
      - **Personal only**: OneDrive for Business sync roots use different identifiers and stay
      - **Inert without OneDrive**: on a machine where OneDrive is not installed the write creates a
        stray key and does nothing

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10, wherever OneDrive Personal is
        installed and signed in
      - **Takes effect**: after restarting Explorer
      - **Reverting**: restores the value to 1, which is what OneDrive setup writes

      ## Recommendation
      Apply it if OneDrive is installed but you store nothing there. If OneDrive is your primary
      document location, leave the node alone; hiding it makes sync state invisible.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [How to Remove the OneDrive Icon from File Explorer in Windows 11](https://winaero.com/how-to-remove-onedrive-icon-from-file-explorer-in-windows-11/)
      - [Add or Remove OneDrive in Navigation Pane of File Explorer in Windows 11](https://www.elevenforum.com/t/add-or-remove-onedrive-in-navigation-pane-of-file-explorer-in-windows-11.2478/)
```

**Sources:**

1. How to Remove OneDrive Icon from File Explorer in Windows 11 (states the value defaults to 1 when OneDrive is installed), https://winaero.com/how-to-remove-onedrive-icon-from-file-explorer-in-windows-11/ (tier C)
2. Add or Remove OneDrive in Navigation Pane of File Explorer in Windows 11, https://www.elevenforum.com/t/add-or-remove-onedrive-in-navigation-pane-of-file-explorer-in-windows-11.2478/ (tier C)

### `disable_toast_notifications` Toast notifications

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications`, `ToastEnabled`
(REG_DWORD). Off = `0`, On (Stock Default) = `1`. `requires_reboot: true`, `risk_level: medium`.

`ToastEnabled` is the master per-user notification switch, the registry value behind Settings > System
> Notifications > "Notifications". Setting `0` suppresses every toast and banner from every source,
including WNS-delivered, local and system notifications. A sign-out and sign-in is needed for the
change to take hold, so `requires_reboot: true` is a defensible if slightly heavy representation. The
Settings UI toggle and this value can drift out of sync if written while the shell is running, which
is part of why the sign-out matters.

**Corrections needed:** none. This is the tweak in this file most likely to cause real harm by hiding a
security or data-loss warning, and `medium` risk is the right call; it should not be lowered.

**Ready-to-paste info block:**

```yaml
    info: |
      **Silences every pop-up notification on the machine, from every app and from Windows itself.**

      ## What it does
      Sets `ToastEnabled` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications`, the master switch behind
      Settings > System > Notifications. It suppresses all toasts and banners, including cloud
      delivered, local and system notifications.

      ## Benefits
      - **Total silence**: nothing pops up during a presentation, recording or game
      - **One switch**: no need to disable notifications app by app
      - **Fully reversible**: a single per-user value

      ## Drawbacks
      - **Security alerts vanish**: Windows Security warnings and BitLocker prompts are suppressed
      - **Data-loss warnings vanish**: backup failures and low-battery warnings go with them
      - **Everything else too**: calendar reminders, Teams and messaging apps, delivery alerts
      - **Focus is usually better**: Do Not Disturb suppresses banners while still queuing
        notifications in the Notification Center, which is what most people actually want

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: after sign-out and sign-in
      - **Reverting**: restores the previous value from the snapshot
      - Writing this while the shell is running can leave the Settings toggle showing a stale state
        until the next sign-in.

      ## Recommendation
      Use it only for a specific situation, such as a demo, a recording session or a kiosk. For
      day-to-day quiet, turn on Do Not Disturb instead; it keeps the notifications you missed.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [How to disable notifications in Windows 11](https://winaero.com/how-to-disable-notifications-in-windows-11/)
      - [Turn On or Off Notifications in Windows 11](https://www.elevenforum.com/t/turn-on-or-off-notifications-in-windows-11.821/)
```

**Sources:**

1. How to disable notifications in Windows 11 (documents `ToastEnabled` and the sign-out requirement), https://winaero.com/how-to-disable-notifications-in-windows-11/ (tier C)
2. Turn On or Off Notifications in Windows 11, https://www.elevenforum.com/t/turn-on-or-off-notifications-in-windows-11.821/ (tier C)

### `verbose_logon_messages` Verbose logon messages

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`, `VerboseStatus`
(REG_DWORD). On = `1`, Off (Stock Default) = `absent`. `elevation: admin`, `requires_reboot: true`.

`VerboseStatus` is the registry backing for the ADMX policy "Display highly detailed status messages"
(Computer Configuration > Administrative Templates > System). When enabled, Windows replaces the
generic "Welcome" and "Signing out" screens with step-by-step text such as "Applying computer settings"
and "Loading your profile". Hive, key path, value name and type match the policy definition exactly,
and `absent` is the correct unset state.

Documented interaction: this policy is ignored if the "Remove Boot / Shutdown / Logon / Logoff status
messages" policy (`DisableStatusMessages`, same key) is enabled.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Replaces the sign-in spinner with step-by-step text so you can see what Windows is doing.**

      ## What it does
      Sets `VerboseStatus` to 1 under
      `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`, the registry backing for the
      Group Policy "Display highly detailed status messages". Sign-in, sign-out and shutdown screens
      show phases such as "Applying computer settings" and "Loading your profile".

      ## Benefits
      - **Diagnoses slow boots**: you can see which phase is stalling instead of watching a spinner
      - **Zero performance cost**: it changes what is drawn, nothing else
      - **Policy-backed**: a documented Microsoft setting, not a hack

      ## Drawbacks
      - **Looks alarming**: a non-technical user may read normal phase text as an error
      - **Leaks names**: policy names and internal hostnames can appear on a shared or projected screen
      - **Can be overridden**: it is ignored if the "Remove boot / shutdown / logon / logoff status
        messages" policy is enabled

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also every Windows version since Windows 2000
      - **Takes effect**: after reboot
      - **Reverting**: removes the value, which is the unconfigured state
      - Needs administrator rights, since the value lives in HKLM.

      ## Recommendation
      Apply it while you are diagnosing a slow or hanging sign-in, and on any machine you maintain.
      Turn it back off on a shared or public-facing machine.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Display highly detailed status messages (ADMX policy reference)](https://admx.help/?Category=Windows_10_2016&Policy=Microsoft.Policies.WindowsLogon::VerboseStatus)
      - [Enable Detailed Status Messages at Shut down, Sign out, and Sign in](https://www.tenforums.com/tutorials/100262-enable-detailed-status-messages-shut-down-sign-out-sign.html)
```

**Sources:**

1. Display highly detailed status messages (`Microsoft.Policies.WindowsLogon::VerboseStatus`), https://admx.help/?Category=Windows_10_2016&Policy=Microsoft.Policies.WindowsLogon::VerboseStatus (tier A, ADMX policy reference; the host returned HTTP 522 intermittently during this work)
2. Enable Detailed Status Messages at Shut down, Sign out, and Sign in, https://www.tenforums.com/tutorials/100262-enable-detailed-status-messages-shut-down-sign-out-sign.html (tier C)

### `disable_startup_sound` Startup sound

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI\BootAnimation`,
`DisableStartupSound` (REG_DWORD). Disabled = `1`. `elevation: admin`, `requires_reboot: true`.

`DisableStartupSound` under the LogonUI `BootAnimation` key suppresses the Windows startup chime played
by the boot animation. The same setting name appears in Microsoft's unattend reference as
`Microsoft-Windows-Authentication-AuthUI\DisableStartupSound`, a tier A confirmation of the setting's
existence and meaning even though that page describes the image-time route rather than the runtime
registry value.

The shipped `Logon.admx` on 26100.4061 does carry a `DisableStartupSound` policy, but at a **different**
key, `Software\Microsoft\Windows\CurrentVersion\Policies\System`, gated `supportedOn` =
`SUPPORTED_WindowsVistaTo7`, and its explain string in `en-US\Logon.adml` reads verbatim: "This policy
is not available in this version of Windows." The Group Policy route is retired and the `BootAnimation`
value is the correct live location. It is not the only one read, though: `mmsys.cpl`, `authui.dll` and
`LogonController.dll` on 26100.4061 each reference both `LogonUI\BootAnimation` and `Policies\System`,
so a machine that has the retired policy value set can override the preference this tweak writes.

**Wrong Stock Default as currently authored:**

```
"Disabled" = 1
"Enabled (Stock Default)" = 0     <-- turns ON a chime the 26100 image ships switched off
```

On Windows 11 24H2 build 26100.4061 (IoT Enterprise LTSC 2024) the value is present and reads
`DisableStartupSound = 1 (REG_DWORD)`, and the `BootAnimation` key's last-write timestamp is
**2024-04-01**, the OS image-build date, more than a year before this machine was installed
(2025-05-13) and never touched since. The `1` is shipped state, not user or tool modification. So
selecting "Disabled" writes a value the machine already has (a no-op the app will report as a
successful change), and selecting the stock option *enables* a boot chime the image shipped off. Under
ADR-0003 that is reachable by a user who only ever wanted to undo the tweak.

**Corrections needed:** the safe shape is to make the shipped value (`1`) the Stock Default and offer
`0` as an explicit "play the startup sound" choice, renaming the tweak accordingly. The snapshot still
carries whatever the machine actually had, so a real revert is unaffected either way. Do not simply
flip the literal without checking a retail Home or Pro 26100 image first, because the SKU families may
differ; see open question 4. `elevation: admin`, `requires_reboot: true` and the REG_DWORD typing are
all correct.

**Ready-to-paste info block:**

```yaml
    info: |
      **Silences the Windows startup chime played when the sign-in screen appears.**

      ## What it does
      Sets `DisableStartupSound` to 1 under
      `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI\BootAnimation`, the
      value the boot animation reads before playing the startup sound. Microsoft's unattend
      reference documents the same setting name for image-time configuration.

      ## Benefits
      - **Quiet boot**: nothing plays in a shared office, a lecture hall or a bedroom
      - **Machine-wide**: applies to every account, not just yours
      - **Documented setting**: the same name Microsoft exposes for unattended installs

      ## Drawbacks
      - **Loses an audible cue**: no signal that the machine has reached the sign-in screen
      - **Often already off**: many Windows 11 images ship with the chime disabled, so applying it
        may change nothing
      - **Two gates**: the "Windows Startup" entry in the Sound settings scheme is a second switch,
        so clearing only one can leave the chime playing
      - **A stale Group Policy value can win**: a machine carrying the retired
        `Policies\System\DisableStartupSound` value can override this preference

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: after reboot, since the sound is played by the boot animation
      - **Reverting**: restores the previous value from the snapshot
      - Needs administrator rights, since the value lives in HKLM.

      ## Recommendation
      Apply it on any machine that boots in a shared or quiet space. Leave it if you use the chime as
      your cue that the machine is ready to sign in to.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft-Windows-Authentication-AuthUI DisableStartupSound (unattend reference)](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-authentication-authui-disablestartupsound)
      - [Enable or Disable Startup Sound in Windows 11](https://www.elevenforum.com/t/enable-or-disable-startup-sound-in-windows-11.85/)
```

**Sources:**

1. Shipped registry on Windows 11 24H2 build 26100.4061 (IoT Enterprise LTSC 2024): `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI\BootAnimation\DisableStartupSound = 1 (REG_DWORD)`, key last-write 2024-04-01, the image build date, versus an install date of 2025-05-13 (tier A, shipped OS state)
2. Shipped ADMX on 26100.4061: `C:\Windows\PolicyDefinitions\Logon.admx` policy `DisableStartupSound`, key `Software\Microsoft\Windows\CurrentVersion\Policies\System`, `supportedOn` `SUPPORTED_WindowsVistaTo7`; `en-US\Logon.adml` explain string "This policy is not available in this version of Windows." (tier A)
3. `Microsoft-Windows-Authentication-AuthUI-DisableStartupSound` unattend reference, https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-authentication-authui-disablestartupsound (tier A)
4. Enable or Disable Startup Sound in Windows 11, https://www.elevenforum.com/t/enable-or-disable-startup-sound-in-windows-11.85/ (tier C)
5. Disable Windows 11 Startup Sound using these three methods, https://winaero.com/disable-windows-11-startup-sound-using-these-three-methods/ (tier C)

### `disable_accessibility_key_prompts` Accessibility shortcut prompts

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** three REG_SZ `Flags` values under `HKCU\Control Panel\Accessibility`.

| Key | Value | Type | Prompts off | Common default |
|---|---|---|---|---|
| `...\StickyKeys` | `Flags` | REG_SZ | `"506"` | `"510"` |
| `...\Keyboard Response` | `Flags` | REG_SZ | `"122"` | `"126"` |
| `...\ToggleKeys` | `Flags` | REG_SZ | `"58"` | `"62"` |

Each `Flags` value is the decimal form of the `dwFlags` member of the corresponding Win32 structure
(`STICKYKEYS`, `FILTERKEYS`, `TOGGLEKEYS`), stored as a string. The REG_SZ typing is correct and
load-bearing: written as REG_DWORD these are ignored.

The arithmetic checks out against Microsoft's documented flag constants. In all three structures the
hotkey-activation bit is `0x00000004` (`SKF_HOTKEYACTIVE`, `FKF_HOTKEYACTIVE`, `TKF_HOTKEYACTIVE`).
510 - 506 = 4, 126 - 122 = 4, 62 - 58 = 4. Each option clears exactly the hotkey-active bit and leaves
`*_AVAILABLE`, `*_CONFIRMHOTKEY`, `*_HOTKEYSOUND`, `*_INDICATOR` and `*_AUDIBLEFEEDBACK` untouched.
That is precisely the right bit: it disables the five-Shift-presses, eight-second-Shift-hold and
five-second-NumLock-hold activation gestures without turning off the accessibility feature for anyone
who enables it deliberately from Settings.

**Wrong revert as currently authored:**

```
"Default (Stock Default)" = "510" / "126" / "62"     <-- constants, not the observed value
```

`Flags` is a live bitmask reflecting every sub-option the user has configured. 510, 126 and 62 are only
the *common* defaults on a machine where nothing else was changed. Restoring the constants silently
resets unrelated accessibility sub-settings such as audible feedback, lock modifier and visual
indicators.

**Corrections needed:** two items. (1) The revert must read the current value and set bit `0x4` rather
than write a fixed string. (2) Add a sign-out requirement, or note it in the info text; Windows reads
these at session start through `SystemParametersInfo`, and the YAML sets no `requires_reboot`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the Sticky Keys, Filter Keys and Toggle Keys popups that fire from stray key presses.**

      ## What it does
      Clears bit 0x4 (the hotkey-active bit) from the `Flags` string under
      `HKCU\Control Panel\Accessibility\StickyKeys`, `...\Keyboard Response` and `...\ToggleKeys`.
      That disables the keyboard activation gestures (five Shift presses, an eight-second Shift
      hold, a five-second NumLock hold) without disabling the accessibility features themselves.

      ## Benefits
      - **No mid-game interruption**: rapid Shift presses stop triggering a modal prompt
      - **Features stay available**: Settings can still turn Sticky Keys on deliberately
      - **Surgical**: only one bit changes, so other accessibility sub-options are untouched

      ## Drawbacks
      - **Removes an accessibility shortcut**: someone who relies on the keyboard gesture to enable
        Sticky Keys loses it, which is a real consideration on a shared account
      - **Settings UI lags**: the change may not show in Settings until the next sign-in
      - **Needs a sign-out**: Windows reads these values when the session starts

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after sign-out
      - **Reverting**: restores the previous flag values from the snapshot

      ## Recommendation
      Apply it if you game or type fast and the prompts keep interrupting you. Do not apply it on an
      account shared with someone who uses the keyboard gestures to switch these features on.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [STICKYKEYS structure (SKF_HOTKEYACTIVE = 0x00000004)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys)
      - [FILTERKEYS structure (FKF_HOTKEYACTIVE = 0x00000004)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-filterkeys)
      - [TOGGLEKEYS structure (TKF_HOTKEYACTIVE = 0x00000004)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-togglekeys)
```

**Sources:**

1. STICKYKEYS structure (documents `SKF_HOTKEYACTIVE` = 0x00000004 and the rest of the bitmask), https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys (tier A)
2. FILTERKEYS structure (documents `FKF_HOTKEYACTIVE` = 0x00000004), https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-filterkeys (tier A)
3. TOGGLEKEYS structure (documents `TKF_HOTKEYACTIVE` = 0x00000004), https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-togglekeys (tier A)
4. Turn On or Off Sticky Keys in Windows 11 (documents the REG_SZ `Flags` values and the 510 / 506 pair), https://www.elevenforum.com/t/turn-on-or-off-sticky-keys-in-windows-11.8889/ (tier C)
5. Disabling StickyKeys for Good, https://blog.duklabs.com/disabling-stickykeys-for-good/ (tier C)

### `classic_context_menu_win11` Classic context menu

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32`,
default value (name `""`), REG_SZ. Classic = `""` (empty string), Modern (Stock Default) = `absent`.
Gated `windows: { products: [11] }`.

`{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}` is the COM class implementing the Windows 11 compact
("Show more options") context menu. Registering an empty `InprocServer32` default value under the
per-user CLSID key makes COM activation for that class fail, and Explorer falls back to the legacy
Windows 10 context menu, in which every registered shell extension appears on the first click. The
YAML's shape (empty value name, REG_SZ, empty data) is exactly what
`reg add "HKCU\Software\Classes\CLSID\{86ca1aa0-...}\InprocServer32" /f /ve` produces. `absent` is the
correct stock default: the per-user key does not exist on a clean install.

**On whether Microsoft has blocked it.** The method has been described as deprecated since 24H2, but
multiple 2025 and 2026 references state that it continues to work on 24H2 and 25H2, and it is still
the standard registry route being documented for 25H2. The 24H2 re-scope reached the same conclusion
independently: the CLSID workaround is absent from both the deprecated-features and removed-features
lists, and "broken on 25H2" reports trace to a Microsoft-answered thread where the cause was writing to
the elevated account's HKCU or not restarting Explorer. Flagged deprecated, not blocked.

**On the revert.** `absent` deletes the `(Default)` value only and leaves an empty
`HKCU\Software\Classes\CLSID\{86ca1aa0-...}\InprocServer32` subkey behind, whereas every published
recipe deletes the whole CLSID key. That difference was worth testing, because the mechanism depends on
an empty string shadowing the HKLM registration and a leftover shadowing key would mean the classic
menu never comes back. Tested directly on 26100.4061: `HKEY_CLASSES_ROOT` merges HKCU over HKLM **per
value, not per key**. With an HKCU `InprocServer32` subkey present but carrying no values,
`HKCR\CLSID\{...}\InprocServer32` still resolves to the HKLM registration; only writing the empty
default value shadows it, and deleting that one value alone restores the HKLM path immediately. The
revert therefore works as authored, and the residue is a stray empty key, cosmetic only. (Verified
against `{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}`, which has the same HKLM `InprocServer32` shape, so as
not to disturb the live context-menu class.)

**Corrections needed:** none. The `absent` revert is confirmed sufficient.

**Ready-to-paste info block:**

```yaml
    info: |
      **Brings back the full Windows 10 right-click menu, so shell extensions appear on the first click.**

      ## What it does
      Creates an empty default (REG_SZ) value at
      `HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32`. That makes
      COM activation of the Windows 11 compact context menu fail, and Explorer falls back to the
      legacy menu where every registered extension is listed directly.

      ## Benefits
      - **No "Show more options" step**: 7-Zip, Git, TortoiseSVN and similar entries appear at once
      - **All extensions visible**: legacy shell extensions that never adopted the new menu API show up
      - **Clean revert**: deleting the one value restores the Windows 11 menu immediately

      ## Drawbacks
      - **Loses the Windows 11 design**: including the icon row for cut, copy, rename and delete
      - **Modern-only entries may vanish**: apps that register only into the new menu API can lose
        their items
      - **Officially deprecated**: Microsoft has flagged the key, so a future release could stop
        honouring it
      - **Leaves a stray key**: reverting removes the value but leaves an empty subkey, which is
        cosmetic and harmless

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (and every Windows 11 build); confirmed still
        working on 25H2
      - **Takes effect**: after restarting Explorer
      - **Reverting**: deletes the empty default value, which is enough to restore the modern menu
      - Apply it as your own user. Writing it while running elevated targets the wrong profile, which
        is the usual cause of "it did not work" reports.

      ## Recommendation
      Apply it if you use shell-extension tools daily; the extra click is a constant tax. Skip it if
      you like the compact menu and its icon row.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Restore old right-click context menu in Windows 11 (Microsoft Q&A article)](https://learn.microsoft.com/en-us/answers/questions/2287432/article-restore-old-right-click-context-menu-in-wi)
      - [4 Ways to Get the Old Context Menus Back in Windows 11 25H2](https://www.techbloat.com/4-ways-to-get-the-old-context-menus-back-in-windows-11-25h2.html)
```

**Sources:**

1. Direct test on 26100.4061: `HKEY_CLASSES_ROOT` merges HKCU over HKLM per value, not per key, so deleting the empty default value alone restores the HKLM `InprocServer32` registration (tier A, primary measurement)
2. Restore old Right-click Context menu in Windows 11, Microsoft Q&A article, https://learn.microsoft.com/en-us/answers/questions/2287432/article-restore-old-right-click-context-menu-in-wi (tier D as a Q&A post, but the widely cited canonical write-up and hosted on Microsoft Q&A)
3. 4 Ways to Get the Old Context Menus Back in Windows 11 25H2 (confirms the registry method remains the working route on 25H2), https://www.techbloat.com/4-ways-to-get-the-old-context-menus-back-in-windows-11-25h2.html (tier C)
4. How To Enable Classic Context Menu In Windows 11 (Regedit), https://memstechtips.com/enable-classic-context-menu-windows-11-regedit/ (tier C)

### `numlock_on_startup` NumLock at startup

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** `HKCU\Control Panel\Keyboard`, `InitialKeyboardIndicators` (REG_SZ).
NumLock on = `"2147483650"`, Default (Stock Default) = `absent`. `requires_reboot: true`,
`elevation: user`.

Microsoft documents `InitialKeyboardIndicators` at `HKCU\Control Panel\Keyboard` as `REG_SZ` with a
range of `0` or `2` and a default of `0`, where `0` means NumLock off and `2` means NumLock on.
Critically, the same page states that "the system stores the state of the NUMLOCK key in this entry
during logoff and shutdown, and then it uses this value to restore the state when the user logs on".
The value is therefore a scratch record Windows overwrites at every sign-out, not a preference the
system reads and respects indefinitely.

`2147483650` is `0x80000002`, the documented `2` with the high bit set. The high bit is an undocumented
extension used from Windows 8 onward and is community-reported to be more reliable than a bare `2` on
modern builds. `2147483648` (`0x80000000`) is the corresponding "NumLock off" form and is what commonly
appears in `HKU\.DEFAULT`.

**Corrected mechanism if the sign-in screen is actually wanted:**

```
HKU\.DEFAULT\Control Panel\Keyboard
  InitialKeyboardIndicators  (REG_SZ)  "2147483650"   <-- requires elevation: admin
HKCU\Control Panel\Keyboard
  InitialKeyboardIndicators  (REG_SZ)  "2147483650"   Stock Default: "0", not absent
```

**Wrong as currently authored:**

```
effects: HKCU write only                              <-- cannot affect the sign-in screen
"Default (Stock Default)" = absent                    <-- Microsoft documents a default of "0"
```

The sign-in screen does not run in the user's profile. It reads
`HKU\.DEFAULT\Control Panel\Keyboard\InitialKeyboardIndicators`, which the tweak never touches, so as
authored it cannot turn NumLock on at the sign-in screen, which is exactly what its own info text
promises.

**Corrections needed:** four items. (1) The info text's sign-in-screen claim is false for the tweak as
authored; either remove it or add the `HKU\.DEFAULT` write and raise `elevation` to `admin`, since
`.DEFAULT` is not writable by a standard user. (2) The Stock Default of `absent` is wrong: Microsoft
documents a default of `"0"`, and in practice the value is present on every profile because Windows
writes it at logoff. Revert should write `"0"`, not delete. (3) `2147483650` is undocumented; `"2"` is
the value Microsoft lists, with the caveat that community reports favour the high-bit form on Windows 8
and later. (4) Add the laptop numeric-keypad-overlay caution, which is a genuine footgun and is
currently missing.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns NumLock on automatically for your sessions, so the numeric keypad works right away.**

      ## What it does
      Writes `InitialKeyboardIndicators` as a string under `HKCU\Control Panel\Keyboard`. Microsoft
      documents 0 for NumLock off and 2 for NumLock on; the high-bit form 2147483650 is the variant
      community-reported as more reliable on Windows 8 and later. This is the per-user value only.

      ## Benefits
      - **Keypad ready**: digits work from the moment your session starts
      - **No manual toggle**: nothing to press after every sign-in
      - **No admin rights**: it writes only your own profile

      ## Drawbacks
      - **Does not cover the sign-in screen**: that reads a separate value under the default profile
        which this tweak does not touch
      - **Un-applies itself**: Windows rewrites this value at every sign-out and shutdown from the
        live NumLock state, so it can drift back and read as a failed apply
      - **Laptop keypad overlay**: on laptops with an embedded numeric overlay, NumLock-on turns
        letter keys into digits
      - **Fast Startup can defeat it**: the pre-shutdown keyboard state may be restored from the
        hibernation image instead

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after signing out and back in
      - **Reverting**: restores the previous value from the snapshot
      - Some UEFI firmwares have their own NumLock-at-boot setting that interacts with this.

      ## Recommendation
      Apply it on a desktop with a full-size keyboard where you enter numbers often. Skip it on a
      laptop with an embedded keypad overlay, where NumLock-on breaks normal typing.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [InitialKeyboardIndicators (Microsoft reference: REG_SZ, range 0 or 2, default 0, rewritten at logoff)](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/cc978657(v=technet.10))
      - [Enable NumLock on the Windows login screen](https://winaero.com/enable-numlock-logon-screen-windows-10/)
```

**Sources:**

1. `InitialKeyboardIndicators` (documents `HKCU\Control Panel\Keyboard`, REG_SZ, range 0 or 2, default 0, and the logoff-rewrite behaviour), https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/cc978657(v=technet.10) (tier A, archived Microsoft reference)
2. Enable NumLock on the Windows 10 Login screen / Lock screen (documents the `HKU\.DEFAULT` requirement for the sign-in screen and the 2147483650 form), https://winaero.com/enable-numlock-logon-screen-windows-10/ (tier C)
3. `InitialKeyboardIndicators` registry setting to fix NumLock in a certain state (documents 2147483648 in `.DEFAULT`), https://wiert.me/2016/09/30/initialkeyboardindicators-registry-setting-to-fix-numlock-in-a-certain-state/ (tier C)

### `disable_start_recommended_section` Start Recommended section

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` **or**
`HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, `HideRecommendedSection` (REG_DWORD). Hidden =
`1`, Stock Default = `absent`.

The shipped `StartMenu.admx` on 26100 declares it `class="Both"`, so both hives are available:

```xml
<policy name="HideRecommendedSection" class="Both"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="HideRecommendedSection">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_SE" />
</policy>
```

The `SUPPORTED_Windows_11_0_SE` reference is a **gpedit display annotation only**: it controls the
"Requirements:" line in the policy editor and does not gate what the OS reads. Microsoft's own Policy
CSP contradicts it and is authoritative on editions: Pro, Enterprise, Education, IoT Enterprise and
IoT Enterprise LTSC are all supported; applicable OS is Windows 11 version 22H2 [10.0.22621] and
later; allowed values are `0 (Default)` section shown and `1` section hidden. Polarity is not inverted.

The value is read on 26100: the literal `HideRecommendedSection` is present in
`C:\Windows\SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\StartMenu.dll` and
`C:\Windows\System32\SHCore.dll`. In `StartMenu.dll` it sits inside a contiguous run of Start policy
value names alongside `HideAppList`, `NoStartMenuMorePrograms` and `DisableContextMenus`, all of which
are known-working Start policies read by the same code path.

**Not a duplicate.** The existing `disable_start_recommendations` writes `Start_IrisRecommendations`
under `Explorer\Advanced`: different key, different value, different scope. That one removes the
promoted rows; this one removes the whole section.

**Corrections needed:** ship without the PolicyManager fallback. Writing
`HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Start` is unnecessary and must not be used: it is
the MDM policy cache, not an authoring surface. The direct policy value is read on Pro-class editions.
The revert must be `absent`; the `<policy>` element declares no `enabledValue` and no `disabledValue`,
so there is no Microsoft-defined `0` state and writing one is undefined behaviour no source covers.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the entire Recommended section from the Start menu, not just the promoted rows.**

      ## What it does
      Sets the `HideRecommendedSection` policy to 1 under
      `SOFTWARE\Policies\Microsoft\Windows\Explorer`. The Start menu drops the whole Recommended
      area, so the pinned app grid takes the space instead. This is a different control from the
      "Show recommendations" toggle, which only clears the promoted rows inside the section.

      ## Benefits
      - **Reclaims the space**: pinned apps fill the panel instead of a half-empty area
      - **Removes everything in it**: promoted tips, suggested apps and recent files together
      - **Documented policy**: defined in the shipped Start menu ADMX and the Start Policy CSP

      ## Drawbacks
      - **Loses recent files in Start**: the fastest route back to a document you just closed
      - **Not on Home**: Microsoft lists Pro, Enterprise, Education and IoT Enterprise editions
      - **Start menu restart needed**: the layout does not reflow until the Start host restarts

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (policy applies from 22H2, build 22621)
      - **Takes effect**: after signing out, or when `StartMenuExperienceHost.exe` restarts; no
        reboot is required
      - **Reverting**: removes the value, which is the unconfigured state; the policy defines no
        explicit "off" value

      ## Recommendation
      Apply it if you use Start as an app launcher and nothing else. Skip it if you reopen recent
      documents from Start, since that list lives in this section.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Start > HideRecommendedSection](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start)
      - Shipped `StartMenu.admx` and `en-US\StartMenu.adml` on Windows 11 build 26100
```

**Sources:**

1. `C:\Windows\PolicyDefinitions\StartMenu.admx` and `en-US\StartMenu.adml` on 26100: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, `valueName="HideRecommendedSection"`, no `enabledValue` or `disabledValue` declared (tier A)
2. Policy CSP - Start > HideRecommendedSection (editions, applicable OS 22621+, allowed values 0 default / 1 hidden, Computer and User Configuration), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start (tier A)
3. Binary string evidence on 26100.4061: `HideRecommendedSection` present in `StartMenu.dll` and `SHCore.dll`, in `StartMenu.dll` inside a run of known-working Start policy value names (tier A, product artifact)

### `hide_unsupported_hardware_notice` Unsupported hardware notice

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`,
`HideUnsupportedHardwareNotifications` (REG_DWORD). Hidden = `1`, Stock Default = `absent`.
Machine scope, requires admin.

The shipped `ControlPanel.admx` on 26100 declares it exactly as authored:

```xml
<policy name="HideUnsupportedHardwareNotifications" class="Machine"
        key="Software\Microsoft\Windows\CurrentVersion\Policies\System"
        valueName="HideUnsupportedHardwareNotifications">
  <supportedOn ref="windows:SUPPORTED_Windows_11_0_NOSERVER" />
</policy>
```

The ADML is unambiguous: "This policy controls messages which are shown when Windows is running on a
device that does not meet the minimum system requirements for this OS version. If you enable this
policy setting, these messages will never appear on desktop or in the Settings app."

Two surfaces, and the binary evidence maps one to one onto them: `C:\Windows\System32\shell32.dll`
(the desktop watermark) and `C:\Windows\System32\AboutSettingsHandlers.dll` (the Settings > System >
About banner). Those are precisely the watermark host and the About page handler.

The benefit is real but purely cosmetic. It does **not** change update eligibility or servicing
behaviour; the value is read only by presentation-layer binaries, which supports that. Keep the
negative claim in the copy so nobody reads the tweak as "makes my PC supported".

`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` is a high-traffic key (UAC values,
legal notice and more), but under per-value snapshotting that is not a collision, and no other corpus
tweak writes this value name.

**Corrections needed:** none, with one hard constraint. The `<policy>` element declares **no**
`enabledValue` and **no** `disabledValue`, so there is no Microsoft-defined `0` state. The revert must
be `absent`, and the YAML must not later be "helpfully" changed to write `0`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Clears the "system requirements not met" watermark from the desktop and the About page.**

      ## What it does
      Sets the `HideUnsupportedHardwareNotifications` policy to 1 under
      `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`. Per the shipped policy text,
      messages about not meeting minimum system requirements never appear on the desktop or in the
      Settings app.

      ## Benefits
      - **Clean desktop**: the corner watermark over your wallpaper is gone
      - **Clean About page**: the Settings > System > About banner clears too
      - **Documented policy**: defined in the shipped Control Panel ADMX, machine-wide

      ## Drawbacks
      - **Cosmetic only**: it changes nothing about update eligibility or servicing; an unsupported
        PC stays unsupported
      - **Hides a real signal**: if you forget the machine is unsupported, an upgrade may still be
        blocked later
      - **Needs admin rights**: it writes to HKLM and applies to every user

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (all Windows 11 client editions)
      - **Takes effect**: after restarting Explorer for the desktop watermark; the Settings About
        banner clears the next time that page is opened
      - **Reverting**: removes the value, which is the unconfigured state; the policy defines no
        explicit "off" value

      ## Recommendation
      Apply it on a machine you deliberately run Windows 11 on despite the hardware check, where the
      watermark is just noise. Skip it if the reminder is useful to you.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - Shipped `ControlPanel.admx` and `en-US\ControlPanel.adml` on Windows 11 build 26100
      - Binary evidence: the value name appears in `shell32.dll` (watermark) and
        `AboutSettingsHandlers.dll` (About page) on build 26100
```

**Sources:**

1. `C:\Windows\PolicyDefinitions\ControlPanel.admx` and `en-US\ControlPanel.adml` on 26100: `class="Machine"`, key `Software\Microsoft\Windows\CurrentVersion\Policies\System`, `valueName="HideUnsupportedHardwareNotifications"`, `supportedOn` `SUPPORTED_Windows_11_0_NOSERVER`, no `enabledValue` or `disabledValue` (tier A)
2. Binary string evidence on 26100: the value name appears in `C:\Windows\System32\shell32.dll` and `C:\Windows\System32\AboutSettingsHandlers.dll`, the desktop watermark host and the Settings About page handler (tier A, product artifact)

### `disable_phone_companion_start` Mobile device panel in Start

**Verdict:** VERIFIED (new in this revision)

**Mechanism:**
`HKCU\Software\Microsoft\Windows\CurrentVersion\Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe`,
`IsEnabled` (REG_DWORD). Disabled = `0`, Stock Default = `absent`. `requires_reboot: false`.

The key path literal `Software\Microsoft\Windows\CurrentVersion\Start\Companions` is present in two
shipped Start binaries on 26100:
`SystemApps\Microsoft.Windows.StartMenuExperienceHost_cw5n1h2txyewy\StartDocked.dll` and
`SystemApps\MicrosoftWindows.Client.Core_cw5n1h2txyewy\StartMenu.dll`. In `StartMenu.dll` the key path
is followed immediately by the per-companion value-name run, which contains `IsEnabled`:

```
Software\Microsoft\Windows\CurrentVersion\Start\Companions | com.microsoft.startmenucompanion |
CompanionSettingName | CompanionSettingExtendedName | DetailedSettingsLaunchUri |
DetailedSettingsLaunchName | RefreshCardLaunchUri | OverridesPackage | DefaultState | Icon |
IsEnabled | IsAvailable | ... | Enabled | Disabled | Unavailable | %ls\%ls
```

The trailing `%ls\%ls` format string is the `<CompanionsKey>\<PFN>` subkey composition, so `IsEnabled`
is a real per-companion value under a per-PFN subkey.

The hive and the package family name cannot be read off a binary string, but three genuinely
independent sources agree on `HKEY_CURRENT_USER` plus that exact PFN: Win11Debloat's
`Regfiles/Disable_Phone_Link_In_Start.reg` (raw `.reg` body, `"IsEnabled"=dword:00000000`), the
ElevenForum tutorial 26919 via the Wayback Machine (identical body, plus the build attribution
26100.3915 for 24H2 and 22631.5262 for 23H2), and Microsoft Q&A 5510106, whose answer walks the user to
the same key and value.

`absent` is the right revert: the binary carries a `DefaultState` value name alongside `IsEnabled`,
implying the shell falls back to a package-declared default when `IsEnabled` is missing. Note that
Win11Debloat's undo writes `1` instead of deleting, so the two reverts are **not** the same end state;
`absent` is the more faithful one. The revert restores "whatever the package declares", not necessarily
"shown", so the copy must not promise the panel comes back if Phone Link has since been removed.

This is Settings-backed, not a hidden hack: Microsoft's support article "Mobile device in Start menu"
documents Personalization > Start > "Show mobile device in Start", and this value is its backing store.
The practical consequence is drift: a user flipping the Settings toggle desynchronises the tweak from
its snapshot.

**Not a duplicate.** `Companions` and `YourPhone` appear nowhere else in the corpus as registry
effects. `debloat:remove_phone_link` removes the appx, a different and less reliable action.

**Corrections needed:** `requires_reboot` must be **false**. No source specifies a sign-out step, and
ElevenForum reliably calls one out when it is needed; the Microsoft Q&A "restart your PC" line is
troubleshooting advice for a panel that failed to appear. Also, the tweak must treat a missing key as a
clean no-op rather than an error, since it is inert wherever Phone Link is not installed, which
includes IoT Enterprise LTSC images.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the phone panel from the Start menu without uninstalling Phone Link.**

      ## What it does
      Sets `IsEnabled` to 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe`,
      the backing store for Settings > Personalization > Start > "Show mobile device in Start". The
      Phone Link app itself is untouched.

      ## Benefits
      - **Narrower Start menu**: the side panel goes and Start returns to its normal width
      - **App still works**: Phone Link keeps running and syncing if you open it directly
      - **Settings-backed**: a supported toggle, so the behaviour is stable

      ## Drawbacks
      - **Loses quick phone access**: notifications, battery and recent photos are no longer one
        click from Start
      - **Drifts from the snapshot**: flipping the Settings toggle later changes the value behind
        this tweak's back
      - **Inert without Phone Link**: on an image that does not ship Phone Link there is nothing to
        turn off

      ## Good to know
      - **Applies to**: Windows 11 24H2 from build 26100.3915 (and 23H2 from 22631.5262)
      - **Takes effect**: immediately, the next time the Start menu is opened; no reboot needed
      - **Reverting**: removes the value, so the panel returns to whatever the app package declares
        as its default rather than to a forced "shown"

      ## Recommendation
      Apply it if you do not pair a phone or you find the panel makes Start too wide. Skip it if you
      use the phone panel for notifications and photo transfers.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Mobile device in Start menu (Microsoft support)](https://support.microsoft.com/en-us/windows/mobile-device-in-start-menu-21676d6a-3bc3-439a-aaa3-7463b91cda79)
      - Binary evidence: the `Start\Companions` key path and the `IsEnabled` value name appear in
        `StartDocked.dll` and `StartMenu.dll` on build 26100
```

**Sources:**

1. Binary string evidence on 26100.4061: `Software\Microsoft\Windows\CurrentVersion\Start\Companions` in `StartDocked.dll` and `StartMenu.dll`, with the per-companion value-name run containing `IsEnabled`, `DefaultState` and the `%ls\%ls` subkey format string (tier A, product artifact)
2. Mobile device in Start menu, Microsoft support (documents the Settings surface this value backs), https://support.microsoft.com/en-us/windows/mobile-device-in-start-menu-21676d6a-3bc3-439a-aaa3-7463b91cda79 (tier A, for the Settings surface)
3. Win11Debloat `Regfiles/Disable_Phone_Link_In_Start.reg`, read as the raw `.reg` body (tier C)
4. ElevenForum tutorial 26919 via the Wayback Machine (identical `.reg` body plus the 26100.3915 / 22631.5262 build attribution) (tier C)
5. Microsoft Q&A 5510106 (same key and value; a participant reports the key on 26100.4770) (tier C)

Explicitly **not** counted: GeekRewind, which reads as a rewrite of the ElevenForum tutorial;
TheWindowsClub and NinjaOne, which are JavaScript-gated and whose served HTML contains neither
`Companions` nor `IsEnabled`.

### `disable_drag_tray` Drop Tray share overlay

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\CDP`, `DragTrayEnabled` (REG_DWORD).
Disabled = `0`, Stock Default = `absent`. `0` = off, `1` = on, absent = on.

The key is genuinely `CurrentVersion\CDP`, the Connected Devices Platform key, and specifically **not**
`Explorer\Advanced` or any Shell key. That was the main doubt, because CDP's established per-user
tenants are authorisation policies (`RomeSdkChannelUserAuthzPolicy`, `NearShareChannelUserAuthzPolicy`,
`CdpUserSettingsVersion`, `EnableRemoteLaunchToast`) and a UI affordance toggle is not an obvious
neighbour. Three independent sources carry the identical literal:

```
[HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\CDP]
"DragTrayEnabled"=dword:00000000
```

Win11Debloat `Regfiles/Disable_Share_Drag_Tray.reg` (raw `.reg` body; its undo writes `1`), ElevenForum
tutorial 40485 via the Wayback Machine (carrying both the on and off bodies), and MajorGeeks, which is
firsthand and useful precisely because the author narrates creating the value and says "I had to create
it from scratch while testing this". That sentence independently confirms **Stock Default = `absent`**,
which is what the revert depends on. AskVG is a fourth, partially independent source with the same key,
value and polarity.

**On the negative binary result.** `DragTrayEnabled`, and the substrings `DragTray` and `DropTray`,
were searched in both UTF-16LE and ASCII across 8,194 binaries under `System32` and `SystemApps`, 5,207
under `C:\Program Files\WindowsApps`, and 1,359 shell, CDP, share and Explorer related `WinSxS`
component directories. Zero hits. That does not refute the tweak: the inspected machine is 26100.4061
and the feature ships at 26100.4202, so the code is genuinely not on that image. The negative result is
what the sources predict, and it doubles as corroboration of the default, since the `CDP` key exists
there with its five established values and no `DragTrayEnabled`.

**Corrections needed:** three items.

1. **The feature was renamed.** Per ElevenForum's changelog, at builds 26100.8328 (24H2), 26200.8328
   (25H2) and 28000.2179 (26H1), "Drag Tray has been now renamed to Drop Tray", and its Settings home
   moved from System > Nearby sharing to **System > Multitasking**. Name the tweak for **Drop Tray**
   with "Drag Tray" as an alias in the description, or users on current servicing will not recognise
   it. A Settings toggle has existed since 26100.7309 / 26200.7309, so this is a supported switch now,
   which is good for reliability and bad for drift.
2. **Strike the Pureinfotech source.** That page never mentions `DragTrayEnabled` or the `CDP` key. It
   documents a completely different mechanism, a feature-flag override at
   `HKLM\SYSTEM\ControlSet001\Control\FeatureManagement\Overrides\14\3895955085` with `EnabledState`
   and `EnabledStateOptions` plus a reboot. Counting it would be a false corroboration, and shipping
   it would be dangerous: feature-management override IDs are build-specific and are not a stable
   authoring surface. The claim clears the three-independent-source bar without it.
3. **Build attribution.** 26100.4202 is right for the 24H2 GA channel; AskVG's 26200.5518 (KB5055625,
   April 2025) is the earlier Dev-channel debut. Both are true and not in conflict. The corpus has no
   revision-level gate, so `build >= 26100` plus copy noting that early 26100 servicing levels have
   nothing to disable is the best available.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the share overlay from sliding down at the top of the screen when you drag a file.**

      ## What it does
      Sets `DragTrayEnabled` to 0 under `HKCU\Software\Microsoft\Windows\CurrentVersion\CDP`. That is
      the backing value for the Drop Tray (originally Drag Tray) share surface, which Windows shows
      at the top of the screen while you drag files. Its Settings home is System > Multitasking.

      ## Benefits
      - **No surprise overlay**: dragging files between folders stops summoning a share panel
      - **Fewer misdrops**: the overlay can intercept a drag aimed at a window behind it
      - **Ordinary drag and drop unaffected**: only the share surface goes

      ## Drawbacks
      - **Loses quick sharing**: the drag-to-share route to nearby devices and apps is gone
      - **Nothing to disable on older servicing**: the feature ships at 26100.4202, so earlier 24H2
        revisions have no Drop Tray at all
      - **Drifts from the snapshot**: the Settings toggle writes the same value behind this tweak's back

      ## Good to know
      - **Applies to**: Windows 11 24H2 from build 26100.4202, and 25H2; the feature was renamed from
        Drag Tray to Drop Tray at 26100.8328 and 26200.8328
      - **Takes effect**: immediately, no restart or sign-out needed
      - **Reverting**: removes the value, which is the stock state; the overlay is on when the value
        is absent

      ## Recommendation
      Apply it if you drag files between Explorer windows often and the overlay keeps getting in the
      way. Skip it if you actually share files to nearby devices by dragging.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or Disable Drop Tray in Windows 11 (ElevenForum tutorial 40485)](https://www.elevenforum.com/t/enable-or-disable-drag-tray-in-windows-11.40485/)
      - [How To Disable Drag Tray (MajorGeeks)](https://www.majorgeeks.com/content/page/how_to_disable_drag_tray.html)
```

**Sources:**

1. Win11Debloat `Regfiles/Disable_Share_Drag_Tray.reg`, raw `.reg` body (tier C)
2. ElevenForum tutorial 40485 via the Wayback Machine, https://www.elevenforum.com/t/enable-or-disable-drag-tray-in-windows-11.40485/ (tier C; also the source for the rename to Drop Tray and the Settings relocation)
3. MajorGeeks "How To Disable Drag Tray", https://www.majorgeeks.com/content/page/how_to_disable_drag_tray.html (tier C, firsthand, and the source for value-absent being the default)
4. AskVG (tier C, Dev-channel build attribution 26200.5518 / KB5055625)
5. Negative binary search across 14,760 shipped binaries on 26100.4061, consistent with the feature shipping at 26100.4202 (tier A, primary measurement)

**Rejected as a source:** Pureinfotech, which describes an unrelated feature-management override at
`HKLM\SYSTEM\ControlSet001\Control\FeatureManagement\Overrides\14\3895955085`.

### `alt_tab_hide_browser_tabs` Alt+Tab shows windows only

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**CRITICAL: the same value name exists at two keys with different enum bases.** Getting this wrong
produces the opposite of the requested behaviour while reporting success.

| Surface | Key | Value | Type | Enum |
|---|---|---|---|---|
| Group Policy (tier A) | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` | `MultiTaskingAltTabFilter` | REG_DWORD | `1` = windows + 20 tabs, `2` = windows + 5 tabs, `3` = windows + 3 tabs, **`4` = open windows only** |
| User preference (tier C) | `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `MultiTaskingAltTabFilter` | REG_DWORD | `0` = 20 tabs, `1` = 5 tabs, `2` = 3 tabs, **`3` = open windows only** |

**Mechanism (corrected, recommended):**

```
HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer
  MultiTaskingAltTabFilter (REG_DWORD)
    Windows only        = 4
    3 most recent tabs  = 3
    5 most recent tabs  = 2
    20 most recent tabs = 1
    Stock Default       = absent
```

**Wrong mechanism if the enums are crossed:**

```
HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer
  MultiTaskingAltTabFilter = 3     <-- this is "windows + 3 recent tabs", NOT windows only
```

Optionally also write the `Explorer\Advanced` preference with the **0-based** enum so the Settings UI
reflects the change. If both are written they must be kept in step; never reuse one number across both
keys.

**Evidence.** Shipped 26100 `Multitasking.admx`:
`<policy name="BrowserAltTabBlowout" class="User" key="Software\Policies\Microsoft\Windows\Explorer">`,
`supportedOn="windows:SUPPORTED_Windows_10_0_RS7"`, with
`<enum id="AltTabFilterDropdown" valueName="MultiTaskingAltTabFilter" required="true">` and items `1` =
`AltTabFilter_All`, `2` = `AltTabFilter_Five`, `3` = `AltTabFilter_Three`, `4` = `AltTabFilter_None`.
`en-US\Multitasking.adml` gives `AltTabFilter_None` = "Open windows only", `AltTabFilter_Three` = "Open
windows and 3 most recent tabs in apps", and the help text "If this is set to show 'Open windows only',
the whole feature will be disabled."

The 0-based preference enum is pinned by Win11Debloat's four sibling reg files, fetched and decoded:
`Hide_Tabs_In_Alt_Tab.reg` = `dword:00000003`, `Show_3_Tabs_In_Alt_Tab.reg` = `dword:00000002`,
`Show_5_Tabs_In_Alt_Tab.reg` = `dword:00000001`, `Show_20_Tabs_In_Alt_Tab.reg` = `dword:00000000`, all
at `[HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced]`. Binary presence
on 26100 in `twinui.dll` (which implements Alt+Tab, in the `Explorer\Advanced` string block alongside
`SnapAssist` and `StoreAppsOnTaskbar`), `SettingsHandlers_nt.dll` and `SHCore.dll`.

**Source correction.** The original proposal claimed three independent sources agreeing on the enum,
naming Win11Debloat, Sophia Script and an ElevenForum tutorial. Sophia's Windows 11 `Sophia.psm1`
contains no `MultiTaskingAltTabFilter` occurrence; neither do privacy.sexy or WinUtil; and the
ElevenForum page could not be retrieved through the Wayback Machine. The real community evidence is one
project's four sibling files, which pin the 0-based enum internally but count as one source. The tier A
ADMX is what carries this tweak.

**Corrections needed:** two beyond the enum split. `supportedOn` is `SUPPORTED_Windows_10_0_RS7`
(Windows 10 2004), not "Windows 11 21H2 and later"; and the ADML says "app tabs" generically, so the
copy must not assert that only Microsoft Edge is affected.

**Ready-to-paste info block:**

```yaml
    info: |
      **Alt+Tab lists your open windows again instead of filling up with browser tabs.**

      ## What it does
      Sets the `MultiTaskingAltTabFilter` policy to 4 under
      `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`. In the shipped Multitasking policy that
      value is "Open windows only", which the policy text says disables the tab-in-Alt-Tab feature
      entirely. Intermediate values keep 3, 5 or 20 recent tabs.

      ## Benefits
      - **Short, predictable list**: Alt+Tab returns to one entry per window
      - **Faster switching**: no scanning past twenty tab thumbnails to find an application
      - **Documented policy**: defined in the shipped Multitasking ADMX with a named enum

      ## Drawbacks
      - **Loses tab switching**: you can no longer reach a specific browser tab from Alt+Tab
      - **All tabbed apps**: the policy text says "app tabs" generically, so this is not limited to
        one browser
      - **Two keys, two enums**: the same value name exists as a user preference with a different
        numbering, so a hand-edit copied from the wrong guide sets the wrong option

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the policy is declared from Windows 10 version 2004
      - **Takes effect**: after restarting Explorer or signing out; the shipped policy states no
        reboot requirement
      - **Reverting**: removes the value, which is the unconfigured state

      ## Recommendation
      Apply it if you keep many tabs open and Alt+Tab has become useless for finding applications.
      Skip it if you deliberately switch to individual tabs with Alt+Tab.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - Shipped `Multitasking.admx` and `en-US\Multitasking.adml` on Windows 11 build 26100
      - Binary evidence: `MultiTaskingAltTabFilter` in `twinui.dll`, `SettingsHandlers_nt.dll` and
        `SHCore.dll` on build 26100
```

**Sources:**

1. Shipped `C:\Windows\PolicyDefinitions\Multitasking.admx` and `en-US\Multitasking.adml` on 26100: policy `BrowserAltTabBlowout`, `class="User"`, key `Software\Policies\Microsoft\Windows\Explorer`, enum `MultiTaskingAltTabFilter` with items 1 through 4 and the ADML display strings (tier A)
2. Binary string evidence on 26100: `MultiTaskingAltTabFilter` in `twinui.dll`, `SettingsHandlers_nt.dll` and `SHCore.dll` (tier A, product artifact)
3. Win11Debloat sibling reg files `Hide_Tabs_In_Alt_Tab.reg`, `Show_3_Tabs_In_Alt_Tab.reg`, `Show_5_Tabs_In_Alt_Tab.reg`, `Show_20_Tabs_In_Alt_Tab.reg`, fetched and decoded, pinning the 0-based preference enum (tier C, one source)

### `disable_snap_assist` Snap Assist suggestion picker

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `SnapAssist`
(REG_DWORD). Disabled = `0`, Enabled = `1`.

Two independent projects agree on key, name, type and polarity: Sophia Script `Sophia.psm1` lines 1697
and 1701 write `New-ItemProperty -Path HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced
-Name SnapAssist -PropertyType DWord -Value 0` for Disable and `-Value 1` for Enable; Win11Debloat
`Regfiles/Disable_Snap_Assist.reg` writes `"SnapAssist"=dword:00000000`.

Binary presence on 26100 in `twinui.dll` and `twinui.pcshell.dll` (the components that implement snap),
`Taskbar.View.dll`, and `SettingsHandlers_nt.dll` (the Settings Multitasking page handler), where it
sits in the same string block as `SnapFill`, `JointResize` and `MultiTaskingAltTabFilter`. It is a live
`Explorer\Advanced` preference on the target build and maps to a visible Settings toggle, which makes
the effect self-verifying.

**Source correction.** The original proposal cited privacy.sexy, which contains no `SnapAssist`
occurrence. Two projects plus binary presence plus the Settings-UI mapping is still sufficient.

**Not a duplicate.** `disable_snap_flyout` writes `EnableSnapAssistFlyout`, the hover-the-maximise-button
flyout. This is the post-snap suggestion picker, a different surface.

**Corrections needed:** none blocking. One low-severity residual: whether `SnapAssist` is present and
`1` on a stock profile, or absent, could not be established from an admissible source. If it is absent
by default, writing `1` on revert leaves a stray value, but the effective behaviour (Snap Assist on) is
identical, so this is not a harmful-revert case. See open question 8.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows suggesting what to put in the empty half after you snap a window.**

      ## What it does
      Sets `SnapAssist` to 0 under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
      the Settings > System > Multitasking checkbox "When I snap a window, show what I can snap next
      to it". Snapping itself is unaffected; only the suggestion picker goes.

      ## Benefits
      - **No interruption after snapping**: the other half stays showing your desktop or the window
        that was already there
      - **Snapping still works**: Win+Arrow, drag-to-edge and the layout grid are untouched
      - **Settings-backed**: a supported toggle you can verify by snapping a window

      ## Drawbacks
      - **Slower two-window setups**: you now pick the second window yourself
      - **Loses a discovery aid**: new users learn snapping partly through this picker
      - **One of three snap switches**: the hover flyout and the drag-to-top layout bar are separate
        settings

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the next time you snap a window
      - **Reverting**: restores the previous value from the snapshot

      ## Recommendation
      Apply it if you snap windows constantly and always know what goes beside them. Skip it if you
      routinely build side-by-side layouts, since the picker saves a step there.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - Sophia Script `Sophia.psm1` (writes `SnapAssist` at this key with values 0 and 1)
      - Win11Debloat `Regfiles/Disable_Snap_Assist.reg`
      - Binary evidence: `SnapAssist` in `twinui.dll`, `twinui.pcshell.dll` and
        `SettingsHandlers_nt.dll` on build 26100
```

**Sources:**

1. Sophia Script `Sophia.psm1` lines 1697 and 1701, writing `SnapAssist` as a DWord at `HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` with values 0 and 1 (tier C)
2. Win11Debloat `Regfiles/Disable_Snap_Assist.reg`, raw `.reg` body `"SnapAssist"=dword:00000000` (tier C)
3. Binary string evidence on 26100: `SnapAssist` in `twinui.dll`, `twinui.pcshell.dll`, `Taskbar.View.dll` and `SettingsHandlers_nt.dll`, in the same string block as `SnapFill` and `JointResize` (tier A, product artifact)

### `explorer_expand_to_current_folder` Expand the tree to the open folder

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`,
`NavPaneExpandToCurrentFolder` (REG_DWORD). Expand = `1`, Stock Default = `absent`.

Sophia Script `Sophia.psm1` lines 3434 and 3438 write exactly this value at exactly this key with
`-Value 0` and `-Value 1`, confirming type and polarity. Binary presence on 26100 in `shell32.dll`,
`ExplorerFrame.dll` and `Windows.UI.FileExplorer.dll`, all Explorer consumers. The Folder Options
checkbox "Expand to open folder" makes the mapping self-verifying in the UI.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **The Explorer sidebar tree opens and highlights whatever folder you are currently viewing.**

      ## What it does
      Sets `NavPaneExpandToCurrentFolder` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Folder Options > View
      checkbox "Expand to open folder". The navigation pane expands the branch containing the folder
      shown in the file list and scrolls it into view.

      ## Benefits
      - **Always know where you are**: the tree matches the file list instead of staying collapsed
      - **Faster sideways navigation**: sibling folders are one click away
      - **Officially exposed**: a Folder Options checkbox, easy to verify

      ## Drawbacks
      - **Long trees**: deep paths expand a lot of nodes and push everything else off screen
      - **Constant scrolling**: the pane jumps whenever you change folders
      - **Slow on network shares**: expanding a branch enumerates each level on the way

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: after restarting Explorer
      - **Reverting**: removes the value, which is the stock state

      ## Recommendation
      Apply it if you navigate by the folder tree and keep losing your place. Skip it if you work in
      deeply nested paths or over slow network shares, where the expansion is a nuisance.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - Sophia Script `Sophia.psm1` (writes `NavPaneExpandToCurrentFolder` at this key, values 0 and 1)
      - Binary evidence: the value name appears in `shell32.dll`, `ExplorerFrame.dll` and
        `Windows.UI.FileExplorer.dll` on build 26100
```

**Sources:**

1. Sophia Script `Sophia.psm1` lines 3434 and 3438, writing `NavPaneExpandToCurrentFolder` at `HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` with values 0 and 1 (tier C)
2. Binary string evidence on 26100: `NavPaneExpandToCurrentFolder` in `shell32.dll`, `ExplorerFrame.dll` and `Windows.UI.FileExplorer.dll` (tier A, product artifact)
3. Folder Options > View > "Expand to open folder", the Windows UI surface this value backs (tier A, product artifact)

### `explorer_restore_folders_at_logon` Restore Explorer windows at sign-in

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `PersistBrowsers`
(REG_DWORD). Restore = `1`, Stock Default = `absent`.

Sophia Script `Sophia.psm1` lines 6629 and 6633 write exactly this value at exactly this key with
`-Value 0` and `-Value 1`. Binary presence on 26100 in `shell32.dll`, `ExplorerFrame.dll` and
`gpprefcl.dll`. It maps to the Folder Options checkbox "Restore previous folder windows at logon", so
the effect is trivially verifiable.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Reopens the File Explorer windows you had open when you last signed out.**

      ## What it does
      Sets `PersistBrowsers` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, the Folder Options > View
      checkbox "Restore previous folder windows at logon". Windows records the open Explorer windows
      at sign-out and reopens them at the next sign-in.

      ## Benefits
      - **Resume where you left off**: your working folders come back without retyping paths
      - **Survives restarts**: useful on a machine that reboots for updates overnight
      - **Officially exposed**: a Folder Options checkbox, easy to verify

      ## Drawbacks
      - **Slower sign-in**: several windows are opened before the desktop settles
      - **Reveals your work**: previously open folder names appear on screen at sign-in, which is
        awkward on a shared or projected display
      - **Stale windows**: folders on disconnected network shares or removed drives fail to reopen

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10
      - **Takes effect**: at the next sign-in, which is when Explorer restores the windows
      - **Reverting**: removes the value, which is the stock state

      ## Recommendation
      Apply it on a personal workstation where you keep a fixed set of folders open. Skip it on a
      shared or laptop machine, especially if the folder names would be visible to others.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - Sophia Script `Sophia.psm1` (writes `PersistBrowsers` at this key, values 0 and 1)
      - Binary evidence: the value name appears in `shell32.dll`, `ExplorerFrame.dll` and
        `gpprefcl.dll` on build 26100
```

**Sources:**

1. Sophia Script `Sophia.psm1` lines 6629 and 6633, writing `PersistBrowsers` at `HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` with values 0 and 1 (tier C)
2. Binary string evidence on 26100: `PersistBrowsers` in `shell32.dll`, `ExplorerFrame.dll` and `gpprefcl.dll` (tier A, product artifact)
3. Folder Options > View > "Restore previous folder windows at logon", the Windows UI surface this value backs (tier A, product artifact)

### `taskbar_full_date_time` Full date and time in the tray

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`,
`TurnOffAbbreviatedDateTimeFormat` (REG_DWORD). Full format = `1`, Stock Default = `absent`.

Shipped 26100 `Taskbar.admx`: `class="User"`, key `Software\Policies\Microsoft\Windows\Explorer`,
`enabledValue 1` and `disabledValue 0`, `supportedOn="windows:SUPPORTED_Windows_11_0_22H2"`. The ADML
reads: "This policy setting allows you to show the longer time and date format in the system tray. If
this setting is enabled, the time format will include the AM/PM time marker and the date will include
the year. **A reboot is required for this policy setting to take effect.**"

Binary presence on 26100 in `Taskbar.View.dll` (twice) and `SettingsHandlers_DesktopTaskbar.dll`, so
the Windows 11 tray clock reads it.

**Not a duplicate.** `seconds_in_tray_clock` uses `ShowSecondsInSystemClock` under `Explorer\Advanced`:
different key, different value, no collision.

**Optional companion.** `AlwaysShowNotificationIcon` is equally real: same ADMX file, same key,
`class="User"`, `enabledValue 1`, `supportedOn="windows:SUPPORTED_Windows_11_0_22H2"`, ADML "Show
notification bell icon ... Otherwise, the notification icon will only be shown when there's a special
status (for example, Do Not Disturb is turned on). A reboot is required." Ship or skip both together.

**Corrections needed:** the copy must state the reboot requirement. Without it a user applies the
tweak, sees nothing, and reasonably concludes it is broken.

**Ready-to-paste info block:**

```yaml
    info: |
      **Shows the full date with the year and an AM/PM marker in the taskbar clock.**

      ## What it does
      Sets the `TurnOffAbbreviatedDateTimeFormat` policy to 1 under
      `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`. Per the shipped policy text, the time
      format gains the AM/PM marker and the date gains the year, replacing the abbreviated tray
      format Windows 11 uses by default.

      ## Benefits
      - **Unambiguous time**: no guessing whether 7:15 is morning or evening on a 12-hour clock
      - **Year visible**: useful on machines whose clock drifts or that dual-boot
      - **Documented policy**: defined in the shipped Taskbar ADMX with explicit enabled and disabled
        values

      ## Drawbacks
      - **Wider clock**: the tray takes noticeably more horizontal space
      - **Needs a reboot**: nothing changes until you restart, which reads as a failed apply if you
        do not know
      - **Windows 11 only**: the policy is declared from Windows 11 22H2

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (policy applies from 22H2)
      - **Takes effect**: after reboot, which the shipped policy text states explicitly
      - **Reverting**: removes the value, which is the unconfigured state

      ## Recommendation
      Apply it if you use a 12-hour clock and keep misreading the tray. On a narrow laptop taskbar
      the extra width is a real cost, so skip it there.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - Shipped `Taskbar.admx` and `en-US\Taskbar.adml` on Windows 11 build 26100
      - Binary evidence: `TurnOffAbbreviatedDateTimeFormat` in `Taskbar.View.dll` and
        `SettingsHandlers_DesktopTaskbar.dll` on build 26100
```

**Sources:**

1. Shipped `C:\Windows\PolicyDefinitions\Taskbar.admx` and `en-US\Taskbar.adml` on 26100: `class="User"`, key `Software\Policies\Microsoft\Windows\Explorer`, `valueName="TurnOffAbbreviatedDateTimeFormat"`, `enabledValue 1` / `disabledValue 0`, `supportedOn` `SUPPORTED_Windows_11_0_22H2`, and the explicit reboot sentence (tier A)
2. Binary string evidence on 26100: `TurnOffAbbreviatedDateTimeFormat` in `Taskbar.View.dll` (twice) and `SettingsHandlers_DesktopTaskbar.dll` (tier A, product artifact)

### `hide_recently_added_apps` Recently added apps in Start

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` **or**
`HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, `HideRecentlyAddedApps` (REG_DWORD). Hidden = `1`,
Stock Default = `absent`.

Shipped 26100 `StartMenu.admx`: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, no
explicit `enabledValue`, which per the ADMX schema defaults to REG_DWORD `1` enabled and `0` disabled.
The ADML reads "Remove 'Recently added' list from Start Menu ... The corresponding setting will also be
disabled in Settings."

**Applicability correction.** The ADMX says `SUPPORTED_Windows_10_0_RS4` (1803), but Policy CSP
Start > `HideRecentlyAddedApps` says "Windows 10, version 1703 [10.0.15063] and later", with editions
Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC, and both Device and User scope. Use
**1703**.

**Two additions from the CSP page.** First, "This policy requires a reboot to take effect." Second,
Microsoft's own validation steps are written against the Windows 11 Settings toggle: "In the Settings
app, enable the Show recently added apps option ... Check that the Show recently added apps Settings
toggle is grayed out." That settles the concern that the policy is a Windows 10-only surface; it is
documented against the current Start.

Binary presence on 26100 in `StartTileData.dll`, the Start app-list data model. It does not appear in
any other shell consumer, which is consistent with `StartTileData.dll` owning the recently-added list.

**Not subsumed by anything shipped.** `disable_start_recommendations` (`Start_IrisRecommendations = 0`)
suppresses the promotional rows only; recently-installed apps keep appearing. This is also the fallback
if `disable_start_recommended_section` does not land on a given edition.

**Corrections needed:** applicability is 1703 rather than 1803, and the reboot requirement must be
stated.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the "Recently added" list of newly installed apps from the Start menu.**

      ## What it does
      Sets the `HideRecentlyAddedApps` policy to 1 under
      `SOFTWARE\Policies\Microsoft\Windows\Explorer`. Per the shipped policy text this removes the
      "Recently added" list from Start and greys out the corresponding Settings toggle so it cannot
      be turned back on from the UI.

      ## Benefits
      - **Stable Start menu**: the app list stops reshuffling every time you install something
      - **Fewer surprises**: apps installed by an updater or a bundled installer do not announce
        themselves at the top of Start
      - **Documented policy**: defined in the shipped Start menu ADMX and the Start Policy CSP

      ## Drawbacks
      - **Harder to find new apps**: you launch a freshly installed program by searching instead
      - **Settings toggle locked**: the matching switch is greyed out while the policy is set, which
        can look like a fault
      - **Needs a reboot**: Microsoft documents a restart requirement for this policy

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the policy is documented from Windows 10 version
        1703, editions Pro, Enterprise, Education and IoT Enterprise
      - **Takes effect**: after reboot
      - **Reverting**: removes the value, which is the unconfigured state, and restores the Settings
        toggle

      ## Recommendation
      Apply it if you want a Start menu that stays where you put it, particularly on a machine where
      software is installed often. Skip it if you launch new programs from the Recently added list.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Start > HideRecentlyAddedApps](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start)
      - Shipped `StartMenu.admx` and `en-US\StartMenu.adml` on Windows 11 build 26100
```

**Sources:**

1. Shipped `C:\Windows\PolicyDefinitions\StartMenu.admx` and `en-US\StartMenu.adml` on 26100: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, `valueName="HideRecentlyAddedApps"`, no explicit `enabledValue` (schema default 1 / 0), `supportedOn` `SUPPORTED_Windows_10_0_RS4` (tier A)
2. Policy CSP - Start > HideRecentlyAddedApps (applicable OS from 1703, edition list, Device and User scope, reboot requirement, and validation steps written against the Windows 11 Settings toggle), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start (tier A)
3. Binary string evidence on 26100: `HideRecentlyAddedApps` in `StartTileData.dll`, the Start app-list data model (tier A, product artifact)

### `taskbar_last_active_click` Click to the last active window

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `LastActiveClick`
(REG_DWORD). Focus last active = `1`, Stock Default = `absent`.

Binary presence on 26100 in **`Taskbar.View.dll`** (twice) and **`Taskbar.dll`**. These are the
Windows 11 taskbar itself, not policy plumbing, so the value is read by the shipping taskbar on the
target build. That is the decisive evidence.

Win11Debloat `Regfiles/Enable_Last_Active_Click.reg`, fetched and decoded, writes
`"LastActiveClick"=dword:00000001` at that key, with an unusually explicit inline description matching
the claimed behaviour, including "the pop-up window display will still show if you hover your mouse
over the taskbar icon".

**Source correction.** The original proposal cited Sophia Script; the Windows 11 `Sophia.psm1` contains
no `LastActiveClick` occurrence, nor do privacy.sexy or WinUtil, and the ElevenForum tutorial could not
be retrieved through the Wayback Machine. Real support is one project plus binary presence. That is
below the three-independent-tier-C bar, but binary presence in the consumer is stronger than any number
of community sources for the question "does this value exist and get read", and the semantics are
trivially self-verifying by clicking a grouped taskbar icon. Confirmed on that basis, labelled
community-corroborated.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Clicking a grouped taskbar button jumps straight to the window you used last.**

      ## What it does
      Sets `LastActiveClick` to 1 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`. When several windows share
      one taskbar button, a click switches to the most recently active one instead of opening the
      thumbnail chooser. Hovering the icon still shows the previews.

      ## Benefits
      - **One click instead of two**: no thumbnail step for the window you almost always want
      - **Previews still available**: hover to see and pick a different window
      - **Read by the shipping taskbar**: the value is live in the Windows 11 taskbar binaries

      ## Drawbacks
      - **Wrong window sometimes**: if you wanted a different one you now have to hover and pick
      - **Undocumented**: Microsoft publishes no reference for this value
      - **Per user only**: other accounts keep the default behaviour

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: after restarting Explorer
      - **Reverting**: removes the value, which is the stock state

      ## Recommendation
      Apply it if you keep several windows of the same app open and mostly return to the last one.
      Skip it if you regularly switch between many windows of one app, where the chooser is faster.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - Binary evidence: `LastActiveClick` appears in `Taskbar.View.dll` and `Taskbar.dll`, the
        Windows 11 taskbar itself, on build 26100
      - Win11Debloat `Regfiles/Enable_Last_Active_Click.reg`
```

**Sources:**

1. Binary string evidence on 26100: `LastActiveClick` in `Taskbar.View.dll` (twice) and `Taskbar.dll`, the shipping Windows 11 taskbar rather than policy plumbing (tier A, product artifact)
2. Win11Debloat `Regfiles/Enable_Last_Active_Click.reg`, fetched and decoded, `"LastActiveClick"=dword:00000001` with an inline description of the behaviour (tier C)

### `disable_notification_center` Notification Center

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism (corrected):** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` **or**
`HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, `DisableNotificationCenter` (REG_DWORD).
Removed = `1`, Present (Stock Default) = `absent`. **Requires a reboot.**

Shipped 26100 `Taskbar.admx`: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`,
`enabledValue 1` and `disabledValue 0`, `supportedOn="windows:SUPPORTED_Windows_10_0"`. Because the
class is `Both`, HKLM is available and is the better choice for a machine-wide toolbox action; the
original proposal assumed HKCU only.

Binary presence in `Windows.UI.ActionCenter.dll`, `Taskbar.View.dll` (twice) and `twinui.pcshell.dll`,
that is, in the components that actually render the notification centre on Windows 11. This is the
strongest consumer evidence in the whole Low set and it settles the "does it still work on 26100"
question affirmatively.

**Unsupported claim to strike.** The original proposal stated it removes "the Notification Center *and*
the calendar flyout". The ADML does not say that. It says: "This policy setting removes Notifications
and Action Center from the notification area on the taskbar ... The user will be able to read
notifications when they appear, but they won't be able to review any notifications they miss." On
Windows 11 the calendar and the notification list share one flyout, so losing the calendar is
plausible, but it is an **inference**. Phrase it as "may also remove the calendar flyout, which shares
the same panel", or probe it.

**Sourcing.** WinUtil `config/tweaks.json` does contain `DisableNotificationCenter` (line 1012), so
that citation holds. Sophia Script does not contain it; privacy.sexy does not either. The ADMX is
tier A, so this does not affect the verdict.

**Not a duplicate.** `disable_toast_notifications` writes `ToastEnabled` under `PushNotifications`:
different key, different effect. The corpus previously offered nothing between "all toasts off" and
"leave it alone".

**Corrections needed:** three items. (1) Use the HKLM key, since the class is `Both`. (2) State the
reboot requirement, which the ADML gives explicitly. (3) Do not assert the calendar-flyout removal as
documented fact. Ship it only with a blunt warning in the copy: notification history is gone, not just
quieter.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Notification Center panel and its taskbar entry point entirely.**

      ## What it does
      Sets the `DisableNotificationCenter` policy to 1 under
      `SOFTWARE\Policies\Microsoft\Windows\Explorer`. Per the shipped policy text, Notifications and
      Action Center are removed from the notification area on the taskbar. Notifications can still
      appear as they arrive, but there is no panel to review the ones you missed.

      ## Benefits
      - **No history panel**: nothing accumulates in a list for anyone to scroll through later
      - **Reclaims the tray entry**: the notification button leaves the taskbar corner
      - **Documented policy**: defined in the shipped Taskbar ADMX, and available machine-wide

      ## Drawbacks
      - **Missed notifications are lost**: this is the blunt part, there is no way to review anything
        you did not catch on screen
      - **May take the calendar with it**: on Windows 11 the calendar shares the same flyout, so it
        is likely to go too, though the policy text does not say so
      - **Needs a reboot**: nothing changes until you restart
      - **Heavier than Do Not Disturb**: if you only want quiet, Focus keeps the history

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the policy is declared from Windows 10
      - **Takes effect**: after reboot
      - **Reverting**: removes the value, which is the unconfigured state
      - Written to HKLM this applies to every account on the machine and needs admin rights.

      ## Recommendation
      Apply it on a kiosk, a signage machine or a locked-down workstation where a notification
      history is unwanted. For a normal desktop, turn on Do Not Disturb instead; losing the history
      is a bigger cost than it sounds.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - Shipped `Taskbar.admx` and `en-US\Taskbar.adml` on Windows 11 build 26100
      - Binary evidence: `DisableNotificationCenter` in `Windows.UI.ActionCenter.dll`,
        `Taskbar.View.dll` and `twinui.pcshell.dll` on build 26100
```

**Sources:**

1. Shipped `C:\Windows\PolicyDefinitions\Taskbar.admx` and `en-US\Taskbar.adml` on 26100: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, `valueName="DisableNotificationCenter"`, `enabledValue 1` / `disabledValue 0`, `supportedOn` `SUPPORTED_Windows_10_0`, plus the explicit reboot statement and the "won't be able to review any notifications they miss" wording (tier A)
2. Binary string evidence on 26100: `DisableNotificationCenter` in `Windows.UI.ActionCenter.dll`, `Taskbar.View.dll` (twice) and `twinui.pcshell.dll`, the components that render the notification centre (tier A, product artifact)
3. Chris Titus WinUtil `config/tweaks.json` line 1012, containing `DisableNotificationCenter` (tier C)

### `disable_copilot_taskbar` Copilot taskbar button (moved out)

**Verdict:** MOVED to the `ai` category.

This tweak writes `TurnOffWindowsCopilot` (REG_DWORD) at
`HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot`, Hidden = `1`, Stock Default = `absent`. It
is confirmed at tier A by the `WindowsCopilot.admx` shipping on 26100.4061 and is not defective in
mechanism. It is documented in `ai.md`, together with its three outstanding corrections: the stale
Win+C claim, the missing Home SKU exclusion, and Microsoft's near-term-deprecation notice pointing
administrators to an AppLocker publisher rule for `MICROSOFT.COPILOT` or `Remove-AppxPackage` on
`Microsoft.Copilot`.

Note the cross-category relationship recorded in `_cross-category.md`: this tweak and
`debloat:remove_copilot_app` address the same feature at different depths, the policy hiding the shell
entry point and the app removal actually removing Copilot. That pairing now spans the `ai` and
`debloat` documents rather than `interface` and `debloat`.

