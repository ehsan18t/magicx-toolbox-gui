export interface SelectOption<T extends string | number = string | number> {
  value: T;
  label: string;
  disabled?: boolean;
}

export interface SegmentOption<T extends string | number = string | number> extends SelectOption<T> {
  icon?: string;
  /** Tooltip in place of the label, e.g. why the segment is disabled. */
  tip?: string;
}
