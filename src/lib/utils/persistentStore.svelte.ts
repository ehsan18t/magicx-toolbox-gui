import { browser } from "$app/environment";
import { logError, logWarning } from "$lib/utils/logger";

/** A rune-backed value mirrored to localStorage as JSON. */
export class PersistentStore<T> {
  #value = $state() as T;
  #key: string;
  /** Something was stored under the key at construction, even if it did not parse. */
  readonly hadStoredValue: boolean = false;

  constructor(key: string, initialValue: T) {
    this.#key = key;
    this.#value = initialValue;
    if (!browser) return;

    try {
      const stored = localStorage.getItem(key);
      if (stored === null) return;
      this.hadStoredValue = true;
      try {
        this.#value = JSON.parse(stored);
      } catch {
        logWarning(`Invalid JSON in ${key}, resetting to default.`);
        try {
          localStorage.setItem(key, JSON.stringify(initialValue));
        } catch (writeErr) {
          logError(`Failed to reset ${key}`, writeErr);
        }
      }
    } catch (error) {
      logError(`Error loading ${key} from localStorage`, error);
    }
  }

  get value() {
    return this.#value;
  }

  set value(newValue: T) {
    this.#value = newValue;
    if (!browser) return;
    try {
      localStorage.setItem(this.#key, JSON.stringify(newValue));
    } catch (error) {
      logError(`Error saving ${this.#key} to localStorage`, error);
    }
  }
}
