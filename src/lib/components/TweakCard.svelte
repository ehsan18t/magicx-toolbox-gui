<script lang="ts">
  import { searchStore } from "$lib/stores/search.svelte";
  import { openTweakDetailsModal } from "$lib/stores/tweakDetailsModal.svelte";
  import { errorStore, loadingStore, pendingChangesStore, stageChange, unstageChange } from "$lib/stores/tweaks.svelte";
  import type { RiskLevel, TweakWithStatus } from "$lib/types";
  import { RISK_INFO } from "$lib/types";
  import type { Snippet } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    tweak: TweakWithStatus;
    /** Optional slot for custom title rendering (e.g., with highlights) */
    titleSlot?: Snippet;
    /** Optional slot for custom description rendering (e.g., with highlights) */
    descriptionSlot?: Snippet;
  }

  const { tweak, titleSlot, descriptionSlot }: Props = $props();

  const isLoading = $derived(loadingStore.isLoading(tweak.definition.id));
  const tweakError = $derived(errorStore.getError(tweak.definition.id));
  // Detection error from backend (status couldn't be determined)
  const hasDetectionError = $derived(!!tweak.status.error);

  // Highlight state for search navigation
  const shouldHighlight = $derived(searchStore.highlightTweakId === tweak.definition.id);
  let isHighlighting = $state(false);

  // Handle highlight animation
  $effect(() => {
    if (!shouldHighlight) return;

    isHighlighting = true;
    // Clear highlight after animation
    const timer = setTimeout(() => {
      isHighlighting = false;
      searchStore.clearHighlight();
    }, 1500);
    return () => clearTimeout(timer);
  });

  let showConfirmDialog = $state(false);

  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level as RiskLevel]);
  const isHighRisk = $derived(tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical");

  // Risk level config
  const riskConfig: Record<RiskLevel, { icon: string; color: string }> = {
    low: { icon: "mdi:check-circle", color: "text-success" },
    medium: { icon: "mdi:alert", color: "text-warning" },
    high: { icon: "mdi:alert-circle", color: "text-orange-500" },
    critical: { icon: "mdi:alert-octagon", color: "text-error" },
  };

  // Get options from tweak definition
  const options = $derived(tweak.definition.options);

  // Check if this is a toggle (2 options) or dropdown (3+ options)
  const isToggle = $derived(tweak.definition.is_toggle);

  // Current option index from registry (actual applied state, null if no match = system default)
  const currentOptionIndex = $derived(tweak.status.current_option_index);

  // Get pending change for this tweak
  const pendingChange = $derived(pendingChangesStore.get(tweak.definition.id));

  // Determine if there's a pending change
  const hasPending = $derived(pendingChange !== undefined);

  // Calculate the effective state for toggle (what the user sees in the UI)
  // For toggles: option 0 = enabled, option 1 = disabled
  const effectiveEnabled = $derived.by(() => {
    if (pendingChange !== undefined) {
      return pendingChange.optionIndex === 0;
    }
    // If no current option matches (system default), treat as disabled
    return currentOptionIndex === 0;
  });

  // Calculate effective option index for dropdowns
  const effectiveOptionIndex = $derived.by(() => {
    if (pendingChange !== undefined) {
      return pendingChange.optionIndex;
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
    // For toggles: option 0 = enabled, option 1 = disabled
    const newOptionIndex = newEnabled ? 0 : 1;

    if (newOptionIndex === currentOptionIndex) {
      unstageChange(tweak.definition.id);
    } else {
      stageChange(tweak.definition.id, { tweakId: tweak.definition.id, optionIndex: newOptionIndex });
    }
  }

  function handleOptionChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const optionIndex = parseInt(select.value, 10);

    // Guard against invalid values
    if (isNaN(optionIndex)) return;

    if (optionIndex === currentOptionIndex) {
      unstageChange(tweak.definition.id);
    } else {
      stageChange(tweak.definition.id, { tweakId: tweak.definition.id, optionIndex });
    }
  }
</script>

<article
  id="tweak-{tweak.definition.id}"
  class="relative flex overflow-hidden rounded-lg border border-border bg-card transition-all duration-200 hover:border-border-hover hover:shadow-md {tweak
    .status.is_applied
    ? 'border-accent/40 bg-accent/5'
    : ''} {hasPending ? 'border-warning/50 bg-warning/5' : ''} {isHighlighting ? 'tweak-highlight' : ''}"
>
  <!-- Status bar -->
  <div
    class="w-0.75 shrink-0 transition-colors duration-200 {hasPending
      ? 'bg-warning'
      : tweak.status.is_applied
        ? 'bg-accent'
        : 'bg-muted'}"
  ></div>

  <div class="flex min-w-0 flex-1 flex-col gap-2 px-4 py-3.5">
    <!-- Header Section -->
    <div class="mb-2 flex items-center justify-between gap-3">
      <h3 class="m-0 flex flex-1 items-center gap-2 text-sm leading-tight font-semibold text-foreground">
        {#if titleSlot}
          {@render titleSlot()}
        {:else}
          {tweak.definition.name}
        {/if}
        {#if hasDetectionError}
          <span
            class="inline-flex items-center gap-1 rounded bg-warning/15 px-1.5 py-0.5 text-[10px] font-semibold tracking-wide text-warning uppercase"
            title={tweak.status.error}
          >
            <Icon icon="mdi:alert" width="10" />
            unknown state
          </span>
        {/if}
        {#if hasPending}
          <span
            class="inline-flex rounded bg-warning/15 px-1.5 py-0.5 text-[10px] font-semibold tracking-wide text-warning uppercase"
            >pending</span
          >
        {/if}
      </h3>

      {#if !isToggle}
        <!-- Dropdown for multi-option tweaks -->
        <select
          class="bg-muted max-w-45 min-w-30 shrink-0 cursor-pointer appearance-none rounded-lg border border-border bg-[url('data:image/svg+xml,%3Csvg_xmlns=%27http://www.w3.org/2000/svg%27_width=%2712%27_height=%2712%27_viewBox=%270_0_24_24%27%3E%3Cpath_fill=%27%23888%27_d=%27M7_10l5_5_5-5z%27/%3E%3C/svg%3E')] bg-position-[right_8px_center] bg-no-repeat px-2.5 py-1.5 pr-7 text-xs font-medium text-foreground transition-all duration-200 hover:not-disabled:border-accent focus:border-accent focus:ring-2 focus:ring-accent/20 focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 {hasPending
            ? 'border-warning bg-warning/10'
            : ''} {isLoading ? 'opacity-70' : ''}"
          disabled={isLoading}
          value={effectiveOptionIndex ?? -1}
          onchange={handleOptionChange}
        >
          {#if currentOptionIndex === null}
            <option value={-1} disabled>System Default</option>
          {/if}
          {#each options as option, i (option.label)}
            <option value={i}>{option.label}</option>
          {/each}
        </select>
      {:else}
        <!-- Toggle Switch -->
        <button
          type="button"
          class="toggle-switch shrink-0 cursor-pointer border-0 bg-transparent p-0 disabled:cursor-not-allowed disabled:opacity-70"
          class:active={effectiveEnabled}
          class:pending={hasPending}
          disabled={isLoading}
          onclick={handleToggleClick}
          aria-label={effectiveEnabled ? "Will revert tweak" : "Will apply tweak"}
          role="switch"
          aria-checked={effectiveEnabled}
        >
          <span
            class="switch-track flex h-6 w-11 items-center rounded-full border-2 transition-colors duration-200 {effectiveEnabled
              ? hasPending
                ? 'border-warning bg-warning'
                : 'border-accent bg-accent'
              : 'border-border bg-surface'} hover:not-disabled:brightness-95"
          >
            <span
              class="switch-thumb flex h-4.5 w-4.5 items-center justify-center rounded-full bg-white shadow-md transition-transform duration-200 {effectiveEnabled
                ? 'translate-x-5'
                : 'translate-x-0.5'} {isLoading ? 'text-foreground-muted' : 'text-accent'}"
            >
              {#if isLoading}
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
    <p class="m-0 mb-2.5 grow text-sm leading-relaxed text-foreground-muted">
      {#if descriptionSlot}
        {@render descriptionSlot()}
      {:else}
        {tweak.definition.description}
      {/if}
    </p>

    <!-- Error message -->
    {#if tweakError}
      <div
        class="mb-2.5 flex items-start gap-2 rounded-lg border border-error/30 bg-error/10 px-3 py-2.5 text-xs leading-relaxed text-error"
      >
        <Icon icon="mdi:alert-circle" width="16" class="mt-0.5 shrink-0" />
        <span class="flex-1 wrap-break-word">{tweakError}</span>
        <button
          class="flex shrink-0 cursor-pointer items-center justify-center rounded border-0 bg-transparent p-0.5 text-error transition-colors duration-150 hover:bg-error/20"
          onclick={() => errorStore.clearError(tweak.definition.id)}
        >
          <Icon icon="mdi:close" width="14" />
        </button>
      </div>
    {/if}

    <!-- Metadata Section -->
    <div class="flex items-center justify-between border-t border-border/30 pt-2">
      <div class="flex items-center gap-3">
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
            class="text-xs font-semibold tracking-wide uppercase {riskConfig[tweak.definition.risk_level as RiskLevel]
              .color}">{riskInfo.name}</span
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

        <!-- SYSTEM elevation required -->
        {#if tweak.definition.requires_system}
          <div
            class="inline-flex cursor-help items-center gap-1.5 text-xs font-medium text-accent transition-colors duration-150 hover:text-foreground"
            title="Requires SYSTEM elevation for protected registry keys and services"
          >
            <Icon icon="mdi:shield-lock" width="16" class="opacity-90" />
            <span class="text-xs font-semibold tracking-wide uppercase">System</span>
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

      <!-- Details (modal) -->
      <button
        type="button"
        class="hover:bg-muted/50 inline-flex cursor-pointer items-center gap-1 rounded-md border-0 bg-transparent px-2 py-1 text-xs text-foreground-muted transition-all duration-150 hover:text-foreground"
        onclick={() => openTweakDetailsModal(tweak.definition.id)}
        aria-label="Open tweak details"
      >
        <span>Details</span>
        <Icon icon="mdi:open-in-new" width="16" />
      </button>
    </div>
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
