// Theme store for dark/light mode management
// Using Svelte 5 runes for reactive state

import { browser } from "$app/environment";
import { APP_CONFIG } from "@/lib/config/app";

export type Theme = "light" | "dark";

// Reactive state
let currentTheme = $state<Theme>("dark");

function applyTheme(theme: Theme) {
  if (browser) {
    // Add transitioning class for smooth fade
    document.documentElement.classList.add("theme-transitioning");

    localStorage.setItem(APP_CONFIG.theme.storageKey, theme);
    document.documentElement.setAttribute("data-theme", theme);

    // Remove class after transition completes
    // Timeout matches the CSS animation duration (150ms) plus a small buffer
    setTimeout(() => {
      document.documentElement.classList.remove("theme-transitioning");
    }, 200);
  }
}

// Export the theme store with methods
export const themeStore = {
  get current() {
    return currentTheme;
  },

  get isDark() {
    return currentTheme === "dark";
  },

  init() {
    if (!browser) return;

    const stored = localStorage.getItem(APP_CONFIG.theme.storageKey) as Theme | null;
    const systemPrefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const initialTheme: Theme = stored || (systemPrefersDark ? "dark" : "light");

    currentTheme = initialTheme;
    document.documentElement.setAttribute("data-theme", initialTheme);
  },

  toggle() {
    const newTheme: Theme = currentTheme === "dark" ? "light" : "dark";
    currentTheme = newTheme;
    applyTheme(newTheme);
  },

  set(theme: Theme) {
    currentTheme = theme;
    applyTheme(theme);
  },
};
