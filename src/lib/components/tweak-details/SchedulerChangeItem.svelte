<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import { Badge } from "$lib/components/ui";
  import type { SchedulerChange } from "$lib/types";

  interface Props {
    change: SchedulerChange;
  }

  const { change }: Props = $props();

  function schedulerTarget(c: SchedulerChange): string {
    if (c.task_name) return `${c.task_path}\\${c.task_name}`;
    if (c.task_name_pattern) return `${c.task_path}\\(pattern: ${c.task_name_pattern})`;
    return c.task_path;
  }
</script>

<div class="rounded-lg border border-border/60 bg-background px-3 py-2">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <Icon icon="mdi:calendar" width="14" class="text-foreground-muted" />
        <code class="bg-transparent p-0 font-mono text-[10px] break-all text-foreground">
          {schedulerTarget(change)}
        </code>
      </div>
      <div class="mt-1 flex flex-wrap items-center gap-2">
        <Badge size="sm" variant="default">action: {change.action}</Badge>
        {#if change.ignore_not_found}
          <Badge size="sm" variant="default">ignore_not_found</Badge>
        {/if}
        {#if change.skip_validation}
          <Badge size="sm" variant="default">skip_validation</Badge>
        {/if}
      </div>
    </div>
  </div>
</div>
