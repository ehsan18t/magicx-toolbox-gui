<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { Select, StatusBadge, Switch } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { searchStore } from "$lib/stores/search.svelte";
  import { openTweakDetailsModal } from "$lib/stores/tweakDetailsModal.svelte";
  import {
    discardSnapshots,
    errorStore,
    loadingStore,
    pendingChangesStore,
    revertTweak,
    stageChange,
    unstageChange,
  } from "$lib/stores/tweaks.svelte";
  import type { RiskLevel, TweakWithStatus } from "$lib/types";
  import { PERMISSION_INFO, permissionFromElevation, RISK_INFO } from "$lib/types";
  import type { Snippet } from "svelte";

  interface Props {
    tweak: TweakWithStatus;
    /** Optional slot for custom title rendering (e.g., with highlights) */
    titleSlot?: Snippet;
    /** Optional slot for custom description rendering (e.g., with highlights) */
    descriptionSlot?: Snippet;
  }

  let { tweak, titleSlot, descriptionSlot }: Props = $props();

  // Sentinel dropdown value for the computed "System Default" position (ADR-0003).
  const SYSTEM_DEFAULT = "__system_default__";

  const isLoading = $derived(loadingStore.isLoading(tweak.definition.id));
  const tweakError = $derived(errorStore.getError(tweak.definition.id));

  const status = $derived(tweak.status);
  const availability = $derived(tweak.definition.availability);

  // --- new engine states ---------------------------------------------------
  const isChecking = $derived(status.state === "loading");
  const isUnknown = $derived(status.state === "unknown");
  const isUnavailable = $derived(status.state === "unavailable");
  const residues = $derived(status.residues);
  const heldShared = $derived(status.heldShared);
  const unknownTip = $derived.by(() => {
    if (!isUnknown) return "";
    const causes = status.unknownReasons.map((r) => `${r.effect}: ${r.cause}`).join("; ");
    const base = `Could not read this tweak's state (${causes || "unknown"}).`;
    return status.needsElevation ? `${base} Restart as administrator to resolve.` : base;
  });

  // Why the control is disabled (availability gate or a detected-unavailable tweak).
  const controlDisabledReason = $derived.by(() => {
    if (isUnavailable) return status.unavailableReason ?? "Not available on this system";
    if (availability.state === "needs_elevation") return availability.reason;
    if (availability.state === "sid_mismatch") return availability.reason;
    return null;
  });
  const controlDisabled = $derived(isLoading || controlDisabledReason !== null);

  // Highlight state for search navigation
  const shouldHighlight = $derived(searchStore.highlightTweakId === tweak.definition.id);
  let isHighlighting = $state(false);

  $effect(() => {
    if (!shouldHighlight) return;
    isHighlighting = true;
    const timer = setTimeout(() => {
      isHighlighting = false;
      searchStore.clearHighlight();
    }, 1500);
    return () => clearTimeout(timer);
  });

  let showConfirmDialog = $state(false);
  let showRestoreConfirmDialog = $state(false);
  let showKeepStateConfirmDialog = $state(false);

  const riskInfo = $derived(RISK_INFO[tweak.definition.risk_level]);
  const isHighRisk = $derived(tweak.definition.risk_level === "high" || tweak.definition.risk_level === "critical");

  // Permission level derived from the declared elevation floor.
  const permission = $derived(permissionFromElevation(tweak.definition.elevation));
  const permissionInfo = $derived(permission !== "none" ? PERMISSION_INFO[permission] : null);

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

  const hasSnapshot = $derived(status.has_backup);

  // Needs Attention (ADR-0001): a restore didn't fully succeed; the snapshot was kept.
  const needsAttention = $derived(status.needs_attention);
  const unrestorableResources = $derived(status.unrestorable_resources);
  const restoreLabel = $derived(needsAttention ? "Retry" : "Restore");

  // --- option shape (1 authored option -> toggle; >=2 -> dropdown) ----------
  const optionLabels = $derived(tweak.definition.optionLabels);
  const isToggle = $derived(optionLabels.length === 1);
  const singleLabel = $derived(optionLabels[0] ?? "On");

  const pendingChange = $derived(pendingChangesStore.get(tweak.definition.id));
  const hasPending = $derived(pendingChange !== undefined);
  const activeOption = $derived(status.activeOption);

  // Toggle: checked when the single option is active (or pending).
  const switchChecked = $derived(
    hasPending ? pendingChange?.optionLabel === singleLabel : activeOption === singleLabel,
  );

  // Dropdown: pending label, else the active option, else the System Default position.
  const selectValue = $derived(pendingChange?.optionLabel ?? activeOption ?? SYSTEM_DEFAULT);
  const selectOptions = $derived.by(() => {
    const opts: { value: string; label: string; disabled?: boolean }[] = [];
    // ADR-0003: "System Default" selectable (-> Revert) only when a snapshot exists;
    // otherwise a display-only placeholder for a state that is already System Default.
    opts.push({ value: SYSTEM_DEFAULT, label: "System Default", disabled: !hasSnapshot });
    for (const label of optionLabels) {
      const un = status.unavailableOptions.find((u) => u.label === label);
      opts.push({ value: label, label: un ? `${label} — unavailable` : label, disabled: !!un });
    }
    return opts;
  });

  // Track a pending high-risk apply for confirmation.
  let pendingHighRiskLabel: string | null = $state(null);

  function stageApply(label: string) {
    if (label === activeOption) {
      unstageChange(tweak.definition.id);
      return;
    }
    if (isHighRisk) {
      pendingHighRiskLabel = label;
      showConfirmDialog = true;
      return;
    }
    stageChange(tweak.definition.id, { tweakId: tweak.definition.id, optionLabel: label });
  }

  function goSystemDefault() {
    unstageChange(tweak.definition.id);
    if (hasSnapshot) handleRestoreClick();
  }

  function handleSwitchChange(checked: boolean) {
    if (checked) {
      stageApply(singleLabel);
    } else {
      goSystemDefault();
    }
  }

  function handleSelectChange(value: string | number) {
    const v = String(value);
    if (v === SYSTEM_DEFAULT) {
      goSystemDefault();
      return;
    }
    stageApply(v);
  }

  function handleConfirmHighRisk() {
    showConfirmDialog = false;
    if (pendingHighRiskLabel !== null) {
      stageChange(tweak.definition.id, { tweakId: tweak.definition.id, optionLabel: pendingHighRiskLabel });
      pendingHighRiskLabel = null;
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
    await revertTweak(tweak.definition.id, { showToast: true, tweakName: tweak.definition.name });
  }

  async function executeDiscard() {
    showKeepStateConfirmDialog = false;
    await discardSnapshots(tweak.definition.id, { showToast: true, tweakName: tweak.definition.name });
  }
</script>

<article
  id="tweak-{tweak.definition.id}"
  class="tweak-card group relative flex overflow-hidden rounded-lg border transition-all duration-200
    {hasPending
    ? 'border-warning/40 bg-warning/3'
    : status.is_applied
      ? 'border-accent/30 bg-accent/3'
      : 'border-border bg-card hover:border-border-hover'}
    {isHighlighting ? 'tweak-highlight' : ''}"
>
  <!-- Status indicator -->
  <div
    class="absolute top-0 left-0 h-full w-1 transition-colors duration-200 {hasPending
      ? 'bg-warning'
      : status.is_applied
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

          {#if isChecking}
            <span
              class="bg-muted/50 inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium tracking-wide text-foreground-muted"
              use:tooltip={"Checking this tweak's current state…"}
            >
              <Icon icon="mdi:loading" width="10" class="animate-spin" />
              Checking
            </span>
          {/if}

          {#if isUnknown}
            <span
              class="inline-flex items-center gap-1 rounded-full bg-warning/10 px-2 py-0.5 text-[10px] font-medium tracking-wide text-warning"
              use:tooltip={unknownTip}
            >
              <Icon icon="mdi:help-circle-outline" width="10" />
              Unknown{status.needsElevation ? " · needs elevation" : ""}
            </span>
          {/if}

          {#if isUnavailable}
            <span
              class="bg-muted/50 inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium tracking-wide text-foreground-muted"
              use:tooltip={status.unavailableReason ?? "Not available on this system"}
            >
              <Icon icon="mdi:cancel" width="10" />
              Unavailable
            </span>
          {/if}

          {#if !isUnavailable && availability.state !== "available"}
            <span
              class="inline-flex items-center gap-1 rounded-full bg-warning/10 px-2 py-0.5 text-[10px] font-medium tracking-wide text-warning"
              use:tooltip={availability.reason}
            >
              <Icon icon="mdi:shield-lock-outline" width="10" />
              {availability.state === "sid_mismatch" ? "SID mismatch" : "Needs elevation"}
            </span>
          {/if}

          {#if residues.length > 0}
            <span
              class="inline-flex items-center gap-1 rounded-full bg-info/10 px-2 py-0.5 text-[10px] font-medium tracking-wide text-info"
              use:tooltip={`Residual settings remain outside the active option: ${residues.join(", ")}`}
            >
              <Icon icon="mdi:information-outline" width="10" />
              Residue
            </span>
          {/if}

          {#if heldShared.length > 0}
            <span
              class="bg-muted/50 inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium tracking-wide text-foreground-muted"
              use:tooltip={`Shared with: ${heldShared.map((h) => `${h.shared} (${h.holders.join(", ")})`).join("; ")}`}
            >
              <Icon icon="mdi:link-variant" width="10" />
              Shared
            </span>
          {/if}

          {#if needsAttention}
            <span
              class="inline-flex items-center gap-1 rounded-full bg-error/10 px-2 py-0.5 text-[10px] font-medium tracking-wide text-error"
              use:tooltip={unrestorableResources.length
                ? `The last restore didn't fully complete: ${unrestorableResources.join("; ")}. The snapshot is kept — retry, or keep the current state.`
                : "The last restore didn't fully succeed. The snapshot is kept for retry."}
            >
              <Icon icon="mdi:alert-circle" width="10" />
              Needs Attention
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
      <div class="shrink-0 pt-0.5" use:tooltip={controlDisabledReason}>
        {#if isToggle}
          <Switch
            checked={switchChecked}
            pending={hasPending}
            loading={isLoading}
            disabled={controlDisabled}
            ariaLabel="Toggle {tweak.definition.name}"
            onchange={handleSwitchChange}
          />
        {:else}
          <Select
            value={selectValue}
            options={selectOptions}
            pending={hasPending}
            loading={isLoading}
            disabled={controlDisabled}
            onchange={handleSelectChange}
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
          aria-label="Dismiss error"
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
          icon={riskConfig[tweak.definition.risk_level].icon}
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
            tooltip="System restart required after applying or restoring"
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
            aria-label={needsAttention ? "Retry restore" : "Restore snapshot"}
            use:tooltip={needsAttention
              ? "Retry restoring the original state"
              : "Restore to original state from snapshot"}
          >
            <Icon icon="mdi:history" width="18" class="card-action-icon" />
            <span class="card-action-label">{restoreLabel}</span>
          </button>
        {/if}

        {#if needsAttention}
          <button
            type="button"
            class="card-action hover:bg-muted/50 inline-flex cursor-pointer items-center gap-1.5 rounded-md border-0 bg-transparent px-2 py-1 text-[11px] font-medium text-foreground-muted transition-all duration-150 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => (showKeepStateConfirmDialog = true)}
            disabled={isLoading}
            aria-label="Keep current state"
            use:tooltip={"Accept the current state and release the snapshot"}
          >
            <Icon icon="mdi:check" width="18" class="card-action-icon" />
            <span class="card-action-label">Keep</span>
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
    pendingHighRiskLabel = null;
  }}
/>

<ConfirmDialog
  open={showRestoreConfirmDialog}
  title="Restore Snapshot?"
  message="This will restore the original state from before the tweak was applied."
  confirmText="Restore"
  cancelText="Cancel"
  onconfirm={executeRestore}
  oncancel={() => (showRestoreConfirmDialog = false)}
/>

<ConfirmDialog
  open={showKeepStateConfirmDialog}
  title="Keep Current State?"
  message="This releases the saved snapshot and accepts the current state as-is. The original state can no longer be restored for this tweak."
  confirmText="Keep current state"
  cancelText="Cancel"
  onconfirm={executeDiscard}
  oncancel={() => (showKeepStateConfirmDialog = false)}
/>

<style>
  .tweak-card {
    container-type: inline-size;
  }

  .tweak-card:hover {
    box-shadow: 0 2px 8px -2px rgba(0, 0, 0, 0.08);
  }

  :global([data-theme="dark"]) .tweak-card:hover {
    box-shadow: 0 2px 12px -2px rgba(0, 0, 0, 0.3);
  }

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
