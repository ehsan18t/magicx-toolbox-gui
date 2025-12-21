import { browser } from "$app/environment";

/**
 * A persistent store implementation using Svelte 5 runes.
 * Automatically syncs with localStorage throughout the application lifecycle.
 */
export class PersistentStore<T> {
  #value: T = $state(undefined as unknown as T);
  #key: string;

  constructor(key: string, initialValue: T) {
    this.#key = key;
    // Set initial value first
    this.#value = initialValue;

    if (browser) {
      try {
        const stored = localStorage.getItem(key);
        if (stored !== null) {
          try {
            this.#value = JSON.parse(stored);
          } catch {
            console.warn(`Invalid JSON in ${key}, resetting to default.`);
            // Self-heal: Overwrite corrupted data with initial value immediately
            try {
              localStorage.setItem(key, JSON.stringify(initialValue));
            } catch (writeErr) {
              console.error(`Failed to reset ${key}:`, writeErr);
            }
          }
        }
      } catch (error) {
        console.error(`Error loading ${key} from localStorage:`, error);
      }
    }
  }

  get value() {
    return this.#value;
  }

  set value(newValue: T) {
    this.#value = newValue;
    if (browser) {
      try {
        localStorage.setItem(this.#key, JSON.stringify(newValue));
      } catch (error) {
        console.error(`Error saving ${this.#key} to localStorage:`, error);
      }
    }
  }

  /**
   * Resets the store to the provided default value (or the one passed in constructor if I stored it)
   * For now, just a helper to set value
   */
  set(newValue: T) {
    this.value = newValue;
  }

  /**
   * Manually reload from local storage (useful if modified externally)
   */
  load() {
    if (browser) {
      try {
        const stored = localStorage.getItem(this.#key);
        if (stored !== null) {
          this.#value = JSON.parse(stored);
        }
      } catch (error) {
        console.error(`Error reloading ${this.#key}:`, error);
      }
    }
  }
}
