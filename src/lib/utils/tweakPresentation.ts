import type { Availability, RiskLevel, TweakStatus, TweakWithStatus } from "$lib/types";

export type Tone = "accent" | "success" | "warning" | "error" | "info" | "neutral";

export const TONE_TEXT: Record<Tone, string> = {
  accent: "text-accent",
  success: "text-success",
  warning: "text-warning",
  error: "text-error",
  info: "text-info",
  neutral: "text-foreground-muted",
};

export const TONE_SOFT: Record<Tone, string> = {
  accent: "bg-accent/12 text-accent",
  success: "bg-success/12 text-success",
  warning: "bg-warning/12 text-warning",
  error: "bg-error/12 text-error",
  info: "bg-info/12 text-info",
  neutral: "bg-muted text-foreground-muted",
};

export const RISK_TONE: Record<RiskLevel, Tone> = {
  low: "success",
  medium: "warning",
  high: "error",
  critical: "error",
};

/** A bulk restore takes only these: each row's own Restore is disabled otherwise. */
export const canRestore = (t: TweakWithStatus) =>
  t.status.has_backup && t.definition.availability.state === "available";

export const restoreMessage = (count: number) =>
  `Restore ${count === 1 ? "1 tweak" : `${count} tweaks`} to the state saved before ${count === 1 ? "its" : "each one's"} last change?`;

export const isHighRisk = (level: RiskLevel) => level === "high" || level === "critical";

export interface StateSummary {
  label: string;
  tone: Tone;
  icon: string;
}

export function stateSummary(status: TweakStatus): StateSummary {
  switch (status.state) {
    case "active":
      return { label: status.activeOption ?? "Active", tone: "accent", icon: "mdi:check-circle" };
    case "system_default":
      return { label: "System default", tone: "neutral", icon: "mdi:monitor" };
    case "unavailable":
      return { label: "Unavailable", tone: "neutral", icon: "mdi:cancel" };
    case "unknown":
      return {
        label: status.needsElevation ? "Unknown, needs admin" : "Unknown",
        tone: "warning",
        icon: "mdi:help-circle-outline",
      };
    default:
      return { label: "Checking", tone: "neutral", icon: "mdi:loading" };
  }
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
    default:
      return "Administrator required";
  }
}

export function matchesQuery(tweak: TweakWithStatus, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return tweak.definition.name.toLowerCase().includes(q) || tweak.definition.description.toLowerCase().includes(q);
}
