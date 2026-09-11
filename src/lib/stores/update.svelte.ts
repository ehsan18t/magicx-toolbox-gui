/**
 * Update Store - Svelte 5 Runes
 *
 * Manages app update checking and installation state.
 */

import { APP_CONFIG } from "$lib/config/app";
import { toastStore } from "$lib/stores/toast.svelte";
import type { UpdateInfo } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { invoke } from "@tauri-apps/api/core";

// === State ===

/** Whether we're currently checking for updates */
let isChecking = $state(false);

/** Whether we're currently downloading/installing an update */
let isInstalling = $state(false);

/** The latest update info from the last check */
let updateInfo = $state<UpdateInfo | null>(null);

/** Error message from the last check */
let error = $state<string | null>(null);

/** Whether the last check was done silently (background) */
let lastCheckWasSilent = $state(false);

// Derived: is update available
const isAvailable = $derived(updateInfo?.available ?? false);

// === Export ===

export const updateStore = {
  /** Whether currently checking for updates */
  get isChecking() {
    return isChecking;
  },

  /** Whether currently installing an update */
  get isInstalling() {
    return isInstalling;
  },

  /** Current update info */
  get updateInfo() {
    return updateInfo;
  },

  /** Current error message */
  get error() {
    return error;
  },

  /** Whether last check was silent */
  get lastCheckWasSilent() {
    return lastCheckWasSilent;
  },

  /** Whether an update is available */
  get isAvailable() {
    return isAvailable;
  },

  /**
   * Check for updates from GitHub releases
   * @param silent If true, errors won't be stored (for background checks)
   */
  async checkForUpdate(silent: boolean = false): Promise<UpdateInfo | null> {
    isChecking = true;
    if (!silent) {
      error = null;
    }
    lastCheckWasSilent = silent;

    try {
      const config = {
        releasesApiUrl: APP_CONFIG.update.releasesApiUrl,
        assetPattern: APP_CONFIG.update.assetPattern.source,
      };

      const result = await invoke<UpdateInfo>("check_for_update", { config });
      updateInfo = result;
      error = null;
      return result;
    } catch (err) {
      const message = errorMessage(err);
      console.error("Update check failed:", message);

      // Only store error for non-silent checks
      if (!silent) {
        error = message;
      }

      return null;
    } finally {
      isChecking = false;
    }
  },

  /**
   * Download and install an available update
   */
  async installUpdate(): Promise<boolean> {
    if (!updateInfo?.available || !updateInfo.downloadUrl || !updateInfo.assetName) {
      error = "No update available to install";
      return false;
    }

    isInstalling = true;
    error = null;

    try {
      await invoke("install_update", {
        downloadUrl: updateInfo.downloadUrl,
        assetName: updateInfo.assetName,
      });
      return true;
    } catch (err) {
      const message = errorMessage(err);
      console.error("Update installation failed:", message);
      if (isAppExiting(err)) toastStore.warning(message);
      else error = message;
      return false;
    } finally {
      isInstalling = false;
    }
  },

  /** Clear any stored error */
  clearError() {
    error = null;
  },

  /** Reset the store to initial state */
  reset() {
    isChecking = false;
    isInstalling = false;
    updateInfo = null;
    error = null;
    lastCheckWasSilent = false;
  },
};
