import { STORAGE_KEYS } from "$lib/config/app";
import type { AppSettings } from "$lib/types";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

const DEFAULT_SETTINGS: AppSettings = {
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  lastUpdateCheck: null,
  showUnsupported: false,
  includePrereleases: false,
};

// A stored object can predate a key: the default fills it.
const settingsState = new PersistentStore<AppSettings>(STORAGE_KEYS.settings, DEFAULT_SETTINGS, (stored) =>
  typeof stored === "object" && stored !== null ? { ...DEFAULT_SETTINGS, ...stored } : undefined,
);

const autoCheckUpdates = $derived(settingsState.value.autoCheckUpdates);
const autoInstallUpdates = $derived(settingsState.value.autoInstallUpdates);
const lastUpdateCheck = $derived(settingsState.value.lastUpdateCheck);
const showUnsupported = $derived(settingsState.value.showUnsupported);
const includePrereleases = $derived(settingsState.value.includePrereleases);

function update(changes: Partial<AppSettings>) {
  settingsState.value = { ...settingsState.value, ...changes };
}

export const settingsStore = {
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

  get includePrereleases() {
    return includePrereleases;
  },

  setShowUnsupported(show: boolean) {
    update({ showUnsupported: show });
  },

  setIncludePrereleases(include: boolean) {
    update({ includePrereleases: include });
  },

  setAutoCheckUpdates(enabled: boolean) {
    update({ autoCheckUpdates: enabled });
  },

  setAutoInstallUpdates(enabled: boolean) {
    update({ autoInstallUpdates: enabled });
  },

  setLastUpdateCheck(date: string | null) {
    update({ lastUpdateCheck: date });
  },
};
