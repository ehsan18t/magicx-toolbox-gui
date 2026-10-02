import { logError } from "$lib/utils/logger";
import { openUrl } from "@tauri-apps/plugin-opener";

const EXTERNAL_SCHEME = /^(https?:\/\/|mailto:)/i;

/** Click handler: sends an http(s) or mailto `href` to the system handler instead of navigating the webview. */
export function openExternal(event: MouseEvent, href: string | null | undefined): void {
  const url = href?.trim() ?? "";
  if (!EXTERNAL_SCHEME.test(url)) return;
  event.preventDefault();
  openUrl(url).catch((error) => logError(`Failed to open external link ${url}`, error));
}
