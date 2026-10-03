import * as systemApi from "$lib/api/system";
import * as tweaksApi from "$lib/api/tweaks";
import type { Level } from "$lib/types";
import { errorMessage, isElevationDeclined } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { toastStore } from "./toast.svelte";

let level = $state<Level | null>(null);
// A second call while the UAC prompt is up would raise a second prompt.
let isRestarting = $state(false);

/** The app's elevation ceiling (ADR-0005); the per-tweak `availability` carries the SID guard's reason. */
export const elevationStore = {
  get level(): Level {
    return level ?? "User";
  },

  /** Null until loaded. */
  get isAdmin(): boolean | null {
    return level === null ? null : level !== "User";
  },

  /** Null until loaded. */
  get runningAs(): string | null {
    return level === null ? null : level === "User" ? "Standard user" : "Administrator";
  },

  get isRestarting() {
    return isRestarting;
  },

  async load() {
    try {
      level = (await tweaksApi.getElevationState()).level;
    } catch (error) {
      logError("Failed to load elevation state", error);
    }
  },

  /** Relaunches elevated; on success this process exits, so only a failure returns. */
  async restartAsAdmin(): Promise<void> {
    if (isRestarting) return;
    isRestarting = true;
    try {
      await systemApi.restartAsAdmin();
    } catch (error) {
      if (isElevationDeclined(error)) toastStore.info(errorMessage(error));
      else toastStore.failure("Failed to restart as admin", error);
    } finally {
      isRestarting = false;
    }
  },
};
