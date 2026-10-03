// One function per command in src-tauri/src/commands/update.rs.
import { Channel, invoke } from "@tauri-apps/api/core";
import type { DownloadProgress, UpdateConfig, UpdateInfo } from "$lib/types";

export async function checkForUpdate(config: UpdateConfig): Promise<UpdateInfo> {
  return await invoke<UpdateInfo>("check_for_update", { config });
}

export async function installUpdate(
  downloadUrl: string,
  assetName: string,
  assetDigest: string | null,
  assetSize: number | null,
  onProgress: (progress: DownloadProgress) => void,
): Promise<void> {
  const channel = new Channel<DownloadProgress>(onProgress);
  await invoke("install_update", { downloadUrl, assetName, assetDigest, assetSize, onProgress: channel });
}

/** Once the interface has mounted: the backend then deletes the exe the last update replaced. */
export async function frontendReady(): Promise<void> {
  await invoke("frontend_ready");
}

/** How to go back to the version the last update replaced, while it is still there. */
export async function getPreviousVersionHint(): Promise<string | null> {
  return await invoke<string | null>("get_previous_version_hint");
}
