# Corpus re-scope: Windows 11 24H2/25H2 primary, Windows 10 IoT Enterprise LTSC 2021 secondary

Research date: 2026-07-26. Scope decision under review:

- PRIMARY: Windows 11 24H2 (26100) and newer, including 25H2 (26200).
- SECONDARY (low priority): Windows 10 IoT Enterprise LTSC 2021 (19044).
- OUT OF SCOPE: Windows 10 22H2 consumer, Windows 11 21H2 / 22H2 / 23H2.

Files reviewed: `src-tauri/tweaks/interface.yaml`, `src-tauri/tweaks/debloat.yaml`.

Status vocabulary: STILL NEEDED (keep as is), NEEDS GATE CHANGE (keep, but the gate, the
target, or the copy is now wrong), OBSOLETE (the mechanism no longer does what the tweak
claims), DELETE (no supported platform left, remove from the corpus).

## Summary table

| Tweak id | File | Status | One-line reason |
| --- | --- | --- | --- |
| `disable_cortana_button` | interface | DELETE | Gated `products: [10]`; Cortana as a standalone app was retired in 2023 and LTSC editions ship no Cortana, so no supported platform remains. |
| `disable_meet_now` | interface | DELETE | Meet Now is a Skype entry point; Skype was retired on 2025-05-05, and LTSC excludes the consumer app set. |
| `disable_people_bar` | interface | DELETE | My People is deprecated ("no longer being developed"), absent on Windows 11, and People is on Microsoft's LTSC-excluded app list. |
| `disable_news_interests` | interface | DELETE | News and Interests was a GA-channel Windows 10 rollout that LTSC does not receive, and the News app is explicitly excluded from LTSC editions. Verify on a real LTSC 2021 image before final removal. |
| `classic_context_menu_win11` | interface | STILL NEEDED | Microsoft has not blocked or removed the CLSID workaround on 24H2 or 25H2; it is absent from both the deprecated-features and removed-features lists. |
| `taskbar_hover_time` | interface | OBSOLETE | `ExtendedUIHoverTime` is widely reported as no longer honoured since 24H2 and has never been documented by Microsoft. |
| `disable_copilot_taskbar` | interface | NEEDS GATE CHANGE | `TurnOffWindowsCopilot` is documented as deprecated; Microsoft directs admins to AppLocker or app removal instead. |
| `disable_web_search_start` | debloat | NEEDS GATE CHANGE | `DisableSearchBoxSuggestions` still works on 26100+; the paired `BingSearchEnabled` value is undocumented and unreliable on current builds. |
| `disable_chat_taskbar` | interface | DELETE | The Chat button and its `TaskbarMn` value were removed in Windows 11 23H2, before the primary support floor. |
| `remove_teams_chat_taskbar` | debloat | DELETE | Same feature; the backing `ConfigureChatIcon` policy is marked deprecated by Microsoft and the button no longer exists on 24H2/25H2. |
| `remove_cortana` | debloat | DELETE | `Microsoft.549981C3F5F10` is not part of the 24H2/25H2 preinstalled Store app set, and LTSC has no Store apps at all. |
| `remove_dev_home` | debloat | DELETE | "Starting May 2025, Dev Home will no longer be supported as a feature in Windows 11"; not in the 24H2/25H2 preinstalled set. |
| `remove_maps` | debloat | DELETE | Microsoft: "Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release", and the app was made nonfunctional in July 2025. |
| `remove_people` | debloat | DELETE | `Microsoft.People` is not in the 24H2/25H2 preinstalled set and is excluded from LTSC editions. |
| `remove_teams_consumer_app` | debloat | NEEDS GATE CHANGE | Targets the old `MicrosoftTeams` package; the consumer Teams app preinstalled on 24H2/25H2 is `MSTeams`, so the tweak is currently inert. |
| `remove_getstarted_tips` | debloat | OBSOLETE | The Tips app is on Microsoft's deprecated list and is absent from the 24H2/25H2 policy-removable inbox app set; presence on a stock image is unconfirmed. |
| `remove_get_help` | debloat | NEEDS GATE CHANGE | Absent from Microsoft's 24H2/25H2 policy-removable inbox app list; presence unconfirmed, and inert on LTSC. |
| `remove_phone_link` | debloat | NEEDS GATE CHANGE | Absent from Microsoft's 24H2/25H2 policy-removable inbox app list; inert on LTSC. |
| `remove_bing_news_weather` | debloat | STILL NEEDED | `BingNews` and `BingWeather` are both in Microsoft's supported inbox app removal list for 24H2/25H2. |
| `remove_clipchamp` | debloat | STILL NEEDED | `Clipchamp` is in Microsoft's 24H2/25H2 inbox app removal list. |
| `remove_solitaire` | debloat | STILL NEEDED | `MicrosoftSolitaireCollection` is in Microsoft's 24H2/25H2 inbox app removal list. |
| `remove_feedback_hub` | debloat | STILL NEEDED | `WindowsFeedbackHub` is in Microsoft's 24H2/25H2 inbox app removal list. |
| `remove_quick_assist` | debloat | STILL NEEDED | `QuickAssist` is in Microsoft's 24H2/25H2 inbox app removal list (inert on LTSC 2021, which ships the legacy Win32 build). |
| `remove_outlook_new` | debloat | STILL NEEDED | `OutlookForWindows` is in Microsoft's 24H2/25H2 inbox app removal list. |
| `remove_xbox_game_bar` | debloat | STILL NEEDED | `XboxGamingOverlay` is in Microsoft's 24H2/25H2 inbox app removal list. |
| `remove_copilot_app` | debloat | STILL NEEDED | `Copilot` is in Microsoft's 24H2/25H2 inbox app removal list and is the documented replacement path for the deprecated Copilot policy. |
| `disable_widgets` | debloat | STILL NEEDED | `AllowNewsAndInterests` remains the documented machine-wide Widgets control on Windows 11. |
| `remove_recall_feature` | debloat | STILL NEEDED | Already gated `build >= 26100`, which now matches the primary platform exactly. |
| `taskbar_ungroup_labels` | interface | NEEDS GATE CHANGE | Copy still warns about Windows 11 21H2/22H2, which are now out of scope; the caveat is dead text. |
| `enable_end_task_taskbar`, `remove_gallery_nav_pane` | interface | STILL NEEDED | `build >= 22631` gates are now trivially satisfied on the primary platform but still correctly exclude LTSC 2021. |
| All other `products: [11]` interface tweaks | interface | STILL NEEDED | The Windows 11 gate is still meaningful only because LTSC 2021 stays in scope. |

## A. The four `products: [10]` tweaks on Windows 10 IoT Enterprise LTSC 2021 (19044)

The governing fact is what an LTSC image ships. Microsoft states it directly:

> The Long-term Servicing Channel is available only in the Windows Enterprise LTSC editions.
> This edition of Windows doesn't include some applications, such as Microsoft Edge, Microsoft
> Store, Microsoft Mail, Calendar, OneNote, Weather, News, Sports, Money, Photos, Camera, Music,
> and Clock. **These apps aren't supported in the Enterprise LTSC editions, even if you install
> by using sideloading.**
>
> Source: <https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview>

And on the servicing model:

> Features from Windows 10 and 11 that could be updated with new functionality, including
> Microsoft Edge and in-box Windows apps, are also not included.
>
> Source: <https://learn.microsoft.com/en-us/windows/whats-new/ltsc/overview>

That second point matters because News and Interests, Meet Now, and the Chat button were all
delivered to the general availability channel through cumulative or feature updates, which LTSC
does not receive as feature payloads.

### `disable_cortana_button` (`ShowCortanaButton`)

- Cortana as a standalone Windows app is deprecated: "Cortana in Windows as a standalone app is
  deprecated" (<https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features>).
- "Cortana voice assistance in Windows as a standalone app was retired in the spring of 2023"
  (<https://support.microsoft.com/en-us/topic/end-of-support-for-cortana-d025b39f-ee5b-4836-a954-0ab646ee1efa>).
- On Windows 10 21H2 (the LTSC 2021 feature baseline) Cortana is a separate Store app
  (`Microsoft.549981C3F5F10`). LTSC ships no Store and no bundled Store apps, so the app and its
  taskbar button are not present on a stock LTSC 2021 image. Microsoft community and support
  answers describe the same behavior for LTSC images (no Store, no Cortana).

**Verdict: DELETE.** Not applicable on Windows 11 (already gated out), retired on Windows 10, and
absent from LTSC 2021. No supported platform.

### `disable_meet_now` (`HideSCAMeetNow`)

- Meet Now is a Skype shell entry point rolled out to Windows 10 starting with the October 2020
  Update. It exists to launch a Skype meeting.
- Skype was retired on 2025-05-05 for free and paid consumer users
  (<https://support.microsoft.com/en-us/skype/how-do-i-use-skype-s-meet-now-from-my-windows-10-taskbar-or-outlook-com-3bdebac0-3008-4c28-bdc8-6253d8680f40>).
  The feature the icon points at no longer exists.
- LTSC 2021 does not receive GA-channel shell feature rollouts and ships no consumer app set.

**Verdict: DELETE.** Even in the case where the tray control still renders on some LTSC image, the
underlying service is retired, so hiding it is a no-op with no user value.

### `disable_people_bar` (`PeopleBand`)

- Microsoft lists "My People / People in the Shell" as deprecated: "My People is no longer being
  developed. It may be removed in a future update."
  (<https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features>).
- The People bar does not exist on Windows 11 at all.
- People is part of the Mail/Calendar/People app family that Microsoft names as excluded from LTSC
  editions (see the waas-overview quote above, which lists Mail and Calendar; People shares the
  same OneSync-backed app family, itself deprecated on the same page).

**Verdict: DELETE.** Deprecated on the only OS family that ever had it, and the app family is
excluded from LTSC.

### `disable_news_interests` (`ShellFeedsTaskbarViewMode`)

- News and Interests was delivered to general availability channel Windows 10 (1909 and later)
  through cumulative updates. LTSC releases do not receive in-box app or shell feature payloads
  (<https://learn.microsoft.com/en-us/windows/whats-new/ltsc/overview>).
- The News app itself is explicitly on Microsoft's LTSC-excluded list (waas-overview quote above).
- On Windows 10 22H2 the News and Interests board was superseded by Widgets, and Windows 11 uses
  Widgets exclusively, which the corpus already covers via `disable_widgets`.

**Verdict: DELETE**, with one caveat. Microsoft does not publish a per-edition applicability
statement for News and Interests, and there are third-party forum reports of the feature appearing
on Windows 10 21H2 Enterprise LTSC after cumulative updates. That evidence is weak and
contradicted by the LTSC servicing model, but if the maintainer wants certainty, boot an LTSC 2021
image and check whether `Feeds` renders in the taskbar context menu before deleting.

## B. `classic_context_menu_win11` on 24H2 / 25H2

The tweak writes an empty default value under
`HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32`.

Findings:

- The CLSID workaround does not appear anywhere in Microsoft's deprecated-features or
  removed-features lists
  (<https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features>,
  <https://learn.microsoft.com/en-us/windows/whats-new/removed-features>). There is no published
  block, removal, or end-of-support announcement.
- The Microsoft Q&A thread most often cited as "it stopped working" resolves to a user error, not
  a Microsoft block. The accepted answer from a Microsoft engineer confirms the key still works
  and that the failures come from (a) not restarting Explorer and (b) running `reg add` from an
  elevated shell under a different user, which writes the value into the admin account's HKCU
  rather than the signed-in user's
  (<https://learn.microsoft.com/en-us/answers/questions/4045956/unable-to-change-win11-context-menu-with-registry>).
- Third-party 25H2 guides published in 2025 still document the same registry command as the
  primary method
  (<https://pureinfotech.com/bring-back-classic-context-menu-windows-11/>,
  <https://4sysops.com/archives/restore-classic-context-menu-in-windows-11-explorer-using-group-policy-or-powershell/>).

**Verdict: STILL NEEDED.** Keep the existing "Microsoft has flagged this key as deprecated and may
block it in a future build" caveat, which remains accurate as forward-looking risk. The tweak's
own implementation already writes to the signed-in user's HKCU through the elevation broker, so it
is not exposed to the elevated-shell failure mode described above. This is worth confirming in the
broker's HKCU handling if it has not been checked.

Note the corpus copy is currently misleading in one respect: it says the workaround "sets an empty
`InprocServer32` default under a specific CLSID that suppresses the new compact menu". Accurate,
but the "deprecated" claim should cite that the deprecation is Microsoft's general position on
shell CLSID overriding, not a dated removal notice, because no dated notice exists.

## C. `taskbar_hover_time` (`ExtendedUIHoverTime`) on 24H2 / 25H2

Findings:

- `ExtendedUIHoverTime` has never been documented by Microsoft. It does not appear in any Policy
  CSP, Group Policy reference, or Learn article. There is no authoritative statement either way.
- Multiple independent reports say the value stopped taking effect starting with 24H2, which is
  consistent with the taskbar having been rewritten as an XAML/WinUI surface that no longer reads
  the legacy `Explorer\Advanced` timer:
  - <https://learn.microsoft.com/en-us/answers/questions/2263244/extendeduihovertime-is-not-working-anymore> (24H2; no working fix found in the thread)
  - <https://www.elevenforum.com/t/disabling-taskbar-thumbnails-no-longer-works-on-24h2.33793/>
  - <https://www.elevenforum.com/t/registry-key-extendeduihovertime-to-modify-hover-time-for-showing-taskbar-thumbnail-previews.24693/>
- The commonly suggested substitute on 24H2 is a Windhawk taskbar mod, that is, a third-party code
  injection, which is not something this app should ship.

**Verdict: OBSOLETE on the primary platform.** The tweak has no supported mechanism on 26100+ and
the app would report a state it cannot actually deliver, which conflicts with the "did-it-work"
contract. Two acceptable outcomes:

1. Delete it outright (recommended, since LTSC 2021 is low priority and this is a cosmetic timer).
2. Gate it to `windows: { products: [10] }` if the maintainer wants to keep it for LTSC 2021,
   where the legacy taskbar still honours the value.

Do not leave it ungated on 24H2/25H2.

## D. `disable_copilot_taskbar` (`TurnOffWindowsCopilot`)

Microsoft's Policy CSP page carries an explicit deprecation banner and an applicable-OS list that
stops before 24H2:

> **TurnOffWindowsCopilot** ... Note: This policy is deprecated and may be removed in a future
> release.
>
> Applicable OS: Windows 10 version 21H2 [10.0.19044.3758] and later; Windows 10 version 22H2 with
> KB5032278; Windows 11 version 22H2 with KB5030310; **Windows 11 version 23H2 [10.0.22631] and later**.
>
> Note: The TurnOffWindowsCopilot policy isn't for the new Copilot experience ... that will be
> gradually rolling out to Windows 11 and Windows 10 devices.
>
> Source: <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai>

The supported replacement is named directly:

> **AppLocker policy should be used instead of the Turn Off Windows Copilot legacy policy setting
> and its MDM equivalent, TurnOffWindowsCopilot. The policy is subject to near-term deprecation.**
>
> Source: <https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot>

That same page documents removing the app itself (Settings > Apps > Installed Apps > Uninstall, or
`Microsoft.Copilot` removal by an admin). On 25H2 and 24H2 there is also a first-party policy path:

> IT admins now have an easy way to remove preinstalled Microsoft Store apps (inbox apps) ...
> `RemoveDefaultMicrosoftStorePackages` in the ApplicationManagement Policy CSP.
>
> Source: <https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-25h2>
> and <https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal>

`Copilot` is one of the app ids in that policy's supported list.

**Verdict: NEEDS GATE CHANGE.** The policy still exists and still hides the button plus the Win+C
hotkey on current builds, so the tweak is not inert, but the corpus copy should say plainly that
Microsoft has deprecated the policy and that the durable removal path on 24H2/25H2 is uninstalling
`Microsoft.Copilot` (already covered by `remove_copilot_app`). The existing info text partially
says this; strengthen it to name the deprecation explicitly. The `remove_copilot_app` tweak is now
the primary control and `disable_copilot_taskbar` the cosmetic supplement.

## E. `disable_web_search_start` on 26100+

The tweak sets two values:

1. `HKCU\Software\Policies\Microsoft\Windows\Explorer\DisableSearchBoxSuggestions` = 1
2. `HKCU\Software\Microsoft\Windows\CurrentVersion\Search\BingSearchEnabled` = 0

Findings:

- `DisableSearchBoxSuggestions` is the policy-backed path and continues to work on current builds,
  including 24H2. It is the value every current guide recommends as the update-resistant one
  (<https://pureinfotech.com/disable-search-web-results-windows-11/>).
- `BingSearchEnabled` is not a policy value, is undocumented on Learn, and is widely reported to be
  reset by feature updates and to be unreliable as the sole control on current builds. It does no
  harm, but it should not be presented as the mechanism.
- The documented, supported, machine-scoped alternatives on Learn are
  `Search/DoNotUseWebResults` (Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC; not Pro)
  and `Search/AllowSearchHighlights`
  (<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search>).
  `DoNotUseWebResults` is edition-limited and excludes Pro, which is why the HKCU policy value
  remains the right default for a consumer-facing tool.
- 24H2 also added Settings > Privacy and security > Search permissions with a cloud content
  control, which is the in-box equivalent.

**Verdict: NEEDS GATE CHANGE.** Keep `DisableSearchBoxSuggestions` as the effect that carries the
tweak. Either drop the `bing_enabled` effect or keep it and mark it `skip_validation: true` so a
value the OS may silently reset cannot produce a false Needs Attention state. Update the info copy
so the user is not told that `BingSearchEnabled` is what does the work.

## F. The Chat / Teams taskbar button

Two tweaks target the same vanished feature:

- `disable_chat_taskbar` (interface, `TaskbarMn`)
- `remove_teams_chat_taskbar` (debloat, `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Chat\ChatIcon` = 3)

Findings:

- Microsoft's own 23H2 release notes: "**Microsoft Teams: Chat is being removed from the Microsoft
  Teams in-box app.** Teams will no longer be pinned to the taskbar for enterprise editions of
  Windows 11, version 23H2 or later."
  (<https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-23h2>)
- The backing policy is marked deprecated: "**ConfigureChatIcon** ... Note: This policy is
  deprecated and may be removed in a future release."
  (<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-experience>)
- Coverage of the 23H2 release confirms the Chat button is gone from Taskbar settings and was
  replaced by a normal pinnable Microsoft Teams (free) app
  (<https://www.windowscentral.com/software-apps/windows-11/whats-new-with-taskbar-and-start-menu-on-windows-11-2023-update>,
  <https://www.howtogeek.com/898705/the-chat-taskbar-button-in-windows-11-is-going-away/>).

**Verdict: DELETE both.** The feature was removed one release before the primary support floor.
`TaskbarMn` on 24H2 writes a value nothing reads; the `ChatIcon` policy configures an icon that no
longer exists. Neither has any effect on 24H2/25H2, and neither ever applied to Windows 10.

Related defect surfaced while checking this: `remove_teams_consumer_app` targets the Appx package
`MicrosoftTeams`, which is the old Chat-era consumer build. On 24H2/25H2 the preinstalled consumer
Teams package is `MSTeams`, which is the id Microsoft lists in its policy-based inbox app removal
ADMX sample. As written, `remove_teams_consumer_app` finds nothing on a stock 24H2/25H2 image and
its probe reports "already removed" while the actual Teams app is still installed. This should be
retargeted to `MSTeams` (or to both ids) before the re-scope ships.

## G. Appx packages that no longer ship preinstalled on 24H2 / 25H2

The strongest available first-party signal is the app list in Microsoft's policy-based inbox app
removal ADMX, which enumerates the preinstalled Microsoft Store apps that 24H2 and 25H2 support
removing by policy
(<https://learn.microsoft.com/en-us/windows/configuration/policy-based-inbox-app-removal/policy-based-inbox-app-removal>):

`WindowsFeedbackHub`, `MicrosoftOfficeHub`, `Clipchamp`, `Copilot`, `BingNews`, `Photos`,
`MicrosoftSolitaireCollection`, `MicrosoftStickyNotes`, `MSTeams`, `Todo`, `BingWeather`,
`OutlookForWindows`, `Paint`, `QuickAssist`, `ScreenSketch`, `WindowsCalculator`, `WindowsCamera`,
`MediaPlayer`, `WindowsNotepad`, `WindowsSoundRecorder`, `WindowsTerminal`, `GamingApp`,
`XboxGamingOverlay`, `XboxIdentityProvider`, `XboxSpeechToTextOverlay`, `XboxTCUI`, plus a
`DynamicRemovalList` for arbitrary package family names.

Per-package answers for the ids the task asked about:

| Package | Preinstalled on stock 24H2 / 25H2? | Evidence |
| --- | --- | --- |
| `Microsoft.549981C3F5F10` (Cortana) | No | Cortana standalone app deprecated and retired spring 2023; absent from the 24H2/25H2 inbox app list. |
| `Microsoft.Windows.DevHome` | No | "Starting May 2025, Dev Home will no longer be supported as a feature in Windows 11" (<https://learn.microsoft.com/en-us/windows/dev-home/>); absent from the 24H2/25H2 inbox app list. It was preinstalled on early 24H2 images, so removal may still find it on a machine that shipped before mid-2025. |
| `Microsoft.People` | No | "My People is no longer being developed" (deprecated-features); absent from the 24H2/25H2 inbox app list. |
| `Microsoft.WindowsMaps` | No | "**Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release.**" The app was also made nonfunctional by a final Store update in July 2025 (<https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources>). This is the single most explicit statement in the whole review. |
| `Microsoft.Getstarted` (Tips) | Unconfirmed, probably no | "The Tips app is deprecated and will be removed in a future release of Windows" (deprecated-features); absent from the 24H2/25H2 inbox app list, but Microsoft has not published a "no longer preinstalled" statement the way it did for Maps. |
| `Microsoft.BingNews` | Yes | `BingNews` is in the 24H2/25H2 inbox app removal list. |
| `Microsoft.BingWeather` | Yes | `BingWeather` is in the 24H2/25H2 inbox app removal list. |
| `MicrosoftTeams` | No, superseded | The preinstalled consumer Teams package on 24H2/25H2 is `MSTeams`. `MicrosoftTeams` was the Chat-era package removed in 23H2. |

Caveat on reading absence from that list: it enumerates the apps the removal policy supports, not
every package on the image. Microsoft's own troubleshooting section notes event IDs for package
family names that are "a system component" or "part of an AI component that is not removable", so
some preinstalled packages are deliberately excluded from the list. Absence is therefore strong
evidence but not proof, except for Maps, where Microsoft says outright that it is no longer
preinstalled. Two corpus entries fall into this ambiguous band and should be verified on a real
24H2 image rather than deleted on the strength of the list alone:

- `remove_get_help` (`Microsoft.GetHelp`)
- `remove_phone_link` (`Microsoft.YourPhone`)

Everything else in `debloat.yaml` maps cleanly onto the supported list and stays.

## H. Debloat app-removal tweaks that are meaningless on Windows 10 IoT Enterprise LTSC 2021

The controlling statement, again from Microsoft:

> This edition of Windows doesn't include some applications, such as Microsoft Edge, Microsoft
> Store, Microsoft Mail, Calendar, OneNote, Weather, News, Sports, Money, Photos, Camera, Music,
> and Clock. These apps aren't supported in the Enterprise LTSC editions, even if you install by
> using sideloading.
>
> Source: <https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview>

LTSC images ship without the Microsoft Store client and without the bundled UWP app set. Every
`Get-AppxPackage ... | Remove-AppxPackage` tweak therefore finds nothing on LTSC 2021, and every
`winget install` undo is doubly broken there because the Store is absent.

Inert on Windows 10 IoT Enterprise LTSC 2021:

- `remove_bing_news_weather` (News and Weather are named on the excluded list)
- `remove_solitaire`
- `remove_get_help`
- `remove_getstarted_tips`
- `remove_feedback_hub`
- `remove_maps`
- `remove_people` (People is part of the excluded Mail/Calendar/People family)
- `remove_phone_link`
- `remove_cortana` (app half; the `AllowCortana` policy half is also pointless with no Cortana present)
- `remove_xbox_game_bar`
- `remove_quick_assist` (LTSC 2021 ships the legacy Win32 Quick Assist, not the Appx package)
- `remove_clipchamp`, `remove_dev_home`, `remove_outlook_new`, `remove_teams_consumer_app`,
  `remove_copilot_app` (already Windows 11 or build-gated, so correctly excluded today)

Still meaningful on LTSC 2021:

- `remove_onedrive` (OneDrive ships as a Win32 setup payload, not a Store app)
- The Edge policy tweaks, only if Edge was installed separately, since LTSC does not include it
- `verbose_logon_messages`, `disable_startup_sound`, and the rest of the HKCU/HKLM interface set

**Recommendation:** gate every Appx removal tweak to `windows: { products: [11] }` or
`windows: { build: ">=26100" }`. That turns the whole class into a Windows 11 feature, matches the
primary platform exactly, and stops the app from advertising removals it cannot perform on the
secondary platform. It also removes the need to reason about LTSC per tweak.

## Unknowns and follow-ups

1. `disable_news_interests` on a real LTSC 2021 image. Microsoft publishes no per-edition
   applicability for News and Interests, and third-party forum reports conflict with the LTSC
   servicing model. Verify before final deletion, or delete and accept the small risk.
2. `disable_meet_now` presence on LTSC 2021. Not verifiable from documentation. Moot in practice
   because Skype is retired.
3. `taskbar_hover_time` on 24H2/25H2. No Microsoft documentation exists for `ExtendedUIHoverTime`
   in either direction. The 24H2 breakage is well attested by users but not officially confirmed.
   A single manual check on a 26100 machine would settle it.
4. `Microsoft.GetHelp` and `Microsoft.YourPhone` presence on a stock 24H2 image. Absent from
   Microsoft's policy-removable inbox app list, which may mean not preinstalled or may mean
   classified as a system component. Needs a `Get-AppxPackage` check on a clean image.
5. `Microsoft.Getstarted` presence on a stock 24H2 image. Deprecated by Microsoft, absent from the
   removal list, but no explicit "no longer preinstalled" statement.
6. Whether the elevation broker writes `classic_context_menu_win11` into the signed-in user's HKCU
   rather than the elevated token's. This is the documented cause of the "it stopped working"
   reports, and it is a code question, not a research question.
