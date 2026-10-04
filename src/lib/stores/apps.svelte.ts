// App items: the curated removable apps and their live presence (ADR-0009). Remove and Install run
// immediately, never through pending changes, snapshots or profiles.

import * as appsApi from "$lib/api/apps";
import type { AppOperationKind, AppStatusView, AppView } from "$lib/types";
import { openExternalUrl } from "$lib/api/platform";
import { isPermanent, removeConfirmMessage } from "$lib/utils/appPresentation";
import { errorMessage, isAppCancelled, isAppExiting } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { isStaleReading } from "$lib/utils/stamp";
import { SvelteMap } from "svelte/reactivity";
import { confirmStore } from "./confirm.svelte";
import { settingsStore } from "./settings.svelte";
import { toastStore } from "./toast.svelte";

export interface AppOperation {
  kind: AppOperationKind;
  startedAt: number;
  cancelling: boolean;
}

const STORE_PAGE_URL = "ms-windows-store://pdp/?ProductId=";
const STORE_WATCH_CHECKS = 5;

let apps = $state.raw<AppView[]>([]);
let modelVersion = $state(0);
let scanError = $state<string | null>(null);
let loadError = $state<string | null>(null);
const statuses = new SvelteMap<string, AppStatusView>();
// Store-side, so a row that remounts mid-operation keeps its label and elapsed time.
const operations = new SvelteMap<string, AppOperation>();
const errors = new SvelteMap<string, string>();
/** Opened in the Store: id -> focus re-checks left. Each check spawns PowerShell, so it is bounded. */
// eslint-disable-next-line svelte/prefer-svelte-reactivity -- re-check budget, never rendered
const awaitingStore = new Map<string, number>();
let loadPromise: Promise<void> | null = null;
let scanning: Promise<void> | null = null;
let rescan: Promise<void> | null = null;
let focusWatched = false;

// A scan whose reads began before a removal must not undo it.
function adopt(view: AppStatusView): void {
  if (isStaleReading(view.stamp, statuses.get(view.app_id)?.stamp)) return;
  statuses.set(view.app_id, view);
}

/** Absent with no way back on this machine is hidden; Unknown never is (hiding it would fail open). */
function isVisible(app: AppView): boolean {
  if (!app.supported && !settingsStore.showUnsupported) return false;
  const status = statuses.get(app.id);
  return !status || status.presence.state !== "absent" || status.install_route !== "none";
}

const appsById = $derived(new Map(apps.map((a) => [a.id, a])));

const appsByCategory = $derived.by(() => {
  const byCategory: Record<string, AppView[]> = {};
  for (const app of apps) if (isVisible(app)) (byCategory[app.category] ??= []).push(app);
  return byCategory;
});

async function scan(): Promise<void> {
  try {
    for (const view of await appsApi.getAppStatuses()) adopt(view);
    scanError = null;
  } catch (error) {
    logError("Failed to scan app statuses", error);
    scanError = errorMessage(error);
  }
}

/**
 * Resolves after a scan that started after the call. A call during a scan queues one more (shared by every
 * call meanwhile) rather than joining it: the running scan may have read before the change being checked.
 */
function refreshStatuses(): Promise<void> {
  if (!scanning) {
    scanning = scan().finally(() => {
      scanning = null;
    });
    return scanning;
  }
  rescan ??= scanning.then(() => {
    rescan = null;
    return refreshStatuses();
  });
  return rescan;
}

async function onWindowFocus(): Promise<void> {
  if (awaitingStore.size === 0) return;
  await refreshStatuses();
  for (const [id, left] of awaitingStore) {
    if (statuses.get(id)?.presence.state === "installed" || left <= 1) awaitingStore.delete(id);
    else awaitingStore.set(id, left - 1);
  }
}

async function run(id: string, kind: AppOperationKind, done: string): Promise<void> {
  if (operations.has(id)) return;
  operations.set(id, { kind, startedAt: Date.now(), cancelling: false });
  errors.delete(id);
  let failed = false;
  const subject = appsById.get(id)?.name;
  try {
    adopt(await (kind === "remove" ? appsApi.removeApp(id) : appsApi.installApp(id)));
    toastStore.success(done, { subject });
  } catch (error) {
    // Refused before anything ran, so there is nothing to re-scan.
    if (isAppExiting(error)) {
      toastStore.warning(errorMessage(error), { subject });
      return;
    }
    failed = true;
    if (isAppCancelled(error)) toastStore.info("Install cancelled", { subject });
    else errors.set(id, errorMessage(error));
  } finally {
    operations.delete(id);
  }
  // The operation may have partly happened, so show what the machine reads now.
  if (failed) await refreshStatuses();
}

export const appsStore = {
  get list() {
    return apps;
  },

  get byCategory() {
    return appsByCategory;
  },

  /** Bumped when the model reloads. */
  get modelVersion() {
    return modelVersion;
  },

  /** Why the model failed to load: no app row can render. */
  get loadError() {
    return loadError;
  },

  /** Why the last presence scan failed, for rows that have no status yet. */
  get scanError() {
    return scanError;
  },

  status(id: string): AppStatusView | undefined {
    return statuses.get(id);
  },

  app(id: string): AppView | undefined {
    return appsById.get(id);
  },

  isVisible(id: string): boolean {
    const app = appsById.get(id);
    return !!app && isVisible(app);
  },

  /** The remove or install in flight, with when it started. */
  operation(id: string): AppOperation | undefined {
    return operations.get(id);
  },

  error(id: string): string | undefined {
    return errors.get(id);
  },

  clearError(id: string): void {
    errors.delete(id);
  },

  /** Loads the model (availability included) and scans presence. Concurrent calls share one run. */
  load(): Promise<void> {
    loadPromise ??= (async () => {
      try {
        apps = await appsApi.getApps();
        modelVersion++;
        loadError = null;
      } catch (error) {
        logError("Failed to load apps", error);
        loadError = errorMessage(error);
        return;
      }
      await refreshStatuses();
    })().finally(() => {
      loadPromise = null;
    });
    return loadPromise;
  },

  /** Asks first; the message says whether the app can be reinstalled from here. */
  async removeWithConfirm(id: string): Promise<void> {
    const app = appsById.get(id);
    if (!app) return;
    const confirmed = await confirmStore.ask({
      title: `Remove ${app.name}?`,
      message: removeConfirmMessage(app, isPermanent(statuses.get(id))),
      confirmText: "Remove",
      variant: "danger",
    });
    if (confirmed) await run(id, "remove", "Removed");
  },

  install(id: string): Promise<void> {
    return run(id, "install", "Installed");
  },

  /** The install keeps its operation until the backend reports what the cancel left behind. */
  async cancelInstall(id: string): Promise<void> {
    const operation = operations.get(id);
    if (operation?.kind !== "install" || operation.cancelling) return;
    operations.set(id, { ...operation, cancelling: true });
    let found = false;
    try {
      found = await appsApi.cancelAppInstall(id);
    } catch (error) {
      toastStore.failure("Could not cancel the install", error, { subject: appsById.get(id)?.name });
    }
    // Not found: the install already ended, or has not reached the backend yet and can be cancelled again.
    const current = operations.get(id);
    if (!found && current) operations.set(id, { ...current, cancelling: false });
  },

  /** The Store install is unverified, so presence is re-read when the user comes back. */
  async openStorePage(id: string): Promise<void> {
    const productId = appsById.get(id)?.install?.id;
    if (!productId) return;
    errors.delete(id);
    try {
      await openExternalUrl(`${STORE_PAGE_URL}${productId}`);
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
