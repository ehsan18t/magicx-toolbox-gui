/**
 * Shared tailwind-variants definitions for the UI component system
 * @see https://www.tailwind-variants.org/
 */
import { tv, type VariantProps } from "tailwind-variants";

// =============================================================================
// BUTTON VARIANTS
// =============================================================================

/**
 * Base button variant - used by Button, ActionButton, LinkButton
 */
export const button = tv({
  base: "inline-flex items-center justify-center gap-2 rounded-lg border-0 font-medium transition-all duration-150 cursor-pointer disabled:cursor-not-allowed disabled:opacity-60",
  variants: {
    variant: {
      primary: "bg-accent text-accent-foreground hover:bg-accent-hover",
      secondary: "bg-muted text-foreground hover:bg-muted/80",
      ghost: "bg-transparent text-foreground-muted hover:bg-muted hover:text-foreground",
      danger: "bg-error text-white hover:bg-error/90",
      warning: "bg-warning text-black hover:bg-warning/90",
      success: "bg-success text-white hover:bg-success/90",
      outline: "border border-border bg-card text-foreground hover:bg-foreground/5 hover:border-foreground-muted",
    },
    size: {
      xs: "px-2 py-1 text-[11px]",
      sm: "px-3 py-1.5 text-xs",
      md: "px-4 py-2 text-sm",
      lg: "px-5 py-2.5 text-base",
    },
    fullWidth: {
      true: "w-full",
    },
  },
  defaultVariants: {
    variant: "secondary",
    size: "md",
  },
});

export type ButtonVariants = VariantProps<typeof button>;

/**
 * Action button variant - toolbar buttons with icon + optional badge
 * Used in CategoryTab, FavoritesTab, SnapshotsTab, SearchTab toolbars
 */
export const actionButton = tv({
  base: "flex cursor-pointer items-center gap-2 rounded-lg border bg-card text-sm font-medium text-foreground transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-50",
  variants: {
    intent: {
      default: "border-border hover:not-disabled:border-foreground-muted hover:not-disabled:bg-foreground/5",
      apply:
        "border-border hover:not-disabled:border-success hover:not-disabled:bg-success/15 hover:not-disabled:text-success",
      discard: "border-border hover:not-disabled:border-foreground-muted hover:not-disabled:bg-foreground/5",
      restore:
        "border-border hover:not-disabled:border-error hover:not-disabled:bg-error/15 hover:not-disabled:text-error",
      accent:
        "border-border hover:not-disabled:border-accent hover:not-disabled:bg-accent/15 hover:not-disabled:text-accent",
      danger:
        "border-border hover:not-disabled:border-error hover:not-disabled:bg-error/15 hover:not-disabled:text-error",
    },
    size: {
      sm: "px-3 py-2",
      md: "px-4 py-2.5",
    },
    active: {
      true: "",
      false: "",
    },
  },
  compoundVariants: [
    {
      intent: "apply",
      active: true,
      class: "border-warning bg-warning/15 text-warning",
    },
  ],
  defaultVariants: {
    intent: "default",
    size: "md",
    active: false,
  },
});

export type ActionButtonVariants = VariantProps<typeof actionButton>;

/**
 * Link button variant - styled links/buttons for AboutModal, etc.
 */
export const linkButton = tv({
  base: "flex items-center justify-center gap-2 rounded-lg border border-border bg-surface text-foreground transition-colors hover:bg-muted",
  variants: {
    size: {
      sm: "px-2.5 py-2 text-xs",
      md: "px-3 py-2.5 text-sm",
    },
  },
  defaultVariants: {
    size: "md",
  },
});

export type LinkButtonVariants = VariantProps<typeof linkButton>;

/**
 * Icon button variant - small icon-only buttons
 */
export const iconButton = tv({
  base: "flex shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-foreground-muted transition-colors hover:text-foreground disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-transparent disabled:hover:text-foreground-muted",
  variants: {
    size: {
      sm: "h-6 w-6",
      md: "h-8 w-8",
      lg: "h-10 w-10",
    },
    variant: {
      ghost: "hover:bg-muted",
      subtle: "hover:bg-foreground/5",
    },
  },
  defaultVariants: {
    size: "md",
    variant: "ghost",
  },
});

export type IconButtonVariants = VariantProps<typeof iconButton>;

// =============================================================================
// BADGE VARIANTS
// =============================================================================

/**
 * Status pill/badge variant - for risk level, permission, reboot required, etc.
 */
export const statusBadge = tv({
  base: "inline-flex cursor-help items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase transition-colors duration-150",
  variants: {
    variant: {
      success: "bg-success/8 text-success hover:bg-success/15",
      warning: "bg-warning/8 text-warning hover:bg-warning/15",
      error: "bg-error/8 text-error hover:bg-error/15",
      orange: "bg-orange-500/8 text-orange-500 hover:bg-orange-500/15",
      info: "bg-info/8 text-info hover:bg-info/15",
      muted: "bg-muted/50 text-foreground-muted hover:bg-muted hover:text-foreground-muted",
      accent: "bg-accent/10 text-accent",
    },
  },
  defaultVariants: {
    variant: "muted",
  },
});

export type StatusBadgeVariants = VariantProps<typeof statusBadge>;

/**
 * Counter badge - small round badge for counts
 */
export const counterBadge = tv({
  base: "inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-bold",
  variants: {
    variant: {
      warning: "bg-warning text-white",
      error: "bg-error/20 text-error",
      accent: "bg-accent/20 text-accent",
      success: "bg-success/20 text-success",
      muted: "bg-muted text-foreground-muted",
    },
    size: {
      sm: "h-4 min-w-4 text-[10px]",
      md: "h-5 min-w-5 text-xs",
    },
  },
  defaultVariants: {
    variant: "warning",
    size: "md",
  },
});

export type CounterBadgeVariants = VariantProps<typeof counterBadge>;

// =============================================================================
// CARD VARIANTS
// =============================================================================

/**
 * Card container variant
 */
export const card = tv({
  base: "rounded-lg border transition-all duration-200",
  variants: {
    variant: {
      default: "border-border bg-card",
      elevated: "border-border bg-elevated shadow-md",
      outlined: "border-border bg-transparent",
      ghost: "border-transparent bg-transparent",
      surface: "border-border bg-surface",
    },
    hover: {
      true: "hover:border-border-hover hover:shadow-md",
    },
    padding: {
      none: "",
      sm: "p-2",
      md: "p-4",
      lg: "p-6",
    },
  },
  defaultVariants: {
    variant: "default",
    hover: false,
    padding: "md",
  },
});

export type CardVariants = VariantProps<typeof card>;

/**
 * Panel variant - for sections like toolbar panels
 */
export const panel = tv({
  base: "flex items-center gap-4 rounded-xl border border-border bg-card",
  variants: {
    size: {
      sm: "px-3 py-2",
      md: "px-5 py-3",
    },
  },
  defaultVariants: {
    size: "md",
  },
});

export type PanelVariants = VariantProps<typeof panel>;

// =============================================================================
// LAYOUT VARIANTS
// =============================================================================

/**
 * Flex row variant - common flex patterns
 */
export const flexRow = tv({
  base: "flex items-center",
  variants: {
    gap: {
      none: "",
      xs: "gap-1",
      sm: "gap-2",
      md: "gap-3",
      lg: "gap-4",
    },
    justify: {
      start: "justify-start",
      center: "justify-center",
      end: "justify-end",
      between: "justify-between",
    },
    wrap: {
      true: "flex-wrap",
    },
  },
  defaultVariants: {
    gap: "sm",
    justify: "start",
    wrap: false,
  },
});

export type FlexRowVariants = VariantProps<typeof flexRow>;

// =============================================================================
// TEXT VARIANTS
// =============================================================================

/**
 * Section heading variant
 */
export const sectionHeading = tv({
  base: "m-0 flex items-center gap-2 font-semibold text-foreground",
  variants: {
    size: {
      xs: "mb-2 text-xs tracking-wide text-foreground-muted",
      sm: "mb-3 text-sm",
      md: "mb-4 text-base",
    },
    uppercase: {
      true: "uppercase",
    },
  },
  defaultVariants: {
    size: "md",
    uppercase: false,
  },
});

export type SectionHeadingVariants = VariantProps<typeof sectionHeading>;
