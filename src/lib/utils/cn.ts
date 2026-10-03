import { type ClassValue, clsx } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";
import { createTV } from "tailwind-variants";

export type { VariantProps } from "tailwind-variants";

// Mirrors app.css's custom theme names (cn.test.ts enforces it); unknown ones merge wrongly, e.g. `text-ui` as a colour.
const twMergeConfig = {
  extend: {
    theme: {
      text: ["body", "badge-sm", "badge-md", "caption", "code", "ui", "lead", "title", "hero"],
      tracking: ["display"],
      container: ["item-row", "overview-split", "dialog-sm", "dialog-md", "dialog-lg", "dialog-full"],
      spacing: ["rail", "titlebar", "toast-offset", "toast-offset-banner", "dock-clearance"],
      shadow: ["flyout", "dialog"],
      animate: [
        "fade-in",
        "fade-out",
        "reveal",
        "rise-in",
        "pop-in",
        "modal-in",
        "modal-out",
        "highlight",
        "activity",
        "pulse",
      ],
      ease: ["overshoot"],
    },
    classGroups: {
      w: [{ w: ["toast", "summary-panel"] }],
      "max-w": [{ "max-w": ["item-row-control"] }],
      "grid-cols": [
        {
          "grid-cols": ["item-row", "label-value", "icon-label-value", "icon-label-meter-value", "main-aside", "cards"],
        },
      ],
      z: [{ z: ["raised", "dock", "scrim", "drawer", "modal", "busy", "toast", "popover", "theme-fade"] }],
      duration: [{ duration: ["fast", "normal", "slow", "slower", "highlight", "activity", "pulse"] }],
      delay: [{ delay: ["reveal", "settle", "tooltip", "feedback"] }],
    },
  },
};

const twMerge = extendTailwindMerge(twMergeConfig);

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

export const tv = createTV({ twMergeConfig });
