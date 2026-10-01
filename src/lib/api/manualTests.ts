// Manual Tests (test build only). Every command but `manual_tests_available` exists only in a
// binary built with the `test-build` Cargo feature.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type ManualTestStatus = "pass" | "fail" | "info";

export interface ManualTest {
  id: string;
  title: string;
  description: string;
  changes: string;
  changes_system: boolean;
  /** Default duration in minutes, for a test that runs over time. */
  minutes: number | null;
}

export interface ManualTestReport {
  test_id: string;
  status: ManualTestStatus;
  summary: string;
  details: string[];
  report: string;
}

export interface ManualTestLogEvent {
  test_id: string;
  line: string;
}

export async function manualTestsAvailable(): Promise<boolean> {
  return await invoke<boolean>("manual_tests_available");
}

export async function listManualTests(): Promise<ManualTest[]> {
  return await invoke<ManualTest[]>("list_manual_tests");
}

export async function runManualTest(testId: string, minutes: number | null): Promise<ManualTestReport> {
  return await invoke<ManualTestReport>("run_manual_test", { testId, minutes });
}

export async function cancelManualTest(): Promise<void> {
  await invoke("cancel_manual_test");
}

export async function onManualTestLog(handler: (event: ManualTestLogEvent) => void): Promise<UnlistenFn> {
  return await listen<ManualTestLogEvent>("manual-test-log", (event) => handler(event.payload));
}
