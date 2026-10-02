import { SvelteMap, SvelteSet } from "svelte/reactivity";

const runningTweaks = new SvelteSet<string>();
// Spans a whole batch: the per-tweak set is empty between items.
let batchRunning = $state(false);

const errors = new SvelteMap<string, string>();

/** Apply, restore and keep operations in flight. */
export const tweakOps = {
  isRunning(tweakId: string): boolean {
    return runningTweaks.has(tweakId);
  },

  /** True while any tweak operation or batch runs; gate every batch control on this. */
  get isBusy(): boolean {
    return batchRunning || runningTweaks.size > 0;
  },

  /** Runs `fn` as the only batch; resolves to null when another batch is already running. */
  async exclusive<T>(fn: () => Promise<T>): Promise<T | null> {
    if (batchRunning) return null;
    batchRunning = true;
    try {
      return await fn();
    } finally {
      batchRunning = false;
    }
  },

  start(tweakId: string) {
    runningTweaks.add(tweakId);
  },

  stop(tweakId: string) {
    runningTweaks.delete(tweakId);
  },
};

/** The last operation's failure per tweak. */
export const errorStore = {
  get(tweakId: string): string | undefined {
    return errors.get(tweakId);
  },

  set(tweakId: string, message: string) {
    errors.set(tweakId, message);
  },

  clear(tweakId: string) {
    errors.delete(tweakId);
  },
};
