// Test build only: `available` stays false in a normal build, which hides the view.

import {
  cancelManualTest,
  listManualTests,
  manualTestsAvailable,
  onManualTestLog,
  runManualTest,
  type ManualTest,
  type ManualTestReport,
} from "$lib/api/manualTests";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";

let available = $state(false);
let tests = $state.raw<ManualTest[]>([]);
let runningId = $state<string | null>(null);
let cancelling = $state(false);
let logs = $state<Record<string, string[]>>({});
let results = $state<Record<string, ManualTestReport>>({});
let failures = $state<Record<string, string>>({});
let initialized = false;

export const manualTestsStore = {
  get isAvailable() {
    return available;
  },
  get tests() {
    return tests;
  },
  get runningId() {
    return runningId;
  },
  get isCancelling() {
    return cancelling;
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

  async init() {
    if (initialized) return;
    initialized = true;
    try {
      available = await manualTestsAvailable();
      if (!available) return;
      tests = await listManualTests();
      await onManualTestLog(({ test_id, line }) => {
        (logs[test_id] ??= []).push(line);
      });
    } catch (e) {
      logError("Manual tests unavailable", e);
      available = false;
    }
  },

  async run(id: string, minutes: number | null) {
    if (runningId) return;
    runningId = id;
    cancelling = false;
    logs[id] = [];
    delete results[id];
    delete failures[id];
    try {
      results[id] = await runManualTest(id, minutes);
    } catch (e) {
      failures[id] = errorMessage(e);
    } finally {
      runningId = null;
      cancelling = false;
    }
  },

  async cancel() {
    if (!runningId) return;
    cancelling = true;
    try {
      await cancelManualTest();
    } catch (e) {
      cancelling = false;
      logError("Cancel failed", e);
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
