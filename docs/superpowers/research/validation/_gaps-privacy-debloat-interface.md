# Corpus gap hunt: privacy and telemetry, debloat and consumer/AI, interface and Explorer and taskbar

**Scope.** Windows 11 24H2 (26100) and 25H2 (26200) primary. Windows 10 IoT Enterprise LTSC 2021
(19044) secondary. This is a *gap* pass: it lists tweaks that should exist in the corpus and do not.
It does not re-validate anything already shipped.

**Method.** Inventory of the 100 tweaks currently in `privacy.yaml` (29), `debloat.yaml` (29) and
`interface.yaml` (36) plus every registry `key` / `name` pair across all seven category files (238
distinct pairs), diffed against:

- privacy.sexy `src/application/collections/windows.yaml` (structured `keyPath` / `valueName` pairs)
- Chris Titus WinUtil `config/tweaks.json` and `config/feature.json`
- Sophia Script for Windows 11, `Module/Sophia.psm1`
- Win11Debloat `Regfiles/*.reg` (read as raw `.reg` bodies, not from the README)
- Optimizer (`hellzerg/optimizer`) repository tree and embedded `.reg` resources
- The shipped ADMX/ADML set at `C:\Windows\PolicyDefinitions` on a live **26100** machine, parsed for
  `key` / `valueName` / `enabledValue` / `disabledValue` / `supportedOn`
- Microsoft Policy CSP reference pages (WindowsAI, Experience, Start, Search, System, TextInput,
  NewsAndInterests, Notifications) and the Microsoft Edge policy reference

`_rescope-24h2.md` was read first; nothing it marks DELETE or OBSOLETE is proposed here.

**Source tiers used below.**

- **T1** Microsoft primary: shipped ADMX/ADML on 26100, or a Policy CSP / Learn page with the exact
  registry key, value name and allowed values.
- **T2** Three or more genuinely independent community projects or references agreeing on key, value
  name, type and semantics.
- **T3** Fewer than three independent sources. Nothing at T3 is proposed; T3 items are in the
  rejected list.

---

## Ranked proposals

| # | Proposed id | Category | One-line description | Mechanism | Value | Tier |
|---|---|---|---|---|---|---|
| 1 | `disable_start_recommended_section` | interface | Remove the whole Recommended section from Start, not just its promo content | `HideRecommendedSection` policy | High | T1 |
| 2 | `disable_explorer_cloud_recommendations` | privacy | Stop Explorer requesting cloud file metadata for Home, Recent, Favorites and the details pane | `DisableGraphRecentItems` policy | High | T1 |
| 3 | `disable_account_notifications` | debloat | Kill the OneDrive, Microsoft 365, Xbox and backup upsell badges on the Start user tile | `DisableAccountNotifications` policy | High | T1 |
| 4 | `disable_app_device_inventory` | privacy | Turn off the four App and Device Inventory collectors added in 24H2 | 4 x `AppCompat` policy values | High | T1 |
| 5 | `disable_ai_fabric_service` | debloat | Stop the 25H2 Windows AI Fabric service and its WorkloadsSessionHost children | `WSAIFabricSvc` service start type | High | T2 |
| 6 | `disable_notepad_ai` | debloat | Remove Copilot, Rewrite and Summarize from Notepad | `DisableAIFeatures` policy | High | T1 |
| 7 | `disable_paint_ai` | debloat | Remove Cocreator, Image Creator and Generative Fill from Paint | 3 x `Policies\Paint` values | High | T1 |
| 8 | `disable_settings_account_ads` | debloat | Remove the Microsoft 365 and account-state ad cards from the Settings homepage | `DisableConsumerAccountStateContent` | High | T1 |
| 9 | `disable_phone_companion_start` | debloat | Remove the Phone Link companion panel from the Start menu | `Start\Companions` `IsEnabled` | High | T2 |
| 10 | `disable_drag_tray` | interface | Remove the drag-to-share tray that appears when dragging a file | `CDP` `DragTrayEnabled` | High | T2 |
| 11 | `hide_unsupported_hardware_notice` | interface | Hide the "system requirements not met" desktop watermark and Settings banner | `HideUnsupportedHardwareNotifications` | High | T1 |
| 12 | `disable_edge_ai_features` | debloat | Turn off Edge Copilot page context, AI history search, inline Compose and new-tab Bing Chat | 5 x Edge policy values | High | T1 |
| 13 | `disable_windows_spotlight_all` | privacy | Single switch that turns off every Windows Spotlight surface at once | `DisableWindowsSpotlightFeatures` | Medium | T1 |
| 14 | `disable_third_party_suggestions` | privacy | Block third-party publisher app and content suggestions in Spotlight surfaces | `DisableThirdPartySuggestions` | Medium | T1 |
| 15 | `disable_spotlight_desktop` | interface | Remove the Spotlight desktop-wallpaper option and its daily image downloads | `DisableSpotlightCollectionOnDesktop` | Medium | T1 |
| 16 | `disable_search_history` | privacy | Stop Windows Search recording a per-device search history | `IsDeviceSearchHistoryEnabled` + policy | Medium | T2 |
| 17 | `disable_cloud_content_search` | privacy | Stop taskbar search querying OneDrive, SharePoint and Outlook cloud content | 2 x `SearchSettings` + `AllowCloudSearch` | Medium | T1 |
| 18 | `disable_voice_activation` | privacy | Block apps from listening for a wake word, including above the lock screen | 2 x `AppPrivacy` policy values | Medium | T1 |
| 19 | `disable_windows_backup` | privacy | Lock the periodic Windows Backup to OneDrive off | `EnableWindowsBackup` = 0 | Medium | T1 |
| 20 | `disable_online_tips` | privacy | Stop the Settings app contacting Microsoft content services for tips and help | `AllowOnlineTips` = 0 | Medium | T1 |
| 21 | `hide_settings_ai_page` | debloat | Hide the AI components page from the Settings app | `SettingsPageVisibility` = `hide:aicomponents` | Medium | T1 |
| 22 | `disable_nag_toasts` | debloat | Silence the "Suggested" and "Backup reminder" system toasts specifically | 2 x per-app notification `Enabled` | Medium | T2 |
| 23 | `disable_snap_assist` | interface | Stop the "pick a window for the other half" prompt after snapping | `SnapAssist` = 0 | Medium | T2 |
| 24 | `alt_tab_hide_browser_tabs` | interface | Show only windows in Alt+Tab, never browser tabs | `MultiTaskingAltTabFilter` = 3 | Medium | T2 |
| 25 | `disable_widgets_lock_screen` | interface | Remove the widgets panel from the lock screen | `Dsh` `DisableWidgetsOnLockScreen` | Medium | T1 (conflicted, see entry) |
| 26 | `taskbar_last_active_click` | interface | Clicking a grouped taskbar icon focuses the last active window instead of opening the flyout | `LastActiveClick` = 1 | Low | T2 |
| 27 | `explorer_expand_to_current_folder` | interface | Auto-expand the navigation pane to the folder you are in | `NavPaneExpandToCurrentFolder` | Low | T2 |
| 28 | `explorer_restore_folders_at_logon` | interface | Reopen the Explorer windows that were open at shutdown | `PersistBrowsers` | Low | T2 |
| 29 | `taskbar_full_date_time` | interface | Show the unabbreviated date and time format in the tray clock | `TurnOffAbbreviatedDateTimeFormat` | Low | T1 |
| 30 | `hide_recently_added_apps` | interface | Remove the "Recently added" group from the Start all-apps list | `HideRecentlyAddedApps` | Low | T1 |
| 31 | `disable_new_app_alert` | interface | Suppress the "new application installed" file-association notification | `NoNewAppAlert` = 1 | Low | T1 |
| 32 | `disable_notification_center` | interface | Remove the Notification Center and calendar flyout from the tray | `DisableNotificationCenter` = 1 | Low | T1 |
| 33 | (extend `disable_suggested_content_settings`) | privacy | Add the one Settings suggestion slot the existing tweak misses | `SubscribedContent-353698Enabled` | Low | T2 |

Counts: **12 High, 13 Medium, 8 Low.**

---

## Full entries

### 1. `disable_start_recommended_section` (interface, High)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` and
  `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `HideRecommendedSection`
- **Type:** `REG_DWORD`
- **Options:** Hidden = `1`; Shown (Stock Default) = `absent`
- **Stock default:** value-absent. CSP default value is `0`.
- **Applicability:** Windows 11 22H2 (22621) and later, so all of 26100 and 26200. Pro, Enterprise,
  Education, IoT Enterprise / IoT Enterprise LTSC per the Policy CSP. Not applicable to LTSC 2021.
- **Why it is not a duplicate:** the corpus ships `disable_start_recommendations`, which sets
  `Start_IrisRecommendations` = 0. That value only suppresses the *promotional* rows (tips,
  shortcuts, suggested new apps). The Recommended **section itself** stays, still listing recently
  opened files and newly installed apps. `HideRecommendedSection` removes the section outright and
  reflows the pinned grid. Different surface, different value, different result.
- **Risks:** low. Purely cosmetic and per-policy. Note that the shipped `StartMenu.admx` on 26100
  declares `supportedOn` as `SUPPORTED_Windows_11_0_SE` while the Policy CSP page lists Pro,
  Enterprise, Education and IoT Enterprise. Microsoft's CSP page is the more current of the two and
  is what admins report working on Pro. If a probe finds the section still present on Pro, the
  documented MDM mirror path
  `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Start` `HideRecommendedSection` is the
  fallback both WinUtil and Sophia Script write.
- **Sources:** Policy CSP - Start > HideRecommendedSection (registry key, value name, allowed values,
  editions); `C:\Windows\PolicyDefinitions\StartMenu.admx` on 26100 (policy element, key, valueName);
  `C:\Windows\PolicyDefinitions\en-US\StartMenu.adml` friendly name "Remove Recommended section from
  Start Menu"; WinUtil `tweaks.json` (ships a restore-to-`0` action, confirming real-world use);
  Sophia Script `Sophia.psm1`.

### 2. `disable_explorer_cloud_recommendations` (privacy, High)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `DisableGraphRecentItems`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent, meaning cloud metadata requests are allowed.
- **Applicability:** Windows 11 22H2 and later, client only (`SUPPORTED_Windows_11_0_22H2_NOSERVER`).
  Machine scope. Not applicable to LTSC 2021.
- **What it actually does:** the 26100 ADML text is explicit: "Turning off this setting will prevent
  File Explorer from requesting cloud file metadata and displaying it in the homepage and other views
  in File Explorer. Any insights and files available based on account activity will be stopped in
  views such as Recent, Recommended, Favorites, Details pane, etc." This is a **network** control,
  not a display toggle: it stops the Microsoft Graph calls, not just the rendering.
- **Why it is not a duplicate:** the corpus has `remove_home_nav_pane` (hides the Home node) and
  `disable_recent_files` (`ShowRecent` / `ShowFrequent`, local MRU). Neither stops Explorer from
  calling Graph for account-based file insights, and the details pane keeps doing so even with Home
  hidden.
- **Risks:** low. Cloud-sourced "Recommended" files stop appearing in Explorer Home. Local recents
  are unaffected. Fully reversible by deleting the value.
- **Sources:** `C:\Windows\PolicyDefinitions\Explorer.admx` on 26100 (policy `DisableGraphRecentItems`,
  class Machine, key `Software\Policies\Microsoft\Windows\Explorer`);
  `C:\Windows\PolicyDefinitions\en-US\Explorer.adml` (display and help strings quoted above).
  Notably absent from privacy.sexy, WinUtil, Sophia Script and Win11Debloat, which is exactly the
  "no tweak tool exposes it" case the brief asked for.

### 3. `disable_account_notifications` (debloat, High)

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CurrentVersion\AccountNotifications`
- **Value name:** `DisableAccountNotifications`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 20H1 and later, client only
  (`SUPPORTED_Windows_10_0_20H1_NOSERVER`). User scope. Applies to 26100, 26200 and LTSC 2021.
- **What it removes:** Microsoft's own Start-policy documentation lists the notification classes this
  kills: reauthenticate prompts, "back up your device", cloud storage quota nags, and Microsoft 365 /
  Xbox subscription pitches, all rendered on the Start user tile. This is the single largest
  remaining account-upsell surface on a clean 24H2 install.
- **Why it is not a duplicate:** the corpus covers ContentDeliveryManager suggestion slots and
  `ScoobeSystemSettingEnabled`, none of which touch the user-tile notification channel. The
  per-user, non-policy equivalent that the Settings UI writes is
  `HKCU\...\Explorer\Advanced` `Start_AccountNotifications`; the policy above supersedes it and is
  the version Microsoft documents. Consider writing both, with the policy as the primary effect.
- **Risks:** low. You stop being told that your OneDrive is full or that a backup is pending. If the
  user relies on those reminders this is a behaviour change, not a defect.
- **Sources:** `C:\Windows\PolicyDefinitions\AccountNotifications.admx` on 26100 (class User, key and
  valueName, enabledValue `1` / disabledValue `0`); Microsoft Learn "Start menu policy settings >
  Disable Account Notifications" (maps to `./User/.../Notifications/DisableAccountNotifications`,
  GPO path Windows Components > Account Notifications); Win11Debloat
  `Regfiles/Disable_Windows_Suggestions.reg` and Sophia Script (both write the
  `Start_AccountNotifications` sibling).

### 4. `disable_app_device_inventory` (privacy, High)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat` (all four values)
- **Value names / types / options** (each `REG_DWORD`, each Disabled = `1`, Stock Default = `absent`):
  - `DisableInstallTracing` - Install Tracing, which tracks application installs
  - `DisableAPISamping` - API Sampling, sampled collection of APIs used at runtime
    (note the misspelling: it really is `DisableAPISamping`, not `Sampling`)
  - `DisableApplicationFootprint` - sampled collection of registry and file usage
  - `DisableWin32AppBackup` - the compatibility scan over backed-up applications
- **Stock default:** all four value-absent, all four collectors running.
- **Applicability:** `SUPPORTED_Windows_11_0_24H2`. **New in 24H2**, which is precisely why no
  pre-24H2 tweak list contains them. Machine scope. Does not apply to LTSC 2021.
- **Why it is not a duplicate:** the corpus has `disable_compat_appraiser` covering `AITEnable` and
  `DisableInventory` under the same `AppCompat` key. Those are the *legacy* Application Impact
  Telemetry and inventory switches. The four above are a distinct, newer collection family with their
  own Group Policy category ("App and Device Inventory") that Microsoft added in 24H2.
- **Risks:** low. These collectors exist to diagnose app-compatibility problems, so a user who later
  hits a compatibility issue gives up some Microsoft-side diagnostics. No functional impact on app
  execution. Value the `DisableWin32AppBackup` one carefully if the user relies on Windows Backup app
  restore, since it turns off the compatibility scan over backed-up apps.
- **Sources:** `C:\Windows\PolicyDefinitions\AppDeviceInventory.admx` on 26100 (four policies, class
  Machine, shared key, `supportedOn ref="windows:SUPPORTED_Windows_11_0_24H2"`, enabledValue `1`);
  `C:\Windows\PolicyDefinitions\en-US\AppDeviceInventory.adml` (the descriptions quoted above).

### 5. `disable_ai_fabric_service` (debloat, High, 25H2 only)

- **Mechanism:** service, not registry. Service name `WSAIFabricSvc` ("Windows AI Fabric Service").
  Use the corpus's existing `service` effect kind rather than writing
  `HKLM\SYSTEM\CurrentControlSet\Services\WSAIFabricSvc` `Start` directly.
- **Options:** Manual = start type `Manual` (SC `demand`, registry `Start` = 3); Automatic (Stock
  Default) = start type `Automatic`.
- **Stock default:** present on 25H2 (26200) images and starting automatically. See UNKNOWNS.
- **Applicability:** Windows 11 25H2 (26200) and later. The service does not exist on 26100 or LTSC
  2021, so the tweak must be gated `build >= 26200` and must fail cleanly when the service is absent.
- **Why it matters:** this is a genuine resource item, not a placebo. The service supervises
  `WorkloadsSessionHost.exe` processes; multiple independent reports, including a Microsoft Q&A
  thread, document eight such processes and multi-gigabyte resident memory on idle machines. That is
  a measurable, user-visible win, distinct from the debunked "telemetry costs you FPS" claims.
- **Risks:** medium. Setting Manual rather than Disabled is the right default so on-demand AI
  features still start when explicitly invoked. Disabled would break Click to Do and any 25H2 agentic
  feature that depends on the fabric. Reversible by restoring the original start type.
- **Sources:** Microsoft Q&A "Windows 11 25H2: WSAIFabricSvc launches 8 WorkloadsSessionHost
  processes and consumes about 5 GB of RAM"; Win11Debloat `Regfiles/Disable_AI_Service_Auto_Start.reg`
  (sets `Start` = 3) and its tracking issue #497; `revertservice.com` and `batcmd.com` service
  reference entries for `WSAIFabricSvc` on Windows 11.

### 6. `disable_notepad_ai` (debloat, High)

- **Key:** `HKLM\SOFTWARE\Policies\WindowsNotepad`
- **Value name:** `DisableAIFeatures`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 11 22H2 and later with Notepad app version 11.2503.16.0 or later, so all
  current 26100 and 26200 machines that have the Store Notepad. Inert on LTSC 2021, which ships the
  classic Win32 Notepad with no AI surface. Gate on `products: [11]`.
- **What it removes:** Copilot entry point, Rewrite and Summarize inside Notepad.
- **Risks:** low. It is an app-scoped policy; nothing outside Notepad changes. Fully reversible.
- **Sources:** Microsoft's `DisableAIFeaturesInNotepad` policy (Intune / Group Policy / Registry,
  documented with this exact registry path); Win11Debloat
  `Regfiles/Disable_Notepad_AI_Features.reg`; WinUtil `tweaks.json`; ElevenForum and TechPress
  writeups agreeing on key, value name and semantics.

### 7. `disable_paint_ai` (debloat, High)

- **Key:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint`
- **Value names:** `DisableCocreator`, `DisableGenerativeFill`, `DisableImageCreator`
- **Type:** `REG_DWORD` (each)
- **Options:** Disabled = `1` on all three; Enabled (Stock Default) = `absent` on all three
- **Stock default:** all three value-absent; CSP default value is `0`.
- **Applicability:** Windows 11 22H2 (22621.4870) and later, and 24H2 (26100.3360) and later. Pro,
  Enterprise, Education, IoT Enterprise / IoT Enterprise LTSC. Machine scope. Gate `products: [11]`.
- **Risks:** low. Paint keeps working; only the generative surfaces disappear.
- **Two extra values to consider but not to claim as T1:** Win11Debloat also writes
  `DisableGenerativeErase` and `DisableRemoveBackground` under the same key. Neither appears in the
  26100 `WindowsCopilot.admx` and neither is in the Policy CSP, so they are T3 at best. Ship the
  three confirmed values; treat the other two as an optional extra effect only if a probe confirms
  them on a live 26200 box.
- **Sources:** Policy CSP - WindowsAI > DisableImageCreator / DisableCocreator / DisableGenerativeFill
  (each gives Registry Key Name `Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, the value
  name, and `0 (Default)` / `1`); `C:\Windows\PolicyDefinitions\WindowsCopilot.admx` on 26100;
  Win11Debloat `Regfiles/Disable_Paint_AI_Features.reg`.

### 8. `disable_settings_account_ads` (debloat, High)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent`
- **Value name:** `DisableConsumerAccountStateContent`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent; CSP default `0`.
- **Applicability:** Windows 10 2004 (`SUPPORTED_Windows_10_0_RS7`) and later. Per the Policy CSP the
  *device*-scoped cloud-content policies in this family are Enterprise, Education and IoT Enterprise
  only, and are ignored on Pro and Home. Since IoT Enterprise LTSC is in scope this is still worth
  shipping, but the tweak copy must say plainly that Pro and Home will not honour it.
- **What it removes:** the "cloud consumer account state content" injected into Windows experiences,
  which in practice is the Microsoft 365 and OneDrive upsell card on the Settings homepage and the
  account-state banners that ride alongside it.
- **Why it is not a duplicate:** the corpus has `DisableWindowsConsumerFeatures` and
  `DisableSoftLanding` under the same key. Consumer Features governs auto-installed suggested apps
  and Spotlight-driven content; it does not govern the account-state cards. Win11Debloat ships this
  value as a separate `Disable_Settings_365_Ads.reg` for exactly this reason.
- **Risks:** low, and inert on Pro and Home.
- **Sources:** `C:\Windows\PolicyDefinitions\CloudContent.admx` on 26100 (class Machine, enabledValue
  `1`); `en-US\CloudContent.adml` "Turn off cloud consumer account state content"; Policy CSP -
  Experience > `DisableCloudOptimizedContent` sibling entry for the edition matrix; Win11Debloat
  `Regfiles/Disable_Settings_365_Ads.reg`.

### 9. `disable_phone_companion_start` (debloat, High)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Start\Companions\Microsoft.YourPhone_8wekyb3d8bbwe`
- **Value name:** `IsEnabled`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent, panel shown once Phone Link has been set up. The key itself may
  need to be created.
- **Applicability:** Windows 11 23H2 and later; the panel shipped broadly on 24H2. User scope. Not
  present on LTSC 2021. Requires sign-out or restart to take effect.
- **Why it is not a duplicate:** the corpus has `remove_phone_link` (removes the `YourPhone` appx)
  and `_rescope-24h2.md` already flags that appx removal as NEEDS GATE CHANGE because Phone Link is
  absent from Microsoft's 24H2/25H2 policy-removable inbox list. Removing the app is a different and
  currently less reliable action than hiding the Start companion panel. This tweak covers the case
  where the user wants Phone Link installed but does not want a permanent phone sidebar eating a
  third of the Start menu.
- **Risks:** low. Phone Link itself keeps working; only the Start panel goes.
- **Sources:** Win11Debloat `Regfiles/Disable_Phone_Link_In_Start.reg`; ElevenForum tutorial "Add or
  Remove Phone Link Mobile Device on Start Menu in Windows 11"; NinjaOne "How to Manage Recent Mobile
  Device Content in Windows 11"; a Microsoft Q&A thread on the same key. Four independent sources
  agreeing on key, value name, type and `0` / `1` semantics.

### 10. `disable_drag_tray` (interface, High)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\CDP`
- **Value name:** `DragTrayEnabled`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** the feature shipped in 24H2 build **26100.4202** and 23H2 build 22631.5413.
  Gate `build >= 26100` and note in the copy that on very early 26100 servicing levels there is
  nothing to disable. Not present on LTSC 2021. User scope.
- **What it removes:** the tray that pops in at the top of the screen the moment you start dragging a
  file out of Explorer or off the desktop, offering share targets. It is a new 24H2 interaction that
  intercepts an existing muscle-memory gesture, which is why it draws complaints.
- **Risks:** low. Windows Share is still reachable from the context menu.
- **Sources:** Win11Debloat `Regfiles/Disable_Share_Drag_Tray.reg`; Pureinfotech "How to disable Drag
  Tray (Drop Tray) sharing UI on Windows 11"; ElevenForum "Enable or Disable Drop Tray in Windows 11";
  MajorGeeks "How To Disable Drag Tray"; a Microsoft Q&A thread naming build 26100.4202 as the
  introduction point.

### 11. `hide_unsupported_hardware_notice` (interface, High)

- **Key:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`
- **Value name:** `HideUnsupportedHardwareNotifications`
- **Type:** `REG_DWORD`
- **Options:** Hidden = `1`; Shown (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** `SUPPORTED_Windows_11_0_NOSERVER`, so all Windows 11 client editions including
  26100 and 26200. Machine scope. Not applicable to LTSC 2021 (Windows 10).
- **What it removes:** the ADML text is unambiguous: "This policy controls messages which are shown
  when Windows is running on a device that does not meet the minimum system requirements for this OS
  version. If you enable this policy setting, these messages will never appear on desktop or in the
  Settings app." That is the desktop watermark plus the Settings > System > About banner.
- **Why it earns High:** a large share of 24H2 and 25H2 installs are on officially unsupported
  hardware, and this is a first-party, fully documented, reversible way to remove the nag. It is also
  a good example of the brief's premise: it is a shipped Microsoft policy that essentially no tweak
  tool surfaces.
- **Risks:** low, and purely cosmetic. It does not change update eligibility or servicing behaviour.
  The tweak copy should say that explicitly so nobody reads it as "makes my PC supported".
- **Sources:** `C:\Windows\PolicyDefinitions\ControlPanel.admx` on 26100 (class Machine, key and
  valueName exactly as above, `supportedOn ref="windows:SUPPORTED_Windows_11_0_NOSERVER"`);
  `C:\Windows\PolicyDefinitions\en-US\ControlPanel.adml` (help text quoted above).

### 12. `disable_edge_ai_features` (debloat, High)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Edge` (all values)
- **Value names / types / options** (each `REG_DWORD`, each Off = `0`, Stock Default = `absent`):
  - `CopilotPageContext` - lets Copilot in the side pane read the current page
  - `EdgeEntraCopilotPageContext` - the Entra-profile equivalent of the above
  - `EdgeHistoryAISearchEnabled` - AI search over browsing history
  - `ComposeInlineEnabled` - inline AI writing in text fields
  - `NewTabPageBingChatEnabled` - Bing Chat entry point on the new tab page
- **Stock default:** all value-absent, all features on.
- **Applicability:** current Edge stable on any supported Windows, so 26100, 26200 and LTSC 2021 all
  qualify wherever Edge is installed. Machine scope.
- **Why it is not a duplicate:** the corpus already sets `HubsSidebarEnabled` = 0 (sidebar and
  Discover) plus a set of Edge telemetry values. The sidebar switch removes the *panel*; it does not
  stop Copilot reading page content in other entry points, does not touch AI history search, does not
  touch inline Compose, and does not touch the new-tab Bing Chat button. If a single stronger switch
  is preferred, `EdgeCopilotEnabled` = 0 is also documented and removes Copilot in Edge entirely; that
  would make a good third option value on this tweak.
- **Risks:** low. Edge shows "managed by your organization" on its settings page once any Edge policy
  is set, which is worth calling out in the tweak copy since the corpus's existing Edge tweaks have
  the same side effect.
- **Sources:** Microsoft Edge Browser Policy Documentation pages for `CopilotPageContext`,
  `EdgeEntraCopilotPageContext`, `EdgeCopilotEnabled` and `Microsoft365CopilotChatIconEnabled`;
  Win11Debloat `Regfiles/Disable_Edge_AI_Features.reg`; privacy.sexy `SetEdgePolicyViaRegistry`
  entries.

### 13. `disable_windows_spotlight_all` (privacy, Medium)

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`
- **Value name:** `DisableWindowsSpotlightFeatures`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent; CSP default `0`.
- **Applicability:** Windows 10 1607 and later, client only. User scope. Applies to 26100, 26200 and
  LTSC 2021.
- **Why Medium not High:** the corpus already covers the two most visible Spotlight outputs
  (`RotatingLockScreenOverlayEnabled` and `SubscribedContent-338387Enabled`). This is the single
  master switch that also covers the Spotlight welcome experience and Action Center Spotlight, so it
  is a real superset, but the marginal gain over what is already shipped is moderate.
- **Risks:** low. Lock-screen imagery falls back to a static picture.
- **Sources:** `CloudContent.admx` / `.adml` on 26100 ("Turn off all Windows spotlight features");
  Policy CSP - Experience > AllowWindowsSpotlight family; privacy.sexy.

### 14. `disable_third_party_suggestions` (privacy, Medium)

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`
- **Value name:** `DisableThirdPartySuggestions`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent; CSP default `1` on the `AllowThirdPartySuggestionsInWindowsSpotlight`
  side, meaning third-party suggestions are allowed.
- **Applicability:** Windows 10 1607 and later, client only. Pro, Enterprise, Education, IoT
  Enterprise / IoT Enterprise LTSC. User scope.
- **Semantics gotcha worth encoding:** the CSP name is `AllowThirdPartySuggestionsInWindowsSpotlight`
  (`0` = not allowed) but the registry value is the inverted
  `DisableThirdPartySuggestions` (`1` = disabled). The registry form is what the corpus writes.
  The CSP also declares a dependency on `Experience/AllowWindowsSpotlight` being `1`, so this value
  is meaningful only when Spotlight is not already fully off. If proposal 13 is applied first, this
  one becomes redundant. Do not ship both as independently recommended.
- **Risks:** low.
- **Sources:** Policy CSP - Experience > AllowThirdPartySuggestionsInWindowsSpotlight (registry
  mapping, dependency, default); `CloudContent.admx` / `.adml` on 26100; privacy.sexy.

### 15. `disable_spotlight_desktop` (interface, Medium)

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent`
- **Value name:** `DisableSpotlightCollectionOnDesktop`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `1`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 1607 and later, client only. User scope. Relevant on 26100 and 26200;
  the desktop Spotlight background option is a Windows 11 feature.
- **What it does:** the 26100 ADML says it "removes the Spotlight collection setting in
  Personalization, rendering the user unable to select and subsequently download daily images from
  Microsoft to desktop". So it removes both the option and the recurring image downloads.
- **Risks:** low. If the user is currently on a Spotlight desktop background, applying this leaves
  them on a static wallpaper.
- **Sources:** `CloudContent.admx` / `.adml` on 26100; Win11Debloat
  `Regfiles/Disable_Desktop_Spotlight.reg`; privacy.sexy.

### 16. `disable_search_history` (privacy, Medium)

- **Keys / values:**
  - `HKCU\Software\Microsoft\Windows\CurrentVersion\SearchSettings` `IsDeviceSearchHistoryEnabled`
    `REG_DWORD`, Disabled = `0`
  - `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` `DisableSearchHistory` `REG_DWORD`,
    Disabled = `1` (machine-wide reinforcement)
- **Stock default:** `IsDeviceSearchHistoryEnabled` is normally **present and `1`** on a stock
  profile, so the "Stock Default" option must write `1`, not `absent`. `DisableSearchHistory` is
  value-absent by default. See UNKNOWNS: the presence of `IsDeviceSearchHistoryEnabled` on a fresh
  26100 profile should be probed before the YAML is written.
- **Applicability:** Windows 10 1903 and later. Applies to 26100, 26200 and LTSC 2021.
- **Why it is not a duplicate:** the corpus has `disable_search_highlights`
  (`IsDynamicSearchBoxEnabled`) and `disable_web_search_start` (`DisableSearchBoxSuggestions`).
  Neither stops Windows recording what you searched for locally and replaying it as "recent searches".
- **Risks:** low, and the trade-off is honest: recent-search suggestions stop working. Reversible.
- **Sources:** Win11Debloat `Regfiles/Disable_Search_History.reg`; privacy.sexy (both the HKCU value
  and the HKLM policy, with the note "breaks recent suggestions"); Sophia Script.

### 17. `disable_cloud_content_search` (privacy, Medium)

- **Keys / values:**
  - `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\SearchSettings` `IsMSACloudSearchEnabled`
    `REG_DWORD`, Disabled = `0`
  - `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\SearchSettings` `IsAADCloudSearchEnabled`
    `REG_DWORD`, Disabled = `0`
  - `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` `AllowCloudSearch` `REG_DWORD`,
    Disabled = `0`
- **Stock default:** the two HKCU values are normally present. `AllowCloudSearch` is value-absent
  with a documented CSP default of `1`.
- **Applicability:** `AllowCloudSearch` is Windows 10 1709 and later, Pro / Enterprise / Education /
  IoT Enterprise, device scope. The HKCU pair is Windows 10 1903 and later. Applies across 26100,
  26200 and LTSC 2021.
- **What it does:** stops taskbar search from issuing queries against OneDrive, SharePoint and
  Outlook content tied to the signed-in Microsoft or Entra account. This is a network egress
  reduction, not a cosmetic change.
- **Risks:** low. Local file and app search is unaffected. Users who deliberately search their
  OneDrive from the taskbar will lose that.
- **Sources:** Policy CSP - Search > AllowCloudSearch (registry key name, default `1`, editions);
  privacy.sexy (both HKCU values, labelled "Disable personal cloud content search in taskbar");
  Sophia Script.

### 18. `disable_voice_activation` (privacy, Medium)

- **Keys / values:**
  - `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` `LetAppsActivateWithVoice` `REG_DWORD`,
    Force Deny = `2`
  - `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` `LetAppsActivateWithVoiceAboveLock`
    `REG_DWORD`, Force Deny = `2`
- **Options:** Blocked = `2` on both; User in control (Stock Default) = `absent` on both.
  (`0` = user in control, `1` = force allow, `2` = force deny.)
- **Stock default:** value-absent on both.
- **Applicability:** Windows 10 1809 and later, machine scope. Applies to 26100, 26200 and LTSC 2021.
- **Why it is not a duplicate:** the corpus has `disable_online_speech_recognition` (`HasAccepted`)
  and `LetAppsGetDiagnosticInfo` / `LetAppsRunInBackground` from the same AppPrivacy family. Neither
  covers wake-word activation, which is a separate always-listening surface with its own above-lock
  variant.
- **Risks:** medium-low. Any voice assistant the user actually wants (including third-party ones that
  register for voice activation) stops responding to its wake word. Push-to-talk and manual
  invocation still work. Reversible by deleting the values.
- **Sources:** `C:\Windows\PolicyDefinitions\AppPrivacy.admx` on 26100 (both policies, class Machine,
  key and valueName); Policy CSP - Privacy `LetAppsActivateWithVoice` family; privacy.sexy
  (`AgentActivationEnabled` / `AgentActivationOnLockScreenEnabled` per-user siblings under
  `HKCU\Software\Microsoft\Speech_OneCore\Settings\VoiceActivation\UserPreferenceForAllApps`, which
  are a reasonable additional effect).

### 19. `disable_windows_backup` (privacy, Medium)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync`
- **Value name:** `EnableWindowsBackup`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Not configured (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 and later, client only (`SUPPORTED_Windows_10_0_NOSERVER`). Machine
  scope. Applies to 26100, 26200 and LTSC 2021.
- **Honest framing required in the copy:** the 26100 ADML says "If you disable or do not configure
  this policy setting, windows backup will not take place." So writing `0` does not change the
  *documented* default behaviour; what it buys is a hard lock, so nothing (an OOBE flow, a Settings
  nudge, a servicing update) can turn periodic backup on later. That is a real but modest benefit and
  is why this sits at Medium rather than High. Do not overclaim it as "stops Windows backing up your
  files", because on an unmanaged consumer machine the Windows Backup app can still be run manually.
- **Why it is not a duplicate:** the corpus has `disable_settings_sync`
  (`DisableSettingSync` / `DisableSettingSyncUserOverride`), which governs roaming settings sync.
  `EnableWindowsBackup` governs the newer Windows Backup / cloud restore pipeline, a different
  feature under the same policy key.
- **Risks:** low, but the user gives up automatic cloud restore on a future reinstall. That trade-off
  belongs in the tweak copy.
- **Sources:** `C:\Windows\PolicyDefinitions\SettingSync.admx` on 26100 (class Machine, key,
  valueName, enabledValue `1` / disabledValue `0`); `en-US\SettingSync.adml` (help text quoted).

### 20. `disable_online_tips` (privacy, Medium)

- **Key:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer`
- **Value name:** `AllowOnlineTips`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Enabled (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 1709 (`SUPPORTED_Windows_10_0_RS3`) and later. Machine scope. Applies
  to 26100, 26200 and LTSC 2021.
- **What it does:** the 26100 ADML is precise: "If disabled, Settings will not contact Microsoft
  content services to retrieve tips and help content." A concrete network-egress reduction from the
  Settings app.
- **Why it is not a duplicate:** the corpus's `disable_tips_and_suggestions` targets the
  ContentDeliveryManager `SoftLandingEnabled` / `SubscribedContent-338389Enabled` slots, which are
  the *notification-style* tips. `AllowOnlineTips` is the Settings-app help-content fetch. Different
  surface, different traffic.
- **Risks:** low. Settings pages show local help text only.
- **Sources:** `C:\Windows\PolicyDefinitions\ControlPanel.admx` and `en-US\ControlPanel.adml` on
  26100.

### 21. `hide_settings_ai_page` (debloat, Medium)

- **Key:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer`
- **Value name:** `SettingsPageVisibility`
- **Type:** `REG_SZ`
- **Options:** Hidden = `hide:aicomponents`; Visible (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 1703 (`SUPPORTED_Windows_10_0_RS2`) and later, class Both. The
  `aicomponents` page identifier itself is a 24H2-era page, so gate `build >= 26100`.
- **Serious caveat that must be encoded in the schema:** this is a **single shared string value**
  with `hide:` / `showonly:` list semantics. Win11Debloat writes `hide:home` to the same value to
  suppress the Settings homepage, and WinUtil writes `hide:aicomponents`. Two tweaks that each own
  this value will silently clobber each other, and a snapshot-based revert will restore only one
  author's intent. Either ship exactly one tweak that owns `SettingsPageVisibility` and offers a
  dropdown of composed values (`hide:home`, `hide:aicomponents`, `hide:home;aicomponents`, absent), or
  do not ship it at all. Do not ship two independent tweaks against this value.
- **Risks:** medium, entirely because of the shared-value collision above. The visibility change
  itself is cosmetic and reversible. Note that hiding a page also blocks direct URI navigation to it,
  which redirects to the Settings front page.
- **Sources:** `C:\Windows\PolicyDefinitions\ControlPanel.admx` and `en-US\ControlPanel.adml` on
  26100 (policy `SettingsPageVisibility`, class Both, `text` element, help text describing the
  block-list semantics); WinUtil `tweaks.json` (`hide:aicomponents`); Win11Debloat
  `Regfiles/Disable_Settings_Home.reg` (`hide:home`).

### 22. `disable_nag_toasts` (debloat, Medium)

- **Keys / values** (each `REG_DWORD` named `Enabled`):
  - `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.Suggested`
  - `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Notifications\Settings\Windows.SystemToast.BackupReminder`
- **Options:** Silenced = `0` on both; Allowed (Stock Default) = `absent` on both
- **Stock default:** value-absent (keys may not exist until the first toast fires).
- **Applicability:** Windows 10 and later. User scope. Applies to 26100, 26200 and LTSC 2021.
- **Why it is not a duplicate:** the corpus's `disable_toast_notifications` sets `ToastEnabled` = 0,
  which is the sledgehammer: it silences **every** toast including ones the user wants (messages,
  alarms, Defender). These two per-source values silence only the Microsoft nags, keeping the rest of
  the notification system intact. That is the strictly better default recommendation and the two
  tweaks should be presented as alternatives.
- **Risks:** low. Windows Backup reminders and "Suggested" app promos stop; nothing else changes.
- **Sources:** Win11Debloat `Regfiles/Disable_Windows_Suggestions.reg` (writes both); Sophia Script
  (`Windows.ActionCenter.SmartOptOut` sibling under the same `Notifications\Settings` scheme);
  ElevenForum threads on per-app notification registry entries. The
  `Notifications\Settings\<AppId>\Enabled` scheme itself is Microsoft's documented per-app
  notification store.

### 23. `disable_snap_assist` (interface, Medium)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`
- **Value name:** `SnapAssist`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Enabled (Stock Default) = `1`
- **Stock default:** value **present** and `1`. This one is not value-absent, so the revert option
  must write `1`.
- **Applicability:** Windows 10 and later. User scope. Applies to 26100, 26200 and LTSC 2021.
- **Why it is not a duplicate:** the corpus has `disable_snap_flyout`
  (`EnableSnapAssistFlyout`), which is the layouts flyout that appears when you hover the maximise
  button. `SnapAssist` is the separate "now pick a window for the other half" screen that appears
  *after* you snap. Two different interruptions, two different values.
- **Risks:** low. Snapping still works; you just do not get prompted to fill the remaining space.
- **Sources:** Win11Debloat `Regfiles/Disable_Snap_Assist.reg`; Sophia Script; privacy.sexy; the value
  is long-standing and consistently documented.

### 24. `alt_tab_hide_browser_tabs` (interface, Medium)

- **Key:** `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced`
- **Value name:** `MultiTaskingAltTabFilter`
- **Type:** `REG_DWORD`
- **Options:** Windows only = `3`; 3 most recent tabs (Stock Default) = `absent`.
  (`0` = 20 tabs, `1` = 5 tabs, `2` = 3 tabs, `3` = windows only.)
- **Stock default:** value-absent; effective default is the 3-most-recent-tabs behaviour exposed in
  Settings > System > Multitasking.
- **Applicability:** Windows 11 21H2 and later. User scope. Not applicable to LTSC 2021.
- **Risks:** low, purely a preference. Worth noting in the copy that this only affects Microsoft Edge
  tabs, since Edge is the only browser that registers tabs with the shell's Alt+Tab.
- **Sources:** Win11Debloat `Regfiles/Hide_Tabs_In_Alt_Tab.reg` plus the sibling
  `Show_3_Tabs_In_Alt_Tab.reg` / `Show_5_Tabs_In_Alt_Tab.reg` / `Show_20_Tabs_In_Alt_Tab.reg` files,
  which together pin down the full enum; Sophia Script; ElevenForum tutorial. Three independent
  sources agreeing on the enum.

### 25. `disable_widgets_lock_screen` (interface, Medium, conflicted)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Dsh`
- **Value name:** `DisableWidgetsOnLockScreen`
- **Type:** `REG_DWORD`
- **Applicability:** the shipped 26100 `NewsAndInterests.admx` declares
  `SUPPORTED_Windows_11_0_22H2_NOSERVER`. Machine scope. Not applicable to LTSC 2021.
- **The conflict, stated plainly.** Microsoft's two primary sources disagree on both the value
  semantics and the supported OS, and this must be resolved on a real machine before the tweak ships:
  - **Policy CSP - NewsAndInterests > DisableWidgetsOnLockScreen** says the format default is `0`,
    and lists allowed values as `0 (Default) = Enabled`, `1 = Disabled`. It also lists Applicable OS
    as *Windows Insider Preview* only.
  - **`C:\Windows\PolicyDefinitions\NewsAndInterests.admx` on 26100** declares the inverse:
    `<enabledValue><decimal value="0"/></enabledValue>` and
    `<disabledValue><decimal value="1"/></disabledValue>`. Since the policy is *named*
    "Disable Widgets On Lock Screen", enabling it in gpedit writes `0`. The same inversion is present
    on the sibling `DisableWidgetsBoard`.
  So gpedit and the CSP would write opposite values for the same intent. Until a probe on 26100 or
  26200 settles which value actually suppresses lock-screen widgets, this cannot ship. See UNKNOWNS.
- **Why it is still worth listing:** the brief explicitly asks about taskbar widgets on the lock
  screen, and this is the only first-party control for it. The corpus's existing `disable_widgets`
  (`AllowNewsAndInterests` = 0) governs the Widgets board reached from the taskbar; whether it also
  suppresses the lock-screen panel on 24H2 and later is itself unverified.
- **Risks:** medium until the semantics are pinned down, because writing the wrong value would
  silently *enable* a surface the user asked to remove, which violates the did-it-work contract.
- **Sources:** Policy CSP - NewsAndInterests > DisableWidgetsOnLockScreen and > DisableWidgetsBoard;
  `NewsAndInterests.admx` and `en-US\NewsAndInterests.adml` on 26100.

### 26. `taskbar_last_active_click` (interface, Low)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`
- **Value name:** `LastActiveClick`
- **Type:** `REG_DWORD`
- **Options:** Focus last active = `1`; Show flyout (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 and later. User scope. Applies to 26100, 26200 and LTSC 2021.
- **Effect:** clicking a taskbar icon for an app with several windows focuses the most recent window
  and cycles on repeat clicks, instead of opening the thumbnail flyout. Hovering still shows
  thumbnails.
- **Risks:** low, purely a preference.
- **Sources:** Win11Debloat `Regfiles/Enable_Last_Active_Click.reg` (with an unusually clear inline
  explanation); Sophia Script; long-standing ElevenForum and TenForums tutorials.

### 27. `explorer_expand_to_current_folder` (interface, Low)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`
- **Value name:** `NavPaneExpandToCurrentFolder`
- **Type:** `REG_DWORD`
- **Options:** Expand = `1`; Do not expand (Stock Default) = `absent`
- **Stock default:** value-absent (effective `0`).
- **Applicability:** Windows 10 and later. User scope. Applies to 26100, 26200 and LTSC 2021.
- **Risks:** low. On deep trees the navigation pane can get long.
- **Sources:** Sophia Script; privacy.sexy adjacent Explorer entries; the Explorer Folder Options UI
  exposes this as "Expand to open folder", which makes the mapping self-verifying.

### 28. `explorer_restore_folders_at_logon` (interface, Low)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`
- **Value name:** `PersistBrowsers`
- **Type:** `REG_DWORD`
- **Options:** Restore = `1`; Do not restore (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 and later. User scope. Applies to 26100, 26200 and LTSC 2021.
- **Risks:** low. Corresponds to the "Restore previous folder windows at logon" checkbox in Folder
  Options, so it is trivially verifiable in the UI.
- **Sources:** Sophia Script; Folder Options UI mapping; widely documented.

### 29. `taskbar_full_date_time` (interface, Low)

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `TurnOffAbbreviatedDateTimeFormat`
- **Type:** `REG_DWORD`
- **Options:** Full format = `1`; Abbreviated (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** `SUPPORTED_Windows_11_0_22H2`. User scope. Not applicable to LTSC 2021.
- **Why it is not a duplicate:** the corpus has `seconds_in_tray_clock`
  (`ShowSecondsInSystemClock`), a different aspect of the same clock.
- **Risks:** none beyond taskbar width.
- **Sources:** `C:\Windows\PolicyDefinitions\Taskbar.admx` and `en-US\Taskbar.adml` on 26100.
- **Companion at the same tier:** `AlwaysShowNotificationIcon` under the same key ("Show notification
  bell icon", `SUPPORTED_Windows_11_0_22H2`, user scope) is equally well documented and equally low
  value; bundle or skip both together.

### 30. `hide_recently_added_apps` (interface, Low)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` and
  `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `HideRecentlyAddedApps`
- **Type:** `REG_DWORD`
- **Options:** Hidden = `1`; Shown (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 1803 and later, class Both. Applies to 26100, 26200 and LTSC 2021.
- **Note:** largely subsumed by proposal 1 on Windows 11, where the Recommended section is the only
  place "recently added" surfaces. Worth shipping only if proposal 1 is not shipped, or for LTSC
  2021 where the Start layout is the Windows 10 one.
- **Risks:** none.
- **Sources:** `C:\Windows\PolicyDefinitions\StartMenu.admx` on 26100; Policy CSP - Start >
  HideRecentlyAddedApps; Sophia Script.

### 31. `disable_new_app_alert` (interface, Low)

- **Key:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `NoNewAppAlert`
- **Type:** `REG_DWORD`
- **Options:** Suppressed = `1`; Shown (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 8 and later. Machine scope. Applies to 26100, 26200 and LTSC 2021.
- **Effect:** suppresses the "You have new apps that can open this type of file" toast that fires
  after installs.
- **Risks:** low, but note it also hides the legitimate signal that something changed your file
  associations, which has a mild security-awareness cost. Say so in the copy.
- **Sources:** `C:\Windows\PolicyDefinitions\WindowsExplorer.admx` and `en-US\WindowsExplorer.adml` on
  26100 ("Do not show the 'new application installed' notification").

### 32. `disable_notification_center` (interface, Low)

- **Key:** `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer`
- **Value name:** `DisableNotificationCenter`
- **Type:** `REG_DWORD`
- **Options:** Removed = `1`; Present (Stock Default) = `absent`
- **Stock default:** value-absent.
- **Applicability:** Windows 10 and later, class Both. Applies to 26100, 26200 and LTSC 2021.
- **Why Low and not higher:** it removes the Notification Center *and* the calendar flyout from the
  tray, which is a large, blunt change most users will not want. It is included because it is the
  only documented way to remove that surface entirely, and the corpus currently offers nothing
  between "all toasts off" and "leave it alone".
- **Risks:** medium for the user, low technically. Notifications still fire as toasts; there is just
  no history and no calendar. Reversible.
- **Sources:** `C:\Windows\PolicyDefinitions\Taskbar.admx` and `en-US\Taskbar.adml` on 26100 ("Remove
  Notifications and Action Center"); WinUtil `tweaks.json`; Sophia Script.

### 33. Extend `disable_suggested_content_settings` (privacy, Low)

- **Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`
- **Value name:** `SubscribedContent-353698Enabled`
- **Type:** `REG_DWORD`
- **Options:** Disabled = `0`; Enabled (Stock Default) = `absent`
- **This is a modification, not a new tweak.** The existing `disable_suggested_content_settings`
  covers `SubscribedContent-338393Enabled`, `-353694Enabled` and `-353696Enabled`. Win11Debloat
  writes a fourth slot, `-353698Enabled`, alongside those three. Adding it closes the set.
- **Risks:** none.
- **Sources:** Win11Debloat `Regfiles/Disable_Windows_Suggestions.reg`; the ContentDeliveryManager
  slot scheme is corroborated by privacy.sexy and Sophia Script.

---

## Rejected, with reasons

Recorded so these are not rediscovered and re-litigated.

| Candidate | Source it came from | Reason for rejection |
|---|---|---|
| `AllowCommercialDataPipeline` | privacy.sexy, DataCollection.admx | The shipped 26100 ADML explain text states: "This policy is deprecated and will only work on Windows 10 version 1809. Setting this policy will have no effect for other supported versions of Windows." Dead on the target platform. |
| `AllowDesktopAnalyticsProcessing` | privacy.sexy | Same deprecation notice verbatim in the 26100 ADML. |
| `AllowUpdateComplianceProcessing` | privacy.sexy | Same deprecation notice verbatim in the 26100 ADML. |
| `AllowWUfBCloudProcessing` | privacy.sexy | Same deprecation notice verbatim in the 26100 ADML. |
| `TurnOffSavingSnapshots` (WindowsAI) | Win11Debloat only | T3. Absent from `WindowsCopilot.admx` on 26100 and absent from the WindowsAI Policy CSP. Single source. `DisableAIDataAnalysis`, which the corpus already ships, is the documented control for the same intent. |
| `DisableGenerativeErase`, `DisableRemoveBackground` (Paint) | Win11Debloat only | T3. Not in the 26100 `WindowsCopilot.admx` and not in the Policy CSP, unlike the three sibling Paint values that are. Ship only the three confirmed ones (proposal 7). |
| `AllowCortana`, `CortanaConsent`, `CanCortanaBeEnabled`, `CortanaInAmbientMode`, `AllowCortanaAboveLock`, `HistoryViewEnabled`, `DeviceHistoryEnabled`, `VoiceShortcut` | privacy.sexy, Win11Debloat, Sophia | Cortana as a standalone app was retired in 2023 and is not in the 24H2/25H2 inbox app set. `_rescope-24h2.md` already marks the corpus's Cortana tweaks DELETE. Adding more Cortana surface area contradicts that decision. |
| `ShowCopilotButton` | privacy.sexy, Win11Debloat | Duplicate of the corpus's `disable_copilot_taskbar`, which `_rescope-24h2.md` already flags NEEDS GATE CHANGE. Adding a second value against the same button does not fix the underlying deprecation of `TurnOffWindowsCopilot`. |
| `TaskbarDa` | privacy.sexy, Sophia, Win11Debloat | Duplicate. The corpus's `disable_widgets` sets `AllowNewsAndInterests` = 0, the documented machine-wide control, which removes the taskbar entry point as well. A per-user cosmetic value adds nothing and can drift out of sync with the policy. |
| `DisableWidgetsBoard` | NewsAndInterests.admx | Duplicate of `AllowNewsAndInterests` = 0, and carries the same ADMX-versus-CSP value inversion described in proposal 25 without any compensating benefit. |
| `NoStartMenuMorePrograms` ("Disable Start All Apps") | Win11Debloat, Sophia | Removes the all-apps list, a core navigation surface. That is a lockdown control for kiosk imaging, not a user benefit. Fails the "real user-visible benefit" test. |
| `LetAppsAccessCamera`, `LetAppsAccessMicrophone`, and the rest of the `LetAppsAccess*` force-deny family | AppPrivacy.admx, privacy.sexy | Force-deny at machine scope breaks working apps in ways that are hard for a user to diagnose, and Settings > Privacy already exposes per-app control with a much better UX. High breakage, low marginal privacy gain over the shipped UI. `LetAppsActivateWithVoice` is the exception and is proposed at #18. |
| `NoThumbnailCache` / `DisableThumbnails` | WindowsExplorer.admx, Thumbnails.admx | Explicitly in the brief's myth category. Disabling the thumbnail cache makes Explorer slower, not faster, and saves a trivial amount of disk. |
| Icon cache resizing (`Max Cached Icons`) | Optimizer-adjacent guidance | Named myth in the brief. No measurable benefit on modern builds. |
| Any "disable telemetry for FPS" framing | common in community lists | Named myth in the brief. Telemetry changes are a privacy action, not a performance action. The corpus's existing telemetry tweaks are correctly framed already and must stay that way. |
| `SystemRestorePointCreationFrequency` = 0 | WinUtil | Not in these domains, and it weakens recoverability. |
| `PreventDeviceEncryption` | Win11Debloat | Security domain, not privacy/debloat/interface, and it is a meaningful security downgrade on 24H2 where automatic device encryption is a default protection. |
| `NoLockScreen` | WinUtil | Edition-limited and unreliable on Windows 11 client; Microsoft has repeatedly changed whether it is honoured. Cannot meet the "works on 26100 or later" bar with confidence. |
| `EnableOrganizationalMessages` = 0 | CloudContent.admx | The 26100 ADML states "By default, this policy is disabled." There is nothing to turn off on an unmanaged device. |
| `SetDenyAppListForRecall`, `SetDenyUriListForRecall`, `SetMaximumStorageSpaceForRecallSnapshots`, `SetMaximumStorageDurationForRecallSnapshots` | WindowsCopilot.admx | Only meaningful if Recall is left running. The corpus's stance (`disable_recall_snapshots`, `remove_recall_component`) is to turn Recall off entirely, which makes these inert. Also Enterprise and Education SKUs only. |
| `MaxTelemetryAllowed` | Sophia Script | T3. Undocumented by Microsoft, no ADMX or CSP backing on 26100. |
| `AicEnabled` (App Install Control) | privacy.sexy | Turning it off weakens SmartScreen's app-reputation gate. Security regression dressed as a debloat. |
| `RealTimeIsUniversal` | WinUtil | Not in these domains and it changes RTC interpretation, which can silently break time on dual-boot and VM setups. |
| `Search.admx` policies generally (`ConnectedSearchUseWeb`, `DisableWebSearch`) | privacy.sexy | privacy.sexy's own comment marks `DisableWebSearch` "Obsolete since Windows 10". The shipped 26100 `Search.admx` contains zero policy elements, so the whole legacy Windows Search policy family cannot be confirmed on the target platform from a Microsoft source. `DisableSearchBoxSuggestions`, which the corpus already ships, remains the working control. |
| `ExtendedUIHoverTime`-style values generally | assorted | `_rescope-24h2.md` already marks the corpus's own `taskbar_hover_time` OBSOLETE on 24H2. Do not add more values in that family. |
| Brave, Visual Studio, Nvidia and Office telemetry values | WinUtil, privacy.sexy | Third-party application settings, outside a Windows-tweak corpus's remit and unverifiable against a Microsoft source. |
| `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\*` writes generally | WinUtil, Sophia | This is the MDM policy cache, not an authoring surface. Writing it directly can be overwritten by the policy engine at any refresh, which breaks the reversibility guarantee. Acceptable only as a documented fallback effect (see proposal 1), never as a primary mechanism. |

---

## UNKNOWNS

1. **`DisableWidgetsOnLockScreen` value polarity (blocks proposal 25).** The Policy CSP says `1` =
   Disabled, the 26100 ADMX says enabling the policy writes `0`. These cannot both be right. Needs a
   read-modify-observe probe on a real 26100 or 26200 machine with lock-screen widgets enabled. The
   same question applies to `DisableWidgetsBoard`, which is rejected anyway.
2. **`DisableWidgetsOnLockScreen` applicability.** The CSP lists Applicable OS as *Windows Insider
   Preview* while the shipped 26100 ADMX says 22H2 and later. Unresolved.
3. **`IsDeviceSearchHistoryEnabled` stock state (affects proposal 16's revert option).** Believed to
   be present and `1` on a fresh profile rather than value-absent, but not confirmed on 26100. The
   revert option must match reality or the snapshot restore will write a value that was never there.
4. **`IsMSACloudSearchEnabled` / `IsAADCloudSearchEnabled` stock state (proposal 17).** Same question.
5. **`WSAIFabricSvc` stock start type on 26200 (proposal 5).** Win11Debloat writes `Start` = 3
   (Manual); whether the shipped default is `2` (Automatic) or `3` needs confirmation on a real 25H2
   image before the "Stock Default" option can be written correctly. The service does not exist on
   26100, so it cannot be probed locally.
6. **`HideRecommendedSection` on Pro (proposal 1).** The CSP claims Pro support; the shipped ADMX
   `supportedOn` says Windows 11 SE. Needs an on-device check on Pro to decide whether the
   PolicyManager fallback effect is required.
7. **Whether the corpus's existing `AllowNewsAndInterests` = 0 already suppresses lock-screen
   widgets on 24H2.** If it does, proposal 25 is redundant regardless of how the polarity question
   resolves.
8. **25H2 agent workspace.** Microsoft's 25H2 "what's new for IT pros" page documents Click to Do,
   Recall and semantic search as the Copilot+ features, and describes no separate "agent workspace"
   policy surface. No ADMX, CSP or registry control for an agent workspace could be confirmed from a
   Microsoft source. Either it has not shipped a management surface yet, or it is gated behind a
   servicing update not present on 26100. Re-check against `WindowsAI` CSP after the next 25H2
   servicing wave rather than guessing a key now.
9. **Edge "Copilot Mode".** `EdgeCopilotEnabled`, `CopilotPageContext` and
   `Microsoft365CopilotChatIconEnabled` are all documented, but no policy named for Copilot Mode
   specifically was found. Proposal 12 covers the confirmed values; a Copilot-Mode-specific switch
   remains unlocated.
10. **`Windows.SystemToast.BackupReminder` key existence before first fire (proposal 22).** Per-app
    notification keys are typically created lazily. If the key is absent, the tweak must create it,
    and the status probe must treat "key absent" as the stock state rather than as an error.
