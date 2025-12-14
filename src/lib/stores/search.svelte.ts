/**
 * Search Store - Svelte 5 Runes
 *
 * Manages fuzzy search state with result caching.
 * Results only update when query changes or user explicitly clears.
 */

import { fuzzySearchTweaks, type SearchResult } from "$lib/api/tweaks";

// === State ===

/** Current search query */
let query = $state<string>("");

/** Cached search results */
let results = $state<SearchResult[]>([]);

/** Whether a search is in progress */
let isSearching = $state(false);

/** Error message if search failed */
let error = $state<string | null>(null);

/** The query that produced the current results (for cache comparison) */
let cachedQuery = $state<string>("");

/** Tweak ID to highlight after navigation */
let highlightTweakId = $state<string | null>(null);

// === Derived ===

/** Whether there are any results */
const hasResults = $derived(results.length > 0);

/** Whether search is active (has a query) */
const isActive = $derived(query.trim().length > 0);

/** Result count */
const resultCount = $derived(results.length);

// === Export ===

export const searchStore = {
  /** Current search query */
  get query() {
    return query;
  },

  /** Search results */
  get results() {
    return results;
  },

  /** Whether searching is in progress */
  get isSearching() {
    return isSearching;
  },

  /** Error message if any */
  get error() {
    return error;
  },

  /** Whether there are results */
  get hasResults() {
    return hasResults;
  },

  /** Whether search is active */
  get isActive() {
    return isActive;
  },

  /** Result count */
  get resultCount() {
    return resultCount;
  },

  /** Tweak ID to highlight */
  get highlightTweakId() {
    return highlightTweakId;
  },

  /**
   * Set the search query and trigger search if changed
   * Uses caching - only searches if query actually changed
   */
  async setQuery(newQuery: string) {
    query = newQuery;

    // Don't search if query hasn't changed
    if (newQuery === cachedQuery) {
      return;
    }

    // Clear results for empty query
    if (!newQuery.trim()) {
      results = [];
      cachedQuery = "";
      error = null;
      return;
    }

    // Perform search
    await this.search();
  },

  /**
   * Force a new search with current query
   */
  async search() {
    const currentQuery = query.trim();

    if (!currentQuery) {
      results = [];
      cachedQuery = "";
      return;
    }

    isSearching = true;
    error = null;

    try {
      const searchResults = await fuzzySearchTweaks(currentQuery);
      results = searchResults;
      cachedQuery = currentQuery;
    } catch (e) {
      error = e instanceof Error ? e.message : "Search failed";
      console.error("Search failed:", e);
    } finally {
      isSearching = false;
    }
  },

  /**
   * Clear search query and results
   */
  clear() {
    query = "";
    results = [];
    cachedQuery = "";
    error = null;
    highlightTweakId = null;
  },

  /**
   * Set a tweak ID to highlight (for navigation from search)
   */
  setHighlight(tweakId: string | null) {
    highlightTweakId = tweakId;
  },

  /**
   * Clear the highlight
   */
  clearHighlight() {
    highlightTweakId = null;
  },
};
