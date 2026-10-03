import type { IconName } from "$lib/design";

export interface SelectOption<T extends string | number = string | number> {
  value: T;
  label: string;
  disabled?: boolean;
}

export interface SegmentOption<T extends string | number = string | number> extends SelectOption<T> {
  icon?: IconName;
  /** In place of the label's own tooltip, e.g. why the segment is disabled. */
  tooltip?: string;
  /** Choosing it asks first, so arrow keys focus it without choosing it. */
  confirms?: boolean;
}

export interface MeterSegment {
  key: string;
  value: number;
  /** A fill class, e.g. from TONE_FILL. */
  fill: string;
}
