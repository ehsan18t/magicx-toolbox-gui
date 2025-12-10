<script lang="ts">
  import { categoriesMap, loadingStore, systemStore, toggleTweak } from "$lib/stores/tweaks";
  import type { RegistryChange, RiskLevel, TweakWithStatus } from "$lib/types";
  import { RISK_INFO } from "$lib/types";
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
  const categoryInfo = $derived(get(categoriesMap)[tweak.definition.category]);
  const isHighRisk = $derived(
    tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical",
  );

  // Get registry changes for current Windows version
  const registryChanges = $derived(() => {
    const system = get(systemStore);
    if (!system) return [];
    const version = system.windows.is_windows_11 ? 11 : 10;
    return tweak.definition.registry_changes.filter((change: RegistryChange) => {
      if (!change.windows_versions || change.windows_versions.length === 0) {
        return true;
      }
      return change.windows_versions.includes(version);
    });
  });

  function handleToggleClick() {
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

<article class="tweak-card" class:compact class:applied={tweak.status.is_applied}>
  <!-- Status indicator bar -->
  <div class="status-bar" class:active={tweak.status.is_applied}></div>

  <div class="card-content">
    <!-- Header section -->
    <header class="card-header">
      <div class="header-left">
        <div class="icon-wrapper {tweak.definition.risk_level}">
          <span class="category-icon">{categoryInfo?.icon || "📦"}</span>
        </div>
        <div class="title-section">
          <div class="title-row">
            <h3 class="tweak-title">{tweak.definition.name}</h3>
            <div class="badges">
              {#if tweak.definition.requires_admin}
                <span class="badge admin" title="Requires Administrator">
                  <Icon icon="mdi:shield-account" width="11" />
                </span>
              {/if}
              {#if tweak.definition.requires_reboot}
                <span class="badge reboot" title="Requires Reboot">
                  <Icon icon="mdi:restart" width="11" />
                </span>
              {/if}
            </div>
          </div>
          <span class="risk-label {tweak.definition.risk_level}">{riskInfo.name}</span>
        </div>
      </div>

      <div class="header-right">
        <button
          class="toggle-button"
          class:active={tweak.status.is_applied}
          class:loading={$isLoading}
          disabled={$isLoading}
          onclick={handleToggleClick}
          aria-label={tweak.status.is_applied ? "Revert tweak" : "Apply tweak"}
        >
          {#if $isLoading}
            <Icon icon="mdi:loading" width="18" class="spin" />
          {:else if tweak.status.is_applied}
            <Icon icon="mdi:check" width="18" />
          {:else}
            <Icon icon="mdi:power" width="18" />
          {/if}
        </button>
      </div>
    </header>

    <!-- Description -->
    {#if !compact}
      <p class="description">{tweak.definition.description}</p>
    {/if}

    <!-- Details toggle & section -->
    {#if !compact}
      <button
        class="details-toggle"
        onclick={() => (showDetails = !showDetails)}
        aria-expanded={showDetails}
      >
        <span class="toggle-text">{showDetails ? "Hide details" : "Show details"}</span>
        <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
      </button>

      {#if showDetails}
        <div class="details-section">
          {#if tweak.definition.info}
            <div class="info-box">
              <Icon icon="mdi:information-outline" width="14" />
              <p>{tweak.definition.info}</p>
            </div>
          {/if}

          <div class="registry-section">
            <h4 class="section-title">
              <Icon icon="mdi:database-cog-outline" width="14" />
              Registry Modifications
              <span class="count">{registryChanges().length}</span>
            </h4>

            <div class="registry-list">
              {#each registryChanges() as change (change.hive + change.key + change.value_name)}
                <div class="registry-item">
                  <div class="registry-header">
                    <Icon icon="mdi:key-variant" width="12" />
                    <code class="registry-path">{formatRegistryPath(change)}</code>
                  </div>
                  <div class="registry-body">
                    <div class="value-row">
                      <span class="value-name">{change.value_name || "(Default)"}</span>
                      <span class="value-type">{change.value_type}</span>
                    </div>
                    <div class="value-changes">
                      <div class="change-item enable">
                        <span class="change-label">ON</span>
                        <code>{formatValue(change.enable_value)}</code>
                      </div>
                      {#if change.disable_value !== undefined}
                        <div class="change-item disable">
                          <span class="change-label">OFF</span>
                          <code>{formatValue(change.disable_value)}</code>
                        </div>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</article>

<!-- Confirmation dialog for high-risk tweaks -->
<ConfirmDialog
  open={showConfirmDialog}
  title="Apply High-Risk Tweak?"
  message="This tweak is marked as {tweak.definition
    .risk_level} risk. {riskInfo.description} Are you sure you want to apply it?"
  confirmText="Yes, Apply"
  cancelText="Cancel"
  onconfirm={executeToggle}
  oncancel={() => (showConfirmDialog = false)}
/>

<style>
  .tweak-card {
    position: relative;
    display: flex;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 12px;
    overflow: hidden;
    transition: all 0.2s ease;
  }

  .tweak-card:hover {
    border-color: hsl(var(--border-hover, var(--border)));
    box-shadow: 0 4px 12px hsla(0, 0%, 0%, 0.08);
  }

  .tweak-card.applied {
    border-color: hsl(var(--primary) / 0.3);
    background: hsl(var(--primary) / 0.02);
  }

  .tweak-card.compact {
    border-radius: 8px;
  }

  /* Status indicator bar on the left */
  .status-bar {
    width: 4px;
    flex-shrink: 0;
    background: hsl(var(--muted));
    transition: background 0.2s ease;
  }

  .status-bar.active {
    background: hsl(var(--primary));
  }

  /* Main content area */
  .card-content {
    flex: 1;
    padding: 16px;
    min-width: 0;
  }

  .compact .card-content {
    padding: 12px;
  }

  /* Header */
  .card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .header-left {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    min-width: 0;
    flex: 1;
  }

  .icon-wrapper {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 10px;
    background: hsl(var(--muted));
    flex-shrink: 0;
    font-size: 18px;
  }

  .compact .icon-wrapper {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    font-size: 14px;
  }

  .icon-wrapper.low {
    background: hsl(142 76% 36% / 0.12);
  }
  .icon-wrapper.medium {
    background: hsl(48 96% 53% / 0.15);
  }
  .icon-wrapper.high {
    background: hsl(25 95% 53% / 0.12);
  }
  .icon-wrapper.critical {
    background: hsl(0 84% 60% / 0.12);
  }

  .title-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .tweak-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: hsl(var(--foreground));
    line-height: 1.3;
  }

  .compact .tweak-title {
    font-size: 13px;
  }

  .badges {
    display: flex;
    gap: 4px;
  }

  .badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 5px;
    color: white;
  }

  .badge.admin {
    background: hsl(217 91% 60%);
  }

  .badge.reboot {
    background: hsl(280 68% 60%);
  }

  .risk-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .risk-label.low {
    color: hsl(142 76% 36%);
  }
  .risk-label.medium {
    color: hsl(48 96% 40%);
  }
  .risk-label.high {
    color: hsl(25 95% 53%);
  }
  .risk-label.critical {
    color: hsl(0 84% 60%);
  }

  /* Toggle button */
  .header-right {
    flex-shrink: 0;
  }

  .toggle-button {
    width: 42px;
    height: 42px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid hsl(var(--border));
    border-radius: 10px;
    background: hsl(var(--background));
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .compact .toggle-button {
    width: 36px;
    height: 36px;
    border-radius: 8px;
  }

  .toggle-button:hover:not(:disabled) {
    border-color: hsl(var(--primary));
    color: hsl(var(--primary));
    background: hsl(var(--primary) / 0.05);
  }

  .toggle-button.active {
    background: hsl(var(--primary));
    border-color: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
  }

  .toggle-button.active:hover:not(:disabled) {
    background: hsl(var(--primary) / 0.9);
  }

  .toggle-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .toggle-button.loading {
    cursor: wait;
  }

  /* Description */
  .description {
    margin: 12px 0 0;
    padding-left: 52px;
    font-size: 13px;
    line-height: 1.5;
    color: hsl(var(--muted-foreground));
  }

  /* Details toggle */
  .details-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin: 12px 0 0 52px;
    padding: 4px 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: hsl(var(--muted-foreground));
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .details-toggle:hover {
    background: hsl(var(--muted) / 0.5);
    color: hsl(var(--foreground));
  }

  /* Details section */
  .details-section {
    margin: 12px 0 0 52px;
    padding-top: 12px;
    border-top: 1px solid hsl(var(--border) / 0.5);
  }

  .info-box {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    margin-bottom: 12px;
    background: hsl(var(--muted) / 0.3);
    border-radius: 8px;
    font-size: 12px;
    line-height: 1.5;
    color: hsl(var(--muted-foreground));
  }

  .info-box p {
    margin: 0;
    flex: 1;
  }

  .registry-section {
    margin-top: 8px;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 0 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: hsl(var(--muted-foreground));
  }

  .section-title .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    margin-left: 4px;
    background: hsl(var(--muted));
    border-radius: 9px;
    font-size: 10px;
    font-weight: 600;
    color: hsl(var(--foreground));
  }

  .registry-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .registry-item {
    background: hsl(var(--background));
    border: 1px solid hsl(var(--border) / 0.6);
    border-radius: 8px;
    overflow: hidden;
  }

  .registry-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    background: hsl(var(--muted) / 0.3);
    border-bottom: 1px solid hsl(var(--border) / 0.4);
    color: hsl(var(--muted-foreground));
  }

  .registry-path {
    font-size: 10px;
    font-family: "Consolas", "Monaco", "Fira Code", monospace;
    color: hsl(var(--primary));
    word-break: break-all;
    background: none;
    padding: 0;
  }

  .registry-body {
    padding: 8px 10px;
  }

  .value-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .value-name {
    font-size: 12px;
    font-weight: 600;
    color: hsl(var(--foreground));
    font-family: "Consolas", "Monaco", "Fira Code", monospace;
  }

  .value-type {
    font-size: 9px;
    padding: 2px 6px;
    background: hsl(var(--muted));
    border-radius: 4px;
    color: hsl(var(--muted-foreground));
    font-family: "Consolas", "Monaco", "Fira Code", monospace;
  }

  .value-changes {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .change-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
  }

  .change-label {
    font-size: 9px;
    font-weight: 700;
    padding: 2px 5px;
    border-radius: 3px;
    text-transform: uppercase;
  }

  .change-item.enable .change-label {
    background: hsl(142 76% 36% / 0.15);
    color: hsl(142 76% 36%);
  }

  .change-item.disable .change-label {
    background: hsl(var(--muted));
    color: hsl(var(--muted-foreground));
  }

  .change-item code {
    font-size: 10px;
    font-family: "Consolas", "Monaco", "Fira Code", monospace;
    color: hsl(var(--foreground) / 0.8);
    background: none;
    padding: 0;
  }

  /* Spin animation */
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
