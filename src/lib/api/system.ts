// One function per command in src-tauri/src/commands/{system,elevation,general}.rs, plus the events lib.rs emits.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { SystemReading } from "$lib/types";
import { logError } from "$lib/utils/logger";

/** `machine` is null unless `withHardware`: that part is a slow WMI read. */
export async function getSystemInfo(withHardware: boolean): Promise<SystemReading> {
  return await invoke<SystemReading>("get_system_info", { withHardware });
}

/** On success this process exits, so only a failure returns. */
export async function restartAsAdmin(): Promise<void> {
  await invoke("restart_as_admin");
}

/** The colour the window paints behind the page, as when resizing. */
export async function setWindowBackground(dark: boolean): Promise<void> {
  await invoke("set_window_background", { dark });
}

/** Closing still goes through the backend's close-mid-apply refusal. */
export async function closeWindow(): Promise<void> {
  await getCurrentWindow().close();
}

/** For an error screen whose toasts may be gone: null once closed, else what to show in place. */
export async function closeWindowOrHint(): Promise<string | null> {
  try {
    await closeWindow();
    return null;
  } catch (error) {
    logError("Failed to close the window", error);
    return "The window could not close. Close it from the taskbar.";
  }
}

export async function minimizeWindow(): Promise<void> {
  await getCurrentWindow().minimize();
}

/** The capability grants maximize and unmaximize, not toggle-maximize. */
export async function setWindowMaximized(maximized: boolean): Promise<void> {
  const window = getCurrentWindow();
  await (maximized ? window.maximize() : window.unmaximize());
}

export async function isWindowMaximized(): Promise<boolean> {
  return await getCurrentWindow().isMaximized();
}

export async function onWindowResized(handler: () => void): Promise<UnlistenFn> {
  return await getCurrentWindow().onResized(handler);
}

/** The backend refuses to close mid-apply (a half-applied tweak has nothing to undo it); the payload says so. */
export async function onCloseBlocked(handler: (message: string) => void): Promise<UnlistenFn> {
  return await listen<string>("close-blocked", (event) => handler(event.payload));
}
