# Network, Update & Power tweak validation

Rebuilt 2026-07-27. Source corpus: `src-tauri/tweaks/network.yaml` (20 tweaks) plus 7 verified
additions carried in from the gap-hunt verification passes, for **27 tweaks total**.

Primary target platform: **Windows 11 24H2 (build 26100) and newer, including 25H2**, x64.
Windows 10 IoT Enterprise LTSC 2021 (build 19044) is a low-priority secondary target and is called
out per tweak only where the applicability genuinely differs.

This document consolidates: the first validation pass, two adversarial re-verification rounds over
it, the policy-hive audit, the harmful-revert audit, the cross-category audit, and the adversarial
verification of the gap proposals in `_verify-gaps-b-high.md` and `_verify-gaps-b-medlow.md`.

Some facts are marked **(measured)**. Those were confirmed by direct query on a live Windows 11
IoT Enterprise LTSC 2024 machine (24H2, build 26100.4061) using `powercfg`, registry reads, and
UTF-16 string extraction from shipped binaries. Per `_harmful-revert.md`, that machine is modified
by its owner, so a measurement is evidence about **that machine**, never proof of a Windows default.
Where a measurement is the only support for a claimed default, it is labelled as such. Facts drawn
from `C:\Windows\PolicyDefinitions` ADMX and ADML files describe the product rather than the machine
and are treated as tier A.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `dns_over_https` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Key, name, type and value 2 are Microsoft's own, and `dnsrslvr.dll` on 26100 still reads the value, but `EnableAutoDoh = 2` is only an auto-upgrade switch: it encrypts nothing unless the configured resolver is already on Windows' known-DoH template list, and this tweak sets no resolver. Microsoft also said registry DoH configuration would not be supported in release builds. |
| `disable_llmnr` | VERIFIED | medium | Microsoft-documented | none |
| `disable_netbios_tcpip` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | `undo` hardcodes 0 for every interface instead of restoring the per-interface prior value; adapters created after apply are not covered |
| `disable_wpad` | VERIFIED | medium | Microsoft-documented | none on mechanism; two Microsoft caveats missing from the copy (DNS-level WPAD still resolves; the Settings UI toggle must also be turned off for third-party apps) |
| `disable_ipv6_transition` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Copy calls it "safe and recommended"; Microsoft warns any `DisabledComponents` value other than 0 or 32 breaks Routing and Remote Access. Teredo and ISATAP are already off by default, so the real delta is 6to4. |
| `disable_nic_power_management` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | `PnPCapabilities = 24` is Microsoft's documented deployment value (KB 2740020), so the value is settled. Every remaining defect is in the action: `undo` deletes rather than restores, the undo filter is wider than the apply filter, apply writes to virtual and miniport adapters, and the probe false-positives. |
| `disable_delivery_optimization_p2p` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Mechanism confirmed against the shipped 26100 ADMX. The copy claims it stops uploads "across the internet"; internet peering is mode 3 and is not the default under either of Microsoft's two contradictory statements of the default. Also drop the unsourced `0x80d03002` code. |
| `defer_quality_updates` | VERIFIED | medium | Microsoft-documented | none on mechanism; one unsourced claim in the copy about a missing 24H2 Group Policy UI |
| `target_release_version` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | The only offered pin is 24H2, whose Home and Pro servicing ends 2026-10-13; 25H2 and 26H1 have shipped. Also now mutually exclusive with the new `defer_feature_updates`. |
| `set_active_hours` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | "Stock Default" writes fabricated values (`SmartActiveHoursState = 1`, 8, 17). The real stock state is value-absent for `SmartActiveHoursState` and whatever hours the user or OS already set. Writes a Windows-internal settings key, not a policy key. |
| `disable_auto_restart_logged_on` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Missing companion values. Microsoft states the policy applies only when Configure Automatic Updates is set to option 4. |
| `exclude_wu_driver_updates` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Copy claims it works on Windows Home. It is a Windows Update client policy: Pro, Education, Enterprise, IoT Enterprise only. |
| `disable_auto_driver_install` | VERIFIED-WITH-CORRECTION | medium | Community-corroborated | Both values are written to the non-policy UI keys. The documented policy keys are under `SOFTWARE\Policies\...`. `SearchOrderConfig = 0` semantics on the non-policy path have no tier A or B source. |
| `disable_store_auto_updates` | VERIFIED | medium | Microsoft-documented | none |
| `block_update_over_metered` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Key, value, type and default confirmed by Policy CSP and the shipped ADMX. The documented default is already 0, so the tweak enforces and locks the existing default in the policy branch rather than changing behaviour; the copy must say that accurately. |
| `disable_auto_update_download` | VERIFIED | high | Microsoft-documented | none on mechanism; Microsoft's own pages word `AUOptions = 2` three different ways. High risk, and only reliable on Pro, Education and Enterprise; Home ignores the Windows Update AU policy entirely. |
| `disable_hibernation` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | `undo` hardcodes `powercfg /hibernate on` and will enable hibernation on machines where it was already off; prior `/size` and `/type reduced` configuration is lost |
| `disable_usb_selective_suspend` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | `undo` hardcodes 1/1 rather than restoring; only the active scheme is touched; probe checks AC only |
| `disable_wake_timers` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `undo` writes 1 to both AC and DC, which is **more permissive than the factory default on both power sources** (measured stock Balanced is AC = 2, DC = 0). Apply then revert leaves a machine that wakes for scheduled tasks on battery, which it never did before. |
| `disable_modern_standby` | VERIFIED | high | Community-corroborated | `PlatformAoAcOverride` confirmed inside `ntoskrnl.exe` on 26100.4061 and corroborated by four independent tier C sources; make the `powercfg /a` S3 precondition enforceable rather than advisory |
| `disable_mdns` (new) | VERIFIED-WITH-CORRECTION | medium (proposed) | Microsoft-documented | The `>=26100` build gate is unjustified: the shipped ADMX declares "At least Windows 10". The `Dnscache\Parameters` value must be written too, not treated as optional, because the ADML defers the disabled state to "locally configured settings". |
| `update_feature_control` (new) | VERIFIED-WITH-CORRECTION | low (proposed) | Microsoft-documented | `SetAllowOptionalContent` is only a 1/0 enable flag; the selection lives in a separate `AllowOptionalContent` value where 1 is **more** permissive than 2. Two different applicability gates, not one. |
| `defer_feature_updates` (new) | VERIFIED | low (proposed) | Microsoft-documented | Mechanism exact. Microsoft states feature-update deferrals are fully inert when a target release version is pinned, so this and `target_release_version` are mutually exclusive, not merely cross-referenced. |
| `block_insider_builds_policy` (new) | VERIFIED-WITH-CORRECTION | low (proposed) | Microsoft-documented | Both proposed value names and both proposed values were wrong. Group Policy writes one value, `ManagePreviewBuildsPolicyValue`, and the blocking value is 1, not 0. Needs a Pro-and-above edition gate. |
| `disable_internet_connection_sharing` (new) | VERIFIED | medium (proposed) | Microsoft-documented | Mechanism and inverted polarity confirmed. The proposal's Mobile Hotspot service dependency does not exist in the SCM graph, and the breakage that actually bites (Hyper-V Default Switch, WSL2, Windows Sandbox networking) was missing. Stock start type unconfirmed. |
| `require_doh` (new) | VERIFIED-WITH-CORRECTION | medium (proposed) | Microsoft-documented | Two of the three `DoHPolicy` enum values were transposed in the proposal: 1 prohibits, 2 allows, 3 requires. Three sibling elements are `required="true"` in the ADMX. |
| `firewall_logging_and_merge` (new) | VERIFIED-WITH-CORRECTION | medium (proposed) | Microsoft-documented | `LogFilePath` and `LogFileSize` are `required="true"` alongside `LogDroppedPackets`; writing one alone leaves the policy half-configured. Do not copy the ADMX `disabledList` string form. `AllowLocalPolicyMerge` failure mode must be stated bluntly. |

**Totals: 27 tweaks. VERIFIED 8, VERIFIED-WITH-CORRECTION 19, UNVERIFIED 0, DISPUTED 0,
INCORRECT 0.**

The 8 VERIFIED: `disable_llmnr`, `disable_wpad`, `defer_quality_updates`,
`disable_store_auto_updates`, `disable_auto_update_download`, `disable_modern_standby`,
`defer_feature_updates`, `disable_internet_connection_sharing`. The other 19 are
VERIFIED-WITH-CORRECTION.

**How the two verdicts are separated here.** `VERIFIED-WITH-CORRECTION` is reserved for a concrete
mechanism defect: wrong key, wrong value name, wrong type, wrong value, wrong stock default, wrong
applicability gate, missing companion value, inverted polarity, or an `undo` that does not restore.
A correction that is purely to the user-facing copy or to how two tweaks interact leaves the verdict
at `VERIFIED`, with the correction still listed and still required. Four of the eight VERIFIED
entries carry copy corrections on that basis: `disable_wpad`, `defer_quality_updates`,
`defer_feature_updates` and `disable_internet_connection_sharing`.

No tweak in this category is UNVERIFIED, DISPUTED or INCORRECT. Two carry `Community-corroborated`
confidence rather than Microsoft-documented (`set_active_hours` for `SmartActiveHoursState`,
`disable_modern_standby` for `PlatformAoAcOverride`), and one is mixed
(`disable_auto_driver_install`: the metadata value is Microsoft-documented at a *different* key than
the one written, while `SearchOrderConfig` on the non-policy path is not documented at all).

## Corrections required

Every concrete defect found, naming the tweak, the exact wrong thing, and the exact right thing.

### Corrections for `dns_over_https`

1. **The mechanism is real but unsupported, and the copy does not say so.** `EnableAutoDoh` under
   `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters` is correctly named and typed: the
   May 2020 Windows networking blog gives the key, the DWORD name and the value 2 verbatim, and the
   string is still present in `dnsrslvr.dll` on 26100.4061, so the DNS Client still reads it. The
   same blog says the registry keys "are only for enabling DoH client testing on Insider builds" and
   that "when the DoH client is made available in general release builds, registry configuration of
   DoH will not be supported". Microsoft's shipping documentation for the DoH client (the Settings
   UI, the "Configure DNS over HTTPS (DoH) name resolution" Group Policy, `netsh dns add
   encryption`, `Add-DnsClientDohServerAddress`, and the per-interface
   `Dnscache\InterfaceSpecificParameters\{GUID}\DohInterfaceSettings\Doh\{IP}` `DohFlags` value)
   never mentions `EnableAutoDoh` again. Mark it working-but-unsupported, or move to the documented
   route.
2. **The tweak on its own cannot enable DoH.** Auto-upgrade fires only when the interface's
   configured resolvers are already on Windows' known-DoH template list (Cloudflare 1.1.1.1 and
   1.0.0.1, Google 8.8.8.8 and 8.8.4.4, Quad9 9.9.9.9 and 149.112.112.112, plus anything added with
   `Add-DnsClientDohServerAddress`). This tweak sets no resolver, so on an ordinary ISP-assigned
   resolver it encrypts nothing while still reporting success. For a security-relevant tweak,
   silently doing nothing while reporting success is the worst outcome. Either also set a DoH-capable
   resolver, or state the precondition in the first two lines of the copy.

### Corrections for `disable_netbios_tcpip`

3. **`undo` is not a restore.** It writes `NetbiosOptions = 0` to every `Tcpip_*` subkey. 0 is the
   documented default ("use the DHCP server's setting") and is what all seven interfaces carried on
   the test machine **(measured)**, so it is usually harmless. It still destroys an interface an
   admin or unattend file deliberately set to 1. Snapshot and restore the prior per-interface value.
4. **Adapters created after apply are not covered.** A new USB NIC, VPN adapter or Hyper-V virtual
   switch gets a fresh interface subkey carrying the default. The probe correctly fails in that case,
   so it surfaces as drift rather than silently, but the copy should say the setting is per-adapter
   and does not follow new hardware.

### Corrections for `disable_wpad`

5. **Two Microsoft caveats are missing from the copy.** First: "Even with this registry key set,
   applications can still resolve the name 'WPAD' by calling Domain Name System (DNS) directly", so
   `nslookup wpad` still resolves and DNS-level WPAD queries continue. Second: "In addition to
   setting the registry key, WPAD should also be disabled in the Windows Settings UI, because
   third-party apps and Internet browsers may rely on these settings for Proxy Auto-Discovery."
   Neither is a mechanism defect, but omitting them lets a user overestimate the coverage.

### Corrections for `disable_ipv6_transition`

6. **The RRAS warning is missing and the framing contradicts a tier A source.** Microsoft's IPv6
   configuration guidance carries the note "Values other than 0 or 32 causes the Routing and Remote
   Access service to fail after this change takes effect." `0x01` is such a value. The copy's "Safe
   and recommended for most users" must be qualified.
7. **The claimed delta is overstated.** The same page states "ISATAP and Teredo are disabled by
   default in Windows." The incremental effect of `0x01` on a stock machine is therefore limited to
   6to4 and the generic tunnel interface. Do not promise removal of Teredo and ISATAP as though they
   were running.

### Corrections for `disable_nic_power_management`

8. **The `undo` is destructive.** It runs `Remove-ItemProperty ... -Name PnPCapabilities` on every
   four-digit class subkey. On the test machine the MediaTek Wi-Fi adapter already carried
   `PnPCapabilities = 16` before any tweak was applied **(measured)**. The undo deletes that
   pre-existing vendor or OEM value rather than restoring it. It must snapshot each key's prior state
   (including "absent") and write that back.
9. **The undo filter is wider than the apply filter.** `undo` matches `$_.PSChildName -match
   '^\d{4}$'` with no `DriverDesc` guard, while `apply` requires `DriverDesc`. The undo therefore
   touches a strictly larger key set than the apply wrote.
10. **The apply is not scoped to physical adapters.** On the test machine it writes 24 into the
    driver keys of the Kernel Debug Network Adapter, the Hyper-V Virtual Switch Extension Adapter,
    both Wi-Fi Direct virtual adapters, nine WAN Miniports and the Bluetooth PAN device
    **(measured)**, none of which expose a Power Management tab. Restrict the write, or replace the
    whole action with `Set-NetAdapterPowerManagement`, which Microsoft documents.
11. **The probe false-positives.** It returns "applied" if *any* class subkey holds 24. An adapter
    whose vendor INF already set 24 makes the probe report applied when the tweak never ran.
12. **The `warning` text is now wrong in the user's favour and should be rewritten.** It calls
    `PnPCapabilities = 24` "community-sourced". Microsoft's KB 2740020 successor article names the
    value, gives the default of 0, states "A value of 24 will prevent Windows from turning off the
    network adapter or let the network adapter wake the computer from standby", and says "For
    deployment purpose, to keep option 1 cleared, one needs to use the value 24 (0x18)." The value is
    Microsoft's own documented deployment value. Say that, and move the warning onto the blanket
    write and the delete-based undo, which are the parts that are actually risky.

### Corrections for `disable_delivery_optimization_p2p`

13. **The copy promises something the stock default never did.** It says the tweak stops the PC
    "uploading update chunks to other machines on your network **and across the internet**". Uploading
    across the internet is download mode 3, and mode 3 is not the stock default under either of
    Microsoft's two contradictory statements. The Delivery Optimization reference page says "Default
    is configured to LAN(1)", which is "HTTP blended with peering **behind the same NAT**", so peering
    never leaves the local subnet. The Policy CSP page for `DODownloadMode` says the opposite,
    "Default Value: 0" with "0 (Default) HTTP only, no peering", under which there is no peering at
    all. Both pages are tier A and current, so the live default is genuinely unresolved from
    documentation. State the benefit as removing same-subnet LAN peering and pinning the mode so
    peering cannot be enabled later, and record the unresolved default rather than asserting either
    reading. This is not an argument against the tweak: the control is real and pinning it is the
    point.
14. **Drop or mark anecdotal the error code `0x80d03002`.** It appears in the copy as the consequence
    of download mode 100 and is not in Microsoft's documentation. Microsoft's own wording is
    sufficient: "Starting in Windows 11, this option is deprecated. Don't configure Download Mode to
    '100' (Bypass), which can cause some content to fail to download."

### Corrections for `target_release_version`

15. **The offered pin is nearly expired.** The only non-default option pins to `24H2`. Windows 11
    24H2 Home and Pro reach end of updates on 2026-10-13; 25H2 shipped 2025-09-30 and 26H1 shipped
    2026-02-10. Offering 24H2 as the pin today parks a user roughly eleven weeks from falling off
    security updates, which is the exact failure mode the tweak's own text warns about. The options
    list must track currently supported releases and the UI should surface the end-of-servicing date
    for whichever version is pinned.
16. **Licensing intent.** Microsoft's `ProductVersion` documentation states that using this policy to
    move a device to a new product carries a licensing attestation (volume licence, or authority to
    accept the licence terms on an organization's behalf). Setting `ProductVersion = "Windows 11"` on
    a consumer machine is outside the documented intent and should be noted.
17. **Newly mutually exclusive with `defer_feature_updates`.** Microsoft: "When you specify target
    version policy, feature update deferrals won't be in effect." If both tweaks ship, the UI must
    make them exclusive rather than merely cross-referenced, or a user will set a deferral that is
    documented to be inert.

### Corrections for `set_active_hours`

18. **The "Stock Default" option fabricates a state the machine never had.** It writes
    `SmartActiveHoursState = 1`, `ActiveHoursStart = 8`, `ActiveHoursEnd = 17`. On the test machine
    `SmartActiveHoursState` did not exist at all and the active hours carried user-set values of 16
    and 10 **(measured)**. Reverting therefore invents a value and destroys the user's real hours.
    The Stock Default must be `absent` for `SmartActiveHoursState`, and the hours must be restored
    from the snapshot.
19. **It writes a Windows-internal settings key, not a policy key.**
    `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` is the store the Settings app writes when a
    user sets active hours by hand. The supported route is `ActiveHoursStart` and `ActiveHoursEnd`
    under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` (ADMX policy "Turn off auto-restart
    for updates during active hours"). Writing the UX key works, because it is the same store the UI
    reads, but it is user-overwritable, unversioned, and silently overridden by the policy key.
20. **`SmartActiveHoursState` semantics are tier C only.** 1 meaning on and 2 meaning off rests
    entirely on Winaero; other community sources give 0 and 1 instead. Microsoft documents neither the
    value nor its range. Flag it as community-sourced in the tweak text.

### Corrections for `disable_auto_restart_logged_on`

21. **Missing companion values make it documented-inert on an unconfigured machine.** Microsoft:
    "This policy only applies when Configure Automatic Updates is set to option 4 - Auto download and
    schedule the install." Writing `NoAutoRebootWithLoggedOnUsers = 1` with no `NoAutoUpdate` or
    `AUOptions` companion does nothing on a machine that has not separately configured automatic
    updates. Add the companions, or merge into the automatic-updates tweak (merge candidate 1).
22. **Microsoft's own caveat is missing.** The same page: "In Group Policy this policy doesn't work
    exactly as per description. This policy can result in no quality update reboots period, given
    many users never log off." Microsoft recommends compliance deadlines instead. Also: the policy
    "was never created as a CSP", and over RDP only active sessions count as signed-in users, so a
    machine with only disconnected sessions restarts anyway.

### Corrections for `exclude_wu_driver_updates`

23. **The Home claim is wrong.** The copy says "This works on Windows Home directly via the registry
    key, with no Group Policy editor needed." Windows Update client policies are documented as
    available on Pro (including Pro for Workstations), Education, and Enterprise (including
    Enterprise LTSC, IoT Enterprise and IoT Enterprise LTSC). Home is not in the list, and the Policy
    CSP entry for `ExcludeWUDriversInQualityUpdate` shows Pro, Enterprise, Education and IoT
    Enterprise only. Replace with a Pro-and-above gate matching the other Windows Update policies in
    this file.

### Corrections for `disable_auto_driver_install`

24. **`PreventDeviceMetadataFromNetwork` is written to the UI store, not the policy key.** Microsoft
    documents it at `HKLM\SOFTWARE\Policies\Microsoft\Windows\Device Metadata`. The YAML writes
    `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Device Metadata`, which is the Control Panel
    "Device Installation Settings" store. The Policy CSP states the precedence outright: "This policy
    setting overrides the setting in the Device Installation Settings dialog box."
25. **`SearchOrderConfig` is likewise written to the non-policy path.** The ADMX policy "Specify
    search order for device driver source locations" writes `SearchOrderConfig` under
    `HKLM\SOFTWARE\Policies\Microsoft\Windows\DriverSearching`. The YAML writes
    `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching`.
26. **`SearchOrderConfig = 0` on the non-policy path has no tier A or B definition.** Community
    sources actively disagree: one set says 0 blocks Windows Update drivers, another says only 1, 2
    and 3 are meaningful there. Mark the semantics unverified until a tier A or B source is found.
    The Stock Default of `1` is correct: `SearchOrderConfig = 1` was present out of the box
    **(measured)**.

### Corrections for `block_update_over_metered`

27. **The copy describes a behaviour change that does not occur, and must be reframed without calling
    the tweak pointless.** Microsoft documents the default for
    `AllowAutoWindowsUpdateDownloadOverMeteredNetwork` as "0 (Default) Not allowed", and the shipped
    `WindowsUpdate.admx` on 26100.4061 confirms the key, the value name and the 1/0 mapping. Writing
    0 therefore does not change whether a stock machine downloads over a metered link. What it does
    add is real: the decision moves into the policy branch, where it takes precedence over the
    per-machine "Download updates over metered connections" toggle in Settings, so a later user, tool
    or image change cannot opt the machine in. Reframe the copy as enforcing and locking the existing
    default, say plainly that a stock machine already does not auto-download over metered, note that
    priority and security updates can still come through, and state that Home does not honour it.

### Corrections for `disable_hibernation`

28. **`undo` runs `powercfg /hibernate on` unconditionally.** On the test machine `HibernateEnabled`
    was already 0 before any tweak **(measured)**, so reverting would turn hibernation *on*, recreate
    a multi-gigabyte `hiberfil.sys`, and re-enable Fast Startup on a machine that had none of those.
    The undo must be conditional on the snapshotted prior state.
29. **`powercfg /hibernate on` restores hibernation at full size.** A machine previously configured
    with `powercfg /hibernate /size 0` or `/type reduced` (Fast Startup only, no full hibernate)
    loses that distinction. Snapshot the hibernate file type and size as well.

### Corrections for `disable_usb_selective_suspend`

30. **`undo` hardcodes index 1 for AC and DC.** 1 is the stock Balanced default for both
    **(measured)**, so this is usually right, but it is not a restore: a user who had already
    disabled selective suspend, or an OEM scheme with a different value, gets overwritten.

### Corrections for `disable_usb_selective_suspend` and `disable_wake_timers` (shared)

31. **Both operate on `SCHEME_CURRENT`.** If the user switches power plan between apply and undo, the
    undo writes to a different scheme and the applied scheme keeps the tweak. Snapshot the scheme
    GUID at apply time, or apply to all schemes.
32. **Both probes test only `Current AC Power Setting Index`.** A machine with AC applied and DC
    reverted reports "applied". Check both rails.

### Corrections for `disable_wake_timers`

33. **The `undo` leaves the machine more wake-prone than it has ever been. This is the most
    consequential revert defect in the file.** On stock `SCHEME_BALANCED` under Windows 11 24H2 the
    "Allow wake timers" setting is AC = 2 ("Important Wake Timers Only") and DC = 0 ("Disable")
    **(measured)**. The `undo` writes 1 ("Enable") to both, which is more permissive than the factory
    default on **both** power sources. A user who applies and then reverts ends up with a machine
    that wakes for ordinary scheduled tasks on battery, which it never did before, and the interface
    tells them nothing. The undo must restore the snapshotted indices, or at minimum AC = 2 and
    DC = 0 on Windows 11.
34. **The premise overstates the stock behaviour.** Because DC is already 0 by default, the apply's
    DC half changes nothing on a stock machine; the only real change is AC from 2 to 0. And because
    AC defaults to "Important Wake Timers Only" rather than "Enable", Windows already excludes
    ordinary scheduled tasks from waking the machine on mains power. Say what the tweak actually
    tightens.

### Corrections for `disable_modern_standby`

35. **The `powercfg /a` precondition must be enforceable, not advisory.** `PlatformAoAcOverride` is
    community-corroborated and confirmed present in the 26100 kernel, but it does not create an S3
    path where the firmware has none. If `powercfg /a` reports "Standby (S3): The system firmware
    does not support this standby state", applying leaves the machine with no S0-class and no
    S3-class sleep. The app should read `powercfg /a` and refuse or hard-warn, because a text warning
    is the wrong protection for an outcome that can leave a laptop unable to sleep. Also add
    Microsoft's own statement that "Switching the power model is not supported in Windows without a
    complete OS re-install", so the tweak is presented as working-but-unsupported.

### Corrections for `disable_mdns` (new)

36. **Drop the `>=26100` build gate.** The proposal specified `windows: { build: ">=26100" }` and said
    "do not claim it for LTSC 2021". The shipped 26100 `DnsClient.admx` declares
    `supportedOn ref="windows:SUPPORTED_Windows_10_0_RS2"`, that is "At least Windows 10". LTSC 2021
    is build 19044, well above RS2. There is no tier A evidence for a 26100 floor. Ship it ungated
    and say in the copy that it was verified on 26100 and is expected but not verified on 19044.
37. **Write both values, not one.** The proposal treated
    `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\EnableMDNS = 0` as an optional second
    effect. The shipped ADML for `DNS_MDNS` reads: "If you disable this policy setting, **or if you do
    not configure this policy setting, the DNS client will use locally configured settings.**" Read
    literally, the policy's disabled value returns the client to local settings rather than
    unconditionally off, and `dnsrslvr.dll` carries the `Dnscache\Parameters` base path alongside the
    policy path, so both are read. The service-key value is the "locally configured setting" the ADML
    defers to. Write both, revert both.

### Corrections for `update_feature_control` (new)

38. **`SetAllowOptionalContent` is only an enable flag.** The proposal's table treated it as the
    selector ("0 no optional content, 1 automatically receive optional updates, 2 also get the latest
    optional non-security preview"). The shipped 26100 `WindowsUpdate.admx` shows the GP policy
    `AllowOptionalContent` writes `SetAllowOptionalContent` as `enabledValue` 1 / `disabledValue` 0,
    and puts the selection in a **separate** `AllowOptionalContent` value.
39. **The 1 and 2 ordering is effectively inverted.** Policy CSP: `AllowOptionalContent` 0 (Default)
    don't receive optional updates, **1 automatically receive optional updates including gradual
    feature rollouts (CFRs)**, 2 automatically receive optional updates (optional cumulative updates
    only), 3 users can select which optional updates to receive. So 1 is **more** permissive than 2,
    not less. For the stability option write `SetAllowOptionalContent = 0` and leave
    `AllowOptionalContent` absent; for revert delete all three values.
40. **Two applicability gates, not one.** `AllowTemporaryEnterpriseFeatureControl` is
    `SUPPORTED_Windows_11_0_22H2` (Windows 11 22H2 and later, so **not** LTSC 2021).
    `SetAllowOptionalContent` and `AllowOptionalContent` are
    `WU_SUPPORTED_WinServer2025_Win1021H2_Win1122H2`, rendered as "At least Windows Server 2025,
    Windows 10 Version 21H2, or Windows 11 Version 22H2", and Policy CSP gives Windows 10 21H2
    (10.0.19044.3757) and later, so a patched LTSC 2021 qualifies for that half only. A single gate
    would either exclude a working control or promise one that is absent.

### Corrections for `defer_feature_updates` (new)

41. **Mutual exclusion with `target_release_version` is a hard interaction, not a cross-reference.**
    The proposal asked only that the two tweaks "reference each other in their info copy". Microsoft
    is blunter: "When you specify target version policy, feature update deferrals won't be in
    effect." If the pin is applied, this tweak is **fully inert**. Treat them as mutually exclusive
    in the UI. Two further mechanical notes: the same ADMX policy owns `PauseFeatureUpdatesStartTime`
    (`REG_SZ`), which the tweak must not write and whose absence the revert must not assume, because
    the user may have paused updates from the Settings UI; and `BranchReadinessLevel` is **not** part
    of this policy on 26100 (it belongs to `ManagePreviewBuilds`), so a naive port of older guidance
    would wrongly add it.

### Corrections for `block_insider_builds_policy` (new)

42. **Both proposed value names and both proposed values were wrong.** The proposal asked for
    `ManagePreviewBuilds = 1` and `ManagePreviewBuildsPolicyValue = 0`. The shipped 26100
    `WindowsUpdate.admx` defines one policy here, *named* `ManagePreviewBuilds`, which writes exactly
    one flag value, `ManagePreviewBuildsPolicyValue`, with `enabledValue` 2 and `disabledValue` 1.
    There is no `ManagePreviewBuilds` DWORD written by Group Policy, and 0 is not a value the ADMX
    ever writes. The blocking state is the policy's **Disabled** state, which writes **1**. The
    companion element is `BranchReadinessLevel` (Dev 2, Beta 4, Release Preview 8, Release Preview
    quality-only 64), not a second preview-builds value.
43. **An edition gate is missing.** Policy CSP `Update/ManagePreviewBuilds` lists Pro, Enterprise,
    Education, IoT Enterprise and IoT Enterprise LTSC. **Not Home.** The proposal stated no gate.
    Unresolved and deliberately not shipped: the CSP documents the *setting* on a 0 to 3 scale that
    does not match what the ADMX writes into `ManagePreviewBuildsPolicyValue`. Whether the update
    stack also reads a literal `ManagePreviewBuilds` value when written by MDM was not established.
    Ship the ADMX-backed value only.

### Corrections for `disable_internet_connection_sharing` (new)

44. **The stated risk is wrong and the real one is missing.** The proposal said `SharedAccess` backs
    Mobile Hotspot and that disabling it removes the feature. The SCM dependency graph on 26100 does
    not support that: `icssvc` ("Windows Mobile Hotspot Service") depends on `RpcSs` and `wcmsvc`,
    not on `SharedAccess`. Mobile Hotspot may still fail at runtime because it uses the same NAT
    engine, but a service dependency that does not exist must not be repeated as fact. Meanwhile
    `SharedAccess` / `ipnathlp.dll` is the NAT engine behind the **Hyper-V Default Switch**, and
    therefore behind **WSL2 networking** and Windows Sandbox networking. Disabling ICS is a
    well-known way to break WSL2 internet access. On a developer machine that is a far more likely
    and far more confusing failure than losing Mobile Hotspot, and the copy must lead with it.
45. **The stock start type is unconfirmed.** The proposal asserts `Manual`. No authoritative Microsoft
    list of Windows 11 client default start types was found, and the research machine's own `Start`
    value cannot stand as evidence of a Windows default. Confirm against a clean 26100 image before
    writing the revert option; reverting a trigger-started service to the wrong start type is a real
    state leak.

### Corrections for `require_doh` (new)

46. **Two of the three `DoHPolicy` enum values were transposed.** The proposal's table said "1 Allow
    DoH, 2 Prohibit DoH, 3 Require DoH". The shipped 26100 `DnsClient.admx` and its ADML give
    `DNS_Doh_Force` = "Require encryption" = **3**, `DNS_Doh_Auto` = "Allow encryption" = **2**,
    `DNS_Doh_Disabled` = "Prohibit encryption" = **1**. The proposal's 3 is right; its 1 and 2 are
    swapped. A dropdown built from the proposal's table would have made "Allow DoH" write the value
    that turns encryption off.
47. **Three sibling elements are `required="true"`, and the applicability gate differs from the
    existing DoH tweak.** The policy also carries `DohPolicySetting` (0 allow DoH, 1 block DoH) and
    `DotPolicySetting` (0 allow DoT, 1 block DoT); DoT is already present in the 26100 ADMX, not a
    future 25H2 addition. Writing `DoHPolicy` alone still works at the DNS client level, but the
    corpus should know it is writing one of three required elements and say so. `supportedOn` is
    `SUPPORTED_Windows_10_0_20H2_SERVER_20H2`, so the policy is not Windows 11 only; if this ships as
    an option on `dns_over_https`, it would inherit that tweak's `windows: { products: [11] }` gate,
    which is narrower than the policy's own applicability (though the shipping DoH client itself is
    Windows 11 and Server 2022, so the practical effect on Windows 10 is likely nil).

### Corrections for `firewall_logging_and_merge` (new)

48. **`LogFilePath` and `LogFileSize` are `required="true"` alongside `LogDroppedPackets`.** Writing
    `LogDroppedPackets` alone leaves the policy half-configured from Group Policy tooling's point of
    view. Write all three together per profile, and revert all three together.
49. **Do not copy the ADMX `disabledList` string form.** The `disabledList` writes
    `LogDroppedPackets` and `LogSuccessfulConnections` as `<string>0</string>` while the enabled path
    writes decimals. The effective values are `REG_DWORD`. Writing `REG_SZ` here would be silently
    ignored.
50. **The `AllowLocalPolicyMerge = 0` failure mode must be stated bluntly, and the edition gate
    added.** On a standalone machine there is no policy-pushed rule set to fall back to, so turning
    merge off on the Public profile means inbound simply stops working there, silently, with no
    prompt: the familiar "Windows Defender Firewall has blocked some features of this app" flow never
    appears and games or servers fail to accept connections. Keep it a separate, clearly warned
    option; logging alone should be the default recommendation. Editions: Pro and above per the
    Firewall CSP; Home is out of scope for `AllowLocalPolicyMerge`. Note also that
    `AllowLocalPolicyMerge` is documented by the Firewall CSP (Default Value: true) but is **not**
    present in any of the 218 shipped ADMX files; the registry path is corroborated by CIS check text
    plus the corpus's own working use of the same profile keys in `security:firewall_all_profiles`.

## New in this revision

Seven tweaks join the category, all carried in from the gap hunt and each independently
re-verified against shipped 26100 ADMX, shipped binaries, or Policy CSP before being written up here.

| Tweak | What it adds that nothing in the corpus had | Verdict |
|---|---|---|
| `disable_mdns` | The third leg of the multicast name-resolution triad. The corpus disabled LLMNR and NetBIOS over TCP/IP and left mDNS, which is exploitable by exactly the same Responder-class poisoning. Windows grew a first-party Group Policy control for it. | VERIFIED-WITH-CORRECTION |
| `update_feature_control` | The most 24H2-specific control in the corpus. Since 22H2 Microsoft ships new *features* inside monthly quality updates behind temporary enterprise feature control, and offers optional content through the same channel. Neither `defer_quality_updates` nor `target_release_version` stops a feature arriving inside a monthly cumulative. | VERIFIED-WITH-CORRECTION |
| `defer_feature_updates` | Feature-update deferral by N days, the counterpart to the existing quality deferral. Different from `target_release_version`: a pin holds you on a named version until support ends and then jumps, while a deferral slides every feature update back by a chosen number of days. | VERIFIED |
| `block_insider_builds_policy` | Blocks Insider preview builds by policy. The corpus previously addressed Insider only by disabling the `wisvc` service, which is the weaker lever: a service can be restarted, a policy cannot be bypassed from the Settings UI. Keep both. | VERIFIED-WITH-CORRECTION |
| `disable_internet_connection_sharing` | Disables the ICS service and hides its UI. ICS can turn a client into an unmanaged router and NAT, and there is a dedicated DISA STIG rule for it. | VERIFIED |
| `require_doh` | The enforcing DoH control, as against the existing auto-upgrade one. `EnableAutoDoh = 2` falls back to plaintext; `DoHPolicy = 3` fails resolution instead. Different guarantees, and the enforcing one is what a privacy-motivated user actually wants. | VERIFIED-WITH-CORRECTION |
| `firewall_logging_and_merge` | Firewall drop logging per profile, plus the local-policy-merge control. The corpus's `firewall_all_profiles` turns the firewall on and blocks inbound but neither logs nor stops applications silently adding their own allow rules. | VERIFIED-WITH-CORRECTION |

Two of the seven change the shape of the category rather than just adding to it:

- **`disable_mdns` turns the name-resolution pair into a triad**, which strengthens merge candidate 3
  from "tidier" to "the natural shape of the control".
- **`defer_feature_updates` creates the corpus's first documented mutually-exclusive pair**
  (with `target_release_version`). That is a schema question, not just a copy question: the tweak
  system currently has no way to express "applying A makes B inert".

## Merge candidates

This file holds the strongest merge candidates in the corpus, and two of them are now correctness
issues rather than tidiness issues. Twelve of the twenty-seven tweaks are Windows Update controls
writing into two adjacent policy keys, several of which Microsoft documents as only working in
combination or as cancelling each other outright.

A note on what a merge costs, since the brief asks for it explicitly. In this schema a tweak is a
single-select control: 2 options render as a toggle, 3 or more as a dropdown. Merging N tweaks into
one therefore converts N independent booleans into one enumeration, and the granularity lost is
exactly the set of combinations the enumeration cannot express. That cost is only acceptable when the
lost combinations are either documented-inert or ones no user would deliberately choose. Each merge
below states which.

### Merge 1: Windows Update automatic behaviour and reboot (strongest, and required for correctness)

Group: `disable_auto_update_download` + `disable_auto_restart_logged_on`.

This is not merely tidier. Microsoft documents that `NoAutoRebootWithLoggedOnUsers` "only applies
when Configure Automatic Updates is set to option 4 - Auto download and schedule the install." As two
independent tweaks, a user can enable the reboot block without ever configuring automatic updates, in
which case the value sits in the registry with nothing to gate. The corpus currently lets a user
reach a state Microsoft documents as having no effect, and reports it as applied.

Proposed shape, one tweak "Windows Update automatic behaviour" over the surface
`{ no_auto_update, au_options, no_auto_reboot_logged_on }` in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU`:

| Option | `NoAutoUpdate` | `AUOptions` | `NoAutoRebootWithLoggedOnUsers` |
|---|---|---|---|
| Windows default (Stock Default) | absent | absent | absent |
| Notify of download and installation | 0 | 2 | absent |
| Automatically download and notify of installation | 0 | 3 | absent |
| Schedule installs, never reboot while signed in | 0 | 4 | 1 |

**Granularity lost:** the user cannot set `NoAutoRebootWithLoggedOnUsers = 1` alongside
`AUOptions = 2` or `3`. Per Microsoft that combination has no effect anyway, so the lost combinations
are precisely the inert ones. This is the rare merge that costs nothing real and removes a documented
trap. Note the merged tweak inherits `risk_level: high` from `disable_auto_update_download` and must
keep the Home warning: Home ignores the Windows Update AU policy entirely.

### Merge 2: driver installation control (strong)

Group: `exclude_wu_driver_updates` + `disable_auto_driver_install`.

Both tweaks' own copy already tells the user to run them together ("Combined with excluding drivers
from Windows Update, it gives you end-to-end say over driver installation"). They are one escalating
decision, not two orthogonal ones, and the escalation has a natural order: stop Windows Update
shipping drivers, then stop Windows searching for drivers at all.

Proposed shape, one tweak "Driver update control" over
`{ exclude_wu_drivers, search_order, prevent_metadata }`, **written to the documented policy keys**
(see corrections 24 and 25, which must land first or the merge bakes in the wrong paths):

| Option | `ExcludeWUDriversInQualityUpdate` | `SearchOrderConfig` | `PreventDeviceMetadataFromNetwork` |
|---|---|---|---|
| Windows default (Stock Default) | absent | 1 | absent |
| Exclude drivers from Windows Update | 1 | 1 | absent |
| Fully manual driver installation | 1 | 0 | 1 |

**Granularity lost:** the user cannot block device metadata retrieval without also blocking the
driver search. That is a privacy-only sub-case, and it arguably belongs in the privacy category
rather than here; pulling `PreventDeviceMetadataFromNetwork` out into privacy would sharpen both
files and reduce this merge to a clean two-value pair. Also lost: excluding Windows Update drivers is
Pro-and-above while the device-installation controls are not edition-gated, so a merged tweak has a
mixed applicability story and must say per option which halves apply on Home.

### Merge 3: multicast and broadcast name resolution (strong, and now a triad)

Group: `disable_llmnr` + `disable_netbios_tcpip` + `disable_mdns`.

With `disable_mdns` added this stops being a pair and becomes the complete set. All three are
broadcast or multicast name-resolution protocols vulnerable to the same Responder-class poisoning,
every security benchmark that recommends one recommends the others, and the existing tweaks' copy
already cross-references. A user picks how far down the ladder to go rather than mixing arbitrarily,
and the ladder has a real ordering by cost: LLMNR is nearly free, NetBIOS costs legacy device
discovery, mDNS costs `.local` names, AirPrint and IPP Everywhere printers, Chromecast and Google
Cast discovery, and Apple device discovery.

Proposed shape, one tweak "Legacy name resolution":

| Option | `EnableMulticast` | NetBIOS action | `EnableMDNS` (both keys) |
|---|---|---|---|
| All enabled (Stock Default) | absent | not run | absent |
| LLMNR off | 0 | not run | absent |
| LLMNR and NetBIOS off | 0 | run | absent |
| LLMNR, NetBIOS and mDNS off | 0 | run | 0 |

**Granularity lost:** the user cannot disable mDNS while keeping LLMNR, and cannot disable NetBIOS
while keeping LLMNR. Neither is a combination anyone would deliberately choose, because LLMNR is the
most dangerous and the cheapest to lose, so the ladder ordering matches the real cost ordering. What
*is* genuinely lost is the ability to disable mDNS alone, which a user with no printers but a real
need for `.local`-free hardening might want; that is a narrow case. One mechanical cost: the three
tweaks have different `requires_reboot` and different revert mechanics (two registry values, a
per-interface PowerShell action, and two more registry values), so the merged tweak inherits the
NetBIOS action's per-interface snapshot obligation from correction 3.

### Merge 4: feature-update cadence (now mandatory, because the pair is mutually exclusive)

Group: `target_release_version` + `defer_feature_updates`.

This one changed character with the new tweak. Microsoft states: "When you specify target version
policy, feature update deferrals won't be in effect." That is not a soft interaction. If
`target_release_version` is applied, `defer_feature_updates` is **fully inert**, and the corpus would
report it as applied. Shipping both as independent tweaks reproduces exactly the defect that makes
merge candidate 1 mandatory.

Proposed shape, one tweak "Feature update cadence" over
`{ target_enabled, target_version, target_product, defer_feature, defer_feature_days }` in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`:

| Option | `TargetReleaseVersion` | `TargetReleaseVersionInfo` | `ProductVersion` | `DeferFeatureUpdates` | `DeferFeatureUpdatesPeriodInDays` |
|---|---|---|---|---|---|
| Windows default (Stock Default) | absent | absent | absent | absent | absent |
| Defer feature updates 30 days | absent | absent | absent | 1 | 30 |
| Defer feature updates 180 days | absent | absent | absent | 1 | 180 |
| Defer feature updates 365 days | absent | absent | absent | 1 | 365 |
| Pin to a specific release | 1 | current release | "Windows 11" | absent | absent |

**Granularity lost:** nothing real. The excluded combination (a pin plus a deferral) is the one
Microsoft documents as inert. The merge also removes the trap where a user sets a deferral, sees it
reported applied, and gets no deferral. Two cautions carry into the merged tweak: the pinned release
must track currently supported releases (correction 15), and the merged tweak must not write
`PauseFeatureUpdatesStartTime`, nor assume it is absent on revert, because the user may have paused
updates from the Settings UI (correction 41).

### Merge 5: DNS encryption (strong, and the shape the verification pass recommends)

Group: `dns_over_https` + `require_doh`.

These are two mechanisms for one user intention with materially different guarantees.
`EnableAutoDoh = 2` is the auto-upgrade route: use DoH where the configured resolver supports it,
fall back to plaintext otherwise, unsupported by Microsoft since 2020, and inert on an ordinary ISP
resolver. `DoHPolicy = 3` is the enforcing, ADMX-backed, first-party route: fail resolution rather
than fall back. A user picks a posture; they do not combine these.

Proposed shape, one tweak "Encrypted DNS":

| Option | `EnableAutoDoh` (Dnscache\Parameters) | `DoHPolicy` (DNSClient policy) |
|---|---|---|
| Windows default (Stock Default) | absent | absent |
| Auto-upgrade where supported | 2 | absent |
| Allow encryption (policy) | absent | 2 |
| Require encryption (policy) | absent | 3 |

**Granularity lost:** the user cannot set both the unsupported registry auto-upgrade and the policy at
once. That is a benefit, not a cost, since the two would then disagree about what happens on
fallback. The real cost of the merge is an applicability mismatch: `dns_over_https` carries
`windows: { products: [11] }`, correct for `EnableAutoDoh`, while `DNS_Doh` declares
`SUPPORTED_Windows_10_0_20H2_SERVER_20H2`. A merged tweak either inherits the narrower gate (and
hides a policy that is declared for Windows 10 20H2 and later) or the wider one (and offers a policy
on Windows 10 where no shipping DoH client exists). The honest resolution is to keep the Windows 11
gate on the merged tweak and say why in the copy.

### Merge 6: Windows Update bandwidth (good)

Group: `disable_delivery_optimization_p2p` + `block_update_over_metered`.

Both answer "how much of my connection may Windows Update use". A user picks a level rather than
combining them arbitrarily.

| Option | `DODownloadMode` | `AllowAutoWindowsUpdateDownloadOverMeteredNetwork` |
|---|---|---|
| Windows default (Stock Default) | absent | absent |
| No peer-to-peer sharing | 0 | absent |
| No peering, nothing over metered | 0 | 0 |

**Granularity lost:** the user cannot pin the metered block while leaving LAN peering on. Low cost,
since LAN peering and metered downloads are rarely a meaningful pair. The merge also gives
`block_update_over_metered` a clearer identity: as a component of a stated bandwidth posture it reads
honestly, whereas as a standalone toggle its copy has to work hard to explain that it locks a default
rather than changing one. Note the mixed edition story: Delivery Optimization reads its policy on all
editions, the metered policy is Pro and above.

### Merge 7: update content control (weaker, propose with reservation)

Group: `update_feature_control` + `block_insider_builds_policy`.

Both write into `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, both are Pro and above,
and both answer "what kinds of change may arrive on this machine". A combined "Update content"
control with options for stability, no optional content, and no preview builds would read well.

**Granularity genuinely lost:** blocking Insider enrolment and refusing mid-cycle features are
orthogonal decisions. A user plausibly wants Insider blocked on a machine where they *do* want
optional content and CFRs, for example a test box that must never enrol in the Insider Program.
Merging would make those mutually exclusive for no documented reason. There is also an applicability
mismatch: `AllowTemporaryEnterpriseFeatureControl` is Windows 11 22H2 and later while
`ManagePreviewBuildsPolicyValue` is `SUPPORTED_Windows_10_0_RS3`, so a merged tweak would carry two
gates across its options. **Recommend keeping these separate** and grouping them visually.

### Merge 8: quality-update deferral with the version pin (rejected, keep separate)

Group: `defer_quality_updates` + `target_release_version`.

Proposed in the previous revision and now explicitly rejected. Quality-update deferral and
feature-version pinning are orthogonal in practice, and, importantly, Microsoft's inertness statement
covers **feature** update deferrals only: a target version pin does not make a quality deferral
inert. An administrator plausibly wants a 7-day quality deferral *and* a version pin, and that is a
supported, working combination. A merge would make them mutually exclusive for no reason.
`defer_quality_updates` should instead be grouped visually with merge 4.

### Explicitly not merge candidates

- **The power group:** `disable_hibernation`, `disable_usb_selective_suspend`, `disable_wake_timers`,
  `disable_nic_power_management`, `disable_modern_standby`. All power-related, but a user combines
  them freely rather than choosing between them; disabling wake timers says nothing about whether you
  also want hibernation gone. Merging would force false exclusivity across five independent
  decisions. They should share a UI group heading and nothing more.
- **`disable_nic_power_management` and `disable_usb_selective_suspend`** specifically look like a
  "device power" pair and are not one: different mechanisms (a per-adapter driver key value versus a
  power-scheme index), different revert obligations, and different symptoms. The apparent pair is
  cosmetic.
- **`disable_wpad` and `disable_ipv6_transition`.** Different subsystems, different failure modes,
  no shared decision.
- **`set_active_hours` and the reboot policies.** Active hours is a scheduling preference; the AU
  policies are an update-delivery posture. Microsoft notes active hours has *no effect* when
  `NoAutoRebootWithLoggedOnUsers` or "Always automatically restart at scheduled time" is enabled,
  which is an interaction worth documenting in both tweaks but not a reason to fuse them, because the
  two answer different questions and a user reasonably sets both.
- **`disable_internet_connection_sharing`.** Nothing else in the category touches the NAT engine.
- **`firewall_logging_and_merge` with `security:firewall_all_profiles`.** Tempting, and rejected:
  turning the firewall on is a posture almost everyone wants, while `AllowLocalPolicyMerge = 0` is an
  aggressive setting that silently breaks inbound on standalone machines. Folding the second into the
  first would put a foot-gun behind a control users are told to enable. Logging is a plausible
  addition to `firewall_all_profiles`; the merge control is not.

## Tweak entries

### `dns_over_https` DNS over HTTPS

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters` | `EnableAutoDoh` | `REG_DWORD` | Auto-upgrade to DoH | `2` |
| same | `EnableAutoDoh` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

Gated `windows: { products: [11] }`, `requires_reboot: true`, `risk_level: medium`.

Key, value name, type and the value 2 are Microsoft's own. The May 2020 Windows networking blog that
introduced the DoH client says verbatim: "Navigate to the
HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters registry key. Create a new
DWORD value named 'EnableAutoDoh'. Set its value to 2." The value is still live on the target build:
`EnableAutoDoh` appears as a UTF-16 string in `C:\Windows\System32\dnsrslvr.dll` on 26100.4061,
alongside `DohInterfaceSettings` and `DohFlags`, so the DNS Client service still reads it. It was
absent under `Dnscache\Parameters` on the test machine, consistent with the `absent` stock default.

`EnableAutoDoh = 2` is an auto-*upgrade* switch, not an enable switch. It tells the resolver to
promote plaintext queries to DoH **when the configured resolver is already on Windows' known-DoH
template list**: Cloudflare 1.1.1.1 and 1.0.0.1, Google 8.8.8.8 and 8.8.4.4, Quad9 9.9.9.9 and
149.112.112.112, plus anything added with `Add-DnsClientDohServerAddress`.

The documented, supported surfaces are different values entirely: the per-interface Settings UI
("DNS over HTTPS" under "Preferred DNS encryption"), the Group Policy "Configure DNS over HTTPS (DoH)
name resolution" under `Network\DNS Client` (which writes `DoHPolicy`, see `require_doh`),
`netsh dns add encryption`, and `Add-DnsClientDohServerAddress`, with per-server state at
`HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\InterfaceSpecificParameters\{interface GUID}\DohInterfaceSettings\Doh\{server IP}`
under a `DohFlags` value.

**Corrections needed:** corrections 1 and 2. No spelling, typing or default defect: key, name,
`REG_DWORD`, the value 2 and the `absent` stock default are all correct as authored. The defects are
completeness and framing. The tweak sets no resolver, so on a machine using its ISP's DNS server the
write is inert and the user gets no encryption at all while the app reports success. Either also set
a DoH-capable resolver (ideally by moving to `Add-DnsClientDohServerAddress` or `netsh dns add
encryption` plus the DoH Group Policy), or say in the first lines of the copy that it only upgrades
an already DoH-capable resolver. The copy must also stop calling the value "community-documented
only, with no official Microsoft source", which is now wrong: it is Microsoft-sourced but
Microsoft-unsupported, which is a different and more useful thing to tell the user.

**Ready-to-paste info block:**

````yaml
    info: |
      **Upgrades your DNS lookups to encrypted HTTPS, but only if you already use a resolver Windows recognises as DoH-capable.**

      ## What it does
      Sets `EnableAutoDoh = 2` under `Dnscache\Parameters`, which tells the Windows 11 DNS client to
      promote plaintext lookups to DNS over HTTPS whenever the configured resolver is on Windows'
      known-DoH list (Cloudflare, Google, Quad9). It does not choose a resolver for you, and it does
      not force encryption.

      ## Benefits
      - **Hides your lookups**: your ISP and anyone on the path stop seeing which names you resolve
      - **Blocks on-path tampering**: encrypted answers cannot be silently rewritten in transit
      - **No configuration UI needed**: one value, applied machine-wide across every adapter

      ## Drawbacks
      - **Does nothing by itself**: on an ordinary ISP resolver nothing is encrypted, because auto-upgrade only fires for known-DoH servers
      - **Unsupported by Microsoft**: the blog that published this value said registry DoH configuration would not be supported in release builds
      - **No feedback**: nothing in Settings reflects the change, so you cannot tell whether it took effect
      - **Breaks captive portals and internal DNS**: hotel and airport sign-in pages, and split-horizon corporate names, can stop resolving

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer. The built-in DoH client does not exist on Windows 10, including 22H2 and LTSC 2021, so this does nothing there.
      - **Takes effect**: after reboot
      - **Reverting**: deletes the value, restoring the Windows default of no auto-upgrade
      - **Set a DoH resolver first**: point the adapter at 1.1.1.1, 8.8.8.8 or 9.9.9.9, or this write is inert
      - **The supported route** is the "Require DoH" policy tweak, the Settings UI, or `Add-DnsClientDohServerAddress`

      ## Recommendation
      Use it only if you have already set a DoH-capable resolver such as Cloudflare or Quad9; on an
      ISP-assigned resolver it changes nothing. If you want encryption guaranteed rather than
      opportunistic, use the policy-based "Require DoH" control instead.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Windows Insiders can now test DNS over HTTPS](https://techcommunity.microsoft.com/blog/networkingblog/windows-insiders-can-now-test-dns-over-https/1381282)
      - [Secure DNS Client over HTTPS (DoH)](https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support)
      - [Add-DnsClientDohServerAddress](https://learn.microsoft.com/en-us/powershell/module/dnsclient/add-dnsclientdohserveraddress)
````

**Sources:**
1. Windows Insiders can now test DNS over HTTPS, Microsoft networking blog,
   https://techcommunity.microsoft.com/blog/networkingblog/windows-insiders-can-now-test-dns-over-https/1381282
   (tier B; gives the key, the `EnableAutoDoh` DWORD and the value 2 verbatim, and carries the
   "registry configuration of DoH will not be supported" caveat)
2. Secure DNS Client over HTTPS (DoH) on Windows Server 2022,
   https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support (tier A; the
   supported per-server route, `Add-DnsClientDohServerAddress -AutoUpgrade $True`, and the known-DoH
   server list)
3. Direct binary inspection, Windows 11 24H2 build 26100.4061: `EnableAutoDoh`,
   `DohInterfaceSettings` and `DohFlags` present as UTF-16 strings in
   `C:\Windows\System32\dnsrslvr.dll`, confirming the DNS Client still reads the value (primary
   measurement, tier A for existence only)
4. Enabling DNS over HTTPS on Windows 11, Windows OS Hub, https://woshub.com/enable-dns-over-https-windows/
   (tier C; independent corroboration of the known-DoH resolver list and of the requirement to
   configure a DoH-capable server)

### `disable_llmnr` Disable LLMNR

**Verdict:** VERIFIED

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `EnableMulticast` | `REG_DWORD` | Disabled | `0` |
| same | `EnableMulticast` | `REG_DWORD` | Enabled (Stock Default) | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

This is the registry backing for the Group Policy "Turn off multicast name resolution"
(`Turn_Off_Multicast` in `DnsClient.admx`), under Computer Configuration > Administrative Templates >
Network > DNS Client. Microsoft: "If you enable this policy setting, LLMNR will be disabled on all
available network adapters on the DNS client. If you disable this policy setting, or you don't
configure this policy setting, LLMNR will be enabled on all available network adapters." Polarity is
straightforward: the value is named for the feature and 0 turns it off. The policy-hive audit
confirms the ADMX declares the policy `class="Machine"` and the YAML writes HKLM, so the hive is
correct.

LLMNR is the multicast fallback that Responder-class tooling abuses to answer name queries and
harvest NTLM challenge-response material. CIS covers it as control 18.5.4.1.

Applicability per Policy CSP: Windows 10 2004 with KB5005101 and later (and 20H2 / 21H1 with the same
KB), Windows 11 21H2 and later, editions Pro, Enterprise, Education, IoT Enterprise and IoT
Enterprise LTSC. The DNS client itself has read `EnableMulticast` since Windows Vista, so that table
reflects when the MDM surface was added, not when the registry value started working.

**Corrections needed:** `none`. Key, value name, type, polarity and the `absent` stock default are
all correct. Two non-defect notes worth folding into the copy. (a) Microsoft does not document a
reboot requirement, but the DNS Client service reads the policy, so a reboot or `Dnscache` restart is
the honest statement and `requires_reboot: true` would be the more accurate flag; this is a
recommendation, not a defect, because no source states a requirement either way. (b) The policy is
documented for Pro and above, though the DNS client reads the value regardless of edition, so the
practical Home story is "probably works, not documented".

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off the legacy LLMNR name-resolution fallback, closing a favourite target for local-network credential theft.**

      ## What it does
      Sets the `EnableMulticast` policy value to 0, the registry form of the Group Policy "Turn off
      multicast name resolution". LLMNR is the fallback Windows uses to find machines by name when
      DNS does not answer, and it broadcasts those queries to the whole local network.

      ## Benefits
      - **Closes LLMNR poisoning**: the highest-yield, lowest-effort attack on a flat Windows network
      - **Stops credential capture**: tools like Responder answer the broadcast and harvest NTLM hashes
      - **Benchmark-backed**: CIS control 18.5.4.1 recommends exactly this
      - **Survives updates**: it is a policy value, not a preference

      ## Drawbacks
      - **Short names may fail**: on a network with no DNS server, single-label lookups fall back to NetBIOS or stop working
      - **Pairs with NetBIOS**: if you also disable NetBIOS, short-name discovery on a DNS-less network stops entirely
      - **Not the whole picture**: mDNS is a third broadcast protocol and needs its own control

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021
      - **Takes effect**: after reboot, when the DNS Client service next reads the policy
      - **Reverting**: deletes the value, which is the Windows default state (LLMNR enabled)
      - **Editions**: documented for Pro, Education, Enterprise and IoT Enterprise; the DNS client reads the value on Home too, but Microsoft does not document that

      ## Recommendation
      Apply it. On any network with a working DNS server, which includes essentially every home
      router, the cost is zero and the security gain is real. Skip it only if you depend on short-name
      discovery on a network with no DNS at all.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP - ADMX_DnsClient, Turn_Off_Multicast](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient)
      - [CIS 18.5.4.1, Turn off multicast name resolution](https://www.tenable.com/audits/items/CIS_DC_SERVER_2012_Level_1_v2.2.0.audit:f9584c0c934378405ec052736485f437)
````

**Sources:**
1. Policy CSP - ADMX_DnsClient, `Turn_Off_Multicast`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient (tier A)
2. `C:\Windows\PolicyDefinitions\DnsClient.admx` on build 26100, policy `Turn_Off_Multicast`,
   `class="Machine"`, `key="Software\Policies\Microsoft\Windows NT\DNSClient"`,
   `valueName="EnableMulticast"` (tier A, shipped ADMX)
3. CIS Benchmark control 18.5.4.1 "Ensure 'Turn off multicast name resolution' is set to Enabled",
   via Tenable audit item,
   https://www.tenable.com/audits/items/CIS_DC_SERVER_2012_Level_1_v2.2.0.audit:f9584c0c934378405ec052736485f437
   (tier B)

### `disable_netbios_tcpip` Disable NetBIOS over TCP/IP

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** an HKCU state marker plus a PowerShell action.

State marker: `HKCU\Software\MagicXToolbox\State` `NetbiosTcpip` `REG_DWORD`, 1 when applied, 0 when
not.

Action, `shell: powershell`, verbatim:

```powershell
# apply
Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces' |
  Where-Object { $_.PSChildName -like 'Tcpip_*' } |
  ForEach-Object { Set-ItemProperty -Path $_.PSPath -Name NetbiosOptions -Value 2 -Type DWord }

# undo
Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces' |
  Where-Object { $_.PSChildName -like 'Tcpip_*' } |
  ForEach-Object { Set-ItemProperty -Path $_.PSPath -Name NetbiosOptions -Value 0 -Type DWord }

# probe
$ifaces = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces' |
  Where-Object { $_.PSChildName -like 'Tcpip_*' }
if (-not $ifaces) { exit 1 }
foreach ($i in $ifaces) {
  if ((Get-ItemProperty $i.PSPath -Name NetbiosOptions -ErrorAction SilentlyContinue).NetbiosOptions -ne 2) { exit 1 }
}
exit 0
```

`requires_reboot: true`, `risk_level: medium`.

`NetbiosOptions` is Microsoft's documented per-interface NetBT mode selector, `REG_DWORD`, with 0
meaning "use the DHCP server's setting" (the default), 1 meaning NetBIOS over TCP/IP enabled, and 2
meaning disabled. Setting 2 stops the NetBIOS Name Service broadcasts that Responder-class tools
spoof, and is the registry equivalent of selecting "Disable NetBIOS over TCP/IP" on the WINS tab of a
connection's advanced TCP/IP properties. On the test machine all seven `Tcpip_*` interfaces carried
`NetbiosOptions = 0` **(measured)**, consistent with the documented default.

Microsoft states a change takes effect after `ipconfig /renew` on a DHCP interface and otherwise
after a reboot, so `requires_reboot: true` is correct and conservative.

**Corrections needed:** corrections 3 and 4. The `undo` writes a fixed 0 rather than restoring the
snapshotted per-interface value, which destroys a deliberate 1 set by an admin or an unattend file;
and adapters created after apply are not covered, so a new USB NIC, VPN adapter or Hyper-V virtual
switch silently keeps NetBIOS. The probe correctly fails in the second case, so it surfaces as drift.
One further note from the cross-category audit: the HKCU state marker makes the applied-state
bookkeeping per-user while the change is machine-wide, so a second account sees the tweak as not
applied even though it is.

**Ready-to-paste info block:**

````yaml
    info: |
      **Shuts off the ancient NetBIOS name service on every adapter, removing a second broadcast attack surface that credential-theft tools abuse.**

      ## What it does
      Writes `NetbiosOptions = 2` to every `Tcpip_*` interface under `NetBT\Parameters\Interfaces`,
      the registry equivalent of selecting "Disable NetBIOS over TCP/IP" on the WINS tab. Like LLMNR,
      NBT-NS answers name requests by broadcast, which attackers hijack.

      ## Benefits
      - **Closes NBT-NS poisoning**: the twin of LLMNR poisoning, targeted by the same tooling
      - **Completes the pair**: with LLMNR off, the two classic broadcast name-resolution holes are both shut
      - **Machine-wide**: applied to every adapter present, not just the active one

      ## Drawbacks
      - **Breaks legacy discovery**: NetBIOS-only NAS boxes, old network printers and some line-of-business apps stop resolving
      - **Not applied to new adapters**: a USB NIC, VPN adapter or Hyper-V switch added later keeps NetBIOS until you re-apply
      - **WINS networks**: any network still using WINS depends on this
      - **`net view` browsing** stops working where it relied on NetBIOS names

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions
      - **Takes effect**: after reboot (or `ipconfig /renew` on a DHCP interface)
      - **Reverting**: restores the previous per-interface value from the snapshot
      - Modern file sharing over SMB on port 445 with DNS names is unaffected

      ## Recommendation
      Apply it on any modern network, ideally together with disabling LLMNR. Skip it if you still run
      genuinely old NetBIOS-only hardware, or test first and re-enable NetBIOS on the one adapter that
      needs it.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [NetbiosOptions (Core Services)](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc776524(v=ws.10))
      - [TCP/IP and NBT configuration parameters](https://learn.microsoft.com/en-us/troubleshoot/windows-client/networking/tcpip-and-nbt-configuration-parameters)
      - [Unattend NetbiosOptions setting](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-netbt-interfaces-interface-netbiosoptions)
````

**Sources:**
1. NetbiosOptions (Core Services),
   https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc776524(v=ws.10)
   (tier A; the 0 / 1 / 2 semantics and the default of 0)
2. TCP/IP and NBT configuration parameters,
   https://learn.microsoft.com/en-us/troubleshoot/windows-client/networking/tcpip-and-nbt-configuration-parameters
   (tier A; the key path and the `ipconfig /renew` versus reboot behaviour)
3. Unattend `NetbiosOptions` setting,
   https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-netbt-interfaces-interface-netbiosoptions
   (tier A; independent confirmation of the value semantics)
4. Direct registry read of `HKLM\SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces` on
   Windows 11 24H2 build 26100 (primary measurement; seven interfaces, all `NetbiosOptions = 0`)

### `disable_wpad` Disable WPAD auto-proxy discovery

**Verdict:** VERIFIED

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings\WinHttp` | `DisableWpad` | `REG_DWORD` | Disabled | `1` |
| same | `DisableWpad` | `REG_DWORD` | Enabled (Stock Default) | `absent` |

`requires_reboot: true`, `risk_level: medium`.

Microsoft's support article "How to disable HTTP proxy features" states: "Starting in Windows Server
2019 and Windows 10, version 1809, you can disable WPAD by setting a DWORD value for the following
registry subkey to 1: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings\WinHttp\DisableWpad`".
Key path, value name, type and value all match the YAML exactly. It stops WPAD detection for every
proxy-detection call made through the WinHTTP API.

This is a product key rather than a policy key, so there is no edition gate and it applies on Home.
Microsoft does not state a reboot requirement, but WinHTTP caches proxy configuration per process, so
`requires_reboot: true` is a safe over-approximation. The YAML's deliberate decision not to touch
`WinHttpAutoProxySvc` is correct: many components depend on that service.

**Corrections needed:** `none` on the mechanism. Correction 5 applies to the copy: add Microsoft's
two caveats, that applications can still resolve the name "WPAD" by calling DNS directly, and that
WPAD should also be disabled in the Windows Settings UI because third-party apps and browsers may
rely on that setting for proxy auto-discovery. Without them a user overestimates the coverage.

**Ready-to-paste info block:**

````yaml
    info: |
      **Blocks Windows from auto-hunting the network for proxy settings, killing a classic man-in-the-middle trick that reroutes your web traffic.**

      ## What it does
      Sets `DisableWpad = 1` under `Internet Settings\WinHttp`, the method Microsoft documents for
      switching Web Proxy Auto-Discovery off since Windows 10 1809. Every proxy-detection call made
      through the WinHTTP API stops looking for a WPAD configuration file.

      ## Benefits
      - **Kills a live MITM vector**: a rogue device can no longer answer WPAD and route your traffic through its proxy
      - **Documented switch**: Microsoft's own supported way to turn WPAD off
      - **Free if unused**: costs nothing on any network that does not hand out proxy config by WPAD
      - **Works on Home**: a product key, not a policy key, so no edition gate

      ## Drawbacks
      - **Breaks WPAD networks**: where proxy settings are distributed this way, web access stops until you set a proxy by hand
      - **Not complete coverage**: applications can still resolve the name "WPAD" through plain DNS, so DNS-level WPAD queries continue
      - **Settings UI still matters**: third-party apps and browsers may read the Windows Settings proxy auto-discovery toggle, which this does not change

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later including LTSC 2021, all editions
      - **Takes effect**: after reboot, because WinHTTP caches proxy configuration per process
      - **Reverting**: deletes the value, restoring automatic proxy discovery
      - This deliberately does not disable the `WinHttpAutoProxySvc` service, because many Windows components depend on it

      ## Recommendation
      Apply it on any home or small-office machine that does not use automatic proxy configuration,
      and also turn off auto-discovery in Settings for full coverage. On a corporate network that
      distributes proxy settings by WPAD, leave it alone or configure the proxy manually first.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [How to disable HTTP proxy features](https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/disable-http-proxy-auth-features)
      - [WinHTTP AutoProxy functions](https://learn.microsoft.com/en-us/windows/win32/winhttp/winhttp-autoproxy-api)
````

**Sources:**
1. How to disable HTTP proxy features,
   https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/disable-http-proxy-auth-features
   (tier A; gives the key, value name, type and value 1 verbatim, plus both caveats)
2. WinHTTP AutoProxy Functions,
   https://learn.microsoft.com/en-us/windows/win32/winhttp/winhttp-autoproxy-api (tier A; what WPAD
   detection does and which API calls it covers)

### `disable_ipv6_transition` Disable IPv6 transition technologies

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters` | `DisabledComponents` | `REG_DWORD` | Tunnels disabled | `0x01` |
| same | `DisabledComponents` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

`requires_reboot: true`, `risk_level: medium`.

`DisabledComponents` is a documented bitmask. Microsoft's table maps the low bit to "Disable tunnel
interfaces", and the guidance section says directly: "You can disable the 6to4 tunneling protocol and
other IPv6 transition Technologies by using one of the following methods: Set the
`DisabledComponents` registry key to 0x01." Minimum is 0x00 (the default), maximum 0xFF (IPv6 fully
disabled). Native IPv6 is bit 4 (0x10) and is untouched by 0x01, so the YAML's claim that native IPv6
keeps working is correct. Microsoft states "You must restart your computer for these changes to take
effect", so `requires_reboot: true` is correct and required.

**Corrections needed:** corrections 6 and 7. Add the RRAS warning ("Values other than 0 or 32 causes
the Routing and Remote Access service to fail after this change takes effect"), which contradicts the
current copy's "Safe and recommended for most users". And correct the claimed delta: Microsoft states
"ISATAP and Teredo are disabled by default in Windows", so on a stock machine the incremental effect
is 6to4 and the generic tunnel interface only. The copy is right to warn against confusing `0x01`
with `0xFF`.

**Ready-to-paste info block:**

````yaml
    info: |
      **Drops the legacy IPv6 tunnel interfaces so 6to4 cannot quietly build a tunnel and register its addresses in DNS, while native IPv6 keeps working.**

      ## What it does
      Sets `DisabledComponents = 0x01` on the IPv6 stack, the low bit of Microsoft's documented
      bitmask, which disables the tunnel interfaces (6to4, Teredo, ISATAP). Native IPv6 lives in a
      different bit and is untouched.

      ## Benefits
      - **Stops 6to4 auto-tunnels**: 6to4 enables itself whenever an interface holds a public IPv4 address, and Microsoft itself recommends disabling it when that is unwanted
      - **Fewer address leaks**: no tunnel addresses registered in DNS and no unexpected connectivity path
      - **Native IPv6 intact**: this is bit 0, not the 0x10 bit that disables native IPv6

      ## Drawbacks
      - **Breaks Routing and Remote Access**: Microsoft states any value other than 0 or 32 causes RRAS to fail, and `0x01` is such a value
      - **Smaller change than it sounds**: ISATAP and Teredo are already disabled by default in Windows, so on a stock machine only 6to4 and the generic tunnel interface actually go away
      - **Teredo-dependent apps**: some older peer-to-peer software and legacy console networking need Teredo

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions
      - **Takes effect**: after reboot, which Microsoft requires for this value
      - **Reverting**: deletes the value, restoring the Windows default of 0x00
      - **Do not confuse `0x01` with `0xFF`**: 0xFF disables IPv6 entirely, which Microsoft does not support and which breaks Windows features that assume IPv6 is present

      ## Recommendation
      Worth applying on an ordinary client that does not need tunnelled IPv6, which is most machines.
      Do not apply it on anything running Routing and Remote Access, or on a machine that depends on
      Teredo or on an ISATAP intranet deployment.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Guidance for configuring IPv6 in Windows for advanced users](https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/configure-ipv6-in-windows)
      - [IPv6 transition technologies overview](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/bb726951(v=technet.10))
````

**Sources:**
1. Guidance for configuring IPv6 in Windows for advanced users,
   https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/configure-ipv6-in-windows
   (tier A; the `DisabledComponents` bitmask table, the 0x01 recommendation, the RRAS warning, the
   restart requirement, and the statement that ISATAP and Teredo are disabled by default)
2. IPv6 transition technologies,
   https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/bb726951(v=technet.10)
   (tier A; what 6to4, Teredo and ISATAP are and when each activates)

### `disable_nic_power_management` Disable NIC power management

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** an HKCU state marker plus a PowerShell action.

State marker: `HKCU\Software\MagicXToolbox\State` `NicPowerManagement` `REG_DWORD`, 1 when applied.

Target key: `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}\<NNNN>`,
value `PnPCapabilities`, `REG_DWORD`, written as `24` (`0x18`).

Action, `shell: powershell`, verbatim:

```powershell
# apply
Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}' -ErrorAction SilentlyContinue |
  Where-Object { $_.PSChildName -match '^\d{4}$' -and (Get-ItemProperty $_.PSPath -Name DriverDesc -ErrorAction SilentlyContinue) } |
  ForEach-Object { Set-ItemProperty -Path $_.PSPath -Name PnPCapabilities -Value 24 -Type DWord }

# undo
Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}' -ErrorAction SilentlyContinue |
  Where-Object { $_.PSChildName -match '^\d{4}$' } |
  ForEach-Object { Remove-ItemProperty -Path $_.PSPath -Name PnPCapabilities -ErrorAction SilentlyContinue }

# probe
$keys = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}' -ErrorAction SilentlyContinue |
  Where-Object { $_.PSChildName -match '^\d{4}$' }
foreach ($k in $keys) {
  if ((Get-ItemProperty $k.PSPath -Name PnPCapabilities -ErrorAction SilentlyContinue).PnPCapabilities -eq 24) { exit 0 }
}
exit 1
```

`requires_reboot: true`, `risk_level: medium`, carries a `warning`.

Microsoft documents this value, in the support article "Power management setting on a network
adapter" (originally KB 2740020). It gives the key as
`HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}\DeviceNumber`,
names the value `PnPCapabilities`, and states: "By default, a value of 0 indicates that power
management of the network adapter is enabled. A value of 24 will prevent Windows from turning off the
network adapter or let the network adapter wake the computer from standby." It maps the three Device
Manager checkboxes to values: all three checked is `0x100 (256)`, only the first checked is
`0x110 (272)`, the first unchecked with the other two greyed out is `0x118 (280)`, and the default
state of the first two checked is `0`. It then says: "For deployment purpose, to keep option 1
cleared, one needs to use the value 24 (0x18)." So 24 is Microsoft's own documented deployment value,
not folklore. The reader is NDIS: `PnPCapabilities` appears as a UTF-16 string in
`C:\Windows\System32\drivers\ndis.sys` on 26100.4061 and in neither `pci.sys` nor `umpnpmgr.dll`.

Real stock state is per-adapter and mixed, which is what matters for revert. On the test machine
exactly one of sixteen network class subkeys carried a `PnPCapabilities` value at all: the MediaTek
Wi-Fi 6 MT7921 adapter, with `16` **(measured)**. The other fifteen, including the Realtek GbE
controller and nine WAN Miniports, had no value. The correct restore state is therefore "whatever
this particular key held before, including absent", never a blanket delete and never a blanket
literal.

**Corrections needed:** corrections 8 through 12. The value is settled and Microsoft-documented; every
remaining defect is in the action. (a) `undo` deletes `PnPCapabilities` instead of restoring the prior
per-key state, destroying a vendor INF value such as the 16 measured here. (b) The undo filter lacks
the `DriverDesc` guard the apply has, so it touches keys the apply never wrote. (c) The apply is not
scoped to physical adapters and writes 24 into virtual adapters, WAN Miniports and the Bluetooth PAN
device. (d) The probe returns "applied" if any single key holds 24, including one a vendor INF set.
(e) The `warning` text calling the value "community-sourced" is now wrong and should point at the
blanket write and the delete-based undo instead. `Set-NetAdapterPowerManagement
-AllowComputerToTurnOffDevice Disabled` remains the better mechanism: per-adapter, documented, and it
round-trips cleanly.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows parking your network adapter to save power, curing Wi-Fi that drops when the PC sits idle.**

      ## What it does
      Writes `PnPCapabilities = 24` into the network adapter driver keys, which is Microsoft's
      documented deployment value for clearing "Allow the computer to turn off this device to save
      power". Windows then leaves the adapter powered instead of parking it when idle.

      ## Benefits
      - **Stops idle disconnects**: the standard fix for Wi-Fi that drops after the machine sits unused
      - **No post-idle latency**: traffic resumes on a link Windows never parked
      - **Microsoft-documented value**: 24 (0x18) is the value Microsoft names for deployment, not a community guess
      - **Endorsed by Microsoft tooling**: the Exchange Health Checker "Sleepy NIC Check" flags NIC power saving as a cause of packet loss

      ## Drawbacks
      - **Higher idle power**: measurable battery cost on laptops
      - **Written to every adapter**: the current action also writes to virtual adapters, WAN Miniports and the Bluetooth PAN device, none of which have a power management tab
      - **Revert is imperfect today**: the undo deletes the value rather than restoring whatever your adapter vendor had set
      - **Stock state varies**: some adapters ship with a vendor value already present and some with none, so there is no single correct "off"

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions, on physical adapters whose driver exposes the Power Management tab
      - **Takes effect**: after reboot, when the device next starts
      - **Reverting**: restores the previous per-adapter value from the snapshot
      - Microsoft's supported per-adapter alternative is `Set-NetAdapterPowerManagement -AllowComputerToTurnOffDevice Disabled`

      ## Recommendation
      Worth it on a desktop, or on any machine that actually suffers idle network drops. On a laptop
      with no connectivity symptoms, leave power management enabled and keep the battery life.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Power management setting on a network adapter (KB 2740020)](https://learn.microsoft.com/en-us/troubleshoot/windows-client/networking/power-management-on-network-adapter)
      - [Set-NetAdapterPowerManagement](https://learn.microsoft.com/en-us/powershell/module/netadapter/set-netadapterpowermanagement)
      - [How NDIS sets the power policy for a network adapter](https://learn.microsoft.com/en-us/windows-hardware/drivers/network/how-ndis-sets-the-power-policy-for-a-network-adapter)
````

**Sources:**
1. Power management setting on a network adapter (originally KB 2740020),
   https://learn.microsoft.com/en-us/troubleshoot/windows-client/networking/power-management-on-network-adapter
   (tier A; the key, the `PnPCapabilities` value name, the default of 0, "A value of 24 will prevent
   Windows from turning off the network adapter or let the network adapter wake the computer from
   standby", the 256 / 272 / 280 checkbox mapping, and "For deployment purpose, to keep option 1
   cleared, one needs to use the value 24 (0x18)")
2. How NDIS Sets the Power Policy for a Network Adapter,
   https://learn.microsoft.com/en-us/windows-hardware/drivers/network/how-ndis-sets-the-power-policy-for-a-network-adapter
   (tier A; checkbox semantics and defaults)
3. Set-NetAdapterPowerManagement,
   https://learn.microsoft.com/en-us/powershell/module/netadapter/set-netadapterpowermanagement
   (tier A; the documented per-adapter alternative)
4. Exchange Health Checker "Sleepy NIC Check",
   https://microsoft.github.io/CSS-Exchange/Diagnostics/HealthChecker/SleepyNICCheck/ (tier A/B;
   Microsoft's own diagnostic tooling validates NIC power-saving options and recommends disabling
   them because they can cause packet loss)
5. Direct binary and registry inspection, Windows 11 24H2 build 26100.4061: `PnPCapabilities` present
   as a UTF-16 string in `ndis.sys` and absent from `pci.sys` and `umpnpmgr.dll`; 1 of 16 network
   class subkeys carried a value (MediaTek Wi-Fi 6 MT7921, `16`) (primary measurement)

### `disable_delivery_optimization_p2p` Disable Delivery Optimization P2P

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization` | `DODownloadMode` | `REG_DWORD` | HTTP only, no peering | `0` |
| same | `DODownloadMode` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

No `requires_reboot` flag, `risk_level: low`.

The shipped 26100 `DeliveryOptimization.admx` confirms policy `DownloadMode`,
`key="SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization"`, `valueName="DODownloadMode"`, with
the enumeration 0, 1, 2, 3, 99, 100 and the help string "0 = HTTP only, no peering ... 1 = HTTP
blended with peering behind the same NAT ... 100 = Bypass mode, deprecated in Windows 11". Mode 0 is
"HTTP Only": "This setting disables peer-to-peer caching but still allows Delivery Optimization to
download content over HTTP from the download's original source or a Microsoft Connected Cache
server."

**The unconfigured default is unresolved and both sources are tier A.** The Delivery Optimization
reference page says "Default is configured to LAN(1)". The Policy CSP page for `DODownloadMode` gives
"Default Value: 0" and marks the allowed value list "0 (Default) HTTP only, no peering". This matters
because it is the difference between the tweak changing behaviour and being a no-op on a stock
machine, and it cannot be resolved from documentation alone. It does **not** threaten revert:
`absent` is the correct stock state for the policy value under either reading, and the policy key was
absent on 26100.4061 **(measured)**. `DoSvc` was `DEMAND_START` with a Group Policy start trigger on
the same machine, confirming the service picks the policy up without a reboot, so the omitted
`requires_reboot` flag is correct.

**Corrections needed:** corrections 13 and 14. No mechanism defect: key, value name, `REG_DWORD`, the
0 semantics and the `absent` stock default are all correct. Two copy corrections. Remove the claim
that this stops uploads "across the internet": internet peering is mode 3, and mode 3 is not the
stock default under either of Microsoft's two readings, so a stock consumer machine is not seeding to
strangers. State the benefit as removing same-subnet LAN peering and pinning the mode so peering
cannot be enabled later. Remove or mark anecdotal the `0x80d03002` error code. Record the unresolved
default rather than asserting LAN(1). The tweak is not diminished by this: the control is real, and
pinning the download mode is the point.

**Ready-to-paste info block:**

````yaml
    info: |
      **Pins Windows Update to downloading straight from Microsoft, with no peer-to-peer sharing on your network.**

      ## What it does
      Sets `DODownloadMode = 0` under the Delivery Optimization policy key, which Microsoft defines as
      "HTTP Only": no peer-to-peer caching, downloads coming from the original source or a Microsoft
      Connected Cache. Windows Update and Store downloads keep working.

      ## Benefits
      - **No LAN peering**: your PC stops sharing update chunks with other machines behind the same router
      - **Locks the mode**: with the policy set, nothing can turn peering on later, from Settings or from an image change
      - **Predictable traffic**: updates come from one place, which is easier to reason about on a capped link
      - **Service stays intact**: this changes the mode only and does not disable `DoSvc`, which the Store depends on

      ## Drawbacks
      - **More total WAN traffic**: with several Windows PCs on one network, each downloads its own copy
      - **Smaller change than it sounds**: your PC was never seeding to strangers on the internet, because that is download mode 3 and is not the default
      - **Default is ambiguous**: Microsoft's reference page says the unconfigured default is LAN peering while its Policy CSP page says it is no peering, so on some machines this may change nothing visible

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1511 and later including LTSC 2021. The Delivery Optimization client reads this policy on all editions, including Home.
      - **Takes effect**: immediately, the Delivery Optimization service picks the policy up without a reboot
      - **Reverting**: deletes the value, returning the machine to Windows' unconfigured behaviour
      - **Avoid mode 100**: Microsoft deprecated Bypass mode in Windows 11 and says it "can cause some content to fail to download"

      ## Recommendation
      A sensible default for a single-PC household, especially on a connection with limited upload.
      Leave peering on if you run several Windows PCs on the same network and want them to share
      downloads locally.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Delivery Optimization reference, Download Mode](https://learn.microsoft.com/en-us/windows/deployment/update/waas-delivery-optimization-reference)
      - [Policy CSP - DeliveryOptimization, DODownloadMode](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-deliveryoptimization)
      - [Manage connections from Windows to Microsoft services, section 28.3](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
````

**Sources:**
1. Delivery Optimization reference, Download Mode,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-delivery-optimization-reference
   (tier A; the mode enumeration, "HTTP blended with peering behind the same NAT" for mode 1, the
   mode 100 deprecation, and "Default is configured to LAN(1)")
2. Policy CSP - DeliveryOptimization, `DODownloadMode`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-deliveryoptimization
   (tier A; "Default Value: 0" and "0 (Default) HTTP only, no peering", which contradicts source 1)
3. Manage connections from Windows to Microsoft services, section 28.3,
   https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services
   (tier A; confirms the registry location and the `REG_DWORD` type)
4. Shipped ADMX and ADML, `C:\Windows\PolicyDefinitions\DeliveryOptimization.admx` and
   `en-US\DeliveryOptimization.adml` on build 26100.4061: policy `DownloadMode`,
   `key="SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization"`, `valueName="DODownloadMode"`,
   enumeration 0/1/2/3/99/100 (tier A, primary)

### `defer_quality_updates` Defer quality updates

**Verdict:** VERIFIED

**Mechanism:** two registry effects, both in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`.

| Value name | Type | Defer 7 days | Defer 14 days | Windows default (Stock Default) |
|---|---|---|---|---|
| `DeferQualityUpdates` | `REG_DWORD` | `1` | `1` | `absent` |
| `DeferQualityUpdatesPeriodInDays` | `REG_DWORD` | `7` | `14` | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

Microsoft's Windows Update client policies documentation lists the GPO "Select when Quality Updates
are received" as setting both `\Policies\Microsoft\Windows\WindowsUpdate\DeferQualityUpdates` and
`\Policies\Microsoft\Windows\WindowsUpdate\DeferQualityUpdatesPeriodInDays` under `HKLM\Software`.
Policy CSP gives `DeferQualityUpdatesPeriodInDays` as the range 0 to 30 with default 0, and maps it
to the ADMX policy `DeferQualityUpdates` under Windows Components > Windows Update > Manage updates
offered from Windows Update. Both value names, both types, the range and the enable flag match the
YAML, and 7 and 14 are inside the documented range.

Editions: Pro (including Pro for Workstations), Education and Enterprise (including Enterprise LTSC,
IoT Enterprise and IoT Enterprise LTSC), Windows 10 1607 and later. **Home is excluded and ignores
the policy entirely.** No reboot required; the Windows Update client re-evaluates policy on its next
scan.

**Corrections needed:** `none` on the mechanism. One copy correction: the claim that "On 24H2 clean
installs the Group Policy UI for this may be missing, but the registry key is still honored on Pro"
has no source found in two passes; either source it or remove it. Also worth adding: Microsoft
advises "To help ensure that devices stay secure, configure quality update deferral to be less than
3 days", which is shorter than either option offered, so a 3-day option would match Microsoft's own
guidance. This tweak is **not** made inert by `target_release_version`: Microsoft's inertness
statement covers feature-update deferrals only.

**Ready-to-paste info block:**

````yaml
    info: |
      **Holds back the monthly security rollup for a week or two, so a bad patch gets reported before it reaches you.**

      ## What it does
      Sets `DeferQualityUpdates = 1` and `DeferQualityUpdatesPeriodInDays` to 7 or 14 under the
      Windows Update policy key, the registry form of "Select when Quality Updates are received".
      The updates still install, just later.

      ## Benefits
      - **Buffer against bad patches**: by the time a deferred update reaches you, problems others hit have usually surfaced
      - **Still fully patched**: a deferral is not a block, so nothing is skipped
      - **Middle ground**: control over cadence without turning automatic updates off

      ## Drawbacks
      - **Unpatched window**: you run without the newest security fixes for the deferral period
      - **Longer than Microsoft advises**: Microsoft recommends keeping quality deferral under 3 days, so both 7 and 14 exceed that
      - **Does nothing on Home**: Windows Home does not honour Windows Update client policies at all
      - **Not a pause**: you cannot skip a month, only delay it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores this policy.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes both values, restoring Windows' normal cadence
      - Deferring quality updates is independent of pinning a feature version; the two work together

      ## Recommendation
      Use it on Pro if you want to dodge bad patches, and keep the deferral short. Do not use it on
      Home, where it does nothing, and do not treat it as a substitute for patching.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Configure Windows Update client policies](https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb)
      - [Policy CSP - Update, DeferQualityUpdatesPeriodInDays](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. Configure Windows Update client policies, "Configure when devices receive quality updates",
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb (tier A; both
   registry values and the under-3-days recommendation)
2. Policy CSP - Update, `DeferQualityUpdatesPeriodInDays`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; range 0
   to 30, default 0, ADMX mapping)
3. What are Windows Update client policies?,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A; the
   supported edition list, with Home absent)

### `target_release_version` Pin Windows feature version

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** three registry effects, all in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`.

| Value name | Type | Pinned to 24H2 | Windows default (Stock Default) |
|---|---|---|---|
| `TargetReleaseVersion` | `REG_DWORD` | `1` | `absent` |
| `TargetReleaseVersionInfo` | `REG_SZ` | `"24H2"` | `absent` |
| `ProductVersion` | `REG_SZ` | `"Windows 11"` | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

The ADMX policy "Select the target Feature Update version" writes an enable flag plus two strings
under the WindowsUpdate policy key. Policy CSP documents `TargetReleaseVersion` as "a string
containing Windows version number. For example, 1809, 1903" and `ProductVersion` as "a string
containing a Windows product. For example, 'Windows 11' or '11' or 'Windows 10'", with the explicit
note "You need to set up the ProductVersion CSP along with the TargetReleaseVersion CSP for it to
work." The YAML's three-value structure (DWORD enable flag plus two `REG_SZ` strings) matches the
ADMX shape and the types are right; writing the version as a DWORD would be silently ignored.

Applicability: `TargetReleaseVersion` needs Windows 10 1803 with KB4556807, 1809 with KB4551853, 1903
or 1909 with KB4556799, or 2004 and later. `ProductVersion` needs Windows 10 2004 with KB5005101 (or
20H2 / 21H1 with the same KB) and Windows 11 21H2 and later. Editions: Pro, Enterprise, Education,
IoT Enterprise and IoT Enterprise LTSC. **Home ignores it.** No reboot required.

**Corrections needed:** corrections 15, 16 and 17. The offered pin must track currently supported
releases: as of 2026-07-27 Microsoft lists Windows 11 24H2 end of updates for Home, Pro, Pro
Education and Pro for Workstations as 2026-10-13, with 25H2 (available 2025-09-30) ending 2027-10-12
and 26H1 (available 2026-02-10) ending 2028-03-14, so pinning to 24H2 today parks a user roughly
eleven weeks from end of servicing. The UI should surface the end-of-servicing date for whichever
version is pinned. Add Microsoft's licensing note attached to `ProductVersion`. And make the tweak
mutually exclusive with the new `defer_feature_updates`, because Microsoft states feature-update
deferrals are not in effect while a target version is specified.

**Ready-to-paste info block:**

````yaml
    info: |
      **Locks the PC to one Windows feature version, so it keeps getting monthly security patches but never jumps to a new release on its own.**

      ## What it does
      Sets `TargetReleaseVersion = 1` plus the version string (`TargetReleaseVersionInfo`) and the
      product string (`ProductVersion`) under the Windows Update policy key. Microsoft requires the
      product value alongside the version value for the pin to work.

      ## Benefits
      - **No surprise upgrades**: the annual feature update does not land until you move the pin
      - **Security patches continue**: monthly quality updates keep arriving for the pinned version
      - **The supported route**: this replaced the deprecated "defer feature updates by N days" control for version pinning

      ## Drawbacks
      - **Silent end of updates**: once the pinned release reaches end of servicing, the device stops receiving security updates entirely
      - **Does nothing on Home**: Windows Home does not honour Windows Update client policies
      - **Cancels feature deferrals**: Microsoft states feature update deferrals are not in effect while a target version is pinned, so do not set both
      - **Licensing intent**: Microsoft attaches a licensing attestation to using this policy to move a device to a new product

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 2004 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores this policy.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes all three values, restoring the normal upgrade path
      - **Check the servicing date**: Windows 11 24H2 Home and Pro end updates on 2026-10-13; 25H2 ends 2027-10-12 and 26H1 ends 2028-03-14

      ## Recommendation
      Good on Pro if you want to decide when to take a feature upgrade, provided you pin to a release
      with plenty of servicing left and set yourself a reminder. Do not pin a version that is within
      months of end of servicing, and do not use it if nobody will move the pin forward.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Update, TargetReleaseVersion and ProductVersion](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [Windows 11 release information](https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. Policy CSP - Update, `TargetReleaseVersion` and `ProductVersion`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; value
   forms, the requirement to set both, edition list, and the licensing language)
2. Windows 11 release information,
   https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information (tier A;
   availability and end-of-servicing dates for 24H2, 25H2 and 26H1)
3. Walkthrough: Use Group Policy to configure Windows Update client policies,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-wufb-group-policy (tier A; "When
   you specify target version policy, feature update deferrals won't be in effect")
4. What are Windows Update client policies?,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A;
   supported editions, Home excluded)

### `set_active_hours` Set active hours

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** three registry effects, all in `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings`,
all `REG_DWORD`.

| Value name | Manual 8:00 to 23:00 | Auto-adjust (labelled Stock Default) | **Real stock state** |
|---|---|---|---|
| `SmartActiveHoursState` | `2` | `1` | **value-absent** |
| `ActiveHoursStart` | `8` | `8` | whatever the user or OS already set |
| `ActiveHoursEnd` | `23` | `17` | whatever the user or OS already set |

No `requires_reboot` flag, `risk_level: low`.

`HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` is the internal store the Settings app writes when
a user sets active hours by hand. It is **not** a policy key. On the test machine that key held
`ActiveHoursStart = 16` and `ActiveHoursEnd = 10` with no `SmartActiveHoursState` value at all
**(measured)**, which confirms both that Windows reads active hours from here and that
`SmartActiveHoursState` is not seeded out of the box.

The supported route is the ADMX policy "Turn off auto-restart for updates during active hours", which
writes `ActiveHoursStart` and `ActiveHoursEnd` under
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`. Microsoft documents the policy range as 0 to
23 for both with defaults 8 and 17, states "By default, active hours are from 8 AM to 5 PM on PCs",
and caps the span at 18 hours (12 on Windows 10 1607 and Server 2016), so the YAML's 15-hour span is
legal. `SmartActiveHoursState` itself is undocumented by Microsoft; 1 meaning on and 2 meaning off
rests on Winaero alone, and other community sources give 0 and 1 instead.

**Corrections needed:** corrections 18, 19 and 20. The "Auto-adjust (Stock Default)" option is not a
default: it writes `SmartActiveHoursState = 1`, which does not exist on a stock machine, and
overwrites the user's real active hours with 8 and 17. On the test machine that would have destroyed
a deliberate 16-to-10 setting. The Stock Default must be `absent` for `SmartActiveHoursState` and the
hours must come from the snapshot. The tweak should ideally target the policy key. And the
`SmartActiveHoursState` semantics must be labelled community-sourced, because Microsoft documents
neither the value nor its range.

**Ready-to-paste info block:**

````yaml
    info: |
      **Tells Windows the hours you actually use the PC, so update restarts happen while you are away.**

      ## What it does
      Writes `ActiveHoursStart` and `ActiveHoursEnd` to the Windows Update settings store, and sets
      `SmartActiveHoursState = 2` so Windows stops auto-adjusting the window from your usage
      patterns. Automatic restarts are then scheduled outside the hours you chose.

      ## Benefits
      - **No mid-task reboots**: restarts move to times you are not at the machine
      - **Your window sticks**: turning Smart Active Hours off stops Windows revising the hours behind you
      - **Works on every edition**: this is the same store the consumer Settings UI writes, not a policy key

      ## Drawbacks
      - **Not a "never restart" switch**: Windows can still restart after an update deadline passes
      - **Capped at 18 hours**: you cannot cover a full day
      - **Overrides your existing hours**: applying replaces whatever start and end you had set
      - **`SmartActiveHoursState` is undocumented**: 2 meaning off comes from community sources, not from Microsoft

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, all editions
      - **Takes effect**: immediately
      - **Reverting**: restores your previous active hours from the snapshot and removes the Smart Active Hours value
      - **Interaction**: active hours has no effect if "No auto-restart with logged-on users" or "Always automatically restart at scheduled time" is enabled, so this and the auto-restart block partly cancel each other
      - On a managed machine the Windows Update policy key overrides this settings store

      ## Recommendation
      Worth setting for anyone annoyed by mistimed restarts. Set your genuine working window, and
      expect it to reduce badly timed reboots rather than eliminate them.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Manage device restarts after updates, Configure active hours](https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart)
      - [Policy CSP - Update, ActiveHoursStart / ActiveHoursEnd / ActiveHoursMaxRange](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [Automatically adjust active hours](https://winaero.com/automatically-adjust-active-hours-windows-10/)
````

**Sources:**
1. Manage device restarts after updates, "Configure active hours",
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A; the 8 AM to 5 PM
   default, the 18-hour cap, and the rule that active hours has no effect alongside the
   no-auto-restart policy)
2. Policy CSP - Update, `ActiveHoursStart`, `ActiveHoursEnd`, `ActiveHoursMaxRange`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; ranges
   and defaults for the policy form)
3. Enable Automatically Adjust Active Hours in Windows 10, Winaero,
   https://winaero.com/automatically-adjust-active-hours-windows-10/ (tier C; sole source for
   `SmartActiveHoursState` 1 = on and 2 = off, and contradicted by other community sources that give
   0 and 1)
4. Direct registry read of `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` on Windows 11 24H2
   build 26100 (primary measurement; `ActiveHoursStart = 16`, `ActiveHoursEnd = 10`, no
   `SmartActiveHoursState`)

### `disable_auto_restart_logged_on` Block auto-restart while signed in

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU` | `NoAutoRebootWithLoggedOnUsers` | `REG_DWORD` | Blocked | `1` |
| same | `NoAutoRebootWithLoggedOnUsers` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

Microsoft documents the value in the AU key restart table: "`0`: If users are signed in,
automatically restart... `1`: If a user is signed in, don't restart after an update installation."
Key, value name, type and polarity all match the YAML. Microsoft also states the gating condition
plainly: "This policy only applies when Configure Automatic Updates is set to option 4 - Auto
download and schedule the install." Since this tweak writes neither `NoAutoUpdate` nor `AUOptions`,
on a machine with no separate Configure Automatic Updates policy the value sits in the registry with
nothing to gate.

Editions: Pro, Education, Enterprise and IoT Enterprise. **Home does not honour the Windows Update AU
policies.** No reboot required.

**Corrections needed:** corrections 21 and 22. Add the companion values `NoAutoUpdate = 0` and
`AUOptions = 4`, or merge into the automatic-updates tweak (merge candidate 1); as authored the tweak
lets a user reach a state Microsoft documents as inert while the app reports it applied. Add
Microsoft's own caveats: the policy "doesn't work exactly as per description" in Group Policy and
"can result in no quality update reboots period, given many users never log off", with compliance
deadlines recommended instead; the policy "was never created as a CSP"; and over RDP only active
sessions count as signed-in users, so a machine with only disconnected sessions restarts anyway.
Change the Home wording from "honor it inconsistently" to "not supported".

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows Update rebooting the PC on its own while you are signed in, protecting open work.**

      ## What it does
      Sets `NoAutoRebootWithLoggedOnUsers = 1` under the Windows Update AU policy key, so scheduled
      update installs do not restart the machine while a user is signed in. Microsoft applies this
      only when automatic updates are configured to "auto download and schedule the install", so it
      must be paired with that setting to do anything.

      ## Benefits
      - **No reboot mid-task**: unsaved documents and long-running sessions survive update night
      - **You choose the moment**: Windows still notifies, it just does not act
      - **Policy-backed**: written to the policy branch, so a later Settings change cannot override it

      ## Drawbacks
      - **Inert on its own**: Microsoft applies it only when Configure Automatic Updates is set to option 4, so without that companion setting it does nothing
      - **Patches can stall indefinitely**: Microsoft warns it "can result in no quality update reboots period, given many users never log off"
      - **Does nothing on Home**: Windows Home does not honour the Windows Update AU policies
      - **Cancels active hours**: Microsoft states active hours has no effect while this policy is enabled

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, on Pro, Education, Enterprise and IoT Enterprise. **Windows Home does not honour it.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes the value, restoring Windows' normal restart behaviour
      - **Over RDP** only active sessions count as signed-in users, so a machine with only disconnected sessions still restarts
      - Microsoft recommends compliance deadlines instead of relying on this alone

      ## Recommendation
      Use it on Pro alongside scheduled automatic installs if you keep long work sessions open, and
      restart soon after updates so the fixes actually apply. Do not use it on Home, and do not use it
      on a machine that will simply never be restarted.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Manage device restarts after updates](https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart)
      - [Manage additional Windows Update settings](https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. Manage device restarts after updates, "Delay automatic restart" and the AU registry key table,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A; the 0 / 1
   semantics, the option-4 gating condition, the "no quality update reboots period" caveat, the
   never-a-CSP note, and the RDP session behaviour)
2. Manage additional Windows Update settings,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings (tier A; the AU key
   and its companion values)
3. What are Windows Update client policies?,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A;
   supported editions, Home excluded)

### `exclude_wu_driver_updates` Exclude driver updates from Windows Update

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `ExcludeWUDriversInQualityUpdate` | `REG_DWORD` | Excluded | `1` |
| same | `ExcludeWUDriversInQualityUpdate` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

Policy CSP gives the Group Policy mapping exactly: friendly name "Do not include drivers with Windows
Updates", registry key `Software\Policies\Microsoft\Windows\WindowsUpdate`, registry value name
`ExcludeWUDriversInQualityUpdate`, ADMX file `WindowsUpdate.admx`, allowed values 0 (default, allow
Windows Update drivers) and 1 (exclude). Key, value name, type, semantics and the `absent` default
all match the YAML.

Microsoft adds an important scope limit: "This policy won't apply to updates to drivers provided with
the operating system (which will be packaged within a security or critical update) or to feature
updates, where drivers might be dynamically installed to ensure the feature update process can
complete."

Note that the same value name also appears under `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings`
on real machines **(measured)**. That is the UX mirror, not the policy; the policy key is the one that
governs.

**Corrections needed:** correction 23. The copy's claim that "This works on Windows Home directly via
the registry key, with no Group Policy editor needed" is wrong. Windows Update client policies are
documented for Pro (including Pro for Workstations), Education and Enterprise (including Enterprise
LTSC, IoT Enterprise and IoT Enterprise LTSC), and the Policy CSP entry lists Pro, Enterprise,
Education and IoT Enterprise only. Replace it with a Pro-and-above gate. Also add Microsoft's scope
limit about OS-bundled drivers and feature-update drivers, so users do not expect it to block more
than it does.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows Update pushing hardware drivers, so you decide which driver versions your PC runs.**

      ## What it does
      Sets `ExcludeWUDriversInQualityUpdate = 1` under the Windows Update policy key, the registry
      form of "Do not include drivers with Windows Updates". Quality and security updates still
      install normally; only the driver payload is left out.

      ## Benefits
      - **No surprise driver swaps**: Windows Update stops replacing a working GPU, chipset or audio driver
      - **Vendor drivers win**: if you install NVIDIA, AMD, Intel or board-vendor packages, this ends the tug of war
      - **Narrow blast radius**: it changes driver delivery only, not patching

      ## Drawbacks
      - **You own driver updates now**: driver security and stability fixes shipped through Windows Update no longer arrive
      - **Does nothing on Home**: this is a Windows Update client policy and Windows Home does not honour it
      - **Not a total block**: Microsoft excludes drivers bundled inside a security or critical update, and drivers installed dynamically during a feature update

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores this policy.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes the value, restoring driver delivery through Windows Update
      - A same-named value under `WindowsUpdate\UX\Settings` is the Settings mirror, not the policy; the policy key governs

      ## Recommendation
      Good for anyone who manages drivers from vendor sites and has been bitten by an automatic driver
      swap. Leave it off if you would rather not think about drivers, and do not bother on Home where
      it has no effect.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Update, ExcludeWUDriversInQualityUpdate](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [Configure Windows Update client policies, exclude drivers](https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. Policy CSP - Update, `ExcludeWUDriversInQualityUpdate`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; key,
   value name, type, 0/1 semantics, edition list)
2. Configure Windows Update client policies, "Exclude drivers from quality updates",
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb (tier A; the scope
   limit on OS-bundled and feature-update drivers)
3. What are Windows Update client policies?,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A;
   supported editions, Home excluded)

### `disable_auto_driver_install` Disable automatic driver installation

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** two registry effects, both written to the **non-policy** UI stores.

| Key (as authored) | Value name | Type | Disabled | Windows default (Stock Default) |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching` | `SearchOrderConfig` | `REG_DWORD` | `0` | `1` |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Device Metadata` | `PreventDeviceMetadataFromNetwork` | `REG_DWORD` | `1` | `absent` |

**Corrected mechanism (the documented policy keys):**

| Key (correct) | Value name | Type | Disabled | Windows default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DriverSearching` | `SearchOrderConfig` | `REG_DWORD` | `0` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Device Metadata` | `PreventDeviceMetadataFromNetwork` | `REG_DWORD` | `1` | `absent` |

`requires_reboot: true`, `risk_level: medium`.

Both values exist and both are read, but both are being written to the Control Panel "Device
Installation Settings" store rather than to the documented policy keys. Microsoft documents
`PreventDeviceMetadataFromNetwork` at `HKLM\SOFTWARE\Policies\Microsoft\Windows\Device Metadata`, and
Policy CSP states the precedence: "This policy setting overrides the setting in the Device
Installation Settings dialog box." The ADMX policy "Specify search order for device driver source
locations" writes `SearchOrderConfig` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\DriverSearching`.

On the test machine `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching` held
`SearchOrderConfig = 1` and the `Policies\...\Device Metadata` key did not exist at all
**(measured)**, so the YAML's Stock Default of `1` for the UI path is right *for the UI path*. On the
policy path the correct stock default is `absent`, which is a different value and must not be
confused if the writes are moved.

`requires_reboot: true` is a reasonable over-approximation: the values are read on the next device
installation, so a reboot is not strictly needed but does no harm.

**Corrections needed:** corrections 24, 25 and 26. Move both writes to the documented policy keys, and
change the `SearchOrderConfig` stock default to `absent` when you do. Mark the `SearchOrderConfig = 0`
semantics on the non-policy path as unverified: no tier A or B source defines it and community
sources actively disagree about whether 0 is even meaningful there. Note in the copy that, as
authored, a user visiting Device Installation Settings in Control Panel can undo the tweak without
the app knowing, and any Group Policy on the machine silently overrides it.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows installing drivers by itself and fetching device details over the network when you plug in new hardware.**

      ## What it does
      Sets `SearchOrderConfig = 0` to stop the automatic driver search and
      `PreventDeviceMetadataFromNetwork = 1` to stop device metadata downloads (icons, names,
      details). New devices then wait for you to install their driver deliberately.

      ## Benefits
      - **You curate every driver**: nothing installs itself when hardware is attached
      - **Fewer outbound requests**: device metadata retrieval stops, a small privacy gain
      - **Pairs with driver exclusion**: together with excluding drivers from Windows Update it gives end-to-end control

      ## Drawbacks
      - **New hardware may not work**: peripherals sit undriven until you install software by hand
      - **Written to the Settings store, not the policy store**: opening Device Installation Settings in Control Panel can undo it without this app noticing, and a Group Policy on the machine silently overrides it
      - **`SearchOrderConfig = 0` is undocumented on this path**: Microsoft defines the value for the policy key only, and community sources disagree about what 0 means outside it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021. The Settings-store form is not edition-gated; the policy form is Pro and above.
      - **Takes effect**: after reboot
      - **Reverting**: restores `SearchOrderConfig` to 1 and removes the metadata value, matching the stock state of the Settings store
      - Keep vendor driver packages handy before adding new devices

      ## Recommendation
      For power users who want to approve every driver, ideally alongside excluding drivers from
      Windows Update. Skip it if you plug in new peripherals often and expect them to work
      immediately.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [Policy CSP - DeviceInstallation, PreventDeviceMetadataFromNetwork](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-deviceinstallation)
      - [Manage connections from Windows to Microsoft services, device metadata retrieval](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Specify search order for device driver source locations](https://www.windows-security.org/4e9792f8fcb59047c7a25891728d5f0c/specify-search-order-for-device-driver-source-locations)
````

**Sources:**
1. Policy CSP - DeviceInstallation, `PreventDeviceMetadataFromNetwork`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-deviceinstallation
   (tier A; the documented policy key and the statement that the policy overrides the Device
   Installation Settings dialog)
2. Manage connections from Windows to Microsoft services, section 4 "Device metadata retrieval",
   https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services
   (tier A; the policy path and the `REG_DWORD` type)
3. Specify search order for device driver source locations,
   https://www.windows-security.org/4e9792f8fcb59047c7a25891728d5f0c/specify-search-order-for-device-driver-source-locations
   (tier C; the ADMX policy's registry path under `Policies\...\DriverSearching`)
4. Direct registry read on Windows 11 24H2 build 26100 (primary measurement; `SearchOrderConfig = 1`
   present on the non-policy path, `Policies\...\Device Metadata` key absent)

### `disable_store_auto_updates` Disable Microsoft Store auto-updates

**Verdict:** VERIFIED

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsStore` | `AutoDownload` | `REG_DWORD` | Disabled | `2` |
| same | `AutoDownload` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

Microsoft's privacy configuration guidance says directly: "Create a new REG_DWORD registry setting
named **AutoDownload** in `HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\WindowsStore` with a value
of 2 (two)", as the registry equivalent of enabling the Group Policy "Turn off Automatic Download and
Install of updates" under Windows Components > Store. Policy CSP for `AllowAppStoreAutoUpdate`
confirms the mapping: ADMX name `DisableAutoInstall`, registry key
`Software\Policies\Microsoft\WindowsStore`, registry value name `AutoDownload`, ADMX file
`WindowsStore.admx`. The ADMX writes 2 when Enabled and 4 when Disabled, so the YAML's statement
about 4 is correct. Key, value name, type, value and the `absent` default all match.

Policy CSP lists Pro, Enterprise, Education and IoT Enterprise editions. Microsoft's privacy guide
presents the registry form with no edition caveat, so Home behaviour is probable but not documented.
No reboot required.

**Corrections needed:** `none`. Optionally add that Home edition support for this policy is not
documented by Microsoft, and note that recent Windows 11 Store builds have reduced the in-app "off"
control to a pause while the policy value continues to apply.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops the Microsoft Store updating your apps in the background, so app versions change only when you say so.**

      ## What it does
      Sets `AutoDownload = 2` under the Store policy key, the registry form of "Turn off Automatic
      Download and Install of updates". Store apps stay at their current versions until you update
      them from the Store's Library page.

      ## Benefits
      - **No background downloads**: useful on a metered or slow connection
      - **Version stability**: an app you rely on does not change shape overnight
      - **Survives the UI**: recent Store builds reduced the in-app toggle to a pause, but the policy value still applies

      ## Drawbacks
      - **Apps go stale**: including Store apps that ship security fixes
      - **Manual upkeep**: you have to remember to check the Library
      - **Home not documented**: Policy CSP lists Pro and above, so Home behaviour is likely but unconfirmed

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1507 and later including LTSC 2021. Policy CSP lists Pro, Enterprise, Education and IoT Enterprise; Microsoft's privacy guide gives the registry form with no edition caveat.
      - **Takes effect**: immediately
      - **Reverting**: deletes the value, restoring automatic Store updates
      - Setting the value to 4 rather than deleting it explicitly forces auto-updates on

      ## Recommendation
      Reasonable on a metered connection or where you want app versions pinned, provided you check the
      Store Library now and then. If you would forget, leave auto-updates on so security fixes keep
      arriving.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Manage connections from Windows to Microsoft services, section 26 Microsoft Store](https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services)
      - [Policy CSP - ApplicationManagement, AllowAppStoreAutoUpdate](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-applicationmanagement)
````

**Sources:**
1. Manage connections from Windows to Microsoft services, section 26 "Microsoft Store",
   https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services
   (tier A; gives the key, the `AutoDownload` name, `REG_DWORD` and the value 2 verbatim)
2. Policy CSP - ApplicationManagement, `AllowAppStoreAutoUpdate`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-applicationmanagement
   (tier A; the ADMX mapping and the edition list)
3. Turn off Automatic Download and Install of updates, ADMX Viewer,
   https://gpedit.tplant.com.au/en-us/policy/WindowsStore/DisableAutoInstall/ (tier C; the Enabled = 2
   and Disabled = 4 mapping)

### `block_update_over_metered` Block auto-download over metered

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `AllowAutoWindowsUpdateDownloadOverMeteredNetwork` | `REG_DWORD` | Blocked | `0` |
| same | same | `REG_DWORD` | Windows default (Stock Default) | `absent` |

No `requires_reboot` flag, `risk_level: medium`.

The shipped `WindowsUpdate.admx` on 26100.4061 declares
`<policy name="AllowAutoWindowsUpdateDownloadOverMeteredNetwork" class="Machine"
key="Software\Policies\Microsoft\Windows\WindowsUpdate"
valueName="AllowAutoWindowsUpdateDownloadOverMeteredNetwork">` with enabled = 1 and disabled = 0,
matching the YAML exactly. Policy CSP gives the friendly name "Allow updates to be downloaded
automatically over metered connections" and the allowed values 0 (Not allowed) and 1 (Allowed), and
states the default outright: `Default Value: 0`, `0 (Default) Not allowed`.

**What this means, stated accurately.** Because the documented default is already 0, writing 0 does
not change whether a stock machine downloads over a metered link. What it does change is where the
decision lives: in the policy branch it takes precedence over the per-machine "Download updates over
metered connections" toggle in Settings, so a later user, tool or image change cannot opt the machine
in. That is a real effect and the only effect. On 26100.4061,
`HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` carried no metered value at all and the policy
branch was unpopulated **(measured)**, consistent with both the documented default and the `absent`
stock default.

Editions: Pro, Enterprise, Education and IoT Enterprise / IoT Enterprise LTSC, Windows 10 1709 and
later. **Not Home.** No reboot required.

**Corrections needed:** correction 27. No registry defect: key, value name, `REG_DWORD`, polarity and
the `absent` stock default are all correct, the last because the policy branch is genuinely
unpopulated on a stock machine and `absent` is what "Not Configured" means. The correction is
entirely in the framing. The current copy ("You protect a limited or pay-per-gigabyte connection...
from having update downloads eat your data allowance without warning") implies the machine currently
does eat the allowance; it does not. Reframe as enforcing and locking the existing default, state
that priority and security updates can still come through, state that the connection must actually be
flagged metered, and state that Home does not honour it.

**Ready-to-paste info block:**

````yaml
    info: |
      **Locks "no automatic downloads over metered connections" into policy, so nothing can turn it back on.**

      ## What it does
      Sets `AllowAutoWindowsUpdateDownloadOverMeteredNetwork = 0` under the Windows Update policy key.
      Microsoft documents 0 as the shipped default, so this does not change today's behaviour: it
      moves the decision into the policy branch, where it overrides the Settings toggle.

      ## Benefits
      - **Cannot be opted in later**: the Settings toggle for metered downloads no longer takes effect
      - **Auditable posture**: the machine's intent is recorded in policy rather than left implicit
      - **Zero cost**: nothing is given up, because the machine already behaves this way

      ## Drawbacks
      - **No visible change today**: Windows already does not auto-download over metered links, so you will not observe a difference
      - **Does nothing on Home**: Windows Home does not honour Windows Update client policies
      - **Not a full block**: high-priority and critical updates can still download over a metered connection
      - **Needs a metered connection**: it does nothing until you actually mark a connection as metered in Settings

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores this policy.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes the value, returning the decision to the Settings toggle
      - Mark your hotspot or capped connection as metered in Settings for this to have anything to govern

      ## Recommendation
      Worth setting on a Pro machine that regularly runs on a hotspot or capped plan, as a lock rather
      than a change. Skip it on Home, where it is not honoured.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Update, AllowAutoWindowsUpdateDownloadOverMeteredNetwork](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. Policy CSP - Update, `AllowAutoWindowsUpdateDownloadOverMeteredNetwork`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; gives
   `Default Value: 0`, `0 (Default) Not allowed`, `1 Allowed`, the edition list and the Group Policy
   registry mapping)
2. Shipped ADMX, Windows 11 24H2 build 26100.4061,
   `C:\Windows\PolicyDefinitions\WindowsUpdate.admx`, policy
   `AllowAutoWindowsUpdateDownloadOverMeteredNetwork`, `class="Machine"`,
   `key="Software\Policies\Microsoft\Windows\WindowsUpdate"`, enabled 1 / disabled 0 (tier A)
3. What are Windows Update client policies?,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A;
   supported editions, Home excluded)
4. Direct registry inspection, Windows 11 24H2 build 26100.4061: no metered value under
   `HKLM\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` and no policy value under
   `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` (primary measurement)

### `disable_auto_update_download` Notify before downloading updates

**Verdict:** VERIFIED

**Mechanism:** two registry effects, both in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU`.

| Value name | Type | Notify before download and install | Windows default (Stock Default) |
|---|---|---|---|
| `NoAutoUpdate` | `REG_DWORD` | `0` | `absent` |
| `AUOptions` | `REG_DWORD` | `2` | `absent` |

No `requires_reboot` flag, `risk_level: high`, carries a `warning`.

Microsoft documents both values under exactly this key. `NoAutoUpdate`: "0: Automatic Updates is
enabled (default). 1: Automatic Updates is disabled." `AUOptions`: "1: Keep my computer up to date is
disabled... 2: Notify of download and installation. 3: Automatically download and notify of
installation. 4: Automatically download and scheduled installation. 5: Allow local admin to select
the configuration mode. This option isn't available for Windows 10 or later versions." The
combination the YAML writes (`NoAutoUpdate = 0` with `AUOptions = 2`) is exactly what the ADMX
"Configure Automatic Updates" policy produces when Enabled with option 2, and the YAML's explanation
that `NoAutoUpdate = 1` would make `AUOptions` moot is correct.

Editions: Pro, Education, Enterprise and IoT Enterprise. **Windows Home ignores the Windows Update AU
policy entirely**, which the YAML's `warning` field states correctly. No reboot required.

**Corrections needed:** `none`. Key, both value names, both types, both values and the `absent` Stock
Default are all correct, and the SKU warning is accurate. Three notes for the copy rather than
defects. (a) Microsoft words `AUOptions = 2` three different ways across its own pages: "Notify of
download and installation" in the registry table of "Manage additional Windows Update settings",
"Notify for download and notify for installation of updates" in the AU table on the restart page, and
"2 - Notify for download and auto install" in the Group Policy section of the same settings page. Use
Microsoft's registry-table wording verbatim rather than asserting one reading. (b) Once this is
applied, `disable_auto_restart_logged_on` becomes inert, because `NoAutoRebootWithLoggedOnUsers` only
applies at `AUOptions = 4`. (c) This is the highest-risk tweak in the file and the `risk_level: high`
plus warning are appropriate; keep both.

**Ready-to-paste info block:**

````yaml
    info: |
      **Puts Windows Update in notify-only mode, so nothing downloads or installs until you approve it.**

      ## What it does
      Sets `NoAutoUpdate = 0` with `AUOptions = 2` under the Windows Update AU policy key, which
      Microsoft's registry table describes as "Notify of download and installation". Automatic
      updates remain enabled as a subsystem; they just wait for your approval.

      ## Benefits
      - **Nothing lands unannounced**: no download and no install without an explicit decision
      - **Tight scheduling**: useful where updates must be applied in a controlled window
      - **Standard mechanism**: exactly what the "Configure Automatic Updates" Group Policy writes

      ## Drawbacks
      - **You will be unpatched between checks**: this is a real security exposure if you forget, which is why the risk is high
      - **Does nothing on Home**: Windows Home ignores the Windows Update AU policy entirely
      - **Cancels the reboot block**: `NoAutoRebootWithLoggedOnUsers` only applies at `AUOptions = 4`, so applying this makes that tweak inert
      - **Microsoft's own wording varies**: three Microsoft pages describe `AUOptions = 2` slightly differently

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, on Pro, Education, Enterprise and IoT Enterprise. **Windows Home ignores this policy.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes both values, restoring automatic updates
      - Setting `NoAutoUpdate = 1` instead would disable automatic updates entirely and make `AUOptions` irrelevant; this tweak deliberately does not do that

      ## Recommendation
      Only for disciplined Pro, Education or Enterprise users who will actually check for and apply
      updates promptly. If there is any chance patches will pile up, do not use it; the security cost
      outweighs the control.

      ## Evidence
      - **Risk**: high
      - **Confidence**: Microsoft-documented
      - [Manage additional Windows Update settings](https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings)
      - [Manage device restarts after updates, AU registry table](https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart)
      - [Policy CSP - Update, AllowAutoUpdate](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
````

**Sources:**
1. Manage additional Windows Update settings, "Configuring Automatic Updates by editing the
   registry", https://learn.microsoft.com/en-us/windows/deployment/update/waas-wu-settings (tier A;
   the AU key, `NoAutoUpdate` and `AUOptions` semantics)
2. Manage device restarts after updates, AU registry key table,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-restart (tier A; the second,
   differently worded description of `AUOptions = 2`)
3. Policy CSP - Update, `AllowAutoUpdate`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; the
   edition list, Home excluded)

### `disable_hibernation` Disable hibernation

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** an HKCU state marker plus an action.

State marker: `HKCU\Software\MagicXToolbox\State` `Hibernation` `REG_DWORD`, 1 when applied.

Action, `shell: powershell`, verbatim:

```powershell
# apply
powercfg /hibernate off

# undo
powercfg /hibernate on

# probe
$v = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Power' -Name HibernateEnabled -ErrorAction SilentlyContinue).HibernateEnabled
if ($v -eq 0) { exit 0 } else { exit 1 }
```

No `requires_reboot` flag, `risk_level: medium`.

`powercfg /hibernate off` is the correct and long-standing command. It clears hibernation support and
deletes `hiberfil.sys`. Because Fast Startup is implemented as a partial hibernation, turning
hibernation off also removes Fast Startup: on the test machine `powercfg /a` reported Hibernate
unavailable ("Hibernation has not been enabled"), Hybrid Sleep unavailable ("Hibernation is not
available") and Fast Startup unavailable ("Hibernation is not available") **(measured)**, confirming
the cascade. The probe target is correct: `HibernateEnabled` was 0 with `HibernateEnabledDefault`
still 1 on that machine **(measured)**.

All Windows 10 and Windows 11 versions and editions in scope. Requires administrator. No reboot
needed; the change is immediate.

**Corrections needed:** corrections 28 and 29. The `undo` runs `powercfg /hibernate on`
unconditionally. On a machine where hibernation was already off before the tweak (as on the test
machine, where `HibernateEnabled` was already 0), reverting *enables* hibernation, recreates a
multi-gigabyte `hiberfil.sys` and re-enables Fast Startup, none of which the user had. Snapshot
`HibernateEnabled` before apply and make the undo conditional. Also snapshot the hibernate file type
and size, because `powercfg /hibernate on` restores hibernation at full size and loses a prior
`/size 0` or `/type reduced` configuration.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off hibernation and deletes hiberfil.sys, reclaiming gigabytes of disk space.**

      ## What it does
      Runs `powercfg /hibernate off`, which clears hibernation support and removes the hidden
      `hiberfil.sys` file Windows reserves for it. Because Fast Startup is a partial hibernation,
      turning hibernation off also disables Fast Startup.

      ## Benefits
      - **Reclaims disk space**: `hiberfil.sys` is sized as a fraction of installed RAM, often several gigabytes
      - **Immediate**: the file is deleted on apply, no reboot needed
      - **No loss on a desktop that never hibernates**: dead space recovered for nothing given up

      ## Drawbacks
      - **No hibernate, no hybrid sleep, no Fast Startup**: all three depend on the same file
      - **Laptops lose the low-battery safety net**: hibernation is what saves your session to disk when the battery runs critically low
      - **Modern Standby machines drain flat**: hibernate-after-sleep is what stops an idle Modern Standby laptop discharging completely
      - **Reverting is currently blunt**: it enables hibernation unconditionally at full size, even on a machine that never had it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions. Requires administrator.
      - **Takes effect**: immediately
      - **Reverting**: re-enables hibernation from the snapshot; a machine that already had hibernation off before applying should stay that way
      - A prior `powercfg /hibernate /size` or `/type reduced` configuration is not preserved by a plain re-enable

      ## Recommendation
      Good on a desktop or a space-constrained SSD where you never hibernate. On a laptop, think
      twice: you give up the low-battery hibernate safety net, which is the one thing standing between
      a flat battery and a lost session.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [powercfg command-line options](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options)
      - [Sleep settings overview](https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings)
````

**Sources:**
1. powercfg command-line options,
   https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options
   (tier A; `/hibernate on|off`, `/hibernate /size` and `/hibernate /type reduced`)
2. Sleep settings overview,
   https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings (tier A;
   the hibernate idle timeout and hybrid sleep relationships)
3. `powercfg /a` and a read of `HKLM\SYSTEM\CurrentControlSet\Control\Power` on Windows 11 24H2 build
   26100 (primary measurement; `HibernateEnabled = 0`, `HibernateEnabledDefault = 1`, and the
   Hibernate / Hybrid Sleep / Fast Startup cascade)

### `disable_usb_selective_suspend` Disable USB selective suspend

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** an HKCU state marker plus a `powercfg` action.

State marker: `HKCU\Software\MagicXToolbox\State` `UsbSelectiveSuspend` `REG_DWORD`, 1 when applied.

Subgroup GUID `2a737441-1930-4402-8d77-b2bebba308a3` ("USB settings"), setting GUID
`48e6b7a6-50f5-4782-a5d4-53bb8f07e226` ("USB selective suspend setting"), indices 000 "Disabled" and
001 "Enabled".

Action, `shell: powershell`, verbatim:

```powershell
# apply
powercfg /SETACVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0
powercfg /SETDCVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0
powercfg /SETACTIVE SCHEME_CURRENT

# undo
powercfg /SETACVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 1
powercfg /SETDCVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 1
powercfg /SETACTIVE SCHEME_CURRENT

# probe
$q = powercfg /QUERY SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226
if ($q | Select-String 'Current AC Power Setting Index: 0x00000000') { exit 0 } else { exit 1 }
```

No `requires_reboot` flag, `risk_level: medium`.

Both GUIDs are correct. Queried live, the subgroup resolves to "USB settings" and the setting to "USB
selective suspend setting", with possible indices 000 "Disabled" and 001 "Enabled" **(measured)**. On
stock `SCHEME_BALANCED` the setting is AC = 1 and DC = 1 **(measured)**, so the `undo` value of 1
happens to match the factory default. `SCHEME_CURRENT` is accepted by `powercfg` even though it does
not appear in `powercfg /aliases`; `powercfg /SETACTIVE SCHEME_CURRENT` returned exit code 0
**(measured)**. All Windows 10 and Windows 11 versions and editions in scope, requires administrator,
and `SETACTIVE` applies the change immediately so no reboot is needed.

**Corrections needed:** corrections 30, 31 and 32. The `undo` hardcodes index 1 for AC and DC rather
than restoring the snapshotted indices, so a user who had already disabled selective suspend, or an
OEM scheme with a different value, gets overwritten. Both this tweak and `disable_wake_timers` operate
on `SCHEME_CURRENT`, so switching power plan between apply and undo writes the undo to a different
scheme and leaves the original one modified; snapshot the scheme GUID at apply time or apply to all
schemes. And both probes test only `Current AC Power Setting Index`, so a machine with AC applied and
DC reverted reports "applied".

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops Windows parking idle USB ports, fixing device dropouts, audio glitches and slow wakeups.**

      ## What it does
      Sets the "USB selective suspend setting" power-plan value to Disabled for both AC and battery on
      the current power scheme, then reactivates the scheme so the change applies. Ports stay powered
      instead of being suspended when idle.

      ## Benefits
      - **Fixes USB audio dropouts**: the standard cure for USB DACs and interfaces that glitch after idle
      - **Stops dongle disconnects**: wireless receivers stay awake and responsive
      - **No slow-to-wake hubs**: devices behind a hub respond immediately instead of after a stall

      ## Drawbacks
      - **Higher idle power**: a small but real battery cost on laptops
      - **Only the active power plan**: switching to another plan silently loses the change
      - **No benefit without symptoms**: if your USB devices behave, this only costs power

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions. Requires administrator.
      - **Takes effect**: immediately, the power scheme is reactivated on apply
      - **Reverting**: restores the previous AC and battery indices from the snapshot
      - Stock Balanced has selective suspend enabled on both AC and battery

      ## Recommendation
      Apply it if you actually experience USB dropouts, audio glitches or slow wakeups. If your USB
      devices behave fine, especially on a laptop, leave selective suspend on and keep the power
      saving.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [USB selective suspend](https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-selective-suspend)
      - [powercfg command-line options](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options)
````

**Sources:**
1. USB selective suspend,
   https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-selective-suspend (tier A;
   what the feature does and why it causes device stalls)
2. powercfg command-line options,
   https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options
   (tier A; `/SETACVALUEINDEX`, `/SETDCVALUEINDEX`, `/SETACTIVE` and `/QUERY` syntax)
3. `powercfg /QUERY SCHEME_CURRENT` and `powercfg /QUERY 381b4222-f694-41f0-9685-ff5bb260df2e` on
   Windows 11 24H2 build 26100 (primary measurement; subgroup and setting names, indices 000
   "Disabled" and 001 "Enabled", stock Balanced AC = 1 and DC = 1)

### `disable_wake_timers` Disable wake timers

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** an HKCU state marker plus a `powercfg` action.

State marker: `HKCU\Software\MagicXToolbox\State` `WakeTimers` `REG_DWORD`, 1 when applied.

`SUB_SLEEP` is the Sleep settings subgroup, GUID `238c9fa8-0aad-41ed-83f4-97be242c8f20`. `RTCWAKE` is
the PowerCfg alias for `bd3b718a-0680-4d9d-8ab2-e1d2b4ac806d`, which Microsoft names "Automatically
wake for tasks". The setting has **three** indices, not two: 000 "Disable", 001 "Enable", 002
"Important Wake Timers Only" **(measured)**.

Action, `shell: powershell`, verbatim:

```powershell
# apply
powercfg /SETACVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 0
powercfg /SETDCVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 0
powercfg /SETACTIVE SCHEME_CURRENT

# undo
powercfg /SETACVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 1
powercfg /SETDCVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 1
powercfg /SETACTIVE SCHEME_CURRENT

# probe
$q = powercfg /QUERY SCHEME_CURRENT SUB_SLEEP RTCWAKE
if ($q | Select-String 'Current AC Power Setting Index: 0x00000000') { exit 0 } else { exit 1 }
```

No `requires_reboot` flag, `risk_level: low`.

**The corrected undo.** On stock `SCHEME_BALANCED` under Windows 11 24H2 the setting is
**AC = 2 ("Important Wake Timers Only") and DC = 0 ("Disable")** **(measured)**. The `undo` above
writes **1 ("Enable") to both**, which is more permissive than the factory default on **both** power
sources. The correct undo restores the snapshotted indices, or failing that:

```powershell
# corrected undo (minimum acceptable form, Windows 11 stock Balanced values)
powercfg /SETACVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 2
powercfg /SETDCVALUEINDEX SCHEME_CURRENT SUB_SLEEP RTCWAKE 0
powercfg /SETACTIVE SCHEME_CURRENT
```

Microsoft describes the setting as: "Specifies whether the system uses the system-wide
wake-on-timer capability. The system can automatically use wake-on-timer on capable hardware to
perform scheduled tasks. For example, the system might wake automatically to install updates." All
Windows 10 and Windows 11 versions and editions in scope, on hardware with RTC wake, requires
administrator, no reboot needed.

**Corrections needed:** corrections 31, 32, 33 and 34. The `undo` writing 1 to both rails is the most
consequential revert defect in this file: a user who applies and then reverts ends up with a machine
that wakes for ordinary scheduled tasks on battery, which it never did before, and nothing in the
interface tells them. Fix the undo first. Then: the probe must check both AC and DC, the scheme-drift
issue applies here as it does to USB selective suspend, and the copy must stop implying that Windows
freely wakes for maintenance on a stock machine. Because DC is already 0 by default, the apply's DC
half changes nothing on a stock machine; the only real change is AC from 2 ("Important Wake Timers
Only") to 0 ("Disable"), and "Important Wake Timers Only" already excludes ordinary scheduled tasks.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops scheduled tasks waking your PC out of sleep, ending mystery overnight wakeups.**

      ## What it does
      Sets the "Automatically wake for tasks" power-plan value to Disable for both AC and battery on
      the current scheme. Windows stops using the system-wide wake-on-timer capability, so scheduled
      jobs can no longer rouse a sleeping machine.

      ## Benefits
      - **The PC stays asleep**: no 3 a.m. wake for maintenance, and no fans spinning up overnight
      - **No battery drain from wakes**: an unattended laptop is not woken to do work
      - **Tightens the mains behaviour**: on AC, Windows normally allows "important" wake timers, and this removes those too

      ## Drawbacks
      - **Legitimate scheduled wakes stop**: overnight backups, media indexing and alarms no longer wake the machine
      - **Update maintenance shifts**: Windows Update work may only run while you are actively using the PC
      - **Less change than it sounds on battery**: Windows 11 already disables wake timers on battery by default, so only the mains behaviour actually changes
      - **Only the active power plan**: switching to another plan silently loses the change

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions, on hardware with RTC wake. Requires administrator.
      - **Takes effect**: immediately, the power scheme is reactivated on apply
      - **Reverting**: restores the previous AC and battery indices from the snapshot. Stock Windows 11 Balanced is "Important Wake Timers Only" on AC and "Disable" on battery, not "Enable" on both.
      - The setting has three states, not two: Disable, Enable, and Important Wake Timers Only

      ## Recommendation
      Worth applying if a PC that wakes itself overnight bothers you. Skip it if you rely on scheduled
      overnight work such as automated backups, because those wakes stop too.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Automatically wake for tasks](https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings-automatically-wake-for-tasks)
      - [Sleep settings overview](https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings)
      - [powercfg command-line options](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options)
````

**Sources:**
1. Automatically wake for tasks,
   https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings-automatically-wake-for-tasks
   (tier A; the setting GUID, the `RTCWAKE` alias, and the description of what wake-on-timer does)
2. Sleep settings overview,
   https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings (tier A;
   the `SUB_SLEEP` subgroup GUID and alias)
3. powercfg command-line options,
   https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options
   (tier A; the index-setting syntax)
4. `powercfg /aliases` and `powercfg /QUERY 381b4222-f694-41f0-9685-ff5bb260df2e SUB_SLEEP RTCWAKE` on
   Windows 11 24H2 build 26100 (primary measurement; three indices 000 / 001 / 002, and stock Balanced
   AC = 2 with DC = 0)

### `disable_modern_standby` Disable Modern Standby (force S3)

**Verdict:** VERIFIED (community-corroborated, not Microsoft-documented)

**Mechanism:** one registry effect.

| Key | Value name | Type | Option | Value |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Power` | `PlatformAoAcOverride` | `REG_DWORD` | Force S3 | `0` |
| same | `PlatformAoAcOverride` | `REG_DWORD` | Windows default (Stock Default) | `absent` |

`requires_reboot: true`, `risk_level: high`, carries a `warning`.

Modern Standby (S0 Low Power Idle) and S3 are both documented by Microsoft, including the platform
requirements and the fact that a system supports one or the other, not both at runtime. Microsoft does
not document `PlatformAoAcOverride`, but the value is unambiguously real and read by the kernel: a
sweep of 5,418 binaries under `System32` on build 26100.4061 finds the UTF-16 string
`PlatformAoAcOverride` in exactly two modules, `ntoskrnl.exe` and its LA57 variant `ntkrla57.exe`, and
in no driver or user-mode DLL. The kernel power manager is where the S0ix versus S3 decision is made,
so the component and the name are both right. Four or more independent tier C references from
different authors and years agree on the key path, the value name, `REG_DWORD` and that 0 turns Modern
Standby off, with consistent reported behaviour: on firmware that still exposes S3, setting 0 makes
Windows select S3 (verifiable with `powercfg /a`) and re-expose the Power Management tab in Device
Manager. Under the corroboration rule that is VERIFIED, labelled community-corroborated.

What it does **not** do is create an S3 path where the firmware has none. If `powercfg /a` reports
"Standby (S3): The system firmware does not support this standby state", disabling Modern Standby
leaves the machine with no S0-class and no S3-class sleep, that is only hibernate or shutdown, and lid
close can behave badly.

Applicability is firmware, not Windows version: Windows 10 and all Windows 11 builds in scope
including 26100 and 26200, all SKUs, provided the platform still exposes S3. Microsoft's own guidance
is a caution here rather than a blessing: "Switching between S3 and Modern Standby cannot be done by
changing a setting in the BIOS. Switching the power model is not supported in Windows without a
complete OS re-install." The test machine reports `Standby (S3)` available and `Standby (S0 Low Power
Idle)` unavailable with `PlatformAoAcOverride` absent **(measured)**, consistent with the `absent`
stock default; being an S3 platform it cannot demonstrate the S0-to-S3 switch itself.

**Corrections needed:** correction 35. No registry defect: key, value name, `REG_DWORD`, the value 0,
the `absent` stock default and `requires_reboot: true` are all correct, and `risk_level: high` plus
the `powercfg /a` precondition are the right safety framing. Two changes. (1) Say the override is
community-corroborated and confirmed present in the 26100 kernel but not Microsoft-documented, and
carry Microsoft's statement that switching the power model is not supported without a complete OS
re-install, so users understand this is working-but-unsupported. (2) Make the `powercfg /a`
precondition enforceable: the app should read `powercfg /a` and refuse or hard-warn when "Standby
(S3)" is not available, because a text warning is the wrong protection for an outcome that can leave a
laptop unable to sleep. With that gate the tweak is safe to expose; without it, exposing it to a user
on an S0ix-only laptop is the failure case.

**Ready-to-paste info block:**

````yaml
    info: |
      **Forces old-style S3 deep sleep instead of Modern Standby, stopping in-bag battery drain, on hardware that still supports S3.**

      ## What it does
      Sets `PlatformAoAcOverride = 0` under the kernel power key, which tells Windows not to use the
      always-on always-connected power model. On a platform whose firmware still exposes S3, Windows
      then selects S3 instead of S0 Low Power Idle.

      ## Benefits
      - **Real deep sleep**: no background work while "asleep", so no warm laptop and no drained battery in a bag
      - **Verifiable**: `powercfg /a` shows the change afterwards
      - **Fully reversible**: deleting the value restores Modern Standby on the next boot

      ## Drawbacks
      - **Catastrophic on hardware without S3**: if `powercfg /a` does not list "Standby (S3)" as available, you can end up with no working sleep state at all
      - **No instant-on or background connectivity**: S3 gives up the things Modern Standby exists to provide
      - **Undocumented and unsupported**: Microsoft does not document this value and states that switching the power model "is not supported in Windows without a complete OS re-install"
      - **Not always better**: Microsoft's own guidance is to measure idle power first, because it can be worse rather than better

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions. The real gate is firmware, not Windows version.
      - **Takes effect**: after reboot
      - **Reverting**: deletes the value, restoring Modern Standby
      - **Run `powercfg /a` first.** If "Standby (S3)" is not listed as available, do not apply this.
      - The value is confirmed present in the Windows 11 24H2 kernel and corroborated by several independent references, but Microsoft has never documented it

      ## Recommendation
      Only for users who have confirmed with `powercfg /a` that Standby (S3) is available and who are
      actually fighting Modern Standby drain. If S3 is not listed, do not touch this; you risk a
      machine that cannot sleep or wake properly.

      ## Evidence
      - **Risk**: high
      - **Confidence**: Community-corroborated
      - [What is Modern Standby](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby)
      - [Modern Standby vs S3](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby-vs-s3)
      - [How to disable Modern Standby on Windows](https://pureinfotech.com/disable-modern-standby-windows/)
      - [Disable Modern Standby in Windows 10 and Windows 11](https://www.elevenforum.com/t/disable-modern-standby-in-windows-10-and-windows-11.3929/)
````

**Sources:**
1. What is Modern Standby,
   https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby (tier A;
   "Switching between S3 and Modern Standby cannot be done by changing a setting in the BIOS.
   Switching the power model is not supported in Windows without a complete OS re-install.")
2. Modern Standby vs S3,
   https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby-vs-s3
   (tier A; what S0ix and S3 are, and that a platform supports one)
3. Direct binary inspection, Windows 11 24H2 build 26100.4061: `PlatformAoAcOverride` present as a
   UTF-16 string in `ntoskrnl.exe` and `ntkrla57.exe` only, out of 5,418 `System32` binaries scanned;
   value absent under `HKLM\SYSTEM\CurrentControlSet\Control\Power`; `powercfg /a` reports S3
   available and S0 Low Power Idle unavailable (primary measurement)
4. How to Disable Modern Standby (S0) on Windows, Pureinfotech,
   https://pureinfotech.com/disable-modern-standby-windows/ (tier C; key, value name, `REG_DWORD`,
   value 0, and the `powercfg /a` verification step)
5. Disable Modern Standby in Windows 10 and Windows 11, elevenforum tutorial,
   https://www.elevenforum.com/t/disable-modern-standby-in-windows-10-and-windows-11.3929/ (tier C;
   maintained long-lived reference, same key, name, type and value; corroborated further by MakeUseOf
   and WinBuzzer as third and fourth independent origins)

### `disable_mdns` Disable mDNS (new)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (CORRECTED):** two registry effects, both written.

| Key | Value name | Type | Disabled | Windows default (Stock Default) |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `EnableMDNS` | `REG_DWORD` | `0` | `absent` |
| `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters` | `EnableMDNS` | `REG_DWORD` | `0` | `absent` |

Proposed `requires_reboot: true`, proposed `risk_level: medium`. **No build gate.**

**Mechanism as proposed (WRONG, shown for contrast):** the policy value only, with the
`Dnscache\Parameters` value described as "an optional second effect", and an applicability gate of
`windows: { build: ">=26100" }` with the instruction "do not claim it for LTSC 2021".

The shipped 26100 `DnsClient.admx` declares:

```xml
<policy name="DNS_MDNS" class="Machine"
        key="Software\Policies\Microsoft\Windows NT\DNSClient"
        valueName="EnableMDNS">
  <parentCategory ref="DNS_Client" />
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_RS2" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

ADML display name: "Configure multicast DNS (mDNS) protocol". UTF-16 string extraction from the
shipped 26100 `dnsrslvr.dll` returns `EnableMDNS`, `SOFTWARE\Policies\Microsoft\Windows NT\DNSClient`,
`Software\Policies\Microsoft\Windows NT\DnsClient` and
`SYSTEM\CurrentControlSet\Services\Dnscache\Parameters`; `dnsapi.dll` also carries `EnableMDNS`. So
the resolver reads the value name from the policy path **and** carries the service base path.

Both values must be written because the shipped `DNS_MDNS_Help` string reads: "Specifies if the DNS
client will perform name resolution over mDNS. If you enable this policy, the DNS client will use mDNS
protocol. If you disable this policy setting, **or if you do not configure this policy setting, the
DNS client will use locally configured settings.**" Read literally, the ADMX `disabledValue` of 0
returns the client to local settings rather than switching mDNS off unconditionally. That is most
likely sloppy ADML boilerplate, but it is the only first-party statement of what 0 does, and the
service-key value is exactly the "locally configured setting" it defers to.

This is genuinely the third leg of the multicast name-resolution triad and not a duplicate:
`disable_llmnr` writes `EnableMulticast` (same policy key, different value) and
`disable_netbios_tcpip` writes per-adapter `NetbiosOptions`.

**Corrections needed:** corrections 36 and 37. Drop the `>=26100` build gate: the ADMX declares
`SUPPORTED_Windows_10_0_RS2`, "At least Windows 10", and LTSC 2021 (build 19044) is well above RS2,
so there is no tier A basis for a 26100 floor. Ship ungated and say in the copy that it was verified
on 26100 and is expected but not verified on 19044. And write both values rather than one, reverting
both. Risk is accurate as proposed and is the highest of the three name-resolution tweaks, because
mDNS is what makes `.local` names, AirPrint and IPP Everywhere printers, Chromecast and Google Cast
discovery, and Apple device discovery work.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off mDNS, the third broadcast name-resolution protocol, closing the last leg of the LLMNR and NetBIOS spoofing triad.**

      ## What it does
      Sets `EnableMDNS = 0` both under the DNS Client policy key and under the `Dnscache\Parameters`
      service key, which is the "locally configured setting" the policy's own help text defers to.
      The DNS client then stops answering and issuing multicast DNS queries.

      ## Benefits
      - **Closes mDNS poisoning**: the same Responder-class attack that motivates disabling LLMNR works over mDNS
      - **Completes the set**: with LLMNR and NetBIOS already off, this is the remaining broadcast name-resolution protocol
      - **First-party control**: Windows ships a Group Policy for it, "Configure multicast DNS (mDNS) protocol"
      - **Fully reversible**: deleting both values restores the default

      ## Drawbacks
      - **`.local` names stop resolving**: anything you reach by a `.local` hostname becomes unreachable by name
      - **Network printers vanish**: AirPrint and IPP Everywhere printers are discovered over mDNS
      - **Casting and smart-home discovery break**: Chromecast, Google Cast, Apple device discovery and some smart-home apps rely on it
      - **Highest cost of the three**: this is the name-resolution tweak users are most likely to notice

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer. The policy declares "At least Windows 10", so Windows 10 including LTSC 2021 should honour it, but that was verified on 26100 only.
      - **Takes effect**: after reboot, or after restarting the DNS Client (`Dnscache`) service
      - **Reverting**: deletes both values, restoring mDNS
      - Apply this after LLMNR and NetBIOS, not before; it is the one most likely to be noticed

      ## Recommendation
      Worth applying on a hardened workstation with no `.local` dependencies and no mDNS-discovered
      printers or cast devices. If you print to a network printer found by name, or cast to a TV,
      expect that to stop and skip this one.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP - ADMX_DnsClient](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient)
      - [DNS Client Group Policy settings](https://learn.microsoft.com/en-us/windows-server/networking/dns/dns-top)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\DnsClient.admx` and `en-US\DnsClient.adml` on build 26100, policy
   `DNS_MDNS`, verbatim: `class="Machine"`, `key="Software\Policies\Microsoft\Windows NT\DNSClient"`,
   `valueName="EnableMDNS"`, `supportedOn ref="windows:SUPPORTED_Windows_10_0_RS2"`, `enabledValue`
   1, `disabledValue` 0, and the `DNS_MDNS_Help` string (tier A, shipped ADMX)
2. UTF-16 string extraction from the shipped 26100 `dnsrslvr.dll` and `dnsapi.dll`: `EnableMDNS`
   present alongside both the policy path and `SYSTEM\CurrentControlSet\Services\Dnscache\Parameters`
   (tier A, shipped binary)
3. Policy CSP - ADMX_DnsClient,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient (tier A;
   the DNS Client policy surface and its edition scope)
4. CIS Windows 11 v4.0.0, mDNS recommendation; corroborated independently by WinUtil and privacy.sexy
   (tier B and tier C; agreement on the key, value name and value 0)

### `update_feature_control` Control mid-cycle features and optional content (new)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (CORRECTED):** three registry effects, all in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, all `REG_DWORD`.

| Value name | Stability option | Permissive option | Windows default (Stock Default) |
|---|---|---|---|
| `AllowTemporaryEnterpriseFeatureControl` | `0` | `1` | `absent` |
| `SetAllowOptionalContent` | `0` | `1` | `absent` |
| `AllowOptionalContent` | `absent` | `1`, `2` or `3` | `absent` |

`AllowOptionalContent` semantics per Policy CSP: **0 (Default)** don't receive optional updates;
**1** automatically receive optional updates **including gradual feature rollouts (CFRs)**; **2**
automatically receive optional updates (optional cumulative updates only); **3** users can select
which optional updates to receive. Note that **1 is more permissive than 2**.

No `requires_reboot`, proposed `risk_level: low`.

**Mechanism as proposed (WRONG, shown for contrast):**

> `SetAllowOptionalContent` | `0` (no optional content) | `1` automatically receive optional updates,
> `2` also get the latest optional non-security preview

That treats `SetAllowOptionalContent` as the selector. The shipped 26100 `WindowsUpdate.admx` shows
the GP policy `AllowOptionalContent` writes `SetAllowOptionalContent` as `enabledValue` 1 /
`disabledValue` 0 and puts the selection in a separate `AllowOptionalContent` enum element:

```xml
<policy name="AllowOptionalContent" class="Machine"
        key="Software\Policies\Microsoft\Windows\WindowsUpdate"
        valueName="SetAllowOptionalContent">
  <supportedOn ref="WU_SUPPORTED_WinServer2025_Win1021H2_Win1122H2" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
  <elements>
    <enum id="AllowOptionalContent" valueName="AllowOptionalContent" required="true">
      <item displayName="$(string.ReceiveOptionalContentWithAutoApproval)"><value><decimal value="1"/></value></item>
      <item displayName="$(string.ReceiveOptionalLCUWithAutoApproval)"><value><decimal value="2"/></value></item>
      <item displayName="$(string.ReceiveOptionalContentNeedUserApproval)"><value><decimal value="3"/></value></item>
    </enum>
  </elements>
</policy>
```

`AllowTemporaryEnterpriseFeatureControl` is confirmed as proposed: `REG_DWORD`, `enabledValue` 1,
`disabledValue` 0, `supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"`. The ADML help string:
"Features introduced via servicing (outside of the annual feature update) are off by default for
devices that have their Windows updates managed. If this policy is configured to 'Enabled', then all
features available in the latest monthly quality update installed will be on. If this policy is set to
'Not Configured' or 'Disabled' then features that are shipped via a monthly quality update (servicing)
will remain off until the feature update that includes these features is installed. *Windows update
managed devices are those that have their Windows updates managed via policy; whether via the cloud
using Windows Update for Business or on-premises with Windows Server Update Services (WSUS)."

That last sentence is the honest caveat: the gating applies **only** to update-managed devices. On an
unmanaged consumer PC with no other Windows Update policy, writing
`AllowTemporaryEnterpriseFeatureControl = 0` may change nothing. Setting `DeferQualityUpdates` or
`TargetReleaseVersion` does make the device managed. Whether writing this value alone is itself enough
to make the device "policy managed" is not stated by any tier A source and is recorded as unresolved.

**Applicability, two gates.** `AllowTemporaryEnterpriseFeatureControl`: Windows 11 22H2 and later, so
**not LTSC 2021**. `SetAllowOptionalContent` and `AllowOptionalContent`:
`WU_SUPPORTED_WinServer2025_Win1021H2_Win1122H2`, rendered as "At least Windows Server 2025,
Windows 10 Version 21H2, or Windows 11 Version 22H2", with Policy CSP giving Windows 10 21H2
(10.0.19044.3757) and later, so a patched LTSC 2021 qualifies for that half. Editions for both per
Policy CSP: Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC. **Not Home.**

**Corrections needed:** corrections 38, 39 and 40. Move the selection to `AllowOptionalContent` and
treat `SetAllowOptionalContent` as a 1/0 enable flag only. Correct the value ordering: 1 is more
permissive than 2, not less. Carry two applicability gates rather than one, so the tweak does not
promise `AllowTemporaryEnterpriseFeatureControl` on LTSC 2021 where it does not exist. For the
stability option write `SetAllowOptionalContent = 0` and leave `AllowOptionalContent` absent; for
revert delete all three.

**Ready-to-paste info block:**

````yaml
    info: |
      **Stops new features and optional content arriving inside monthly updates, so the installed Windows stops changing shape between feature updates.**

      ## What it does
      Writes `AllowTemporaryEnterpriseFeatureControl = 0` so features shipped inside a monthly quality
      update stay off until the feature update that includes them, and `SetAllowOptionalContent = 0`
      so optional non-security preview updates and optional driver updates are not offered.

      ## Benefits
      - **A stable install**: the machine does not gain new UI or behaviour mid-cycle
      - **No optional previews**: optional non-security preview updates stop being offered
      - **Nothing breaks**: features simply arrive later, with the annual feature update
      - **Genuine second state**: users who want features early can pick the permissive option instead

      ## Drawbacks
      - **May do nothing on an unmanaged PC**: Microsoft applies temporary enterprise feature control only to devices whose updates are policy-managed, so on a machine with no other Windows Update policy this half may be inert
      - **Does nothing on Home**: Windows Home does not honour Windows Update client policies
      - **You wait for fixes delivered as features**: improvements shipped through servicing arrive with the next feature update instead
      - **Split applicability**: the temporary-feature-control half needs Windows 11 22H2 or newer, so it does not exist on Windows 10 LTSC 2021

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores these policies.** The optional-content half also applies to Windows 10 21H2 and later, including a patched LTSC 2021; the temporary-feature-control half does not.
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes all three values, restoring Windows' normal behaviour
      - Setting a quality-update deferral or a target release version makes the device update-managed, which is what activates the temporary feature control half

      ## Recommendation
      Worth applying on a Pro machine you want to stop changing shape month to month, particularly
      alongside a quality-update deferral, which is what makes the device update-managed. Skip it if
      you like getting new features as soon as they ship, and skip it entirely on Home.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Update, AllowOptionalContent](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [Configure Windows Update client policies](https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` and `en-US\WindowsUpdate.adml` on build 26100,
   policies `AllowTemporaryEnterpriseFeatureControl` and `AllowOptionalContent`, verbatim, including
   the enum element and both `supportedOn` references (tier A, shipped ADMX)
2. Policy CSP - Update, `AllowOptionalContent` allowed values and edition list,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; the
   0 / 1 / 2 / 3 semantics and the Windows 10 21H2 build floor)
3. Configure Windows Update client policies,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb (tier A; what
   temporary enterprise feature control is and what "update-managed" means)

### `defer_feature_updates` Defer feature updates (new)

**Verdict:** VERIFIED

**Mechanism:** two registry effects, both in
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, both `REG_DWORD`.

| Value name | 30 days | 180 days | 365 days | Windows default (Stock Default) |
|---|---|---|---|---|
| `DeferFeatureUpdates` | `1` | `1` | `1` | `absent` |
| `DeferFeatureUpdatesPeriodInDays` | `30` | `180` | `365` | `absent` |

No `requires_reboot`, proposed `risk_level: low`. Four options means the corpus renders it as a
dropdown, consistent with `defer_quality_updates`.

Shipped 26100 `WindowsUpdate.admx`:

```xml
<policy name="DeferFeatureUpdates" class="Machine"
        key="Software\Policies\Microsoft\Windows\WindowsUpdate"
        valueName="DeferFeatureUpdates">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_NOARM" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
  <elements>
    <decimal id="DeferFeatureUpdatesPeriodId" valueName="DeferFeatureUpdatesPeriodInDays"
             minValue="0" maxValue="365" />
    <text id="PauseFeatureUpdatesStartId" valueName="PauseFeatureUpdatesStartTime" maxLength="10" />
  </elements>
</policy>
```

Both value names, both types and the 0 to 365 range match. Policy CSP
`DeferFeatureUpdatesPeriodInDays` gives Windows 10 1607 (10.0.14393) and later, editions Pro,
Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC, range 0 to 365, default 0. **Not
Home.** A deprecation was specifically looked for, because deferral policies are the kind of thing
Microsoft has been retiring: none found. The policy is present and undecorated in the 26100 ADMX, the
Policy CSP page carries no deprecation banner, and the Windows Update client policies overview still
lists feature updates with a 365-day maximum deferral.

Not a duplicate. `defer_quality_updates` writes different value names with a 30-day maximum;
`target_release_version` writes three different values and pins rather than slides.

**Corrections needed:** correction 41. None to key, value name, type or range. The correction is to
the interaction. Microsoft states "When you specify target version policy, feature update deferrals
won't be in effect", so if `target_release_version` is applied this tweak is **fully inert**, not
merely overridden in some soft sense. Treat the two as mutually exclusive in the UI (see merge
candidate 4) rather than shipping them as independent tweaks with a cross-reference; otherwise a user
sets a deferral, the app reports it applied, and no deferral happens. Two mechanical notes: the same
ADMX policy also owns `PauseFeatureUpdatesStartTime` (`REG_SZ`), which this tweak must not write and
whose absence the revert must not assume, because the user may have paused updates from the Settings
UI; and `BranchReadinessLevel` is **not** part of this policy on 26100 (it belongs to
`ManagePreviewBuilds`), so it must not be added.

**Ready-to-paste info block:**

````yaml
    info: |
      **Slides every annual Windows feature update back by a number of days you choose, so other people find the bugs first.**

      ## What it does
      Sets `DeferFeatureUpdates = 1` and `DeferFeatureUpdatesPeriodInDays` to 30, 180 or 365 under the
      Windows Update policy key. Feature updates still arrive, just later. Monthly security updates
      are unaffected.

      ## Benefits
      - **Security patches keep flowing**: only feature updates are delayed, never quality updates
      - **Up to a full year**: the documented maximum deferral is 365 days
      - **Slides rather than pins**: unlike pinning a version, you never have to remember to move it forward before servicing ends

      ## Drawbacks
      - **Inert if you pin a version**: Microsoft states feature update deferrals are not in effect while a target release version is set, so do not use both
      - **Does nothing on Home**: Windows Home does not honour Windows Update client policies
      - **New features arrive late**: anything shipped in the annual feature update waits out the deferral

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1607 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores this policy.**
      - **Takes effect**: immediately, at the next Windows Update scan
      - **Reverting**: deletes both values, restoring Windows' normal feature update cadence
      - **Do not combine with pinning a feature version.** Pick one: a deferral slides, a pin holds.
      - This does not touch the separate "pause updates" state the Settings UI writes

      ## Recommendation
      A better default than pinning for most people who want caution without risk: it delays feature
      updates without the trap of a pinned version silently ageing out of support. Use 180 days unless
      you have a reason for more or less, and do not set it on a machine that already has a version
      pin.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Update, DeferFeatureUpdatesPeriodInDays](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [Walkthrough: use Group Policy to configure Windows Update client policies](https://learn.microsoft.com/en-us/windows/deployment/update/waas-wufb-group-policy)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` and `en-US\WindowsUpdate.adml` on build 26100,
   policy `DeferFeatureUpdates`, verbatim, including the 0 to 365 range and the
   `PauseFeatureUpdatesStartTime` sibling element (tier A, shipped ADMX)
2. Policy CSP - Update, `DeferFeatureUpdatesPeriodInDays`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; build
   floor, edition list, range and default, with no deprecation banner)
3. Walkthrough: Use Group Policy to configure Windows Update client policies, "I want to stay on a
   specific version", https://learn.microsoft.com/en-us/windows/deployment/update/waas-wufb-group-policy
   (tier A; "When you specify target version policy, feature update deferrals won't be in effect")

### `block_insider_builds_policy` Block Insider preview builds by policy (new)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (CORRECTED):** one registry effect.

| Key | Value name | Type | Blocked | Windows default (Stock Default) |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `ManagePreviewBuildsPolicyValue` | `REG_DWORD` | `1` (the policy's Disabled state) | `absent` |

No `requires_reboot`, proposed `risk_level: low`. Edition gate: **Pro and above, not Home.**

**Mechanism as proposed (WRONG, shown for contrast):**

| Key | Value name | Type | Hardened | Stock default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `ManagePreviewBuilds` | REG_DWORD | `1` | value-absent |
| same | `ManagePreviewBuildsPolicyValue` | REG_DWORD | `0` (disable preview builds) | value-absent |

Both proposed value names and both proposed values were wrong. The shipped 26100 `WindowsUpdate.admx`
defines exactly one policy here, which writes exactly one flag value:

```xml
<policy name="ManagePreviewBuilds" class="Machine"
        key="Software\Policies\Microsoft\Windows\WindowsUpdate"
        valueName="ManagePreviewBuildsPolicyValue">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_RS3" />
  <enabledValue><decimal value="2" /></enabledValue>
  <disabledValue><decimal value="1" /></disabledValue>
  <elements>
    <enum id="BranchReadinessLevelId" valueName="BranchReadinessLevel" required="true">
      <!-- Dev 2, Beta 4, Release Preview 8, Release Preview quality-only 64 -->
    </enum>
  </elements>
</policy>
```

Three separate errors in the proposal: there is no `ManagePreviewBuilds` DWORD written by Group Policy
(the policy is *named* that; the value it writes is `ManagePreviewBuildsPolicyValue`);
`ManagePreviewBuildsPolicyValue = 0` is not a value the ADMX ever writes (Disabled writes 1, Enabled
writes 2); and the companion element is `BranchReadinessLevel`, not a second preview-builds value.

Policy CSP for `Update/ManagePreviewBuilds` documents the *setting* with allowed values 0 Disable
Preview builds, 1 Disable Preview builds once the next release is public, 2 Enable Preview builds, 3
(Default) left to user selection, with Registry Key Name
`Software\Policies\Microsoft\Windows\WindowsUpdate`, and confirms the edition scope: Pro, Enterprise,
Education, IoT Enterprise and IoT Enterprise LTSC. **Not Home.** `SUPPORTED_Windows_10_0_RS3` means
both target platforms qualify.

Not a duplicate: the corpus's existing `disable_windows_insider` disables the `wisvc` service, which
is the weaker lever because a service can be restarted, while a policy cannot be bypassed from the
Settings UI. Keep both.

**Corrections needed:** corrections 42 and 43. Write `ManagePreviewBuildsPolicyValue = 1` only, and
add the Pro-and-above edition gate the proposal omitted. Deliberately do not ship the CSP's 0 to 3
scale: whether the update stack reads a literal `ManagePreviewBuilds` value when written by MDM, in
addition to `ManagePreviewBuildsPolicyValue`, was not established, and guessing it would be exactly
the kind of unsourced literal this corpus has been burned by.

**Ready-to-paste info block:**

````yaml
    info: |
      **Blocks enrolment in the Windows Insider Program by policy, which the Settings UI cannot override.**

      ## What it does
      Sets `ManagePreviewBuildsPolicyValue = 1` under the Windows Update policy key, which is what the
      "Manage preview builds" Group Policy writes in its Disabled state. Windows then refuses Insider
      preview builds regardless of what the Settings UI offers.

      ## Benefits
      - **Cannot be undone from Settings**: unlike disabling the Insider service, a policy is not something a user can restart around
      - **Stops accidental enrolment**: no route from the Windows Update settings page into preview builds
      - **Complements the service tweak**: the Insider service tweak and this policy close different routes, so both are worth having

      ## Drawbacks
      - **No Insider builds at all**: if you actually want to test upcoming releases on this machine, this blocks it
      - **Does nothing on Home**: Windows Home does not honour Windows Update client policies
      - **Only preview builds**: it does not change anything about ordinary feature or quality updates

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC. **Windows Home ignores this policy.**
      - **Takes effect**: immediately
      - **Reverting**: deletes the value, returning the Insider choice to the user
      - The same policy can also select a preview channel; this tweak deliberately writes only the blocking value and does not touch the channel

      ## Recommendation
      Worth applying on any Pro machine that should never run preview builds, especially a shared or
      work machine. Do not apply it on a box you use for testing upcoming Windows releases.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - Update, ManagePreviewBuilds](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update)
      - [What are Windows Update client policies?](https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` on build 26100, policy `ManagePreviewBuilds`,
   verbatim: `valueName="ManagePreviewBuildsPolicyValue"`, `enabledValue` 2, `disabledValue` 1,
   `supportedOn ref="windows:SUPPORTED_Windows_10_0_RS3"`, and the `BranchReadinessLevel` enum
   (tier A, shipped ADMX)
2. Policy CSP - Update, `ManagePreviewBuilds`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update (tier A; the
   registry key name and the edition list, Home excluded)
3. What are Windows Update client policies?,
   https://learn.microsoft.com/en-us/windows/deployment/update/waas-manage-updates-wufb (tier A;
   supported editions)

### `disable_internet_connection_sharing` Disable Internet Connection Sharing (new)

**Verdict:** VERIFIED

**Mechanism:** a service change plus one registry effect.

| Effect | Target | Disabled | Windows default (Stock Default) |
|---|---|---|---|
| service | `SharedAccess` ("Internet Connection Sharing (ICS)") | start type `Disabled`, service stopped | start type **`Manual` (unconfirmed, see correction 45)** |
| registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Network Connections` `NC_ShowSharedAccessUI` `REG_DWORD` | `0` | `absent` |

No `requires_reboot`, proposed `risk_level: medium`.

`SharedAccess` is present on 26100 with the display name "Internet Connection Sharing (ICS)", backed
by `ipnathlp.dll` in `System32`. It depends on `BFE` and has no SCM dependents.

The policy value checks out, including a polarity that is easy to get wrong. Shipped 26100
`NetworkConnections.admx`:

```xml
<policy name="NC_ShowSharedAccessUI" class="Machine"
        key="Software\Policies\Microsoft\Windows\Network Connections"
        valueName="NC_ShowSharedAccessUI">
  <supportedOn ref="windows:SUPPORTED_WindowsXP" />
  <enabledValue><decimal value="0" /></enabledValue>
  <disabledValue><decimal value="1" /></disabledValue>
```

The policy is "Prohibit use of Internet Connection Sharing on your DNS domain network", and
**enabling** it writes `0`. The proposed hardened value of 0 is therefore correct. Neither the service
nor the value is already in the corpus. DISA's Windows 11 STIG carries a dedicated rule
("Internet connection sharing must be disabled").

**Corrections needed:** corrections 44 and 45, both to the copy and to the revert value rather than to
the mechanism.

The proposal claimed `SharedAccess` backs Mobile Hotspot and that disabling it removes that feature.
The SCM dependency graph on 26100 does not support that: `icssvc` ("Windows Mobile Hotspot Service")
depends on `RpcSs` and `wcmsvc`, not on `SharedAccess`. Mobile Hotspot may still fail at runtime
because it uses the same NAT engine, but a service dependency that does not exist must not be stated
as fact. The breakage that actually bites is the one the proposal omitted: `SharedAccess` /
`ipnathlp` is the NAT engine behind the **Hyper-V Default Switch**, and therefore behind **WSL2
networking** and **Windows Sandbox networking**. Disabling ICS is a well-known way to break WSL2's
internet access, and on a developer machine that is both more likely and far more confusing than
losing Mobile Hotspot. The copy must lead with it.

The stock start type is asserted as `Manual` and is **not confirmed**. No authoritative Microsoft list
of Windows 11 client default start types was found, and the research machine's own `Start` value
cannot stand as evidence of a Windows default. Confirm against a clean 26100 image before shipping the
revert option; reverting a trigger-started service to the wrong start type is a real state leak of
exactly the class `_harmful-revert.md` describes.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns off Internet Connection Sharing, so the PC cannot be turned into an unmanaged router and NAT.**

      ## What it does
      Disables the `SharedAccess` service (Internet Connection Sharing) and enables the "Prohibit use
      of Internet Connection Sharing" policy, which writes `NC_ShowSharedAccessUI = 0` and removes the
      sharing option from network connection properties.

      ## Benefits
      - **No accidental routing**: the machine cannot start NATting another network onto your connection
      - **Removes the UI**: the sharing tab option disappears, so it cannot be enabled by mistake
      - **Benchmark-backed**: DISA's Windows 11 STIG has a dedicated rule requiring ICS be disabled

      ## Drawbacks
      - **Breaks WSL2 and the Hyper-V Default Switch**: `SharedAccess` is the NAT engine behind them, so WSL2 and Windows Sandbox lose internet access
      - **Mobile Hotspot may stop working**: the Hotspot service does not formally depend on this one, but it uses the same NAT engine
      - **No sharing a connection**: if you ever tether one adapter's connection to another, that stops

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021. The policy is declared for Windows XP and later, so every target platform qualifies.
      - **Takes effect**: immediately, the service is stopped on apply
      - **Reverting**: restores the previous service start type from the snapshot and deletes the policy value
      - **Do not apply this on a machine that runs WSL2, Docker Desktop or Windows Sandbox.** It is the most common cause of "WSL2 suddenly has no internet".

      ## Recommendation
      Apply it on an ordinary desktop or work machine that will never share a connection. Do not apply
      it on a development machine using WSL2, Docker Desktop or Windows Sandbox, and do not apply it if
      you use Mobile Hotspot.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Prohibit use of Internet Connection Sharing, Policy CSP ADMX_NetworkConnections](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-networkconnections)
      - [Internet Connection Sharing overview](https://learn.microsoft.com/en-us/windows/win32/ics/internet-connection-sharing)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\NetworkConnections.admx` on build 26100, policy
   `NC_ShowSharedAccessUI`, verbatim: `class="Machine"`,
   `key="Software\Policies\Microsoft\Windows\Network Connections"`, `enabledValue` 0, `disabledValue`
   1, `supportedOn ref="windows:SUPPORTED_WindowsXP"` (tier A, shipped ADMX)
2. Live service presence and SCM dependency graph on build 26100: `SharedAccess` present with display
   name "Internet Connection Sharing (ICS)", backed by `ipnathlp.dll`, depending on `BFE` with no SCM
   dependents; `icssvc` depending on `RpcSs` and `wcmsvc` and **not** on `SharedAccess` (primary
   measurement, tier A for existence and dependencies only, not for the default start type)
3. DISA STIG for Windows 11 V2R2, "Internet connection sharing must be disabled" (tier B)
4. Internet Connection Sharing, https://learn.microsoft.com/en-us/windows/win32/ics/internet-connection-sharing
   (tier A; what ICS does and that `ipnathlp` provides the NAT)

### `require_doh` Require encrypted DNS (new)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (CORRECTED):** one registry effect, from the `DNS_Doh` policy.

| Key | Value name | Type | Values | Windows default (Stock Default) |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `DoHPolicy` | `REG_DWORD` | `1` Prohibit encryption, `2` Allow encryption, `3` Require encryption | `absent` |

Proposed `requires_reboot: true`, proposed `risk_level: medium`.

**Mechanism as proposed (WRONG, shown for contrast):** "`1` Allow DoH, `2` Prohibit DoH, `3` Require
DoH". Values 1 and 2 are transposed. A dropdown built from that table would have made "Allow DoH"
write the value that turns encryption off.

Shipped 26100 `DnsClient.admx`:

```xml
<policy name="DNS_Doh" class="Machine" key="Software\Policies\Microsoft\Windows NT\DNSClient">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_20H2_SERVER_20H2" />
  <elements>
    <enum id="DNS_Doh_Box" valueName="DoHPolicy" required="true">
      <item displayName="$(string.DNS_Doh_Force)">   <value><decimal value="3" /></value></item>
      <item displayName="$(string.DNS_Doh_Auto)">    <value><decimal value="2" /></value></item>
      <item displayName="$(string.DNS_Doh_Disabled)"><value><decimal value="1" /></value></item>
    </enum>
    <enum id="DNS_Doh_Setting_Box" valueName="DohPolicySetting" required="true"> <!-- 0 Allow DoH, 1 Block DoH --> </enum>
    <enum id="DNS_Dot_Setting_Box" valueName="DotPolicySetting" required="true"> <!-- 0 Allow DoT, 1 Block DoT --> </enum>
  </elements>
</policy>
```

The ADML strings settle it: `DNS_Doh_Force` = "Require encryption", `DNS_Doh_Auto` = "Allow
encryption", `DNS_Doh_Disabled` = "Prohibit encryption". So **1 prohibits, 2 allows, 3 requires**.

All three elements are `required="true"`. The policy also carries `DohPolicySetting` (0 allow DoH, 1
block DoH) and `DotPolicySetting` (0 allow DoT, 1 block DoT); DNS over TLS is already present in the
26100 ADMX rather than being a future 25H2 addition. Writing `DoHPolicy` alone still works at the DNS
client level, but the corpus should know it is writing one of three required elements.

This is the first-party, ADMX-backed, enforcing counterpart to the existing `dns_over_https`
(`EnableAutoDoh = 2`), which the corpus's own copy admits is "community-documented only".
`EnableAutoDoh` upgrades opportunistically and falls back to plaintext; `DoHPolicy = 3` fails
resolution rather than falling back. Different guarantees, and the enforcing one is what a
privacy-motivated user actually wants. The verification pass recommends shipping it as an added option
on the existing tweak rather than as a standalone toggle (see merge candidate 5).

**Corrections needed:** corrections 46 and 47. Use 1 prohibits, 2 allows, 3 requires. State that the
ADMX carries three required elements and that this writes one of them. And handle the applicability
mismatch: `supportedOn` is `SUPPORTED_Windows_10_0_20H2_SERVER_20H2`, so the policy is not Windows 11
only, while the existing `dns_over_https` carries `windows: { products: [11] }`, which is correct for
`EnableAutoDoh`. Adding `DoHPolicy` as an option on that tweak inherits a gate narrower than the
policy's own applicability; that is defensible because no shipping Windows 10 release has the DoH
client, but it should be a stated decision rather than an accident.

**Ready-to-paste info block:**

````yaml
    info: |
      **Forces encrypted DNS: if the resolver will not do DNS over HTTPS, name resolution fails rather than falling back to plaintext.**

      ## What it does
      Sets `DoHPolicy = 3` ("Require encryption") under the DNS Client policy key, the first-party
      Group Policy "Configure DNS over HTTPS (DoH) name resolution". Unlike opportunistic
      auto-upgrade, this never falls back to unencrypted queries.

      ## Benefits
      - **A real guarantee**: no silent plaintext fallback, which is the whole weakness of auto-upgrade
      - **Microsoft-documented**: an ADMX-backed policy, not an undocumented registry value
      - **Machine-wide**: applies across every interface, not per-server

      ## Drawbacks
      - **Total name-resolution failure if misconfigured**: if your DNS servers do not speak DoH, nothing resolves and the machine looks like it has no internet
      - **Breaks captive portals**: hotel and airport sign-in pages depend on plaintext DNS interception
      - **Wrong for domain-joined machines**: Microsoft warns against requiring DoH where Windows Server DNS answers internal names, because it does not serve DoH
      - **Needs a DoH-capable resolver set first**: this policy enforces encryption, it does not provide a server

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer. The policy itself declares Windows 10 20H2 and later, but no shipping Windows 10 release has the DoH client, so it has no practical effect there.
      - **Takes effect**: after reboot, or after restarting the DNS Client (`Dnscache`) service
      - **Reverting**: deletes the value, restoring Windows' default encryption behaviour
      - **Set a DoH resolver first** (Cloudflare, Google or Quad9, or one added with `Add-DnsClientDohServerAddress`), then apply this
      - The same policy also carries separate DoH and DNS over TLS sub-settings, which this does not write

      ## Recommendation
      The right choice for a privacy-motivated user on a standalone machine who has already configured
      a DoH-capable resolver and wants encryption guaranteed rather than opportunistic. Do not apply it
      on a domain-joined machine, or on a laptop that regularly connects through captive portals,
      without an easy way to revert.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Secure DNS Client over HTTPS (DoH)](https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support)
      - [Policy CSP - ADMX_DnsClient](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient)
      - [Add-DnsClientDohServerAddress](https://learn.microsoft.com/en-us/powershell/module/dnsclient/add-dnsclientdohserveraddress)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\DnsClient.admx` and `en-US\DnsClient.adml` on build 26100, policy
   `DNS_Doh`, verbatim, with the ADML strings `DNS_Doh_Force` = "Require encryption", `DNS_Doh_Auto` =
   "Allow encryption" and `DNS_Doh_Disabled` = "Prohibit encryption", plus the `DohPolicySetting` and
   `DotPolicySetting` required elements (tier A, shipped ADMX)
2. Secure DNS Client over HTTPS (DoH),
   https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support (tier A; the DoH
   client, the known-server list, and the warning about requiring DoH on domain-joined machines)
3. Policy CSP - ADMX_DnsClient,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient (tier A;
   the DNS Client policy surface)

### `firewall_logging_and_merge` Firewall logging and local policy merge (new)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism (CORRECTED):** four values per profile. `<profile>` is `DomainProfile`, `StandardProfile`
(private) and `PublicProfile`.

| Key | Value name | Type | Hardened | Windows default (Stock Default) |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<profile>\Logging` | `LogDroppedPackets` | `REG_DWORD` | `1` | `absent` |
| same | `LogFilePath` | `REG_SZ` | `%SystemRoot%\System32\logfiles\firewall\<profile>fw.log` | `absent` |
| same | `LogFileSize` | `REG_DWORD` | `16384` (KB) | `absent` (effective 4096) |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<profile>` | `AllowLocalPolicyMerge` | `REG_DWORD` | `0` | `absent` (effective true) |

The `Logging` policy also owns `LogSuccessfulConnections` (`REG_DWORD`, 0 or 1), which this tweak
should leave at 0 to keep the log readable.

No `requires_reboot`, proposed `risk_level: medium`.

Shipped 26100 `WindowsFirewall.admx`, policies `WF_Logging_Name_1` through `_3` (one per profile),
each `class="Machine"` with key `SOFTWARE\Policies\Microsoft\WindowsFirewall\<Profile>\Logging`:

```xml
<boolean id="WF_Logging_LogDroppedPackets"        valueName="LogDroppedPackets">        <trueValue><decimal value="1"/></trueValue><falseValue><decimal value="0"/></falseValue></boolean>
<boolean id="WF_Logging_LogSuccessfulConnections" valueName="LogSuccessfulConnections">  <!-- same shape --> </boolean>
<text    id="WF_Logging_LogFilePathAndName"       valueName="LogFilePath" required="true" />
<decimal id="WF_Logging_SizeLimit"                valueName="LogFileSize" required="true" minValue="128" maxValue="32767" />
```

Value names and types are all correct as proposed, `LogFilePath` is text so `REG_SZ` and the rest are
`REG_DWORD`, and 16384 is inside the documented 128 to 32767 range.

`AllowLocalPolicyMerge` is real but **not ADMX-backed**: it is confirmed absent from all 218 shipped
ADMX files. Microsoft's Firewall CSP documents it per profile
(`MdmStore/DomainProfile/AllowLocalPolicyMerge` and the Private and Public equivalents) with **Default
Value: true**, the semantics "If this value is false, firewall rules from the local store are ignored
and not enforced", scope Device, Windows 10 1709 and later, editions Pro, Enterprise, Education, IoT
Enterprise and IoT Enterprise LTSC. The registry path is corroborated by the corpus itself:
`security:firewall_all_profiles` already writes `EnableFirewall` and `DefaultInboundAction` under
`HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\{DomainProfile,StandardProfile,PublicProfile}`, so
`AllowLocalPolicyMerge` as a sibling value in the same profile key is consistent with the corpus's own
working mechanism plus the CIS check text.

**Corrections needed:** corrections 48, 49 and 50. Write all three logging values together per profile
and revert all three, because `LogFilePath` and `LogFileSize` are `required="true"` and writing
`LogDroppedPackets` alone leaves the policy half-configured from Group Policy tooling's point of view.
Do not copy the ADMX `disabledList` string form: it writes `LogDroppedPackets` and
`LogSuccessfulConnections` as `<string>0</string>` while the enabled path writes decimals, and the
effective values are `REG_DWORD` (a `REG_SZ` here would be silently ignored). Split
`AllowLocalPolicyMerge` into a separate, clearly warned option with its failure mode stated bluntly,
and add the Pro-and-above edition gate; logging alone should be the default recommendation.

**Ready-to-paste info block:**

````yaml
    info: |
      **Turns on firewall drop logging across all three profiles, and optionally stops applications adding their own allow rules.**

      ## What it does
      Writes `LogDroppedPackets = 1` with a log path and a 16 MB size limit into the Domain, Private
      and Public firewall profile policy keys, so blocked inbound packets are recorded. The stricter
      option also sets `AllowLocalPolicyMerge = 0`, which makes Windows ignore locally created
      firewall rules.

      ## Benefits
      - **You can see what is blocked**: dropped packets land in a per-profile log instead of vanishing silently
      - **Bigger log**: 16 MB instead of the 4 MB default, so the log covers more than a few minutes of noise
      - **Stops silent rule creation**: with local policy merge off, an application cannot quietly add an allow rule for itself
      - **Benchmark-backed**: CIS recommends firewall logging and merge control on all three profiles

      ## Drawbacks
      - **Local policy merge off breaks inbound, silently**: on a standalone machine there is no pushed rule set to fall back to, so the "Windows Defender Firewall has blocked some features of this app" prompt never appears and games or servers just fail to accept connections
      - **Log noise**: a Public-profile log fills quickly on a busy network
      - **Small disk cost**: up to 16 MB per profile
      - **Does nothing on Home for the merge half**: `AllowLocalPolicyMerge` is documented for Pro and above

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later including LTSC 2021, on Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC
      - **Takes effect**: immediately
      - **Reverting**: deletes all four values per profile, restoring the default log settings and re-enabling local rule merge
      - Logs land at `%SystemRoot%\System32\logfiles\firewall\<profile>fw.log`
      - `AllowLocalPolicyMerge` is documented by Microsoft's Firewall CSP but is not in the shipped ADMX; it is written directly

      ## Recommendation
      Turn on logging on any machine where you want to diagnose blocked connections; it is cheap and
      reversible. Only turn off local policy merge on a machine whose firewall rules are managed
      centrally, because on a standalone PC it breaks inbound connections with no prompt and no
      obvious cause.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Firewall CSP, MdmStore AllowLocalPolicyMerge](https://learn.microsoft.com/en-us/windows/client-management/mdm/firewall-csp)
      - [Configure the Windows Firewall log](https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/configure-logging)
````

**Sources:**
1. `C:\Windows\PolicyDefinitions\WindowsFirewall.admx` on build 26100, policies `WF_Logging_Name_1`
   through `_3`, verbatim: `class="Machine"`, key
   `SOFTWARE\Policies\Microsoft\WindowsFirewall\<Profile>\Logging`, the four elements with
   `LogFilePath` and `LogFileSize` marked `required="true"` and the 128 to 32767 range (tier A,
   shipped ADMX)
2. Firewall CSP, `MdmStore/<Profile>/AllowLocalPolicyMerge`,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/firewall-csp (tier A; Default
   Value: true, "If this value is false, firewall rules from the local store are ignored and not
   enforced", Windows 10 1709 and later, edition list)
3. Configure the Windows Firewall log,
   https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/configure-logging
   (tier A; the default log path and the 4096 KB default size)
4. CIS Windows 11 v4.0.0 Level 1, firewall logging and local policy merge recommendations for all
   three profiles (tier B); registry path corroborated by `src-tauri/tweaks/security.yaml`
   (`firewall_all_profiles`), which already writes sibling values in the same profile keys

## Unresolved

Recorded so they are not rediscovered. None of these blocks shipping, but each affects either a
revert value or a factual claim in user-facing copy.

1. **The Delivery Optimization unconfigured default.** Microsoft's reference page says LAN(1) and its
   Policy CSP page says 0. Both tier A, both current. This determines whether
   `disable_delivery_optimization_p2p` changes behaviour on a stock machine or only pins it. It does
   not affect revert, because `absent` is correct under either reading.
2. **The stock start type of `SharedAccess`.** Asserted as `Manual`, not confirmed against a clean
   image. Blocks a safe revert option for `disable_internet_connection_sharing`.
3. **`SmartActiveHoursState` semantics.** Winaero gives 1 = on and 2 = off; other community sources
   give 0 and 1. Microsoft documents neither the value nor its range.
4. **`SearchOrderConfig = 0` on the non-policy path.** No tier A or B definition, and community
   sources disagree about whether 0 is meaningful outside the policy key.
5. **Whether `AllowTemporaryEnterpriseFeatureControl` alone makes a device "update-managed".** The
   ADML says the gating applies only to update-managed devices; no tier A source says whether writing
   this value is itself sufficient to qualify.
6. **`ManagePreviewBuilds` versus `ManagePreviewBuildsPolicyValue`.** The shipped ADMX writes only the
   latter (1 or 2); the Policy CSP documents the setting as the former on a 0 to 3 scale. Whether the
   update stack also reads a literal `ManagePreviewBuilds` value when written by MDM was not
   established, so only the ADMX-backed value is shipped.
7. **The exact `AllowLocalPolicyMerge` registry path.** Semantics and per-profile default are
   Microsoft-documented via the Firewall CSP; the
   `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<Profile>\AllowLocalPolicyMerge` path is
   corroborated by CIS check text plus the corpus's own working use of the same profile keys, not by
   a shipped ADMX.
8. **The clean-image baseline.** Per `_harmful-revert.md` this remains the single highest-value
   missing input for the project. In this category it would settle items 1 and 2 above, plus the stock
   per-adapter `PnPCapabilities` distribution and the stock active hours question.
