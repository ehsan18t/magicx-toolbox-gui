import * as tweaksApi from "$lib/api/tweaks";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { appsStore } from "./apps.svelte";
import { elevationStore } from "./elevation.svelte";
import { manualTestsStore } from "./manualTests.svelte";
import { systemStore } from "./system.svelte";
import { tweaksStore } from "./tweaksData.svelte";

let error = $state<string | null>(null);
let started = false;

/** The first failure is the one shown. */
function fail(context: string, cause: unknown) {
  error ??= errorMessage(cause);
  logError(context, cause);
}

export const bootStore = {
  /** Set when boot failed; the layout shows its one error screen. */
  get error() {
    return error;
  },

  /** Run once by App.svelte: the tweak model alongside the status stream, system info and elevation, then app presence. */
  async load(onModelSettled: () => void): Promise<void> {
    if (started) return;
    started = true;
    void manualTestsStore.load();
    // Started before the model: statuses that arrive first wait in tweaksData's pending buffer.
    const rest = Promise.all([tweaksStore.streamStatuses(), systemStore.load(), elevationStore.load()]).catch(
      (cause: unknown) => fail("Failed to initialize", cause),
    );
    try {
      await tweaksStore.load();
    } catch (cause) {
      fail("Failed to load the tweak model", cause);
      return;
    } finally {
      onModelSettled();
    }
    // Not awaited: the app presence scan is slow and must not hold up the tweak UI.
    void appsStore.load();
    await rest;
  },

  /** Re-runs the full scan after an elevation change so Unknowns become readable. */
  async rescan(): Promise<void> {
    await elevationStore.load();
    void appsStore.load();
    await tweaksApi.rescanAfterElevation();
  },
};
