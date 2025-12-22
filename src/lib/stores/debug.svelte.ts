import { browser } from "$app/environment";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

export interface DebugLogEntry {
  id: number;
  timestamp: Date;
  level: "info" | "warn" | "error" | "success";
  source: "frontend" | "backend";
  action: string;
  details: string;
  data?: unknown;
}

// Backend event payload type
interface BackendDebugLog {
  timestamp: string;
  level: "info" | "warn" | "error" | "success";
  message: string;
  context?: string;
}

const DEBUG_STORAGE_KEY = "magicx-debug-mode";
let logIdCounter = 0;
let unlistenDebugLog: UnlistenFn | null = null;

// Persistent state for enabled flag
const debugEnabledState = new PersistentStore(DEBUG_STORAGE_KEY, false);

// Other reactive state
let logs = $state<DebugLogEntry[]>([]);
let isPanelOpen = $state(false);

// Derived values using $derived
const logCounts = $derived({
  total: logs.length,
  info: logs.filter((l) => l.level === "info").length,
  warn: logs.filter((l) => l.level === "warn").length,
  error: logs.filter((l) => l.level === "error").length,
  success: logs.filter((l) => l.level === "success").length,
});

// Sync debug mode to Rust backend
async function syncDebugModeToBackend(value: boolean) {
  if (!browser) return;
  try {
    await invoke("set_debug_mode", { enabled: value });
  } catch (error) {
    console.warn("Failed to sync debug mode to backend:", error);
  }
}

// Set up listener for backend debug events
async function setupBackendListener() {
  if (unlistenDebugLog || !browser) return;

  try {
    unlistenDebugLog = await listen<BackendDebugLog>("debug-log", (event) => {
      const payload = event.payload;
      const entry: DebugLogEntry = {
        id: ++logIdCounter,
        timestamp: new Date(),
        level: payload.level,
        source: "backend",
        action: payload.context ?? "Registry",
        details: payload.message,
      };

      // Prepend new log, keep max 500
      logs = [entry, ...logs].slice(0, 500);
    });
  } catch (error) {
    console.warn("Failed to set up debug listener:", error);
  }
}

// Clean up listener
function cleanupBackendListener() {
  if (unlistenDebugLog) {
    unlistenDebugLog();
    unlistenDebugLog = null;
  }
}

// Initialize based on persistent state
if (browser && debugEnabledState.value) {
  syncDebugModeToBackend(true).then(() => setupBackendListener());
}

// Export reactive getters and methods
export const debugState = {
  get enabled() {
    return debugEnabledState.value;
  },
  get logs() {
    return logs;
  },
  get isPanelOpen() {
    return isPanelOpen;
  },
  get logCounts() {
    return logCounts;
  },

  toggle() {
    const newValue = !debugEnabledState.value;
    debugEnabledState.value = newValue;

    syncDebugModeToBackend(newValue);
    if (newValue) {
      setupBackendListener();
    } else {
      cleanupBackendListener();
    }
  },

  setEnabled(value: boolean) {
    debugEnabledState.value = value;

    syncDebugModeToBackend(value);
    if (value) {
      setupBackendListener();
    } else {
      cleanupBackendListener();
    }
  },

  togglePanel() {
    isPanelOpen = !isPanelOpen;
  },

  openPanel() {
    isPanelOpen = true;
  },

  closePanel() {
    isPanelOpen = false;
  },

  log(level: DebugLogEntry["level"], source: DebugLogEntry["source"], action: string, details: string, data?: unknown) {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const timestamp = new Date();
    const entry: DebugLogEntry = {
      id: ++logIdCounter,
      timestamp,
      level,
      source,
      action,
      details,
      data,
    };

    // Prepend new log, keep max 500
    logs = [entry, ...logs].slice(0, 500);

    // Also log to browser console when debug is enabled
    if (debugEnabledState.value) {
      const prefix = `[${source.toUpperCase()}] ${action}:`;
      switch (level) {
        case "error":
          console.error(prefix, details, data ?? "");
          break;
        case "warn":
          console.warn(prefix, details, data ?? "");
          break;
        case "success":
          console.log(`✅ ${prefix}`, details, data ?? "");
          break;
        default:
          console.log(prefix, details, data ?? "");
      }
    }
  },

  info(source: DebugLogEntry["source"], action: string, details: string, data?: unknown) {
    this.log("info", source, action, details, data);
  },

  warn(source: DebugLogEntry["source"], action: string, details: string, data?: unknown) {
    this.log("warn", source, action, details, data);
  },

  error(source: DebugLogEntry["source"], action: string, details: string, data?: unknown) {
    this.log("error", source, action, details, data);
  },

  success(source: DebugLogEntry["source"], action: string, details: string, data?: unknown) {
    this.log("success", source, action, details, data);
  },

  clear() {
    logs = [];
  },

  /**
   * Export all logs to a file using the system save dialog.
   * Logs are formatted as JSON for easy parsing and analysis.
   * @returns true if export was successful, false if cancelled or failed
   */
  async exportLogs(): Promise<boolean> {
    if (logs.length === 0) {
      return false;
    }

    try {
      // Generate default filename with timestamp
      const timestamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      const defaultPath = `magicx-debug-logs-${timestamp}.json`;

      // Open save dialog
      const filePath = await save({
        defaultPath,
        filters: [
          { name: "JSON Files", extensions: ["json"] },
          { name: "Text Files", extensions: ["txt"] },
        ],
      });

      if (!filePath) {
        // User cancelled
        return false;
      }

      // Format logs for export
      const exportData = {
        exportedAt: new Date().toISOString(),
        totalLogs: logs.length,
        logs: logs.map((log) => ({
          id: log.id,
          timestamp: log.timestamp.toISOString(),
          level: log.level,
          source: log.source,
          action: log.action,
          details: log.details,
          data: log.data,
        })),
      };

      // Write to file
      await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
      return true;
    } catch (error) {
      console.error("Failed to export debug logs:", error);
      return false;
    }
  },
};
