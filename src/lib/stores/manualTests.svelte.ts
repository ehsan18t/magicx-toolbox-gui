// Test build only: `isAvailable` stays false in a normal build, which hides the view.

import * as manualTestsApi from "$lib/api/manualTests";
import type { ManualTest, ManualTestReport } from "$lib/types";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";

let isAvailable = $state(false);
let tests = $state.raw<ManualTest[]>([]);
let runningId = $state<string | null>(null);
let isCancelling = $state(false);
let logs = $state<Record<string, string[]>>({});
let results = $state<Record<string, ManualTestReport>>({});
let failures = $state<Record<string, string>>({});
let isLoaded = false;

export const manualTestsStore = {
  get isAvailable() {
    return isAvailable;
  },
  get tests() {
    return tests;
  },
  get runningId() {
    return runningId;
  },
  get isCancelling() {
    return isCancelling;
  },
  log(id: string): string[] {
    return logs[id] ?? [];
  },
  result(id: string): ManualTestReport | undefined {
    return results[id];
  },
  failure(id: string): string | undefined {
    return failures[id];
  },

  async load() {
    if (isLoaded) return;
    isLoaded = true;
    try {
      isAvailable = await manualTestsApi.manualTestsAvailable();
      if (!isAvailable) return;
      tests = await manualTestsApi.listManualTests();
      await manualTestsApi.onManualTestLog(({ test_id, line }) => {
        (logs[test_id] ??= []).push(line);
      });
    } catch (error) {
      logError("Manual tests unavailable", error);
      isAvailable = false;
    }
  },

  async run(id: string, minutes: number | null) {
    if (runningId) return;
    runningId = id;
    isCancelling = false;
    logs[id] = [];
    delete results[id];
    delete failures[id];
    try {
      results[id] = await manualTestsApi.runManualTest(id, minutes);
    } catch (error) {
      failures[id] = errorMessage(error);
    } finally {
      runningId = null;
      isCancelling = false;
    }
  },

  async cancel() {
    if (!runningId) return;
    isCancelling = true;
    try {
      await manualTestsApi.cancelManualTest();
    } catch (error) {
      isCancelling = false;
      logError("Cancel failed", error);
    }
  },

  /** The backend's plain-text report, or the streamed log when the run never produced one. */
  report(id: string): string {
    const result = results[id];
    if (result) return result.report;
    const lines = [`Test: ${id}`, `Error: ${failures[id] ?? "no result"}`, "", "Log:", ...(logs[id] ?? [])];
    return lines.join("\n");
  },
};
