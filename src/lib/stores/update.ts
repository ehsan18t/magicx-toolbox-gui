// Update state store for tracking update availability and status
import { APP_CONFIG } from "$lib/config/app";
import type { UpdateInfo } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { get, writable } from "svelte/store";

export interface UpdateState {
  /** Whether we're currently checking for updates */
  isChecking: boolean;
  /** Whether we're currently downloading/installing an update */
  isInstalling: boolean;
  /** The latest update info from the last check */
  updateInfo: UpdateInfo | null;
  /** Error message from the last check */
  error: string | null;
  /** Whether the last check was done silently (background) */
  lastCheckWasSilent: boolean;
}

const initialState: UpdateState = {
  isChecking: false,
  isInstalling: false,
  updateInfo: null,
  error: null,
  lastCheckWasSilent: false,
};

function createUpdateStore() {
  const { subscribe, set, update } = writable<UpdateState>(initialState);

  return {
    subscribe,

    /**
     * Check for updates from GitHub releases
     * @param silent If true, errors won't be stored (for background checks)
     */
    async checkForUpdate(silent: boolean = false): Promise<UpdateInfo | null> {
      update((state) => ({
        ...state,
        isChecking: true,
        error: silent ? state.error : null,
        lastCheckWasSilent: silent,
      }));

      try {
        const config = {
          releasesApiUrl: APP_CONFIG.update.releasesApiUrl,
          assetPattern: APP_CONFIG.update.assetPattern.source,
        };

        const result = await invoke<UpdateInfo>("check_for_update", { config });

        update((state) => ({
          ...state,
          isChecking: false,
          updateInfo: result,
          error: null,
        }));

        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.error("Update check failed:", errorMessage);

        update((state) => ({
          ...state,
          isChecking: false,
          // Only store error for non-silent checks
          error: silent ? state.error : errorMessage,
        }));

        return null;
      }
    },

    /**
     * Download and install an available update
     */
    async installUpdate(): Promise<boolean> {
      const state = get({ subscribe });

      if (!state.updateInfo?.available || !state.updateInfo.downloadUrl || !state.updateInfo.assetName) {
        update((s) => ({
          ...s,
          error: "No update available to install",
        }));
        return false;
      }

      update((s) => ({
        ...s,
        isInstalling: true,
        error: null,
      }));

      try {
        await invoke("install_update", {
          downloadUrl: state.updateInfo.downloadUrl,
          assetName: state.updateInfo.assetName,
        });

        update((s) => ({
          ...s,
          isInstalling: false,
        }));

        return true;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.error("Update installation failed:", errorMessage);

        update((s) => ({
          ...s,
          isInstalling: false,
          error: errorMessage,
        }));

        return false;
      }
    },

    /**
     * Clear any stored error
     */
    clearError() {
      update((state) => ({ ...state, error: null }));
    },

    /**
     * Reset the store to initial state
     */
    reset() {
      set(initialState);
    },

    /**
     * Get current state synchronously
     */
    get() {
      return get({ subscribe });
    },
  };
}

export const updateStore = createUpdateStore();

// Derived store for checking if update is available
export const isUpdateAvailable = {
  subscribe(callback: (value: boolean) => void) {
    return updateStore.subscribe((state) => {
      callback(state.updateInfo?.available ?? false);
    });
  },
};
