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

export type Tone = "accent" | "success" | "warning" | "error" | "info" | "neutral";

export const TONE_TEXT: Record<Tone, string> = {
  accent: "text-accent",
  success: "text-success",
  warning: "text-warning",
  error: "text-error",
  info: "text-info",
  neutral: "text-foreground-muted",
};

export const CHECKING = { label: "Checking", icon: "mdi:loading" } as const;
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

export interface PermissionInfo {
  name: string;
  description: string;
  icon: string;
}

const PERMISSION_INFO: Record<Exclude<Level, "User">, PermissionInfo> = {
  Admin: {
    name: "Admin",
    description: "Requires Administrator privileges to apply",
    icon: "mdi:shield-account-outline",
  },
  Ti: {
    name: "TrustedInstaller",
    description: "Requires TrustedInstaller elevation for highly protected resources",
    icon: "mdi:shield-key",
  },
};

export function permissionInfoFor(level: Level): PermissionInfo | null {
  return level === "User" ? null : PERMISSION_INFO[level];
}

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

export interface StateSummary {
  label: string;
  tone: Tone;
  icon: string;
  spin?: boolean;
}

// Each switch keeps a default: a state added in Rust must not crash the rows.
export function stateSummary(status: TweakStatus): StateSummary {
  switch (status.state) {
    case "active":
      return { label: status.activeOption ?? "Active", tone: "accent", icon: "mdi:check-circle" };
    case "system_default":
      return { label: "System default", tone: "neutral", icon: "mdi:monitor" };
    case "unavailable":
      return { label: "Unavailable", tone: "neutral", icon: "mdi:cancel" };
    case "unknown":
      return { label: status.needsElevation ? UNKNOWN_NEEDS_ADMIN : "Unknown", tone: "warning", icon: UNKNOWN_ICON };
    case "loading":
    default:
      return { label: CHECKING.label, tone: "neutral", icon: CHECKING.icon };
  }
}

/** Whether a row's Restore can run, and what its tooltip says. */
export interface RestoreState {
  disabled: boolean;
  tip: string;
}

export function restoreState(def: TweakDefinition, status: TweakStatus, isRunning: boolean): RestoreState {
  if (def.availability.state !== "available") return { disabled: true, tip: def.availability.reason };
  return {
    disabled: isRunning,
    tip:
      status.attention?.reason === "restore_failed"
        ? "Retry restoring the saved state"
        : "Restore the state saved before the last change",
  };
}

export function availabilityLabel(availability: Availability): string {
  switch (availability.state) {
    case "available":
      return "";
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
