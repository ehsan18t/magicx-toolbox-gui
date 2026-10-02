<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { tweakOps } from "$lib/stores/tweakOps.svelte";
  import { delay, fade } from "$lib/utils/motion";
  import { untrack } from "svelte";

  const isApplying = $derived(tweakOps.isBusy);

  let visible = $state(false);
  let shownAt = 0;
  let seen = false;
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    clearTimeout(hideTimer);
    if (isApplying) {
      if (!untrack(() => visible)) shownAt = performance.now();
      visible = true;
      return;
    }
    // Gone before it faded in: drop it now. Once seen, hold it so a batch does not flicker between items.
    seen = performance.now() - shownAt >= delay("reveal");
    hideTimer = setTimeout(() => (visible = false), seen ? delay("settle") : 0);
  });

  $effect(() => () => clearTimeout(hideTimer));

  // Never seen: leave at once, or the outro would overlap the inner layer's delayed fade-in.
  const exit = (node: Element) => (seen ? fade(node, { speed: "fast" }) : { duration: 0 });
</script>

{#if visible}
  <!-- Blocks input at once; the inner layer only shows if the work outlasts the reveal delay. -->
  <div class="fixed inset-x-0 top-titlebar bottom-0 z-busy" role="presentation" aria-busy="true" out:exit>
    <div class="flex h-full animate-reveal items-center justify-center bg-black/40 p-4">
      <div class="w-full max-w-sm rounded-lg border border-border bg-elevated px-6 py-5 shadow-dialog">
        <div class="flex items-center gap-3">
          <span class="inline-flex animate-spin text-accent">
            <Icon icon="mdi:loading" width="24" class="text-accent" />
          </span>
          <div class="min-w-0">
            <div class="text-base font-semibold text-foreground">Changing system settings…</div>
            <div class="mt-0.5 text-sm text-foreground-muted">Please wait and do not close the app.</div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
