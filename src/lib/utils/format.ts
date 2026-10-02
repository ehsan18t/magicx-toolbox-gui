const KIB = 1024;
const MIB = KIB * KIB;

export interface DateFormat {
  month?: "numeric" | "short" | "long";
  /** "seconds" matches a bare `toLocaleString()`. */
  time?: "minutes" | "seconds";
}

export function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** Empty for missing, zero or unparseable input, never "Invalid Date" or 1970. */
export function formatDate(
  value: string | number | Date | null | undefined,
  { month = "numeric", time }: DateFormat = {},
): string {
  const date = value ? new Date(value) : null;
  if (!date || Number.isNaN(date.getTime())) return "";
  return date.toLocaleString(undefined, {
    year: "numeric",
    month,
    day: "numeric",
    ...(time && { hour: "numeric", minute: "2-digit", ...(time === "seconds" && { second: "2-digit" }) }),
  });
}

/** Empty for negative or non-finite input. */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "";
  const kib = Math.ceil(bytes / KIB);
  return kib >= KIB ? `${(bytes / MIB).toFixed(1)} MB` : `${kib} KB`;
}
