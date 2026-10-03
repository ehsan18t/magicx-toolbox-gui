// WCAG: a control disabled with a reason stays focusable (aria-disabled, not native disabled) and is described by it.
export const blockedReason = (
  disabled: boolean | null | undefined,
  loading: boolean,
  reason: string | null | undefined,
): string | null => (disabled && !loading && reason) || null;

/** `busy` also sets aria-disabled, without a reason to describe. */
export const reasonAttrs = (reason: string | null | undefined, reasonId: string, busy = false) => ({
  "aria-disabled": busy || !!reason || undefined,
  "aria-describedby": reason ? reasonId : undefined,
});
