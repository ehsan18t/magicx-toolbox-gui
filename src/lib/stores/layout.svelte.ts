// Sidebar layout state using Svelte 5 runes
import { PersistentStore } from "$lib/utils/persistentStore.svelte";

export interface SidebarState {
  isExpanded: boolean;
  isPinned: boolean;
  isWidgetsOpen: boolean;
}

const SIDEBAR_PIN_KEY = "magicx-sidebar-pinned";
const SIDEBAR_WIDGETS_OPEN_KEY = "magicx-sidebar-widgets-open";

// Persistent state
const pinnedState = new PersistentStore(SIDEBAR_PIN_KEY, false);
const widgetsOpenState = new PersistentStore(SIDEBAR_WIDGETS_OPEN_KEY, true);

// Reactive state (transient)
let isExpanded = $state(false);

// Derived values
const isOpen = $derived(isExpanded || pinnedState.value);
const widthClass = $derived(isOpen ? "w-60" : "w-16");
const contentLeftOffset = $derived(isOpen ? "left-60" : "left-16");

// Export the sidebar store
export const sidebarStore = {
  get isExpanded() {
    return isExpanded;
  },

  get isPinned() {
    return pinnedState.value;
  },

  get isWidgetsOpen() {
    return widgetsOpenState.value;
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
    pinnedState.value = pinned;
    if (!pinned) {
      isExpanded = false;
    }
  },

  togglePinned() {
    pinnedState.value = !pinnedState.value;
    if (!pinnedState.value) {
      isExpanded = false;
    }
  },

  setWidgetsOpen(open: boolean) {
    widgetsOpenState.value = open;
  },

  toggleWidgets() {
    widgetsOpenState.value = !widgetsOpenState.value;
  },

  init(pinned: boolean) {
    isExpanded = false;
    pinnedState.value = pinned;
  },
};

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
