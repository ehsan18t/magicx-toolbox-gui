// Modal state store for managing modal visibility
import { writable } from "svelte/store";

export type ModalType = "about" | "settings" | "update" | null;

function createModalStore() {
  const { subscribe, set, update } = writable<ModalType>(null);

  return {
    subscribe,
    open(modal: ModalType) {
      set(modal);
    },
    close() {
      set(null);
    },
    toggle(modal: Exclude<ModalType, null>) {
      update((current) => (current === modal ? null : modal));
    },
  };
}

export const modalStore = createModalStore();

// Convenience functions
export function openAboutModal() {
  modalStore.open("about");
}

export function openSettingsModal() {
  modalStore.open("settings");
}

export function openUpdateModal() {
  modalStore.open("update");
}

export function closeModal() {
  modalStore.close();
}
