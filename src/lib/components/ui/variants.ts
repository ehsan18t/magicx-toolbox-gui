import { twMergeConfig } from "$lib/utils/cn";
import { createTV, type VariantProps } from "tailwind-variants";

const tv = createTV({ twMergeConfig });

export const button = tv({
  base: "inline-flex items-center justify-center gap-1.5 rounded-md border-0 font-medium cursor-pointer disabled:cursor-not-allowed disabled:opacity-50",
  variants: {
    variant: {
      primary: "bg-accent text-accent-foreground hover:bg-accent-hover",
      secondary: "border border-border bg-secondary text-foreground hover:bg-secondary-hover",
      ghost: "bg-transparent text-foreground-muted hover:bg-muted hover:text-foreground",
      danger: "bg-error text-background hover:bg-error/90",
      warning: "bg-warning text-warning-foreground hover:bg-warning/90",
      outline: "border border-border bg-transparent text-foreground hover:bg-muted hover:border-border-hover",
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
  base: "flex shrink-0 cursor-pointer items-center justify-center border-0 bg-transparent text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-transparent disabled:hover:text-foreground-muted",
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

export type IconButtonVariants = VariantProps<typeof iconButton>;

export const callout = tv({
  base: "border",
  variants: {
    tone: {
      warning: "border-warning/30 bg-warning/8",
      error: "border-error/30 bg-error/8",
      info: "border-info/30 bg-info/8",
      success: "border-success/30 bg-success/8",
      neutral: "border-border bg-muted",
    },
    size: {
      compact: "rounded-md px-2.5 py-2 text-xs",
      banner: "rounded-lg px-3 py-2.5",
      panel: "rounded-lg p-3",
    },
  },
  compoundVariants: [{ tone: "warning", size: "compact", class: "border-0" }],
  defaultVariants: {
    tone: "warning",
    size: "banner",
  },
});

export type CalloutVariants = VariantProps<typeof callout>;

export const textLink = tv({
  base: "font-medium underline decoration-foreground-subtle underline-offset-4 hover:text-accent hover:decoration-accent",
});
