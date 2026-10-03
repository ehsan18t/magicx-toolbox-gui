import { tv, type VariantProps } from "$lib/utils/cn";
import { CHIP_QUIET_HOVER, CHIP_SOLID, CHIP_TINT, TONE_SOFT, TONE_TEXT, TONE_TINT } from "$lib/design";

/** Every control dims the same when disabled: `DISABLED` through the attribute, `DIMMED` by hand. */
export const DISABLED = "disabled:cursor-not-allowed disabled:opacity-50";
export const DIMMED = "opacity-50";
/** A control busy with its own action: dimmed less, still readable. */
export const BUSY = "cursor-wait opacity-70";
/** Disabled because something holds it in its state, e.g. a warning kept open: dimmed like BUSY. */
export const HELD = "disabled:cursor-default disabled:opacity-70";

// `enabled:hover:`, not `hover:` plus a `disabled:hover:` reset: a disabled control never reacts.
const GHOST = "bg-transparent text-foreground-muted enabled:hover:bg-muted enabled:hover:text-foreground";
// `not-disabled:`, as `enabled:` never matches an <a>: link icon buttons share the look.
const GHOST_LINKABLE =
  "bg-transparent text-foreground-muted not-disabled:hover:bg-muted not-disabled:hover:text-foreground";

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
    /** Text colour on a neutral variant, e.g. a secondary Remove in error. */
    tone: {
      foreground: "text-foreground",
      accent: "text-accent enabled:hover:text-accent",
      warning: "text-warning enabled:hover:text-warning",
      error: "text-error enabled:hover:text-error",
    },
  },
  defaultVariants: {
    variant: "secondary",
    size: "md",
  },
});

export type ButtonVariants = VariantProps<typeof button>;

export const iconButton = tv({
  base: ["inline-flex shrink-0 cursor-pointer items-center justify-center border-0", GHOST_LINKABLE, DISABLED],
  variants: {
    size: {
      xs: "h-6 w-6 rounded",
      sm: "h-7 w-7 rounded-md",
      md: "h-8 w-8 rounded-md",
    },
    /** The colour while `active`: an open panel, a starred item. */
    tone: {
      accent: "",
      warning: "",
    },
    active: {
      true: "",
    },
  },
  compoundVariants: [
    { active: true, tone: "accent", class: "text-accent not-disabled:hover:text-accent" },
    { active: true, tone: "warning", class: "text-warning not-disabled:hover:text-warning" },
  ],
  defaultVariants: {
    size: "md",
    tone: "accent",
  },
});

export type IconButtonVariants = VariantProps<typeof iconButton>;
export type IconButtonSize = NonNullable<IconButtonVariants["size"]>;

export const badge = tv({
  base: "inline-flex items-center gap-1 font-semibold",
  variants: {
    tone: {
      accent: TONE_SOFT.accent,
      success: TONE_SOFT.success,
      warning: TONE_SOFT.warning,
      error: TONE_SOFT.error,
      info: TONE_SOFT.info,
      neutral: TONE_SOFT.neutral,
    },
    /** `caption`: a status chip in running text; `count`: a number in a pill. */
    size: {
      sm: "rounded-md px-1.5 py-0.5 text-badge-sm",
      md: "rounded-md px-2 py-1 text-badge-md",
      caption: "rounded px-1.5 py-0.5 text-caption",
      count: "h-6 min-w-6 justify-center rounded-full px-1.5 text-xs font-bold tabular-nums",
    },
    case: {
      upper: "tracking-wide uppercase",
      none: "",
    },
    solid: {
      true: "",
    },
  },
  compoundVariants: [
    { solid: true, tone: "accent", class: CHIP_SOLID.accent },
    { solid: true, tone: "warning", class: CHIP_SOLID.warning },
    { solid: true, tone: "error", class: CHIP_SOLID.error },
  ],
  defaultVariants: {
    tone: "neutral",
    size: "sm",
    case: "upper",
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
      /** Neutral on the card colour, for notes that are not warnings. */
      surface: { base: "border-border bg-card", icon: TONE_TEXT.neutral },
    },
    density: {
      compact: { base: "gap-2 rounded-md px-2.5 py-2 text-xs", icon: "mt-px" },
      banner: { base: "gap-3 rounded-lg px-3 py-2.5", icon: "mt-0.5" },
      panel: { base: "gap-3 rounded-lg p-3", icon: "mt-0.5" },
      /** A short note in a dialog. */
      note: { base: "gap-2 rounded-lg px-3 py-2 text-sm", icon: "mt-0.5" },
    },
    withIcon: {
      true: { base: "flex items-start" },
    },
    bordered: {
      true: {},
      false: { base: "border-0" },
    },
  },
  defaultVariants: {
    tone: "neutral",
    density: "banner",
    bordered: true,
  },
});

export type CalloutVariants = Omit<VariantProps<typeof callout>, "withIcon">;

export const card = tv({
  base: "border border-border",
  variants: {
    radius: {
      md: "rounded-md",
      lg: "rounded-lg",
    },
    /** Raised above the page, e.g. a toast, drawer or busy panel. */
    elevation: {
      none: "bg-card",
      flyout: "bg-elevated shadow-flyout",
      dialog: "bg-elevated shadow-dialog",
    },
  },
  defaultVariants: {
    radius: "lg",
    elevation: "none",
  },
});

export type CardVariants = VariantProps<typeof card>;

/** A row of MetaItems under a title. */
export const META_LINE = "flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs";

/** A list row that is one button, e.g. a summary entry or a category link. */
export const rowButton = tv({
  base: "cursor-pointer text-left hover:bg-muted",
  variants: {
    radius: {
      md: "rounded-md",
      sm: "rounded",
      none: "",
    },
  },
  defaultVariants: {
    radius: "md",
  },
});

export const toggleChip = tv({
  base: ["inline-flex cursor-pointer items-center rounded-full border font-medium whitespace-nowrap", DISABLED],
  variants: {
    size: {
      sm: "gap-1 px-2 py-0.5 text-xs",
      md: "h-8 gap-1.5 px-3 text-ui",
    },
    /** `quiet` is neutral until its tone shows on hover; `tint` and `solid` are the off and on looks. */
    variant: {
      quiet: "border-border-hover bg-muted text-foreground-muted",
      tint: "",
      solid: "",
    },
    tone: {
      accent: "",
      warning: "",
      error: "",
    },
  },
  compoundVariants: [
    { variant: "quiet", tone: "accent", class: CHIP_QUIET_HOVER.accent },
    { variant: "quiet", tone: "warning", class: CHIP_QUIET_HOVER.warning },
    { variant: "quiet", tone: "error", class: CHIP_QUIET_HOVER.error },
    { variant: "tint", tone: "accent", class: CHIP_TINT.accent },
    { variant: "tint", tone: "warning", class: CHIP_TINT.warning },
    { variant: "tint", tone: "error", class: CHIP_TINT.error },
    { variant: "solid", tone: "accent", class: CHIP_SOLID.accent },
    { variant: "solid", tone: "warning", class: CHIP_SOLID.warning },
    { variant: "solid", tone: "error", class: CHIP_SOLID.error },
  ],
  defaultVariants: {
    size: "sm",
    variant: "tint",
    tone: "accent",
  },
});

export type ToggleChipVariants = VariantProps<typeof toggleChip>;

export const iconTile = tv({
  base: "flex shrink-0 items-center justify-center",
  variants: {
    size: {
      sm: "h-8 w-8",
      md: "h-10 w-10",
      lg: "h-12 w-12",
      xl: "h-14 w-14",
      "2xl": "h-16 w-16",
      "3xl": "h-20 w-20",
      "4xl": "h-24 w-24",
    },
    shape: {
      square: "",
      circle: "rounded-full",
    },
    tone: {
      accent: TONE_SOFT.accent,
      success: TONE_SOFT.success,
      warning: TONE_SOFT.warning,
      error: TONE_SOFT.error,
      info: TONE_SOFT.info,
      neutral: TONE_SOFT.neutral,
    },
  },
  // A square's corner grows with it.
  compoundVariants: [
    { shape: "square", size: "sm", class: "rounded-md" },
    { shape: "square", size: ["md", "lg"], class: "rounded-lg" },
    { shape: "square", size: ["xl", "2xl", "3xl", "4xl"], class: "rounded-xl" },
  ],
  defaultVariants: {
    size: "md",
    shape: "square",
    tone: "accent",
  },
});

export type IconTileVariants = VariantProps<typeof iconTile>;

export const link = tv({
  base: "cursor-pointer font-medium",
  variants: {
    /** `underline`: always underlined, for links in prose; `hover`: underlined on hover, for inline actions. */
    variant: {
      underline:
        "underline decoration-foreground-subtle underline-offset-4 hover:text-accent hover:decoration-accent focus-visible:text-accent focus-visible:decoration-accent",
      hover: "underline-offset-2 hover:underline focus-visible:underline",
    },
    /** `inherit` takes the surrounding text colour. */
    tone: {
      inherit: "",
      foreground: "text-foreground",
      accent: "text-accent",
      muted: "text-foreground-muted",
    },
  },
  defaultVariants: {
    variant: "underline",
    tone: "inherit",
  },
});

export type LinkVariants = VariantProps<typeof link>;

/** The box every input shares: TextField, TextArea, SearchInput and Select's trigger. */
export const field = tv({
  base: "rounded-md border border-border bg-secondary text-ui text-foreground",
  variants: {
    /** Where focus lights the border: `self` an input, `within` a box around one, `none` a trigger that shows its open state. */
    focus: {
      self: ["w-full px-2.5 outline-none placeholder:text-foreground-subtle focus:border-accent", DISABLED],
      within: "flex min-w-0 items-center gap-2 px-2.5 transition-colors focus-within:border-accent",
      none: "",
    },
    multiline: {
      false: "h-8",
      true: "resize-none py-2",
    },
  },
  defaultVariants: {
    focus: "self",
    multiline: false,
  },
});

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

/** A close button pinned to a dialog's corner, over a layout with no header row. */
export const FLOATING_CLOSE = "absolute top-3 right-3";

/** Side padding of a dialog led by a large title under a floating close (About, Updates): a step past px-6. */
export const WIDE_DIALOG_INSET = "px-7";

/** The bar that marks an item: a `pill` beside the current entry, a `stripe` down a row's edge. */
export const indicator = tv({
  base: "absolute left-0 w-0.75",
  variants: {
    shape: {
      pill: "h-4 -translate-y-1/2 rounded-full bg-accent",
      stripe: "top-3 bottom-3 rounded-r-full",
    },
  },
  defaultVariants: {
    shape: "pill",
  },
});
