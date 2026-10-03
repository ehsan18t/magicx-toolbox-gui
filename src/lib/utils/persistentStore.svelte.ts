import { logError, logWarning } from "$lib/utils/logger";

/** A rune-backed value mirrored to localStorage as JSON. */
export class PersistentStore<T> {
  #value = $state() as T;
  #key: string;
  /** A valid stored value was adopted at construction. */
  readonly restored: boolean = false;

  /** `parse` vets what was stored: undefined rejects it, and the default is written back. */
  constructor(key: string, initialValue: T, parse: (stored: unknown) => T | undefined) {
    this.#key = key;
    this.#value = initialValue;

    try {
      const stored = localStorage.getItem(key);
      if (stored === null) return;
      let parsed: T | undefined;
      try {
        parsed = parse(JSON.parse(stored));
      } catch {
        parsed = undefined;
      }
      if (parsed !== undefined) {
        this.#value = parsed;
        this.restored = true;
        return;
      }
      logWarning(`Invalid value in ${key}, resetting to default.`);
      try {
        localStorage.setItem(key, JSON.stringify(initialValue));
      } catch (writeError) {
        logError(`Failed to reset ${key}`, writeError);
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
    try {
      localStorage.setItem(this.#key, JSON.stringify(newValue));
    } catch (error) {
      logError(`Error saving ${this.#key} to localStorage`, error);
    }
  }
}
