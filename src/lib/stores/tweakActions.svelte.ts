// Apply by label, restore (single head-walk) and keep-current-state, with the per-tweak running and error
// state they drive. Batches are client-side loops over the per-tweak commands: there is no backend batch.

import * as tweaksApi from "$lib/api/tweaks";
import type { ApplyOutcome, RestoreOutcome, TweakDefinition, TweakStatusView, TweakWithStatus } from "$lib/types";
import { errorMessage, isAppExiting, tweakFailureAdvice } from "$lib/utils/error";
import { capitalize, plural } from "$lib/utils/format";
import { logError } from "$lib/utils/logger";
import { ensureSentence, restoreMessage } from "$lib/utils/tweakPresentation";
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { confirmStore } from "./confirm.svelte";
import { elevationStore } from "./elevation.svelte";
import { toastStore } from "./toast.svelte";
import { tweaksStore } from "./tweaksData.svelte";
import { pendingChangesStore, pendingRebootStore } from "./tweaksPending.svelte";

export interface ActionOptions {
  showToast?: boolean;
  /** Toast title; the tweak's name by default. */
  subject?: string;
}

export interface BatchCounts {
  success: number;
  failed: number;
}

/** A status re-read either answered, or failed as itself, never silently as "no attention". */
export type StatusRead = { ok: true; needsAttention: boolean } | { ok: false; message: string };

type ActionResult = { status: "ok" | "failed" } | { status: "exiting"; message: string };

/** Why a batch stopped early: the backend refuses new work while the app exits. */
const EXIT_STOP_REASON = "because the app is restarting or updating.";

const running = new SvelteSet<string>();
// Spans a whole batch: the per-tweak set is empty between items.
let isBatchRunning = $state(false);
const errors = new SvelteMap<string, string>();

async function refresh(tweakId: string): Promise<StatusRead> {
  try {
    const view = await tweaksApi.getTweakStatus(tweakId);
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
  return advice ? `${ensureSentence(message)} ${advice}` : message;
}

interface TweakOp<T> {
  run: () => Promise<T>;
  statusOf: (result: T) => TweakStatusView;
  /** Side effects of a success; returns the toast text. */
  succeeded: (result: T) => string;
  message: (error: unknown) => string;
  /** The failure toast, worded by what the engine recorded. */
  failed: (read: StatusRead, message: string) => { text: string; tone: "error" | "warning" };
}

async function runTweakOp<T>(
  tweakId: string,
  options: ActionOptions | undefined,
  op: TweakOp<T>,
): Promise<ActionResult> {
  const showToast = options?.showToast ?? true;
  const subject = options?.subject ?? tweaksStore.tweak(tweakId)?.definition.name;

  running.add(tweakId);
  try {
    const result = await op.run();
    errors.delete(tweakId);
    tweaksStore.setStatusView(tweakId, op.statusOf(result));
    const text = op.succeeded(result);
    if (showToast) toastStore.success(text, { subject });
    return { status: "ok" };
  } catch (error) {
    const message = op.message(error);
    // An exit refusal touched nothing, so the tweak's status and error are left as they were.
    if (isAppExiting(error)) {
      if (showToast) toastStore.warning(message, { subject });
      return { status: "exiting", message };
    }
    errors.set(tweakId, message);
    // A failure may have recorded Needs Attention, and only the engine knows: without the re-read's
    // answer the row keeps its pre-op status for the rest of the session.
    const read = await refresh(tweakId);
    if (showToast) {
      const { text, tone } = op.failed(read, message);
      toastStore[tone](text, { subject });
    }
    return { status: "failed" };
  } finally {
    running.delete(tweakId);
  }
}

function applyResult(tweakId: string, optionLabel: string, options?: ActionOptions): Promise<ActionResult> {
  const definition = tweaksStore.tweak(tweakId)?.definition;
  return runTweakOp<ApplyOutcome>(tweakId, options, {
    run: () => tweaksApi.applyTweak(tweakId, optionLabel),
    statusOf: (outcome) => outcome.status,
    succeeded: () => {
      // A different label staged while this call was in flight is the user's newer choice.
      if (pendingChangesStore.change(tweakId)?.optionLabel === optionLabel) pendingChangesStore.remove(tweakId);
      if (definition?.requiresReboot) pendingRebootStore.add(tweakId);
      return definition?.requiresReboot ? "Applied (reboot required)" : "Applied successfully";
    },
    message: failureText,
    failed: (read, message) => ({
      tone: "error",
      text: !read.ok
        ? `Apply failed, and the tweak's state could not be re-read: ${message}`
        : read.needsAttention
          ? `Apply needs attention: ${message}`
          : message,
    }),
  });
}

/** A restore that cannot fully complete surfaces as an error (ADR-0001): the snapshot is kept. */
function restoreResult(tweakId: string, options?: ActionOptions): Promise<ActionResult> {
  const definition = tweaksStore.tweak(tweakId)?.definition;
  return runTweakOp<RestoreOutcome>(tweakId, options, {
    run: () => tweaksApi.restoreTweak(tweakId),
    statusOf: (outcome) => outcome.status,
    succeeded: (outcome) => {
      pendingChangesStore.remove(tweakId);
      if (definition?.requiresReboot || outcome.reboot_advisory) pendingRebootStore.add(tweakId);
      else pendingRebootStore.remove(tweakId);
      return definition?.requiresReboot
        ? "Restored (reboot required)"
        : outcome.reboot_advisory
          ? "Restored (reboot advised)"
          : "Restored successfully";
    },
    message: failureText,
    // A restore can fail with nothing left to attend to (the machine was restored and only the spent
    // entry outlived it), so the wording follows what the engine actually recorded.
    failed: (read, message) => ({
      tone: "warning",
      text: !read.ok
        ? `Restore failed, and the tweak's state could not be re-read: ${message}`
        : read.needsAttention
          ? `Restore needs attention: ${message}`
          : `Restore reported a problem: ${message}`,
    }),
  });
}

/**
 * Explicit-consent release (ADR-0002), the only way out of Needs Attention that accepts the current
 * state. One backend operation clears the record and discards any entries left, then returns the status.
 */
async function keep(tweakId: string, options?: ActionOptions): Promise<boolean> {
  const result = await runTweakOp<TweakStatusView>(tweakId, options, {
    run: () => tweaksApi.keepCurrentState(tweakId),
    statusOf: (view) => view,
    succeeded: () => {
      pendingRebootStore.remove(tweakId);
      return "Current state kept";
    },
    message: errorMessage,
    failed: (_read, message) => ({ tone: "error", text: message }),
  });
  return result.status === "ok";
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

/** Runs `fn` as the only batch; resolves to null when another batch is already running. */
async function exclusive<T>(fn: () => Promise<T>): Promise<T | null> {
  if (isBatchRunning) return null;
  isBatchRunning = true;
  try {
    return await fn();
  } finally {
    isBatchRunning = false;
  }
}

const NOTHING_RAN: BatchCounts = { success: 0, failed: 0 };

async function restoreAll(tweakIds: string[]): Promise<BatchCounts> {
  const words = { done: "restored", verb: "restore", noun: "snapshot" };
  const step = (id: string) => restoreResult(id, { showToast: false });
  return (await exclusive(() => runBatch(tweakIds, words, step))) ?? { ...NOTHING_RAN };
}

export const tweakActionsStore = {
  isRunning(tweakId: string): boolean {
    return running.has(tweakId);
  },

  /** True while any tweak operation or batch runs; gate every batch control on this. */
  get isBusy(): boolean {
    return isBatchRunning || running.size > 0;
  },

  /** The last operation's failure for this tweak. */
  error(tweakId: string): string | undefined {
    return errors.get(tweakId);
  },

  clearError(tweakId: string) {
    errors.delete(tweakId);
  },

  /** Re-reads one tweak's status from the engine, which alone knows whether Needs Attention was recorded. */
  refresh,

  /** Adopts the fresh post-op status the command returns: no re-fetch, no re-scan. */
  async apply(tweakId: string, optionLabel: string, options?: ActionOptions): Promise<boolean> {
    return (await applyResult(tweakId, optionLabel, options)).status === "ok";
  },

  async restore(tweakId: string, options?: ActionOptions): Promise<boolean> {
    return (await restoreResult(tweakId, options)).status === "ok";
  },

  keep,

  async applyPending(): Promise<BatchCounts> {
    const ids = Array.from(pendingChangesStore.all.keys());
    const words = { done: "applied", verb: "apply", noun: "tweak" };
    // Read at its turn: rows stay focusable under the overlay, so the stage can change mid-batch.
    const step = async (id: string) => {
      const change = pendingChangesStore.change(id);
      return change ? applyResult(id, change.optionLabel, { showToast: false }) : null;
    };
    return (await exclusive(() => runBatch(ids, words, step))) ?? { ...NOTHING_RAN };
  },

  restoreAll,

  /** One Restore step, after a confirmation when `ask` is set. */
  async restoreWithConfirm(def: TweakDefinition, ask: boolean): Promise<void> {
    const ok =
      !ask ||
      (await confirmStore.ask({
        title: `Restore ${def.name}?`,
        message: `This ${def.riskLevel}-risk tweak steps back to the state saved before its last change.`,
        confirmText: "Restore",
        variant: "warning",
      }));
    // A second restore while one is in flight would walk back to the next-older snapshot.
    if (!ok || running.has(def.id)) return;
    await restoreResult(def.id, { showToast: true, subject: def.name });
  },

  async restoreAllWithConfirm(title: string, tweaks: TweakWithStatus[]): Promise<void> {
    const ok = await confirmStore.ask({
      title,
      message: restoreMessage(tweaks.length),
      confirmText: "Restore",
      variant: "danger",
    });
    if (ok) await restoreAll(tweaks.map((t) => t.definition.id));
  },

  /** Resolves to whether the snapshot was released; false when declined or refused. */
  async keepWithConfirm(def: TweakDefinition): Promise<boolean> {
    const ok = await confirmStore.ask({
      title: "Keep the current state?",
      message:
        "This accepts the current state as-is and releases the saved snapshot, so the earlier state can no longer be restored for this tweak.",
      confirmText: "Keep current state",
      variant: "danger",
    });
    return ok && (await keep(def.id, { showToast: true, subject: def.name }));
  },
};
