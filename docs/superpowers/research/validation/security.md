# Security Hardening tweak validation

Rebuilt 2026-07-27. Source corpus: `src-tauri/tweaks/security.yaml` (37 shipped tweaks) plus 25
verified additions carried in from `_verify-gaps-b-high.md` and `_verify-gaps-b-medlow.md`.
**62 tweaks in total.**

Target platform: **Windows 11 24H2 (build 26100) and newer, including 25H2 (26200)**. Windows 10 IoT
Enterprise LTSC 2021 (19044) is a low-priority secondary target and is called out per entry only where
the behaviour differs.

This document consolidates three earlier passes over the shipped corpus (the original category
validation, a second adversarial pass against on-box 26100.4061 evidence, and a third revert-safety
pass) with the two gap-verification passes that produced the additions. It is the authoritative
research record for the `security` category. Every entry carries a ready-to-paste `info:` block in the
shape defined by `_INFO_TEMPLATE.md`.

**Scope note on the count.** The rebuild brief named 21 additions and 58 tweaks total. The explicit
list in that brief enumerates **25** tweak ids (9 from `_verify-gaps-b-high.md`, 16 from
`_verify-gaps-b-medlow.md`). The enumeration governs, so this document carries all 25 and the real
total is **62**, not 58. Flagged rather than silently reconciled.

**Evidence provenance.** Per `_harmful-revert.md`, the development machine used during these passes is
heavily modified by its owner and is not a stock image. No current registry value, service start type
or scheduled-task enabled state on that machine establishes a Windows default. What remains admissible,
because it describes the product rather than one machine: the ADMX and ADML definitions under
`C:\Windows\PolicyDefinitions`, the shipped default security template `C:\Windows\inf\defltbase.inf`,
the presence of a UTF-16 symbol inside a shipped binary (proof a value name exists and is read, never
proof of its default), Microsoft Learn, KBs, and CIS or DISA STIG controls.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `disable_remote_registry` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Stock start type is Disabled, not Manual |
| `disable_remote_desktop` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | **Single option, no revert path**; stock is already 1 |
| `remove_smbv1` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Revert omits the feature effect; Win10 Home/Pro still ship SMB1 |
| `disable_wdigest` | VERIFIED | low | Microsoft-documented | none |
| `enable_lsa_protection` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Value 1 sets a UEFI lock a registry revert cannot undo; use 2 |
| `enforce_ntlmv2` | VERIFIED | medium | Microsoft-documented | none (reboot flag stricter than Microsoft's "no restart") |
| `require_smb_signing` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | **Harmful revert:** writes 0/0, below the 24H2 Pro/Ent/Edu default |
| `rdp_security_hardening` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Revert hardcodes `UserAuthentication` = 0, downgrading NLA |
| `disable_admin_shares` | VERIFIED | medium | Microsoft-documented | none (Server service restart, not a full reboot) |
| `disable_lmhash_storage` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | **Single option, no revert path**; default is Enabled (1); no reboot |
| `restrict_anonymous_enum` | VERIFIED | medium | Microsoft-documented | none |
| `reduce_credential_caching` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Info text claims a reboot; Microsoft documents none |
| `block_vulnerable_drivers` | VERIFIED-WITH-CORRECTION | low | Community-corroborated | Registry value undocumented by Microsoft; on by default since 22H2 |
| `disable_wpbt` | VERIFIED-WITH-CORRECTION | medium | Community-corroborated | Eclypsium attribution is false and must go |
| `require_ctrlaltdel` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Missing the `Winlogon\DisableCAD` companion that actually governs |
| `disable_autorun` | VERIFIED | low | Microsoft-documented | none (see `autoplay_non_volume` for the missing third value) |
| `powershell_scriptblock_logging` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Covers Windows PowerShell 5.1 only, not PowerShell 7 |
| `uac_max` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Writes `EnableLUA`, which needs a reboot; flag unset |
| `filter_admin_token` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | No reboot required; must be paired with a prompting `ConsentPromptBehaviorAdmin` |
| `hide_last_user` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Stock is a seeded REG_DWORD 0; `absent` revert deletes a shipped value |
| `printnightmare_point_and_print` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Absent already means 1 since Aug 2021; both labels misleading |
| `disable_remote_assistance` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | **Harmful revert:** "Enabled (Stock Default)" turns the feature ON |
| `disable_windows_script_host` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Missing WOW6432Node twin; 32-bit `wscript.exe` still runs |
| `enable_controlled_folder_access` | VERIFIED | medium | Microsoft-documented | none |
| `enable_network_protection` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Pro and Enterprise only; info omits behavior monitoring and active mode |
| `enable_pua_protection` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Current default is Audit (2), not off |
| `asr_block_lsass_theft` | **INCORRECT** | medium | Microsoft-documented | **GUID ends `e4b0`; Microsoft's is `e4b2`. Inert as shipped.** |
| `asr_block_office_script_vectors` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Missing the ADMX parent `ExploitGuard_ASR_Rules` = 1 |
| `enforce_smartscreen` | VERIFIED | low | Microsoft-documented | none |
| `disable_tls_legacy` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Missing the `DisabledByDefault` = 1 companion |
| `dotnet_strong_crypto` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Missing `SystemDefaultTlsVersions` = 1 companion |
| `disable_smb_guest` | VERIFIED | low | Microsoft-documented | none |
| `remove_powershell_v2` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Revert omits the feature effect; HKCU marker for a machine-wide change |
| `firewall_all_profiles` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | `StandardProfile` is the wrong subkey; policy path uses `PrivateProfile` |
| `audit_logon_events` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Undo disables failure auditing, below the shipped default |
| `lock_on_inactivity` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | No-op without a selected screen saver; hardcoded revert values |
| `enable_credential_guard` | **INCORRECT** | medium | Microsoft-documented | `LsaCfgFlags` written under `Control\DeviceGuard`, a key nothing reads |
| `defender_cloud_protection` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Tamper Protection can make three of five writes inert; option 2 self-contradictory |
| `smb_client_block_ntlm` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | It is a plain registry value, not a PowerShell action |
| `enhanced_phishing_protection` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Work and school passwords only; `ServiceEnabled` = 1 is audit mode |
| `asr_standard_protection_rules` | VERIFIED | low | Microsoft-documented | none (drop the unsourced Home claim) |
| `disable_winrm_remoting` | VERIFIED | medium | Microsoft-documented | none (restore the start type from the snapshot, never hardcode Manual) |
| `powershell_module_transcript_logging` | VERIFIED | low | Microsoft-documented | none |
| `restrict_remote_sam` | VERIFIED | low | Microsoft-documented | none |
| `block_always_install_elevated` | VERIFIED | low | Microsoft-documented | none (restate why both hives are written) |
| `kernel_dma_protection` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Stock default is 1, not 2; reboot is mandatory |
| `asr_extended_rules` | VERIFIED | medium | Microsoft-documented | none (three dependency notes missing from the copy) |
| `ntlm_outgoing_restriction` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Session-security bit semantics inverted; revert must be `absent` |
| `rdp_session_hardening` | VERIFIED | low | Microsoft-documented | none (`MinEncryptionLevel` stops at 3; inert under TLS) |
| `disable_secondary_logon` | VERIFIED | medium | Community-corroborated | none (restore the start type from the snapshot) |
| `early_launch_antimalware_policy` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | "Good only" is `8` in the shipped ADMX, not `0` |
| `credssp_encryption_oracle` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Stock default is Mitigated (1), not Vulnerable (2) |
| `device_encryption_posture` | VERIFIED | medium | Microsoft-documented | none (the `>=26100` gate is too narrow) |
| `hide_admin_accounts_on_elevation` | VERIFIED | low | Microsoft-documented | none (`DisablePasswordReveal` is class Both; write HKCU too) |
| `event_log_retention` | VERIFIED | low | Microsoft-documented | none (Setup channel omitted; the 20480 KB default is unsourced) |
| `audit_process_creation_cmdline` | VERIFIED | low | Microsoft-documented | none (the `auditpol` half must revert from a captured state) |
| `spooler_remote_rpc_off` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | `RegisterSpoolerRemoteRpcEndPoint` is under the policy key, not `Control\Print` |
| `remote_uac_token_filter` | VERIFIED | medium | Microsoft-documented | none |
| `enable_sehop` | VERIFIED | low | Community-corroborated | none (no behavioural delta on x64) |
| `disable_pku2u_online_id` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | Effective default on a client is **enabled**, so this really turns something off |
| `no_index_encrypted_files` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | It **is** in the shipped ADMX (`Search.admx` is UTF-16LE) |
| `autoplay_non_volume` | VERIFIED | low | Microsoft-documented | none (class Both; write HKCU too) |

**Tally over 62 entries:** VERIFIED 23, VERIFIED-WITH-CORRECTION 37, UNVERIFIED 0, DISPUTED 0,
INCORRECT 2.

Split by cohort: the 37 shipped tweaks are VERIFIED 8, VERIFIED-WITH-CORRECTION 27, INCORRECT 2. The
25 additions are VERIFIED 15, VERIFIED-WITH-CORRECTION 10.

## Corrections required

### Blocking defects (the tweak does nothing, or the revert makes the machine worse)

1. **`asr_block_lsass_theft`: the GUID is wrong and the tweak is inert.** The YAML writes value name
   `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b0`. Microsoft's ASR rules overview and ASR rules reference both
   give `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2` for "Block credential stealing from the Windows local
   security authority subsystem". The final character is `2`, not `0`. Defender ignores an unrecognised
   GUID silently, so the tweak reports success and does nothing. **Corrected value name:
   `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2`.** Because of this the corpus currently ships zero working
   standard-protection ASR rules, not one.
2. **`enable_credential_guard`: wrong registry key.** The YAML writes `LsaCfgFlags` under
   `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard`. Microsoft documents `LsaCfgFlags` under
   `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` (registry route) and under
   `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` (policy route). `Control\DeviceGuard` holds
   `EnableVirtualizationBasedSecurity` and `RequirePlatformSecurityFeatures`, not `LsaCfgFlags`.
3. **`enable_credential_guard`: revert must write 0, not delete.** Microsoft: "Deleting these registry
   settings may not disable Credential Guard. They must be set to a value of 0."
4. **`enable_credential_guard`: value 1 applies a UEFI lock.** 1 is "Enabled with UEFI lock" and can
   only be removed with a `bcdedit` plus `SecConfig.efi` procedure requiring physical presence. A tweak
   declared `reversible: true` must use 2.
5. **`enable_credential_guard`: missing prerequisite and wrong gate.** Microsoft's registry route also
   sets `EnableVirtualizationBasedSecurity` = 1 (and optionally `RequirePlatformSecurityFeatures`)
   under `Control\DeviceGuard`. The `windows: { products: [11] }` gate excludes Windows 10 Enterprise
   and Education, where the feature exists, and fails to exclude Home, where it does not. Credential
   Guard is also on by default on eligible Windows 11 22H2 and later, so "Windows default = absent" is
   untrue on many target machines. See `_cross-category.md` finding 1: fixing the key **activates** a
   live conflict with `performance:disable_vbs_hvci`, so the dependency must be handled in the same
   change.
6. **`require_smb_signing`: the revert weakens Windows 11 24H2.** Per `_harmful-revert.md`, the "Windows
   default (Stock Default)" option writes `RequireSecuritySignature` = 0 to both the LanmanWorkstation
   and LanmanServer keys. Microsoft documents that Windows 11 24H2 Enterprise, Pro and Education
   **require** both outbound and inbound SMB signing by default, so the revert drops the machine below
   its own OS default and leaves it less secure than never applying the tweak. **Change both revert
   values to `absent`** so the version- and edition-specific OS default applies.
7. **`disable_remote_assistance`: the revert switches on something Windows ships off.** The YAML labels
   `fAllowToGetHelp` = 1 "Enabled (Stock Default)". Microsoft's unattend reference states of **false**:
   "Specifies that the user cannot request assistance from a friend or a support professional. This is
   the default value." Selecting the "stock" option opens an inbound remote-control channel that was
   closed. Change the revert to `absent` and relabel it "Windows default (Stock Default)".
8. **`hide_last_user`: the `absent` revert deletes a shipped value.**
   `Policies\System\DontDisplayLastUserName` is seeded by Windows setup as REG_DWORD 0, confirmed on
   26100.4061 by direct read and by `secedit /export` emitting `...\DontDisplayLastUserName=4,0`. The
   Stock Default option must write `0`, not `absent`.
9. **`firewall_all_profiles`: wrong profile subkey.** Under
   `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall` the three profile subkeys are `DomainProfile`,
   **`PrivateProfile`** and `PublicProfile`. `StandardProfile` is the subkey used only under the
   non-policy path `...\SharedAccess\Parameters\FirewallPolicy`. As authored the Private profile is
   never configured and two orphan values are written.
10. **`require_ctrlaltdel`: missing companion value, probably a no-op as authored.** On 26100.4061 the
    `Policies\System\DisableCAD` value is absent (already effectively 0) while
    `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\DisableCAD` is present as REG_DWORD 1,
    and that is the store `authui.dll` reads to skip the "Press Ctrl+Alt+Delete" screen. Add a second
    effect writing the Winlogon value: `0` for "Required", `1` for Stock Default.
11. **`disable_windows_script_host`: missing WOW6432Node companion leaves a trivial bypass.** The
    native and WOW64 views of `...\Windows Script Host\Settings` are separate physical keys (proven on
    26100.4061 by differing key last-write timestamps). 32-bit `C:\Windows\SysWOW64\wscript.exe` reads
    the WOW64 copy and still runs. Add
    `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows Script Host\Settings` -> `Enabled` (REG_DWORD).
12. **`spooler_remote_rpc_off` (new): wrong key in the proposal.** `RegisterSpoolerRemoteRpcEndPoint`
    lives at `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers` per the shipped `Printing2.admx`,
    not `HKLM\SYSTEM\CurrentControlSet\Control\Print`. Writing the wrong key would satisfy the probe
    while changing nothing, a "did-it-work" contract violation.

### Structural defects

13. **`disable_remote_desktop` has only one option and therefore no revert path.** The tweak offers
    "Disabled" (`deny_ts: 1`) and nothing else, so ADR-0003's "System Default is a selectable state" is
    unexpressible and the user has no in-app way back. Add a second option writing `deny_ts: 0`
    labelled "Allowed". Do **not** label it "Stock Default": Microsoft states "By default, remote
    connections aren't allowed", so the shipped value is 1, the same value the tweak applies.
14. **`disable_lmhash_storage` has only one option and therefore no revert path.** It offers "Enabled"
    (`no_lm_hash: 1`) and nothing else. Microsoft's default-values table gives "Client Computer
    Effective Default Settings: Enabled", so the correct stock value is also `1`. Add the second option
    for state-model completeness and be honest that both states are the same value.
15. **`remove_smbv1`: revert never re-installs the feature.** "Present (Stock Default)" lists only
    `smb1_server: absent` and omits `smb1_feature`, so `Enable-WindowsOptionalFeature` is never invoked.
16. **`remove_powershell_v2`: revert never re-installs the feature.** Same structural defect: "Present
    (Stock Default)" sets only `state: 0`.
17. **`remove_powershell_v2` and `audit_logon_events`: HKCU marker for a machine-wide change.** Both
    store applied state in `HKCU\Software\MagicXToolbox\State`. The change is machine-wide and applied
    through an elevated broker, so the marker can land in a different hive from the one the UI reads.
    Move it to HKLM.
18. **`audit_logon_events`: the undo disables failure auditing.** `undo` runs
    `/success:enable /failure:disable`. Windows 10 and 11 clients ship with the Logon subcategory
    auditing Success **and** Failure, so the revert leaves the machine auditing less than stock.
    Capture and restore the pre-apply inclusion setting.
19. **`lock_on_inactivity`: does not lock without a screen saver.** `ScreenSaveActive` = "1" plus
    `ScreenSaverIsSecure` = "1" only take effect when `SCRNSAVE.EXE` is set under the same key. On a
    stock Windows 11 the screen saver is "(None)", so the tweak is a no-op. Either set `SCRNSAVE.EXE`
    as well, or use the machine-wide `InactivityTimeoutSecs` under
    `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`.
20. **`lock_on_inactivity`: hardcoded revert values.** The Stock Default option writes
    `ScreenSaveActive` = "1", `ScreenSaverIsSecure` = "0", `ScreenSaveTimeOut` = "600". None is a
    documented Windows default. Restore captured values or use `absent`.
21. **`rdp_security_hardening`: revert hardcodes NLA off.** The Stock Default option writes
    `UserAuthentication` = 0, but Windows 10 and 11 ship with NLA required, so reverting disables NLA
    rather than restoring it. Restore captured values.
22. **`disable_remote_registry`: wrong stock default.** Independent service-default references for both
    Windows 10 22H2 and Windows 11 record the shipped `RemoteRegistry` start type as **Disabled**, not
    Manual. The revert option "Manual (Stock Default)" therefore enables a service that was off, and
    the info text repeats the error.

### Missing companion values

23. **`disable_tls_legacy`: missing `DisabledByDefault`.** Every Microsoft example and the CIS and STIG
    baselines write `DisabledByDefault` = 1 alongside `Enabled` = 0 in each Client and Server subkey.
    Add the four values.
24. **`dotnet_strong_crypto`: missing `SystemDefaultTlsVersions`.** Microsoft's reference `.reg` sets
    `SystemDefaultTlsVersions` = 1 alongside `SchUseStrongCrypto` in the same keys and also covers
    `v2.0.50727`.
25. **`asr_block_office_script_vectors`: missing the ADMX parent enabling value.**
    `WindowsDefender.admx` defines policy `ExploitGuard_ASR_Rules` at
    `Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR` with
    `valueName="ExploitGuard_ASR_Rules"`, whose `<list>` element is the `ASR\Rules` subkey the YAML
    writes. Without the parent value the policy reads Not Configured in `gpedit.msc` and a policy
    refresh can strip the orphaned rule values. Add `ExploitGuard_ASR_Rules` = 1 (REG_DWORD), `absent`
    on revert. The same applies to `asr_block_lsass_theft`, `asr_standard_protection_rules` and
    `asr_extended_rules`.
26. **`powershell_scriptblock_logging`: PowerShell 7 not covered.** PowerShell 7 (`pwsh.exe`) reads
    `HKLM\SOFTWARE\Policies\Microsoft\PowerShellCore\ScriptBlockLogging`. An attacker with PowerShell 7
    installed is unlogged.
27. **`filter_admin_token`: incomplete and the reboot flag is wrong.** Microsoft states "Restart
    requirement: None", and separately that Admin Approval Mode for the built-in Administrator requires
    `ConsentPromptBehaviorAdmin` = 2 ("Prompt for consent on the secure desktop"). Setting
    `FilterAdministratorToken` = 1 without that can leave the built-in Administrator unable to elevate.
28. **`disable_autorun`: the third AutoPlay value is missing.** `NoAutoplayfornonVolume` under
    `Software\Policies\Microsoft\Windows\Explorer` covers MTP devices (phones, cameras) that never get
    a drive letter. Tracked as the separate addition `autoplay_non_volume`; the cleanest fix is to fold
    it into `disable_autorun` as a third effect in both hives.

### Reboot-flag and applicability corrections

29. **`uac_max`: `requires_reboot` is missing and should be true.** The tweak writes `EnableLUA`, which
    Microsoft documents as requiring a computer restart. Also note that on 24H2 and newer, machines
    with Administrator Protection enabled (`TypeOfAdminApprovalMode` = 2) are governed by
    `ConsentPromptBehaviorEnhancedAdmin`, not `ConsentPromptBehaviorAdmin`.
30. **`disable_lmhash_storage`: `requires_reboot` is wrong.** Microsoft states "Restart requirement:
    None". What gates the effect is the next password change.
31. **`reduce_credential_caching`: the info block claims a reboot.** Microsoft states no restart is
    required. The YAML's own omission of `requires_reboot` contradicts its prose. Also rename the
    revert option "Windows default (10)" to carry the "(Stock Default)" marker the UI's OFF-position
    heuristic relies on.
32. **`enforce_ntlmv2`, `restrict_anonymous_enum`, `disable_admin_shares`: `requires_reboot`
    overstated.** Microsoft states "Restart requirement: None" for the first two;
    `disable_admin_shares` needs `net stop server` / `net start server`. Harmless but inaccurate.
33. **`enable_network_protection`: wrong applicability.** Microsoft supports network protection on
    Windows 10 and 11 **Pro or Enterprise** only. The prerequisite list must also name **behavior
    monitoring** and Defender being in **active** (not passive) mode.
34. **`enable_pua_protection`: the default is not off.** With security intelligence 1.329.495.0 or
    later, PUA protection defaults to **Audit mode (2)** on Windows 10 and later, and Block (1) on
    devices onboarded to Defender for Endpoint.
35. **`printnightmare_point_and_print`: both option labels are wrong.** Per KB5005010, with the
    August 10, 2021 and later updates the default when the value is absent is **1 (restricted)**. So
    "Admins only" is a no-op on any patched machine, and deleting the value does not restore
    unrestricted installation. The info text's Microsoft quote comes from the CVE-2021-34527 MSRC
    advisory FAQ, not KB5005010.
36. **`block_vulnerable_drivers`: registry route is undocumented.** Microsoft documents the feature and
    states it is on by default since the Windows 11 2022 Update but does not document
    `VulnerableDriverBlocklistEnable`. Say so, and note it cannot override HVCI, Smart App Control or
    S mode enforcement in either direction.
37. **`disable_wpbt`: wrong attribution, correct mechanism.** Eclypsium's WPBT research documents the
    attack surface and names WDAC as the mitigation; it never mentions `DisableWpbtExecution`. Replace
    the attribution with the accurate one (present in `smss.exe` on 26100.4061, corroborated by four
    independent tier C projects) and label it community-corroborated.
38. **`enable_lsa_protection`: unrecoverable UEFI variable.** `RunAsPPL` = 1 stores the setting in a
    UEFI variable on Secure Boot machines; deleting the registry value then has no effect. Use
    `run_as_ppl: 2` ("without UEFI lock", enforced on Windows 11 22H2 and later) for a reversible
    tweak, or set `reversible: false`.

### Corrections carried in with the additions

39. **`smb_client_block_ntlm`: it is a plain registry value, not a PowerShell action.** The gap
    proposal claimed `LanmanWorkstation.admx` on 26100 does not contain the policy. It does, verbatim:
    policy `Pol_BlockNTLM`, class Machine, key `Software\Policies\Microsoft\Windows\LanmanWorkstation`,
    `valueName="BlockNTLM"`, enabled 1 / disabled 0. Implement as
    `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation` -> `BlockNTLM` (REG_DWORD) = 1, revert
    by deleting. No `Set-SmbClientConfiguration` action is needed.
40. **`kernel_dma_protection`: the stock default is 1, not 2.** The shipped `DmaGuard.adml` labels
    "Only while logged in (default)", and that string maps to registry value **1**. Value 2 is "Allow
    all". A revert writing 2 would be less protective than stock. Also, Policy CSP DmaGuard states
    "This policy requires a system reboot to take effect", so `requires_reboot: true` is mandatory, and
    the policy is Pro and above (not Home) from Windows 10 1809.
41. **`enhanced_phishing_protection`: work and school passwords only, and `ServiceEnabled` is audit
    mode.** Microsoft's scope is "Microsoft school or work passwords". On a consumer machine with a
    local or personal Microsoft account the feature is largely inert. The shipped ADML states enabling
    `ServiceEnabled` turns the feature on **in audit mode** and prevents users turning it off; the
    three `Notify*` values produce the warnings. Three of the five values already default hardened;
    the genuine additions are `NotifyPasswordReuse` and `NotifyUnsafeApp`.
42. **`defender_cloud_protection`: Tamper Protection can make part of it inert, and option 2 is
    self-contradictory.** `SpynetReporting` and `DisableBlockAtFirstSeen` sit on the tamper-protected
    surface, so the write lands while the effective setting does not move; `MpCloudBlockLevel` and
    `MpBafsExtendedTimeout` are not tamper-protected and deliver the real change. The probe must read
    `Get-MpPreference` rather than the registry, or carry `skip_validation`. The proposed "no sample
    submission" option sets `SubmitSamplesConsent` = 2 (Never Send) while claiming Block at First
    Sight, which Microsoft states NeverSend disables. Also drop the ASR-dependency justification: none
    of the corpus's ASR rules is cloud-gated; `enable_network_protection` is.
43. **`ntlm_outgoing_restriction`: the session-security bits are transposed, and the revert must be
    absent.** `0x20000000` is "Require 128-bit encryption (Default)"; `0x20080000` adds the **NTLMv2
    session-security** bit. The hardened value stays `0x20080000` but the reasoning was inverted.
    Microsoft documents 536870912 as an effective default, not a value present in the registry, so the
    revert must write `absent` for `NtlmMinClientSec` and `NtlmMinServerSec`.
44. **`early_launch_antimalware_policy`: "Good only" is `8`, not `0`.** The shipped `EarlyLaunchAM.admx`
    enum members are 8 (Good only), 1 (Good and unknown), 3 (Good, unknown and bad but critical) and 7
    (All). `0` is not a member. Offer 3 and 1 only; never 8 (can refuse to boot on unusual hardware)
    and never 7 (no filtering).
45. **`credssp_encryption_oracle`: the stock default is Mitigated (1), not Vulnerable (2).** Microsoft's
    CVE-2018-0886 KB records "May 8, 2018. An update to change the default setting from Vulnerable to
    Mitigated." Any 24H2 machine has been at Mitigated for years, so the hardened value 0 changes only
    the server half of the behaviour and the risk is smaller than the proposal claimed.
46. **`disable_pku2u_online_id`: the effective default is enabled, not disabled.** Microsoft states the
    policy "is enabled by default in Windows 10, Version 1607, and later" for a standalone client. This
    tweak really does turn something off, which raises both its value and its risk; PKU2U backs
    peer-to-peer authentication between non-domain machines.
47. **`no_index_encrypted_files`: it is in the shipped ADMX.** `Search.admx` ships as **UTF-16LE** while
    216 of the 218 shipped ADMX files are UTF-8, so a naive grep misses it. Policy
    `AllowIndexingEncryptedStoresOrItems`, class Machine, key
    `SOFTWARE\Policies\Microsoft\Windows\Windows Search`, enabled 1 / disabled 0. The "not in the ADMX"
    claim in the gap document lowered confidence wrongly.
48. **`hide_admin_accounts_on_elevation` and `autoplay_non_volume`: class Both, so write HKCU too.**
    `DisablePasswordReveal` and `NoAutoplayfornonVolume` are both declared `class="Both"`. Write and
    revert the HKCU twin alongside HKLM.
49. **`audit_process_creation_cmdline`: the `auditpol` half must revert from a captured state.** The
    registry value alone logs nothing; event 4688 only appears when the Detailed Tracking > Process
    Creation subcategory is enabled. Deleting only the DWORD on revert leaves process-creation auditing
    switched on, a silent state leak.
50. **`disable_winrm_remoting` and `disable_secondary_logon`: do not hardcode `Manual` as the stock
    start type.** No tier A source pins the shipped start type of `WinRM` or `seclogon` on a Windows 11
    client. Restore the start type from the snapshot. For `WinRM` this matters concretely: any machine
    where `winrm quickconfig` has run sits at delayed auto start, and a hardcoded Manual revert would
    silently downgrade a working configuration.

## New in this revision

Twenty-five tweaks are added to the category, all carried in from the two gap-verification passes with
their mechanisms independently attacked against the shipped 26100 ADMX set, shipped binaries and
Microsoft Learn. Nine came from `_verify-gaps-b-high.md` and sixteen from `_verify-gaps-b-medlow.md`.

| Tweak | Source pass | Verdict there | Fills |
|---|---|---|---|
| `defender_cloud_protection` | b-high 1 | CORRECTED | Cloud protection and MAPS, the prerequisite `enable_network_protection` needs |
| `smb_client_block_ntlm` | b-high 2 | CORRECTED | 24H2's SMB client NTLM block, the strongest consumer NTLM control |
| `enhanced_phishing_protection` | b-high 3 | CORRECTED | The SmartScreen password-protection component `enforce_smartscreen` never touches |
| `asr_standard_protection_rules` | b-high 4 | CONFIRMED | The other two of Microsoft's three "standard protection" ASR rules |
| `disable_winrm_remoting` | b-high 5 | CONFIRMED | WinRM, the last untouched remote-access surface |
| `powershell_module_transcript_logging` | b-high 6 | CONFIRMED | Module logging and transcription, beyond script-block logging |
| `restrict_remote_sam` | b-high 7 | CONFIRMED | Authenticated remote SAM enumeration (BloodHound-class harvesting) |
| `block_always_install_elevated` | b-high 8 | CONFIRMED | Pins off a classic local privilege-escalation primitive |
| `kernel_dma_protection` | b-high 12 | CORRECTED | Drive-by DMA over Thunderbolt and external PCIe |
| `asr_extended_rules` | b-medlow 13 | CONFIRMED | Six further ASR rules beyond the corpus's Office and script set |
| `ntlm_outgoing_restriction` | b-medlow 14 | CORRECTED | OS-level outgoing NTLM audit, deny and session-security floor |
| `rdp_session_hardening` | b-medlow 15 | CONFIRMED | Five STIG RDP settings beyond NLA and TLS |
| `disable_secondary_logon` | b-medlow 18 | CONFIRMED | `seclogon`, a routine privilege-escalation step |
| `early_launch_antimalware_policy` | b-medlow 19 | CORRECTED | Pins the ELAM boot-driver policy so malware cannot loosen it |
| `credssp_encryption_oracle` | b-medlow 20 | CORRECTED | CVE-2018-0886 remediation, server half |
| `device_encryption_posture` | b-medlow 21 | CONFIRMED | Control over 24H2 automatic device encryption and key custody |
| `hide_admin_accounts_on_elevation` | b-medlow 22 | CONFIRMED | Stops UAC enumerating admin accounts; removes the password-reveal eye |
| `event_log_retention` | b-medlow 23 | CONFIRMED | Sizes the logs the corpus's auditing tweaks fill |
| `audit_process_creation_cmdline` | b-medlow 24 | CONFIRMED | Command lines in event 4688 |
| `spooler_remote_rpc_off` | b-medlow 26 | CORRECTED | The middle option between disabling the spooler and only restricting drivers |
| `remote_uac_token_filter` | b-medlow 29 | CONFIRMED | Pins `LocalAccountTokenFilterPolicy` = 0 against pass-the-hash |
| `enable_sehop` | b-medlow 33 | CONFIRMED | Pins SEHOP so it cannot be turned off |
| `disable_pku2u_online_id` | b-medlow 35 | CORRECTED | Closes peer-to-peer online-identity authentication |
| `no_index_encrypted_files` | b-medlow 34 | CORRECTED | Keeps EFS content out of the unencrypted search index |
| `autoplay_non_volume` | b-medlow 28 | CONFIRMED | The MTP half of AutoPlay that `disable_autorun` misses |

Four gap proposals in the same passes were **rejected** and are deliberately not here:
`task_compatibility_appraiser` and `task_ceip` (duplicates of `privacy.yaml`),
`mss_network_stack_hardening` (no established stock defaults, one member a no-op) and
`ntfs_disable_8dot3` (the load-bearing Microsoft performance claim is not on the cited page).

## Merge candidates

**1. Remote Desktop: `disable_remote_desktop` + `rdp_security_hardening` + `rdp_session_hardening`.**
One "Remote Desktop" tweak with "Off (recommended)", "On, hardened" and "Windows default". This fixes
`disable_remote_desktop`'s missing revert and removes the trap where hardening RDP is offered
independently of whether RDP is on. Granularity lost: hardening RDP-Tcp while the listener is denied, a
state with no practical value.

**2. UAC: `uac_max` + `filter_admin_token` + `remote_uac_token_filter`.** Microsoft explicitly couples
the first two. The third writes a third value in the same key and users will otherwise assume it
duplicates `filter_admin_token`. If they stay separate, each info block must say what the others do.

**3. LSASS credential protection: `disable_wdigest` + `asr_block_lsass_theft` + `enable_lsa_protection`
+ `enable_credential_guard`.** Escalating levels in one tweak. Microsoft states the ASR rule "isn't
required" and "doesn't provide extra protection" when LSA protection is on. Four overlapping controls
invite users to enable all of them and inherit all four sets of compatibility problems.

**4. Defender ASR rules: `asr_block_lsass_theft` + `asr_block_office_script_vectors` +
`asr_standard_protection_rules` + `asr_extended_rules`.** Twelve GUIDs, one registry key, one value
type, one set of prerequisites, and Microsoft's own deployment guidance is per-mode, not per-rule. One
tweak with "Block", "Audit" and "Not configured" would also let the missing `ExploitGuard_ASR_Rules`
parent value be written once instead of four times.

**5. SMB client hardening: `require_smb_signing` + `disable_smb_guest` + `smb_client_block_ntlm`.**
Microsoft couples the first two directly ("Requiring SMB signing also disables guest access to
shares"). All three produce the same NAS breakage and their warnings duplicate.

**6. Legacy authentication lockdown: `enforce_ntlmv2` + `disable_lmhash_storage` +
`ntlm_outgoing_restriction`.** `NoLMHash` has nothing to offer standalone; as a companion to the
`LmCompatibilityLevel` change it completes the picture, and the outgoing-NTLM values are the same
subsystem.

**7. AutoPlay: fold `autoplay_non_volume` into `disable_autorun`.** Three values, one concept, one
tweak. This is the gap pass's own recommendation.

**8. Legacy TLS: `disable_tls_legacy` + `dotnet_strong_crypto`.** Granularity lost here is real:
`dotnet_strong_crypto` is almost always safe while `disable_tls_legacy` can cut off legacy internal
servers. Keep separate unless the merged tweak exposes both as options.

## Tweak entries

### `disable_remote_registry` Remote Registry service

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Service effect on `RemoteRegistry` (display name "Remote Registry"). Options:
"Disabled" (start type `disabled`), "Manual (Stock Default)" (start type `manual`). No reboot flag.

The service hosts the RPC endpoint that lets `RegConnectRegistry` reach this machine's registry from
the network. Start type Disabled prevents SCM from starting it, including by trigger start.

CORRECTED stock default: **`disabled`**. Independent service-default references for both Windows 10
22H2 and Windows 11 record the shipped start type as Disabled. The current YAML claims `manual`.

**Corrections needed:** The revert option label "Manual (Stock Default)" and the info sentence "The
service is already Manual and stopped by default" are both wrong; the shipped start type is Disabled,
so reverting currently makes the machine *more* permissive than it was. Both supporting sources are
tier C; `sc.exe qc RemoteRegistry` on a clean install would settle it definitively. Safest fix:
restore the start type from the snapshot rather than hardcoding either literal.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops anyone from reading or editing this PC's registry over the network.**

      ## What it does
      Sets the `RemoteRegistry` service start type to Disabled. That service hosts the RPC endpoint
      remote administrators use to reach this machine's registry. Disabled means the Service Control
      Manager will not start it, not even by trigger.

      ## Benefits
      - **Closes a remote channel**: no remote registry session can be established, credentials or not
      - **Pins the safe state**: a compromised management agent cannot quietly flip it back on
      - **Blunts recon**: remote registry reads are a standard reconnaissance and lateral-movement step

      ## Drawbacks
      - **Breaks remote management**: some RMM agents, inventory tools and backup pre-flight checks
        read this machine's registry across the network
      - **Usually no visible change**: the service ships Disabled and stopped, so most PCs see nothing
      - **Not a boundary**: an administrator who reaches the machine can set it back

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: restores the previous start type from the snapshot
      - The shipped start type is Disabled on both Windows 10 and Windows 11, so a revert that writes
        Manual would leave the service more available than it was

      ## Recommendation
      Apply it on any standalone or home machine. Skip it only if you knowingly run software that
      manages this PC's registry from another computer.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Remote Registry service defaults in Windows 11](https://revertservice.com/11/remoteregistry/)
      - [Remote Registry, Windows 11 service reference](https://batcmd.com/windows/11/services/remoteregistry/)
```

**Sources:**
1. Remote Registry (RemoteRegistry) Service Defaults in Windows 11, https://revertservice.com/11/remoteregistry/ (tier C)
2. Remote Registry, Windows 11 Service, https://batcmd.com/windows/11/services/remoteregistry/ (tier C)

### `disable_remote_desktop` Remote Desktop (RDP)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server` -> `fDenyTSConnections`
(REG_DWORD). **Single option: "Disabled" = 1. There is no second option, so the tweak has no revert
path from the UI.** No reboot flag, correctly.

1 denies inbound Remote Desktop; 0 allows it. This is the value the System Properties "Remote" tab and
the Settings > System > Remote Desktop toggle write. A Group Policy counterpart lives under
`HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` (policy `TS_DISABLE_CONNECTIONS`) and
overrides it when configured.

**Corrections needed:** Add a second option writing `deny_ts: 0`, labelled "Allowed". Do **not** label
it "Stock Default": Microsoft states "By default, remote connections aren't allowed", so the shipped
value is 1, the same value the tweak applies. As shipped this is a one-way door from the UI's point of
view and violates ADR-0003's requirement that System Default be a selectable state (see
`_cross-category.md` section 2.1, which lists this and `disable_lmhash_storage` as the corpus's only
single-option tweaks).

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off inbound Remote Desktop so there is nothing on port 3389 to attack.**

      ## What it does
      Writes `fDenyTSConnections` = 1 under `HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server`,
      the same value the Settings > System > Remote Desktop toggle uses. The Terminal Services
      listener then refuses every incoming session.

      ## Benefits
      - **Removes a top target**: RDP is among the most heavily attacked Windows listeners
      - **No listener, no pre-auth bugs**: BlueKeep-class flaws need a reachable listener
      - **Outbound still works**: you can still connect from this PC to others

      ## Drawbacks
      - **No inbound RDP**: remote-support tools and VDI agents that use the listener stop working
      - **Already the default**: Microsoft ships remote connections denied, so on a clean install this
        confirms the current state rather than changing it
      - **Group Policy wins**: if a policy sets the counterpart value, the policy overrides this
      - **Inert on Home**: Windows Home cannot host RDP at all

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, Pro / Enterprise / Education
      - **Takes effect**: immediately, no reboot
      - **Reverting**: restores the previous value from the snapshot
      - If you need RDP, harden it with the NLA and TLS tweak instead of leaving it wide open

      ## Recommendation
      Apply it unless you deliberately connect into this PC over Remote Desktop. If you do rely on
      RDP, leave this alone and apply the RDP hardening tweaks instead.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Enable Remote Desktop on your PC](https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access)
      - [RemoteDesktopServices Policy CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-remotedesktopservices)
```

**Sources:**
1. Enable Remote Desktop on your PC, https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access (tier A)
2. RemoteDesktopServices Policy CSP, AllowUsersToConnectRemotely, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-remotedesktopservices (tier A)
3. `docs/superpowers/research/validation/_cross-category.md` section 2.1 (single-option tweaks)

### `remove_smbv1` Remove SMBv1 protocol

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Two effects.

- Registry: `HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters` -> `SMB1` (REG_DWORD).
- Action (PowerShell): apply `Disable-WindowsOptionalFeature -Online -FeatureName SMB1Protocol
  -NoRestart -ErrorAction Stop`; undo `Enable-WindowsOptionalFeature -Online -FeatureName SMB1Protocol
  -NoRestart -All -ErrorAction Stop`; probe
  `(Get-WindowsOptionalFeature -Online -FeatureName SMB1Protocol).State -eq 'Disabled'`.

Options: "Removed" (`smb1_server: 0`, `smb1_feature: run`), "Present (Stock Default)"
(`smb1_server: absent` **only**). `requires_reboot: true`, correct.

Microsoft documents the registry half as key `...\LanmanServer\Parameters`, entry `SMB1`, REG_DWORD,
data 0 for Disabled, and states "The default value is 1, or Enabled. In this case, no registry key is
created", so `absent` is the correct stock representation.

**Corrections needed:** (a) The "Present (Stock Default)" option omits `smb1_feature`, so the `undo`
action never runs and SMBv1 stays uninstalled after a revert. Add `smb1_feature: undo` (or the engine's
equivalent). (b) The `warning` text says SMB1 is "already removed by default on Win11 and Win10 1709+".
Microsoft excludes Windows 10 **Home and Pro** from that: "SMBv1 also isn't installed by default in
Windows 10, except Home and Pro editions."

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the legacy SMBv1 protocol behind WannaCry and EternalBlue.**

      ## What it does
      Disables the `SMB1Protocol` Windows optional feature and sets the `LanmanServer` `SMB1` value
      to 0. That removes both the SMBv1 client driver and the server component. Modern file sharing
      uses SMB 2 and SMB 3, which are unaffected.

      ## Benefits
      - **Kills a wormable protocol**: SMBv1 carried EternalBlue and WannaCry
      - **Removes the listener**: an installed but unused SMBv1 still answers on the network
      - **Matches Microsoft**: SMBv1 is deprecated and shipped off on Windows 11

      ## Drawbacks
      - **Ancient devices drop off**: SMB1-only NAS boxes, USB shares on old routers, Windows XP and
        2003 hosts become unreachable
      - **Needs a restart**: the optional-feature change is not complete until you reboot
      - **Often already done**: not installed by default on Windows 11 or on Windows 10 except the
        Home and Pro editions

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 (Home and Pro still ship it)
      - **Takes effect**: after reboot
      - **Reverting**: reinstalls the optional feature and restores the previous `SMB1` value from the
        snapshot, then needs another reboot
      - Applying on a machine where the feature is already Disabled is a safe no-op

      ## Recommendation
      Apply it on any modern setup; SMBv1 has no place on today's networks. Hold off only if you still
      reach hardware that speaks nothing newer.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Detect, enable, and disable SMBv1, SMBv2, and SMBv3](https://learn.microsoft.com/en-us/windows-server/storage/file-server/troubleshoot/detect-enable-and-disable-smbv1-v2-v3)
      - [SMBv1 is not installed by default in Windows 10 1709 and later](https://learn.microsoft.com/en-us/windows-server/storage/file-server/troubleshoot/smbv1-not-installed-by-default-in-windows)
```

**Sources:**
1. Detect, enable, and disable SMBv1, SMBv2, and SMBv3 in Windows, https://learn.microsoft.com/en-us/windows-server/storage/file-server/troubleshoot/detect-enable-and-disable-smbv1-v2-v3 (tier A)
2. SMBv1 is not installed by default in Windows 10 version 1709 and later, https://learn.microsoft.com/en-us/windows-server/storage/file-server/troubleshoot/smbv1-not-installed-by-default-in-windows (tier A)
3. Disable-WindowsOptionalFeature, https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature (tier A)

### `disable_wdigest` WDigest credential caching

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\WDigest` ->
`UseLogonCredential` (REG_DWORD). Options: "Disabled" = 0, "Windows default (Stock Default)" =
`absent`. No reboot flag, correctly: the value is read at logon.

Microsoft's KB2871997 advisory documents the exact key and value name, and states that caching is
disabled when the entry is not present, so `absent` is the right stock representation.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows from ever keeping your plaintext password in memory.**

      ## What it does
      Pins `UseLogonCredential` = 0 under the WDigest security provider key. WDigest is a legacy
      authentication package that, when it caches logon credentials, keeps a reversibly encrypted
      copy of the password inside LSASS. With the value at 0 it never does.

      ## Benefits
      - **Blocks a Mimikatz step**: credential-theft tooling writes 1 here before dumping LSASS
      - **Makes tampering loud**: an attacker must change the value again, which is detectable
      - **No practical cost**: nothing modern authenticates with WDigest digest credentials

      ## Drawbacks
      - **Already the default**: caching has been off since Windows 8.1, so nothing visibly changes
      - **Not a boundary**: an attacker with SYSTEM can flip the value back and wait for a logon
      - **Rare legacy break**: Microsoft notes you "may notice that credentials are required more
        frequently when you use WDigest"

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 8.1 and later
      - **Takes effect**: at your next sign-in, no reboot
      - **Reverting**: deletes the value, which is the shipped state
      - Reverting is safe here because absent and 0 behave identically

      ## Recommendation
      Apply it. There is no practical downside and it removes a silent default an attacker can abuse.
      Everyone should have this pinned.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft Security Advisory KB2871997: update to improve credentials protection](https://support.microsoft.com/en-us/topic/microsoft-security-advisory-update-to-improve-credentials-protection-and-management-may-13-2014-93434251-04ac-b7f3-52aa-9f951c14b649)
```

**Sources:**
1. Microsoft Security Advisory: Update to improve credentials protection and management (KB2871997), https://support.microsoft.com/en-us/topic/microsoft-security-advisory-update-to-improve-credentials-protection-and-management-may-13-2014-93434251-04ac-b7f3-52aa-9f951c14b649 (tier A)

### `enable_lsa_protection` LSA protection (RunAsPPL)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` -> `RunAsPPL` (REG_DWORD). Options:
"Enabled" = 1, "Windows default (Stock Default)" = `absent`. `requires_reboot: true`, correct.

Microsoft's value semantics: **1 = enabled with a UEFI variable (UEFI lock)**; **2 = enabled without a
UEFI variable**, and 2 is "only enforced on Windows 11 build 22H2 and later".

CORRECTED applied value: **`run_as_ppl: 2`**. Value 2 keeps the setting reversible from the registry.
The current YAML uses 1, which writes a UEFI variable on Secure Boot machines; after that, deleting the
registry value has no effect and the variable must be cleared with Microsoft's LSA Protected Process
Opt-out tool. `RunAsPPLBoot` is not documented by Microsoft and is not needed.

**Corrections needed:** Change the "Enabled" option to `run_as_ppl: 2` and gate it to Windows 11 22H2
and later where 2 is enforced. The YAML's own info text already describes value 2 while the tweak only
offers 1. If value 1 is kept instead, `reversible` must be `false` and the warning must state that a
firmware-level opt-out tool is required to undo it, because a snapshot revert cannot.

**Ready-to-paste info block:**

```yaml
    info: |
      **Runs LSASS as a protected process so nothing can read your credentials out of its memory.**

      ## What it does
      Sets `RunAsPPL` = 2 under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`. LSASS then runs as a
      protected process: even code holding SeDebugPrivilege cannot read its memory or inject into it,
      and every LSA plug-in must carry a Microsoft signature. Value 2 enables protection without
      writing a UEFI variable, so it stays reversible.

      ## Benefits
      - **Blocks LSASS dumping**: the single most common post-exploitation step stops working
      - **Signed plug-ins only**: unsigned code can no longer load into the authentication stack
      - **Reversible at 2**: unlike value 1, no firmware variable is written

      ## Drawbacks
      - **Breaks LSA add-ins**: some smart card middleware, VPN credential providers, legacy password
        filters and EDR agents load unsigned code into LSA and will silently fail
      - **Needs a restart**: the protection level is set at boot
      - **Value 1 is a trap**: on a Secure Boot machine `RunAsPPL` = 1 writes a UEFI variable that a
        registry revert cannot clear
      - **Often already on**: enabled by default on clean, enterprise-joined, HVCI-capable Windows 11
        22H2 and later installs

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (value 2 is enforced from 22H2 onward)
      - **Takes effect**: after reboot
      - **Reverting**: deletes the value from the snapshot, which restores the shipped state
      - Microsoft's recommended rollout is to run audit mode first and check CodeIntegrity events
        3065 and 3066 for plug-ins that would be blocked

      ## Recommendation
      Strongly worth it if you do not depend on unusual authentication add-ins. Test smart card, VPN
      and security agents first, and enable Secure Boot so the protection resists bypass.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Configure added LSA protection](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
      - [LocalSecurityAuthority Policy CSP, ConfigureLsaProtectedProcess](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-lsa)
```

**Sources:**
1. Configure added LSA protection, https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection (tier A)
2. LocalSecurityAuthority Policy CSP, ConfigureLsaProtectedProcess, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-lsa (tier A)

### `enforce_ntlmv2` Enforce NTLMv2 only

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` -> `LmCompatibilityLevel` (REG_DWORD).
Options: "NTLMv2 only" = 5, "Windows default (Stock Default)" = `absent`. `requires_reboot: true`.

Microsoft's level table: 0 send LM and NTLM; 1 LM and NTLM with NTLMv2 session security; 2 NTLM only;
3 NTLMv2 only; 4 NTLMv2 only, refuse LM; **5 NTLMv2 only, refuse LM and NTLM**. The default-values
table gives "Client Computer Effective Default Settings: Not defined", so `absent` is correct.

**Corrections needed:** `none`. The `requires_reboot: true` flag is stricter than Microsoft's
documented "Restart requirement: None", which is harmless but inaccurate.

**Ready-to-paste info block:**

```yaml
    info: |
      **Forces network logons onto NTLMv2 and refuses the crackable LM and NTLMv1 protocols.**

      ## What it does
      Sets `LmCompatibilityLevel` = 5 under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`. Level 5 is
      the strictest setting Microsoft defines: send NTLMv2 only, and refuse both LM and NTLMv1 on the
      accept side as well as the send side.

      ## Benefits
      - **No downgrade path**: LM and NTLMv1 responses are trivially crackable once captured
      - **Two-way enforcement**: applies to what this PC sends and what it accepts
      - **Where Windows is going**: NTLMv1 is being removed outright in Windows 11 24H2

      ## Drawbacks
      - **Old gear stops authenticating**: NAS boxes, print and scan appliances that only speak
        NTLMv1 can no longer authenticate to or from this PC
      - **Domain risk**: on a domain, verify against your controllers before applying broadly
      - **Restart flagged**: the tweak reboots even though Microsoft documents no restart requirement

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after reboot (Microsoft documents no restart requirement, so the flag is
        deliberately conservative)
      - **Reverting**: deletes the value, restoring the "Not defined" shipped state
      - Microsoft: "Client devices that don't support NTLMv2 authentication can't authenticate in the
        domain and access domain resources by using LM and NTLM"

      ## Recommendation
      Apply it on modern networks. Hold off only if you still authenticate against legacy appliances
      that cannot do NTLMv2.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Network security: LAN Manager authentication level](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-lan-manager-authentication-level)
```

**Sources:**
1. Network security: LAN Manager authentication level, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-lan-manager-authentication-level (tier A)

### `require_smb_signing` Require SMB signing

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Two REG_DWORD values.

- `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters` -> `RequireSecuritySignature`
- `HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters` -> `RequireSecuritySignature`

Options: "Required" = 1 / 1, "Windows default (Stock Default)" = **0 / 0**. `requires_reboot: true`.

CORRECTED revert: **`absent` / `absent`**. Microsoft's *Control SMB signing behavior* page states
"Windows 11, version 24H2 Enterprise, Pro, and Education require both outbound and inbound SMB signing"
by default. Writing 0 on revert therefore drops the machine **below** its own OS default, which is the
harmful-revert class described in `_harmful-revert.md` and is ranked there as the number one priority
in the whole corpus. `absent` lets the version- and edition-specific default apply.

`EnableSecuritySignature` (the "if the client agrees" variant) is ignored for SMB 2.x and later, so the
tweak is right to write only `RequireSecuritySignature`.

**Corrections needed:** Change both Stock Default values from `0` to `absent`. This is a security
regression that is documentation-confirmed and needs no clean image to act on.

**Ready-to-paste info block:**

```yaml
    info: |
      **Requires cryptographic signing on every SMB session so file sharing cannot be relayed.**

      ## What it does
      Sets `RequireSecuritySignature` = 1 on both the SMB client (`LanmanWorkstation`) and the SMB
      server (`LanmanServer`). Signing stamps each message with an HMAC-SHA-256 (SMB 2.x) or
      AES-CMAC / AES-128-GMAC (SMB 3.x) signature, so tampered or replayed traffic is rejected.

      ## Benefits
      - **Defeats SMB relay**: the most reliable lateral-movement technique on a Windows network
      - **Blocks tampering**: an in-path attacker cannot alter file-sharing traffic undetected
      - **Also kills guest fallback**: Microsoft notes requiring signing disables guest access

      ## Drawbacks
      - **Cheap NAS breaks**: servers that cannot sign fail with STATUS_INVALID_SIGNATURE (0xc000a000)
      - **Needs a restart**: the services read the registry values at start
      - **Mostly already on**: Windows 11 24H2 Pro, Enterprise and Education require both directions
        by default, so on those editions this confirms the state rather than changing it
      - **Home is different**: Windows 11 24H2 Home requires neither direction

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 (which requires neither by
        default)
      - **Takes effect**: after reboot
      - **Reverting**: deletes both values so the OS default for your Windows version and edition
        applies again; it must never write 0, which would be weaker than 24H2 ships
      - Microsoft's own guidance is to fix the non-signing server rather than turn signing off

      ## Recommendation
      Apply it on modern networks; signing is the direction Windows itself is moving. Skip it only if
      you depend on network storage that cannot sign, and replace that device when you can.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Overview of SMB signing in Windows](https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-signing-overview)
      - [Control SMB signing behavior](https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-signing)
      - [SMB signing required by default in Windows Insider](https://techcommunity.microsoft.com/blog/filecab/smb-signing-required-by-default-in-windows-insider/3831704)
```

**Sources:**
1. Overview of Server Message Block signing in Windows, https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-signing-overview (tier A)
2. Control SMB signing behavior, https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-signing (tier A)
3. SMB signing required by default in Windows Insider, https://techcommunity.microsoft.com/blog/filecab/smb-signing-required-by-default-in-windows-insider/3831704 (tier B)
4. `docs/superpowers/research/validation/_harmful-revert.md`, "Confirmed, documentation-backed" table

### `rdp_security_hardening` Harden RDP (NLA + TLS)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Two REG_DWORD values under
`HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server\WinStations\RDP-Tcp`:

- `UserAuthentication`: 1 requires Network Level Authentication (CredSSP completes before a session
  is created)
- `SecurityLayer`: 0 native RDP security, 1 negotiate, 2 TLS

Options: "NLA + TLS required" = 1 / 2, "Windows default (Stock Default)" = **0 / 1**. No reboot flag,
correct: the change applies to new connections.

CORRECTED revert: restore the **captured pre-apply values** (or `absent` if the engine treats absent as
"restore from snapshot"). Microsoft's unattend reference gives `UserAuthentication` = 0 as the
*component* default, but Windows 10 and 11 ship with "Allow connections only from computers running
Remote Desktop with Network Level Authentication" checked, so the value on a real machine is 1.
Reverting to 0 turns NLA off rather than restoring it.

**Corrections needed:** Replace the hardcoded 0 / 1 revert with a snapshot restore. The shipped values
should be confirmed on a clean 26100 install; this is listed in UNKNOWNS.

**Ready-to-paste info block:**

```yaml
    info: |
      **Locks Remote Desktop to Network Level Authentication over TLS, killing pre-auth attacks.**

      ## What it does
      Sets `UserAuthentication` = 1 and `SecurityLayer` = 2 on the RDP-Tcp listener. NLA makes the
      client authenticate through CredSSP before any desktop session is created, and `SecurityLayer`
      2 forces TLS instead of letting the connection negotiate down to native RDP security.

      ## Benefits
      - **No pre-auth surface**: an unauthenticated caller never reaches session creation, which is
        the mitigation Microsoft named for BlueKeep (CVE-2019-0708)
      - **No downgrade**: TLS is required rather than negotiated
      - **Cheap**: nothing else about RDP changes

      ## Drawbacks
      - **Old clients cannot connect**: RDP clients without CredSSP or NLA support are refused
      - **Inert if RDP is off**: with `fDenyTSConnections` = 1 these values do nothing
      - **Group Policy wins**: settings under `Policies\Microsoft\Windows NT\Terminal Services`
        override this listener key
      - **Careful reverting**: Windows ships NLA required, so a revert must restore the captured
        value rather than writing 0

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, Pro / Enterprise / Education (no effect on Home)
      - **Takes effect**: immediately, on the next connection
      - **Reverting**: restores the captured pre-apply values from the snapshot
      - If you do not use Remote Desktop at all, denying the listener outright is the stronger move

      ## Recommendation
      If you use Remote Desktop, apply this; NLA plus TLS should be treated as mandatory for any
      reachable RDP listener. If you do not use RDP, disable it instead.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [SecurityLayer (unattend reference)](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-terminalservices-rdp-winstationextensions-securitylayer)
      - [UserAuthentication (unattend reference)](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-terminalservices-rdp-winstationextensions-userauthentication)
      - [Enable Remote Desktop on your PC (NLA rationale)](https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access)
```

**Sources:**
1. SecurityLayer (unattend reference), https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-terminalservices-rdp-winstationextensions-securitylayer (tier A)
2. UserAuthentication (unattend reference), https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-terminalservices-rdp-winstationextensions-userauthentication (tier A)
3. Enable Remote Desktop on your PC, https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access (tier A)

### `disable_admin_shares` Administrative shares (C$, ADMIN$)

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters` -> `AutoShareWks`
(REG_DWORD). Options: "Disabled" = 0, "Windows default (Stock Default)" = `absent`.
`requires_reboot: true`.

The Server service recreates the hidden per-drive administrative shares (`C$`, `D$`, `ADMIN$`) each
time it starts. `AutoShareWks` = 0 on a workstation SKU suppresses that; server SKUs read
`AutoShareServer`. Microsoft's documented apply step is `net stop server` then `net start server`, not
a reboot. The value does not affect `IPC$` or manually created shares.

**Corrections needed:** `none`. The `requires_reboot: true` flag is stronger than Microsoft's
documented service restart, which is harmless. One open question is recorded in UNKNOWNS: a string scan
of all 4,393 System32 modules on 26100.4061 found `AutoShareWks` only in `smbwmiv2.dll` (the provider
behind `Get-SmbServerConfiguration`), not in `srvsvc.dll`, so it is worth confirming empirically that a
raw registry write is still honoured rather than only the SMB configuration store.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows recreating the hidden `C$` and `ADMIN$` shares attackers use to spread.**

      ## What it does
      Sets `AutoShareWks` = 0 under the `LanmanServer` parameters key. The Server service normally
      recreates the hidden per-drive administrative shares every time it starts; with this set, it
      does not. `IPC$` and any shares you created yourself are unaffected.

      ## Benefits
      - **Removes lateral-movement targets**: PsExec, wmiexec and smbexec stage files through `ADMIN$`
      - **Defence in depth**: credential reuse against this machine has one less easy path
      - **Reversible**: the shares come back when the value is removed and the service restarts

      ## Drawbacks
      - **Remote admin breaks**: remote-administration tooling, some backup products and remote
        WMI or DCOM workflows that stage files through `ADMIN$` stop working
      - **Not a boundary**: an administrator who reaches the machine can recreate the shares with
        `net share`, so treat this as friction
      - **Microsoft advises against it**: their guidance is not to remove administrative shares
        "because it can break many different things"

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 (client SKUs use
        `AutoShareWks`, server SKUs use `AutoShareServer`)
      - **Takes effect**: after reboot (Microsoft's documented step is a Server service restart, so
        the reboot is deliberately conservative)
      - **Reverting**: deletes the value, restoring automatic share creation
      - Manually created shares and `IPC$` are never affected

      ## Recommendation
      Good on a standalone personal PC that never uses these shares. Avoid it if you rely on remote
      admin tooling, network backup agents or PsExec-style management.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Remove administrative shares](https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/remove-administrative-shares)
```

**Sources:**
1. Remove administrative shares, https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/remove-administrative-shares (tier A)

### `disable_lmhash_storage` Prevent LM hash storage

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` -> `NoLMHash` (REG_DWORD). **Single option:
"Enabled" = 1. There is no second option, so the tweak has no revert path from the UI.**
`requires_reboot: true`.

Backs *Network security: Do not store LAN Manager hash value on next password change*. Microsoft's
default-values table gives "Client Computer Effective Default Settings: **Enabled**", and "Restart
requirement: **None**". The setting applies at the next password change and does not remove an LM hash
already stored.

**Corrections needed:** (a) Add a revert option. The correct Windows default is `no_lm_hash: 1`
(Enabled), so this is a pin-the-default tweak with no meaningful alternative state; say so plainly
rather than inventing a 0 option, which would be a security downgrade. (b) Remove
`requires_reboot: true`; Microsoft documents no restart requirement, and what gates the effect is the
next password change. See `_cross-category.md` section 2.1 for the single-option structural defect.

**Ready-to-paste info block:**

```yaml
    info: |
      **Keeps the ancient, easily cracked LM hash of your password off the disk.**

      ## What it does
      Sets `NoLMHash` = 1 under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`. From the next time a
      password changes, Windows stops writing the DES-based LAN Manager hash into the SAM and keeps
      only the NT hash.

      ## Benefits
      - **Removes a crackable artefact**: LM hashes fall to offline cracking almost instantly
      - **Pins the safe state**: the value cannot be quietly lowered without the change showing up
      - **Zero compatibility cost**: nothing modern authenticates against an LM hash

      ## Drawbacks
      - **No visible change**: Enabled has been the effective default since Windows Vista
      - **Does not clean up**: an LM hash already in the SAM stays until that password changes, so
        Microsoft's countermeasure guidance is to also make everyone set a new password
      - **Rare legacy break**: Microsoft notes "some non-Microsoft applications might not be able to
        connect to the system"
      - **No alternative state**: the shipped value is the same value the tweak writes, so there is
        nothing meaningful to revert to

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: at the next password change; Microsoft documents no restart requirement
      - **Reverting**: restores the previous value from the snapshot, which on a stock machine is the
        same value
      - The effect is about future password changes, not about the hashes already stored

      ## Recommendation
      Apply it. It is pure upside on current Windows, and pinning the value means a future
      misconfiguration cannot quietly reintroduce LM hashes.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Network security: Do not store LAN Manager hash value on next password change](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-do-not-store-lan-manager-hash-value-on-next-password-change)
```

**Sources:**
1. Network security: Do not store LAN Manager hash value on next password change, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-do-not-store-lan-manager-hash-value-on-next-password-change (tier A)
2. `docs/superpowers/research/validation/_cross-category.md` section 2.1 (single-option tweaks)

### `restrict_anonymous_enum` Restrict anonymous enumeration

**Verdict:** VERIFIED

**Mechanism:** Three REG_DWORD values under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`:
`RestrictAnonymousSAM`, `RestrictAnonymous`, `EveryoneIncludesAnonymous`. Options: "Restricted" =
1 / 1 / 0, "Windows default (Stock Default)" = 1 / 0 / 0. `requires_reboot: true`.

Microsoft's default-values tables give Client Computer Effective Default: `RestrictAnonymousSAM`
Enabled (1), `RestrictAnonymous` Disabled (0), `EveryoneIncludesAnonymous` Disabled (0), so the stock
triple 1 / 0 / 0 is exactly right. All three state "Restart requirement: None". None affects domain
controllers.

**Revert safety, confirmed.** This is the only tweak in its group whose revert writes literals rather
than `absent`, so it was attacked hardest and it holds. The shipped default security template
`C:\Windows\inf\defltbase.inf` contains all three explicitly in `[Registry Values]`:
`RestrictAnonymousSAM=4,1`, `RestrictAnonymous=4,0`, `EveryoneIncludesAnonymous=4,0` (type code 4 is
REG_DWORD). That is a Microsoft-authored file describing the product, and it matches the YAML exactly.

**Corrections needed:** `none`. The `requires_reboot: true` flag exceeds Microsoft's documented "no
restart required", which is harmless.

**Ready-to-paste info block:**

```yaml
    info: |
      **Denies anonymous network callers a list of this PC's accounts and shares.**

      ## What it does
      Sets three LSA security options: `RestrictAnonymousSAM` = 1 (no anonymous SAM account
      enumeration), `RestrictAnonymous` = 1 (no anonymous enumeration of accounts **or** shares) and
      `EveryoneIncludesAnonymous` = 0 (the Everyone SID is not added to an anonymous token). On a
      stock machine only the middle value actually changes.

      ## Benefits
      - **Blunts reconnaissance**: an unauthenticated caller cannot build a target list of usernames
      - **Hides shares**: anonymous share enumeration stops as well
      - **Safe value**: uses `RestrictAnonymous` = 1, not the legacy 2 that breaks domain trusts

      ## Drawbacks
      - **Legacy discovery breaks**: old cross-domain setups with one-way trusts and appliances that
        enumerate anonymously stop working
      - **Small real delta**: `RestrictAnonymousSAM` is already 1 by default, so only
        `RestrictAnonymous` changes on a clean machine
      - **Drifts back on managed PCs**: these are Security Options delivered by the security
        client-side extension, so a domain GPO refresh, an Intune baseline or a local
        `secedit /configure /cfg %windir%\inf\defltbase.inf` resets `RestrictAnonymous` to 0

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2. No effect on domain
        controllers
      - **Takes effect**: after reboot (Microsoft documents no restart requirement, so the flag is
        deliberately conservative)
      - **Reverting**: writes back Windows' own shipped values 1 / 0 / 0, which are present verbatim
        in the shipped default security template
      - Never use `RestrictAnonymous` = 2; it is known to break networking and trusts

      ## Recommendation
      Apply it on standalone or home networks. Be cautious in older domain environments that still
      depend on anonymous enumeration, and expect it to drift back on a managed machine.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Network access: Do not allow anonymous enumeration of SAM accounts](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-do-not-allow-anonymous-enumeration-of-sam-accounts)
      - [Network access: Do not allow anonymous enumeration of SAM accounts and shares](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-do-not-allow-anonymous-enumeration-of-sam-accounts-and-shares)
      - [Network access: Let Everyone permissions apply to anonymous users](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-let-everyone-permissions-apply-to-anonymous-users)
```

**Sources:**
1. Network access: Do not allow anonymous enumeration of SAM accounts, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-do-not-allow-anonymous-enumeration-of-sam-accounts (tier A)
2. Network access: Do not allow anonymous enumeration of SAM accounts and shares, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-do-not-allow-anonymous-enumeration-of-sam-accounts-and-shares (tier A)
3. Network access: Let Everyone permissions apply to anonymous users, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-let-everyone-permissions-apply-to-anonymous-users (tier A)
4. `C:\Windows\inf\defltbase.inf`, `[Registry Values]` (tier A, shipped Microsoft default security template)

### `reduce_credential_caching` Reduce cached domain logons

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` -> `CachedLogonsCount`
(**REG_SZ**, a string of digits). Options: "Limit to 4" = "4", "Windows default (10)" = "10". No
`requires_reboot`, correct.

Backs *Interactive logon: Number of previous logons to cache*. Microsoft's possible values are 0
through 50; the default-values table gives "Client Computer Effective Default Settings: 10 logons", and
the registry type is documented as REG_SZ in Microsoft's cached-domain-logon support article. Microsoft
states "Restart requirement: None". The REG_SZ typing matters: written as a DWORD, Winlogon does not
read it and the limit silently never applies.

**Corrections needed:** (a) The `info` block ends "A reboot is needed". Microsoft's page states under
Restart requirement: "None. Changes to this policy become effective without a computer restart when
they're saved locally or distributed through Group Policy." The YAML's own (correct) omission of
`requires_reboot` contradicts its prose, and the prose is the wrong side. (b) Authoring convention: the
revert label "Windows default (10)" lacks the "(Stock Default)" marker that 198 other options in the
corpus carry and that `TweakCard.svelte` uses to pick the toggle's OFF position
(`optionLabels.find((l) => /\(stock default\)/i.test(l)) ?? optionLabels[1]`). It works today only
through the positional fallback and would silently invert if the options were reordered.

**Ready-to-paste info block:**

```yaml
    info: |
      **Shrinks the pool of cached domain credentials an attacker could crack offline.**

      ## What it does
      Lowers `CachedLogonsCount` to 4. That value bounds how many distinct domain users' verifier
      hashes Winlogon keeps locally so they can still sign in with no domain controller reachable.
      It is written as a `REG_SZ` string of digits, which is how Winlogon reads it.

      ## Benefits
      - **Fewer offline targets**: each cached entry is a crackable DCC2 hash if the SECURITY hive
        is stolen
      - **Keeps offline logon working**: 4 is small enough to matter and large enough to be usable
      - **No restart**: the change is live as soon as it is written

      ## Drawbacks
      - **Domain-only**: on a machine that is not domain-joined or Entra-joined it does nothing
      - **Lockout risk if set too low**: users who have not signed in recently while off the network
        can be locked out; 0 disables caching entirely
      - **Type-sensitive**: written as a DWORD instead of a string, Winlogon ignores it and the limit
        never applies
      - **Not in Microsoft's baselines**: Microsoft's security baselines deliberately leave it unset

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2. Meaningful only on
        domain-joined or Entra-joined machines
      - **Takes effect**: immediately, no restart is required
      - **Reverting**: writes back Microsoft's documented client default of 10
      - Microsoft suggests 2 for end-user computers as a workable balance

      ## Recommendation
      Useful on managed or domain-joined laptops. Skip it on ordinary home PCs, where it changes
      nothing at all.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Interactive logon: Number of previous logons to cache](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-number-of-previous-logons-to-cache-in-case-domain-controller-is-not-available)
      - [Cached domain logon information](https://github.com/MicrosoftDocs/SupportArticles-docs/blob/main/support/windows-server/user-profiles-and-logon/cached-domain-logon-information.md)
```

**Sources:**
1. Interactive logon: Number of previous logons to cache, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-number-of-previous-logons-to-cache-in-case-domain-controller-is-not-available (tier A)
2. Cached domain logon information, https://github.com/MicrosoftDocs/SupportArticles-docs/blob/main/support/windows-server/user-profiles-and-logon/cached-domain-logon-information.md (tier A, Microsoft support article source)

### `block_vulnerable_drivers` Microsoft vulnerable-driver blocklist

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\CI\Config` -> `VulnerableDriverBlocklistEnable`
(REG_DWORD). Options: "Enabled" = 1, "Windows default (Stock Default)" = `absent`.
`requires_reboot: true`, correct (already-loaded drivers are only blocked after a restart).

Microsoft documents the **feature** and its Windows Security app toggle, and states "Since the
Windows 11 2022 update, the vulnerable driver blocklist is enabled by default for all devices". It does
**not** document this registry value name; that rests on tier C sources only. The blocklist is also
force-enforced whenever memory integrity (HVCI), Smart App Control or S mode is active, in which case
the toggle is greyed out and the registry value cannot override it.

**Corrections needed:** (a) State in the info text that the registry value is community-sourced rather
than Microsoft-documented. (b) State that on Windows 11 22H2 and later this confirms an existing state,
and that it cannot override HVCI, Smart App Control or S mode enforcement in either direction. Recorded
in UNKNOWNS: verify that toggling the Windows Security control writes exactly this value.

**Ready-to-paste info block:**

```yaml
    info: |
      **Refuses to load kernel drivers on Microsoft's known-vulnerable list.**

      ## What it does
      Sets `VulnerableDriverBlocklistEnable` = 1 under `HKLM\SYSTEM\CurrentControlSet\Control\CI\Config`,
      the value behind the Windows Security app's vulnerable-driver blocklist toggle. Windows then
      refuses to load kernel drivers that appear on Microsoft's blocklist of known-vulnerable or
      malicious signed drivers.

      ## Benefits
      - **Stops BYOVD**: bring-your-own-vulnerable-driver is the standard way malware disables EDR
        and reaches the kernel
      - **Maintained by Microsoft**: the in-OS list is refreshed through monthly and quarterly
        servicing
      - **Pins the state**: the toggle cannot be flipped off without the change being visible

      ## Drawbacks
      - **Some legitimate drivers stop loading**: Microsoft notes blocking drivers "can cause devices
        or software to malfunction, and in rare cases, lead to blue screen"
      - **Already on since 22H2**: on Windows 11 22H2 and later this confirms the current state
      - **Cannot override enforcement**: with HVCI, Smart App Control or S mode active the blocklist
        is forced on and this value has no say either way
      - **Registry value is community-sourced**: Microsoft documents the feature but not this value
        name, so a future build could stop honouring it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 (enabled via KB5018482 and
        Windows 11 KB5018483 / KB5018496)
      - **Takes effect**: after reboot, so already-loaded drivers keep running until then
      - **Reverting**: deletes the value, returning control to the Windows Security app toggle
      - Microsoft's downloadable blocklist is usually more complete than the in-OS one

      ## Recommendation
      Apply it. The small chance of a blocked legacy driver is well worth closing the kernel
      escalation path, and on 22H2 and later you are confirming a state Microsoft already ships.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [Microsoft recommended driver block rules](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules)
      - [Enable or disable the Microsoft vulnerable driver blocklist in Windows 11](https://www.elevenforum.com/t/enable-or-disable-microsoft-vulnerable-driver-blocklist-in-windows-11.10031/)
      - [SigmaHQ rule: vulnerable driver blocklist disabled](https://detection.fyi/sigmahq/sigma/windows/registry/registry_set/registry_set_vulnerable_driver_blocklist_disable/)
```

**Sources:**
1. Microsoft recommended driver block rules, https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules (tier A, feature and defaults)
2. Enable or Disable Microsoft Vulnerable Driver Blocklist in Windows 11, https://www.elevenforum.com/t/enable-or-disable-microsoft-vulnerable-driver-blocklist-in-windows-11.10031/ (tier C, registry value)
3. SigmaHQ rule: Vulnerable Driver Blocklist Disabled, https://detection.fyi/sigmahq/sigma/windows/registry/registry_set/registry_set_vulnerable_driver_blocklist_disable/ (tier C, corroborates the value name)

### `disable_wpbt` WPBT vendor binary execution

**Verdict:** VERIFIED-WITH-CORRECTION (community-corroborated, not Microsoft-documented)

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager` -> `DisableWpbtExecution`
(REG_DWORD). Options: "Disabled" = 1, "Windows default (Stock Default)" = `absent`.
`requires_reboot: true`. Key, value name, type, polarity, the `absent` stock default and the reboot
flag are all correct.

The Windows Platform Binary Table is an ACPI table through which firmware hands Windows a PE binary.
Session Manager (`smss.exe`) extracts it to `%SystemRoot%\System32\wpbbin.exe` and executes it every
boot. `DisableWpbtExecution` = 1 makes Session Manager skip that step. Direct inspection of
`C:\Windows\System32\smss.exe` on 26100.4061 finds the UTF-16 strings `DisableWpbtExecution` and
`wpbbin` inside the binary that performs the extraction, which is primary evidence the value name is
read on the target build.

**Corrections needed:** The info text's claim that the value is "corroborated by security researchers
(Eclypsium and others)" is false and must be removed. Eclypsium's WPBT research ("Everyone Gets a
Rootkit", 2021) documents the attack surface and names WDAC as Microsoft's mitigation; it never
mentions this registry value. Replace it with the accurate attribution: present in `smss.exe` on
26100.4061 and corroborated by four independent tier C projects (dropWPBT,
persistence-info.github.io, WinUtil, Security-ADMX). Label the tweak community-corroborated.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks firmware-supplied binaries from auto-running, cutting a channel that survives a wipe.**

      ## What it does
      Sets `DisableWpbtExecution` = 1 under `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager`.
      WPBT lets the motherboard firmware hand Windows an executable that Session Manager extracts to
      `System32\wpbbin.exe` and runs at every boot. With this set, Session Manager skips it.

      ## Benefits
      - **Kills firmware persistence**: a WPBT payload lives in firmware and re-runs after a clean
        Windows reinstall
      - **Stops OEM bloat returning**: vendor software that reappears after a wipe stops doing so
      - **Confirmed in the shipped binary**: the value name is present inside `smss.exe` on 26100

      ## Drawbacks
      - **Breaks legitimate OEM features**: some vendors deliver drivers or anti-theft agents through
        WPBT and those will not reinstall
      - **Not Microsoft-documented**: there is no contract that a future build keeps honouring it,
        and a mistyped value under Session Manager is ignored silently rather than rejected
      - **Hard to verify**: the only positive check is confirming `wpbbin.exe` is no longer produced
        after a reboot on a machine whose firmware actually publishes a WPBT table
      - **Does nothing on most machines**: WPBT only fires where the firmware publishes the table,
        mostly OEM prebuilt systems

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; the feature exists on Windows 8 and later, all
        editions
      - **Takes effect**: after reboot
      - **Reverting**: deletes the value, which is the shipped state
      - Microsoft's own published mitigations are a WDAC policy (enforced for WPBT binaries), a DFCI
        firmware setting managed through Intune, or firmware-level removal

      ## Recommendation
      A solid hardening step for a clean, self-managed install where you do not want OEM firmware
      software returning. Avoid it if you rely on a vendor anti-theft or firmware-delivered driver.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [dropWPBT: disables the Windows Platform Binary Table](https://github.com/Jamesits/dropWPBT)
      - [Windows Platform Binary Table, persistence-info reference](https://persistence-info.github.io/Data/wpbbin.html)
      - [Chris Titus WinUtil, WPBT tweak definition](https://winutil.christitus.com/dev/tweaks/essential-tweaks/wpbt/)
```

**Sources:**
1. Direct binary inspection, Windows 11 24H2 build 26100.4061: `C:\Windows\System32\smss.exe` contains the UTF-16 strings `DisableWpbtExecution` and `wpbbin` (primary measurement)
2. dropWPBT, https://github.com/Jamesits/dropWPBT (tier C)
3. Windows Platform Binary Table, persistence-info.github.io, https://persistence-info.github.io/Data/wpbbin.html (tier C)
4. Chris Titus WinUtil tweak `WPFTweaksWPBT`, https://winutil.christitus.com/dev/tweaks/essential-tweaks/wpbt/ (tier C)
5. Security-ADMX issue 8, "Disable WPBT execution" policy, https://github.com/Harvester57/Security-ADMX/issues/8 (tier C)
6. "Everyone Gets a Rootkit", Eclypsium, https://eclypsium.com/blog/everyone-gets-a-rootkit/ (tier C; cited only to show it documents the WPBT threat and the WDAC mitigation, **not** this registry value)

### `require_ctrlaltdel` Require Ctrl+Alt+Del at sign-in

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` ->
`DisableCAD` (REG_DWORD). Options: "Required" = 0, "Windows default (Stock Default)" = `absent`. No
reboot flag, correct.

The polarity is inverted relative to the tweak name: the policy is *Do not require CTRL+ALT+DEL*, so
`DisableCAD` = 0 means Ctrl+Alt+Del **is** required. The YAML gets that right, and Microsoft explicitly
validates the `absent` revert: to revert this policy "it is not enough to set its value to Not defined,
this registry value needs to be removed as well".

CORRECTED mechanism, add a second effect:

| Key | Value name | Type | Required | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` | `DisableCAD` | REG_DWORD | `0` | `absent` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `DisableCAD` | REG_DWORD | `0` | **`1`** |

On 26100.4061 the `Policies\System` value is absent (so already effectively 0) while the Winlogon value
is present as REG_DWORD 1, and that is what produces the observed "no Ctrl+Alt+Del at sign-in"
behaviour. A UTF-16 string scan of all 4,393 System32 modules splits the references cleanly: in
`authui.dll` (the component that renders or skips the "Press Ctrl+Alt+Delete" screen) `DisableCAD` sits
immediately beside `Software\Microsoft\Windows NT\CurrentVersion\Winlogon`, and in `winlogon.exe` it
sits inside the Winlogon value-name block next to `AutoAdminLogon` and `AutoRestartShell`. Only
`credprovs.dll` pairs it with the `Policies\System` path.

**Corrections needed:** Add the `Winlogon\DisableCAD` effect above. Keep the existing `Policies\System`
effect with its `absent` revert; that half is correct. As authored the tweak writes 0 to a value that
is already effectively 0 and leaves the store that actually suppresses the secure attention sequence
untouched, which makes it at best partial and quite possibly a silent no-op on 26100.

**Ready-to-paste info block:**

```yaml
    info: |
      **Forces the secure Ctrl+Alt+Del sequence at sign-in so a fake login screen cannot take your
      password.**

      ## What it does
      Writes `DisableCAD` = 0 in both stores Windows reads: the policy store under
      `Policies\System` and the Winlogon store under
      `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon`. Only the Windows kernel can
      intercept Ctrl+Alt+Del, so pressing it guarantees you are typing into the real sign-in screen.

      ## Benefits
      - **Defeats credential-harvesting overlays**: a spoofed login window cannot receive the secure
        attention sequence
      - **Covers both stores**: the Winlogon value is the one that actually suppresses the prompt on
        24H2, and the policy value alone is not enough
      - **Classic, well understood**: no compatibility surprises

      ## Drawbacks
      - **One extra keypress**: at every sign-in and after every lock
      - **Awkward without a keyboard**: touch-only tablets, convertibles and kiosk devices have no
        easy way to send the sequence
      - **No effect on remote threats**: this is a physical-presence protection only

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: at your next sign-in, no reboot
      - **Reverting**: deletes the policy value (Microsoft states the value must be removed, not just
        set to Not Defined) and restores the Winlogon value to the shipped `1`
      - On a fresh non-domain install Windows does not prompt for Ctrl+Alt+Del even though the
        security-policy default reads "Disabled"; the Winlogon value is why

      ## Recommendation
      Worth it on desktops and laptops that care about sign-in security. Skip it on touch-only
      tablets and keyboard-less kiosks where sending the sequence is impractical.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Interactive logon: Do not require CTRL+ALT+DEL](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-do-not-require-ctrl-alt-del)
```

**Sources:**
1. Interactive logon: Do not require CTRL+ALT+DEL, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-do-not-require-ctrl-alt-del (tier A)
2. On-box registry state and `secedit /export /areas SECURITYPOLICY` on Windows 11 24H2 build 26100.4061 (tier A for value presence in shipped binaries; not admissible as a default)
3. UTF-16 string scan of `C:\Windows\System32\authui.dll`, `winlogon.exe` and `credprovs.dll` on 26100.4061 (tier A, shipped binaries)

### `disable_autorun` AutoRun/AutoPlay on all drives

**Verdict:** VERIFIED

**Mechanism:** Two REG_DWORD values under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer`: `NoDriveTypeAutoRun` = 255 and
`NoAutorun` = 1. Revert: both `absent`. No reboot flag, correct.

Microsoft's Autoplay Policy CSP confirms both live under
`Software\Microsoft\Windows\CurrentVersion\Policies\Explorer`: *Turn off Autoplay* (value
`NoDriveTypeAutoRun`) and *Set the default behavior for AutoRun* (value `NoAutorun`).
`NoDriveTypeAutoRun` is a drive-type bitmask and 255 (0xFF) covers every type. `NoAutorun` = 1
completely disables autorun commands rather than prompting.

**Corrections needed:** `none` for what is written. A third value in the same family is missing:
*Disallow Autoplay for non-volume devices* (`NoAutoplayfornonVolume` under
`Software\Policies\Microsoft\Windows\Explorer`, class Both) covers MTP devices such as phones and
cameras that never get a drive letter. It is tracked separately as `autoplay_non_volume`; the cleanest
fix is to fold it in here as a third effect written to both hives.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops USB sticks and discs from auto-running anything.**

      ## What it does
      Sets `NoDriveTypeAutoRun` = 255 and `NoAutorun` = 1 under the Explorer policies key. The first
      is a bitmask covering every drive type; the second disables autorun commands outright rather
      than prompting. Windows stops acting on `autorun.inf` and stops showing AutoPlay prompts.

      ## Benefits
      - **Closes a classic worm vector**: a generation of USB worms ran the moment media was attached
      - **Covers every drive type**: fixed, removable, network, optical and RAM disks
      - **No prompts to misclick**: AutoPlay cannot offer to run something for you

      ## Drawbacks
      - **Manual opening**: you browse removable media in File Explorer yourself
      - **AutoPlay handlers stop**: camera import and disc-burning prompts no longer appear
      - **Smaller gain than it sounds**: `autorun.inf` execution from removable drives has been off
        by default since the Windows 7 era, so this mostly closes prompts and edge cases
      - **Misses MTP devices**: phones and cameras that get no drive letter need
        `NoAutoplayfornonVolume`, which this tweak does not set

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1703 and later. The policy UI is
        Pro and above but the registry values are read on every edition
      - **Takes effect**: after sign-out or an Explorer restart, no reboot
      - **Reverting**: deletes both values, restoring Windows' own AutoPlay behaviour

      ## Recommendation
      Apply it. The manual step is trivial and it removes an old malware vector plus every AutoPlay
      prompt. Pair it with the non-volume AutoPlay control for full coverage.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Autoplay Policy CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-autoplay)
```

**Sources:**
1. Autoplay Policy CSP, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-autoplay (tier A)
2. `C:\Windows\PolicyDefinitions\AutoPlay.admx` (26100), policy `NoAutoplayfornonVolume` (tier A, for the missing third value)

### `powershell_scriptblock_logging` PowerShell script-block logging

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging` ->
`EnableScriptBlockLogging` (REG_DWORD). Options: "Enabled" = 1, "Windows default (Stock Default)" =
`absent`. No reboot flag, correct: new sessions pick it up.

Microsoft's `about_Logging` for PowerShell 5.1 gives exactly this path and value name. Logging happens
after decoding, so `-EncodedCommand`, string concatenation and `Invoke-Expression` obfuscation are all
defeated; the deobfuscated text lands in `Microsoft-Windows-PowerShell/Operational` event 4104. The
`_policy-hive-audit.md` pass notes the ADMX class is Both, and confirms the HKLM write is the correct
machine-wide choice.

CORRECTED mechanism, add a second key for PowerShell 7:

| Key | Value name | Type | Enabled | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging` | `EnableScriptBlockLogging` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\PowerShellCore\ScriptBlockLogging` | `EnableScriptBlockLogging` | REG_DWORD | `1` | `absent` |

**Corrections needed:** Add the `PowerShellCore` key, gated on PowerShell 7 being present, or state in
the info text that `pwsh.exe` activity is unlogged. The companion `EnableScriptBlockInvocationLogging`
(events 4105 and 4106) is deliberately not set, which is correct: it is extremely noisy for little
added value.

**Ready-to-paste info block:**

```yaml
    info: |
      **Records the real, deobfuscated PowerShell that runs on this PC.**

      ## What it does
      Sets `EnableScriptBlockLogging` = 1 under the PowerShell policy key. Windows PowerShell then
      writes the fully decoded text of every script block it compiles to event 4104 in the
      `Microsoft-Windows-PowerShell/Operational` log. Because logging happens after decoding,
      `-EncodedCommand` and string-concatenation obfuscation are defeated.

      ## Benefits
      - **Highest-value endpoint log**: turns the most common living-off-the-land tool into evidence
      - **Beats obfuscation**: the readable command is logged, not the encoded blob
      - **No performance cost worth measuring**: it is a write to an existing channel

      ## Drawbacks
      - **Secrets can leak into the log**: scripts that carry plaintext credentials write them to
        Event Viewer, so treat the PowerShell operational log as sensitive
      - **Log volume grows**: pair it with a larger event log or accept faster rollover
      - **PowerShell 7 is separate**: `pwsh.exe` reads
        `HKLM\SOFTWARE\Policies\Microsoft\PowerShellCore\ScriptBlockLogging`, so activity through
        PowerShell 7 is unlogged unless that key is set too

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2. Windows PowerShell 5.1 by
        default, plus PowerShell 7 when the second key is written
      - **Takes effect**: immediately, for new PowerShell sessions
      - **Reverting**: deletes the value, which is the shipped state
      - Microsoft recommends pairing this with Protected Event Logging if secrets are a concern

      ## Recommendation
      Enable it if you care about detection or incident response; it is the single highest-value
      logging setting on a Windows endpoint. Weigh the exposure first only if you routinely run
      scripts with hardcoded secrets and cannot restrict access to the log.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [about_Logging (PowerShell 5.1)](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging?view=powershell-5.1)
      - [about_Logging_Windows (PowerShell 7)](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
```

**Sources:**
1. about_Logging (PowerShell 5.1), https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging?view=powershell-5.1 (tier A)
2. about_Logging_Windows (PowerShell 7), https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows (tier A)
3. `docs/superpowers/research/validation/_policy-hive-audit.md` (ADMX class Both; HKLM is the effective machine-wide store)

### `uac_max` Raise UAC to always notify

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Three REG_DWORD values under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`: `ConsentPromptBehaviorAdmin`,
`PromptOnSecureDesktop`, `EnableLUA`. Options: "Always notify" = 2 / 1 / 1, "Windows default (Stock
Default)" = 5 / 1 / 1. **No `requires_reboot` flag.**

All six numeric values are confirmed. Microsoft's default-values table for
*User Account Control: Behavior of the elevation prompt for administrators in Admin Approval Mode*
gives "Client Computer Effective Default Settings: Prompt for consent for non-Windows binaries", which
is value **5**, and the page states "This prompt for consent is the default". "Prompt for consent on
the secure desktop" is value **2**. So the stock triple 5 / 1 / 1 and the applied triple 2 / 1 / 1 are
both correct, and the revert is safe against all three revert failure shapes.

CORRECTED metadata: **`requires_reboot: true`**. The flag was reasoned from the
`ConsentPromptBehaviorAdmin` page's "Restart requirement: None", but this tweak also writes
`EnableLUA`, a different security option (*Run all administrators in Admin Approval Mode*) that
Microsoft documents as "A computer restart is required before this policy will be effective". Both
options write it, so on any machine where UAC was previously off, applying either option needs a
reboot and the YAML does not say so.

**Corrections needed:** Set `requires_reboot: true`. Note in the info text that on 24H2 and newer,
machines with Administrator Protection enabled (`TypeOfAdminApprovalMode` = 2) are governed by
`ConsentPromptBehaviorEnhancedAdmin` rather than `ConsentPromptBehaviorAdmin`, so this tweak is no
longer sufficient there. Note also that these three values are Security Options delivered on managed
machines through `GptTmpl.inf`, so a baseline refresh can re-assert them; they are **not** in
`C:\Windows\inf\defltbase.inf`, so a plain `secedit /configure` will not reset them but a GPO that
defines them will. Finally, "Stock Default" is a genuine downgrade on a machine already hardened past
stock, since CIS and DISA STIG both require `ConsentPromptBehaviorAdmin` = 2; that is inherent to
ADR-0003 rather than a defect, but the info text should not call the default "still reasonably safe"
without saying that choosing it can undo an existing baseline.

**Ready-to-paste info block:**

```yaml
    info: |
      **Makes every elevation need a deliberate confirmation on a tamper-proof secure desktop.**

      ## What it does
      Sets `ConsentPromptBehaviorAdmin` = 2 (consent required for every elevation, including signed
      Windows binaries), `PromptOnSecureDesktop` = 1 (the prompt renders on the isolated secure
      desktop) and `EnableLUA` = 1 (UAC and Admin Approval Mode stay on).

      ## Benefits
      - **Removes the auto-elevation allowance**: the Windows default value 5 elevates signed Windows
        binaries silently, which is what `fodhelper`, `eventvwr` and COM-based UAC bypasses abuse
      - **Prompt cannot be spoofed or clicked for you**: the secure desktop blocks other software
      - **Matches CIS and STIG**: value 2 is what both baselines require

      ## Drawbacks
      - **More prompts**: including for built-in Windows tools that used to elevate silently
      - **Needs a restart**: `EnableLUA` only takes effect after a reboot
      - **Not the whole story on 24H2**: with Administrator Protection on, elevation is governed by
        `ConsentPromptBehaviorEnhancedAdmin` instead and this tweak stops being the deciding value
      - **Managed machines drift**: a domain or Intune security baseline re-asserts UAC values at the
        next policy refresh and the tweak will report as no longer active

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after reboot
      - **Reverting**: writes Windows' own documented default triple (5 / 1 / 1)
      - Choosing the default option on a machine that was hardened to a CIS or STIG baseline drops it
        out of compliance, because those baselines require value 2
      - Do not confuse this with the "disable UAC" tweak that writes `EnableLUA` = 0; that breaks all
        packaged and UWP apps

      ## Recommendation
      Recommended for security-conscious users who can tolerate a few extra prompts. If the prompts
      are unacceptable, leave UAC at the Windows default rather than lowering it further.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [UAC: Behavior of the elevation prompt for administrators in Admin Approval Mode](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/user-account-control-behavior-of-the-elevation-prompt-for-administrators-in-admin-approval-mode)
      - [UAC: Run all administrators in Admin Approval Mode](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-run-all-administrators-in-admin-approval-mode)
```

**Sources:**
1. User Account Control: Behavior of the elevation prompt for administrators in Admin Approval Mode, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/user-account-control-behavior-of-the-elevation-prompt-for-administrators-in-admin-approval-mode (tier A)
2. User Account Control: Run all administrators in Admin Approval Mode, https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-run-all-administrators-in-admin-approval-mode (tier A)
3. `C:\Windows\inf\defltbase.inf` (tier A, shipped default security template): contains other `Policies\System` values but none of this tweak's three

### `filter_admin_token` Apply UAC to built-in Administrator

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` ->
`FilterAdministratorToken` (REG_DWORD). Options: "Enabled" = 1, "Windows default (Stock Default)" =
`absent`. `requires_reboot: true`.

Backs *User Account Control: Admin Approval Mode for the Built-in Administrator account*. Microsoft:
"By default, Admin Approval Mode is set to Disabled", and the default-values table gives Disabled for
client computers, so `absent` is a faithful revert. Microsoft states "Restart requirement: None", and
separately that after enabling you "must first log in and out" or run `gpupdate /force`.

CORRECTED metadata and prerequisite: remove `requires_reboot: true`, and pair with
`ConsentPromptBehaviorAdmin` = 2. Microsoft states that to enable Admin Approval Mode you "must also
configure the local security policy setting ... to Prompt for consent on the secure desktop".

**Corrections needed:** (a) Remove `requires_reboot: true`; Microsoft documents no restart requirement.
(b) Either pair this tweak with `ConsentPromptBehaviorAdmin` = 2 (see merge candidate 2) or state the
dependency in the `warning` field. Applying `FilterAdministratorToken` = 1 while
`ConsentPromptBehaviorAdmin` sits at a non-prompting value can leave the built-in Administrator unable
to elevate at all, which is a real lockout risk where that account is the only usable one.

**Ready-to-paste info block:**

```yaml
    info: |
      **Subjects even the built-in Administrator account to UAC prompts.**

      ## What it does
      Sets `FilterAdministratorToken` = 1. The built-in RID-500 Administrator then receives a
      filtered token like any other admin and must consent to elevation. Without it, that account
      runs everything with a full administrative token and never prompts.

      ## Benefits
      - **Removes a UAC blind spot**: no fully unfiltered admin token exists on the machine
      - **Covers a real case**: installers, OEM images and recovery workflows do enable that account
      - **Dormant when unused**: costs nothing while the account stays disabled

      ## Drawbacks
      - **Lockout risk if misconfigured**: Microsoft requires
        `ConsentPromptBehaviorAdmin` = 2 alongside this; without a prompting value the built-in
        Administrator can end up unable to elevate at all
      - **Breaks unattended automation**: anything running as the built-in Administrator that cannot
        answer a prompt stops working
      - **Usually invisible**: the account is disabled by default, so most machines see no change

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after signing out and back in, or `gpupdate /force`. Microsoft documents no
        restart requirement
      - **Reverting**: deletes the value, restoring the shipped Disabled state
      - On machines upgraded from an older Windows where Administrator was the only account, that
        account stays enabled, which is exactly where this matters most

      ## Recommendation
      Apply it as defence in depth, especially if you or anyone might enable the built-in
      Administrator. Set UAC to always notify at the same time so the account can still elevate.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [UAC: Admin Approval Mode for the Built-in Administrator account](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/user-account-control-admin-approval-mode-for-the-built-in-administrator-account)
```

**Sources:**
1. User Account Control: Admin Approval Mode for the Built-in Administrator account, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/user-account-control-admin-approval-mode-for-the-built-in-administrator-account (tier A)

### `hide_last_user` Hide last signed-in username

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` ->
`DontDisplayLastUserName` (REG_DWORD). Options: "Hidden" = 1, "Windows default (Stock Default)" =
`absent`. No reboot flag, correct.

CORRECTED stock default: **`0`, not `absent`.** On 26100.4061 the key contains
`dontdisplaylastusername` as REG_DWORD 0, and `secedit /export /areas SECURITYPOLICY` emits
`MACHINE\Software\Microsoft\Windows\CurrentVersion\Policies\System\DontDisplayLastUserName=4,0` (type 4
is REG_DWORD, data 0). It sits in the lowercase value cluster Windows setup writes from `defltbase.inf`
alongside `legalnoticecaption`, `scforceoption`, `shutdownwithoutlogon` and `undockwithoutlogon`.

**Corrections needed:** The "Windows default (Stock Default)" option must write `0`, not `absent`. This
is the inverse of the corpus's usual defect: the value ships seeded, so `absent` **deletes a shipped
security-option value**. Behaviourally absent and 0 are identical here, but after the revert `secedit`
and the Local Security Policy UI report the option as Not Defined where a stock machine reports
Disabled, and any later baseline comparison drifts. The corpus already handles this identical
setup-seeded pattern correctly in `restrict_anonymous_enum`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Leaves the sign-in screen blank instead of naming the last person who logged in.**

      ## What it does
      Sets `DontDisplayLastUserName` = 1, the *Interactive logon: Don't display last signed-in*
      security option. The sign-in screen shows an empty username field rather than pre-filling the
      last account.

      ## Benefits
      - **No free account name**: anyone with line of sight to the lock screen learns nothing
      - **Cheap**: one value, instant, fully reversible
      - **Useful in shared spaces**: offices, labs and public-facing machines

      ## Drawbacks
      - **You type your username every time**: at every sign-in and after every lock
      - **Worse with Microsoft accounts**: on Microsoft-account and Entra-joined machines you must
        type the full UPN
      - **Close to pointless at home**: on a single-user desktop nobody else sees the lock screen
      - **Does not hide the name during unlock**: that needs the separate `DontDisplayUserName` value

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: at your next sign-in, no reboot
      - **Reverting**: writes back the shipped value `0`; it must not delete the value, because
        Windows setup seeds it and deleting it leaves the security option reporting Not Defined
      - The related *Don't display username at sign-in* setting is a different value and is not
        covered here

      ## Recommendation
      Worth it in shared or physically exposed settings. On a private single-user PC the extra typing
      outweighs the small gain.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Interactive logon: Don't display last signed-in](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-do-not-display-last-user-name)
```

**Sources:**
1. Interactive logon: Don't display last signed-in, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-do-not-display-last-user-name (tier A)
2. On-box registry state and `secedit /export /areas SECURITYPOLICY` output on Windows 11 24H2 build 26100.4061, cross-checked against the `defltbase.inf` lowercase seeded cluster (tier A)

### `printnightmare_point_and_print` Restrict printer-driver install to admins

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint` ->
`RestrictDriverInstallationToAdministrators` (REG_DWORD). Options: "Admins only" = 1, "Windows default
(Stock Default)" = `absent`. No reboot flag, correct: the spooler picks it up.

Key path, value name and DWORD type are documented verbatim in KB5005010. This is the control Microsoft
shipped in the July and August 2021 servicing updates as the mitigation for CVE-2021-34527
(PrintNightmare).

CORRECTED default semantics: KB5005010 states "July 6, 2021 through August 9, 2021 updates: 0
(disabled); **August 10, 2021 and later updates: 1 (enabled)**". So on any currently patched machine
`absent` already behaves as 1.

**Corrections needed:** (a) The "Admins only" option is a confirm-the-default no-op on a patched
machine. (b) The "Windows default (Stock Default)" option deleting the value does **not** restore
unrestricted driver installation; it leaves the restriction active. If an unrestricted state is wanted
it must write 0 explicitly and carry a loud warning. (c) The info text's quote ("no other combination
of mitigations provides equivalent protection") comes from the CVE-2021-34527 MSRC advisory FAQ, not
KB5005010; cite the advisory or drop the claim.

**Ready-to-paste info block:**

```yaml
    info: |
      **Requires administrator rights to install a printer driver, closing the PrintNightmare path.**

      ## What it does
      Sets `RestrictDriverInstallationToAdministrators` = 1 under the Point and Print policy key.
      Without it, a standard user (or an attacker in any user context) can push a printer driver that
      the spooler loads as SYSTEM, which is the CVE-2021-34527 escalation.

      ## Benefits
      - **Closes a SYSTEM-level RCE path**: driver installation is the PrintNightmare primitive
      - **Microsoft's own mitigation**: shipped in the July and August 2021 servicing updates
      - **No security cost**: there is no downside to the applied state

      ## Drawbacks
      - **Standard users cannot self-install printers**: any printer needing a new driver requires an
        administrator
      - **Already the default when patched**: since the August 10, 2021 updates, the value being
        absent behaves as 1, so on a current machine this confirms rather than changes
      - **The revert does not unrestrict**: deleting the value leaves the restriction in force on a
        patched machine; only an explicit 0 loosens it

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2, all editions with the
        July 6, 2021 or later cumulative updates
      - **Takes effect**: immediately, the spooler picks it up without a reboot
      - **Reverting**: deletes the value, which on a patched machine leaves the restriction active
      - Microsoft's alternative for environments that need self-service printing is to pre-stage
        drivers or use a print management solution

      ## Recommendation
      Apply it for essentially everyone; the PrintNightmare risk is severe and this is the
      authoritative mitigation. Reconsider only if standard users must self-install printer drivers.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [KB5005010: restricting installation of new printer drivers](https://support.microsoft.com/en-us/topic/kb5005010-restricting-installation-of-new-printer-drivers-after-applying-the-july-6-2021-updates-31b91c02-05bc-4ada-a7ea-183b129578a7)
```

**Sources:**
1. KB5005010: Restricting installation of new printer drivers after applying the July 6, 2021 updates, https://support.microsoft.com/en-us/topic/kb5005010-restricting-installation-of-new-printer-drivers-after-applying-the-july-6-2021-updates-31b91c02-05bc-4ada-a7ea-183b129578a7 (tier A)

### `disable_remote_assistance` Remote Assistance

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Control\Remote Assistance` -> `fAllowToGetHelp`
(REG_DWORD). Options: "Disabled" = 0, **"Enabled (Stock Default)" = 1**. No reboot flag, correct.

CORRECTED revert: **`absent`**, relabelled "Windows default (Stock Default)". Microsoft's unattend
reference for `Microsoft-Windows-RemoteAssistance-Exe | fAllowToGetHelp` carries a Values table stating
of **false**: "Specifies that the user cannot request assistance from a friend or a support
professional. **This is the default value.**" So the tweak's "stock" option writes the *on* value.
Selecting it on a machine that never had solicited Remote Assistance enabled does not restore anything,
it opens an inbound remote-control channel that was closed. That is the third revert failure shape from
`_harmful-revert.md`.

Two qualifications, stated plainly. The cited page is the unattend reference, which describes the
default of the setting as consumed by an answer file; it is the strongest Microsoft statement available
but is not a statement about the byte in a given retail SKU's SYSTEM hive. And the on-box value was
read on the modified development machine and is deliberately **not** cited, so the clean-image value
remains an open default (see UNKNOWNS).

**Corrections needed:** Change the second option to `allow_get_help: absent` and relabel it "Windows
default (Stock Default)". That is correct regardless of how the clean-image question resolves. If a
literal is kept it must be 0, not 1, and only after confirming the shipped value on a clean 26100
install. Separately, consider also writing `fAllowUnsolicited` = 0 under the policy key, and note that
the Group Policy counterpart (confirmed in the shipped `RemoteAssistance.admx`, policy `RA_Solicit`,
`class="Machine"`, key `Software\policies\Microsoft\Windows NT\Terminal Services`, same value name,
enabled 1 / disabled 0) overrides the non-policy key this tweak writes.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off inbound Remote Assistance so nobody can be invited to control your desktop.**

      ## What it does
      Sets `fAllowToGetHelp` = 0 under `HKLM\SYSTEM\CurrentControlSet\Control\Remote Assistance`, the
      value behind the "Allow Remote Assistance connections to this computer" checkbox on the System
      Properties Remote tab. Solicited Remote Assistance sessions are then refused.

      ## Benefits
      - **Removes a social-engineering channel**: invitation-based remote control is a scam staple
      - **Largely obsolete anyway**: Quick Assist is the supported path on modern Windows
      - **Instant and reversible**: no reboot, one value

      ## Drawbacks
      - **No invitation-based help**: family or helpdesk support through classic Remote Assistance
        stops working
      - **Does not cover everything**: unsolicited ("offer") Remote Assistance is governed by
        `fAllowUnsolicited` under the Terminal Services policy key, which this does not set
      - **Group Policy overrides it**: this is the preference store, not the policy store, so a
        managed machine can have it silently defeated
      - **Probably already off**: Microsoft documents the shipped default as not allowing assistance

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value so the machine keeps whatever the image shipped; it must not
        write 1, which would switch on a feature Windows documents as off by default
      - Quick Assist is a separate Store app with its own relay service and is unaffected

      ## Recommendation
      Apply it if you never use classic Remote Assistance, which is most people. Leave it alone only
      if you genuinely rely on invitation-based remote help.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft-Windows-RemoteAssistance-Exe | fAllowToGetHelp](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-remoteassistance-exe-fallowtogethelp)
```

**Sources:**
1. Microsoft-Windows-RemoteAssistance-Exe | fAllowToGetHelp, https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-remoteassistance-exe-fallowtogethelp (tier A). The Values table states that **false** "is the default value".
2. `C:\Windows\PolicyDefinitions\RemoteAssistance.admx`, policy `RA_Solicit` (tier A, shipped ADMX)
3. `docs/superpowers/research/validation/_harmful-revert.md` (revert failure shape 3)

### `disable_windows_script_host` Windows Script Host

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** `HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings` -> `Enabled`
(REG_DWORD). Options: "Disabled" = 0, "Windows default (Stock Default)" = `absent`. No reboot flag,
correct.

Microsoft's scripting documentation states that to disable WSH you create a REG_DWORD `Enabled` set to
0, under `HKCU\Software\Microsoft\Windows Script Host\Settings` for one user or the HKLM equivalent for
all users. With it off, running any WSH script fails with "Windows Script Host access is disabled on
this machine".

CORRECTED mechanism, add a second effect:

| Key | Value name | Type | Disabled | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings` | `Enabled` | REG_DWORD | `0` | `absent` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows Script Host\Settings` | `Enabled` | REG_DWORD | `0` | `absent` |

The two views are **separate physical keys**, proven on 26100.4061 by differing key last-write
timestamps (native 2025-06-29T14:20:05Z, WOW64 2025-05-15T19:37:17Z); a shared or reflected key would
report one identical timestamp. `C:\Windows\SysWOW64\wscript.exe` and `cscript.exe` are 32-bit and
therefore resolve `HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings` to the WOW6432Node copy.

**Corrections needed:** Add the `Wow6432Node` companion effect. Without it the tweak blocks only the
64-bit hosts and `%SystemRoot%\SysWOW64\wscript.exe payload.vbs` still runs, which is a trivially
discoverable bypass for exactly the dropper class the tweak exists to stop, and it makes the
"did-it-work" contract report success on a half-applied hardening. The corpus already handles this
correctly in `dotnet_strong_crypto`. Recorded in UNKNOWNS: on 26100.4061 `Enabled` is present as
REG_DWORD 1 in both views, contradicting "the value does not exist on a stock install"; re-read both
views on a clean image and change the Stock Default to `1` if that holds. Behaviourally absent and 1
are identical for WSH, so the revert is not broken either way.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks `.vbs`, `.js` and `.wsf` scripts from running at all.**

      ## What it does
      Sets the Windows Script Host `Enabled` value to 0 in **both** the 64-bit and the
      `Wow6432Node` views of `HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings`. Any attempt to
      run a WSH script, including through `cscript.exe` or from a batch file, then fails with
      "Windows Script Host access is disabled on this machine".

      ## Benefits
      - **Kills a top dropper format**: VBScript and JScript files remain a common initial-access
        payload
      - **Covers 32-bit too**: the WOW64 view is a separate key, and without it
        `SysWOW64\wscript.exe` still runs
      - **Almost nothing modern needs it**: VBScript is deprecated and moving to an on-demand feature

      ## Drawbacks
      - **Legacy automation stops**: `.vbs` and `.js` logon scripts, MSI custom actions implemented
        in VBScript and older installers break, sometimes with obscure errors
      - **Per-user override exists**: HKCU wins for that user, so a user (or malware running as that
        user) can re-enable WSH for themselves unless HKCU is locked down too
      - **Hard to attribute failures**: an installer that suddenly fails may give no hint that WSH is
        the cause

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2, all editions
      - **Takes effect**: immediately, no reboot
      - **Reverting**: removes both values, restoring the shipped state
      - If a needed installer or script starts failing, this tweak is the first thing to check

      ## Recommendation
      High value on typical home and office PCs that never run legitimate WSH scripts. Avoid it if
      your workflow or deployment depends on VBScript or JScript automation.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Running your scripts (Windows Script Host)](https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/windows-scripting/xazzc41b(v=vs.84))
      - [Registry redirector](https://learn.microsoft.com/en-us/windows/win32/winprog64/registry-redirector)
```

**Sources:**
1. Running Your Scripts (Windows Script Host), https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/windows-scripting/xazzc41b(v=vs.84) (tier A, archived Microsoft documentation)
2. Registry Redirector, https://learn.microsoft.com/en-us/windows/win32/winprog64/registry-redirector (tier A). `HKLM\SOFTWARE` is redirected for 32-bit callers and `Microsoft\Windows Script Host` is not on the shared-key list.
3. On-box key enumeration and `RegQueryInfoKey` last-write timestamps for both registry views on 26100.4061 (corroborating only; establishes that the two views are physically separate, not either view's default)

### `enable_controlled_folder_access` Controlled Folder Access (ransomware shield)

**Verdict:** VERIFIED

**Mechanism:**
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\Controlled Folder Access`
-> `EnableControlledFolderAccess` (REG_DWORD). Options: "Enabled" = 1, "Windows default (Stock
Default)" = `absent`. No reboot flag, correct.

Documented modes: **0 Disabled (default)**, 1 Enabled (block), 2 Audit, 3 Block disk modification only,
4 Audit disk modification only. Confirmed against the shipped ADMX rather than prose:
`C:\Windows\PolicyDefinitions\WindowsDefender.admx` on 26100.4061 defines policy
`ExploitGuard_ControlledFolderAccess_EnableControlledFolderAccess` with
`key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\Controlled Folder Access"`
and `valueName="EnableControlledFolderAccess"`, an exact match including all five modes.

The Tamper Protection concern is **refuted**: Microsoft's enumeration of tamper-protected settings
covers virus and threat protection, real-time protection, behaviour monitoring, IOAV, cloud protection,
security intelligence updates, automatic actions, notifications, archive scanning and exclusions.
Controlled Folder Access is not on that list, so Tamper Protection being on by default on consumer 24H2
does not make this tweak inert. It helps, by keeping the real-time protection prerequisite on.

**Corrections needed:** `none`. Consider exposing mode 2 (Audit) as an option, since that is
Microsoft's recommended first step.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks unknown apps from writing to Documents, Pictures and your other protected folders.**

      ## What it does
      Sets `EnableControlledFolderAccess` = 1 in the Defender Exploit Guard policy key. Defender then
      blocks untrusted processes from writing to a fixed list of folders (Documents, Pictures,
      Videos, Music, Desktop, Favorites and the public equivalents) plus any you add.

      ## Benefits
      - **Targets ransomware directly**: it is the only built-in Windows control that blocks mass
        file modification by an unrecognised process
      - **Protects data even if malware runs**: the block happens at the file-write layer
      - **Extendable**: you can add your own folders and approve your own apps

      ## Drawbacks
      - **False positives are common at first**: game saves, photo and video editors, backup agents
        and portable apps get blocked until you allow-list them
      - **Needs Defender in active mode**: it does nothing if a third-party antivirus has taken over
      - **Managed machines override it**: Intune or Configuration Manager policies overwrite
        conflicting Group Policy at startup
      - **The probe reads policy, not state**: the runtime state lives under a different key without
        `Policies`, so a successful write does not prove the feature engaged

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 and Server 2019 or later
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, restoring the shipped Disabled state
      - Microsoft's recommended rollout is Audit mode (value 2) first, so you can see what would be
        blocked before enforcing

      ## Recommendation
      Worth it if you want ransomware protection and are willing to allow-list apps when something is
      blocked. If managing exceptions will frustrate you, leave it off rather than turning it off
      halfway through a bad week.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Configure controlled folder access](https://learn.microsoft.com/en-us/defender-endpoint/controlled-folder-access-configure)
      - [Protect security settings with tamper protection](https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection)
```

**Sources:**
1. Configure controlled folder access, https://learn.microsoft.com/en-us/defender-endpoint/controlled-folder-access-configure (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100.4061 (tier A, shipped ADMX)
3. Protect security settings with tamper protection, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)

### `enable_network_protection` Defender Network Protection

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\Network Protection`
-> `EnableNetworkProtection` (REG_DWORD). Options: "Enabled" = 1, "Windows default (Stock Default)" =
`absent`. No reboot flag, correct.

Microsoft documents the value and its states directly: **0 Off, 1 On, 2 Audit**. Confirmed against the
shipped ADMX: `WindowsDefender.admx` on 26100.4061 defines `ExploitGuard_EnableNetworkProtection` with
`key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\Network Protection"`,
`valueName="EnableNetworkProtection"` and the three modes, matching the YAML exactly. Microsoft's stated
default is Disabled, so `absent` is a correct revert and it is safe against all three revert failure
shapes. The Tamper Protection concern is refuted for the same reason as Controlled Folder Access.

CORRECTED applicability: Microsoft's *Requirements for network protection* names the supported client
operating systems as "Windows 10 or 11 (**Pro or Enterprise**)". Home is not on that list. The same
section requires Microsoft Defender Antivirus with **real-time protection, behavior monitoring, and
cloud-delivered protection** all enabled and active, and the overview adds that Defender must be in
**active** mode.

**Corrections needed:** (a) Correct the applicability claim: gate the tweak away from Home, or state in
the info text that it does nothing there. (b) Add the two missing prerequisites, behavior monitoring
and Defender being in active rather than passive mode. This matters in practice because the tweak is a
silent no-op in exactly the two situations the current info text does not warn about: a Home machine,
and a machine running a third-party antivirus. In both, the policy write succeeds, the "did-it-work"
contract reports success, and network protection never engages. (c) Consider exposing Audit mode (2).

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks every app on the PC, not just Edge, from reaching known-malicious sites and IPs.**

      ## What it does
      Sets `EnableNetworkProtection` = 1 in the Defender Exploit Guard policy key. Defender then
      extends SmartScreen's URL and IP reputation checks to every process on the machine and blocks
      outbound connections to destinations with a bad reputation.

      ## Benefits
      - **Covers all applications**: command-and-control traffic from any process is cut off, not
        just browser navigation
      - **Cheap and broad**: no per-app configuration
      - **Enables enforcement features**: block mode is what makes custom IP and URL indicators and
        Web Content Filtering enforceable

      ## Drawbacks
      - **Pro and Enterprise only**: Microsoft does not list Windows Home as supported, so the write
        succeeds and nothing happens there
      - **Needs Defender in active mode with three features on**: real-time protection, behavior
        monitoring **and** cloud-delivered protection must all be enabled; with a third-party
        antivirus installed Defender goes passive and this is inert
      - **Reputation lookups go to Microsoft**: if you turned cloud protection off for privacy, this
        cannot function
      - **Occasional false positives**: niche or newly registered domains get blocked

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, **Pro / Enterprise / Education only**
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, restoring the shipped Disabled state
      - Audit mode (value 2) only logs; block mode is what enforces
      - Intune or Configuration Manager settings overwrite conflicting Group Policy at startup

      ## Recommendation
      Recommended if you run Defender as your active antivirus on Pro or better; it is a strong,
      low-friction layer. Skip it on Home or with a third-party antivirus, where it does nothing.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Turn on network protection](https://learn.microsoft.com/en-us/defender-endpoint/enable-network-protection)
      - [Use network protection to help prevent connections to malicious sites](https://learn.microsoft.com/en-us/defender-endpoint/network-protection)
```

**Sources:**
1. Turn on network protection, https://learn.microsoft.com/en-us/defender-endpoint/enable-network-protection (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100.4061 (tier A, shipped ADMX)
3. Use network protection to help prevent connections to malicious or suspicious sites, "Requirements for network protection", https://learn.microsoft.com/en-us/defender-endpoint/network-protection (tier A)
4. Protect security settings with tamper protection, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)

### `enable_pua_protection` PUA/PUP protection

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender` -> `PUAProtection` (REG_DWORD).
Options: "Enabled" = 1, "Windows default (Stock Default)" = `absent`. No reboot flag, correct.

Microsoft documents the numeric states: **0** off, **1** on with detected items blocked, **2** audit
mode. The Group Policy setting is *Configure detection for potentially unwanted applications*.

CORRECTED default: with security intelligence version **1.329.495.0 or later**, PUA protection defaults
to **Audit mode (2)** on Windows 10 and later for devices not onboarded to Defender for Endpoint, and
to Block (1) on onboarded devices. The info text's implication that PUA blocking is off by default is
outdated.

**Corrections needed:** (a) Correct the "off by default" claim to "audit by default". (b) The registry
value name needs an on-box confirmation. Microsoft's PUA article documents only the Group Policy
setting and `Set-MpPreference -PUAProtection`; the historically documented registry route was
`MpEnablePus` under `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine`, while the current
GP-backed value is `PUAProtection` directly under `...\Windows Defender`. The YAML uses the latter,
which matches the modern ADMX, but this was not confirmable from a tier A prose page. Check with
`Get-MpPreference | Format-Table PUAProtection` after applying, since a value the engine does not read
fails silently. Recorded in UNKNOWNS.

**Ready-to-paste info block:**

```yaml
    info: |
      **Lets Defender block bundleware, adware and aggressive "optimizer" tools outright.**

      ## What it does
      Sets `PUAProtection` = 1 (Block) under `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender`.
      Defender then quarantines potentially unwanted applications: bundleware, adware, aggressive
      toolbars and evasive installers that are not outright malware and so are not covered by normal
      signatures.

      ## Benefits
      - **Covers a real gap**: regular antivirus signatures deliberately do not cover this category
      - **Blocks rather than logs**: value 1 quarantines, where the current Windows default only
        audits
      - **Keeps the machine cleaner**: stops the junk that rides along with free installers

      ## Drawbacks
      - **Occasional false positives**: some system utilities and cracking-adjacent tooling are
        classified as PUA, and you will need exclusions
      - **Classification can change**: a tool that works today can be quarantined after a signature
        update, because the classification is behavioural
      - **Smaller delta than it sounds**: modern Defender already defaults to Audit mode (2), so this
        changes logging into blocking rather than turning something on from nothing
      - **Needs Defender active**: inert if a third-party antivirus has taken over

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 8.1 and Windows 10
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, returning to the current Defender default of Audit mode
      - Confirm it took with `Get-MpPreference | Format-Table PUAProtection` after applying

      ## Recommendation
      Enable it for most people; the cleanliness benefit outweighs the rare false positive. If you
      routinely run niche utilities Defender dislikes, be ready to add exclusions.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Block potentially unwanted applications with Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/detect-block-potentially-unwanted-apps-microsoft-defender-antivirus)
```

**Sources:**
1. Block potentially unwanted applications with Microsoft Defender Antivirus, https://learn.microsoft.com/en-us/defender-endpoint/detect-block-potentially-unwanted-apps-microsoft-defender-antivirus (tier A)

### `asr_block_lsass_theft` ASR rule: block LSASS credential theft

**Verdict:** **INCORRECT**

**Mechanism as authored (defective):**
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules` -> value
name **`9e6c4e1f-7d60-472f-ba1a-a39ef669e4b0`** (REG_SZ), data "1". Revert: `absent`. No reboot flag,
correct.

CORRECTED mechanism:

| Key | Value name | Type | Block | Stock Default |
|---|---|---|---|---|
| `...\Windows Defender Exploit Guard\ASR\Rules` | **`9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2`** | REG_SZ | `"1"` | `absent` |
| `...\Windows Defender Exploit Guard\ASR` | `ExploitGuard_ASR_Rules` | REG_DWORD | `1` | `absent` |

The wrong one currently shipped, shown for comparison: value name
`9e6c4e1f-7d60-472f-ba1a-a39ef669e4b0`, final character `0` instead of `2`.

Both current Microsoft Learn pages (the ASR rules overview and the ASR rules reference, revised
2 July 2026) give `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2` for "Block credential stealing from the
Windows local security authority subsystem". The `...e4b0` form appears in older material and in many
community scripts. **Defender ignores an unrecognised GUID silently**, so the tweak as shipped reports
success and does nothing. The gap-verification pass reached the same conclusion independently: "As
shipped, `asr_block_lsass_theft` writes a value name that maps to no rule and therefore does nothing."

Group Policy value semantics for the rule data: 0 Off, 1 Block, 2 Audit, 5 Not configured, 6 Warn. This
rule does **not** support Warn mode.

**Corrections needed:** (1) Change the value name to `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2` and verify
on a real machine with `(Get-MpPreference).AttackSurfaceReductionRules_Ids` after applying. (2) Add the
ADMX parent enabling value `ExploitGuard_ASR_Rules` = 1 (REG_DWORD) at
`...\Windows Defender Exploit Guard\ASR`, `absent` on revert; without it the policy reads Not Configured
in `gpedit.msc` and a Group Policy refresh can strip the orphaned rule value. (3) The REG_SZ typing is
correct in practice (ADMX list elements with `explicitValue="true"` write REG_SZ) and is confirmed by
the shipped `WindowsDefender.admx`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks Mimikatz-style credential theft from LSASS memory.**

      ## What it does
      Enables the Defender attack surface reduction rule
      `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2` ("Block credential stealing from the Windows local
      security authority subsystem") in Block mode. Defender then denies process handle-open and
      memory-read operations against `lsass.exe` from unexpected callers.

      ## Benefits
      - **Mimikatz-class protection without VBS**: useful on machines that cannot run LSA protection
        or Credential Guard
      - **Safe to deploy directly**: Microsoft classes it as a standard protection rule, no audit
        period needed
      - **Instantly reversible**: removing the value turns it off with no reboot

      ## Drawbacks
      - **Very noisy in audit**: Chrome's updater and many enumerating tools trip it, though Microsoft
        says almost all of it is safe to ignore
      - **Limited exclusions**: the rule does not honour Defender file and folder exclusions
      - **Redundant with LSA protection**: Microsoft states the rule "isn't required" and "doesn't
        provide extra protection" when LSA protection is enabled, and marks it not applicable in
        Defender for Endpoint in that case
      - **Known incompatibility**: Quest DirSync Password Sync

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1803 and later. ASR rules need
        Microsoft Defender Antivirus in active mode with real-time protection on
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the rule value and the policy's enabling value
      - The rule blocks access to LSASS memory, not process execution, so a blocked `svchost.exe` in
        the log is usually benign
      - Group Policy configuration needs Pro or better; on Home the rule can only be set with
        PowerShell

      ## Recommendation
      A good extra layer on Defender-protected machines, especially where LSA protection is not
      enabled. If LSA protection or Credential Guard is already on, skip it rather than inheriting a
      second set of compatibility problems.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [ASR rules overview (GUID table)](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview)
      - [ASR rules reference](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference)
      - [Configure ASR rules and exclusions](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-configure)
```

**Sources:**
1. ASR rules overview (GUID table), https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview (tier A)
2. ASR rules reference, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
3. Configure ASR rules and exclusions, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-configure (tier A)
4. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policy `ExploitGuard_ASR_Rules` (tier A, shipped ADMX)

### `asr_block_office_script_vectors` ASR rules: block Office/script malware vectors

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Four REG_SZ values under
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`, each set
to "1" (Block). Revert: all `absent`. No reboot flag, correct.

| Value name (GUID) | Microsoft rule name |
|---|---|
| `d4f940ab-401b-4efc-aadc-ad5f3c50688a` | Block all Office applications from creating child processes |
| `3b576869-a4ec-4529-8536-b80a7769e899` | Block Office applications from creating executable content |
| `5beb7efe-fd9a-4556-801d-275e5ffc04cc` | Block execution of potentially obfuscated scripts |
| `be9ba2d9-53ea-4cdc-84e5-9b1eeee46550` | Block executable content from email client and webmail |

All four GUIDs, the REG_SZ typing and the "1" = Block semantics are re-confirmed against the current
Microsoft ASR rules table.

CORRECTED mechanism, add the ADMX parent enabling value:

| Key | Value name | Type | Block | Stock Default |
|---|---|---|---|---|
| `...\Windows Defender Exploit Guard\ASR` | `ExploitGuard_ASR_Rules` | REG_DWORD | `1` | `absent` |

`WindowsDefender.admx` on 26100.4061 defines `<policy name="ExploitGuard_ASR_Rules" class="Machine"
key="Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR"
valueName="ExploitGuard_ASR_Rules">` whose `<list>` element is the `ASR\Rules` subkey the YAML writes.
The YAML writes the list half only, so the policy renders as **Not Configured** in `gpedit.msc` even
with the GUID values present, and because the Group Policy engine treats values under a policy key it
owns as its own to manage, a later refresh can remove the orphaned `Rules` values. Tamper Protection is
not the culprit here: ASR rules are absent from Microsoft's list of tamper-protected settings.

**Corrections needed:** (1) Add the `ExploitGuard_ASR_Rules` = 1 companion, `absent` on revert. (2)
Extend the `warning` to name cloud-delivered protection, not just real-time protection: the
obfuscated-scripts rule `5beb7efe-fd9a-4556-801d-275e5ffc04cc` requires cloud-delivered protection plus
AMSI, and `be9ba2d9-53ea-4cdc-84e5-9b1eeee46550` produces user notifications only at cloud protection
level High or above. On a machine where cloud protection was turned off for privacy, which
`privacy.yaml` actively encourages, one of the four rules silently does nothing. (3) Consider offering
Audit mode ("2"), which the info text already recommends but the tweak does not expose.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks the Office macro, script and email-attachment tricks behind most phishing attacks.**

      ## What it does
      Enables four Defender attack surface reduction rules in Block mode: Office apps creating child
      processes, Office creating executable content, execution of potentially obfuscated scripts, and
      executable content arriving from email or webmail. Each GUID is written as a `REG_SZ` value of
      "1" under the Exploit Guard ASR rules key, alongside the policy's own enabling value.

      ## Benefits
      - **Covers the dominant initial-access chains**: macro spawning `cmd`, `powershell` or `mshta`,
        macro dropping a payload, obfuscated script execution, and email-borne executables
      - **Stops attacks at stage one**: before any payload runs
      - **Reversible instantly**: no reboot in either direction

      ## Drawbacks
      - **Legitimate macros break**: macro workbooks that shell out, and admin scripts that look
        obfuscated, are blocked
      - **Email rule blocks archives**: `.zip` files containing executables are caught too
      - **Two prerequisites, not one**: all four need Defender real-time protection, and the
        obfuscated-scripts rule additionally needs **cloud-delivered protection** and AMSI, so it is
        silently inert if you turned cloud protection off
      - **Uneven exclusions**: the Office executable-content rule has limited exclusion support and
        does not honour Defender file and folder exclusions

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1709 and later
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all four rule values and the policy's enabling value
      - The two Office child-process rules are enforced only when Office is installed under
        `%ProgramFiles%` or `%ProgramFiles(x86)%`
      - Microsoft classifies all four as "other ASR rules", meaning audit-mode testing before block
      - Intune and Configuration Manager overwrite conflicting Group Policy at startup

      ## Recommendation
      High value on typical home and office PCs, where these behaviours are almost always malicious.
      If your work depends on Office macros or scripted automation, audit first and add exceptions
      before enforcing.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [ASR rules overview (GUID table)](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview)
      - [ASR rules reference (per-rule dependencies)](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference)
      - [Configure ASR rules in Group Policy](https://learn.microsoft.com/en-us/defender-endpoint/enable-attack-surface-reduction)
```

**Sources:**
1. ASR rules overview (GUID table), https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview (tier A)
2. ASR rules reference (per-rule details and dependencies), https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
3. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100.4061, policy `ExploitGuard_ASR_Rules` (tier A, shipped ADMX)
4. Configure ASR rules and exclusions in group policy, https://learn.microsoft.com/en-us/defender-endpoint/enable-attack-surface-reduction (tier A)
5. Protect security settings with tamper protection, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)

### `enforce_smartscreen` Enforce SmartScreen (apps and Edge)

**Verdict:** VERIFIED

**Mechanism:** Three values.

| Key | Value name | Type | Enforced | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `EnableSmartScreen` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `ShellSmartScreenLevel` | REG_SZ | `"Block"` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Edge` | `SmartScreenEnabled` | REG_DWORD | `1` | `absent` |

No reboot flag, correct.

The REG_SZ typing question is **resolved** from the shipped `C:\Windows\PolicyDefinitions\SmartScreen.admx`
on 26100.4061: policy `ShellConfigureSmartScreen`, `key="Software\Policies\Microsoft\Windows\System"`,
`valueName="EnableSmartScreen"` with enabled 1 / disabled 0, and an `<enum>` element on the same key
with `valueName="ShellSmartScreenLevel"` whose two items are `<string>Block</string>` and
`<string>Warn</string>`. A `<string>` element in an ADMX is written as REG_SZ, so the YAML's REG_SZ
"Block" is confirmed rather than inferred. Microsoft's SmartScreen Policy CSP gives
`EnableSmartScreenInShell` a default of 1, so SmartScreen is on out of the box; the value this tweak
adds is "Block" instead of "Warn".

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns SmartScreen from a warning you can click through into a block you cannot.**

      ## What it does
      Sets `EnableSmartScreen` = 1 and `ShellSmartScreenLevel` = "Block" under the Windows System
      policy key, plus Edge's `SmartScreenEnabled` = 1. SmartScreen already checks the reputation of
      files you download and sites you visit; "Block" removes the "Run anyway" escape hatch.

      ## Benefits
      - **Removes the reflex click-through**: the real value here is Block rather than Warn
      - **Covers the shell and the browser**: app and file reputation plus Edge browsing
      - **Well-proven**: SmartScreen is a mature, low-false-positive reputation service

      ## Drawbacks
      - **Rare software is blocked outright**: in-house tools, niche utilities and self-compiled
        binaries have no reputation and there is no user-visible way past
      - **Reputation data goes to Microsoft**: file hashes and URLs are sent for lookup
      - **Can make a machine feel locked down**: combined with PUA protection and network protection
        the machine becomes noticeably restrictive
      - **Needs an administrator to unblock**: a standard user genuinely cannot run an unrecognised
        installer

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1703 and later, Pro and above for
        the policy. The Edge value requires Microsoft Edge (Chromium)
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all three values, returning SmartScreen to the shipped Warn behaviour
      - SmartScreen is already on by default; what this changes is Warn to Block

      ## Recommendation
      Recommended for most users; the downsides are minor and the default is already this feature
      turned on. Skip Block mode if you regularly run unsigned or low-prevalence software.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [SmartScreen Policy CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-smartscreen)
      - [Microsoft Edge browser policies, SmartScreenEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#smartscreenenabled)
```

**Sources:**
1. SmartScreen Policy CSP, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-smartscreen (tier A)
2. Microsoft Edge browser policies, SmartScreenEnabled, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#smartscreenenabled (tier A)
3. `C:\Windows\PolicyDefinitions\SmartScreen.admx` on 26100.4061, policy `ShellConfigureSmartScreen` (tier A, shipped ADMX; confirms the REG_SZ enum values "Block" and "Warn")

### `disable_tls_legacy` Disable legacy TLS 1.0/1.1 (Schannel)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Four REG_DWORD `Enabled` values set to 0 under
`HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\SCHANNEL\Protocols\TLS 1.0\{Client,Server}`
and `...\TLS 1.1\{Client,Server}`. Revert: all `absent`. `requires_reboot: true`.

Microsoft documents the path format (`<SSL/TLS/DTLS> <major>.<minor>\<Client|Server>`) and states: "In
order to override a system default and set a supported (D)TLS or SSL protocol version to the `Disabled`
state, change the DWORD registry value of `Enabled` to 0 under the corresponding version-specific
subkey." The subkeys do not exist by default and must be created, so `absent` is the correct revert.

CORRECTED mechanism, add four companion values:

| Key | Value name | Type | Disabled | Stock Default |
|---|---|---|---|---|
| `...\Protocols\TLS 1.0\Client` | `DisabledByDefault` | REG_DWORD | `1` | `absent` |
| `...\Protocols\TLS 1.0\Server` | `DisabledByDefault` | REG_DWORD | `1` | `absent` |
| `...\Protocols\TLS 1.1\Client` | `DisabledByDefault` | REG_DWORD | `1` | `absent` |
| `...\Protocols\TLS 1.1\Server` | `DisabledByDefault` | REG_DWORD | `1` | `absent` |

**Corrections needed:** Add the four `DisabledByDefault` = 1 values. `Enabled` = 0 alone does force the
Disabled state per current documentation, but every Microsoft hardening example and every CIS and DISA
STIG check writes both, so the half-configuration will fail compliance checks and differs from every
reference implementation.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off TLS 1.0 and 1.1 system-wide so everything on the PC negotiates TLS 1.2 or 1.3.**

      ## What it does
      Writes `Enabled` = 0 and `DisabledByDefault` = 1 in the Schannel Client and Server subkeys for
      TLS 1.0 and TLS 1.1. Schannel is the Windows TLS stack, so this covers every application that
      uses it rather than just browsers.

      ## Benefits
      - **Removes broken protocols**: TLS 1.0 and 1.1 have known cryptographic weaknesses
      - **Closes downgrade paths**: an attacker cannot force a connection onto a breakable version
      - **Covers services too**: applies to background services that ignore browser settings

      ## Drawbacks
      - **Legacy endpoints become unreachable**: internal appliances, printer web UIs and old servers
        that only offer TLS 1.0 or 1.1 stop working over HTTPS from every Schannel client
      - **Can fail hard**: Microsoft warns that reducing the enabled set can make
        `AcquireCredentialsHandle` fail outright if the caller's allowed set becomes empty
      - **Increasingly a no-op**: TLS 1.0 and 1.1 have been off by default in Windows 11 since 22H2
      - **Does not cover other stacks**: OpenSSL, Java and NSS-based clients have their own settings

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after reboot (Microsoft says the change applies to credential handles opened
        by subsequent calls, so an application or service restart can be enough; a reboot is a safe
        superset)
      - **Reverting**: deletes all eight values, since the subkeys do not exist on a stock machine
      - Microsoft warns against creating Schannel settings that are not explicitly documented

      ## Recommendation
      Apply it on modern systems; almost everything uses TLS 1.2 or 1.3 already. Hold off only if you
      must reach legacy internal servers stuck on the old protocols.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Transport Layer Security (TLS) registry settings](https://learn.microsoft.com/en-us/windows-server/security/tls/tls-registry-settings)
```

**Sources:**
1. Transport Layer Security (TLS) registry settings, https://learn.microsoft.com/en-us/windows-server/security/tls/tls-registry-settings (tier A)

### `dotnet_strong_crypto` Force .NET strong crypto (TLS 1.2+)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** `HKLM\SOFTWARE\Microsoft\.NETFramework\v4.0.30319` and
`HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework\v4.0.30319` -> `SchUseStrongCrypto` (REG_DWORD) = 1.
Revert: both `absent`. No reboot flag, correct: it affects newly started processes.

Microsoft documents this exactly: "The `HKEY_LOCAL_MACHINE\SOFTWARE\[Wow6432Node\]Microsoft\.NETFramework\<VERSION>:
SchUseStrongCrypto` registry entry has a value of type DWORD. A value of 1 causes your app to use strong
cryptography." It makes .NET Framework pass `SCH_USE_STRONG_CRYPTO` to Schannel. Microsoft: "These
registry keys don't exist by default. You must add them manually", so `absent` is correct. Both keys
were confirmed to lack `SchUseStrongCrypto` on a 26100.4061 install (only `AspNetEnforceViewStateMac` is
present in each).

CORRECTED mechanism, add the companion from Microsoft's own reference `.reg`:

| Key | Value name | Type | Enabled | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework\v4.0.30319` | `SystemDefaultTlsVersions` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework\v4.0.30319` | `SystemDefaultTlsVersions` | REG_DWORD | `1` | `absent` |

**Corrections needed:** Add the two `SystemDefaultTlsVersions` values. Microsoft's reference file
additionally covers `v2.0.50727` in both registry views for .NET Framework 3.5 apps; add it if 3.5
support matters, or state in the info text that .NET 3.5 applications are deliberately out of scope.
Worth stating plainly for the 24H2 floor: build 26100.4061 ships .NET Framework 4.8.09032 (Release
533320), `SchUseStrongCrypto` already defaults to 1 for apps targeting 4.6 or later, and
`SystemDefaultTlsVersions` already defaults to 1 for apps targeting 4.7 or later. On the primary target
the tweak therefore reaches only legacy binaries targeting 4.5.2 or earlier. That is a genuine and
shrinking benefit, not a defect, and the info text must not imply the machine is insecure without it.

**Ready-to-paste info block:**

```yaml
    info: |
      **Pushes legacy .NET applications onto TLS 1.2 and later instead of SSL 3.0 or TLS 1.0.**

      ## What it does
      Sets `SchUseStrongCrypto` = 1 and `SystemDefaultTlsVersions` = 1 for .NET Framework 4 and later
      in both the 64-bit and 32-bit registry views. .NET then passes `SCH_USE_STRONG_CRYPTO` to
      Schannel, which disables known-weak algorithms, cipher suites and protocol versions for
      outgoing connections.

      ## Benefits
      - **Fixes broken connections too**: legacy .NET apps that fail against servers which dropped
        old protocols start working again
      - **Improves security and compatibility at once**: rare for a hardening setting
      - **Both registry views covered**: 32-bit .NET apps read the `Wow6432Node` copy

      ## Drawbacks
      - **Rare legacy break**: an old in-house .NET app talking to a weak-crypto-only endpoint can
        stop connecting; Microsoft says the value should only be 0 "if you need to connect to legacy
        services that don't support strong cryptography and can't be upgraded"
      - **Little effect on current builds**: 24H2 ships .NET Framework 4.8, where both values already
        default to 1 for apps targeting 4.6 (or 4.7) and later, so this only reaches binaries
        targeting 4.5.2 or earlier
      - **Needs app restarts**: the setting applies to newly started processes
      - **.NET 3.5 not covered**: Microsoft's reference file also sets `v2.0.50727`, which this does
        not

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer with .NET Framework installed; also Windows 10 22H2
      - **Takes effect**: immediately for newly started processes, no reboot
      - **Reverting**: deletes the values, which do not exist on a stock machine
      - Client (outgoing) connections only

      ## Recommendation
      Apply it. It improves both security and compatibility for older .NET software at negligible
      cost, though on a current 24H2 machine most applications already behave this way.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Transport Layer Security (TLS) best practices with .NET Framework](https://learn.microsoft.com/en-us/dotnet/framework/network-programming/tls)
```

**Sources:**
1. Transport Layer Security (TLS) best practices with .NET Framework, https://learn.microsoft.com/en-us/dotnet/framework/network-programming/tls (tier A)
2. On-box state of both `.NETFramework\v4.0.30319` registry views and the installed .NET Framework release on build 26100.4061 (corroborating)

### `disable_smb_guest` Disable SMB insecure guest logons

**Verdict:** VERIFIED

**Mechanism:** `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation` -> `AllowInsecureGuestAuth`
(REG_DWORD) = 0. Revert: `absent`. No reboot flag, correct.

Microsoft's LanmanWorkstation Policy CSP gives the mapping exactly: policy
`Pol_EnableInsecureGuestLogons`, Registry Key Name `Software\Policies\Microsoft\Windows\LanmanWorkstation`,
Registry Value Name `AllowInsecureGuestAuth`, ADMX `LanmanWorkstation.admx`. Disabled (0) means "the SMB
client will reject insecure guest logons". `_policy-hive-audit.md` confirms the class is Machine and the
HKLM write is correct. The non-policy twin lives at
`HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters\AllowInsecureGuestAuth`.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the SMB client silently connecting to shares as an anonymous guest.**

      ## What it does
      Sets `AllowInsecureGuestAuth` = 0 in the LanmanWorkstation policy key. The SMB client then
      refuses to fall back to guest authentication when a share offers it, which also means a rogue
      server can no longer lure this PC into an unauthenticated, unsigned, unencrypted session.

      ## Benefits
      - **Blocks a rogue-server setup**: guest fallback is the entry point for SMB
        adversary-in-the-middle
      - **Guest sessions cannot be protected**: Microsoft notes guest logons cannot use SMB signing
        or encryption at all
      - **Forces real credentials**: shares must be accessed with an account

      ## Drawbacks
      - **Guest-only NAS stops working**: cheap home NAS boxes and router USB shares fail with "your
        organization's security policies block unauthenticated guest access" until you configure a
        real account on them
      - **Often already the case**: guest credentials are already rejected on Windows 10 Enterprise,
        Pro for Workstations and Education, on Windows 11 Pro from build 25267 onward, and on 24H2
        where required SMB signing makes guest authentication fail anyway
      - **Overlaps with SMB signing**: applying both produces the same NAS breakage twice

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1803 and later. Windows 10 Home
        and Pro still allow guest authentication by default
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, returning to the edition's own default
      - The non-policy twin under `Services\LanmanWorkstation\Parameters` is a separate store; this
        tweak writes the policy one, which wins

      ## Recommendation
      Apply it and set proper credentials on your shares. Leave it off only if you depend on a device
      that can only offer guest access and cannot be given a real account.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [LanmanWorkstation Policy CSP, EnableInsecureGuestLogons](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-lanmanworkstation)
      - [Enable insecure guest logons in SMB2 and SMB3](https://learn.microsoft.com/en-us/windows-server/storage/file-server/enable-insecure-guest-logons-smb2-and-smb3)
```

**Sources:**
1. LanmanWorkstation Policy CSP, EnableInsecureGuestLogons, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-lanmanworkstation (tier A)
2. Enable insecure guest logons in SMB2 and SMB3, https://learn.microsoft.com/en-us/windows-server/storage/file-server/enable-insecure-guest-logons-smb2-and-smb3 (tier A)
3. `docs/superpowers/research/validation/_policy-hive-audit.md` (class Machine, HKLM correct)

### `remove_powershell_v2` Remove PowerShell 2.0 engine

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Two effects.

- Registry state marker: `HKCU\Software\MagicXToolbox\State` -> `PowerShellV2` (REG_DWORD).
- Action (PowerShell): apply `Disable-WindowsOptionalFeature -Online -FeatureName
  MicrosoftWindowsPowerShellV2Root -NoRestart -ErrorAction Stop`; undo the `Enable-` equivalent with
  `-All`; probe `(Get-WindowsOptionalFeature -Online -FeatureName MicrosoftWindowsPowerShellV2Root).State
  -eq 'Disabled'`.

Options: "Removed" (`state: 1`, `ps2_feature: run`), "Present (Stock Default)" (`state: 0` **only**).
`requires_reboot: true`, correct.

`MicrosoftWindowsPowerShellV2Root` is the parent optional feature ("Windows PowerShell 2.0");
`MicrosoftWindowsPowerShellV2` is the child ("Windows PowerShell 2.0 Engine"). Disabling the parent
disables both. Removing the v2 engine prevents `powershell.exe -Version 2`, the standard way to run
script without AMSI, script-block logging or module logging.

**Corrections needed:** (a) "Present (Stock Default)" sets only `state: 0` and omits `ps2_feature`, so
the `undo` action never runs and the feature stays removed after a revert. (b) Move the state marker out
of `HKCU\Software\MagicXToolbox\State`: the change is machine-wide and applied through an elevated
broker, so the marker can be written to a different user hive than the one the UI reads. (c) Add an
applicability gate or a graceful path for builds where the feature no longer exists: DISA's Windows 11
STIG marks the requirement Not Applicable from 24H2 onward, and Microsoft has begun removing
PowerShell 2.0 from Insider builds (27891 Canary and later), where `-ErrorAction Stop` will make the
call fail rather than no-op. (d) Consider probing `MicrosoftWindowsPowerShellV2` as well; DISA's check
requires both to report Disabled, and a partial state after a failed servicing operation would make the
current probe report success incorrectly.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the obsolete PowerShell 2.0 engine that malware uses to dodge logging and AMSI.**

      ## What it does
      Disables the `MicrosoftWindowsPowerShellV2Root` optional feature, which removes both the
      PowerShell 2.0 parent feature and its engine. Without it, `powershell.exe -Version 2` no longer
      works, so scripts cannot be downgraded onto an engine that predates AMSI, script-block logging
      and module logging.

      ## Benefits
      - **Closes the standard downgrade evasion**: `-Version 2` is the classic way to run unlogged,
        unscanned PowerShell
      - **Every baseline requires it**: CIS and the DISA STIG both mandate it
      - **Effectively no compatibility cost**: PowerShell 2.0 has been deprecated since 2017

      ## Drawbacks
      - **Needs a reboot**: an optional-feature change is not complete until you restart
      - **Very rare legacy break**: software that explicitly forces `-Version 2` stops working
      - **Being removed anyway**: Microsoft has started removing PowerShell 2.0 from Insider builds,
        where the disable command fails instead of quietly succeeding
      - **Revert needs a second reboot**: reinstating the feature is another servicing operation

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 11 up to 23H2,
        where the feature is present and enabled by default
      - **Takes effect**: after reboot
      - **Reverting**: reinstalls the optional feature and then needs another reboot
      - The full check is that both `MicrosoftWindowsPowerShellV2Root` and
        `MicrosoftWindowsPowerShellV2` report Disabled

      ## Recommendation
      Apply it for almost everyone; the security gain is real and the compatibility cost is
      negligible. Skip it only if you know you run software that forces PowerShell 2.0.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Disable-WindowsOptionalFeature](https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature)
      - [DISA Windows 11 STIG V-253285: PowerShell 2.0 must be disabled](https://www.stigviewer.com/stigs/microsoft_windows_11/2025-05-15/finding/V-253285)
```

**Sources:**
1. Disable-WindowsOptionalFeature, https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature (tier A)
2. DISA Windows 11 STIG V-253285, https://www.stigviewer.com/stigs/microsoft_windows_11/2025-05-15/finding/V-253285 (tier B)

### `firewall_all_profiles` Firewall on all profiles (block inbound)

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism as authored:** Six REG_DWORD values under
`HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\{DomainProfile, **StandardProfile**, PublicProfile}`:
`EnableFirewall` = 1 and `DefaultInboundAction` = 1 in each. Revert: all `absent`. No reboot flag,
correct.

CORRECTED mechanism: the middle profile subkey must be **`PrivateProfile`**, not `StandardProfile`.

| Key | Values |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile` | `EnableFirewall` = 1, `DefaultInboundAction` = 1 |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile` | `EnableFirewall` = 1, `DefaultInboundAction` = 1 |
| `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile` | `EnableFirewall` = 1, `DefaultInboundAction` = 1 |

`StandardProfile` is the subkey name used only under the **non-policy** path
`HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy`. DISA's Windows Defender
Firewall STIG check for the private profile reads
`HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile\EnableFirewall` and names the
`SharedAccess` path only as the fallback.

**Corrections needed:** Rename the two `StandardProfile` effects to `PrivateProfile`. As authored the
Private profile, which is the one most home machines actually use, is never configured, and two orphan
values are written to a key nothing reads.

**Ready-to-paste info block:**

```yaml
    info: |
      **Forces the Windows firewall on for every network type with unsolicited inbound blocked.**

      ## What it does
      Writes `EnableFirewall` = 1 and `DefaultInboundAction` = 1 under the Domain, Private and Public
      profile subkeys of `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall`. Because these live under
      `Policies`, they also lock the corresponding controls in the Windows Security app.

      ## Benefits
      - **Cannot be silently switched off**: the policy overrides the user-facing toggles
      - **Default-deny inbound everywhere**: nothing reaches the PC without an explicit allow rule
      - **Covers all three profiles**: Domain, Private and Public

      ## Drawbacks
      - **Toggles are greyed out**: users see "This setting is managed by your administrator" and
        cannot turn the firewall off for troubleshooting until the tweak is reverted
      - **P2P and LAN apps may need rules**: services that expect open inbound ports need explicit
        allow rules created for them
      - **Conflicts with third-party firewalls**: a product that manages the machine's firewall will
        fight this
      - **Usually already on**: the firewall is enabled by default, so this mostly enforces rather
        than changes

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2, all editions
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all six values, which returns control to the user and lets Windows fall
        back to the non-policy firewall state
      - Setting these under `Policies` is a lock, not a preference

      ## Recommendation
      Apply it; the firewall on with default-block inbound is baseline security. Just be ready to add
      allow rules for any legitimate app that needs inbound access.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [DISA Windows Defender Firewall STIG V-241990 (private profile check path)](https://www.stigviewer.com/stigs/microsoft_windows_defender_firewall_with_advanced_security/2023-08-23/finding/V-241990)
      - [Firewall CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/firewall-csp)
```

**Sources:**
1. DISA Windows Defender Firewall with Advanced Security STIG V-241990, https://www.stigviewer.com/stigs/microsoft_windows_defender_firewall_with_advanced_security/2023-08-23/finding/V-241990 (tier B)
2. Firewall CSP, https://learn.microsoft.com/en-us/windows/client-management/mdm/firewall-csp (tier A)

### `audit_logon_events` Enable logon/credential auditing

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Two effects.

- Registry state marker: `HKCU\Software\MagicXToolbox\State` -> `AuditLogonEvents` (REG_DWORD).
- Action (PowerShell): apply
  `auditpol /set /subcategory:"{0CCE9215-69AE-11D9-BED3-505054503030}" /success:enable /failure:enable`;
  **undo `/success:enable /failure:disable`**; probe parses `auditpol /get ... /r` for both Success and
  Failure.

Options: "Success and failure" (`state: 1`, `audit_logon: run`), "Windows default (Stock Default)"
(`state: 0`). No reboot flag, correct. `elevation: admin` is correct because `auditpol.exe` requires it.
Using the GUID rather than the localised subcategory name is the right choice: it works regardless of
display language. `{0CCE9215-69AE-11D9-BED3-505054503030}` is the Logon subcategory of Logon/Logoff, and
enabling it produces events 4624, 4625, 4648 and 4675.

CORRECTED undo: capture the pre-apply inclusion setting with
`auditpol /get /subcategory:{0CCE9215-...} /r` into the snapshot and restore that, instead of the
hardcoded `/success:enable /failure:disable`.

**Corrections needed:** (a) The undo lowers the audit level. Microsoft's Audit Logon guidance lists
Success **and** Failure as expected on workstations, and Windows 10 and 11 clients ship auditing both,
so the current revert leaves the machine auditing less than before the tweak was ever applied. (b) Move
the state marker out of HKCU for the same reason as `remove_powershell_v2`. Confirm the shipped default
with `auditpol /get /category:*` on a clean install; recorded in UNKNOWNS.

**Ready-to-paste info block:**

```yaml
    info: |
      **Logs every successful and failed sign-in so intrusions leave a trail.**

      ## What it does
      Uses `auditpol` to enable Success and Failure auditing on the Logon subcategory
      (`{0CCE9215-69AE-11D9-BED3-505054503030}`) of the advanced audit policy. Windows then writes
      events 4624 (successful logon), 4625 (failed logon), 4648 (logon with explicit credentials) and
      4675 to the Security log.

      ## Benefits
      - **Backbone of detection**: 4624 and 4625 are what brute-force, password-spray and lateral
        movement detection are built on
      - **Low volume**: Microsoft rates the event volume as "low on a client computer"
      - **Language-independent**: the tweak uses the subcategory GUID, not a localised name

      ## Drawbacks
      - **Security log fills faster**: Microsoft's guidance is to raise the log's maximum size, which
        this tweak does not do
      - **No value if you never look**: it adds evidence, not protection
      - **Managed machines override it**: an audit-policy GPO overwrites whatever `auditpol` sets at
        the next refresh
      - **Legacy audit policy can override**: unless *Audit: Force audit policy subcategory settings
        to override audit policy category settings* (`SCENoApplyLegacyAuditPolicy`) is enabled

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2, all editions
      - **Takes effect**: immediately, no reboot
      - **Reverting**: restores the captured pre-apply inclusion setting; it must not hardcode
        Success-only, because Windows clients ship auditing both Success and Failure
      - Pair it with a larger Security log so events are not overwritten before you read them

      ## Recommendation
      Enable it if you want visibility into access attempts and can spare the log space. If you never
      review event logs the benefit is limited, but the cost is still close to zero.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Audit Logon](https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-logon)
      - [auditpol set](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/auditpol-set)
```

**Sources:**
1. Audit Logon, https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-logon (tier A)
2. auditpol set, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/auditpol-set (tier A)

### `lock_on_inactivity` Auto-lock on inactivity

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** Three **REG_SZ** values under `HKCU\Control Panel\Desktop`: `ScreenSaveActive`,
`ScreenSaverIsSecure`, `ScreenSaveTimeOut`. Options: "Lock after 10 min" = "1" / "1" / "600", "Windows
default (Stock Default)" = "1" / "0" / "600". `elevation: user`, correct. No reboot flag, correct.

All three are read as strings, which the YAML gets right: written as DWORDs the idle lock never engages.
The values live in the per-user hive, so the change applies to one user only.

CORRECTED mechanism, one of two paths:

- Add a fourth effect `HKCU\Control Panel\Desktop` -> `SCRNSAVE.EXE` (REG_SZ), for example
  `%SystemRoot%\System32\scrnsave.scr`, because Windows only starts the idle timer when a screen saver
  executable is actually selected; **or**
- Switch to the machine-wide, policy-backed *Interactive logon: Machine inactivity limit*:
  `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` -> `InactivityTimeoutSecs`
  (REG_DWORD, seconds), which locks the session regardless of screen saver configuration and covers
  every user on the machine. This is the robust mechanism.

**Corrections needed:** (a) The tweak can be a complete no-op. On a stock Windows 11 the screen saver is
"(None)" and `SCRNSAVE.EXE` is unset, in which case `ScreenSaveActive` = "1" and `ScreenSaverIsSecure` =
"1" produce no lock at all. (b) The Stock Default option hardcodes `ScreenSaverIsSecure` = "0" and
`ScreenSaveTimeOut` = "600". Neither is a documented Windows default and `ScreenSaverIsSecure` is
commonly absent on a clean profile. Restore captured values or use `absent`. Recorded in UNKNOWNS: read
all four values on a fresh profile.

**Ready-to-paste info block:**

```yaml
    info: |
      **Locks the screen after 10 minutes idle so a walk-away does not leave your session open.**

      ## What it does
      Sets `ScreenSaveActive` = "1", `ScreenSaverIsSecure` = "1" and `ScreenSaveTimeOut` = "600" under
      `HKCU\Control Panel\Desktop`, all as `REG_SZ` strings, and selects a screen saver executable so
      the idle timer actually starts. Windows then locks the session after ten idle minutes and asks
      for your password on resume.

      ## Benefits
      - **Closes the walk-away window**: an unattended signed-in session locks itself
      - **Simple and instant**: no reboot, no service, no policy lock
      - **Adjustable**: the timeout is just a number of seconds

      ## Drawbacks
      - **Per-user only**: a second account on the same machine is unaffected, because the values live
        in the user hive
      - **Needs a screen saver selected**: without `SCRNSAVE.EXE` set, Windows never starts the idle
        timer and the tweak does nothing at all
      - **Interrupts long reads and playback**: a ten-minute timeout can bite during video, reading or
        a presentation
      - **Physical threats only**: it does nothing against remote or malware attacks

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: at the next sign-in, or when Explorer re-reads user preferences; no reboot
      - **Reverting**: restores the captured values from the snapshot rather than writing literals,
        because none of these three has a documented Windows default
      - The machine-wide alternative is *Interactive logon: Machine inactivity limit*
        (`InactivityTimeoutSecs`), which covers every user and does not depend on a screen saver

      ## Recommendation
      Recommended for laptops and any PC used in shared or public spaces. On a physically secure home
      desktop the benefit is small, and you can lengthen the timeout if ten minutes feels short.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Group Policy screensaver setting isn't working in Windows](https://learn.microsoft.com/en-us/troubleshoot/windows-client/group-policy/group-policy-screensaver-setting-not-work)
      - [You cannot change or save screen saver settings](https://support.microsoft.com/en-us/help/968558/you-cannot-change-or-save-screen-saver-settings)
```

**Sources:**
1. Group Policy Screensaver setting isn't working in Windows, https://learn.microsoft.com/en-us/troubleshoot/windows-client/group-policy/group-policy-screensaver-setting-not-work (tier A, confirms the screen-saver-selected dependency)
2. You cannot change or save Screen Saver settings, https://support.microsoft.com/en-us/help/968558/you-cannot-change-or-save-screen-saver-settings (tier A)
3. Auto Lock Computer Screen After Inactivity with GPO, https://woshub.com/windows-lock-screen-after-idle-via-gpo/ (tier C, confirms REG_SZ typing of all three values)

### `enable_credential_guard` Credential Guard

**Verdict:** **INCORRECT**

**Mechanism as authored (defective):** `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` ->
`LsaCfgFlags` (REG_DWORD). Options: "Enabled" = 1, "Windows default (Stock Default)" = `absent`.
`requires_reboot: true`, `windows: { products: [11] }`.

CORRECTED mechanism:

| Key | Value name | Type | Enabled | Stock Default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` | `LsaCfgFlags` | REG_DWORD | **`2`** (enabled without UEFI lock) | **`0`** |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `EnableVirtualizationBasedSecurity` | REG_DWORD | `1` | `0` |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `RequirePlatformSecurityFeatures` | REG_DWORD | `1` or `3` | `absent` |

The policy route, as an alternative to the direct registry route, is
`HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` -> `LsaCfgFlags`.

The key currently written, `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard`, holds
`EnableVirtualizationBasedSecurity` and `RequirePlatformSecurityFeatures` but **not** `LsaCfgFlags`. As
authored the tweak is a no-op.

**Corrections needed:**

1. **Wrong key.** Move `LsaCfgFlags` to `Control\Lsa` (registry route) or the `Policies\...\DeviceGuard`
   key (policy route).
2. **Missing prerequisite.** Also set `EnableVirtualizationBasedSecurity` = 1 (and optionally
   `RequirePlatformSecurityFeatures`) under `Control\DeviceGuard`.
3. **Wrong value for a reversible tweak.** 1 is "enabled with UEFI lock", removable only through a
   `bcdedit` plus `SecConfig.efi` procedure with physical presence. Use 2.
4. **Wrong revert.** Microsoft: "Deleting these registry settings may not disable Credential Guard.
   They must be set to a value of 0." The revert must write 0, not `absent`.
5. **Wrong default claim and wrong gate.** Credential Guard is enabled by default on eligible
   Windows 11 22H2 and later and on Windows Server 2025. The `windows: { products: [11] }` gate
   excludes Windows 10 Enterprise and Education, where the feature exists, and fails to exclude
   Windows Home, where it does not.
6. **Live conflict once fixed.** Per `_cross-category.md` finding 1, `performance:disable_vbs_hvci`
   writes `EnableVirtualizationBasedSecurity` = 0 to the same key. Today the conflict is masked by the
   wrong-key defect. Fixing the key activates it, so the dependency must be declared (or cross-warned)
   in the same change.

**Ready-to-paste info block:**

```yaml
    info: |
      **Moves your domain and NTLM secrets into a hypervisor-isolated enclave, out of LSASS.**

      ## What it does
      Sets `LsaCfgFlags` = 2 under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` and
      `EnableVirtualizationBasedSecurity` = 1 under `Control\DeviceGuard`. Credential Guard then uses
      virtualization-based security to hold NTLM hashes, Kerberos tickets and application domain
      credentials in the isolated LSA process (`LsaIso.exe`), unreachable from the normal OS. Value 2
      enables it without a UEFI lock, so it stays reversible.

      ## Benefits
      - **Defeats pass-the-hash**: the secrets are not in LSASS memory to steal
      - **Hardware-backed**: isolation is enforced by the hypervisor, not by process permissions
      - **Strongest option for domain credentials**: on a domain-joined or Entra-joined machine

      ## Drawbacks
      - **Breaks legacy protocols**: unconstrained Kerberos delegation, DES and RC4 Kerberos
        encryption, NTLMv1 and MS-CHAPv2 for some VPN configurations stop working, as do credential
        providers that load into LSA without meeting the requirements
      - **Requires VBS**: it cannot run if virtualization-based security is turned off, so it directly
        conflicts with any tweak that disables VBS or memory integrity
      - **Costs memory and, on older hardware, performance**
      - **Little benefit standalone**: on a home PC there are few domain secrets to protect
      - **Value 1 is a trap**: "enabled with UEFI lock" can only be removed with a boot-time
        `bcdedit` and `SecConfig.efi` procedure at the machine

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, Enterprise and Education (and Pro under some
        licensing); not available on Home. Requires 64-bit, Secure Boot, virtualization extensions
        and VBS
      - **Takes effect**: after reboot
      - **Reverting**: writes `LsaCfgFlags` = 0; Microsoft states deleting the value may not disable
        Credential Guard, so the revert must write 0 rather than remove it
      - Already enabled by default on eligible Windows 11 22H2 and later installs
      - Do not apply this alongside a tweak that disables VBS or memory integrity

      ## Recommendation
      Recommended for domain-joined or Entra-joined Windows 11 machines that meet the hardware
      requirements. On a standalone home PC the legacy-protocol restrictions cost more than the
      protection returns.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Configure Credential Guard](https://learn.microsoft.com/en-us/windows/security/identity-protection/credential-guard/configure)
      - [Configure added LSA protection](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
```

**Sources:**
1. Configure Credential Guard, https://learn.microsoft.com/en-us/windows/security/identity-protection/credential-guard/configure (tier A)
2. Configure added LSA protection (LSA and Credential Guard interaction, default enablement), https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection (tier A)
3. `docs/superpowers/research/validation/_cross-category.md` finding 1 (VBS conflict, masked by the wrong-key defect)

### `defender_cloud_protection` Defender cloud protection and MAPS

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:** Five REG_DWORD values, class Machine, all confirmed verbatim in the shipped
`WindowsDefender.admx` on 26100.

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet` | `SpynetReporting` | REG_DWORD | `2` (Advanced MAPS) | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet` | `SubmitSamplesConsent` | REG_DWORD | `1` (send safe samples) | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet` | `DisableBlockAtFirstSeen` | REG_DWORD | `0` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine` | `MpCloudBlockLevel` | REG_DWORD | `2` (High) | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine` | `MpBafsExtendedTimeout` | REG_DWORD | `50` (seconds) | `absent` |

Enum members from the ADMX: `SpynetReporting` 0 Disabled, 1 Basic, 2 Advanced. `SubmitSamplesConsent`
0 Always Prompt, 1 Send Safe, 2 Never Send, 3 Send All. `DisableBlockAtFirstSeen` enabledValue 0,
disabledValue 1. `MpCloudBlockLevel` 0 Default, 1 Moderate, 2 High, 4 High Plus, 6 Zero Tolerance.
`MpBafsExtendedTimeout` is a decimal element with `minValue="0" maxValue="50"`, so the proposed 50 sits
exactly on the maximum.

CORRECTED effective defaults and behaviour:

- `SubmitSamplesConsent` effective default is `SendSafeSamples`, which Microsoft calls "the default,
  recommended setting", so the hardened value 1 is the same as the default.
- Tamper Protection, on by default on consumer Windows 11, forces cloud protection on and lists
  "Cloud protection remains enabled" among settings that cannot be changed. Microsoft: "any changes
  made to tamper-protected settings are ignored". `SpynetReporting` and `DisableBlockAtFirstSeen` sit
  on that surface; `MpCloudBlockLevel` and `MpBafsExtendedTimeout` do not and are what actually
  delivers new behaviour.
- The dependency justification must be rewritten. None of the corpus's five ASR rules is cloud-gated
  (the ASR rules reference lists "Dependencies: Microsoft Defender Antivirus" for all of them), and
  Controlled Folder Access needs real-time protection, not cloud protection. The one genuine dependency
  is `enable_network_protection`, whose requirements table names real-time protection, behavior
  monitoring **and** cloud-delivered protection.

**Corrections needed:** (1) Restate the justification around `enable_network_protection` and drop the
ASR framing. (2) The probe must read `Get-MpPreference` or `Get-MpComputerStatus` rather than the
registry, or the tweak must carry `skip_validation`, because the registry write can land while the
effective setting does not move. (3) The proposed second option, "Cloud protection on, no sample
submission", sets `SubmitSamplesConsent` = 2 (Never Send) while keeping `DisableBlockAtFirstSeen` = 0;
Microsoft states "the NeverSend setting means that the Block at First Sight feature ... won't work", so
the option advertises and disables the same feature. Either rename it to make the trade explicit or use
`SubmitSamplesConsent` = 0 (Always Prompt). (4) Say in Drawbacks that `SubmitSamplesConsent` = 1 is the
existing default. (5) Gate on Defender presence: inert where a third-party antivirus has put Defender
in passive mode, and on images with Defender removed. (6) This tweak and any future "disable MAPS"
privacy tweak are mutually exclusive and must be presented as such.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns Defender's cloud lookups up to High so brand-new malware is blocked on first sight.**

      ## What it does
      Sets Advanced MAPS membership (`SpynetReporting` = 2), safe-sample submission
      (`SubmitSamplesConsent` = 1), Block at First Sight on (`DisableBlockAtFirstSeen` = 0), the cloud
      block level to High (`MpCloudBlockLevel` = 2) and the cloud check timeout to its 50-second
      maximum (`MpBafsExtendedTimeout` = 50).

      ## Benefits
      - **Catches new malware**: cloud lookups classify files that have no signature yet
      - **High block level is the real change**: `MpCloudBlockLevel` and the extended timeout are the
        two values Tamper Protection does not already force
      - **Enables a prerequisite**: Defender Network Protection requires cloud-delivered protection

      ## Drawbacks
      - **More false positives**: Microsoft documents block level High as carrying "a greater chance
        of false positives"
      - **Sends files to Microsoft**: safe-sample submission uploads suspicious files, which is a
        direct trade against a privacy posture
      - **Partly already on**: Tamper Protection forces cloud protection on by default on consumer
        Windows 11, and safe-sample submission is already the default, so the registry write can land
        without the effective setting moving
      - **Inert without Defender**: with a third-party antivirus installed Defender goes passive and
        none of this applies

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2. Requires Microsoft Defender
        Antivirus as the active antivirus
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all five values, returning Defender to its own defaults
      - Check the result with `Get-MpPreference`, not the registry: tamper-protected settings ignore
        policy writes
      - This tweak is the opposite of a "turn MAPS off" privacy posture; do not apply both

      ## Recommendation
      Apply it if you run Defender and want maximum detection, and accept that suspicious files are
      uploaded for analysis. If sample submission is unacceptable to you, use the no-submission
      option and understand that Block at First Sight will not work.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Turn on cloud protection in Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/enable-cloud-protection-microsoft-defender-antivirus)
      - [Specify the cloud protection level](https://learn.microsoft.com/en-us/defender-endpoint/specify-cloud-protection-level-microsoft-defender-antivirus)
      - [Protect security settings with tamper protection](https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policies `SpynetReporting`, `SubmitSamplesConsent`, `DisableBlockAtFirstSeen`, `MpEngine_MpCloudBlockLevel`, `MpEngine_MpBafsExtendedTimeout` (tier A, shipped ADMX)
2. Turn on cloud protection in Microsoft Defender Antivirus, https://learn.microsoft.com/en-us/defender-endpoint/enable-cloud-protection-microsoft-defender-antivirus (tier A)
3. Specify the cloud protection level, https://learn.microsoft.com/en-us/defender-endpoint/specify-cloud-protection-level-microsoft-defender-antivirus (tier A)
4. Protect security settings with tamper protection, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)
5. Attack surface reduction (ASR) rules reference, per-rule Dependencies fields, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)

### `smb_client_block_ntlm` Block NTLM on the SMB client

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism (CORRECTED, registry not PowerShell):**

| Key | Value name | Type | Block NTLM | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation` | `BlockNTLM` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation` | `BlockNTLMServerExceptionList` | REG_MULTI_SZ | one server name per entry | `absent` |

The gap proposal claimed the policy is not in the shipped 26100 `LanmanWorkstation.admx` and that a
`Set-SmbClientConfiguration -BlockNTLM $true` action was therefore required. That is false. The shipped
ADMX contains, verbatim:

```xml
<policy class="Machine" displayName="$(string.Pol_BlockNTLM_Name)"
        key="Software\Policies\Microsoft\Windows\LanmanWorkstation"
        name="Pol_BlockNTLM" valueName="BlockNTLM">
  <supportedOn ref="SUPPORTED_Windows_Server_2025_Windows_11_0" />
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

The companion `Pol_BlockNTLMServerExceptionList` is in the same file writing
`BlockNTLMServerExceptionList` via a `multiText` element (REG_MULTI_SZ). The ADML resolves
`SUPPORTED_Windows_Server_2025_Windows_11_0` to "At least Windows Server 2025, Windows 11", and the
help string reads: "This policy controls if the SMB client will block NTLM for remote connection
authentication."

Binary evidence corroborates the store split. The shipped 26100 `smbwmiv2.dll` (the WMI provider behind
`Set-SmbClientConfiguration`) contains `BlockNTLM`, `BlockNTLMServerExceptionList`,
`GetBoolRegistryValue(BlockNTLM)`, `SetBoolRegistryValue(BlockNTLM)`,
`SmbResetBooleanPropertyToDefault(BlockNTLM)`, `LanmanWorkstationParameters` and
`Software\Policies\Microsoft\Windows\LanmanWorkstation`. `wkssvc.dll` carries `BlockNTLMInfo` and
"BlockNTLM params specified by user"; `mrxsmb20.sys` carries `qBlockNTLMFromGlobalSetting` and
`qBlockNTLMFromNetUseSetting`, which is the per-mapping (`net use`) setting. The cmdlet writes the
non-policy value at `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters\BlockNTLM`;
the policy key is the documented Group Policy surface and wins.

**Corrections needed:** Implement as a registry effect on the policy key, not as a PowerShell action.
Revert by **deleting** the value: 0 is the ADMX `disabledValue`, which means "policy explicitly says
allow NTLM" rather than "unconfigured". `SmbResetBooleanPropertyToDefault(BlockNTLM)` in the shipped
provider confirms delete is a first-class reset path. Applicability `windows: { build: ">=26100" }` is
correct and survives; the control does not exist on LTSC 2021.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops this PC ever sending an NTLM response to an SMB server, killing NTLM relay at the
      source.**

      ## What it does
      Sets `BlockNTLM` = 1 under `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation`, the
      Group Policy "Block NTLM (LM, NTLM, NTLMv2)" setting introduced with Windows 11 24H2. The SMB
      client then refuses to use NTLM for remote connection authentication and uses Kerberos only.

      ## Benefits
      - **Kills coerced authentication**: you cannot be tricked into sending NTLM challenge responses
        to a hostile SMB server
      - **Stronger than picking an NTLM version**: `LmCompatibilityLevel` chooses which NTLM variant
        is used; this stops NTLM being offered at all
      - **Has an exception list**: `BlockNTLMServerExceptionList` lets you allow specific servers

      ## Drawbacks
      - **Breaks anything that cannot do Kerberos**: NAS reached by IP address rather than name,
        workgroup file shares, and older network printers with scan-to-folder all stop working
      - **24H2 and newer only**: the control does not exist on Windows 10 or LTSC 2021
      - **Easy to under-estimate**: home networks lean heavily on IP-address SMB, which is exactly
        what this blocks

      ## Good to know
      - **Applies to**: Windows 11 24H2 (build 26100) and newer, and Windows Server 2025
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value; it must not write 0, which is the policy explicitly saying
        "allow NTLM" rather than "unconfigured"
      - Outbound SMB client behaviour only, so it cannot lock you out of this machine
      - A per-mapping exception exists too: `New-SmbMapping -BlockNTLM $false`

      ## Recommendation
      Apply it if all your SMB targets are reachable by name and can do Kerberos, which usually means
      a domain or a modern NAS with proper DNS. Skip it on a typical home network that mounts shares
      by IP address.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Block NTLM connections on SMB](https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-ntlm-blocking)
      - [What's new in Windows 11, version 24H2](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-24h2)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\LanmanWorkstation.admx` and `en-US\LanmanWorkstation.adml` (26100), policies `Pol_BlockNTLM` and `Pol_BlockNTLMServerExceptionList` (tier A, shipped ADMX)
2. Block NTLM connections on SMB, Group Policy tab, https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-ntlm-blocking (tier A)
3. UTF-16 string extraction from shipped 26100 `smbwmiv2.dll`, `wkssvc.dll`, `mrxsmb20.sys`, `mrxsmb.sys` (tier A, shipped binaries)

### `enhanced_phishing_protection` Enhanced Phishing Protection

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:** Five REG_DWORD values, class Machine, key
`HKLM\SOFTWARE\Policies\Microsoft\Windows\WTDS\Components`, all confirmed in the shipped
`WebThreatDefense.admx` on 26100 with `enabledValue` 1 and `disabledValue` 0, all gated
`supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"`.

| Value name | Hardened | Registry default | **Effective** default on consumer 26100 |
|---|---|---|---|
| `ServiceEnabled` | `1` | `absent` | **Enabled** (Policy CSP "Default Value: 1") |
| `NotifyMalicious` | `1` | `absent` | **Enabled** for devices not onboarded to Defender for Endpoint |
| `NotifyPasswordReuse` | `1` | `absent` | **Disabled** |
| `NotifyUnsafeApp` | `1` | `absent` | **Disabled** |
| `CaptureThreatWindow` | `1` | `absent` | **Disabled** unless domain-joined or MDM-enrolled |

Authoring note: the ADMX policy that writes `CaptureThreatWindow` is named `AutomaticDataCollection`,
not `CaptureThreatWindow`.

CORRECTED scope and semantics:

- The feature covers **work or school passwords only**. Microsoft repeats this for every component:
  "helps protect Microsoft school or work passwords", "warns users if they reuse their work or school
  password", "notifications when users type their work or school passwords in Notepad and Microsoft 365
  Office Apps". On a machine signed in with a local account or a personal Microsoft account it is
  largely inert.
- `ServiceEnabled` = 1 means the feature is enabled **in audit mode**, not "on with warnings". The
  shipped ADML: "If you enable this policy setting, Enhanced Phishing Protection in Microsoft Defender
  SmartScreen is enabled **in audit mode** and your users are unable to turn it off." The three
  `Notify*` values produce the warnings.
- The genuine additions on a consumer 26100 are `NotifyPasswordReuse` and `NotifyUnsafeApp`.

**Corrections needed:** (1) Rewrite the description to say work or school passwords, not "a Windows
password". (2) Do not attribute the warnings to `ServiceEnabled`. (3) Note that writing `ServiceEnabled`
stops users turning the feature off in the Windows Security UI, which belongs in Drawbacks. (4) Replace
the flat "value-absent (feature ships in audit mode)" default column with the per-value effective
defaults above. Applicability `windows: { products: [11] }` is correct: Policy CSP gives Windows 11 22H2
(10.0.22621) and later, editions Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. No
lockout or breakage risk.

**Ready-to-paste info block:**

```yaml
    info: |
      **Warns you when your work or school password is typed into a phishing site, reused, or saved in
      an unsafe app.**

      ## What it does
      Turns on all five Enhanced Phishing Protection policy values under
      `HKLM\SOFTWARE\Policies\Microsoft\Windows\WTDS\Components`. `ServiceEnabled` enables the
      component in audit mode; `NotifyMalicious`, `NotifyPasswordReuse` and `NotifyUnsafeApp` produce
      the actual warnings; `CaptureThreatWindow` sends a screenshot of the offending window to
      Microsoft for analysis.

      ## Benefits
      - **Catches password reuse**: warns when a work or school password is typed into a website
      - **Catches unsafe storage**: warns when that password is typed into Notepad or Office apps
      - **Two values genuinely change behaviour**: password-reuse and unsafe-app warnings are off by
        default on a consumer machine

      ## Drawbacks
      - **Work and school passwords only**: with a local account or a personal Microsoft account the
        feature has nothing to protect and does effectively nothing
      - **Locks the UI toggle**: with `ServiceEnabled` written, users can no longer turn the feature
        off in the Windows Security app
      - **Screenshot upload**: `CaptureThreatWindow` is a privacy cost, not a security gain, and
        should be a separate effect the user can see
      - **Two of five are already on**: `ServiceEnabled` and `NotifyMalicious` are already the
        effective defaults, so writing them changes nothing except the lock

      ## Good to know
      - **Applies to**: Windows 11 22H2 and newer (24H2 and 25H2 included), Pro / Enterprise /
        Education / IoT Enterprise. The component does not exist on Windows 10
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all five values, restoring the shipped behaviour and unlocking the
        Windows Security toggle
      - `ServiceEnabled` = 1 is audit mode; the warnings come from the three `Notify*` values

      ## Recommendation
      Apply it on a machine signed in with a work or school account, where the password-reuse and
      unsafe-app warnings are real protection. On a personal machine skip it, or apply it without
      `CaptureThreatWindow` so nothing is uploaded.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Enhanced Phishing Protection in Microsoft Defender SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/enhanced-phishing-protection)
      - [Policy CSP WebThreatDefense](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-webthreatdefense)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\WebThreatDefense.admx` and `en-US\WebThreatDefense.adml` (26100) (tier A, shipped ADMX)
2. Enhanced Phishing Protection in Microsoft Defender SmartScreen, "Recommended settings for your organization" default-value table, https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/enhanced-phishing-protection (tier A)
3. Policy CSP WebThreatDefense, `ServiceEnabled` Default Value 1, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-webthreatdefense (tier A)

### `asr_standard_protection_rules` ASR standard protection rules

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Two REG_SZ values under
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`, plus the
ADMX parent enabling value.

| Key | Value name | Type | Block | Audit | Stock Default |
|---|---|---|---|---|---|
| `...\Exploit Guard\ASR\Rules` | `56a863a9-875e-4185-98a7-b882c64b5ce5` | REG_SZ | `"1"` | `"2"` | `absent` |
| `...\Exploit Guard\ASR\Rules` | `e6db77e5-3df2-4cf1-b95a-636979351e5b` | REG_SZ | `"1"` | `"2"` | `absent` |
| `...\Exploit Guard\ASR` | `ExploitGuard_ASR_Rules` | REG_DWORD | `1` | `1` | `absent` |

`56a863a9-875e-4185-98a7-b882c64b5ce5` is "Block abuse of exploited vulnerable signed drivers";
`e6db77e5-3df2-4cf1-b95a-636979351e5b` is "Block persistence through WMI event subscription". Both GUIDs
were checked character by character against Microsoft's ASR rules reference, and both sit under its
"Standard protection rules" heading. That section contains exactly three rules: these two plus the LSASS
credential-theft rule. Value semantics: `"0"` off, `"1"` block, `"2"` audit, `"6"` warn.

The REG_SZ typing is confirmed structurally: `WindowsDefender.admx` declares the `Rules` subkey as an
ADMX `<list>` element with `explicitValue="true"`, which means the value **name** is the GUID and the
value **data** is the state, written as a string.

Dependencies, checked explicitly: `56a863a9` **Dependencies: None**; `e6db77e5` **Dependencies:
Microsoft Defender Antivirus, RPC**. Neither requires cloud protection, so this tweak does not depend on
`defender_cloud_protection`. Both need Defender to be the active antivirus.

**Corrections needed:** `none` to the mechanism. Drop the proposal's claim that "both rules work on
Windows 11 Home": no tier A source confirms or denies ASR enforcement on Home, so treat it as
unsupported rather than false. Two copy notes: `56a863a9` supports user notification pop-ups but does
not raise EDR alerts, while `e6db77e5` does both.

**Ready-to-paste info block:**

```yaml
    info: |
      **Adds the two Defender rules Microsoft recommends deploying straight to Block with no testing.**

      ## What it does
      Enables two attack surface reduction rules in Block mode: "Block abuse of exploited vulnerable
      signed drivers" (`56a863a9-875e-4185-98a7-b882c64b5ce5`) and "Block persistence through WMI
      event subscription" (`e6db77e5-3df2-4cf1-b95a-636979351e5b`). Both are written as string values
      under the Exploit Guard ASR rules key alongside the policy's enabling value.

      ## Benefits
      - **Microsoft's own standard set**: these are two of the three rules Microsoft designates for
        direct Block deployment with no audit period
      - **Blocks BYOVD at the write**: the driver rule stops apps saving vulnerable signed drivers to
        disk, complementing the HVCI blocklist rather than duplicating it
      - **Closes a stealthy persistence path**: WMI event subscriptions survive reboots and are easy
        to miss

      ## Drawbacks
      - **Needs Defender active**: both are inert when a third-party antivirus puts Defender in
        passive mode
      - **Limited exclusions on the WMI rule**: `e6db77e5` has limited exclusion support, and
        Configuration Manager clients lean heavily on WMI (irrelevant on a home machine, not on a
        managed one)
      - **Does not unload what is already loaded**: the driver rule prevents saving vulnerable
        drivers, not loading ones already present
      - **Home is unproven**: Microsoft makes no statement about ASR enforcement on Windows 11 Home

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; `56a863a9` also on Windows 10 1709 and later and
        `e6db77e5` on Windows 10 1903 and later, so both work on LTSC 2021
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes both rule values and the policy's enabling value
      - Neither rule requires cloud-delivered protection
      - The driver rule shows user pop-ups but raises no EDR alerts; the WMI rule does both

      ## Recommendation
      Apply it on any machine where Defender is the active antivirus. Microsoft's own guidance is that
      these rules go straight to Block, and the breakage risk on a consumer machine is close to zero.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [ASR rules reference, standard protection rules](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference)
      - [ASR rules overview](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview)
```

**Sources:**
1. Attack surface reduction (ASR) rules reference, per-rule GUID and Dependencies fields and the "Standard protection rules" section, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policy `ExploitGuard_ASR_Rules` (tier A, shipped ADMX)

### `disable_winrm_remoting` Disable WinRM remoting

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Six REG_DWORD values plus one service effect. All six confirmed verbatim in the shipped
`WindowsRemoteManagement.admx` on 26100, class Machine.

| ADMX policy | Key | Value name | enabledValue | disabledValue | Hardened |
|---|---|---|---|---|---|
| `AllowBasic_2` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client` | `AllowBasic` | 1 | 0 | `0` |
| `AllowUnencrypted_2` | `...\WinRM\Client` | `AllowUnencryptedTraffic` | 1 | 0 | `0` |
| `DisallowDigest` | `...\WinRM\Client` | `AllowDigest` | **0** | **1** | `0` |
| `AllowBasic_1` | `...\WinRM\Service` | `AllowBasic` | 1 | 0 | `0` |
| `AllowUnencrypted_1` | `...\WinRM\Service` | `AllowUnencryptedTraffic` | 1 | 0 | `0` |
| `DisableRunAs` | `...\WinRM\Service` | `DisableRunAs` | 1 | 0 | `1` |

Service effect: `WinRM` (Windows Remote Management), hardened start type `disabled`.

`DisallowDigest` is the inverted one: the policy being "Enabled" writes `AllowDigest` = 0. All six stock
defaults are value-absent.

**Lockout analysis:** this cannot lock a user out. WinRM governs **inbound** WS-Man only; local sign-in,
console and the internal keyboard are untouched. Microsoft: "By default, no WinRM listener is
configured. Even if the WinRM service is running, WS-Management protocol messages that request data
can't be received or sent." Outbound `Invoke-Command` from this machine is unaffected, SSH remoting is a
separate channel, and RDP is a separate service.

**Corrections needed:** `none` to the six values. One correction to the proposal's revert: **do not
hardcode `Manual` as the stock start type.** No tier A source states a start-type value; Microsoft says
only "The WinRM service starts automatically on Windows Server 2008, and later. On earlier versions of
Windows (client or server), you need to start the service manually", which supports "not automatic on
client" but does not pin Manual versus trigger-start. Restore the start type from the snapshot. This
matters concretely: any machine where `winrm quickconfig` has run sits at **delayed auto start** with a
listener configured, so a hardcoded Manual revert would silently downgrade a working configuration.
Recorded in UNKNOWNS.

**Ready-to-paste info block:**

```yaml
    info: |
      **Shuts down WinRM, the remote-execution channel PowerShell Remoting and attack tooling use.**

      ## What it does
      Sets the `WinRM` service to Disabled and pins six policy values that forbid Basic
      authentication, unencrypted traffic and Digest authentication on both the WinRM client and the
      WinRM service, plus `DisableRunAs` = 1 so stored RunAs credentials cannot be used.

      ## Benefits
      - **Removes the standard remote-execution channel**: WinRM is what post-exploitation tooling
        reaches for after credential theft
      - **Hardens the client too**: even if you re-enable the service later, Basic auth and cleartext
        transport stay off
      - **Six STIG rules in one**: matches the DISA WinRM control set

      ## Drawbacks
      - **Inbound PowerShell Remoting stops**: `Enter-PSSession` and `Invoke-Command` **into** this
        machine fail
      - **Event forwarding breaks**: the Windows Event Collector service (`wecsvc`) depends on WinRM
      - **Remote management tools break**: Windows Admin Center and several remote-management agents
        stop working
      - **Usually no visible change**: a stock Windows 11 client has no WinRM listener configured, so
        disabling the service typically changes nothing observable

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: restores the previous service start type from the snapshot and deletes the six
        policy values; it must not hardcode Manual, because a machine where `winrm quickconfig` has
        run sits at delayed auto start
      - Outbound `Invoke-Command` from this PC to other machines is unaffected, and SSH remoting is a
        separate channel
      - It cannot lock you out: WinRM is inbound only

      ## Recommendation
      Apply it on a standalone or home machine that never accepts PowerShell Remoting. If you manage
      this PC remotely or use event forwarding, use the values-only option and leave the service
      alone.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Installation and configuration for Windows Remote Management](https://learn.microsoft.com/en-us/windows/win32/winrm/installation-and-configuration-for-windows-remote-management)
      - [DISA STIG for Windows 11 V2R2, WinRM rules](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\WindowsRemoteManagement.admx` (26100), policies `AllowBasic_1`, `AllowBasic_2`, `AllowUnencrypted_1`, `AllowUnencrypted_2`, `DisallowDigest`, `DisableRunAs` (tier A, shipped ADMX)
2. Installation and configuration for Windows Remote Management, "Configuration of WinRM and IPMI" and "Quick default configuration", https://learn.microsoft.com/en-us/windows/win32/winrm/installation-and-configuration-for-windows-remote-management (tier A)
3. DISA STIG for Windows 11 V2R2, six WinRM rules (tier B)

### `powershell_module_transcript_logging` PowerShell module logging and transcription

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Five values across three keys, all confirmed in the shipped
`PowerShellExecutionPolicy.admx` on 26100. Policy class is `Both`, so an HKCU copy would also take
effect for that user; writing HKLM is the correct machine-wide choice and HKCU is not needed.

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ModuleLogging` | `EnableModuleLogging` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ModuleLogging\ModuleNames` | `*` | REG_SZ | `"*"` | key absent |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription` | `EnableTranscripting` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription` | `EnableInvocationHeader` | REG_DWORD | `1` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription` | `OutputDirectory` | REG_SZ | a fixed admin-writable path under `%ProgramData%` | `absent` |

Type mapping comes from the ADMX element types: `text` element -> REG_SZ (`OutputDirectory`), `boolean`
element -> REG_DWORD (`EnableInvocationHeader`).

**The `ModuleNames` layout is settled from PowerShell's own reader.** The ADMX `<list>` element carries
no `explicitValue`, which would normally mean Group Policy writes numbered value names. PowerShell's
`TrySetPolicySettingsFromRegistryKey` in `src/System.Management.Automation/engine/Utils.cs` reads the
**value names** of the `ModuleNames` subkey (`rawRegistryValue = subKey.GetValueNames();`) and ignores
the data entirely. So a value **named** `*` is correct and is the only layout that works. A numbered
value name such as `1` = `*` would register a module literally called "1" and match nothing, so a
generic ADMX list writer must not be allowed to produce that shape. The `REG_SZ` type and `"*"` data
are cosmetic to PowerShell but match what CIS, the DISA STIG and Sophia Script all write, so keep them.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Records what PowerShell actually executed and writes a durable transcript of every session.**

      ## What it does
      Turns on module logging for all modules (`EnableModuleLogging` = 1 with a value named `*` under
      the `ModuleNames` subkey) and PowerShell transcription (`EnableTranscripting` = 1,
      `EnableInvocationHeader` = 1) with `OutputDirectory` pointed at a fixed admin-writable path.
      Module logging records pipeline execution details; transcription writes a text file per session.

      ## Benefits
      - **Closes the console blind spot**: script-block logging records what a script contained,
        transcription records what someone actually did interactively
      - **Two separate baseline requirements**: the DISA STIG requires transcription and CIS lists
        module logging as Level 1
      - **Durable**: transcripts survive on disk even if the event log rolls

      ## Drawbacks
      - **Transcript files accumulate**: one file per session, which adds up on a machine that runs a
        lot of scripted work
      - **Very verbose**: module logging with `*` fills the PowerShell operational log faster, so
        pair it with a larger event log
      - **Secrets land in files**: anything typed or piped in plaintext is written to the transcript,
        so the output directory must be protected
      - **Wrong default location**: without `OutputDirectory` set, transcripts go into the user's
        Documents folder

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2. Covers Windows PowerShell 5.1
        and PowerShell 7
      - **Takes effect**: immediately, for new PowerShell sessions
      - **Reverting**: deletes all five values and removes the `ModuleNames` subkey
      - The `ModuleNames` entry must be a value **named** `*`; PowerShell reads the value names and
        ignores the data, so a numbered entry would register a module called "1" and match nothing
      - Point `OutputDirectory` somewhere under `%ProgramData%`, not the user's Documents folder

      ## Recommendation
      Enable both if you investigate incidents or want an audit trail. If transcript files on disk
      bother you, take module logging alone; it still records pipeline activity into the event log.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
      - [PowerShell source, Utils.cs policy reader](https://raw.githubusercontent.com/PowerShell/PowerShell/master/src/System.Management.Automation/engine/Utils.cs)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\PowerShellExecutionPolicy.admx` (26100), policies `EnableModuleLogging` and `EnableTranscripting` (tier A, shipped ADMX)
2. PowerShell source, `src/System.Management.Automation/engine/Utils.cs`, `TrySetPolicySettingsFromRegistryKey`, https://raw.githubusercontent.com/PowerShell/PowerShell/master/src/System.Management.Automation/engine/Utils.cs (tier A, first-party implementation)
3. about_Logging_Windows, https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows (tier A)
4. Sophia Script for Windows 11, `ModuleNames` write using value name `*` (tier C, corroboration only)

### `restrict_remote_sam` Restrict remote SAM calls to administrators

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Microsoft's Security Policy Settings reference states the fields directly.

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` | `RestrictRemoteSam` | **REG_SZ** | `O:BAG:BAD:(A;;RC;;;BA)` | `absent` |

**The type is REG_SZ, not REG_DWORD.** Microsoft's page gives "Registry type: REG_SZ" and "Registry
value: A string that will contain the SDDL of the security descriptor to be deployed". A DWORD write
here would be silently ignored, which is the corpus's most common real defect class.

The hardened SDDL `O:BAG:BAD:(A;;RC;;;BA)` is the standard CIS and DISA STIG value and parses as owner
Built-in Administrators, group Built-in Administrators, DACL granting Read Control to Built-in
Administrators only.

**Not a duplicate.** `restrict_anonymous_enum` writes `RestrictAnonymousSAM`, `RestrictAnonymous` and
`EveryoneIncludesAnonymous` in the same `Lsa` key, and those govern **anonymous** sessions.
`RestrictRemoteSam` governs which **authenticated** principals may make SAMRPC calls. Different value,
different attack, additive.

**Corrections needed:** `none`. One caution to add: because the value is a raw SDDL, a malformed or
over-restrictive string is not validated at write time and would silently deny SAMRPC to principals
that need it. Ship the exact CIS string as a fixed option value and never let the user type an
arbitrary SDDL. One evidence note: the proposal cited "verified absent on 26100.4061" from the research
machine, which is not admissible under the machine rule, but Microsoft's page carries the default claim
independently so the conclusion stands.

**Ready-to-paste info block:**

```yaml
    info: |
      **Limits who can enumerate this PC's local users and groups over the network to administrators.**

      ## What it does
      Writes the security descriptor `O:BAG:BAD:(A;;RC;;;BA)` to `RestrictRemoteSam` under
      `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`, as a `REG_SZ` string. Only Built-in Administrators
      may then make remote SAM RPC calls, which is what BloodHound-style tooling uses to harvest local
      user and group membership from an ordinary account.

      ## Benefits
      - **Defeats authenticated recon**: covers a different attack from the anonymous-enumeration
        controls, which only stop unauthenticated callers
      - **Standard baseline value**: the SDDL is the exact CIS and DISA STIG string
      - **Pins a good state**: locks the descriptor so a later misconfiguration cannot loosen it

      ## Drawbacks
      - **Inventory agents lose visibility**: non-admin asset-management agents that enumerate local
        groups stop working, which matters on domain-joined machines
      - **No visible change at home**: on Windows 10 1607 and later the built-in default already
        restricts remote SAM calls to administrators
      - **Type-sensitive and unvalidated**: it must be `REG_SZ` (a DWORD is ignored), and a malformed
        SDDL is not checked at write time and would silently deny access

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1607 and later
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, restoring Windows' built-in default descriptor
      - It is a different control from the anonymous-enumeration tweak, not a duplicate of it
      - No lockout risk for local sign-in

      ## Recommendation
      Apply it. It is free on a consumer machine and it closes an authenticated reconnaissance path
      the anonymous controls do not cover. Skip it only if a non-admin inventory agent depends on
      enumerating local groups.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Network access: Restrict clients allowed to make remote calls to SAM](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-restrict-clients-allowed-to-make-remote-sam-calls)
      - [DISA STIG for Windows 11 V2R2, remote SAM rule](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. Network access: Restrict clients allowed to make remote calls to SAM, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-restrict-clients-allowed-to-make-remote-sam-calls (tier A)
2. DISA STIG for Windows 11 V2R2 and CIS Windows 11 v4.0.0 Level 1 (tier B, the SDDL string)
3. `src-tauri/tweaks/security.yaml`, `restrict_anonymous_enum` effects (dedupe evidence)

### `block_always_install_elevated` Block AlwaysInstallElevated

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Confirmed verbatim in the shipped `MSI.admx` on 26100:

```xml
<policy name="AlwaysInstallElevated" class="Both"
        key="Software\Policies\Microsoft\Windows\Installer"
        valueName="AlwaysInstallElevated">
  <enabledValue><decimal value="1" /></enabledValue>
  <disabledValue><decimal value="0" /></disabledValue>
</policy>
```

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Installer` | `AlwaysInstallElevated` | REG_DWORD | `0` | `absent` |
| `HKCU\SOFTWARE\Policies\Microsoft\Windows\Installer` | `AlwaysInstallElevated` | REG_DWORD | `0` | `absent` |

Class is `Both`, so both hives are real targets. Stock default is value-absent in both.

CORRECTED reasoning: the gap proposal said "Class is Both in `MSI.admx`, so both hives must be written
for the control to be effective." That is backwards. The escalation exists only when **both** hives are
set to 1, so writing `0` to HKLM alone already defeats it. Writing HKCU as well is defence in depth, not
a requirement. There is also a scope limit the proposal does not state: an HKCU write only pins the hive
of the user the tweak runs as, so on a multi-user machine other users' hives are untouched. Harmless,
because HKLM = 0 is sufficient, but the copy must not imply machine-wide HKCU coverage.

**Corrections needed:** `none` to the mechanism, one to the reasoning: restate why both hives are
written.

**Ready-to-paste info block:**

```yaml
    info: |
      **Pins off a classic escalation where any user can run an installer as SYSTEM.**

      ## What it does
      Writes `AlwaysInstallElevated` = 0 in both the machine and user Windows Installer policy keys.
      When that setting is turned on in both hives, any user can run an arbitrary MSI with SYSTEM
      privileges. Pinning it to 0 makes the escalation impossible regardless of what else sets it.

      ## Benefits
      - **Closes a well-known escalation**: it is one of the first things local privilege-escalation
        tooling checks for
      - **Nothing warns about it**: Windows gives no indication when the setting has been turned on
      - **Cannot break a working install**: a legitimate installer never depends on the escalation

      ## Drawbacks
      - **No visible change**: the value is absent by default, so on a clean machine this pins an
        existing state rather than changing behaviour
      - **HKCU covers one user**: the per-user write pins only the hive of the account the tweak runs
        as, so other users on the machine are untouched
      - **Deployment scripts may fight it**: some poorly written enterprise deployment scripts set
        this intentionally and would break

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes both values, which is the shipped state
      - The escalation only exists when **both** hives are 1, so the HKLM write alone is sufficient
        and the HKCU write is defence in depth

      ## Recommendation
      Apply it. It costs nothing, cannot break a working installer, and defends against a setting
      that a surprising number of deployment scripts turn on and never turn back off.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [MSI.admx policy AlwaysInstallElevated (shipped ADMX)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-configuration-service-provider)
      - [DISA STIG for Windows 11 V2R2, always install with elevated privileges](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\MSI.admx` (26100), policy `AlwaysInstallElevated` (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2, "The Windows Installer feature 'Always install with elevated privileges' must be disabled", and CIS Windows 11 v4.0.0 Level 1 (tier B)
3. privacy.sexy `windows.yaml`, `AlwaysInstallElevated` = 0 with `deleteOnRevert: true` (tier C, corroboration of the revert-by-delete shape)

### `kernel_dma_protection` Kernel DMA protection policy

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:** Two REG_DWORD values in two different policy keys.

| Key | Value name | Type | Block all | Allow after sign-in | Stock Default |
|---|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Kernel DMA Protection` | `DeviceEnumerationPolicy` | REG_DWORD | `0` | `1` | `absent`, effective **`1`** |
| `HKLM\SOFTWARE\Policies\Microsoft\FVE` | `DisableExternalDMAUnderLock` | REG_DWORD | `1` | `1` | `absent` |

`DmaGuard.admx` on 26100 confirms the enum with an unusually helpful in-file comment: item value `0` is
"Block all", `1` is "Only while logged in", `2` is "Allow all". `VolumeEncryption.admx` confirms
`DisableExternalDMAUnderLock`, class Machine, enabledValue 1 / disabledValue 0,
`supportedOn ref="windows:SUPPORTED_Windows_10_0_RS2"`.

CORRECTED stock default: the shipped ADML resolves the option strings as
`DmaGuardEnumerationPolicy_Options_1 = "Allow all"`,
`DmaGuardEnumerationPolicy_Options_2 = "Only while logged in (default)"`,
`DmaGuardEnumerationPolicy_Options_3 = "Block all"`. Microsoft labels **"Only while logged in"** the
default, and that string maps to registry value **1**, not 2. Microsoft Learn agrees behaviourally: "By
default, peripherals with DMA Remapping incompatible drivers are blocked from starting and performing
DMA until an authorized user signs into the system or unlocks the screen." A revert writing 2 would
therefore be **less** protective than stock. The gap proposal's "effective default 2 (allow after sign
in)" was wrong on both halves and contradicted its own enum table.

The wrong one proposed, shown for comparison: stock default described as "value-absent, effective
default `2` (allow after sign-in)".

CORRECTED metadata: Policy CSP DmaGuard states "This policy requires a system reboot to take effect", so
`requires_reboot: true` is mandatory, not optional. The CSP also gives Windows 10 1809 (10.0.17763) and
later, editions Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. **Not Home.** The
ADMX `supportedOn` of `SUPPORTED_Windows_10_0` is looser than the CSP page; use 1809 as the floor.

Scope of the second value is narrower than the proposal claimed. The shipped ADML: "This policy setting
allows you to block direct memory access (DMA) for all **Thunderbolt hot pluggable PCI downstream
ports** until a user logs into Windows... Devices which were already enumerated when the machine was
unlocked will continue to function until unplugged or the system is rebooted or hibernated. **This
policy setting is only enforced when BitLocker or device encryption is enabled.**" So it is BitLocker
**or device encryption**, which matters on consumer 24H2 machines where automatic device encryption is
common, and it is Thunderbolt hot-plug ports specifically, not all external DMA.

**Corrections needed:** (1) Stock default for `DeviceEnumerationPolicy` is 1, not 2, and a revert must
not write 2. (2) Set `requires_reboot: true`. (3) Gate to Pro and above from Windows 10 1809. (4) Probe
the actual Kernel DMA Protection state (`msinfo32` or the `Win32_DeviceGuard` security properties) and
report "not applicable" rather than claiming success on a machine with no firmware DMA remapping; this
is a requirement, not a suggestion. (5) Note that the "Allow only after sign-in" option (value 1) is
identical to the Windows default, so only "Block all" changes behaviour; keep the middle option as an
explicit pin but say so in Drawbacks.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks external Thunderbolt and PCIe peripherals from touching memory before you sign in.**

      ## What it does
      Sets `DeviceEnumerationPolicy` = 0 ("Block all") in the Kernel DMA Protection policy key so
      external peripherals whose drivers are not DMA-remapping compatible never start, and
      `DisableExternalDMAUnderLock` = 1 in the BitLocker policy key so Thunderbolt hot-plug ports get
      no DMA while the machine is locked.

      ## Benefits
      - **Stops drive-by DMA**: a malicious Thunderbolt or PCIe device plugged into a locked laptop
        cannot read memory
      - **Covers the locked state too**: the second value blocks new Thunderbolt hot-plugs while
        locked, not only before first sign-in
      - **No effect on internal hardware**: the internal keyboard, display and storage are untouched

      ## Drawbacks
      - **External GPUs and docks can stop working**: eGPU enclosures, some Thunderbolt docks and
        older PCIe capture cards refuse to start under "Block all"
      - **Lid-closed dock scenario**: if a dock supplying your only keyboard, mouse and display is
        DMA-remapping incompatible and gets blocked, you lose input until you open the lid
      - **Inert on many machines**: it needs firmware DMA remapping (VT-d or AMD IOMMU) present at
        manufacture, is not available on Home, and does not apply to 1394, PCMCIA or ExpressCard
      - **Second value needs encryption**: `DisableExternalDMAUnderLock` is only enforced when
        BitLocker or device encryption is on

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, Pro / Enterprise / Education / IoT Enterprise;
        also Windows 10 1809 and later. Not Home
      - **Takes effect**: after reboot
      - **Reverting**: deletes both values so the Windows default applies again; the default is
        "Only while logged in" (value 1), so a revert must never write 2 ("Allow all")
      - The "Allow only after sign-in" option matches the Windows default and changes nothing
      - Kernel DMA Protection itself cannot be turned on by policy; the platform either supports it
        or it does not

      ## Recommendation
      Apply "Block all" on a laptop you carry, where the drive-by DMA threat is real. On a desktop
      with an eGPU or a capture card, use the sign-in option or leave it alone.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP DmaGuard](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-dmaguard)
      - [Kernel DMA Protection](https://learn.microsoft.com/en-us/windows/security/hardware-security/kernel-dma-protection-for-thunderbolt)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\DmaGuard.admx` and `en-US\DmaGuard.adml` (26100), policy `DmaGuardEnumerationPolicy` including the in-file comment mapping options to registry values (tier A)
2. `C:\Windows\PolicyDefinitions\VolumeEncryption.admx` and `en-US\VolumeEncryption.adml` (26100), policy `DisableExternalDMAUnderLock_Name` (tier A)
3. Policy CSP DmaGuard, applicability, editions and reboot requirement, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-dmaguard (tier A)
4. Kernel DMA Protection, "How Windows protects against DMA drive-by attacks", https://learn.microsoft.com/en-us/windows/security/hardware-security/kernel-dma-protection-for-thunderbolt (tier A)

### `asr_extended_rules` ASR extended rule set

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Six REG_SZ values under
`HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`, plus the
ADMX parent enabling value `ExploitGuard_ASR_Rules` = 1 (REG_DWORD) at `...\Exploit Guard\ASR`. Block =
`"1"`, Audit = `"2"`, stock default `absent` for every one.

| GUID | Microsoft rule name | Dependencies per Microsoft |
|---|---|---|
| `d3e037e1-3eb8-44c8-a917-57927947596d` | Block JavaScript or VBScript from launching downloaded executable content | Defender AV, AMSI |
| `92e97fa1-2edf-4476-bdd6-9dd0b4dddc7b` | Block Win32 API calls from Office macros | Defender AV, AMSI |
| `b2b3f03d-6a65-4f7b-a9c7-1c7ef74a9ba4` | Block untrusted and unsigned processes that run from USB | Defender AV |
| `c1db55ab-c21a-4637-bb3f-a12568109d35` | Use advanced protection against ransomware | Defender AV, **cloud-delivered protection** |
| `d1e49aac-8f56-4280-b9ba-993a6d77406c` | Block process creations originating from PSExec and WMI commands | Defender AV |
| `26190899-1602-49e8-8b27-eb1d0a1ce869` | Block Office communication application from creating child processes | Defender AV |

All six GUIDs were checked verbatim against Microsoft's ASR rules reference. None of them appears
anywhere in the existing corpus, which ships exactly five ASR GUIDs (`9e6c4e1f`, `d4f940ab`,
`3b576869`, `5beb7efe`, `be9ba2d9`), so there is no overlap.

**Corrections needed:** `none` to the mechanism. Three Microsoft-documented constraints are missing from
the proposal's copy and must be added: (1) `c1db55ab` requires cloud-delivered protection, so without
`defender_cloud_protection` the user gets a rule that is configured and non-functional; either gate it
or say so plainly. (2) `d3e037e1` in Block or Warn mode generates EDR alerts only when the device cloud
protection level is High Plus or Zero Tolerance; blocking still works, alerting does not. (3)
`26190899` "is enforced only if Office is installed in `%ProgramFiles%` or `%ProgramFiles(x86)%`", so
Microsoft Store and per-user Office installs are outside it and the rule is inert on a large share of
consumer machines. The proposal calls it "Outlook-specific"; it is narrower than that.

**Ready-to-paste info block:**

```yaml
    info: |
      **Adds six more Defender rules covering scripts, macros, USB, ransomware and PsExec.**

      ## What it does
      Enables six further attack surface reduction rules in Block mode: script-launched downloads,
      Win32 API calls from Office macros, untrusted processes running from USB, advanced ransomware
      protection, process creation from PsExec and WMI, and Office communication apps creating child
      processes.

      ## Benefits
      - **Broad coverage in one setting**: six distinct initial-access and lateral-movement
        behaviours, all through one registry key
      - **Audit mode available**: every rule accepts "2" so you can watch before enforcing
      - **No overlap with the existing rules**: none of the six GUIDs is already in the corpus

      ## Drawbacks
      - **PsExec and WMI rule is the risky one**: `d1e49aac` blocks processes spawned by PsExec and
        by WMI `Win32_Process.Create`, which legitimate management and automation tooling uses
      - **Ransomware rule needs cloud protection**: `c1db55ab` does nothing unless cloud-delivered
        protection is on, and it errs on the side of caution, so brand-new indie software can trip it
      - **Office rule often inert**: `26190899` is enforced only when Office is installed under
        `%ProgramFiles%` or `%ProgramFiles(x86)%`, which excludes Store and per-user installs
      - **Alerting is gated separately**: `d3e037e1` raises EDR alerts only at cloud protection level
        High Plus or Zero Tolerance, though the blocking itself always works

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10. Requires Microsoft Defender
        Antivirus as the active antivirus, and AMSI for two of the rules
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all six rule values and the policy's enabling value
      - Audit all six first if you use any scripted automation; switching to Block afterwards is one
        value change

      ## Recommendation
      Worth it on a consumer machine, where these behaviours are almost always malicious. Audit first
      if you use PsExec or WMI-driven automation, and skip the ransomware rule unless cloud
      protection is on.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [ASR rules reference, per-rule detail sections](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference)
      - [ASR rules overview](https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview)
```

**Sources:**
1. Attack surface reduction (ASR) rules reference, per-rule detail sections, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policy `ExploitGuard_ASR_Rules` (tier A)
3. `src-tauri/tweaks/security.yaml` (corpus convention and dedupe evidence)

### `ntlm_outgoing_restriction` Restrict outgoing NTLM

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism (CORRECTED):** Four REG_DWORD values under
`HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0`.

| Value name | Type | Hardened | Stock Default |
|---|---|---|---|
| `AuditOutgoingNTLMTraffic` | REG_DWORD | `2` (audit all) | `absent` |
| `RestrictSendingNTLMTraffic` | REG_DWORD | `2` (deny all) | `absent` |
| `NtlmMinClientSec` | REG_DWORD | `0x20080000` | **`absent`**, effective `0x20000000` |
| `NtlmMinServerSec` | REG_DWORD | `0x20080000` | **`absent`**, effective `0x20000000` |

Microsoft's Policy CSP documents the session-security values explicitly: `0` None, `0x00080000`
(524288) Require NTLMv2 session security, `0x20000000` (536870912) **Require 128-bit encryption
(Default)**, `0x20080000` (537395200) Require NTLM and 128-bit encryption.

The wrong reasoning in the gap proposal, shown for comparison: "the stock 26100 value `0x20000000`
requires NTLMv2 session security but **not** 128-bit encryption. `0x20080000` adds
`NTLMSSP_NEGOTIATE_128`." The two bits are transposed. `0x20000000` **is** the 128-bit bit and is
already the default; what `0x20080000` adds is the NTLMv2 session-security bit. The hardened value stays
`0x20080000`, which is what CIS and the STIG ask for, but the reason is the opposite of what was
written.

CORRECTED revert: Microsoft documents 536870912 as the **effective** default, not as a value
necessarily present in the registry, so the Stock Default option must write `absent` for
`NtlmMinClientSec` and `NtlmMinServerSec`. Writing the number back would pin a value the OS did not
have, which is the corpus's most damaging defect class.

Sourcing note for the copy: Microsoft's Security Policy Settings page for *Network security: Restrict
NTLM: Outgoing NTLM traffic to remote servers* confirms the three states (Allow all, Audit all, Deny
all) and that "not defined" behaves as Allow all, which validates the `absent` revert. It does **not**
publish the registry value name or the numeric mapping. `RestrictSendingNTLMTraffic` and
`AuditOutgoingNTLMTraffic` with 0 / 1 / 2 come from the DISA STIG and CIS check text (tier B), and
neither name appears in any of the 218 shipped ADMX files, consistent with them being Security Options
rather than administrative templates.

**Corrections needed:** (1) Rewrite the session-security rationale; the bits are transposed. (2) Change
the revert for `NtlmMinClientSec` and `NtlmMinServerSec` to `absent`. (3) Label the two outgoing-NTLM
value names as benchmark-sourced rather than Microsoft-documented.

**Ready-to-paste info block:**

```yaml
    info: |
      **Audits or refuses outgoing NTLM, and requires NTLMv2 session security on top of 128-bit
      encryption.**

      ## What it does
      Writes four values under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0`.
      `AuditOutgoingNTLMTraffic` = 2 logs every outgoing NTLM attempt;
      `RestrictSendingNTLMTraffic` = 2 refuses them; `NtlmMinClientSec` and `NtlmMinServerSec` =
      `0x20080000` require **both** 128-bit encryption and NTLMv2 session security, where Windows
      requires only the 128-bit half by default.

      ## Benefits
      - **Audit before you break anything**: the audit value alone tells you exactly what still uses
        NTLM before you deny it
      - **A genuine one-bit hardening**: the session-security values add the NTLMv2 requirement that
        the Windows default leaves off
      - **Complements SMB NTLM blocking**: this is the OS-level control, not just the SMB client

      ## Drawbacks
      - **Deny mode is aggressive on a workgroup**: with `RestrictSendingNTLMTraffic` = 2, any
        resource reachable only by NTLM stops working, including most consumer NAS devices
      - **Value names are benchmark-sourced**: Microsoft documents the policy and its three states but
        not the registry names, which come from the DISA STIG and CIS check text
      - **Session-security values are nearly free**: on any network without pre-Vista hosts the two
        session-security values change almost nothing

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all four values; the session-security default is an effective value,
        not a value present in the registry, so the revert must not write `0x20000000` back
      - None of these can lock you out of the local machine
      - Start with audit mode; move to deny only after the log is quiet

      ## Recommendation
      Apply the audit option on any machine so you can see what still needs NTLM. Move to deny only
      on a machine whose resources are all Kerberos-capable; on a typical home network deny will
      break your NAS.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP LocalPoliciesSecurityOptions, minimum session security for NTLM SSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-localpoliciessecurityoptions)
      - [Network security: Restrict NTLM: Outgoing NTLM traffic to remote servers](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-restrict-ntlm-outgoing-ntlm-traffic-to-remote-servers)
```

**Sources:**
1. Policy CSP LocalPoliciesSecurityOptions, `NetworkSecurity_MinimumSessionSecurityForNTLMSSPBasedClients` and `...Servers` (tier A)
2. Network security: Restrict NTLM: Outgoing NTLM traffic to remote servers, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-restrict-ntlm-outgoing-ntlm-traffic-to-remote-servers (tier A)
3. DISA STIG for Windows 11 V2R2 (tier B, the two registry value names)
4. All-ADMX scan across the 218 shipped `.admx` files on 26100 (tier A, negative result: neither name is an administrative template)

### `rdp_session_hardening` Harden the RDP session

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Five values, all confirmed verbatim in `TerminalServer.admx` on 26100, key
`HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` in every case, all REG_DWORD, all stock
default `absent`.

| Value name | ADMX policy | Class | Hardened | Meaning |
|---|---|---|---|---|
| `MinEncryptionLevel` | `TS_ENCRYPTION_POLICY` | Machine | `3` | enum: 1 Low, 2 Client Compatible, 3 High Level |
| `fEncryptRPCTraffic` | `TS_RPC_ENCRYPTION` | Machine | `1` | require secure RPC for the session host |
| `fPromptForPassword` | `TS_PASSWORD` | Machine | `1` | always prompt for password on connection |
| `DisablePasswordSaving` | `TS_CLIENT_DISABLE_PASSWORD_SAVING_1` and `_2` | **User and Machine** | `1` | the RDP client cannot save passwords |
| `fDisableCdm` | `TS_CLIENT_DRIVE_M` | Machine | `1` | block local drive redirection into RDP sessions |

**Corrections needed:** `none` to the mechanism, three to the copy. (1) The proposal says "all class
Machine". `DisablePasswordSaving` is defined twice, once class User (`_1`) and once class Machine
(`_2`), both against the same key; the HKLM write is the correct and effective one, so this is a
documentation fix rather than a defect. (2) `MinEncryptionLevel` = 3 is the **maximum** the shipped ADMX
offers; there is no FIPS (4) item in the 26100 enum and no `TS_ENCRYPTION_FIPS_LEVEL` string in the
ADML, so any option list must stop at 3. (3) A real overlap: `MinEncryptionLevel` governs the legacy RDP
security layer, and the corpus's `rdp_security_hardening` forces TLS, under which the encryption level
is negotiated by TLS rather than by this value. Applying both leaves `MinEncryptionLevel` largely inert.
That is not a reason to drop it (it is still the STIG check), but the copy must not imply it is doing
work it is not.

**Ready-to-paste info block:**

```yaml
    info: |
      **Tightens five Remote Desktop session settings beyond the NLA and TLS basics.**

      ## What it does
      Writes five policy values under
      `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services`: require High encryption
      (`MinEncryptionLevel` = 3), require secure RPC (`fEncryptRPCTraffic` = 1), always prompt for a
      password (`fPromptForPassword` = 1), stop the RDP client saving passwords
      (`DisablePasswordSaving` = 1) and block local drive redirection (`fDisableCdm` = 1).

      ## Benefits
      - **No saved RDP passwords**: removes a credential store an attacker can harvest from this PC
      - **No drive redirection**: a compromised remote host cannot reach into your local drives
      - **Always prompts**: a cached or passed-through credential cannot be used silently

      ## Drawbacks
      - **File transfer breaks**: `fDisableCdm` stops copying files through a mapped drive inside an
        RDP session, which many people rely on; keep it as a separable effect
      - **Encryption level is largely inert under TLS**: if you also force TLS with the NLA tweak,
        TLS negotiates the encryption and `MinEncryptionLevel` mostly does not decide anything
      - **Client-side effect too**: `DisablePasswordSaving` affects this machine connecting **out**,
        not just inbound sessions
      - **Inert without RDP**: all five do nothing unless Remote Desktop is in use

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, Pro and above; the ADMX `supportedOn` reaches back
        to Windows XP, so no build gate is needed
      - **Takes effect**: immediately, on the next connection
      - **Reverting**: deletes all five values, restoring the shipped defaults
      - Value 3 is the maximum the shipped ADMX offers; there is no FIPS level on 26100
      - This tweak and disabling Remote Desktop entirely are opposite postures; pick one

      ## Recommendation
      Apply it if you use Remote Desktop and want the STIG-level session settings. If you do not use
      RDP at all, disable the listener instead; these five will do nothing for you.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [TerminalServer.admx policies (shipped ADMX, 26100)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-remotedesktopservices)
      - [DISA STIG for Windows 11 V2R2, five Remote Desktop rules](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\TerminalServer.admx` and `en-US\TerminalServer.adml` (26100), policies `TS_ENCRYPTION_POLICY`, `TS_RPC_ENCRYPTION`, `TS_PASSWORD`, `TS_CLIENT_DISABLE_PASSWORD_SAVING_1` / `_2`, `TS_CLIENT_DRIVE_M` (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2, five separate Remote Desktop rules (tier B)
3. `src-tauri/tweaks/security.yaml`, `rdp_security_hardening` (overlap evidence)

### `disable_secondary_logon` Disable the Secondary Logon service

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Service effect on `seclogon` (display name "Secondary Logon"). Hardened start type
`disabled`; stock start type restored from the snapshot.

`seclogon` is present on 26100 with `seclogon.dll` in System32. The SCM dependency graph shows no
dependent services and no service dependencies, so disabling it does not cascade. It is genuinely absent
from the corpus: `services.yaml` ships 29 service tweaks and none targets `seclogon`. This is a
standalone DISA STIG rule ("The Secondary Logon service must be disabled on Windows 11").

**Corrections needed:** `none` to the mechanism, one to the revert and one to the risk copy. The shipped
default start type could not be established from an authoritative source. The proposal says Manual,
which is consistent with the STIG having a rule requiring it be set to Disabled (pointless if it already
shipped Disabled), but it was not confirmed against a clean image, so **restore the start type from the
snapshot rather than hardcoding Manual**. Recorded in UNKNOWNS. On risk, the proposal is slightly
optimistic: beyond `runas` and the Shift plus right-click "Run as different user" verb, `seclogon` backs
`CreateProcessWithLogonW`, which some installers, deployment tools and scripted automation use directly.

**Ready-to-paste info block:**

```yaml
    info: |
      **Disables the service behind `runas` and "Run as different user".**

      ## What it does
      Sets the `seclogon` service start type to Disabled. Secondary Logon is what lets a process be
      started under a different user account, through `runas`, the Shift plus right-click "Run as
      different user" shell verb, or the `CreateProcessWithLogonW` API.

      ## Benefits
      - **Removes an escalation step**: running a process as another user is a routine move in local
        privilege-escalation and lateral-movement chains
      - **Nothing cascades**: no other service depends on it, so disabling it is contained
      - **Rarely used on a consumer machine**: single-account users never touch it

      ## Drawbacks
      - **`runas` stops working**: as does the "Run as different user" menu entry
      - **Some installers break**: anything calling `CreateProcessWithLogonW` directly, which includes
        a few deployment tools and scripted automation
      - **Bites admins**: if you keep a separate admin account, or administer other machines from
        this one, you will notice immediately

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: restores the previous start type from the snapshot; the shipped start type is
        not established by any authoritative source, so it must not be hardcoded
      - Normal UAC elevation of the **same** user is unaffected, so the everyday admin experience
        does not change

      ## Recommendation
      Apply it on a single-account consumer machine, where it is essentially never used. Skip it if
      you keep a separate administrator account or use `runas` as part of your workflow.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Community-corroborated
      - [DISA STIG for Windows 11 V2R2, Secondary Logon rule](https://www.stigviewer.com/stigs/microsoft_windows_11)
      - [CreateProcessWithLogonW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw)
```

**Sources:**
1. DISA STIG for Windows 11 V2R2, "The Secondary Logon service must be disabled on Windows 11" (tier B)
2. Live service presence and SCM dependency graph on 26100 (tier A for existence only, not for the default start type)
3. `src-tauri/tweaks/services.yaml` (dedupe evidence); privacy.sexy and Sophia Script (tier C, corroboration)

### `early_launch_antimalware_policy` Early Launch Antimalware driver policy

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:** `HKLM\SYSTEM\CurrentControlSet\Policies\EarlyLaunch` -> `DriverLoadPolicy` (REG_DWORD),
class Machine, `supportedOn ref="windows:SUPPORTED_Windows8"`. Stock default `absent`, effective `3`.
Requires reboot.

CORRECTED enum. The shipped `EarlyLaunchAM.admx` on 26100 defines exactly four members:

| Value | ADML display name |
|---|---|
| **`8`** | Good only |
| `1` | Good and unknown |
| `3` | Good, unknown and bad but critical |
| `7` | All (no filtering) |

The wrong one in the gap proposal, shown for comparison: "Accepted values: `0` Good only, `1` Good and
unknown, `3` ..., `7` All". **"Good only" is `8`, not `0`, and `0` is not a member of the enum at all.**
The proposal then spent a paragraph warning against offering a value that cannot be selected. The
warning content is right; it just needs to attach to `8`.

**Corrections needed:** Offer `3` (Good, unknown and bad but critical) and `1` (Good and unknown) only.
Do **not** offer `8`: it refuses to boot-load any driver ELAM cannot vouch for and on unusual hardware
can produce a machine that will not start, which violates the "must not brick" rule. Do not offer `7`
either, since that is "no filtering" and is the state malware wants.

**Ready-to-paste info block:**

```yaml
    info: |
      **Pins the boot-time driver policy so malware cannot loosen it to load an unsigned driver.**

      ## What it does
      Sets `DriverLoadPolicy` = 3 under `HKLM\SYSTEM\CurrentControlSet\Policies\EarlyLaunch`. That is
      the Early Launch Antimalware setting "Good, unknown and bad but critical": at boot, Windows
      loads drivers the ELAM driver classifies as good or unknown, plus bad-but-boot-critical ones,
      and refuses the rest.

      ## Benefits
      - **Stops the loosening attack**: without the value pinned, malware can set `7` ("no
        filtering") and get an unsigned boot driver loaded
      - **Auditable**: it is an explicit STIG check with a specific value
      - **Safe value**: 3 is what Windows already does, so nothing legitimate stops loading

      ## Drawbacks
      - **No behavioural change**: `3` is the effective default, so applying it pins rather than
        alters
      - **Needs a reboot**: the policy is read at boot
      - **Stricter values are dangerous**: "Good only" is value `8` and will refuse any driver ELAM
        cannot vouch for, which on unusual hardware can leave a machine that does not start

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and anything from Windows 8
      - **Takes effect**: after reboot
      - **Reverting**: deletes the value, which is the shipped state
      - The enum members are `8` Good only, `1` Good and unknown, `3` Good, unknown and bad but
        critical, `7` All; value `0` does not exist
      - The DISA STIG accepts `3` or `1`

      ## Recommendation
      Apply it at value 3 on any machine. Choose 1 only if you want to refuse known-bad boot-critical
      drivers as well and are willing to risk a boot failure on unusual hardware.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Early Launch Antimalware](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/early-launch-antimalware)
      - [DISA STIG for Windows 11 V2R2, ELAM boot-start driver policy](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\EarlyLaunchAM.admx` (26100), policy `POL_DriverLoadPolicy_Name`, verbatim (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2 and CIS Windows 11 v4.0.0 Level 1 (tier B)

### `credssp_encryption_oracle` CredSSP encryption oracle remediation

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:**

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\CredSSP\Parameters` | `AllowEncryptionOracle` | REG_DWORD | `0` (Force Updated Clients) | `absent`, effective **`1` (Mitigated)** |

Confirmed in `CredSsp.admx` on 26100: policy `AllowEncryptionOracle`, class Machine, key
`Software\Microsoft\Windows\CurrentVersion\Policies\System\CredSSP\Parameters`, enum with `0` Force
Updated Clients, `1` Mitigated, `2` Vulnerable. The Microsoft KB for CVE-2018-0886 confirms the same
three values with the same names. `supportedOn` is `SUPPORTED_WindowsVista`, so no build gate. Not
already in the corpus.

CORRECTED stock default. The gap proposal claimed the effective default is `2` (Vulnerable) and reasoned
that "the OS still ships with the permissive behaviour so that unpatched RDP servers remain reachable".
Microsoft's KB states plainly: **"May 8, 2018. An update to change the default setting from Vulnerable
to Mitigated."** and "By default, after this update is installed, patched clients cannot communicate
with unpatched servers." Any 24H2 machine has been at Mitigated for years.

That materially shrinks the tweak. Going from the effective default `1` to the hardened `0` changes only
the **server** half of the behaviour: with `1`, services that use CredSSP accept unpatched clients; with
`0`, they do not. The client half (no fallback to insecure versions) is already in force at the default,
so the proposal's stated risk (outbound RDP to an old unpatched server starts failing) describes what
the machine already does and is smaller than claimed.

**Corrections needed:** Correct the stock default to `1` (Mitigated) and rewrite the rationale and risk
copy accordingly. The revert stays `absent`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Refuses CredSSP connections from clients that never got the 2018 encryption-oracle fix.**

      ## What it does
      Sets `AllowEncryptionOracle` = 0 ("Force Updated Clients") under the CredSSP policy key. CredSSP
      is the protocol behind Remote Desktop Network Level Authentication. Value 0 means services on
      this machine that use CredSSP reject any client still running the vulnerable pre-May-2018
      behaviour from CVE-2018-0886.

      ## Benefits
      - **Closes the remaining half**: the client side has been mitigated since 2018, but the server
        side still accepts unpatched clients at the default
      - **Explicit and auditable**: it is a CIS Level 1 check with a specific value
      - **No effect on normal use**: every supported Windows client is patched

      ## Drawbacks
      - **Old clients cannot connect in**: anything running the pre-2018 CredSSP behaviour is refused
      - **Smaller change than it sounds**: the effective default has been Mitigated (1) since the May
        2018 update, so only the accept side moves
      - **Inert without CredSSP**: if nothing on this machine accepts CredSSP connections, nothing
        changes

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and back to Windows Vista
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, returning to the Mitigated default
      - The three values are `0` Force Updated Clients, `1` Mitigated (the current default) and
        `2` Vulnerable; never write 2

      ## Recommendation
      Apply it. On a modern network nothing legitimate is still unpatched, and it closes the accept
      side that the Windows default leaves open. Skip it only if you must accept RDP from a machine
      that has not been updated since 2018.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [CredSSP updates for CVE-2018-0886](https://support.microsoft.com/en-us/topic/credssp-updates-for-cve-2018-0886-5cbf9e5f-dc6d-744f-9e97-7ba400d6d3ea)
      - [CIS Windows 11 v4.0.0, encryption oracle remediation](https://www.cisecurity.org/benchmark/microsoft_windows_desktop)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\CredSsp.admx` (26100), policy `AllowEncryptionOracle` (tier A, shipped ADMX)
2. Microsoft KB "CredSSP updates for CVE-2018-0886", the changelog line and the three-option table (tier A)
3. CIS Windows 11 v4.0.0 (tier B)

### `device_encryption_posture` Automatic device encryption posture

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Microsoft Learn's BitLocker overview, section "Disable device encryption", gives the
fields exactly.

| Key | Value name | Type | Prevent auto-encryption | Stock Default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\BitLocker` | `PreventDeviceEncryption` | REG_DWORD | `1` | `absent` (automatic encryption allowed) |

Not present in any of the 218 shipped ADMX files, which is consistent: Microsoft documents it as a
direct registry setting rather than a policy. Not already in the corpus.

**Corrections needed:** `none` to the mechanism. One correction to applicability: the proposal gates on
`windows: { build: ">=26100" }`, which is too narrow. The registry value and automatic device
encryption long predate 24H2; what 24H2 changed was the hardware prerequisite, so many more machines
auto-encrypt. On a 19044 LTSC machine the value is read just the same, and gating it out hides a control
that works there. If a gate is wanted, gate the **copy** (say that 24H2 made this much more likely to
bite), not the mechanism. Note the awkwardness that a tweak whose hardened direction is *less*
protection sits oddly in a security category; the honest framing is user control over encryption and
recovery-key custody. Do not lead with performance: Microsoft shipped hardware-accelerated BitLocker
with 25H2, which materially shrinks the storage-overhead case.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops Windows automatically encrypting your drive and escrowing the recovery key.**

      ## What it does
      Sets `PreventDeviceEncryption` = 1 under `HKLM\SYSTEM\CurrentControlSet\Control\BitLocker`, the
      registry value Microsoft documents for opting out of automatic device encryption. Windows 11
      24H2 turns device encryption on by default on clean installs and qualifying new PCs, including
      Home, and escrows the recovery key to your Microsoft account.

      ## Benefits
      - **You choose encryption, not the installer**: automatic enablement stops
      - **Recovery-key custody stays with you**: no key is escrowed to a Microsoft account without a
        deliberate decision
      - **Documented opt-out**: this is Microsoft's own supported route, not a workaround

      ## Drawbacks
      - **It reduces data-at-rest protection**: an unencrypted drive can be read by anyone who takes
        it, and that is the whole point of this setting
      - **Only affects automatic enablement**: on a machine that is already encrypted it does
        nothing, so the tweak must check the actual volume state
      - **One-way in practice**: once device encryption is off it does not re-enable itself; you turn
        it on in Settings
      - **The performance argument has shrunk**: 25H2 shipped hardware-accelerated BitLocker, so the
        throughput cost on fast NVMe drives is much smaller than it was

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, where automatic device encryption is common; the
        value is also read on Windows 10 and LTSC 2021
      - **Takes effect**: immediately, but only for encryption that has not started yet
      - **Reverting**: deletes the value, allowing automatic device encryption again
      - Check the BitLocker volume state before and after; a machine that is already encrypted will
        show no change

      ## Recommendation
      Use it only if you deliberately want to manage encryption yourself and understand the trade.
      For a laptop that leaves the house, leave device encryption on: the data-at-rest protection is
      worth more than the control.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [BitLocker overview, "Disable device encryption"](https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/)
      - [Announcing hardware-accelerated BitLocker](https://techcommunity.microsoft.com/blog/windows-itpro-blog/announcing-hardware-accelerated-bitlocker/4474609)
```

**Sources:**
1. Microsoft Learn BitLocker overview, "Disable device encryption" table (path, name, type and value), https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/ (tier A)
2. Announcing hardware-accelerated BitLocker, https://techcommunity.microsoft.com/blog/windows-itpro-blog/announcing-hardware-accelerated-bitlocker/4474609 (tier B)
3. All-ADMX scan across the 218 shipped `.admx` files on 26100 (tier A, negative result: it is a direct registry setting, not a policy)

### `hide_admin_accounts_on_elevation` Hide admin accounts on the UAC prompt

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Two REG_DWORD values in **two different keys with two different classes**, both confirmed
verbatim in `CredUI.admx` on 26100.

| Key | Value name | ADMX class | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\CredUI` | `EnumerateAdministrators` | Machine | `0` | `absent` |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\CredUI` | `DisablePasswordReveal` | **Both** | `1` | `absent` |
| `HKCU\SOFTWARE\Policies\Microsoft\Windows\CredUI` | `DisablePasswordReveal` | **Both** | `1` | `absent` |

Polarity check: `EnumerateAdministrators` enabled (1) means *do* enumerate, so the hardened value is 0.
`DisablePasswordReveal` enabled (1) means *do* disable the reveal button, so the hardened value is 1.
Neither is inverted relative to the proposal.

Neither value appears anywhere in the corpus. `hide_last_user` covers `DontDisplayLastUserName`, a
different value in a different key, so these are complementary rather than overlapping.

**Corrections needed:** `none` to the mechanism. One addition: `DisablePasswordReveal` is class **Both**,
so the HKCU twin at `HKCU\SOFTWARE\Policies\Microsoft\Windows\CredUI` should be written as well and the
revert must remove both.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the UAC prompt listing every administrator account, and removes the password-reveal eye.**

      ## What it does
      Sets `EnumerateAdministrators` = 0 under the CredUI policies key so the credential prompt no
      longer lists local administrator accounts by name and picture, and `DisablePasswordReveal` = 1
      in both the machine and user CredUI policy keys so the eye icon that reveals a typed password
      disappears.

      ## Benefits
      - **No free account list**: anyone standing at the machine cannot read off which accounts are
        administrators
      - **Blocks shoulder surfing**: the reveal button cannot be used to display a typed password
      - **Complements hiding the last user**: different value, different key, additive

      ## Drawbacks
      - **More typing at the prompt**: you type the administrator username instead of clicking it
      - **No password verification**: without the reveal button, a mistyped long password is harder
        to spot
      - **No effect on remote attacks**: both are physical-presence protections only

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2. `EnumerateAdministrators`
        reaches back to Vista and `DisablePasswordReveal` to Windows 8
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes all three values, including the per-user copy
      - `DisablePasswordReveal` is a Both-class policy, so it is written to HKLM **and** HKCU

      ## Recommendation
      Apply it on any machine used somewhere other people can see the screen. On a private desktop
      the gain is small but the cost is only a little extra typing.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [CredUI.admx policies (shipped ADMX, 26100)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-credentialsui)
      - [DISA STIG for Windows 11 V2R2, administrator enumeration during elevation](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\CredUI.admx` (26100), policies `EnumerateAdministrators` and `DisablePasswordReveal`, verbatim (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2 ("Administrator accounts must not be enumerated during elevation") and CIS Windows 11 v4.0.0 Level 1 (tier B)
3. `src-tauri/tweaks/security.yaml`, `hide_last_user` (dedupe evidence)

### `event_log_retention` Event log retention size

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** `MaxSize` (REG_DWORD, kilobytes) under per-channel keys, confirmed in `EventLog.admx` on
26100. There are **four** channel policies, not three.

| ADMX policy | Key |
|---|---|
| `Channel_LogMaxSize_1` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Application` |
| `Channel_LogMaxSize_2` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Security` |
| `Channel_LogMaxSize_3` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Setup` |
| `Channel_LogMaxSize_4` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\System` |

Each is class Machine with a single element:
`<decimal id="Channel_LogMaxSize" valueName="MaxSize" required="true" minValue="1024" maxValue="2147483647" />`.

Proposed option values: Standard = Application 32768, System 32768, Security 196608. Large (STIG) =
Security 1024000. All are inside the declared range, so none can be silently rejected. Stock default is
`absent` for every channel. Not already in the corpus.

**Corrections needed:** `none` to the mechanism. Two copy fixes: (1) the proposal's "channel default
20480 KB" is asserted with no source and was not independently confirmed; it does not affect the revert
(which is correctly `absent`) but must not be stated as fact. (2) The Setup channel exists and is
omitted, so the entry should say "three of the four channels" rather than implying the set is complete.

**Ready-to-paste info block:**

```yaml
    info: |
      **Makes the event logs big enough that a busy day does not overwrite the evidence.**

      ## What it does
      Sets `MaxSize` (in kilobytes) under the Application, System and Security channel policy keys of
      `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog`. The standard option uses 32 MB for
      Application and System and 192 MB for Security; the large option raises Security to the STIG's
      roughly 1 GB.

      ## Benefits
      - **Auditing becomes useful**: turning on logon or process auditing without sizing the log is
        half a control
      - **Longer retention window**: incidents are often noticed days after they happen
      - **Safe values**: all three sizes sit inside the ADMX-declared range of 1024 to 2147483647 KB

      ## Drawbacks
      - **Uses disk**: the STIG's 1 GB Security log is a real allocation on a small SSD
      - **No security effect on its own**: it changes retention, not detection
      - **Covers three of four channels**: the Setup channel has its own policy and is not touched
      - **Managed machines override it**: an event-log GPO wins at the next refresh

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the values, returning each channel to its own default size
      - Pair this with the logon-auditing and PowerShell logging tweaks, which are what fill the logs

      ## Recommendation
      Apply the standard option if you have enabled any auditing or PowerShell logging. Take the
      large option only if you actually review the Security log and have the disk space.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [EventLog.admx channel log size policies (shipped ADMX, 26100)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-eventlogservice)
      - [DISA STIG for Windows 11 V2R2, event log size rules](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\EventLog.admx` (26100), all four `Channel_LogMaxSize_*` policy blocks verbatim, including `minValue="1024" maxValue="2147483647"` (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2, three separate event-log size rules (tier B)

### `audit_process_creation_cmdline` Command line in process-creation events

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** One registry value plus one `auditpol` action.

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\Audit` | `ProcessCreationIncludeCmdLine_Enabled` | REG_DWORD | `1` | `absent` |

Confirmed verbatim in `AuditSettings.admx` on 26100: policy `IncludeCmdLine`, class Machine,
`enabledValue` 1, `disabledValue` 0, `supportedOn ref="windows:SUPPORTED_Windows_6_3"` (Windows 8.1 and
later), so both target platforms qualify.

Action half: `auditpol /set /subcategory:"{0CCE922B-69AE-11D9-BED3-505054503030}" /success:enable`,
which is the Detailed Tracking > Process Creation subcategory. This follows the pattern the corpus
already uses in `audit_logon_events`, which uses `{0CCE9215-...}`, a different subcategory, so there is
no collision.

**Corrections needed:** `none` to the mechanism, one to the revert design that the proposal understates.
The registry value on its own logs nothing: it only adds the command line to event 4688, which is
generated only when the Process Creation subcategory is enabled. **`auditpol` state is not a registry
value**, so the snapshot and revert path must capture and restore the prior subcategory setting, not
just delete the DWORD. If the revert only deletes `ProcessCreationIncludeCmdLine_Enabled` it leaves
process-creation auditing switched on, which is a silent state leak.

**Ready-to-paste info block:**

```yaml
    info: |
      **Records the full command line of every process Windows starts, not just its name.**

      ## What it does
      Sets `ProcessCreationIncludeCmdLine_Enabled` = 1 under the audit policy key and enables the
      Detailed Tracking > Process Creation audit subcategory with `auditpol`. Event 4688 then carries
      the full command line, so the log says what was run rather than only that `powershell.exe` ran.

      ## Benefits
      - **Turns 4688 into evidence**: a process name alone tells you almost nothing
      - **The highest-value addition to the audit story**: it is two separate STIG rules
      - **Cheap**: one policy value plus one subcategory

      ## Drawbacks
      - **Command lines can contain secrets**: passwords passed as arguments end up in the Security
        log, readable by every administrator
      - **Log volume**: process creation is frequent, so pair it with a larger Security log
      - **The registry value alone does nothing**: without the audit subcategory enabled, event 4688
        is never generated
      - **Revert is not just a delete**: the audit subcategory state has to be restored separately or
        process-creation auditing stays on

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and back to Windows 8.1
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the policy value **and** restores the captured Process Creation
        subcategory setting from the snapshot
      - It uses a different audit subcategory from the logon-auditing tweak, so the two do not collide

      ## Recommendation
      Enable it if you have enabled logon auditing already; without command lines the process events
      are close to useless. Skip it if administrators on this machine should not see credentials that
      scripts pass as arguments.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Audit Process Creation](https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-process-creation)
      - [auditpol set](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/auditpol-set)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\AuditSettings.admx` (26100), policy `IncludeCmdLine`, verbatim (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2 ("Command line data must be included in process creation events" and the Detailed Tracking rule) (tier B)
3. `src-tauri/tweaks/security.yaml`, `audit_logon_events` (existing `auditpol` pattern, no subcategory collision)

### `spooler_remote_rpc_off` Turn off the spooler's remote RPC endpoint

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism (CORRECTED):**

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers` | `RegisterSpoolerRemoteRpcEndPoint` | REG_DWORD | `2` (do not accept client connections) | `absent`, effective accept |
| `HKLM\SYSTEM\CurrentControlSet\Control\Print` | `RpcAuthnLevelPrivacyEnabled` | REG_DWORD | `1` | `absent` |

The wrong one in the gap proposal, shown for comparison: `RegisterSpoolerRemoteRpcEndPoint` placed under
`HKLM\SYSTEM\CurrentControlSet\Control\Print`. The shipped `Printing2.admx` on 26100 puts it under the
**policy** key:

```xml
<policy class="Machine" name="RegisterSpoolerRemoteRpcEndPoint"
        key="Software\Policies\Microsoft\Windows NT\Printers"
        valueName="RegisterSpoolerRemoteRpcEndPoint">
  <enabledValue><decimal value="1"/></enabledValue>
  <disabledValue><decimal value="2"/></disabledValue>
</policy>
```

Written to `Control\Print` the value would land where the spooler policy path does not read it, the
probe would report the tweak applied, and nothing would change: a "did-it-work" contract violation. The
**value** is right: the policy is "Allow Print Spooler to accept client connections", enabled writes 1
(accept), disabled writes 2 (do not accept), and hardened 2 is correct.

The second value is correct as proposed: `Printing.admx`, policy `ConfigureRpcAuthnLevelPrivacyEnabled`,
class Machine, key `System\CurrentControlSet\Control\Print`, valueName `RpcAuthnLevelPrivacyEnabled`,
enabled 1 / disabled 0.

Neither value is in the corpus. `disable_print_spooler` (services) and `printnightmare_point_and_print`
(security) cover different surfaces, so the "middle option" framing is fair.

**Corrections needed:** Move `RegisterSpoolerRemoteRpcEndPoint` to
`HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers`. Add the detail the ADML explain text carries and
the proposal omits: "When the policy is disabled, the spooler will not accept client connections nor
allow users to share printers. **All printers currently shared will continue to be shared.** The spooler
must be restarted for changes to this policy to take effect."

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the print spooler accepting connections from the network while local printing keeps
      working.**

      ## What it does
      Sets `RegisterSpoolerRemoteRpcEndPoint` = 2 in the Printers policy key, which is the policy
      "Allow Print Spooler to accept client connections" in its Disabled state, and
      `RpcAuthnLevelPrivacyEnabled` = 1 under `Control\Print` so spooler RPC uses packet privacy.

      ## Benefits
      - **The middle option**: you keep local and network printing without exposing the spooler's
        remote RPC endpoint
      - **Covers the remote half of PrintNightmare**: the driver-installation restriction covers the
        other half
      - **Reversible in seconds**: two values plus a spooler restart

      ## Drawbacks
      - **No printing to this PC**: remote printing to this machine and sharing printers from it stop
        working
      - **Existing shares persist**: Microsoft's own explain text notes that printers already shared
        continue to be shared, so it is less complete than it sounds
      - **Needs a spooler restart**: the change is not live until the spooler restarts

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: after the Print Spooler service restarts
      - **Reverting**: deletes both values, restoring the shipped accept-connections behaviour
      - Printing **from** this PC to local and network printers is unaffected
      - If you do not print at all, disabling the Print Spooler service outright is stronger

      ## Recommendation
      Apply it on any machine that prints but never receives print jobs, which is nearly every home
      and office PC. Skip it if this machine shares a printer with others.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Printing2.admx, Allow Print Spooler to accept client connections (shipped ADMX, 26100)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-printers)
      - [Manage Windows print spooler RPC connection settings](https://learn.microsoft.com/en-us/troubleshoot/windows-server/printing/manage-windows-printer-rpc-connection-settings)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\Printing2.admx` and `en-US\Printing2.adml` (26100), policy `RegisterSpoolerRemoteRpcEndPoint`, verbatim including the explain text (tier A, shipped ADMX)
2. `C:\Windows\PolicyDefinitions\Printing.admx` (26100), policy `ConfigureRpcAuthnLevelPrivacyEnabled` (tier A, shipped ADMX)
3. CIS Windows 11 v4.0.0 and the DISA STIG Point and Print rules (tier B)

### `remote_uac_token_filter` Filter the remote local-admin token

**Verdict:** VERIFIED (new in this revision)

**Mechanism:**

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` | `LocalAccountTokenFilterPolicy` | REG_DWORD | `0` | `absent`, effective `0` |

Microsoft's "User Account Control and remote restrictions" article gives the semantics directly: `0`
"builds a filtered token. **It's the default value.** The administrator credentials are removed"; `1`
"builds an elevated token". Not present in any of the 218 shipped ADMX files, consistent with it being a
direct registry setting rather than a policy. Not already in the corpus.

**Why pin an existing default.** The value is absent by default, but it is set to `1` by a great many
"fix my network shares" guides, by some remote-support tools and by several popular tweak scripts. The
gap pass found it set to `1` on the research machine, which is the point: this is a control worth
pinning precisely because something will have flipped it.

**Not a duplicate, but a near-miss worth naming.** `security.yaml` already ships `filter_admin_token`,
which writes `FilterAdministratorToken` in the **same key**. Two similarly named values in one key
governing adjacent behaviour (built-in Administrator elevation versus network-logon token filtering).
If both ship, each info block must say what the other does, or users will assume one duplicates the
other and pick arbitrarily.

**Corrections needed:** `none`

**Ready-to-paste info block:**

```yaml
    info: |
      **Ensures a local admin connecting over the network gets a filtered token, not full rights.**

      ## What it does
      Pins `LocalAccountTokenFilterPolicy` = 0 under
      `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`. With that value at 1, a local
      administrator account connecting over the network receives a full unfiltered admin token, which
      is exactly what pass-the-hash lateral movement wants. At 0, Windows strips the administrator
      credentials from the network logon token.

      ## Benefits
      - **Removes a pass-the-hash target**: a stolen local admin hash no longer buys full remote
        admin rights
      - **Pins a state something else will have changed**: countless "fix my network shares" guides
        and tweak scripts set this to 1 and never reset it
      - **Matches Microsoft's own default**: value 0 is what Windows does when the value is absent

      ## Drawbacks
      - **Breaks remote administration with local accounts**: remote `C$` access, remote WMI and
        remote MMC using a local (non-domain) admin account start failing, which is the intended
        effect but a real one
      - **No visible change on a clean machine**: 0 is already the effective default
      - **Confusable neighbour**: `FilterAdministratorToken` lives in the same key and governs
        built-in Administrator elevation, which is a different thing

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, which behaves the same as 0
      - Domain accounts are unaffected; this governs local accounts over the network
      - Check this one before assuming it is already correct: it is very commonly set to 1

      ## Recommendation
      Apply it on any machine that does not need remote administration with a local account, which is
      almost every home PC. Skip it if you deliberately manage this machine remotely with a local
      admin account.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [User Account Control and remote restrictions](https://learn.microsoft.com/en-us/troubleshoot/windows-server/windows-security/user-account-control-and-remote-restriction)
      - [DISA STIG for Windows 11 V2R2, local administrator token filtering](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. Microsoft Learn "User Account Control and remote restrictions", the UAC remote settings table (tier A)
2. DISA STIG for Windows 11 V2R2 ("Local administrator accounts must have their privileged token filtered") and CIS Windows 11 v4.0.0 (tier B)
3. All-ADMX scan across the 218 shipped `.admx` files on 26100 (tier A, negative result); `src-tauri/tweaks/security.yaml`, `filter_admin_token` (near-miss evidence)

### `enable_sehop` Enable SEHOP

**Verdict:** VERIFIED (new in this revision)

**Mechanism:**

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel` | `DisableExceptionChainValidation` | REG_DWORD | `0` | `absent`, effective enabled on 64-bit |

The name is inverted and the tweak handles it correctly: `0` means SEHOP is **enabled**. Confirmed
absent from every one of the 218 shipped ADMX files, which matches the claim that this is a direct write
per the STIG check text rather than an administrative template. Requires reboot. Revert is
value-absent.

**Sourcing note, stated plainly:** there is no reachable tier A page. Microsoft's original SEHOP KB
(956607) is retired and the current Learn troubleshooting URL returned 404 during verification. The
control rests on the DISA STIG check text (tier B) plus long-standing community corroboration. That
clears the corroboration bar, but the confidence label must say benchmark-sourced rather than
Microsoft-documented.

**Corrections needed:** `none`. One honesty requirement: SEHOP validates the structured exception
handler chain, and x64 processes do not use an SEH chain at all, because exception handling on x64 is
table-driven through `.pdata` and `.xdata`. On a 26100 x64 machine this therefore affects only 32-bit
processes, and it is already on. The delta is an auditable registry value and nothing else, so the copy
must not imply any behavioural change.

**Ready-to-paste info block:**

```yaml
    info: |
      **Pins SEHOP on so it cannot be quietly turned off.**

      ## What it does
      Sets `DisableExceptionChainValidation` = 0 under
      `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel`. The name is inverted: 0 means
      Structured Exception Handling Overwrite Protection is enabled. SEHOP validates the exception
      handler chain so an overwritten handler pointer cannot be used to redirect execution.

      ## Benefits
      - **Pins a mitigation**: with the value written, SEHOP cannot be turned off without the change
        being visible
      - **Auditable**: it is an explicit STIG check with a specific value
      - **Zero cost**: no measurable performance or compatibility effect

      ## Drawbacks
      - **No behavioural change on x64**: 64-bit processes do not use an SEH chain at all, exception
        handling is table-driven, so this only affects 32-bit processes and SEHOP is already on for
        them
      - **Needs a reboot**: the value is read at boot
      - **Not Microsoft-documented today**: the original KB is retired and the current Learn URL is
        dead, so the control rests on the DISA STIG check text plus community sources

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2, x64
      - **Takes effect**: after reboot
      - **Reverting**: deletes the value, which behaves the same as 0 on a healthy machine
      - Expect no visible difference; the value of this tweak is that the setting is now explicit and
        auditable rather than implicit

      ## Recommendation
      Apply it if you want a baseline-clean machine or run 32-bit software. If you are looking for a
      behavioural improvement, this is not one; skip it and spend the reboot elsewhere.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Community-corroborated
      - [DISA STIG for Windows 11 V2R2, SEHOP rule](https://www.stigviewer.com/stigs/microsoft_windows_11)
      - [privacy.sexy, SEHOP script](https://privacy.sexy/)
```

**Sources:**
1. DISA STIG for Windows 11 V2R2, "Structured Exception Handling Overwrite Protection (SEHOP) must be enabled" (tier B)
2. All-ADMX scan across the 218 shipped `.admx` files on 26100 (tier A, negative result)
3. privacy.sexy (tier C, corroboration)

### `disable_pku2u_online_id` Block PKU2U online identities

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:**

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\pku2u` | `AllowOnlineID` | REG_DWORD | `0` | `absent`; **effective default on a standalone Windows 11 client is enabled** |

Confirmed absent from every one of the 218 shipped ADMX files, consistent with it being a Security
Options setting. Microsoft's page documents the policy and its semantics but not the registry path; the
path comes from the DISA STIG and CIS check text (tier B). Not already in the corpus.

CORRECTED stock default. The gap proposal stated "value-absent, effective disabled" and rated the tweak
Low on the grounds that the default already matches. Microsoft's Security Policy Settings page for
*Network security: Allow PKU2U authentication requests to this computer to use online identities* says:
"This policy is **enabled by default in Windows 10, Version 1607, and later.**" Its default-values table
gives "Stand-alone server default settings: Not defined", "Member server effective default settings:
Disabled", "Domain controller effective default settings: Disabled", and the "enabled by default in
1607 and later" statement is the one that applies to a standalone Windows 11 client, which is this
corpus's target.

That cuts two ways. The tweak is worth more than claimed, because it is a real behavioural change. It is
also riskier for the same reason: Microsoft's Potential impact section states "Some roles/features (such
as Failover Clustering) don't utilize a domain account for its PKU2U authentication and will cease to
function properly when disabling this policy", and peer-to-peer authentication between non-domain
machines is exactly the workgroup scenario a consumer is in.

**Corrections needed:** Correct the stock default to "effective enabled on a standalone client", raise
the risk level above Low, and name the peer-to-peer sharing scenarios explicitly in the copy. The revert
stays value-absent.

**Ready-to-paste info block:**

```yaml
    info: |
      **Blocks peer-to-peer authentication that uses online identities instead of a real account.**

      ## What it does
      Sets `AllowOnlineID` = 0 under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\pku2u`. PKU2U lets two
      machines that share no domain authenticate each other using online identities, without either
      one holding an account for the other. With this at 0, this PC refuses those requests.

      ## Benefits
      - **A real change, not a pin**: Microsoft documents the policy as enabled by default on a
        standalone Windows 11 client, so this genuinely turns something off
      - **Closes a domainless authentication path**: no account, no shared directory, still
        authenticates
      - **Standard baseline control**: a DISA STIG and CIS rule

      ## Drawbacks
      - **Peer-to-peer sharing breaks**: workgroup device-to-device authentication is exactly what
        PKU2U is for, and a consumer machine is usually in a workgroup
      - **Some features stop working**: Microsoft names Failover Clustering, which does not use a
        domain account for its PKU2U authentication and "will cease to function properly"
      - **Registry path is benchmark-sourced**: Microsoft documents the policy but not the value
        location, which comes from the DISA STIG and CIS check text

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 1607 and later
      - **Takes effect**: immediately, no reboot
      - **Reverting**: deletes the value, restoring the default enabled behaviour on a client
      - Domain-joined machines and member servers already default to Disabled, so this matters most
        on standalone and workgroup PCs

      ## Recommendation
      Apply it on a machine that never authenticates peer-to-peer with another non-domain PC. If you
      share files or use device-to-device features across a workgroup, leave it alone; this is not
      the free pin it looks like.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Network security: Allow PKU2U authentication requests to this computer to use online identities](https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-allow-pku2u-authentication-requests-to-this-computer-to-use-online-identities)
      - [DISA STIG for Windows 11 V2R2, PKU2U rule](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. Network security: Allow PKU2U authentication requests to this computer to use online identities, the default-values table, the "enabled by default in Windows 10, Version 1607, and later" statement, and the Potential impact section, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-allow-pku2u-authentication-requests-to-this-computer-to-use-online-identities (tier A)
2. DISA STIG for Windows 11 V2R2 (tier B, the registry path)
3. All-ADMX scan across the 218 shipped `.admx` files on 26100 (tier A, negative result)

### `no_index_encrypted_files` Do not index encrypted files

**Verdict:** VERIFIED-WITH-CORRECTION (new in this revision)

**Mechanism:**

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` | `AllowIndexingEncryptedStoresOrItems` | REG_DWORD | `0` | `absent`, effective not-indexing |

CORRECTED sourcing, and it **upgrades** the item. The gap proposal stated "Not present in the shipped
ADMX under that value name; written directly per the STIG check text." That is wrong. It is in the
shipped ADMX on 26100:

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

It looks absent because of an encoding trap: `Search.admx` (and `Camera.admx`) ship as **UTF-16LE**
while the other 216 shipped ADMX files are UTF-8, so a byte-level UTF-8 grep silently misses every
string in them. This matters beyond this entry, because "not in the ADMX" was being used as a signal of
lower confidence and the opposite is true here.

The default is confirmed from the shipped `Search.adml` explain text: "This policy setting is not
configured by default. If you do not configure this policy setting, the local setting, configured
through Control Panel, will be used. **By default, the Control Panel setting is set to not index
encrypted content.**"

**Corrections needed:** Change the sourcing claim: this is Microsoft-documented via the shipped ADMX and
ADML, not a STIG-only direct write. Add the ADML warning the proposal omits: enabling or disabling this
setting causes the index to be **rebuilt completely**. Keep the proposal's own no-op caveat, which is
correct: `performance.yaml` ships `disable_search_indexing` (`WSearch`), and anyone who applied that has
no index at all, so this value becomes moot.

**Ready-to-paste info block:**

```yaml
    info: |
      **Keeps the contents of encrypted files out of the unencrypted search index.**

      ## What it does
      Sets `AllowIndexingEncryptedStoresOrItems` = 0 under
      `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search`. Windows Search then refuses to
      extract and store content from EFS-encrypted files. The search index is an ordinary
      unencrypted database, so indexing encrypted content copies it out from behind the encryption.

      ## Benefits
      - **Removes a plaintext copy**: encrypted file content never lands in the index database
      - **Policy-backed and enforced**: writing the policy value overrides whatever the Control Panel
        setting says
      - **Standard baseline control**: a DISA STIG and CIS rule

      ## Drawbacks
      - **Rebuilds the whole index**: Microsoft's own explain text warns that changing this setting
        causes a complete index rebuild, which is a period of heavy disk and CPU activity
      - **Encrypted files stop appearing in search**: you find them by browsing, not searching
      - **Already the default**: the shipped Control Panel setting is not to index encrypted content,
        so this pins rather than changes
      - **Moot without an index**: if you disabled the Windows Search service, this value does
        nothing at all

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and back to Vista
      - **Takes effect**: immediately, followed by a full index rebuild
      - **Reverting**: deletes the value, returning control to the Control Panel setting, and triggers
        another full rebuild
      - Only affects EFS-encrypted files; BitLocker whole-volume encryption is unrelated

      ## Recommendation
      Apply it if you use EFS at all. If you do not use EFS, or you have already turned off the
      Windows Search service, skip it: the index rebuild costs more than the pin is worth.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Search.admx / Search.adml, AllowIndexingEncryptedStoresOrItems (shipped ADMX, 26100)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search)
      - [DISA STIG for Windows 11 V2R2, indexing of encrypted files](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\Search.admx` (26100, **UTF-16LE**), policy `AllowIndexingEncryptedStoresOrItems`, verbatim, and `en-US\Search.adml` `ExplainAllowIndexingEncryptedStoresOrItems` (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2 ("Indexing of encrypted files must be turned off") and CIS Windows 11 v4.0.0 (tier B)
3. `src-tauri/tweaks/performance.yaml`, `disable_search_indexing` (no-op interaction)

### `autoplay_non_volume` Disallow AutoPlay for non-volume devices

**Verdict:** VERIFIED (new in this revision)

**Mechanism:** Confirmed verbatim in `AutoPlay.admx` on 26100:

```xml
<policy name="NoAutoplayfornonVolume" class="Both"
        key="Software\Policies\Microsoft\Windows\Explorer"
        valueName="NoAutoplayfornonVolume">
  <supportedOn ref="windows:SUPPORTED_Windows7" />
  <enabledValue><decimal value="1" /></enabledValue>
```

| Key | Value name | Type | Hardened | Stock Default |
|---|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` | `NoAutoplayfornonVolume` | REG_DWORD | `1` | `absent` |
| `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` | `NoAutoplayfornonVolume` | REG_DWORD | `1` | `absent` |

Key, value name, type, polarity and the hardened value are all correct. `SUPPORTED_Windows7`, so no
build gate is needed.

**Not a duplicate.** The corpus's `disable_autorun` writes `NoAutorun` and `NoDriveTypeAutoRun` only,
which handle drive-letter volumes. `NoAutoplayfornonVolume` handles MTP devices such as phones and
cameras, which never get a drive letter. Three STIG rules exist in this family and the corpus implements
two.

**Corrections needed:** `none` to the mechanism. One addition: the class is **Both**, and this should be
mandatory rather than optional. MTP device handling is driven from the shell and the per-user policy is
the one users actually have, so write both hives and revert both. The cleanest structural fix is to add
this as a third effect on `disable_autorun` rather than shipping a separate tweak: three values, one
concept, one tweak.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops AutoPlay acting on phones, cameras and other devices that get no drive letter.**

      ## What it does
      Sets `NoAutoplayfornonVolume` = 1 in both the machine and user Explorer policy keys. That covers
      MTP and PTP devices, which connect as media transfer devices rather than as drives and are
      therefore not covered by the drive-type AutoPlay controls.

      ## Benefits
      - **Closes the last AutoPlay gap**: the drive-type bitmask does not reach non-volume devices
      - **Covers both hives**: the per-user policy is the one that governs the shell in practice
      - **Completes a family**: three STIG rules exist here and the corpus implements the other two

      ## Drawbacks
      - **Camera and phone prompts disappear**: plugging in a phone no longer offers to import photos
      - **Manual browsing instead**: you open the device in File Explorer yourself
      - **Very small attack-surface gain**: MTP devices cannot autorun code the way removable drives
        historically could

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and back to Windows 7
      - **Takes effect**: after sign-out or an Explorer restart, no reboot
      - **Reverting**: deletes both values, restoring the shipped AutoPlay behaviour
      - This belongs with the AutoRun tweak; the two together cover every AutoPlay path

      ## Recommendation
      Apply it alongside the AutoRun tweak so AutoPlay is fully off. Skip it if you regularly import
      photos from a phone or camera and want the prompt.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Autoplay Policy CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-autoplay)
      - [DISA STIG for Windows 11 V2R2, AutoPlay for non-volume devices](https://www.stigviewer.com/stigs/microsoft_windows_11)
```

**Sources:**
1. `C:\Windows\PolicyDefinitions\AutoPlay.admx` (26100), policy `NoAutoplayfornonVolume`, verbatim (tier A, shipped ADMX)
2. DISA STIG for Windows 11 V2R2 ("Autoplay must be turned off for non-volume devices") (tier B)
3. `src-tauri/tweaks/security.yaml`, `disable_autorun` (dedupe evidence)

## UNKNOWNS

These could not be settled from available sources and need on-box verification on a **clean 26100
image** before the corpus ships. Per `_harmful-revert.md`, a clean-image baseline is the single
highest-value missing input for this project.

1. **`asr_block_lsass_theft` GUID, end to end.** The documentation question is settled (`...e4b2`), but
   confirm empirically by applying the corrected tweak and reading
   `(Get-MpPreference).AttackSurfaceReductionRules_Ids`.
2. **`require_ctrlaltdel` precedence.** With both `Policies\System\DisableCAD` = 0 and
   `Winlogon\DisableCAD` = 1 present, which store wins? Needs a reboot test. The recommended fix writes
   both values, so it is correct either way.
3. **`rdp_security_hardening` stock values.** Read `UserAuthentication` and `SecurityLayer` under
   `...\Terminal Server\WinStations\RDP-Tcp` on a clean install.
4. **`enable_pua_protection` registry value.** Confirm that writing `PUAProtection` under
   `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender` is reflected in
   `Get-MpPreference | ft PUAProtection`, and whether `MpEnablePus` under `MpEngine` is still honoured.
5. **`block_vulnerable_drivers` registry value.** Verify that toggling the Windows Security app control
   writes exactly `VulnerableDriverBlocklistEnable` under `HKLM\SYSTEM\CurrentControlSet\Control\CI\Config`.
6. **`disable_wpbt` end-to-end effect.** The value name is confirmed inside `smss.exe`; what is untested
   is whether `%SystemRoot%\System32\wpbbin.exe` stops being produced after a reboot on a machine whose
   firmware actually publishes a WPBT table.
7. **`disable_windows_script_host` stock value.** `Enabled` is present as REG_DWORD 1 in both registry
   views on the (modified) research machine. Re-read both views on a clean image; if `Enabled` = 1
   really ships, the Stock Default should write `1` rather than `absent` in both effects.
8. **`audit_logon_events` shipped default.** Run `auditpol /get /category:*` on a clean install and
   record the Logon subcategory's shipped inclusion setting, so the undo restores it.
9. **`disable_remote_registry` shipped start type.** Both supporting sources are tier C. Confirm with
   `sc.exe qc RemoteRegistry`.
10. **`require_smb_signing` shipped registry state on 24H2.** Microsoft documents the *behaviour*
    default but not whether `RequireSecuritySignature` is physically present as 1 in either Parameters
    key. Read both on a clean 24H2 Pro install to confirm `absent` is the right revert.
11. **`lock_on_inactivity` stock values.** Read `ScreenSaveActive`, `ScreenSaverIsSecure`,
    `ScreenSaveTimeOut` and `SCRNSAVE.EXE` under `HKCU\Control Panel\Desktop` on a fresh profile.
12. **`disable_remote_assistance` clean-image value.** Microsoft's unattend reference documents the
    default as off, but the byte present in a retail SKU's SYSTEM hive is unconfirmed.
13. **`disable_admin_shares`: is a raw registry write honoured on 26100?** A string scan of all 4,393
    System32 modules plus `System32\drivers` found `AutoShareWks` in exactly one module, `smbwmiv2.dll`
    (the SMB WMI v2 provider behind `Get-SmbServerConfiguration`). `srvsvc.dll`, which creates the `C$`
    and `ADMIN$` special shares, contains no reference in either encoding. Verify empirically: write
    `AutoShareWks` = 0, restart the Server service, then check `net share` and
    `Get-SmbServerConfiguration | Select AutoShareWorkstation`. If the share survives, the tweak should
    go through `Set-SmbServerConfiguration -AutoShareWorkstation $false`.
14. **`disable_winrm_remoting` and `disable_secondary_logon` shipped start types.** Microsoft states
    only that WinRM is "not automatic on client versions"; nothing pins Manual versus trigger-start for
    either service. Mitigated by restoring from the snapshot rather than hardcoding.
15. **Does Tamper Protection block `MpCloudBlockLevel` specifically?** Microsoft's list of
    tamper-protected settings includes "Cloud protection remains enabled" but does not name the cloud
    block level. This decides how much of `defender_cloud_protection` actually lands on a stock consumer
    machine.
16. **ASR rule behaviour on Windows 11 Home.** No tier A source confirms or denies ASR enforcement on
    Home. Treated as unsupported rather than false throughout this document.
17. **`event_log_retention` channel defaults.** The commonly repeated 20480 KB default is unsourced and
    was not confirmed. It does not affect the revert (which is `absent`), but it must not be stated as
    fact in the info copy.
