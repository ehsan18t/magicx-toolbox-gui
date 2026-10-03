import assert from "node:assert/strict";
import { test } from "node:test";
import type { Availability, TweakDefinition, TweakStatus, TweakWithStatus } from "$lib/types";
import { controlShowsState, isComplete, restoreState, tallies } from "./tweakPresentation.ts";

const definition = (availability: Availability = { state: "available" }): TweakDefinition => ({
  id: "t",
  name: "T",
  description: "",
  categoryId: "c",
  riskLevel: "low",
  reversible: true,
  requiresReboot: false,
  requiredLevel: "User",
  availability,
  supported: true,
  options: [],
  info: null,
  warning: null,
});

const status = (patch: Partial<TweakStatus> = {}): TweakStatus => ({
  state: "system_default",
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
  ...patch,
});

const tweak = (patch: Partial<TweakStatus> = {}, availability?: Availability): TweakWithStatus => ({
  definition: definition(availability),
  status: status(patch),
});

test("restoreState runs when available and idle", () => {
  assert.deepEqual(restoreState(definition(), status(), false), {
    label: "Restore",
    disabled: false,
    tip: "Restore the state saved before the last change",
  });
});

test("restoreState is disabled while running", () => {
  assert.equal(restoreState(definition(), status(), true).disabled, true);
});

test("restoreState offers a retry after a failed restore", () => {
  const s = status({ attention: { reason: "restore_failed", items: [] } });
  const r = restoreState(definition(), s, false);
  assert.equal(r.label, "Retry restore");
  assert.equal(r.tip, "Retry restoring the saved state");
});

test("restoreState explains an unavailable tweak with its reason", () => {
  const r = restoreState(definition({ state: "needs_elevation", reason: "Needs admin" }), status(), false);
  assert.deepEqual(r, { label: "Restore", disabled: true, tip: "Needs admin" });
});

test("tallies counts every field in one pass", () => {
  const t = tallies([
    tweak({ state: "active", hasHistory: true }),
    tweak({ state: "active", attention: { reason: "apply_failed", items: [] } }),
    tweak({ state: "unknown" }, { state: "needs_elevation", reason: "x" }),
    tweak({ state: "loading" }),
  ]);
  assert.deepEqual(t, {
    total: 4,
    applied: 2,
    attention: 1,
    withSnapshot: 1,
    needsAdmin: 1,
    byState: { loading: 1, active: 2, system_default: 0, unavailable: 0, unknown: 1 },
  });
});

test("isComplete needs every tweak applied and at least one tweak", () => {
  assert.equal(isComplete({ applied: 0, total: 0 }), false);
  assert.equal(isComplete({ applied: 1, total: 2 }), false);
  assert.equal(isComplete({ applied: 2, total: 2 }), true);
});

test("controlShowsState only when the control highlights the current state", () => {
  const withOptions = (...labels: string[]): TweakDefinition => ({
    ...definition(),
    options: labels.map((label) => ({ label }) as TweakDefinition["options"][number]),
  });
  const two = withOptions("Off", "On");
  assert.equal(controlShowsState(two, status({ state: "active", activeOption: "On" })), true);
  assert.equal(controlShowsState(two, status({ state: "active", activeOption: "Gone" })), false);
  assert.equal(controlShowsState(two, status({ state: "system_default" })), false);
  assert.equal(controlShowsState(withOptions("Off"), status({ state: "system_default" })), true);
  assert.equal(controlShowsState(two, status({ state: "unknown" })), false);
  assert.equal(controlShowsState(two, status({ state: "loading" })), false);
});
