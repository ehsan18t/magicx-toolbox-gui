import type { BackendErrorCode, Level } from "$lib/types";
import { ELEVATE_HINT } from "$lib/utils/tweakPresentation";

/** Tauri rejects with the backend's serialized `{ code, message }` object, not an `Error`. */
export interface BackendError {
  code: BackendErrorCode;
  message: string;
}

export function asBackendError(error: unknown): BackendError | null {
  const candidate = error as Partial<BackendError> | null | undefined;
  return typeof candidate?.code === "string" && typeof candidate.message === "string"
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
