<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { StatusBadge } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
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

  // Favorite state
  const isFavorite = $derived(favoritesStore.isFavorite(tweak.definition.id));

  function toggleFavorite() {
    favoritesStore.toggle(tweak.definition.id);
  }

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
  const showDefaultSegment = $derived(currentOptionIndex === null || snapshotOriginalOptionIndex === null);

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
    if (currentOptionIndex === null) {
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
      icon: "mdi:check-circle",
    });

    // Add Default segment in the middle if needed
    if (showDefaultSegment) {
      segments.push({
        value: -1,
        label: "Default",
        icon: "mdi:restore",
      });
    }

    // Option 1 is last (OFF/Original state)
    segments.push({
      value: 1,
      label: options[1]?.label ?? "OFF",
      icon: "mdi:close-circle-outline",
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
  class="tweak-card group relative flex overflow-hidden rounded-lg border transition-all duration-200
    {hasPending
    ? 'border-warning/40 bg-warning/3'
    : tweak.status.is_applied
      ? 'border-accent/30 bg-accent/3'
      : 'border-border bg-card hover:border-border-hover'}
    {isHighlighting ? 'tweak-highlight' : ''}"
>
  <!-- Status indicator -->
  <div
    class="absolute top-0 left-0 h-full w-1 transition-colors duration-200 {hasPending
      ? 'bg-warning'
      : tweak.status.is_applied
        ? 'bg-accent'
        : 'group-hover:bg-muted bg-transparent'}"
  ></div>

  <div class="flex min-w-0 flex-1 flex-col px-3 pt-2.5 pb-2">
    <!-- Header: Title + Control -->
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0 flex-1">
        <h3 class="m-0 flex flex-wrap items-center gap-2 text-[13px] leading-tight font-semibold text-foreground">
          {#if titleSlot}
            {@render titleSlot()}
          {:else}
            {tweak.definition.name}
          {/if}
          {#if hasDetectionError}
            <span
              class="inline-flex items-center gap-1 rounded-full bg-warning/10 px-2 py-0.5 text-[10px] font-medium tracking-wide text-warning"
              use:tooltip={tweak.status.error}
            >
              <Icon icon="mdi:alert" width="10" />
              Unknown
            </span>
          {/if}
          {#if hasPending}
            <span
              class="inline-flex items-center rounded-full bg-warning/10 px-2 py-0.5 text-[10px] font-medium tracking-wide text-warning"
            >
              Pending
            </span>
          {/if}
        </h3>

        <!-- Description -->
        <p class="m-0 mt-1.5 mb-1.5 text-[12px] leading-relaxed text-foreground-muted/80">
          {#if descriptionSlot}
            {@render descriptionSlot()}
          {:else}
            {tweak.definition.description}
          {/if}
        </p>
      </div>

      <!-- Control -->
      <div class="shrink-0 pt-0.5">
        {#if !isToggle}
          <Select
            value={effectiveOptionIndex ?? -1}
            options={selectOptions}
            pending={hasPending}
            loading={isLoading}
            disabled={isLoading}
            onchange={handleSelectChange}
          />
        {:else}
          <SegmentedSwitch
            value={effectiveSegmentValue}
            options={segmentOptions}
            pending={hasPending}
            loading={isLoading}
            disabled={isLoading}
            iconOnly
            onchange={handleSegmentChange}
          />
        {/if}
      </div>
    </div>

    <!-- Error message -->
    {#if tweakError}
      <div
        class="mt-3 flex items-start gap-2 rounded-lg border border-error/20 bg-error/5 px-3 py-2 text-xs leading-relaxed text-error"
      >
        <Icon icon="mdi:alert-circle" width="16" class="mt-0.5 shrink-0" />
        <span class="flex-1 wrap-break-word">{tweakError}</span>
        <button
          class="flex shrink-0 cursor-pointer items-center justify-center rounded border-0 bg-transparent p-0.5 text-error/70 transition-colors duration-150 hover:bg-error/10 hover:text-error"
          onclick={() => errorStore.clearError(tweak.definition.id)}
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      </div>
    {/if}

    <!-- Footer: Metadata + Actions -->
    <div class="mt-auto flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-border/40 pt-2.5">
      <!-- Metadata tags -->
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
        <!-- Risk level -->
        <StatusBadge
          variant={tweak.definition.risk_level === "low"
            ? "success"
            : tweak.definition.risk_level === "medium"
              ? "warning"
              : tweak.definition.risk_level === "high"
                ? "orange"
                : "error"}
          icon={riskConfig[tweak.definition.risk_level as RiskLevel].icon}
          label={riskInfo.name}
          tooltip={riskInfo.description}
        />

        <!-- Permission level -->
        {#if permissionInfo}
          <StatusBadge
            variant="muted"
            icon={permissionInfo.icon}
            label={permissionInfo.name}
            tooltip={permissionInfo.description}
          />
        {/if}

        <!-- Reboot required -->
        {#if tweak.definition.requires_reboot}
          <StatusBadge
            variant="info"
            icon="mdi:restart"
            label="Reboot"
            tooltip="System restart required after applying or reverting"
          />
        {/if}
      </div>

      <!-- Actions -->
      <div class="card-actions flex shrink-0 items-center gap-1" class:has-restore={hasSnapshot}>
        <!-- Favorite button -->
        <button
          type="button"
          class="card-action inline-flex cursor-pointer items-center justify-center rounded-md border-0 bg-transparent p-1.5 transition-all duration-150 {isFavorite
            ? 'text-warning hover:bg-warning/10'
            : 'hover:bg-muted/50 text-foreground-muted hover:text-foreground'}"
          onclick={toggleFavorite}
          aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
          use:tooltip={isFavorite ? "Remove from favorites" : "Add to favorites"}
        >
          <Icon icon={isFavorite ? "mdi:star" : "mdi:star-outline"} width="18" />
        </button>

        {#if hasSnapshot}
          <button
            type="button"
            class="card-action inline-flex cursor-pointer items-center gap-1.5 rounded-md border-0 bg-transparent px-2 py-1 text-[11px] font-medium text-accent transition-all duration-150 hover:bg-accent/10 disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleRestoreClick}
            disabled={isLoading}
            aria-label="Restore snapshot"
            use:tooltip={"Restore to original state from snapshot"}
          >
            <Icon icon="mdi:history" width="18" class="card-action-icon" />
            <span class="card-action-label">Restore</span>
          </button>
        {/if}

        <button
          type="button"
          class="card-action hover:bg-muted/50 inline-flex cursor-pointer items-center gap-1.5 rounded-md border-0 bg-transparent px-2 py-1 text-[11px] font-medium text-foreground-muted transition-all duration-150 hover:text-foreground"
          onclick={() => openTweakDetailsModal(tweak.definition.id)}
          aria-label="Open tweak details"
          use:tooltip={"Details"}
        >
          <span class="card-action-label">Details</span>
          <Icon icon="mdi:chevron-right" width="18" class="card-action-icon" />
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

  /* Subtle shadow on hover for depth */
  .tweak-card:hover {
    box-shadow: 0 2px 8px -2px rgba(0, 0, 0, 0.08);
  }

  /* Dark theme shadow adjustment */
  :global([data-theme="dark"]) .tweak-card:hover {
    box-shadow: 0 2px 12px -2px rgba(0, 0, 0, 0.3);
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
      transform: scale(1.1);
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
      transform: scale(1.1);
      transform-origin: center;
    }
  }
</style>
