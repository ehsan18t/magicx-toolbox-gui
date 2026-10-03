import type { PendingChange } from "$lib/types";
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { tweaksStore } from "./tweaksData.svelte";

/** Staged but not applied. */
const pendingChanges = new SvelteMap<string, PendingChange>();

const countByCategory = $derived.by(() => {
  const counts: Record<string, number> = {};
  for (const { tweakId } of pendingChanges.values()) {
    const category = tweaksStore.tweak(tweakId)?.definition.categoryId;
    if (category) counts[category] = (counts[category] ?? 0) + 1;
  }
  return counts;
});

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

  /** Staged changes per category id; a category with none is absent. */
  get countByCategory(): Readonly<Record<string, number>> {
    return countByCategory;
  },

  change(tweakId: string): PendingChange | undefined {
    return pendingChanges.get(tweakId);
  },

  has(tweakId: string): boolean {
    return pendingChanges.has(tweakId);
  },

  stage(tweakId: string, optionLabel: string) {
    pendingChanges.set(tweakId, { tweakId, optionLabel });
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

  has(tweakId: string): boolean {
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
