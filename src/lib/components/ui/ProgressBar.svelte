<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import ActivityBar from "./ActivityBar.svelte";
  import { toPercent } from "./percent";

  interface Props {
    /** Null runs the bar indeterminate. */
    value: number | null;
    max?: number | undefined;
    label: string;
    showValue?: boolean;
    class?: string;
  }

  let { value, max = 100, label, showValue = false, class: className }: Props = $props();

  const percent = $derived(value === null ? null : toPercent(value, max));
  // Floored, as the taskbar's: a bar never reads 100% before the work is done.
  const shown = $derived(percent === null ? null : Math.floor(percent));
</script>

<div class={cn("flex w-full items-center gap-3", className)}>
  <div
    class="relative h-4 flex-1 overflow-hidden rounded-full bg-muted"
    role="progressbar"
    aria-valuenow={shown ?? undefined}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label={label}
  >
    {#if percent === null}
      <ActivityBar class="h-full" />
    {:else}
      <div
        class="absolute inset-y-0 left-0 rounded-full bg-accent transition-[width] duration-slower ease-out"
        style:width="{percent}%"
      ></div>
    {/if}
  </div>
  {#if showValue && shown !== null}
    <span class="min-w-12 text-right text-sm font-medium text-foreground-muted tabular-nums">{shown}%</span>
  {/if}
</div>
