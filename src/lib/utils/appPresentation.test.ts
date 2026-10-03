import assert from "node:assert/strict";
import { test } from "node:test";
import type { AppPresence, AppStatusView, AppView, InstallRoute } from "$lib/types";
import { appAction } from "./appPresentation.ts";

const BLOCKED = { state: "needs_elevation", reason: "Needs admin" } as const;

const app = (patch: Partial<AppView> = {}): AppView => ({
  id: "a",
  name: "A",
  description: "",
  info: null,
  warning: null,
  category: "c",
  risk: "Low",
  source: "appx",
  install: null,
  remove_availability: { state: "available" },
  install_availability: { state: "available" },
  supported: true,
  ...patch,
});

const status = (presence: AppPresence, install_route: InstallRoute = "none"): AppStatusView => ({
  app_id: "a",
  presence,
  install_route,
  stamp: 1,
});

const installed: AppPresence = { state: "installed", provisioned_only: false };
const absent: AppPresence = { state: "absent" };

test("appAction blocks Remove while presence is still loading or failed to scan", () => {
  assert.deepEqual(appAction(app(), undefined, null), {
    kind: "remove",
    disabledReason: "Checking whether this app is installed…",
  });
  assert.deepEqual(appAction(app(), undefined, "Scan failed"), { kind: "remove", disabledReason: "Scan failed" });
});

test("appAction blocks Remove on an unknown presence with its reason", () => {
  const unknown: AppPresence = { state: "unknown", reason: "Unreadable", needs_elevation: false };
  assert.deepEqual(appAction(app(), status(unknown), null), { kind: "remove", disabledReason: "Unreadable" });
});

test("appAction offers Remove for an installed app, blocked by its availability", () => {
  assert.deepEqual(appAction(app(), status(installed), null), { kind: "remove", disabledReason: null });
  assert.deepEqual(appAction(app({ remove_availability: BLOCKED }), status(installed), null), {
    kind: "remove",
    disabledReason: "Needs admin",
  });
});

test("appAction picks the absent app's action from its install route", () => {
  assert.deepEqual(appAction(app({ install_availability: BLOCKED }), status(absent, "winget"), null), {
    kind: "install",
    disabledReason: "Needs admin",
  });
  assert.deepEqual(appAction(app(), status(absent, "store_page"), null), { kind: "store", disabledReason: null });
  assert.equal(appAction(app(), status(absent, "none"), null), null);
});
