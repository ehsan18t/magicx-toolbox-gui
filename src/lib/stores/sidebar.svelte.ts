import { STORAGE_KEYS } from "$lib/config/app";
import { remToken } from "$lib/utils/cssToken";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { innerWidth } from "svelte/reactivity/window";

const collapsedState = new PersistentStore(STORAGE_KEYS.navCollapsed, false, (stored) =>
  typeof stored === "boolean" ? stored : undefined,
);

const canDockExpanded = $derived(
  innerWidth.current === undefined || innerWidth.current >= remToken("--breakpoint-nav-expanded"),
);
// Closed whenever the dock mode flips: narrowing again must not reopen the drawer and take focus.
let isOverlayOpen = $derived.by(() => {
  void canDockExpanded;
  return false;
});
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
