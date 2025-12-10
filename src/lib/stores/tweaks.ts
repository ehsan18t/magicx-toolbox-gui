// Svelte store for tweak state management
import { derived, writable, type Readable } from "svelte/store";
import * as api from "../api/tweaks";
import type { SystemInfo, TweakCategory, TweakStatus, TweakWithStatus } from "../types";

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

// Create store instances
export const systemStore = createSystemStore();
export const tweaksStore = createTweaksStore();
export const loadingStore = createLoadingStore();
export const errorStore = createErrorStore();
export const pendingRebootStore = createPendingRebootStore();

// Selected category filter
export const selectedCategory = writable<TweakCategory | "all">("all");

// Search filter
export const searchQuery = writable<string>("");

// Derived stores
export const tweaksByCategory = derived(tweaksStore, ($tweaks) => {
  const byCategory: Record<TweakCategory, TweakWithStatus[]> = {
    privacy: [],
    performance: [],
    ui: [],
    security: [],
    services: [],
    gaming: [],
  };

  for (const tweak of $tweaks) {
    byCategory[tweak.definition.category].push(tweak);
  }

  return byCategory;
});

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

export const categoryStats = derived(tweaksByCategory, ($byCategory) => {
  const stats: Record<TweakCategory, { total: number; applied: number }> = {} as Record<
    TweakCategory,
    { total: number; applied: number }
  >;

  for (const [category, tweaks] of Object.entries($byCategory)) {
    stats[category as TweakCategory] = {
      total: tweaks.length,
      applied: tweaks.filter((t) => t.status.is_applied).length,
    };
  }

  return stats;
});

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

// Initialize all stores
export async function initializeStores(): Promise<void> {
  await Promise.all([systemStore.load(), tweaksStore.load()]);
}
