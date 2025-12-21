// App settings store with localStorage persistence using Svelte 5 runes

import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import type { AppSettings } from "../types";

const SETTINGS_STORAGE_KEY = "magicx-app-settings";

const defaultSettings: AppSettings = {
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  checkUpdateInterval: 24, // hours
  lastUpdateCheck: null,
};

// Persistent state
const settingsState = new PersistentStore<AppSettings>(SETTINGS_STORAGE_KEY, defaultSettings);

// Derived values for convenience
const autoCheckUpdates = $derived(settingsState.value.autoCheckUpdates);
const autoInstallUpdates = $derived(settingsState.value.autoInstallUpdates);
const checkUpdateInterval = $derived(settingsState.value.checkUpdateInterval);
const lastUpdateCheck = $derived(settingsState.value.lastUpdateCheck);

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

  get checkUpdateInterval() {
    return checkUpdateInterval;
  },

  get lastUpdateCheck() {
    return lastUpdateCheck;
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
