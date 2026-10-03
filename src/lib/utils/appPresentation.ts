import type { IconName } from "$lib/design";
import type { AppActionKind, AppOperationKind, AppStatusView, AppView, Availability } from "$lib/types";
import { CHECKING, ELEVATE_HINT, type MetaFact, UNKNOWN_ICON, UNKNOWN_NEEDS_ADMIN } from "./presentation";

export interface AppAction {
  kind: AppActionKind;
  disabledReason: string | null;
}

export const APP_OPERATION_LABEL: Record<AppOperationKind, string> = { remove: "Removing", install: "Installing" };

/** `aria` prefixes the app name to form the button's accessible name. */
export const APP_ACTION_UI: Record<
  AppActionKind,
  { label: string; icon: IconName; aria: string; tone?: "accent" | "error" }
> = {
  remove: { label: "Remove", icon: "mdi:delete-outline", aria: "Remove", tone: "error" },
  install: { label: "Install", icon: "mdi:download", aria: "Install", tone: "accent" },
  store: { label: "Get in Store", icon: "mdi:open-in-new", aria: "Open the Microsoft Store page for" },
};

/** The presence chip on an app row. */
export function appPresenceChip(
  status: AppStatusView | undefined,
  scanError: string | null,
): MetaFact & { tooltip: string } {
  const presence = status?.presence;
  if (!presence) {
    return scanError
      ? { label: "Unknown", tooltip: scanError, icon: UNKNOWN_ICON, tone: "warning" }
      : { ...CHECKING, tooltip: "Checking whether this app is installed…" };
  }
  switch (presence.state) {
    case "installed":
      return presence.provisioned_only
        ? {
            label: "Provisioned only",
            tooltip: "Not installed for any account yet, but Windows installs it for every new account.",
            icon: "mdi:package-variant",
            tone: "info",
          }
        : { label: "Installed", tooltip: "Installed on this PC", icon: "mdi:check-circle", tone: "success" };
    case "absent":
      return {
        label: "Not installed",
        tooltip: "Not installed on this PC",
        icon: "mdi:circle-outline",
        tone: "neutral",
      };
    case "unknown":
    default:
      return {
        label: presence.needs_elevation ? UNKNOWN_NEEDS_ADMIN : "Unknown",
        tooltip: presence.needs_elevation ? `${presence.reason} ${ELEVATE_HINT}` : presence.reason,
        icon: UNKNOWN_ICON,
        tone: "warning",
      };
  }
}

const reasonIfBlocked = (a: Availability) => (a.state === "available" ? null : a.reason);

/** One action per row; null only for an absent app with no install route, which is hidden anyway. */
export function appAction(app: AppView, status: AppStatusView | undefined, scanError: string | null): AppAction | null {
  const presence = status?.presence;
  if (!presence) return { kind: "remove", disabledReason: appPresenceChip(status, scanError).tooltip };
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

/** Present with no install source on this PC: removing it cannot be undone from here. */
export const isPermanent = (status: AppStatusView | undefined): boolean =>
  status?.install_route === "none" && status.presence.state !== "absent";

export const PERMANENT_REMOVAL =
  "There is no install source for this app on this PC, so removing it cannot be undone from here.";

export function removeConfirmMessage(app: AppView, permanent: boolean): string {
  const scope = app.source === "appx" ? "for every account on this PC" : "for your account";
  const outcome = `${app.name} is removed ${scope}. ${permanent ? PERMANENT_REMOVAL : "You can reinstall it later from here."}`;
  return app.warning ? `${outcome} ${app.warning}` : outcome;
}
