import assert from "node:assert/strict";
import { test } from "node:test";
import { EXCERPT_LEAD, EXCERPT_LENGTH, infoExcerpt } from "./excerpt.ts";

/** Ranges over `source` for the first occurrence of each needle. */
function rangesOf(source: string, ...needles: string[]): number[] {
  return needles.flatMap((needle) => {
    const at = source.indexOf(needle);
    assert.notEqual(at, -1, `"${needle}" not in source`);
    return [at, at + needle.length];
  });
}

function marked(excerpt: ReturnType<typeof infoExcerpt>): string[] {
  assert.ok(excerpt);
  const out: string[] = [];
  for (let i = 0; i < excerpt.ranges.length; i += 2)
    out.push(excerpt.text.slice(excerpt.ranges[i], excerpt.ranges[i + 1]));
  return out;
}

test("strips bold and code markers", () => {
  const info = "Sets **Enabled** to `0` under the key.";
  const excerpt = infoExcerpt(info, rangesOf(info, "key"));
  assert.equal(excerpt?.text, "Sets Enabled to 0 under the key.");
  assert.deepEqual(marked(excerpt), ["key"]);
});

test("highlights a match inside bold", () => {
  const info = "There are alternatives: **Harden RDP (NLA + TLS)** and a VPN.";
  const excerpt = infoExcerpt(info, rangesOf(info, "NLA"));
  assert.equal(excerpt?.text, "There are alternatives: Harden RDP (NLA + TLS) and a VPN.");
  assert.deepEqual(marked(excerpt), ["NLA"]);
});

test("a range spanning the markers covers only the text", () => {
  const info = "Use __strict__ mode.";
  const excerpt = infoExcerpt(info, rangesOf(info, "__strict__"));
  assert.deepEqual(marked(excerpt), ["strict"]);
});

test("keeps a link's label and drops its url", () => {
  const info = "See [the Microsoft docs](https://learn.microsoft.com/rdp) for details.";
  const excerpt = infoExcerpt(info, rangesOf(info, "Microsoft"));
  assert.equal(excerpt?.text, "See the Microsoft docs for details.");
  assert.deepEqual(marked(excerpt), ["Microsoft"]);
});

test("a match only in a link's url has nothing to show", () => {
  const info = "See [the docs](https://learn.microsoft.com/rdp).";
  assert.equal(infoExcerpt(info, rangesOf(info, "learn")), null);
});

test("strips heading and list markers and separates blocks", () => {
  const info = "## Why\n\nIt helps:\n- faster   boot\n1. less noise\n  * smaller logs";
  const excerpt = infoExcerpt(info, rangesOf(info, "boot", "logs"));
  assert.equal(excerpt?.text, "Why · It helps: · faster boot · less noise · smaller logs");
  assert.deepEqual(marked(excerpt), ["boot", "logs"]);
});

test("a window never opens or closes on a block separator", () => {
  const info = `${"lead ".repeat(10)}\n## Notes\n\nSee kerberos here.\n${"tail ".repeat(40)}`;
  const excerpt = infoExcerpt(info, rangesOf(info, "kerberos"));
  assert.ok(excerpt);
  assert.ok(!/^…?\s*·/.test(excerpt.text), excerpt.text);
  assert.ok(!/·\s*…?$/.test(excerpt.text), excerpt.text);
  assert.deepEqual(marked(excerpt), ["kerberos"]);
});

test("a match at the start has no leading ellipsis", () => {
  const info = `Telemetry ${"word ".repeat(60)}end.`;
  const excerpt = infoExcerpt(info, rangesOf(info, "Telemetry"));
  assert.ok(excerpt);
  assert.ok(excerpt.text.startsWith("Telemetry "));
  assert.ok(excerpt.text.endsWith("word…"), "cut at a word boundary");
  assert.ok(excerpt.text.length <= EXCERPT_LENGTH + 1);
  assert.deepEqual(marked(excerpt), ["Telemetry"]);
});

test("a match at the end has no trailing ellipsis", () => {
  const info = `${"word ".repeat(60)}Telemetry`;
  const excerpt = infoExcerpt(info, rangesOf(info, "Telemetry"));
  assert.ok(excerpt);
  assert.ok(excerpt.text.startsWith("…word "), "cut at a word boundary");
  assert.ok(excerpt.text.endsWith("Telemetry"));
  assert.ok(excerpt.text.length <= EXCERPT_LEAD + "…Telemetry".length);
  assert.deepEqual(marked(excerpt), ["Telemetry"]);
});

test("keeps every range inside the window and drops those outside", () => {
  const info = `${"lorem ".repeat(30)}Remote **Desktop** uses NLA ${"ipsum ".repeat(60)}NLA again`;
  const first = rangesOf(info, "Desktop", "NLA");
  const last = info.lastIndexOf("NLA");
  const excerpt = infoExcerpt(info, [...first, last, last + 3]);
  assert.ok(excerpt);
  assert.ok(excerpt.text.startsWith("…") && excerpt.text.endsWith("…"));
  assert.deepEqual(marked(excerpt), ["Desktop", "NLA"]);
});

test("windows around the earliest range even when ranges are unordered", () => {
  const info = `${"lorem ".repeat(40)}alpha ${"ipsum ".repeat(5)}beta`;
  const excerpt = infoExcerpt(info, rangesOf(info, "beta", "alpha"));
  assert.deepEqual(marked(excerpt), ["alpha", "beta"]);
});

test("no ranges, no excerpt", () => {
  assert.equal(infoExcerpt("Some info.", []), null);
});
