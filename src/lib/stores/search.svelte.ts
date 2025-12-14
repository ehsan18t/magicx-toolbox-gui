/**
 * Search Store - Svelte 5 Runes
 *
 * Manages fuzzy search state with result caching using fuzzysort.
 * Results only update when query changes or user explicitly clears.
 */

import { tweaksStore } from "$lib/stores/tweaks.svelte";

import fuzzysort from "fuzzysort";

/** A search result from fuzzy search */
export interface SearchResult {
  /** The tweak ID */
  tweak_id: string;
  /** Fuzzy match score (higher is better) */
  score: number;
  /** Category ID for navigation */
  category_id: string;
  /** Match indices (compatibility field, unused in UI currently) */
  match_indices: number[];
}

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
      // Get tweaks from store (already loaded in memory)
      const tweaks = tweaksStore.list;

      // Perform fuzzy search
      // Search across name, description, and info
      const searchResults = fuzzysort.go(currentQuery, tweaks, {
        keys: ["definition.name", "definition.description", "definition.info"],
        threshold: -10000, // Don't return bad matches
        limit: 100, // Limit results for performance
      });

      // Map to SearchResult interface
      results = searchResults.map((res) => ({
        tweak_id: res.obj.definition.id,
        score: res.score,
        category_id: res.obj.definition.category_id,
        match_indices: [], // Not currently used by UI
      }));

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
