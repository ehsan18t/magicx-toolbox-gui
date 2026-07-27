# AI and Copilot tweak validation

Validated 2026-07-27. Proposed new corpus file: `src-tauri/tweaks/ai.yaml` (10 tweaks).
Scope: Windows 11 24H2 (build 26100) and newer, including 25H2 (26200), x64. Windows 10 IoT
Enterprise LTSC 2021 (19044) is a low-priority secondary target and is called out per tweak where the
control exists there or is inert.

This category does not exist yet. It is created here by consolidating the AI-specific controls that
are currently scattered across three shipped category files, plus four newly verified additions.

The reason for the consolidation is that the six existing tweaks are grouped by the surface they
happen to touch rather than by what they control. `disable_copilot_taskbar` sits in Interface because
it hides a taskbar button; `remove_copilot_app` sits in Debloat because it uninstalls a package;
`disable_recall_snapshots` sits in Privacy because snapshots are a privacy concern. A user who wants
to decide how much Windows AI runs on their machine currently has to visit three screens and know
which one holds which lever. Windows AI is now a subsystem in its own right: the `WindowsAI` Policy
CSP, the `WindowsCopilot.admx` template, the `WSAIFabricSvc` broker, and per-app AI policies in
Notepad, Paint and Edge. A category that mirrors that subsystem is easier to reason about and makes
the interactions between the levers visible, which matters because several of them only cover part
of the surface on their own.

The four additions (`disable_notepad_ai`, `disable_paint_ai`, `disable_edge_ai_features`,
`disable_ai_fabric_service`) close the obvious gaps: inbox app AI, browser AI, and the AI service
broker itself. Without them the category would cover Recall and Copilot and nothing else.

## Category definition

Proposed YAML category block for `src-tauri/tweaks/ai.yaml`:

```yaml
category:
  id: ai
  name: "AI & Copilot"
  icon: "mdi:robot-outline"
  description: "Windows AI controls in one place: Recall snapshots, Click to Do, the Copilot app and taskbar button, AI features in Notepad, Paint and Edge, and the AI service broker. Mostly policy values, reversible from snapshots."
```

`mdi:robot-outline` is **not currently registered** in `src/lib/components/shared/Icon.svelte`. Per
`CLAUDE.md` ("Any new icon must be imported in `Icon.svelte`"), adding this category requires adding
the import and the map entry. If a new icon import is unwanted, `mdi:magic-staff` and
`mdi:tune-variant` are already registered and would serve, though neither reads as "AI" on its own.

#### Corpus moves this implies

This is a change to the shipped corpus, not just a new file. Six tweaks move:

| Tweak id | Moves from | Moves to |
|---|---|---|
| `disable_recall_snapshots` | `src-tauri/tweaks/privacy.yaml` | `src-tauri/tweaks/ai.yaml` |
| `remove_recall_component` | `src-tauri/tweaks/privacy.yaml` | `src-tauri/tweaks/ai.yaml` |
| `disable_click_to_do` | `src-tauri/tweaks/privacy.yaml` | `src-tauri/tweaks/ai.yaml` |
| `remove_copilot_app` | `src-tauri/tweaks/debloat.yaml` | `src-tauri/tweaks/ai.yaml` |
| `remove_recall_feature` | `src-tauri/tweaks/debloat.yaml` | `src-tauri/tweaks/ai.yaml` |
| `disable_copilot_taskbar` | `src-tauri/tweaks/interface.yaml` | `src-tauri/tweaks/ai.yaml` |

Four are new and have no prior home: `disable_notepad_ai`, `disable_paint_ai`,
`disable_edge_ai_features`, `disable_ai_fabric_service`.

Two knock-on items follow from the moves:

1. **`debloat.yaml`'s category name is currently "Debloat, AI & Consumer".** With the AI tweaks gone
   it should become something like "Debloat & Consumer", and its description should drop "AI nags".
2. **Existing user snapshots and profiles reference these tweaks.** If profiles or snapshot files key
   on a category-qualified id, moving a tweak between files changes that key. Confirm how snapshots
   and exported profiles identify a tweak before the move lands; if the id is category-qualified,
   a migration or an alias is required so existing snapshots stay restorable. This is a
   pre-merge item, not a research finding, but it is the one that can silently break a revert.

## Verdict summary

| Tweak | Verdict | Risk | Confidence | Correction needed |
|---|---|---|---|---|
| `disable_recall_snapshots` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Build gate, edition gate, snapshot deletion not stated |
| `remove_recall_component` | VERIFIED-WITH-CORRECTION | medium | Microsoft-documented | "Available" option is wrong; needs a third option writing 1; risk level |
| `disable_click_to_do` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Copilot+ hardware gate, `requires_reboot` unsupported |
| `remove_copilot_app` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Provisioned package not removed, probe misses provisioned state, second Store listing |
| `remove_recall_feature` | INCORRECT | medium | Microsoft-documented | Probe, exit code handling, and undo are all broken |
| `disable_copilot_taskbar` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Stale Win+C claim, Home no-op, deprecation not stated |
| `disable_notepad_ai` | VERIFIED | low | Microsoft-documented | none |
| `disable_paint_ai` | VERIFIED | low | Microsoft-documented | none |
| `disable_edge_ai_features` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Three values are Entra-only; `EdgeCopilotEnabled` must not ship; Edge version floors |
| `disable_ai_fabric_service` | VERIFIED-WITH-CORRECTION | low | Microsoft-documented | Display name, build gate, memory claim overstated |

## Corrections required

1. **`disable_recall_snapshots`, build gate too wide.** The YAML has
   `windows: { build: ">=26100" }`. The Policy CSP applicability for `WindowsAI/DisableAIDataAnalysis`
   is Windows 11 24H2 **with KB5055627, build 10.0.26100.3915** and later. On 26100 builds below
   3915 the write is inert. Correct gate: `>=26100.3915`.
2. **`disable_recall_snapshots`, edition gate missing.** The policy's supported editions are Pro,
   Enterprise, Education, IoT Enterprise and IoT Enterprise LTSC. **Home is not listed.** The tweak
   carries no edition gate and the copy does not say the write is expected to be inert on Home.
3. **`disable_recall_snapshots`, permanent data loss not stated.** Microsoft documents that enabling
   this policy also **deletes snapshots already stored on the device**. That deletion is not undone
   by reverting the registry value. The current `info` text does not say this, and `reversible: true`
   is accurate for the value while being misleading about the outcome. State it in Drawbacks.
4. **`remove_recall_component`, the "Available (Stock Default)" option does not make Recall
   available.** The YAML's default option writes `absent`. Microsoft documents that when
   `AllowRecallEnablement` is **not configured**, "end users will have the Recall component in a
   disabled state". Only an explicit `1` makes Recall available. So the option label promises
   something the option does not do, and a user who applies then reverts does not get Recall back.
   The current `info` text's claim "Fully reversible by setting the value back to 1" describes an
   option the tweak does not have. Correct shape is three options: `Removed` = 0,
   `Not configured (Stock Default)` = `absent`, `Available` = 1. This is the case recorded in
   `_harmful-revert.md` line 59 as "writes `absent` where a literal is required".
5. **`remove_recall_component`, risk level too low.** `risk_level: low` understates a tweak whose
   apply deletes snapshots, removes the feature payload, and whose revert path does not restore the
   pre-apply state without an explicit re-provision. Raise to `medium`.
6. **`disable_click_to_do`, gate is a build number where the real gate is hardware.** The feature
   requires a Copilot+ PC (40 TOPS NPU, 16 GB RAM, 8 logical processors, 256 GB storage) or an
   eligible Cloud PC. `windows: { build: ">=26100" }` admits every 24H2 machine, on the great
   majority of which the value is inert. Add the hardware prerequisite to the copy, and note that
   the Policy CSP applicability column still reads "Windows Insider Preview", so Microsoft does not
   guarantee the behaviour on a given retail 24H2 or 25H2 build.
7. **`disable_click_to_do`, `requires_reboot: true` is unsupported.** No Microsoft source documents a
   reboot for this policy, and the same setting is user-togglable at Settings > Privacy & security >
   Click to Do without one. Set `requires_reboot: false`.
8. **`remove_copilot_app`, the provisioned package is not removed.** Apply runs only
   `Get-AppxPackage -AllUsers 'Microsoft.Copilot' | Remove-AppxPackage -AllUsers`. Without a matching
   `Get-AppxProvisionedPackage -Online | Where-Object PackageName -like 'Microsoft.Copilot*' |
   Remove-AppxProvisionedPackage -Online`, the app returns for any newly created user profile and is
   re-provisioned by feature updates.
9. **`remove_copilot_app`, the probe cannot see the provisioned state.** `if (Get-AppxPackage -AllUsers
   'Microsoft.Copilot') { exit 1 } else { exit 0 }` reports Applied while the provisioned package is
   still staged and will reappear. Extend the probe to check both installed and provisioned state.
10. **`remove_copilot_app`, second Store listing not mentioned.** Store product `9NHT9RB2F4HD` is the
    Appx "Microsoft Copilot on Windows" with package family name `Microsoft.Copilot_8wekyb3d8bbwe`,
    which apply, probe and undo all agree on. A second listing, `XP9CXNGPPJ97XX` "Microsoft Copilot",
    now exists and returns no package family name at all, indicating a non-Appx delivery path. The
    copy should say this tweak targets the Appx one. Related: the `build: ">=26100"` gate is narrower
    than reality (the app is present on 23H2 / 22631 too). Harmless given this project's 24H2 target,
    but record it.
11. **`remove_recall_feature`, the probe reports the wrong state.** The probe is
    `if ($s -eq 'Disabled') { exit 0 } else { exit 1 }`. On a machine inspected at build 26100.4061
    the real state is `DisabledWithPayloadRemoved`, which both DISM and `Get-WindowsOptionalFeature`
    use to mean "off and payload not staged locally". The strict equality reports "not applied" on a
    machine where Recall is more thoroughly removed than the tweak itself achieves. Accept both
    `Disabled` and `DisabledWithPayloadRemoved`.
12. **`remove_recall_feature`, DISM's success code is read as a failure.** With `/NoRestart`, DISM
    returns 3010 (`ERROR_SUCCESS_REBOOT_REQUIRED`) when the operation succeeded and a restart is
    needed, which is the normal outcome for disabling an optional feature. The engine's
    `run_and_require_zero` treats any non-zero code as `ActionFailed`, so a successful disable is
    reported as a failure. Accept 3010 as success.
13. **`remove_recall_feature`, the undo installs a feature the machine may never have had.** Undo is
    an unconditional `DISM /Online /Enable-Feature /FeatureName:Recall /NoRestart`. On any machine
    whose stock state was `Disabled` or `DisabledWithPayloadRemoved`, which is every machine that is
    not a Copilot+ PC with Recall on, reverting installs Recall rather than restoring the prior
    state. Where the payload is not staged, the enable also fails with 0x800F081F because no
    `/Source` is given. Capture the pre-apply feature state and branch the undo on it.
14. **`remove_recall_feature`, the copy overstates what apply does.** `/Disable-Feature` without
    `/Remove` turns the feature off and leaves the payload on disk. The `info` text claims "the
    Recall component is gone from the machine, not merely paused". Either add `/Remove` and document
    that re-enabling then needs a `/Source`, or correct the text.
15. **`remove_recall_feature`, applicability text is wrong about who has the feature.** The `Recall`
    optional feature entry is enumerated on ordinary Windows 11 24H2 x64, not only on Copilot+ PCs;
    it was observed on IoT Enterprise LTSC 2024 (26100.4061), a SKU where Recall itself is not
    offered. What is Copilot+ gated is the Recall experience, not the feature entry. The
    `build: ">=26100"` gate and `requires_reboot: true` are both correct.
16. **`disable_copilot_taskbar`, the Win+C claim is stale.** The `description` and `info` both say the
    policy blocks the Win+C shortcut. As of the May 2025 optional preview, Win+C and the Copilot
    hardware key open the Microsoft 365 Copilot prompt box, a different surface this policy does not
    govern. Drop the claim.
17. **`disable_copilot_taskbar`, no SKU gate.** Both the shipped `WindowsCopilot.admx`
    (`supportedOn` `SUPPORTED_Windows_11_0_NOSERVER_ENTERPRISE_EDUCATION_PRO_SANDBOX`) and the Policy
    CSP exclude Home. The write is expected to be a no-op there. Add an edition gate or say so
    plainly in the copy.
18. **`disable_copilot_taskbar`, the deprecation is not stated.** Microsoft Learn says "AppLocker
    policy should be used instead of the Turn Off Windows Copilot legacy policy setting and its MDM
    equivalent, TurnOffWindowsCopilot. The policy is subject to near-term deprecation", and the
    Policy CSP page carries "This policy is deprecated and may be removed in a future release". The
    ADMX still ships on 26100 and the shell still reads the value, so the tweak works today, but the
    copy should name the deprecation and the supported replacements (an AppLocker publisher rule for
    `MICROSOFT.COPILOT`, or removing the `Microsoft.Copilot` package).
19. **`disable_edge_ai_features`, three of the five values are Entra-only.** `CopilotPageContext`,
    `EdgeEntraCopilotPageContext` and `ComposeInlineEnabled` are documented by Microsoft as applying
    only to Microsoft Entra ID profiles. On a Microsoft account profile the write **succeeds and
    does nothing**. That is a harder version of the did-it-work problem than a failure would be: the
    app reports Applied and Copilot in Edge carries on reading pages. The limitation must be explicit
    in the option labels and in Drawbacks. Do **not** use `skip_validation: true` to paper over it.
20. **`disable_edge_ai_features`, `EdgeCopilotEnabled` must not ship.** The proposal suggested adding
    `EdgeCopilotEnabled` = 0 as a third option value. Microsoft's page for it lists **Windows: Not
    supported**, macOS: Not supported, Android and iOS 123+ only, and carries no Windows registry
    settings section at all. Writing it to `HKLM\SOFTWARE\Policies\Microsoft\Edge` on Windows does
    nothing. Also do not add `CopilotCDPPageContext`, marked "(obsolete)" on the Edge policy index
    and superseded by `EdgeEntraCopilotPageContext`.
21. **`disable_edge_ai_features`, Edge version floors missing.** Each value has its own minimum Edge
    version: `ComposeInlineEnabled` 115+, `NewTabPageBingChatEnabled` 117+, `CopilotPageContext`
    124+, `EdgeEntraCopilotPageContext` 130+, `EdgeHistoryAISearchEnabled` 138+. A machine on Edge
    120 ignores the last one. Put the floors in the copy. The `windows:` build gate cannot express
    this because Edge updates independently of the OS.
22. **`disable_ai_fabric_service`, the display name is wrong.** The proposal gives "Windows AI Fabric
    Service". Resolving the MUI resource `System32\en-US\WSAIFabricHost.dll.mui` gives
    `DisplayName: WSAIFabricSvc` and `Description: Provides support to communicate with AIFabric in
    Local service context over COM.` The display name is literally `WSAIFabricSvc`. Matching or
    displaying on the friendly name will not match.
23. **`disable_ai_fabric_service`, the build gate would skip the primary target.** The proposal
    gates `build >= 26200` on the claim that the service does not exist on 26100. It does exist on
    26100: the service is registered on a 26100.4061 machine and its host binary
    `C:\Windows\System32\WSAIFabricHost.dll` is a Microsoft-shipped 26100 file
    (FileVersion 10.0.26100.3624, FileDescription `WSAIFabricSvc`, timestamp May 2025). The service
    arrived in 24H2 servicing around 26100.3624. Correct gate: `build >= 26100` **plus a presence
    check that fails cleanly when the service is not registered**, because 26100 servicing levels
    below roughly 26100.3624 will not have it. The service effect must treat "service not registered"
    as a skip, not an error.
24. **`disable_ai_fabric_service`, the memory claim does not generalise.** The proposal justifies the
    tweak with a `WorkloadsSessionHost.exe` memory story ("eight processes, multi-gigabyte
    resident"). A recursive search of all of `C:\Windows` for `WorkloadsSessionHost*` on a 26100.4061
    host returns nothing, and zero such processes run even while `WSAIFabricSvc` is running.
    `WSAIFabricHost.dll` contains `AIFabric` and `Workloads` but not `WorkloadsSessionHost`,
    `WorkloadSession` or `SessionHost` in either encoding. Every report of the memory symptom is from
    an NPU-equipped or Copilot+ machine. The service is universal; the memory symptom is not. The
    copy must not promise a multi-gigabyte win on ordinary hardware.
25. **`disable_ai_fabric_service`, drop two sources.** `batcmd.com` and `revertservice.com` both state
    the 24H2 start type is Manual. They are contradicted by every other line of evidence, they
    publish the identical wrong value, and neither tracks 25H2. Two auto-generated service catalogs
    carrying the same wrong figure is one derivative source, and it is wrong. Remove both from the
    tweak's source list.

## Tweak entries

### `disable_recall_snapshots` Recall snapshots

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**

- Key: `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`
- Value name: `DisableAIDataAnalysis`
- Type: `REG_DWORD`
- Options (2, so a toggle):
  - `Disabled` writes `1` (snapshots are not saved)
  - `Enabled (Stock Default)` writes `absent` (documented Default Value 0, snapshots may be saved
    subject to user opt-in)
- `elevation: admin`, `reversible: true`, `requires_reboot: true`
- Corrected gate: `windows: { build: ">=26100.3915" }` plus editions Pro, Enterprise, Education, IoT
  Enterprise, IoT Enterprise LTSC. **Current wrong gate:** `windows: { build: ">=26100" }` with no
  edition restriction.

This is the group policy "Turn off saving snapshots for use with Recall", Policy CSP
`WindowsAI/DisableAIDataAnalysis`. Microsoft documents Format `int`, Default Value 0, allowed values
0 (snapshots may be saved) and 1 (snapshots not saved), and states that enabling the policy also
deletes any snapshots already stored on the device. A user-scope variant exists at
`HKCU\Software\Policies\Microsoft\Windows\WindowsAI`; the tweak's machine-scope write is the stronger
of the two and is the correct choice.

Recall itself requires a Copilot+ PC meeting the Secured-core standard: a 40 TOPS NPU, 16 GB RAM,
8 logical processors, 256 GB storage, device encryption or BitLocker, and Windows Hello Enhanced
Sign-in Security with a biometric enrolled. On hardware without those, the policy has no feature to
act on. Microsoft also states that on managed devices snapshots are not enabled by default and that
enabling saving requires individual user opt-in consent, so on a commercial device the policy largely
locks in the existing state. Not present on Windows 10 LTSC 2021.

**Corrections needed:** Corrections 1, 2 and 3. Build gate should be `>=26100.3915` (KB5055627), not
`>=26100`. Home is not a supported edition and needs an edition gate or an honest note. The `info`
text must state that applying permanently deletes snapshots already on the device, which reverting
the value does not bring back.

**Ready-to-paste info block:**

```yaml
    info: |
      **Locks Recall's screen-snapshot capture off for every user on the PC, and deletes the snapshots already saved.**

      ## What it does
      Sets the `DisableAIDataAnalysis` policy under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`
      to 1, the group policy "Turn off saving snapshots for use with Recall". Windows stops saving
      Recall snapshots and, per Microsoft, deletes any snapshots already stored on the device.

      ## Benefits
      - **No new snapshots**: Recall stops writing periodic screen captures to local storage
      - **Existing data cleared**: snapshots already saved on the device are deleted
      - **Machine-wide**: applies to every account, not just the one that applied it

      ## Drawbacks
      - **Recall stops working**: the timeline and its search go away for all users on the PC
      - **Deletion is permanent**: reverting the policy restores the setting, never the deleted snapshots
      - **Inert on most PCs**: Recall needs a Copilot+ machine, so on other hardware nothing changes
      - **Not read on Home**: the policy's supported editions are Pro, Enterprise, Education and IoT Enterprise

      ## Good to know
      - **Applies to**: Windows 11 24H2 build 26100.3915 (KB5055627) and newer; earlier 26100 builds ignore the value
      - **Takes effect**: after reboot, which also lets the snapshot deletion settle
      - **Reverting**: restores the previous value from the snapshot; the deleted Recall data does not come back
      - Recall additionally requires a Copilot+ PC (40 TOPS NPU, 16 GB RAM, 8 logical processors, 256 GB storage, device encryption, and Windows Hello Enhanced Sign-in Security)

      ## Recommendation
      Apply it on any Copilot+ PC where you want certainty that screen capture stays off, and accept
      that the existing snapshots go with it. If you actively use Recall's timeline, leave it alone;
      this removes the choice rather than presenting it.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - WindowsAI](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
      - [Manage Recall for Windows clients](https://learn.microsoft.com/en-us/windows/client-management/manage-recall)
```

**Sources:**

1. Policy CSP - WindowsAI, `DisableAIDataAnalysis` (Format `int`, Default Value 0, allowed values 0
   and 1, applicability Windows 11 24H2 with KB5055627 / 10.0.26100.3915 and later, editions Pro /
   Enterprise / Education / IoT Enterprise / IoT Enterprise LTSC, both Device and User scope),
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
2. Manage Recall for Windows clients (Copilot+ hardware and Secured-core prerequisites, snapshot
   deletion on enabling the policy, managed-device default and user opt-in consent),
   https://learn.microsoft.com/en-us/windows/client-management/manage-recall (tier A)

---

### `remove_recall_component` Recall feature component

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**

- Key: `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`
- Value name: `AllowRecallEnablement`
- Type: `REG_DWORD`
- **Corrected options (3, so a dropdown):**
  - `Removed` writes `0` (component disabled, Recall bits removed from the device, saved snapshots
    deleted, restart required)
  - `Not configured (Stock Default)` writes `absent` (component present but in a disabled state)
  - `Available` writes `1` (Recall becomes available, then governed by `DisableAIDataAnalysis`)
- **Current wrong options (2):** `Removed` = 0, and `Available (Stock Default)` = `absent`. The
  second label claims to make Recall available. It does not. Microsoft documents that with the
  policy not configured, "end users will have the Recall component in a disabled state". There is no
  option in the shipped tweak that writes `1`, so nothing in the tweak can make Recall available,
  and the `info` text's "Fully reversible by setting the value back to 1" describes an option that
  does not exist.
- `elevation: admin`, `reversible: true`, `requires_reboot: true` (correct, the component removal
  genuinely needs a restart)
- Corrected `risk_level: medium`. **Current wrong value:** `low`.
- Gate: `windows: { build: ">=26100.3915" }`, device scope only (no user scope), editions Pro,
  Enterprise, Education, IoT Enterprise, IoT Enterprise LTSC.

This is Policy CSP `WindowsAI/AllowRecallEnablement`, group policy "Allow Recall to be enabled". It
is the strongest available lever because it removes the feature payload rather than gating it.
Restoring availability needs a policy value of 1 and, on some commercial devices, an explicit
`Enable-WindowsOptionalFeature -Online -FeatureName "Recall"`. Not present on Windows 10 LTSC 2021.

**Corrections needed:** Corrections 4 and 5. Replace the two-option shape with the three-option shape
above so the default option's label matches Microsoft's documented semantics and so a real
"Available" path exists. Raise `risk_level` from `low` to `medium`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Strips the Recall component off the device instead of just switching it off.**

      ## What it does
      Sets the `AllowRecallEnablement` policy under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`.
      Set to 0, Microsoft removes the Recall bits from the device and deletes saved snapshots. Left
      unconfigured, the component stays on disk in a disabled state. Set to 1, Recall becomes
      available again, subject to the separate snapshot policy.

      ## Benefits
      - **Payload removed**: the feature is gone from disk, not merely gated behind a toggle
      - **Nothing to re-enable**: no leftover component a setting or update can switch back on
      - **Snapshots deleted**: saved Recall data is removed along with the component

      ## Drawbacks
      - **Restoring is work**: putting Recall back needs the Available option and may need `Enable-WindowsOptionalFeature -Online -FeatureName "Recall"`
      - **Data loss is permanent**: deleted snapshots do not return on any option
      - **Not configured is not available**: the stock state leaves Recall present but disabled, so reverting does not hand Recall back
      - **Inert on most PCs**: Recall needs a Copilot+ machine, and the policy is not read on Home

      ## Good to know
      - **Applies to**: Windows 11 24H2 build 26100.3915 (KB5055627) and newer, Pro / Enterprise / Education / IoT Enterprise editions
      - **Takes effect**: after reboot, which Microsoft requires for the component removal
      - **Reverting**: restores the previous value from the snapshot, which returns the machine to the not-configured state where Recall is present but disabled
      - Machine scope only; there is no per-user version of this policy

      ## Recommendation
      Pick this over the snapshot-only tweak if you want Recall gone rather than held off and you are
      confident you will not want it back. If there is any chance you will try Recall later, use the
      snapshot policy instead, which is a clean on-off.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Policy CSP - WindowsAI](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
      - [Manage Recall for Windows clients](https://learn.microsoft.com/en-us/windows/client-management/manage-recall)
```

**Sources:**

1. Policy CSP - WindowsAI, `AllowRecallEnablement` (three documented states, device scope only,
   restart required on disable, snapshot deletion, applicability Windows 11 24H2 with KB5055627 /
   10.0.26100.3915 and later, editions Pro / Enterprise / Education / IoT Enterprise / IoT Enterprise
   LTSC), https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
2. Manage Recall for Windows clients (re-provisioning via
   `Enable-WindowsOptionalFeature -Online -FeatureName "Recall"` on commercial devices, Copilot+
   hardware prerequisites), https://learn.microsoft.com/en-us/windows/client-management/manage-recall
   (tier A)
3. `_harmful-revert.md` line 59, this repository: the "Available" option writes `absent` where a
   literal `1` is required, recorded as one of the corpus's harmful-revert cases (internal analysis)

---

### `disable_click_to_do` Click to Do overlay

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**

- Key: `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI`
- Value name: `DisableClickToDo`
- Type: `REG_DWORD`
- Options (2, so a toggle):
  - `Disabled` writes `1` (the Click to Do component and its entry points are not available to users)
  - `Enabled (Stock Default)` writes `absent` (documented Default Value 0, Click to Do enabled)
- `elevation: admin`, `reversible: true`
- Corrected `requires_reboot: false`. **Current wrong value:** `true`, which no Microsoft source
  supports.
- Corrected applicability: `windows: { build: ">=26100" }` **plus a Copilot+ hardware prerequisite
  stated in the copy**. **Current wrong shape:** a bare build gate that admits every 24H2 machine.

This is Policy CSP `WindowsAI/DisableClickToDo`, group policy "Disable Click to Do" at Windows
Components > Windows AI, available in both Computer and User Configuration, with an ADMX registry key
beginning `SOFTWARE\Policies\`. Microsoft documents Default Value 0, value 1 disables, and that when
enabled "the Click to Do component and entry points won't be available to users".

Two caveats that must survive. First, screenshot analysis is performed locally on the device in all
cases, so this removes a capture surface rather than blocking a cloud upload. Second, Microsoft notes
the policy **does not affect Click to Do inside Recall**, so pairing with the Recall tweaks is
required for full coverage. A third: the Policy CSP applicability column still reads "Windows Insider
Preview", which is the only Microsoft statement on GA availability found, so behaviour on a given
retail 24H2 or 25H2 build is not guaranteed by Microsoft's own table. Editions are Pro, Enterprise,
Education, IoT Enterprise and IoT Enterprise LTSC. Users can also turn it off per user at Settings >
Privacy & security > Click to Do. Not present on Windows 10 LTSC 2021.

**Corrections needed:** Corrections 6 and 7. Add the Copilot+ hardware prerequisite and the Insider
Preview applicability caveat to the copy rather than relying on a bare build gate, and set
`requires_reboot: false`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Click to Do overlay, so nothing screenshots and analyzes your screen on demand.**

      ## What it does
      Sets the `DisableClickToDo` policy under `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsAI` to 1.
      Microsoft's documented behaviour is that the Click to Do component and its entry points stop
      being available to users, so the on-demand full-screen capture and its on-device analysis path
      go away.

      ## Benefits
      - **No capture surface**: the on-demand full-screen grab and its entry points are removed
      - **Machine-wide**: covers every account, unlike the per-user Settings toggle
      - **Removes, not hides**: the entry points go, rather than a switch being greyed out

      ## Drawbacks
      - **Loses the actions**: Summarize, Rewrite and the other Click to Do actions become unavailable
      - **Not the whole story**: Microsoft states this does not disable Click to Do inside Recall
      - **Inert on most PCs**: Click to Do needs a Copilot+ machine or an eligible Cloud PC
      - **Not a cloud block**: the analysis already ran on-device, so this removes capture, not upload

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer on a Copilot+ PC (40 TOPS NPU, 16 GB RAM, 8 logical processors, 256 GB storage) or an eligible Cloud PC; Pro / Enterprise / Education / IoT Enterprise editions
      - **Takes effect**: immediately; sign out and back in if an entry point is still visible
      - **Reverting**: restores the previous value from the snapshot
      - Microsoft's policy page still lists this under Windows Insider Preview, so behaviour on a given retail build is not guaranteed by Microsoft's own table

      ## Recommendation
      Apply it on a Copilot+ PC if you have no interest in the overlay, and pair it with the Recall
      tweaks, since this one does not cover Click to Do inside Recall. On non-Copilot+ hardware it is
      harmless but changes nothing you can see.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - WindowsAI](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
      - [Manage Click to Do for Windows clients](https://learn.microsoft.com/en-us/windows/client-management/manage-click-to-do)
```

**Sources:**

1. Policy CSP - WindowsAI, `DisableClickToDo` (Default Value 0, value 1 disables, both Device and
   User scope, ADMX key beginning `SOFTWARE\Policies\`, applicability column reading "Windows Insider
   Preview", editions Pro / Enterprise / Education / IoT Enterprise / IoT Enterprise LTSC),
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
2. Manage Click to Do for Windows clients (entry points removed when the policy is enabled, on-device
   analysis, the statement that the policy does not affect Click to Do inside Recall, Copilot+ and
   Cloud PC prerequisites, Settings > Privacy & security > Click to Do per-user path),
   https://learn.microsoft.com/en-us/windows/client-management/manage-click-to-do (tier A)

---

### `remove_copilot_app` Copilot app

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** a state marker plus a PowerShell action.

- State marker: `HKCU\Software\MagicXToolbox\Debloat`, value `Copilot`, `REG_DWORD`
- Action, `shell: powershell`:
  - **Corrected apply:**
    ```
    Get-AppxPackage -AllUsers 'Microsoft.Copilot' | Remove-AppxPackage -AllUsers
    Get-AppxProvisionedPackage -Online | Where-Object { $_.PackageName -like 'Microsoft.Copilot*' } | Remove-AppxProvisionedPackage -Online
    ```
  - **Current wrong apply:** the first line only, so the provisioned package is left staged and the
    app returns for new user profiles and after feature updates.
  - Undo: `winget install --id 9NHT9RB2F4HD -e --accept-source-agreements --accept-package-agreements`
  - **Corrected probe:**
    ```
    if ((Get-AppxPackage -AllUsers 'Microsoft.Copilot') -or (Get-AppxProvisionedPackage -Online | Where-Object { $_.PackageName -like 'Microsoft.Copilot*' })) { exit 1 } else { exit 0 }
    ```
  - **Current wrong probe:** `if (Get-AppxPackage -AllUsers 'Microsoft.Copilot') { exit 1 } else { exit 0 }`,
    which reports Applied while the provisioned package is still staged.
- Options (2, so a toggle): `Removed` = `{ state: 1, app: run }`, `Installed (Stock Default)` =
  `{ state: 0 }`
- `elevation: admin` (required for `-AllUsers`), `reversible: true`, no reboot
- Gate: `windows: { build: ">=26100" }`, which is narrower than reality (the app is present on 23H2 /
  22631 too) but correct for this project's 24H2 target.

Store product `9NHT9RB2F4HD` resolves in Microsoft's Store catalog to title "Microsoft Copilot on
Windows" with package family name `Microsoft.Copilot_8wekyb3d8bbwe`, so apply, probe and undo all
agree on one identity, and `winget show --id 9NHT9RB2F4HD --source msstore` resolves today with
publisher Microsoft Corporation, so the undo is executable. The removal does not touch
`Microsoft.Windows.Ai.Copilot.Provider`, a separate component.

Two things that must survive into the copy. Copilot's implementation has changed repeatedly (sidebar,
then PWA, then a WebView2 build, then a WinUI native app, and in 2026 back to a WebView-based build
that ships its own copy of Edge), so this is a target that needs re-validation each release. And
there is now a second Store listing, `XP9CXNGPPJ97XX` "Microsoft Copilot", which the catalog returns
with no package family name at all, indicating a non-Appx delivery path this tweak does not cover.
On Enterprise, Education and IoT Enterprise running 24H2 or later the documented alternative is the
`RemoveMicrosoftCopilotApp` policy, which Microsoft gates on Copilot and Microsoft 365 Copilot both
being installed, the app not having been user-installed, and no launch in the last 28 days. The app
is not present on Windows 10 LTSC 2021.

**Corrections needed:** Corrections 8, 9 and 10. Add `Remove-AppxProvisionedPackage` to the apply so
the removal survives new user profiles and feature updates; extend the probe to check provisioned
state; note in the copy that a second, non-Appx Store listing for Copilot now exists and that this
tweak targets the Appx one. Optionally widen the `build` gate, which is narrower than reality.

**Ready-to-paste info block:**

```yaml
    info: |
      **Uninstalls the Copilot app so it stops appearing in Start, on the taskbar, and in your program list.**

      ## What it does
      Removes the `Microsoft.Copilot` Store package for all users and removes the provisioned copy
      that Windows stages for new accounts, so the app is off disk rather than hidden. It does not
      touch `Microsoft.Windows.Ai.Copilot.Provider`, the separate component behind Click to Do and
      right-click AI actions.

      ## Benefits
      - **Off disk**: the app is uninstalled, not just hidden from the taskbar
      - **Survives new profiles**: removing the provisioned package stops it reappearing for new accounts
      - **No background app**: nothing left to launch or update

      ## Drawbacks
      - **Copilot is gone**: no assistant from Start, taskbar, or the app list
      - **Updates can restore it**: a feature update can re-provision the app, so it may need re-applying
      - **Not the only Copilot**: a second, non-Appx Store listing for Copilot now exists and this tweak does not cover it
      - **Moving target**: Copilot's packaging has changed repeatedly, so the removal may need revisiting each release

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer (the app also exists on 23H2)
      - **Takes effect**: immediately, no reboot needed
      - **Reverting**: reinstalls the app from the Microsoft Store via winget, which needs an internet connection
      - On Enterprise, Education and IoT Enterprise the managed alternative is Microsoft's `RemoveMicrosoftCopilotApp` policy

      ## Recommendation
      Apply it if you do not use Copilot; it is the cleanest way to get rid of it and there is no
      cost beyond losing the assistant. Leave it if you use Copilot at all, since reinstalling
      depends on the Store being reachable.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - WindowsAI, RemoveMicrosoftCopilotApp](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
      - [Manage Windows Copilot](https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot)
      - [Remove-AppxProvisionedPackage](https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage)
```

**Sources:**

1. Microsoft Store catalog service, product `9NHT9RB2F4HD`, returning title "Microsoft Copilot on
   Windows" and package family name `Microsoft.Copilot_8wekyb3d8bbwe`,
   https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NHT9RB2F4HD?market=US&locale=en-us&deviceFamily=Windows.Desktop
   (tier A, primary observation)
2. Policy CSP - WindowsAI, `RemoveMicrosoftCopilotApp` (managed alternative and its three
   preconditions) and `TurnOffWindowsCopilot` (marked deprecated),
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
3. Manage Windows Copilot (documents the 24H2 Store-app model and `Remove-AppxPackage` on
   `Microsoft.Copilot` as a supported removal route),
   https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot (tier A)
4. `Remove-AppxProvisionedPackage` cmdlet reference,
   https://learn.microsoft.com/en-us/powershell/module/dism/remove-appxprovisionedpackage (tier A)
5. Microsoft Store catalog service, product `XP9CXNGPPJ97XX` "Microsoft Copilot", returning no
   package family name (tier A, primary observation)
6. "New Copilot for Windows 11 includes a full Microsoft Edge package", Windows Latest,
   https://www.windowslatest.com/2026/04/05/new-copilot-for-windows-11-includes-a-full-microsoft-edge-package-uses-more-ram/
   (tier D, used only for the packaging-history narrative)

---

### `remove_recall_feature` Recall optional feature

**Verdict:** INCORRECT

The feature name is right. Everything around it is wrong in a way that breaks state reporting or
revert, and the revert is the worst of it: it can install Recall on a machine that never had it.
The info block below is written for the **corrected** mechanism and should not be pasted until the
action is fixed.

**Mechanism:** a state marker plus a PowerShell action.

- State marker: `HKCU\Software\MagicXToolbox\Debloat`, value `RecallFeature`, `REG_DWORD`
- Action, `shell: powershell`:
  - **Corrected apply:** `DISM /Online /Disable-Feature /FeatureName:Recall /NoRestart`, with exit
    code **3010 accepted as success** alongside 0. Add `/Remove` only if payload removal is genuinely
    the intent, and then document that re-enabling needs a `/Source`.
  - **Current wrong apply:** the same command, but the engine's `run_and_require_zero` treats DISM's
    3010 (`ERROR_SUCCESS_REBOOT_REQUIRED`) as `ActionFailed`. 3010 is the normal outcome for
    disabling an optional feature with `/NoRestart`, so a successful disable is reported as a failure.
  - **Corrected undo:** capture the pre-apply feature state and branch on it. If the stock state was
    `Disabled` or `DisabledWithPayloadRemoved`, the undo must do nothing.
  - **Current wrong undo:** `DISM /Online /Enable-Feature /FeatureName:Recall /NoRestart`,
    unconditionally. On any machine whose stock state was `Disabled` or `DisabledWithPayloadRemoved`
    (which is every machine that is not a Copilot+ PC with Recall on) this installs Recall rather
    than restoring the prior state, and where the payload is not staged it fails with 0x800F081F
    because no `/Source` is given.
  - **Corrected probe:**
    ```
    $s = (Get-WindowsOptionalFeature -Online -FeatureName Recall -ErrorAction SilentlyContinue).State
    if ($s -eq 'Disabled' -or $s -eq 'DisabledWithPayloadRemoved') { exit 0 } else { exit 1 }
    ```
  - **Current wrong probe:** `if ($s -eq 'Disabled') { exit 0 } else { exit 1 }`. On a machine
    inspected at 26100.4061 the real state is `DisabledWithPayloadRemoved`, so the probe reports "not
    applied" where Recall is more thoroughly removed than the tweak itself achieves.
- Options (2, so a toggle): `Removed` = `{ state: 1, feature: run }`, `Installed (Stock Default)` =
  `{ state: 0 }`
- `elevation: admin`, `reversible: true`, `requires_reboot: true` (correct), `windows: { build: ">=26100" }`
  (correct)
- Corrected `risk_level: medium`. **Current wrong value:** `low`.

`Get-WindowsOptionalFeature -Online` on Windows 11 build 26100.4061 lists a feature literally named
`Recall`. The feature **entry** exists on ordinary Windows 11 24H2 x64, not only on Copilot+ PCs; it
was observed on IoT Enterprise LTSC 2024, a SKU where Recall itself is not offered. What is Copilot+
gated is the Recall experience. `/Disable-Feature` without `/Remove` turns the feature off and leaves
the payload on disk, which contradicts the current copy's claim that "the Recall component is gone
from the machine, not merely paused". Not present on Windows 10 LTSC 2021.

Where a documented, reversible alternative is wanted, `DisableAIDataAnalysis` under the WindowsAI CSP
stops snapshots without touching the component, and Microsoft states that on managed devices
snapshots are not enabled by default and require individual user opt-in consent regardless.

**Corrections needed:** Corrections 11 through 15. Accept both `Disabled` and
`DisabledWithPayloadRemoved` in the probe. Accept exit code 3010 as success. Capture the pre-apply
feature state and branch the undo on it rather than unconditionally enabling. Add `/Remove` if payload
removal is really the intent, and then document that re-enabling needs a source. Correct the
applicability text: the feature is enumerated on ordinary 24H2 builds, not only on Copilot+ PCs.
Raise `risk_level` to `medium`.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off the Recall optional Windows feature at the component level, below the Settings toggle.**

      ## What it does
      Runs `DISM /Online /Disable-Feature /FeatureName:Recall`, which disables the `Recall` optional
      feature that Windows 11 24H2 enumerates on every x64 install. This sits below the Settings
      toggle, so it survives the paths that only flip the user-facing setting.

      ## Benefits
      - **Below the toggle**: disables the feature itself, not the setting on top of it
      - **Survives re-enable paths**: a Settings flip alone cannot bring Recall back
      - **Works alongside policy**: complements the Recall snapshot and component policies

      ## Drawbacks
      - **Payload stays**: without `/Remove` the feature files remain on disk, just inactive
      - **Nothing to see on most PCs**: the feature entry exists everywhere, but the Recall experience needs Copilot+ hardware, so on other machines applying changes nothing visible
      - **Revert is conditional**: on a machine that shipped with Recall already disabled, reverting correctly does nothing rather than installing it
      - **Reboot required**: the change does not complete until the machine restarts

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer, x64; the `Recall` feature entry is present even on SKUs where Recall itself is never offered
      - **Takes effect**: after reboot; DISM reports a pending restart on success
      - **Reverting**: restores the feature state captured in the snapshot, so a machine that never had Recall enabled does not gain it
      - If the payload was removed rather than just disabled, re-enabling later needs a DISM `/Source`

      ## Recommendation
      Use it on a Copilot+ PC where Recall is present and you want it off at the component level. On
      any other machine prefer the Recall snapshot policy, which is documented, reversible, and does
      not depend on optional-feature state.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Disable-WindowsOptionalFeature](https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature)
      - [DISM operating system package servicing command-line options](https://learn.microsoft.com/en-us/windows/deployment/usmt/dism-operating-system-package-servicing-command-line-options)
      - [Policy CSP - WindowsAI](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
```

**Sources:**

1. `Get-WindowsOptionalFeature -Online` on Windows 11 build 26100.4061 IoT Enterprise LTSC returns
   FeatureName `Recall`, State `DisabledWithPayloadRemoved` (tier A, primary observation)
2. `Disable-WindowsOptionalFeature` cmdlet reference (feature states, `-Remove` semantics),
   https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature (tier A)
3. DISM operating system package servicing command-line options (`/Disable-Feature`,
   `/Enable-Feature`, `/Remove`, `/Source`, `/NoRestart`),
   https://learn.microsoft.com/en-us/windows/deployment/usmt/dism-operating-system-package-servicing-command-line-options
   (tier A)
4. Policy CSP - WindowsAI, `DisableAIDataAnalysis` as the documented reversible alternative,
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
5. DISM exit code 3010 (`ERROR_SUCCESS_REBOOT_REQUIRED`) with `/NoRestart`, discussed in
   microsoft/winget-cli issue 2707, https://github.com/microsoft/winget-cli/issues/2707 (tier C)
6. "Completely uninstall Recall feature on Windows 11", Pureinfotech,
   https://pureinfotech.com/uninstall-recall-windows-11/ (tier D, used only for the DISM command
   shape, which was independently confirmed by feature enumeration)

---

### `disable_copilot_taskbar` Copilot taskbar button

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**

- Key: `HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot`
- Value name: `TurnOffWindowsCopilot`
- Type: `REG_DWORD`
- Options (2, so a toggle):
  - `Hidden` writes `1`
  - `Shown (Stock Default)` writes `absent`
- `elevation: user`, `reversible: true`, no reboot flag

The registry effect is correct exactly as authored, and the `absent` stock default is right. The
shipped `WindowsCopilot.admx` on Windows 11 24H2 build 26100.4061 declares it as
`<policy name="TurnOffWindowsCopilot" class="User" key="SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot" valueName="TurnOffWindowsCopilot">`
with `<enabledValue><decimal value="1"/></enabledValue>` and
`<disabledValue><decimal value="0"/></disabledValue>`. Key, value name, DWORD type, polarity and the
`absent` default are therefore confirmed by a tier A source that ships on the target build. The shell
still reads it: `TurnOffWindowsCopilot` appears as a UTF-16 string in `Taskbar.dll`,
`CustomShellHost.exe`, `ShellAppRuntime.exe`, `TransmogProvider.dll` and
`assignedaccessmanagersvc.dll` on 26100.4061.

The corrections are all in the copy. **Current wrong claims:** that the policy "blocks the Win+C
hotkey", and silence on both the Home no-op and the deprecation. As of the May 2025 optional preview,
Win+C and the Copilot hardware key open the Microsoft 365 Copilot prompt box for commercial users, a
different surface this policy does not govern. Microsoft Learn states that "AppLocker policy should be
used instead of the Turn Off Windows Copilot legacy policy setting and its MDM equivalent,
TurnOffWindowsCopilot. The policy is subject to near-term deprecation", and the Policy CSP page carries
"This policy is deprecated and may be removed in a future release". Deprecated is not removed: the
ADMX still ships and the shell still reads the value on 26100, so the tweak works today.

What genuinely changed is scope. Since the September and October 2024 updates the Copilot in Windows
sidebar was replaced, and on 24H2 and later Copilot is an ordinary Store-packaged app, so the policy
hides the shell entry point but no longer removes the app. Microsoft's supported replacements are an
AppLocker publisher rule for `MICROSOFT.COPILOT`, or `Remove-AppxPackage` on `Microsoft.Copilot`.
`remove_copilot_app` is now the primary control and this tweak the cosmetic supplement.

Applicability per the Policy CSP: Windows 10 21H2 (19044.3758) and later, Windows 10 22H2 with
KB5032278, Windows 11 22H2 with KB5030310, and Windows 11 23H2 and later including 24H2 and 25H2.
**This is the one tweak in the category that genuinely applies to Windows 10 IoT Enterprise LTSC
2021**, given a build of 19044.3758 or later. Editions are Pro, Enterprise, Education and IoT
Enterprise / IoT Enterprise LTSC; the shipped ADMX `supportedOn` is
`SUPPORTED_Windows_11_0_NOSERVER_ENTERPRISE_EDUCATION_PRO_SANDBOX`, so Home is excluded by both and
the write is expected to be a no-op there. The policy is user-scope only: the Policy CSP marks Device
scope not applicable and the ADMX declares `class="User"`, so there is no HKLM equivalent. Sign-out
or reboot is needed to apply reliably, which the YAML's absent `requires_reboot` flag does not
express; state it in the copy rather than forcing a reboot for a per-user cosmetic change.

**Corrections needed:** Corrections 16, 17 and 18. Drop the Win+C claim from the `description` and
the `info`. Add the Home no-op, either as an edition gate or plainly in the copy. Name the
deprecation and the supported replacements. Keeping the tweak is right: it is the only per-user,
non-managed way to hide the button and it still works on 26100.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes the Copilot button from your taskbar without touching anything else.**

      ## What it does
      Sets the `TurnOffWindowsCopilot` policy under
      `HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot` to 1, the user-scope group policy
      "Turn off Windows Copilot". Microsoft's documented effect is that the Copilot icon does not
      appear on the taskbar. On 24H2 and newer Copilot is an ordinary Store app, so this hides the
      entry point and leaves the app installed.

      ## Benefits
      - **Cleaner taskbar**: the Copilot button is gone
      - **No admin needed**: a per-user policy value, applied without elevation
      - **Fully reversible**: removing the value restores the button exactly

      ## Drawbacks
      - **App still there**: Copilot can still be opened from Search, Start, or Edge
      - **Deprecated by Microsoft**: flagged for near-term deprecation, so plan for it to stop working
      - **Nothing on Home**: the shipped policy template excludes Home, where the write is inert
      - **Not the Copilot key**: Win+C and the Copilot hardware key now open the Microsoft 365 Copilot prompt box, which this policy does not govern

      ## Good to know
      - **Applies to**: Windows 11 22H2 and newer, and Windows 10 21H2 build 19044.3758 and newer; Pro / Enterprise / Education / IoT Enterprise editions
      - **Takes effect**: after signing out and back in
      - **Reverting**: restores the previous value from the snapshot
      - Microsoft's supported replacements are an AppLocker publisher rule for `MICROSOFT.COPILOT` or removing the `Microsoft.Copilot` app

      ## Recommendation
      Worth applying if you just want the button out of the way; it costs nothing and is instantly
      undone. If you want Copilot actually gone on 24H2 or newer, use the Copilot app removal tweak
      instead, since this only hides the shell entry point.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - WindowsAI, TurnOffWindowsCopilot](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
      - [Manage Windows Copilot](https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot)
```

**Sources:**

1. Shipped ADMX, Windows 11 24H2 build 26100.4061: `C:\Windows\PolicyDefinitions\WindowsCopilot.admx`,
   policy `TurnOffWindowsCopilot`, `class="User"`,
   `key="SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot"`, `valueName="TurnOffWindowsCopilot"`,
   enabled 1 / disabled 0, `supportedOn`
   `SUPPORTED_Windows_11_0_NOSERVER_ENTERPRISE_EDUCATION_PRO_SANDBOX` (tier A)
2. Policy CSP - WindowsAI, `TurnOffWindowsCopilot` (deprecation note, User-only scope, edition list,
   applicable OS list including Windows 10 21H2 19044.3758, and "The Copilot icon won't appear on the
   taskbar either"),
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
3. Manage Windows Copilot / Updated Windows and Microsoft 365 Copilot Chat experience (states
   AppLocker should be used instead and that the policy is subject to near-term deprecation;
   documents the 24H2 Store-app model, the AppLocker publisher rule for `MICROSOFT.COPILOT`, the
   `Remove-AppxPackage` route, and the May 2025 Win+C prompt-box behaviour),
   https://learn.microsoft.com/en-us/windows/client-management/manage-windows-copilot (tier A)
4. Direct binary inspection, Windows 11 24H2 build 26100.4061: `TurnOffWindowsCopilot` present as a
   UTF-16 string in `Taskbar.dll`, `CustomShellHost.exe`, `ShellAppRuntime.exe`,
   `TransmogProvider.dll` and `assignedaccessmanagersvc.dll`, confirming the shell still reads the
   policy (tier A, primary measurement)

---

### `disable_notepad_ai` Notepad AI features

**Verdict:** VERIFIED

**Mechanism:**

- Key: `HKLM\SOFTWARE\Policies\WindowsNotepad` (no `Microsoft` path component; this is correct, see
  below)
- Value name: `DisableAIFeatures`
- Type: `REG_DWORD`
- Options (2, so a toggle):
  - `Disabled` writes `1`
  - `Enabled (Stock Default)` writes `absent`
- `elevation: admin`, `reversible: true`, `requires_reboot: false`
- Gate: `windows: { products: [11] }`, plus a stated Notepad app version floor of 11.2503.16.0

**Corrections needed:** `none`. The unusual key path is correct exactly as proposed.

The key path looks wrong and is right. The literal bytes inside the shipped Notepad binary settle it.
From `C:\Program Files\WindowsApps\Microsoft.WindowsNotepad_11.2604.5.0_x64__8wekyb3d8bbwe\Notepad\Notepad.exe`
the strings `SOFTWARE\Policies\WindowsNotepad` and `DisableAIFeatures` are adjacent literals in the
same string block, followed by the `0` and `1` literals. Microsoft Learn agrees: "To disable AI
features in Notepad, set the DisableAIFeatures registry value to 1 under
`HKLM:\SOFTWARE\Policies\WindowsNotepad`."

Three findings worth recording so they are not re-litigated:

- **One value covers all AI surfaces.** There is no separate value per feature. `Notepad.exe` contains
  no `DisableRewrite`, `DisableSummarize` or `DisableCopilot` strings, and Learn's policy section
  lists exactly one policy.
- **Name mismatch trap.** The Group Policy / ADMX policy is named `DisableAIFeaturesInNotepad`; the
  **registry value** is `DisableAIFeatures`. Writing the ADMX name would be silently ignored. The
  YAML must use `DisableAIFeatures`.
- **Not in the inbox ADMX set.** `WindowsNotepad.admx` ships out of band in
  `WindowsNotepadAdminTemplates.cab`, so no ADMX on 26100 contains `DisableAIFeatures`. That is not
  evidence against the tweak; registry-only application works whether or not the ADMX is installed.

The real applicability gate is wider than a build number. Windows 11 22H2 and later, **and Notepad
app version 11.2503.16.0 or later**. Notepad updates through the Store independently of the OS build,
so a fully patched 26100 or 26200 machine can still carry a pre-11.2503 Notepad on which the value is
inert. Inert on Windows 10 LTSC 2021, which ships the classic Win32 Notepad with no AI surface.

**Ready-to-paste info block:**

```yaml
    info: |
      **Turns off Copilot, Rewrite and Summarize inside Notepad, leaving a plain text editor.**

      ## What it does
      Sets `DisableAIFeatures` to 1 under `HKLM\SOFTWARE\Policies\WindowsNotepad`, the registry value
      Microsoft documents for this. One value covers every AI surface in the app; there is no
      per-feature switch.

      ## Benefits
      - **Plain editor back**: Copilot, Rewrite and Summarize disappear from the menus and toolbar
      - **Machine-wide**: applies to every account on the PC
      - **No sign-in prompts**: removes the entry points that ask you to sign in to use AI

      ## Drawbacks
      - **Loses the features**: Rewrite, Summarize and Copilot in Notepad are gone, all or nothing
      - **Needs a current Notepad**: app versions before 11.2503.16.0 ignore the value entirely
      - **Notepad only**: other apps' AI features are unaffected

      ## Good to know
      - **Applies to**: Windows 11 22H2 and newer, with Notepad app version 11.2503.16.0 or later; Notepad updates through the Store independently of Windows, so a fully patched PC can still have an older Notepad
      - **Takes effect**: after restarting Notepad
      - **Reverting**: restores the previous value from the snapshot
      - The Group Policy setting is named `DisableAIFeaturesInNotepad`, but the registry value is `DisableAIFeatures`; only the latter is read

      ## Recommendation
      Apply it if you want Notepad to stay the fast, plain text editor it has always been. Skip it if
      you actually use Rewrite or Summarize, since the switch is all or nothing.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Manage Notepad](https://learn.microsoft.com/en-us/windows/client-management/manage-notepad)
      - [Policy CSP - WindowsAI](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
```

**Sources:**

1. Microsoft Learn, Manage Notepad ("set the DisableAIFeatures registry value to 1 under
   `HKLM:\SOFTWARE\Policies\WindowsNotepad`", the app version floor, and the single-policy list),
   https://learn.microsoft.com/en-us/windows/client-management/manage-notepad (tier A)
2. Binary string evidence in `Notepad.exe` 11.2604.5.0 from
   `C:\Program Files\WindowsApps\Microsoft.WindowsNotepad_11.2604.5.0_x64__8wekyb3d8bbwe\Notepad\`:
   `SOFTWARE\Policies\WindowsNotepad` and `DisableAIFeatures` adjacent in one string block, followed
   by `0` and `1`; no `DisableRewrite`, `DisableSummarize` or `DisableCopilot` strings present
   (tier A, product artifact)
3. Negative ADMX search across `C:\Windows\PolicyDefinitions` on 26100: no shipped ADMX contains
   `DisableAIFeatures`, consistent with `WindowsNotepad.admx` shipping in
   `WindowsNotepadAdminTemplates.cab` (tier A, primary observation)

---

### `disable_paint_ai` Paint AI features

**Verdict:** VERIFIED

**Mechanism:**

- Key: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint` (machine scope; the Policy CSP
  marks User scope explicitly not supported)
- Three values, each `REG_DWORD`:
  - `DisableCocreator`
  - `DisableGenerativeFill`
  - `DisableImageCreator`
- Options (2, so a toggle):
  - `Disabled` writes `1` to all three
  - `Enabled (Stock Default)` writes `absent` for all three (documented Default Value 0 for each)
- `elevation: admin`, `reversible: true`, `requires_reboot: false`
- Gate: Windows 11 22H2 and newer

**Corrections needed:** `none`. All three values are exact, and the rejection of two extra values is
upheld (see below).

All three appear in the shipped `WindowsCopilot.admx` on 26100, each `class="Machine"`, each on
`Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, each with `enabledValue` 1 and
`disabledValue` 0, each `supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"`. Policy CSP -
WindowsAI independently gives, for each of the three, `Registry Key Name`
`Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, the matching `Registry Value Name`,
`ADMX File Name` `WindowsCopilot.admx`, Format `int`, Default Value `0`, and allowed values
`0 (Default)` enabled and `1` disabled. Two independent tier A sources on both the mechanism and the
polarity.

Polarity is **not** inverted despite the `Disable*` naming: both the ADMX `enabledValue` and the CSP
allowed-values table give 1 = disabled. The ADML confirms: "If this policy is enabled, Cocreator
functionality will not be accessible in the Paint app."

Applicable OS on the CSP pages: "Windows 11, version 22H2 [10.0.22621.4870] and later" and
"Windows 11, version 24H2 [10.0.26100.3360] and later". Editions Pro, Enterprise, Education, IoT
Enterprise and IoT Enterprise LTSC, all supported. Inert on Windows 10 LTSC 2021, which is outside
the `supportedOn` range.

**Two extra values rejected.** `DisableGenerativeErase` and `DisableRemoveBackground` circulate in
Win11Debloat. They appear in **no** ADMX on 26100 and on **no** Microsoft Learn page; the complete
WindowsAI CSP policy list does not contain them. Single-source. Do not ship them.

**Verification limit worth recording:** the three value names were not found in any binary under
`C:\Program Files\WindowsApps` on the machine inspected, because the Paint app was not installed
there. The ADMX plus CSP pair is tier A on both sides, so this does not weaken the verdict.

**Ready-to-paste info block:**

```yaml
    info: |
      **Removes Cocreator, Generative fill and Image Creator from Paint, leaving the classic drawing tools.**

      ## What it does
      Sets `DisableCocreator`, `DisableGenerativeFill` and `DisableImageCreator` to 1 under
      `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Paint`, the three policies Microsoft
      ships in `WindowsCopilot.admx`. Paint's AI tools stop being accessible.

      ## Benefits
      - **Classic Paint back**: the AI buttons leave the toolbar
      - **Machine-wide**: covers every account, and Microsoft documents this as a device-scope policy
      - **No credit prompts**: removes the surfaces that ask you to sign in or spend AI credits

      ## Drawbacks
      - **Loses three tools**: Cocreator, Generative fill and Image Creator all go together
      - **Paint only**: image generation elsewhere in Windows is unaffected
      - **Nothing on Windows 10**: the policies are declared for Windows 11 22H2 and newer only

      ## Good to know
      - **Applies to**: Windows 11 22H2 and newer; Pro / Enterprise / Education / IoT Enterprise editions
      - **Takes effect**: after restarting Paint
      - **Reverting**: restores the previous values from the snapshot
      - Two other values that circulate online, `DisableGenerativeErase` and `DisableRemoveBackground`, are not real policies and are deliberately not written here

      ## Recommendation
      Apply it if you use Paint as a quick drawing and cropping tool and have no interest in its
      image generation. Skip it if you use Cocreator or Generative fill, since all three tools are
      controlled together.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Policy CSP - WindowsAI](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai)
      - [Manage Paint](https://learn.microsoft.com/en-us/windows/client-management/manage-paint)
```

**Sources:**

1. Shipped `C:\Windows\PolicyDefinitions\WindowsCopilot.admx` and
   `en-US\WindowsCopilot.adml` on Windows 11 24H2 build 26100: `DisableCocreator`,
   `DisableGenerativeFill` and `DisableImageCreator`, each `class="Machine"`, each on
   `Software\Microsoft\Windows\CurrentVersion\Policies\Paint`, each `enabledValue` 1 /
   `disabledValue` 0, each `supportedOn ref="windows:SUPPORTED_Windows_11_0_22H2"` (tier A)
2. Policy CSP - WindowsAI (per-value `Registry Key Name`, `Registry Value Name`, `ADMX File Name`,
   Format `int`, Default Value 0, allowed values `0 (Default)` and `1`, Device scope only, applicable
   OS Windows 11 22H2 [10.0.22621.4870] and later and Windows 11 24H2 [10.0.26100.3360] and later),
   https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-windowsai (tier A)
3. Negative search of all ADMX on 26100 and of the complete WindowsAI CSP policy list for
   `DisableGenerativeErase` and `DisableRemoveBackground`: absent from both, so those two values are
   single-source and are not shipped (tier A, primary observation)

---

### `disable_edge_ai_features` Edge AI features

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:**

- Key: `HKLM\SOFTWARE\Policies\Microsoft\Edge` (each Edge policy page lists
  `Path (Mandatory): SOFTWARE\Policies\Microsoft\Edge`)
- Five values, each `REG_DWORD`, each `Off` = `0`, each `Stock Default` = `absent`:

| Value | Type | `0` means | Real gate |
|---|---|---|---|
| `EdgeHistoryAISearchEnabled` | REG_DWORD | AI history search off, exact-match only | Edge 138+ |
| `NewTabPageBingChatEnabled` | REG_DWORD | all Bing Chat entry points removed from the new tab page | Edge 117+ |
| `CopilotPageContext` | REG_DWORD | Copilot cannot read page content | Edge 124+, **Entra profiles only** |
| `EdgeEntraCopilotPageContext` | REG_DWORD | Copilot cannot read page content or history | Edge 130+, **Entra profiles only** |
| `ComposeInlineEnabled` | REG_DWORD | Rewrite unavailable | Edge 115+, **Entra profiles only** |

- Options (2, so a toggle): `Off` writes `0` to all five, `Default (Stock Default)` writes `absent`
  for all five.
- `elevation: admin`, `reversible: true`, `requires_reboot: false`

All five values exist, all five are `REG_DWORD`, the base key is correct for all five, and none
appears in Edge's deprecated or obsolete policy tables.

**Corrections needed:** Corrections 19, 20 and 21.

**The Entra gate.** `CopilotPageContext`, `EdgeEntraCopilotPageContext` and `ComposeInlineEnabled`
are documented by Microsoft as applying only to Microsoft Entra ID profiles. Microsoft states it in
the policy text itself, for example on `CopilotPageContext`: "This policy applies only to Microsoft
Entra ID profiles in Microsoft Edge. It doesn't apply to Microsoft account (MSA) profiles"; on
`ComposeInlineEnabled`: "This policy applies only to Microsoft Entra accounts and doesn't apply to
Microsoft accounts"; and `EdgeEntraCopilotPageContext` lists "Applies to a profile that is signed in
with a Microsoft account: No".

This matters more than a footnote. `CLAUDE.md` sets the did-it-work contract: a failed privileged or
effect operation must surface as `Err`, never a benign-looking value. These three do not fail. They
succeed and do nothing. The user applies a tweak called "turn off Edge Copilot page context", the app
reports Applied, and Copilot in Edge carries on reading pages, because the profile is an MSA. Two
acceptable shapes:

1. **Ship all five and name the Entra gate in the option label and in Drawbacks.** This is the shape
   the info block below is written for, because under `_SPEC.md`'s inclusion rule a real control does
   exist for the three, and dropping them removes control from Entra-joined users. It matches how the
   corpus already handles Copilot+ gated tweaks: honest copy rather than removal.
2. **Ship only the two that work on consumer profiles** (`EdgeHistoryAISearchEnabled`,
   `NewTabPageBingChatEnabled`). This was the verification pass's first preference, on the grounds
   that it is the cleanest and preserves the tweak's honesty without relying on the user reading
   Drawbacks. If the maintainer prefers a stricter reading of the did-it-work contract, take this
   shape and delete the last three rows from the table above.

Do **not** use `skip_validation: true` to paper over the difference. That hides the problem rather
than solving it.

**Two values that must not ship.** `EdgeCopilotEnabled` was proposed as a third option value on the
grounds that it "removes Copilot in Edge entirely". Microsoft's page for it lists Windows: Not
supported, macOS: Not supported, Android and iOS 123+ only, and the page carries no Windows registry
settings section at all. Writing it to `HKLM\SOFTWARE\Policies\Microsoft\Edge` on Windows does
nothing. `CopilotCDPPageContext` is marked "(obsolete)" on the Edge policy index and superseded by
`EdgeEntraCopilotPageContext`. Ship neither.

**No collision with the existing Edge tweaks.** The corpus's five existing Edge tweaks write
`HideFirstRunExperience`, `StartupBoostEnabled`, `BackgroundModeEnabled`, `HubsSidebarEnabled`,
`EdgeCollectionsEnabled`, `MetricsReportingEnabled`, `SendSiteInfoToImproveServices`,
`PersonalizationReportingEnabled`, `Edge3PSerpTelemetryEnabled` and `SmartScreenEnabled`. None of the
five proposed values collides, and `HubsSidebarEnabled` removes only the sidebar panel, not the AI
features themselves.

Applicability is wherever Edge is installed, machine scope, subject to the per-value Edge version
floors in the table. Because Edge updates independently of the OS, a `windows:` build gate cannot
express this; the floors belong in the copy. That independence is also why this is the one new tweak
in the category that is genuinely usable on Windows 10 IoT Enterprise LTSC 2021, where a current Edge
is supported. Setting any Edge policy makes Edge display "managed by your organization" on its
settings page; the corpus's existing Edge tweaks already do this, so it is not new, but it belongs in
the copy.

**Ready-to-paste info block:**

```yaml
    info: |
      **Strips Copilot and Bing Chat entry points out of Microsoft Edge and stops AI reading your pages and history.**

      ## What it does
      Writes five Edge policies under `HKLM\SOFTWARE\Policies\Microsoft\Edge`:
      `EdgeHistoryAISearchEnabled` and `NewTabPageBingChatEnabled` turn off AI history search and the
      new tab page chat entry points, and `CopilotPageContext`, `EdgeEntraCopilotPageContext` and
      `ComposeInlineEnabled` stop Copilot reading page content or history and remove inline Rewrite.

      ## Benefits
      - **No chat on new tab**: the Bing Chat entry points leave the new tab page
      - **History stays local**: browsing history search returns to plain exact matching
      - **No page reading**: on work accounts, Copilot loses access to page content and history

      ## Drawbacks
      - **Three are work-account only**: `CopilotPageContext`, `EdgeEntraCopilotPageContext` and `ComposeInlineEnabled` apply only to Microsoft Entra ID profiles and do nothing on a personal Microsoft account, so on a consumer PC only the first two change anything
      - **Managed banner**: Edge will show "managed by your organization" on its settings page
      - **Version floors**: older Edge builds ignore some values, `EdgeHistoryAISearchEnabled` needs Edge 138 or newer
      - **Loses the features**: AI history search and inline Rewrite go with them

      ## Good to know
      - **Applies to**: any Windows version with Microsoft Edge installed, machine-wide; per-value Edge minimums are 115 for `ComposeInlineEnabled`, 117 for `NewTabPageBingChatEnabled`, 124 for `CopilotPageContext`, 130 for `EdgeEntraCopilotPageContext` and 138 for `EdgeHistoryAISearchEnabled`
      - **Takes effect**: after restarting Edge
      - **Reverting**: restores the previous values from the snapshot, and the managed banner clears once no Edge policies remain
      - Copilot's own button in Edge is not removed by these; Microsoft's `EdgeCopilotEnabled` policy is not supported on Windows

      ## Recommendation
      Apply it if you want Edge to stop offering AI in the browsing surface, and expect a visible
      change on a personal account only from the history search and new tab page values. On a
      work-joined PC all five take effect and this is the strongest in-browser AI control available.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Microsoft Edge Browser Policy Documentation](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies)
      - [EdgeHistoryAISearchEnabled](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgehistoryaisearchenabled)
      - [CopilotPageContext](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#copilotpagecontext)
```

**Sources:**

1. Microsoft Edge Browser Policy Documentation, `EdgeHistoryAISearchEnabled` (Edge 138+, `0` gives
   exact-match history search),
   https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgehistoryaisearchenabled
   (tier A)
2. Microsoft Edge Browser Policy Documentation, `NewTabPageBingChatEnabled` (Edge 117+, `0` removes
   all Bing Chat entry points from the new tab page),
   https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#newtabpagebingchatenabled
   (tier A)
3. Microsoft Edge Browser Policy Documentation, `CopilotPageContext` (Edge 124+, "This policy applies
   only to Microsoft Entra ID profiles in Microsoft Edge. It doesn't apply to Microsoft account (MSA)
   profiles"),
   https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#copilotpagecontext (tier A)
4. Microsoft Edge Browser Policy Documentation, `EdgeEntraCopilotPageContext` (Edge 130+, "Applies to
   a profile that is signed in with a Microsoft account: No"),
   https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgeentracopilotpagecontext
   (tier A)
5. Microsoft Edge Browser Policy Documentation, `ComposeInlineEnabled` (Edge 115+, "This policy
   applies only to Microsoft Entra accounts and doesn't apply to Microsoft accounts"),
   https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#composeinlineenabled (tier A)
6. Microsoft Edge Browser Policy Documentation, `EdgeCopilotEnabled` (Windows: Not supported, macOS:
   Not supported, Android and iOS 123+ only, no Windows registry settings section), used as the basis
   for excluding it,
   https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#edgecopilotenabled (tier A)
7. Microsoft Edge policy index, `CopilotCDPPageContext` marked "(obsolete)" and superseded by
   `EdgeEntraCopilotPageContext`, https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies
   (tier A)

---

### `disable_ai_fabric_service` AI Fabric service

**Verdict:** VERIFIED-WITH-CORRECTION

**Mechanism:** a service start-type change.

- Service short name: `WSAIFabricSvc`
- **Corrected display name: `WSAIFabricSvc`** (literally that string). **Current wrong value:**
  "Windows AI Fabric Service". Resolving the MUI resource
  `C:\Windows\System32\en-US\WSAIFabricHost.dll.mui` gives `DisplayName: WSAIFabricSvc` and
  `Description: Provides support to communicate with AIFabric in Local service context over COM.`
  If the corpus matches or displays on the friendly name, it will not match.
- Start types:
  - `Manual (applied)` = `3`
  - `Automatic (Stock Default)` = `2`
- Not trigger-started (`sc qtriggerinfo` reports no registered triggers) and not delayed-auto (no
  `DelayedAutostart` value in the service key), so the revert option is a plain Automatic.
- Registration shape on 26100.4061:
  ```
  ImagePath       : %systemroot%\system32\svchost.exe -k WSAIFabricSvcGroup -p
  ObjectName      : NT AUTHORITY\LocalService
  Type            : 0x20 (shared svchost process)
  DependOnService : RPCSS, BrokerInfrastructure
  Description     : @%systemroot%\system32\WSAIFabricHost.dll,-103
  DisplayName     : @%systemroot%\system32\WSAIFabricHost.dll,-102
  ```
- **Corrected gate: `windows: { build: ">=26100" }` plus a presence check that skips cleanly when the
  service is not registered.** **Current wrong gate:** `build >= 26200`, on the claim that the
  service does not exist on 26100 or LTSC 2021. It does exist on 26100. The service is registered on
  a 26100.4061 machine and its host binary is a Microsoft-shipped 26100 file:
  ```
  C:\Windows\System32\WSAIFabricHost.dll
    FileVersion    : 10.0.26100.3624 (WinBuild.160101.0800)
    ProductVersion : 10.0.26100.3624
    FileDescription: WSAIFabricSvc
    (timestamp May 2025)
  ```
  The registration and the shipped DLL are product artifacts, not owner modifications; nobody
  hand-authors a `WSAIFabricHost.dll` with a Microsoft version resource. The file version pins the
  arrival: the service came to 24H2 in servicing around **26100.3624**, not only in 25H2. 26100
  servicing levels below roughly 26100.3624 will not have it, which is why the presence check is
  required and why "service not registered" must be a skip, not an error.
- `elevation: admin`, `reversible: true`, `requires_reboot: true`
- Not present on Windows 10 LTSC 2021.

**The default start type is Automatic.** Three lines of evidence, none of which is any single
machine's own configured state. First, Win11Debloat's own undo file writes it back to Automatic:
`Regfiles/Disable_AI_Service_Auto_Start.reg` sets `"Start"=dword:00000003` and
`Regfiles/Undo/Enable_AI_Service_Auto_Start.reg` sets `"Start"=dword:00000002`. A project's
restore-to-default artifact is a statement about the default. Second, the Microsoft Q&A 25H2 thread's
remedy is `Set-Service WSAIFabricSvc -StartupType Manual` with the note that this "prevents the
service from starting automatically after reboot", which only makes sense if the shipped state is
Automatic. Third, ElevenForum users on 25H2 report the service re-enabling itself after reboot when
disabled.

**The memory claim must not ship as written.** The proposal justified the tweak with a
`WorkloadsSessionHost.exe` story ("eight processes, multi-gigabyte resident"). It does not
generalise. A recursive search of all of `C:\Windows` for `WorkloadsSessionHost*` on a 26100.4061
host returns nothing, and zero such processes run even though `WSAIFabricSvc` itself is running.
`WSAIFabricHost.dll` does not contain the strings `WorkloadsSessionHost`, `WorkloadSession` or
`SessionHost` in either encoding; it does contain `AIFabric` and `Workloads`. Every report of the
memory symptom is from an NPU-equipped or Copilot+ machine: a Microsoft Q&A thread on 26200.8894
measured 8 instances at 2,449 MB private and 4,454 MB working set, dropping to zero after
`Stop-Service WSAIFabricSvc -Force`, and ElevenForum threads report 5 to 8 GB alongside an AMD NPU
interaction tracked in `microsoft/onnxruntime-genai` issue 2013. The service is universal; the memory
symptom is NPU and Copilot+ specific. On a non-NPU 24H2 machine this is a "stop a COM broker for AI
workloads from auto-starting" tweak, which is a legitimate control under the corpus's inclusion
principle but is not the resource reclaim the proposal advertised.

Manual rather than Disabled remains the right applied value: the service is a demand-startable COM
broker, so Manual stops the boot-time start while leaving anything that genuinely needs it able to
start it.

`WSAIFabricSvc` appears nowhere in `services.yaml` or any other corpus file, so there is no duplicate.

**Corrections needed:** Corrections 22 through 25. Fix the display name to the literal
`WSAIFabricSvc`. Change the gate from `>=26200` to `>=26100` plus a clean skip when the service is
absent. Remove the multi-gigabyte memory promise from the copy and scope it to NPU and Copilot+
hardware. Drop `batcmd.com` and `revertservice.com` from the sources; they publish an identical wrong
start type, are one derivative source, and are contradicted by every other line of evidence.

**Ready-to-paste info block:**

```yaml
    info: |
      **Stops the Windows AI broker service from starting with the machine, leaving it available on demand.**

      ## What it does
      Sets the `WSAIFabricSvc` service from Automatic to Manual. Microsoft describes it as providing
      "support to communicate with AIFabric in Local service context over COM"; it is the broker AI
      workloads talk to. On Manual it no longer starts at boot, and anything that genuinely needs it
      can still start it.

      ## Benefits
      - **One less at boot**: a service that runs from startup no longer does
      - **Large win on NPU PCs**: on Copilot+ and NPU machines the associated AI workload hosts have been measured holding several gigabytes, which this prevents
      - **Reversible cleanly**: Manual keeps the service startable, unlike Disabled

      ## Drawbacks
      - **No visible gain on ordinary PCs**: without an NPU there is no measurable memory or CPU saving, this simply stops an idle broker auto-starting
      - **AI features may start it**: using a Windows AI feature can start the service on demand anyway
      - **Not on every build**: 24H2 machines below roughly build 26100.3624 do not have this service at all, and the tweak skips them

      ## Good to know
      - **Applies to**: Windows 11 24H2 build 26100.3624 and newer, including 25H2; the service does not exist on Windows 10
      - **Takes effect**: after reboot, since the change is to the start type rather than the running service
      - **Reverting**: restores the previous start type from the snapshot, which is Automatic on a stock machine
      - Set to Manual rather than Disabled deliberately, so nothing that needs the broker is blocked outright

      ## Recommendation
      Worth applying on a Copilot+ or NPU-equipped PC where the AI workload hosts are consuming real
      memory. On an ordinary machine with no NPU it is a reasonable way to keep an unused AI broker
      out of your startup, but do not expect to see a difference in Task Manager.

      ## Evidence
      - **Risk**: low
      - **Confidence**: Microsoft-documented
      - [Set-Service](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/set-service)
      - [Service start types](https://learn.microsoft.com/en-us/windows/win32/services/service-installation-and-configuration)
```

**Sources:**

1. Shipped `C:\Windows\System32\WSAIFabricHost.dll` version resource on 26100.4061 (FileVersion
   10.0.26100.3624, ProductVersion 10.0.26100.3624, FileDescription `WSAIFabricSvc`, timestamp
   May 2025) (tier A, product artifact)
2. Shipped `C:\Windows\System32\en-US\WSAIFabricHost.dll.mui` string table, resolving
   `DisplayName: WSAIFabricSvc` and `Description: Provides support to communicate with AIFabric in
   Local service context over COM.` (tier A, product artifact)
3. Service registration under `HKLM\SYSTEM\CurrentControlSet\Services\WSAIFabricSvc` on 26100.4061,
   used as evidence of existence and registration shape (ImagePath, ObjectName, Type,
   DependOnService), not of the owner's configured start type (tier A, primary observation)
4. `sc qtriggerinfo WSAIFabricSvc` returning no registered triggers, and the absence of a
   `DelayedAutostart` value in the service key, establishing a plain Automatic rather than
   trigger-start or delayed-auto (tier A, primary observation)
5. Negative binary search for `WorkloadsSessionHost*` across all of `C:\Windows` on 26100.4061, and
   negative string search for `WorkloadsSessionHost`, `WorkloadSession` and `SessionHost` inside
   `WSAIFabricHost.dll` in both encodings (tier A, primary observation)
6. Win11Debloat, `Regfiles/Disable_AI_Service_Auto_Start.reg` (`"Start"=dword:00000003`) and
   `Regfiles/Undo/Enable_AI_Service_Auto_Start.reg` (`"Start"=dword:00000002`), read as raw `.reg`
   bodies, https://github.com/Raphire/Win11Debloat (tier C)
7. Win11Debloat issue 265 (2025-06-25, predates 25H2 general availability, reports the service on
   24H2, and the maintainer confirms the action is Manual rather than Disabled) and issue 497,
   https://github.com/Raphire/Win11Debloat/issues/265 (tier C)
8. Microsoft Q&A question 5955428 on build 26200.8894: 8 `WorkloadsSessionHost` instances at 2,449 MB
   private and 4,454 MB working set, dropping to zero after `Stop-Service WSAIFabricSvc -Force`;
   remedy given as `Set-Service WSAIFabricSvc -StartupType Manual` (tier C)
9. ElevenForum thread 45675 via the Wayback Machine: 5 to 8 GB reports on NPU hardware, and reports
   of the service re-enabling itself after reboot when set to Disabled (tier C)
10. `microsoft/onnxruntime-genai` issue 2013, AMD NPU interaction,
    https://github.com/microsoft/onnxruntime-genai/issues/2013 (tier C)
11. **Explicitly rejected as sources:** `batcmd.com` and `revertservice.com`, which state the 24H2
    start type is Manual. They publish an identical wrong value, neither tracks 25H2, and the shared
    figure is the signature of a common upstream, so they count as one derivative source and it is
    contradicted by every other line of evidence.
