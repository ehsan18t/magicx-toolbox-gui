// Components
export { default as CategorySection } from "./components/CategorySection.svelte";
export { default as CategoryTab } from "./components/CategoryTab.svelte";
export { default as ColorSchemePicker } from "./components/ColorSchemePicker.svelte";
export { default as ConfirmDialog } from "./components/ConfirmDialog.svelte";
export { default as ControlButton } from "./components/ControlButton.svelte";
export { default as DebugPanel } from "./components/DebugPanel.svelte";
export { default as ExternalLink } from "./components/ExternalLink.svelte";
export { default as FilterBar } from "./components/FilterBar.svelte";
export { default as OverviewTab } from "./components/OverviewTab.svelte";
export { default as PendingRebootBanner } from "./components/PendingRebootBanner.svelte";
export { default as Sidebar } from "./components/Sidebar.svelte";
export { default as StatsCard } from "./components/StatsCard.svelte";
export { default as SystemInfoCard } from "./components/SystemInfoCard.svelte";
export { default as TitleBar } from "./components/TitleBar.svelte";
export { default as TweakCard } from "./components/TweakCard.svelte";

// Stores
export * from "./stores/navigation";
export { colorSchemeStore, COLOR_SCHEMES, type ColorSchemeId } from "./stores/colorScheme";
export { themeStore, type Theme } from "./stores/theme";
export * from "./stores/tweaks";

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
