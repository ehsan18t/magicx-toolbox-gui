// Fuzzy search over tweaks and apps (uFuzzy): out-of-order terms, `-term` exclusion, per-field highlights.

import type { ItemKind } from "$lib/types";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import uFuzzy from "@leeoniya/ufuzzy";
import { untrack } from "svelte";
import { appsStore } from "./apps.svelte";
import { settingsStore } from "./settings.svelte";
import { tweaksStore } from "./tweaksData.svelte";

export interface SearchResult {
  /** Restore on the results page reads tweak results alone. */
  kind: ItemKind;
  /** Unique across tweaks and apps. */
  id: string;
  categoryId: string;
  /** Highlight ranges as flat [start, end, start, end, …] pairs, per field. */
  nameRanges: number[];
  descriptionRanges: number[];
  infoRanges: number[];
}

interface HaystackEntry {
  kind: SearchResult["kind"];
  id: string;
  categoryId: string;
  nameEnd: number;
  descEnd: number;
}

/** Joins name, description and info into one searchable string. */
const FIELD_SEP = " | ";
/** Permutations tried for out-of-order terms (2 = 2!). */
const OUT_OF_ORDER = 2;
/** Matches ranked and highlighted; beyond this uFuzzy only filters. */
const INFO_THRESHOLD = 1000;
const MAX_RESULTS = 100;
const DEBOUNCE_MS = 200;

const uf = new uFuzzy({
  // SingleError: tolerates one typo per term, e.g. "telmetry".
  intraMode: 1,
  intraIns: 1,
  intraSub: 1,
  intraTrn: 1,
  intraDel: 1,
  // Terms start at word boundaries but may end anywhere.
  interLft: 1,
  interRgt: 0,
});

let query = $state("");
let searchedQuery = $state("");
// Bumped by search(), so a retry re-runs the same query.
let runs = $state(0);
let highlightTweakId = $state<string | null>(null);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Keyed on the model versions, not the lists: status events replace the tweak list but never its text.
const haystack = $derived.by(() => {
  void [tweaksStore.version, appsStore.version, settingsStore.showUnsupported];
  return untrack(() => {
    const items = [
      ...tweaksStore.list.map(({ definition: d }) => ({
        kind: "tweak" as const,
        id: d.id,
        categoryId: d.categoryId,
        name: d.name,
        description: d.description || "",
        info: d.info || "",
      })),
      ...appsStore.list.map((a) => ({
        kind: "app" as const,
        id: a.id,
        categoryId: a.category,
        name: a.name,
        description: a.description || "",
        info: a.info || "",
      })),
    ];
    const strings: string[] = [];
    const entries: HaystackEntry[] = [];
    for (const { name, description, info, ...item } of items) {
      strings.push([name, description, info].join(FIELD_SEP));
      entries.push({ ...item, nameEnd: name.length, descEnd: name.length + FIELD_SEP.length + description.length });
    }
    return { strings, entries };
  });
});

/** Splits ranges over the joined string back into per-field ranges. */
function fieldRanges(ranges: number[], entry: HaystackEntry) {
  const nameRanges: number[] = [];
  const descriptionRanges: number[] = [];
  const infoRanges: number[] = [];

  const { nameEnd, descEnd } = entry;
  const descStart = nameEnd + FIELD_SEP.length;
  const infoStart = descEnd + FIELD_SEP.length;

  for (let i = 0; i < ranges.length; i += 2) {
    const start = ranges[i];
    const end = ranges[i + 1];
    if (end <= nameEnd) {
      nameRanges.push(start, end);
    } else if (start >= infoStart) {
      infoRanges.push(start - infoStart, end - infoStart);
    } else if (start >= descStart && end <= descEnd) {
      descriptionRanges.push(start - descStart, end - descStart);
    } else {
      if (start < nameEnd) nameRanges.push(start, Math.min(end, nameEnd));
      if (start < descEnd && end > descStart) {
        descriptionRanges.push(Math.max(0, start - descStart), Math.min(end - descStart, descEnd - descStart));
      }
      if (end > infoStart) infoRanges.push(Math.max(0, start - infoStart), end - infoStart);
    }
  }
  return { nameRanges, descriptionRanges, infoRanges };
}

function toResult(idx: number, ranges: number[]): SearchResult {
  const entry = haystack.entries[idx];
  return {
    kind: entry.kind,
    id: entry.id,
    categoryId: entry.categoryId,
    ...fieldRanges(ranges, entry),
  };
}

/** Every fuzzy match, unranked and uncapped, keyed by id: the in-page filter keeps its own order. */
export function fuzzyMatches(needle: string): Record<string, SearchResult> {
  const q = needle.trim();
  if (!q) return {};
  const { strings } = haystack;
  const [idxs, info] = uf.search(strings, q, OUT_OF_ORDER, strings.length);
  // A multi-term miss returns an info object with no idx, so the empty case must return first.
  if (!idxs || idxs.length === 0) return {};
  const hits = info?.idx
    ? info.idx.map((idx, i) => toResult(idx, info.ranges[i] ?? []))
    : idxs.map((idx) => toResult(idx, []));
  return Object.fromEntries(hits.map((hit) => [hit.id, hit]));
}

function rank(needle: string): SearchResult[] {
  const { strings } = haystack;
  if (strings.length === 0) return [];
  const [idxs, info, order] = uf.search(strings, needle, OUT_OF_ORDER, INFO_THRESHOLD);
  if (!idxs || idxs.length === 0) return [];
  if (info && order) {
    return order.slice(0, MAX_RESULTS).map((i) => toResult(info.idx[i], info.ranges[i] || []));
  }
  // Past INFO_THRESHOLD: filtered, unranked, no highlights.
  return idxs.slice(0, MAX_RESULTS).map((idx) => toResult(idx, []));
}

const outcome = $derived.by((): { results: SearchResult[]; error: string | null } => {
  void runs;
  if (!searchedQuery) return { results: [], error: null };
  try {
    return { results: rank(searchedQuery), error: null };
  } catch (e) {
    logError("[search] Search failed", e);
    return { results: [], error: errorMessage(e) };
  }
});

const isActive = $derived(query.trim().length > 0);

function search() {
  searchedQuery = query.trim();
  runs++;
}

export const searchStore = {
  get query() {
    return query;
  },

  /** Recomputed when the query is searched or the model reloads. */
  get results() {
    return outcome.results;
  },

  /** The query the current results answer; lags `query` while a search is debounced. */
  get searchedQuery() {
    return searchedQuery;
  },

  get error() {
    return outcome.error;
  },

  get isActive() {
    return isActive;
  },

  get highlightTweakId() {
    return highlightTweakId;
  },

  /** Searches after a debounce; an empty query clears at once. */
  setQuery(newQuery: string) {
    query = newQuery;
    const trimmed = newQuery.trim();

    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    if (!trimmed) {
      searchedQuery = "";
      return;
    }
    if (trimmed === searchedQuery) return;

    debounceTimer = setTimeout(() => {
      debounceTimer = null;
      search();
    }, DEBOUNCE_MS);
  },

  /** Searches the current query now. */
  search,

  setHighlight(tweakId: string | null) {
    highlightTweakId = tweakId;
  },

  clearHighlight() {
    highlightTweakId = null;
  },
};
