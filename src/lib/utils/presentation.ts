import type { IconName, TextTone } from "$lib/design";
import { capitalize } from "$lib/utils/format";

/** One fact on an item's meta line: MetaItem's props. */
export interface MetaFact {
  icon: IconName;
  label: string;
  tone?: TextTone;
  tooltip?: string;
  spin?: boolean;
}

export const CHECKING: MetaFact = { label: "Checking…", icon: "mdi:loading", tone: "neutral", spin: true };
export const UNKNOWN_ICON = "mdi:help-circle-outline";
export const UNKNOWN_NEEDS_ADMIN = "Unknown, needs admin";
/** The remedy when only elevation can read or change something, as a clause; ELEVATE_HINT is the sentence. */
export const ELEVATE_REMEDY = "restart as administrator to resolve";
export const ELEVATE_HINT = `${capitalize(ELEVATE_REMEDY)}.`;
