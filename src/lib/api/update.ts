// One function per command in commands/update.rs.
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "../types";

export interface UpdateConfig {
  releasesApiUrl: string;
  /** regex_lite syntax. */
  assetPattern: string;
  includePrereleases: boolean;
}

export async function checkForUpdate(config: UpdateConfig): Promise<UpdateInfo> {
  return await invoke<UpdateInfo>("check_for_update", { config });
}

export async function installUpdate(downloadUrl: string, assetName: string, assetDigest: string | null): Promise<void> {
  await invoke("install_update", { downloadUrl, assetName, assetDigest });
}
