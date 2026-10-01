import * as api from "$lib/api/logs";
import type { LogLine, LogSettings, LogTail } from "$lib/api/logs";
import { errorMessage } from "$lib/utils/error";
import { toastStore } from "./toast.svelte";

export const LOGS_PANEL_ID = "logs-panel";
export const LOGS_TOGGLE_ID = "logs-toggle";

const MAX_ROWS = 2000;
const POLL_MS = 500;

/** Lines that left the backend's buffer before the panel read them. */
export interface LogGap {
  gap: number;
  after: number;
}

export type LogRow = LogLine | LogGap;

export const isGap = (row: LogRow): row is LogGap => "gap" in row;

/** The session-file layout, continuation lines indented with a tab. */
export function formatLogLine(line: LogLine): string {
  return `${line.ts} ${line.level.toUpperCase().padEnd(5)} ${line.source.padEnd(6)} ${line.target}: ${line.msg.replaceAll("\n", "\n\t")}`;
}

let rows = $state.raw<LogRow[]>([]);
let since = 0;
let panelOpen = $state(false);
let generation = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

let settings = $state<LogSettings | null>(null);
let settingsBusy = $state(false);
let exporting = $state(false);

function append(requested: number, tail: LogTail) {
  // A poll that overlapped a close/reopen answers for a `since` another poll already consumed.
  if (requested !== since || tail.lines.length === 0) return;
  const added: LogRow[] = tail.skipped > 0 ? [{ gap: tail.skipped, after: requested }, ...tail.lines] : tail.lines;
  const next = rows.concat(added);
  rows = next.length > MAX_ROWS ? next.slice(-MAX_ROWS) : next;
  since = tail.lines[tail.lines.length - 1].seq;
}

async function poll(gen: number) {
  try {
    const requested = since;
    append(requested, await api.getLogTail(requested));
  } catch {
    // Retried on the next tick.
  }
  if (gen === generation && panelOpen) timer = setTimeout(() => void poll(gen), POLL_MS);
}

async function loadSettings() {
  try {
    settings = await api.getLogSettings();
  } catch (e) {
    toastStore.error(`Could not read the log settings: ${errorMessage(e)}`);
  }
}

export const logsStore = {
  get rows() {
    return rows;
  },
  get isPanelOpen() {
    return panelOpen;
  },
  get settings() {
    return settings;
  },
  get settingsBusy() {
    return settingsBusy;
  },
  get exporting() {
    return exporting;
  },

  openPanel() {
    if (panelOpen) return;
    panelOpen = true;
    void poll(++generation);
    if (!settings) void loadSettings();
  },

  closePanel() {
    panelOpen = false;
    generation++;
    clearTimeout(timer);
  },

  togglePanel() {
    if (panelOpen) this.closePanel();
    else this.openPanel();
  },

  /** Empties the view only; lines already read are not fetched again. */
  clearView() {
    rows = [];
  },

  loadSettings,

  async setSettings(persist: boolean, detailed: boolean) {
    if (settingsBusy) return;
    settingsBusy = true;
    try {
      settings = await api.setLogSettings(persist, detailed);
    } catch (e) {
      toastStore.error(`Could not change the log settings: ${errorMessage(e)}`);
      await loadSettings();
    } finally {
      settingsBusy = false;
    }
  },

  async deleteLogs() {
    if (settingsBusy) return;
    settingsBusy = true;
    try {
      settings = await api.deleteLogs();
      toastStore.success("Logs deleted");
    } catch (e) {
      toastStore.error(`Some log files could not be deleted: ${errorMessage(e)}`);
      await loadSettings();
    } finally {
      settingsBusy = false;
    }
  },

  async exportDiagnostics() {
    if (exporting) return;
    exporting = true;
    try {
      const path = await api.exportDiagnostics();
      if (path === null) return;
      toastStore.success("Diagnostics exported. Check the file before sharing.", {
        duration: 10000,
        action: {
          label: "Show in folder",
          run: () => void api.revealLastExport().catch((e) => toastStore.error(errorMessage(e))),
        },
      });
    } catch (e) {
      toastStore.error(`Could not export diagnostics: ${errorMessage(e)}`);
    } finally {
      exporting = false;
    }
  },

  async openFolder() {
    try {
      await api.openLogFolder();
    } catch (e) {
      toastStore.error(`Could not open the logs folder: ${errorMessage(e)}`);
    }
  },
};
