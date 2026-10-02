import { openUrl } from "@tauri-apps/plugin-opener";

const EXTERNAL_SCHEME = /^(https?:\/\/|mailto:)/i;

/** Hands `href` to the system handler when it is external; the caller then calls preventDefault. */
export function openExternal(href: string): boolean {
  if (!EXTERNAL_SCHEME.test(href)) return false;
  openUrl(href).catch((error) => console.error(`Failed to open external link: ${href}`, error));
  return true;
}
