<script lang="ts">
  import type { SystemInfo } from "$lib/types";
  import Icon from "./Icon.svelte";

  const { systemInfo } = $props<{
    systemInfo: SystemInfo | null;
  }>();
</script>

<div class="rounded-lg border border-border bg-card p-4">
  {#if systemInfo}
    <div class="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
      <div class="flex items-start gap-2.5">
        <Icon icon="mdi:microsoft-windows" width="20" class="mt-0.5 shrink-0 text-accent" />
        <div class="flex min-w-0 flex-col">
          <span class="text-xs font-medium tracking-wide text-foreground-muted uppercase">Windows</span>
          <span class="text-sm font-medium wrap-break-word text-foreground">{systemInfo.windows.product_name}</span>
        </div>
      </div>

      <div class="flex items-start gap-2.5">
        <Icon icon="mdi:update" width="20" class="mt-0.5 shrink-0 text-accent" />
        <div class="flex min-w-0 flex-col">
          <span class="text-xs font-medium tracking-wide text-foreground-muted uppercase">Version</span>
          <span class="text-sm font-medium wrap-break-word text-foreground"
            >{systemInfo.windows.display_version} (Build {systemInfo.windows.build_number})</span
          >
        </div>
      </div>

      <div class="flex items-start gap-2.5">
        <Icon icon="mdi:account" width="20" class="mt-0.5 shrink-0 text-accent" />
        <div class="flex min-w-0 flex-col">
          <span class="text-xs font-medium tracking-wide text-foreground-muted uppercase">User</span>
          <span class="text-sm font-medium wrap-break-word text-foreground"
            >{systemInfo.username}@{systemInfo.computer_name}</span
          >
        </div>
      </div>

      <div class="flex items-start gap-2.5">
        <Icon
          icon={systemInfo.is_admin ? "mdi:shield-check" : "mdi:shield-alert"}
          width="20"
          class="mt-0.5 shrink-0 {systemInfo.is_admin ? 'text-success' : 'text-warning'}"
        />
        <div class="flex min-w-0 flex-col">
          <span class="text-xs font-medium tracking-wide text-foreground-muted uppercase">Privileges</span>
          <span class="text-sm font-medium wrap-break-word {systemInfo.is_admin ? 'text-success' : 'text-warning'}">
            {systemInfo.is_admin ? "Administrator" : "Standard User"}
          </span>
        </div>
      </div>
    </div>
  {:else}
    <div class="flex items-center justify-center gap-2 p-4 text-foreground-muted">
      <Icon icon="mdi:loading" width="24" class="animate-spin" />
      <span>Loading system info...</span>
    </div>
  {/if}
</div>
