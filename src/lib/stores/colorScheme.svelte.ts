import { browser } from "$app/environment";
import { STORAGE_KEYS } from "$lib/config/app";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

// app.html mirrors these colours to paint the saved accent before the bundle loads.
export const COLOR_SCHEMES = [
  { id: "purple", name: "Purple", color: "#8b5cf6" },
  { id: "blue", name: "Blue", color: "#3b82f6" },
  { id: "green", name: "Green", color: "#10b981" },
  { id: "orange", name: "Orange", color: "#f97316" },
  { id: "pink", name: "Pink", color: "#ec4899" },
  { id: "red", name: "Red", color: "#ef4444" },
  { id: "cyan", name: "Cyan", color: "#06b6d4" },
] as const;

export type ColorSchemeId = (typeof COLOR_SCHEMES)[number]["id"];

const DEFAULT_SCHEME: ColorSchemeId = "purple";

const schemeState = new PersistentStore<ColorSchemeId>(STORAGE_KEYS.colorScheme, DEFAULT_SCHEME);

const isScheme = (id: string): id is ColorSchemeId => COLOR_SCHEMES.some((s) => s.id === id);

function paint(scheme: ColorSchemeId) {
  document.documentElement.setAttribute("data-scheme", scheme);
}

/** The accent, independent of light or dark. */
export const colorSchemeStore = {
  get current() {
    return schemeState.value;
  },

  init() {
    if (!browser) return;
    if (!isScheme(schemeState.value)) schemeState.value = DEFAULT_SCHEME;
    paint(schemeState.value);
  },

  set(scheme: ColorSchemeId) {
    if (!isScheme(scheme)) return;
    schemeState.value = scheme;
    if (browser) paint(scheme);
  },
};
