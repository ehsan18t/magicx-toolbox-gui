// Unified stores barrel export
// All reactive stores following Svelte 5 runes pattern

// Theme & UI
export { COLOR_SCHEMES, colorSchemeStore, type ColorSchemeId } from "./colorScheme.svelte";
export { debugState, type DebugLogEntry } from "./debug.svelte";
export { isSidebarOpen, sidebarStore, sidebarWidthClass, type SidebarState } from "./layout.svelte";
export { settingsStore } from "./settings.svelte";
export { themeStore, type Theme } from "./theme.svelte";

// Modal state
export {
  closeModal,
  modalStore,
  openAboutModal,
  openSettingsModal,
  openUpdateModal,
  type ModalType,
} from "./modal.svelte";
export { tweakDetailsModalStore } from "./tweakDetailsModal.svelte";

// Navigation
export { navigationStore, type TabDefinition } from "./navigation.svelte";

// Tweaks system (split into logical modules)
export {
  applyPendingChanges,
  applyTweak,
  categoriesStore,
  errorStore,
  // Actions & filters
  filterStore,
  getCategoryStats,
  initializeQuick,
  // Loading state
  loadingStore,
  loadRemainingData,
  // Pending changes
  pendingChangesStore,
  pendingRebootStore,
  revertTweak,
  stageChange,
  // Data stores
  systemStore,
  toggleTweak,
  tweaksStore,
  unstageChange,
} from "./tweaks.svelte";

// System elevation
export { systemElevationStore } from "./systemElevation.svelte";

// Update system
export { updateStore } from "./update.svelte";

// Toast notifications
export { toastStore, type Toast, type ToastType } from "./toast.svelte";
