import assert from "node:assert/strict";
import { test } from "node:test";
import type { CachedSystemInfo, LiveSystemInfo } from "$lib/types";
import { firstPaint, parseSystemCache, UNKNOWN_LIVE } from "./systemCache.ts";

const cached: CachedSystemInfo = {
  hardware: {
    cpu: { name: "CPU", cores: 8, threads: 16, architecture: "x64", max_clock_mhz: 3200 },
    gpu: [],
    monitors: [],
    memory: { total_gb: 16, speed_mhz: 3200, memory_type: "DDR4", slots_used: 2 },
    motherboard: { manufacturer: "Board Co", product: "B1", bios_version: "1.0" },
    disks: [],
    network: [],
    total_storage_gb: 0,
  },
  device: { manufacturer: "Maker", model: "Model", system_type: "x64-based PC", pc_type: "Laptop" },
  cachedAt: "2026-01-01T00:00:00.000Z",
};

const live: LiveSystemInfo = { ...UNKNOWN_LIVE, computer_name: "PC", username: "me", is_admin: true };

test("parseSystemCache adopts a valid cache and an empty one", () => {
  assert.deepEqual(parseSystemCache(JSON.parse(JSON.stringify(cached))), cached);
  assert.equal(parseSystemCache(null), null);
});

test("parseSystemCache rejects a cache an older build wrote", () => {
  const { gpu: _gpu, ...withoutGpu } = cached.hardware;
  assert.equal(parseSystemCache({ ...cached, hardware: withoutGpu }), undefined);
  const { cachedAt: _cachedAt, ...undated } = cached;
  assert.equal(parseSystemCache(undated), undefined);
});

test("parseSystemCache rejects garbage", () => {
  for (const value of [42, "cache", [], {}, { hardware: "x", device: {} }, { hardware: {}, device: {} }])
    assert.equal(parseSystemCache(value), undefined);
});

test("firstPaint is a skeleton with nothing cached", () => {
  assert.equal(firstPaint(live, null), null);
  assert.equal(firstPaint(null, null), null);
});

test("firstPaint shows the cached hardware under the live fields, or placeholders when unread", () => {
  const shown = firstPaint(live, cached);
  assert.equal(shown?.computer_name, "PC");
  assert.equal(shown?.hardware, cached.hardware);
  assert.equal(shown && "cachedAt" in shown, false);
  assert.equal(firstPaint(null, cached)?.windows.product_name, "Windows");
});
