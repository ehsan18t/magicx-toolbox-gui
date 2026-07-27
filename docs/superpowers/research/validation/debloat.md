# Debloat and Consumer tweak validation

Rebuilt 2026-07-27. Source corpus: `src-tauri/tweaks/debloat.yaml` (29 tweaks as authored).
Target platform: Windows 11 24H2 (build 26100) and newer including 25H2, x64. Windows 10 IoT
Enterprise LTSC 2021 (19044) is a low-priority secondary target.

This revision consolidates the original validation pass, two adversarial verification rounds, the
24H2 re-scope review, the probe fail-open audit, and the cross-category analysis into one
authoritative document, then adds five newly verified tweaks.

**Scope of this document.** 27 live tweaks and 5 recommended deletions.

- 29 tweaks were authored in `debloat.yaml`.
- 2 moved out to the new `ai` category and are no longer covered here: `remove_copilot_app` and
  `remove_recall_feature`. Their validation record lives in `ai.md`.
- 5 are recommended for deletion and carry a short note instead of a full entry:
  `remove_teams_chat_taskbar`, `remove_cortana`, `remove_dev_home`, `remove_maps`, `remove_people`.
- 22 existing tweaks are carried forward with full entries.
- 5 newly verified tweaks are added: `disable_account_notifications`, `disable_settings_account_ads`,
  `disable_windows_spotlight_all`, `disable_spotlight_desktop`, `disable_nag_toasts`.

22 carried forward plus 5 new gives **27 live tweaks**, plus **5 deletion notes**.

Two proposals that were routed through this category during the gap hunt are not here.
`disable_edge_ai_features` went to `ai.md`. `disable_online_tips` went to `privacy.md`.

## Evidence base

Five classes of evidence stand behind this document.

1. **Microsoft Learn, the Policy CSP pages, and the Microsoft Edge policy reference** (tier A).
2. **Shipped ADMX and ADML files** in `C:\Windows\PolicyDefinitions` on build 26100.4061 (tier A,
   shipped product artifact). Where an entry quotes a `<policy>` element it was parsed from the
   shipped file, not from a mirror.
3. **Shipped OS metadata**: the folder-option definitions under
   `HKLM\...\Explorer\Advanced\Folder`, and `C:\Users\Default\NTUSER.DAT`, the template every new
   user profile is cloned from and therefore the authority on what a fresh profile does and does not
   contain.
4. **String scans of shipped binaries**, 4051 modules across `System32` and `SystemApps` on build
   26100.4061, used to establish which value names the OS actually reads. Presence in a feature
   consumer is strong evidence; presence only in `SHCore.dll` or `DMWmiBridgeProv.dll` is weak,
   because those carry policy plumbing tables rather than feature code.
5. **Live resolution of every Store product ID** against the Microsoft Store catalog service
   (`storeedgefd.dsx.mp.microsoft.com/v9.0/products/<id>`, the same service `winget` uses) and
   against `winget` 1.29.280 itself. That pass is treated as tier A for identity facts, because it
   is Microsoft's own catalog answering for its own product IDs.

Absence of a Microsoft page is not evidence against a tweak. Where a claim rests only on community
references, the entry says so and labels the tweak community-corroborated rather than
Microsoft-documented.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `disable_start_suggestions` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Stock Default must be `absent`; 338388 is the Windows 10 Start toggle and the Windows 11 surface is `Start_IrisRecommendations`, which the info text claims but the tweak never writes |
| `disable_auto_install_sponsored_apps` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Drop `SubscribedContentEnabled`: no shipped 24H2 binary reads it and it is not in the shipped Default hive; info text claims `ContentDeliveryAllowed` is written but it is not |
| `disable_welcome_experience` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Stock Default must be `absent`, not 1 |
| `disable_scoobe_nag` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Stock Default must be `absent`; neither the value nor the `UserProfileEngagement` key exists on a fresh profile |
| `disable_explorer_sync_ads` | VERIFIED | low | Microsoft-documented | None mechanically; info text overstates that only advertising is affected |
| `disable_web_search_start` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Stock Default writes `BingSearchEnabled: 1` where a clean profile has no value; info text omits the documented File Explorer search-history side effect |
| `disable_widgets` | VERIFIED | low | Microsoft-documented | None (note: Windows 11 Home is not a documented supported edition) |
| `disable_edge_first_run` | VERIFIED | low | Microsoft-documented | None. Survived adversarial attack |
| `disable_edge_startup_boost` | VERIFIED | low | Microsoft-documented | None. Survived adversarial attack |
| `disable_edge_sidebar` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | As of Edge 141 `HubsSidebarEnabled` no longer controls the toolbar Copilot button, so the Copilot promise is false; Collections is also bundled into a tweak named for the sidebar |
| `disable_account_notifications` | VERIFIED | low | Microsoft-documented | New tweak. `requires_reboot` must be `false` |
| `disable_settings_account_ads` | VERIFIED | low | Microsoft-documented | New tweak. Applicability is Windows 11 21H2 and later, not Windows 10 2004, and it is a no-op on Pro and Home |
| `disable_windows_spotlight_all` | VERIFIED | low | Microsoft-documented | New tweak. Master switch; copy must disclose the overlap with four shipped tweaks |
| `disable_spotlight_desktop` | VERIFIED | low | Microsoft-documented | New tweak. Subset of `disable_windows_spotlight_all`; the relationship must be stated |
| `disable_nag_toasts` | VERIFIED | low | Community-corroborated | New tweak. Apply must create the lazily-created per-app keys; probe must treat key-absent as stock |
| `remove_teams_consumer_app` | INCORRECT | low | Microsoft-documented | Store ID `9NZTWSQNTK1S` returns HTTP 404; the `MicrosoftTeams` package does not ship on 24H2; Teams is now one unified `MSTeams` package shared with work and school |
| `remove_clipchamp` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Provisioned package not removed; fail-open probe |
| `remove_quick_assist` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Provisioned package not removed; fail-open probe |
| `remove_bing_news_weather` | INCORRECT | low | Microsoft-documented | `Get-AppxPackage -AllUsers 'A','B'` is a parameter-binding error, so the apply removes nothing and the probe reports "Removed" unconditionally; the `;` between the two `winget install` calls masks a failed first install |
| `remove_solitaire` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Store ID is right but `winget` cannot resolve it, so the undo fails; bundle needs `-PackageTypeFilter Bundle` |
| `remove_get_help` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Provisioned package not removed; fail-open probe; presence on a stock 24H2 image unconfirmed |
| `remove_getstarted_tips` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Store ID is right but `winget` cannot resolve it, so the undo fails; app is on Microsoft's deprecated list |
| `remove_feedback_hub` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Provisioned package not removed; fail-open probe |
| `remove_phone_link` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Provisioned package not removed; fail-open probe; presence on a stock 24H2 image unconfirmed |
| `remove_outlook_new` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Canonical package identity is `Microsoft.OutlookforWindows` (lowercase f); provisioned package not removed |
| `remove_xbox_game_bar` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Provisioned package not removed; bundle needs `-PackageTypeFilter Bundle`; removal breaks the `ms-gamingoverlay:` protocol handler |
| `remove_onedrive` | VERIFIED-WITH-CORRECTION | medium | Community-corroborated | Per-machine OneDrive installs are handled by neither the apply nor the probe; info text describes a `Run`-key edit the tweak does not perform |

Counts across the 27 live tweaks: **VERIFIED 9, VERIFIED-WITH-CORRECTION 16, UNVERIFIED 0,
DISPUTED 0, INCORRECT 2.**

## Corrections required

Every concrete defect found, numbered. Each names the tweak, the exact wrong thing, and the exact
right thing.

### Broken mechanisms

1. **`remove_teams_consumer_app` is incorrect twice over.** First, Store product ID `9NZTWSQNTK1S`
   does not exist: the Store catalog service returns HTTP 404 for it and
   `winget show --id 9NZTWSQNTK1S --exact` finds nothing in any source, so the undo can never
   succeed. Second, the `MicrosoftTeams` package does not ship on Windows 11 24H2 at all. It was the
   Chat-era consumer build, removed in 23H2. The preinstalled Teams on 24H2 and 25H2 is `MSTeams`
   (Store listing `XP8BT8DW290MPQ`, package family `MSTeams_8wekyb3d8bbwe`), which is **one unified
   package shared by personal, work and school Teams**. Removing it is therefore no longer a
   consumer-only action, and the tweak's claim that work Teams is a different package is false on the
   target platform. Because the queried package is absent, the probe reports "Removed" on every 24H2
   machine, including machines that never had it.
2. **`remove_bing_news_weather` apply and probe are broken PowerShell.** `Get-AppxPackage` declares
   its `-Name` parameter as `System.String`, not `System.String[]`. Passing
   `'Microsoft.BingNews','Microsoft.BingWeather'` binds an `Object[]` to a `String` parameter and
   raises `CannotConvertArgument`. Reproduced on Windows PowerShell 5.1.26100.4061 (the shell the
   engine actually spawns) and on PowerShell 7. The cmdlet returns nothing, so the apply pipeline
   removes nothing, and the probe's `if (Get-AppxPackage ...)` evaluates false and exits 0, which
   the engine reads as "applied". The tweak reports success while doing nothing, violating the
   project's did-it-work contract. Fix: two separate `Get-AppxPackage` calls, or
   `'Microsoft.BingNews','Microsoft.BingWeather' | ForEach-Object { Get-AppxPackage -AllUsers $_ }`.
3. **`remove_bing_news_weather` undo masks the first failure.** `winget install A; winget install B`
   exits with B's code. If the News install fails, the action still reports success. Replace the `;`
   with `&&` or with explicit exit-code checking.

### The fail-open probe class

4. **Every app-removal probe in this file fails open.** All of them are shaped
   `if (Get-AppxPackage -AllUsers '<name>') { exit 1 } else { exit 0 }`. `Get-AppxPackage` failures
   are *non-terminating*: the error is written to the error stream, the expression evaluates to
   `$null`, `$null` is falsy, the `else` branch runs, and the probe exits 0. The engine reads exit 0
   as "the removal is applied". So the probe answers "is this app gone?" with "yes" whenever it fails
   to find out. The realistic trigger is ordinary: **`Get-AppxPackage -AllUsers` requires
   elevation**, so running unelevated reports installed apps as removed. Other triggers are a package
   identity that no longer exists on the build, a parameter binding error (defect 2), and any
   transient Appx service failure. The fix is to promote the error and exit non-zero on every path
   that did not prove the desired state:

   ```powershell
   $ErrorActionPreference = 'Stop'
   try { $pkg = Get-AppxPackage -AllUsers -Name 'Microsoft.GetHelp' } catch { exit 1 }
   if ($pkg) { exit 1 } else { exit 0 }
   ```

   This affects `remove_teams_consumer_app`, `remove_clipchamp`, `remove_quick_assist`,
   `remove_bing_news_weather`, `remove_solitaire`, `remove_get_help`, `remove_getstarted_tips`,
   `remove_feedback_hub`, `remove_phone_link`, `remove_outlook_new`, `remove_xbox_game_bar` and,
   in a different shape, `remove_onedrive`.
5. **Every app-removal tweak leaves the provisioned package in place.** `Remove-AppxPackage
   -AllUsers` removes the installed package for existing users. It does not remove the *provisioned*
   package registered against the image, so the app is reinstalled for any newly created user profile
   and can be restored by a feature update. Durable removal requires
   `Remove-AppxProvisionedPackage -Online -PackageName <full name>`. The probes have the matching
   blind spot: `Get-AppxPackage -AllUsers <name>` does not see provisioned-only state, so a tweak can
   read as "applied" on a machine where the next new user will get the app back.
6. **`Remove-AppxPackage -AllUsers` has a documented bundle caveat none of these tweaks handle.**
   Microsoft's reference states the parameter "works off the parent package type. If it's a bundle,
   use `PackageTypeFilter` with the `Get-AppxPackage` command and specify the bundle." Solitaire,
   News, Weather and Game Bar ship as bundles. Without `-PackageTypeFilter Bundle` the removal can
   leave bundle registration behind.
7. **Undo commands assume `winget` is present and usable.** `winget.exe` is a per-user app execution
   alias provided by `Microsoft.DesktopAppInstaller`. It is absent on LTSC and on images where the
   App Installer was stripped, and `msstore`-source installs from an elevated context are subject to
   winget's own restrictions. None of the undos degrade gracefully when it is missing.
8. **Four Store product IDs are correct but not installable through `winget`.** The catalog service
   resolves each one to exactly the app the tweak claims, but `winget show --id <id> --exact` returns
   "No package found matching input criteria" in every source, reproducibly, on winget 1.29.280:
   `9WZDNCRFHWD2` (Solitaire) and `9WZDNCRDTBJJ` (Tips) are the two live cases in this document; the
   same was true of `9NBLGGH10PG8` (People) and `9NFFX4SZZ23L` (Cortana), both now recommended for
   deletion. Those undos fail at the command line even though the apps install fine from the Store UI.

### Revert states that fabricate values

9. **`disable_start_suggestions` stock default is wrong.** `SubscribedContent-338388Enabled` is
   absent from the shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061, as are all other
   `SubscribedContent-*Enabled` values. "On (Stock Default)" must be `absent`, not 1. Writing 1 on
   revert leaves the machine in a state it was never in.
10. **`disable_welcome_experience` stock default is wrong.** Same finding for
    `SubscribedContent-310093Enabled`. Must be `absent`, not 1.
11. **`disable_scoobe_nag` stock default is wrong.** Neither `ScoobeSystemSettingEnabled` nor its
    parent `UserProfileEngagement` key exists in the shipped Default user hive. Must be `absent`,
    not 1. Reverting by writing 1 creates a key and a value the machine never had.
12. **`disable_web_search_start` stock default writes a value that was absent.** The revert option
    sets `bing_enabled: 1`, but the entire `Software\Microsoft\Windows\CurrentVersion\Search` key is
    absent from the shipped Default user hive on 26100.4061, so `BingSearchEnabled` does not exist on
    a fresh profile and is created only when the setting is changed. The correct stock default is
    `absent`, matching what the same option already does for `box_suggestions`.
13. **`disable_explorer_sync_ads` is the documented exception.** `ShowSyncProviderNotifications` is
    also absent on a fresh profile, but Windows declares `DefaultValue` = 1 for that folder option, so
    an explicit 1 and an absent value behave identically. `absent` is the more faithful restoration;
    the current 1 is not a defect.

### Wrong or dead values

14. **`disable_auto_install_sponsored_apps` writes a value name Windows does not read.**
    `SubscribedContentEnabled` (with no numeric ID) is absent from the shipped Default user hive,
    absent from the live hive, and a string scan of 4051 shipped `System32` and `SystemApps` modules
    on build 26100.4061 found it in **zero** modules, while every sibling name from the same key
    (`ContentDeliveryAllowed`, `SystemPaneSuggestionsEnabled`, `SoftLandingEnabled`,
    `RotatingLockScreenOverlayEnabled`, `PreInstalledAppsEnabled`, and the `SubscribedContent-`
    prefix) was found. No independent tool or tutorial writes it. **Drop it from the effects.** Its
    revert value of 1 creates a value that was never there. The other three values in this tweak are
    correct, including their stock default of 1, which the shipped Default user hive confirms.
15. **`disable_auto_install_sponsored_apps` info text overstates the effects.** It says the tweak
    clears `ContentDeliveryAllowed`. It does not; that value is not among the effects. Adding it
    would be legitimate, since `ContentDeliveryAllowed` is genuinely read by six shipped modules.
16. **`disable_start_suggestions` targets the wrong surface on Windows 11.** The info text says "on
    24H2 the `Start_IrisRecommendations` value" is cleared. No effect writes it. On 24H2 and 25H2 the
    Start recommendation surface is driven by `Start_IrisRecommendations` under
    `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` (a literal string in the
    shipped `StartTileData.dll`) and by the tier A `HideRecommendedSection` policy in
    `StartMenu.admx`. Every independent source maps `SubscribedContent-338388Enabled` to the
    **Windows 10** toggle "Occasionally show suggestions in Start", not to the Windows 11 wording the
    info text uses. Correct the description, or add the Windows 11 effect, or gate the tweak to the
    secondary Windows 10 target.
17. **`disable_edge_sidebar` promises to remove a Copilot button it can no longer remove.**
    Microsoft's `HubsSidebarEnabled` policy page states: "As of Microsoft Edge version 141, the
    `Microsoft365CopilotChatIconEnabled` policy is the only means of controlling the display of
    Copilot in the toolbar." Edge auto-updates independently of Windows, and the Edge on the
    26100.4061 test machine is 150.0.4078.99. The tweak's `name`, its summary, its "What it does" and
    its "What you gain" all promise an outcome the policy does not produce on any current Edge. The
    sidebar panel itself is still hidden correctly, so this is a text defect, not a dead value.
    Fix by dropping the Copilot wording, or by adding `Microsoft365CopilotChatIconEnabled` as a
    companion effect if the button really is in scope. `disable_edge_startup_boost` and
    `disable_edge_first_run` were attacked in the same round and survived unchanged.
18. **`disable_web_search_start` should stop presenting `BingSearchEnabled` as the mechanism.**
    `DisableSearchBoxSuggestions` is the policy-backed value that carries the tweak on 26100 and
    later. `BingSearchEnabled` is not a policy value, is undocumented on Learn, and is reported reset
    by feature updates. Either drop it or mark it `skip_validation: true` so a value the OS may
    silently reset cannot produce a false Needs Attention state.
19. **`disable_web_search_start` info text omits a documented side effect.** The shipped
    `WindowsExplorer.adml` names this policy "Turn off display of recent search entries in the File
    Explorer search box" and explains it as suppressing suggestion pop-ups built from past Search Box
    entries and preventing those entries from being stored. That is the only behaviour Microsoft
    attributes to the value, and the tweak text should disclose it.
20. **`remove_outlook_new` package identity casing.** The Store catalog reports the package family as
    `Microsoft.OutlookforWindows_8wekyb3d8bbwe` (lowercase "f" in "for"). The YAML uses
    `Microsoft.OutlookForWindows`. `Get-AppxPackage -Name` matching is case-insensitive, so this has
    no functional effect, but the canonical identity should be recorded correctly.

### OneDrive

21. **`remove_onedrive` does not handle per-machine installs.** The apply only checks
    `%SystemRoot%\SysWOW64\OneDriveSetup.exe` and `%SystemRoot%\System32\OneDriveSetup.exe`. A
    per-machine OneDrive (installed with `OneDriveSetup.exe /allusers`, which is what Microsoft 365
    and modern imaging deployments use) lives at
    `%ProgramFiles%\Microsoft OneDrive\<version>\OneDriveSetup.exe` and must be removed with
    `/uninstall /allusers`. On such a machine the apply silently does nothing.
22. **`remove_onedrive` probe misses per-machine installs.** It checks only for a running `OneDrive`
    process and for `%LOCALAPPDATA%\Microsoft\OneDrive\OneDrive.exe`. A per-machine install under
    `Program Files` with the process not currently running reads as "removed".
23. **`remove_onedrive` discards the uninstaller's exit code.** `Start-Process ... -Wait` does not
    capture it, so a failed uninstall is indistinguishable from a successful one.
24. **`remove_onedrive` info text describes an edit the tweak does not make.** It says the tweak
    "clears the OneDrive value from the user `Run` key". No effect touches
    `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. Either add the effect or drop the claim.

### Authoring requirements for the five new tweaks

25. **`disable_account_notifications` must set `requires_reboot: false`.** The shipped
    `AccountNotifications.adml` states plainly: "No reboots or service restarts are required for this
    policy setting to take effect."
26. **`disable_account_notifications` should ship the policy value alone.** The non-policy sibling
    `HKCU\...\Explorer\Advanced\Start_AccountNotifications` is real (found in `StartDocked.dll`) but
    it is the value the Settings toggle owns. If the tweak owns it too, a user flipping the Settings
    switch silently desynchronises the tweak from its snapshot.
27. **`disable_settings_account_ads` applicability is Windows 11 21H2 and later, not Windows 10
    2004.** The shipped 26100 `Windows.adml` renders `SUPPORTED_Windows_10_0_RS7` as "At least
    Windows Server 2016, Windows 10 Version 1909", and Microsoft's Policy CSP page states the
    applicable OS as "Windows 11, version 21H2 [10.0.22000] and later". The CSP is the narrower and
    more current claim.
28. **`disable_settings_account_ads` is a no-op on Pro and Home and must say so.** The Policy CSP
    edition matrix for `DisableConsumerAccountStateContent` reads "Pro NOT supported", with
    Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC supported. Do not oversell it. This
    is the same gate `privacy:disable_consumer_features` already lives with, which is likewise a
    no-op on Home and Pro.
29. **Do not "fix" `disable_settings_account_ads` against the CSP allowed-values table.** That table
    reads "0 (Default) Disabled. 1 Enabled", which naively read would invert the tweak. It describes
    the **policy** state, not the feature state, and it contradicts the same page's own description
    block. The ADMX `enabledValue` of 1 is decisive: 1 turns the account-state content **off**.
30. **`disable_windows_spotlight_all` copy must disclose its blast radius.** It is a master switch
    that overlaps four shipped tweaks (`RotatingLockScreenOverlayEnabled`,
    `SubscribedContent-338387Enabled`, `SoftLandingEnabled` / `SubscribedContent-338389Enabled`,
    `DisableWindowsConsumerFeatures`). It does not clobber their values, so revert stays correct, but
    a user who applies this and then looks at those tweaks will see them still reading "not applied"
    while their surfaces are gone.
31. **`disable_spotlight_desktop` overlaps `disable_windows_spotlight_all` and must say so.** Desktop
    Spotlight is inside the "and other related features" that the master switch turns off. Ship at
    most one of the two as a *recommended* item; the other is fine as an available control.
32. **`disable_nag_toasts` apply must create the keys.** The per-app
    `Notifications\Settings\<AppId>` keys are created lazily on the first toast, so on a fresh
    profile they do not exist. The apply must create the key before writing `Enabled`, and the status
    probe must treat "key absent" as the stock state rather than an error, or the did-it-work
    contract produces false failures.
33. **`disable_nag_toasts` is community-corroborated, not Microsoft-documented.** The proposal's
    citations to Sophia Script and privacy.sexy do not hold: a raw grep of the Windows 11
    `Sophia.psm1` returns no `SystemToast` match at all, and privacy.sexy uses a different mechanism
    (`HKLM\SOFTWARE\Classes\AppUserModelId\<id>` plus `PushNotifications\Applications\<id>`) for two
    different toast identifiers. Real support is Win11Debloat plus shipped-binary presence.

### Cross-references carried in from other categories

34. **`privacy:disable_consumer_features` is a no-op on Home and Pro.** Its backing policy,
    `DisableWindowsConsumerFeatures` / `AllowWindowsConsumerFeatures`, is supported only on
    Enterprise, Education and IoT Enterprise. Several debloat entries point users at it as the
    "stronger machine-wide block"; that recommendation is only true on those editions.
35. **State-marker key naming is inconsistent corpus-wide.** Seventeen debloat tweaks write their
    state marker under `HKCU\Software\MagicXToolbox\Debloat`, while nine tweaks across network,
    performance and security use `HKCU\Software\MagicXToolbox\State`. Cosmetic and harmless at
    runtime, but worth unifying.
36. **Asymmetric action options are correct by design, not a defect.** Action-carrying tweaks list
    the action effect in only one option (`{state: 1, app: run}` versus `{state: 0}`). The engine
    drives the action's `undo` when the target option omits an undo-carrying action whose probe reads
    present (`src-tauri/src/tweaks/engine/apply.rs:391` and `:221`). Recorded so a future review does
    not re-raise it.
37. **HKCU writes under an admin elevation floor are correct by design, not a defect.** ADR-0005
    states that a user-hive effect ignores the tweak's elevation floor and always runs in-process as
    the real user, so it never lands in the elevated account's hive. Recorded for the same reason.

## Recommended for deletion

Five tweaks target features or packages that no longer ship on the Windows 11 24H2 floor. Each is
recorded here with the reason rather than a full entry. All five were VERIFIED-WITH-CORRECTION in the
prior pass, so the deletion is a scope decision, not a correctness one: the mechanisms were real on
the platforms they were written for.

### `remove_teams_chat_taskbar` Chat (Teams) taskbar icon

Wrote `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Chat`, `ChatIcon`, `REG_DWORD` = 3
(Disabled), stock default `absent`.

**Delete.** Microsoft's own 23H2 release notes state "Microsoft Teams: Chat is being removed from the
Microsoft Teams in-box app." The Chat flyout the policy governs was replaced by a normal pinnable
"Microsoft Teams (free)" taskbar app that `ChatIcon` does not control, and the backing
`ConfigureChatIcon` policy carries an explicit deprecation note in the Policy CSP. The feature was
removed one release before the support floor and never existed on Windows 10, so no supported
platform remains. `interface:disable_chat_taskbar` (`TaskbarMn`) is the same vanished feature by a
different mechanism and is recommended for deletion in that category for the same reason.

Sources: Microsoft, What's new in Windows 11 version 23H2,
https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2 (tier A);
Policy CSP - Experience, `ConfigureChatIcon`, deprecation note,
https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A).

### `remove_cortana` Cortana

Wrote `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`, `AllowCortana`, `REG_DWORD` = 0,
plus removal of the `Microsoft.549981C3F5F10` Appx package with undo
`winget install --id 9NFFX4SZZ23L`.

**Delete.** Cortana as a standalone Windows app was retired in spring 2023 and is on Microsoft's
deprecated-features list. `Microsoft.549981C3F5F10` is not part of the 24H2 or 25H2 preinstalled
Store app set, and Windows 10 IoT Enterprise LTSC ships no Store apps at all, so the app half finds
nothing on either target. The undo was already impossible: the Store catalog record for
`9NFFX4SZZ23L` still exists but `winget show --id 9NFFX4SZZ23L --exact` returns "No package found
matching input criteria" from every source on winget 1.29.280, because the app has been delisted.
The `AllowCortana` policy half is also pointless with no Cortana present.

Sources: Microsoft, End of support for Cortana,
https://support.microsoft.com/en-us/topic/end-of-support-for-cortana-d025b39f-ee5b-4836-a954-0ab646ee1efa
(tier A); Deprecated features in the Windows client,
https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A);
`winget show --id 9NFFX4SZZ23L --exact` returns no package on winget 1.29.280 (primary observation).

### `remove_dev_home` Dev Home

Wrote removal of `Microsoft.Windows.DevHome` with undo `winget install --id 9N8MHTPHNGVV`.

**Delete.** Microsoft states "Starting May 2025, Dev Home will no longer be supported as a feature in
Windows 11", archived the GitHub repository in June 2025, and dropped the app from the preinstalled
set. It is absent from the 24H2 and 25H2 policy-removable inbox app list. Worse, the Store listing
was reused: `9N8MHTPHNGVV` now resolves to title "Windows Advanced Settings" on package family
`Microsoft.Windows.DevHome_8wekyb3d8bbwe`, so on a current machine the apply removes a supported
successor app and the undo installs that successor rather than Dev Home. The shared package identity
means the probe cannot distinguish the two apps. Removing a tweak whose only remaining effect is to
uninstall a different, supported application is the right call.

Sources: Microsoft, Dev Home documentation, https://learn.microsoft.com/en-us/windows/dev-home/
(tier A); Microsoft Store catalog service, product `9N8MHTPHNGVV` returns title "Windows Advanced
Settings", package family `Microsoft.Windows.DevHome_8wekyb3d8bbwe` (tier A, primary observation);
Policy-based inbox app removal app list,
https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal
(tier A).

### `remove_maps` Windows Maps

Wrote removal of `Microsoft.WindowsMaps` with undo `winget install --id 9WZDNCRDTBVB`.

**Delete.** This is the most explicit statement in the whole review. Microsoft: "Maps is no longer
preinstalled with Windows starting with the Windows 11, version 24H2 release." The app was pulled
from the Store in July 2025 and a final update rendered it non-functional, so it cannot be
reinstalled even by users who previously had it; `winget show --id 9WZDNCRDTBVB --exact` returns no
package on winget 1.29.280. Microsoft also deprecated the UWP Map control and the
`Windows.Services.Maps` platform APIs on 8 April 2025. Nothing to remove, and `reversible: true` was
never true for it.

Sources: Deprecated features resources, "Maps is no longer preinstalled with Windows starting with
the Windows 11, version 24H2 release",
https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources (tier A);
Deprecated features in the Windows client, UWP Map control deprecation 8 April 2025,
https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); "Microsoft kills
Windows Maps app", Neowin, https://www.neowin.net/news/microsoft-kills-windows-maps-app/ (tier C).

### `remove_people` People app

Wrote removal of `Microsoft.People` with undo `winget install --id 9NBLGGH10PG8`.

**Delete.** Microsoft lists My People and People in the Shell as deprecated ("My People is no longer
being developed"), `Microsoft.People` is absent from the 24H2 and 25H2 policy-removable inbox app
list, and People is part of the Mail / Calendar / People family that Microsoft names as excluded from
LTSC editions, so it is absent on both targets. The new Outlook's own Store listing states it
"will replace the Windows Mail, Calendar, and People apps beginning in 2024". The undo was already
broken: `winget show --id 9NBLGGH10PG8 --exact` returns no package on winget 1.29.280 even though the
catalog record resolves.

Sources: Deprecated features in the Windows client,
https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A); Policy-based inbox
app removal app list,
https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal
(tier A); Windows as a service overview, LTSC excluded app list,
https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A).

## New in this revision

Five tweaks are added, all adversarially verified against shipped ADMX, shipped binaries and
Microsoft Learn before inclusion.

| Tweak | Verification | What it adds |
|---|---|---|
| `disable_account_notifications` | CONFIRMED | Turns off the Microsoft account nags in the Start user tile (reauthenticate, back up your device, storage quota, Microsoft 365 and Xbox subscription prompts). Shipped `AccountNotifications.admx` is exact, `class="User"`, and `windowsudk.shellcommon.dll` reads the value. `requires_reboot` must be `false`. |
| `disable_settings_account_ads` | CORRECTED | Replaces the account-state upsell cards in Settings with default fallback content. Mechanism verified exactly, but applicability is Windows 11 21H2 and later (not Windows 10 2004) and it is a no-op on Pro and Home. |
| `disable_windows_spotlight_all` | CONFIRMED | One master switch for every Windows Spotlight surface, matching `CloudContent.admx` exactly. Overlaps four shipped tweaks, which the copy must disclose. |
| `disable_spotlight_desktop` | CONFIRMED | Stops the rotating Spotlight desktop wallpaper collection. Subset of `disable_windows_spotlight_all`; the relationship must be stated. |
| `disable_nag_toasts` | CONFIRMED | Silences the "Suggested" ad toasts and the Windows Backup reminder toasts individually, as the surgical alternative to `interface:disable_toast_notifications`. Both toast identifiers exist in shipped 26100 binaries. The apply must create the lazily-created keys. |

Two other proposals routed through this category during the gap hunt went elsewhere:
`disable_edge_ai_features` to `ai.md` (three of its five values are documented Entra-only and change
nothing on a consumer Microsoft account profile) and `disable_online_tips` to `privacy.md`.

## Merge candidates

**Group A: the four `ContentDeliveryManager` per-user suggestion switches.**
`disable_start_suggestions`, `disable_auto_install_sponsored_apps`, `disable_welcome_experience`,
`disable_scoobe_nag`. All four are per-user, HKCU, `REG_DWORD`, undocumented, low risk, and a user
who wants one usually wants all of them. Proposed shape: one tweak, "Consumer suggestions and nags",
with options "Off" and "On (Stock Default)", writing the full value set including
`ContentDeliveryAllowed`, `SystemPaneSuggestionsEnabled` and `Start_IrisRecommendations`.
Granularity lost: a user who wants Start promotions gone but still wants the post-update welcome tour
can no longer split them. Consolidating also fixes the current problem that two of these tweaks
describe values their effects do not write.

**Group B: the three HKLM Edge policy tweaks.** `disable_edge_first_run`,
`disable_edge_startup_boost`, `disable_edge_sidebar`. A user configures these in one sitting.
Proposed shape: a single "Microsoft Edge cleanup" tweak with per-effect options, or at minimum keep
them separate but split `EdgeCollectionsEnabled` out of `disable_edge_sidebar`. Granularity lost:
none if implemented as a multi-option tweak. The concrete win is un-bundling Collections, which is a
real feature with real users and does not belong behind a switch named "sidebar and Discover".

**Group C: `disable_windows_spotlight_all` and `disable_spotlight_desktop`.** The second is a strict
subset of the first. Proposed shape: one Spotlight tweak with options `On (Stock Default)` /
`Desktop collection off` / `All Spotlight features off`. Granularity lost: none. If they ship
separately, only one should be marked recommended.

**Group D: `remove_bing_news_weather` and `disable_widgets`.** Not a merge, but they overlap in user
intent and the tweak text already acknowledges it. Worth a cross-reference rather than a merge,
because the Widgets policy is machine-scoped and Windows 11 only while the app removal is per-package.

**Group E: the eleven single-app removals.** These share one template and one set of defects. Rather
than merging them, which would destroy the per-app choice that is the whole point, the right
consolidation is at the engine level: one app-removal action kind that takes a package family name
and a Store ID, handles the provisioned package, and has one correct fail-closed probe. Granularity
lost: none.

## Tweak entries

### `disable_start_suggestions` Start menu app promotions

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated, not Microsoft-documented)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value
`SubscribedContent-338388Enabled`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `SubscribedContent-338388Enabled` = 0 |
| On (Stock Default) | **CORRECTED: value absent.** As authored: 1 |

The corrected stock default is `absent`. The value is not present in the shipped
`C:\Users\Default\NTUSER.DAT` on build 26100.4061, so a fresh profile carries no value at all and
writing 1 on revert leaves the machine in a state it was never in.

*What it actually does.* The `ContentDeliveryManager` key is the per-user backing store for the
Windows Spotlight and suggestion surfaces, and each `SubscribedContent-<ID>Enabled` value gates one
content slot. The prefix `SubscribedContent-` is present as a literal in the shipped
`ContentDeliveryManager.Utilities.dll` and `ContentDeliveryManager.Background.dll` on 26100.4061,
with the numeric ID appended at runtime from the subscription identifier. That is why no individual
ID appears in a binary or in Microsoft documentation, and why the ID to surface mapping can only be
established empirically. Four independent tier C sources agree on the exact key, value name,
`REG_DWORD` type and polarity (0 = off, 1 = on) for 338388, and all four map it to the **Windows 10**
Start toggle "Occasionally show suggestions in Start".

*The Windows 11 surface is different.* "Show recommendations for tips, shortcuts, new apps, and more"
is driven by `Start_IrisRecommendations` under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` (a literal in the shipped
`StartTileData.dll` on 26100.4061), plus the tier A `HideRecommendedSection` policy in
`StartMenu.admx` at `Software\Policies\Microsoft\Windows\Explorer`. Neither is written by this tweak.
`interface:disable_start_recommendations` is the corpus tweak that owns the Windows 11 control.

*Applicability.* The key exists on Windows 10 1607 and later and on all Windows 11 builds. Per user,
no elevation, matching `elevation: user`. No reboot, matching the absent `requires_reboot`.

*Cautions.* Microsoft moves these slots between releases without notice. Sophia Script, which tracks
Windows 11 25H2 and later, has dropped 338388 entirely and now uses `Start_IrisRecommendations`.

**Corrections needed:** Three items. (1) "On (Stock Default)" must be `absent`, not 1. (2) The info
text claims the tweak clears `Start_IrisRecommendations` on 24H2. It does not. Add that effect, or
add the tier A `HideRecommendedSection` policy, or remove the claim. (3) The info text uses the
Windows 11 wording; every source maps 338388 to the Windows 10 wording. Correct the description or
gate the tweak to the secondary Windows 10 target.

**Ready-to-paste info block:**

```yaml
    info: |
      **Clears promoted-app and tip cards out of the Start menu's suggestion slot.**

      ## What it does
      Clears `SubscribedContent-338388Enabled` under the per-user `ContentDeliveryManager` key, the
      switch behind the Start menu suggestion slot. On Windows 11 the visually similar
      "recommendations" row is a separate control, `Start_IrisRecommendations`, which this tweak
      does not write.

      ## Benefits
      - **No promoted apps**: the suggestion slot stops surfacing apps you never asked for
      - **Fewer nudges**: one less Microsoft content channel pointed at the Start menu
      - **Per-user and instant**: no policy, no elevation, no reboot

      ## Drawbacks
      - **Partial on Windows 11**: the 24H2 recommendations row is driven by a different value, so
        use the Start recommendations tweak as well if that row is what bothers you
      - **Undocumented slot**: Microsoft composes these IDs at runtime and can move the surface
        between releases
      - **May look like nothing changed**: on a machine where the slot is already empty there is no
        visible difference

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the surface it maps to is the Windows 10 Start
        suggestion toggle
      - **Takes effect**: immediately, Start may need a sign-out to redraw
      - **Reverting**: deletes the value so the profile returns to its shipped state
      - Pair with the Start recommendations tweak in Interface for the Windows 11 row

      ## Recommendation
      Apply it if you want every Start suggestion channel closed. If your only complaint is the
      Windows 11 recommendations row, the Interface tweak for that row is the one that fixes it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Turn on or off app suggestions in Start (TenForums 24117)](https://www.tenforums.com/tutorials/24117-turn-off-app-suggestions-start-windows-10-a.html)
      - [Win11Debloat, Disable_Windows_Suggestions.reg](https://github.com/Raphire/Win11Debloat)
      - [Policy CSP - Experience, AllowWindowsSpotlight](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
```

**Sources:**
1. Shipped `StartMenu.admx` on build 26100.4061: policy `HideRecommendedSection`, key `Software\Policies\Microsoft\Windows\Explorer` (tier A, shipped ADMX)
2. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: no `SubscribedContent-*Enabled` value present (tier A, shipped OS data, primary observation)
3. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100.4061: `SubscribedContent-` in `ContentDeliveryManager.Utilities.dll` and `ContentDeliveryManager.Background.dll`; `Start_IrisRecommendations` in `StartTileData.dll` (tier A, product artifact, primary observation)
4. Brink, "Turn On or Off App Suggestions in Start in Windows 10", TenForums tutorial 24117, https://www.tenforums.com/tutorials/24117-turn-off-app-suggestions-start-windows-10-a.html (tier C)
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C)
6. Sophia Script for Windows 11 v7.1.6 (2026-06-16), function `StartRecommendedSection`, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
7. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C)
8. Disassembler0, Win10-Initial-Setup-Script, `DisableAppSuggestions`, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C)
9. Policy CSP - Experience, `AllowWindowsConsumerFeatures` and `AllowWindowsSpotlight`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)

### `disable_auto_install_sponsored_apps` Auto-installed sponsored apps

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated, not Microsoft-documented)

**Mechanism (CORRECTED):** three `REG_DWORD` values under
`HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`.

| Effect | Value name | Type | Off | On (Stock Default) |
|---|---|---|---|---|
| `silent` | `SilentInstalledAppsEnabled` | `REG_DWORD` | 0 | 1 |
| `preinstalled` | `PreInstalledAppsEnabled` | `REG_DWORD` | 0 | 1 |
| `oem` | `OemPreInstalledAppsEnabled` | `REG_DWORD` | 0 | 1 |

**Wrong mechanism as authored, for reference.** The YAML declares a fourth effect,
`subscribed` = `SubscribedContentEnabled`, `REG_DWORD`, Off = 0, Stock Default = 1. **Drop it.**
`SubscribedContentEnabled` (with no numeric ID) is absent from the shipped Default user hive, absent
from the live hive, and a string scan of 4051 shipped `System32` and `SystemApps` modules on
26100.4061 found it in **zero** modules, while every sibling name from the same key
(`ContentDeliveryAllowed`, `SystemPaneSuggestionsEnabled`, `SoftLandingEnabled`,
`RotatingLockScreenOverlayEnabled`, `PreInstalledAppsEnabled`, and the `SubscribedContent-` prefix)
was found. No independent tool or tutorial writes it. As authored it is a no-op that pollutes the
profile on both apply and revert.

*What it actually does.* The three remaining values gate the silent post-OOBE installation of
promoted, preinstalled and OEM-supplied Store apps. All three are **present with data 1** in the
shipped `C:\Users\Default\NTUSER.DAT` on 26100.4061, which is the template every new profile is
cloned from, so Windows itself seeds them and the stock default of 1 is correct for these three.
Four independent tier C sources agree on the key, the three names, the type and the polarity, and
`PreInstalledAppsEnabled` appears as a literal in the shipped `StartTileData.dll`.

*Applicability.* Windows 10 1607 and later, all Windows 11 builds, all SKUs. Per user, no elevation,
no reboot. Only prevents future installs; apps already present are untouched.

*Cautions.* The documented machine-wide equivalent is `DisableWindowsConsumerFeatures` under
`HKLM\Software\Policies\Microsoft\Windows\CloudContent`, which Microsoft describes as covering
"Post-OOBE app install and redirect tiles", but it is supported only on Enterprise, Education and IoT
Enterprise. On Pro and Home this per-user key is the only lever, and it is not enforced against a
determined re-provisioning by a feature update.

**Corrections needed:** Two items. (1) Drop the `SubscribedContentEnabled` effect entirely, per the
evidence above. (2) The info text says the tweak clears `ContentDeliveryAllowed`; it does not. Either
add `ContentDeliveryAllowed` as a real effect (it is genuine: the string appears in
`ContentDeliveryManager.Utilities.dll`, `ContentDeliveryManager.Background.dll`,
`SettingsHandlers_ContentDeliveryManager.dll`, `CloudExperienceHostCommon.dll`,
`Windows.UI.Immersive.dll` and `StartTileData.dll`) or remove the claim. No change is needed to the
stock default of 1 for the three real values; the shipped Default user hive confirms it.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows from silently installing promoted, preinstalled and OEM apps into your
      account.**

      ## What it does
      Clears `SilentInstalledAppsEnabled`, `PreInstalledAppsEnabled` and `OemPreInstalledAppsEnabled`
      under the per-user `ContentDeliveryManager` key. Those three values gate the post-setup silent
      install of promoted Store apps, the bundled app set, and anything the OEM added.

      ## Benefits
      - **No surprise apps**: game and trial tiles stop appearing in Start on their own
      - **Survives new accounts**: each profile gets its own copy of the switch, so setting it early
        keeps the account clean
      - **Reversible per value**: the snapshot restores the exact prior data

      ## Drawbacks
      - **Future installs only**: promo apps already on the machine must be uninstalled separately
      - **Not enforced**: a feature update can re-provision apps regardless of these per-user values
      - **No policy backing on Home and Pro**: the machine-wide consumer-features policy is
        Enterprise, Education and IoT Enterprise only

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: immediately, for any install that has not already started
      - **Reverting**: restores the shipped value of 1 for all three
      - Pair with the consumer features tweak in Privacy on Enterprise, Education and IoT Enterprise,
        where that policy is honoured

      ## Recommendation
      Apply it. There is no scenario where you want Windows quietly installing sponsored apps into
      your profile, and the only cost is that you install wanted apps yourself.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Turn off automatic installation of suggested apps (TenForums 68217)](https://www.tenforums.com/tutorials/68217-turn-off-automatic-installation-suggested-apps-windows-10-a.html)
      - [Sophia Script for Windows, AppsSilentInstalling](https://github.com/farag2/Sophia-Script-for-Windows)
      - [Policy CSP - Experience, AllowWindowsConsumerFeatures](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
```

**Sources:**
1. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: `SilentInstalledAppsEnabled`, `PreInstalledAppsEnabled` and `OemPreInstalledAppsEnabled` all present as `REG_DWORD` 1; `SubscribedContentEnabled` absent (tier A, shipped OS data, primary observation)
2. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100.4061: `SubscribedContentEnabled` found in zero modules; `PreInstalledAppsEnabled` and `ContentDeliveryAllowed` found in `StartTileData.dll` (tier A, product artifact, primary observation)
3. Brink, "Turn Off Automatic Installation of Suggested Apps in Windows 10", TenForums tutorial 68217, https://www.tenforums.com/tutorials/68217-turn-off-automatic-installation-suggested-apps-windows-10-a.html (tier C)
4. Sophia Script for Windows 11 v7.1.6, function `AppsSilentInstalling`, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C)
6. Disassembler0, Win10-Initial-Setup-Script, `DisableAppSuggestions`, https://github.com/Disassembler0/Win10-Initial-Setup-Script (tier C)
7. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, which lists the three but not `SubscribedContentEnabled`, https://github.com/Biswa96/WinLight (tier C)
8. Policy CSP - Experience, `AllowWindowsConsumerFeatures`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)

### `disable_welcome_experience` Post-update welcome experience

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated, not Microsoft-documented)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`, value
`SubscribedContent-310093Enabled`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `SubscribedContent-310093Enabled` = 0 |
| On (Stock Default) | **CORRECTED: value absent.** As authored: 1 |

*What it actually does.* Suppresses the full-screen "here is what is new" page shown after some
cumulative and feature updates. Three genuinely independent tier C sources give the same key, value
name, `REG_DWORD` type and surface, quoting the Settings toggle almost verbatim: "Show me the Windows
welcome experience after updates and occasionally when I sign in to highlight what's new and
suggested". They agree the toggle is on by default and that 1 = shown, 0 = hidden.

Microsoft corroborates that the surface exists and is separately controllable: `CloudContent.admx`
shipped on 26100.4061 carries the User-class policy
`DisableWindowsSpotlightWindowsWelcomeExperience` under
`Software\Policies\Microsoft\Windows\CloudContent`, supported from Windows 10 RS2. That policy is
Enterprise, Education and IoT Enterprise only. The 310093 value is the consumer-reachable
equivalent, and Microsoft does not document it because the numeric part of a
`SubscribedContent-<ID>Enabled` name is composed at runtime from the subscription identifier.

*Applicability.* Windows 10 1703 and later, all Windows 11 builds, all SKUs. Per user, no elevation,
no reboot. The value is **absent** from the shipped `C:\Users\Default\NTUSER.DAT` on 26100.4061.

*Cautions.* Microsoft can relocate the slot between releases. As of Sophia Script v7.1.6 (June 2026,
targeting Windows 11 25H2 and later) 310093 is still the current value, so it has been stable across
at least Windows 10 1703 through Windows 11 25H2.

**Corrections needed:** "On (Stock Default)" = 1 is wrong. The value is absent from the shipped
Default user hive on 26100.4061, so the correct stock default is `absent`. Writing an explicit 1 on
revert leaves state the machine never had and masks any later change to the Windows default.

**Ready-to-paste info block:**

```yaml
    info: |
      **Skips the full-screen "what's new" tour after an update so you land straight on your
      desktop.**

      ## What it does
      Clears `SubscribedContent-310093Enabled` under the per-user `ContentDeliveryManager` key, the
      value behind Settings > System > Notifications > "Show me the Windows welcome experience after
      updates". The update itself is untouched.

      ## Benefits
      - **No post-update splash**: you reach the desktop instead of a promotional tour
      - **Fewer sign-in interruptions**: the same slot drives the occasional sign-in highlight page
      - **Purely cosmetic**: nothing functional depends on the welcome screen

      ## Drawbacks
      - **No change summary**: you no longer get the built-in "here is what changed" page
      - **Undocumented slot**: the numeric ID is composed at runtime, so Microsoft can move it
      - **Only this surface**: other post-update prompts, such as the setup nag, need their own tweak

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: immediately, from the next update onward
      - **Reverting**: deletes the value so the profile returns to its shipped state
      - The Enterprise-only policy equivalent is `DisableWindowsSpotlightWindowsWelcomeExperience`

      ## Recommendation
      Apply it if you already track what changes in Windows updates or simply do not want a tour.
      Leave it on if you like being shown new features after an update.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or disable Windows welcome experience (ElevenForum 3657)](https://www.elevenforum.com/t/enable-or-disable-windows-welcome-experience-in-windows-11.3657/)
      - [Sophia Script for Windows, WindowsWelcomeExperience](https://github.com/farag2/Sophia-Script-for-Windows)
      - [Policy CSP - Experience, AllowWindowsSpotlightWindowsWelcomeExperience](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
```

**Sources:**
1. Shipped `CloudContent.admx` on build 26100.4061: policy `DisableWindowsSpotlightWindowsWelcomeExperience`, class User, key `Software\Policies\Microsoft\Windows\CloudContent`, `supportedOn` Windows 10 RS2 (tier A, shipped ADMX)
2. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: `SubscribedContent-310093Enabled` absent (tier A, shipped OS data, primary observation)
3. Brink, "Enable or Disable Windows Welcome Experience in Windows 11", ElevenForum tutorial 3657, https://www.elevenforum.com/t/enable-or-disable-windows-welcome-experience-in-windows-11.3657/ (tier C)
4. Sophia Script for Windows 11 v7.1.6, function `WindowsWelcomeExperience`, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C)
6. Biswa96, WinLight `ContentDeliveryManager.reg.ini`, https://github.com/Biswa96/WinLight (tier C)
7. Policy CSP - Experience, `AllowWindowsSpotlightWindowsWelcomeExperience`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)

### `disable_scoobe_nag` Get even more out of Windows nag

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated, not Microsoft-documented)

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement`, value
`ScoobeSystemSettingEnabled`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `ScoobeSystemSettingEnabled` = 0 |
| On (Stock Default) | **CORRECTED: value absent.** As authored: 1 |

*What it actually does.* SCOOBE is the second-run out-of-box experience, the post-sign-in full-screen
prompt that pushes a Microsoft account, OneDrive backup, Edge as default and Microsoft 365.
`UserProfileEngagement\ScoobeSystemSettingEnabled` is the per-user switch that Settings > System >
Notifications > "Suggest ways to get the most out of Windows and finish setting up this device"
writes.

That the shell reads this value is established directly: a string scan of 4051 shipped `System32` and
`SystemApps` modules on 26100.4061 found the literal `ScoobeSystemSettingEnabled` in exactly two
modules, `SettingsHandlers_nt.dll` (the Settings handler that backs the System > Notifications page)
and `windowsudk.shellcommon.dll`. Three independent tier C sources, spanning both the Windows 10 and
Windows 11 wording of the toggle, agree on the key, value name, type, polarity and default-on state.
Microsoft documents neither the key nor the value in any Learn article or ADMX.

*Applicability.* Windows 10 1803 and later, all Windows 11 builds. Per user, no elevation, no reboot.
The corresponding Settings checkbox is present on Home and Pro, so the value is reachable on consumer
editions, unlike the CloudContent policies. The entire `UserProfileEngagement` key is **absent** from
the shipped `C:\Users\Default\NTUSER.DAT` on 26100.4061, so it is created lazily rather than seeded.

*Cautions.* Microsoft has shipped builds where the SCOOBE screen is gated by additional server-side
flags, so the prompt can still appear once after a large feature update even with this set.

**Corrections needed:** "On (Stock Default)" = 1 is wrong. Neither the value nor its parent
`UserProfileEngagement` key exists in the shipped Default user hive on 26100.4061, so the correct
stock default is `absent`. Reverting by writing 1 creates a key and a value the machine never had.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the full-screen "finish setting up your device" prompt after you sign in.**

      ## What it does
      Clears `ScoobeSystemSettingEnabled` under `HKCU\...\UserProfileEngagement`, the value behind
      Settings > System > Notifications > "Suggest ways to get the most out of Windows and finish
      setting up this device". That is the screen pushing a Microsoft account, OneDrive backup and
      Edge as default.

      ## Benefits
      - **No sign-in upsell**: you reach the desktop instead of a setup wizard
      - **Reachable on Home**: unlike the policy alternatives, this value works on every edition
      - **Instant**: takes effect with no reboot or service restart

      ## Drawbacks
      - **Not absolute**: some builds gate the screen on server-side flags too, so it can still
        appear once after a large feature update
      - **Undocumented**: Microsoft has no Learn page or ADMX for this key
      - **Per user**: a second account on the machine needs it applied separately

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: immediately, from the next sign-in onward
      - **Reverting**: deletes the value, and the key with it, so the profile returns to shipped state
      - The key does not exist on a fresh profile; apply creates it

      ## Recommendation
      Apply it. The screen offers nothing you cannot reach from Settings and it interrupts every
      sign-in that follows a large update.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Enable or disable "Let's finish setting up your device" (ElevenForum 5205)](https://www.elevenforum.com/t/enable-or-disable-lets-finish-setting-up-your-device-in-windows-11.5205/)
      - [Sophia Script for Windows, WhatsNewInWindows](https://github.com/farag2/Sophia-Script-for-Windows)
      - [Win11Debloat, Disable_Windows_Suggestions.reg](https://github.com/Raphire/Win11Debloat)
```

**Sources:**
1. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100.4061: `ScoobeSystemSettingEnabled` present in `SettingsHandlers_nt.dll` and `windowsudk.shellcommon.dll` only (tier A, product artifact, primary observation)
2. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: the `UserProfileEngagement` key does not exist (tier A, shipped OS data, primary observation)
3. Brink, "Enable or Disable Let's finish setting up your device in Windows 11", ElevenForum tutorial 5205, https://www.elevenforum.com/t/enable-or-disable-lets-finish-setting-up-your-device-in-windows-11.5205/ (tier C)
4. Sophia Script for Windows 11 v7.1.6, function `WhatsNewInWindows`, which creates the key first because it may not exist, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
5. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, https://github.com/Raphire/Win11Debloat (tier C)
6. hellzerg, Optimizer, `OptimizeHelper.cs`, writes the same value under the same key, https://github.com/hellzerg/optimizer (tier C)
7. No Microsoft Learn or ADMX documentation exists for this key; the Policy CSP index and the shipped `PolicyDefinitions` set were searched without a match (primary observation)

### `disable_explorer_sync_ads` File Explorer sync-provider ads

**Verdict:** VERIFIED

**Mechanism:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, value
`ShowSyncProviderNotifications`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `ShowSyncProviderNotifications` = 0 |
| On (Stock Default) | 1 |

*What it actually does.* This is the backing value for the File Explorer folder option "Show sync
provider notifications" on the View tab of Folder Options. Despite the name, the notifications it
controls are largely promotional banners for OneDrive and Microsoft 365 rendered in the Explorer
window chrome, not sync status. Setting it to 0 removes those banners.

Windows declares the option itself, in shipped OS data. The key
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\ShowSyncProviderNotifications`
on build 26100.4061 contains the full definition: `ValueName` = `ShowSyncProviderNotifications`,
`RegPath` = `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, `HKeyRoot` = `0x80000001`
(HKEY_CURRENT_USER), `Type` = `checkbox`, `CheckedValue` = 1, `UncheckedValue` = 0, `DefaultValue` =
1, and `Text` = `@shell32.dll,-30552`, which resolves on this machine to the literal string "Show
sync provider notifications". That is Microsoft's own declaration of hive, key, value name, type,
polarity and default, and it takes this tweak past the community-corroboration route entirely. A
string scan of 4051 shipped modules found the value name in exactly one module, `shell32.dll`, which
is the component that owns the folder option.

*Applicability.* Windows 10 1703 and later, all Windows 11 builds, all SKUs. Per user, no elevation.
Explorer picks the change up on the next window or after an Explorer restart; no reboot needed, which
matches the absent `requires_reboot`.

*Cautions.* The value is absent from the shipped `C:\Users\Default\NTUSER.DAT` on 26100.4061, so a
fresh profile carries no value. Unlike the other tweaks in this group, writing 1 on revert is still
correct, because Windows declares `DefaultValue` = 1, so an explicit 1 and an absent value behave
identically. `absent` is the more faithful restoration; 1 is not a defect.

**Corrections needed:** None mechanically. One text correction: the info text claims the tweak "only
affects those promotional notifications". The checkbox governs genuine sync-provider notifications
too, so that separation overstates the case. Optionally change the stock default to `absent` for
fidelity, which is behaviourally identical here.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the OneDrive and Microsoft 365 upsell banners from File Explorer.**

      ## What it does
      Clears `ShowSyncProviderNotifications` under `HKCU\...\Explorer\Advanced`, the value behind the
      "Show sync provider notifications" checkbox in Folder Options > View. Windows declares that
      option in shipped OS data with a default of 1, so this is the same switch the UI writes.

      ## Benefits
      - **No upsell banners**: the Explorer header stops advertising OneDrive and Microsoft 365
      - **Documented switch**: Windows itself declares the value, type and default
      - **No elevation**: a per-user preference, applied instantly

      ## Drawbacks
      - **Real sync notices go too**: the checkbox governs genuine sync-provider notifications, not
        only the promotional ones
      - **Explorer only**: OneDrive's own tray notifications are unaffected
      - **Per user**: other accounts on the machine need it applied separately

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: after restarting Explorer, or on the next new Explorer window
      - **Reverting**: restores the checkbox to its shipped checked state
      - Same as unticking Folder Options > View > "Show sync provider notifications"

      ## Recommendation
      Apply it unless you actively rely on OneDrive status messages appearing inside Explorer. For
      everyone else this is an advertising surface in a window you open dozens of times a day.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Enable or disable sync provider notifications (ElevenForum 5200)](https://www.elevenforum.com/t/enable-or-disable-sync-provider-notifications-in-file-explorer-in-windows-11.5200/)
      - [privacy.sexy, Disable sync provider notifications](https://github.com/undergroundwires/privacy.sexy)
      - [Sophia Script for Windows, OneDriveFileExplorerAd](https://github.com/farag2/Sophia-Script-for-Windows)
```

**Sources:**
1. Shipped folder-option definition at `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Folder\ShowSyncProviderNotifications` on build 26100.4061: `ValueName`, `RegPath`, `HKeyRoot` 0x80000001, `Type` checkbox, `CheckedValue` 1, `UncheckedValue` 0, `DefaultValue` 1, `Text` `@shell32.dll,-30552` (tier A, shipped OS metadata)
2. `shell32.dll` string resource 30552 on build 26100.4061 resolves to "Show sync provider notifications" (tier A, shipped OS resource)
3. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100.4061: `ShowSyncProviderNotifications` present only in `shell32.dll` (tier A, product artifact, primary observation)
4. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: the value is absent from `Explorer\Advanced` (tier A, shipped OS data, primary observation)
5. Brink, "Enable or Disable Sync Provider Notifications in File Explorer in Windows 11", ElevenForum tutorial 5200, https://www.elevenforum.com/t/enable-or-disable-sync-provider-notifications-in-file-explorer-in-windows-11.5200/ (tier C)
6. privacy.sexy, "Disable sync provider notifications", marked `deleteOnRevert` with the annotation "Missing by default since Windows 10 Pro (>= 22H2) and Windows 11 Pro (>= 23H2)", https://github.com/undergroundwires/privacy.sexy (tier C)
7. Sophia Script for Windows 11 v7.1.6, function `OneDriveFileExplorerAd`, https://github.com/farag2/Sophia-Script-for-Windows (tier C)

### `disable_web_search_start` Web results in Start search

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** two effects.

| Effect | Key | Value name | Type |
|---|---|---|---|
| `box_suggestions` | `HKCU\Software\Policies\Microsoft\Windows\Explorer` | `DisableSearchBoxSuggestions` | `REG_DWORD` |
| `bing_enabled` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search` | `BingSearchEnabled` | `REG_DWORD` |

| Option | Values |
|---|---|
| Off | `DisableSearchBoxSuggestions` = 1, `BingSearchEnabled` = 0 |
| On (Stock Default) | `DisableSearchBoxSuggestions` absent, **CORRECTED: `BingSearchEnabled` absent.** As authored: 1 |

`requires_reboot: true`.

*What it actually does.* `C:\Windows\PolicyDefinitions\WindowsExplorer.admx` shipped on build
26100.4061 contains:

```xml
<policy name="DisableSearchBoxSuggestions" class="User"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="DisableSearchBoxSuggestions">
  <supportedOn ref="windows:SUPPORTED_Windows7" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

`class="User"` means HKCU, so the key path, value name, `REG_DWORD` type and polarity in the YAML
match Microsoft's shipped definition exactly, and "not configured" is expressed by the value being
absent. That is a tier A confirmation of the mechanism.

The **semantics** are a separate question. The shipped `WindowsExplorer.adml` names this policy "Turn
off display of recent search entries in the File Explorer search box" and explains it as suppressing
suggestion pop-ups built from the user's past Search Box entries and preventing those entries from
being stored in the registry. Microsoft therefore documents a File Explorer search-history behaviour,
not Bing suppression in Start. The additional effect the tweak relies on, that the same value also
removes web and Bing suggestions from the Start and taskbar search flyout on Windows 10 20H2 and
later, is corroborated by three independent tier C sources but is nowhere stated by Microsoft. On
26100.4061 the literal appears in `SHCore.dll` only, and not in the search host's own
`SearchUx.Core.dll` or `SearchUx.UI.dll`, which is consistent with the search host reading it through
a shared shell policy helper but is not proof either way. String absence in a binary is not proof a
policy is ignored.

`BingSearchEnabled` is still live but is the weaker half. On 26100.4061 the literal appears in
`SearchUx.Core.dll` and `SearchUx.UI.dll` (both inside
`C:\Windows\SystemApps\MicrosoftWindows.Client.CBS_cw5n1h2txyewy`, the current Start and taskbar
search host) and in `windowsudk.shellcommon.dll`, so it is referenced by the shipping 24H2 search UI.
But it is not a policy value, is undocumented on Learn, and is reported reset by feature updates.
`DisableSearchBoxSuggestions` is the value that carries the tweak.

The documented, first-party way to achieve the intended outcome is `DoNotUseWebResults`, which writes
`ConnectedSearchUseWeb` = 0 under `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`, but that
policy is supported only on Enterprise, Education and IoT Enterprise, not Pro and not Home, which is
why the HKCU policy value remains the right default for a consumer-facing tool.

*Applicability.* The ADMX declares `supportedOn` = Windows 7 and later, so the policy exists across
the whole supported range on all SKUs; it is a per-user policy value and is not edition gated the way
the CloudContent and Windows Search policies are. Per user, no elevation needed, and `elevation:
user` matches. A restart of the search host, an Explorer restart, or a sign-out is required;
`requires_reboot: true` is a conservative and honest fit. Neither
`Software\Policies\Microsoft\Windows\Explorer` nor `Software\Microsoft\Windows\CurrentVersion\Search`
exists in the shipped `C:\Users\Default\NTUSER.DAT`, so both values are absent on a fresh profile.

*Cautions.* Reports of this setting disabling the taskbar search box entirely on some builds exist on
Microsoft Q&A. Windows updates have been observed resetting `BingSearchEnabled`. 24H2 also added
Settings > Privacy and security > Search permissions with a cloud content control, which is the
in-box equivalent.

**Corrections needed:** Three items. (1) "On (Stock Default)" writes `BingSearchEnabled: 1`. The
whole `CurrentVersion\Search` key is absent from the shipped Default user hive on 26100.4061, so
reverting creates state the machine never had. It must be `absent`, matching what the same option
already does for `box_suggestions`. (2) Either drop the `bing_enabled` effect or mark it
`skip_validation: true`, so a value the OS may silently reset cannot produce a false Needs Attention
state, and stop presenting it as the mechanism. (3) The info text must disclose the documented File
Explorer search-history side effect, since that is the only behaviour Microsoft attributes to the
policy. Optionally add `ConnectedSearchUseWeb` for the editions where it is supported.

**Ready-to-paste info block:**

```yaml
    info: |
      **Keeps Start search local, so typing an app name stops sending keystrokes to Bing.**

      ## What it does
      Sets the documented per-user policy `DisableSearchBoxSuggestions` = 1 under
      `HKCU\Software\Policies\Microsoft\Windows\Explorer` and clears `BingSearchEnabled`. Local
      results for apps, settings and files keep working; the web layer and the Copilot entry in the
      search flyout go away.

      ## Benefits
      - **Local-only results**: apps and files instead of web answers and promoted cards
      - **Less sent to Bing**: what you type in Start stops leaving the machine
      - **Policy backed**: `DisableSearchBoxSuggestions` is declared in the shipped
        `WindowsExplorer.admx` and works on every edition, including Home

      ## Drawbacks
      - **Explorer search history**: Microsoft's own description of this policy is that it stops
        File Explorer suggesting and storing your recent search entries, so you lose that too
      - **No web answers**: unit conversions, definitions and quick lookups from the search box stop
      - **Bing value is fragile**: `BingSearchEnabled` is undocumented and feature updates have been
        seen resetting it
      - **Taskbar search box reports**: a few builds have been reported hiding the box entirely

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: after reboot (a sign-out or an Explorer restart also works)
      - **Reverting**: deletes both values so the profile returns to its shipped state
      - Enterprise, Education and IoT Enterprise have a stronger machine-wide option,
        `DoNotUseWebResults`

      ## Recommendation
      Apply it if you use Start to launch apps and open settings, which is most people. Skip it if
      you deliberately use the Start box as a web search bar or rely on File Explorer's recent
      search suggestions.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Search, DoNotUseWebResults](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search)
      - [privacy.sexy, Disable Bing search and recent search suggestions](https://github.com/undergroundwires/privacy.sexy)
      - [Win11Debloat, Disable_Bing_Cortana_In_Search.reg](https://github.com/Raphire/Win11Debloat)
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\WindowsExplorer.admx` on build 26100.4061: policy `DisableSearchBoxSuggestions`, `class="User"`, `key="Software\Policies\Microsoft\Windows\Explorer"`, `valueName="DisableSearchBoxSuggestions"`, enabled = 1, disabled = 0, `supportedOn` Windows 7 (tier A, shipped ADMX)
2. Shipped `C:\Windows\PolicyDefinitions\en-US\WindowsExplorer.adml` on build 26100.4061: display name "Turn off display of recent search entries in the File Explorer search box" and its explain text (tier A, shipped ADMX)
3. Shipped `C:\Users\Default\NTUSER.DAT` on build 26100.4061: neither `Software\Policies\Microsoft\Windows\Explorer` nor `Software\Microsoft\Windows\CurrentVersion\Search` exists (tier A, shipped OS data, primary observation)
4. String scan of 4051 shipped `System32` and `SystemApps` modules on build 26100.4061: `DisableSearchBoxSuggestions` in `SHCore.dll` only; `BingSearchEnabled` in `SearchUx.Core.dll`, `SearchUx.UI.dll` and `windowsudk.shellcommon.dll` (tier A, product artifact, primary observation)
5. privacy.sexy, "Disable Bing search and recent search suggestions (breaks search history)", which writes `DisableSearchBoxSuggestions` = 1 and marks it `deleteOnRevert`, https://github.com/undergroundwires/privacy.sexy (tier C)
6. Sophia Script for Windows 11 v7.1.6, which writes the same policy value and removes it to re-enable, and reads both values to detect whether web results are already off, https://github.com/farag2/Sophia-Script-for-Windows (tier C)
7. Win11Debloat (Raphire), `Regfiles/Disable_Bing_Cortana_In_Search.reg`, https://github.com/Raphire/Win11Debloat (tier C)
8. Policy CSP - Search, `DoNotUseWebResults` / `ConnectedSearchUseWeb`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search (tier A)
9. "DisableSearchBoxSuggestions disables task bar search", Microsoft Q&A, https://learn.microsoft.com/en-us/answers/questions/3234673/disablesearchboxsuggestions-disables-task-bar-sear (tier D)
10. pureinfotech, "How to disable web search results on Windows 11", which names `DisableSearchBoxSuggestions` as the update-resistant value, https://pureinfotech.com/disable-search-web-results-windows-11/ (tier C)

### `disable_widgets` Widgets

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Dsh`, value `AllowNewsAndInterests`, type
`REG_DWORD`.

| Option | Value |
|---|---|
| Off | `AllowNewsAndInterests` = 0 |
| On (Stock Default) | value absent |

`windows: { products: [11] }`, `elevation: admin`, `requires_reboot: true`.

*What it actually does.* Microsoft documents this exactly. The NewsAndInterests Policy CSP maps
`AllowNewsAndInterests` to the Group Policy "Allow widgets" under Computer Configuration > Windows
Components > Widgets, registry key `SOFTWARE\Policies\Microsoft\Dsh`, value name
`AllowNewsAndInterests`, format `int` (which is `REG_DWORD` in the registry), default value 1,
allowed values 0 (not allowed) and 1 (allowed). The documentation states the policy "applies to the
entire widgets experience, including content on the taskbar". The WebExperience host package is not
uninstalled, which the tweak text states correctly.

*Applicability.* Device scope, Windows 11 version 21H2 (10.0.22000) and later, confirmed current on
24H2 and 25H2. Documented editions are Pro, Enterprise, Education and IoT Enterprise / IoT Enterprise
LTSC. **Windows 11 Home is not listed as a supported edition.** Requires administrator rights,
matching `elevation: admin`. A sign-out or reboot is needed for the taskbar to drop the entry point,
so `requires_reboot: true` is correct. Does not exist on Windows 10, and `products: [11]` is correct.

*Cautions.* On Windows 11 Home this policy is outside the documented support matrix. It is generally
honoured because the widgets host reads the value directly rather than through the Group Policy
engine, but that is not a Microsoft guarantee and should not be presented as one.
`interface:disable_news_interests` is the Windows 10 equivalent of the same taskbar feed feature and
is recommended for deletion under the 24H2 re-scope.

**Corrections needed:** None. Consider surfacing the Home caveat in the tweak text, since the current
text says "confirmed on Microsoft Learn" without noting that the Learn edition matrix excludes Home.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off the Widgets board machine-wide, clearing the news and weather feed off your
      taskbar.**

      ## What it does
      Sets the documented `AllowNewsAndInterests` policy to 0 under
      `HKLM\SOFTWARE\Policies\Microsoft\Dsh`. Microsoft states this covers the entire widgets
      experience including taskbar content. The WebExperience host package stays installed on disk.

      ## Benefits
      - **No taskbar feed**: no hover-open board and no MSN headlines one mouse-bump away
      - **Fewer background fetches**: the widgets host stops pulling content you are not reading
      - **Machine-wide**: one policy covers every account on the PC

      ## Drawbacks
      - **Widgets are gone entirely**: including the weather, calendar and any pinned widget you did
        want
      - **Not an uninstall**: the WebExperience host package still occupies disk
      - **Home is outside the matrix**: Microsoft documents this policy for Pro, Enterprise,
        Education and IoT Enterprise, not Home, though the widgets host generally honours it anyway

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the feature does not exist on Windows 10
      - **Takes effect**: after reboot, so the taskbar drops the entry point
      - **Reverting**: deletes the policy value, restoring the shipped default of allowed
      - Requires administrator rights because it writes a machine policy

      ## Recommendation
      Apply it if you never open the widgets board. Leave it alone if you actually glance at the
      feed, because there is no partial setting here.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - NewsAndInterests, AllowNewsAndInterests](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-newsandinterests)
      - [Manage connections from Windows components to Microsoft services, section 32 Widgets](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Policy CSP - NewsAndInterests, `AllowNewsAndInterests`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-newsandinterests (tier A)
2. Manage connections from Windows operating system components to Microsoft services, section 32 Widgets, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)

### `disable_edge_first_run` Edge first-run experience

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Edge`, value `HideFirstRunExperience`, type
`REG_DWORD`.

| Option | Value |
|---|---|
| Off | `HideFirstRunExperience` = 1 |
| On (Stock Default) | value absent |

*What it actually does.* `HideFirstRunExperience` is a current, non-deprecated Microsoft Edge policy.
It suppresses the first-run wizard, the import walkthrough and the sign-in and personalization
prompts on first launch of a profile. Data type is Boolean, which in the Windows registry means a
`REG_DWORD` of 0 or 1; Microsoft's own example registry value on the policy page is `0x00000001`. The
Windows registry location for machine-scoped Edge policies is `SOFTWARE\Policies\Microsoft\Edge`,
which is what the YAML writes. This tweak was attacked in the second verification round and survived
unchanged.

*Applicability.* Microsoft Edge 80 and later on Windows, so effectively every supported Windows 11
build. Not SKU gated. Requires administrator rights for the HKLM write, matching `elevation: admin`.
No reboot; it takes effect on the next Edge launch, and the absent `requires_reboot` is correct. On
Windows 10 IoT Enterprise LTSC, Edge is not included in the image, so the policy is inert unless Edge
was installed separately.

*Cautions.* None. The stock default of `absent` is correct; the policy is unset out of the box.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Edge opens straight to a normal window instead of a setup and import wizard.**

      ## What it does
      Sets the documented Edge policy `HideFirstRunExperience` to 1 under
      `HKLM\SOFTWARE\Policies\Microsoft\Edge`. That suppresses the first-run walkthrough, the
      "import your browsing data" prompt and the sign-in and personalization pages.

      ## Benefits
      - **No setup gauntlet**: useful on fresh installs and every new Windows profile
      - **No default-browser nag**: the first-run flow is where Edge pushes hardest
      - **Documented policy**: Microsoft publishes the value, type and behaviour

      ## Drawbacks
      - **No guided setup**: new users on a shared PC are not walked through Edge
      - **Edge stays installed**: this hides a wizard, it does not remove or disable the browser
      - **Nothing else changes**: sidebar, Copilot and background processes need their own tweaks

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, with Microsoft Edge 80 or later
      - **Takes effect**: immediately, on the next Edge launch
      - **Reverting**: deletes the policy value, restoring Edge's own default behaviour
      - Requires administrator rights because it writes a machine policy

      ## Recommendation
      Apply it, especially on new setups. There is no cost beyond losing a walkthrough that exists
      to change your defaults.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft Edge policy, HideFirstRunExperience](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hidefirstrunexperience)
```

**Sources:**
1. Microsoft Edge Browser Policy Documentation, `HideFirstRunExperience`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hidefirstrunexperience (tier A)

### `disable_edge_startup_boost` Edge startup boost

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Edge`, two values, both `REG_DWORD`.

| Effect | Value name | Off | Default (Stock Default) |
|---|---|---|---|
| `startup_boost` | `StartupBoostEnabled` | 0 | value absent |
| `background_mode` | `BackgroundModeEnabled` | 0 | value absent |

*What it actually does.* `StartupBoostEnabled` controls whether Edge pre-launches a set of processes
at sign-in so the first window appears faster. `BackgroundModeEnabled` controls whether Edge keeps
running after the last window closes. Both are current, non-deprecated Edge policies with Boolean
data type, so `REG_DWORD` 0 is correct for the "off" state. Both are Windows-only policies, which is
consistent with a Windows-only app writing them. This tweak was attacked in the second verification
round and survived unchanged.

*Applicability.* `StartupBoostEnabled` requires Edge 88 or later; `BackgroundModeEnabled` requires
Edge 77 or later. Both are satisfied by any Edge shipping on a supported Windows 11 build. Not SKU
gated. Administrator rights required for HKLM, matching `elevation: admin`. Takes effect at the next
sign-in or Edge restart, no reboot required, and the absent `requires_reboot` is acceptable.

*Cautions.* None. The stock default of `absent` for both is correct; neither policy is set out of the
box, and Edge's own in-product defaults have both features on.

**Corrections needed:** none.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Edge preloading at sign-in and lingering after you close it, so closing it actually
      closes it.**

      ## What it does
      Sets the documented Edge policies `StartupBoostEnabled` and `BackgroundModeEnabled` to 0 under
      `HKLM\SOFTWARE\Policies\Microsoft\Edge`. The first stops Edge pre-launching processes at
      sign-in; the second stops it staying resident after the last window closes.

      ## Benefits
      - **No idle Edge processes**: memory is not held while the browser is closed
      - **Quieter sign-in**: nothing pre-warms a browser you may not open
      - **Both halves documented**: Microsoft publishes each policy with its data type

      ## Drawbacks
      - **Slower first launch**: Edge's cold start each session is measurably slower without
        pre-warming
      - **Background extensions stop**: extensions that rely on background mode no longer run with
        Edge closed
      - **Edge only**: other Chromium browsers have their own equivalent settings

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, with Microsoft Edge 88 or later
      - **Takes effect**: immediately, at the next sign-in or Edge restart
      - **Reverting**: deletes both policy values, restoring Edge's own defaults, which have both on
      - Requires administrator rights because it writes machine policies

      ## Recommendation
      Apply it if Edge is not your main browser, or if you simply want nothing resident when the
      browser is closed. Skip it if you use Edge all day and value the faster cold start.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft Edge policy, StartupBoostEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/startupboostenabled)
      - [Microsoft Edge policy, BackgroundModeEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/backgroundmodeenabled)
```

**Sources:**
1. Microsoft Edge Browser Policy Documentation, `StartupBoostEnabled`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/startupboostenabled (tier A)
2. Microsoft Edge Browser Policy Documentation, `BackgroundModeEnabled`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/backgroundmodeenabled (tier A)

### `disable_edge_sidebar` Edge sidebar and Discover

**Verdict:** VERIFIED-WITH-CORRECTION (both policies are live and correctly written; the Copilot half
of the promise is dead as of Edge 141)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Edge`, two values, both `REG_DWORD`.

| Effect | Value name | Off | Default (Stock Default) |
|---|---|---|---|
| `sidebar` | `HubsSidebarEnabled` | 0 | value absent |
| `collections` | `EdgeCollectionsEnabled` | 0 | value absent |

The registry mechanism is correct as authored: both value names, the
`SOFTWARE\Policies\Microsoft\Edge` path, the `REG_DWORD` type, the 0 semantics and the `absent` stock
defaults all check out, and neither policy is deprecated. **What is corrected is scope, not values.**

*The Copilot claim no longer holds.* Microsoft's current policy page for `HubsSidebarEnabled` states:
"As of Microsoft Edge version 141, the `Microsoft365CopilotChatIconEnabled` policy is the only means
of controlling the display of Copilot in the toolbar." Edge is evergreen and auto-updating, and the
Edge installed on the 26100.4061 test machine is 150.0.4078.99, well past that cutover. So on any
current Edge, setting `HubsSidebarEnabled` = 0 hides the sidebar panel but **leaves the Copilot
button sitting in the toolbar**. The YAML promises the opposite in four places: the `name` ("Edge
sidebar and Discover"), the summary ("Hides the Edge sidebar and its Copilot/Discover button"), the
"What it does" phrase ("along with its Copilot and Discover button") and the "What you gain" line
("no always-present Copilot/Discover button"). A user applying this on Edge 141 or newer still sees
the Copilot button and will reasonably conclude the tweak did not work.

The same page notes that the *recommended* variant of `HubsSidebarEnabled` (the `Edge\Recommended`
path) "is obsolete. This policy has never supported the recommended capability." That does not affect
this tweak, which correctly writes only the mandatory `SOFTWARE\Policies\Microsoft\Edge` path.

*Collections is a bigger commitment than the name suggests.* `EdgeCollectionsEnabled` = 0 disables
the Collections feature entirely, not just its sidebar placement, and existing collections become
inaccessible from the UI while the policy is set. Collections stores user data, so bundling it behind
a switch named for the sidebar is a trap.

*Applicability.* `HubsSidebarEnabled` requires Edge 99 or later (Windows and macOS);
`EdgeCollectionsEnabled` requires Edge 78 or later. Both are satisfied by any Edge shipping on a
supported Windows 11 build. Not SKU gated. Administrator rights required, matching `elevation:
admin`. Takes effect on the next Edge restart, no reboot needed.

*Cautions.* Because Edge updates itself independently of Windows, the Copilot gap applies to
essentially every machine that has let Edge update, not to some future version.

**Corrections needed:** Two text corrections, no registry change. (1) Remove the Copilot promise from
the `name`, the summary, "What it does" and "What you gain". Restate the effect as hiding the sidebar
panel only. If the Copilot button is genuinely meant to be in scope, add
`Microsoft365CopilotChatIconEnabled` as a companion effect rather than leaving the claim unbacked.
(2) Split `EdgeCollectionsEnabled` into its own tweak or its own option, so hiding a panel does not
silently take a data-storing feature with it.

**Ready-to-paste info block:**

```yaml
    info: |
      **Hides the Edge sidebar panel, reclaiming browser width.**

      ## What it does
      Sets the documented Edge policies `HubsSidebarEnabled` and `EdgeCollectionsEnabled` to 0 under
      `HKLM\SOFTWARE\Policies\Microsoft\Edge`. The first removes the right-edge hubs panel and its
      toggle; the second disables Collections outright.

      ## Benefits
      - **More page width**: the right-edge panel and its rail stop taking space
      - **Less clutter**: no Discover pane inviting you into Bing content
      - **Documented policies**: both are current, non-deprecated and published by Microsoft

      ## Drawbacks
      - **Collections is disabled**: existing collections become unreachable from the UI while the
        policy is set, which is a real data feature, not decoration
      - **The Copilot toolbar button stays**: from Edge 141 Microsoft moved that button to a
        separate policy, so this tweak no longer removes it
      - **Sidebar tools go too**: anything you used in the panel, such as the calculator or unit
        converter, goes with it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, with Microsoft Edge 99 or later
      - **Takes effect**: immediately, on the next Edge restart
      - **Reverting**: deletes both policy values, restoring Edge's own defaults
      - Removing the toolbar Copilot button needs `Microsoft365CopilotChatIconEnabled` instead

      ## Recommendation
      Apply it if the sidebar is noise to you and you do not use Collections. If you use Collections
      at all, leave it alone until the two are split, because you would lose saved content access.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft Edge policy, HubsSidebarEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hubssidebarenabled)
      - [Microsoft Edge policy, EdgeCollectionsEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/edgecollectionsenabled)
```

**Sources:**
1. Microsoft Edge Browser Policy Documentation, `HubsSidebarEnabled`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/hubssidebarenabled (tier A; supported Windows Edge 99 and later, and the Edge 141 Copilot toolbar statement)
2. Microsoft Edge Browser Policy Documentation, `EdgeCollectionsEnabled`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/edgecollectionsenabled (tier A; supported Edge 78 and later, no deprecation marker)
3. Direct inspection, Windows 11 24H2 build 26100.4061: installed Microsoft Edge version 150.0.4078.99, past the Edge 141 cutover (primary measurement)

### `disable_account_notifications` Microsoft account nags in Start

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`, value
`DisableAccountNotifications`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `DisableAccountNotifications` = 1 |
| On (Stock Default) | value absent |

Proposed tweak metadata: `risk_level: low`, `elevation: user`, `reversible: true`,
**`requires_reboot: false`**.

*What it actually does.* The shipped `AccountNotifications.admx` on build 26100 declares the policy
verbatim:

```xml
<policy name="DisableAccountNotifications" class="User"
        key="SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications"
        valueName="DisableAccountNotifications">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_20H1_NOSERVER" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

`class="User"` confirms HKCU. Key path, value name, type and polarity all match. The value name also
appears in `C:\Windows\System32\windowsudk.shellcommon.dll`, so shipped shell code looks it up.

The shipped 26100 ADML enumerates the notification classes it suppresses: "This policy allows you to
prevent Windows from displaying notifications to Microsoft account (MSA) and local users in Start
(user tile). Notifications include getting users to: reauthenticate; backup their device; manage
cloud storage quotas as well as manage their Microsoft 365 or XBOX subscription. [...] No reboots or
service restarts are required for this policy setting to take effect." Microsoft Learn's Start policy
settings page repeats the same list and maps it to
`./User/Vendor/MSFT/Policy/Config/Notifications/DisableAccountNotifications`, GPO path User
Configuration > Administrative Templates > Windows Components > Account Notifications.

*Applicability.* Windows 10 2004 and later, client only, user scope. Applies to 26100, 26200 and the
secondary LTSC 2021 target. No elevation needed. No reboot needed, per the ADML.

*Duplicate check.* Passes. `disable_scoobe_nag` writes `ScoobeSystemSettingEnabled` under
`UserProfileEngagement` and the ContentDeliveryManager tweaks cover subscribed-content slots. Neither
touches the user-tile notification channel. `DisableAccountNotifications` appears zero times elsewhere
in the corpus.

*Cautions.* There is a non-policy sibling, `HKCU\...\Explorer\Advanced\Start_AccountNotifications`,
confirmed present in `StartDocked.dll`. It is real, but it is the value the Settings toggle owns. If
the tweak owns it too, a user flipping that switch in Settings silently desynchronises the tweak from
its snapshot. Ship the policy value alone.

**Corrections needed:** Two authoring requirements, since this is a new tweak with no YAML yet.
(1) `requires_reboot` must be `false`; the ADML says so explicitly. (2) Do not add the
`Start_AccountNotifications` sibling as a second effect, for the drift reason above.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows nagging you in the Start menu user tile to back up, re-sign-in, or buy more
      storage.**

      ## What it does
      Sets the documented policy `DisableAccountNotifications` to 1 under
      `HKCU\SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`. Microsoft's own
      description lists what that suppresses: prompts to reauthenticate, to back up the device, to
      manage cloud storage quota, and to manage a Microsoft 365 or Xbox subscription.

      ## Benefits
      - **Clean user tile**: the Start account area stops carrying upsell badges
      - **No storage-quota nag**: the OneDrive "you are almost full" prompt in Start goes quiet
      - **Instant and per user**: no reboot, no elevation, no service restart

      ## Drawbacks
      - **You lose real reminders too**: a genuine "sign in again" prompt is suppressed along with
        the marketing
      - **Start only**: notifications from OneDrive, Microsoft 365 and Xbox in the Action Center are
        unaffected
      - **Per user**: another account on the machine needs it applied separately

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: immediately, no reboot or sign-out needed
      - **Reverting**: deletes the policy value so Windows returns to its shipped behaviour
      - Settings has a related switch, `Start_AccountNotifications`; this tweak deliberately leaves
        that one to you so the two do not fight

      ## Recommendation
      Apply it if you use a Microsoft account and are tired of the account tile selling you storage
      and subscriptions. Skip it if you rely on Windows telling you when your sign-in has expired.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Start menu policy settings, Disable Account Notifications](https://learn.microsoft.com/en-us/windows/configuration/start/policy-settings)
      - [Policy CSP - Start](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-start)
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\AccountNotifications.admx` and `en-US\AccountNotifications.adml` on build 26100: policy `DisableAccountNotifications`, `class="User"`, key `SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`, enabled = 1, disabled = 0, `supportedOn` Windows 10 2004, and the "no reboots or service restarts are required" statement (tier A, shipped ADMX)
2. Microsoft Learn, Start menu policy settings, Disable Account Notifications, https://learn.microsoft.com/en-us/windows/configuration/start/policy-settings (tier A)
3. String presence of `DisableAccountNotifications` in `C:\Windows\System32\windowsudk.shellcommon.dll` on build 26100 (tier A, product artifact, primary observation)
4. String presence of the sibling `Start_AccountNotifications` in `StartDocked.dll` on build 26100 (tier A, product artifact, primary observation)

### `disable_settings_account_ads` Account upsell cards in Settings

**Verdict:** VERIFIED (new in this revision; mechanism exact, applicability corrected)

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value
`DisableConsumerAccountStateContent`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `DisableConsumerAccountStateContent` = 1 |
| On (Stock Default) | value absent |

Proposed tweak metadata: `risk_level: low`, `elevation: admin`, `reversible: true`,
`requires_reboot: false`.

*What it actually does.* The shipped `CloudContent.admx` on 26100 declares the policy `class="Machine"`
at key `Software\Policies\Microsoft\Windows\CloudContent`, with `enabledValue` 1 and `disabledValue`
0. Key, value name, type and polarity are all verified correct. The value name also appears in
`System32\windowsudk.shellcommon.dll` and `System32\wbem\DMWmiBridgeProv.dll`, so shipped code reads
it. When enabled, Windows experiences that would otherwise show account-state content (the "finish
setting up your account", storage-upgrade and subscription cards inside Settings) present the default
fallback content instead.

*Polarity trap, do not "fix" this.* The Policy CSP page's allowed-values table reads "0 (Default)
Disabled. 1 Enabled." Read naively that says 1 **enables** the content, which would invert the tweak.
It does not: that table describes the **policy** state, not the feature state, and it contradicts the
same page's own description block ("If you enable this policy, Windows experiences [...] will instead
present the default fallback content"). The ADMX `enabledValue` of 1 is decisive. 1 = account-state
content off. A future editor "correcting" this against the CSP table would break the tweak.

*Applicability, corrected.* Windows 11 21H2 (10.0.22000) and later, machine scope. Two Microsoft
sources disagree on the floor: the shipped 26100 `Windows.adml` renders the ADMX token
`SUPPORTED_Windows_10_0_RS7` as "At least Windows Server 2016, Windows 10 Version 1909", while the
Policy CSP page states "Windows 11, version 21H2 [10.0.22000] and later". The CSP is the narrower and
more current claim, so use Windows 11 21H2 and later. The practical consequence is that the Windows
10 IoT Enterprise LTSC 2021 secondary target is not supported for this policy.

*Edition gate, and it is decisive.* The Policy CSP edition matrix reads "**Pro NOT supported**", with
Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC supported, scope Device only. **This
tweak is a no-op on Pro and on Home**, which is the majority of consumer Windows 11 installs. It is
the same gate `privacy:disable_consumer_features` already lives with, whose backing
`AllowWindowsConsumerFeatures` carries the identical "Pro not supported" marking. The copy must say
so plainly rather than overselling the tweak.

*Duplicate check.* Passes. The corpus's three CloudContent tweaks write
`DisableTailoredExperiencesWithDiagnosticData`, `DisableSoftLanding` and
`DisableWindowsConsumerFeatures`. Different values, and the distinction between consumer features and
account-state cards is supported by the two policies having separate ADML descriptions and separate
client components.

**Corrections needed:** Three authoring requirements. (1) Set applicability to Windows 11 21H2
(22000) and later, not Windows 10 2004. (2) The copy must state that Pro and Home silently ignore
this policy; do not present it as effective everywhere. (3) Do not change the polarity to match the
CSP allowed-values table.

**Ready-to-paste info block:**

```yaml
    info: |
      **Replaces the account upsell cards inside Settings with plain default content, on the
      editions that honour it.**

      ## What it does
      Sets the documented policy `DisableConsumerAccountStateContent` to 1 under
      `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`. Windows experiences that would show
      account-state promotions (finish setting up your account, upgrade your storage, manage your
      subscription) present the default fallback content instead.

      ## Benefits
      - **Quieter Settings**: the account-state promo cards stop appearing
      - **Machine-wide**: one policy covers every account on the PC
      - **Documented**: shipped `CloudContent.admx` declares the key, value and polarity

      ## Drawbacks
      - **Ignored on Home and Pro**: Microsoft supports this policy only on Enterprise, Education
        and IoT Enterprise, so most consumer PCs will see no change at all
      - **Not all upsells**: Microsoft 365 and OneDrive prompts from their own apps are unaffected
      - **Requires admin**: it writes a machine policy, unlike the per-user suggestion tweaks

      ## Good to know
      - **Applies to**: Windows 11 21H2 and newer, Enterprise, Education, IoT Enterprise and IoT
        Enterprise LTSC only
      - **Takes effect**: immediately; sign out and back in if a card is still on screen
      - **Reverting**: deletes the policy value so Windows returns to its shipped behaviour
      - Check your edition first: on Home and Pro this applies cleanly and changes nothing

      ## Recommendation
      Apply it on Enterprise, Education or IoT Enterprise, where it does what it says. On Home and
      Pro it is honest to skip it, because Windows will not honour the policy.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Experience, DisableConsumerAccountStateContent](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` and `en-US\CloudContent.adml` on build 26100: policy `DisableConsumerAccountStateContent`, `class="Machine"`, key `Software\Policies\Microsoft\Windows\CloudContent`, `enabledValue` 1, `disabledValue` 0 (tier A, shipped ADMX)
2. Policy CSP - Experience, `DisableConsumerAccountStateContent`, applicable OS "Windows 11, version 21H2 [10.0.22000] and later" and the edition matrix marking Pro NOT supported, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience (tier A)
3. Shipped `en-US\Windows.adml` `supportedOn` string table on build 26100: `SUPPORTED_Windows_10_0_RS7` renders as "At least Windows Server 2016, Windows 10 Version 1909" (tier A, shipped ADMX)
4. String presence of `DisableConsumerAccountStateContent` in `System32\windowsudk.shellcommon.dll` and `System32\wbem\DMWmiBridgeProv.dll` on build 26100 (tier A, product artifact, primary observation)

### `disable_windows_spotlight_all` All Windows Spotlight features

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value
`DisableWindowsSpotlightFeatures`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `DisableWindowsSpotlightFeatures` = 1 |
| On (Stock Default) | value absent |

Proposed tweak metadata: `risk_level: low`, `elevation: user`, `reversible: true`,
`requires_reboot: false`.

*What it actually does.* The shipped `CloudContent.admx` on 26100 declares the policy `class="User"`,
`key="Software\Policies\Microsoft\Windows\CloudContent"`,
`valueName="DisableWindowsSpotlightFeatures"`, `enabledValue` 1 / `disabledValue` 0,
`supportedOn="windows:SUPPORTED_Windows_10_0_NOSERVER"`. Every element matches the mechanism above
exactly. The shipped `en-US\CloudContent.adml`, under the title "Turn off all Windows spotlight
features", states: "If you enable this policy setting, Windows spotlight on lock screen, Windows
tips, Microsoft consumer features and other related features will be turned off. You should enable
this policy setting if your goal is to minimize network traffic from target devices."

Binary evidence on 26100 puts the value name in `ContentDeliveryManager.Background.dll`,
`StartTileData.dll`, `Taskbar.dll`, `SettingsHandlers_ContentDeliveryManager.dll`,
`CustomShellHost.exe` and `ShellAppRuntime.exe`. Those are feature consumers, not policy plumbing, so
the value is genuinely read on the target build. privacy.sexy writes the same value name, confirmed
by direct fetch of its `windows.yaml`.

*Applicability.* Windows 10 and all Windows 11 builds, user scope, all editions (the policy is
`class="User"` and is not edition gated the way the machine-scoped CloudContent policies are). No
elevation. No reboot; sign out and back in for the lock screen and tip surfaces to refresh.

*Blast radius, which the copy must disclose.* This is a master switch that overlaps four shipped
tweaks: `RotatingLockScreenOverlayEnabled`, `SubscribedContent-338387Enabled`, `SoftLandingEnabled` /
`SubscribedContent-338389Enabled`, and `DisableWindowsConsumerFeatures`. It does not clobber their
values, so revert stays correct, but a user who applies this and then looks at those tweaks will see
them still reading "not applied" while their surfaces are gone. Say so, or the corpus looks
inconsistent.

*Duplicate check.* Passes. The corpus's other CloudContent values are
`DisableTailoredExperiencesWithDiagnosticData`, `DisableSoftLanding` and
`DisableWindowsConsumerFeatures` (all HKLM, all different names).

**Corrections needed:** One authoring requirement: the copy must disclose the overlap described
above, and the tweak should not be shipped as *recommended* alongside `disable_spotlight_desktop`
(pick one).

**Ready-to-paste info block:**

```yaml
    info: |
      **One switch that turns off every Windows Spotlight surface at once.**

      ## What it does
      Sets the documented policy `DisableWindowsSpotlightFeatures` to 1 under
      `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`. Microsoft's own description covers
      Spotlight on the lock screen, Windows tips, Microsoft consumer features and related surfaces,
      and names reducing network traffic as a reason to enable it.

      ## Benefits
      - **Everything at once**: lock screen images, tips and consumer content stop in one write
      - **Less network chatter**: Microsoft names traffic reduction as the point of this policy
      - **Works on Home**: this is the user-scoped Spotlight policy, not the edition-gated
        machine one

      ## Drawbacks
      - **No Spotlight wallpapers**: the rotating lock screen and desktop images stop, and the lock
        screen falls back to a static picture
      - **Broad**: it also covers surfaces some people like, such as Windows tips
      - **Makes other tweaks look unapplied**: the individual lock screen, soft landing and consumer
        content tweaks will still read "not applied" even though their surfaces are gone

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: after sign-out, so the lock screen and tip surfaces refresh
      - **Reverting**: deletes the policy value, and the individual tweaks resume controlling their
        own surfaces
      - Choose this or the desktop-only Spotlight tweak, not both; this one already includes it

      ## Recommendation
      Apply it if you want the whole Spotlight system off and do not care about the wallpapers. If
      you like the lock screen images and only want the ads gone, use the individual tweaks instead.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Experience, AllowWindowsSpotlight](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
      - [Manage connections from Windows components to Microsoft services](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` on build 26100: policy `DisableWindowsSpotlightFeatures`, `class="User"`, key `Software\Policies\Microsoft\Windows\CloudContent`, `enabledValue` 1, `disabledValue` 0, `supportedOn` `SUPPORTED_Windows_10_0_NOSERVER` (tier A, shipped ADMX)
2. Shipped `en-US\CloudContent.adml` on build 26100, "Turn off all Windows spotlight features" and its explain text (tier A, shipped ADMX)
3. String presence on build 26100 in `ContentDeliveryManager.Background.dll`, `StartTileData.dll`, `Taskbar.dll`, `SettingsHandlers_ContentDeliveryManager.dll`, `CustomShellHost.exe` and `ShellAppRuntime.exe` (tier A, product artifact, primary observation)
4. privacy.sexy `windows.yaml`, which writes the same value name, https://github.com/undergroundwires/privacy.sexy (tier C)

### `disable_spotlight_desktop` Spotlight desktop wallpaper

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`, value
`DisableSpotlightCollectionOnDesktop`, type `REG_DWORD`.

| Option | Value |
|---|---|
| Off | `DisableSpotlightCollectionOnDesktop` = 1 |
| On (Stock Default) | value absent |

Proposed tweak metadata: `risk_level: low`, `elevation: user`, `reversible: true`,
`requires_reboot: false`.

*What it actually does.* The shipped `CloudContent.admx` on 26100 declares the policy `class="User"`
at the same CloudContent key, with `enabledValue` 1 and
`supportedOn="windows:SUPPORTED_Windows_10_0_NOSERVER"`. The ADML text matches, including Microsoft's
own typo "subsequentyly". The value name is present in `SettingsHandlers_ContentDeliveryManager.dll`,
`StartTileData.dll`, `SettingsHandlers_nt.dll` and `CustomShellHost.exe` on 26100, so it is read by
feature code. Win11Debloat's `Regfiles/Disable_Desktop_Spotlight.reg`, fetched and decoded, is
byte-for-byte the same effect:
`[HKEY_CURRENT_USER\Software\Policies\Microsoft\Windows\CloudContent]`
`"DisableSpotlightCollectionOnDesktop"=dword:00000001`.

*Relationship to `disable_windows_spotlight_all`.* This is a **strict subset**. Desktop Spotlight
falls inside the "and other related features" that the master switch turns off, so applying
`disable_windows_spotlight_all` already covers it and applying both is redundant but harmless (they
are separate value names, so each snapshot and revert stays correct). Ship at most one of the two as
a *recommended* item; the other is fine as an available control. Choose this one if you want the
rotating desktop wallpaper gone but want to keep lock screen Spotlight and Windows tips.

*Applicability.* Windows 10 and all Windows 11 builds, user scope, all editions. No elevation. Sign
out and back in for the desktop wallpaper provider to fall back.

*Source correction.* The original proposal cited privacy.sexy for this value. privacy.sexy does not
contain this value name. Actual support is the shipped ADMX plus Win11Debloat, which is sufficient
because the ADMX is tier A.

**Corrections needed:** One authoring requirement: the copy must state the relationship to
`disable_windows_spotlight_all` so users do not apply both expecting different outcomes.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the rotating Spotlight image collection from taking over your desktop wallpaper.**

      ## What it does
      Sets the documented policy `DisableSpotlightCollectionOnDesktop` to 1 under
      `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`. That removes "Windows spotlight" as a
      desktop background option and returns the desktop to whatever wallpaper you set.

      ## Benefits
      - **Your wallpaper stays put**: no daily image swap chosen by Microsoft
      - **No desktop info bubble**: the Spotlight "learn about this picture" icon goes with it
      - **Narrow**: lock screen Spotlight and Windows tips are untouched

      ## Drawbacks
      - **No rotating images**: if you enjoy the daily photo, this removes it
      - **Redundant with the master switch**: the "all Spotlight features" tweak already covers this
        surface, so applying both changes nothing extra
      - **Per user**: another account on the machine needs it applied separately

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: after sign-out, when the wallpaper provider falls back
      - **Reverting**: deletes the policy value so Spotlight becomes selectable again
      - Pick this or the all-Spotlight tweak; this one is the narrower of the two

      ## Recommendation
      Apply it if you want to keep lock screen Spotlight but want control of your own desktop. If
      you want the whole Spotlight system gone, use the all-Spotlight tweak instead of this one.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Experience](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience)
      - [Win11Debloat, Disable_Desktop_Spotlight.reg](https://github.com/Raphire/Win11Debloat)
```

**Sources:**
1. Shipped `C:\Windows\PolicyDefinitions\CloudContent.admx` and `en-US\CloudContent.adml` on build 26100: policy `DisableSpotlightCollectionOnDesktop`, `class="User"`, key `Software\Policies\Microsoft\Windows\CloudContent`, `enabledValue` 1, `supportedOn` `SUPPORTED_Windows_10_0_NOSERVER` (tier A, shipped ADMX)
2. String presence on build 26100 in `SettingsHandlers_ContentDeliveryManager.dll`, `StartTileData.dll`, `SettingsHandlers_nt.dll` and `CustomShellHost.exe` (tier A, product artifact, primary observation)
3. Win11Debloat (Raphire), `Regfiles/Disable_Desktop_Spotlight.reg`, fetched and decoded, byte-for-byte the same effect, https://github.com/Raphire/Win11Debloat (tier C)

### `disable_nag_toasts` Suggested and backup reminder toasts

**Verdict:** VERIFIED (new in this revision; community-corroborated, not Microsoft-documented)

**Mechanism:** two per-app notification keys under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings`, each with the value
`Enabled`, type `REG_DWORD`.

| Effect | Key | Value name | Type |
|---|---|---|---|
| `suggested` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.Suggested` | `Enabled` | `REG_DWORD` |
| `backup` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.BackupReminder` | `Enabled` | `REG_DWORD` |

| Option | Values |
|---|---|
| Silenced | both `Enabled` = 0 |
| On (Stock Default) | both values absent |

Proposed tweak metadata: `risk_level: low`, `elevation: user`, `reversible: true`,
`requires_reboot: false`.

*What it actually does.* The `Notifications\Settings\<AppId>\Enabled` scheme is the per-app store the
Settings notifications page writes, so this is the same mechanism the shipped UI uses, applied to two
system toast identifiers rather than to notifications as a whole. Both identifiers exist in shipped
26100 binaries: `Windows.SystemToast.Suggested` in `ContentDeliveryManager.Utilities.dll`,
`NotificationController.dll` and `SmartActionPlatform.dll`; `Windows.SystemToast.BackupReminder` in
`shell32.dll`. Neither is a stale identifier. Win11Debloat's
`Regfiles/Disable_Windows_Suggestions.reg`, fetched and decoded, writes both with the inline comments
"Disable 'Suggested' app notifications (Ads for MS services)" and "Disable Windows Backup reminder
notifications".

*Implementation requirement, not optional.* The per-app keys are **created lazily on the first
toast**, so on a fresh profile neither key exists. The apply must create the key before writing
`Enabled`, and the status probe must treat "key absent" as the stock state rather than as an error.
Without that, the did-it-work contract produces false failures on any profile that has not yet seen
one of these toasts.

*Applicability.* Windows 10 and all Windows 11 builds, user scope, all editions. No elevation, no
reboot; the notification platform reads the per-app store on the next toast.

*Positioning.* This is the surgical alternative to `interface:disable_toast_notifications`, which
sets `ToastEnabled` = 0 and silences everything. Present them as alternatives, not as companions.

*Sourcing, corrected.* The original proposal cited Sophia Script for a
`Windows.ActionCenter.SmartOptOut` sibling; a raw grep of the Windows 11 `Sophia.psm1` returns no
`SystemToast` match at all. It also cited privacy.sexy, which does have per-app toast suppression but
uses a different mechanism (`HKLM\SOFTWARE\Classes\AppUserModelId\<id>` plus
`PushNotifications\Applications\<id>`) and only for `Windows.SystemToast.SecurityAndMaintenance` and
`Windows.SystemToast.SecurityCenter`. Neither corroborates these two identifiers. Real support is
Win11Debloat plus shipped-binary presence, which is enough for the identifiers but makes this
community-corroborated rather than Microsoft-documented.

**Corrections needed:** Two authoring requirements. (1) The apply must create the two lazily-created
keys, and the probe must treat key-absent as the stock state. (2) The `info` block must carry
Confidence `Community-corroborated`, not Microsoft-documented.

**Ready-to-paste info block:**

```yaml
    info: |
      **Silences the "Suggested" ad toasts and the Windows Backup reminder without muting every
      notification.**

      ## What it does
      Writes `Enabled` = 0 under
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings` for two system toast
      identifiers, `Windows.SystemToast.Suggested` and `Windows.SystemToast.BackupReminder`. That is
      the same per-app store the Settings notifications page uses.

      ## Benefits
      - **Surgical**: only these two nag channels stop, everything else still notifies you
      - **Both real**: both toast identifiers exist in shipped Windows 11 24H2 binaries
      - **Instant and per user**: no elevation, no reboot

      ## Drawbacks
      - **No backup reminders**: if you rely on Windows Backup prompting you, that prompt is gone
      - **Not a master mute**: other Microsoft promo toasts use different identifiers and keep coming
      - **Undocumented**: Microsoft publishes no page for these identifiers, though the storage
        scheme is the one Settings itself writes

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, all editions
      - **Takes effect**: immediately, from the next toast onward
      - **Reverting**: deletes both values so the notification platform returns to its default
      - The keys do not exist on a fresh profile; applying creates them

      ## Recommendation
      Apply it if you want notifications generally but are tired of being sold Microsoft services in
      the Action Center. If you want silence overall, use the toast notifications tweak in Interface
      instead of this one.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Win11Debloat, Disable_Windows_Suggestions.reg](https://github.com/Raphire/Win11Debloat)
      - [Change notification settings in Windows](https://support.microsoft.com/en-us/windows/change-notification-settings-in-windows-8942c744-6198-fe56-4639-34320cf9444e)
```

**Sources:**
1. Win11Debloat (Raphire), `Regfiles/Disable_Windows_Suggestions.reg`, fetched and decoded from UTF-16, writing `Enabled` = 0 under both toast identifiers with the comments "Disable 'Suggested' app notifications (Ads for MS services)" and "Disable Windows Backup reminder notifications", https://github.com/Raphire/Win11Debloat (tier C)
2. String presence on build 26100: `Windows.SystemToast.Suggested` in `ContentDeliveryManager.Utilities.dll`, `NotificationController.dll` and `SmartActionPlatform.dll`; `Windows.SystemToast.BackupReminder` in `shell32.dll` (tier A, product artifact, primary observation)
3. Raw grep of the Windows 11 `Sophia.psm1`: no `SystemToast` match, refuting the proposal's Sophia Script citation (primary observation)
4. privacy.sexy `windows.yaml`: per-app toast suppression exists but uses `HKLM\SOFTWARE\Classes\AppUserModelId\<id>` plus `PushNotifications\Applications\<id>` and only for two different identifiers, https://github.com/undergroundwires/privacy.sexy (tier C)

### `remove_teams_consumer_app` Consumer Teams app

**Verdict:** INCORRECT

**Mechanism (CORRECTED).** There is no correct form of this tweak as authored. On Windows 11 24H2 and
25H2 the preinstalled Teams is a **single unified package**:

- Package identity: `MSTeams`, package family `MSTeams_8wekyb3d8bbwe`
- Store product ID: `XP8BT8DW290MPQ`, title "Microsoft Teams"
- Policy id in Microsoft's policy-based inbox app removal list: `MSTeams`

That package serves personal, work and school Teams from one identity. Removing it is therefore **not
a consumer-only action**, and any retarget must say so in the loudest possible terms. A corrected
apply and probe would also have to be fail-closed:

```powershell
# apply
$ErrorActionPreference = 'Stop'
Get-AppxPackage -AllUsers -Name 'MSTeams' | Remove-AppxPackage -AllUsers
# probe
$ErrorActionPreference = 'Stop'
try { $pkg = Get-AppxPackage -AllUsers -Name 'MSTeams' } catch { exit 1 }
if ($pkg) { exit 1 } else { exit 0 }
```

**Wrong mechanism as authored, for reference.** State marker
`HKCU\Software\MagicXToolbox\Debloat`, value `TeamsConsumer`, `REG_DWORD`, plus a PowerShell action:

- Apply: `Get-AppxPackage -AllUsers 'MicrosoftTeams' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9NZTWSQNTK1S -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'MicrosoftTeams') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- `windows: { products: [11] }`

*Two independent failures.* First, the Store product ID is dead: the Store catalog service returns
HTTP 404 for `9NZTWSQNTK1S`, and `winget show --id 9NZTWSQNTK1S --exact` finds no package in any
source on winget 1.29.280. The undo can never succeed. Second, the target package is the wrong one
for the entire supported range. `MicrosoftTeams` was the consumer chat package that backed the
Windows 11 21H2 and 22H2 Chat flyout; from 23H2 it was replaced by `MSTeams`. **On 24H2 and 25H2 the
apply matches nothing.**

*The probe therefore reports a state that was never achieved.* Because `MicrosoftTeams` is not
installed on 24H2, `Get-AppxPackage` returns nothing, the `else` branch runs, the probe exits 0, and
the engine reads "Removed" on a machine where nothing was done and where the real Teams app is still
installed. This is the fail-open probe class at its most visible.

*Applicability.* None on the target platform. Effective only on Windows 11 21H2 and 22H2, both out of
scope, and only for the apply half; there is no working revert on any build.

*Cautions.* A naive fix that just swaps the package name would start removing the client that
organizations deploy, because `MSTeams` is one identity for both audiences. The tweak's existing
claim that work Teams is a different package is true of `MicrosoftTeams` and false of `MSTeams`.

**Corrections needed:** Four items. (1) Replace `9NZTWSQNTK1S`, which returns HTTP 404, with
`XP8BT8DW290MPQ` or remove the undo entirely. (2) Retarget the package from `MicrosoftTeams` to
`MSTeams`, or delete the tweak. (3) If retargeted, rename it: it is no longer a consumer-only
removal, and the copy must warn that work and school Teams share the identity. (4) Make the probe
fail-closed per correction 4 in the corrections list, so a query failure cannot read as "Removed".

**Ready-to-paste info block** (written for the corrected `MSTeams` target; do not ship this text
against the current `MicrosoftTeams` effect):

```yaml
    info: |
      **Uninstalls the preinstalled Microsoft Teams app, which on current Windows is one app for
      personal, work and school.**

      ## What it does
      Removes the `MSTeams` Appx package for all users. Since Windows 11 23H2 Microsoft ships a
      single unified Teams package (`MSTeams_8wekyb3d8bbwe`) instead of the old separate consumer
      chat build, so this is not a personal-only removal.

      ## Benefits
      - **One less preinstalled app**: Teams stops occupying disk and the app list
      - **No sign-in prompts**: the app cannot ask you to connect a personal account
      - **Reinstallable**: it is a normal Store app, so you can put it back

      ## Drawbacks
      - **Work and school Teams goes too**: the unified package is the same one organizations
        deploy, so removing it removes your work client
      - **Meeting links break**: joining a Teams meeting falls back to the browser
      - **Feature updates can bring it back**: a Windows feature update may re-provision the package,
        in which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Teams from the Microsoft Store, which needs internet access
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it only on a personal machine where nobody uses Teams for work. If the PC is used for
      any work or school account, leave it, because there is no longer a separate consumer package
      to target.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [What's new in Windows 11 version 23H2, Teams Chat removal](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2)
```

**Sources:**
1. Microsoft Store catalog service, product `9NZTWSQNTK1S` returns HTTP 404, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NZTWSQNTK1S?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Microsoft Store catalog service, product `XP8BT8DW290MPQ` returns title "Microsoft Teams", package family `MSTeams_8wekyb3d8bbwe` (tier A, primary observation)
3. `winget show --id 9NZTWSQNTK1S --exact` on winget 1.29.280 returns "No package found matching input criteria" (primary observation)
4. Policy-based inbox app removal supported app list, which names `MSTeams` and not `MicrosoftTeams`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
5. What's new in Windows 11 version 23H2, "Chat is being removed from the Microsoft Teams in-box app", https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2 (tier A)

### `remove_clipchamp` Clipchamp

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `Clipchamp`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `Clipchamp.Clipchamp`, package family `Clipchamp.Clipchamp_yxz26nhyzhsrt`
- Store product ID: `9P1J8S7CCWWT`, title "Microsoft Clipchamp"
- Apply: `Get-AppxPackage -AllUsers 'Clipchamp.Clipchamp' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9P1J8S7CCWWT -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Clipchamp.Clipchamp') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- `windows: { products: [11] }`, `elevation: admin`, `risk_level: low`

*What it actually does.* Both identities check out. The Store catalog resolves `9P1J8S7CCWWT` to
"Microsoft Clipchamp" with package family `Clipchamp.Clipchamp_yxz26nhyzhsrt`, and
`winget show --id 9P1J8S7CCWWT --source msstore` resolves with publisher "Microsoft Corp." and
publisher URL clipchamp.com. The non-Microsoft publisher hash in the package family name is a
consequence of Clipchamp having been an acquisition; it does not affect the removal. `Clipchamp` is
in Microsoft's policy-based inbox app removal list for 24H2 and 25H2, which is first-party
confirmation that the app is preinstalled and supported for removal on the target platform.

*Applicability.* Preinstalled on Windows 11 22H2 and later, so `products: [11]` is correct. Not a
system component. Administrator rights required for `-AllUsers`, matching `elevation: admin`. No
reboot. Inert on Windows 10 IoT Enterprise LTSC 2021, which ships no Store app set.

*Cautions.* A feature update can re-provision the package, which the tweak text already says.

**Corrections needed:** Three items, all from the shared app-removal defect classes. (1) The
provisioned package is not removed, so a newly created user profile still gets Clipchamp and a
feature update can restore it; add `Remove-AppxProvisionedPackage -Online` and extend the probe to
cover provisioned state. (2) The probe fails open: `Get-AppxPackage` errors are non-terminating, so
any failure, including running unelevated, lands in the `else` and reports "Removed". Rewrite it
fail-closed. (3) The undo depends on `winget`, which is absent on LTSC and on stripped images.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the bundled Clipchamp video editor.**

      ## What it does
      Removes the `Clipchamp.Clipchamp` Appx package for all users. Clipchamp is a consumer video
      editor Microsoft bundles with Windows 11; it is not an operating system component and nothing
      in Windows depends on it.

      ## Benefits
      - **Reclaims space**: Clipchamp is one of the larger bundled apps
      - **Tidier app list**: one fewer entry in Start and in Installed apps
      - **Safe to remove**: Microsoft lists it among the inbox apps supported for removal on 24H2

      ## Drawbacks
      - **No built-in video editor**: nothing else in Windows replaces it
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again
      - **Store needed to undo**: reinstalling requires the Microsoft Store or winget and internet

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Clipchamp from the Microsoft Store
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it if you have never opened it, which is most people. Keep it if you actually edit video
      on this machine, because there is no in-box alternative.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
```

**Sources:**
1. Microsoft Store catalog service, product `9P1J8S7CCWWT` returns title "Microsoft Clipchamp", package family `Clipchamp.Clipchamp_yxz26nhyzhsrt`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9P1J8S7CCWWT?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy-based inbox app removal supported app list, which includes `Clipchamp`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)
4. Remove-AppxPackage, `-AllUsers` parameter reference, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A)

### `remove_quick_assist` Quick Assist

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `QuickAssist`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `MicrosoftCorporationII.QuickAssist`, package family
  `MicrosoftCorporationII.QuickAssist_8wekyb3d8bbwe`
- Store product ID: `9P7BP5VNWKX5`, title "Quick Assist"
- Apply: `Get-AppxPackage -AllUsers 'MicrosoftCorporationII.QuickAssist' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9P7BP5VNWKX5 -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'MicrosoftCorporationII.QuickAssist') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, `risk_level: low`

*What it actually does.* Both identities confirmed. The Store catalog resolves `9P7BP5VNWKX5` to
"Quick Assist" with package family `MicrosoftCorporationII.QuickAssist_8wekyb3d8bbwe`, and
`winget show` resolves it in the msstore source with publisher "Microsoft Corp." and category
"Utilities & tools". The `MicrosoftCorporationII` publisher prefix is correct: this is the modern
Store-delivered Quick Assist that replaced the in-box Win32 app in Windows 10 2004 and later.
`QuickAssist` is in Microsoft's 24H2 and 25H2 policy-based inbox app removal list.

*Applicability.* Present on Windows 10 20H2 and later and all Windows 11 builds as a Store app, all
SKUs. Administrator rights required for `-AllUsers`. No reboot. On Windows 10 IoT Enterprise LTSC
2021 the tweak is inert, because that image ships the legacy Win32 Quick Assist, not the Appx
package.

*Cautions.* Removing it does not stop a determined attacker talking a user into installing it again
from the Store, so treat the security benefit as friction rather than as a control.

**Corrections needed:** Three items. (1) The provisioned package is not removed, so a new user
profile gets Quick Assist back and a feature update can restore it; add
`Remove-AppxProvisionedPackage -Online` and extend the probe. (2) The probe fails open on any
`Get-AppxPackage` error, including running unelevated; rewrite it fail-closed. (3) Consider gating to
`products: [11]` since the tweak cannot do anything on the LTSC secondary target.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls Quick Assist, the remote-help tool that tech-support scammers lean on.**

      ## What it does
      Removes the `MicrosoftCorporationII.QuickAssist` Appx package for all users. Quick Assist lets
      another person view or control your PC after you read them a code, which is exactly the flow
      support scams use.

      ## Benefits
      - **Removes a scam vector**: no in-box path for a caller to take control of your screen
      - **One less Store app**: it is not a system component and nothing depends on it
      - **Reinstallable**: available from the Microsoft Store if you ever need it

      ## Drawbacks
      - **No built-in remote help**: a family member who supports you remotely loses their easiest
        route in
      - **Friction, not a block**: someone can still be talked into reinstalling it from the Store
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the LTSC image ships a different, Win32 build that
        this does not touch
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Quick Assist from the Microsoft Store
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it on a personal machine that never receives remote assistance, especially one used by
      someone who might fall for a support call. Keep it if you or your helper actually use it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
```

**Sources:**
1. Microsoft Store catalog service, product `9P7BP5VNWKX5` returns title "Quick Assist", package family `MicrosoftCorporationII.QuickAssist_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9P7BP5VNWKX5?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy-based inbox app removal supported app list, which includes `QuickAssist`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)

### `remove_bing_news_weather` Bing News and Weather

**Verdict:** INCORRECT

**Mechanism (CORRECTED).** Two packages, queried separately. Both package names and both Store IDs in
the YAML are correct; only the PowerShell is broken.

| App | Appx package identity | Package family | Store product ID | Catalog title |
|---|---|---|---|---|
| News | `Microsoft.BingNews` | `Microsoft.BingNews_8wekyb3d8bbwe` | `9WZDNCRFHVFW` | Microsoft News |
| Weather | `Microsoft.BingWeather` | `Microsoft.BingWeather_8wekyb3d8bbwe` | `9WZDNCRFJ3Q2` | MSN Weather |

Corrected apply and probe, fail-closed and iterating rather than passing an array:

```powershell
# apply
$ErrorActionPreference = 'Stop'
'Microsoft.BingNews','Microsoft.BingWeather' | ForEach-Object {
    Get-AppxPackage -AllUsers -Name $_ | Remove-AppxPackage -AllUsers
}
# probe
$ErrorActionPreference = 'Stop'
try {
    $pkg = 'Microsoft.BingNews','Microsoft.BingWeather' |
        ForEach-Object { Get-AppxPackage -AllUsers -Name $_ }
} catch { exit 1 }
if ($pkg) { exit 1 } else { exit 0 }
```

Corrected undo, with the failure of the first install no longer masked:

```
winget install --id 9WZDNCRFHVFW -e --accept-source-agreements --accept-package-agreements && winget install --id 9WZDNCRFJ3Q2 -e --accept-source-agreements --accept-package-agreements
```

**Wrong mechanism as authored, for reference.** State marker `HKCU\Software\MagicXToolbox\Debloat`,
value `BingNewsWeather`, `REG_DWORD`, plus:

- Apply: `Get-AppxPackage -AllUsers 'Microsoft.BingNews','Microsoft.BingWeather' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9WZDNCRFHVFW -e ...; winget install --id 9WZDNCRFJ3Q2 -e ...`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.BingNews','Microsoft.BingWeather') { exit 1 } else { exit 0 }`

*Why it is broken.* `Get-AppxPackage` declares its `-Name` parameter as `System.String`, not
`System.String[]`. Supplying `'Microsoft.BingNews','Microsoft.BingWeather'` constructs an `Object[]`
and PowerShell cannot bind it, raising `CannotConvertArgument: Cannot convert 'System.Object[]' to
the type 'System.String' required by parameter 'Name'`. This was reproduced on Windows PowerShell
5.1.26100.4061, which is the shell the engine spawns (`powershell.exe -NoProfile -NonInteractive
-EncodedCommand`), and on PowerShell 7. The cmdlet emits an error and returns nothing, so the pipeline
into `Remove-AppxPackage` is empty and **nothing is uninstalled**. The error is non-terminating in
this position, so `powershell.exe` still exits 0 and the engine reads the apply as successful. The
probe has the same construction, so `if (Get-AppxPackage ...)` is false and it exits 0, which the
engine reads as "applied". **The tweak reports success and applied-state while having removed
nothing**, on every machine, whether or not either app is present.

*The undo masks a failure too.* `winget install A; winget install B` exits with B's code. If the News
install fails, the action still reports success.

*Applicability.* Both apps are in Microsoft's 24H2 and 25H2 policy-based inbox app removal list, so
they do ship preinstalled on the target platform and the tweak is worth fixing rather than deleting.
Both Store IDs resolve in `winget --source msstore`, so each undo half is individually executable.
All SKUs except LTSC, where neither app ships; Microsoft names News and Weather explicitly on the
LTSC excluded app list. Administrator rights required for `-AllUsers`. No reboot.

*Cautions.* Widgets can still show weather through the WebExperience host, so removing these apps
does not remove every weather surface. As written, this tweak is a silent no-op that reports success,
which is worse than an absent tweak because the user believes the apps are gone.

**Corrections needed:** Four items. (1) Split the apply and probe into two separate `Get-AppxPackage`
calls, or pipe the names through `ForEach-Object`, so the binder is not handed an array. (2) Replace
the `;` between the two `winget install` calls with `&&` or explicit exit-code checking. (3) Make the
probe fail-closed so a query error cannot report "Removed". (4) Add
`Remove-AppxProvisionedPackage -Online`, and note that both apps ship as bundles, so the
`Get-AppxPackage` side should use `-PackageTypeFilter Bundle`.

**Ready-to-paste info block** (written for the corrected mechanism):

```yaml
    info: |
      **Uninstalls the MSN News and Weather apps, cutting one MSN content and notification channel.**

      ## What it does
      Removes the `Microsoft.BingNews` and `Microsoft.BingWeather` Appx packages for all users. These
      are the standalone MSN apps; both are in Microsoft's supported inbox app removal list for
      Windows 11 24H2.

      ## Benefits
      - **Two content apps gone**: no MSN News or Weather in Start or in Installed apps
      - **Fewer notifications**: their toast and badge channels go with them
      - **Supported removal**: Microsoft lists both as removable inbox apps on 24H2

      ## Drawbacks
      - **Widgets still shows weather**: the widgets board pulls content through its own host, so
        disable Widgets separately if that is what you want gone
      - **No offline weather**: nothing in Windows replaces the standalone app
      - **Feature updates can restore them**: a Windows feature update may re-provision either
        package, in which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; neither app ships on Windows 10 LTSC
      - **Takes effect**: immediately, both apps are uninstalled on apply
      - **Reverting**: reinstalls both from the Microsoft Store, which needs internet access
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove them if you get news and weather anywhere else, which is most people. Keep them if you
      actually open the Weather app, because the widgets feed is not a replacement.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Remove-AppxPackage, bundle guidance](https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage)
```

**Sources:**
1. `Get-AppxPackage` parameter metadata: `(Get-Command Get-AppxPackage).Parameters['Name'].ParameterType.FullName` returns `System.String` on Windows PowerShell 5.1.26100.4061 and on PowerShell 7 (primary observation)
2. Reproduction of `CannotConvertArgument` on both shells (primary observation)
3. Microsoft Store catalog service, products `9WZDNCRFHVFW` ("Microsoft News", `Microsoft.BingNews_8wekyb3d8bbwe`) and `9WZDNCRFJ3Q2` ("MSN Weather", `Microsoft.BingWeather_8wekyb3d8bbwe`) (tier A, primary observation)
4. Policy-based inbox app removal supported app list, which includes both `BingNews` and `BingWeather`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
5. Remove-AppxPackage, `-AllUsers` parameter and bundle guidance, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A)
6. Windows as a service overview, LTSC excluded app list naming Weather and News, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A)

### `remove_solitaire` Solitaire Collection

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `Solitaire`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `Microsoft.MicrosoftSolitaireCollection`, package family
  `Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe`
- Store product ID: `9WZDNCRFHWD2`, title "Microsoft Solitaire Collection", categories "Card & board,
  Classics, Puzzle & trivia"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.MicrosoftSolitaireCollection' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9WZDNCRFHWD2 -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.MicrosoftSolitaireCollection') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, `risk_level: low`

*What it actually does.* The identity is correct and the removal works.
`MicrosoftSolitaireCollection` is in Microsoft's 24H2 and 25H2 policy-based inbox app removal list,
which confirms it is preinstalled and supported for removal on the target platform.

*Applicability.* Preinstalled on Windows 10 and on Windows 11 through 25H2, all SKUs except LTSC and
the Enterprise images where consumer experiences are suppressed. Administrator rights required for
`-AllUsers`. No reboot.

*Cautions, and the undo is the real one.* `winget show --id 9WZDNCRFHWD2 --exact` returns "No package
found matching input criteria" across all sources on winget 1.29.280, even though the catalog record
is intact and the app installs normally from the Store UI. The product is not surfaced by winget's
msstore search index, most likely because it is categorised as a game. The tweak's `reversible: true`
therefore overstates what will happen at the command line.

**Corrections needed:** Four items. (1) The undo will fail through winget: replace it with a Store
deep link (`ms-windows-store://pdp/?ProductId=9WZDNCRFHWD2`) or state clearly that reinstalling
requires the Store app. (2) Add provisioned-package removal. (3) This app ships as a bundle, so
`Remove-AppxPackage -AllUsers` should be paired with `-PackageTypeFilter Bundle` on the
`Get-AppxPackage` side per Microsoft's own guidance. (4) Make the probe fail-closed.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the ad-supported Microsoft Solitaire Collection.**

      ## What it does
      Removes the `Microsoft.MicrosoftSolitaireCollection` Appx package for all users. This is the
      bundled games app that shows video ads between hands and sells a subscription to remove them.

      ## Benefits
      - **Removes an ad-supported app**: no video ads or Premium upsell on the machine
      - **Reclaims space**: the collection is a large bundle
      - **Supported removal**: Microsoft lists it among the removable inbox apps on 24H2

      ## Drawbacks
      - **The games go**: Klondike, Spider and the rest are gone, with no in-box replacement
      - **Reinstall needs the Store**: this product is not resolvable through winget, so the revert
        may need the Store app rather than a command
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; not present on LTSC images
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls from the Microsoft Store; winget cannot resolve this product ID
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it unless you play these games. If you do play them, note that removal is easy to undo
      only through the Store, so decide before applying.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Remove-AppxPackage, bundle guidance](https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage)
```

**Sources:**
1. Microsoft Store catalog service, product `9WZDNCRFHWD2` returns title "Microsoft Solitaire Collection", package family `Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9WZDNCRFHWD2?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. `winget show --id 9WZDNCRFHWD2 --exact` returns no package on winget 1.29.280 (primary observation)
3. Policy-based inbox app removal supported app list, which includes `MicrosoftSolitaireCollection`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
4. Remove-AppxPackage, `-AllUsers` parameter and bundle guidance, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A)

### `remove_get_help` Get Help

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `GetHelp`, `REG_DWORD`, plus
a PowerShell action.

- Appx package identity: `Microsoft.GetHelp`, package family `Microsoft.GetHelp_8wekyb3d8bbwe`
- Store product ID: `9PKDZBMV1H3T`, title "Get Help", category "System Components"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.GetHelp' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9PKDZBMV1H3T -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.GetHelp') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, `risk_level: low`

*What it actually does.* Identity confirmed. The Store catalog resolves `9PKDZBMV1H3T` to "Get Help"
with package family `Microsoft.GetHelp_8wekyb3d8bbwe`, category "System Components", and
`winget show` resolves it in the msstore source with publisher URL support.microsoft.com, so the undo
is executable.

*Applicability, with an open question.* Preinstalled on Windows 10 1709 and later and on Windows 11
builds through at least 23H2, all consumer and business SKUs. **`Microsoft.GetHelp` is absent from
Microsoft's 24H2 and 25H2 policy-removable inbox app list**, which may mean it is no longer
preinstalled or may mean it is classified as a system component and therefore deliberately excluded
from that list; Microsoft's own troubleshooting section documents event IDs for package family names
that are "a system component". Absence from the list is strong evidence but not proof. This needs a
`Get-AppxPackage` check on a clean 24H2 image before the applicability line is finalised.
Administrator rights required. No reboot. Inert on LTSC.

*Cautions.* The Store category "System Components" is a signal that Windows expects the app to be
present. Removal is still supported.

**Corrections needed:** Three items. (1) The provisioned package is not removed; add
`Remove-AppxProvisionedPackage -Online` and extend the probe. (2) Make the probe fail-closed. (3)
Confirm presence on a stock 24H2 image and set the applicability line accordingly; if the app is not
preinstalled on 24H2 the tweak still has a real control to offer on machines that upgraded, but the
copy should say so.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the Get Help support app.**

      ## What it does
      Removes the `Microsoft.GetHelp` Appx package for all users. Get Help is Microsoft's in-app
      support and troubleshooting front end, and it is the target of the `ms-get-started:` links some
      Settings troubleshooters use.

      ## Benefits
      - **One less preinstalled app**: for people who never use Microsoft's guided support
      - **Removes a support-chat surface**: the app is also the entry point to Microsoft's virtual
        agent
      - **Reinstallable**: available from the Microsoft Store if you change your mind

      ## Drawbacks
      - **Troubleshooter links break**: some Settings flows deep-link into Get Help and those links
        fail rather than falling back
      - **Store lists it as a System Component**: Windows expects it to be present, even though
        removal is supported
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; presence on a stock 24H2 image is not guaranteed,
        so on some machines there may be nothing to remove
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Get Help from the Microsoft Store
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it if you troubleshoot Windows yourself or with a search engine. Keep it if you use the
      built-in guided troubleshooters, because those links stop working without it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
      - [Policy-based inbox app removal](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
```

**Sources:**
1. Microsoft Store catalog service, product `9PKDZBMV1H3T` returns title "Get Help", package family `Microsoft.GetHelp_8wekyb3d8bbwe`, category "System Components", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9PKDZBMV1H3T?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy-based inbox app removal supported app list, which does **not** include `GetHelp`, plus its troubleshooting section on system-component exclusions, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)

### `remove_getstarted_tips` Tips (Get Started)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `GetStarted`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `Microsoft.Getstarted` (note the lowercase "s"), package family
  `Microsoft.Getstarted_8wekyb3d8bbwe`
- Store product ID: `9WZDNCRDTBJJ`, title "Microsoft Tips", category "System Components"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.Getstarted' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9WZDNCRDTBJJ -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.Getstarted') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, `risk_level: low`

*What it actually does.* Identity confirmed, including the unusual lowercase "s" in
`Microsoft.Getstarted`. The Store catalog resolves `9WZDNCRDTBJJ` to "Microsoft Tips" with package
family `Microsoft.Getstarted_8wekyb3d8bbwe`. The app delivers onboarding tips and promotional
"getting started" content and is separate from the Windows tips surfaced through Spotlight, which
`disable_windows_spotlight_all` covers.

*Applicability, with an open question.* Preinstalled on Windows 10 and Windows 11 through at least
23H2, all consumer SKUs. Microsoft lists the app on its deprecated-features page ("The Tips app is
deprecated and will be removed in a future release of Windows") and it is absent from the 24H2 and
25H2 policy-removable inbox app list, but Microsoft has published no explicit "no longer
preinstalled" statement of the kind it published for Maps. So presence on a stock 24H2 image is
unconfirmed. Administrator rights required. No reboot. Inert on LTSC.

*Cautions.* The undo will fail at the command line. `winget show --id 9WZDNCRDTBJJ --exact` returns
"No package found matching input criteria" across all sources on winget 1.29.280, despite the catalog
record being present. Reinstallation requires the Store app.

**Corrections needed:** Three items. (1) The undo is not executable through winget: replace it with a
Store deep link (`ms-windows-store://pdp/?ProductId=9WZDNCRDTBJJ`) or document the limitation. (2)
Add provisioned-package removal. (3) Make the probe fail-closed.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the Tips app, which exists to show onboarding and promotional content.**

      ## What it does
      Removes the `Microsoft.Getstarted` Appx package for all users. The app, listed in the Store as
      "Microsoft Tips", delivers "getting started" walkthroughs and tip content; Microsoft has marked
      it deprecated.

      ## Benefits
      - **Removes a promotional app**: its whole purpose is onboarding and tip content
      - **Deprecated upstream**: Microsoft has said it will be removed from Windows in a future
        release
      - **No functional loss**: nothing in Windows depends on it

      ## Drawbacks
      - **No tip walkthroughs**: if you use the guided tips, they go
      - **Reinstall needs the Store**: this product is not resolvable through winget, so the revert
        may need the Store app rather than a command
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; presence on a stock 24H2 image is not guaranteed,
        so on some machines there may be nothing to remove
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls from the Microsoft Store; winget cannot resolve this product ID
      - Windows tips shown through Spotlight are a separate surface with their own tweak

      ## Recommendation
      Remove it. It is deprecated, purely promotional, and nothing depends on it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Deprecated features in the Windows client, Tips app](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features)
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
```

**Sources:**
1. Microsoft Store catalog service, product `9WZDNCRDTBJJ` returns title "Microsoft Tips", package family `Microsoft.Getstarted_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9WZDNCRDTBJJ?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. `winget show --id 9WZDNCRDTBJJ --exact` returns no package on winget 1.29.280 (primary observation)
3. Deprecated features in the Windows client, "The Tips app is deprecated and will be removed in a future release of Windows", https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
4. Policy-based inbox app removal supported app list, which does not include `Getstarted`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)

### `remove_feedback_hub` Feedback Hub

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `FeedbackHub`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `Microsoft.WindowsFeedbackHub`, package family
  `Microsoft.WindowsFeedbackHub_8wekyb3d8bbwe`
- Store product ID: `9NBLGGH4R32N`, title "Feedback Hub"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.WindowsFeedbackHub' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9NBLGGH4R32N -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.WindowsFeedbackHub') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, `risk_level: low`

*What it actually does.* Identity confirmed. The Store catalog resolves `9NBLGGH4R32N` to "Feedback
Hub" with package family `Microsoft.WindowsFeedbackHub_8wekyb3d8bbwe`, and `winget show` resolves it
in the msstore source, so the undo is executable. `WindowsFeedbackHub` is in Microsoft's 24H2 and
25H2 policy-based inbox app removal list.

*Applicability.* Preinstalled on Windows 10 and Windows 11, all SKUs except LTSC. Administrator
rights required for `-AllUsers`. No reboot.

*Cautions.* Windows Insider feedback and several diagnostic flows require Feedback Hub, and
Microsoft's own deprecated-features documentation directs users to it for feature feedback.

**Corrections needed:** Two items. (1) The provisioned package is not removed; add
`Remove-AppxProvisionedPackage -Online` and extend the probe. (2) Make the probe fail-closed.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls Feedback Hub, the app for sending feedback and diagnostic traces to Microsoft.**

      ## What it does
      Removes the `Microsoft.WindowsFeedbackHub` Appx package for all users. Feedback Hub is where
      you file bugs and suggestions with Microsoft, and it is wired into some diagnostic and trace
      collection flows.

      ## Benefits
      - **Removes an app most people never open**: it exists for people who report bugs to Microsoft
      - **One fewer diagnostic surface**: the app collects traces and attaches them to reports
      - **Supported removal**: Microsoft lists it among the removable inbox apps on 24H2

      ## Drawbacks
      - **Insiders need it**: Windows Insider feedback and several diagnostic flows require this app
      - **No in-box way to report bugs**: Microsoft's own deprecated-features page directs feature
        feedback here
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; not present on LTSC images
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Feedback Hub from the Microsoft Store
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it on a stable, non-Insider machine that never files feedback. Keep it if you run
      Insider builds, because the feedback flow depends on it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Deprecated features in the Windows client](https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features)
```

**Sources:**
1. Microsoft Store catalog service, product `9NBLGGH4R32N` returns title "Feedback Hub", package family `Microsoft.WindowsFeedbackHub_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH4R32N?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy-based inbox app removal supported app list, which includes `WindowsFeedbackHub`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
3. Deprecated features in the Windows client, which directs feature feedback to Feedback Hub, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
4. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)

### `remove_phone_link` Phone Link

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `PhoneLink`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `Microsoft.YourPhone`, package family `Microsoft.YourPhone_8wekyb3d8bbwe`
- Store product ID: `9NMPJ99VJBWV`, title "Phone Link"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.YourPhone' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9NMPJ99VJBWV -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.YourPhone') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, `risk_level: low`

*What it actually does.* Identity confirmed. The Store catalog resolves `9NMPJ99VJBWV` to "Phone
Link" with package family `Microsoft.YourPhone_8wekyb3d8bbwe`, and `winget show` resolves it in the
msstore source, so the undo is executable. `Microsoft.YourPhone` is the correct legacy identity: the
app was renamed from Your Phone to Phone Link without a package rename.

*Applicability, with an open question.* Preinstalled on Windows 10 1809 and later and on Windows 11
builds through at least 23H2, all consumer SKUs. **`Microsoft.YourPhone` is absent from Microsoft's
24H2 and 25H2 policy-removable inbox app list**, which may mean it is no longer preinstalled or may
mean it is classified as a system component and excluded from that list. Absence is strong evidence
but not proof, and this needs a `Get-AppxPackage` check on a clean 24H2 image. Administrator rights
required. No reboot. Inert on LTSC.

*Cautions.* Windows 11 24H2 and later surface phone integration in the Start menu side panel and in
Settings; removing the package disables those surfaces too, not only the standalone app. The package
is also among the more frequently re-provisioned by feature updates.

**Corrections needed:** Three items. (1) The provisioned package is not removed; add
`Remove-AppxProvisionedPackage -Online` and extend the probe. (2) Make the probe fail-closed. (3)
Mention the Start menu phone panel dependency on 24H2 and later in the tweak text.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls Phone Link, the companion app that mirrors an Android phone or iPhone on the PC.**

      ## What it does
      Removes the `Microsoft.YourPhone` Appx package for all users. Phone Link mirrors phone
      notifications, messages, calls and photos, and on Windows 11 24H2 it also backs the phone panel
      in the Start menu.

      ## Benefits
      - **Removes a background companion**: nothing pairing or polling if you never link a phone
      - **Clears the Start phone panel**: the 24H2 side panel goes with the app
      - **Reinstallable**: it is a normal Store app

      ## Drawbacks
      - **Phone integration stops**: notification mirroring, messaging, calls and photo transfer all
        go, on both Android and iPhone
      - **Start panel disappears**: the 24H2 Start phone panel is backed by this package
      - **Feature updates can restore it**: this package is re-provisioned unusually often, in which
        case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; presence on a stock 24H2 image is not guaranteed,
        so on some machines there may be nothing to remove
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Phone Link from the Microsoft Store
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it if you never connect a phone to this PC. Keep it if you use phone mirroring at all,
      because the Start panel and the app share the same package.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
      - [Policy-based inbox app removal](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
```

**Sources:**
1. Microsoft Store catalog service, product `9NMPJ99VJBWV` returns title "Phone Link", package family `Microsoft.YourPhone_8wekyb3d8bbwe`, https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NMPJ99VJBWV?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy-based inbox app removal supported app list, which does **not** include `YourPhone`, plus its troubleshooting section on system-component exclusions, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)

### `remove_outlook_new` New Outlook app

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (CORRECTED package identity):** state marker `HKCU\Software\MagicXToolbox\Debloat`, value
`OutlookNew`, `REG_DWORD`, plus a PowerShell action.

- Appx package identity: **`Microsoft.OutlookforWindows`** (lowercase "f"), package family
  `Microsoft.OutlookforWindows_8wekyb3d8bbwe`
- Store product ID: `9NRX63209R7B`, title "Outlook for Windows"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.OutlookforWindows' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9NRX63209R7B -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.OutlookforWindows') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- `windows: { products: [11] }`, `elevation: admin`, `risk_level: low`

**Wrong identity as authored, for reference.** The YAML writes `Microsoft.OutlookForWindows` with a
capital "F" in all three scripts. `Get-AppxPackage -Name` matching is case-insensitive, so the apply,
probe and undo all still work; this is a correctness-of-record defect rather than a functional one.

*What it actually does.* The Store catalog resolves `9NRX63209R7B` to "Outlook for Windows" with
package family `Microsoft.OutlookforWindows_8wekyb3d8bbwe`, and the listing confirms the app's role:
"This app will replace the Windows Mail, Calendar, and People apps beginning in 2024." `winget show`
resolves the ID in the msstore source, so the undo is executable. `OutlookForWindows` is in
Microsoft's 24H2 and 25H2 policy-based inbox app removal list. Classic Outlook from Microsoft 365 is
a separate Win32 installation and is unaffected.

*Applicability.* Preinstalled on Windows 11 22H2 and later, and also pushed to Windows 10 devices
through a Microsoft 365 and Windows update channel, so the `products: [11]` gate is arguably narrower
than reality. Administrator rights required. No reboot.

*Cautions.* Microsoft re-pushes this app aggressively. It has been delivered through Windows Update
as well as through Store provisioning, so removal is likely to be temporary without also removing the
provisioned package, and even then it can return.

**Corrections needed:** Four items. (1) Record the canonical identity as `Microsoft.OutlookforWindows`
with a lowercase "f". (2) Add `Remove-AppxProvisionedPackage -Online` and extend the probe. (3) Make
the probe fail-closed. (4) Consider dropping the `products: [11]` gate, since the app also lands on
Windows 10 22H2.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the preinstalled new Outlook for Windows, leaving classic Outlook untouched.**

      ## What it does
      Removes the `Microsoft.OutlookforWindows` Appx package for all users. This is the web-wrapper
      mail client Microsoft bundles with Windows 11 and positions as the replacement for the Mail,
      Calendar and People apps. Classic Outlook from Microsoft 365 is a separate Win32 install and is
      not affected.

      ## Benefits
      - **Removes a mail client you did not choose**: it is preinstalled, not selected
      - **Stops the migration nudges**: no prompts to move from classic Outlook or Mail
      - **Classic Outlook is untouched**: the Office desktop app is a separate installation

      ## Drawbacks
      - **Mail and Calendar have no in-box replacement**: Microsoft retired those apps in favour of
        this one, so removing it leaves no bundled mail client
      - **Microsoft re-pushes it hard**: it has arrived through Windows Update as well as Store
        provisioning, so a feature update can re-provision the package; apply the tweak again if so
      - **Removes it for every account**: other users on the PC lose it too

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Outlook for Windows from the Microsoft Store
      - Removing for all users needs administrator rights

      ## Recommendation
      Remove it if you use classic Outlook, a browser, or another mail client. Keep it if it is your
      mail app, because Windows no longer ships an alternative.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
```

**Sources:**
1. Microsoft Store catalog service, product `9NRX63209R7B` returns title "Outlook for Windows", package family `Microsoft.OutlookforWindows_8wekyb3d8bbwe`, and the description "This app will replace the Windows Mail, Calendar, and People apps beginning in 2024", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NRX63209R7B?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Policy-based inbox app removal supported app list, which includes `OutlookForWindows`, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)
3. Remove-AppxProvisionedPackage (Dism), https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)

### `remove_xbox_game_bar` Xbox Game Bar

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `XboxGameBar`, `REG_DWORD`,
plus a PowerShell action.

- Appx package identity: `Microsoft.XboxGamingOverlay`, package family
  `Microsoft.XboxGamingOverlay_8wekyb3d8bbwe`
- Store product ID: `9NZKPSTSNW4P`, title "Game Bar", category "System Components"
- Apply: `Get-AppxPackage -AllUsers 'Microsoft.XboxGamingOverlay' | Remove-AppxPackage -AllUsers`
- Undo: `winget install --id 9NZKPSTSNW4P -e --accept-source-agreements --accept-package-agreements`
- Probe: `if (Get-AppxPackage -AllUsers 'Microsoft.XboxGamingOverlay') { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, app: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, **`risk_level: medium`**

*What it actually does.* Identity confirmed. The Store catalog resolves `9NZKPSTSNW4P` to "Game Bar"
with package family `Microsoft.XboxGamingOverlay_8wekyb3d8bbwe`, category "System Components", and
`winget show` resolves it in the msstore source, so the undo is executable. Removing the package
removes the Win+G overlay and its capture, performance and social widgets. It does **not** affect
Game Mode, hardware-accelerated GPU scheduling, or Auto HDR, which the tweak name correctly notes.
`XboxGamingOverlay` is in Microsoft's 24H2 and 25H2 policy-based inbox app removal list.

*Applicability.* Present on Windows 10 1709 and later and all Windows 11 builds, all consumer SKUs.
Administrator rights required for `-AllUsers`. No reboot. This is a **bundle**, so per Microsoft's
guidance the `Get-AppxPackage` side should use `-PackageTypeFilter Bundle` when removing for all
users. The package is also marked as a system component, and on some builds `Remove-AppxPackage` can
fail with 0x80073CFA for packages the servicing stack considers non-removable, in which case the
action correctly surfaces an error.

*Cautions.* Games and apps that invoke the `ms-gamingoverlay:` URI produce the "You'll need a new app
to open this ms-gamingoverlay link" dialog after removal. Xbox Identity Provider is a separate package
and must be kept for game sign-in. The `risk_level: medium` classification is appropriate.

**Corrections needed:** Four items. (1) The provisioned package is not removed; add
`Remove-AppxProvisionedPackage -Online` and extend the probe. (2) Use `-PackageTypeFilter Bundle` on
the `Get-AppxPackage` call. (3) Make the probe fail-closed. (4) Consider suppressing the
`ms-gamingoverlay` prompt as a companion effect
(`HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR\AppCaptureEnabled` and the `GameBar`
`ShowStartupPanel` value) so the removal does not leave a visible error dialog.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the Win+G Game Bar overlay, without touching Game Mode or any GPU feature.**

      ## What it does
      Removes the `Microsoft.XboxGamingOverlay` Appx package for all users. That is the overlay
      itself: capture, the performance widget, and the Xbox social panels. Game Mode, hardware
      accelerated GPU scheduling and Auto HDR are separate and unaffected.

      ## Benefits
      - **No overlay hooks**: the capture and overlay layer stops loading with games
      - **No accidental Win+G**: the overlay cannot open mid-game
      - **Game Mode is untouched**: this removes an app, not a performance feature

      ## Drawbacks
      - **ms-gamingoverlay errors**: games that call the overlay URI show a "You'll need a new app to
        open this ms-gamingoverlay link" dialog
      - **You lose built-in capture**: the Win+Alt+R recorder and screenshot hotkeys go with it
      - **Some builds refuse**: the package is flagged as a system component and removal can fail
        with 0x80073CFA, which the tool reports as an error rather than hiding
      - **Feature updates can restore it**: a Windows feature update may re-provision the package, in
        which case apply the tweak again

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the app is uninstalled on apply
      - **Reverting**: reinstalls Game Bar from the Microsoft Store
      - Keep the Xbox Identity Provider package, which game sign-in depends on

      ## Recommendation
      Remove it only if you never press Win+G and can live with the occasional ms-gamingoverlay
      prompt. If you only want background recording off, use the Game DVR tweak instead; it is a much
      smaller change.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy-based inbox app removal, supported app list](https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal)
      - [Remove-AppxPackage, bundle guidance](https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage)
```

**Sources:**
1. Microsoft Store catalog service, product `9NZKPSTSNW4P` returns title "Game Bar", package family `Microsoft.XboxGamingOverlay_8wekyb3d8bbwe`, category "System Components", https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NZKPSTSNW4P?market=US&locale=en-us&deviceFamily=Windows.Desktop (tier A, primary observation)
2. Remove-AppxPackage, `-AllUsers` parameter and bundle guidance, https://learn.microsoft.com/en-us/powershell/module/appx/remove-appxpackage (tier A)
3. Policy-based inbox app removal supported app list, which includes `XboxGamingOverlay` and `XboxIdentityProvider` as separate ids, https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal (tier A)

### `remove_onedrive` OneDrive

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** state marker `HKCU\Software\MagicXToolbox\Debloat`, value `OneDrive`, `REG_DWORD`,
plus a PowerShell action. This is not an Appx removal; OneDrive ships as a Win32 setup payload.

- Apply:
  `$s = "$env:SystemRoot\SysWOW64\OneDriveSetup.exe"; if (-not (Test-Path $s)) { $s = "$env:SystemRoot\System32\OneDriveSetup.exe" }; Start-Process $s '/uninstall' -Wait`
- Undo: `winget install --id Microsoft.OneDrive -e --accept-source-agreements --accept-package-agreements`
- Probe:
  `if (Get-Process OneDrive -ErrorAction SilentlyContinue) { exit 1 } elseif (Test-Path "$env:LOCALAPPDATA\Microsoft\OneDrive\OneDrive.exe") { exit 1 } else { exit 0 }`
- Options: "Removed" = `{state: 1, onedrive: run}`; "Installed (Stock Default)" = `{state: 0}`
- No `windows` gate, `elevation: admin`, **`risk_level: medium`**

*What it actually does.* The SysWOW64-before-System32 ordering is correct for the in-box per-user
stub on 64-bit Windows. Windows ships a 32-bit `OneDriveSetup.exe`, which on a 64-bit installation
lives in `%SystemRoot%\SysWOW64`; `%SystemRoot%\System32` holds the 64-bit copy on ARM64 and on builds
where a 64-bit OneDrive stub is shipped. Running that stub with `/uninstall` removes the per-user
OneDrive installation. The `winget` undo is valid: `winget show --id Microsoft.OneDrive --exact`
resolves to "Microsoft OneDrive" version 26.113.0614.0004, publisher Microsoft Corporation, moniker
`onedrive`.

*The gap is per-machine OneDrive.* A per-machine install is created by running
`OneDriveSetup.exe /allusers` and places the client under `%ProgramFiles%\Microsoft OneDrive` (or
`Program Files (x86)` depending on architecture), with the uninstall command
`%ProgramFiles(x86)%\Microsoft OneDrive\<version>\OneDriveSetup.exe /uninstall /allusers`. The
tweak's apply never looks in `Program Files` and never passes `/allusers`, so on a per-machine
deployment (which is what Microsoft 365 and modern imaging use) it does nothing. The probe has the
same blind spot: it checks only for a running process and for the per-user path under
`%LOCALAPPDATA%`, so a per-machine install with the process stopped reads as "removed".

*Applicability.* Windows 10 and all Windows 11 builds where OneDrive is present. **Not present at all
on LTSC and IoT LTSC images**: neither `SysWOW64\OneDriveSetup.exe` nor `System32\OneDriveSetup.exe`
exists on Windows 11 IoT Enterprise LTSC build 26100.4061, where the apply resolves to a non-existent
path. Administrator rights are requested, though a per-user uninstall does not strictly need them. No
reboot. Unlike every other removal in this file, this one remains meaningful on the Windows 10 IoT
Enterprise LTSC 2021 secondary target, because OneDrive is a Win32 payload rather than a Store app.

*Cautions.* The apply does not verify that the uninstaller ran or what it returned; `Start-Process
... -Wait` discards the child's exit code. Combined with the probe's per-machine blind spot, a machine
can be reported as "OneDrive removed" with OneDrive still installed. A feature update can reinstall
the OneDrive stub later.

**Corrections needed:** Five items. (1) Handle per-machine installs: check
`%ProgramFiles%\Microsoft OneDrive` and `%ProgramFiles(x86)%\Microsoft OneDrive` for a versioned
`OneDriveSetup.exe` and call it with `/uninstall /allusers`. (2) Extend the probe to those paths.
(3) Check the uninstaller's exit code rather than discarding it. (4) Either add the
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run` OneDrive value removal that the info text
promises, or delete that sentence. (5) Fail rather than silently succeed when no `OneDriveSetup.exe`
can be found anywhere, which is the LTSC case.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the OneDrive client so it stops syncing and stops asking to back up your
      folders.**

      ## What it does
      Runs the in-box `OneDriveSetup.exe /uninstall`, which unlinks the account, removes the client
      and drops its startup entry. Files already downloaded to your disk are left where they are.

      ## Benefits
      - **No background sync**: no OneDrive process, no upload activity, no sync icon
      - **No backup nags**: the recurring "back up your folders" prompts stop
      - **Local files stay**: anything already downloaded remains on disk

      ## Drawbacks
      - **Cloud-only files become unreachable**: Files On-Demand placeholders that were never
        downloaded are lost to you locally, so download them first
      - **Known Folder Move can misplace folders**: if Desktop, Documents or Pictures were redirected
        into OneDrive, they can end up somewhere you do not expect
      - **Office AutoSave changes**: documents stop auto-saving to the cloud
      - **A feature update can reinstall the stub**: apply the tweak again if it comes back

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10; LTSC images do not ship OneDrive
        at all
      - **Takes effect**: immediately, the uninstaller runs on apply
      - **Reverting**: reinstalls OneDrive through winget, which needs internet access
      - Unlink your account and confirm nothing is cloud-only before you apply this

      ## Recommendation
      Remove it if you never use OneDrive and have checked that no files are cloud-only and no
      folders are redirected. If you sync anything, or use Known Folder backup, leave it alone.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [Turn off, disable, or uninstall OneDrive](https://support.microsoft.com/en-us/office/turn-off-disable-or-uninstall-onedrive-f32a17ce-3336-40fe-9c38-6efb09f944b0)
      - [Install OneDrive per machine](https://learn.microsoft.com/en-us/sharepoint/per-machine-installation)
```

**Sources:**
1. `winget show --id Microsoft.OneDrive --exact` resolves to "Microsoft OneDrive" version 26.113.0614.0004, publisher Microsoft Corporation, moniker `onedrive` (primary observation)
2. Per-machine OneDrive install location and `/uninstall /allusers` uninstall command, ManageEngine OneDrive administration reference, https://www.manageengine.com/microsoft-365-management-reporting/kb/onedrive-administration/per-machine-installation.html (tier C)
3. "Installing the OneDrive Sync Client in Per-Machine mode", byteben, https://byteben.com/bb/installing-the-onedrive-sync-client-in-per-machine-mode-during-your-task-sequence-for-a-lightening-fast-first-logon-experience/ (tier C)
4. Direct filesystem inspection on Windows 11 IoT Enterprise LTSC build 26100.4061: neither `SysWOW64\OneDriveSetup.exe` nor `System32\OneDriveSetup.exe` exists (primary observation)
5. Microsoft, Turn off, disable, or uninstall OneDrive, https://support.microsoft.com/en-us/office/turn-off-disable-or-uninstall-onedrive-f32a17ce-3336-40fe-9c38-6efb09f944b0 (tier A)

## Unknowns and follow-ups

1. **`Microsoft.GetHelp` and `Microsoft.YourPhone` presence on a stock 24H2 image.** Both are absent
   from Microsoft's policy-removable inbox app list, which may mean not preinstalled or may mean
   classified as a system component. A `Get-AppxPackage` check on a clean image would settle it.
2. **`Microsoft.Getstarted` presence on a stock 24H2 image.** Deprecated by Microsoft and absent from
   the removal list, but no explicit "no longer preinstalled" statement exists of the kind published
   for Maps.
3. **Whether `disable_web_search_start` still hides the taskbar search box on any current build.**
   The Microsoft Q&A report is tier D and unconfirmed on 26100.
4. **Whether the `MSTeams` retarget of `remove_teams_consumer_app` is wanted at all**, given that the
   unified package is the same one work and school deployments use. This is a product decision, not a
   research question.
5. **Whether `disable_settings_account_ads` should ship to Home and Pro users at all**, given that
   the policy is documented as unsupported there. Shipping it with honest copy is defensible under
   the inclusion principle; hiding it on unsupported editions is also defensible.
