<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Badge } from "$lib/components/ui";
  import type { FirewallChange } from "$lib/types";

  interface Props {
    change: FirewallChange;
  }

  let { change }: Props = $props();

  const isCreate = $derived(change.operation === "create");
  const actionVariant = $derived(change.action === "block" ? "error" : "success");
  const directionIcon = $derived(change.direction === "inbound" ? "mdi:arrow-down-bold" : "mdi:arrow-up-bold");
</script>

<div class="rounded-lg border border-border/60 bg-background px-3 py-2">
  <div class="flex flex-col gap-2">
    <!-- Rule Name and Operation -->
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex items-center gap-2">
        <Icon icon="mdi:shield-outline" width="14" class="text-foreground-muted" />
        <span class="font-mono text-xs font-semibold text-foreground">{change.name}</span>
      </div>
      <div class="flex items-center gap-2">
        {#if isCreate}
          <Badge size="sm" variant="info">create</Badge>
          {#if change.direction}
            <Badge size="sm" variant="default">
              <Icon icon={directionIcon} width="12" class="mr-1" />
              {change.direction}
            </Badge>
          {/if}
          {#if change.action}
            <Badge size="sm" variant={actionVariant}>{change.action}</Badge>
          {/if}
        {:else}
          <Badge size="sm" variant="error">delete</Badge>
        {/if}
        {#if change.skip_validation}
          <Badge size="sm" variant="default">skip_validation</Badge>
        {/if}
      </div>
    </div>

    <!-- Additional Details (for create) -->
    {#if isCreate && (change.protocol || change.program || change.service || change.remote_addresses || change.remote_ports)}
      <div class="flex flex-wrap gap-1.5 pl-5">
        {#if change.protocol && change.protocol !== "any"}
          <Badge size="sm" variant="default">protocol: {change.protocol}</Badge>
        {/if}
        {#if change.program}
          <Badge size="sm" variant="default" class="max-w-48 truncate">program: {change.program}</Badge>
        {/if}
        {#if change.service}
          <Badge size="sm" variant="default">service: {change.service}</Badge>
        {/if}
        {#if change.remote_addresses && change.remote_addresses.length > 0}
          <Badge size="sm" variant="default" class="max-w-48 truncate">
            remote: {change.remote_addresses.join(", ")}
          </Badge>
        {/if}
        {#if change.remote_ports}
          <Badge size="sm" variant="default">ports: {change.remote_ports}</Badge>
        {/if}
      </div>
    {/if}
  </div>
</div>
