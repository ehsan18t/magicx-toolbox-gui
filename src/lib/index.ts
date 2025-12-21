// Components (re-export all from organized structure)
export * from "./components";

// Stores (Svelte 5 runes-based)
export { COLOR_SCHEMES, colorSchemeStore, type ColorSchemeId } from "./stores/colorScheme.svelte";
export { sidebarStore } from "./stores/layout.svelte";
export {
  closeModal,
  modalStore,
  openAboutModal,
  openSettingsModal,
  openUpdateModal,
  type ModalType,
} from "./stores/modal.svelte";
export { settingsStore } from "./stores/settings.svelte";
export { themeStore, type Theme } from "./stores/theme.svelte";
export {
  closeTweakDetailsModal,
  openTweakDetailsModal,
  tweakDetailsModalStore,
} from "./stores/tweakDetailsModal.svelte";

// Navigation store (runes-based)
export { navigationStore, type TabDefinition, type TabId } from "./stores/navigation.svelte";

// Tweaks stores (runes-based)
export {
  applyPendingChanges,
  applyTweak,
  categoriesStore,
  errorStore,
  filterStore,
  getCategoryStats,
  initializeQuick,
  loadRemainingData,
  loadingStore,
  pendingChangesStore,
  pendingRebootStore,
  revertTweak,
  stageChange,
  systemStore,
  toggleTweak,
  tweaksStore,
  unstageChange,
} from "./stores/tweaks.svelte";

// System elevation store (runes-based)
export { systemElevationStore } from "./stores/systemElevation.svelte";

// Update store (runes-based)
export { updateStore } from "./stores/update.svelte";

// Config
export { APP_CONFIG, type AppConfig } from "./config/app";

// Types
export * from "./types";

// API - explicitly export to avoid conflicts with stores
export {
  batchApplyTweaks,
  getAllTweaksWithStatus,
  getAvailableTweaks,
  getSystemInfo,
  getTweakStatus,
  getWindowsVersion,
  isAdmin,
} from "./api";
