export type Tone = "accent" | "success" | "warning" | "error" | "info" | "neutral";
/** Text only: `subtle` is a de-emphasised neutral with no fill of its own. */
export type TextTone = Tone | "subtle";
export type TintTone = Exclude<Tone, "accent">;

export const TONE_TEXT: Record<TextTone, string> = {
  accent: "text-accent",
  success: "text-success",
  warning: "text-warning",
  error: "text-error",
  info: "text-info",
  neutral: "text-foreground-muted",
  subtle: "text-foreground-subtle",
};

/** Soft fill with tone text: badges, status chips. */
export const TONE_SOFT: Record<Tone, string> = {
  accent: "bg-accent/15 text-accent",
  success: "bg-success/15 text-success",
  warning: "bg-warning/15 text-warning",
  error: "bg-error/15 text-error",
  info: "bg-info/15 text-info",
  neutral: "bg-muted text-foreground-muted",
};

/** Faint panel with a tone border: callouts. */
export const TONE_TINT: Record<TintTone, string> = {
  success: "border-success/30 bg-success/8",
  warning: "border-warning/30 bg-warning/8",
  error: "border-error/30 bg-error/8",
  info: "border-info/30 bg-info/8",
  neutral: "border-border bg-muted",
};

/** A staged value not applied yet. */
export const PENDING_TINT = "border-warning/50 bg-warning/10";

/** Tones a control can be filled with: each has a matching `-foreground` text token. */
export type FillTone = "accent" | "warning" | "error";

/** A chip at rest: tinted fill and border in its tone. */
export const CHIP_TINT: Record<FillTone, string> = {
  accent: "border-accent/40 bg-accent/10 text-accent enabled:hover:bg-accent/15",
  warning: "border-warning/40 bg-warning/10 text-warning enabled:hover:bg-warning/15",
  error: "border-error/40 bg-error/10 text-error enabled:hover:bg-error/15",
};

/** A chip switched on: solid fill. */
export const CHIP_SOLID: Record<FillTone, string> = {
  accent: "border-accent bg-accent text-accent-foreground",
  warning: "border-warning bg-warning text-warning-foreground",
  error: "border-error bg-error text-error-foreground",
};

/** Hover wash for a neutral control that acts in a tone, e.g. a delete in a list. */
export const TONE_WASH: Record<FillTone, string> = {
  accent: "enabled:hover:bg-accent/10 enabled:hover:text-accent",
  warning: "enabled:hover:bg-warning/10 enabled:hover:text-warning",
  error: "enabled:hover:bg-error/10 enabled:hover:text-error",
};

/** An item card's border while it is the selected one or holds a staged change. */
export const EMPHASIS_BORDER = {
  selected: "border-accent/70",
  pending: "border-warning/45",
} as const;

/** A highlighted table column: the live values, the applied option, or the staged one. */
export type ColumnTone = "now" | "current" | "pending";

/** The header sits a step above its cells. */
export const COLUMN_TINT: Record<ColumnTone, { head: string; cell: string }> = {
  now: { head: "bg-warning/8", cell: "bg-warning/5" },
  current: { head: "bg-accent/12", cell: "bg-accent/8" },
  pending: { head: "bg-warning/10", cell: "bg-warning/6" },
};

/** A table's header row. */
export const TABLE_HEAD = "bg-muted/40";

/** Veils the page under a dragged file, leaving it faintly visible. */
export const DROP_VEIL = "bg-background/85";
