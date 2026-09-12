# Adversarial verification: gap proposals 13 to 38 (Medium and Low)

Verified 2026-07-26. Subject: `_gaps-security-performance-network.md`, proposals **13 to 32 (Medium)**
and **33 to 38 (Low)**. Proposals 1 to 12 (High) are another verifier's assignment and are not
covered here.

Target platform: Windows 11 24H2 (26100) and 25H2 (26200) primary; Windows 10 IoT Enterprise
LTSC 2021 (19044) secondary.

## Method and what counts as evidence here

The posture is refutation. Each proposal was attacked on: does a real control exist on 26100; is the
key, value name and type exactly right; do the option values mean what is claimed; is it a duplicate
of something the corpus already ships; is it gated by build, edition or hardware; and (for
performance) is there a measurement rather than an assertion.

Evidence used:

1. **The ADMX and ADML set shipped on this 26100 machine** (`C:\Windows\PolicyDefinitions`,
   218 `.admx` files). Policy blocks were extracted verbatim, including `class`, `key`, `valueName`,
   `enabledValue` / `disabledValue`, enum items, and element `minValue` / `maxValue`. Where a value
   name is reported "not in any shipped ADMX", all 218 files were scanned.
   **Encoding note:** `Search.admx` and `Camera.admx` ship as UTF-16LE while the other 216 are UTF-8.
   A naive UTF-8 grep silently misses them. This is almost certainly why proposal 34 wrongly reports
   its policy as absent from the shipped ADMX, and it is worth knowing for any future ADMX pass.
2. **Microsoft Learn**, fetched live: Policy CSP pages, the ASR rules reference, the BitLocker
   overview, the Firewall CSP, `fsutil 8dot3name`, `Set-WindowsReservedStorageState`, the CredSSP
   CVE-2018-0886 KB, the NTLM and PKU2U Security Policy Settings pages.
3. **Presence of scheduled tasks** by file presence under `C:\Windows\System32\Tasks`, and
   **presence of services** plus their SCM dependency graph. Per the ground rules, no registry value,
   service start type or task `<Enabled>` element on this machine was used as evidence of any Windows
   default.
4. **The corpus itself** (`security.yaml`, `performance.yaml`, `network.yaml`, `services.yaml`, and
   for duplicate hunting also `privacy.yaml`, `debloat.yaml`, `interface.yaml`).

What this pass could **not** establish: shipped default start types for `seclogon`, `WerSvc` and
`SharedAccess`, and stock-image absence for the MSS values. Those are listed under Unknowns.

---

## Verdict summary

| # | Proposal | Verdict | Headline |
|---|---|---|---|
| 13 | `asr_extended_rules` | CONFIRMED | All six GUIDs verbatim-correct, no corpus overlap; three dependency notes missing from the copy |
| 14 | `ntlm_outgoing_restriction` | CORRECTED | NTLM session-security bit semantics are inverted in the rationale |
| 15 | `rdp_session_hardening` | CONFIRMED | All five confirmed; `DisablePasswordSaving` class is Both-not-Machine; `MinEncryptionLevel` is inert under TLS |
| 16 | `task_compatibility_appraiser` | REJECTED | Duplicate: `privacy.yaml` already disables 2 of the 5 tasks plus the policies |
| 17 | `task_ceip` | REJECTED | Duplicate: `privacy.yaml` already disables Consolidator and UsbCeip |
| 18 | `disable_secondary_logon` | CONFIRMED | Service present, not in corpus, STIG-backed |
| 19 | `early_launch_antimalware_policy` | CORRECTED | "Good only" is `8` in the shipped ADMX, not `0` |
| 20 | `credssp_encryption_oracle` | CORRECTED | Stock default is Mitigated (1), not Vulnerable (2), since May 2018 |
| 21 | `device_encryption_posture` | CONFIRMED | Exact match to Microsoft Learn; the build gate is too narrow |
| 22 | `hide_admin_accounts_on_elevation` | CONFIRMED | Both values, keys and classes correct |
| 23 | `event_log_retention` | CONFIRMED | `MaxSize` confirmed with range 1024 to 2147483647; Setup channel omitted |
| 24 | `audit_process_creation_cmdline` | CONFIRMED | Exact ADMX match; the auditpol half is not a registry effect and must revert too |
| 25 | `disable_wer_service` | CONFIRMED | Service present, genuinely absent from the corpus; drop the CPU claim |
| 26 | `spooler_remote_rpc_off` | CORRECTED | Wrong key for `RegisterSpoolerRemoteRpcEndPoint` |
| 27 | `block_insider_builds_policy` | CORRECTED | Both value names and both values are wrong |
| 28 | `autoplay_non_volume` | CONFIRMED | Correct; class is Both, so write HKCU as well |
| 29 | `remote_uac_token_filter` | CONFIRMED | Matches Microsoft exactly, including the default |
| 30 | `disable_internet_connection_sharing` | CONFIRMED | Correct, but the stated risk is wrong and the real one is missing |
| 31 | `reserved_storage_off` | CONFIRMED | Cmdlet confirmed; the 7 GB figure is unsourced |
| 32 | `firewall_logging_and_merge` | CONFIRMED | Confirmed with range limits and a required-element constraint |
| 33 | `enable_sehop` | CONFIRMED | Real, correctly inverted, near-zero practical delta on x64 |
| 34 | `no_index_encrypted_files` | CORRECTED | It **is** in the shipped ADMX; the sourcing claim is wrong |
| 35 | `disable_pku2u_online_id` | CORRECTED | Microsoft says the policy is enabled by default on client since 1607 |
| 36 | `mss_network_stack_hardening` | REJECTED | No stock defaults established, no user-visible benefit, one member is a no-op |
| 37 | `ntfs_disable_8dot3` | REJECTED | The load-bearing Microsoft performance claim is not on the cited page |
| 38 | `require_doh` | CORRECTED | `DoHPolicy` values 1 and 2 are swapped in the proposal |

Counts: 13 CONFIRMED, 9 CORRECTED, 4 REJECTED, 0 UNRESOLVED at the proposal level (per-item
unknowns are listed at the end).

---

## Entries

### 13. `asr_extended_rules` (Medium) - CONFIRMED

**Attack 1, does the control exist.** Yes. All six GUIDs were checked verbatim against the Microsoft
ASR rules reference and every one resolves to the rule the proposal names:

| GUID | Microsoft rule name | Dependencies per Microsoft |
|---|---|---|
| `d3e037e1-3eb8-44c8-a917-57927947596d` | Block JavaScript or VBScript from launching downloaded executable content | Defender AV, AMSI |
| `92e97fa1-2edf-4476-bdd6-9dd0b4dddc7b` | Block Win32 API calls from Office macros | Defender AV, AMSI |
| `b2b3f03d-6a65-4f7b-a9c7-1c7ef74a9ba4` | Block untrusted and unsigned processes that run from USB | Defender AV |
| `c1db55ab-c21a-4637-bb3f-a12568109d35` | Use advanced protection against ransomware | Defender AV, cloud-delivered protection |
| `d1e49aac-8f56-4280-b9ba-993a6d77406c` | Block process creations originating from PSExec and WMI commands | Defender AV |
| `26190899-1602-49e8-8b27-eb1d0a1ce869` | Block Office communication application from creating child processes | Defender AV |

**Attack 2, key and type.** The corpus writes ASR rules as `REG_SZ` under
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`
(confirmed in `security.yaml` at the `asr_block_lsass_theft` effect). The proposal follows that
convention. Correct.

**Attack 3, duplicate.** No. `security.yaml` contains exactly five ASR GUIDs
(`9e6c4e1f`, `d4f940ab`, `3b576869`, `5beb7efe`, `be9ba2d9`). None of the six proposed GUIDs
appears. Clean.

**What the proposal gets wrong (info copy, not mechanism).** Three Microsoft-documented constraints
are missing:

1. `c1db55ab` requires cloud-delivered protection to be enabled. The corpus does not enable it today
   (that is proposal 1, a separate High item). Shipping "Block all six" without proposal 1 gives the
   user one rule that is configured but non-functional, which is exactly the failure mode proposal 1
   was written to fix. Either gate this rule on proposal 1 or say plainly in the copy that it does
   nothing until cloud protection is on.
2. `d3e037e1` in Block or Warn mode: Microsoft notes EDR alerts are generated only when the device
   cloud protection level is High plus or Zero tolerance. The blocking still works; the alerting
   does not.
3. `26190899` "is enforced only if Office is installed in `%ProgramFiles%` or `%ProgramFiles(x86)%`".
   Microsoft Store and per-user Office installs are outside that, so on a large fraction of consumer
   machines this rule is inert. The proposal calls it "Outlook-specific"; it is narrower than that.

**Sources.** Microsoft ASR rules reference, per-rule detail sections (tier A);
`src-tauri/tweaks/security.yaml` for the corpus convention and dedupe.

### 14. `ntlm_outgoing_restriction` (Medium) - CORRECTED

**The refutation.** The proposal's most interesting claim is wrong, and it is wrong in the direction
that matters. It states:

> the stock 26100 value `0x20000000` requires NTLMv2 session security but **not** 128-bit
> encryption. `0x20080000` adds `NTLMSSP_NEGOTIATE_128`.

Microsoft's Policy CSP for `NetworkSecurity_MinimumSessionSecurityForNTLMSSPBasedClients` and
`...ForNTLMSSPBasedServers` documents the allowed values explicitly:

| Value | Decimal | Microsoft's description |
|---|---|---|
| `0` | 0 | None |
| `0x00080000` | 524288 | Require NTLMv2 session security |
| `0x20000000` | 536870912 | **Require 128-bit encryption (Default)** |
| `0x20080000` | 537395200 | Require NTLM and 128-bit encryption |

So `0x20000000` is the 128-bit bit, already the default. What `0x20080000` adds is the **NTLMv2
session-security** bit, not the 128-bit bit. The two are transposed in the proposal's rationale. The
recommended hardened value `0x20080000` is nevertheless correct and is what CIS and the STIG ask
for, so the corrected entry keeps the value and rewrites the reason.

Also worth noting: Microsoft lists 536870912 as the **Default Value** for both the client and the
server setting, which independently corroborates the proposal's live read without relying on this
machine.

**Attack on the revert value.** The proposal's table implies the stock default is a written
`0x20000000`. Microsoft documents 536870912 as the effective default, not as a value that is
necessarily present in the registry. The "Windows default (Stock Default)" option must therefore
write **absent** for `NtlmMinClientSec` and `NtlmMinServerSec`, not `0x20000000`. Writing the number
back would be a plausible-looking revert that pins a value the OS did not have, which is the corpus's
most damaging defect class.

**Attack on the other two values.** Microsoft's Security Policy Settings page for
"Network security: Restrict NTLM: Outgoing NTLM traffic to remote servers" confirms the three states
(Allow all, Audit all, Deny all) and confirms that "not defined" behaves as Allow all, which
validates value-absent as the revert. It does **not** publish the registry value name or the numeric
mapping. `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0`, `RestrictSendingNTLMTraffic` and
`AuditOutgoingNTLMTraffic` with `0` / `1` / `2` come from the DISA STIG and CIS check text (tier B),
and neither name appears in any shipped ADMX (verified across all 218 files, consistent with the
proposal's own statement that these are Security Options rather than administrative templates).
That sourcing level is acceptable but should be labelled as such in the corpus copy.

**Corrected entry.**

| Key | Value name | Type | Hardened | Stock default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `AuditOutgoingNTLMTraffic` | REG_DWORD | `2` (audit all) | absent |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `RestrictSendingNTLMTraffic` | REG_DWORD | `2` (deny all) | absent |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `NtlmMinClientSec` | REG_DWORD | `0x20080000` (128-bit **plus** NTLMv2 session security) | **absent**, effective `0x20000000` (128-bit only) |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `NtlmMinServerSec` | REG_DWORD | `0x20080000` | **absent**, effective `0x20000000` |

**Sources.** Policy CSP LocalPoliciesSecurityOptions,
`NetworkSecurity_MinimumSessionSecurityForNTLMSSPBasedClients` and `...Servers` (tier A);
Microsoft Learn "Network security: Restrict NTLM: Outgoing NTLM traffic to remote servers" (tier A);
DISA STIG for Windows 11 V2R2 (tier B) for the registry names; all-ADMX scan on 26100 (tier A,
negative result).

### 15. `rdp_session_hardening` (Medium) - CONFIRMED

All five confirmed verbatim in `TerminalServer.admx` on 26100, key
`SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` in every case:

| Value name | Policy | Class | Enabled / Disabled |
|---|---|---|---|
| `MinEncryptionLevel` | `TS_ENCRYPTION_POLICY` | Machine | enum: `1` Low Level, `2` Client Compatible, `3` High Level |
| `fEncryptRPCTraffic` | `TS_RPC_ENCRYPTION` | Machine | `1` / `0` |
| `fPromptForPassword` | `TS_PASSWORD` | Machine | `1` / `0` |
| `DisablePasswordSaving` | `TS_CLIENT_DISABLE_PASSWORD_SAVING_1` and `_2` | **User and Machine** | `1` / `0` |
| `fDisableCdm` | `TS_CLIENT_DRIVE_M` | Machine | `1` / `0` |

**Corrections.**

1. The proposal says "all class Machine". `DisablePasswordSaving` is defined twice, once as class
   User (`_1`) and once as class Machine (`_2`), both against the same key. The HKLM write the
   proposal implies is the correct and effective one, so this is a documentation fix rather than a
   defect, but the entry should not claim uniformity that is not there.
2. `MinEncryptionLevel` = `3` is the **maximum** the shipped ADMX offers. There is no FIPS (`4`) item
   in the 26100 enum, and the ADML has no `TS_ENCRYPTION_FIPS_LEVEL` string. Any option list must
   stop at 3.
3. A real overlap the proposal misses: `MinEncryptionLevel` governs the legacy RDP security layer.
   The corpus's existing `rdp_security_hardening` forces TLS plus NLA, and under TLS the encryption
   level is negotiated by TLS rather than by this value. Applying both leaves `MinEncryptionLevel`
   largely inert. That is not a reason to drop it (it is still the STIG check), but the copy must not
   imply it is doing work it is not.

**Applicability.** `SUPPORTED_WindowsXP` / `SUPPORTED_WindowsNET` on all five, so no build gate is
needed. All five are inert unless RDP is in use, which makes this tweak and the corpus's
`disable_remote_desktop` mutually exclusive postures. The proposal already says so.

**Sources.** `C:\Windows\PolicyDefinitions\TerminalServer.admx` and `en-US\TerminalServer.adml`
(26100) (tier A); DISA STIG for Windows 11 V2R2 (tier B).

### 16. `task_compatibility_appraiser` (Medium) - REJECTED

**This is a duplicate and the proposal's premise is false.** The proposal states:

> The corpus has `AITEnable` and `DisableInventory` in `privacy.yaml`, which is the policy-level
> control, but **no task-level control**.

`privacy.yaml` contains `disable_compat_appraiser`, whose effects are:

```
- id: ait_enable        registry HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat  AITEnable
- id: disable_inventory registry HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat  DisableInventory
- id: task_appraiser    task '\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser'
- id: task_progdata     task '\Microsoft\Windows\Application Experience\ProgramDataUpdater'
- id: task_startup      task '\Microsoft\Windows\Application Experience\StartupAppTask'
```

Two of the five proposed tasks (`Microsoft Compatibility Appraiser`, `StartupAppTask`) are already
shipped, as are both policy values. A second tweak covering the same tasks would let a user put the
app into two contradictory states over one task, which is exactly the conflict class the corpus is
supposed to avoid.

**What is genuinely new** is three tasks: `Microsoft Compatibility Appraiser Exp`, `PcaPatchDbTask`,
`MareBackup`. Add them as effects on the existing `disable_compat_appraiser` rather than shipping a
new tweak.

**A defect the proposal walked past.** It correctly observes that `ProgramDataUpdater` is not present
on 26100, but does not notice that **the corpus already ships a `ProgramDataUpdater` effect**. Task
presence was re-verified here by file enumeration of
`C:\Windows\System32\Tasks\Microsoft\Windows\Application Experience`, which on this 26100 install
contains exactly: `MareBackup`, `Microsoft Compatibility Appraiser`,
`Microsoft Compatibility Appraiser Exp`, `PcaPatchDbTask`, `SdbinstMergeDbTask`, `StartupAppTask`.
No `ProgramDataUpdater`, no `AitAgent`. So `disable_compat_appraiser` has the same live defect as
D.2 in the source document: an effect targeting a task that does not exist. This belongs in the
defects list, not the gaps list.

`SdbinstMergeDbTask` is present and is not mentioned anywhere in the proposal. It is application
compatibility shim database maintenance; it is not telemetry, and it should stay enabled.

**Sources.** `src-tauri/tweaks/privacy.yaml` (`disable_compat_appraiser`); file enumeration of
`C:\Windows\System32\Tasks\Microsoft\Windows\Application Experience` on 26100 (tier A for presence;
no `<Enabled>` state was read).

### 17. `task_ceip` (Medium) - REJECTED

Same failure as 16. `privacy.yaml` already ships a CEIP tweak whose effects include
`task_consolidator` targeting
`\Microsoft\Windows\Customer Experience Improvement Program\Consolidator` and `task_usbceip`
targeting `\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip`, alongside
`CEIPEnable` under `HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows`. Two of the three proposed
tasks are duplicates.

**What is genuinely new** is `\Microsoft\Windows\PI\Sqm-Tasks`. Add it to the existing tweak.

**A second live defect surfaced.** The existing tweak has a `task_kernelceip` effect. Enumeration of
`C:\Windows\System32\Tasks\Microsoft\Windows\Customer Experience Improvement Program` on 26100 shows
exactly two entries, `Consolidator` and `UsbCeip`. `KernelCeipTask` is not present. The corpus's info
copy hedges ("`KernelCeipTask` may be absent on 24H2, which is normal"), but a hedge in prose does
not stop the effect from producing a missing-task probe result. This should be retargeted to
`\Microsoft\Windows\PI\Sqm-Tasks` or removed.

**The proposal's caution is right and should be carried over:** `\Microsoft\Windows\PI\Secure-Boot-Update`
sits in the same `PI` folder (verified present on 26100) and delivers Secure Boot DBX revocation
updates. It must not be touched.

**Sources.** `src-tauri/tweaks/privacy.yaml`; file enumeration of the CEIP and PI task folders on
26100 (tier A for presence).

### 18. `disable_secondary_logon` (Medium) - CONFIRMED

**Existence.** `seclogon` is present on 26100 with display name "Secondary Logon"; `seclogon.dll` is
present in `System32`. The SCM dependency graph shows no dependent services and no service
dependencies, so disabling it does not cascade.

**Duplicate check.** Not in the corpus. `services.yaml` ships 29 tweaks and none targets `seclogon`.

**Attack on the risk statement.** The proposal is slightly optimistic. Beyond `runas` and the
Shift-right-click "Run as different user" verb, `seclogon` backs `CreateProcessWithLogonW`, which
some installers, deployment tools and scripted automation use directly. The proposal's core claim
that same-user UAC elevation is unaffected is correct. A user who administers other machines from
this one, or who keeps a separate admin account, will notice; a single-account consumer will not.

**Unresolved.** The shipped default start type could not be established from an authoritative source.
The proposal says Manual. That is consistent with the STIG having a rule requiring it be set to
Disabled (which would be pointless if it shipped Disabled), but it was not confirmed against a clean
image. Get this right before writing the revert option.

**Sources.** Live service presence and SCM dependency graph on 26100 (tier A for existence only);
DISA STIG for Windows 11 V2R2 (tier B); `src-tauri/tweaks/services.yaml` for dedupe.

### 19. `early_launch_antimalware_policy` (Medium) - CORRECTED

**The refutation.** The proposal states: "Accepted values: `0` Good only, `1` Good and unknown,
`3` Good, unknown and bad but critical, `7` All (no filtering)", and then spends a paragraph warning
against offering `0`. The shipped `EarlyLaunchAM.admx` on 26100 says otherwise:

```xml
<policy name="POL_DriverLoadPolicy_Name" class="Machine"
        key="System\CurrentControlSet\Policies\EarlyLaunch" valueName="DriverLoadPolicy">
  <supportedOn ref="windows:SUPPORTED_Windows8" />
  <elements>
    <enum id="SelectDriverLoadPolicy" key="System\CurrentControlSet\Policies\EarlyLaunch"
          valueName="DriverLoadPolicy" required="true">
      <item displayName="$(string.SelectDriverLoadPolicy-GoodOnly)">                        <value><decimal value="8" /></value></item>
      <item displayName="$(string.SelectDriverLoadPolicy-GoodPlusUnknown)">                 <value><decimal value="1" /></value></item>
      <item displayName="$(string.SelectDriverLoadPolicy-GoodPlusUnknownPlusKnownBadCritical)"><value><decimal value="3" /></value></item>
      <item displayName="$(string.SelectDriverLoadPolicy-All)">                             <value><decimal value="7" /></value></item>
    </enum>
  </elements>
</policy>
```

"Good only" is **`8`**, not `0`. `0` is not a member of the enum at all. The proposal's value list is
wrong, and its safety warning names a value that cannot be selected. The warning content is still
right, it just needs to attach to `8`.

**Everything else holds.** Key `HKLM\SYSTEM\CurrentControlSet\Policies\EarlyLaunch`, value name
`DriverLoadPolicy`, `REG_DWORD`, class Machine, `SUPPORTED_Windows8` so both target platforms
qualify. Hardened `3` is correct. Not in the corpus. Requires reboot. Revert is value-absent.

**Corrected option list.** Offer `3` (Good, unknown and bad but critical) and `1` (Good and unknown).
Do **not** offer `8`, because it refuses to boot-load any driver ELAM cannot vouch for and can
produce a machine that will not start on unusual hardware. Do not offer `7` either, since that is
"no filtering" and is the state malware wants.

**Sources.** `C:\Windows\PolicyDefinitions\EarlyLaunchAM.admx` (26100), policy
`POL_DriverLoadPolicy_Name`, verbatim (tier A); DISA STIG for Windows 11 V2R2 (tier B).

### 20. `credssp_encryption_oracle` (Medium) - CORRECTED

**The refutation.** The proposal claims the stock default is `2` (Vulnerable) and reasons from that:

> the OS still ships with the permissive behaviour so that unpatched RDP servers remain reachable.

Microsoft's own KB for CVE-2018-0886 states plainly: **"May 8, 2018. An update to change the default
setting from Vulnerable to Mitigated."** and "By default, after this update is installed, patched
clients cannot communicate with unpatched servers." On 26100 the effective default with the value
absent is **Mitigated (`1`)**, not Vulnerable. Any 24H2 machine has been at Mitigated for years.

That materially shrinks the tweak. Going from the effective default `1` to the hardened `0` changes
only the **server** half of the behaviour: with `1`, "services that use CredSSP will accept
unpatched clients"; with `0`, they will not. The client half (no fallback to insecure versions) is
already in force at the default. The proposal's stated risk ("connecting out by RDP to an old,
unpatched server will start failing") is a description of `1`, which is already what the machine
does, so the risk is smaller than stated too.

**Everything else holds.** Confirmed in `CredSsp.admx` on 26100: policy `AllowEncryptionOracle`,
class Machine, key `Software\Microsoft\Windows\CurrentVersion\Policies\System\CredSSP\Parameters`,
enum `AllowEncryptionOracle` with `0` Force Updated Clients, `1` Mitigated, `2` Vulnerable. The
Microsoft KB confirms the same three registry values with the same names. `supportedOn` is
`SUPPORTED_WindowsVista`, so no build gate. Not in the corpus.

**Corrected entry.**

| Key | Value name | Type | Hardened | Stock default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\CredSSP\Parameters` | `AllowEncryptionOracle` | REG_DWORD | `0` (Force Updated Clients) | **value-absent, effective `1` (Mitigated) since the May 2018 update** |

**Sources.** `C:\Windows\PolicyDefinitions\CredSsp.admx` (26100) (tier A); Microsoft KB
"CredSSP updates for CVE-2018-0886", fetched live, the changelog line and the three-option table
(tier A); CIS Windows 11 v4.0.0 (tier B).

### 21. `device_encryption_posture` (Medium) - CONFIRMED

**Exact match.** Microsoft Learn's BitLocker overview, section "Disable device encryption", gives:

| Path | Name | Type | Value |
|---|---|---|---|
| `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\BitLocker` | `PreventDeviceEncryption` | REG_DWORD | 0x1 |

Key, value name, type and value all match the proposal exactly. Not in any shipped ADMX (verified),
which is consistent, since Microsoft documents it as a direct registry setting rather than a policy.
Not in the corpus.

**One correction: the applicability gate is too narrow.** The proposal gates on
`windows: { build: ">=26100" }`. The registry value and automatic device encryption long predate
24H2; what 24H2 changed was the hardware prerequisite, so that many more machines auto-encrypt. On a
19044 LTSC machine the value is read just the same. Gating it out of older builds hides a control
that works there. If a gate is wanted, gate the *copy* (say that 24H2 made this much more likely to
bite), not the mechanism.

**The proposal's own cautions are correct and unusually well judged**: it only affects automatic
enablement so it must probe the actual volume state rather than claim an effect; it is a security
downgrade and must say so; and the performance argument should not lead, since 25H2 shipped
hardware-accelerated BitLocker. All of that survives scrutiny. Note the awkwardness that a tweak
whose hardened direction is *less* protection sits oddly in a security category.

**Sources.** Microsoft Learn BitLocker overview, "Disable device encryption" table, fetched live
(tier A); all-ADMX scan on 26100 (tier A, negative result).

### 22. `hide_admin_accounts_on_elevation` (Medium) - CONFIRMED

Both confirmed verbatim in `CredUI.admx` on 26100, and the proposal's unusual observation that the
two live under different keys with different classes is correct:

```xml
<policy name="EnumerateAdministrators" class="Machine"
        key="Software\Microsoft\Windows\CurrentVersion\Policies\CredUI"
        valueName="EnumerateAdministrators">
  <supportedOn ref="windows:SUPPORTED_WindowsVista" />
  <enabledValue><decimal value="1" /></enabledValue> ...

<policy name="DisablePasswordReveal" class="Both"
        key="Software\Policies\Microsoft\Windows\CredUI"
        valueName="DisablePasswordReveal">
  <supportedOn ref="windows:SUPPORTED_Windows8_Or_IE10" />
  <enabledValue><decimal value="1" /></enabledValue> ...
```

Polarity check: `EnumerateAdministrators` enabled (`1`) means *do* enumerate, so the hardened value
is `0`, which is what the proposal says. `DisablePasswordReveal` enabled (`1`) means *do* disable the
reveal button, hardened `1`. Neither name is inverted relative to the proposal's claim.

**One correction.** `DisablePasswordReveal` is class **Both**. The proposal writes HKLM only. For
reliable effect the HKCU twin (`HKCU\SOFTWARE\Policies\Microsoft\Windows\CredUI`) should be written
as well, and the revert must remove both.

**Duplicate check.** Neither value is anywhere in the corpus. `hide_last_user` in `security.yaml`
covers `DontDisplayLastUserName`, a different value in a different key. Complementary, not
overlapping.

**Sources.** `C:\Windows\PolicyDefinitions\CredUI.admx` (26100), verbatim (tier A); DISA STIG for
Windows 11 V2R2 and CIS Windows 11 v4.0.0 (tier B).

### 23. `event_log_retention` (Medium) - CONFIRMED

Confirmed in `EventLog.admx` on 26100. There are four channel policies, not three:

| Policy | Key |
|---|---|
| `Channel_LogMaxSize_1` | `Software\Policies\Microsoft\Windows\EventLog\Application` |
| `Channel_LogMaxSize_2` | `Software\Policies\Microsoft\Windows\EventLog\Security` |
| `Channel_LogMaxSize_3` | `Software\Policies\Microsoft\Windows\EventLog\Setup` |
| `Channel_LogMaxSize_4` | `Software\Policies\Microsoft\Windows\EventLog\System` |

Each is class Machine with a single element:
`<decimal id="Channel_LogMaxSize" valueName="MaxSize" required="true" minValue="1024" maxValue="2147483647" />`.

Value name `MaxSize` and type `REG_DWORD` are correct. All three proposed sizes (32768, 196608,
1024000) are inside the declared range, so none can be silently rejected. Not in the corpus.

**Corrections.**

1. The proposal's "channel default 20480 KB" is asserted with no source and was not independently
   confirmed here. This does not affect correctness of the revert, because the revert is value-absent
   (which the proposal has right), but it should not be stated as fact in the info copy without a
   source.
2. The Setup channel (`Channel_LogMaxSize_3`) exists and is omitted. Not important, but the entry
   should say "three of the four channels" rather than implying the set is complete.

**Sources.** `C:\Windows\PolicyDefinitions\EventLog.admx` (26100), all four policy blocks verbatim
(tier A); DISA STIG for Windows 11 V2R2 (tier B).

### 24. `audit_process_creation_cmdline` (Medium) - CONFIRMED

Confirmed verbatim in `AuditSettings.admx` on 26100:

```xml
<policy name="IncludeCmdLine" class="Machine"
        key="Software\Microsoft\Windows\CurrentVersion\Policies\System\Audit"
        valueName="ProcessCreationIncludeCmdLine_Enabled">
  <supportedOn ref="windows:SUPPORTED_Windows_6_3" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

Key, value name, type, polarity and hardened value all correct. `SUPPORTED_Windows_6_3` is
Windows 8.1 and later, so both target platforms qualify. Not in the corpus.

**One correction the proposal understates.** The registry value on its own logs nothing. It only adds
the command line to event 4688, which is only generated when the Detailed Tracking, Process Creation
audit subcategory is enabled. The proposal pairs it with
`auditpol /set /subcategory:"{0CCE922B-69AE-11D9-BED3-505054503030}" /success:enable`, which follows
the pattern the corpus already uses (`audit_logon_events` in `security.yaml` uses the
`{0CCE9215-...}` subcategory, a different one, so there is no collision). But `auditpol` state is not
a registry value, so the snapshot and revert path has to capture and restore the prior subcategory
setting, not just delete the DWORD. If the revert only deletes `ProcessCreationIncludeCmdLine_Enabled`
it will leave process-creation auditing switched on, which is a silent state leak.

The proposal's privacy caution (command lines can carry secrets, and they then sit in the Security
log) is real and correctly stated.

**Sources.** `C:\Windows\PolicyDefinitions\AuditSettings.admx` (26100), verbatim (tier A);
DISA STIG for Windows 11 V2R2 (tier B); `src-tauri/tweaks/security.yaml` for the existing auditpol
pattern.

### 25. `disable_wer_service` (Medium) - CONFIRMED

**Existence.** `WerSvc` is present on 26100 with display name "Windows Error Reporting Service";
`WerSvc.dll` is present in `System32`. No SCM dependents and no dependencies, so disabling it does
not cascade.

**Duplicate check.** Genuinely absent from the corpus. `services.yaml` has
`task_wer_queuereporting`, which targets the `\Microsoft\Windows\Windows Error Reporting\QueueReporting`
scheduled task, and its info copy already tells the user this "complements disabling the Windows
Error Reporting service (`WerSvc`)" and calls it "a natural pairing". So the corpus is already
pointing at a control it does not ship. This is a clean gap.

**One correction.** "stops WER consuming CPU on a crash" is an unmeasured performance claim in a
services proposal. Drop it. The honest benefit is that crash reports stop being generated and
uploaded, which is a privacy and noise reduction, not a speed improvement.

**Unstated cost worth adding.** With `WerSvc` disabled, `Get-WindowsErrorReporting`, the Reliability
Monitor's crash history and the "Windows has recovered from an unexpected shutdown" dialogs go quiet.
Some third-party crash handlers query WER state. None of that breaks anything, but a user debugging
an unstable machine has lost their first diagnostic.

**Unresolved.** Shipped default start type. The proposal says Manual; not confirmed from an
authoritative source here.

**Sources.** Live service presence and SCM dependency graph on 26100 (tier A for existence only);
`src-tauri/tweaks/services.yaml` (`task_wer_queuereporting`) for dedupe and for the corpus's own
cross-reference.

### 26. `spooler_remote_rpc_off` (Medium) - CORRECTED

**The refutation: the primary value is under the wrong key.** The proposal puts
`RegisterSpoolerRemoteRpcEndPoint` under `HKLM\SYSTEM\CurrentControlSet\Control\Print`. The shipped
`Printing2.admx` on 26100 says:

```xml
<policy class="Machine" name="RegisterSpoolerRemoteRpcEndPoint"
        displayName="$(string.RegisterSpoolerRemoteRpcEndPoint)"
        key="Software\Policies\Microsoft\Windows NT\Printers"
        valueName="RegisterSpoolerRemoteRpcEndPoint">
  <supportedOn ref="windows:SUPPORTED_WindowsNET"/>
  <enabledValue><decimal value="1"/></enabledValue>
  <disabledValue><decimal value="2"/></disabledValue>
</policy>
```

The key is `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers`. As authored, the effect would
write a DWORD into `Control\Print` that the spooler policy path does not read, the probe would then
report the tweak applied, and nothing would actually change. That is a "did-it-work" contract
violation of exactly the kind the project rules call out.

The **value** is right: the policy display name is "Allow Print Spooler to accept client connections",
enabled writes `1` (accept), disabled writes `2` (do not accept). Hardened `2` is correct.

The ADML explain text confirms the semantics and adds a detail the proposal omits:

> When the policy is disabled, the spooler will not accept client connections nor allow users to
> share printers. **All printers currently shared will continue to be shared.** The spooler must be
> restarted for changes to this policy to take effect.

So already-shared printers keep being shared, which weakens the "closes the remote surface" claim
somewhat and should be in the copy.

**The second value is correct.** `Printing.admx`, policy `ConfigureRpcAuthnLevelPrivacyEnabled`,
class Machine, key `System\CurrentControlSet\Control\Print`, valueName `RpcAuthnLevelPrivacyEnabled`,
enabled `1` / disabled `0`. The proposal's key and value for this one are right.

**Duplicate check.** Neither value is in the corpus. `disable_print_spooler` (services.yaml) and
`printnightmare_point_and_print` (security.yaml) cover different surfaces, so the proposal's "middle
option" framing is fair.

**Corrected entry.**

| Key | Value name | Type | Hardened | Stock default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers` | `RegisterSpoolerRemoteRpcEndPoint` | REG_DWORD | `2` (do not accept client connections) | value-absent, effective accept |
| `HKLM\SYSTEM\CurrentControlSet\Control\Print` | `RpcAuthnLevelPrivacyEnabled` | REG_DWORD | `1` | value-absent |

Requires a spooler restart, which the proposal states.

**Sources.** `C:\Windows\PolicyDefinitions\Printing2.admx` and `Printing.admx` plus
`en-US\Printing2.adml` (26100), verbatim (tier A).

### 27. `block_insider_builds_policy` (Medium) - CORRECTED

**The refutation: both value names and both values are wrong.** The proposal asks for
`ManagePreviewBuilds` = `1` and `ManagePreviewBuildsPolicyValue` = `0`. The shipped
`WindowsUpdate.admx` on 26100 defines exactly one policy here, and it writes exactly one flag value:

```xml
<policy name="ManagePreviewBuilds" class="Machine"
        key="Software\Policies\Microsoft\Windows\WindowsUpdate"
        valueName="ManagePreviewBuildsPolicyValue">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_RS3" />
  <enabledValue><decimal value="2" /></enabledValue>
  <disabledValue><decimal value="1" /></disabledValue>
  <elements>
    <enum id="BranchReadinessLevelId" valueName="BranchReadinessLevel" required="true">
      ... Dev 2, Beta 4, Release Preview 8, Release Preview quality-only 64
    </enum>
  </elements>
</policy>
```

Three separate errors:

1. There is **no `ManagePreviewBuilds` DWORD** written by Group Policy. The policy is *named*
   `ManagePreviewBuilds`; the value it writes is `ManagePreviewBuildsPolicyValue`.
2. `ManagePreviewBuildsPolicyValue` = `0` is not a value the ADMX ever writes. Disabled writes `1`,
   enabled writes `2`.
3. The companion element is `BranchReadinessLevel`, not a second preview-builds value.

Microsoft's Policy CSP for `Update/ManagePreviewBuilds` documents the setting's allowed values as
`0` Disable Preview builds, `1` Disable Preview builds once the next release is public, `2` Enable
Preview builds, `3` (Default) left to user selection, with Registry Key Name
`Software\Policies\Microsoft\Windows\WindowsUpdate`, and confirms the edition scope: Pro, Enterprise,
Education, IoT Enterprise and IoT Enterprise LTSC. **Not Home.** The proposal does not state an
edition gate; it needs one.

**Corrected entry.**

| Key | Value name | Type | Hardened | Stock default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `ManagePreviewBuildsPolicyValue` | REG_DWORD | `1` (policy Disabled: block preview builds) | value-absent |

Edition gate: Pro and above, not Home. `SUPPORTED_Windows_10_0_RS3` so both target platforms qualify.

**Duplicate check.** Neither value is in the corpus. The corpus's `disable_windows_insider`
(`wisvc`) is a different lever, and the proposal's argument for keeping both is sound.

**Unresolved.** The CSP names the setting `ManagePreviewBuilds` with a different numeric scale
(`0` to `3`) than the ADMX writes into `ManagePreviewBuildsPolicyValue` (`1` or `2`). Whether the
update stack reads a literal `ManagePreviewBuilds` value when written by MDM, in addition to
`ManagePreviewBuildsPolicyValue`, was not established. Ship the ADMX-backed value; do not guess the
other.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` (26100), policy `ManagePreviewBuilds`,
verbatim (tier A); Policy CSP Update, `ManagePreviewBuilds` (tier A).

### 28. `autoplay_non_volume` (Medium) - CONFIRMED

Confirmed verbatim in `AutoPlay.admx` on 26100:

```xml
<policy name="NoAutoplayfornonVolume" class="Both"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="NoAutoplayfornonVolume">
  <supportedOn ref="windows:SUPPORTED_Windows7" />
  <enabledValue><decimal value="1" /></enabledValue>
```

Key, value name, type, polarity and hardened `1` all correct. `SUPPORTED_Windows7`, so no gate.

**Duplicate check.** The corpus's `disable_autorun` (security.yaml) writes `NoAutorun` and
`NoDriveTypeAutoRun` only. `NoAutoplayfornonVolume` is genuinely absent, and the proposal's own
recommendation to add it as an effect on `disable_autorun` rather than shipping a separate tweak is
the right call: three values, one concept, one tweak.

**One correction, and it is the same class as 22.** The class is **Both**. The proposal says
"consider writing HKCU as well". Make that mandatory rather than optional: MTP device handling is
driven from the shell, and the per-user policy is the one users actually have. Write both hives and
revert both.

**Sources.** `C:\Windows\PolicyDefinitions\AutoPlay.admx` (26100), verbatim (tier A); DISA STIG for
Windows 11 V2R2 (tier B); `src-tauri/tweaks/security.yaml` for dedupe.

### 29. `remote_uac_token_filter` (Medium) - CONFIRMED

Microsoft's "User Account Control and remote restrictions" article gives the value semantics
directly:

| Value | Microsoft's description |
|---|---|
| `0` | This value builds a filtered token. **It's the default value.** The administrator credentials are removed. |
| `1` | This value builds an elevated token. |

Key, value name, type, hardened `0` and "effective default `0`" in the proposal all match. Not in any
shipped ADMX (verified), consistent with it being a direct registry setting.

**Duplicate check.** Not in the corpus. But there is a near-miss the proposal does not flag:
`security.yaml` ships `filter_admin_token`, which writes `FilterAdministratorToken` in the **same
key** (`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`). Two similarly named values
in one key, controlling adjacent behaviour (built-in Administrator elevation versus network logon
token filtering). If both ship, the info copy on each must say what the other does, or users will
assume one is a duplicate of the other and pick arbitrarily.

**The proposal's own strongest evidence stands.** It found the value set to `1` on the research
machine. That is precisely the "pin the safe state because something will have flipped it" case, and
it survives the "already the default" objection cleanly, because the point is that on real machines
it frequently is not the default.

**Risk statement is accurate.** Setting `0` does break remote `C$`, remote WMI and remote MMC using a
local admin account. That is intended and is correctly disclosed.

**Sources.** Microsoft Learn "User Account Control and remote restrictions", the UAC remote settings
table, fetched live (tier A); all-ADMX scan on 26100 (tier A, negative result);
`src-tauri/tweaks/security.yaml` (`filter_admin_token`).

### 30. `disable_internet_connection_sharing` (Medium) - CONFIRMED, with a wrong risk statement

**Existence.** `SharedAccess` is present on 26100 with display name "Internet Connection Sharing
(ICS)"; `ipnathlp.dll` is present in `System32`. It depends on `BFE` and has no SCM dependents.

**The policy value checks out.** `NetworkConnections.admx` on 26100:

```xml
<policy name="NC_ShowSharedAccessUI" class="Machine"
        key="Software\Policies\Microsoft\Windows\Network Connections"
        valueName="NC_ShowSharedAccessUI">
  <supportedOn ref="windows:SUPPORTED_WindowsXP" />
  <enabledValue><decimal value="0" /></enabledValue>
  <disabledValue><decimal value="1" /></disabledValue>
```

Note the inverted-looking polarity: the policy is "Prohibit use of Internet Connection Sharing",
and *enabling* it writes `0`. The proposal's hardened value `0` is correct, which is easy to get
wrong. Not in the corpus.

**The refutation is on the risk statement, and it matters.** The proposal says:

> `SharedAccess` also backs **Mobile Hotspot**. Disabling it removes the Mobile Hotspot feature.
> That is a user-visible loss and must be in the info copy.

The SCM dependency graph on 26100 does not support that as stated: `icssvc` ("Windows Mobile Hotspot
Service") depends on `RpcSs` and `wcmsvc`, **not** on `SharedAccess`. Mobile Hotspot may still fail
at runtime because it uses the same NAT engine, but the proposal asserts a service dependency that
does not exist and should not be repeated as fact without a test.

**Meanwhile the breakage that actually bites is missing.** `SharedAccess` / `ipnathlp` is the NAT
engine behind the **Hyper-V Default Switch** and therefore behind **WSL2 networking** and Windows
Sandbox networking. Disabling ICS is a well-known way to break WSL2's internet access. On a developer
machine that is a much more likely and much more confusing failure than losing Mobile Hotspot, and
the proposal never mentions it. Any shipped copy must lead with it.

**Unresolved.** Shipped default start type (proposal says Manual, not confirmed here).

**Sources.** Live service presence and SCM dependency graph on 26100 (tier A for existence and
dependencies only); `C:\Windows\PolicyDefinitions\NetworkConnections.admx` (26100), verbatim
(tier A); DISA STIG for Windows 11 V2R2 (tier B).

### 31. `reserved_storage_off` (Medium) - CONFIRMED, with the framing corrected

**Mechanism confirmed.** Microsoft Learn documents `Set-WindowsReservedStorageState` in the DISM
PowerShell module, with `-State` taking "either Disabled or Enabled", and states:

> This command line option is only supported for online Windows images. If reserved storage is in
> use, it may not be disabled, and the following error is returned: *This operation is not supported
> when reserved storage is in use. Please wait for any servicing operations to complete and then try
> again later.*

That confirms both the cmdlet the proposal names and the failure mode it flags. The paired
`Get-WindowsReservedStorageState` gives a real probe. Nothing in the corpus touches reserved storage.

**Corrections.**

1. **The "roughly 7 GB" figure is unsourced.** It does not appear on the cmdlet page. Reserved
   storage size varies with installed language packs and optional features, and Microsoft's own
   messaging has always described it as variable. State it as "typically several GB, varies by
   configuration" and read the actual figure from the system rather than promising a number.
2. **A machine may never have had reserved storage at all.** Microsoft's own documentation notes
   reserved storage is enabled automatically on new PCs with 1903 preinstalled and on clean
   installs, and is **not** enabled when upgrading from an earlier version. So on a large population
   of machines this tweak reclaims nothing and the probe must report "not applicable" rather than
   claiming success. The proposal says "probe rather than assume", which is right, but it frames the
   already-disabled case as an LTSC quirk when it is in fact a common upgrade-path outcome.
3. **Category.** This is a disk-space control filed under `performance`. The proposal says so itself
   ("This is a disk-space tweak, not a speed tweak"). Good. That honesty has to survive into the
   shipped copy, because everything else in `performance.yaml` promises speed.

**Sources.** Microsoft Learn `Set-WindowsReservedStorageState` (DISM module), Description and
`-State` sections, fetched live (tier A); Microsoft Learn "What's new in Windows 10 Enterprise LTSC
2021", Reserved storage section, for the clean-install-only enablement (tier A).

### 32. `firewall_logging_and_merge` (Medium) - CONFIRMED

**Logging values confirmed.** `WindowsFirewall.admx` on 26100, policies `WF_Logging_Name_1` through
`_3` (one per profile), each class Machine with key
`SOFTWARE\Policies\Microsoft\WindowsFirewall\<Profile>\Logging` and these elements:

```xml
<boolean id="WF_Logging_LogDroppedPackets"       valueName="LogDroppedPackets">        <trueValue><decimal value="1"/></trueValue><falseValue><decimal value="0"/></falseValue></boolean>
<boolean id="WF_Logging_LogSuccessfulConnections" valueName="LogSuccessfulConnections"> ... </boolean>
<text    id="WF_Logging_LogFilePathAndName"      valueName="LogFilePath" required="true" />
<decimal id="WF_Logging_SizeLimit"               valueName="LogFileSize" required="true" minValue="128" maxValue="32767" />
```

Value names, types (`LogFilePath` is text so `REG_SZ`, the rest `REG_DWORD`) and the proposal's
`16384` are all correct; 16384 is inside the 128 to 32767 range.

**`AllowLocalPolicyMerge` is real but not ADMX-backed.** Confirmed absent from all 218 shipped ADMX
files, which matches the proposal's own statement. Microsoft's Firewall CSP documents it per profile
(`MdmStore/DomainProfile/AllowLocalPolicyMerge` and the Private and Public equivalents) with
**Default Value: true**, "If this value is false, firewall rules from the local store are ignored and
not enforced", scope Device, Windows 10 1709 and later, editions Pro, Enterprise, Education, IoT
Enterprise and IoT Enterprise LTSC. That confirms the semantics and the default the proposal claims.
The registry path is corroborated by the corpus itself: `firewall_all_profiles` already writes
`EnableFirewall` and `DefaultInboundAction` under
`HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\{DomainProfile,StandardProfile,PublicProfile}`, so
`AllowLocalPolicyMerge` as a sibling value in the same profile key is consistent with the corpus's
own working mechanism plus the CIS check text.

**Corrections.**

1. `LogFilePath` and `LogFileSize` are `required="true"` in the ADMX element set. Writing
   `LogDroppedPackets` alone leaves the policy half-configured from Group Policy tooling's point of
   view. Write all three together and revert all three.
2. The ADMX `disabledList` writes `LogDroppedPackets` and `LogSuccessfulConnections` as
   `<string>0</string>`, while the enabled path writes decimals. Do not copy the string form; the
   effective values are `REG_DWORD`.
3. The proposal's assessment of `AllowLocalPolicyMerge` = 0 as "the aggressive part" is right, and
   the failure mode should be stated more bluntly: on a standalone machine there is no policy-pushed
   rule set to fall back to, so turning merge off on the Public profile means inbound simply stops
   working there, silently, with no prompt.
4. Editions: Pro and above per the CSP. Home is out of scope for `AllowLocalPolicyMerge`.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsFirewall.admx` (26100), `WF_Logging_Name_1`
verbatim (tier A); Microsoft Firewall CSP, `MdmStore/<Profile>/AllowLocalPolicyMerge`, fetched live
(tier A); CIS Windows 11 v4.0.0 (tier B); `src-tauri/tweaks/security.yaml` (`firewall_all_profiles`)
for the registry path corroboration.

### 33. `enable_sehop` (Low) - CONFIRMED

**Mechanism.** `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel`,
`DisableExceptionChainValidation`, `REG_DWORD`, hardened `0`. The name is inverted and the proposal
handles it correctly: `0` means SEHOP is **enabled**. Confirmed absent from every shipped ADMX
(verified across all 218), which matches the proposal's statement that this is a direct write per the
STIG check text rather than an administrative template.

**Attack on the benefit.** The proposal's own Low rating is if anything generous. SEHOP validates the
structured exception handler chain, and x64 processes do not use an SEH chain at all: exception
handling on x64 is table-driven through `.pdata` / `.xdata`. So on a 26100 x64 machine this affects
only 32-bit processes, and it is already on. The delta is an auditable registry value and nothing
else. The proposal says this. It survives inclusion under the corpus's stated principle (a real
control exists, the user gets to pin it), but the copy must not imply any behavioural change.

**Unresolved sourcing.** No reachable tier A page. Microsoft's original SEHOP KB (956607) is retired
and the current Learn troubleshooting URL returned 404 during this pass. The control rests on the
DISA STIG check text (tier B) plus long-standing community corroboration. That clears the bar, but
label it: this one is benchmark-sourced, not Microsoft-documented, on today's web.

Requires reboot. Revert is value-absent.

**Sources.** DISA STIG for Windows 11 V2R2, SEHOP rule (tier B); all-ADMX scan on 26100 (tier A,
negative result); privacy.sexy (tier C, corroboration).

### 34. `no_index_encrypted_files` (Low) - CORRECTED

**The refutation is about sourcing, and it upgrades the proposal.** The proposal states:

> Not present in the shipped ADMX under that value name; written directly per the STIG check text.

That is wrong. It **is** in the shipped ADMX on 26100:

```xml
<policy name="AllowIndexingEncryptedStoresOrItems" class="Machine"
        clientExtension="{7933F41E-56F8-41d6-A31C-4148A711EE93}"
        key="SOFTWARE\Policies\Microsoft\Windows\Windows Search"
        valueName="AllowIndexingEncryptedStoresOrItems">
  <supportedOn ref="VistaOr4" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

The reason it looks absent is the encoding trap noted at the top of this document: `Search.admx`
ships as **UTF-16LE** while almost every other ADMX is UTF-8, so a byte-level UTF-8 grep misses every
string in it. This should be corrected in the source document, because "not in the ADMX" is used
there as a signal of lower confidence and the opposite is true.

**Everything else is right, including the default.** The shipped `Search.adml` explain text confirms
the proposal's stock-default claim from the horse's mouth:

> This policy setting is not configured by default. If you do not configure this policy setting, the
> local setting, configured through Control Panel, will be used. **By default, the Control Panel
> setting is set to not index encrypted content.**

So: key, value name, `REG_DWORD`, hardened `0`, revert value-absent, and effective default already
not-indexing. All correct.

**The proposal's own no-op caveat is right.** `performance.yaml` ships `disable_search_indexing`
(`WSearch`). Anyone who applied that has no index at all, and this value becomes moot. Say so.

Also note the ADML warning, which the proposal omits and which is worth one line: enabling or
disabling this setting causes the index to be **rebuilt completely**.

**Sources.** `C:\Windows\PolicyDefinitions\Search.admx` (26100, UTF-16LE), policy
`AllowIndexingEncryptedStoresOrItems`, verbatim, and `en-US\Search.adml`
`ExplainAllowIndexingEncryptedStoresOrItems` (tier A); `src-tauri/tweaks/performance.yaml`
(`disable_search_indexing`).

### 35. `disable_pku2u_online_id` (Low) - CORRECTED

**The refutation: the stock default is very likely the opposite of what the proposal says.** The
proposal states "value-absent, effective disabled" and rates it Low on the grounds that "the default
already matches". Microsoft's Security Policy Settings page for this setting says:

> This policy is **enabled by default in Windows 10, Version 1607, and later.**

and its Default values table gives "Stand-alone server default settings: Not defined", "Member server
effective default settings: **Disabled**", "Domain controller effective default settings: Disabled".
The "enabled by default in 1607 and later" statement is the one that applies to a standalone Windows
11 client, which is this corpus's target. So on 26100 the effective default is **enabled**, and this
tweak actually turns something off rather than pinning a matching default.

That cuts two ways. The tweak is worth more than the proposal claims, because it is a real
behavioural change. It is also riskier than the proposal claims, for the same reason. Microsoft's own
Potential impact section says:

> Some roles/features (such as Failover Clustering) don't utilize a domain account for its PKU2U
> authentication and will cease to function properly when disabling this policy.

Peer-to-peer authentication between non-domain machines, which is exactly the workgroup scenario a
consumer is in, is what PKU2U is for.

**Mechanism.** `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\pku2u`, `AllowOnlineID`, `REG_DWORD`,
hardened `0`. Confirmed absent from every shipped ADMX (verified), consistent with it being a
Security Options setting. Microsoft's page documents the policy and its semantics but not the
registry path; the path is DISA STIG and CIS check text (tier B). Not in the corpus.

**Corrected entry.**

| Key | Value name | Type | Hardened | Stock default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\pku2u` | `AllowOnlineID` | REG_DWORD | `0` | value-absent; **effective default on a standalone Windows 11 client is enabled** per Microsoft |

Revert stays value-absent. The risk level should rise above "Low" and the copy should name the
peer-to-peer sharing scenarios explicitly.

**Sources.** Microsoft Learn "Network security: Allow PKU2U authentication requests to this computer
to use online identities", the default-values table, the "enabled by default in Windows 10, Version
1607, and later" statement, and the Potential impact section, all fetched live (tier A); DISA STIG
for Windows 11 V2R2 (tier B) for the registry path; all-ADMX scan on 26100 (tier A, negative result).

### 36. `mss_network_stack_hardening` (Low) - REJECTED

The mechanism is not in doubt. All five value names are the standard MSS (legacy) set from the
Microsoft Security Compliance Toolkit, and all five were confirmed absent from every shipped ADMX on
26100, which is exactly what the proposal predicts (they arrive with `MSS-legacy.admx`, which is not
part of the OS). Four are individual DISA STIG rules. The proposal's own insistence that these must
not be confused with the rejected "TCP registry pack" performance myths is correct and well made.

It is rejected on two grounds.

**1. The entry supplies no stock defaults, and for at least two of the five that is a live
revert-correctness hazard.** The table has a "Hardened" column and no "Stock default" column at all.
The prose says only "All verified absent on 26100.4061", read off a machine the source document
itself declares non-pristine. `EnableICMPRedirect` and `PerformRouterDiscovery` are not
absent-by-nature settings in the way a policy value is; they are TCP/IP stack parameters with
documented non-zero effective behaviour. Shipping a "Windows default (Stock Default)" option that
deletes all five, without having established on a clean image that all five are genuinely absent
there, is precisely the `_harmful-revert.md` defect class. Establish the defaults first.

**2. There is no benefit left to deliver.** The proposal itself writes that on a consumer machine
behind NAT, "the real-world exposure to IP source routing and ICMP redirect attacks is close to nil",
and that `NoNameReleaseOnDemand` "only matters where NetBIOS is in use, which the corpus already
disables" (`disable_netbios_tcpip` in `network.yaml`). That is one member of the set self-declared as
a no-op given another corpus tweak, and four more that the author says have negligible exposure. For
a Low-ranked item competing for a corpus slot, that is not enough, and the entry's closing line
("Include them if the goal is auditable benchmark alignment; skip them if the goal is user-visible
benefit") reads as the author agreeing.

**Revive it** as an explicit, clearly-labelled "CIS and STIG benchmark alignment" bundle if the
maintainer decides benchmark auditability is a product goal, and only after each of the five stock
defaults is established from a clean image or from Microsoft's MSS documentation.

**Sources.** All-ADMX scan on 26100 (tier A, negative result, corroborating the proposal's own
statement); DISA STIG for Windows 11 V2R2, four rules (tier B); `src-tauri/tweaks/network.yaml`
(`disable_netbios_tcpip`) for the `NoNameReleaseOnDemand` overlap.

### 37. `ntfs_disable_8dot3` (Low) - REJECTED

This is a performance proposal, so it needs a measurement. It does not have one, and its single
load-bearing citation does not say what it is claimed to say.

**The refutation.** The proposal's entire justification for inclusion is one sentence:

> Microsoft's own file-server performance tuning guidance states that disabling 8.3 name creation
> improves file-creation performance in directories with large numbers of files. **That is a real,
> first-party, measured claim, which is why this is included at all.**

The cited page is
`https://learn.microsoft.com/en-us/windows-server/administration/performance-tuning/role/file-server/`.
It was fetched live during this pass and full-text scanned. It contains **zero** occurrences of
"8dot3", "8.3" or "ShortName". The claim is not on the cited page. Even if the intended source is the
retired Windows Server 2012 tuning guide, a recommendation to disable a feature is not a measurement,
and no benchmark showing a file-creation improvement on a client workload at 26100 was found.

**The mechanism itself is real and the proposal describes it accurately.** Two distinct surfaces
exist and the proposal is right about both:

- Runtime: `fsutil 8dot3name` documentation confirms
  `HKLM\System\CurrentControlSet\Control\FileSystem\NtfsDisable8dot3NameCreation` as the value
  `fsutil 8dot3name set <defaultvalue>` writes, and confirms the significance of `2`: "You must set
  the default file system behavior for 8dot3 name creation to the value 2 before you can enable or
  disable 8dot3 name creation for a specified volume", which is consistent with `2` meaning
  per-volume.
- Policy: `FileSys.admx` on 26100, policy `ShortNameCreationSettings`, class Machine, key
  `System\CurrentControlSet\Policies` (**a different key from the runtime one**), enum
  `NtfsDisable8dot3NameCreation` with `0` "Enable on all volumes", `1` "Disable on all volumes",
  `2` "Enable / disable on a per volume basis", `3` "Disable on all data volumes".

So the proposal's hardened `1` and its `2` default are internally coherent, and its statement that
existing short names are not removed is right.

**Why it is still rejected.** No measured benefit; a Low self-rating; a real compatibility cost
(16-bit installers and applications that hardcode short paths, plus the fsutil documentation's own
warning that registry keys pointing at 8.3 names can lead to "unexpected application failures,
including the inability to uninstall an application"); no effect on existing files; and it is
competing for a slot in a category whose other entries promise speed. A performance tweak that cannot
show a measurement does not belong.

**If the maintainer wants it anyway**, ship it as a filesystem hygiene or compatibility control with
no speed claim whatsoever, state clearly that the runtime key and the policy key are different, and
be explicit that the default `2` was not confirmed from a clean image here.

**Sources.** Microsoft Learn "Performance tuning for file servers", fetched live and full-text
scanned, zero matches for 8dot3 / 8.3 / ShortName (tier A, negative result); Microsoft Learn
`fsutil 8dot3name`, fetched live, the `set <defaultvalue>` and warning text (tier A);
`C:\Windows\PolicyDefinitions\FileSys.admx` and `en-US\FileSys.adml` (26100), policy
`ShortNameCreationSettings` with all four enum labels, verbatim (tier A).

### 38. `require_doh` (Low) - CORRECTED

**The refutation: two of the three enum values are swapped.** The proposal's table says
"`1` Allow DoH, `2` Prohibit DoH, `3` Require DoH". The shipped `DnsClient.admx` and its ADML on
26100 say:

```xml
<policy name="DNS_Doh" class="Machine" key="Software\Policies\Microsoft\Windows NT\DNSClient">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_20H2_SERVER_20H2" />
  <elements>
    <enum id="DNS_Doh_Box" valueName="DoHPolicy" required="true">
      <item displayName="$(string.DNS_Doh_Force)">   <value><decimal value="3" /></value></item>
      <item displayName="$(string.DNS_Doh_Auto)">    <value><decimal value="2" /></value></item>
      <item displayName="$(string.DNS_Doh_Disabled)"><value><decimal value="1" /></value></item>
    </enum>
    <enum id="DNS_Doh_Setting_Box" valueName="DohPolicySetting" required="true"> 0 Allow DoH, 1 Block DoH </enum>
    <enum id="DNS_Dot_Setting_Box" valueName="DotPolicySetting" required="true"> 0 Allow DoT, 1 Block DoT </enum>
  </elements>
</policy>
```

ADML strings: `DNS_Doh_Force` = "Require encryption", `DNS_Doh_Auto` = "Allow encryption",
`DNS_Doh_Disabled` = "Prohibit encryption". So **`1` prohibits, `2` allows, `3` requires**. The
proposal's `3` is right; its `1` and `2` are transposed. If a future dropdown offers all three
states from that table, "Allow DoH" would write the value that turns encryption off.

**Two further corrections.**

1. All three elements are `required="true"`. The policy also carries `DohPolicySetting` (0 Allow DoH,
   1 Block DoH) and `DotPolicySetting` (0 Allow DoT, 1 Block DoT). The proposal treats DoT as a
   future 25H2 addition; it is already in the 26100 ADMX. Writing `DoHPolicy` alone still works at
   the DNS client level, but the corpus should know it is writing one of three required elements and
   say so.
2. `supportedOn` is `SUPPORTED_Windows_10_0_20H2_SERVER_20H2`, so the policy is not Windows 11 only.
   The existing `dns_over_https` tweak carries `windows: { products: [11] }`, which is correct for
   `EnableAutoDoh`, but adding `DoHPolicy` as an option on that tweak would inherit a gate narrower
   than the policy's own applicability.

**The proposal's core argument survives and is good.** The corpus's `dns_over_https` writes
`EnableAutoDoh` = 2, and the corpus's own info copy for it admits "`EnableAutoDoh = 2` is
*community-documented only*, with no official Microsoft source". `DoHPolicy` is the first-party,
ADMX-backed, enforcing control, and it delivers a different guarantee (fail rather than fall back).
Adding it as an option on the existing tweak, as the proposal recommends, is the right shape. The
Medium-risk warning about total name-resolution failure is accurate and must be prominent.

**Corrected entry.**

| Key | Value name | Type | Values | Stock default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `DoHPolicy` | REG_DWORD | `1` Prohibit encryption, `2` Allow encryption, `3` Require encryption | value-absent |

**Sources.** `C:\Windows\PolicyDefinitions\DnsClient.admx` (26100), policy `DNS_Doh`, verbatim, and
`en-US\DnsClient.adml` strings `DNS_Doh_Force` / `DNS_Doh_Auto` / `DNS_Doh_Disabled` (tier A);
`src-tauri/tweaks/network.yaml` (`dns_over_https`) for the overlap and for the corpus's own admission
about `EnableAutoDoh`.

---

## Additional defects found while verifying

These are not gaps and were not in the source document's defect list. Both are live.

**V.1 `disable_compat_appraiser` (privacy.yaml) targets a task that does not exist on 26100.** Its
`task_progdata` effect points at
`\Microsoft\Windows\Application Experience\ProgramDataUpdater`. Enumeration of
`C:\Windows\System32\Tasks\Microsoft\Windows\Application Experience` on 26100 returns exactly
`MareBackup`, `Microsoft Compatibility Appraiser`, `Microsoft Compatibility Appraiser Exp`,
`PcaPatchDbTask`, `SdbinstMergeDbTask`, `StartupAppTask`. This is the same defect class as D.2 in the
source document, which flagged `task_device_census` but missed this one.

**V.2 `disable_ceip_tasks` (privacy.yaml) targets a task that does not exist on 26100.** Its
`task_kernelceip` effect points at `KernelCeipTask` under the Customer Experience Improvement Program
folder. Enumeration of that folder on 26100 returns exactly `Consolidator` and `UsbCeip`. The tweak's
info copy hedges about this in prose, but the effect still runs.

---

## Unknowns

1. **Shipped default start types for `seclogon`, `WerSvc` and `SharedAccess`.** All three services
   are confirmed present on 26100 with the expected display names and dependency graphs, but this
   machine's `Start` values cannot be used as evidence of a Windows default, and no authoritative
   Microsoft list of Windows 11 client default start types was found. Proposals 18, 25 and 30 all
   assert `Manual`. Confirm against a clean 26100 image before writing the revert option, because
   reverting a trigger-started service to the wrong start type is a real state leak.
2. **Stock-image absence of the five MSS values (proposal 36).** Asserted from a machine the source
   document itself declares non-pristine. `EnableICMPRedirect` and `PerformRouterDiscovery`
   especially need a clean-image check before any "delete the value" revert is written.
3. **`ManagePreviewBuilds` versus `ManagePreviewBuildsPolicyValue` (proposal 27).** The shipped ADMX
   writes only `ManagePreviewBuildsPolicyValue` (1 or 2). The Policy CSP documents the setting as
   `ManagePreviewBuilds` with a 0 to 3 scale. Whether the update stack also reads a literal
   `ManagePreviewBuilds` value under the same key, when written by MDM, was not established.
4. **Event log channel defaults (proposal 23).** The "20480 KB" figure was not confirmed from a
   Microsoft source during this pass. It does not affect the revert (value-absent), only the copy.
5. **Mobile Hotspot's actual runtime dependency on `SharedAccess` (proposal 30).** Refuted as an SCM
   dependency; not tested at runtime. The Hyper-V Default Switch and WSL2 dependency on `ipnathlp` is
   the more consequential one and should be confirmed by test before shipping the info copy.
6. **`AllowLocalPolicyMerge` registry path (proposal 32).** Semantics and per-profile default are
   Microsoft-documented via the Firewall CSP; the exact `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<Profile>\AllowLocalPolicyMerge`
   registry path is corroborated by CIS check text plus the corpus's own working use of the same
   profile keys, not by a Microsoft page naming that path.
7. **A measured benefit for `ntfs_disable_8dot3` (proposal 37).** None was found. If one exists at
   26100 on a client workload, it would change that verdict; nothing found during this pass supports
   the claim as written.
