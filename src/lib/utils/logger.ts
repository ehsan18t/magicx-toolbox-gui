import * as logsApi from "$lib/api/logs";
import type { ForwardedLogLevel } from "$lib/types";

const DEDUPE_MS = 5000;
const RATE_WINDOW_MS = 1000;
const MAX_PER_WINDOW = 5;
const MAX_CHARS = 2000;
const DEDUPE_PRUNE_AT = 100;

const lastSent = new Map<string, number>();
let windowStart = 0;
let sentInWindow = 0;
let forwarding = false;

function describe(value: unknown): string {
  if (value instanceof Error) return value.stack ?? `${value.name}: ${value.message}`;
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value) ?? String(value);
  } catch {
    return String(value);
  }
}

/** De-duplicated and rate-limited; never throws and never rejects. */
function forwardToLog(level: ForwardedLogLevel, message: string): void {
  // A failure inside the forwarder must not be forwarded again.
  if (forwarding) return;
  forwarding = true;
  try {
    const now = Date.now();
    if ((lastSent.get(message) ?? -Infinity) > now - DEDUPE_MS) return;
    if (now - windowStart >= RATE_WINDOW_MS) {
      windowStart = now;
      sentInWindow = 0;
    }
    if (sentInWindow >= MAX_PER_WINDOW) return;
    sentInWindow++;
    if (lastSent.size > DEDUPE_PRUNE_AT) {
      for (const [key, at] of lastSent) if (at <= now - DEDUPE_MS) lastSent.delete(key);
    }
    lastSent.set(message, now);
    logsApi.logFrontend(level, message.slice(0, MAX_CHARS)).catch(() => {});
  } finally {
    forwarding = false;
  }
}

/** The console plus the session log. */
export function logError(context: string, error: unknown): void {
  console.error(`${context}:`, error);
  forwardToLog("error", `${context}: ${describe(error)}`);
}

export function logWarning(message: string): void {
  console.warn(message);
  forwardToLog("warn", message);
}

export function installErrorForwarding(): void {
  window.addEventListener("error", (e) =>
    forwardToLog("error", e.error ? describe(e.error) : `${e.message} (${e.filename}:${e.lineno}:${e.colno})`),
  );
  window.addEventListener("unhandledrejection", (e) =>
    forwardToLog("error", `Unhandled rejection: ${describe(e.reason)}`),
  );
}
