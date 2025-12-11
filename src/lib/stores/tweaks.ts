// Svelte store for tweak state management
import { derived, get, writable, type Readable } from "svelte/store";
import * as api from "../api/tweaks";
import type {
  CategoryDefinition,
  PendingChange,
  SystemInfo,
  TweakStatus,
  TweakWithStatus,
} from "../types";

// System info store
function createSystemStore() {
  const { subscribe, set } = writable<SystemInfo | null>(null);
  return {
    subscribe,
    async load() {
      try {
        const info = await api.getSystemInfo();
        set(info);
        return info;
      } catch (error) {
        console.error("Failed to load system info:", error);
        return null;
      }
    },
    reset() {
      set(null);
    },
  };
}

// Categories store - dynamic categories loaded from backend
function createCategoriesStore() {
  const { subscribe, set } = writable<CategoryDefinition[]>([]);

  return {
    subscribe,
    async load() {
      try {
        const categories = await api.getCategories();
        set(categories);
        return categories;
      } catch (error) {
        console.error("Failed to load categories:", error);
        return [];
      }
    },
    reset() {
      set([]);
    },
  };
}

// Tweaks store
function createTweaksStore() {
  const { subscribe, set, update } = writable<TweakWithStatus[]>([]);

  return {
    subscribe,
    async load() {
      try {
        const tweaks = await api.getAllTweaksWithStatus();
        set(tweaks);
        return tweaks;
      } catch (error) {
        console.error("Failed to load tweaks:", error);
        return [];
      }
    },
    updateStatus(tweakId: string, status: Partial<TweakStatus>) {
      update((tweaks) =>
        tweaks.map((t) =>
          t.definition.id === tweakId ? { ...t, status: { ...t.status, ...status } } : t,
        ),
      );
    },
    reset() {
      set([]);
    },
  };
}

// Loading state store
function createLoadingStore() {
  const { subscribe, update } = writable<Set<string>>(new Set());
  return {
    subscribe,
    startLoading(id: string) {
      update((set) => {
        const newSet = new Set(set);
        newSet.add(id);
        return newSet;
      });
    },
    stopLoading(id: string) {
      update((set) => {
        const newSet = new Set(set);
        newSet.delete(id);
        return newSet;
      });
    },
    isLoading(id: string): Readable<boolean> {
      return derived({ subscribe }, ($set) => $set.has(id));
    },
  };
}

// Error store
function createErrorStore() {
  const { subscribe, set, update } = writable<Map<string, string>>(new Map());

  return {
    subscribe,
    setError(id: string, message: string) {
      update((map) => {
        const newMap = new Map(map);
        newMap.set(id, message);
        return newMap;
      });
    },
    clearError(id: string) {
      update((map) => {
        const newMap = new Map(map);
        newMap.delete(id);
        return newMap;
      });
    },
    clearAll() {
      set(new Map());
    },
  };
}

// Pending reboot store - tracks tweaks that need a reboot
function createPendingRebootStore() {
  const { subscribe, update, set } = writable<Set<string>>(new Set());

  return {
    subscribe,
    addTweak(tweakId: string) {
      update((s) => {
        const newSet = new Set(s);
        newSet.add(tweakId);
        return newSet;
      });
    },
    removeTweak(tweakId: string) {
      update((s) => {
        const newSet = new Set(s);
        newSet.delete(tweakId);
        return newSet;
      });
    },
    clear() {
      set(new Set());
    },
  };
}

// Pending changes store - tracks desired state before applying
function createPendingChangesStore() {
  const { subscribe, update, set } = writable<Map<string, PendingChange>>(new Map());

  return {
    subscribe,
    setPending(tweakId: string, change: PendingChange) {
      update((map) => {
        const newMap = new Map(map);
        newMap.set(tweakId, change);
        return newMap;
      });
    },
    clearPending(tweakId: string) {
      update((map) => {
        const newMap = new Map(map);
        newMap.delete(tweakId);
        return newMap;
      });
    },
    clearAll() {
      set(new Map());
    },
    clearCategory(categoryId: string, tweaks: TweakWithStatus[]) {
      // Clear pending changes only for tweaks in the specified category
      const categoryTweakIds = new Set(
        tweaks.filter((t) => t.definition.category === categoryId).map((t) => t.definition.id),
      );
      update((map) => {
        const newMap = new Map(map);
        for (const tweakId of categoryTweakIds) {
          newMap.delete(tweakId);
        }
        return newMap;
      });
    },
    getPending(tweakId: string): PendingChange | undefined {
      return get({ subscribe }).get(tweakId);
    },
  };
}

// Create store instances
export const systemStore = createSystemStore();
export const categoriesStore = createCategoriesStore();
export const tweaksStore = createTweaksStore();
export const loadingStore = createLoadingStore();
export const errorStore = createErrorStore();
export const pendingRebootStore = createPendingRebootStore();
export const pendingChangesStore = createPendingChangesStore();

// Selected category filter (now uses string instead of enum)
export const selectedCategory = writable<string>("all");

// Search filter
export const searchQuery = writable<string>("");

// Categories lookup map for quick access
export const categoriesMap = derived(categoriesStore, ($categories) => {
  const map: Record<string, CategoryDefinition> = {};
  for (const cat of $categories) {
    map[cat.id] = cat;
  }
  return map;
});

// Derived stores
export const tweaksByCategory = derived(
  [tweaksStore, categoriesStore],
  ([$tweaks, $categories]) => {
    const byCategory: Record<string, TweakWithStatus[]> = {};

    // Initialize with all categories
    for (const cat of $categories) {
      byCategory[cat.id] = [];
    }

    // Populate tweaks
    for (const tweak of $tweaks) {
      const categoryId = tweak.definition.category;
      if (byCategory[categoryId]) {
        byCategory[categoryId].push(tweak);
      }
    }

    return byCategory;
  },
);

export const filteredTweaks = derived(
  [tweaksStore, selectedCategory, searchQuery],
  ([$tweaks, $category, $query]) => {
    let filtered = $tweaks;

    // Filter by category
    if ($category !== "all") {
      filtered = filtered.filter((t) => t.definition.category === $category);
    }

    // Filter by search query
    if ($query.trim()) {
      const q = $query.toLowerCase();
      filtered = filtered.filter(
        (t) =>
          t.definition.name.toLowerCase().includes(q) ||
          t.definition.description.toLowerCase().includes(q) ||
          t.definition.id.toLowerCase().includes(q),
      );
    }

    return filtered;
  },
);

export const tweakStats = derived(tweaksStore, ($tweaks) => {
  const total = $tweaks.length;
  const applied = $tweaks.filter((t) => t.status.is_applied).length;
  const pending = total - applied;

  return { total, applied, pending };
});

export const categoryStats = derived(
  [tweaksByCategory, categoriesStore],
  ([$byCategory, $categories]) => {
    const stats: Record<string, { total: number; applied: number }> = {};

    for (const cat of $categories) {
      const tweaks = $byCategory[cat.id] || [];
      stats[cat.id] = {
        total: tweaks.length,
        applied: tweaks.filter((t) => t.status.is_applied).length,
      };
    }

    return stats;
  },
);

// Pending reboot derived stores
export const pendingRebootCount = derived(pendingRebootStore, ($pending) => $pending.size);

export const pendingRebootTweaks = derived(
  [tweaksStore, pendingRebootStore],
  ([$tweaks, $pending]) => {
    return $tweaks.filter((t) => $pending.has(t.definition.id));
  },
);

// Actions
export async function applyTweak(tweakId: string): Promise<boolean> {
  loadingStore.startLoading(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.applyTweak(tweakId);

    if (result.success) {
      tweaksStore.updateStatus(tweakId, { is_applied: true, has_backup: true });

      // Track if this tweak requires reboot
      if (result.requires_reboot) {
        pendingRebootStore.addTweak(tweakId);
      }

      return true;
    } else {
      errorStore.setError(tweakId, result.message);
      return false;
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "Failed to apply tweak";
    errorStore.setError(tweakId, message);
    return false;
  } finally {
    loadingStore.stopLoading(tweakId);
  }
}

export async function revertTweak(tweakId: string): Promise<boolean> {
  loadingStore.startLoading(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.revertTweak(tweakId);

    if (result.success) {
      tweaksStore.updateStatus(tweakId, { is_applied: false });

      // Remove from pending reboot if it was there
      pendingRebootStore.removeTweak(tweakId);

      // If reverting also requires reboot, add it back
      if (result.requires_reboot) {
        pendingRebootStore.addTweak(tweakId);
      }

      return true;
    } else {
      errorStore.setError(tweakId, result.message);
      return false;
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "Failed to revert tweak";
    errorStore.setError(tweakId, message);
    return false;
  } finally {
    loadingStore.stopLoading(tweakId);
  }
}

export async function toggleTweak(tweakId: string, currentlyApplied: boolean): Promise<boolean> {
  if (currentlyApplied) {
    return revertTweak(tweakId);
  } else {
    return applyTweak(tweakId);
  }
}

export async function applyTweakOption(
  tweakId: string,
  optionIndex: number,
  requiresReboot: boolean = false,
): Promise<boolean> {
  loadingStore.startLoading(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.applyTweakOption(tweakId, optionIndex);

    if (result.success) {
      tweaksStore.updateStatus(tweakId, {
        is_applied: true,
        has_backup: true,
        current_option_index: optionIndex,
      });

      if (result.requires_reboot || requiresReboot) {
        pendingRebootStore.addTweak(tweakId);
      }

      return true;
    } else {
      errorStore.setError(tweakId, result.message);
      return false;
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "Failed to apply tweak option";
    errorStore.setError(tweakId, message);
    return false;
  } finally {
    loadingStore.stopLoading(tweakId);
  }
}

// Stage a change (doesn't apply yet, just marks it pending)
export function stageChange(tweakId: string, change: PendingChange): void {
  pendingChangesStore.setPending(tweakId, change);
}

// Clear a pending change
export function unstageChange(tweakId: string): void {
  pendingChangesStore.clearPending(tweakId);
}

// Apply all pending changes
export async function applyPendingChanges(): Promise<{ success: number; failed: number }> {
  const pending = get(pendingChangesStore);
  const tweaks = get(tweaksStore);
  let success = 0;
  let failed = 0;

  for (const [tweakId, change] of pending) {
    const tweak = tweaks.find((t) => t.definition.id === tweakId);
    if (!tweak) continue;

    let result = false;
    if (change.type === "binary") {
      if (change.enabled) {
        result = await applyTweak(tweakId);
      } else {
        result = await revertTweak(tweakId);
      }
    } else if (change.type === "multistate") {
      result = await applyTweakOption(
        tweakId,
        change.optionIndex,
        tweak.definition.requires_reboot,
      );
    }

    if (result) {
      success++;
      pendingChangesStore.clearPending(tweakId);
    } else {
      failed++;
    }
  }

  return { success, failed };
}

// Derived: count of pending changes
export const pendingChangesCount = derived(pendingChangesStore, ($pending) => $pending.size);

// Derived: pending changes for a specific category
export function getPendingForCategory(categoryId: string): Readable<Map<string, PendingChange>> {
  return derived([pendingChangesStore, tweaksStore], ([$pending, $tweaks]) => {
    const result = new Map<string, PendingChange>();
    for (const [tweakId, change] of $pending) {
      const tweak = $tweaks.find((t) => t.definition.id === tweakId);
      if (tweak?.definition.category === categoryId) {
        result.set(tweakId, change);
      }
    }
    return result;
  });
}

// Initialize all stores
export async function initializeStores(): Promise<void> {
  await Promise.all([systemStore.load(), categoriesStore.load(), tweaksStore.load()]);
}
