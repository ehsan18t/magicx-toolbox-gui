import assert from "node:assert/strict";
import { test } from "node:test";
import { placeFlyout, type FlyoutPlacement } from "./flyout.ts";

const viewport = { width: 800, height: 600 };
const size = { width: 100, height: 40 };
const anchorAt = (top: number, left: number) => ({ top, bottom: top + 20, left, width: 60 });
const TOOLTIP: FlyoutPlacement = { side: "above", align: "center", gap: 8, inset: 4 };
const MENU: FlyoutPlacement = { side: "below", align: "start", gap: 4, inset: 8 };

test("a tooltip sits centred above its anchor", () => {
  assert.deepEqual(placeFlyout(anchorAt(300, 400), size, viewport, TOOLTIP), { top: 252, left: 380, above: true });
});

test("a tooltip flips below when there is no room above", () => {
  assert.deepEqual(placeFlyout(anchorAt(20, 400), size, viewport, TOOLTIP), { top: 48, left: 380, above: false });
});

test("a menu sits start-aligned below its anchor", () => {
  assert.deepEqual(placeFlyout(anchorAt(100, 200), size, viewport, MENU), { top: 124, left: 200, above: false });
});

test("a menu flips above when it would overflow the bottom", () => {
  assert.deepEqual(placeFlyout(anchorAt(560, 200), size, viewport, MENU), { top: 516, left: 200, above: true });
});

test("a menu stays below when neither side fits", () => {
  const tall = { width: 100, height: 590 };
  assert.equal(placeFlyout(anchorAt(300, 200), tall, viewport, MENU).above, false);
});

test("a tooltip stays above when neither side fits", () => {
  const tall = { width: 100, height: 590 };
  assert.equal(placeFlyout(anchorAt(300, 200), tall, viewport, TOOLTIP).above, true);
});

test("the flyout is clamped inside the viewport's sides", () => {
  assert.equal(placeFlyout(anchorAt(300, 0), size, viewport, TOOLTIP).left, 4);
  assert.equal(placeFlyout(anchorAt(300, 780), size, viewport, TOOLTIP).left, 696);
  assert.equal(placeFlyout(anchorAt(100, 750), size, viewport, MENU).left, 692);
  assert.equal(placeFlyout(anchorAt(100, -20), size, viewport, MENU).left, 8);
});
