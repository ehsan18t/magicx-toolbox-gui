import * as logsApi from "$lib/api/logs";
import type { LogLine, LogSettings, LogTail } from "$lib/types";
import { errorMessage } from "$lib/utils/error";
import { TOAST_DURATION, toastStore } from "./toast.svelte";

export const LOGS_PANEL_ID = "logs-panel";
export const LOGS_TOGGLE_ID = "logs-toggle";

const MAX_ROWS = 2000;
const POLL_MS = 500;
// `ts` is RFC 3339: these bound HH:MM:SS.mmm.
const TIME_START = 11;
const TIME_END = 23;
const CRATE_PREFIX = /^app_lib::/;

/** Lines that left the backend's buffer before the panel read them. */
export interface LogGap {
  gap: number;
  after: number;
}

/** A line with its display and search text derived once, on arrival. */
export interface LogEntry extends LogLine {
  time: string;
  module: string;
  msgLower: string;
  targetLower: string;
}

export type LogRow = LogEntry | LogGap;

export const isGap = (row: LogRow): row is LogGap => "gap" in row;

export const gapLabel = (row: LogGap) => `${row.gap} lines skipped`;

/** The session-file layout, continuation lines indented with a tab. */
export function formatLogLine(line: LogLine): string {
  return `${line.ts} ${line.level.toUpperCase().padEnd(5)} ${line.source.padEnd(6)} ${line.target}: ${line.msg.replaceAll("\n", "\n\t")}`;
}

let rows = $state.raw<LogRow[]>([]);
let since = 0;
let isPanelOpen = $state(false);
let generation = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

let settings = $state<LogSettings | null>(null);
let isSettingsBusy = $state(false);
let isExporting = $state(false);

function append(requested: number, tail: LogTail) {
  // A poll that overlapped a close/reopen answers for a `since` another poll already consumed.
  if (requested !== since || tail.lines.length === 0) return;
  const lines: LogEntry[] = tail.lines.map((line) => ({
    ...line,
    time: line.ts.slice(TIME_START, TIME_END),
    module: line.target.replace(CRATE_PREFIX, ""),
    msgLower: line.msg.toLowerCase(),
    targetLower: line.target.toLowerCase(),
  }));
  const added: LogRow[] = tail.skipped > 0 ? [{ gap: tail.skipped, after: requested }, ...lines] : lines;
  const next = rows.concat(added);
  rows = next.length > MAX_ROWS ? next.slice(-MAX_ROWS) : next;
  since = tail.lines[tail.lines.length - 1].seq;
}

async function poll(gen: number) {
  try {
    const requested = since;
    append(requested, await logsApi.getLogTail(requested));
  } catch {
    // Retried on the next tick.
  }
  if (gen === generation && isPanelOpen) timer = setTimeout(() => void poll(gen), POLL_MS);
}

async function loadSettings() {
  try {
    settings = await logsApi.getLogSettings();
  } catch (error) {
    toastStore.error(`Could not read the log settings: ${errorMessage(error)}`);
  }
}

function openPanel() {
  if (isPanelOpen) return;
  isPanelOpen = true;
  void poll(++generation);
  if (!settings) void loadSettings();
}

function closePanel() {
  isPanelOpen = false;
  generation++;
  clearTimeout(timer);
}

export const logsStore = {
  get rows() {
    return rows;
  },
  get isPanelOpen() {
    return isPanelOpen;
  },
  get settings() {
    return settings;
  },
  get isSettingsBusy() {
    return isSettingsBusy;
  },
  get isExporting() {
    return isExporting;
  },

  closePanel,

  togglePanel() {
    if (isPanelOpen) closePanel();
    else openPanel();
  },

  /** Empties the view only; lines already read are not fetched again. */
  clearView() {
    rows = [];
  },

  loadSettings,

  async setSettings(persist: boolean, detailed: boolean) {
    if (isSettingsBusy) return;
    isSettingsBusy = true;
    try {
      settings = await logsApi.setLogSettings(persist, detailed);
    } catch (error) {
      toastStore.error(`Could not change the log settings: ${errorMessage(error)}`);
      await loadSettings();
    } finally {
      isSettingsBusy = false;
    }
  },

  async deleteLogs() {
    if (isSettingsBusy) return;
    isSettingsBusy = true;
    try {
      settings = await logsApi.deleteLogs();
      toastStore.success("Logs deleted");
    } catch (error) {
      toastStore.error(`Some log files could not be deleted: ${errorMessage(error)}`);
      await loadSettings();
    } finally {
      isSettingsBusy = false;
    }
  },

  async exportDiagnostics() {
    if (isExporting) return;
    isExporting = true;
    try {
      const path = await logsApi.exportDiagnostics();
      if (path === null) return;
      toastStore.success("Diagnostics exported. Check the file before sharing.", {
        duration: TOAST_DURATION.long,
        action: {
          label: "Show in folder",
          run: () => void logsApi.revealLastExport().catch((error) => toastStore.error(errorMessage(error))),
        },
      });
    } catch (error) {
      toastStore.error(`Could not export diagnostics: ${errorMessage(error)}`);
    } finally {
      isExporting = false;
    }
  },

  async openFolder() {
    try {
      await logsApi.openLogFolder();
    } catch (error) {
      toastStore.error(`Could not open the logs folder: ${errorMessage(error)}`);
    }
  },
};
