/**
 * Toast Store - Svelte 5 Runes
 *
 * Manages toast notifications for the application.
 */

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
  tweakName?: string;
}

let toasts = $state<Toast[]>([]);
let idCounter = 0;

// Store timeout IDs so we can clear them when toasts are manually dismissed
// Note: Intentionally using plain Map since this is not rendered and doesn't need reactivity
/* eslint-disable svelte/prefer-svelte-reactivity -- Intentionally using a plain Map for timeout IDs.
    This Map is used only for internal timeout tracking and is not used in any reactive context,
    so Svelte reactivity is not needed or desired here. */
// Store timeout IDs so we can clear them when toasts are manually dismissed
const timeoutIds = new Map<string, ReturnType<typeof setTimeout>>();
/* eslint-enable svelte/prefer-svelte-reactivity */

function generateId(): string {
  return `toast-${++idCounter}-${Date.now()}`;
}

export const toastStore = {
  get list() {
    return toasts;
  },

  /**
   * Show a toast notification
   */
  show(type: ToastType, message: string, options?: { duration?: number; tweakName?: string }) {
    const id = generateId();
    const duration = options?.duration ?? (type === "error" ? 5000 : 3000);

    const toast: Toast = {
      id,
      type,
      message,
      duration,
      tweakName: options?.tweakName,
    };

    toasts = [...toasts, toast];

    // Auto-dismiss with proper cleanup
    if (duration > 0) {
      const timeoutId = setTimeout(() => {
        toastStore.dismiss(id);
      }, duration);
      timeoutIds.set(id, timeoutId);
    }

    return id;
  },

  /**
   * Dismiss a specific toast
   */
  dismiss(id: string) {
    // Clear the auto-dismiss timeout if it exists
    const timeoutId = timeoutIds.get(id);
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutIds.delete(id);
    }
    toasts = toasts.filter((t) => t.id !== id);
  },

  /**
   * Clear all toasts
   */
  clear() {
    // Clear all pending timeouts
    for (const timeoutId of timeoutIds.values()) {
      clearTimeout(timeoutId);
    }
    timeoutIds.clear();
    toasts = [];
  },

  // Convenience methods
  success(message: string, options?: { duration?: number; tweakName?: string }) {
    return this.show("success", message, options);
  },

  error(message: string, options?: { duration?: number; tweakName?: string }) {
    return this.show("error", message, options);
  },

  warning(message: string, options?: { duration?: number; tweakName?: string }) {
    return this.show("warning", message, options);
  },

  info(message: string, options?: { duration?: number; tweakName?: string }) {
    return this.show("info", message, options);
  },
};
