// The tweak model (`get_tweaks`), its categories, and live per-tweak statuses filled in incrementally
// from the `tweak-status` event stream (spec §8.4).

import * as tweaksApi from "$lib/api/tweaks";
import { type IconName, isIconName } from "$lib/design";
import type {
  CategoryDefinition,
  CategoryMeta,
  TweakDefinition,
  TweakStatus,
  TweakStatusView,
  TweakView,
  TweakWithStatus,
} from "$lib/types";
import { logError } from "$lib/utils/logger";
import { tallies, toRiskLevel } from "$lib/utils/tweakPresentation";
import { favoritesStore } from "./favorites.svelte";
import { settingsStore } from "./settings.svelte";

const DEFAULT_CATEGORY_ICON: IconName = "mdi:folder";

// Raw: replaced whole on every status batch, so a deep proxy over hundreds of rows would only cost.
let tweaks = $state.raw<TweakWithStatus[]>([]);
let modelVersion = $state(0);
let isModelLoaded = false;
let isLoading = $state(true);
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
  hasHistory: false,
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
    hasHistory: view.has_history,
    // Engine-owned, so it survives a rescan, a restart, and any snapshot entry being released (ADR-0001/0002).
    attention: view.attention,
  };
}

// Lookups by id read the full list: a pending change or a status event can name a hidden tweak.
const tweaksById = $derived(new Map(tweaks.map((t) => [t.definition.id, t])));

// The scan thread can outrun the model load: without this an early status is dropped and the row spins forever.
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

const visibleTweaks = $derived(settingsStore.showUnsupported ? tweaks : tweaks.filter((t) => t.definition.supported));
const withSnapshot = $derived(visibleTweaks.filter((t) => t.status.hasHistory));
const favoriteTweaks = $derived(visibleTweaks.filter((t) => favoritesStore.isFavorite(t.definition.id)));

// Corpus metadata once `get_categories` resolves; until then, ids discovered from the model.
const categories = $derived.by((): CategoryDefinition[] => {
  if (categoryMeta.length > 0) {
    return categoryMeta.map((c, i) => ({
      id: c.id,
      name: c.name,
      description: c.description,
      icon: isIconName(c.icon) ? c.icon : DEFAULT_CATEGORY_ICON,
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

export const categoriesStore = {
  get list() {
    return categories;
  },

  /** Tallies of the visible tweaks, per category id. */
  get stats() {
    return categoryStats;
  },

  /** The id itself when unknown. */
  name(categoryId: string): string {
    return categories.find((c) => c.id === categoryId)?.name ?? categoryId;
  },

  icon(categoryId: string): IconName {
    return categories.find((c) => c.id === categoryId)?.icon ?? DEFAULT_CATEGORY_ICON;
  },
};

let loadPromise: Promise<void> | null = null;

async function loadModel(): Promise<void> {
  isLoading = true;
  try {
    const [views, cats] = await Promise.all([tweaksApi.getTweaks(), tweaksApi.getCategories()]);
    categoryMeta = cats;
    tweaks = views.map((v) => {
      const early = pendingStatusViews[v.id];
      return { definition: mapView(v), status: early ? mapStatusView(early) : LOADING_STATUS };
    });
    pendingStatusViews = {};
    favoritesStore.prune((id) => tweaksById.has(id));
    modelVersion++;
    isModelLoaded = true;
  } catch (error) {
    logError("Failed to load tweaks", error);
    // The app cannot function without the tweak model.
    throw error;
  } finally {
    isLoading = false;
  }
}

// A full scan emits one event per tweak; replacing `tweaks` per event re-derives every list each time.
// eslint-disable-next-line svelte/prefer-svelte-reactivity -- a queue, never rendered
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

let isStreamStarted = false;

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

  get isLoading() {
    return isLoading;
  },

  /** Bumped when the model reloads; status events do not bump it. */
  get modelVersion() {
    return modelVersion;
  },

  /** Searches the full list. */
  tweak(tweakId: string): TweakWithStatus | undefined {
    return tweaksById.get(tweakId);
  },

  /** Loads the compiled model once so rows and categories render; concurrent calls share one load. */
  load(): Promise<void> {
    if (isModelLoaded) return Promise.resolve();
    loadPromise ??= loadModel().finally(() => {
      loadPromise = null;
    });
    return loadPromise;
  },

  /** Adopts a freshly detected status at once, unlike the batched stream. */
  setStatusView(tweakId: string, view: TweakStatusView) {
    adoptStatusViews([[tweakId, view]]);
  },

  /** Registers the `tweak-status` listener once, then kicks the background scan. */
  async streamStatuses(): Promise<void> {
    if (!isStreamStarted) {
      isStreamStarted = true;
      // Registered before the scan starts, so no early event is missed.
      await tweaksApi.onTweakStatus((event) => queueStatus(event.tweak_id, event.status));
    }
    await tweaksApi.getStatusesStream();
  },
};
