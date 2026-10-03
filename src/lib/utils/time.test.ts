import assert from "node:assert/strict";
import { test } from "node:test";
import { elapsedClock, formatDate, formatDuration, HOUR_MS, SECOND_MS } from "./time.ts";

test("formatDate is empty for missing or zero input", () => {
  for (const value of [null, undefined, "", 0]) assert.equal(formatDate(value), "");
});

test("formatDate returns an unparseable string as given", () => {
  assert.equal(formatDate("not a date"), "not a date");
  assert.equal(formatDate(new Date(Number.NaN)), "");
});

test("formatDate adds the time only when asked, seconds only at seconds precision", () => {
  const date = new Date(2024, 0, 2, 3, 4, 5);
  const dateOnly = formatDate(date);
  const minutes = formatDate(date.toISOString(), { time: "minutes" });
  const seconds = formatDate(date.getTime(), { time: "seconds" });
  assert.match(dateOnly, /2024/);
  assert.doesNotMatch(dateOnly, /04/);
  assert.match(minutes, /04/);
  assert.doesNotMatch(minutes, /05/);
  assert.match(seconds, /04.*05/);
  assert.notEqual(formatDate(date, { month: "short" }), dateOnly);
});

test("elapsedClock pads seconds and clamps a clock that runs backwards", () => {
  assert.equal(elapsedClock(0, 0), "0:00");
  assert.equal(elapsedClock(0, 9 * SECOND_MS + 999), "0:09");
  assert.equal(elapsedClock(0, 61 * SECOND_MS), "1:01");
  assert.equal(elapsedClock(0, HOUR_MS), "60:00");
  assert.equal(elapsedClock(5000, 0), "0:00");
});

test("formatDuration keeps the two largest units", () => {
  assert.equal(formatDuration(0), "");
  assert.equal(formatDuration(-5), "");
  assert.equal(formatDuration(Number.NaN), "");
  assert.equal(formatDuration(59), "0m");
  assert.equal(formatDuration(12 * 60 + 30), "12m");
  assert.equal(formatDuration(3600 + 120), "1h 2m");
  assert.equal(formatDuration(3 * 86400 + 4 * 3600 + 59 * 60), "3d 4h");
  assert.equal(formatDuration(86400), "1d 0h");
});
