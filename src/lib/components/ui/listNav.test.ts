import assert from "node:assert/strict";
import { test } from "node:test";
import { nextEnabledIndex, radioKeyIndex } from "./listNav.ts";

const items = [{ disabled: true }, {}, { disabled: false }, { disabled: true }, {}];

test("nextEnabledIndex skips disabled items and wraps both ways", () => {
  assert.equal(nextEnabledIndex(items, -1, 1), 1);
  assert.equal(nextEnabledIndex(items, 1, 1), 2);
  assert.equal(nextEnabledIndex(items, 2, 1), 4);
  assert.equal(nextEnabledIndex(items, 4, 1), 1);
  assert.equal(nextEnabledIndex(items, 1, -1), 4);
  assert.equal(nextEnabledIndex(items, items.length, -1), 4);
});

test("nextEnabledIndex returns the start itself when it is the only enabled item", () => {
  assert.equal(nextEnabledIndex([{}, { disabled: true }], 0, 1), 0);
});

test("nextEnabledIndex is -1 when nothing is enabled", () => {
  assert.equal(nextEnabledIndex([{ disabled: true }, { disabled: true }], 0, 1), -1);
  assert.equal(nextEnabledIndex([], -1, 1), -1);
});

test("radioKeyIndex maps arrows, Home and End", () => {
  assert.equal(radioKeyIndex(items, 1, "ArrowRight"), 2);
  assert.equal(radioKeyIndex(items, 1, "ArrowDown"), 2);
  assert.equal(radioKeyIndex(items, 1, "ArrowLeft"), 4);
  assert.equal(radioKeyIndex(items, 1, "ArrowUp"), 4);
  assert.equal(radioKeyIndex(items, 2, "Home"), 1);
  assert.equal(radioKeyIndex(items, 2, "End"), 4);
});

test("radioKeyIndex is null for any other key", () => {
  for (const key of ["Enter", " ", "Tab", "a"]) assert.equal(radioKeyIndex(items, 1, key), null);
});
