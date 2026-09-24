# Security Hardening tweaks

The Security Hardening category holds 62 curated Windows hardening tweaks: credential protection (LSASS, NTLM, cached logons, Credential Guard), network and remote-access lockdown (SMB, RDP, WinRM, Remote Registry, the print spooler), Microsoft Defender defenses (Attack Surface Reduction rules, cloud protection, network protection, Controlled Folder Access, SmartScreen), UAC and sign-in hardening, auditing and logging, and legacy protocol removal. The primary supported platform is Windows 11 24H2 (build 26100) and newer, including 25H2 (build 26200); Windows 10 IoT Enterprise LTSC 2021 (build 19044) is a secondary target, and each entry calls out where behavior differs there. Almost every tweak here writes a machine-wide (HKLM) policy or system value and needs administrator rights; every change is captured in a snapshot first and can be returned with Restore Snapshot. Several Defender tweaks depend on Defender being the active antivirus, and some settings are ignored or overridden on Home editions, under Tamper Protection, or on managed (domain, Intune) machines; each entry says so where it applies.

The verdicts on this page come from the July 2026 validation research. That research read shipped Windows files on build 26100 (ADMX and ADML policy definitions, the default security template, strings inside shipped binaries) and Microsoft documentation. The research machine was not a stock image, so no value read from its live registry is used here as proof of a Windows default.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Disable the Remote Registry service](#disable-the-remote-registry-service) | `disable_remote_registry` | Switch | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp) | `disable_remote_desktop` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Remove SMBv1 protocol](#remove-smbv1-protocol) | `remove_smbv1` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable WDigest credential caching](#disable-wdigest-credential-caching) | `disable_wdigest` | Switch (2 options) | low | admin | no | VERIFIED |
| [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl) | `enable_lsa_protection` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Enforce NTLMv2 only](#enforce-ntlmv2-only) | `enforce_ntlmv2` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Require SMB signing](#require-smb-signing) | `require_smb_signing` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Harden RDP (NLA + TLS)](#harden-rdp-nla--tls) | `rdp_security_hardening` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable administrative shares (C$, ADMIN$)](#disable-administrative-shares-c-admin) | `disable_admin_shares` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Prevent LM hash storage](#prevent-lm-hash-storage) | `disable_lmhash_storage` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Restrict anonymous enumeration](#restrict-anonymous-enumeration) | `restrict_anonymous_enum` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Reduce cached domain logons](#reduce-cached-domain-logons) | `reduce_credential_caching` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable the Microsoft vulnerable-driver blocklist](#enable-the-microsoft-vulnerable-driver-blocklist) | `block_vulnerable_drivers` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable WPBT vendor binary execution](#disable-wpbt-vendor-binary-execution) | `disable_wpbt` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Require Ctrl+Alt+Del at sign-in](#require-ctrlaltdel-at-sign-in) | `require_ctrlaltdel` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable AutoRun/AutoPlay on all drives](#disable-autorunautoplay-on-all-drives) | `disable_autorun` | Switch (2 options) | low | admin | no | VERIFIED |
| [Enable PowerShell script-block logging](#enable-powershell-script-block-logging) | `powershell_scriptblock_logging` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Raise UAC to always notify](#raise-uac-to-always-notify) | `uac_max` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Apply UAC to built-in Administrator](#apply-uac-to-built-in-administrator) | `filter_admin_token` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Hide last signed-in username](#hide-last-signed-in-username) | `hide_last_user` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Restrict printer-driver install to admins](#restrict-printer-driver-install-to-admins) | `printnightmare_point_and_print` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Remote Assistance](#disable-remote-assistance) | `disable_remote_assistance` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Windows Script Host](#disable-windows-script-host) | `disable_windows_script_host` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Controlled Folder Access (ransomware shield)](#controlled-folder-access-ransomware-shield) | `enable_controlled_folder_access` | Dropdown (3 options) | medium | admin | no | VERIFIED |
| [Defender Network Protection](#defender-network-protection) | `enable_network_protection` | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable PUA/PUP protection](#enable-puapup-protection) | `enable_pua_protection` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule) | `asr_block_lsass_theft` | Switch (2 options) | medium | admin | no | INCORRECT (corrected form ships) |
| [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors) | `asr_block_office_script_vectors` | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Enforce SmartScreen (apps and Edge)](#enforce-smartscreen-apps-and-edge) | `enforce_smartscreen` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable legacy TLS 1.0/1.1 (Schannel)](#disable-legacy-tls-1011-schannel) | `disable_tls_legacy` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Force .NET strong crypto (TLS 1.2+)](#force-net-strong-crypto-tls-12) | `dotnet_strong_crypto` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable SMB insecure guest logons](#disable-smb-insecure-guest-logons) | `disable_smb_guest` | Switch (2 options) | low | admin | no | VERIFIED |
| [Remove PowerShell 2.0 engine](#remove-powershell-20-engine) | `remove_powershell_v2` | Switch | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Enforce the firewall on all profiles](#enforce-the-firewall-on-all-profiles) | `firewall_all_profiles` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Enable logon/credential auditing](#enable-logoncredential-auditing) | `audit_logon_events` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Lock the screen when idle](#lock-the-screen-when-idle) | `lock_on_inactivity` | Switch | low | none | no | VERIFIED-WITH-CORRECTION |
| [Enable Credential Guard](#enable-credential-guard) | `enable_credential_guard` | Switch (2 options) | medium | admin | yes | INCORRECT (corrected form ships) |
| [Defender cloud protection and MAPS](#defender-cloud-protection-and-maps) | `defender_cloud_protection` | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Block NTLM on the SMB client](#block-ntlm-on-the-smb-client) | `smb_client_block_ntlm` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Enhanced Phishing Protection](#enhanced-phishing-protection) | `enhanced_phishing_protection` | Dropdown (3 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [ASR standard protection rules](#asr-standard-protection-rules) | `asr_standard_protection_rules` | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Disable WinRM remoting](#disable-winrm-remoting) | `disable_winrm_remoting` | Switch | medium | admin | yes | VERIFIED |
| [PowerShell module logging and transcription](#powershell-module-logging-and-transcription) | `powershell_module_transcript_logging` | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Restrict remote SAM calls to administrators](#restrict-remote-sam-calls-to-administrators) | `restrict_remote_sam` | Switch (2 options) | low | admin | no | VERIFIED |
| [Block AlwaysInstallElevated](#block-alwaysinstallelevated) | `block_always_install_elevated` | Switch (2 options) | low | admin | no | VERIFIED |
| [Kernel DMA protection policy](#kernel-dma-protection-policy) | `kernel_dma_protection` | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [ASR extended rule set](#asr-extended-rule-set) | `asr_extended_rules` | Dropdown (3 options) | medium | admin | no | VERIFIED |
| [Restrict outgoing NTLM](#restrict-outgoing-ntlm) | `ntlm_outgoing_restriction` | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Harden the RDP session](#harden-the-rdp-session) | `rdp_session_hardening` | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Disable the Secondary Logon service](#disable-the-secondary-logon-service) | `disable_secondary_logon` | Switch | medium | admin | yes | VERIFIED |
| [Early Launch Antimalware driver policy](#early-launch-antimalware-driver-policy) | `early_launch_antimalware_policy` | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Force updated CredSSP clients](#force-updated-credssp-clients) | `credssp_encryption_oracle` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Prevent automatic device encryption](#prevent-automatic-device-encryption) | `device_encryption_posture` | Switch (2 options) | medium | admin | no | VERIFIED |
| [Hide admin accounts on the UAC prompt](#hide-admin-accounts-on-the-uac-prompt) | `hide_admin_accounts_on_elevation` | Switch (2 options) | low | admin | no | VERIFIED |
| [Event log retention size](#event-log-retention-size) | `event_log_retention` | Dropdown (3 options) | low | admin | no | VERIFIED |
| [Log command lines in process-creation events](#log-command-lines-in-process-creation-events) | `audit_process_creation_cmdline` | Switch (2 options) | low | admin | no | VERIFIED |
| [Turn off the spooler's remote RPC endpoint](#turn-off-the-spoolers-remote-rpc-endpoint) | `spooler_remote_rpc_off` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Filter the remote local-admin token](#filter-the-remote-local-admin-token) | `remote_uac_token_filter` | Switch (2 options) | medium | admin | no | VERIFIED |
| [Enable SEHOP](#enable-sehop) | `enable_sehop` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Block PKU2U online identities](#block-pku2u-online-identities) | `disable_pku2u_online_id` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Do not index encrypted files](#do-not-index-encrypted-files) | `no_index_encrypted_files` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disallow AutoPlay for non-volume devices](#disallow-autoplay-for-non-volume-devices) | `autoplay_non_volume` | Switch (2 options) | low | admin | no | VERIFIED |

## Shared settings

A shared setting is one registry value that more than one tweak needs set to the same value. MagicX Toolbox allows two tweaks to touch one address only through a declared shared setting, so that reverting one tweak can never silently undo another.

### Defender ASR policy enabled

`defender_asr_policy_enabled` · kind: registry · Value: `1`

| Target | Value written |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR` → `ExploitGuard_ASR_Rules` (REG_DWORD) | `1` |

**What it is.** `ExploitGuard_ASR_Rules` is the parent value of the Group Policy setting "Configure Attack Surface Reduction rules". The shipped `WindowsDefender.admx` on build 26100 defines policy `ExploitGuard_ASR_Rules` (class Machine) at key `Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR` with `valueName="ExploitGuard_ASR_Rules"`, and declares the per-rule list (one REG_SZ value per rule GUID, value `0` off, `1` block, `2` audit, `5` not configured, `6` warn) as a `<list>` element whose key is the `ASR\Rules` subkey. Writing rule GUIDs under `ASR\Rules` without the parent value leaves the policy reading as Not Configured in `gpedit.msc`, and a policy refresh can strip the orphaned rule values. Setting the parent to `1` makes the rule list a configured policy.

**Which tweaks claim it.** Four tweaks in this category claim it in every option that turns at least one rule on (Block or Audit), and leave it unclaimed in their "Off" option:

| Tweak | Options that claim | Option that does not claim |
|---|---|---|
| [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule) | Block | Off |
| [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors) | Block, Audit only | Off |
| [ASR standard protection rules](#asr-standard-protection-rules) | Block, Audit only | Off |
| [ASR extended rule set](#asr-extended-rule-set) | Block, Audit only | Off |

Because the value is declared once for the whole corpus, the four tweaks cannot disagree about it: all of them want `1`.

**How a shared claim behaves.** The app keeps one claims record per machine in the portable `snapshots` folder (`shared_claims.<MachineGuid>.json`), holding for each shared setting the original value, the highest elevation level any claim was made at, and the list of tweaks currently claiming it. The lifecycle is reference-counted:

- **First claim.** When the first of the four tweaks is applied with a claiming option, the app reads the live `ExploitGuard_ASR_Rules` value once (including "absent" when the value does not exist), stores it as the original, writes `1`, reads it back to verify, and records the tweak as a claimant.
- **Further claims.** When a second, third or fourth ASR tweak is applied, the app writes nothing: it adds the tweak to the claimant list and verifies the live value is still `1` (a mismatch fails the apply). The original captured by the first claim is kept.
- **Release while others hold it.** Switching a claiming tweak to "Off", or restoring it to its snapshot, removes it from the claimant list. If another ASR tweak still claims the value, it is left at `1` and the releasing tweak reports that the value is held by the remaining tweaks; this is information, not a failure.
- **Last release.** When the last claimant releases, the app writes the captured original back (deleting the value if it was absent), verifies the result, and clears the record; if the restore or its verification fails, the record is kept so the original is never lost. The restore runs at the higher of the recorded claim level and the releasing tweak's own elevation route. Any change made to the value by something else in the meantime is overwritten by this restore.
- **Detection.** Detection reads the claims record, not the live value: a claiming option (Block, Audit only) matches while any of the four tweaks holds the claim, and an "Off" option matches only while none does. So one ASR tweak never appears to drift because another holds the value, but while any ASR tweak is on, another ASR tweak whose rule values are all absent reads as System Default rather than "Off". Each tweak is still told apart by its own rule GUID values, never by the shared value alone.
- **Snapshots.** The shared value is not part of any per-tweak snapshot. Its only return path is the claims record, so two tweaks' snapshots can never fight over it.

**Interactions.** On a machine where an administrator or management tool (Group Policy, Intune) already configures ASR, the captured original is `1` and the last release writes `1` back. A managed policy refresh can also rewrite the value and the rule list independently of the app.

**Sources.** Shipped `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100, policy `ExploitGuard_ASR_Rules` (tier A in the research); the per-rule GUID list and values are sourced in each ASR tweak's entry below.

## Tweaks

### Disable the Remote Registry service

`disable_remote_registry` · Switch · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops anyone from reading or editing this PC's registry over the network.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `remote_registry` | service | Service `RemoteRegistry` (display name "Remote Registry"), start type | Not `optional`; no elevation override |

| Option | `remote_registry` |
|---|---|
| Disabled | `disabled` |

System Default is shown whenever the service's start type is anything other than Disabled (Manual, Automatic, Automatic (Delayed Start)); selecting it restores the start type captured in the snapshot before the first apply. The stock Windows start type is Disabled on both Windows 10 and Windows 11 according to two independent service-default references (tier C), so on an untouched machine the toggle already reads as applied.

#### How it works

The Remote Registry service hosts the RPC endpoint that `RegConnectRegistry` uses when another computer opens this machine's registry across the network (for example `regedit` "Connect Network Registry", `reg query \\host\...`, inventory agents and some backup pre-flight checks). With the start type set to Disabled the Service Control Manager refuses to start the service at all, including through a trigger start, so no remote registry session can be established whatever credentials the caller holds.

The tweak writes the start type through the Service Control Manager; it does not stop an instance that is already running. On a stock machine the service is not running, so nothing changes visibly. If some management tool had started it, the running instance keeps serving until it stops or the machine restarts, after which it cannot start again.

There is only one authored option because the shipped start type is the same value the tweak writes: a second "stock" option would be byte-identical. The way back is the snapshot, which restores whatever start type the machine had before the first apply. That matters because the stock default is Disabled, not Manual: a revert that hardcoded Manual would leave the service more available than it ever was.

#### Benefits
- **Closes a remote channel**: no remote registry session can be established, with or without credentials.
- **Pins the safe state**: a compromised management agent cannot quietly switch the service back to a startable state without the change being visible as drift.
- **Blunts reconnaissance**: remote registry reads are a standard reconnaissance and lateral-movement step.

#### Drawbacks
- **Breaks remote management**: some RMM agents, inventory tools and backup pre-flight checks read this machine's registry across the network and stop working.
- **Usually no visible change**: the service ships Disabled and stopped, so most PCs see nothing.
- **Not a boundary**: an administrator who reaches the machine can set the start type back.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 (build 26100) and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions ship the service.
- **Takes effect**: immediately for new start attempts. An instance already running is not stopped by the tweak and keeps serving until it stops or the machine restarts, which is why the tweak is flagged as needing a reboot.
- **Reverting**: Restore Snapshot sets the start type back to the captured pre-apply value. It never hardcodes Manual.

#### Interactions
None known on the same surface. Related remote-access lockdowns in this category: [Restrict remote SAM calls to administrators](#restrict-remote-sam-calls-to-administrators), [Disable administrative shares (C$, ADMIN$)](#disable-administrative-shares-c-admin) and [Disable WinRM remoting](#disable-winrm-remoting). No other category touches `RemoteRegistry`.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is about the stock state: the shipped `RemoteRegistry` start type is Disabled, not Manual, so the correct way back is a snapshot restore rather than a hardcoded Manual.
- **Confidence**: Community-corroborated. Both sources for the stock start type are tier C service-default references; Microsoft does not publish a per-service default table for Windows 11 clients.
- **Reasoning**: The mechanism (start type Disabled stops SCM starting the service, including by trigger) is standard SCM behaviour and was not disputed. The claim that was attacked was the stock default, and two independent references agree on Disabled for Windows 10 22H2 and Windows 11. Open question recorded in the research UNKNOWNS: confirm with `sc.exe qc RemoteRegistry` on a clean 26100 image. The snapshot-restore design makes the tweak correct whichever value ships.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on any standalone or home machine. Skip it only if you knowingly run software that manages this PC's registry from another computer (RMM agents, some enterprise inventory or backup products).

#### Sources
1. Remote Registry (RemoteRegistry) Service Defaults in Windows 11, stock start type Disabled, https://revertservice.com/11/remoteregistry/ (tier C)
2. Remote Registry, Windows 11 Service, stock start type and service description, https://batcmd.com/windows/11/services/remoteregistry/ (tier C)

### Disable Remote Desktop (RDP)

`disable_remote_desktop` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off inbound Remote Desktop so there is nothing on port 3389 to attack.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `deny_ts` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server` value `fDenyTSConnections` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `deny_ts` |
|---|---|
| Disabled | `1` |
| Allowed | `0` |

System Default is shown when `fDenyTSConnections` is absent or holds any value other than 0 or 1; selecting it restores the value captured in the snapshot. Microsoft states "By default, remote connections aren't allowed", so the stock value is 1 and a clean machine already reads as "Disabled". Note the polarity: "Allowed" writes 0, which opens inbound RDP.

#### How it works

`fDenyTSConnections` is the value the System Properties "Remote" tab and the Settings > System > Remote Desktop toggle write. When it is 1 the Terminal Services listener refuses every incoming Remote Desktop session; when it is 0 the listener accepts connections (subject to NLA, TLS and user-rights checks). Outbound connections from this PC to other machines (`mstsc.exe`) are not affected.

A Group Policy counterpart lives under `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` (policy `TS_DISABLE_CONNECTIONS`, the "Allow users to connect remotely by using Remote Desktop Services" setting, also exposed as the RemoteDesktopServices Policy CSP `AllowUsersToConnectRemotely`). When that policy is configured it overrides this preference value, so on a managed machine the policy decides and this tweak's write has no effect on behaviour.

Only Pro, Enterprise and Education (and the Windows 10 IoT Enterprise LTSC edition) can host an inbound RDP session. Windows Home has no RDP host, so on Home the value is inert either way.

The tweak offers two authored options, so it is a dropdown: "Disabled" (deny) and "Allowed" (permit). "Allowed" is not labelled as the stock state because the stock state is the same as "Disabled".

#### Benefits
- **Removes a top target**: RDP is among the most heavily attacked Windows listeners.
- **No listener, no pre-auth bugs**: BlueKeep-class (CVE-2019-0708) flaws need a reachable listener.
- **Outbound still works**: you can still connect from this PC to others.
- **A real way back**: the "Allowed" option lets you deliberately open RDP from the same control.

#### Drawbacks
- **No inbound RDP**: remote-support tools and VDI agents that rely on the listener stop working.
- **Already the default**: Microsoft ships remote connections denied, so on a clean install this confirms the current state rather than changing it.
- **Group Policy wins**: if a policy configures the counterpart value, the policy overrides this.
- **Inert on Home**: Windows Home cannot host RDP at all.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 IoT Enterprise LTSC 2021; meaningful on Pro, Enterprise, Education and IoT Enterprise only. Home is unaffected.
- **Takes effect**: immediately for new connections, no reboot.
- **Reverting**: Restore Snapshot writes back the captured pre-apply value (or deletes it if it was absent). Choosing "Allowed" is not a revert; it actively opens RDP.

#### Interactions
- [Harden RDP (NLA + TLS)](#harden-rdp-nla--tls) and [Harden the RDP session](#harden-the-rdp-session) only matter when RDP is allowed. With `fDenyTSConnections` = 1 their values are inert, so this tweak and those two are alternative postures: disable RDP, or keep it and harden it. The research proposes merging all three into one "Remote Desktop" control (merge candidate 1).
- [Force updated CredSSP clients](#force-updated-credssp-clients) and [Disable Remote Assistance](#disable-remote-assistance) are related remote-access controls.
- No other category writes `fDenyTSConnections`.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction concerns the option set: the stock value is already 1 (denied), and a second option writing 0 labelled "Allowed" (not "stock") is what gives the tweak a selectable opposite state.
- **Confidence**: Microsoft-documented. Microsoft Learn documents both the value's meaning and the "remote connections aren't allowed" default.
- **Reasoning**: Key, value, type and polarity were checked against Microsoft Learn and survived. The attack focused on whether labelling the opposite state as the Windows default would be honest; it would not, because 1 is the shipped value. The Group Policy override is documented in the Policy CSP.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it unless you deliberately connect into this PC over Remote Desktop. If you do rely on RDP, leave this at "Allowed" and apply [Harden RDP (NLA + TLS)](#harden-rdp-nla--tls) and [Harden the RDP session](#harden-the-rdp-session) instead.

#### Sources
1. Enable Remote Desktop on your PC, the Settings toggle, NLA rationale and "By default, remote connections aren't allowed", https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access (tier A)
2. RemoteDesktopServices Policy CSP, AllowUsersToConnectRemotely, the Group Policy counterpart and its precedence, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-remotedesktopservices (tier A)
3. `_cross-category.md` section 2.1 (single-option tweaks), internal research note cited by the July 2026 research

### Remove SMBv1 protocol

`remove_smbv1` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes the legacy SMBv1 protocol behind WannaCry and EternalBlue.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `smb1_server` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters` value `SMB1` (REG_DWORD) | Not `optional`; no elevation override |
| `smb1_feature` | action (PowerShell, timeout 900 s) | apply: `Disable-WindowsOptionalFeature -Online -FeatureName SMB1Protocol -NoRestart -ErrorAction Stop`; undo: `Enable-WindowsOptionalFeature -Online -FeatureName SMB1Protocol -NoRestart -All -ErrorAction Stop`; probe: exits 0 only when `(Get-WindowsOptionalFeature -Online -FeatureName SMB1Protocol).State` is `Disabled`, `DisablePending` or `DisabledWithPayloadRemoved` | Reversible (has undo) and detectable (has probe) |

| Option | `smb1_server` | `smb1_feature` |
|---|---|---|
| Removed | `0` | `run` (apply: disable the feature) |
| Installed | `absent` | omitted (undo: re-enable the feature) |

System Default is shown when the live state matches neither row, for example `SMB1` absent with the feature disabled (the common stock state on Windows 11); selecting it restores the snapshot. Applying "Removed" there writes `SMB1` = 0 and leaves the already-disabled feature alone: the engine does not run an action whose probe already reads present, so nothing is recorded that a revert could undo, and a revert only deletes `SMB1` again. SMBv1 is never installed on a machine that did not have it. Microsoft states the `SMB1` value's default is 1 (Enabled) and "no registry key is created", so absent is the stock registry state; the feature itself is not installed by default on Windows 11 or on Windows 10 1709 and later, except Windows 10 Home and Pro.

#### How it works

SMBv1 is delivered as the `SMB1Protocol` Windows optional feature, which carries both the SMBv1 client and the SMBv1 server component. Disabling the feature through DISM (`Disable-WindowsOptionalFeature`) removes both halves; the change is staged and finishes on the next restart, which is why the tweak passes `-NoRestart` and sets `requires_reboot`. SMB 2.x and 3.x, used by every modern file share, are separate and unaffected.

The registry half is Microsoft's documented server-side switch: `LanmanServer\Parameters\SMB1` = 0 disables SMBv1 on the server even if the feature is present. Writing it alongside the feature removal makes the server side explicit and pins it off if the feature is later reinstalled by something else.

The action has a probe, so the app detects the real feature state instead of trusting that the script ran. The probe is written in the fail-safe polarity: only an explicit state of `Disabled` exits 0; any other state, or a failed query, exits 1 and the effect reads as not applied. The cross-cutting probe audit lists this tweak among the probes that are already correct. `DisablePending` (the disable is staged until the restart) and `DisabledWithPayloadRemoved` also read as removed.

"Installed" leaves the action out of its values. Omitting an action drives it back to its not-run state, which runs the undo script: `Enable-WindowsOptionalFeature ... -All` reinstalls the feature (with `-All` also enabling any parent features it needs) and the `SMB1` value is deleted so the server default applies. The undo can take several minutes, hence the 900 second timeout.

#### Benefits
- **Kills a wormable protocol**: SMBv1 carried EternalBlue and WannaCry.
- **Removes the listener**: an installed but unused SMBv1 still answers on the network.
- **Matches Microsoft**: SMBv1 is deprecated and not installed by default on Windows 11.
- **Probe-verified**: the app reads the actual optional-feature state rather than assuming success.

#### Drawbacks
- **Ancient devices drop off**: SMB1-only NAS boxes, USB shares on old routers, and Windows XP and Server 2003 hosts become unreachable.
- **Needs a restart**: the optional-feature change is not complete until you reboot.
- **Often already done**: SMBv1 is not installed by default on Windows 11, nor on Windows 10 1709 and later except the Home and Pro editions.
- **Slow**: DISM feature changes can take minutes to apply or undo.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer (usually already absent); Windows 10 22H2 Home and Pro still ship it; Microsoft's exception list names only Home and Pro, so Enterprise-family editions such as Windows 10 IoT Enterprise LTSC 2021 do not install it by default.
- **Takes effect**: after reboot.
- **Reverting**: choosing "Installed" runs the undo (reinstalls the feature) and deletes `SMB1`; Restore Snapshot restores the captured `SMB1` value and reinstalls the feature only if the apply actually disabled it. Either path needs another reboot. Applying on a machine where the feature is already disabled leaves the feature alone, so no revert can reinstall it.

#### Interactions
- [Require SMB signing](#require-smb-signing), [Disable SMB insecure guest logons](#disable-smb-insecure-guest-logons) and [Block NTLM on the SMB client](#block-ntlm-on-the-smb-client) harden SMB 2 and 3; they produce the same class of legacy-NAS breakage, so apply them together and test old storage once.
- Writes `LanmanServer\Parameters`, the same key as [Require SMB signing](#require-smb-signing) (`RequireSecuritySignature`) and [Disable administrative shares (C$, ADMIN$)](#disable-administrative-shares-c-admin) (`AutoShareWks`), but a different value, so there is no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections concern completeness: the Installed option must reinstall the optional feature (which it does, by omitting the action so the undo runs), and Windows 10 Home and Pro are an exception to "not installed by default".
- **Confidence**: Microsoft-documented. The registry value, its default, the feature and its install defaults are all on Microsoft Learn.
- **Reasoning**: The registry key, value, type, the "0 disables" semantics and "absent is the default" were confirmed verbatim against Microsoft's SMBv1 page. The probe polarity was checked in the cross-cutting probe audit and is fail-safe. The claim attacked was the blanket "already removed by default on Windows 10 1709 and later", which Microsoft contradicts for Home and Pro.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on any modern setup; SMBv1 has no place on today's networks. Hold off only if you still reach hardware that speaks nothing newer than SMBv1, and plan to replace that hardware.

#### Sources
1. Detect, enable, and disable SMBv1, SMBv2, and SMBv3 in Windows, the `SMB1` registry value, its default of 1 and "no registry key is created", https://learn.microsoft.com/en-us/windows-server/storage/file-server/troubleshoot/detect-enable-and-disable-smbv1-v2-v3 (tier A)
2. SMBv1 is not installed by default in Windows 10 version 1709 and later, including the Home and Pro exception, https://learn.microsoft.com/en-us/windows-server/storage/file-server/troubleshoot/smbv1-not-installed-by-default-in-windows (tier A)
3. Disable-WindowsOptionalFeature, the DISM cmdlet the apply script uses, https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature (tier A)

### Disable WDigest credential caching

`disable_wdigest` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows from ever keeping your plaintext password in memory.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `use_logon_credential` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\WDigest` value `UseLogonCredential` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `use_logon_credential` |
|---|---|
| Disabled | `0` |
| Not configured | `absent` |

System Default is shown when `UseLogonCredential` holds any value other than 0 (for example 1, the value credential-theft tooling writes); selecting it restores the snapshot. The stock state is absent. "Not configured" deletes the value, and Microsoft documents that with the value absent WDigest does **not** cache credentials on Windows 8.1 and later, so "Not configured" is the Windows default, not "caching turned on".

#### How it works

WDigest is a legacy authentication package (Digest authentication, used by old HTTP and SASL clients). When `UseLogonCredential` is 1, LSASS keeps a reversibly encrypted copy of the user's password in memory at logon so WDigest can answer digest challenges, and tools such as Mimikatz can read it out as plaintext. Microsoft's KB2871997 advisory introduced the `UseLogonCredential` switch and states that when the entry is not present, caching is disabled; that is the default on Windows 8.1, Windows Server 2012 R2 and everything since.

The value is read at logon, so a change affects the next sign-in. Setting it explicitly to 0 has the same runtime effect as leaving it absent, but it pins the state: an attacker who wants plaintext passwords must now overwrite a 0 rather than create a missing value, and the app shows that drift as the tweak leaving its applied state.

#### Benefits
- **Blocks a Mimikatz step**: credential-theft tooling writes 1 here before dumping LSASS.
- **Makes tampering visible**: an attacker must change the value again, which detection rules and this app's drift detection can see.
- **No practical cost**: nothing modern authenticates with WDigest digest credentials.

#### Drawbacks
- **Already the default**: caching has been off since Windows 8.1, so nothing visibly changes.
- **Not a boundary**: an attacker with SYSTEM can flip the value back and wait for the next logon.
- **Rare legacy break**: Microsoft notes you "may notice that credentials are required more frequently when you use WDigest".

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (Windows 11 24H2 and newer, Windows 10 IoT Enterprise LTSC 2021); the switch exists on Windows 8.1 and later, all editions.
- **Takes effect**: at the next sign-in, no reboot.
- **Reverting**: "Not configured" deletes the value, which is the stock state; Restore Snapshot restores the captured pre-apply value (absent on stock). Reverting is safe because absent and 0 behave identically.

#### Interactions
- Part of the research's "LSASS credential protection" merge candidate (3) with [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl), [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule) and [Enable Credential Guard](#enable-credential-guard). They are complementary layers: this one stops the plaintext secret existing; LSA protection and Credential Guard stop it being read.
- No conflicts; no other tweak writes this key.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented. KB2871997 documents the exact key, value name and the "not present means disabled" semantics.
- **Reasoning**: The key path, value name, type, the 0 semantics and the absent default were checked against the advisory and survived. The cross-cutting inclusion review notes only that the entry should not editorialize about it being a no-op: the control exists and is currently off, so it belongs in the corpus, with "no visible change" stated as a drawback.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it. There is no practical downside and it pins a default an attacker otherwise flips silently. Everyone should have this set.

#### Sources
1. Microsoft Security Advisory: Update to improve credentials protection and management (KB2871997), key, value name and default, https://support.microsoft.com/en-us/topic/microsoft-security-advisory-update-to-improve-credentials-protection-and-management-may-13-2014-93434251-04ac-b7f3-52aa-9f951c14b649 (tier A)

### Enable LSA protection (RunAsPPL)

`enable_lsa_protection` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: build >= 22621 (Windows 11 22H2 and newer) · Reversible: yes

**Runs LSASS as a protected process so nothing can read your credentials out of its memory.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `run_as_ppl` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` value `RunAsPPL` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `run_as_ppl` |
|---|---|
| Enabled | `2` |
| Off | `absent` |

System Default is shown when `RunAsPPL` holds any value other than 2 (for example 1, the UEFI-locked form, or 0); selecting it restores the snapshot. The stock registry state is absent on a typical consumer install.

#### How it works

When `RunAsPPL` is set, LSASS starts at boot as a Protected Process Light. A protected process cannot be opened for memory read or code injection by ordinary processes, even ones holding `SeDebugPrivilege`, which is what LSASS dumping tools (Mimikatz, procdump-style dumps) rely on. In addition, every LSA plug-in (authentication packages, security support providers, password filters) must carry a Microsoft signature to load.

Microsoft defines two enabling values. **1** enables protection and, on a Secure Boot machine, also stores the setting in a UEFI variable (a "UEFI lock"); after that, deleting the registry value does nothing, and the variable must be cleared with Microsoft's LSA Protected Process Opt-out tool. **2** enables protection without a UEFI variable, and Microsoft states it is only enforced on Windows 11 22H2 and later. The tweak writes 2 so that a snapshot revert can really undo it. `RunAsPPLBoot`, seen in some guides, is not documented by Microsoft and is not needed.

Because value 2 is ignored before Windows 11 22H2, the tweak is gated to build 22621 and newer rather than falling back to value 1. It is therefore not offered on Windows 10 IoT Enterprise LTSC 2021 (build 19044).

The protection level is set when LSASS starts, so the change needs a reboot. Microsoft's recommended rollout is to run LSA protection in audit mode first and look for CodeIntegrity events 3065 and 3066, which name plug-ins that would be blocked. Enabling Secure Boot makes the protection harder to bypass. On clean, enterprise-joined, HVCI-capable Windows 11 22H2 and later installs Microsoft turns LSA protection on by default.

#### Benefits
- **Blocks LSASS dumping**: the single most common post-exploitation credential step stops working.
- **Signed plug-ins only**: unsigned code can no longer load into the authentication stack.
- **Reversible at 2**: unlike value 1, no firmware variable is written, so the snapshot can undo it.

#### Drawbacks
- **Breaks LSA add-ins**: some smart card middleware, VPN credential providers, legacy password filters and EDR agents load unsigned code into LSA and will silently fail.
- **Needs a restart**: the protection level is set at boot.
- **Value 1 is a trap**: on a Secure Boot machine `RunAsPPL` = 1 writes a UEFI variable a registry revert cannot clear. The tweak never writes 1, but if something else already has, this tweak cannot undo it.
- **Often already on**: enabled by default on clean, enterprise-joined, HVCI-capable Windows 11 22H2 and later installs.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 (build 22621) and newer, so Windows 11 24H2 and newer in this app's support range. Not offered on Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: after reboot.
- **Reverting**: "Off" deletes the value; Restore Snapshot restores the captured pre-apply value (absent on stock). Both need a reboot. Neither can clear a UEFI variable left by an earlier `RunAsPPL` = 1.

#### Interactions
- [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule): Microsoft states the ASR rule "isn't required" and "doesn't provide extra protection" when LSA protection is enabled. Applying both adds compatibility risk without extra protection.
- [Enable Credential Guard](#enable-credential-guard) and [Disable WDigest credential caching](#disable-wdigest-credential-caching) are the other layers of the research's LSASS merge candidate (3).
- Shares the `Control\Lsa` key with [Enforce NTLMv2 only](#enforce-ntlmv2-only), [Prevent LM hash storage](#prevent-lm-hash-storage), [Restrict anonymous enumeration](#restrict-anonymous-enumeration) and [Enable Credential Guard](#enable-credential-guard) (`LsaCfgFlags`), each on a different value, so there is no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is about the value: 1 sets a UEFI lock a registry revert cannot undo, so a reversible tweak must write 2, which is enforced only on Windows 11 22H2 and later (hence the build gate).
- **Confidence**: Microsoft-documented. The value semantics, the UEFI behaviour, the 22H2 enforcement floor and the audit-mode rollout are on Microsoft Learn and in the LSA Policy CSP.
- **Reasoning**: The attack targeted reversibility: a tweak declared reversible that wrote 1 would leave a firmware variable behind. With 2 and the build gate, the registry value fully controls the state. Open point: the research does not establish how the default-on state on enterprise-joined HVCI-capable machines is represented in the registry, so whether "Off" (delete) actually turns protection off on those machines is not settled.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Strongly worth it if you do not depend on unusual authentication add-ins. Test smart card, VPN and security agents first, and turn on Secure Boot so the protection resists bypass. Skip it if a required product loads unsigned LSA plug-ins.

#### Sources
1. Configure added LSA protection, value 1 versus 2, UEFI lock, 22H2 enforcement, audit mode and events 3065 / 3066, https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection (tier A)
2. LocalSecurityAuthority Policy CSP, ConfigureLsaProtectedProcess, the policy-managed equivalent and its values, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-lsa (tier A)

### Enforce NTLMv2 only

`enforce_ntlmv2` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Forces network logons onto NTLMv2 and refuses the crackable LM and NTLMv1 protocols.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `lm_compat_level` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` value `LmCompatibilityLevel` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `lm_compat_level` |
|---|---|
| NTLMv2 only | `5` |
| Not configured | `absent` |

System Default is shown when `LmCompatibilityLevel` is set to any level from 0 to 4 (or any other value); selecting it restores the snapshot. The stock state is absent: Microsoft's default-values table gives "Client Computer Effective Default Settings: Not defined".

#### How it works

`LmCompatibilityLevel` backs the Security Option *Network security: LAN Manager authentication level* and controls which challenge-response protocols this machine sends when it authenticates to others and which it accepts when others authenticate to it. Microsoft's level table:

| Level | Meaning |
|---|---|
| 0 | Send LM and NTLM responses |
| 1 | Send LM and NTLM, use NTLMv2 session security if negotiated |
| 2 | Send NTLM response only |
| 3 | Send NTLMv2 response only |
| 4 | Send NTLMv2 only, refuse LM |
| 5 | Send NTLMv2 only, refuse LM and NTLM |

Level 5 is the strictest level Microsoft defines: this PC sends only NTLMv2, and when it acts as the server (file sharing, RPC) it refuses LM and NTLMv1 responses as well. LM and NTLMv1 responses can be cracked or relayed trivially once captured, so removing them on both sides closes the downgrade path. The value is an LSA setting read by the authentication packages, not an ADMX policy; on a domain it is usually delivered as a Security Option through Group Policy, which would overwrite a local write on refresh.

The "Not configured" option deletes the value, returning to "Not defined". The research does not state what "Not defined" means on current Windows, for either the send or the accept side.

Microsoft documents "Restart requirement: None" for this setting. The tweak still sets `requires_reboot`, a conservative choice so long-lived LSA sessions pick up the new level.

#### Benefits
- **No downgrade path**: LM and NTLMv1 responses are trivially crackable once captured.
- **Two-way enforcement**: applies to what this PC sends and what it accepts.
- **Where Windows is going**: Microsoft is removing NTLMv1 from Windows.

#### Drawbacks
- **Old gear stops authenticating**: NAS boxes, print and scan appliances that only speak LM or NTLMv1 can no longer authenticate to or from this PC.
- **Domain risk**: Microsoft: "Client devices that don't support NTLMv2 authentication can't authenticate in the domain and access domain resources by using LM and NTLM". On a domain, check against your controllers before applying broadly.
- **Restart flagged**: the app asks for a reboot even though Microsoft documents no restart requirement.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (Windows 11 24H2 and newer, Windows 10 IoT Enterprise LTSC 2021), all editions.
- **Takes effect**: Microsoft documents no restart requirement; the app flags a reboot as a conservative measure.
- **Reverting**: "Not configured" deletes the value; Restore Snapshot restores the captured pre-apply value (absent on stock, the "Not defined" state).

#### Interactions
- [Block NTLM on the SMB client](#block-ntlm-on-the-smb-client) goes further for SMB: `LmCompatibilityLevel` picks which NTLM variant is used, while that tweak stops the SMB client offering NTLM at all.
- [Restrict outgoing NTLM](#restrict-outgoing-ntlm) and [Prevent LM hash storage](#prevent-lm-hash-storage) form the research's "legacy authentication lockdown" merge candidate (6) with this tweak.
- Shares the `Control\Lsa` key with [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl), [Prevent LM hash storage](#prevent-lm-hash-storage) and [Restrict anonymous enumeration](#restrict-anonymous-enumeration), on different values.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the research notes only that the reboot flag is stricter than Microsoft's "Restart requirement: None", which is harmless.
- **Confidence**: Microsoft-documented. The level table, default and restart requirement come from the Microsoft Learn security-policy reference.
- **Reasoning**: Key, value, type, the meaning of level 5 and the "Not defined" default all match Microsoft's page. The absent revert was checked against "Not defined" and holds. No open questions remain for the mechanism.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on modern networks. Hold off only if you still authenticate against legacy appliances that cannot do NTLMv2, and on a domain, coordinate with whoever manages the domain's LAN Manager authentication level.

#### Sources
1. Network security: LAN Manager authentication level, level table, "Not defined" default, restart requirement and compatibility warning, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-lan-manager-authentication-level (tier A)

### Require SMB signing

`require_smb_signing` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Requires cryptographic signing on every SMB session so file sharing cannot be relayed.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `workstation_signing` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters` value `RequireSecuritySignature` (REG_DWORD) | SMB client; not `optional`; no elevation override |
| `server_signing` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters` value `RequireSecuritySignature` (REG_DWORD) | SMB server; not `optional`; no elevation override |

| Option | `workstation_signing` | `server_signing` |
|---|---|---|
| Required | `1` | `1` |
| Not configured | `absent` | `absent` |

System Default is shown for any mixed or other state (for example 1 on one side only, or an explicit 0); selecting it restores the snapshot. "Not configured" deletes both values so the OS default for the installed version and edition applies. Microsoft documents that Windows 11 24H2 Enterprise, Pro and Education require both outbound and inbound signing by default, while Windows 11 24H2 Home and Windows 10 require neither; so on 24H2 Pro and above, "Not configured" still means signing is required by the OS default. Whether the default is expressed as a physical registry value or as a built-in default is not settled.

#### How it works

`RequireSecuritySignature` on the Workstation service (client) makes this PC refuse to talk to an SMB server without signing; on the Server service it makes this PC refuse unsigned client sessions. Signing stamps each SMB message with an HMAC-SHA-256 signature (SMB 2.x) or AES-CMAC / AES-128-GMAC (SMB 3.x), so a machine in the path cannot tamper with or relay the traffic. SMB relay is the most reliable lateral-movement technique on a Windows network, and signing is its standard mitigation.

`EnableSecuritySignature` (the "sign if the other side agrees" variant) is ignored for SMB 2.x and later, so the tweak correctly writes only `RequireSecuritySignature`. Microsoft notes that requiring signing also disables guest access to shares, because a guest session has no key to sign with.

A server that cannot sign fails the connection with STATUS_INVALID_SIGNATURE (0xc000a000). Microsoft's guidance in that case is to fix or replace the server rather than turn signing off.

"Not configured" deletes both values rather than writing 0. Writing 0 would drop a 24H2 Pro, Enterprise or Education machine below its own OS default, leaving it less secure than never applying the tweak; the cross-cutting harmful-revert review ranks that pattern as the top-priority defect class, and deleting avoids it. Both services read the values when they start, so the change needs a reboot (or a restart of both services).

#### Benefits
- **Defeats SMB relay**: the most reliable lateral-movement technique on a Windows network.
- **Blocks tampering**: an in-path attacker cannot alter file-sharing traffic undetected.
- **Also kills guest fallback**: Microsoft notes requiring signing disables guest access.
- **Consistent across editions**: brings Home and Windows 10 up to the 24H2 Pro default.

#### Drawbacks
- **Cheap NAS breaks**: servers that cannot sign fail with STATUS_INVALID_SIGNATURE (0xc000a000).
- **Needs a restart**: the services read the registry values at start.
- **Mostly already on**: Windows 11 24H2 Pro, Enterprise and Education require both directions by default, so on those editions this confirms the state rather than changing it.
- **"Not configured" still signs on 24H2 Pro and above**: it leaves the OS default in force, which itself requires signing.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build. Changes behaviour on Windows 11 24H2 Home and on Windows 10 (including IoT Enterprise LTSC 2021), which require neither direction by default; confirms the default on Windows 11 24H2 Pro, Enterprise and Education.
- **Takes effect**: after reboot.
- **Reverting**: "Not configured" deletes both values; Restore Snapshot restores the captured pre-apply values. Neither path ever writes 0.

#### Interactions
- Research merge candidate 5 groups this with [Disable SMB insecure guest logons](#disable-smb-insecure-guest-logons) (which this tweak makes largely redundant for signed sessions, since requiring signing disables guest access) and [Block NTLM on the SMB client](#block-ntlm-on-the-smb-client). All three produce the same NAS breakage.
- [Remove SMBv1 protocol](#remove-smbv1-protocol) and [Disable administrative shares (C$, ADMIN$)](#disable-administrative-shares-c-admin) write other values in `LanmanServer\Parameters`; no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction concerns the way back: it must delete both values, never write 0, because Windows 11 24H2 Pro, Enterprise and Education require signing by default.
- **Confidence**: Microsoft-documented. The 24H2 defaults, the value semantics, the ignored `EnableSecuritySignature` and the error code come from Microsoft Learn, with the Insider announcement as a tier B corroboration.
- **Reasoning**: The harmful-revert pass attacked the opposite state and found that a literal 0 is a documentation-confirmed security regression; deleting is safe on every edition. Open question recorded in the research UNKNOWNS: whether `RequireSecuritySignature` is physically present as 1 in either key on a clean 24H2 Pro install. If it is, a delete would differ from the stock bytes (behaviour stays governed by Microsoft's documented default).
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on modern networks; signing is the direction Windows itself is moving. Skip it only if you depend on network storage that cannot sign, and replace that device when you can.

#### Sources
1. Overview of Server Message Block signing in Windows, signing algorithms, guest-access interaction and the invalid-signature error, https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-signing-overview (tier A)
2. Control SMB signing behavior, "Windows 11, version 24H2 Enterprise, Pro, and Education require both outbound and inbound SMB signing", https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-signing (tier A)
3. SMB signing required by default in Windows Insider, the rollout announcement, https://techcommunity.microsoft.com/blog/filecab/smb-signing-required-by-default-in-windows-insider/3831704 (tier B)
4. `_harmful-revert.md`, "Confirmed, documentation-backed" table, internal cross-cutting research ranking the 0 revert as the top-priority defect

### Harden RDP (NLA + TLS)

`rdp_security_hardening` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Locks Remote Desktop to Network Level Authentication over TLS, killing pre-auth attacks.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `rdp_nla` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server\WinStations\RDP-Tcp` value `UserAuthentication` (REG_DWORD) | Not `optional`; no elevation override |
| `rdp_security_layer` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server\WinStations\RDP-Tcp` value `SecurityLayer` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `rdp_nla` | `rdp_security_layer` |
|---|---|---|
| NLA + TLS required | `1` | `2` |

System Default is shown whenever the pair is not exactly 1 / 2; selecting it (toggling off) restores the values captured in the snapshot. Windows 10 and 11 ship with "Allow connections only from computers running Remote Desktop with Network Level Authentication" checked, so `UserAuthentication` is 1 on a real machine; the stock `SecurityLayer` is not confirmed (Microsoft's unattend reference gives negotiate, 1, as the component default).

#### How it works

Both values live on the RDP-Tcp listener definition read by the Remote Desktop Services host.

- `UserAuthentication` = 1 requires **Network Level Authentication**: the client must authenticate through CredSSP before the server creates a desktop session or draws a logon screen. An unauthenticated caller never reaches session creation, which is the mitigation Microsoft named for BlueKeep (CVE-2019-0708).
- `SecurityLayer` selects the transport security: 0 is native RDP security, 1 negotiates (TLS if the client supports it, otherwise native RDP), 2 requires TLS. Setting 2 removes the negotiation step, so a connection cannot be downgraded to native RDP encryption.

Group Policy equivalents under `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` override this listener key when configured, so on a managed machine the policy decides.

There is only one authored option. The opposite state cannot be authored safely: Microsoft's unattend reference lists `UserAuthentication` = 0 as the component default, but real installs ship with NLA required, so a fixed "stock" option writing 0 would turn NLA off rather than restore it. The toggle's off position restores the captured pre-apply values instead.

#### Benefits
- **No pre-auth surface**: an unauthenticated caller never reaches session creation.
- **No downgrade**: TLS is required rather than negotiated.
- **Cheap**: nothing else about RDP changes for modern clients.

#### Drawbacks
- **Old clients cannot connect**: RDP clients without CredSSP or NLA support are refused.
- **Inert if RDP is off**: with `fDenyTSConnections` = 1 these values do nothing.
- **Group Policy wins**: settings under `Policies\Microsoft\Windows NT\Terminal Services` override this listener key.
- **Home is unaffected**: Windows Home has no RDP host.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build; meaningful on Pro, Enterprise, Education and IoT Enterprise, which can host RDP. No effect on Home.
- **Takes effect**: immediately, on the next connection; no reboot.
- **Reverting**: toggling off (System Default) restores the captured pre-apply values from the snapshot. It never writes 0.

#### Interactions
- [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp): when that tweak denies connections, this one is inert. Research merge candidate 1 proposes one "Remote Desktop" control with "Off", "On, hardened" and the Windows default.
- [Harden the RDP session](#harden-the-rdp-session): its `MinEncryptionLevel` governs the legacy RDP security layer; with `SecurityLayer` = 2 the encryption is negotiated by TLS instead, so that value becomes largely inert when both tweaks are applied (the STIG still checks it).
- [Force updated CredSSP clients](#force-updated-credssp-clients) hardens the CredSSP protocol that NLA runs on.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction concerns the way back: Windows ships NLA required, so the opposite state must restore captured values rather than write `UserAuthentication` = 0.
- **Confidence**: Microsoft-documented. Both values and their meanings are in Microsoft's unattend reference; the NLA rationale is on Microsoft Learn.
- **Reasoning**: Key, value names, types and the 1 / 2 semantics were confirmed. The attack targeted the revert and found the component default (0) differs from the shipped state (1), which the single-option design avoids. Open question recorded in the research UNKNOWNS: read `UserAuthentication` and `SecurityLayer` on a clean 26100 install to confirm the shipped values.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
If you use Remote Desktop, apply this; NLA plus TLS should be treated as mandatory for any reachable RDP listener. If you do not use RDP at all, use [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp) instead.

#### Sources
1. SecurityLayer (unattend reference), value meanings 0 / 1 / 2 and component default, https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-terminalservices-rdp-winstationextensions-securitylayer (tier A)
2. UserAuthentication (unattend reference), NLA value meaning and component default, https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-terminalservices-rdp-winstationextensions-userauthentication (tier A)
3. Enable Remote Desktop on your PC, the NLA rationale and the shipped "NLA required" checkbox, https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access (tier A)

### Disable administrative shares (C$, ADMIN$)

`disable_admin_shares` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows recreating the hidden `C$` and `ADMIN$` shares attackers use to spread.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `auto_share_wks` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters` value `AutoShareWks` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `auto_share_wks` |
|---|---|
| Disabled | `0` |
| Enabled | `absent` |

System Default is shown when `AutoShareWks` holds any value other than 0 (for example an explicit 1); selecting it restores the snapshot. The stock state is absent, which means the Server service creates the administrative shares automatically.

#### How it works

Every time the Server service (`LanmanServer`) starts, it recreates the hidden administrative shares: one per fixed drive (`C$`, `D$`, ...) plus `ADMIN$` (the Windows directory). With `AutoShareWks` = 0 on a workstation SKU, the service skips that step; server SKUs read the parallel value `AutoShareServer` instead. `IPC$` and any shares you created yourself are never affected.

Remote-execution tools such as PsExec, wmiexec and smbexec copy their service binaries through `ADMIN$`, and many worms and ransomware operators browse `C$` with stolen admin credentials. Removing the shares takes those easy paths away, but it is friction rather than a boundary: an administrator who reaches the machine can recreate them with `net share`.

Microsoft's documented apply step is to restart the Server service (`net stop server` then `net start server`); the tweak's reboot flag is a conservative superset of that.

A string scan of the shipped binaries on 26100 (research observation) found `AutoShareWks` only in `smbwmiv2.dll`, the SMB WMI provider behind `Get-SmbServerConfiguration`, and not in `srvsvc.dll`, which creates the special shares. That leaves open whether a raw registry write is still honoured on 26100 or whether only the SMB configuration store (`Set-SmbServerConfiguration -AutoShareWorkstation`) is. The Microsoft article documents the registry value.

#### Benefits
- **Removes lateral-movement targets**: PsExec, wmiexec and smbexec stage files through `ADMIN$`.
- **Defence in depth**: credential reuse against this machine has one less easy path.
- **Reversible**: the shares come back when the value is removed and the service restarts.

#### Drawbacks
- **Remote admin breaks**: remote-administration tooling, some backup products and remote WMI or DCOM workflows that stage files through `ADMIN$` stop working.
- **Not a boundary**: an administrator who reaches the machine can recreate the shares with `net share`.
- **Microsoft advises against it**: their guidance is not to remove administrative shares "because it can break many different things".
- **Effect on 26100 not empirically confirmed**: see How it works.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, client SKUs (which read `AutoShareWks`).
- **Takes effect**: after reboot (Microsoft documents a Server service restart as sufficient).
- **Reverting**: "Enabled" deletes the value; Restore Snapshot restores the captured pre-apply value. The shares reappear the next time the Server service starts.

#### Interactions
- Related lateral-movement controls: [Filter the remote local-admin token](#filter-the-remote-local-admin-token) (keeps remote local-account logons unelevated, which already blocks most `ADMIN$` use by local accounts) and [Restrict remote SAM calls to administrators](#restrict-remote-sam-calls-to-administrators).
- [Remove SMBv1 protocol](#remove-smbv1-protocol) and [Require SMB signing](#require-smb-signing) write other values in `LanmanServer\Parameters`; no conflict.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the research notes only that the reboot flag exceeds Microsoft's documented service restart, which is harmless.
- **Confidence**: Microsoft-documented. The value, its client and server variants, the restart step and Microsoft's caution come from a Microsoft support article.
- **Reasoning**: The mechanism and the absent default match Microsoft's article. The open question, recorded in the research UNKNOWNS, is whether `srvsvc.dll` on 26100 still honours the raw registry value, given the string-scan result. The research's suggested check: write 0, restart the Server service, then run `net share` and `Get-SmbServerConfiguration | Select AutoShareWorkstation`; if the shares survive, the tweak would need to go through `Set-SmbServerConfiguration -AutoShareWorkstation $false`.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Good on a standalone personal PC that never uses these shares. Avoid it if you rely on remote admin tooling, network backup agents or PsExec-style management. After applying, confirm with `net share` that `C$` and `ADMIN$` are gone.

#### Sources
1. Remove administrative shares, `AutoShareWks` / `AutoShareServer`, the Server service restart and Microsoft's caution, https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/remove-administrative-shares (tier A)

### Prevent LM hash storage

`disable_lmhash_storage` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Keeps the ancient, easily cracked LM hash of your password off the disk.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `no_lm_hash` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` value `NoLMHash` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `no_lm_hash` |
|---|---|
| Prevented | `1` |

System Default is shown whenever `NoLMHash` is not 1 (absent, or 0); selecting it (toggling off) restores the value captured in the snapshot. Microsoft's default-values table gives "Client Computer Effective Default Settings: Enabled", the same value the tweak writes, so on a stock machine there is no meaningful opposite state; whether the value is physically present on a clean install is not recorded in the research.

#### How it works

`NoLMHash` backs the Security Option *Network security: Do not store LAN Manager hash value on next password change*. When a password is set or changed, Windows normally stores the NT hash in the SAM (or Active Directory). The LAN Manager hash is a DES-based legacy format that uppercases the password and splits it into two 7-character halves, so it falls to offline cracking almost instantly. With `NoLMHash` = 1, Windows does not write an LM hash at the next password change and keeps only the NT hash.

The setting affects future password changes only. An LM hash already stored stays until that account's password changes, which is why Microsoft's countermeasure guidance is to also have users set a new password. Microsoft states "Restart requirement: None".

The tweak has one authored option because the opposite value (0) would be a security downgrade and is not a Windows default. Its purpose is to pin the safe state so a misconfiguration cannot quietly reintroduce LM hashes.

#### Benefits
- **Removes a crackable artefact**: LM hashes fall to offline cracking almost instantly.
- **Pins the safe state**: the value cannot be quietly lowered without the change showing up as drift.
- **Zero compatibility cost**: nothing modern authenticates against an LM hash.

#### Drawbacks
- **No visible change**: LM hash storage has been prevented by default since Windows Vista.
- **Does not clean up**: an LM hash already in the SAM stays until that password changes.
- **Rare legacy break**: Microsoft notes "some non-Microsoft applications might not be able to connect to the system".
- **No alternative state**: the shipped value is the same value the tweak writes.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, all editions.
- **Takes effect**: at the next password change of each account; no restart.
- **Reverting**: toggling off restores the captured pre-apply value from the snapshot, which on a stock machine is the same effective state.

#### Interactions
- Research merge candidate 6 ("legacy authentication lockdown") groups this with [Enforce NTLMv2 only](#enforce-ntlmv2-only) and [Restrict outgoing NTLM](#restrict-outgoing-ntlm): Level 5 stops LM responses going over the network, while this tweak stops the LM hash being stored for offline cracking.
- Shares the `Control\Lsa` key with [Enforce NTLMv2 only](#enforce-ntlmv2-only), [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl) and [Restrict anonymous enumeration](#restrict-anonymous-enumeration), on different values.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections concern the state model and the reboot flag: Enabled (1) is already the effective client default, so the tweak is a pin-the-default toggle, and no restart is required because the next password change gates the effect.
- **Confidence**: Microsoft-documented. Default, semantics and restart requirement come from the Microsoft Learn security-policy reference.
- **Reasoning**: The value and its polarity were confirmed; the attack was on the option set, since a 0 option would be a downgrade and a single option must still have a way back (the snapshot). The inclusion review keeps it: the control exists and is currently in its safe state, so "no visible change" goes in Drawbacks rather than excluding it.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it. It is pure upside on current Windows, and pinning the value means a future misconfiguration cannot quietly reintroduce LM hashes. If you suspect an old LM hash is stored, change the password afterwards.

#### Sources
1. Network security: Do not store LAN Manager hash value on next password change, default, restart requirement and countermeasure guidance, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-do-not-store-lan-manager-hash-value-on-next-password-change (tier A)
2. `_cross-category.md` section 2.1 (single-option tweaks), internal research note cited by the July 2026 research

### Restrict anonymous enumeration

`restrict_anonymous_enum` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Denies anonymous network callers a list of this PC's accounts and shares.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `restrict_anonymous_sam` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` value `RestrictAnonymousSAM` (REG_DWORD) | Not `optional`; no elevation override |
| `restrict_anonymous` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` value `RestrictAnonymous` (REG_DWORD) | Not `optional`; no elevation override |
| `everyone_includes_anonymous` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` value `EveryoneIncludesAnonymous` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `restrict_anonymous_sam` | `restrict_anonymous` | `everyone_includes_anonymous` |
|---|---|---|---|
| Restricted | `1` | `1` | `0` |
| Standard restrictions | `1` | `0` | `0` |

System Default is shown for any other combination (for example the legacy `RestrictAnonymous` = 2, or `EveryoneIncludesAnonymous` = 1); selecting it restores the snapshot. The stock state is exactly "Standard restrictions" (1 / 0 / 0): Microsoft's default-values tables give that triple as the client effective default, and the shipped default security template `C:\Windows\inf\defltbase.inf` seeds all three explicitly.

#### How it works

The three values back three Security Options read by the LSA when it evaluates an anonymous (null-session) caller:

- `RestrictAnonymousSAM` = 1, *Do not allow anonymous enumeration of SAM accounts*: an anonymous caller cannot list local account names through SAMRPC.
- `RestrictAnonymous` = 1, *Do not allow anonymous enumeration of SAM accounts and shares*: extends the block to share enumeration as well. This is the only value that changes on a stock machine.
- `EveryoneIncludesAnonymous` = 0, *Let Everyone permissions apply to anonymous users*: the Everyone SID is not added to an anonymous token, so resources granted to Everyone are not reachable anonymously.

The tweak uses `RestrictAnonymous` = 1, not the legacy value 2 ("no access without explicit anonymous permissions"), which is known to break networking and domain trusts. None of these settings affects domain controllers.

Unlike most options in this category, "Standard restrictions" writes literals instead of deleting. That is deliberate and evidence-based: `defltbase.inf` contains `RestrictAnonymousSAM=4,1`, `RestrictAnonymous=4,0` and `EveryoneIncludesAnonymous=4,0` in `[Registry Values]` (type 4 is REG_DWORD), so Windows setup writes these exact values and a delete would not match stock.

These are Security Options applied by the security client-side extension, not ADMX registry policies. On a domain-joined or MDM-managed machine a Group Policy refresh, an Intune baseline, or a local `secedit /configure /cfg %windir%\inf\defltbase.inf` resets `RestrictAnonymous` to 0, and the app then shows the tweak as drifted.

Microsoft documents "Restart requirement: None" for all three; the tweak's reboot flag is conservative.

#### Benefits
- **Blunts reconnaissance**: an unauthenticated caller cannot build a target list of usernames.
- **Hides shares**: anonymous share enumeration stops as well.
- **Safe value**: uses `RestrictAnonymous` = 1, not the legacy 2 that breaks domain trusts.
- **Exact way back**: "Standard restrictions" matches the shipped template byte for byte.

#### Drawbacks
- **Legacy discovery breaks**: old cross-domain setups with one-way trusts and appliances that enumerate anonymously stop working.
- **Small real delta**: `RestrictAnonymousSAM` is already 1 by default, so only `RestrictAnonymous` changes on a clean machine.
- **Drifts back on managed PCs**: a domain GPO refresh, an Intune baseline or `secedit` with the default template resets `RestrictAnonymous` to 0.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, all editions; no effect on domain controllers.
- **Takes effect**: Microsoft documents no restart requirement; the app flags a reboot as a conservative measure.
- **Reverting**: "Standard restrictions" writes Windows' own shipped values 1 / 0 / 0; Restore Snapshot restores whatever was captured before the first apply.

#### Interactions
- [Restrict remote SAM calls to administrators](#restrict-remote-sam-calls-to-administrators) is additive, not a duplicate: it writes `RestrictRemoteSam` (REG_SZ) in the same key and governs which **authenticated** principals may call SAMRPC, while this tweak governs **anonymous** sessions.
- Shares the `Control\Lsa` key with [Enforce NTLMv2 only](#enforce-ntlmv2-only), [Prevent LM hash storage](#prevent-lm-hash-storage) and [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl), on different values.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the research notes only that the reboot flag exceeds Microsoft's "Restart requirement: None", which is harmless.
- **Confidence**: Microsoft-documented. Three Microsoft Learn security-policy pages plus the shipped `defltbase.inf` template.
- **Reasoning**: This is the only tweak in its group whose opposite option writes literals rather than deleting, so the revert was attacked hardest; it holds because the shipped Microsoft template contains exactly 1 / 0 / 0. Defaults, semantics and the domain-controller exclusion match Microsoft's pages.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on standalone or home networks. Be cautious in older domain environments that still depend on anonymous enumeration, and expect it to drift back on a managed machine.

#### Sources
1. Network access: Do not allow anonymous enumeration of SAM accounts, `RestrictAnonymousSAM` default and semantics, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-do-not-allow-anonymous-enumeration-of-sam-accounts (tier A)
2. Network access: Do not allow anonymous enumeration of SAM accounts and shares, `RestrictAnonymous` default and semantics, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-do-not-allow-anonymous-enumeration-of-sam-accounts-and-shares (tier A)
3. Network access: Let Everyone permissions apply to anonymous users, `EveryoneIncludesAnonymous` default and semantics, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-let-everyone-permissions-apply-to-anonymous-users (tier A)
4. `C:\Windows\inf\defltbase.inf`, `[Registry Values]`, the shipped Microsoft default security template seeding 1 / 0 / 0 (tier A)

### Reduce cached domain logons

`reduce_credential_caching` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Shrinks the pool of cached domain credentials an attacker could crack offline.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `cached_logons_count` | registry | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` value `CachedLogonsCount` (**REG_SZ**) | Not `optional`; no elevation override |

| Option | `cached_logons_count` |
|---|---|
| Limit to 4 | `"4"` |
| Cache 10 logons | `"10"` |

System Default is shown for any other count (for example "2", "0" or "25") or if the value is absent; selecting it restores the snapshot. Microsoft's documented client default is 10 logons, which is what "Cache 10 logons" writes.

#### How it works

`CachedLogonsCount` backs the Security Option *Interactive logon: Number of previous logons to cache (in case domain controller is not available)*. When a domain (or Entra ID) user signs in successfully, Winlogon stores a verifier for that account in the SECURITY hive so the user can still sign in later when no domain controller is reachable, for example on a laptop away from the office. The count bounds how many distinct users' verifiers are kept. Each cached entry is a DCC2 (mscash2) hash that an attacker who steals the SECURITY hive can attack offline, so a smaller count means fewer targets.

Microsoft's allowed range is 0 to 50; 0 disables caching entirely (no offline sign-in at all). The value is stored as a `REG_SZ` string of digits, documented in Microsoft's cached-domain-logon article: written as a DWORD, Winlogon does not read it and the limit silently never applies. The tweak writes REG_SZ.

Microsoft states "Restart requirement: None. Changes to this policy become effective without a computer restart when they're saved locally or distributed through Group Policy." Microsoft suggests 2 for end-user computers as a workable balance; its security baselines leave the value unset.

On a machine that is neither domain-joined nor Entra-joined there are no cached domain verifiers, so the value changes nothing.

#### Benefits
- **Fewer offline targets**: each cached entry is a crackable DCC2 hash if the SECURITY hive is stolen.
- **Keeps offline logon working**: 4 is small enough to matter and large enough to be usable on a shared laptop.
- **No restart**: the change is live as soon as it is written.

#### Drawbacks
- **Domain-only**: on a machine that is not domain-joined or Entra-joined it does nothing.
- **Lockout risk if set too low**: users who have not signed in recently while on the network can be unable to sign in offline.
- **Type-sensitive**: a DWORD write would be silently ignored (the tweak writes the correct REG_SZ).
- **Not in Microsoft's baselines**: Microsoft's security baselines deliberately leave it unset.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build; meaningful only on domain-joined or Entra-joined machines.
- **Takes effect**: immediately, no restart.
- **Reverting**: "Cache 10 logons" writes Microsoft's documented client default; Restore Snapshot restores whatever was captured before the first apply.

#### Interactions
- [Enable Credential Guard](#enable-credential-guard) protects domain secrets in memory; this tweak limits the on-disk cached verifiers. Complementary.
- Other tweaks write different values under the same Winlogon key: [Require Ctrl+Alt+Del at sign-in](#require-ctrlaltdel-at-sign-in) (`DisableCAD`). No conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is about timing: Microsoft documents no restart requirement, so the setting is live when written.
- **Confidence**: Microsoft-documented. Range, default and restart behaviour come from the Microsoft Learn security-policy page; the REG_SZ type from Microsoft's support article.
- **Reasoning**: Key, name, type and default survived the adversarial pass; the REG_SZ type was specifically checked because a DWORD would be a silent no-op. The research also noted an authoring point about option labels and UI toggle position, which does not apply to the current two-option dropdown.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Useful on managed or domain-joined laptops, especially ones shared between several users. Skip it on ordinary home PCs, where it changes nothing at all. Do not go below 2 on a laptop that is regularly used offline.

#### Sources
1. Interactive logon: Number of previous logons to cache (in case domain controller is not available), range, default of 10, restart requirement, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-number-of-previous-logons-to-cache-in-case-domain-controller-is-not-available (tier A)
2. Cached domain logon information, the REG_SZ registry type, https://github.com/MicrosoftDocs/SupportArticles-docs/blob/main/support/windows-server/user-profiles-and-logon/cached-domain-logon-information.md (tier A, Microsoft support article source)

### Enable the Microsoft vulnerable-driver blocklist

`block_vulnerable_drivers` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Refuses to load kernel drivers on Microsoft's known-vulnerable list.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `vuln_driver_blocklist` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\CI\Config` value `VulnerableDriverBlocklistEnable` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `vuln_driver_blocklist` |
|---|---|
| Enabled | `1` |
| Not configured | `absent` |

System Default is shown when the value holds anything other than 1 (for example 0, which the Windows Security toggle's off position is reported to write); selecting it restores the snapshot. "Not configured" deletes the value, which returns control to the Windows Security app toggle and Microsoft's default. Microsoft states "Since the Windows 11 2022 update, the vulnerable driver blocklist is enabled by default for all devices", so "Not configured" does not necessarily switch the blocklist off.

#### How it works

Code Integrity (CI) checks every kernel driver when it loads. With the vulnerable-driver blocklist on, CI also refuses drivers on Microsoft's list of known-vulnerable or malicious signed drivers. Those are the drivers attackers bring along in "bring your own vulnerable driver" (BYOVD) attacks: a legitimately signed but exploitable driver is loaded to reach the kernel and switch off EDR or security software.

Microsoft documents the feature and its Windows Security app toggle (Device security > Core isolation > Microsoft Vulnerable Driver Blocklist), and it is refreshed through monthly and quarterly servicing. Microsoft does **not** document the registry value name; `VulnerableDriverBlocklistEnable` under `CI\Config` rests on community sources (Eleven Forum tutorial) and a SigmaHQ detection rule that alerts when it is set to 0.

The blocklist is force-enabled whenever memory integrity (HVCI), Smart App Control or S mode is active. In that case the Windows Security toggle is greyed out and this registry value cannot override the enforcement in either direction.

Already-loaded drivers are not unloaded, so the change takes effect at the next boot. Microsoft publishes a downloadable App Control (WDAC) blocklist policy that is usually more complete than the in-OS list, for users who want the strictest coverage.

#### Benefits
- **Stops BYOVD**: bring-your-own-vulnerable-driver is the standard way malware disables EDR and reaches the kernel.
- **Maintained by Microsoft**: the in-OS list is refreshed through servicing.
- **Pins the state**: the toggle cannot be flipped off without the change being visible as drift.

#### Drawbacks
- **Some legitimate drivers stop loading**: Microsoft notes blocking drivers "can cause devices or software to malfunction, and in rare cases, lead to blue screen".
- **Already on since 22H2**: on Windows 11 22H2 and later this usually confirms the current state.
- **Cannot override enforcement**: with HVCI, Smart App Control or S mode active the blocklist is forced on and this value has no say either way.
- **Registry value is community-sourced**: Microsoft documents the feature but not this value name, so a future build could stop honouring it.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build. Windows 11 24H2 and newer ship it on by default; Windows 10 22H2 gained the toggle via KB5018482 (Windows 11: KB5018483 / KB5018496). Whether Windows 10 IoT Enterprise LTSC 2021 received the same servicing is not stated in the research.
- **Takes effect**: after reboot; already-loaded drivers keep running until then.
- **Reverting**: "Not configured" deletes the value, returning control to the Windows Security toggle; Restore Snapshot restores the captured pre-apply value.

#### Interactions
- `performance:disable_vbs_hvci` turns memory integrity off. While HVCI is on, the blocklist is forced on and this value is moot; once HVCI is off, this value (or the Windows Security toggle) is what keeps the blocklist on. Applying both is coherent and gives the gaming-performance trade without losing BYOVD protection.
- [Early Launch Antimalware driver policy](#early-launch-antimalware-driver-policy) is a separate boot-driver control; no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections concern documentation and defaults: the registry value is community-sourced rather than Microsoft-documented, the blocklist is on by default since Windows 11 22H2, and the value cannot override HVCI, Smart App Control or S mode.
- **Confidence**: Community-corroborated. The feature and defaults are tier A; the value name rests on two independent tier C sources.
- **Reasoning**: The value name, path and polarity agree across the community tutorial and the SigmaHQ rule; the attack established that Microsoft never names the value. Open question recorded in the research UNKNOWNS: verify that toggling the Windows Security control writes exactly this value. The inclusion review keeps the tweak: the control exists and is currently on, so "no visible change" is a drawback, not a reason to drop it.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it. The small chance of a blocked legacy driver is well worth closing the kernel escalation path, and on 22H2 and later you are pinning a state Microsoft already ships. If a device or tool stops working after the reboot, check whether its driver is on the list before reverting.

#### Sources
1. Microsoft recommended driver block rules, the feature, its default-on status since the Windows 11 2022 update, HVCI / Smart App Control / S mode enforcement and the blue-screen caution, https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules (tier A, feature and defaults)
2. Enable or Disable Microsoft Vulnerable Driver Blocklist in Windows 11, the registry value, https://www.elevenforum.com/t/enable-or-disable-microsoft-vulnerable-driver-blocklist-in-windows-11.10031/ (tier C, registry value)
3. SigmaHQ rule: Vulnerable Driver Blocklist Disabled, corroborates the value name and its 0 = off polarity, https://detection.fyi/sigmahq/sigma/windows/registry/registry_set/registry_set_vulnerable_driver_blocklist_disable/ (tier C, corroborates the value name)

### Disable WPBT vendor binary execution

`disable_wpbt` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Blocks firmware-supplied binaries from auto-running, cutting a channel that survives a wipe.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `disable_wpbt_execution` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager` value `DisableWpbtExecution` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `disable_wpbt_execution` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` |

System Default is shown when the value holds anything other than 1 (for example an explicit 0); selecting it restores the snapshot. The stock state is absent, meaning WPBT execution is allowed.

#### How it works

The Windows Platform Binary Table (WPBT) is an ACPI table through which the motherboard firmware hands Windows a signed PE executable. During boot, Session Manager (`smss.exe`) reads the table, writes the binary to `%SystemRoot%\System32\wpbbin.exe` and runs it. OEMs use it to reinstall drivers, update utilities or anti-theft agents (for example laptop tracking software) even after a clean Windows install. Because the payload lives in firmware, it also survives a disk wipe, which makes it a persistence channel if the firmware or the vendor binary is compromised. The feature exists on Windows 8 and later, all editions.

With `DisableWpbtExecution` = 1, Session Manager skips the extraction and execution step. Direct inspection of `C:\Windows\System32\smss.exe` on 26100.4061 (research observation of the shipped binary) found the UTF-16 strings `DisableWpbtExecution` and `wpbbin` inside the very binary that performs the extraction, which is primary evidence the value name is read on the target build. Four independent community projects use the same value (dropWPBT, persistence-info.github.io, Chris Titus WinUtil, Security-ADMX).

Microsoft does not document this value. Microsoft's published mitigations for WPBT are an App Control (WDAC) policy, which is enforced for WPBT binaries, a DFCI firmware setting managed through Intune, or removing the table at firmware level. Eclypsium's 2021 WPBT research ("Everyone Gets a Rootkit") documents the attack surface and names WDAC as the mitigation; it does not mention `DisableWpbtExecution`.

Session Manager does not reject unknown values, so a mistyped value would be ignored silently; the only positive end-to-end check is confirming that `wpbbin.exe` is no longer produced after a reboot on a machine whose firmware actually publishes a WPBT table. Most self-built desktops publish no WPBT table, so the tweak has nothing to block there.

#### Benefits
- **Kills firmware persistence**: a WPBT payload lives in firmware and re-runs after a clean Windows reinstall.
- **Stops OEM software returning**: vendor software that reappears after a wipe stops doing so.
- **Confirmed in the shipped binary**: the value name is present inside `smss.exe` on 26100.

#### Drawbacks
- **Breaks legitimate OEM features**: some vendors deliver drivers or anti-theft agents through WPBT and those will not reinstall.
- **Not Microsoft-documented**: there is no contract that a future build keeps honouring it.
- **Hard to verify**: see How it works.
- **Does nothing on most machines**: WPBT only fires where the firmware publishes the table, mostly OEM prebuilt systems.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, all editions; only has an effect where the firmware publishes a WPBT table.
- **Takes effect**: after reboot (Session Manager reads it early in boot).
- **Reverting**: "Enabled" deletes the value (the shipped state); Restore Snapshot restores the captured pre-apply value. A `wpbbin.exe` that already ran before applying is not removed by this tweak, and software it installed stays installed.

#### Interactions
None known. No other tweak writes this value; the other `Session Manager` writes in the corpus are in subkeys: `Power` and `Memory Management` (`performance.yaml`) and `kernel` ([Enable SEHOP](#enable-sehop)).

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction concerns attribution: the value is corroborated by the shipped `smss.exe` and four community projects, not by Eclypsium, whose research covers the threat and the WDAC mitigation only.
- **Confidence**: Community-corroborated. Key, name, type, polarity, the absent default and the reboot requirement all check out, but no Microsoft source documents the value.
- **Reasoning**: The binary string evidence proves the value name is read by the component that runs WPBT binaries on 26100; it does not prove behaviour. Open question recorded in the research UNKNOWNS: whether `wpbbin.exe` stops being produced after a reboot on WPBT-publishing firmware is untested.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
A solid hardening step for a clean, self-managed install where you do not want OEM firmware software returning. Avoid it if you rely on a vendor anti-theft agent or a firmware-delivered driver. On a self-built desktop it is harmless but usually does nothing.

#### Sources
1. Direct binary inspection, Windows 11 24H2 build 26100.4061: `C:\Windows\System32\smss.exe` contains the UTF-16 strings `DisableWpbtExecution` and `wpbbin` (primary measurement)
2. dropWPBT: disables the Windows Platform Binary Table, the same value, https://github.com/Jamesits/dropWPBT (tier C)
3. Windows Platform Binary Table, persistence-info.github.io, WPBT as a persistence technique and the disabling value, https://persistence-info.github.io/Data/wpbbin.html (tier C)
4. Chris Titus WinUtil tweak `WPFTweaksWPBT`, the same value, https://winutil.christitus.com/dev/tweaks/essential-tweaks/wpbt/ (tier C)
5. Security-ADMX issue 8, "Disable WPBT execution" policy, https://github.com/Harvester57/Security-ADMX/issues/8 (tier C)
6. "Everyone Gets a Rootkit", Eclypsium, documents the WPBT threat and the WDAC mitigation, not this registry value, https://eclypsium.com/blog/everyone-gets-a-rootkit/ (tier C)

### Require Ctrl+Alt+Del at sign-in

`require_ctrlaltdel` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Forces the secure Ctrl+Alt+Del sequence at sign-in so a fake login screen cannot take your password.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `policy_disable_cad` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` value `DisableCAD` (REG_DWORD) | Security Option store; not `optional`; no elevation override |
| `winlogon_disable_cad` | registry | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` value `DisableCAD` (REG_DWORD) | Winlogon store read by `authui.dll`; not `optional`; no elevation override |

| Option | `policy_disable_cad` | `winlogon_disable_cad` |
|---|---|---|
| Required | `0` | `0` |
| Not required | `absent` | `1` |

System Default is shown for any other combination (for example policy absent with Winlogon 0, or policy 1); selecting it restores the snapshot. The research's read of a 26100.4061 machine found the policy value absent and the Winlogon value present as REG_DWORD 1, which is exactly "Not required" and matches the observed stock behaviour of no Ctrl+Alt+Del prompt on a non-domain install.

#### How it works

Ctrl+Alt+Del is the Secure Attention Sequence: only the Windows kernel and Winlogon can intercept it, so an application drawing a fake sign-in screen can never receive it. When Windows requires the sequence, pressing it guarantees you are typing your password into the real sign-in desktop. The policy name is inverted: it is *Interactive logon: Do not require CTRL+ALT+DEL*, so `DisableCAD` = 0 means the sequence **is** required.

Windows reads `DisableCAD` from two stores, and the tweak writes both:

- `Policies\System\DisableCAD` is the Security Option delivered by local or domain security policy. Microsoft states that to revert this policy "it is not enough to set its value to Not defined, this registry value needs to be removed as well", which is why "Not required" deletes it.
- `Winlogon\DisableCAD` is the store that actually suppresses the prompt on a stock 24H2 install. A UTF-16 string scan of all 4,393 System32 modules on 26100.4061 (research observation of shipped binaries) found `DisableCAD` in `authui.dll`, the component that renders or skips the "Press Ctrl+Alt+Delete" screen, immediately beside the Winlogon key path, and in `winlogon.exe` inside the Winlogon value-name block; only `credprovs.dll` pairs it with the `Policies\System` path. Writing only the policy value would leave the Winlogon 1 in place and could leave the prompt suppressed.

"Not required" writes the Winlogon value back to 1 (the observed shipped value) rather than deleting it. `DisableCAD` is a Security Option, not an ADMX policy, so on a domain-joined or MDM-managed machine a security-policy refresh can overwrite the policy-store value.

#### Benefits
- **Defeats credential-harvesting overlays**: a spoofed login window cannot receive the secure attention sequence.
- **Covers both stores**: the Winlogon value is the one that suppresses the prompt on 24H2; the policy value alone is not enough.
- **Classic, well understood**: no compatibility surprises for desktop use.

#### Drawbacks
- **One extra keypress**: at every sign-in and after every lock.
- **Awkward without a keyboard**: touch-only tablets, convertibles and kiosk devices have no easy way to send the sequence.
- **No effect on remote threats**: this is a local sign-in protection only.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, all editions.
- **Takes effect**: at the next sign-in or lock screen, no reboot.
- **Reverting**: "Not required" deletes the policy value and sets the Winlogon value to 1; Restore Snapshot restores both captured pre-apply values.

#### Interactions
- [Hide last signed-in username](#hide-last-signed-in-username) writes another Security Option (`DontDisplayLastUserName`) in the same `Policies\System` key; both harden the sign-in screen and combine cleanly.
- [Reduce cached domain logons](#reduce-cached-domain-logons) writes `CachedLogonsCount` in the same Winlogon key; different value, no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction concerns the mechanism: the `Winlogon\DisableCAD` companion is what governs the prompt on 26100, so both stores must be written.
- **Confidence**: Microsoft-documented for the policy value and its revert rule; the Winlogon half rests on the research's string scan of shipped binaries and an observation on a modified 26100.4061 machine.
- **Reasoning**: Polarity and the delete-on-revert rule are straight from Microsoft's page. The binary scan cleanly attributes the Winlogon read to `authui.dll`. Open question recorded in the research UNKNOWNS: with `Policies\System\DisableCAD` = 0 and `Winlogon\DisableCAD` = 1 both present, which store wins needs a reboot test; writing both values, as the tweak does, is correct either way. The shipped Winlogon value of 1 was observed on a heavily modified machine, so it is a research observation rather than an established default.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Worth it on desktops and laptops that care about sign-in security. Skip it on touch-only tablets and keyboard-less kiosks where sending the sequence is impractical.

#### Sources
1. Interactive logon: Do not require CTRL+ALT+DEL, polarity, defaults and the "value needs to be removed" revert rule, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-do-not-require-ctrl-alt-del (tier A)
2. On-box registry state and `secedit /export /areas SECURITYPOLICY` on Windows 11 24H2 build 26100.4061 (tier A for value presence in shipped binaries; not admissible as a default)
3. UTF-16 string scan of `C:\Windows\System32\authui.dll`, `winlogon.exe` and `credprovs.dll` on 26100.4061 (tier A, shipped binaries)

### Disable AutoRun/AutoPlay on all drives

`disable_autorun` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops USB sticks and discs from auto-running anything.**

#### What it changes

| Effect id | Kind | Target | Notes |
|---|---|---|---|
| `no_drive_type_autorun` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer` value `NoDriveTypeAutoRun` (REG_DWORD) | Not `optional`; no elevation override |
| `no_autorun` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer` value `NoAutorun` (REG_DWORD) | Not `optional`; no elevation override |

| Option | `no_drive_type_autorun` | `no_autorun` |
|---|---|---|
| Disabled | `255` | `1` |
| Enabled | `absent` | `absent` |

System Default is shown for any other combination (for example a partial bitmask such as 0x91, or only one of the two values set); selecting it restores the snapshot. The stock state is both values absent, which leaves Windows' own AutoPlay behaviour in charge.

#### How it works

Both values are Group Policy settings from `AutoPlay.admx`, confirmed by the Autoplay Policy CSP:

- `NoDriveTypeAutoRun` backs *Turn off Autoplay*. It is a bitmask of drive types for which AutoPlay is disabled; 255 (0xFF) sets every bit, covering unknown, removable, fixed, network, optical (CD-ROM) and RAM disks.
- `NoAutorun` = 1 backs *Set the default behavior for AutoRun* set to "Do not execute any autorun commands", so `autorun.inf` commands are ignored outright rather than offered in a prompt.

Together, Windows stops acting on `autorun.inf` and stops showing AutoPlay prompts for drives. The policies are declared `class="Both"`: they can be set per user in HKCU as well, and when both hives are set Computer Configuration wins. Writing HKLM, as the tweak does, is therefore the stronger choice and applies to every user. The policy editor that exposes these settings is only on Pro and above, but the registry values are read on every edition.

`autorun.inf` execution from removable drives has been off by default since the Windows 7 era, so the practical gain is closing AutoPlay prompts (which users can misclick) and edge cases such as optical media. Devices that connect over MTP (phones, cameras) get no drive letter, so these values do not cover them; that needs `NoAutoplayfornonVolume`, which [Disallow AutoPlay for non-volume devices](#disallow-autoplay-for-non-volume-devices) sets.

Explorer reads the policies when it starts, so the change applies after sign-out or an Explorer restart.

#### Benefits
- **Closes a classic worm vector**: a generation of USB worms ran the moment media was attached.
- **Covers every drive type**: fixed, removable, network, optical and RAM disks.
- **No prompts to misclick**: AutoPlay cannot offer to run something for you.

#### Drawbacks
- **Manual opening**: you browse removable media in File Explorer yourself.
- **AutoPlay handlers stop**: camera import and disc-burning prompts no longer appear for drives.
- **Smaller gain than it sounds**: `autorun.inf` execution from removable drives has been off by default since the Windows 7 era.
- **Misses MTP devices**: phones and cameras that get no drive letter need `NoAutoplayfornonVolume`, which this tweak does not set.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, all editions (the values are read on Home even though the policy editor is Pro and above); the Autoplay CSP lists Windows 10 1703 and later.
- **Takes effect**: after sign-out or an Explorer restart; no reboot.
- **Reverting**: "Enabled" deletes both values, restoring Windows' own AutoPlay behaviour; Restore Snapshot restores the captured pre-apply values.

#### Interactions
- [Disallow AutoPlay for non-volume devices](#disallow-autoplay-for-non-volume-devices) is the MTP half of the same concept; apply both for full coverage. The research's merge candidate 7 proposes folding it into this tweak as a third effect.
- No other tweak writes `NoDriveTypeAutoRun` or `NoAutorun`, in either hive.

#### Validation
- **Verdict**: VERIFIED. No correction was needed for what is written; the research noted only that `NoAutoplayfornonVolume` is missing from the family, which a separate tweak provides.
- **Confidence**: Microsoft-documented. The Autoplay Policy CSP gives both value names and their key, and the shipped `AutoPlay.admx` confirms class Both.
- **Reasoning**: Key, names, type, 255 covering every drive type and the absent revert were confirmed. The cross-cutting policy-hive audit checked the ADMX class (Both) and found HKLM correct and stronger, since Computer Configuration wins. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it. The manual step is trivial and it removes an old malware vector plus every drive AutoPlay prompt. Pair it with [Disallow AutoPlay for non-volume devices](#disallow-autoplay-for-non-volume-devices) for full coverage.

#### Sources
1. Autoplay Policy CSP, `NoDriveTypeAutoRun` and `NoAutorun` under the Explorer policies key, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-autoplay (tier A)
2. `C:\Windows\PolicyDefinitions\AutoPlay.admx` (26100), class Both and policy `NoAutoplayfornonVolume` for the missing third value (tier A)

### Enable PowerShell script-block logging

`powershell_scriptblock_logging` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Records the real, deobfuscated text of every PowerShell script block that runs on this PC, in both Windows PowerShell 5.1 and PowerShell 7.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `enable_scriptblock_logging` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging` → `EnableScriptBlockLogging` (REG_DWORD) |
| `enable_pwsh7_scriptblock_logging` | registry | `HKLM\SOFTWARE\Policies\Microsoft\PowerShellCore\ScriptBlockLogging` → `EnableScriptBlockLogging` (REG_DWORD) |

| Option | `enable_scriptblock_logging` | `enable_pwsh7_scriptblock_logging` |
|---|---|---|
| Enabled | `1` | `1` |
| Off | `absent` | `absent` |

System Default is shown when the live values match neither option, for example when only one of the two keys carries `EnableScriptBlockLogging` = 1, or when a value holds 0; Restore Snapshot then writes back exactly what was there before the first apply. On a stock Windows install neither value exists (script-block logging is not configured by policy), which is the "Off" state.

#### How it works

Script-block logging is a Group Policy setting ("Turn on PowerShell Script Block Logging") that the PowerShell engine reads when a session starts. With `EnableScriptBlockLogging` = 1, every script block the engine compiles is written in full to event ID 4104 in the `Microsoft-Windows-PowerShell/Operational` event log. The logging happens after PowerShell has decoded the code it is about to run, so obfuscation layers such as `-EncodedCommand` (Base64), string concatenation and `Invoke-Expression` indirection are stripped: the event contains the text that actually executed, not the blob that was passed in. Long script blocks are split across several 4104 events.

Windows PowerShell 5.1 (`powershell.exe`, built into Windows) and PowerShell 7 (`pwsh.exe`, installed separately) read different policy keys. Windows PowerShell reads `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging`; PowerShell 7 reads `HKLM\SOFTWARE\Policies\Microsoft\PowerShellCore\ScriptBlockLogging`. This tweak writes both, so a script moved to `pwsh.exe` to dodge logging is still recorded. Writing the PowerShellCore key on a machine without PowerShell 7 is harmless: nothing reads it until PowerShell 7 is installed, and then logging is on from its first session. PowerShell 7 writes its events to its own channel rather than to the Windows PowerShell channel (the separate `about_Logging_Windows` page documents its logging).

The ADMX class of the Windows PowerShell policy is Both (it can be set per user or per machine). The tweak writes the HKLM (machine) copy, which is the effective machine-wide store and applies to every user and every service account on the PC.

The companion value `EnableScriptBlockInvocationLogging` (start and stop events 4105 and 4106) is deliberately not set: it is extremely noisy for little added value.

#### Benefits
- **The highest-value endpoint log**: PowerShell is the most common living-off-the-land tool, and this turns every use of it into readable evidence for detection and incident response.
- **Defeats obfuscation**: the decoded command is logged, not the encoded or concatenated input.
- **Covers both PowerShell editions**: Windows PowerShell 5.1 and PowerShell 7 are logged by the same tweak.
- **No measurable performance cost**: it is a write to an existing event channel.

#### Drawbacks
- **Secrets can leak into the log**: scripts that contain plaintext passwords, API keys or tokens write them into Event Viewer, so the PowerShell operational log becomes sensitive data. Microsoft recommends pairing script-block logging with Protected Event Logging (which encrypts event content) if this is a concern; this tweak does not configure it.
- **Log volume grows**: busy machines roll the PowerShell operational log over faster. Pair it with a larger event log or accept shorter history.
- **Not tamper-proof on its own**: an attacker who already has administrator rights can clear the log or remove the policy value.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 (build 26100) and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. Windows PowerShell 5.1 everywhere, PowerShell 7 wherever it is installed.
- **Takes effect**: for new PowerShell sessions, no reboot. Sessions already open when the tweak is applied keep their old setting until they are restarted.
- **Reverting**: the "Off" option deletes both values, which is the stock state. Restore Snapshot writes back whatever each value held before the first apply (including `absent`).

#### Interactions
- [PowerShell module logging and transcription](#powershell-module-logging-and-transcription) (`powershell_module_transcript_logging`) writes separate values under `...\Windows\PowerShell\ModuleLogging` and `...\Windows\PowerShell\Transcription`; the two are complementary and together give full PowerShell visibility.
- [Remove PowerShell 2.0 engine](#remove-powershell-20-engine) (`remove_powershell_v2`) closes the downgrade path: the PowerShell 2.0 engine predates script-block logging, so an attacker who can start `powershell -Version 2` bypasses this log.
- [Event log retention size](#event-log-retention-size) (`event_log_retention`) sizes the Application, System and Security logs only; it does not enlarge the PowerShell operational log this tweak fills.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is about coverage: the Windows PowerShell policy key applies to Windows PowerShell 5.1 only, and PowerShell 7 needs its own `PowerShellCore` key, which this tweak writes as a second effect.
- **Confidence**: Microsoft-documented. The key path, value name and event behaviour are given verbatim in Microsoft's `about_Logging` (5.1) and `about_Logging_Windows` (7) pages.
- **Reasoning**: the path, value and type match Microsoft's documentation exactly, and the policy-hive audit confirmed that HKLM is the correct machine-wide store for a class-Both policy. The research recommended gating the PowerShell 7 key on PowerShell 7 being present or disclosing the gap; writing it unconditionally is safe because an unread value has no effect. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Enable it if you care at all about detection or incident response; it is the single highest-value logging setting on a Windows endpoint and costs nothing measurable. Think first only if you routinely run scripts with hardcoded secrets and cannot restrict who reads the event log.

#### Sources
1. about_Logging (PowerShell 5.1), establishes the Windows PowerShell policy key, `EnableScriptBlockLogging`, event 4104 and the Protected Event Logging recommendation, https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging?view=powershell-5.1 (tier A)
2. about_Logging_Windows (PowerShell 7), establishes that PowerShell 7 reads its own `PowerShellCore` policy key, https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows (tier A)
3. MagicX research `docs/superpowers/research/validation/_policy-hive-audit.md`, establishes the ADMX class (Both, `PowerShellExecutionPolicy.admx`) and that HKLM is the effective machine-wide store

### Raise UAC to always notify

`uac_max` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Makes every elevation, including Windows' own tools, need a deliberate confirmation on the tamper-proof secure desktop.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `consent_prompt_admin` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` → `ConsentPromptBehaviorAdmin` (REG_DWORD) |
| `prompt_secure_desktop` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` → `PromptOnSecureDesktop` (REG_DWORD) |
| `enable_lua` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` → `EnableLUA` (REG_DWORD) |

| Option | `consent_prompt_admin` | `prompt_secure_desktop` | `enable_lua` |
|---|---|---|---|
| Always notify | `2` | `1` | `1` |
| Notify on app changes | `5` | `1` | `1` |

"Notify on app changes" is Windows' own documented default triple (5 / 1 / 1), the position the UAC slider ships at. System Default is shown when the live triple matches neither option, for example UAC set to "Notify me only when apps try to make changes (do not dim my desktop)" (`PromptOnSecureDesktop` = 0), UAC turned off (`EnableLUA` = 0), or `ConsentPromptBehaviorAdmin` at any other value; Restore Snapshot writes back the captured pre-apply values.

#### How it works

These three values are Local Security Policy "Security Options", read by the UAC service (Application Information, `appinfo`) and the consent UI whenever a process requests elevation:

- `ConsentPromptBehaviorAdmin` backs *User Account Control: Behavior of the elevation prompt for administrators in Admin Approval Mode*. Microsoft documents six values: 0 elevate without prompting, 1 prompt for credentials on the secure desktop, 2 prompt for consent on the secure desktop, 3 prompt for credentials, 4 prompt for consent, and 5 prompt for consent for non-Windows binaries. The client default is 5 ("This prompt for consent is the default"): signed Windows binaries that are marked for auto-elevation elevate silently, and only other programs prompt. Value 2 removes that auto-elevation allowance, so every elevation prompts.
- `PromptOnSecureDesktop` = 1 draws the prompt on the secure desktop, an isolated desktop that ordinary user-mode software cannot read, draw on or send input to. This is the "dimmed screen" behaviour, and it is also the default.
- `EnableLUA` = 1 backs *User Account Control: Run all administrators in Admin Approval Mode*, the master switch for UAC. It is already 1 on a stock machine; the tweak writes it so that a machine where UAC had been switched off is brought back into a working UAC configuration. Microsoft documents that a change to this setting requires a computer restart before it takes effect, which is why the tweak is flagged as needing a reboot.

The values are written in the preference (non-policy) `CurrentVersion\Policies\System` key, which is where Local Security Policy itself stores them; they are not ADMX Group Policy settings, so there is no separate `SOFTWARE\Policies` copy. They are not in the shipped default security template `C:\Windows\inf\defltbase.inf`, so a plain `secedit /configure` does not reset them, but a domain GPO or Intune security baseline that defines them (delivered through `GptTmpl.inf`) re-asserts its own values at the next policy refresh.

On Windows 11 24H2 and newer with **Administrator Protection** enabled (`TypeOfAdminApprovalMode` = 2), elevation is governed by `ConsentPromptBehaviorEnhancedAdmin` instead of `ConsentPromptBehaviorAdmin`. On such a machine this tweak still writes its values, but they are no longer the deciding setting for how elevation prompts behave.

#### Benefits
- **Removes the auto-elevation allowance**: value 5 lets auto-elevating Windows binaries elevate silently, which is exactly what `fodhelper`, `eventvwr` and COM-based UAC bypasses abuse. Value 2 makes all of them prompt.
- **The prompt cannot be spoofed or clicked for you**: the secure desktop isolates it from other software.
- **Matches CIS and DISA STIG**: both baselines require `ConsentPromptBehaviorAdmin` = 2.
- **Also repairs a disabled UAC**: writing `EnableLUA` = 1 turns UAC back on if another tool switched it off.

#### Drawbacks
- **More prompts**: built-in tools such as Task Manager, Computer Management or Registry Editor that elevated silently now ask every time.
- **Needs a restart**: `EnableLUA` only takes effect after a reboot, so the app flags this tweak as reboot-required in both directions.
- **Not the deciding value under Administrator Protection**: see above.
- **Managed machines drift**: a domain or Intune baseline that defines these values re-asserts them at the next refresh, and the tweak then shows a different state.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: `ConsentPromptBehaviorAdmin` and `PromptOnSecureDesktop` apply to the next elevation; `EnableLUA` needs a restart. The tweak is marked reboot-required because both options write `EnableLUA`.
- **Reverting**: "Notify on app changes" writes Windows' documented default triple 5 / 1 / 1. On a machine that was hardened to a CIS or STIG baseline before the app ever touched it, choosing that option drops it out of compliance, because those baselines require 2. Restore Snapshot instead writes back whatever was there before the first apply, which is the correct way back on a previously hardened machine.

#### Interactions
- [Apply UAC to built-in Administrator](#apply-uac-to-built-in-administrator) (`filter_admin_token`) depends on this tweak: Microsoft requires `ConsentPromptBehaviorAdmin` = 2 alongside `FilterAdministratorToken` = 1, and the research lists the two (plus `remote_uac_token_filter`) as a merge candidate.
- [Filter the remote local-admin token](#filter-the-remote-local-admin-token) (`remote_uac_token_filter`) and [Hide admin accounts on the UAC prompt](#hide-admin-accounts-on-the-uac-prompt) (`hide_admin_accounts_on_elevation`) write other UAC values in the same or a neighbouring key; they do not overlap.
- Any tweak or tool that writes `EnableLUA` = 0 ("disable UAC") is the opposite of this one; `EnableLUA` = 0 also breaks all packaged (UWP and MSIX) apps. No shipped MagicX tweak writes it.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: because the tweak writes `EnableLUA`, a setting Microsoft documents as needing a computer restart, it requires a reboot even though `ConsentPromptBehaviorAdmin` itself does not.
- **Confidence**: Microsoft-documented. All six numeric values (2 / 1 / 1 applied, 5 / 1 / 1 default) are confirmed against Microsoft's default-values tables.
- **Reasoning**: the adversarial pass checked the revert against all three revert failure shapes (below stock, above stock, deleting a seeded value) and found it safe: 5 / 1 / 1 is Microsoft's documented client default. It also noted the Administrator Protection caveat and the baseline-downgrade caveat above, both inherent to the design rather than defects. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Recommended for security-conscious users who can tolerate a few extra prompts, and effectively required if you also apply the built-in Administrator tweak. If the prompts are unacceptable, stay at "Notify on app changes" rather than lowering UAC further.

#### Sources
1. User Account Control: Behavior of the elevation prompt for administrators in Admin Approval Mode, establishes the value table, the client default 5 and value 2, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/user-account-control-behavior-of-the-elevation-prompt-for-administrators-in-admin-approval-mode (tier A)
2. User Account Control: Run all administrators in Admin Approval Mode, establishes that `EnableLUA` changes need a computer restart, https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-run-all-administrators-in-admin-approval-mode (tier A)
3. `C:\Windows\inf\defltbase.inf`, the shipped default security template, establishes that it contains other `Policies\System` values but none of this tweak's three (tier A, shipped file)
4. MagicX research `docs/superpowers/research/validation/_policy-hive-audit.md`, establishes that all three values are Security Options, not ADMX policies

### Apply UAC to built-in Administrator

`filter_admin_token` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Subjects even the built-in Administrator account to UAC, so no account on the PC runs with an unfiltered admin token.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `filter_admin_token` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` → `FilterAdministratorToken` (REG_DWORD) |

| Option | `filter_admin_token` |
|---|---|
| Applied | `1` |
| Not applied | `absent` |

System Default is shown when the value holds anything other than 1 or absent (for example an explicit 0 written by a baseline); Restore Snapshot writes back the captured pre-apply value. The stock state is the value absent, which Microsoft documents as Disabled.

#### How it works

The value backs the security option *User Account Control: Admin Approval Mode for the Built-in Administrator account*. Windows treats the built-in Administrator (the account with relative ID 500) as a special case: by default it is exempt from Admin Approval Mode, so when it signs in it receives a full administrative token and every program it starts runs elevated without a prompt. With `FilterAdministratorToken` = 1 that account is treated like any other administrator: it gets a filtered standard-user token at sign-in and has to consent to elevation through UAC.

Microsoft states that when enabling this you "must also configure the local security policy setting ... to Prompt for consent on the secure desktop", which is `ConsentPromptBehaviorAdmin` = 2 (the [Raise UAC to always notify](#raise-uac-to-always-notify) tweak). This tweak does not write that value itself; it carries a warning telling you to apply the UAC tweak as well. Applying this tweak while `ConsentPromptBehaviorAdmin` sits at a non-prompting value can leave the built-in Administrator unable to elevate at all, which is a real lockout risk on a machine where that account is the only usable administrator.

The built-in Administrator is disabled by default on Windows 10 and 11, so on most machines this setting is dormant. It matters on machines where the account has been enabled: OEM and deployment images, recovery workflows, some installers, and machines upgraded from an older Windows where Administrator was the only account (that account stays enabled after the upgrade).

Like the other UAC values, this is a Security Option stored in the preference `CurrentVersion\Policies\System` key, not an ADMX policy; a domain or Intune baseline that defines it re-asserts its own value at the next refresh.

#### Benefits
- **Removes a UAC blind spot**: no fully unfiltered admin token exists on the machine once both UAC tweaks are applied.
- **Covers a real case**: images, installers and recovery tools do enable that account.
- **Dormant when unused**: costs nothing while the account stays disabled.

#### Drawbacks
- **Lockout risk if applied alone**: without a prompting `ConsentPromptBehaviorAdmin` the built-in Administrator can end up unable to elevate.
- **Breaks unattended automation**: scripts or scheduled work that run as the built-in Administrator and cannot answer a prompt stop working.
- **Usually invisible**: on a normal machine the account is disabled and nothing changes.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: the next time the built-in Administrator signs in (sign out and back in, or run `gpupdate /force`). Microsoft documents "Restart requirement: None".
- **Reverting**: "Not applied" deletes the value, restoring the shipped Disabled state. Restore Snapshot writes back the captured pre-apply value.

#### Interactions
- [Raise UAC to always notify](#raise-uac-to-always-notify) (`uac_max`) supplies the `ConsentPromptBehaviorAdmin` = 2 this setting requires. Apply both.
- [Filter the remote local-admin token](#filter-the-remote-local-admin-token) (`remote_uac_token_filter`) writes `LocalAccountTokenFilterPolicy` in the same key. The names are similar but the behaviour is different: that value governs whether local administrators get a filtered token over the network (pass-the-hash), while this one governs the built-in Administrator's interactive sign-in. They are complementary, not duplicates.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections: Microsoft documents no restart requirement, and the setting must be paired with `ConsentPromptBehaviorAdmin` = 2.
- **Confidence**: Microsoft-documented.
- **Reasoning**: the value, the default (Disabled, so deleting it is a faithful revert) and the sign-out requirement are all on Microsoft's page. The adversarial pass kept the lockout risk as the main hazard; the tweak discloses it in its warning rather than enforcing the pairing, which is why applying the UAC tweak alongside is essential. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it as defence in depth, especially if you or anyone else might ever enable the built-in Administrator, and always apply "Raise UAC to always notify" at the same time. Skip it if the built-in Administrator runs unattended jobs that cannot answer a prompt.

#### Sources
1. User Account Control: Admin Approval Mode for the Built-in Administrator account, establishes the value, the Disabled default, "Restart requirement: None", the sign-out or `gpupdate /force` requirement and the `ConsentPromptBehaviorAdmin` pairing, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/user-account-control-admin-approval-mode-for-the-built-in-administrator-account (tier A)
2. MagicX research `docs/superpowers/research/validation/_verify-gaps-b-medlow.md` (entry 29), establishes that `LocalAccountTokenFilterPolicy` and `FilterAdministratorToken` share a key and control adjacent but different behaviour

### Hide last signed-in username

`hide_last_user` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Leaves the sign-in screen's username field blank instead of naming the last person who signed in.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `dont_display_last_user` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` → `DontDisplayLastUserName` (REG_DWORD) |

| Option | `dont_display_last_user` |
|---|---|
| Hidden | `1` |
| Shown | `0` |

"Shown" writes `0`, which is the value Windows setup seeds on a stock install, so a stock machine is detected as "Shown". System Default is shown when the value is absent or holds anything else; Restore Snapshot writes back the captured pre-apply value.

#### How it works

The value backs the security option *Interactive logon: Don't display last signed-in*. When it is 1, the Windows sign-in screen does not pre-fill or show the account that last signed in; the user has to type (or pick "Other user" and type) the account name before the password. When it is 0, the sign-in screen shows the last account, as normal.

Windows setup seeds this value as REG_DWORD 0: on build 26100 the research found it in the lowercase cluster of values setup writes from the default security template `defltbase.inf` (alongside `legalnoticecaption`, `scforceoption`, `shutdownwithoutlogon` and `undockwithoutlogon`), and `secedit /export /areas SECURITYPOLICY` reports `...\DontDisplayLastUserName=4,0` (type 4 is REG_DWORD, data 0). That is why the "Shown" option writes 0 rather than deleting the value. Behaviourally absent and 0 are identical, but deleting a seeded value makes `secedit` and the Local Security Policy console report the option as Not Defined where a stock machine reports Disabled, and any later baseline comparison drifts.

The related security option *Interactive logon: Display user information when the session is locked* and the value `DontDisplayUserName` (which hides the name at unlock) are separate and not touched here.

#### Benefits
- **No free account name**: someone with physical access or line of sight to the sign-in screen learns nothing about which accounts exist.
- **Cheap**: one value, instant, fully reversible.
- **Useful in shared spaces**: offices, labs, kiosks and public-facing machines.

#### Drawbacks
- **You type your username every time**: at every sign-in and, depending on configuration, after locking.
- **Worse with Microsoft and Entra accounts**: on Microsoft-account and Entra-joined machines the name to type is the full email-style UPN.
- **Little value at home**: on a single-user desktop nobody else sees the sign-in screen.
- **Does not hide the name on the lock screen during unlock**: that needs the separate `DontDisplayUserName` setting.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: the next time the sign-in screen is shown (sign out, restart); no reboot needed.
- **Reverting**: "Shown" writes back the shipped value 0. Restore Snapshot writes back whatever was captured before the first apply.

#### Interactions
- [Hide admin accounts on the UAC prompt](#hide-admin-accounts-on-the-uac-prompt) (`hide_admin_accounts_on_elevation`) is complementary: it stops UAC credential prompts listing administrator accounts, a different disclosure in a different key (`CredUI`).
- [Require Ctrl+Alt+Del at sign-in](#require-ctrlaltdel-at-sign-in) (`require_ctrlaltdel`) also changes the sign-in experience; the two combine without conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the value is seeded by Windows setup as REG_DWORD 0, so the way back to stock is to write 0, not to delete the value.
- **Confidence**: Microsoft-documented. The security option and value are on Microsoft's page; the seeded value was established from `secedit /export` output and the `defltbase.inf` seeded cluster on build 26100, which describe the product rather than one machine's configuration.
- **Reasoning**: the revert is safe against all three revert failure shapes once it writes 0. The research notes the corpus handles the same setup-seeded pattern identically in `restrict_anonymous_enum`. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Worth it on shared or physically exposed machines. On a private single-user PC the extra typing outweighs the small gain; leave it off.

#### Sources
1. Interactive logon: Don't display last signed-in, establishes the security option and its value, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/interactive-logon-do-not-display-last-user-name (tier A)
2. `secedit /export /areas SECURITYPOLICY` output and the `defltbase.inf` lowercase seeded cluster on Windows 11 24H2 build 26100.4061, establish that setup seeds the value as REG_DWORD 0 (tier A, shipped template)
3. MagicX research `docs/superpowers/research/validation/_verify-gaps-b-medlow.md` (entry 22), establishes that `hide_admin_accounts_on_elevation` touches different values in a different key

### Restrict printer-driver install to admins

`printnightmare_point_and_print` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Pins printer-driver installation to administrators, Microsoft's mitigation for the PrintNightmare (CVE-2021-34527) escalation.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `restrict_driver_install` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint` → `RestrictDriverInstallationToAdministrators` (REG_DWORD) |

| Option | `restrict_driver_install` |
|---|---|
| Admins only | `1` |
| Not configured | `absent` |

System Default is shown when the value holds anything else, notably an explicit `0`; Restore Snapshot writes back the captured pre-apply value. On a stock, fully updated machine the value does not exist, so a stock machine is detected as "Not configured", even though (see below) a missing value already behaves as admins-only.

#### How it works

The Print Spooler service loads printer drivers as SYSTEM. Point and Print lets a user connect to a shared printer and have its driver downloaded and installed automatically; before July 2021 a standard user (or malware running as one) could abuse this to make the spooler install and load an attacker-supplied driver, which is the CVE-2021-34527 PrintNightmare privilege escalation and remote code execution path.

KB5005010 introduced `RestrictDriverInstallationToAdministrators` under the Point and Print policy key. With 1, only administrators can install or update printer drivers, whether through Point and Print or otherwise, regardless of other Point and Print settings. With 0, non-administrators can install drivers subject to the older Point and Print restrictions. The default changed with servicing: in the July 6, 2021 through August 9, 2021 updates a missing value meant 0; **in the August 10, 2021 and later updates a missing value means 1 (restricted)**.

The practical consequences on any currently patched machine:

- **"Admins only" confirms the default.** It writes an explicit 1, which protects against something else writing 0 later and makes the restriction visible to policy tooling, but it does not change behaviour on a patched machine.
- **"Not configured" does not unrestrict.** It deletes the value, which on a patched machine leaves the restriction in force. Only an explicit 0 loosens the restriction, and this tweak never writes 0. If something else writes 0, the tweak shows System Default.

The value lives in the Policies hive and is a Machine-class policy in `Printing.admx`, so HKLM is the correct store. The spooler reads it when a driver installation is attempted, so no reboot or service restart is needed.

#### Benefits
- **Closes a SYSTEM-level escalation path**: driver installation by non-admins is the PrintNightmare primitive.
- **Microsoft's own mitigation**: shipped in the July and August 2021 servicing updates.
- **Pins the safe state explicitly**: an explicit 1 cannot be flipped by a missing-value default and shows up in policy tools.

#### Drawbacks
- **Standard users cannot add printers that need a new driver**: an administrator must install the driver. Microsoft's alternatives for self-service printing are pre-staging drivers or a print-management solution.
- **No behavioural change on a patched machine**: since August 10, 2021 a missing value already behaves as 1, so "Admins only" confirms rather than changes.
- **"Not configured" keeps the restriction on a patched machine**: it deletes the value, and the patched default is the restriction.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 22H2 / IoT Enterprise LTSC 2021, all editions with the July 6, 2021 or later cumulative updates (every supported build qualifies).
- **Takes effect**: immediately; the spooler reads it at the next driver installation.
- **Reverting**: "Not configured" deletes the value, which on a patched machine leaves driver installation restricted to administrators. Restore Snapshot writes back the captured pre-apply value, including an explicit 0 if that is what was there.

#### Interactions
- `services:disable_print_spooler` disables the spooler entirely, which removes the whole attack surface (and all printing). With it applied, this tweak has nothing to protect.
- [Turn off the spooler's remote RPC endpoint](#turn-off-the-spoolers-remote-rpc-endpoint) (`spooler_remote_rpc_off`) is the middle option: it keeps local printing but stops the spooler accepting remote RPC. It writes different values under `...\Windows NT\Printers` and `Control\Print`, so the three tweaks are complementary layers.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is about default semantics: since the August 10, 2021 updates a missing value means 1 (restricted), so the applied option confirms the default and deleting the value does not restore unrestricted installation.
- **Confidence**: Microsoft-documented. Key path, value name, DWORD type and the date-dependent defaults are verbatim in KB5005010.
- **Reasoning**: the mechanism is exactly Microsoft's. The research found both option labels misleading on a patched machine (the applied option is a no-op there, and the "revert" does not unrestrict); the unapplied option is therefore labelled "Not configured", and this entry states the no-op plainly. The research also noted that a quote often attributed to KB5005010 ("no other combination of mitigations provides equivalent protection") actually comes from the CVE-2021-34527 MSRC advisory FAQ; it is not relied on here. The inclusion-principle review keeps the tweak: a control that Windows already ships in the safe state is still a legitimate control to expose.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on essentially every machine: it pins the authoritative PrintNightmare mitigation and costs nothing on a patched system. Reconsider only if standard users must install printer drivers themselves, and know that choosing "Not configured" will not actually give them that on a patched machine.

#### Sources
1. KB5005010: Restricting installation of new printer drivers after applying the July 6, 2021 updates, establishes the key, value, type, meaning of 0 and 1, and the date-dependent default, https://support.microsoft.com/en-us/topic/kb5005010-restricting-installation-of-new-printer-drivers-after-applying-the-july-6-2021-updates-31b91c02-05bc-4ada-a7ea-183b129578a7 (tier A)
2. MagicX research `docs/superpowers/research/validation/_policy-hive-audit.md`, establishes the policy is class Machine in `Printing.admx` and HKLM is correct
3. MagicX research `docs/superpowers/research/validation/_inclusion-principle.md`, establishes that controls Windows already ships in the safe state stay in the corpus

### Disable Remote Assistance

`disable_remote_assistance` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off inbound (solicited) Remote Assistance so nobody can be invited to view or control your desktop.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `allow_get_help` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Remote Assistance` → `fAllowToGetHelp` (REG_DWORD) |

| Option | `allow_get_help` |
|---|---|
| Disabled | `0` |
| Not configured | `absent` |

System Default is shown when the value holds anything else, notably `1` (the state the System Properties checkbox writes when a user ticks "Allow Remote Assistance connections to this computer"); Restore Snapshot writes back the captured pre-apply value. Microsoft documents the shipped default as "cannot request assistance".

#### How it works

`fAllowToGetHelp` is the value behind the "Allow Remote Assistance connections to this computer" checkbox on the Remote tab of System Properties. Classic Remote Assistance (`msra.exe`) lets a user send an invitation (file, email or Easy Connect) so a helper can connect, see the desktop and, with permission, take control. With `fAllowToGetHelp` = 0 the machine refuses solicited Remote Assistance sessions.

The two options are asymmetric by design:

- **"Disabled"** writes 0 explicitly.
- **"Not configured"** deletes the value; it never writes 1. Microsoft's unattend reference for `fAllowToGetHelp` states of the false value: "Specifies that the user cannot request assistance from a friend or a support professional. This is the default value." Writing a literal 1 would therefore open an inbound remote-control channel that Windows ships closed. Deleting the value returns the machine to whatever the image ships. Selecting "Not configured" does not by itself switch Remote Assistance on; to actually allow it, tick the checkbox in System Properties (which writes 1, and the tweak then shows System Default).

This is the preference (non-policy) store. The Group Policy counterpart is policy `RA_Solicit` in the shipped `RemoteAssistance.admx`: class Machine, key `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services`, the same value name `fAllowToGetHelp`, enabled 1 / disabled 0. When that policy is set it overrides the value this tweak writes, so on a managed machine a GPO can silently defeat or supersede the tweak. Unsolicited Remote Assistance ("Offer Remote Assistance", where a helper connects without an invitation) is governed by `fAllowUnsolicited` under the same Terminal Services policy key; this tweak does not set it.

Quick Assist, the modern replacement, is a separate Store app with its own relay service and is not affected.

#### Benefits
- **Removes a social-engineering channel**: invitation-based remote control is a staple of tech-support scams.
- **Largely obsolete anyway**: Quick Assist is the supported remote-help path on modern Windows.
- **Instant and reversible**: one value, no reboot.

#### Drawbacks
- **No invitation-based help**: family or helpdesk support through classic Remote Assistance stops working.
- **Does not cover unsolicited offers**: `fAllowUnsolicited` is not set.
- **Group Policy overrides it**: this is the preference store, not the policy store.
- **Probably already off**: Microsoft documents the default as not allowing assistance, so on many machines this confirms rather than changes.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately, for the next Remote Assistance connection attempt; no reboot.
- **Reverting**: "Not configured" deletes the value so the machine falls back to the image's shipped behaviour, which Microsoft documents as off. Restore Snapshot writes back the captured pre-apply value, including 1 if Remote Assistance had been turned on before the first apply.

#### Interactions
- [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp) (`disable_remote_desktop`) closes the other built-in inbound remote-control channel. The two are independent: Remote Assistance works even when Remote Desktop is denied.
- [Harden the RDP session](#harden-the-rdp-session) (`rdp_session_hardening`) writes other values under the same Terminal Services policy key (`...\Windows NT\Terminal Services`), but not `fAllowToGetHelp` or `fAllowUnsolicited`; no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the way back must delete the value rather than write 1, because Microsoft documents the default as "cannot request assistance" and a literal 1 would switch on a feature Windows ships off.
- **Confidence**: Microsoft-documented.
- **Reasoning**: the value and its meaning are on Microsoft's unattend reference, and the Group Policy counterpart was confirmed in the shipped `RemoteAssistance.admx`. Two qualifications remain. The cited page is the unattend reference, which describes the default of the setting as consumed by an answer file; it is the strongest Microsoft statement available, but it is not a statement about the byte present in a given retail image's SYSTEM hive. The research machine's own value was deliberately not used as evidence. The clean-image value is listed as an open question; deleting the value is correct however it resolves.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it if you never use classic Remote Assistance, which is most people; use Quick Assist when you need remote help. Leave it alone only if you genuinely rely on invitation-based Remote Assistance.

#### Sources
1. Microsoft-Windows-RemoteAssistance-Exe | fAllowToGetHelp, establishes the value and that false (not allowed) "is the default value", https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-remoteassistance-exe-fallowtogethelp (tier A)
2. `C:\Windows\PolicyDefinitions\RemoteAssistance.admx`, policy `RA_Solicit`, establishes the Group Policy counterpart under the Terminal Services policy key (tier A, shipped ADMX)
3. MagicX research `docs/superpowers/research/validation/_harmful-revert.md`, establishes the revert failure shape a literal 1 would cause (a revert that turns on something stock leaves off)

### Disable Windows Script Host

`disable_windows_script_host` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Blocks `.vbs`, `.js`, `.wsf` and other Windows Script Host scripts from running, in both the 64-bit and 32-bit script hosts.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `wsh_enabled` | registry | `HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings` → `Enabled` (REG_DWORD) |
| `wsh_enabled_wow64` | registry | `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows Script Host\Settings` → `Enabled` (REG_DWORD) |

| Option | `wsh_enabled` | `wsh_enabled_wow64` |
|---|---|---|
| Disabled | `0` | `0` |
| Enabled | `absent` | `absent` |

System Default is shown when the two values match neither option, for example when only one view is disabled, or when either view carries `Enabled` = 1; Restore Snapshot writes back each view's captured pre-apply value. Whether a clean install has no `Enabled` value or an explicit 1 is not settled (see Validation); behaviourally the two are identical.

#### How it works

Windows Script Host is the runtime behind `wscript.exe` (windowed) and `cscript.exe` (console), which run VBScript (`.vbs`, `.vbe`), JScript (`.js`, `.jse`) and Windows Script Files (`.wsf`). Microsoft's WSH documentation says that to disable WSH you create a REG_DWORD `Enabled` set to 0 under `Software\Microsoft\Windows Script Host\Settings`, in HKCU for one user or HKLM for all users. With it off, any attempt to run a WSH script, whether by double-clicking, through `cscript.exe`, or from a batch file, fails with "Windows Script Host access is disabled on this machine". JavaScript run by browsers, Node.js or other engines is not affected; only the Windows Script Host is.

A 64-bit Windows has two copies of the key. 64-bit processes read `HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings`; 32-bit processes are redirected by the WOW64 registry redirector to `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows Script Host\Settings`, because `Microsoft\Windows Script Host` is not on the redirector's shared-key list. The research confirmed on build 26100 that the two are physically separate keys (they carry different last-write timestamps). `C:\Windows\SysWOW64\wscript.exe` and `cscript.exe` are 32-bit and read only the WOW64 copy, so disabling only the native view would leave `%SystemRoot%\SysWOW64\wscript.exe payload.vbs` as a trivial bypass. This tweak writes both.

The per-user HKCU copy of the same value takes precedence for that user. This tweak does not write HKCU, so a user (or malware running as that user) can re-enable WSH for their own account by writing `Enabled` = 1 under HKCU.

#### Benefits
- **Kills a top dropper format**: VBScript and JScript files, often delivered in archives or as email attachments, remain a common initial-access payload.
- **Covers 32-bit too**: both registry views are written, so the SysWOW64 hosts are blocked as well.
- **Almost nothing modern needs it**: VBScript is deprecated and becoming an on-demand Windows feature.

#### Drawbacks
- **Legacy automation stops**: `.vbs` and `.js` logon scripts, MSI custom actions implemented in VBScript or JScript, and some older installers break, sometimes with obscure errors.
- **Per-user override exists**: HKCU wins for that user unless HKCU is locked down too.
- **Hard to attribute failures**: an installer that suddenly fails may give no hint that WSH is the cause. If a needed installer or script starts failing, check this tweak first.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately, for the next script launch; no reboot.
- **Reverting**: "Enabled" deletes both values, which re-enables WSH. Restore Snapshot writes back each view's captured value (including an explicit 1 if one existed).

#### Interactions
- [Force .NET strong crypto (TLS 1.2+)](#force-net-strong-crypto-tls-12) (`dotnet_strong_crypto`) uses the same two-view pattern for the same reason (32-bit processes read the `Wow6432Node` copy).
- [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors) (`asr_block_office_script_vectors`) blocks obfuscated scripts and email-delivered executables through Defender; this tweak removes the WSH host entirely. They layer without conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the native and `Wow6432Node` views are separate physical keys, so both must be written or the 32-bit script host still runs.
- **Confidence**: Microsoft-documented. The value and its effect are in Microsoft's WSH documentation; the redirection behaviour is in Microsoft's registry redirector documentation.
- **Reasoning**: the adversarial pass proved the two views separate by their different last-write timestamps (a shared or reflected key would show one) and confirmed `SysWOW64\wscript.exe` is 32-bit. Open question: the research machine carried `Enabled` = 1 in both views, which may mean a clean install ships an explicit 1 rather than no value. That needs a clean-image read; if 1 ships, the "Enabled" option would ideally write 1, but absent and 1 behave identically for WSH, so the revert is not broken either way.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
High value on typical home and office PCs that never run legitimate WSH scripts. Avoid it if your workflow, logon scripts or software deployment depend on VBScript or JScript.

#### Sources
1. Running Your Scripts (Windows Script Host), establishes the `Enabled` = 0 mechanism in HKCU and HKLM and the resulting error, https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/windows-scripting/xazzc41b(v=vs.84) (tier A, archived Microsoft documentation)
2. Registry Redirector, establishes that `HKLM\SOFTWARE` is redirected for 32-bit callers and `Microsoft\Windows Script Host` is not a shared key, https://learn.microsoft.com/en-us/windows/win32/winprog64/registry-redirector (tier A)
3. On-box key enumeration and `RegQueryInfoKey` last-write timestamps for both registry views on build 26100.4061 (corroborating only: establishes that the two views are physically separate, not either view's default)

### Controlled Folder Access (ransomware shield)

`enable_controlled_folder_access` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Blocks unrecognised apps from changing files in Documents, Pictures and your other protected folders, Windows' built-in ransomware shield.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `cfa_enable` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\Controlled Folder Access` → `EnableControlledFolderAccess` (REG_DWORD) |

| Option | `cfa_enable` |
|---|---|
| Block | `1` |
| Audit only | `2` |
| Off | `absent` |

System Default is shown when the value holds something else (for example `0`, or the disk-modification modes `3` or `4`); Restore Snapshot writes back the captured pre-apply value. Microsoft documents the default as Disabled, and on a stock machine the policy value does not exist.

#### How it works

Controlled Folder Access is a Microsoft Defender Antivirus feature (part of the Exploit Guard family). When it is on, Defender checks every attempt to modify a file in a protected folder against its list of trusted applications (known, reputable software, plus apps you allow). Untrusted processes are denied write, rename and delete access at the file-system layer, and Windows Security shows a "Unauthorized changes blocked" notification. The default protected folders are Documents, Pictures, Videos, Music, Desktop and Favorites, plus their Public equivalents; you can add folders and allowed apps in Windows Security (Virus and threat protection, Ransomware protection) or by policy.

Microsoft documents five modes for `EnableControlledFolderAccess`: 0 Disabled (default), 1 Enabled (block), 2 Audit (log what would be blocked, block nothing), 3 Block disk modification only, 4 Audit disk modification only. The research matched these against the shipped `WindowsDefender.admx` on build 26100 (policy `ExploitGuard_ControlledFolderAccess_EnableControlledFolderAccess`, same key and value name, all five modes). The tweak offers Block and Audit only; audit is Microsoft's recommended first step, because it shows in the event log which of your apps would have been blocked before you enforce.

The value is written to the Policies hive. Defender keeps its effective runtime state under a separate non-policy key (the same path without `Policies`); detection in the app reads the policy value, so a successful apply proves the policy was set, not that the feature engaged. `Get-MpPreference` shows the effective setting. When the policy is set, the Windows Security toggle shows as managed by your organisation.

Tamper Protection does not interfere: Controlled Folder Access is not on Microsoft's list of tamper-protected settings, so the policy write takes effect even with Tamper Protection on (the default on consumer 24H2). Tamper Protection actually helps here by keeping real-time protection, which this feature relies on, switched on.

#### Benefits
- **Targets ransomware directly**: it is the only built-in Windows control that blocks mass file modification by an unrecognised process.
- **Protects data even if malware runs**: the block happens at the file-write layer, after the malware has already started.
- **Extendable**: add your own folders and approve your own apps.
- **Audit mode available**: see what would be blocked before enforcing.

#### Drawbacks
- **False positives are common at first**: game saves, photo and video editors, backup agents, portable apps and development tools get blocked until you allow them.
- **Needs Defender in active mode**: it does nothing if a third-party antivirus has put Defender in passive mode or disabled it.
- **Managed machines override it**: Intune or Configuration Manager policies overwrite conflicting Group Policy at startup.
- **Detection reads policy, not effective state**: see above.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 (including IoT Enterprise LTSC 2021), with Microsoft Defender Antivirus active; also Windows Server 2019 and later.
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Off" deletes the value, returning control to Windows Security with the shipped Disabled default. Restore Snapshot writes back the captured pre-apply value.

#### Interactions
- [Defender Network Protection](#defender-network-protection) (`enable_network_protection`) writes a sibling value under `...\Windows Defender Exploit Guard\Network Protection`; no conflict.
- The ASR tweaks write under `...\Windows Defender Exploit Guard\ASR`; no conflict.
- [Enforce SmartScreen (apps and Edge)](#enforce-smartscreen-apps-and-edge), [Enable PUA/PUP protection](#enable-puapup-protection) and this tweak together make a noticeably stricter machine; expect to manage allow-lists.

#### Validation
- **Verdict**: VERIFIED
- **Confidence**: Microsoft-documented, and confirmed against the shipped `WindowsDefender.admx` on build 26100.
- **Reasoning**: key, value name and all five modes match both Microsoft's documentation and the shipped ADMX exactly. The Tamper Protection concern was attacked and refuted: Microsoft's list of tamper-protected settings (real-time protection, behaviour monitoring, IOAV, cloud protection, security intelligence updates, automatic actions, notifications, archive scanning, exclusions) does not include Controlled Folder Access. The revert (delete) matches the documented Disabled default. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Worth it if you want ransomware protection and are willing to allow apps when something is blocked. Start with "Audit only" for a week, check what it would have blocked, then switch to "Block". If managing exceptions will frustrate you, leave it off rather than disabling it halfway through a bad week.

#### Sources
1. Configure controlled folder access, establishes the key, value and modes (0 Disabled default, 1 block, 2 audit, 3 and 4 disk modification), https://learn.microsoft.com/en-us/defender-endpoint/controlled-folder-access-configure (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100.4061, policy `ExploitGuard_ControlledFolderAccess_EnableControlledFolderAccess`, establishes the exact key, value name and five modes (tier A, shipped ADMX)
3. Protect security settings with tamper protection, establishes that Controlled Folder Access is not a tamper-protected setting, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)

### Defender Network Protection

`enable_network_protection` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Extends SmartScreen's malicious-site blocking from Edge to every app on the PC, so malware cannot reach known-bad domains and IPs.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `network_protection` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\Network Protection` → `EnableNetworkProtection` (REG_DWORD) |

| Option | `network_protection` |
|---|---|
| Block | `1` |
| Audit only | `2` |
| Off | `absent` |

System Default is shown when the value holds something else (for example an explicit `0`); Restore Snapshot writes back the captured pre-apply value. Microsoft documents the default as Disabled, and on a stock machine the policy value does not exist.

#### How it works

Network Protection is a Microsoft Defender Antivirus feature that inspects outbound connections from every process (not just the browser) and checks the destination domain or IP against Microsoft's SmartScreen reputation service. In block mode (1), connections to destinations with a bad reputation (phishing, malware hosting, command-and-control, exploit sites) are dropped and Windows shows a notification. In audit mode (2), such connections are only logged. 0 is off. The research matched these three modes against the shipped `WindowsDefender.admx` on build 26100 (policy `ExploitGuard_EnableNetworkProtection`, same key and value name).

Block mode is also what makes custom IP and URL indicators and Web Content Filtering enforceable in Defender for Endpoint environments.

Microsoft's requirements for Network Protection are strict, and the tweak is silently inert when they are not met (the policy write succeeds and the app's "did-it-work" check reports success, but no connection is ever blocked):

- **Edition**: supported client operating systems are "Windows 10 or 11 (Pro or Enterprise)". Windows Home is not on the list. The tweak is not gated by edition; on Home it writes the value and nothing happens.
- **Defender active**: Microsoft Defender Antivirus must be in active mode. A third-party antivirus normally puts Defender in passive mode, which makes this inert.
- **Three Defender features on**: real-time protection, behavior monitoring and cloud-delivered protection must all be enabled and active. Because reputation lookups go to Microsoft's cloud, turning cloud-delivered protection off (a common privacy tweak) disables Network Protection.

Tamper Protection does not interfere: Network Protection is not on Microsoft's list of tamper-protected settings. Intune or Configuration Manager settings overwrite conflicting Group Policy at startup on managed machines.

#### Benefits
- **Covers all applications**: command-and-control traffic and malicious downloads from any process are cut off, not just browser navigation.
- **Cheap and broad**: no per-app configuration.
- **Enables enforcement features**: custom indicators and Web Content Filtering depend on block mode.

#### Drawbacks
- **Pro, Enterprise and Education only**: on Home it silently does nothing.
- **Needs Defender active with three features on**: inert with a third-party antivirus, or with cloud-delivered protection or behavior monitoring off.
- **Reputation lookups go to Microsoft**: destinations are checked against Microsoft's cloud service.
- **Occasional false positives**: niche or newly registered domains can be blocked.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 22H2, Pro, Enterprise and Education; also Windows 10 IoT Enterprise LTSC 2021 (an Enterprise edition). Not Home.
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Off" deletes the value, restoring the shipped Disabled default. Restore Snapshot writes back the captured pre-apply value.

#### Interactions
- [Defender cloud protection and MAPS](#defender-cloud-protection-and-maps) (`defender_cloud_protection`) turns on the cloud-delivered protection this tweak requires; the research identifies this as the one real dependency among the Defender tweaks. Any tweak or setting that turns cloud protection or MAPS off makes this one inert.
- [Enforce SmartScreen (apps and Edge)](#enforce-smartscreen-apps-and-edge) (`enforce_smartscreen`) uses the same reputation service for downloads and Edge browsing; this tweak extends it to every process.
- [Controlled Folder Access (ransomware shield)](#controlled-folder-access-ransomware-shield) writes a sibling Exploit Guard value; no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is about applicability: Microsoft supports Network Protection on Pro and Enterprise only, and requires behavior monitoring and Defender in active mode in addition to real-time and cloud-delivered protection.
- **Confidence**: Microsoft-documented, and confirmed against the shipped `WindowsDefender.admx`.
- **Reasoning**: the key, value and modes match Microsoft's documentation and the shipped ADMX exactly; the documented default (Disabled) makes deleting the value a safe revert against all three revert failure shapes. The Tamper Protection concern was refuted for the same reason as Controlled Folder Access. The research's remedy for Home was "gate it away from Home or disclose it"; the tweak discloses it in its warning. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Recommended if Defender is your active antivirus on Pro or better and cloud-delivered protection is on; it is a strong, low-friction layer. Skip it on Home, with a third-party antivirus, or if you have turned Defender's cloud protection off, because in all three cases it does nothing.

#### Sources
1. Turn on network protection, establishes the key, value and modes (0 off, 1 on, 2 audit), https://learn.microsoft.com/en-us/defender-endpoint/enable-network-protection (tier A)
2. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100.4061, policy `ExploitGuard_EnableNetworkProtection`, establishes the exact key, value name and three modes (tier A, shipped ADMX)
3. Use network protection to help prevent connections to malicious or suspicious sites, "Requirements for network protection", establishes the Pro/Enterprise support list, the three required Defender features and active mode, https://learn.microsoft.com/en-us/defender-endpoint/network-protection (tier A)
4. Protect security settings with tamper protection, establishes that Network Protection is not a tamper-protected setting, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)
5. MagicX research `docs/superpowers/research/validation/_verify-gaps-b-high.md` (entry 1), establishes that Network Protection is the corpus tweak that genuinely depends on cloud-delivered protection

### Enable PUA/PUP protection

`enable_pua_protection` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Lets Defender block and quarantine bundleware, adware and other potentially unwanted apps instead of only logging them.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `pua_protection` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender` → `PUAProtection` (REG_DWORD) |

| Option | `pua_protection` |
|---|---|
| Block | `1` |
| Not configured | `absent` |

System Default is shown when the value holds something else (for example `0` or `2`); Restore Snapshot writes back the captured pre-apply value. On a stock machine the policy value does not exist, and Defender then runs PUA protection in its own default mode, which is Audit (see below), not off.

#### How it works

Potentially unwanted applications (PUA) are programs that are not malware but that Microsoft classifies as unwanted: bundleware that installs extra software, adware, aggressive browser toolbars, cryptocurrency miners bundled with other software, and installers that evade security products. Regular antivirus signatures deliberately do not act on this category. The Group Policy setting *Configure detection for potentially unwanted applications* (and `Set-MpPreference -PUAProtection`) controls how Defender treats it: **0** off, **1** block (detected items are blocked and quarantined), **2** audit (detections are logged only).

With security intelligence version 1.329.495.0 or later, Defender defaults PUA protection to **Audit (2)** on Windows 10 and later for devices not onboarded to Defender for Endpoint, and to Block (1) on onboarded devices. So on a current consumer machine this tweak turns logging into blocking rather than switching something on from nothing.

The tweak writes `PUAProtection` directly under the Defender policy key, which is the value the modern `WindowsDefender.admx` defines (class Machine) for this policy. An older route, `MpEnablePus` under `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine`, also exists in historical documentation and is not written here. Microsoft's PUA article documents only the Group Policy setting and the PowerShell cmdlet, not the registry value, so confirm the tweak took with `Get-MpPreference | Format-Table PUAProtection` after applying; a value the engine does not read would fail silently.

Microsoft Edge has its own, separate PUA blocking for downloads; this tweak governs Defender's file-level detection.

#### Benefits
- **Covers a real gap**: regular signatures deliberately ignore this category.
- **Blocks rather than logs**: value 1 quarantines, where the current consumer default only audits.
- **Keeps the machine cleaner**: stops the junk that rides along with free installers.

#### Drawbacks
- **Occasional false positives**: some system utilities, crack-adjacent and "optimizer" tooling are classified as PUA, and you will need exclusions.
- **Classification can change**: a tool that works today can be quarantined after a signature update.
- **Smaller delta than it sounds**: modern Defender already audits PUA by default.
- **Needs Defender active**: inert if a third-party antivirus has taken over.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 (including IoT Enterprise LTSC 2021), all editions, with Microsoft Defender Antivirus active; Microsoft also documents it for Windows 8.1.
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Not configured" deletes the value. That does not switch PUA protection off: Defender returns to its own default, which is Audit (2) on a current, non-onboarded machine. Restore Snapshot writes back the captured pre-apply value.

#### Interactions
- [Enforce SmartScreen (apps and Edge)](#enforce-smartscreen-apps-and-edge) and [Defender Network Protection](#defender-network-protection) combined with this tweak make the machine noticeably more restrictive.
- [Defender cloud protection and MAPS](#defender-cloud-protection-and-maps) writes other values under the same Defender policy key (in the `Spynet` and `MpEngine` subkeys); no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the current default is Audit (2), not off, so the tweak changes auditing into blocking.
- **Confidence**: Microsoft-documented for the setting, its values and the default; the exact registry value name is not documented in Microsoft's prose (see below).
- **Reasoning**: the numeric states, the Group Policy setting and the Audit default are on Microsoft's PUA page. The registry value `PUAProtection` directly under `...\Windows Defender` matches the modern ADMX (the policy-hive audit lists it as class Machine in `WindowsDefender.admx`), but no tier A prose page names it. Open question: confirm on a clean machine that writing `PUAProtection` is reflected in `Get-MpPreference`, and whether the older `MpEnablePus` value is still honoured.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Enable it for most people; the cleanliness benefit outweighs the rare false positive. If you routinely run niche utilities that Defender dislikes, be ready to add exclusions.

#### Sources
1. Block potentially unwanted applications with Microsoft Defender Antivirus, establishes the 0 / 1 / 2 states, the Group Policy setting and the Audit default from security intelligence 1.329.495.0, https://learn.microsoft.com/en-us/defender-endpoint/detect-block-potentially-unwanted-apps-microsoft-defender-antivirus (tier A)
2. MagicX research `docs/superpowers/research/validation/_policy-hive-audit.md`, establishes `PUAProtection` as a class Machine value in `WindowsDefender.admx` and HKLM as correct

### Block LSASS credential theft (ASR rule)

`asr_block_lsass_theft` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns on the Defender attack surface reduction rule that stops Mimikatz-style tools reading credentials out of LSASS memory.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `asr_lsass` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules` → `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2` (REG_SZ) |
| `asr_policy` | shared | shared setting `defender_asr_policy_enabled` (see [Shared settings](#shared-settings)) |

| Option | `asr_lsass` | `asr_policy` |
|---|---|---|
| Block | `"1"` | `claim` |
| Off | `absent` | `unclaimed` |

System Default is shown when the rule value holds something else (for example `"2"` audit or `"0"`, set by another tool); Restore Snapshot writes back the captured pre-apply rule value. The shared `ExploitGuard_ASR_Rules` value is not in this tweak's snapshot; it is returned through the shared-setting claims record when the last ASR tweak releases it. On a stock machine neither value exists and the rule is not configured.

#### How it works

Attack surface reduction (ASR) rules are Microsoft Defender Antivirus behaviour rules, each identified by a GUID. They are configured through the Group Policy setting *Configure Attack Surface Reduction rules*: the parent value `ExploitGuard_ASR_Rules` = 1 under `...\Windows Defender Exploit Guard\ASR` marks the policy as configured, and each rule is a REG_SZ value under the `ASR\Rules` subkey whose name is the rule GUID and whose data is the mode: 0 off, 1 block, 2 audit, 5 not configured, 6 warn. REG_SZ is correct: ADMX list elements with `explicitValue="true"` write strings, which the shipped `WindowsDefender.admx` confirms.

This tweak writes rule `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2`, "Block credential stealing from the Windows local security authority subsystem", in block mode, and claims the shared parent value. The GUID ends in `e4b2`: a variant ending in `e4b0` circulates in older material and community scripts, maps to no rule, and is ignored silently by Defender. With the rule active, Defender denies unexpected processes the handle-open and memory-read access to `lsass.exe` that credential dumpers such as Mimikatz need. It blocks access to LSASS memory, not process execution, so a blocked entry for a system process such as `svchost.exe` in the log is usually benign.

Microsoft classes this as one of three "standard protection" rules that can be deployed directly in block mode without an audit period. This rule does not support warn mode (6). It does not honour Defender's general file and folder exclusions, only ASR-specific exclusions, and in audit mode it is very noisy (Chrome's updater and many tools that enumerate processes trip it), though Microsoft says almost all of that noise is safe to ignore. Quest DirSync Password Sync is a documented incompatibility.

Microsoft states the rule "isn't required" and "doesn't provide extra protection" when LSA protection (`RunAsPPL`) is enabled, and Defender for Endpoint marks it not applicable in that case, because a protected LSASS already refuses the same access.

ASR rules need Microsoft Defender Antivirus in active mode with real-time protection on. Group Policy management of ASR needs Pro or better; on Home, Microsoft documents configuring ASR only through PowerShell. Whether Defender enforces the policy registry values on Home is not confirmed by any tier A source (see Validation). Tamper Protection does not block these writes: ASR rules are not on Microsoft's list of tamper-protected settings.

#### Benefits
- **Mimikatz-class protection without VBS**: useful on machines that cannot run LSA protection or Credential Guard (for example because of incompatible drivers or hardware).
- **Safe to deploy directly**: a standard protection rule, no audit period needed.
- **Instantly reversible**: no reboot in either direction.
- **Policy shows as configured**: the shared parent value keeps `gpedit.msc` and policy refresh consistent with the rule.

#### Drawbacks
- **Redundant with LSA protection**: no extra protection when `RunAsPPL` is on.
- **Limited exclusions**: Defender file and folder exclusions do not apply to this rule.
- **Noisy logs**: many benign blocks are recorded.
- **Known incompatibility**: Quest DirSync Password Sync, and some IT or security tools that legitimately read LSASS.
- **Needs Defender active**: inert with a third-party antivirus.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 1803 and later (including IoT Enterprise LTSC 2021). Requires Microsoft Defender Antivirus in active mode with real-time protection. Home enforcement is unconfirmed.
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Off" deletes the rule value and releases the shared claim. If another ASR tweak still claims `ExploitGuard_ASR_Rules` it stays at 1 (reported as held by those tweaks); when the last claim is released, the value captured at the first claim is restored. Restore Snapshot writes back this tweak's captured rule value and releases the claim the same way.

#### Interactions
- Shares `defender_asr_policy_enabled` with [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors), [ASR standard protection rules](#asr-standard-protection-rules) and [ASR extended rule set](#asr-extended-rule-set); see [Shared settings](#shared-settings). All four write different GUIDs under the same `ASR\Rules` key, so they never conflict. The research lists them as a merge candidate.
- [ASR standard protection rules](#asr-standard-protection-rules) (`asr_standard_protection_rules`) carries Microsoft's other two standard protection rules; together with this tweak they form the full standard set.
- [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl), [Enable Credential Guard](#enable-credential-guard) and [Disable WDigest credential caching](#disable-wdigest-credential-caching) are overlapping LSASS protections (research merge candidate 3). With LSA protection on, this rule adds nothing.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. What the research had to correct: Microsoft's GUID for this rule ends `e4b2`, not `e4b0` (the `e4b0` form maps to no rule and Defender ignores it silently), and the ADMX parent value `ExploitGuard_ASR_Rules` = 1 must accompany the rule value. The tweak writes the `e4b2` GUID and claims the shared parent value.
- **Confidence**: Microsoft-documented. Both the ASR rules overview and the ASR rules reference (revised 2 July 2026) give `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2`; an independent gap-verification pass reached the same conclusion.
- **Reasoning**: the GUID, key, REG_SZ typing, mode semantics and the parent value were all checked against Microsoft Learn and the shipped `WindowsDefender.admx`. Open questions: an end-to-end check that `(Get-MpPreference).AttackSurfaceReductionRules_Ids` lists the rule after applying, and whether ASR rules are enforced on Windows 11 Home (treated as unsupported, not as false).
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
A good extra layer on Defender-protected machines where LSA protection is not enabled. If LSA protection or Credential Guard is already on, skip it rather than inheriting a second set of compatibility problems. If you run tools that legitimately read LSASS, test them after applying.

#### Sources
1. ASR rules overview (GUID table), establishes the rule GUID `9e6c4e1f-7d60-472f-ba1a-a39ef669e4b2` and the standard protection classification, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview (tier A)
2. ASR rules reference, establishes the rule's behaviour, exclusions, warn-mode support, LSA protection redundancy and known incompatibility, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
3. Configure ASR rules and exclusions, establishes the configuration routes and mode values, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-configure (tier A)
4. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (build 26100), policy `ExploitGuard_ASR_Rules`, establishes the parent value and the REG_SZ rule list (tier A, shipped ADMX)
5. MagicX research `docs/superpowers/research/validation/_verify-gaps-b-high.md`, establishes the independent finding that the `e4b0` GUID maps to no rule

### ASR rules: block Office/script malware vectors

`asr_block_office_script_vectors` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns on four Defender ASR rules that stop the Office-macro, obfuscated-script and email-attachment tricks behind most phishing attacks.**

#### What it changes

All four rule values live under `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`.

| Effect id | Kind | Target |
|---|---|---|
| `asr_office_child` | registry | `...\ASR\Rules` → `d4f940ab-401b-4efc-aadc-ad5f3c50688a` (REG_SZ): Block all Office applications from creating child processes |
| `asr_office_exe_content` | registry | `...\ASR\Rules` → `3b576869-a4ec-4529-8536-b80a7769e899` (REG_SZ): Block Office applications from creating executable content |
| `asr_obfuscated_scripts` | registry | `...\ASR\Rules` → `5beb7efe-fd9a-4556-801d-275e5ffc04cc` (REG_SZ): Block execution of potentially obfuscated scripts |
| `asr_email_exe` | registry | `...\ASR\Rules` → `be9ba2d9-53ea-4cdc-84e5-9b1eeee46550` (REG_SZ): Block executable content from email client and webmail |
| `asr_policy` | shared | shared setting `defender_asr_policy_enabled` (see [Shared settings](#shared-settings)) |

| Option | `asr_office_child` | `asr_office_exe_content` | `asr_obfuscated_scripts` | `asr_email_exe` | `asr_policy` |
|---|---|---|---|---|---|
| Block | `"1"` | `"1"` | `"1"` | `"1"` | `claim` |
| Audit only | `"2"` | `"2"` | `"2"` | `"2"` | `claim` |
| Off | `absent` | `absent` | `absent` | `absent` | `unclaimed` |

System Default is shown when the four rule values match none of the options, for example a mix of modes or only some rules present; Restore Snapshot writes back each captured rule value. The shared `ExploitGuard_ASR_Rules` value is returned through the shared-setting claims record, not the snapshot. On a stock machine none of the values exists.

#### How it works

The ASR mechanism (rule GUIDs as REG_SZ values under `ASR\Rules`, data 0 off, 1 block, 2 audit, 5 not configured, 6 warn, and the parent value `ExploitGuard_ASR_Rules` = 1 that makes the policy read as configured) is described under [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule) and [Shared settings](#shared-settings). This tweak sets four rules to the same mode:

- **Block all Office applications from creating child processes** (`d4f940ab-...688a`): Word, Excel, PowerPoint, OneNote and Access cannot start other processes, which stops a macro launching `cmd`, `powershell`, `mshta` or a downloaded payload.
- **Block Office applications from creating executable content** (`3b576869-...e899`): Office apps cannot write executable files to disk, which stops a macro dropping a payload. This rule has limited exclusion support and does not honour Defender file and folder exclusions.
- **Block execution of potentially obfuscated scripts** (`5beb7efe-...04cc`): Defender uses AMSI and cloud-delivered machine learning to stop scripts (PowerShell, JavaScript, VBScript and similar) whose content looks deliberately obfuscated. This rule additionally **requires cloud-delivered protection and AMSI**; with cloud protection off it is silently inert.
- **Block executable content from email client and webmail** (`be9ba2d9-...6550`): executable files and scripts (including `.zip` archives containing them) arriving through Outlook or webmail cannot be launched. It produces user notifications only when cloud protection is at level High or above.

The two Office child-process and executable-content rules are enforced only when Office is installed under `%ProgramFiles%` or `%ProgramFiles(x86)%`. Microsoft classifies all four as "other ASR rules" (as opposed to the three standard protection rules), which means its guidance is to audit before blocking; the tweak exposes "Audit only" for exactly that, and audit events appear in the Defender operational event log. All four rules need Microsoft Defender Antivirus active with real-time protection. On managed machines, Intune and Configuration Manager overwrite conflicting Group Policy at startup. Tamper Protection does not block the writes: ASR rules are not on Microsoft's list of tamper-protected settings.

The shared parent value matters: without `ExploitGuard_ASR_Rules` = 1 the policy renders as Not Configured in `gpedit.msc` even with the GUID values present, and because the Group Policy engine treats values under a policy key it owns as its own to manage, a later policy refresh can remove the orphaned rule values.

#### Benefits
- **Covers the dominant initial-access chains**: macro spawning a shell, macro dropping a payload, obfuscated script execution and email-borne executables.
- **Stops attacks at stage one**: before any payload runs.
- **Audit mode available**: see what would be blocked first.
- **Reversible instantly**: no reboot in either direction.

#### Drawbacks
- **Legitimate macros break**: workbooks whose macros shell out or write executables, and admin scripts that look obfuscated, are blocked.
- **The email rule catches archives**: `.zip` files containing executables are blocked too.
- **Two prerequisites, not one**: all four need real-time protection, and the obfuscated-scripts rule also needs cloud-delivered protection and AMSI.
- **Uneven exclusions**: the executable-content rule does not honour Defender file and folder exclusions.
- **Office location dependency**: portable or non-standard Office installs are not covered by the two Office rules.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 1709 and later (including IoT Enterprise LTSC 2021). Requires Microsoft Defender Antivirus in active mode. Home enforcement is unconfirmed (see Validation).
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Off" deletes all four rule values and releases the shared claim; `ExploitGuard_ASR_Rules` stays at 1 while another ASR tweak claims it and is restored to its captured original when the last claim is released. Restore Snapshot writes back each captured rule value and releases the claim the same way.

#### Interactions
- Shares `defender_asr_policy_enabled` with [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule), [ASR standard protection rules](#asr-standard-protection-rules) and [ASR extended rule set](#asr-extended-rule-set); different GUIDs, no conflict (research merge candidate 4).
- [Defender cloud protection and MAPS](#defender-cloud-protection-and-maps) (`defender_cloud_protection`) provides the cloud-delivered protection the obfuscated-scripts rule needs and the cloud level at which the email rule notifies.
- [Disable Windows Script Host](#disable-windows-script-host) removes the WSH host outright; this tweak blocks obfuscated scripts across script engines. They layer without conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: the ADMX parent value `ExploitGuard_ASR_Rules` = 1 must accompany the rule values, or the policy reads Not Configured and a refresh can strip the rules. The tweak claims it through the shared setting.
- **Confidence**: Microsoft-documented.
- **Reasoning**: all four GUIDs, the REG_SZ typing and the mode semantics were re-confirmed against Microsoft's current ASR rules table, and the parent value against the shipped `WindowsDefender.admx`. Tamper Protection was ruled out as a cause of the missing-parent problem. The research also required the cloud-delivered protection dependency to be disclosed, which the tweak's warning does. Open question shared with all ASR tweaks: enforcement on Windows 11 Home is unconfirmed.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
High value on typical home and office PCs, where these behaviours are almost always malicious. If your work depends on Office macros or scripted automation, run "Audit only" first, review the events, add ASR exclusions, then switch to "Block".

#### Sources
1. ASR rules overview (GUID table), establishes the four GUIDs, rule names and the "other ASR rules" classification, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview (tier A)
2. ASR rules reference (per-rule details and dependencies), establishes the cloud-delivered protection and AMSI dependency, the email rule's notification behaviour, the Office install-location condition and the exclusion limits, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
3. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` on build 26100.4061, policy `ExploitGuard_ASR_Rules`, establishes the parent value and that the `ASR\Rules` subkey is its list (tier A, shipped ADMX)
4. Configure ASR rules and exclusions in group policy, establishes the Group Policy route and the Intune and Configuration Manager precedence, https://learn.microsoft.com/en-us/defender-endpoint/enable-attack-surface-reduction (tier A)
5. Protect security settings with tamper protection, establishes that ASR rules are not tamper-protected settings, https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)

### Enforce SmartScreen (apps and Edge)

`enforce_smartscreen` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Forces SmartScreen on for downloaded apps and Edge, and turns the shell's "Run anyway" warning into a block.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `enable_smartscreen` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` → `EnableSmartScreen` (REG_DWORD) |
| `smartscreen_level` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` → `ShellSmartScreenLevel` (REG_SZ) |
| `edge_smartscreen` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Edge` → `SmartScreenEnabled` (REG_DWORD) |

| Option | `enable_smartscreen` | `smartscreen_level` | `edge_smartscreen` |
|---|---|---|---|
| Enforced (block) | `1` | `"Block"` | `1` |
| User's choice | `absent` | `absent` | `absent` |

System Default is shown when the three values match neither option, for example `ShellSmartScreenLevel` = "Warn" or a value set to 0 by another policy; Restore Snapshot writes back the captured pre-apply values. On a stock machine none of the three policy values exists, SmartScreen is on and in Warn mode, and the user controls it from Windows Security and Edge settings, which is the "User's choice" state.

#### How it works

Microsoft Defender SmartScreen checks the reputation of files and web addresses against Microsoft's cloud service. Two parts are configured here:

- **SmartScreen for apps and files in the Windows shell** (Explorer). The shipped `SmartScreen.admx` on build 26100 defines policy `ShellConfigureSmartScreen` at `Software\Policies\Microsoft\Windows\System` with `EnableSmartScreen` (1 enabled, 0 disabled) and an enum `ShellSmartScreenLevel` with two string items, `Block` and `Warn`. Because they are `<string>` enum items, the value is REG_SZ. With `Warn` (the default behaviour), an unrecognised downloaded app shows the "Windows protected your PC" dialog with a "More info, Run anyway" escape. With `Block`, that escape is removed: the user cannot run the file past the warning. Microsoft's SmartScreen Policy CSP gives `EnableSmartScreenInShell` a default of 1, so SmartScreen is on out of the box; what this tweak changes is Warn to Block, and it also stops the user switching SmartScreen off.
- **SmartScreen in Microsoft Edge (Chromium)**. `SmartScreenEnabled` = 1 under Edge's policy key forces Edge's SmartScreen on and prevents the user turning it off in Edge settings. The policy is class Both (it exists per user and per machine); the tweak writes the machine copy, which applies to every user. This value alone does not stop a user clicking past an Edge SmartScreen warning: Edge controls that with separate policies (`PreventSmartScreenPromptOverride` for sites and `PreventSmartScreenPromptOverrideForFiles` for downloads, documented on the same Edge policy page), which this tweak does not set. The "block, not warn" behaviour therefore applies to the Windows shell, while in Edge the tweak forces SmartScreen on.

Reputation lookups send file hashes and URLs to Microsoft. Policy values override the Windows Security app, which then shows these settings as managed by your organisation.

#### Benefits
- **Removes the reflex click-through in the shell**: unrecognised downloaded apps are blocked, not merely warned about.
- **Pins SmartScreen on**: neither the shell nor Edge SmartScreen can be switched off by the user or by software running as the user.
- **Well-proven**: SmartScreen is a mature, low-false-positive reputation service.

#### Drawbacks
- **Rare software is blocked outright**: in-house tools, niche utilities, new releases and self-compiled binaries have no reputation, and there is no user-visible way past the block.
- **Needs an administrator to unblock**: the block has no user-facing override, so getting an unrecognised download past it takes an administrator (for example to lift the policy).
- **Reputation data goes to Microsoft**: file hashes and URLs are sent for lookup, as they are by default.
- **Can make a machine feel locked down**: combined with PUA protection and Network Protection.
- **Edge click-through is not blocked**: see above.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 1703 and later (including IoT Enterprise LTSC 2021). The research states the Windows policy applies to Pro and above; the tweak is not gated by edition. The Edge value requires Microsoft Edge (Chromium).
- **Takes effect**: immediately for new downloads and new Edge sessions; no reboot.
- **Reverting**: "User's choice" deletes all three values, returning SmartScreen to the shipped Warn behaviour under user control. Restore Snapshot writes back the captured pre-apply values.

#### Interactions
- [Enhanced Phishing Protection](#enhanced-phishing-protection) (`enhanced_phishing_protection`) configures the password-protection component of SmartScreen, which this tweak never touches.
- [Defender Network Protection](#defender-network-protection) extends the same reputation checks to every process.
- Other tweaks write different values under `HKLM\SOFTWARE\Policies\Microsoft\Edge` (for example `debloat:disable_edge_first_run`, `debloat:disable_edge_startup_boost`, `debloat:disable_edge_sidebar`, `privacy:disable_edge_telemetry` and Edge AI settings in `ai.yaml`); none writes `SmartScreenEnabled`, so there is no conflict.

#### Validation
- **Verdict**: VERIFIED
- **Confidence**: Microsoft-documented, and confirmed against the shipped `SmartScreen.admx`.
- **Reasoning**: the REG_SZ typing of `ShellSmartScreenLevel` was the one open question and it was resolved from the shipped ADMX (the enum items are `<string>` elements, written as REG_SZ). The CSP default (on) confirms that deleting the values is a safe revert. The policy-hive audit confirmed HKLM is correct for the two Machine-class values and acceptable for the class-Both Edge value. The research did not examine Edge's prompt-override policies; the note above about Edge click-through comes from the Edge policy page the research cites.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Recommended for most users; SmartScreen is already on by default and the main change is removing the shell's "Run anyway". Skip it if you regularly run unsigned or low-prevalence software, or be ready to use an administrator account to get past a block.

#### Sources
1. SmartScreen Policy CSP, establishes the shell SmartScreen settings and the default of 1 (on), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-smartscreen (tier A)
2. Microsoft Edge browser policies, SmartScreenEnabled, establishes Edge's SmartScreen policy (and, on the same page, the separate prompt-override policies), https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#smartscreenenabled (tier A)
3. `C:\Windows\PolicyDefinitions\SmartScreen.admx` on build 26100.4061, policy `ShellConfigureSmartScreen`, establishes the key, `EnableSmartScreen` and the REG_SZ enum values "Block" and "Warn" (tier A, shipped ADMX)
4. MagicX research `docs/superpowers/research/validation/_policy-hive-audit.md`, establishes the ADMX classes (Machine for the shell values, Both for `SmartScreenEnabled`)

### Disable legacy TLS 1.0/1.1 (Schannel)

`disable_tls_legacy` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off TLS 1.0 and 1.1 in the Windows TLS stack, client and server, so everything that uses Schannel negotiates TLS 1.2 or 1.3.**

#### What it changes

All eight values live under `HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\SCHANNEL\Protocols`.

| Effect id | Kind | Target |
|---|---|---|
| `tls10_client_enabled` | registry | `...\Protocols\TLS 1.0\Client` → `Enabled` (REG_DWORD) |
| `tls10_client_disabled_by_default` | registry | `...\Protocols\TLS 1.0\Client` → `DisabledByDefault` (REG_DWORD) |
| `tls10_server_enabled` | registry | `...\Protocols\TLS 1.0\Server` → `Enabled` (REG_DWORD) |
| `tls10_server_disabled_by_default` | registry | `...\Protocols\TLS 1.0\Server` → `DisabledByDefault` (REG_DWORD) |
| `tls11_client_enabled` | registry | `...\Protocols\TLS 1.1\Client` → `Enabled` (REG_DWORD) |
| `tls11_client_disabled_by_default` | registry | `...\Protocols\TLS 1.1\Client` → `DisabledByDefault` (REG_DWORD) |
| `tls11_server_enabled` | registry | `...\Protocols\TLS 1.1\Server` → `Enabled` (REG_DWORD) |
| `tls11_server_disabled_by_default` | registry | `...\Protocols\TLS 1.1\Server` → `DisabledByDefault` (REG_DWORD) |

| Option | each `*_enabled` (4 values) | each `*_disabled_by_default` (4 values) |
|---|---|---|
| Disabled | `0` | `1` |
| Enabled | `absent` | `absent` |

System Default is shown when the eight values match neither option, for example when only TLS 1.0 is disabled, or when another tool wrote `Enabled` = 1; Restore Snapshot writes back each captured pre-apply value. On a stock machine these subkeys do not exist, and Schannel uses its built-in per-version defaults, which is the "Enabled" state.

#### How it works

Schannel is the Windows implementation of SSL, TLS and DTLS. Every program that uses the Windows TLS APIs (Edge's and Windows' own HTTPS, WinHTTP and WinINet, .NET Framework, RDP, LDAPS, IIS, SQL Server client connections, many third-party apps and services) negotiates its protocol version through Schannel. Microsoft documents per-version subkeys in the form `<protocol> <major>.<minor>\<Client|Server>` with two values:

- `Enabled` = 0 disables the version outright: "In order to override a system default and set a supported (D)TLS or SSL protocol version to the Disabled state, change the DWORD registry value of Enabled to 0 under the corresponding version-specific subkey."
- `DisabledByDefault` = 1 means the version is not used unless an application explicitly asks for it.

`Enabled` = 0 alone already forces the Disabled state per Microsoft's current documentation, but every Microsoft hardening example and the CIS and DISA STIG checks write both values, so the tweak writes both in each of the four subkeys (TLS 1.0 and 1.1, Client and Server). The Client subkeys govern outgoing connections from this PC; the Server subkeys govern connections this PC accepts (for example RDP, IIS or file sharing over TLS).

The change applies to credential handles opened by subsequent `AcquireCredentialsHandle` calls, so already-running programs and services keep their old handles; restarting them can be enough, and a reboot is the reliable superset, which is why the tweak is flagged reboot-required. Microsoft warns that narrowing the enabled protocol set can make `AcquireCredentialsHandle` fail outright if an application asks for a set of versions that becomes empty (for example an old app hard-coded to TLS 1.0 only), and warns against creating Schannel settings that are not explicitly documented; this tweak writes only documented values.

On Windows 11, TLS 1.0 and 1.1 are already off by default in current builds (the research dates this to 22H2), so on the primary target the tweak mostly pins an existing state. On Windows 10, including IoT Enterprise LTSC 2021, the older versions are still enabled by default, so there the tweak makes a real change. SSL 2.0, SSL 3.0 and DTLS are not touched. Software with its own TLS stack (OpenSSL, Java, NSS-based browsers such as Firefox, Go and many cross-platform tools) is not affected.

#### Benefits
- **Removes broken protocols**: TLS 1.0 and 1.1 have known cryptographic weaknesses.
- **Closes downgrade paths**: an attacker cannot force a connection onto a breakable version.
- **Covers services too**: applies to background services that ignore browser settings, and to inbound connections.
- **Compliance**: matches the CIS and STIG configuration (both values in all four subkeys).

#### Drawbacks
- **Legacy endpoints become unreachable**: internal appliances, printer and NAS web interfaces, and old servers that only offer TLS 1.0 or 1.1 stop working from every Schannel client on this PC.
- **Old clients cannot connect to this PC**: legacy clients that only speak TLS 1.0 or 1.1 cannot reach TLS services hosted here.
- **Can fail hard**: an application restricted to only the old versions gets an outright failure, not a fallback.
- **Largely pins an existing default on Windows 11**.
- **Other TLS stacks are not covered**.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021 (where it has the largest effect).
- **Takes effect**: after a reboot (or after restarting each affected application or service, since new credential handles pick up the change).
- **Reverting**: "Enabled" deletes all eight values, returning Schannel to its built-in defaults for the running Windows version. Restore Snapshot writes back each captured value. Both need the same reboot or restart to take effect.

#### Interactions
- [Force .NET strong crypto (TLS 1.2+)](#force-net-strong-crypto-tls-12) (`dotnet_strong_crypto`) makes legacy .NET Framework apps use Schannel's defaults and strong protocols. Together they keep old .NET apps from failing once TLS 1.0 and 1.1 are gone. The research considered merging the two but kept them separate because the .NET tweak is almost always safe while this one can cut off legacy servers.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: `DisabledByDefault` = 1 belongs alongside `Enabled` = 0 in each Client and Server subkey, matching every Microsoft example and the CIS and STIG baselines. The tweak writes both.
- **Confidence**: Microsoft-documented.
- **Reasoning**: the path format, the `Enabled` semantics, the fact that the subkeys do not exist by default (so deleting is the correct revert) and the `AcquireCredentialsHandle` warning are all on Microsoft's TLS registry settings page. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it on modern systems; almost everything uses TLS 1.2 or 1.3 already, and on Windows 10 it removes protocols that are still enabled. Hold off if you must reach legacy internal servers, appliances or printer interfaces stuck on TLS 1.0 or 1.1, or if old clients must connect to services on this PC.

#### Sources
1. Transport Layer Security (TLS) registry settings, establishes the subkey format, the `Enabled` and `DisabledByDefault` values, that the subkeys do not exist by default, the credential-handle timing and the `AcquireCredentialsHandle` and undocumented-settings warnings, https://learn.microsoft.com/en-us/windows-server/security/tls/tls-registry-settings (tier A)

### Force .NET strong crypto (TLS 1.2+)

`dotnet_strong_crypto` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Pushes legacy .NET Framework applications onto the operating system's strong TLS versions instead of SSL 3.0 or TLS 1.0.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `strong_crypto_64` | registry | `HKLM\SOFTWARE\Microsoft\.NETFramework\v4.0.30319` → `SchUseStrongCrypto` (REG_DWORD) |
| `strong_crypto_32` | registry | `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework\v4.0.30319` → `SchUseStrongCrypto` (REG_DWORD) |
| `default_tls_versions_64` | registry | `HKLM\SOFTWARE\Microsoft\.NETFramework\v4.0.30319` → `SystemDefaultTlsVersions` (REG_DWORD) |
| `default_tls_versions_32` | registry | `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework\v4.0.30319` → `SystemDefaultTlsVersions` (REG_DWORD) |

| Option | `strong_crypto_64` | `strong_crypto_32` | `default_tls_versions_64` | `default_tls_versions_32` |
|---|---|---|---|---|
| Strong crypto | `1` | `1` | `1` | `1` |
| Legacy crypto allowed | `absent` | `absent` | `absent` | `absent` |

System Default is shown when the four values match neither option, for example only one registry view set, or a value set to 0; Restore Snapshot writes back each captured pre-apply value. On a stock machine none of the four values exists (Microsoft: "These registry keys don't exist by default"), which is the "Legacy crypto allowed" state.

#### How it works

`v4.0.30319` is the registry key read by every .NET Framework 4.x application (4.0 through 4.8.1). Two values control how such an app uses Schannel for outgoing TLS connections:

- `SchUseStrongCrypto` = 1: "A value of 1 causes your app to use strong cryptography." .NET passes the `SCH_USE_STRONG_CRYPTO` flag to Schannel, which excludes known-weak cryptographic algorithms, cipher suites and protocol versions (SSL 3.0 and the like) from the connection.
- `SystemDefaultTlsVersions` = 1: the app lets the operating system choose the TLS versions instead of using a version list built into .NET. Combined with the OS's defaults, that means TLS 1.2 and 1.3 on Windows 11; on Windows 10, TLS 1.0 and 1.1 stay available unless [Disable legacy TLS 1.0/1.1 (Schannel)](#disable-legacy-tls-1011-schannel) is also applied.

32-bit .NET applications read the `Wow6432Node` copy of the key through the WOW64 registry redirector, so the tweak writes both views. Microsoft's reference `.reg` file sets both values in both views.

The effect on a current machine is smaller than it sounds. Build 26100 ships .NET Framework 4.8 (4.8.09032, release 533320 on 26100.4061). For apps compiled to target .NET Framework 4.6 or later, `SchUseStrongCrypto` already behaves as 1; for apps targeting 4.7 or later, `SystemDefaultTlsVersions` already behaves as 1. So on the primary target the tweak changes behaviour only for binaries targeting .NET Framework 4.5.2 or earlier. That is a genuine and shrinking set: old line-of-business tools, installers and utilities that fail against servers which dropped TLS 1.0.

Microsoft's reference file also covers `v2.0.50727` in both views, which is what .NET Framework 3.5 (and 2.0/3.0) applications read. This tweak does not write it, so .NET 3.5 applications are out of scope. Modern .NET (.NET Core and .NET 5 and later) does not read these values. The setting affects client (outgoing) connections only.

#### Benefits
- **Fixes broken connections as well as hardening**: legacy .NET apps that fail against servers which dropped old protocols start working again.
- **Security and compatibility at once**: rare for a hardening setting.
- **Both registry views covered**: 32-bit .NET apps are included.

#### Drawbacks
- **Rare legacy break**: an old in-house .NET app talking to an endpoint that only supports weak cryptography can stop connecting. Microsoft says the value should only be 0 "if you need to connect to legacy services that don't support strong cryptography and can't be upgraded".
- **Little effect on current builds**: only apps targeting .NET Framework 4.5.2 or earlier change behaviour on 24H2.
- **Needs app restarts**: applies to newly started processes.
- **.NET Framework 3.5 not covered**: `v2.0.50727` is not written.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 22H2 / IoT Enterprise LTSC 2021, all editions, with .NET Framework 4.x installed (it ships with Windows).
- **Takes effect**: for newly started .NET Framework processes; no reboot.
- **Reverting**: "Legacy crypto allowed" deletes all four values, the stock state. Apps targeting 4.6 or 4.7 and later still use strong crypto because that is their own default. Restore Snapshot writes back each captured value.

#### Interactions
- [Disable legacy TLS 1.0/1.1 (Schannel)](#disable-legacy-tls-1011-schannel) (`disable_tls_legacy`) is the OS-side counterpart; with `SystemDefaultTlsVersions` = 1 legacy .NET apps follow the Schannel protocol set that tweak defines (research merge candidate 8, kept separate on purpose).
- [Disable Windows Script Host](#disable-windows-script-host) uses the same two-view pattern for the same WOW64 reason.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction: Microsoft's reference configuration sets `SystemDefaultTlsVersions` = 1 alongside `SchUseStrongCrypto` in the same keys, and the tweak writes both.
- **Confidence**: Microsoft-documented.
- **Reasoning**: the key paths (including the `Wow6432Node` form), value semantics and the "don't exist by default" statement are verbatim on Microsoft's .NET TLS page, so deleting is the correct revert; the research also found neither value present in either view on a 26100.4061 install (only `AspNetEnforceViewStateMac`). The per-target-framework defaults explain the small effect on 24H2, which the research frames as a real but shrinking benefit, not a defect. The inclusion-principle review keeps the tweak despite the small visible delta. No open questions.
- **Tested**: Build validation (schema, ownership and conflict checks)

#### Recommendation
Apply it. It improves both security and compatibility for older .NET Framework software at negligible cost, though on a current 24H2 machine most applications already behave this way. The only reason to skip it is an old in-house app that must reach a weak-crypto-only endpoint.

#### Sources
1. Transport Layer Security (TLS) best practices with .NET Framework, establishes both values, both registry views, the per-target-framework defaults, the `v2.0.50727` reference entries and that the values do not exist by default, https://learn.microsoft.com/en-us/dotnet/framework/network-programming/tls (tier A)
2. On-box state of both `.NETFramework\v4.0.30319` registry views and the installed .NET Framework release on build 26100.4061 (corroborating: .NET Framework 4.8.09032, release 533320, neither value present)
3. MagicX research `docs/superpowers/research/validation/_inclusion-principle.md`, establishes that controls with little visible change today stay in the corpus

### Disable SMB insecure guest logons

`disable_smb_guest` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops this PC's SMB client from silently connecting to file shares as an anonymous, unauthenticated guest.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `allow_insecure_guest` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation`, value `AllowInsecureGuestAuth` (REG_DWORD) | none |

| Option | `allow_insecure_guest` |
|---|---|
| Refused | `0` |
| Allowed | `absent` |

System Default is shown when the value holds anything other than `0` or nothing at all (for example `1`, written by another tool or a Group Policy object); Restore Snapshot writes back the value captured before the first apply. Stock Windows ships with the value absent, so a stock machine reads as "Allowed". Note that "Allowed" only means "no policy is set": the edition's own default then decides, and several editions and builds already refuse guest logons without any policy (see below).

#### How it works

`AllowInsecureGuestAuth` is the registry value behind the Group Policy setting *Enable insecure guest logons* (ADMX `LanmanWorkstation.admx`, policy `Pol_EnableInsecureGuestLogons`, surfaced in MDM as `LanmanWorkstation/EnableInsecureGuestLogons`). It is read by the SMB 2 and SMB 3 client (the Workstation service). When the value is `0`, Microsoft states that "the SMB client will reject insecure guest logons": if a server offers only guest access, or falls back to guest after failing to authenticate the user, the client refuses instead of silently accepting an unauthenticated session.

The security point is that a guest session cannot be protected. Microsoft notes guest logons cannot use SMB signing or SMB encryption at all, so a rogue or spoofed server that offers guest access can pull the client into a session that is unauthenticated, unsigned and unencrypted, which is the entry point for SMB adversary-in-the-middle attacks. Refusing guest fallback closes that path and forces every share connection to use a real account.

The policy is class Machine (confirmed by the category's policy-hive audit), so HKLM is the correct and only hive. There is also a non-policy twin at `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters\AllowInsecureGuestAuth`; it is a separate store, and the policy value this tweak writes takes precedence over it.

Edition behaviour matters here. Guest credentials are already rejected by default on Windows 10 Enterprise, Pro for Workstations and Education, on Windows 11 Pro from build 25267 onward, and on Windows 11 24H2 where required SMB signing makes guest authentication fail anyway. Windows 10 Home and Pro still allow guest authentication by default. On those machines this tweak changes behaviour; on the others it pins a state Windows already enforces so that nothing can loosen it later.

#### Benefits
- **Blocks a rogue-server setup**: guest fallback is the entry point for SMB adversary-in-the-middle attacks.
- **Guest sessions cannot be protected**: Microsoft notes guest logons cannot use SMB signing or encryption, so refusing them removes an unprotected session type entirely.
- **Forces real credentials**: every share must be accessed with an account.
- **Pins the state**: on editions that already refuse guest logons, the policy stops a later change (by a tool or a script) from quietly re-enabling guest access.

#### Drawbacks
- **Guest-only NAS stops working**: cheap home NAS boxes and router USB shares fail with "your organization's security policies block unauthenticated guest access" until a real account is configured on the device.
- **No visible change on many machines**: Windows 10 Enterprise, Pro for Workstations and Education, Windows 11 Pro from build 25267 and Windows 11 24H2 with required signing already refuse guest logons.
- **Overlaps with SMB signing**: applying this and [Require SMB signing](#require-smb-signing) produces the same NAS breakage twice.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 (build 26100) and newer; also Windows 10 1803 and later, including Windows 10 IoT Enterprise LTSC 2021. All editions accept the policy.
- **Takes effect**: immediately for new SMB connections; no reboot.
- **Reverting**: "Allowed" deletes the value so the edition's own default applies again. Restore Snapshot writes back exactly what was captured before the first apply (usually absent). Neither can re-enable guest access on an edition that refuses it by default.

#### Interactions
- [Require SMB signing](#require-smb-signing): Microsoft couples the two directly ("Requiring SMB signing also disables guest access to shares"). The research lists both, plus the next item, as merge candidate 5 (SMB client hardening), because they cause the same NAS breakage and their warnings duplicate.
- [Block NTLM on the SMB client](#block-ntlm-on-the-smb-client): writes a different value (`BlockNTLM`) in the same `LanmanWorkstation` policy key. No ownership conflict; both tighten the SMB client and both can break home NAS access.
- [Remove SMBv1 protocol](#remove-smbv1-protocol): unrelated value, but the same class of old NAS devices tends to need both SMBv1 and guest access.

#### Validation
- **Verdict**: VERIFIED. The research found nothing to correct in the mechanism.
- **Confidence**: Microsoft-documented. The Policy CSP gives the registry key, value name and ADMX mapping exactly, and the SMB guest-logon article documents the semantics.
- **Reasoning**: The research matched key, value name and polarity (`0` = reject) against the LanmanWorkstation Policy CSP and confirmed the class Machine hive in the policy-hive audit. The inclusion-principle review asked only that the copy not editorialize about the tweak being "already the default" on some editions; that is stated here as a Drawback rather than as a reason to skip it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine, and give your shares proper credentials. Leave it at "Allowed" only if you depend on a device that can offer nothing but guest access and cannot be given a real account.

#### Sources
1. LanmanWorkstation Policy CSP, EnableInsecureGuestLogons: registry key, value name, ADMX mapping and the "reject insecure guest logons" semantics, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-lanmanworkstation (tier A)
2. Enable insecure guest logons in SMB2 and SMB3: why guest sessions cannot be signed or encrypted, the error message, and edition defaults, https://learn.microsoft.com/en-us/windows-server/storage/file-server/enable-insecure-guest-logons-smb2-and-smb3 (tier A)
3. MagicX research, policy-hive audit (`_policy-hive-audit.md`): `AllowInsecureGuestAuth` resolved to `LanmanWorkstation.admx` class Machine, so HKLM is correct (internal research record)

### Remove PowerShell 2.0 engine

`remove_powershell_v2` · Switch · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes the obsolete PowerShell 2.0 engine, so scripts can no longer be downgraded onto an engine with no AMSI scanning and no logging.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `ps2_feature` | action (PowerShell, timeout 900 s) | apply: `Disable-WindowsOptionalFeature -Online -FeatureName MicrosoftWindowsPowerShellV2Root -NoRestart -ErrorAction Stop`; undo: `Enable-WindowsOptionalFeature -Online -FeatureName MicrosoftWindowsPowerShellV2Root -NoRestart -All -ErrorAction Stop`; probe: exits 0 only when both `MicrosoftWindowsPowerShellV2Root` and `MicrosoftWindowsPowerShellV2` report `Disabled`, `DisablePending` or `DisabledWithPayloadRemoved` | has apply, undo and probe, so it is reversible and detectable |

| Option | `ps2_feature` |
|---|---|
| Removed | run |

System Default is shown while either feature is enabled, which is the stock state where the feature ships; selecting it after applying restores the snapshot, which runs the undo and reinstalls the feature. Detection reads the feature state itself, so a machine where PowerShell 2.0 is already disabled reads as "Removed" and a revert never installs it there.

#### How it works

`MicrosoftWindowsPowerShellV2Root` is the parent optional feature ("Windows PowerShell 2.0") and `MicrosoftWindowsPowerShellV2` is its child ("Windows PowerShell 2.0 Engine"). Disabling the parent disables both. With the engine gone, `powershell.exe -Version 2` no longer works. That switch is the classic downgrade evasion: the 2.0 engine predates AMSI (the Antimalware Scan Interface that lets Defender scan script content), script-block logging and module logging, so a script started under it runs unscanned and unlogged even on a machine where every modern PowerShell logging control is on.

The change is made through DISM's PowerShell cmdlets, which service the Windows image. That is why the action carries a 900-second timeout (servicing routinely takes minutes) and why a reboot is needed before the change is complete. `-NoRestart` suppresses DISM's own restart so the app controls the reboot prompt.

The probe checks both features, which is what DISA's check requires. Checking only the parent would let a half-finished servicing operation (parent disabled, engine still present) read as success. `DisablePending` (a disable that waits for a restart) and `DisabledWithPayloadRemoved` count as disabled too. The probe is written fail-safe: only the explicit "both Disabled" branch exits 0, and any query failure exits non-zero, so "could not tell" never reads as "removed". The category's probe review lists this tweak among the probes with the correct polarity.

PowerShell 2.0 is being withdrawn by Microsoft. DISA's Windows 11 STIG marks the requirement Not Applicable from 24H2 onward, and Microsoft has begun removing PowerShell 2.0 from Insider builds (Canary 27891 and later). On a build where the feature no longer exists, the apply script uses `-ErrorAction Stop`, so it fails with an error rather than quietly succeeding; the app surfaces that failure instead of reporting a change it did not make.

#### Benefits
- **Closes the standard downgrade evasion**: `-Version 2` is the classic way to run unlogged, unscanned PowerShell.
- **Every baseline requires it**: CIS and the DISA STIG both mandate disabling PowerShell 2.0.
- **Effectively no compatibility cost**: PowerShell 2.0 has been deprecated since 2017.
- **Makes the logging tweaks trustworthy**: script-block, module and transcription logging cannot be bypassed by a version downgrade.

#### Drawbacks
- **Needs a reboot**: an optional-feature change is not complete until you restart.
- **Very rare legacy break**: software that explicitly forces `-Version 2` stops working.
- **Fails where the feature is already gone**: on builds from which Microsoft has removed PowerShell 2.0, the disable command errors instead of succeeding, and the tweak has no build gate to hide it there.
- **Revert needs a second reboot**: reinstalling the feature is another servicing operation.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer where the feature is still present; also Windows 10 22H2, Windows 10 IoT Enterprise LTSC 2021 and Windows 11 up to 23H2, where the feature is present and enabled by default. All editions.
- **Takes effect**: after a reboot (servicing completes at restart).
- **Reverting**: turning the switch off (System Default) after applying runs the undo (`Enable-WindowsOptionalFeature ... -All`), which reinstalls the parent and its engine; a second reboot completes it.

#### Interactions
- [Enable PowerShell script-block logging](#enable-powershell-script-block-logging) and [PowerShell module logging and transcription](#powershell-module-logging-and-transcription): both log only on engines newer than 2.0. This tweak removes the engine that bypasses them, so the three are complementary.
- [Disable Windows Script Host](#disable-windows-script-host): a separate script host, unaffected.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the revert must actually reinstall the feature, that a per-user marker cannot track a machine-wide change, and that a complete check requires both `MicrosoftWindowsPowerShellV2Root` and `MicrosoftWindowsPowerShellV2` to report Disabled. The shipped tweak reads its state from both features directly, with no marker, and reinstalls them on revert.
- **Confidence**: Microsoft-documented. The cmdlets are documented by Microsoft, and the requirement and its check are defined by the DISA STIG.
- **Reasoning**: The feature names, the parent/child relationship and the downgrade rationale held under the adversarial pass. The open point is applicability: the research recommended a build gate or a graceful path for builds where Microsoft has removed the feature (Insider 27891 and later, and STIG Not Applicable from 24H2). The shipped tweak has no gate, so on such a build the apply fails with an error rather than silently succeeding, which is safe but not graceful.
- **Tested**: Build validation (schema, ownership and conflict checks); on build 26100 the disable and the re-enable each completed without a pending restart (`RestartNeeded` False), both features reading `Disabled` and then `Enabled` at once.

#### Recommendation
Apply it on almost every machine; the security gain is real and the compatibility cost is negligible. Skip it only if you know you run software that forces PowerShell 2.0.

#### Sources
1. Disable-WindowsOptionalFeature (DISM PowerShell module): the cmdlet, `-Online`, `-NoRestart` and optional-feature servicing, https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature (tier A)
2. DISA Windows 11 STIG V-253285, "PowerShell 2.0 must be disabled": the requirement and the check that both features report Disabled, https://www.stigviewer.com/stigs/microsoft_windows_11/2025-05-15/finding/V-253285 (tier B)

### Enforce the firewall on all profiles

`firewall_all_profiles` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Forces Windows Defender Firewall on for Domain, Private and Public networks with unsolicited inbound traffic blocked, and locks it there.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `domain_firewall` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile`, value `EnableFirewall` (REG_DWORD) | none |
| `domain_inbound` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile`, value `DefaultInboundAction` (REG_DWORD) | none |
| `private_firewall` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile`, value `EnableFirewall` (REG_DWORD) | none |
| `private_inbound` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile`, value `DefaultInboundAction` (REG_DWORD) | none |
| `public_firewall` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile`, value `EnableFirewall` (REG_DWORD) | none |
| `public_inbound` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile`, value `DefaultInboundAction` (REG_DWORD) | none |

| Option | `domain_firewall` | `domain_inbound` | `private_firewall` | `private_inbound` | `public_firewall` | `public_inbound` |
|---|---|---|---|---|---|---|
| Enforced on all profiles | `1` | `1` | `1` | `1` | `1` | `1` |
| User's choice | `absent` | `absent` | `absent` | `absent` | `absent` | `absent` |

System Default is shown when the six values match neither option (for example only some profiles set, or a value of `0` written by a Group Policy object); selecting it restores the values captured before the first apply. Stock Windows has no firewall policy values, so a stock machine reads as "User's choice"; the firewall itself is on by default with inbound blocked, controlled through the non-policy store.

#### How it works

Windows Defender Firewall keeps two configuration stores. The local (user-facing) store lives under `HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy`, where the profiles are named `DomainProfile`, `StandardProfile` and `PublicProfile`. The policy store lives under `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall`, where the Windows Defender Firewall with Advanced Security policy uses `DomainProfile`, `PrivateProfile` and `PublicProfile`. The firewall service reads both and a configured policy value overrides the local setting for that profile.

`EnableFirewall` = 1 turns the profile's firewall on; `DefaultInboundAction` = 1 sets the default for inbound connections that match no rule to Block. Allow rules (built-in and app-created) still apply, so the practical effect is default-deny for unsolicited inbound traffic, not a blanket block. Because the values are policy, the Windows Security app greys out the matching toggles and shows "This setting is managed by your administrator": the user cannot switch the firewall off until the policy is removed.

The DISA Windows Defender Firewall STIG check for the private profile reads exactly `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile\EnableFirewall`, naming the `SharedAccess` path only as a fallback, which is why the middle profile is written to `PrivateProfile`. Only `EnableFirewall` on Domain and Standard profiles is defined in the classic `WindowsFirewall.admx` (class Machine); `DefaultInboundAction` and the Private and Public profile values belong to the Windows Defender Firewall with Advanced Security client-side extension, which writes into the same policy store. Firewall profiles are per-machine, so there is no user-hive variant.

This tweak does not change `AllowLocalPolicyMerge`, so locally created allow rules keep working alongside the policy.

#### Benefits
- **Cannot be silently switched off**: the policy overrides the user-facing toggles, so malware or a careless click cannot disable a profile.
- **Default-deny inbound everywhere**: nothing reaches the PC unless a rule explicitly allows it.
- **Covers all three profiles**: Domain, Private (the one most home machines use) and Public.

#### Drawbacks
- **Toggles are greyed out**: users see "This setting is managed by your administrator" and cannot turn the firewall off for troubleshooting until the tweak is reverted.
- **P2P and LAN apps may need rules**: software that expects open inbound ports needs explicit allow rules.
- **Conflicts with third-party firewalls**: a product that manages the Windows firewall will fight this.
- **Usually already on**: the firewall is enabled with inbound blocked by default, so on a stock machine this mostly enforces rather than changes.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions.
- **Takes effect**: immediately; the firewall service picks up policy changes without a reboot.
- **Reverting**: "User's choice" deletes all six values, which returns control to the user and lets the firewall fall back to its non-policy (local) state. System Default restores exactly the captured values.

#### Interactions
- `network:firewall_logging_and_merge`: writes logging values and `AllowLocalPolicyMerge` under the same `WindowsFirewall` profile policy keys, including `PrivateProfile`. Different value names, so there is no ownership conflict.
- Any third-party firewall product that manages Windows Defender Firewall.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction is a fact about the policy path: under `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall` the Windows Defender Firewall with Advanced Security profile subkeys are `DomainProfile`, `PrivateProfile` and `PublicProfile`; `StandardProfile` is the subkey name used in the non-policy `SharedAccess\Parameters\FirewallPolicy` store.
- **Confidence**: Microsoft-documented. The Firewall CSP documents the profile settings, and the DISA STIG gives the exact private-profile policy path.
- **Reasoning**: The value names and polarities (1 = on, 1 = block) held. The policy-hive audit adds a nuance that is not fully settled: the classic `WindowsFirewall.admx` on 26100 does define `EnableFirewall` under a policy `StandardProfile` key (the legacy "Standard Profile" Group Policy), so the policy tree can carry both a legacy `StandardProfile` and a modern `PrivateProfile`. The shipped tweak follows the DISA check path (`PrivateProfile`); how the service reconciles the legacy key was not tested.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it; the firewall on with default-block inbound is baseline security. Be ready to add allow rules for any legitimate app that needs inbound access, and skip it if a third-party firewall manages this machine.

#### Sources
1. DISA Windows Defender Firewall with Advanced Security STIG V-241990: the private-profile check path under the policy key, with the `SharedAccess` path only as fallback, https://www.stigviewer.com/stigs/microsoft_windows_defender_firewall_with_advanced_security/2023-08-23/finding/V-241990 (tier B)
2. Firewall CSP: per-profile `EnableFirewall` and `DefaultInboundAction` settings, https://learn.microsoft.com/en-us/windows/client-management/mdm/firewall-csp (tier A)

### Enable logon/credential auditing

`audit_logon_events` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Records every successful and failed sign-in in the Security log, so password guessing and intrusions leave a trail.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `audit_logon` | action (PowerShell) | apply: reads the numeric setting of the Logon subcategory `{0CCE9215-69AE-11D9-BED3-505054503030}` from an `auditpol /backup` file (fails if it cannot), stores it as `AuditLogonPriorSetting` (REG_DWORD) under `HKLM\SOFTWARE\MagicXToolbox\State` unless a stash is already there, then runs `auditpol /set /subcategory:"{0CCE9215-...}" /success:enable /failure:enable`; undo: sets Success and Failure back from the stash (Success and Failure if there is none) and deletes the stash once `auditpol` succeeds; probe: exits 0 only when the numeric setting is 3 (Success and Failure) | default 30 s timeout; exit codes are `auditpol`'s own |

| Option | `audit_logon` |
|---|---|
| Success and failure | run |

This is a toggle: On is the option above, and Off is System Default. System Default is shown whenever the probe does not see both Success and Failure. A machine already auditing Success and Failure, which Windows clients are documented to ship with (and which the research machine showed), reads as "Success and failure" without any apply, so there is nothing to revert. Turning the toggle off after applying restores the snapshot: the action's undo puts the Logon subcategory back to the captured pre-apply setting. Windows 10 and 11 clients are documented as shipping with Logon auditing at Success and Failure, but that has not been confirmed on a clean 26100 image.

#### How it works

The advanced audit policy (the `auditpol` subcategories) decides which security events the Local Security Authority writes to the Security event log. The Logon subcategory of Logon/Logoff, GUID `{0CCE9215-69AE-11D9-BED3-505054503030}`, produces event 4624 (successful logon), 4625 (failed logon), 4648 (logon attempted with explicit credentials) and 4675. These are the events that brute-force, password-spray and lateral-movement detection is built on.

The tweak addresses the subcategory by GUID, not by its name, because `auditpol` subcategory names are translated on non-English Windows and a name-based command would fail there. `auditpol.exe` needs administrator rights.

The revert is designed not to leave the machine auditing less than before. The apply captures the pre-apply setting as a number (bit 1 Success, bit 2 Failure) before changing anything, and the undo writes each half back from it; with no stash it enables both, the documented client default. The number is read from the `Setting Value` column of `auditpol /backup`, because the text `auditpol /get` prints ("Success and Failure", "No Auditing") is translated on non-English Windows. A stash already present is kept, so re-applying never replaces the pre-tweak setting with this tweak's own. If the capture fails, the apply fails rather than guessing. This matters because a hardcoded Success-only revert would silently switch off failure auditing that Windows ships with.

The probe follows the fail-safe pattern: only a parsed setting of exactly 3 (Success and Failure) exits 0; anything else, including a failed read, exits 1.

Two things can override the setting. On a managed machine, an audit-policy Group Policy object overwrites whatever `auditpol` set at the next refresh. And the legacy (category-level) audit policy can override subcategory settings unless *Audit: Force audit policy subcategory settings to override audit policy category settings* (`SCENoApplyLegacyAuditPolicy`) is enabled.

#### Benefits
- **Backbone of detection**: 4624 and 4625 are what brute-force, password-spray and lateral-movement detection relies on.
- **Low volume**: Microsoft rates the event volume as "low on a client computer".
- **Language-independent**: the GUID works on any display language.
- **Safe revert**: the pre-apply setting is captured and restored rather than guessed.

#### Drawbacks
- **Security log fills faster**: Microsoft's guidance is to raise the log's maximum size, which this tweak does not do.
- **No value if nobody looks**: it adds evidence, not protection.
- **Managed machines override it**: an audit-policy GPO overwrites the `auditpol` setting at the next refresh.
- **Legacy audit policy can override it**: unless `SCENoApplyLegacyAuditPolicy` is enabled.
- **Often no visible change**: if the machine already audits Success and Failure, the tweak only pins that state.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions.
- **Takes effect**: immediately; the next sign-in event is audited. No reboot.
- **Reverting**: turning the toggle off after applying runs the undo, which restores the captured setting and removes the `AuditLogonPriorSetting` stash. If a Group Policy object has changed the subcategory since the apply, the undo still writes the captured pre-apply setting.

#### Interactions
- [Event log retention size](#event-log-retention-size): sizes the Security log this tweak fills. Pair them so events are not overwritten before you read them.
- [Log command lines in process-creation events](#log-command-lines-in-process-creation-events): uses `auditpol` on a different subcategory (Process Creation, `{0CCE922B-...}`) with the same capture-and-restore pattern; no collision.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the revert must restore the captured pre-apply inclusion setting (Windows clients ship auditing Success and Failure, so a Success-only undo lowers the audit level), and that a per-user marker cannot track a machine-wide change. The shipped tweak reads its state from the audit policy itself, with no marker, so a stock machine that already audits Success and Failure is never applied to or reverted.
- **Confidence**: Microsoft-documented. The Audit Logon article documents the events, the recommended Success and Failure setting and the volume; the `auditpol set` reference documents the command.
- **Reasoning**: The GUID, the event list and the admin requirement held under the adversarial pass. The probe was independently reviewed and has the correct fail-safe polarity. Open question: the exact shipped inclusion setting of the Logon subcategory on a clean 26100 install was not read (research UNKNOWN 8); the capture-and-restore design makes the revert correct regardless.
- **Tested**: Build validation (schema, ownership and conflict checks); on build 26100 the probe reads the machine's Success and Failure setting (3) as applied under Windows PowerShell 5.1.

#### Recommendation
Enable it if you want visibility into access attempts, and pair it with a larger Security log. If you never review event logs the benefit is limited, but the cost is close to zero.

#### Sources
1. Audit Logon: the subcategory, events 4624, 4625, 4648 and 4675, recommended Success and Failure on workstations, and "low on a client computer" volume, https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-logon (tier A)
2. auditpol set: command syntax for `/subcategory`, `/success` and `/failure`, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/auditpol-set (tier A)

### Lock the screen when idle

`lock_on_inactivity` · Switch · Risk: low · Elevation: none · Reboot: no · Windows: all supported builds · Reversible: yes

**Locks your session after ten idle minutes and asks for your password on return, so walking away does not leave the PC open.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `screensaver_active` | registry | `HKCU\Control Panel\Desktop`, value `ScreenSaveActive` (REG_SZ) | none |
| `screensaver_secure` | registry | `HKCU\Control Panel\Desktop`, value `ScreenSaverIsSecure` (REG_SZ) | none |
| `screensaver_timeout` | registry | `HKCU\Control Panel\Desktop`, value `ScreenSaveTimeOut` (REG_SZ) | seconds, as a string |
| `screensaver_exe` | registry | `HKCU\Control Panel\Desktop`, value `SCRNSAVE.EXE` (REG_SZ) | the screen saver executable |

| Option | `screensaver_active` | `screensaver_secure` | `screensaver_timeout` | `screensaver_exe` |
|---|---|---|---|---|
| Lock after 10 min | `"1"` | `"1"` | `"600"` | `%SystemRoot%\System32\scrnsave.scr` |

This is a toggle: On is the option above, and Off is System Default. System Default is shown whenever the four values do not all match; turning the toggle off restores the four values captured before the first apply, including deleting any that did not exist. None of these values has a documented Windows default, and on a stock Windows 11 profile the screen saver is "(None)" with `SCRNSAVE.EXE` unset.

#### How it works

The per-user screen saver settings drive the idle lock. Windows starts an idle timer for the signed-in session when a screen saver is active (`ScreenSaveActive` = "1") and a screen saver executable is selected (`SCRNSAVE.EXE`); after `ScreenSaveTimeOut` seconds of no input it starts the screen saver, and when `ScreenSaverIsSecure` = "1" ending the screen saver requires the user's password, which in practice locks the session.

`SCRNSAVE.EXE` is load-bearing. Microsoft's Group Policy screen saver troubleshooting article confirms that without a selected screen saver executable the idle timer never starts, so the other three values do nothing on their own. The tweak therefore selects `scrnsave.scr`, the screen saver shipped in `System32`.

All four values are written as strings (REG_SZ). The three screen saver switches must be strings: written as DWORDs, the idle lock never engages. That typing is corroborated by a community write-up of the same policy.

The values live in `HKCU`, so the tweak applies only to the account that applies it; it runs without elevation. A second account on the same machine is unaffected.

The machine-wide alternative is the security policy *Interactive logon: Machine inactivity limit* (`InactivityTimeoutSecs`, REG_DWORD seconds, under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`), which locks every user's session regardless of screen saver configuration. This tweak does not write it.

#### Benefits
- **Closes the walk-away window**: an unattended signed-in session locks itself.
- **Simple and instant**: no reboot, no service, no policy lock, no admin rights.
- **Adjustable**: the timeout is a number of seconds, changeable later in Settings.

#### Drawbacks
- **Per-user only**: other accounts on the same machine are unaffected, because the values live in the user hive.
- **Interrupts long reads and playback**: a ten-minute timeout can bite during video, reading or a presentation.
- **Physical threats only**: it does nothing against remote or malware attacks.
- **Replaces your screen saver choice**: any screen saver you had selected is replaced by `scrnsave.scr` until you revert.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions.
- **Takes effect**: at the next sign-in, or when Explorer re-reads user preferences; no reboot.
- **Reverting**: turning the toggle off restores the four captured values from the snapshot rather than writing literals, because none of them has a documented Windows default. It reverts only the account that applied it.

#### Interactions
None known in this corpus. A Group Policy screen saver or `InactivityTimeoutSecs` policy set by an administrator would govern alongside or instead of these per-user values.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the idle lock only engages when `SCRNSAVE.EXE` is set, so the tweak must select a screen saver, and that the revert must restore captured values because none of the four has a documented default.
- **Confidence**: Microsoft-documented. Microsoft's troubleshooting articles document the screen-saver-selected dependency; the REG_SZ typing is corroborated by a tier C source.
- **Reasoning**: The value names, their string typing and the per-user scope held under the adversarial pass. The stock values of all four on a fresh profile were not read on a clean image (research UNKNOWN 11); the snapshot-based revert makes that irrelevant to correctness. Not independently verified: whether Windows expands the `%SystemRoot%` variable inside a REG_SZ `SCRNSAVE.EXE` value.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on laptops and any PC used in shared or public spaces. On a physically secure home desktop the benefit is small; lengthen the timeout in Settings if ten minutes feels short. For every account on the machine at once, an administrator should use the machine inactivity limit instead.

#### Sources
1. Group Policy screensaver setting isn't working in Windows: the screen saver only runs when a screen saver executable is selected, https://learn.microsoft.com/en-us/troubleshoot/windows-client/group-policy/group-policy-screensaver-setting-not-work (tier A)
2. You cannot change or save screen saver settings: the per-user screen saver values, https://support.microsoft.com/en-us/help/968558/you-cannot-change-or-save-screen-saver-settings (tier A)
3. Auto Lock Computer Screen After Inactivity with GPO: confirms REG_SZ typing of all three values, https://woshub.com/windows-lock-screen-after-idle-via-gpo/ (tier C)

### Enable Credential Guard

`enable_credential_guard` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Moves NTLM hashes, Kerberos tickets and domain credentials out of LSASS into a hypervisor-isolated process that malware in the normal OS cannot read.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `lsa_cfg_flags` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`, value `LsaCfgFlags` (REG_DWORD) | the Credential Guard switch |
| `platform_security_features` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard`, value `RequirePlatformSecurityFeatures` (REG_DWORD) | VBS platform requirement |

| Option | `lsa_cfg_flags` | `platform_security_features` |
|---|---|---|
| Enabled | `2` | `1` |
| Off | `0` | `absent` |

System Default is shown when the two values match neither option. On many machines `LsaCfgFlags` is absent, so the tweak reads as System Default; on eligible Windows 11 22H2 and later installs Credential Guard may be on by default regardless. Selecting System Default restores the captured values (deleting any that did not exist). Note that "Off" is not the Windows default: `LsaCfgFlags` = 0 explicitly disables Credential Guard, including on machines where Windows had turned it on by default. To return to the Windows default, use System Default. Credential Guard is on by default on eligible Windows 11 22H2 and later installs.

#### How it works

Credential Guard uses virtualization-based security (VBS): the hypervisor runs a small isolated LSA process, `LsaIso.exe`, in a separate virtual trust level. NTLM password hashes, Kerberos ticket-granting tickets and credentials stored by applications as domain credentials are held there, and the normal LSA process (`lsass.exe`) only talks to it over a controlled channel. Code running in the normal OS, even with SYSTEM or kernel privileges, cannot read the isolated memory, which defeats pass-the-hash and pass-the-ticket tools that dump LSASS.

`LsaCfgFlags` under `Control\Lsa` is the registry route Microsoft documents. Its values are `0` (disabled), `1` (enabled with UEFI lock) and `2` (enabled without UEFI lock). The tweak uses `2`. Value `1` copies the setting into a UEFI firmware variable that no registry write can clear; removing it needs a boot-time `bcdedit` plus `SecConfig.efi` procedure with someone physically at the machine, which is incompatible with a reversible tweak. The same value name under `Control\DeviceGuard` is an OS state mirror, not an input, so writing there does nothing. The Group Policy route writes `LsaCfgFlags` under `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` instead; this tweak uses the direct registry route.

`RequirePlatformSecurityFeatures` = 1 under `Control\DeviceGuard` sets the VBS platform security requirement (the research lists 1 or 3 as valid). The tweak does not write `EnableVirtualizationBasedSecurity`; it relies on VBS being available and enabled, which it is by default on eligible Windows 11 22H2 and later hardware. Credential Guard cannot run while VBS is off.

The "Off" option must write `0` rather than delete: Microsoft states "Deleting these registry settings may not disable Credential Guard. They must be set to a value of 0."

Hardware and edition requirements: 64-bit Windows, Secure Boot, CPU virtualization extensions and VBS. Credential Guard is available on Enterprise and Education (and Pro under some licensing), not on Home. The tweak carries no edition gate, so on Home it can be applied but does nothing.

#### Benefits
- **Defeats pass-the-hash**: the secrets are not in LSASS memory to steal.
- **Hardware-backed**: isolation is enforced by the hypervisor, not by process permissions.
- **Strongest option for domain credentials**: on a domain-joined or Entra-joined machine this is the top of the credential-protection ladder.

#### Drawbacks
- **Breaks legacy protocols**: unconstrained Kerberos delegation, DES and RC4 Kerberos encryption, NTLMv1 and MS-CHAPv2 for some VPN configurations stop working, as do credential providers and security packages that load into LSA without meeting the requirements.
- **Requires VBS**: it cannot run if virtualization-based security is off, so it directly conflicts with any tweak that disables VBS or memory integrity.
- **Costs memory and, on older hardware, performance**.
- **Little benefit standalone**: on a home PC signed in with a local or personal Microsoft account there are few domain secrets to protect.
- **"Off" really turns it off**: on a machine where Windows enabled Credential Guard by default, choosing "Off" disables it; use System Default to go back to what Windows chose.
- **Not available on Home**, even though the tweak is offered there.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 (including IoT Enterprise LTSC 2021), Enterprise and Education (and Pro under some licensing); not Home. Requires 64-bit, Secure Boot, virtualization extensions and VBS.
- **Takes effect**: after a reboot; LSA and the isolated process start at boot.
- **Reverting**: "Off" writes `LsaCfgFlags` = 0 and deletes `RequirePlatformSecurityFeatures`, and needs a reboot. System Default restores the captured values. Because the tweak uses value 2, no UEFI variable is written and the revert is complete after the reboot.

#### Interactions
- `performance:disable_vbs_hvci`: writes `EnableVirtualizationBasedSecurity` = 0 in the same `Control\DeviceGuard` key. Credential Guard is hosted by VBS and cannot run once that tweak is applied. Never enable both; both tweaks carry warnings saying so.
- [Enable LSA protection (RunAsPPL)](#enable-lsa-protection-runasppl), [Disable WDigest credential caching](#disable-wdigest-credential-caching) and [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule): overlapping LSASS credential protections at escalating strength (research merge candidate 3). Enabling all four inherits all four sets of compatibility constraints.
- [Enforce NTLMv2 only](#enforce-ntlmv2-only) and [Restrict outgoing NTLM](#restrict-outgoing-ntlm): Credential Guard already makes NTLMv1 unusable for protected credentials.

#### Validation
- **Verdict**: INCORRECT as researched; the shipped tweak implements the research's corrected form. The research found the mechanism facts that make this work: `LsaCfgFlags` is read under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` (or the `Policies\Microsoft\Windows\DeviceGuard` policy key), not under `Control\DeviceGuard`; the reversible enabled value is `2`, not `1`; and disabling requires writing `0`, not deleting.
- **Confidence**: Microsoft-documented. Microsoft's Configure Credential Guard page documents the keys, the values, the UEFI lock and the "set to 0" requirement.
- **Reasoning**: The shipped effects use the Microsoft-documented key, value 2 and a 0 revert. Two research recommendations are not in the shipped tweak: the research's registry route also sets `EnableVirtualizationBasedSecurity` = 1 under `Control\DeviceGuard`, which the tweak omits (it relies on VBS being on already), and it asked for an edition gate excluding Home, which the tweak lacks. The cross-category finding that fixing the key activates a real conflict with `performance:disable_vbs_hvci` is handled by warnings on both tweaks.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on domain-joined or Entra-joined Windows 11 Enterprise or Education machines that meet the hardware requirements and do not rely on the legacy protocols above. On a standalone home PC the legacy-protocol restrictions cost more than the protection returns, and on Home it does nothing. Never pair it with the performance tweak that disables VBS.

#### Sources
1. Configure Credential Guard: `LsaCfgFlags` keys and values, the UEFI lock, the requirement to set 0 rather than delete, requirements and default enablement, https://learn.microsoft.com/en-us/windows/security/identity-protection/credential-guard/configure (tier A)
2. Configure added LSA protection: the interaction between LSA protection and Credential Guard and default enablement, https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection (tier A)
3. MagicX research, cross-category finding 1: the VBS conflict with `performance:disable_vbs_hvci` (internal research record)

### Defender cloud protection and MAPS

`defender_cloud_protection` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns Microsoft Defender's cloud lookups up to the High block level so brand-new malware can be blocked the first time it is seen.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `spynet_reporting` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet`, value `SpynetReporting` (REG_DWORD) | MAPS membership |
| `submit_samples_consent` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet`, value `SubmitSamplesConsent` (REG_DWORD) | sample submission |
| `block_at_first_seen` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet`, value `DisableBlockAtFirstSeen` (REG_DWORD) | Block at First Sight (inverted name) |
| `cloud_block_level` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine`, value `MpCloudBlockLevel` (REG_DWORD) | cloud block level |
| `bafs_extended_timeout` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\MpEngine`, value `MpBafsExtendedTimeout` (REG_DWORD) | extra seconds a file is held for a cloud verdict |

| Option | `spynet_reporting` | `submit_samples_consent` | `block_at_first_seen` | `cloud_block_level` | `bafs_extended_timeout` |
|---|---|---|---|---|---|
| High, send safe samples | `2` | `1` | `0` | `2` | `50` |
| High, prompt before sending samples | `2` | `0` | `0` | `2` | `50` |
| User's choice | `absent` | `absent` | `absent` | `absent` | `absent` |

System Default is shown when the five values match no option (for example a partial set written by another tool); selecting it restores the captured values. Stock Windows has none of these policy values, so a stock machine reads as "User's choice", where Defender's own defaults and the Windows Security app decide. Those defaults are: cloud protection on (forced on by Tamper Protection on consumer Windows 11) and sample submission at Send safe samples, which Microsoft calls "the default, recommended setting".

#### How it works

All five are Microsoft Defender Antivirus policy values, class Machine, confirmed verbatim in the shipped `WindowsDefender.admx` on 26100. Defender's engine reads them and they override the corresponding Windows Security app settings.

- `SpynetReporting` sets Microsoft Active Protection Service (MAPS) membership: 0 Disabled, 1 Basic, 2 Advanced. MAPS is the cloud-delivered protection channel.
- `SubmitSamplesConsent` sets sample submission: 0 Always Prompt, 1 Send safe samples, 2 Never Send, 3 Send all samples.
- `DisableBlockAtFirstSeen` controls Block at First Sight, which holds a new, unknown file while Defender asks the cloud for a verdict. The ADMX enabled value is 0 and disabled value is 1, so `0` means Block at First Sight is on.
- `MpCloudBlockLevel` sets how aggressively the cloud blocks suspicious files: 0 Default, 1 Moderate, 2 High, 4 High Plus, 6 Zero Tolerance. The tweak uses High, which Microsoft documents as carrying "a greater chance of false positives".
- `MpBafsExtendedTimeout` adds up to 50 seconds (the ADMX range is 0 to 50) during which a file is held while the cloud check completes. The tweak uses the maximum, 50.

Tamper Protection changes what actually lands. It is on by default on consumer Windows 11 and lists "Cloud protection remains enabled" among the settings it protects, and Microsoft states "any changes made to tamper-protected settings are ignored". `SpynetReporting` and `DisableBlockAtFirstSeen` sit on that protected surface, so the policy write lands in the registry while the effective setting does not move (it is already on). `MpCloudBlockLevel` and `MpBafsExtendedTimeout` are not on Microsoft's list of tamper-protected settings and are what actually delivers new behaviour. The tweak's post-apply check reads the registry, so it confirms the policy values were written, not the effective Defender state; check the effective state with `Get-MpPreference`.

The second option uses Always Prompt (`SubmitSamplesConsent` = 0) rather than Never Send, because Microsoft states "the NeverSend setting means that the Block at First Sight feature ... won't work"; an option offering Block at First Sight together with Never Send would contradict itself.

The one feature in this category that genuinely depends on cloud-delivered protection is Defender Network Protection: its requirements table names real-time protection, behavior monitoring and cloud-delivered protection. None of the ASR rules this category ships is cloud-gated (each lists only "Microsoft Defender Antivirus" as its dependency), and Controlled Folder Access needs real-time protection, not cloud protection.

Everything here requires Microsoft Defender Antivirus to be the active antivirus. With a third-party antivirus installed, Defender runs in passive mode and none of these values applies.

#### Benefits
- **Catches new malware**: cloud lookups classify files that have no signature yet, and Block at First Sight holds them until a verdict arrives.
- **High block level is the real change**: `MpCloudBlockLevel` and the extended timeout are the two values Tamper Protection does not already force.
- **Supports Network Protection**: [Defender Network Protection](#defender-network-protection) requires cloud-delivered protection.
- **Pins the settings**: the policy stops malware or a user from lowering MAPS membership in the Windows Security app.

#### Drawbacks
- **More false positives**: Microsoft documents block level High as carrying "a greater chance of false positives".
- **Sends files to Microsoft**: Advanced MAPS and safe-sample submission upload information and suspicious files, a direct trade against a privacy posture. The prompt option asks first but still uses Advanced MAPS.
- **Partly already on**: Tamper Protection forces cloud protection on by default on consumer Windows 11, and Send safe samples is already the default, so under "High, send safe samples" three of the five writes can land without the effective setting moving (two under the prompt option).
- **Inert without Defender**: with a third-party antivirus Defender goes passive and none of this applies.
- **Registry check, not effective state**: the app verifies the policy values, not what Defender actually enforces.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. Requires Microsoft Defender Antivirus as the active antivirus. No edition restriction in the sources.
- **Takes effect**: immediately; Defender picks up policy changes without a reboot.
- **Reverting**: "User's choice" deletes all five values, returning Defender to its own defaults and unlocking the Windows Security controls. System Default restores the captured values. Files already uploaded to Microsoft are not recalled.

#### Interactions
- [Defender Network Protection](#defender-network-protection): depends on cloud-delivered protection.
- [ASR standard protection rules](#asr-standard-protection-rules), [ASR extended rule set](#asr-extended-rule-set), [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule), [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors): do not depend on this tweak.
- [Controlled Folder Access (ransomware shield)](#controlled-folder-access-ransomware-shield): needs real-time protection, not cloud protection.
- Any "turn MAPS off" privacy setting is the opposite of this tweak; the two are mutually exclusive. No such tweak ships in this corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that Tamper Protection can make the `SpynetReporting` and `DisableBlockAtFirstSeen` writes inert while `MpCloudBlockLevel` and `MpBafsExtendedTimeout` deliver the real change; that Never Send disables Block at First Sight, so a no-submission option must not use it; that Send safe samples is already the default; and that the dependency that holds is Network Protection, not ASR.
- **Confidence**: Microsoft-documented. All five values, enums and ranges come from the shipped ADMX; the defaults, Tamper Protection behaviour and Never Send caveat come from Microsoft Learn.
- **Reasoning**: Every value name, type, key and enum member matched the shipped ADMX exactly, and 50 sits on the ADMX maximum. Open questions: whether Tamper Protection blocks `MpCloudBlockLevel` specifically (Microsoft's list names "Cloud protection remains enabled" but not the block level; research UNKNOWN 15). The research also asked for a probe that reads `Get-MpPreference` or a Defender-presence gate; the shipped tweak verifies the registry values and carries a warning instead.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "High, send safe samples" if Defender is your antivirus and you want maximum detection, and you accept that suspicious files are uploaded for analysis. If automatic uploads are unacceptable, use "High, prompt before sending samples". Skip it if you run a third-party antivirus or hold a strict privacy posture that turns MAPS off.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policies `SpynetReporting`, `SubmitSamplesConsent`, `DisableBlockAtFirstSeen`, `MpEngine_MpCloudBlockLevel`, `MpEngine_MpBafsExtendedTimeout`: keys, value names, enums and the 0 to 50 range (tier A, shipped ADMX)
2. Turn on cloud protection in Microsoft Defender Antivirus: MAPS, sample submission defaults and the Never Send caveat, https://learn.microsoft.com/en-us/defender-endpoint/enable-cloud-protection-microsoft-defender-antivirus (tier A)
3. Specify the cloud protection level: block levels and the false-positive trade-off, https://learn.microsoft.com/en-us/defender-endpoint/specify-cloud-protection-level-microsoft-defender-antivirus (tier A)
4. Protect security settings with tamper protection: protected settings and "changes made to tamper-protected settings are ignored", https://learn.microsoft.com/en-us/defender-endpoint/prevent-changes-to-security-settings-with-tamper-protection (tier A)
5. Attack surface reduction (ASR) rules reference: per-rule Dependencies fields, none cloud-gated for the rules this corpus ships, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
6. Use network protection, Requirements section: needs real-time protection, behavior monitoring and cloud-delivered protection, https://learn.microsoft.com/en-us/defender-endpoint/network-protection (tier A)

### Block NTLM on the SMB client

`smb_client_block_ntlm` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: build 26100 and newer · Reversible: yes

**Stops this PC ever sending an NTLM response to an SMB server, which kills NTLM relay and coerced authentication at the source.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `block_ntlm` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation`, value `BlockNTLM` (REG_DWORD) | none |

| Option | `block_ntlm` |
|---|---|
| Blocked | `1` |
| Allowed | `absent` |

System Default is shown when the value holds anything other than `1` or nothing (for example `0`, written by a Group Policy object); selecting it restores the captured value. Stock Windows has no value, so a stock machine reads as "Allowed", and the SMB client uses NTLM when Kerberos is not available.

#### How it works

`BlockNTLM` is the registry value behind the Group Policy setting Computer Configuration > Administrative Templates > Network > Lanman Workstation > *Block NTLM (LM, NTLM, NTLMv2)*, introduced with Windows 11 24H2 and Windows Server 2025. The shipped 26100 `LanmanWorkstation.admx` defines it as policy `Pol_BlockNTLM`, class Machine, enabled value 1, disabled value 0, supported on "At least Windows Server 2025, Windows 11". Its help text: "This policy controls if the SMB client will block NTLM for remote connection authentication. If you enable this policy setting, the SMB client won't use NTLM for remote connection authentication."

With the value at 1, the SMB client only authenticates with Kerberos. An attacker who tricks the machine into connecting to a hostile SMB server (a coerced-authentication or relay attack) gets no NTLM challenge response to crack or relay. This is stronger than choosing an NTLM variant with `LmCompatibilityLevel`: that picks which NTLM is used, while this stops NTLM being offered to SMB servers at all.

Kerberos needs a name the domain's Key Distribution Center can resolve to a service principal, so connections by IP address, to workgroup machines and to devices with no Kerberos support fall back to NTLM and are therefore refused.

There are two stores. `Set-SmbClientConfiguration -BlockNTLM $true` writes the non-policy value `HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters\BlockNTLM`; the policy key this tweak writes is the documented Group Policy surface and takes precedence. Shipped binaries corroborate the design: `smbwmiv2.dll` (the WMI provider behind the cmdlet) reads both locations and has a reset-to-default path for `BlockNTLM`; `mrxsmb20.sys` distinguishes a global setting from a per-mapping setting.

Deleting the value (not writing 0) is the clean revert: 0 is the ADMX disabled value, meaning "policy explicitly allows NTLM", not "unconfigured".

A companion policy, `BlockNTLMServerExceptionList` (REG_MULTI_SZ, same key, one server name per entry), lets you allow specific servers. The tweak does not manage it, so any exception list you configure yourself is left untouched. A per-mapping override also exists: `New-SmbMapping -BlockNTLM $false`.

#### Benefits
- **Kills coerced authentication**: the machine cannot be tricked into sending NTLM challenge responses to a hostile SMB server.
- **Stronger than picking an NTLM version**: it stops NTLM being offered at all for SMB.
- **Has escape hatches**: the server exception list and the per-mapping override let you keep specific legacy targets working.
- **Cannot lock you out**: it governs outbound SMB client behaviour only.

#### Drawbacks
- **Breaks anything that cannot do Kerberos**: NAS devices reached by IP address rather than name, workgroup file shares, and older network printers with scan-to-folder all stop working.
- **Easy to underestimate**: home networks lean heavily on IP-address SMB, which is exactly what this blocks.
- **24H2 and newer only**: the control does not exist on Windows 10 or LTSC 2021.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 (build 26100) and newer, and Windows Server 2025; the tweak is hidden on older builds, including Windows 10 IoT Enterprise LTSC 2021. No edition restriction in the sources.
- **Takes effect**: immediately for new SMB connections; no reboot.
- **Reverting**: "Allowed" deletes the value, restoring stock behaviour. System Default restores the captured value. The exception list is never touched.

#### Interactions
- [Disable SMB insecure guest logons](#disable-smb-insecure-guest-logons) and [Require SMB signing](#require-smb-signing): research merge candidate 5 (SMB client hardening). All three can break the same home NAS setups. This tweak shares the `LanmanWorkstation` policy key with the guest-logon tweak but writes a different value.
- [Enforce NTLMv2 only](#enforce-ntlmv2-only) and [Restrict outgoing NTLM](#restrict-outgoing-ntlm): OS-wide NTLM controls; this one is SMB-specific and stricter for SMB.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that this is a plain registry policy value present in the shipped 26100 `LanmanWorkstation.admx`, so no PowerShell action is needed, and that the revert must delete the value rather than write 0.
- **Confidence**: Microsoft-documented. The shipped ADMX and Microsoft's SMB NTLM blocking page document the policy; shipped binaries corroborate the store layout.
- **Reasoning**: The adversarial pass refuted the gap proposal's claim that the policy was missing from the 26100 ADMX by parsing the shipped file. The `>=26100` gate matches Microsoft ("Starting with Windows Server 2025 and Windows 11, version 24H2"). No lockout path exists because the setting is outbound only.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if all your SMB targets are reachable by name and can do Kerberos, which usually means a domain or a modern NAS with proper DNS. Skip it on a typical home network that mounts shares by IP address, or add those servers to the exception list first.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\LanmanWorkstation.admx` and `en-US\LanmanWorkstation.adml` (26100), policies `Pol_BlockNTLM` and `Pol_BlockNTLMServerExceptionList`: key, value, enabled and disabled values, supported-on string and help text (tier A, shipped ADMX)
2. Block NTLM connections on SMB, Group Policy tab: the Group Policy path, applicability and exception mechanisms, https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-ntlm-blocking (tier A)
3. What's new in Windows 11, version 24H2: SMB NTLM blocking introduced in 24H2, https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-24h2 (cited in the tweak's evidence; no tier given in the research)
4. UTF-16 string extraction from shipped 26100 `smbwmiv2.dll`, `wkssvc.dll`, `mrxsmb20.sys`, `mrxsmb.sys`: both stores and the reset-to-default path exist (tier A, shipped binaries)

### Enhanced Phishing Protection

`enhanced_phishing_protection` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: Windows 11 only · Reversible: yes

**Warns you when your work or school password is typed into a phishing site, reused on a website, or typed into an unsafe app such as Notepad.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `wtds_service_enabled` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WTDS\Components`, value `ServiceEnabled` (REG_DWORD) | none |
| `wtds_notify_malicious` | registry | same key, value `NotifyMalicious` (REG_DWORD) | none |
| `wtds_notify_password_reuse` | registry | same key, value `NotifyPasswordReuse` (REG_DWORD) | none |
| `wtds_notify_unsafe_app` | registry | same key, value `NotifyUnsafeApp` (REG_DWORD) | none |
| `wtds_capture_threat_window` | registry | same key, value `CaptureThreatWindow` (REG_DWORD) | written by the ADMX policy named `AutomaticDataCollection` |

| Option | `wtds_service_enabled` | `wtds_notify_malicious` | `wtds_notify_password_reuse` | `wtds_notify_unsafe_app` | `wtds_capture_threat_window` |
|---|---|---|---|---|---|
| Warnings on, no screenshot upload | `1` | `1` | `1` | `1` | `0` |
| Warnings on, with screenshot upload | `1` | `1` | `1` | `1` | `1` |
| Not configured | `absent` | `absent` | `absent` | `absent` | `absent` |

System Default is shown when the five values match no option; selecting it restores the captured values. Stock Windows has none of these policy values, so a stock machine reads as "Not configured". "Not configured" means "no policy": the shipped behaviour then applies, which is not fully off. The effective defaults on a consumer 26100 machine are:

| Value | Effective default with no policy |
|---|---|
| `ServiceEnabled` | Enabled (Policy CSP "Default Value: 1") |
| `NotifyMalicious` | Enabled for devices not onboarded to Defender for Endpoint |
| `NotifyPasswordReuse` | Disabled |
| `NotifyUnsafeApp` | Disabled |
| `CaptureThreatWindow` | Disabled unless domain-joined or MDM-enrolled |

#### How it works

Enhanced Phishing Protection is a component of Microsoft Defender SmartScreen (the Web Threat Defense service, WTDS) that watches where a Microsoft work or school password is typed. All five values are defined in the shipped 26100 `WebThreatDefense.admx`, class Machine, enabled value 1, disabled value 0, supported from Windows 11 22H2.

- `ServiceEnabled` = 1 turns the component on **in audit mode** and stops users turning it off in the Windows Security app. The shipped ADML: "If you enable this policy setting, Enhanced Phishing Protection in Microsoft Defender SmartScreen is enabled in audit mode and your users are unable to turn it off." It does not by itself show warnings.
- `NotifyMalicious` warns when the password is typed into a site or app SmartScreen considers malicious or phishing.
- `NotifyPasswordReuse` warns when the work or school password is reused on a website.
- `NotifyUnsafeApp` warns when the password is typed into Notepad or Microsoft 365 Office apps, where it would be stored in plain text.
- `CaptureThreatWindow` (ADMX policy `AutomaticDataCollection`) sends a screenshot of the offending window to Microsoft for analysis when the password is entered somewhere unsafe.

Scope is the key limit: Microsoft states the feature "helps protect Microsoft school or work passwords". On a machine signed in with a local account or a personal Microsoft account there is no work or school password to watch, and the feature does effectively nothing.

On a consumer machine, `ServiceEnabled` and `NotifyMalicious` are already effectively on, so writing them changes nothing except locking the UI toggle. The genuine behavioural additions are `NotifyPasswordReuse` and `NotifyUnsafeApp`. The first option writes `CaptureThreatWindow` = 0, which explicitly disables screenshot upload even on domain-joined or MDM-enrolled machines where it would otherwise be on.

#### Benefits
- **Catches password reuse**: warns when a work or school password is typed into a website.
- **Catches unsafe storage**: warns when that password is typed into Notepad or Office apps.
- **Two values genuinely change behaviour**: the password-reuse and unsafe-app warnings are off by default on a consumer machine.
- **Screenshot choice is explicit**: one option pins screenshot upload off, the other turns it on.

#### Drawbacks
- **Work and school passwords only**: with a local account or a personal Microsoft account the feature has nothing to protect.
- **Locks the UI toggle**: with `ServiceEnabled` written, users can no longer turn the feature off in Windows Security.
- **Screenshot upload is a privacy cost**: the "with screenshot upload" option sends window captures to Microsoft; it is not a security gain for the user.
- **Two of five are already on**: `ServiceEnabled` and `NotifyMalicious` are the effective defaults, so writing them changes nothing except the lock.
- **"Not configured" leaves protection on**: it removes the policy, and the shipped defaults (audit mode plus malicious-site warnings) resume.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 22H2 and newer (24H2 and 25H2 included), editions Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC per the Policy CSP. The component does not exist on Windows 10, and the tweak is hidden there.
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Not configured" deletes all five values, restoring the shipped behaviour and unlocking the Windows Security toggle. System Default restores the captured values. Screenshots already uploaded are not recalled.

#### Interactions
- [Enforce SmartScreen (apps and Edge)](#enforce-smartscreen-apps-and-edge): configures SmartScreen's app and Edge checks; Enhanced Phishing Protection is a separate SmartScreen component that tweak does not touch.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the feature covers work or school passwords only, that `ServiceEnabled` = 1 means audit mode (the warnings come from the three `Notify*` values), that writing `ServiceEnabled` locks the Windows Security toggle, and that most values already default to the hardened state on consumer machines.
- **Confidence**: Microsoft-documented. The shipped ADMX and ADML, the Enhanced Phishing Protection article's default-value table and the Policy CSP agree.
- **Reasoning**: Key, value names, types and polarity matched the shipped ADMX exactly, and the Windows 11 gate matches the Policy CSP (22H2, 10.0.22621, and later). No lockout or breakage path exists. The Policy CSP lists no Home edition; the tweak's gate is by product (Windows 11) only.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Warnings on, no screenshot upload" on a machine signed in with a work or school account, where the password-reuse and unsafe-app warnings are real protection. On a personal machine with a local or personal Microsoft account, skip it: it only locks the toggle.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\WebThreatDefense.admx` and `en-US\WebThreatDefense.adml` (26100): the five values, polarity, Windows 11 22H2 gate and the "enabled in audit mode" help text (tier A, shipped ADMX)
2. Enhanced Phishing Protection in Microsoft Defender SmartScreen, "Recommended settings for your organization" default-value table and the work-or-school scope, https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/enhanced-phishing-protection (tier A)
3. Policy CSP WebThreatDefense: `ServiceEnabled` Default Value 1, applicability and editions, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-webthreatdefense (tier A)

### ASR standard protection rules

`asr_standard_protection_rules` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Adds the two Defender attack surface reduction rules Microsoft recommends deploying straight to Block: vulnerable signed drivers and WMI persistence.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `asr_vulnerable_drivers` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules`, value `56a863a9-875e-4185-98a7-b882c64b5ce5` (REG_SZ) | "Block abuse of exploited vulnerable signed drivers" |
| `asr_wmi_persistence` | registry | same key, value `e6db77e5-3df2-4cf1-b95a-636979351e5b` (REG_SZ) | "Block persistence through WMI event subscription" |
| `asr_policy` | shared | shared setting `defender_asr_policy_enabled` (see [Shared settings](#shared-settings)) | `ExploitGuard_ASR_Rules` = 1 under `...\Windows Defender Exploit Guard\ASR`, refcounted with the other ASR tweaks |

| Option | `asr_vulnerable_drivers` | `asr_wmi_persistence` | `asr_policy` |
|---|---|---|---|
| Block | `"1"` | `"1"` | claim |
| Audit only | `"2"` | `"2"` | claim |
| Off | `absent` | `absent` | unclaimed |

System Default is shown when the two rule values match no option (for example one rule set to Warn, `"6"`, by another tool); selecting it restores the captured rule values. The shared parent value is not part of this tweak's snapshot: it is released through the shared-setting refcount. Stock Windows has no ASR policy values, so a stock machine reads as "Off".

#### How it works

Attack surface reduction rules are Microsoft Defender Antivirus behaviour rules, each identified by a GUID. The shipped `WindowsDefender.admx` defines policy `ExploitGuard_ASR_Rules` with a parent value `ExploitGuard_ASR_Rules` under `...\Exploit Guard\ASR` and a `<list>` element with `explicitValue="true"` for the `ASR\Rules` subkey. That structure means each value **name** is a rule GUID and each value's **data** is the rule state as a string: `"0"` off, `"1"` block, `"2"` audit, `"6"` warn. So REG_SZ is the correct type, and the parent value is what makes the policy read as Configured in `gpedit.msc`; without it a policy refresh can treat the rule values as orphaned.

The parent value is a shared setting, because four ASR tweaks in this category need it and must agree. The first tweak to claim it captures the original value once and writes 1; further claims are verified no-ops; releasing it while another ASR tweak still claims it leaves it alone ("held by" those tweaks); the last release restores the captured original and verifies it. Detection counts the claimed shared value as matching while any claim holds.

The two rules:

- `56a863a9-875e-4185-98a7-b882c64b5ce5`, **Block abuse of exploited vulnerable signed drivers**: stops applications writing known-vulnerable signed drivers to disk, a common bring-your-own-vulnerable-driver (BYOVD) step used to get kernel code execution. Microsoft notes it "doesn't prevent loading existing drivers already on the computer", so it cannot brick a running machine. Dependencies: none. It shows user notification pop-ups but raises no EDR alerts. It complements the vulnerable-driver blocklist rather than duplicating it: the blocklist stops loading, this rule stops the write.
- `e6db77e5-3df2-4cf1-b95a-636979351e5b`, **Block persistence through WMI event subscription**: stops malware creating WMI event subscriptions, a stealthy persistence mechanism that survives reboots. Dependencies: Microsoft Defender Antivirus, RPC. It has limited exclusion support, and Microsoft warns Configuration Manager clients rely heavily on WMI. It raises both user pop-ups and EDR alerts.

Both GUIDs were checked character by character against Microsoft's ASR rules reference. Both sit under its "Standard protection rules" heading, which holds exactly three rules: these two plus the LSASS credential-theft rule. Microsoft's guidance is that standard protection rules can be deployed directly to Block without an audit period.

Neither rule needs cloud-delivered protection. Both need Defender to be the active antivirus; with a third-party antivirus Defender is in passive mode and ASR rules are not enforced.

#### Benefits
- **Microsoft's own standard set**: two of the three rules Microsoft designates for direct Block deployment with no audit period.
- **Blocks BYOVD at the write**: the driver rule stops apps saving vulnerable signed drivers to disk.
- **Closes a stealthy persistence path**: WMI event subscriptions survive reboots and are easy to miss.
- **Audit option**: "Audit only" logs what would be blocked without blocking it.

#### Drawbacks
- **Needs Defender active**: both rules are inert when a third-party antivirus puts Defender in passive mode.
- **Limited exclusions on the WMI rule**: `e6db77e5` has limited exclusion support, and Configuration Manager clients lean heavily on WMI (irrelevant on a home machine, not on a managed one).
- **Does not unload what is already present**: the driver rule prevents saving vulnerable drivers, not loading ones already on disk.
- **Home is unproven**: Microsoft makes no statement either way about ASR enforcement on Windows 11 Home.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; `56a863a9` also on Windows 10 1709 and later and `e6db77e5` on Windows 10 1903 and later, so both work on Windows 10 IoT Enterprise LTSC 2021. Requires Microsoft Defender Antivirus as the active antivirus. Home is not confirmed.
- **Takes effect**: immediately; no reboot.
- **Reverting**: "Off" deletes both rule values and releases the shared parent claim; the parent value is restored to its original only when no other ASR tweak still claims it. System Default restores the captured rule values.

#### Interactions
- [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule): the third standard protection rule, in its own tweak.
- [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors) and [ASR extended rule set](#asr-extended-rule-set): other rules in the same `ASR\Rules` key. All four ASR tweaks claim the shared `defender_asr_policy_enabled` value; research merge candidate 4 proposes one ASR tweak.
- [Enable the Microsoft vulnerable-driver blocklist](#enable-the-microsoft-vulnerable-driver-blocklist): stops vulnerable drivers loading; this tweak's driver rule stops them being written.
- [Defender cloud protection and MAPS](#defender-cloud-protection-and-maps): not required by either rule.

#### Validation
- **Verdict**: VERIFIED. No correction to the mechanism; the research dropped an unsourced claim that the rules work on Windows 11 Home.
- **Confidence**: Microsoft-documented. GUIDs, dependencies, notification behaviour and applicability come from Microsoft's ASR rules reference; the REG_SZ typing and parent value come from the shipped ADMX.
- **Reasoning**: Both GUIDs matched character by character, the REG_SZ typing is proven by the ADMX `explicitValue="true"` list, and the dependency check confirmed neither rule depends on cloud protection. Open question: ASR enforcement on Windows 11 Home has no Microsoft statement either way (research UNKNOWN 16), so it is treated as unsupported rather than false.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Block" on any machine where Defender is the active antivirus; Microsoft's own guidance is that these rules go straight to Block, and breakage on a consumer machine is close to zero. On a managed machine with Configuration Manager, start with "Audit only".

#### Sources
1. Attack surface reduction (ASR) rules reference: per-rule GUID, Dependencies, notification support, applicability and the "Standard protection rules" section, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
2. Shipped `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policy `ExploitGuard_ASR_Rules`: parent value and the `explicitValue="true"` rule list (tier A, shipped ADMX)
3. ASR rules overview: deployment guidance and rule states, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview (cited in the tweak's evidence)

### Disable WinRM remoting

`disable_winrm_remoting` · Switch · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Shuts down WinRM, the inbound remote-execution channel behind PowerShell Remoting that attackers use after stealing credentials.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `winrm_service` | service | `WinRM` (Windows Remote Management) start type | none |
| `winrm_client_basic` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client`, value `AllowBasic` (REG_DWORD) | ADMX `AllowBasic_2` |
| `winrm_client_unencrypted` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client`, value `AllowUnencryptedTraffic` (REG_DWORD) | ADMX `AllowUnencrypted_2` |
| `winrm_client_digest` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Client`, value `AllowDigest` (REG_DWORD) | ADMX `DisallowDigest` (inverted polarity) |
| `winrm_service_basic` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Service`, value `AllowBasic` (REG_DWORD) | ADMX `AllowBasic_1` |
| `winrm_service_unencrypted` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Service`, value `AllowUnencryptedTraffic` (REG_DWORD) | ADMX `AllowUnencrypted_1` |
| `winrm_service_runas` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\WinRM\Service`, value `DisableRunAs` (REG_DWORD) | ADMX `DisableRunAs` |

| Option | `winrm_service` | `winrm_client_basic` | `winrm_client_unencrypted` | `winrm_client_digest` | `winrm_service_basic` | `winrm_service_unencrypted` | `winrm_service_runas` |
|---|---|---|---|---|---|---|---|
| Disabled | `disabled` | `0` | `0` | `0` | `0` | `0` | `1` |

This is a toggle: On is "Disabled", and Off is System Default. System Default is shown whenever the service start type or any of the six values differs from the option; turning the toggle off restores the captured start type exactly and the six values as they were (normally absent). All six policy values are absent on a stock machine. No tier A source pins the shipped `WinRM` start type on a Windows 11 client, which is why the tweak has no authored "stock" option and relies on the snapshot.

#### How it works

Windows Remote Management (WinRM) is Microsoft's WS-Management implementation. It carries PowerShell Remoting (`Enter-PSSession`, `Invoke-Command` into this machine), Windows Event Collector forwarding and many remote-management agents. Post-exploitation tooling reaches for it after credential theft because it offers remote command execution with legitimate credentials.

Setting the `WinRM` service to Disabled removes the inbound channel. On a stock client the practical effect is usually invisible: Microsoft states "By default, no WinRM listener is configured. Even if the WinRM service is running, WS-Management protocol messages that request data can't be received or sent." The service matters on machines where `winrm quickconfig` (or `Enable-PSRemoting`) has run: those sit at delayed automatic start with a listener configured, and this tweak stops the service starting again, so the listener goes away at the next restart.

The six policy values, all confirmed in the shipped 26100 `WindowsRemoteManagement.admx` (class Machine), harden both roles so that re-enabling the service later does not reopen weak authentication:

- Client and Service `AllowBasic` = 0: no Basic authentication (password sent effectively in clear).
- Client and Service `AllowUnencryptedTraffic` = 0: no unencrypted WS-Man traffic.
- Client `AllowDigest` = 0: no Digest authentication. This one is inverted: the ADMX policy is named `DisallowDigest`, and setting that policy to Enabled writes `AllowDigest` = 0.
- Service `DisableRunAs` = 1: the WinRM service refuses stored RunAs credentials for plug-ins.

These six match the DISA STIG WinRM control set.

The tweak cannot lock anyone out. WinRM governs inbound WS-Man only; local sign-in and the console are untouched, RDP is a separate service, outbound `Invoke-Command` from this PC to other machines still works (the client side only loses Basic, Digest and unencrypted transport), and PowerShell remoting over SSH is a separate channel.

#### Benefits
- **Removes the standard remote-execution channel**: WinRM is what post-exploitation tooling uses after credential theft.
- **Hardens the client too**: even if the service is re-enabled later, Basic and Digest authentication and cleartext transport stay off.
- **Six STIG rules in one**: matches the DISA WinRM control set.
- **Safe revert**: the service start type is restored exactly from the snapshot, including delayed automatic start.

#### Drawbacks
- **Inbound PowerShell Remoting stops**: `Enter-PSSession` and `Invoke-Command` into this machine fail.
- **Event forwarding breaks**: the Windows Event Collector service (`wecsvc`) depends on WinRM.
- **Remote management tools break**: Windows Admin Center and several remote-management agents stop working.
- **Usually no visible change**: a stock Windows 11 client has no WinRM listener configured.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions.
- **Takes effect**: the start type and the policy values are written immediately, and the policy values are read on the next WinRM operation. The app does not stop a running `WinRM` service, so a configured listener keeps accepting connections until the service stops or the machine restarts; the tweak is flagged as needing a reboot for that reason.
- **Reverting**: turning the toggle off restores the previous service start type from the snapshot and restores the six policy values (normally by deleting them). It never hardcodes Manual, because a machine where `winrm quickconfig` has run sits at delayed automatic start.

#### Interactions
- [Disable the Secondary Logon service](#disable-the-secondary-logon-service): same design (service start type restored from the snapshot, never hardcoded).
- [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp) and [Disable the Remote Registry service](#disable-the-remote-registry-service): other remote-access surfaces, independent of WinRM.
- [Event log retention size](#event-log-retention-size): unrelated, but event forwarding through `wecsvc` stops once this is applied.

#### Validation
- **Verdict**: VERIFIED. No correction to the six values; the research established that the stock start type must be restored from the snapshot rather than hardcoded as Manual.
- **Confidence**: Microsoft-documented. The six values and polarities come from the shipped ADMX; the listener behaviour and client start behaviour come from Microsoft's WinRM installation and configuration page.
- **Reasoning**: All six values, including the inverted `DisallowDigest`, were parsed from the shipped XML, and the lockout analysis found no path to lock a user out. Open question: the exact shipped `WinRM` start type on a clean Windows 11 24H2 client (Manual versus trigger-start) is not pinned by any tier A source (research UNKNOWN 14); the snapshot restore makes that irrelevant to correctness.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a standalone or home machine that never accepts PowerShell Remoting. Leave it alone if you manage this PC remotely, use Windows Admin Center or event forwarding, or run a remote-management agent that depends on WinRM.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\WindowsRemoteManagement.admx` (26100), policies `AllowBasic_1`, `AllowBasic_2`, `AllowUnencrypted_1`, `AllowUnencrypted_2`, `DisallowDigest`, `DisableRunAs`: keys, value names and polarity (tier A, shipped ADMX)
2. Installation and configuration for Windows Remote Management, "Configuration of WinRM and IPMI" and "Quick default configuration": no listener by default, client start behaviour, and the delayed automatic start set by `winrm quickconfig`, https://learn.microsoft.com/en-us/windows/win32/winrm/installation-and-configuration-for-windows-remote-management (tier A)
3. DISA STIG for Windows 11 V2R2, the six WinRM rules, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)

### PowerShell module logging and transcription

`powershell_module_transcript_logging` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Records what PowerShell actually executed in the event log and writes a durable text transcript of every PowerShell session.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `module_logging` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ModuleLogging`, value `EnableModuleLogging` (REG_DWORD) | none |
| `module_names_wildcard` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ModuleLogging\ModuleNames`, value named `*` (REG_SZ) | the value name is what PowerShell reads |
| `transcripting` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription`, value `EnableTranscripting` (REG_DWORD) | none |
| `transcript_invocation_header` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription`, value `EnableInvocationHeader` (REG_DWORD) | none |
| `transcript_output_dir` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription`, value `OutputDirectory` (REG_SZ) | none |

| Option | `module_logging` | `module_names_wildcard` | `transcripting` | `transcript_invocation_header` | `transcript_output_dir` |
|---|---|---|---|---|---|
| Module logging and transcription | `1` | `"*"` | `1` | `1` | `C:\ProgramData\PowerShellTranscripts` |
| Module logging only | `1` | `"*"` | `absent` | `absent` | `absent` |
| Off | `absent` | `absent` | `absent` | `absent` | `absent` |

System Default is shown when the five values match no option (for example a module list configured by an administrator); selecting it restores the captured values. Stock Windows has none of these values, so a stock machine reads as "Off".

#### How it works

Both settings are Windows PowerShell Group Policy settings, defined in the shipped 26100 `PowerShellExecutionPolicy.admx` as policies `EnableModuleLogging` and `EnableTranscripting`, class Both. The policy engine inside PowerShell reads them when a session starts.

**Module logging** records pipeline execution details (the commands run and their parameters) for the listed modules in the PowerShell operational event log. The module list lives in the `ModuleNames` subkey. PowerShell's own policy reader (`TrySetPolicySettingsFromRegistryKey` in `Utils.cs`) reads the **value names** of that subkey and ignores the data, so the wildcard must be a value **named** `*`. A numbered value such as `1` = `*` (the shape a generic ADMX list writer would produce) would register a module literally called "1" and match nothing. The REG_SZ type and `"*"` data are cosmetic to PowerShell but match what CIS, the DISA STIG and other tools write.

**Transcription** writes a text file per session recording everything typed and every output, like a console recorder. `EnableInvocationHeader` = 1 adds a timestamp header for each command. `OutputDirectory` sets where transcripts go; the tweak pins `C:\ProgramData\PowerShellTranscripts`. Without it, transcripts land as `PowerShell_transcript.*.txt` files in each user's Documents folder, where the user (or malware running as the user) can read or delete them.

Script-block logging (a separate tweak) records the content of script blocks as they are compiled; module logging records what pipelines executed; transcription records what someone actually did interactively. They are complementary.

The class is Both, so an HKCU copy would also apply for one user; writing HKLM is the machine-wide choice and HKCU is not needed.

PowerShell 7 (`pwsh.exe`) reads its own `HKLM\SOFTWARE\Policies\Microsoft\PowerShellCore\...` policy keys for script-block logging, as documented in this category's research. This tweak writes only the Windows PowerShell keys; whether PowerShell 7 honours them for module logging and transcription was not established by the research.

#### Benefits
- **Closes the console blind spot**: script-block logging records what a script contained; transcription records what someone actually did interactively.
- **Two separate baseline requirements**: the DISA STIG requires transcription and CIS lists module logging as Level 1.
- **Durable**: transcripts survive on disk even if the event log rolls over.
- **Central location**: transcripts go to one machine-wide folder, not each user's Documents.

#### Drawbacks
- **Transcript files accumulate**: one file per session, which adds up on a machine that runs a lot of scripted work. Nothing rotates or deletes them.
- **Very verbose**: module logging with `*` fills the PowerShell operational log faster.
- **Secrets land in files**: anything typed or piped in plaintext, including passwords and tokens, is written to the transcript, so the output folder must be protected. The tweak does not create the folder or set its permissions: it inherits `C:\ProgramData`'s defaults, which give every local user read access to the files (and full control to whichever account creates the folder), so on a PC other people use, restrict the folder to administrators yourself.
- **PowerShell 7 coverage not established**: only the Windows PowerShell policy keys are written.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions. Covers Windows PowerShell 5.1.
- **Takes effect**: immediately for new PowerShell sessions; no reboot.
- **Reverting**: "Off" deletes all five values; "Module logging only" deletes the three transcription values. System Default restores the captured values. Transcript files already written stay on disk, and the now-empty `ModuleNames`, `ModuleLogging` and `Transcription` keys remain. They are harmless: PowerShell treats a missing `EnableModuleLogging` or `EnableTranscripting` value as off. A `registry_key` effect cannot remove `ModuleNames`, because the build rejects deleting a key that holds one of the tweak's own values.

#### Interactions
- [Enable PowerShell script-block logging](#enable-powershell-script-block-logging): complementary; that tweak also writes the `PowerShellCore` key for PowerShell 7.
- [Remove PowerShell 2.0 engine](#remove-powershell-20-engine): the 2.0 engine does not support module logging or transcription policy, so removing it closes the downgrade bypass.
- [Event log retention size](#event-log-retention-size): sizes the Application, System and Security logs only; it does not enlarge the PowerShell operational log that module logging fills.

#### Validation
- **Verdict**: VERIFIED. No correction needed.
- **Confidence**: Microsoft-documented. Keys, value names and types come from the shipped ADMX; the `ModuleNames` layout is settled from PowerShell's own first-party source code.
- **Reasoning**: The one plausible failure mode, the `ModuleNames` value layout (the ADMX list carries no `explicitValue`), was settled by reading PowerShell's policy reader: value names are read, data is ignored. Types follow the ADMX element types (`text` is REG_SZ, `boolean` is REG_DWORD). No lockout path exists. Not covered by the research: whether PowerShell 7 honours these Windows PowerShell keys. The output folder's permissions were read afterwards: `C:\ProgramData` grants `BUILTIN\Users` read and execute (inherited by files) plus create-folder and write-data rights, and `CREATOR OWNER` full control, so without a manual ACL change other local users can read transcripts.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Choose "Module logging and transcription" if you investigate incidents or want a full audit trail, and, on a PC other people use, restrict `C:\ProgramData\PowerShellTranscripts` to administrators, because by default every local user can read it. If transcript files on disk bother you (they can contain secrets), choose "Module logging only"; it still records pipeline activity in the event log.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\PowerShellExecutionPolicy.admx` (26100), policies `EnableModuleLogging` and `EnableTranscripting`: keys, value names, element types and class Both (tier A, shipped ADMX)
2. PowerShell source, `src/System.Management.Automation/engine/Utils.cs`, `TrySetPolicySettingsFromRegistryKey`: `ModuleNames` value names are read and data ignored, https://raw.githubusercontent.com/PowerShell/PowerShell/master/src/System.Management.Automation/engine/Utils.cs (tier A, first-party implementation)
3. about_Logging_Windows: module logging, transcription and their policy settings, https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows (tier A)
4. Sophia Script for Windows 11: writes the `ModuleNames` entry as a value named `*` (tier C, corroboration only; no URL given in the research)

### Restrict remote SAM calls to administrators

`restrict_remote_sam` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Pins that only administrators can enumerate this PC's local users and groups over the network, blocking BloodHound-style reconnaissance from ordinary accounts.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `restrict_remote_sam` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`, value `RestrictRemoteSam` (REG_SZ) | an SDDL string; a DWORD here is ignored |

| Option | `restrict_remote_sam` |
|---|---|
| Administrators only | `O:BAG:BAD:(A;;RC;;;BA)` |
| Not configured | `absent` |

System Default is shown when the value holds any other string (a custom descriptor); selecting it restores the captured value. Stock Windows has no value, so a stock machine reads as "Not configured". With the value absent, Windows 10 1607 and later apply a built-in default that already restricts remote SAM calls to administrators, so on every supported build both options currently grant the same access.

#### How it works

The Security Account Manager (SAM) remote protocol (SAMRPC) lets a network client enumerate local accounts and group memberships. Reconnaissance tools such as BloodHound use it, from any ordinary domain account, to map which users are local administrators on which machines. `RestrictRemoteSam` is the registry value behind the security policy *Network access: Restrict clients allowed to make remote calls to SAM*. Microsoft documents it as "Registry type: REG_SZ" holding "a string that will contain the SDDL of the security descriptor to be deployed". The LSA checks each incoming remote SAM call against that descriptor.

The tweak writes the standard CIS and DISA STIG descriptor `O:BAG:BAD:(A;;RC;;;BA)`: owner Built-in Administrators, group Built-in Administrators, and a DACL granting Read Control only to Built-in Administrators. Every other caller's remote SAM call is denied.

The type matters. A REG_DWORD written here is silently ignored. The string is also not validated when written: a malformed or over-restrictive SDDL would silently deny SAMRPC to principals that need it, which is why the tweak ships the exact baseline string as a fixed option value and never lets the user type one.

This is a different control from anonymous-enumeration hardening. `RestrictAnonymousSAM`, `RestrictAnonymous` and `EveryoneIncludesAnonymous` (same `Lsa` key, a separate tweak) govern anonymous, unauthenticated sessions; `RestrictRemoteSam` governs which authenticated principals may call SAM remotely. The two are additive.

#### Benefits
- **Defeats authenticated reconnaissance**: covers a different attack from the anonymous-enumeration controls, which only stop unauthenticated callers.
- **Standard baseline value**: the SDDL is the exact CIS and DISA STIG string.
- **Pins a good state**: locks the descriptor so a later misconfiguration cannot loosen it.
- **No lockout risk**: local sign-in does not use remote SAM calls.

#### Drawbacks
- **Inventory agents lose visibility**: non-admin asset-management agents that enumerate local groups remotely stop working, which matters on domain-joined machines.
- **No visible change on supported builds**: on Windows 10 1607 and later the built-in default already restricts remote SAM calls to administrators.
- **Type-sensitive and unvalidated**: it must be REG_SZ, and a malformed SDDL would silently deny access (the fixed option value avoids this).

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 1607 and later, including Windows 10 IoT Enterprise LTSC 2021. All editions.
- **Takes effect**: immediately for new remote SAM calls; no reboot.
- **Reverting**: "Not configured" deletes the value, restoring Windows' built-in default descriptor (administrators only on 1607 and later). System Default restores the captured value.

#### Interactions
- [Restrict anonymous enumeration](#restrict-anonymous-enumeration): same `Lsa` key, different values, different attack (anonymous rather than authenticated callers). Additive, not a duplicate.
- [Filter the remote local-admin token](#filter-the-remote-local-admin-token): a related remote-admin hardening in a different key.

#### Validation
- **Verdict**: VERIFIED. No correction needed.
- **Confidence**: Microsoft-documented. Microsoft's Security Policy Settings reference states the key, the REG_SZ type, the SDDL semantics and the 1607 default directly.
- **Reasoning**: The REG_SZ type, the SDDL string and the "not a duplicate" analysis all held. The gap proposal's claim that the value is absent on 26100 was based on the modified research machine and was ruled inadmissible, but Microsoft's page carries the default independently, so the conclusion stands. The research added the caution never to let a user type an arbitrary SDDL.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Administrators only". It costs nothing on a consumer machine, pins a state Windows already uses, and guards against a later loosening. Skip it only if a non-admin inventory agent depends on enumerating local groups remotely.

#### Sources
1. Network access: Restrict clients allowed to make remote calls to SAM: registry location, REG_SZ type, SDDL value semantics and the 1607 default, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-access-restrict-clients-allowed-to-make-remote-sam-calls (tier A)
2. DISA STIG for Windows 11 V2R2 and CIS Windows 11 v4.0.0 Level 1: the baseline SDDL string, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
3. `src-tauri/tweaks/security.yaml`, `restrict_anonymous_enum` effects: the dedupe evidence that the anonymous controls are different values (internal)

### Block AlwaysInstallElevated

`block_always_install_elevated` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Pins off the Windows Installer setting that would let any user run an arbitrary MSI package with SYSTEM privileges.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `always_install_elevated_machine` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Installer`, value `AlwaysInstallElevated` (REG_DWORD) | none |
| `always_install_elevated_user` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Installer`, value `AlwaysInstallElevated` (REG_DWORD) | the hive of the account the tweak runs as |

| Option | `always_install_elevated_machine` | `always_install_elevated_user` |
|---|---|---|
| Blocked | `0` | `0` |
| Not configured | `absent` | `absent` |

System Default is shown when the two values match neither option (for example one of them set to 1 by a deployment script); selecting it restores the captured values. Stock Windows has neither value, so a stock machine reads as "Not configured". That means no policy: with both values absent the escalation is not enabled, so "Not configured" is the safe shipped state, not an open one.

#### How it works

`AlwaysInstallElevated` is the Windows Installer policy *Always install with elevated privileges*, defined in the shipped 26100 `MSI.admx` as class Both (a machine and a user copy), enabled value 1, disabled value 0. When it is 1 in **both** `HKLM` and `HKCU`, Windows Installer runs any `.msi` package the user starts with SYSTEM privileges. An attacker with an ordinary account can then build a package that adds an administrator or runs any command as SYSTEM, which is why local privilege-escalation tools check this setting first. Windows gives no indication when it is on.

The escalation needs both hives set to 1, so writing `0` to HKLM alone already defeats it, whatever the user hive says. The tweak also writes the HKCU copy as defence in depth. That per-user write pins only the hive of the account the tweak runs as; other users' hives on the same machine are untouched, which is harmless because the HKLM value is sufficient.

Writing `0` cannot break a working installer: a legitimate installation never depends on this escalation.

#### Benefits
- **Closes a well-known escalation**: one of the first things local privilege-escalation tooling checks for.
- **Pins the state against deployment scripts**: some deployment scripts turn this on and never turn it back off; the policy value 0 overrides that.
- **Cannot break a working install**: no legitimate installer depends on the escalation.

#### Drawbacks
- **No visible change on a clean machine**: both values are absent by default, so this pins an existing safe state.
- **HKCU covers one user**: the per-user write pins only the account the tweak runs as (HKLM = 0 still protects everyone).
- **Deployment scripts may fight it**: some poorly written enterprise deployment scripts set this intentionally and would break.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer; also Windows 10 22H2 and Windows 10 IoT Enterprise LTSC 2021. All editions.
- **Takes effect**: immediately for the next installer run; no reboot.
- **Reverting**: "Not configured" deletes both values, which is the shipped state. System Default restores the captured values, including a 1 if one was present before the first apply.

#### Interactions
None known. No other tweak in the corpus writes the Windows Installer policy key.

#### Validation
- **Verdict**: VERIFIED. No correction to the mechanism; the research restated the reasoning: the escalation needs both hives at 1, so HKLM = 0 is sufficient and the HKCU write is defence in depth, not a requirement.
- **Confidence**: Microsoft-documented. The policy, its class and values come from the shipped `MSI.admx`; the baseline requirement comes from the DISA STIG.
- **Reasoning**: Key, value name, type and class were parsed from the shipped ADMX. The research corrected the gap proposal's claim that both hives must be written for the control to work (it is the opposite: both must be 1 for the escalation to work) and added the one-user scope of the HKCU write. The revert-by-delete shape is corroborated by a community tool.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Blocked" on every machine. It costs nothing, cannot break a working installer, and defends against a setting that deployment scripts sometimes turn on and never turn off.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\MSI.admx` (26100), policy `AlwaysInstallElevated`: key, value name, class Both and values (tier A, shipped ADMX)
2. Policy configuration service provider (the tweak's evidence link for the MSI policy set): https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-configuration-service-provider (tier A, general Policy CSP index rather than a page specific to this setting)
3. DISA STIG for Windows 11 V2R2, "The Windows Installer feature 'Always install with elevated privileges' must be disabled", and CIS Windows 11 v4.0.0 Level 1, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
4. privacy.sexy `windows.yaml`: `AlwaysInstallElevated` = 0 with delete on revert (tier C, corroboration of the revert shape; no URL given in the research)

### Kernel DMA protection policy

`kernel_dma_protection` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: build 17763 and newer · Reversible: yes

**Stops external Thunderbolt and PCIe devices from reading memory before you sign in or while the machine is locked, closing drive-by DMA attacks on an unattended laptop.**

#### What it changes

| Effect | Kind | Target | Notes |
|---|---|---|---|
| `dma_device_enumeration` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Kernel DMA Protection`, value `DeviceEnumerationPolicy` (REG_DWORD) | 0 Block all, 1 Only while logged in, 2 Allow all |
| `dma_disable_under_lock` | registry | `HKLM\SOFTWARE\Policies\Microsoft\FVE`, value `DisableExternalDMAUnderLock` (REG_DWORD) | BitLocker policy key |

| Option | `dma_device_enumeration` | `dma_disable_under_lock` |
|---|---|---|
| Block all | `0` | `1` |
| Allow only after sign-in | `1` | `1` |
| Not configured | `absent` | `absent` |

System Default is shown when the two values match no option (for example `DeviceEnumerationPolicy` = 2 set by another tool); selecting it restores the captured values. Stock Windows has neither value, so a stock machine reads as "Not configured": with the values absent, Windows applies its own default, which Microsoft labels "Only while logged in (default)" (the behaviour of value 1). The tweak never writes 2, the real "Allow all" enum value, because that would be less protective than the Windows default.

#### How it works

Thunderbolt and other external PCIe ports give devices direct memory access (DMA): a malicious device plugged into a locked laptop can read or write system memory without the CPU's involvement, bypassing the lock screen and disk encryption. Kernel DMA Protection uses the platform's IOMMU (Intel VT-d or AMD IOMMU DMA remapping) to confine each external device to memory the OS has explicitly mapped for it. Devices whose drivers support DMA remapping are always allowed; the policy decides what happens to devices whose drivers do not.

`DeviceEnumerationPolicy` (ADMX `DmaGuard.admx`, policy `DmaGuardEnumerationPolicy`, class Machine) has three values, confirmed by a comment inside the shipped ADMX: `0` Block all (DMA-remapping-incompatible external devices never start), `1` Only while logged in (they are blocked until an authorized user signs in or unlocks, then allowed), `2` Allow all. The shipped ADML marks "Only while logged in" as the default, and Microsoft Learn agrees: "By default, peripherals with DMA Remapping incompatible drivers are blocked from starting and performing DMA until an authorized user signs into the system or unlocks the screen."

`DisableExternalDMAUnderLock` = 1 (ADMX `VolumeEncryption.admx`, class Machine, supported from Windows 10 1703) is a BitLocker policy that blocks DMA on Thunderbolt hot-pluggable PCI downstream ports every time the machine is locked, until the user signs in again. Devices already enumerated while unlocked keep working until unplugged, rebooted or hibernated. The ADML states it "is only enforced when BitLocker or device encryption is enabled", which on consumer 24H2 machines is often the case because automatic device encryption is common.

Important limits: Kernel DMA Protection itself cannot be turned on by policy. Microsoft: "Kernel DMA Protection is a platform feature that can't be controlled via policy or by end user. It has to be supported by the system at the time of manufacturing." On a machine without firmware DMA remapping, `DeviceEnumerationPolicy` does nothing. The policy also does not apply to 1394 (FireWire), PCMCIA or ExpressCard devices. Microsoft's Policy CSP limits it to Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC from Windows 10 1809; it is not available on Home. The Policy CSP also states "This policy requires a system reboot to take effect".

The tweak cannot lock you out: the internal keyboard, display and storage are not external DMA devices, and USB HID devices on native USB ports are unaffected.

#### Benefits
- **Stops drive-by DMA**: a malicious Thunderbolt or PCIe device plugged into a locked laptop cannot read memory.
- **Covers the locked state too**: `DisableExternalDMAUnderLock` blocks new Thunderbolt hot-plugs while locked, not only before first sign-in.
- **No effect on internal hardware**: the internal keyboard, display and storage are untouched.

#### Drawbacks
- **External GPUs and docks can stop working**: eGPU enclosures, some Thunderbolt docks and older PCIe capture cards refuse to start under "Block all".
- **Lid-closed dock scenario**: if a dock supplying your only keyboard, mouse and display is DMA-remapping incompatible and gets blocked, you lose input until you open the lid. Recoverable, not a lockout.
- **Inert on many machines**: it needs firmware DMA remapping present at manufacture, is not available on Home, and does not cover 1394, PCMCIA or ExpressCard. The tweak does not check whether Kernel DMA Protection is active, so it can apply successfully on a machine where it does nothing.
- **Second value needs encryption**: `DisableExternalDMAUnderLock` is only enforced when BitLocker or device encryption is on.
- **The middle option changes little**: "Allow only after sign-in" pins the Windows default for `DeviceEnumerationPolicy`; its only real change is `DisableExternalDMAUnderLock`.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer and Windows 10 1809 (build 17763) and later, including Windows 10 IoT Enterprise LTSC 2021; Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. Not Home (the tweak is gated by build only, so it is offered on Home but does nothing there). Requires a platform shipped with Kernel DMA Protection.
- **Takes effect**: after a reboot, as the Policy CSP requires.
- **Reverting**: "Not configured" deletes both values so the Windows default ("Only while logged in") applies again; it never writes 2. System Default restores the captured values. A reboot completes either.

#### Interactions
- [Prevent automatic device encryption](#prevent-automatic-device-encryption): `DisableExternalDMAUnderLock` is only enforced with BitLocker or device encryption on, so preventing automatic device encryption can leave that half of this tweak inert.
- [Enable Credential Guard](#enable-credential-guard): both depend on platform security hardware (IOMMU, VBS); no shared values.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the Windows default for `DeviceEnumerationPolicy` is value 1 ("Only while logged in"), not 2, so a revert must never write 2; that a reboot is mandatory; that the policy is Pro and above from Windows 10 1809; and that `DisableExternalDMAUnderLock` covers Thunderbolt hot-plug ports only and requires BitLocker or device encryption.
- **Confidence**: Microsoft-documented. The enum and default come from the shipped ADMX and ADML; applicability, editions and the reboot come from the Policy CSP; the default behaviour is confirmed on Microsoft Learn.
- **Reasoning**: The value names, keys and enum held; the gap proposal's default was wrong on both halves and was corrected from the ADML. Two research requirements are not in the shipped tweak: a probe that reports "not applicable" on machines without Kernel DMA Protection (the tweak verifies the registry values only), and an edition gate excluding Home (the tweak gates on build 17763 only). The lockout analysis found no lockout path.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Block all" on a laptop you carry, where the drive-by DMA threat is real, after checking that your docks and peripherals still work. On a desktop with an eGPU or a capture card, use "Allow only after sign-in" or leave it alone. Check `msinfo32` ("Kernel DMA Protection: On") first; if it is off, this tweak does nothing on your machine.

#### Sources
1. Shipped `C:\Windows\PolicyDefinitions\DmaGuard.admx` and `en-US\DmaGuard.adml` (26100), policy `DmaGuardEnumerationPolicy`, including the in-file comment mapping options to registry values and the "(default)" label (tier A, shipped ADMX)
2. Shipped `C:\Windows\PolicyDefinitions\VolumeEncryption.admx` and `en-US\VolumeEncryption.adml` (26100), policy `DisableExternalDMAUnderLock_Name`: scope and the BitLocker or device encryption condition (tier A, shipped ADMX)
3. Policy CSP DmaGuard: applicability from 1809, editions, reboot requirement and platform limits, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-dmaguard (tier A)
4. Kernel DMA Protection, "How Windows protects against DMA drive-by attacks": the default behaviour and the platform requirement, https://learn.microsoft.com/en-us/windows/security/hardware-security/kernel-dma-protection-for-thunderbolt (tier A)

### ASR extended rule set

`asr_extended_rules` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Six more Microsoft Defender attack surface reduction rules, in Block or Audit mode, covering script-launched downloads, Win32 calls from Office macros, untrusted USB executables, ransomware, PsExec and WMI process creation, and Office communication apps spawning child processes.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `asr_script_downloads` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR\Rules` → `d3e037e1-3eb8-44c8-a917-57927947596d` (REG_SZ) |
| `asr_macro_win32_api` | registry | same key → `92e97fa1-2edf-4476-bdd6-9dd0b4dddc7b` (REG_SZ) |
| `asr_usb_untrusted` | registry | same key → `b2b3f03d-6a65-4f7b-a9c7-1c7ef74a9ba4` (REG_SZ) |
| `asr_ransomware` | registry | same key → `c1db55ab-c21a-4637-bb3f-a12568109d35` (REG_SZ) |
| `asr_psexec_wmi` | registry | same key → `d1e49aac-8f56-4280-b9ba-993a6d77406c` (REG_SZ) |
| `asr_office_comm_child` | registry | same key → `26190899-1602-49e8-8b27-eb1d0a1ce869` (REG_SZ) |
| `asr_policy` | shared | shared setting `defender_asr_policy_enabled` (see [Shared settings](#shared-settings)): `HKLM\SOFTWARE\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR` → `ExploitGuard_ASR_Rules` (REG_DWORD) = 1 |

| Option | `asr_script_downloads` | `asr_macro_win32_api` | `asr_usb_untrusted` | `asr_ransomware` | `asr_psexec_wmi` | `asr_office_comm_child` | `asr_policy` |
|---|---|---|---|---|---|---|---|
| Block | `"1"` | `"1"` | `"1"` | `"1"` | `"1"` | `"1"` | claim |
| Audit only | `"2"` | `"2"` | `"2"` | `"2"` | `"2"` | `"2"` | claim |
| Off | absent | absent | absent | absent | absent | absent | unclaimed |

System Default is shown when the six rule values match none of the three options (for example a mix of Block and Audit, or a mode written by other management tooling); selecting it restores the snapshot captured before the first apply. Stock Windows has none of the six values and no `ExploitGuard_ASR_Rules` value, so a clean machine matches "Off". The shared parent value is not part of this tweak's snapshot: it is refcounted across the four ASR tweaks (see [Shared settings](#shared-settings)).

#### How it works

Microsoft Defender Antivirus reads attack surface reduction (ASR) rules from the policy key `...\Windows Defender Exploit Guard\ASR\Rules`. Each value name is a rule GUID and each value's data is a string giving the rule's mode: `"1"` is Block, `"2"` is Audit (the rule logs what it would have blocked and lets it run), and a missing value means the rule is not configured. Defender silently ignores a GUID it does not recognise, so a mistyped GUID reports success and does nothing; all six GUIDs here were checked character for character against Microsoft's ASR rules reference.

The six rules, with Microsoft's names and the dependencies Microsoft documents:

| GUID | Microsoft rule name | Dependencies per Microsoft |
|---|---|---|
| `d3e037e1-3eb8-44c8-a917-57927947596d` | Block JavaScript or VBScript from launching downloaded executable content | Defender Antivirus, AMSI |
| `92e97fa1-2edf-4476-bdd6-9dd0b4dddc7b` | Block Win32 API calls from Office macros | Defender Antivirus, AMSI |
| `b2b3f03d-6a65-4f7b-a9c7-1c7ef74a9ba4` | Block untrusted and unsigned processes that run from USB | Defender Antivirus |
| `c1db55ab-c21a-4637-bb3f-a12568109d35` | Use advanced protection against ransomware | Defender Antivirus, cloud-delivered protection |
| `d1e49aac-8f56-4280-b9ba-993a6d77406c` | Block process creations originating from PSExec and WMI commands | Defender Antivirus |
| `26190899-1602-49e8-8b27-eb1d0a1ce869` | Block Office communication application from creating child processes | Defender Antivirus |

Three Microsoft-documented constraints narrow what the rules actually do on a given machine. The ransomware rule (`c1db55ab`) works only when cloud-delivered protection is on; without it the rule is configured and non-functional, and because it errs on the side of caution, brand-new or rarely seen software can trip it. The script-download rule (`d3e037e1`) always blocks in Block mode, but Microsoft notes it raises EDR alerts only when the device's cloud protection level is High Plus or Zero Tolerance. The Office communication rule (`26190899`) "is enforced only if Office is installed in `%ProgramFiles%` or `%ProgramFiles(x86)%`", so Microsoft Store and per-user Office installs fall outside it, which makes it inert on a large share of consumer machines.

The shipped `WindowsDefender.admx` (26100) defines the policy `ExploitGuard_ASR_Rules` at `Software\Policies\Microsoft\Windows Defender\Windows Defender Exploit Guard\ASR` with `valueName="ExploitGuard_ASR_Rules"`, and that policy's list element is the `ASR\Rules` subkey. Without the parent value, `gpedit.msc` shows the policy as Not Configured and a policy refresh can strip the rule values as orphans. The tweak therefore claims the corpus-level shared setting `defender_asr_policy_enabled`, which writes that parent value once for all four ASR tweaks.

Rules are written under the Policies hive, which is the Group Policy surface Defender honours; the rules act only while Microsoft Defender Antivirus is the active antivirus. None of the six GUIDs is written by any other tweak in this corpus; the other ASR tweaks own different GUIDs.

#### Benefits
- Six distinct initial-access and lateral-movement behaviours covered by one setting, all through one registry key.
- An Audit mode for every rule, so you can watch what would be blocked before enforcing; moving from Audit to Block is one option change.
- On a consumer machine these behaviours (scripts launching downloaded executables, macros calling Win32 APIs, unsigned executables running from USB, PsExec and WMI process creation) are almost always malicious.
- No overlap with the other ASR tweaks in this category.

#### Drawbacks
- The PsExec and WMI rule (`d1e49aac`) blocks processes spawned by PsExec and by WMI `Win32_Process.Create`, which legitimate management and automation tooling uses. This is the risky rule of the six.
- The ransomware rule (`c1db55ab`) does nothing unless Defender cloud-delivered protection is on, and it can flag brand-new independent software.
- The Office communication rule (`26190899`) is often inert because it is enforced only for Office installed under Program Files.
- EDR alerting for `d3e037e1` is gated on cloud protection level High Plus or Zero Tolerance, even though blocking works.
- All six rules are set to the same mode; the tweak offers no per-rule choice, so you cannot, for example, enforce five and leave the PsExec rule in Audit.
- Requires Microsoft Defender Antivirus as the active antivirus; with a third-party antivirus the rules do nothing.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (Windows 11 24H2 build 26100 and newer, Windows 10 IoT Enterprise LTSC 2021 build 19044); Windows 10 is covered by the ASR reference too. Requires Microsoft Defender Antivirus as the active antivirus, and AMSI for `d3e037e1` and `92e97fa1`. ASR enforcement on Windows 11 Home is not confirmed or denied by any tier A source the research found, so treat Home as unsupported.
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Off" deletes all six rule values and releases this tweak's claim on `ExploitGuard_ASR_Rules`; the parent value is restored to its captured original only when no other ASR tweak still claims it (otherwise it is left in place, "held by" the other tweaks). Selecting System Default restores the six values captured in the snapshot, including `absent` for values that did not exist.

#### Interactions
- Shares `ExploitGuard_ASR_Rules` with [Block LSASS credential theft (ASR rule)](#block-lsass-credential-theft-asr-rule), [ASR rules: block Office/script malware vectors](#asr-rules-block-officescript-malware-vectors) and [ASR standard protection rules](#asr-standard-protection-rules) through the shared setting `defender_asr_policy_enabled`. The research lists these four as a merge candidate (thirteen GUIDs across the four tweaks, one key, one value type, one set of prerequisites; Microsoft's deployment guidance is per mode, not per rule).
- [Defender cloud protection and MAPS](#defender-cloud-protection-and-maps) turns on the cloud-delivered protection that the ransomware rule needs. That tweak sets the cloud block level to High, which is below the High Plus or Zero Tolerance level that `d3e037e1` needs for EDR alerting.
- Any scripted automation that uses PsExec or WMI process creation (including some remote-management tools) is blocked in Block mode.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research added three Microsoft-documented dependency notes (cloud protection for `c1db55ab`, the High Plus alerting gate for `d3e037e1`, the Program Files scope of `26190899`) and the ADMX parent value `ExploitGuard_ASR_Rules`, which applies to all four ASR tweaks.
- **Confidence**: Microsoft-documented. Every GUID, rule name and dependency comes from Microsoft's per-rule ASR reference; the parent value comes from the shipped `WindowsDefender.admx`.
- **Reasoning**: The adversarial pass attacked existence (all six GUIDs resolve verbatim to the named rules), key and type (REG_SZ under `ASR\Rules`, matching the corpus convention), and duplication (none of the six GUIDs appears elsewhere in the corpus). All three survived. Open questions: ASR behaviour on Windows 11 Home is unconfirmed (research UNKNOWNS item 16). The research's note on the cloud protection tweak says no ASR rule in the corpus is cloud-gated; that note predates this rule set, and `c1db55ab` is cloud-gated per Microsoft.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying on a consumer machine, where these six behaviours are almost always malicious. Start with "Audit only" if you use PsExec, WMI-driven automation or remote-management tools, review Defender's ASR events, then move to "Block". Pair it with the Defender cloud protection tweak, or accept that the ransomware rule does nothing. Skip it if a third-party antivirus is active.

#### Sources
1. Attack surface reduction (ASR) rules reference, per-rule detail sections, establishes all six GUIDs, rule names, dependencies, the High Plus alerting gate and the Program Files scope, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-reference (tier A)
2. ASR rules overview, Microsoft's overview of the rule set and modes, https://learn.microsoft.com/en-us/defender-endpoint/attack-surface-reduction-rules-overview (cited in the YAML Evidence)
3. `C:\Windows\PolicyDefinitions\WindowsDefender.admx` (26100), policy `ExploitGuard_ASR_Rules`, establishes the parent enabling value and its list subkey (tier A)
4. `src-tauri/tweaks/security.yaml`, corpus convention for ASR values and the dedupe check (corpus evidence)

### Restrict outgoing NTLM

`ntlm_outgoing_restriction` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Audits or refuses NTLM authentication this PC sends to remote servers, and requires NTLMv2 session security on top of 128-bit encryption for NTLM sessions in both directions.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `audit_outgoing_ntlm` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` → `AuditOutgoingNTLMTraffic` (REG_DWORD) |
| `restrict_sending_ntlm` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` → `RestrictSendingNTLMTraffic` (REG_DWORD) |
| `ntlm_min_client_sec` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` → `NtlmMinClientSec` (REG_DWORD) |
| `ntlm_min_server_sec` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0` → `NtlmMinServerSec` (REG_DWORD) |

| Option | `audit_outgoing_ntlm` | `restrict_sending_ntlm` | `ntlm_min_client_sec` | `ntlm_min_server_sec` |
|---|---|---|---|---|
| Audit outgoing NTLM | `2` | absent | `0x20080000` (537395200) | `0x20080000` (537395200) |
| Deny outgoing NTLM | `2` | `2` | `0x20080000` | `0x20080000` |
| Allow all | absent | absent | absent | absent |

System Default is shown when the four values match none of the options (for example if a Group Policy or another tool wrote a different session-security value); selecting it restores the snapshot captured before the first apply. Stock Windows has none of the four values, which matches "Allow all"; with the values absent, Windows behaves as "Allow all" for outgoing NTLM and as `0x20000000` (require 128-bit encryption) for session security.

#### How it works

The MSV1_0 authentication package inside LSA (the component that implements NTLM) reads its configuration from `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\MSV1_0`. These are Security Options settings, not administrative templates: none of the four value names appears in any of the 218 ADMX files shipped with build 26100.

Outgoing NTLM restriction: Microsoft's Security Policy Settings page for "Network security: Restrict NTLM: Outgoing NTLM traffic to remote servers" documents three states, Allow all, Audit all and Deny all, and states that "not defined" behaves as Allow all. It does not publish the registry value names or the numbers. The names `RestrictSendingNTLMTraffic` and `AuditOutgoingNTLMTraffic` with values 0, 1 and 2 come from the DISA STIG and CIS check text. With `RestrictSendingNTLMTraffic` = 2 (deny all), this PC refuses to send NTLM credentials to any remote server, so anything that can only authenticate with NTLM (no Kerberos) stops working. The audit option writes `AuditOutgoingNTLMTraffic` = 2 and leaves `RestrictSendingNTLMTraffic` absent, so outgoing NTLM continues to be allowed while being logged.

Minimum session security: `NtlmMinClientSec` governs NTLM SSP sessions this PC opens as a client and `NtlmMinServerSec` governs sessions it accepts as a server. Microsoft's Policy CSP (`NetworkSecurity_MinimumSessionSecurityForNTLMSSPBasedClients` and `...Servers`) documents the values:

| Value | Decimal | Microsoft's description |
|---|---|---|
| `0` | 0 | None |
| `0x00080000` | 524288 | Require NTLMv2 session security |
| `0x20000000` | 536870912 | Require 128-bit encryption (Default) |
| `0x20080000` | 537395200 | Require NTLM and 128-bit encryption (the 128-bit bit plus the NTLMv2 session-security bit) |

`0x20000000` is the 128-bit bit and is already the default. What `0x20080000` adds is the NTLMv2 session-security bit (`0x00080000`). That is the value CIS and the DISA STIG ask for. Microsoft documents 536870912 as the effective default, not as a value Windows writes to the registry, which is why "Allow all" deletes the two values rather than writing `0x20000000` back: writing it would pin a value the OS never had.

#### Benefits
- The audit option shows exactly what still uses NTLM before you deny anything.
- The session-security values add the NTLMv2 session-security requirement that the Windows default leaves off, a genuine one-bit hardening that CIS and the STIG require.
- This is the OS-level NTLM control for everything that authenticates through LSA, not just the SMB client.
- None of these values can lock you out of the local machine: they govern network authentication, not local sign-in.

#### Drawbacks
- Deny mode is aggressive on a workgroup: with `RestrictSendingNTLMTraffic` = 2, any resource reachable only by NTLM stops working, including most consumer NAS devices and many workgroup shares.
- The two outgoing-NTLM value names are benchmark-sourced (DISA STIG and CIS), not published by Microsoft.
- On any network without pre-Vista hosts, the session-security values change almost nothing in practice.
- The tweak offers no option that denies without also auditing, and no option that tightens session security without also auditing.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (Windows 11 24H2 and newer; Windows 10 22H2 and LTSC 2021). Every edition, since these are LSA settings rather than edition-gated policies.
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Allow all" deletes all four values, returning to Allow all and the effective `0x20000000` session-security default. Selecting System Default restores whatever the snapshot captured, including a pre-existing value set by other tooling.

#### Interactions
- [Block NTLM on the SMB client](#block-ntlm-on-the-smb-client) blocks NTLM for SMB only; this tweak is the OS-wide control. Both break NTLM-only NAS devices.
- [Enforce NTLMv2 only](#enforce-ntlmv2-only) and [Prevent LM hash storage](#prevent-lm-hash-storage) harden the same subsystem; the research lists these three as a "legacy authentication lockdown" merge candidate.
- [Require SMB signing](#require-smb-signing) and [Disable SMB insecure guest logons](#disable-smb-insecure-guest-logons) produce the same class of NAS breakage.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the rationale for the session-security value: `0x20000000` is the 128-bit bit (already the default) and `0x20080000` adds the NTLMv2 session-security bit, not the other way round; and the default is an effective value, so the revert deletes rather than writes `0x20000000`.
- **Confidence**: Microsoft-documented for the policy states and the session-security values (Policy CSP and the Security Policy Settings page); the two outgoing-NTLM registry names are benchmark-sourced (tier B).
- **Reasoning**: The session-security values and their default are confirmed verbatim in Microsoft's Policy CSP, which lists 536870912 as the Default Value for both client and server. The three outgoing states and "not defined behaves as Allow all" are confirmed on Microsoft's policy page, which validates deleting as the revert. An all-ADMX scan confirmed neither outgoing-NTLM name is an administrative template. Open question: Microsoft describes auditing as one of the three states of a single "Restrict NTLM: Outgoing NTLM traffic" policy, while the audit option writes a separate `AuditOutgoingNTLMTraffic` value; the research took that name from benchmark check text and did not show on a machine which component reads it or that audit events appear.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Audit outgoing NTLM" on any machine: it logs NTLM use and adds the NTLMv2 session-security requirement at almost no cost. Move to "Deny outgoing NTLM" only on a machine whose network resources all support Kerberos; on a typical home network with a NAS, deny will break it.

#### Sources
1. Policy CSP LocalPoliciesSecurityOptions, `NetworkSecurity_MinimumSessionSecurityForNTLMSSPBasedClients` and `...Servers`, establishes the session-security values and the `0x20000000` default, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-localpoliciessecurityoptions (tier A)
2. Network security: Restrict NTLM: Outgoing NTLM traffic to remote servers, establishes the three states and that "not defined" behaves as Allow all, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-restrict-ntlm-outgoing-ntlm-traffic-to-remote-servers (tier A)
3. DISA STIG for Windows 11 V2R2, the registry value names `RestrictSendingNTLMTraffic` and `AuditOutgoingNTLMTraffic` (tier B)
4. All-ADMX scan across the 218 shipped `.admx` files on 26100, negative result: neither outgoing-NTLM name is an administrative template (tier A)

### Harden the RDP session

`rdp_session_hardening` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Five DISA STIG Remote Desktop session settings beyond the NLA and TLS basics: High encryption, secure RPC, always prompt for a password, no saved RDP passwords and, optionally, no local drive redirection.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `rdp_min_encryption` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` → `MinEncryptionLevel` (REG_DWORD) |
| `rdp_encrypt_rpc` | registry | same key → `fEncryptRPCTraffic` (REG_DWORD) |
| `rdp_prompt_password` | registry | same key → `fPromptForPassword` (REG_DWORD) |
| `rdp_disable_password_saving` | registry | same key → `DisablePasswordSaving` (REG_DWORD) |
| `rdp_disable_drive_redirection` | registry | same key → `fDisableCdm` (REG_DWORD) |

| Option | `rdp_min_encryption` | `rdp_encrypt_rpc` | `rdp_prompt_password` | `rdp_disable_password_saving` | `rdp_disable_drive_redirection` |
|---|---|---|---|---|---|
| Hardened | `3` | `1` | `1` | `1` | `1` |
| Hardened, keep drive redirection | `3` | `1` | `1` | `1` | absent |
| Not hardened | absent | absent | absent | absent | absent |

System Default is shown when the five values match none of the options (for example a Group Policy that sets only some of them, or `fDisableCdm` = 0); selecting it restores the snapshot captured before the first apply. Stock Windows has none of the five policy values, which matches "Not hardened".

#### How it works

All five are Group Policy values from the shipped `TerminalServer.admx` (26100), under the Terminal Services policy key, and all five were confirmed verbatim there:

| Value | ADMX policy | Class | Hardened value and meaning |
|---|---|---|---|
| `MinEncryptionLevel` | `TS_ENCRYPTION_POLICY` | Machine | `3` High Level (enum: `1` Low Level, `2` Client Compatible, `3` High Level) |
| `fEncryptRPCTraffic` | `TS_RPC_ENCRYPTION` | Machine | `1`: require secure RPC communication with the session host |
| `fPromptForPassword` | `TS_PASSWORD` | Machine | `1`: always prompt for a password on connection, even if the client supplied saved credentials |
| `DisablePasswordSaving` | `TS_CLIENT_DISABLE_PASSWORD_SAVING_1` and `_2` | User and Machine | `1`: the Remote Desktop client cannot save passwords |
| `fDisableCdm` | `TS_CLIENT_DRIVE_M` | Machine | `1`: do not allow drive redirection into RDP sessions |

Remote Desktop Services reads the host-side values (`MinEncryptionLevel`, `fEncryptRPCTraffic`, `fPromptForPassword`, `fDisableCdm`) when a session is set up. `DisablePasswordSaving` is a client-side policy: it affects this machine's Remote Desktop client when it connects out. The ADMX defines it twice against the same key, once as class User (`_1`) and once as class Machine (`_2`); the HKLM write this tweak makes is the correct and effective one.

`MinEncryptionLevel` = 3 is the maximum the shipped ADMX offers: the 26100 enum has no FIPS level (`4`) and the ADML has no `TS_ENCRYPTION_FIPS_LEVEL` string. The encryption level governs the legacy RDP security layer. When TLS is the security layer (which the [Harden RDP (NLA + TLS)](#harden-rdp-nla--tls) tweak forces), TLS negotiates the encryption and `MinEncryptionLevel` largely does not decide anything. It is still the STIG check, which is why it is included.

The ADMX `supportedOn` for all five is `SUPPORTED_WindowsXP` / `SUPPORTED_WindowsNET`, so no build gate is needed. All five do nothing unless Remote Desktop is in use.

#### Benefits
- No saved RDP passwords: removes a credential store an attacker could harvest from this PC.
- No drive redirection (in "Hardened"): drives on computers that connect to this PC are not mapped into their sessions, so files cannot be copied between the two through a redirected drive.
- Always prompts: a cached or passed-through credential cannot be used silently to open a session.
- A second hardened option keeps drive redirection, so the file-transfer cost is separable.

#### Drawbacks
- `fDisableCdm` stops copying files through a redirected drive inside an RDP session, which many people rely on; choose "Hardened, keep drive redirection" if you need it.
- `MinEncryptionLevel` is largely inert when TLS is the security layer.
- `DisablePasswordSaving` also affects this machine connecting out: you cannot save passwords in the Remote Desktop client.
- All five do nothing unless Remote Desktop is in use.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build. The host-side settings matter only where the Remote Desktop host is available, which the YAML states as Pro and above; the ADMX `supportedOn` reaches back to Windows XP.
- **Takes effect**: immediately, on the next connection.
- **Reverting**: "Not hardened" deletes all five values, restoring the shipped defaults. Selecting System Default restores the values captured in the snapshot.

#### Interactions
- [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp) is the opposite posture: with the listener off, these five do nothing. Pick one.
- [Harden RDP (NLA + TLS)](#harden-rdp-nla--tls) forces TLS, under which `MinEncryptionLevel` is largely inert. The research lists all three Remote Desktop tweaks as a merge candidate ("Off", "On, hardened", "Windows default").
- [Force updated CredSSP clients](#force-updated-credssp-clients) hardens the CredSSP layer that Network Level Authentication uses.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting. The research noted that `DisablePasswordSaving` is defined for both User and Machine classes (the HKLM write is the effective one), that 3 is the highest encryption level the shipped ADMX offers, and that `MinEncryptionLevel` is largely inert under TLS.
- **Confidence**: Microsoft-documented: every value, class, enum and polarity was read verbatim from the shipped `TerminalServer.admx` and `.adml` on 26100.
- **Reasoning**: The adversarial pass checked each value against the shipped ADMX (all five correct), looked for a FIPS level (none on 26100), and found the overlap with the TLS tweak. None of that changes the mechanism; it changes what the copy may claim.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use Remote Desktop and want the STIG-level session settings; take "Hardened, keep drive redirection" if you copy files through RDP. If you do not use Remote Desktop at all, disable it instead; these five settings do nothing for you.

#### Sources
1. `C:\Windows\PolicyDefinitions\TerminalServer.admx` and `en-US\TerminalServer.adml` (26100), policies `TS_ENCRYPTION_POLICY`, `TS_RPC_ENCRYPTION`, `TS_PASSWORD`, `TS_CLIENT_DISABLE_PASSWORD_SAVING_1` / `_2`, `TS_CLIENT_DRIVE_M`, establishes keys, value names, classes and the enum (tier A, shipped ADMX)
2. Policy CSP RemoteDesktopServices, Microsoft's reference for the same Terminal Services policies, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-remotedesktopservices (cited in the YAML Evidence)
3. DISA STIG for Windows 11 V2R2, five separate Remote Desktop rules, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
4. `src-tauri/tweaks/security.yaml`, `rdp_security_hardening`, overlap evidence (corpus evidence)

### Disable the Secondary Logon service

`disable_secondary_logon` · Switch · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Disables the service behind `runas` and "Run as different user", removing a routine step in privilege-escalation and lateral-movement chains.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `seclogon_service` | service | `seclogon` (display name "Secondary Logon"), start type |

| Option | `seclogon_service` |
|---|---|
| Disabled | `disabled` |

This is a toggle: the only authored option is "Disabled". System Default is shown whenever the start type is anything other than Disabled; selecting it restores the start type captured in the snapshot before the first apply. The stock start type is not established by any authoritative source (see Validation), which is why the tweak authors no "stock" option.

#### How it works

Secondary Logon (`seclogon`, implemented in `System32\seclogon.dll`) is the service that starts a process under a different user account. It backs the `runas` command, the Shift plus right-click "Run as different user" shell verb, and the `CreateProcessWithLogonW` API, which some installers, deployment tools and scripted automation call directly. The DISA STIG has a standalone rule, "The Secondary Logon service must be disabled on Windows 11".

The tweak sets the start type through the Service Control Manager. The SCM dependency graph on 26100 shows no dependent services and no dependencies, so disabling it does not cascade to anything else. The service effect changes the start type only; it does not stop an instance that is already running. Once the start type is Disabled, the service cannot be started again, so `runas` fails as soon as no instance is running (at the latest after the next restart).

Normal UAC elevation of the same user does not go through Secondary Logon, so the everyday "Run as administrator" experience does not change.

#### Benefits
- Removes an escalation step: running a process as another user is a routine move in local privilege-escalation and lateral-movement chains.
- Contained: no other service depends on it.
- Rarely used on a consumer machine: single-account users never touch it.

#### Drawbacks
- `runas` stops working, as does the "Run as different user" menu entry.
- Some installers, deployment tools and scripts that call `CreateProcessWithLogonW` directly break.
- If you keep a separate administrator account, or administer other machines from this one with different credentials, you will notice immediately.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (Windows 11 24H2 and newer; Windows 10 22H2 and LTSC 2021).
- **Takes effect**: new starts are blocked immediately; an instance that is already running keeps running until it stops or the machine restarts. The tweak is flagged as needing a reboot, since only a restart guarantees no instance is left running.
- **Reverting**: selecting System Default restores the start type captured in the snapshot, exactly (including delayed-start). The tweak never hardcodes a stock start type.

#### Interactions
- None known in this corpus: `services.yaml` ships no tweak targeting `seclogon`. Same-user UAC elevation, including [Raise UAC to always notify](#raise-uac-to-always-notify), is unaffected.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting. The research required the revert to restore the start type from the snapshot rather than hardcode Manual, and added `CreateProcessWithLogonW` callers to the risk.
- **Confidence**: Community-corroborated: the control is a DISA STIG rule (tier B) and the service's existence and dependency graph were read on 26100; no Microsoft page documents disabling it.
- **Reasoning**: The research confirmed the service exists on 26100 with no dependents or dependencies and is not in the corpus. It attacked the risk statement and found it slightly optimistic (installers and automation using `CreateProcessWithLogonW`), while confirming that same-user UAC elevation is unaffected. Open question: the shipped start type could not be established. The gap proposal said Manual, which is consistent with the STIG requiring Disabled, but it was not confirmed against a clean image (research UNKNOWNS item 14). Restoring from the snapshot makes the revert correct either way.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a single-account consumer machine, where Secondary Logon is essentially never used. Skip it if you keep a separate administrator account, use `runas` in your workflow, or run deployment tooling that launches processes under other credentials.

#### Sources
1. DISA STIG for Windows 11 V2R2, "The Secondary Logon service must be disabled on Windows 11", https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
2. CreateProcessWithLogonW, the API that Secondary Logon backs, https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw (cited in the YAML Evidence)
3. Live service presence and SCM dependency graph on 26100, establishes existence and no dependents (tier A for existence only, not for the default start type)
4. `src-tauri/tweaks/services.yaml`, dedupe evidence (corpus evidence); privacy.sexy and Sophia Script, corroboration (tier C)

### Early Launch Antimalware driver policy

`early_launch_antimalware_policy` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Pins the boot-time driver policy that Early Launch Antimalware enforces, so malware cannot loosen it to "no filtering" and get an unsigned boot driver loaded.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `driver_load_policy` | registry | `HKLM\SYSTEM\CurrentControlSet\Policies\EarlyLaunch` → `DriverLoadPolicy` (REG_DWORD) |

| Option | `driver_load_policy` |
|---|---|
| Good, unknown and bad but critical | `3` |
| Good and unknown | `1` |
| Windows decides | absent |

System Default is shown when the value is something none of the options writes (`8` or `7`, for example if other software set it); selecting it restores the snapshot captured before the first apply. Stock Windows has no value, which matches "Windows decides"; the effective behaviour with no value is `3`.

#### How it works

Early Launch Antimalware (ELAM) lets an antimalware vendor's ELAM driver start before other boot-start drivers and classify each one as good, bad, bad but boot-critical, or unknown. Windows then decides which drivers to initialise according to `DriverLoadPolicy`. The shipped `EarlyLaunchAM.admx` (26100) defines policy `POL_DriverLoadPolicy_Name`, class Machine, key `System\CurrentControlSet\Policies\EarlyLaunch`, `supportedOn` `SUPPORTED_Windows8`, with exactly four enum members:

| Value | ADML display name |
|---|---|
| `8` | Good only |
| `1` | Good and unknown |
| `3` | Good, unknown and bad but critical |
| `7` | All (no filtering) |

`0` is not a member. The value is read at boot. Value `3` is the effective default: it loads drivers classified good or unknown plus bad drivers that are boot-critical, and refuses the rest. Without the value pinned, malware with admin rights can set `7` ("no filtering") so a bad boot driver loads on the next start. The tweak never offers `7`, and never offers `8`, which refuses any driver ELAM cannot vouch for and on unusual hardware can leave a machine that does not start. `1` additionally refuses known-bad drivers even when they are boot-critical.

#### Benefits
- Stops the loosening attack: with the value written, a change to `7` is an explicit, visible policy change rather than a silent one.
- Auditable: it is an explicit DISA STIG and CIS check with a specific value; the STIG accepts `3` or `1`.
- Value `3` is what Windows already does, so nothing legitimate stops loading.

#### Drawbacks
- No behavioural change at `3`: it pins the effective default rather than altering it.
- Needs a reboot, because the policy is read at boot.
- "Good and unknown" (`1`) refuses bad-but-critical boot drivers; if a boot-critical driver is classified bad, the machine may fail to start.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the ADMX supports Windows 8 and later). The policy acts on the classifications an ELAM driver reports at boot.
- **Takes effect**: after a reboot.
- **Reverting**: "Windows decides" deletes the value, which is the shipped state. Selecting System Default restores the snapshot value. Either takes effect at the next boot.

#### Interactions
- [Enable the Microsoft vulnerable-driver blocklist](#enable-the-microsoft-vulnerable-driver-blocklist) is a separate driver-loading control; the two are complementary.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the enum: "Good only" is `8`, not `0`, and `0` is not a member of the enum at all.
- **Confidence**: Microsoft-documented: the enum, key, value name and class were read verbatim from the shipped `EarlyLaunchAM.admx` on 26100, and the STIG and CIS back the hardened value.
- **Reasoning**: Key, value name, type, class, `supportedOn` and the hardened value `3` all survived the adversarial pass. The only defect was the value list in the proposal. The option set is deliberately limited to `3` and `1` to avoid both the unbootable state (`8`) and the no-filtering state (`7`).
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it at "Good, unknown and bad but critical" on any machine: it costs a reboot and pins the setting malware would loosen. Choose "Good and unknown" only if you also want known-bad boot-critical drivers refused and accept a small risk of a boot failure on unusual hardware.

#### Sources
1. `C:\Windows\PolicyDefinitions\EarlyLaunchAM.admx` (26100), policy `POL_DriverLoadPolicy_Name`, verbatim, establishes the key, value, class and the four enum members (tier A, shipped ADMX)
2. Early Launch Antimalware, Microsoft's description of ELAM classification, https://learn.microsoft.com/en-us/windows-hardware/drivers/install/early-launch-antimalware (cited in the YAML Evidence)
3. DISA STIG for Windows 11 V2R2 and CIS Windows 11 v4.0.0 Level 1, the ELAM boot-start driver policy rule accepting `3` or `1`, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)

### Force updated CredSSP clients

`credssp_encryption_oracle` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Refuses CredSSP connections (such as Remote Desktop with Network Level Authentication) from clients that never received the 2018 fix for CVE-2018-0886.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `allow_encryption_oracle` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\CredSSP\Parameters` → `AllowEncryptionOracle` (REG_DWORD) |

| Option | `allow_encryption_oracle` |
|---|---|
| Force updated clients | `0` |
| Mitigated | absent |

System Default is shown when the value is present with anything other than `0` (an explicit `1` or `2` written by Group Policy or another tool); selecting it restores the snapshot captured before the first apply. Stock Windows has no value, which matches "Mitigated"; the effective default with no value is `1` (Mitigated) since the May 8, 2018 update.

#### How it works

CredSSP (Credential Security Support Provider) is the protocol that delegates credentials during Remote Desktop Network Level Authentication and a few other remote-management scenarios. CVE-2018-0886 is an encryption-oracle flaw in older CredSSP. The shipped `CredSsp.admx` (26100) defines policy `AllowEncryptionOracle` ("Encryption Oracle Remediation"), class Machine, at the key above, with three values that Microsoft's CVE-2018-0886 KB documents with the same names:

| Value | Name | Behaviour |
|---|---|---|
| `0` | Force Updated Clients | Services that use CredSSP refuse unpatched clients; the client never falls back to insecure versions. |
| `1` | Mitigated | The client does not fall back to insecure versions; services that use CredSSP still accept unpatched clients. |
| `2` | Vulnerable | Both sides allow the insecure behaviour. Never written by this tweak. |

Microsoft's KB records "May 8, 2018. An update to change the default setting from Vulnerable to Mitigated." and "By default, after this update is installed, patched clients cannot communicate with unpatched servers." So every 24H2 machine is already at Mitigated, and moving to `0` changes only the server (accept) half: CredSSP services on this PC stop accepting clients that still run the pre-2018 behaviour. The client half is already in force at the default. The ADMX `supportedOn` is `SUPPORTED_WindowsVista`, so no build gate is needed.

#### Benefits
- Closes the remaining half of CVE-2018-0886: the default already protects this PC as a client, but still accepts unpatched clients as a server.
- Explicit and auditable: it is a CIS Level 1 check with a specific value.
- No effect on normal use: every supported Windows client has been patched since 2018.

#### Drawbacks
- Clients still running the pre-2018 CredSSP behaviour can no longer connect in.
- Smaller change than it sounds: only the accept side moves.
- Inert if nothing on this machine accepts CredSSP connections (for example, Remote Desktop is off).

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the ADMX supports Windows Vista and later), every edition.
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Mitigated" deletes the value, returning to the Mitigated default. Selecting System Default restores the snapshot value.

#### Interactions
- [Harden RDP (NLA + TLS)](#harden-rdp-nla--tls) requires Network Level Authentication, which runs over CredSSP; this tweak hardens that layer.
- [Disable Remote Desktop (RDP)](#disable-remote-desktop-rdp) removes the main CredSSP listener, which makes this tweak largely inert.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the stock default: it is Mitigated (`1`) since the May 2018 update, not Vulnerable (`2`), so the hardened value changes only the server half.
- **Confidence**: Microsoft-documented: the ADMX was read verbatim on 26100 and the KB's changelog line and three-value table confirm the values and the default change.
- **Reasoning**: Key, value, class, enum and `supportedOn` all survived. The gap proposal's premise ("the OS still ships with the permissive behaviour") was refuted by Microsoft's own changelog; the proposal's stated risk (outbound RDP to old unpatched servers failing) describes what Mitigated already does, so the real risk is only that old clients cannot connect in.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it. On a modern network nothing legitimate is still unpatched, and it closes the accept side the Windows default leaves open. Skip it only if you must accept Remote Desktop connections from a machine that has not been updated since 2018.

#### Sources
1. `C:\Windows\PolicyDefinitions\CredSsp.admx` (26100), policy `AllowEncryptionOracle`, establishes the key, value, class and three-value enum (tier A, shipped ADMX)
2. CredSSP updates for CVE-2018-0886, the changelog line changing the default from Vulnerable to Mitigated and the three-option table, https://support.microsoft.com/en-us/topic/credssp-updates-for-cve-2018-0886-5cbf9e5f-dc6d-744f-9e97-7ba400d6d3ea (tier A)
3. CIS Windows 11 v4.0.0, encryption oracle remediation, https://www.cisecurity.org/benchmark/microsoft_windows_desktop (tier B)

### Prevent automatic device encryption

`device_encryption_posture` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows automatically turning on device encryption (BitLocker) and escrowing the recovery key to your Microsoft account, so encryption becomes your decision. This reduces protection rather than adding it.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `prevent_device_encryption` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\BitLocker` → `PreventDeviceEncryption` (REG_DWORD) |

| Option | `prevent_device_encryption` |
|---|---|
| Prevent automatic encryption | `1` |
| Automatic encryption allowed | absent |

System Default is shown when the value is present with anything other than `1` (for example `0`); selecting it restores the snapshot captured before the first apply. Stock Windows has no value, which matches "Automatic encryption allowed".

#### How it works

Device encryption is the automatic form of BitLocker that Windows turns on during setup or first sign-in on qualifying hardware, including Home edition, and it escrows the recovery key to the signed-in Microsoft account. Windows 11 24H2 relaxed the hardware prerequisites, so clean installs and new PCs auto-encrypt far more often. Microsoft Learn's BitLocker overview, section "Disable device encryption", documents exactly this value: path `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\BitLocker`, name `PreventDeviceEncryption`, type REG_DWORD, value `0x1`. It is a direct registry setting, not a Group Policy: it appears in none of the 218 shipped ADMX files.

The value only stops automatic enablement. It does not decrypt a volume that is already encrypted, and the tweak writes and verifies only the registry value; it does not read or change the BitLocker volume state. Check the volume state yourself (Settings, or `manage-bde -status`) before and after.

The registry value and automatic device encryption long predate 24H2; what 24H2 changed is how many machines qualify. The value is read the same way on Windows 10 and LTSC 2021, which is why the tweak has no build gate.

#### Benefits
- You choose whether to encrypt, not the installer.
- Recovery-key custody stays with you: no key is escrowed to a Microsoft account without a deliberate decision.
- It is Microsoft's own documented opt-out, not a workaround.

#### Drawbacks
- It reduces data-at-rest protection: an unencrypted drive can be read by anyone who takes it. That is the point of the setting, and it is the opposite direction to every other tweak in this category.
- It only affects automatic enablement: on an already-encrypted machine it changes nothing visible.
- One-way in practice: once device encryption is off it does not re-enable itself when you revert; you turn it on in Settings.
- The performance argument has shrunk: Windows 11 25H2 shipped hardware-accelerated BitLocker, so the throughput cost on fast NVMe drives is much smaller than it was.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build. Most relevant on Windows 11 24H2 and newer, where automatic device encryption is common; the value is also read on Windows 10 and LTSC 2021.
- **Takes effect**: immediately, but only for encryption that has not started yet.
- **Reverting**: "Automatic encryption allowed" deletes the value, allowing automatic device encryption again. Selecting System Default restores the snapshot value. Neither re-encrypts or decrypts anything by itself.

#### Interactions
- None known in this corpus.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research found the proposal's `>=26100` build gate too narrow, because the value works on older builds too.
- **Confidence**: Microsoft-documented: the path, name, type and value match the Microsoft Learn BitLocker overview exactly.
- **Reasoning**: Key, name, type and value all matched Microsoft's table; an all-ADMX scan confirmed it is a direct registry setting rather than a policy. The research noted the awkwardness of a tweak whose applied direction is less protection sitting in a security category; the honest framing is user control over encryption and recovery-key custody, not performance. The research also said the tweak should look at the actual volume state; the shipped tweak does not, so the volume check is left to the user.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Use it only if you deliberately want to manage encryption and recovery keys yourself and understand the trade-off. For a laptop that leaves the house, leave automatic device encryption alone: the data-at-rest protection is worth more than the control.

#### Sources
1. BitLocker overview, "Disable device encryption" table, establishes the path, name, type and value, https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/ (tier A)
2. Announcing hardware-accelerated BitLocker, establishes the 25H2 performance change, https://techcommunity.microsoft.com/blog/windows-itpro-blog/announcing-hardware-accelerated-bitlocker/4474609 (tier B)
3. All-ADMX scan across the 218 shipped `.admx` files on 26100, negative result: a direct registry setting, not a policy (tier A)

### Hide admin accounts on the UAC prompt

`hide_admin_accounts_on_elevation` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the UAC credential prompt listing every administrator account by name and picture, and removes the eye button that reveals a typed password.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `enumerate_administrators` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\CredUI` → `EnumerateAdministrators` (REG_DWORD) |
| `disable_password_reveal_machine` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\CredUI` → `DisablePasswordReveal` (REG_DWORD) |
| `disable_password_reveal_user` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\CredUI` → `DisablePasswordReveal` (REG_DWORD) |

| Option | `enumerate_administrators` | `disable_password_reveal_machine` | `disable_password_reveal_user` |
|---|---|---|---|
| Hidden | `0` | `1` | `1` |
| Shown | absent | absent | absent |

System Default is shown when the three values match neither option (for example only one of them set, or `EnumerateAdministrators` = 1); selecting it restores the snapshot captured before the first apply. Stock Windows has none of the three values, which matches "Shown".

#### How it works

Both policies come from the shipped `CredUI.admx` (26100), and they live under two different keys with two different classes:

- `EnumerateAdministrators` (policy of the same name, class Machine, key `Software\Microsoft\Windows\CurrentVersion\Policies\CredUI`, supported from Windows Vista). Enabled (`1`) means the credential prompt does enumerate administrator accounts, so the hardened value is `0`: when a standard user triggers elevation, the prompt no longer lists the local administrators to pick from, and the user must type an administrator name.
- `DisablePasswordReveal` (class Both, key `Software\Policies\Microsoft\Windows\CredUI`, supported from Windows 8 or IE10). Enabled (`1`) removes the password-reveal button from password fields, so the hardened value is `1`.

Because `DisablePasswordReveal` is class Both, the tweak writes it in HKLM and HKCU and removes both on revert. The HKCU write lands in the hive of the account the app runs as when elevated; if you elevate MagicX Toolbox with a different administrator account, the per-user copy goes to that account, and the machine copy is what covers everyone else.

These are physical-presence protections: they change what someone at the screen can see.

#### Benefits
- Anyone at the machine can no longer read off which accounts are administrators.
- Blocks shoulder surfing: the reveal button cannot be used to display a typed password.
- Complements [Hide last signed-in username](#hide-last-signed-in-username): a different value in a different key, additive.

#### Drawbacks
- More typing at the prompt: you type the administrator username instead of clicking it.
- Without the reveal button, a mistyped long password is harder to spot.
- No effect on remote attacks.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build. `EnumerateAdministrators` is supported from Windows Vista and `DisablePasswordReveal` from Windows 8.
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Shown" deletes all three values, including the per-user copy. Selecting System Default restores the snapshot values.

#### Interactions
- [Hide last signed-in username](#hide-last-signed-in-username) covers `DontDisplayLastUserName`, a different value on the sign-in screen; the two are complementary.
- [Raise UAC to always notify](#raise-uac-to-always-notify) and [Apply UAC to built-in Administrator](#apply-uac-to-built-in-administrator) change when the prompt appears; this tweak changes what it shows.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research added that `DisablePasswordReveal` is class Both, so the HKCU twin is written and reverted too.
- **Confidence**: Microsoft-documented: both policies, their keys, classes and polarity were read verbatim from the shipped `CredUI.admx` on 26100.
- **Reasoning**: The pass checked polarity for both values (`EnumerateAdministrators` 1 means enumerate, so 0 is hardened; `DisablePasswordReveal` 1 means disable the reveal, so 1 is hardened) and confirmed the unusual two-key, two-class split. Neither value appears elsewhere in the corpus.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine used where other people can see the screen. On a private desktop the gain is small, but the cost is only a little extra typing.

#### Sources
1. `C:\Windows\PolicyDefinitions\CredUI.admx` (26100), policies `EnumerateAdministrators` and `DisablePasswordReveal`, verbatim, establishes keys, classes, polarity and `supportedOn` (tier A, shipped ADMX)
2. Policy CSP CredentialsUI, Microsoft's reference for the same CredUI policies, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-credentialsui (cited in the YAML Evidence)
3. DISA STIG for Windows 11 V2R2 ("Administrator accounts must not be enumerated during elevation") and CIS Windows 11 v4.0.0 Level 1, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
4. `src-tauri/tweaks/security.yaml`, `hide_last_user`, dedupe evidence (corpus evidence)

### Event log retention size

`event_log_retention` · Dropdown (3 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Makes the Application, System and Security event logs big enough that a busy day does not overwrite the evidence.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `application_log_max_size` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Application` → `MaxSize` (REG_DWORD, kilobytes) |
| `system_log_max_size` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\System` → `MaxSize` (REG_DWORD, kilobytes) |
| `security_log_max_size` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Security` → `MaxSize` (REG_DWORD, kilobytes) |

| Option | `application_log_max_size` | `system_log_max_size` | `security_log_max_size` |
|---|---|---|---|
| Standard (32 MB / 32 MB / 192 MB) | `32768` | `32768` | `196608` |
| Large, STIG (32 MB / 32 MB / 1 GB) | `32768` | `32768` | `1024000` |
| Windows decides | absent | absent | absent |

System Default is shown when the three values match none of the options (for example sizes set by a Group Policy); selecting it restores the snapshot captured before the first apply. Stock Windows has none of the three policy values, which matches "Windows decides".

#### How it works

The shipped `EventLog.admx` (26100) defines four channel log-size policies, each class Machine with a single element `<decimal id="Channel_LogMaxSize" valueName="MaxSize" required="true" minValue="1024" maxValue="2147483647" />`:

| ADMX policy | Key |
|---|---|
| `Channel_LogMaxSize_1` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Application` |
| `Channel_LogMaxSize_2` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Security` |
| `Channel_LogMaxSize_3` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\Setup` |
| `Channel_LogMaxSize_4` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\EventLog\System` |

The tweak sets three of the four; the Setup channel is not touched. The Windows Event Log service honours the policy `MaxSize` as the channel's maximum size in kilobytes, overriding the size set in Event Viewer. All sizes the tweak writes (32768, 196608 and 1024000 KB) sit inside the declared range, so none can be silently rejected. With the default retention behaviour, a full log overwrites its oldest events, which is why a small Security log loses evidence quickly once auditing is on.

The per-channel default size when no policy is set is not established by any source the research found; the commonly repeated 20480 KB figure is unsourced and not stated here.

#### Benefits
- Makes auditing useful: turning on logon or process auditing without sizing the log is half a control.
- A longer retention window: incidents are often noticed days after they happen.
- Safe values inside the ADMX-declared range.

#### Drawbacks
- Uses disk: the 1 GB Security log in the STIG option is a real allocation on a small SSD.
- No security effect on its own: it changes retention, not detection.
- Covers three of the four channels; the Setup channel has its own policy and is left alone.
- On a managed machine, an event-log Group Policy wins at the next refresh.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build.
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Windows decides" deletes the three values, returning each channel to its own default size. Selecting System Default restores the snapshot values.

#### Interactions
- Pairs with [Enable logon/credential auditing](#enable-logoncredential-auditing), [Log command lines in process-creation events](#log-command-lines-in-process-creation-events), [Enable PowerShell script-block logging](#enable-powershell-script-block-logging) and [PowerShell module logging and transcription](#powershell-module-logging-and-transcription), which are what fill the logs.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research noted that the Setup channel exists and is not covered, and that the 20480 KB default is unsourced.
- **Confidence**: Microsoft-documented: all four channel policy blocks, including the `minValue` and `maxValue`, were read verbatim from the shipped `EventLog.admx` on 26100.
- **Reasoning**: Value name, type, keys and the range check all survived. Open question: the channel defaults with no policy value were not confirmed (research UNKNOWNS item 17); this does not affect the revert, which deletes.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Standard" if you have enabled any auditing or PowerShell logging. Take "Large, STIG" only if you actually review the Security log and have the disk space.

#### Sources
1. `C:\Windows\PolicyDefinitions\EventLog.admx` (26100), all four `Channel_LogMaxSize_*` policy blocks verbatim, including `minValue="1024" maxValue="2147483647"` (tier A, shipped ADMX)
2. Policy CSP EventLogService, Microsoft's reference for the same channel size policies, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-eventlogservice (cited in the YAML Evidence)
3. DISA STIG for Windows 11 V2R2, three separate event-log size rules, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)

### Log command lines in process-creation events

`audit_process_creation_cmdline` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns on process-creation auditing and records the full command line of every process Windows starts in event 4688, so the log says what was run, not only that `powershell.exe` ran.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `include_cmdline` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System\Audit` → `ProcessCreationIncludeCmdLine_Enabled` (REG_DWORD) |
| `audit_process_creation` | action (PowerShell, `apply` / `undo` / `probe`) | Advanced audit policy subcategory Detailed Tracking > Process Creation, `{0CCE922B-69AE-11D9-BED3-505054503030}`, via `auditpol`; stashes the prior setting in `HKLM\SOFTWARE\MagicXToolbox\State`, value `AuditProcessCreationPriorSetting` (REG_DWORD) |

| Option | `include_cmdline` | `audit_process_creation` |
|---|---|---|
| Enabled | `1` | run |
| Off | absent | not run (its undo restores the stashed setting) |

The action's scripts:

- **apply**: reads the subcategory's current setting as a number (bit 1 Success, bit 2 Failure) from the `Setting Value` column of an `auditpol /backup` file, failing if it cannot; stores it as `AuditProcessCreationPriorSetting` unless a stash is already there (so a re-apply never replaces the pre-tweak setting with this tweak's own); then runs `auditpol /set /subcategory:{0CCE922B-...} /success:enable` and exits with `auditpol`'s exit code. Failure auditing is left as it was.
- **undo**: sets Success and Failure back from the stashed number (both off if there is no stash, the Windows default), deletes the stash once `auditpol` succeeds, and exits with `auditpol`'s exit code.
- **probe**: reports present when the subcategory's numeric setting has the Success bit.

The setting is read as a number because the text `auditpol /get` prints ("No Auditing", "Success") is translated on non-English Windows.

System Default is shown when the live state matches neither option: for example `ProcessCreationIncludeCmdLine_Enabled` = 1 while process-creation success auditing is off, or the value absent while success auditing is on (set by another tool or policy). Selecting it restores the snapshot. "Enabled" expects the value at 1 and the probe present; "Off" expects the value absent and the probe absent. The shipped default of the Process Creation subcategory was not established by the research.

#### How it works

Two pieces are needed, and each alone does nothing useful. The shipped `AuditSettings.admx` (26100) defines policy `IncludeCmdLine` ("Include command line in process creation events"), class Machine, key `Software\Microsoft\Windows\CurrentVersion\Policies\System\Audit`, value `ProcessCreationIncludeCmdLine_Enabled`, enabled 1, disabled 0, supported from Windows 8.1 (`SUPPORTED_Windows_6_3`). That value only adds the command line to event 4688 in the Security log. Event 4688 itself is generated only when the Detailed Tracking > Process Creation audit subcategory is enabled for success, which is what the `auditpol` action does.

Audit subcategory state is not a registry value the snapshot can capture, so the action stashes the prior inclusion setting in HKLM before changing it and its undo restores that setting. Deleting only the DWORD on revert would leave process-creation auditing switched on, a silent state leak. The stash lives in HKLM because the change is machine-wide.

This uses a different audit subcategory from the logon-auditing tweak (`{0CCE9215-...}`), so the two do not collide.

#### Benefits
- Turns event 4688 into evidence: a process name alone tells you almost nothing; the command line tells you what was run.
- The highest-value addition to the audit story: two separate DISA STIG rules (the command-line policy and the Detailed Tracking subcategory).
- Cheap: one policy value plus one audit subcategory.

#### Drawbacks
- Command lines can contain secrets: a password passed as an argument lands in the Security log, readable by every administrator on the machine.
- Log volume: process creation is frequent, so the Security log fills faster; pair it with a larger Security log.
- The revert depends on the stash: if `AuditProcessCreationPriorSetting` is missing when the undo runs, it disables both success and failure auditing for the subcategory.
- If process-creation success auditing is already on (for example by Group Policy) while the command-line value is absent, the tweak reads System Default; applying writes the value and leaves the audit action alone, because its probe already reads present, so nothing is recorded that a revert would undo.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the ADMX supports Windows 8.1 and later).
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Off" deletes the policy value and runs the action's undo, which restores the Process Creation subcategory to the setting stashed at apply time. Selecting System Default restores the registry value from the snapshot; the audit subcategory is restored by the action's undo.

#### Interactions
- [Enable logon/credential auditing](#enable-logoncredential-auditing) uses the same `auditpol` pattern on a different subcategory and its own stash value; no collision.
- [Event log retention size](#event-log-retention-size) sizes the Security log these events fill.
- [Enable PowerShell script-block logging](#enable-powershell-script-block-logging) records script content, which complements the command lines recorded here.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research required the `auditpol` half to revert from a captured state rather than leaving auditing on.
- **Confidence**: Microsoft-documented: the policy was read verbatim from the shipped `AuditSettings.admx` on 26100, and Microsoft documents the Process Creation subcategory and `auditpol /set`.
- **Reasoning**: Key, value, type, polarity and `supportedOn` survived. The pass found that the registry value alone logs nothing and that `auditpol` state is outside the registry snapshot; the shipped action captures and restores it. The privacy caution about secrets in command lines was confirmed as real.
- **Tested**: Build validation (schema, ownership and conflict checks); on build 26100 the action applied twice and then undone under Windows PowerShell 5.1 kept the first stash and restored `No Auditing` exactly.

#### Recommendation
Enable it if you have enabled logon auditing or care about forensic evidence; without command lines, process events are close to useless. Enlarge the Security log at the same time. Skip it if administrators on this machine should not see credentials that scripts pass as arguments.

#### Sources
1. `C:\Windows\PolicyDefinitions\AuditSettings.admx` (26100), policy `IncludeCmdLine`, verbatim (tier A, shipped ADMX)
2. Audit Process Creation, Microsoft's description of the subcategory and event 4688, https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-process-creation (cited in the YAML Evidence)
3. auditpol set, the command syntax, https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/auditpol-set (cited in the YAML Evidence)
4. DISA STIG for Windows 11 V2R2 ("Command line data must be included in process creation events" and the Detailed Tracking rule) (tier B)
5. `src-tauri/tweaks/security.yaml`, `audit_logon_events`, the existing `auditpol` pattern and no subcategory collision (corpus evidence)

### Turn off the spooler's remote RPC endpoint

`spooler_remote_rpc_off` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the Print Spooler accepting client connections from the network, and requires packet privacy on spooler RPC, while printing from this PC keeps working.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `spooler_remote_rpc` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers` → `RegisterSpoolerRemoteRpcEndPoint` (REG_DWORD) |
| `spooler_rpc_privacy` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Print` → `RpcAuthnLevelPrivacyEnabled` (REG_DWORD) |

| Option | `spooler_remote_rpc` | `spooler_rpc_privacy` |
|---|---|---|
| No client connections | `2` | `1` |
| Client connections allowed | absent | absent |

System Default is shown when the two values match neither option (for example `RegisterSpoolerRemoteRpcEndPoint` = 1, or only one of the two set); selecting it restores the snapshot captured before the first apply. Stock Windows has neither value, which matches "Client connections allowed"; with the policy value absent the spooler accepts client connections.

#### How it works

`RegisterSpoolerRemoteRpcEndPoint` is the policy "Allow Print Spooler to accept client connections" from the shipped `Printing2.admx` (26100): class Machine, key `Software\Policies\Microsoft\Windows NT\Printers`, enabled writes `1` (accept), disabled writes `2` (do not accept). The value must be under the Printers policy key: written under `Control\Print` it lands where the spooler's policy path never reads it, and the tweak would report success while changing nothing. The ADML explain text: "When the policy is disabled, the spooler will not accept client connections nor allow users to share printers. All printers currently shared will continue to be shared. The spooler must be restarted for changes to this policy to take effect."

`RpcAuthnLevelPrivacyEnabled` is policy `ConfigureRpcAuthnLevelPrivacyEnabled` from the shipped `Printing.admx`: class Machine, key `System\CurrentControlSet\Control\Print`, enabled `1`, disabled `0`. At `1` the spooler requires RPC packet privacy (encryption) on incoming print RPC connections.

The result is a middle position between disabling the Print Spooler service (no printing at all) and only restricting driver installation: printing from this PC to local and network printers keeps working, while the spooler's remote RPC endpoint (the remote half of the PrintNightmare class of bugs) is no longer exposed.

#### Benefits
- Keeps local and network printing without exposing the spooler's remote RPC endpoint.
- Covers the remote half of PrintNightmare; the driver-installation restriction covers the other half.
- Quick to reverse: two values plus a spooler restart.

#### Drawbacks
- Remote printing to this PC and sharing printers from it stop working.
- Printers already shared continue to be shared, per Microsoft's explain text, so it is less complete than it sounds.
- The change is not live until the Print Spooler service restarts; the tweak does not restart it for you and does not declare a reboot.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the ADMX `supportedOn` is `SUPPORTED_WindowsNET`), every edition.
- **Takes effect**: after the Print Spooler service restarts (or the next reboot).
- **Reverting**: "Client connections allowed" deletes both values, restoring the shipped accept-connections behaviour after the next spooler restart. Selecting System Default restores the snapshot values.

#### Interactions
- `services:disable_print_spooler` disables the spooler entirely, which is stronger and makes this tweak moot.
- [Restrict printer-driver install to admins](#restrict-printer-driver-install-to-admins) covers the driver-installation half of PrintNightmare; the two are complementary.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that `RegisterSpoolerRemoteRpcEndPoint` lives under the policy key `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers`, not `Control\Print`, and added the "already-shared printers stay shared" and spooler-restart details from the explain text.
- **Confidence**: Microsoft-documented: both policies were read verbatim from the shipped `Printing2.admx`, `Printing.admx` and `Printing2.adml` on 26100.
- **Reasoning**: The key was the one defect found; the value (`2` = do not accept) and the second value were confirmed correct. Neither value is elsewhere in the corpus, and the "middle option" framing between disabling the spooler and restricting drivers survived.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine that prints but never receives print jobs or shares printers, which is nearly every home and office PC, then restart the Print Spooler service. Skip it if this machine shares a printer with others. If you never print at all, disabling the Print Spooler service is stronger.

#### Sources
1. `C:\Windows\PolicyDefinitions\Printing2.admx` and `en-US\Printing2.adml` (26100), policy `RegisterSpoolerRemoteRpcEndPoint`, verbatim including the explain text (tier A, shipped ADMX)
2. `C:\Windows\PolicyDefinitions\Printing.admx` (26100), policy `ConfigureRpcAuthnLevelPrivacyEnabled` (tier A, shipped ADMX)
3. Policy CSP Printers, Microsoft's reference for the printer policies, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-printers (cited in the YAML Evidence)
4. Manage Windows print spooler RPC connection settings, https://learn.microsoft.com/en-us/troubleshoot/windows-server/printing/manage-windows-printer-rpc-connection-settings (cited in the YAML Evidence)
5. CIS Windows 11 v4.0.0 and the DISA STIG Point and Print rules (tier B)

### Filter the remote local-admin token

`remote_uac_token_filter` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Ensures a local administrator account connecting over the network gets a filtered token rather than full admin rights, removing the pass-the-hash payoff that many "fix my network shares" guides switch back on.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `local_account_token_filter` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System` → `LocalAccountTokenFilterPolicy` (REG_DWORD) |

| Option | `local_account_token_filter` |
|---|---|
| Filtered | `0` |
| Not filtered | `1` |

There is deliberately no `absent` option: an absent value behaves exactly as `0`, so an option that deleted the value would be labelled for a state it cannot reach. On a stock machine the value is absent, which matches neither option, so the tweak shows System Default even though the behaviour is "Filtered". Selecting System Default restores the snapshot captured before the first apply (including `absent` if the value did not exist).

#### How it works

Remote UAC restrictions apply to local (non-domain) accounts that are members of Administrators and log on over the network (SMB, WMI, remote MMC and similar). Microsoft's "User Account Control and remote restrictions" article defines the value:

| Value | Microsoft's description |
|---|---|
| `0` | Builds a filtered token. It's the default value. The administrator credentials are removed. |
| `1` | Builds an elevated token. |

With `1`, a local administrator connecting over the network receives a full unfiltered admin token, which is exactly what pass-the-hash lateral movement needs. With `0`, the network logon gets a filtered token, so remote `C$` access, remote WMI and remote MMC with a local admin account fail. Domain accounts are unaffected.

The value is absent by default, but it is set to `1` by many "fix my network shares" guides, some remote-support tools and several popular tweak scripts; the research found it set to `1` on its own machine. That is why the tweak pins `0` rather than trusting the default. It is a direct registry setting, not a Group Policy administrative template: it appears in none of the 218 shipped ADMX files.

`FilterAdministratorToken`, in the same key, is a different setting: it controls Admin Approval Mode for the built-in Administrator account (see [Apply UAC to built-in Administrator](#apply-uac-to-built-in-administrator)).

#### Benefits
- Removes a pass-the-hash target: a stolen local admin hash no longer buys full remote admin rights.
- Pins a state other software commonly changes.
- Value `0` is Microsoft's own default behaviour.

#### Drawbacks
- Breaks remote administration with local accounts: remote `C$`, remote WMI and remote MMC with a local admin account fail. That is the intended effect, but it is real.
- No visible change on a clean machine, where `0` is already the effective behaviour.
- The other option, "Not filtered", writes `1`, which hands remote local admins a full token: the exact state this tweak exists to undo.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, every edition; local accounts only.
- **Takes effect**: immediately for new network logons, no reboot.
- **Reverting**: selecting System Default restores the value captured in the snapshot (usually `absent`, which behaves as `0`). If the value was `1` before you applied the tweak, reverting puts `1` back.

#### Interactions
- [Apply UAC to built-in Administrator](#apply-uac-to-built-in-administrator) writes `FilterAdministratorToken` in the same key; the research lists it, [Raise UAC to always notify](#raise-uac-to-always-notify) and this tweak as a UAC merge candidate, and warns that the two similar names are easy to confuse.
- [Disable administrative shares (C$, ADMIN$)](#disable-administrative-shares-c-admin) removes the shares a remote local admin would use.
- [Restrict remote SAM calls to administrators](#restrict-remote-sam-calls-to-administrators) is another control against remote enumeration and lateral movement.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting; the value, semantics and default match Microsoft exactly.
- **Confidence**: Microsoft-documented: the UAC remote-restrictions article gives both values and states that `0` is the default.
- **Reasoning**: The "already the default" objection was attacked and survived: the point is that on real machines the value is frequently `1`. The risk statement (breaks remote `C$`, WMI and MMC with local accounts) was confirmed accurate. The research recommended a delete-to-revert; the shipped tweak instead offers `0` and `1` and returns to the pre-apply state through the snapshot, because an `absent` option would be indistinguishable from `0`.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply "Filtered" on any machine that is not administered remotely with a local admin account, which is almost every home PC, and especially if you have ever followed a guide to make network shares work. Choose "Not filtered" only if you knowingly manage this machine remotely with a local administrator account.

#### Sources
1. User Account Control and remote restrictions, the UAC remote settings table with both values and the default, https://learn.microsoft.com/en-us/troubleshoot/windows-server/windows-security/user-account-control-and-remote-restriction (tier A)
2. DISA STIG for Windows 11 V2R2 ("Local administrator accounts must have their privileged token filtered") and CIS Windows 11 v4.0.0, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
3. All-ADMX scan across the 218 shipped `.admx` files on 26100, negative result (tier A); `src-tauri/tweaks/security.yaml`, `filter_admin_token`, near-miss evidence (corpus evidence)

### Enable SEHOP

`enable_sehop` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Pins Structured Exception Handling Overwrite Protection (SEHOP) on with an explicit, auditable value, so it cannot be quietly turned off.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `disable_exception_chain_validation` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel` → `DisableExceptionChainValidation` (REG_DWORD) |

| Option | `disable_exception_chain_validation` |
|---|---|
| Enabled | `0` |
| Not configured | absent |

The option labelled "Not configured" deletes the value; it does not turn SEHOP off. With the value absent, SEHOP is on for 64-bit Windows, exactly as with `0`. System Default is shown when the value is present with anything other than `0` (for example `1`, which does disable SEHOP); selecting it restores the snapshot captured before the first apply. Stock Windows has no value, which matches "Not configured".

#### How it works

SEHOP validates a thread's structured exception handler chain before dispatching an exception, so an attacker who overwrites a handler pointer on the stack cannot use it to redirect execution. The value name is inverted: `DisableExceptionChainValidation` = `0` means SEHOP is enabled. The kernel reads it at boot.

On x64 the practical effect is nil. 64-bit processes do not use an SEH chain at all; exception handling on x64 is table-driven through `.pdata` and `.xdata`. SEHOP therefore only matters for 32-bit processes, and it is already on for them. The only change this tweak makes is that the setting becomes an explicit registry value that a baseline check can see.

It is a direct registry write per the DISA STIG check text, not a Group Policy: it appears in none of the 218 shipped ADMX files. Microsoft's original SEHOP KB (956607) is retired and the current Learn troubleshooting URL returned 404 during the research.

#### Benefits
- With the value written, SEHOP cannot be turned off without the change being visible.
- Auditable: an explicit STIG check with a specific value.
- No measurable performance or compatibility cost.

#### Drawbacks
- No behavioural change on x64: it only affects 32-bit processes, where SEHOP is already on.
- Needs a reboot, because the value is read at boot.
- Not Microsoft-documented on today's web: the control rests on the DISA STIG check text plus community sources.
- "Not configured" deletes the value, which leaves SEHOP on; no option turns SEHOP off.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build, x64.
- **Takes effect**: after a reboot.
- **Reverting**: "Not configured" deletes the value, which behaves the same as `0` on a healthy machine. Selecting System Default restores the snapshot value. Either takes effect at the next boot.

#### Interactions
- None known in this corpus.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting; the research required the copy not to claim any behavioural change on x64.
- **Confidence**: Community-corroborated: the DISA STIG check text (tier B) plus long-standing community corroboration such as privacy.sexy (tier C); no reachable Microsoft page.
- **Reasoning**: The inverted name was checked and is handled correctly (`0` enables). The benefit was attacked: x64 does not use SEH chains, so the delta is an auditable value and nothing else. It survives inclusion because a real control exists and the user gets to pin it. Open question: no tier A source is currently reachable.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want a baseline-clean machine that passes the STIG check, or you run 32-bit software and want the setting explicit. If you are looking for a behavioural improvement, this is not one; skip it and save the reboot.

#### Sources
1. DISA STIG for Windows 11 V2R2, "Structured Exception Handling Overwrite Protection (SEHOP) must be enabled", establishes the key, value and hardened value, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
2. All-ADMX scan across the 218 shipped `.admx` files on 26100, negative result: a direct registry write, not a policy (tier A)
3. privacy.sexy, SEHOP script, corroboration, https://privacy.sexy/ (tier C)

### Block PKU2U online identities

`disable_pku2u_online_id` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Refuses peer-to-peer authentication requests that use online identities instead of an account on this PC. On a standalone client this really turns something off.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `pku2u_allow_online_id` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\pku2u` → `AllowOnlineID` (REG_DWORD) |

| Option | `pku2u_allow_online_id` |
|---|---|
| Blocked | `0` |
| Allowed | absent |

System Default is shown when the value is present with anything other than `0` (for example an explicit `1`); selecting it restores the snapshot captured before the first apply. Stock Windows has no value, which matches "Allowed"; on a standalone Windows 11 client the effective default is enabled.

#### How it works

PKU2U (Public Key Cryptography Based User-to-User) is a security protocol that lets two machines that share no domain authenticate each other using online identities, without either one holding an account for the other. The policy "Network security: Allow PKU2U authentication requests to this computer to use online identities" controls whether this PC accepts such requests. With `AllowOnlineID` = 0 it refuses them.

Microsoft states the policy "is enabled by default in Windows 10, Version 1607, and later". Its default-values table gives "Member server effective default settings: Disabled" and "Domain controller effective default settings: Disabled", and the "enabled by default in 1607 and later" statement is the one that applies to a standalone client. So on a workgroup Windows 11 PC this tweak changes behaviour rather than pinning a default. Microsoft's Potential impact section: "Some roles/features (such as Failover Clustering) don't utilize a domain account for its PKU2U authentication and will cease to function properly when disabling this policy."

It is a Security Options setting: it appears in none of the 218 shipped ADMX files. Microsoft documents the policy and its semantics but not the registry location, which comes from the DISA STIG and CIS check text.

#### Benefits
- A real change, not a pin: Microsoft documents the policy as enabled by default on a standalone client.
- Closes a domainless authentication path: no account, no shared directory, still authenticates.
- A standard baseline control (DISA STIG and CIS).

#### Drawbacks
- Workgroup device-to-device authentication is exactly what PKU2U is for, and a consumer machine is usually in a workgroup, so peer-to-peer features that rely on it can break.
- Microsoft names Failover Clustering as a feature that "will cease to function properly".
- The registry path is benchmark-sourced, not Microsoft-documented.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the policy exists from Windows 10 1607). Matters most on standalone and workgroup PCs; domain-joined member machines already default to Disabled.
- **Takes effect**: immediately, no reboot.
- **Reverting**: "Allowed" deletes the value, restoring the default enabled behaviour on a client. Selecting System Default restores the snapshot value.

#### Interactions
- None known in this corpus.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research corrected the default: on a standalone client the policy is effectively enabled (Microsoft: "enabled by default in Windows 10, Version 1607, and later"), so blocking it is a real behavioural change with real risk.
- **Confidence**: Microsoft-documented for the policy, its default and its impact; the registry path is benchmark-sourced (tier B).
- **Reasoning**: The proposal's "default already matches" claim was refuted by Microsoft's page, which raised the risk level from low to medium and made the peer-to-peer breakage explicit. The mechanism (key, value, type, hardened `0`) survived, and an all-ADMX scan confirmed it is a Security Options setting.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine that never authenticates peer-to-peer with another non-domain PC. If you share files or use device-to-device features across a workgroup, leave it alone: this is not the free pin it looks like.

#### Sources
1. Network security: Allow PKU2U authentication requests to this computer to use online identities, the default-values table, the "enabled by default in Windows 10, Version 1607, and later" statement and the Potential impact section, https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-security-allow-pku2u-authentication-requests-to-this-computer-to-use-online-identities (tier A)
2. DISA STIG for Windows 11 V2R2, PKU2U rule, the registry path, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
3. All-ADMX scan across the 218 shipped `.admx` files on 26100, negative result (tier A)

### Do not index encrypted files

`no_index_encrypted_files` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Keeps the contents of EFS-encrypted files out of the Windows Search index, which is an ordinary unencrypted database.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `allow_indexing_encrypted` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search` → `AllowIndexingEncryptedStoresOrItems` (REG_DWORD) |

| Option | `allow_indexing_encrypted` |
|---|---|
| Not indexed | `0` |
| Not configured | absent |

The option labelled "Not configured" deletes the policy value, which hands control back to the Control Panel setting; by default that setting does not index encrypted content either, so "Not configured" does not by itself turn indexing of encrypted files on. System Default is shown when the value is present with anything other than `0` (an explicit `1`); selecting it restores the snapshot captured before the first apply. Stock Windows has no value, which matches "Not configured".

#### How it works

The shipped `Search.admx` (26100) defines policy `AllowIndexingEncryptedStoresOrItems` ("Allow indexing of encrypted files"), class Machine, key `SOFTWARE\Policies\Microsoft\Windows\Windows Search`, enabled `1`, disabled `0`, supported from Vista (`VistaOr4`). `Search.admx` ships as UTF-16LE while almost every other shipped ADMX is UTF-8, which is why a naive text search misses it. With the policy at `0`, Windows Search does not extract and store content from EFS-encrypted files, so their plaintext never lands in the index database, and the policy overrides the Control Panel setting.

The shipped `Search.adml` explain text: "This policy setting is not configured by default. If you do not configure this policy setting, the local setting, configured through Control Panel, will be used. By default, the Control Panel setting is set to not index encrypted content." It also warns that enabling or disabling this setting causes the index to be rebuilt completely.

Only EFS (per-file) encryption is involved; BitLocker whole-volume encryption is unrelated.

#### Benefits
- No plaintext copy of encrypted file content in the index database.
- Policy-backed and enforced: the value overrides whatever the Control Panel setting says.
- A standard baseline control (DISA STIG and CIS).

#### Drawbacks
- Changing the setting (either way) rebuilds the whole search index, a period of heavy disk and CPU activity.
- Encrypted files stop appearing in search results; you find them by browsing.
- Already the default behaviour: the Control Panel default is not to index encrypted content, so this pins rather than changes.
- Moot without an index: if the Windows Search service is disabled, this value does nothing.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the ADMX supports Vista and later).
- **Takes effect**: immediately, followed by a full index rebuild.
- **Reverting**: "Not configured" deletes the value, returning control to the Control Panel setting, and triggers another full rebuild. Selecting System Default restores the snapshot value.

#### Interactions
- `performance:disable_search_indexing` disables the Windows Search service (`WSearch`); with it applied there is no index and this tweak is moot.
- [Prevent automatic device encryption](#prevent-automatic-device-encryption) concerns BitLocker, which this tweak does not involve.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the policy is in the shipped ADMX (`Search.admx` is UTF-16LE), so it is Microsoft-documented rather than a STIG-only direct write, and added the full-rebuild warning from the explain text.
- **Confidence**: Microsoft-documented: the policy and its explain text were read verbatim from the shipped `Search.admx` and `Search.adml` on 26100.
- **Reasoning**: The correction upgraded the entry: the gap proposal had said the value was not in any ADMX, which the pass traced to the UTF-16LE encoding trap. Key, value, type, hardened `0`, delete-to-revert and the not-indexing default all survived, and the no-op interaction with disabling Windows Search was confirmed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you use EFS at all: it guarantees encrypted content never reaches the index. If you do not use EFS, or you have turned off the Windows Search service, skip it: the index rebuild costs more than the pin is worth.

#### Sources
1. `C:\Windows\PolicyDefinitions\Search.admx` (26100, UTF-16LE), policy `AllowIndexingEncryptedStoresOrItems`, verbatim, and `en-US\Search.adml` `ExplainAllowIndexingEncryptedStoresOrItems`, establishes the key, values, default and the rebuild warning (tier A, shipped ADMX)
2. Policy CSP Search, Microsoft's reference for the Search policies, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-search (cited in the YAML Evidence)
3. DISA STIG for Windows 11 V2R2 ("Indexing of encrypted files must be turned off") and CIS Windows 11 v4.0.0, https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
4. `src-tauri/tweaks/performance.yaml`, `disable_search_indexing`, no-op interaction (corpus evidence)

### Disallow AutoPlay for non-volume devices

`autoplay_non_volume` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops AutoPlay acting on phones, cameras and other MTP or PTP devices that never get a drive letter, closing the AutoPlay path the drive-type controls do not reach.**

#### What it changes

| Effect | Kind | Target |
|---|---|---|
| `no_autoplay_non_volume_machine` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Explorer` → `NoAutoplayfornonVolume` (REG_DWORD) |
| `no_autoplay_non_volume_user` | registry | `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` → `NoAutoplayfornonVolume` (REG_DWORD) |

| Option | `no_autoplay_non_volume_machine` | `no_autoplay_non_volume_user` |
|---|---|---|
| Disallowed | `1` | `1` |
| Allowed | absent | absent |

System Default is shown when the two values match neither option (for example only one hive set); selecting it restores the snapshot captured before the first apply. Stock Windows has neither value, which matches "Allowed".

#### How it works

The shipped `AutoPlay.admx` (26100) defines policy `NoAutoplayfornonVolume` ("Disallow Autoplay for non-volume devices"), class Both, key `Software\Policies\Microsoft\Windows\Explorer`, enabled `1`, supported from Windows 7. Non-volume devices are MTP and PTP devices such as phones, cameras and media players: they connect as media-transfer devices rather than as drives, so the drive-type bitmask (`NoDriveTypeAutoRun`) and `NoAutorun` do not cover them. With the policy enabled, Explorer does not raise the AutoPlay prompt or default action when such a device is connected.

Because the class is Both and MTP handling is driven from the shell, the per-user policy is the one users actually have in practice, so the tweak writes and reverts both hives. The HKCU write lands in the hive of the account the app runs as when elevated; if you elevate MagicX Toolbox with a different administrator account, the per-user copy goes to that account and the machine copy covers everyone else.

The DISA STIG has three rules in this AutoPlay family; [Disable AutoRun/AutoPlay on all drives](#disable-autorunautoplay-on-all-drives) implements the other two.

#### Benefits
- Closes the last AutoPlay gap: the drive-type controls do not reach non-volume devices.
- Covers both hives, including the per-user policy that governs the shell.
- Completes the three-rule STIG family together with the AutoRun tweak.

#### Drawbacks
- Camera and phone prompts disappear: plugging in a phone no longer offers to import photos.
- You open the device in File Explorer yourself instead.
- A very small attack-surface gain: MTP devices cannot autorun code the way removable drives historically could.

#### Applies to, takes effect, reverting
- **Applies to**: every supported build (the ADMX supports Windows 7 and later).
- **Takes effect**: after sign-out or an Explorer restart, no reboot.
- **Reverting**: "Allowed" deletes both values, restoring the shipped AutoPlay behaviour. Selecting System Default restores the snapshot values.

#### Interactions
- [Disable AutoRun/AutoPlay on all drives](#disable-autorunautoplay-on-all-drives) writes `NoAutorun` and `NoDriveTypeAutoRun` for drive-letter volumes; the two together cover every AutoPlay path. The research recommends folding this value into that tweak as a third effect.

#### Validation
- **Verdict**: VERIFIED. Nothing in the mechanism needed correcting; the research made the HKCU write mandatory because the policy is class Both.
- **Confidence**: Microsoft-documented: the policy was read verbatim from the shipped `AutoPlay.admx` on 26100.
- **Reasoning**: Key, value name, type, polarity, hardened `1` and `supportedOn` all survived. The dedupe check confirmed the AutoRun tweak does not write this value.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it alongside the AutoRun tweak so AutoPlay is fully off. Skip it if you regularly import photos from a phone or camera and want the prompt.

#### Sources
1. `C:\Windows\PolicyDefinitions\AutoPlay.admx` (26100), policy `NoAutoplayfornonVolume`, verbatim, establishes the key, class Both, polarity and `supportedOn` (tier A, shipped ADMX)
2. Autoplay Policy CSP, Microsoft's reference for the AutoPlay policies, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-autoplay (cited in the YAML Evidence)
3. DISA STIG for Windows 11 V2R2 ("Autoplay must be turned off for non-volume devices"), https://www.stigviewer.com/stigs/microsoft_windows_11 (tier B)
4. `src-tauri/tweaks/security.yaml`, `disable_autorun`, dedupe evidence (corpus evidence)

## Considered and not shipped

The July 2026 gap-verification passes proposed four more tweaks, recorded in the security research, that the research rejected. None is shipped in this category.

### Compatibility Appraiser tasks (`task_compatibility_appraiser`)

**What it was**: a tweak disabling five scheduled tasks under `\Microsoft\Windows\Application Experience\` (the Compatibility Appraiser family) as a task-level telemetry control.

**Why it is not shipped**: it duplicates the Privacy category. `privacy:disable_compat_appraiser` already sets the `AITEnable` and `DisableInventory` policy values under `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat` and disables the `Microsoft Compatibility Appraiser`, `Microsoft Compatibility Appraiser Exp` and `StartupAppTask` tasks. A second tweak over the same tasks would let the app hold two contradictory states for one task. The research recommended adding the three genuinely new tasks (`Microsoft Compatibility Appraiser Exp`, `PcaPatchDbTask`, `MareBackup`) to the Privacy tweak instead (the shipped Privacy tweak includes `Microsoft Compatibility Appraiser Exp`; `PcaPatchDbTask` and `MareBackup` are not included), and noted that `SdbinstMergeDbTask` in the same folder is application-compatibility shim maintenance, not telemetry, and should stay enabled. See the Privacy category page.

**Sources**: `src-tauri/tweaks/privacy.yaml` (`disable_compat_appraiser`); enumeration of `C:\Windows\System32\Tasks\Microsoft\Windows\Application Experience` on build 26100 (tier A for task presence), both recorded in `_verify-gaps-b-medlow.md`, proposal 16.

### Customer Experience Improvement Program tasks (`task_ceip`)

**What it was**: a tweak disabling the CEIP scheduled tasks (`Consolidator`, `UsbCeip` and `\Microsoft\Windows\PI\Sqm-Tasks`).

**Why it is not shipped**: two of the three tasks are already disabled by `privacy:disable_ceip_tasks`, alongside the `CEIPEnable` policy under `HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows`. The research recommended adding only `\Microsoft\Windows\PI\Sqm-Tasks` to that tweak, and warned that `\Microsoft\Windows\PI\Secure-Boot-Update` in the same folder delivers Secure Boot DBX revocation updates and must never be touched. See the Privacy category page.

**Sources**: `src-tauri/tweaks/privacy.yaml`; enumeration of the CEIP and PI task folders on build 26100 (tier A for task presence), recorded in `_verify-gaps-b-medlow.md`, proposal 17.

### MSS network stack hardening (`mss_network_stack_hardening`)

**What it was**: a bundle of five legacy "MSS" TCP/IP hardening values from the Microsoft Security Compliance Toolkit (including `EnableICMPRedirect`, `PerformRouterDiscovery` and `NoNameReleaseOnDemand`), four of which are individual DISA STIG rules. They are not in any ADMX shipped with Windows; they arrive with the separate `MSS-legacy.admx`.

**Why it is not shipped**: two reasons. First, no stock default was established for any of the five values from a clean image or Microsoft documentation; `EnableICMPRedirect` and `PerformRouterDiscovery` are TCP/IP stack parameters with documented non-zero effective behavior, so a revert that deletes them could leave a machine in a state it never shipped in. Second, there is little benefit left: on a consumer machine behind NAT the exposure to source-routing and ICMP-redirect attacks is close to nil, and `NoNameReleaseOnDemand` only matters where NetBIOS is in use, which `network:disable_netbios_tcpip` already turns off. The research left room to revive it as an explicitly labeled CIS and STIG benchmark-alignment bundle once each stock default is established.

**Sources**: scan of every shipped ADMX on build 26100 (tier A, negative result); DISA STIG for Windows 11 V2R2, four rules (tier B); `src-tauri/tweaks/network.yaml` (`disable_netbios_tcpip`); all recorded in `_verify-gaps-b-medlow.md`, proposal 36.

### Disable NTFS 8.3 short names (`ntfs_disable_8dot3`)

**What it was**: a performance tweak setting `HKLM\System\CurrentControlSet\Control\FileSystem\NtfsDisable8dot3NameCreation` to `1` (disable 8.3 short-name creation on all volumes). The mechanism is real: `fsutil 8dot3name` writes that value, `2` means per-volume, and the shipped `FileSys.admx` policy `ShortNameCreationSettings` uses a different key (`System\CurrentControlSet\Policies`) with the same enumeration (`0` enable on all volumes, `1` disable on all volumes, `2` per volume, `3` disable on all data volumes). Existing short names are not removed.

**Why it is not shipped**: its only justification was a claim that Microsoft's file-server performance tuning guidance says disabling 8.3 names speeds file creation in large directories. The cited page was fetched and full-text scanned during the research and contains no mention of "8dot3", "8.3" or "ShortName", and no measurement on a client workload at build 26100 was found. Against no measured benefit stand real compatibility costs: 16-bit installers and applications that hardcode short paths, and the `fsutil` documentation's own warning that registry entries pointing at 8.3 names can cause "unexpected application failures, including the inability to uninstall an application". The Performance research adds that the real stock default on 26100 is `2` (per volume), not the `0` or absent value the proposal assumed, so a delete-to-revert would itself be a hazard.

**Sources**: Windows Server file-server performance tuning, the page cited for the claim and found not to contain it, https://learn.microsoft.com/en-us/windows-server/administration/performance-tuning/role/file-server/ ; `fsutil 8dot3name` documentation and the shipped `FileSys.admx` on build 26100 (both as recorded in `_verify-gaps-b-medlow.md`, proposal 37).