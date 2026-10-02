import * as api from "$lib/api/update";
import { APP_CONFIG } from "$lib/config/app";
import type { UpdateInfo } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { settingsStore } from "./settings.svelte";
import { toastStore } from "./toast.svelte";

const UPDATE_CHECK_INTERVAL_MS = 60 * 60 * 1000;

let isChecking = $state(false);
let isInstalling = $state(false);
let updateInfo = $state<UpdateInfo | null>(null);
let error = $state<string | null>(null);

const isAvailable = $derived(updateInfo?.available ?? false);

// Overlapping checks (startup, manual, pre-release toggle) finish out of order; only the latest writes.
let checkSeq = 0;

function stampLastCheck(): void {
  settingsStore.setLastUpdateCheck(new Date().toISOString());
}

/** A silent check (background) never records its error. */
async function checkForUpdate(silent = false): Promise<UpdateInfo | null> {
  const seq = ++checkSeq;
  isChecking = true;
  if (!silent) error = null;

  try {
    const { source, flags } = APP_CONFIG.update.assetPattern;
    const result = await api.checkForUpdate({
      releasesApiUrl: APP_CONFIG.update.releasesApiUrl,
      // `source` drops the flags; regex_lite reads case-insensitivity inline.
      assetPattern: (flags.includes("i") ? "(?i)" : "") + source,
      includePrereleases: settingsStore.includePrereleases,
    });
    stampLastCheck();
    if (seq === checkSeq) {
      updateInfo = result;
      error = null;
    }
    return result;
  } catch (err) {
    logError("Update check failed", err);
    if (!silent && seq === checkSeq) error = errorMessage(err);
    return null;
  } finally {
    if (seq === checkSeq) isChecking = false;
  }
}

export const updateStore = {
  get isChecking() {
    return isChecking;
  },

  get isInstalling() {
    return isInstalling;
  },

  get updateInfo() {
    return updateInfo;
  },

  get error() {
    return error;
  },

  get isAvailable() {
    return isAvailable;
  },

  checkForUpdate,

  /** Silent background check, at most once per `UPDATE_CHECK_INTERVAL_MS`, when enabled. */
  autoCheckIfDue() {
    if (!settingsStore.autoCheckUpdates) return;
    const lastCheck = settingsStore.lastUpdateCheck;
    const due = !lastCheck || Date.now() - Date.parse(lastCheck) > UPDATE_CHECK_INTERVAL_MS;
    if (due) void checkForUpdate(true);
  },

  async installUpdate(): Promise<boolean> {
    if (!updateInfo?.available || !updateInfo.downloadUrl || !updateInfo.assetName) {
      error = "No update available to install";
      return false;
    }

    isInstalling = true;
    error = null;
    try {
      await api.installUpdate(updateInfo.downloadUrl, updateInfo.assetName, updateInfo.assetDigest ?? null);
      return true;
    } catch (err) {
      logError("Update installation failed", err);
      const message = errorMessage(err);
      if (isAppExiting(err)) toastStore.warning(message);
      else error = message;
      return false;
    } finally {
      isInstalling = false;
    }
  },

  clearError() {
    error = null;
  },
};
