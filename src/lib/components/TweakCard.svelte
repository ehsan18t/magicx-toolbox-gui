<script lang="ts">
  import { searchStore } from "$lib/stores/search.svelte";
  import { openTweakDetailsModal } from "$lib/stores/tweakDetailsModal.svelte";
  import {
    errorStore,
    loadingStore,
    pendingChangesStore,
    revertTweak,
    stageChange,
    unstageChange,
  } from "$lib/stores/tweaks.svelte";
  import type { RiskLevel, TweakWithStatus } from "$lib/types";
  import { getHighestPermission, PERMISSION_INFO, RISK_INFO } from "$lib/types";
  import type { Snippet } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import { SegmentedSwitch, Select } from "./ui";
  import type { SegmentOption } from "./ui/SegmentedSwitch.svelte";

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
  let showRestoreConfirmDialog = $state(false);

  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level as RiskLevel]);
  const isHighRisk = $derived(tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical");

  // Get highest permission level (hierarchy: ti > system > admin > none)
  const highestPermission = $derived(getHighestPermission(tweak.definition));
  const permissionInfo = $derived(highestPermission !== "none" ? PERMISSION_INFO[highestPermission] : null);

  // Risk level config
  const riskConfig: Record<RiskLevel, { icon: string; color: string }> = {
    low: { icon: "mdi:check-circle", color: "text-success" },
    medium: { icon: "mdi:alert", color: "text-warning" },
    high: { icon: "mdi:alert-circle", color: "text-orange-500" },
    critical: { icon: "mdi:alert-octagon", color: "text-error" },
  };

  // Has a snapshot that can be restored
  const hasSnapshot = $derived(tweak.status.has_backup);

  // Get options from tweak definition
  const options = $derived(tweak.definition.options);

  // Check if this is a toggle (2 options and not forced dropdown) or dropdown (3+ options or forced)
  const isToggle = $derived(tweak.definition.options.length === 2 && !tweak.definition.force_dropdown);

  // Current option index from registry (actual applied state, null/undefined if no match = system default)
  const currentOptionIndex = $derived(tweak.status.current_option_index);

  // Original option index from snapshot (undefined = no snapshot, null = unknown original, number = known original)
  const snapshotOriginalOptionIndex = $derived(tweak.status.snapshot_original_option_index);

  // Determine if we should show the "Default" segment in segmented switch
  // Show when: current state is unknown OR snapshot exists with unknown original state
  const showDefaultSegment = $derived(
    currentOptionIndex === null || currentOptionIndex === undefined || snapshotOriginalOptionIndex === null,
  );

  // Get pending change for this tweak
  const pendingChange = $derived(pendingChangesStore.get(tweak.definition.id));

  // Determine if there's a pending change
  const hasPending = $derived(pendingChange !== undefined);

  // Calculate effective value for segmented switch
  // -1 = Default/System, 0 = ON (option 0), 1 = OFF (option 1)
  const effectiveSegmentValue = $derived.by(() => {
    if (pendingChange !== undefined) {
      return pendingChange.optionIndex;
    }
    // If current state is unknown, show as Default (-1)
    if (currentOptionIndex === null || currentOptionIndex === undefined) {
      return -1;
    }
    return currentOptionIndex;
  });

  // Build segment options for segmented switch
  const segmentOptions = $derived.by(() => {
    const segments: SegmentOption[] = [];

    // Option 0 is always first (ON/Applied state)
    segments.push({
      value: 0,
      label: options[0]?.label ?? "ON",
      icon: "mdi:check",
    });

    // Add Default segment in the middle if needed
    if (showDefaultSegment) {
      segments.push({
        value: -1,
        label: "Default",
      });
    }

    // Option 1 is last (OFF/Original state)
    segments.push({
      value: 1,
      label: options[1]?.label ?? "OFF",
    });

    return segments;
  });

  // Calculate effective option index for dropdowns
  const effectiveOptionIndex = $derived.by(() => {
    if (pendingChange !== undefined) {
      return pendingChange.optionIndex;
    }
    return currentOptionIndex;
  });

  // Build options for Select component
  const selectOptions = $derived.by(() => {
    const opts: { value: number; label: string; disabled?: boolean }[] = [];

    // Add "System Default" placeholder if current state is unknown
    if (currentOptionIndex === null) {
      opts.push({ value: -1, label: "System Default", disabled: true });
    }

    // Add actual options
    options.forEach((option, i) => {
      opts.push({ value: i, label: option.label });
    });

    return opts;
  });

  // Track pending high-risk action for confirmation
  let pendingHighRiskValue: number | null = $state(null);

  function handleSegmentChange(newValue: number) {
    // Handle "Default" selection - just unstage any pending change
    if (newValue === -1) {
      unstageChange(tweak.definition.id);
      return;
    }

    // Check for high-risk confirmation (only when enabling option 0)
    if (isHighRisk && newValue === 0 && effectiveSegmentValue !== 0) {
      pendingHighRiskValue = newValue;
      showConfirmDialog = true;
      return;
    }

    executeSegmentChange(newValue);
  }

  function executeSegmentChange(newValue: number) {
    showConfirmDialog = false;
    pendingHighRiskValue = null;

    // If selecting current state, unstage
    if (newValue === currentOptionIndex) {
      unstageChange(tweak.definition.id);
    } else {
      stageChange(tweak.definition.id, { tweakId: tweak.definition.id, optionIndex: newValue });
    }
  }

  function handleConfirmHighRisk() {
    if (pendingHighRiskValue !== null) {
      executeSegmentChange(pendingHighRiskValue);
    }
  }

  function handleSelectChange(value: string | number) {
    const optionIndex = typeof value === "number" ? value : parseInt(value, 10);

    // Guard against invalid values
    if (isNaN(optionIndex)) return;

    if (optionIndex === currentOptionIndex) {
      unstageChange(tweak.definition.id);
    } else {
      stageChange(tweak.definition.id, { tweakId: tweak.definition.id, optionIndex });
    }
  }

  function handleRestoreClick() {
    if (isHighRisk) {
      showRestoreConfirmDialog = true;
    } else {
      executeRestore();
    }
  }

  async function executeRestore() {
    showRestoreConfirmDialog = false;
    await revertTweak(tweak.definition.id, {
      showToast: true,
      tweakName: tweak.definition.name,
    });
  }
</script>

<article
  id="tweak-{tweak.definition.id}"
  class="tweak-card relative flex overflow-hidden rounded-lg border border-border bg-card transition-all duration-200 hover:border-border-hover hover:shadow-md {tweak
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
        <Select
          value={effectiveOptionIndex ?? -1}
          options={selectOptions}
          pending={hasPending}
          loading={isLoading}
          disabled={isLoading}
          onchange={handleSelectChange}
        />
      {:else}
        <!-- Segmented Switch for toggle tweaks -->
        <SegmentedSwitch
          value={effectiveSegmentValue}
          options={segmentOptions}
          pending={hasPending}
          loading={isLoading}
          disabled={isLoading}
          onchange={handleSegmentChange}
        />
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
    <div class="flex flex-wrap items-center gap-2 border-t border-border/30 pt-2">
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-3">
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

        <!-- Permission level (only show highest: ti > system > admin) -->
        {#if permissionInfo}
          <div
            class="inline-flex cursor-help items-center gap-1.5 text-xs font-medium {permissionInfo.colorClass} transition-colors duration-150 hover:text-foreground"
            title={permissionInfo.description}
          >
            <Icon icon={permissionInfo.icon} width="16" class="opacity-80" />
            <span class="text-xs font-semibold tracking-wide uppercase">{permissionInfo.name}</span>
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
      <div class="card-actions ml-auto flex shrink-0 items-center gap-2" class:has-restore={hasSnapshot}>
        <!-- Restore Snapshot Button - only shown when snapshot exists -->
        {#if hasSnapshot}
          <button
            type="button"
            class="card-action inline-flex cursor-pointer items-center gap-1 rounded-md border-0 bg-transparent px-2 py-1 text-xs text-accent transition-all duration-150 hover:bg-accent/10 hover:text-accent disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleRestoreClick}
            disabled={isLoading}
            aria-label="Restore snapshot"
            title="Restore to original state from snapshot"
          >
            <Icon icon="mdi:history" width="16" class="card-action-icon" />
            <span class="card-action-label">Restore</span>
          </button>
        {/if}

        <button
          type="button"
          class="card-action hover:bg-muted/50 inline-flex cursor-pointer items-center gap-1 rounded-md border-0 bg-transparent px-2 py-1 text-xs text-foreground-muted transition-all duration-150 hover:text-foreground"
          onclick={() => openTweakDetailsModal(tweak.definition.id)}
          aria-label="Open tweak details"
          title="Details"
        >
          <span class="card-action-label">Details</span>
          <Icon icon="mdi:open-in-new" width="16" class="card-action-icon" />
        </button>
      </div>
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
  onconfirm={handleConfirmHighRisk}
  oncancel={() => {
    showConfirmDialog = false;
    pendingHighRiskValue = null;
  }}
/>

<ConfirmDialog
  open={showRestoreConfirmDialog}
  title="Restore Snapshot?"
  message="This will restore the original state from before the tweak was applied. {tweak.definition.requires_reboot
    ? 'A system restart may be required.'
    : ''}"
  confirmText="Restore"
  cancelText="Cancel"
  onconfirm={executeRestore}
  oncancel={() => (showRestoreConfirmDialog = false)}
/>

<style>
  .tweak-card {
    container-type: inline-size;
  }

  /*
    Labels should be visible by default.
    Collapse to icon-only only when the card is tight.
    Cards with BOTH actions (Restore + Details) need more room, so collapse earlier.
  */

  @container (max-width: 430px) {
    .card-actions.has-restore .card-action {
      gap: 0;
      padding-inline: 0.375rem;
    }

    .card-actions.has-restore .card-action-label {
      display: none;
    }

    .card-actions.has-restore :global(.card-action-icon) {
      transform: scale(1.2);
      transform-origin: center;
    }
  }

  @container (max-width: 360px) {
    .card-action {
      gap: 0;
      padding-inline: 0.375rem;
    }

    .card-action-label {
      display: none;
    }

    :global(.card-action-icon) {
      transform: scale(1.2);
      transform-origin: center;
    }
  }
</style>
