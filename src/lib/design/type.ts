// Class strings, not @utility: cn/tailwind-merge must see the real classes to resolve a call site's override.
/** Heading looks by role; margins and colour stay with the caller. */
export const HEADING = {
  page: "font-display text-title leading-tight font-semibold",
  /** The About dialog's title. */
  hero: "font-display text-hero leading-none font-semibold tracking-display",
  /** A detail dialog's title. */
  dialog: "font-display text-xl font-semibold",
  /** A side pane's or content dialog's title. */
  pane: "font-display text-lg font-semibold",
  /** A status line that leads a dialog, e.g. "You're up to date". */
  status: "text-lg font-semibold",
  /** A finished operation's outcome. */
  outcome: "text-xl font-bold",
  /** A full-surface prompt, e.g. over a dragged file. */
  prompt: "text-xl font-semibold",
  /** A section in a page or dialog, a card's title, an empty state. */
  section: "text-base font-semibold",
  /** An item in a list, a panel's title. */
  item: "text-sm font-semibold",
  /** A card header or a group inside a panel. */
  group: "text-ui font-semibold",
} as const;
