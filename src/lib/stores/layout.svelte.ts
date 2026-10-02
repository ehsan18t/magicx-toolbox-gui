import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { innerWidth } from "svelte/reactivity/window";

/** Window width at which the navigation pane docks expanded (WinUI NavigationView's threshold). */
const EXPANDED_MIN_WIDTH = 1008;

const collapsedState = new PersistentStore("magicx-nav-collapsed", false);

let overlayOpen = $state(false);

const canDockExpanded = $derived((innerWidth.current ?? EXPANDED_MIN_WIDTH) >= EXPANDED_MIN_WIDTH);
const isDockedExpanded = $derived(canDockExpanded && !collapsedState.value);
const isOverlay = $derived(overlayOpen && !isDockedExpanded);

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
      overlayOpen = false;
    } else {
      overlayOpen = !overlayOpen;
    }
  },

  closeOverlay() {
    overlayOpen = false;
  },
};
