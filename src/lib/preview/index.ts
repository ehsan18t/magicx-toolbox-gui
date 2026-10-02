import { pendingRebootStore } from "$lib/stores/tweaksPending.svelte";
import type { LogLine } from "$lib/types";
import type {
  AppStatusView,
  AppView,
  Availability,
  EffectAgreement,
  EntrySummary,
  ObservedState,
  TweakEffectOption,
  TweakStatusEvent,
  TweakStatusView,
  TweakView,
} from "$lib/types";
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { apps, categories, type CorpusTweak, tweaks } from "./corpus";
import {
  adminAvailability,
  APP_VERSION,
  appAdminAvailability,
  appPresence,
  logSettings,
  NEVER_LOADS,
  PENDING_REBOOT,
  scenarios,
  seedLogLines,
  systemInfo,
} from "./fixtures";

type Args = Record<string, unknown>;

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const fail = (code: string, message: string) => Promise.reject({ code, message });

let stamp = 0;
const statuses = new Map<string, TweakStatusView>();
const entries = new Map<string, EntrySummary[]>();
const appStatuses = new Map<string, AppStatusView>();
const logLines: LogLine[] = seedLogLines();

function log(level: LogLine["level"], source: LogLine["source"], msg: string): void {
  const seq = (logLines.at(-1)?.seq ?? 0) + 1;
  logLines.push({ seq, ts: new Date().toISOString(), level, source, target: "preview", msg });
}

function tweakById(id: unknown): CorpusTweak {
  const t = tweaks.find((x) => x.id === id);
  if (!t) throw { code: "TWEAK_NOT_FOUND", message: `Unknown tweak '${String(id)}'` };
  return t;
}

function tweakAvailability(t: CorpusTweak, admin: boolean): Availability {
  if (admin) return adminAvailability[t.id] ?? { state: "available" };
  if (t.required_level === "User") return { state: "available" };
  return {
    state: "needs_elevation",
    reason:
      t.required_level === "Ti"
        ? "Part of this tweak runs as TrustedInstaller. Restart the app as administrator to enable it."
        : "Restart the app as administrator to enable this tweak.",
  };
}

function toView(t: CorpusTweak, admin: boolean): TweakView {
  const { surface: _surface, ...view } = t;
  const supported = scenarios[t.id]?.state?.state !== "unavailable";
  return { ...view, availability: tweakAvailability(t, admin), supported };
}

function appView(a: (typeof apps)[number], admin: boolean): AppView {
  const elevate: Availability = {
    state: "needs_elevation",
    reason: "Restart the app as administrator to remove apps.",
  };
  const override = admin ? appAdminAvailability[a.id] : undefined;
  return {
    ...a,
    remove_availability: override ?? (admin ? { state: "available" } : elevate),
    install_availability: override ?? { state: "available" },
    supported: true,
  };
}

/** The surface as Windows ships it: values absent, services manual, tasks enabled. */
function observedFor(t: CorpusTweak): ObservedState {
  const base = t.options[0];
  const changes: TweakEffectOption = {
    ...base,
    label: "Current",
    registry_changes: base.registry_changes.map((c) => ({
      ...c,
      action: "delete_value",
      value_type: null,
      value: null,
    })),
    service_changes: base.service_changes.map((c) => ({ ...c, startup: "manual" })),
    scheduler_changes: base.scheduler_changes.map((c) => ({ ...c, action: "enable" })),
    commands: [],
  };
  const effectFor = (name: string) => t.surface.find((e) => e.name === name)?.id ?? name;
  // A task's surface name is the leaf of its path, as the backend's agreement names it.
  const leaf = (path: string) => path.slice(path.lastIndexOf("\\") + 1);
  const agreement: EffectAgreement[] = [
    ...changes.registry_changes.map((c) => ({
      effect: effectFor(c.value_name),
      name: c.value_name,
      wanted_by: t.options
        .filter((o) => o.registry_changes.some((r) => r.value_name === c.value_name && r.action === "delete_value"))
        .map((o) => o.label),
    })),
    ...changes.service_changes.map((c) => ({
      effect: effectFor(c.name),
      name: c.name,
      wanted_by: t.options
        .filter((o) => o.service_changes.some((s) => s.name === c.name && s.startup === "manual"))
        .map((o) => o.label),
    })),
    ...changes.scheduler_changes.map((c) => ({
      effect: effectFor(leaf(c.task_path)),
      name: leaf(c.task_path),
      wanted_by: t.options
        .filter((o) => o.scheduler_changes.some((s) => s.task_path === c.task_path && s.action === "enable"))
        .map((o) => o.label),
    })),
  ];
  return { changes, agreement };
}

function seedStatuses(): void {
  tweaks.forEach((t, i) => {
    const s = scenarios[t.id];
    const state =
      s?.state ??
      (i % 3 === 0
        ? { state: "system_default" as const }
        : { state: "active" as const, option: t.options[i % t.options.length].label });
    const hasHistory = s?.has_history ?? i % 3 === 1;
    statuses.set(t.id, {
      state,
      unavailable: s?.unavailable ?? [],
      residues: s?.residues ?? [],
      has_history: hasHistory,
      attention: s?.attention ?? null,
      stamp: 0,
      held_shared: s?.held_shared ?? [],
      observed: state.state === "system_default" ? observedFor(t) : null,
    });
    if (hasHistory)
      entries.set(
        t.id,
        s?.entries ?? [
          { seq: 1, validity: "Valid", timestamp: new Date(Date.now() - i * 3_600_000).toISOString(), captured: null },
        ],
      );
  });
}

function seedApps(admin: boolean): void {
  for (const [id, p] of Object.entries(appPresence(admin))) appStatuses.set(id, { app_id: id, ...p, stamp: ++stamp });
}

function stamped(id: string): TweakStatusView {
  const view = { ...statuses.get(id)!, stamp: ++stamp };
  statuses.set(id, view);
  return view;
}

function update(id: string, patch: Partial<TweakStatusView>): TweakStatusView {
  const view = { ...statuses.get(id)!, ...patch, stamp: ++stamp };
  statuses.set(id, view);
  void emit("tweak-status", { tweak_id: id, status: view } satisfies TweakStatusEvent);
  return view;
}

function streamStatuses(): void {
  tweaks.forEach((t, i) => {
    if (t.id === NEVER_LOADS) return;
    setTimeout(
      () => void emit("tweak-status", { tweak_id: t.id, status: stamped(t.id) } satisfies TweakStatusEvent),
      150 + i * 30,
    );
  });
}

async function apply(id: string, label: string) {
  const t = tweakById(id);
  if (!t.options.some((o) => o.label === label)) return fail("TWEAK_NOT_FOUND", `'${id}' has no option '${label}'`);
  await wait(700);
  if (id === "enable_credential_guard") {
    update(id, {
      attention: {
        reason: "apply_failed",
        items: [
          {
            effect: "lsa_cfg_flags",
            kind: "verify",
            class: "failed",
            message: "LsaCfgFlags read back 0 after writing 1",
          },
        ],
      },
    });
    return fail("TWEAK_VERIFY_MISMATCH", "Apply could not be verified: LsaCfgFlags read back 0 after writing 1");
  }
  const list = entries.get(id) ?? [];
  entries.set(id, [
    { seq: (list[0]?.seq ?? 0) + 1, validity: "Valid", timestamp: new Date().toISOString(), captured: null },
    ...list,
  ]);
  log("info", "app", `apply_tweak: ${id} -> ${label}`);
  const status = update(id, {
    state: { state: "active", option: label },
    has_history: true,
    attention: null,
    observed: null,
    unavailable: [],
  });
  return { effects: [], status };
}

async function restore(id: string) {
  const t = tweakById(id);
  const [head, ...rest] = entries.get(id) ?? [];
  if (!head) return fail("TWEAK_NOT_FOUND", "There is no snapshot to restore for this tweak.");
  await wait(700);
  entries.set(id, rest);
  log("info", "app", `restore_tweak: ${id} consumed entry ${head.seq}`);
  const status = update(id, {
    state: { state: "system_default" },
    has_history: rest.length > 0,
    attention: null,
    residues: [],
    observed: observedFor(t),
  });
  return { status, consumed: head.seq, reboot_advisory: t.requires_reboot, skipped_invalid: [] };
}

async function appOp(id: string, removing: boolean): Promise<AppStatusView> {
  // Installs run long enough to show the activity bar and its timer.
  await wait(removing ? 1_200 : 4_000);
  const prev = appStatuses.get(id);
  if (!prev) return fail("APP_NOT_FOUND", `Unknown app '${id}'`);
  const next: AppStatusView = {
    ...prev,
    presence: removing ? { state: "absent" } : { state: "installed", provisioned_only: false },
    stamp: ++stamp,
  };
  appStatuses.set(id, next);
  return next;
}

const APP_INFO: Record<string, string> = {
  version: APP_VERSION,
  name: "MagicX Toolbox",
  tauri_version: "2.12.1",
  identifier: "com.magicx.toolbox",
};

function plugin(cmd: string, args: Args): unknown {
  const [name, op] = cmd.slice("plugin:".length).split("|");
  switch (name) {
    case "app":
      return APP_INFO[op] ?? null;
    case "window":
      if (op.startsWith("is_")) return op === "is_visible" || op === "is_enabled";
      return op === "scale_factor" ? window.devicePixelRatio : null;
    case "path":
      if (op === "join") return (args.paths as string[]).join("\\");
      return op === "resolve_directory" ? "C:\\Users\\PreviewUser\\AppData\\Roaming\\com.magicx.toolbox" : null;
    case "opener":
      console.info("[preview] open", args.url ?? args.path);
      return null;
    case "dialog":
    case "process":
    case "webview":
      return null;
  }
  return undefined;
}

export function installPreview(admin: boolean): void {
  seedStatuses();
  seedApps(admin);
  for (const id of PENDING_REBOOT) pendingRebootStore.add(id);
  mockWindows("main");
  mockIPC(
    (cmd, payload) => {
      const args = (payload ?? {}) as Args;
      const id = (args.tweakId ?? args.appId) as string;
      switch (cmd) {
        case "show_main_window":
        case "cancel_manual_test":
        case "install_update":
        case "reveal_last_export":
        case "open_log_folder":
          return null;
        case "restart_as_admin":
          location.search = "?preview";
          return null;
        case "get_system_info":
          return systemInfo(admin);
        case "get_elevation_state":
          return { level: admin ? "Admin" : "User", sid_mismatch: false };
        case "get_categories":
          return categories;
        case "get_tweaks":
          return tweaks.map((t) => toView(t, admin));
        case "get_statuses_stream":
        case "rescan_after_elevation":
          streamStatuses();
          return null;
        case "get_tweak_status":
          tweakById(id);
          return stamped(id);
        case "apply_tweak":
          return apply(id, args.optionLabel as string);
        case "restore_tweak":
          return restore(id);
        case "list_snapshot_entries":
          return entries.get(id) ?? [];
        case "discard_snapshot_entry": {
          const left = (entries.get(id) ?? []).filter((e) => e.seq !== args.seq);
          entries.set(id, left);
          update(id, { has_history: left.length > 0 });
          return null;
        }
        case "keep_current_state":
          entries.delete(id);
          return update(id, { attention: null, has_history: false, residues: [] });
        case "get_apps":
          return apps.map((a) => appView(a, admin));
        case "get_app_statuses":
          return wait(400).then(() => [...appStatuses.values()]);
        case "remove_app":
          return appOp(id, true);
        case "install_app":
          return appOp(id, false);
        case "get_log_tail":
          return { lines: logLines.filter((l) => l.seq > (args.since as number)), skipped: 0 };
        case "log_frontend":
          log(args.level as LogLine["level"], "ui", args.message as string);
          return null;
        case "get_log_settings":
        case "delete_logs":
          return logSettings;
        case "set_log_settings":
          return { ...logSettings, persist: args.persist, detailed: args.detailed };
        case "export_diagnostics":
          return "C:\\Users\\PreviewUser\\Desktop\\magicx-diagnostics-2026-10-02.zip";
        case "check_for_update":
          return wait(500).then(() => ({ available: false, currentVersion: APP_VERSION, prerelease: false }));
        case "manual_tests_available":
          return false;
        case "list_manual_tests":
          return [];
      }
      if (cmd.startsWith("plugin:")) {
        const result = plugin(cmd, args);
        if (result !== undefined) return result;
      }
      console.warn(`[preview] unmocked command: ${cmd}`, args);
      // A silent null would read as success, which the real backend never fakes.
      return fail("unmocked", `Preview does not mock "${cmd}"`);
    },
    { shouldMockEvents: true },
  );
}
