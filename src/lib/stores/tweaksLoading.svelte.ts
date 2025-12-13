/**
 * Tweaks Loading Store - Svelte 5 Runes
 *
 * Manages loading states and error messages for tweak operations.
 */

import { SvelteMap, SvelteSet } from "svelte/reactivity";

// === Loading State ===
const loadingTweaks = new SvelteSet<string>();

// === Error State ===
const errors = new SvelteMap<string, string>();

// === Exports ===

export const loadingStore = {
  /** Get all loading tweak IDs */
  get all() {
    return loadingTweaks;
  },

  /** Check if a specific tweak is loading */
  isLoading(tweakId: string): boolean {
    return loadingTweaks.has(tweakId);
  },

  /** Check if any tweak is loading */
  get isAnyLoading(): boolean {
    return loadingTweaks.size > 0;
  },

  /** Mark a tweak as loading */
  start(tweakId: string) {
    loadingTweaks.add(tweakId);
  },

  /** Mark a tweak as done loading */
  stop(tweakId: string) {
    loadingTweaks.delete(tweakId);
  },

  /** Clear all loading states */
  clear() {
    loadingTweaks.clear();
  },
};

export const errorStore = {
  /** Get all errors */
  get all() {
    return errors;
  },

  /** Get error for a specific tweak */
  getError(tweakId: string): string | undefined {
    return errors.get(tweakId);
  },

  /** Check if a tweak has an error */
  hasError(tweakId: string): boolean {
    return errors.has(tweakId);
  },

  /** Set an error for a tweak */
  setError(tweakId: string, message: string) {
    errors.set(tweakId, message);
  },

  /** Clear error for a tweak */
  clearError(tweakId: string) {
    errors.delete(tweakId);
  },

  /** Clear all errors */
  clearAll() {
    errors.clear();
  },
};
