/**
 * Tweaks Data Store - Svelte 5 Runes
 *
 * Manages core data: system info, categories, tweaks with their statuses.
 * Supports progressive loading for better perceived performance.
 */

import * as api from "$lib/api/tweaks";
import type { CategoryDefinition, SystemInfo, TweakStatus, TweakWithStatus } from "$lib/types";

// === Loading States ===
let systemInfoLoading = $state(true);
let categoriesLoading = $state(true);
let tweaksLoading = $state(true);
let initialLoadComplete = $state(false);

// === System Info State ===
let systemInfo = $state<SystemInfo | null>(null);

// === Categories State ===
let categories = $state<CategoryDefinition[]>([]);

// Derived: categories lookup map
const categoriesMap = $derived.by(() => {
  const map: Record<string, CategoryDefinition> = {};
  for (const cat of categories) {
    map[cat.id] = cat;
  }
  return map;
});

// === Tweaks State ===
let tweaks = $state<TweakWithStatus[]>([]);

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

// === Exports ===

export const systemStore = {
  get info() {
    return systemInfo;
  },

  get isLoading() {
    return systemInfoLoading;
  },

  async load() {
    systemInfoLoading = true;
    try {
      const info = await api.getSystemInfo();
      systemInfo = info;
      return info;
    } catch (error) {
      console.error("Failed to load system info:", error);
      return null;
    } finally {
      systemInfoLoading = false;
    }
  },

  reset() {
    systemInfo = null;
  },
};

export const categoriesStore = {
  get list() {
    return categories;
  },

  get map() {
    return categoriesMap;
  },

  get isLoading() {
    return categoriesLoading;
  },

  async load() {
    categoriesLoading = true;
    try {
      const result = await api.getCategories();
      categories = result;
      return result;
    } catch (error) {
      console.error("Failed to load categories:", error);
      return [];
    } finally {
      categoriesLoading = false;
    }
  },

  reset() {
    categories = [];
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
      return result;
    } catch (error) {
      console.error("Failed to load tweaks:", error);
      return [];
    } finally {
      tweaksLoading = false;
    }
  },

  /** Update a single tweak's status */
  updateStatus(tweakId: string, status: Partial<TweakStatus>) {
    tweaks = tweaks.map((t) => (t.definition.id === tweakId ? { ...t, status: { ...t.status, ...status } } : t));
  },

  /** Get a tweak by ID */
  getById(tweakId: string): TweakWithStatus | undefined {
    return tweaks.find((t) => t.definition.id === tweakId);
  },

  reset() {
    tweaks = [];
  },
};

/** Category stats getter - exposed separately for components that need it */
export const getCategoryStats = () => categoryStats;

/** Loading state store for progressive loading */
export const loadingStateStore = {
  get systemInfoLoading() {
    return systemInfoLoading;
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
  /** True when we have enough data to show the app shell */
  get canShowApp() {
    return categories.length > 0;
  },
  /** True when all data is loaded */
  get isFullyLoaded() {
    return initialLoadComplete && !systemInfoLoading && !categoriesLoading && !tweaksLoading;
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
      // Convert CategoryDefinition[] to void
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
