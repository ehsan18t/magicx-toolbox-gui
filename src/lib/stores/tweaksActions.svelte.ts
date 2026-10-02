// Apply by label, restore (single head-walk) and keep-current-state. Batches are client-side loops over
// the per-tweak commands: there is no backend batch command.

import * as api from "$lib/api/tweaks";
import { errorMessage, isAppExiting, tweakFailureAdvice } from "$lib/utils/error";
import { capitalize, plural } from "$lib/utils/format";
import { logError } from "$lib/utils/logger";
import { elevationStore } from "./elevation.svelte";
import { toastStore } from "./toast.svelte";
import { errorStore, tweakOps } from "./tweakOps.svelte";
import { tweaksStore } from "./tweaksData.svelte";
import { pendingChangesStore, pendingRebootStore } from "./tweaksPending.svelte";

export interface ActionOptions {
  showToast?: boolean;
  /** Toast title; the tweak's name by default. */
  subject?: string;
}

type ActionResult = { status: "ok" | "failed" } | { status: "exiting"; message: string };

export interface BatchCounts {
  success: number;
  failed: number;
}

/** A status re-read either answered, or failed as itself, never silently as "no attention". */
export type StatusRead = { ok: true; needsAttention: boolean } | { ok: false; message: string };

/** Why a batch stopped early: the backend refuses new work while the app exits. */
const EXIT_STOP_REASON = "because the app is restarting or updating.";

/**
 * Re-reads one tweak's status from the engine, which owns Needs Attention: only the backend knows whether
 * a failed apply or restore recorded it. A failed read is reported as itself, never as "no attention".
 */
export async function refreshTweakStatus(tweakId: string): Promise<StatusRead> {
  try {
    const view = await api.getTweakStatus(tweakId);
    tweaksStore.setStatusView(tweakId, view);
    return { ok: true, needsAttention: view.attention !== null };
  } catch (error) {
    logError("Failed to re-read tweak status", error);
    return { ok: false, message: errorMessage(error) };
  }
}

/** The backend's message, plus what to do about it when its code calls for more. */
function failureText(error: unknown): string {
  const message = errorMessage(error);
  const advice = tweakFailureAdvice(error, elevationStore.level);
  return advice ? `${message}. ${advice}` : message;
}

/** Adopts the fresh post-op status the command returns: no re-fetch, no re-scan. */
export async function applyTweak(tweakId: string, optionLabel: string, options?: ActionOptions): Promise<boolean> {
  return (await applyTweakResult(tweakId, optionLabel, options)).status === "ok";
}

async function applyTweakResult(tweakId: string, optionLabel: string, options?: ActionOptions): Promise<ActionResult> {
  const showToast = options?.showToast ?? true;
  const definition = tweaksStore.getById(tweakId)?.definition;
  const subject = options?.subject ?? definition?.name;

  tweakOps.start(tweakId);
  try {
    const outcome = await api.applyTweak(tweakId, optionLabel);
    errorStore.clear(tweakId);
    tweaksStore.setStatusView(tweakId, outcome.status);
    // A different label staged while this call was in flight is the user's newer choice.
    if (pendingChangesStore.get(tweakId)?.optionLabel === optionLabel) pendingChangesStore.remove(tweakId);
    if (definition?.requiresReboot) pendingRebootStore.add(tweakId);

    if (showToast) {
      toastStore.success(definition?.requiresReboot ? "Applied (reboot required)" : "Applied successfully", {
        subject,
      });
    }
    return { status: "ok" };
  } catch (error) {
    const message = failureText(error);
    // An exit refusal touched nothing, so the tweak's status and error are left as they were.
    if (isAppExiting(error)) {
      if (showToast) toastStore.warning(message, { subject });
      return { status: "exiting", message };
    }
    errorStore.set(tweakId, message);
    // A failed apply may have recorded Needs Attention, and only the engine knows: without the
    // re-read's answer the row keeps its pre-apply status for the rest of the session.
    const read = await refreshTweakStatus(tweakId);

    if (showToast) {
      const text = !read.ok
        ? `Apply failed, and the tweak's state could not be re-read: ${message}`
        : read.needsAttention
          ? `Apply needs attention: ${message}`
          : message;
      toastStore.error(text, { subject });
    }
    return { status: "failed" };
  } finally {
    tweakOps.stop(tweakId);
  }
}

/**
 * Restores a tweak from its most recent snapshot. A restore that cannot fully complete surfaces as an
 * error (ADR-0001): the snapshot is kept and the tweak marked Needs Attention.
 */
export async function restoreTweak(tweakId: string, options?: ActionOptions): Promise<boolean> {
  return (await restoreTweakResult(tweakId, options)).status === "ok";
}

async function restoreTweakResult(tweakId: string, options?: ActionOptions): Promise<ActionResult> {
  const showToast = options?.showToast ?? true;
  const definition = tweaksStore.getById(tweakId)?.definition;
  const subject = options?.subject ?? definition?.name;

  tweakOps.start(tweakId);
  try {
    const outcome = await api.restoreTweak(tweakId);
    errorStore.clear(tweakId);
    tweaksStore.setStatusView(tweakId, outcome.status);
    pendingChangesStore.remove(tweakId);

    if (definition?.requiresReboot || outcome.reboot_advisory) pendingRebootStore.add(tweakId);
    else pendingRebootStore.remove(tweakId);

    if (showToast) {
      const text = definition?.requiresReboot
        ? "Restored (reboot required)"
        : outcome.reboot_advisory
          ? "Restored (reboot advised)"
          : "Restored successfully";
      toastStore.success(text, { subject });
    }
    return { status: "ok" };
  } catch (error) {
    const message = failureText(error);
    // An exit refusal touched nothing: not a failed rollback, so no Needs Attention.
    if (isAppExiting(error)) {
      if (showToast) toastStore.warning(message, { subject });
      return { status: "exiting", message };
    }
    errorStore.set(tweakId, message);
    // A restore can fail without leaving anything to attend to (the machine was restored and only
    // the spent entry outlived it), so the wording follows what the engine actually recorded.
    const read = await refreshTweakStatus(tweakId);

    if (showToast) {
      const text = !read.ok
        ? `Restore failed, and the tweak's state could not be re-read: ${message}`
        : read.needsAttention
          ? `Restore needs attention: ${message}`
          : `Restore reported a problem: ${message}`;
      toastStore.warning(text, { subject });
    }
    return { status: "failed" };
  } finally {
    tweakOps.stop(tweakId);
  }
}

/**
 * Explicit-consent release (ADR-0002), the only way out of Needs Attention that accepts the current
 * state. One backend operation clears the record and discards any entries left, then returns the status.
 */
export async function keepCurrentState(tweakId: string, options?: ActionOptions): Promise<boolean> {
  const showToast = options?.showToast ?? true;
  const subject = options?.subject ?? tweaksStore.getById(tweakId)?.definition.name;

  tweakOps.start(tweakId);
  try {
    tweaksStore.setStatusView(tweakId, await api.keepCurrentState(tweakId));
    errorStore.clear(tweakId);
    pendingRebootStore.remove(tweakId);
    if (showToast) toastStore.success("Current state kept", { subject });
    return true;
  } catch (error) {
    const message = errorMessage(error);
    // Whatever the call managed to release is in the engine's own status, so re-read rather than guess.
    await refreshTweakStatus(tweakId);
    if (isAppExiting(error)) {
      if (showToast) toastStore.warning(message, { subject });
      return false;
    }
    errorStore.set(tweakId, message);
    if (showToast) toastStore.error(message, { subject });
    return false;
  } finally {
    tweakOps.stop(tweakId);
  }
}

/** "applied", "apply", "tweak": the batch's words for its stop and summary toasts. */
interface BatchWords {
  done: string;
  verb: string;
  noun: string;
}

/** The backend's refusal says nothing was changed, so it is quoted only when nothing ran. */
function batchStopMessage(
  { done, noun }: BatchWords,
  { success, failed }: BatchCounts,
  skipped: number,
  refusal: string,
) {
  const items = plural(skipped, noun);
  const were = skipped === 1 ? "was" : "were";
  if (success + failed === 0) return `${items} ${were} not ${done}. ${refusal}`;
  return `${capitalize(done)} ${success}, failed ${failed}; the remaining ${items} ${were} not attempted ${EXIT_STOP_REASON}`;
}

/** Runs `step` over `ids` in order; a null result skips the id, an exit refusal stops the batch. */
async function runBatch(
  ids: string[],
  words: BatchWords,
  step: (id: string) => Promise<ActionResult | null>,
): Promise<BatchCounts> {
  const counts = { success: 0, failed: 0 };
  for (const [index, id] of ids.entries()) {
    const result = await step(id);
    if (result === null) continue;
    if (result.status === "exiting") {
      toastStore.warning(batchStopMessage(words, counts, ids.length - index, result.message));
      return counts;
    }
    if (result.status === "ok") counts.success++;
    else counts.failed++;
  }

  const { success, failed } = counts;
  const { done, verb, noun } = words;
  if (failed === 0 && success > 0) {
    toastStore.success(`${capitalize(done)} ${plural(success, noun)} successfully`);
  } else if (failed > 0 && success > 0) {
    toastStore.warning(`${capitalize(done)} ${success}, failed ${plural(failed, noun)}`);
  } else if (failed > 0) {
    toastStore.error(`Failed to ${verb} ${plural(failed, noun)}`);
  }
  return counts;
}

const NOTHING_RAN: BatchCounts = { success: 0, failed: 0 };

export async function applyPendingChanges(): Promise<BatchCounts> {
  const ids = Array.from(pendingChangesStore.all.keys());
  const words = { done: "applied", verb: "apply", noun: "tweak" };
  // Read at its turn: rows stay focusable under the overlay, so the stage can change mid-batch.
  const step = async (id: string) => {
    const change = pendingChangesStore.get(id);
    return change ? applyTweakResult(id, change.optionLabel, { showToast: false }) : null;
  };
  return (await tweakOps.exclusive(() => runBatch(ids, words, step))) ?? { ...NOTHING_RAN };
}

export async function restoreTweaks(tweakIds: string[]): Promise<BatchCounts> {
  const words = { done: "restored", verb: "restore", noun: "snapshot" };
  const step = (id: string) => restoreTweakResult(id, { showToast: false });
  return (await tweakOps.exclusive(() => runBatch(tweakIds, words, step))) ?? { ...NOTHING_RAN };
}
