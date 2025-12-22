/**
 * Profile Store - Svelte 5 Runes
 *
 * Manages profile export/import state for configuration profiles.
 */

import type { ConfigurationProfile, ProfileApplyResult, ProfileValidation, TweakChangePreview } from "$lib/api/profile";
import * as profileApi from "$lib/api/profile";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { open, save } from "@tauri-apps/plugin-dialog";

// === Export State ===
let isExporting = $state(false);
let exportError = $state<string | null>(null);

// === Import State ===
let isImporting = $state(false);
let importError = $state<string | null>(null);

// === Saved Profiles State ===
let savedProfiles = $state<profileApi.ProfileMetadata[]>([]);
let loadingSavedProfiles = $state(false);
let savedProfilesError = $state<string | null>(null);

// Persistent store for profile directory
const currentProfileDirStore = new PersistentStore<string | null>("magicx_profile_dir", null);

// === Apply State ===
let isApplying = $state(false);
let applyProgress = $state<{ current: number; total: number } | null>(null);
let applyError = $state<string | null>(null);

// === Profile Data ===
let currentProfile = $state<ConfigurationProfile | null>(null);
let validation = $state<ProfileValidation | null>(null);
let applyResult = $state<ProfileApplyResult | null>(null);

// === Derived Values ===
const applicableTweaks = $derived.by(() => {
  if (!validation) return [];
  return validation.preview.filter((p) => p.applicable && !p.already_applied);
});

const warningCount = $derived(validation?.warnings.length ?? 0);
const errorCount = $derived(validation?.errors.length ?? 0);

// === Export Store ===
export const profileStore = {
  // Getters
  get isExporting() {
    return isExporting;
  },
  get exportError() {
    return exportError;
  },
  get isImporting() {
    return isImporting;
  },
  get importError() {
    return importError;
  },
  get savedProfiles() {
    return savedProfiles;
  },
  get loadingSavedProfiles() {
    return loadingSavedProfiles;
  },
  get savedProfilesError() {
    return savedProfilesError;
  },
  get currentProfileDir() {
    return currentProfileDirStore.value;
  },
  get isApplying() {
    return isApplying;
  },
  get applyProgress() {
    return applyProgress;
  },
  get applyError() {
    return applyError;
  },
  get currentProfile() {
    return currentProfile;
  },
  get validation() {
    return validation;
  },
  get applyResult() {
    return applyResult;
  },
  get applicableTweaks() {
    return applicableTweaks;
  },
  get warningCount() {
    return warningCount;
  },
  get errorCount() {
    return errorCount;
  },

  /**
   * Set custom profile directory and reload profiles.
   */

  /**
   * Set custom profile directory and reload profiles.
   */
  setProfileDir(path: string | null) {
    currentProfileDirStore.value = path;
    this.loadSavedProfiles();
  },

  /**
   * Export a profile to a file.
   * Opens a save dialog and exports the selected tweaks.
   */
  async exportProfile(
    name: string,
    tweakIds: string[],
    options?: {
      description?: string;
      includeSystemState?: boolean;
    },
  ): Promise<boolean> {
    if (isExporting) return false;

    isExporting = true;
    exportError = null;

    try {
      // Open save dialog
      const filePath = await save({
        defaultPath: `${name.toLowerCase().replace(/\s+/g, "-")}.mgx`,
        filters: [{ name: "MagicX Profile", extensions: ["mgx"] }],
      });

      if (!filePath) {
        // User cancelled
        return false;
      }

      await profileApi.exportProfile(filePath, name, tweakIds, options);
      return true;
    } catch (error) {
      console.error("Failed to export profile:", error);
      exportError = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      isExporting = false;
      isExporting = false;
      // Reload profiles after export
      this.loadSavedProfiles();
    }
  },

  /**
   * Load the list of saved profiles.
   */
  async loadSavedProfiles() {
    loadingSavedProfiles = true;
    savedProfilesError = null;
    try {
      savedProfiles = await profileApi.getSavedProfiles(currentProfileDirStore.value);
    } catch (error) {
      console.error("Failed to load saved profiles:", error);
      savedProfilesError = error instanceof Error ? error.message : String(error);
    } finally {
      loadingSavedProfiles = false;
    }
  },

  /**
   * Delete a saved profile.
   */
  async deleteProfile(name: string): Promise<boolean> {
    try {
      await profileApi.deleteSavedProfile(name, currentProfileDirStore.value);
      await this.loadSavedProfiles();
      return true;
    } catch (error) {
      console.error("Failed to delete profile:", error);
      // throw error to let UI handle it or set a store error?
      // Since it's a specific action, returning false/throwing is often better for immediate UI feedback.
      // But let's set a shared error for simplicity if we want, or just rethrow.
      // Let's rely on caller to show toast, but we handle the reload.
      throw error;
    }
  },

  /**
   * Import a profile from a file.
   * Opens a file dialog and loads + validates the profile.
   */
  async importProfile(): Promise<boolean> {
    if (isImporting) return false;

    isImporting = true;
    importError = null;
    currentProfile = null;
    validation = null;
    applyResult = null;

    try {
      // Open file dialog
      const filePath = await open({
        multiple: false,
        filters: [{ name: "MagicX Profile", extensions: ["mgx"] }],
      });

      if (!filePath || typeof filePath !== "string") {
        // User cancelled
        return false;
      }

      const [profile, validationResult] = await profileApi.importProfile(filePath);
      currentProfile = profile;
      validation = validationResult;
      return true;
    } catch (error) {
      console.error("Failed to import profile:", error);
      importError = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      isImporting = false;
    }
  },

  /**
   * Import a profile from a file path (for drag-drop).
   * @param filePath - Path to the profile file
   */
  async importProfileFromPath(filePath: string): Promise<boolean> {
    if (isImporting) return false;

    isImporting = true;
    importError = null;
    currentProfile = null;
    validation = null;
    applyResult = null;

    try {
      const [profile, validationResult] = await profileApi.importProfile(filePath);
      currentProfile = profile;
      validation = validationResult;
      return true;
    } catch (error) {
      console.error("Failed to import profile:", error);
      importError = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      isImporting = false;
    }
  },

  /**
   * Apply the currently loaded profile.
   */
  async applyProfile(options?: {
    skipTweakIds?: string[];
    skipAlreadyApplied?: boolean;
    createRestorePoint?: boolean;
  }): Promise<boolean> {
    if (isApplying || !currentProfile) return false;

    isApplying = true;
    applyError = null;
    applyResult = null;

    // Calculate total tweaks to apply for progress tracking
    const tweaksToApply =
      validation?.preview.filter(
        (p) =>
          p.applicable &&
          !(options?.skipAlreadyApplied && p.already_applied) &&
          !options?.skipTweakIds?.includes(p.tweak_id),
      ) ?? [];

    applyProgress = { current: 0, total: tweaksToApply.length };

    try {
      const result = await profileApi.applyProfile(currentProfile, options);
      applyResult = result;
      applyProgress = { current: result.applied_count, total: tweaksToApply.length };
      return result.success;
    } catch (error) {
      console.error("Failed to apply profile:", error);
      applyError = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      isApplying = false;
    }
  },

  /**
   * Clear all profile state.
   */
  clear() {
    isExporting = false;
    exportError = null;
    isImporting = false;
    importError = null;
    isApplying = false;
    applyProgress = null;
    applyError = null;
    currentProfile = null;
    validation = null;
    applyResult = null;
  },

  /**
   * Get a preview item by tweak ID.
   */
  getPreviewByTweakId(tweakId: string): TweakChangePreview | undefined {
    return validation?.preview.find((p) => p.tweak_id === tweakId);
  },
};

// Re-export types for convenience
export type {
  ChangeDetail,
  ChangeType,
  ConfigurationProfile,
  ProfileApplyResult,
  ProfileValidation,
  TweakChangePreview,
  ValidationError,
  ValidationStats,
  ValidationWarning,
} from "$lib/api/profile";
