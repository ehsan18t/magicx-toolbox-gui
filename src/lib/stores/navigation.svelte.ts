/**
 * Navigation Store - Svelte 5 Runes
 *
 * Manages tab-based UI navigation between Overview and category tabs.
 */

import type { CategoryDefinition } from "$lib/types";
import { categoriesStore } from "./tweaksData.svelte";

/** Tab types - "overview" or category ID */
export type TabId = "overview" | string;

/** Tab definition for navigation */
export interface TabDefinition {
  id: TabId;
  name: string;
  icon: string;
  description?: string;
}

// === State ===
let activeTab = $state<TabId>("overview");

// Overview tab definition (static)
const overviewTab: TabDefinition = {
  id: "overview",
  name: "Overview",
  icon: "mdi:view-dashboard",
  description: "System information and statistics",
};

// Derived: All tabs from categories
const allTabs = $derived.by((): TabDefinition[] => {
  const categoryTabs: TabDefinition[] = categoriesStore.list.map((cat: CategoryDefinition) => ({
    id: cat.id,
    name: cat.name,
    icon: cat.icon,
    description: cat.description,
  }));

  return [overviewTab, ...categoryTabs];
});

// Derived: Current tab definition
const currentTab = $derived.by((): TabDefinition | undefined => {
  return allTabs.find((tab) => tab.id === activeTab);
});

// Derived: Is on a category tab (not overview)
const isOnCategoryTab = $derived(activeTab !== "overview");

// === Export ===

export const navigationStore = {
  /** Get the current active tab ID */
  get activeTab() {
    return activeTab;
  },

  /** Get all available tabs */
  get allTabs() {
    return allTabs;
  },

  /** Get the current tab definition */
  get currentTab() {
    return currentTab;
  },

  /** Check if currently on a category tab */
  get isOnCategoryTab() {
    return isOnCategoryTab;
  },

  /** Get the overview tab definition */
  get overviewTab() {
    return overviewTab;
  },

  /** Navigate to a specific tab by ID */
  navigateToTab(tabId: TabId) {
    activeTab = tabId;
  },

  /** Navigate to the overview tab */
  navigateToOverview() {
    activeTab = "overview";
  },

  /** Navigate to a specific category */
  navigateToCategory(categoryId: string) {
    activeTab = categoryId;
  },
};
