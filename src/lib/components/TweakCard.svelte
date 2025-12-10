<script lang="ts">
  import { loadingStore, systemStore, toggleTweak } from "$lib/stores/tweaks";
  import type { RegistryChange, RiskLevel, TweakCategory, TweakWithStatus } from "$lib/types";
  import { CATEGORY_INFO, RISK_INFO } from "$lib/types";
  import Icon from "@iconify/svelte";
  import { derived, get } from "svelte/store";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  const { tweak, compact = false } = $props<{
    tweak: TweakWithStatus;
    compact?: boolean;
  }>();

  const isLoading = derived(loadingStore, ($loading) => $loading.has(tweak.definition.id));

  let showDetails = $state(false);
  let showConfirmDialog = $state(false);

  // Use $derived to make these reactive
  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level as RiskLevel]);
  const categoryInfo = $derived(CATEGORY_INFO[tweak.definition.category as TweakCategory]);
  const isHighRisk = $derived(
    tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical",
  );

  // Get registry changes for current Windows version
  const registryChanges = $derived(() => {
    const system = get(systemStore);
    if (!system) return [];
    const version = system.windows.is_windows_11 ? "11" : "10";
    return tweak.definition.registry_changes[version] || [];
  });

  function handleToggleClick() {
    // Show confirmation for high/critical risk tweaks when applying (not reverting)
    if (isHighRisk && !tweak.status.is_applied) {
      showConfirmDialog = true;
    } else {
      executeToggle();
    }
  }

  async function executeToggle() {
    showConfirmDialog = false;
    await toggleTweak(tweak.definition.id, tweak.status.is_applied);
  }

  function formatRegistryPath(change: RegistryChange): string {
    return `${change.hive}\\${change.key}`;
  }

  function formatValue(value: unknown): string {
    if (value === null || value === undefined) return "(delete)";
    if (typeof value === "number") return `0x${value.toString(16).toUpperCase()} (${value})`;
    if (typeof value === "string") return value === "" ? '""' : `"${value}"`;
    return JSON.stringify(value);
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
        onclick={handleToggleClick}
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

  {#if !compact}
    <button class="details-toggle" onclick={() => (showDetails = !showDetails)}>
      <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
      {showDetails ? "Hide details" : "Show details"}
    </button>

    {#if showDetails}
      <div class="details">
        {#if tweak.definition.info}
          <p class="info-text">{tweak.definition.info}</p>
        {/if}

        <div class="registry-section">
          <h4 class="registry-title">
            <Icon icon="mdi:database-cog" width="14" />
            Registry Changes
          </h4>
          <div class="registry-changes">
            {#each registryChanges() as change}
              <div class="registry-change">
                <div class="registry-path">
                  <Icon icon="mdi:folder-key" width="12" />
                  <code>{formatRegistryPath(change)}</code>
                </div>
                <div class="registry-value">
                  <span class="value-name">{change.value_name || "(Default)"}</span>
                  <span class="value-type">[{change.value_type}]</span>
                </div>
                <div class="registry-values-row">
                  <span class="value-label">Enable:</span>
                  <code class="value-data">{formatValue(change.enable_value)}</code>
                  {#if change.disable_value !== undefined}
                    <span class="value-label">Disable:</span>
                    <code class="value-data">{formatValue(change.disable_value)}</code>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<ConfirmDialog
  open={showConfirmDialog}
  title="Apply {tweak.definition.risk_level === 'critical' ? 'Critical' : 'High'} Risk Tweak"
  message="'{tweak.definition.name}' is a {tweak.definition
    .risk_level} risk tweak. {riskInfo.description}. Are you sure you want to apply it?"
  confirmText="Apply Anyway"
  variant={tweak.definition.risk_level === "critical" ? "danger" : "warning"}
  onconfirm={executeToggle}
  oncancel={() => (showConfirmDialog = false)}
/>

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

  .details .info-text {
    margin: 0 0 12px 0;
  }

  .registry-section {
    border-top: 1px solid hsl(var(--border) / 0.5);
    padding-top: 8px;
    margin-top: 8px;
  }

  .registry-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    color: hsl(var(--foreground) / 0.8);
    margin: 0 0 8px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .registry-changes {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .registry-change {
    background: hsl(var(--background));
    border: 1px solid hsl(var(--border) / 0.5);
    border-radius: 4px;
    padding: 8px;
    font-family: "Consolas", "Monaco", monospace;
  }

  .registry-path {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    color: hsl(var(--primary));
    margin-bottom: 4px;
    word-break: break-all;
  }

  .registry-path code {
    background: none;
    padding: 0;
  }

  .registry-value {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    margin-bottom: 4px;
  }

  .value-name {
    font-weight: 600;
    color: hsl(var(--foreground));
  }

  .value-type {
    font-size: 9px;
    color: hsl(var(--muted-foreground));
    background: hsl(var(--muted));
    padding: 1px 4px;
    border-radius: 3px;
  }

  .registry-values-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    font-size: 10px;
  }

  .value-label {
    color: hsl(var(--muted-foreground));
  }

  .value-data {
    background: hsl(var(--muted) / 0.8);
    padding: 2px 6px;
    border-radius: 3px;
    color: hsl(142 76% 36%);
    font-size: 10px;
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
