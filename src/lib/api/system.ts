// One function per command in src-tauri/src/commands/{system,elevation,general}.rs, plus the events lib.rs emits.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SystemInfo } from "$lib/types";

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

/** The backend refuses to close mid-apply (a half-applied tweak has nothing to undo it); the payload says so. */
export async function onCloseBlocked(handler: (message: string) => void): Promise<UnlistenFn> {
  return await listen<string>("close-blocked", (event) => handler(event.payload));
}
