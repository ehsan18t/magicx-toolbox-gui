// API for app items. Every function maps 1:1 to a command in src-tauri/src/lib.rs's generate_handler!.
import { invoke } from "@tauri-apps/api/core";
import type { AppStatusView, AppView } from "../types";

export async function getApps(): Promise<AppView[]> {
  return await invoke<AppView[]>("get_apps");
}

/** One presence scan; ids with a removal or install in flight are omitted. */
export async function getAppStatuses(): Promise<AppStatusView[]> {
  return await invoke<AppStatusView[]>("get_app_statuses");
}

/** Removes the app, then re-probes: resolves only once presence reads Absent. */
export async function removeApp(appId: string): Promise<AppStatusView> {
  return await invoke<AppStatusView>("remove_app", { appId });
}

/** Installs through winget, then re-probes: resolves only once presence reads Installed. */
export async function installApp(appId: string): Promise<AppStatusView> {
  return await invoke<AppStatusView>("install_app", { appId });
}
