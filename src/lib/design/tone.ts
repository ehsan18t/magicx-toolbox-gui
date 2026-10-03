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
