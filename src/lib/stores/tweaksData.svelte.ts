// System info, the tweak model (`get_tweaks`), its categories, and live per-tweak statuses filled in
// incrementally from the `tweak-status` event stream (spec §8.4).

import * as systemApi from "$lib/api/system";
import * as api from "$lib/api/tweaks";
import { STORAGE_KEYS } from "$lib/config/app";
import type {
  CachedSystemInfo,
  CategoryDefinition,
  CategoryMeta,
  SystemInfo,
  TweakDefinition,
  TweakStatus,
  TweakStatusView,
  TweakView,
  TweakWithStatus,
  WindowsInfo,
} from "$lib/types";
import { logError } from "$lib/utils/logger";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { tallies, toRiskLevel } from "$lib/utils/tweakPresentation";
import { appsStore } from "./apps.svelte";
import { elevationStore } from "./elevation.svelte";
import { favoritesStore } from "./favorites.svelte";
import { settingsStore } from "./settings.svelte";

export const DEFAULT_CATEGORY_ICON = "mdi:folder";

/** Stands in for the dynamic fields when only the cached hardware could be read. */
const UNKNOWN_WINDOWS: WindowsInfo = {
  version_string: "",
  display_version: "",
  build_number: "",
  product_name: "Windows",
  uptime_seconds: 0,
  is_windows_11: false,
  is_windows_server: false,
  install_date: null,
};

// Hardware rarely changes, so it is cached across launches.
const systemInfoCache = new PersistentStore<CachedSystemInfo | null>(STORAGE_KEYS.systemInfoCache, null);

let isLoadingSystemInfo = $state(true);
let isRefreshingSystemInfo = $state(false);
let isLoadingTweaks = $state(true);
let initialLoadComplete = false;

let systemInfo = $state<SystemInfo | null>(null);

// Raw: replaced whole on every status batch, so a deep proxy over hundreds of rows would only cost.
let tweaks = $state.raw<TweakWithStatus[]>([]);
let tweaksVersion = $state(0);
let categoryMeta = $state.raw<CategoryMeta[]>([]);

function mapView(view: TweakView): TweakDefinition {
  return {
    id: view.id,
    name: view.name,
    description: view.description,
    categoryId: view.category,
    riskLevel: toRiskLevel(view.risk),
    reversible: view.reversible,
    requiresReboot: view.requires_reboot,
    requiredLevel: view.required_level,
    availability: view.availability,
    supported: view.supported,
    options: view.options,
    info: view.info ?? undefined,
    warning: view.warning ?? undefined,
  };
}

const LOADING_STATUS: TweakStatus = {
  state: "loading",
  activeOption: null,
  unavailableReason: null,
  unknownReasons: [],
  needsElevation: false,
  unavailableOptions: [],
  residues: [],
  heldShared: [],
  observed: null,
  hasSnapshot: false,
  attention: null,
};

function mapStatusView(view: TweakStatusView): TweakStatus {
  const s = view.state;
  const unknownReasons = s.state === "unknown" ? s.reasons : [];
  return {
    state: s.state,
    activeOption: s.state === "active" ? s.option : null,
    unavailableReason: s.state === "unavailable" ? s.reason : null,
    unknownReasons,
    needsElevation: unknownReasons.some((r) => r.needs_elevation),
    unavailableOptions: view.unavailable,
    residues: view.residues,
    heldShared: view.held_shared,
    observed: view.observed,
    hasSnapshot: view.has_history,
    // Engine-owned, so it survives a rescan, a restart, and any snapshot entry being released (ADR-0001/0002).
    attention: view.attention,
  };
}

// The scan thread can outrun `loadModel`: without this an early status is dropped and the row spins forever.
let pendingStatusViews: Record<string, TweakStatusView> = {};

/** Highest stamp adopted per tweak, so an older reading can never replace a newer one. */
const statusStamps: Record<string, number> = {};

/** Adopts each view the stamp rule admits, replacing `tweaks` once for the whole set. */
function adoptStatusViews(views: Iterable<[string, TweakStatusView]>) {
  // A sweep event whose reads began before an apply recorded Needs Attention would otherwise land
  // afterwards and clear it. The backend stamps each status when its reads begin.
  const fresh: Record<string, TweakStatusView> = {};
  let adopted = false;
  for (const [tweakId, view] of views) {
    if (view.stamp < (statusStamps[tweakId] ?? 0)) continue;
    statusStamps[tweakId] = view.stamp;
    if (tweaksById.has(tweakId)) {
      fresh[tweakId] = view;
      adopted = true;
    } else pendingStatusViews[tweakId] = view;
  }
  if (!adopted) return;
  tweaks = tweaks.map((t) => {
    const view = fresh[t.definition.id];
    return view ? { ...t, status: mapStatusView(view) } : t;
  });
}

// Lookups by id read the full list: a pending change or a status event can name a hidden tweak.
const tweaksById = $derived(new Map(tweaks.map((t) => [t.definition.id, t])));

const visibleTweaks = $derived(settingsStore.showUnsupported ? tweaks : tweaks.filter((t) => t.definition.supported));
const withSnapshot = $derived(visibleTweaks.filter((t) => t.status.hasSnapshot));
const favoriteTweaks = $derived(visibleTweaks.filter((t) => favoritesStore.isFavorite(t.definition.id)));

// Corpus metadata once `get_categories` resolves; until then, ids discovered from the model.
const categories = $derived.by((): CategoryDefinition[] => {
  if (categoryMeta.length > 0) {
    return categoryMeta.map((c, i) => ({
      id: c.id,
      name: c.name,
      description: c.description,
      icon: c.icon || DEFAULT_CATEGORY_ICON,
      order: i,
    }));
  }
  const seen: Record<string, true> = {};
  const list: CategoryDefinition[] = [];
  for (const tweak of tweaks) {
    const id = tweak.definition.categoryId;
    if (seen[id]) continue;
    seen[id] = true;
    list.push({ id, name: id, description: "", icon: DEFAULT_CATEGORY_ICON, order: list.length });
  }
  return list;
});

const tweaksByCategory = $derived.by(() => {
  const byCategory: Record<string, TweakWithStatus[]> = {};
  for (const cat of categories) byCategory[cat.id] = [];
  for (const tweak of visibleTweaks) byCategory[tweak.definition.categoryId]?.push(tweak);
  return byCategory;
});

const categoryStats = $derived(
  Object.fromEntries(categories.map((cat) => [cat.id, tallies(tweaksByCategory[cat.id] ?? [])])),
);

const cacheTimestamp = $derived(systemInfoCache.value?.cachedAt ?? null);

function updateCache(info: SystemInfo): void {
  systemInfoCache.value = {
    hardware: info.hardware,
    device: info.device,
    computer_name: info.computer_name,
    cachedAt: new Date().toISOString(),
  };
}

function withCachedHardware(cache: CachedSystemInfo, windows: WindowsInfo, live?: SystemInfo): SystemInfo {
  return {
    windows,
    username: live?.username ?? "",
    is_admin: live?.is_admin ?? false,
    hardware: cache.hardware,
    device: cache.device,
    computer_name: cache.computer_name,
  };
}

export const systemStore = {
  get info() {
    return systemInfo;
  },

  /** ISO timestamp of when hardware info was last cached. */
  get cachedAt() {
    return cacheTimestamp;
  },

  /** Fresh dynamic info over the cached hardware; everything fresh when nothing is cached. */
  async load() {
    isLoadingSystemInfo = true;
    try {
      const cached = systemInfoCache.value;
      const fresh = await systemApi.getSystemInfo();
      if (cached) {
        systemInfo = withCachedHardware(cached, fresh.windows, fresh);
      } else {
        systemInfo = fresh;
        updateCache(fresh);
      }
      return systemInfo;
    } catch (error) {
      logError("Failed to load system info", error);
      const cached = systemInfoCache.value;
      if (!cached) return null;
      systemInfo = withCachedHardware(cached, UNKNOWN_WINDOWS);
      return systemInfo;
    } finally {
      isLoadingSystemInfo = false;
    }
  },

  /** Re-reads everything, hardware included. */
  async refresh() {
    isRefreshingSystemInfo = true;
    try {
      const fresh = await systemApi.getSystemInfo();
      systemInfo = fresh;
      updateCache(fresh);
      return fresh;
    } catch (error) {
      logError("Failed to refresh system info", error);
      throw error;
    } finally {
      isRefreshingSystemInfo = false;
    }
  },
};

export const categoriesStore = {
  get list() {
    return categories;
  },

  /** Tallies of the visible tweaks, per category id. */
  get stats() {
    return categoryStats;
  },

  /** The id itself when unknown. */
  getName(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.name ?? categoryId;
  },

  getIcon(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.icon ?? DEFAULT_CATEGORY_ICON;
  },
};

async function loadModel() {
  isLoadingTweaks = true;
  try {
    const [views, cats] = await Promise.all([api.getTweaks(), api.getCategories()]);
    categoryMeta = cats;
    tweaks = views.map((v) => {
      const early = pendingStatusViews[v.id];
      return { definition: mapView(v), status: early ? mapStatusView(early) : LOADING_STATUS };
    });
    pendingStatusViews = {};
    favoritesStore.prune((id) => tweaksById.has(id));
    tweaksVersion++;
    return tweaks;
  } catch (error) {
    logError("Failed to load tweaks", error);
    // The app cannot function without the tweak model.
    throw error;
  } finally {
    isLoadingTweaks = false;
  }
}

export const tweaksStore = {
  /** Visible tweaks: unsupported ones only when the setting shows them. */
  get list() {
    return visibleTweaks;
  },

  /** Every tweak, hidden ones included. */
  get all() {
    return tweaks;
  },

  get byCategory() {
    return tweaksByCategory;
  },

  get withSnapshot() {
    return withSnapshot;
  },

  get favorites() {
    return favoriteTweaks;
  },

  /** Bumped when the model reloads; status events do not bump it. */
  get version() {
    return tweaksVersion;
  },

  /** Searches the full list. */
  getById(tweakId: string): TweakWithStatus | undefined {
    return tweaksById.get(tweakId);
  },

  /** Loads the compiled model; statuses arrive later via the stream. */
  loadModel,

  /** Adopts a freshly detected status at once, unlike the batched stream. */
  setStatusView(tweakId: string, view: TweakStatusView) {
    adoptStatusViews([[tweakId, view]]);
  },
};

/** Bootstrap loading: the model, then system info. */
export const initStatus = {
  get isLoadingSystemInfo() {
    return isLoadingSystemInfo;
  },
  get isRefreshingSystemInfo() {
    return isRefreshingSystemInfo;
  },
  get isLoadingTweaks() {
    return isLoadingTweaks;
  },
};

// A full scan emits one event per tweak; replacing `tweaks` per event re-derives every list each time.
// eslint-disable-next-line svelte/prefer-svelte-reactivity
let queuedStatuses = new Map<string, TweakStatusView>();
let flushFrame = 0;

function flushStatuses() {
  flushFrame = 0;
  const batch = queuedStatuses;
  queuedStatuses = new Map();
  adoptStatusViews(batch);
}

function queueStatus(tweakId: string, view: TweakStatusView) {
  const queued = queuedStatuses.get(tweakId);
  if (!queued || view.stamp >= queued.stamp) queuedStatuses.set(tweakId, view);
  flushFrame ||= requestAnimationFrame(flushStatuses);
}

let statusStreamStarted = false;

/** Registers the `tweak-status` listener once, then kicks the background scan. */
async function startStatusStream(): Promise<void> {
  if (!statusStreamStarted) {
    statusStreamStarted = true;
    // Registered before the scan starts, so no early event is missed.
    await api.onTweakStatus((event) => queueStatus(event.tweak_id, event.status));
  }
  await api.getStatusesStream();
}

/** Re-runs the full scan after an elevation change so Unknowns become readable. */
export async function rescanStatuses(): Promise<void> {
  await elevationStore.load();
  void appsStore.load();
  await api.rescanAfterElevation();
}

let quickInitPromise: Promise<void> | null = null;
let remainingDataPromise: Promise<void> | null = null;

/** Loads the tweak model so rows and categories render; concurrent calls share one load. */
export async function initializeQuick(): Promise<void> {
  if (quickInitPromise) return quickInitPromise;
  if (tweaks.length > 0) return;

  quickInitPromise = loadModel()
    .then(() => {})
    .finally(() => {
      quickInitPromise = null;
    });
  return quickInitPromise;
}

/**
 * System info, elevation and the status stream. Awaits the model first: Svelte mounts `+page` before
 * `+layout`, so this runs before the layout's `initializeQuick()` and joins its promise-cached load.
 */
export async function loadRemainingData(): Promise<void> {
  if (remainingDataPromise) return remainingDataPromise;
  if (initialLoadComplete) return;

  remainingDataPromise = initializeQuick()
    .then(() => {
      // Not awaited: the app presence scan is slow and must not hold up the tweak UI.
      void appsStore.load();
      return Promise.all([systemStore.load(), elevationStore.load(), startStatusStream()]);
    })
    .then(() => {
      initialLoadComplete = true;
    })
    .finally(() => {
      remainingDataPromise = null;
    });
  return remainingDataPromise;
}
