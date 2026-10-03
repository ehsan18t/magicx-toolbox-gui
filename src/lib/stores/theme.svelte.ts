import { setWindowBackground } from "$lib/api/system";
import { STORAGE_KEYS } from "$lib/config/app";
import { logError } from "$lib/utils/logger";
import { duration } from "$lib/utils/motion";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

export type Theme = "light" | "dark";

const isTheme = (value: unknown): value is Theme => value === "light" || value === "dark";

const themeState = new PersistentStore<Theme>(STORAGE_KEYS.theme, "dark", (stored) =>
  isTheme(stored) ? stored : undefined,
);
let transitionTimer: ReturnType<typeof setTimeout> | undefined;

function paint(theme: Theme) {
  document.documentElement.setAttribute("data-theme", theme);
  setWindowBackground(theme === "dark").catch((error) => logError("Failed to set the window background", error));
}

function set(theme: Theme) {
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

  init() {
    if (!themeState.restored) {
      themeState.value = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    paint(themeState.value);
  },

  toggle() {
    set(themeState.value === "dark" ? "light" : "dark");
  },

  set,
};
