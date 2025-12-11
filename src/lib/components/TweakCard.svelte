<script lang="ts">
  import {
    errorStore,
    loadingStore,
    pendingChangesStore,
    stageChange,
    systemStore,
    unstageChange,
  } from "$lib/stores/tweaks";
  import type { RegistryChange, RiskLevel, TweakWithStatus } from "$lib/types";
  import { RISK_INFO } from "$lib/types";
  import Icon from "@iconify/svelte";
  import { derived } from "svelte/store";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  const { tweak } = $props<{
    tweak: TweakWithStatus;
  }>();

  const isLoading = derived(loadingStore, ($loading) => $loading.has(tweak.definition.id));
  const tweakError = derived(errorStore, ($errors) => $errors.get(tweak.definition.id));

  let showDetails = $state(false);
  let showConfirmDialog = $state(false);

  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level as RiskLevel]);
  const isHighRisk = $derived(
    tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical",
  );

  // Risk level config
  const riskConfig: Record<RiskLevel, { icon: string; color: string }> = {
    low: { icon: "mdi:check-circle", color: "text-success" },
    medium: { icon: "mdi:alert", color: "text-warning" },
    high: { icon: "mdi:alert-circle", color: "text-[hsl(25_95%_53%)]" },
    critical: { icon: "mdi:alert-octagon", color: "text-error" },
  };

  // Reactively compute registry changes based on store value
  const registryChanges = $derived.by(() => {
    const system = $systemStore;
    if (!system) return [];
    const version = system.windows.is_windows_11 ? 11 : 10;
    return tweak.definition.registry_changes.filter((change: RegistryChange) => {
      if (!change.windows_versions || change.windows_versions.length === 0) {
        return true;
      }
      return change.windows_versions.includes(version);
    });
  });

  // Check if this is a multi-state tweak (has options)
  const isMultiState = $derived.by(() => {
    const firstChange = registryChanges[0];
    return firstChange?.options && firstChange.options.length > 1;
  });

  // Get options for multi-state tweaks
  const options = $derived.by(() => {
    const firstChange = registryChanges[0];
    return firstChange?.options || [];
  });

  // Current option index from registry (actual applied state)
  const currentOptionIndex = $derived(tweak.status.current_option_index ?? 0);

  // Get pending change for this tweak
  const pendingChange = derived(pendingChangesStore, ($pending) =>
    $pending.get(tweak.definition.id),
  );

  // Determine if there's a pending change
  const hasPending = $derived($pendingChange !== undefined);

  // Calculate the effective state (what the user sees in the UI)
  const effectiveEnabled = $derived.by(() => {
    const pending = $pendingChange;
    if (pending?.type === "binary") {
      return pending.enabled;
    }
    return tweak.status.is_applied;
  });

  const effectiveOptionIndex = $derived.by(() => {
    const pending = $pendingChange;
    if (pending?.type === "multistate") {
      return pending.optionIndex;
    }
    return currentOptionIndex;
  });

  function handleToggleClick() {
    if (isHighRisk && !effectiveEnabled) {
      showConfirmDialog = true;
    } else {
      executeToggle();
    }
  }

  function executeToggle() {
    showConfirmDialog = false;
    const newEnabled = !effectiveEnabled;

    if (newEnabled === tweak.status.is_applied) {
      unstageChange(tweak.definition.id);
    } else {
      stageChange(tweak.definition.id, { type: "binary", enabled: newEnabled });
    }
  }

  function handleOptionChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const optionIndex = parseInt(select.value, 10);

    if (optionIndex === currentOptionIndex) {
      unstageChange(tweak.definition.id);
    } else {
      stageChange(tweak.definition.id, { type: "multistate", optionIndex });
    }
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

<article
  class="relative flex overflow-hidden rounded-lg border border-border bg-card transition-all duration-200 hover:border-border-hover hover:shadow-md {tweak
    .status.is_applied
    ? 'border-primary/40 bg-primary/3'
    : ''} {hasPending ? 'border-warning/50 bg-warning/5' : ''}"
>
  <!-- Status bar -->
  <div
    class="w-0.75 shrink-0 transition-colors duration-200 {hasPending
      ? 'bg-warning'
      : tweak.status.is_applied
        ? 'bg-primary'
        : 'bg-[hsl(var(--muted))]'}"
  ></div>

  <div class="min-w-0 flex-1 px-4 py-3.5">
    <!-- Header Section -->
    <div class="mb-2 flex items-center justify-between gap-3">
      <h3
        class="m-0 flex flex-1 items-center gap-2 text-sm leading-tight font-semibold text-foreground"
      >
        {tweak.definition.name}
        {#if hasPending}
          <span
            class="inline-flex rounded bg-warning/15 px-1.5 py-0.5 text-[10px] font-semibold tracking-wide text-warning uppercase"
            >pending</span
          >
        {/if}
      </h3>

      {#if isMultiState}
        <!-- Dropdown for multi-state -->
        <select
          class="max-w-45 min-w-30 shrink-0 cursor-pointer appearance-none rounded-lg border border-border bg-[hsl(var(--muted))] bg-[url('data:image/svg+xml,%3Csvg_xmlns=%27http://www.w3.org/2000/svg%27_width=%2712%27_height=%2712%27_viewBox=%270_0_24_24%27%3E%3Cpath_fill=%27%23888%27_d=%27M7_10l5_5_5-5z%27/%3E%3C/svg%3E')] bg-position-[right_8px_center] bg-no-repeat px-2.5 py-1.5 pr-7 text-xs font-medium text-foreground transition-all duration-200 hover:not-disabled:border-primary focus:border-primary focus:ring-2 focus:ring-primary/20 focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 {hasPending
            ? 'border-warning bg-warning/10'
            : ''} {$isLoading ? 'opacity-70' : ''}"
          disabled={$isLoading}
          value={effectiveOptionIndex}
          onchange={handleOptionChange}
        >
          {#each options as option, i (i)}
            <option value={i}>{option.label}</option>
          {/each}
        </select>
      {:else}
        <!-- Toggle Switch -->
        <button
          class="toggle-switch shrink-0 cursor-pointer border-0 bg-transparent p-0 disabled:cursor-not-allowed disabled:opacity-70"
          class:active={effectiveEnabled}
          class:pending={hasPending}
          disabled={$isLoading}
          onclick={handleToggleClick}
          aria-label={effectiveEnabled ? "Will revert tweak" : "Will apply tweak"}
          role="switch"
          aria-checked={effectiveEnabled}
        >
          <span
            class="switch-track flex h-6 w-11 items-center rounded-full p-0.5 transition-colors duration-200 {effectiveEnabled
              ? hasPending
                ? 'bg-warning'
                : 'bg-primary'
              : 'bg-[hsl(var(--muted))]'} hover:not-disabled:brightness-95"
          >
            <span
              class="switch-thumb flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-md transition-transform duration-200 {effectiveEnabled
                ? 'translate-x-5'
                : 'translate-x-0'} {$isLoading ? 'text-foreground-muted' : 'text-primary'}"
            >
              {#if $isLoading}
                <Icon icon="mdi:loading" width="14" class="animate-spin" />
              {:else if tweak.status.is_applied}
                <Icon icon="mdi:check" width="14" />
              {/if}
            </span>
          </span>
        </button>
      {/if}
    </div>

    <!-- Description -->
    <p class="m-0 mb-2.5 text-sm leading-relaxed text-foreground-muted">
      {tweak.definition.description}
    </p>

    <!-- Error message -->
    {#if $tweakError}
      <div
        class="mb-2.5 flex items-start gap-2 rounded-lg border border-error/30 bg-error/10 px-3 py-2.5 text-xs leading-relaxed text-error"
      >
        <Icon icon="mdi:alert-circle" width="16" class="mt-0.5 shrink-0" />
        <span class="flex-1 wrap-break-word">{$tweakError}</span>
        <button
          class="flex shrink-0 cursor-pointer items-center justify-center rounded border-0 bg-transparent p-0.5 text-error transition-colors duration-150 hover:bg-error/20"
          onclick={() => errorStore.clearError(tweak.definition.id)}
        >
          <Icon icon="mdi:close" width="14" />
        </button>
      </div>
    {/if}

    <!-- Metadata Section -->
    <div class="flex flex-wrap items-center gap-3 border-t border-border/30 pt-2">
      <!-- Risk level -->
      <div
        class="inline-flex cursor-help items-center gap-1.5 text-xs font-medium text-foreground-muted transition-colors duration-150 hover:text-foreground"
        title={riskInfo.description}
      >
        <Icon
          icon={riskConfig[tweak.definition.risk_level as RiskLevel].icon}
          width="16"
          class="opacity-70 {riskConfig[tweak.definition.risk_level as RiskLevel].color}"
        />
        <span
          class="text-xs font-semibold tracking-wide uppercase {riskConfig[
            tweak.definition.risk_level as RiskLevel
          ].color}">{riskInfo.name}</span
        >
      </div>

      <!-- Admin required -->
      {#if tweak.definition.requires_admin}
        <div
          class="inline-flex cursor-help items-center gap-1.5 text-xs font-medium text-foreground-muted transition-colors duration-150 hover:text-foreground"
          title="Requires Administrator privileges to apply"
        >
          <Icon icon="mdi:shield-account-outline" width="16" class="opacity-70" />
          <span class="text-xs font-semibold tracking-wide uppercase">Admin</span>
        </div>
      {/if}

      <!-- Reboot required -->
      {#if tweak.definition.requires_reboot}
        <div
          class="inline-flex cursor-help items-center gap-1.5 text-xs font-medium text-foreground-muted transition-colors duration-150 hover:text-foreground"
          title="System restart required after applying or reverting"
        >
          <Icon icon="mdi:restart" width="16" class="opacity-70" />
          <span class="text-xs font-semibold tracking-wide uppercase">Reboot</span>
        </div>
      {/if}
    </div>

    <!-- Details toggle -->
    <button
      class="mt-2.5 inline-flex cursor-pointer items-center gap-1 rounded-md border-0 bg-transparent px-2 py-1 text-xs text-foreground-muted transition-all duration-150 hover:bg-[hsl(var(--muted)/0.5)] hover:text-foreground"
      onclick={() => (showDetails = !showDetails)}
      aria-expanded={showDetails}
    >
      <span>{showDetails ? "Hide details" : "Show details"}</span>
      <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
    </button>

    <!-- Details section -->
    {#if showDetails}
      <div class="mt-3 border-t border-border/50 pt-3">
        {#if tweak.definition.info}
          <div
            class="mb-3 flex gap-2 rounded-lg bg-[hsl(var(--muted)/0.3)] px-3 py-2.5 text-xs leading-relaxed text-foreground-muted"
          >
            <Icon icon="mdi:information-outline" width="14" class="shrink-0" />
            <p class="m-0 flex-1">{tweak.definition.info}</p>
          </div>
        {/if}

        <div class="mt-2">
          <h4
            class="m-0 mb-2.5 flex items-center gap-1.5 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
          >
            <Icon icon="mdi:database-cog-outline" width="14" />
            Registry Modifications
            <span
              class="ml-1 inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
              >{registryChanges.length}</span
            >
          </h4>

          <div class="flex flex-col gap-2">
            {#each registryChanges as change (change.hive + change.key + change.value_name)}
              <div class="overflow-hidden rounded-lg border border-border/60 bg-background">
                <div
                  class="flex items-center gap-1.5 border-b border-border/40 bg-[hsl(var(--muted)/0.3)] px-2.5 py-2 text-foreground-muted"
                >
                  <Icon icon="mdi:key-variant" width="12" />
                  <code class="bg-transparent p-0 font-mono text-[10px] break-all text-primary"
                    >{formatRegistryPath(change)}</code
                  >
                </div>
                <div class="px-2.5 py-2">
                  <div class="mb-1.5 flex items-center gap-2">
                    <span class="font-mono text-xs font-semibold text-foreground"
                      >{change.value_name || "(Default)"}</span
                    >
                    <span
                      class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 font-mono text-[9px] text-foreground-muted"
                      >{change.value_type}</span
                    >
                  </div>
                  <div class="flex flex-wrap gap-2">
                    <div class="flex items-center gap-1.5 text-xs">
                      <span
                        class="rounded bg-success/15 px-1.5 py-0.5 text-[9px] font-bold text-success uppercase"
                        >ON</span
                      >
                      <code class="bg-transparent p-0 font-mono text-[10px] text-foreground/80"
                        >{formatValue(change.enable_value)}</code
                      >
                    </div>
                    {#if change.disable_value !== undefined}
                      <div class="flex items-center gap-1.5 text-xs">
                        <span
                          class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-bold text-foreground-muted uppercase"
                          >OFF</span
                        >
                        <code class="bg-transparent p-0 font-mono text-[10px] text-foreground/80"
                          >{formatValue(change.disable_value)}</code
                        >
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
