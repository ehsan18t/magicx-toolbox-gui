# Registry / Service Address Collision Audit

Companion to [the tweak catalog](./2026-07-23-windows-tweak-catalog.md). Checks whether any two tweaks
in the catalog write the **same address** (registry `hive` + `key` + `value-name`, or the same service /
scheduled task), which matters because the redesigned engine enforces **one owner per address**: two
tweaks owning the same whole value is a build-time `DuplicateAddress` error.

Method: every tweak's mechanism was transcribed into normalized rows (272 registry writes, 59
service/task rows), then grouped by address, case-insensitively, exactly like the engine's `CoarseKey`.
This is deterministic; no collision is inferred by judgement.

## 1. Same-address, different-value conflict (the task-1 case)

**One** found, and it is a **packed value**, not a whole-value clash:

- `HKCU\Software\Microsoft\DirectX\UserGpuPreferences\DirectXUserGlobalSettings` (REG_SZ)
  - `variable_refresh_rate` writes the field `VRROptimizeEnable=1;`
  - `optimizations_windowed_games` writes the field `SwapEffectUpgradeEnable=1;`

`DirectXUserGlobalSettings` is a single REG_SZ holding semicolon-separated `key=value` fields. The two
tweaks own **different fields of the same value**, which is exactly the collision discussed in task 1.

**Resolution: no new machinery needed.** The engine already supports `format: kv_semicolon` field
addressing, and its uniqueness guard permits two tweaks to own **different field names** in one packed
value (only whole-value-vs-field or same-field collisions are rejected). Author each as a field claim
(`field: VRROptimizeEnable` and `field: SwapEffectUpgradeEnable`) on the shared REG_SZ address. This is
the concrete evidence that the deferred bitmask / shared-value feature is **not required for this
corpus**; the existing kv_semicolon mechanism covers the one real case.

(The same key also carries per-app GPU-preference values under different value-names via
`gpu_preference_default`; different value-name, so no conflict.)

## 2. Duplicate reg addresses to consolidate before authoring

Same address, **same** value, authored twice under different tweak names across domains. Each pair
would build-error if both were authored, so consolidate to one owner (merge, or make one a shared
setting):

| Address | Tweaks | Value |
|---|---|---|
| `HKLM\...\WindowsAI\DisableAIDataAnalysis` | `disable_recall_snapshots`, `disable_recall` | `1` |
| `HKLM\...\Windows Search\AllowCortana` | `disable_cortana`, `remove_cortana` | `0` |
| `HKCU\...\WindowsCopilot\TurnOffWindowsCopilot` | `disable_copilot_taskbar`, `disable_copilot_policy` | `1` |
| `HKCU\...\CLSID\{e88865ea-...}\System.IsPinnedToNameSpaceTree` (Gallery) | `remove_gallery_nav_pane`, `hide_explorer_gallery` | `0` |
| `HKCU\System\GameConfigStore\GameDVR_Enabled` | `disable_gamedvr_capture`, `disable_game_dvr` | `0` |
| `HKLM\...\GameDVR\AllowGameDVR` | `disable_gamedvr_capture`, `disable_game_dvr` | `0` |

These are cross-domain redundancies the research produced (the same idea authored under a Privacy name
and a Debloat name, etc.). They are not conflicts (values agree), just duplication to fold together.

## 3. Duplicate service / scheduled-task targets

Same target driven by two tweaks (again, consolidate to one owner):

- Service `DiagTrack`: `disable_diagtrack` and `disable_diagtrack_service` (both set it Disabled).
- Task `...\Application Experience\Microsoft Compatibility Appraiser`: `task_compat_appraiser` and `disable_compat_appraiser`.
- Task `...\Application Experience\ProgramDataUpdater`: `task_program_data_updater` and `disable_compat_appraiser`.
- Task `...\Customer Experience Improvement Program\Consolidator`: `task_ceip_consolidator` and `disable_ceip_tasks`.
- Task `...\Customer Experience Improvement Program\UsbCeip`: `task_ceip_usbceip` and `disable_ceip_tasks`.

Decision to make when authoring: let the per-item Services tweaks own these tasks, or let the Privacy
"bundle" tweaks own them, but not both.

## 4. Co-located keys (informational, not conflicts)

24 keys are written by 2+ tweaks under **different value-names**, which the engine allows (each
value-name is a distinct address). No action needed, but two are worth noting:

- `...\WindowsUpdate\AU` (and the wider WindowsUpdate policy area): several update tweaks live here.
  The `NoAutoUpdate` / `AUOptions` relationship (see the round-1 value fix) means these must be
  authored as **one tweak's Surface with multiple Options**, not as competing tweaks, or they will
  fight over `AUOptions`.
- `HKCU\...\Explorer\Advanced` (16 tweaks) and `ContentDeliveryManager` (7 tweaks) are shared authoring
  areas: fine as independent value-names, but a good argument for grouping them into fewer, broader
  tweaks.

## 5. Extraction caveats

A few rows came back with `UNKNOWN` hive or empty value data where the mechanism did not state them
crisply (e.g. `LanmanServer\Parameters`, `Policies\System`, `DeviceGuard`, the WindowsUpdate policy
block). These are almost certainly `HKLM`, but confirm the hive, value-name, and exact data at
authoring time. This audit finds address *collisions*; it does not re-verify the values themselves
(that was the job of the two validation passes).

## Conclusion for task 1

The real corpus contains exactly one same-value multi-writer case, and it is a `kv_semicolon` packed
REG_SZ that the current engine already handles through field addressing. No bitmask or shared-value
feature is needed to author this catalog. The remaining overlaps are duplicate authoring (Sections 2
and 3) to fold together, not engine gaps.
