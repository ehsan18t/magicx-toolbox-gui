// One function per command in commands/system.rs, elevation.rs and general.rs.
import { invoke } from "@tauri-apps/api/core";
import type { SystemInfo } from "../types";

export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke<SystemInfo>("get_system_info");
}

/** On success this process exits, so only a failure returns. */
export async function restartAsAdmin(): Promise<void> {
  await invoke("restart_as_admin");
}

export async function showMainWindow(): Promise<void> {
  await invoke("show_main_window");
}
