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
      // System info failure is recoverable - return null and let UI handle gracefully
      // (Unlike categories which are critical for app structure)
      return null;
    } finally {
      systemInfoLoading = false;
    }
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
  get tweaksLoading() {
    return tweaksLoading;
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
