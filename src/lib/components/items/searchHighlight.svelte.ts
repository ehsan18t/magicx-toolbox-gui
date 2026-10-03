import { navigationStore } from "$lib/stores/navigation.svelte";
import { duration } from "$lib/utils/motion";

/** Takes a search "Go to" highlight as the row mounts; only the target row runs an effect. */
export function searchHighlight(id: () => string, el: () => HTMLElement | null) {
  const isTarget = navigationStore.takeHighlight(id());
  let active = $state(isTarget);

  if (isTarget) {
    const ms = duration("highlight");
    $effect(() => {
      // After a frame: rows above are still settling their height on first render.
      const frame = requestAnimationFrame(() => {
        const row = el();
        row?.scrollIntoView({ block: "center" });
        row?.focus({ preventScroll: true });
      });
      const timer = setTimeout(() => (active = false), ms);
      return () => {
        cancelAnimationFrame(frame);
        clearTimeout(timer);
      };
    });
  }

  return {
    get active() {
      return active;
    },
  };
}
