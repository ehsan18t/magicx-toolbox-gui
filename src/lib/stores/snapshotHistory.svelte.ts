import * as tweaksApi from "$lib/api/tweaks";
import type { EntrySummary, TweakWithStatus } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { confirmStore } from "./confirm.svelte";
import { toastStore } from "./toast.svelte";
import { tweakActionsStore } from "./tweakActions.svelte";

interface Listing {
  tweakId: string;
  entries: EntrySummary[];
  error: string | null;
}

/**
 * One tweak's snapshot entries, listed while `active()`; call during component init. Re-lists on every
 * new status of the tweak, keeping the shown list meanwhile: a restore that leaves older entries changes
 * nothing a narrower key (id, `hasHistory`) would see.
 */
export function createSnapshotHistory(tweak: () => TweakWithStatus | null, active: () => boolean) {
  let listing = $state<Listing | null>(null);
  let busySeq = $state<number | null>(null);
  let latestRequest = 0;

  const own = $derived(listing?.tweakId === tweak()?.definition.id ? listing : null);

  async function list(tweakId: string) {
    const request = ++latestRequest;
    try {
      const entries = await tweaksApi.listSnapshotEntries(tweakId);
      if (request === latestRequest) listing = { tweakId, entries, error: null };
    } catch (error) {
      if (request !== latestRequest) return;
      logError("Failed to list snapshot entries", error);
      const entries = listing?.tweakId === tweakId ? listing.entries : [];
      listing = { tweakId, entries, error: errorMessage(error) };
    }
  }

  $effect(() => {
    const current = tweak();
    if (current && active()) void list(current.definition.id);
  });

  async function discard(tweakId: string, seq: number) {
    busySeq = seq;
    try {
      await tweaksApi.discardSnapshotEntry(tweakId, seq);
      if (listing?.tweakId === tweakId) {
        listing = { ...listing, entries: listing.entries.filter((entry) => entry.seq !== seq) };
      }
      // The engine owns `hasHistory`, and its stamped status cannot be undone by an in-flight sweep.
      const read = await tweakActionsStore.refresh(tweakId);
      if (!read.ok) {
        toastStore.warning(`The entry was discarded, but the tweak's state could not be re-read: ${read.message}`);
      }
    } catch (error) {
      logError("Failed to discard snapshot entry", error);
      toastStore[isAppExiting(error) ? "warning" : "error"](errorMessage(error));
    } finally {
      busySeq = null;
    }
  }

  return {
    get entries(): EntrySummary[] {
      return own?.entries ?? [];
    },
    get loading(): boolean {
      return own === null;
    },
    get error(): string | null {
      return own?.error ?? null;
    },
    // Entries, not only `hasHistory`: an all-invalid history still needs a discard path (ADR-0002).
    get visible(): boolean {
      return !!tweak()?.status.hasHistory || (own !== null && (own.entries.length > 0 || own.error !== null));
    },
    get busySeq(): number | null {
      return busySeq;
    },

    async discardWithConfirm(seq: number): Promise<void> {
      const tweakId = tweak()?.definition.id;
      if (!tweakId) return;
      const ok = await confirmStore.ask({
        title: `Discard snapshot entry #${seq}?`,
        message: "The state it recorded can no longer be restored for this tweak.",
        confirmText: "Discard",
        variant: "danger",
      });
      if (ok) await discard(tweakId, seq);
    },
  };
}

export type SnapshotHistory = ReturnType<typeof createSnapshotHistory>;
