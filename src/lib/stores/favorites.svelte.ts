/**
 * Favorites Store - Svelte 5 Runes
 *
 * Manages favorite tweaks with localStorage persistence.
 * Only stores tweak IDs - tweak data comes from tweaksStore.
 */

import { PersistentStore } from "$lib/utils/persistentStore.svelte";

const STORAGE_KEY = "magicx-favorites";

// Persistent state
const favoritesState = new PersistentStore<string[]>(STORAGE_KEY, []);

// === Derived Values ===
const count = $derived(favoritesState.value.length);
const isEmpty = $derived(favoritesState.value.length === 0);
const ids = $derived(favoritesState.value);

// === Export ===
export const favoritesStore = {
  /** Get the count of favorites */
  get count() {
    return count;
  },

  /** Check if favorites is empty */
  get isEmpty() {
    return isEmpty;
  },

  /** Get all favorite IDs as array */
  get ids() {
    return ids;
  },

  /** Check if a tweak is favorited */
  isFavorite(tweakId: string): boolean {
    return favoritesState.value.includes(tweakId);
  },

  /** Add a tweak to favorites */
  add(tweakId: string): void {
    if (!favoritesState.value.includes(tweakId)) {
      favoritesState.value = [...favoritesState.value, tweakId];
    }
  },

  /** Remove a tweak from favorites */
  remove(tweakId: string): void {
    favoritesState.value = favoritesState.value.filter((id) => id !== tweakId);
  },

  /** Toggle a tweak's favorite status */
  toggle(tweakId: string): boolean {
    if (favoritesState.value.includes(tweakId)) {
      this.remove(tweakId);
      return false;
    } else {
      this.add(tweakId);
      return true;
    }
  },

  /** Clear all favorites */
  clear(): void {
    favoritesState.value = [];
  },
};
