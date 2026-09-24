# Tweak wiki

The complete reference for every tweak MagicX Toolbox ships. Each entry is meant to be enough on its own: the exact change the tweak makes, what that does to the machine, the benefits and drawbacks, when it applies, how it reverts, how it was validated, and the sources behind it. If you find yourself searching the web to understand a tweak, its entry is missing something; fix the entry.

Tweaks are authored as YAML in `src-tauri/tweaks/`, one file per category. The YAML is deliberately lean, carrying what the app needs at runtime. This wiki carries everything else. How to write a tweak is covered in [`../TWEAK_AUTHORING.md`](../TWEAK_AUTHORING.md), and how the engine runs one in [`../TWEAK_SYSTEM.md`](../TWEAK_SYSTEM.md).

**Supported platform:** Windows 11 24H2 (build 26100) and newer, including 25H2 (26200), primary. Windows 10 IoT Enterprise LTSC 2021 (build 19044), secondary. Windows 10 22H2 consumer and Windows 11 21H2 through 23H2 are out of scope.

## Pages

| Page | Tweaks |
|---|---|
| [AI & Copilot tweaks](ai.md) | 10 |
| [Debloat & Consumer tweaks](debloat.md) | 27 |
| [Interface & Explorer tweaks](interface.md) | 42 |
| [Network & Power tweaks](network.md) | 14 |
| [Performance & Gaming tweaks](performance.md) | 24 |
| [Privacy & Telemetry tweaks](privacy.md) | 34 |
| [Security Hardening tweaks](security.md) | 62 |
| [Services & Scheduled Tasks tweaks](services.md) | 29 |
| [Windows Update tweaks](windows_update.md) | 14 |

256 tweaks in total.

## How to read an entry

Every entry has the same shape.

- **Header line**: the tweak id, how the UI presents it (a switch for one or two authored options, a dropdown for three or more; System Default joins either while it is the live state), its risk level, the elevation it needs (`none` for tweaks the YAML marks `elevation: user`, `admin`, or `ti` for TrustedInstaller; a tweak may escalate individual effects to `ti`), whether it needs a reboot, the Windows builds it is gated to, and whether it is reversible.
- **What it changes**: every effect the tweak owns (registry value, service, scheduled task, firewall rule, hosts entry, app package, or script action) with its exact target, then a table of what each option writes to each effect. `absent` means the value is deleted.
- **System Default** is never authored. It is the state the app reports when the live machine matches none of the tweak's options, and selecting it restores the snapshot the app took before the tweak was first applied. It is not always the same as the stock Windows state; entries say so when they differ.
- **How it works, Benefits, Drawbacks, Applies to**: the mechanism and its consequences, including edition limits and when the change takes effect.
- **Interactions**: other tweaks and shared settings that touch the same or related surfaces.
- **Validation**: the verdict, the kind of confidence behind it, the reasoning, and what testing it has had.
- **Recommendation**: a position on who should apply it.
- **Sources**: every source, with what it establishes.

## How the tweaks were validated

### The July 2026 research

Every tweak in the corpus as of July 2026 was re-validated against primary sources in four passes: a first validation round; an adversarial round in which a second agent tried to refute the entries the first round had called clean; a gap hunt for controls the corpus should have had and did not; and a rewrite of every tweak's user-facing copy into one template. Its findings were applied to the YAML, and this wiki is built from the result.

The adversarial round earned its cost. It attacked the 87 entries the first round had called clean and flipped 27 of them, a 31 percent miss rate on work that had already been reviewed once. The gap hunt proposed 71 candidate controls; after the same adversarial treatment 41 were confirmed, 22 confirmed with corrections and 8 rejected, and 62 shipped.

### The evidence standard

A mechanism counts as verified when it is either:

- **Microsoft-documented**: a Microsoft Learn page, a KB article, a Policy CSP reference, or the shipped ADMX templates in `C:\Windows\PolicyDefinitions` (Microsoft's own artifact, identical on every 26100 install, and the most reliable ADMX source available); or
- **Community-corroborated**: three or more genuinely independent community sources agreeing on the exact key, value name, type and semantics. Independence is enforced: three sites that copied one blog count as one source, and one author's tutorials on two forums count as one origin.

An earlier Microsoft-only standard was abandoned because it was wrong for this domain. Microsoft documents what it wants administrators to configure, not everything the OS reads, so "no Microsoft page" says nothing about whether a working tweak works. Under the revised rule every UNVERIFIED verdict resolved. Several mechanisms turned out to be documented after all, just not where the first pass looked: `DisableSearchBoxSuggestions` is defined in the shipped `WindowsExplorer.admx`, `PnPCapabilities = 24` is given verbatim in KB 2740020, and the MMCSS throttling cap is documented in KB 948066.

A third kind of evidence applies to tweaks Microsoft does not sanction at all, such as blocking the Windows Update pipeline: **empirically tested**, meaning the tweak was applied and reverted on a real machine by the app and every effect was read back. Microsoft not endorsing a change is never a reason to downgrade a verdict.

**Revert values are held to a higher bar.** A wrong mechanism makes a tweak inert, and the user notices. A wrong revert does damage at the exact moment the user is trying to be careful, and nothing in the interface tells them. So community agreement is not sufficient evidence for what a tweak restores; that needs documentation. Two directions of error are guarded against: reverting by writing a literal where Windows ships the value absent (which can fabricate a consent or disable a protection the OS enables by default), and sweeping to `absent` a value Windows genuinely ships seeded (`NetworkThrottlingIndex` ships as 10 and `SystemResponsiveness` as 20, both documented, and both keep their literals).

### Verdicts

| Verdict | Meaning |
|---|---|
| VERIFIED | The mechanism, values and semantics held up as authored. |
| VERIFIED-WITH-CORRECTION | The control is real, but something about it had to be corrected (a value name, hive, type, option shape, revert value, gate or risk level). The corrections are applied; the entry describes the corrected tweak. |
| INCORRECT | The tweak as researched did not work as described. Where the research's corrected form now ships, the entry says "corrected form ships" and describes what ships; the verdict stays as researched so the record is honest. |
| DISPUTED | Sources conflict on some part of the behaviour; the entry says which part and why the tweak still ships. |
| NOT RESEARCHED | No independent research yet; the entry says so. |

The per-category documents did not apply VERIFIED-WITH-CORRECTION identically: the network researcher reserved it for mechanism defects and left copy-only fixes at VERIFIED, while the others counted copy fixes as corrections. No correction was lost either way, but verdict counts are not strictly comparable across categories.

### Internal research citations

Some entries cite the July 2026 research files by name (`_policy-hive-audit.md`, `_harmful-revert.md`, `_verify-gaps-b-medlow.md` and others under `docs/superpowers/research/validation/`). Everything still true in them is folded into this wiki, and the files are no longer in the tree. To read one, use `git show ddbc355:docs/superpowers/research/validation/<file>`.

### Confidence of individual findings

Findings carry one of two levels. Those that went through the adversarial round (the verification pass, the policy-hive audit and all gap-hunt candidates) were independently attacked by an agent prompted to refute rather than confirm. Everything else is a single-pass research claim: sourced, but not adversarially checked. Entries say which applies where it matters.

## What belongs in the corpus

**The product exists to give the user control over their own machine.** Whether Microsoft ships a setting on or off is not an inclusion criterion. The single test is:

> Does a real control exist that changes something on Windows 11 24H2 or newer?

If yes, the tweak belongs, even when Windows already ships the setting the way the tweak sets it. If no, it is excluded, because there is nothing to control. "Windows ships this off" means a control exists and is currently off, so it is included. "Windows removed this" means no control exists, so it is excluded. That is why features Windows removed at or below the support floor (the Cortana button, Meet Now, the Chat button and others) appear only in the "Considered and not shipped" sections.

The default state still matters in exactly two places, both about correctness rather than inclusion: the revert value (above), and honest description. If applying a tweak changes nothing visible on a stock machine, its entry says so under Drawbacks rather than leaving the user hunting for a difference.

## Corpus-wide rules and decisions

**One address, one owner.** The engine enforces corpus-wide that each registry value, service, task or other address is owned by exactly one tweak. Where a bundle tweak and a dedicated single-purpose tweak both wanted an address, the dedicated tweak keeps it and the bundle drops it:

| Address | Bundle that gave it up | Owner |
|---|---|---|
| `MinAnimate`, `TaskbarAnimations` | `performance:optimize_visual_effects` | `interface:disable_ui_animations` |
| service `WMPNetworkSvc` | `services:disable_ssdp_upnp` | `services:disable_wmp_network_sharing` |
| task `\Microsoft\Windows\Autochk\Proxy` | `privacy:disable_ceip_tasks` | `services:task_autochk_proxy` |

Addresses several tweaks legitimately need with the same value are **shared settings** instead, claimed with a reference count: written when the first tweak that needs it applies, and restored to the captured original only when the last one reverts. The corpus has one, the Defender ASR master switch, documented on the [security page](security.md).

**A deliberate unresolved overlap.** `EnableVirtualizationBasedSecurity` is a Credential Guard prerequisite, but `performance:disable_vbs_hvci` owns it and wants the opposite value, so no shared setting can express it. `security:enable_credential_guard` warns about this in its entry instead.

**Look-alikes that are not duplicates.** The Start-menu suggestion tweaks look mergeable by name but are not: `SubscribedContent-338388Enabled` is the Windows 10 surface, while Windows 11 uses `Start_IrisRecommendations` backed by the `HideRecommendedSection` policy. Similarly, `alt_tab_hide_browser_tabs` shows why value names alone mislead: the same value name exists at two keys with different enum bases, the policy key needing `4` where `Explorer\Advanced` needs `3`.

## Open questions

1. **A clean 24H2 image baseline is the highest-value missing input.** Reading the offline hives from a clean install ISO's `install.wim` would settle, in one pass: the stock defaults behind roughly a third of the interface category, whether several ContentDeliveryManager values ship absent or as literals (affecting six privacy and debloat tweaks), the SysMain and Delivery Optimization defaults, and the task states behind `task_disk_diagnostic_datacollector` and `task_maps_update`.
2. **`disable_people_bar` and `disable_news_interests` were dropped pending a re-check** against a real LTSC 2021 image. The reasoning does not depend on that check (People and News are both on the LTSC-excluded app list, and the User Choice Protection Driver has reverted writes to the `Feeds` key from non-allowlisted processes since March 2024), but the check has not been done.
3. **Whether `Microsoft.GetHelp`, `Microsoft.YourPhone` and `Microsoft.Getstarted` are preinstalled on a stock 24H2 image.** Absence from Microsoft's removable-app list may mean either "not preinstalled" or "system component", and those need different handling.

**Source availability.** `admx.help` returned HTTP 522 throughout the research and `getadmx.com` is now a squatted gambling domain; neither is cited. `elevenforum.com` and `tenforums.com` return HTTP 403 to automated fetchers and were read through the Wayback Machine.

## Keeping this wiki current

When a tweak's behaviour changes (an effect, an option, a value, a gate, the risk level), update its entry in the same commit. When a tweak is added, add its entry and its row in the index below; when one is removed, move its entry to its page's "Considered and not shipped" section with the reason.

## Every tweak

| Tweak | Id | Category | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|---|
| [Disable Windows Recall snapshots](ai.md#disable-windows-recall-snapshots) | `disable_recall_snapshots` | AI & Copilot tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Recall feature component](ai.md#recall-feature-component) | `remove_recall_component` | AI & Copilot tweaks | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable the Click to Do overlay](ai.md#disable-the-click-to-do-overlay) | `disable_click_to_do` | AI & Copilot tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the Copilot app](ai.md#remove-the-copilot-app) | `remove_copilot_app` | AI & Copilot tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable the Recall optional feature](ai.md#disable-the-recall-optional-feature) | `remove_recall_feature` | AI & Copilot tweaks | Switch | medium | admin | yes | INCORRECT (corrected form ships) |
| [Hide the Copilot taskbar button](ai.md#hide-the-copilot-taskbar-button) | `disable_copilot_taskbar` | AI & Copilot tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Notepad AI features](ai.md#disable-notepad-ai-features) | `disable_notepad_ai` | AI & Copilot tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Paint AI features](ai.md#disable-paint-ai-features) | `disable_paint_ai` | AI & Copilot tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off Edge AI features](ai.md#turn-off-edge-ai-features) | `disable_edge_ai_features` | AI & Copilot tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Set the AI Fabric service to Manual start](ai.md#set-the-ai-fabric-service-to-manual-start) | `disable_ai_fabric_service` | AI & Copilot tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off Start menu app promotions](debloat.md#turn-off-start-menu-app-promotions) | `disable_start_suggestions` | Debloat & Consumer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off auto-installed sponsored apps](debloat.md#turn-off-auto-installed-sponsored-apps) | `disable_auto_install_sponsored_apps` | Debloat & Consumer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off the post-update welcome experience](debloat.md#turn-off-the-post-update-welcome-experience) | `disable_welcome_experience` | Debloat & Consumer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off the 'Get even more out of Windows' nag](debloat.md#turn-off-the-get-even-more-out-of-windows-nag) | `disable_scoobe_nag` | Debloat & Consumer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off File Explorer sync-provider ads](debloat.md#turn-off-file-explorer-sync-provider-ads) | `disable_explorer_sync_ads` | Debloat & Consumer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off web results in Start search](debloat.md#turn-off-web-results-in-start-search) | `disable_web_search_start` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off Widgets](debloat.md#turn-off-widgets) | `disable_widgets` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Turn off Microsoft account nags in Start](debloat.md#turn-off-microsoft-account-nags-in-start) | `disable_account_notifications` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off account upsell cards in Settings](debloat.md#turn-off-account-upsell-cards-in-settings) | `disable_settings_account_ads` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off all Windows Spotlight features](debloat.md#turn-off-all-windows-spotlight-features) | `disable_windows_spotlight_all` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off Spotlight desktop wallpaper](debloat.md#turn-off-spotlight-desktop-wallpaper) | `disable_spotlight_desktop` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Silence suggested and backup reminder toasts](debloat.md#silence-suggested-and-backup-reminder-toasts) | `disable_nag_toasts` | Debloat & Consumer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off the Edge first-run experience](debloat.md#turn-off-the-edge-first-run-experience) | `disable_edge_first_run` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off Edge startup boost](debloat.md#turn-off-edge-startup-boost) | `disable_edge_startup_boost` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off the Edge sidebar and Collections](debloat.md#turn-off-the-edge-sidebar-and-collections) | `disable_edge_sidebar` | Debloat & Consumer tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the Microsoft Teams app](debloat.md#remove-the-microsoft-teams-app) | `remove_teams_consumer_app` | Debloat & Consumer tweaks | Switch | low | admin | no | INCORRECT (corrected form ships) |
| [Remove Clipchamp](debloat.md#remove-clipchamp) | `remove_clipchamp` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Quick Assist](debloat.md#remove-quick-assist) | `remove_quick_assist` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Bing News and Weather](debloat.md#remove-bing-news-and-weather) | `remove_bing_news_weather` | Debloat & Consumer tweaks | Switch | low | admin | no | INCORRECT (corrected form ships) |
| [Remove Solitaire Collection](debloat.md#remove-solitaire-collection) | `remove_solitaire` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the Get Help app](debloat.md#remove-the-get-help-app) | `remove_get_help` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Tips (Get Started)](debloat.md#remove-tips-get-started) | `remove_getstarted_tips` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Feedback Hub](debloat.md#remove-feedback-hub) | `remove_feedback_hub` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Phone Link](debloat.md#remove-phone-link) | `remove_phone_link` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove the New Outlook app](debloat.md#remove-the-new-outlook-app) | `remove_outlook_new` | Debloat & Consumer tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove Xbox Game Bar](debloat.md#remove-xbox-game-bar) | `remove_xbox_game_bar` | Debloat & Consumer tweaks | Switch | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove OneDrive](debloat.md#remove-onedrive) | `remove_onedrive` | Debloat & Consumer tweaks | Switch | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn on dark mode](interface.md#turn-on-dark-mode) | `enable_dark_mode` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off transparency effects](interface.md#turn-off-transparency-effects) | `disable_transparency` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off window animations](interface.md#turn-off-window-animations) | `disable_ui_animations` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable Aero Shake](interface.md#disable-aero-shake) | `disable_aero_shake` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Show file name extensions](interface.md#show-file-name-extensions) | `show_file_extensions` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Show hidden files and folders](interface.md#show-hidden-files-and-folders) | `show_hidden_files` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Open File Explorer to This PC](interface.md#open-file-explorer-to-this-pc) | `open_explorer_to_this_pc` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Show the full path in the Explorer title](interface.md#show-the-full-path-in-the-explorer-title) | `explorer_full_path_title` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hide recent files and folders in Explorer](interface.md#hide-recent-files-and-folders-in-explorer) | `disable_recent_files` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | INCORRECT (corrected form ships) |
| [Turn off recent items tracking](interface.md#turn-off-recent-items-tracking) | `disable_start_recent_items` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hide the Task View button](interface.md#hide-the-task-view-button) | `disable_task_view_button` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Show seconds in the tray clock](interface.md#show-seconds-in-the-tray-clock) | `seconds_in_tray_clock` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off search highlights](interface.md#turn-off-search-highlights) | `disable_search_highlights` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Taskbar search style](interface.md#taskbar-search-style) | `taskbar_search_mode` | Interface & Explorer tweaks | Dropdown (4 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Ungroup taskbar buttons](interface.md#ungroup-taskbar-buttons) | `taskbar_ungroup_labels` | Interface & Explorer tweaks | Dropdown (3 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Show taskbar thumbnails instantly](interface.md#show-taskbar-thumbnails-instantly) | `taskbar_hover_time` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | DISPUTED |
| [Left-align the taskbar](interface.md#left-align-the-taskbar) | `taskbar_alignment_left` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off the snap layouts hover flyout](interface.md#turn-off-the-snap-layouts-hover-flyout) | `disable_snap_flyout` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn on Explorer compact view](interface.md#turn-on-explorer-compact-view) | `explorer_compact_view` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off promoted recommendations in Start](interface.md#turn-off-promoted-recommendations-in-start) | `disable_start_recommendations` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Add End task to the taskbar menu](interface.md#add-end-task-to-the-taskbar-menu) | `enable_end_task_taskbar` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Hide Gallery in Explorer](interface.md#hide-gallery-in-explorer) | `remove_gallery_nav_pane` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Hide Home in Explorer](interface.md#hide-home-in-explorer) | `remove_home_nav_pane` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Hide OneDrive in Explorer](interface.md#hide-onedrive-in-explorer) | `remove_onedrive_nav_pane` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off toast notifications](interface.md#turn-off-toast-notifications) | `disable_toast_notifications` | Interface & Explorer tweaks | Switch (2 options) | medium | none | no | VERIFIED |
| [Show verbose logon messages](interface.md#show-verbose-logon-messages) | `verbose_logon_messages` | Interface & Explorer tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Silence the startup sound](interface.md#silence-the-startup-sound) | `disable_startup_sound` | Interface & Explorer tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off accessibility shortcut prompts](interface.md#turn-off-accessibility-shortcut-prompts) | `disable_accessibility_key_prompts` | Interface & Explorer tweaks | Switch | low | none | no | VERIFIED-WITH-CORRECTION |
| [Restore the classic context menu](interface.md#restore-the-classic-context-menu) | `classic_context_menu_win11` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn NumLock on at startup](interface.md#turn-numlock-on-at-startup) | `numlock_on_startup` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hide the Recommended section in Start](interface.md#hide-the-recommended-section-in-start) | `disable_start_recommended_section` | Interface & Explorer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Hide the unsupported hardware notice](interface.md#hide-the-unsupported-hardware-notice) | `hide_unsupported_hardware_notice` | Interface & Explorer tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Hide the mobile device panel in Start](interface.md#hide-the-mobile-device-panel-in-start) | `disable_phone_companion_start` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Turn off the Drop Tray share overlay](interface.md#turn-off-the-drop-tray-share-overlay) | `disable_drag_tray` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Alt+Tab shows windows only](interface.md#alttab-shows-windows-only) | `alt_tab_hide_browser_tabs` | Interface & Explorer tweaks | Dropdown (5 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off the Snap Assist suggestion picker](interface.md#turn-off-the-snap-assist-suggestion-picker) | `disable_snap_assist` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Expand the tree to the open folder](interface.md#expand-the-tree-to-the-open-folder) | `explorer_expand_to_current_folder` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Restore Explorer windows at sign-in](interface.md#restore-explorer-windows-at-sign-in) | `explorer_restore_folders_at_logon` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Show the full date and time in the tray](interface.md#show-the-full-date-and-time-in-the-tray) | `taskbar_full_date_time` | Interface & Explorer tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Hide recently added apps in Start](interface.md#hide-recently-added-apps-in-start) | `hide_recently_added_apps` | Interface & Explorer tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Focus the last active window on click](interface.md#focus-the-last-active-window-on-click) | `taskbar_last_active_click` | Interface & Explorer tweaks | Switch (2 options) | low | none | no | VERIFIED |
| [Remove the Notification Center](interface.md#remove-the-notification-center) | `disable_notification_center` | Interface & Explorer tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Enable DNS over HTTPS auto-upgrade](network.md#enable-dns-over-https-auto-upgrade) | `dns_over_https` | Network & Power tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Require encrypted DNS](network.md#require-encrypted-dns) | `require_doh` | Network & Power tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable LLMNR](network.md#disable-llmnr) | `disable_llmnr` | Network & Power tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable NetBIOS over TCP/IP](network.md#disable-netbios-over-tcpip) | `disable_netbios_tcpip` | Network & Power tweaks | Switch | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable mDNS](network.md#disable-mdns) | `disable_mdns` | Network & Power tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable WPAD auto-proxy discovery](network.md#disable-wpad-auto-proxy-discovery) | `disable_wpad` | Network & Power tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable IPv6 transition technologies](network.md#disable-ipv6-transition-technologies) | `disable_ipv6_transition` | Network & Power tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Internet Connection Sharing](network.md#disable-internet-connection-sharing) | `disable_internet_connection_sharing` | Network & Power tweaks | Switch | medium | admin | yes | VERIFIED |
| [Firewall logging and local policy merge](network.md#firewall-logging-and-local-policy-merge) | `firewall_logging_and_merge` | Network & Power tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable NIC power management](network.md#disable-nic-power-management) | `disable_nic_power_management` | Network & Power tweaks | Switch | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable hibernation](network.md#disable-hibernation) | `disable_hibernation` | Network & Power tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable USB selective suspend](network.md#disable-usb-selective-suspend) | `disable_usb_selective_suspend` | Network & Power tweaks | Switch | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable wake timers](network.md#disable-wake-timers) | `disable_wake_timers` | Network & Power tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Modern Standby (force S3)](network.md#disable-modern-standby-force-s3) | `disable_modern_standby` | Network & Power tweaks | Switch (2 options) | high | admin | yes | VERIFIED |
| [Set visual effects to best performance](performance.md#set-visual-effects-to-best-performance) | `optimize_visual_effects` | Performance & Gaming tweaks | Switch (2 options) | low | none | no | INCORRECT (corrected form ships) |
| [Disable Windows Search indexing](performance.md#disable-windows-search-indexing) | `disable_search_indexing` | Performance & Gaming tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Block UWP background apps](performance.md#block-uwp-background-apps) | `disable_background_apps` | Performance & Gaming tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable SysMain (SuperFetch) prefetching](performance.md#disable-sysmain-superfetch-prefetching) | `memory_prefetch_mode` | Performance & Gaming tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Halve the MMCSS reserved CPU (System Responsiveness)](performance.md#halve-the-mmcss-reserved-cpu-system-responsiveness) | `system_responsiveness` | Performance & Gaming tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn on Windows Game Mode](performance.md#turn-on-windows-game-mode) | `enable_game_mode` | Performance & Gaming tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Hardware-accelerated GPU scheduling (HAGS)](performance.md#hardware-accelerated-gpu-scheduling-hags) | `enable_gpu_scheduling` | Performance & Gaming tweaks | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Force exclusive fullscreen in games](performance.md#force-exclusive-fullscreen-in-games) | `disable_fullscreen_optimizations` | Performance & Gaming tweaks | Switch (2 options) | medium | none | no | INCORRECT (open: value semantics unsettled) |
| [Disable multi-plane overlay (MPO)](performance.md#disable-multi-plane-overlay-mpo) | `disable_multiplane_overlay` | Performance & Gaming tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Variable refresh rate for windowed games](performance.md#variable-refresh-rate-for-windowed-games) | `variable_refresh_rate` | Performance & Gaming tweaks | Dropdown (3 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn on optimizations for windowed games](performance.md#turn-on-optimizations-for-windowed-games) | `optimizations_windowed_games` | Performance & Gaming tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable Xbox Game Bar capture (Game DVR)](performance.md#disable-xbox-game-bar-capture-game-dvr) | `disable_gamedvr_capture` | Performance & Gaming tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Activate the Ultimate Performance power plan](performance.md#activate-the-ultimate-performance-power-plan) | `ultimate_performance_power_plan` | Performance & Gaming tweaks | Switch | medium | admin | no | INCORRECT (corrected form ships) |
| [Disable CPU power throttling](performance.md#disable-cpu-power-throttling) | `disable_power_throttling` | Performance & Gaming tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Lift the multimedia network throttling cap](performance.md#lift-the-multimedia-network-throttling-cap) | `network_throttling_index` | Performance & Gaming tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Storage Sense auto-cleanup](performance.md#disable-storage-sense-auto-cleanup) | `disable_storage_sense` | Performance & Gaming tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable SSD TRIM (delete notification)](performance.md#enable-ssd-trim-delete-notification) | `ssd_optimize_trim` | Performance & Gaming tweaks | Switch (2 options) | low | admin | no | INCORRECT (corrected form ships) |
| [Pin NTFS last-access updates off](performance.md#pin-ntfs-last-access-updates-off) | `ntfs_disable_lastaccess` | Performance & Gaming tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable RAM memory compression](performance.md#disable-ram-memory-compression) | `disable_memory_compression` | Performance & Gaming tweaks | Switch | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Fast Startup (hiberboot)](performance.md#disable-fast-startup-hiberboot) | `disable_fast_startup` | Performance & Gaming tweaks | Switch (2 options) | low | admin | no (next shutdown) | VERIFIED-WITH-CORRECTION |
| [Disable VBS and Memory Integrity (HVCI)](performance.md#disable-vbs-and-memory-integrity-hvci) | `disable_vbs_hvci` | Performance & Gaming tweaks | Switch (2 options) | critical | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Spectre / Meltdown CPU mitigations](performance.md#disable-spectre--meltdown-cpu-mitigations) | `disable_spectre_meltdown` | Performance & Gaming tweaks | Switch (2 options) | critical | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable mouse acceleration (Enhance Pointer Precision)](performance.md#disable-mouse-acceleration-enhance-pointer-precision) | `disable_mouse_acceleration` | Performance & Gaming tweaks | Switch (2 options) | low | none | no (sign-out) | VERIFIED-WITH-CORRECTION |
| [Turn off reserved storage](performance.md#turn-off-reserved-storage) | `reserved_storage_off` | Performance & Gaming tweaks | Switch | medium | admin | no | VERIFIED |
| [Limit diagnostic data to the Required level](privacy.md#limit-diagnostic-data-to-the-required-level) | `disable_diagnostic_data` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable the Customer Experience Improvement Program](privacy.md#disable-the-customer-experience-improvement-program) | `disable_ceip_tasks` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Compatibility Appraiser tasks](privacy.md#disable-compatibility-appraiser-tasks) | `disable_compat_appraiser` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off feedback request notifications](privacy.md#turn-off-feedback-request-notifications) | `disable_feedback_notifications` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Set feedback prompt frequency to Never](privacy.md#set-feedback-prompt-frequency-to-never) | `disable_feedback_frequency` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Block OneSettings config downloads](privacy.md#block-onesettings-config-downloads) | `disable_onesettings_downloads` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Exclude the device name from telemetry](privacy.md#exclude-the-device-name-from-telemetry) | `disable_device_name_in_telemetry` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Activity History and Timeline](privacy.md#disable-activity-history-and-timeline) | `disable_activity_history` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable the cross-device cloud clipboard](privacy.md#disable-the-cross-device-cloud-clipboard) | `disable_cloud_clipboard` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off online speech recognition](privacy.md#turn-off-online-speech-recognition) | `disable_online_speech_recognition` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable inking and typing personalization](privacy.md#disable-inking-and-typing-personalization) | `disable_inking_typing_personalization` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off the advertising ID](privacy.md#turn-off-the-advertising-id) | `disable_advertising_id` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off tailored experiences](privacy.md#turn-off-tailored-experiences) | `disable_tailored_experiences` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off location tracking](privacy.md#turn-off-location-tracking) | `disable_location_tracking` | Privacy & Telemetry tweaks | Switch | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Block app diagnostic access](privacy.md#block-app-diagnostic-access) | `disable_app_diagnostics` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off Find My Device](privacy.md#turn-off-find-my-device) | `disable_find_my_device` | Privacy & Telemetry tweaks | Switch (2 options) | medium | admin | no | VERIFIED |
| [Disable settings sync](privacy.md#disable-settings-sync) | `disable_settings_sync` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Disable Windows Error Reporting](privacy.md#disable-windows-error-reporting) | `disable_error_reporting` | Privacy & Telemetry tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Turn off suggested content in Settings](privacy.md#turn-off-suggested-content-in-settings) | `disable_suggested_content_settings` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off Start menu app suggestions](privacy.md#turn-off-start-menu-app-suggestions) | `disable_start_app_suggestions` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off app launch tracking](privacy.md#turn-off-app-launch-tracking) | `disable_app_launch_tracking` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off lock screen ads and fun facts](privacy.md#turn-off-lock-screen-ads-and-fun-facts) | `disable_lockscreen_spotlight_ads` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Turn off Windows tips and suggestions](privacy.md#turn-off-windows-tips-and-suggestions) | `disable_tips_and_suggestions` | Privacy & Telemetry tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Windows consumer features](privacy.md#disable-windows-consumer-features) | `disable_consumer_features` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Turn off Microsoft Edge telemetry](privacy.md#turn-off-microsoft-edge-telemetry) | `disable_edge_telemetry` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block website access to your language list](privacy.md#block-website-access-to-your-language-list) | `disable_language_list_access` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable File Explorer cloud recommendations](privacy.md#disable-file-explorer-cloud-recommendations) | `disable_explorer_cloud_recommendations` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable app and device inventory collectors](privacy.md#disable-app-and-device-inventory-collectors) | `disable_app_device_inventory` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off device search history](privacy.md#turn-off-device-search-history) | `disable_search_history` | Privacy & Telemetry tweaks | Switch (2 options) | low | none | no | VERIFIED-WITH-CORRECTION |
| [Disable cloud content in search](privacy.md#disable-cloud-content-in-search) | `disable_cloud_content_search` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block voice activation and wake words](privacy.md#block-voice-activation-and-wake-words) | `disable_voice_activation` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Windows Backup and cloud restore](privacy.md#disable-windows-backup-and-cloud-restore) | `disable_windows_backup` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Settings app online tips](privacy.md#disable-settings-app-online-tips) | `disable_online_tips` | Privacy & Telemetry tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable the Windows Error Reporting service](privacy.md#disable-the-windows-error-reporting-service) | `disable_wer_service` | Privacy & Telemetry tweaks | Switch | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable the Remote Registry service](security.md#disable-the-remote-registry-service) | `disable_remote_registry` | Security Hardening tweaks | Switch | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Remote Desktop (RDP)](security.md#disable-remote-desktop-rdp) | `disable_remote_desktop` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove SMBv1 protocol](security.md#remove-smbv1-protocol) | `remove_smbv1` | Security Hardening tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable WDigest credential caching](security.md#disable-wdigest-credential-caching) | `disable_wdigest` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Enable LSA protection (RunAsPPL)](security.md#enable-lsa-protection-runasppl) | `enable_lsa_protection` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Enforce NTLMv2 only](security.md#enforce-ntlmv2-only) | `enforce_ntlmv2` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Require SMB signing](security.md#require-smb-signing) | `require_smb_signing` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Harden RDP (NLA + TLS)](security.md#harden-rdp-nla--tls) | `rdp_security_hardening` | Security Hardening tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable administrative shares (C$, ADMIN$)](security.md#disable-administrative-shares-c-admin) | `disable_admin_shares` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Prevent LM hash storage](security.md#prevent-lm-hash-storage) | `disable_lmhash_storage` | Security Hardening tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Restrict anonymous enumeration](security.md#restrict-anonymous-enumeration) | `restrict_anonymous_enum` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Reduce cached domain logons](security.md#reduce-cached-domain-logons) | `reduce_credential_caching` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable the Microsoft vulnerable-driver blocklist](security.md#enable-the-microsoft-vulnerable-driver-blocklist) | `block_vulnerable_drivers` | Security Hardening tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable WPBT vendor binary execution](security.md#disable-wpbt-vendor-binary-execution) | `disable_wpbt` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Require Ctrl+Alt+Del at sign-in](security.md#require-ctrlaltdel-at-sign-in) | `require_ctrlaltdel` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable AutoRun/AutoPlay on all drives](security.md#disable-autorunautoplay-on-all-drives) | `disable_autorun` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Enable PowerShell script-block logging](security.md#enable-powershell-script-block-logging) | `powershell_scriptblock_logging` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Raise UAC to always notify](security.md#raise-uac-to-always-notify) | `uac_max` | Security Hardening tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Apply UAC to built-in Administrator](security.md#apply-uac-to-built-in-administrator) | `filter_admin_token` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Hide last signed-in username](security.md#hide-last-signed-in-username) | `hide_last_user` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Restrict printer-driver install to admins](security.md#restrict-printer-driver-install-to-admins) | `printnightmare_point_and_print` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Remote Assistance](security.md#disable-remote-assistance) | `disable_remote_assistance` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Windows Script Host](security.md#disable-windows-script-host) | `disable_windows_script_host` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Controlled Folder Access (ransomware shield)](security.md#controlled-folder-access-ransomware-shield) | `enable_controlled_folder_access` | Security Hardening tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED |
| [Defender Network Protection](security.md#defender-network-protection) | `enable_network_protection` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable PUA/PUP protection](security.md#enable-puapup-protection) | `enable_pua_protection` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block LSASS credential theft (ASR rule)](security.md#block-lsass-credential-theft-asr-rule) | `asr_block_lsass_theft` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | INCORRECT (corrected form ships) |
| [ASR rules: block Office/script malware vectors](security.md#asr-rules-block-officescript-malware-vectors) | `asr_block_office_script_vectors` | Security Hardening tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Enforce SmartScreen (apps and Edge)](security.md#enforce-smartscreen-apps-and-edge) | `enforce_smartscreen` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable legacy TLS 1.0/1.1 (Schannel)](security.md#disable-legacy-tls-1011-schannel) | `disable_tls_legacy` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Force .NET strong crypto (TLS 1.2+)](security.md#force-net-strong-crypto-tls-12) | `dotnet_strong_crypto` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable SMB insecure guest logons](security.md#disable-smb-insecure-guest-logons) | `disable_smb_guest` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Remove PowerShell 2.0 engine](security.md#remove-powershell-20-engine) | `remove_powershell_v2` | Security Hardening tweaks | Switch | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Enforce the firewall on all profiles](security.md#enforce-the-firewall-on-all-profiles) | `firewall_all_profiles` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable logon/credential auditing](security.md#enable-logoncredential-auditing) | `audit_logon_events` | Security Hardening tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Lock the screen when idle](security.md#lock-the-screen-when-idle) | `lock_on_inactivity` | Security Hardening tweaks | Switch | low | none | no | VERIFIED-WITH-CORRECTION |
| [Enable Credential Guard](security.md#enable-credential-guard) | `enable_credential_guard` | Security Hardening tweaks | Switch (2 options) | medium | admin | yes | INCORRECT (corrected form ships) |
| [Defender cloud protection and MAPS](security.md#defender-cloud-protection-and-maps) | `defender_cloud_protection` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block NTLM on the SMB client](security.md#block-ntlm-on-the-smb-client) | `smb_client_block_ntlm` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Enhanced Phishing Protection](security.md#enhanced-phishing-protection) | `enhanced_phishing_protection` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [ASR standard protection rules](security.md#asr-standard-protection-rules) | `asr_standard_protection_rules` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Disable WinRM remoting](security.md#disable-winrm-remoting) | `disable_winrm_remoting` | Security Hardening tweaks | Switch | medium | admin | yes | VERIFIED |
| [PowerShell module logging and transcription](security.md#powershell-module-logging-and-transcription) | `powershell_module_transcript_logging` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Restrict remote SAM calls to administrators](security.md#restrict-remote-sam-calls-to-administrators) | `restrict_remote_sam` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Block AlwaysInstallElevated](security.md#block-alwaysinstallelevated) | `block_always_install_elevated` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Kernel DMA protection policy](security.md#kernel-dma-protection-policy) | `kernel_dma_protection` | Security Hardening tweaks | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [ASR extended rule set](security.md#asr-extended-rule-set) | `asr_extended_rules` | Security Hardening tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED |
| [Restrict outgoing NTLM](security.md#restrict-outgoing-ntlm) | `ntlm_outgoing_restriction` | Security Hardening tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Harden the RDP session](security.md#harden-the-rdp-session) | `rdp_session_hardening` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Disable the Secondary Logon service](security.md#disable-the-secondary-logon-service) | `disable_secondary_logon` | Security Hardening tweaks | Switch | medium | admin | yes | VERIFIED |
| [Early Launch Antimalware driver policy](security.md#early-launch-antimalware-driver-policy) | `early_launch_antimalware_policy` | Security Hardening tweaks | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Force updated CredSSP clients](security.md#force-updated-credssp-clients) | `credssp_encryption_oracle` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Prevent automatic device encryption](security.md#prevent-automatic-device-encryption) | `device_encryption_posture` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED |
| [Hide admin accounts on the UAC prompt](security.md#hide-admin-accounts-on-the-uac-prompt) | `hide_admin_accounts_on_elevation` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Event log retention size](security.md#event-log-retention-size) | `event_log_retention` | Security Hardening tweaks | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Log command lines in process-creation events](security.md#log-command-lines-in-process-creation-events) | `audit_process_creation_cmdline` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off the spooler's remote RPC endpoint](security.md#turn-off-the-spoolers-remote-rpc-endpoint) | `spooler_remote_rpc_off` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Filter the remote local-admin token](security.md#filter-the-remote-local-admin-token) | `remote_uac_token_filter` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED |
| [Enable SEHOP](security.md#enable-sehop) | `enable_sehop` | Security Hardening tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Block PKU2U online identities](security.md#block-pku2u-online-identities) | `disable_pku2u_online_id` | Security Hardening tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Do not index encrypted files](security.md#do-not-index-encrypted-files) | `no_index_encrypted_files` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disallow AutoPlay for non-volume devices](security.md#disallow-autoplay-for-non-volume-devices) | `autoplay_non_volume` | Security Hardening tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable User Experiences and Telemetry (DiagTrack)](services.md#disable-user-experiences-and-telemetry-diagtrack) | `disable_diagtrack` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Print Spooler (Spooler)](services.md#disable-print-spooler-spooler) | `disable_print_spooler` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Fax Service (Fax)](services.md#disable-fax-service-fax) | `disable_fax` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | INCORRECT on the primary platform (Windows 10 only) |
| [Program Compatibility Assistant Service (PcaSvc)](services.md#program-compatibility-assistant-service-pcasvc) | `disable_program_compat_assistant` | Services & Scheduled Tasks tweaks | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Distributed Link Tracking Client (TrkWks)](services.md#disable-distributed-link-tracking-client-trkwks) | `disable_distributed_link_tracking` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Disable Retail Demo Service (RetailDemo)](services.md#disable-retail-demo-service-retaildemo) | `disable_retail_demo` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Wallet Service (WalletService)](services.md#disable-wallet-service-walletservice) | `disable_wallet_service` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Touch Keyboard Service (TabletInputService)](services.md#disable-touch-keyboard-service-tabletinputservice) | `disable_touch_keyboard` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | INCORRECT on the primary platform (Windows 10 only) |
| [Disable Bluetooth Support Services (bthserv)](services.md#disable-bluetooth-support-services-bthserv) | `disable_bluetooth` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable AllJoyn Router Service (AJRouter)](services.md#disable-alljoyn-router-service-ajrouter) | `disable_alljoyn_router` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | INCORRECT on the primary platform (Windows 10 only) |
| [Disable Phone Service (PhoneSvc)](services.md#disable-phone-service-phonesvc) | `disable_phone_service` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Network Device Discovery (SSDPSRV/upnphost)](services.md#disable-network-device-discovery-ssdpsrvupnphost) | `disable_ssdp_upnp` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Windows Insider Service (wisvc)](services.md#disable-windows-insider-service-wisvc) | `disable_windows_insider` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Downloaded Maps Manager (MapsBroker)](services.md#disable-downloaded-maps-manager-mapsbroker) | `disable_maps_broker` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED |
| [Disable Geolocation Service (lfsvc)](services.md#disable-geolocation-service-lfsvc) | `disable_geolocation` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Xbox Live Services (XblAuthManager et al.)](services.md#xbox-live-services-xblauthmanager-et-al) | `disable_xbox_services` | Services & Scheduled Tasks tweaks | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Connected Devices Platform Service (CDPSvc)](services.md#disable-connected-devices-platform-service-cdpsvc) | `disable_cdpsvc` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Windows Biometric Service (WbioSrvc)](services.md#disable-windows-biometric-service-wbiosrvc) | `disable_biometrics` | Services & Scheduled Tasks tweaks | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Smart Card Services (SCardSvr)](services.md#disable-smart-card-services-scardsvr) | `disable_smartcard` | Services & Scheduled Tasks tweaks | Switch (2 options) | high | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Sensor Services (SensorService)](services.md#disable-sensor-services-sensorservice) | `disable_sensor_services` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Family Safety Monitor (WpcMonSvc)](services.md#disable-family-safety-monitor-wpcmonsvc) | `disable_parental_controls` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Payments and NFC/SE Manager (SEMgrSvc)](services.md#disable-payments-and-nfcse-manager-semgrsvc) | `disable_payments_nfc` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Media Player Network Sharing (WMPNetworkSvc)](services.md#disable-media-player-network-sharing-wmpnetworksvc) | `disable_wmp_network_sharing` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Autochk Proxy task](services.md#disable-autochk-proxy-task) | `task_autochk_proxy` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Feedback (SIUF) DmClient tasks](services.md#disable-feedback-siuf-dmclient-tasks) | `task_feedback_dmclient` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Windows Error Reporting QueueReporting task](services.md#disable-windows-error-reporting-queuereporting-task) | `task_wer_queuereporting` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Offline Maps update tasks](services.md#disable-offline-maps-update-tasks) | `task_maps_update` | Services & Scheduled Tasks tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable DiskDiagnosticDataCollector task](services.md#disable-diskdiagnosticdatacollector-task) | `task_disk_diagnostic_datacollector` | Services & Scheduled Tasks tweaks | Switch | low | admin | no | INCORRECT (no trigger; stock state unestablished) |
| [Disable Device Census tasks](services.md#disable-device-census-tasks) | `task_device_census` | Services & Scheduled Tasks tweaks | Switch (2 options) | low | admin | no | INCORRECT (corrected form ships) |
| [Disable Delivery Optimization P2P](windows_update.md#disable-delivery-optimization-p2p) | `disable_delivery_optimization_p2p` | Windows Update tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Defer quality updates](windows_update.md#defer-quality-updates) | `defer_quality_updates` | Windows Update tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED |
| [Defer feature updates](windows_update.md#defer-feature-updates) | `defer_feature_updates` | Windows Update tweaks | Dropdown (4 options) | low | admin | no | VERIFIED |
| [Pin Windows feature version](windows_update.md#pin-windows-feature-version) | `target_release_version` | Windows Update tweaks | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Control mid-cycle features and optional content](windows_update.md#control-mid-cycle-features-and-optional-content) | `update_feature_control` | Windows Update tweaks | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block Insider preview builds by policy](windows_update.md#block-insider-preview-builds-by-policy) | `block_insider_builds_policy` | Windows Update tweaks | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Set active hours](windows_update.md#set-active-hours) | `set_active_hours` | Windows Update tweaks | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block auto-restart while signed in](windows_update.md#block-auto-restart-while-signed-in) | `disable_auto_restart_logged_on` | Windows Update tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Exclude driver updates from Windows Update](windows_update.md#exclude-driver-updates-from-windows-update) | `exclude_wu_driver_updates` | Windows Update tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable automatic driver installation](windows_update.md#disable-automatic-driver-installation) | `disable_auto_driver_install` | Windows Update tweaks | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Microsoft Store auto-updates](windows_update.md#disable-microsoft-store-auto-updates) | `disable_store_auto_updates` | Windows Update tweaks | Switch (2 options) | medium | admin | no | VERIFIED |
| [Block auto-download over metered](windows_update.md#block-auto-download-over-metered) | `block_update_over_metered` | Windows Update tweaks | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Windows Update mode](windows_update.md#windows-update-mode) | `windows_update_mode` | Windows Update tweaks | Dropdown (4 options) | high | admin | no | VERIFIED |
| [Block the Windows Update pipeline](windows_update.md#block-the-windows-update-pipeline) | `block_update_pipeline` | Windows Update tweaks | Switch | high | admin (ti for 21 of 30 effects) | yes | VERIFIED (empirically tested; two open questions) |
