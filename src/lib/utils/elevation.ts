import { toastStore } from "$lib/stores/toast.svelte";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { invoke } from "@tauri-apps/api/core";

/** Relaunches elevated; on success this process exits, so only a failure returns. */
export async function restartAsAdmin(): Promise<void> {
  try {
    await invoke("restart_as_admin");
  } catch (error) {
    const message = errorMessage(error);
    console.error("Failed to restart as admin:", message);
    if (isAppExiting(error)) toastStore.warning(message);
    else toastStore.error(message);
  }
}
