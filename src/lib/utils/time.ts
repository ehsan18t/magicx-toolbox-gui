export const SECOND_MS = 1000;
const MINUTE_MS = 60 * SECOND_MS;
export const HOUR_MS = 60 * MINUTE_MS;

const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY = 24 * SECONDS_PER_HOUR;

export interface DateFormat {
  month?: "numeric" | "short" | "long";
  /** "seconds" matches a bare `toLocaleString()`. */
  time?: "minutes" | "seconds";
}

/** Empty for missing or zero input; an unparseable string comes back as given, never "Invalid Date" or 1970. */
export function formatDate(
  value: string | number | Date | null | undefined,
  { month = "numeric", time }: DateFormat = {},
): string {
  const date = value ? new Date(value) : null;
  if (!date || Number.isNaN(date.getTime())) return typeof value === "string" ? value : "";
  return date.toLocaleString(undefined, {
    year: "numeric",
    month,
    day: "numeric",
    ...(time && { hour: "numeric", minute: "2-digit", ...(time === "seconds" && { second: "2-digit" }) }),
  });
}

/** `m:ss` since `startedAt`. */
export function elapsedClock(startedAt: number, now: number): string {
  const seconds = Math.max(0, Math.floor((now - startedAt) / SECOND_MS));
  return `${Math.floor(seconds / SECONDS_PER_MINUTE)}:${String(seconds % SECONDS_PER_MINUTE).padStart(2, "0")}`;
}

/** The two largest units, e.g. "3d 4h" or "12m"; empty for zero or negative input. */
export function formatDuration(seconds: number): string {
  if (!(seconds > 0)) return "";
  const days = Math.floor(seconds / SECONDS_PER_DAY);
  const hours = Math.floor((seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR);
  const minutes = Math.floor((seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}
