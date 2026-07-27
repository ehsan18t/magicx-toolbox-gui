# Cross-category analysis

Produced 2026-07-26 against `src-tauri/tweaks/*.yaml` (203 tweaks across 7 category files).

This document covers what no single-category review can see: conflicts between tweaks in different
files, the same Windows feature addressed from several categories, and structural defects in the
corpus as a whole. Per-tweak source validation lives in the sibling category documents.

Method: mechanical extraction of every registry key, value name, service name, and task path from
all seven files, then grouping by target. Findings below are derived from the corpus itself, not
from external sources, so they are stated as facts about the YAML rather than claims about Windows.

## 1. Direct conflict between two shipped tweaks

**`performance:disable_vbs_hvci` and `security:enable_credential_guard` fight each other.**

Both are authored against `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard`:

| Tweak | Writes |
|---|---|
| `performance:disable_vbs_hvci` | `EnableVirtualizationBasedSecurity = 0`, `Scenarios\HypervisorEnforcedCodeIntegrity\Enabled = 0` |
| `security:enable_credential_guard` | `LsaCfgFlags = 1` |

Credential Guard is a VBS-hosted feature and cannot run without VBS. A user who applies the security
tweak and then the performance tweak ends up with `LsaCfgFlags = 1` requesting a feature the machine
can no longer host. Nothing in the corpus warns about this, and neither tweak's snapshot knows about
the other.

**Amended after category validation.** `security.md` finds that `enable_credential_guard` writes
`LsaCfgFlags` to the wrong key: Microsoft's registry route is
`HKLM\SYSTEM\CurrentControlSet\Control\Lsa\LsaCfgFlags`, and the policy route is
`HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard\LsaCfgFlags`. The key the tweak actually
writes, `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard`, holds the VBS values but is not where
Credential Guard is read from.

Two consequences follow. First, the tweak is currently a no-op, so the conflict does not bite today.
Second, and more important, **fixing that key activates the conflict.** The dependency must therefore
be handled as part of the same change that corrects the key, not deferred. Do not treat "no conflict
observed in testing" as evidence the problem is absent; it is masked by a separate defect.

This matters because the two tweaks target opposite audiences (gamer, hardened workstation) and both
present as low-friction picks inside their own category.

Recommended handling, in order of preference:

1. Declare a mutual-exclusion or dependency relationship in the schema so selecting one surfaces the
   other's state, rather than relying on the user to notice.
2. Failing that, add explicit cross-references in both `info` blocks and both `warning` fields.

`security:enable_credential_guard` should in any case state that it requires VBS to be enabled,
which is a prerequisite the current text does not make explicit as a hard requirement.

## 2. Structural defects

### 2.1 Two tweaks have no revert option

| Tweak | Options |
|---|---|
| `security:disable_remote_desktop` | 1 (`Disabled` only) |
| `security:disable_lmhash_storage` | 1 (`Enabled` only) |

Every other tweak in the corpus offers a second option representing the stock state. ADR-0003 treats
System Default as a selectable state that performs a revert, so a single-option tweak cannot express
that state and the user has no in-app path back. Both need a stock-default option added; the correct
default values are established in `security.md`.

### 2.2 State-marker key naming is inconsistent

Seventeen debloat tweaks write their state marker under `HKCU\Software\MagicXToolbox\Debloat`, while
nine tweaks across network, performance, and security use `HKCU\Software\MagicXToolbox\State`. This
is cosmetic and harmless at runtime, but it means an operator inspecting the machine has to know two
locations. Worth unifying on one.

### 2.3 Not a defect: HKCU writes under admin elevation

The 24H2 re-scope pass raised a concern that HKCU-writing tweaks declared `elevation: admin` might
land in the elevated account's hive rather than the signed-in user's. Thirty-one tweaks write HKCU
under an admin floor: five write real user settings (`performance:disable_gamedvr_capture`,
`privacy:disable_inking_typing_personalization`, `privacy:disable_advertising_id`,
`privacy:disable_tailored_experiences`, `privacy:disable_tips_and_suggestions`) and twenty-six write
only `HKCU\Software\MagicXToolbox` state markers that gate detection.

This is already handled by design. ADR-0005 states that a user-hive effect ignores the tweak's
elevation floor and always runs in-process as the real user, even inside a System or TrustedInstaller
tweak, "so it never lands in the elevated account's hive" (lines 17-18 and 27-28). The ADR also
covers the residual over-the-shoulder case, where a different admin's credentials elevated the app,
by disabling HKCU-touching tweaks with an explicit message.

No action needed. Recorded so a future review does not re-raise it.

### 2.4 Not a defect: asymmetric action options

Twenty-eight action-carrying tweaks list the action effect in only one option, for example
`{state: 1, app: run}` versus `{state: 0}`. This looks like an incomplete revert but is correct by
design: `src-tauri/src/tweaks/engine/apply.rs:391` and `:221` show that when a target option omits an
undo-carrying action whose probe currently reads present, the engine drives that action's `undo`.
Recorded here so a future review does not re-raise it.

## 3. Same feature addressed from multiple categories

These are not bugs. They are places where a user must find and apply two or three tweaks in
different parts of the app to fully achieve one intent, and where applying only one may leave them
believing they achieved more than they did.

### 3.1 Windows telemetry: policy and service are split

`privacy:disable_diagnostic_data` writes the `AllowTelemetry` policy. `services:disable_diagtrack`
disables the service that actually transmits. The privacy tweak's own text concedes that on Home and
Pro the policy is floored, which makes the services tweak the only effective lever on the editions
most users run. A user who applies the privacy tweak alone gets substantially less than they think.

**Merge recommendation (strong):** one telemetry tweak with a ladder of options, for example
`Windows default` / `Required only` / `Required only, transmitter disabled`. This turns two tweaks
whose relationship is invisible into one honest choice.

### 3.2 Location: platform and service are split

`privacy:disable_location_tracking` writes the consent store and `DisableLocation` policy.
`services:disable_geolocation` disables `lfsvc`. Both produce "no location," at different strengths.
`privacy:disable_find_my_device` is a distinct feature and should stay separate.

**Merge recommendation (moderate):** combine the first two into a strength ladder,
`On` / `Denied to apps` / `Location service off`.

### 3.3 Windows Recall: four tweaks, two files

| Tweak | Mechanism |
|---|---|
| `privacy:disable_recall_snapshots` | `DisableAIDataAnalysis = 1` |
| `privacy:remove_recall_component` | `AllowRecallEnablement = 0` |
| `debloat:remove_recall_feature` | DISM `/Disable-Feature /FeatureName:Recall` |
| `privacy:disable_click_to_do` | `DisableClickToDo = 1` |

The first three are three rungs of one ladder against the same feature, and the third lives in a
different category from the other two.

**Merge recommendation (strong):** fold the first three into one Recall tweak with options
`Available` / `Snapshots disabled` / `Feature removed`. Keep Click to Do separate; it is a distinct
feature that merely shares the WindowsAI policy key.

### 3.4 Copilot: hide the button or remove the app

`interface:disable_copilot_taskbar` sets `TurnOffWindowsCopilot`; `debloat:remove_copilot_app`
uninstalls the package. On 24H2 the first no longer removes Copilot, only its button, which the
interface tweak's text already admits while pointing at the debloat entry.

**Merge recommendation (moderate):** one Copilot tweak with `Shown` / `Button and hotkey hidden` /
`App removed`. This is exactly the case where the current split makes users apply one tweak and
wrongly believe Copilot is gone.

### 3.5 Chat / Teams taskbar button: two mechanisms, one outcome

`debloat:remove_teams_chat_taskbar` sets the HKLM `ChatIcon` policy to 3; `interface:disable_chat_taskbar`
clears the HKCU `TaskbarMn` value. Same visible result, machine scope versus user scope.

**Merge recommendation (strong):** one tweak. Two entries that hide the same button by different
means is a coin flip for the user with no way to know which to pick.
`debloat:remove_teams_consumer_app` is the app itself and stays separate.

### 3.6 OneDrive: hide it or uninstall it

`interface:remove_onedrive_nav_pane` hides the Explorer node; `debloat:remove_onedrive` uninstalls
the client. A clear ladder split across two files.

**Merge recommendation (moderate):** `Shown` / `Hidden from Explorer` / `Uninstalled`. If kept
separate, the nav-pane tweak must keep its current, correct warning that hiding is not removing.

### 3.7 Error reporting: policy and upload task

`privacy:disable_error_reporting` disables WER; `services:task_wer_queuereporting` disables the task
that uploads queued reports. The second is a strict subset of the first's intent.

**Merge recommendation (moderate):** one tweak covering both, or at minimum a cross-reference.

### 3.8 Feedback prompts: machine scope and user scope

`privacy:disable_feedback_notifications` (HKLM policy) and `privacy:disable_feedback_frequency`
(HKCU Siuf rules) are two halves of one intent, and the second tweak's own text says to apply it
alongside the first.

**Merge recommendation (strong, same file):** merge. When the corpus tells users to always apply two
tweaks together, they are one tweak.

### 3.9 Widgets and News: same feature across OS generations

`debloat:disable_widgets` is Windows 11 only; `interface:disable_news_interests` is Windows 10 only.
They address the same taskbar feed feature on the two OS generations, and each is inert on the other.

**Merge recommendation (moderate):** one tweak with per-effect `windows` gating, which the schema
already supports (`performance:disable_multiplane_overlay` uses exactly this pattern for its
build-dependent keys).

### 3.10 Maps: app, service, and tasks

`debloat:remove_maps`, `services:disable_maps_broker`, `services:task_maps_update`. The latter two
are the same subsystem and the task tweak's own text says it pairs with the broker service.

**Merge recommendation (moderate, within services):** merge the broker service and the update tasks.
Keep the app removal separate.

### 3.11 Start menu suggestions: three values in three files

| Tweak | Value |
|---|---|
| `debloat:disable_start_suggestions` | `SubscribedContent-338388Enabled` |
| `interface:disable_start_recommendations` | `Start_IrisRecommendations` |
| `privacy:disable_start_app_suggestions` | `SystemPaneSuggestionsEnabled` |

Three tweaks in three categories all aimed at suggestion content in the Start menu.

**Resolved: do not merge.** The corroboration pass established that these are three genuinely
different surfaces, and that two of them are Windows 10 mechanisms:

- `SubscribedContent-338388Enabled` is the **Windows 10** Start setting "Occasionally show
  suggestions in Start", paired with `SystemPaneSuggestionsEnabled`. Four independent sources agree
  and none dissent.
- Windows 11's visually similar "Show recommendations for tips, shortcuts, new apps, and more" is a
  **different mechanism entirely**: `Start_IrisRecommendations`, with the tier A policy
  `HideRecommendedSection` behind the same surface.

So `debloat:disable_start_suggestions` and `privacy:disable_start_app_suggestions` are the Windows 10
pair, and `interface:disable_start_recommendations` is the Windows 11 control. Merging them would
have combined controls for two different operating systems into one tweak.

**Consequence for the 24H2+ re-scope.** Since the supported floor is now Windows 11 24H2, the two
Windows 10 tweaks apply only to the low-priority Windows 10 IoT Enterprise LTSC 2021 target, and
`disable_start_recommendations` is the one that matters on the primary platform. Gate them
accordingly rather than merging them.

This is a case where surface-level similarity in three tweak names strongly suggested duplication and
the evidence said otherwise. Worth remembering before merging on name resemblance.

## 4. Category placement smells

Not merge candidates, but organizational inconsistencies worth a decision.

- **Consumer content is split across `debloat` and `privacy`.** Seven tweaks write under
  `ContentDeliveryManager`: three in debloat, four in privacy. The dividing line is not apparent from
  the tweak names. Pick one category for this key.
- **Edge policies span three categories.** Three in debloat, one in privacy (`disable_edge_telemetry`),
  one in security (`enforce_smartscreen`, which writes an Edge value alongside its Windows values).
  The security placement is defensible; the debloat and privacy split is arbitrary.
- **`performance:disable_background_apps` writes an `AppPrivacy` policy** (`LetAppsRunInBackground`),
  the same key family as `privacy:disable_app_diagnostics`. Its framing is performance, its mechanism
  is privacy. Either placement is arguable; note it and move on.

## 5. Verified clean

The registry-collision audit found exactly one case of two tweaks writing the same key and value
name: `performance:variable_refresh_rate` and `performance:optimizations_windowed_games` both write
`HKCU\Software\Microsoft\DirectX\UserGpuPreferences\DirectXUserGlobalSettings`. This is intentional
and safe, because both use field addressing (`field:` plus `format: kv_semicolon`) into a packed
REG_SZ rather than scalar overwrites.

No service is targeted by more than one tweak. No scheduled task is targeted by more than one tweak.
No tweak declares an effect that no option references, and no option references an undeclared
effect.
