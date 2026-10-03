/** Where focus goes when the control holding it is removed: the topmost open dialog, else the page heading. */
export function focusFallback(except?: Element | null): HTMLElement | null {
  const dialog = [...document.querySelectorAll<HTMLElement>('[aria-modal="true"]')]
    .filter((el) => el !== except)
    .at(-1);
  return dialog ?? document.querySelector<HTMLElement>("main h1");
}

/** Moves focus to `target` (else the fallback) only when it was lost: on the body, or on a removed or outgoing (inert) control. */
export function reclaimFocus(target?: HTMLElement | null): void {
  const el = document.activeElement;
  if (el && el !== document.body && el.isConnected && !el.closest("[inert]")) return;
  (target ?? focusFallback())?.focus({ preventScroll: true });
}
