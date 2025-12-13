// Sidebar layout state using Svelte 5 runes

export interface SidebarState {
  isExpanded: boolean;
  isPinned: boolean;
}

// Reactive state
let isExpanded = $state(false);
let isPinned = $state(false);

// Derived values
const isOpen = $derived(isExpanded || isPinned);
const widthClass = $derived(isOpen ? "w-60" : "w-16");
const contentLeftOffset = $derived(isOpen ? "left-60" : "left-16");

// Export the sidebar store
export const sidebarStore = {
  get isExpanded() {
    return isExpanded;
  },

  get isPinned() {
    return isPinned;
  },

  get isOpen() {
    return isOpen;
  },

  get widthClass() {
    return widthClass;
  },

  get contentLeftOffset() {
    return contentLeftOffset;
  },

  setExpanded(expanded: boolean) {
    isExpanded = expanded;
  },

  setPinned(pinned: boolean) {
    isPinned = pinned;
  },

  togglePinned() {
    isPinned = !isPinned;
    // If unpinning, collapse sidebar
    if (!isPinned) {
      isExpanded = false;
    }
  },

  init(pinned: boolean) {
    isExpanded = false;
    isPinned = pinned;
  },
};

// Legacy compatibility exports (derived stores for backward compatibility)
// These are kept for components that haven't been migrated yet
export const isSidebarOpen = {
  get value() {
    return isOpen;
  },
};

export const sidebarWidthClass = {
  get value() {
    return widthClass;
  },
};
