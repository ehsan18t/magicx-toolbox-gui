// API for the redesigned tweak engine (Task 16 command + event contract).
// Every function here maps 1:1 to a command registered in src-tauri/src/lib.rs's
// generate_handler! — nothing else. Tauri maps camelCase JS args to snake_case Rust
// params (e.g. `tweakId` -> `tweak_id`) and serializes Rust snake_case fields back.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ApplyOutcome,
  ElevationState,
  EntrySummary,
  RestoreOutcome,
  SystemInfo,
  TweakStatusEvent,
  TweakView,
} from "../types";

/** System information including Windows version (from commands/system.rs — survives). */
export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke<SystemInfo>("get_system_info");
}

/** The compiled tweak model: identity/display metadata + this moment's availability. */
export async function getTweaks(): Promise<TweakView[]> {
  return await invoke<TweakView[]>("get_tweaks");
}

/**
 * Kick the background-progressive full status scan. Returns immediately; results
 * stream back one-per-tweak via the `tweak-status` event (subscribe with `onTweakStatus`).
 */
export async function getStatusesStream(): Promise<void> {
  await invoke("get_statuses_stream");
}

/** Full re-scan after an elevation change (so Unknowns become readable). */
export async function rescanAfterElevation(): Promise<void> {
  await invoke("rescan_after_elevation");
}

/**
 * Apply a tweak by option LABEL (not index). The outcome carries the fresh post-op
 * status — callers use it directly rather than re-fetching.
 */
export async function applyTweak(tweakId: string, optionLabel: string): Promise<ApplyOutcome> {
  return await invoke<ApplyOutcome>("apply_tweak", { tweakId, optionLabel });
}

/** Single head-walk restore of a tweak's most recent snapshot entry. */
export async function restoreTweak(tweakId: string): Promise<RestoreOutcome> {
  return await invoke<RestoreOutcome>("restore_tweak", { tweakId });
}

/** List a tweak's snapshot entries (drives the discard affordance). */
export async function listSnapshotEntries(tweakId: string): Promise<EntrySummary[]> {
  return await invoke<EntrySummary[]>("list_snapshot_entries", { tweakId });
}

/** Explicit-consent snapshot release: discard one entry by its sequence number. */
export async function discardSnapshotEntry(tweakId: string, seq: number): Promise<void> {
  await invoke("discard_snapshot_entry", { tweakId, seq });
}

/** The app's current elevation ceiling + over-the-shoulder SID guard reading. */
export async function getElevationState(): Promise<ElevationState> {
  return await invoke<ElevationState>("get_elevation_state");
}

/**
 * Subscribe to the per-tweak `tweak-status` event stream. Returns an unlisten fn.
 * Register this BEFORE calling `getStatusesStream`/`rescanAfterElevation` so no early
 * event is missed.
 */
export async function onTweakStatus(handler: (event: TweakStatusEvent) => void): Promise<UnlistenFn> {
  return await listen<TweakStatusEvent>("tweak-status", (event) => handler(event.payload));
}
