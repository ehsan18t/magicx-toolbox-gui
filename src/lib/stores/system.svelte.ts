import * as systemApi from "$lib/api/system";
import { STORAGE_KEYS } from "$lib/config/app";
import type { CachedSystemInfo, LiveSystemInfo, SystemInfo, SystemReading } from "$lib/types";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { composeSystemInfo, firstPaint, parseSystemCache, replacesCache } from "$lib/utils/systemCache";
import { toastStore } from "./toast.svelte";

// The hardware read is slow WMI: the last good one paints at once, and every load rereads it behind.
const cache = new PersistentStore<CachedSystemInfo | null>(STORAGE_KEYS.systemInfoCache, null, parseSystemCache);

let info = $state<SystemInfo | null>(null);
let isLoading = $state(true);
let isRefreshing = $state(false);
let loadError = $state<string | null>(null);

const cachedAt = $derived(cache.value?.cachedAt ?? null);

/** Shows a full reading and caches its hardware unless that would put a partial read over a complete one. */
function adoptFull(reading: SystemReading): SystemInfo {
  const machine = reading.machine;
  if (!machine) throw new Error("The system info read returned no hardware.");
  if (replacesCache(cache.value, machine)) cache.value = { ...machine, cachedAt: new Date().toISOString() };
  info = composeSystemInfo(reading.live, machine);
  return info;
}

/** Rereads everything, hardware included; a failure is toasted and keeps what is shown. Only a user's refresh mentions a partial read. */
async function reread(announcePartial: boolean): Promise<SystemInfo | null> {
  isRefreshing = true;
  try {
    const fresh = adoptFull(await systemApi.getSystemInfo(true));
    loadError = null;
    if (announcePartial && fresh.partial) toastStore.info("Some hardware details could not be read; see the log.");
    return fresh;
  } catch (error) {
    toastStore.failure("Could not refresh system info", error, { withContext: true });
    return null;
  } finally {
    isRefreshing = false;
  }
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

  /** The live fields over the cached hardware without waiting on WMI, then a full read behind it; with nothing cached, the full read is the load. */
  async load() {
    isLoading = true;
    loadError = null;
    const cached = cache.value;
    try {
      if (!cached) return adoptFull(await systemApi.getSystemInfo(true));
      let live: LiveSystemInfo | null = null;
      try {
        live = (await systemApi.getSystemInfo(false)).live;
      } catch (error) {
        logError("Failed to read live system info", error);
      }
      info = firstPaint(live, cached);
      void reread(false);
      return info;
    } catch (error) {
      logError("Failed to load system info", error);
      loadError = errorMessage(error);
      return null;
    } finally {
      isLoading = false;
    }
  },

  refresh: () => reread(true),
};
