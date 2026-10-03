import * as updateApi from "$lib/api/update";
import type { DownloadProgress, UpdateInfo } from "$lib/types";
import { errorMessage, isAppExiting, isUpdateFolderReadOnly } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { HOUR_MS } from "$lib/utils/time";
import { settingsStore } from "./settings.svelte";
import { toastStore } from "./toast.svelte";

const UPDATE_CHECK_HOURS = 1;
const UPDATE_CHECK_INTERVAL_MS = UPDATE_CHECK_HOURS * HOUR_MS;
/** How often the background check may run, as the setting describes it. */
export const UPDATE_CHECK_CADENCE = UPDATE_CHECK_HOURS > 1 ? `once every ${UPDATE_CHECK_HOURS} hours` : "once an hour";

let isChecking = $state(false);
let isInstalling = $state(false);
let downloadProgress = $state<DownloadProgress | null>(null);
let updateInfo = $state<UpdateInfo | null>(null);
let lastError = $state<string | null>(null);
let manualDownloadOnly = $state(false);

function setError(message: string | null, manualDownload = false): void {
  lastError = message;
  manualDownloadOnly = manualDownload;
}

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
  if (!silent) setError(null);

  try {
    const result = await updateApi.checkForUpdate({
      includePrereleases: settingsStore.includePrereleases,
    });
    stampLastCheck();
    if (seq === checkSeq) {
      updateInfo = result;
      setError(null);
    }
    return result;
  } catch (error) {
    logError("Update check failed", error);
    if (!silent && seq === checkSeq) setError(errorMessage(error));
    return null;
  } finally {
    if (seq === checkSeq) isChecking = false;
  }
}

/** On success the backend has started the new version and exits this one. */
async function installUpdate(): Promise<void> {
  if (isInstalling) return;
  if (!updateInfo?.available || !updateInfo.downloadUrl || !updateInfo.assetName) {
    setError("No update available to install");
    return;
  }

  isInstalling = true;
  setError(null);
  downloadProgress = null;
  try {
    await updateApi.installUpdate(
      updateInfo.downloadUrl,
      updateInfo.assetName,
      updateInfo.assetDigest ?? null,
      updateInfo.assetSize ?? null,
      (progress) => (downloadProgress = progress),
    );
  } catch (error) {
    logError("Update installation failed", error);
    const message = errorMessage(error);
    if (isAppExiting(error)) toastStore.warning(message);
    else setError(message, isUpdateFolderReadOnly(error));
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

  /** The last install failed because this copy cannot replace itself. */
  get manualDownloadOnly() {
    return manualDownloadOnly;
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

  installUpdate,

  clearError() {
    setError(null);
  },
};
