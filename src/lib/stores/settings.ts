// App settings store with localStorage persistence
import { get, writable } from "svelte/store";
import type { AppSettings } from "../types";

const SETTINGS_STORAGE_KEY = "magicx-app-settings";

const defaultSettings: AppSettings = {
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  checkUpdateInterval: 24, // hours
  lastUpdateCheck: null,
};

function loadSettings(): AppSettings {
  if (typeof localStorage === "undefined") {
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
  if (typeof localStorage === "undefined") {
    return;
  }

  try {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
  } catch (error) {
    console.error("Failed to save settings to localStorage:", error);
  }
}

function createSettingsStore() {
  const { subscribe, set, update } = writable<AppSettings>(loadSettings());

  return {
    subscribe,
    update(newSettings: Partial<AppSettings>) {
      update((current) => {
        const updated = { ...current, ...newSettings };
        saveSettings(updated);
        return updated;
      });
    },
    reset() {
      set(defaultSettings);
      saveSettings(defaultSettings);
    },
    get() {
      return get({ subscribe });
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
}

export const settingsStore = createSettingsStore();
