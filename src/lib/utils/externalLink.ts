import { openExternalUrl } from "$lib/api/platform";
import { toastStore } from "$lib/stores/toast.svelte";

const EXTERNAL_SCHEME = /^(https?:\/\/|mailto:)/i;

/** Click handler: sends an http(s) or mailto `href` to the system handler instead of navigating the webview. */
export function openExternal(event: MouseEvent, href: string | null | undefined): void {
  const url = href?.trim() ?? "";
  if (!EXTERNAL_SCHEME.test(url)) return;
  event.preventDefault();
  openExternalUrl(url).catch((error) => toastStore.failure("Could not open link", error, { withContext: true }));
}
