# Windows Tweak Catalog (research draft)

Curated catalog of genuinely useful Windows tweaks for the MagicX Toolbox corpus (task 3 on the
`tweak-system-redesign` branch). Every entry was researched and cross-checked against Microsoft
documentation and reputable community projects, then filtered to drop placebo and cargo-cult tweaks.
The rejected ones are listed at the end ("Excluded") so we never re-add them by mistake.

Scope: Windows 10 22H2, and Windows 11 22H2 / 23H2 / 24H2 (x64).

Each entry reads as: what it does and what you gain, then the concrete mechanism (registry value,
service, scheduled task, or appx package) so it can be authored directly, then a meta tag with risk,
applicability, and whether a reboot or sign-out is needed. A `Note:` line is added where a tweak
carries a real trade-off or warning.

Risk legend:

- **low**: safe or cosmetic, easily reversible.
- **medium**: a functional trade-off (may disable a convenience feature or break a niche workflow).
- **high**: weakens security or stability; for advanced users who accept the cost.
- **critical**: disables a core protection; only with a clear, deliberate reason.

Validated 2026-07-23 by three independent adversarial passes plus an address-collision audit, against
authoritative sources (Microsoft Learn, admx.help, KB). Pass 1 (mechanism/classification/applicability):
20 fixes, including Wi-Fi Sense demoted to myths. Pass 2 (exact value/type/polarity): 4 REG_SZ-versus-
DWORD and enum fixes. Pass 3 (whole-catalog): removed cross-domain duplicate entries whose metadata had
drifted (an earlier count double-counted them), reconciled risk labels, dropped a self-contradicting
entry (dmwappushservice), raised an under-labeled lockout risk (smart-card service), and added two
missing high-value tweaks (mouse acceleration, accessibility key prompts). The collision audit
(see 2026-07-23-address-collision-audit.md) confirmed the one remaining same-value multi-writer is a
packed REG_SZ handled by field addressing. Inline notes: `Verifier fix:` / `Round-2 value fix:` /
`Consistency fix:` / `Safety fix:` record corrections; `Unverified:` marks community-only tweaks.

Status: research draft feeding task 3, not yet authored into YAML. Generated from structured research
(207 curated tweaks after cross-domain dedup, 86 excluded myths). See `PRE_MERGE_TASKS.md`.

## Privacy & Telemetry

- **Disable Windows Recall snapshots**: Stop Windows Recall from saving AI screenshots of everything on screen. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI\DisableAIDataAnalysis (DWORD) = 1` _(low; Win11 24H2+ (Copilot+ PCs); reboot: yes)_
  - Note: Recall is opt-in and off by default on managed/commercial devices, so this mainly hard-locks the off state; enabling the policy deletes any snapshots already saved. Only meaningful on Copilot+ hardware where Recall exists. Reversible by setting to 0 or deleting the value. Consistency fix: absorbed a duplicate Recall-disable entry from another domain (same DisableAIDataAnalysis=1).
- **Remove the Recall optional component**: Uninstall the Recall feature bits entirely, not just disable saving. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI\AllowRecallEnablement (DWORD) = 0` _(low; Win11 24H2+ (Copilot+ PCs); reboot: yes)_
  - Note: More aggressive than DisableAIDataAnalysis: removes the Recall bits from the device and deletes existing snapshots; requires a restart. Fully reversible by setting to 1. No effect on non-Copilot+ hardware.
- **Disable Click to Do**: Turn off the AI Click to Do overlay that screenshots and analyzes the screen. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI\DisableClickToDo (DWORD) = 1` _(low; Win11 24H2+ (Copilot+ PCs); reboot: yes)_
  - Note: Click to Do analysis is local-only, but it does capture the screen on demand. Feature ships on Copilot+ PCs and is still rolling out; the policy is a no-op on hardware where the feature is absent. Reversible by setting to 0.
- **Minimize diagnostic data to Required**: Lower Windows diagnostic data from Optional/Full down to the minimum level. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection\AllowTelemetry (DWORD) = 1 on Home/Pro (Required), = 0 (Security) only on Enterprise/Education/IoT/LTSC` _(low; Win10+11; Security(0) honored only on Enterprise/Education/IoT/LTSC, Home/Pro floor to Required(1); reboot: no)_
  - Note: Honest limit: Home and Pro floor a 0 value to 1 (Required/Basic) and cannot reach Security(0); only Enterprise, Education, and IoT Enterprise/LTSC honor 0. Setting also gates behind DiagTrack running. Windows Insider builds force a higher level. Does not weaken security. Verifier fix: AllowTelemetry=0 selects the Security level, which Home/Pro silently floor to 1 (Required), so 0 does not reach 'off' on consumer SKUs. Set =1 on Home/Pro to land on Required explicitly.
- **Disable CEIP scheduled tasks**: Turn off Customer Experience Improvement Program telemetry tasks. `task: \Microsoft\Windows\Customer Experience Improvement Program\{Consolidator, UsbCeip, KernelCeipTask} -> Disabled; reg: HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows\CEIPEnable (DWORD) = 0` _(low; Win10+11; reboot: no)_
- **Disable Compatibility Appraiser tasks**: Stop the Application Experience appraiser/inventory telemetry tasks. `task: \Microsoft\Windows\Application Experience\{Microsoft Compatibility Appraiser, ProgramDataUpdater, StartupAppTask} -> Disabled; reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat\AITEnable (DWORD) = 0, DisableInventory (DWORD) = 1` _(low; Win10+11; reboot: no)_
  - Note: Microsoft Compatibility Appraiser (CompatTelRunner.exe) is the heaviest telemetry task and can cause CPU/disk spikes; disabling it is safe and reduces upgrade-readiness reporting. Do not delete or rename CompatTelRunner.exe (it reverts on update and breaks servicing). Reversible.
- **Disable feedback request notifications**: Stop Windows from prompting for feedback surveys. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection\DoNotShowFeedbackNotifications (DWORD) = 1` _(low; Win10+11; reboot: no)_
  - Note: Cosmetic/behavioral only; suppresses the periodic 'How are you feeling about Windows?' prompts. Fully reversible by deleting the value.
- **Set feedback frequency to never**: Zero out the per-user Windows feedback prompt schedule. `reg: HKCU\Software\Microsoft\Siuf\Rules\NumberOfSIUFInPeriod (DWORD) = 0; delete PeriodInNanoSeconds` _(low; Win10+11; reboot: no)_
- **Block OneSettings config downloads**: Stop Windows from pulling remote OneSettings that can re-toggle telemetry. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection\DisableOneSettingsDownloads (DWORD) = 1` _(low; Win10+11; reboot: no)_
- **Strip device name from telemetry**: Prevent the computer name from being included in diagnostic data. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection\AllowDeviceNameInTelemetry (DWORD) = 0` _(low; Win10+11; reboot: no)_
  - Note: Only relevant while any diagnostic data is still sent (default excludes device name, so this hard-locks the exclusion). Reversible by deleting the value.
- **Disable Activity History and Timeline**: Stop Windows collecting and uploading activity history to your account. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\System -> EnableActivityFeed = 0, PublishUserActivities = 0, UploadUserActivities = 0 (all DWORD)` _(low; Win10+11; reboot: yes)_
  - Note: Cloud upload of activity history was already retired for consumers, but these policies also stop local collection feeding Timeline/resume features. Functional trade-off only; reversible by setting to 1 or deleting.
- **Disable cross-device cloud clipboard**: Stop clipboard contents syncing to Microsoft cloud and other devices. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\System\AllowCrossDeviceClipboard (DWORD) = 0` _(low; Win10+11; reboot: no)_
- **Disable online speech recognition**: Turn off cloud speech processing and voice-clip data sharing. `reg: HKCU\Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy\HasAccepted (DWORD) = 0` _(low; Win10+11; reboot: no)_
- **Disable inking and typing personalization**: Stop Windows harvesting your typing/handwriting to build a language model. `reg: HKCU\Software\Microsoft\Personalization\Settings\AcceptedPrivacyPolicy = 0, HKCU\Software\Microsoft\InputPersonalization -> RestrictImplicitTextCollection = 1, RestrictImplicitInkCollection = 1, HKCU\Software\Microsoft\InputPersonalization\TrainedDataStore\HarvestContacts = 0; machine lock: HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization\AllowInputPersonalization = 0 (all DWORD)` _(low; Win10+11; reboot: no)_
- **Disable the Advertising ID**: Turn off the per-user advertising ID apps use for targeted ads. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo\DisabledByGroupPolicy (DWORD) = 1; per-user: HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo\Enabled (DWORD) = 0` _(low; Win10+11; reboot: no)_
  - Note: Cosmetic privacy win; ads still appear but are no longer tied to the tracked ID. Reversible by deleting the policy value.
- **Disable tailored experiences with diagnostic data**: Stop Windows using diagnostic data to target tips, ads, and recommendations. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Privacy\TailoredExperiencesWithDiagnosticDataEnabled (DWORD) = 0; machine: HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent\DisableTailoredExperiencesWithDiagnosticData (DWORD) = 1` _(low; Win10+11; reboot: no)_
- **Disable location tracking**: Turn off the system location platform and per-app location access. `reg: HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location\Value = "Deny" (REG_SZ); policy: HKLM\SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors\DisableLocation (DWORD) = 1` _(medium; Win10+11; reboot: no)_
  - Note: Breaks Find My Device, weather, Maps, automatic time zone, and any app needing location. Functional trade-off, not a security change. Reversible by setting Value back to Allow and removing DisableLocation.
- **Disable app diagnostic access**: Block apps from reading diagnostic info about other running apps. `reg: HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\appDiagnostics\Value = "Deny" (REG_SZ); policy: HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy\LetAppsGetDiagnosticInfo (DWORD) = 2` _(low; Win10+11; reboot: no)_
  - Note: Rarely used by legitimate apps; some diagnostics/monitoring utilities may lose data. Reversible by restoring Allow / removing the policy.
- **Disable Find My Device**: Turn off periodic location reporting used to locate a lost device. `reg: HKLM\SOFTWARE\Policies\Microsoft\FindMyDevice\AllowFindMyDevice (DWORD) = 0` _(medium; Win10+11; reboot: no)_
  - Note: Anti-theft trade-off: disabling removes the ability to locate/lock a lost or stolen device from your Microsoft account. It also stops periodic location pings. Reversible by setting to 1.
- **Disable settings sync**: Stop Windows syncing settings and preferences to your Microsoft account. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\SettingSync -> DisableSettingSync (DWORD) = 2, DisableSettingSyncUserOverride (DWORD) = 1` _(low; Win10+11; reboot: yes)_
- **Disable Windows Error Reporting**: Stop crash and error reports from being sent to Microsoft. `reg: HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Disabled (DWORD) = 1; policy: HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting\Disabled (DWORD) = 1` _(medium; Win10+11; reboot: no)_
  - Note: Reduces telemetry but also hinders your own crash diagnosis: local WER logs and dumps used by Reliability Monitor and third-party support are suppressed. Reversible by setting to 0. Prefer leaving on if you troubleshoot crashes.
- **Disable suggested content in Settings**: Remove the promotional suggested content shown inside the Settings app. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager -> SubscribedContent-338393Enabled = 0, SubscribedContent-353694Enabled = 0, SubscribedContent-353696Enabled = 0 (all DWORD)` _(low; Win10+11; reboot: no)_
- **Disable Start menu app suggestions**: Stop promoted app and 'occasionally show suggestions' entries in Start. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager\SystemPaneSuggestionsEnabled (DWORD) = 0` _(low; Win10+11; reboot: no)_
  - Note: Cosmetic; removes suggested apps in the Start menu. Reversible by setting to 1.
- **Disable app launch tracking**: Stop Windows tracking app launches to personalize Start and search. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Start_TrackProgs (DWORD) = 0` _(low; Win10+11; reboot: no)_
- **Disable lock screen ads and fun facts**: Turn off Windows Spotlight tips, ads, and 'fun facts' on the lock screen. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager -> RotatingLockScreenOverlayEnabled = 0, SubscribedContent-338387Enabled = 0 (all DWORD)` _(low; Win10+11; reboot: no)_
- **Disable Windows tips and suggestions**: Stop tip, trick, and suggestion notifications from Windows. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager -> SoftLandingEnabled = 0, SubscribedContent-338389Enabled = 0 (all DWORD); policy: HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent\DisableSoftLanding (DWORD) = 1` _(low; Win10+11; reboot: no)_
- **Disable Windows consumer features**: Block silent auto-install of promoted apps and Microsoft consumer suggestions. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\CloudContent\DisableWindowsConsumerFeatures (DWORD) = 1; HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager\SilentInstalledAppsEnabled (DWORD) = 0` _(low; Win10+11, Enterprise/Education only; silently ignored on Home AND Pro (Win11); reboot: yes)_
  - Note: Stops the auto-installed promoted apps (games, trials) and Microsoft account consumer suggestions. On Windows Home the DisableWindowsConsumerFeatures policy is historically ignored, so pair it with SilentInstalledAppsEnabled. Reversible. Verifier fix: per the ADMX 'supported on', this is Enterprise/Education-only and ignored on Home and Pro; pair it with the per-user ContentDeliveryManager keys (disable_auto_install_sponsored_apps), which do work on consumer SKUs.
- **Disable Microsoft Edge telemetry**: Turn off Edge usage, browsing-history, and third-party search telemetry. `reg: HKLM\SOFTWARE\Policies\Microsoft\Edge -> MetricsReportingEnabled = 0, SendSiteInfoToImproveServices = 0, PersonalizationReportingEnabled = 0, Edge3PSerpTelemetryEnabled = 0 (all DWORD)` _(low; Win10+11; reboot: no)_
  - Note: Applies only if Edge is installed. MetricsReportingEnabled is Edge's own diagnostic switch (separate from Windows AllowTelemetry). Takes effect on Edge restart; reversible by deleting the values.
- **Block website access to your language list**: Stop websites reading your language list for locally-relevant content. `reg: HKCU\Control Panel\International\User Profile\HttpAcceptLanguageOptOut (DWORD) = 1` _(low; Win10+11; reboot: no)_

## Debloat, AI & Consumer

- **Remove the Copilot app**: Uninstall the native Microsoft Copilot app so it stops appearing. `appx: Microsoft.Copilot (winget uninstall / Get-AppxPackage -AllUsers *Microsoft.Copilot* | Remove-AppxPackage); enterprise: policy Windows AI > Remove Microsoft Copilot app (RemoveMicrosoftCopilotApp)` _(low; Win11 24H2+; reboot: no)_
- **Uninstall the Recall optional feature**: Remove the Recall component entirely from the OS, not just disable it. `feature: DISM /Online /Disable-Feature /FeatureName:Recall (or optionalfeatures.exe > uncheck Recall)` _(low; Win11 24H2+ (Copilot+ PCs only); reboot: yes)_
- **Disable web/Bing results in Start search**: Stop Start menu search from querying Bing so it returns only local apps and files. `reg: HKCU\Software\Policies\Microsoft\Windows\Explorer\DisableSearchBoxSuggestions = 1 (also HKCU\Software\Microsoft\Windows\CurrentVersion\Search\BingSearchEnabled = 0)` _(low; Win10+11; reboot: yes)_
  - Note: Takes effect after an Explorer restart or sign-out. Also removes the Copilot entry from the search flyout. Purely local search still works fully.
- **Disable Search Highlights**: Remove promoted/trending 'search highlights' content from the search box and flyout. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search\EnableDynamicContentInWSB = 0 (per-user: ...\SearchSettings\IsDynamicSearchBoxEnabled = 0)` _(low; Win10+11; reboot: no)_
  - Note: Cosmetic; removes rotating illustrations/promoted items only. Reversible.
- **Hide the Start 'Recommended' section**: Remove the Recommended feed (recent files and promoted apps) from the Start menu. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer\HideRecommendedSection = 1 (+ PolicyManager\current\device\Start\HideRecommendedSection = 1)` _(low; Win11 (full hide historically Enterprise/Education; broadened on 24H2/25H2); reboot: yes)_
  - Note: On consumer editions the policy historically only collapses the section rather than fully removing it; the Education-environment flag is sometimes paired to force it. Reversible by deleting the values.
- **Disable Start tips/app promotions**: Stop 'recommendations for tips, shortcuts, new apps' from appearing in Start. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager\SubscribedContent-338388Enabled = 0 (24H2 also HKCU\...\Explorer\Advanced\Start_IrisRecommendations = 0)` _(low; Win10+11; reboot: no)_
- **Stop auto-installed sponsored apps**: Block silent install of promoted/OEM apps (Candy Crush, Spotify tiles, etc.) `reg (HKCU\...\ContentDeliveryManager): SilentInstalledAppsEnabled=0, PreInstalledAppsEnabled=0, OemPreInstalledAppsEnabled=0, SubscribedContentEnabled=0, ContentDeliveryAllowed=0` _(low; Win10+11; reboot: no)_
- **Disable post-update 'welcome experience'**: Suppress the full-screen 'what's new / welcome' page shown after updates. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager\SubscribedContent-310093Enabled = 0` _(low; Win10+11; reboot: no)_
- **Disable 'Get even more out of Windows' OOBE nag**: Stop the post-login 'finish setting up your device' full-screen upsell. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement\ScoobeSystemSettingEnabled = 0` _(low; Win10+11; reboot: no)_
- **Disable Widgets (News & Interests)**: Turn off the Widgets board and its taskbar entry point system-wide. `reg: HKLM\SOFTWARE\Policies\Microsoft\Dsh\AllowNewsAndInterests = 0` _(low; Win11; reboot: yes)_
  - Note: Machine-scoped policy (confirmed on Microsoft Learn). Blocks the whole widgets experience including taskbar content; the WebExperience host package remains installed. Reversible by deleting the value.
- **Remove the Chat/Teams taskbar icon**: Hide and lock off the consumer Teams 'Chat' button on the taskbar. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Chat\ChatIcon = 3 (policy Configure the Chat icon = Disabled)` _(low; Win11; reboot: no)_
  - Note: Only removes the icon; the consumer Teams app itself is removed separately (remove_teams_consumer_app).
- **Remove consumer Teams / free chat app**: Uninstall the preinstalled personal Microsoft Teams (chat) app. `appx: MicrosoftTeams (consumer PFN MicrosoftTeams_8wekyb3d8bbwe) via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win11; reboot: no)_
- **Uninstall OneDrive**: Unlink and remove the OneDrive client and its background sync. `run: %SystemRoot%\SysWOW64\OneDriveSetup.exe /uninstall (or winget uninstall Microsoft.OneDrive); remove HKCU\...\Run\OneDrive value` _(medium; Win10+11; reboot: no)_
  - Note: Unlink first. Files that are cloud-only (Files On-Demand) become inaccessible locally, Known Folder Move can strip Desktop/Documents/Pictures out of their folders, and Office AutoSave behavior changes. A feature update can reinstall the stub.
- **Disable File Explorer sync-provider ads**: Stop the promotional 'sync provider' banners/ads inside File Explorer. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\ShowSyncProviderNotifications = 0` _(low; Win10+11; reboot: no)_
- **Disable Edge first-run experience**: Skip Edge's import/sign-in first-run wizard on first launch. `reg: HKLM\SOFTWARE\Policies\Microsoft\Edge\HideFirstRunExperience = 1` _(low; Win10+11; reboot: no)_
- **Disable Edge startup boost and background mode**: Stop Edge from pre-launching processes at boot and running in the background. `reg: HKLM\SOFTWARE\Policies\Microsoft\Edge\StartupBoostEnabled = 0 and BackgroundModeEnabled = 0` _(low; Win10+11; reboot: no)_
- **Disable Edge sidebar and Copilot/Discover**: Remove the Edge sidebar (hubs) and its Copilot/Discover button. `reg: HKLM\SOFTWARE\Policies\Microsoft\Edge\HubsSidebarEnabled = 0 (+ EdgeCollectionsEnabled = 0)` _(low; Win10+11; reboot: no)_
  - Note: Cosmetic; removes the right-edge panel. Reversible.
- **Remove Cortana**: Uninstall the Cortana app and block the assistant. `appx: Microsoft.549981C3F5F10 via Get-AppxPackage -AllUsers | Remove-AppxPackage (+ policy HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search\AllowCortana = 0)` _(low; Win10+11 (already inert on Win11); reboot: no)_
  - Note: On Windows 11 Cortana is already deprecated, disabled by default, and not a system component, so this mostly matters on Windows 10. Removal does not break Search. Consistency fix: this app removal supersets and replaces a duplicate AllowCortana policy entry from another domain.
- **Remove Clipchamp**: Uninstall the preinstalled Clipchamp video editor. `appx: Clipchamp.Clipchamp via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win11; reboot: no)_
- **Remove Dev Home**: Uninstall the Dev Home app (deprecated by Microsoft) `appx: Microsoft.Windows.DevHome via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win11; reboot: no)_
- **Remove Quick Assist**: Uninstall the Quick Assist remote-help tool. `appx: MicrosoftCorporationII.QuickAssist via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
  - Note: You lose the built-in remote-assist tool, but Quick Assist is a common tech-support-scam vector, so removal on machines that never use it is also a security improvement. Reinstallable from the Store.
- **Remove Bing News and Weather apps**: Uninstall the MSN News and Weather apps that feed Widgets/live tiles. `appx: Microsoft.BingNews, Microsoft.BingWeather via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
  - Note: Widgets may still show weather via the web-experience host; removing these only kills the standalone apps. Safe.
- **Remove Microsoft Solitaire Collection**: Uninstall the ad-supported Solitaire games bundle. `appx: Microsoft.MicrosoftSolitaireCollection via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
- **Remove Get Help**: Uninstall the Get Help support app. `appx: Microsoft.GetHelp via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
- **Remove Tips (Get Started)**: Uninstall the Windows Tips / Get Started app. `appx: Microsoft.Getstarted via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
- **Remove Feedback Hub**: Uninstall the Feedback Hub telemetry/feedback app. `appx: Microsoft.WindowsFeedbackHub via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
- **Remove Windows Maps**: Uninstall the Windows Maps app. `appx: Microsoft.WindowsMaps via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
- **Remove the People app**: Uninstall the legacy People contacts app. `appx: Microsoft.People via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
- **Remove Phone Link**: Uninstall the Phone Link (Your Phone) companion app. `appx: Microsoft.YourPhone via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win10+11; reboot: no)_
  - Note: You lose phone mirroring/notifications integration. Reinstallable from the Store; may be re-provisioned by a feature update.
- **Remove the new Outlook app**: Uninstall the preinstalled 'new Outlook for Windows' web-wrapper client. `appx: Microsoft.OutlookForWindows via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(low; Win11; reboot: no)_
- **Remove the Xbox Game Bar overlay app**: Uninstall the Game Bar overlay app (Win+G) itself. `appx: Microsoft.XboxGamingOverlay via Get-AppxPackage -AllUsers | Remove-AppxPackage` _(medium; Win10+11; reboot: no)_
  - Note: A handful of games call Game Bar APIs (activity/social, some capture prompts) and can throw an ms-gamingoverlay error after removal. Prefer disable_game_dvr if you only want to stop background recording. Keep Microsoft.XboxIdentityProvider for game sign-in.

## Performance & Gaming

- **Optimize Visual Effects for Performance**: Disable UI animations and shadows for a snappier desktop and freed GPU/CPU cycles. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects\VisualFXSetting = 2 (Adjust for best performance); backed by packed HKCU\Control Panel\Desktop\UserPreferencesMask + per-effect Explorer\Advanced flags` _(low; Win10+11; reboot: no)_
- **Disable Windows Search Indexing**: Stop the background indexer to cut idle disk and CPU churn. `service: WSearch -> Disabled` _(medium; Win10+11; reboot: no)_
  - Note: Start Menu and File Explorer search become slow/non-instant. On modern SSD+RAM the gain is marginal (indexer writes <20 MB/day, <1% idle CPU); only worthwhile on low-RAM/HDD machines or if you use Everything or a third-party search.
- **Disable UWP Background Apps**: Stop Store/UWP apps from running and refreshing in the background. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy\LetAppsRunInBackground = 2 (Force Deny); or HKCU\...\CurrentVersion\Search\BackgroundAppGlobalToggle = 0` _(low; Win10+11; reboot: no)_
- **Disable SysMain (SuperFetch) Prefetching**: Stop the SysMain preloader to cut background disk and RAM pre-caching. `service: SysMain -> Disabled (optionally reg: ...\Memory Management\PrefetchParameters\EnableSuperfetch = 0)` _(medium; Win10+11; reboot: no)_
  - Note: Microsoft and most testing recommend leaving SysMain ON: on modern SSD+RAM it costs almost nothing and warms useful cache. Disabling only helps the minority of machines where SysMain spikes disk to 100%; on a healthy system there is no measurable gaming benefit and app launches can get slightly slower.
- **Reduce MMCSS Reserved CPU (System Responsiveness)**: Lower the CPU slice MMCSS reserves for background tasks. `reg: HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\SystemResponsiveness = 10 (default 20; 0 = none)` _(low; Win10+11; reboot: yes)_
- **Enable Windows Game Mode**: Let Windows deprioritize background work during play to smooth 1% lows. `reg: HKCU\Software\Microsoft\GameBar\AutoGameModeEnabled = 1 (and AllowAutoGameMode = 1)` _(low; Win10+11; reboot: no)_
- **Enable Hardware-Accelerated GPU Scheduling (HAGS)**: Let the GPU manage its own frame queue to trim CPU-side scheduling latency. `reg: HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\HwSchMode = 2 (1 = off)` _(medium; Win10 2004+ / Win11 (needs GPU + WDDM 2.7+ driver); reboot: yes)_
  - Note: Effect is small and hardware-dependent: usually a slight latency win, near-zero average FPS, with some extra VRAM use. Required for NVIDIA DLSS 4 Frame Generation. Has caused stutter for some VR users; verify per system. The key/toggle only appears when the GPU+driver support it.
- **Disable Fullscreen Optimizations Globally**: Force true exclusive fullscreen to drop the DWM compositing layer's latency. `reg: HKCU\System\GameConfigStore\GameDVR_FSEBehaviorMode = 2 + GameDVR_HonorUserFSEBehaviorMode = 1 (+ GameDVR_DXGIHonorFSEWindowsCompatible = 1)` _(medium; Win10+11; reboot: no)_
  - Note: Modern DXGI flip-model FSO is usually equal to or better than exclusive fullscreen and keeps fast Alt-Tab, Auto HDR and VRR in borderless. Forcing exclusive can break those and cause Alt-Tab black-screen flicker; it helps only specific stutter-prone titles. Test per game. Unverified: the GameConfigStore values are community-canonical but undocumented by Microsoft; on modern flip-model DWM the benefit is title-dependent and often negligible, not a guaranteed win.
- **Disable Multi-Plane Overlay (MPO)**: Turn off DWM hardware overlays to fix flicker and frame-time spikes. `reg (<=23H2): HKLM\SOFTWARE\Microsoft\Windows\Dwm\OverlayTestMode = 5; reg (24H2+): HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\DisableOverlays = 1` _(medium; Win10+11 (22H2/23H2 uses OverlayTestMode; 24H2/25H2 uses GraphicsDrivers); reboot: yes)_
  - Note: Targeted fix for real MPO flicker/micro-stutter (historically NVIDIA plus mixed-refresh multi-monitor). Disabling overlays raises DWM GPU/power cost and hurts efficiency. Recent GPU drivers fixed most MPO bugs, so update the driver first; the old OverlayTestMode=5 key is ignored on 24H2. Unverified/undocumented by Microsoft. Note DisableOverlays=1 (24H2+) disables ALL hardware overlays, which breaks third-party in-game overlays (Discord/NVIDIA/AMD), a broader effect than just DWM overlays.
- **Set High-Performance GPU Preference per App**: Pin a game to the discrete GPU on hybrid/laptop systems. `reg: HKCU\Software\Microsoft\DirectX\UserGpuPreferences\<exe path> = "GpuPreference=2;" (2 = high perf, 1 = power saving, 0 = auto)` _(low; Win10+11 (multi-GPU / hybrid graphics only); reboot: no)_
- **Enable Variable Refresh Rate for Windowed Games**: Extend adaptive-sync to DX11/borderless titles to reduce tearing. `reg: HKCU\Software\Microsoft\DirectX\UserGpuPreferences\DirectXUserGlobalSettings = "VRROptimizeEnable=1;" (Settings > Display > Graphics > Default graphics settings)` _(low; Win10+11 (VRR-capable display + driver); reboot: no)_
- **Enable Optimizations for Windowed Games**: Route borderless games through DirectFlip for lower latency, Auto HDR and VRR. `toggle 'Optimizations for windowed games' (Settings > System > Display > Graphics); headless equivalent: reg HKCU\Software\Microsoft\DirectX\UserGpuPreferences\DirectXUserGlobalSettings = "SwapEffectUpgradeEnable=1;" (REG_SZ)` _(low; Win11 22H2+; reboot: no)_
  - Note: Applies to borderless/windowed DX10/11 games, bringing latency close to exclusive fullscreen. Rare title-specific glitches; can be toggled per game. No exposed single registry value in current builds, so this is UI/API-driven. Verifier fix: the toggle has existed since Win11 22H2, not 23H2.
- **Disable Xbox Game Bar Background Capture (Game DVR)**: Stop the always-on DVR recorder that adds capture overhead in games. `reg: HKCU\System\GameConfigStore\GameDVR_Enabled = 0 + HKCU\...\GameDVR\AppCaptureEnabled = 0 + HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR\AllowGameDVR = 0` _(low; Win10+11; reboot: no)_
- **Enable the Ultimate Performance Power Plan**: Unlock the low-latency scheme that parks fewer cores and cuts idle transitions. `powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61, then set it active` _(medium; Win10+11 (desktop / AC power); reboot: no)_
  - Note: Keeps the CPU in high P-states and disables core parking, so idle power, heat and fan noise rise; on laptops it hurts battery for little gain. Real benefit is reduced micro-latency/jitter, not higher average FPS. The stock High Performance plan is nearly equivalent.
- **Disable CPU Power Throttling**: Stop Windows from clocking down foreground apps to save power. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling\PowerThrottlingOff (DWORD) = 1 (create the PowerThrottling subkey; 0 = enable, 1 = disable)` _(medium; Win10+11; reboot: yes)_
  - Note: Mainly relevant on laptops/handhelds; increases power draw and heat and can noticeably cut battery life. Marginal on a desktop already on a high-performance plan. Verifier fix: the original path omitted the PowerThrottling subkey, so the value had no effect. Reboot to apply.
- **Disable Multimedia Network Throttling**: Remove the MMCSS 10-packet/ms cap that throttles network throughput during playback. `reg: HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\NetworkThrottlingIndex = 0xffffffff (default 0x0A)` _(medium; Win10+11; reboot: yes)_
  - Note: Only affects the legacy MMCSS packet cap active while audio/video plays; negligible for most modern games and can theoretically hurt A/V smoothness under contention. It is not a latency/ping fix and does not reduce network jitter. Consistency fix: reconciled to medium (a functional trade-off; it protects multimedia/pro-audio smoothness under load). Was duplicated with a low-labeled copy; the duplicate was removed.
- **Disable Storage Sense Auto-Cleanup**: Stop scheduled automatic disk cleanup runs. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\StorageSense\AllowStorageSenseGlobal = 0 (or HKCU\...\StorageSense\Parameters\StoragePolicy\01 = 0)` _(low; Win10+11; reboot: no)_
- **Ensure SSD TRIM / Scheduled Optimization Is Enabled**: Keep TRIM on and let Windows retrim SSDs, never defragment them. `fsutil behavior set DisableDeleteNotify 0 (TRIM on); the dfrgui/defrag schedule issues Retrim (not defrag) on detected SSDs` _(low; Win10+11 (SSD/NVMe); reboot: no)_
- **Disable NTFS Last-Access Timestamp Updates**: Skip metadata writes on every file read to cut disk churn. `fsutil behavior set disablelastaccess 1 (reg: ...\Control\FileSystem\NtfsDisableLastAccessUpdate)` _(low; Win10+11; reboot: no)_
- **Disable RAM Memory Compression**: Turn off page compression to trade RAM for lower CPU/latency on high-RAM systems. `PowerShell: Disable-MMAgent -MemoryCompression (Set-MMAgent -MemoryCompression $false)` _(medium; Win10+11; reboot: yes)_
  - Note: Only sensible with abundant RAM (>=32 GB) that rarely hits pressure. On 8-16 GB it removes a cushion and causes MORE paging to disk / hard faults, which hurts. The 'Memory Compression' process CPU is normally tiny; disable only after proving compression is your bottleneck.
- **Disable Fast Startup (Hiberboot)**: Force a clean cold boot so drivers and kernel state fully re-initialize. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power\HiberbootEnabled = 0` _(low; Win10+11; reboot: yes)_
- **Disable Virtualization-Based Security / Memory Integrity (HVCI)**: Turn off VBS/HVCI to recover CPU-bound FPS and stutter-prone 1% lows. `reg: HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\EnableVirtualizationBasedSecurity = 0 + \Scenarios\HypervisorEnforcedCodeIntegrity\Enabled = 0` _(critical; Win11 (on by default on many 22H2+/24H2 clean installs; Win10 if enabled); reboot: yes)_
  - Note: CRITICAL SECURITY DOWNGRADE. VBS/HVCI is a hypervisor-enforced defense against kernel-mode malware, credential theft (Credential Guard) and malicious/vulnerable drivers; disabling removes it. The FPS gain is real (often big 1%-low/stutter recovery, ~5-15% in CPU-bound titles, more on older CPUs) but only acceptable on a personal rig you accept the risk on. Some anti-cheat and all enterprise/compliance policies require it ON.
- **Disable Spectre / Meltdown CPU Mitigations**: Remove speculative-execution mitigations to reclaim CPU throughput. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\FeatureSettingsOverride = 3 + FeatureSettingsOverrideMask = 3` _(critical; Win10+11 (largest effect on older CPUs lacking silicon fixes); reboot: yes)_
  - Note: CRITICAL SECURITY DOWNGRADE. Re-exposes the machine to Spectre/Meltdown and related side-channel attacks that read memory across process/VM boundaries. On modern CPUs the mitigations are largely hardware-accelerated, so the payoff is small-to-none and some benchmarks even regress. Never do this on a shared, work, or sensitive machine.
- **Disable mouse acceleration (Enhance Pointer Precision)**: Turn off pointer acceleration for 1:1 raw mouse movement, steadier aiming in games. `reg (all REG_SZ under HKCU\Control Panel\Mouse): MouseSpeed = "0", MouseThreshold1 = "0", MouseThreshold2 = "0"` _(low; Win10+11; reboot: yes)_

## Services & Scheduled Tasks

- **Disable Connected User Experiences and Telemetry (DiagTrack)**: Disable the primary telemetry service to stop background diagnostic uploads to Microsoft. `service: DiagTrack -> Disabled (reg: SYSTEM\CurrentControlSet\Services\DiagTrack\Start = 4)` _(medium; Win10+11; reboot: yes)_
  - Note: On Home/Pro the AllowTelemetry=0 policy is silently treated as 1, so disabling the service is the effective lever; breaks Feedback Hub diagnostics and enterprise telemetry-driven features, and feature updates may re-enable it.
- **Disable Print Spooler (Spooler)**: Disable the Print Spooler to remove all printing and close a recurring RCE attack surface (PrintNightmare-class). `service: Spooler -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks ALL printing including Microsoft Print to PDF and Print to XPS, printer enumeration in apps, and some label/receipt software. Only for machines that never print.
- **Disable Fax Service (Fax)**: Disable the legacy Fax service that virtually no modern PC uses. `service: Fax -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks sending faxes via Windows Fax and Scan; document scanning through other apps is unaffected.
- **Disable Program Compatibility Assistant Service (PcaSvc)**: Disable the Program Compatibility Assistant service to stop compat monitoring and its pop-ups. `service: PcaSvc -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: PCA 'this app may not have installed correctly' prompts and automatic compatibility shims stop; Microsoft states no core functional impact, but Win11 24H2 leans on PcaSvc for additional app-compat handling (PcaPatchDbTask).
- **Disable Distributed Link Tracking Client (TrkWks)**: Disable the service that maintains links between NTFS files as they move across volumes. `service: TrkWks -> Disabled` _(low; Win10+11; reboot: yes)_
- **Disable Retail Demo Service (RetailDemo)**: Disable Retail Demo mode used only on in-store display units. `service: RetailDemo -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable Wallet Service (WalletService)**: Disable the service backing Microsoft Wallet and tap-to-pay. `service: WalletService -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks Microsoft Wallet payment features; unrelated to browser-stored cards.
- **Disable Touch Keyboard and Handwriting Service (TabletInputService)**: Disable the touch/handwriting input service on keyboard-and-mouse desktops. `service: TabletInputService -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks the on-screen touch keyboard, handwriting/ink, and the emoji & symbol panel (Win + .). Leave enabled on tablets/2-in-1s or if you use the emoji picker.
- **Disable Bluetooth Support Services (bthserv)**: Disable Bluetooth stack services on desktops with no Bluetooth hardware or need. `services: bthserv, BTAGService, BthAvctpSvc -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks ALL Bluetooth: mice, keyboards, headphones, controllers, and file transfer. Do not use on laptops that rely on BT peripherals.
- **Disable AllJoyn Router Service (AJRouter)**: Disable the AllJoyn router for an IoT interop protocol almost nothing uses. `service: AJRouter -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks AllJoyn IoT device interop (extremely rare); no effect on normal networking.
- **Disable Phone Service (PhoneSvc)**: Disable the service that manages device telephony state. `service: PhoneSvc -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable SSDP Discovery and UPnP Device Host (SSDPSRV/upnphost)**: Disable SSDP/UPnP discovery to shrink the local-network attack surface. `services: SSDPSRV, upnphost -> Disabled` _(medium; Win10+11; reboot: no)_
  - Note: Breaks UPnP/DLNA device discovery, 'Cast to Device' media casting, and some smart-home/router auto-config. Leave on if you cast to TVs or stream media.
- **Disable Windows Insider Service (wisvc)**: Disable the Insider service on machines not enrolled in the Windows Insider Program. `service: wisvc -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable Downloaded Maps Manager (MapsBroker)**: Disable the offline Maps broker if you never use offline maps. `service: MapsBroker -> Disabled` _(low; Win10+11; reboot: yes)_
  - Note: Breaks offline map downloads and apps that use the Windows Maps platform.
- **Disable Geolocation Service (lfsvc)**: Disable the system geolocation service to stop OS-wide location resolution. `service: lfsvc -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks location for all apps, Find My Device, automatic time-zone, and location-based weather; equivalent to hard-off for the Location privacy toggle.
- **Disable Xbox Live Services (XblAuthManager et al.)**: Disable Xbox Live services on PCs that do not game through Xbox or PC Game Pass. `services: XblAuthManager, XblGameSave, XboxNetApiSvc, XboxGipSvc -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks the Xbox app, PC Game Pass, cloud game saves, Game Bar sign-in/captures, and any title that requires Xbox Live networking. Verifier warning: keep the Xbl* trio, but XboxGipSvc (Xbox Accessory Management) drives Xbox controller input in ANY launcher (Steam/Epic included), so disabling it can break Xbox controllers system-wide. Split it out or warn controller users.
- **Disable Connected Devices Platform Service (CDPSvc)**: Disable the cross-device platform service to stop background sync and device-linking chatter. `service: CDPSvc -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks Nearby Sharing, Phone Link, shared clipboard, and 'resume on other devices'; also curbs some background CDP telemetry.
- **Disable Windows Biometric Service (WbioSrvc)**: Disable the biometric service on machines that do not use Windows Hello fingerprint or face. `service: WbioSrvc -> Disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Breaks Windows Hello fingerprint and facial sign-in; leave enabled if you use any biometric login.
- **Disable Smart Card Services (SCardSvr)**: Disable smart-card services on consumer PCs with no smart-card reader. `services: SCardSvr, ScDeviceEnum, SCPolicySvc -> Disabled` _(medium; Win10+11; reboot: no)_
  - Note: Breaks smart-card / CAC / PIV / YubiKey-PIV logon and some enterprise VPN auth. Do not disable in managed or enterprise environments. Safety fix: raised to medium. On any machine using smart-card / CAC / PIV / YubiKey-PIV logon this removes the only sign-in path and can lock the user out (not 'easily reversible').
- **Disable Sensor Services (SensorService)**: Disable sensor services on desktops with no ambient-light or orientation sensors. `services: SensorService, SensrSvc, SensorDataService -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks auto-brightness, adaptive screen rotation, and sensor-aware apps on laptops/tablets; safe on sensorless desktops.
- **Disable Family Safety Monitor (WpcMonSvc)**: Disable the Family Safety monitor service if you do not use Microsoft Family features. `service: WpcMonSvc -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks Microsoft Family Safety screen-time and content limits; do not disable on managed child accounts.
- **Disable Payments and NFC/SE Manager (SEMgrSvc)**: Disable the NFC secure-element manager on devices without NFC payments. `service: SEMgrSvc -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks NFC tap-to-pay and secure-element wallet features; no effect on typical desktops.
- **Disable Windows Media Player Network Sharing (WMPNetworkSvc)**: Disable DLNA media serving from Windows Media Player. `service: WMPNetworkSvc -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: Breaks streaming your WMP library to DLNA/UPnP renderers; on Windows 11 the service is usually absent unless legacy WMP is installed. Verifier fix: WMPNetworkSvc still ships on Win11 (it backs Windows Media Player Legacy), present with a Manual/trigger start type.
- **Disable Autochk Proxy task**: Disable the task that uploads disk-check SQM/CEIP data. `task: \Microsoft\Windows\Autochk\Proxy -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable Feedback (SIUF) DmClient tasks**: Disable the Windows Feedback tasks that transmit feedback and diagnostic data. `tasks: \Microsoft\Windows\Feedback\Siuf\DmClient, \Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable Windows Error Reporting QueueReporting task**: Disable the task that uploads queued crash reports to Microsoft. `task: \Microsoft\Windows\Windows Error Reporting\QueueReporting -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable offline Maps update tasks**: Disable the background tasks that download and toast offline map updates. `tasks: \Microsoft\Windows\Maps\MapsUpdateTask, \Microsoft\Windows\Maps\MapsToastTask -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable DiskDiagnosticDataCollector task**: Disable the task that sends SMART disk data to Microsoft under CEIP. `task: \Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector -> Disabled` _(low; Win10+11; reboot: no)_
- **Disable Devicecensus task**: Disable the task that inventories the device for Microsoft telemetry and update targeting. `task: \Microsoft\Windows\Device Information\Devicecensus -> Disabled` _(low; Win10+11; reboot: no)_

## UI/UX, Explorer & Taskbar

- **Enable dark mode**: Switch Windows apps and system UI to the dark color theme. `reg: Themes\Personalize\AppsUseLightTheme = 0 and SystemUsesLightTheme = 0 (HKCU)` _(low; Win10+11; reboot: no)_
- **Disable transparency effects**: Turn off acrylic/Mica transparency for a flat, opaque UI. `reg: Themes\Personalize\EnableTransparency = 0 (HKCU)` _(low; Win10+11; reboot: no)_
- **Disable UI animations**: Turn off window and taskbar animations for a snappier-feeling desktop. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarAnimations = 0 (DWORD) + HKCU\Control Panel\Desktop\WindowMetrics\MinAnimate = "0" (REG_SZ). Do NOT scalar-write UserPreferencesMask (it is a REG_BINARY packed bitmask).` _(low; Win10+11; reboot: no)_
  - Note: Perceived snappiness only, not a frame-rate boost; restart Explorer to apply. Round-2 value fix: MinAnimate is a REG_SZ under ...\Desktop\WindowMetrics (not a DWORD under Desktop), and UserPreferencesMask is a REG_BINARY bitmask, so a scalar =0 is invalid and clobbers unrelated UI prefs; clear only the relevant bit if editing it. Restart Explorer to apply.
- **Disable Aero Shake**: Stop title-bar shaking from minimizing every other window by accident. `reg: Explorer\Advanced\DisallowShaking = 1 (HKCU); or policy Explorer\NoWindowMinimizingShortcuts = 1` _(low; Win10+11; reboot: no)_
- **Left-align the taskbar**: Move the Windows 11 taskbar icons and Start button to the left edge. `reg: Explorer\Advanced\TaskbarAl = 0 (HKCU)` _(low; Win11 only; reboot: no)_
- **Restore classic context menu (Win11)**: Bring back the full Windows 10 right-click menu, skipping 'Show more options'. `reg: HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32 default = "" (empty string)` _(low; Win11 only; reboot: no)_
- **Hide Task View button**: Remove the Task View button from the taskbar. `reg: Explorer\Advanced\ShowTaskViewButton = 0 (HKCU)` _(low; Win10+11; reboot: no)_
- **Hide Chat/Teams button**: Remove the consumer Chat (Teams) button from the taskbar. `reg: Explorer\Advanced\TaskbarMn = 0 (HKCU)` _(low; Win11; reboot: no)_
- **Hide the Copilot taskbar button**: Remove the Windows Copilot button and hotkey from the taskbar. `reg: HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot\TurnOffWindowsCopilot = 1` _(low; Win10+11 (pre-24H2 hides Copilot; on 24H2 hides only the taskbar button and Win+C); reboot: no)_
  - Note: On 24H2 Copilot is a normal Store app: unpin or uninstall the appx instead; the old Explorer\Advanced\ShowCopilotButton value no longer applies. Verifier fix: on 24H2 TurnOffWindowsCopilot only removes the taskbar button and blocks Win+C; Copilot is a standalone app still launchable from Search/Start/Edge. To actually remove it, uninstall the Microsoft.Copilot appx (see the Debloat 'Remove the Copilot app' entry).
- **Set taskbar search style**: Show the taskbar search as hidden, icon only, or a full search box. `reg: Search\SearchboxTaskbarMode (HKCU): 0 hidden, 1 icon, 2 box, 3 icon+label` _(low; Win10+11; reboot: no)_
- **Ungroup taskbar buttons + show labels**: Never combine taskbar buttons and show window text labels. `reg: Explorer\Advanced\TaskbarGlomLevel = 2 (HKCU)` _(low; Win10 + Win11 23H2+; reboot: no)_
- **Enable taskbar End Task**: Add 'End Task' to the taskbar right-click menu to force-kill hung apps. `reg: Explorer\Advanced\TaskbarDeveloperSettings\TaskbarEndTask = 1 (HKCU)` _(low; Win11 23H2+; reboot: no)_
- **Show seconds in tray clock**: Display seconds on the system-tray clock. `reg: Explorer\Advanced\ShowSecondsInSystemClock = 1 (HKCU)` _(low; Win10+11; reboot: no)_
- **Adjust taskbar hover delay**: Shorten the delay before taskbar thumbnail/hover previews appear. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\ExtendedUIHoverTime (DWORD, milliseconds; default 400, set 1 for instant taskbar thumbnail preview)` _(low; Win10+11; reboot: no)_
  - Note: Very low values make previews feel twitchy; re-logon or restart Explorer to apply. Verifier fix: the taskbar thumbnail delay is ExtendedUIHoverTime, not Mouse\MouseHoverTime (the general hover timer, coincidentally also 400). Requires an Explorer restart or sign-out.
- **Disable snap-layouts flyout**: Stop the snap-layouts flyout appearing on the Maximize button. `reg: Explorer\Advanced\EnableSnapAssistFlyout = 0 (HKCU)` _(low; Win11; reboot: no)_
- **Open Explorer to This PC**: Open File Explorer to This PC instead of Home/Quick Access. `reg: Explorer\Advanced\LaunchTo = 1 (HKCU)` _(low; Win10+11; reboot: no)_
- **Show file extensions**: Always show known file extensions to spot disguised .exe/.scr files. `reg: Explorer\Advanced\HideFileExt = 0 (HKCU)` _(low; Win10+11; reboot: no)_
- **Show hidden files**: Show hidden files and folders in File Explorer. `reg: Explorer\Advanced\Hidden = 1 (HKCU)` _(low; Win10+11; reboot: no)_
- **Enable Explorer compact view**: Use tighter row spacing in File Explorer lists. `reg: Explorer\Advanced\UseCompactMode = 1 (HKCU)` _(low; Win11 only; reboot: no)_
- **Show full path in Explorer**: Show the full folder path in the File Explorer title/address bar. `reg: Explorer\CabinetState\FullPath = 1 (HKCU)` _(low; Win10+11; reboot: no)_
- **Disable recent files in Explorer**: Hide recent files and frequent folders from Explorer Home/Quick Access. `reg: Explorer\Advanced\ShowRecent = 0 and ShowFrequent = 0 (HKCU)` _(low; Win10+11; reboot: no)_
- **Stop Start tracking recent items**: Stop Start Recommended and jump lists from listing recently opened items. `reg: Explorer\Advanced\Start_TrackDocs = 0 (HKCU)` _(low; Win10+11; reboot: no)_
- **Remove Gallery from Explorer**: Remove the Gallery entry from the File Explorer navigation pane. `reg: HKCU\Software\Classes\CLSID\{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}\System.IsPinnedToNameSpaceTree = 0` _(low; Win11 23H2+; reboot: no)_
  - Note: Win11 22H2+; restart Explorer to apply. Consistency fix: the File Explorer Gallery node shipped on 23H2; applies_to reconciled to Win11 23H2+ (a duplicate copy claimed 22H2+).
- **Remove Home from Explorer**: Remove the Home entry from the File Explorer navigation pane. `reg: HKCU\Software\Classes\CLSID\{f874310e-b6b7-47dc-bc84-b9e6b38f5903}\System.IsPinnedToNameSpaceTree = 0` _(low; Win11; reboot: no)_
- **Hide OneDrive from Explorer**: Hide the OneDrive icon from the File Explorer navigation pane. `reg: HKCU\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}\System.IsPinnedToNameSpaceTree = 0` _(low; Win10+11; reboot: no)_
- **Disable all toast notifications**: Silence every app toast notification and banner system-wide. `reg: PushNotifications\ToastEnabled = 0 (HKCU)` _(medium; Win10+11; reboot: yes)_
  - Note: Suppresses ALL notifications including security, calendar and messaging alerts; sign out to apply. Prefer Focus/Do-Not-Disturb for selective silencing.
- **Hide Cortana button (Win10)**: Remove the Cortana button from the Windows 10 taskbar. `reg: Explorer\Advanced\ShowCortanaButton = 0 (HKCU)` _(low; Win10 only; reboot: no)_
- **Disable Meet Now**: Remove the Meet Now (Skype) icon from the system tray. `reg: HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\HideSCAMeetNow = 1` _(low; Win10 only; reboot: no)_
- **Disable People bar (Win10)**: Remove the People (My People) button from the Windows 10 taskbar. `reg: Explorer\Advanced\PeopleBand = 0 (HKCU); or policy Explorer\HidePeopleBar = 1` _(low; Win10 only; reboot: no)_
- **Disable News and Interests (Win10)**: Turn off the News and Interests weather/feed widget on the Win10 taskbar. `reg: Feeds\ShellFeedsTaskbarViewMode = 2 (HKCU); or policy Windows Feeds\EnableFeeds = 0` _(low; Win10 only; reboot: no)_
- **Enable verbose logon messages**: Show detailed status text during startup, shutdown, logon and logoff. `reg: HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\VerboseStatus = 1` _(low; Win10+11; reboot: yes)_
- **NumLock on at startup**: Turn NumLock on automatically at boot and the sign-in screen. `reg: InitialKeyboardIndicators = 2147483650 (0x80000002, REG_SZ) under HKU\.DEFAULT\Control Panel\Keyboard AND the live user's HKCU\Control Panel\Keyboard` _(low; Win10+11; reboot: yes)_
  - Note: Fast Startup can override the state; value 2 is more reliable than 2147483650 on some machines. Verifier fix: value 2 alone is unreliable because Fast Startup restores keyboard state from the hiberfile; use 2147483650, set it under both HKU\.DEFAULT and HKCU, and note it may need Fast Startup disabled to hold.
- **Disable startup sound**: Turn off the Windows startup/logon chime. `reg: HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI\BootAnimation\DisableStartupSound = 1` _(low; Win10+11; reboot: yes)_
- **Disable accessibility key-shortcut prompts**: Stop the Sticky Keys (5x Shift), Filter Keys, and Toggle Keys activation popups. `reg (all REG_SZ): HKCU\Control Panel\Accessibility\StickyKeys\Flags = "506", HKCU\Control Panel\Accessibility\Keyboard Response\Flags = "122", HKCU\Control Panel\Accessibility\ToggleKeys\Flags = "58"` _(low; Win10+11; reboot: no)_

## Network, Windows Update & Power

- **DNS over HTTPS**: Route DNS lookups over encrypted HTTPS to stop plaintext queries and ISP/on-path snooping. `reg: SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\EnableAutoDoh = 2 (auto-upgrade known resolvers); per-server DohFlags under ...\DohInterfaceSettings` _(medium; Win11 only; reboot: yes)_
  - Note: Windows' built-in DoH client only exists on Win11 (Win10 22H2 has none). Only upgrades resolvers Windows recognizes as DoH-capable; can break captive-portal sign-in and split-horizon/corporate internal DNS. Unverified: EnableAutoDoh=2 is community-documented only (no official Microsoft source) and only auto-upgrades resolvers Windows recognizes as DoH-capable, so with an ordinary resolver it encrypts nothing. Prefer documented per-server DohInterfaceSettings/DohFlags and pair with a DoH-capable resolver. Win11 only.
- **Disable LLMNR**: Turn off multicast LLMNR name resolution to block a common credential-theft/spoofing vector. `reg: SOFTWARE\Policies\Microsoft\Windows NT\DNSClient\EnableMulticast = 0 ('Turn off multicast name resolution')` _(medium; Win10+11; reboot: no)_
  - Note: Security hardening that blocks LLMNR poisoning (Responder-style attacks). May break single-label name resolution on flat home/SOHO networks that rely on LLMNR instead of a DNS server.
- **Disable NetBIOS over TCP/IP**: Disable legacy NetBIOS name service (NBT-NS) to close a spoofing/poisoning attack surface. `reg: SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces\Tcpip_{GUID}\NetbiosOptions = 2 (disable NBT per adapter)` _(medium; Win10+11; reboot: yes)_
  - Note: Security hardening paired with LLMNR-off. Can break discovery/file sharing with very old NetBIOS-only devices and a few legacy line-of-business apps.
- **Disable WPAD auto-proxy discovery**: Disable Web Proxy Auto-Discovery so attackers can't hijack traffic via rogue WPAD responses. `reg: HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings\WinHttp\DisableWpad (DWORD) = 1 (machine-wide disable, since Win10 1809)` _(medium; Win10+11; reboot: yes)_
  - Note: Security hardening. Breaks environments that legitimately use WPAD for automatic proxy config; set the proxy manually there. Verifier fix: do NOT disable the WinHttpAutoProxySvc service (Microsoft warns against it; many apps depend on it and it can wedge update/store/proxy detection). Use the DisableWpad value instead.
- **Disable IPv6 transition technologies**: Disable Teredo/6to4/ISATAP tunnels to remove IPv6 tunneling attack surface and address leaks. `reg: SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters\DisabledComponents = 0x01 (disable all tunnel interfaces); or netsh interface teredo|6to4|isatap set state disabled` _(medium; Win10+11; reboot: yes)_
  - Note: Disable only the tunnel interfaces (0x01), NOT native IPv6. Setting 0xFF to kill IPv6 entirely is unsupported by Microsoft and breaks features. May affect apps that depend on Teredo (some legacy P2P/older Xbox scenarios).
- **Disable NIC power management**: Stop Windows powering down the network adapter to prevent Wi-Fi drops and idle latency spikes. `reg: SYSTEM\...\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}\<NNNN>\PnPCapabilities = 24 (clears 'Allow the computer to turn off this device')` _(medium; Win10+11; reboot: yes)_
  - Note: Increases idle power draw on laptops. Per-adapter: the correct 4-digit subkey must be located via the adapter's driver key. Real fix for adapters that disconnect when idle. Unverified: PnPCapabilities=24 is community-sourced and inconsistent across adapter drivers, with undocumented bit semantics. Prefer the documented Set-NetAdapterPowerManagement cmdlet, or verify per-adapter that 24 clears 'Allow the computer to turn off this device'.
- **Disable Delivery Optimization P2P**: Stop peer-to-peer update sharing so your PC isn't uploading/downloading updates to other machines. `reg: SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization\DODownloadMode = 0 (HTTP only, no peering)` _(low; Win10+11; reboot: no)_
  - Note: Set DODownloadMode=0; do NOT disable the DoSvc service itself, which breaks Microsoft Store and some Windows Update downloads. Legacy mode 100 is deprecated and errors (0x80d03002).
- **Defer quality updates**: Hold back monthly quality (security) updates for a set number of days on Pro/Edu/Ent editions. `reg: SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\DeferQualityUpdates = 1 + DeferQualityUpdatesPeriodInDays = 0-30` _(medium; Win10+11 Pro/Edu/Ent; reboot: no)_
  - Note: Delays security patches up to the 30-day max. Ignored on Home. On 24H2 clean installs the Group Policy UI may be missing even though the policy key is still honored on Pro.
- **Pin Windows feature version**: Lock the PC to a specific feature version (e.g. 24H2) so it won't jump to a newer release. `reg: SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\TargetReleaseVersion = 1 + TargetReleaseVersionInfo = "24H2" + ProductVersion = "Windows 11"` _(medium; Win10+11 Pro/Edu/Ent; reboot: no)_
  - Note: This is the supported replacement for the deprecated 'defer feature updates by N days' control. Clear the pin before the pinned version reaches end-of-servicing or the device stops getting security updates. Home ignores it.
- **Set active hours**: Tell Windows when you use the PC so automatic update restarts happen outside those hours. `reg: HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings - SmartActiveHoursState (DWORD) = 2 to disable auto-adjust (1 = on), plus ActiveHoursStart and ActiveHoursEnd (DWORD, hours; span <= 18).` _(low; Win10+11; reboot: no)_
  - Note: Not a 'never restart' switch: Windows can still restart after a deadline or outside the window. Maximum active-hours span is 18 hours. Round-2 value fix: the disable value for SmartActiveHoursState is 2 (1 = on, 2 = off), not 0; with 0 Smart Active Hours stays on and Windows overrides the manual ActiveHoursStart/End.
- **Block auto-restart while signed in**: Prevent Windows Update from automatically rebooting the PC while a user is logged on. `reg: SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU\NoAutoRebootWithLoggedOnUsers = 1` _(medium; Win10+11; reboot: no)_
  - Note: Can postpone security-update reboots indefinitely if the user never signs out, leaving patches un-applied. Reliable on Pro; consumer Home honors it inconsistently.
- **Exclude driver updates from Windows Update**: Stop Windows Update from delivering hardware drivers so you control driver versions yourself. `reg: SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\ExcludeWUDriversInQualityUpdate = 1` _(medium; Win10+11; reboot: no)_
  - Note: You then miss driver security/stability fixes shipped via WU; update GPU/chipset drivers manually. Works on Home via the registry key (no gpedit needed).
- **Disable automatic driver installation**: Block Windows from auto-installing device drivers and fetching device metadata over the network. `reg: SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching\SearchOrderConfig = 0 + ...\Device Metadata\PreventDeviceMetadataFromNetwork = 1` _(medium; Win10+11; reboot: yes)_
  - Note: New/unknown hardware may lack drivers until you install them manually. Complements ExcludeWUDriversInQualityUpdate for full driver control.
- **Disable Microsoft Store auto-updates**: Stop the Microsoft Store from automatically downloading and installing app updates. `reg: SOFTWARE\Policies\Microsoft\WindowsStore\AutoDownload = 2 (always off; 4 = always on to revert)` _(medium; Win10+11; reboot: no)_
  - Note: Store apps, including ones that ship security fixes, go stale until you update them manually from the Store's Library page.
- **Block auto-download over metered**: Prevent Windows Update from auto-downloading updates over connections marked metered. `reg: SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AllowAutoWindowsUpdateDownloadOverMeteredNetwork = 0` _(medium; Win10+11; reboot: no)_
  - Note: Not a permanent update block: high-priority/critical updates can still download over metered. Only effective once the connection is actually set as metered.
- **Notify before downloading updates**: Switch Windows Update to notify-only so updates don't download or install without your approval. `reg: HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU\NoAutoUpdate = 0 and AUOptions = 2 (notify before download and install); AUOptions is only honored when NoAutoUpdate = 0` _(high; Win10+11 Pro/Edu/Ent; reboot: no)_
  - Note: Leaves the system unpatched until you manually update, a real security exposure. Windows 11 consumer builds increasingly ignore/override this; reliable only on Pro/Edu/Ent. Verifier fix: NoAutoUpdate=1 disables Automatic Updates entirely and makes AUOptions ignored, so the original NoAutoUpdate=1 + AUOptions=2 was contradictory. Windows Home ignores the WU AU policy.
- **Disable hibernation**: Remove hibernation and delete hiberfil.sys to reclaim disk space (also disables Fast Startup). `powercfg: /hibernate off (deletes hiberfil.sys, roughly 40% of installed RAM in size)` _(medium; Win10+11; reboot: no)_
  - Note: Also disables Fast Startup and the sleep-to-disk/Hibernate option; laptops lose that battery-safety fallback. Re-enable with 'powercfg /hibernate on'.
- **Disable USB selective suspend**: Stop Windows suspending idle USB ports to fix device dropouts, audio glitches and slow wakeups. `powercfg: SETACVALUEINDEX/SETDCVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0` _(medium; Win10+11; reboot: no)_
  - Note: Increases idle power draw on laptops. Fixes real problems with USB DACs, dongles and flaky hubs; leave it enabled if you have no such symptoms.
- **Disable wake timers**: Block scheduled tasks from waking the PC from sleep (stops overnight update/maintenance wakes). `powercfg: SETACVALUEINDEX/SETDCVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 0` _(low; Win10+11; reboot: no)_
- **Disable Modern Standby (force S3)**: Force legacy S3 sleep instead of Modern Standby (S0) to stop in-bag battery drain and overheating. `reg: SYSTEM\CurrentControlSet\Control\Power\PlatformAoAcOverride = 0` _(high; Win10+11; reboot: yes)_
  - Note: CRITICAL: only safe if firmware still exposes S3. Run 'powercfg /a' first; if 'Standby (S3)' is not listed, this can leave the machine unable to sleep/wake at all. Many modern laptops (and some platforms on 24H2) provide no S3 fallback. Unverified: PlatformAoAcOverride=0 is undocumented, and it is a no-op or harmful on modern laptops whose firmware exposes no S3 at all (leaving no working sleep). Gate on 'powercfg /a' listing Standby (S3) before applying.

## Security Hardening

- **Disable Remote Registry service**: Disable the Remote Registry service to block remote editing of this PC's registry. `service: RemoteRegistry -> Disabled` _(low; Win10+11; reboot: no)_
  - Note: HARDENING. Breaks remote management tools that read this machine's registry (rare on personal PCs); the service is already Manual/stopped by default, so this pins the safe state.
- **Disable Remote Desktop (RDP)**: Turn off inbound Remote Desktop connections to shrink the remote-attack surface. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server\fDenyTSConnections = 1` _(low; Win10+11; reboot: no)_
  - Note: HARDENING. You lose the ability to RDP into this PC; already the default on Home and most fresh installs.
- **Remove SMBv1 protocol**: Remove the legacy SMBv1 file-sharing protocol exploited by WannaCry/EternalBlue. `optional-feature: SMB1Protocol -> Disabled; reg: LanmanServer\Parameters\SMB1 = 0` _(low; Win10+11 (already absent on fresh Win11 and Win10 1709+); reboot: yes)_
- **Disable WDigest credential caching**: Pin UseLogonCredential=0 so plaintext passwords are not kept in LSASS memory. `reg: HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\WDigest\UseLogonCredential = 0` _(low; Win10+11; reboot: no)_
- **Enable LSA protection (RunAsPPL)**: Run LSASS as a protected process to block credential-dumping tools like Mimikatz. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Lsa\RunAsPPL = 1 (2 = enable without UEFI lock)` _(medium; Win10+11 (default-on for clean Win11 22H2+ enterprise-joined installs); reboot: yes)_
  - Note: HARDENING. Can block unsigned LSA/authentication plugins and some third-party security, smartcard, or VPN products; test first. Enable Secure Boot so the setting resists bypass.
- **Enforce NTLMv2 only**: Set LmCompatibilityLevel=5 to send/accept NTLMv2 only and refuse weak LM and NTLMv1. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Lsa\LmCompatibilityLevel = 5` _(medium; Win10+11; reboot: yes)_
  - Note: HARDENING. Very old devices/appliances that only speak LM/NTLMv1 can no longer authenticate; NTLMv1 is removed outright in Win11 24H2, so this is increasingly the enforced baseline.
- **Require SMB signing**: Require SMB client and server signing to defeat SMB relay and man-in-the-middle attacks. `reg: LanmanWorkstation\Parameters\RequireSecuritySignature = 1 and LanmanServer\Parameters\RequireSecuritySignature = 1` _(medium; Win10+11 (default-on in Win11 24H2 and Server 2025); reboot: yes)_
  - Note: HARDENING. Old NAS/USB-router shares that cannot sign will fail to connect; already the enforced default in Win11 24H2. Overlaps the network domain.
- **Harden RDP (NLA + TLS)**: Require Network Level Authentication and TLS for RDP to block pre-auth and downgrade attacks. `reg: Terminal Server\WinStations\RDP-Tcp\UserAuthentication = 1, SecurityLayer = 2` _(low; Win10+11; reboot: no)_
- **Disable administrative shares (C$, ADMIN$)**: Stop auto-creation of hidden admin shares to hamper lateral movement. `reg: LanmanServer\Parameters\AutoShareWks = 0` _(medium; Win10+11; reboot: yes)_
  - Note: HARDENING (defense in depth). Breaks remote admin tools, some backup software, and PsExec/remote-WMI workflows that rely on C$/ADMIN$; an admin can recreate them, so this is not a hard boundary.
- **Prevent LM hash storage**: Set NoLMHash=1 so weak LAN Manager password hashes are never stored on disk. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Lsa\NoLMHash = 1` _(low; Win10+11 (default-on); reboot: yes)_
- **Restrict anonymous enumeration**: Block anonymous listing of SAM accounts and shares to blunt reconnaissance. `reg: Lsa\RestrictAnonymousSAM = 1, Lsa\RestrictAnonymous = 1, Lsa\EveryoneIncludesAnonymous = 0` _(medium; Win10+11; reboot: yes)_
  - Note: HARDENING. RestrictAnonymous=1 is safe; avoid the legacy '=2' which breaks networking/trusts (see myths). Some old cross-domain or appliance scenarios rely on anonymous access.
- **Reduce cached domain logons**: Lower the cached logon count to limit offline credential-cracking exposure. `reg: HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\CachedLogonsCount = "4" (REG_SZ, quoted digits; range 0-50, 0 disables caching). Winlogon reads this as a STRING, not a DWORD.` _(medium; Win10+11 (domain-relevant); reboot: no)_
  - Note: HARDENING. Setting 0 blocks logon whenever a domain controller is unreachable (offline laptops locked out); pick 1-4 as a balance. Irrelevant to non-domain home PCs. Round-2 value fix: CachedLogonsCount is REG_SZ, not DWORD; a DWORD is silently ignored and the limit never applies. Reboot to apply.
- **Enable Microsoft vulnerable-driver blocklist**: Turn on the HVCI vulnerable-driver blocklist to stop bring-your-own-vulnerable-driver (BYOVD) attacks. `reg: HKLM\SYSTEM\CurrentControlSet\Control\CI\Config\VulnerableDriverBlocklistEnable = 1` _(low; Win10+11 (default-on since Win11 22H2 / KB5018482); reboot: yes)_
- **Disable WPBT vendor binary execution**: Block firmware-provided WPBT binaries from auto-running to stop OEM/rootkit persistence. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\DisableWpbtExecution = 1` _(medium; Win10+11; reboot: yes)_
  - Note: HARDENING. Some OEMs use WPBT for legitimate driver or anti-theft delivery; disabling can stop those from reinstalling after a wipe. Unverified: DisableWpbtExecution=1 is corroborated only by community/security-research sources (Eclypsium et al.); Microsoft's own WPBT off-switch is a DFCI firmware setting via Intune. The key appears correct and functional.
- **Require Ctrl+Alt+Del at sign-in**: Force the secure attention sequence to defeat credential-harvesting fake login screens. `reg: SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\DisableCAD = 0` _(low; Win10+11; reboot: no)_
- **Disable AutoRun/AutoPlay on all drives**: Block automatic execution from USB/optical media to stop autorun-worm infection. `reg: SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\NoDriveTypeAutoRun = 255, NoAutorun = 1` _(low; Win10+11; reboot: no)_
- **Enable PowerShell script-block logging**: Log deobfuscated PowerShell to the event log for malware forensics and detection. `reg: SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging\EnableScriptBlockLogging = 1` _(low; Win10+11; reboot: no)_
- **Raise UAC to always notify (secure desktop)**: Set UAC to always prompt on the secure desktop for the strongest elevation control. `reg: Policies\System\ConsentPromptBehaviorAdmin = 2, PromptOnSecureDesktop = 1, EnableLUA = 1` _(low; Win10+11; reboot: no)_
- **Apply UAC to built-in Administrator**: Enable Admin Approval Mode for the built-in Administrator so even it gets UAC prompts. `reg: Policies\System\FilterAdministratorToken = 1` _(low; Win10+11; reboot: yes)_
- **Hide last signed-in username**: Stop the sign-in screen from displaying the last username to limit account disclosure. `reg: Policies\System\DontDisplayLastUserName = 1` _(low; Win10+11; reboot: no)_
- **Restrict printer-driver install to admins (PrintNightmare)**: Require admin rights to install print drivers, closing the PrintNightmare (CVE-2021-34527) RCE path. `reg: SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint\RestrictDriverInstallationToAdministrators = 1` _(medium; Win10+11; reboot: no)_
  - Note: HARDENING. Standard users can no longer add printers needing a new driver (an admin must). Microsoft states no other combination of mitigations equals this setting.
- **Disable Remote Assistance**: Turn off inbound Remote Assistance invitations to reduce remote-help attack surface. `reg: HKLM\SYSTEM\CurrentControlSet\Control\Remote Assistance\fAllowToGetHelp = 0` _(low; Win10+11; reboot: no)_
- **Disable Windows Script Host**: Block .vbs/.js/.wsf execution to shut down a common script-malware vector. `reg: HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings\Enabled = 0` _(medium; Win10+11; reboot: no)_
  - Note: HARDENING. Breaks legitimate .vbs/.js scripts, some installers, and logon scripts; test if you rely on scripting. High payoff against script droppers.
- **Enable Controlled Folder Access (ransomware shield)**: Turn on Defender Controlled Folder Access to block unauthorized apps writing to protected folders. `reg: ...\Windows Defender\Windows Defender Exploit Guard\Controlled Folder Access\EnableControlledFolderAccess = 1 (or Set-MpPreference -EnableControlledFolderAccess Enabled)` _(medium; Win10+11 (Microsoft Defender required); reboot: no)_
  - Note: HARDENING. Produces false positives; legitimate apps (games, editors, backup tools) may be blocked from writing to Documents/Pictures until you whitelist them.
- **Enable Defender Network Protection**: Block outbound connections to malicious domains/IPs at OS level via Defender Network Protection. `reg: ...\Windows Defender\Windows Defender Exploit Guard\Network Protection\EnableNetworkProtection = 1 (or Set-MpPreference -EnableNetworkProtection Enabled)` _(low; Win10+11 (Microsoft Defender required); reboot: no)_
- **Enable PUA/PUP protection**: Turn on Defender potentially-unwanted-app blocking to stop bundleware and adware. `reg: ...\Windows Defender\MpEngine\MpEnablePus = 1 (or Set-MpPreference -PUAProtection Enabled)` _(low; Win10+11 (Microsoft Defender required); reboot: no)_
- **ASR rule: block LSASS credential theft**: Enable the Defender ASR rule that blocks credential theft from LSASS (Mimikatz-class) `ASR GUID: 9e6c4e1f-7d60-472f-ba1a-a39ef669e4b0 -> 1 (Block) via Set-MpPreference/policy` _(medium; Win10+11 (Microsoft Defender required); reboot: no)_
  - Note: HARDENING. Redundant if LSA protection + Credential Guard are on, and can block legitimate IT/security tools that read LSASS; requires Defender real-time protection. Safe to deploy in Block without audit per Microsoft.
- **ASR rules: block Office/script malware vectors**: Enable Defender ASR rules blocking macro child-processes, obfuscated scripts, and email executables. `ASR GUIDs: d4f940ab-401b-4efc-aadc-ad5f3c50688a (Office child process), 3b576869-a4ec-4529-8536-b80a7769e899 (Office exe content), 5beb7efe-fd9a-4556-801d-275e5ffc04cc (obfuscated scripts), be9ba2d9-53ea-4cdc-84e5-9b1eeee46550 (email exe) -> 1 (Block)` _(medium; Win10+11 (Microsoft Defender required); reboot: no)_
  - Note: HARDENING. Can block legitimate macro-heavy business documents and admin scripts; deploy in Audit (2) first if you rely on Office automation. Requires Defender real-time protection.
- **Enforce SmartScreen (apps and Edge)**: Set SmartScreen to warn/block for downloaded apps and Edge browsing to catch malicious files/sites. `reg: Policies\Microsoft\Windows\System\EnableSmartScreen = 1, ShellSmartScreenLevel = Block; Edge SmartScreenEnabled = 1` _(low; Win10+11; reboot: no)_
- **Disable legacy TLS 1.0/1.1 (Schannel)**: Disable weak TLS 1.0/1.1 system-wide so apps negotiate TLS 1.2/1.3. `reg: SCHANNEL\Protocols\TLS 1.0\{Client,Server}\Enabled = 0 and TLS 1.1\{Client,Server}\Enabled = 0` _(medium; Win10+11; reboot: yes)_
  - Note: HARDENING. Very old servers/appliances offering only TLS 1.0/1.1 become unreachable; these protocols are already off by default in modern browsers and deprecated in Win11 24H2.
- **Force .NET strong crypto (TLS 1.2+)**: Set SchUseStrongCrypto so legacy .NET apps use TLS 1.2+ instead of SSL3/TLS1.0. `reg: SOFTWARE\Microsoft\.NETFramework\v4.0.30319\SchUseStrongCrypto = 1 (and Wow6432Node)` _(low; Win10+11; reboot: no)_
- **Disable SMB insecure guest logons**: Reject SMB guest fallback so shares require real authentication (blocks rogue-server MITM) `reg: SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation\AllowInsecureGuestAuth = 0` _(low; Win10+11 (default-on Win11 Pro/Ent build 25267+ and 24H2); reboot: no)_
- **Remove PowerShell 2.0 engine**: Uninstall the legacy PowerShell 2.0 engine that bypasses modern logging and AMSI. `optional-feature: MicrosoftWindowsPowerShellV2Root / MicrosoftWindowsPowerShellV2 -> Disabled` _(low; Win10+11; reboot: yes)_
  - Note: HARDENING. Extremely rare legacy scripts that force -Version 2 will break; removes a well-known AMSI/logging-downgrade attack path.
- **Enable firewall on all profiles (block inbound)**: Ensure Windows Firewall is on for Domain/Private/Public with default-block inbound. `reg: ...\WindowsFirewall\{DomainProfile,StandardProfile,PublicProfile}\EnableFirewall = 1, DefaultInboundAction = 1 (Block)` _(low; Win10+11; reboot: no)_
- **Enable logon/credential auditing**: Turn on success/failure auditing for logons to leave a forensic trail of access attempts. `auditpol/secpol: Advanced Audit Policy 'Logon' = Success+Failure` _(low; Win10+11; reboot: no)_
- **Auto-lock on inactivity**: Force a screen-saver lock after idle to protect an unattended, signed-in session. `reg (all REG_SZ under HKCU\Control Panel\Desktop): ScreenSaveActive = "1", ScreenSaverIsSecure = "1", ScreenSaveTimeOut = "600" (seconds). String values, not DWORD.` _(low; Win10+11; reboot: no)_
  - Note: HARDENING (physical security). Mild interruption if the idle timeout is short; no benefit against remote/malware threats. Round-2 value fix: these three are REG_SZ strings under HKCU\Control Panel\Desktop (not DWORD, not HKLM); written as DWORDs the idle lock never engages.
- **Enable Virtualization-Based Security (VBS)**: Turn on VBS to isolate security assets in a hypervisor-protected enclave. `reg: ...\DeviceGuard\EnableVirtualizationBasedSecurity = 1, RequirePlatformSecurityFeatures = 1` _(medium; Win10+11 (default-on on many eligible Win11 devices); reboot: yes)_
  - Note: HARDENING. Costs a few percent CPU/GPU (measurable in games) and needs CPU virtualization; it underpins HVCI and Credential Guard.
- **Enable Memory Integrity (HVCI)**: Enable hypervisor-protected code integrity so only signed, verified code runs in the kernel. `reg: ...\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity\Enabled = 1` _(medium; Win10+11; reboot: yes)_
  - Note: HARDENING. Blocks old/unsigned kernel drivers (some peripherals, older anti-cheat) and adds a small performance cost; verify driver compatibility first.
- **Enable Credential Guard**: Enable VBS-based Credential Guard to isolate domain/NTLM secrets from LSASS theft. `reg: ...\DeviceGuard\LsaCfgFlags = 1 (with UEFI lock)` _(medium; Win11 (default-on for eligible Win11 22H2+ Enterprise); reboot: yes)_
  - Note: HARDENING. Blocks unconstrained delegation, NTLMv1, Kerberos DES, and some VPN/credential plugins; mainly relevant to domain-joined machines. Requires VBS.
- **Store hardware clock as UTC**: Set RealTimeIsUniversal so the RTC is UTC for clean dual-boot and consistent timestamps. `reg: HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation\RealTimeIsUniversal = 1` _(low; Win10+11; reboot: yes)_
  - Note: Primarily a dual-boot (Linux) convenience with only marginal security relevance; can confuse the clock if you also boot older OSes that expect a local-time RTC. Verifier note: this is a dual-boot/timekeeping convenience, NOT security hardening (zero security benefit); it fits a Time/Date category. Mechanism is correct.

## Excluded: myths and cargo-cult tweaks

Every item below was evaluated and rejected. Kept here so the reasoning is not lost.

### Privacy & Telemetry

- **AllowTelemetry = 0 fully disables telemetry on any edition**: Home and Pro floor a 0 to 1 (Required/Basic); only Enterprise, Education, and IoT/LTSC honor Security(0), so on consumer SKUs it never reaches 'off'.
- **Block Microsoft telemetry via the HOSTS file or firewall rules**: Unreliable and breaks the Store, activation, and updates; Windows uses hardcoded IPs and bypasses the HOSTS file for some system endpoints, so it is not a dependable mechanism.
- **sc delete DiagTrack (delete the telemetry service)**: Harmful and hard to revert: breaks Feedback Hub and servicing signals and can be recreated by updates; the supported, reversible approach is setting the service start type to Disabled.
- **Delete or rename CompatTelRunner.exe / take ownership of telemetry binaries**: Breaks Windows servicing and cumulative updates, and the files are restored on the next update; disabling the scheduled task achieves the same with no breakage.
- **Disabling telemetry noticeably speeds up the PC**: It is a privacy change, not a performance one; the resource impact of telemetry on a modern install is negligible, so any 'speed boost' is placebo.
- **dmwappushservice is core spyware that must be killed**: It is the WAP Push routing service for device management (MDM), not the primary telemetry channel; disabling it yields negligible privacy benefit and can break MDM enrollment.
- **Flip hundreds of 'anti-spy' registry keys for maximum privacy**: Many such keys are redundant, no-ops, or revert after feature updates; over-tweaking breaks Settings pages and the Store without adding real privacy over the documented toggles.
- **Disabling DiagTrack breaks Windows Update**: False: Windows Update continues to function with DiagTrack disabled; only Delivery Optimization and Feedback telemetry degrade, which are not required for patching.
- **Disable Wi-Fi Sense auto-connect**: Wi-Fi Sense was removed from Windows in version 1803 (2018); the AutoConnectAllowedOEM key persists for compatibility but governs a feature that is gone on Win10 22H2 and all Win11, so the tweak is a near-no-op on the target OSes.

### Debloat, AI & Consumer

- **Legacy TurnOffWindowsCopilot removes Copilot on 24H2**: On 24H2 Copilot is a standalone app and Microsoft marked this policy legacy/deprecated; it hides the old integrated panel but does not remove the pinned app. Real removal is uninstalling the Microsoft.Copilot appx or the RemoveMicrosoftCopilotApp policy.
- **Uninstall Microsoft Edge to speed up Windows**: Edge/WebView2 is a servicing-managed system component that backs Widgets, Start/Search surfaces, Copilot and many apps; forced removal breaks those, is unsupported, and reinstalls on the next update. Disable its nags (first-run, startup boost, sidebar) instead.
- **Remove Microsoft Store / App Installer as debloat**: Microsoft.WindowsStore and Microsoft.DesktopAppInstaller are must-keep: removing them breaks app installs/updates and winget. Not bloat.
- **Disable SysMain (Superfetch) for speed**: Placebo-to-harmful on modern systems: it increases cold-start latency for apps and yields no FPS or responsiveness gain; Windows self-tunes it. A cargo-cult 'gaming' tweak.
- **Disable Prefetch as debloat**: Negligible or negative effect; the memory/standby manager handles it and there is no measurable win from turning it off on SSD-era hardware.
- **Delete or disable the pagefile to free RAM**: Harmful: causes commit-limit crashes and app failures under load, breaks crash dumps, and gives no speed gain. A classic debloat-script mistake.
- **Disabling DiagTrack/telemetry massively boosts FPS**: Turning off telemetry is a legitimate privacy choice (covered by the consumer-features tweaks) but it is not a performance tweak; there is no measurable frame-rate improvement from stopping DiagTrack.
- **Removing OneDrive makes the PC dramatically faster**: It is a clutter/privacy decision; idle-time performance impact is negligible. The real reasons to remove it are workflow and File Explorer clutter, not speed.
- **Disabling Xbox Game Bar dramatically increases FPS**: Mostly myth on modern hardware; any gain comes only from stopping background Game DVR recording (already covered), not from the Game Bar overlay being present.
- **Remove Microsoft.ZuneMusic to debloat 'Groove'**: On current Windows that package IS the modern Windows Media Player, not legacy Groove; removing it deletes your default media player. A must-keep for most users.
- **Recall can be removed on any Windows 11 PC**: Recall and Click to Do ship only on Copilot+ (NPU) PCs; on standard x64 hardware there is nothing installed to disable or uninstall.
- **Disable Windows Defender / SmartScreen as part of debloat**: Security-critical, not bloat: disabling Defender real-time protection, SmartScreen or tamper protection leaves the machine exposed with no genuine performance payoff. Belongs to a security-hardening decision, never a debloat step.

### Performance & Gaming

- **enable_large_system_cache (LargeSystemCache = 1)**: Server file-cache setting; on a workstation it lets the file cache starve application/GPU working set and can hurt performance or destabilize drivers. No gaming benefit.
- **disable_paging_executive (DisablePagingExecutive = 1)**: Only pins ntoskrnl/drivers in RAM (a kernel-debugging aid); it does not disable the pagefile and yields no measurable speedup on modern RAM.
- **clear_page_file_shutdown (ClearPageFileAtShutdown = 1)**: Merely zeroes the pagefile at shutdown, lengthening shutdown and adding needless SSD writes, for zero runtime performance and negligible security value on modern/encrypted systems.
- **io_page_lock_limit (IoPageLockLimit)**: Has had no effect since Windows 2000; registry monitoring shows the key is never read. Pure placebo.
- **icon_cache_size (Max Cached Icons)**: Only changes icon-cache capacity (fewer icon-cache rebuilds). No effect on FPS, latency, or system speed.
- **font_cache_size**: There is no legitimate 'font cache size' performance value; adjusting or deleting the font cache gives no lasting speed change. Cargo-cult.
- **timer_resolution registry / bcdedit useplatformtick**: Blanket timer hacks (GlobalTimerResolutionRequests, useplatformtick/useplatformclock) are unreliable and on 24H2 can ADD input lag or stutter; games already request ~1 ms via the OS. Not a global FPS lever.
- **reduce_menu_show_delay (MenuShowDelay = 0)**: Only speeds the menu open/fade animation (UI feel). Zero effect on game FPS, frame time, or throughput. Cosmetic, not performance.
- **ntfs_optimizations bundle (as an FPS booster)**: Bundled 'NTFS optimizations' marketed as FPS boosts have no measurable gaming effect; at most they trim metadata I/O. (The one honest member, NtfsDisableLastAccessUpdate, is kept as a low-risk I/O tweak, not sold as FPS.)
- **Win32PrioritySeparation = 26 / 38**: A misread scheduler bit-field, not a performance dial; Windows already boosts the foreground and this cannot raise clocks or add CPU capacity. Most guides even use the wrong decimal/hex value.
- **QoS 20% reservation / Limit Reservable Bandwidth (NonBestEffortLimit = 0)**: Windows never permanently reserves 20% of the link; unused reservation is already fully available to apps. Setting it to 0 frees nothing in normal use. Classic myth.
- **MMCSS 'Games' task priority (SystemProfile\Tasks\Games GPU Priority / Scheduling Category)**: Raising the MMCSS 'Games' task priority does nothing measurable: no observed game threads actually register under the MMCSS 'Games' task.
- **disable_prefetcher on SSD (EnablePrefetcher = 0)**: Provides no speed benefit on SSD and can slow some app launches; Windows already tunes prefetch/ReadyBoot behavior for solid-state drives.
- **manual core-parking disable (registry ValueMax/ValueMin = 0)**: Redundant with a high-performance/Ultimate power plan and, forced blindly, raises power and heat with negligible FPS on modern schedulers.
- **disabledynamictick / force HPET (bcdedit useplatformclock)**: Increases power draw and interrupt overhead and can worsen stutter; forcing HPET is a known stutter source on modern Windows, not a latency win.
- **force GPU MSI-mode / interrupt affinity as a blanket FPS tweak**: Device-specific and risky (can break drivers or prevent boot) and not a guaranteed FPS boost; at best an advanced per-device fix, never a general performance toggle.
- **standby-list / RAM 'cleaner' auto-flushing**: Periodically emptying the standby list throws away useful cached file data, forcing re-reads from disk and causing MORE stutter, not less. Free RAM is not wasted RAM.
- **disable / shrink pagefile to 'save RAM and boost FPS'**: Disabling or hard-capping the pagefile causes out-of-memory app crashes and commit-limit failures under load; Windows manages virtual memory well and there is no FPS upside.

### Services & Scheduled Tasks

- **Disable HomeGroup Provider / Listener services (HomeGroupProvider, HomeGroupListener)**: HomeGroup was removed in Windows 10 1803; the services no longer exist on Win10 22H2 or Windows 11, so disabling them is a no-op (this is the reference corpus item disable_homegroup_provider).
- **Disable Background Intelligent Transfer Service (BITS)**: BITS is demand/trigger-started so it consumes nothing while idle; disabling gives zero performance gain and breaks Windows Update, Microsoft Store, and background app downloads.
- **Disable Windows Search (WSearch) for speed**: The indexer is idle-priority and I/O-throttled, so the 'faster PC' claim is placebo on modern hardware while Start-menu and File Explorer search break.
- **Disable Windows Update service (wuauserv)**: Stops all security patches; UsoSvc/WaaSMedicSvc re-trigger it anyway. Pure risk with no performance benefit.
- **Disable Update Medic Service (WaaSMedicSvc)**: Protected service that self-heals and restarts; it cannot be cleanly disabled and doing so only sabotages Windows Update repair.
- **Disable Delivery Optimization service (DoSvc)**: Disabling the service breaks Microsoft Store and Windows Update downloads (and tools like WUMT); to stop P2P, set the DODownloadMode policy to 0 or 99 instead of killing the service.
- **Disable Diagnostic Policy Service (DPS)**: Breaks Windows troubleshooters, network diagnostics, and some feature detection for near-zero resource savings.
- **Disable Windows Defender / Security Center via services (WinDefend, wscsvc, SecurityHealthService)**: Tamper Protection blocks the change, and forcing it off removes real-time malware protection with no legitimate performance upside (security-critical, out of scope).
- **Disable RPC / DCOM (RpcSs, DcomLaunch)**: Core OS dependency for nearly every service; disabling prevents Windows from booting to a usable desktop.
- **Disable Windows Time (W32Time) for performance**: No measurable performance effect; breaks clock sync, which cascades into Kerberos auth, HTTPS certificate validation, and time-based 2FA codes.
- **Disable Windows Push Notifications (WpnService / WpnUserService)**: Not a performance tweak: it silently breaks all toast notifications, some sign-in and app-update flows, with no real resource payoff.
- **Disable Themes service to free RAM**: Negligible memory footprint; disabling only reverts the UI to Classic visual styles with no meaningful gain.

### UI/UX, Explorer & Taskbar

- **MenuShowDelay = 0 as a performance/gaming tweak**: It only removes the menu-open animation delay (a real cosmetic setting); it does not speed up the CPU, GPU, disk or game frame-rate. The 'makes your PC faster' framing sold in tweak packs is placebo.
- **Restore the Windows 10 taskbar via a native registry key on current Win11**: UndockingDisabled-style edits have crashed explorer.exe since 22H2. A genuine Win10-style taskbar now requires a third-party shell (ExplorerPatcher / StartAllBack), not a native registry value.
- **Restore the classic Start menu via a native registry value**: No native key (Start_ShowClassicMode / EnableXamlStartMenu and friends) brings back the old Start menu on modern builds; it requires OpenShell or StartAllBack.
- **Small Windows 11 taskbar via TaskbarSi registry**: TaskbarSi only resizes the Windows 10 taskbar; Windows 11 ignores it and has no native small-taskbar option.
- **Disabling animations/transparency for a big FPS or gaming boost**: On modern GPUs the desktop compositor cost is negligible; these are cosmetic snappiness tweaks (kept as such), not frame-rate boosts.
- **Enlarging Explorer's icon cache (Max Cached Icons) to speed up File Explorer**: Placebo on modern SSD/RAM systems; it does not measurably speed up Explorer and just bloats the icon-cache database.
- **'God Mode' (All Tasks) folder makes Windows faster**: It is only a flat shortcut list of Control Panel applets; it has zero performance or system effect.
- **'Always show icons, never thumbnails' as a speed tweak**: Negligible on SSDs; it only helps marginally with huge media folders on slow HDDs and it strips useful previews, so it is not a general speed win.

### Network, Windows Update & Power

- **QoS '20% reserved bandwidth' (NonBestEffortLimit)**: Windows never sits on 20% idle: the QoS Packet Scheduler only reserves bandwidth when an app actually requests priority and releases it otherwise. Setting NonBestEffortLimit=0 frees nothing and can only slow prioritized traffic. Pure myth.
- **Increase IRPStackSize**: A LAN/SMB server-storage buffer parameter, not an internet-speed knob. Raising it does nothing for throughput on a stock client and can cause driver/network instability; only relevant to fix specific 'not enough server storage is available' errors.
- **Universal Nagle / TcpAckFrequency / TCPNoDelay 'FPS boost'**: Disabling delayed-ACK/Nagle shaves at most a few ms on some TCP-based games and does nothing for UDP titles, ping, packet loss, or Wi-Fi; most games already set TCP_NODELAY. Marketed as a universal latency cure it is not, and blanket per-interface edits reduce small-packet efficiency.
- **Disable TCP receive-window autotuning (autotuninglevel=disabled)**: Actively harmful: caps throughput on high bandwidth-delay links. Autotuning is correct behavior on modern Windows; disabling it is a classic cargo-cult 'optimizer' that lowers real speeds.
- **DNS cache size registry tweak (CacheHashTableBucketSize / MaxCacheEntryTtlLimit)**: Enlarging the resolver cache tables yields no measurable browsing speedup on a normal client; the defaults are already ample. Placebo.
- **TCP Fast Open (netsh int tcp set global fastopen=enabled)**: Windows' TFO is effectively dormant: major browsers removed TFO support and it delivers no measurable real-world speedup while risking breakage through middleboxes/firewalls. Not worth toggling.
- **Timer-resolution registry/boot hacks (GlobalTimerResolutionRequests, bcdedit disabledynamictick / useplatformtick)**: Forcing a global high timer resolution or platform tick gives no reliable FPS/smoothness gain on modern builds (24H2 already grants per-process timer resolution), while raising power draw, DPC latency, and battery drain.
- **'TCP Optimizer' blanket registry packs (TcpWindowSize, Tcp1323Opts, GlobalMaxTcpWindowSize, hardcoded MTU)**: These XP-era parameters are ignored or auto-managed on Windows 10/11; hardcoding them overrides autotuning and can only degrade a stock connection.
- **Fully disable IPv6 (DisabledComponents=0xFF) for 'speed'**: Microsoft explicitly recommends against disabling IPv6 entirely; it speeds nothing up and can break Windows features, sharing, and app connectivity. Disable only the transition tunnels (0x01) if there's a real need.
- **Disable the Windows Update service (wuauserv / UsoSvc) to 'stop updates'**: Breaks security patching, Defender definition delivery, and Store updates, and Windows repairs/re-enables the service anyway. Use deferral, pause, or target-version policies instead of killing the service.
- **Memory 'optimizers': ClearPageFileAtShutdown, DisablePagingExecutive, LargeSystemCache**: None speeds up a modern SSD system. ClearPageFileAtShutdown only lengthens shutdown and adds SSD writes (it is a security, not perf, setting); DisablePagingExecutive doesn't stop paging, it just pins the kernel; LargeSystemCache favors file-cache over apps and can starve them. Listed for completeness (a Memory-domain concern).

### Security Hardening

- **Rename the built-in Administrator account for security**: The RID-500 SID is well-known and remotely discoverable, so renaming provides negligible protection against real attacks.
- **Disable IPv6 entirely (DisabledComponents=0xFF) to reduce attack surface**: Microsoft explicitly advises against it; it breaks Windows components and offers no security gain. Prefer prefix-policy priority (leave IPv6 enabled).
- **Set RestrictAnonymous = 2**: Overly aggressive legacy setting that breaks networking, trusts, and app compatibility; the correct modern control is RestrictAnonymousSAM = 1.
- **Clear the pagefile at shutdown (ClearPageFileAtShutdown=1) for security**: Real but obsolete: BitLocker/device encryption already protects at-rest data, and this adds a large shutdown delay for effectively no gain on modern PCs.
- **Disable SMBv2/SMBv3 'to be safe'**: Harmful and breaks all modern file sharing; only the legacy SMBv1 should ever be removed.
- **Enable FIPS-compliant algorithm policy for stronger crypto**: Microsoft no longer recommends it; it only blocks newer non-FIPS-validated algorithms and breaks apps, without improving real security. Enable only when regulation requires.
- **Install a third-party AV because 'Defender is weak'**: Modern Microsoft Defender tests at parity with leading AVs; swapping is a preference, not a hardening tweak, and often adds its own attack surface.
- **Disable UAC because 'it does nothing'**: UAC is a core privilege boundary; disabling it (EnableLUA=0) is a serious weakening and is included in the weakening tweaks, not hardening.
- **Disable telemetry/DiagTrack as a 'security' measure**: Telemetry is a privacy concern, not a remote attack surface; disabling it does not harden the system (belongs to the privacy domain).
- **Blanket-disable all non-essential services for 'attack-surface reduction'**: Breaks features and rarely closes a real remote vector; only targeted disables (RemoteRegistry, Spooler when unused) meaningfully help.
- **The DisableAntiSpyware registry key turns off Defender**: On consumer Windows, Tamper Protection ignores that key; it only takes effect when a compatible third-party AV registers, so as a standalone 'tweak' it is placebo.
- **Block all outbound NTLM (RestrictSendingNTLMTraffic=2) on a home/standalone PC**: A legitimate control inside a managed AD domain, but on a home PC it breaks IP-based shares and many apps for no benefit; it is a footgun outside a domain.
- **Disable Windows Update to avoid 'bad patches'**: Harmful: unpatched systems are the number-one real-world compromise vector; the security loss dwarfs any stability worry.
- **Disable Secure Boot to apply tweaks / 'it is not needed'**: Harmful: Secure Boot is a boot-integrity control, and disabling it opens the door to bootkits and undermines VBS/HVCI/Credential Guard.
- **Disable the Windows Defender scheduled tasks to speed up and 'stay safe'**: Harmful: it cripples scheduled scans and signature maintenance while providing no security benefit.
- **Disable paging executive / enable large system cache for a 'faster, more secure' PC**: Neither setting has any security relevance; they are performance placebos on modern RAM-rich systems (belongs to the performance domain).

## Counts

- Privacy & Telemetry: 29
- Debloat, AI & Consumer: 31
- Performance & Gaming: 24
- Services & Scheduled Tasks: 29
- UI/UX, Explorer & Taskbar: 34
- Network, Windows Update & Power: 20
- Security Hardening: 40
- **Grand total: 207 curated tweaks** (86 myths excluded)

## Key sources

- AskVG - Restore Windows 10 classic taskbar in Windows 11 (third-party required, UndockingDisabled broken 22H2): https://www.askvg.com/tip-restore-windows-10-classic-taskbar-in-windows-11-along-with-classic-start-menu/
- Chris Titus WinUtil - Verbose logon and NumLock-on-startup tweak docs: https://winutil.christitus.com/dev/tweaks/customize-preferences/
- Elevenforum - Always/Never Combine Taskbar buttons and Hide Labels (TaskbarGlomLevel, 23H2+): https://www.elevenforum.com/t/always-or-never-combine-taskbar-buttons-and-hide-labels-in-windows-11.15135/
- https://4sysops.com/archives/configuring-the-cloud-clipboard-in-windows-1011-with-group-policy-and-powershell/
- https://batcmd.com/windows/11/services/pcasvc/
- https://blogs.windows.com/windows-insider/2024/11/22/previewing-recall-with-click-to-do-on-copilot-pcs-with-windows-insiders-in-the-dev-channel/
- https://computeraid.com.au/disablepagingexecutive-and-largesystemcache-xp-corruption/
- https://forums.blurbusters.com/viewtopic.php?t=13284
- https://geekchamp.com/how-to-completely-disable-or-uninstall-recall-in-windows-11-24h2/
- https://gist.github.com/Aldaviva/0eb62993639da319dc456cc01efa3fe5
- https://github.com/NicholasBly/Windows-11-Latency-Optimization
- https://helpdeskgeek.com/what-is-limit-reservable-bandwidth-in-windows-and-why-you-shouldnt-change-it/
- https://itm4n.github.io/printnightmare-exploitation/
- https://lifetips.alibaba.com/tech-efficiency/windows-search-indexing-on-ssds-truth-vs-myth
- https://manuals.oo-software.com/ooshutup10/docs/faq/
- https://oneuptime.com/blog/post/2026-03-20-disable-ipv6-transition-technologies-windows/view
- https://pbelamri.com/tcpnodelay/
- https://privacy.sexy/
- https://smoothfps.com/guides/windows-11-stuttering
- https://support.microsoft.com/en-us/topic/kb4073119-windows-client-guidance-for-it-pros-to-protect-against-silicon-based-microarchitectural-and-speculative-execution-side-channel-vulnerabilities-35820a8a-ae13-1299-88cc-357f104f5b11
- https://techcommunity.microsoft.com/blog/windows-itpro-blog/dynamically-remove-apps-from-managed-windows-11-devices/4516291
- https://windowsforum.com/threads/should-you-disable-sysmain-in-windows-boost-performance-on-modern-pcs.376200/
- https://www.automox.com/worklets/windows-11-feature-upgrade-registry
- https://www.blumira.com/integration/disable-llmnr-netbios-wpad-lm-hash/
- https://www.dsinternals.com/en/smb-signing-windows-server-2025-client-11-24h2-defaults/

