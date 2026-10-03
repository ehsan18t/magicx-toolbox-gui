<script lang="ts">
  import { Meter } from "$lib/components/ui";
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";

  interface Props {
    applied: number;
    total: number;
    /** Replaces the plain meter, e.g. with a per-state breakdown. */
    bar?: Snippet;
    class?: string;
  }

  let { applied, total, bar, class: className }: Props = $props();
</script>

<div class={cn("flex flex-col gap-1.5", className)}>
  <div class="flex items-baseline justify-between text-xs">
    <span class="text-foreground-muted">Applied</span>
    <span class="font-semibold tabular-nums">{applied} of {total}</span>
  </div>
  {#if bar}
    {@render bar()}
  {:else}
    <Meter value={applied} max={total} label="Applied" />
  {/if}
</div>
