// Theme store for dark/light mode management
// Using Svelte 5 runes for reactive state

import { browser } from "$app/environment";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { APP_CONFIG } from "@/lib/config/app";

export type Theme = "light" | "dark";

// Persistent state
const themeState = new PersistentStore<Theme>(APP_CONFIG.theme.storageKey, "dark");

function applyTheme(theme: Theme) {
  if (browser) {
    // Add transitioning class for smooth fade
    document.documentElement.classList.add("theme-transitioning");

    themeState.value = theme;
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
    return themeState.value;
  },

  get isDark() {
    return themeState.value === "dark";
  },

  init() {
    if (!browser) return;

    // Check if we should use system preference (if nothing stored)
    // We check directly here because we want to override the default "dark" if necessary
    const stored = localStorage.getItem(APP_CONFIG.theme.storageKey);
    if (!stored) {
      const systemPrefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      themeState.value = systemPrefersDark ? "dark" : "light";
    }

    document.documentElement.setAttribute("data-theme", themeState.value);
  },

  toggle() {
    const newTheme: Theme = themeState.value === "dark" ? "light" : "dark";
    applyTheme(newTheme);
  },

  set(theme: Theme) {
    applyTheme(theme);
  },
};
