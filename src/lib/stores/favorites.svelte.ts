import { STORAGE_KEYS } from "$lib/config/app";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

const favoritesState = new PersistentStore<readonly string[]>(STORAGE_KEYS.favorites, []);

const ids = $derived(favoritesState.value);
const idSet = $derived(new Set(ids));

function add(tweakId: string) {
  if (!idSet.has(tweakId)) favoritesState.value = [...ids, tweakId];
}

function remove(tweakId: string) {
  favoritesState.value = ids.filter((id) => id !== tweakId);
}

/** Tweak ids only; the tweak data lives in tweaksStore. */
export const favoritesStore = {
  get count() {
    return ids.length;
  },

  get ids(): readonly string[] {
    return ids;
  },

  isFavorite(tweakId: string): boolean {
    return idSet.has(tweakId);
  },

  add,
  remove,

  /** Whether the tweak is now a favorite. */
  toggle(tweakId: string): boolean {
    if (idSet.has(tweakId)) {
      remove(tweakId);
      return false;
    }
    add(tweakId);
    return true;
  },

  /** Drops ids the tweak model does not define, so they don't inflate the count. */
  prune(isKnown: (id: string) => boolean): void {
    const kept = ids.filter(isKnown);
    if (kept.length !== ids.length) favoritesState.value = kept;
  },

  clear(): void {
    favoritesState.value = [];
  },
};
