// Each step is a size from the shipped design, kept apart on purpose: do not merge near neighbours.
/** Icon widths in px; components map their own size names onto these. */
export const ICON_SIZE = {
  "3xs": 12,
  "2xs": 13,
  xs: 14,
  sm: 15,
  md: 16,
  lg: 18,
  xl: 20,
  "2xl": 22,
  "3xl": 24,
  "4xl": 28,
  "5xl": 32,
  "6xl": 40,
  "7xl": 48,
} as const;

export type IconSize = keyof typeof ICON_SIZE;
