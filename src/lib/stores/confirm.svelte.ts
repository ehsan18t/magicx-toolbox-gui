export interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: "default" | "warning" | "danger";
}

interface Request extends ConfirmOptions {
  resolve: (ok: boolean) => void;
}

let current = $state<Request | null>(null);
// Outlives `current`, so the dialog keeps its text through the exit animation.
let shown = $state<ConfirmOptions>({ title: "", message: "" });

export const confirmStore = {
  get current() {
    return current;
  },
  get shown() {
    return shown;
  },
  settle(ok: boolean) {
    const request = current;
    current = null;
    request?.resolve(ok);
  },
};

/** One shared dialog for every row: per-row dialogs cost three component trees and window listeners each. */
export function confirm(options: ConfirmOptions): Promise<boolean> {
  confirmStore.settle(false);
  shown = options;
  return new Promise((resolve) => (current = { ...options, resolve }));
}
