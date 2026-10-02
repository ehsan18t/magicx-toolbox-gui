import { logFrontend } from "$lib/api/logs";

const DEDUPE_MS = 5000;
const MAX_PER_SECOND = 5;
const MAX_CHARS = 2000;

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
export function forwardToLog(level: "error" | "warn" | "info", message: string): void {
  // A failure inside the forwarder must not be forwarded again.
  if (forwarding) return;
  forwarding = true;
  try {
    const now = Date.now();
    if ((lastSent.get(message) ?? -Infinity) > now - DEDUPE_MS) return;
    if (now - windowStart >= 1000) {
      windowStart = now;
      sentInWindow = 0;
    }
    if (sentInWindow >= MAX_PER_SECOND) return;
    sentInWindow++;
    if (lastSent.size > 100) {
      for (const [key, at] of lastSent) if (at <= now - DEDUPE_MS) lastSent.delete(key);
    }
    lastSent.set(message, now);
    logFrontend(level, message.slice(0, MAX_CHARS)).catch(() => {});
  } finally {
    forwarding = false;
  }
}

/** Sends uncaught errors and unhandled rejections to the session log. */
export function installErrorForwarding(): void {
  window.addEventListener("error", (e) =>
    forwardToLog("error", e.error ? describe(e.error) : `${e.message} (${e.filename}:${e.lineno}:${e.colno})`),
  );
  window.addEventListener("unhandledrejection", (e) =>
    forwardToLog("error", `Unhandled rejection: ${describe(e.reason)}`),
  );
}
