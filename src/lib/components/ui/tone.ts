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
