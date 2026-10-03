import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { cn } from "./cn.ts";

const css = readFileSync(join(import.meta.dirname, "../../app.css"), "utf8");

// Per theme namespace: the utility prefix, a class from the same merge group (must replace the token's class),
// and optionally one from a group sharing the prefix (must sit beside it).
const probes: Record<string, { prefix: string; replacedBy: string; keptBeside?: string }> = {
  text: { prefix: "text", replacedBy: "text-xs", keptBeside: "text-red-500" },
  color: { prefix: "text", replacedBy: "text-red-500", keptBeside: "text-xs" },
  font: { prefix: "font", replacedBy: "font-serif", keptBeside: "font-bold" },
  tracking: { prefix: "tracking", replacedBy: "tracking-wide" },
  container: { prefix: "max-w", replacedBy: "max-w-xs" },
  width: { prefix: "w", replacedBy: "w-4" },
  "max-width": { prefix: "max-w", replacedBy: "max-w-xs" },
  spacing: { prefix: "p", replacedBy: "p-4" },
  "grid-template-columns": { prefix: "grid-cols", replacedBy: "grid-cols-2" },
  shadow: { prefix: "shadow", replacedBy: "shadow-md", keptBeside: "shadow-red-500" },
  animate: { prefix: "animate", replacedBy: "animate-spin" },
  ease: { prefix: "ease", replacedBy: "ease-linear" },
  "transition-duration": { prefix: "duration", replacedBy: "duration-75" },
  "transition-delay": { prefix: "delay", replacedBy: "delay-75" },
  "z-index": { prefix: "z", replacedBy: "z-10" },
};

// Longest first, so `--max-width-x` is not read as `--width-x`.
const namespaces = Object.keys(probes).sort((a, b) => b.length - a.length);
const declared = [...css.matchAll(new RegExp(`--(${namespaces.join("|")})-([a-z0-9]+(?:-[a-z0-9]+)*):`, "g"))];

test("app.css declares tokens in every probed namespace", () => {
  const seen = new Set(declared.map((m) => m[1]));
  assert.deepEqual(
    namespaces.filter((ns) => !seen.has(ns)),
    [],
  );
});

for (const [, ns, name] of declared) {
  const { prefix, replacedBy, keptBeside } = probes[ns];
  const cls = `${prefix}-${name}`;
  test(`tailwind-merge knows ${cls}`, () => {
    if (replacedBy !== cls) assert.equal(cn(cls, replacedBy), replacedBy);
    if (keptBeside) assert.equal(cn(cls, keptBeside), `${cls} ${keptBeside}`);
  });
}
