# Adversarial verification: the 12 High-value proposals in `_gaps-security-performance-network.md`

Verified 2026-07-26. Target platform: Windows 11 24H2 (26100) and 25H2 (26200) primary; Windows 10 IoT
Enterprise LTSC 2021 (19044) secondary.

Posture: refutation. Each proposal was attacked on existence, key path, value name, value type, option
semantics, duplication against the shipped corpus, hidden dependencies, and lockout risk. Default was
reject under uncertainty.

## Evidence rules applied

The research machine is heavily modified and was **not** used as evidence for any Windows default.
Nothing in this document rests on a live registry read, a live service start type, or scheduled-task XML.
What was used:

- **Shipped ADMX and ADML** under `C:\Windows\PolicyDefinitions` on 26100 (tier A). Every policy block
  below was parsed out of the shipped XML, not copied from a blog. ADMX gives the authoritative key,
  value name, class, enabled and disabled values, enum members, element types, and `supportedOn` gate.
- **Presence of a value name inside a shipped binary** as proof the value exists and is read. UTF-16
  string extraction was used, because registry value names are stored as wide strings and an ASCII-only
  scan misses them. This is what settled `EnableMDNS` and `BlockNTLM`.
- **Microsoft Learn**, including the Policy CSP pages, the ASR rules reference, the SMB NTLM blocking
  page, the Kernel DMA Protection page, and the Security Policy Settings reference (tier A).
- **PowerShell's own policy reader source** (`src/System.Management.Automation/engine/Utils.cs`) to
  settle how the `ModuleNames` subkey is consumed.

Neither admx.help nor getadmx.com was used.

## Verdict summary

| # | Proposal | Verdict | Headline |
| --- | --- | --- | --- |
| 1 | `defender_cloud_protection` | CORRECTED | Values right; the stated dependency on ASR is mostly false and Tamper Protection can make the write inert |
| 2 | `smb_client_block_ntlm` | CORRECTED | The policy **is** in the shipped 26100 ADMX; a registry-only implementation is possible and preferable |
| 3 | `enhanced_phishing_protection` | CORRECTED | Work or school passwords only; three of the five values already default to the hardened state |
| 4 | `asr_standard_protection_rules` | CONFIRMED | Both GUIDs verified, REG_SZ correct, genuinely additive |
| 5 | `disable_winrm_remoting` | CONFIRMED | All six values and polarities verified; no lockout; the `Manual` default is unsourced |
| 6 | `powershell_module_transcript_logging` | CONFIRMED | Verified, including that `ModuleNames` value **names** are what PowerShell reads |
| 7 | `restrict_remote_sam` | CONFIRMED | REG_SZ SDDL confirmed by Microsoft; not a duplicate of the anonymous-enumeration tweak |
| 8 | `block_always_install_elevated` | CONFIRMED | Confirmed; the "both hives required" reasoning is backwards but harmless |
| 9 | `disable_mdns` | CORRECTED | Control is real and in shipped binaries; the `>=26100` gate is unjustified |
| 10 | `update_feature_control` | CORRECTED | The `SetAllowOptionalContent` value table is materially wrong; the enum lives in a second value |
| 11 | `defer_feature_updates` | CONFIRMED | Live in the 26100 ADMX and Policy CSP, no deprecation found; conflicts hard with `target_release_version` |
| 12 | `kernel_dma_protection` | CORRECTED | Stock default is `1`, not `2`; the proposal's own option table contradicts itself |

Counts: **5 CONFIRMED, 7 CORRECTED, 0 REJECTED, 0 UNRESOLVED at the proposal level.**

No proposal was rejected, because in every case a real control demonstrably exists on 26100, which is
the only rejection criterion. That is not a soft pass. Four of the twelve (2, 3, 10, 12) carried
mechanism, semantics, or default-state errors severe enough that shipping them as written would have
produced a broken tweak or a broken revert.

## Duplication check

Read from `src-tauri/tweaks/security.yaml` (36 tweak ids), `network.yaml` (20) and `services.yaml` (29).
Every one of the four flagged near-duplicates is genuinely additive:

| Proposal | Nearest existing tweak | Verdict |
| --- | --- | --- |
| `restrict_remote_sam` | `restrict_anonymous_enum` writes `RestrictAnonymousSAM`, `RestrictAnonymous`, `EveryoneIncludesAnonymous` under `Lsa` | Additive. `RestrictRemoteSam` is a different value, a different type (REG_SZ), and governs authenticated rather than anonymous SAMRPC callers |
| `asr_standard_protection_rules` | `asr_block_lsass_theft` plus `asr_block_office_script_vectors` cover 5 GUIDs | Additive. Neither `56a863a9` nor `e6db77e5` appears in `security.yaml` |
| `defer_feature_updates` | `defer_quality_updates` writes `DeferQualityUpdates`; `target_release_version` writes `TargetReleaseVersion` | Additive on values, but see the hard conflict noted in entry 11 |
| `disable_mdns` | `disable_llmnr` writes `EnableMulticast` in the same key; `disable_netbios_tcpip` writes per-adapter `NetbiosOptions` | Additive. Three different protocols, three different values |

Also checked and clear: no `Spynet` or `MpEngine` value, no `WTDS` value, no `WinRM` value, no
`AlwaysInstallElevated`, no `ModuleLogging` or `Transcription` value, no `AllowOptionalContent` or
`AllowTemporaryEnterpriseFeatureControl`, no `DeviceEnumerationPolicy`, no `DisableExternalDMAUnderLock`,
and no `BlockNTLM` anywhere in the three files.

One partial overlap worth a line of info copy rather than a merge: `update_feature_control` disabling
optional content also suppresses optional driver updates, which the existing
`exclude_wu_driver_updates` addresses from a different angle with a different value.

---

## 1. `defender_cloud_protection`: CORRECTED

### What survives

All five values are confirmed in the shipped `WindowsDefender.admx` on 26100, class Machine, all
`REG_DWORD`, at exactly the paths proposed:

| Policy name in ADMX | Key | Value name | Evidence |
| --- | --- | --- | --- |
| `SpynetReporting` | `Software\Policies\Microsoft\Windows Defender\Spynet` | `SpynetReporting` | enum: 0 Disabled, 1 Basic, 2 Advanced |
| `SubmitSamplesConsent` | same | `SubmitSamplesConsent` | enum: 0 Always Prompt, 1 Send Safe, 2 Never Send, 3 Send All |
| `DisableBlockAtFirstSeen` | same | `DisableBlockAtFirstSeen` | enabledValue 0, disabledValue 1 |
| `MpEngine_MpCloudBlockLevel` | `Software\Policies\Microsoft\Windows Defender\MpEngine` | `MpCloudBlockLevel` | enum: 0 Default, 1 Moderate, 2 High, 4 High Plus, 6 Zero Tolerance |
| `MpEngine_MpBafsExtendedTimeout` | same | `MpBafsExtendedTimeout` | decimal element, `minValue="0" maxValue="50"` |

Every value name, type, key path and enum member in the proposal matches the shipped ADMX exactly. The
proposed `MpBafsExtendedTimeout` = 50 sits precisely on the ADMX maximum.

### Correction A: the dependency claim is mostly false

The proposal justifies itself partly on "the corpus enables Controlled Folder Access, Network Protection,
PUA protection and five ASR rules, but never turns on the cloud protection those features depend on."
Checked against the ASR rules reference, per rule:

- `d4f940ab` Block all Office applications from creating child processes: **Dependencies: Microsoft
  Defender Antivirus**. No cloud dependency.
- `3b576869`, `5beb7efe`, `be9ba2d9`: same, Defender Antivirus only.
- The LSASS rule: **Dependencies: Microsoft Defender Antivirus**. No cloud dependency.

The ASR rules Microsoft actually documents as cloud-gated are the prevalence/age rule and the advanced
ransomware rule, **neither of which the corpus ships**. The cloud-block-level caveat about notifications
(EDR alerts only at High plus or Zero tolerance, user pop-ups only at High or above) attaches to rules
such as Block Adobe Reader from creating child processes, which the corpus also does not ship.

Controlled Folder Access requires real-time protection, not cloud protection.

The one dependency that **does** hold: Microsoft's network protection requirements table states that on
Windows 10 1709 or later and Windows 11, real-time protection, behavior monitoring **and cloud-delivered
protection** must all be enabled and active. So `enable_network_protection` in the corpus genuinely
depends on this. Restate the justification around that single tweak and drop the ASR framing.

### Correction B: Tamper Protection can make this inert

Microsoft's tamper protection page lists "Cloud protection remains enabled" among the settings that
cannot be changed when tamper protection is on, and states plainly: "If you're using Group Policy to
manage Microsoft Defender Antivirus settings, keep in mind that any changes made to tamper-protected
settings are ignored."

Consequences the tweak must handle:

- Tamper Protection is on by default on consumer Windows 11, which means cloud protection is **already
  forced on**. The tweak's headline claim ("turn on the cloud protection those features depend on") is
  already satisfied on a stock 26100 box.
- `SpynetReporting` and `DisableBlockAtFirstSeen` sit directly on that tamper-protected surface, so the
  registry write can land while the effective setting does not move.
- `MpCloudBlockLevel` and `MpBafsExtendedTimeout` are **not** in the tamper-protected list, so they are
  the values that actually deliver new behaviour.
- Therefore the probe must not report success from the registry value alone. It should read
  `Get-MpPreference` / `Get-MpComputerStatus`, or the tweak should carry `skip_validation`.

### Correction C: the second proposed option is self-contradictory

The proposal's option 2, "Cloud protection on, no sample submission", sets `SubmitSamplesConsent` = 2
(Never Send) while keeping `DisableBlockAtFirstSeen` = 0. Microsoft states: "the `NeverSend` setting
means that the Block at First Sight feature of Microsoft Defender for Endpoint won't work." The option
therefore advertises Block at First Sight and disables it in the same breath. Either rename the option
to make the trade explicit or use `SubmitSamplesConsent` = 0 (Always Prompt), which also lowers
protection but at least leaves the user in the loop.

### Correction D: effective defaults

`SubmitSamplesConsent` effective default is `SendSafeSamples`, which Microsoft calls "the default,
recommended setting". The proposed hardened value of 1 is therefore the same as the default. That is not
a reason to exclude it, but Drawbacks must say so rather than implying a change.

### Other applicability

Inert where a third-party AV has put Defender in passive mode and on images with Defender removed, as the
proposal says. The proposal's suggested gate is sound.

### Sources

1. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policies `SpynetReporting`,
   `SubmitSamplesConsent`, `DisableBlockAtFirstSeen`, `MpEngine_MpCloudBlockLevel`,
   `MpEngine_MpBafsExtendedTimeout` (tier A)
2. Attack surface reduction (ASR) rules reference, per-rule Dependencies fields,
   <https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference> (tier A)
3. Use network protection, Requirements section,
   <https://learn.microsoft.com/en-us/defender-endpoint/network-protection> (tier A)
4. Turn on cloud protection in Microsoft Defender Antivirus,
   <https://learn.microsoft.com/en-us/defender-endpoint/enable-cloud-protection-microsoft-defender-antivirus> (tier A)
5. Protect security settings with tamper protection,
   <https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection> (tier A)

---

## 2. `smb_client_block_ntlm`: CORRECTED

### The proposal's central factual claim is wrong

The proposal states: "Note that `LanmanWorkstation.admx` as shipped on 26100 does **not** contain this
policy; it arrived with the Windows Server 2025 ADMX. Use the cmdlet, not a guessed registry value."

That is false. The shipped 26100 `LanmanWorkstation.admx` contains, verbatim:

```xml
<policy
    class="Machine"
    displayName="$(string.Pol_BlockNTLM_Name)"
    explainText="$(string.Pol_BlockNTLM_Help)"
    key="Software\Policies\Microsoft\Windows\LanmanWorkstation"
    name="Pol_BlockNTLM"
    valueName="BlockNTLM"
    >
  <parentCategory ref="Cat_LanmanWorkstation"/>
  <supportedOn ref="SUPPORTED_Windows_Server_2025_Windows_11_0" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

The companion `Pol_BlockNTLMServerExceptionList` policy is present in the same file, writing
`BlockNTLMServerExceptionList` in the same key via a `multiText` element, which is `REG_MULTI_SZ`. The
ADML resolves `SUPPORTED_Windows_Server_2025_Windows_11_0` to "At least Windows Server 2025, Windows 11",
and the `Pol_BlockNTLM_Help` string reads: "This policy controls if the SMB client will block NTLM for
remote connection authentication. If you enable this policy setting, the SMB client won't use NTLM for
remote connection authentication."

Microsoft Learn's own page documents the Group Policy route first: Computer Configuration >
Administrative Templates > Network > Lanman Workstation > "Block NTLM (LM, NTLM, NTLMv2)" > Enabled.

### What the cmdlet actually writes

The WMI provider behind `Set-SmbClientConfiguration` is `smbwmiv2.dll`. Its shipped 26100 binary contains
these UTF-16 strings:

```
BlockNTLM
BlockNTLMServerExceptionList
GetBoolRegistryValue(BlockNTLM)
SetBoolRegistryValue(BlockNTLM)
SmbResetBooleanPropertyToDefault(BlockNTLM)
MSFT_SmbClientConfiguration_Set_BlockNTLM
LanmanWorkstationParameters
Software\Policies\Microsoft\Windows\LanmanWorkstation
```

Read together with `wkssvc.dll`, which carries `System\CurrentControlSet\Services\LanmanWorkstation\Parameters`
and the strings `BlockNTLMInfo` and "BlockNTLM params specified by user", and `mrxsmb20.sys`, which
carries `qBlockNTLMFromGlobalSetting` and `qBlockNTLMFromNetUseSetting`:

- The cmdlet writes a `REG_DWORD` named `BlockNTLM` under
  `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters`.
- The provider also consults the policy key `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation`,
  which is the Group Policy override.
- `SmbResetBooleanPropertyToDefault(BlockNTLM)` confirms a first-class reset path, so the change is
  cleanly reversible by deleting the value, not only by writing 0.
- The driver distinguishes a global setting from a per-mapping (`net use`) setting, which corroborates the
  proposal's note about `New-SmbMapping -BlockNTLM $false`.

### Corrected mechanism

Implement as registry, not as a PowerShell action:

| Key | Value name | Type | Block NTLM | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation` | `BlockNTLM` | REG_DWORD | `1` | value-absent |

Optional companion, same key, `BlockNTLMServerExceptionList`, `REG_MULTI_SZ`, one server name per entry.

The non-policy location `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters\BlockNTLM`
is what the cmdlet writes and is an equally valid target; prefer the policy key because it is the
documented Group Policy surface and it wins over the service key.

Reverting by deleting the policy value restores stock behaviour. If the corpus instead writes 0, note
that 0 is the ADMX `disabledValue`, which is semantically "policy explicitly says allow NTLM" rather than
"unconfigured"; for a clean revert, delete.

### What survives unchanged

- Applicability `windows: { build: ">=26100" }` is correct. Learn: "Starting with Windows Server 2025 and
  Windows 11, version 24H2." Not available on LTSC 2021.
- The risk assessment is accurate: this breaks SMB to any server that cannot do Kerberos, including NAS
  reached by IP address, workgroup shares, and scan-to-folder printers.
- No lockout risk. This is outbound SMB client behaviour only. It cannot lock a user out of the machine.

### Sources

1. `C:\Windows\PolicyDefinitions\LanmanWorkstation.admx` and `en-US\LanmanWorkstation.adml` (26100),
   policies `Pol_BlockNTLM` and `Pol_BlockNTLMServerExceptionList` (tier A)
2. Block NTLM connections on SMB, Group Policy tab,
   <https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-ntlm-blocking> (tier A)
3. UTF-16 string extraction from shipped 26100 `smbwmiv2.dll`, `wkssvc.dll`, `mrxsmb20.sys`,
   `mrxsmb.sys` (tier A, shipped binary)

---

## 3. `enhanced_phishing_protection`: CORRECTED

### What survives

All five values confirmed in the shipped `WebThreatDefense.admx` on 26100: key
`Software\Policies\Microsoft\Windows\WTDS\Components`, class Machine, all `REG_DWORD`, all with
`enabledValue` 1 and `disabledValue` 0, all gated `supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"`.
The value names `ServiceEnabled`, `NotifyMalicious`, `NotifyPasswordReuse`, `NotifyUnsafeApp` and
`CaptureThreatWindow` are exact.

One naming note for the author guide: the policy that writes `CaptureThreatWindow` is named
`AutomaticDataCollection` in the ADMX, not `CaptureThreatWindow`.

Applicability `windows: { products: [11] }` is right. Policy CSP gives Windows 11 22H2 (10.0.22621) and
later, editions Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC.

### Correction A: the feature only covers work or school passwords

The proposal describes this as warning "when a Windows password is typed into a phishing site, reused on
a website, or stored in an unsafe app such as Notepad." Microsoft's scope is narrower and stated
repeatedly:

- "Enhanced Phishing Protection in Microsoft Defender SmartScreen helps protect Microsoft **school or
  work** passwords against phishing and unsafe usage on sites and apps."
- Notify Password Reuse: "warns users if they reuse their **work or school** password."
- Notify Unsafe App: "notifications when users type their **work or school** passwords in Notepad and
  Microsoft 365 Office Apps."
- Automatic Data Collection: triggers "when your users enter their **work or school** password."

On a consumer machine signed in with a local account or a personal Microsoft account, this tweak is
largely inert. It is still includable, because a real control exists and the corpus's own inclusion rule
does not judge the default. But the description must not promise protection this feature does not
deliver to the target audience.

### Correction B: three of the five values already default to the hardened state

Microsoft publishes the defaults directly. Corrected default column:

| Value name | Registry default | Effective default on a consumer 26100 | Proposal's hardened value | Real delta |
| --- | --- | --- | --- | --- |
| `ServiceEnabled` | value-absent | **Enabled** (Policy CSP "Default Value: 1") | 1 | none, plus it locks users out of turning it off |
| `NotifyMalicious` | value-absent | **Enabled** for all devices not onboarded to MDE | 1 | none on a consumer box |
| `NotifyPasswordReuse` | value-absent | **Disabled** | 1 | real |
| `NotifyUnsafeApp` | value-absent | **Disabled** | 1 | real |
| `CaptureThreatWindow` | value-absent | **Disabled** for anything not domain-joined or MDM-enrolled | 1 | real, and it is a privacy cost, not a gain |

The proposal's flat "value-absent (feature ships in audit mode)" for all five is registry-true but
behaviourally misleading. The genuine additions on a consumer 26100 are `NotifyPasswordReuse` and
`NotifyUnsafeApp`.

### Correction C: `ServiceEnabled` = 1 means audit mode, not "on with warnings"

The shipped ADML is explicit: "If you enable this policy setting, Enhanced Phishing Protection in
Microsoft Defender SmartScreen is enabled **in audit mode** and your users are unable to turn it off."
Setting it to 1 does not enable warnings. The three `Notify*` values do that. The proposal's option
labelled "Enabled with all warnings" is only accurate because it also sets the notify values; the info
copy must not attribute the warnings to `ServiceEnabled`.

Also note the side effect: with `ServiceEnabled` written, users can no longer turn the feature off in the
Windows Security UI. That belongs in Drawbacks.

### No lockout or breakage risk

Confirmed. Nothing here can lock a user out or break an application.

### Sources

1. `C:\Windows\PolicyDefinitions\WebThreatDefense.admx` and `en-US\WebThreatDefense.adml` (26100) (tier A)
2. Enhanced Phishing Protection in Microsoft Defender SmartScreen, "Recommended settings for your
   organization" default-value table,
   <https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/enhanced-phishing-protection> (tier A)
3. Policy CSP WebThreatDefense, `ServiceEnabled` Default Value 1,
   <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-webthreatdefense> (tier A)

---

## 4. `asr_standard_protection_rules`: CONFIRMED

### GUIDs

Both verified character by character against the ASR rules reference:

| GUID | Rule name in Microsoft Learn |
| --- | --- |
| `56a863a9-875e-4185-98a7-b882c64b5ce5` | Block abuse of exploited vulnerable signed drivers (Device) |
| `e6db77e5-3df2-4cf1-b95a-636979351e5b` | Block persistence through WMI event subscription |

Both sit under the heading "Standard protection rules" in the reference. That section contains exactly
three rules: these two plus Block credential stealing from the Windows local security authority
subsystem. The proposal's claim that Microsoft designates exactly three is correct.

### Value type

The ASR Rules key is populated by an ADMX list element with `explicitValue="true"`:

```xml
<policy name="ExploitGuard_ASR_Rules" class="Machine"
        key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR"
        valueName="ExploitGuard_ASR_Rules">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0_RS3" />
  <elements>
    <list id="ExploitGuard_ASR_Rules" additive="true" explicitValue="true"
          key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules" />
  </elements>
</policy>
```

`explicitValue="true"` means the value **name** is the GUID and the value **data** is the state, written
as a string. `REG_SZ` is correct, and it matches the existing corpus convention in `security.yaml`.
States `"0"` off, `"1"` block, `"2"` audit, `"6"` warn are correct as proposed.

### Dependencies

Checked explicitly, because the parent asked whether this proposal silently requires proposal 1:

- `56a863a9`: **Dependencies: None.**
- `e6db77e5`: **Dependencies: Microsoft Defender Antivirus, RPC.**

Neither requires cloud protection. This tweak does **not** depend on `defender_cloud_protection`. Both
require Defender Antivirus to be the active antivirus, so both are inert in passive mode.

### Risk

The proposal's risk read holds. `56a863a9` prevents apps from *saving* vulnerable signed drivers and
explicitly "doesn't prevent loading existing drivers already on the computer", so it cannot brick a
running machine. `e6db77e5` has limited exclusion support and Microsoft warns that Configuration Manager
clients rely heavily on WMI, which is irrelevant on a consumer box.

Minor addition for the info copy: `56a863a9` supports user notification pop-ups but does not raise EDR
alerts; `e6db77e5` does both.

### Applicability

Learn's support matrix: `56a863a9` Windows 11 and Windows 10 1709 or later; `e6db77e5` Windows 11 and
Windows 10 1903 or later. Both match the proposal. Both apply on LTSC 2021 (19044 > 1903).

The proposal's claim "Both rules work on Windows 11 Home" was not confirmed by any tier A source and
should be dropped or softened. ASR rule enforcement on Home has no Microsoft statement either way.

### Corrections needed

None to the proposal.

### Side finding outside the 12: the existing corpus ASR LSASS rule is inert

`security.yaml` line 828 writes the GUID `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b0`. Microsoft's documented
GUID for Block credential stealing from the Windows local security authority subsystem is
`9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2`. The final character is `2`, not `0`. As shipped,
`asr_block_lsass_theft` writes a value name that maps to no rule and therefore does nothing. This is a
pre-existing corpus defect, not a defect in the 12 proposals, but it changes the framing of proposal 4:
the corpus currently ships **zero** working standard protection rules, not one.

### Sources

1. Attack surface reduction (ASR) rules reference, per-rule GUID and Dependencies fields, and the
   "Standard protection rules" section,
   <https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference> (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policy `ExploitGuard_ASR_Rules` (tier A)

---

## 5. `disable_winrm_remoting`: CONFIRMED

### All six values and their polarity

Parsed from the shipped `WindowsRemoteManagement.admx` on 26100. Every one matches the proposal,
including the trap in `AllowDigest`:

| ADMX policy | Key | Value name | enabledValue | disabledValue | Hardened |
| --- | --- | --- | --- | --- | --- |
| `AllowBasic_2` | `Software\Policies\Microsoft\Windows\WinRM\Client` | `AllowBasic` | 1 | 0 | `0` |
| `AllowUnencrypted_2` | `...\WinRM\Client` | `AllowUnencryptedTraffic` | 1 | 0 | `0` |
| `DisallowDigest` | `...\WinRM\Client` | `AllowDigest` | **0** | **1** | `0` |
| `AllowBasic_1` | `...\WinRM\Service` | `AllowBasic` | 1 | 0 | `0` |
| `AllowUnencrypted_1` | `...\WinRM\Service` | `AllowUnencryptedTraffic` | 1 | 0 | `0` |
| `DisableRunAs` | `...\WinRM\Service` | `DisableRunAs` | 1 | 0 | `1` |

All `REG_DWORD`, class Machine. `DisallowDigest` is the inverted one, where the policy being "Enabled"
writes `AllowDigest` = 0. The proposal got this right.

### Lockout analysis (the parent's specific question)

This cannot lock a user out of their own machine.

- WinRM governs **inbound** WS-Man only. Local sign-in, console, and the internal keyboard are untouched.
- Microsoft: "By default, no WinRM listener is configured. Even if the WinRM service is running,
  WS-Management protocol messages that request data can't be received or sent." So on a machine that has
  never run `winrm quickconfig`, disabling the service removes an already-nonfunctional surface.
- The proposal is right that outbound `Invoke-Command` from this machine to others is not affected, and
  that SSH remoting is a separate channel.
- RDP is a separate service and is untouched.

Real breakage to state in Cautions, going slightly beyond the proposal:

- Incoming PowerShell Remoting and `Enter-PSSession` into this machine stop working.
- **Windows Event Collector (`wecsvc`) depends on WinRM.** Anyone using event forwarding loses it.
- Windows Admin Center and several remote-management agents stop working.

### Correction: the `Manual` stock start type is unsourced

The proposal asserts the stock default is `Manual` on a Windows 11 client. No tier A source states a
start-type value. Microsoft says only: "The WinRM service starts automatically on Windows Server 2008,
and later. On earlier versions of Windows (client or server), you need to start the service manually."
That supports "not automatic on client" but does not pin `Manual` versus trigger-start.

This matters for revert. Do not hardcode `Manual` as the restore value. The corpus already snapshots
service start types; the Stock Default option must restore from the snapshot. Note also that any machine
where `winrm quickconfig` has ever run will be at **delayed auto start** with a listener configured, per
Microsoft's own quoted output, so a hardcoded `Manual` revert would silently downgrade a working
configuration.

### Corrections needed

One: replace the hardcoded `Manual` stock default with a snapshot restore, and stop asserting `Manual` in
the info copy.

### Sources

1. `C:\Windows\PolicyDefinitions\WindowsRemoteManagement.admx` (26100), policies `AllowBasic_1`,
   `AllowBasic_2`, `AllowUnencrypted_1`, `AllowUnencrypted_2`, `DisallowDigest`, `DisableRunAs` (tier A)
2. Installation and configuration for Windows Remote Management, "Configuration of WinRM and IPMI" and
   "Quick default configuration",
   <https://learn.microsoft.com/en-us/windows/win32/winrm/installation-and-configuration-for-windows-remote-management> (tier A)

---

## 6. `powershell_module_transcript_logging`: CONFIRMED

### Keys, value names and types

Confirmed in the shipped `PowerShellExecutionPolicy.admx` on 26100:

```xml
<policy name="EnableModuleLogging" class="Both"
        key="Software\Policies\Microsoft\Windows\PowerShell\ModuleLogging"
        valueName="EnableModuleLogging">
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
  <elements>
    <list id="Listbox_ModuleNames"
          key="Software\Policies\Microsoft\Windows\PowerShell\ModuleLogging\ModuleNames" />
  </elements>
</policy>

<policy name="EnableTranscripting" class="Both"
        key="Software\Policies\Microsoft\Windows\PowerShell\Transcription"
        valueName="EnableTranscripting">
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
  <elements>
    <text id="OutputDirectory" valueName="OutputDirectory" />
    <boolean id="EnableInvocationHeader" valueName="EnableInvocationHeader" />
  </elements>
</policy>
```

Type mapping is therefore: `EnableModuleLogging` `REG_DWORD`, `EnableTranscripting` `REG_DWORD`,
`OutputDirectory` `REG_SZ` (a `text` element), `EnableInvocationHeader` `REG_DWORD` (a `boolean`
element). Every type in the proposal is correct.

Note the class is `Both`, so an HKCU copy of these keys also takes effect for that user. Writing HKLM is
the right machine-wide choice; the tweak should not also need HKCU.

### The `ModuleNames` subkey: settled from PowerShell's own reader

This was the one detail with a plausible failure mode, because the ADMX `list` element carries no
`explicitValue`, which would normally mean Group Policy writes numbered value names. PowerShell's policy
reader settles it. From `src/System.Management.Automation/engine/Utils.cs`,
`TrySetPolicySettingsFromRegistryKey`:

```csharp
else if (subKeyNameSet != null && subKeyNameSet.Contains(settingName))
{
    using (RegistryKey subKey = gpoKey.OpenSubKey(settingName))
    {
        if (subKey != null)
        {
            rawRegistryValue = subKey.GetValueNames();
        }
    }
}
```

For the `ModuleNames` property, PowerShell reads **the value names of the `ModuleNames` subkey** and
ignores the data entirely. So:

- The proposal's layout, a value **named** `*` under `...\ModuleLogging\ModuleNames`, is correct and is
  the only layout that works.
- The `REG_SZ` type and the `"*"` data are cosmetic as far as PowerShell is concerned, but they match what
  CIS, DISA STIG write-ups and Sophia Script all write, so keep them.
- A numbered value name such as `1` = `*` would register a module literally named "1" and would match
  nothing. Do not let a generic ADMX list writer produce that shape.

The proposal's statement that "`EnableModuleLogging` requires the `ModuleNames` subkey with at least one
entry" is now proven rather than assumed.

### Lockout and breakage

None. Neither setting can lock a user out. The proposal's cautions are accurate: transcription writes a
file per session, and `*` module logging is verbose and rolls the PowerShell operational log faster.
Pointing `OutputDirectory` at a path under `%ProgramData%` rather than the user's Documents folder is
sound advice; the default with the value absent is a `PowerShell_transcript.*.txt` file in the user's
Documents folder.

### Corrections needed

None.

### Sources

1. `C:\Windows\PolicyDefinitions\PowerShellExecutionPolicy.admx` (26100), policies `EnableModuleLogging`
   and `EnableTranscripting` (tier A)
2. PowerShell source, `src/System.Management.Automation/engine/Utils.cs`,
   `TrySetPolicySettingsFromRegistryKey`,
   <https://raw.githubusercontent.com/PowerShell/PowerShell/master/src/System.Management.Automation/engine/Utils.cs> (tier A, first-party implementation)
3. Sophia Script for Windows 11, `ModuleNames` write using value name `*` (tier C, corroboration only)

---

## 7. `restrict_remote_sam`: CONFIRMED

### Key, value name and type

Microsoft's Security Policy Settings reference states it directly:

| Field | Value |
| --- | --- |
| Registry location | `HKEY_LOCAL_MACHINE\System\CurrentControlSet\Control\Lsa\RestrictRemoteSam` |
| Registry type | `REG_SZ` |
| Registry value | A string that will contain the SDDL of the security descriptor to be deployed |

This is one of the values the parent flagged as a REG_SZ that authors often write as REG_DWARD. The
proposal has it correct: **`REG_SZ`, not DWORD.** A DWORD write here would be silently ignored.

The hardened SDDL `O:BAG:BAD:(A;;RC;;;BA)` is the standard CIS and DISA STIG value and parses as owner
Built-in Administrators, group Built-in Administrators, DACL granting Read Control to Built-in
Administrators only. The proposal's reading is correct.

### Not a duplicate

Distinct from the corpus's `restrict_anonymous_enum`, which writes `RestrictAnonymousSAM`,
`RestrictAnonymous` and `EveryoneIncludesAnonymous` in the same `Lsa` key. Those govern **anonymous**
sessions. `RestrictRemoteSam` governs which **authenticated** principals may make SAMRPC calls. Different
value, different attack, additive.

### Default and honest framing

Microsoft notes the Group Policy setting is only available on Windows 10 1607 / Server 2016 and later, and
that on those versions the default already restricts remote SAM calls to administrators. So the registry
default is value-absent, and applying this pins an already-good state rather than changing behaviour on a
healthy machine. Under the corpus's inclusion rule that is fine, and the proposal says so honestly.

### Risk

Low, as claimed. Additional caution worth adding: because the value is a raw SDDL, a malformed or
over-restrictive string is not validated at write time and would silently deny SAMRPC to principals that
need it. Ship the exact CIS string as a fixed option value; never let the user type an arbitrary SDDL.

No lockout risk for local sign-in.

### Corrections needed

None. One evidence note: the proposal cites "Verified absent on 26100.4061" from the research machine,
which is not admissible under the machine rule. The Microsoft page carries the claim independently, so
the conclusion stands.

### Sources

1. Network access: Restrict clients allowed to make remote calls to SAM,
   <https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-restrict-clients-allowed-to-make-remote-sam-calls> (tier A)
2. `src-tauri/tweaks/security.yaml` lines 327 to 346, existing `restrict_anonymous_enum` effects
   (dedupe evidence)

---

## 8. `block_always_install_elevated`: CONFIRMED

### Key, value name, type, class

Confirmed in the shipped `MSI.admx` on 26100:

```xml
<policy name="AlwaysInstallElevated" class="Both"
        key="Software\Policies\Microsoft\Windows\Installer"
        valueName="AlwaysInstallElevated">
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

`REG_DWORD`, class `Both`, so both `HKLM\SOFTWARE\Policies\Microsoft\Windows\Installer` and
`HKCU\SOFTWARE\Policies\Microsoft\Windows\Installer` are real targets. Stock default is value-absent in
both hives.

### Correction: the reasoning about "both hives" is backwards

The proposal says "Class is Both in `MSI.admx`, so both hives must be written for the control to be
effective." That is the wrong way round. The escalation exists only when **both** hives are set to 1, so
writing `0` to HKLM alone already defeats it. Writing HKCU as well is defence in depth, not a
requirement.

There is also a scope limitation the proposal does not state: an HKCU write only pins the hive of the
user the tweak runs as. On a multi-user machine, other users' HKCU hives are untouched. Since HKLM=0 is
sufficient, this is harmless, but the info copy should not imply machine-wide HKCU coverage.

### Risk

None meaningful, as claimed. Writing 0 cannot break a working installer flow, because a working installer
flow does not depend on the escalation. Fully reversible by deleting both values.

### Corrections needed

One, cosmetic and in the reasoning rather than the mechanism: restate why both hives are written.

### Sources

1. `C:\Windows\PolicyDefinitions\MSI.admx` (26100), policy `AlwaysInstallElevated` (tier A)
2. privacy.sexy `windows.yaml`, `AlwaysInstallElevated` = 0 with `deleteOnRevert: true` (tier C,
   corroboration of the revert-by-delete shape)

---

## 9. `disable_mdns`: CORRECTED

### The control is real, and the binary evidence is stronger than the proposal's

Shipped 26100 `DnsClient.admx`:

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

ADML display name: "Configure multicast DNS (mDNS) protocol".

UTF-16 string extraction from the shipped 26100 `dnsrslvr.dll` returns both:

```
EnableMDNS
SOFTWARE\Policies\Microsoft\Windows NT\DNSClient
Software\Policies\Microsoft\Windows NT\DnsClient
SYSTEM\CurrentControlSet\Services\Dnscache\Parameters
```

and `dnsapi.dll` also carries `EnableMDNS`. The resolver reads the value name from the policy path. Key,
value name and `REG_DWORD` type in the proposal are all correct.

### Correction A: the `>=26100` applicability gate is unjustified

The proposal specifies `windows: { build: ">=26100" }` and says "do not claim it for LTSC 2021." The
shipped ADMX declares `supportedOn ref="windows:SUPPORTED_Windows_10_0_RS2"`, that is "At least Windows
10". There is no tier A evidence for a 26100 floor. LTSC 2021 is build 19044, well above RS2.

Drop the hard build gate. If caution is wanted, keep the tweak ungated and say in the info copy that it
was verified on 26100 and is expected but not verified on 19044.

### Correction B: the shipped ADML does not promise an off state for the Disabled setting

`DNS_MDNS_Help` reads: "Specifies if the DNS client will perform name resolution over mDNS. If you enable
this policy, the DNS client will use mDNS protocol. If you disable this policy setting, **or if you do
not configure this policy setting, the DNS client will use locally configured settings.**"

Read literally, the ADMX `disabledValue` of 0 puts the client back on local settings rather than
unconditionally off. This is most likely sloppy ADML boilerplate rather than real behaviour, but it is the
only first-party statement of what 0 does, and it is not the statement the proposal needs.

Consequence: do not treat
`HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\EnableMDNS` = 0 as "an optional second
effect", as the proposal suggests. Write **both** values. `dnsrslvr.dll` carries the `Dnscache\Parameters`
base path alongside the policy path, so both are read, and the service-key value is the "locally
configured setting" the ADML defers to.

Corrected effect set:

| Key | Value name | Type | Disable mDNS | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `EnableMDNS` | REG_DWORD | `0` | value-absent |
| `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters` | `EnableMDNS` | REG_DWORD | `0` | value-absent |

### What survives

- Not a duplicate of `disable_llmnr` (`EnableMulticast`, same policy key, different value) or
  `disable_netbios_tcpip` (per-adapter `NetbiosOptions`). Genuinely the third leg.
- The risk assessment is accurate and is the highest of the three name-resolution tweaks: `.local` names,
  AirPrint and IPP Everywhere printers, Chromecast and Google Cast discovery, Apple device discovery.
  Users with a network printer found by name will notice.
- Reboot or a `Dnscache` restart is required. `requires_reboot: true`.
- Fully reversible by deleting both values.

### Sources

1. `C:\Windows\PolicyDefinitions\DnsClient.admx` and `en-US\DnsClient.adml` (26100), policy `DNS_MDNS`
   (tier A)
2. UTF-16 string extraction from shipped 26100 `dnsrslvr.dll` and `dnsapi.dll` (tier A, shipped binary)

---

## 10. `update_feature_control`: CORRECTED

### `AllowTemporaryEnterpriseFeatureControl`: confirmed

Shipped 26100 `WindowsUpdate.admx`: key `Software\Policies\Microsoft\Windows\WindowsUpdate`, value name
`AllowTemporaryEnterpriseFeatureControl`, `REG_DWORD`, enabledValue 1, disabledValue 0,
`supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"`.

ADML `AllowTemporaryEnterpriseFeatureControl_Help`: "Features introduced via servicing (outside of the
annual feature update) are off by default for devices that have their Windows updates managed. If this
policy is configured to 'Enabled', then all features available in the latest monthly quality update
installed will be on. If this policy is set to 'Not Configured' or 'Disabled' then features that are
shipped via a monthly quality update (servicing) will remain off until the feature update that includes
these features is installed. *Windows update managed devices are those that have their Windows updates
managed via policy; whether via the cloud using Windows Update for Business or on-premises with Windows
Server Update Services (WSUS)."

The proposal's semantics for this value are correct, including the "only on update-managed devices"
nuance.

### `SetAllowOptionalContent`: materially wrong

The proposal's table says:

> `SetAllowOptionalContent` | `0` (no optional content) | `1` automatically receive optional updates,
> `2` also get the latest optional non-security preview

The shipped ADMX shows the GP policy named `AllowOptionalContent` writes **two different values**:

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

So `SetAllowOptionalContent` is only the enable flag (1 or 0). The selection lives in a **separate**
`AllowOptionalContent` value. Policy CSP confirms the allowed values and, critically, that 1 is the
**more** permissive option, not the less:

| `AllowOptionalContent` | Meaning |
| --- | --- |
| 0 (Default) | Don't receive optional updates |
| 1 | Automatically receive optional updates, **including gradual feature rollouts (CFRs)** |
| 2 | Automatically receive optional updates (optional cumulative updates only) |
| 3 | Users can select which optional updates to receive |

The proposal has the value name wrong and the ordering of 1 and 2 effectively inverted.

### Corrected effect table

| Key | Value name | Type | Stability | Permissive | Stock default |
| --- | --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `AllowTemporaryEnterpriseFeatureControl` | REG_DWORD | `0` | `1` | value-absent |
| same | `SetAllowOptionalContent` | REG_DWORD | `0` | `1` | value-absent |
| same | `AllowOptionalContent` | REG_DWORD | absent | `1`, `2` or `3` | value-absent |

For the stability option, write `SetAllowOptionalContent` = 0 and leave `AllowOptionalContent` absent.
For revert, delete all three.

### Correction: two different applicability gates, not one

The proposal gives a single gate. The shipped ADMX gives two:

- `AllowTemporaryEnterpriseFeatureControl`: `SUPPORTED_Windows_11_0_22H2`, that is Windows 11 22H2 and
  later. Not on LTSC 2021.
- `AllowOptionalContent` / `SetAllowOptionalContent`: `WU_SUPPORTED_WinServer2025_Win1021H2_Win1122H2`,
  which the ADML renders as "At least Windows Server 2025, Windows 10 Version 21H2, or Windows 11
  Version 22H2". Policy CSP gives Windows 10 21H2 (10.0.19044.3757) and later, so LTSC 2021 qualifies if
  patched.

Editions for both, per Policy CSP: Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC. Not
Home. The proposal's "Not Home" is correct.

### Honest note the proposal already half-makes

Per the ADML, temporary enterprise feature control gating applies **only** to update-managed devices. On
an unmanaged consumer PC with no other Windows Update policy, writing `AllowTemporaryEnterpriseFeatureControl`
= 0 may change nothing. The proposal correctly flags that setting `DeferQualityUpdates` or
`TargetReleaseVersion` makes the device managed. Whether writing this value alone is itself enough to
make the device "policy managed" is not stated by any tier A source; see Unknowns.

### Sources

1. `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` and `en-US\WindowsUpdate.adml` (26100), policies
   `AllowTemporaryEnterpriseFeatureControl` and `AllowOptionalContent` (tier A)
2. Policy CSP Update, `AllowOptionalContent` allowed values and editions,
   <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update> (tier A)

---

## 11. `defer_feature_updates`: CONFIRMED

### The control is live on 26100

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

Both value names, both `REG_DWORD`, and the 0 to 365 range in the proposal match exactly. Policy CSP
`DeferFeatureUpdatesPeriodInDays` gives Windows 10 1607 (10.0.14393) and later, editions Pro, Enterprise,
Education, IoT Enterprise and IoT Enterprise LTSC, range 0 to 365, default 0. The proposal's edition gate
("Not Home") is correct.

I looked specifically for a deprecation, because deferral policies are the kind of thing Microsoft has
been retiring. None found: the policy is present and undecorated in the 26100 ADMX, the Policy CSP page
carries no deprecation banner, and the WUfB overview still lists feature updates with a 365-day maximum
deferral. Nothing supports rejecting this.

### Not a duplicate

`defer_quality_updates` writes `DeferQualityUpdates` and `DeferQualityUpdatesPeriodInDays` (max 30).
`target_release_version` writes `TargetReleaseVersion`, `TargetReleaseVersionInfo` and `ProductVersion`.
Different value names, different behaviour. Additive.

### Strengthen the conflict warning

The proposal says the two "should reference each other in their info copy so a user does not set
contradictory states". Microsoft is blunter than that: "When you specify target version policy, feature
update deferrals won't be in effect."

That is not a soft interaction. If the user has `target_release_version` applied, this tweak is **fully
inert**. Treat them as mutually exclusive in the UI, the same way the proposal asks for
`defender_cloud_protection` versus a future "disable MAPS" tweak, rather than as two independent tweaks
with a cross-reference.

### Two mechanical notes

- The same ADMX policy also owns `PauseFeatureUpdatesStartTime`, a `REG_SZ`. The tweak must not write it,
  and the revert must not assume it is absent if the user paused updates from the Settings UI.
- `BranchReadinessLevel` is **not** part of this policy in the 26100 ADMX. It belongs to
  `ManagePreviewBuilds`. The proposal correctly omits it; a naive port of older guidance would have added
  it.

### Corrections needed

None to key, value name, type or range. One to the interaction framing, described above.

### Sources

1. `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` and `en-US\WindowsUpdate.adml` (26100), policy
   `DeferFeatureUpdates` (tier A)
2. Policy CSP Update, `DeferFeatureUpdatesPeriodInDays`,
   <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update> (tier A)
3. Walkthrough: Use Group Policy to configure Windows Update client policies, "I want to stay on a
   specific version": "When you specify target version policy, feature update deferrals won't be in
   effect", <https://learn.microsoft.com/en-us/windows/deployment/update/waas-wufb-group-policy> (tier A)

---

## 12. `kernel_dma_protection`: CORRECTED

### `DeviceEnumerationPolicy`: key, value, type and enum all confirmed

Shipped 26100 `DmaGuard.admx`, with an unusually helpful comment left in by the authors:

```xml
<policy name="DmaGuardEnumerationPolicy" class="Machine"
        key="Software\Policies\Microsoft\Windows\Kernel DMA Protection">
  <supportedOn ref="windows:SUPPORTED_Windows_10_0" />
  <elements>
    <enum id="DmaGuardEnumerationPolicy_Options" valueName="DeviceEnumerationPolicy">
      <!-- Note that the displayName values do not match the registry
           values. This has no functional impact, but changing them will
           affect localization, so leave as is. -->
      <!-- Block all -->
      <item displayName="$(string.DmaGuardEnumerationPolicy_Options_3)"><value><decimal value="0"/></value></item>
      <!-- Only while logged in -->
      <item displayName="$(string.DmaGuardEnumerationPolicy_Options_2)"><value><decimal value="1"/></value></item>
      <!-- Allow all -->
      <item displayName="$(string.DmaGuardEnumerationPolicy_Options_1)"><value><decimal value="2"/></value></item>
    </enum>
  </elements>
</policy>
```

`REG_DWORD`. The enum mapping in the proposal (0 Block all, 1 Allow only after sign-in, 2 Allow all) is
correct.

### Correction A: the stock default is 1, not 2

The proposal's table says: "value-absent, effective default `2` (allow after sign-in)". That is wrong on
both halves, and it contradicts the proposal's own enum table two lines later, where 2 is "Allow all".

The shipped ADML resolves the option strings:

```
DmaGuardEnumerationPolicy_Options_1 = "Allow all"
DmaGuardEnumerationPolicy_Options_2 = "Only while logged in (default)"
DmaGuardEnumerationPolicy_Options_3 = "Block all"
```

Microsoft labels **"Only while logged in"** as the default, and that string maps to registry value **1**.
Microsoft Learn agrees behaviourally: "By default, peripherals with DMA Remapping incompatible drivers
are blocked from starting and performing DMA until an authorized user signs into the system or unlocks
the screen."

Corrected: registry default is value-absent; effective default is **1**, "Only while logged in".

This is a revert-correctness defect, exactly the class `_harmful-revert.md` is about. It also collapses
one of the proposed options: the "Allow only after sign-in" middle option (value 1) is identical to the
Windows default, so only the "Block all incompatible external devices" option changes behaviour. Keep the
middle option if the corpus wants an explicit pin, but say in Drawbacks that it matches the default.

### `DisableExternalDMAUnderLock`: confirmed, with a narrower scope than claimed

Shipped 26100 `VolumeEncryption.admx`: key `Software\Policies\Microsoft\FVE`, value name
`DisableExternalDMAUnderLock`, `REG_DWORD`, enabledValue 1, disabledValue 0,
`supportedOn ref="windows:SUPPORTED_Windows_10_0_RS2"`.

ADML `DisableExternalDMAUnderLock_Help`: "This policy setting allows you to block direct memory access
(DMA) for all **Thunderbolt hot pluggable PCI downstream ports** until a user logs into Windows... Every
time the user locks the machine, DMA will be blocked on hot plug Thunderbolt PCI ports with no children
devices, until the user logs in again. Devices which were already enumerated when the machine was
unlocked will continue to function until unplugged or the system is rebooted or hibernated. **This policy
setting is only enforced when BitLocker or device encryption is enabled.**"

The proposal says it "requires BitLocker to be active on the OS volume". Close enough, but the exact
condition is BitLocker **or device encryption**, which matters on consumer 24H2 machines where automatic
device encryption is common. Also narrow the claim: it is Thunderbolt hot-plug ports specifically, not all
external DMA.

### Correction B: applicability and reboot

Policy CSP DmaGuard: Windows 10 1809 (10.0.17763) and later; editions Pro, Enterprise, Education, IoT
Enterprise, IoT Enterprise LTSC. **Not Home.** The ADMX `supportedOn` of `SUPPORTED_Windows_10_0` is
looser than the CSP page; use 1809 as the floor.

Also from the CSP page, and missing from the proposal: "This policy requires a system reboot to take
effect." `requires_reboot: true` is mandatory, not optional.

And: "This policy only takes effect when Kernel DMA Protection is enabled and supported by the system...
Kernel DMA Protection is a platform feature that can't be controlled via policy or by end user. It has to
be supported by the system at the time of manufacturing." Plus "this policy doesn't apply to 1394, PCMCIA
or ExpressCard devices."

So the tweak is inert on Home, inert on any machine without firmware DMA remapping, and inert for the
three legacy bus types. The proposal's instinct to probe the Kernel DMA Protection state and report "not
applicable" is right and should be a requirement, not a suggestion.

### Lockout analysis (the parent's specific question)

This cannot lock a user out of their own machine.

- The machine still boots and signs in. The internal keyboard, internal display and internal storage are
  not external DMA-capable peripherals and are unaffected.
- USB HID devices on native USB ports are not affected; the policy governs external PCIe and Thunderbolt
  peripherals whose drivers are not DMA-remapping compatible.
- The real breakage, correctly identified by the proposal, is external GPU enclosures, some Thunderbolt
  docks and older PCIe capture cards refusing to start under `DeviceEnumerationPolicy` = 0.
- One case the proposal does not name: a laptop used lid-closed with a Thunderbolt dock supplying the
  only keyboard, mouse and display. If the dock's downstream controller is DMA-remapping incompatible and
  is blocked, that user loses input until they open the lid. Recoverable, not a lockout, but it belongs in
  Cautions.
- `DisableExternalDMAUnderLock` = 1 only blocks **new** Thunderbolt hot-plugs while locked. Already
  enumerated devices keep working. No lockout.

### Sources

1. `C:\Windows\PolicyDefinitions\DmaGuard.admx` and `en-US\DmaGuard.adml` (26100), policy
   `DmaGuardEnumerationPolicy` including the in-file comment mapping options to registry values (tier A)
2. `C:\Windows\PolicyDefinitions\VolumeEncryption.admx` and `en-US\VolumeEncryption.adml` (26100),
   policy `DisableExternalDMAUnderLock_Name` (tier A)
3. Policy CSP DmaGuard, applicability, editions and reboot requirement,
   <https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-dmaguard> (tier A)
4. Kernel DMA Protection, "How Windows protects against DMA drive-by attacks",
   <https://learn.microsoft.com/en-us/windows/security/hardware-security/kernel-dma-protection-for-thunderbolt> (tier A)

---

## Cross-cutting findings

### Inertness conditions the proposals do not consistently state

| Proposal | Silently inert when |
| --- | --- |
| 1 `defender_cloud_protection` | Tamper Protection on (cloud settings), Defender passive or removed |
| 3 `enhanced_phishing_protection` | No work or school account signed in, which is most consumer machines; also three of five values already default hardened |
| 4 `asr_standard_protection_rules` | Defender not the active antivirus |
| 5 `disable_winrm_remoting` | No WinRM listener configured, which is the stock state |
| 10 `update_feature_control` | Device is not update-managed (for the temporary feature control half); Home edition |
| 11 `defer_feature_updates` | `target_release_version` is also applied; Home edition |
| 12 `kernel_dma_protection` | No firmware DMA remapping; Home edition; and `DisableExternalDMAUnderLock` without BitLocker or device encryption |

None of these is a reason to exclude a tweak under the corpus's inclusion rule. All of them are reasons
the Drawbacks text and the probe design must be honest.

### Home edition

Five of the twelve are Pro and above only per Policy CSP: proposals 3, 10, 11, 12, and the DmaGuard half
of 12. Proposals 1, 2, 4, 5, 6, 7, 8, 9 carry no edition gate in any tier A source.

### Pre-existing corpus defect surfaced during verification

`security.yaml` `asr_block_lsass_theft` writes GUID `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b0`. Microsoft's
documented GUID ends `...e4b2`. The tweak is inert as shipped. Outside the scope of these 12, but it
should be filed.

## Unknowns

1. **Does writing `AllowTemporaryEnterpriseFeatureControl` = 0 by itself make a device "update
   managed"?** The ADML gates the behaviour on the device being update-managed via policy, but no tier A
   source says whether the presence of this single policy value satisfies that condition, or whether a
   separate WUfB policy such as `DeferQualityUpdates` is required first. Affects whether proposal 10 does
   anything on an otherwise unmanaged consumer 26100.
2. **Whether `EnableMDNS` = 0 in the policy key alone is sufficient.** The value name and the policy path
   are both proven present in `dnsrslvr.dll`, but the only first-party description of the Disabled state
   says "the DNS client will use locally configured settings" rather than "mDNS is off". Writing both the
   policy value and the `Dnscache\Parameters` value removes the ambiguity, which is why that is the
   correction, but the policy-only case remains unproven.
3. **The exact stock start type of the `WinRM` service on a clean Windows 11 24H2 client.** Microsoft
   states only that it is not automatic on client versions. Since the research machine is inadmissible and
   no install image was available, `Manual` versus trigger-start is unresolved. Mitigated by restoring
   from the snapshot rather than hardcoding.
4. **ASR rule behaviour on Windows 11 Home.** The proposal asserts both new rules work on Home. No tier A
   source confirms or denies ASR enforcement on Home. Treated as unsupported rather than false.
5. **Whether Tamper Protection blocks `MpCloudBlockLevel` specifically.** Microsoft's list of
   tamper-protected settings includes "Cloud protection remains enabled" but does not name the cloud block
   level. The distinction decides how much of proposal 1 actually lands on a stock consumer machine.
6. **CIS Windows 11 v4.0.0 and DISA STIG V2R2 rule identifiers** were not independently re-read in this
   pass. The proposal's tier T2 citations were accepted where a tier A source independently confirmed the
   same key, value and type, which was the case for every control cited that way. They were not used as
   sole evidence anywhere in this document.
