import { BACKEND_ERROR_CODES, type BackendErrorCode, type Level } from "$lib/types";
import { ELEVATE_HINT } from "$lib/utils/presentation";

/** Tauri rejects with the backend's serialized `{ code, message }` object, not an `Error`. */
export interface BackendError {
  code: BackendErrorCode;
  message: string;
}

function asBackendError(error: unknown): BackendError | null {
  const candidate = error as { code?: unknown; message?: unknown } | null | undefined;
  return (BACKEND_ERROR_CODES as readonly unknown[]).includes(candidate?.code) && typeof candidate?.message === "string"
    ? (candidate as BackendError)
    : null;
}

export function errorMessage(error: unknown): string {
  const message = typeof error === "string" ? error : (error as { message?: unknown } | null | undefined)?.message;
  return typeof message === "string" && message.trim() !== "" ? message : "An unexpected error occurred.";
}

/** The backend refused because the app is restarting, installing an update, or closing its window. */
export function isAppExiting(error: unknown): boolean {
  return asBackendError(error)?.code === "APP_EXITING";
}

/** The user cancelled an app install and the app reads absent afterwards. */
export function isAppCancelled(error: unknown): boolean {
  return asBackendError(error)?.code === "APP_CANCELLED";
}

/** The UAC prompt was declined or dismissed: nothing failed, the app keeps running as it was. */
export function isElevationDeclined(error: unknown): boolean {
  return asBackendError(error)?.code === "ELEVATION_DECLINED";
}

/** The app cannot replace its own exe, so the update must be downloaded by hand. */
export function isUpdateFolderReadOnly(error: unknown): boolean {
  return asBackendError(error)?.code === "UPDATE_FOLDER_READ_ONLY";
}

/** What the user can do about a failed apply or restore, or null when the message already covers it. */
export function tweakFailureAdvice(error: unknown, level: Level): string | null {
  switch (asBackendError(error)?.code) {
    case "TWEAK_ACCESS_DENIED":
      return level === "User" ? ELEVATE_HINT : null;
    case "TWEAK_ELEVATION_UNAVAILABLE":
      return "TrustedInstaller is unavailable on this PC right now.";
    case "TWEAK_OUTCOME_UNKNOWN":
      return "The change may have partly happened. Check this tweak's Needs Attention details.";
    case "TWEAK_BUSY":
      return "Windows reported it busy; try again in a moment.";
    default:
      return null;
  }
}
