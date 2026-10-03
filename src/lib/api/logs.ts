// One function per command in src-tauri/src/commands/logging.rs.
import { invoke } from "@tauri-apps/api/core";
import type { ForwardedLogLevel, LogSettings, LogTail } from "$lib/types";

export async function getLogTail(since: number): Promise<LogTail> {
  return await invoke<LogTail>("get_log_tail", { since });
}

export async function logFrontend(level: ForwardedLogLevel, message: string): Promise<void> {
  await invoke("log_frontend", { level, message });
}

export async function getLogSettings(): Promise<LogSettings> {
  return await invoke<LogSettings>("get_log_settings");
}

/** Resolves to the effective state, which can differ from the request. */
export async function setLogSettings(persist: boolean, detailed: boolean): Promise<LogSettings> {
  return await invoke<LogSettings>("set_log_settings", { persist, detailed });
}

/** Rejects when a file could not be deleted; the current session file is restarted either way. */
export async function deleteLogs(): Promise<LogSettings> {
  return await invoke<LogSettings>("delete_logs");
}

/** The saved path, or null when the save dialog was cancelled. */
export async function exportDiagnostics(): Promise<string | null> {
  return await invoke<string | null>("export_diagnostics");
}

export async function revealLastExport(): Promise<void> {
  await invoke("reveal_last_export");
}

export async function openLogFolder(): Promise<void> {
  await invoke("open_log_folder");
}
