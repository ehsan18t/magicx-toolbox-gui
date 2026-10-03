/** The backend stamps a reading when its reads begin: one older than the newest adopted must not replace it. */
export const isStaleReading = (stamp: number, newest: number | undefined): boolean =>
  newest !== undefined && stamp < newest;
