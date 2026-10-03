import { logError } from "./logger";

/** False, logged, when the clipboard refuses; the caller says so. */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (error) {
    logError("Clipboard write failed", error);
    return false;
  }
}
