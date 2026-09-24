# Services & Scheduled Tasks tweaks

This category sets the start type of optional Windows services and the enabled state of telemetry-related scheduled tasks, to cut background activity and shrink attack surface. It contains 23 service tweaks and 6 scheduled-task tweaks. The supported platform is Windows 11 24H2 (build 26100) and newer, including 25H2 (build 26200), as the primary target, and Windows 10 IoT Enterprise LTSC 2021 (build 19044) as the secondary target; three tweaks (Fax, Touch Keyboard, AllJoyn Router) exist only on Windows 10 because their services are absent from the primary platform.

Two engine facts apply to every entry on this page. First, a service effect changes only the start type, through the Service Control Manager (`ChangeServiceConfigW`) plus an explicit write of the `DelayedAutostart` companion value; the app never starts or stops a service, so every service tweak takes effect at the next reboot, and a service that is running at apply time keeps running until then. Setting a service to Disabled (`Start=4`) also overrides any trigger registration, so a trigger-started service genuinely cannot start. Second, a task effect enables or disables a scheduled task through the Task Scheduler COM service and takes effect immediately for future runs; the app never creates or deletes tasks.

**How "System Default" works here.** System Default is not an option you pick from a list of authored values: it is the status the app computes when the live service start type or task state matches none of the tweak's options. Selecting it restores the state captured in the tweak's snapshot. Because most service tweaks here author the stock start type as one of their options (for example "Manual"), a stock machine usually reads as that option, not as System Default; System Default appears when the service sits at a start type no option names. Each entry states the stock start type on 24H2 so you can tell which row a stock machine lands on.

**About the dependency lines.** The "depends on" and "required by" lists in each entry were read from the Service Control Manager on Windows 11 IoT Enterprise LTSC 2024, build 26100. Dependency registrations are part of a service's definition and are not changed by start-type changes, so they are reliable even though that machine's start types are not stock. Stock start types come from the July 2026 research, not from that machine.

## Index

| Tweak | Id | Control | Risk | Elevation | Reboot | Verdict |
|---|---|---|---|---|---|---|
| [Disable User Experiences and Telemetry (DiagTrack)](#disable-user-experiences-and-telemetry-diagtrack) | `disable_diagtrack` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Print Spooler (Spooler)](#disable-print-spooler-spooler) | `disable_print_spooler` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Fax Service (Fax)](#disable-fax-service-fax) | `disable_fax` | Switch (2 options) | low | admin | yes | INCORRECT on the primary platform (Windows 10 only) |
| [Program Compatibility Assistant Service (PcaSvc)](#program-compatibility-assistant-service-pcasvc) | `disable_program_compat_assistant` | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Distributed Link Tracking Client (TrkWks)](#disable-distributed-link-tracking-client-trkwks) | `disable_distributed_link_tracking` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Disable Retail Demo Service (RetailDemo)](#disable-retail-demo-service-retaildemo) | `disable_retail_demo` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Wallet Service (WalletService)](#disable-wallet-service-walletservice) | `disable_wallet_service` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Touch Keyboard Service (TabletInputService)](#disable-touch-keyboard-service-tabletinputservice) | `disable_touch_keyboard` | Switch (2 options) | medium | admin | yes | INCORRECT on the primary platform (Windows 10 only) |
| [Disable Bluetooth Support Services (bthserv)](#disable-bluetooth-support-services-bthserv) | `disable_bluetooth` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable AllJoyn Router Service (AJRouter)](#disable-alljoyn-router-service-ajrouter) | `disable_alljoyn_router` | Switch (2 options) | low | admin | yes | INCORRECT on the primary platform (Windows 10 only) |
| [Disable Phone Service (PhoneSvc)](#disable-phone-service-phonesvc) | `disable_phone_service` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Network Device Discovery (SSDPSRV/upnphost)](#disable-network-device-discovery-ssdpsrvupnphost) | `disable_ssdp_upnp` | Switch (2 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Windows Insider Service (wisvc)](#disable-windows-insider-service-wisvc) | `disable_windows_insider` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Downloaded Maps Manager (MapsBroker)](#disable-downloaded-maps-manager-mapsbroker) | `disable_maps_broker` | Switch (2 options) | low | admin | yes | VERIFIED |
| [Disable Geolocation Service (lfsvc)](#disable-geolocation-service-lfsvc) | `disable_geolocation` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Xbox Live Services (XblAuthManager et al.)](#xbox-live-services-xblauthmanager-et-al) | `disable_xbox_services` | Dropdown (3 options) | medium | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Connected Devices Platform Service (CDPSvc)](#disable-connected-devices-platform-service-cdpsvc) | `disable_cdpsvc` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Windows Biometric Service (WbioSrvc)](#disable-windows-biometric-service-wbiosrvc) | `disable_biometrics` | Switch (2 options) | medium | admin | yes | VERIFIED |
| [Disable Smart Card Services (SCardSvr)](#disable-smart-card-services-scardsvr) | `disable_smartcard` | Switch (2 options) | high | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Sensor Services (SensorService)](#disable-sensor-services-sensorservice) | `disable_sensor_services` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Family Safety Monitor (WpcMonSvc)](#disable-family-safety-monitor-wpcmonsvc) | `disable_parental_controls` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Payments and NFC/SE Manager (SEMgrSvc)](#disable-payments-and-nfcse-manager-semgrsvc) | `disable_payments_nfc` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Media Player Network Sharing (WMPNetworkSvc)](#disable-media-player-network-sharing-wmpnetworksvc) | `disable_wmp_network_sharing` | Switch (2 options) | low | admin | yes | VERIFIED-WITH-CORRECTION |
| [Disable Autochk Proxy task](#disable-autochk-proxy-task) | `task_autochk_proxy` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Feedback (SIUF) DmClient tasks](#disable-feedback-siuf-dmclient-tasks) | `task_feedback_dmclient` | Switch (2 options) | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable Windows Error Reporting QueueReporting task](#disable-windows-error-reporting-queuereporting-task) | `task_wer_queuereporting` | Switch (2 options) | low | admin | no | VERIFIED |
| [Disable Offline Maps update tasks](#disable-offline-maps-update-tasks) | `task_maps_update` | Switch | low | admin | no | VERIFIED-WITH-CORRECTION |
| [Disable DiskDiagnosticDataCollector task](#disable-diskdiagnosticdatacollector-task) | `task_disk_diagnostic_datacollector` | Switch | low | admin | no | INCORRECT (no trigger; stock state unestablished) |
| [Disable Device Census tasks](#disable-device-census-tasks) | `task_device_census` | Switch (2 options) | low | admin | no | INCORRECT (corrected form ships) |

## Tweaks

### Disable User Experiences and Telemetry (DiagTrack)

`disable_diagtrack` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops the service that collects Windows diagnostic and usage data and uploads it to Microsoft.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `diagtrack` | service | `DiagTrack` (display name "Connected User Experiences and Telemetry") | none |

| Option | `diagtrack` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Automatic | `automatic` (`Start=2`, `DelayedAutostart=0`) |

System Default: the status shown when `DiagTrack` is at any start type other than Disabled or plain Automatic (for example Manual, or Automatic (Delayed Start)); selecting it restores the start type captured in the snapshot. The stock start type on 24H2 is **Automatic**, not delayed and not trigger-started, so a stock machine reads as the "Automatic" option. Depends on: `RpcSs`. Required by: no other service. Features that rely on it: Feedback Hub diagnostic submission, Microsoft Defender for Endpoint, Windows Update for Business reports and Update Compliance.

#### How it works

`DiagTrack` is the Connected User Experiences and Telemetry component, hosted in the `utcsvc` svchost group as LocalSystem. Microsoft documents it as the component that manages diagnostic data events and diagnostic logs sent back to Microsoft. Disabling the service stops the collection and upload pipeline itself, rather than lowering the amount of data the pipeline is allowed to send.

That distinction matters by edition. The diagnostic data policy (`AllowTelemetry`) can cap the level, but Microsoft states that the "Diagnostic data off (Security)" level is available only on Enterprise, Education, IoT and Server editions; on Home and Pro the lowest effective level is Required. On those editions disabling the service is the only lever that stops the transmitter.

The engine writes the start type through the Service Control Manager and never stops the running service, so the pipeline keeps running until the next reboot. Feature updates re-provision service configuration and are widely reported to put `DiagTrack` back to Automatic.

#### Benefits
- **Stops the transmitter**: the pipeline does not run at all, rather than running at a lower level.
- **Works on every edition**: Home and Pro cannot reach "Diagnostic data off" through policy; this reaches the same end by stopping the service.
- **Quieter machine**: removes the background network and disk activity the uploader generates.

#### Drawbacks
- **Feedback Hub breaks**: diagnostic submission through Feedback Hub stops working.
- **Enterprise signals stop**: Defender for Endpoint, Windows Update for Business reports and Update Compliance all read this pipeline.
- **Updates can undo it**: feature updates are widely reported to restore the service to Automatic.
- **No speed gain**: the benefit is privacy, not performance.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021, all editions.
- **Takes effect**: at the next reboot, because changing a start type never stops an already running service.
- **Reverting**: choosing "Automatic" writes the stock start type; System Default restores whatever start type the snapshot captured. A feature update may also re-enable it on its own.

#### Interactions
- **Privacy: Limit diagnostic data to the Required level** (`disable_diagnostic_data`) caps the level by policy; it cannot stop the transmitter on Home or Pro. The two are complementary.
- **Disable Feedback (SIUF) DmClient tasks** (`task_feedback_dmclient`, this page): with `DiagTrack` disabled those tasks have no working upload path.
- **Disable Device Census tasks** (`task_device_census`, this page): feeds the same enterprise reporting (Windows Update for Business reports, Update Compliance).

#### Validation
- **Verdict**: VERIFIED. The research restated the edition limit in Microsoft's own terms ("Diagnostic data off (Security)" exists only on Enterprise, Education, IoT and Server, so Home and Pro bottom out at Required); the mechanism, stock start type and reboot requirement were correct.
- **Confidence**: Microsoft-documented. The component's role and the edition limit come from Microsoft's diagnostic data documentation; the start type is corroborated by Microsoft's Server service guidance (always installed, Automatic, "No guidance") and by two clean-install dumps.
- **Reasoning**: Service name, stock start type (Automatic on every release checked) and the effect of disabling were all confirmed at tier A. The only claim attacked was the phrasing of the policy floor, which survived once restated.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a personal machine where privacy matters more than Feedback Hub. Do not apply it on a machine managed by an IT department or enrolled in Defender for Endpoint, where the diagnostic pipeline is load-bearing.

#### Sources
1. Configure Windows diagnostic data in your organization, the component's role and the edition limit on "Diagnostic data off", https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
2. Manage connections from Windows operating system components to Microsoft services, the connection endpoints this component uses, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Guidelines for disabling system services in Windows Server with Desktop Experience, `DiagTrack` always installed, Automatic, "No guidance", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
4. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, stock start type, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)

### Disable Print Spooler (Spooler)

`disable_print_spooler` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes all printing and shuts down the component repeatedly hit by remote-code-execution bugs.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `spooler` | service | `Spooler` (display name "Print Spooler") | none |

| Option | `spooler` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Automatic | `automatic` (`Start=2`, `DelayedAutostart=0`) |

System Default: the status shown when `Spooler` is at a start type other than Disabled or plain Automatic; selecting it restores the snapshot. The stock start type on 24H2 is **Automatic**, so a stock machine reads as the "Automatic" option. Depends on: `RPCSS`, `http`. Required by: no other service. Features that rely on it: every print path, including Microsoft Print to PDF, Microsoft XPS Document Writer, printer enumeration in application print dialogs, and print-based export in label, receipt and some scanner software.

#### How it works

The Print Spooler (`%WinDir%\System32\spoolsv.exe`, LocalSystem, its own process) queues every print job and mediates every interaction with printer drivers and print devices. With it disabled, Windows cannot talk to any printer, physical or virtual. There is no partial mode: virtual printers such as Print to PDF are spooler clients like any other.

The spooler has produced a sustained run of remote-code-execution and privilege-escalation flaws (PrintNightmare among them). Microsoft lists disabling the service as a documented mitigation, and its Server guidance rates it "OK to disable if not a print server or a domain controller". Disabling removes the whole class of surface rather than mitigating one bug, but it is not a substitute for installing security updates.

#### Benefits
- **Removes an attack surface**: the spooler has a long history of RCE and privilege-escalation bugs.
- **Kills the whole class**: no spooler process means no spooler bug can be reached.
- **One less boot service**: an Automatic-start service stops loading at boot.

#### Drawbacks
- **All printing breaks**: including Microsoft Print to PDF and Print to XPS.
- **Printers vanish from apps**: printer lists disappear from every print dialog.
- **Export flows break**: label and receipt software, some PDF export paths and some scanner suites route through the spooler.
- **Not a patch substitute**: a documented mitigation, not a replacement for updates.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021, all editions. Unsupported by Microsoft on a domain controller or a print server.
- **Takes effect**: at the next reboot; the spooler keeps running (and printing keeps working) until then.
- **Reverting**: choosing "Automatic" writes the stock start type; System Default restores the snapshot's captured start type.

#### Interactions
- **Security: Turn off the spooler's remote RPC endpoint** (`spooler_remote_rpc_off`) and **Security: Restrict printer-driver install to admins** (`printnightmare_point_and_print`) harden a spooler that keeps running. They are the middle ground for a machine that still prints; with this tweak applied they have nothing left to protect.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented. Microsoft's Server guidance gives the default and the stance; KB5005010 documents the driver-installation hardening context.
- **Reasoning**: The service name, the stock Automatic start type and the total loss of printing were all confirmed; the warning text was found accurate.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a machine that genuinely never prints, such as a kiosk, a server, or a hardened workstation. Do not apply it if you ever print or export to PDF through the print dialog.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `Spooler` Automatic, "OK to disable if not a print server or a domain controller", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. KB5005010: Restricting installation of new printer drivers after applying the July 6, 2021 updates, the PrintNightmare hardening context, https://support.microsoft.com/en-us/topic/kb5005010-restricting-installation-of-new-printer-drivers-after-applying-the-july-6-2021-updates-31b91c02-05bc-4ada-a7ea-183b129578a7 (tier A)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, stock start type, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)

### Disable Fax Service (Fax)

`disable_fax` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: `products: [10]` (Windows 10 builds 10240 to 19045 only) · Reversible: yes

**Turns off the legacy fax service on a Windows 10 machine that has no fax hardware.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `FaxService`, `REG_DWORD` (app marker) | none |
| `fax` | service | `Fax` (display name "Fax") | `optional: true`, `if_missing: disabled` |

| Option | `state` | `fax` |
|---|---|---|
| Disabled | `1` | `disabled` (`Start=4`) |
| Manual | `0` | `manual` (`Start=3`) |

System Default: shown until you pick an option, because the app's own marker value (`FaxService`) does not exist on a machine where the tweak was never applied; selecting it restores the snapshot, which deletes the marker and puts the service back to its captured start type. The stock start type on Windows 10 is **Manual**. On Windows 11 22H2 and newer, which includes the whole primary platform, the service does not exist, and its `windows` gate shows the tweak as Unavailable there, with the reason. Depends on / required by: not observed (the service is absent from the build 26100 reference machine); no Windows feature other than Windows Fax and Scan uses it.

#### How it works

The Fax service (`%WinDir%\system32\fxssvc.exe`, NetworkService, its own process) sends and receives faxes through a fax modem or a network fax device, and it is what Windows Fax and Scan talks to. Scanning through other applications does not go through it. On Windows 10 the service is carried by the Windows Fax and Scan optional feature rather than by the base OS; Windows Fax and Scan stopped being installed by default from Windows 11 22H2.

Two authoring details make the tweak behave on every Windows 10 image. The `fax` effect is `optional` with `if_missing: disabled`, so on an image without the feature the service reads as Disabled rather than making the tweak Unknown: applying "Disabled" is then a verified no-op, and "Manual" is shown as unavailable on this machine because the engine never installs services. The `state` marker is a non-optional value under the app's own `HKCU` key that keeps both options detectable even when the service is missing. The marker is per user, so another account on the same machine sees System Default until it applies an option.

#### Benefits
- **Removes a legacy service**: nothing on a modern PC uses fax.
- **Smaller service list**: one less entry to audit on a hardened machine.
- **No side effects**: scanning, printing and imaging are unaffected.

#### Drawbacks
- **Faxing stops**: sending and receiving through Windows Fax and Scan no longer works.
- **Windows 10 only**: the service is absent from Windows 11 22H2 and newer, so the tweak is not offered there.
- **Depends on an optional feature**: if Windows Fax and Scan was never installed, there is no service to change.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 (the `products: [10]` gate), which for this app means Windows 10 IoT Enterprise LTSC 2021. Not offered on Windows 11.
- **Takes effect**: at the next reboot, because changing a start type never stops an already running service.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot (marker removed, captured start type restored).

#### Interactions
None known.

#### Validation
- **Verdict**: INCORRECT on the primary platform, per the July 2026 research: `Fax` is absent from Windows 11 22H2 onward, so the tweak has no supported platform on 24H2. The tweak as shipped is gated to Windows 10 and its service effect is optional, which is the scope the research called for.
- **Confidence**: Community-corroborated. No Microsoft source gives the client default; two clean-install dumps and a per-release table agree (Manual on Windows 10 and 21H2; "not exists" on 22H2, 23H2 and 24H2).
- **Reasoning**: The research showed that an ungated, non-optional effect on an absent service makes the tile Unknown and refuses to apply, rather than doing nothing. The gate plus `optional`/`if_missing` plus the marker avoid that. The Windows 10 default comes from Windows 10 22H2 dumps and the per-release tables.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a Windows 10 LTSC machine with no fax hardware, where it costs nothing. On Windows 11 there is nothing to disable and its `windows` gate shows the tweak as Unavailable, with the reason.

#### Sources
1. Windows 10 Default Services Configuration, Windows 10 Pro 22H2 clean install, `Fax` = Manual, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
2. Windows 11 Default Services Configuration, Windows 11 Pro 23H2 clean install, `Fax` absent, https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Fax, Windows 11 service reference, 21H2 Manual and 22H2, 23H2, 24H2 "not exists", https://batcmd.com/windows/11/services/fax/ (tier C)
4. `src-tauri/src/tweaks/kinds/service.rs`, a missing service yields `ResourceMissing` on drive and `Missing` on read (tier A, this repository)

### Program Compatibility Assistant Service (PcaSvc)

`disable_program_compat_assistant` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops the compatibility pop-ups and the background monitoring of every program launch.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `pcasvc` | service | `PcaSvc` (display name "Program Compatibility Assistant Service") | none |

| Option | `pcasvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Automatic (Delayed) | `automatic_delayed` (`Start=2`, `DelayedAutostart=1`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `PcaSvc` is at a start type none of the three options names (for example plain Automatic without the delayed flag); selecting it restores the snapshot. The stock start type is release-dependent: **Automatic (Delayed Start, Trigger Start)** on Windows 11 24H2, so a stock 24H2 machine reads as "Automatic (Delayed)"; **Manual (Trigger Start)** on Windows 10, so a stock LTSC 2021 machine reads as "Manual". Depends on: `RpcSs`. Required by: no other service. Features that rely on it: automatic compatibility shims and PCA prompts for legacy programs.

#### How it works

PCA (`%WinDir%\System32\pcasvc.dll`, hosted in the `LocalSystemNetworkRestricted` svchost group as LocalSystem) watches programs as they install and run, detects known compatibility problems, applies automatic compatibility shims, and raises prompts such as "This app may not have installed correctly". It is trigger-started as well as delayed, so it starts itself on demand.

The option list exists because the engine always writes the `DelayedAutostart` companion explicitly. A plain `automatic` option would write `DelayedAutostart=0` and move `PcaSvc` from the post-boot queue into the boot start path, a state a stock 24H2 machine was never in. So the 24H2 stock state is authored as `automatic_delayed`, and the Windows 10 stock state as `manual`; pick the one that matches your build.

On 24H2 the `\Microsoft\Windows\Application Experience\PcaPatchDbTask` scheduled task maintains the compatibility database PCA consumes. The research found that task's existence supports the idea that newer builds lean on this path, but found no Microsoft statement saying so.

#### Benefits
- **No compatibility prompts**: the "may not have installed correctly" dialog stops appearing.
- **No launch monitoring**: PCA stops inspecting each program start.
- **Microsoft rates it optional**: the Server guidance marks this service "OK to disable".

#### Drawbacks
- **Shims stop applying**: legacy programs that relied on automatic fixes may misbehave.
- **Hard to attribute**: the failure is invisible until an old program breaks, and the cause is not obvious.
- **Removes live behaviour**: the service is trigger-started, so it does run today.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: choose "Automatic (Delayed)" on Windows 11 or "Manual" on Windows 10 to return to stock, or select System Default to restore the snapshot, which includes the delayed-start flag.

#### Interactions
- **Privacy: Disable Compatibility Appraiser tasks** (`disable_compat_appraiser`) and **Privacy: Disable app and device inventory collectors** (`disable_app_device_inventory`) act on the same Application Experience family (appraiser tasks and AppCompat policy values). They do not touch `PcaSvc` itself, and this tweak does not touch them.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the 24H2 stock state is Automatic with the delayed flag set (`DelayedAutostart=1`) and Manual on Windows 10, and that a plain `automatic` value would strip the delayed flag. The shipped options author both stock states.
- **Confidence**: Microsoft-documented for the service and Microsoft's "OK to disable" stance; the per-release default comes from clean-install dumps and per-release tables (tier C).
- **Reasoning**: The engine's explicit `DelayedAutostart` write was confirmed in `service.rs`, which is why the delayed variant matters. Open: the Windows 10 Manual (Trigger Start) default was sourced from Windows 10 22H2 dumps, not from an LTSC 2021 image; LTSC 2021 shares the 21H2 baseline, where the tables agree.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if the prompts annoy you and you do not run finicky legacy software. Leave it enabled if you depend on old applications, since automatic shims are what keep some of them working.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `PcaSvc` Desktop Experience only, Automatic, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 11 Default Services Configuration, Windows 11 Pro 23H2 clean install, `PcaSvc` = Auto (Delayed, Triggered), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Windows 10 Default Services Configuration, Windows 10 Pro 22H2 clean install, `PcaSvc` = Manual, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
4. Program Compatibility Assistant Service per-release tables, https://batcmd.com/windows/10/services/pcasvc/ and https://batcmd.com/windows/11/services/pcasvc/ (tier C)
5. `src-tauri/src/tweaks/kinds/service.rs`, `drive_service` always writes `DelayedAutostart` explicitly (tier A, this repository)

### Disable Distributed Link Tracking Client (TrkWks)

`disable_distributed_link_tracking` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the service that repairs shortcuts and document links when files move between NTFS drives.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `trkwks` | service | `TrkWks` (display name "Distributed Link Tracking Client") | none |

| Option | `trkwks` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Automatic | `automatic` (`Start=2`, `DelayedAutostart=0`) |

System Default: the status shown when `TrkWks` is at a start type other than Disabled or plain Automatic; selecting it restores the snapshot. The stock start type on 24H2 is **Automatic**, not delayed and not trigger-started, so a stock machine reads as the "Automatic" option and the service really is running. Depends on: `RpcSs`. Required by: no other service. Features that rely on it: automatic repair of shell shortcuts and embedded OLE links whose target moved.

#### How it works

The service tracks the NTFS object identifier on a file, so that shortcuts and embedded OLE links can be repaired when the target is moved or renamed within or across NTFS volumes on the same machine. The repair scenario needs a file to move across NTFS volumes, which most people never do. With the service disabled, a shortcut whose target moved simply fails to find it; the fix is easy by hand but hard to connect to this change.

Microsoft's Server guidance lists the service as Desktop Experience only, Automatic, and gives "No guidance". The case for disabling rests on the absence of a dependency, not on a Microsoft recommendation.

#### Benefits
- **One less boot service**: it is Automatic, so it loads on every start.
- **Rarely exercised**: the repair path needs cross-volume moves.
- **No dependency**: nothing else in Windows requires this service.

#### Drawbacks
- **Links stop self-repairing**: a shortcut whose target moved no longer finds it.
- **Confusing failure**: the broken shortcut is hard to connect to this change.
- **OLE links affected**: embedded links in documents lose the same repair.
- **No Microsoft endorsement**: the Server guidance gives "No guidance".

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Automatic" writes the stock start type; System Default restores the snapshot.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Microsoft-documented for the default (Server guidance), corroborated by clean-install dumps and per-release tables.
- **Reasoning**: The stock Automatic start type, the absence of a dependent service and the reboot requirement were all confirmed. The research asked that the copy not imply Microsoft recommends disabling it, since Microsoft gives no guidance.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a single-drive machine or any system where shortcuts point at fixed locations. Leave it enabled if you move files between NTFS volumes and rely on shortcuts or linked documents following them.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `TrkWks` Desktop Experience only, Automatic, "No guidance", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Distributed Link Tracking Client per-release tables, https://batcmd.com/windows/10/services/trkwks/ and https://batcmd.com/windows/11/services/trkwks/ (tier C)

### Disable Retail Demo Service (RetailDemo)

`disable_retail_demo` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the in-store demonstration mode that only ever runs on shop display units.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `retaildemo` | service | `RetailDemo` (display name "Retail Demo Service") | none |

| Option | `retaildemo` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `RetailDemo` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual**, not trigger-started, so a stock machine reads as the "Manual" option. Depends on: nothing. Required by: no other service. Features that rely on it: Retail Demo Experience only.

#### How it works

The service supports Retail Demo Experience, the scripted demonstration mode a retailer puts on a display machine. On a PC that has never been placed into demo mode it has nothing to do, and as a plain Manual service with no trigger it does not start. Disabling it is a tidy-up: it removes an entry from the set of things that could start, with no resource saving to measure. Microsoft's Server guidance does not cover it, which is expected for a client-only feature.

#### Benefits
- **No purpose on your PC**: the feature only applies to shop display units.
- **Shorter service list**: one fewer entry when auditing what can start.
- **Fully reversible**: nothing else depends on it.

#### Drawbacks
- **Retail demo stops**: a genuine display unit would lose its demonstration mode.
- **No measurable gain**: a Manual service that is never triggered consumes nothing.
- **Nothing visible changes**: there is no user-facing difference to look for.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The only correction was that the tweak needs the reboot flag, because the engine never stops a running service; the service name, options and stock start type were correct. The shipped tweak carries the flag.
- **Confidence**: Community-corroborated. No Microsoft source covers this client-only service; clean-install dumps and two per-release reference sites agree on Manual.
- **Reasoning**: Three independent tier C sources agree on Manual and no trigger; nothing in the mechanism was disputed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any personally owned machine; no home or work PC needs Retail Demo. Skip it only on an actual in-store display unit.

#### Sources
1. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, `RetailDemo` = Manual, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
2. Retail Demo Service per-release tables, https://batcmd.com/windows/10/services/retaildemo/ and https://batcmd.com/windows/11/services/retaildemo/ (tier C)
3. Retail Demo Service (RetailDemo) defaults, https://revertservice.com/10/retaildemo/ and https://revertservice.com/11/retaildemo/ (tier C)

### Disable Wallet Service (WalletService)

`disable_wallet_service` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the service behind Microsoft Wallet and tap-to-pay from Windows.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `walletservice` | service | `WalletService` (display name "WalletService") | none |

| Option | `walletservice` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `WalletService` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual**, so a stock machine reads as the "Manual" option. Depends on: nothing. Required by: no other service. Features that rely on it: Microsoft Wallet and tap-to-pay payments from Windows.

#### How it works

`WalletService` is hosted in the `appmodel` svchost group as LocalSystem. It backs the Windows wallet and tap-to-pay payment features on hardware that supports them; it is the front end of the Windows payment stack, and the NFC secure element behind it is managed by the separate `SEMgrSvc` service. It has nothing to do with card details saved in a web browser, which keep working. Microsoft's Server guidance lists it as Desktop Experience only, Manual, "OK to disable".

#### Benefits
- **Microsoft rates it safe**: the Server guidance marks it "OK to disable".
- **No hardware to serve**: a desktop with no payment hardware gets nothing from it.
- **Rarely used feature**: paying from a Windows PC is uncommon.

#### Drawbacks
- **Wallet payments stop**: Microsoft Wallet and tap-to-pay from this device no longer work.
- **Half the stack**: the NFC secure element is a separate service, which this tweak leaves running.
- **Nothing visible changes**: on a machine with no payment hardware there is no observable difference.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
- **Disable Payments and NFC/SE Manager (SEMgrSvc)** (`disable_payments_nfc`, this page) is the other half of the same payment stack. The research proposed merging the two into one tweak, since no user sensibly wants one without the other; they ship as two tweaks, so apply both for the full effect.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The only correction was the reboot flag (the engine never stops a running service); the shipped tweak carries it.
- **Confidence**: Microsoft-documented (Server guidance: Manual, "OK to disable"), corroborated by clean-install dumps.
- **Reasoning**: Service name, default and Microsoft's stance all agree across tiers.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine where you do not pay from Windows, which is nearly every desktop. Leave it enabled if you use Microsoft Wallet or tap-to-pay on a laptop or tablet with NFC.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `WalletService` Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. WalletService per-release table, https://batcmd.com/windows/11/services/walletservice/ (tier C)

### Disable Touch Keyboard Service (TabletInputService)

`disable_touch_keyboard` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: `products: [10]` (Windows 10 builds 10240 to 19045 only) · Reversible: yes

**Turns off the Windows 10 touch keyboard, pen input and emoji panel on a desktop that only uses a physical keyboard.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `TouchKeyboardService`, `REG_DWORD` (app marker) | none |
| `tabletinputservice` | service | `TabletInputService` (display name "Touch Keyboard and Handwriting Panel Service") | `optional: true`, `if_missing: disabled` |

| Option | `state` | `tabletinputservice` |
|---|---|---|
| Disabled | `1` | `disabled` (`Start=4`) |
| Manual | `0` | `manual` (`Start=3`) |

System Default: shown until you pick an option, because the marker value does not exist on a machine where the tweak was never applied; selecting it restores the snapshot. The stock start type on Windows 10 is **Manual (Trigger Start)**. The service is absent on Windows 11 22H2, 23H2 and 24H2 in all editions, and its `windows` gate shows the tweak as Unavailable there, with the reason. Depends on / required by: not observed (absent from the build 26100 reference machine). Features that rely on it on Windows 10: the on-screen touch keyboard, pen and handwriting input, and the emoji and symbol panel (Win and period).

#### How it works

On Windows 10, `TabletInputService` (`%WinDir%\System32\TabSvc.dll`, hosted in the `LocalSystemNetworkRestricted` svchost group, member of the `PlugPlay` group) provides the touch keyboard, the handwriting and ink panel, and the emoji and symbol panel. From Windows 11 22H2 those surfaces are served by the Windows Input Experience host (`TextInputHost.exe`) instead, and the service does not exist, so none of the breakage described here applies on Windows 11.

The service is trigger-started, so on a Windows 10 machine with no touch or pen it is usually not running anyway and costs nothing while idle. Microsoft's own Server guidance rates it **"Do not disable"**.

The `optional`/`if_missing` pair and the `state` marker work as described for the Fax tweak: an image without the service reads as Disabled, "Manual" becomes unavailable there, and the per-user marker keeps both options detectable.

#### Benefits
- **Unused on a desktop**: a keyboard-and-mouse machine never invokes these surfaces.
- **One less service**: removes an input-stack service from the start path.

#### Drawbacks
- **Emoji picker breaks**: Win and period stops opening the emoji and symbol panel.
- **Touch keyboard breaks**: the on-screen keyboard and the handwriting and ink panel stop working.
- **Microsoft says do not**: the Server guidance rates this service "Do not disable".
- **Costs nothing when idle**: a trigger-started service with no touch or pen hardware is not running anyway.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 (the `products: [10]` gate), which for this app means Windows 10 IoT Enterprise LTSC 2021. Not offered on Windows 11.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot (marker removed, captured start type restored).

#### Interactions
None known.

#### Validation
- **Verdict**: INCORRECT on the primary platform, per the July 2026 research: `TabletInputService` does not exist on Windows 11 22H2 and newer, where these input surfaces come from `TextInputHost.exe`, so the Win and period warning cannot apply there. The tweak as shipped is gated to Windows 10, uses an optional service effect, scopes its warning to Windows 10 and states Microsoft's "Do not disable" rating.
- **Confidence**: Microsoft-documented for Microsoft's stance (Server guidance); absence on Windows 11 is corroborated by a Microsoft Q&A thread and two tier C sources.
- **Reasoning**: The research treated Microsoft's "Do not disable" rating as a tier A signal against the tweak even where it applies; the entry keeps it and the recommendation follows it.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Skip this one. Microsoft rates the service "Do not disable", a trigger-started service costs nothing when idle, and the emoji picker is a real loss for a small gain. If you want it anyway, apply it only on a Windows 10 desktop with no touch or pen and no use of Win and period.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `TabletInputService` Manual, "Do not disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Microsoft Q&A, "The Touch Keyboard and Handwriting Panel Service is missing when I installed Windows 11", https://learn.microsoft.com/en-us/answers/questions/4147866/the-touch-keyboard-and-handwriting-panel-service-i (tier A host, community-authored content)
3. Windows 11 Default Services Configuration, Windows 11 Pro 23H2 clean install, `TabletInputService` absent, https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Touch Keyboard and Handwriting Panel Service, Windows 11 per-release table, 21H2 Manual and 22H2, 23H2, 24H2 "not exists", https://batcmd.com/windows/11/services/tabletinputservice/ (tier C)
5. Windows 10 Default Services Configuration, Windows 10 Pro 22H2 clean install, Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)

### Disable Bluetooth Support Services (bthserv)

`disable_bluetooth` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Shuts down the whole Windows Bluetooth stack on a machine that has no Bluetooth hardware.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `bthserv` | service | `bthserv` (display name "Bluetooth Support Service") | none |
| `btagservice` | service | `BTAGService` (display name "Bluetooth Audio Gateway Service") | none |
| `bthavctpsvc` | service | `BthAvctpSvc` (display name "AVCTP service") | none |

| Option | `bthserv` | `btagservice` | `bthavctpsvc` |
|---|---|---|---|
| Disabled | `disabled` | `disabled` | `disabled` |
| Manual | `manual` | `manual` | `manual` |

System Default: the status shown when the three services do not all match one option (for example one of them Disabled and the others Manual); selecting it restores the snapshot. The stock start type on 24H2 is **Manual (Trigger Start)** for all three, so a stock machine reads as the "Manual" option. Dependencies: `bthserv` depends on nothing and is required by the per-user `BluetoothUserService` (so that per-user service cannot start while `bthserv` is disabled); `BTAGService` and `BthAvctpSvc` each depend on `RpcSs` and nothing depends on them. Features that rely on them: every Bluetooth device and function (mice, keyboards, headsets, controllers, file transfer, Bluetooth audio).

#### How it works

`bthserv` handles discovery and association of remote Bluetooth devices; `BTAGService` provides the hands-free audio gateway role; `BthAvctpSvc` carries the Audio/Video Control Transport Protocol used by Bluetooth audio and remote control. Together they are the Bluetooth stack's service layer, so disabling all three prevents it from starting at all, and grouping them avoids a half-working stack.

"Manual" here does not mean dormant. All three are trigger-started, and `bthserv` carries 3 trigger registrations on build 26100, so on a machine with a Bluetooth radio they start themselves when hardware or a pairing event appears. Disabled overrides the trigger registrations, so the stack genuinely cannot start. On a machine with no radio they never start, so the gain there is a hardening guarantee rather than a saving.

#### Benefits
- **Removes a radio surface**: nothing can bring the Bluetooth stack up.
- **Microsoft rates it safe**: the Server guidance marks `bthserv` "OK to disable".
- **Correctly grouped**: all three change together.

#### Drawbacks
- **All Bluetooth breaks**: mice, keyboards, headphones, game controllers and file transfer stop working.
- **Input lockout risk**: on a machine whose only mouse or keyboard is Bluetooth, you can be left unable to interact with it after the reboot.
- **Not dormant today**: on a machine with a radio you are removing live behaviour.
- **No performance gain**: on a machine with no radio they never start anyway.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot; Bluetooth keeps working until then.
- **Reverting**: "Manual" writes the stock start type on all three; System Default restores the snapshot. If you have no wired input device after the reboot you cannot revert from inside Windows without one.

#### Interactions
- **Xbox Live Services** (`disable_xbox_services`, this page): Xbox controllers connected over Bluetooth need this stack as well as the Xbox accessory service.

#### Validation
- **Verdict**: VERIFIED. The research asked only that the copy state the services are trigger-started (3 registrations on `bthserv`), since "Manual" reads as dormant; the mechanism, list and defaults were correct.
- **Confidence**: Microsoft-documented for `bthserv` (Server guidance: Manual, "OK to disable"); the two audio-path services are not in that guidance and rest on clean-install dumps.
- **Reasoning**: The trigger count was read from `TriggerInfo` on build 26100, which start-type changes cannot create or remove, so it survives the research's rule against using the modified machine's state.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a wired desktop with no Bluetooth radio and no wireless peripherals. Never apply it on a laptop, or on any machine with a Bluetooth mouse, keyboard, headset or controller, and confirm you have a wired input device before you reboot.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `bthserv` Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, all three Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Bluetooth Support Service per-release table, https://batcmd.com/windows/11/services/bthserv/ (tier C)
4. Trigger registration count for `bthserv` (3), read from `HKLM\SYSTEM\CurrentControlSet\Services\bthserv\TriggerInfo` on build 26100 (tier A for trigger existence only)

### Disable AllJoyn Router Service (AJRouter)

`disable_alljoyn_router` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: `products: [10]` (Windows 10 builds 10240 to 19045 only) · Reversible: yes

**Turns off the router for AllJoyn, an IoT device protocol Microsoft has retired.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `AllJoynRouter`, `REG_DWORD` (app marker) | none |
| `ajrouter` | service | `AJRouter` (display name "AllJoyn Router Service") | `optional: true`, `if_missing: disabled` |

| Option | `state` | `ajrouter` |
|---|---|---|
| Disabled | `1` | `disabled` (`Start=4`) |
| Manual | `0` | `manual` (`Start=3`) |

System Default: shown until you pick an option, because the marker value does not exist on a machine where the tweak was never applied; selecting it restores the snapshot. The stock start type on Windows 10 (and Windows 11 21H2 through 23H2) is **Manual (Trigger Start)**. The service is **absent on Windows 11 24H2 and newer**, and its `windows` gate shows the tweak as Unavailable there, with the reason. Depends on / required by: not observed (absent from the build 26100 reference machine). Features that rely on it: AllJoyn-based smart devices only.

#### How it works

AllJoyn was an open discovery and messaging protocol for Internet-of-Things devices, and `AJRouter` routed AllJoyn messages for local AllJoyn clients. Microsoft deprecated its AllJoyn implementation, explicitly including the AllJoyn Router Service, on 1 September 2023, and lists it as retired on 1 October 2024, which lines up with the 24H2 release. As a trigger-started service it can start itself on demand on Windows 10, so disabling it closes a listener path rather than declining something that never runs. Nothing in normal networking or internet use touches AllJoyn.

The optional service effect and the per-user marker behave as described for the Fax tweak.

#### Benefits
- **Retired technology**: Microsoft has removed AllJoyn from Windows.
- **Closes a trigger-started listener**: the router can no longer start itself.
- **No dependency**: normal networking does not use AllJoyn.

#### Drawbacks
- **AllJoyn devices stop**: interop with AllJoyn-based smart devices ends, which is rare but real on older setups.
- **Windows 10 only**: the service does not exist on Windows 11 24H2 or newer.
- **Nothing visible changes**: with no AllJoyn devices there is no observable difference.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 10 (the `products: [10]` gate), which for this app means Windows 10 IoT Enterprise LTSC 2021. Not offered on Windows 11.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
None known.

#### Validation
- **Verdict**: INCORRECT on the primary platform, per the July 2026 research: AllJoyn was retired on 1 October 2024 and `AJRouter` is absent on 24H2, so the tweak has no supported platform there. The tweak as shipped is gated to Windows 10 with an optional service effect and the reboot flag, which is the scope the research called for.
- **Confidence**: Microsoft-documented. The deprecation and retirement dates come from Microsoft's deprecated and removed features lists; absence on 24H2 is corroborated by a per-release table.
- **Reasoning**: Microsoft's own removal list names the router service, so absence on 24H2 is not in doubt. Microsoft's Server guidance lists `AJRouter` as Manual with "No guidance".
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a Windows 10 LTSC machine with no AllJoyn smart devices, which is almost all of them. On Windows 11 the service was removed with the protocol and its `windows` gate shows the tweak as Unavailable, with the reason.

#### Sources
1. Features and functionality removed in Windows client, AllJoyn including the AllJoyn Router Service retired 1 October 2024, https://learn.microsoft.com/en-us/windows/whats-new/removed-features (tier A)
2. Deprecated features in the Windows client, AllJoyn deprecated 1 September 2023, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
3. Guidelines for disabling system services in Windows Server with Desktop Experience, `AJRouter` Desktop Experience only, Manual, "No guidance", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
4. AllJoyn Router Service, Windows 11 per-release table, 24H2 "not exists", https://batcmd.com/windows/11/services/ajrouter/ (tier C)
5. Windows 10 Default Services Configuration, `AJRouter` = Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)

### Disable Phone Service (PhoneSvc)

`disable_phone_service` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the cellular telephony service on a machine that has no mobile modem.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `phonesvc` | service | `PhoneSvc` (display name "Phone Service") | none |

| Option | `phonesvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `PhoneSvc` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual (Trigger Start)**, so a stock machine reads as the "Manual" option. Depends on: `RpcSs`. Required by: no other service. Features that rely on it: telephony and call-state handling on hardware with a built-in cellular modem. Phone Link does not use it.

#### How it works

`PhoneSvc` manages device telephony state, meaning the cellular and phone-call capability of hardware that has a mobile modem. It is not the Phone Link app that mirrors an Android phone or iPhone, which uses a different service and is unaffected. The service is trigger-started with 3 trigger registrations on build 26100, so it starts itself when a telephony event fires; disabling it removes that behaviour rather than declining to start something dormant. Microsoft's Server guidance lists it as Desktop Experience only, Manual, "OK to disable".

#### Benefits
- **Microsoft rates it safe**: the Server guidance marks it "OK to disable".
- **No hardware to manage**: a desktop or a Wi-Fi-only laptop has no cellular radio.
- **Removes a trigger**: the service no longer starts itself on telephony events.

#### Drawbacks
- **Cellular telephony stops**: on a device with a built-in modem, phone and call-state handling breaks.
- **Not dormant today**: it is trigger-started with three registrations.
- **Nothing visible changes**: on a machine with no modem there is no observable difference.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
- **Debloat: Remove Phone Link** (`remove_phone_link`) and **Interface: Hide the mobile device panel in Start** (`disable_phone_companion_start`) concern Phone Link, which this service does not serve. Phone Link depends on the Connected Devices Platform instead (see `disable_cdpsvc` on this page).

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research required the reboot flag and a statement that the service is trigger-started (3 registrations); the name, options, stock start type and the Phone Link clarification were correct. The shipped tweak carries both.
- **Confidence**: Microsoft-documented (Server guidance: Manual, "OK to disable").
- **Reasoning**: Tier A and tier C agree on Manual; the trigger count comes from `TriggerInfo` on 26100, which start-type changes cannot alter.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a desktop or any laptop without a cellular modem. Leave it enabled on a tablet, always-connected laptop or any device with built-in mobile broadband.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `PhoneSvc` Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Phone Service per-release table, https://batcmd.com/windows/11/services/phonesvc/ (tier C)
4. Trigger registration count for `PhoneSvc` (3), read from `HKLM\SYSTEM\CurrentControlSet\Services\PhoneSvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### Disable Network Device Discovery (SSDPSRV/upnphost)

`disable_ssdp_upnp` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows discovering and advertising devices over SSDP and UPnP.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `ssdpsrv` | service | `SSDPSRV` (display name "SSDP Discovery") | none |
| `upnphost` | service | `upnphost` (display name "UPnP Device Host") | none |

| Option | `ssdpsrv` | `upnphost` |
|---|---|---|
| Disabled | `disabled` | `disabled` |
| Manual | `manual` | `manual` |

System Default: the status shown when the two services do not match one option together; selecting it restores the snapshot. The stock start type on 24H2 is **Manual** for both, and neither is trigger-started, so a stock machine reads as the "Manual" option. Dependencies: `SSDPSRV` depends on `HTTP` and `NSI` and is required by `upnphost`; `upnphost` depends on `SSDPSRV` and `HTTP`. Features that rely on them: Cast to Device, DLNA and UPnP media discovery in both directions, UPnP port mapping requested by applications, and SSDP-based smart-home discovery. Windows Media Player network sharing rides on this transport.

#### How it works

`SSDPSRV` discovers network devices and services using the SSDP discovery protocol and announces SSDP devices running on this machine. `upnphost` (LocalService, `LocalServiceAndNoImpersonation` group) hosts UPnP devices on this machine. Together they are how Windows finds and advertises itself to smart TVs, media renderers and UPnP-capable routers. Because `upnphost` registers a dependency on `SSDPSRV`, the host cannot run without discovery.

Neither service is trigger-started. On a machine that casts or streams they are started on demand and are genuinely running at apply time, and they keep running until the next reboot because the engine never stops services. Microsoft's Server guidance lists both as Desktop Experience only, Manual, "OK to disable". UPnP has a long history of local-network exposure issues.

#### Benefits
- **Smaller local footprint**: the PC stops announcing itself to everything on the LAN.
- **Microsoft rates both safe**: the Server guidance marks both "OK to disable".
- **Known-risky protocol**: UPnP has a long history of local-network exposure issues.

#### Drawbacks
- **Cast to Device breaks**: streaming media from Windows to a TV stops working.
- **DLNA discovery breaks**: UPnP and DLNA devices no longer appear, in either direction.
- **Router auto-configuration breaks**: applications that open ports through UPnP, including some games and torrent clients, must be configured by hand.
- **Smart-home discovery**: some smart-home apps that find devices over SSDP stop seeing them.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type on both; System Default restores the snapshot.

#### Interactions
- **Disable Media Player Network Sharing (WMPNetworkSvc)** (`disable_wmp_network_sharing`, this page): library serving over DLNA needs this discovery transport. This tweak alone already breaks DLNA serving; the WMP tweak alone stops serving a library while keeping discovery up. The research proposed a single three-option tweak covering both; they ship as two independent tweaks.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The only correction was the reboot flag, which matters here because both services can be running at apply time on a machine that casts; the shipped tweak carries it.
- **Confidence**: Microsoft-documented (Server guidance for both), corroborated by clean-install dumps that also confirm neither is trigger-started.
- **Reasoning**: Names, defaults and Microsoft's stance agree across tiers; nothing else was disputed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a security-focused machine that does not cast, stream or rely on UPnP port forwarding. Leave it alone if you use Cast to Device, DLNA, or a game or client that opens ports through the router automatically.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `SSDPSRV` and `upnphost` Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, both Manual, neither trigger-started, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. SSDP Discovery and UPnP Device Host per-release tables, https://batcmd.com/windows/11/services/ssdpsrv/ and https://batcmd.com/windows/11/services/upnphost/ (tier C)

### Disable Windows Insider Service (wisvc)

`disable_windows_insider` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the service that enrols this PC in Windows Insider preview builds.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `wisvc` | service | `wisvc` (display name "Windows Insider Service") | none |

| Option | `wisvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `wisvc` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual (Trigger Start)**, so a stock machine reads as the "Manual" option. Depends on: `rpcss`. Required by: no other service. Features that rely on it: Windows Insider Program enrolment, channel selection and preview-build delivery. Normal Windows Update servicing does not.

#### How it works

`wisvc` provides the infrastructure for the Windows Insider Program: enrolment, channel selection and delivery of preview builds. Stable-channel Windows Update servicing does not go through it. With it disabled, the Windows Insider Program page in Settings fails to enrol without explaining why, so nobody can move the machine onto a preview build without reverting first. Microsoft's Server guidance lists it as always installed, Manual, "OK to disable".

#### Benefits
- **Microsoft rates it safe**: the Server guidance marks it "OK to disable".
- **No function on stable**: it does nothing on a machine kept on the retail channel.
- **Blocks accidental enrolment**: no preview-build enrolment without reverting first.

#### Drawbacks
- **Insider enrolment breaks**: you cannot join the program or receive preview builds.
- **Silent failure in Settings**: the Insider Program page fails without explanation.
- **Nothing visible changes**: on a stable-channel machine there is no observable difference.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot. Revert this first if you later decide to join the Insider Program.

#### Interactions
- **Windows Update: Block Insider preview builds by policy** (`block_insider_builds_policy`) is a separate, policy-based lever (`ManagePreviewBuildsPolicyValue`, Pro and above). The research judged keeping both sound: the policy blocks preview builds by management intent, while this tweak removes the enrolment service itself.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The only correction was the reboot flag; the shipped tweak carries it. The claim that normal Windows Update does not depend on this service was confirmed.
- **Confidence**: Microsoft-documented (Server guidance: always installed, Manual, "OK to disable").
- **Reasoning**: Name, default and stance agree across tiers.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any machine you keep on stable Windows, which is nearly all of them. Leave it enabled if you are an Insider or plan to become one.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `wisvc` always installed, Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Windows Insider Service per-release table, https://batcmd.com/windows/11/services/wisvc/ (tier C)

### Disable Downloaded Maps Manager (MapsBroker)

`disable_maps_broker` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the offline-maps download service on a PC that never uses offline maps.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `mapsbroker` | service | `MapsBroker` (display name "Downloaded Maps Manager") | none |

| Option | `mapsbroker` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Automatic (Delayed) | `automatic_delayed` (`Start=2`, `DelayedAutostart=1`) |

System Default: the status shown when `MapsBroker` is at a start type other than Disabled or Automatic (Delayed), for example plain Automatic or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Automatic (Delayed Start)**, so a stock machine reads as the "Automatic (Delayed)" option. Depends on: `rpcss`. Required by: no other service. Features that rely on it: offline map download and update, the `MapsToastTask` notification (which `MapsBroker` invokes), and any application built on the Windows Maps platform.

#### How it works

`MapsBroker` (`%SystemRoot%\System32\moshost.dll`, NetworkService) downloads offline maps and serves them to any application using the Windows Maps platform. Microsoft's own service description says it "is started on-demand by application accessing downloaded maps" and that "Disabling this service will prevent apps from accessing maps". Because it is Automatic (Delayed Start), it loads shortly after every boot, so disabling it removes genuine post-boot work, though a small amount, since the service is idle once loaded.

The feature is winding down: Microsoft states that Maps is no longer preinstalled with Windows starting with Windows 11 24H2, deprecated the Maps app in May 2025, and deprecated the Windows UWP Map control and Maps platform APIs on 8 April 2025.

#### Benefits
- **Removes a delayed-autostart service**: it no longer loads after every boot.
- **Microsoft rates it safe**: the Server guidance marks it "OK to disable".
- **Feature is winding down**: the Maps app and Maps platform APIs are deprecated.

#### Drawbacks
- **Offline maps stop**: downloading and updating offline map data no longer works.
- **Map-using apps fail**: applications built on the Windows Maps platform may error rather than degrade gracefully.
- **Not a big saving**: the service is idle once loaded.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021. Maps is not preinstalled from 24H2.
- **Takes effect**: at the next reboot.
- **Reverting**: "Automatic (Delayed)" writes the stock start type including the delayed flag; System Default restores the snapshot.

#### Interactions
- **Disable Offline Maps update tasks** (`task_maps_update`, this page) is the task half of the same subsystem. With `MapsBroker` disabled, `MapsToastTask` cannot fire at all, and a manual map update is only possible while the service is enabled. The research proposed merging the two into one three-option tweak; they ship separately, so apply both for the full effect.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the tweak correctly authors the stock state as `automatic_delayed`.
- **Confidence**: Microsoft-documented (Server guidance, the service's own description, and the deprecation notices).
- **Reasoning**: Every claim, including the 24H2 preinstall change, has a tier A source.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you do not use offline maps or the Maps app, which on 24H2 is most people. Leave it enabled if you keep downloaded map regions for offline navigation.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `MapsBroker` Automatic, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Deprecated features in the Windows client, Maps app deprecated May 2025, Windows UWP Map control and Maps platform APIs deprecated 8 April 2025, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
3. Resources for deprecated features, "Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release", https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources (tier A)
4. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, `MapsBroker` = Auto (Delayed), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)

### Disable Geolocation Service (lfsvc)

`disable_geolocation` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Removes system-wide location resolution, so nothing on the PC can work out where you are.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `lfsvc` | service | `lfsvc` (display name "Geolocation Service") | none |

| Option | `lfsvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `lfsvc` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual (Trigger Start)**, so a stock machine reads as the "Manual" option. Depends on: `RpcSs`. Required by: no other service registers a dependency. Features that rely on it: location for every app, Find My Device, automatic time zone (the Auto Time Zone Updater depends on it), location-based weather and maps.

#### How it works

`lfsvc` monitors the current location of the system and manages geofences, feeding the OS location provider that applications and Windows features query. Disabling it removes the provider itself rather than denying access to it, which is stronger than the Location privacy toggle: there is nothing for an app to be granted. Microsoft documents Find My Device as depending on device location, so a lost machine cannot be located with the service disabled.

The service is trigger-started with 4 trigger registrations on build 26100, so on a machine that uses location it starts itself whenever an app or Windows feature requests location, and it is genuinely running today. Microsoft's Server guidance lists it as Desktop Experience only, Manual, "OK to disable".

#### Benefits
- **Stronger than the toggle**: the provider is gone, so no app can be granted location.
- **Microsoft rates it safe**: the Server guidance marks it "OK to disable".
- **Removes a live trigger**: the service no longer starts itself on demand.

#### Drawbacks
- **All app location breaks**: every app that asks Windows where you are gets nothing.
- **Find My Device breaks**: a lost machine cannot be located.
- **Automatic time zone breaks**: the clock stops following you when you travel.
- **Weather and maps degrade**: location-based weather and maps that centre on you stop working.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
- **Privacy: Turn off location tracking** (`disable_location_tracking`) turns location off by policy and consent; this tweak removes the provider underneath. Either alone stops apps from getting a location; together they are belt and braces.

#### Validation
- **Verdict**: VERIFIED. The research asked only that the copy state the service is trigger-started (4 registrations); the mechanism, options, stock start type and the warning list were correct.
- **Confidence**: Microsoft-documented (Server guidance, and Microsoft's Find My Device documentation).
- **Reasoning**: The trigger count comes from `TriggerInfo` on 26100, which start-type changes cannot alter; the list of affected features survived the adversarial pass intact.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a stationary desktop where privacy matters and nothing needs to know where the machine is. Leave it enabled on a laptop you travel with, where automatic time zone and Find My Device are worth more than the privacy gain.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `lfsvc` Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Manage connections from Windows operating system components to Microsoft services, Find My Device section, https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services (tier A)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, `lfsvc` = Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Trigger registration count for `lfsvc` (4), read from `HKLM\SYSTEM\CurrentControlSet\Services\lfsvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### Xbox Live Services (XblAuthManager et al.)

`disable_xbox_services` · Dropdown (3 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off Xbox sign-in, cloud saves and networking on a PC that never uses Xbox or Game Pass, with the controller service selectable separately.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `xblauthmanager` | service | `XblAuthManager` (display name "Xbox Live Auth Manager") | none |
| `xblgamesave` | service | `XblGameSave` (display name "Xbox Live Game Save") | none |
| `xboxnetapisvc` | service | `XboxNetApiSvc` (display name "Xbox Live Networking Service") | none |
| `xboxgipsvc` | service | `XboxGipSvc` (display name "Xbox Accessory Management Service") | none |

| Option | `xblauthmanager` | `xblgamesave` | `xboxnetapisvc` | `xboxgipsvc` |
|---|---|---|---|---|
| Disabled | `disabled` | `disabled` | `disabled` | `disabled` |
| Xbox Live disabled, accessories kept | `disabled` | `disabled` | `disabled` | `manual` |
| Manual | `manual` | `manual` | `manual` | `manual` |

System Default: the status shown when the four services match none of the three combinations; selecting it restores the snapshot. The stock start type on 24H2 is **Manual** for all four (`XblGameSave` and `XboxGipSvc` additionally trigger-started; `XblAuthManager` and `XboxNetApiSvc` plain Manual), so a stock machine reads as the "Manual" option. Dependencies: `XblAuthManager` depends on `RpcSs` and is required by `XblGameSave`; `XblGameSave` depends on `UserManager` and `XblAuthManager`; `XboxNetApiSvc` depends on `BFE`, `mpssvc`, `IKEEXT` and `KeyIso`; `XboxGipSvc` has no registered dependencies either way. Features that rely on them: the Xbox app, PC Game Pass, Xbox Live sign-in, cloud save sync, Game Bar sign-in and capture features, and (for `XboxGipSvc`) connected Xbox accessories.

#### How it works

`XblAuthManager`, `XblGameSave` and `XboxNetApiSvc` are Xbox Live account and networking plumbing: authentication and authorization, save-data synchronisation, and the networking API. A PC that never touches the Xbox app or Game Pass gets nothing from them. `XboxGipSvc` (`%WinDir%\System32\XboxGipSvc.dll`, LocalSystem, `netsvcs` group) is different: it manages connected Xbox accessories, and it is the only member of the group tied to hardware you may own.

That is why the tweak has a middle option. "Xbox Live disabled, accessories kept" disables the three Live services and leaves the accessory service at its stock Manual. Raw gamepad input is delivered by the kernel-mode GIP and XUSB drivers rather than by this service, and CIS records the documented impact of disabling it narrowly as "Connected Xbox accessories may not function"; broader controller breakage in Steam, Epic and other launchers is reported at community tier. Treat it as a real risk anyway.

Microsoft's Server guidance rates `XblAuthManager` and `XblGameSave` "Should be disabled"; the CIS Windows 11 Enterprise Benchmark (item 5.42, Level 1) recommends `XboxGipSvc` be Disabled.

#### Benefits
- **Microsoft recommends two of them**: `XblAuthManager` and `XblGameSave` are rated "Should be disabled".
- **CIS recommends a third**: `XboxGipSvc` is Disabled at CIS Level 1.
- **Dead weight without Xbox**: on a PC that never uses the Xbox app or Game Pass, none of the four does anything useful.

#### Drawbacks
- **Xbox controllers can break system-wide**: disabling `XboxGipSvc` can stop controllers working in any launcher; the middle option avoids this.
- **Xbox app and Game Pass break**: sign-in, the store front end and PC Game Pass stop working.
- **Cloud saves stop**: progress stays local.
- **Game Bar loses features**: sign-in and capture features that depend on Xbox Live stop working.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type on all four; System Default restores the snapshot.

#### Interactions
- **Debloat: Remove Xbox Game Bar** (`remove_xbox_game_bar`) and **Performance: Disable Xbox Game Bar capture (Game DVR)** (`disable_gamedvr_capture`) cover the Game Bar app and capture; this tweak covers the Live services underneath.
- **Disable Bluetooth Support Services** (`disable_bluetooth`, this page): a Bluetooth-connected Xbox controller also needs the Bluetooth stack.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found that `XboxGipSvc` had to be independently selectable, because it is hardware-bound and the tweak's own advice (keep it while disabling the rest) needs a matching option; the shipped tweak has that option. It also recorded that controller breakage beyond "accessories may not function" is community-tier evidence.
- **Confidence**: Microsoft-documented for the two Live services (Server guidance); CIS benchmark (tier B) for `XboxGipSvc`; clean-install dumps for all four defaults.
- **Reasoning**: Defaults agree across Microsoft, CIS ("Windows 10 R1709 and newer: Manual") and clean-install dumps. The research explicitly advised against merging this tweak with anything further.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply the full disable on a PC that never uses Xbox, Game Pass or an Xbox controller. If you game with an Xbox controller in any launcher, choose "Xbox Live disabled, accessories kept".

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `XblAuthManager` and `XblGameSave` Manual, "Should be disabled", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. CIS Microsoft Windows 11 Enterprise Benchmark, item 5.42 "Ensure 'Xbox Accessory Management Service (XboxGipSvc)' is set to 'Disabled'", impact "Connected Xbox accessories may not function", default "Windows 10 R1709 and newer: Manual", https://www.tenable.com/audits/items/CIS_MS_Windows_11_Enterprise_Level_1_v1.0.0.audit:00bba2f087d462b5662f1987510165b6 (tier B)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, all four present with the stated defaults, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Xbox Accessory Management Service, Windows 11 per-release table, https://batcmd.com/windows/11/services/xboxgipsvc/ (tier C)

### Disable Connected Devices Platform Service (CDPSvc)

`disable_cdpsvc` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops the background sync that keeps this PC in step with your other Microsoft-linked devices.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `cdpsvc` | service | `CDPSvc` (display name "Connected Devices Platform Service") | none |

| Option | `cdpsvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Automatic (Delayed) | `automatic_delayed` (`Start=2`, `DelayedAutostart=1`) |

System Default: the status shown when `CDPSvc` is at a start type other than Disabled or Automatic (Delayed); selecting it restores the snapshot. The stock start type on 24H2 is **Automatic (Delayed Start, Trigger Start)**, so a stock machine reads as the "Automatic (Delayed)" option. Depends on: `ncbservice`, `RpcSS`, `Tcpip`. Required by: no service registers a dependency, but the per-user companion `CDPUserSvc_*` relies on it functionally. Features that rely on it: Phone Link, Nearby Sharing, the shared (cross-device) clipboard and "resume on other devices" handoff.

#### How it works

`CDPSvc` (`%SystemRoot%\System32\CDPSvc.dll`, LocalService) is the cross-device fabric behind Nearby Sharing, Phone Link, the cross-device clipboard and handoff, and it maintains background connections to keep those in sync. It is a network-active service, not an idle one, and as Automatic (Delayed Start) it starts shortly after every boot.

The per-user companion service `CDPUserSvc_*` is Automatic and is not touched by this tweak. With `CDPSvc` disabled, the per-user service keeps running and simply fails to reach its backend. Microsoft's Server guidance lists `CDPSvc` as Desktop Experience only, Automatic, "No guidance", so the case for disabling rests on the absence of a dependency.

#### Benefits
- **Cuts background connections**: a genuinely network-active service stops.
- **Removes a delayed-autostart load**: it no longer starts after every boot.
- **No value alone**: a single-device user gets nothing from cross-device sync.

#### Drawbacks
- **Phone Link breaks**: your phone no longer connects to this PC.
- **Nearby Sharing breaks**: sending files to a nearby Windows device stops working.
- **Shared clipboard breaks**: copy on one device and paste on another stops working, as does handoff.
- **Per-user half stays up**: `CDPUserSvc_*` keeps running and fails to reach its backend.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Automatic (Delayed)" writes the stock start type including the delayed flag; System Default restores the snapshot.

#### Interactions
- **Privacy: Disable the cross-device cloud clipboard** (`disable_cloud_clipboard`) turns off the clipboard sync by policy; this tweak removes the platform under it and more.
- **Debloat: Remove Phone Link** (`remove_phone_link`) and **Interface: Hide the mobile device panel in Start** (`disable_phone_companion_start`) touch Phone Link surfaces that stop working once this service is disabled.

#### Validation
- **Verdict**: VERIFIED. No correction was needed; the tweak correctly authors the stock state as `automatic_delayed`. The research added the `CDPUserSvc_*` behaviour as context.
- **Confidence**: Microsoft-documented (Server guidance for the default), corroborated by clean-install dumps and a per-release reference.
- **Reasoning**: Every source agrees on Automatic (Delayed, Triggered); nothing was disputed.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a single-device machine that does not use Phone Link, Nearby Sharing or the shared clipboard. Leave it enabled if you move between a phone and this PC, since those features have no fallback.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `CDPSvc` Automatic, "No guidance", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 Default Services Configuration, `CDPSvc` = Auto (Delayed, Triggered), `CDPUserSvc_*` = Auto, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
3. Windows 11 Default Services Configuration, `CDPSvc` = Auto (Delayed, Triggered), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
4. Connected Devices Platform Service (CDPSvc) defaults, https://revertservice.com/11/cdpsvc/ (tier C)

### Disable Windows Biometric Service (WbioSrvc)

`disable_biometrics` · Switch (2 options) · Risk: medium · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off Windows Hello fingerprint and face sign-in on a PC that logs in with a PIN or password.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `wbiosrvc` | service | `WbioSrvc` (display name "Windows Biometric Service") | none |

| Option | `wbiosrvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `WbioSrvc` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual (Trigger Start)**, so a stock machine reads as the "Manual" option. Depends on: `RpcSs`. Required by: no other service. Features that rely on it: Windows Hello fingerprint and facial recognition sign-in, and any app or credential provider that uses biometric samples.

#### How it works

`WbioSrvc` gives client applications the ability to capture, compare, manipulate and store biometric data without direct access to biometric hardware or samples; it manages fingerprint readers and infrared cameras, and Windows Hello fingerprint and face sign-in run on top of it. The service is trigger-started with 2 trigger registrations on build 26100, so on a machine with biometric hardware it starts itself and is running today. With it disabled, Windows falls back to PIN or password rather than locking you out, but a Windows Hello for Business setup that gates a credential on biometrics stops working, so confirm a PIN or password works first. Microsoft's Server guidance lists it as Desktop Experience only, Manual, "No guidance".

#### Benefits
- **Hardware you may not have**: on a desktop with no reader or IR camera it manages nothing.
- **Removes a credential path**: one fewer way to authenticate, which is a hardening choice.
- **Fully reversible**: Windows falls back to PIN or password.

#### Drawbacks
- **Fingerprint sign-in breaks**: the reader stops being an option at sign-in.
- **Face sign-in breaks**: Windows Hello facial recognition stops working.
- **Hello for Business paths**: a biometric-gated credential stops working; verify a PIN or password first.
- **Not dormant today**: the service is trigger-started.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
- **Disable Smart Card Services** (`disable_smartcard`, this page) also removes an alternate credential provider. The research explicitly rejected merging the two: losing biometrics falls back to PIN or password, while losing smart card can remove the only sign-in path, so their risk levels differ (medium versus high).

#### Validation
- **Verdict**: VERIFIED. The research asked only that the copy state the service is trigger-started (2 registrations).
- **Confidence**: Microsoft-documented for the service's role and presence in the Server guidance; the default is corroborated by clean-install dumps.
- **Reasoning**: Defaults agree across sources; the trigger count comes from `TriggerInfo` on 26100.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you sign in only with a password or PIN and have no fingerprint reader or IR camera. Leave it enabled if you use any biometric unlock, since the convenience loss is immediate and the gain is negligible.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, `WbioSrvc` Manual, "No guidance", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, `WbioSrvc` = Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Windows Biometric Service per-release table, https://batcmd.com/windows/11/services/wbiosrvc/ (tier C)
4. Trigger registration count for `WbioSrvc` (2), read from `HKLM\SYSTEM\CurrentControlSet\Services\WbioSrvc\TriggerInfo` on build 26100 (tier A for trigger existence only)

### Disable Smart Card Services (SCardSvr)

`disable_smartcard` · Switch (2 options) · Risk: high · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off smart card support on a consumer PC that has no card reader and no card-based sign-in.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `scardsvr` | service | `SCardSvr` (display name "Smart Card") | none |
| `scdeviceenum` | service | `ScDeviceEnum` (display name "Smart Card Device Enumeration Service") | none |
| `scpolicysvc` | service | `SCPolicySvc` (display name "Smart Card Removal Policy") | none |

| Option | `scardsvr` | `scdeviceenum` | `scpolicysvc` |
|---|---|---|---|
| Disabled | `disabled` | `disabled` | `disabled` |
| Manual | `manual` | `manual` | `manual` |

System Default: the status shown when the three services do not match one option together; selecting it restores the snapshot. The stock start type on client 24H2 is **Manual (Trigger Start)** for `SCardSvr` and `ScDeviceEnum`, and plain **Manual** for `SCPolicySvc`, so a stock machine reads as the "Manual" option. (Windows Server ships `SCardSvr` Disabled; that is a Server-only default.) Dependencies: `SCardSvr` and `ScDeviceEnum` have no registered service dependencies either way; `SCPolicySvc` depends on `RpcSs`. Features that rely on them: every smart card component (the smart card credential provider, CAC, PIV and YubiKey PIV sign-in, certificate authentication from a card, card-based VPN), and lock-on-card-removal.

#### How it works

`SCardSvr` (the Smart Cards for Windows service, formerly the Smart Card Resource Manager) provides, in Microsoft's words, "the basic infrastructure for all other smart card components as it manages smart card readers and application interactions on the computer". `ScDeviceEnum` creates software device nodes for all smart card readers accessible to a session. `SCPolicySvc` enforces the removal policy, for example locking the desktop when a card is withdrawn. With `SCardSvr` disabled, the smart card credential provider has no path to the card, so on a machine whose only sign-in path is a smart card there is no interactive sign-in left and no way to revert from inside Windows.

`SCardSvr` has 4 trigger registrations and `ScDeviceEnum` 3 on build 26100, so on a machine with a reader attached they are usually already running when you apply the tweak. The engine never stops them, so the card keeps working until the next reboot and the change lands at the worst possible moment: the next sign-in.

#### Benefits
- **Hardware you do not have**: on a home PC with no reader, three services manage nothing.
- **Removes a credential path**: no card-based authentication surface remains.
- **Removes live triggers**: `SCardSvr` and `ScDeviceEnum` no longer start themselves.

#### Drawbacks
- **Can lock you out**: on a machine that signs in with a smart card, this removes the only sign-in path and cannot be reverted from inside Windows.
- **CAC, PIV and YubiKey PIV break**: card-based logon of every kind stops working.
- **Enterprise VPN and certificate authentication break**: anything that reads a card stops.
- **Removal policy stops**: with `SCPolicySvc` disabled the desktop no longer locks when a card is withdrawn, a security regression on a card-using machine.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot; a card keeps working until then.
- **Reverting**: "Manual" writes the stock start types; System Default restores the snapshot. If smart card was the only sign-in path, revert is only possible from outside Windows (for example recovery media or another admin account with a password). Confirm a working password or PIN sign-in first, and never apply it remotely.

#### Interactions
- **Disable Windows Biometric Service** (`disable_biometrics`, this page) removes a different credential provider; the research rejected merging the two because their lockout risk differs.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found the risk must be high, not medium, because the lockout it warns about cannot be undone from inside Windows; that the tweak needs the reboot flag, which matters most here because the triggered services are usually running at apply time; and that the copy must name `SCPolicySvc` as the lock-on-removal service. The shipped tweak carries all three.
- **Confidence**: Microsoft-documented. Microsoft Learn reproduces the `SCardSvr` service manifest with `start="demand"` (Manual) and describes its role; the Server guidance covers the other two.
- **Reasoning**: The dependency of every smart card component on `SCardSvr` is Microsoft's own statement, which is what makes the lockout real. The inclusion-principle review also asked that the recommendation state what the control does rather than editorialise about defaults.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it only on a personal PC with no card reader and no card-based sign-in. Never apply it on a work, managed or enterprise machine, or anywhere smart card, CAC, PIV or YubiKey PIV sign-in is in use, because the lockout is not recoverable from inside Windows.

#### Sources
1. Smart Cards for Windows Service, the service manifest with `start="demand"` and its role as base infrastructure for all smart card components, https://learn.microsoft.com/en-us/windows/security/identity-protection/smart-cards/smart-card-smart-cards-for-windows-service (tier A)
2. Guidelines for disabling system services in Windows Server with Desktop Experience, `SCardSvr` Disabled on Server, `ScDeviceEnum` Manual "OK to disable", `SCPolicySvc` Manual "No guidance", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
3. Windows 10 Default Services Configuration, `SCardSvr` and `ScDeviceEnum` = Manual (Triggered), `SCPolicySvc` = Manual, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
4. Windows 11 Default Services Configuration, same three defaults, https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
5. Trigger registration counts for `SCardSvr` (4) and `ScDeviceEnum` (3), read from `TriggerInfo` on build 26100 (tier A for trigger existence only)

### Disable Sensor Services (SensorService)

`disable_sensor_services` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the sensor stack on a desktop that has no ambient-light or orientation sensors.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `sensorservice` | service | `SensorService` (display name "Sensor Service") | none |
| `sensrsvc` | service | `SensrSvc` (display name "Sensor Monitoring Service") | none |
| `sensordataservice` | service | `SensorDataService` (display name "Sensor Data Service") | none |

| Option | `sensorservice` | `sensrsvc` | `sensordataservice` |
|---|---|---|---|
| Disabled | `disabled` | `disabled` | `disabled` |
| Manual | `manual` | `manual` | `manual` |

System Default: the status shown when the three services do not match one option together; selecting it restores the snapshot. The stock start type on 24H2 (and on LTSC 2021) is **Manual (Trigger Start)** for all three, so a stock machine reads as the "Manual" option. Dependencies: none of the three registers a service dependency either way. Features that rely on them: adaptive (automatic) brightness, automatic screen rotation, and any app that reads orientation, light or motion sensors.

#### How it works

`SensorService` manages the device's sensors and the sensor functionality Windows exposes to applications. `SensrSvc` (`%WinDir%\system32\sensrsvc.dll`, LocalService, `LocalServiceAndNoImpersonation` group) monitors sensors and adapts system state, its documented example being screen brightness following ambient light. `SensorDataService` (`SensorDataService.exe`, LocalSystem) delivers the sensor data itself.

All three are trigger-started, and `SensorService` carries 8 trigger registrations on build 26100, the most of any service on this page, so on hardware with sensors it starts itself. On a genuinely sensorless machine they never start, so the result is a guarantee rather than a speed gain. Whether a machine has sensors is not always obvious (a laptop lid sensor, or a monitor with an ambient-light sensor), so check Device Manager under Sensors before applying. Microsoft's Server guidance rates all three "OK to disable".

#### Benefits
- **Microsoft rates all three safe**: every one is "OK to disable".
- **Hardware you may not have**: a tower desktop typically has no light, orientation or motion sensors.
- **Removes many triggers**: `SensorService` alone has eight.

#### Drawbacks
- **Auto-brightness breaks**: adaptive brightness stops responding to ambient light.
- **Screen rotation breaks**: automatic rotation on a tablet or convertible stops.
- **Sensor-aware apps break**: anything reading orientation, light or motion gets nothing.
- **Hard to be sure**: check Device Manager under Sensors first.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start types; System Default restores the snapshot.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research required the reboot flag and a statement that `SensorService` is trigger-started with eight registrations; it also confirmed `SensrSvc` is still present on 21H2 through 24H2, against a suspicion that it had been removed. The shipped tweak carries the flag and the wording.
- **Confidence**: Microsoft-documented (Server guidance for all three).
- **Reasoning**: Defaults agree across all tiers, and no per-release table has a "not exists" entry for any of the three.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on a desktop you have confirmed has no sensors listed in Device Manager. Leave it enabled on any laptop, tablet or convertible, where auto-brightness and rotation are worth more than removing three idle services.

#### Sources
1. Guidelines for disabling system services in Windows Server with Desktop Experience, all three Manual, "OK to disable", https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server (tier A)
2. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, all three Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
3. Sensor Monitoring Service, Windows 11 per-release table, Manual on 21H2 through 24H2 with no "not exists" entry, https://batcmd.com/windows/11/services/sensrsvc/ (tier C)
4. Trigger registration count for `SensorService` (8), read from `TriggerInfo` on build 26100 (tier A for trigger existence only)

### Disable Family Safety Monitor (WpcMonSvc)

`disable_parental_controls` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the Family Safety monitor on an account that is not supervised by Microsoft Family.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `wpcmonsvc` | service | `WpcMonSvc` (display name "Parental Controls") | none |

| Option | `wpcmonsvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `WpcMonSvc` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual**, not trigger-started, so a stock machine reads as the "Manual" option. Depends on: nothing. Required by: no other service. Features that rely on it: Microsoft Family Safety enforcement on this device (screen time limits, app and content filtering, activity reporting).

#### How it works

`WpcMonSvc` is what enforces Microsoft Family Safety restrictions applied to an account on this device: screen time limits, app and content filtering, and activity reporting to the parent. On an adult or standalone account it enforces nothing. Unlike most services on this page it is plain Manual with no trigger, so disabling it is a clean removal.

This is the one tweak on the page whose misuse harms someone other than the person applying it. On a device with a supervised child account, disabling it silently stops enforcement, and the parent gets no notification: limits simply stop working with no obvious cause. The low risk rating describes risk to the machine, not to the supervised child. Microsoft's Server guidance does not cover this consumer service.

#### Benefits
- **No account to supervise**: on an adult or standalone account it enforces nothing.
- **Not trigger-started**: plain Manual, so this is a clean removal.
- **Fully reversible**: nothing else depends on it.

#### Drawbacks
- **Supervision stops silently**: on a device with a supervised child account this defeats parental controls with no notification to the parent.
- **Screen time limits break**: time restrictions are not applied on this device.
- **Content filtering breaks**: app and web content limits stop being enforced.
- **Activity reporting breaks**: the parent's report for this device goes quiet.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
None known.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research required the reboot flag and a plain statement of the supervision-bypass consequence; no mechanism defect was found. The shipped tweak carries both.
- **Confidence**: Community-corroborated. No Microsoft source gives the default; two clean-install dumps and a per-release table agree on Manual.
- **Reasoning**: Three tier C sources agree; the third-party harm was the one point the adversarial pass added.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on an adult or standalone account that has never used Microsoft Family Safety. Never apply it on a device with a supervised child account, where it is a supervision bypass rather than a tidy-up.

#### Sources
1. Windows 11 Default Services Configuration, `WpcMonSvc` = Manual, https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
2. Windows 10 Default Services Configuration, `WpcMonSvc` = Manual, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
3. Parental Controls per-release table, https://batcmd.com/windows/11/services/wpcmonsvc/ (tier C)

### Disable Payments and NFC/SE Manager (SEMgrSvc)

`disable_payments_nfc` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Turns off the NFC secure-element manager on a device with no tap-to-pay hardware.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `semgrsvc` | service | `SEMgrSvc` (display name "Payments and NFC/SE Manager") | none |

| Option | `semgrsvc` |
|---|---|
| Disabled | `disabled` (`Start=4`) |
| Manual | `manual` (`Start=3`) |

System Default: the status shown when `SEMgrSvc` is at a start type other than Disabled or Manual; selecting it restores the snapshot. The stock start type on 24H2 is **Manual (Trigger Start)**, so a stock machine reads as the "Manual" option. Depends on: `RpcSs`. Required by: no other service. Features that rely on it: NFC tap-to-pay and secure-element wallet operations.

#### How it works

`SEMgrSvc` manages the hardware secure element that NFC tap-to-pay and secure wallet operations run through, on devices that have NFC hardware. It is the hardware half of the Windows payment stack; the Wallet service (`WalletService`) is the front end. The service is trigger-started with 2 trigger registrations on build 26100, so it starts itself on demand today. Microsoft's Server guidance does not cover it.

#### Benefits
- **Hardware you may not have**: a desktop has no NFC radio and no secure element.
- **Removes a payment surface**: nothing can reach the secure element.
- **Removes a live trigger**: the service no longer starts itself.

#### Drawbacks
- **Tap-to-pay breaks**: NFC payments from this device stop working.
- **Secure wallet breaks**: wallet features that use the secure element stop working.
- **Half the stack**: the Wallet service is separate and keeps running.
- **Nothing visible changes**: on a machine without NFC there is no observable difference.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type; System Default restores the snapshot.

#### Interactions
- **Disable Wallet Service (WalletService)** (`disable_wallet_service`, this page) is the front-end half. The research proposed merging the two; they ship separately, so apply both for the full effect.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research required the reboot flag and a statement that the service is trigger-started (2 registrations); the shipped tweak carries both.
- **Confidence**: Community-corroborated. No Microsoft source gives the default; clean-install dumps and a per-release table agree.
- **Reasoning**: Three tier C sources agree on Manual (Triggered); the trigger count comes from `TriggerInfo` on 26100.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it on any desktop, and on any laptop without NFC. Leave it enabled if you tap to pay from this device or use a secure-element wallet.

#### Sources
1. Windows 11 Default Services Configuration, `SEMgrSvc` = Manual (Triggered), https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C)
2. Windows 10 Default Services Configuration, `SEMgrSvc` = Manual (Triggered), https://www.winhelponline.com/blog/windows-10-default-services-configuration/ (tier C)
3. Payments and NFC/SE Manager per-release table, https://batcmd.com/windows/11/services/semgrsvc/ (tier C)
4. Trigger registration count for `SEMgrSvc` (2), read from `TriggerInfo` on build 26100 (tier A for trigger existence only)

### Disable Media Player Network Sharing (WMPNetworkSvc)

`disable_wmp_network_sharing` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: yes · Windows: all supported builds · Reversible: yes

**Stops Windows Media Player serving your media library to other devices on the network.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `state` | registry | `HKCU\Software\MagicXToolbox\State`, value `WmpNetworkSharing`, `REG_DWORD` (app marker) | none |
| `wmpnetworksvc` | service | `WMPNetworkSvc` (display name "Windows Media Player Network Sharing Service") | `optional: true`, `if_missing: disabled` |

| Option | `state` | `wmpnetworksvc` |
|---|---|---|
| Disabled | `1` | `disabled` (`Start=4`) |
| Manual | `0` | `manual` (`Start=3`) |

System Default: shown until you pick an option, because the marker value does not exist on a machine where the tweak was never applied; selecting it restores the snapshot. Where the service exists, its stock start type is **Manual**, with no trigger registration. The service is **not part of the base OS**: it is absent on Windows 11 24H2 IoT Enterprise LTSC 2024 (and on any image without the Windows Media Player Legacy feature), and whether retail Home and Pro 24H2 images still install that feature by default was not verified. Depends on / required by: not observed (absent from the build 26100 reference machine). Features that rely on it: serving the Windows Media Player library over DLNA and UPnP to TVs and network players.

#### How it works

`WMPNetworkSvc` (`%ProgramFiles%\Windows Media Player\wmpnetwk.exe`, NetworkService, its own process) shares Windows Media Player libraries to other networked players and media devices over UPnP. Local playback is unaffected. It is delivered by the removable `Media.WindowsMediaPlayer` optional feature (Windows Media Player Legacy), not by Windows itself; on 24H2 IoT Enterprise LTSC 2024 there is no service key, no `wmpnetwk.exe`, and `Get-WindowsCapability -Online` reports the feature `NotPresent`.

The effect is therefore `optional` with `if_missing: disabled`. On an image without the feature the service reads as Disabled: applying "Disabled" is a verified no-op that only sets the marker, and "Manual" is shown as unavailable on this machine because the engine never installs services. The non-optional per-user marker keeps both options detectable on every build; another account on the same machine sees System Default until it applies an option. DLNA also needs the SSDP and UPnP discovery services, which this tweak leaves alone.

#### Benefits
- **Closes a network listener**: the service stops serving media to the LAN.
- **Nothing else depends on it**: no other Windows feature needs library sharing.
- **Legacy component**: Microsoft deprecated the legacy DRM services this player relies on in December 2024.

#### Drawbacks
- **DLNA serving stops**: streaming your library to a TV or other renderer no longer works.
- **Discovery is separate**: the SSDP and UPnP transport stays up with this tweak alone.
- **May not be installed**: absent on Enterprise LTSC and IoT Enterprise LTSC images and wherever the feature was removed, so there may be nothing to change.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021, but it changes something only where Windows Media Player Legacy is installed.
- **Takes effect**: at the next reboot.
- **Reverting**: "Manual" writes the stock start type where the service exists; System Default restores the snapshot (marker removed, captured start type restored).

#### Interactions
- **Disable Network Device Discovery (SSDPSRV/upnphost)** (`disable_ssdp_upnp`, this page) removes the discovery transport that DLNA rides on. Applying that tweak already breaks DLNA serving; this tweak stops serving a library while keeping discovery. The research proposed a single three-option tweak; they ship as two.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research established that the service rides a removable optional feature and is absent on IoT Enterprise LTSC, so the effect must be optional (a required effect on an absent service makes the tile Unknown and blocks apply and snapshot capture); it also required the reboot flag and found the service plain Manual with no trigger where present. The shipped tweak reflects all of this.
- **Confidence**: Community-corroborated for the default where present (clean-install dumps describing images with the feature); tier A for absence on 24H2 LTSC (direct inspection of the shipped image) and for the DRM deprecation.
- **Reasoning**: Absence is an image fact rather than a configuration default, so it survives the research's provenance rule. Open: how many retail Home and Pro 24H2 machines still have the feature.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you do not stream your media library from this PC to other devices. Leave it enabled if you use DLNA sharing from Windows Media Player to a TV or a network player.

#### Sources
1. Direct inspection of Windows 11 24H2 build 26100 (IoT Enterprise LTSC 2024): no `WMPNetworkSvc` service key, no `wmpnetwk.exe`, `Media.WindowsMediaPlayer~~~~0.0.12.0 = NotPresent` (tier A for absence)
2. Deprecated features in the Windows client, legacy DRM services used by Windows Media Player deprecated December 2024, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
3. Windows 10 and Windows 11 Default Services Configuration clean-install dumps, `WMPNetworkSvc` = Manual, https://www.winhelponline.com/blog/windows-10-default-services-configuration/ and https://www.winhelponline.com/blog/windows-11-default-services-configuration/ (tier C, images with the feature installed)
4. Windows Media Player Network Sharing Service, Windows 11 per-release table, https://batcmd.com/windows/11/services/wmpnetworksvc/ (tier C, same caveat)
5. `src-tauri/src/tweaks/kinds/service.rs` and the engine's missing-required path (tier A, this repository)

### Disable Autochk Proxy task

`task_autochk_proxy` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the scheduled upload of disk-check telemetry to Microsoft, without touching disk checking itself.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `proxy` | task | `\Microsoft\Windows\Autochk\Proxy` | none |

| Option | `proxy` |
|---|---|
| Disabled | `disabled` |
| Enabled | `enabled` |

System Default: with only two possible task states and both authored, a readable task always matches an option, so System Default appears only after a snapshot exists and you choose to restore it. The shipped enabled state of this task on a clean 24H2 image is not established (the research found no clean-image baseline), though it is presumed Enabled; if it is, a stock machine reads as "Enabled".

#### How it works

The task runs `rundll32.exe acproxy.dll,PerformAutochkOperations` from a boot trigger. Microsoft's own shipped description string for it (resource `-102` in `%SystemRoot%\System32\acproxy.dll`) reads: "This task collects and uploads autochk SQM data if opted-in to the Microsoft Customer Experience Improvement Program." It is the telemetry upload of disk-check data, not the disk check: boot-time `autochk` and `chkdsk` are a different mechanism and keep working.

The upload is gated on CEIP consent by Microsoft's own description, so on a machine that never opted in the task uploads nothing and disabling it produces no observable change. Tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates.

#### Benefits
- **One less upload**: removes a boot-time diagnostic transmission.
- **Disk checking unaffected**: `autochk` and `chkdsk` keep working.
- **No functional cost**: nothing on the machine reads the data this task sends.

#### Drawbacks
- **Nothing changes without CEIP**: on a machine that never opted in there is no difference.
- **Updates can re-enable it**: feature updates can re-create or re-enable the task.
- **Partial coverage**: one of several CEIP and census tasks.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately, for the next boot-triggered run.
- **Reverting**: "Enabled" re-enables the task; System Default restores the snapshot's captured state.

#### Interactions
- **Privacy: Disable the Customer Experience Improvement Program** (`disable_ceip_tasks`) sets the CEIP policy this task honours and disables the `Consolidator`, `UsbCeip` and `KernelCeipTask` tasks, but does not cover this task, even though Microsoft's string makes it a CEIP SQM uploader. Use both for full CEIP coverage.
- **Disable DiskDiagnosticDataCollector task**, **Disable Feedback (SIUF) DmClient tasks** and **Disable Device Census tasks** (this page) are the other telemetry tasks. The research proposed grouping the CEIP-type tasks into one tweak; they ship separately.

#### Validation
- **Verdict**: VERIFIED. The research asked only that the copy say the upload is CEIP-gated.
- **Confidence**: Microsoft-documented: the task's purpose comes from Microsoft's own shipped description string.
- **Reasoning**: The path is corroborated by independent tooling. Open: the shipped enabled state, for which no clean-image baseline exists.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want fewer diagnostic uploads leaving the machine; there is no functional downside. Skip it if you deliberately participate in CEIP for enterprise reporting.

#### Sources
1. Shipped task description resource `%SystemRoot%\System32\acproxy.dll,-102`, "This task collects and uploads autochk SQM data if opted-in to the Microsoft Customer Experience Improvement Program" (tier A, shipped binary resource)
2. CEIPEnable, the Customer Experience Improvement Program control, https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable (tier A)
3. Configure Windows diagnostic data in your organization, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
4. Windows-10-Hardening, `components/scheduled_tasks.bat`, identical task path, https://github.com/aghorler/Windows-10-Hardening/blob/master/components/scheduled_tasks.bat (tier D, path form only)
5. windows10fixup `Fixup.ps1`, identical task path, https://github.com/iDigitalFlame/windows10fixup (tier D, path form only)

### Disable Feedback (SIUF) DmClient tasks

`task_feedback_dmclient` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the Windows Feedback tasks that refresh feedback scenarios and send feedback and diagnostic data to Microsoft.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `dmclient` | task | `\Microsoft\Windows\Feedback\Siuf\DmClient` | none |
| `dmclient_scenario` | task | `\Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload` | none |

| Option | `dmclient` | `dmclient_scenario` |
|---|---|---|
| Disabled | `disabled` | `disabled` |
| Enabled | `enabled` | `enabled` |

System Default: the status shown when the two tasks are in different states (one enabled, one disabled); selecting it restores the snapshot. The shipped enabled state of both tasks is not established from a clean image; the absence of an `<Enabled>` element in their definitions suggests they were never rewritten (so Enabled), but the research did not treat that as proof.

#### How it works

SIUF is Microsoft's internal name for the feedback subsystem. Both tasks drive `dmclient.exe`, identified as the "Microsoft Feedback SIUF Deployment Manager Client". `DmClientOnScenarioDownload` refreshes the feedback scenario configuration (its description reads "Update SIUF...") and carries a real `WnfStateChangeTrigger`, so it runs on an event. `DmClient` has an empty `<Triggers />` set on build 26100: nothing schedules it, and disabling it only blocks an on-demand or externally invoked run.

With the telemetry service (`DiagTrack`) disabled, these tasks have no working upload path anyway. Tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates.

#### Benefits
- **Removes a feedback channel**: nothing on the machine depends on these uploads.
- **Blocks the scenario refresh**: `DmClientOnScenarioDownload` is the task with a live trigger.
- **No functional cost**: neither task provides a user-facing feature.

#### Drawbacks
- **Feedback Hub degrades**: submissions and scenario-driven feedback prompts may stop working properly.
- **Less than it sounds**: `DmClient` has no trigger on 24H2, so disabling it stops nothing scheduled.
- **Updates can re-enable it**: feature updates can re-create or re-enable the tasks.
- **Overlaps with the telemetry service**: with `DiagTrack` disabled there is no upload path anyway.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" re-enables both tasks; System Default restores the snapshot.

#### Interactions
- **Disable User Experiences and Telemetry (DiagTrack)** (`disable_diagtrack`, this page) removes the transport these tasks would use.
- **Privacy: Turn off feedback request notifications** (`disable_feedback_notifications`) and **Privacy: Set feedback prompt frequency to Never** (`disable_feedback_frequency`) control the feedback prompts by policy; **Debloat: Remove Feedback Hub** (`remove_feedback_hub`) removes the app.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found that `DmClient` has an empty trigger set on 26100 (only `DmClientOnScenarioDownload` has a real trigger), so the tweak does not switch off a recurring transmission; the paths and option shape were correct. The shipped copy says so.
- **Confidence**: Community-corroborated for the binary's identity; the trigger observation is tier A because no enable or disable operation can produce an empty trigger set.
- **Reasoning**: The trigger observation survived the provenance rule. Open: the shipped enabled state of both tasks.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you do not use Feedback Hub, since there is nothing to lose. Leave it alone if you actively submit feedback to Microsoft and want the attached diagnostics to arrive.

#### Sources
1. Shipped task definitions on build 26100: `DmClient` has `<Triggers />` (empty), `DmClientOnScenarioDownload` has a `WnfStateChangeTrigger` (tier A for the trigger observation)
2. STRONTIC xcyclopedia, `dmclient.exe` identified as "Microsoft Feedback SIUF Deployment Manager Client", https://strontic.github.io/xcyclopedia/library/dmclient.exe-7E90BC5211DC802DEDFA345E234B0F95.html (tier C)
3. Configure Windows diagnostic data in your organization, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
4. Optimize-Offline `ScheduledTasks.json`, task `DmClientOnScenarioDownload` with description "Update SIUF...", https://github.com/DrEmpiricism/Optimize-Offline/blob/master/Content/Additional/Setup/ScheduledTasks.json (tier D, path form only)
5. windows10fixup `Fixup.ps1`, both full paths, https://github.com/iDigitalFlame/windows10fixup (tier D, path form only)

### Disable Windows Error Reporting QueueReporting task

`task_wer_queuereporting` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops queued crash reports being uploaded to Microsoft, while local crash logs stay on your PC.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `queuereporting` | task | `\Microsoft\Windows\Windows Error Reporting\QueueReporting` | none |

| Option | `queuereporting` |
|---|---|
| Disabled | `disabled` |
| Enabled | `enabled` |

System Default: with both task states authored, a readable task always matches an option, so System Default appears only when you restore a snapshot. The shipped enabled state is not established from a clean image (presumed Enabled; if so, a stock machine reads as "Enabled").

#### How it works

Windows Error Reporting queues crash reports that could not be sent at the time of the crash, and this task drains that queue by uploading them to Microsoft. Microsoft documents crash reporting and crash dumps as a diagnostic channel "managed by Windows Error Reporting", distinct from the Connected User Experiences and Telemetry component, so disabling this task is a real additional gain on top of the telemetry service tweak. Local error logging (`.wer` reports, Reliability Monitor) stays in place. Tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates.

#### Benefits
- **Crash data stays local**: reports can contain application and system state, and none of it leaves the machine.
- **Local logging intact**: `.wer` reports and Reliability Monitor keep working.
- **Separate from telemetry**: a distinct channel from the diagnostic data pipeline.

#### Drawbacks
- **Microsoft stops hearing about your crashes**: one input into the bug you are hitting getting fixed is gone.
- **Support cases suffer**: a support case that expects WER submissions has nothing to reference.
- **Updates can re-enable it**: feature updates can re-create or re-enable the task.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" re-enables the task; System Default restores the snapshot.

#### Interactions
- **Privacy: Disable the Windows Error Reporting service** (`disable_wer_service`) stops reports being generated in the first place, and **Privacy: Disable Windows Error Reporting** (`disable_error_reporting`) turns WER off by policy. This task tweak is the narrowest of the three: it only stops the queued upload.
- The research deliberately kept this task out of any merged telemetry-task group, because it is the one task a user may rationally want to keep so that Microsoft and OEMs receive crash reports.

#### Validation
- **Verdict**: VERIFIED. No correction was needed.
- **Confidence**: Community-corroborated for the task path (independent tooling), with Microsoft documentation for the queue-and-upload model and the separate channel.
- **Reasoning**: The path, option shape and immediate effect were confirmed. Open: the shipped enabled state.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you would rather crash contents stayed on your machine, since local diagnostics are unaffected. Leave it enabled if you are chasing a recurring crash and want Microsoft or an OEM to receive the reports.

#### Sources
1. Configure Windows diagnostic data in your organization, crash reporting managed by Windows Error Reporting as a distinct channel, https://learn.microsoft.com/en-us/windows/privacy/configure-windows-diagnostic-data-in-your-organization (tier A)
2. Windows Error Reporting, the queue and upload model, https://learn.microsoft.com/en-us/windows/win32/wer/windows-error-reporting (tier A)
3. Windows-10-Hardening, `components/scheduled_tasks.bat`, identical path, https://github.com/aghorler/Windows-10-Hardening/blob/master/components/scheduled_tasks.bat (tier D, path form only)
4. windows10fixup `Fixup.ps1`, identical path, https://github.com/iDigitalFlame/windows10fixup (tier D, path form only)

### Disable Offline Maps update tasks

`task_maps_update` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the background download of offline map updates and the notification about them.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `maps_update` | task | `\Microsoft\Windows\Maps\MapsUpdateTask` | none |
| `maps_toast` | task | `\Microsoft\Windows\Maps\MapsToastTask` | none |

| Option | `maps_update` | `maps_toast` |
|---|---|---|
| Disabled | `disabled` | `disabled` |

System Default: the toggle's "off" position, shown whenever the two tasks are not both disabled; selecting it restores the snapshot captured before you applied the tweak. There is deliberately no "Enabled" option, because the shipped enabled state of `MapsUpdateTask` on a clean 24H2 image is unestablished and an authored value would have to guess it; Restore Snapshot returns the tasks to exactly what they were on your machine.

#### How it works

`MapsUpdateTask` downloads updates to offline maps in the background. `MapsToastTask` raises the notification about them; it has an empty `<Triggers />` set and a `ComHandler` action on build 26100, so it has no schedule of its own and fires only when the Downloaded Maps Manager service (`MapsBroker`) invokes it. Disabling it therefore stops an invoked toast, not a scheduled one.

Microsoft states that Maps is no longer preinstalled with Windows starting with Windows 11 24H2, and deprecated the Maps app (May 2025) and the Maps platform APIs (8 April 2025), so on a clean 24H2 install both tasks may sit behind an app that is not installed. Tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates.

#### Benefits
- **No background downloads**: map data is large, so this saves real bandwidth on a metered link.
- **No update notifications**: the toast about new map data stops.
- **Pairs with the service**: the task half of the same subsystem as Downloaded Maps Manager.

#### Drawbacks
- **Offline maps stop updating**: downloaded regions go stale until refreshed by hand.
- **Manual updates still need the service**: an update can be triggered only while `MapsBroker` is enabled.
- **Little to stop on 24H2**: Maps is not preinstalled, so there may be no app behind these tasks.
- **Updates can re-enable it**: feature updates can re-create or re-enable the tasks.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: switch the toggle back to System Default to restore the snapshot, which returns each task to its captured state. There is no authored "Enabled" state.

#### Interactions
- **Disable Downloaded Maps Manager (MapsBroker)** (`disable_maps_broker`, this page) is the service half. With the service disabled `MapsToastTask` cannot fire at all. The research proposed one three-option "Offline Maps" tweak (on, background updates off, fully off), blocked on the same unknown stock state; they ship as two tweaks.

#### Validation
- **Verdict**: VERIFIED-WITH-CORRECTION. The research found that a stock "Enabled" value for `MapsUpdateTask` has no established provenance (the claim that it ships disabled rested on a modified machine and was withdrawn), and that `MapsToastTask` has no trigger of its own. The shipped tweak authors only the Disabled state and relies on the snapshot for the way back.
- **Confidence**: Community-corroborated for the task paths (independent tooling); tier A for the empty trigger set on `MapsToastTask` and for the Maps deprecation and 24H2 preinstall change.
- **Reasoning**: Under the research's harmful-revert rule, a revert value must come from evidence about Windows, not assumption, and because System Default is a selectable state a wrong literal would be written on the path a cautious user takes. Open: `MapsUpdateTask`'s shipped enabled state on a clean image.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you do not use offline maps, which on 24H2 covers most people since Maps is not preinstalled. Leave it off if you keep downloaded map regions current for offline navigation.

#### Sources
1. Shipped task definitions on build 26100: `MapsToastTask` has an empty `<Triggers />` set and a `ComHandler` action (tier A for the trigger observation only)
2. Resources for deprecated features, "Maps is no longer preinstalled with Windows starting with the Windows 11, version 24H2 release", https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features-resources (tier A)
3. Deprecated features in the Windows client, Maps app deprecated May 2025, Maps platform APIs deprecated 8 April 2025, https://learn.microsoft.com/en-us/windows/whats-new/deprecated-features (tier A)
4. windows-cleaner `Disable-Scheduled-Tasks.ps1`, both full paths, https://github.com/markulie/windows-cleaner/blob/main/Disable-Scheduled-Tasks.ps1 (tier D, path form only)
5. windows10fixup `Fixup.ps1`, identical paths, https://github.com/iDigitalFlame/windows10fixup (tier D, path form only)

### Disable DiskDiagnosticDataCollector task

`task_disk_diagnostic_datacollector` · Switch · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the hidden task that reports drive SMART data to Microsoft, without touching failing-disk warnings.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `disk_datacollector` | task | `\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector` | none |

| Option | `disk_datacollector` |
|---|---|
| Disabled | `disabled` |

System Default: the toggle's "off" position, shown whenever the task is not disabled; selecting it restores the snapshot captured before you applied the tweak. There is deliberately no "Enabled" option: the shipped enabled state of this task on a clean 24H2 image is unestablished, and authoring one would risk turning on a hidden collector that Windows may ship switched off.

#### How it works

The task's own description says it "reports general disk and system information to Microsoft for users participating in the Customer Experience Program", and its action is `%windir%\system32\rundll32.exe dfdts.dll,DfdGetDefaultPolicyAndSMART`. It is marked `<Hidden>true</Hidden>`, so Task Scheduler does not show it by default.

On build 26100 the task definition has an empty `<Triggers />` element and a `MaintenanceSettings` block (period 14 days, deadline 1 month). There is no weekly trigger and no trigger of any kind; the only scheduling hint makes it a candidate for Automatic Maintenance. Disabling it therefore stops no recurring run, and the reporting is CEIP-gated anyway.

The sibling task `Microsoft-Windows-DiskDiagnosticResolver` in the same folder is what surfaces the failing-disk (SMART predictive failure) warning. This tweak does not touch it, and you should not disable it.

#### Benefits
- **Drive data stays local**: SMART attributes and disk information are not sent to Microsoft.
- **Failing-disk warnings intact**: the separate Resolver task keeps warning you.
- **Hidden task made explicit**: surfaces a control you would otherwise not see.

#### Drawbacks
- **Nothing was scheduled**: on 24H2 the task has no trigger, so disabling it stops no recurring run.
- **CEIP-gated anyway**: the description ties reporting to Customer Experience Program participation.
- **Updates can re-enable it**: feature updates can re-create or re-enable the task.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021.
- **Takes effect**: immediately.
- **Reverting**: switch the toggle back to System Default to restore the snapshot. There is no authored "Enabled" state.

#### Interactions
- **Disable Autochk Proxy task** (`task_autochk_proxy`, this page) and **Privacy: Disable the Customer Experience Improvement Program** (`disable_ceip_tasks`) cover other CEIP uploaders. The research proposed grouping the CEIP tasks; they ship separately.

#### Validation
- **Verdict**: INCORRECT, per the July 2026 research: the task carries no trigger on 26100 (the weekly trigger in third-party references describes Windows 7), and a stock "Enabled" value has no established provenance. The shipped tweak describes the task as untriggered and authors only the Disabled state, which is the scope the research allowed pending a clean-image check.
- **Confidence**: Community-corroborated for the task description and action (third-party task reference); tier A for the empty trigger set, which no enable or disable operation can produce.
- **Reasoning**: The claim that the task ships disabled was withdrawn because it rested on a modified machine; the empty trigger set survived. Open: the shipped enabled state of both this task and the Resolver.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want the control to be explicit, but expect no observable change, because the task has no trigger on current builds. Skip it if you are looking for a difference you can measure.

#### Sources
1. Shipped task definition on build 26100: `<Hidden>true</Hidden>`, a `MaintenanceSettings` block and an empty `<Triggers />` element (tier A for the trigger observation only)
2. "Microsoft-Windows-DiskDiagnosticDataCollector" scheduled task reference, full path, description and action (its weekly-trigger claim describes Windows 7), https://windows.fyicenter.com/4257_Microsoft-Windows-DiskDiagnosticDataCollector_Scheduled_Task_on_Windows_7.html (tier C)
3. "Microsoft-Windows-DiskDiagnosticResolver" scheduled task reference, the separate SMART failure warning task, https://windows.fyicenter.com/4384_Microsoft-Windows-DiskDiagnosticResolver_Scheduled_Task_on_Windows_8.html (tier C)
4. CEIPEnable, the Customer Experience Improvement Program control, https://learn.microsoft.com/en-us/windows/win32/devnotes/ceipenable (tier A)
5. Windows-10-Hardening, `components/scheduled_tasks.bat`, identical path, https://github.com/aghorler/Windows-10-Hardening/blob/master/components/scheduled_tasks.bat (tier D, path form only)

### Disable Device Census tasks

`task_device_census` · Switch (2 options) · Risk: low · Elevation: admin · Reboot: no · Windows: all supported builds · Reversible: yes

**Stops the daily inventory that reports your hardware and configuration to Microsoft.**

#### What it changes

| Effect | Kind | Target | Flags |
|---|---|---|---|
| `device_census` | task | `\Microsoft\Windows\Device Information\Device` | none |
| `device_census_user` | task | `\Microsoft\Windows\Device Information\Device User` | `optional: true`, `if_missing: disabled` |

| Option | `device_census` | `device_census_user` |
|---|---|---|
| Disabled | `disabled` | `disabled` |
| Enabled | `enabled` | `enabled` |

System Default: the status shown when the two tasks are in different states; selecting it restores the snapshot. On a build without `Device User`, that task reads as disabled, so the tweak is driven by the `Device` task alone and "Enabled" is shown as unavailable there (the engine never creates tasks). The shipped enabled state of both tasks is not established from a clean image (presumed Enabled; if so, a stock machine reads as "Enabled").

#### How it works

The `Device` task runs `devicecensus.exe SystemCxt` daily, and `Device User` runs `devicecensus.exe UserCxt` at user logon. `devicecensus.exe` (`%WinDir%\System32\devicecensus.exe`) inventories the machine and reports hardware and configuration data used for update eligibility and targeting. Disabling only the system-context task would leave the user-context census running at every logon, which is why both are covered. `Device User` is not present on every build, hence `optional`. Both tasks exist on build 26100.

This does not block Windows Update; it removes an input to how updates are targeted. Windows Update for Business reports and Update Compliance read census data. Tasks under `\Microsoft\Windows\` can be re-created or re-enabled by feature updates.

#### Benefits
- **Stops a daily upload**: one of the few telemetry tasks with a real recurring schedule.
- **Covers both contexts**: the system and the user census both stop.
- **No user-facing feature**: nothing you interact with depends on the census.

#### Drawbacks
- **Update targeting degrades**: eligibility signals go stale, so feature update offers may be less well matched.
- **Enterprise reporting breaks**: Windows Update for Business reports and Update Compliance go quiet.
- **Updates can re-enable it**: feature updates can re-create or re-enable the tasks.

#### Applies to, takes effect, reverting
- **Applies to**: Windows 11 24H2 and newer, and Windows 10 IoT Enterprise LTSC 2021; `Device User` is not present on every build.
- **Takes effect**: immediately.
- **Reverting**: "Enabled" re-enables both tasks where they exist; System Default restores the snapshot.

#### Interactions
- **Disable User Experiences and Telemetry (DiagTrack)** (`disable_diagtrack`, this page) and this tweak both feed Windows Update for Business reports and Update Compliance. The research suggested keeping the census standalone rather than folding it into a CEIP task group, because it is the data source those enterprise reports depend on.

#### Validation
- **Verdict**: INCORRECT, per the July 2026 research, on the task path: no task named `Devicecensus` exists (that is the executable's name); the census tasks are `Device` and `Device User` in the `Device Information` folder. The shipped tweak targets those two tasks, with `Device User` optional, which is the correction the research specified.
- **Confidence**: Community-corroborated. Two third-party references walk the Task Scheduler folder and name the task `Device`; independent tooling disables the same path; direct enumeration on 26100 returned `Device` and `Device User`.
- **Reasoning**: Task presence is an image fact, so the 26100 enumeration survives the provenance rule. Open: the shipped enabled state of both tasks.
- **Tested**: Build validation (schema, ownership and conflict checks).

#### Recommendation
Apply it if you want less hardware and configuration data leaving the machine and accept slightly less precise update targeting. Leave it enabled on a machine that reports into Windows Update for Business reports or Update Compliance.

#### Sources
1. gHacks, "What is devicecensus.exe on Windows 10", walks the Device Information folder and names the task `Device`, https://www.ghacks.net/2019/09/23/what-is-devicecensus-exe-on-windows-10-and-why-does-it-need-internet-connectivity/ (tier C)
2. MajorGeeks, "What is Device Census or devicecensus.exe in Windows 10?", same path and task name, https://www.majorgeeks.com/content/page/what_is_devicecensus_exe.html (tier C)
3. windows10fixup `Fixup.ps1`, disables `Microsoft\Windows\Device Information\Device`, https://github.com/iDigitalFlame/windows10fixup (tier D, path form used by working tooling)
4. Direct enumeration of `C:\Windows\System32\Tasks\Microsoft\Windows\Device Information` on build 26100, returning `Device` and `Device User` (tier A for task presence)
