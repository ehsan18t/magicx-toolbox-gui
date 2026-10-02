import { searchStore } from "$lib/stores/search.svelte";
import { duration } from "$lib/utils/motion";

/** Consumes a search "Go to" highlight once, so it cannot replay when the row mounts again elsewhere. */
export function searchHighlight(id: () => string, el: () => HTMLElement | null) {
  let active = $state(false);
  let frame = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (searchStore.highlightTweakId !== id()) return;
    searchStore.clearHighlight();
    active = true;
    // After a frame: rows above are still settling their height on first render.
    frame = requestAnimationFrame(() => el()?.scrollIntoView({ block: "center" }));
    clearTimeout(timer);
    timer = setTimeout(() => (active = false), duration("highlight"));
  });

  $effect(() => () => {
    cancelAnimationFrame(frame);
    clearTimeout(timer);
  });

  return {
    get active() {
      return active;
    },
  };
}
