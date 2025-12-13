// Unified stores barrel export
// All reactive stores following Svelte 5 runes pattern

// Theme & UI
export { colorSchemeStore, COLOR_SCHEMES, type ColorSchemeId } from "./colorScheme.svelte";
export { themeStore, type Theme } from "./theme.svelte";
export { sidebarStore, isSidebarOpen, sidebarWidthClass, type SidebarState } from "./layout.svelte";
export { debugState, type DebugLogEntry } from "./debug.svelte";
export { settingsStore } from "./settings.svelte";

// Modal state
export {
  modalStore,
  openAboutModal,
  openSettingsModal,
  openUpdateModal,
  closeModal,
  type ModalType,
} from "./modal.svelte";
export { tweakDetailsModalStore } from "./tweakDetailsModal.svelte";

// Navigation
export { navigationStore, type TabDefinition } from "./navigation.svelte";

// Tweaks system (split into logical modules)
export {
  // Data stores
  systemStore,
  categoriesStore,
  tweaksStore,
  getCategoryStats,
  // Loading state
  loadingStore,
  errorStore,
  // Pending changes
  pendingChangesStore,
  pendingRebootStore,
  // Actions & filters
  filterStore,
  applyTweak,
  revertTweak,
  toggleTweak,
  stageChange,
  unstageChange,
  applyPendingChanges,
  initializeData,
} from "./tweaks.svelte";

// System elevation
export { systemElevationStore } from "./systemElevation.svelte";

// Update system
export { updateStore } from "./update.svelte";
