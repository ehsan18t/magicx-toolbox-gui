<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Badge } from "$lib/components/ui";
  import type { ServiceChange } from "$lib/types";

  interface Props {
    change: ServiceChange;
  }

  const { change }: Props = $props();
</script>

<div class="rounded-lg border border-border/60 bg-background px-3 py-2">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <div class="flex items-center gap-2">
      <Icon icon="mdi:server" width="14" class="text-foreground-muted" />
      <span class="font-mono text-xs font-semibold text-foreground">{change.name}</span>
    </div>
    <div class="flex items-center gap-2">
      <Badge size="sm" variant="default">startup: {change.startup}</Badge>
      {#if change.skip_validation}
        <Badge size="sm" variant="default">skip_validation</Badge>
      {/if}
    </div>
  </div>
</div>
