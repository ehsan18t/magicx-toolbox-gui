// The Tauri plugin calls the stores make: opener, dialog and path.
import { appDataDir, join } from "@tauri-apps/api/path";
import { open, save, type DialogFilter } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";

export async function openExternalUrl(url: string): Promise<void> {
  await openUrl(url);
}

/** The chosen folder, or null when cancelled. */
export async function pickFolder(title: string): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false, title });
  return typeof selected === "string" ? selected : null;
}

/** The chosen file, or null when cancelled. */
export async function pickFile(filters: DialogFilter[]): Promise<string | null> {
  const selected = await open({ multiple: false, filters });
  return typeof selected === "string" ? selected : null;
}

/** The chosen path, or null when cancelled. */
export async function pickSavePath(defaultPath: string, filters: DialogFilter[]): Promise<string | null> {
  return await save({ defaultPath, filters });
}

export async function appDataPath(...parts: string[]): Promise<string> {
  return await join(await appDataDir(), ...parts);
}

export async function joinPath(...parts: string[]): Promise<string> {
  return await join(...parts);
}
