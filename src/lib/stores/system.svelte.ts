import * as systemApi from "$lib/api/system";
import { STORAGE_KEYS } from "$lib/config/app";
import type { CachedSystemInfo, SystemInfo, WindowsInfo } from "$lib/types";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { toastStore } from "./toast.svelte";

/** Stands in for the dynamic fields when only the cached hardware could be read. */
const UNKNOWN_WINDOWS: WindowsInfo = {
  version_string: "",
  display_version: "",
  build_number: "",
  product_name: "Windows",
  uptime_seconds: 0,
  is_windows_11: false,
  install_date: null,
};

const isObject = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null;

/** The shape systemInfoRows destructures: a cache from an older build would otherwise crash the render. */
function parseCache(stored: unknown): CachedSystemInfo | null | undefined {
  if (stored === null) return null;
  if (!isObject(stored) || !isObject(stored.hardware) || !isObject(stored.device)) return undefined;
  const { cpu, memory, motherboard, gpu, monitors, disks, network } = stored.hardware;
  const valid =
    typeof stored.computer_name === "string" &&
    typeof stored.cachedAt === "string" &&
    [cpu, memory, motherboard].every(isObject) &&
    [gpu, monitors, disks, network].every(Array.isArray);
  return valid ? (stored as CachedSystemInfo) : undefined;
}

// Hardware rarely changes, so it is cached across launches.
const cache = new PersistentStore<CachedSystemInfo | null>(STORAGE_KEYS.systemInfoCache, null, parseCache);

let info = $state<SystemInfo | null>(null);
let isLoading = $state(true);
let isRefreshing = $state(false);
let loadError = $state<string | null>(null);

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

  /** Why the first load failed with nothing cached to fall back on. */
  get loadError() {
    return loadError;
  },

  /** ISO timestamp of when hardware info was last cached. */
  get cachedAt() {
    return cachedAt;
  },

  /** Fresh dynamic info over the cached hardware; everything fresh when nothing is cached. */
  async load() {
    isLoading = true;
    loadError = null;
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
      if (!cache.value) {
        loadError = errorMessage(error);
        return null;
      }
      info = withCachedHardware(cache.value);
      return info;
    } finally {
      isLoading = false;
    }
  },

  /** Re-reads everything, hardware included; a failure is toasted and keeps what is shown. */
  async refresh() {
    isRefreshing = true;
    try {
      const fresh = await systemApi.getSystemInfo();
      info = fresh;
      loadError = null;
      updateCache(fresh);
      return fresh;
    } catch (error) {
      toastStore.failure("Could not refresh system info", error, { withContext: true });
      return null;
    } finally {
      isRefreshing = false;
    }
  },
};
