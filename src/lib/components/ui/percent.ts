/** `value` as a 0–100 share of `max`; 0 for an empty range. */
export const toPercent = (value: number, max: number): number =>
  max > 0 ? Math.min(100, Math.max(0, (value / max) * 100)) : 0;
