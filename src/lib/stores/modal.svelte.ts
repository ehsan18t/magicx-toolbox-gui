// Modal state store for managing modal visibility
// Using Svelte 5 runes for reactive state

export type ModalType = "about" | "settings" | "update" | null;

// Reactive state
let currentModal = $state<ModalType>(null);

// Export the modal store with methods
export const modalStore = {
  get current() {
    return currentModal;
  },

  get isOpen() {
    return currentModal !== null;
  },

  open(modal: ModalType) {
    currentModal = modal;
  },

  close() {
    currentModal = null;
  },

  toggle(modal: Exclude<ModalType, null>) {
    currentModal = currentModal === modal ? null : modal;
  },
};

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
