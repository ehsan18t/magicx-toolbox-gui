# Interface & Explorer tweaks

This category covers the look and behaviour of the desktop, the taskbar, the Start menu, File Explorer, notifications and the sign-in screen. Most tweaks here are per-user (HKCU) shell preferences that need no elevation, take effect on an Explorer restart or a sign-out, and revert cleanly from the snapshot. A handful are documented Group Policy values: four of them live under `HKCU\Software\Policies`, which the user can only read, so they need administrator rights even though they are per user, and four other tweaks write to HKLM and need administrator rights too. The supported platform is Windows 11 24H2 (build 26100) and newer, including 25H2 (26200), as the primary target, and Windows 10 IoT Enterprise LTSC 2021 (build 19044) as the secondary target; tweaks gated to Windows 11 or to a minimum build are hidden on LTSC 2021.

Two facts apply to the whole category. First, Microsoft publishes no reference for most `HKCU\...\Explorer\Advanced` value names, so a shell preference counts as verified only when several independent long-lived community references agree on key, name, type and values and the setting is exposed in Settings or Folder Options, where its effect can be observed directly; policy-backed tweaks are sourced from the ADMX templates shipped in `C:\Windows\PolicyDefinitions` on build 26100. Second, every HKCU effect runs as the signed-in user even when the app is elevated, so a per-user value always lands in the hive of the person at the keyboard. In the tables below, elevation `none` is the YAML's `elevation: user`.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Turn on dark mode](#turn-on-dark-mode) | `enable_dark_mode` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off transparency effects](#turn-off-transparency-effects) | `disable_transparency` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off window animations](#turn-off-window-animations) | `disable_ui_animations` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable Aero Shake](#disable-aero-shake) | `disable_aero_shake` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Show file name extensions](#show-file-name-extensions) | `show_file_extensions` | Switch (2 options) | low | none | no | VERIFIED |
| [Show hidden files and folders](#show-hidden-files-and-folders) | `show_hidden_files` | Switch (2 options) | low | none | no | VERIFIED |
| [Open File Explorer to This PC](#open-file-explorer-to-this-pc) | `open_explorer_to_this_pc` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Show the full path in the Explorer title](#show-the-full-path-in-the-explorer-title) | `explorer_full_path_title` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hide recent files and folders in Explorer](#hide-recent-files-and-folders-in-explorer) | `disable_recent_files` | Switch (2 options) | low | none | no | INCORRECT (corrected form ships) |
| [Turn off recent items tracking](#turn-off-recent-items-tracking) | `disable_start_recent_items` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hide the Task View button](#hide-the-task-view-button) | `disable_task_view_button` | Switch (2 options) | low | none | no | VERIFIED |
| [Show seconds in the tray clock](#show-seconds-in-the-tray-clock) | `seconds_in_tray_clock` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off search highlights](#turn-off-search-highlights) | `disable_search_highlights` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Taskbar search style (Windows 10)](#taskbar-search-style-windows-10) | `taskbar_search_mode` | Dropdown (3 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Taskbar search style (Windows 11)](#taskbar-search-style-windows-11) | `taskbar_search_mode_win11` | Dropdown (4 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Ungroup taskbar buttons](#ungroup-taskbar-buttons) | `taskbar_ungroup_labels` | Dropdown (3 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Show taskbar thumbnails instantly](#show-taskbar-thumbnails-instantly) | `taskbar_hover_time` | Switch (2 options) | low | none | no | DISPUTED |
| [Left-align the taskbar](#left-align-the-taskbar) | `taskbar_alignment_left` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off the snap layouts hover flyout](#turn-off-the-snap-layouts-hover-flyout) | `disable_snap_flyout` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn on Explorer compact view](#turn-on-explorer-compact-view) | `explorer_compact_view` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off promoted recommendations in Start](#turn-off-promoted-recommendations-in-start) | `disable_start_recommendations` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Add End task to the taskbar menu](#add-end-task-to-the-taskbar-menu) | `enable_end_task_taskbar` | Switch (2 options) | low | none | no | VERIFIED |
| [Hide Gallery in Explorer](#hide-gallery-in-explorer) | `remove_gallery_nav_pane` | Switch (2 options) | low | none | no | VERIFIED |
| [Hide Home in Explorer](#hide-home-in-explorer) | `remove_home_nav_pane` | Switch (2 options) | low | none | no | VERIFIED |
| [Hide OneDrive in Explorer](#hide-onedrive-in-explorer) | `remove_onedrive_nav_pane` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off toast notifications](#turn-off-toast-notifications) | `disable_toast_notifications` | Switch (2 options) | medium | none | no | VERIFIED |
| [Show verbose logon messages](#show-verbose-logon-messages) | `verbose_logon_messages` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Silence the startup sound](#silence-the-startup-sound) | `disable_startup_sound` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off accessibility shortcut prompts](#turn-off-accessibility-shortcut-prompts) | `disable_accessibility_key_prompts` | Switch | low | none | no | VERIFIED-WITH-CORRECTION |
| [Restore the classic context menu](#restore-the-classic-context-menu) | `classic_context_menu_win11` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn NumLock on at startup](#turn-numlock-on-at-startup) | `numlock_on_startup` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hide the Recommended section in Start](#hide-the-recommended-section-in-start) | `disable_start_recommended_section` | Switch (2 options) | low | admin | no | VERIFIED |
| [Hide the unsupported hardware notice](#hide-the-unsupported-hardware-notice) | `hide_unsupported_hardware_notice` | Switch (2 options) | low | admin | no | VERIFIED |
| [Hide the mobile device panel in Start](#hide-the-mobile-device-panel-in-start) | `disable_phone_companion_start` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off the Drop Tray share overlay](#turn-off-the-drop-tray-share-overlay) | `disable_drag_tray` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Alt+Tab shows windows only](#alttab-shows-windows-only) | `alt_tab_hide_browser_tabs` | Dropdown (5 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off the Snap Assist suggestion picker](#turn-off-the-snap-assist-suggestion-picker) | `disable_snap_assist` | Switch (2 options) | low | none | no | VERIFIED |
| [Expand the tree to the open folder](#expand-the-tree-to-the-open-folder) | `explorer_expand_to_current_folder` | Switch (2 options) | low | none | no | VERIFIED |
| [Restore Explorer windows at sign-in](#restore-explorer-windows-at-sign-in) | `explorer_restore_folders_at_logon` | Switch (2 options) | low | none | no | VERIFIED |
| [Show the full date and time in the tray](#show-the-full-date-and-time-in-the-tray) | `taskbar_full_date_time` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Hide recently added apps in Start](#hide-recently-added-apps-in-start) | `hide_recently_added_apps` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Focus the last active window on click](#focus-the-last-active-window-on-click) | `taskbar_last_active_click` | Switch (2 options) | low | none | no | VERIFIED |
| [Remove the Notification Center](#remove-the-notification-center) | `disable_notification_center` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |

## Tweaks

### Turn on dark mode

`enable_dark_mode` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Switches Windows, the shell and your apps to the dark colour theme in one step.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `apps_theme` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`, value `AppsUseLightTheme`, REG_DWORD |
| `system_theme` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`, value `SystemUsesLightTheme`, REG_DWORD |

| Option | `apps_theme` | `system_theme` |
|---|---|---|
| Dark | `0` | `0` |
| Light | `1` | `1` |

System Default is shown when the two values do not match either option, for example apps dark with the shell light (the mixed state Settings can produce) or either value missing; selecting it restores the values captured in the snapshot before the tweak was first applied. Stock Windows is Light, `1` for both.

#### How it works

These are the two values behind Settings > Personalization > Colors > "Choose your mode". `AppsUseLightTheme` drives the theme that apps and File Explorer content use; `SystemUsesLightTheme` drives the shell surfaces: taskbar, Start and Action Center. The polarity is inverted relative to the tweak's name: the values ask "use light?", so `0` means dark. Writing only one of the two produces a mixed desktop with light apps and a dark shell or the reverse, which is why the tweak always writes both together. Both are plain per-user preferences, not policy, so the Settings page stays fully usable and reflects the change. Apps that follow the system theme listen for the change and repaint; apps that read the theme only at start-up keep their old look until relaunched.

#### Benefits
- **Both surfaces together**: apps and shell flip in one action instead of two Settings toggles.
- **Easier on the eyes**: less glare in dim rooms and at night.
- **No policy, no admin**: a plain per-user preference that the Settings page continues to control.

#### Drawbacks
- **Not uniform**: some legacy dialogs, MMC snap-ins and Control Panel applets ignore the theme and stay light.
- **App restarts**: a few already-running apps only pick up the new theme when relaunched.
- **Worse in bright rooms**: dark surfaces are harder to read in direct sunlight.
- **All or nothing**: the dropdown sets both surfaces; a mixed light and dark setup reads as System Default.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 LTSC 2021 (the values exist from Windows 10 1809); every edition.
- **Takes effect**: immediately for the shell and theme-aware apps; some running apps need a restart.
- **Reverting**: restores both captured values from the snapshot. Windows updates do not reset this setting.

#### Interactions
Shares the `Themes\Personalize` key with [Turn off transparency effects](#turn-off-transparency-effects), which writes a different value (`EnableTransparency`); the two are independent.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated. Microsoft does not publish these value names; three independent references agree on key, names, type and polarity, and the effect is directly observable in Settings.
- **Reasoning**: the key, both value names, the REG_DWORD type and the inverted polarity all agree across sources, and the tweak writes both values, which avoids the mixed-theme trap. No claim was disputed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you work in a dim room or simply prefer dark surfaces. Skip it if you rely on an app that renders badly in dark mode, since the theme applies to everything.

#### Sources
1. Choose Dark or Light Mode for Colors in Windows 11, the two values and their polarity, https://www.elevenforum.com/t/choose-dark-or-light-mode-for-colors-in-windows-11.555/ (tier C)
2. Enable Dark mode on Windows, the same values on Windows 10, https://pureinfotech.com/enable-dark-mode-windows-10/ (tier C)
3. Change Between Light and Dark Mode for Default App Mode, `AppsUseLightTheme` semantics, https://www.tweaknow.com/RegTweakAppModeTheme.php (tier C)

### Turn off transparency effects

`disable_transparency` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Makes the taskbar, Start menu and flyouts solid instead of translucent.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `transparency` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`, value `EnableTransparency`, REG_DWORD |

| Option | `transparency` |
|---|---|
| Off | `0` |
| On | `1` |

System Default is shown when the value is missing or holds anything other than 0 or 1; selecting it restores the snapshot. Stock Windows 10 and 11 ship transparency on (`1`).

#### How it works

`EnableTransparency` is the Settings > Personalization > Colors > "Transparency effects" toggle. With it at `0` the Desktop Window Manager stops rendering the acrylic and Mica materials on the taskbar, Start, the Settings app and flyouts, which then draw as flat, opaque surfaces. It is a per-user preference read live by the shell; no restart is needed. Windows can also suppress transparency on its own, for example in battery saver or when reduced visual effects are on, independently of this value.

#### Benefits
- **Cleaner contrast**: taskbar and Start text stops competing with the wallpaper behind it.
- **Less compositing**: a small reduction in DWM work on very weak integrated graphics.
- **Consistent look**: surfaces no longer shift in tone as windows and wallpaper move behind them.

#### Drawbacks
- **Aesthetic loss**: the Fluent design look goes, with nothing gained visually.
- **Not a performance tweak**: on any modern GPU the compositing saving is noise.
- **State can drift**: battery saver and reduced-effects modes disable transparency by themselves, so what you see may not match the written value.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 LTSC 2021; every edition.
- **Takes effect**: immediately, within a second or two.
- **Reverting**: restores the captured value from the snapshot.

#### Interactions
Shares the `Themes\Personalize` key with [Turn on dark mode](#turn-on-dark-mode); the values are independent and the two are orthogonal choices.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated. The value name is unpublished by Microsoft but confirmed by several references and observable in Settings.
- **Reasoning**: key, name, type and polarity agree across sources; the only nuance (automatic suppression in battery saver) affects what the user sees, not whether the value works.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if the blur makes taskbar and Start text hard to read against your wallpaper. Do not apply it expecting a frame-rate gain; that is not what it does.

#### Sources
1. Enable or Disable Transparency in Windows 11, the value and its Settings mapping, https://winaero.com/how-to-enable-or-disable-transparency-in-windows-11/ (tier C)
2. Enable or Disable Transparency Effects in Windows 11, the same value and polarity, https://www.ninjaone.com/blog/enable-or-disable-transparency-effects-in-windows-11/ (tier C)

### Turn off window animations

`disable_ui_animations` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no (the window half needs a sign-out) · Windows: all supported builds · Reversible: yes

**Removes the slide and fade animations when windows open, close, minimise and maximise.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `taskbar_anim` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `TaskbarAnimations`, REG_DWORD |
| `min_animate` | registry | `HKCU\Control Panel\Desktop\WindowMetrics`, value `MinAnimate`, REG_SZ |

| Option | `taskbar_anim` | `min_animate` |
|---|---|---|
| Off | `0` | `"0"` |
| On | `1` | `"1"` |

System Default is shown when the two values disagree (for example taskbar animation off but window animation on) or either is missing; selecting it restores the snapshot. Both are enabled on a stock machine; whether a fresh profile stores `TaskbarAnimations` as `1` or leaves it absent is not settled, so an untouched machine may read System Default with animations on.

#### How it works

`TaskbarAnimations` controls taskbar and Start animation, including the slide and fade of taskbar thumbnail previews; Explorer reads it when it starts. `MinAnimate` is the long-standing "Animate windows when minimizing and maximizing" performance option and is stored as a string: written as a DWORD it is ignored, so the REG_SZ type is load-bearing. Windows reads `MinAnimate` at session start or when a program broadcasts the change through `SystemParametersInfo`; a plain registry write is not broadcast, so the window half waits for the next sign-in. The tweak deliberately does not touch `UserPreferencesMask` under `Control Panel\Desktop`, a packed bitmask of many visual preferences that a blind rewrite would corrupt.

#### Benefits
- **Snappier feel**: a short but real delay disappears from every window state change.
- **Helps weak hardware**: most noticeable on low-end GPUs and in remote desktop sessions.
- **Targeted**: two values only, so unrelated visual preferences stay as they are.

#### Drawbacks
- **Lost motion cues**: harder to see where a window went when it minimised.
- **Split timing**: the taskbar half applies on an Explorer restart; the window half only after you sign out and back in.
- **Overwritten by Settings**: Accessibility > Visual effects > "Animation effects" rewrites both values (and others), so flipping that toggle later undoes this.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 LTSC 2021; every edition.
- **Takes effect**: taskbar animation after Explorer restarts; minimise and maximise animation after sign-out. The tweak does not set `requires_reboot`, so the app does not prompt for the sign-out.
- **Reverting**: restores both captured values from the snapshot; the window half again needs a sign-out to show.

#### Interactions
`performance:optimize_visual_effects` applies the rest of the "Adjust for best performance" profile (`VisualFXSetting`, `UserPreferencesMask`, `ListviewAlphaSelect`, `ListviewShadow`) and deliberately leaves `MinAnimate` and `TaskbarAnimations` to this tweak, which is their single owner in the corpus; apply both for the full profile. The Settings "Animation effects" toggle overwrites these values.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: `MinAnimate` is read at session start, so it needs a sign-out rather than an Explorer restart.
- **Confidence**: Community-corroborated, three references on the two values, their types and defaults.
- **Reasoning**: key paths, names, the REG_SZ typing of `MinAnimate` and the defaults held up. The attacked claim was the timing, and it was corrected: `TaskbarAnimations` does apply on an Explorer restart, `MinAnimate` does not. Avoiding `UserPreferencesMask` was judged the right call.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth it on low-end hardware or over remote desktop, where the animation delay is real. Skip it if you use the motion to track where windows went.

#### Sources
1. Enable or Disable Animate Windows when Minimizing and Maximizing, `MinAnimate` as REG_SZ, https://www.tenforums.com/tutorials/126788-enable-disable-animate-windows-when-minimizing-maximizing.html (tier C)
2. Disable Animate Windows when Minimizing and Maximizing in Windows 10, the same value and default, https://winaero.com/disable-animate-windows-when-minimizing-and-maximizing-in-windows-10/ (tier C)
3. Enable or Disable Animations in the Taskbar in Windows 10, `TaskbarAnimations`, https://www.tenforums.com/tutorials/126795-enable-disable-animations-taskbar-windows-10-a.html (tier C)

### Disable Aero Shake

`disable_aero_shake` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops windows from minimising when you grab a title bar and shake it.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `shaking` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `DisallowShaking`, REG_DWORD |

| Option | `shaking` |
|---|---|
| Disabled | `1` |
| Enabled | `0` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Windows 11 ships the gesture off, and from Windows 10 build 21277 it is off by default too; whether that stock state is stored as `DisallowShaking = 1` or as the value being absent is not established, so on a stock machine the tweak may read System Default while behaving exactly like Disabled. Note the polarity: "Enabled" (`0`) turns the gesture on, which is an opt-in on every supported build, not a restore.

#### How it works

`DisallowShaking` is the preference behind Settings > System > Multitasking > "Title bar window shake". At `1` Explorer ignores a rapid shake of a window's title bar; at `0` a shake minimises every other window. A policy twin exists, `NoWindowMinimizingShortcuts` under `HKCU\Software\Policies\Microsoft\Windows\Explorer` (Group Policy "Turn off Aero Shake window minimizing mouse gesture"); the tweak uses the preference instead, which keeps the Settings toggle usable and the change easy to reverse.

#### Benefits
- **No accidental minimise**: dragging a window quickly no longer hides everything else.
- **Predictable dragging**: useful if you reposition windows often or use a sensitive trackpad.
- **Simple revert**: one per-user value, no policy involved.

#### Drawbacks
- **Often no visible change**: Windows 11 already ships the gesture off, so applying Disabled may change nothing you can see.
- **Preference, not policy**: the Settings toggle stays available and can turn the gesture back on.
- **All or nothing**: there is no partial setting.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition. On Windows 10 LTSC 2021 (build 19044, above 21277) the gesture is also off by default.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured value, including removing it if it was absent before.

#### Interactions
None known. No other tweak writes `DisallowShaking` or the `NoWindowMinimizingShortcuts` policy.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the stock state is gesture off, so `0` is an opt-in that turns the gesture on, not the Windows default.
- **Confidence**: Community-corroborated, three references agreeing on key, name and polarity; the default-off change is widely documented and visible in Settings.
- **Reasoning**: the mechanism and polarity survived; the attacked point was which option represents stock, and it was corrected. Open question: whether stock is `1` present or value absent, which only a clean-profile dump can settle.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine where the gesture has been turned on and keeps firing by accident. If you deliberately use shake-to-minimise, choose Enabled instead.

#### Sources
1. Enable or Disable Title Bar Window Shake in Windows 11, the value, polarity and default-off state, https://www.elevenforum.com/t/enable-or-disable-title-bar-window-shake-in-windows-11.2078/ (tier C)
2. Enable Minimize Windows with Title Bar Shake in Windows 11 (Aero Shake), the same value, https://winaero.com/enable-minimize-windows-with-title-bar-shake-in-windows-11-aero-shake/ (tier C)
3. How to Enable or Disable Aero Shake in Windows 10, the value and the policy twin, https://www.tenforums.com/tutorials/4417-how-enable-disable-aero-shake-windows-10-a.html (tier C)

### Show file name extensions

`show_file_extensions` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Shows the real file extension on every file, so a disguised executable cannot hide.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hide_ext` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `HideFileExt`, REG_DWORD |

| Option | `hide_ext` |
|---|---|
| Show | `0` |
| Hide | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Stock Windows hides extensions (`1`): Windows itself declares that default under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\HideFileExt` with `DefaultValue = 1`.

#### How it works

`HideFileExt` is the Folder Options > View > "Hide extensions for known file types" checkbox. With it at `0` Explorer shows the extension of every registered file type (`.exe`, `.js`, `.scr` and the rest) in lists, dialogs and the desktop. The `Explorer\Advanced\Folder` declaration is what Folder Options itself uses to map the checkbox to this value, so the written literals are exactly what the checkbox writes.

#### Benefits
- **Anti-phishing**: `invoice.pdf.exe` stops rendering as `invoice.pdf` with a PDF icon.
- **Cheapest hardening**: one per-user value, no admin rights, no side effects.
- **Clearer file types**: a `.txt` and a `.md` are distinguishable at a glance.

#### Drawbacks
- **Noisier lists**: every file name is longer.
- **Rename hazard**: renaming a file can now strip or alter its extension by accident (Explorer does warn).
- **Per user only**: other accounts on the machine are unaffected.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after Explorer restarts, or press F5 in an open window.
- **Reverting**: restores the captured value. Windows updates do not reset this.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated for the value name, with the default backed by a shipped-OS declaration (tier A product artifact).
- **Reasoning**: key, name, type and polarity agree across sources, and the stock `1` is not merely community-reported: Windows declares it under `Explorer\Advanced\Folder`.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine you administer. The only reason to skip it is a shared family PC where someone is likely to rename files and lose the extension.

#### Sources
1. Show or Hide File Name Extensions for Known File Types in Windows 11, the value and the Folder Options mapping, https://www.elevenforum.com/t/show-or-hide-file-name-extensions-for-known-file-types-in-windows-11.898/ (tier C)
2. `HKEY_CURRENT_USER\...\Explorer\Advanced` registry reference, value names and defaults, https://renenyffenegger.ch/notes/Windows/registry/tree/HKEY_CURRENT_USER/Software/Microsoft/Windows/CurrentVersion/Explorer/Advanced/index (tier C)
3. Shipped OS state on build 26100.4061: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\HideFileExt` declares `DefaultValue = 1` (tier A, product artifact)

### Show hidden files and folders

`show_hidden_files` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Reveals hidden files and folders in File Explorer, without exposing protected system files.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hidden` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `Hidden`, REG_DWORD |

| Option | `hidden` |
|---|---|
| Show | `1` |
| Hide | `2` |

System Default is shown when the value is missing or holds anything other than 1 or 2 (including `0`, which is not a defined state); selecting it restores the snapshot. Stock Windows hides them (`2`), declared by Windows under `HKLM\...\Explorer\Advanced\Folder\Hidden` with `DefaultValue = 2`.

#### How it works

`Hidden` backs the Folder Options > View radio pair: `1` is "Show hidden files, folders, and drives" and `2` is "Don't show hidden files, folders, and drives". The enum is genuinely 1 and 2, a historical quirk of the radio group; `0` is not documented and does not reliably mean hide. Protected operating-system files are controlled separately by `ShowSuperHidden` (REG_DWORD, default 0) in the same key, the "Hide protected operating system files (Recommended)" checkbox. The tweak leaves `ShowSuperHidden` alone, so items that carry both the Hidden and System attributes, such as `pagefile.sys`, `hiberfil.sys` and `System Volume Information`, stay invisible.

#### Benefits
- **Reach app data**: `%AppData%`, `ProgramData` and dotfile configuration folders become browsable.
- **Safer than "show everything"**: protected system files remain hidden.
- **Troubleshooting**: hidden caches and leftover installer folders become visible.

#### Drawbacks
- **Busier folders**: hidden metadata such as `desktop.ini` clutters listings.
- **Deletion risk**: someone may delete an application's hidden data folder as clutter.
- **EDR noise**: several malware families flip this value and a public Sigma detection rule watches it, so on a monitored endpoint the write may raise an alert.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after Explorer restarts, or press F5 in an open window.
- **Reverting**: restores the captured value.

#### Interactions
None known. No tweak writes `ShowSuperHidden`.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, with the default backed by a shipped-OS declaration.
- **Reasoning**: the unusual 1/2 enum is confirmed by references and by Windows' own `DefaultValue = 2`; the separation from `ShowSuperHidden` is confirmed and is why the tweak is safe.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you edit configuration files or troubleshoot apps. Leave it off on a machine used by someone who might delete a hidden folder they do not recognise, or on a monitored corporate endpoint where the change may alert.

#### Sources
1. `HKEY_CURRENT_USER\...\Explorer\Advanced` registry reference, `Hidden` default 2 and `ShowSuperHidden` default 0, https://renenyffenegger.ch/notes/Windows/registry/tree/HKEY_CURRENT_USER/Software/Microsoft/Windows/CurrentVersion/Explorer/Advanced/index (tier C)
2. What does the SuperHidden Registry value control?, the independence of `ShowSuperHidden`, https://fleexlab.blogspot.com/2017/08/what-does-superhidden-registry-value.html (tier C)
3. Sigma rule: Displaying Hidden Files Feature Disabled, the detection that watches this value, https://detection.fyi/sigmahq/sigma/windows/registry/registry_set/registry_set_hide_file/ (tier C)
4. Shipped OS state on build 26100.4061: `HKLM\...\Explorer\Advanced\Folder\Hidden` declares `DefaultValue = 2` (tier A, product artifact)

### Open File Explorer to This PC

`open_explorer_to_this_pc` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**File Explorer opens on your drives instead of a feed of recent files.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `launch_to` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `LaunchTo`, REG_DWORD |

| Option | `launch_to` |
|---|---|
| This PC | `1` |
| Home / Quick Access | `2` |

System Default is shown when the value is missing or holds another number (some sources mention `0` and `3`); selecting it restores the snapshot. The stock landing page is value `2`: Home on Windows 11, Quick Access on Windows 10.

#### How it works

`LaunchTo` is the Folder Options > General > "Open File Explorer to" dropdown, read each time a new Explorer window opens without a target path (the taskbar icon, Win+E). `1` opens This PC, the drive and device view. `2` opens the default page, which Windows 11 22H2 and later calls Home; Quick Access survives as the pinned and recent section inside Home, and Windows 11 does not use a separate number for Home. Some references mention a value `3` (variously the Downloads folder or the personal OneDrive folder) and `0`; they disagree with each other, so neither is offered.

#### Benefits
- **Straight to drives**: no extra click to reach C:, D: or a mapped network drive.
- **Shoulder-surfing**: recent file names are not on screen the moment Explorer opens.
- **Stable landing**: This PC always looks the same, unlike a feed that reorders itself.

#### Drawbacks
- **Loses pins**: pinned folders live in Home and are one click further away.
- **Slower for recents workflows**: if you reopen files from the recent list, this costs a click.
- **Ordering matters** with hiding Home; see Interactions.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: for new Explorer windows after Explorer restarts.
- **Reverting**: restores the captured value.

#### Interactions
[Hide Home in Explorer](#hide-home-in-explorer) removes the Home node. Apply this tweak (This PC) first, or together with it, otherwise Explorer is configured to open a node you have removed from the pane.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: on Windows 11 value 2 is called Home, not Quick Access; the number itself was always right.
- **Confidence**: Community-corroborated, two references giving 1 = This PC and 2 = Home as default, plus the visible Folder Options dropdown.
- **Reasoning**: the enum and default survived; only the naming changed. Open question: the meaning of `3` and `0`, which the shipped options avoid.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you navigate by folder tree or drives. Skip it if you work mainly from pinned folders and recent files, since Home is where both live.

#### Sources
1. How to change startup page on File Explorer for Windows 11, `1` = This PC, `2` = Home as default, https://pureinfotech.com/open-file-explorer-this-pc-instead-quick-access-windows-11/ (tier C)
2. Open File Explorer To This PC By Default (Windows 10 and 11), the same value on both systems, https://memstechtips.com/set-file-explorer-launch-this-pc-regedit/ (tier C)

### Show the full path in the Explorer title

`explorer_full_path_title` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Puts the full folder path in the Explorer window title instead of just the folder name.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `full_path` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\CabinetState`, value `FullPath`, REG_DWORD |

| Option | `full_path` |
|---|---|
| On | `1` |
| Off | `0` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Stock is Off (`0`).

#### How it works

`FullPath` under `CabinetState` makes Explorer use the fully qualified path, such as `C:\Users\You\Projects\build`, as the window title instead of the leaf folder name `build`. It does not change the address bar, which is always a breadcrumb control and already shows the path segments. On Windows 11 the Explorer title bar is the tab strip, so the visible effect is mostly in the taskbar hover tooltip, the Alt+Tab label and any tool that reads window titles; on Windows 10 the full path shows directly in the title bar.

#### Benefits
- **Disambiguates windows**: several folders named `src` stop looking identical in Alt+Tab.
- **Better taskbar hover**: the tooltip shows where each window actually is.
- **Window tooling**: scripts and window managers that read titles get the full path.

#### Drawbacks
- **Small payoff on Windows 11**: the tabbed title bar hides most of it.
- **Truncation**: long paths are cut off in narrow tooltips.
- **No address bar change**: despite many guides, the breadcrumb is unaffected.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition; most visible on Windows 10 LTSC 2021.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the value changes the window title only, not the address bar.
- **Confidence**: Community-corroborated, two references on key, name, type and default.
- **Reasoning**: the mechanism is confirmed; the widespread "title bar and address bar" claim was attacked and failed for the address bar on every build.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you juggle several similarly named folders. On Windows 11 alone the gain is small enough to skip.

#### Sources
1. Turn On or Off Display Full Path in Title Bar of File Explorer in Windows 11, the value and its Windows 11 behaviour, https://www.elevenforum.com/t/turn-on-or-off-display-full-path-in-title-bar-of-file-explorer-in-windows-11.3585/ (tier C)
2. Display Full Path in Title Bar of File Explorer in Windows 10, the same value on Windows 10, https://www.tenforums.com/tutorials/3430-display-full-path-title-bar-file-explorer-windows-10-a.html (tier C)

### Hide recent files and folders in Explorer

`disable_recent_files` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Empties the recent files and frequent folders lists from the File Explorer Home page.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `show_recent` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer`, value `ShowRecent`, REG_DWORD |
| `show_frequent` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer`, value `ShowFrequent`, REG_DWORD |

| Option | `show_recent` | `show_frequent` |
|---|---|---|
| Off | `0` | `0` |
| On | `1` | `1` |

System Default is shown when the values disagree or either is missing; selecting it restores the snapshot. Stock is both on (`1`).

#### How it works

These two values back the File Explorer Options > General > Privacy checkboxes "Show recently used files" and "Show frequently used folders". They live under the main `Explorer` key, not under `Explorer\Advanced`: written under `Advanced` they are inert values Explorer never reads. With both at `0` the Home page (Quick Access on Windows 10) stops listing recent files and frequent folders; pinned folders stay. This hides the lists; it does not delete `%AppData%\Microsoft\Windows\Recent`, stop Windows recording activity, or stop apps keeping their own most-recently-used lists. One community source reports that on Windows 11 builds from 22635.3930 onward turning `ShowRecent` off clears the stored Recent list rather than only hiding it; if that holds on retail 24H2 and 25H2, entries recorded before the apply do not come back when you revert.

#### Benefits
- **Shoulder-surfing**: file and project names leave the screen on a shared or projected display.
- **Quieter Home page**: no auto-updating feed when a window opens.
- **Two checkboxes in one**: covers both lists.

#### Drawbacks
- **Empty Home page**: without pinned folders, Home shows almost nothing.
- **Slower reopening**: the fastest route back to a file you just closed is gone.
- **Not a wipe**: recording continues elsewhere; see [Turn off recent items tracking](#turn-off-recent-items-tracking) for that.
- **Possibly lossy revert**: on recent builds the existing Recent list may be cleared, not hidden.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores both captured values; see the lossy-revert caveat above.

#### Interactions
[Turn off recent items tracking](#turn-off-recent-items-tracking) (`Start_TrackDocs`) gates the shared recent-activity store behind this list; with both applied, reverting only one leaves Explorer Recent empty, which looks like a failed revert. `privacy:disable_explorer_cloud_recommendations` stops the separate cloud (Microsoft Graph) file suggestions that Explorer shows in Recent and Recommended views, which this tweak does not affect. [Hide Home in Explorer](#hide-home-in-explorer) removes the page these lists appear on.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The correction: both values live under the main `Explorer` key, not `Explorer\Advanced`, where Explorer never reads them; the value names, types, enum and defaults needed no change.
- **Confidence**: Community-corroborated, two references giving the key without `Advanced`, one of them stating it explicitly.
- **Reasoning**: the key path was the attacked claim and the fix is unambiguous. Open question: whether the 22635.3930 clearing behaviour applies to retail 24H2 and 25H2, which decides how restorative the revert is.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine whose screen other people see, such as a shared desk or a machine you present from. Skip it if you navigate primarily by recent files.

#### Sources
1. Explorer Group Policy: Disable Recent Files / Frequent Folders, a Group Policy Preferences recipe giving the key without `Advanced`, https://matthewhill.uk/windows/group-policy-disable-recent-files-frequent-folder-explorer/ (tier C)
2. How to Remove Recent Files in File Explorer, Windows 11, states the values are in the main `Explorer` key, https://www.ninjaone.com/blog/how-to-remove-recent-files-in-file-explorer/ (tier C)

### Turn off recent items tracking

`disable_start_recent_items` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows recording what you opened, across Start, jump lists and File Explorer at once.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `track_docs` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `Start_TrackDocs`, REG_DWORD |

| Option | `track_docs` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is shown when the value is present with anything other than `0`, in practice `1`, which Settings writes once its toggle has been moved; selecting it restores the snapshot. On a stock 26100 install the value does not exist, and the shell treats absence as tracking on, which is why the On option deletes the value.

#### How it works

`Start_TrackDocs` is the Settings > Personalization > Start toggle "Show recently opened items in Start, Jump Lists, and File Explorer". It gates the shared recent-activity store, so at `0` the Start Recommended file list, the Recent section of taskbar and Start jump lists, and File Explorer's Recent files all go quiet together. The value name is present in `shell32.dll`, `StartTileData.dll`, `Windows.Internal.Shell.Broker.dll`, `gpprefcl.dll` and `AssignedAccessManager.dll` on build 26100. The shipped default user profile (`C:\Users\Default\NTUSER.DAT`) carries only `Start_SearchFiles` under `Explorer\Advanced`, and `Start_TrackDocs` has no `DefaultValue` declaration under `Explorer\Advanced\Folder`, so the value is materialised only when the user moves the toggle. `1` and absent behave identically; writing `absent` for On returns an untouched profile to a pristine registry.

#### Benefits
- **One switch, three surfaces**: Start Recommended files, jump lists and Explorer Recent.
- **Privacy on shared screens**: document names stop appearing where anyone can read them.
- **Settings-backed**: a supported toggle, not an undocumented hack.

#### Drawbacks
- **Jump lists lose Recent**: right-clicking a taskbar icon no longer lists your documents.
- **Overlaps another tweak**: with [Hide recent files and folders in Explorer](#hide-recent-files-and-folders-in-explorer) also applied, reverting only one leaves Explorer Recent empty.
- **No history recovery**: existing entries are hidden, not archived.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured state; on an untouched machine that means removing the value rather than writing `1`.

#### Interactions
Overlaps [Hide recent files and folders in Explorer](#hide-recent-files-and-folders-in-explorer) on the Explorer surface, and [Hide the Recommended section in Start](#hide-the-recommended-section-in-start) on the Start surface (that one removes the whole section, including these recent files). Also related: [Turn off promoted recommendations in Start](#turn-off-promoted-recommendations-in-start) and [Hide recently added apps in Start](#hide-recently-added-apps-in-start), which clear other rows of the same Start region.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the stock state is value-absent on 26100, not a written `1`.
- **Confidence**: Community-corroborated for the behaviour; key, name and type confirmed at tier A by binary presence and by the shipped default profile.
- **Reasoning**: the attacked claim was the stock representation, settled by inspecting the default user profile and a live profile on 26100.4061. Behaviour is identical either way, so the correction is about a pristine revert, not a behavioural defect.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if the shell surfacing your documents bothers you or the machine is shared. Skip it if you use jump lists to reopen work, which is a genuinely efficient habit.

#### Sources
1. Shipped default user profile and live profile on Windows 11 24H2 build 26100.4061: `Start_TrackDocs` absent from both, no `DefaultValue` declaration under `Explorer\Advanced\Folder`, value name present in `shell32.dll` and `StartTileData.dll` (tier A, shipped OS state)
2. Enable or Disable Recommended Files in Start, Recent Files in File Explorer, and items in Jump Lists in Windows 11, the value and its three surfaces, https://www.elevenforum.com/t/enable-or-disable-recommended-files-in-start-recent-files-in-file-explorer-and-items-in-jump-lists-in-windows-11.1161/ (tier C)
3. Recently Opened Files: How To Hide or Show Them In Jump Lists, File Explorer, and Start Menu, the same value, https://www.majorgeeks.com/content/page/recently_opened_files_how_to_hide_or_show_them_in_jump_listsfile_explorerand_start_menu.html (tier C)

### Hide the Task View button

`disable_task_view_button` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Removes the Task View icon from the taskbar without disabling virtual desktops.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `task_view` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `ShowTaskViewButton`, REG_DWORD |

| Option | `task_view` |
|---|---|
| Hidden | `0` |
| Shown | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Stock is Shown on Windows 10 and 11; whether a fresh profile stores it as `1` or leaves it absent is not settled (it was present on the inspected live 26100 profile), so a never-touched profile may read System Default while the button is visible.

#### How it works

`ShowTaskViewButton` is the Settings > Personalization > Taskbar > "Task view" toggle on Windows 11 and the taskbar context-menu item "Show Task View button" on Windows 10. The taskbar reads it and adds or removes the button. Only the button goes: Win+Tab, virtual desktops, Win+Ctrl+Arrow switching and the Task View surface itself are untouched.

#### Benefits
- **One less icon**: reclaims taskbar space next to Start and search.
- **Nothing disabled**: virtual desktops keep working exactly as before.
- **Fewer misclicks**: the button sits next to Start, a common mis-hit.

#### Drawbacks
- **Keyboard only**: Win+Tab becomes the only route into Task View.
- **Awkward on touch**: tablet and touch users lose their tap target.
- **Per user only**: other accounts keep their button.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: immediately on the Windows 11 taskbar; after Explorer restarts on Windows 10.
- **Reverting**: restores the captured value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, two references plus the visible Settings toggle.
- **Reasoning**: key, name, type and polarity agree. Open question: whether stock is a written `1` or value-absent; no source contradicts the literal, so it stays pending a clean-profile dump.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you switch desktops from the keyboard or do not use them. Leave it if you reach Task View with the mouse or on a touch device.

#### Sources
1. Add or Remove Task View Button on Taskbar in Windows 11, the value and Settings mapping, https://www.elevenforum.com/t/add-or-remove-task-view-button-on-taskbar-in-windows-11.1037/ (tier C)
2. Add or Remove the Task View Button on the Taskbar in Windows 11, the same value, https://www.ninjaone.com/blog/add-or-remove-the-task-view-button-on-the-taskbar-in-windows-11/ (tier C)

### Show seconds in the tray clock

`seconds_in_tray_clock` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**The taskbar clock ticks in seconds instead of only showing hours and minutes.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `clock_seconds` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `ShowSecondsInSystemClock`, REG_DWORD |

| Option | `clock_seconds` |
|---|---|
| Shown | `1` |
| Hidden | `0` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Windows ships seconds off.

#### How it works

The taskbar clock reads `ShowSecondsInSystemClock` and, at `1`, renders and repaints the time every second instead of every minute. On Windows 11 the value is honoured from 22H2 build 22621.1344 (the "Moment 2" update), where it is also exposed as Settings > Personalization > Taskbar > Taskbar behaviours > "Show seconds in system tray clock"; the redesigned taskbar of earlier Windows 11 builds ignored it. On Windows 10 it has been honoured since 1607, so it works on the LTSC 2021 secondary target too. The per-second repaint is the cost Microsoft's own Settings text warns about: it uses more power.

#### Benefits
- **Seconds at a glance**: no clock app needed for short timings.
- **Log correlation**: handy when matching events to a wall clock.
- **Settings-backed**: a supported toggle on current Windows 11.

#### Drawbacks
- **Extra power use**: a repaint every second, which Microsoft's Settings text calls out.
- **Wider clock**: the tray takes slightly more room.
- **Visual noise**: a constantly changing number in the corner of the screen.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition (Windows 11 from 22621.1344, satisfied by the 26100 floor; Windows 10 from 1607, satisfied by LTSC 2021). The tweak carries no `windows:` gate because both targets honour the value.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured value.

#### Interactions
[Show the full date and time in the tray](#show-the-full-date-and-time-in-the-tray) changes the same clock through a different, policy value; the two combine.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the value's build applicability (22621.1344 on Windows 11, 1607 on Windows 10) had to be recorded; both supported targets satisfy it.
- **Confidence**: Community-corroborated, two references plus the Settings toggle.
- **Reasoning**: key, name, type and enum held up. The attacked point was applicability on the secondary target, where the value does work.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a desktop where you time things or want a precise clock. Skip it on a laptop you run on battery.

#### Sources
1. Turn On or Off Show Seconds in System Tray Clock in Windows 11, the value, Settings mapping and build requirement, https://www.elevenforum.com/t/turn-on-or-off-show-seconds-in-system-tray-clock-in-windows-11.10591/ (tier C)
2. How to Enable Seconds for the Taskbar Clock in Windows 11, the same value, https://winaero.com/how-to-enable-seconds-for-the-taskbar-clock-in-windows-11/ (tier C)

### Turn off search highlights

`disable_search_highlights` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Removes the rotating artwork and trending topics from the Windows search box.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `search_highlights` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings`, value `IsDynamicSearchBoxEnabled`, REG_DWORD |

| Option | `search_highlights` |
|---|---|
| Off | `0` |
| On | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Highlights are on by default; whether a fresh profile stores `1` or leaves the value absent could not be sourced, so an untouched machine may read System Default with highlights showing.

#### How it works

`IsDynamicSearchBoxEnabled` is the per-user "Search highlights" toggle: Settings > Privacy & security > Search permissions > More settings on Windows 11, and the taskbar search context menu on Windows 10. At `0` the illustrations, seasonal artwork and Microsoft-curated trending items disappear from the search box and the search flyout, and that content is no longer fetched for display. It is a preference, not a policy, so a Settings change or a feature update can put it back. The durable, machine-wide control is the Group Policy "Allow search highlights", which writes `EnableDynamicContentInWSB = 0` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`; this tweak deliberately does not write it, so it stays a no-admin per-user tweak.

#### Benefits
- **No promoted content**: the system search box stops carrying editorial material.
- **Quieter flyout**: opening search shows your apps and files, not a daily graphic.
- **Less background fetching**: the highlight content stops being downloaded for display.

#### Drawbacks
- **Plainer search panel**: some people like the daily artwork.
- **Preference, not policy**: a feature update or a Settings change can restore highlights.
- **Local search unchanged**: this does not affect how well search finds your files.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition (the value exists from Windows 10 21H2).
- **Takes effect**: after Explorer restarts, or the next time the search flyout opens.
- **Reverting**: restores the captured value.

#### Interactions
`debloat:disable_web_search_start` removes Bing web results from Start search, a separate surface; both are needed for a fully local, promotion-free search. No tweak in the corpus writes `EnableDynamicContentInWSB`.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: the value is a preference only, with the durable policy companion `EnableDynamicContentInWSB` documented rather than written; and the stock "On" representation (`1` versus absent) is unsourced.
- **Confidence**: Community-corroborated, three references on key, name and polarity; the policy companion is documented from the ADMX by a tier C source.
- **Reasoning**: key path and value name are correct. The open question is only the fresh-install state, which decides whether On should write `1` or delete the value.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it. There is no functional cost, and a search box is a poor place for promoted content. Skip it only if you enjoy the illustrations.

#### Sources
1. Disable Windows search highlights, documents the ADMX policy and `EnableDynamicContentInWSB` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`, https://4sysops.com/archives/turn-off-windows-search-enhancements/ (tier C)
2. Enable or Disable Search Highlights in Windows 11, the per-user value, https://www.elevenforum.com/t/enable-or-disable-search-highlights-in-windows-11.5735/ (tier C)
3. Enable or Disable Search Highlights in Windows 10, the same value on Windows 10, https://www.tenforums.com/tutorials/194711-enable-disable-search-highlights-windows-10-a.html (tier C)

### Taskbar search style (Windows 10)

`taskbar_search_mode` · Dropdown (3 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 10 only (`products: [10]`) · Reversible: yes

**Shrinks or hides the Windows 10 taskbar search box.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `search_mode` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`, value `SearchboxTaskbarMode`, REG_DWORD |
| `search_mode_cache` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`, value `SearchboxTaskbarModeCache`, REG_DWORD |

| Option | `search_mode` | `search_mode_cache` |
|---|---|---|
| Hidden | `0` | `1` |
| Icon only | `1` | `1` |
| Search box | `2` | `absent` |

System Default is shown when the pair matches no row; selecting it restores the snapshot. The stock state is the search box (`2`), which the "Search box" option reproduces.

#### How it works

`SearchboxTaskbarMode` is the value behind the taskbar's Search menu on Windows 10: `0` hides search, `1` shows the search icon, `2` shows the full search box. Windows 10 has no icon-and-label mode (`3`), which is why the Windows 11 variant is a separate tweak, [Taskbar search style (Windows 11)](#taskbar-search-style-windows-11). The two write the same two values, and their Windows gates never overlap, so only one is ever available. `SearchboxTaskbarModeCache` is the Windows 11 re-migration guard (see the Windows 11 entry); it is written here with the same shape so behaviour matches the single tweak this was split from, and nothing establishes that Windows 10 reads it. Community guides say an Explorer restart may be needed on Windows 10 for the taskbar to pick up the change. Hiding the box does not disable search: pressing the Windows key and typing still searches.

#### Benefits
- **Reclaims space**: the search box is the widest default taskbar element.
- **Keyboard search unaffected**: Windows key and type still works in every mode.

#### Drawbacks
- **Keyboard only when hidden**: mode 0 leaves no mouse entry point to search.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10, including LTSC 2021, every edition. Windows 11 does not list it (debug and test builds show it as unavailable); the Windows 11 variant takes its place.
- **Takes effect**: usually at once; otherwise after an Explorer restart or a sign-out.
- **Reverting**: restores both captured values, which on a stock machine is mode `2` with no cache value.

#### Interactions
- [Taskbar search style (Windows 11)](#taskbar-search-style-windows-11) owns the same two values on Windows 11.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections (made for the original single tweak): "Search box" (`2`) is the stock state, and `SearchboxTaskbarModeCache` is written alongside non-stock modes. The tweak was split by Windows version because mode `3` does not exist on Windows 10.
- **Confidence**: Community-corroborated: independent guides give the 0, 1 and 2 values for Windows 10.
- **Reasoning**: key, name, type and the 0 to 2 enum held up. Open question: whether Windows 10 reads `SearchboxTaskbarModeCache`.
- **Tested**: Build validation (schema, ownership and conflict checks, including the rule that lets two tweaks share a value only when their Windows gates never overlap).

#### Recommendation
Icon only frees the space without losing the mouse route. Choose Hidden only if you always search with the Windows key.

#### Sources
1. Hide or Show Search Box or Search Icon on Taskbar in Windows 10, Ten Forums tutorial, https://www.tenforums.com/tutorials/2854-hide-show-search-box-search-icon-taskbar-windows-10-a.html (tier C)
2. Control Cortana on the Windows 10 taskbar through the Registry, the 0, 1 and 2 values and the Explorer restart, https://www.404techsupport.com/2015/11/27/control-cortana-windows-10-taskbar-registry/ (tier C)

### Taskbar search style (Windows 11)

`taskbar_search_mode_win11` · Dropdown (4 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 22621 and newer · Reversible: yes

**Chooses how search appears on the Windows 11 taskbar, with the same four choices as Settings: hide, search icon only, search icon and label, or the full search box.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `search_mode` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`, value `SearchboxTaskbarMode`, REG_DWORD |
| `search_mode_cache` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`, value `SearchboxTaskbarModeCache`, REG_DWORD |

| Option | `search_mode` | `search_mode_cache` |
|---|---|---|
| Hide | `0` | `1` |
| Search icon only | `1` | `1` |
| Search icon and label | `3` | `1` |
| Search box | `2` | `absent` |

The options follow the order and wording of Settings > Personalization > Taskbar > Search. System Default is shown when the pair matches no row, for example mode `2` with a cache value present; selecting it restores the snapshot. The stock state is the search box (`2`), which the "Search box" option reproduces.

#### How it works

`SearchboxTaskbarMode` is the Settings > Personalization > Taskbar > "Search" dropdown: `0` hides search, `1` shows an icon, `2` shows the full search box, and `3` shows an icon with a label, a presentation added with the Windows 11 22H2 build 22621.1344 search redesign. Windows treats a missing `SearchboxTaskbarModeCache` as "the user has expressed no preference" and can re-migrate the mode back to `2`, which would read back as a failed apply; deployment guidance therefore writes the cache as `1` next to the mode. The tweak pins the cache for every non-stock option and removes it for "Search box", because re-migration to `2` is harmless there and writing it would leave an artefact on an otherwise pristine profile. Hiding the box does not disable search: pressing the Windows key and typing still searches. The Windows 10 variant, [Taskbar search style (Windows 10)](#taskbar-search-style-windows-10), owns the same two values; their gates never overlap.

#### Benefits
- **Reclaims space**: the search box is the widest default taskbar element.
- **Four presentations**: choose how much room search gets rather than all or nothing.
- **Keyboard search unaffected**: Windows key and type still works in every mode.

#### Drawbacks
- **Keyboard only when hidden**: mode 0 leaves no mouse entry point to search.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 (build 22621) and newer, every edition; "Search icon and label" needs 22621.1344 or later, a revision the build gate cannot express. Windows 11 21H2 (22000) is outside the gate.
- **Takes effect**: immediately, the taskbar re-lays out on its own.
- **Reverting**: restores both captured values, which on a stock machine is mode `2` with no cache value.

#### Interactions
- [Taskbar search style (Windows 10)](#taskbar-search-style-windows-10) owns the same two values on Windows 10.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: "Search box" (`2`) is the stock state, and `SearchboxTaskbarModeCache` must be written alongside non-stock modes to stop re-migration.
- **Confidence**: Community-corroborated. Microsoft's Windows IT Pro blog confirms the four presentations exist as a supported setting (its page body did not render for an automated fetch, so the numbers come from tier C sources); the cache behaviour is documented by a deployment write-up.
- **Reasoning**: key, name, type and the 0 to 3 enum held up; the missing cache companion and the stock option were the defects found, and both are addressed in the shipped shape.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Search icon only is the sweet spot for most people: it frees the space without losing the mouse route. Choose Hide only if you always search with the Windows key.

#### Sources
1. Customizing search on the Windows 11 taskbar, Windows IT Pro blog, confirms the four presentation options exist, https://techcommunity.microsoft.com/blog/windows-itpro-blog/customizing-search-on-the-windows-11-taskbar/3730314 (tier B)
2. Disabling the Windows 11 Taskbar Search Box for All Users, documents the re-migration behaviour and `SearchboxTaskbarModeCache`, https://awakecoding.com/posts/disabling-the-windows-11-taskbar-search-box-for-all-users/ (tier C)
3. How to Change Windows 11 Taskbar Search Button Layout, all four values and the default of 2, https://geekrewind.com/how-to-change-windows-11-taskbar-search-button-layout/ (tier C)

### Ungroup taskbar buttons

`taskbar_ungroup_labels` · Dropdown (3 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Gives every window its own labelled taskbar button instead of stacking them under one icon.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `glom_level` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `TaskbarGlomLevel`, REG_DWORD |
| `mm_glom_level` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `MMTaskbarGlomLevel`, REG_DWORD |

| Option | `glom_level` | `mm_glom_level` |
|---|---|---|
| Never combine | `2` | `2` |
| Combine when the taskbar is full | `1` | `1` |
| Always combine, hide labels | `0` | `0` |

System Default is shown when the two values differ (for example a different setting for other displays) or either is missing; selecting it restores the snapshot. Stock is "Always combine, hide labels" (`0`).

#### How it works

`TaskbarGlomLevel` is the "Combine taskbar buttons and hide labels" dropdown for the main taskbar: `0` always combines windows of one app under a single unlabelled button, `1` combines only when the taskbar fills up, and `2` never combines, giving each window its own labelled button. From Windows 11 24H2 the setting is split in two, and `MMTaskbarGlomLevel`, with the same enum, governs taskbars on additional displays; writing only the main value leaves secondary monitors combined. The tweak writes both with the same number. The redesigned taskbar of Windows 11 21H2 and 22H2 ignored these values; Microsoft restored the behaviour in 23H2 and it works on 24H2 and 25H2. On Windows 10, including LTSC 2021, `TaskbarGlomLevel` has always worked.

#### Benefits
- **One click per window**: no hovering a stack and picking from thumbnails.
- **Readable labels**: three browser windows are distinguishable without previewing them.
- **Graceful middle option**: "Combine when the taskbar is full" keeps labels until space runs out.

#### Drawbacks
- **Taskbar fills fast**: with many windows open, buttons shrink or overflow.
- **Poor on narrow screens**: labelled buttons need horizontal space.
- **All displays together**: the tweak cannot give the main and other displays different settings.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: immediately, the taskbar re-lays out on its own.
- **Reverting**: restores both captured values.

#### Interactions
[Focus the last active window on click](#focus-the-last-active-window-on-click) changes what clicking a combined button does, so it only matters while buttons are combined. [Show taskbar thumbnails instantly](#show-taskbar-thumbnails-instantly) affects the previews of combined buttons.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: 24H2 split the setting, so `MMTaskbarGlomLevel` has to be written with `TaskbarGlomLevel` for multi-monitor setups.
- **Confidence**: Community-corroborated, three references including one confirming the value on 25H2 and the 24H2 split.
- **Reasoning**: the enum and default survived; the missing multi-monitor companion was the defect found.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Choose "Never combine" on a wide monitor where you keep fewer than a dozen windows open. On a laptop screen choose "Combine when the taskbar is full", or leave the default.

#### Sources
1. How to enable Taskbar labels and never combine on Windows 11, the 0/1/2 enum and the 23H2 requirement, https://pureinfotech.com/show-taskbar-labels-never-combine-windows-11/ (tier C)
2. Always or Never Combine Taskbar buttons and Hide Labels in Windows 11, the same values, https://www.elevenforum.com/t/always-or-never-combine-taskbar-buttons-and-hide-labels-in-windows-11.15135/ (tier C)
3. How to Enable Never Combine Taskbar Buttons in Windows 11 25H2, confirms the value works on 25H2 and that 24H2 split the setting, https://www.tech2geek.net/how-to-enable-never-combine-taskbar-buttons-in-windows-11-25h2/ (tier C)

### Show taskbar thumbnails instantly

`taskbar_hover_time` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 22631 and older · Reversible: yes

**Taskbar thumbnail previews appear the moment you hover instead of after a pause, on builds that still honour the value.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hover_time` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `ExtendedUIHoverTime`, REG_DWORD (milliseconds) |

| Option | `hover_time` |
|---|---|
| Instant | `1` |
| 400 ms | `absent` |

System Default is shown when the value holds any other number (a custom delay); selecting it restores the snapshot. Stock is value-absent, confirmed by registry inspection on 26100.4061, with the shell falling back to an internal 400 ms. The tweak is gated to build 22631 and older, because it is widely reported to do nothing on Windows 11 24H2 and newer.

#### How it works

`ExtendedUIHoverTime` sets how long the pointer must rest on a taskbar button before its thumbnail preview appears; `1` makes it effectively instant. It is not `MouseHoverTime` under `Control Panel\Mouse`, which shares the 400 ms default but governs general hover. Whether it still works on the primary target is genuinely disputed. For: a sweep of 5,418 binaries under `System32` on 26100.4061 finds the string in exactly one module, `Taskbar.dll`, the live Windows 11 taskbar. Against: in that binary it sits in the ported legacy taskband string block, next to `MSTaskSwWClass`, `TaskbandExtendedUI`, `ThumbnailLivePreviewHoverTime`, `DisablePreviewWindow` and `MMTaskbarGlomLevel`, so its presence shows the code was carried forward, not that the path is reached; the maintained ElevenForum tutorial for this value now states it "no longer works starting with at least Windows 11 version 24H2", a Microsoft Q&A thread reports the same, and the same forum confirms it worked on 23H2. The reported cause is the taskbar's rewrite as an XAML surface that no longer reads the legacy timer. Microsoft has never documented the value.

#### Benefits
- **Faster window switching** where it works: removes a 400 ms pause from hover-switching.
- **Per user**: no admin rights and no policy.

#### Drawbacks
- **Reported broken since 24H2**: on 24H2 and 25H2 the write succeeds and reads back, but the delay may not change.
- **Twitchy at very low values**: previews fire whenever the cursor crosses the taskbar.
- **Undocumented**: no Microsoft reference in either direction.

#### Applies to, takes effect, reverting
- **Applies to**: works on Windows 10 LTSC 2021 and Windows 11 up to 23H2; on 24H2 (26100) and 25H2 the effect is disputed and most likely absent, so the tweak is gated to build 22631 and older and hidden from 24H2 on.
- **Takes effect**: after sign-out or an Explorer restart.
- **Reverting**: restores the captured state; on an untouched machine that means removing the value rather than writing 400.

#### Interactions
[Ungroup taskbar buttons](#ungroup-taskbar-buttons) changes which buttons show thumbnails. The commonly suggested 24H2 substitute is a third-party taskbar code-injection mod, which the app does not ship.

#### Validation
- **Verdict**: DISPUTED. The value's function on the primary target is contested by current evidence; the stock state was confirmed as value-absent, which the "400 ms" option reproduces.
- **Confidence**: Community-corroborated for the mechanism on older builds. Binary and registry inspection on 26100.4061 (tier A measurements) confirm the name survives and the stock is absent, but not that the value has an effect.
- **Reasoning**: a genuine evidence conflict rather than silence. The 24H2 re-scope judged it obsolete on the primary platform and recommended deleting it or gating it to Windows 10; the category research accepted an explicit warning as the alternative. The tweak now ships gated to build 22631 and older, following the re-scope (Windows 11 22H2 and 23H2 keep it, since the value is confirmed working on 22631). Only a visual test settles it: set the value to 1, restart Explorer, and watch whether the thumbnail appears immediately.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying on Windows 10 and on Windows 11 up to 23H2 if the thumbnail delay bothers you. The tweak is not offered on 24H2 and newer, where it does nothing.

#### Sources
1. Direct binary inspection, Windows 11 24H2 build 26100.4061: `ExtendedUIHoverTime` in `Taskbar.dll` only, out of 5,418 `System32` binaries, inside the legacy taskband string block (tier A, primary measurement)
2. Direct registry inspection, build 26100.4061: `ExtendedUIHoverTime` absent under `Explorer\Advanced`, so stock is value-absent (tier A, primary measurement)
3. Change Hover Time to Show Taskbar Thumbnail Previews in Windows 11, maintained tutorial carrying the note that the setting no longer works from 24H2, https://www.elevenforum.com/t/change-hover-time-to-show-taskbar-thumbnail-previews-in-windows-11.6366/ (tier C)
4. Registry key: ExtendedUIHoverTime to modify hover time for showing taskbar thumbnail previews, forum discussion confirming it works on 23H2 (22631), https://www.elevenforum.com/t/registry-key-extendeduihovertime-to-modify-hover-time-for-showing-taskbar-thumbnail-previews.24693/ (tier C)
5. ExtendedUIHoverTime is NOT working anymore, Microsoft Q&A thread reporting 24H2 breakage with no fix, https://learn.microsoft.com/en-us/answers/questions/2263244/extendeduihovertime-is-not-working-anymore (tier D)
6. Disabling taskbar thumbnails no longer works on 24H2, forum report of related taskbar values breaking on 24H2, https://www.elevenforum.com/t/disabling-taskbar-thumbnails-no-longer-works-on-24h2.33793/ (tier C)
7. Change Taskbar Thumbnail Hover Delay in Windows 10, the 400 ms fallback and the sign-out requirement, https://winaero.com/taskbar-thumbnail-hover-delay-windows-10/ (tier C)

### Left-align the taskbar

`taskbar_alignment_left` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Moves Start and the pinned icons back to the left corner of the taskbar.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `taskbar_align` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `TaskbarAl`, REG_DWORD |

| Option | `taskbar_align` |
|---|---|
| Left | `0` |
| Center | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Stock is centred (`1`); whether a fresh profile stores the `1` or leaves it absent is not settled.

#### How it works

`TaskbarAl` (the last character is a lower-case L, not an I) is the Settings > Personalization > Taskbar > Taskbar behaviours > "Taskbar alignment" dropdown. At `0` the Windows 11 taskbar anchors Start and the pinned and running icons to the left edge; at `1` it centres them, so their positions shift as apps open and close. The Windows 10 taskbar has no such setting, hence the Windows 11 gate.

#### Benefits
- **Fixed target**: Start stays in the corner instead of moving as windows open and close.
- **Muscle memory**: matches Windows 10 and every earlier release.
- **Easy target**: a screen corner is the easiest place to hit with a mouse.

#### Drawbacks
- **Longer travel on wide screens**: centred icons are closer on an ultrawide.
- **Fights shell replacements**: tools that restyle the taskbar manage alignment themselves.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); hidden on Windows 10 LTSC 2021.
- **Takes effect**: immediately, the taskbar re-lays out on its own.
- **Reverting**: restores the captured value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, two references plus the Settings dropdown.
- **Reasoning**: key, name, type, enum and the Windows 11 gate all held up. The only open point is whether stock is a written `1`.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if icons moving under your cursor annoys you. On a very wide monitor, leave it centred.

#### Sources
1. Change Taskbar Alignment in Windows 11, the value and enum, https://www.elevenforum.com/t/change-taskbar-alignment-in-windows-11.12/ (tier C)
2. How to left align Taskbar on Windows 11, the same value, https://pureinfotech.com/align-taskbar-icons-left-windows-11/ (tier C)

### Turn off the snap layouts hover flyout

`disable_snap_flyout` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Stops the snap layout grid from popping out when your cursor passes a window's maximise button.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `snap_flyout` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `EnableSnapAssistFlyout`, REG_DWORD |

| Option | `snap_flyout` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is shown when the value is present with anything other than `0` (in practice `1`, which Settings writes once its checkbox has been moved); selecting it restores the snapshot. On a stock 26100 install the value does not exist and the flyout is on.

#### How it works

`EnableSnapAssistFlyout` is the Settings > System > Multitasking > Snap windows checkbox "Show snap layouts when I hover over a window's maximize button". At `0` the layout grid no longer appears on hover. Windows 11 has three independent snap helpers, and `twinui.dll` and `SettingsHandlers_nt.dll` on 26100.4061 carry a value name for each: `EnableSnapAssistFlyout` (this hover flyout), `EnableSnapBar` (the layout bar that drops down when you drag a window to the top of the screen) and `EnableSnapAssist` (the picker shown after snapping). This tweak touches only the first. The shipped default user profile carries no `EnableSnapAssistFlyout`, there is no `DefaultValue` declaration for it, and it was absent from a fourteen-month-old live profile; the Multitasking settings handler creates it only when the checkbox is moved.

#### Benefits
- **No accidental popups**: the grid stops firing when you reach for the close button.
- **Snapping still works**: Win+Arrow, drag-to-edge and drag-to-top are unaffected.
- **Targeted**: one checkbox, not the whole snap subsystem.

#### Drawbacks
- **Loses the mouse route to layouts**: picking a layout by hovering is gone.
- **One of three switches**: the drag-to-top layout bar and the post-snap picker stay on.
- **Win+Z unchanged**: the keyboard shortcut still opens the layout grid.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); Windows 10 has no snap-layouts flyout.
- **Takes effect**: after Explorer restarts or sign-out.
- **Reverting**: restores the captured state; on an untouched machine that removes the value.

#### Interactions
[Turn off the Snap Assist suggestion picker](#turn-off-the-snap-assist-suggestion-picker) covers the post-snap picker, through the `SnapAssist` value. The drag-to-top snap bar (`EnableSnapBar`) is not covered by any tweak.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: stock is value-absent on 26100, and the value controls only the hover flyout, one of three snap helpers.
- **Confidence**: Community-corroborated for the behaviour; the value name and stock state are confirmed at tier A by binary presence and the shipped default profile.
- **Reasoning**: key, name, type and polarity held up. The name's scope ("snap layouts") was attacked and narrowed to the one surface the value controls.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if the flyout keeps appearing when you reach for the window controls. Skip it if you pick layouts from the grid with the mouse.

#### Sources
1. Shipped default user profile and live profile on build 26100.4061: `EnableSnapAssistFlyout` absent from both, no `DefaultValue` declaration; the name present in `twinui.dll` and `SettingsHandlers_nt.dll` alongside `EnableSnapBar` and `EnableSnapAssist` (tier A, shipped OS state)
2. How to Disable Snap Layouts Flyout on Maximize Button in Windows 11, the value and polarity, https://www.askvg.com/how-to-disable-snap-layouts-flyout-on-maximize-button-in-windows-11/ (tier C)
3. Enable or disable Snap Layouts in Windows 11 when you hover over Maximize button, the same value, https://www.thewindowsclub.com/enable-snap-layouts-on-windows-11 (tier C)

### Turn on Explorer compact view

`explorer_compact_view` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Tightens File Explorer row spacing so more files fit on screen.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `compact_mode` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `UseCompactMode`, REG_DWORD |

| Option | `compact_mode` |
|---|---|
| On | `1` |
| Off | `0` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Stock is Off (`0`), declared by Windows under `HKLM\...\Explorer\Advanced\Folder\UseCompactMode` with `DefaultValue = 0`.

#### How it works

`UseCompactMode` is the Folder Options > View checkbox "Use compact mode". Windows 11 increased the row padding in File Explorer lists to suit touch; at `1` Explorer returns to the tighter spacing Windows 10 uses. Windows 10 already uses the tight spacing, hence the Windows 11 gate.

#### Benefits
- **More rows per screen**: fewer scrolls in large folders.
- **Familiar density**: matches Windows 10's File Explorer.
- **Officially supported**: a Folder Options checkbox, not a hack.

#### Drawbacks
- **Smaller touch targets**: harder to hit rows with a finger or pen.
- **Shrinking payoff**: some users report that on recent 24H2 and 25H2 servicing levels compact mode removes less padding than it used to.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); hidden on Windows 10 LTSC 2021.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, with the default backed by a shipped-OS declaration (tier A).
- **Reasoning**: key, name, type, polarity and the Windows-declared default all agree.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a keyboard-and-mouse machine, where the extra padding buys nothing. Leave it off on a tablet or touchscreen laptop.

#### Sources
1. Turn On or Off Compact View in File Explorer in Windows 11, the value and Folder Options mapping, https://www.elevenforum.com/t/turn-on-or-off-compact-view-in-file-explorer-in-windows-11.896/ (tier C)
2. Windows 11 Enable Compact View in File Explorer, the same value, https://winaero.com/windows-11-enable-compact-view-in-file-explorer/ (tier C)
3. Shipped OS state on build 26100.4061: `HKLM\...\Explorer\Advanced\Folder\UseCompactMode` declares `DefaultValue = 0` (tier A, product artifact)

### Turn off promoted recommendations in Start

`disable_start_recommendations` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Stops Microsoft's promoted tips, shortcuts and app suggestions appearing in the Start menu.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `iris_recommendations` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `Start_IrisRecommendations`, REG_DWORD |

| Option | `iris_recommendations` |
|---|---|
| Off | `0` |
| On | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Recommendations are on by default.

#### How it works

`Start_IrisRecommendations` is the Settings > Personalization > Start toggle "Show recommendations for tips, shortcuts, new apps, and more". At `0` Start stops placing Microsoft-curated tips, shortcut suggestions and new-app promotions in the Recommended area. It does not remove the Recommended section: that is a different control, the `HideRecommendedSection` policy, shipped as [Hide the Recommended section in Start](#hide-the-recommended-section-in-start). It is also distinct from `SubscribedContent-338388Enabled` under `ContentDeliveryManager`, which backs "Show suggestions occasionally in Start", the Content Delivery Manager promoted-app channel; neither subsumes the other. The setting is meaningful from the Windows 11 22H2 Start redesign onward.

#### Benefits
- **No promoted content**: Microsoft-curated app and feature pitches stop appearing.
- **Settings-backed**: a supported toggle.
- **No admin rights**: a plain per-user preference.

#### Drawbacks
- **Section remains**: the Recommended area stays on screen and may look sparse.
- **Not the only channel**: Content Delivery Manager suggestions are a separate setting.
- **Loses genuine tips**: the occasional useful hint goes with the promotions.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); hidden on Windows 10 LTSC 2021.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured value.

#### Interactions
`debloat:disable_start_suggestions` clears `SubscribedContent-338388Enabled`, the other promotion channel; a quiet Start needs both. [Hide the Recommended section in Start](#hide-the-recommended-section-in-start) removes the whole section, which makes this tweak redundant while it is applied. [Turn off recent items tracking](#turn-off-recent-items-tracking) and [Hide recently added apps in Start](#hide-recently-added-apps-in-start) clear other rows of the same region.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the value clears the promoted rows inside Recommended; it does not hide the section.
- **Confidence**: Community-corroborated, three references, one of which documents `HideRecommendedSection` as the separate section-removal route.
- **Reasoning**: key, name, type and polarity held up; the scope claim was attacked and narrowed, and section removal is its own policy-backed tweak.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it. There is no functional loss worth naming. Pair it with [Hide the Recommended section in Start](#hide-the-recommended-section-in-start) if you want the space back as well.

#### Sources
1. Enable or Disable Recommended Tips, Shortcuts, New Apps, and more on Start Menu in Windows 11, the value and Settings mapping, https://www.elevenforum.com/t/enable-or-disable-recommended-tips-shortcuts-new-apps-and-more-on-start-menu-in-windows-11.14346/ (tier C)
2. How to Disable Recommended in Start Menu on Windows 11, documents `HideRecommendedSection` and `IsEducationEnvironment` as the separate section-removal route, https://winaero.com/disable-recommended-start/ (tier C)
3. How to remove Recommended section from Start menu in Windows 11, the section-removal route, https://www.thewindowsclub.com/remove-recommended-section-from-start-menu-in-windows (tier C)

### Add End task to the taskbar menu

`enable_end_task_taskbar` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 22631 and newer · Reversible: yes

**Adds "End task" to the taskbar right-click menu so you can kill a hung app in one click.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `end_task` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings`, value `TaskbarEndTask`, REG_DWORD |

| Option | `end_task` |
|---|---|
| On | `1` |
| Off | `0` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Stock is Off.

#### How it works

`TaskbarEndTask`, in the `TaskbarDeveloperSettings` subkey, is the registry backing for Settings > System > For developers > "End Task". At `1` the right-click menu of every taskbar button gains an "End task" item that terminates the app's process tree directly, like Task Manager's End task, with no save prompt. The subkey is part of the path; the value is not read from `Explorer\Advanced` itself.

#### Benefits
- **No Task Manager hunt**: kill the app from the button you are already looking at.
- **Kills the tree**: child processes go too.
- **Officially supported**: exposed as a Settings toggle.

#### Drawbacks
- **Unsaved work is lost**: a hard terminate with no prompt.
- **Easy to misuse**: on a shared machine someone may use it as a normal way to close apps, which can corrupt app state.
- **One more menu item**: the right-click menu gets slightly longer.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 build 22631 and newer (every 24H2 and 25H2 build), every edition; hidden on Windows 10 LTSC 2021, which the gate correctly excludes.
- **Takes effect**: after Explorer restarts or sign-out.
- **Reverting**: restores the captured value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, two references plus the Settings toggle.
- **Reasoning**: the full path including the subkey, the name, type, default and the build gate all held up; the 24H2 re-scope confirmed the gate still earns its place by excluding LTSC 2021.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you develop software or regularly deal with apps that stop responding. Leave it off on a family machine.

#### Sources
1. Enable or Disable End Task in Taskbar by Right Click in Windows 11, the path, value and Settings mapping, https://www.elevenforum.com/t/enable-or-disable-end-task-in-taskbar-by-right-click-in-windows-11.14325/ (tier C)
2. Enable/Disable End Task Option In Taskbar On Windows 11, the same path and value, https://cloudinfra.net/enable-disable-end-task-option-in-taskbar-on-windows-11/ (tier C)

### Hide Gallery in Explorer

`remove_gallery_nav_pane` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 22631 and newer · Reversible: yes

**Removes the Gallery entry from the File Explorer sidebar.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `gallery_pin` | registry | `HKCU\Software\Classes\CLSID\{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}`, value `System.IsPinnedToNameSpaceTree`, REG_DWORD |

| Option | `gallery_pin` |
|---|---|
| Hidden | `0` |
| Shown | `absent` |

System Default is shown when the per-user value is present with anything other than `0`; selecting it restores the snapshot. On a clean install the per-user CLSID key does not exist, so stock is value-absent and Gallery is shown.

#### How it works

`{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}` is the Gallery shell namespace extension that Windows 11 23H2 added to File Explorer. Explorer reads class registrations through `HKEY_CLASSES_ROOT`, which merges the per-user `HKCU\Software\Classes` over the machine-wide `HKLM\SOFTWARE\Classes`; a per-user `System.IsPinnedToNameSpaceTree = 0` therefore overrides the machine definition and unpins the node from the navigation tree for this user only. The pinned state otherwise comes from the machine-wide CLSID registration together with `HKLM\...\Explorer\Advanced\NavPane\ShowGallery` (default 1). Deleting the per-user value brings the machine default back. Photos, folders and the Photos app are untouched.

#### Benefits
- **Shorter sidebar**: one less node above This PC.
- **Nothing deleted**: photos, folders and apps are untouched.
- **Clean revert**: removing the per-user value restores the machine default.

#### Drawbacks
- **Loses the photo view**: no quick chronological view of recent camera and phone imports.
- **Per user only**: other accounts keep their Gallery node.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 build 22631 and newer, every edition; Gallery does not exist on earlier builds, and the gate hides the tweak on Windows 10 LTSC 2021.
- **Takes effect**: after Explorer restarts.
- **Reverting**: removes the per-user value, restoring the machine default of pinned.

#### Interactions
[Hide Home in Explorer](#hide-home-in-explorer) and [Hide OneDrive in Explorer](#hide-onedrive-in-explorer) use the same per-user unpinning mechanism on other nodes; they combine freely.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, three references on the CLSID, value and polarity.
- **Reasoning**: CLSID, value, type, the `absent` stock state and the build gate held up; the 24H2 re-scope confirmed the gate still excludes LTSC 2021.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you never browse photos through Explorer. Leave it if Gallery is how you find recent imports.

#### Sources
1. Add or Remove Gallery in File Explorer Navigation Pane in Windows 11, the CLSID and value, https://www.elevenforum.com/t/add-or-remove-gallery-in-file-explorer-navigation-pane-in-windows-11.14178/ (tier C)
2. How to Remove Gallery from File Explorer, the same mechanism, https://winaero.com/remove-gallery-from-file-explorer/ (tier C)
3. Hide Gallery from Explorer on Windows 11, the same mechanism, https://endurtech.com/hide-gallery-from-explorer-on-windows-11/ (tier C)

### Hide Home in Explorer

`remove_home_nav_pane` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Removes the Home entry from the File Explorer sidebar.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `home_pin` | registry | `HKCU\Software\Classes\CLSID\{f874310e-b6b7-47dc-bc84-b9e6b38f5903}`, value `System.IsPinnedToNameSpaceTree`, REG_DWORD |

| Option | `home_pin` |
|---|---|
| Hidden | `0` |
| Shown | `absent` |

System Default is shown when the per-user value is present with anything other than `0`; selecting it restores the snapshot. Stock is value-absent, with Home shown.

#### How it works

`{f874310e-b6b7-47dc-bc84-b9e6b38f5903}` is the Home namespace extension in Windows 11 File Explorer (Windows 11 22H2 and later). The mechanism is the same as Gallery's: a per-user `System.IsPinnedToNameSpaceTree = 0` under `HKCU\Software\Classes\CLSID` shadows the machine-wide registration and unpins the node. An alternative route exists, deleting or renaming the `{f874310e-...}` entry under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace`, but that is machine-wide and needs admin; the per-user value is reversible and scoped to one account. Home is also File Explorer's default landing page (`LaunchTo = 2`), so hiding the node while Explorer still opens to it leaves Explorer configured to open a page that is no longer in the pane. On Windows 11 21H2 the node was still called Quick access and used a different CLSID, so there this tweak does nothing; 21H2 is below the support floor.

#### Benefits
- **Drive-focused sidebar**: the pane leads with This PC and libraries.
- **No recents feed**: the pinned and recent aggregation is out of the way.
- **Clean revert**: removing the per-user value restores the machine default.

#### Drawbacks
- **Pinned folders harder to reach**: quick-access pins live inside Home.
- **Ordering trap**: Explorer's default landing page is Home; see Interactions.
- **Per user only**: other accounts keep their Home node.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); hidden on Windows 10 LTSC 2021.
- **Takes effect**: after Explorer restarts.
- **Reverting**: removes the per-user value, restoring the machine default of pinned.

#### Interactions
Apply [Open File Explorer to This PC](#open-file-explorer-to-this-pc) first or together with this; treat it as a hard ordering requirement. [Hide recent files and folders in Explorer](#hide-recent-files-and-folders-in-explorer) empties the page this node leads to. [Hide Gallery in Explorer](#hide-gallery-in-explorer) and [Hide OneDrive in Explorer](#hide-onedrive-in-explorer) use the same mechanism.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, three references on the CLSID, value and polarity.
- **Reasoning**: CLSID, value, type and the `absent` stock state held up. The `products: [11]` gate is slightly loose (21H2 uses another CLSID) but harmless below the support floor. The dependency on the landing page was raised from advice to an ordering requirement.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it together with opening File Explorer to This PC. Skip it if you rely on pinned folders, since Home is where they live.

#### Sources
1. Add or Remove Home in Navigation Pane of File Explorer in Windows 11, the CLSID and value, https://www.elevenforum.com/t/add-or-remove-home-in-navigation-pane-of-file-explorer-in-windows-11.2449/ (tier C)
2. Remove Home in the Navigation Pane of File Explorer in Windows 11, the same mechanism, https://www.ninjaone.com/blog/remove-home-in-the-navigation-pane-of-file-explorer/ (tier C)
3. How to remove File Explorer Home page on Windows 11, the same mechanism and the landing-page interaction, https://pureinfotech.com/remove-home-file-explorer-windows-11/ (tier C)

### Hide OneDrive in Explorer

`remove_onedrive_nav_pane` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Hides the OneDrive entry from the File Explorer sidebar without uninstalling OneDrive.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `onedrive_pin` | registry | `HKCU\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}`, value `System.IsPinnedToNameSpaceTree`, REG_DWORD |

| Option | `onedrive_pin` |
|---|---|
| Hidden | `0` |
| Shown | `1` |

System Default is shown when the value is absent or holds another number; on a machine without OneDrive Personal the value does not exist, so the tweak reads System Default there. Selecting it restores the snapshot. With OneDrive Personal installed, stock is `1`.

#### How it works

`{018D5C66-4533-4307-9B53-224DE2ED1FE6}` is the OneDrive Personal shell namespace extension. Unlike Gallery and Home, this CLSID key is created in HKCU by OneDrive setup, with `System.IsPinnedToNameSpaceTree = 1` already present, which is why "Shown" writes `1` rather than deleting the value: deleting it would leave the pin state undefined rather than restored. At `0` the node disappears from the navigation pane; the sync client, the local OneDrive folder and your files are untouched and keep syncing. The CLSID covers OneDrive Personal only: OneDrive for Business sync roots use per-tenant CLSIDs and are unaffected. On a machine without OneDrive Personal, applying Hidden creates a stray per-user key that does nothing.

#### Benefits
- **Cleaner sidebar**: removes a node you do not use without uninstalling anything.
- **Sync unaffected**: files keep syncing exactly as before.
- **Faithful revert**: Shown writes the `1` that OneDrive setup writes.

#### Drawbacks
- **Confusing while syncing**: files keep uploading from a location you can no longer see in the pane.
- **Personal only**: OneDrive for Business nodes stay.
- **Inert without OneDrive**: the tweak is not gated on OneDrive being installed.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition where OneDrive Personal is installed and signed in, including Windows 10 LTSC 2021 (OneDrive ships there as a Win32 setup payload).
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured value, normally `1`.

#### Interactions
`debloat:remove_onedrive` uninstalls the OneDrive client, after which this tweak has nothing to hide. [Hide Gallery in Explorer](#hide-gallery-in-explorer) and [Hide Home in Explorer](#hide-home-in-explorer) use the same unpinning mechanism.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the stock state is `1` present, written by OneDrive setup, not value-absent.
- **Confidence**: Community-corroborated, two references, one stating the value defaults to `1` when OneDrive is installed.
- **Reasoning**: CLSID, value, type and polarity held up; the stock representation was the attacked point. A gate on OneDrive being installed was suggested and is not implemented.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if OneDrive is installed but you store nothing there. If OneDrive is your main document location, leave the node alone; hiding it makes sync state invisible.

#### Sources
1. How to Remove OneDrive Icon from File Explorer in Windows 11, the value and its default of 1 when OneDrive is installed, https://winaero.com/how-to-remove-onedrive-icon-from-file-explorer-in-windows-11/ (tier C)
2. Add or Remove OneDrive in Navigation Pane of File Explorer in Windows 11, the same CLSID and value, https://www.elevenforum.com/t/add-or-remove-onedrive-in-navigation-pane-of-file-explorer-in-windows-11.2478/ (tier C)

### Turn off toast notifications

`disable_toast_notifications` · Switch (2 options) · Risk: medium · Elevation: none · Reboot: no (a sign-out is needed) · Windows: all supported builds · Reversible: yes

**Silences every pop-up notification on the machine, from every app and from Windows itself.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `toast_enabled` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications`, value `ToastEnabled`, REG_DWORD |

| Option | `toast_enabled` |
|---|---|
| Off | `0` |
| On | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Notifications are on by default; whether a fresh profile stores `1` or leaves the value absent is not settled.

#### How it works

`ToastEnabled` is the master per-user notification switch, the value behind Settings > System > Notifications > "Notifications". At `0` Windows suppresses every toast and banner from every source: cloud-delivered (WNS) push notifications, local app notifications and system notifications alike. The notification platform reads the value at session start, so the change takes hold after signing out and back in. Writing it while the shell is running can also leave the Settings toggle showing a stale state until the next sign-in.

#### Benefits
- **Total silence**: nothing pops up during a presentation, recording or game.
- **One switch**: no need to disable notifications app by app.
- **Simple revert**: one per-user value.

#### Drawbacks
- **Security alerts vanish**: Windows Security warnings and BitLocker prompts are suppressed.
- **Data-loss warnings vanish**: backup failures and low-battery warnings go with them.
- **Everything else too**: calendar reminders, messaging apps and delivery alerts.
- **Focus is usually better**: Do Not Disturb suppresses banners while still queuing notifications in the Notification Center.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after sign-out and sign-in. The tweak does not set `requires_reboot`, so the app does not prompt for it.
- **Reverting**: restores the captured value; again needs a sign-out.

#### Interactions
[Remove the Notification Center](#remove-the-notification-center) is adjacent but different: it removes the history panel and its taskbar entry, while notifications still pop up. `debloat:disable_nag_toasts` is the surgical alternative, silencing only two Windows nag toast sources (`Windows.SystemToast.Suggested` and `Windows.SystemToast.BackupReminder`).

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting; `medium` risk was judged right and should not be lowered.
- **Confidence**: Community-corroborated, two references including one documenting the sign-out requirement.
- **Reasoning**: key, name, type and polarity held up. This is the tweak in the category most likely to cause real harm, by hiding a security or data-loss warning, which is why it is rated medium.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Use it only for a specific situation such as a demo, a recording session or a kiosk. For day-to-day quiet, turn on Do Not Disturb instead; it keeps the notifications you missed.

#### Sources
1. How to disable notifications in Windows 11, documents `ToastEnabled` and the sign-out requirement, https://winaero.com/how-to-disable-notifications-in-windows-11/ (tier C)
2. Turn On or Off Notifications in Windows 11, the same value, https://www.elevenforum.com/t/turn-on-or-off-notifications-in-windows-11.821/ (tier C)

### Show verbose logon messages

`verbose_logon_messages` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Replaces the sign-in spinner with step-by-step text so you can see what Windows is doing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `verbose_status` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`, value `VerboseStatus`, REG_DWORD |

| Option | `verbose_status` |
|---|---|
| On | `1` |
| Off | `absent` |

System Default is shown when the value is present with anything other than `1` (for example an explicit `0`); selecting it restores the snapshot. Stock is value-absent, the unconfigured policy state.

#### How it works

`VerboseStatus` is the registry backing for the Group Policy "Display highly detailed status messages" (Computer Configuration > Administrative Templates > System, defined in `Logon.admx` as a Machine-class policy, so HKLM is the correct hive). When set, Windows replaces the generic "Welcome", "Signing out" and "Shutting down" screens with the phase it is in, such as "Applying computer settings" or "Loading your profile". It changes only what is drawn. The policy is ignored when the "Remove Boot / Shutdown / Logon / Logoff status messages" policy (`DisableStatusMessages`, same key) is enabled.

#### Benefits
- **Diagnoses slow boots**: you can see which phase stalls instead of watching a spinner.
- **Zero performance cost**: it changes what is displayed, nothing else.
- **Policy-backed**: a documented Microsoft setting.

#### Drawbacks
- **Looks alarming**: a non-technical user may read normal phase text as an error.
- **Leaks names**: policy names and internal hostnames can appear on a shared or projected screen.
- **Can be overridden**: ignored while `DisableStatusMessages` is enabled.
- **Needs admin**: the value lives in HKLM and applies to every account.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition (the policy exists on every Windows version since Windows 2000).
- **Takes effect**: at the next sign-in, sign-out or restart; the tweak is marked as needing a reboot.
- **Reverting**: restores the captured state, normally by removing the value.

#### Interactions
[Hide the unsupported hardware notice](#hide-the-unsupported-hardware-notice) writes a different value in the same `Policies\System` key; the app snapshots per value, so they do not collide. No tweak writes `DisableStatusMessages`.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Microsoft-documented: the ADMX policy definition, with the hive confirmed against the shipped `Logon.admx` class by the policy-hive audit.
- **Reasoning**: hive, key, name, type and the `absent` unset state match the policy definition exactly; the documented `DisableStatusMessages` override is the only caveat.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it while diagnosing a slow or hanging sign-in, and on any machine you maintain. Turn it back off on a shared or public-facing machine.

#### Sources
1. Display highly detailed status messages (`Microsoft.Policies.WindowsLogon::VerboseStatus`), ADMX policy reference, https://admx.help/?Category=Windows_10_2016&Policy=Microsoft.Policies.WindowsLogon::VerboseStatus (tier A; the host returned HTTP 522 intermittently during research)
2. Enable Detailed Status Messages at Shut down, Sign out, and Sign in, the same value and its effect, https://www.tenforums.com/tutorials/100262-enable-detailed-status-messages-shut-down-sign-out-sign.html (tier C)
3. Shipped `Logon.admx` on build 26100: `VerboseStatus` declared Machine class, so HKLM is correct (tier A, via the policy-hive audit)

### Silence the startup sound

`disable_startup_sound` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Silences the Windows startup chime played when the sign-in screen appears.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `startup_sound` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI\BootAnimation`, value `DisableStartupSound`, REG_DWORD |

| Option | `startup_sound` |
|---|---|
| Silent | `1` |
| Play the startup sound | `0` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. The stock value is not settled: one Windows 11 24H2 IoT Enterprise LTSC 2024 machine (26100.4061) carries `1`, but that evidence may be contaminated (see Validation), and retail Home and Pro images were not checked. Note the polarity: "Play the startup sound" (`0`) turns the chime on; it is an explicit choice, not a restore.

#### How it works

`DisableStartupSound` under the LogonUI `BootAnimation` key is read before the boot animation plays the Windows startup chime; at `1` the chime is suppressed. Microsoft's unattend reference documents the same setting name, `Microsoft-Windows-Authentication-AuthUI\DisableStartupSound`, for image-time configuration, which confirms the setting's meaning. There is also a Group Policy of the same name in the shipped `Logon.admx`, but at a different key (`Software\Microsoft\Windows\CurrentVersion\Policies\System`), gated `SUPPORTED_WindowsVistaTo7`, and its explain text on 26100 reads "This policy is not available in this version of Windows"; the policy route is retired and the `BootAnimation` value is the live location. It is not the only location read, though: `mmsys.cpl`, `authui.dll` and `LogonController.dll` on 26100 each reference both keys, so a machine that still carries the retired `Policies\System\DisableStartupSound` value can override this preference. The "Windows Startup" event in the Sound control panel's scheme is a second, per-user gate.

#### Benefits
- **Quiet boot**: nothing plays in a shared office, lecture hall or bedroom.
- **Machine-wide**: applies to every account.
- **Documented setting name**: the same name Microsoft exposes for unattended installs.

#### Drawbacks
- **Loses an audible cue**: no signal that the machine reached the sign-in screen.
- **Often already off**: many Windows 11 images appear to ship the chime disabled, so Silent may change nothing.
- **Two gates**: the Sound scheme's "Windows Startup" entry can still silence or allow the chime independently.
- **A stale policy value can win**: a leftover `Policies\System\DisableStartupSound` overrides this preference.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: at the next boot, since the sound is played by the boot animation.
- **Reverting**: restores the captured value exactly, so an undo never has to guess the stock state.

#### Interactions
None among shipped tweaks. The retired policy value and the Sound scheme entry, described above, are the two outside influences.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: `0` enables a chime that at least some 26100 images ship disabled, so it is offered as an explicit "Play the startup sound" choice rather than as the Windows default.
- **Confidence**: Microsoft-documented for the setting (unattend reference and shipped ADMX), with the retired policy route confirmed from the shipped ADML.
- **Reasoning**: the setting and key held up. The stock value is contested: the category research read `1` on 26100.4061 with a key timestamp matching the image build date, but the harmful-revert review withdrew that finding because the machine's owner had very likely applied this very tweak, making the reading circular. Two open questions remain: whether retail Home and Pro 26100 images ship `1`, and whether the HKLM value alone silences the chime without the per-user Sound scheme entry.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply Silent on any machine that boots in a shared or quiet space. Leave it if you use the chime as your cue that the machine is ready.

#### Sources
1. Shipped registry on Windows 11 24H2 build 26100.4061 (IoT Enterprise LTSC 2024): `DisableStartupSound = 1` under `LogonUI\BootAnimation` (tier A as a measurement, but judged possibly contaminated by the harmful-revert review)
2. Shipped `Logon.admx` and `en-US\Logon.adml` on 26100: the retired `DisableStartupSound` policy at `Policies\System`, `SUPPORTED_WindowsVistaTo7`, "This policy is not available in this version of Windows." (tier A)
3. `Microsoft-Windows-Authentication-AuthUI-DisableStartupSound` unattend reference, the setting's existence and meaning, https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-authentication-authui-disablestartupsound (tier A)
4. Enable or Disable Startup Sound in Windows 11, the `BootAnimation` value, https://www.elevenforum.com/t/enable-or-disable-startup-sound-in-windows-11.85/ (tier C)
5. Disable Windows 11 Startup Sound using these three methods, the value and the Sound scheme route, https://winaero.com/disable-windows-11-startup-sound-using-these-three-methods/ (tier C)

### Turn off accessibility shortcut prompts

`disable_accessibility_key_prompts` · Switch · Risk: low · Elevation: none · Reboot: no (a sign-out is needed) · Windows: all supported builds · Reversible: yes

**Stops the Sticky Keys, Filter Keys and Toggle Keys popups that fire from stray key presses.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `sticky_keys` | registry | `HKCU\Control Panel\Accessibility\StickyKeys`, value `Flags`, REG_SZ |
| `filter_keys` | registry | `HKCU\Control Panel\Accessibility\Keyboard Response`, value `Flags`, REG_SZ |
| `toggle_keys` | registry | `HKCU\Control Panel\Accessibility\ToggleKeys`, value `Flags`, REG_SZ |

| Option | `sticky_keys` | `filter_keys` | `toggle_keys` |
|---|---|---|---|
| Prompts off | `"506"` | `"122"` | `"58"` |

This is a toggle with one authored option. Any other combination reads as System Default, which is the "off" side of the toggle; selecting it restores the three values captured in the snapshot. The common Windows defaults are `"510"`, `"126"` and `"62"`, but these are live bitmasks, so a machine's actual values depend on which accessibility sub-options its user has changed.

#### How it works

Each `Flags` value is the decimal `dwFlags` member of the corresponding Win32 structure (`STICKYKEYS`, `FILTERKEYS`, `TOGGLEKEYS`), stored as a string; written as REG_DWORD they are ignored, so the REG_SZ type is load-bearing. In all three structures bit `0x4` (`SKF_HOTKEYACTIVE`, `FKF_HOTKEYACTIVE`, `TKF_HOTKEYACTIVE`) enables the keyboard activation gesture: five Shift presses for Sticky Keys, holding Shift for eight seconds for Filter Keys, holding NumLock for five seconds for Toggle Keys. 510 minus 506, 126 minus 122 and 62 minus 58 are each 4, so from the common defaults the option clears exactly that bit and leaves the availability, confirm-hotkey, hotkey-sound, indicator and audible-feedback bits alone; the features themselves stay available in Settings. The option writes fixed strings, however, not a bit operation on the live value. On a machine whose flags differ from the common defaults (for example one where Sticky Keys is switched on, which sets bit `0x1`), applying replaces the whole value and resets those other sub-options too; that is why there is no authored "on" option and the undo goes through the snapshot, which holds the machine's own pre-apply values. Windows reads these values at session start.

#### Benefits
- **No mid-game interruption**: rapid Shift presses stop triggering a modal prompt.
- **Features stay available**: Settings can still turn Sticky Keys and the others on deliberately.
- **Exact undo**: the snapshot restores the machine's own values, not a guessed default.

#### Drawbacks
- **Removes an accessibility shortcut**: someone who relies on the keyboard gesture to enable Sticky Keys loses it, a real consideration on a shared account.
- **Fixed values**: on a machine with non-default accessibility sub-options, applying resets them until you revert.
- **Needs a sign-out**, and Settings may not reflect the change until then.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after sign-out and sign-in. The tweak does not set `requires_reboot`.
- **Reverting**: restores all three captured values from the snapshot, then needs a sign-out.

#### Interactions
Changing any Sticky Keys, Filter Keys or Toggle Keys option in Settings rewrites these `Flags` values and moves the tweak to System Default.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: `Flags` is a live bitmask, so restoring hardcoded constants would clobber other sub-settings; the revert therefore comes from the snapshot, and a sign-out is needed.
- **Confidence**: Microsoft-documented: the three structure pages define the hotkey-active bit as `0x4`; the REG_SZ storage and the 510/506 pair are community-documented.
- **Reasoning**: the bit arithmetic checks out against Microsoft's constants. The attacked point was the revert, which the one-option shape addresses. The apply side still writes constants, so it is a single-bit change only when the starting values are the common defaults.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you game or type fast and the prompts keep interrupting you. Do not apply it on an account shared with someone who uses the keyboard gestures, or on an account that already has customised accessibility keyboard options.

#### Sources
1. STICKYKEYS structure, documents `SKF_HOTKEYACTIVE` = 0x00000004 and the rest of the bitmask, https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys (tier A)
2. FILTERKEYS structure, documents `FKF_HOTKEYACTIVE` = 0x00000004, https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-filterkeys (tier A)
3. TOGGLEKEYS structure, documents `TKF_HOTKEYACTIVE` = 0x00000004, https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-togglekeys (tier A)
4. Turn On or Off Sticky Keys in Windows 11, the REG_SZ `Flags` values and the 510/506 pair, https://www.elevenforum.com/t/turn-on-or-off-sticky-keys-in-windows-11.8889/ (tier C)
5. Disabling StickyKeys for Good, the same values, https://blog.duklabs.com/disabling-stickykeys-for-good/ (tier C)

### Restore the classic context menu

`classic_context_menu_win11` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Brings back the full Windows 10 right-click menu, so shell extensions appear on the first click.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `context_menu` | registry | `HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32`, the default (unnamed) value, REG_SZ |

| Option | `context_menu` |
|---|---|
| Classic | `""` (empty string) |
| Modern | `absent` |

System Default is shown when the default value holds a non-empty string; selecting it restores the snapshot. Stock is value-absent: the per-user key does not exist on a clean install.

#### How it works

`{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}` is the COM class that implements the Windows 11 compact context menu. Registering an empty default value for its `InprocServer32` under the per-user classes key makes COM activation of that class fail, and Explorer falls back to the legacy menu, in which every registered shell extension appears directly instead of behind "Show more options". The shape (empty value name, REG_SZ, empty data) is exactly what `reg add "HKCU\Software\Classes\CLSID\{86ca1aa0-...}\InprocServer32" /f /ve` produces. Reverting deletes only the value and leaves an empty `InprocServer32` subkey behind, whereas published recipes delete the whole CLSID key; a direct test on 26100.4061 showed this is enough, because `HKEY_CLASSES_ROOT` merges HKCU over HKLM per value, not per key, so an empty subkey does not shadow the machine registration. Most "it stopped working" reports come from running `reg add` in an elevated shell under a different account, which writes the wrong user's hive, or from not restarting Explorer. The app writes HKCU effects in-process as the signed-in user even when elevated, and blocks HKCU tweaks when the elevated account differs from the session owner, so it avoids that failure.

#### Benefits
- **No "Show more options" step**: 7-Zip, Git, TortoiseSVN and similar entries appear at once.
- **All extensions visible**: legacy extensions that never adopted the new menu API show up.
- **Clean revert**: deleting the one value restores the Windows 11 menu.

#### Drawbacks
- **Loses the Windows 11 design**, including the icon row for cut, copy, rename and delete.
- **Modern-only entries may vanish**: apps that register only with the new menu API can lose their items.
- **Flagged as deprecated, not blocked**: a future release could stop honouring it, though it is on neither Microsoft's deprecated-features nor removed-features list.
- **Leaves an empty subkey** after revert, cosmetic and harmless.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition), confirmed working on 24H2 and 25H2; hidden on Windows 10 LTSC 2021, which already has the classic menu.
- **Takes effect**: after Explorer restarts.
- **Reverting**: deletes the empty default value, which restores the modern menu.

#### Interactions
None known. [Hide Gallery in Explorer](#hide-gallery-in-explorer) and the other navigation-pane tweaks also write under `HKCU\Software\Classes\CLSID`, for different classes.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting.
- **Confidence**: Community-corroborated, with the revert behaviour empirically tested (the per-value merge was measured on 26100.4061 against a class with the same HKLM `InprocServer32` shape, so as not to disturb the live menu).
- **Reasoning**: the 24H2 re-scope attacked "Microsoft has blocked it" and found no block or removal notice; a Microsoft-answered Q&A thread traced the failures to user error. The value-only revert was attacked as possibly leaving a shadowing key and survived the test.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use shell-extension tools daily; the extra click is a constant tax. Skip it if you like the compact menu and its icon row.

#### Sources
1. Direct test on 26100.4061: `HKEY_CLASSES_ROOT` merges HKCU over HKLM per value, so deleting the empty default value alone restores the machine registration (tier A, primary measurement)
2. Restore old Right-click Context menu in Windows 11, Microsoft Q&A article, the canonical write-up of the method, https://learn.microsoft.com/en-us/answers/questions/2287432/article-restore-old-right-click-context-menu-in-wi (tier D)
3. Unable to change Win11 context menu with registry, Microsoft Q&A thread whose answer traces failures to elevated-shell writes and missing Explorer restarts, https://learn.microsoft.com/en-us/answers/questions/4045956/unable-to-change-win11-context-menu-with-registry (tier D)
4. 4 Ways to Get the Old Context Menus Back in Windows 11 25H2, confirms the method on 25H2, https://www.techbloat.com/4-ways-to-get-the-old-context-menus-back-in-windows-11-25h2.html (tier C)
5. How To Enable Classic Context Menu In Windows 11 (Regedit), the same method, https://memstechtips.com/enable-classic-context-menu-windows-11-regedit/ (tier C)
6. Microsoft deprecated features and removed features lists, neither of which lists the method, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features and https://learn.microsoft.com/en-us/windows/whats-new/removed-features (tier A)

### Turn NumLock on at startup

`numlock_on_startup` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no (a sign-out is needed) · Windows: all supported builds · Reversible: yes

**Turns NumLock on automatically for your sessions, so the numeric keypad works right away.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `numlock` | registry | `HKCU\Control Panel\Keyboard`, value `InitialKeyboardIndicators`, REG_SZ |

| Option | `numlock` |
|---|---|
| NumLock on | `"2147483650"` |
| NumLock off | `"0"` |

System Default is shown when the value holds any other string, such as `"2"` or `"2147483648"`, or is missing; selecting it restores the snapshot. Microsoft documents the default as `"0"` (NumLock off), and the value exists on essentially every profile because Windows writes it itself.

#### How it works

Microsoft documents `InitialKeyboardIndicators` under `HKCU\Control Panel\Keyboard` as REG_SZ with a range of `0` (NumLock off) or `2` (NumLock on) and a default of `0`, and states that "the system stores the state of the NUMLOCK key in this entry during logoff and shutdown, and then it uses this value to restore the state when the user logs on". `2147483650` is `0x80000002`: the documented `2` with the high bit set, an undocumented extension used from Windows 8 onward that community sources report as more reliable than a bare `2` on modern builds (`2147483648`, `0x80000000`, is the matching "off" form commonly seen in `HKU\.DEFAULT`). Two limits follow. First, the value is a scratch record Windows overwrites at every sign-out and shutdown from the live NumLock state, so if you turn NumLock off during a session the tweak un-applies itself and the status can read System Default afterwards. Second, the sign-in screen does not run in your profile: it reads `HKU\.DEFAULT\Control Panel\Keyboard\InitialKeyboardIndicators`, which this tweak does not write (the app's registry effects address only HKLM and HKCU), so NumLock at the sign-in screen is not covered. Fast Startup can restore the pre-shutdown keyboard state from the hibernation image instead, and some UEFI firmware has its own NumLock-at-boot setting.

#### Benefits
- **Keypad ready**: digits work from the moment your session starts.
- **No manual toggle**: nothing to press after every sign-in.
- **No admin rights**: it writes only your own profile.

#### Drawbacks
- **Not the sign-in screen**: that reads `HKU\.DEFAULT`, which this tweak does not touch.
- **Un-applies itself**: Windows rewrites the value at sign-out from the live state.
- **Laptop keypad overlay**: on laptops with an embedded numeric overlay, NumLock on turns letter keys into digits.
- **Fast Startup can defeat it**: the hibernated keyboard state may win.
- **Undocumented form**: `2147483650` rests on community reports, not Microsoft's documented range.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after signing out and back in.
- **Reverting**: restores the captured value; "NumLock off" writes the documented default `"0"`.

#### Interactions
`performance:disable_fast_startup` turns Fast Startup off, which removes the hibernation-image path that can override this value.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: an HKCU-only write cannot affect the sign-in screen; the documented default is `"0"`, not a missing value; the high-bit form is undocumented; and the laptop-overlay hazard needed stating.
- **Confidence**: Microsoft-documented for the key, type, range, default and logoff rewrite (an archived Microsoft reference); community-corroborated for the high-bit form and the `.DEFAULT` requirement.
- **Reasoning**: the value and its session semantics held up. The attacked claim, NumLock at the sign-in screen, failed and is not made. The self-rewriting behaviour means a stable "applied" status is not guaranteed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a desktop with a full-size keyboard where you enter numbers often. Skip it on a laptop with an embedded keypad overlay.

#### Sources
1. `InitialKeyboardIndicators`, archived Microsoft reference: REG_SZ, range 0 or 2, default 0, rewritten at logoff and shutdown, https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/cc978657(v=technet.10) (tier A)
2. Enable NumLock on the Windows 10 Login screen / Lock screen, the `HKU\.DEFAULT` requirement and the 2147483650 form, https://winaero.com/enable-numlock-logon-screen-windows-10/ (tier C)
3. `InitialKeyboardIndicators` registry setting to fix NumLock in a certain state, documents 2147483648 in `.DEFAULT`, https://wiert.me/2016/09/30/initialkeyboardindicators-registry-setting-to-fix-numlock-in-a-certain-state/ (tier C)

### Hide the Recommended section in Start

`disable_start_recommended_section` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: build 22621 and newer · Reversible: yes

**Removes the entire Recommended section from the Start menu, not just the promoted rows.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hide_recommended` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `HideRecommendedSection`, REG_DWORD |

| Option | `hide_recommended` |
|---|---|
| Hidden | `1` |
| Shown | `absent` |

System Default is shown when the value is present with anything other than `1`, for example an explicit `0`; selecting it restores the snapshot. Stock is value-absent, the unconfigured policy.

#### How it works

`HideRecommendedSection` is a Group Policy defined in the shipped `StartMenu.admx` on 26100 with `class="Both"`, so it may be set per user (HKCU, as here) or per machine (HKLM). When set to `1`, Start drops the whole Recommended area (promoted tips, suggested apps and recent files together) and the pinned app grid takes the space. The Policy CSP documents allowed values `0` (default, shown) and `1` (hidden), applicability from Windows 11 22H2 (build 22621), and editions Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC; Home is not listed. The ADMX `supportedOn` reference (`SUPPORTED_Windows_11_0_SE`) is only the "Requirements" line shown in the policy editor and does not limit what the OS reads. The value is read on 26100: the name appears in `StartMenu.dll` inside a run of known-working Start policy names (`HideAppList`, `NoStartMenuMorePrograms`, `DisableContextMenus`) and in `SHCore.dll`. The ADMX declares no explicit enabled or disabled value, so the tweak reverts by deleting the value rather than writing `0`, and it does not write the MDM cache under `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Start`, which is not an authoring surface.

#### Benefits
- **Reclaims the space**: pinned apps fill the panel.
- **Removes everything in it**: promotions, suggestions and recent files together.
- **Documented policy**: defined in the shipped ADMX and the Policy CSP.

#### Drawbacks
- **Loses recent files in Start**: the fastest route back to a document you just closed.
- **Not on Home**: Microsoft lists Pro, Enterprise, Education and IoT Enterprise editions only.
- **Start host restart needed**: the layout reflows after sign-out or a restart of `StartMenuExperienceHost.exe`.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 build 22621 and newer on Pro, Enterprise, Education and IoT Enterprise editions; hidden on Windows 10 LTSC 2021. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after sign-out, or when `StartMenuExperienceHost.exe` restarts; no reboot.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
Makes [Turn off promoted recommendations in Start](#turn-off-promoted-recommendations-in-start) and the Start half of [Turn off recent items tracking](#turn-off-recent-items-tracking) redundant while applied; [Hide recently added apps in Start](#hide-recently-added-apps-in-start) is the fallback on editions this policy does not reach. Several other tweaks write different values under the same `Policies\Microsoft\Windows\Explorer` key (for example [Alt+Tab shows windows only](#alttab-shows-windows-only) and `debloat:disable_web_search_start`); the app snapshots per value, so they do not collide.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Microsoft-documented: the shipped ADMX and ADML on 26100 and the Policy CSP, plus binary evidence that the Start menu reads the value.
- **Reasoning**: the adversarial pass confirmed the key, name, type, polarity and the `Both` class, and resolved the misleading ADMX `supportedOn` string against the CSP. It required the revert to be `absent` and the PolicyManager fallback to be dropped; both are how the tweak ships.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use Start purely as an app launcher. Skip it if you reopen recent documents from Start.

#### Sources
1. Shipped `StartMenu.admx` and `en-US\StartMenu.adml` on 26100: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, no enabled or disabled value declared (tier A)
2. Policy CSP - Start > HideRecommendedSection, editions, applicability from 22621 and allowed values, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start (tier A)
3. Binary string evidence on 26100.4061: `HideRecommendedSection` in `StartMenu.dll` and `SHCore.dll` (tier A, product artifact)

### Hide the unsupported hardware notice

`hide_unsupported_hardware_notice` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Clears the "system requirements not met" watermark from the desktop and the About page.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hide_unsupported` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`, value `HideUnsupportedHardwareNotifications`, REG_DWORD |

| Option | `hide_unsupported` |
|---|---|
| Hidden | `1` |
| Shown | `absent` |

System Default is shown when the value is present with anything other than `1`; selecting it restores the snapshot. Stock is value-absent.

#### How it works

The shipped `ControlPanel.admx` on 26100 defines this as a Machine-class policy at exactly this key, supported on Windows 11 client editions (`SUPPORTED_Windows_11_0_NOSERVER`). Its ADML text: "This policy controls messages which are shown when Windows is running on a device that does not meet the minimum system requirements for this OS version. If you enable this policy setting, these messages will never appear on desktop or in the Settings app." The value name appears in exactly the two binaries that draw those messages: `shell32.dll` (the desktop watermark) and `AboutSettingsHandlers.dll` (the Settings > System > About banner). It is cosmetic only: nothing in update eligibility or servicing reads it, so an unsupported PC stays unsupported. The policy declares no enabled or disabled value, so there is no Microsoft-defined `0` state; the tweak reverts by deleting the value and must never write `0`.

#### Benefits
- **Clean desktop**: the corner watermark over your wallpaper is gone.
- **Clean About page**: the Settings banner clears too.
- **Documented policy**: defined in the shipped ADMX.

#### Drawbacks
- **Cosmetic only**: it changes nothing about update eligibility or servicing.
- **Hides a real signal**: an upgrade may still be blocked later on unsupported hardware.
- **Needs admin**: it writes HKLM and applies to every user.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 client editions (all supported builds); hidden on Windows 10 LTSC 2021.
- **Takes effect**: the About banner clears the next time that page opens; the research could not establish whether the desktop watermark clears on an Explorer restart or only at the next sign-in.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
[Show verbose logon messages](#show-verbose-logon-messages) writes a different value in the same `Policies\System` key; the key is also home to UAC and legal-notice values used elsewhere. The app snapshots per value, and no other tweak writes this name.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Microsoft-documented: the shipped ADMX and ADML, with binary evidence mapping one to one onto the two surfaces.
- **Reasoning**: key, name, type and class matched the ADMX exactly in the adversarial pass; the only constraint added was that the revert stays `absent`. Open question: the watermark's apply latency.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine you deliberately run Windows 11 on despite the hardware check. Skip it if the reminder is useful to you.

#### Sources
1. Shipped `ControlPanel.admx` and `en-US\ControlPanel.adml` on 26100: `class="Machine"`, key `Software\Microsoft\Windows\CurrentVersion\Policies\System`, `SUPPORTED_Windows_11_0_NOSERVER`, no enabled or disabled value, and the explain text quoted above (tier A)
2. Binary string evidence on 26100: the value name in `shell32.dll` and `AboutSettingsHandlers.dll` (tier A, product artifact)

### Hide the mobile device panel in Start

`disable_phone_companion_start` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 22631 and newer · Reversible: yes

**Removes the phone panel from the Start menu without uninstalling Phone Link.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `phone_companion` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe`, value `IsEnabled`, REG_DWORD |

| Option | `phone_companion` |
|---|---|
| Hidden | `0` |
| Shown | `absent` |

System Default is shown when the value is present with anything other than `0`, for example `1` written by the Settings toggle; selecting it restores the snapshot. Stock is value-absent, in which case Start falls back to the default the Phone Link package declares.

#### How it works

This is the backing store for Settings > Personalization > Start > "Show mobile device in Start", documented by Microsoft's support article on the feature. The Start binaries on 26100 (`StartDocked.dll` and `StartMenu.dll`) carry the key path `Software\Microsoft\Windows\CurrentVersion\Start\Companions` followed by a per-companion value run that includes `IsEnabled`, `IsAvailable` and `DefaultState`, and a `%ls\%ls` format string that composes the per-package subkey. So `IsEnabled` is a per-companion value under a subkey named for the package family, here Phone Link's `Microsoft.YourPhone_8wekyb3d8bbwe`. When `IsEnabled` is absent the shell uses the package's `DefaultState`, which is why "Shown" deletes the value rather than writing `1`: it returns to whatever the package declares, not to a forced "shown". The Phone Link app itself is untouched and keeps syncing if you open it. On an image without Phone Link (including IoT Enterprise LTSC) there is nothing to hide, and the write is inert.

#### Benefits
- **Narrower Start menu**: the side panel goes and Start returns to its normal width.
- **App still works**: Phone Link keeps running if you open it directly.
- **Settings-backed**: a supported toggle.

#### Drawbacks
- **Loses quick phone access**: notifications, battery and recent photos are no longer one click from Start.
- **Drift**: flipping the Settings toggle later changes the value behind the tweak's back.
- **Inert without Phone Link**.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 from 26100.3915 (and 23H2 from 22631.5262), every edition that ships Phone Link; the gate is build 22631 and newer because the app cannot gate on a servicing revision, so earlier 24H2 revisions show the tweak with nothing to hide.
- **Takes effect**: the next time Start opens; no sign-out or reboot.
- **Reverting**: restores the captured state, normally removing the value. If Phone Link was removed in the meantime, the panel does not come back.

#### Interactions
`debloat:remove_phone_link` uninstalls the Phone Link app instead, after which this tweak has nothing to act on. `services:disable_cdpsvc` stops the Connected Devices Platform service that Phone Link relies on.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Community-corroborated for the hive and package name (three independent sources agree), with the key path and value name confirmed at tier A by binary evidence and the Settings surface by Microsoft's support article.
- **Reasoning**: the adversarial pass confirmed the mechanism, established `absent` as the more faithful revert than the `1` one community script writes, and corrected the reboot flag to false: no source requires a sign-out.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you do not pair a phone or find the panel makes Start too wide. Skip it if you use the panel for notifications and photo transfers.

#### Sources
1. Binary string evidence on 26100.4061: `Start\Companions` in `StartDocked.dll` and `StartMenu.dll`, with the per-companion value run containing `IsEnabled`, `DefaultState` and the `%ls\%ls` subkey format (tier A, product artifact)
2. Mobile device in Start menu, Microsoft support, the Settings surface this value backs, https://support.microsoft.com/en-us/windows/mobile-device-in-start-menu-21676d6a-3bc3-439a-aaa3-7463b91cda79 (tier A for the Settings surface)
3. Win11Debloat `Regfiles/Disable_Phone_Link_In_Start.reg`, `"IsEnabled"=dword:00000000` at this key (tier C)
4. ElevenForum tutorial 26919 (via the Wayback Machine), the identical `.reg` body plus the 26100.3915 and 22631.5262 build attribution (tier C)
5. Microsoft Q&A 5510106, the same key and value, reported present on 26100.4770 (tier C)

### Turn off the Drop Tray share overlay

`disable_drag_tray` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: build 26100 and newer · Reversible: yes

**Stops the share overlay from sliding down at the top of the screen when you drag a file.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `drag_tray` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\CDP`, value `DragTrayEnabled`, REG_DWORD |

| Option | `drag_tray` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is shown when the value is present with anything other than `0`, for example `1` written by the Settings toggle; selecting it restores the snapshot. Stock is value-absent, which means on.

#### How it works

When you drag a file, Windows 11 shows a share surface at the top of the screen for dropping the file onto nearby devices or apps. It was introduced as Drag Tray and renamed Drop Tray at builds 26100.8328 (24H2), 26200.8328 (25H2) and 28000.2179 (26H1), when its Settings home also moved from System > Nearby sharing to System > Multitasking; a Settings toggle has existed since 26100.7309 and 26200.7309. The backing value lives under `CurrentVersion\CDP`, the Connected Devices Platform key, not under any Explorer key: `0` is off, `1` or absent is on. It first shipped to the 24H2 general channel at 26100.4202 (with an earlier Dev-channel debut at 26200.5518), so on 24H2 revisions below that there is no Drop Tray to turn off. A search of 14,760 shipped binaries on 26100.4061 found no `DragTray` or `DropTray` string at all, which is consistent with that build predating the feature, and the `CDP` key there held its established values and no `DragTrayEnabled`. One community page suggests a feature-management override under `HKLM\SYSTEM\ControlSet001\Control\FeatureManagement\Overrides` instead; that is an unrelated, build-specific mechanism and is not used.

#### Benefits
- **No surprise overlay**: dragging files between folders stops summoning a share panel.
- **Fewer misdrops**: the overlay can intercept a drag aimed at a window behind it.
- **Ordinary drag and drop unaffected**.

#### Drawbacks
- **Loses quick sharing**: the drag-to-share route to nearby devices and apps is gone.
- **Nothing to disable on older servicing**: 24H2 below 26100.4202 has no Drop Tray.
- **Drift**: the Settings toggle writes the same value.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 from 26100.4202 and 25H2, every edition; the gate is build 26100 and newer because the app cannot gate on a servicing revision.
- **Takes effect**: immediately.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
`services:disable_cdpsvc` disables the Connected Devices Platform service behind Nearby Sharing, a heavier way to lose the same sharing targets.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: the feature is now called Drop Tray, and one proposed source described an unrelated feature-management override and is not counted.
- **Confidence**: Community-corroborated: three independent sources carry the identical key and value, one of them firsthand and confirming the value-absent default; a fourth partially independent source agrees.
- **Reasoning**: the main doubt, the unusual `CDP` key, was attacked and survived three-source agreement. The negative binary search does not refute the tweak because the inspected build predates the feature.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you drag files between Explorer windows often and the overlay gets in the way. Skip it if you share files to nearby devices by dragging.

#### Sources
1. Win11Debloat `Regfiles/Disable_Share_Drag_Tray.reg`, the key and value (tier C)
2. Enable or Disable Drop Tray in Windows 11, ElevenForum tutorial 40485 (via the Wayback Machine), the on and off bodies plus the rename and Settings relocation, https://www.elevenforum.com/t/enable-or-disable-drag-tray-in-windows-11.40485/ (tier C)
3. How To Disable Drag Tray, MajorGeeks, firsthand, the value had to be created, so stock is value-absent, https://www.majorgeeks.com/content/page/how_to_disable_drag_tray.html (tier C)
4. AskVG, the same key, value and polarity, and the Dev-channel build 26200.5518 (tier C)
5. Negative binary search across 14,760 shipped binaries on 26100.4061, consistent with the feature shipping at 26100.4202 (tier A, primary measurement)

### Alt+Tab shows windows only

`alt_tab_hide_browser_tabs` · Dropdown (5 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Alt+Tab lists your open windows again instead of filling up with browser tabs.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `alt_tab_filter` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `MultiTaskingAltTabFilter`, REG_DWORD |

| Option | `alt_tab_filter` |
|---|---|
| Open windows only | `4` |
| Windows and 3 most recent tabs | `3` |
| Windows and 5 most recent tabs | `2` |
| Windows and 20 most recent tabs | `1` |
| Windows decides | `absent` |

"Windows decides" is the unconfigured policy, which is also the stock state; System Default is shown only when the value holds a number outside 1 to 4. Selecting it restores the snapshot.

#### How it works

The shipped `Multitasking.admx` on 26100 defines the policy `BrowserAltTabBlowout` (User class, key `Software\Policies\Microsoft\Windows\Explorer`, supported from Windows 10 version 2004) with an enum on `MultiTaskingAltTabFilter`: `1` = open windows and all (20) tabs, `2` = five tabs, `3` = three tabs, `4` = "Open windows only". The ADML's help text: "If this is set to show 'Open windows only', the whole feature will be disabled." The same value name also exists as a user preference under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, which the Settings Multitasking page writes, with a different, 0-based numbering: `0` = 20 tabs, `1` = five, `2` = three, `3` = windows only. Copying a number across the two keys produces the wrong result: `3` at the policy key means "windows and 3 recent tabs", the opposite of windows only. This tweak writes only the policy key, with the policy numbering. The value is read by `twinui.dll` (which implements Alt+Tab), `SettingsHandlers_nt.dll` and `SHCore.dll` on 26100. The ADML says "app tabs" generically, so any app that feeds tabs into Alt+Tab is affected, not only one browser.

#### Benefits
- **Short, predictable list**: one entry per window.
- **Faster switching**: no scanning past tab thumbnails to find an application.
- **Documented policy** with a named enum and graded options.

#### Drawbacks
- **Loses tab switching**: a specific browser tab can no longer be reached from Alt+Tab.
- **All tabbed apps**: not limited to one browser.
- **Two keys, two numberings**: a hand edit copied from the wrong guide sets the wrong option.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition (policy declared from Windows 10 2004). Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after Explorer restarts or sign-out; the ADML states no reboot requirement, and whether `twinui.dll` caches the value per session is unconfirmed.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
The Settings Multitasking option writes the preference-key twin. Other tweaks write different values under the same policy key ([Hide the Recommended section in Start](#hide-the-recommended-section-in-start), [Show the full date and time in the tray](#show-the-full-date-and-time-in-the-tray), [Hide recently added apps in Start](#hide-recently-added-apps-in-start), `debloat:disable_web_search_start`); per-value snapshots keep them apart.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: the policy key and the preference key use different enum bases, so windows-only is `4` here; applicability starts at Windows 10 2004; and the scope is all app tabs.
- **Confidence**: Microsoft-documented: the shipped ADMX and ADML carry the enum and help text; binary evidence confirms the consumers. Community evidence for the 0-based preference enum is one project's four sibling files.
- **Reasoning**: the adversarial pass caught the enum split and struck two of the originally claimed community sources that did not contain the value. The tier A ADMX carries the tweak.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Choose "Open windows only" if you keep many tabs open and Alt+Tab has become useless for finding applications. Skip it if you switch to individual tabs with Alt+Tab.

#### Sources
1. Shipped `Multitasking.admx` and `en-US\Multitasking.adml` on 26100: policy `BrowserAltTabBlowout`, User class, the 1 to 4 enum and its display strings (tier A)
2. Binary string evidence on 26100: `MultiTaskingAltTabFilter` in `twinui.dll`, `SettingsHandlers_nt.dll` and `SHCore.dll` (tier A, product artifact)
3. Win11Debloat `Hide_Tabs_In_Alt_Tab.reg`, `Show_3_Tabs_In_Alt_Tab.reg`, `Show_5_Tabs_In_Alt_Tab.reg`, `Show_20_Tabs_In_Alt_Tab.reg`, the 0-based preference enum under `Explorer\Advanced` (tier C, one source)

### Turn off the Snap Assist suggestion picker

`disable_snap_assist` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Stops Windows suggesting what to put in the empty half after you snap a window.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `snap_assist` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `SnapAssist`, REG_DWORD |

| Option | `snap_assist` |
|---|---|
| Off | `0` |
| On | `1` |

System Default is shown when the value is missing or holds another number; selecting it restores the snapshot. Snap Assist is on by default; whether a stock profile stores `1` or leaves the value absent could not be established, so an untouched machine may read System Default with the picker on.

#### How it works

`SnapAssist` is the Settings > System > Multitasking checkbox "When I snap a window, show what I can snap next to it". After you snap a window to one side, Windows normally fills the other side with thumbnails of your other windows to pick from; at `0` that picker no longer appears and the other half keeps showing whatever was there. Snapping itself (Win+Arrow, drag-to-edge, the layout grid) is unaffected. The value is present on 26100 in `twinui.dll` and `twinui.pcshell.dll` (the components that implement snapping), `Taskbar.View.dll` and `SettingsHandlers_nt.dll`, in the same string block as `SnapFill` and `JointResize`. It is one of three independent snap helpers; see [Turn off the snap layouts hover flyout](#turn-off-the-snap-layouts-hover-flyout).

#### Benefits
- **No interruption after snapping**: the other half keeps showing your desktop or the window already there.
- **Snapping still works**: Win+Arrow, drag-to-edge and the layout grid are untouched.
- **Settings-backed**: verifiable by snapping a window.

#### Drawbacks
- **Slower two-window setups**: you pick the second window yourself.
- **Loses a discovery aid**: new users learn snapping partly through this picker.
- **One of three snap switches**: the hover flyout and the drag-to-top layout bar are separate settings.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); hidden on Windows 10 LTSC 2021.
- **Takes effect**: the next time you snap a window.
- **Reverting**: restores the captured value.

#### Interactions
[Turn off the snap layouts hover flyout](#turn-off-the-snap-layouts-hover-flyout) covers the hover flyout (`EnableSnapAssistFlyout`), a different surface. The drag-to-top bar (`EnableSnapBar`) is not covered.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Community-corroborated: two independent projects agree on key, name, type and polarity, backed by binary presence (tier A) and the Settings mapping.
- **Reasoning**: the adversarial pass struck one originally cited source that did not contain the value; two projects plus binary evidence and a self-verifying Settings toggle still suffice. Open question: whether stock is `1` or absent; the behaviour is the same either way, so a revert is not harmful.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you snap windows constantly and always know what goes beside them. Skip it if you routinely build side-by-side layouts, where the picker saves a step.

#### Sources
1. Sophia Script `Sophia.psm1`, writes `SnapAssist` as a DWord at this key with values 0 and 1 (tier C)
2. Win11Debloat `Regfiles/Disable_Snap_Assist.reg`, `"SnapAssist"=dword:00000000` (tier C)
3. Binary string evidence on 26100: `SnapAssist` in `twinui.dll`, `twinui.pcshell.dll`, `Taskbar.View.dll` and `SettingsHandlers_nt.dll` (tier A, product artifact)

### Expand the tree to the open folder

`explorer_expand_to_current_folder` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**The Explorer sidebar tree opens and highlights whatever folder you are currently viewing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `expand_to_folder` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `NavPaneExpandToCurrentFolder`, REG_DWORD |

| Option | `expand_to_folder` |
|---|---|
| Expand | `1` |
| Do not expand | `absent` |

System Default is shown when the value is present with anything other than `1`, for example `0` written by Folder Options; selecting it restores the snapshot. Stock is value-absent, with the tree not expanding.

#### How it works

`NavPaneExpandToCurrentFolder` is the Folder Options > View checkbox "Expand to open folder" (also in the navigation pane's context menu). At `1` the navigation pane expands the branch containing the folder shown in the file list and scrolls it into view every time you change folders. The value name is present on 26100 in `shell32.dll`, `ExplorerFrame.dll` and `Windows.UI.FileExplorer.dll`, all Explorer consumers.

#### Benefits
- **Always know where you are**: the tree matches the file list.
- **Faster sideways navigation**: sibling folders are one click away.
- **Officially exposed**: a Folder Options checkbox.

#### Drawbacks
- **Long trees**: deep paths expand many nodes and push the rest off screen.
- **Constant scrolling**: the pane jumps whenever you change folders.
- **Slow on network shares**: expanding a branch enumerates each level on the way.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Community-corroborated: one project writes exactly this value, with binary presence (tier A) and the Folder Options checkbox making the mapping self-verifying.
- **Reasoning**: key, name, type and polarity confirmed in the adversarial pass; nothing needed correcting.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you navigate by the folder tree and keep losing your place. Skip it for deeply nested paths or slow network shares.

#### Sources
1. Sophia Script `Sophia.psm1`, writes `NavPaneExpandToCurrentFolder` at this key with values 0 and 1 (tier C)
2. Binary string evidence on 26100: the value name in `shell32.dll`, `ExplorerFrame.dll` and `Windows.UI.FileExplorer.dll` (tier A, product artifact)
3. Folder Options > View > "Expand to open folder", the UI surface this value backs (tier A, product artifact)

### Restore Explorer windows at sign-in

`explorer_restore_folders_at_logon` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Reopens the File Explorer windows you had open when you last signed out.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `persist_browsers` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `PersistBrowsers`, REG_DWORD |

| Option | `persist_browsers` |
|---|---|
| Restore | `1` |
| Do not restore | `absent` |

System Default is shown when the value is present with anything other than `1`, for example `0` written by Folder Options; selecting it restores the snapshot. Stock is value-absent, with no windows restored.

#### How it works

`PersistBrowsers` is the Folder Options > View checkbox "Restore previous folder windows at logon". At `1` Explorer records which folder windows are open when you sign out and reopens them at the next sign-in. The value name is present on 26100 in `shell32.dll`, `ExplorerFrame.dll` and `gpprefcl.dll`.

#### Benefits
- **Resume where you left off**: working folders come back without retyping paths.
- **Survives restarts**: useful on a machine that reboots for updates overnight.
- **Officially exposed**: a Folder Options checkbox.

#### Drawbacks
- **Slower sign-in**: several windows open before the desktop settles.
- **Reveals your work**: previous folder names appear at sign-in, awkward on a shared or projected display.
- **Stale windows**: folders on disconnected shares or removed drives fail to reopen.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: at the next sign-out and sign-in pair, which is when Explorer records and restores the windows.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Community-corroborated: one project writes exactly this value, with binary presence (tier A) and the Folder Options checkbox.
- **Reasoning**: key, name, type and polarity confirmed in the adversarial pass; nothing needed correcting.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a personal workstation where you keep a fixed set of folders open. Skip it on a shared machine or one whose screen others see.

#### Sources
1. Sophia Script `Sophia.psm1`, writes `PersistBrowsers` at this key with values 0 and 1 (tier C)
2. Binary string evidence on 26100: the value name in `shell32.dll`, `ExplorerFrame.dll` and `gpprefcl.dll` (tier A, product artifact)
3. Folder Options > View > "Restore previous folder windows at logon", the UI surface this value backs (tier A, product artifact)

### Show the full date and time in the tray

`taskbar_full_date_time` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: build 22621 and newer · Reversible: yes

**Shows the full date with the year and an AM/PM marker in the taskbar clock.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `full_date_time` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `TurnOffAbbreviatedDateTimeFormat`, REG_DWORD |

| Option | `full_date_time` |
|---|---|
| Full format | `1` |
| Abbreviated | `absent` |

System Default is shown when the value is present with anything other than `1`, for example the policy's explicit disabled value `0`; selecting it restores the snapshot. Stock is value-absent, the unconfigured policy.

#### How it works

The shipped `Taskbar.admx` on 26100 defines this as a User-class policy at `Software\Policies\Microsoft\Windows\Explorer`, enabled value `1`, disabled value `0`, supported from Windows 11 22H2. Its ADML text: "This policy setting allows you to show the longer time and date format in the system tray. If this setting is enabled, the time format will include the AM/PM time marker and the date will include the year. A reboot is required for this policy setting to take effect." The Windows 11 tray clock reads it: the name appears twice in `Taskbar.View.dll` and in `SettingsHandlers_DesktopTaskbar.dll` on 26100. The same ADMX also offers a sibling policy, `AlwaysShowNotificationIcon` (always show the notification bell), which is not shipped.

#### Benefits
- **Unambiguous time**: no guessing whether 7:15 is morning or evening on a 12-hour clock.
- **Year visible**: useful on machines whose clock drifts or that dual-boot.
- **Documented policy** with explicit enabled and disabled values.

#### Drawbacks
- **Wider clock**: the tray takes noticeably more room.
- **Needs a reboot**: nothing changes until you restart, which looks like a failed apply if you do not know.
- **Windows 11 only**: declared from 22H2.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 build 22621 and newer, every edition; hidden on Windows 10 LTSC 2021. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after a reboot, as the ADML states.
- **Reverting**: restores the captured state, normally removing the value; also needs a reboot.

#### Interactions
[Show seconds in the tray clock](#show-seconds-in-the-tray-clock) changes the same clock through `ShowSecondsInSystemClock`, a different key and value; the two combine. Other tweaks write different values under the same policy key; per-value snapshots keep them apart.

#### Validation
- **Verdict**: VERIFIED. The research required the reboot requirement to be stated, and it is.
- **Confidence**: Microsoft-documented: the shipped ADMX and ADML, with binary evidence that the tray clock reads the value.
- **Reasoning**: key, name, type, class and applicability confirmed in the adversarial pass.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use a 12-hour clock and keep misreading the tray. On a narrow laptop taskbar the extra width is a real cost.

#### Sources
1. Shipped `Taskbar.admx` and `en-US\Taskbar.adml` on 26100: User class, key `Software\Policies\Microsoft\Windows\Explorer`, enabled 1, disabled 0, `SUPPORTED_Windows_11_0_22H2`, and the reboot sentence (tier A)
2. Binary string evidence on 26100: `TurnOffAbbreviatedDateTimeFormat` in `Taskbar.View.dll` and `SettingsHandlers_DesktopTaskbar.dll` (tier A, product artifact)

### Hide recently added apps in Start

`hide_recently_added_apps` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes the "Recently added" list of newly installed apps from the Start menu.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `hide_recently_added` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `HideRecentlyAddedApps`, REG_DWORD |

| Option | `hide_recently_added` |
|---|---|
| Hidden | `1` |
| Shown | `absent` |

System Default is shown when the value is present with anything other than `1`; selecting it restores the snapshot. Stock is value-absent, the unconfigured policy.

#### How it works

The shipped `StartMenu.admx` on 26100 defines `HideRecentlyAddedApps` with `class="Both"` at `Software\Policies\Microsoft\Windows\Explorer`, with no explicit enabled value, which under the ADMX schema means DWORD `1` enabled and `0` disabled. The ADML: "Remove 'Recently added' list from Start Menu ... The corresponding setting will also be disabled in Settings." The Policy CSP adds that the policy applies from Windows 10 version 1703 (the ADMX's own `supportedOn` says 1803; the CSP is used), lists Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC editions with Device and User scope, states that "This policy requires a reboot to take effect", and writes its validation steps against the Windows 11 Settings toggle "Show recently added apps", which is greyed out while the policy is set. On 26100 the name appears only in `StartTileData.dll`, the Start app-list data model, consistent with it owning that list.

#### Benefits
- **Stable Start menu**: the list stops reshuffling every time you install something.
- **Fewer surprises**: apps installed by an updater or bundled installer do not announce themselves at the top of Start.
- **Documented policy**: defined in the shipped ADMX and the Policy CSP.

#### Drawbacks
- **Harder to find new apps**: you launch a freshly installed program by searching.
- **Settings toggle locked**: the matching switch is greyed out while the policy is set, which can look like a fault.
- **Needs a reboot**, per Microsoft.
- **Not on Home**: the CSP's edition list does not include Home.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build on Pro, Enterprise, Education and IoT Enterprise editions, including Windows 10 IoT Enterprise LTSC 2021. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after a reboot.
- **Reverting**: restores the captured state, normally removing the value, which also unlocks the Settings toggle; needs a reboot.

#### Interactions
[Hide the Recommended section in Start](#hide-the-recommended-section-in-start), [Turn off promoted recommendations in Start](#turn-off-promoted-recommendations-in-start) and [Turn off recent items tracking](#turn-off-recent-items-tracking) clear other parts of the Start menu; `Start_IrisRecommendations` alone does not stop newly installed apps appearing. Other tweaks write different values under the same policy key; per-value snapshots keep them apart.

#### Validation
- **Verdict**: VERIFIED. The research corrected applicability to 1703 (CSP) rather than 1803 (ADMX) and required the reboot requirement, which the tweak states.
- **Confidence**: Microsoft-documented: the shipped ADMX and ADML, the Policy CSP, and binary evidence.
- **Reasoning**: the concern that this is a Windows 10-only surface was settled by the CSP's Windows 11 validation steps.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want a Start menu that stays where you put it, particularly where software is installed often. Skip it if you launch new programs from the Recently added list.

#### Sources
1. Shipped `StartMenu.admx` and `en-US\StartMenu.adml` on 26100: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, no explicit enabled value, `SUPPORTED_Windows_10_0_RS4` (tier A)
2. Policy CSP - Start > HideRecentlyAddedApps, applicability from 1703, editions, scope, reboot requirement and Windows 11 validation steps, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start (tier A)
3. Binary string evidence on 26100: `HideRecentlyAddedApps` in `StartTileData.dll` (tier A, product artifact)

### Focus the last active window on click

`taskbar_last_active_click` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Clicking a grouped taskbar button jumps straight to the window you used last.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `last_active_click` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `LastActiveClick`, REG_DWORD |

| Option | `last_active_click` |
|---|---|
| Focus the last active window | `1` |
| Show the thumbnail chooser | `absent` |

System Default is shown when the value is present with anything other than `1`; selecting it restores the snapshot. Stock is value-absent, with a click on a grouped button showing the thumbnail chooser.

#### How it works

When several windows of one app share a taskbar button, a click normally opens the thumbnail chooser. With `LastActiveClick = 1` the click switches straight to the most recently active of those windows; hovering the button still shows the previews for picking another. The value name appears twice in `Taskbar.View.dll` and in `Taskbar.dll` on 26100, the Windows 11 taskbar itself rather than policy plumbing, which is the decisive evidence that the shipping taskbar reads it. Microsoft publishes no reference for it.

#### Benefits
- **One click instead of two** for the window you almost always want.
- **Previews still available**: hover to pick a different window.
- **Read by the shipping taskbar**: live in the Windows 11 taskbar binaries.

#### Drawbacks
- **Wrong window sometimes**: for a different one you now hover and pick.
- **Undocumented**: no Microsoft reference.
- **Per user only**.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (all supported builds, every edition); hidden on Windows 10 LTSC 2021.
- **Takes effect**: after Explorer restarts.
- **Reverting**: restores the captured state, normally removing the value.

#### Interactions
Only matters while buttons are combined; with "Never combine" from [Ungroup taskbar buttons](#ungroup-taskbar-buttons) each window has its own button and this has little to do.

#### Validation
- **Verdict**: VERIFIED.
- **Confidence**: Community-corroborated: one community project plus binary presence in the taskbar itself (tier A), with self-verifying behaviour.
- **Reasoning**: the adversarial pass struck an originally cited source that did not contain the value, leaving one community source; it judged binary presence in the consumer stronger than more community sources for "does this value exist and get read", and the behaviour is visible with one click.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you keep several windows of one app open and mostly return to the last one. Skip it if you regularly move between many windows of one app, where the chooser is faster.

#### Sources
1. Binary string evidence on 26100: `LastActiveClick` in `Taskbar.View.dll` and `Taskbar.dll` (tier A, product artifact)
2. Win11Debloat `Regfiles/Enable_Last_Active_Click.reg`, `"LastActiveClick"=dword:00000001` with an inline description of the behaviour (tier C)

### Remove the Notification Center

`disable_notification_center` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes the Notification Center panel and its taskbar entry point entirely.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `notification_center` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`, value `DisableNotificationCenter`, REG_DWORD |

| Option | `notification_center` |
|---|---|
| Removed | `1` |
| Shown | `absent` |

System Default is shown when the value is present with anything other than `1`, for example the policy's disabled value `0`; selecting it restores the snapshot. Stock is value-absent.

#### How it works

The shipped `Taskbar.admx` on 26100 defines `DisableNotificationCenter` with `class="Both"` at `Software\Policies\Microsoft\Windows\Explorer`, enabled `1`, disabled `0`, supported from Windows 10; because the class is Both, the tweak uses the HKLM hive so the change applies to every account. The ADML: "This policy setting removes Notifications and Action Center from the notification area on the taskbar ... The user will be able to read notifications when they appear, but they won't be able to review any notifications they miss", and it states a reboot is required. The value is read on 26100 by `Windows.UI.ActionCenter.dll`, `Taskbar.View.dll` and `twinui.pcshell.dll`, the components that render the notification centre, which confirms it still works on the primary target. On Windows 11 the calendar shares the same flyout, so it is likely to go too, but the policy text does not say so; treat that as an inference.

#### Benefits
- **No history panel**: nothing accumulates for anyone to scroll through later.
- **Reclaims the tray entry**: the notification button leaves the taskbar corner.
- **Documented policy**, machine-wide.

#### Drawbacks
- **Missed notifications are lost**: there is no way to review anything you did not catch on screen.
- **May take the calendar with it**: likely on Windows 11, not documented.
- **Needs a reboot**.
- **Heavier than Do Not Disturb**: if you only want quiet, Focus keeps the history.
- **Needs admin** and applies to every account.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition.
- **Takes effect**: after a reboot.
- **Reverting**: restores the captured state, normally removing the value; needs a reboot.

#### Interactions
[Turn off toast notifications](#turn-off-toast-notifications) is the opposite trade-off: it silences the pop-ups but keeps nothing to review. They are adjacent, not interchangeable. Other tweaks write values under the HKCU twin of this policy key; this one writes HKLM.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: the policy class is Both, so the machine-wide HKLM hive is used; a reboot is required; and calendar removal is an inference, not documented.
- **Confidence**: Microsoft-documented: the shipped ADMX and ADML, with the strongest consumer binary evidence in the set.
- **Reasoning**: the adversarial pass confirmed the mechanism, moved the write to HKLM, added the reboot requirement and struck the calendar claim as fact. Risk is medium because losing notification history is a larger cost than it sounds.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a kiosk, a signage machine or a locked-down workstation where a notification history is unwanted. For a normal desktop, use Do Not Disturb instead.

#### Sources
1. Shipped `Taskbar.admx` and `en-US\Taskbar.adml` on 26100: `class="Both"`, key `Software\Policies\Microsoft\Windows\Explorer`, enabled 1, disabled 0, `SUPPORTED_Windows_10_0`, the reboot statement and the "won't be able to review any notifications they miss" wording (tier A)
2. Binary string evidence on 26100: `DisableNotificationCenter` in `Windows.UI.ActionCenter.dll`, `Taskbar.View.dll` and `twinui.pcshell.dll` (tier A, product artifact)
3. Chris Titus WinUtil `config/tweaks.json`, contains `DisableNotificationCenter` (tier C)

## Considered and not shipped

### `disable_copilot_taskbar` (moved to AI & Copilot)

The Copilot taskbar button tweak (`TurnOffWindowsCopilot` under `HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot`) now lives in the AI & Copilot category; see [the AI category page](ai.md).

### `disable_chat_taskbar` (Chat button, removed from Windows)

Wrote `TaskbarMn` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, gated to Windows 11, to hide the Teams Chat button. Microsoft removed Chat from the in-box Teams app and stopped pinning Teams to the taskbar in Windows 11 23H2, one release below the support floor, so on every supported build the value hides nothing. The backing `ConfigureChatIcon` policy is also marked deprecated in the Policy CSP. It never applied to Windows 10. It is not shipped in any category; the matching debloat tweak `remove_teams_chat_taskbar` was dropped for the same reason.

Sources: What's new in Windows 11, version 23H2, https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2 (tier A); Policy CSP - Experience, `ConfigureChatIcon` deprecation note, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A); What's new with taskbar and Start menu on Windows 11 2023 update, https://www.windowscentral.com/software-apps/windows-11/whats-new-with-taskbar-and-start-menu-on-windows-11-2023-update (tier C); The Chat taskbar button in Windows 11 is going away, https://www.howtogeek.com/898705/the-chat-taskbar-button-in-windows-11-is-going-away/ (tier C).

### `disable_cortana_button` (Cortana button, no supported platform)

Wrote `ShowCortanaButton` under `Explorer\Advanced`, gated to Windows 10. Windows 11 never pinned Cortana to the taskbar and does not read the value. Cortana as a standalone app is deprecated, and its voice assistance was retired in spring 2023. On Windows 10 21H2, the LTSC 2021 baseline, Cortana is a separate Store app (`Microsoft.549981C3F5F10`), and LTSC editions ship no Store and no bundled Store apps, so the button does not exist on a stock LTSC 2021 image. The value name still appears in one 26100 binary (`windowsudk.shellcommon.dll`), which is why the mechanism itself checked out, but no supported platform has a button to hide.

Sources: Deprecated features for Windows client, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); End of support for Cortana, https://support.microsoft.com/en-us/topic/end-of-support-for-cortana-d025b39f-ee5b-4836-a954-0ab646ee1efa (tier A); Windows as a service overview, the LTSC app exclusions, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A).

### `disable_meet_now` (Meet Now, service retired)

Wrote the `HideSCAMeetNow` policy under `HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`, gated to Windows 10. The mechanism was real and correctly placed (a User-class policy in `Taskbar.admx`, written to HKCU). Meet Now is a Skype entry point added to the Windows 10 taskbar with the October 2020 update, and Skype was retired on 5 May 2025, so the icon points at a service that no longer exists. Windows 11 has no Meet Now control, and LTSC 2021 does not receive general-channel shell feature rollouts. Whether the icon ever rendered on LTSC 2021 was not verified, which is moot given the retirement.

Sources: How do I use Skype's Meet Now from my Windows 10 taskbar, Microsoft support, the Skype retirement, https://support.microsoft.com/en-us/skype/how-do-i-use-skype-s-meet-now-from-my-windows-10-taskbar-or-outlook-com-3bdebac0-3008-4c28-bdc8-6253d8680f40 (tier A); Windows LTSC overview, in-box features not included, https://learn.microsoft.com/en-us/windows/whats-new/ltsc/overview (tier A).

### `disable_news_interests` (News and Interests, no supported platform)

Wrote `ShellFeedsTaskbarViewMode` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Feeds`, gated to Windows 10. Key, value, type and the 0/1/2 enum were correct. News and Interests reached general-channel Windows 10 through cumulative updates, which LTSC does not receive as feature payloads, and the News app is on Microsoft's LTSC exclusion list; Windows 11 uses Widgets instead, which `debloat:disable_widgets` covers through the machine-wide `AllowNewsAndInterests` policy. Two further problems: since the March 2024 servicing updates the User Choice Protection Driver watches the `Feeds` key and reverts writes from processes it does not allow, so the value might not stick; and the durable control was always the `EnableFeeds` machine policy under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Feeds`. Microsoft publishes no per-edition statement for News and Interests, and a few forum reports claim it appeared on LTSC after cumulative updates; the research recommended confirming its absence on a real LTSC 2021 image before final removal.

Sources: Windows LTSC overview, https://learn.microsoft.com/en-us/windows/whats-new/ltsc/overview (tier A); Windows as a service overview, the LTSC app exclusions including News, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A).

### `disable_people_bar` (My People, deprecated and absent)

Wrote `PeopleBand` under `Explorer\Advanced\People`, gated to Windows 10; the subkey was checked and correct. My People is on Microsoft's deprecated-features list ("no longer being developed"), does not exist on Windows 11, and the People app belongs to the Mail and Calendar family that Microsoft excludes from LTSC editions. `PeopleBand.dll` still ships on 26100 and the feature was never moved to the removed-features list, so it is deprecated rather than formally removed; the research recommended confirming on a clean LTSC 2021 image that the People button does not appear before final removal.

Sources: Deprecated features for Windows client, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); Windows as a service overview, the LTSC app exclusions, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A).

### Rejected at proposal

Two further interface controls were proposed during the gap hunt and rejected. `disable_widgets_lock_screen` was dropped because the shipped `NewsAndInterests.admx` and the Policy CSP prescribe opposite values for the same intent, systematically across sibling policies, so neither can be called a typo, and the CSP lists the policy as Insider Preview only; writing the wrong one would leave the surface on while reporting success. `disable_new_app_alert` (`NoNewAppAlert`, `WindowsExplorer.admx`) was dropped because on 26100 it appears only in the shell policy table and in no feature component, so its effect is unproven, and the notification it suppresses is the only signal that an installer changed your file or protocol associations, a known adware vector.

