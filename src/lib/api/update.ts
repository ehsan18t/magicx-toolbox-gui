// One function per command in src-tauri/src/commands/update.rs.
import { invoke } from "@tauri-apps/api/core";
import type { UpdateConfig, UpdateInfo } from "$lib/types";

export async function checkForUpdate(config: UpdateConfig): Promise<UpdateInfo> {
  return await invoke<UpdateInfo>("check_for_update", { config });
}

export async function installUpdate(downloadUrl: string, assetName: string, assetDigest: string | null): Promise<void> {
  await invoke("install_update", { downloadUrl, assetName, assetDigest });
}
