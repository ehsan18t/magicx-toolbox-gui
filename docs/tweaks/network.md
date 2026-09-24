# Network & Power tweaks

This category covers network hardening (encrypted DNS, the three broadcast name-resolution protocols LLMNR, NetBIOS over TCP/IP and mDNS, WPAD, IPv6 tunnels, Internet Connection Sharing, firewall logging) and power and sleep behaviour (NIC and USB power saving, hibernation, wake timers, Modern Standby). Every tweak here is machine-wide and needs administrator rights: most write HKLM values (policy or service keys), and the rest run PowerShell or `powercfg` actions that record the prior state before changing it, so a revert restores what was there. The primary platform is Windows 11 24H2 (build 26100) and newer; Windows 10 IoT Enterprise LTSC 2021 (build 19044) is a secondary target, and the two DNS-over-HTTPS tweaks are gated to Windows 11 only. The Windows Update controls researched alongside these tweaks in July 2026 are on their own page, [Windows Update](windows_update.md).

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Enable DNS over HTTPS auto-upgrade](#enable-dns-over-https-auto-upgrade) | `dns_over_https` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Require encrypted DNS](#require-encrypted-dns) | `require_doh` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable LLMNR](#disable-llmnr) | `disable_llmnr` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable NetBIOS over TCP/IP](#disable-netbios-over-tcpip) | `disable_netbios_tcpip` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable mDNS](#disable-mdns) | `disable_mdns` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable WPAD auto-proxy discovery](#disable-wpad-auto-proxy-discovery) | `disable_wpad` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable IPv6 transition technologies](#disable-ipv6-transition-technologies) | `disable_ipv6_transition` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Internet Connection Sharing](#disable-internet-connection-sharing) | `disable_internet_connection_sharing` | Switch | medium | admin | yes | VERIFIED |
| [Firewall logging and local policy merge](#firewall-logging-and-local-policy-merge) | `firewall_logging_and_merge` | Dropdown (3 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable NIC power management](#disable-nic-power-management) | `disable_nic_power_management` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable hibernation](#disable-hibernation) | `disable_hibernation` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable USB selective suspend](#disable-usb-selective-suspend) | `disable_usb_selective_suspend` | Switch (2 options) | medium | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable wake timers](#disable-wake-timers) | `disable_wake_timers` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Modern Standby (force S3)](#disable-modern-standby-force-s3) | `disable_modern_standby` | Switch (2 options) | high | admin | yes | VERIFIED |

A note on the Control column: in this app one authored option renders as a toggle (System Default or that option) and two or more render as a dropdown whose first entry is the computed System Default status. System Default is never written; it is what the app shows when the live machine matches none of the authored options, and selecting it walks back through the tweak's snapshot (Restore Snapshot) rather than writing a guessed "default" value.

## Tweaks

### Enable DNS over HTTPS auto-upgrade

`dns_over_https` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Upgrades your DNS lookups to encrypted DNS over HTTPS, but only when the resolver you already use is one Windows recognises as DoH-capable.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `auto_doh` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters`, value `EnableAutoDoh`, `REG_DWORD` |

| Option | `auto_doh` |
|---|---|
| Auto-upgrade to DoH | `2` |
| No auto-upgrade | `absent` (value deleted) |

System Default: shown only if `EnableAutoDoh` holds some value other than 2 (for example 1 or 0 written by another tool); selecting it restores the snapshot. Stock Windows has no `EnableAutoDoh` value, so an untouched machine reads as "No auto-upgrade".

#### How it works

`EnableAutoDoh` is read by the DNS Client service (`Dnscache`, implemented in `dnsrslvr.dll`). Microsoft published the key, the DWORD name and the value 2 in its May 2020 networking blog that introduced the Windows DoH client, and the July 2026 research confirmed the string `EnableAutoDoh` is still present in `dnsrslvr.dll` on build 26100.4061 alongside `DohInterfaceSettings` and `DohFlags`, so the service still reads it.

The value is an auto-upgrade switch, not an "enable encryption" switch. With it set to 2, the DNS client promotes a plaintext query to DNS over HTTPS only when the interface's configured resolver is already on Windows' known-DoH template list: Cloudflare 1.1.1.1 and 1.0.0.1, Google 8.8.8.8 and 8.8.4.4, Quad9 9.9.9.9 and 149.112.112.112, plus any server added with `Add-DnsClientDohServerAddress`. The tweak does not set a resolver. On a machine that uses the DNS server handed out by its router or ISP, the value is inert: nothing is encrypted, and nothing tells you so.

The same blog said the registry keys were "only for enabling DoH client testing on Insider builds" and that "when the DoH client is made available in general release builds, registry configuration of DoH will not be supported". Microsoft's shipping documentation for DoH (the Settings page "Preferred DNS encryption", the Group Policy "Configure DNS over HTTPS (DoH) name resolution", `netsh dns add encryption`, `Add-DnsClientDohServerAddress`, and the per-interface `Dnscache\InterfaceSpecificParameters\{GUID}\DohInterfaceSettings\Doh\{IP}` `DohFlags` value) never mentions `EnableAutoDoh`. So this is Microsoft-sourced but not Microsoft-supported: it works on 26100 today, with no guarantee it keeps working.

The value lives in the service's own parameters key, not a policy key, so there is no edition gate in the value itself. The tweak is gated to Windows 11 because no shipping Windows 10 release, including 22H2 and LTSC 2021, has the built-in DoH client.

#### Benefits
- Hides DNS lookups from your ISP and anyone on the network path, when the resolver is DoH-capable.
- Encrypted answers cannot be silently rewritten in transit.
- One machine-wide value covers every adapter; no per-adapter configuration.
- Falls back to plaintext rather than failing, so it cannot cut you off the network.

#### Drawbacks
- Does nothing on its own: with an ISP-assigned resolver, nothing is encrypted and the app still reports the option as applied.
- Unsupported by Microsoft: the blog that published the value said registry DoH configuration would not be supported in release builds.
- No feedback: nothing in Settings reflects the change, so you cannot see whether it took effect.
- Opportunistic only: if the upgrade is not possible, lookups go out in plaintext without warning.
- Can interfere with captive portals and split-horizon internal DNS on networks that depend on intercepting plaintext DNS.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, all editions. Hidden on Windows 10 (including LTSC 2021) by the `products: [11]` gate, because the DoH client does not exist there.
- **Takes effect**: after a reboot, when the DNS Client service starts and reads its parameters.
- **Reverting**: "No auto-upgrade" deletes the value, which is the stock state. Restore Snapshot puts back whatever value (or absence) was captured before the first apply.

#### Interactions
- [Require encrypted DNS](#require-encrypted-dns) (`require_doh`) is the enforcing, policy-based alternative. The research recommends picking one posture, not combining them: this one falls back to plaintext, `require_doh` fails resolution instead.
- [Disable mDNS](#disable-mdns) (`disable_mdns`) writes a different value (`EnableMDNS`) in the same `Dnscache\Parameters` key. No conflict.
- Either DoH tweak needs a DoH-capable resolver set on the adapter first (Settings, `Set-DnsClientServerAddress`, or the router).

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. Key, name, type, value and absent stock default are all correct; the correction was to what the value does: it only upgrades an already DoH-capable resolver, it sets no resolver, and Microsoft calls registry DoH configuration unsupported.
- **Confidence**: Microsoft-documented (the value comes verbatim from a Microsoft networking blog), plus a binary check on 26100.4061 showing the DNS Client still carries the value name.
- **Reasoning**: the adversarial pass attacked two claims: that this is "community-documented only" (wrong: it is Microsoft-sourced) and that it enables encryption (wrong: it only upgrades a resolver already on the known-DoH list). Both are reflected here. Open question: Microsoft could remove support without notice, since it never documented the value outside the Insider blog.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it only if you have already pointed your adapter at Cloudflare, Google, Quad9 or another server registered with `Add-DnsClientDohServerAddress`, and you want encryption where possible without risking a DNS outage. If you want encryption guaranteed, use [Require encrypted DNS](#require-encrypted-dns) instead. On an ISP resolver, skip it: it changes nothing.

#### Sources
1. Windows Insiders can now test DNS over HTTPS (Microsoft networking blog), gives the key, the `EnableAutoDoh` DWORD and value 2 verbatim, and the "registry configuration of DoH will not be supported" statement, https://techcommunity.microsoft.com/blog/networkingblog/windows-insiders-can-now-test-dns-over-https/1381282 (tier B)
2. Secure DNS Client over HTTPS (DoH), the supported per-server route and the known-DoH server list, https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support (tier A)
3. Add-DnsClientDohServerAddress, the supported way to register a DoH server with auto-upgrade, https://learn.microsoft.com/en-us/powershell/module/dnsclient/add-dnsclientdohserveraddress
4. Direct binary inspection of `C:\Windows\System32\dnsrslvr.dll` on build 26100.4061 (July 2026 research): `EnableAutoDoh`, `DohInterfaceSettings` and `DohFlags` present, confirming the value is still read (primary measurement, tier A for existence only)
5. Enabling DNS over HTTPS on Windows 11 (Windows OS Hub), independent corroboration of the known-DoH resolver list and the need to configure a DoH-capable server, https://woshub.com/enable-dns-over-https-windows/ (tier C)

### Require encrypted DNS

`require_doh` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: Windows 11 only (`products: [11]`) · Reversible: yes

**Forces DNS over HTTPS by policy: if the resolver cannot do encrypted DNS, name resolution fails instead of quietly falling back to plaintext.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `doh_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient`, value `DoHPolicy`, `REG_DWORD` |

| Option | `doh_policy` |
|---|---|
| Require encryption | `3` |
| Encryption optional | `absent` (value deleted) |

System Default: shown when `DoHPolicy` holds 1 ("Prohibit encryption") or 2 ("Allow encryption"), for example from Group Policy or MDM; selecting it restores the snapshot. Stock Windows has no `DoHPolicy` value, so an untouched machine reads as "Encryption optional".

#### How it works

`DoHPolicy` is the registry backing of the first-party Group Policy "Configure DNS over HTTPS (DoH) name resolution" (policy `DNS_Doh` in the shipped `DnsClient.admx`, under Computer Configuration > Administrative Templates > Network > DNS Client). The DNS Client service reads it. The shipped 26100 ADMX and ADML define three enum values: 1 = "Prohibit encryption", 2 = "Allow encryption", 3 = "Require encryption". This tweak writes 3.

With "Require encryption", the DNS client sends queries only over DoH. If the configured resolver does not speak DoH (it is not on the known-DoH list and was not registered with `Add-DnsClientDohServerAddress`), resolution fails rather than falling back to plaintext port 53. That is the guarantee [Enable DNS over HTTPS auto-upgrade](#enable-dns-over-https-auto-upgrade) cannot give.

The `DNS_Doh` policy has three elements, all marked `required="true"` in the ADMX: `DoHPolicy` (this one), `DohPolicySetting` (0 allow DoH, 1 block DoH) and `DotPolicySetting` (0 allow DNS over TLS, 1 block DoT). The tweak writes only `DoHPolicy`. The DNS client honours it on its own, but Group Policy tooling (for example `gpedit.msc` or RSoP) may display the policy as partially configured. DNS over TLS is already present in the 26100 ADMX.

The ADMX declares the policy for Windows 10 20H2 / Server 20H2 and later, which is wider than the tweak's Windows 11 gate. The gate is deliberate: no shipping Windows 10 release has the DoH client, so the policy would have no practical effect there. Because the value lives under `SOFTWARE\Policies`, it is a lock rather than a preference.

#### Benefits
- A real guarantee: no silent plaintext fallback.
- Microsoft-documented, ADMX-backed policy, not an undocumented registry value.
- Machine-wide across every interface.

#### Drawbacks
- Total name-resolution failure if misconfigured: with a resolver that does not speak DoH, nothing resolves and the machine looks offline.
- Breaks captive portals (hotel and airport Wi-Fi sign-in pages rely on plaintext DNS interception).
- Wrong for domain-joined machines: Microsoft warns against requiring DoH where Windows Server DNS answers internal names, because Windows Server DNS does not serve DoH.
- Does not provide a server: you must configure a DoH-capable resolver first.
- Writes only one of the policy's three required elements.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer. The research does not establish edition behaviour on Home; the Policy CSP documents the DNS Client policy area for Pro and above. Hidden on Windows 10 by the gate.
- **Takes effect**: after a reboot, or after restarting the DNS Client (`Dnscache`) service.
- **Reverting**: "Encryption optional" deletes the value, which is the stock state. Restore Snapshot puts back the captured prior value.

#### Interactions
- [Enable DNS over HTTPS auto-upgrade](#enable-dns-over-https-auto-upgrade) is the opportunistic alternative. The research's recommendation is one posture or the other, not both: they disagree about what happens when encryption is impossible.
- [Disable LLMNR](#disable-llmnr) (`EnableMulticast`) and [Disable mDNS](#disable-mdns) (`EnableMDNS`) write other values in the same `DNSClient` policy key. No conflict.
- A Group Policy or MDM `DNS_Doh` setting will overwrite this value on the next policy refresh.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The correction concerned the enum: 1 prohibits, 2 allows, 3 requires encryption (a proposal had 1 and 2 swapped); 3 is the right value for "require".
- **Confidence**: Microsoft-documented (shipped 26100 `DnsClient.admx` and ADML verbatim, plus Microsoft's DoH client documentation).
- **Reasoning**: the ADMX and ADML strings settle the value mapping directly. The adversarial pass also flagged the three required elements and the applicability mismatch (policy declared for Windows 10 20H2, tweak gated to Windows 11); both are recorded above as deliberate. No open question on the mechanism.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
The right choice for a privacy-motivated user on a standalone Windows 11 machine who has already set a DoH-capable resolver and wants encryption guaranteed. Do not apply it on a domain-joined machine, and think twice on a laptop that regularly signs in through captive portals, since those portals will fail until you revert.

#### Sources
1. `C:\Windows\PolicyDefinitions\DnsClient.admx` and `en-US\DnsClient.adml` on build 26100, policy `DNS_Doh`: the three `DoHPolicy` values and their names, the `DohPolicySetting` and `DotPolicySetting` required elements, `supportedOn` Windows 10 20H2 (tier A, shipped ADMX)
2. Secure DNS Client over HTTPS (DoH), the DoH client, the known-server list, and the warning about requiring DoH on domain-joined machines, https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support (tier A)
3. Policy CSP - ADMX_DnsClient, the DNS Client policy surface, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient (tier A)
4. Add-DnsClientDohServerAddress, registering a DoH server so the policy has something to use, https://learn.microsoft.com/en-us/powershell/module/dnsclient/add-dnsclientdohserveraddress

### Disable LLMNR

`disable_llmnr` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off LLMNR, the multicast name-resolution fallback that credential-theft tools like Responder answer to harvest NTLM hashes.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `enable_multicast` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient`, value `EnableMulticast`, `REG_DWORD` |

| Option | `enable_multicast` |
|---|---|
| Disabled | `0` |
| Enabled | `absent` (value deleted) |

System Default: shown when `EnableMulticast` holds a value other than 0 (for example 1 from a policy); selecting it restores the snapshot. Stock Windows has no value (LLMNR on), so an untouched machine reads as "Enabled".

#### How it works

This is the registry form of the Group Policy "Turn off multicast name resolution" (`Turn_Off_Multicast` in `DnsClient.admx`, Computer Configuration > Administrative Templates > Network > DNS Client). The ADMX declares it `class="Machine"`, and the tweak writes HKLM, so the hive matches (confirmed by the policy-hive audit). Microsoft: "If you enable this policy setting, LLMNR will be disabled on all available network adapters on the DNS client. If you disable this policy setting, or you don't configure this policy setting, LLMNR will be enabled on all available network adapters."

LLMNR (Link-Local Multicast Name Resolution) is what Windows falls back to when DNS cannot resolve a name: it multicasts "who is FILESRV?" to the local subnet and trusts whoever answers. An attacker on the same network answers every such query, the victim then tries to authenticate to the attacker, and the attacker captures NTLM challenge-response material. That is Responder's core technique, and it is why CIS control 18.5.4.1 recommends this policy.

The DNS Client has read `EnableMulticast` since Windows Vista. The Policy CSP applicability table (Windows 10 2004 with KB5005101 and later, Windows 11 21H2 and later; Pro, Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC) reflects when the MDM surface was added, not when the registry value started working. On Home the DNS client reads the value too, but Microsoft does not document that.

#### Benefits
- Closes LLMNR poisoning, the highest-yield, lowest-effort attack on a flat Windows network.
- Stops Responder-style NTLM hash capture over LLMNR.
- Benchmark-backed (CIS 18.5.4.1).
- A policy value, so Windows feature updates do not reset it.

#### Drawbacks
- On a network with no DNS server, single-label names that only LLMNR could resolve stop resolving (NetBIOS may still answer, unless you also disable it).
- Only one of three broadcast protocols: NetBIOS name service and mDNS are separate and need their own tweaks.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021. Documented for Pro, Education, Enterprise and IoT Enterprise; read on Home but undocumented there.
- **Takes effect**: after a reboot (or a `Dnscache` restart), when the DNS Client service re-reads the policy. Microsoft states no reboot requirement either way; the flag is the conservative choice.
- **Reverting**: "Enabled" deletes the value (the stock state). Restore Snapshot puts back the captured value.

#### Interactions
- Part of the name-resolution triad with [Disable NetBIOS over TCP/IP](#disable-netbios-over-tcpip) and [Disable mDNS](#disable-mdns); all three defend against the same poisoning technique. In cost order: LLMNR is nearly free, NetBIOS costs legacy device discovery, mDNS costs `.local` names, AirPrint printers and casting.
- Shares the `DNSClient` policy key with [Require encrypted DNS](#require-encrypted-dns) and [Disable mDNS](#disable-mdns) (different values, no conflict).
- A domain Group Policy setting for the same policy overrides this on refresh.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting: key, value name, type, polarity and absent stock default are all correct.
- **Confidence**: Microsoft-documented (Policy CSP and the shipped 26100 `DnsClient.admx`), with CIS as an independent benchmark source.
- **Reasoning**: the value is named for the feature and 0 turns it off; the ADMX, the CSP and CIS agree. The policy-hive audit confirmed the Machine class and HKLM write. The research recommended `requires_reboot: true` as the honest flag since the DNS Client reads the policy at service start; the tweak carries it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it. On any network with a working DNS server, which includes essentially every home router, the cost is zero and the security gain is real. Skip it only if you depend on short-name discovery on a network with no DNS at all.

#### Sources
1. Policy CSP - ADMX_DnsClient, `Turn_Off_Multicast`, the policy text and applicability, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient (tier A)
2. `C:\Windows\PolicyDefinitions\DnsClient.admx` on build 26100, policy `Turn_Off_Multicast`, `class="Machine"`, key `Software\Policies\Microsoft\Windows NT\DNSClient`, value `EnableMulticast` (tier A, shipped ADMX)
3. CIS Benchmark control 18.5.4.1 "Ensure 'Turn off multicast name resolution' is set to Enabled" (Tenable audit item), https://www.tenable.com/audits/items/CIS_DC_SERVER_2012_Level_1_v2.2.0.audit:f9584c0c934378405ec052736485f437 (tier B)

### Disable NetBIOS over TCP/IP

`disable_netbios_tcpip` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Shuts off the NetBIOS name service on every network adapter, removing the second broadcast name-resolution protocol that credential-theft tools spoof.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `NetbiosTcpip`, `REG_DWORD` (the app's own marker of the chosen option) |
| `netbios_off` | action (PowerShell, with `apply`, `undo` and `probe`) | writes `NetbiosOptions = 2` (`REG_DWORD`) to every `Tcpip_*` subkey of `HKLM\SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces`; records each interface's prior value under `HKLM\SOFTWARE\MagicXToolbox\ActionSnapshots\NetbiosTcpip` |

| Option | `state` | `netbios_off` |
|---|---|---|
| Disabled | `1` | run (apply) |
| Enabled | `0` | not run (the engine runs the action's `undo` if the probe reads it present) |

What the action does, precisely:

- **apply**: for each `Tcpip_*` interface subkey, reads the current `NetbiosOptions`, stores it under `ActionSnapshots\NetbiosTcpip` with the interface name as the value name (`-1` if the value was absent), then writes `NetbiosOptions = 2`. Stops on the first error.
- **undo**: for each `Tcpip_*` subkey, restores the stored value; if the stored value is `-1` or there is no stored value for that interface, deletes `NetbiosOptions`. Then deletes the `ActionSnapshots\NetbiosTcpip` key.
- **probe**: "applied" only if at least one `Tcpip_*` interface exists and every one of them reads exactly 2. Any interface that reads anything else (including an unreadable or absent value) reports "not applied".

System Default: shown whenever the machine matches neither option, which includes an untouched machine (the HKCU marker does not exist until you pick an option) and a machine where a new adapter has appeared since you applied (the probe then fails). Selecting it restores the snapshot. Stock Windows: every interface carries `NetbiosOptions = 0` ("use the DHCP server's setting"), measured on all seven interfaces of the research machine and matching Microsoft's documented default.

#### How it works

`NetbiosOptions` is Microsoft's documented per-interface NetBT mode selector: 0 = use the setting the DHCP server supplies (the default), 1 = NetBIOS over TCP/IP enabled, 2 = disabled. Writing 2 is the registry equivalent of choosing "Disable NetBIOS over TCP/IP" on the WINS tab of a connection's Advanced TCP/IP settings. NetBT reads it per interface; Microsoft states the change applies after `ipconfig /renew` on a DHCP interface and otherwise after a reboot.

NetBIOS Name Service (NBT-NS) resolves names by broadcast, so like LLMNR any host on the subnet can answer and redirect a victim's authentication attempt. Responder poisons NBT-NS and LLMNR with the same tooling.

There is no single machine-wide switch: the setting is per interface, which is why this is an action rather than a single registry effect. An adapter that did not exist at apply time (a new USB NIC, a VPN adapter, a Hyper-V virtual switch) gets a fresh subkey with the default and keeps NetBIOS. The probe catches this and the tweak drops to System Default, so the drift is visible; re-apply to cover the new adapter.

The applied-state marker is in HKCU (per user) while the change is machine-wide, so another Windows account sees the tweak as System Default even though NetBIOS is off for everyone. Because of the HKCU marker, the app's over-the-shoulder guard applies: if the app was elevated with a different account's credentials, the tweak is disabled rather than writing the wrong user's hive.

#### Benefits
- Closes NBT-NS poisoning, the twin of LLMNR poisoning.
- With LLMNR off too, both classic broadcast name-resolution holes are shut.
- Applied to every adapter present at apply time, not only the active one.
- The revert restores each interface's own prior value, including one an admin or unattend file deliberately set to 1.

#### Drawbacks
- Breaks NetBIOS-only discovery: very old NAS boxes, network printers and some line-of-business apps stop resolving by short name.
- Networks that still use WINS depend on NetBIOS.
- `net view` browsing stops where it relied on NetBIOS names.
- Adapters added later are not covered until you re-apply.
- On a DNS-less network with LLMNR also disabled, short-name discovery stops entirely.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions.
- **Takes effect**: after a reboot, or after `ipconfig /renew` on a DHCP interface.
- **Reverting**: "Enabled" runs the undo, which restores each interface's recorded value and deletes `NetbiosOptions` on any interface that had none or that the snapshot does not list. If the `ActionSnapshots\NetbiosTcpip` key is missing (for example deleted by hand), the undo deletes the value on every interface instead of restoring it; the research does not establish whether an absent value behaves exactly like 0.

#### Interactions
- The triad with [Disable LLMNR](#disable-llmnr) and [Disable mDNS](#disable-mdns); apply LLMNR first (cheapest), then this, then mDNS.
- SMB file sharing on port 445 using DNS names is unaffected.
- A DHCP server that pushes a NetBIOS setting only matters to interfaces set to 0; interfaces set to 2 ignore it.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The value and its meaning were right; the correction concerned the revert, which must restore each interface's own prior value rather than write a fixed 0. The shipped action does this, and the setting is per adapter so new adapters are not covered.
- **Confidence**: Microsoft-documented (three Microsoft Learn pages give the 0/1/2 semantics, the key path, the default and the renew-or-reboot behaviour), plus a live registry read of seven interfaces on 26100.
- **Reasoning**: the mechanism survived the adversarial pass unchanged; the attacks landed on the revert and on coverage of new adapters, both addressed above. The probe-fail-open audit asked that every path that does not prove the state exit non-zero; the shipped probe exits 1 when no interface is found and when any interface is not 2. Open question: whether a deleted `NetbiosOptions` is exactly equivalent to 0.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any modern network, ideally together with [Disable LLMNR](#disable-llmnr). Skip it if you still rely on genuinely old NetBIOS-only hardware or WINS. Re-apply after adding a new network adapter.

#### Sources
1. NetbiosOptions (Core Services), the 0/1/2 semantics and the default of 0, https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc776524(v=ws.10) (tier A)
2. TCP/IP and NBT configuration parameters, the key path and the `ipconfig /renew` versus reboot behaviour, https://learn.microsoft.com/en-us/troubleshoot/windows-client/networking/tcpip-and-nbt-configuration-parameters (tier A)
3. Unattend `NetbiosOptions` setting, independent confirmation of the value semantics, https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-netbt-interfaces-interface-netbiosoptions (tier A)
4. Direct registry read of `NetBT\Parameters\Interfaces` on Windows 11 24H2 build 26100 (July 2026 research): seven interfaces, all `NetbiosOptions = 0` (primary measurement)

### Disable mDNS

`disable_mdns` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off multicast DNS, the third broadcast name-resolution protocol, closing the last leg of the LLMNR and NetBIOS spoofing triad.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `mdns_policy` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient`, value `EnableMDNS`, `REG_DWORD` |
| `mdns_service` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters`, value `EnableMDNS`, `REG_DWORD` |

| Option | `mdns_policy` | `mdns_service` |
|---|---|---|
| Disabled | `0` | `0` |
| Enabled | `absent` | `absent` |

System Default: shown when the two values disagree or hold anything other than the combinations above (for example only one of them set, or `EnableMDNS = 1` from a policy); selecting it restores the snapshot. Stock Windows has neither value, so an untouched machine reads as "Enabled".

#### How it works

Windows ships a first-party Group Policy, "Configure multicast DNS (mDNS) protocol" (policy `DNS_MDNS` in `DnsClient.admx`, `class="Machine"`, key `Software\Policies\Microsoft\Windows NT\DNSClient`, value `EnableMDNS`, enabled = 1, disabled = 0, supported on "At least Windows 10"). The DNS Client service (`dnsrslvr.dll`) and `dnsapi.dll` both carry the `EnableMDNS` string, and `dnsrslvr.dll` carries both the policy path and the `Dnscache\Parameters` service path, so both locations are read.

The tweak writes both values because of how the shipped ADML describes the Disabled state: "If you disable this policy setting, or if you do not configure this policy setting, the DNS client will use locally configured settings." Read literally, policy 0 hands control back to local settings instead of switching mDNS off unconditionally. The service-key value is exactly that "locally configured setting", so writing 0 in both places makes mDNS off whichever way the resolver interprets the policy. The ADML wording is most likely boilerplate, but it is the only first-party statement of what 0 does.

With mDNS off, the DNS client neither issues nor answers multicast DNS queries on UDP 5353. mDNS is how `.local` host names, AirPrint and IPP Everywhere printers, Chromecast and Google Cast targets and Apple devices are discovered, and it is poisonable by the same Responder-class technique as LLMNR.

#### Benefits
- Closes mDNS poisoning.
- Completes the set: with LLMNR and NetBIOS already off, mDNS is the remaining broadcast name-resolution protocol.
- Uses a first-party Group Policy control.
- Fully reversible by deleting both values.

#### Drawbacks
- `.local` host names stop resolving.
- Printers discovered by mDNS (AirPrint, IPP Everywhere) disappear from discovery.
- Casting and device discovery break: Chromecast, Google Cast, Apple device discovery, some smart-home apps.
- The most noticeable of the three name-resolution tweaks.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer (verified on 26100). The ADMX declares "At least Windows 10", so Windows 10 including LTSC 2021 is expected to honour it, but that was not verified on 19044.
- **Takes effect**: after a reboot, or after restarting the DNS Client (`Dnscache`) service.
- **Reverting**: "Enabled" deletes both values (the stock state). Restore Snapshot puts back both captured values.

#### Interactions
- The triad with [Disable LLMNR](#disable-llmnr) (`EnableMulticast`, same policy key, different value) and [Disable NetBIOS over TCP/IP](#disable-netbios-over-tcpip) (per-adapter `NetbiosOptions`). Apply it last; it is the one users notice.
- Shares `Dnscache\Parameters` with [Enable DNS over HTTPS auto-upgrade](#enable-dns-over-https-auto-upgrade) and the `DNSClient` policy key with [Require encrypted DNS](#require-encrypted-dns); different values, no conflict.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections concerned scope and completeness: the policy is declared for Windows 10 and later (so no 26100-only gate), and the `Dnscache\Parameters` value is required, not optional, because the ADML defers the Disabled state to local settings.
- **Confidence**: Microsoft-documented (shipped 26100 ADMX and ADML verbatim, plus binary string extraction from `dnsrslvr.dll` and `dnsapi.dll`), with CIS Windows 11 v4.0.0 and two community tools agreeing on key, name and value.
- **Reasoning**: the control is real and in shipped binaries; the adversarial pass attacked the build gate (no tier A evidence for a 26100 floor) and the "optional second value" (contradicted by the ADML), and the shipped tweak reflects both. Open question: behaviour on 19044 is expected but not measured.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying on a hardened workstation with no `.local` dependencies and no mDNS-discovered printers or cast devices. If you print to a network printer found automatically, or cast to a TV, expect that to stop and skip this one.

#### Sources
1. `C:\Windows\PolicyDefinitions\DnsClient.admx` and `en-US\DnsClient.adml` on build 26100, policy `DNS_MDNS`: key, value, enabled 1, disabled 0, `supportedOn` Windows 10 RS2, and the `DNS_MDNS_Help` text (tier A, shipped ADMX)
2. UTF-16 string extraction from the shipped 26100 `dnsrslvr.dll` and `dnsapi.dll`: `EnableMDNS` alongside both the policy path and `Dnscache\Parameters` (tier A, shipped binary)
3. Policy CSP - ADMX_DnsClient, the DNS Client policy surface, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-dnsclient (tier A)
4. DNS Client Group Policy settings, https://learn.microsoft.com/en-us/windows-server/networking/dns/dns-top
5. CIS Windows 11 v4.0.0 mDNS recommendation, corroborated by WinUtil and privacy.sexy, agreement on key, value name and value 0 (tier B and tier C; no URL recorded in the research)

### Disable WPAD auto-proxy discovery

`disable_wpad` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows hunting the network for a proxy auto-configuration file, killing a classic man-in-the-middle trick that reroutes web traffic.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `disable_wpad_value` | registry | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings\WinHttp`, value `DisableWpad`, `REG_DWORD` |

| Option | `disable_wpad_value` |
|---|---|
| Disabled | `1` |
| Enabled | `absent` (value deleted) |

System Default: shown when `DisableWpad` holds a value other than 1; selecting it restores the snapshot. Stock Windows has no value (WPAD on), so an untouched machine reads as "Enabled".

#### How it works

Web Proxy Auto-Discovery (WPAD) makes WinHTTP look for a proxy configuration file by querying DHCP and resolving the host name `wpad` on the local network. Anyone who can answer that lookup (a rogue DHCP server, or a device that wins an LLMNR/NBT-NS/DNS race for `wpad`) can hand the machine a proxy script and route its HTTP traffic through themselves.

Microsoft's support article "How to disable HTTP proxy features" states: "Starting in Windows Server 2019 and Windows 10, version 1809, you can disable WPAD by setting a DWORD value for the following registry subkey to 1: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings\WinHttp\DisableWpad`". With it set, every proxy-detection call made through the WinHTTP API skips WPAD. This is a product key, not a policy key, so there is no edition gate.

Microsoft adds two caveats. First, "applications can still resolve the name 'WPAD' by calling Domain Name System (DNS) directly", so DNS-level `wpad` lookups continue. Second, "WPAD should also be disabled in the Windows Settings UI, because third-party apps and Internet browsers may rely on these settings for Proxy Auto-Discovery". This tweak does not change the Settings toggle (Settings > Network & internet > Proxy > Automatically detect settings).

The tweak deliberately leaves the `WinHttpAutoProxySvc` service alone: many Windows components depend on it, and disabling the service is the heavy-handed alternative.

#### Benefits
- Removes a live man-in-the-middle vector on untrusted networks.
- Microsoft's own documented switch.
- Free on any network that does not distribute proxy settings by WPAD, which is almost every home and small-office network.
- Works on every edition including Home.

#### Drawbacks
- Breaks networks that distribute proxy settings by WPAD: web access fails until a proxy is set by hand.
- Not complete coverage: applications can still resolve `wpad` through DNS themselves.
- Browsers and third-party apps may follow the Settings "Automatically detect settings" toggle, which this does not change.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1809 and later including LTSC 2021, all editions.
- **Takes effect**: after a reboot. Microsoft states no reboot requirement, but WinHTTP caches proxy configuration per process, so a reboot is the safe statement.
- **Reverting**: "Enabled" deletes the value (the stock state). Restore Snapshot puts back the captured value.

#### Interactions
- [Disable LLMNR](#disable-llmnr), [Disable NetBIOS over TCP/IP](#disable-netbios-over-tcpip) and [Disable mDNS](#disable-mdns) remove the broadcast channels an attacker could use to answer a `wpad` lookup; they complement this tweak.
- The Settings proxy auto-detect toggle is separate and should also be turned off for full coverage.

#### Validation
- **Verdict**: VERIFIED. Nothing needed correcting in the mechanism; the research added Microsoft's two coverage caveats (DNS lookups of `wpad` continue, and the Settings toggle matters for browsers and third-party apps).
- **Confidence**: Microsoft-documented (a Microsoft support article gives the key, value name, type and value verbatim).
- **Reasoning**: the key path, name, type and value match Microsoft's article exactly; the adversarial pass found only omissions in coverage claims, recorded above. The decision not to touch `WinHttpAutoProxySvc` was judged correct.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any home or small-office machine that does not use automatic proxy configuration, and also turn off "Automatically detect settings" in Settings. On a corporate network that distributes its proxy by WPAD, leave it alone or configure the proxy manually first.

#### Sources
1. How to disable HTTP proxy features, the key, value name, type and value 1 verbatim, plus both caveats, https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/disable-http-proxy-auth-features (tier A)
2. WinHTTP AutoProxy functions, what WPAD detection does and which API calls it covers, https://learn.microsoft.com/en-us/windows/win32/winhttp/winhttp-autoproxy-api (tier A)

### Disable IPv6 transition technologies

`disable_ipv6_transition` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Drops the legacy IPv6 tunnel interfaces so 6to4 cannot quietly build a tunnel and register its addresses, while native IPv6 keeps working.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `disabled_components` | registry | `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters`, value `DisabledComponents`, `REG_DWORD` |

| Option | `disabled_components` |
|---|---|
| Tunnels disabled | `0x01` |
| Enabled | `absent` (value deleted) |

System Default: shown when `DisabledComponents` holds any other value, for example `0x20` (prefer IPv4) or `0xFF` (IPv6 disabled) set by another tool or admin; selecting it restores the snapshot. Stock Windows has no value (equivalent to `0x00`), so an untouched machine reads as "Enabled".

#### How it works

`DisabledComponents` is a documented bitmask read by the IPv6 stack (`Tcpip6`) at boot. Microsoft's table maps bit 0 (`0x01`) to "Disable tunnel interfaces", and its guidance says directly: "You can disable the 6to4 tunneling protocol and other IPv6 transition Technologies by using one of the following methods: Set the `DisabledComponents` registry key to 0x01." Native IPv6 on LAN and PPP interfaces is bit 4 (`0x10`), which `0x01` does not touch. The range is `0x00` (default) to `0xFF` (IPv6 fully disabled). Microsoft states "You must restart your computer for these changes to take effect."

The transition technologies are 6to4, Teredo and ISATAP. 6to4 activates on its own whenever an interface holds a public IPv4 address, and can register tunnel addresses in DNS. Microsoft states "ISATAP and Teredo are disabled by default in Windows", so on a stock machine the actual change is 6to4 and the generic tunnel interface.

Microsoft also warns: "Values other than 0 or 32 causes the Routing and Remote Access service to fail after this change takes effect." `0x01` is such a value.

#### Benefits
- Stops 6to4 auto-tunnels, which Microsoft itself recommends disabling when unwanted.
- No tunnel addresses registered in DNS and no unexpected tunnelled connectivity path.
- Native IPv6 is intact (this is bit 0, not the `0x10` bit).

#### Drawbacks
- Breaks Routing and Remote Access (RRAS), per Microsoft.
- Smaller change than it sounds: ISATAP and Teredo are already off by default.
- Apps that depend on Teredo (some older peer-to-peer software and legacy console networking) lose it.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions.
- **Takes effect**: after a reboot, which Microsoft requires for this value.
- **Reverting**: "Enabled" deletes the value, returning to the `0x00` default. Restore Snapshot puts back the captured value (useful if the machine had `0x20` or another deliberate value before).

#### Interactions
- Any other tool or admin setting of `DisabledComponents` (for example `0x20` "prefer IPv4 over IPv6") shares this one value; applying this tweak replaces it with `0x01`, and the snapshot remembers the old value.
- Do not confuse `0x01` with `0xFF`: `0xFF` disables IPv6 entirely, which Microsoft does not support.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The value is right; the corrections concerned consequences: Microsoft's RRAS warning applies to `0x01`, and ISATAP and Teredo are already off by default, so the real change on a stock machine is 6to4.
- **Confidence**: Microsoft-documented (Microsoft's IPv6 configuration guidance gives the bitmask, the `0x01` recommendation, the reboot requirement, the RRAS warning and the default state of ISATAP and Teredo).
- **Reasoning**: the adversarial pass attacked the "safe and recommended for most users" framing using Microsoft's own RRAS note, and the claimed delta using Microsoft's statement about defaults; both are reflected here. The bit mapping itself was never in doubt.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying on an ordinary client that does not need tunnelled IPv6, which is most machines. Do not apply it on anything running Routing and Remote Access, or on a machine that depends on Teredo or an ISATAP intranet deployment.

#### Sources
1. Guidance for configuring IPv6 in Windows for advanced users, the `DisabledComponents` bitmask, the `0x01` recommendation, the RRAS warning, the restart requirement, and the default state of ISATAP and Teredo, https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/configure-ipv6-in-windows (tier A)
2. IPv6 transition technologies, what 6to4, Teredo and ISATAP are and when each activates, https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/bb726951(v=technet.10) (tier A)

### Disable Internet Connection Sharing

`disable_internet_connection_sharing` · Switch · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off Internet Connection Sharing so the PC cannot be turned into an unmanaged router and NAT, at the cost of breaking WSL2 and Windows Sandbox networking.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `ics_service` | service | `SharedAccess` ("Internet Connection Sharing (ICS)"), start type |
| `show_shared_access_ui` | registry | `HKLM\SOFTWARE\Policies\Microsoft\Windows\Network Connections`, value `NC_ShowSharedAccessUI`, `REG_DWORD` |

| Option | `ics_service` | `show_shared_access_ui` |
|---|---|---|
| Disabled | `disabled` | `0` |

System Default: this is a toggle with one authored option, so any state other than "service Disabled and policy value 0" shows as System Default, including an untouched machine. Turning the toggle off restores the snapshot (the captured start type and policy value). The stock start type of `SharedAccess` on a clean Windows 11 image is not confirmed by the research (it has been asserted as Manual), which is why no "enabled" option is authored and the way back is always the snapshot. The stock policy value is absent.

#### How it works

`SharedAccess` is the ICS service, implemented in `ipnathlp.dll`. On 26100 it depends on `BFE` (Base Filtering Engine) and nothing depends on it in the service control manager's graph. Its NAT engine is what lets one adapter share its connection with another, and it is also the NAT behind the Hyper-V Default Switch, which is how WSL2 and Windows Sandbox reach the network.

The registry value is the Group Policy "Prohibit use of Internet Connection Sharing on your DNS domain network" (`NC_ShowSharedAccessUI` in the shipped `NetworkConnections.admx`, `class="Machine"`, supported on Windows XP and later). Its polarity is inverted: enabling the policy writes 0 and disabling it writes 1. With 0, the Sharing tab option disappears from network connection properties.

The service effect changes the start type through the service control manager; the app does not stop a running service. If `SharedAccess` is already running when you apply, it keeps running (and keeps NATting) until it stops or the machine restarts; from then on it cannot start.

The Windows Mobile Hotspot service (`icssvc`) depends on `RpcSs` and `wcmsvc`, not on `SharedAccess`, so Mobile Hotspot is not a formal dependent. It may still fail at runtime because it uses the same NAT engine; the research did not establish this either way.

#### Benefits
- The machine cannot start NATting another network onto your connection.
- The Sharing option disappears from connection properties, so it cannot be enabled by mistake.
- Benchmark-backed: DISA's Windows 11 STIG has a dedicated rule, "Internet connection sharing must be disabled".

#### Drawbacks
- Breaks WSL2, the Hyper-V Default Switch and Windows Sandbox networking; this is the most common cause of "WSL2 suddenly has no internet". Docker Desktop on WSL2 is affected the same way.
- Mobile Hotspot may stop working (same NAT engine, no formal dependency).
- You can no longer share one adapter's connection with another.
- A running `SharedAccess` instance is not stopped until it exits or the machine restarts.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021. The policy is declared for Windows XP and later, so every supported platform qualifies.
- **Takes effect**: the policy value and the start type are written immediately. The service is not stopped by the app, so a running instance stops only when it exits on its own or at the next restart; the tweak is flagged as needing a reboot for that reason.
- **Reverting**: turning the toggle off restores the captured start type and the captured policy value (normally absent) from the snapshot. There is no hard-coded "stock" start type.

#### Interactions
- None in this category touches the NAT engine.
- Any WSL2, Docker Desktop (WSL2 backend), Hyper-V Default Switch or Windows Sandbox use on the same machine depends on this service.

#### Validation
- **Verdict**: VERIFIED. The mechanism and the inverted polarity check out; the research corrected the risk story (the Mobile Hotspot service does not depend on `SharedAccess`, while the Hyper-V Default Switch, WSL2 and Windows Sandbox do rely on its NAT) and left the stock start type unconfirmed.
- **Confidence**: Microsoft-documented (shipped 26100 `NetworkConnections.admx`, Microsoft's ICS documentation), plus a live read of the service dependency graph on 26100.
- **Reasoning**: the adversarial pass refuted a claimed Mobile Hotspot service dependency from the SCM graph and added the WSL2 breakage, which is far more likely on a developer machine. Open question: the stock start type of `SharedAccess` on a clean image, which is why the tweak authors only the hardened state.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on an ordinary desktop or work machine that will never share a connection and never runs WSL2, Docker Desktop or Windows Sandbox. Do not apply it on a development machine, or if you use Mobile Hotspot.

#### Sources
1. `C:\Windows\PolicyDefinitions\NetworkConnections.admx` on build 26100, policy `NC_ShowSharedAccessUI`: `class="Machine"`, key, `enabledValue` 0, `disabledValue` 1, supported on Windows XP (tier A, shipped ADMX)
2. Prohibit use of Internet Connection Sharing, Policy CSP ADMX_NetworkConnections, https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-networkconnections
3. Internet Connection Sharing, what ICS does and that `ipnathlp` provides the NAT, https://learn.microsoft.com/en-us/windows/win32/ics/internet-connection-sharing (tier A)
4. Live service presence and SCM dependency graph on build 26100 (July 2026 research): `SharedAccess` backed by `ipnathlp.dll`, depends on `BFE`, no dependents; `icssvc` depends on `RpcSs` and `wcmsvc` (primary measurement, tier A for existence and dependencies only)
5. DISA STIG for Windows 11 V2R2, "Internet connection sharing must be disabled" (tier B; no URL recorded in the research)

### Firewall logging and local policy merge

`firewall_logging_and_merge` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Logs dropped packets on all three firewall profiles, and optionally makes Windows ignore firewall rules created locally by applications.**

#### What it changes

All twelve effects are `REG_DWORD` except the three `*_log_path` values, which are `REG_SZ`.

| Effect id | Kind | Target |
|---|---|---|
| `domain_log_dropped` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile\Logging`, `LogDroppedPackets` |
| `domain_log_path` | registry | same key, `LogFilePath` (`REG_SZ`) |
| `domain_log_size` | registry | same key, `LogFileSize` |
| `domain_merge` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile`, `AllowLocalPolicyMerge` |
| `private_log_dropped` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile\Logging`, `LogDroppedPackets` |
| `private_log_path` | registry | same key, `LogFilePath` (`REG_SZ`) |
| `private_log_size` | registry | same key, `LogFileSize` |
| `private_merge` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PrivateProfile`, `AllowLocalPolicyMerge` |
| `public_log_dropped` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile\Logging`, `LogDroppedPackets` |
| `public_log_path` | registry | same key, `LogFilePath` (`REG_SZ`) |
| `public_log_size` | registry | same key, `LogFileSize` |
| `public_merge` | registry | `HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile`, `AllowLocalPolicyMerge` |

Per profile (the same pattern for Domain, Private and Public; `<p>` is `domain`, `private` or `public`):

| Option | `LogDroppedPackets` | `LogFilePath` | `LogFileSize` | `AllowLocalPolicyMerge` |
|---|---|---|---|---|
| Log dropped packets | `1` | `%SystemRoot%\System32\logfiles\firewall\<p>fw.log` | `16384` | `absent` |
| Log dropped packets and ignore local rules | `1` | `%SystemRoot%\System32\logfiles\firewall\<p>fw.log` | `16384` | `0` |
| No dropped-packet logging | `absent` | `absent` | `absent` | `absent` |

The three log files are `domainfw.log`, `privatefw.log` and `publicfw.log`. System Default: shown when the twelve values match none of the three rows (for example logging set by Group Policy with a different path or size); selecting it restores the snapshot. Stock Windows has none of these values, so an untouched machine reads as "No dropped-packet logging".

#### How it works

The logging values are the registry form of the Windows Firewall "Allow logging" policy: the shipped `WindowsFirewall.admx` declares `LogDroppedPackets` (boolean, 1/0), `LogSuccessfulConnections` (not written here), `LogFilePath` (text, `required="true"`) and `LogFileSize` (decimal KB, `required="true"`, range 128 to 32767). The tweak writes all three required values together per profile, so the policy is never half-configured, and writes `LogDroppedPackets` as a DWORD (the ADMX `disabledList` uses a string form that must not be copied). With logging on, the Windows Defender Firewall service records every dropped inbound packet in the profile's log. 16384 KB (16 MB) replaces the 4096 KB default size.

`AllowLocalPolicyMerge` is documented by Microsoft's Firewall CSP (`MdmStore/<Profile>/AllowLocalPolicyMerge`, Default Value: true, "If this value is false, firewall rules from the local store are ignored and not enforced", Windows 10 1709 and later, Pro, Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC). It is not in any of the 218 shipped ADMX files; the registry path is corroborated by CIS check text. With merge off, only policy-delivered rules apply: the rules applications create for themselves (including the ones created when you answer the "Windows Defender Firewall has blocked some features of this app" prompt) are ignored, and that prompt never appears. On a standalone PC there is no policy rule set to replace them, so inbound connections to games, servers and sharing simply fail, silently.

Profile subkey names: under `SOFTWARE\Policies\Microsoft\WindowsFirewall` the Windows Defender Firewall with Advanced Security policy subkeys are `DomainProfile`, `PrivateProfile` and `PublicProfile` (the July 2026 security research, matching DISA's Windows Defender Firewall STIG check path, V-241990), so the tweak writes the private profile under `PrivateProfile`, as the sibling [security] tweak `firewall_all_profiles` does. `StandardProfile` is the name used under the non-policy `SharedAccess\Parameters\FirewallPolicy` path and by the legacy `WindowsFirewall.admx`, which on this platform defines logging policies only for `DomainProfile\Logging` and `StandardProfile\Logging` and has no Public logging policy; the tweak does not write `StandardProfile`.

#### Benefits
- Blocked inbound traffic is recorded per profile instead of vanishing, which makes "why can't X connect" diagnosable.
- A 16 MB log covers far more than the default 4 MB.
- With merge off, an application cannot quietly add its own allow rule.
- CIS recommends logging (and, for managed machines, merge control) on all three profiles.

#### Drawbacks
- "Ignore local rules" breaks inbound silently on a standalone PC: no prompt, and games, servers and file sharing just fail to accept connections.
- The Public-profile log fills quickly on a busy network.
- Up to 16 MB of disk per profile.
- `AllowLocalPolicyMerge` is documented for Pro and above only; on Home the merge half is not expected to do anything.
- Because the values live under `SOFTWARE\Policies`, the Windows Defender Firewall logging settings show as managed while applied.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 1709 and later including LTSC 2021. Logging is ADMX-backed; the merge option is documented for Pro, Education, Enterprise, IoT Enterprise and IoT Enterprise LTSC.
- **Takes effect**: immediately; the firewall service picks up policy changes without a reboot.
- **Reverting**: "No dropped-packet logging" deletes all twelve values, returning to default logging (off, 4 MB) and re-enabling local rule merge. Restore Snapshot puts back each captured value, including any the machine had from an earlier configuration.

#### Interactions
- [security] `firewall_all_profiles` writes `EnableFirewall` and `DefaultInboundAction` in the same profile policy keys (different values, no ownership conflict). The research rejected merging the two: turning the firewall on is something almost everyone wants, while disabling local merge is a foot-gun.
- Domain Group Policy for the same values overrides this on refresh.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The corrections concerned completeness and warning: `LogFilePath` and `LogFileSize` must be written together with `LogDroppedPackets` (all three are required elements), the values must be DWORDs, and the merge-off failure mode must be stated plainly as a separate option. The shipped tweak does all three.
- **Confidence**: Microsoft-documented for the logging values (shipped 26100 ADMX) and for `AllowLocalPolicyMerge` semantics (Firewall CSP); the `AllowLocalPolicyMerge` registry path is corroborated by CIS check text, not by an ADMX.
- **Reasoning**: value names, types and the 16384 size (inside the 128 to 32767 range) were confirmed against the shipped ADMX. Open questions: the research states the ADMX has one logging policy per profile including Public, but the shipped `WindowsFirewall.admx` defines only Domain and Standard logging policies, so the `PublicProfile\Logging` values and all three `AllowLocalPolicyMerge` values rest on the CSP and CIS rather than an ADMX. The category research wrote the private profile as `StandardProfile`; the security research found that is not the private profile's policy subkey, so the shipped tweak writes `PrivateProfile`, matching `firewall_all_profiles` and the STIG check path.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Choose "Log dropped packets" on any machine where you want to diagnose blocked connections; it is cheap and fully reversible. Choose "ignore local rules" only on a machine whose firewall rules are managed centrally (Group Policy or MDM). On a standalone home PC, never choose it.

#### Sources
1. `C:\Windows\PolicyDefinitions\WindowsFirewall.admx` on build 26100, the `WF_Logging_Name_*` policies: `class="Machine"`, the `Logging` keys, `LogFilePath` and `LogFileSize` marked `required="true"`, the 128 to 32767 range (tier A, shipped ADMX)
2. Firewall CSP, `MdmStore/<Profile>/AllowLocalPolicyMerge`: Default Value true, "If this value is false, firewall rules from the local store are ignored and not enforced", Windows 10 1709 and later, edition list, https://learn.microsoft.com/en-us/windows/client-management/mdm/firewall-csp (tier A)
3. Configure the Windows Firewall log, the default log path and the 4096 KB default size, https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/configure-logging (tier A)
4. CIS Windows 11 v4.0.0 Level 1, firewall logging and local policy merge recommendations for all three profiles (tier B; no URL recorded in the research)
5. DISA Windows Defender Firewall STIG V-241990, the `PrivateProfile` policy check path, https://www.stigviewer.com/stigs/microsoft_windows_defender_firewall_with_advanced_security/2023-08-23/finding/V-241990 (tier B)

### Disable NIC power management

`disable_nic_power_management` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows powering down your physical network adapters to save power, the standard cure for Wi-Fi or Ethernet that drops when the PC sits idle.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `NicPowerManagement`, `REG_DWORD` (the app's own marker of the chosen option) |
| `nic_power_off` | action (PowerShell, with `apply`, `undo` and `probe`) | writes `PnPCapabilities = 24` (`0x18`, `REG_DWORD`) under `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002bE10318}\<NNNN>` for each physical adapter whose driver description matches exactly (see How it works for the duplicate-adapter gap); records each key's prior value under `HKLM\SOFTWARE\MagicXToolbox\ActionSnapshots\NicPowerManagement` |

| Option | `state` | `nic_power_off` |
|---|---|---|
| Disabled | `1` | run (apply) |
| Enabled | `0` | not run (the engine runs the action's `undo` if the probe reads it present) |

What the action does, precisely:

- **apply**: lists the adapters Windows reports as physical (`Get-NetAdapter -Physical`) and takes their interface descriptions. It fails (exit 1) if there are none. For each four-digit subkey of the network adapter class key whose `DriverDesc` matches one of those descriptions, it stores the prior `PnPCapabilities` under the snapshot key (value name = the subkey number, `-1` if absent) and writes 24. It fails if no subkey matched. Virtual adapters, WAN Miniports, the Kernel Debug adapter, Wi-Fi Direct virtual adapters and the Bluetooth PAN device are skipped.
- **undo**: for each subkey number recorded in the snapshot, writes the recorded value back, or deletes `PnPCapabilities` where the record is `-1`. Then deletes the snapshot key. Keys the apply never touched are left alone.
- **probe**: "applied" only if at least one physical adapter's class subkey was found and every such subkey reads exactly 24.

System Default: shown on an untouched machine (the HKCU marker does not exist yet) and whenever the physical adapters' values stop matching (for example after a driver reinstall resets `PnPCapabilities`, or a new physical adapter appears). Selecting it restores the snapshot. Stock Windows: per adapter and mixed. Microsoft documents 0 (power management enabled) as the default meaning; on the research machine only 1 of 16 network class subkeys had any value (a MediaTek Wi-Fi 6 MT7921 with `16`, set by its driver) and the other 15 had none.

#### How it works

`PnPCapabilities` in an adapter's driver key is read by NDIS (`ndis.sys`; the string is present there on 26100.4061 and absent from `pci.sys` and `umpnpmgr.dll`) when the device starts. Microsoft's support article "Power management setting on a network adapter" (originally KB 2740020) documents it: "By default, a value of 0 indicates that power management of the network adapter is enabled. A value of 24 will prevent Windows from turning off the network adapter or let the network adapter wake the computer from standby." It maps the Device Manager Power Management checkboxes to values (all three checked `0x100`/256, only the first `0x110`/272, first cleared with the other two greyed `0x118`/280, default 0) and states "For deployment purpose, to keep option 1 cleared, one needs to use the value 24 (0x18)." So 24 is Microsoft's own deployment value for clearing "Allow the computer to turn off this device to save power". It also stops the adapter waking the computer from standby (Wake-on-LAN through that adapter).

The action matches adapters by comparing the class key's `DriverDesc` to the physical adapter's interface description, because only physical adapters expose the Power Management tab. The match is exact text, and Windows appends " #2" (and so on) to the interface description of a second identical adapter but not to its `DriverDesc` (on the test machine, "Microsoft Wi-Fi Direct Virtual Adapter #2" has `DriverDesc` "Microsoft Wi-Fi Direct Virtual Adapter"). A second identical physical NIC is therefore skipped by both the apply and the probe, silently. Because it records each key's prior value, including "absent", the revert puts back a vendor value such as the MediaTek's 16 rather than deleting it.

Microsoft's per-adapter PowerShell alternative is `Set-NetAdapterPowerManagement -AllowComputerToTurnOffDevice Disabled`. Microsoft's Exchange Health Checker "Sleepy NIC Check" flags NIC power saving as a cause of packet loss.

The applied-state marker is in HKCU (per user) while the change is machine-wide, so another account sees System Default; the over-the-shoulder guard applies as for any HKCU-writing tweak.

#### Benefits
- Stops idle disconnects and the latency spike when traffic resumes on a parked link.
- Uses Microsoft's documented deployment value.
- Touches only physical adapters (never virtual ones), and the revert restores each adapter's own prior value.

#### Drawbacks
- Higher idle power; a measurable battery cost on laptops.
- Also removes the adapter's ability to wake the PC from standby (value 24 clears wake too), so Wake-on-LAN through that adapter stops.
- A driver reinstall or upgrade can reset the value without the app noticing until the next scan shows System Default.
- The stock state differs per adapter, so there is no single "stock" value to go back to other than the snapshot.
- A second identical physical adapter (same model, interface description ending in " #2") is not matched, so it keeps power management on and the probe does not notice.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions, on physical adapters whose driver exposes the Power Management tab.
- **Takes effect**: after a reboot (or when the device next starts), since NDIS reads the value at device start.
- **Reverting**: "Enabled" runs the undo, which restores each recorded adapter's prior value or deletes it where there was none. If the `ActionSnapshots\NicPowerManagement` key is missing, the undo changes nothing on the adapters and exits successfully, which leaves 24 in place; the engine's post-undo probe then still reads "applied", so the revert fails verification and surfaces as Needs Attention instead of reporting success.

#### Interactions
- [Disable USB selective suspend](#disable-usb-selective-suspend) looks like a pair but is independent: a different mechanism (power-scheme index versus per-adapter driver value), different symptoms and a different revert.
- [Disable Modern Standby (force S3)](#disable-modern-standby-force-s3) and [Disable wake timers](#disable-wake-timers) also change sleep and wake behaviour; this one only affects network adapters.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The value 24 is settled and Microsoft-documented; every correction was about the action: restore each key's prior value (including absent) instead of deleting, write only to physical adapters, keep the undo to exactly the keys the apply wrote, and have the probe require every physical adapter to read 24. The shipped action does all four.
- **Confidence**: Microsoft-documented (KB 2740020 successor article names the value, the default and the deployment value 24), plus binary and registry inspection on 26100.4061.
- **Reasoning**: the adversarial pass initially treated 24 as community-sourced; the Microsoft article overturned that. The measured MediaTek value of 16 is what drove the per-key restore. Open question: the stock `PnPCapabilities` distribution across adapters on a clean image.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth it on a desktop, or on any machine that actually suffers idle network drops. On a laptop with no connectivity symptoms, leave power management on and keep the battery life. Skip it if you rely on Wake-on-LAN.

#### Sources
1. Power management setting on a network adapter (originally KB 2740020): the key, the value name, the default 0, the meaning of 24, the checkbox mapping and "For deployment purpose ... use the value 24 (0x18)", https://learn.microsoft.com/en-us/troubleshoot/windows-client/networking/power-management-on-network-adapter (tier A)
2. How NDIS sets the power policy for a network adapter, checkbox semantics and defaults, https://learn.microsoft.com/en-us/windows-hardware/drivers/network/how-ndis-sets-the-power-policy-for-a-network-adapter (tier A)
3. Set-NetAdapterPowerManagement, the documented per-adapter alternative, https://learn.microsoft.com/en-us/powershell/module/netadapter/set-netadapterpowermanagement (tier A)
4. Exchange Health Checker "Sleepy NIC Check", Microsoft tooling that recommends disabling NIC power saving because it can cause packet loss, https://microsoft.github.io/CSS-Exchange/Diagnostics/HealthChecker/SleepyNICCheck/ (tier A/B)
5. Direct binary and registry inspection on Windows 11 24H2 build 26100.4061 (July 2026 research): `PnPCapabilities` in `ndis.sys` only; 1 of 16 network class subkeys carried a value (MediaTek Wi-Fi 6 MT7921, `16`) (primary measurement)

### Disable hibernation

`disable_hibernation` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Turns off hibernation and deletes `hiberfil.sys`, reclaiming several gigabytes of disk, at the cost of hibernate, hybrid sleep and Fast Startup.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `Hibernation`, `REG_DWORD` (the app's own marker of the chosen option) |
| `hibernate_off` | action (PowerShell, with `apply`, `undo` and a native registry `probe`) | runs `powercfg /hibernate off`; records the prior state under `HKLM\SOFTWARE\MagicXToolbox\ActionSnapshots\Hibernation`; the probe reads `HKLM\SYSTEM\CurrentControlSet\Control\Power` `HibernateEnabled` and reports "applied" when it equals 0 |

| Option | `state` | `hibernate_off` |
|---|---|---|
| Disabled | `1` | run (apply) |
| Enabled | `0` | not run (the engine runs the action's `undo` if the probe reads it present) |

What the action does, precisely:

- **apply**: records `HibernateEnabled` and `HiberFileSizePercent` from `HKLM\SYSTEM\CurrentControlSet\Control\Power` (as `Enabled` and `SizePercent`, `-1` if absent), then runs `powercfg /hibernate off` and fails if it returns non-zero.
- **undo**: if the recorded `Enabled` is 0 (hibernation was already off before apply), it leaves hibernation off, deletes the snapshot key and exits 0. The probe then still reads `HibernateEnabled = 0` ("applied"), so the engine's post-undo check fails: choosing "Enabled" fails and is rolled back to "Disabled", and Restore Snapshot ends in Needs Attention. Hibernation does stay off, but the revert never reports success. Otherwise it runs `powercfg /hibernate on`, fails if that returns non-zero, then, if a size percentage above 0 was recorded, runs `powercfg /hibernate /size <percent>`, and deletes the snapshot key.

System Default: shown on an untouched machine (the HKCU marker does not exist yet) and whenever the marker and the live `HibernateEnabled` disagree (for example hibernation turned back on with `powercfg /h on` after applying). Selecting it restores the snapshot. Stock Windows: hibernation enabled on most machines; the research machine read `HibernateEnabled = 0` with `HibernateEnabledDefault = 1`, and that machine is owner-modified, so it is not evidence of the Windows default.

#### How it works

`powercfg /hibernate off` clears hibernation support and deletes the hidden `C:\hiberfil.sys`, which Windows sizes as a fraction of installed RAM. The power manager records the state in `HibernateEnabled`. Because Fast Startup is a partial hibernation (the kernel session is written to `hiberfil.sys` at shutdown), turning hibernation off also makes Fast Startup unavailable, and hybrid sleep, which writes a hibernation image while sleeping, goes too. On the research machine `powercfg /a` reported Hibernate ("Hibernation has not been enabled"), Hybrid Sleep and Fast Startup ("Hibernation is not available") all unavailable, confirming the cascade.

The revert is conditional: a machine that had hibernation off before you applied stays off, rather than having a multi-gigabyte `hiberfil.sys` and Fast Startup recreated. Because the probe cannot tell "off because this tweak turned it off" from "off before apply", that case fails the engine's post-undo check (see the undo bullet above). A previously reduced hibernation file size is put back with `/size`. A previous `powercfg /hibernate /type reduced` configuration (hibernation file kept only for Fast Startup) is not recorded, so a revert brings back a full hibernation file type.

#### Benefits
- Reclaims disk space equal to a large fraction of installed RAM, often several gigabytes.
- Immediate: the file is deleted on apply, no reboot needed.
- No loss on a desktop that never hibernates.

#### Drawbacks
- No hibernate, no hybrid sleep and no Fast Startup: all three depend on the same file.
- Laptops lose the low-battery safety net: hibernation saves the session to disk when the battery runs critically low.
- Modern Standby machines can drain flat: hibernate-after-standby is what stops an idle Modern Standby laptop discharging completely.
- A prior `/type reduced` configuration is not preserved by the revert.
- On a machine that already had hibernation off before apply, the revert cannot verify: "Enabled" fails and rolls back to "Disabled", and Restore Snapshot ends in Needs Attention (hibernation stays off either way).

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions. Requires administrator.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" runs the undo. If hibernation was on before apply, it comes back with the recorded size percentage. If it was already off, it stays off but the revert fails its check: "Enabled" is rolled back to "Disabled", and Restore Snapshot ends in Needs Attention. If the snapshot key is missing, the undo turns hibernation on at the default size.

#### Interactions
- [performance] `disable_fast_startup` (`HiberbootEnabled = 0`) turns off Fast Startup alone; with hibernation disabled here, Fast Startup is already unavailable, so that tweak changes nothing extra until hibernation is re-enabled.
- [Disable Modern Standby (force S3)](#disable-modern-standby-force-s3): on a machine still using Modern Standby, hibernation is the protection against in-bag drain; do not remove both protections.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The command and the probe target are correct; the correction concerned the revert, which must not turn hibernation on where it was already off and should keep the prior size. The shipped undo is conditional on the recorded state and restores the size percentage; the hibernation file type is not recorded. When hibernation was already off before apply, the undo leaves it off but the probe still reads "applied", so that revert fails verification rather than reporting success.
- **Confidence**: Microsoft-documented (`powercfg` reference and Microsoft's sleep settings documentation), plus `powercfg /a` and registry reads on 26100.
- **Reasoning**: the adversarial pass attacked the unconditional revert using a measured machine where `HibernateEnabled` was already 0. The probe-fail-open audit lists this tweak among the probes that already fail safe (the native registry probe reports "applied" only when the value reads 0).
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Good on a desktop or a space-constrained SSD where you never hibernate. On a laptop, think twice: you give up the low-battery hibernate safety net, which is the one thing standing between a flat battery and a lost session.

#### Sources
1. powercfg command-line options, `/hibernate on|off`, `/hibernate /size` and `/hibernate /type reduced`, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A)
2. Sleep settings overview, the hibernate idle timeout and hybrid sleep relationships, https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings (tier A)
3. `powercfg /a` and a read of `HKLM\SYSTEM\CurrentControlSet\Control\Power` on Windows 11 24H2 build 26100 (July 2026 research): `HibernateEnabled = 0`, `HibernateEnabledDefault = 1`, and the Hibernate, Hybrid Sleep and Fast Startup cascade (primary measurement)

### Disable USB selective suspend

`disable_usb_selective_suspend` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops Windows suspending idle USB ports, the standard fix for USB audio dropouts, dongle disconnects and slow-to-wake hubs.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `UsbSelectiveSuspend`, `REG_DWORD` (the app's own marker of the chosen option) |
| `usb_suspend_off` | action (PowerShell, with `apply`, `undo` and `probe`) | power-plan setting "USB selective suspend setting" (subgroup `2a737441-1930-4402-8d77-b2bebba308a3` "USB settings", setting `48e6b7a6-50f5-4782-a5d4-53bb8f07e226`) on the active scheme, AC and DC indices; records the prior state under `HKLM\SOFTWARE\MagicXToolbox\ActionSnapshots\UsbSelectiveSuspend` |

| Option | `state` | `usb_suspend_off` |
|---|---|---|
| Disabled | `1` | run (apply) |
| Enabled | `0` | not run (the engine runs the action's `undo` if the probe reads it present) |

The setting's indices are 0 "Disabled" and 1 "Enabled". What the action does, precisely:

- **apply**: reads the active scheme GUID (`powercfg /GETACTIVESCHEME`) and fails if none is found; queries the current AC and DC indices for the setting on that scheme; records the scheme GUID and both indices (falling back to 1 if an index cannot be parsed); sets AC and DC to 0 with `/SETACVALUEINDEX` and `/SETDCVALUEINDEX`; re-activates the scheme with `/SETACTIVE`, failing if that returns non-zero.
- **undo**: writes the recorded AC and DC indices back to the recorded scheme (not whichever plan is active now) and re-activates that scheme; if no record exists, it writes 1 and 1 to the current scheme. Then deletes the snapshot key.
- **probe**: "applied" only if both the AC and the DC index of the currently active scheme read 0.

System Default: shown on an untouched machine (the HKCU marker does not exist yet), and whenever the active plan does not have both indices at 0, which includes switching to a different power plan after applying. Selecting it restores the snapshot. Stock Windows: Balanced has selective suspend enabled (index 1) on both AC and DC (measured on 26100).

#### How it works

USB selective suspend lets the USB hub driver suspend an individual idle port so the device on it draws less power, without suspending the whole bus. The power plan decides whether the hub driver may do this. Some devices (USB audio interfaces and DACs, wireless receivers, some hubs) resume badly: audio glitches, a stall before the device responds, or a disconnect. Setting the plan value to Disabled keeps ports powered.

Power-plan values are per scheme and per power source. The action writes only the scheme that is active at apply time, and records its GUID so the undo writes back to the same scheme even if you have switched plans since. `/SETACTIVE` applies the change without a reboot.

#### Benefits
- Fixes USB audio dropouts and glitches after idle.
- Wireless receivers and dongles stay awake and responsive.
- Devices behind a hub respond immediately.
- The revert restores the exact prior AC and DC indices on the exact scheme that was changed.

#### Drawbacks
- Higher idle power, a small but real battery cost on laptops.
- Only the plan active at apply time is changed: switching to another plan (including enabling Ultimate Performance) brings selective suspend back, and the tweak then shows System Default.
- Reverting re-activates the recorded plan with `/SETACTIVE`, so if you switched plans after applying, the revert silently makes the old plan your active plan again.
- No benefit if your USB devices behave.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions. Requires administrator.
- **Takes effect**: immediately; the scheme is re-activated on apply.
- **Reverting**: "Enabled" runs the undo, restoring the recorded AC and DC indices on the recorded scheme. Without a record, it writes the stock Balanced value (1) to both on the current scheme.

#### Interactions
- [performance] `ultimate_performance_power_plan` changes which scheme is active; after it switches plans, this tweak's change stays on the old plan and the probe reads the new one.
- [Disable wake timers](#disable-wake-timers) uses the same scheme-pinning pattern on a different setting; independent.
- [Disable NIC power management](#disable-nic-power-management) is a different mechanism for network adapters, not USB ports.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. Both GUIDs and the index meanings are correct; the corrections concerned the revert and detection: restore the recorded indices rather than a fixed 1, pin the scheme so a plan switch cannot misdirect the undo, and check both AC and DC in the probe. The shipped action does all three.
- **Confidence**: Microsoft-documented (Microsoft's USB selective suspend and `powercfg` references), plus live `powercfg /QUERY` output on 26100 for the names, indices and stock values.
- **Reasoning**: the adversarial pass accepted the mechanism and attacked only revert fidelity, scheme drift and a one-rail probe. The probe-fail-open audit lists this probe as failing safe (it reports "applied" only on an explicit match of both rails).
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you actually experience USB dropouts, audio glitches or slow wake-ups. If your USB devices behave, especially on a laptop, leave selective suspend on and keep the power saving.

#### Sources
1. USB selective suspend, what the feature does and why it can stall devices, https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-selective-suspend (tier A)
2. powercfg command-line options, `/SETACVALUEINDEX`, `/SETDCVALUEINDEX`, `/SETACTIVE` and `/QUERY`, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A)
3. `powercfg /QUERY` on Windows 11 24H2 build 26100 (July 2026 research): subgroup and setting names, indices 0 "Disabled" and 1 "Enabled", stock Balanced AC = 1 and DC = 1 (primary measurement)

### Disable wake timers

`disable_wake_timers` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops scheduled tasks waking the PC from sleep, ending mystery overnight wake-ups.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `WakeTimers`, `REG_DWORD` (the app's own marker of the chosen option) |
| `wake_timers_off` | action (PowerShell, with `apply`, `undo` and `probe`) | power-plan setting "Allow wake timers" / "Automatically wake for tasks" (`SUB_SLEEP` = `238c9fa8-0aad-41ed-83f4-97be242c8f20`, `RTCWAKE` = `bd3b718a-0680-4d9d-8ab2-e1d2b4ac806d`) on the active scheme, AC and DC indices; records the prior state under `HKLM\SOFTWARE\MagicXToolbox\ActionSnapshots\WakeTimers` |

| Option | `state` | `wake_timers_off` |
|---|---|---|
| Disabled | `1` | run (apply) |
| Enabled | `0` | not run (the engine runs the action's `undo` if the probe reads it present) |

The setting has three indices: 0 "Disable", 1 "Enable", 2 "Important Wake Timers Only". What the action does, precisely:

- **apply**: reads the active scheme GUID and fails if none is found; queries the current AC and DC indices; records the scheme GUID and both indices (falling back to the Windows 11 stock Balanced values AC 2 and DC 0 if an index cannot be parsed, never to "Enable"); sets AC and DC to 0; re-activates the scheme, failing if that returns non-zero.
- **undo**: writes the recorded indices back to the recorded scheme and re-activates it; without a record, writes AC 2 and DC 0 to the current scheme. Then deletes the snapshot key.
- **probe**: "applied" only if both the AC and the DC index of the currently active scheme read 0.

System Default: shown on an untouched machine (the HKCU marker does not exist yet) and whenever the active plan does not have both indices at 0, including after switching plans. Selecting it restores the snapshot. Stock Windows 11 Balanced (measured on 26100): AC = 2 "Important Wake Timers Only", DC = 0 "Disable".

#### How it works

Microsoft describes the setting as: "Specifies whether the system uses the system-wide wake-on-timer capability. The system can automatically use wake-on-timer on capable hardware to perform scheduled tasks. For example, the system might wake automatically to install updates." A scheduled task with "Wake the computer to run this task" arms an RTC wake timer; this plan value decides whether the power manager honours it.

On stock Windows 11 the battery (DC) value is already "Disable", so the DC half of the apply changes nothing on a stock machine. On mains (AC) the stock value is "Important Wake Timers Only", which already excludes ordinary scheduled tasks and allows only timers Windows marks as important (such as update restarts). The real effect of this tweak on a stock machine is therefore AC from 2 to 0: even the important wake timers stop waking the PC on mains power.

As with USB selective suspend, only the scheme active at apply time is changed, its GUID is recorded so the undo returns to the same scheme, and `/SETACTIVE` applies the change without a reboot.

#### Benefits
- The PC stays asleep: no 3 a.m. maintenance or update wake with fans spinning up.
- An unattended laptop is not woken to do work.
- Removes the "important" wake timers on mains power that stock Windows still allows.
- The revert restores the exact prior indices, so apply then revert never leaves the machine more wake-prone than before.

#### Drawbacks
- Legitimate scheduled wakes stop: overnight backups, media recording and alarms that rely on waking the machine.
- Windows Update maintenance may only run while you are using the PC.
- On battery nothing changes on a stock Windows 11 machine, since wake timers are already disabled there.
- Only the plan active at apply time is changed; switching plans brings wake timers back.
- Reverting re-activates the recorded plan with `/SETACTIVE`, so if you switched plans after applying, the revert silently makes the old plan your active plan again.
- Wake sources that are not timers (a keyboard, mouse, network adapter wake, or the lid) are unaffected.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions, on hardware with RTC wake. Requires administrator.
- **Takes effect**: immediately; the scheme is re-activated on apply.
- **Reverting**: "Enabled" runs the undo, restoring the recorded AC and DC indices on the recorded scheme. Without a record, it writes the Windows 11 stock Balanced values (AC 2, DC 0) to the current scheme; on a Windows 10 machine whose stock values differ, that fallback may not match its original state.

#### Interactions
- [Disable USB selective suspend](#disable-usb-selective-suspend) uses the same scheme-pinned pattern on another setting; independent.
- [performance] `ultimate_performance_power_plan` switches the active scheme; this tweak's change stays on the old plan.
- [Disable NIC power management](#disable-nic-power-management) writes value 24, which also stops network adapters waking the PC from standby; a different wake source.
- Windows Update maintenance and scheduled backups that rely on waking the PC stop waking it.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The setting and aliases are correct; the corrections concerned the revert and the stock state: stock Windows 11 Balanced is AC 2 and DC 0 (not "Enable" on both), so the revert must restore the recorded indices (or those stock values), pin the scheme, and the probe must check both rails. The shipped action does all of this.
- **Confidence**: Microsoft-documented (Microsoft's "Automatically wake for tasks" and sleep settings pages), plus `powercfg /aliases` and `/QUERY` on 26100 for the three indices and stock values.
- **Reasoning**: the adversarial pass identified a revert to "Enable" on both rails as the most consequential revert defect in the category, because it would leave a machine waking on battery where it never did; the shipped undo restores recorded values and falls back to the stock pair. The stock values are from one owner-modified machine plus Balanced defaults; a clean-image baseline is the open question.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Worth applying if a PC that wakes itself on mains power at night bothers you. Skip it if you rely on scheduled overnight work, such as automated backups, that needs to wake the machine.

#### Sources
1. Automatically wake for tasks, the setting GUID, the `RTCWAKE` alias and what wake-on-timer does, https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings-automatically-wake-for-tasks (tier A)
2. Sleep settings overview, the `SUB_SLEEP` subgroup GUID and alias, https://learn.microsoft.com/en-us/windows-hardware/customize/power-settings/sleep-settings (tier A)
3. powercfg command-line options, the index-setting syntax, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options (tier A)
4. `powercfg /aliases` and `powercfg /QUERY 381b4222-f694-41f0-9685-ff5bb260df2e SUB_SLEEP RTCWAKE` on Windows 11 24H2 build 26100 (July 2026 research): three indices, stock Balanced AC = 2 and DC = 0 (primary measurement)

### Disable Modern Standby (force S3)

`disable_modern_standby` · Switch (2 options) · Risk: high · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Forces classic S3 sleep instead of Modern Standby (S0 Low Power Idle) to stop in-bag battery drain, on hardware whose firmware still offers S3.**

#### What it changes

| Effect id | Kind | Target |
|---|---|---|
| `platform_aoac_override` | registry | `HKLM\SYSTEM\CurrentControlSet\Control\Power`, value `PlatformAoAcOverride`, `REG_DWORD` |
| `require_s3` | action (PowerShell, `ephemeral: true`, apply only) | a precondition check: parses `powercfg /a` and exits 0 only if the "Standby (S3)" line is listed as available; otherwise exits 1, which fails the apply |

| Option | `platform_aoac_override` | `require_s3` |
|---|---|---|
| Force S3 | `0` | run |
| Modern Standby | `absent` (value deleted) | not run |

How the precondition decides: it finds the line containing `(S3)` in `powercfg /a` output. An unavailable state is followed by a more-indented reason line (such as "The system firmware does not support this standby state"); an available one is not. The check passes only when the next line is blank or not more indented, and fails if no `(S3)` line exists. When it fails, the apply fails and the engine rolls back the registry write from the snapshot, so the value is not left behind.

System Default: shown when `PlatformAoAcOverride` holds a value other than 0 (for example 1 from another tool); selecting it restores the snapshot. The precondition is ephemeral and plays no part in detection. Stock Windows has no value, so an untouched machine reads as "Modern Standby" (even on a machine that uses S3 by design, since the option only describes the value).

#### How it works

A Windows platform uses either Modern Standby (S0 Low Power Idle, where the system stays in a low-power "on" state and can keep network connectivity and do background work while the screen is off) or traditional S3 sleep (the CPU halts and RAM is kept in self-refresh), not both at once. Microsoft documents both models, and states: "Switching between S3 and Modern Standby cannot be done by changing a setting in the BIOS. Switching the power model is not supported in Windows without a complete OS re-install."

`PlatformAoAcOverride` (AoAc = "always on, always connected", the internal name for Modern Standby) is not documented by Microsoft, but it is real: a sweep of 5,418 binaries in `System32` on build 26100.4061 found the string only in the kernel (`ntoskrnl.exe` and its LA57 variant `ntkrla57.exe`), which is where the S0 versus S3 decision is made. Four or more independent community references agree on the key, the name, `REG_DWORD`, and that 0 turns Modern Standby off. On firmware that still exposes S3, setting 0 makes Windows use S3 at the next boot (visible in `powercfg /a`), and Device Manager's Power Management tabs reappear.

The value does not create an S3 path where the firmware has none. If the firmware does not offer S3, turning Modern Standby off can leave the machine with no working sleep state (only hibernate or shut down), and lid-close behaviour can go wrong. That is why the tweak refuses to apply unless `powercfg /a` lists Standby (S3) as available. Note the limit of that check: on a machine currently running Modern Standby, Windows typically lists S3 among the unavailable states with a reason line beneath it, and the research machine (an S3 platform) could not demonstrate an S0-to-S3 switch, so the research does not establish whether the check ever passes on a machine that is actually using Modern Standby today.

#### Benefits
- Real deep sleep: no background activity while asleep, so no warm laptop and no drained battery in a bag.
- Verifiable afterwards with `powercfg /a`.
- Reversible: deleting the value restores Modern Standby at the next boot.
- The apply is refused, not just warned about, when `powercfg /a` does not show S3 as available.

#### Drawbacks
- Catastrophic on hardware without S3 (the precondition exists to stop this).
- Loses instant-on and connected standby: no notifications, mail sync or background updates while asleep.
- Undocumented and unsupported: Microsoft says switching the power model is not supported without a reinstall.
- Not always better: Microsoft's guidance is to measure idle power first, because S3 can draw more on some platforms.
- The precondition may refuse on machines currently using Modern Standby, where the tweak would matter most (see How it works).

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 including LTSC 2021, all editions. The real gate is firmware, not Windows version.
- **Takes effect**: after a reboot, when the kernel power manager chooses the power model.
- **Reverting**: "Modern Standby" deletes the value (the stock state); the precondition does not run. Restore Snapshot puts back the captured value. Takes effect at the next boot.

#### Interactions
- [Disable hibernation](#disable-hibernation): on a Modern Standby machine, hibernate-after-standby is the protection against drain; if S3 cannot be forced, keep hibernation.
- [Disable NIC power management](#disable-nic-power-management) and [Disable wake timers](#disable-wake-timers) change other sleep and wake behaviour; independent.

#### Validation
- **Verdict**: VERIFIED (community-corroborated, not Microsoft-documented). No registry defect; the research required the `powercfg /a` precondition to be enforced rather than advisory, and required Microsoft's "not supported without a complete OS re-install" statement to be carried. The shipped tweak enforces the precondition.
- **Confidence**: Community-corroborated (four or more independent tier C sources), plus direct binary evidence that only the 26100 kernel carries the value name.
- **Reasoning**: the adversarial pass attacked the absence of any Microsoft documentation; the binary sweep confirmed the kernel reads the name, and independent sources agree on semantics. The measured machine reported S3 available and S0 unavailable with the value absent, consistent with the stock default, but as an S3 platform it could not demonstrate the switch. Open question: whether the enforced precondition passes on real Modern Standby hardware with S3-capable firmware.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Only for users who have confirmed with `powercfg /a` that Standby (S3) is available and who are actually fighting Modern Standby drain. If S3 is not listed as available, do not try to work around the refusal; you risk a machine that cannot sleep or wake properly.

#### Sources
1. What is Modern Standby, including "Switching the power model is not supported in Windows without a complete OS re-install", https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby (tier A)
2. Modern Standby vs S3, what S0 Low Power Idle and S3 are and that a platform supports one, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/modern-standby-vs-s3 (tier A)
3. Direct binary inspection on Windows 11 24H2 build 26100.4061 (July 2026 research): `PlatformAoAcOverride` only in `ntoskrnl.exe` and `ntkrla57.exe` out of 5,418 `System32` binaries; value absent; `powercfg /a` reports S3 available and S0 Low Power Idle unavailable (primary measurement)
4. How to disable Modern Standby on Windows (Pureinfotech), key, name, type, value 0 and the `powercfg /a` check, https://pureinfotech.com/disable-modern-standby-windows/ (tier C)
5. Disable Modern Standby in Windows 10 and Windows 11 (Eleven Forum tutorial), same key, name, type and value; further corroborated by MakeUseOf and WinBuzzer, https://www.elevenforum.com/t/disable-modern-standby-in-windows-10-and-windows-11.3929/ (tier C)

## Considered and not shipped

### Windows Update controls (moved or retired)

The July 2026 research for this category also covered thirteen Windows Update tweaks. Twelve of them moved to their own category: `block_insider_builds_policy`, `block_update_over_metered`, `defer_feature_updates`, `defer_quality_updates`, `disable_auto_driver_install`, `disable_auto_restart_logged_on`, `disable_delivery_optimization_p2p`, `disable_store_auto_updates`, `exclude_wu_driver_updates`, `set_active_hours`, `target_release_version` and `update_feature_control`. The thirteenth, `disable_auto_update_download`, is retired and replaced by `windows_update_mode`, a four-option control over the same `NoAutoUpdate` and `AUOptions` values. All of them are documented on the [Windows Update](windows_update.md) page.
