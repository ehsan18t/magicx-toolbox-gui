/** Next enabled index from `from` (exclusive) in `step` direction, wrapping; -1 when every item is disabled. */
export function nextEnabledIndex(items: readonly { disabled?: boolean }[], from: number, step: 1 | -1): number {
  const len = items.length;
  for (let i = 1; i <= len; i++) {
    const idx = (((from + step * i) % len) + len) % len;
    if (!items[idx].disabled) return idx;
  }
  return -1;
}
