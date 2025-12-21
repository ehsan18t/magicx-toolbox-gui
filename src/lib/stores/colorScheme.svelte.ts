/**
 * Color Scheme Store using Svelte 5 runes
 *
 * Manages the accent color scheme independently from light/dark theme.
 * Each scheme can be used with both light and dark modes.
 */

import { browser } from "$app/environment";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

/** Available color schemes */
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

const STORAGE_KEY = "magicx-color-scheme";
const DEFAULT_SCHEME: ColorSchemeId = "purple";

// Persistent state
const schemeState = new PersistentStore<ColorSchemeId>(STORAGE_KEY, DEFAULT_SCHEME);

// Derived value for the full scheme object
const currentSchemeInfo = $derived(COLOR_SCHEMES.find((s) => s.id === schemeState.value) ?? COLOR_SCHEMES[0]);

export const colorSchemeStore = {
  get current() {
    return schemeState.value;
  },

  get info() {
    return currentSchemeInfo;
  },

  get schemes() {
    return COLOR_SCHEMES;
  },

  /** Initialize the store - load from localStorage and apply */
  init() {
    if (!browser) return;

    // Validate the loaded value from PersistentStore
    const loaded = schemeState.value;
    const isValid = COLOR_SCHEMES.some((s) => s.id === loaded);

    const finalScheme = isValid ? loaded : DEFAULT_SCHEME;

    // If invalid, reset it
    if (!isValid) {
      schemeState.value = finalScheme;
    }

    document.documentElement.setAttribute("data-scheme", finalScheme);
  },

  /** Set a specific color scheme */
  setScheme(scheme: ColorSchemeId) {
    if (!COLOR_SCHEMES.find((s) => s.id === scheme)) return;

    schemeState.value = scheme;
    if (browser) {
      document.documentElement.setAttribute("data-scheme", scheme);
    }
  },
};
