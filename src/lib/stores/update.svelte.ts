import { exitApp } from "$lib/api/platform";
import * as updateApi from "$lib/api/update";
import { APP_CONFIG } from "$lib/config/app";
import type { DownloadProgress, UpdateInfo } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { HOUR_MS } from "$lib/utils/time";
import { modalStore } from "./modal.svelte";
import { settingsStore } from "./settings.svelte";
import { toastStore } from "./toast.svelte";

const UPDATE_CHECK_HOURS = 1;
const UPDATE_CHECK_INTERVAL_MS = UPDATE_CHECK_HOURS * HOUR_MS;
/** How often the background check may run, as the setting describes it. */
export const UPDATE_CHECK_CADENCE = UPDATE_CHECK_HOURS > 1 ? `once every ${UPDATE_CHECK_HOURS} hours` : "once an hour";
// Lets the installer start before the app exits.
const EXIT_AFTER_INSTALL_MS = 1000;

let isChecking = $state(false);
let isInstalling = $state(false);
let downloadProgress = $state<DownloadProgress | null>(null);
let updateInfo = $state<UpdateInfo | null>(null);
let lastError = $state<string | null>(null);

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
  if (!silent) lastError = null;

  try {
    const { source, flags } = APP_CONFIG.update.assetPattern;
    const result = await updateApi.checkForUpdate({
      releasesApiUrl: APP_CONFIG.update.releasesApiUrl,
      // `source` drops the flags; regex_lite reads case-insensitivity inline.
      assetPattern: (flags.includes("i") ? "(?i)" : "") + source,
      includePrereleases: settingsStore.includePrereleases,
    });
    stampLastCheck();
    if (seq === checkSeq) {
      updateInfo = result;
      lastError = null;
    }
    return result;
  } catch (error) {
    logError("Update check failed", error);
    if (!silent && seq === checkSeq) lastError = errorMessage(error);
    return null;
  } finally {
    if (seq === checkSeq) isChecking = false;
  }
}

async function installUpdate(): Promise<boolean> {
  if (isInstalling) return false;
  if (!updateInfo?.available || !updateInfo.downloadUrl || !updateInfo.assetName) {
    lastError = "No update available to install";
    return false;
  }

  isInstalling = true;
  lastError = null;
  downloadProgress = null;
  try {
    await updateApi.installUpdate(
      updateInfo.downloadUrl,
      updateInfo.assetName,
      updateInfo.assetDigest ?? null,
      updateInfo.assetSize ?? null,
      (progress) => (downloadProgress = progress),
    );
    return true;
  } catch (error) {
    logError("Update installation failed", error);
    const message = errorMessage(error);
    if (isAppExiting(error)) toastStore.warning(message);
    else lastError = message;
    return false;
  } finally {
    isInstalling = false;
    downloadProgress = null;
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

  get downloadProgress() {
    return downloadProgress;
  },

  get error() {
    return lastError;
  },

  get isAvailable() {
    return isAvailable;
  },

  checkForUpdate,

  /** Silent background check, at most once per `UPDATE_CHECK_INTERVAL_MS`, when enabled. */
  autoCheckIfDue() {
    if (!settingsStore.autoCheckUpdates) return;
    const lastCheck = settingsStore.lastUpdateCheck;
    // Negated so an unparseable stamp (NaN) counts as due.
    const due = !lastCheck || !(Date.now() - Date.parse(lastCheck) <= UPDATE_CHECK_INTERVAL_MS);
    // Skipped while any check runs: a newer sequence number would discard a manual check's result.
    if (due && !isChecking) void checkForUpdate(true);
  },

  /** Installs, then exits so the installer can replace the app. */
  async installAndExit(): Promise<void> {
    if (!(await installUpdate())) return;
    await new Promise((resolve) => setTimeout(resolve, EXIT_AFTER_INSTALL_MS));
    try {
      await exitApp();
    } catch {
      // The installer is already running, and the backend keeps refusing applies until exit.
      modalStore.close();
      toastStore.warning(
        "The installer is running, but the app could not close itself. Close the app to finish the update.",
      );
    }
  },

  clearError() {
    lastError = null;
  },
};
