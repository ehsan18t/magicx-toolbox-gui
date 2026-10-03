import { toastStore } from "$lib/stores/toast.svelte";
import { logError } from "./logger";

/** False, after an error toast, when the clipboard refuses. */
export async function copyText(text: string, failureMessage: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (error) {
    logError(failureMessage, error);
    toastStore.error(failureMessage);
    return false;
  }
}
