// Navigation store for tab-based UI
import { derived, writable } from "svelte/store";
import type { CategoryDefinition } from "../types";
import { categoriesStore } from "./tweaks";

/** Tab types - "overview" or category ID */
export type TabId = "overview" | string;

/** Tab definition for navigation */
export interface TabDefinition {
  id: TabId;
  name: string;
  icon: string;
  description?: string;
}

// Currently active tab
export const activeTab = writable<TabId>("overview");

// Overview tab definition (static)
export const overviewTab: TabDefinition = {
  id: "overview",
  name: "Overview",
  icon: "mdi:view-dashboard",
  description: "System information and statistics",
};

// All tabs derived from categories
export const allTabs = derived(categoriesStore, ($categories): TabDefinition[] => {
  const categoryTabs: TabDefinition[] = $categories.map((cat: CategoryDefinition) => ({
    id: cat.id,
    name: cat.name,
    icon: cat.icon,
    description: cat.description,
  }));

  return [overviewTab, ...categoryTabs];
});

// Current tab definition
export const currentTab = derived([activeTab, allTabs], ([$activeTab, $allTabs]): TabDefinition | undefined => {
  return $allTabs.find((tab) => tab.id === $activeTab);
});

// Check if current tab is a category tab
export const isOnCategoryTab = derived(activeTab, ($activeTab): boolean => {
  return $activeTab !== "overview";
});

// Navigation helpers
export function navigateToTab(tabId: TabId): void {
  activeTab.set(tabId);
}

export function navigateToOverview(): void {
  activeTab.set("overview");
}

export function navigateToCategory(categoryId: string): void {
  activeTab.set(categoryId);
}
