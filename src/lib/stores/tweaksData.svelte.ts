/**
 * Tweaks Data Store - Svelte 5 Runes
 *
 * Manages core data: system info, categories, tweaks with their statuses.
 */

import * as api from "$lib/api/tweaks";
import type { CategoryDefinition, SystemInfo, TweakStatus, TweakWithStatus } from "$lib/types";

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

  async load() {
    try {
      const info = await api.getSystemInfo();
      systemInfo = info;
      return info;
    } catch (error) {
      console.error("Failed to load system info:", error);
      return null;
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

  async load() {
    try {
      const result = await api.getCategories();
      categories = result;
      return result;
    } catch (error) {
      console.error("Failed to load categories:", error);
      return [];
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

  async load() {
    try {
      const result = await api.getAllTweaksWithStatus();
      tweaks = result;
      return result;
    } catch (error) {
      console.error("Failed to load tweaks:", error);
      return [];
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

/** Initialize all data stores */
export async function initializeData(): Promise<void> {
  await Promise.all([systemStore.load(), categoriesStore.load(), tweaksStore.load()]);
}
