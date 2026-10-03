import type { IconName } from "$lib/design";
import { manualTestsStore } from "./manualTests.svelte";
import { searchStore } from "./search.svelte";
import { categoriesStore } from "./tweaksData.svelte";

const PAGE_IDS = ["overview", "search", "favorites", "snapshots", "profiles", "settings", "manual-tests"] as const;

export type PageId = (typeof PAGE_IDS)[number];

/** A fixed page, or a category id (any string the corpus defines). */
export type TabId = PageId | (string & {});

export interface TabDefinition {
  id: TabId;
  name: string;
  icon: IconName;
  description?: string;
}

export const isPageId = (tab: TabId): tab is PageId => (PAGE_IDS as readonly string[]).includes(tab);

let activeTab = $state<TabId>("overview");
let focusSearchSignal = $state(0);
// Bumped on every page change, so per-page UI state can tell a revisit from staying put.
let visit = $state(0);
let attentionVisit = -1;

function go(tab: TabId) {
  if (tab === activeTab) return;
  // Leaving Search clears the query; setQuery keeps a go-to-location highlight.
  if (activeTab === "search") searchStore.setQuery("");
  activeTab = tab;
  visit++;
}

const PAGE_TABS: TabDefinition[] = [
  { id: "overview", name: "Overview", icon: "mdi:view-dashboard", description: "System information and statistics" },
  { id: "search", name: "Search", icon: "mdi:magnify", description: "Search tweaks by name, description, or info" },
  { id: "favorites", name: "Favorites", icon: "mdi:star", description: "Quick access to your saved tweaks" },
  {
    id: "snapshots",
    name: "Snapshots",
    icon: "mdi:history",
    description: "View and manage tweaks with saved snapshots",
  },
  { id: "profiles", name: "Profiles", icon: "mdi:file-multiple", description: "Manage saved configuration profiles" },
];

const MANUAL_TESTS_TAB: TabDefinition = {
  id: "manual-tests",
  name: "Manual Tests",
  icon: "mdi:flask-outline",
  description: "Real-machine checks for this test build",
};

const fixedTabs = $derived(manualTestsStore.isAvailable ? [...PAGE_TABS, MANUAL_TESTS_TAB] : PAGE_TABS);

const categoryTabs = $derived(
  categoriesStore.list.map((cat): TabDefinition => ({
    id: cat.id,
    name: cat.name,
    icon: cat.icon,
    description: cat.description,
  })),
);

const allTabs = $derived([...fixedTabs, ...categoryTabs]);

const isOnCategoryTab = $derived(!isPageId(activeTab));

// List pages whose rows the title-bar search can filter in place.
const isScopable = $derived(isOnCategoryTab || activeTab === "favorites" || activeTab === "snapshots");

export const navigationStore = {
  get activeTab() {
    return activeTab;
  },

  get allTabs() {
    return allTabs;
  },

  get fixedTabs() {
    return fixedTabs;
  },

  get categoryTabs() {
    return categoryTabs;
  },

  get isOnCategoryTab() {
    return isOnCategoryTab;
  },

  get isScopable() {
    return isScopable;
  },

  get visit() {
    return visit;
  },

  /** Bumped to ask the title bar search box for focus. */
  get focusSearchSignal() {
    return focusSearchSignal;
  },

  navigateToTab: go,

  /** Opens a category already filtered to its Needs Attention tweaks. */
  navigateToAttention(categoryId: string) {
    go(categoryId);
    attentionVisit = visit;
  },

  /** Read once by the category view as it mounts. */
  takeAttentionFilter(): boolean {
    const requested = attentionVisit === visit;
    attentionVisit = -1;
    return requested;
  },

  /** A list page keeps the search scoped to itself; any other page goes to Search. */
  focusSearch() {
    if (!isScopable) go("search");
    focusSearchSignal++;
  },
};
