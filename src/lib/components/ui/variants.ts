import { tv, type VariantProps } from "$lib/utils/cn";
import { TONE_SOFT, TONE_TEXT, TONE_TINT } from "./tone";

/** Every control dims the same when disabled: `DISABLED` through the attribute, `DIMMED` by hand. */
export const DISABLED = "disabled:cursor-not-allowed disabled:opacity-50";
export const DIMMED = "opacity-50";
/** A control busy with its own action: dimmed less, still readable. */
export const BUSY = "cursor-wait opacity-70";

// `enabled:hover:`, not `hover:` plus a `disabled:hover:` reset: a disabled control never reacts.
const GHOST = "bg-transparent text-foreground-muted enabled:hover:bg-muted enabled:hover:text-foreground";

export const button = tv({
  base: ["inline-flex items-center justify-center gap-1.5 rounded-md border-0 font-medium cursor-pointer", DISABLED],
  variants: {
    // Intent names, not tones: `danger` is a destructive action, as in ConfirmOptions.
    variant: {
      primary: "bg-accent text-accent-foreground enabled:hover:bg-accent-hover",
      secondary: "border border-border bg-secondary text-foreground enabled:hover:bg-secondary-hover",
      ghost: GHOST,
      danger: "bg-error text-error-foreground enabled:hover:bg-error-hover",
      warning: "bg-warning text-warning-foreground enabled:hover:bg-warning-hover",
      outline:
        "border border-border bg-transparent text-foreground enabled:hover:bg-muted enabled:hover:border-border-hover",
    },
    size: {
      sm: "h-7 px-2.5 text-xs",
      md: "h-8 px-3 text-ui",
    },
  },
  defaultVariants: {
    variant: "secondary",
    size: "md",
  },
});

export type ButtonVariants = VariantProps<typeof button>;

export const iconButton = tv({
  base: ["inline-flex shrink-0 cursor-pointer items-center justify-center border-0", GHOST, DISABLED],
  variants: {
    size: {
      xs: "h-6 w-6 rounded",
      sm: "h-7 w-7 rounded-md",
      md: "h-8 w-8 rounded-md",
    },
  },
  defaultVariants: {
    size: "md",
  },
});

export type IconButtonSize = NonNullable<VariantProps<typeof iconButton>["size"]>;

export const badge = tv({
  base: "inline-flex items-center gap-1 rounded-md font-semibold tracking-wide uppercase",
  variants: {
    tone: {
      accent: TONE_SOFT.accent,
      success: TONE_SOFT.success,
      warning: TONE_SOFT.warning,
      error: TONE_SOFT.error,
      info: TONE_SOFT.info,
      neutral: TONE_SOFT.neutral,
    },
    size: {
      sm: "px-1.5 py-0.5 text-badge-sm",
      md: "px-2 py-1 text-badge-md",
    },
  },
  defaultVariants: {
    tone: "neutral",
    size: "sm",
  },
});

export type BadgeVariants = VariantProps<typeof badge>;

export const callout = tv({
  slots: {
    base: "border",
    icon: "shrink-0",
  },
  variants: {
    tone: {
      success: { base: TONE_TINT.success, icon: TONE_TEXT.success },
      warning: { base: TONE_TINT.warning, icon: TONE_TEXT.warning },
      error: { base: TONE_TINT.error, icon: TONE_TEXT.error },
      info: { base: TONE_TINT.info, icon: TONE_TEXT.info },
      neutral: { base: TONE_TINT.neutral, icon: TONE_TEXT.neutral },
    },
    density: {
      compact: { base: "gap-2 rounded-md px-2.5 py-2 text-xs", icon: "mt-px" },
      banner: { base: "gap-3 rounded-lg px-3 py-2.5", icon: "mt-0.5" },
      panel: { base: "gap-3 rounded-lg p-3", icon: "mt-0.5" },
    },
    withIcon: {
      true: { base: "flex items-start" },
    },
  },
  defaultVariants: {
    tone: "neutral",
    density: "banner",
  },
});

export type CalloutVariants = Omit<VariantProps<typeof callout>, "withIcon">;

export const modal = tv({
  slots: {
    scrim: "fixed inset-0 z-modal flex items-center justify-center bg-scrim p-4 backdrop-blur-xs",
    panel: "flex max-h-full w-full flex-col overflow-hidden rounded-lg border border-border bg-elevated shadow-dialog",
  },
  variants: {
    size: {
      sm: { panel: "max-w-dialog-sm" },
      md: { panel: "max-w-dialog-md" },
      lg: { panel: "max-w-dialog-lg" },
      full: { panel: "h-full max-w-dialog-full" },
    },
    closing: {
      false: { scrim: "animate-fade-in", panel: "animate-modal-in" },
      true: { scrim: "animate-fade-out", panel: "animate-modal-out" },
    },
  },
  defaultVariants: {
    size: "md",
    closing: false,
  },
});

export type ModalSize = NonNullable<VariantProps<typeof modal>["size"]>;

export const textLink =
  "font-medium underline decoration-foreground-subtle underline-offset-4 hover:text-accent hover:decoration-accent focus-visible:text-accent focus-visible:decoration-accent";
