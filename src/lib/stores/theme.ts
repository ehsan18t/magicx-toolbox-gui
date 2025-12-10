import { browser } from "$app/environment";
import { APP_CONFIG } from "@/lib/config/app";
import { writable } from "svelte/store";

export type Theme = "light" | "dark";

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>("dark");

  return {
    subscribe,

    init: () => {
      if (!browser) return;

      const stored = localStorage.getItem(APP_CONFIG.theme.storageKey) as Theme | null;
      const systemPrefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const initialTheme: Theme = stored || (systemPrefersDark ? "dark" : "light");

      set(initialTheme);
      document.documentElement.setAttribute("data-theme", initialTheme);
    },

    toggle: () => {
      update((current) => {
        const newTheme: Theme = current === "dark" ? "light" : "dark";
        if (browser) {
          localStorage.setItem(APP_CONFIG.theme.storageKey, newTheme);
          document.documentElement.setAttribute("data-theme", newTheme);
        }
        return newTheme;
      });
    },

    set: (theme: Theme) => {
      set(theme);
      if (browser) {
        localStorage.setItem(APP_CONFIG.theme.storageKey, theme);
        document.documentElement.setAttribute("data-theme", theme);
      }
    },
  };
}

export const themeStore = createThemeStore();
