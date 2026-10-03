import assert from "node:assert/strict";
import { test } from "node:test";
import { FIELD_SEP, fieldRanges } from "./fieldRanges.ts";

const name = "Disable telemetry";
const description = "Stops usage data";
const info = "Sets AllowTelemetry";
const joined = [name, description, info].join(FIELD_SEP);
const bounds = { nameEnd: name.length, descEnd: name.length + FIELD_SEP.length + description.length };

const span = (word: string): [number, number] => {
  const start = joined.indexOf(word);
  assert.ok(start >= 0, word);
  return [start, start + word.length];
};

test("a range inside one field maps to that field's offsets", () => {
  const r = fieldRanges([...span("telemetry"), ...span("usage"), ...span("Allow")], bounds);
  assert.deepEqual(r.nameRanges, [8, 17]);
  assert.equal(description.slice(r.descriptionRanges[0], r.descriptionRanges[1]), "usage");
  assert.equal(info.slice(r.infoRanges[0], r.infoRanges[1]), "Allow");
});

test("a range spanning the separator is clipped into each field it touches", () => {
  const start = joined.indexOf("telemetry");
  const end = joined.indexOf("usage");
  const r = fieldRanges([start, end], bounds);
  assert.deepEqual(r.nameRanges, [start, bounds.nameEnd]);
  assert.equal(description.slice(r.descriptionRanges[0], r.descriptionRanges[1]), "Stops ");
  assert.deepEqual(r.infoRanges, []);
});

test("a range covering every field reaches each one", () => {
  const r = fieldRanges([0, joined.length], bounds);
  assert.deepEqual(r.nameRanges, [0, name.length]);
  assert.deepEqual(r.descriptionRanges, [0, description.length]);
  assert.deepEqual(r.infoRanges, [0, info.length]);
});

test("no ranges give empty fields", () => {
  assert.deepEqual(fieldRanges([], bounds), { nameRanges: [], descriptionRanges: [], infoRanges: [] });
});
