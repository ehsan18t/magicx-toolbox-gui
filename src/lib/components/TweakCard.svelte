<script lang="ts">
  import { loadingStore, systemStore, toggleTweak } from "$lib/stores/tweaks";
  import type { RegistryChange, RiskLevel, TweakWithStatus } from "$lib/types";
  import { RISK_INFO } from "$lib/types";
  import Icon from "@iconify/svelte";
  import { derived, get } from "svelte/store";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  const { tweak } = $props<{
    tweak: TweakWithStatus;
  }>();

  const isLoading = derived(loadingStore, ($loading) => $loading.has(tweak.definition.id));

  let showDetails = $state(false);
  let showConfirmDialog = $state(false);

  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level as RiskLevel]);
  const isHighRisk = $derived(
    tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical",
  );

  // Risk level to icon mapping
  const riskIcons: Record<RiskLevel, string> = {
    low: "mdi:check-circle",
    medium: "mdi:alert",
    high: "mdi:alert-circle",
    critical: "mdi:alert-octagon",
  };

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

<article class="tweak-card" class:applied={tweak.status.is_applied}>
  <div class="status-bar" class:active={tweak.status.is_applied}></div>

  <div class="card-content">
    <!-- Main row: title, badges, toggle -->
    <div class="main-row">
      <div class="info-section">
        <div class="title-row">
          <h3 class="tweak-title">{tweak.definition.name}</h3>
        </div>

        <!-- Meta row: risk, admin, reboot -->
        <div class="meta-row">
          <!-- Risk level with icon -->
          <div class="meta-item risk" title={riskInfo.description}>
            <Icon icon={riskIcons[tweak.definition.risk_level as RiskLevel]} width="14" />
            <span class="risk-label {tweak.definition.risk_level}">{riskInfo.name}</span>
          </div>

          <!-- Admin badge -->
          {#if tweak.definition.requires_admin}
            <div class="meta-item badge admin" title="Requires Administrator privileges to apply">
              <Icon icon="mdi:shield-account" width="14" />
              <span>Admin</span>
            </div>
          {/if}

          <!-- Reboot badge -->
          {#if tweak.definition.requires_reboot}
            <div
              class="meta-item badge reboot"
              title="System restart required after applying or reverting"
            >
              <Icon icon="mdi:restart" width="14" />
              <span>Reboot</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Toggle Switch -->
      <button
        class="toggle-switch"
        class:active={tweak.status.is_applied}
        class:loading={$isLoading}
        disabled={$isLoading}
        onclick={handleToggleClick}
        aria-label={tweak.status.is_applied ? "Revert tweak" : "Apply tweak"}
        role="switch"
        aria-checked={tweak.status.is_applied}
      >
        <span class="switch-track">
          <span class="switch-thumb">
            {#if $isLoading}
              <Icon icon="mdi:loading" width="14" class="spin" />
            {:else if tweak.status.is_applied}
              <Icon icon="mdi:check" width="14" />
            {/if}
          </span>
        </span>
      </button>
    </div>

    <!-- Description -->
    <p class="description">{tweak.definition.description}</p>

    <!-- Details toggle -->
    <button
      class="details-toggle"
      onclick={() => (showDetails = !showDetails)}
      aria-expanded={showDetails}
    >
      <span>{showDetails ? "Hide details" : "Show details"}</span>
      <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
    </button>

    <!-- Details section -->
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
  </div>
</article>

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
    border-radius: 10px;
    overflow: hidden;
    transition: all 0.2s ease;
  }

  .tweak-card:hover {
    border-color: hsl(var(--border-hover, var(--border)));
    box-shadow: 0 2px 8px hsla(0, 0%, 0%, 0.06);
  }

  .tweak-card.applied {
    border-color: hsl(var(--primary) / 0.4);
    background: hsl(var(--primary) / 0.03);
  }

  /* Status bar */
  .status-bar {
    width: 3px;
    flex-shrink: 0;
    background: hsl(var(--muted));
    transition: background 0.2s ease;
  }

  .status-bar.active {
    background: hsl(var(--primary));
  }

  /* Card content */
  .card-content {
    flex: 1;
    padding: 14px 16px;
    min-width: 0;
  }

  /* Main row */
  .main-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .info-section {
    flex: 1;
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

  /* Meta row for risk, admin, reboot */
  .meta-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-top: 6px;
  }

  .meta-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
  }

  .meta-item.risk {
    color: hsl(var(--foreground));
    cursor: help;
  }

  .meta-item.risk :global(svg) {
    flex-shrink: 0;
  }

  .meta-item.badge {
    padding: 4px 8px;
    border-radius: 6px;
    color: white;
    cursor: help;
    transition: all 0.15s ease;
  }

  .meta-item.badge:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 6px hsla(0, 0%, 0%, 0.15);
  }

  .meta-item.badge.admin {
    background: hsl(217 91% 60%);
  }

  .meta-item.badge.reboot {
    background: hsl(280 68% 60%);
  }

  .meta-item.badge :global(svg) {
    flex-shrink: 0;
  }

  .risk-label {
    font-size: 12px;
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

  /* Toggle Switch */
  .toggle-switch {
    flex-shrink: 0;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
  }

  .toggle-switch:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }

  .switch-track {
    display: flex;
    align-items: center;
    width: 44px;
    height: 24px;
    padding: 2px;
    background: hsl(var(--muted));
    border-radius: 12px;
    transition: background 0.2s ease;
  }

  .toggle-switch:hover:not(:disabled) .switch-track {
    background: hsl(var(--muted-foreground) / 0.3);
  }

  .toggle-switch.active .switch-track {
    background: hsl(var(--primary));
  }

  .toggle-switch.active:hover:not(:disabled) .switch-track {
    background: hsl(var(--primary) / 0.85);
  }

  .switch-thumb {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 1px 3px hsla(0, 0%, 0%, 0.2);
    transition: transform 0.2s ease;
    color: hsl(var(--primary));
  }

  .toggle-switch.active .switch-thumb {
    transform: translateX(20px);
  }

  .toggle-switch.loading .switch-thumb {
    color: hsl(var(--muted-foreground));
  }

  /* Description */
  .description {
    margin: 10px 0 0;
    font-size: 13px;
    line-height: 1.5;
    color: hsl(var(--muted-foreground));
  }

  /* Details toggle */
  .details-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 10px;
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
    margin-top: 12px;
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
