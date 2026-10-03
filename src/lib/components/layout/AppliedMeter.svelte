<script lang="ts">
  import { Meter, type MeterSegment } from "$lib/components/ui";
  import { cn } from "$lib/utils/cn";

  interface Props {
    applied: number;
    total: number;
    /** A per-state breakdown in place of the plain bar. */
    segments?: MeterSegment[];
    class?: string;
  }

  let { applied, total, segments, class: className }: Props = $props();
</script>

<div class={cn("flex flex-col gap-1.5", className)}>
  <div class="flex items-baseline justify-between text-xs">
    <span class="text-foreground-muted">Applied</span>
    <span class="font-semibold tabular-nums">{applied} of {total}</span>
  </div>
  <Meter value={applied} max={total} label="Applied" {segments} size={segments ? "md" : "sm"} />
</div>
