import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

// Parsed as text: the registry imports `~icons/*` modules, which only the Vite build resolves.
const registry = readFileSync(join(import.meta.dirname, "icons.ts"), "utf8");
const registered = new Set([...registry.matchAll(/^\s*"([\w-]+:[\w-]+)":/gm)].map((m) => m[1]));

const tweaksDir = join(import.meta.dirname, "../../../src-tauri/tweaks");

for (const file of readdirSync(tweaksDir).filter((f) => f.endsWith(".yaml"))) {
  const header = readFileSync(join(tweaksDir, file), "utf8").split(/^tweaks:/m)[0];
  const icon = header.match(/^\s+icon:\s*"?([^"\s]+)"?\s*$/m)?.[1];
  test(`${file} category icon is registered`, () => {
    assert.ok(icon, "category header has no icon");
    assert.ok(registered.has(icon), `"${icon}" is missing from src/lib/design/icons.ts`);
  });
}
