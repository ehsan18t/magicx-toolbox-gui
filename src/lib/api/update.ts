// API functions for update-related Tauri commands
import { invoke } from "@tauri-apps/api/core";

/**
 * Install a pending update
 */
export async function installUpdate(): Promise<void> {
  return await invoke("install_update");
}
