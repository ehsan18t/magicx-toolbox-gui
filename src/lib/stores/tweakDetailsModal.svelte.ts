import { navigationStore } from "./navigation.svelte";

// Keyed by page visit, so navigating away closes the modal with no effect to reset it.
let opened = $state<{ tweakId: string; visit: number } | null>(null);

export const tweakDetailsModalStore = {
  get tweakId(): string | null {
    return opened?.visit === navigationStore.visit ? opened.tweakId : null;
  },

  open(tweakId: string) {
    opened = { tweakId, visit: navigationStore.visit };
  },

  close() {
    opened = null;
  },
};
