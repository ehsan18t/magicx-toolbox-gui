<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Badge } from "$lib/components/ui";
  import type { HostsChange } from "$lib/types";

  interface Props {
    change: HostsChange;
  }

  let { change }: Props = $props();

  const actionVariant = $derived(change.action === "add" ? "success" : "error");
  const actionIcon = $derived(change.action === "add" ? "mdi:plus-circle" : "mdi:minus-circle");
</script>

<div class="rounded-lg border border-border/60 bg-background px-3 py-2">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <div class="flex items-center gap-2">
      <Icon icon="mdi:file-document-outline" width="14" class="text-foreground-muted" />
      <span class="font-mono text-xs text-foreground">{change.domain}</span>
      <Icon icon="mdi:arrow-right" width="12" class="text-foreground-muted" />
      <span class="font-mono text-xs text-accent">{change.ip}</span>
    </div>
    <div class="flex items-center gap-2">
      <Badge size="sm" variant={actionVariant}>
        <Icon icon={actionIcon} width="12" class="mr-1" />
        {change.action}
      </Badge>
      {#if change.comment}
        <Badge size="sm" variant="default" class="max-w-32 truncate">{change.comment}</Badge>
      {/if}
      {#if change.skip_validation}
        <Badge size="sm" variant="default">skip_validation</Badge>
      {/if}
    </div>
  </div>
</div>
