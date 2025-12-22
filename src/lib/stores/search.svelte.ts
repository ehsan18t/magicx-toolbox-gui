/**
 * Search Store - Svelte 5 Runes
 *
 * High-performance fuzzy search using uFuzzy.
 * Features:
 * - Out-of-order term matching (e.g., "telemetry disable" matches "disable_telemetry")
 * - Term exclusion support (e.g., "privacy -feedback")
 * - Match highlighting with character ranges
 * - Efficient haystack preparation (search strings cached)
 * - Single uFuzzy instance (no memory leaks)
 * - Result caching to avoid redundant searches
 */

import { tweaksStore } from "$lib/stores/tweaks.svelte";
import type { TweakWithStatus } from "$lib/types";
import uFuzzy from "@leeoniya/ufuzzy";

// === Types ===

/** A search result with match information */
export interface SearchResult {
  /** The tweak ID */
  tweakId: string;
  /** Category ID for navigation */
  categoryId: string;
  /** The original haystack index */
  haystackIndex: number;
  /** Highlight ranges for name field: [start, end, start, end, ...] */
  nameRanges: number[];
  /** Highlight ranges for description field */
  descriptionRanges: number[];
  /** Highlight ranges for info field */
  infoRanges: number[];
}

/** Prepared haystack entry for efficient searching */
interface HaystackEntry {
  /** Combined searchable text: "name | description | info" */
  searchText: string;
  /** Original tweak reference */
  tweak: TweakWithStatus;
  /** Pre-computed field boundaries for highlight extraction */
  nameEnd: number;
  descEnd: number;
}

// === uFuzzy Configuration ===

/**
 * Create uFuzzy instance with optimal settings for tweak search:
 * - intraMode: 1 (SingleError) - tolerate typos like "telmetry" -> "telemetry"
 * - interLft/interRgt: 1 (Loose) - match at word boundaries for better relevance
 */
const uf = new uFuzzy({
  intraMode: 1, // Allow single-char typos
  intraIns: 1, // Allow 1 char insertion within terms
  intraSub: 1, // Allow 1 substitution
  intraTrn: 1, // Allow 1 transposition
  intraDel: 1, // Allow 1 deletion
  interLft: 1, // Loose left boundary (word starts)
  interRgt: 0, // Any right boundary (allow partial matches)
});

// === State ===

/** Current search query (user input) */
let query = $state("");

/** Cached search results */
let results = $state<SearchResult[]>([]);

/** Whether a search is in progress */
let isSearching = $state(false);

/** Error message if search failed */
let error = $state<string | null>(null);

/** Query that produced current cached results */
let cachedQuery = "";

/** Tweak ID to highlight after navigation */
let highlightTweakId = $state<string | null>(null);

/** Debounce timer for search queries */
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

/** Debounce delay in milliseconds */
const DEBOUNCE_DELAY = 200;

// === Haystack Cache ===

/** Cached haystack data - rebuilt when tweaks change */
let haystackCache: {
  /** Array of searchable strings for uFuzzy */
  strings: string[];
  /** Parallel array of metadata */
  entries: HaystackEntry[];
  /** Tweaks list hash to detect changes */
  tweaksHash: string;
} | null = null;

/**
 * Build or retrieve cached haystack from current tweaks.
 * The haystack is an array of strings combining name, description, and info
 * with separators that allow us to extract highlight ranges per field.
 */
function getHaystack(): { strings: string[]; entries: HaystackEntry[] } {
  const tweaks = tweaksStore.list;

  // Create a composite hash of length + version to detect any changes
  // version increments on status updates or reloads
  // Use string concatenation to avoid arithmetic collisions (e.g., 10+5 vs 11+4)
  const tweaksHash = `${tweaks.length}-${tweaksStore.version}`;

  // Return cached if still valid
  if (haystackCache && haystackCache.tweaksHash === tweaksHash) {
    return haystackCache;
  }

  // Build new haystack
  const strings: string[] = [];
  const entries: HaystackEntry[] = [];

  for (const tweak of tweaks) {
    const name = tweak.definition.name;
    const description = tweak.definition.description || "";
    const info = tweak.definition.info || "";

    // Combine fields with separator for single-string search
    // Format: "name | description | info"
    const searchText = `${name} | ${description} | ${info}`;

    strings.push(searchText);
    entries.push({
      searchText,
      tweak,
      nameEnd: name.length,
      descEnd: name.length + 3 + description.length, // +3 for " | "
    });
  }

  haystackCache = { strings, entries, tweaksHash };
  return haystackCache;
}

/**
 * Extract per-field highlight ranges from combined string ranges.
 * uFuzzy returns ranges for the combined "name | description | info" string,
 * we need to split them back into individual field ranges.
 */
function extractFieldRanges(
  ranges: number[],
  entry: HaystackEntry,
): { nameRanges: number[]; descriptionRanges: number[]; infoRanges: number[] } {
  const nameRanges: number[] = [];
  const descriptionRanges: number[] = [];
  const infoRanges: number[] = [];

  const nameEnd = entry.nameEnd;
  const descStart = nameEnd + 3; // After " | "
  const descEnd = entry.descEnd;
  const infoStart = descEnd + 3; // After second " | "

  // Process range pairs [start, end, start, end, ...]
  for (let i = 0; i < ranges.length; i += 2) {
    const start = ranges[i];
    const end = ranges[i + 1];

    // Check which field(s) this range overlaps
    if (end <= nameEnd) {
      // Entirely in name
      nameRanges.push(start, end);
    } else if (start >= infoStart) {
      // Entirely in info
      infoRanges.push(start - infoStart, end - infoStart);
    } else if (start >= descStart && end <= descEnd) {
      // Entirely in description
      descriptionRanges.push(start - descStart, end - descStart);
    } else {
      // Range spans multiple fields - split it
      if (start < nameEnd) {
        nameRanges.push(start, Math.min(end, nameEnd));
      }
      if (start < descEnd && end > descStart) {
        descriptionRanges.push(Math.max(0, start - descStart), Math.min(end - descStart, descEnd - descStart));
      }
      if (end > infoStart) {
        infoRanges.push(Math.max(0, start - infoStart), end - infoStart);
      }
    }
  }

  return { nameRanges, descriptionRanges, infoRanges };
}

// === Derived State ===

const hasResults = $derived(results.length > 0);
const isActive = $derived(query.trim().length > 0);
const resultCount = $derived(results.length);

// === Export ===

export const searchStore = {
  // --- Getters ---

  get query() {
    return query;
  },

  get results() {
    return results;
  },

  get isSearching() {
    return isSearching;
  },

  get error() {
    return error;
  },

  get hasResults() {
    return hasResults;
  },

  get isActive() {
    return isActive;
  },

  get resultCount() {
    return resultCount;
  },

  get highlightTweakId() {
    return highlightTweakId;
  },

  // --- Actions ---

  /**
   * Set search query and trigger debounced search.
   * @param newQuery - The search query string
   */
  setQuery(newQuery: string) {
    query = newQuery;
    const trimmed = newQuery.trim();

    // Clear any pending debounced search
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }

    // Clear for empty query immediately (no debounce needed)
    if (!trimmed) {
      results = [];
      cachedQuery = "";
      error = null;
      return;
    }

    // Skip if query unchanged
    if (trimmed === cachedQuery) {
      return;
    }

    // Debounce search execution to reduce CPU usage during rapid typing
    debounceTimer = setTimeout(() => {
      debounceTimer = null;
      this.search();
    }, DEBOUNCE_DELAY);
  },

  /**
   * Execute fuzzy search with current query.
   * Uses uFuzzy's integrated search() API with out-of-order matching.
   */
  search() {
    const needle = query.trim();

    if (!needle) {
      results = [];
      cachedQuery = "";
      return;
    }

    isSearching = true;
    error = null;

    try {
      const { strings, entries } = getHaystack();

      if (strings.length === 0) {
        results = [];
        cachedQuery = needle;
        isSearching = false;
        return;
      }

      // Use integrated search with out-of-order support
      // outOfOrder = 2 means up to 2! = 2 permutations (reasonable for most queries)
      // infoThresh = 1000 means rank/sort up to 1000 results
      const [idxs, info, order] = uf.search(strings, needle, 2, 1000);

      // Handle no results
      if (!idxs || idxs.length === 0) {
        results = [];
        cachedQuery = needle;
        isSearching = false;
        return;
      }

      // Build results with highlight ranges
      const searchResults: SearchResult[] = [];

      if (info && order) {
        // We have ranked results with highlight info
        for (let i = 0; i < order.length && i < 100; i++) {
          const infoIdx = order[i];
          const haystackIdx = info.idx[infoIdx];
          const entry = entries[haystackIdx];
          const ranges = info.ranges[infoIdx] || [];

          const fieldRanges = extractFieldRanges(ranges, entry);

          searchResults.push({
            tweakId: entry.tweak.definition.id,
            categoryId: entry.tweak.definition.category_id,
            haystackIndex: haystackIdx,
            ...fieldRanges,
          });
        }
      } else {
        // Only filtered results (no ranking) - rare case for very large result sets
        for (let i = 0; i < Math.min(idxs.length, 100); i++) {
          const haystackIdx = idxs[i];
          const entry = entries[haystackIdx];

          searchResults.push({
            tweakId: entry.tweak.definition.id,
            categoryId: entry.tweak.definition.category_id,
            haystackIndex: haystackIdx,
            nameRanges: [],
            descriptionRanges: [],
            infoRanges: [],
          });
        }
      }

      results = searchResults;
      cachedQuery = needle;
    } catch (e) {
      error = e instanceof Error ? e.message : "Search failed";
      console.error("[search] Search failed:", e);
    } finally {
      isSearching = false;
    }
  },

  /**
   * Clear search state completely.
   */
  clear() {
    query = "";
    results = [];
    cachedQuery = "";
    error = null;
    highlightTweakId = null;
  },

  /**
   * Set tweak ID to highlight (for navigation animation).
   */
  setHighlight(tweakId: string | null) {
    highlightTweakId = tweakId;
  },

  /**
   * Clear highlight state.
   */
  clearHighlight() {
    highlightTweakId = null;
  },

  /**
   * Invalidate haystack cache (call when tweaks are modified).
   */
  invalidateCache() {
    haystackCache = null;
  },
};
