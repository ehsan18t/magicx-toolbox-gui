import { browser } from "$app/environment";
import { STORAGE_KEYS } from "$lib/config/app";
import { duration } from "$lib/utils/motion";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

export type Theme = "light" | "dark";

const themeState = new PersistentStore<Theme>(STORAGE_KEYS.theme, "dark");
let transitionTimer: ReturnType<typeof setTimeout> | undefined;

function paint(theme: Theme) {
  document.documentElement.setAttribute("data-theme", theme);
}

function set(theme: Theme) {
  if (!browser) return;
  const root = document.documentElement;
  root.classList.add("theme-transitioning");
  themeState.value = theme;
  paint(theme);
  clearTimeout(transitionTimer);
  // The theme-fade overlay in app.css runs for the same token.
  transitionTimer = setTimeout(() => root.classList.remove("theme-transitioning"), duration("normal"));
}

export const themeStore = {
  get current() {
    return themeState.value;
  },

  get isDark() {
    return themeState.value === "dark";
  },

  init() {
    if (!browser) return;
    if (!themeState.hadStoredValue) {
      themeState.value = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    paint(themeState.value);
  },

  toggle() {
    set(themeState.value === "dark" ? "light" : "dark");
  },

  set,
};
