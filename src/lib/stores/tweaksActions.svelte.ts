/**
 * Tweaks Actions Store - Svelte 5 Runes
 *
 * Action functions for applying, reverting, and managing tweaks.
 */

import * as api from "$lib/api/tweaks";
import type { PendingChange, TweakWithStatus } from "$lib/types";
import { toastStore } from "./toast.svelte";
import { tweaksStore } from "./tweaksData.svelte";
import { errorStore, loadingStore } from "./tweaksLoading.svelte";
import { pendingChangesStore, pendingRebootStore } from "./tweaksPending.svelte";

// === Search and Filter State ===
let searchQuery = $state<string>("");

// Derived: filtered tweaks based on search
const filteredTweaks = $derived.by(() => {
  let filtered = tweaksStore.list;

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

  get filteredTweaks() {
    return filteredTweaks;
  },

  get pendingRebootTweaks() {
    return pendingRebootTweaks;
  },

  setSearchQuery(query: string) {
    searchQuery = query;
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
  options?: { showToast?: boolean; tweakName?: string },
): Promise<boolean> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.applyTweak(tweakId, optionIndex);

    if (result.success) {
      // Query actual backup status (backend may skip if already at desired state)
      const actualHasBackup = await api.hasBackup(tweakId);

      tweaksStore.updateStatus(tweakId, {
        is_applied: optionIndex === 0,
        has_backup: actualHasBackup,
        current_option_index: optionIndex,
      });

      // Track if this tweak requires reboot
      if (result.requires_reboot || requiresReboot) {
        pendingRebootStore.add(tweakId);
        if (showToast) {
          toastStore.success("Applied (reboot required)", { tweakName });
        }
      } else if (showToast) {
        toastStore.success("Applied successfully", { tweakName });
      }

      return true;
    } else {
      // Apply failed - backend rolled back, extract detailed error if available
      const failureDetails =
        result.failures && result.failures.length > 0
          ? result.failures.map(([, msg]) => msg).join("; ")
          : result.message;

      errorStore.setError(tweakId, failureDetails);
      if (showToast) {
        toastStore.error(result.message, { tweakName });
      }
      return false;
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "Failed to apply tweak";
    errorStore.setError(tweakId, message);
    if (showToast) {
      toastStore.error(message, { tweakName });
    }
    return false;
  } finally {
    loadingStore.stop(tweakId);
  }
}

/**
 * Revert a tweak to its original state
 */
export async function revertTweak(
  tweakId: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<boolean> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);
  errorStore.clearError(tweakId);

  try {
    const result = await api.revertTweak(tweakId);

    if (result.success) {
      // Query actual status after revert to get the correct current_option_index
      // (snapshot restored original values, which could be any option)
      const actualStatus = await api.getTweakStatus(tweakId);

      tweaksStore.updateStatus(tweakId, {
        is_applied: false,
        has_backup: false,
        current_option_index: actualStatus.current_option_index,
      });

      // Clear any pending changes for this tweak
      pendingChangesStore.clear(tweakId);

      // Remove from pending reboot if it was there
      pendingRebootStore.remove(tweakId);

      // If reverting also requires reboot, add it back
      if (result.requires_reboot) {
        pendingRebootStore.add(tweakId);
        if (showToast) {
          toastStore.success("Reverted (reboot required)", { tweakName });
        }
      } else if (showToast) {
        toastStore.success("Reverted successfully", { tweakName });
      }

      return true;
    } else {
      // Partial failure - snapshot was KEPT for retry, tweak is still "applied"
      // Do NOT update is_applied to false - the snapshot still exists
      const failureDetails =
        result.failures && result.failures.length > 0
          ? result.failures.map(([, msg]) => msg).join("; ")
          : result.message;

      errorStore.setError(tweakId, failureDetails);

      if (showToast) {
        // Use warning instead of error for partial success
        toastStore.warning(`Partial revert: ${result.failures?.length ?? 0} operations failed`, {
          tweakName,
        });
      }
      return false;
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "Failed to revert tweak";
    errorStore.setError(tweakId, message);
    if (showToast) {
      toastStore.error(message, { tweakName });
    }
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
 * Apply all pending changes using batch API for efficiency
 */
export async function applyPendingChanges(): Promise<{ success: number; failed: number }> {
  const pending = pendingChangesStore.all;

  if (pending.size === 0) {
    return { success: 0, failed: 0 };
  }

  // Build operations array for batch API
  const operations: [string, number][] = [];
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Local variable, not reactive state
  const tweakMap = new Map<string, { change: PendingChange; tweak: TweakWithStatus }>();

  for (const [tweakId, change] of pending) {
    const tweak = tweaksStore.getById(tweakId);
    if (!tweak) continue;
    operations.push([change.tweakId, change.optionIndex]);
    tweakMap.set(tweakId, { change, tweak });
  }

  if (operations.length === 0) {
    return { success: 0, failed: 0 };
  }

  try {
    // Use batch API for single IPC call instead of N calls
    const result = await api.batchApplyTweaks(operations);

    // Count successes and failures from result
    const totalAttempted = operations.length;
    const failedCount = result.failures?.length ?? 0;
    const successCount = totalAttempted - failedCount;

    // Clear successful pending changes
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Local variable, not reactive state
    const failedIds = new Set(result.failures?.map(([id]) => id) ?? []);
    for (const tweakId of tweakMap.keys()) {
      if (!failedIds.has(tweakId)) {
        pendingChangesStore.clear(tweakId);

        // Track reboot requirement
        const entry = tweakMap.get(tweakId);
        if (entry?.tweak.definition.requires_reboot) {
          pendingRebootStore.add(tweakId);
        }

        // Update status in store
        tweaksStore.updateStatus(tweakId, {
          is_applied: entry?.change.optionIndex === 0,
          current_option_index: entry?.change.optionIndex,
          has_backup: true,
        });
      }
    }

    // Show summary toast
    if (failedCount === 0 && successCount > 0) {
      toastStore.success(`Applied ${successCount} tweak${successCount > 1 ? "s" : ""} successfully`);
    } else if (failedCount > 0 && successCount > 0) {
      toastStore.warning(`Applied ${successCount}, failed ${failedCount} tweak${failedCount > 1 ? "s" : ""}`);
    } else if (failedCount > 0) {
      toastStore.error(`Failed to apply ${failedCount} tweak${failedCount > 1 ? "s" : ""}`);
    }

    return { success: successCount, failed: failedCount };
  } catch (error) {
    console.error("Batch apply failed:", error);
    toastStore.error("Failed to apply pending changes");
    return { success: 0, failed: operations.length };
  }
}

/**
 * Batch revert multiple tweaks using batch API for efficiency
 */
export async function batchRevertTweaks(tweakIds: string[]): Promise<{ success: number; failed: number }> {
  if (tweakIds.length === 0) {
    return { success: 0, failed: 0 };
  }

  try {
    // Use batch API for single IPC call instead of N calls
    const result = await api.batchRevertTweaks(tweakIds);

    // Count successes and failures from result
    const totalAttempted = tweakIds.length;
    const failedCount = result.failures?.length ?? 0;
    const successCount = totalAttempted - failedCount;

    // Update status for successful reverts
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Local variable, not reactive state
    const failedIds = new Set(result.failures?.map(([id]) => id) ?? []);
    for (const tweakId of tweakIds) {
      if (!failedIds.has(tweakId)) {
        // Update status to reflect reverted state
        tweaksStore.updateStatus(tweakId, {
          is_applied: false,
          current_option_index: null,
          has_backup: false,
        });
        // Clear from pending reboot if it was there
        pendingRebootStore.remove(tweakId);
      }
    }

    // Show summary toast
    if (failedCount === 0 && successCount > 0) {
      toastStore.success(`Restored ${successCount} snapshot${successCount > 1 ? "s" : ""} successfully`);
    } else if (failedCount > 0 && successCount > 0) {
      toastStore.warning(`Restored ${successCount}, failed ${failedCount} snapshot${failedCount > 1 ? "s" : ""}`);
    } else if (failedCount > 0) {
      toastStore.error(`Failed to restore ${failedCount} snapshot${failedCount > 1 ? "s" : ""}`);
    }

    return { success: successCount, failed: failedCount };
  } catch (error) {
    console.error("Batch revert failed:", error);
    toastStore.error("Failed to restore snapshots");
    return { success: 0, failed: tweakIds.length };
  }
}
