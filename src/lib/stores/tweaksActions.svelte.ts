/**
 * Tweaks Actions Store - Svelte 5 Runes
 *
 * Action functions for applying, reverting, and managing tweaks.
 */

import * as api from "$lib/api/tweaks";
import type { PendingChange, TweakWithStatus } from "$lib/types";
import { tweaksStore } from "./tweaksData.svelte";
import { errorStore, loadingStore } from "./tweaksLoading.svelte";
import { pendingChangesStore, pendingRebootStore } from "./tweaksPending.svelte";

// === Search and Filter State ===
let searchQuery = $state<string>("");
let selectedCategory = $state<string>("all");

// Derived: filtered tweaks based on search and category
const filteredTweaks = $derived.by(() => {
  let filtered = tweaksStore.list;

  // Filter by category
  if (selectedCategory !== "all") {
    filtered = filtered.filter((t) => t.definition.category_id === selectedCategory);
  }

  // Filter by search query
  if (searchQuery.trim()) {
    const q = searchQuery.toLowerCase();
    filtered = filtered.filter(
      (t) =>
        t.definition.name.toLowerCase().includes(q) ||
        t.definition.description.toLowerCase().includes(q) ||
        t.definition.id.toLowerCase().includes(q),
    );
  }

  return filtered;
});

// Derived: tweaks that need reboot
const pendingRebootTweaks = $derived.by(() => {
  return tweaksStore.list.filter((t) => pendingRebootStore.needsReboot(t.definition.id));
});

// === Filter Store ===

export const filterStore = {
  get searchQuery() {
    return searchQuery;
  },

  get selectedCategory() {
    return selectedCategory;
  },

  get filteredTweaks() {
    return filteredTweaks;
  },

  get pendingRebootTweaks() {
    return pendingRebootTweaks;
  },

  setSearchQuery(query: string) {
    searchQuery = query;
  },

  setCategory(category: string) {
    selectedCategory = category;
  },

  clearFilters() {
    searchQuery = "";
    selectedCategory = "all";
  },
};

// === Actions ===

/**
 * Apply a tweak with a specific option
 */
export async function applyTweak(
  tweakId: string,
  optionIndex: number,
  requiresReboot: boolean = false,
): Promise<boolean> {
  loadingStore.start(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.applyTweak(tweakId, optionIndex);

    if (result.success) {
      tweaksStore.updateStatus(tweakId, {
        is_applied: true,
        has_backup: true,
        current_option_index: optionIndex,
      });

      // Track if this tweak requires reboot
      if (result.requires_reboot || requiresReboot) {
        pendingRebootStore.add(tweakId);
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
    loadingStore.stop(tweakId);
  }
}

/**
 * Revert a tweak to its original state
 */
export async function revertTweak(tweakId: string): Promise<boolean> {
  loadingStore.start(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.revertTweak(tweakId);

    if (result.success) {
      tweaksStore.updateStatus(tweakId, { is_applied: false });

      // Remove from pending reboot if it was there
      pendingRebootStore.remove(tweakId);

      // If reverting also requires reboot, add it back
      if (result.requires_reboot) {
        pendingRebootStore.add(tweakId);
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
    loadingStore.stop(tweakId);
  }
}

/**
 * Toggle a tweak on/off (for is_toggle tweaks)
 */
export async function toggleTweak(
  tweakId: string,
  currentlyApplied: boolean,
  tweak: TweakWithStatus,
): Promise<boolean> {
  if (currentlyApplied) {
    return revertTweak(tweakId);
  } else {
    // For toggles, option 0 is the "applied" state
    return applyTweak(tweakId, 0, tweak.definition.requires_reboot);
  }
}

/**
 * Stage a change (doesn't apply yet, just marks it pending)
 */
export function stageChange(tweakId: string, change: PendingChange): void {
  pendingChangesStore.set(tweakId, change);
}

/**
 * Clear a pending change
 */
export function unstageChange(tweakId: string): void {
  pendingChangesStore.clear(tweakId);
}

/**
 * Apply all pending changes
 */
export async function applyPendingChanges(): Promise<{ success: number; failed: number }> {
  const pending = pendingChangesStore.all;
  let success = 0;
  let failed = 0;

  for (const [tweakId, change] of pending) {
    const tweak = tweaksStore.getById(tweakId);
    if (!tweak) continue;

    const result = await applyTweak(change.tweakId, change.optionIndex, tweak.definition.requires_reboot);

    if (result) {
      success++;
      pendingChangesStore.clear(tweakId);
    } else {
      failed++;
    }
  }

  return { success, failed };
}
