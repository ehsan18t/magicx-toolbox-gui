/**
 * SYSTEM Elevation Store
 *
 * Tracks whether SYSTEM elevation mode is enabled for modifying protected registry keys.
 */

import { invoke } from "@tauri-apps/api/core";
import { get, writable } from "svelte/store";

interface SystemElevationState {
  /** Whether SYSTEM elevation is available (running as admin) */
  available: boolean;
  /** Whether SYSTEM elevation mode is currently enabled */
  enabled: boolean;
  /** Whether we're currently checking availability */
  checking: boolean;
}

const initialState: SystemElevationState = {
  available: false,
  enabled: false,
  checking: true,
};

function createSystemElevationStore() {
  const { subscribe, update } = writable<SystemElevationState>(initialState);

  return {
    subscribe,

    /** Initialize the store - check if elevation is available */
    async init() {
      update((s) => ({ ...s, checking: true }));
      try {
        const available = await invoke<boolean>("can_use_system_elevation");
        update((s) => ({ ...s, available, checking: false }));
      } catch (error) {
        console.error("Failed to check SYSTEM elevation availability:", error);
        update((s) => ({ ...s, available: false, checking: false }));
      }
    },

    /** Toggle SYSTEM elevation mode on/off */
    toggle() {
      update((s) => {
        if (!s.available) return s;
        const newEnabled = !s.enabled;
        console.log(`SYSTEM elevation mode: ${newEnabled ? "ENABLED" : "DISABLED"}`);
        return { ...s, enabled: newEnabled };
      });
    },

    /** Enable SYSTEM elevation mode */
    enable() {
      update((s) => {
        if (!s.available) return s;
        console.log("SYSTEM elevation mode: ENABLED");
        return { ...s, enabled: true };
      });
    },

    /** Disable SYSTEM elevation mode */
    disable() {
      update((s) => {
        console.log("SYSTEM elevation mode: DISABLED");
        return { ...s, enabled: false };
      });
    },

    /** Check if currently enabled */
    isEnabled(): boolean {
      return get({ subscribe }).enabled;
    },

    /** Check if available */
    isAvailable(): boolean {
      return get({ subscribe }).available;
    },
  };
}

export const systemElevation = createSystemElevationStore();
