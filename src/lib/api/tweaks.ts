// API functions for Tauri commands
import { invoke } from "@tauri-apps/api/core";
import type {
  BatchApplyResult,
  CategoryDefinition,
  SystemInfo,
  TweakDefinition,
  TweakResult,
  TweakStatus,
  TweakWithStatus,
} from "../types";

/**
 * Get system information including Windows version
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke<SystemInfo>("get_system_info");
}

/**
 * Get all available categories (auto-discovered from YAML files)
 */
export async function getCategories(): Promise<CategoryDefinition[]> {
  return await invoke<CategoryDefinition[]>("get_categories");
}

/**
 * Get all available tweaks
 */
export async function getAvailableTweaks(): Promise<TweakDefinition[]> {
  return await invoke<TweakDefinition[]>("get_available_tweaks");
}

/**
 * Get tweaks filtered by the current Windows version
 */
export async function getTweaksForCurrentVersion(): Promise<TweakDefinition[]> {
  const systemInfo = await getSystemInfo();
  const version = systemInfo.windows.is_windows_11 ? "11" : "10";
  return await invoke<TweakDefinition[]>("get_available_tweaks_for_version", { version });
}

/**
 * Get the status of a specific tweak
 */
export async function getTweakStatus(tweakId: string): Promise<TweakStatus> {
  return await invoke<TweakStatus>("get_tweak_status", { tweakId });
}

/**
 * Get statuses for multiple tweaks at once
 */
export async function getTweakStatuses(tweakIds: string[]): Promise<Record<string, TweakStatus>> {
  const statuses: Record<string, TweakStatus> = {};

  // Get statuses in parallel
  await Promise.all(
    tweakIds.map(async (id) => {
      try {
        statuses[id] = await getTweakStatus(id);
      } catch {
        statuses[id] = {
          tweak_id: id,
          is_applied: false,
          has_backup: false,
        };
      }
    }),
  );

  return statuses;
}

/**
 * Get all tweaks with their status
 */
export async function getAllTweaksWithStatus(): Promise<TweakWithStatus[]> {
  const tweaks = await getTweaksForCurrentVersion();
  const tweakIds = tweaks.map((t) => t.id);
  const statuses = await getTweakStatuses(tweakIds);

  return tweaks.map((definition) => ({
    definition,
    status: statuses[definition.id] || {
      tweak_id: definition.id,
      is_applied: false,
      has_backup: false,
    },
  }));
}

/**
 * Apply a specific tweak
 */
export async function applyTweak(tweakId: string): Promise<TweakResult> {
  return await invoke<TweakResult>("apply_tweak", { tweakId });
}

/**
 * Revert a specific tweak
 */
export async function revertTweak(tweakId: string): Promise<TweakResult> {
  return await invoke<TweakResult>("revert_tweak", { tweakId });
}

/**
 * Apply multiple tweaks at once
 */
export async function batchApplyTweaks(tweakIds: string[]): Promise<BatchApplyResult> {
  return await invoke<BatchApplyResult>("batch_apply_tweaks", { tweakIds });
}

/**
 * Check if running as administrator
 */
export async function isAdmin(): Promise<boolean> {
  const systemInfo = await getSystemInfo();
  return systemInfo.is_admin;
}

/**
 * Get the current Windows version string ("10" or "11")
 */
export async function getWindowsVersion(): Promise<string> {
  const systemInfo = await getSystemInfo();
  return systemInfo.windows.version_string;
}

// ============================================================================
// Backup API
// ============================================================================

export interface BackupInfo {
  tweak_id: string;
  tweak_name: string;
  created_at: string;
}

/**
 * Check if a backup exists for a tweak
 */
export async function hasBackup(tweakId: string): Promise<boolean> {
  return await invoke<boolean>("has_backup", { tweakId });
}

/**
 * List all backup tweak IDs
 */
export async function listBackups(): Promise<string[]> {
  return await invoke<string[]>("list_backups");
}

/**
 * Get backup information for a tweak
 */
export async function getBackupInfo(tweakId: string): Promise<BackupInfo | null> {
  return await invoke<BackupInfo | null>("get_backup_info", { tweakId });
}

/**
 * Restore a tweak from its backup (reverts to original state)
 */
export async function restoreFromBackup(tweakId: string): Promise<void> {
  return await invoke<void>("restore_from_backup", { tweakId });
}

/**
 * Delete a backup file
 */
export async function deleteBackup(tweakId: string): Promise<void> {
  return await invoke<void>("delete_backup", { tweakId });
}
