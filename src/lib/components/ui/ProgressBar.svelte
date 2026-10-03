<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import { toPercent } from "./percent";

  interface Props {
    value: number;
    max?: number;
    label: string;
    showValue?: boolean;
    class?: string;
  }

  let { value, max = 100, label, showValue = false, class: className }: Props = $props();

  const percent = $derived(toPercent(value, max));
</script>

<div class={cn("flex w-full items-center gap-3", className)}>
  <div
    class="relative h-4 flex-1 overflow-hidden rounded-full bg-muted"
    role="progressbar"
    aria-valuenow={Math.round(percent)}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label={label}
  >
    <div
      class="absolute inset-y-0 left-0 rounded-full bg-accent transition-[width] duration-slower ease-out"
      style:width="{percent}%"
    ></div>
  </div>
  {#if showValue}
    <span class="min-w-12 text-right text-sm font-medium text-foreground-muted tabular-nums"
      >{Math.round(percent)}%</span
    >
  {/if}
</div>
