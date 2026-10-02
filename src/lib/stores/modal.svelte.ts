export type ModalType = "about" | "update" | "profileExport" | "profileImport";

let currentModal = $state<ModalType | null>(null);

export const modalStore = {
  get current() {
    return currentModal;
  },

  open(modal: ModalType) {
    currentModal = modal;
  },

  close() {
    currentModal = null;
  },
};
