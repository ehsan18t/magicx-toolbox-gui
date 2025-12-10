<script lang="ts">
  import type { SystemInfo } from "$lib/types";
  import Icon from "@iconify/svelte";

  const { systemInfo } = $props<{
    systemInfo: SystemInfo | null;
  }>();
</script>

<div class="system-info-card">
  {#if systemInfo}
    <div class="info-grid">
      <div class="info-item">
        <Icon icon="mdi:microsoft-windows" width="20" class="info-icon" />
        <div class="info-content">
          <span class="info-label">Windows</span>
          <span class="info-value">{systemInfo.windows.product_name}</span>
        </div>
      </div>

      <div class="info-item">
        <Icon icon="mdi:update" width="20" class="info-icon" />
        <div class="info-content">
          <span class="info-label">Version</span>
          <span class="info-value"
            >{systemInfo.windows.display_version} (Build {systemInfo.windows.build_number})</span
          >
        </div>
      </div>

      <div class="info-item">
        <Icon icon="mdi:account" width="20" class="info-icon" />
        <div class="info-content">
          <span class="info-label">User</span>
          <span class="info-value">{systemInfo.username}@{systemInfo.computer_name}</span>
        </div>
      </div>

      <div class="info-item">
        <Icon
          icon={systemInfo.is_admin ? "mdi:shield-check" : "mdi:shield-alert"}
          width="20"
          class="info-icon {systemInfo.is_admin ? 'admin' : 'not-admin'}"
        />
        <div class="info-content">
          <span class="info-label">Privileges</span>
          <span class="info-value {systemInfo.is_admin ? 'admin' : 'not-admin'}">
            {systemInfo.is_admin ? "Administrator" : "Standard User"}
          </span>
        </div>
      </div>
    </div>
  {:else}
    <div class="loading">
      <Icon icon="mdi:loading" width="24" class="spin" />
      <span>Loading system info...</span>
    </div>
  {/if}
</div>

<style>
  .system-info-card {
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    padding: 16px;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
  }

  .info-item {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  :global(.info-icon) {
    color: hsl(var(--primary));
    flex-shrink: 0;
    margin-top: 2px;
  }

  :global(.info-icon.admin) {
    color: hsl(142 76% 36%);
  }

  :global(.info-icon.not-admin) {
    color: hsl(45 93% 47%);
  }

  .info-content {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .info-label {
    font-size: 11px;
    font-weight: 500;
    color: hsl(var(--muted-foreground));
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .info-value {
    font-size: 13px;
    font-weight: 500;
    color: hsl(var(--foreground));
    word-break: break-word;
  }

  .info-value.admin {
    color: hsl(142 76% 36%);
  }

  .info-value.not-admin {
    color: hsl(45 93% 47%);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    color: hsl(var(--muted-foreground));
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
