const cache = new Map<string, string>();

/** An app.css token as the root computes it; cached, as tokens do not change at runtime. */
export function token(name: string): string {
  let value = cache.get(name);
  if (value === undefined) {
    value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (value) cache.set(name, value);
  }
  return value;
}

/** A rem token in px. */
export function remToken(name: string): number {
  return parseFloat(token(name)) * parseFloat(getComputedStyle(document.documentElement).fontSize);
}
