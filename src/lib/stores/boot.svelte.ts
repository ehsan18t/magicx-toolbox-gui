import * as tweaksApi from "$lib/api/tweaks";
import { appsStore } from "./apps.svelte";
import { elevationStore } from "./elevation.svelte";
import { systemStore } from "./system.svelte";
import { tweaksStore } from "./tweaksData.svelte";

let isLoaded = false;
let loadPromise: Promise<void> | null = null;

/** Everything after the tweak model: system info, elevation, app presence and the status stream. */
export const bootStore = {
  /**
   * Awaits the model first: Svelte mounts `+page` before `+layout`, so this runs before the layout's
   * `tweaksStore.load()` and joins its promise-cached load.
   */
  load(): Promise<void> {
    if (isLoaded) return Promise.resolve();
    loadPromise ??= tweaksStore
      .load()
      .then(() => {
        // Not awaited: the app presence scan is slow and must not hold up the tweak UI.
        void appsStore.load();
        return Promise.all([systemStore.load(), elevationStore.load(), tweaksStore.streamStatuses()]);
      })
      .then(() => {
        isLoaded = true;
      })
      .finally(() => {
        loadPromise = null;
      });
    return loadPromise;
  },

  /** Re-runs the full scan after an elevation change so Unknowns become readable. */
  async rescan(): Promise<void> {
    await elevationStore.load();
    void appsStore.load();
    await tweaksApi.rescanAfterElevation();
  },
};
