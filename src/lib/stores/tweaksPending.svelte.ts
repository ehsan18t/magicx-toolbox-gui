/**
 * Tweaks Pending Store - Svelte 5 Runes
 *
 * Manages pending changes (staged but not applied) and pending reboots.
 */

import type { PendingChange, TweakWithStatus } from "$lib/types";
import { SvelteMap, SvelteSet } from "svelte/reactivity";

// === Pending Changes State ===
const pendingChanges = new SvelteMap<string, PendingChange>();

// Derived: count of pending changes
const pendingCount = $derived(pendingChanges.size);

// === Pending Reboot State ===
const pendingReboot = new SvelteSet<string>();

// Derived: count of tweaks needing reboot
const rebootCount = $derived(pendingReboot.size);

// === Exports ===

export const pendingChangesStore = {
  /** Get all pending changes */
  get all() {
    return pendingChanges;
  },

  /** Get count of pending changes */
  get count() {
    return pendingCount;
  },

  /** Get pending change for a specific tweak */
  get(tweakId: string): PendingChange | undefined {
    return pendingChanges.get(tweakId);
  },

  /** Check if a tweak has pending changes */
  has(tweakId: string): boolean {
    return pendingChanges.has(tweakId);
  },

  /** Stage a change (doesn't apply, just marks pending) */
  set(tweakId: string, change: PendingChange) {
    pendingChanges.set(tweakId, change);
  },

  /** Remove a pending change */
  clear(tweakId: string) {
    pendingChanges.delete(tweakId);
  },

  /** Clear all pending changes */
  clearAll() {
    pendingChanges.clear();
  },

  /** Clear pending changes for a specific category */
  clearCategory(categoryId: string, tweaks: TweakWithStatus[]) {
    const categoryTweakIds = tweaks.filter((t) => t.definition.category_id === categoryId).map((t) => t.definition.id);
    for (const tweakId of categoryTweakIds) {
      pendingChanges.delete(tweakId);
    }
  },

  /** Get pending changes for a specific category */
  getForCategory(categoryId: string, tweaks: TweakWithStatus[]): SvelteMap<string, PendingChange> {
    const result = new SvelteMap<string, PendingChange>();
    for (const [tweakId, change] of pendingChanges) {
      const tweak = tweaks.find((t) => t.definition.id === tweakId);
      if (tweak?.definition.category_id === categoryId) {
        result.set(tweakId, change);
      }
    }
    return result;
  },
};

export const pendingRebootStore = {
  /** Get all tweak IDs pending reboot */
  get all() {
    return pendingReboot;
  },

  /** Get count of tweaks pending reboot */
  get count() {
    return rebootCount;
  },

  /** Check if a tweak needs reboot */
  needsReboot(tweakId: string): boolean {
    return pendingReboot.has(tweakId);
  },

  /** Mark a tweak as needing reboot */
  add(tweakId: string) {
    pendingReboot.add(tweakId);
  },

  /** Remove a tweak from pending reboot */
  remove(tweakId: string) {
    pendingReboot.delete(tweakId);
  },

  /** Clear all pending reboots */
  clear() {
    pendingReboot.clear();
  },
};
