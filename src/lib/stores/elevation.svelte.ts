import * as systemApi from "$lib/api/system";
import { getElevationState } from "$lib/api/tweaks";
import type { Level } from "$lib/types";
import { reportFailure } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";

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
      level = (await getElevationState()).level;
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
      reportFailure("Failed to restart as admin", error);
    } finally {
      isRestarting = false;
    }
  },
};
