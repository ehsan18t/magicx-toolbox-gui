import { navigationStore } from "./navigation.svelte";

// Keyed by page visit, so navigating away closes the dialog with no effect to reset it.
let opened = $state<{ appId: string; visit: number } | null>(null);
// Outlives `opened`, so the dialog keeps its content through the exit animation.
let shownId = $state<string | null>(null);

/** One shared dialog for every app row, as with the tweak details. */
export const appDetailsModalStore = {
  get appId(): string | null {
    return opened?.visit === navigationStore.visit ? opened.appId : null;
  },

  get shownId(): string | null {
    return shownId;
  },

  open(appId: string) {
    opened = { appId, visit: navigationStore.visit };
    shownId = appId;
  },

  close() {
    opened = null;
  },
};
