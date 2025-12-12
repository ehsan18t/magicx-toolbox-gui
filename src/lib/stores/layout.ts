import { derived, writable } from "svelte/store";

export interface SidebarState {
  isExpanded: boolean;
  isPinned: boolean;
}

function createSidebarStore() {
  const { subscribe, set, update } = writable<SidebarState>({
    isExpanded: false,
    isPinned: false,
  });

  return {
    subscribe,
    setExpanded: (expanded: boolean) => update((s) => ({ ...s, isExpanded: expanded })),
    setPinned: (pinned: boolean) => update((s) => ({ ...s, isPinned: pinned })),
    togglePinned: () =>
      update((s) => {
        const newPinned = !s.isPinned;
        return {
          ...s,
          isPinned: newPinned,
          // If unpinning, collapse sidebar
          isExpanded: newPinned ? s.isExpanded : false,
        };
      }),
    init: (pinned: boolean) => set({ isExpanded: false, isPinned: pinned }),
  };
}

export const sidebarState = createSidebarStore();

// Derived store for "effectively expanded" (either pinned or hovered)
export const isSidebarOpen = derived(sidebarState, ($s) => $s.isExpanded || $s.isPinned);

// Derived store for sidebar width class
// Matches sidebar width classes: w-60 (15rem/240px) vs w-16 (4rem/64px)
export const sidebarWidthClass = derived(isSidebarOpen, ($isOpen) => ($isOpen ? "w-60" : "w-16"));

// Derived store for left offset class (for fixed elements)
export const contentLeftOffset = derived(isSidebarOpen, ($isOpen) =>
  $isOpen ? "left-60" : "left-16",
);
