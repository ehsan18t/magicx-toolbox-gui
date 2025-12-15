/**
 * Navigation Store - Svelte 5 Runes
 *
 * Manages tab-based UI navigation between Overview, Search, and category tabs.
 */

import type { CategoryDefinition } from "$lib/types";
import { categoriesStore } from "./tweaksData.svelte";

/** Tab types - "overview", "search", or category ID */
export type TabId = "overview" | "search" | string;

/** Tab definition for navigation */
export interface TabDefinition {
  id: TabId;
  name: string;
  icon: string;
  description?: string;
  /** Whether this is a permanent/fixed tab (Overview, Search) vs dynamic category tab */
  isPermanent?: boolean;
}

// === State ===
let activeTab = $state<TabId>("overview");

// Overview tab definition (static)
const overviewTab: TabDefinition = {
  id: "overview",
  name: "Overview",
  icon: "mdi:view-dashboard",
  description: "System information and statistics",
  isPermanent: true,
};

// Search tab definition (static)
const searchTab: TabDefinition = {
  id: "search",
  name: "Search",
  icon: "mdi:magnify",
  description: "Search tweaks by name, description, or info",
  isPermanent: true,
};

// Derived: All tabs from categories
const allTabs = $derived.by((): TabDefinition[] => {
  const categoryTabs: TabDefinition[] = categoriesStore.list.map((cat: CategoryDefinition) => ({
    id: cat.id,
    name: cat.name,
    icon: cat.icon,
    description: cat.description,
    isPermanent: false,
  }));

  return [overviewTab, searchTab, ...categoryTabs];
});

// Derived: Fixed/permanent tabs (Overview, Search)
const fixedTabs = $derived.by((): TabDefinition[] => {
  return allTabs.filter((tab) => tab.isPermanent === true);
});

// Derived: Category tabs (dynamic from YAML)
const categoryTabs = $derived.by((): TabDefinition[] => {
  return allTabs.filter((tab) => tab.isPermanent !== true);
});

// Derived: Current tab definition
const currentTab = $derived.by((): TabDefinition | undefined => {
  return allTabs.find((tab) => tab.id === activeTab);
});

// Derived: Is on a category tab (not overview or search)
const isOnCategoryTab = $derived(activeTab !== "overview" && activeTab !== "search");

// Derived: Is on search tab
const isOnSearchTab = $derived(activeTab === "search");

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

  /** Get fixed/permanent tabs (Overview, Search) */
  get fixedTabs() {
    return fixedTabs;
  },

  /** Get category tabs (dynamic from YAML) */
  get categoryTabs() {
    return categoryTabs;
  },

  /** Get the current tab definition */
  get currentTab() {
    return currentTab;
  },

  /** Check if currently on a category tab */
  get isOnCategoryTab() {
    return isOnCategoryTab;
  },

  /** Check if currently on the search tab */
  get isOnSearchTab() {
    return isOnSearchTab;
  },

  /** Get the overview tab definition */
  get overviewTab() {
    return overviewTab;
  },

  /** Get the search tab definition */
  get searchTab() {
    return searchTab;
  },

  /** Navigate to a specific tab by ID */
  navigateToTab(tabId: TabId) {
    activeTab = tabId;
  },

  /** Navigate to the overview tab */
  navigateToOverview() {
    activeTab = "overview";
  },

  /** Navigate to the search tab */
  navigateToSearch() {
    activeTab = "search";
  },

  /** Navigate to a specific category */
  navigateToCategory(categoryId: string) {
    activeTab = categoryId;
  },
};
