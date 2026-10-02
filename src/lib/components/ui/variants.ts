/**
 * Shared tailwind-variants definitions for the UI component system
 * @see https://www.tailwind-variants.org/
 */
import { tv, type VariantProps } from "tailwind-variants";

// =============================================================================
// BUTTON VARIANTS
// =============================================================================

/**
 * Base button variant - used by Button
 */
export const button = tv({
  base: "inline-flex items-center justify-center gap-2 rounded-md border-0 font-medium transition-colors duration-150 cursor-pointer disabled:cursor-not-allowed disabled:opacity-60",
  variants: {
    variant: {
      primary: "bg-accent text-accent-foreground hover:bg-accent-hover",
      secondary: "border border-border bg-secondary text-foreground hover:bg-secondary-hover",
      ghost: "bg-transparent text-foreground-muted hover:bg-muted hover:text-foreground",
      danger: "bg-error text-background hover:bg-error/90",
      warning: "bg-warning text-warning-foreground hover:bg-warning/90",
      success: "bg-success text-background hover:bg-success/90",
      outline: "border border-border bg-transparent text-foreground hover:bg-muted hover:border-border-hover",
    },
    size: {
      xs: "h-6 px-2 text-xs",
      sm: "h-7 px-2.5 text-xs",
      md: "h-8 px-3 text-[13px]",
      lg: "h-9 px-4 text-sm",
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
 * Icon button variant - small icon-only buttons
 */
export const iconButton = tv({
  base: "flex shrink-0 cursor-pointer items-center justify-center rounded-md border-0 bg-transparent text-foreground-muted transition-colors hover:text-foreground disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-transparent disabled:hover:text-foreground-muted",
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
 * Counter badge - small round badge for counts
 */
export const counterBadge = tv({
  base: "inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-bold",
  variants: {
    variant: {
      warning: "bg-warning text-warning-foreground",
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
      elevated: "border-border bg-elevated shadow-flyout",
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
