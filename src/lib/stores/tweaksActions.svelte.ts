/**
 * Tweaks Actions Store - Svelte 5 Runes
 *
 * Action functions for the redesigned engine: apply BY LABEL, restore (single
 * head-walk), and discard snapshot entries. Batch flows are client-side loops over
 * the per-tweak commands (there is no backend batch command).
 */

import * as api from "$lib/api/tweaks";
import type { PendingChange } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
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

type ActionResult = { status: "ok" | "failed" } | { status: "exiting"; message: string };

const EXITING_REST_KEPT = "because the app is restarting or updating.";

/**
 * Apply a tweak's option by LABEL. The command returns the fresh post-op status,
 * which we adopt directly (no re-fetch / no re-scan).
 */
export async function applyTweak(
  tweakId: string,
  optionLabel: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<boolean> {
  return (await applyTweakResult(tweakId, optionLabel, options)).status === "ok";
}

async function applyTweakResult(
  tweakId: string,
  optionLabel: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<ActionResult> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);

  try {
    const outcome = await api.applyTweak(tweakId, optionLabel);
    errorStore.clearError(tweakId);
    tweaksStore.setStatusView(tweakId, outcome.status);
    pendingChangesStore.clear(tweakId);

    if (showToast) {
      toastStore.success("Applied successfully", { tweakName });
    }
    return { status: "ok" };
  } catch (error) {
    const message = errorMessage(error);
    // An exit refusal touched nothing, so the tweak's status and error are left as they were.
    if (isAppExiting(error)) {
      if (showToast) toastStore.warning(message, { tweakName });
      return { status: "exiting", message };
    }
    errorStore.setError(tweakId, message);
    if (showToast) {
      toastStore.error(message, { tweakName });
    }
    return { status: "failed" };
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
  return (await revertTweakResult(tweakId, options)).status === "ok";
}

async function revertTweakResult(
  tweakId: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<ActionResult> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);

  try {
    const outcome = await api.restoreTweak(tweakId);
    errorStore.clearError(tweakId);
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
    return { status: "ok" };
  } catch (error) {
    const message = errorMessage(error);
    // An exit refusal touched nothing: not a failed rollback, so no Needs Attention.
    if (isAppExiting(error)) {
      if (showToast) toastStore.warning(message, { tweakName });
      return { status: "exiting", message };
    }
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
    return { status: "failed" };
  } finally {
    loadingStore.stop(tweakId);
  }
}

async function refreshSnapshotFlags(tweakId: string): Promise<void> {
  try {
    const remaining = await api.listSnapshotEntries(tweakId);
    if (remaining.length === 0) {
      tweaksStore.patchStatus(tweakId, { has_backup: false, needs_attention: false, unrestorable_resources: [] });
      pendingRebootStore.remove(tweakId);
    }
  } catch (error) {
    console.error("Failed to re-read snapshot entries:", errorMessage(error));
  }
}

/**
 * Explicit-consent snapshot release (ADR-0002): discard every snapshot entry for the
 * tweak, the way out of Needs Attention when the user accepts the current state.
 */
export async function discardSnapshots(
  tweakId: string,
  options?: { showToast?: boolean; tweakName?: string },
): Promise<boolean> {
  const showToast = options?.showToast ?? true;
  const tweakName = options?.tweakName ?? tweaksStore.getById(tweakId)?.definition.name;

  loadingStore.start(tweakId);
  let discarded = 0;
  try {
    const entries = await api.listSnapshotEntries(tweakId);
    for (const entry of entries) {
      await api.discardSnapshotEntry(tweakId, entry.seq);
      discarded++;
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
    const message = errorMessage(error);
    // Entries discarded before the failure are gone, so the backup flags are re-read.
    if (discarded > 0) await refreshSnapshotFlags(tweakId);
    if (isAppExiting(error)) {
      if (showToast) {
        const entries = `${discarded} snapshot ${discarded === 1 ? "entry" : "entries"}`;
        const text = discarded === 0 ? message : `Discarded ${entries}; the rest were kept ${EXITING_REST_KEPT}`;
        toastStore.warning(text, { tweakName });
      }
      return false;
    }
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

/** The backend's refusal says nothing was changed, so it is quoted only when nothing ran. */
function batchStopMessage(
  verb: string,
  noun: string,
  counts: { success: number; failed: number; skipped: number },
  refusal: string,
): string {
  const { success, failed, skipped } = counts;
  const items = `${skipped} ${noun}${skipped === 1 ? "" : "s"}`;
  const were = skipped === 1 ? "was" : "were";
  if (success + failed === 0) return `${items} ${were} not ${verb}. ${refusal}`;
  const done = verb[0].toUpperCase() + verb.slice(1);
  return `${done} ${success}, failed ${failed}; the remaining ${items} ${were} not attempted ${EXITING_REST_KEPT}`;
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
  for (const [index, change] of changes.entries()) {
    const tweakName = tweaksStore.getById(change.tweakId)?.definition.name;
    const result = await applyTweakResult(change.tweakId, change.optionLabel, { showToast: false, tweakName });
    if (result.status === "exiting") {
      const counts = { success, failed, skipped: changes.length - index };
      toastStore.warning(batchStopMessage("applied", "tweak", counts, result.message));
      return { success, failed };
    }
    if (result.status === "ok") success++;
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
  for (const [index, tweakId] of tweakIds.entries()) {
    const result = await revertTweakResult(tweakId, { showToast: false });
    if (result.status === "exiting") {
      const counts = { success, failed, skipped: tweakIds.length - index };
      toastStore.warning(batchStopMessage("restored", "snapshot", counts, result.message));
      return { success, failed };
    }
    if (result.status === "ok") success++;
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
