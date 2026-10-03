export type ConfirmVariant = "default" | "warning" | "danger";

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: ConfirmVariant;
}

interface PendingConfirm extends ConfirmOptions {
  resolve: (ok: boolean) => void;
}

let current = $state<PendingConfirm | null>(null);
// Outlives `current`, so the dialog keeps its text through the exit animation.
let shown = $state<ConfirmOptions>({ title: "", message: "" });

function settle(ok: boolean) {
  const pending = current;
  current = null;
  pending?.resolve(ok);
}

/** One shared dialog for every row: per-row dialogs cost three component trees and window listeners each. */
export const confirmStore = {
  get current() {
    return current;
  },
  get shown() {
    return shown;
  },
  settle,
  /** Resolves to whether the user confirmed; a newer request cancels an open one. */
  ask(options: ConfirmOptions): Promise<boolean> {
    settle(false);
    shown = options;
    return new Promise((resolve) => (current = { ...options, resolve }));
  },
};
