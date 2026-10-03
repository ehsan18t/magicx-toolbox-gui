import assert from "node:assert/strict";
import { test } from "node:test";
import { markdownToHtml } from "./markdown.ts";

const anchors = (html: string) => [...html.matchAll(/<a\s[^>]*>/g)].map((m) => m[0]);
const attributeNames = (tag: string) => [...tag.matchAll(/\s([\w-]+)=/g)].map((m) => m[1]);

function assertSafeAnchor(tag: string) {
  assert.deepEqual(attributeNames(tag), ["href", "rel", "class"], tag);
}

test("escapes raw HTML", () => {
  const html = markdownToHtml('<script>alert(1)</script> & <img src=x onerror="y">');
  assert.ok(!html.includes("<script"));
  assert.ok(!html.includes("<img"));
  assert.ok(html.includes("&lt;script&gt;alert(1)&lt;/script&gt; &amp; &lt;img"));
});

test("allows http and https links", () => {
  for (const url of ["http://example.com", "https://example.com/a?b=c", "HTTPS://EXAMPLE.COM"]) {
    const [tag] = anchors(markdownToHtml(`[site](${url})`));
    assert.ok(tag, url);
    assert.ok(tag.includes(`href="${url}"`));
    assertSafeAnchor(tag);
  }
});

test("drops a link with any other scheme, keeping its label", () => {
  for (const url of [
    "javascript:alert(1)",
    " JavaScript:alert(1)",
    "data:text/html,<b>x</b>",
    "&#106;avascript:alert(1)",
    "java&#x09;script:alert(1)",
    "//example.com",
    "file:///C:/x",
  ]) {
    const html = markdownToHtml(`[label](${url})`);
    assert.deepEqual(anchors(html), [], url);
    assert.ok(html.includes("label"), url);
  }
});

test("a quote in the href cannot break out of the attribute", () => {
  const [tag] = anchors(markdownToHtml(`[x](https://e.com/"onclick="alert(1)'x)`));
  assert.ok(tag);
  assertSafeAnchor(tag);
  assert.ok(tag.includes("%22onclick=%22"));
});

test("a quote in the link text stays text", () => {
  const html = markdownToHtml(`[a" onmouseover="alert(1)](https://e.com)`);
  const [tag] = anchors(html);
  assertSafeAnchor(tag);
  assert.ok(html.includes(`>a" onmouseover="alert(1)</a>`));
});

test("emphasis is not applied inside a URL", () => {
  const [tag] = anchors(markdownToHtml("[x](https://e.com/a_b_c/**d**/`e`)"));
  assert.ok(tag.includes('href="https://e.com/a_b_c/**d**/`e`"'), tag);
});

test("emphasis still applies to link text and around links", () => {
  const html = markdownToHtml("**bold** [*it*](https://e.com) _x_");
  assert.ok(html.includes('<strong class="font-semibold text-foreground">bold</strong>'));
  assert.ok(html.includes("<em>it</em></a>"));
  assert.ok(html.includes("<em>x</em>"));
});

test("typed placeholder tokens are left as text", () => {
  for (const input of ["@@LINK0@@", "&lt;@0&gt;", "<@0>", "[x](https://e.com) <@0> @@LINK0@@"]) {
    const html = markdownToHtml(input);
    assert.ok(anchors(html).length <= 1, input);
    assert.ok(!html.includes("undefined"), input);
  }
  assert.ok(markdownToHtml("@@LINK0@@").includes("@@LINK0@@"));
  assert.ok(markdownToHtml("<@0>").includes("&lt;@0&gt;"));
});

test("headings, lists and paragraphs", () => {
  const html = markdownToHtml("## Title\n- one\n\n- two\n1. first\ntext");
  assert.match(html, /^<h3 [^>]*>Title<\/h3><ul [^>]*><li [^>]*>one<\/li><li [^>]*>two<\/li><\/ul><ol [^>]*>/);
  assert.ok(html.endsWith('<p class="text-foreground-muted">text</p>'));
  assert.equal(markdownToHtml(""), "");
});
