import { navigationStore, type TabId } from "./navigation.svelte";
import { type SearchResult, searchStore } from "./search.svelte";

// Keyed by page visit, so every visit opens unfiltered and scoped with no effect to reset it.
let filter = $state({ visit: -1, query: "", scoped: true });
// The list page a global search left from, valid only while the Search page it opened is showing.
let origin = $state<{ tab: TabId; visit: number } | null>(null);

const isCurrent = () => filter.visit === navigationStore.visit;
const set = (query: string, scoped: boolean) => (filter = { visit: navigationStore.visit, query, scoped });

const scoped = $derived(navigationStore.isScopable && (!isCurrent() || filter.scoped));
const query = $derived(isCurrent() ? filter.query : "");
const matches = $derived(scoped && query.trim() ? searchStore.fuzzyMatches(query) : null);
const originTab = $derived(
  origin && origin.visit === navigationStore.visit && navigationStore.activeTab === "search" ? origin.tab : null,
);

/** The title-bar search while it is scoped to the current list page. */
export const pageFilterStore = {
  get isScoped(): boolean {
    return scoped;
  },
  get query(): string {
    return query;
  },
  /** The page the scope chip names: the current list page, or the one a global search came from. */
  get chipTab(): TabId | null {
    return navigationStore.isScopable ? navigationStore.activeTab : originTab;
  },
  /** Null while unfiltered; otherwise only matching ids are present, with their highlight ranges. */
  match(id: string): SearchResult | null | undefined {
    return matches === null ? null : Object.hasOwn(matches, id) ? matches[id] : undefined;
  },
  passes(id: string): boolean {
    return matches === null || Object.hasOwn(matches, id);
  },
  setQuery(text: string) {
    set(text, true);
  },
  /** Scope on: here it filters in place; on the Search page it returns to the origin with the same text. */
  scopeIn() {
    if (navigationStore.isScopable) {
      set("", true);
      return;
    }
    if (!originTab) return;
    const text = searchStore.query;
    navigationStore.navigateToTab(originTab);
    set(text, true);
  },
  /** Scope off, carrying typed text to the Search page and remembering this page for scopeIn. */
  searchEverywhere(text = query) {
    const from = navigationStore.activeTab;
    set("", false);
    if (!text.trim()) return;
    searchStore.setQuery(text);
    navigationStore.navigateToTab("search");
    origin = { tab: from, visit: navigationStore.visit };
  },
};
