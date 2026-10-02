import type { AppActionKind, AppOperationKind, AppStatusView, AppView, Availability } from "$lib/types";
import { CHECKING, ELEVATE_HINT, type StateSummary, UNKNOWN_ICON, UNKNOWN_NEEDS_ADMIN } from "./tweakPresentation";

export interface AppAction {
  kind: AppActionKind;
  disabledReason: string | null;
}

export const APP_OPERATION_LABEL: Record<AppOperationKind, string> = { remove: "Removing", install: "Installing" };

/** The presence chip on an app row. */
export function appPresenceChip(
  status: AppStatusView | undefined,
  scanError: string | null,
): StateSummary & { tip: string } {
  const presence = status?.presence;
  if (!presence) {
    return scanError
      ? { label: "Unknown", tip: scanError, icon: UNKNOWN_ICON, tone: "warning" }
      : { ...CHECKING, tip: "Checking whether this app is installed…", tone: "neutral", spin: true };
  }
  switch (presence.state) {
    case "installed":
      return presence.provisioned_only
        ? {
            label: "Provisioned only",
            tip: "Not installed for any account yet, but Windows installs it for every new account.",
            icon: "mdi:package-variant",
            tone: "info",
          }
        : { label: "Installed", tip: "Installed on this PC", icon: "mdi:check-circle", tone: "success" };
    case "absent":
      return { label: "Not installed", tip: "Not installed on this PC", icon: "mdi:circle-outline", tone: "neutral" };
    case "unknown":
    default:
      return {
        label: presence.needs_elevation ? UNKNOWN_NEEDS_ADMIN : "Unknown",
        tip: presence.needs_elevation ? `${presence.reason} ${ELEVATE_HINT}` : presence.reason,
        icon: UNKNOWN_ICON,
        tone: "warning",
      };
  }
}

const reasonIfBlocked = (a: Availability) => (a.state === "available" ? null : a.reason);

/** One action per row; null only for an absent app with no install route, which is hidden anyway. */
export function appAction(app: AppView, status: AppStatusView | undefined, scanError: string | null): AppAction | null {
  const presence = status?.presence;
  if (!presence) return { kind: "remove", disabledReason: appPresenceChip(status, scanError).tip };
  if (presence.state === "unknown") return { kind: "remove", disabledReason: presence.reason };
  if (presence.state === "installed") {
    return { kind: "remove", disabledReason: reasonIfBlocked(app.remove_availability) };
  }
  if (status.install_route === "winget") {
    return { kind: "install", disabledReason: reasonIfBlocked(app.install_availability) };
  }
  if (status.install_route === "store_page") return { kind: "store", disabledReason: null };
  return null;
}
