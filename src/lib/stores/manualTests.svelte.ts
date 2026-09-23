/**
 * Manual Tests store (test build only). `available` stays false in a normal build, which hides the view.
 */

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

let available = $state(false);
let tests = $state<ManualTest[]>([]);
let runningId = $state<string | null>(null);
let cancelling = $state(false);
let logs = $state<Record<string, string[]>>({});
let results = $state<Record<string, ManualTestReport>>({});
let failures = $state<Record<string, string>>({});
let initialized = false;

export const manualTestsStore = {
  get available() {
    return available;
  },
  get tests() {
    return tests;
  },
  get runningId() {
    return runningId;
  },
  get cancelling() {
    return cancelling;
  },
  logFor(id: string): string[] {
    return logs[id] ?? [];
  },
  resultFor(id: string): ManualTestReport | undefined {
    return results[id];
  },
  failureFor(id: string): string | undefined {
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
      console.error("Manual tests unavailable:", e);
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
      console.error("Cancel failed:", e);
    }
  },

  /** The backend's plain-text report, or the streamed log when the run never produced one. */
  reportFor(id: string): string {
    const result = results[id];
    if (result) return result.report;
    const lines = [`Test: ${id}`, `Error: ${failures[id] ?? "no result"}`, "", "Log:", ...(logs[id] ?? [])];
    return lines.join("\n");
  },
};
