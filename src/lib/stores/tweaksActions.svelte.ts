/**
 * Tweaks Actions Store - Svelte 5 Runes
 *
 * Action functions for the redesigned engine: apply BY LABEL, restore (single
 * head-walk), and discard snapshot entries. Batch flows are client-side loops over
 * the per-tweak commands (there is no backend batch command).
 */

import * as api from "$lib/api/tweaks";
import type { PendingChange } from "$lib/types";
import { toastStore } from "./toast.svelte";
import { tweaksStore } from "./tweaksData.svelte";
import { errorStore, loadingStore } from "./tweaksLoading.svelte";
import { pendingChangesStore, pendingRebootStore } from "./tweaksPending.svelte";

// === Search and Filter State ===
let searchQuery = $state<string>("");

// Derived: filtered tweaks based on search
const filteredTweaks = $derived.by(() => {
  let filtered = tweaksStore.list;
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
 * Apply a tweak's option by LABEL. The command returns the fresh post-op status,
 * which we adopt directly (no re-fetch / no re-scan).
 */
export async function applyTweak(
  tweakId: string,
  optionLabel: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<boolean> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);
  errorStore.clearError(tweakId);

  try {
    const outcome = await api.applyTweak(tweakId, optionLabel);
    tweaksStore.setStatusView(tweakId, outcome.status);
    pendingChangesStore.clear(tweakId);

    if (showToast) {
      toastStore.success("Applied successfully", { tweakName });
    }
    return true;
  } catch (error) {
    // Tauri v2 rejects with the raw serialized Error object ({code, message}), not a JS
    // Error instance — read `.message` off the object, fall back to a stringified form.
    const message = (error as { message?: string })?.message ?? String(error);
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
 * Restore a tweak from its most recent snapshot (single head-walk). A restore that
 * cannot fully complete surfaces as an error (ADR-0001): the snapshot is kept and the
 * tweak is marked Needs Attention rather than silently reporting success.
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
    const outcome = await api.restoreTweak(tweakId);
    tweaksStore.setStatusView(tweakId, outcome.status);
    pendingChangesStore.clear(tweakId);

    if (outcome.reboot_advisory) {
      pendingRebootStore.add(tweakId);
    } else {
      pendingRebootStore.remove(tweakId);
    }

    if (showToast) {
      toastStore.success(outcome.reboot_advisory ? "Restored (reboot advised)" : "Restored successfully", {
        tweakName,
      });
    }
    return true;
  } catch (error) {
    // Tauri v2 rejects with the raw serialized Error object ({code, message}) — carrying the
    // real per-resource RollbackReport text — not a JS Error instance. Read it off the object.
    const message = (error as { message?: string })?.message ?? String(error);
    errorStore.setError(tweakId, message);

    // Needs Attention (ADR-0001): only meaningful while a snapshot still exists to retry from.
    if (tweaksStore.getById(tweakId)?.status.has_backup) {
      tweaksStore.patchStatus(tweakId, {
        needs_attention: true,
        unrestorable_resources: [message],
      });
    }

    if (showToast) {
      toastStore.warning(`Restore needs attention: ${message}`, { tweakName });
    }
    return false;
  } finally {
    loadingStore.stop(tweakId);
  }
}

/**
 * Explicit-consent snapshot release (ADR-0002): discard every snapshot entry for the
 * tweak. Replaces the old keep_current_state — the way out of Needs Attention when the
 * user accepts the current state.
 */
export async function discardSnapshots(
  tweakId: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<boolean> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);
  try {
    const entries = await api.listSnapshotEntries(tweakId);
    for (const entry of entries) {
      await api.discardSnapshotEntry(tweakId, entry.seq);
    }

    tweaksStore.patchStatus(tweakId, {
      has_backup: false,
      needs_attention: false,
      unrestorable_resources: [],
    });
    errorStore.clearError(tweakId);
    pendingRebootStore.remove(tweakId);

    if (showToast) {
      toastStore.success("Snapshot discarded", { tweakName });
    }
    return true;
  } catch (error) {
    const message = (error as { message?: string })?.message ?? String(error);
    errorStore.setError(tweakId, message);
    if (showToast) {
      toastStore.error(message, { tweakName });
    }
    return false;
  } finally {
    loadingStore.stop(tweakId);
  }
}

/** Stage a change (doesn't apply yet, just marks it pending) */
export function stageChange(tweakId: string, change: PendingChange): void {
  pendingChangesStore.set(tweakId, change);
}

/** Clear a pending change */
export function unstageChange(tweakId: string): void {
  pendingChangesStore.clear(tweakId);
}

/**
 * Apply all pending changes as a client-side sequential loop over apply_tweak
 * (no backend batch command exists). Per-tweak results are surfaced via the loop.
 */
export async function applyPendingChanges(): Promise<{ success: number; failed: number }> {
  const changes = Array.from(pendingChangesStore.all.values());
  if (changes.length === 0) {
    return { success: 0, failed: 0 };
  }

  let success = 0;
  let failed = 0;
  for (const change of changes) {
    const tweakName = tweaksStore.getById(change.tweakId)?.definition.name;
    const ok = await applyTweak(change.tweakId, change.optionLabel, { showToast: false, tweakName });
    if (ok) success++;
    else failed++;
  }

  if (failed === 0 && success > 0) {
    toastStore.success(`Applied ${success} tweak${success > 1 ? "s" : ""} successfully`);
  } else if (failed > 0 && success > 0) {
    toastStore.warning(`Applied ${success}, failed ${failed} tweak${failed > 1 ? "s" : ""}`);
  } else if (failed > 0) {
    toastStore.error(`Failed to apply ${failed} tweak${failed > 1 ? "s" : ""}`);
  }

  return { success, failed };
}

/**
 * Restore multiple tweaks as a client-side sequential loop over restore_tweak
 * (no backend batch command exists).
 */
export async function batchRevertTweaks(tweakIds: string[]): Promise<{ success: number; failed: number }> {
  if (tweakIds.length === 0) {
    return { success: 0, failed: 0 };
  }

  let success = 0;
  let failed = 0;
  for (const tweakId of tweakIds) {
    const ok = await revertTweak(tweakId, { showToast: false });
    if (ok) success++;
    else failed++;
  }

  if (failed === 0 && success > 0) {
    toastStore.success(`Restored ${success} snapshot${success > 1 ? "s" : ""} successfully`);
  } else if (failed > 0 && success > 0) {
    toastStore.warning(`Restored ${success}, failed ${failed} snapshot${failed > 1 ? "s" : ""}`);
  } else if (failed > 0) {
    toastStore.error(`Failed to restore ${failed} snapshot${failed > 1 ? "s" : ""}`);
  }

  return { success, failed };
}
