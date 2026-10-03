import { STORAGE_KEYS } from "$lib/config/app";
import { plural } from "$lib/utils/format";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { confirmStore } from "./confirm.svelte";
import { toastStore } from "./toast.svelte";

const favoritesState = new PersistentStore<readonly string[]>(STORAGE_KEYS.favorites, [], (stored) =>
  Array.isArray(stored) && stored.every((id) => typeof id === "string") ? stored : undefined,
);

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
  isFavorite(tweakId: string): boolean {
    return idSet.has(tweakId);
  },

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

  /** Clears only what the Favorites page lists, as its other actions do: hidden favorites stay. */
  async clearWithConfirm(shownIds: readonly string[]): Promise<void> {
    const ok = await confirmStore.ask({
      title: "Clear favorites?",
      message: `Remove ${plural(shownIds.length, "tweak")} from your favorites? This won't change the tweaks themselves.`,
      confirmText: "Clear favorites",
      variant: "danger",
    });
    if (!ok) return;
    favoritesState.value = ids.filter((id) => !shownIds.includes(id));
    toastStore.success("Favorites cleared");
  },
};
