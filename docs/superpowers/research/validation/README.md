# Tweak corpus validation

Independent re-validation of the tweak corpus in `src-tauri/tweaks/`, run 2026-07-26 and 2026-07-27
against primary sources, in four passes: a first validation round, an adversarial verification round
that re-attacked the entries the first round had called clean, a gap hunt for controls that should
exist but did not, and a rewrite of every tweak's user-facing copy into one scannable template.

The findings in this directory are **applied**. `src-tauri/tweaks/` now matches them.

**Supported platform:** Windows 11 24H2 (build 26100) and newer including 25H2 (26200), primary.
Windows 10 IoT Enterprise LTSC 2021 (build 19044), low priority secondary. Windows 10 22H2 consumer
and Windows 11 21H2 through 23H2 are out of scope.

The prior research pass (`../2026-07-23-windows-tweak-catalog.md`) was deliberately not consulted, so
these findings are independent rather than a re-reading of earlier conclusions.

## What changed

| | Before | After |
|---|---|---|
| Tweaks | 203 | **255** |
| Category files | 7 | **8** (`ai` is new) |
| Deleted | | 10, features Windows removed at or below the support floor |
| Added | | 62, from the gap hunt |
| Corrected | | 166, from a value name to a whole mechanism |

Every one of the 255 carries a rewritten `info` block in one fixed shape: a one-sentence hook, What
it does, Benefits, Drawbacks, Good to know (with Applies to / Takes effect / Reverting), a
Recommendation that takes a position, and an Evidence block naming the risk level, whether the
mechanism is Microsoft-documented or community-corroborated, and two to four linked sources.

## Documents

| Document | Covers |
|---|---|
| [`_SPEC.md`](_SPEC.md) | The contract each category document follows, including the evidence rules |
| [`_INFO_TEMPLATE.md`](_INFO_TEMPLATE.md) | The required shape of every tweak's `info` block |
| [`_inclusion-principle.md`](_inclusion-principle.md) | The single test for what belongs in the corpus |
| [`_cross-category.md`](_cross-category.md) | Conflicts between categories, structural defects, cross-file merges |
| [`_rescope-24h2.md`](_rescope-24h2.md) | What the 24H2 floor makes obsolete |
| [`_policy-hive-audit.md`](_policy-hive-audit.md) | Every `\Policies\` value checked against its ADMX class |
| [`_harmful-revert.md`](_harmful-revert.md) | Reverts that leave the machine worse than stock |
| [`_probe-fail-open.md`](_probe-fail-open.md) | Action probes that report success they never verified |
| [`_verification.md`](_verification.md) | Which findings survived an adversarial refutation pass |
| [`_gaps-privacy-debloat-interface.md`](_gaps-privacy-debloat-interface.md) · [`_gaps-security-performance-network.md`](_gaps-security-performance-network.md) | The gap hunt: 71 candidate controls the corpus lacked |
| [`_verify-gaps-a-high.md`](_verify-gaps-a-high.md) · [`_verify-gaps-a-medlow.md`](_verify-gaps-a-medlow.md) · [`_verify-gaps-b-high.md`](_verify-gaps-b-high.md) · [`_verify-gaps-b-medlow.md`](_verify-gaps-b-medlow.md) | Adversarial verification of all 71 candidates |
| [`ai.md`](ai.md) · [`debloat.md`](debloat.md) · [`interface.md`](interface.md) · [`network.md`](network.md) · [`performance.md`](performance.md) · [`privacy.md`](privacy.md) · [`security.md`](security.md) · [`services.md`](services.md) | Per-tweak research, corrections, ready-to-paste copy, and sources |

## Result

| Verdict | Count |
|---|---|
| VERIFIED | 88 |
| VERIFIED-WITH-CORRECTION | 151 |
| INCORRECT | 15 |
| DISPUTED | 1 |
| UNVERIFIED | 0 |

Per category, live entries after the rewrite:

| Category | Live | Verdicts |
|---|---|---|
| `security` | 62 | 23 V, 37 VWC, 2 INCORRECT |
| `interface` | 42 | 22 V, 18 VWC, 1 INCORRECT, 1 DISPUTED |
| `privacy` | 34 | 9 V, 25 VWC |
| `services` | 29 | 10 V, 14 VWC, 5 INCORRECT |
| `network` | 27 | 8 V, 19 VWC |
| `debloat` | 27 | 9 V, 16 VWC, 2 INCORRECT |
| `performance` | 24 | 5 V, 15 VWC, 4 INCORRECT |
| `ai` | 10 | 2 V, 7 VWC, 1 INCORRECT |

The second round earned its cost. It attacked the 87 entries the first round had called clean and
flipped 27 of them, a 31 percent miss rate on work that had already been reviewed once.

### A note on how "verified" was decided

The first pass used a Microsoft-only standard and produced 14 UNVERIFIED verdicts. That standard was
wrong for this domain. Microsoft documents what it wants administrators to configure, not everything
the OS reads, so treating "no Microsoft page" as "doubtful" systematically penalises working tweaks.

The revised rule in `_SPEC.md` accepts **three or more genuinely independent** community sources
agreeing on exact key, value name, type, and semantics, labelled as community-corroborated so the
reader knows which kind of confidence it is. Independence is enforced: three sites that copied one
blog count as one source, and one re-assessment explicitly counted the same author's tenforums and
elevenforum tutorials as a single origin.

Under the revised rule all 14 UNVERIFIED verdicts resolved, and 3 of the 4 DISPUTED did too. Several
turned out to be documented after all, just not where the first pass looked: `DisableSearchBoxSuggestions`
is defined in the shipped `WindowsExplorer.admx`, `PnPCapabilities = 24` is given verbatim in Microsoft
KB 2740020, and the MMCSS throttling cap is documented in KB 948066.

## Tweaks that did not work as authored

All fixed in the applied corpus.

| Tweak | Problem |
|---|---|
| `performance:ssd_optimize_trim` | Value name was `NtfsDisableDeleteNotify`; the real value is `DisableDeleteNotification`. No-op. |
| `performance:ultimate_performance_power_plan` | `/duplicatescheme` mints a new GUID, so `/setactive` on the template GUID failed and the probe could never succeed. |
| `performance:optimize_visual_effects` | `VisualFXSetting` alone changes nothing; the values the shell actually reads were never written. |
| `performance:disable_fullscreen_optimizations` | Both options wrote `GameDVR_FSEBehaviorMode: 2`, so it could not round-trip. |
| `security:enable_credential_guard` | Wrote `LsaCfgFlags` to `Control\DeviceGuard`, the OS-maintained state mirror, not the input under `Control\Lsa`. |
| `security:asr_block_lsass_theft` | GUID ended `e4b0`; Microsoft's ends `e4b2`. Defender silently ignores unrecognised GUIDs, so the rule was inert. |
| `services:task_device_census` | Task path `\Device Information\Devicecensus` does not exist; the tasks are `Device` and `Device User`. |
| `services:disable_fax`, `disable_touch_keyboard`, `disable_alljoyn_router` | Presented as universal, but the services are absent across the entire primary platform. Re-scoped to the Windows 10 secondary tier. |
| `interface:disable_recent_files` | Both effects targeted `Explorer\Advanced`; the values live under `Explorer`. |
| `debloat:remove_teams_consumer_app` | Store ID `9NZTWSQNTK1S` returns HTTP 404, and `MicrosoftTeams` does not ship on 24H2 (the unified app is `MSTeams`). |
| `debloat:remove_bing_news_weather` | An array passed to `Get-AppxPackage -Name`, a `System.String` parameter, is rejected at the binder. |
| `ai:remove_recall_feature` | Probe rejected `DisabledWithPayloadRemoved`; DISM's 3010 success-pending-reboot was treated as failure. |

## The revert-correctness class

A wrong mechanism makes a tweak inert, and the user investigates. A wrong **revert** does damage at
the exact moment the user is trying to be careful, and nothing in the interface tells them. That is
why this class outranks the others, and why community agreement is **not** sufficient evidence for a
revert value even under the corroboration rule that governs mechanism verification.

Three were confirmed against documentation and fixed:

- `privacy:disable_online_speech_recognition` reverted by writing `HasAccepted = 1`, **fabricating a
  cloud-speech consent the user never gave.**
- `security:require_smb_signing` reverted by writing `RequireSecuritySignature = 0`, **explicitly
  disabling a protection Windows 11 24H2 enables by default**, leaving the machine worse than if the
  tweak had never been applied.
- `privacy:remove_recall_component` made the mirror error: its "Available" option wrote `absent`, but
  policy semantics leave Recall **disabled** when not configured, so it never did what it said.

The inverse error is equally real and was guarded against: some values genuinely ship seeded, and
sweeping those to `absent` deletes a shipped value. `NetworkThrottlingIndex` (default 10, KB 948066)
and `SystemResponsiveness` (default 20) are documentation-confirmed and keep their literals.

## Gap hunt

71 candidate controls the corpus lacked were proposed, then verified adversarially: **41 confirmed,
22 confirmed with corrections, 8 rejected.** 63 survived and 62 shipped.

The verification pass paid for itself twice over: it caught 7 false source attributions in the
original proposals, and 2 genuine upstream bugs in privacy.sexy. Its sharpest catch was
`alt_tab_hide_browser_tabs`, where the same value name exists at two keys with **different enum
bases**: the policy key needs `4` where `Explorer\Advanced` needs `3`, so writing `3` at the policy
key produces the opposite of the intended behaviour.

## Merge candidates

Conservative same-subsystem merges only. Two candidates were **dissolved by evidence**, which is
worth noting before merging anything on name resemblance:

- **The three Start-menu suggestion tweaks are not duplicates.** `SubscribedContent-338388Enabled` is
  the Windows 10 surface; Windows 11's lookalike is a different mechanism, `Start_IrisRecommendations`
  with tier A `HideRecommendedSection` behind it. Merging would have fused two operating systems'
  controls into one tweak.
- **The two Chat-button tweaks both die instead**, since the button was removed in 23H2, below the
  support floor.

Surviving candidates are recorded per category and were **not** applied; they are proposals, not
corrections. The strongest is the Windows Update behaviour group in `network`. One is mandatory
rather than optional: `target_release_version` and `defer_feature_updates` must merge, because
deferrals are inert under a version pin.

## Address-ownership decisions made at integration

The engine enforces one address, one owner, corpus-wide. Four overlaps only became visible when all
eight files were compiled together, because each category was validated independently and neither
side's document knew about the other. All four were resolved the same way: **the dedicated
single-purpose tweak keeps the address, the bundle drops it and stops claiming it in its copy.**

| Address | Bundle that gave it up | Owner |
|---|---|---|
| `MinAnimate`, `TaskbarAnimations` | `performance:optimize_visual_effects` | `interface:disable_ui_animations` |
| service `WMPNetworkSvc` | `services:disable_ssdp_upnp` | `services:disable_wmp_network_sharing` |
| task `\Microsoft\Windows\Autochk\Proxy` | `privacy:disable_ceip_tasks` | `services:task_autochk_proxy` |

A fifth overlap was left deliberately unresolved: `EnableVirtualizationBasedSecurity` is a Credential
Guard prerequisite, but `performance:disable_vbs_hvci` already owns it and wants the opposite value,
so no shared entry can express it. `security:enable_credential_guard` cross-warns instead.

## Confidence

Findings carry one of two levels. Items in [`_verification.md`](_verification.md), the
`_policy-hive-audit.md` summary, and all 71 gap candidates were **independently attacked** by a second
agent prompted to refute rather than confirm. Everything else is a **single-pass researcher claim**:
sourced, but not adversarially checked.

The refutation pass earned its cost by catching an overstatement: the Advertising ID finding was
originally reported as "the HKCU write is wrong", when in fact HKLM is the documented control and
nothing shows the HKCU write is inert. Acting on the original claim would have removed a working
write. The correct fix is to add, not replace.

## Open questions

1. **A clean 24H2 image baseline is the single highest-value missing input.** It would settle, in one
   pass: the stock defaults behind roughly a third of the `interface` category, the
   ContentDeliveryManager absent-versus-literal question affecting six privacy and debloat tweaks, the
   SysMain and Delivery Optimization defaults, and the withdrawn task-state claims for
   `task_disk_diagnostic_datacollector` and `task_maps_update`. Mounting a clean install ISO's
   `install.wim` and reading the offline hives is cheaper than a VM and gives the true shipped state.
2. **`disable_people_bar` and `disable_news_interests` were deleted on a conditional.** Both are
   marked "delete after re-check" against a real LTSC 2021 image, and that re-check has not happened.
   The supporting reasoning is independent of the re-check (People and News are both on the
   LTSC-excluded app list, and the User Choice Protection Driver has reverted writes to the `Feeds`
   key from non-allowlisted processes since March 2024), so they were dropped. Reversible if the image
   check contradicts it.
3. **`Microsoft.GetHelp`, `Microsoft.YourPhone`, `Microsoft.Getstarted` presence on a stock 24H2
   image.** Absence from Microsoft's removable-app list may mean "not preinstalled" or "system
   component", and those imply different handling.
4. **Verdict criteria diverged between category documents.** The `network` researcher reserved
   VERIFIED-WITH-CORRECTION for mechanism defects and left copy fixes at VERIFIED; the other seven
   counted copy fixes as corrections. No correction was lost either way, but the per-category tallies
   above are not strictly comparable.

**Source-availability warning.** `admx.help` returned HTTP 522 throughout and `getadmx.com` is now a
squatted gambling domain. Both were relied on by the earlier catalog. `elevenforum.com` and
`tenforums.com` return HTTP 403 to automated fetchers and were reached through the Wayback Machine.
The most reliable ADMX source proved to be the shipped `C:\Windows\PolicyDefinitions` set, which is
Microsoft's own artifact and identical on any 26100 install.
