// App settings store with localStorage persistence using Svelte 5 runes

import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import type { AppSettings } from "../types";

const SETTINGS_STORAGE_KEY = "magicx-app-settings";

const defaultSettings: AppSettings = {
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  lastUpdateCheck: null,
  showUnsupported: false,
  includePrereleases: false,
};

// Persistent state
const settingsState = new PersistentStore<AppSettings>(SETTINGS_STORAGE_KEY, defaultSettings);

// Derived values for convenience
const autoCheckUpdates = $derived(settingsState.value.autoCheckUpdates);
const autoInstallUpdates = $derived(settingsState.value.autoInstallUpdates);
const lastUpdateCheck = $derived(settingsState.value.lastUpdateCheck);
// Settings stored before this key existed read as undefined, which keeps them hidden.
const showUnsupported = $derived(settingsState.value.showUnsupported === true);
const includePrereleases = $derived(settingsState.value.includePrereleases === true);

export const settingsStore = {
  get settings() {
    return settingsState.value;
  },

  get autoCheckUpdates() {
    return autoCheckUpdates;
  },

  get autoInstallUpdates() {
    return autoInstallUpdates;
  },

  get lastUpdateCheck() {
    return lastUpdateCheck;
  },

  get showUnsupported() {
    return showUnsupported;
  },

  setShowUnsupported(show: boolean) {
    settingsState.value = { ...settingsState.value, showUnsupported: show };
  },

  get includePrereleases() {
    return includePrereleases;
  },

  setIncludePrereleases(include: boolean) {
    settingsState.value = { ...settingsState.value, includePrereleases: include };
  },

  update(newSettings: Partial<AppSettings>) {
    settingsState.value = { ...settingsState.value, ...newSettings };
  },

  reset() {
    settingsState.value = { ...defaultSettings };
  },

  setAutoCheckUpdates(enabled: boolean) {
    this.update({ autoCheckUpdates: enabled });
  },

  setAutoInstallUpdates(enabled: boolean) {
    this.update({ autoInstallUpdates: enabled });
  },

  setLastUpdateCheck(date: string | null) {
    this.update({ lastUpdateCheck: date });
  },
};
