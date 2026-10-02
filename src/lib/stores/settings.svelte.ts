import { STORAGE_KEYS } from "$lib/config/app";
import type { AppSettings } from "$lib/types";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

const defaultSettings: AppSettings = {
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  lastUpdateCheck: null,
  showUnsupported: false,
  includePrereleases: false,
};

const settingsState = new PersistentStore<AppSettings>(STORAGE_KEYS.settings, defaultSettings);

const autoCheckUpdates = $derived(settingsState.value.autoCheckUpdates);
const autoInstallUpdates = $derived(settingsState.value.autoInstallUpdates);
const lastUpdateCheck = $derived(settingsState.value.lastUpdateCheck);
// A stored object may lack these keys; undefined must read as false.
const showUnsupported = $derived(settingsState.value.showUnsupported === true);
const includePrereleases = $derived(settingsState.value.includePrereleases === true);

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
