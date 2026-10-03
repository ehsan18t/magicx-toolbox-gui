<script lang="ts">
  import { Card, Spinner } from "$lib/components/ui";
  import { HEADING } from "$lib/design";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { delay, fade } from "$lib/utils/motion";
  import { untrack } from "svelte";
  import { BUSY_HINT } from "./busy";

  /** The app behind the overlay, inert while it shows. */
  let { shell }: { shell: HTMLElement | null } = $props();

  const isApplying = $derived(tweakActionsStore.isBusy);

  let visible = $state(false);
  let shownAt = 0;
  let seen = false;

  $effect(() => {
    if (isApplying) {
      if (!untrack(() => visible)) shownAt = performance.now();
      visible = true;
      return;
    }
    // Gone before it faded in: drop it now. Once seen, hold it so a batch does not flicker between items.
    seen = performance.now() - shownAt >= delay("reveal");
    const hideTimer = setTimeout(() => (visible = false), seen ? delay("settle") : 0);
    return () => clearTimeout(hideTimer);
  });

  // Inert drops focus to the body, so it goes back where it was if that control is still there.
  $effect(() => {
    if (!visible || !shell) return;
    const blocked = shell;
    const focused = document.activeElement;
    blocked.inert = true;
    return () => {
      blocked.inert = false;
      if (focused instanceof HTMLElement && focused.isConnected) focused.focus();
    };
  });

  // Never seen: leave at once, or the outro would overlap the inner layer's delayed fade-in.
  const exit = (node: Element) => (seen ? fade(node, { speed: "fast" }) : { duration: 0 });
</script>

{#if visible}
  <!-- Blocks input at once; the inner layer only shows if the work outlasts the reveal delay. -->
  <div class="fixed inset-x-0 top-titlebar bottom-0 z-busy" role="presentation" aria-busy="true" out:exit>
    <div class="flex h-full animate-reveal items-center justify-center bg-scrim p-4">
      <Card elevation="dialog" class="flex w-full max-w-sm items-center gap-3 px-6 py-5" role="status">
        <Spinner />
        <div class="min-w-0">
          <div class={["text-foreground", HEADING.section]}>Changing system settings…</div>
          <div class="mt-0.5 text-sm text-foreground-muted">{BUSY_HINT}</div>
        </div>
      </Card>
    </div>
  </div>
{/if}
