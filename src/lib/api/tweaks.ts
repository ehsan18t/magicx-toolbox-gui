// One function per command in src-tauri/src/commands/tweaks.rs; Tauri maps camelCase args to snake_case.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ApplyOutcome,
  CategoryMeta,
  ElevationState,
  EntrySummary,
  RestoreOutcome,
  TweakStatusEvent,
  TweakStatusView,
  TweakView,
} from "$lib/types";

/** The compiled model with this moment's availability. */
export async function getTweaks(): Promise<TweakView[]> {
  return await invoke<TweakView[]>("get_tweaks");
}

export async function getCategories(): Promise<CategoryMeta[]> {
  return await invoke<CategoryMeta[]>("get_categories");
}

/** Returns at once; statuses stream back per tweak as `tweak-status` events. */
export async function getStatusesStream(): Promise<void> {
  await invoke("get_statuses_stream");
}

/** Full re-scan after an elevation change, so Unknowns become readable. */
export async function rescanAfterElevation(): Promise<void> {
  await invoke("rescan_after_elevation");
}

/** By option label, not index. The outcome carries the fresh post-op status. */
export async function applyTweak(tweakId: string, optionLabel: string): Promise<ApplyOutcome> {
  return await invoke<ApplyOutcome>("apply_tweak", { tweakId, optionLabel });
}

/** Single head-walk restore of the most recent snapshot entry. */
export async function restoreTweak(tweakId: string): Promise<RestoreOutcome> {
  return await invoke<RestoreOutcome>("restore_tweak", { tweakId });
}

/** For the failure paths that carry no outcome. */
export async function getTweakStatus(tweakId: string): Promise<TweakStatusView> {
  return await invoke<TweakStatusView>("get_tweak_status", { tweakId });
}

export async function listSnapshotEntries(tweakId: string): Promise<EntrySummary[]> {
  return await invoke<EntrySummary[]>("list_snapshot_entries", { tweakId });
}

/** Explicit-consent release of one entry (ADR-0002). */
export async function discardSnapshotEntry(tweakId: string, seq: number): Promise<void> {
  await invoke("discard_snapshot_entry", { tweakId, seq });
}

/** Releases the Needs Attention record and any entries left (ADR-0002), returning the fresh status. */
export async function keepCurrentState(tweakId: string): Promise<TweakStatusView> {
  return await invoke<TweakStatusView>("keep_current_state", { tweakId });
}

export async function getElevationState(): Promise<ElevationState> {
  return await invoke<ElevationState>("get_elevation_state");
}

/** Register before `getStatusesStream`/`rescanAfterElevation`, so no early event is missed. Each event is a batch. */
export async function onTweakStatus(handler: (events: TweakStatusEvent[]) => void): Promise<UnlistenFn> {
  return await listen<TweakStatusEvent[]>("tweak-status", (event) => handler(event.payload));
}
