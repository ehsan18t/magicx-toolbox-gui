/**
 * Tweaks Store - Unified API
 *
 * Re-exports all tweak-related stores and actions for a unified import.
 *
 * @example
 * ```ts
 * import {
 *   tweaksStore,
 *   categoriesStore,
 *   systemStore,
 *   loadingStore,
 *   errorStore,
 *   pendingChangesStore,
 *   pendingRebootStore,
 *   filterStore,
 *   applyTweak,
 *   revertTweak,
 *   initializeQuick,
 *   loadRemainingData
 * } from "$lib/stores/tweaks.svelte";
 * ```
 */

// Data stores
export {
  categoriesStore,
  getCategoryStats,
  initializeQuick,
  loadingStateStore,
  loadRemainingData,
  systemStore,
  tweaksStore,
} from "./tweaksData.svelte";

// Loading & error stores
export { errorStore, loadingStore } from "./tweaksLoading.svelte";

// Pending changes & reboot stores
export { pendingChangesStore, pendingRebootStore } from "./tweaksPending.svelte";

// Actions & filter store
export {
  applyPendingChanges,
  applyTweak,
  filterStore,
  revertTweak,
  stageChange,
  toggleTweak,
  unstageChange,
} from "./tweaksActions.svelte";
