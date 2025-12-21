import { browser } from "$app/environment";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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
};
