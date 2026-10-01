/**
 * App items: the curated removable apps and their live presence (ADR-0009).
 * Remove and Install run immediately, never through pending changes, snapshots or profiles.
 */

import * as api from "$lib/api/apps";
import type { AppStatusView, AppView } from "$lib/types";
import { errorMessage, isAppExiting } from "$lib/utils/error";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { toastStore } from "./toast.svelte";

let apps = $state<AppView[]>([]);
let appsVersion = $state(0);
let scanError = $state<string | null>(null);
const statuses = new SvelteMap<string, AppStatusView>();
const busy = new SvelteSet<string>();
const errors = new SvelteMap<string, string>();
/** Opened in the Store: id -> focus re-checks left. Each check spawns PowerShell, so it is bounded. */
const awaitingStore = new SvelteMap<string, number>();
const STORE_WATCH_CHECKS = 5;
let loadPromise: Promise<void> | null = null;
let refreshPromise: Promise<void> | null = null;
let focusWatched = false;

// Mirrors tweaksData's stamp rule: a scan whose reads began before a removal must not undo it.
function adopt(view: AppStatusView): void {
  const current = statuses.get(view.app_id);
  if (current && view.stamp < current.stamp) return;
  statuses.set(view.app_id, view);
}

/** Absent with no way back on this machine is hidden; Unknown never is (hiding it would fail open). */
function isVisible(app: AppView): boolean {
  const status = statuses.get(app.id);
  return !status || status.presence.state !== "absent" || status.install_route !== "none";
}

const visibleApps = $derived(apps.filter(isVisible));

const appsByCategory = $derived.by(() => {
  const byCategory: Record<string, AppView[]> = {};
  for (const app of visibleApps) (byCategory[app.category] ??= []).push(app);
  return byCategory;
});

/** Concurrent callers share one scan: each one invalidates the backend index. */
function refreshStatuses(): Promise<void> {
  refreshPromise ??= (async () => {
    try {
      for (const view of await api.getAppStatuses()) adopt(view);
      scanError = null;
    } catch (error) {
      console.error("Failed to scan app statuses:", error);
      scanError = errorMessage(error);
    }
  })().finally(() => {
    refreshPromise = null;
  });
  return refreshPromise;
}

async function onWindowFocus(): Promise<void> {
  if (awaitingStore.size === 0) return;
  await refreshStatuses();
  for (const [id, left] of awaitingStore) {
    if (statuses.get(id)?.presence.state === "installed" || left <= 1) awaitingStore.delete(id);
    else awaitingStore.set(id, left - 1);
  }
}

async function run(id: string, op: (id: string) => Promise<AppStatusView>, done: string): Promise<void> {
  if (busy.has(id)) return;
  busy.add(id);
  errors.delete(id);
  let failed = false;
  const tweakName = apps.find((a) => a.id === id)?.name;
  try {
    adopt(await op(id));
    toastStore.success(done, { tweakName });
  } catch (error) {
    // Refused before anything ran, so there is nothing to re-scan.
    if (isAppExiting(error)) {
      toastStore.warning(errorMessage(error), { tweakName });
      return;
    }
    failed = true;
    errors.set(id, errorMessage(error));
  } finally {
    busy.delete(id);
  }
  // The operation may have partly happened, so show what the machine reads now.
  if (failed) await refreshStatuses();
}

export const appsStore = {
  get list() {
    return apps;
  },

  get visible() {
    return visibleApps;
  },

  get byCategory() {
    return appsByCategory;
  },

  get version() {
    return appsVersion;
  },

  /** Why the last presence scan failed, for cards that have no status yet. */
  get scanError() {
    return scanError;
  },

  status(id: string): AppStatusView | undefined {
    return statuses.get(id);
  },

  isVisible(id: string): boolean {
    const app = apps.find((a) => a.id === id);
    return !!app && isVisible(app);
  },

  isBusy(id: string): boolean {
    return busy.has(id);
  },

  error(id: string): string | undefined {
    return errors.get(id);
  },

  clearError(id: string): void {
    errors.delete(id);
  },

  /** Load the model (availability included) and scan presence. Concurrent calls share one run. */
  load(): Promise<void> {
    loadPromise ??= (async () => {
      try {
        apps = await api.getApps();
        appsVersion++;
      } catch (error) {
        console.error("Failed to load apps:", error);
        return;
      }
      await refreshStatuses();
    })().finally(() => {
      loadPromise = null;
    });
    return loadPromise;
  },

  refresh: refreshStatuses,

  remove(id: string): Promise<void> {
    return run(id, api.removeApp, "Removed");
  },

  install(id: string): Promise<void> {
    return run(id, api.installApp, "Installed");
  },

  /** The Store install is unverified, so presence is re-read when the user comes back. */
  async openStorePage(id: string): Promise<void> {
    const productId = apps.find((a) => a.id === id)?.install?.id;
    if (!productId) return;
    errors.delete(id);
    try {
      await openUrl(`ms-windows-store://pdp/?ProductId=${productId}`);
    } catch (error) {
      errors.set(id, errorMessage(error));
      return;
    }
    awaitingStore.set(id, STORE_WATCH_CHECKS);
    if (!focusWatched) {
      focusWatched = true;
      window.addEventListener("focus", () => void onWindowFocus());
    }
  },
};
