import type { PendingChange } from "$lib/types";
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { tweaksStore } from "./tweaksData.svelte";

/** Staged but not applied. */
const pendingChanges = new SvelteMap<string, PendingChange>();

const pendingReboot = new SvelteSet<string>();

// The full list, hidden tweaks included, so it always matches `count`.
const rebootTweaks = $derived(tweaksStore.all.filter((t) => pendingReboot.has(t.definition.id)));

export const pendingChangesStore = {
  get all(): ReadonlyMap<string, PendingChange> {
    return pendingChanges;
  },

  get count() {
    return pendingChanges.size;
  },

  get(tweakId: string): PendingChange | undefined {
    return pendingChanges.get(tweakId);
  },

  has(tweakId: string): boolean {
    return pendingChanges.has(tweakId);
  },

  set(tweakId: string, change: PendingChange) {
    pendingChanges.set(tweakId, change);
  },

  remove(tweakId: string) {
    pendingChanges.delete(tweakId);
  },

  clear() {
    pendingChanges.clear();
  },
};

export const pendingRebootStore = {
  get count() {
    return pendingReboot.size;
  },

  get tweaks() {
    return rebootTweaks;
  },

  needsReboot(tweakId: string): boolean {
    return pendingReboot.has(tweakId);
  },

  add(tweakId: string) {
    pendingReboot.add(tweakId);
  },

  remove(tweakId: string) {
    pendingReboot.delete(tweakId);
  },

  clear() {
    pendingReboot.clear();
  },
};
