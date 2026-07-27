# Corpus gap hunt: security, performance and gaming, network, Windows Update, power, services and tasks

Research date: 2026-07-26. Target platform: Windows 11 24H2 (26100) and 25H2 (26200) primary,
Windows 10 IoT Enterprise LTSC 2021 (19044) secondary.

Scope of this pass: what the corpus is **missing**, not whether what it ships is correct. Domains
covered: security hardening, performance and gaming, network, Windows Update, power, services and
scheduled tasks.

## Method and evidence base

Inventory built from `src-tauri/tweaks/security.yaml` (43 tweaks), `performance.yaml` (25),
`network.yaml` (21), `services.yaml` (31), cross-checked against `privacy.yaml`, `debloat.yaml` and
`interface.yaml` so cross-category duplicates were not proposed. 240 distinct registry key plus
value pairs, 33 service ids and 9 scheduled-task paths were enumerated and used as the dedupe set.

Primary sources, in descending trust order:

1. **The ADMX set shipped on the target OS itself.** The research machine runs 26100.4061
   (IoT Enterprise LTSC 2024), so `C:\Windows\PolicyDefinitions` is the 24H2 ADMX set. Every
   policy-backed registry key, value name and class in this document was read out of that shipped
   ADMX rather than copied from a blog. Where a value is marked "not in shipped ADMX" it is a
   Security Options or MSS setting delivered through `secedit` templates, not Group Policy
   administrative templates, and the Microsoft Learn Security Policy Settings reference was used
   instead.
2. **Live state on a 26100 machine.** Stock-default determination for `HKLM\SYSTEM` values, SMB
   client and server configuration, present scheduled tasks and present services was read directly
   off the 26100 box. See "Caveat on default determination" below for where this is and is not
   reliable.
3. Microsoft Learn: Policy CSP pages, Defender for Endpoint ASR reference, SMB security and SMB NTLM
   blocking, BitLocker overview, Kernel DMA Protection, Windows Update client policies.
4. DISA STIG for Microsoft Windows 11 V2R2 (260 rules, full title list enumerated and diffed against
   the corpus) and CIS Microsoft Windows 11 Enterprise Benchmark v4.0.0.
5. Community corpora used only as corroboration and never as a sole source: Chris Titus WinUtil
   `config/tweaks.json`, privacy.sexy `windows.yaml`, Sophia Script for Windows 11.

### Caveat on default determination

The research machine is **not pristine**. Values under
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender`,
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` and
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` show evidence of prior tweaking
(`SpynetReporting`=0, `MpCloudBlockLevel`=0, `DeferFeatureUpdates`=1,
`LocalAccountTokenFilterPolicy`=1, `EnableSecureUIAPaths`=0), and several services in the corpus's
own list are already disabled. Observed state was therefore used as authoritative **only** for
non-policy `HKLM\SYSTEM` values, SMB configuration, and task and service presence. For policy keys
the documented default was used and is stated as such.

---

## A. Ranked table of proposed additions

Value column: High means a real, defensible, user-visible benefit that the corpus currently cannot
deliver at all. Medium means a real benefit that is narrower, or overlaps something already present.
Low means auditable and free but of marginal practical effect on a consumer machine.

Source tier: **T1** Microsoft first-party (shipped ADMX, Learn, Policy CSP). **T2** CIS benchmark or
DISA STIG control. **T3** three or more independent community corpora in agreement.

| # | Proposed id | Category | One line | Mechanism | Value | Tier |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `defender_cloud_protection` | security | Turn on MAPS, block at first sight and a High cloud block level | 5 values under `Windows Defender\Spynet` and `\MpEngine` | High | T1, T2 |
| 2 | `smb_client_block_ntlm` | security | Stop the SMB client offering NTLM to remote servers | `Set-SmbClientConfiguration -BlockNTLM` (24H2 feature) | High | T1 |
| 3 | `enhanced_phishing_protection` | security | Turn on Windows 11 Enhanced Phishing Protection and its warnings | 5 values under `Policies\Microsoft\Windows\WTDS\Components` | High | T1, T2 |
| 4 | `asr_standard_protection_rules` | security | Add the two remaining Microsoft "standard protection" ASR rules | 2 GUID values in the ASR `Rules` key | High | T1 |
| 5 | `disable_winrm_remoting` | security | Close the WinRM / PowerShell Remoting remote-access surface | `WinRM` service plus 6 `WinRM\Client` and `\Service` values | High | T1, T2 |
| 6 | `powershell_module_transcript_logging` | security | Add module logging and transcription to the existing script-block logging | `PowerShell\ModuleLogging` and `\Transcription` | High | T1, T2 |
| 7 | `restrict_remote_sam` | security | Restrict remote SAM enumeration to Administrators | `Lsa\RestrictRemoteSam` REG_SZ SDDL | High | T1, T2 |
| 8 | `block_always_install_elevated` | security | Pin the MSI "always install elevated" escalation path to off | `Policies\Microsoft\Windows\Installer\AlwaysInstallElevated` | High | T2 |
| 9 | `disable_mdns` | network | Kill mDNS, the third leg of the LLMNR/NBT-NS spoofing triad | `DNSClient\EnableMDNS` | High | T1, T2 |
| 10 | `update_feature_control` | network | Stop mid-cycle features and optional content arriving via monthly updates | 2 `WindowsUpdate` policy values | High | T1 |
| 11 | `defer_feature_updates` | network | Defer feature updates by N days (the counterpart to the existing quality deferral) | `DeferFeatureUpdates` plus period | High | T1 |
| 12 | `kernel_dma_protection` | security | Block DMA-incapable external peripherals and new DMA devices while locked | `Kernel DMA Protection\DeviceEnumerationPolicy`, `FVE\DisableExternalDMAUnderLock` | High | T1, T2 |
| 13 | `asr_extended_rules` | security | Add 6 further ASR rules covering scripts, macros, USB, ransomware and lateral movement | 6 GUID values in the ASR `Rules` key | Medium | T1 |
| 14 | `ntlm_outgoing_restriction` | security | Audit then restrict outgoing NTLM, and require 128-bit NTLM session security | 4 values under `Lsa\MSV1_0` | Medium | T1, T2 |
| 15 | `rdp_session_hardening` | security | High encryption, secure RPC, always prompt for password, no drive redirection | 5 `Terminal Services` policy values | Medium | T2 |
| 16 | `task_compatibility_appraiser` | services | Disable the Application Experience appraiser tasks | 5 scheduled tasks, all present and Ready on 26100 | Medium | T1, T3 |
| 17 | `task_ceip` | services | Disable the CEIP tasks including the kernel SQM task | 3 scheduled tasks | Medium | T3 |
| 18 | `disable_secondary_logon` | services | Disable the Secondary Logon (`runas`) service | `seclogon` service | Medium | T2 |
| 19 | `early_launch_antimalware_policy` | security | Set boot-start driver policy to block bad-and-unknown critical drivers | `Policies\EarlyLaunch\DriverLoadPolicy` | Medium | T1, T2 |
| 20 | `credssp_encryption_oracle` | security | Force CredSSP encryption oracle remediation to Force Updated Clients | `Policies\System\CredSSP\Parameters\AllowEncryptionOracle` | Medium | T1, T2 |
| 21 | `device_encryption_posture` | security | Choose whether 24H2 auto-enables BitLocker device encryption | `Control\BitLocker\PreventDeviceEncryption` | Medium | T1 |
| 22 | `hide_admin_accounts_on_elevation` | security | Stop enumerating admin accounts at the UAC prompt, hide password reveal | `Policies\CredUI` plus `Policies\Microsoft\Windows\CredUI` | Medium | T2 |
| 23 | `event_log_retention` | security | Size the Application, Security and System logs for real forensics | `EventLog\<channel>\MaxSize` | Medium | T2 |
| 24 | `audit_process_creation_cmdline` | security | Log process creation with full command lines | `Policies\System\Audit\ProcessCreationIncludeCmdLine_Enabled` plus auditpol | Medium | T2 |
| 25 | `disable_wer_service` | services | Disable the Windows Error Reporting service, not just its policy | `WerSvc` service | Medium | T3 |
| 26 | `spooler_remote_rpc_off` | security | Keep local printing but close the spooler's remote RPC endpoint | `Control\Print\RegisterSpoolerRemoteRpcEndPoint`, `RpcAuthnLevelPrivacyEnabled` | Medium | T1 |
| 27 | `block_insider_builds_policy` | network | Block Insider preview builds by policy, not only by service | `ManagePreviewBuilds` plus value | Medium | T1 |
| 28 | `autoplay_non_volume` | security | Complete the AutoPlay lockdown with the non-volume device case | `Policies\Microsoft\Windows\Explorer\NoAutoplayfornonVolume` | Medium | T2 |
| 29 | `remote_uac_token_filter` | security | Force remote local-admin logons to get a filtered token | `Policies\System\LocalAccountTokenFilterPolicy` | Medium | T2 |
| 30 | `disable_internet_connection_sharing` | services | Disable ICS and hide its UI | `SharedAccess` service plus `NC_ShowSharedAccessUI` | Medium | T2 |
| 31 | `reserved_storage_off` | performance | Reclaim roughly 7 GB of reserved storage | `Set-WindowsReservedStorageState` | Medium | T1 |
| 32 | `firewall_logging_and_merge` | security | Log dropped packets and stop apps silently adding firewall rules | `WindowsFirewall\<profile>\Logging` plus `AllowLocalPolicyMerge` | Medium | T2 |
| 33 | `enable_sehop` | security | Explicitly enable Structured Exception Handling Overwrite Protection | `Session Manager\kernel\DisableExceptionChainValidation` | Low | T2 |
| 34 | `no_index_encrypted_files` | security | Stop Windows Search indexing EFS-encrypted files | `Windows Search\AllowIndexingEncryptedStoresOrItems` | Low | T2 |
| 35 | `disable_pku2u_online_id` | security | Block PKU2U authentication using online identities | `Lsa\pku2u\AllowOnlineID` | Low | T2 |
| 36 | `mss_network_stack_hardening` | network | The MSS legacy IP stack settings (source routing, ICMP redirect, NBT release) | 5 values under `Services\Tcpip*` and `Netbt` | Low | T2 |
| 37 | `ntfs_disable_8dot3` | performance | Stop 8.3 short-name generation on all volumes | `Control\FileSystem\NtfsDisable8dot3NameCreation` | Low | T1 |
| 38 | `require_doh` | network | Add a "Require DoH" option alongside the existing auto-upgrade | `DNSClient\DoHPolicy` = 3 | Low | T1 |

Counts: **12 High, 20 Medium, 6 Low.**

---

## B. Full entries

### 1. `defender_cloud_protection` (High)

**Why it is a gap.** The corpus enables Controlled Folder Access, Network Protection, PUA protection
and five ASR rules, but never turns on the cloud protection those features depend on. Microsoft
states plainly that "Use advanced protection against ransomware" requires cloud-delivered protection
to be enabled, and that CloudBlockLevel "requires the Join Microsoft MAPS setting enabled in order to
function". Several ASR rules only produce user notifications at cloud block level High or above. As
shipped, the corpus can leave a machine with ASR rules configured but degraded.

**Keys and values.** All `REG_DWORD`, class Machine, confirmed in `WindowsDefender.admx` on 26100.

| Key | Value name | Hardened | Stock default |
| --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet` | `SpynetReporting` | `2` (Advanced MAPS) | value-absent (policy unset; Windows Security UI default is Advanced) |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet` | `SubmitSamplesConsent` | `1` (send safe samples) | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet` | `DisableBlockAtFirstSeen` | `0` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine` | `MpCloudBlockLevel` | `2` (High) | value-absent (engine default is `0`) |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine` | `MpBafsExtendedTimeout` | `50` (seconds) | value-absent (engine default 10 s) |

`MpCloudBlockLevel` accepts `0` Default, `1` Moderate, `2` High, `4` High plus, `6` Zero tolerance.
`SubmitSamplesConsent` accepts `0` always prompt, `1` send safe samples, `2` never send,
`3` send all samples.

**Proposed options.** Three: "Cloud protection on, High block level" (the table above),
"Cloud protection on, no sample submission" (`SubmitSamplesConsent`=2, rest as above, for the
privacy-sensitive user), and "Windows default (Stock Default)" with every value absent.

**Applicability.** Windows 11 and Windows 10. Requires Microsoft Defender Antivirus to be the active
antivirus. Inert where a third-party AV has put Defender into passive mode, and inert on images with
Defender removed. Gate on Defender presence, or mark `skip_validation` on the probe.

**Risks.** `MpCloudBlockLevel` = 2 is documented as carrying a "greater chance of false positives".
Sample submission sends files to Microsoft, so the second option exists for users who object.
Setting `SpynetReporting` conflicts head-on with a privacy posture; this tweak and any future
"disable MAPS" tweak must be presented as mutually exclusive so the app never claims both states.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100);
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-defender>;
<https://learn.microsoft.com/en-us/defender-endpoint/specify-cloud-protection-level-microsoft-defender-antivirus>;
<https://learn.microsoft.com/en-us/defender-endpoint/enable-cloud-protection-microsoft-defender-antivirus>;
<https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference>.

### 2. `smb_client_block_ntlm` (High)

**Why it is a gap.** The task asked for NTLM hardening beyond what exists. The corpus has
`LmCompatibilityLevel` = 5, which controls *which* NTLM variant is used, and nothing that stops NTLM
being offered at all. SMB client NTLM blocking is new in 24H2 and is the single strongest
consumer-reachable NTLM control: it prevents a user being tricked into sending NTLM challenge
responses to a hostile SMB server, which is the entire coerced-authentication and NTLM-relay class.

**Mechanism.** `Set-SmbClientConfiguration -BlockNTLM $true -Force`. Verified on 26100:
`Get-SmbClientConfiguration | FL BlockNTLM` returns `False` on a machine where the value has never
been set, so **the stock default is off**. The Group Policy path is
Computer Configuration > Administrative Templates > Network > Lanman Workstation >
"Block NTLM (LM, NTLM, NTLMv2)". Note that `LanmanWorkstation.admx` as shipped on 26100 does **not**
contain this policy; it arrived with the Windows Server 2025 ADMX. Use the cmdlet, not a guessed
registry value. Undo is `Set-SmbClientConfiguration -BlockNTLM $false -Force`; probe is the cmdlet
read.

**Applicability.** `windows: { build: ">=26100" }`. Not available on LTSC 2021.

**Risks.** Medium. Breaks SMB access to any server that cannot do Kerberos: NAS boxes on IP address
rather than name, workgroup file shares, older network printers with SMB scan-to-folder. Microsoft
provides a per-mapping exception list (`New-SmbMapping -BlockNTLM $false`) which this tweak should
mention in its info copy but need not implement. Fully reversible; no lockout risk since it only
affects outbound SMB.

**Related.** While verifying this, a defect surfaced in the existing `require_smb_signing` tweak. See
"Defects found while gap hunting" at the end.

**Sources.** <https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-ntlm-blocking>;
<https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-24h2>;
live `Get-SmbClientConfiguration` on 26100.4061.

### 3. `enhanced_phishing_protection` (High)

**Why it is a gap.** Enhanced Phishing Protection is the Windows 11 SmartScreen component that warns
when a Windows password is typed into a phishing site, reused on a website, or stored in an unsafe
app such as Notepad. The corpus's `enforce_smartscreen` covers app and Edge SmartScreen only and
never touches this. CIS added these as Level 1 recommendations for Windows 11.

**Keys and values.** All `REG_DWORD`, class Machine, key
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WTDS\Components`, confirmed in `WebThreatDefense.admx` on
26100.

| Value name | Hardened | Stock default |
| --- | --- | --- |
| `ServiceEnabled` | `1` | value-absent (feature ships in audit mode) |
| `NotifyMalicious` | `1` | value-absent |
| `NotifyPasswordReuse` | `1` | value-absent |
| `NotifyUnsafeApp` | `1` | value-absent |
| `CaptureThreatWindow` | `1` | value-absent |

Verified absent on the 26100 research machine.

**Proposed options.** "Enabled with all warnings" (all five = 1), "Enabled, no password-reuse
warning" (`NotifyPasswordReuse` = 0, rest 1, for users who dislike the reuse nag), and
"Windows default (Stock Default)" with all five absent.

**Applicability.** `windows: { products: [11] }`. The component does not exist on Windows 10.

**Risks.** Low. `CaptureThreatWindow` sends a screenshot of the offending window to Microsoft
Defender for analysis, so it is a privacy consideration and should be a separate effect the user can
see. No lockout or breakage risk.

**Sources.** `C:\Windows\PolicyDefinitions\WebThreatDefense.admx` (26100);
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-webthreatdefense>;
<https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/enhanced-phishing-protection>;
CIS Microsoft Windows 11 Enterprise Benchmark v4.0.0.

### 4. `asr_standard_protection_rules` (High)

**Why it is a gap.** Microsoft designates exactly three ASR rules as "standard protection rules",
recommended for deployment straight to Block with no audit period. The corpus ships one of them
(LSASS credential theft) and neither of the other two.

**Key.** `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`,
type `REG_SZ` for each value, matching the existing corpus convention.

| Value name (GUID) | Rule | Hardened | Stock default |
| --- | --- | --- | --- |
| `56a863a9-875e-4185-98a7-b882c64b5ce5` | Block abuse of exploited vulnerable signed drivers | `"1"` | value-absent |
| `e6db77e5-3df2-4cf1-b95a-636979351e5b` | Block persistence through WMI event subscription | `"1"` | value-absent |

Value semantics: `"0"` off, `"1"` block, `"2"` audit, `"6"` warn.

**Proposed options.** "Block", "Audit only" (`"2"` for both, useful as a safe first step), and
"Windows default (Stock Default)" with both absent.

**Applicability.** Windows 11 and Windows 10 1709+/1903+ respectively. Requires Defender Antivirus
active; the WMI rule additionally depends on RPC. Both rules work on Windows 11 Home.

**Risks.** Low. `56a863a9` only blocks *writing* vulnerable drivers to disk; it does not unload
drivers already present, and it therefore does not brick a running machine. It complements, and does
not duplicate, the corpus's existing `block_vulnerable_drivers` (which is the HVCI blocklist, a
different mechanism). `e6db77e5` has limited exclusion support and Microsoft warns Configuration
Manager clients rely heavily on WMI; irrelevant on a consumer machine but worth a line of info copy.

**Sources.** <https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference>
(GUIDs read from the per-rule detail sections).

### 5. `disable_winrm_remoting` (High)

**Why it is a gap.** The corpus closes Remote Registry, Remote Desktop and Remote Assistance but
leaves WinRM (PowerShell Remoting) entirely untouched. The STIG has six separate rules for WinRM,
and it is the standard remote-execution channel for post-exploitation tooling. This is the clearest
remaining hole in the "remote access surfaces" story.

**Effects.**

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client` | `AllowBasic` | REG_DWORD | `0` | value-absent (effective: allowed) |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client` | `AllowUnencryptedTraffic` | REG_DWORD | `0` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client` | `AllowDigest` | REG_DWORD | `0` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Service` | `AllowBasic` | REG_DWORD | `0` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Service` | `AllowUnencryptedTraffic` | REG_DWORD | `0` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Service` | `DisableRunAs` | REG_DWORD | `1` | value-absent |
| service `WinRM` | (start type) | | `Disabled` | `Manual` on stock Windows 11 client |

`AllowBasic`, `AllowUnencryptedTraffic` and `DisableRunAs` keys and classes confirmed in
`WindowsRemoteManagement.admx` on 26100. `AllowDigest` is client-side only and is the
`WINRM_CLIENT_ALLOW_DIGEST` policy in the same file.

**Proposed options.** "Disabled" (service `Disabled` plus all six values), "Hardened, service left
alone" (six values only, `Manual` service, for users who genuinely use PowerShell Remoting), and
"Windows default (Stock Default)" with service `Manual` and all values absent.

**Applicability.** Windows 11 and Windows 10. `windows:` gate not needed.

**Risks.** Low to medium. Disabling the service breaks incoming PowerShell Remoting, `Enter-PSSession`
into this machine, some Windows Admin Center scenarios and a handful of remote-management agents.
It does **not** break outbound `Invoke-Command` to other machines from a non-domain client, and it
does not affect SSH remoting. On a stock Windows 11 client there is no WinRM listener configured, so
disabling the service usually changes nothing observable, which is the same situation the corpus
already accepts for `disable_remote_registry`.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsRemoteManagement.admx` (26100); DISA STIG for
Windows 11 V2R2, six WinRM rules; live `Get-Service WinRM` on 26100.

### 6. `powershell_module_transcript_logging` (High)

**Why it is a gap.** The corpus has `powershell_scriptblock_logging` and stops there. Script-block
logging records what a script contained; module logging records pipeline execution details, and
transcription writes a durable on-disk record of an interactive session. The STIG requires both
script-block logging and transcription as separate rules, and CIS lists module logging as Level 1.
Together they close the "attacker ran something in a console" blind spot that script-block logging
alone leaves.

**Effects.** Keys and classes confirmed in `PowerShellExecutionPolicy.admx` on 26100.

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ModuleLogging` | `EnableModuleLogging` | REG_DWORD | `1` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ModuleLogging\ModuleNames` | `*` | REG_SZ | `"*"` | key-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription` | `EnableTranscripting` | REG_DWORD | `1` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription` | `EnableInvocationHeader` | REG_DWORD | `1` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription` | `OutputDirectory` | REG_SZ | a fixed admin-writable path | value-absent (defaults to the user's Documents folder) |

`EnableModuleLogging` requires the `ModuleNames` subkey with at least one entry; `*` for all modules
is what both CIS and the STIG specify.

**Proposed options.** "Module logging and transcription", "Module logging only" (transcription
writes files and some users will not want that), and "Windows default (Stock Default)" with
everything absent and the `ModuleNames` subkey removed.

**Applicability.** Windows 11 and Windows 10, Windows PowerShell 5.1 and PowerShell 7.

**Risks.** Low but not zero. Transcription writes a text file per session; on a machine that runs a
lot of scripted work this accumulates. Pick an explicit `OutputDirectory` under `%ProgramData%`
rather than letting it default into the user's Documents folder, and say so in the info copy.
Module logging with `*` is verbose and will fill the PowerShell operational log faster; pair with
proposal 23 (`event_log_retention`) or accept faster log rollover. Neither setting can lock a user
out.

**Sources.** `C:\Windows\PolicyDefinitions\PowerShellExecutionPolicy.admx` (26100);
<https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows>;
DISA STIG for Windows 11 V2R2 ("PowerShell Transcription must be enabled on Windows 11").

### 7. `restrict_remote_sam` (High)

**Why it is a gap.** The corpus has `restrict_anonymous_enum` (`RestrictAnonymousSAM`,
`RestrictAnonymous`, `EveryoneIncludesAnonymous`), which covers *anonymous* enumeration.
`RestrictRemoteSam` is a different control: it restricts *authenticated* remote SAM enumeration to
Administrators. This is the setting that defeats BloodHound-style local group and local user
harvesting from an ordinary domain or workgroup account.

**Key and value.**
`HKLM\SYSTEM\CurrentControlSet\Control\Lsa`, value name `RestrictRemoteSam`, type **`REG_SZ`**
(not DWORD), hardened value `O:BAG:BAD:(A;;RC;;;BA)`. That SDDL reads: owner Built-in Administrators,
group Built-in Administrators, DACL granting Read Control to Built-in Administrators only.

**Stock default: value-absent.** Verified absent on 26100.4061. When absent, Windows 11 uses a
built-in default that already restricts remote SAM calls to administrators, so this tweak pins a
good state rather than changing behaviour on a healthy machine. That is the same rationale the
corpus already uses for `disable_remote_registry`.

**Applicability.** Windows 11 and Windows 10.

**Risks.** Low. Domain-joined machines using non-admin inventory or asset-management agents that
enumerate local groups will lose that capability. On a consumer machine nothing user-visible changes.
Reversible by deleting the value.

**Sources.**
<https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-restrict-clients-allowed-to-make-remote-sam-calls>;
DISA STIG for Windows 11 V2R2 ("Remote calls to the Security Account Manager (SAM) must be restricted
to Administrators"); CIS Windows 11 v4.0.0 Level 1.

### 8. `block_always_install_elevated` (High)

**Why it is a gap.** `AlwaysInstallElevated` is one of the best-known local privilege escalation
primitives on Windows: when set, any user can run an arbitrary MSI as SYSTEM. It is absent by default,
but it is set by a surprising number of poorly written enterprise deployment scripts, and once set
nothing in Windows warns about it. Pinning it to `0` is exactly the kind of "lock in the safe state"
tweak the corpus already does elsewhere.

**Keys and values.** `REG_DWORD`. Class is Both in `MSI.admx`, so both hives must be written for the
control to be effective.

| Key | Value name | Hardened | Stock default |
| --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Installer` | `AlwaysInstallElevated` | `0` | value-absent |
| `HKCU\SOFTWARE\Policies\Microsoft\Windows\Installer` | `AlwaysInstallElevated` | `0` | value-absent |

Verified absent under HKLM on 26100.4061.

**Applicability.** Windows 11 and Windows 10.

**Risks.** None meaningful. The escalation only exists when *both* hives are set to 1, so writing 0
can never break a working installer flow. Fully reversible.

**Sources.** `C:\Windows\PolicyDefinitions\MSI.admx` (26100); DISA STIG for Windows 11 V2R2
("The Windows Installer feature 'Always install with elevated privileges' must be disabled");
CIS Windows 11 v4.0.0 Level 1; privacy.sexy `windows.yaml` (corroboration).

### 9. `disable_mdns` (High)

**Why it is a gap.** The corpus disables LLMNR (`EnableMulticast` = 0) and NetBIOS over TCP/IP. It
leaves mDNS running. mDNS is the third multicast name-resolution protocol on a modern Windows box and
is exploitable by exactly the same Responder-class poisoning that motivates the LLMNR tweak. Windows
grew a first-party Group Policy control for it, so this is now a clean, documented switch.

**Keys and values.**

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `EnableMDNS` | REG_DWORD | `0` | value-absent |
| `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters` | `EnableMDNS` | REG_DWORD | `0` | value-absent |

The policy value (`DNSClient`) is the documented one, confirmed as the `DNS_MDNS` policy in
`DnsClient.admx` on 26100. The `Dnscache\Parameters` value is the non-policy equivalent and is what
older guidance targets; write the policy value as the mechanism and treat the service value as an
optional second effect. Both verified absent on 26100.4061.

**Applicability.** `windows: { build: ">=26100" }` for the policy value. The `DnsClient.admx`
`DNS_MDNS` policy is present in the 26100 ADMX set; do not claim it for LTSC 2021.

**Risks.** Medium, and higher than the LLMNR tweak. mDNS is what makes `.local` names work, and it is
used by AirPrint and IPP Everywhere printers, Chromecast and Google Cast discovery, some smart-home
apps, and Apple device discovery. Users with a network printer discovered by name will notice.
Requires a reboot or a `Dnscache` restart. Fully reversible.

**Sources.** `C:\Windows\PolicyDefinitions\DnsClient.admx` (26100), policy `DNS_MDNS`;
CIS Windows 11 v4.0.0; corroborated by WinUtil and privacy.sexy.

### 10. `update_feature_control` (High)

**Why it is a gap.** This is the most 24H2-specific item in the whole document and the corpus has
nothing like it. Since 22H2 Microsoft ships new *features* inside monthly quality updates, gated
behind "temporary enterprise feature control", and separately offers "optional content" (optional
non-security preview updates and driver updates) through the same channel. A user who wants their
26100 install to stop changing shape month to month cannot express that with any tweak the corpus
currently has: `defer_quality_updates` delays the patch, and `target_release_version` pins the
feature update, but neither stops a feature arriving inside a monthly cumulative.

**Keys and values.** Key `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, both `REG_DWORD`,
confirmed in `WindowsUpdate.admx` on 26100.

| Value name | Hardened (stability) | Permissive | Stock default |
| --- | --- | --- | --- |
| `AllowTemporaryEnterpriseFeatureControl` | `0` (features stay off until the next feature update) | `1` | value-absent, which behaves as `0` **only on update-managed devices** |
| `SetAllowOptionalContent` | `0` (no optional content) | `1` automatically receive optional updates, `2` also get the latest optional non-security preview | value-absent |

The important nuance for the info copy: `AllowTemporaryEnterpriseFeatureControl` gating applies only
to devices whose updates are policy-managed. On a machine where the corpus has already set
`DeferQualityUpdates` or `TargetReleaseVersion`, the device *is* update-managed, so this policy
becomes live. That interaction is worth stating.

**Applicability.** Windows 11 22H2 with KB5022913 and later, so all of 24H2 and 25H2. Pro, Enterprise,
Education, IoT Enterprise and IoT Enterprise LTSC. Not Home.

**Risks.** Low. Nothing breaks; features simply arrive later. Reversible by deleting both values.
Users who *want* new features early can select the permissive option, which is a legitimate second
state rather than a token opposite.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` (26100), policies
`AllowTemporaryEnterpriseFeatureControl` and `AllowOptionalContent`;
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update>;
<https://learn.microsoft.com/en-us/windows/deployment/update/waas-configure-wufb>.

### 11. `defer_feature_updates` (High)

**Why it is a gap.** The corpus defers *quality* updates and can pin a target feature version, but has
no feature-update deferral. These are different controls with different behaviour: `TargetReleaseVersion`
holds you on a named version until support ends and then jumps; `DeferFeatureUpdates` slides every
feature update back by a chosen number of days, which is what most users actually want ("let other
people find the bugs first").

**Keys and values.** Key `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate`, confirmed in
`WindowsUpdate.admx` on 26100 (policy `DeferFeatureUpdates`).

| Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- |
| `DeferFeatureUpdates` | REG_DWORD | `1` | value-absent |
| `DeferFeatureUpdatesPeriodInDays` | REG_DWORD | `180` suggested, range 0 to 365 | value-absent |

**Proposed options.** Dropdown: "30 days", "180 days", "365 days (maximum)", "Windows default
(Stock Default)" with both absent. Three or more options means the corpus renders it as a dropdown,
consistent with the existing `defer_quality_updates`.

**Applicability.** Windows 10 1607 and later; Pro, Enterprise, Education, IoT Enterprise and IoT
Enterprise LTSC. Not Home. Same edition gate the existing `defer_quality_updates` needs.

**Risks.** Low. Deferral never blocks security updates, only feature updates. It interacts with
`target_release_version`: if both are set, the pin wins. The two tweaks should reference each other in
their info copy so a user does not set contradictory states.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` (26100);
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update>.

### 12. `kernel_dma_protection` (High)

**Why it is a gap.** The task explicitly asked about Kernel DMA Protection and the corpus has nothing.
Kernel DMA Protection itself turns on automatically where firmware supports it, but the two policy
knobs around it are not automatic, and they are the ones that matter for the drive-by DMA attack
class (malicious Thunderbolt or PCIe device plugged into a locked laptop).

**Keys and values.**

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Kernel DMA Protection` | `DeviceEnumerationPolicy` | REG_DWORD | `0` (Block all) | value-absent, effective default `2` (allow after sign-in) |
| `HKLM\SOFTWARE\Policies\Microsoft\FVE` | `DisableExternalDMAUnderLock` | REG_DWORD | `1` | value-absent |

`DeviceEnumerationPolicy` accepts `0` Block all, `1` Allow only after the user signs in, `2` Allow
all. Both values confirmed in `DmaGuard.admx` and `VolumeEncryption.admx` on 26100 and verified
absent on the research machine.

**Proposed options.** "Block all incompatible external devices" (`0` plus `1`), "Allow only after
sign-in" (`1` plus `1`, the safer middle), and "Windows default (Stock Default)" with both absent.

**Applicability.** Requires firmware with DMA remapping (VT-d or AMD IOMMU) and a Kernel DMA
Protection capable platform, which in practice means most business laptops from roughly 2019 onward
and Thunderbolt-equipped machines. The tweak should probe `msinfo32`'s Kernel DMA Protection state
(or the `Win32_DeviceGuard` security properties) and report "not applicable" rather than claiming
success on a desktop with no IOMMU. `DisableExternalDMAUnderLock` requires BitLocker to be active on
the OS volume to have any effect, which ties it to proposal 21.

**Risks.** Medium. `DeviceEnumerationPolicy` = 0 will refuse DMA-incapable external peripherals
outright, which on some machines includes external GPU enclosures, certain Thunderbolt docks, and
older PCIe capture cards. This is why the "allow after sign-in" middle option matters. No lockout
risk: the machine still boots and the internal keyboard still works. Fully reversible.

**Sources.** `C:\Windows\PolicyDefinitions\DmaGuard.admx` and `VolumeEncryption.admx` (26100);
<https://learn.microsoft.com/en-us/windows/security/hardware-security/kernel-dma-protection-for-thunderbolt>;
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-dmaguard>;
DISA STIG for Windows 11 V2R2 ("Windows 11 Kernel (Direct Memory Access) DMA Protection must be
enabled").

### 13. `asr_extended_rules` (Medium)

Six further ASR rules, same key and `REG_SZ` convention as proposal 4, all GUIDs read from the
Microsoft ASR reference. All stock defaults are value-absent.

| GUID | Rule | Note |
| --- | --- | --- |
| `d3e037e1-3eb8-44c8-a917-57927947596d` | Block JavaScript or VBScript from launching downloaded executable content | Low breakage on consumer machines |
| `92e97fa1-2edf-4476-bdd6-9dd0b4dddc7b` | Block Win32 API calls from Office macros | Only bites if Office is installed |
| `b2b3f03d-6a65-4f7b-a9c7-1c7ef74a9ba4` | Block untrusted and unsigned processes that run from USB | Blocks running, not copying |
| `c1db55ab-c21a-4637-bb3f-a12568109d35` | Use advanced protection against ransomware | **Requires cloud protection**, see proposal 1 |
| `d1e49aac-8f56-4280-b9ba-993a6d77406c` | Block process creations originating from PSExec and WMI commands | Highest false-positive risk here |
| `26190899-1602-49e8-8b27-eb1d0a1ce869` | Block Office communication application from creating child processes | Outlook-specific |

**Proposed options.** "Block all six", "Audit all six" (`"2"`), "Windows default (Stock Default)".
Suggest shipping these as one tweak with an audit option rather than six tweaks, matching the
existing `asr_block_office_script_vectors` pattern.

**Risks.** `d1e49aac` is the one to warn about: it blocks processes spawned by PsExec and by WMI
`Win32_Process.Create`, which some legitimate management and automation tooling uses. `c1db55ab` is
documented as erring on the side of caution and blocking files that merely lack a positive
reputation, so brand-new indie software can trip it. Both are reversible instantly. The Adobe Reader
rule (`7674ba52`) and the webshell rule (`a8f5898e`) were considered and are in the rejected list.

**Sources.** <https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference>.

### 14. `ntlm_outgoing_restriction` (Medium)

**Why it is a gap.** Complements proposal 2 at the OS level rather than the SMB level, and adds the
audit-first path CIS and the STIG both want.

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `AuditOutgoingNTLMTraffic` | REG_DWORD | `2` (audit all) | value-absent |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `RestrictSendingNTLMTraffic` | REG_DWORD | `2` (deny all) | value-absent |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `NtlmMinClientSec` | REG_DWORD | `0x20080000` | **`0x20000000`** (verified on 26100) |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` | `NtlmMinServerSec` | REG_DWORD | `0x20080000` | **`0x20000000`** (verified on 26100) |

The session-security pair is the interesting one: the stock 26100 value `0x20000000` requires NTLMv2
session security but **not** 128-bit encryption. `0x20080000` adds `NTLMSSP_NEGOTIATE_128`. That is a
genuine one-bit hardening the corpus does not have, and it is two STIG rules.

**Proposed options.** "Audit outgoing NTLM" (`AuditOutgoingNTLMTraffic`=2 plus the two session-security
values), "Deny outgoing NTLM" (adds `RestrictSendingNTLMTraffic`=2), "128-bit session security only"
(the two session-security values alone), and "Windows default (Stock Default)".

**Risks.** `RestrictSendingNTLMTraffic` = 2 is aggressive on a workgroup machine: any resource
reachable only by NTLM stops working, including most consumer NAS devices. Ship the audit option as
the default recommendation and gate the deny option behind a clear warning. The session-security
values are near-zero risk on any network without pre-Vista hosts. All reversible; none can lock a
user out of the local machine.

**Sources.** DISA STIG for Windows 11 V2R2 (minimum session security for NTLM SSP clients and
servers); CIS Windows 11 v4.0.0;
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-localpoliciessecurityoptions>;
live registry read on 26100.4061.

### 15. `rdp_session_hardening` (Medium)

Extends the existing `rdp_security_hardening` (NLA plus TLS) with five STIG-required settings it does
not cover. Key `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services`, all `REG_DWORD`,
all class Machine, all confirmed in `TerminalServer.admx` on 26100. All stock defaults are
value-absent.

| Value name | Hardened | Meaning |
| --- | --- | --- |
| `MinEncryptionLevel` | `3` | Require High (128-bit) encryption |
| `fEncryptRPCTraffic` | `1` | Require secure RPC for the session host |
| `fPromptForPassword` | `1` | Always prompt for password on connection |
| `DisablePasswordSaving` | `1` | Do not let the RDP client save passwords |
| `fDisableCdm` | `1` | Block local drive redirection into RDP sessions |

**Applicability.** Windows 11 and Windows 10 Pro and above. Meaningful only when RDP is in use, so
this tweak and the corpus's `disable_remote_desktop` are natural opposites and their info copy should
say so.

**Risks.** Low. `fDisableCdm` breaks copying files through a mapped drive in an RDP session, which
some users rely on; keep it as a separable effect. `DisablePasswordSaving` is client-side and affects
this machine connecting out.

**Sources.** `C:\Windows\PolicyDefinitions\TerminalServer.admx` (26100); DISA STIG for Windows 11
V2R2, five separate Remote Desktop rules.

### 16. `task_compatibility_appraiser` (Medium)

**Why it is a gap.** The corpus has `AITEnable` and `DisableInventory` in `privacy.yaml`, which is the
policy-level control, but no task-level control. On a real 26100 machine the appraiser tasks are all
present and in the `Ready` state, so the policy alone leaves them scheduled.

Verified present and `Ready` on 26100.4061:

- `\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser`
- `\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser Exp`
- `\Microsoft\Windows\Application Experience\PcaPatchDbTask`
- `\Microsoft\Windows\Application Experience\StartupAppTask`
- `\Microsoft\Windows\Application Experience\MareBackup`

Note that `\Microsoft\Windows\Application Experience\ProgramDataUpdater` and `AitAgent`, which most
community lists still include, are **not present on 26100**. Do not ship them.

**Options.** "Disabled" for all five, "Windows default (Stock Default)" restoring all to `Ready`.

**Risks.** Low. Disabling `PcaPatchDbTask` and `MareBackup` affects application compatibility shim
maintenance; long-tail legacy apps could in principle regress. `StartupAppTask` feeds the Startup Apps
impact ratings in Task Manager, which will go stale. Nothing breaks. Note the corpus already disables
the `PcaSvc` service, so the `PcaPatchDbTask` entry is consistent with existing policy.

**Sources.** Live `Get-ScheduledTask` enumeration on 26100.4061; corroborated by WinUtil,
privacy.sexy and Sophia Script.

### 17. `task_ceip` (Medium)

Verified present and `Ready` on 26100.4061:

- `\Microsoft\Windows\Customer Experience Improvement Program\Consolidator`
- `\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip`
- `\Microsoft\Windows\PI\Sqm-Tasks` (the kernel CEIP task; `KernelCeipTask` under the CEIP folder no
  longer exists on 26100)

The corpus already disables `\Microsoft\Windows\Autochk\Proxy`, which is part of the same CEIP family,
and sets `CEIPEnable` = 0 in `privacy.yaml`, so this is the remaining piece of a job already started.

**Risks.** Low, near zero. These tasks only submit CEIP telemetry. Reversible.

**Caution.** Do **not** extend this to `\Microsoft\Windows\PI\Secure-Boot-Update`, which lives in the
same folder and delivers Secure Boot DBX revocation updates. That one must stay enabled.

**Sources.** Live `Get-ScheduledTask` enumeration on 26100.4061; WinUtil, privacy.sexy, Sophia Script
in agreement.

### 18. `disable_secondary_logon` (Medium)

Service `seclogon` ("Secondary Logon"). Stock start type `Manual`; hardened `Disabled`. This is a
standalone STIG rule ("The Secondary Logon service must be disabled on Windows 11") and it is absent
from the corpus's 33-service list.

**Why it matters.** `seclogon` is what backs `runas` and the "Run as different user" shell verb. It is
a routine step in local privilege escalation and lateral movement chains, and on a single-user
consumer machine it is essentially never used.

**Risks.** Low. Breaks `runas /user:...`, the Shift plus right-click "Run as different user" menu
entry, and a small number of installers that shell out through it. UAC elevation of the *same* user is
unaffected, so the normal admin experience does not change. Reversible.

**Sources.** DISA STIG for Windows 11 V2R2; corroborated by privacy.sexy and Sophia Script.

### 19. `early_launch_antimalware_policy` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Policies\EarlyLaunch` | `DriverLoadPolicy` | REG_DWORD | `3` (Good, unknown and bad but critical) | value-absent, effective default `3` |

Confirmed in `EarlyLaunchAM.admx` on 26100, policy `POL_DriverLoadPolicy_Name`. Verified absent on the
research machine. Accepted values: `0` Good only, `1` Good and unknown, `3` Good, unknown and bad but
critical, `7` All (no filtering).

The STIG requires this explicitly set to `3` or `1`. Setting it pins a good state and, more usefully,
prevents malware from setting it to `7` to get an unsigned boot driver loaded. Do **not** offer `0`
("Good only") as an option: it will refuse to boot-load any driver the ELAM driver cannot vouch for,
which on unusual hardware can produce a machine that will not start. That would violate the "must not
brick" rule.

**Risks.** Low at `3` or `1`. Requires reboot. Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\EarlyLaunchAM.admx` (26100); DISA STIG for Windows 11 V2R2;
CIS Windows 11 v4.0.0 Level 1.

### 20. `credssp_encryption_oracle` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\CredSSP\Parameters` | `AllowEncryptionOracle` | REG_DWORD | `0` (Force Updated Clients) | value-absent, effective default `2` (Vulnerable) |

Confirmed in `CredSsp.admx` on 26100, policy `AllowEncryptionOracle`. Verified absent on the research
machine. This is the CVE-2018-0886 remediation setting; the OS still ships with the permissive
behaviour so that unpatched RDP servers remain reachable.

**Risks.** Medium in the specific case of connecting out by RDP to an old, unpatched server, which
will start failing. No effect otherwise. Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\CredSsp.admx` (26100); CIS Windows 11 v4.0.0 Level 1.

### 21. `device_encryption_posture` (Medium)

| Key | Value name | Type | Values | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Control\BitLocker` | `PreventDeviceEncryption` | REG_DWORD | `1` prevent automatic device encryption; absent to allow | value-absent, verified on 26100 |

**Why it belongs in the corpus.** Windows 11 24H2 made BitLocker device encryption the default on
clean installs and on qualifying new PCs, including Home. That is a real, user-visible change with two
real consequences: the recovery key is escrowed to a Microsoft account (or, if there is no account,
potentially nowhere the user knows about), and on pre-25H2 builds without hardware-accelerated
BitLocker there is a measurable storage throughput cost on fast NVMe drives. Microsoft documents this
exact registry value as the supported way to opt out. A tweak toolbox that ships a BitLocker-adjacent
category and does not surface this is leaving a significant 24H2 behavioural change invisible.

**Applicability.** `windows: { build: ">=26100" }`. The value only affects *automatic* enablement, so
it is only meaningful before encryption has started.

**Risks.** This one deserves careful copy. Setting `1` reduces data-at-rest protection, which is a
real downgrade, and the tweak must say so plainly. Once device encryption is turned off it does not
re-enable itself automatically; the user must turn it on in Settings. Conversely, on a machine that is
*already* encrypted this value does nothing, so the tweak must probe the BitLocker volume state and
report accurately rather than claiming an effect it did not have.

**Note on the performance argument.** Do not lead with performance. Microsoft shipped
hardware-accelerated BitLocker with 25H2 (reported roughly 70 percent CPU reduction), which materially
shrinks the storage overhead case on current builds. The durable rationale is user control over
encryption and recovery-key custody, not benchmark numbers.

**Sources.** <https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/>
(the "Disable device encryption" table, which gives the exact path, name, type and value);
<https://techcommunity.microsoft.com/blog/windows-itpro-blog/announcing-hardware-accelerated-bitlocker/4474609>.

### 22. `hide_admin_accounts_on_elevation` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\CredUI` | `EnumerateAdministrators` | REG_DWORD | `0` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\CredUI` | `DisablePasswordReveal` | REG_DWORD | `1` | value-absent |

Both confirmed in `CredUI.admx` on 26100 (note the two different keys: `EnumerateAdministrators` is
class Machine under `CurrentVersion\Policies\CredUI`, `DisablePasswordReveal` is class Both under
`Policies\Microsoft\Windows\CredUI`). Both verified absent on the research machine.

`EnumerateAdministrators` = 0 stops the UAC credential prompt listing every local administrator
account by name and picture, which is free reconnaissance for anyone standing at the machine.
`DisablePasswordReveal` = 1 removes the eye icon that reveals typed passwords, a shoulder-surfing
control. Complements the corpus's existing `hide_last_user`.

**Risks.** Very low. Users must type the admin username instead of clicking it. Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\CredUI.admx` (26100); DISA STIG for Windows 11 V2R2
("Administrator accounts must not be enumerated during elevation"); CIS Windows 11 v4.0.0 Level 1.

### 23. `event_log_retention` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Application` | `MaxSize` | REG_DWORD | `32768` (KB) | value-absent (channel default 20480 KB) |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\System` | `MaxSize` | REG_DWORD | `32768` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Security` | `MaxSize` | REG_DWORD | `196608` (192 MB) or `1024000` for the STIG value | value-absent |

Keys confirmed in `EventLog.admx` on 26100 (policies `Channel_LogMaxSize_1` through `_4`).

**Why it belongs.** The corpus already enables logon auditing (`audit_logon_events`) and script-block
logging. Both generate events into logs that default to a size where a busy day rolls them over before
anyone investigates. Turning on auditing without sizing the log is half a control.

**Options.** Dropdown: "Standard (32 MB app/system, 192 MB security)", "Large (STIG: 1 GB security)",
"Windows default (Stock Default)".

**Risks.** Disk consumption only. The STIG's 1024000 KB security log is roughly 1 GB and should be the
non-default option. Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\EventLog.admx` (26100); DISA STIG for Windows 11 V2R2
(three separate event-log size rules).

### 24. `audit_process_creation_cmdline` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\Audit` | `ProcessCreationIncludeCmdLine_Enabled` | REG_DWORD | `1` | value-absent |

Confirmed in `AuditSettings.admx` on 26100 (policy `IncludeCmdLine`). Pairs with an `auditpol` effect
enabling the Process Creation subcategory
(`auditpol /set /subcategory:"{0CCE922B-69AE-11D9-BED3-505054503030}" /success:enable`), following the
exact pattern the corpus already uses in `audit_logon_events`.

**Why it belongs.** Event 4688 without the command line tells you `powershell.exe ran`. With the
command line it tells you what it ran. This is the single highest-value addition to the corpus's
auditing story and it is two STIG rules.

**Risks.** Low but genuine and worth stating: command lines can contain secrets (passwords passed as
arguments), and those then sit in the Security log, readable by administrators. Pair with proposal 23.
Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\AuditSettings.admx` (26100); DISA STIG for Windows 11 V2R2
("Command line data must be included in process creation events", "audit Detailed Tracking - Process
Creation successes").

### 25. `disable_wer_service` (Medium)

Service `WerSvc` ("Windows Error Reporting Service"). Stock start type `Manual` (verified on 26100);
hardened `Disabled`.

The corpus already sets the WER `Disabled` policy value in `privacy.yaml` and disables the
`QueueReporting` scheduled task in `services.yaml`. The service itself is the third leg and is absent.
Disabling it completes a job already two-thirds done and stops WER consuming CPU on a crash.

**Risks.** Low. Crash dumps stop being collected and submitted, which removes a diagnostic aid; some
applications query WER state. Reversible.

**Sources.** Live service enumeration on 26100.4061; WinUtil, privacy.sexy and Sophia Script in
agreement on `WerSvc`.

### 26. `spooler_remote_rpc_off` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Control\Print` | `RegisterSpoolerRemoteRpcEndPoint` | REG_DWORD | `2` (do not register) | value-absent, effective `1` (registered) |
| `HKLM\SYSTEM\CurrentControlSet\Control\Print` | `RpcAuthnLevelPrivacyEnabled` | REG_DWORD | `1` | value-absent, effective `1` on patched builds |

Both verified absent on 26100.4061.

**Why it belongs.** The corpus offers a binary choice today: disable the Print Spooler entirely
(`disable_print_spooler`) or restrict driver installation (`printnightmare_point_and_print`). There is
no middle option for the very common user who needs to print locally but has no reason to expose the
spooler's remote RPC endpoint to the network. `RegisterSpoolerRemoteRpcEndPoint` = 2 is exactly that
middle option and is the mitigation Microsoft published for the remote half of the PrintNightmare
class.

**Risks.** Low. Breaks remote printing *to* this machine and printer sharing from this machine. Local
and network-printer printing are unaffected. Requires a spooler restart. Reversible.

**Sources.** Microsoft print spooler RPC connection settings guidance; corroborated by CIS Windows 11
v4.0.0 and by the STIG's related Point and Print rules.

### 27. `block_insider_builds_policy` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `ManagePreviewBuilds` | REG_DWORD | `1` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` | `ManagePreviewBuildsPolicyValue` | REG_DWORD | `0` (disable preview builds) | value-absent |

Confirmed in `WindowsUpdate.admx` on 26100 (policy `ManagePreviewBuilds`). The corpus currently
addresses Insider builds only by disabling the `wisvc` service, which is the weaker lever: the service
can be restarted and the policy cannot be bypassed from the Settings UI. Keep both.

**Risks.** Low. Blocks enrolment in the Insider Program. Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsUpdate.admx` (26100);
<https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-update>.

### 28. `autoplay_non_volume` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` | `NoAutoplayfornonVolume` | REG_DWORD | `1` | value-absent |

Confirmed in `AutoPlay.admx` on 26100 (class Both, so consider writing HKCU as well). Verified absent
on the research machine.

The corpus's `disable_autorun` covers `NoAutorun` and `NoDriveTypeAutoRun`, which handle drive-letter
volumes. `NoAutoplayfornonVolume` handles MTP devices such as phones and cameras, which do not get a
drive letter and are therefore not covered. Three STIG rules exist in this family and the corpus
implements two of them. Cheap completion of an existing tweak rather than a new one; the cleanest
approach is to add the effect to `disable_autorun`.

**Risks.** None meaningful. Reversible.

**Sources.** `C:\Windows\PolicyDefinitions\AutoPlay.admx` (26100); DISA STIG for Windows 11 V2R2
("Autoplay must be turned off for non-volume devices").

### 29. `remote_uac_token_filter` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` | `LocalAccountTokenFilterPolicy` | REG_DWORD | `0` | value-absent, effective `0` |

When this value is `1`, local administrator accounts connecting over the network receive a full,
unfiltered admin token, which is precisely what pass-the-hash lateral movement wants. It is absent by
default, but it is set to `1` by a great many "fix my network shares" guides, by some remote-support
tools, and by several popular tweak scripts. **It was found set to `1` on the research machine**,
which illustrates the point: this is a control worth pinning because something will have flipped it.

**Risks.** Setting `0` breaks remote administration using a local (non-domain) admin account: remote
`C$` access, remote WMI and remote MMC from another machine will start failing for local accounts.
That is the intended effect but it must be said clearly. Reversible.

**Sources.** DISA STIG for Windows 11 V2R2 ("Local administrator accounts must have their privileged
token filtered"); CIS Windows 11 v4.0.0; live registry read on 26100.4061.

### 30. `disable_internet_connection_sharing` (Medium)

Service `SharedAccess` ("Internet Connection Sharing (ICS)"). Stock start type `Manual` (verified on
26100); hardened `Disabled`. Optionally paired with
`HKLM\SOFTWARE\Policies\Microsoft\Windows\Network Connections\NC_ShowSharedAccessUI` = 0.

The STIG has a dedicated rule ("Internet connection sharing must be disabled"). ICS can turn a machine
into an unmanaged router and NAT, and it is almost never wanted on a client.

**Risks.** Medium and specific: `SharedAccess` also backs **Mobile Hotspot**. Disabling it removes the
Mobile Hotspot feature. That is a user-visible loss and must be in the info copy. Reversible.

**Sources.** DISA STIG for Windows 11 V2R2; live service enumeration on 26100.4061.

### 31. `reserved_storage_off` (Medium)

Mechanism: `Set-WindowsReservedStorageState -State Disabled` (and `-State Enabled` to undo);
probe with `Get-WindowsReservedStorageState`. This is a first-party Microsoft cmdlet, not a registry
poke, and it reclaims roughly 7 GB on a typical 24H2 install.

**Why it belongs.** The corpus's performance category currently offers no disk-reclamation tweak at
all beyond Storage Sense (which it disables). Reserved storage is the largest single reclaimable
allocation on a stock 24H2 image and Microsoft ships a supported, reversible switch for it.

**Applicability.** Windows 11 and Windows 10 1903 and later. On the LTSC/IoT research image it was
already `Disabled`, so the tweak should probe rather than assume. Note the cmdlet fails while an
update is pending.

**Risks.** Medium and worth stating honestly: with reserved storage off, a feature update on a nearly
full disk can fail or require the user to free space manually. This is a disk-space tweak, not a
speed tweak, and the copy should not imply otherwise.

**Sources.** Microsoft `Set-WindowsReservedStorageState` documentation; live
`Get-WindowsReservedStorageState` on 26100.4061.

### 32. `firewall_logging_and_merge` (Medium)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<profile>\Logging` | `LogDroppedPackets` | REG_DWORD | `1` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<profile>\Logging` | `LogFileSize` | REG_DWORD | `16384` (KB) | value-absent (default 4096) |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\<profile>\Logging` | `LogFilePath` | REG_SZ | `%SystemRoot%\System32\logfiles\firewall\<profile>fw.log` | value-absent |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile` | `AllowLocalPolicyMerge` | REG_DWORD | `0` | value-absent, effective `1` |

`<profile>` is `DomainProfile`, `StandardProfile` (private) and `PublicProfile`. The `Logging` subkeys
are confirmed in `WindowsFirewall.admx` on 26100 (policies `WF_Logging_Name_1` and `_2`).
`AllowLocalPolicyMerge` is a CIS control not present in the shipped ADMX; it is written directly.

Extends the corpus's existing `firewall_all_profiles`, which turns the firewall on and blocks inbound
but neither logs nor prevents applications from silently adding their own allow rules.

**Risks.** `AllowLocalPolicyMerge` = 0 on the Public profile is the aggressive part: applications can
no longer add firewall rules for themselves, so the familiar "Windows Defender Firewall has blocked
some features of this app" flow stops working and games or servers will silently fail to accept
inbound connections on public networks. Keep it as a separate, clearly warned option; logging alone
should be the default recommendation. All reversible.

**Sources.** `C:\Windows\PolicyDefinitions\WindowsFirewall.admx` (26100); CIS Windows 11 v4.0.0
Level 1 (firewall logging and local policy merge recommendations for all three profiles).

### 33. `enable_sehop` (Low)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel` | `DisableExceptionChainValidation` | REG_DWORD | `0` | value-absent, effective enabled on 64-bit |

Verified absent on 26100.4061. Rated Low because on 64-bit Windows SEHOP is already on and this only
pins it (and matters for 32-bit processes). It is a STIG rule with a specific check, so it is
auditable and free, but it is not a behavioural improvement on a healthy 24H2 machine. Requires
reboot. Reversible.

**Sources.** DISA STIG for Windows 11 V2R2 ("Structured Exception Handling Overwrite Protection
(SEHOP) must be enabled"); corroborated by privacy.sexy.

### 34. `no_index_encrypted_files` (Low)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` | `AllowIndexingEncryptedStoresOrItems` | REG_DWORD | `0` | value-absent, effective disabled |

Verified absent on 26100.4061. Not present in the shipped ADMX under that value name; written directly
per the STIG check text. Prevents the search index (an unencrypted database) from containing content
extracted from EFS-encrypted files. Low value because the default already matches, and because the
corpus disables Windows Search indexing entirely in `performance.yaml` anyway, making this a no-op for
anyone who applied that. Reversible.

**Sources.** DISA STIG for Windows 11 V2R2 ("Indexing of encrypted files must be turned off");
CIS Windows 11 v4.0.0.

### 35. `disable_pku2u_online_id` (Low)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\pku2u` | `AllowOnlineID` | REG_DWORD | `0` | value-absent, effective disabled |

Verified absent on 26100.4061. Blocks PKU2U authentication using online identities, closing a
peer-to-peer authentication path between non-domain machines. Low value on a standalone consumer
machine because the default already matches; included because it is a STIG rule and cheap to pin.

**Risks.** Can break some device-to-device sharing scenarios (Nearby Sharing to a non-domain peer,
certain HomeGroup-successor flows). Reversible.

**Sources.** DISA STIG for Windows 11 V2R2 ("PKU2U authentication using online identities must be
prevented"); CIS Windows 11 v4.0.0.

### 36. `mss_network_stack_hardening` (Low)

The MSS (legacy) IP-stack settings from the Microsoft Security Compliance Toolkit. Not in the shipped
ADMX (they arrive with `MSS-legacy.admx` from the toolkit) and written directly. All verified absent
on 26100.4061.

| Key | Value name | Type | Hardened |
| --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters` | `DisableIPSourceRouting` | REG_DWORD | `2` |
| `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters` | `DisableIPSourceRouting` | REG_DWORD | `2` |
| `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters` | `EnableICMPRedirect` | REG_DWORD | `0` |
| `HKLM\SYSTEM\CurrentControlSet\Services\Netbt\Parameters` | `NoNameReleaseOnDemand` | REG_DWORD | `1` |
| `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters` | `PerformRouterDiscovery` | REG_DWORD | `0` |

Four of these are individual STIG rules. Rated Low deliberately: on a consumer machine behind a NAT
router the real-world exposure to IP source routing and ICMP redirect attacks is close to nil, and
`NoNameReleaseOnDemand` only matters where NetBIOS is in use, which the corpus already disables.
Include them if the goal is auditable benchmark alignment; skip them if the goal is user-visible
benefit.

**Important:** this is **not** the "TCP registry pack" performance myth. These are security controls
from the Microsoft baseline and none of them claim or produce a throughput or latency change. Do not
let them be conflated with the rejected performance items.

**Risks.** Low. `PerformRouterDiscovery` = 0 is harmless on any network with DHCP. Requires reboot.
Reversible.

**Sources.** DISA STIG for Windows 11 V2R2 (four rules: IPv6 source routing, IP source routing, ICMP
redirects, NetBIOS name release); Microsoft Security Compliance Toolkit MSS settings.

### 37. `ntfs_disable_8dot3` (Low)

| Key | Value name | Type | Hardened | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` | `NtfsDisable8dot3NameCreation` | REG_DWORD | `1` (disable on all volumes) | **`2`** (per-volume: enabled on the system volume, disabled elsewhere), verified on 26100 |

Also exposed as the `ShortNameCreationSettings` policy under `HKLM\SYSTEM\CurrentControlSet\Policies`
in `FileSys.admx` on 26100.

Microsoft's own file-server performance tuning guidance states that disabling 8.3 name creation
improves file-creation performance in directories with large numbers of files. That is a real,
first-party, measured claim, which is why this is included at all. It is rated **Low** because the
effect on a typical client workload is small and the corpus already covers the sibling value
(`NtfsDisableLastAccessUpdate`) in the same key, so this is an increment rather than a new capability.

**Risks.** Low but real: some very old 16-bit installers and a few applications that hardcode short
paths will fail. Existing short names are not removed, so the change only affects newly created files.
Reversible by restoring `2`.

**Sources.** `C:\Windows\PolicyDefinitions\FileSys.admx` (26100);
<https://learn.microsoft.com/en-us/windows-server/administration/performance-tuning/role/file-server/>;
live registry read on 26100.4061.

### 38. `require_doh` (Low)

| Key | Value name | Type | Values | Stock default |
| --- | --- | --- | --- | --- |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient` | `DoHPolicy` | REG_DWORD | `1` Allow DoH, `2` Prohibit DoH, `3` Require DoH | value-absent |

Confirmed in `DnsClient.admx` on 26100 (policy `DNS_Doh`, alongside `DohPolicySetting` and
`DotPolicySetting`). Verified absent on the research machine.

The corpus's existing `dns_over_https` uses `EnableAutoDoh` = 2, which is the *auto-upgrade* mechanism:
use DoH where the configured resolver supports it, fall back to plaintext otherwise. `DoHPolicy` = 3 is
the enforcing mechanism: fail resolution rather than fall back. These are different guarantees and the
enforcing one is what a privacy-motivated user actually wants. Best implemented as an added option on
the existing tweak rather than a new one.

**Risks.** Medium if selected. If the configured DNS servers do not speak DoH, name resolution fails
completely and the machine appears to have no internet. Microsoft explicitly warns against Require DoH
on domain-joined machines. Any UI for this must make the failure mode obvious and the revert path easy.
Also note 25H2 added DNS over TLS as a separate `DotPolicySetting` value in the same policy, which is
worth a future look.

**Sources.** `C:\Windows\PolicyDefinitions\DnsClient.admx` (26100), policy `DNS_Doh`;
<https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support>.

---

## C. Rejected, with reasons

Recorded so these decisions are not rediscovered and relitigated.

### C.1 Performance and gaming myths (explicitly excluded, plus the ones checked and dropped)

| Candidate | Reason for rejection |
| --- | --- |
| Timer resolution forcing, HPET, `bcdedit /set useplatformclock`, `disabledynamictick` | Excluded by scope. Independently: since Windows 11 22H2 a process requesting a high timer resolution no longer changes the global resolution, so the classic mechanism no longer does what the guides claim on 26100. |
| `Win32PrioritySeparation` | Excluded by scope. No measured evidence at 26100; the value is a foreground/background quantum hint with no reproducible gaming benefit. |
| QoS `NonBestEffortLimit` (the "20 percent reserved bandwidth" claim) | Excluded by scope. The premise is false: the reservation only applies to applications that explicitly request QoS. |
| Pagefile disabling or fixed sizing | Excluded by scope. Causes hard failures in applications that commit large amounts of memory, and provides no measured gain. |
| Standby list clearing / `EmptyStandbyList` | Excluded by scope. Discards a cache the OS then has to refill; measured effect is negative on average. |
| `LargeSystemCache` | Excluded by scope. Server-oriented, deprecated behaviour on client SKUs. |
| Prefetch and Superfetch registry disabling | Excluded by scope. The corpus already exposes the supported control (`SysMain` service) in `memory_prefetch_mode`. |
| TCP registry packs (`TcpAckFrequency`, `TCPNoDelay`, `TcpWindowSize`, autotuning off, "gaming netsh" bundles) | Excluded by scope. Per-interface, undocumented, and repeatedly shown not to reduce game latency on modern stacks. |
| MMCSS `SystemProfile\Tasks\Games` tuning (`GPU Priority` = 8, `Scheduling Category` = High, `SFIO Priority` = High) | Considered and rejected. No published measurement of a frame-time or latency improvement on 26100; the stock values are already the tuned ones for most fields, and the commonly circulated "fix" mostly rewrites defaults. Fails the measured-evidence bar. |
| MSI mode (`MSISupported` under the device's `Interrupt Management` key) | Rejected. The registry path is per-device-instance (`PCI\VEN_...\Device Parameters\...`), so it is not expressible as a portable tweak; setting it on a device whose driver does not support MSI can produce an unbootable system. Modern GPUs already default to MSI. |
| `DisablePagingExecutive` | Rejected. Long-standing myth; no measured benefit, and it removes the kernel's ability to page under memory pressure. |
| `SvcHostSplitThresholdInKB` (force service grouping) | Rejected. Trades process isolation for a small reduction in process count; no measured performance benefit and it weakens service isolation. |
| `NtfsMemoryUsage` | Rejected. Ignored on modern builds. |
| `TdrDelay` / `TdrDdiDelay` | Rejected. A GPU-hang timeout, not a performance setting. Increasing it masks driver crashes rather than improving anything. |
| NVIDIA, AMD and Intel latency guidance | Rejected as tweaks. Reviewed: NVIDIA Reflex and Low Latency Mode Ultra, AMD Anti-Lag, Intel Presentmon-driven guidance. All are in-driver or in-game settings with no supported, portable OS registry surface. The one OS-level prerequisite they share, hardware-accelerated GPU scheduling, is **already in the corpus** (`enable_gpu_scheduling`). There is nothing left to add. |
| PCI Express ASPM off, USB 3 link power management off (via `powercfg` subgroup GUIDs) | Rejected for now. Plausible latency mechanism for some NICs and USB audio interfaces, but no published measurement at 26100 that survives scrutiny. Fails the measured-evidence bar. Revisit if a credible measurement appears. |
| Processor `IDLEDISABLE`, minimum processor state 100 percent, core parking overrides | Rejected. Either duplicated by the corpus's existing Ultimate Performance power plan or thermally harmful (`IDLEDISABLE` prevents C-state entry entirely). |
| Windows 11 power mode overlay (`powercfg /overlaysetactive`) | Rejected as a duplicate. The corpus's `ultimate_performance_power_plan` already covers this ground; adding a second, overlapping control invites contradictory states. |
| `ClearPageFileAtShutdown` | Rejected. CIS Level 2. Adds minutes to every shutdown for a benefit that BitLocker already provides on any encrypted machine. Cost far exceeds value. |
| `Ndu` (Network Data Usage driver) and `DusmSvc` disabling | Rejected as Low-value noise. Small non-paged pool saving; breaks the Settings data-usage page and Task Manager network columns. Not worth a corpus slot. |

### C.2 Security items considered and rejected

| Candidate | Reason for rejection |
| --- | --- |
| Smart App Control (`HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy\VerifiedAndReputablePolicyState`) | **Rejected on reversibility.** Smart App Control can be turned off but cannot be turned back on without a clean Windows install. That is a one-way door and directly violates the corpus's reversibility requirement and ADR-0002's snapshot model. |
| Defender Tamper Protection (`Windows Defender\Features\TamperProtection`) | Rejected. The value is protected; writing it directly is unsupported and Defender reverts it. Any tweak here would silently fail, breaking the "did-it-work" contract. |
| `DisableAntiSpyware` | Rejected. Ignored on current builds when Tamper Protection is on. Would report success while doing nothing. |
| Windows LAPS (`HKLM\SOFTWARE\Microsoft\Policies\LAPS\BackupDirectory`) | Rejected. Requires Active Directory or Microsoft Entra ID as a backup directory. No function on a standalone consumer machine, which is this corpus's target. |
| Secure Boot enablement | Rejected. A UEFI firmware setting. Not settable from the running OS, so there is no reversible OS-level mechanism. The corpus can only report state, not tweak it. |
| FIPS mode (`FipsAlgorithmPolicy\Enabled`) | Rejected. Microsoft advises against it outside specific compliance mandates; it breaks applications that use non-FIPS-validated crypto and provides no security improvement for a consumer. |
| `WaaSMedicSvc` disabling | Rejected on reversibility and safety. The service key requires an ownership change to modify, the corpus's elevation broker would have to permanently alter an ACL, and undo cannot reliably restore the original security descriptor. |
| `DoSvc` (Delivery Optimization service), `UsoSvc`, `wuauserv`, `BITS` disabling | Rejected. All are required for Windows Update to function. The corpus already handles the actual user concern (peer-to-peer sharing) correctly via `DODownloadMode` = 0. |
| Microsoft Edge updater services (`edgeupdate`, `edgeupdatem`) | Rejected. Disabling them stops Edge receiving security updates while leaving Edge installed, which is a net security loss. |
| Block Microsoft accounts (`Policies\System\NoConnectedUser` = 3) | Rejected for this corpus. It is a CIS Level 1 control, but on a consumer machine it prevents signing in with a Microsoft account, which can leave a user unable to reach their own device encryption recovery key or Store purchases. Too close to a lockout for a general-audience tool. |
| Account lockout and password policy (`net accounts`: 3 bad attempts, 15 minute lockout, 14 character minimum) | Rejected for this corpus. Real STIG controls, but a 3-attempt lockout on a single-user machine is a self-denial-of-service waiting to happen, and Windows 11 already ships a default local lockout threshold of 10. The risk of locking a user out of their own machine outweighs the benefit. |
| `DisableDomainCreds` = 1 (no storage of network credentials) | Rejected. CIS Level 2. Wipes Credential Manager entries for network shares and RDP, which consumers rely on daily. |
| Kerberos `SupportedEncryptionTypes` (`Policies\System\Kerberos\Parameters`) | Rejected as out of audience. A domain-only control; on a workgroup machine Kerberos is not used for anything the setting affects. Dropped to zero practical value for the target user. |
| LDAP client signing (`Services\LDAP\LDAPClientIntegrity`) | Rejected as already default. **Verified `1` on 26100**, which is the CIS-required value. Nothing to change. |
| `RestrictNullSessAccess` | Rejected as already default. **Verified `1` on 26100.** |
| `LimitBlankPasswordUse` | Rejected as already default. **Verified `1` on 26100.** |
| SMB server auth rate limiter (`EnableAuthRateLimiter`, `InvalidAuthenticationDelayTimeInMs`) | Rejected as already default. Verified `True` / `2000` ms on 26100 via `Get-SmbServerConfiguration`. 24H2 ships this on. |
| SMB dialect management (`MinSMB2Dialect`, `MaxSMB2Dialect`) | Rejected on breakage-to-benefit ratio. Real 24H2 feature, but restricting to SMB 3.1.1 only breaks most consumer NAS devices and network printers, and the corpus already removes SMBv1, which is the dialect that actually matters. |
| SMB client `RequireEncryption` | Deferred rather than rejected outright. Verified `False` by default on 26100 and it is a genuine 24H2 feature, but requiring encryption on *all* outbound SMB breaks every server that does not offer SMB 3 encryption, which includes most consumer NAS. Lower value than `BlockNTLM` for the same breakage cost. Revisit if the maintainer wants a full SMB hardening bundle. |
| Hardened UNC paths (`Network Provider\HardenedPaths` for `\\*\SYSVOL` and `\\*\NETLOGON`) | Rejected as domain-only. Those shares do not exist on a workgroup machine. |
| `fMinimizeConnections` / `fBlockNonDomain` (WcmSvc) | Rejected as domain-only. The STIG rules are about not bridging a domain network and the internet; meaningless standalone. |
| Exploit Protection / process mitigation policy | Rejected on mechanism. The supported configuration surface is an exported XML imported with `Set-ProcessMitigation`, plus an opaque `MitigationOptions` binary blob per image under Image File Execution Options. Neither maps onto the corpus's registry-value effect model, and a per-application mitigation set is not something a general-audience toolbox should ship blind. The corpus already covers the two system-wide pieces that matter (DEP via the OS default, SEHOP as proposal 33). |
| SafeDllSearchMode | Rejected as already-default behaviour. Absent on 26100 and the OS default is safe search order. Pinning it adds an auditable value but no behavioural change. |
| `ScreenSaverGracePeriod` | Rejected as negligible. Five-second grace after the screensaver engages. Not worth a corpus slot. |
| `NoAutoplayfornonVolume` as a standalone tweak | Rejected as a standalone; **accepted as an added effect on `disable_autorun`** (proposal 28). Splitting one AutoPlay concept across two tweaks would be worse UX. |
| Windows Hello convenience PIN disable, lock-screen camera, toast-on-lock-screen | Rejected as out of my assigned domains (interface and privacy). Flagged for whoever owns those files. |
| Legal notice text (`LegalNoticeCaption` / `LegalNoticeText`) | Rejected. A DoD banner requirement with no consumer value. |
| User rights assignments (the roughly 30 STIG `Se*Privilege` rules) | Rejected on mechanism and risk. They require `secedit` template manipulation rather than registry writes, snapshot and restore of a user-rights database is materially harder than a registry value, and a mistake here can produce an unbootable or unusable machine. Out of proportion for this corpus. |

### C.3 Services and scheduled tasks considered and rejected

| Candidate | Reason for rejection |
| --- | --- |
| `\Microsoft\Windows\Application Experience\ProgramDataUpdater`, `AitAgent` | **Not present on 26100.** Verified by live task enumeration. Every community list still ships them. Do not add. |
| `\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipTask` | Not present on 26100 under that path; superseded by `\Microsoft\Windows\PI\Sqm-Tasks`, which proposal 17 covers. |
| `\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector` | Already `Disabled` by default on 26100 (verified). The corpus tweak that targets it is confirming a default, which is fine, but there is nothing further to add. |
| `\Microsoft\Windows\Maps\MapsUpdateTask` | Already `Disabled` by default on 26100 (verified). |
| `\Microsoft\Windows\PI\Secure-Boot-Update` | **Must not be disabled.** Delivers Secure Boot DBX revocation updates. Called out explicitly because it sits in the same folder as `Sqm-Tasks`. |
| `\Microsoft\Windows\Registry\RegIdleBackup` | Rejected as a no-op. The task is `Ready` but has not actually created registry backups since Windows 10 1803. Disabling it changes nothing. |
| `\Microsoft\Windows\UpdateOrchestrator\*` | Rejected. Breaking update orchestration produces a machine that silently stops patching. |
| `diagnosticshub.standardcollector.service` | **Not present on 26100.** Verified. Still listed by several community tools. |
| `SgrmBroker` (System Guard Runtime Monitor Broker) | **Not present on 26100.** Verified. Microsoft deprecated it. |
| `GamingServices`, `GamingServicesNet` | Not present on the LTSC/IoT research image; on a consumer 26100 image they exist but back Xbox Game Pass installs. Disabling breaks Game Pass. Rejected as too high a breakage cost for a "safe to disable" claim. |
| `DPS`, `WdiServiceHost`, `WdiSystemHost` | Rejected. Disabling the Diagnostic Policy Service breaks the network troubleshooters and the Settings troubleshooting flows, which are the first thing a non-expert user reaches for when something goes wrong. Poor trade for a small idle-memory saving. |
| `stisvc` (Windows Image Acquisition) | Rejected. Backs scanners **and** some camera flows. Too likely to break something the user did not associate with the tweak. |
| `wlidsvc` (Microsoft Account Sign-in Assistant) | Rejected. Breaks Microsoft account sign-in and Store. |
| `WpnService` (push notifications), `cbdhsvc` (clipboard history) | Rejected. Break visible, wanted features. |
| `RemoteAccess`, `NetTcpPortSharing`, `dmwappushservice`, `CscService`, `RmSvc` | Rejected as already `Disabled` by default on 26100 (verified). Nothing to gain. |
| `MessagingService`, `PimIndexMaintenanceSvc`, `UnistoreSvc`, `UserDataSvc` | Deferred rather than rejected. Genuinely safe to disable when Mail, People and Contacts sync are unused, but they are per-user template services whose instances carry a `_XXXXX` suffix, which the corpus's service effect model does not currently express. Revisit if template-service support is added. |
| `BcastDVRUserService`, `WFDSConMgrSvc`, `DevicePickerUserSvc`, `DevicesFlowUserSvc`, `lltdsvc`, `icssvc`, `TieringEngineService`, `WEPHOSTSVC` | Rejected as Low-value. Each is `Manual` and idle by default; disabling them frees nothing measurable and adds surface area to the corpus for no user-visible gain. |

---

## D. Defects found while gap hunting

Not gaps, but both were surfaced by this work and both are live correctness problems.

**D.1 `require_smb_signing` writes an unsafe "Windows default".** The tweak's stock-default option sets
`workstation_signing: 0` and `server_signing: 0`. On 26100 the real defaults are
`RequireSecuritySignature` **value-absent** on both `LanmanWorkstation\Parameters` and
`LanmanServer\Parameters`, while the *effective* state is required, because 24H2 made SMB signing
mandatory by default on Home, Pro, Education and Enterprise. Verified on 26100.4061:
`Get-SmbClientConfiguration | FL RequireSecuritySignature` returns `True` with the registry value
absent. A user who selects "Windows default (Stock Default)" therefore writes an explicit `0` and ends
up **less secure than a stock machine**, with SMB signing genuinely disabled. The option should write
`absent` for both values, and the info copy should say that 24H2 already requires signing.

**D.2 `task_device_census` targets a task that does not exist on 26100.** The tweak targets
`\Microsoft\Windows\Device Information\Devicecensus`. Live enumeration on 26100.4061 shows the tasks in
that folder are `\Microsoft\Windows\Device Information\Device` and
`\Microsoft\Windows\Device Information\Device User`. As written the tweak finds nothing, and depending
on how the probe treats a missing task it will either report "already applied" or a permanent Needs
Attention. Retarget to both current names.

---

## E. Unknowns

1. **`SpynetReporting` and `SubmitSamplesConsent` true policy defaults.** The research machine had
   `SpynetReporting` = 0 and `MpCloudBlockLevel` = 0 already set under the policy key, so the
   value-absent claim in proposal 1 rests on documentation rather than observation. Confirm on a
   pristine 26100 image before shipping the "Windows default" option.
2. **SMB client `BlockNTLM` registry backing.** The mechanism is confirmed (the cmdlet, and the
   Server 2025 Group Policy), but the exact registry value name and hive that
   `Set-SmbClientConfiguration -BlockNTLM` writes were not confirmed. `LanmanWorkstation.admx` on
   26100 does not contain the policy, and `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation`
   was empty on the research machine. Proposal 2 therefore specifies the cmdlet as the mechanism. If
   the maintainer prefers a registry effect, set the value with the cmdlet on a test machine and diff
   the registry to find it.
3. **Enhanced Phishing Protection default state.** Microsoft describes the feature as shipping in
   audit mode with the policy unset, but does not publish the effective per-notification defaults. The
   five values were verified absent on 26100; what the OS actually does with them absent was not
   directly observed.
4. **Kernel DMA Protection applicability probe.** Proposal 12 assumes the corpus can determine whether
   the platform supports Kernel DMA Protection before claiming an effect. `Win32_DeviceGuard`
   `AvailableSecurityProperties` returned `1,2,3,4,5,6,7` on the research machine; the exact property
   id that indicates DMA remapping support should be confirmed against Microsoft's enumeration before
   the probe is written.
5. **`MpBafsExtendedTimeout` upper bound.** Proposal 1 suggests `50` seconds on the strength of
   Microsoft's block-at-first-sight guidance. The ADMX element's declared maximum was not read; confirm
   before shipping so an out-of-range value cannot be written.
6. **CIS Windows 11 v4.0.0 exact recommendation numbers.** The benchmark was used as a corroborating
   tier throughout, but the PDF was not parsed section by section, so individual CIS recommendation
   numbers are not cited per proposal. Every CIS-attributed item in this document is independently
   backed by either the shipped ADMX or the STIG, so no proposal depends on CIS alone. If the
   maintainer wants per-item CIS numbering in the corpus info copy, that is a separate parsing pass.
7. **Windows 10 IoT Enterprise LTSC 2021 applicability.** Every "verified" claim in this document was
   read from a 26100 machine. None of the proposals were checked against a 19044 image. The
   policy-backed items with `SUPPORTED_Windows_10_0` in their ADMX almost certainly apply, but the
   24H2-specific ones (proposals 2, 9, 10, 21) certainly do not and are gated accordingly.
