// Tweak details modal store using Svelte 5 runes

export interface TweakDetailsModalState {
  tweakId: string;
}

// Reactive state
let modalState = $state<TweakDetailsModalState | null>(null);

// Derived values
const isOpen = $derived(modalState !== null);

export const tweakDetailsModalStore = {
  get state() {
    return modalState;
  },

  get isOpen() {
    return isOpen;
  },

  get tweakId() {
    return modalState?.tweakId ?? null;
  },

  open(tweakId: string) {
    modalState = { tweakId };
  },

  close() {
    modalState = null;
  },
};

export function openTweakDetailsModal(tweakId: string) {
  tweakDetailsModalStore.open(tweakId);
}

export function closeTweakDetailsModal() {
  tweakDetailsModalStore.close();
}
