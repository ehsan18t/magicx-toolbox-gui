/**
 * Color Scheme Store using Svelte 5 runes
 *
 * Manages the accent color scheme independently from light/dark theme.
 * Each scheme can be used with both light and dark modes.
 */

import { browser } from "$app/environment";

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

// Reactive state
let currentScheme = $state<ColorSchemeId>(DEFAULT_SCHEME);

// Derived value for the full scheme object
const currentSchemeInfo = $derived(COLOR_SCHEMES.find((s) => s.id === currentScheme) ?? COLOR_SCHEMES[0]);

export const colorSchemeStore = {
  get current() {
    return currentScheme;
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

    const stored = localStorage.getItem(STORAGE_KEY);
    const validScheme = COLOR_SCHEMES.find((s) => s.id === stored);
    const initialScheme = validScheme?.id ?? DEFAULT_SCHEME;

    currentScheme = initialScheme;
    document.documentElement.setAttribute("data-scheme", initialScheme);
  },

  /** Set a specific color scheme */
  setScheme(scheme: ColorSchemeId) {
    if (!COLOR_SCHEMES.find((s) => s.id === scheme)) return;

    currentScheme = scheme;
    if (browser) {
      localStorage.setItem(STORAGE_KEY, scheme);
      document.documentElement.setAttribute("data-scheme", scheme);
    }
  },
};
