import { STORAGE_KEYS } from "$lib/config/app";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

// Each id has a `--swatch-<id>` colour in app.css.
export const COLOR_SCHEMES = [
  { id: "purple", name: "Purple" },
  { id: "blue", name: "Blue" },
  { id: "green", name: "Green" },
  { id: "orange", name: "Orange" },
  { id: "pink", name: "Pink" },
  { id: "red", name: "Red" },
  { id: "cyan", name: "Cyan" },
] as const;

export type ColorSchemeId = (typeof COLOR_SCHEMES)[number]["id"];

const DEFAULT_SCHEME: ColorSchemeId = "purple";

const isScheme = (id: unknown): id is ColorSchemeId => COLOR_SCHEMES.some((s) => s.id === id);

const schemeState = new PersistentStore<ColorSchemeId>(STORAGE_KEYS.colorScheme, DEFAULT_SCHEME, (stored) =>
  isScheme(stored) ? stored : undefined,
);

function paint(scheme: ColorSchemeId) {
  document.documentElement.setAttribute("data-scheme", scheme);
}

/** The accent, independent of light or dark. */
export const colorSchemeStore = {
  get current() {
    return schemeState.value;
  },

  init() {
    paint(schemeState.value);
  },

  set(scheme: ColorSchemeId) {
    if (!isScheme(scheme)) return;
    schemeState.value = scheme;
    paint(scheme);
  },
};
