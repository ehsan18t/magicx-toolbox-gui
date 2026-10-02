import { tv, type VariantProps } from "$lib/utils/cn";
import type { Tone } from "$lib/utils/tweakPresentation";

// `enabled:hover:`, not `hover:` plus a `disabled:hover:` reset: a disabled control never reacts.
const GHOST = "bg-transparent text-foreground-muted enabled:hover:bg-muted enabled:hover:text-foreground";

export const button = tv({
  base: "inline-flex items-center justify-center gap-1.5 rounded-md border-0 font-medium cursor-pointer disabled:cursor-not-allowed disabled:opacity-50",
  variants: {
    // Intent names, not tones: `danger` is a destructive action, as in ConfirmOptions.
    variant: {
      primary: "bg-accent text-accent-foreground enabled:hover:bg-accent-hover",
      secondary: "border border-border bg-secondary text-foreground enabled:hover:bg-secondary-hover",
      ghost: GHOST,
      danger: "bg-error text-background enabled:hover:bg-error/90",
      warning: "bg-warning text-warning-foreground enabled:hover:bg-warning/90",
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
  base: `inline-flex shrink-0 cursor-pointer items-center justify-center border-0 ${GHOST} disabled:cursor-not-allowed disabled:opacity-50`,
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
      accent: "bg-accent/15 text-accent",
      success: "bg-success/15 text-success",
      warning: "bg-warning/15 text-warning",
      error: "bg-error/15 text-error",
      info: "bg-info/15 text-info",
      neutral: "bg-muted text-foreground-muted",
    } satisfies Record<Tone, string>,
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

export type CalloutTone = Exclude<Tone, "accent">;

export const callout = tv({
  base: "border",
  variants: {
    tone: {
      success: "border-success/30 bg-success/8",
      warning: "border-warning/30 bg-warning/8",
      error: "border-error/30 bg-error/8",
      info: "border-info/30 bg-info/8",
      neutral: "border-border bg-muted",
    } satisfies Record<CalloutTone, string>,
    density: {
      compact: "rounded-md px-2.5 py-2 text-xs",
      banner: "rounded-lg px-3 py-2.5",
      panel: "rounded-lg p-3",
    },
  },
  defaultVariants: {
    tone: "neutral",
    density: "banner",
  },
});

export type CalloutVariants = VariantProps<typeof callout>;

export const textLink =
  "font-medium underline decoration-foreground-subtle underline-offset-4 hover:text-accent hover:decoration-accent focus-visible:text-accent focus-visible:decoration-accent";
