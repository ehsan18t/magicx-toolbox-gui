/**
 * SYSTEM Elevation Store - Svelte 5 Runes
 *
 * Tracks whether SYSTEM elevation mode is enabled for modifying protected registry keys.
 */

import { invoke } from "@tauri-apps/api/core";

// === State ===

/** Whether SYSTEM elevation is available (running as admin) */
let available = $state(false);

/** Whether SYSTEM elevation mode is currently enabled */
let enabled = $state(false);

/** Whether we're currently checking availability */
let checking = $state(true);

// === Export ===

export const systemElevationStore = {
  /** Whether SYSTEM elevation is available */
  get available() {
    return available;
  },

  /** Whether SYSTEM elevation is enabled */
  get enabled() {
    return enabled;
  },

  /** Whether we're checking availability */
  get checking() {
    return checking;
  },

  /** Initialize the store - check if elevation is available */
  async init() {
    checking = true;
    try {
      const result = await invoke<boolean>("can_use_system_elevation");
      available = result;
    } catch (error) {
      console.error("Failed to check SYSTEM elevation availability:", error);
      available = false;
    } finally {
      checking = false;
    }
  },

  /** Toggle SYSTEM elevation mode on/off */
  toggle() {
    if (!available) return;
    enabled = !enabled;
  },

  /** Enable SYSTEM elevation mode */
  enable() {
    if (!available) return;
    enabled = true;
  },

  /** Disable SYSTEM elevation mode */
  disable() {
    enabled = false;
  },

  /** Check if currently enabled */
  isEnabled(): boolean {
    return enabled;
  },

  /** Check if available */
  isAvailable(): boolean {
    return available;
  },
};
