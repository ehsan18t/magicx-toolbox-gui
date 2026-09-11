/** Tauri rejects with the backend's serialized `{ code, message }` object, not an `Error`. */
export function errorMessage(error: unknown): string {
  const message = typeof error === "string" ? error : (error as { message?: unknown } | null | undefined)?.message;
  return typeof message === "string" && message.trim() !== "" ? message : "An unexpected error occurred.";
}

/** The backend refused because the app is restarting, installing an update, or closing its window. */
export function isAppExiting(error: unknown): boolean {
  return (error as { code?: string } | null | undefined)?.code === "APP_EXITING";
}
