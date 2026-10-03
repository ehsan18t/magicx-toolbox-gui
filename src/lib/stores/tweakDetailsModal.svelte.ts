import { navigationStore } from "./navigation.svelte";

// Keyed by page visit, so navigating away closes the modal with no effect to reset it.
let opened = $state<{ tweakId: string; visit: number } | null>(null);
// Outlives `opened`, so the dialog keeps its content through the exit animation.
let shownId = $state<string | null>(null);

export const tweakDetailsModalStore = {
  get tweakId(): string | null {
    return opened?.visit === navigationStore.visit ? opened.tweakId : null;
  },

  get shownId(): string | null {
    return shownId;
  },

  open(tweakId: string) {
    opened = { tweakId, visit: navigationStore.visit };
    shownId = tweakId;
  },

  close() {
    opened = null;
  },
};
