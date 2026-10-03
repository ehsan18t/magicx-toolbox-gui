import type { IconName, TextTone, Tone } from "$lib/design";
import type {
  Attention,
  Availability,
  BackendRiskLevel,
  ItemKind,
  Level,
  RiskLevel,
  TweakDefinition,
  TweakStatus,
  TweakWithStatus,
} from "$lib/types";
import { plural } from "$lib/utils/format";

export const SYSTEM_DEFAULT_LABEL = "System default";

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
export const ELEVATE_HINT = "Restart as administrator to resolve.";

const RISK_LEVEL: Record<BackendRiskLevel, RiskLevel> = {
  Low: "low",
  Medium: "medium",
  High: "high",
  Critical: "critical",
};

export const toRiskLevel = (risk: BackendRiskLevel): RiskLevel => RISK_LEVEL[risk];

export const RISK_INFO: Record<RiskLevel, { name: string; description: string }> = {
  low: { name: "Low", description: "Safe to apply/revert without issues" },
  medium: { name: "Medium", description: "May require restart or have minor side effects" },
  high: { name: "High", description: "Could significantly impact system" },
  critical: { name: "Critical", description: "Could break Windows, use with caution" },
};

export const RISK_TONE: Record<RiskLevel, Tone> = {
  low: "success",
  medium: "warning",
  high: "error",
  critical: "error",
};

export const isHighRisk = (level: RiskLevel): boolean => level === "high" || level === "critical";

export const riskFact = (level: RiskLevel): MetaFact => ({
  icon: "mdi:shield-half-full",
  label: `${RISK_INFO[level].name} risk`,
  tone: RISK_TONE[level],
  tooltip: RISK_INFO[level].description,
});

const PERMISSION_FACT: Record<Level, MetaFact> = {
  User: { icon: "mdi:account", label: "Standard user", tone: "neutral" },
  Admin: {
    icon: "mdi:shield-account-outline",
    label: "Admin",
    tone: "neutral",
    tooltip: "Requires Administrator privileges to apply",
  },
  Ti: {
    icon: "mdi:shield-key",
    label: "TrustedInstaller",
    tone: "neutral",
    tooltip: "Requires TrustedInstaller elevation for highly protected resources",
  },
};

export const permissionFact = (level: Level): MetaFact => PERMISSION_FACT[level];

const ATTENTION_CAUSE: Record<Attention["reason"], string> = {
  apply_failed: "The last apply couldn't be fully verified",
  restore_failed: "The last restore didn't fully complete",
  crash_residue: "A change to this tweak was never recorded as finished, so part of it is unconfirmed",
  outcome_unrecorded: "The last operation ended in a verified state, but the app couldn't record that",
  record_unreadable: "This tweak's Needs Attention record couldn't be read, so whatever it holds is unresolved",
};

/** Falls back, so a reason added in Rust never interpolates `undefined` into the UI. */
export function attentionCause(reason: Attention["reason"] | undefined): string {
  return (reason && ATTENTION_CAUSE[reason]) || "The last operation couldn't be fully verified";
}

/** Above this many options a tweak's control is a dropdown, not a segmented switch. */
const SEGMENTED_MAX = 2;

export const labelsOf = (def: TweakDefinition): string[] => def.options.map((o) => o.label);

export const usesDropdown = (def: TweakDefinition): boolean => def.options.length > SEGMENTED_MAX;

export const rowDomId = (kind: ItemKind, id: string): string => `${kind}-${id}`;

/** Groups in first-seen order, so a sorted list stays sorted. */
export function groupByCategory(tweaks: TweakWithStatus[]): [categoryId: string, tweaks: TweakWithStatus[]][] {
  const groups = new Map<string, TweakWithStatus[]>();
  for (const t of tweaks) {
    const group = groups.get(t.definition.categoryId);
    if (group) group.push(t);
    else groups.set(t.definition.categoryId, [t]);
  }
  return [...groups];
}

/** A bulk restore takes only these: each row's own Restore is disabled otherwise. */
export const canRestore = (t: TweakWithStatus): boolean =>
  t.status.hasHistory && t.definition.availability.state === "available";

export const restoreMessage = (count: number): string =>
  `Restore ${plural(count, "tweak")} to the state saved before ${count === 1 ? "its" : "each one's"} last change?`;

export const ensureSentence = (text: string): string => (/[.!?]$/.test(text) ? text : `${text}.`);

export interface Tallies {
  total: number;
  applied: number;
  attention: number;
  withSnapshot: number;
  needsAdmin: number;
  byState: Record<TweakStatus["state"], number>;
}

/** Every count a page or panel shows, in one pass. */
export function tallies(list: TweakWithStatus[]): Tallies {
  const t: Tallies = {
    total: list.length,
    applied: 0,
    attention: 0,
    withSnapshot: 0,
    needsAdmin: 0,
    byState: { loading: 0, active: 0, system_default: 0, unavailable: 0, unknown: 0 },
  };
  for (const { definition, status } of list) {
    if (status.state === "active") t.applied++;
    if (status.attention) t.attention++;
    if (status.hasHistory) t.withSnapshot++;
    if (definition.availability.state === "needs_elevation") t.needsAdmin++;
    t.byState[status.state]++;
  }
  return t;
}

export const unavailableReason = (status: TweakStatus): string =>
  status.unavailableReason ?? "Not available on this system";

function stateTip(status: TweakStatus): string {
  switch (status.state) {
    case "unknown": {
      const causes = status.unknownReasons.map((r) => `${r.effect}: ${r.cause}`).join("; ");
      const base = `Could not read this tweak's state (${causes || "unknown"}).`;
      return status.needsElevation ? `${base} ${ELEVATE_HINT}` : base;
    }
    case "unavailable":
      return unavailableReason(status);
    default:
      return "Current state";
  }
}

// Each switch keeps a default: a state added in Rust must not crash the rows.
function stateLook(status: TweakStatus): MetaFact {
  switch (status.state) {
    case "active":
      return { label: status.activeOption ?? "Active", tone: "accent", icon: "mdi:check-circle" };
    case "system_default":
      return { label: SYSTEM_DEFAULT_LABEL, tone: "neutral", icon: "mdi:monitor" };
    case "unavailable":
      return { label: "Unavailable", tone: "neutral", icon: "mdi:cancel" };
    case "loading":
      return CHECKING;
    // A state this build does not know is not still loading: a static label, not a spinner.
    case "unknown":
    default:
      return { label: status.needsElevation ? UNKNOWN_NEEDS_ADMIN : "Unknown", tone: "warning", icon: UNKNOWN_ICON };
  }
}

export const stateSummary = (status: TweakStatus): MetaFact => ({ ...stateLook(status), tooltip: stateTip(status) });

export const residueText = (status: TweakStatus): string =>
  `Residual settings remain outside the active option: ${status.residues.join(", ")}`;

/** Finds any tweak by id, e.g. `tweaksStore.tweak`. */
export type TweakLookup = (tweakId: string) => TweakWithStatus | undefined;

/** Names each holder, falling back to its id. */
export function heldSharedText(status: TweakStatus, find: TweakLookup): string {
  const name = (id: string) => find(id)?.definition.name ?? id;
  return `Shared settings held: ${status.heldShared.map((h) => `${h.shared} (${h.holders.map(name).join(", ")})`).join("; ")}`;
}

export const pendingFact = (optionLabel: string): MetaFact => ({
  icon: "mdi:arrow-right",
  label: `${optionLabel} pending`,
  tone: "warning",
  tooltip: "Staged, not applied yet",
});

/** Every fact a tweak's meta line can show; the row and the details header each pick theirs. */
export interface TweakMeta {
  state: MetaFact;
  risk: MetaFact;
  permission: MetaFact;
  restart: MetaFact | null;
  irreversible: MetaFact | null;
  /** Null while the state already says the tweak is unavailable. */
  availability: MetaFact | null;
  residue: MetaFact | null;
  shared: MetaFact | null;
  snapshot: MetaFact;
}

export function tweakMeta({ definition: def, status }: TweakWithStatus, find: TweakLookup): TweakMeta {
  const { availability } = def;
  return {
    state: stateSummary(status),
    risk: riskFact(def.riskLevel),
    permission: permissionFact(def.requiredLevel),
    restart: def.requiresReboot
      ? { icon: "mdi:restart", label: "Restart", tone: "info", tooltip: "Restart required after applying or restoring" }
      : null,
    irreversible: def.reversible ? null : { icon: "mdi:undo-variant", label: "Not reversible", tone: "warning" },
    availability:
      availability.state !== "available" && status.state !== "unavailable"
        ? {
            icon: "mdi:shield-lock-outline",
            label: availabilityLabel(availability),
            tone: "warning",
            tooltip: availability.reason,
          }
        : null,
    residue: status.residues.length
      ? { icon: "mdi:information-outline", label: "Residue", tone: "info", tooltip: residueText(status) }
      : null,
    shared: status.heldShared.length
      ? { icon: "mdi:link-variant", label: "Shared", tone: "neutral", tooltip: heldSharedText(status, find) }
      : null,
    snapshot: status.hasHistory
      ? { icon: "mdi:history", label: "Snapshot saved", tone: "neutral" }
      : { icon: "mdi:history", label: "No snapshot", tone: "subtle" },
  };
}

/** Whether a tweak's Restore can run, and how it reads. */
export interface RestoreState {
  label: string;
  disabled: boolean;
  tip: string;
}

export function restoreState(def: TweakDefinition, status: TweakStatus, isRunning: boolean): RestoreState {
  const retry = status.attention?.reason === "restore_failed";
  const label = retry ? "Retry restore" : "Restore";
  if (def.availability.state !== "available") return { label, disabled: true, tip: def.availability.reason };
  return {
    label,
    disabled: isRunning,
    tip: retry ? "Retry restoring the saved state" : "Restore the state saved before the last change",
  };
}

function availabilityLabel(availability: Exclude<Availability, { state: "available" }>): string {
  switch (availability.state) {
    case "sid_mismatch":
      return "Different account";
    case "sid_unknown":
      return "Account unconfirmed";
    case "elevation_path_unavailable":
      return "Not available on this PC";
    case "needs_elevation":
    default:
      return "Needs admin";
  }
}

export function availabilityTitle(availability: Availability): string {
  switch (availability.state) {
    case "sid_mismatch":
      return "Over-the-shoulder guard";
    case "sid_unknown":
      return "Session owner unconfirmed";
    case "elevation_path_unavailable":
      return "Not available on this PC";
    case "available":
    case "needs_elevation":
    default:
      return "Administrator required";
  }
}
