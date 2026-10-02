// One function per command in src-tauri/src/commands/manual_tests.rs. All but `manual_tests_available`
// exist only in a binary built with the `test-build` Cargo feature.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ManualTest, ManualTestReport, ManualTestLogEvent } from "$lib/types";

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
