// API functions for update-related Tauri commands
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "../types";

/**
 * Check for available updates
 */
export async function checkForUpdate(): Promise<UpdateInfo> {
  return await invoke<UpdateInfo>("check_for_update");
}

/**
 * Install a pending update
 */
export async function installUpdate(): Promise<void> {
  return await invoke("install_update");
}
