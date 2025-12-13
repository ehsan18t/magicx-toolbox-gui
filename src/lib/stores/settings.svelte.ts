// App settings store with localStorage persistence using Svelte 5 runes

import { browser } from "$app/environment";
import type { AppSettings } from "../types";

const SETTINGS_STORAGE_KEY = "magicx-app-settings";

const defaultSettings: AppSettings = {
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  checkUpdateInterval: 24, // hours
  lastUpdateCheck: null,
};

function loadSettings(): AppSettings {
  if (!browser) {
    return defaultSettings;
  }

  try {
    const stored = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (stored) {
      return { ...defaultSettings, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error("Failed to load settings from localStorage:", error);
  }

  return defaultSettings;
}

function saveSettings(settings: AppSettings): void {
  if (!browser) return;

  try {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
  } catch (error) {
    console.error("Failed to save settings to localStorage:", error);
  }
}

// Reactive state
let settings = $state<AppSettings>(loadSettings());

// Derived values for convenience
const autoCheckUpdates = $derived(settings.autoCheckUpdates);
const autoInstallUpdates = $derived(settings.autoInstallUpdates);
const checkUpdateInterval = $derived(settings.checkUpdateInterval);
const lastUpdateCheck = $derived(settings.lastUpdateCheck);

export const settingsStore = {
  get settings() {
    return settings;
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
    settings = { ...settings, ...newSettings };
    saveSettings(settings);
  },

  reset() {
    settings = { ...defaultSettings };
    saveSettings(settings);
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
