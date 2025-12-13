import { writable } from "svelte/store";

export interface TweakDetailsModalState {
  tweakId: string;
}

function createTweakDetailsModalStore() {
  const { subscribe, set } = writable<TweakDetailsModalState | null>(null);

  return {
    subscribe,
    open(tweakId: string) {
      set({ tweakId });
    },
    close() {
      set(null);
    },
  };
}

export const tweakDetailsModalStore = createTweakDetailsModalStore();

export function openTweakDetailsModal(tweakId: string) {
  tweakDetailsModalStore.open(tweakId);
}

export function closeTweakDetailsModal() {
  tweakDetailsModalStore.close();
}
