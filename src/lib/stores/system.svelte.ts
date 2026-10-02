import * as systemApi from "$lib/api/system";
import { STORAGE_KEYS } from "$lib/config/app";
import type { CachedSystemInfo, SystemInfo, WindowsInfo } from "$lib/types";
import { logError } from "$lib/utils/logger";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

/** Stands in for the dynamic fields when only the cached hardware could be read. */
const UNKNOWN_WINDOWS: WindowsInfo = {
  version_string: "",
  display_version: "",
  build_number: "",
  product_name: "Windows",
  uptime_seconds: 0,
  is_windows_11: false,
  is_windows_server: false,
  install_date: null,
};

// Hardware rarely changes, so it is cached across launches.
const cache = new PersistentStore<CachedSystemInfo | null>(STORAGE_KEYS.systemInfoCache, null);

let info = $state<SystemInfo | null>(null);
let isLoading = $state(true);
let isRefreshing = $state(false);

const cachedAt = $derived(cache.value?.cachedAt ?? null);

function updateCache(fresh: SystemInfo): void {
  cache.value = {
    hardware: fresh.hardware,
    device: fresh.device,
    computer_name: fresh.computer_name,
    cachedAt: new Date().toISOString(),
  };
}

/** The cached hardware with the live dynamic fields, or placeholders when the live read failed. */
function withCachedHardware(cached: CachedSystemInfo, live?: SystemInfo): SystemInfo {
  return {
    windows: live?.windows ?? UNKNOWN_WINDOWS,
    username: live?.username ?? "",
    is_admin: live?.is_admin ?? false,
    hardware: cached.hardware,
    device: cached.device,
    computer_name: cached.computer_name,
  };
}

export const systemStore = {
  get info() {
    return info;
  },

  get isLoading() {
    return isLoading;
  },

  get isRefreshing() {
    return isRefreshing;
  },

  /** ISO timestamp of when hardware info was last cached. */
  get cachedAt() {
    return cachedAt;
  },

  /** Fresh dynamic info over the cached hardware; everything fresh when nothing is cached. */
  async load() {
    isLoading = true;
    try {
      const cached = cache.value;
      const fresh = await systemApi.getSystemInfo();
      if (cached) {
        info = withCachedHardware(cached, fresh);
      } else {
        info = fresh;
        updateCache(fresh);
      }
      return info;
    } catch (error) {
      logError("Failed to load system info", error);
      if (!cache.value) return null;
      info = withCachedHardware(cache.value);
      return info;
    } finally {
      isLoading = false;
    }
  },

  /** Re-reads everything, hardware included. */
  async refresh() {
    isRefreshing = true;
    try {
      const fresh = await systemApi.getSystemInfo();
      info = fresh;
      updateCache(fresh);
      return fresh;
    } catch (error) {
      logError("Failed to refresh system info", error);
      throw error;
    } finally {
      isRefreshing = false;
    }
  },
};
