/**
 * Tweaks Data Store - Svelte 5 Runes
 *
 * Manages core data: system info, categories, tweaks with their statuses.
 * Supports progressive loading for better perceived performance.
 * System hardware info is cached in localStorage since it rarely changes.
 */

import * as api from "$lib/api/tweaks";
import type { CachedSystemInfo, CategoryDefinition, SystemInfo, TweakStatus, TweakWithStatus } from "$lib/types";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

// Storage key for cached hardware info
const SYSTEM_INFO_CACHE_KEY = "magicx-system-info-cache";

// Cached hardware info (static data that rarely changes)
const systemInfoCache = new PersistentStore<CachedSystemInfo | null>(SYSTEM_INFO_CACHE_KEY, null);

// === Loading States ===
let systemInfoLoading = $state(true);
let systemInfoRefreshing = $state(false);
let categoriesLoading = $state(true);
let tweaksLoading = $state(true);
let initialLoadComplete = $state(false);

// === System Info State ===
let systemInfo = $state<SystemInfo | null>(null);

// === Categories State ===
let categories = $state<CategoryDefinition[]>([]);

// === Tweaks State ===
let tweaks = $state<TweakWithStatus[]>([]);
let tweaksVersion = $state(0);

// Derived: tweaks grouped by category
const tweaksByCategory = $derived.by(() => {
  const byCategory: Record<string, TweakWithStatus[]> = {};

  // Initialize with all categories
  for (const cat of categories) {
    byCategory[cat.id] = [];
  }

  // Populate tweaks
  for (const tweak of tweaks) {
    const categoryId = tweak.definition.category_id;
    if (byCategory[categoryId]) {
      byCategory[categoryId].push(tweak);
    }
  }

  return byCategory;
});

// Derived: overall stats
const stats = $derived({
  total: tweaks.length,
  applied: tweaks.filter((t) => t.status.is_applied).length,
  pending: tweaks.filter((t) => !t.status.is_applied).length,
});

// Derived: stats per category
const categoryStats = $derived.by(() => {
  const result: Record<string, { total: number; applied: number }> = {};

  for (const cat of categories) {
    const catTweaks = tweaksByCategory[cat.id] || [];
    result[cat.id] = {
      total: catTweaks.length,
      applied: catTweaks.filter((t) => t.status.is_applied).length,
    };
  }

  return result;
});

// Derived: cache timestamp for display
const cacheTimestamp = $derived(systemInfoCache.value?.cachedAt ?? null);

// === Helper Functions ===

/**
 * Update the cache with hardware info from fresh system info
 */
function updateCache(info: SystemInfo): void {
  systemInfoCache.value = {
    hardware: info.hardware,
    device: info.device,
    computer_name: info.computer_name,
    cachedAt: new Date().toISOString(),
  };
}

/**
 * Build a SystemInfo object using cached hardware data and fresh dynamic data
 */
function buildSystemInfoFromCache(cache: CachedSystemInfo, freshInfo: SystemInfo): SystemInfo {
  return {
    // Use fresh dynamic data
    windows: freshInfo.windows,
    username: freshInfo.username,
    is_admin: freshInfo.is_admin,
    // Use cached static data
    hardware: cache.hardware,
    device: cache.device,
    computer_name: cache.computer_name,
  };
}

// === Exports ===

export const systemStore = {
  get info() {
    return systemInfo;
  },

  get isLoading() {
    return systemInfoLoading;
  },

  /** Whether a manual refresh is in progress */
  get isRefreshing() {
    return systemInfoRefreshing;
  },

  /** ISO timestamp of when hardware info was last cached */
  get cachedAt() {
    return cacheTimestamp;
  },

  /**
   * Load system info, using cache for hardware if available.
   * On first load with no cache, fetches everything fresh.
   * On subsequent loads, uses cached hardware + fresh dynamic info.
   */
  async load() {
    systemInfoLoading = true;
    try {
      const cached = systemInfoCache.value;

      // Always fetch fresh info (for dynamic data like uptime, is_admin)
      const freshInfo = await api.getSystemInfo();

      if (cached) {
        // Use cached hardware data, merge with fresh dynamic data
        systemInfo = buildSystemInfoFromCache(cached, freshInfo);
      } else {
        // No cache - use fresh data and create cache
        systemInfo = freshInfo;
        updateCache(freshInfo);
      }

      return systemInfo;
    } catch (error) {
      console.error("Failed to load system info:", error);
      // Try to use cached data as fallback
      const cached = systemInfoCache.value;
      if (cached) {
        // Build partial info from cache (dynamic fields will be empty/default)
        systemInfo = {
          windows: {
            version_string: "",
            display_version: "",
            build_number: "",
            product_name: "Windows",
            uptime_seconds: 0,
            is_windows_11: false,
            is_windows_server: false,
            install_date: null,
          },
          username: "",
          is_admin: false,
          hardware: cached.hardware,
          device: cached.device,
          computer_name: cached.computer_name,
        };
        return systemInfo;
      }
      return null;
    } finally {
      systemInfoLoading = false;
    }
  },

  /**
   * Force refresh all system info including hardware (ignores cache).
   * Use this when user wants to refresh hardware info.
   */
  async refresh() {
    systemInfoRefreshing = true;
    try {
      const freshInfo = await api.getSystemInfo();
      systemInfo = freshInfo;
      updateCache(freshInfo);
      return freshInfo;
    } catch (error) {
      console.error("Failed to refresh system info:", error);
      throw error;
    } finally {
      systemInfoRefreshing = false;
    }
  },

  /** Clear the cache (useful for debugging) */
  clearCache() {
    systemInfoCache.value = null;
  },
};

export const categoriesStore = {
  get list() {
    return categories;
  },

  get isLoading() {
    return categoriesLoading;
  },

  /** Get category by ID */
  getById(categoryId: string): CategoryDefinition | undefined {
    return categories.find((c) => c.id === categoryId);
  },

  /** Get category name by ID, returns the ID if not found */
  getName(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.name ?? categoryId;
  },

  /** Get category icon by ID, returns default folder icon if not found */
  getIcon(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.icon ?? "mdi:folder";
  },

  async load() {
    categoriesLoading = true;
    try {
      const result = await api.getCategories();
      categories = result;
      return result;
    } catch (error) {
      console.error("Failed to load categories:", error);
      // CRITICAL: Re-throw instead of returning empty array
      // Categories are compiled at build time and should ALWAYS load successfully
      // If this fails, it indicates a serious IPC or runtime error that must surface
      throw error;
    } finally {
      categoriesLoading = false;
    }
  },
};

export const tweaksStore = {
  get list() {
    return tweaks;
  },

  get byCategory() {
    return tweaksByCategory;
  },

  get stats() {
    return stats;
  },

  get isLoading() {
    return tweaksLoading;
  },

  async load() {
    tweaksLoading = true;
    try {
      const result = await api.getAllTweaksWithStatus();
      tweaks = result;
      tweaksVersion++;
      return result;
    } catch (error) {
      console.error("Failed to load tweaks:", error);
      // CRITICAL: Re-throw instead of returning empty array
      // If loading fails, the app cannot function properly - surface the error
      throw error;
    } finally {
      tweaksLoading = false;
    }
  },

  /** Update a single tweak's status */
  updateStatus(tweakId: string, status: Partial<TweakStatus>) {
    tweaks = tweaks.map((t) => (t.definition.id === tweakId ? { ...t, status: { ...t.status, ...status } } : t));
    tweaksVersion++;
  },

  /** Get a tweak by ID */
  getById(tweakId: string): TweakWithStatus | undefined {
    return tweaks.find((t) => t.definition.id === tweakId);
  },

  get version() {
    return tweaksVersion;
  },
};

/** Category stats getter - exposed separately for components that need it */
export const getCategoryStats = () => categoryStats;

/** Loading state store for progressive loading */
export const loadingStateStore = {
  get systemInfoLoading() {
    return systemInfoLoading;
  },
  get systemInfoRefreshing() {
    return systemInfoRefreshing;
  },
  get categoriesLoading() {
    return categoriesLoading;
  },
  get tweaksLoading() {
    return tweaksLoading;
  },
  get initialLoadComplete() {
    return initialLoadComplete;
  },
};

// Promise cache for deduplicating concurrent initialization calls
let quickInitPromise: Promise<void> | null = null;
let remainingDataPromise: Promise<void> | null = null;

/**
 * Quick initialize - only load categories for immediate UI display
 * Call loadRemainingData() after to load the rest
 *
 * Uses promise caching to prevent duplicate requests if called concurrently
 */
export async function initializeQuick(): Promise<void> {
  // Return existing promise if already loading
  if (quickInitPromise) {
    return quickInitPromise;
  }

  // Skip if categories already loaded
  if (categories.length > 0) {
    return;
  }

  quickInitPromise = categoriesStore
    .load()
    .then(() => {
      // Discard result to match Promise<void> signature
    })
    .finally(() => {
      quickInitPromise = null;
    });

  return quickInitPromise;
}

/**
 * Load remaining data after quick init
 *
 * Uses promise caching to prevent duplicate requests if called concurrently
 * (e.g., if both layout and page call this before the first call completes)
 */
export async function loadRemainingData(): Promise<void> {
  // Return existing promise if already loading
  if (remainingDataPromise) {
    return remainingDataPromise;
  }

  // Skip if already complete
  if (initialLoadComplete) {
    return;
  }

  remainingDataPromise = Promise.all([systemStore.load(), tweaksStore.load()])
    .then(() => {
      initialLoadComplete = true;
    })
    .finally(() => {
      remainingDataPromise = null;
    });

  return remainingDataPromise;
}
