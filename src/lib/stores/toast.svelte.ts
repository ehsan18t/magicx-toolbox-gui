import { errorMessage, isAppExiting } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";

export type ToastType = "success" | "error" | "warning" | "info";

export interface ToastAction {
  label: string;
  run: () => void;
}

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  /** Milliseconds; 0 stays until dismissed. */
  duration: number;
  /** The tweak or app the toast is about, shown as its title. */
  subject?: string;
  action?: ToastAction;
}

interface ToastOptions {
  duration?: number;
  subject?: string;
  action?: ToastAction;
}

interface FailureOptions extends ToastOptions {
  /** Prefix the message with the context, for a bare error that does not say what failed. */
  withContext?: boolean;
}

/** Milliseconds per severity; `long` for a toast with something to act on. */
export const TOAST_DURATION = { success: 3000, info: 3000, warning: 5000, error: 5000, long: 10000 } as const;

const MAX_TOASTS = 5;

interface Timer {
  remaining: number;
  startedAt: number;
  handle?: ReturnType<typeof setTimeout>;
}

let toasts = $state.raw<Toast[]>([]);
let announcement = $state.raw<{ id: string; text: string; assertive: boolean } | null>(null);
let idCounter = 0;
// eslint-disable-next-line svelte/prefer-svelte-reactivity -- timers, never rendered
const timers = new Map<string, Timer>();
let paused = false;
let heldId: string | null = null;

function start(id: string, timer: Timer) {
  timer.startedAt = Date.now();
  timer.handle = setTimeout(() => dismiss(id), timer.remaining);
}

function stop(timer: Timer) {
  if (timer.handle === undefined) return;
  clearTimeout(timer.handle);
  timer.handle = undefined;
  timer.remaining -= Date.now() - timer.startedAt;
}

function dismiss(id: string) {
  const timer = timers.get(id);
  if (timer) clearTimeout(timer.handle);
  timers.delete(id);
  toasts = toasts.filter((t) => t.id !== id);
}

function show(type: ToastType, message: string, options?: ToastOptions) {
  const id = `toast-${++idCounter}`;
  const duration = options?.duration ?? TOAST_DURATION[type];

  while (toasts.length >= MAX_TOASTS) dismiss((toasts.find((t) => t.id !== heldId) ?? toasts[0]).id);
  toasts = [...toasts, { id, type, message, duration, subject: options?.subject, action: options?.action }];
  const text = options?.subject ? `${options.subject}: ${message}` : message;
  announcement = { id, text, assertive: type === "error" };

  if (duration > 0) {
    const timer: Timer = { remaining: duration, startedAt: 0 };
    timers.set(id, timer);
    if (!paused) start(id, timer);
  }
  return id;
}

export const toastStore = {
  get list() {
    return toasts;
  },
  /** The newest toast's text, for the live regions: removals never re-announce an older one. */
  get announcement() {
    return announcement;
  },
  dismiss,
  /** Held while hovered or focused (WCAG 2.2.1); time left resumes on release. */
  setPaused(next: boolean) {
    if (next === paused) return;
    paused = next;
    for (const [id, timer] of timers) {
      if (next) stop(timer);
      else start(id, timer);
    }
  },
  /** The toast holding focus, kept when the stack overflows. */
  hold(id: string | null) {
    heldId = id;
  },
  success: (message: string, options?: ToastOptions) => show("success", message, options),
  error: (message: string, options?: ToastOptions) => show("error", message, options),
  warning: (message: string, options?: ToastOptions) => show("warning", message, options),
  info: (message: string, options?: ToastOptions) => show("info", message, options),
  /** Logs the failure, then toasts it: a warning when the app refused because it is exiting, as nothing ran. */
  failure(context: string, error: unknown, options?: FailureOptions) {
    logError(context, error);
    const message = errorMessage(error);
    return show(
      isAppExiting(error) ? "warning" : "error",
      options?.withContext ? `${context}: ${message}` : message,
      options,
    );
  },
};
