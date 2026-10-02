import { STORAGE_KEYS } from "$lib/config/app";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { innerWidth } from "svelte/reactivity/window";

/** Window width at which the navigation pane docks expanded (WinUI NavigationView's threshold). */
const EXPANDED_MIN_WIDTH = 1008;

const collapsedState = new PersistentStore(STORAGE_KEYS.navCollapsed, false);

let isOverlayOpen = $state(false);

const canDockExpanded = $derived((innerWidth.current ?? EXPANDED_MIN_WIDTH) >= EXPANDED_MIN_WIDTH);
const isDockedExpanded = $derived(canDockExpanded && !collapsedState.value);
const isOverlay = $derived(isOverlayOpen && !isDockedExpanded);

export const sidebarStore = {
  /** Labels visible: docked expanded, or opened over the content. */
  get isOpen() {
    return isDockedExpanded || isOverlay;
  },

  get isDockedExpanded() {
    return isDockedExpanded;
  },

  get isOverlay() {
    return isOverlay;
  },

  toggle() {
    if (canDockExpanded) {
      collapsedState.value = !collapsedState.value;
      isOverlayOpen = false;
    } else {
      isOverlayOpen = !isOverlayOpen;
    }
  },

  closeOverlay() {
    isOverlayOpen = false;
  },
};
