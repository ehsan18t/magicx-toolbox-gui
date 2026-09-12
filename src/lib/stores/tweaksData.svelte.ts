/**
 * Tweaks Data Store - Svelte 5 Runes
 *
 * Manages core data for the redesigned engine: system info, the tweak model
 * (`get_tweaks`), categories (derived from the model), and live per-tweak statuses
 * filled in INCREMENTALLY from the `tweak-status` event stream (spec §8.4 grill Q1/Q5).
 * System hardware info is cached in localStorage since it rarely changes.
 */

import * as api from "$lib/api/tweaks";
import type {
  CachedSystemInfo,
  CategoryDefinition,
  CategoryMeta,
  ElevationState,
  RiskLevel,
  SystemInfo,
  TweakDefinition,
  TweakStatus,
  TweakStatusView,
  TweakView,
  TweakWithStatus,
} from "$lib/types";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import type { UnlistenFn } from "@tauri-apps/api/event";

// Storage key for cached hardware info
const SYSTEM_INFO_CACHE_KEY = "magicx-system-info-cache";

// Cached hardware info (static data that rarely changes)
const systemInfoCache = new PersistentStore<CachedSystemInfo | null>(SYSTEM_INFO_CACHE_KEY, null);

// === Loading States ===
let systemInfoLoading = $state(true);
let systemInfoRefreshing = $state(false);
let tweaksLoading = $state(true);
let initialLoadComplete = $state(false);

// === System Info State ===
let systemInfo = $state<SystemInfo | null>(null);

// === Elevation State (app ceiling + over-the-shoulder SID guard) ===
let elevationState = $state<ElevationState | null>(null);

// === Tweaks State ===
let tweaks = $state<TweakWithStatus[]>([]);
let tweaksVersion = $state(0);
// Corpus category metadata from `get_categories` (real names/icons, not derived from ids).
let categoryMeta = $state<CategoryMeta[]>([]);

// === Adapters: engine DTOs -> the presentation model the components consume ===

function mapView(view: TweakView): TweakDefinition {
  return {
    id: view.id,
    name: view.name,
    description: view.description,
    category_id: view.category,
    risk_level: view.risk.toLowerCase() as RiskLevel,
    reversible: view.reversible,
    requires_reboot: view.requires_reboot,
    required_level: view.required_level,
    availability: view.availability,
    optionLabels: view.options.map((o) => o.label),
    options: view.options,
    info: view.info ?? undefined,
  };
}

/** Placeholder status shown until this tweak's first `tweak-status` event arrives. */
function loadingStatus(tweakId: string): TweakStatus {
  return {
    tweak_id: tweakId,
    loaded: false,
    state: "loading",
    activeOption: null,
    unavailableReason: null,
    unknownReasons: [],
    needsElevation: false,
    unavailableOptions: [],
    residues: [],
    heldShared: [],
    observed: null,
    is_applied: false,
    has_backup: false,
    needs_attention: false,
    unrestorable_resources: [],
  };
}

function mapStatusView(tweakId: string, view: TweakStatusView): TweakStatus {
  const s = view.state;
  const unknownReasons = s.state === "unknown" ? s.reasons : [];
  return {
    tweak_id: tweakId,
    loaded: true,
    state: s.state,
    activeOption: s.state === "active" ? s.option : null,
    unavailableReason: s.state === "unavailable" ? s.reason : null,
    unknownReasons,
    needsElevation: unknownReasons.some((r) => r.needs_elevation),
    unavailableOptions: view.unavailable,
    residues: view.residues,
    heldShared: view.held_shared,
    observed: view.observed,
    is_applied: s.state === "active",
    has_backup: view.has_history,
    needs_attention: false,
    unrestorable_resources: [],
  };
}

/**
 * Status views that arrived before their tweak existed in `tweaks`. The scan thread can outrun
 * `loadModel`, and without this the event would be dropped and the card would spin forever.
 */
let pendingStatusViews: Record<string, TweakStatusView> = {};

// Derived: tweaks grouped by category
const tweaksByCategory = $derived.by(() => {
  const byCategory: Record<string, TweakWithStatus[]> = {};
  for (const cat of categories) {
    byCategory[cat.id] = [];
  }
  for (const tweak of tweaks) {
    const categoryId = tweak.definition.category_id;
    if (byCategory[categoryId]) {
      byCategory[categoryId].push(tweak);
    }
  }
  return byCategory;
});

// Derived: categories. Prefer real corpus metadata from `get_categories` (name/icon/description
// in corpus order); fall back to ids discovered from the tweak model until that load resolves.
const categories = $derived.by((): CategoryDefinition[] => {
  if (categoryMeta.length > 0) {
    return categoryMeta.map((c, i) => ({
      id: c.id,
      name: c.name,
      description: c.description,
      icon: c.icon || "mdi:folder",
      order: i,
    }));
  }
  const seen: Record<string, true> = {};
  const list: CategoryDefinition[] = [];
  for (const tweak of tweaks) {
    const id = tweak.definition.category_id;
    if (!seen[id]) {
      seen[id] = true;
      list.push({ id, name: id, description: "", icon: "mdi:folder", order: list.length });
    }
  }
  return list;
});

// Derived: overall stats
const stats = $derived({
  total: tweaks.length,
  applied: tweaks.filter((t) => t.status.is_applied).length,
  pending: tweaks.filter((t) => !t.status.is_applied).length,
});

// Derived: stats per category
const categoryStats = $derived.by(() => {
  const result: Record<string, { total: number; applied: number }> = {};
  for (const cat of categories) {
    const catTweaks = tweaksByCategory[cat.id] || [];
    result[cat.id] = {
      total: catTweaks.length,
      applied: catTweaks.filter((t) => t.status.is_applied).length,
    };
  }
  return result;
});

// Derived: cache timestamp for display
const cacheTimestamp = $derived(systemInfoCache.value?.cachedAt ?? null);

// === Helper Functions ===

/** Update the cache with hardware info from fresh system info */
function updateCache(info: SystemInfo): void {
  systemInfoCache.value = {
    hardware: info.hardware,
    device: info.device,
    computer_name: info.computer_name,
    cachedAt: new Date().toISOString(),
  };
}

/** Build a SystemInfo object using cached hardware data and fresh dynamic data */
function buildSystemInfoFromCache(cache: CachedSystemInfo, freshInfo: SystemInfo): SystemInfo {
  return {
    windows: freshInfo.windows,
    username: freshInfo.username,
    is_admin: freshInfo.is_admin,
    hardware: cache.hardware,
    device: cache.device,
    computer_name: cache.computer_name,
  };
}

// === Exports ===

export const systemStore = {
  get info() {
    return systemInfo;
  },

  get isLoading() {
    return systemInfoLoading;
  },

  /** Whether a manual refresh is in progress */
  get isRefreshing() {
    return systemInfoRefreshing;
  },

  /** ISO timestamp of when hardware info was last cached */
  get cachedAt() {
    return cacheTimestamp;
  },

  /**
   * Load system info, using cache for hardware if available.
   * On first load with no cache, fetches everything fresh.
   * On subsequent loads, uses cached hardware + fresh dynamic info.
   */
  async load() {
    systemInfoLoading = true;
    try {
      const cached = systemInfoCache.value;
      const freshInfo = await api.getSystemInfo();

      if (cached) {
        systemInfo = buildSystemInfoFromCache(cached, freshInfo);
      } else {
        systemInfo = freshInfo;
        updateCache(freshInfo);
      }

      return systemInfo;
    } catch (error) {
      console.error("Failed to load system info:", error);
      const cached = systemInfoCache.value;
      if (cached) {
        systemInfo = {
          windows: {
            version_string: "",
            display_version: "",
            build_number: "",
            product_name: "Windows",
            uptime_seconds: 0,
            is_windows_11: false,
            is_windows_server: false,
            install_date: null,
          },
          username: "",
          is_admin: false,
          hardware: cached.hardware,
          device: cached.device,
          computer_name: cached.computer_name,
        };
        return systemInfo;
      }
      return null;
    } finally {
      systemInfoLoading = false;
    }
  },

  /**
   * Force refresh all system info including hardware (ignores cache).
   * Use this when user wants to refresh hardware info.
   */
  async refresh() {
    systemInfoRefreshing = true;
    try {
      const freshInfo = await api.getSystemInfo();
      systemInfo = freshInfo;
      updateCache(freshInfo);
      return freshInfo;
    } catch (error) {
      console.error("Failed to refresh system info:", error);
      throw error;
    } finally {
      systemInfoRefreshing = false;
    }
  },

  /** Clear the cache (useful for debugging) */
  clearCache() {
    systemInfoCache.value = null;
  },
};

/**
 * App elevation ceiling + SID guard (spec §9). `level` drives the elevate affordance; `sidMismatch`
 * means "per-user tweaks are blocked" without saying why (see `ElevationState`) and currently has no
 * consumer, since the per-tweak `availability` carries the reason the UI actually shows.
 */
export const elevationStore = {
  get state() {
    return elevationState;
  },
  get level() {
    return elevationState?.level ?? "User";
  },
  get sidMismatch() {
    return elevationState?.sid_mismatch ?? false;
  },
  async load() {
    try {
      elevationState = await api.getElevationState();
    } catch (error) {
      console.error("Failed to load elevation state:", error);
    }
    return elevationState;
  },
};

export const categoriesStore = {
  get list() {
    return categories;
  },

  get isLoading() {
    return tweaksLoading;
  },

  /** Get category by ID */
  getById(categoryId: string): CategoryDefinition | undefined {
    return categories.find((c) => c.id === categoryId);
  },

  /** Get category name by ID, returns the ID if not found */
  getName(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.name ?? categoryId;
  },

  /** Get category icon by ID, returns default folder icon if not found */
  getIcon(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.icon ?? "mdi:folder";
  },
};

export const tweaksStore = {
  get list() {
    return tweaks;
  },

  get byCategory() {
    return tweaksByCategory;
  },

  get stats() {
    return stats;
  },

  get isLoading() {
    return tweaksLoading;
  },

  /** Load the compiled tweak model (`get_tweaks`); statuses arrive later via the stream. */
  async loadModel() {
    tweaksLoading = true;
    try {
      const [views, cats] = await Promise.all([api.getTweaks(), api.getCategories()]);
      categoryMeta = cats;
      tweaks = views.map((v) => {
        const early = pendingStatusViews[v.id];
        return {
          definition: mapView(v),
          status: early ? mapStatusView(v.id, early) : loadingStatus(v.id),
        };
      });
      pendingStatusViews = {};
      tweaksVersion++;
      return tweaks;
    } catch (error) {
      console.error("Failed to load tweaks:", error);
      // Surface the error — the app cannot function without the tweak model.
      throw error;
    } finally {
      tweaksLoading = false;
    }
  },

  /** Replace a tweak's status from a freshly detected engine status view. */
  setStatusView(tweakId: string, view: TweakStatusView) {
    if (!tweaks.some((t) => t.definition.id === tweakId)) {
      pendingStatusViews[tweakId] = view;
      return;
    }
    tweaks = tweaks.map((t) => (t.definition.id === tweakId ? { ...t, status: mapStatusView(tweakId, view) } : t));
  },

  /** Patch selected status fields (needs-attention / has_backup after restore/discard). */
  patchStatus(tweakId: string, patch: Partial<TweakStatus>) {
    tweaks = tweaks.map((t) => (t.definition.id === tweakId ? { ...t, status: { ...t.status, ...patch } } : t));
  },

  /** Get a tweak by ID */
  getById(tweakId: string): TweakWithStatus | undefined {
    return tweaks.find((t) => t.definition.id === tweakId);
  },

  get version() {
    return tweaksVersion;
  },
};

/** Category stats getter - exposed separately for components that need it */
export const getCategoryStats = () => categoryStats;

/** Loading state store for progressive loading */
export const loadingStateStore = {
  get systemInfoLoading() {
    return systemInfoLoading;
  },
  get systemInfoRefreshing() {
    return systemInfoRefreshing;
  },
  get categoriesLoading() {
    return tweaksLoading;
  },
  get tweaksLoading() {
    return tweaksLoading;
  },
  get initialLoadComplete() {
    return initialLoadComplete;
  },
};

// === Status event stream (incremental, spec §8.4 grill Q1/Q5) ===

let statusStreamStarted = false;
let unlistenStatus: UnlistenFn | null = null;

/**
 * Register the `tweak-status` listener once, then kick the background scan. Each event
 * fills in one tweak's status as the backend detects it — never awaiting one bulk result.
 */
async function startStatusStream(): Promise<void> {
  if (statusStreamStarted) {
    await api.getStatusesStream();
    return;
  }
  statusStreamStarted = true;
  // Register BEFORE kicking the scan so no early event is missed.
  unlistenStatus = await api.onTweakStatus((event) => {
    tweaksStore.setStatusView(event.tweak_id, event.status);
  });
  await api.getStatusesStream();
}

/**
 * Re-run the full scan after an elevation change so Unknowns become readable
 * (the listener is already registered by `startStatusStream`).
 */
export async function rescanStatuses(): Promise<void> {
  await elevationStore.load();
  await api.rescanAfterElevation();
}

// Promise cache for deduplicating concurrent initialization calls
let quickInitPromise: Promise<void> | null = null;
let remainingDataPromise: Promise<void> | null = null;

/**
 * Quick initialize - load the tweak model so cards + categories render immediately.
 * Call loadRemainingData() after to load system info and start the status stream.
 */
export async function initializeQuick(): Promise<void> {
  if (quickInitPromise) {
    return quickInitPromise;
  }
  if (tweaks.length > 0) {
    return;
  }

  quickInitPromise = tweaksStore
    .loadModel()
    .then(() => {
      // Discard result to match Promise<void> signature
    })
    .finally(() => {
      quickInitPromise = null;
    });

  return quickInitPromise;
}

/**
 * Load remaining data after quick init: system info, elevation state, and the
 * background-progressive status stream (statuses fill in incrementally).
 *
 * The model load is awaited first, not assumed: Svelte mounts `+page` before `+layout`, so this
 * runs before the layout's `initializeQuick()` and would otherwise kick the scan against an empty
 * `tweaks` array (the call is promise-cached, so it joins the layout's load rather than repeating it).
 */
export async function loadRemainingData(): Promise<void> {
  if (remainingDataPromise) {
    return remainingDataPromise;
  }
  if (initialLoadComplete) {
    return;
  }

  remainingDataPromise = initializeQuick()
    .then(() => Promise.all([systemStore.load(), elevationStore.load(), startStatusStream()]))
    .then(() => {
      initialLoadComplete = true;
    })
    .finally(() => {
      remainingDataPromise = null;
    });

  return remainingDataPromise;
}

/** Stop listening to the status stream (app-lifetime; exposed for completeness). */
export function stopStatusStream(): void {
  unlistenStatus?.();
  unlistenStatus = null;
  statusStreamStarted = false;
}
