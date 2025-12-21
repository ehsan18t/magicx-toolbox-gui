/**
 * Favorites Store - Svelte 5 Runes
 *
 * Manages favorite tweaks with localStorage persistence.
 * Only stores tweak IDs - tweak data comes from tweaksStore.
 */

import { browser } from "$app/environment";
import { SvelteSet } from "svelte/reactivity";

const STORAGE_KEY = "magicx-favorites";

/**
 * Load favorites from localStorage
 */
function loadFavorites(): SvelteSet<string> {
  const favorites = new SvelteSet<string>();

  if (!browser) return favorites;

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed)) {
        for (const id of parsed) {
          if (typeof id === "string") {
            favorites.add(id);
          }
        }
      }
    }
  } catch (error) {
    console.error("Failed to load favorites from localStorage:", error);
  }

  return favorites;
}

/**
 * Save favorites to localStorage
 */
function saveFavorites(favorites: SvelteSet<string>): void {
  if (!browser) return;

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...favorites]));
  } catch (error) {
    console.error("Failed to save favorites to localStorage:", error);
  }
}

// === Reactive State ===
const favorites = loadFavorites();

// === Derived Values ===
const count = $derived(favorites.size);
const isEmpty = $derived(favorites.size === 0);
const ids = $derived([...favorites]);

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
    return favorites.has(tweakId);
  },

  /** Add a tweak to favorites */
  add(tweakId: string): void {
    favorites.add(tweakId);
    saveFavorites(favorites);
  },

  /** Remove a tweak from favorites */
  remove(tweakId: string): void {
    favorites.delete(tweakId);
    saveFavorites(favorites);
  },

  /** Toggle a tweak's favorite status */
  toggle(tweakId: string): boolean {
    if (favorites.has(tweakId)) {
      favorites.delete(tweakId);
      saveFavorites(favorites);
      return false;
    } else {
      favorites.add(tweakId);
      saveFavorites(favorites);
      return true;
    }
  },

  /** Clear all favorites */
  clear(): void {
    favorites.clear();
    saveFavorites(favorites);
  },
};
