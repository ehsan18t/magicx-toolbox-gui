import type { IconName } from "$lib/design";
import type { TweakWithStatus } from "$lib/types";
import { tick } from "svelte";
import { manualTestsStore } from "./manualTests.svelte";
import { searchStore } from "./search.svelte";
import { categoriesStore, tweaksStore } from "./tweaksData.svelte";

const PAGE_IDS = ["overview", "search", "favorites", "snapshots", "profiles", "settings", "manual-tests"] as const;

export type PageId = (typeof PAGE_IDS)[number];

/** A fixed page, or a category id (any string the corpus defines). */
export type TabId = PageId | (string & {});

export interface TabDefinition<Id extends TabId = TabId> {
  id: Id;
  name: string;
  icon: IconName;
  description?: string;
}

export const isPageId = (tab: TabId): tab is PageId => (PAGE_IDS as readonly string[]).includes(tab);

/** The tweaks a list page shows, null for a page that lists none: the one source for page lists and counts. */
export function pageTweaks(tab: TabId): TweakWithStatus[] | null {
  if (tab === "favorites") return tweaksStore.favorites;
  if (tab === "snapshots") return tweaksStore.withSnapshot;
  return isPageId(tab) ? null : (tweaksStore.byCategory[tab] ?? []);
}

let activeTab = $state<TabId>("overview");
let focusSearchInput: (() => void) | null = null;
// Bumped on every page change, so per-page UI state can tell a revisit from staying put.
let visit = $state(0);
let attentionVisit = -1;
let highlight: { id: string; visit: number } | null = null;

function go(tab: TabId) {
  if (tab === activeTab) return;
  // Leaving Search clears its query.
  if (activeTab === "search") searchStore.setQuery("");
  activeTab = tab;
  visit++;
}

const PAGE_TABS: TabDefinition<PageId>[] = [
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

const MANUAL_TESTS_TAB: TabDefinition<PageId> = {
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
const isScopable = $derived(pageTweaks(activeTab) !== null);

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

  navigateToPage(page: PageId) {
    go(page);
  },

  navigateToCategory(categoryId: string) {
    go(categoryId);
  },

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

  /** Opens the item's category; its row scrolls into view and flashes once. */
  navigateToItem(categoryId: string, itemId: string) {
    go(categoryId);
    highlight = { id: itemId, visit };
  },

  /** Read once by each row as it mounts; true only for the target row on the visit that asked. */
  takeHighlight(itemId: string): boolean {
    if (highlight?.id !== itemId) return false;
    const current = highlight.visit === visit;
    highlight = null;
    return current;
  },

  /** The title-bar search box registers how to focus it; returns the unregister. */
  registerSearchFocus(focus: () => void): () => void {
    focusSearchInput = focus;
    return () => {
      if (focusSearchInput === focus) focusSearchInput = null;
    };
  },

  /** A list page keeps the search scoped to itself; any other page goes to Search. */
  focusSearch() {
    if (!isScopable) go("search");
    void tick().then(() => focusSearchInput?.());
  },
};
