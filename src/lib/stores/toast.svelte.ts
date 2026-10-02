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

export const TOAST_DURATION = { default: 3000, error: 5000, long: 10000 } as const;

const MAX_TOASTS = 5;

let toasts = $state.raw<Toast[]>([]);
let idCounter = 0;
// eslint-disable-next-line svelte/prefer-svelte-reactivity -- timers, never rendered
const timeouts = new Map<string, ReturnType<typeof setTimeout>>();

function clearTimer(id: string) {
  clearTimeout(timeouts.get(id));
  timeouts.delete(id);
}

function dismiss(id: string) {
  clearTimer(id);
  toasts = toasts.filter((t) => t.id !== id);
}

function show(type: ToastType, message: string, options?: ToastOptions) {
  const id = `toast-${++idCounter}`;
  const duration = options?.duration ?? (type === "error" ? TOAST_DURATION.error : TOAST_DURATION.default);

  while (toasts.length >= MAX_TOASTS) {
    clearTimer(toasts[0].id);
    toasts = toasts.slice(1);
  }
  toasts = [...toasts, { id, type, message, duration, subject: options?.subject, action: options?.action }];

  if (duration > 0) {
    const timer = setTimeout(() => dismiss(id), duration);
    timeouts.set(id, timer);
  }
  return id;
}

export const toastStore = {
  get list() {
    return toasts;
  },
  dismiss,
  success: (message: string, options?: ToastOptions) => show("success", message, options),
  error: (message: string, options?: ToastOptions) => show("error", message, options),
  warning: (message: string, options?: ToastOptions) => show("warning", message, options),
  info: (message: string, options?: ToastOptions) => show("info", message, options),
};
