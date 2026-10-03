import { navigationStore } from "./navigation.svelte";

/** One shared details dialog for every row of a kind. */
function createDetailsModal() {
  // Keyed by page visit, so navigating away closes the dialog with no effect to reset it.
  let opened = $state<{ id: string; visit: number } | null>(null);
  // Outlives `opened`, so the dialog keeps its content through the exit animation.
  let shownId = $state<string | null>(null);

  return {
    get openId(): string | null {
      return opened?.visit === navigationStore.visit ? opened.id : null;
    },

    get shownId(): string | null {
      return shownId;
    },

    open(id: string) {
      opened = { id, visit: navigationStore.visit };
      shownId = id;
    },

    close() {
      opened = null;
    },
  };
}

export const tweakDetailsModalStore = createDetailsModal();
export const appDetailsModalStore = createDetailsModal();
