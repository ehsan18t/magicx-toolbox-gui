import { type ClassValue, clsx } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";

// Mirrors the custom theme names in app.css; unknown, `text-ui` reads as a colour and is merged away.
export const twMergeConfig = {
  extend: {
    theme: {
      text: ["micro", "tiny", "caption", "code", "ui", "lead", "title", "hero"],
      tracking: ["display"],
      container: [
        "row",
        "overview",
        "dialog-sm",
        "dialog-md",
        "dialog-lg",
        "dialog-xl",
        "dialog-full",
        "row-control",
        "toast",
        "summary-panel",
      ],
      spacing: ["titlebar", "toast-offset", "toast-offset-banner", "dock-clearance", "rail"],
    },
    classGroups: {
      "grid-cols": [{ "grid-cols": ["row", "hardware", "category-progress", "main-aside", "cards"] }],
    },
  },
};

const twMerge = extendTailwindMerge(twMergeConfig);

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
