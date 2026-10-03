// `object`, not `{ disabled?: boolean }`: TypeScript rejects items with no `disabled` key against that weak type.
const isDisabled = (item: object) => "disabled" in item && item.disabled === true;

/** Next enabled index from `from` (exclusive) in `step` direction, wrapping; -1 when every item is disabled. */
export function nextEnabledIndex(items: readonly object[], from: number, step: 1 | -1): number {
  const len = items.length;
  for (let i = 1; i <= len; i++) {
    const idx = (((from + step * i) % len) + len) % len;
    if (!isDisabled(items[idx])) return idx;
  }
  return -1;
}

/** Where a radio group's arrow, Home or End key moves from `from`; null for any other key. */
export function radioKeyIndex(items: readonly object[], from: number, key: string): number | null {
  switch (key) {
    case "ArrowRight":
    case "ArrowDown":
      return nextEnabledIndex(items, from, 1);
    case "ArrowLeft":
    case "ArrowUp":
      return nextEnabledIndex(items, from, -1);
    case "Home":
      return nextEnabledIndex(items, -1, 1);
    case "End":
      return nextEnabledIndex(items, items.length, -1);
    default:
      return null;
  }
}
