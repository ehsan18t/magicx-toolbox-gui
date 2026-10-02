import * as systemApi from "$lib/api/system";
import * as tweaksApi from "$lib/api/tweaks";
import type { Level } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { toastStore } from "./toast.svelte";

let level = $state<Level | null>(null);
// A second call while the UAC prompt is up would raise a second prompt.
let isRestarting = $state(false);

/** The app's elevation ceiling (spec §9); the per-tweak `availability` carries the SID guard's reason. */
export const elevationStore = {
  get level(): Level {
    return level ?? "User";
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
      logError("Failed to restart as admin", error);
      // An exit refusal ran nothing, so it is a warning.
      if (isAppExiting(error)) toastStore.warning(errorMessage(error));
      else toastStore.error(errorMessage(error));
    } finally {
      isRestarting = false;
    }
  },
};
