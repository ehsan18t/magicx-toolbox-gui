import { navigationStore } from "./navigation.svelte";
import { searchStore } from "./search.svelte";

// Keyed by page visit, so every visit opens unfiltered and scoped with no effect to reset it.
let state = $state({ visit: -1, query: "", scoped: true });
const isCurrent = () => state.visit === navigationStore.visit;
const set = (query: string, scoped: boolean) => (state = { visit: navigationStore.visit, query, scoped });

/** The title-bar search while it is scoped to the current list page. */
export const pageFilterStore = {
  get scoped(): boolean {
    return navigationStore.isScopable && (!isCurrent() || state.scoped);
  },
  get query(): string {
    return isCurrent() ? state.query : "";
  },
  setQuery(query: string) {
    set(query, true);
  },
  rescope() {
    set("", true);
  },
  /** Drops the scope, carrying any typed text over to the global search. */
  searchEverywhere() {
    const query = (isCurrent() ? state.query : "").trim();
    set("", false);
    if (!query) return;
    searchStore.setQuery(query);
    navigationStore.navigateToSearch();
  },
};
