<script lang="ts">
  import { loadingStore, toggleTweak } from "$lib/stores/tweaks";
  import type { RiskLevel, TweakCategory, TweakWithStatus } from "$lib/types";
  import { CATEGORY_INFO, RISK_INFO } from "$lib/types";
  import Icon from "@iconify/svelte";
  import { derived } from "svelte/store";

  const { tweak, compact = false } = $props<{
    tweak: TweakWithStatus;
    compact?: boolean;
  }>();

  const isLoading = derived(loadingStore, ($loading) => $loading.has(tweak.definition.id));

  let showDetails = $state(false);

  // Use $derived to make these reactive
  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level as RiskLevel]);
  const categoryInfo = $derived(CATEGORY_INFO[tweak.definition.category as TweakCategory]);

  async function handleToggle() {
    await toggleTweak(tweak.definition.id, tweak.status.is_applied);
  }
</script>

<div class="tweak-card" class:compact class:applied={tweak.status.is_applied}>
  <div class="card-header">
    <div class="card-info">
      <div class="title-row">
        <span class="category-icon" title={categoryInfo.name}>{categoryInfo.icon}</span>
        <h3 class="tweak-name">{tweak.definition.name}</h3>
        {#if tweak.definition.requires_admin}
          <span class="badge admin" title="Requires Administrator">
            <Icon icon="mdi:shield-account" width="12" />
          </span>
        {/if}
        {#if tweak.definition.requires_reboot}
          <span class="badge reboot" title="Requires Reboot">
            <Icon icon="mdi:restart" width="12" />
          </span>
        {/if}
      </div>
      {#if !compact}
        <p class="tweak-description">{tweak.definition.description}</p>
      {/if}
    </div>

    <div class="card-actions">
      <span class="risk-badge {tweak.definition.risk_level}" title={riskInfo.description}>
        {riskInfo.name}
      </span>
      <button
        class="toggle-btn"
        class:active={tweak.status.is_applied}
        disabled={$isLoading}
        onclick={handleToggle}
        title={tweak.status.is_applied ? "Click to revert" : "Click to apply"}
      >
        {#if $isLoading}
          <Icon icon="mdi:loading" width="18" class="spin" />
        {:else}
          <Icon
            icon={tweak.status.is_applied ? "mdi:check-circle" : "mdi:circle-outline"}
            width="18"
          />
        {/if}
      </button>
    </div>
  </div>

  {#if !compact && tweak.definition.info}
    <button class="details-toggle" onclick={() => (showDetails = !showDetails)}>
      <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
      {showDetails ? "Hide details" : "Show details"}
    </button>

    {#if showDetails}
      <div class="details">
        <p>{tweak.definition.info}</p>
      </div>
    {/if}
  {/if}
</div>

<style>
  .tweak-card {
    background: var(--card-bg, hsl(var(--background)));
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    padding: 12px 16px;
    transition: all 0.2s ease;
  }

  .tweak-card:hover {
    border-color: hsl(var(--primary) / 0.5);
    box-shadow: 0 2px 8px hsl(var(--primary) / 0.1);
  }

  .tweak-card.applied {
    background: hsl(var(--primary) / 0.05);
    border-color: hsl(var(--primary) / 0.3);
  }

  .tweak-card.compact {
    padding: 8px 12px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .card-info {
    flex: 1;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .category-icon {
    font-size: 16px;
  }

  .tweak-name {
    font-size: 14px;
    font-weight: 600;
    color: hsl(var(--foreground));
    margin: 0;
  }

  .compact .tweak-name {
    font-size: 13px;
  }

  .tweak-description {
    font-size: 12px;
    color: hsl(var(--muted-foreground));
    margin: 4px 0 0 0;
    line-height: 1.4;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 4px;
    border-radius: 4px;
    font-size: 10px;
  }

  .badge.admin {
    background: hsl(var(--warning) / 0.2);
    color: hsl(var(--warning));
  }

  .badge.reboot {
    background: hsl(var(--info) / 0.2);
    color: hsl(var(--info));
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .risk-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    text-transform: uppercase;
  }

  .risk-badge.low {
    background: hsl(142 76% 36% / 0.2);
    color: hsl(142 76% 36%);
  }

  .risk-badge.medium {
    background: hsl(45 93% 47% / 0.2);
    color: hsl(45 93% 47%);
  }

  .risk-badge.high {
    background: hsl(24 94% 50% / 0.2);
    color: hsl(24 94% 50%);
  }

  .risk-badge.critical {
    background: hsl(0 84% 60% / 0.2);
    color: hsl(0 84% 60%);
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 6px;
    background: hsl(var(--muted));
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn:hover:not(:disabled) {
    background: hsl(var(--accent));
  }

  .toggle-btn.active {
    background: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
  }

  .toggle-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .details-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 8px;
    padding: 4px 0;
    border: none;
    background: none;
    color: hsl(var(--muted-foreground));
    font-size: 11px;
    cursor: pointer;
    transition: color 0.2s ease;
  }

  .details-toggle:hover {
    color: hsl(var(--foreground));
  }

  .details {
    margin-top: 8px;
    padding: 8px 12px;
    background: hsl(var(--muted) / 0.5);
    border-radius: 6px;
    font-size: 12px;
    color: hsl(var(--muted-foreground));
    line-height: 1.5;
  }

  .details p {
    margin: 0;
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
