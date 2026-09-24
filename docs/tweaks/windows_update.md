# Windows Update tweaks

This category controls how Windows Update behaves on the machine: Delivery Optimization peering, quality and feature update deferral, version pinning, mid-cycle feature control, Insider enrolment, active hours and restarts, driver delivery, Microsoft Store app updates, metered downloads, the automatic-update mode, and a full block of the update pipeline. The primary platform is Windows 11 24H2 (build 26100) and newer; Windows 10 IoT Enterprise LTSC 2021 (build 19044) is a secondary target and is called out per tweak only where it differs. Most tweaks here are Windows Update client policies under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, which Microsoft documents for Pro, Education, Enterprise and IoT Enterprise only: **Windows Home ignores them**. The exceptions are Delivery Optimization, active hours, the Store policy (Home undocumented) and the services-and-tasks half of the pipeline block, which do not depend on edition.

"Switch" means one or two authored options, shown as a segmented switch; "Dropdown" means three or more. The app adds the computed **System Default** state to either while it is the live state (the state when the machine matches no authored option; selecting it restores the snapshot).

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Disable Delivery Optimization P2P](#disable-delivery-optimization-p2p) | `disable_delivery_optimization_p2p` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Defer quality updates](#defer-quality-updates) | `defer_quality_updates` | Dropdown (3 options) | medium | admin | no | VERIFIED |
| [Defer feature updates](#defer-feature-updates) | `defer_feature_updates` | Dropdown (4 options) | low | admin | no | VERIFIED |
| [Pin Windows feature version](#pin-windows-feature-version) | `target_release_version` | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Control mid-cycle features and optional content](#control-mid-cycle-features-and-optional-content) | `update_feature_control` | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block Insider preview builds by policy](#block-insider-preview-builds-by-policy) | `block_insider_builds_policy` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Set active hours](#set-active-hours) | `set_active_hours` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block auto-restart while signed in](#block-auto-restart-while-signed-in) | `disable_auto_restart_logged_on` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Exclude driver updates from Windows Update](#exclude-driver-updates-from-windows-update) | `exclude_wu_driver_updates` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable automatic driver installation](#disable-automatic-driver-installation) | `disable_auto_driver_install` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Microsoft Store auto-updates](#disable-microsoft-store-auto-updates) | `disable_store_auto_updates` | Switch (2 options) | medium | admin | no | VERIFIED |
| [Block auto-download over metered](#block-auto-download-over-metered) | `block_update_over_metered` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Windows Update mode](#windows-update-mode) | `windows_update_mode` | Dropdown (4 options) | high | admin | no | VERIFIED |
| [Block the Windows Update pipeline](#block-the-windows-update-pipeline) | `block_update_pipeline` | Switch | high | admin (ti for 21 of 30 effects) | yes | VERIFIED (empirically tested; two open questions) |

## Tweaks

### Disable Delivery Optimization P2P

`disable_delivery_optimization_p2p` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Pins Windows Update and Store downloads to plain HTTP from Microsoft (or a Microsoft Connected Cache), with no peer-to-peer sharing with other PCs on your network.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `download_mode` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization` → `DODownloadMode` (`REG_DWORD`) |

| Option | `download_mode` |
|---|---|
| HTTP only, no peering | `0` |
| Peering allowed | `absent` |

System Default: the live `DODownloadMode` policy value is something other than 0 or absent (for example 1, 2 or 3 set by another tool or an organization); selecting it restores the snapshot. The stock state is value-absent: the policy key was absent on the 26100.4061 test machine (measured), which matches the "Peering allowed" option.

#### How it works

Delivery Optimization (`DoSvc`) is the download engine Windows Update and the Microsoft Store use. It reads its download mode from the Delivery Optimization policy key. The shipped 26100 `DeliveryOptimization.admx` defines policy `DownloadMode` writing `DODownloadMode` with the enumeration 0, 1, 2, 3, 99 and 100. Mode 0 is "HTTP only, no peering": in Microsoft's words it "disables peer-to-peer caching but still allows Delivery Optimization to download content over HTTP from the download's original source or a Microsoft Connected Cache server." Mode 1 is "HTTP blended with peering behind the same NAT" (LAN peering), mode 2 is group peering, mode 3 adds internet peers, 99 is simple mode and 100 is Bypass, which Microsoft deprecated in Windows 11 with the warning that it "can cause some content to fail to download".

What the unconfigured default is remains unresolved, and both sources are tier A and current: the Delivery Optimization reference says "Default is configured to LAN(1)", while the Policy CSP page for `DODownloadMode` says "Default Value: 0" and "0 (Default) HTTP only, no peering". So on a stock machine the tweak either removes same-subnet LAN peering or pins a mode that was already in effect. Under neither reading was a stock machine seeding to strangers on the internet (that is mode 3). The real value of the tweak is removing LAN peering where it is active, and locking the mode in policy so it cannot be turned on later from Settings or by an image change.

The tweak changes only the mode. It does not disable `DoSvc`, which the Store depends on. The Delivery Optimization client reads this policy on all editions, including Home. `DoSvc` was demand-start with a Group Policy start trigger on the test machine, which is why no reboot is needed.

#### Benefits
- No LAN peering: the PC stops exchanging update chunks with other machines behind the same router.
- The mode is locked by policy, so neither Settings nor a later tool can turn peering back on.
- Predictable traffic from one source, easier to reason about on a capped link.
- The service stays intact, so Store and Windows Update downloads keep working.

#### Drawbacks
- On a network with several Windows PCs, each downloads its own copy, so total WAN traffic rises.
- Smaller change than it sounds: the PC was never uploading to internet peers by default.
- Because Microsoft's two pages disagree on the default, the tweak may change nothing visible on some machines.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1511 and later including LTSC 2021; all editions including Home.
- **Takes effect**: immediately; `DoSvc` picks up the policy without a reboot.
- **Reverting**: "Peering allowed" deletes the value, returning to Windows' unconfigured behaviour; System Default restores whatever the snapshot captured.

#### Interactions
- [Block the Windows Update pipeline](#block-the-windows-update-pipeline) leaves `DoSvc` alone, but its `DoNotConnectToWindowsUpdateInternetLocations` value is documented by Microsoft as possibly causing Delivery Optimization to stop working.
- The July 2026 research proposed merging this with [Block auto-download over metered](#block-auto-download-over-metered) into one "bandwidth" control; they ship as separate tweaks, and they combine freely.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). The mechanism (key, value name, `REG_DWORD`, the meaning of 0, the absent stock state) was confirmed against the shipped 26100 ADMX; the correction was factual framing: internet peering is mode 3 and is not the default, and the unconfigured default is disputed between two Microsoft pages.
- **Confidence**: Microsoft-documented (Delivery Optimization reference, Policy CSP, privacy connection guide, and the shipped 26100 ADMX and ADML).
- **Reasoning**: the value, key and enumeration are Microsoft's own and match the shipped ADMX exactly. The adversarial pass attacked the claimed benefit (stopping internet uploads) and the stated default; both were narrowed, the mechanism survived. No source ties an error code to mode 100. Open question: the true unconfigured default, which does not affect revert because `absent` is correct under either reading.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
A sensible choice for a single-PC household, particularly on a connection with limited upload. Leave peering on if you run several Windows PCs on the same network and want them to share downloads locally.

#### Sources
1. Delivery Optimization reference, Download Mode: the mode enumeration, mode 1 as "peering behind the same NAT", the mode 100 deprecation, and "Default is configured to LAN(1)", https://learn.microsoft.com/en-us/windows/deployment/update/waas-delivery-optimization-reference (tier A)
2. Policy CSP, DeliveryOptimization, `DODownloadMode`: "Default Value: 0" and "0 (Default) HTTP only, no peering", contradicting source 1, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-deliveryoptimization (tier A)
3. Manage connections from Windows to Microsoft services, section 28.3: registry location and `REG_DWORD` type, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
4. Shipped `C:\Windows\PolicyDefinitions\DeliveryOptimization.admx` and `en-US\DeliveryOptimization.adml` on build 26100.4061: policy `DownloadMode`, key, value name and enumeration (tier A, primary)

### Defer quality updates

`defer_quality_updates` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Holds back the monthly security and quality rollup by 7 or 14 days, so a bad patch is usually reported before it reaches you.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `defer_quality` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `DeferQualityUpdates` (`REG_DWORD`) |
| `defer_quality_days` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `DeferQualityUpdatesPeriodInDays` (`REG_DWORD`) |

| Option | `defer_quality` | `defer_quality_days` |
|---|---|---|
| Defer 7 days | `1` | `7` |
| Defer 14 days | `1` | `14` |
| No deferral | `absent` | `absent` |

System Default: any other combination (for example a 3-day or 30-day deferral set elsewhere); selecting it restores the snapshot. The stock state is both values absent, which is the "No deferral" option.

#### How it works

These two values are the registry form of the Group Policy "Select when Quality Updates are received" (Windows Components > Windows Update > Manage updates offered from Windows Update), part of Windows Update client policies (formerly Windows Update for Business). `DeferQualityUpdates = 1` enables the deferral and `DeferQualityUpdatesPeriodInDays` sets it; Policy CSP documents the range as 0 to 30 days with default 0. The Windows Update client re-evaluates policy at its next scan, so offered quality updates simply become visible to the device that many days after release. Nothing is skipped: every update still arrives, only later.

Windows Update client policies are documented for Pro (including Pro for Workstations), Education and Enterprise (including Enterprise LTSC, IoT Enterprise and IoT Enterprise LTSC), Windows 10 1607 and later. Home is not in the list and ignores the values. A quality deferral is independent of a feature-version pin: Microsoft's statement that a target version makes deferrals inert covers feature-update deferrals only.

#### Benefits
- A buffer against bad patches: problems other people hit usually surface before a deferred update reaches you.
- Still fully patched; a deferral is not a block.
- Control over cadence without turning automatic updates off.

#### Drawbacks
- You run without the newest security fixes for the deferral period.
- Microsoft advises "To help ensure that devices stay secure, configure quality update deferral to be less than 3 days"; both offered options exceed that.
- Does nothing on Home.
- Not a pause: you cannot skip a month, only delay it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. Windows Home ignores it.
- **Takes effect**: at the next Windows Update scan; no reboot.
- **Reverting**: "No deferral" deletes both values; System Default restores the snapshot.

#### Interactions
- Works together with [Pin Windows feature version](#pin-windows-feature-version); the research explicitly rejected merging the two because a quality deferral plus a version pin is a supported combination.
- Setting a quality deferral makes the device "update-managed", which is what activates the temporary-feature-control half of [Control mid-cycle features and optional content](#control-mid-cycle-features-and-optional-content).
- Inert while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied (nothing is scanned), but it takes effect again the moment the block is reverted.

#### Validation
- **Verdict**: VERIFIED (July 2026 research). No mechanism correction.
- **Confidence**: Microsoft-documented.
- **Reasoning**: both value names, types, the enable flag and the 0 to 30 range are given verbatim by Microsoft's client-policy page and Policy CSP; 7 and 14 are inside the range. The policy-hive audit confirmed both values are Machine-class and written to HKLM. The adversarial pass checked whether a version pin makes this inert; it does not.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Use it on Pro and above if you want to dodge bad patches, and prefer the 7-day option. Do not bother on Home, where it does nothing, and do not treat it as a substitute for patching.

#### Sources
1. Configure Windows Update client policies, "Configure when devices receive quality updates": both registry values and the under-3-days recommendation, https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb (tier A)
2. Policy CSP, Update, `DeferQualityUpdatesPeriodInDays`: range 0 to 30, default 0, ADMX mapping, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
3. What are Windows Update client policies?: supported editions, Home absent, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Defer feature updates

`defer_feature_updates` · Dropdown (4 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Slides every annual Windows feature update back by 30, 180 or 365 days, without pinning a version, while monthly security updates keep flowing.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `defer_feature` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `DeferFeatureUpdates` (`REG_DWORD`) |
| `defer_feature_days` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `DeferFeatureUpdatesPeriodInDays` (`REG_DWORD`) |

| Option | `defer_feature` | `defer_feature_days` |
|---|---|---|
| Defer 30 days | `1` | `30` |
| Defer 180 days | `1` | `180` |
| Defer 365 days | `1` | `365` |
| No deferral | `absent` | `absent` |

System Default: any other combination; selecting it restores the snapshot. The stock state is both values absent ("No deferral").

The tweak carries a UI warning: feature update deferrals are not in effect while a target release version is pinned, so it must not be combined with [Pin Windows feature version](#pin-windows-feature-version).

#### How it works

The shipped 26100 `WindowsUpdate.admx` defines policy `DeferFeatureUpdates` (supported on Windows 10, non-ARM) writing `DeferFeatureUpdates` (enabled 1, disabled 0) and the decimal element `DeferFeatureUpdatesPeriodInDays` (0 to 365). Policy CSP gives Windows 10 1607 and later, editions Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC, range 0 to 365, default 0. The research looked for a deprecation specifically, because Microsoft has been retiring deferral policies: none exists; the policy is live and undecorated in the 26100 ADMX and Policy CSP carries no deprecation banner.

A deferral slides: each feature update becomes available that many days after its release. That differs from a pin, which holds the device on a named version until the pin is moved or servicing ends. Microsoft states the two are exclusive: "When you specify target version policy, feature update deferrals won't be in effect."

The same ADMX policy also owns `PauseFeatureUpdatesStartTime` (`REG_SZ`), which this tweak does not write and does not assume is absent, because the user may have paused updates from Settings. `BranchReadinessLevel` is not part of this policy on 26100 (it belongs to `ManagePreviewBuilds`), so older guidance that pairs them does not apply.

#### Benefits
- Security patches keep arriving; only feature updates are delayed.
- Up to a full year of deferral, the documented maximum.
- Slides rather than pins, so there is no pinned version silently ageing out of support.

#### Drawbacks
- Fully inert while a target release version is pinned.
- Does nothing on Home.
- New features in the annual update arrive late.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. Windows Home ignores it. (LTSC does not receive annual feature updates through Windows Update, so the deferral has little to act on there.)
- **Takes effect**: at the next Windows Update scan.
- **Reverting**: "No deferral" deletes both values; System Default restores the snapshot. A pause set from Settings is untouched either way.

#### Interactions
- **Mutually exclusive in effect with [Pin Windows feature version](#pin-windows-feature-version)**: with a pin applied, this tweak writes its values but no deferral happens. The research recommended merging the two into one control; they ship separately with a warning on each.
- Independent of [Defer quality updates](#defer-quality-updates) (different value names, additive).
- Inert while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied; the pipeline tweak's own copy suggests pairing it with this one so the deferral is already in place when the block is lifted.

#### Validation
- **Verdict**: VERIFIED (July 2026 research). No mechanism correction; the correction was to the interaction: a pin makes this fully inert, not merely overridden.
- **Confidence**: Microsoft-documented.
- **Reasoning**: key, both value names, types and range match the shipped 26100 ADMX verbatim and Policy CSP. The adversarial pass attacked two things: whether the policy is deprecated on 24H2 (no), and whether it duplicates the quality deferral or the pin (no: different values, different behaviour).
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
A better choice than pinning for most people who want caution without risk. Use 180 days unless you have a reason for more or less. Do not apply it on a machine that has a version pin, and skip it on Home.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` and `en-US\WindowsUpdate.adml` on build 26100: policy `DeferFeatureUpdates`, the 0 to 365 range and the `PauseFeatureUpdatesStartTime` sibling element (tier A, primary)
2. Policy CSP, Update, `DeferFeatureUpdatesPeriodInDays`: build floor, editions, range and default, no deprecation banner, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
3. Walkthrough: use Group Policy to configure Windows Update client policies, "I want to stay on a specific version": "When you specify target version policy, feature update deferrals won't be in effect", https://learn.microsoft.com/en-us/windows/deployment/update/waas-wufb-group-policy (tier A)
4. What are Windows Update client policies?: supported editions and the 365-day maximum, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Pin Windows feature version

`target_release_version` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Locks the PC to one Windows 11 feature version (25H2 or 26H1), so it keeps getting monthly security patches but does not jump to a newer release on its own.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `target_enabled` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `TargetReleaseVersion` (`REG_DWORD`) |
| `target_version` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `TargetReleaseVersionInfo` (`REG_SZ`) |
| `target_product` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `ProductVersion` (`REG_SZ`) |

| Option | `target_enabled` | `target_version` | `target_product` |
|---|---|---|---|
| Pinned to 25H2 | `1` | `"25H2"` | `"Windows 11"` |
| Pinned to 26H1 | `1` | `"26H1"` | `"Windows 11"` |
| Not pinned | `absent` | `absent` | `absent` |

System Default: any other combination (for example a pin to 24H2 set elsewhere); selecting it restores the snapshot. The stock state is all three absent ("Not pinned").

The tweak carries a UI warning: a pinned release stops receiving security updates the day its servicing ends (25H2 on 2027-10-12, 26H1 on 2028-03-14), and 24H2 is deliberately not offered because its Home and Pro servicing ends 2026-10-13.

#### How it works

These values are what the Group Policy "Select the target Feature Update version" writes: a DWORD enable flag plus two strings. Policy CSP describes `TargetReleaseVersion` as letting an administrator "specify which version they would like their device(s) to move to and/or stay on until they reach end of service or reconfigure the policy", as a version string such as `1809` or `25H2`, and `ProductVersion` as a product string such as `"Windows 11"`, with the note "You need to set up the ProductVersion CSP along with the TargetReleaseVersion CSP for it to work." The types matter: the version as a DWORD would be silently ignored.

"Move to and/or stay on" is literal. On a 24H2 machine, "Pinned to 25H2" makes 25H2 the target, so Windows Update offers the 25H2 feature update and then holds there; it does not freeze the machine on 24H2. On a Windows 10 machine, `ProductVersion = "Windows 11"` asks Windows Update for Windows 11. Microsoft attaches a licensing attestation to using `ProductVersion` to move a device to a new product: the licence was bought through volume licensing, or the person applying it is authorised to accept the licence terms on an organization's behalf.

Applicability: `TargetReleaseVersion` needs Windows 10 2004 or later (or older builds with specific KBs); `ProductVersion` needs Windows 10 2004 with KB5005101 or Windows 11 21H2 and later. Editions: Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. Home ignores it. Monthly quality updates keep arriving for the pinned version until its end of servicing, after which the device stops receiving security updates entirely.

#### Benefits
- No surprise feature upgrades; the annual update lands only when you move the pin.
- Security patches continue for the pinned version.
- The supported route for version control, replacing older deferral-by-days pinning guidance.

#### Drawbacks
- Silent end of updates once the pinned release reaches end of servicing.
- Does nothing on Home.
- Cancels feature-update deferrals entirely.
- Licensing intent: Microsoft frames product moves via this policy as an organizational, volume-licensed action.
- On a machine older than the pinned version, applying the pin triggers the upgrade to it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 2004 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. Windows Home ignores it.
- **Takes effect**: at the next Windows Update scan.
- **Reverting**: "Not pinned" deletes all three values, restoring the normal upgrade path; System Default restores the snapshot. A feature update already installed because of the pin is not rolled back.

#### Interactions
- **Makes [Defer feature updates](#defer-feature-updates) fully inert.** Pick one.
- Works with [Defer quality updates](#defer-quality-updates).
- Makes the device "update-managed", which activates temporary feature control in [Control mid-cycle features and optional content](#control-mid-cycle-features-and-optional-content).
- Inert while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). The three-value shape and types were confirmed; the correction was that the offered pin must track currently serviced releases (24H2 servicing ends 2026-10-13) and that the pin is mutually exclusive with feature deferral.
- **Confidence**: Microsoft-documented.
- **Reasoning**: value names, types, the requirement to set both strings, the editions and the licensing language come from Policy CSP; release and end-of-servicing dates from Windows 11 release information. The policy-hive audit confirmed all three values are Machine-class in HKLM. The adversarial pass attacked whether the offered versions have enough servicing left (24H2 is excluded because servicing ends 2026-10-13) and the interaction with the new feature-deferral tweak.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Good on Pro and above if you want to decide when to take a feature upgrade, provided you pin a release with plenty of servicing left and set a reminder to move it. Do not use it if nobody will move the pin forward, and remember it moves an older machine up to the pinned version.

#### Sources
1. Policy CSP, Update, `TargetReleaseVersion` and `ProductVersion`: value forms, "move to and/or stay on", the requirement to set both, editions and the licensing language, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
2. Windows 11 release information: availability and end-of-servicing dates for 24H2, 25H2 and 26H1, https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information (tier A)
3. Walkthrough: use Group Policy to configure Windows Update client policies: "When you specify target version policy, feature update deferrals won't be in effect", https://learn.microsoft.com/en-us/windows/deployment/update/waas-wufb-group-policy (tier A)
4. What are Windows Update client policies?: supported editions, Home excluded, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Control mid-cycle features and optional content

`update_feature_control` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds (the `temporary_feature_control` effect only on build 22621 and newer) · Reversible: yes

**Stops new features and optional preview updates arriving inside monthly updates, so the installed Windows stops changing shape between feature updates; or, in the other direction, opts in to them.**

#### What it changes

| Effect | Kind | Target | Scope |
|---|---|---|---|
| `temporary_feature_control` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `AllowTemporaryEnterpriseFeatureControl` (`REG_DWORD`) | `windows: { build: ">=22621" }` |
| `set_allow_optional_content` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `SetAllowOptionalContent` (`REG_DWORD`) | all builds |
| `allow_optional_content` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `AllowOptionalContent` (`REG_DWORD`) | all builds |

| Option | `temporary_feature_control` | `set_allow_optional_content` | `allow_optional_content` |
|---|---|---|---|
| No mid-cycle features or optional content | `0` | `0` | `absent` |
| Receive optional content and gradual feature rollouts | `1` | `1` | `1` |
| Windows decides | `absent` | `absent` | `absent` |

System Default: any other combination (for example `AllowOptionalContent = 2` or `3` set elsewhere); selecting it restores the snapshot. The stock state is all three absent ("Windows decides"). On a build below 22621 the first effect is out of scope and the options are told apart by the other two.

#### How it works

Since Windows 11 22H2 Microsoft ships new features inside monthly quality updates, turned on gradually ("controlled feature rollouts"), and offers optional non-security preview updates through the same channel. Neither a quality deferral nor a version pin stops a feature arriving inside a monthly cumulative update; these two policies do.

`AllowTemporaryEnterpriseFeatureControl` (shipped ADMX, `SUPPORTED_Windows_11_0_22H2`, enabled 1 / disabled 0). The ADML help: "Features introduced via servicing (outside of the annual feature update) are off by default for devices that have their Windows updates managed. If this policy is configured to 'Enabled', then all features available in the latest monthly quality update installed will be on. If this policy is set to 'Not Configured' or 'Disabled' then features that are shipped via a monthly quality update (servicing) will remain off until the feature update that includes these features is installed." It adds that "Windows update managed devices are those that have their Windows updates managed via policy". So this half acts only on an update-managed device. Setting a quality deferral or a target release version makes the device managed; whether this value alone qualifies is not stated by any tier A source.

`AllowOptionalContent` is a Group Policy that writes two values: `SetAllowOptionalContent` as a 1/0 enable flag, and a separate `AllowOptionalContent` enum holding the selection. Policy CSP gives the selection as 0 (default) don't receive optional updates, 1 automatically receive optional updates including gradual feature rollouts (CFRs), 2 automatically receive optional cumulative updates only, 3 users choose. Note that 1 is more permissive than 2. The stability option writes the enable flag as 0 and leaves the selector absent; the permissive option writes 1 everywhere, the most permissive selection.

The two halves have different floors: temporary feature control needs Windows 11 22H2 or later, hence the build gate on that effect; the optional-content pair (`WU_SUPPORTED_WinServer2025_Win1021H2_Win1122H2`) reaches back to Windows 10 21H2 build 19044.3757, so a patched LTSC 2021 honours that half. Both are Pro and above; Home ignores them.

#### Benefits
- A stable install that does not gain new UI or behaviour mid-cycle.
- No optional non-security preview updates or optional driver updates offered.
- Nothing breaks; features arrive with the next feature update.
- A real opposite option for people who want features early.

#### Drawbacks
- Temporary feature control may do nothing on an unmanaged PC with no other Windows Update policy.
- Does nothing on Home.
- Improvements shipped as features arrive later.
- Split applicability: the feature-control half does not exist on Windows 10 LTSC 2021.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC (both halves). Windows 10 21H2 build 19044.3757 and later, including a patched LTSC 2021: optional-content half only. Windows Home ignores both.
- **Takes effect**: at the next Windows Update scan.
- **Reverting**: "Windows decides" deletes all three values; System Default restores the snapshot. Features already turned on are not turned off again by reverting the permissive option.

#### Interactions
- [Defer quality updates](#defer-quality-updates) or [Pin Windows feature version](#pin-windows-feature-version) make the device update-managed, which is what activates the temporary-feature-control half.
- Disabling optional content also suppresses optional driver updates, which overlaps with [Exclude driver updates from Windows Update](#exclude-driver-updates-from-windows-update) from a different angle and value.
- The research considered merging this with [Block Insider preview builds by policy](#block-insider-preview-builds-by-policy) and recommended against it (orthogonal decisions, different floors).

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). `SetAllowOptionalContent` is only an enable flag; the selection lives in a separate `AllowOptionalContent` value where 1 is more permissive than 2; and the two halves need two different applicability gates.
- **Confidence**: Microsoft-documented (shipped 26100 ADMX and ADML, Policy CSP).
- **Reasoning**: every value, polarity and floor was read from the shipped ADMX and cross-checked against Policy CSP. The adversarial pass established that a single value and a single gate cannot express the policy; it needs three values and a per-effect build gate. Open question: whether `AllowTemporaryEnterpriseFeatureControl` alone makes a device update-managed.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying on a Pro machine you want to stop changing shape month to month, particularly alongside a quality deferral that makes the device update-managed. Skip it if you like getting features as soon as they ship, and skip it on Home.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` and `en-US\WindowsUpdate.adml` on build 26100: policies `AllowTemporaryEnterpriseFeatureControl` and `AllowOptionalContent`, the enum element and both `supportedOn` references (tier A, primary)
2. Policy CSP, Update, `AllowOptionalContent`: the 0/1/2/3 semantics, editions and the Windows 10 21H2 build floor, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
3. Configure Windows Update client policies: what temporary enterprise feature control is and what update-managed means, https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb (tier A)
4. What are Windows Update client policies?: supported editions, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Block Insider preview builds by policy

`block_insider_builds_policy` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Blocks enrolment in the Windows Insider Program with the policy value that the Settings app cannot override.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `manage_preview_builds` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `ManagePreviewBuildsPolicyValue` (`REG_DWORD`) |

| Option | `manage_preview_builds` |
|---|---|
| Preview builds blocked | `1` |
| Preview builds allowed | `absent` |

System Default: the value holds something else (for example 2, the policy's Enabled state set elsewhere); selecting it restores the snapshot. The stock state is value-absent ("Preview builds allowed").

#### How it works

The shipped 26100 `WindowsUpdate.admx` defines one policy here, named `ManagePreviewBuilds` ("Manage preview builds", `SUPPORTED_Windows_10_0_RS3`, so Windows 10 1709 and later), which writes exactly one flag value, `ManagePreviewBuildsPolicyValue`, with `enabledValue` 2 and `disabledValue` 1. The blocking state is the policy's Disabled state, which writes 1. Its companion element is `BranchReadinessLevel` (Dev 2, Beta 4, Release Preview 8, Release Preview quality-only 64), which selects a preview channel; this tweak writes only the blocking value and leaves the channel alone. There is no `ManagePreviewBuilds` DWORD written by Group Policy, and 0 is not a value the ADMX writes.

Policy CSP documents the setting on a different 0 to 3 scale (0 disable, 1 disable once the next release is public, 2 enable, 3 user selection) under the same registry key. Whether the update stack also reads a literal `ManagePreviewBuilds` value written by MDM was not established, so only the ADMX-backed value ships. Editions per Policy CSP: Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC; Home is excluded.

#### Benefits
- Cannot be undone from Settings; unlike a service, a policy cannot be restarted around.
- Closes the route from the Windows Update settings page into preview builds.
- Complements the service-level Insider tweak, which closes a different route.

#### Drawbacks
- No Insider builds at all on this machine.
- Does nothing on Home.
- Affects only preview builds, not ordinary feature or quality updates.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. Windows Home ignores it.
- **Takes effect**: immediately (policy is read on the next Windows Update and Insider check).
- **Reverting**: "Preview builds allowed" deletes the value, returning the choice to the user; System Default restores the snapshot. A machine already enrolled in a preview channel before the block is not moved back by reverting.

#### Interactions
- `disable_windows_insider` in the Services category disables the `wisvc` Insider service. The research recommended keeping both: the service is the weaker lever (it can be restarted), the policy cannot be bypassed from Settings.
- The research considered merging this with [Control mid-cycle features and optional content](#control-mid-cycle-features-and-optional-content) and recommended keeping them separate.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). The value is `ManagePreviewBuildsPolicyValue`, not `ManagePreviewBuilds`, and the blocking value is 1 (the policy's Disabled state), not 0; and a Pro-and-above edition gate applies.
- **Confidence**: Microsoft-documented (shipped 26100 ADMX, Policy CSP).
- **Reasoning**: the ADMX was read verbatim; `ManagePreviewBuilds` and the value 0 are not what the ADMX writes; the tweak writes `ManagePreviewBuildsPolicyValue = 1`. Open question: the relationship between the CSP's 0 to 3 scale and the ADMX's 1/2 flag.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any Pro machine that should never run preview builds, especially a shared or work machine. Do not apply it on a box you use to test upcoming Windows releases.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` on build 26100: policy `ManagePreviewBuilds`, `valueName="ManagePreviewBuildsPolicyValue"`, enabled 2, disabled 1, `SUPPORTED_Windows_10_0_RS3`, and the `BranchReadinessLevel` enum (tier A, primary)
2. Policy CSP, Update, `ManagePreviewBuilds`: registry key and editions, Home excluded, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
3. What are Windows Update client policies?: supported editions, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Set active hours

`set_active_hours` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Tells Windows you use the PC from 8:00 to 23:00, so automatic update restarts are scheduled outside those hours, and stops Windows re-guessing the window.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `smart_active_hours` | registry | `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` → `SmartActiveHoursState` (`REG_DWORD`) |
| `active_hours_start` | registry | `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` → `ActiveHoursStart` (`REG_DWORD`) |
| `active_hours_end` | registry | `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` → `ActiveHoursEnd` (`REG_DWORD`) |

| Option | `smart_active_hours` | `active_hours_start` | `active_hours_end` |
|---|---|---|---|
| Manual 8:00 to 23:00 | `2` | `8` | `23` |

System Default: anything else, which on almost every machine is the starting state; selecting it restores the snapshot, which puts back the user's previous start and end hours and the previous `SmartActiveHoursState` (normally absent). There is deliberately no second option: the machine's real active hours are whatever the user or Windows already set, so the honest return path is the snapshot. On the 26100 test machine the key held `ActiveHoursStart = 16`, `ActiveHoursEnd = 10` and no `SmartActiveHoursState` (measured).

#### How it works

`HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` is the internal store the Settings app writes when a user sets active hours by hand (Settings > Windows Update > Advanced options > Active hours), and Windows reads the hours from it. It is not a policy key, which is why this works on every edition including Home. Microsoft documents the hours only for the policy form (`ActiveHoursStart` and `ActiveHoursEnd` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, the Group Policy "Turn off auto-restart for updates during active hours"): range 0 to 23, default 8 AM to 5 PM, maximum span 18 hours (12 on Windows 10 1607). The tweak's 15-hour span is legal.

`SmartActiveHoursState` controls "Automatically adjust active hours for this device based on activity". Microsoft documents neither the value nor its range. 2 meaning off comes from Winaero alone; other community sources give 0 and 1 instead. With the policy key set (for example by an organization), the policy overrides this store.

#### Benefits
- Restarts move to hours you are not at the machine.
- Turning automatic adjustment off stops Windows revising the window behind you.
- Works on every edition, because it writes the same store the Settings UI writes.

#### Drawbacks
- Not a "never restart" switch: Windows can still restart after a deadline passes.
- The window is capped at 18 hours, so a full day cannot be covered.
- Applying overwrites your existing hours (the snapshot keeps them for revert).
- `SmartActiveHoursState = 2` rests on community sources only.
- A user-writable store: changing active hours in Settings afterwards moves the tweak to System Default.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021; all editions.
- **Takes effect**: immediately.
- **Reverting**: System Default restores the previous hours from the snapshot and removes `SmartActiveHoursState` if it was absent before.

#### Interactions
- Microsoft states active hours has no effect when "No auto-restart with logged on users" or "Always automatically restart at scheduled time" is enabled, so this and [Block auto-restart while signed in](#block-auto-restart-while-signed-in) partly cancel each other (only when the reboot block is actually active, that is with [Windows Update mode](#windows-update-mode) on Automatic).
- A policy-key `ActiveHoursStart`/`ActiveHoursEnd` set by an organization overrides this store.
- Irrelevant while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). The stock state has no `SmartActiveHoursState` value and keeps the user's own hours; a "stock default" option writing 1, 8 and 17 would have fabricated state and destroyed the user's hours, so the return path is the snapshot. The store is the Windows-internal settings key, not the policy key.
- **Confidence**: community-corroborated (the hours and store are confirmed by Microsoft's policy documentation and a direct registry read; `SmartActiveHoursState` semantics are tier C only).
- **Reasoning**: the measured registry read confirms Windows reads active hours from this key and does not seed `SmartActiveHoursState`. The adversarial pass attacked the revert values and the undocumented value; both led to the current single-option, snapshot-restore shape. Open question: the exact semantics of `SmartActiveHoursState`.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Worth setting for anyone annoyed by mistimed restarts whose working day fits inside 8:00 to 23:00. Expect fewer badly timed reboots, not none.

#### Sources
1. Manage device restarts after updates, "Configure active hours": the 8 AM to 5 PM default, the 18-hour cap, and the rule that active hours has no effect alongside the no-auto-restart policies, https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A)
2. Policy CSP, Update, `ActiveHoursStart`, `ActiveHoursEnd`, `ActiveHoursMaxRange`: ranges and defaults for the policy form, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
3. Enable Automatically Adjust Active Hours in Windows 10, Winaero: sole source for `SmartActiveHoursState` 1 = on, 2 = off, contradicted by other community sources, https://winaero.com/automatically-adjust-active-hours-windows-10/ (tier C)
4. Direct registry read of `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` on Windows 11 24H2 build 26100: `ActiveHoursStart = 16`, `ActiveHoursEnd = 10`, no `SmartActiveHoursState` (primary measurement)

### Block auto-restart while signed in

`disable_auto_restart_logged_on` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows Update rebooting the PC on its own after a scheduled install while someone is signed in, protecting open work.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `no_auto_reboot_logged_on` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU` → `NoAutoRebootWithLoggedOnUsers` (`REG_DWORD`) |

| Option | `no_auto_reboot_logged_on` |
|---|---|
| Blocked | `1` |
| Allowed | `absent` |

System Default: the value holds something else (for example 0 written by an organization); selecting it restores the snapshot. The stock state is value-absent ("Allowed").

The tweak carries a UI warning: Microsoft applies this policy only when Configure Automatic Updates is set to option 4; on a machine that has not configured automatic updates the value is written but has no effect.

#### How it works

This is the Group Policy "No auto-restart with logged on users for scheduled automatic updates installations". Microsoft's AU key table: "`0`: If users are signed in, automatically restart... `1`: If a user is signed in, don't restart after an update installation." The gating condition is explicit: "This policy only applies when Configure Automatic Updates is set to option 4 - Auto download and schedule the install." In this app that is [Windows Update mode](#windows-update-mode) set to "Automatic: download and install" (`NoAutoUpdate = 0`, `AUOptions = 4`). With the mode unset, or on any other mode, the value sits in the registry with nothing to gate.

Microsoft's own caveats on the same page: "In Group Policy this policy doesn't work exactly as per description. This policy can result in no quality update reboots period, given many users never log off." It recommends compliance deadlines instead. The policy "was never created as a CSP". Over RDP only active sessions count as signed-in users, so a machine with only disconnected sessions restarts anyway. If a user schedules the restart in the update notification, the device restarts at that time even with someone signed in. Home does not honour the Windows Update AU policies.

#### Benefits
- Unsaved documents and long-running sessions survive update night.
- Windows still notifies; you choose the moment.
- Policy-backed, so a later Settings change does not override it.

#### Drawbacks
- Inert on its own; it needs Windows Update mode on Automatic.
- Patches can stall indefinitely on a machine nobody signs out of.
- Does nothing on Home.
- Cancels active hours while active.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, on Pro, Education, Enterprise and IoT Enterprise. Windows Home does not honour it.
- **Takes effect**: at the next scheduled install cycle.
- **Reverting**: "Allowed" deletes the value; System Default restores the snapshot.

#### Interactions
- **Requires [Windows Update mode](#windows-update-mode) on "Automatic: download and install"** to do anything; every other mode makes it inert.
- Makes [Set active hours](#set-active-hours) ineffective while it is active.
- The research proposed folding this into the automatic-update tweak; it ships as a separate tweak that owns a different value in the same AU key, so both can be applied together.
- Irrelevant while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). Key, value, type and polarity are exact; the correction was the missing companion: Microsoft documents the policy as applying only with `AUOptions = 4`, which the separate Windows Update mode tweak provides.
- **Confidence**: Microsoft-documented.
- **Reasoning**: the AU registry table and the option-4 gating sentence are on Microsoft's restart page (re-read in September 2026). The policy-hive audit confirmed Machine class in HKLM. The adversarial pass attacked the standalone value (inert) and Microsoft's own caveats on the policy.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Use it on Pro alongside Windows Update mode set to Automatic if you keep long work sessions open, and restart soon after updates so the fixes actually apply. Skip it on Home and on machines that are simply never restarted.

#### Sources
1. Manage device restarts after updates, "Delay automatic restart" and the AU registry table: 0/1 semantics, the option-4 condition, the "no quality update reboots period" caveat, the never-a-CSP note and RDP behaviour, https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A)
2. Manage additional Windows Update settings: the AU key and its companion values, https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings (tier A)
3. What are Windows Update client policies?: supported editions, Home excluded, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Exclude driver updates from Windows Update

`exclude_wu_driver_updates` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows Update delivering hardware drivers, so you decide which driver versions the PC runs; security and quality updates are unaffected.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `exclude_wu_drivers` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `ExcludeWUDriversInQualityUpdate` (`REG_DWORD`) |

| Option | `exclude_wu_drivers` |
|---|---|
| Excluded | `1` |
| Included | `absent` |

System Default: the policy value holds something else (for example 0); selecting it restores the snapshot. The stock state is value-absent ("Included").

#### How it works

This is the Group Policy "Do not include drivers with Windows Updates" (`WindowsUpdate.admx`). Policy CSP gives the mapping exactly: key `Software\Policies\Microsoft\Windows\WindowsUpdate`, value `ExcludeWUDriversInQualityUpdate`, 0 (default) allow drivers, 1 exclude. The Windows Update client stops offering updates with the Driver classification at the next scan.

Microsoft's scope limit: "This policy won't apply to updates to drivers provided with the operating system (which will be packaged within a security or critical update) or to feature updates, where drivers might be dynamically installed to ensure the feature update process can complete." A same-named value also appears under `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` on real machines (measured); that is the Settings mirror, and the policy key governs. It is a Windows Update client policy, so Home ignores it.

#### Benefits
- No surprise swaps of a working GPU, chipset or audio driver.
- Ends the tug of war with NVIDIA, AMD, Intel or board-vendor packages.
- Narrow blast radius: driver delivery only.

#### Drawbacks
- You own driver updates, including their security and stability fixes.
- Does nothing on Home.
- Not a total block: OS-bundled drivers in security updates and drivers installed during a feature update still arrive.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. Windows Home ignores it.
- **Takes effect**: at the next Windows Update scan.
- **Reverting**: "Included" deletes the value; System Default restores the snapshot. Drivers held back meanwhile are offered again at the next scan.

#### Interactions
- Pairs with [Disable automatic driver installation](#disable-automatic-driver-installation), which stops the device-install driver search; together they give end-to-end driver control. The research proposed merging them; they ship separately.
- [Control mid-cycle features and optional content](#control-mid-cycle-features-and-optional-content) with optional content off also suppresses optional driver updates.
- Irrelevant while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). The mechanism is exact; the correction was applicability: it is a Pro-and-above client policy and does not work on Home.
- **Confidence**: Microsoft-documented.
- **Reasoning**: Policy CSP gives key, value, type and semantics verbatim; the policy-hive audit confirmed Machine class in HKLM. The adversarial pass refuted a Home claim and added Microsoft's scope limit.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Good for anyone who manages drivers from vendor sites and has been bitten by an automatic driver swap. Leave it off if you would rather not think about drivers, and do not bother on Home.

#### Sources
1. Policy CSP, Update, `ExcludeWUDriversInQualityUpdate`: key, value, type, 0/1 semantics, editions, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
2. Configure Windows Update client policies, "Exclude drivers from quality updates": the scope limit, https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb (tier A)
3. What are Windows Update client policies?: supported editions, Home excluded, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)

### Disable automatic driver installation

`disable_auto_driver_install` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows searching Windows Update for a driver when new hardware is attached, and stops it downloading device metadata (icons, names, details) from the network.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `search_order` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DriverSearching` → `SearchOrderConfig` (`REG_DWORD`) |
| `prevent_metadata` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Device Metadata` → `PreventDeviceMetadataFromNetwork` (`REG_DWORD`) |

| Option | `search_order` | `prevent_metadata` |
|---|---|---|
| Disabled | `0` | `1` |
| Enabled | `absent` | `absent` |

System Default: any other combination; selecting it restores the snapshot. The stock state is both policy values absent ("Enabled"); on the test machine the `Policies\...\Device Metadata` key did not exist (measured).

#### How it works

Both values are written to the documented policy keys. `SearchOrderConfig` under `Policies\Microsoft\Windows\DriverSearching` is the Group Policy "Specify search order for device driver source locations" (System > Device Installation), supported since Windows 7, whose three states are "Always search Windows Update", "Search Windows Update only if needed" and "Do not search Windows Update". The tweak writes 0 for the last; the numeric mapping is corroborated by community ADMX references rather than by a tier A page. `PreventDeviceMetadataFromNetwork` under `Policies\Microsoft\Windows\Device Metadata` is Microsoft-documented: Policy CSP DeviceInstallation and the privacy connection guide both give it, and Policy CSP states "This policy setting overrides the setting in the Device Installation Settings dialog box."

These policy keys take precedence over the Control Panel "Device Installation Settings" store (`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching` and `...\Device Metadata`), which the dialog writes. On the test machine that store held `SearchOrderConfig = 1` out of the box (measured); the tweak does not touch it. Device installation reads these values when a device is installed, which is why the tweak is marked reboot-advised (a reboot is not strictly required, but guarantees a clean read).

#### Benefits
- You curate every driver; nothing installs itself when hardware is attached.
- Fewer outbound requests: device metadata retrieval stops.
- Policy-backed, so the Control Panel dialog cannot silently undo it.

#### Drawbacks
- New hardware may sit without a driver until you install one by hand.
- The numeric meaning of `SearchOrderConfig = 0` is not given by a tier A source.
- Keep vendor driver packages handy before adding new devices.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021. The metadata policy is documented in Policy CSP for Pro and above; the driver search-order policy is an ordinary machine policy. Home behaviour for these policy keys is not documented.
- **Takes effect**: on the next device installation; reboot advised.
- **Reverting**: "Enabled" deletes both policy values, returning control to the Device Installation Settings store; System Default restores the snapshot. Devices that went undriven meanwhile are not retried automatically until they are re-enumerated.

#### Interactions
- Pairs with [Exclude driver updates from Windows Update](#exclude-driver-updates-from-windows-update). The research proposed merging them and noted the metadata value could instead live in a privacy category; they ship as written here.
- [Block the Windows Update pipeline](#block-the-windows-update-pipeline) also prevents driver searches reaching Windows Update, as a side effect.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). Both values belong under the `SOFTWARE\Policies\...` keys, not the Control Panel store; on the policy path the stock state is absent, not 1. The shipped YAML writes the policy keys.
- **Confidence**: community-corroborated. `PreventDeviceMetadataFromNetwork` at this key is Microsoft-documented; the `SearchOrderConfig` path is from a tier C ADMX reference, and its value 0 is not defined by a tier A or B source.
- **Reasoning**: the precedence of the policy over the dialog is Microsoft's own statement; measurements established the stock state of both stores. The adversarial pass established that both values belong under the policy keys. Open question: a tier A definition of `SearchOrderConfig = 0`.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
For power users who want to approve every driver, ideally alongside excluding drivers from Windows Update. Skip it if you plug in new peripherals often and expect them to work immediately.

#### Sources
1. Policy CSP, DeviceInstallation, `PreventDeviceMetadataFromNetwork`: the policy key and that it overrides the Device Installation Settings dialog, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-deviceinstallation (tier A)
2. Manage connections from Windows to Microsoft services, section 4 "Device metadata retrieval": the policy path and `REG_DWORD` type, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Specify search order for device driver source locations: the policy's registry path under `Policies\...\DriverSearching` and its three states, https://www.windows-security.org/4e9792f8fcb59047c7a25891728d5f0c/specify-search-order-for-device-driver-source-locations (tier C)
4. Direct registry read on Windows 11 24H2 build 26100: `SearchOrderConfig = 1` on the non-policy path, `Policies\...\Device Metadata` absent (primary measurement)

### Disable Microsoft Store auto-updates

`disable_store_auto_updates` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the Microsoft Store updating your apps in the background, so app versions change only when you update them from the Store Library.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `store_auto_download` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsStore` → `AutoDownload` (`REG_DWORD`) |

| Option | `store_auto_download` |
|---|---|
| Disabled | `2` |
| Enabled | `absent` |

System Default: the value holds something else, typically 4 (the policy's Disabled state, which forces auto-updates on); selecting it restores the snapshot. The stock state is value-absent ("Enabled").

#### How it works

Microsoft's privacy guidance says directly: create a `REG_DWORD` named `AutoDownload` in `HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\WindowsStore` with a value of 2, as the registry form of enabling the Group Policy "Turn off Automatic Download and Install of updates" (Windows Components > Store). Policy CSP `AllowAppStoreAutoUpdate` confirms the mapping (ADMX name `DisableAutoInstall`, `WindowsStore.admx`, value `AutoDownload`). The ADMX writes 2 when Enabled and 4 when Disabled, so 4 explicitly forces auto-updates on, while deleting the value returns the choice to the Store's own setting. Recent Windows 11 Store builds reduced the in-app "off" toggle to a pause; the policy value still applies.

#### Benefits
- No background app downloads, useful on metered or slow connections.
- App versions stay stable until you choose.
- Survives the Store UI, which no longer offers a permanent off switch.

#### Drawbacks
- Apps go stale, including Store apps that ship security fixes.
- Manual upkeep from the Store Library.
- Home support is not documented by Microsoft (likely but unconfirmed).

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1507 and later including LTSC 2021 (where the Store is installed). Policy CSP lists Pro, Enterprise, Education and IoT Enterprise; the privacy guide gives the registry form with no edition caveat.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" deletes the value, restoring automatic Store updates; System Default restores the snapshot.

#### Interactions
- [Block the Windows Update pipeline](#block-the-windows-update-pipeline) deliberately leaves the Store's `InstallService` tasks alone, but see its entry: disabling `wuauserv` and the WSUS redirect may affect Store downloads.
- Independent of [Disable Delivery Optimization P2P](#disable-delivery-optimization-p2p), which governs how Store downloads are fetched, not whether they happen.

#### Validation
- **Verdict**: VERIFIED (July 2026 research). No correction.
- **Confidence**: Microsoft-documented.
- **Reasoning**: Microsoft's privacy guide gives key, name, type and value verbatim; Policy CSP confirms the ADMX mapping; the policy-hive audit confirmed Machine class in HKLM. Open question: Home behaviour.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Reasonable on a metered connection or where you want app versions pinned, if you check the Store Library now and then. If you would forget, leave auto-updates on so security fixes keep arriving.

#### Sources
1. Manage connections from Windows to Microsoft services, section 26 "Microsoft Store": key, `AutoDownload`, `REG_DWORD`, value 2, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
2. Policy CSP, ApplicationManagement, `AllowAppStoreAutoUpdate`: ADMX mapping and editions, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-applicationmanagement (tier A)
3. Turn off Automatic Download and Install of updates, ADMX Viewer: Enabled = 2, Disabled = 4, https://gpedit.tplant.com.au/en-us/policy/WindowsStore/DisableAutoInstall/ (tier C)

### Block auto-download over metered

`block_update_over_metered` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Locks "no automatic update downloads over metered connections" into policy, so neither Settings nor another tool can turn it back on.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `metered_download` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `AllowAutoWindowsUpdateDownloadOverMeteredNetwork` (`REG_DWORD`) |

| Option | `metered_download` |
|---|---|
| Blocked | `0` |
| Allowed | `absent` |

System Default: the value holds something else (1, allowed); selecting it restores the snapshot. The stock state is value-absent ("Allowed" as labelled, which in practice means "decided by the Settings toggle").

#### How it works

The shipped 26100 `WindowsUpdate.admx` declares policy `AllowAutoWindowsUpdateDownloadOverMeteredNetwork` (Machine class, enabled 1, disabled 0) under the Windows Update policy key. Policy CSP names it "Allow updates to be downloaded automatically over metered connections" with 0 not allowed and 1 allowed, and states the default outright: "Default Value: 0", "0 (Default) Not allowed".

So writing 0 does not change whether a stock machine downloads over a metered link: it already does not. What changes is where the decision lives. In the policy branch it takes precedence over the per-machine "Download updates over metered connections" toggle in Settings, so a later user, tool or image change cannot opt the machine in. On 26100.4061 neither the UX settings store nor the policy branch held a metered value (measured). The policy only acts on connections Windows considers metered (mark them in Settings), and Microsoft notes high-priority and critical updates can still download over a metered connection. Home ignores it.

#### Benefits
- The Settings toggle for metered downloads can no longer opt the machine in.
- The machine's intent is recorded in policy rather than left implicit.
- Costs nothing: the machine already behaves this way.

#### Drawbacks
- No visible change on a stock machine.
- Does nothing on Home.
- Not a full block: priority and critical updates can still come through.
- Needs the connection to be marked metered.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. Windows Home ignores it.
- **Takes effect**: at the next Windows Update scan.
- **Reverting**: "Allowed" deletes the value, returning the decision to the Settings toggle; System Default restores the snapshot.

#### Interactions
- The research proposed merging this with [Disable Delivery Optimization P2P](#disable-delivery-optimization-p2p) as a bandwidth posture; they ship separately and combine freely.
- Irrelevant while [Block the Windows Update pipeline](#block-the-windows-update-pipeline) is applied.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION (July 2026 research). Key, value, type, polarity and absent stock state are correct; the correction was that the documented default is already 0, so the tweak locks the existing default into policy rather than changing behaviour.
- **Confidence**: Microsoft-documented (Policy CSP and the shipped 26100 ADMX).
- **Reasoning**: the ADMX and Policy CSP agree on every field, and a measurement on 26100.4061 agreed with the absent stock state. The adversarial pass attacked the claimed benefit; the tweak survives as a lock, not a change.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
Worth setting on a Pro machine that regularly runs on a hotspot or capped plan, as a lock rather than a change. Skip it on Home.

#### Sources
1. Policy CSP, Update, `AllowAutoWindowsUpdateDownloadOverMeteredNetwork`: "Default Value: 0", editions and Group Policy mapping, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` on build 26100.4061: policy `AllowAutoWindowsUpdateDownloadOverMeteredNetwork`, Machine class, enabled 1 / disabled 0 (tier A)
3. What are Windows Update client policies?: supported editions, Home excluded, https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A)
4. Direct registry inspection on build 26100.4061: no metered value in the UX settings store or the policy branch (primary measurement)

### Windows Update mode

`windows_update_mode` · Dropdown (4 options) · Risk: high · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Puts Windows Update into one of four automatic-update modes: install automatically, download only, notify only, or off by policy.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `no_auto_update` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU` → `NoAutoUpdate` (`REG_DWORD`) |
| `au_options` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU` → `AUOptions` (`REG_DWORD`) |

| Option | `no_auto_update` | `au_options` |
|---|---|---|
| Automatic: download and install | `0` | `4` |
| Download only: I choose when to install | `0` | `3` |
| Notify only: nothing downloads unprompted | `0` | `2` |
| Off by policy | `1` | `absent` |

System Default: any other state, which on a stock machine is both values absent (no Configure Automatic Updates policy, so Windows uses its unmanaged default of automatic install and restart); selecting it restores the snapshot, so a machine that had no policy goes back to having none.

The tweak carries a UI warning: Windows Home ignores the AU policy entirely, and every mode except Automatic delays security patches.

#### How it works

These two values are exactly what the Group Policy "Configure Automatic Updates" (Windows Components > Windows Update > Manage end user experience) writes. Microsoft's registry table on "Manage additional Windows Update settings": `NoAutoUpdate` "0: Automatic Updates is enabled (default). 1: Automatic Updates is disabled." `AUOptions` "2: Notify of download and installation. 3: Automatically download and notify of installation. 4: Automatically download and scheduled installation." (5 is not available on Windows 10 or later, 7 is Server-only.) The same page describes the Group Policy options in user terms:

- **2, Notify for download and auto install**: "users are notified that updates are ready to be downloaded. After going to Settings > Update & security > Windows Update, users can download and install any available updates." Microsoft words this value three ways across its pages ("Notify of download and installation", "Notify for download and notify for installation of updates", "Notify for download and auto install"); the behaviour is the same: nothing downloads until the user acts.
- **3, Auto download and notify for install**: updates download in the background without interrupting the user; "When the downloads are complete, users will be notified that they're ready to install."
- **4, Auto download and schedule the install**: install on the Group Policy schedule. The tweak writes no `ScheduledInstallDay` or `ScheduledInstallTime`, so Windows uses its defaults (Policy CSP gives "Every day" as the default install day).
- **Disabled** (the Group Policy's Disabled state, which writes `NoAutoUpdate = 1`): "any updates that are available on Windows Update must be downloaded and installed manually", from Settings. With automatic updates off, `AUOptions` is ignored, so the tweak deletes it rather than leaving a stale value.

Microsoft states that when Automatic Updates is configured through these policy registry values, "the policy overrides the preferences that are set by the local administrative user", and removing the values brings those preferences back. The MDM equivalent is Policy CSP `Update/AllowAutoUpdate`, whose own description says an unmanaged device gets "Auto install and restart" and warns that turning automatic updates off "should be used only for systems under regulatory compliance, as you won't get security updates as well". Both Policy CSP and the client-policy editions list Pro, Enterprise, Education and IoT Enterprise; Home is not supported and ignores the AU policy entirely.

On Windows 11, a current Windows Central how-to walks through the same Group Policy and the same registry values (`AU\NoAutoUpdate = 1` to disable, `AU\AUOptions = 2` for notify-only) and reports that with option 2 "updates won't download automatically. Instead, you will see an 'Install now' button". No source found for this research shows 24H2 ignoring the AU policy on a supported edition. "Off by policy" is a scheduler setting, not a pipeline block: the update services keep running, a manual "Check for updates" still works, and other components can still start scans. To stop the pipeline itself, use [Block the Windows Update pipeline](#block-the-windows-update-pipeline).

#### Benefits
- One control for the whole automatic-update posture, instead of loose values that can contradict each other.
- "Notify only" stops background downloads entirely, useful on constrained connections and for people who review updates first.
- "Download only" keeps downloads current while letting you choose when to install and restart.
- "Automatic" locks the default behaviour into policy and is the prerequisite for [Block auto-restart while signed in](#block-auto-restart-while-signed-in).
- Everything stays reversible from the snapshot.

#### Drawbacks
- Does nothing on Home.
- Every mode except Automatic delays security fixes until you act, which is why the risk is high.
- "Off by policy" does not stop the pipeline; it stops the automatic schedule only.
- Any mode other than Automatic makes the auto-restart block inert.
- "Automatic" is not identical to the stock state: it is a policy lock to option 4 with the default daily schedule, where an unmanaged machine uses Microsoft's "auto install and restart" behaviour.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, on Pro, Education, Enterprise and IoT Enterprise. Windows Home ignores it.
- **Takes effect**: immediately; the Windows Update client reads the policy at its next scan.
- **Reverting**: System Default restores the values captured in the snapshot; a machine that had no policy returns to having none and to the user's own Settings preferences.

#### Interactions
- [Block auto-restart while signed in](#block-auto-restart-while-signed-in) works only with "Automatic: download and install".
- With "Off by policy", Microsoft documents that "Specify intranet Microsoft update service location" has no effect ("If the 'Configure Automatic Updates' policy is disabled, then this policy has no effect"). That weakens the WSUS-redirect layer of [Block the Windows Update pipeline](#block-the-windows-update-pipeline) if both are applied; the pipeline's services and tasks layers still hold.
- Deferral, pin and content tweaks in this category work under any mode; they govern what is offered, this governs what happens with it.

#### Validation
- **Verdict**: VERIFIED (September 2026 research, this page). The mechanism is the one the July 2026 research verified for `disable_auto_update_download` (VERIFIED, no mechanism correction); this tweak writes the same two values, with four options.
- **Confidence**: Microsoft-documented (registry table, Group Policy option text, Policy CSP), community-corroborated on Windows 11.
- **Reasoning**: every value the tweak writes is in Microsoft's registry table under exactly this key, and the four options correspond one to one with the Group Policy's options 2, 3 and 4 and its Disabled state. Removing `AUOptions` when `NoAutoUpdate = 1` matches Microsoft's "Automatic Updates is disabled" semantics. Attacked and survived: whether 24H2 still honours the AU policy (no contrary source found; a Windows 11 how-to confirms the behaviour), and whether Home honours it (it does not, which the warning states). Open questions: no real-machine test of each mode's observable behaviour on 26100 was provided for this page, and the Windows Central source is not dated to a specific build.
- **Tested**: build validation (schema, ownership and conflict checks).

#### Recommendation
"Notify only" is the balanced choice for a Pro machine you actually maintain. If there is any chance patches will pile up unattended, stay on Automatic; the security cost outweighs the control. Skip it on Home, where it does nothing.

#### Sources
1. Manage additional Windows Update settings: the AU key, `NoAutoUpdate` and `AUOptions` registry table, the Group Policy option descriptions, the Disabled behaviour, and policy-over-preference precedence, https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings (tier A, fetched September 2026)
2. Manage device restarts after updates, AU registry table: the second wording of `AUOptions` and the option-4 condition for the reboot block, https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A, fetched September 2026)
3. Policy CSP, Update, `AllowAutoUpdate`: the unmanaged default "Auto install and restart", the "Turn off automatic updates" value, the regulatory-compliance warning, editions (Home not listed), and the `ScheduledInstallDay` default, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A, fetched September 2026)
4. How to stop automatic updates on Windows 11, Windows Central: the same Group Policy and registry values on Windows 11 and the observed "Install now" behaviour with option 2, https://www.windowscentral.com/how-stop-automatic-updates-windows-11 (tier C, fetched September 2026)

### Block the Windows Update pipeline

`block_update_pipeline` · Switch · Risk: high · Elevation: admin (21 effects escalate to ti) · Reboot: yes · Windows: all supported builds · Reversible: yes

**Shuts Windows Update down at its roots: the Medic service that repairs it, every scan and notification task, the update services, the Settings update UI and the network path to Microsoft.**

#### What it changes

Effects run in declaration order, which is load-bearing: the Medic pair and service first, services last.

| # | Effect | Kind | Target | Presence | Elevation |
|---|---|---|---|---|---|
| 1 | `medic_remediation_task` | task | `\Microsoft\Windows\WaaSMedic\PerformRemediation` | `optional`, `if_missing: disabled` | ti |
| 2 | `medic_deferred_task` | task | `\Microsoft\Windows\WaaSMedic\DeferredWork` | `optional`, `if_missing: disabled` | ti |
| 3 | `medic_service` | service | `WaaSMedicSvc` | required | ti |
| 4 | `uso_schedule_scan` | task | `\Microsoft\Windows\UpdateOrchestrator\Schedule Scan` | `optional`, `if_missing: disabled` | ti |
| 5 | `uso_schedule_scan_static` | task | `\Microsoft\Windows\UpdateOrchestrator\Schedule Scan Static Task` | `optional`, `if_missing: disabled` | ti |
| 6 | `uso_schedule_work` | task | `\Microsoft\Windows\UpdateOrchestrator\Schedule Work` | `optional`, `if_missing: disabled` | ti |
| 7 | `uso_maintenance_work` | task | `\Microsoft\Windows\UpdateOrchestrator\Schedule Maintenance Work` | `optional`, `if_missing: disabled` | ti |
| 8 | `uso_wake_to_work` | task | `\Microsoft\Windows\UpdateOrchestrator\Schedule Wake To Work` | `optional`, `if_missing: disabled` | ti |
| 9 | `uso_report_policies` | task | `\Microsoft\Windows\UpdateOrchestrator\Report policies` | `optional`, `if_missing: disabled` | ti |
| 10 | `uso_oobe_expedite` | task | `\Microsoft\Windows\UpdateOrchestrator\Start Oobe Expedite Work` | `optional`, `if_missing: disabled` | ti |
| 11 | `uso_oobe_apps_scan` | task | `\Microsoft\Windows\UpdateOrchestrator\StartOobeAppsScan_LicenseAccepted` | `optional`, `if_missing: disabled` | ti |
| 12 | `uso_oobe_apps_scan_after` | task | `\Microsoft\Windows\UpdateOrchestrator\StartOobeAppsScanAfterUpdate` | `optional`, `if_missing: disabled` | ti |
| 13 | `uso_failover` | task | `\Microsoft\Windows\UpdateOrchestrator\UUS Failover Task` | `optional`, `if_missing: disabled` | ti |
| 14 | `uso_uie_orchestrator` | task | `\Microsoft\Windows\UpdateOrchestrator\UIEOrchestrator` | `optional`, `if_missing: disabled` | ti |
| 15 | `uso_retry_scan` | task | `\Microsoft\Windows\UpdateOrchestrator\Schedule Retry Scan` | `optional`, `if_missing: disabled` | ti |
| 16 | `uso_update_model` | task | `\Microsoft\Windows\UpdateOrchestrator\UpdateModelTask` | `optional`, `if_missing: disabled` | ti |
| 17 | `uso_ux_broker` | task | `\Microsoft\Windows\UpdateOrchestrator\USO_UxBroker` | `optional`, `if_missing: disabled` | ti |
| 18 | `wu_scheduled_start` | task | `\Microsoft\Windows\WindowsUpdate\Scheduled Start` | `optional`, `if_missing: disabled` | ti |
| 19 | `wu_refresh_gp_cache` | task | `\Microsoft\Windows\WindowsUpdate\Refresh Group Policy Cache` | `optional`, `if_missing: disabled` | ti |
| 20 | `wu_sih` | task | `\Microsoft\Windows\WindowsUpdate\sih` | `optional`, `if_missing: disabled` | ti |
| 21 | `wu_sihboot` | task | `\Microsoft\Windows\WindowsUpdate\sihboot` | `optional`, `if_missing: disabled` | ti |
| 22 | `uso_svc` | service | `UsoSvc` | required | admin |
| 23 | `wuauserv` | service | `wuauserv` | required | admin |
| 24 | `disable_wu_ux_access` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `SetDisableUXWUAccess` (`REG_DWORD`) | required | admin |
| 25 | `set_notification_level` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `SetUpdateNotificationLevel` (`REG_DWORD`) | required | admin |
| 26 | `notification_level` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `UpdateNotificationLevel` (`REG_DWORD`) | required | admin |
| 27 | `no_wu_internet` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `DoNotConnectToWindowsUpdateInternetLocations` (`REG_DWORD`) | required | admin |
| 28 | `use_wu_server` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU` → `UseWUServer` (`REG_DWORD`) | required | admin |
| 29 | `wu_server` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `WUServer` (`REG_SZ`) | required | admin |
| 30 | `wu_status_server` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` → `WUStatusServer` (`REG_SZ`) | required | admin |

There is one authored option, "Blocked", which drives:

| Effects | Value in "Blocked" |
|---|---|
| All 20 tasks (rows 1, 2, 4 to 21) | `disabled` |
| `medic_service`, `uso_svc`, `wuauserv` | `disabled` (start type) |
| `disable_wu_ux_access` | `1` |
| `set_notification_level` | `1` |
| `notification_level` | `2` |
| `no_wu_internet` | `1` |
| `use_wu_server` | `1` |
| `wu_server` | `'http://127.0.0.1:8530'` |
| `wu_status_server` | `'http://127.0.0.1:8530'` |

System Default: anything other than all 30 effects in their blocked state, which is the normal state of every machine; selecting it restores every service start type, task state and registry value to exactly what the snapshot captured, not to a guessed default. For reference, on the 26100 test machine `wuauserv` and `WaaSMedicSvc` were demand-start (Manual); the Windows servicing manifest for `wuauserv` on 26100 also declares demand start, and Microsoft's Server 2016 service guide lists both `wuauserv` and `UsoSvc` as Manual. Stock start types vary by build, which is why the snapshot, not a table, is the return path.

The tweak carries a UI warning: it stops all Windows security updates until reverted, the Windows Update page shows connection errors by design, every task effect needs TrustedInstaller, and a restart is required before the block is fully effective.

#### How it works

**Why disabling `wuauserv` alone does not hold.** Windows ships a Windows Update Medic Service (`WaaSMedicSvc`) whose purpose, in the community's consistent description, is "remediation and protection of Windows Update components": "even if you disable Windows Update related Services, this service will at some point in time re-enable them." Users report the same thing from the other side: the update service "re-enabling itself each day and running after I disable it", and "Access Denied" when an administrator tries to change the Medic service. Its `PerformRemediation` task "performs recovery actions for update-related services to ensure they run in a supported configuration". So the tweak disables the reverter first: both WaaSMedic tasks, then the service. On 26100, `sc qtriggerinfo WaaSMedicSvc` reports "The service WaaSMedicSvc has not registered for any start or stop triggers", and the service is demand-start, so with its tasks disabled and its start type set to Disabled nothing is left to launch it.

**The scan triggers.** Windows Update is driven by the Update Orchestrator (Microsoft: the Orchestrator "schedules the scan", "starts downloads", "starts the installation" and "starts a restart"). Its scheduled tasks under `\Microsoft\Windows\UpdateOrchestrator` launch `usoclient.exe` (for example Schedule Scan runs `usoclient.exe StartScan`) and re-arm scans after a reboot; the tweak disables every one of them that exists, plus the `\Microsoft\Windows\WindowsUpdate` folder tasks (Scheduled Start, Refresh Group Policy Cache, and the Windows 10 era `sih`/`sihboot` Service Initiated Healing tasks). The list matches the community inventory of Windows Update scheduled tasks, apart from `StartOobeAppsScan_OobeAppReady`, which that inventory lists and this tweak does not drive.

**The nagging.** `USO_UxBroker` runs `MusNotification.exe`, the "your device is missing important updates" prompt that appears once updates stop. It is disabled, and update notifications are switched off by policy: `SetUpdateNotificationLevel = 1` enables the "Display options for update notifications" policy and `UpdateNotificationLevel = 2` selects "Turn off all notifications, including restart warnings" (Policy CSP values 0, 1, 2).

**The UI.** `SetDisableUXWUAccess = 1` is "Remove access to use all Windows Update features": in Microsoft's words it disables "the 'Check for updates' option for users", and Policy CSP says "user access to Windows Update scan, download and install is removed". Microsoft adds that "Any background update scans, downloads, and installations will continue to work as configured", which is why this is only the UI layer, backed by the task and service layers.

**The network path.** `UseWUServer = 1` in the AU key tells Automatic Updates to use an intranet update server instead of Windows Update, and `WUServer`/`WUStatusServer` name it; the tweak points both at `http://127.0.0.1:8530`, the conventional WSUS port on the local machine, where nothing answers, so any scan that does start fails fast instead of reaching Microsoft. `DoNotConnectToWindowsUpdateInternetLocations = 1` stops the client periodically contacting the public Windows Update service, which Microsoft says it otherwise does "even when Windows Update is configured to receive updates from an intranet update service". Microsoft notes this policy "applies only when the device is configured to connect to an intranet update service", which the redirect provides, and that the redirect itself "has no effect" if Configure Automatic Updates is disabled.

**The services.** `UsoSvc` (Microsoft: "Manages Windows Updates. If stopped, your devices will not be able to download and install latest updates") and `wuauserv` ("If you disable this service, users of this computer will not be able to use Windows Update or its automatic updating feature, and programs will not be able to use the Windows Update Agent (WUA) API") are set to Disabled last, once nothing is left to restart them. Start types apply at once; a service already running stays running until it stops or the machine restarts, which is why the tweak requires a reboot.

**What is left alone, and why.** BITS, Delivery Optimization (`DoSvc`) and the Microsoft Store `\Microsoft\Windows\InstallService` tasks (ScanForUpdates, SmartRetry, WakeUpAndContinueUpdates and similar) are not touched, so that Store app installs and other BITS users keep their own transport. The effect of the WSUS redirect and `wuauserv` being disabled on the Store is covered under Drawbacks.

**Why TrustedInstaller.** Windows protects these tasks: on stock 24H2 the UpdateOrchestrator task files grant Administrators read and execute only (`(A;;0x1200a9;;;BA)`), and the WaaSMedic tasks carry a protected DACL, so an elevated administrator cannot disable them; the SCM descriptor on `WaaSMedicSvc` grants Administrators configuration rights on some editions but not others. A level that is too low fails the apply and rolls it back, so all 20 tasks and `WaaSMedicSvc` run at TrustedInstaller (`ti`) and the remaining effects at admin. Because the 21 `ti` effects are consecutive, they share one TrustedInstaller child process. Optional tasks absent on the running build are detected before anything is spawned and treated as already disabled (a verified no-op), so the same tweak covers Windows 10 and 11.

**Edition.** The services and tasks layers do not depend on Group Policy and work on every edition, including Home. The policy layers (`SetDisableUXWUAccess`, the notification level, the intranet server) are documented in Policy CSP for Pro, Enterprise, Education and IoT Enterprise only, so on Home the tweak relies on its services and tasks.

#### Benefits
- Actually holds: the Medic service and its tasks, the component that undoes naive blockers, are disabled first.
- Covers every route the maintainer verified on a live 26100 machine: scan tasks, notification prompt, Settings UI, network path and services.
- The Store, BITS and Delivery Optimization keep their own tasks and services.
- Fully reversible from the snapshot, to the machine's exact prior state.
- Tasks that do not exist on a build are skipped, not failed.

#### Drawbacks
- **No security patches at all while it is applied.** That is the purpose of the tweak and a real, ongoing exposure.
- Needs a restart to fully take hold, and another after reverting.
- The Windows Update page shows errors by design, because the redirect endpoint does not answer.
- **Microsoft Store may be affected.** Microsoft documents that `DoNotConnectToWindowsUpdateInternetLocations` "may cause connection to public services such as the Microsoft Store, Windows Update client policies, and Delivery Optimization to stop working", and community troubleshooting guides tie Store error 0x80070422 ("The service cannot be started, either because it is disabled") to a disabled Windows Update service. The tweak's claim that the Store keeps working was not part of the recorded tests.
- Defender definition updates lose the Windows Update route; Defender's own update channel is expected to keep working (not tested here). If you rely on Windows Update for definitions, use "Notify only" on [Windows Update mode](#windows-update-mode) instead.
- Programs that use the Windows Update Agent API stop working while `wuauserv` is disabled.
- Needs the TrustedInstaller service available; if it is disabled or missing, the tweak shows "Not available on this PC".

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 and Windows 10, all editions; the policy layers only on Pro and above. Tasks absent on a build (for example `sih`/`sihboot` on 24H2, present on 19045) are no-ops.
- **Takes effect**: start types and task states immediately for new starts; fully after a restart, because a running Medic, USO or Update service keeps running until then.
- **Reverting**: System Default restores all 30 effects to the snapshot. A reboot is advised after reverting too: a re-enabled task with a WNF (Windows Notification Facility) trigger may not re-arm until its service restarts or Windows reboots. On 26100 the task `StartOobeAppsScanAfterUpdate` returns `ERROR_NOT_FOUND` (0x80070490) from `SetEnabled` although the change lands; the app re-reads the task and accepts the result.

#### Interactions
- **Supersedes the other Windows Update tweaks while applied**: deferrals, pins, content control, driver exclusion, metered policy and active hours have nothing to act on. They keep their values and take effect again when the block is reverted, so applying [Defer feature updates](#defer-feature-updates) and [Defer quality updates](#defer-quality-updates) beforehand controls what happens when the block is lifted.
- [Windows Update mode](#windows-update-mode) on "Off by policy" disables Configure Automatic Updates, which Microsoft says makes the intranet-server redirect ineffective; the services and tasks layers still block. Pick "Notify only" there if you want both.
- Owns `UseWUServer` in the same AU key as [Windows Update mode](#windows-update-mode) and [Block auto-restart while signed in](#block-auto-restart-while-signed-in); different values, no ownership conflict.
- A machine genuinely managed by WSUS or Configuration Manager already has `WUServer`/`UseWUServer` set; applying this overwrites them (the snapshot restores them). Do not apply it on a managed machine.

#### Validation
- **Verdict**: VERIFIED (September 2026 research, this page), empirically tested. No mechanism correction. Two open questions, below.
- **Confidence**: empirically tested (real-machine apply, watch and restore on 26100.4061), community-corroborated (Medic self-healing, task inventory, `usoclient`/`MusNotification` roles), Microsoft-documented for every policy value and the service descriptions.
- **Reasoning**: every registry value is Microsoft's own (Policy CSP and "Manage additional Windows Update settings" give `SetDisableUXWUAccess`, `SetUpdateNotificationLevel`/`UpdateNotificationLevel`, `UseWUServer`, `WUServer` and `WUStatusServer` under exactly these keys; `DoNotConnectToWindowsUpdateInternetLocations` is named by the shipped 26100 `WindowsUpdate.admx`). The Medic self-healing premise is corroborated by several independent community sources and by user reports on Microsoft Q&A; the absence of start triggers was measured with `sc qtriggerinfo`. The ordering, the TrustedInstaller requirement and the whole apply-watch-restore cycle held on a real machine (see Tested). Microsoft not endorsing turning updates off is not treated as a defect: the approach is deliberate. Attacked and survived: whether WaaSMedic undoes the block (no drift in the watch), whether an elevated administrator is enough for the tasks (no; hence `ti`), whether the Task Scheduler calls work under a minimal environment (yes). Open questions: (1) **`upfc.exe`**: a community report on privacy.sexy describes `upfc.exe`, spawned by `services.exe` at startup roughly every five days (gated by `NextHealthCheckTime` under `HKLM\SYSTEM\WaaS\Upfc`), which "uses XML files under Windows\Waas to reset and restart various services (and scheduled tasks) to re-enable Windows Update, Update Orchestrator and the Windows Update Medic Service". The 26100.4061 test machine has `upfc.exe`, that registry key, and service manifests under `C:\Windows\WaaS\services` for `wuauserv`, `UsoSvc` and `WaaSMedicSvc`. The tweak does not address it, and a one-minute watch cannot rule it out; durability across a multi-day window with reboots is untested. (2) The Microsoft Store claim, as described under Drawbacks.
- **Tested**: build validation (schema, ownership and conflict checks). Real-machine manual tests on 2026-09-24, Windows 11 build 26100.4061, app test build, elevated:
  - `ti_batch_timing` **Pass**: "Blocked" applied through the normal apply path in one TrustedInstaller batch (18 operations, about 59 ms of the 30000 ms broker wait). All 30 effects read back as intended; the 4 tasks absent on that build (`uso_retry_scan`, `uso_update_model`, `wu_sih`, `wu_sihboot`) were no-ops. Restore returned all 30 to baseline (status System Default, reboot advisory shown).
  - `waasmedic_watch`: no drift while blocked over a one-minute watch, so WaaSMedic did not undo the block; restore verified.
  - `system_only_environment` **Pass**: the Task Scheduler COM calls work when the TrustedInstaller child runs under a system-only environment.
  - `systemtemp_transport` **Pass**: apply and restore with the broker request and response routed through `%SystemRoot%\SystemTemp`.
  - `waasmedic_task_read` **Pass**: the protected WaaSMedic tasks read correctly.
  - Known quirks observed: `StartOobeAppsScanAfterUpdate` reports `ERROR_NOT_FOUND` (0x80070490) from `SetEnabled` while the change lands; a re-enabled WNF-triggered task may not re-arm until its service restarts or Windows reboots.

#### Recommendation
Only for a machine where you take responsibility for patching some other way: an offline or tightly controlled system, a lab image, a machine you patch by hand on your own schedule. Revert it at least monthly to patch, and check after a week of uptime that the block still holds (see the `upfc.exe` question). Do not apply it on a machine managed by WSUS or an organization. For everyday use prefer "Notify only" on [Windows Update mode](#windows-update-mode), which leaves the pipeline intact and needs no reboot.

#### Sources
1. How Windows Update works: the Update Orchestrator's role in scan, download, install and restart, https://learn.microsoft.com/en-us/windows/deployment/update/how-windows-update-works (tier A, fetched September 2026)
2. Manage additional Windows Update settings: intranet update service location (and that it has no effect when Configure Automatic Updates is disabled), `UseWUServer`, `WUServer`, `WUStatusServer`, "Do not connect to any Windows Update Internet locations" (including the Store and Delivery Optimization caveat), and "Remove access to use all Windows Update features" (background work continues), https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings (tier A, fetched September 2026)
3. Policy CSP, Update, `SetDisableUXWUAccess` and `UpdateNotificationLevel`: registry value names, values, Group Policy mapping and editions (Home not listed), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A, fetched September 2026)
4. Manage device restarts after updates, "Display options for update notifications": the 0/1/2 notification levels, https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A, fetched September 2026)
5. Guidance on disabling system services on Windows Server 2016: `wuauserv` and `UsoSvc` descriptions and startup types, BITS as a Windows Update dependency, https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A, fetched September 2026)
6. Overview of Windows as a service: servicing context cited by the tweak, https://learn.microsoft.com/en-us/windows/deployment/update/waas-overview (tier A, fetched September 2026)
7. What is Windows Update Medic Service (WaaSMedicSVC.exe)?, The Windows Club: Medic re-enables disabled update services, "Access is Denied" when disabling it, and its Service Initiated Healing task, https://www.thewindowsclub.com/windows-update-medic-service (tier C, fetched September 2026)
8. Windows Update and the Medic Services, Microsoft Q&A: user reports of the update service re-enabling itself daily and "Access Denied" on the Medic service, https://learn.microsoft.com/en-us/answers/questions/3fc61b5d-17ca-4a2f-8ac8-c67c7e0f4a56/windows-update-and-the-medic-services?forum=windows-all (tier C, fetched September 2026)
9. Disable "PerformRemediation" task, PrivacyLearn (privacy.sexy): the task path and its description, https://privacylearn.com/windows/privacy-over-security/disable-automatic-updates/disable-windows-update-scheduled-tasks/disable-performremediation-task (tier C, fetched September 2026)
10. Disable Windows Update scheduled tasks, PrivacyLearn (privacy.sexy): community inventory of UpdateOrchestrator, WindowsUpdate, WaaSMedic and InstallService tasks, https://privacylearn.com/windows/privacy-over-security/disable-automatic-updates/disable-windows-update-scheduled-tasks (tier C, fetched September 2026)
11. Disable Windows update services, PrivacyLearn (privacy.sexy): `wuauserv`, `UsoSvc`, `WaaSMedicSvc` and the `upfc.exe` re-enablement process, https://privacylearn.com/windows/privacy-over-security/disable-automatic-updates/disable-windows-update-services (tier C, fetched September 2026)
12. Enhance "disable Windows Update" mechanism by configuring upfc.exe, privacy.sexy issue #294: what `upfc.exe` resets, its cadence and `HKLM\SYSTEM\WaaS\Upfc`, https://github.com/undergroundwires/privacy.sexy/issues/294 (tier C, fetched September 2026)
13. Error 0x80070422 in Windows Update and Store, Winhelponline: the error means a required service is disabled, and "The same error occurs when opening Microsoft Store", https://www.winhelponline.com/blog/fix-windows-update-error-0x80070422-in-windows-10/ (tier C, fetched September 2026)
14. Local inspection of the 26100.4061 test machine: `sc qtriggerinfo WaaSMedicSvc` (no triggers), `WaaSMedicSvc` and `wuauserv` demand-start, `upfc.exe`, `HKLM\SYSTEM\WaaS\Upfc` and the `C:\Windows\WaaS\services` manifests present (primary measurement; the owner-modified machine is evidence about that machine, not a Windows default)
15. Real-machine manual tests on 2026-09-24 listed under Tested (primary, empirical)
16. Shipped `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` on build 26100: `DoNotConnectToWindowsUpdateInternetLocations`, `SetUpdateNotificationLevel`/`UpdateNotificationLevel` value names (tier A, primary)

## Considered and not shipped

### Notify before downloading updates

`disable_auto_update_download` wrote `NoAutoUpdate = 0` and `AUOptions = 2` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU`, the Configure Automatic Updates policy's "notify for download" option. The July 2026 research rated it VERIFIED (high risk, Microsoft-documented, Home ignores it) with no mechanism correction, noting that Microsoft words `AUOptions = 2` three different ways and that it makes the auto-restart block inert. It is not shipped because [Windows Update mode](#windows-update-mode) replaces it: the same two values, widened from one mode to four (automatic, download only, notify only, off by policy), with its "Notify only" option writing exactly what this tweak wrote. The two own the same addresses, so they cannot coexist.

Sources: Manage additional Windows Update settings, https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings (tier A); Manage device restarts after updates, https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A); Policy CSP, Update, `AllowAutoUpdate`, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A).
