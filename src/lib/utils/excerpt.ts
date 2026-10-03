// Explicit extension: node --test loads this module without a bundler.
import { SEP } from "./format.ts";

/** Characters of context kept before the first match. */
export const EXCERPT_LEAD = 40;
/** Longest excerpt before the ellipses; the row truncates whatever its width cannot show. */
export const EXCERPT_LENGTH = 160;
const ELLIPSIS = "…";

const LINE_MARKER = /[ \t]*(?:#{1,6}[ \t]+|[-*+][ \t]+|\d+\.[ \t]+)?/y;
const LINK = /\[([^\]\n]+)\]\([^)\n]+\)/y;

export interface Excerpt {
  text: string;
  /** Flat [start, end, …] pairs over `text`. */
  ranges: number[];
}

/** Plain single-line text plus, per character, its index in `source`. */
function plain(source: string): { text: string; origin: number[] } {
  let text = "";
  const origin: number[] = [];
  let linkLabelEnd = -1;
  let linkEnd = -1;
  let i = 0;
  const skipLineMarker = () => {
    LINE_MARKER.lastIndex = i;
    if (LINE_MARKER.exec(source)) i = LINE_MARKER.lastIndex;
  };
  skipLineMarker();
  while (i < source.length) {
    if (i === linkLabelEnd) {
      i = linkEnd;
      continue;
    }
    const ch = source[i];
    const pair = source.slice(i, i + 2);
    if (pair === "**" || pair === "__") {
      i += 2;
      continue;
    }
    if (ch === "`") {
      i++;
      continue;
    }
    if (ch === "[") {
      LINK.lastIndex = i;
      const link = LINK.exec(source);
      if (link) {
        linkLabelEnd = i + 1 + link[1].length;
        linkEnd = LINK.lastIndex;
        i++;
        continue;
      }
    }
    if (ch === "\n") {
      // Info keeps one block per line, so a line break separates blocks (a heading from its text).
      while (text.endsWith(" ") && !text.endsWith(SEP)) {
        text = text.slice(0, -1);
        origin.pop();
      }
      if (text && !text.endsWith(SEP)) {
        text += SEP;
        for (let k = 0; k < SEP.length; k++) origin.push(i);
      }
      i++;
      skipLineMarker();
      continue;
    }
    if (/\s/.test(ch)) {
      if (text && !text.endsWith(" ")) {
        text += " ";
        origin.push(i);
      }
      i++;
      continue;
    }
    text += ch;
    origin.push(i);
    i++;
  }
  const tail = text.endsWith(SEP) ? SEP.length : text.endsWith(" ") ? 1 : 0;
  text = text.slice(0, text.length - tail);
  origin.length = text.length;
  return { text, origin };
}

/** First plain index whose source index is at or past `sourceIndex`. */
function lowerBound(origin: number[], sourceIndex: number): number {
  let lo = 0;
  let hi = origin.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (origin[mid] < sourceIndex) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** A one-line, markdown-free window of `info` around its first match; null when no match survives stripping. */
export function infoExcerpt(info: string, ranges: number[]): Excerpt | null {
  const { text, origin } = plain(info);
  const mapped: [number, number][] = [];
  for (let r = 0; r + 1 < ranges.length; r += 2) {
    const start = lowerBound(origin, ranges[r]);
    const end = lowerBound(origin, ranges[r + 1]);
    if (start < end) mapped.push([start, end]);
  }
  if (mapped.length === 0) return null;
  mapped.sort((a, b) => a[0] - b[0]);
  const [[firstStart, firstEnd]] = mapped;

  let start = Math.max(0, firstStart - EXCERPT_LEAD);
  if (start > 0 && text[start - 1] !== " ") {
    const space = text.indexOf(" ", start);
    if (space !== -1 && space < firstStart) start = space + 1;
  }
  let end = Math.min(text.length, Math.max(start + EXCERPT_LENGTH, firstEnd));
  if (end < text.length && text[end] !== " ") {
    const space = text.lastIndexOf(" ", end);
    if (space >= firstEnd) end = space;
  }
  // Never open or close the window on a block separator.
  const sepMark = SEP.trim();
  if (text.startsWith(sepMark, start)) start += sepMark.length + 1;
  if (text.slice(0, end).endsWith(` ${sepMark}`)) end -= sepMark.length + 1;

  const lead = start > 0 ? ELLIPSIS : "";
  const out: number[] = [];
  for (const [rangeStart, rangeEnd] of mapped) {
    const s = Math.max(rangeStart, start);
    const e = Math.min(rangeEnd, end);
    if (s < e) out.push(s - start + lead.length, e - start + lead.length);
  }
  return { text: lead + text.slice(start, end) + (end < text.length ? ELLIPSIS : ""), ranges: out };
}
