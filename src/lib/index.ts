// Components
export { default as CategoryTab } from "./components/CategoryTab.svelte";
export { default as ColorSchemePicker } from "./components/ColorSchemePicker.svelte";
export { default as ConfirmDialog } from "./components/ConfirmDialog.svelte";
export { default as ControlButton } from "./components/ControlButton.svelte";
export { default as DebugPanel } from "./components/DebugPanel.svelte";
export { default as ExternalLink } from "./components/ExternalLink.svelte";
export { default as OverviewTab } from "./components/OverviewTab.svelte";
export { default as PendingRebootBanner } from "./components/PendingRebootBanner.svelte";
export { default as Sidebar } from "./components/Sidebar.svelte";
export { default as TitleBar } from "./components/TitleBar.svelte";
export { default as TweakCard } from "./components/TweakCard.svelte";

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
  initializeData,
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
  getTweakStatuses,
  getTweaksForCurrentVersion,
  getWindowsVersion,
  isAdmin,
} from "./api";
