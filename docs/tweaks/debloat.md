# Debloat & Consumer tweaks

This category removes preinstalled consumer apps and turns off Microsoft's promotional surfaces: Start menu suggestions, silently installed sponsored apps, post-update and post-sign-in nag screens, Spotlight content, account upsells, advertising toasts, Widgets, web results in Start search, and three Microsoft Edge nuisances. The primary platform is Windows 11 24H2 (build 26100) and newer, including 25H2 (26200); the secondary platform is Windows 10 IoT Enterprise LTSC 2021 (build 19044), where every Store app removal is inert because LTSC images ship no Store app set and no Microsoft Store.

Two things apply across the whole page. First, every app removal writes a small state marker under `HKCU\Software\MagicXToolbox\Debloat` alongside a PowerShell action that removes the app for all users and removes its provisioned (image) copy, and the app decides whether the app is gone with a shared, fail-closed package enumeration (`Get-AppxPackage -AllUsers` plus `Get-AppxProvisionedPackage -Online`): that enumeration needs administrator rights, so while the app runs unelevated those tweaks read as Unknown rather than guessing, and a failed enumeration is reported as "cannot tell", never as "removed". Second, per-user (HKCU) effects always run in-process as the signed-in user even inside an `admin` tweak, so they land in your hive and not the elevated account's; if a different account's credentials were used to elevate the app, every tweak that touches HKCU is disabled as "Different account" instead of writing the wrong hive.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Turn off Start menu app promotions](#turn-off-start-menu-app-promotions) | `disable_start_suggestions` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off auto-installed sponsored apps](#turn-off-auto-installed-sponsored-apps) | `disable_auto_install_sponsored_apps` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off the post-update welcome experience](#turn-off-the-post-update-welcome-experience) | `disable_welcome_experience` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off the 'Get even more out of Windows' nag](#turn-off-the-get-even-more-out-of-windows-nag) | `disable_scoobe_nag` | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off File Explorer sync-provider ads](#turn-off-file-explorer-sync-provider-ads) | `disable_explorer_sync_ads` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off web results in Start search](#turn-off-web-results-in-start-search) | `disable_web_search_start` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off Widgets](#turn-off-widgets) | `disable_widgets` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Turn off Microsoft account nags in Start](#turn-off-microsoft-account-nags-in-start) | `disable_account_notifications` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off account upsell cards in Settings](#turn-off-account-upsell-cards-in-settings) | `disable_settings_account_ads` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off all Windows Spotlight features](#turn-off-all-windows-spotlight-features) | `disable_windows_spotlight_all` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off Spotlight desktop wallpaper](#turn-off-spotlight-desktop-wallpaper) | `disable_spotlight_desktop` | Switch (2 options) | low | admin | no | VERIFIED |
| [Silence suggested and backup reminder toasts](#silence-suggested-and-backup-reminder-toasts) | `disable_nag_toasts` | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off the Edge first-run experience](#turn-off-the-edge-first-run-experience) | `disable_edge_first_run` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off Edge startup boost](#turn-off-edge-startup-boost) | `disable_edge_startup_boost` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off the Edge sidebar and Collections](#turn-off-the-edge-sidebar-and-collections) | `disable_edge_sidebar` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the Microsoft Teams app](#remove-the-microsoft-teams-app) | `remove_teams_consumer_app` | Switch (2 options) | low | admin | no | INCORRECT (corrected form ships) |
| [Remove Clipchamp](#remove-clipchamp) | `remove_clipchamp` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Quick Assist](#remove-quick-assist) | `remove_quick_assist` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Bing News and Weather](#remove-bing-news-and-weather) | `remove_bing_news_weather` | Switch (2 options) | low | admin | no | INCORRECT (corrected form ships) |
| [Remove Solitaire Collection](#remove-solitaire-collection) | `remove_solitaire` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the Get Help app](#remove-the-get-help-app) | `remove_get_help` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Tips (Get Started)](#remove-tips-get-started) | `remove_getstarted_tips` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Feedback Hub](#remove-feedback-hub) | `remove_feedback_hub` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Phone Link](#remove-phone-link) | `remove_phone_link` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the New Outlook app](#remove-the-new-outlook-app) | `remove_outlook_new` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Xbox Game Bar](#remove-xbox-game-bar) | `remove_xbox_game_bar` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove OneDrive](#remove-onedrive) | `remove_onedrive` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |

Every tweak here authors two options, so each shows as a dropdown with a third, computed "System Default" position.

## Tweaks

### Turn off Start menu app promotions

`disable_start_suggestions` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Clears promoted-app and tip cards out of the Start menu's suggestion slot.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `start_sug` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `SubscribedContent-338388Enabled`, `REG_DWORD` |

| Option | `start_sug` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is any state that matches neither option, for example an explicit `1` written by Settings or another tool; selecting it restores the snapshot taken before the tweak was first applied. A fresh profile carries no value at all, so a stock machine reads as "On".

#### How it works

`ContentDeliveryManager` is the per-user backing store for the Windows Spotlight and suggestion surfaces, and each `SubscribedContent-<ID>Enabled` value gates one content slot. The `SubscribedContent-` prefix is a literal in the shipped `ContentDeliveryManager.Utilities.dll` and `ContentDeliveryManager.Background.dll` on build 26100.4061; the numeric ID is appended at runtime from the subscription identifier, which is why no individual ID appears in any binary or Microsoft document and why the ID-to-surface mapping is known only empirically. Four independent community sources agree on the key, value name, `REG_DWORD` type and polarity (0 off, 1 on) for 338388, and all four map it to the Windows 10 Start toggle "Occasionally show suggestions in Start". On Windows 11 the visually similar "Show recommendations for tips, shortcuts, new apps, and more" row in Start is a different control: it is driven by `Start_IrisRecommendations` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` (a literal in the shipped `StartTileData.dll`) and by the `HideRecommendedSection` policy in `StartMenu.admx`. This tweak writes neither. It is a per-user preference value, not a policy, so it works on every edition and needs no elevation.

#### Benefits
- Closes the Start suggestion slot so it stops surfacing apps you never asked for.
- One less Microsoft content channel pointed at Start.
- Per-user and instant: no policy, no elevation, no reboot.

#### Drawbacks
- Partial on Windows 11: the 24H2 recommendations row is driven by a different value, so this alone may produce no visible change.
- Undocumented slot: Microsoft composes these IDs at runtime and can move a surface between releases without notice. Sophia Script, which tracks Windows 11 25H2, has dropped 338388 entirely in favour of `Start_IrisRecommendations`.
- On a machine where the slot is already empty there is nothing to see.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition; the key exists on Windows 10 1607 and later and on all Windows 11 builds.
- **Takes effect**: immediately; Start may need a sign-out to redraw.
- **Reverting**: selecting "On" deletes the value, returning the profile to its shipped state. Selecting System Default restores whatever the snapshot recorded before the first apply.

#### Interactions
- `interface:disable_start_recommendations` owns `Start_IrisRecommendations`, the Windows 11 recommendations row. Pair the two if that row is what bothers you.
- `interface:disable_start_recommended_section` sets the `HideRecommendedSection` policy, which hides the whole Recommended section.
- `privacy:disable_start_app_suggestions` writes `SystemPaneSuggestionsEnabled` under the same `ContentDeliveryManager` key; different value, no conflict.
- `disable_windows_spotlight_all` in this category is a policy master switch over Spotlight and consumer content.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The value is absent from a fresh profile, so the stock option deletes it rather than writing 1; and 338388 is the Windows 10 Start suggestion slot, not the Windows 11 recommendations row, which is a separate value this tweak does not write.
- **Confidence**: Community-corroborated (four independent tier C sources), plus shipped-binary evidence that the `SubscribedContent-` prefix is read by the 24H2 ContentDeliveryManager.
- **Reasoning**: key, name, type and polarity are uncontested. The open weakness is effect on Windows 11: every source maps 338388 to the Windows 10 wording, so on 24H2 the visible result may be nothing. The "absent on a fresh profile" finding came from `C:\Users\Default\NTUSER.DAT` on a test machine that the cross-cutting revert audit later flagged as modified (an LTSC image), so the stock-state claim still wants clean-image confirmation; `absent` is the conservative choice either way.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want every Start suggestion channel closed; it costs nothing. If your only complaint is the Windows 11 recommendations row, `interface:disable_start_recommendations` is the tweak that actually fixes that.

#### Sources
1. Shipped `StartMenu.admx` on build 26100.4061, policy `HideRecommendedSection`, key `Software\Policies\Microsoft\Windows\Explorer`: the Windows 11 surface is a separate control (tier A, shipped ADMX).
2. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: no `SubscribedContent-*Enabled` value present (tier A, primary observation; machine later flagged as modified).
3. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100.4061: `SubscribedContent-` in `ContentDeliveryManager.Utilities.dll` and `ContentDeliveryManager.Background.dll`; `Start_IrisRecommendations` in `StartTileData.dll` (tier A, primary observation).
4. Brink, "Turn On or Off App Suggestions in Start in Windows 10", TenForums tutorial 24117, key, value and polarity, https://www.tenforums.com/tutorials/24117-turn-off-app-suggestions-start-windows-10-a.html (tier C).
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, writes the same value, https://github.com/Raphire/Win11Debloat (tier C).
6. Sophia Script for Windows 11 v7.1.6, function `StartRecommendedSection`, which now uses `Start_IrisRecommendations`, https://github.com/farag2/Sophia-Script-for-Windows (tier C).
7. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C).
8. Disassembler0, Win10-Initial-Setup-Script, `DisableAppSuggestions`, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C).
9. Policy CSP - Experience, `AllowWindowsConsumerFeatures` and `AllowWindowsSpotlight`, the documented policy relatives, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### Turn off auto-installed sponsored apps

`disable_auto_install_sponsored_apps` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows from silently installing promoted, preinstalled and OEM apps into your account.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `silent` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `SilentInstalledAppsEnabled`, `REG_DWORD` |
| `preinstalled` | registry | same key, value `PreInstalledAppsEnabled`, `REG_DWORD` |
| `oem` | registry | same key, value `OemPreInstalledAppsEnabled`, `REG_DWORD` |

| Option | `silent` | `preinstalled` | `oem` |
|---|---|---|---|
| Off | `0` | `0` | `0` |
| On | `1` | `1` | `1` |

System Default is any mix that matches neither option (for example one value at 0 and the others at 1, or a value deleted); selecting it restores the snapshot. The shipped Default user hive seeds all three at 1, so a stock profile reads as "On".

#### How it works

These three values gate the silent post-setup installation of promoted Store apps (`SilentInstalledAppsEnabled`), the bundled preinstalled app set (`PreInstalledAppsEnabled`) and anything the OEM added (`OemPreInstalledAppsEnabled`) into the user's profile. ContentDeliveryManager reads them per user; `PreInstalledAppsEnabled` also appears as a literal in the shipped `StartTileData.dll`. All three are present with data 1 in the shipped `C:\Users\Default\NTUSER.DAT` examined on 26100.4061, the template new profiles are cloned from, which is why the stock option writes 1 rather than deleting. They only prevent future installs; apps already on the machine stay. The documented machine-wide equivalent is the `DisableWindowsConsumerFeatures` policy under `HKLM\Software\Policies\Microsoft\Windows\CloudContent`, which Microsoft describes as covering "Post-OOBE app install and redirect tiles", but that policy is honoured only on Enterprise, Education and IoT Enterprise; on Home and Pro this per-user key is the only lever. A sibling value, `ContentDeliveryAllowed`, is genuinely read by six shipped modules but is not part of this tweak.

#### Benefits
- Game and trial tiles stop appearing in Start on their own.
- Setting it early in a new account keeps that account clean.
- The snapshot restores the exact prior data for each value.

#### Drawbacks
- Future installs only: promotional apps already present must be uninstalled separately (see the removal tweaks below).
- Not enforced: a feature update can re-provision apps regardless of these per-user values.
- No policy backing on Home and Pro.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 1607 and later and all Windows 11 builds, all editions; per user.
- **Takes effect**: immediately, for any install that has not already started.
- **Reverting**: "On" writes the shipped value of 1 to all three. System Default restores the snapshot.

#### Interactions
- `privacy:disable_consumer_features` (`DisableWindowsConsumerFeatures`, HKLM policy) is the stronger machine-wide block, but only on Enterprise, Education and IoT Enterprise; it is a no-op on Home and Pro.
- `disable_windows_spotlight_all` covers "Microsoft consumer features" by policy and does not touch these values.
- The app removal tweaks in this category remove apps that are already installed.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. `SubscribedContentEnabled` (with no numeric ID) is not read by any shipped 24H2 module and is not seeded in the Default hive, so it is not part of the mechanism; the three values above are the real ones.
- **Confidence**: Community-corroborated (four independent tier C sources agree on key, names, type and polarity), with shipped-binary presence for `PreInstalledAppsEnabled`.
- **Reasoning**: a string scan of 4051 shipped modules found every sibling name in this key (`ContentDeliveryAllowed`, `SystemPaneSuggestionsEnabled`, `SoftLandingEnabled`, `RotatingLockScreenOverlayEnabled`, `PreInstalledAppsEnabled`) but `SubscribedContentEnabled` in zero, which is what removed it. The stock value of 1 rests on the Default hive of a test machine the revert audit later flagged as modified, so it is listed among the values that still want clean-image confirmation; if Windows actually ships them absent, writing 1 still leaves the documented default behaviour (installs allowed) in place.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it. There is no scenario where you want Windows quietly installing sponsored apps into your profile, and the only cost is installing wanted apps yourself.

#### Sources
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: the three values present as `REG_DWORD` 1; `SubscribedContentEnabled` absent (tier A, primary observation; machine later flagged as modified).
2. String scan of 4051 shipped modules on build 26100.4061: `SubscribedContentEnabled` in zero modules; `PreInstalledAppsEnabled` and `ContentDeliveryAllowed` in `StartTileData.dll` (tier A, primary observation).
3. Brink, "Turn Off Automatic Installation of Suggested Apps in Windows 10", TenForums tutorial 68217, https://www.tenforums.com/tutorials/68217-turn-off-automatic-installation-suggested-apps-windows-10-a.html (tier C).
4. Sophia Script for Windows 11 v7.1.6, function `AppsSilentInstalling`, https://github.com/farag2/Sophia-Script-for-Windows (tier C).
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C).
6. Disassembler0, Win10-Initial-Setup-Script, `DisableAppSuggestions`, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C).
7. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, which lists the three but not `SubscribedContentEnabled`, https://github.com/Biswa96/WinLight (tier C).
8. Policy CSP - Experience, `AllowWindowsConsumerFeatures` and its edition limits, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### Turn off the post-update welcome experience

`disable_welcome_experience` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Skips the full-screen "what's new" tour after an update so you land straight on your desktop.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `welcome` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value `SubscribedContent-310093Enabled`, `REG_DWORD` |

| Option | `welcome` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is any other state, such as an explicit `1` written when the Settings toggle is flipped; selecting it restores the snapshot. A fresh profile has no value, so a stock machine reads as "On".

#### How it works

`SubscribedContent-310093Enabled` is the value behind Settings > System > Notifications > "Show me the Windows welcome experience after updates and occasionally when I sign in to highlight what's new and suggested". Three independent community sources give the same key, name, type and surface, quote that toggle nearly verbatim, and agree that 1 (or no value) shows it and 0 hides it. Microsoft corroborates that the surface exists and is separately controllable: the shipped `CloudContent.admx` carries the User-class policy `DisableWindowsSpotlightWindowsWelcomeExperience` under `Software\Policies\Microsoft\Windows\CloudContent`, supported from Windows 10 RS2, but that policy is honoured only on Enterprise, Education and IoT Enterprise. The 310093 value is the consumer-reachable equivalent; Microsoft does not document it because the numeric part is composed at runtime from the subscription identifier. The update itself is untouched; only the promotional splash is suppressed.

#### Benefits
- No post-update splash: you reach the desktop instead of a promotional tour.
- The same slot drives the occasional sign-in "highlight" page, which also stops.
- Purely cosmetic: nothing functional depends on the welcome screen.

#### Drawbacks
- You lose the built-in "here is what changed" page.
- Undocumented slot; Microsoft can relocate it, although it has held from Windows 10 1703 through Windows 11 25H2 (still current in Sophia Script v7.1.6, June 2026).
- Covers only this surface; other post-update prompts such as the "finish setting up" screen need their own tweak.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 1703 and later and all Windows 11 builds, all editions; per user.
- **Takes effect**: immediately, from the next update onward.
- **Reverting**: "On" deletes the value, returning the profile to its shipped state. System Default restores the snapshot.

#### Interactions
- `disable_scoobe_nag` covers the separate "finish setting up your device" screen.
- `disable_windows_spotlight_all` is a policy master switch over Spotlight surfaces; it does not write this value, so this tweak keeps its own status.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The value is absent from a fresh profile, so the stock option deletes it rather than writing 1.
- **Confidence**: Community-corroborated (three independent sources), with Microsoft's ADMX confirming the surface.
- **Reasoning**: mechanism and polarity are uncontested. The absent-on-fresh-profile finding came from the Default hive of a test machine later flagged as modified, so it wants clean-image confirmation; deleting the value is behaviourally the same as the default-on state either way.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you already follow what changes in Windows updates or simply do not want a tour. Leave it if you like being shown new features after an update.

#### Sources
1. Shipped `CloudContent.admx` on build 26100.4061: policy `DisableWindowsSpotlightWindowsWelcomeExperience`, class User, key `Software\Policies\Microsoft\Windows\CloudContent`, supported from Windows 10 RS2 (tier A, shipped ADMX).
2. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: `SubscribedContent-310093Enabled` absent (tier A, primary observation; machine later flagged as modified).
3. Brink, "Enable or Disable Windows Welcome Experience in Windows 11", ElevenForum tutorial 3657, https://www.elevenforum.com/t/enable-or-disable-windows-welcome-experience-in-windows-11.3657/ (tier C).
4. Sophia Script for Windows 11 v7.1.6, function `WindowsWelcomeExperience`, https://github.com/farag2/Sophia-Script-for-Windows (tier C).
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C).
6. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C).
7. Policy CSP - Experience, `AllowWindowsSpotlightWindowsWelcomeExperience`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### Turn off the 'Get even more out of Windows' nag

`disable_scoobe_nag` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the full-screen "finish setting up your device" prompt after you sign in.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `scoobe` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement`, value `ScoobeSystemSettingEnabled`, `REG_DWORD` |

| Option | `scoobe` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is any other state, such as an explicit `1` from the Settings checkbox; selecting it restores the snapshot. Neither the value nor its parent key exists on a fresh profile, so a stock machine reads as "On"; applying "Off" creates the key.

#### How it works

SCOOBE is the "second chance out-of-box experience": the full-screen post-sign-in prompt that pushes a Microsoft account, OneDrive backup, Edge as the default browser and Microsoft 365. `UserProfileEngagement\ScoobeSystemSettingEnabled` is the per-user switch that Settings > System > Notifications > "Suggest ways to get the most out of Windows and finish setting up this device" writes. A string scan of 4051 shipped modules on 26100.4061 found the literal in exactly two: `SettingsHandlers_nt.dll` (the Settings handler behind the System > Notifications page) and `windowsudk.shellcommon.dll`, so the shell does read it. Microsoft documents neither the key nor the value in any Learn page or ADMX. Because the Settings checkbox exists on Home and Pro, this value works on every edition, unlike the CloudContent policies.

#### Benefits
- No sign-in upsell: you reach the desktop instead of a setup wizard.
- Works on Home, where the policy alternatives are ignored.
- Instant; no reboot or service restart.

#### Drawbacks
- Not absolute: some builds also gate the screen on server-side flags, so it can still appear once after a large feature update.
- Undocumented by Microsoft.
- Per user: other accounts need it applied separately.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 1803 and later and all Windows 11 builds, all editions; per user.
- **Takes effect**: immediately, from the next sign-in onward.
- **Reverting**: "On" deletes the value (the now-empty key may remain). System Default restores the snapshot.

#### Interactions
- `disable_welcome_experience` covers the separate post-update "what's new" page.
- `disable_account_notifications` covers account nags in the Start user tile, a different channel.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. Neither the value nor the `UserProfileEngagement` key exists on a fresh profile, so the stock option deletes the value rather than writing 1.
- **Confidence**: Community-corroborated (three independent sources across the Windows 10 and 11 wording of the toggle), with shipped-binary evidence that Settings and the shell read the value.
- **Reasoning**: the binary scan ties the value to the exact Settings page that owns the checkbox, which is strong evidence for the mechanism. No Microsoft documentation exists; the Policy CSP index and shipped `PolicyDefinitions` were searched without a match.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it. The screen offers nothing you cannot reach from Settings and it interrupts sign-ins after large updates.

#### Sources
1. String scan of 4051 shipped modules on build 26100.4061: `ScoobeSystemSettingEnabled` in `SettingsHandlers_nt.dll` and `windowsudk.shellcommon.dll` only (tier A, primary observation).
2. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: the `UserProfileEngagement` key does not exist (tier A, primary observation; machine later flagged as modified).
3. Brink, "Enable or Disable Let's finish setting up your device in Windows 11", ElevenForum tutorial 5205, https://www.elevenforum.com/t/enable-or-disable-lets-finish-setting-up-your-device-in-windows-11.5205/ (tier C).
4. Sophia Script for Windows 11 v7.1.6, function `WhatsNewInWindows`, which creates the key first because it may not exist, https://github.com/farag2/Sophia-Script-for-Windows (tier C).
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C).
6. hellzerg, Optimizer, `OptimizeHelper.cs`, writes the same value under the same key, https://github.com/hellzerg/optimizer (tier C).

### Turn off File Explorer sync-provider ads

`disable_explorer_sync_ads` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Removes the OneDrive and Microsoft 365 upsell banners from File Explorer.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `sync_ads` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value `ShowSyncProviderNotifications`, `REG_DWORD` |

| Option | `sync_ads` |
|---|---|
| Off | `0` |
| On | `1` |

System Default is any state other than 0 or 1. On a fresh profile the value is absent, which Windows treats exactly like 1 (see below), so a stock machine shows System Default while behaving as "On"; selecting System Default restores the snapshot.

#### How it works

This is the backing value for the File Explorer folder option "Show sync provider notifications" on the View tab of Folder Options. Windows declares the option in shipped OS data: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\ShowSyncProviderNotifications` on build 26100.4061 carries `ValueName` = `ShowSyncProviderNotifications`, `RegPath` = `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `HKeyRoot` = HKEY_CURRENT_USER, `Type` = checkbox, `CheckedValue` = 1, `UncheckedValue` = 0, `DefaultValue` = 1, and `Text` = `@shell32.dll,-30552` ("Show sync provider notifications"). That is Microsoft's own declaration of hive, key, name, type, polarity and default. The name appears in exactly one shipped module, `shell32.dll`, the component that owns the folder option. The notifications it governs are largely promotional banners for OneDrive and Microsoft 365 rendered in the Explorer window chrome, but genuine sync-provider notices go through the same switch.

#### Benefits
- The Explorer header stops advertising OneDrive and Microsoft 365.
- The exact switch Windows itself declares, so behaviour is predictable.
- Per user, no elevation, applied instantly.

#### Drawbacks
- Genuine sync-provider notifications go too, not only the promotional ones.
- Explorer only: OneDrive's own tray notifications are unaffected.
- Per user: other accounts need it separately.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 1703 and later and all Windows 11 builds, all editions; per user.
- **Takes effect**: on the next Explorer window, or after restarting Explorer.
- **Reverting**: "On" writes 1, which is identical in behaviour to the absent stock value because Windows declares `DefaultValue` = 1. System Default restores the snapshot (usually "absent").

#### Interactions
- `remove_onedrive` removes the OneDrive client entirely.
- `interface:remove_onedrive_nav_pane` hides OneDrive from the Explorer navigation pane, a different surface.

#### Validation
- **Verdict**: VERIFIED. No mechanism correction; the copy states that genuine sync notices are affected too.
- **Confidence**: Microsoft-documented, through Windows' own shipped folder-option definition.
- **Reasoning**: the shipped definition removes any need for community corroboration. The only nuance is fidelity of the stock option: writing 1 where the value was absent is behaviourally identical, so it is not a harmful revert.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it unless you rely on OneDrive status messages inside Explorer. For everyone else it is an advertising surface in a window you open many times a day.

#### Sources
1. Shipped folder-option definition `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\ShowSyncProviderNotifications` on build 26100.4061 (tier A, shipped OS metadata).
2. `shell32.dll` string resource 30552 resolves to "Show sync provider notifications" (tier A, shipped OS resource).
3. String scan of 4051 shipped modules: `ShowSyncProviderNotifications` only in `shell32.dll` (tier A, primary observation).
4. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: value absent (tier A, primary observation).
5. Brink, "Enable or Disable Sync Provider Notifications in File Explorer in Windows 11", ElevenForum tutorial 5200, https://www.elevenforum.com/t/enable-or-disable-sync-provider-notifications-in-file-explorer-in-windows-11.5200/ (tier C).
6. privacy.sexy, "Disable sync provider notifications", annotated "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)", https://github.com/undergroundwires/privacy.sexy (tier C).
7. Sophia Script for Windows 11 v7.1.6, function `OneDriveFileExplorerAd`, https://github.com/farag2/Sophia-Script-for-Windows (tier C).

### Turn off web results in Start search

`disable_web_search_start` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Keeps Start search local, so typing an app name stops sending keystrokes to Bing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `box_suggestions` | registry | `HKCU\Software\Policies\Microsoft\Windows\Explorer`, value `DisableSearchBoxSuggestions`, `REG_DWORD` |
| `bing_enabled` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search`, value `BingSearchEnabled`, `REG_DWORD` |

| Option | `box_suggestions` | `bing_enabled` |
|---|---|---|
| Off | `1` | `0` |
| On | `absent` | `absent` |

System Default is any mix that matches neither option, for example the policy at 1 while a feature update has reset `BingSearchEnabled`; selecting it restores the snapshot. Neither key exists on a fresh profile, so a stock machine reads as "On".

#### How it works

`DisableSearchBoxSuggestions` is a documented per-user policy. The shipped `WindowsExplorer.admx` on 26100.4061 declares it `class="User"` at `Software\Policies\Microsoft\Windows\Explorer`, enabled 1, disabled 0, supported from Windows 7, so the HKCU location, type and polarity match Microsoft's definition exactly and "not configured" is the value being absent. Microsoft's own description (the shipped ADML title is "Turn off display of recent search entries in the File Explorer search box") is about File Explorer: it suppresses suggestion pop-ups built from past search-box entries and stops those entries being stored. The additional effect this tweak relies on, removing web and Bing suggestions from the Start and taskbar search flyout on Windows 10 20H2 and later, is corroborated by three independent community sources but not stated by Microsoft; on 26100 the literal appears only in `SHCore.dll`, consistent with the search host reading it through a shared shell policy helper. `BingSearchEnabled` is the weaker half: it is referenced by the 24H2 search host (`SearchUx.Core.dll`, `SearchUx.UI.dll`) and `windowsudk.shellcommon.dll`, but it is not a policy value, is undocumented, and is reported to be reset by feature updates. Because the policy is User-class and not edition-gated, the tweak works on Home. The first-party machine-wide option, `DoNotUseWebResults` (`ConnectedSearchUseWeb` = 0 under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`), is honoured only on Enterprise, Education and IoT Enterprise, which is why this tweak uses the HKCU policy instead.

#### Benefits
- Local-only results: apps, settings and files instead of web answers and promoted cards; the Copilot entry in the search flyout goes too.
- What you type in Start stops leaving the machine.
- Policy-backed and works on every edition, including Home.

#### Drawbacks
- File Explorer stops suggesting and storing your recent search entries; that is the only behaviour Microsoft documents for the policy.
- Web answers from the search box (unit conversions, definitions, quick lookups) stop.
- `BingSearchEnabled` is fragile: if a feature update resets it, the tweak drops to System Default even though the policy half still holds.
- Microsoft Q&A has reports of this policy hiding the taskbar search box entirely on some builds (unconfirmed on 26100).

#### Applies to, takes effect, reverting
- **Applies to**: every supported build and edition; per user. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after a reboot; a sign-out or restarting Explorer and the search host also works.
- **Reverting**: "On" deletes both values, returning the profile to its shipped state. System Default restores the snapshot.

#### Interactions
- `interface:disable_search_highlights` (`IsDynamicSearchBoxEnabled`), `privacy:disable_search_history` (`IsDeviceSearchHistoryEnabled`) and `privacy:disable_cloud_content_search` (`AllowCloudSearch` and the `SearchSettings` cloud values) are related search controls on different values; they combine without conflict.
- 24H2 also has Settings > Privacy and security > Search permissions with a cloud content control, the in-box equivalent of part of this.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. `BingSearchEnabled` is absent on a fresh profile, so the stock option deletes it rather than writing 1; and the copy discloses the documented File Explorer search-history side effect.
- **Confidence**: Microsoft-documented for the policy mechanism (shipped ADMX, confirmed User class by the adversarial policy-hive audit); community-corroborated for the Start-search web effect.
- **Reasoning**: the hive audit attacked the policy's class and confirmed HKCU is correct. The 24H2 re-scope review found `DisableSearchBoxSuggestions` still effective on 26100 and recommended not presenting `BingSearchEnabled` as the mechanism, which the copy follows. Open question: whether the search-box-hiding report reproduces on any current build.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use Start to launch apps and open settings, which is most people. Skip it if you deliberately use the Start box as a web search bar or rely on File Explorer's recent-search suggestions.

#### Sources
1. Shipped `WindowsExplorer.admx` on build 26100.4061: `DisableSearchBoxSuggestions`, `class="User"`, key `Software\Policies\Microsoft\Windows\Explorer`, enabled 1, disabled 0, supported Windows 7 (tier A, shipped ADMX).
2. Shipped `en-US\WindowsExplorer.adml` on build 26100.4061: "Turn off display of recent search entries in the File Explorer search box" and its explain text (tier A, shipped ADMX).
3. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: neither key exists (tier A, primary observation; machine later flagged as modified).
4. String scan of 4051 shipped modules: `DisableSearchBoxSuggestions` in `SHCore.dll` only; `BingSearchEnabled` in `SearchUx.Core.dll`, `SearchUx.UI.dll` and `windowsudk.shellcommon.dll` (tier A, primary observation).
5. privacy.sexy, "Disable Bing search and recent search suggestions (breaks search history)", https://github.com/undergroundwires/privacy.sexy (tier C).
6. Sophia Script for Windows 11 v7.1.6, which writes the same policy value, https://github.com/farag2/Sophia-Script-for-Windows (tier C).
7. Win11Debloat (Raphire), `Regfiles/Disable_Bing_Cortana_In_Search.reg`, https://github.com/Raphire/Win11Debloat (tier C).
8. Policy CSP - Search, `DoNotUseWebResults` / `ConnectedSearchUseWeb` and its edition limits, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search (tier A).
9. "DisableSearchBoxSuggestions disables task bar search", Microsoft Q&A, https://learn.microsoft.com/en-us/answers/questions/3234673/disablesearchboxsuggestions-disables-task-bar-sear (tier D).
10. pureinfotech, "How to disable web search results on Windows 11", names `DisableSearchBoxSuggestions` as the update-resistant value, https://pureinfotech.com/disable-search-web-results-windows-11/ (tier C).

### Turn off Widgets

`disable_widgets` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Turns off the Widgets board machine-wide, clearing the news and weather feed off your taskbar.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `widgets` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Dsh`, value `AllowNewsAndInterests`, `REG_DWORD` |

| Option | `widgets` |
|---|---|
| Off | `0` |
| On | `absent` |

System Default is any other value (for example an explicit `1` set by an administrator); selecting it restores the snapshot. The policy is unset out of the box, so a stock machine reads as "On".

#### How it works

Microsoft documents this exactly. The NewsAndInterests Policy CSP maps `AllowNewsAndInterests` to the Group Policy "Allow widgets" (Computer Configuration > Windows Components > Widgets), registry key `SOFTWARE\Policies\Microsoft\Dsh`, integer (`REG_DWORD`), default 1, allowed values 0 (not allowed) and 1 (allowed), and states that the policy "applies to the entire widgets experience, including content on the taskbar". It is a machine policy, so it needs administrator rights and covers every account. The WebExperience host package that renders widgets is not uninstalled; it simply stops being offered. The feature does not exist on Windows 10, hence the Windows 11 gate.

#### Benefits
- No hover-open board and no MSN headlines on the taskbar.
- The widgets host stops fetching content you are not reading.
- One policy covers every account on the PC.

#### Drawbacks
- There is no partial setting: weather, calendar and any pinned widget you did want go too.
- Not an uninstall: the WebExperience host package still occupies disk.
- Windows 11 Home is outside Microsoft's documented edition list (Pro, Enterprise, Education, IoT Enterprise); the widgets host generally honours it on Home because it reads the value directly, but that is not a Microsoft guarantee.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 21H2 and later, confirmed current on 24H2 and 25H2; not shown on Windows 10, including LTSC 2021.
- **Takes effect**: after a sign-out or reboot, when the taskbar drops the entry point.
- **Reverting**: "On" deletes the policy value, restoring the shipped default of allowed. System Default restores the snapshot.

#### Interactions
- `remove_bing_news_weather` removes the standalone News and Weather apps; widgets pull weather through their own host, so the two are complementary.
- A proposed lock-screen widgets policy (`DisableWidgetsOnLockScreen`) was rejected because Microsoft's ADMX and CSP prescribe opposite values; a device-wide "widgets not allowed" very likely covers the lock-screen panel too.

#### Validation
- **Verdict**: VERIFIED. No correction; the copy notes the Home edition caveat.
- **Confidence**: Microsoft-documented (Policy CSP and the Microsoft connections guide, section 32 Widgets).
- **Reasoning**: the adversarial policy-hive audit confirmed `AllowNewsAndInterests` is Machine class in `NewsAndInterests.admx`, so HKLM is the correct hive. The 24H2 re-scope review found it remains the documented machine-wide Widgets control on Windows 11.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you never open the widgets board. Leave it alone if you glance at the feed, because it is all or nothing.

#### Sources
1. Policy CSP - NewsAndInterests, `AllowNewsAndInterests`, key, type, values, scope and editions, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-newsandinterests (tier A).
2. Manage connections from Windows operating system components to Microsoft services, section 32 Widgets, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A).

### Turn off Microsoft account nags in Start

`disable_account_notifications` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows nagging you in the Start menu user tile to back up, re-sign-in, or buy more storage.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `account_notifications` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`, value `DisableAccountNotifications`, `REG_DWORD` |

| Option | `account_notifications` |
|---|---|
| Off | `1` |
| On | `absent` |

System Default is any other value (for example `0`, "policy explicitly disabled"); selecting it restores the snapshot. The policy is unset out of the box, so a stock machine reads as "On".

#### How it works

The shipped `AccountNotifications.admx` on build 26100 declares `DisableAccountNotifications` as `class="User"` at `SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`, enabled 1, disabled 0, supported from Windows 10 2004 (client only). The value name also appears in the shipped `windowsudk.shellcommon.dll`, so shell code reads it. The ADML explains: "This policy allows you to prevent Windows from displaying notifications to Microsoft account (MSA) and local users in Start (user tile). Notifications include getting users to: reauthenticate; backup their device; manage cloud storage quotas as well as manage their Microsoft 365 or XBOX subscription. [...] No reboots or service restarts are required for this policy setting to take effect." Microsoft Learn's Start policy settings page repeats the list. There is also a non-policy sibling, `Start_AccountNotifications` under `HKCU\...\Explorer\Advanced` (present in `StartDocked.dll`), which is the value the Settings toggle owns; this tweak deliberately leaves it alone so that flipping the Settings switch does not silently desynchronise the tweak from its snapshot.

#### Benefits
- The Start account area stops carrying upsell badges.
- The OneDrive "you are almost full" prompt in Start goes quiet.
- Per user and instant. It needs administrator rights, because `HKCU\Software\Policies` is read-only for the user.

#### Drawbacks
- Genuine reminders go too: a real "sign in again" prompt is suppressed along with the marketing.
- Start only: OneDrive, Microsoft 365 and Xbox notifications in the notification center are unaffected.
- Per user: other accounts need it separately.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 2004 and later (including LTSC 2021) and all Windows 11 builds; user scope; not edition-gated. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: immediately, as the ADML states.
- **Reverting**: "On" deletes the policy value, restoring shipped behaviour. System Default restores the snapshot.

#### Interactions
- The Settings toggle for account notifications writes `Start_AccountNotifications`; both are honoured, and this tweak does not own that value.
- `disable_scoobe_nag` and the ContentDeliveryManager tweaks cover different channels; no overlap.

#### Validation
- **Verdict**: VERIFIED. The tweak does not require a reboot, per the ADML.
- **Confidence**: Microsoft-documented (shipped ADMX and ADML, Microsoft Learn).
- **Reasoning**: this was a gap-hunt proposal that went through the adversarial round; existence, exactness, described effect and duplication were each attacked and held, with the only change being that no reboot is needed and that the Settings-owned sibling must not be added.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use a Microsoft account and are tired of the user tile selling storage and subscriptions. Skip it if you rely on Windows telling you when your sign-in has expired.

#### Sources
1. Shipped `AccountNotifications.admx` and `en-US\AccountNotifications.adml` on build 26100: class, key, values, supportedOn and the no-reboot statement (tier A, shipped ADMX).
2. Microsoft Learn, Start menu policy settings, Disable Account Notifications, https://learn.microsoft.com/en-us/windows/configuration/start/policy-settings (tier A).
3. Policy CSP - Start, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start (tier A, cited in the tweak).
4. String presence of `DisableAccountNotifications` in `windowsudk.shellcommon.dll` on build 26100 (tier A, primary observation).
5. String presence of `Start_AccountNotifications` in `StartDocked.dll` on build 26100 (tier A, primary observation).

### Turn off account upsell cards in Settings

`disable_settings_account_ads` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Replaces the account upsell cards inside Settings with plain default content, on the editions that honour it.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `account_state_content` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value `DisableConsumerAccountStateContent`, `REG_DWORD` |

| Option | `account_state_content` |
|---|---|
| Off | `1` |
| On | `absent` |

System Default is any other value; selecting it restores the snapshot. The policy is unset out of the box, so a stock machine reads as "On".

#### How it works

The shipped `CloudContent.admx` on 26100 declares `DisableConsumerAccountStateContent` as `class="Machine"` at `Software\Policies\Microsoft\Windows\CloudContent`, `enabledValue` 1, `disabledValue` 0. The value is read by `windowsudk.shellcommon.dll` (and the MDM bridge `DMWmiBridgeProv.dll`). When enabled, Windows experiences that would show account-state content (finish setting up your account, upgrade your storage, manage your subscription cards inside Settings) present the default fallback content instead. A polarity trap exists in Microsoft's own documentation: the Policy CSP allowed-values table reads "0 (Default) Disabled. 1 Enabled", which describes the policy state, not the feature state, and contradicts the same page's description; the ADMX is decisive, and 1 turns the content off. The decisive limitation is edition: the Policy CSP edition matrix marks Pro "NOT supported", with Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC supported, so on Home and Pro the value is written and read back correctly but Windows ignores it.

#### Benefits
- The account-state promo cards in Settings stop appearing (on supported editions).
- One machine policy covers every account.
- Documented: shipped ADMX declares key, value and polarity.

#### Drawbacks
- A no-op on Home and Pro, which is most consumer PCs.
- Not every upsell: Microsoft 365 and OneDrive prompts inside their own apps are unaffected.
- Needs administrator rights, unlike the per-user suggestion tweaks.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 21H2 (22000) and later on Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. Not Windows 10, including LTSC 2021.
- **Takes effect**: immediately; sign out and back in if a card is still on screen.
- **Reverting**: "On" deletes the policy value. System Default restores the snapshot.

#### Interactions
- `privacy:disable_consumer_features` (`DisableWindowsConsumerFeatures`) lives in the same key with the same edition gate; different value, no conflict.
- `disable_windows_spotlight_all` is the user-scoped CloudContent policy and does work on Home.

#### Validation
- **Verdict**: VERIFIED. The applicability is Windows 11 21H2 and later (not Windows 10 2004), and the policy is ignored on Pro and Home.
- **Confidence**: Microsoft-documented (shipped ADMX, Policy CSP).
- **Reasoning**: the adversarial round attacked the floor (the shipped `Windows.adml` renders the ADMX token as "Windows 10 Version 1909", while the CSP says Windows 11 21H2; the narrower CSP claim was adopted), the polarity (held, per the ADMX), and the edition gate (confirmed). Open product question in the research: whether to show it on Home and Pro at all; it ships with copy that says plainly it changes nothing there.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on Enterprise, Education or IoT Enterprise, where it works. On Home and Pro skip it: it applies cleanly and does nothing.

#### Sources
1. Shipped `CloudContent.admx` and `en-US\CloudContent.adml` on build 26100: `DisableConsumerAccountStateContent`, `class="Machine"`, enabled 1, disabled 0 (tier A, shipped ADMX).
2. Policy CSP - Experience, `DisableConsumerAccountStateContent`, applicable OS and edition matrix, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).
3. Shipped `en-US\Windows.adml` on build 26100: `SUPPORTED_Windows_10_0_RS7` renders as "At least Windows Server 2016, Windows 10 Version 1909" (tier A, shipped ADMX).
4. String presence in `windowsudk.shellcommon.dll` and `DMWmiBridgeProv.dll` on build 26100 (tier A, primary observation).
5. Manage connections from Windows components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A, cited in the tweak).

### Turn off all Windows Spotlight features

`disable_windows_spotlight_all` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**One switch that turns off every Windows Spotlight surface at once.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `spotlight_all` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value `DisableWindowsSpotlightFeatures`, `REG_DWORD` |

| Option | `spotlight_all` |
|---|---|
| Off | `1` |
| On | `absent` |

System Default is any other value; selecting it restores the snapshot. The policy is unset out of the box, so a stock machine reads as "On".

#### How it works

The shipped `CloudContent.admx` on 26100 declares `DisableWindowsSpotlightFeatures` as `class="User"` at `Software\Policies\Microsoft\Windows\CloudContent`, enabled 1, disabled 0, supported on Windows 10 and later (client). The ADML, titled "Turn off all Windows spotlight features", says: "If you enable this policy setting, Windows spotlight on lock screen, Windows tips, Microsoft consumer features and other related features will be turned off. You should enable this policy setting if your goal is to minimize network traffic from target devices." The value name is present in feature consumers on 26100, not just policy plumbing: `ContentDeliveryManager.Background.dll`, `StartTileData.dll`, `Taskbar.dll`, `SettingsHandlers_ContentDeliveryManager.dll`, `CustomShellHost.exe` and `ShellAppRuntime.exe`. Because it is the User-class policy it is not edition-gated the way the machine-scope CloudContent policies are, so it works on Home.

#### Benefits
- Lock screen images, tips and consumer content stop with one write.
- Microsoft names network-traffic reduction as the purpose of this policy.
- Works on Home.

#### Drawbacks
- The rotating lock screen and desktop images stop; the lock screen falls back to a static picture.
- Broad: it also covers surfaces some people like, such as Windows tips.
- It overrides the surfaces of four other tweaks without writing their values, so those tweaks still read "not applied" while their surfaces are gone.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 (including LTSC 2021) and all Windows 11 builds, all editions; per user. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after a sign-out, when the lock screen and tip surfaces refresh.
- **Reverting**: "On" deletes the policy value and the individual tweaks resume controlling their own surfaces. System Default restores the snapshot.

#### Interactions
- Overlaps, without clobbering: `privacy:disable_lockscreen_spotlight_ads` (`RotatingLockScreenOverlayEnabled`, `SubscribedContent-338387Enabled`), `privacy:disable_tips_and_suggestions` (`SoftLandingEnabled`, `SubscribedContent-338389Enabled`, `DisableSoftLanding`) and `privacy:disable_consumer_features` (`DisableWindowsConsumerFeatures`). Each keeps its own snapshot, so revert stays correct.
- `disable_spotlight_desktop` is a strict subset of this tweak; pick one.
- `remove_getstarted_tips` removes the Tips app, which is separate from Spotlight tips.

#### Validation
- **Verdict**: VERIFIED. The copy must and does disclose the overlap with the four tweaks above.
- **Confidence**: Microsoft-documented (shipped ADMX and ADML), corroborated by privacy.sexy writing the same value.
- **Reasoning**: a gap-hunt proposal that passed the adversarial round: ADMX, ADML and consumer binaries all match. A related proposal, `DisableThirdPartySuggestions`, was rejected as a strict subset of this one.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want the whole Spotlight system off and do not care about the wallpapers. If you like the lock screen images and only want the ads gone, use the individual privacy tweaks instead.

#### Sources
1. Shipped `CloudContent.admx` on build 26100: `DisableWindowsSpotlightFeatures`, `class="User"`, enabled 1, supportedOn `SUPPORTED_Windows_10_0_NOSERVER` (tier A, shipped ADMX).
2. Shipped `en-US\CloudContent.adml` on build 26100, "Turn off all Windows spotlight features" and its explain text (tier A, shipped ADMX).
3. String presence on build 26100 in `ContentDeliveryManager.Background.dll`, `StartTileData.dll`, `Taskbar.dll`, `SettingsHandlers_ContentDeliveryManager.dll`, `CustomShellHost.exe` and `ShellAppRuntime.exe` (tier A, primary observation).
4. privacy.sexy `windows.yaml`, which writes the same value name, https://github.com/undergroundwires/privacy.sexy (tier C).
5. Policy CSP - Experience, `AllowWindowsSpotlight`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).
6. Manage connections from Windows components to Microsoft services, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A).

### Turn off Spotlight desktop wallpaper

`disable_spotlight_desktop` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the rotating Spotlight image collection from taking over your desktop wallpaper.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `spotlight_desktop` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value `DisableSpotlightCollectionOnDesktop`, `REG_DWORD` |

| Option | `spotlight_desktop` |
|---|---|
| Off | `1` |
| On | `absent` |

System Default is any other value; selecting it restores the snapshot. The policy is unset out of the box, so a stock machine reads as "On".

#### How it works

The shipped `CloudContent.admx` on 26100 declares `DisableSpotlightCollectionOnDesktop` as `class="User"` at the CloudContent policy key, enabled 1, supported on Windows 10 and later (client); the ADML text matches, including Microsoft's own typo "subsequentyly". The value is read by feature code on 26100 (`SettingsHandlers_ContentDeliveryManager.dll`, `StartTileData.dll`, `SettingsHandlers_nt.dll`, `CustomShellHost.exe`). With it set, "Windows spotlight" is no longer offered as a desktop background and the desktop returns to whatever wallpaper you set. Win11Debloat's `Disable_Desktop_Spotlight.reg` is byte-for-byte the same effect. Desktop Spotlight is inside the "and other related features" that `DisableWindowsSpotlightFeatures` turns off, so this is the narrow version of the master switch.

#### Benefits
- Your wallpaper stays put: no daily image swap chosen by Microsoft.
- The Spotlight "learn about this picture" desktop icon goes with it.
- Narrow: lock screen Spotlight and Windows tips are untouched.

#### Drawbacks
- If you enjoy the daily photo, this removes it.
- Redundant if you also apply `disable_windows_spotlight_all`.
- Per user: other accounts need it separately.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 and all Windows 11 builds, all editions; per user. Needs administrator rights: the user can only read `HKCU\Software\Policies`, so an unelevated write is refused.
- **Takes effect**: after a sign-out, when the wallpaper provider falls back.
- **Reverting**: "On" deletes the policy value so Spotlight is selectable again. System Default restores the snapshot.

#### Interactions
- Strict subset of `disable_windows_spotlight_all`; applying both is harmless (separate values, separate snapshots) but changes nothing extra.

#### Validation
- **Verdict**: VERIFIED. The copy states the relationship to the master switch.
- **Confidence**: Microsoft-documented (shipped ADMX), with an exact community match in Win11Debloat.
- **Reasoning**: a gap-hunt proposal that passed the adversarial round. Its privacy.sexy citation did not hold (privacy.sexy does not contain this value); the ADMX alone is sufficient.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want lock screen Spotlight but control of your own desktop. If you want all of Spotlight gone, use `disable_windows_spotlight_all` instead.

#### Sources
1. Shipped `CloudContent.admx` and `en-US\CloudContent.adml` on build 26100: `DisableSpotlightCollectionOnDesktop`, `class="User"`, enabled 1 (tier A, shipped ADMX).
2. String presence on build 26100 in `SettingsHandlers_ContentDeliveryManager.dll`, `StartTileData.dll`, `SettingsHandlers_nt.dll` and `CustomShellHost.exe` (tier A, primary observation).
3. Win11Debloat (Raphire), `Regfiles/Disable_Desktop_Spotlight.reg`, byte-for-byte the same effect, https://github.com/Raphire/Win11Debloat (tier C).
4. Policy CSP - Experience, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### Silence suggested and backup reminder toasts

`disable_nag_toasts` · Switch (2 options) · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Silences the "Suggested" ad toasts and the Windows Backup reminder without muting every notification.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `suggested` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.Suggested`, value `Enabled`, `REG_DWORD` |
| `backup` | registry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.BackupReminder`, value `Enabled`, `REG_DWORD` |

| Option | `suggested` | `backup` |
|---|---|---|
| Silenced | `0` | `0` |
| On | `absent` | `absent` |

System Default is any mix that matches neither option (for example one channel silenced in Settings and the other not); selecting it restores the snapshot. The per-app keys do not exist on a fresh profile, so a stock machine reads as "On".

#### How it works

`Notifications\Settings\<AppId>\Enabled` is the per-app store the Settings notifications page writes, so this uses the same mechanism as the shipped UI, applied to two system toast identifiers rather than to notifications as a whole. Both identifiers exist in shipped 26100 binaries: `Windows.SystemToast.Suggested` in `ContentDeliveryManager.Utilities.dll`, `NotificationController.dll` and `SmartActionPlatform.dll`, and `Windows.SystemToast.BackupReminder` in `shell32.dll`. Win11Debloat writes both, commented "Disable 'Suggested' app notifications (Ads for MS services)" and "Disable Windows Backup reminder notifications". The per-app keys are created lazily on the first toast, so on a fresh profile neither exists; the registry effect creates the key on apply, and a missing key reads as absent (the stock state) rather than as an error. The notification platform reads the store on the next toast.

#### Benefits
- Surgical: only these two nag channels stop; everything else still notifies you.
- Both identifiers are real on shipped 24H2 binaries.
- Per user, instant, no elevation.

#### Drawbacks
- No Windows Backup reminders.
- Not a master mute: other Microsoft promotional toasts use other identifiers and keep coming.
- Undocumented by Microsoft, although the storage scheme is the one Settings writes.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 and all Windows 11 builds, all editions; per user.
- **Takes effect**: immediately, from the next toast.
- **Reverting**: "On" deletes both values, returning the platform to its default. System Default restores the snapshot.

#### Interactions
- `interface:disable_toast_notifications` (`ToastEnabled` = 0) silences all toasts; the two are alternatives, not companions.

#### Validation
- **Verdict**: VERIFIED. The apply creates the lazily-created keys, and a missing key reads as the stock state.
- **Confidence**: Community-corroborated (Win11Debloat) plus shipped-binary presence of both identifiers; not Microsoft-documented.
- **Reasoning**: a gap-hunt proposal that passed the adversarial round with corrected sourcing: its Sophia Script citation returned no `SystemToast` match at all, and privacy.sexy uses a different mechanism (`HKLM\SOFTWARE\Classes\AppUserModelId\<id>` plus `PushNotifications\Applications\<id>`) for two different identifiers. The remaining evidence is enough for the identifiers but not for a Microsoft-documented grade.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want notifications in general but not the Microsoft service ads. If you want silence overall, use `interface:disable_toast_notifications` instead.

#### Sources
1. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, writing `Enabled` = 0 under both identifiers, https://github.com/Raphire/Win11Debloat (tier C).
2. String presence on build 26100: `Windows.SystemToast.Suggested` in `ContentDeliveryManager.Utilities.dll`, `NotificationController.dll`, `SmartActionPlatform.dll`; `Windows.SystemToast.BackupReminder` in `shell32.dll` (tier A, primary observation).
3. Raw grep of the Windows 11 `Sophia.psm1`: no `SystemToast` match, refuting a Sophia Script citation (primary observation).
4. privacy.sexy `windows.yaml`: per-app toast suppression via a different mechanism for different identifiers, https://github.com/undergroundwires/privacy.sexy (tier C).
5. Microsoft Support, Change notification settings in Windows, https://support.microsoft.com/en-us/windows/change-notification-settings-in-windows-8942c744-6198-fe56-4639-34320cf9444e (cited in the tweak for the per-app Settings surface).

### Turn off the Edge first-run experience

`disable_edge_first_run` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Edge opens straight to a normal window instead of a setup and import wizard.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `edge_frx` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `HideFirstRunExperience`, `REG_DWORD` |

| Option | `edge_frx` |
|---|---|
| Off | `1` |
| On | `absent` |

System Default is any other value (for example `0` set by an administrator); selecting it restores the snapshot. The policy is unset out of the box, so a stock machine reads as "On".

#### How it works

`HideFirstRunExperience` is a current, non-deprecated Microsoft Edge policy. It suppresses the first-run walkthrough, the "import your browsing data" prompt and the sign-in and personalization pages on first launch of a profile. Its data type is Boolean, stored as `REG_DWORD` 0 or 1 (Microsoft's example registry value is `0x00000001`). `MSEdge.admx` is generated by the Chromium ADMX writer, which marks every Edge policy `class="Both"`, so Edge reads `SOFTWARE\Policies\Microsoft\Edge` from both HKLM and HKCU; machine scope outranks user scope in Chromium's policy merge, which makes this tweak's HKLM write the valid and stronger choice. Edge reads policies at launch.

#### Benefits
- No setup gauntlet on fresh installs and new Windows profiles.
- The first-run flow is where Edge pushes default-browser and sign-in hardest; that goes.
- Documented policy with published value, type and behaviour.

#### Drawbacks
- New users on a shared PC are not walked through Edge.
- Edge stays installed; this hides a wizard only.
- Sidebar, Copilot and background processes need their own tweaks.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build with Microsoft Edge 80 or later. Windows 10 IoT Enterprise LTSC 2021 does not include Edge, so the policy is inert there unless Edge was installed separately.
- **Takes effect**: on the next Edge launch.
- **Reverting**: "On" deletes the policy value, restoring Edge's own default. System Default restores the snapshot.

#### Interactions
- `disable_edge_startup_boost` and `disable_edge_sidebar` write other values under the same Edge policy key; `privacy:disable_edge_telemetry` and `ai:disable_edge_ai_features` do too. All are distinct values and combine without conflict.

#### Validation
- **Verdict**: VERIFIED. No correction.
- **Confidence**: Microsoft-documented (Edge policy reference).
- **Reasoning**: attacked in the second verification round and survived unchanged. The policy-hive audit classed it "Class Both" and confirmed HKLM is legitimate and higher priority.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it, especially on new setups. The only cost is a walkthrough whose job is changing your defaults.

#### Sources
1. Microsoft Edge policy documentation, `HideFirstRunExperience`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hidefirstrunexperience (tier A).
2. Chromium ADMX writer (`GetClass` returns `Both`) and `policy_loader_win.cc` (HKLM machine scope, HKCU user scope), as cited in the policy-hive audit (source-code evidence).

### Turn off Edge startup boost

`disable_edge_startup_boost` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Edge preloading at sign-in and lingering after you close it, so closing it actually closes it.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `startup_boost` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `StartupBoostEnabled`, `REG_DWORD` |
| `background_mode` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `BackgroundModeEnabled`, `REG_DWORD` |

| Option | `startup_boost` | `background_mode` |
|---|---|---|
| Off | `0` | `0` |
| On | `absent` | `absent` |

System Default is any mix that matches neither option; selecting it restores the snapshot. Neither policy is set out of the box, so a stock machine reads as "On".

#### How it works

`StartupBoostEnabled` controls whether Edge pre-launches a set of processes at sign-in so the first window appears faster. `BackgroundModeEnabled` controls whether Edge keeps running after the last window closes (which is what lets extensions and apps run in the background). Both are current, non-deprecated, Windows-only Edge policies with Boolean type, so `REG_DWORD` 0 is the "off" state. Edge's own in-product defaults have both features on. As with every Edge policy, `MSEdge.admx` is `class="Both"` and the HKLM write outranks any user-scope value.

#### Benefits
- No idle Edge processes holding memory while the browser is closed.
- Nothing pre-warms a browser you may not open at sign-in.
- Both halves are documented by Microsoft.

#### Drawbacks
- Edge's first cold start each session is slower.
- Extensions that rely on background mode stop running while Edge is closed.
- Edge only; other Chromium browsers have their own settings.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build with Edge 88 or later (`StartupBoostEnabled`) and Edge 77 or later (`BackgroundModeEnabled`). Inert on LTSC 2021 unless Edge was installed separately.
- **Takes effect**: at the next sign-in or Edge restart.
- **Reverting**: "On" deletes both values, restoring Edge's defaults (both on). System Default restores the snapshot.

#### Interactions
- Shares the Edge policy key with `disable_edge_first_run`, `disable_edge_sidebar`, `privacy:disable_edge_telemetry` and `ai:disable_edge_ai_features`; all distinct values.

#### Validation
- **Verdict**: VERIFIED. No correction.
- **Confidence**: Microsoft-documented.
- **Reasoning**: attacked in the second verification round and survived unchanged; hive confirmed by the policy-hive audit.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if Edge is not your main browser, or you want nothing resident when the browser is closed. Skip it if you use Edge all day and value the faster cold start.

#### Sources
1. Microsoft Edge policy documentation, `StartupBoostEnabled`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/startupboostenabled (tier A).
2. Microsoft Edge policy documentation, `BackgroundModeEnabled`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/backgroundmodeenabled (tier A).

### Turn off the Edge sidebar and Collections

`disable_edge_sidebar` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Hides the Edge sidebar panel, reclaiming browser width, and turns off Collections.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `sidebar` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `HubsSidebarEnabled`, `REG_DWORD` |
| `collections` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `EdgeCollectionsEnabled`, `REG_DWORD` |

| Option | `sidebar` | `collections` |
|---|---|---|
| Off | `0` | `0` |
| On | `absent` | `absent` |

System Default is any mix that matches neither option; selecting it restores the snapshot. Neither policy is set out of the box, so a stock machine reads as "On".

#### How it works

`HubsSidebarEnabled` = 0 removes the right-edge hubs panel (Discover, tools such as the calculator and unit converter) and its toggle. `EdgeCollectionsEnabled` = 0 disables the Collections feature outright, not just its sidebar placement; existing collections become inaccessible from the UI while the policy is set. Both are current, non-deprecated policies written to the mandatory machine path (the "recommended" variant of `HubsSidebarEnabled` is obsolete and is not used). What this tweak does not do is remove the Copilot button from the Edge toolbar: Microsoft's `HubsSidebarEnabled` page states that "As of Microsoft Edge version 141, the `Microsoft365CopilotChatIconEnabled` policy is the only means of controlling the display of Copilot in the toolbar", and Edge updates itself independently of Windows (the 26100.4061 test machine ran Edge 150.0.4078.99), so this applies to essentially every current machine.

#### Benefits
- More page width: the right-edge panel and its rail stop taking space.
- No Discover pane inviting you into Bing content.
- Both policies are documented and current.

#### Drawbacks
- Collections is a real data feature: saved collections become unreachable while the policy is set.
- The toolbar Copilot button stays (needs `Microsoft365CopilotChatIconEnabled`, not written here).
- Sidebar tools such as the calculator and unit converter go with the panel.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build with Edge 99 or later (`HubsSidebarEnabled`) and Edge 78 or later (`EdgeCollectionsEnabled`). Inert on LTSC 2021 unless Edge was installed separately.
- **Takes effect**: on the next Edge restart.
- **Reverting**: "On" deletes both values, restoring Edge's defaults; collections become reachable again. System Default restores the snapshot.

#### Interactions
- Shares the Edge policy key with the other Edge tweaks and `ai:disable_edge_ai_features`; distinct values.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The registry mechanism is correct; what was corrected is scope: `HubsSidebarEnabled` no longer controls the toolbar Copilot button (Edge 141 and later), and Collections is named in the tweak because it is a separate feature bundled in.
- **Confidence**: Microsoft-documented.
- **Reasoning**: both policies are live and correctly written. The research recommended splitting `EdgeCollectionsEnabled` into its own tweak or option so hiding a panel does not take a data feature with it; the tweak instead names Collections explicitly in its title and warns about it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if the sidebar is noise and you do not use Collections. If you use Collections at all, leave it alone, because you would lose access to saved content while it is applied.

#### Sources
1. Microsoft Edge policy documentation, `HubsSidebarEnabled` (Edge 99 and later, and the Edge 141 Copilot toolbar statement), https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hubssidebarenabled (tier A).
2. Microsoft Edge policy documentation, `EdgeCollectionsEnabled` (Edge 78 and later, no deprecation), https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/edgecollectionsenabled (tier A).
3. Direct inspection on Windows 11 24H2 build 26100.4061: installed Edge 150.0.4078.99, past the Edge 141 cutover (primary measurement).

### Remove the Microsoft Teams app

`remove_teams_consumer_app` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Uninstalls the preinstalled Microsoft Teams app, which on current Windows is one app for personal, work and school.**

The app shows a warning on this tweak: removing it removes your work or school Teams client too.

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `Teams`, `REG_DWORD` (the app's own state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'MSTeams' \| Remove-AppxPackage -AllUsers`, then `Remove-AppxProvisionedPackage -Online` for every provisioned package whose `DisplayName` is `MSTeams`, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id XP8BT8DW290MPQ -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [MSTeams]` (present when no installed or provisioned `MSTeams` package remains) |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state: the marker value does not exist until the tweak is first applied, so a machine that has never used this tweak matches neither option. Selecting System Default restores the snapshot taken before the first apply, which deletes the marker and, if the app was present then, runs the undo to reinstall it.

#### How it works

Since Windows 11 23H2 Microsoft ships one unified Teams package, `MSTeams` (package family `MSTeams_8wekyb3d8bbwe`, Store listing `XP8BT8DW290MPQ`, title "Microsoft Teams"), for personal, work and school use; it replaced the Chat-era consumer package `MicrosoftTeams`, which does not ship on 24H2. `MSTeams` is the id Microsoft lists in its policy-based inbox app removal list for 24H2 and 25H2. `Remove-AppxPackage -AllUsers` removes the installed package for every existing user; `Remove-AppxProvisionedPackage -Online` removes the copy registered against the Windows image, which would otherwise reinstall the app for each new user profile. The apply runs elevated (admin); the marker is a per-user value written as you. The probe is answered from one shared enumeration of installed and provisioned packages that fails closed: if enumeration fails, or the app is unelevated, the status is Unknown rather than "Removed". Selecting "Installed" while the app is removed drives the action's undo.

#### Benefits
- One less preinstalled app on disk and in the app list.
- No prompts to connect a personal account.
- Reinstallable from the Microsoft Store.

#### Drawbacks
- Work and school Teams goes too: the unified package is the one organizations deploy.
- Joining a Teams meeting falls back to the browser.
- A Windows feature update may re-provision the package; apply again if it returns.
- The undo needs `winget` (part of App Installer) and internet access.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (the gate); the package exists on 23H2 and later, so on the 24H2 floor it is normally present.
- **Takes effect**: immediately; the app is uninstalled on apply.
- **Reverting**: "Installed" or System Default runs `winget install --id XP8BT8DW290MPQ`, which needs internet access and a working `winget`. If the reinstall fails, the tweak surfaces Needs Attention and keeps the snapshot.

#### Interactions
- None known in the shipped corpus. The old Chat taskbar controls (`remove_teams_chat_taskbar`, `interface:disable_chat_taskbar`) are not shipped; see Considered and not shipped.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The July 2026 research graded a `MicrosoftTeams` target INCORRECT twice over (that package does not ship on 24H2, and Store id `9NZTWSQNTK1S` returns HTTP 404), and specified the corrected mechanism this tweak implements: target `MSTeams`, reinstall from `XP8BT8DW290MPQ`, fail-closed probe, provisioned-package removal, and copy that warns work and school Teams share the package.
- **Confidence**: Microsoft-documented (policy-based inbox app removal list, 23H2 release notes, Store catalog).
- **Reasoning**: the Store catalog resolves `XP8BT8DW290MPQ` to "Microsoft Teams" on `MSTeams_8wekyb3d8bbwe`. Open questions: whether removing a work-capable client belongs in a debloat tool at all is a product decision the research left open; and the research confirmed the catalog record for `XP8BT8DW290MPQ` but did not record a `winget show` resolution for it, so the command-line reinstall is unverified.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it only on a personal machine where nobody uses Teams for work or school. If the PC signs into any organization's Teams, leave it.

#### Sources
1. Microsoft Store catalog service, product `XP8BT8DW290MPQ` returns "Microsoft Teams", package family `MSTeams_8wekyb3d8bbwe` (tier A, primary observation).
2. Microsoft Store catalog service, product `9NZTWSQNTK1S` returns HTTP 404, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NZTWSQNTK1S?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
3. `winget show --id 9NZTWSQNTK1S --exact` on winget 1.29.280 returns no package (primary observation).
4. Policy-based inbox app removal, supported app list, which names `MSTeams` and not `MicrosoftTeams`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
5. What's new in Windows 11 version 23H2, "Chat is being removed from the Microsoft Teams in-box app", https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2 (tier A).

### Remove Clipchamp

`remove_clipchamp` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Uninstalls the bundled Clipchamp video editor.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `Clipchamp`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'Clipchamp.Clipchamp' \| Remove-AppxPackage -AllUsers`, then remove the provisioned package whose `DisplayName` is `Clipchamp.Clipchamp`, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id 9P1J8S7CCWWT -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [Clipchamp.Clipchamp]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, which deletes the marker and, if Clipchamp was present before the first apply, reinstalls it.

#### How it works

Clipchamp is a consumer video editor Microsoft acquired and bundles with Windows 11 (preinstalled on 22H2 and later). The Store catalog resolves `9P1J8S7CCWWT` to "Microsoft Clipchamp" with package family `Clipchamp.Clipchamp_yxz26nhyzhsrt`; the non-Microsoft publisher hash is a legacy of the acquisition and does not affect removal, and `winget show --id 9P1J8S7CCWWT --source msstore` resolves with publisher "Microsoft Corp.". `Clipchamp` is on Microsoft's policy-based inbox app removal list for 24H2 and 25H2, which is first-party confirmation that it is preinstalled and supported for removal. It is not an operating system component. The action removes it for all existing users and removes the provisioned copy so new profiles do not get it; detection uses the shared fail-closed package enumeration.

#### Benefits
- Reclaims space; Clipchamp is one of the larger bundled apps.
- One fewer entry in Start and Installed apps.
- Microsoft supports its removal on 24H2.

#### Drawbacks
- No built-in video editor remains.
- A feature update may re-provision the package; apply again if so.
- Reinstalling needs `winget` and internet access.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (the gate). Windows 10 LTSC 2021 ships no Store apps and is excluded by the gate.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall; a failed reinstall surfaces as Needs Attention with the snapshot kept.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The removal must also remove the provisioned package (otherwise new profiles and feature updates bring the app back), and the "is it removed" check must fail closed rather than read a failed query as "removed".
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog).
- **Reasoning**: both identities were checked against the Store catalog and `winget`. The fail-open class (an unelevated or failing `Get-AppxPackage` reading as "removed") was the research's main finding across all removals; the shipped probe is the shared enumeration, which reports Unknown on failure.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it if you have never opened it, which is most people. Keep it if you edit video on this machine.

#### Sources
1. Microsoft Store catalog service, product `9P1J8S7CCWWT` returns "Microsoft Clipchamp", package family `Clipchamp.Clipchamp_yxz26nhyzhsrt`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9P1J8S7CCWWT?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Policy-based inbox app removal, supported app list, includes `Clipchamp`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).
4. Remove-AppxPackage, `-AllUsers` parameter reference, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A).

### Remove Quick Assist

`remove_quick_assist` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls Quick Assist, the remote-help tool that tech-support scammers lean on.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `QuickAssist`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'MicrosoftCorporationII.QuickAssist' \| Remove-AppxPackage -AllUsers`, then remove the matching provisioned package, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id 9P7BP5VNWKX5 -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [MicrosoftCorporationII.QuickAssist]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling Quick Assist if it was present before the first apply.

#### How it works

Quick Assist lets another person view or control your PC after you read them a code, which is exactly the flow tech-support scams use. The modern Quick Assist is a Store app, `MicrosoftCorporationII.QuickAssist` (package family `MicrosoftCorporationII.QuickAssist_8wekyb3d8bbwe`, Store id `9P7BP5VNWKX5`, title "Quick Assist", resolvable in `winget`'s msstore source), which replaced the in-box Win32 app from Windows 10 2004. `QuickAssist` is on Microsoft's 24H2 and 25H2 inbox app removal list. The action removes it for all users and removes the provisioned copy; detection uses the shared fail-closed package enumeration. It is not a system component and nothing depends on it.

#### Benefits
- Removes the in-box path for a caller to take control of your screen.
- Nothing depends on it.
- Reinstallable from the Store.

#### Drawbacks
- A family member who supports you remotely loses their easiest route in.
- Friction, not a block: someone can still be talked into reinstalling it.
- A feature update may re-provision it; apply again if so.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer. The tweak is not gated, but on Windows 10 IoT Enterprise LTSC 2021 it is inert: that image ships the legacy Win32 Quick Assist, which this does not touch, so the apply finds no package and the tweak reports the Appx package as absent.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall (needs internet and `winget`; LTSC has neither the Store nor, typically, `winget`). A failed reinstall surfaces as Needs Attention.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The removal must also remove the provisioned package, and the check must fail closed.
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog).
- **Reasoning**: identity confirmed in the Store catalog and `winget`. The research suggested gating to Windows 11 because the tweak can do nothing on LTSC 2021; it ships ungated, which is harmless (it removes nothing there) but lets the tweak appear on a platform where it has no effect.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it on a personal machine that never receives remote assistance, especially one used by someone who might fall for a support call. Keep it if you or your helper use it.

#### Sources
1. Microsoft Store catalog service, product `9P7BP5VNWKX5` returns "Quick Assist", package family `MicrosoftCorporationII.QuickAssist_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9P7BP5VNWKX5?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Policy-based inbox app removal, supported app list, includes `QuickAssist`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).

### Remove Bing News and Weather

`remove_bing_news_weather` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls the MSN News and Weather apps, cutting one MSN content and notification channel.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `BingNewsWeather`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply, with `$ErrorActionPreference = 'Stop'`, for each of `Microsoft.BingNews` and `Microsoft.BingWeather`: `Get-AppxPackage -AllUsers -Name $name -PackageTypeFilter Bundle \| Remove-AppxPackage -AllUsers`, then `Get-AppxPackage -AllUsers -Name $name \| Remove-AppxPackage -AllUsers`, then remove the provisioned package with that `DisplayName`. Undo: `winget install --id 9WZDNCRFHVFW -e ... && winget install --id 9WZDNCRFJ3Q2 -e ...` (both with `--accept-source-agreements --accept-package-agreements`). Probe: `appx_absent: [Microsoft.BingNews, Microsoft.BingWeather]` (present only when both are gone) |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the apps are currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and running the undo if the apps were present before the first apply.

#### How it works

These are the standalone MSN apps: News (`Microsoft.BingNews`, family `Microsoft.BingNews_8wekyb3d8bbwe`, Store id `9WZDNCRFHVFW`, "Microsoft News") and Weather (`Microsoft.BingWeather`, family `Microsoft.BingWeather_8wekyb3d8bbwe`, Store id `9WZDNCRFJ3Q2`, "MSN Weather"). Both are on Microsoft's 24H2 and 25H2 inbox app removal list, and both Store ids resolve in `winget`'s msstore source. The apply loops over the two names because `Get-AppxPackage -Name` takes a single string: passing both names as an array raises a parameter-binding error and removes nothing. Both apps ship as bundles, and Microsoft's `Remove-AppxPackage` reference says `-AllUsers` "works off the parent package type. If it's a bundle, use `PackageTypeFilter` with the `Get-AppxPackage` command and specify the bundle", so the bundle pass runs first, then any remaining registration, then the provisioned copy. The probe is the shared fail-closed enumeration and reads present only when neither package remains installed or provisioned.

#### Benefits
- No MSN News or Weather in Start or Installed apps.
- Their toast and badge channels go with them.
- Microsoft supports removing both on 24H2.

#### Drawbacks
- Widgets still show weather through their own host; use `disable_widgets` for that.
- Nothing in Windows replaces the standalone Weather app.
- A feature update may re-provision either package.
- The undo, as shipped, chains the two installs with `&&`. The app runs actions in Windows PowerShell 5.1, which rejects `&&` as a parse error ("The token '&&' is not a valid statement separator in this version"), so the command-line reinstall is expected to fail and the tweak to show Needs Attention; reinstall both apps from the Microsoft Store by hand if you revert.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer. Not gated, but inert on Windows 10 IoT Enterprise LTSC 2021: Microsoft names News and Weather on the LTSC excluded-app list, so there is nothing to remove.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the undo above; see the drawback on `&&`. When the reinstall cannot be verified the snapshot is kept and the tweak shows Needs Attention.

#### Interactions
- `disable_widgets` turns off the Widgets board, the other MSN surface; complementary, no overlap.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The research graded the two-names-at-once form INCORRECT (a binding error that removed nothing while reporting success) and specified the per-name loop, bundle filter, provisioned removal and fail-closed probe this tweak ships.
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog, `Remove-AppxPackage` reference), with the binding error reproduced on Windows PowerShell 5.1.26100.4061 and PowerShell 7.
- **Reasoning**: both package names and Store ids are correct. The research recommended `&&` to stop the first install's failure being masked, but that operator does not exist in Windows PowerShell 5.1, the shell the app uses; this is an open defect in the undo, not in the removal (running `echo a && echo b` through `powershell.exe` 5.1 returns that parse error).
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove them if you get news and weather elsewhere, which is most people. Keep them if you open the Weather app, because widgets are not a replacement; and plan to reinstall from the Store by hand if you change your mind.

#### Sources
1. `Get-AppxPackage` parameter metadata: `-Name` is `System.String` on Windows PowerShell 5.1.26100.4061 and PowerShell 7 (primary observation).
2. Reproduction of `CannotConvertArgument` on both shells (primary observation).
3. Microsoft Store catalog service, products `9WZDNCRFHVFW` ("Microsoft News") and `9WZDNCRFJ3Q2` ("MSN Weather") (tier A, primary observation).
4. Policy-based inbox app removal, supported app list, includes `BingNews` and `BingWeather`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
5. Remove-AppxPackage, `-AllUsers` and bundle guidance, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A).
6. Windows as a service overview, LTSC excluded app list naming Weather and News, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A).

### Remove Solitaire Collection

`remove_solitaire` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls the ad-supported Microsoft Solitaire Collection.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `Solitaire`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply, with `$ErrorActionPreference = 'Stop'`: bundle pass `Get-AppxPackage -AllUsers -Name 'Microsoft.MicrosoftSolitaireCollection' -PackageTypeFilter Bundle \| Remove-AppxPackage -AllUsers`, then the same without the filter, then remove the provisioned package. Undo: `Start-Process 'ms-windows-store://pdp/?ProductId=9WZDNCRFHWD2'` (opens the Store product page). Probe: `appx_absent: [Microsoft.MicrosoftSolitaireCollection]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and running the undo (opening the Store page) if the app was present before the first apply.

#### How it works

The collection (`Microsoft.MicrosoftSolitaireCollection`, family `Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe`, Store id `9WZDNCRFHWD2`) shows video ads between hands and sells a subscription to remove them. It is on Microsoft's 24H2 and 25H2 inbox app removal list and ships as a bundle, so the bundle pass runs first per Microsoft's `-PackageTypeFilter` guidance, then the provisioned copy is removed. The Store catalog resolves the id correctly, but `winget show --id 9WZDNCRFHWD2 --exact` returns "No package found" in every source (most likely because it is categorised as a game), so a command-line reinstall is impossible and the undo opens the Store product page instead. That undo exits as soon as the Store opens; the reinstall itself is yours to click.

#### Benefits
- No video ads or Premium upsell on the machine.
- Reclaims the space of a large bundle.
- Microsoft supports its removal on 24H2.

#### Drawbacks
- Klondike, Spider and the rest are gone, with no in-box replacement.
- Reverting opens the Store rather than reinstalling; until you install it there, the app cannot confirm the "Installed" state and the tweak may show Needs Attention.
- A feature update may re-provision it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer. Not gated, but inert on LTSC 2021 (no Store apps); also absent on Enterprise images where consumer experiences are suppressed.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default opens `ms-windows-store://pdp/?ProductId=9WZDNCRFHWD2`; install from there.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. `winget` cannot resolve the Store id, so the reinstall must go through the Store app; the bundle needs `-PackageTypeFilter Bundle`; the provisioned package must be removed; and the check must fail closed.
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog), with the `winget` gap observed directly on winget 1.29.280.
- **Reasoning**: identity is correct and the removal works. The undo is an honest hand-off rather than a pretend install.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it unless you play these games. If you do play them, decide before applying, because getting them back means a manual Store install.

#### Sources
1. Microsoft Store catalog service, product `9WZDNCRFHWD2` returns "Microsoft Solitaire Collection", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9WZDNCRFHWD2?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. `winget show --id 9WZDNCRFHWD2 --exact` returns no package on winget 1.29.280 (primary observation).
3. Policy-based inbox app removal, supported app list, includes `MicrosoftSolitaireCollection`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
4. Remove-AppxPackage, `-AllUsers` and bundle guidance, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A).

### Remove the Get Help app

`remove_get_help` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls the Get Help support app.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `GetHelp`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'Microsoft.GetHelp' \| Remove-AppxPackage -AllUsers`, then remove the provisioned package, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id 9PKDZBMV1H3T -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [Microsoft.GetHelp]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling Get Help if it was present before the first apply.

#### How it works

Get Help (`Microsoft.GetHelp`, family `Microsoft.GetHelp_8wekyb3d8bbwe`, Store id `9PKDZBMV1H3T`, Store category "System Components") is Microsoft's in-app support and troubleshooting front end and the entry point to its virtual support agent; some Settings troubleshooters deep-link into it. The id resolves in `winget`'s msstore source (publisher URL support.microsoft.com), so the reinstall is executable. It was preinstalled on Windows 10 1709 and later and on Windows 11 through at least 23H2, but it is absent from Microsoft's 24H2 and 25H2 inbox app removal list. That can mean it is no longer preinstalled, or that it is classified as a system component and deliberately excluded from the list (Microsoft's troubleshooting section names event IDs for "a system component"); the research could not settle which without a clean 24H2 image. On a machine without it, the apply finds nothing and the tweak simply reports it absent.

#### Benefits
- One less preinstalled app for people who never use guided support.
- Removes a support-chat surface.
- Reinstallable from the Store.

#### Drawbacks
- Troubleshooter links into Get Help fail instead of falling back.
- The Store lists it as a System Component; Windows expects it present, even though removal works.
- A feature update may re-provision it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer where present; presence on a stock 24H2 image is unconfirmed. Not gated, but inert on LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall; failure surfaces as Needs Attention.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The removal must also remove the provisioned package and the check must fail closed; presence on a stock 24H2 image is an open question.
- **Confidence**: Microsoft-documented for identity (Store catalog); applicability on 24H2 unconfirmed.
- **Reasoning**: the 24H2 re-scope review put this in the ambiguous band (absent from the removal list, but not declared removed by Microsoft) and advised verifying on a real image rather than deleting.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it if you troubleshoot Windows yourself or with a search engine. Keep it if you use the built-in guided troubleshooters.

#### Sources
1. Microsoft Store catalog service, product `9PKDZBMV1H3T` returns "Get Help", category "System Components", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9PKDZBMV1H3T?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Policy-based inbox app removal, supported app list (does not include `GetHelp`) and its troubleshooting section on system-component exclusions, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).

### Remove Tips (Get Started)

`remove_getstarted_tips` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls the Tips app, which exists to show onboarding and promotional content.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `GetStarted`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'Microsoft.Getstarted' \| Remove-AppxPackage -AllUsers`, then remove the provisioned package, with `$ErrorActionPreference = 'Stop'`. Undo: `Start-Process 'ms-windows-store://pdp/?ProductId=9WZDNCRDTBJJ'` (opens the Store product page). Probe: `appx_absent: [Microsoft.Getstarted]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and opening the Store page if the app was present before the first apply.

#### How it works

The app (`Microsoft.Getstarted`, lowercase "s", family `Microsoft.Getstarted_8wekyb3d8bbwe`, Store id `9WZDNCRDTBJJ`, title "Microsoft Tips", category "System Components") delivers "getting started" walkthroughs and tip content. Microsoft's deprecated-features page says "The Tips app is deprecated and will be removed in a future release of Windows", and the app is absent from the 24H2 and 25H2 inbox app removal list, but Microsoft has not said it is no longer preinstalled, so its presence on a stock 24H2 image is unconfirmed. The Store catalog resolves the id, but `winget` cannot (`winget show --id 9WZDNCRDTBJJ --exact` finds nothing in any source), so the undo opens the Store product page instead of pretending to install. These tips are separate from the Windows tips delivered through Spotlight.

#### Benefits
- Removes an app whose whole purpose is onboarding and tip content.
- Deprecated upstream.
- Nothing in Windows depends on it.

#### Drawbacks
- Guided tip walkthroughs go.
- Reverting opens the Store rather than reinstalling; the tweak may show Needs Attention until you install it there.
- A feature update may re-provision it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer where present; presence unconfirmed. Not gated, but inert on LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default opens `ms-windows-store://pdp/?ProductId=9WZDNCRDTBJJ`; install from there.

#### Interactions
- `disable_windows_spotlight_all` and `privacy:disable_tips_and_suggestions` cover Spotlight and soft-landing tips, a separate surface.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. `winget` cannot resolve the Store id, so the reinstall goes through the Store app; the provisioned package must be removed; the check must fail closed.
- **Confidence**: Microsoft-documented (deprecated-features list, Store catalog).
- **Reasoning**: the 24H2 re-scope review graded it OBSOLETE (deprecated and off the removal list, presence on a stock image unconfirmed) rather than DELETE, because Microsoft has published no explicit "no longer preinstalled" statement of the kind it published for Maps.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it. It is deprecated, purely promotional, and nothing depends on it.

#### Sources
1. Microsoft Store catalog service, product `9WZDNCRDTBJJ` returns "Microsoft Tips", package family `Microsoft.Getstarted_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9WZDNCRDTBJJ?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. `winget show --id 9WZDNCRDTBJJ --exact` returns no package on winget 1.29.280 (primary observation).
3. Deprecated features in the Windows client, Tips app, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A).
4. Policy-based inbox app removal, supported app list (does not include `Getstarted`), https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
5. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).

### Remove Feedback Hub

`remove_feedback_hub` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls Feedback Hub, the app for sending feedback and diagnostic traces to Microsoft.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `FeedbackHub`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'Microsoft.WindowsFeedbackHub' \| Remove-AppxPackage -AllUsers`, then remove the provisioned package, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id 9NBLGGH4R32N -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [Microsoft.WindowsFeedbackHub]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling Feedback Hub if it was present before the first apply.

#### How it works

Feedback Hub (`Microsoft.WindowsFeedbackHub`, family `Microsoft.WindowsFeedbackHub_8wekyb3d8bbwe`, Store id `9NBLGGH4R32N`, resolvable in `winget`) is where you file bugs and suggestions with Microsoft; it collects diagnostic traces and attaches them to reports. `WindowsFeedbackHub` is on Microsoft's 24H2 and 25H2 inbox app removal list. The action removes it for all users and removes the provisioned copy; detection uses the shared fail-closed package enumeration.

#### Benefits
- Removes an app most people never open.
- One fewer diagnostic surface.
- Microsoft supports its removal on 24H2.

#### Drawbacks
- Windows Insider feedback and several diagnostic flows require it.
- No in-box way to report bugs remains; Microsoft's own deprecated-features page directs feature feedback here.
- A feature update may re-provision it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer. Not gated, but inert on LTSC 2021 (not shipped there).
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall; failure surfaces as Needs Attention.

#### Interactions
- `privacy:disable_feedback_notifications` (`DoNotShowFeedbackNotifications`) stops Windows asking for feedback; a separate surface that works with or without the app.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The removal must also remove the provisioned package, and the check must fail closed.
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog).
- **Reasoning**: identity confirmed; no open questions beyond the shared removal-class fixes.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it on a stable, non-Insider machine that never files feedback. Keep it on Insider builds.

#### Sources
1. Microsoft Store catalog service, product `9NBLGGH4R32N` returns "Feedback Hub", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH4R32N?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Policy-based inbox app removal, supported app list, includes `WindowsFeedbackHub`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
3. Deprecated features in the Windows client, directing feature feedback to Feedback Hub, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A).
4. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).

### Remove Phone Link

`remove_phone_link` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls Phone Link, the companion app that mirrors an Android phone or iPhone on the PC.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `PhoneLink`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'Microsoft.YourPhone' \| Remove-AppxPackage -AllUsers`, then remove the provisioned package, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id 9NMPJ99VJBWV -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [Microsoft.YourPhone]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling Phone Link if it was present before the first apply.

#### How it works

Phone Link keeps its original package name, `Microsoft.YourPhone` (family `Microsoft.YourPhone_8wekyb3d8bbwe`, Store id `9NMPJ99VJBWV`, title "Phone Link", resolvable in `winget`); the app was renamed from Your Phone without a package rename. It mirrors phone notifications, messages, calls and photos, and on Windows 11 24H2 and later it also backs the phone panel in the Start menu and phone integration in Settings, so removing the package removes those too. It was preinstalled on Windows 10 1809 and later and on Windows 11 through at least 23H2, but it is absent from Microsoft's 24H2 and 25H2 inbox app removal list, which may mean it is no longer preinstalled or is classed as a system component; presence on a stock 24H2 image is unconfirmed. It is also among the packages feature updates re-provision most often.

#### Benefits
- Nothing pairing or polling if you never link a phone.
- The 24H2 Start phone panel goes with it.
- Reinstallable from the Store.

#### Drawbacks
- All phone integration stops, on Android and iPhone.
- The Start phone panel disappears.
- Re-provisioned by feature updates unusually often.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer where present; presence unconfirmed. Not gated, but inert on LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall; failure surfaces as Needs Attention.

#### Interactions
- `interface:disable_phone_companion_start` hides the phone panel in Start (`Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe\IsEnabled` = 0) while keeping the app; use it instead if you only want the panel gone. After this removal, that tweak's revert restores what the package declares, which is nothing while the app is absent.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The removal must also remove the provisioned package, the check must fail closed, and the copy must mention the Start panel dependency on 24H2.
- **Confidence**: Microsoft-documented for identity (Store catalog); applicability on 24H2 unconfirmed.
- **Reasoning**: in the re-scope review's ambiguous band with Get Help; verify on a real image rather than delete.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it if you never connect a phone to this PC. If you only dislike the Start panel, use `interface:disable_phone_companion_start` instead.

#### Sources
1. Microsoft Store catalog service, product `9NMPJ99VJBWV` returns "Phone Link", package family `Microsoft.YourPhone_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NMPJ99VJBWV?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Policy-based inbox app removal, supported app list (does not include `YourPhone`) and its system-component exclusions, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).
4. Microsoft Support, Mobile device in Start menu (the Settings switch behind the Start panel), https://support.microsoft.com/en-us/windows/mobile-device-in-start-menu-21676d6a-3bc3-439a-aaa3-7463b91cda79 (tier A).

### Remove the New Outlook app

`remove_outlook_new` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Uninstalls the preinstalled new Outlook for Windows, leaving classic Outlook untouched.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `OutlookNew`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply: `Get-AppxPackage -AllUsers -Name 'Microsoft.OutlookforWindows' \| Remove-AppxPackage -AllUsers`, then remove the provisioned package, with `$ErrorActionPreference = 'Stop'`. Undo: `winget install --id 9NRX63209R7B -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [Microsoft.OutlookforWindows]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling the app if it was present before the first apply.

#### How it works

The new Outlook (`Microsoft.OutlookforWindows`, lowercase "f", family `Microsoft.OutlookforWindows_8wekyb3d8bbwe`, Store id `9NRX63209R7B`, title "Outlook for Windows", resolvable in `winget`) is the web-based mail client Microsoft bundles with Windows 11 22H2 and later. Its Store listing states "This app will replace the Windows Mail, Calendar, and People apps beginning in 2024". `OutlookForWindows` is on Microsoft's 24H2 and 25H2 inbox app removal list. Classic Outlook from Microsoft 365 is a separate Win32 installation and is not touched. Microsoft re-pushes this app hard, through Windows Update as well as Store provisioning, so even with the provisioned copy removed it can return.

#### Benefits
- Removes a mail client you did not choose.
- Stops the migration nudges from classic Outlook and Mail.
- Classic Outlook is untouched.

#### Drawbacks
- Windows no longer ships Mail and Calendar, so no bundled mail client remains.
- It can come back through Windows Update or a feature update; apply again if so.
- Removed for every account on the PC.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 (the gate). The app has also been pushed to Windows 10 22H2, which is out of scope and excluded by the gate.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall; failure surfaces as Needs Attention.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The canonical package identity is `Microsoft.OutlookforWindows` (lowercase "f"; matching is case-insensitive, so this is a record fix), the removal must include the provisioned package, and the check must fail closed.
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog).
- **Reasoning**: identity confirmed. The research suggested the Windows 11 gate is narrower than reality because the app also lands on Windows 10 22H2; that platform is out of scope, so the gate stands.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it if you use classic Outlook, a browser or another mail client. Keep it if it is your mail app.

#### Sources
1. Microsoft Store catalog service, product `9NRX63209R7B` returns "Outlook for Windows", package family `Microsoft.OutlookforWindows_8wekyb3d8bbwe`, and the Mail, Calendar and People replacement statement, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NRX63209R7B?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Policy-based inbox app removal, supported app list, includes `OutlookForWindows`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A).

### Remove Xbox Game Bar

`remove_xbox_game_bar` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls the Win+G Game Bar overlay, without touching Game Mode or any GPU feature.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `XboxGameBar`, `REG_DWORD` (state marker) |
| `app` | action (PowerShell, timeout 600 s) | Apply, with `$ErrorActionPreference = 'Stop'`: bundle pass `Get-AppxPackage -AllUsers -Name 'Microsoft.XboxGamingOverlay' -PackageTypeFilter Bundle \| Remove-AppxPackage -AllUsers`, then the same without the filter, then remove the provisioned package. Undo: `winget install --id 9NZKPSTSNW4P -e --accept-source-agreements --accept-package-agreements`. Probe: `appx_absent: [Microsoft.XboxGamingOverlay]` |

| Option | `state` | `app` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if the app is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling Game Bar if it was present before the first apply.

#### How it works

`Microsoft.XboxGamingOverlay` (family `Microsoft.XboxGamingOverlay_8wekyb3d8bbwe`, Store id `9NZKPSTSNW4P`, title "Game Bar", category "System Components", resolvable in `winget`) is the overlay itself: capture, the performance widget and the Xbox social panels. Game Mode, hardware-accelerated GPU scheduling and Auto HDR are separate features and are unaffected. `XboxGamingOverlay` is on Microsoft's 24H2 and 25H2 inbox app removal list, as a separate id from `XboxIdentityProvider` (which game sign-in needs and this does not touch). It ships as a bundle, so the bundle pass runs first per Microsoft's guidance. Because the servicing stack treats it as a system component on some builds, `Remove-AppxPackage` can fail with 0x80073CFA; with `$ErrorActionPreference = 'Stop'` that failure ends the script non-zero and the app reports an error and rolls back rather than claiming success. After removal, anything that invokes the `ms-gamingoverlay:` URI shows Windows' "You'll need a new app to open this ms-gamingoverlay link" dialog.

#### Benefits
- The capture and overlay layer stops loading with games.
- No accidental Win+G mid-game.
- Game Mode is untouched.

#### Drawbacks
- `ms-gamingoverlay` link dialogs appear when games or Settings call the overlay.
- The Win+Alt+R recorder and screenshot hotkeys go with it.
- Some builds refuse the removal (0x80073CFA), reported as an error.
- A feature update may re-provision it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer. Not gated, but inert on LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs the `winget` reinstall; failure surfaces as Needs Attention.

#### Interactions
- `performance:disable_gamedvr_capture` turns off background capture (`GameDVR_Enabled`, `AppCaptureEnabled`, `AllowGameDVR`) without removing anything; a much smaller change if recording is your only concern. Its `AppCaptureEnabled` value is also one of the values the research suggested for suppressing the `ms-gamingoverlay` prompt.
- `services:disable_xbox_services` stops Xbox Live services; keep Xbox Identity Provider if you sign into games.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The removal needs the bundle filter and the provisioned-package step, and the check must fail closed; the copy discloses the `ms-gamingoverlay` breakage.
- **Confidence**: Microsoft-documented (inbox app removal list, Store catalog, `Remove-AppxPackage` reference).
- **Reasoning**: medium risk is appropriate because removal leaves a visible error dialog in some games. The research also suggested a companion effect to suppress that prompt; it is not part of this tweak.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it only if you never press Win+G and can live with the occasional `ms-gamingoverlay` prompt. If you only want background recording off, use `performance:disable_gamedvr_capture`.

#### Sources
1. Microsoft Store catalog service, product `9NZKPSTSNW4P` returns "Game Bar", category "System Components", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NZKPSTSNW4P?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation).
2. Remove-AppxPackage, `-AllUsers` and bundle guidance, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A).
3. Policy-based inbox app removal, supported app list, including `XboxGamingOverlay` and `XboxIdentityProvider` as separate ids, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).

### Remove OneDrive

`remove_onedrive` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Uninstalls the OneDrive client so it stops syncing and stops asking to back up your folders.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\Debloat`, value `OneDrive`, `REG_DWORD` (state marker) |
| `onedrive` | action (PowerShell, timeout 600 s) | Apply: collect every uninstaller present: `%SystemRoot%\SysWOW64\OneDriveSetup.exe` and `%SystemRoot%\System32\OneDriveSetup.exe` (run with `/uninstall`), and each versioned `OneDriveSetup.exe` under `%ProgramFiles%\Microsoft OneDrive\` and `%ProgramFiles(x86)%\Microsoft OneDrive\` (run with `/uninstall /allusers`); exit 1 if none is found; run each with `-Wait` and exit with the first non-zero exit code. Undo: `winget install --id Microsoft.OneDrive -e --accept-source-agreements --accept-package-agreements`. Probe (script): not removed if a `OneDrive` process is running or `OneDrive.exe` exists at `%LOCALAPPDATA%\Microsoft\OneDrive\`, `%ProgramFiles%\Microsoft OneDrive\` or `%ProgramFiles(x86)%\Microsoft OneDrive\`; any error also reads as not removed |

| Option | `state` | `onedrive` |
|---|---|---|
| Removed | `1` | run |
| Installed | `0` | not run (undo runs if OneDrive is currently removed) |

System Default is the stock state (no marker yet); selecting it restores the snapshot, deleting the marker and reinstalling OneDrive if it was present before the first apply.

#### How it works

OneDrive is not a Store app; it ships as a Win32 setup payload. The in-box per-user stub lives in `%SystemRoot%\SysWOW64` on 64-bit Windows (and in `System32` on ARM64 or builds shipping a 64-bit stub), and `OneDriveSetup.exe /uninstall` removes the per-user client, unlinks the account and drops its startup entry. A per-machine install, created with `OneDriveSetup.exe /allusers` (what Microsoft 365 and modern imaging deploy), lives under `Program Files\Microsoft OneDrive\<version>\` and needs `/uninstall /allusers`; the apply handles both. Finding no uninstaller at all is a failure, not a silent success, and the uninstaller's exit code is propagated, so a failed uninstall is reported. Files already downloaded to disk stay where they are. The `winget` reinstall is valid: `Microsoft.OneDrive` resolves to "Microsoft OneDrive", publisher Microsoft Corporation.

#### Benefits
- No OneDrive process, upload activity or sync icon.
- The recurring "back up your folders" prompts stop.
- Locally downloaded files stay on disk.

#### Drawbacks
- Cloud-only (Files On-Demand) placeholders that were never downloaded become unreachable locally; download them first.
- If Desktop, Documents or Pictures were redirected into OneDrive (Known Folder Move), they can end up somewhere you do not expect.
- Office documents stop auto-saving to the cloud.
- A feature update can reinstall the stub.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10, including LTSC 2021 where OneDrive was installed separately; the only removal in this category still meaningful on the secondary platform. LTSC and IoT LTSC images do not ship OneDrive at all (neither stub exists on Windows 11 IoT Enterprise LTSC 26100.4061), so on a clean LTSC image the apply fails with "no installer found".
- **Takes effect**: immediately.
- **Reverting**: "Installed" or System Default runs `winget install --id Microsoft.OneDrive`, which needs internet access and `winget`; failure surfaces as Needs Attention. Reinstalling does not re-link your account or restore folder redirection.

#### Interactions
- `disable_explorer_sync_ads` removes OneDrive upsell banners in Explorer without uninstalling.
- `interface:remove_onedrive_nav_pane` hides OneDrive in the Explorer navigation pane.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. Per-machine installs must be found and removed with `/uninstall /allusers`, the probe must cover the `Program Files` paths, the uninstaller's exit code must be checked, and finding no uninstaller must fail.
- **Confidence**: Community-corroborated for the per-machine path and switches (two tier C sources), with Microsoft's support article for the uninstall itself.
- **Reasoning**: the in-box stub ordering and the `winget` id were confirmed; the per-machine gap was the substantive finding. Open point: the research noted the per-user uninstall does not strictly need administrator rights; the tweak requests admin because the per-machine path does.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Remove it if you never use OneDrive and have checked that no files are cloud-only and no folders are redirected. If you sync anything or use folder backup, leave it.

#### Sources
1. `winget show --id Microsoft.OneDrive --exact` resolves to "Microsoft OneDrive", publisher Microsoft Corporation (primary observation).
2. Per-machine OneDrive install location and `/uninstall /allusers`, ManageEngine OneDrive administration reference, https://www.manageengine.com/microsoft-365-management-reporting/kb/onedrive-administration/per-machine-installation.html (tier C).
3. "Installing the OneDrive Sync Client in Per-Machine mode", byteben, https://byteben.com/bb/installing-the-onedrive-sync-client-in-per-machine-mode-during-your-task-sequence-for-a-lightening-fast-first-logon-experience/ (tier C).
4. Direct filesystem inspection on Windows 11 IoT Enterprise LTSC build 26100.4061: no `OneDriveSetup.exe` in `SysWOW64` or `System32` (primary observation).
5. Microsoft Support, Turn off, disable, or uninstall OneDrive, https://support.microsoft.com/en-us/office/turn-off-disable-or-uninstall-onedrive-f32a17ce-3336-40fe-9c38-6efb09f944b0 (tier A).
6. Microsoft Learn, Install OneDrive per machine, https://learn.microsoft.com/en-us/sharepoint/per-machine-installation (cited in the tweak).

## Considered and not shipped

Five controls were researched for this category and are deliberately not shipped, because the feature or package they target is gone at or below the Windows 11 24H2 support floor. In each case the mechanism was real on the platform it was written for; dropping them is a scope decision. Two other app removals researched alongside this category, `remove_copilot_app` and `remove_recall_feature`, moved to the AI category and are documented there.

### Chat (Teams) taskbar icon

`remove_teams_chat_taskbar` wrote the machine policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Chat`, `ChatIcon` (`REG_DWORD`) = 3 (Disabled). Microsoft's 23H2 release notes say "Chat is being removed from the Microsoft Teams in-box app"; the Chat flyout this policy governs was replaced by a normal pinnable "Microsoft Teams (free)" app that `ChatIcon` does not control, and the backing `ConfigureChatIcon` policy carries a deprecation note in the Policy CSP ("This policy is deprecated and may be removed in a future release"). The feature was removed one release before the support floor and never existed on Windows 10, so no supported platform remains. The Interface category's `disable_chat_taskbar` (`TaskbarMn`) targeted the same vanished button and is not shipped either. Sources: What's new in Windows 11 version 23H2, https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2 (tier A); Policy CSP - Experience, `ConfigureChatIcon` deprecation note, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### Cortana

`remove_cortana` wrote `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`, `AllowCortana` = 0, and removed the `Microsoft.549981C3F5F10` Appx package with a `winget install --id 9NFFX4SZZ23L` undo. The standalone Cortana app was retired in spring 2023 and is on Microsoft's deprecated-features list; `Microsoft.549981C3F5F10` is not in the 24H2 or 25H2 preinstalled Store app set, and Windows 10 IoT Enterprise LTSC ships no Store apps, so the removal finds nothing on either target. The undo was already impossible: the Store record for `9NFFX4SZZ23L` still exists, but `winget show --id 9NFFX4SZZ23L --exact` returns no package in any source because the app is delisted. The policy half is pointless with no Cortana present; the policy-hive audit also found no `AllowCortana` policy left in the shipped 26100 `Search.admx`. Sources: End of support for Cortana, https://support.microsoft.com/en-us/topic/end-of-support-for-cortana-d025b39f-ee5b-4836-a954-0ab646ee1efa (tier A); Deprecated features in the Windows client, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); `winget show --id 9NFFX4SZZ23L --exact` on winget 1.29.280 (primary observation).

### Dev Home

`remove_dev_home` removed `Microsoft.Windows.DevHome` with a `winget install --id 9N8MHTPHNGVV` undo. Microsoft states "Starting May 2025, Dev Home will no longer be supported as a feature in Windows 11", archived the repository in June 2025 and dropped the app from the preinstalled set; it is absent from the 24H2 and 25H2 inbox app removal list. Worse, the Store listing was reused: `9N8MHTPHNGVV` now resolves to "Windows Advanced Settings" on the same package family, `Microsoft.Windows.DevHome_8wekyb3d8bbwe`, so on a current machine the tweak would uninstall a supported successor app and its undo would install that successor rather than Dev Home, with no way for the probe to tell the two apart. Sources: Dev Home documentation, https://learn.microsoft.com/en-us/windows/dev-home/ (tier A); Microsoft Store catalog, product `9N8MHTPHNGVV` returns "Windows Advanced Settings" (tier A, primary observation); Policy-based inbox app removal list, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A).

### Windows Maps

`remove_maps` removed `Microsoft.WindowsMaps` with a `winget install --id 9WZDNCRDTBVB` undo. Microsoft: "Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release." The app was pulled from the Store in July 2025 and a final update made it non-functional, so it cannot be reinstalled (`winget` finds no package), and Microsoft deprecated the UWP Map control and `Windows.Services.Maps` APIs on 8 April 2025. There is nothing to remove on the support floor and no way to reverse the removal. Sources: Deprecated features resources, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources (tier A); Deprecated features in the Windows client, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); "Microsoft kills Windows Maps app", Neowin, https://www.neowin.net/news/microsoft-kills-windows-maps-app/ (tier C).

### People app

`remove_people` removed `Microsoft.People` with a `winget install --id 9NBLGGH10PG8` undo. Microsoft lists My People as deprecated ("My People is no longer being developed"), `Microsoft.People` is absent from the 24H2 and 25H2 inbox app removal list, and People belongs to the Mail, Calendar and People family that Microsoft excludes from LTSC editions, so it is absent on both targets. The new Outlook's own Store listing says it "will replace the Windows Mail, Calendar, and People apps beginning in 2024" (see [Remove the New Outlook app](#remove-the-new-outlook-app)). The undo was already broken: `winget` finds no package for `9NBLGGH10PG8` even though the catalog record resolves. Sources: Deprecated features in the Windows client, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); Policy-based inbox app removal list, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A); Windows as a service overview, LTSC excluded app list, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A).
