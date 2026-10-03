// The Tauri plugin calls the stores make: opener, dialog, path and process.
import { appDataDir, join } from "@tauri-apps/api/path";
import { open, save, type DialogFilter } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { exit } from "@tauri-apps/plugin-process";

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

export async function exitApp(code = 0): Promise<void> {
  await exit(code);
}
