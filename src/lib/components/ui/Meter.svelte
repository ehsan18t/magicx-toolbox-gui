<script lang="ts" module>
  const HEIGHT = { sm: "h-1", md: "h-1.5" };
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import { toPercent } from "./percent";
  import type { MeterSegment } from "./types";

  interface Props {
    value: number;
    max?: number;
    label: string;
    /** Stacked parts of `max` in place of the single bar, e.g. a breakdown by state. */
    segments?: MeterSegment[];
    size?: keyof typeof HEIGHT;
    class?: string;
  }

  let { value, max = 100, label, segments, size = "sm", class: className }: Props = $props();
</script>

<span
  role="meter"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={max}
  aria-valuenow={Math.min(max, Math.max(0, value))}
  class={cn("flex overflow-hidden rounded-full bg-muted", HEIGHT[size], className)}
>
  {#if segments}
    {#each segments as segment (segment.key)}
      <span
        class="transition-[width] duration-slower ease-out {segment.fill}"
        style:width="{toPercent(segment.value, max)}%"
      ></span>
    {/each}
  {:else}
    <span
      class="rounded-full bg-accent transition-[width] duration-slower ease-out"
      style:width="{toPercent(value, max)}%"
    ></span>
  {/if}
</span>
