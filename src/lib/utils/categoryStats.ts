import type { Tallies } from "./tweakPresentation";

/** Every tweak applied; an empty category never counts as complete. */
export const isComplete = ({ applied, total }: Pick<Tallies, "applied" | "total">): boolean =>
  total > 0 && applied === total;
